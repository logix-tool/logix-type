pub use crate::span::SourceSpan;
use core::fmt;
use owo_colors::OwoColorize;

use logix_vfs::LogixVfs;
use thiserror::Error;

use crate::{token::Token, Str};

pub type Result<T> = std::result::Result<T, ParseError>;

#[derive(Error, PartialEq, Debug)]
pub enum TokenError {
    #[error("invalid utf-8 sequence")]
    LitStrNotUtf8,
    #[error("unexpected character {0:?}")]
    UnexpectedChar(char),
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

#[derive(Debug, PartialEq)]
pub enum Wanted {
    Token(Token<'static>),
    Tokens(&'static [Token<'static>]),
    LitStr,
}

impl fmt::Display for Wanted {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Token(token) => fmt::Display::fmt(token, f),
            Self::Tokens([token]) => fmt::Display::fmt(token, f),
            Self::Tokens([a, b]) => write!(f, "either {a} or {b}"),
            Self::Tokens(tokens) => {
                let (first, tokens) = tokens.split_first().unwrap();
                let (last, tokens) = tokens.split_last().unwrap();

                write!(f, "one of {first}")?;
                for token in tokens {
                    write!(f, ", {token}")?;
                }
                write!(f, ", or {last}")
            }
            Self::LitStr => write!(f, "string"),
        }
    }
}

#[derive(Error, PartialEq)]
pub enum ParseError {
    #[error(transparent)]
    FsError(#[from] logix_vfs::Error),

    #[error("Warning treated as error: {0}")]
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
}

impl ParseError {
    pub(crate) fn read_error(e: std::io::Error) -> Self {
        match e.kind() {
            unk => todo!("{e:?} => {unk:?}"),
        }
    }
}

impl fmt::Debug for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f)?;
        match self {
            Self::FsError(_) => todo!(),
            Self::Warning(_) => todo!(),
            Self::MissingStructMember {
                span,
                type_name,
                member,
            } => write_error(
                f,
                format_args!("Missing struct member while parsing `{type_name}`"),
                span,
                format_args!("Expected `{member}`"),
            ),
            Self::DuplicateStructMember {
                span,
                type_name,
                member,
            } => write_error(
                f,
                format_args!("Duplicate struct member while parsing `{type_name}`"),
                span,
                format_args!("Unexpected `{member}`"),
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
                format_args!("Expected {wanted}"),
            ),
            Self::StrEscError { span, error } => {
                write_error(f, "Failed to parse escaped string", span, error)
            }
            Self::TokenError { span, error } => {
                write_error(f, "Failed to parse input", span, error)
            }
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
                "^".repeat(span.len().max(1)).bright_red().bold(),
                expected.bright_red().bold(),
            )?;
        }
    }

    Ok(())
}

#[derive(Error)]
#[error("{error}")]
pub struct LoaderError<'fs, FS: LogixVfs> {
    fs: &'fs FS,
    error: ParseError,
}

impl<'fs, FS: LogixVfs> fmt::Debug for LoaderError<'fs, FS> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.error)
    }
}

#[derive(Error, Debug, PartialEq)]
pub enum Warn {
    #[error("Duplicate map entry {key:?}")]
    DuplicateMapEntry { span: SourceSpan, key: Str },
}
