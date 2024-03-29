pub use crate::span::SourceSpan;
use core::fmt;
use owo_colors::OwoColorize;

use thiserror::Error;

use crate::{
    token::{StrTag, StrTagSuffix, Token},
    types::ShortStr,
};

pub type Result<T, E = ParseError> = std::result::Result<T, E>;

#[derive(Error, PartialEq, Debug)]
pub enum TokenError {
    #[error("invalid utf-8 sequence")]
    LitStrNotUtf8,
    #[error("unexpected character {0:?}")]
    UnexpectedChar(char),
    #[error("unexpected end of file, expected `*/`")]
    MissingCommentTerminator,
    #[error("unknown string tag `{0}`")]
    UnknownStrTag(ShortStr),
    #[error("unexpected end of the string, expected `\"`")]
    MissingStringTerminator,
    #[error("unexpected end of {tag} string, expected {suffix}")]
    MissingTaggedStringTerminator { tag: StrTag, suffix: StrTagSuffix },
}

#[derive(Error, PartialEq, Debug)]
pub enum EscStrError {
    #[error("got truncated hex escape code")]
    TruncatedHex,
    #[error("got invalid hex escape code")]
    InvalidHex,
    #[error("got invalid unicode hex escape code")]
    InvalidUnicodeHex,
    #[error("the code point U+{0:x} is invalid")]
    InvalidUnicodePoint(u32),
    #[error("got invalid unicode escape, expected `{{`")]
    InvalidUnicodeMissingStartBrace,
    #[error("got invalid unicode escape, expected `}}`")]
    InvalidUnicodeMissingEndBrace,
    #[error("got invalid escape character {0:?}")]
    InvalidEscapeChar(char),
}

#[derive(Error, PartialEq, Debug)]
pub enum IncludeError {
    #[error("invalid utf-8 sequence")]
    NotUtf8,

    #[error(transparent)]
    Open(logix_vfs::Error),
}

#[derive(Error, PartialEq, Debug, Clone, Copy)]
pub enum PathError {
    #[error("expected an absolute path")]
    NotAbsolute,

    #[error("expected a relative path")]
    NotRelative,

    #[error("the specified path is empty")]
    EmptyPath,

    #[error("expected either a file or directory name")]
    NotName,

    #[error("the path contains the invalid character {0:?}")]
    InvalidChar(char),

    #[error("expected file name or absolute path")]
    NotFullOrNameOnly,
}

#[derive(Debug, PartialEq)]
pub enum Wanted {
    Token(Token<'static>),
    Tokens(&'static [Token<'static>]),
    LitStr,
    FullPath,
    RelPath,
    ExecutablePath,
    NameOnlyPath,
    ValidPath,
    LitNum(&'static str),
    Ident,
}

impl fmt::Display for Wanted {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Token(token) => token.write_token_display_name(f),
            Self::Tokens([token]) => token.write_token_display_name(f),
            Self::Tokens([a, b]) => {
                write!(f, "either ")?;
                a.write_token_display_name(f)?;
                write!(f, " or ")?;
                b.write_token_display_name(f)
            }
            Self::Tokens(tokens) => {
                let (first, tokens) = tokens.split_first().unwrap();
                let (last, tokens) = tokens.split_last().unwrap();

                write!(f, "one of ")?;
                first.write_token_display_name(f)?;
                for token in tokens {
                    write!(f, ", ")?;
                    token.write_token_display_name(f)?;
                }
                write!(f, ", or ")?;
                last.write_token_display_name(f)
            }
            Self::LitStr => write!(f, "string"),
            Self::FullPath => write!(f, "full path"),
            Self::RelPath => write!(f, "relative path"),
            Self::ExecutablePath => write!(f, "executable name or full path"),
            Self::NameOnlyPath => write!(f, "file or directory name"),
            Self::ValidPath => write!(f, "path"),
            Self::LitNum(name) => write!(f, "{name}"),
            Self::Ident => write!(f, "identifier"),
        }
    }
}

