use core::fmt;
use owo_colors::OwoColorize;
use std::{
    collections::HashMap,
    io::{BufRead, BufReader},
    ops::Range,
    path::PathBuf,
};

use logix_vfs::LogixVfs;
use thiserror::Error;

use crate::{loader::FileId, token::Token, LogixLoader, Str};

pub type Result<T> = std::result::Result<T, ParseError>;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct SourceSpan {
    file: FileId,
    range: Range<usize>,
}

impl SourceSpan {
    pub(crate) fn new(file: FileId, range: Range<usize>) -> Self {
        Self { file, range }
    }
}

#[derive(Error, Debug)]
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

    #[error("Unexpected token {got} while parsing {while_parsing}, expected {wanted:?}")]
    UnexpectedToken {
        span: SourceSpan,
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
        if let Some((span, got)) = got {
            Self::UnexpectedToken {
                span,
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

enum Prefix {
    Error,
}

struct Issue {
    line: usize,
    col: usize,
    len: usize,
    message: String,
}

struct IssueGroup {
    prefix: Prefix,
    file: FileId,
    issues: Vec<Issue>,
    message: String,
}

struct FileCache {
    path: PathBuf,
    lines: Vec<(usize, Vec<u8>)>,
}

#[derive(Default)]
pub struct HumanReport {
    lines: HashMap<FileId, FileCache>,
    issues: Vec<IssueGroup>,
}

impl HumanReport {
    fn resolve_span<FS: LogixVfs>(
        &mut self,
        loader: &LogixLoader<FS>,
        span: &SourceSpan,
    ) -> (FileId, usize, usize) {
        let entry = self.lines.entry(span.file).or_insert_with(|| {
            let (path, file) = loader.open_file_by_id(span.file).unwrap();
            let mut i = 0;
            let lines = BufReader::new(file)
                .split(b'\n')
                .map(|b| {
                    let ret = (i, b.unwrap());
                    i += ret.1.len();
                    ret
                })
                .collect();
            FileCache { path, lines }
        });
        match entry.lines.binary_search_by_key(&span.range.start, |v| v.0) {
            Ok(i) => (span.file, i, 0),
            Err(0) => (span.file, 0, span.range.start),
            Err(i) => (span.file, i - 1, span.range.start - entry.lines[i - 1].0),
        }
    }

    fn push_issue(&mut self, file: FileId, prefix: Prefix, message: String) -> &mut IssueGroup {
        self.issues.push(IssueGroup {
            prefix,
            file,
            issues: Vec::new(),
            message,
        });
        self.issues.last_mut().unwrap()
    }

    pub fn from_parse_error<FS: LogixVfs>(loader: &LogixLoader<FS>, e: ParseError) -> Self {
        let mut report = Self::default();
        match &e {
            ParseError::UnexpectedToken {
                span,
                while_parsing,
                wanted,
                got,
            } => {
                let (file, line, col) = report.resolve_span(loader, span);
                let group =
                    report.push_issue(file, Prefix::Error, "Unexpected token while parsing".into());
                group.issues.push(Issue {
                    line,
                    col,
                    len: span.range.len(),
                    message: format_args!("expected {wanted:?}")
                        .bright_red()
                        .bold()
                        .to_string(),
                });
            }
            unk => todo!("{unk:#?}"),
        }
        report
    }
}

impl fmt::Debug for HumanReport {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fn calc_ln_width(i: usize) -> usize {
            match i + 1 {
                0..=999 => 3,
                1000..=9999 => 4,
                10000..=99999 => 5,
                100000..=999999 => 6,
                _ => 10,
            }
        }

        for group in &self.issues {
            let file = &self.lines[&group.file];
            match group.prefix {
                Prefix::Error => writeln!(
                    f,
                    "{}{}",
                    "error: ".bright_red().bold(),
                    group.message.bold()
                )?,
            }
            writeln!(
                f,
                "   {} {}",
                "--->".bright_blue().bold(),
                file.path.display()
            )?;

            for issue in &group.issues {
                let line = String::from_utf8_lossy(&file.lines[issue.line].1);
                let ln_width = calc_ln_width(issue.line);
                writeln!(
                    f,
                    "{:>ln_width$} {} {}",
                    (issue.line + 1).bright_blue().bold(),
                    "|".bright_blue().bold(),
                    line.trim_end(),
                )?;
                writeln!(
                    f,
                    "{estr:ln_width$} {pipe:}{estr:col$}{point:} {message:}",
                    estr = "",
                    pipe = "|".bright_blue().bold(),
                    point = "^".repeat(issue.len).bright_red().bold(),
                    message = issue.message,
                    col = issue.col
                )?;
            }
        }

        Ok(())
    }
}

#[derive(Error, Debug)]
pub enum Warn {
    #[error("Duplicate map entry {key:?}")]
    DuplicateMapEntry { span: SourceSpan, key: Str },
}
