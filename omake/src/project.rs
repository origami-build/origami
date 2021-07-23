use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;
use type_map::TypeMap;

use crate::plugin::Plugin;
use crate::Result;
use crate::task::Task;

pub struct Project {
    name: String,
    source_root: PathBuf,
    build_root: PathBuf,
    build_file: toml::Value,
    extensions: TypeMap,
    plugins: Vec<Box<dyn Plugin>>,
}

impl Project {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn source_root(&self) -> &Path {
        &self.source_root
    }

    pub fn build_root(&self) -> &Path {
        &self.build_root
    }

    pub fn build_file(&self) -> &toml::Value {
        &self.build_file
    }

    pub(crate) fn create_tasks(&self) -> Vec<Box<dyn Task>> {
        let mut vec = Vec::new();

        for plugin in self.plugins.iter() {
            vec.extend(plugin.create_tasks(self));
        }

        vec
    }

    pub fn extensions(&self) -> &TypeMap {
        &self.extensions
    }

    pub fn extensions_mut(&mut self) -> &mut TypeMap {
        &mut self.extensions
    }

    pub fn register_plugin<P>(&mut self, plugin: P)
    where
        P: Plugin + 'static,
    {
        self.plugins.push(Box::new(plugin))
    }
}

pub fn load_project(path: &Path) -> Result<Project> {
    let build_file = path.join("obuild.toml");
    let s = fs::read_to_string(build_file)?;
    let v: toml::Value = s.parse()?;

    #[derive(Deserialize)]
    struct Min {
        project: MinProject,
    }

    #[derive(Deserialize)]
    struct MinProject {
        name: String,
        plugins: Option<Vec<String>>,
        source_root: PathBuf,
    }

    let Min {
        project:
            MinProject {
                name,
                plugins,
                source_root,
            },
    } = Min::deserialize(v.clone()).unwrap();

    Ok(Project {
        name,
        source_root,
        build_root: path.to_path_buf(),
        build_file: v,
        extensions: TypeMap::default(),
        plugins: Vec::new(),
    })
}
