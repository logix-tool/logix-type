use std::ops::Range;

use crate::{
    error::{ParseError, Result, SourceSpan, Wanted, Warn},
    loader::{CachedFile, LogixLoader},
    token::{Brace, Token, TokenState},
    type_trait::Value,
    LogixType,
};
use logix_vfs::LogixVfs;
use smol_str::SmolStr;

pub struct LogixParser<'fs, 'f, FS: LogixVfs> {
    _loader: &'fs mut LogixLoader<FS>,
    file: &'f CachedFile,
    lines: bstr::Lines<'f>,
    cur_line: &'f [u8],
    cur_col: usize,
    cur_ln: usize,
    state: TokenState,
    eof: bool,
    last_was_newline: bool,
}

impl<'fs, 'f, FS: LogixVfs> LogixParser<'fs, 'f, FS> {
    pub(crate) fn new(loader: &'fs mut LogixLoader<FS>, file: &'f CachedFile) -> Self {
        Self {
            _loader: loader,
            file,
            lines: file.lines(),
            cur_line: b"",
            cur_col: 0,
            cur_ln: 0,
            state: TokenState::Normal,
            eof: false,
            last_was_newline: false,
        }
    }

    pub fn next_token(&mut self) -> Result<Option<(SourceSpan, Token)>> {
        Ok(self
            .raw_next_token(true)?
            .map(|(span, _, token)| (span, token)))
    }

    pub fn warning(&self, warning: Warn) -> Result<()> {
        // TODO(2023.10): Make it possible to allow warnings
        Err(ParseError::Warning(warning))
    }

    fn cur_span(&self, range: Range<usize>) -> SourceSpan {
        SourceSpan::new(
            &self.file,
            self.cur_ln,
            self.cur_col + range.start,
            range.len(),
        )
    }

    fn raw_next_token(&mut self, advance: bool) -> Result<Option<(SourceSpan, usize, Token)>> {
        loop {
            let (span, size, token) = 'next_token_loop: loop {
                while self.cur_col == self.cur_line.len() {
                    if let Some(line) = self.lines.next() {
                        let ret = (self.cur_span(0..0), 0, Token::Newline);
                        self.cur_line = line;
                        self.cur_col = 0;
                        self.cur_ln += 1;
                        if self.cur_ln != 1 {
                            break 'next_token_loop ret;
                        }
                    } else if !self.eof {
                        self.eof = true;
                        break 'next_token_loop (self.cur_span(0..0), 0, Token::Newline);
                    } else {
                        return Ok(None);
                    }
                }

                break match self.state.parse_token(&self.cur_line[self.cur_col..]) {
                    Ok(Some((range, size, token))) => {
                        let span = self.cur_span(range);
                        if advance {
                            self.cur_col += size;
                        }

                        (span, size, token)
                    }
                    unk => todo!("{unk:?}"),
                };
            };

            match token {
                Token::CommentChunk { .. } => continue,
                Token::Newline => {
                    if self.last_was_newline {
                        continue;
                    }
                    self.last_was_newline = true;
                }
                _ => self.last_was_newline = false,
            }

            return Ok(Some((span, size, token)));
        }
    }

    pub fn req_token(
        &mut self,
        while_parsing: &'static str,
        want_token: Token<'static>,
    ) -> Result<SourceSpan> {
        if let Some((span, got_token)) = self.next_token()? {
            if want_token == got_token {
                Ok(span)
            } else {
                Err(ParseError::UnexpectedToken {
                    span,
                    while_parsing,
                    wanted: Wanted::Token(want_token),
                    got_token: got_token.token_type_name(),
                })
            }
        } else {
            todo!("{want_token}")
        }
    }

    pub fn read_key_value<T: LogixType>(
        &mut self,
        while_parsing: &'static str,
        end_brace: Brace,
    ) -> Result<Option<(Value<SmolStr>, Value<T>)>> {
        match self.next_token()? {
            Some((span, Token::Ident(key))) => {
                let key = Value {
                    value: SmolStr::new(key),
                    span,
                };

                self.req_token(while_parsing, Token::Colon)?;

                let value = T::logix_parse(self)?;

                self.req_token(while_parsing, Token::Newline)?;

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

    fn s(file: &CachedFile, ln: usize, start: usize, len: usize) -> SourceSpan {
        SourceSpan::new(&file, ln, start, len)
    }

    #[test]
    fn basics() -> Result<()> {
        let root = tempfile::tempdir().unwrap();
        std::fs::write(root.path().join("test.logix"), b"Hello { world: \"!!!\" }").unwrap();

        let mut loader = LogixLoader::new(RelFs::new(root.path()));
        let f = loader.open_file("test.logix")?;
        let f = &f;
        let mut p = LogixParser::new(&mut loader, f);

        assert_eq!(
            p.next_token()?,
            Some((s(f, 1, 0, 5), Token::Ident("Hello")))
        );
        assert_eq!(
            p.next_token()?,
            Some((s(f, 1, 6, 1), Token::BraceStart(Brace::Curly)))
        );
        assert_eq!(
            p.next_token()?,
            Some((s(f, 1, 8, 5), Token::Ident("world")))
        );
        assert_eq!(p.next_token()?, Some((s(f, 1, 13, 1), Token::Colon)));
        assert_eq!(
            p.next_token()?,
            Some((
                s(f, 1, 16, 3),
                Token::LitStrChunk {
                    chunk: "!!!",
                    last: true
                }
            ))
        );
        assert_eq!(
            p.next_token()?,
            Some((s(f, 1, 21, 1), Token::BraceEnd(Brace::Curly)))
        );

        assert_eq!(p.next_token()?, Some((s(f, 1, 22, 0), Token::Newline)));
        assert_eq!(p.next_token()?, None);
        Ok(())
    }
}
