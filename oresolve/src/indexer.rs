use std::collections::HashMap;
use std::hash::Hash;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;

use futures::future::BoxFuture;
use futures::FutureExt;
use reqwest::StatusCode;
use tokio::fs;
use tokio::fs::File;
use tokio::sync::Mutex;
use url::Url;

use origami_common::lockfile::{Dependency, Package, RemoteFile};

use crate::cache::ComputeCache;
use crate::config::{DependencyData, ProjectConfig, RemoteDependency, WorkspaceConfig};
use crate::maven::gen::build_origami_project;
use crate::maven::{ArtifactPath, ArtifactVersion, Located};
use crate::{maven, sha256_stream, vercmp, GlobalConfig};

pub async fn index_project(
    state: IndexerState,
    rc: RequestContext,
    src: ProjectSource,
) -> Option<Arc<IndexedProject>> {
    state
        .cache
        .compute((rc, src), |(rc, src)| {
            let state = state.clone();
            async move { index_project0(state, &rc, src).await }
        })
        .await
}

async fn index_project0(
    state: IndexerState,
    rc: &RequestContext,
    src: ProjectSource,
) -> Option<Arc<IndexedProject>> {
    let ctx = match &src {
        ProjectSource::Local(path) => {
            let project_file = fs::read_to_string(path.join("origami.toml")).await.unwrap();
            let project: ProjectConfig = toml::from_str(&project_file).unwrap();
            let ws_config: WorkspaceConfig = toml::from_str(&project_file).unwrap();

            Some(ProjectContext {
                config: project,
                source: src,
                req: RequestContext {
                    local: rc.local,
                    ws_config: Arc::new(ws_config),
                },
            })
        }
        ProjectSource::Maven(dep) => {
            match maven::get_version_info(&state.g.client, dep).await {
                Ok(meta) => {
                    let config = build_origami_project(&meta);
                    Some(ProjectContext {
                        config,
                        // we want this to have the same repositories as our
                        // project
                        source: src,
                        req: RequestContext {
                            local: false,
                            ws_config: rc.ws_config.clone(),
                        },
                    })
                }
                Err(e) => {
                    eprintln!(
                        "error while retrieving metadata for '{}': {}",
                        dep.inner(),
                        e
                    );
                    None
                }
            }
        }
    };

    match ctx {
        None => None,
        Some(v) => Some(Arc::new(recursive_index(state, v).await)),
    }
}

fn recursive_index(state: IndexerState, ctx: ProjectContext) -> BoxFuture<'static, IndexedProject> {
    async move {
        let dependencies = ctx.config.dependencies.as_ref();

        let sources = tokio::spawn(compute_required_files(state.g.clone(), ctx.clone()));

        let rc = RequestContext {
            local: ctx.req.local && ctx.source.is_local(),
            ws_config: ctx.ws_config().clone(),
        };

        let tasks: Vec<_> = dependencies
            .iter()
            .flat_map(|v| v.iter())
            .map(|(name, data)| {
                let data = data.clone();
                let rc = rc.clone();
                let ctx = ctx.clone();
                let name = name.clone();
                let state = state.clone();
                tokio::spawn(async move {
                    let source = find_source(&state, rc.clone(), data).await;

                    match source {
                        None => {
                            eprintln!(
                                "error: could not resolve dependency '{}' of project '{} {}'",
                                name, ctx.config.project.name, ctx.config.project.version
                            );
                            None
                        }
                        Some(v) => index_project(state, rc, v).await,
                    }
                })
            })
            .collect();

        let mut package = IndexedProject {
            name: ctx.config.project.name.to_string(),
            version: ctx.config.project.version.to_string(),
            sources: sources.await.unwrap(),
            dependencies: vec![],
        };

        for task in tasks {
            let result = task.await.unwrap();

            if let Some(meta) = result {
                package.dependencies.push(meta);
            }
        }

        package
    }
    .boxed()
}

async fn find_source(
    state: &IndexerState,
    rc: RequestContext,
    dd: DependencyData,
) -> Option<ProjectSource> {
    let m = if rc.local {
        dd.path.map(ProjectSource::Local)
    } else {
        None
    };

    match m {
        None => match dd.remote_dependency {
            None => None,
            Some(RemoteDependency::Maven { version, artifact }) => {
                let ver = ArtifactPath::from_str(&artifact)
                    .unwrap()
                    .with_version(&version);
                let located = state
                    .find_matching_version(
                        &ver,
                        rc.ws_config.repositories.iter().flat_map(|v| v.values()),
                    )
                    .await;
                located.map(ProjectSource::Maven)
            }
        },
        x @ Some(_) => x,
    }
}

#[derive(Debug, Clone)]
pub struct ProjectContext {
    config: ProjectConfig,
    source: ProjectSource,
    req: RequestContext,
}

