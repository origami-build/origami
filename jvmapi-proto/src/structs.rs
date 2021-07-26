use std::io;
use std::io::{Error, ErrorKind};
use std::time::Duration;

use binserde::{BinDeserialize, BinDeserializer, BinSerialize, BinSerializer};
use std::borrow::Cow;

/// The message container for all the messages sent by the task dispatcher
/// running on the JVM.
#[derive(BinSerialize, BinDeserialize)]
pub enum FromJvm {
    ExecResult(ExecResult),
    Write(Write),
    Read(Read),
    WaitResult(WaitResult),
    Close(Close),
}

/// The message container for all the messages sent to the JVM.
#[derive(BinSerialize, BinDeserialize)]
pub enum ToJvm<'a> {
    Exec(Exec<'a>),
    WriteResult(WriteResult),
    ReadResult(ReadResult<'a>),
    Wait(Wait),
    CloseResult(CloseResult),
}

/// A message that tells the JVM to execute the given main class.
#[derive(BinSerialize, BinDeserialize)]
pub struct Exec<'a> {
    /// The message tag to keep track of the response.
    pub tag: u32,

    /// The main class to execute, using `.` as a package delimiter.
    pub main_class: Cow<'a, str>,

    /// The parameters to pass to the main class.
    pub params: Cow<'a, [String]>,

    /// The raw stream id to send standard output to. A None value means to
    /// discard anything sent to stdout.
    pub stdout: Option<u32>,

    /// The raw stream id to send standard error to. A None value means to
    /// discard anything sent to stdout.
    pub stderr: Option<u32>,

    /// The raw stream id to read standard input from. A None value means to
    /// discard anything sent to stdout.
    pub stdin: Option<u32>,
}

/// The message sent in response to [`Exec`].
#[derive(BinSerialize, BinDeserialize)]
pub struct ExecResult {
    /// The same value as the corresponding [`Exec::tag`] value.
    pub tag: u32,

    pub result: Result<TaskInfo, ExecError>,
}

#[derive(BinSerialize, BinDeserialize)]
pub struct TaskInfo {
    pub task_id: u32,
}

/// Represents an error that occurred while trying to execute a task on the JVM.
#[derive(BinSerialize, BinDeserialize)]
pub enum ExecError {
    /// Represents a general error trying to launch the task.
    Failure(String),

    /// This error means that the specified class does not exist or could
    /// otherwise not be loaded.
    InvalidClass(String),

    /// This error means that the specified class does not have a main function
    /// `static void main(String[])`.
    NoMainFn(String),
}

/// Read a certain amount of maximum bytes from the specified stream.
#[derive(BinSerialize, BinDeserialize)]
pub struct Read {
    /// The message tag to keep track of the response.
    pub tag: u32,

    /// The stream to read from.
    pub stream: u32,

    /// The maximum amount of bytes to read.
    pub size: u32,
}

#[derive(BinSerialize, BinDeserialize)]
pub struct ReadResult<'a> {
    /// The same value as the corresponding [`Read::tag`] value.
    pub tag: u32,

    pub result: Result<Cow<'a, [u8]>, IoError>,
}

/// Write bytes to the given stream.
#[derive(BinSerialize, BinDeserialize)]
pub struct Write {
    /// The message tag to keep track of the response.
    pub tag: u32,
    
    /// The stream to write to.
    pub stream: u32,
    
    /// The bytes to write to the stream.
    pub data: Vec<u8>,
}

#[derive(BinSerialize, BinDeserialize)]
pub struct WriteResult {
    /// The same value as the corresponding [`Write::tag`] value.
    pub tag: u32,

    /// The amount of bytes written, or the error if any.
    pub result: Result<usize, IoError>,
}

/// Wait for a task to exit, with an optional timeout.
#[derive(BinSerialize, BinDeserialize)]
pub struct Wait {
    /// The message tag to keep track of the response.
    pub tag: u32,
    
    /// The task to wait for.
    pub task: u32,
    
    /// The duration to wait for. After this is expired, return a [`WaitResult`]
    /// regardless of if the task has exited or now. 
    pub timeout: Option<Duration>,
}

#[derive(BinSerialize, BinDeserialize)]
pub struct WaitResult {
    /// The same value as the corresponding [`Wait::tag`] value.
    pub tag: u32,
    
    /// Whether the timeout has been hit. If this is true, does not guarantee
    /// that the task is still running.
    pub timeout: bool,
}

/// Close the specified stream.
#[derive(BinSerialize, BinDeserialize)]
pub struct Close {
    /// The message tag to keep track of the response.
    pub tag: u32,
    
    /// The stream to close.
    pub stream: u32,
}

#[derive(BinSerialize, BinDeserialize)]
pub struct CloseResult {
    /// The same value as the corresponding [`Close::tag`] value.
    pub tag: u32,
    
    pub result: Result<(), IoError>,
}

pub struct IoError(pub io::Error);

impl BinSerialize for IoError {
    fn serialize<S: BinSerializer>(&self, mut serializer: S) -> binserde::Result<()> {
        let kind: usize = match self.0.kind() {
            ErrorKind::NotFound => 1,
            ErrorKind::PermissionDenied => 2,
            ErrorKind::ConnectionRefused => 3,
            ErrorKind::ConnectionReset => 4,
            ErrorKind::ConnectionAborted => 5,
            ErrorKind::NotConnected => 6,
            ErrorKind::AddrInUse => 7,
            ErrorKind::AddrNotAvailable => 8,
            ErrorKind::BrokenPipe => 9,
            ErrorKind::AlreadyExists => 10,
            ErrorKind::WouldBlock => 11,
            ErrorKind::InvalidInput => 12,
            ErrorKind::InvalidData => 13,
            ErrorKind::TimedOut => 14,
            ErrorKind::WriteZero => 15,
            ErrorKind::Interrupted => 16,
            ErrorKind::UnexpectedEof => 17,

            ErrorKind::Other => 0,
            // unimplemented error kind, go with Other for now
            _ => 0,
        };
        kind.serialize(&mut serializer)?;
        self.0.to_string().serialize(&mut serializer)?;
        Ok(())
    }
}

impl<'de> BinDeserialize<'de> for IoError {
    fn deserialize<D: BinDeserializer<'de>>(mut deserializer: D) -> binserde::Result<Self> {
        let kind = match usize::deserialize(&mut deserializer)? {
            0 => ErrorKind::Other,
            1 => ErrorKind::NotFound,
            2 => ErrorKind::PermissionDenied,
            3 => ErrorKind::ConnectionRefused,
            4 => ErrorKind::ConnectionReset,
            5 => ErrorKind::ConnectionAborted,
            6 => ErrorKind::NotConnected,
            7 => ErrorKind::AddrInUse,
            8 => ErrorKind::AddrNotAvailable,
            9 => ErrorKind::BrokenPipe,
            10 => ErrorKind::AlreadyExists,
            11 => ErrorKind::WouldBlock,
            12 => ErrorKind::InvalidInput,
            13 => ErrorKind::InvalidData,
            14 => ErrorKind::TimedOut,
            15 => ErrorKind::WriteZero,
            16 => ErrorKind::Interrupted,
            17 => ErrorKind::UnexpectedEof,
            _ => unreachable!(),
        };
        let message = String::deserialize(&mut deserializer)?;
        Ok(IoError(io::Error::new(kind, message)))
    }
}

impl From<io::Error> for IoError {
    fn from(e: Error) -> Self {
        IoError(e)
    }
}

impl From<IoError> for io::Error {
    fn from(e: IoError) -> Self {
        e.0
    }
}
