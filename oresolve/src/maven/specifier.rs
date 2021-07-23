use std::fmt;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

use url::Url;

#[derive(Debug, Hash, Eq, PartialEq, Clone)]
pub struct ArtifactPath {
    group: String,
    artifact: String,
}

impl ArtifactPath {
    pub fn new(group: &str, artifact: &str) -> Self {
        // TODO: validate syntax
        ArtifactPath {
            group: group.to_string(),
            artifact: artifact.to_string(),
        }
    }

    pub fn to_url_part(&self) -> String {
        format!("{}/{}", self.group.replace('.', "/"), self.artifact)
    }

    pub fn with_version(self, version: &str) -> ArtifactVersion {
        ArtifactVersion {
            path: self,
            version: version.to_string(),
        }
    }
}

impl FromStr for ArtifactPath {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut splits = s.splitn(2, ':');
        let group = splits.next().ok_or(())?;
        let artifact = splits.next().ok_or(())?;

        if artifact.contains(":") {
            return Err(());
        }

        Ok(ArtifactPath {
            group: group.to_string(),
            artifact: artifact.to_string(),
        })
    }
}

impl Display for ArtifactPath {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}:{}", self.group, self.artifact)
    }
}

#[derive(Debug, Hash, Eq, PartialEq, Clone)]
pub struct ArtifactVersion {
    path: ArtifactPath,
    version: String,
}

impl ArtifactVersion {
    pub fn to_url_part(&self) -> String {
        format!("{}/{}", self.path.to_url_part(), self.version)
    }

    pub fn path(&self) -> &ArtifactPath {
        &self.path
    }

    pub fn version(&self) -> &str {
        &self.version
    }
}

impl FromStr for ArtifactVersion {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut splits = s.rsplitn(2, ':');
        let version = splits.next().ok_or(())?;
        let rest = splits.next().ok_or(())?;
        let path = rest.parse()?;

        Ok(ArtifactVersion {
            path,
            version: version.to_string(),
        })
    }
}

impl Display for ArtifactVersion {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}:{}", self.path, self.version)
    }
}

#[derive(Debug, Hash, Eq, PartialEq, Clone)]
pub struct Located<T> {
    repository: Url,
    inner: T,
}

impl<T> Located<T> {
    pub fn new(repository: Url, inner: T) -> Self {
        Located { repository, inner }
    }

    pub fn inner(&self) -> &T {
        &self.inner
    }

    pub fn into_inner(self) -> T {
        self.inner
    }
}

impl Located<ArtifactPath> {
    pub fn metadata_url(&self) -> Url {
        self.to_url().join("maven-metadata.xml").unwrap()
    }

    pub fn to_url(&self) -> Url {
        Url::parse(&format!(
            "{}/{}/",
            self.repository,
            self.inner.to_url_part()
        ))
        .expect("failed to parse URL")
    }
}

impl Located<ArtifactVersion> {
    pub fn prefix(&self) -> String {
        format!("{}-{}", self.inner.path.artifact, self.inner.version)
    }

    pub fn metadata_url(&self) -> Url {
        // this never fails here since all the characters in the joined string
        // are already present in the base URL itself
        self.to_url()
            .join(&format!("{}.pom", self.prefix()))
            .unwrap()
    }

    pub fn jar_url(&self, classifier: Option<&str>) -> Result<Url, url::ParseError> {
        match classifier {
            None => self.to_url().join(&format!("{}.jar", self.prefix())),
            Some(c) => self.to_url().join(&format!("{}-{}.jar", self.prefix(), c)),
        }
    }

    pub fn to_url(&self) -> Url {
        Url::parse(&format!(
            "{}/{}/",
            self.repository,
            self.inner.to_url_part()
        ))
        .expect("failed to parse URL")
    }
}
