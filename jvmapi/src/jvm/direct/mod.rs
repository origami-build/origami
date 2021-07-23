use async_std::task;
use async_std::task::JoinHandle;
use futures::executor::block_on;
use futures::future::BoxFuture;
use futures::FutureExt;

use crate::framed::{FramedRead, FramedWrite};
use crate::jvm::command::Stdio;
use crate::jvm::process::{AsyncJvmProcess, ProcessJvm};
use crate::jvm::{async_command, AsyncJvmTask, Error, Jvm, JvmCommandInner};
use crate::protocol::fncall::{JvmInterface, TaskData};
use crate::protocol::structs::ExecError;
use crate::protocol::{fncall, ProtocolCodec};

pub mod async_task;
pub mod sync_task;

// TODO
const LIB_PATH: &str =
    "/home/saiko/src/origami/task-dispatcher/build/libs/task-dispatcher-0.1.0.jar";
const MAIN_CLASS: &str = "net.dblsaiko.origami.taskdispatcher.Main";

pub struct DirectJvm {
    process: AsyncJvmProcess,
    task: JoinHandle<()>,
    interface: JvmInterface<PacketWriter>,
}

impl DirectJvm {
    pub fn spawn(mut host: ProcessJvm) -> Result<Self, Error> {
        host.with_java_arg("--enable-preview");
        host.with_classpath(&[LIB_PATH]);

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
