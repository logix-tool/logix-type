use std::{borrow::Cow, fmt, path::Path};

use bstr::ByteSlice;
use logix_vfs::LogixVfs;

use crate::{loader::CachedFile, LogixLoader};

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
struct Range {
    start: u16,
    end: u16,
}

impl Range {
    fn len(&self) -> usize {
        usize::from(self.end - self.start)
    }
}

impl fmt::Debug for Range {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let range: std::ops::Range<u16> = self.start..self.end;
        fmt::Debug::fmt(&range, f)
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
enum SpanRange {
    SingleLine {
        line: usize,
        col: Range,
    },
    MultiLine {
        start_line: usize,
        start_col: u16,
        last_line: usize,
        last_col: u16,
        end_pos: usize,
    },
}

impl SpanRange {
    fn get_range_for_line(&self, cur_line: usize) -> Option<std::ops::Range<usize>> {
        match self {
            Self::SingleLine { line, col } => {
                (*line == cur_line).then(|| usize::from(col.start)..usize::from(col.end))
            }
            Self::MultiLine { .. } => todo!(),
        }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub struct SourceSpan {
    file: CachedFile,
    pos: usize,
    range: SpanRange,
}

impl SourceSpan {
    pub fn empty() -> Self {
        Self {
            file: CachedFile::empty(),
            pos: 0,
            range: SpanRange::SingleLine {
                line: 0,
                col: Range { start: 0, end: 0 },
            },
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
            range: SpanRange::SingleLine {
                line,
                col: Range {
                    start: scol,
                    end: ecol,
                },
            },
        }
    }

    pub fn path(&self) -> &Path {
        self.file.path()
    }

    /// The first line in this span
    pub fn line(&self) -> usize {
        match self.range {
            SpanRange::SingleLine { line, col: _ } => line,
            SpanRange::MultiLine { start_line, .. } => start_line,
        }
    }

    /// The last line in this span
    pub fn last_line(&self) -> usize {
        match self.range {
            SpanRange::SingleLine { line, col: _ } => line,
            SpanRange::MultiLine { last_line, .. } => last_line,
        }
    }
    /// The start column of the start line
    pub fn col(&self) -> usize {
        match self.range {
            SpanRange::SingleLine {
                line: _,
                col: Range { start, end: _ },
            } => start.into(),
            SpanRange::MultiLine { start_col, .. } => start_col.into(),
        }
    }

    /// The value of this entire span
    pub fn value(&self) -> Cow<str> {
        let end_pos = match &self.range {
            SpanRange::SingleLine { line: _, col } => self.pos + col.len(),
            &SpanRange::MultiLine { end_pos, .. } => end_pos,
        };
        String::from_utf8_lossy(&self.file.data()[self.pos..end_pos])
    }

    pub fn lines(
        &self,
        context: usize,
    ) -> impl Iterator<Item = (usize, Option<std::ops::Range<usize>>, Cow<str>)> {
        self.file
            .lines()
            .enumerate()
            .skip(self.line().saturating_sub(context + 1))
            .map_while(move |(i, line)| {
                let ln = i + 1;
                if ln <= self.line() + context {
                    Some((ln, self.range.get_range_for_line(ln), line.to_str_lossy()))
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
            range: match self.range {
                SpanRange::SingleLine {
                    line,
                    col: Range { start, end: _ },
                } => SpanRange::SingleLine {
                    line,
                    col: Range {
                        start: start + off,
                        end: start + off + len,
                    },
                },
                SpanRange::MultiLine { .. } => todo!(),
            },
        }
    }

    pub fn calc_ln_width(&self, extra: usize) -> usize {
        match self.last_line() + extra {
            0..=999 => 3,
            1000..=9999 => 4,
            10000..=99999 => 5,
            100000..=999999 => 6,
            _ => 10,
        }
    }

    pub(crate) fn join(&self, other: &Self) -> SourceSpan {
        assert_eq!(self.file, other.file);

        let mut ret = self.clone();

        match (&self.range, &other.range) {
            (
                SpanRange::SingleLine {
                    line: sline,
                    col: scol,
                },
                SpanRange::SingleLine {
                    line: oline,
                    col: ocol,
                },
            ) => match sline.cmp(oline) {
                std::cmp::Ordering::Less => {
                    ret.pos = self.pos;
                    ret.range = SpanRange::MultiLine {
                        start_line: *sline,
                        start_col: scol.start,
                        last_line: *oline,
                        last_col: ocol.end,
                        end_pos: other.pos + ocol.len(),
                    };
                }
                std::cmp::Ordering::Equal => {
                    ret.pos = self.pos.min(other.pos);
                    ret.range = SpanRange::SingleLine {
                        line: *sline,
                        col: Range {
                            start: ocol.start.min(scol.start),
                            end: ocol.end.max(scol.end),
                        },
                    };
                }
                std::cmp::Ordering::Greater => {
                    ret.pos = self.pos;
                    ret.range = SpanRange::MultiLine {
                        start_line: *oline,
                        start_col: ocol.start,
                        last_line: *sline,
                        last_col: scol.end,
                        end_pos: self.pos + scol.len(),
                    };
                }
            },
            (SpanRange::SingleLine { .. }, SpanRange::MultiLine { .. }) => todo!(),
            (SpanRange::MultiLine { .. }, SpanRange::SingleLine { .. }) => todo!(),
            (SpanRange::MultiLine { .. }, SpanRange::MultiLine { .. }) => todo!(),
        }

        ret
    }

    pub(crate) fn from_pos(file: &CachedFile, pos: usize) -> SourceSpan {
        let mut cur = 0;
        let mut ln = 1;
        let mut col = Range { start: 0, end: 0 };

        for line in file.data().lines_with_terminator() {
            let range = cur..cur + line.len();

            if range.contains(&pos) {
                let start = u16::try_from(pos - range.start).unwrap();
                col = Range {
                    start,
                    end: start + 1,
                };
                break;
            }

            cur = range.end;
            ln += 1;
        }

        Self {
            file: file.clone(),
            pos,
            range: SpanRange::SingleLine { line: ln, col },
        }
    }
}

impl fmt::Display for SourceSpan {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}:{}:{}",
            self.file.path().display(),
            self.line(),
            self.col()
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
                range: SpanRange::SingleLine {
                    line: 1,
                    col: Range { start: 6, end: 11 },
                }
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
