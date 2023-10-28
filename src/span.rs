use std::{borrow::Cow, fmt, ops::Range, path::Path};

use bstr::ByteSlice;
use logix_vfs::LogixVfs;

use crate::{loader::CachedFile, LogixLoader};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct SourceSpan {
    file: CachedFile,
    pos: usize,
    line: usize,
    col: Range<u16>,
}

impl SourceSpan {
    pub fn empty() -> Self {
        Self {
            file: CachedFile::empty(),
            pos: 0,
            line: 0,
            col: 0..0,
        }
    }
    pub fn new_for_test(
        loader: &LogixLoader<impl LogixVfs>,
        path: impl AsRef<Path>,
        line: usize,
        col: usize,
        len: usize,
    ) -> Self {
        let file = loader.get_file(path).unwrap();
        let mut pos = 0;
        for (i, line_data) in file.data().lines_with_terminator().enumerate() {
            if i + 1 == line {
                pos += col;
                break;
            }
            pos += line_data.len();
        }

        Self::new(&file, pos, line, col, len)
    }

    pub(crate) fn new(file: &CachedFile, pos: usize, line: usize, col: usize, len: usize) -> Self {
        let scol = u16::try_from(col).unwrap();
        let ecol = u16::try_from(col + len).unwrap();
        Self {
            file: file.clone(),
            pos,
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

    pub fn value(&self) -> Cow<str> {
        String::from_utf8_lossy(&self.file.data()[self.pos..self.pos + usize::from(self.col.len())])
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
            pos: self.pos + usize::from(off),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn coverage_hacks() {
        assert_eq!(
            SourceSpan {
                file: CachedFile::from_slice("test.logix", b"hello world"),
                pos: 6,
                line: 1,
                col: 6..11,
            }
            .value(),
            "world"
        );
        assert_eq!(SourceSpan::empty().value(), "");
    }

    #[test]
    fn line_width() {
        assert_eq!(SourceSpan::empty().calc_ln_width(0), 3);
        assert_eq!(SourceSpan::empty().calc_ln_width(10), 3);
        assert_eq!(SourceSpan::empty().calc_ln_width(100), 3);
        assert_eq!(SourceSpan::empty().calc_ln_width(1000), 4);
        assert_eq!(SourceSpan::empty().calc_ln_width(10000), 5);
        assert_eq!(SourceSpan::empty().calc_ln_width(100000), 6);
        assert_eq!(SourceSpan::empty().calc_ln_width(1000000), 10);
        assert_eq!(SourceSpan::empty().calc_ln_width(10000000), 10);
        assert_eq!(SourceSpan::empty().calc_ln_width(100000000), 10);
        assert_eq!(SourceSpan::empty().calc_ln_width(1000000000), 10);
        assert_eq!(SourceSpan::empty().calc_ln_width(10000000000), 10);
    }
}
