use std::borrow::Cow;
use std::ffi::{OsStr, OsString};
use std::fs;
use std::fs::ReadDir;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use log::error;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct PathDep {
    base: PathBuf,
    optional: bool,
    dt: DepType,
}

#[derive(Debug, Clone, Eq, PartialEq)]
enum DepType {
    File,
    Dir { recursive: bool, pattern: OsString },
}

impl PathDep {
    pub fn new(buf: PathBuf) -> Self {
        PathDep {
            base: buf,
            optional: false,
            dt: DepType::File,
        }
    }

    pub fn with_optional(mut self) -> Self {
        self.optional = true;
        self
    }

    pub fn with_dir<S: AsRef<OsStr>>(mut self, recursive: bool, matches: S) -> Self {
        self.dt = DepType::Dir {
            recursive,
            pattern: matches.as_ref().to_os_string(),
        };
        self
    }

    pub fn base(&self) -> &Path {
        &self.base
    }

    pub fn paths(&self) -> Paths {
        Paths::new(self)
    }
}

pub trait PathDepLike {
    fn to_path_dep(&self) -> PathDep;

    fn base(&self) -> &Path;

    fn optional(&self) -> bool;

    fn path_matches(&self, path: &Path) -> bool;
}

impl PathDepLike for PathBuf {
    fn to_path_dep(&self) -> PathDep {
        PathDep::new(self.clone())
    }

    fn base(&self) -> &Path {
        &self
    }

    fn optional(&self) -> bool {
        false
    }

    fn path_matches(&self, path: &Path) -> bool {
        self == path
    }
}

impl From<PathBuf> for PathDep {
    fn from(pb: PathBuf) -> Self {
        PathDep::new(pb)
    }
}

impl PathDepLike for &Path {
    fn to_path_dep(&self) -> PathDep {
        PathDep::new(self.to_path_buf())
    }

    fn base(&self) -> &Path {
        self
    }

    fn optional(&self) -> bool {
        false
    }

    fn path_matches(&self, path: &Path) -> bool {
        self == &path
    }
}

impl PathDepLike for PathDep {
    fn to_path_dep(&self) -> PathDep {
        self.clone()
    }

    fn base(&self) -> &Path {
        &self.base
    }

    fn optional(&self) -> bool {
        self.optional
    }

    fn path_matches(&self, path: &Path) -> bool {
        match &self.dt {
            DepType::File => self.base == path,
            DepType::Dir { recursive, pattern } => {
                let stripped = match path.strip_prefix(&self.base) {
                    Ok(p) => p,
                    Err(_) => return false,
                };

                if !recursive && stripped.parent().is_some() {
                    return false;
                }

                // TODO: don't hardcode this, lmao
                if pattern.to_str() == Some("*.class") {
                    stripped.extension().and_then(OsStr::to_str) == Some("class")
                } else {
                    todo!()
                }
            }
        }
    }
}

impl<T> PathDepLike for &T
where
    T: PathDepLike,
{
    fn to_path_dep(&self) -> PathDep {
        (*self).to_path_dep()
    }

    fn base(&self) -> &Path {
        (*self).base()
    }

    fn optional(&self) -> bool {
        (*self).optional()
    }

    fn path_matches(&self, path: &Path) -> bool {
        (*self).path_matches(path)
    }
}

pub struct Paths<'a> {
    pd: &'a PathDep,
    state: PathsState<'a>,
}

impl<'a> Paths<'a> {
    pub fn new(pd: &'a PathDep) -> Self {
        let state = match &pd.dt {
            DepType::File => PathsState::Single(Some(pd.base())),
            DepType::Dir { recursive, .. } => {
                let stack = match fs::read_dir(pd.base()) {
                    Ok(rd) => vec![rd],
                    Err(e) => match e.kind() {
                        ErrorKind::NotFound => vec![],
                        _ => {
                            error!("error reading {}: {}", pd.base().display(), e);
                            vec![]
                        }
                    },
                };

                PathsState::Directories {
                    recursive: *recursive,
                    stack,
                }
            }
        };

        Paths { pd, state }
    }
}

impl<'a> Iterator for Paths<'a> {
    type Item = Cow<'a, Path>;

    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.state {
            PathsState::Single(path) => path.take().map(Cow::Borrowed),
            PathsState::Directories { recursive, stack } => {
                while let Some(tos) = stack.last_mut() {
                    // try getting next directory entry
                    let entry = match tos.next() {
                        None => {
                            stack.pop();
                            continue;
                        }
                        Some(Err(e)) => {
                            error!("error reading directory entry, ignoring: {}", e);
                            continue;
                        }
                        Some(Ok(entry)) => entry,
                    };

                    // and its file type
                    let ft = match entry.file_type() {
                        Ok(ft) => ft,
                        Err(e) => {
                            error!(
                                "error reading file type of '{}', ignoring: {}",
                                entry.path().display(),
                                e
                            );
                            continue;
                        }
                    };

                    if ft.is_dir() && *recursive {
                        // if it's a directory and recursive mode is on, read it
                        // and push the new ReadDir onto the stack
                        match fs::read_dir(entry.path()) {
                            Ok(rd) => {
                                stack.push(rd);
                            }
                            Err(e) => {
                                error!(
                                    "error reading directory '{}', ignoring: {}",
                                    entry.path().display(),
                                    e
                                );
                            }
                        }
                    } else if ft.is_file() {
                        // if it's a file, return it if its path matches the
                        // pattern
                        let path = entry.path();

                        if self.pd.path_matches(&path) {
                            return Some(path.into());
                        }
                    }
                }

                None
            }
        }
    }
}

#[derive(Debug)]
enum PathsState<'a> {
    Single(Option<&'a Path>),
    Directories {
        recursive: bool,
        stack: Vec<ReadDir>,
    },
}
