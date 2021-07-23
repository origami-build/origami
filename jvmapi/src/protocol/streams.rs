use std::cmp::min;
use std::collections::HashMap;
use std::io;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, Waker};

use bytes::{Buf, BytesMut};
use futures::{AsyncRead, AsyncWrite};

#[derive(Default)]
pub struct Streams {
    pipes: HashMap<u32, AnonPipe>,
    next_id: u32,
}

impl Streams {
    pub fn new() -> Self {
        Streams {
            pipes: Default::default(),
            next_id: 0,
        }
    }

    pub fn alloc(&mut self, inherit: Option<Inherit>) -> AnonPipe {
        let shared = match inherit {
            None => AnonPipeShared::Impl(AnonPipeImpl {
                buffer: BytesMut::with_capacity(512),
                closed: false,
                write_waker: None,
                read_waker: None,
            }),
            Some(Inherit::Stdout) => AnonPipeShared::Stdout(async_std::io::stdout()),
            Some(Inherit::Stderr) => AnonPipeShared::Stderr(async_std::io::stderr()),
            Some(Inherit::Stdin) => AnonPipeShared::Stdin(async_std::io::stdin()),
        };

        let pipe = AnonPipe {
            id: self.next_id,
            shared: Arc::new(Mutex::new(shared)),
        };
        self.pipes.insert(pipe.id, pipe.clone());
        self.next_id += 1;
        pipe
    }

    pub fn free(&mut self, id: u32) {
        if let Some(pipe) = self.pipes.remove(&id) {
            pipe.close();
        }
    }

    pub fn by_id(&self, id: u32) -> Option<AnonPipe> {
        self.pipes.get(&id).cloned()
    }
}

pub enum Inherit {
    Stdout,
    Stderr,
    Stdin,
}

const MAX_LEN: usize = 4096;

#[derive(Debug, Clone)]
pub struct AnonPipe {
    id: u32,
    shared: Arc<Mutex<AnonPipeShared>>,
}

#[derive(Debug)]
enum AnonPipeShared {
    Impl(AnonPipeImpl),
    Stdout(async_std::io::Stdout),
    Stderr(async_std::io::Stderr),
    Stdin(async_std::io::Stdin),
}

#[derive(Debug)]
struct AnonPipeImpl {
    buffer: BytesMut,
    closed: bool,
    write_waker: Option<Waker>,
    read_waker: Option<Waker>,
}

impl AnonPipe {
    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn is_normal(&self) -> bool {
        match &*self.shared.lock().unwrap() {
            AnonPipeShared::Impl(_) => true,
            _ => false,
        }
    }

    fn close(&self) {
        match &mut *self.shared.lock().unwrap() {
            AnonPipeShared::Impl(imp) => {
                imp.closed = true;

                if let Some(waker) = imp.read_waker.take() {
                    waker.wake();
                }

                if let Some(waker) = imp.write_waker.take() {
                    waker.wake();
                }
            }
            _ => {}
        }
    }
}

impl AsyncWrite for AnonPipe {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        match &mut *self.shared.lock().unwrap() {
            AnonPipeShared::Impl(imp) => {
                if imp.closed {
                    return Poll::Ready(Ok(0));
                }

                let free = MAX_LEN - imp.buffer.len();
                let to_write = min(free, buf.len());

                if to_write > 0 {
                    imp.buffer.extend_from_slice(&buf[..to_write]);

                    if let Some(waker) = imp.read_waker.take() {
                        waker.wake();
                    }

                    Poll::Ready(Ok(to_write))
                } else {
                    imp.write_waker = Some(cx.waker().clone());
                    Poll::Pending
                }
            }
            AnonPipeShared::Stdout(stdout) => Pin::new(stdout).poll_write(cx, buf),
            AnonPipeShared::Stderr(stderr) => Pin::new(stderr).poll_write(cx, buf),
            AnonPipeShared::Stdin(_) => Poll::Ready(Ok(0)),
        }
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        match &mut *self.shared.lock().unwrap() {
            AnonPipeShared::Impl(_) => Poll::Ready(Ok(())),
            AnonPipeShared::Stdout(stdout) => Pin::new(stdout).poll_flush(cx),
            AnonPipeShared::Stderr(stderr) => Pin::new(stderr).poll_flush(cx),
            AnonPipeShared::Stdin(_) => Poll::Ready(Ok(())),
        }
    }

    fn poll_close(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        self.close();

        Poll::Ready(Ok(()))
    }
}

impl AsyncRead for AnonPipe {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<io::Result<usize>> {
        match &mut *self.shared.lock().unwrap() {
            AnonPipeShared::Impl(imp) => {
                let available = imp.buffer.len();

                if available > 0 {
                    let to_read = min(available, buf.len());
                    buf[..to_read].copy_from_slice(&imp.buffer[..to_read]);
                    imp.buffer.advance(to_read);

                    if let Some(waker) = imp.write_waker.take() {
                        waker.wake();
                    }

                    Poll::Ready(Ok(to_read))
                } else if imp.closed {
                    Poll::Ready(Ok(0))
                } else {
                    imp.read_waker = Some(cx.waker().clone());
                    Poll::Pending
                }
            }
            AnonPipeShared::Stdout(_) => Poll::Ready(Ok(0)),
            AnonPipeShared::Stderr(_) => Poll::Ready(Ok(0)),
            AnonPipeShared::Stdin(stdin) => Pin::new(stdin).poll_read(cx, buf),
        }
    }
}
