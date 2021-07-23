use std::collections::HashMap;
use std::fmt::Debug;
use std::io;
use std::pin::Pin;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll};
use std::time::Duration;

use async_codec::ReadFrameError;
use async_std::channel;
use async_std::stream::Stream;
use async_std::sync::Mutex as AsyncMutex;
use async_std::task::spawn;
use futures::{AsyncReadExt, AsyncWrite, AsyncWriteExt, Sink, SinkExt, StreamExt};

use crate::jvm::command::Stdio;
use crate::protocol::streams::{AnonPipe, Inherit, Streams};
use crate::protocol::structs::{
    CloseResult, Exec, ExecError, ExecResult, FromJvm, ReadResult, ToJvm, Wait, WaitResult,
    WriteResult,
};

#[derive(Debug)]
struct AsyncQueueInner<T> {
    queue: HashMap<u32, channel::Sender<T>>,
}

impl<T> Default for AsyncQueueInner<T> {
    fn default() -> Self {
        AsyncQueueInner {
            queue: Default::default(),
        }
    }
}

struct AsyncQueue<T> {
    inner: Arc<Mutex<AsyncQueueInner<T>>>,
}

impl<T> Clone for AsyncQueue<T> {
    fn clone(&self) -> Self {
        AsyncQueue {
            inner: self.inner.clone(),
        }
    }
}

impl<T> AsyncQueue<T> {
    pub fn new() -> Self {
        AsyncQueue {
            inner: Arc::new(Default::default()),
        }
    }

    pub async fn start_callback(&self, tag: u32) -> T {
        let (tx, rx) = channel::bounded(1);
        self.inner.lock().unwrap().queue.insert(tag, tx);
        rx.recv().await.unwrap()
    }

    pub async fn finish_callback(&self, tag: u32, msg: T) {
        let option = self.inner.lock().unwrap().queue.remove(&tag);

        if let Some(tx) = option {
            let _ = tx.send(msg).await;
        }
    }
}

struct Shared<T> {
    sink: Arc<AsyncMutex<T>>,
    exec: AsyncQueue<ExecResult>,
    wait: AsyncQueue<WaitResult>,
}

impl<T> Clone for Shared<T> {
    fn clone(&self) -> Self {
        Shared {
            sink: self.sink.clone(),
            exec: self.exec.clone(),
            wait: self.wait.clone(),
        }
    }
}

pub struct PacketHandler<T, U> {
    call_impl: Calls,
    input: T,
    shared: Shared<U>,
}

#[derive(Clone)]
struct Calls {
    streams: Arc<Mutex<Streams>>,
}

impl Calls {
    fn get_stream(&self, stream: u32) -> io::Result<AnonPipe> {
        self.streams.lock().unwrap().by_id(stream).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::Other,
                format!("invalid stream id {}", stream),
            )
        })
    }

    async fn write(&self, stream: u32, buf: &[u8]) -> io::Result<usize> {
        let mut stream = self.get_stream(stream)?;
        stream.write(buf).await
    }

    async fn read(&self, stream: u32, buf: &mut [u8]) -> io::Result<usize> {
        let mut stream = self.get_stream(stream)?;
        stream.read(buf).await
    }

    async fn close(&self, stream: u32) -> io::Result<()> {
        let mut stream = self.get_stream(stream)?;
        stream.close().await
    }
}

impl<T, U> PacketHandler<T, U>
where
    T: Stream<Item = Result<FromJvm, ReadFrameError<binserde::Error>>> + Send + Unpin + 'static,
    U: Sink<ToJvm<'static>> + Send + Unpin + 'static,
    <U as Sink<ToJvm<'static>>>::Error: Debug,
{
    pub async fn run(mut self) {
        while let Some(next) = self.input.next().await {
            let next = match next {
                Ok(next) => next,
                Err(e) => {
                    eprintln!("{:?}", e);
                    break;
                }
            };

            match next {
                FromJvm::ExecResult(p) => {
                    self.shared.exec.finish_callback(p.tag, p).await;
                }
                FromJvm::Write(p) => {
                    let call_impl = self.call_impl.clone();
                    let sink = self.shared.sink.clone();
                    spawn(async move {
                        let result = call_impl.write(p.stream, &p.data).await;

                        let response = ToJvm::WriteResult(WriteResult {
                            tag: p.tag,
                            result: result.map_err(|e| e.into()),
                        });

                        sink.lock().await.send(response).await.unwrap();
                    });
                }
                FromJvm::Read(p) => {
                    let call_impl = self.call_impl.clone();
                    let sink = self.shared.sink.clone();
                    spawn(async move {
                        let mut buf = vec![0; p.size as usize];
                        let result = call_impl.read(p.stream, &mut buf).await;
                        let result = match result {
                            Ok(size) => Ok(buf[..size].to_vec().into()),
                            Err(e) => Err(e),
                        };

                        let response = ToJvm::ReadResult(ReadResult {
                            tag: p.tag,
                            result: result.map_err(|e| e.into()),
                        });

                        sink.lock().await.send(response).await.unwrap();
                    });
                }
                FromJvm::WaitResult(p) => {
                    self.shared.wait.finish_callback(p.tag, p).await;
                }
                FromJvm::Close(p) => {
                    let call_impl = self.call_impl.clone();
                    let sink = self.shared.sink.clone();
                    spawn(async move {
                        let result = call_impl.close(p.stream).await;

                        let response = ToJvm::CloseResult(CloseResult {
                            tag: p.tag,
                            result: result.map_err(|e| e.into()),
                        });

                        sink.lock().await.send(response).await.unwrap();
                    });
                }
            }
        }
    }
}

