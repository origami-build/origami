use std::io;
use std::io::{Read, Write};

use async_std::io::{Read as AsyncRead, Write as AsyncWrite};
use futures::future::BoxFuture;
use thiserror::Error;

use crate::jvm::command::Stdio;

pub mod direct;
pub mod process;

pub mod async_command;
pub mod command;

pub trait Jvm {
    type Task: JvmTask;
    type AsyncTask: AsyncJvmTask;

    fn exec(&self, d: &JvmCommandInner, default_stdio: Stdio) -> Result<Self::Task, Error>;

    fn exec_async<'a>(
        &'a self,
        d: &'a JvmCommandInner,
        default_stdio: Stdio,
    ) -> BoxFuture<'a, Result<Self::AsyncTask, Error>>;
}

pub trait JvmTask {
    type Stdout: Read;
    type Stderr: Read;
    type Stdin: Write;
    type ExitStatus;

    fn wait(&mut self) -> io::Result<Self::ExitStatus>;

    fn stdout(&mut self) -> &mut Option<Self::Stdout>;

    fn stderr(&mut self) -> &mut Option<Self::Stderr>;

    fn stdin(&mut self) -> &mut Option<Self::Stdin>;
}

pub trait AsyncJvmTask {
    type Stdout: AsyncRead;
    type Stderr: AsyncRead;
    type Stdin: AsyncWrite;
    type ExitStatus;

    fn wait(&mut self) -> BoxFuture<io::Result<Self::ExitStatus>>;

    fn stdout(&mut self) -> &mut Option<Self::Stdout>;

    fn stderr(&mut self) -> &mut Option<Self::Stderr>;

    fn stdin(&mut self) -> &mut Option<Self::Stdin>;
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("I/O error: {0}")]
    Io(io::Error),
    #[error("Invalid main class: {0}")]
    InvalidClass(String),
    #[error("Class does not have a main function: {0}")]
    NoMainFn(String),
    #[error("General error: {0}")]
    Failure(String),
}

pub struct JvmCommandInner {
    main_class: String,
    args: Vec<String>,
    stdout: Option<Stdio>,
    stderr: Option<Stdio>,
    stdin: Option<Stdio>,
}

impl JvmCommandInner {
    pub fn main_class(&self) -> &str {
        &self.main_class
    }

    pub fn args(&self) -> &[String] {
        &self.args
    }

    pub fn stdout(&self) -> Option<Stdio> {
        self.stdout
    }

    pub fn stderr(&self) -> Option<Stdio> {
        self.stderr
    }

    pub fn stdin(&self) -> Option<Stdio> {
        self.stdin
    }
}
