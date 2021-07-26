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

/// Trait that represents a JVM that tasks can be executed on.
pub trait Jvm {
    type Task: JvmTask;
    type AsyncTask: AsyncJvmTask;

    /// Execute the given [`JvmCommandInner`]. Do not call directly, instead use
    /// [`JvmCommand`].
    fn exec(&self, d: &JvmCommandInner, default_stdio: Stdio) -> Result<Self::Task, Error>;

    /// Execute the given [`JvmCommandInner`] asynchronously. Do not call
    /// directly, instead use [`JvmCommand`].
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

/// Contains the options built by the [`JvmCommand`] builder.
pub struct JvmCommandInner {
    main_class: String,
    args: Vec<String>,
    stdout: Option<Stdio>,
    stderr: Option<Stdio>,
    stdin: Option<Stdio>,
}

impl JvmCommandInner {
    /// Returns the path to the Java class to execute, using `.` as the package
    /// separator.
    pub fn main_class(&self) -> &str {
        &self.main_class
    }

    /// Returns the arguments to pass to the main Java class.
    pub fn args(&self) -> &[String] {
        &self.args
    }

    /// Returns how the standard output stream should be hooked up. A value of
    /// None means this option is unset, use the default passed to [`Jvm::exec()`].
    pub fn stdout(&self) -> Option<Stdio> {
        self.stdout
    }

    /// Returns how the standard error stream should be hooked up. A value of
    /// None means this option is unset, use the default passed to [`Jvm::exec()`].
    pub fn stderr(&self) -> Option<Stdio> {
        self.stderr
    }

    /// Returns how the standard input stream should be hooked up. A value of
    /// None means this option is unset, use the default passed to [`Jvm::exec()`].
    pub fn stdin(&self) -> Option<Stdio> {
        self.stdin
    }
}

impl<T> Jvm for &T
where
    T: Jvm,
{
    type Task = T::Task;
    type AsyncTask = T::AsyncTask;

    fn exec(&self, d: &JvmCommandInner, default_stdio: Stdio) -> Result<Self::Task, Error> {
        (*self).exec(d, default_stdio)
    }

    fn exec_async<'a>(
        &'a self,
        d: &'a JvmCommandInner,
        default_stdio: Stdio,
    ) -> BoxFuture<'a, Result<Self::AsyncTask, Error>> {
        (*self).exec_async(d, default_stdio)
    }
}
