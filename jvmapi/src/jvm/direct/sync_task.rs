use std::io;
use std::io::{Read, Write};
use std::time::Duration;

use futures::executor::block_on;
use futures::{AsyncReadExt, AsyncWriteExt};

use crate::jvm::direct::{async_task, PacketWriter};
use crate::jvm::JvmTask;
use crate::protocol::fncall::JvmInterface;

pub struct Task {
    pub(super) id: u32,
    pub(super) stdout: Option<Stdout>,
    pub(super) stderr: Option<Stdout>,
    pub(super) stdin: Option<Stdin>,
    pub(super) interface: JvmInterface<PacketWriter>,
}

impl JvmTask for Task {
    type Stdout = Stdout;
    type Stderr = Stdout;
    type Stdin = Stdin;
    type ExitStatus = ();

    fn wait(&mut self) -> io::Result<Self::ExitStatus> {
        block_on(self.interface.wait(self.id, None));
        Ok(())
    }

    fn stdout(&mut self) -> &mut Option<Self::Stdout> {
        &mut self.stdout
    }

    fn stderr(&mut self) -> &mut Option<Self::Stderr> {
        &mut self.stderr
    }

    fn stdin(&mut self) -> &mut Option<Self::Stdin> {
        &mut self.stdin
    }
}

pub struct Stdin {
    pub(super) inner: async_task::Stdin,
}

pub struct Stdout {
    pub(super) inner: async_task::Stdout,
}

impl Write for Stdin {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        block_on(self.inner.write(buf))
    }

    fn flush(&mut self) -> io::Result<()> {
        block_on(self.inner.flush())
    }
}

impl Read for Stdout {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        block_on(self.inner.read(buf))
    }
}
