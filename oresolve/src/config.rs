use std::collections::HashMap;
use std::ops::Deref;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use url::Url;
use linked_hash_map::LinkedHashMap;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ProjectConfig {
    pub project: Project,
    pub dependencies: Option<DependenciesConfig>,
}

#[derive(Debug, Default, Eq, PartialEq, Hash, Clone, Deserialize, Serialize)]
pub struct WorkspaceConfig {
    pub repositories: Option<RepositoriesConfig>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Project {
    pub name: String,
    pub version: String,
    pub authors: Vec<String>,
}

#[derive(Debug, Clone, Default, Eq, PartialEq, Hash, Deserialize, Serialize)]
pub struct RepositoriesConfig(pub LinkedHashMap<String, Repository>);

impl Deref for RepositoriesConfig {
    type Target = LinkedHashMap<String, Repository>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct DependenciesConfig(pub HashMap<String, DependencyData>);

impl Deref for DependenciesConfig {
    type Target = HashMap<String, DependencyData>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub type Repository = Url; // TODO

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DependencyData {
    #[serde(flatten)]
    pub remote_dependency: Option<RemoteDependency>,
    /// Whether this dependency should be exported to anything using the project
    /// itself as a dependency
    #[serde(default)]
    pub export: bool,
    pub path: Option<PathBuf>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum RemoteDependency {
    Maven { artifact: String, version: String },
}
