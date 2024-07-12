use bstr::ByteSlice;
use logix_vfs::LogixVfs;

use crate::{
    error::{ParseError, PathError, Result, SourceSpan, Wanted},
    parser::LogixParser,
    token::{Literal, StrLit, Token},
    type_trait::{LogixTypeDescriptor, LogixValueDescriptor, Value},
    LogixType,
};
use std::{
    borrow::Cow,
    ffi::OsStr,
    fmt,
    path::{Path, PathBuf},
};

macro_rules! impl_path_type_traits {
    ($name:ident, $doc:literal) => {
        impl $name {
            pub fn from_lit(lit: StrLit, span: &SourceSpan) -> Result<Self> {
                lit.decode_str(span)?
                    .try_into()
                    .map_err(|error| ParseError::PathError {
                        span: span.clone(),
                        error,
                    })
            }
        }

        impl<'a> TryFrom<&'a str> for $name {
            type Error = PathError;

            fn try_from(v: &'a str) -> Result<Self, PathError> {
                PathBuf::from(v).try_into()
            }
        }

        impl<'a> TryFrom<Cow<'a, str>> for $name {
            type Error = PathError;

            fn try_from(v: Cow<'a, str>) -> Result<Self, PathError> {
                v.into_owned().try_into()
            }
        }

        impl TryFrom<String> for $name {
            type Error = PathError;

            fn try_from(v: String) -> Result<Self, PathError> {
                PathBuf::from(v).try_into()
            }
        }

        impl<'a> TryFrom<&'a Path> for $name {
            type Error = PathError;

            fn try_from(v: &'a Path) -> Result<Self, PathError> {
                v.to_path_buf().try_into()
            }
        }

        impl<'a> TryFrom<Cow<'a, Path>> for $name {
            type Error = PathError;

            fn try_from(v: Cow<'a, Path>) -> Result<Self, PathError> {
                v.into_owned().try_into()
            }
        }

        impl std::ops::Deref for $name {
            type Target = Path;

            fn deref(&self) -> &Self::Target {
                self.as_path()
            }
        }

        impl AsRef<Path> for $name {
            fn as_ref(&self) -> &Path {
                self.as_path()
            }
        }

        impl AsRef<::std::ffi::OsStr> for $name {
            fn as_ref(&self) -> &std::ffi::OsStr {
                self.as_path().as_ref()
            }
        }

        impl PartialEq<PathBuf> for $name {
            fn eq(&self, rhs: &PathBuf) -> bool {
                self.as_path() == rhs
            }
        }

        impl<'a> PartialEq<&'a Path> for $name {
            fn eq(&self, rhs: &&'a Path) -> bool {
                self.as_path() == *rhs
            }
        }

        impl fmt::Debug for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                fmt::Debug::fmt(&**self, f)
            }
        }

        impl LogixType for $name {
            fn descriptor() -> &'static LogixTypeDescriptor {
                static DESC: LogixTypeDescriptor = LogixTypeDescriptor {
                    name: stringify!($name),
                    doc: $doc,
                    value: LogixValueDescriptor::Native,
                };
                &DESC
            }

            fn default_value() -> Option<Self> {
                None
            }

            fn logix_parse<FS: LogixVfs>(p: &mut LogixParser<FS>) -> Result<Value<Self>> {
                match p.next_token()? {
                    (span, Token::Literal(Literal::Str(value))) => Ok(Value {
                        value: Self::from_lit(value, &span)?,
                        span,
                    }),
                    (span, token) => Err(ParseError::UnexpectedToken {
                        span,
                        while_parsing: Self::descriptor().name,
                        wanted: Wanted::$name,
                        got_token: token.token_type_name(),
                    }),
                }
            }
        }
    };

    ($name:ident, $doc:literal, ($($variant:ident),+), $error:ident) => {
        impl_path_type_traits!($name, $doc);

        impl $name {
            pub fn as_path(&self) -> &Path {
                &self.path
            }

            pub fn join(&self, path: impl AsRef<Path>) -> Result<Self, PathError> {
                let path = path.as_ref();
                if path.is_absolute() {
                    Err(PathError::JoinAbsolute)
                } else {
                    Ok(Self { path: self.path.join(path) })
                }
            }

            pub fn with_file_name(&self, name: impl AsRef<OsStr>) -> Self {
                Self { path: self.path.with_file_name(name) }
            }

            pub fn with_extension(&self, ext: impl AsRef<OsStr>) -> Self {
                Self { path: self.path.with_extension(ext) }
            }
        }

        impl From<$name> for PathBuf {
            fn from(v: $name) -> Self {
                v.path
            }
        }

        impl TryFrom<PathBuf> for $name {
            type Error = PathError;

            fn try_from(path: PathBuf) -> Result<Self, PathError> {
                match ValidPath::try_from(path)? {
                    $(ValidPath::$variant(ret) => Ok(Self { path: PathBuf::from(ret) }),)+
                    _ => Err(PathError::$error),
                }
            }
        }
    };
}

