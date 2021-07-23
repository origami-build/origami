use crate::config::{DependenciesConfig, DependencyData, Project, ProjectConfig, RemoteDependency};
use crate::maven::{DepScope, VersionMetadata};

pub fn build_origami_project(pom: &VersionMetadata) -> ProjectConfig {
    let mut dependencies = DependenciesConfig::default();

    for dependency in pom.dependencies.iter() {
        let export = match dependency.scope {
            DepScope::Compile => true,
            DepScope::Runtime => false,
            DepScope::Provided => true,
            DepScope::Test => false,
            DepScope::System => true,
            DepScope::Import => true,
        };

        dependencies.0.insert(
            dependency.artifact.to_string(),
            DependencyData {
                remote_dependency: Some(RemoteDependency::Maven {
                    artifact: format!("{}:{}", dependency.group, dependency.artifact),
                    version: dependency.version.to_string(),
                }),
                export,
                path: None,
            },
        );
    }

    ProjectConfig {
        project: Project {
            name: pom.artifact.to_string(),
            version: pom.version.to_string(),
            authors: vec![],
        },
        dependencies: Some(dependencies),
    }
}
