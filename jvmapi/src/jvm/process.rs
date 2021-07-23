use std::ffi::{OsStr, OsString};
use std::io;
use std::path::PathBuf;
use std::process::{Child, ChildStderr, ChildStdin, ChildStdout, Command, ExitStatus, Stdio};

use futures::future::BoxFuture;
use futures::FutureExt;

use crate::jvm;
use crate::jvm::{command, AsyncJvmTask, Error, Jvm, JvmTask};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ProcessJvm {
    java_path: PathBuf,
    java_args: Vec<OsString>,
    classpath: Vec<PathBuf>,
}

impl ProcessJvm {
    pub fn new() -> Self {
        ProcessJvm {
            java_path: PathBuf::from("java"),
            java_args: vec![],
            classpath: vec![],
        }
    }

    pub fn with_java_executable<P>(&mut self, path: P) -> &mut Self
    where
        P: Into<PathBuf>,
    {
        self.java_path = path.into();
        self
    }

    pub fn java_executable(&self) -> &PathBuf {
        &self.java_path
    }

    pub fn with_classpath<I>(&mut self, paths: I) -> &mut Self
    where
        I: IntoIterator,
        I::Item: Into<PathBuf>,
    {
        for path in paths.into_iter() {
            self.classpath.push(path.into());
        }

        self
    }

    pub fn classpath(&self) -> &[PathBuf] {
        &self.classpath
    }
}

impl ProcessJvm {
    pub fn with_java_arg<S>(&mut self, arg: S) -> &mut Self
    where
        S: Into<OsString>,
    {
        self.java_args.push(arg.into());
        self
    }

    pub fn with_java_args<I>(&mut self, args: I) -> &mut Self
    where
        I: IntoIterator,
        I::Item: Into<OsString>,
    {
        for arg in args.into_iter() {
            self.java_args.push(arg.into());
        }

        self
    }
}

impl Jvm for ProcessJvm {
    type Task = JvmProcess;
    type AsyncTask = AsyncJvmProcess;

    fn exec(
        &self,
        d: &jvm::JvmCommandInner,
        default_stdio: command::Stdio,
    ) -> Result<Self::Task, Error> {
        struct OsStrWrap<T>(T);

        impl<T> AsRef<OsStr> for OsStrWrap<T>
        where
            T: AsRef<str>,
        {
            fn as_ref(&self) -> &OsStr {
                self.0.as_ref().as_ref()
            }
        }

        let conv_stdio =
            |stdio: Option<command::Stdio>| conv_stdio1(stdio.unwrap_or(default_stdio));

        let child = Command::new(&self.java_path)
            .args(&self.java_args)
            .args(crate::javacli::jvm_args(&self.classpath, d.main_class()))
            .args(d.args().into_iter().map(OsStrWrap))
            .stdout(conv_stdio(d.stdout()))
            .stderr(conv_stdio(d.stderr()))
            .stdin(conv_stdio(d.stdin()))
            .spawn();

        match child {
            Ok(process) => Ok(JvmProcess { process }),
            Err(e) => Err(Error::Io(e)),
        }
    }

    fn exec_async(
        &self,
        d: &jvm::JvmCommandInner,
        default_stdio: command::Stdio,
    ) -> BoxFuture<Result<Self::AsyncTask, Error>> {
        struct OsStrWrap<T>(T);

        impl<T> AsRef<OsStr> for OsStrWrap<T>
        where
            T: AsRef<str>,
        {
            fn as_ref(&self) -> &OsStr {
                self.0.as_ref().as_ref()
            }
        }

        let conv_stdio =
            |stdio: Option<command::Stdio>| conv_stdio1(stdio.unwrap_or(default_stdio));

        let child = async_process::Command::new(&self.java_path)
            .args(&self.java_args)
            .args(crate::javacli::jvm_args(&self.classpath, d.main_class()))
            .args(d.args().into_iter().map(OsStrWrap))
            .stdout(conv_stdio(d.stdout()))
            .stderr(conv_stdio(d.stderr()))
            .stdin(conv_stdio(d.stdin()))
            .spawn();

        let result = match child {
            Ok(process) => Ok(AsyncJvmProcess { process }),
            Err(e) => Err(Error::Io(e)),
        };

        async { result }.boxed()
    }
}

fn conv_stdio1(stdio: command::Stdio) -> Stdio {
    match stdio {
        command::Stdio::Piped => Stdio::piped(),
        command::Stdio::Inherit => Stdio::inherit(),
        command::Stdio::Null => Stdio::null(),
    }
}

pub struct JvmProcess {
    process: Child,
}

impl JvmTask for JvmProcess {
    type Stdout = ChildStdout;
    type Stderr = ChildStderr;
    type Stdin = ChildStdin;
    type ExitStatus = ExitStatus;

    fn wait(&mut self) -> io::Result<Self::ExitStatus> {
        self.process.wait()
    }

    fn stdout(&mut self) -> &mut Option<Self::Stdout> {
        &mut self.process.stdout
    }

    fn stderr(&mut self) -> &mut Option<Self::Stderr> {
        &mut self.process.stderr
    }

    fn stdin(&mut self) -> &mut Option<Self::Stdin> {
        &mut self.process.stdin
    }
}

pub struct AsyncJvmProcess {
    process: async_process::Child,
}

impl AsyncJvmTask for AsyncJvmProcess {
    type Stdout = async_process::ChildStdout;
    type Stderr = async_process::ChildStderr;
    type Stdin = async_process::ChildStdin;
    type ExitStatus = async_process::ExitStatus;

    fn wait(&mut self) -> BoxFuture<io::Result<Self::ExitStatus>> {
        self.process.status().boxed()
    }

    fn stdout(&mut self) -> &mut Option<Self::Stdout> {
        &mut self.process.stdout
    }

    fn stderr(&mut self) -> &mut Option<Self::Stderr> {
        &mut self.process.stderr
    }

    fn stdin(&mut self) -> &mut Option<Self::Stdin> {
        &mut self.process.stdin
    }
}
