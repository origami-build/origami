use std::fs;
use std::future::Future;
use std::io;
use std::io::Error;
use std::path::PathBuf;
use std::pin::Pin;
use std::task::{Context, Poll};

use async_std::net::{TcpListener, TcpStream};
use async_std::os::unix::net::{UnixListener, UnixStream};
use async_std::task::block_on;
use clap::{app_from_crate, Arg, ArgGroup};
use log::{error, LevelFilter};
use simplelog::{ColorChoice, Config, TerminalMode};

use jvmapi::jvm::direct::DirectJvm;
use jvmapi::jvm::process::ProcessJvm;

fn main() {
    let matches = app_from_crate!()
        .args(&[
            Arg::new("tcp")
                .short('T')
                .long("tcp")
                .value_name("address")
                .about("Listen on a TCP socket")
                .multiple_occurrences(true),
            Arg::new("unix")
                .short('S')
                .long("unix")
                .value_name("path")
                .about("Listen on a Unix domain socket")
                .multiple_occurrences(true),
        ])
        .group(
            ArgGroup::new("sockets")
                .args(&["tcp", "unix"])
                .required(true)
                .multiple(true),
        )
        .get_matches();

    simplelog::TermLogger::new(
        LevelFilter::Info,
        Config::default(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    );

    let mut sockets = Vec::new();
    let mut error = false;

    macro_rules! connect_impl {
        ($k:expr, $b:expr, $v:ident) => {
            if let Some(v) = matches.values_of($k) {
                for value in v {
                    match block_on($b(value)) {
                        Ok(v) => sockets.push(Listener::$v(v)),
                        Err(e) => {
                            error!("error trying to bind to {}: {}", value, e);
                            error = true;
                        }
                    }
                }
            }
        };
    }

    connect_impl!("tcp", TcpListener::bind, Tcp);
    connect_impl!("unix", UnixListener::bind, Unix);

    if error {
        return;
    }

    let jvm = ProcessJvm::new();
    let jvm = DirectJvm::spawn(jvm).expect("failed to spawn jvm");

    let mut last = 0;

    loop {}
}

enum Listener {
    Tcp(TcpListener),
    Unix(UnixListener),
}

impl Listener {
    pub async fn accept(&self) -> io::Result<(Stream, SocketAddr)> {
        match self {
            Listener::Tcp(inner) => match inner.accept().await {
                Ok((stream, addr)) => Ok((Stream::Tcp(stream), SocketAddr::Tcp(addr))),
                Err(e) => Err(e),
            },
            Listener::Unix(inner) => match inner.accept().await {
                Ok((stream, addr)) => Ok((Stream::Unix(stream), SocketAddr::Unix(addr))),
                Err(e) => Err(e),
            },
        }
    }
}

enum Stream {
    Tcp(TcpStream),
    Unix(UnixStream),
}

#[derive(Clone)]
enum SocketAddr {
    Tcp(std::net::SocketAddr),
    Unix(std::os::unix::net::SocketAddr),
}

impl Drop for Listener {
    fn drop(&mut self) {
        match self {
            Listener::Tcp(_) => {}
            Listener::Unix(l) => {
                // Dropping an UnixListener doesn't delete the socket file by
                // itself
                if let Ok(addr) = l.local_addr() {
                    if let Some(path) = addr.as_pathname() {
                        fs::remove_file(path);
                    }
                }
            }
        }
    }
}

fn accept_any(state: &mut usize, listeners: &[Listener]) -> AcceptAny {
    AcceptAny { state, listeners }
}

struct AcceptAny<'a> {
    state: &'a mut usize,
    listeners: &'a [Listener],
}

impl Future for AcceptAny {
    type Output = io::Result<(Stream, SocketAddr)>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {

    }
}
