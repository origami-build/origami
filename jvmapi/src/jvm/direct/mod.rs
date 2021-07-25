use std::io::Write;
use std::sync::{Arc, Mutex, MutexGuard, Weak};

use async_std::task;
use async_std::task::JoinHandle;
use futures::executor::block_on;
use futures::future::BoxFuture;
use futures::FutureExt;
use lazy_static::lazy_static;
use tempfile::{NamedTempFile, TempPath};

use crate::framed::{FramedRead, FramedWrite};
use crate::jvm::command::Stdio;
use crate::jvm::process::{AsyncJvmProcess, ProcessJvm};
use crate::jvm::{async_command, AsyncJvmTask, Error, Jvm, JvmCommandInner};
use crate::protocol::fncall::{JvmInterface, TaskData};
use crate::protocol::structs::ExecError;
use crate::protocol::{fncall, ProtocolCodec};

pub mod async_task;
pub mod sync_task;

const MAIN_CLASS: &str = "net.dblsaiko.origami.taskdispatcher.Main";

/// Extracts the task-dispatcher.jar to a temporary path if necessary and
/// returns that path.
fn get_lib_path() -> Arc<TempPath> {
    lazy_static! {
        static ref RC: Mutex<Weak<TempPath>> = Mutex::new(Weak::new());
    }

    let mut a: MutexGuard<Weak<TempPath>> = RC.lock().unwrap();

    if let Some(pb) = a.upgrade() {
        pb
    } else {
        let mut tempfile = NamedTempFile::new().expect("Failed to create temp jar file");
        let bytes = include_bytes!("../../../../task-dispatcher/build/libs/task-dispatcher.jar");
        tempfile
            .write_all(bytes)
            .expect("Failed to write temp jar file");
        let path = tempfile.into_temp_path();
        let arc = Arc::new(path);
        *a = Arc::downgrade(&arc);
        arc
    }
}

pub struct DirectJvm {
    process: AsyncJvmProcess,
    task: JoinHandle<()>,
    interface: JvmInterface<PacketWriter>,

    // this needs to exist as long as the JVM is running on Windows (and also to
    // potentially prevent having to extract it multiple times), so we keep it
    // around
    lib_path: Arc<TempPath>,
}

impl DirectJvm {
    pub fn spawn(mut host: ProcessJvm) -> Result<Self, Error> {
        let lib_path = get_lib_path();
        host.with_java_arg("--enable-preview");
        host.with_classpath(&[&**lib_path]);

        let mut process = block_on(
            async_command::JvmCommand::new(host, MAIN_CLASS)
                .stdin(Stdio::Piped)
                .stdout(Stdio::Piped)
                .stderr(Stdio::Inherit)
                .spawn(),
        )?;

        let stdin = process.stdin().take().unwrap();
        let stdout = process.stdout().take().unwrap();

        let (interface, ph) = fncall::create(
            PacketReader::new(stdout, ProtocolCodec),
            PacketWriter::new(stdin, ProtocolCodec),
        );

        let task = task::spawn(ph.run());

        Ok(DirectJvm {
            process,
            task,
            interface,
            lib_path,
        })
    }
}

impl Jvm for DirectJvm {
    type Task = sync_task::Task;
    type AsyncTask = async_task::Task;

    fn exec(&self, d: &JvmCommandInner, default_stdio: Stdio) -> Result<Self::Task, Error> {
        block_on(async move {
            let response = self
                .launch(
                    d,
                    d.stdout().unwrap_or(default_stdio),
                    d.stderr().unwrap_or(default_stdio),
                    d.stdin().unwrap_or(default_stdio),
                )
                .await?;

            Ok(sync_task::Task {
                id: response.task_id,
                stdout: response.stdout.map(|pipe| sync_task::Stdout {
                    inner: async_task::Stdout { inner: pipe },
                }),
                stderr: response.stderr.map(|pipe| sync_task::Stdout {
                    inner: async_task::Stdout { inner: pipe },
                }),
                stdin: response.stdin.map(|pipe| sync_task::Stdin {
                    inner: async_task::Stdin { inner: pipe },
                }),
                interface: self.interface.clone(),
            })
        })
    }

    fn exec_async<'a>(
        &'a self,
        d: &'a JvmCommandInner,
        default_stdio: Stdio,
    ) -> BoxFuture<'a, Result<Self::AsyncTask, Error>> {
        async move {
            let response = self
                .launch(
                    d,
                    d.stdout().unwrap_or(default_stdio),
                    d.stderr().unwrap_or(default_stdio),
                    d.stdin().unwrap_or(default_stdio),
                )
                .await?;

            Ok(async_task::Task {
                id: response.task_id,
                stdout: response
                    .stdout
                    .map(|pipe| async_task::Stdout { inner: pipe }),
                stderr: response
                    .stderr
                    .map(|pipe| async_task::Stdout { inner: pipe }),
                stdin: response.stdin.map(|pipe| async_task::Stdin { inner: pipe }),
                interface: self.interface.clone(),
            })
        }
        .boxed()
    }
}

impl DirectJvm {
    async fn launch(
        &self,
        d: &JvmCommandInner,
        stdout: Stdio,
        stderr: Stdio,
        stdin: Stdio,
    ) -> Result<TaskData, Error> {
        let result = self
            .interface
            .exec(d.main_class(), d.args(), stdout, stderr, stdin)
            .await;

        match result {
            Ok(ok) => Ok(ok),
            Err(e) => Err(match e {
                ExecError::Failure(msg) => Error::Failure(msg),
                ExecError::InvalidClass(msg) => Error::InvalidClass(msg),
                ExecError::NoMainFn(msg) => Error::NoMainFn(msg),
            }),
        }
    }
}

type PacketWriter = FramedWrite<async_process::ChildStdin, ProtocolCodec>;
type PacketReader = FramedRead<async_process::ChildStdout, ProtocolCodec>;
