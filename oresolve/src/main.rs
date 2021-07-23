use std::fs::File;
use std::path::PathBuf;
use std::sync::Arc;

use clap::{app_from_crate, Arg};
use serde::export::fmt::Debug;
use sha2::Digest;
use tokio::io;
use tokio::io::{AsyncRead, AsyncReadExt};
use tokio::runtime::Runtime;

use origami_common::lockfile::{Dependency, Package, WorkspaceLock};

use crate::indexer::{IndexedProject, IndexerState, ProjectSource, RequestContext};

mod cache;
mod config;
mod indexer;
mod maven;
mod vercmp;

fn main() {
    let matches = app_from_crate!()
        .args(&[
            Arg::new("cache-dir")
                .long("cache-dir")
                .about("path to the directory dependencies will be cached in")
                .default_value("origami/cache"),
            Arg::new("update").short('u').long("update"),
            Arg::new("offline-only").short('O').long("offline-only"),
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                .about("be verbose (print more information)"),
        ])
        .get_matches();

    let global_config = Arc::new(GlobalConfig {
        cache_dir: PathBuf::from("origami/distfiles"),
        client: reqwest::Client::new(),
    });

    let mut rt = Runtime::new().unwrap();

    let state = IndexerState::new(global_config.clone());
    let rc = RequestContext::default();
    let tree = rt.block_on(indexer::index_project(
        state,
        rc,
        ProjectSource::Local(PathBuf::from(".")),
    ));

    let mut lock = WorkspaceLock::default();
    populate_lock(&mut lock, &tree.unwrap());

    let lock_file = File::create("origami.lock").unwrap();
    lock.write(lock_file).unwrap();


}

#[derive(Debug, Clone)]
pub struct GlobalConfig {
    cache_dir: PathBuf,
    client: reqwest::Client,
}

async fn sha256_stream<R: AsyncRead + Unpin>(mut pipe: R) -> io::Result<[u8; 32]> {
    let mut hasher = sha2::Sha256::new();
    let mut buf = [0; 4096];

    loop {
        match pipe.read(&mut buf).await? {
            0 => break,
            len => hasher.update(&buf[..len]),
        }
    }

    let output = hasher.finalize();
    let mut arr = [0; 32];
    arr.copy_from_slice(&output);
    Ok(arr)
}

fn populate_lock(lock: &mut WorkspaceLock, project: &IndexedProject) {
    let package = to_package(project);

    match lock
        .packages
        .binary_search_by_key(&(project.name(), project.version()), |p| {
            (&p.name, &p.version)
        }) {
        Ok(idx) => {
            lock.packages[idx] = package;
        }
        Err(idx) => {
            lock.packages.insert(idx, package);
        }
    }

    for x in project.dependencies() {
        populate_lock(lock, &x);
    }
}

fn to_package(project: &IndexedProject) -> Package {
    Package {
        name: project.name().to_string(),
        version: project.version().to_string(),
        sources: project.sources().to_vec(),
        dependencies: project
            .dependencies()
            .iter()
            .map(|el| to_ref(&el))
            .collect(),
    }
}

fn to_ref(project: &IndexedProject) -> Dependency {
    Dependency {
        name: project.name().to_string(),
        version: project.version().to_string(),
    }
}
