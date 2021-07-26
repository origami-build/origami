use std::io;
use std::pin::Pin;
use std::task::{Context, Poll};

use futures::future::BoxFuture;
use futures::{AsyncRead, AsyncWrite, FutureExt};
use pin_project_lite::pin_project;

use crate::jvm::direct::PacketWriter;
use crate::jvm::AsyncJvmTask;
use crate::protocol::fncall::JvmInterface;
use crate::protocol::streams::AnonPipe;

/// A spawned JVM task.
pub struct Task {
    pub(super) id: u32,
    pub(super) stdout: Option<Stdout>,
    pub(super) stderr: Option<Stdout>,
    pub(super) stdin: Option<Stdin>,
    pub(super) interface: JvmInterface<PacketWriter>,
}

impl AsyncJvmTask for Task {
    type Stdout = Stdout;
    type Stderr = Stdout;
    type Stdin = Stdin;
    type ExitStatus = ();

    /// Wait for the task to exit.
    fn wait(&mut self) -> BoxFuture<'_, io::Result<Self::ExitStatus>> {
        self.interface
            .wait(self.id, None)
            .map(|_timeout| Ok(()))
            .boxed()
    }

    /// The standard output stream, if the process was spawned with
    /// [`Stdio::Piped`], else None.
    fn stdout(&mut self) -> &mut Option<Self::Stdout> {
        &mut self.stdout
    }

    /// The standard error stream, if the process was spawned with
    /// [`Stdio::Piped`], else None.
    fn stderr(&mut self) -> &mut Option<Self::Stderr> {
        &mut self.stderr
    }

    /// The standard input stream, if the process was spawned with
    /// [`Stdio::Piped`], else None.
    fn stdin(&mut self) -> &mut Option<Self::Stdin> {
        &mut self.stdin
    }
}

pin_project! {
    pub struct Stdin {
        #[pin]
        pub inner: AnonPipe,
    }
}

pin_project! {
    pub struct Stdout {
        #[pin]
        pub inner: AnonPipe,
    }
}

impl AsyncWrite for Stdin {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        let this = self.project();
        this.inner.poll_write(cx, buf)
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        let this = self.project();
        this.inner.poll_flush(cx)
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        let this = self.project();
        this.inner.poll_close(cx)
    }
}

impl AsyncRead for Stdout {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<io::Result<usize>> {
        let this = self.project();
        this.inner.poll_read(cx, buf)
    }
}
