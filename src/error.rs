use crate::loader::CachedFile;
use bstr::ByteSlice;
use core::fmt;
use owo_colors::OwoColorize;
use std::{borrow::Cow, ops::Range};

use logix_vfs::LogixVfs;
use thiserror::Error;

use crate::{token::Token, Str};

pub type Result<T> = std::result::Result<T, ParseError>;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct SourceSpan {
    file: CachedFile,
    line: usize,
    col: Range<u16>,
}

impl SourceSpan {
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
}

#[derive(Debug)]
pub enum Wanted {
    Token(Token<'static>),
    Tokens(&'static [Token<'static>]),
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
        }
    }
}

#[derive(Error)]
pub enum ParseError {
    #[error(transparent)]
    FsError(#[from] logix_vfs::Error),

    #[error("Warning treated as error: {0}")]
    Warning(Warn),

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

    #[error("Unexpected token while parsing {while_parsing}, expected {wanted}")]
    UnexpectedToken {
        span: SourceSpan,
        while_parsing: &'static str,
        got_token: &'static str,
        wanted: Wanted,
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
}

impl fmt::Debug for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f)?;
        match self {
            Self::UnexpectedToken {
                span,
                while_parsing,
                got_token,
                wanted,
            } => {
                let ln_width = calc_ln_width(span.line + 1);
                writeln!(
                    f,
                    "{}{}",
                    "error: ".bright_red().bold(),
                    format_args!("Unexpected {got_token} while parsing {while_parsing}").bold()
                )?;

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
                            "^".repeat(span.len()).bright_red().bold(),
                            format_args!("Expected {wanted}").bright_red().bold(),
                        )?;
                    }
                }
                Ok(())
            }
            _ => writeln!(f, "TODO(2023.10): {self}"),
        }
    }
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

#[derive(Error, Debug)]
pub enum Warn {
    #[error("Duplicate map entry {key:?}")]
    DuplicateMapEntry { span: SourceSpan, key: Str },
}
