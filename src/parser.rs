use crate::{
    error::{ParseError, Result, SourceSpan, Warn},
    token::{Brace, Token, TokenState},
    type_trait::Value,
    LogixType,
};
use bstr::BString;
use logix_vfs::LogixVfs;
use smol_str::SmolStr;
use std::{io::Read, path::Path, sync::Arc};

pub struct LogixParser<FS: LogixVfs> {
    path: Arc<Path>,
    fs: Arc<FS>,
    r: FS::RoFile,
    buf: BString,
    file_pos: usize,
    pos: usize,
    state: TokenState,
    eof: bool,
}

impl<FS: LogixVfs> LogixParser<FS> {
    const READ_SIZE: usize = Token::MAX_LEN * 16;

    pub(crate) fn new(path: Arc<Path>, fs: Arc<FS>, r: FS::RoFile) -> Self {
        Self {
            path,
            fs,
            r,
            buf: BString::new(Vec::new()),
            file_pos: 0,
            pos: 0,
            state: TokenState::Normal,
            eof: false,
        }
    }

    pub fn next_token(&mut self) -> Result<Option<(SourceSpan<FS>, Token)>, FS> {
        Ok(self
            .raw_next_token(true)?
            .map(|(span, _, token)| (span, token)))
    }

    pub fn warning(&self, warning: Warn<FS>) -> Result<(), FS> {
        // TODO(2023.10): Make it possible to allow warnings
        Err(ParseError::Warning(warning))
    }

    fn raw_next_token(
        &mut self,
        advance: bool,
    ) -> Result<Option<(SourceSpan<FS>, usize, Token)>, FS> {
        if !self.eof && self.buf.len() - self.pos < Token::MAX_LEN {
            self.buf.drain(0..self.pos);
            self.file_pos += self.pos;
            self.pos = 0;

            let to_read = Self::READ_SIZE - self.buf.len();
            let actual = self
                .r
                .by_ref()
                .take(to_read.try_into().unwrap())
                .read_to_end(&mut self.buf)
                .map_err(ParseError::read_error)?;
            self.eof = actual < to_read;
        }

        if self.buf.len() == self.pos {
            return Ok(None);
        }

        match self.state.parse_token(&self.buf[self.pos..]) {
            Ok(Some((range, size, token))) => {
                let file_offset = self.file_pos + self.pos;
                let span = SourceSpan {
                    fs: self.fs.clone(),
                    path: self.path.clone(),
                    range: range.start + file_offset..range.end + file_offset,
                };
                if advance {
                    self.pos += size;
                }
                Ok(Some((span, size, token)))
            }
            unk => todo!("{unk:?}"),
        }
    }

    pub fn read_key_value<T: LogixType>(
        &mut self,
        end_brace: Brace,
    ) -> Result<Option<(Value<FS, SmolStr>, Value<FS, T>)>, FS> {
        match self.next_token()? {
            Some((span, Token::Ident(key))) => {
                let key = Value {
                    value: SmolStr::new(key),
                    span,
                };

                match self.next_token()? {
                    Some((_, Token::Colon)) => {}
                    unk => todo!("{unk:#?}"),
                }

                let value = T::logix_parse(self)?;

                match self.next_token()? {
                    Some((_, Token::Newline)) => {}
                    unk => todo!("{unk:#?}"),
                }

                Ok(Some((key, value)))
            }
            Some((_, Token::BraceEnd(brace))) if brace == end_brace => Ok(None),
            unk => todo!("{unk:#?}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use logix_vfs::RelFs;

    use crate::token::Brace;

    use super::*;

    fn s<FS: LogixVfs>(fs: &Arc<FS>, start: usize, len: usize) -> SourceSpan<FS> {
        SourceSpan {
            fs: fs.clone(),
            path: Arc::from(Path::new("test.logix")),
            range: start..start + len,
        }
    }

    #[test]
    fn basics() -> Result<(), impl LogixVfs> {
        let root = tempfile::tempdir().unwrap();
        let path = Arc::<Path>::from(Path::new("test.logix"));
        let fs = Arc::new(RelFs::new(root.path()));
        std::fs::write(root.path().join(&path), b"Hello { world: \"!!!\" }").unwrap();
        let file = fs.open_file(&path)?;
        let mut p = LogixParser::new(path, fs.clone(), file);

        assert_eq!(p.next_token()?, Some((s(&fs, 0, 5), Token::Ident("Hello"))));
        assert_eq!(
            p.next_token()?,
            Some((s(&fs, 6, 1), Token::BraceStart(Brace::Curly)))
        );
        assert_eq!(p.next_token()?, Some((s(&fs, 8, 5), Token::Ident("world"))));
        assert_eq!(p.next_token()?, Some((s(&fs, 13, 1), Token::Colon)));
        assert_eq!(
            p.next_token()?,
            Some((
                s(&fs, 16, 3),
                Token::LitStrChunk {
                    chunk: "!!!",
                    last: true
                }
            ))
        );
        assert_eq!(
            p.next_token()?,
            Some((s(&fs, 21, 1), Token::BraceEnd(Brace::Curly)))
        );
        assert_eq!(p.next_token()?, None);
        Ok(())
    }
}
