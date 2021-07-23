use std::io::{BufRead, BufReader};
use std::time::Instant;

use jvmapi::jvm::command::{JvmCommand, Stdio};
use jvmapi::jvm::direct::DirectJvm;
use jvmapi::jvm::process::ProcessJvm;
use jvmapi::jvm::JvmTask;

fn main() {
    // simplelog::TermLogger::init(LevelFilter::Trace, Config::default(), TerminalMode::Stderr, ColorChoice::Auto);

    let mut proc = ProcessJvm::new();
    // proc.with_java_args(&["-Xdebug", "-Xrunjdwp:transport=dt_socket,address=8000,server=y,suspend=y,quiet=y"]);
    let jvm = DirectJvm::spawn(proc).unwrap();

    let mut cmd = JvmCommand::new(jvm, "net.dblsaiko.origami.TestService");
    cmd.args(&["foo", "bar", "baz"]);

    let tasks: Vec<_> = (0..20).map(|_| cmd.spawn().unwrap()).collect();

    for mut t in tasks {
        t.wait().unwrap();
    }
}