pub struct TaskData {
    pub task_id: u32,
    pub stdout: Option<AnonPipe>,
    pub stderr: Option<AnonPipe>,
    pub stdin: Option<AnonPipe>,
}

pub struct JvmInterface<T> {
    tag: Arc<AtomicU32>,
    streams: Arc<Mutex<Streams>>,
    shared: Shared<T>,
}

impl<T> Clone for JvmInterface<T> {
    fn clone(&self) -> Self {
        JvmInterface {
            tag: self.tag.clone(),
            streams: self.streams.clone(),
            shared: self.shared.clone(),
        }
    }
}

impl<T> JvmInterface<T>
where
    T: Sink<ToJvm<'static>> + Unpin,
    <T as Sink<ToJvm<'static>>>::Error: Debug,
{
    pub async fn exec(
        &self,
        main_class: &str,
        params: &[String],
        stdout: Stdio,
        stderr: Stdio,
        stdin: Stdio,
    ) -> Result<TaskData, ExecError> {
        fn maybe_alloc_pipe(
            streams: &mut Streams,
            stdio: Stdio,
            inherit_type: Inherit,
        ) -> Option<AnonPipe> {
            match stdio {
                Stdio::Piped => Some(streams.alloc(None)),
                Stdio::Inherit => Some(streams.alloc(Some(inherit_type))),
                Stdio::Null => None,
            }
        }

        fn if_normal(pipe: Option<AnonPipe>) -> Option<AnonPipe> {
            match pipe {
                Some(pipe) if pipe.is_normal() => Some(pipe),
                _ => None,
            }
        }

        let (stdout, stderr, stdin) = {
            let mut streams = self.streams.lock().unwrap();
            let stdout = maybe_alloc_pipe(&mut streams, stdout, Inherit::Stdout);
            let stderr = maybe_alloc_pipe(&mut streams, stderr, Inherit::Stderr);
            let stdin = maybe_alloc_pipe(&mut streams, stdin, Inherit::Stdin);
            (stdout, stderr, stdin)
        };

        let tag = self.tag.fetch_add(1, Ordering::Relaxed);
        let packet = ToJvm::Exec(Exec {
            tag,
            main_class: main_class.to_string().into(),
            params: params.to_vec().into(),
            stdout: stdout.as_ref().map(|pipe| pipe.id()),
            stderr: stderr.as_ref().map(|pipe| pipe.id()),
            stdin: stdin.as_ref().map(|pipe| pipe.id()),
        });

        let cb = self.shared.exec.start_callback(tag);

        self.shared.sink.lock().await.send(packet).await.unwrap();

        match cb.await.result {
            Ok(ti) => Ok(TaskData {
                task_id: ti.task_id,
                stdin: if_normal(stdin),
                stdout: if_normal(stdout),
                stderr: if_normal(stderr),
            }),
            Err(e) => Err(e),
        }
    }

    pub async fn wait(&self, task: u32, timeout: Option<Duration>) -> bool {
        let tag = self.tag.fetch_add(1, Ordering::Relaxed);
        let packet = ToJvm::Wait(Wait { tag, task, timeout });

        let cb = self.shared.wait.start_callback(tag);

        self.shared.sink.lock().await.send(packet).await.unwrap();

        cb.await.timeout
    }
}

pub fn create<T, U>(source: T, sink: U) -> (JvmInterface<U>, PacketHandler<T, U>) {
    let shared = Shared {
        sink: Arc::new(AsyncMutex::new(sink)),
        exec: AsyncQueue::new(),
        wait: AsyncQueue::new(),
    };

    let streams = Arc::new(Mutex::new(Streams::new()));

    let i = JvmInterface {
        tag: Arc::new(Default::default()),
        streams: streams.clone(),
        shared: shared.clone(),
    };

    let h = PacketHandler {
        call_impl: Calls { streams },
        input: source,
        shared,
    };

    (i, h)
}
