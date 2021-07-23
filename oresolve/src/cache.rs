use std::path::PathBuf;
use std::time::Duration;

use anyhow::anyhow;
use tokio::fs;
use tokio::prelude::*;
use tokio::stream::StreamExt;
use tokio::time::delay_for;
use url::Url;

use crate::GlobalConfig;
use tokio::sync::{Mutex, watch};
use std::collections::HashMap;
use std::hash::Hash;
use std::fmt::Debug;
use std::future::Future;
use std::collections::hash_map::Entry;

impl GlobalConfig {
    pub async fn download_cached(&self, url: Url) -> anyhow::Result<PathBuf> {
        // TODO more robust file name parsing in the future? this should be used
        //      for maven only right now so it should be fine
        let file_name = url
            .path()
            .rsplit('/')
            .next()
            .ok_or(anyhow!("url '{}' does not have a file name!", url))?;

        fs::create_dir_all(&self.cache_dir).await?;

        let dest = self.cache_dir.join(file_name);

        if dest.is_file() {
            println!("info: using cached '{}'", file_name);
            Ok(dest)
        } else {
            println!("info: downloading '{}'", url);
            let temp_dest = self.cache_dir.join(format!("{}.__download__", file_name));

            let mut printed_delay_warning = false;

            let file = loop {
                let out_file = fs::OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .open(&temp_dest)
                    .await;

                match out_file {
                    Ok(file) => break file,
                    Err(e) if e.kind() == io::ErrorKind::AlreadyExists => {
                        if !printed_delay_warning {
                            println!("notice: '{}' is being downloaded by another oresolve instance,\
                            notice: if this hangs for a long time, or you are sure that no other oresolve\
                            notice: instance is currently running, delete the file at\
                            notice: {}", file_name, temp_dest.canonicalize().unwrap().to_str().unwrap());
                            printed_delay_warning = true;
                        }

                        delay_for(Duration::from_millis(500)).await;

                        if dest.is_file() {
                            return Ok(dest);
                        }
                    }
                    Err(e) => return Err(e.into()),
                }
            };

            match self.download_file_to(url, file).await {
                Ok(_) => match fs::rename(&temp_dest, &dest).await {
                    Ok(_) => Ok(dest),
                    Err(e) => {
                        let _ = fs::remove_file(&temp_dest);
                        Err(e.into())
                    }
                },
                Err(e) => {
                    let _ = fs::remove_file(&temp_dest);
                    Err(e.into())
                }
            }
        }
    }

    async fn download_file_to<W: AsyncWrite + Unpin>(
        &self,
        url: Url,
        mut dest: W,
    ) -> anyhow::Result<()> {
        let response = self.client.get(url).send().await?.error_for_status()?;
        let mut stream = response.bytes_stream();

        while let Some(item) = stream.next().await {
            let item: &[u8] = &item?;
            dest.write_all(item).await?;
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct ComputeCache<K, V> {
    map: Mutex<HashMap<K, watch::Receiver<Option<V>>>>,
}

impl<K, V> ComputeCache<K, V> {
    pub fn new() -> Self {
        ComputeCache {
            map: Mutex::new(HashMap::new()),
        }
    }
}

impl<K, V> ComputeCache<K, V>
    where
        K: Clone + Eq + Hash,
        V: Debug,
{
    pub async fn compute<F, O>(&self, key: K, op: F) -> V
        where
            F: FnOnce(K) -> O,
            O: Future<Output = V>,
            V: Clone,
    {
        let mut lock = self.map.lock().await;

        match lock.entry(key.clone()) {
            Entry::Occupied(e) => {
                let mut rx = e.get().clone();
                drop(lock);

                let result = rx.recv().await.unwrap().unwrap();
                result
            }
            Entry::Vacant(e) => {
                let (tx, mut rx) = watch::channel(None);
                rx.recv().await.unwrap(); // take away first 'None'
                e.insert(rx);
                drop(lock);

                let result = op(key).await;

                if let Err(_) = tx.broadcast(Some(result.clone())) {
                    unreachable!();
                }

                result
            }
        }
    }
}

impl<K, V> Default for ComputeCache<K, V> {
    fn default() -> Self {
        ComputeCache::new()
    }
}