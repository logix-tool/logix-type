use logix_vfs::LogixVfs;

use crate::{
    error::{ParseError, PathError, Result, SourceSpan, Wanted},
    parser::LogixParser,
    string::StrLit,
    token::{Literal, Token},
    type_trait::{LogixTypeDescriptor, LogixValueDescriptor, Value},
    types::{FullPath, NameOnlyPath, ValidPath},
    LogixType,
};
use std::{
    borrow::Cow,
    ffi::OsStr,
    fmt,
    path::{Path, PathBuf},
};

#[derive(Clone, Debug, PartialEq, Eq)]
enum Location {
    Path(FullPath),
    Name(NameOnlyPath),
}

/// The environment used when resolving executable paths
#[derive(Debug, Default, PartialEq, Eq)]
pub struct ExecutableEnv<'a> {
    pub path_env: Option<Cow<'a, OsStr>>,
}

impl<'a> ExecutableEnv<'a> {
    const DEFAULT: ExecutableEnv<'static> = ExecutableEnv { path_env: None };

    pub fn which(&self, name_or_path: impl AsRef<OsStr>) -> Option<FullPath> {
        if let Some(path_env) = self.path_env.as_deref() {
            which::which_in_global(name_or_path, Some(path_env))
                .ok()?
                .next()
                .map(FullPath::try_from)?
                .ok()
        } else {
            which::which_global(name_or_path)
                .ok()
                .map(FullPath::try_from)?
                .ok()
        }
    }
}

/// A path to an executable. This is either a full path, or a filename. If it is a
/// relative path it will fail to avoid issues. For example, imagine what happens
/// if EDITOR is set to a relative path.
#[derive(Clone, PartialEq, Eq)]
pub struct ExecutablePath {
    loc: Location,
}

impl ExecutablePath {
    pub fn as_path(&self) -> &Path {
        match &self.loc {
            Location::Path(v) => v.as_path(),
            Location::Name(v) => v.as_path(),
        }
    }

    pub fn which(&self, env: Option<&ExecutableEnv>) -> Option<FullPath> {
        env.unwrap_or(&ExecutableEnv::DEFAULT).which(self)
    }
}

impl TryFrom<PathBuf> for ExecutablePath {
    type Error = PathError;

    fn try_from(path: PathBuf) -> Result<Self, PathError> {
        match ValidPath::try_from(path)? {
            ValidPath::Full(path) => Ok(Self {
                loc: Location::Path(path),
            }),
            ValidPath::Name(name) => Ok(Self {
                loc: Location::Name(name),
            }),
            ValidPath::Rel(_) => Err(PathError::NotFullOrNameOnly),
        }
    }
}

impl From<ExecutablePath> for PathBuf {
    fn from(v: ExecutablePath) -> PathBuf {
        match v.loc {
            Location::Path(v) => v.into(),
            Location::Name(v) => v.into(),
        }
    }
}

impl_path_type_traits!(
    ExecutablePath,
    "The name of or full path to an executable file"
);