impl ProjectContext {
    pub fn new(config: ProjectConfig, source: ProjectSource, req: RequestContext) -> Self {
        ProjectContext {
            config,
            source,
            req,
        }
    }

    pub fn ws_config(&self) -> &Arc<WorkspaceConfig> {
        &self.req.ws_config
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct RequestContext {
    local: bool,
    ws_config: Arc<WorkspaceConfig>,
}

impl Default for RequestContext {
    fn default() -> Self {
        RequestContext {
            local: true,
            ws_config: Arc::new(Default::default()),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum ProjectSource {
    Local(PathBuf),
    Maven(Located<ArtifactVersion>),
}

impl ProjectSource {
    pub fn is_local(&self) -> bool {
        match self {
            ProjectSource::Local(_) => true,
            _ => false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct IndexerState {
    cache: Arc<ComputeCache<(RequestContext, ProjectSource), Option<Arc<IndexedProject>>>>,
    repo: Arc<Mutex<HashMap<Url, Arc<RepositoryInfo>>>>,
    g: Arc<GlobalConfig>,
}

impl IndexerState {
    pub fn new(g: Arc<GlobalConfig>) -> Self {
        IndexerState {
            cache: Arc::new(ComputeCache::new()),
            repo: Default::default(),
            g,
        }
    }

    async fn get_versions(
        &self,
        url: Url,
        path: &ArtifactPath,
    ) -> Arc<Vec<Located<ArtifactVersion>>> {
        let repo_info = self
            .repo
            .lock()
            .await
            .entry(url.clone())
            .or_default()
            .clone();

        let g = self.g.clone();

        repo_info
            .artifacts
            .compute(path.clone(), |path| async move {
                let lpath = Located::new(url.clone(), path.clone());
                let info = maven::get_artifact_info(&g.client, &lpath).await;
                let vec = match info {
                    Ok(meta) => Arc::new(
                        meta.versioning
                            .versions
                            .iter()
                            .map(|v| Located::new(url.clone(), path.clone().with_version(v)))
                            .collect(),
                    ),
                    Err(e) if e.status() == Some(StatusCode::NOT_FOUND) => {
                        // silently ignore
                        Arc::new(vec![])
                    }
                    Err(e) => {
                        eprintln!("warning: failed to fetch metadata, ignoring: {}", e);
                        Arc::new(vec![])
                    }
                };
                vec
            })
            .await
    }

    async fn find_matching_version(
        &self,
        artifact: &ArtifactVersion,
        // can't use IntoIterator, probably
        // https://github.com/rust-lang/rust/issues/64552
        repos: impl Iterator<Item = &Url>,
    ) -> Option<Located<ArtifactVersion>> {
        let mut artifacts = Vec::new();

        for repo in repos {
            let x = self.get_versions(repo.clone(), artifact.path()).await;
            artifacts.extend_from_slice(&x);
        }

        // TODO take most recent version
        let artifact = artifacts
            .into_iter()
            .filter(|el| vercmp::Style::Maven.matches(artifact.version(), el.inner().version()))
            .next();

        artifact
    }
}

#[derive(Debug, Default)]
struct RepositoryInfo {
    artifacts: ComputeCache<ArtifactPath, Arc<Vec<Located<ArtifactVersion>>>>,
}

#[derive(Debug)]
pub struct IndexedProject {
    name: String,
    version: String,
    sources: Vec<RemoteFile>,
    dependencies: Vec<Arc<IndexedProject>>,
}

impl IndexedProject {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn version(&self) -> &str {
        &self.version
    }

    pub fn sources(&self) -> &[RemoteFile] {
        &self.sources
    }

    pub fn dependencies(&self) -> &[Arc<IndexedProject>] {
        &self.dependencies
    }
}

async fn to_remote_file(g: Arc<GlobalConfig>, url: Url) -> RemoteFile {
    let path = g.download_cached(url.clone()).await.unwrap();
    let size = fs::metadata(&path).await.unwrap().len();
    let hash = sha256_stream(File::open(&path).await.unwrap())
        .await
        .unwrap();
    RemoteFile::Http {
        source: url.to_string(),
        checksum: hash,
        size: size as usize,
    }
}

async fn compute_required_files(g: Arc<GlobalConfig>, ctx: ProjectContext) -> Vec<RemoteFile> {
    let mut sources = Vec::new();
    match &ctx.source {
        ProjectSource::Local(_) => {}
        ProjectSource::Maven(l) => {
            let t1 = tokio::spawn(to_remote_file(g.clone(), l.metadata_url()));
            let t2 = tokio::spawn(to_remote_file(g.clone(), l.jar_url(None).unwrap()));
            sources.push(t1.await.unwrap());
            sources.push(t2.await.unwrap());
        }
    }
    sources
}
