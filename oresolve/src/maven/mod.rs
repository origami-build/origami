use std::ops::Deref;

use reqwest::Client;
use serde::Deserialize;

pub use specifier::*;

pub mod gen;
mod specifier;

pub async fn get_artifact_info(
    r: &Client,
    version: &Located<ArtifactPath>,
) -> reqwest::Result<ArtifactMetadata> {
    let result = r
        .get(version.metadata_url())
        .send()
        .await?
        .error_for_status()?;
    let text = result.text().await?;

    let project = serde_xml_rs::from_str(&text).unwrap();

    Ok(project)
}

#[derive(Debug, Deserialize)]
pub struct ArtifactMetadata {
    pub versioning: Versioning,
}

#[derive(Debug, Deserialize)]
pub struct Versioning {
    pub latest: Option<String>,
    pub release: Option<String>,
    pub versions: Versions,
    #[serde(rename = "lastUpdated")]
    pub last_updated: usize,
}

#[derive(Debug, Deserialize)]
pub struct Versions {
    version: Vec<String>,
}

impl Deref for Versions {
    type Target = [String];

    fn deref(&self) -> &Self::Target {
        &self.version
    }
}

pub async fn get_version_info(
    r: &Client,
    path: &Located<ArtifactVersion>,
) -> reqwest::Result<VersionMetadata> {
    let result = r
        .get(path.metadata_url())
        .send()
        .await?
        .error_for_status()?;
    let text = result.text().await?;

    let project = serde_xml_rs::from_str(&text).unwrap();

    Ok(project)
}

#[derive(Debug, Clone, Deserialize)]
pub struct VersionMetadata {
    #[serde(rename = "groupId")]
    pub group: String,
    #[serde(rename = "artifactId")]
    pub artifact: String,
    pub version: String,
    #[serde(default)]
    pub dependencies: Dependencies,
}

#[derive(Default, Debug, Clone, Deserialize)]
pub struct Dependencies {
    #[serde(default)]
    dependency: Vec<DepInfo>,
}

impl Deref for Dependencies {
    type Target = [DepInfo];

    fn deref(&self) -> &Self::Target {
        &self.dependency
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct DepInfo {
    #[serde(rename = "groupId")]
    pub group: String,
    #[serde(rename = "artifactId")]
    pub artifact: String,
    pub version: String,
    pub scope: DepScope,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DepScope {
    Compile,
    Runtime,
    Provided,
    Test,
    System,
    Import,
}
