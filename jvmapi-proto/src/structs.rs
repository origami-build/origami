use std::io;
use std::io::{Error, ErrorKind};
use std::time::Duration;

use binserde::{BinDeserialize, BinDeserializer, BinSerialize, BinSerializer};
use std::borrow::Cow;

#[derive(BinSerialize, BinDeserialize)]
pub enum FromJvm {
    ExecResult(ExecResult),
    Write(Write),
    Read(Read),
    WaitResult(WaitResult),
    Close(Close),
}

#[derive(BinSerialize, BinDeserialize)]
pub enum ToJvm<'a> {
    Exec(Exec<'a>),
    WriteResult(WriteResult),
    ReadResult(ReadResult<'a>),
    Wait(Wait),
    CloseResult(CloseResult),
}

#[derive(BinSerialize, BinDeserialize)]
pub struct Exec<'a> {
    pub tag: u32,
    pub main_class: Cow<'a, str>,
    pub params: Cow<'a, [String]>,
    pub stdout: Option<u32>,
    pub stderr: Option<u32>,
    pub stdin: Option<u32>,
}

#[derive(BinSerialize, BinDeserialize)]
pub struct ExecResult {
    pub tag: u32,
    pub result: Result<TaskInfo, ExecError>,
}

#[derive(BinSerialize, BinDeserialize)]
pub struct TaskInfo {
    pub task_id: u32,
}

#[derive(BinSerialize, BinDeserialize)]
pub enum ExecError {
    Failure(String),
    InvalidClass(String),
    NoMainFn(String),
}

#[derive(BinSerialize, BinDeserialize)]
pub enum Stdio {
    Null,
    Piped,
    Inherit,
}

#[derive(BinSerialize, BinDeserialize)]
pub struct Read {
    pub tag: u32,
    pub stream: u32,
    pub size: u32,
}

#[derive(BinSerialize, BinDeserialize)]
pub struct ReadResult<'a> {
    pub tag: u32,
    pub result: Result<Cow<'a, [u8]>, IoError>,
}

#[derive(BinSerialize, BinDeserialize)]
pub struct Write {
    pub tag: u32,
    pub stream: u32,
    pub data: Vec<u8>,
}

#[derive(BinSerialize, BinDeserialize)]
pub struct WriteResult {
    pub tag: u32,
    pub result: Result<usize, IoError>,
}

#[derive(BinSerialize, BinDeserialize)]
pub struct Wait {
    pub tag: u32,
    pub task: u32,
    pub timeout: Option<Duration>,
}

#[derive(BinSerialize, BinDeserialize)]
pub struct WaitResult {
    pub tag: u32,
    pub timeout: bool,
}

#[derive(BinSerialize, BinDeserialize)]
pub struct Close {
    pub tag: u32,
    pub stream: u32,
}

#[derive(BinSerialize, BinDeserialize)]
pub struct CloseResult {
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
