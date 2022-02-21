use std::ops::Deref;
use std::path::Path;

use clap::app_from_crate;
use proc_exit::{exit, Code};

use common_args::{read_props, AppExt};
use jvmapi::jvm::JvmTask;
use jvmapi::{JvmCommand, ProcessJvm};

fn main() {
    let matches = app_from_crate!().add_javac_common_args().get_matches();

    let props = read_props(&matches);

    let jar = JarFile::get();

    // safety check so that you don't just get a class not found error from java
    // that doesn't really say anything
    if !jar.exists() {
        exit(Err(Code::NOT_FOUND.with_message(format!(
            "could not locate jar file '{}'",
            jar.display()
        ))));
    }

    let mut jvm = ProcessJvm::new();
    jvm.with_classpath(&[&*jar]);
    let mut cmd = JvmCommand::new(&jvm, "net.dblsaiko.origami.ojavac.Main");

    cmd.arg("-implicit:none");

    let include = jvmapi::javacli::build_classpath(&props.include);
    let link = jvmapi::javacli::build_classpath(&props.link);

    if let Some(include) = &include {
        cmd.arg("-sourcepath");
        cmd.arg(include.to_str().unwrap());
    }

    if let Some(out_dir) = props.out_dir {
        cmd.arg("-d");
        cmd.arg(out_dir.to_str().unwrap());
    }

    if props.debug {
        cmd.arg("-g");
    }

    if let Some(release) = props.release {
        cmd.arg("--release");
        cmd.arg(release);
    }

    for arg in props.ap_args {
        cmd.arg(format!("-A{}", arg));
    }

    match (props.no_ap, props.no_class_gen) {
        (false, false) => {}
        (false, true) => {
            cmd.arg("-proc:only");
        }
        (true, false) => {
            cmd.arg("-proc:none");
        }
        (true, true) => unreachable!(),
    }

    let javac_options_len = cmd.get_args().len();

    for file in props.in_files {
        cmd.arg(file.to_str().unwrap());
    }

    let inputs_len = cmd.get_args().len() - javac_options_len;

    cmd.arg(link.as_deref().map(|x| x.to_str().unwrap()).unwrap_or("."));
    cmd.arg(format!("{}", javac_options_len));
    cmd.arg(format!("{}", inputs_len));
    cmd.arg(props.write_deps.map(|p| p.to_str().unwrap()).unwrap_or(""));
    cmd.arg(
        props
            .write_makedeps
            .map(|p| p.to_str().unwrap())
            .unwrap_or(""),
    );

    let mut proc = cmd.spawn().expect("Failed to spawn javac");
    exit(Code::from_status(proc.wait().expect("Failed to wait for javac")).ok())
}

enum JarFile {
    #[cfg(install)]
    Installed(std::path::PathBuf),
    #[cfg(not(install))]
    Temp(tempfile::NamedTempFile),
}

impl Deref for JarFile {
    type Target = Path;

    fn deref(&self) -> &Self::Target {
        match self {
            #[cfg(install)]
            JarFile::Installed(v) => &v,
            #[cfg(not(install))]
            JarFile::Temp(v) => v.path(),
        }
    }
}

impl JarFile {
    #[cfg(not(install))]
    fn get() -> Self {
        use std::io::Write as _;

        let jar = include_bytes!("../java/ojavac.jar");
        let mut tf = tempfile::NamedTempFile::new().expect("failed to create temporary file");
        tf.write_all(jar).expect("failed to write jar contents");
        JarFile::Temp(tf)
    }

    #[cfg(install)]
    fn get() -> Self {
        let exec_dir = Path::new(origami_common::LIBEXECDIR);
        let mut path = if exec_dir.is_relative() {
            let mut path = std::env::current_exe().unwrap_or_default();
            path.pop();
            path.push(exec_dir);
            path
        } else {
            exec_dir.to_path_buf()
        };

        path.push("ojavac.jar");
        JarFile::Installed(path)
    }
}