/// Represents a validated full path
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ValidPath {
    Full(FullPath),
    Rel(RelPath),
    Name(NameOnlyPath),
}

impl ValidPath {
    pub fn as_path(&self) -> &Path {
        let (ValidPath::Full(FullPath { path })
        | ValidPath::Rel(RelPath { path })
        | ValidPath::Name(NameOnlyPath { path })) = self;
        path
    }

    pub fn join(&self, path: impl AsRef<Path>) -> Result<Self, PathError> {
        match self {
            Self::Full(v) => v.join(path).map(Self::Full),
            Self::Rel(v) => v.join(path).map(Self::Rel),
            Self::Name(v) => v.join(path).map(Self::Name),
        }
    }

    pub fn with_file_name(&self, name: impl AsRef<OsStr>) -> Self {
        match self {
            Self::Full(v) => Self::Full(v.with_file_name(name)),
            Self::Rel(v) => Self::Rel(v.with_file_name(name)),
            Self::Name(v) => Self::Name(v.with_file_name(name)),
        }
    }

    pub fn with_extension(&self, ext: impl AsRef<OsStr>) -> Self {
        match self {
            Self::Full(v) => Self::Full(v.with_extension(ext)),
            Self::Rel(v) => Self::Rel(v.with_extension(ext)),
            Self::Name(v) => Self::Name(v.with_extension(ext)),
        }
    }
}

impl From<ValidPath> for PathBuf {
    fn from(v: ValidPath) -> Self {
        let (ValidPath::Full(FullPath { path })
        | ValidPath::Rel(RelPath { path })
        | ValidPath::Name(NameOnlyPath { path })) = v;
        path
    }
}

impl TryFrom<PathBuf> for ValidPath {
    type Error = PathError;

    fn try_from(path: PathBuf) -> Result<Self, PathError> {
        let raw_bytes = path.as_os_str().as_encoded_bytes();
        // NOTE: We are stricter than the actual operating system as we exclude inconvinient characters
        if let Some(i) = raw_bytes.find_byteset(b"\n|\"'") {
            Err(PathError::InvalidChar(char::from(raw_bytes[i])))
        } else if path == PathBuf::new() {
            Err(PathError::EmptyPath)
        } else if path.is_absolute() {
            Ok(Self::Full(FullPath { path }))
        } else {
            let mut it = path.components();
            if matches!(it.next(), Some(std::path::Component::Normal(_))) && it.next().is_none() {
                Ok(Self::Name(NameOnlyPath { path }))
            } else {
                Ok(Self::Rel(RelPath { path }))
            }
        }
    }
}

impl_path_type_traits!(ValidPath, "A valid path");

/// Represents a validated full path
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FullPath {
    path: PathBuf,
}

impl_path_type_traits!(FullPath, "A full path", (Full), NotAbsolute);

/// Represents a validated relative path
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RelPath {
    path: PathBuf,
}

impl_path_type_traits!(RelPath, "A relative path", (Rel, Name), NotRelative);

/// Represents a validated file or directory name without any path components
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NameOnlyPath {
    path: PathBuf,
}

impl_path_type_traits!(NameOnlyPath, "A file or directory name", (Name), NotName);

impl LogixType for PathBuf {
    fn descriptor() -> &'static LogixTypeDescriptor {
        static RET: LogixTypeDescriptor = LogixTypeDescriptor {
            name: "path",
            doc: "a valid path",
            value: LogixValueDescriptor::Native,
        };
        &RET
    }

    fn default_value() -> Option<Self> {
        None
    }

    fn logix_parse<FS: LogixVfs>(p: &mut LogixParser<FS>) -> Result<Value<Self>> {
        Ok(ValidPath::logix_parse(p)?.map(|v| v.into()))
    }
}