#[derive(Error, PartialEq)]
pub enum ParseError {
    #[error(transparent)]
    FsError(#[from] logix_vfs::Error),

    #[error(transparent)]
    Warning(Warn),

    #[error("Missing struct member `{member}` while parsing `{type_name}` in {span}")]
    MissingStructMember {
        span: SourceSpan,
        type_name: &'static str,
        member: &'static str,
    },

    #[error("Duplicate struct member `{member}` while parsing `{type_name}` in {span}")]
    DuplicateStructMember {
        span: SourceSpan,
        type_name: &'static str,
        member: &'static str,
    },

    #[error("Unexpected {got_token} while parsing `{while_parsing}`, expected {wanted} in {span}")]
    UnexpectedToken {
        span: SourceSpan,
        while_parsing: &'static str,
        got_token: &'static str,
        wanted: Wanted,
    },

    #[error("Failed to parse string, {error} in {span}")]
    StrEscError {
        span: SourceSpan,
        error: EscStrError,
    },

    #[error("Failed to parse input, {error} in {span}")]
    TokenError { span: SourceSpan, error: TokenError },

    #[error("Failed to include file as `{while_parsing}`, {error} in {span}")]
    IncludeError {
        span: SourceSpan,
        while_parsing: &'static str,
        error: IncludeError,
    },

    #[error("Failed to parse path, {error} in {span}")]
    PathError { span: SourceSpan, error: PathError },
}

impl fmt::Debug for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f)?;
        match self {
            Self::FsError(e) => writeln!(f, "{}{}", "error: ".bright_red().bold(), e.bold()),
            Self::Warning(Warn::DuplicateMapEntry { span, key }) => write_error(
                f,
                format_args!("Duplicate entry `{key}` while parsing `Map`"),
                span,
                format_args!("overwrites the previous entry"),
            ),
            Self::MissingStructMember {
                span,
                type_name,
                member,
            } => write_error(
                f,
                format_args!("Missing struct member while parsing `{type_name}`"),
                span,
                format_args!("expected `{member}`"),
            ),
            Self::DuplicateStructMember {
                span,
                type_name,
                member,
            } => write_error(
                f,
                format_args!("Duplicate struct member while parsing `{type_name}`"),
                span,
                format_args!("unexpected `{member}`"),
            ),
            Self::UnexpectedToken {
                span,
                while_parsing,
                got_token,
                wanted,
            } => write_error(
                f,
                format_args!("Unexpected {got_token} while parsing `{while_parsing}`"),
                span,
                format_args!("expected {wanted}"),
            ),
            Self::StrEscError { span, error } => {
                write_error(f, "Failed to parse escaped string", span, error)
            }
            Self::TokenError { span, error } => {
                write_error(f, "Failed to parse input", span, error)
            }
            Self::IncludeError {
                span,
                while_parsing,
                error,
            } => write_error(
                f,
                format_args!("Failed to include file as `{while_parsing}`"),
                span,
                error,
            ),
            Self::PathError { span, error } => write_error(f, "Failed to parse path", span, error),
        }
    }
}

fn write_error(
    f: &mut impl fmt::Write,
    message: impl fmt::Display,
    span: &SourceSpan,
    expected: impl fmt::Display,
) -> fmt::Result {
    let context = 1;
    let ln_width = span.calc_ln_width(context);
    writeln!(f, "{}{}", "error: ".bright_red().bold(), message.bold())?;

    writeln!(
        f,
        "   {} {}:{}:{}",
        "--->".bright_blue().bold(),
        span.path().display(),
        span.line(),
        span.col(),
    )?;
    writeln!(f, "{:>ln_width$} {}", "", "|".bright_blue().bold(),)?;

    for (ln, span, line) in span.lines(context) {
        writeln!(
            f,
            "{:>ln_width$} {} {}",
            ln.bright_blue().bold(),
            "|".bright_blue().bold(),
            line.trim_end(),
        )?;
        if let Some(span) = span {
            let col = span.start;
            writeln!(
                f,
                "{:>ln_width$} {} {:>col$}{} {}",
                "",
                "|".bright_blue().bold(),
                "",
                "^".repeat(
                    line.get(span)
                        .into_iter()
                        .flat_map(|s| s.chars())
                        .count()
                        .max(1)
                )
                .bright_red()
                .bold(),
                expected.bright_red().bold(),
            )?;
        }
    }

    Ok(())
}

#[derive(Error, Debug, PartialEq)]
pub enum Warn {
    #[error(
        "Duplicate entry `{key}` while parsing `Map`, overwrites the previous entry in {span}"
    )]
    DuplicateMapEntry { span: SourceSpan, key: ShortStr },
}

#[cfg(test)]
mod tests {
    use std::{error::Error, path::PathBuf};

    use super::*;

    #[test]
    fn coverage_hacks() {
        assert_eq!(
            format!("{:?}", EscStrError::InvalidUnicodeHex),
            "InvalidUnicodeHex",
        );
        assert_eq!(format!("{:?}", TokenError::LitStrNotUtf8), "LitStrNotUtf8",);
        assert_eq!(format!("{:?}", Wanted::LitStr), "LitStr",);
        assert_eq!(
            format!(
                "{:?}",
                Warn::DuplicateMapEntry {
                    span: SourceSpan::empty(),
                    key: "test".into()
                }
            ),
            "DuplicateMapEntry { span: SourceSpan { file: CachedFile(\"\"), pos: 0, line: 0, col: 0..0 }, key: \"test\" }",
        );
        assert_eq!(
            ParseError::FsError(logix_vfs::Error::NotFound {
                path: PathBuf::new()
            })
            .source()
            .map(|e| e.to_string()),
            None,
        );
    }
}
