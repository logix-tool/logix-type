use core::fmt;
use std::{ops::Range, path::Path, sync::Arc};

use logix_vfs::LogixVfs;
use miette::Diagnostic;
use thiserror::Error;

use crate::{token::Token, Str};

pub type Result<T, FS> = std::result::Result<T, ParseError<FS>>;

#[derive(Debug, Eq)]
pub struct SourceSpan<FS: LogixVfs> {
    pub fs: Arc<FS>,
    pub path: Arc<Path>,
    pub range: Range<usize>,
}

impl<FS1: LogixVfs, FS2: LogixVfs> PartialEq<SourceSpan<FS1>> for SourceSpan<FS2> {
    fn eq(&self, other: &SourceSpan<FS1>) -> bool {
        let Self { fs: _, path, range } = self;
        (path.as_ref(), range) == (other.path.as_ref(), &other.range)
    }
}

impl<'a, FS: LogixVfs> From<&'a SourceSpan<FS>> for miette::SourceSpan {
    fn from(_: &'a SourceSpan<FS>) -> Self {
        todo!()
    }
}

#[derive(Error, Diagnostic, Debug)]
pub enum ParseError<FS: LogixVfs> {
    #[error(transparent)]
    FsError(#[from] logix_vfs::Error),

    #[error("Warning treated as error: {0}")]
    Warning(Warn<FS>),

    #[error("Missing member {type_name} in {member}")]
    MissingMember {
        type_name: &'static str,
        member: &'static str,
    },

    #[error("Duplicate member {type_name} in {member}")]
    DuplicateMember {
        type_name: &'static str,
        member: &'static str,
    },

    #[error("Unexpected end of file while parsing {while_parsing}, expected {wanted:?}")]
    UnexpectedEndOfFile {
        while_parsing: &'static str,
        wanted: &'static str,
    },

    #[error("Unexpected token {got} while parsing {while_parsing}, expected {wanted:?}")]
    UnexpectedToken {
        #[label("here")]
        at: SourceSpan<FS>,
        while_parsing: &'static str,
        wanted: &'static str,
        got: String,
    },
}

impl<FS: LogixVfs> ParseError<FS> {
    pub(crate) fn read_error(e: std::io::Error) -> Self {
        match e.kind() {
            unk => todo!("{e:?} => {unk:?}"),
        }
    }

    pub fn missing_member(type_name: &'static str, member: &'static str) -> Self {
        Self::MissingMember { type_name, member }
    }

    pub fn duplicate_member(type_name: &'static str, member: &'static str) -> Self {
        Self::DuplicateMember { type_name, member }
    }

    pub fn unexpected_token(
        while_parsing: &'static str,
        wanted: &'static str,
        got: Option<(SourceSpan<FS>, Token)>,
    ) -> Self {
        if let Some((at, got)) = got {
            Self::UnexpectedToken {
                at,
                while_parsing,
                wanted,
                got: format!("{got:?}"),
            }
        } else {
            Self::UnexpectedEndOfFile {
                while_parsing,
                wanted,
            }
        }
    }
}

pub struct HumanDisplay<FS: LogixVfs>(ParseError<FS>);

impl<FS: LogixVfs> fmt::Debug for HumanDisplay<FS> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Error, Diagnostic, Debug)]
pub enum Warn<FS: LogixVfs> {
    #[error("Duplicate map entry {key:?}")]
    DuplicateMapEntry {
        #[label("here")]
        span: SourceSpan<FS>,
        key: Str,
    },
}
