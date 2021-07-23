#![feature(command_access)]
#![feature(generators)]
#![feature(generator_trait)]

use std::path::{Path, PathBuf};
use std::{fs, io};

use thiserror::Error;

use crate::project::Project;
use crate::java::plugin::JavaPlugin;
use crate::plugin::Plugin;
use crate::task::Context;

mod java;
mod plugin;
mod project;
mod task;

type Result<T, E = Error> = std::result::Result<T, E>;

fn main() {
    let path = Path::new("/home/saiko/src/origami/test/build");

    let dirs = collect_directories(path).expect("failed to load build root");
    let projects = dirs
        .into_iter()
        .map(|el| project::load_project(&el))
        .try_fold(Vec::new(), |mut acc, a| match a {
            Ok(a) => {
                acc.push(a);
                Ok(acc)
            }
            Err(e) => Err(e),
        })
        .expect("failed to load build directories");

    for mut project in projects {
        java::plugin::register(&mut project);

        let tasks = project.create_tasks();

        let ctx = Context::new(&project);

        println!("tasks = {:#?}", tasks);

        for task in tasks {
            task.make(&ctx).unwrap();
        }
    }
}

fn collect_directories(path: &Path) -> io::Result<Vec<PathBuf>> {
    let mut vec = Vec::new();

    let root_file = path.join("obuildroot");
    let s = fs::read_to_string(root_file)?;
    for line in s.lines() {
        if line.starts_with(";") || line.trim().is_empty() {
            continue;
        }

        vec.push(path.join(line));
    }

    Ok(vec)
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),
    #[error("parse error: {0}")]
    Parse(#[from] toml::de::Error),
}
