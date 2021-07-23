use crate::jvm::command::Stdio;
use crate::jvm::{Error, Jvm, JvmCommandInner};

pub struct JvmCommand<T> {
    jvm: T,
    inner: JvmCommandInner,
}

impl<T> JvmCommand<T> {
    pub fn new(jvm: T, main_class: &str) -> Self {
        JvmCommand {
            jvm,
            inner: JvmCommandInner {
                main_class: main_class.to_string(),
                args: vec![],
                stdout: None,
                stderr: None,
                stdin: None,
            },
        }
    }

    pub fn arg<S>(&mut self, arg: S) -> &mut Self
    where
        S: AsRef<str>,
    {
        self.inner.args.push(arg.as_ref().to_string());
        self
    }

    pub fn args<I>(&mut self, args: I) -> &mut Self
    where
        I: IntoIterator,
        I::Item: AsRef<str>,
    {
        self.inner
            .args
            .extend(args.into_iter().map(|el| el.as_ref().to_string()));
        self
    }

    pub fn stdout(&mut self, cfg: Stdio) -> &mut Self {
        self.inner.stdout = Some(cfg);
        self
    }

    pub fn stderr(&mut self, cfg: Stdio) -> &mut Self {
        self.inner.stderr = Some(cfg);
        self
    }

    pub fn stdin(&mut self, cfg: Stdio) -> &mut Self {
        self.inner.stdin = Some(cfg);
        self
    }
}

impl<T> JvmCommand<T>
where
    T: Jvm,
{
    pub async fn spawn(&self) -> Result<T::AsyncTask, Error> {
        self.jvm.exec_async(&self.inner, Stdio::Inherit).await
    }
}
