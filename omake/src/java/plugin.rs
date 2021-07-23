use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::{fs, io};

use serde::Deserialize;
use thiserror::Error;

use crate::java::JavaTask;
use crate::plugin::Plugin;
use crate::project::Project;
use crate::task::Task;
use std::process::{ExitStatus, Command};

pub fn register(project: &mut Project) {
    #[derive(Deserialize)]
    struct BuildSettings {
        #[serde(default)]
        java: PluginSettings,
    }

    #[derive(Default, Deserialize)]
    struct PluginSettings {
        sources: Option<Vec<PathBuf>>,
        output: Option<PathBuf>,
    }

    let BuildSettings {
        java: PluginSettings { sources, output },
    } = BuildSettings::deserialize(project.build_file().clone()).unwrap();

    let inputs = sources.unwrap_or_else(|| vec![PathBuf::from("src")]);
    let output_root = output.unwrap_or_else(|| PathBuf::from("."));

    let d = Rc::new(JavaData {
        inputs,
        output_root,
        javac_path: PathBuf::from("/usr/bin/javac"),
    });

    let plugin = JavaPlugin { data: d.clone() };
    let ext = JavaExtension { data: d };

    project.extensions_mut().insert(ext);
    project.register_plugin(plugin);
}

#[derive(Debug)]
pub struct JavaData {
    inputs: Vec<PathBuf>,
    output_root: PathBuf,
    javac_path: PathBuf,
}

pub struct JavaPlugin {
    data: Rc<JavaData>,
}

impl Plugin for JavaPlugin {
    fn create_tasks(&self, project: &Project) -> Vec<Box<dyn Task>> {
        let data = &self.data;

        let mut vec: Vec<Box<dyn Task>> = Vec::new();
        let input_dir = project.source_root().join("src");
        let output_dir = project.build_root().join("src");

        let a = recursive_find_java(&input_dir).unwrap();

        for file in a {
            let mut class = file.strip_prefix(&input_dir).unwrap().to_path_buf();
            class.set_file_name(class.file_stem().unwrap().to_os_string());
            vec.push(Box::new(JavaTask::new(
                input_dir.clone(),
                output_dir.clone(),
                class,
            )));
        }

        vec
    }
}

fn recursive_find_java(path: &Path) -> io::Result<Vec<PathBuf>> {
    let mut vec = Vec::new();

    for x in fs::read_dir(path)? {
        let x = x?;

        if x.file_type()?.is_dir() {
            vec.extend(recursive_find_java(&x.path())?.into_iter());
        } else {
            let file_name = x.file_name();
            if file_name.to_str().unwrap().ends_with(".java") {
                vec.push(path.join(file_name));
            }
        }
    }

    Ok(vec)
}

pub struct JavaExtension {
    data: Rc<JavaData>,
}

impl JavaExtension {
    pub fn exec_javac(&self, input: &Path, output_root: &Path) -> Result<(), ExecError> {
        let mut cmd = Command::new(&self.data.javac_path);

        for entry in self.class_path() {
            cmd.arg("--class-path");
            let path = match entry {
                ClassPathEntry::File(path) => path,
                ClassPathEntry::Dir(path) => path,
            };
            cmd.arg(path);
        }

        cmd.arg("-d");
        cmd.arg(output_root);
        cmd.arg(input);

        let map = cmd.get_args().into_iter().map(|s| s.to_string_lossy()).collect::<Vec<_>>();
        println!("{} {}", self.data.javac_path.to_string_lossy(), map.join(" "));

        let exit_code = cmd.status()?;

        if exit_code.success() {
            Ok(())
        } else {
            Err(ExecError::ExitStatus(exit_code))
        }
    }

    pub fn find_class_files(&self, class: &str) -> &[PathBuf] {
        unimplemented!()
    }

    pub fn class_path(&self) -> Vec<ClassPathEntry> {
        vec![
            // ClassPathEntry::Dir(self.project.build_root().join("src"))
        ]
    }
}

pub enum ClassPathEntry {
    File(PathBuf),
    Dir(PathBuf),
}

#[derive(Debug, Error)]
pub enum ExecError {
    #[error("exit code: {0}")]
    ExitStatus(ExitStatus),
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),
}
