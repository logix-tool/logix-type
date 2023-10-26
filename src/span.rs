use std::{borrow::Cow, fmt, ops::Range, path::Path};

use bstr::ByteSlice;
use logix_vfs::LogixVfs;

use crate::{loader::CachedFile, LogixLoader};

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

    pub fn path(&self) -> &Path {
        self.file.path()
    }

    pub fn line(&self) -> usize {
        self.line
    }

    pub fn col(&self) -> usize {
        self.col.start.into()
    }

    pub fn lines(
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

    pub fn calc_ln_width(&self, extra: usize) -> usize {
        match self.line + extra {
            0..=999 => 3,
            1000..=9999 => 4,
            10000..=99999 => 5,
            100000..=999999 => 6,
            _ => 10,
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
