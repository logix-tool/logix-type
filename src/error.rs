use std::{ops::Range, path::Path, sync::Arc};

use miette::Diagnostic;
use thiserror::Error;

use crate::token::Token;

pub type Result<T, E = ParseError> = std::result::Result<T, E>;

#[derive(Debug, PartialEq, Eq)]
pub struct SourceSpan {
    pub path: Arc<Path>,
    pub range: Range<usize>,
}

impl<'a> From<&'a SourceSpan> for miette::SourceSpan {
    fn from(_: &'a SourceSpan) -> Self {
        todo!()
    }
}

#[derive(Error, Diagnostic, Debug)]
pub enum ParseError {
    #[error(transparent)]
    FsError(#[from] logix_vfs::Error),

    #[error("String literal is not valid utf-8")]
    LitStrNotUtf8,

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
        at: SourceSpan,
        while_parsing: &'static str,
        wanted: &'static str,
        got: String,
    },
}

impl ParseError {
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
        got: Option<(SourceSpan, Token)>,
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
