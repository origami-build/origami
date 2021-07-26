use crate::jvm::{Jvm, Error, JvmCommandInner};

/// A builder for executing Java programs.
pub struct JvmCommand<T> {
    jvm: T,
    inner: JvmCommandInner,
}

impl<T> JvmCommand<T> {
    /// Constructs a new [`JvmCommand`] for launching `main_class`.
    pub fn new(jvm: T, main_class: &str) -> Self {
        JvmCommand {
            jvm,
            inner: JvmCommandInner {
                main_class: main_class.to_string(),
                args: vec![],
                stdout: None,
                stderr: None,
                stdin: None,
            }
        }
    }

    /// Adds a single argument to pass to the program.
    pub fn arg<S>(&mut self, arg: S) -> &mut Self
    where
        S: AsRef<str>,
    {
        self.inner.args.push(arg.as_ref().to_string());
        self
    }

    /// Adds multiple arguments to pass to the program.
    pub fn args<I>(&mut self, args: I) -> &mut Self
    where
        I: IntoIterator,
        I::Item: AsRef<str>,
    {
        self.inner.args
            .extend(args.into_iter().map(|el| el.as_ref().to_string()));
        self
    }

    /// Configures the standard output (stdout) for the new task.
    pub fn stdout(&mut self, cfg: Stdio) -> &mut Self {
        self.inner.stdout = Some(cfg);
        self
    }

    /// Configures the standard error (stderr) for the new task.
    pub fn stderr(&mut self, cfg: Stdio) -> &mut Self {
        self.inner.stderr = Some(cfg);
        self
    }

    /// Configures the standard input (stdin) for the new task.
    pub fn stdin(&mut self, cfg: Stdio) -> &mut Self {
        self.inner.stdin = Some(cfg);
        self
    }

    /// Returns the arguments that will be passed to the program.
    pub fn get_args(&self) -> &[String] {
        &self.inner.args
    }
}

impl<T> JvmCommand<T>
where
    T: Jvm,
{
    pub fn spawn(&self) -> Result<T::Task, Error> {
        self.jvm.exec(&self.inner, Stdio::Inherit)
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Stdio {
    /// Capture the stream from the spawned task.
    Piped,

    /// The spawned task should inherit the corresponding stream from the
    /// current process.
    Inherit,

    /// This stream will be ignored. This is the equivalent of attaching the
    /// stream to `/dev/null`.
    Null,
}
