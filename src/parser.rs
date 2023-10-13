use crate::{
    error::{ParseError, Result, SourceSpan},
    token::{Token, TokenState},
};
use bstr::BString;
use std::{io::Read, path::Path, sync::Arc};

pub struct LogixParser<R: Read> {
    path: Arc<Path>,
    r: R,
    buf: BString,
    file_pos: usize,
    pos: usize,
    state: TokenState,
    eof: bool,
}

impl<R: Read> LogixParser<R> {
    const READ_SIZE: usize = Token::MAX_LEN * 16;

    pub(crate) fn new(path: Arc<Path>, r: R) -> Self {
        Self {
            path,
            r,
            buf: BString::new(Vec::new()),
            file_pos: 0,
            pos: 0,
            state: TokenState::Normal,
            eof: false,
        }
    }

    pub fn next_token(&mut self) -> Result<Option<(SourceSpan, Token)>> {
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
            Ok(None)
        } else if let Some((range, size, token)) = self.state.parse_token(&self.buf[self.pos..])? {
            let file_offset = self.file_pos + self.pos;
            let span = SourceSpan {
                path: self.path.clone(),
                range: range.start + file_offset..range.end + file_offset,
            };
            self.pos += size;
            Ok(Some((span, token)))
        } else {
            todo!()
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::token::Brace;

    use super::*;

    fn s(start: usize, len: usize) -> SourceSpan {
        SourceSpan {
            path: Arc::from(Path::new("test.logix")),
            range: start..start + len,
        }
    }

    #[test]
    fn basics() -> Result<()> {
        let mut p = LogixParser::new(
            Path::new("test.logix").into(),
            b"Hello { world: \"!!!\" }".as_slice(),
        );
        assert_eq!(p.next_token()?, Some((s(0, 5), Token::Ident("Hello"))));
        assert_eq!(
            p.next_token()?,
            Some((s(6, 1), Token::BraceStart(Brace::Curly)))
        );
        assert_eq!(p.next_token()?, Some((s(8, 5), Token::Ident("world"))));
        assert_eq!(p.next_token()?, Some((s(13, 1), Token::Colon)));
        assert_eq!(
            p.next_token()?,
            Some((
                s(16, 3),
                Token::LitStrChunk {
                    chunk: "!!!",
                    last: true
                }
            ))
        );
        assert_eq!(
            p.next_token()?,
            Some((s(21, 1), Token::BraceEnd(Brace::Curly)))
        );
        assert_eq!(p.next_token()?, None);
        Ok(())
    }
}
