#![feature(termination_trait_lib)]

use std::process::{ExitStatus, Termination};

use clap::app_from_crate;

use common_args::{read_props, AppExt};
use jvmapi::jvm::JvmTask;
use jvmapi::{JvmCommand, ProcessJvm};

fn main() -> ExitStatusWrap {
    let matches = app_from_crate!().add_javac_common_args().get_matches();

    let props = read_props(&matches);

    let jvm = ProcessJvm::new();
    let mut cmd = JvmCommand::new(&jvm, "com.sun.tools.javac.Main");

    cmd.arg("-implicit:none");

    let include = jvmapi::javacli::build_classpath(&props.include);
    let link = jvmapi::javacli::build_classpath(&props.link);

    if let Some(include) = &include {
        cmd.arg("-sourcepath");
        cmd.arg(include.to_str().unwrap());
    }

    if let Some(link) = &link {
        cmd.arg("-classpath");
        cmd.arg(link.to_str().unwrap());
    }

    if let Some(out_dir) = props.out_dir {
        cmd.arg("-s");
        cmd.arg(out_dir.to_str().unwrap());
    }

    if props.debug {
        cmd.arg("-g");
    }

    if let Some(release) = props.release {
        cmd.arg("--release");
        cmd.arg(release);
    }

    for file in props.in_files {
        cmd.arg(file.to_str().unwrap());
    }

    let mut proc = cmd.spawn().expect("Failed to spawn javac");
    ExitStatusWrap(proc.wait().expect("Failed to wait for javac"))
}

struct ExitStatusWrap(ExitStatus);

impl Termination for ExitStatusWrap {
    fn report(self) -> i32 {
        self.0.code().unwrap_or(10)
    }
}
