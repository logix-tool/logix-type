use crate::{loader::CachedFile, LogixLoader};
use bstr::ByteSlice;
use core::fmt;
use owo_colors::OwoColorize;
use std::{borrow::Cow, ops::Range, path::Path};

use logix_vfs::LogixVfs;
use thiserror::Error;

use crate::{token::Token, Str};

pub type Result<T> = std::result::Result<T, ParseError>;

#[derive(Error, PartialEq, Debug)]
pub enum EscStrError {
    #[error("got truncated hex escape code")]
    TruncatedHex,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct SourceSpan {
    file: CachedFile,
    line: usize,
    col: Range<u16>,
}

impl SourceSpan {
    pub fn new_for_test(
        loader: &LogixLoader<impl LogixVfs>,
        path: impl AsRef<Path>,
        line: usize,
        col: usize,
        len: usize,
    ) -> Self {
        Self::new(&loader.get_file(path).unwrap(), line, col, len)
    }

    pub(crate) fn new(file: &CachedFile, line: usize, col: usize, len: usize) -> Self {
        let scol = u16::try_from(col).unwrap();
        let ecol = u16::try_from(col + len).unwrap();
        Self {
            file: file.clone(),
            line,
            col: scol..ecol,
        }
    }

    fn lines(
        &self,
        context: usize,
    ) -> impl Iterator<Item = (usize, Option<Range<usize>>, Cow<str>)> {
        self.file
            .lines()
            .enumerate()
            .skip(self.line.saturating_sub(context + 1))
            .map_while(move |(i, line)| {
                let ln = i + 1;
                if ln == self.line {
                    Some((
                        ln,
                        Some(usize::from(self.col.start)..usize::from(self.col.end)),
                        line.to_str_lossy(),
                    ))
                } else if ln <= self.line + context {
                    Some((ln, None, line.to_str_lossy()))
                } else {
                    None
                }
            })
    }

    pub fn with_off(&self, off: usize, len: usize) -> Self {
        let off = u16::try_from(off).unwrap();
        let len = u16::try_from(len).unwrap();
        Self {
            file: self.file.clone(),
            line: self.line,
            col: self.col.start + off..self.col.start + off + len,
        }
    }
}

impl fmt::Display for SourceSpan {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}:{}:{}",
            self.file.path().display(),
            self.line,
            self.col.start
        )
    }
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
        }
    }
}

fn write_error(
    f: &mut impl fmt::Write,
    message: impl fmt::Display,
    span: &SourceSpan,
    expected: impl fmt::Display,
) -> fmt::Result {
    let ln_width = calc_ln_width(span.line + 1);
    writeln!(f, "{}{}", "error: ".bright_red().bold(), message.bold())?;

    writeln!(
        f,
        "   {} {}:{}:{}",
        "--->".bright_blue().bold(),
        span.file.path().display(),
        span.line,
        span.col.start,
    )?;
    writeln!(f, "{:>ln_width$} {}", "", "|".bright_blue().bold(),)?;

    for (ln, span, line) in span.lines(1) {
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

fn calc_ln_width(i: usize) -> usize {
    match i + 1 {
        0..=999 => 3,
        1000..=9999 => 4,
        10000..=99999 => 5,
        100000..=999999 => 6,
        _ => 10,
    }
}

#[derive(Error, Debug, PartialEq)]
pub enum Warn {
    #[error("Duplicate map entry {key:?}")]
    DuplicateMapEntry { span: SourceSpan, key: Str },
}
