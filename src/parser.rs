use std::ops::Range;

use crate::{
    error::{ParseError, Result, SourceSpan, Wanted, Warn},
    loader::{CachedFile, LogixLoader},
    token::{parse_token, Brace, Delim, ParseRes, Token},
    type_trait::Value,
    LogixType,
};
use logix_vfs::LogixVfs;
use smol_str::SmolStr;

pub struct LogixParser<'fs, 'f, FS: LogixVfs> {
    _loader: &'fs mut LogixLoader<FS>,
    file: &'f CachedFile,
    cur_pos: usize,
    cur_col: usize,
    cur_ln: usize,
    last_was_newline: bool,
    eof: bool,
}

impl<'fs, 'f, FS: LogixVfs> LogixParser<'fs, 'f, FS> {
    pub(crate) fn new(loader: &'fs mut LogixLoader<FS>, file: &'f CachedFile) -> Self {
        Self {
            _loader: loader,
            file,
            cur_pos: 0,
            cur_col: 0,
            cur_ln: 1,
            last_was_newline: false,
            eof: false,
        }
    }

    pub fn next_token(&mut self) -> Result<(SourceSpan, Token)> {
        self.raw_next_token().map(|(span, _, token)| (span, token))
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

    fn raw_next_token(&mut self) -> Result<(SourceSpan, usize, Token)> {
        if self.eof {
            return Ok((self.cur_span(0..0), 0, Token::Eof));
        }

        'outer: loop {
            let last_was_newline = std::mem::take(&mut self.last_was_newline);

            if last_was_newline {
                self.cur_ln += 1;
                self.cur_col = 0;
            }

            'ignore_token: loop {
                let buf = &self.file.data()[self.cur_pos..];

                return match dbg!(parse_token(buf)) {
                    ParseRes {
                        len,
                        range,
                        lines: 0,
                        token:
                            Ok(
                                token @ (Token::Ident(..)
                                | Token::Brace { .. }
                                | Token::Delim(..)
                                | Token::Literal(..)),
                            ),
                    } => {
                        let span = self.cur_span(range);
                        self.cur_col += len;
                        self.cur_pos += len;
                        Ok((span, len, token))
                    }
                    ParseRes {
                        len,
                        range,
                        lines: 0,
                        token: Ok(Token::Newline),
                    } => {
                        let span = self.cur_span(range);
                        self.cur_col += len;
                        self.cur_pos += len;
                        self.last_was_newline = true;
                        if last_was_newline {
                            continue 'outer;
                        }
                        Ok((span, len, Token::Newline))
                    }
                    ParseRes {
                        len,
                        range: _,
                        lines: 0,
                        token: Ok(Token::Comment(_)),
                    } => {
                        self.cur_col += len;
                        self.cur_pos += len;
                        continue 'ignore_token;
                    }
                    ParseRes {
                        len,
                        range,
                        lines: 0,
                        token: Ok(Token::Eof),
                    } => {
                        let token = if self.file.data().ends_with(b"\n") {
                            Token::Eof
                        } else {
                            Token::Newline
                        };
                        self.eof = true;
                        let span = self.cur_span(range);
                        self.cur_col += len;
                        self.cur_pos += len;
                        Ok((span, len, token))
                    }
                    unk => todo!("{unk:#?}"),
                };
            }
        }
    }

    pub fn req_token(
        &mut self,
        while_parsing: &'static str,
        want_token: Token<'static>,
    ) -> Result<SourceSpan> {
        let (span, got_token) = self.next_token()?;

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
    }

    pub fn read_key_value<T: LogixType>(
        &mut self,
        while_parsing: &'static str,
        end_brace: Brace,
    ) -> Result<Option<(Value<SmolStr>, Value<T>)>> {
        match self.next_token()? {
            (span, Token::Ident(key)) => {
                let key = Value {
                    value: SmolStr::new(key),
                    span,
                };

                self.req_token(while_parsing, Token::Delim(Delim::Colon))?;

                let value = T::logix_parse(self)?;

                self.req_token(while_parsing, Token::Newline)?;

                Ok(Some((key, value)))
            }
            (
                _,
                Token::Brace {
                    start: false,
                    brace,
                },
            ) if brace == end_brace => Ok(None),
            unk => todo!("{unk:#?}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use logix_vfs::RelFs;

    use crate::token::{Brace, Literal, StrTag};

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

        assert_eq!(p.next_token()?, (s(f, 1, 0, 5), Token::Ident("Hello")));
        assert_eq!(
            p.next_token()?,
            (
                s(f, 1, 6, 1),
                Token::Brace {
                    start: true,
                    brace: Brace::Curly
                }
            )
        );
        assert_eq!(p.next_token()?, (s(f, 1, 8, 5), Token::Ident("world")));
        assert_eq!(
            p.next_token()?,
            (s(f, 1, 13, 1), Token::Delim(Delim::Colon))
        );
        assert_eq!(
            p.next_token()?,
            (
                s(f, 1, 15, 5),
                Token::Literal(Literal::Str(StrTag::Raw, "!!!"))
            )
        );
        assert_eq!(
            p.next_token()?,
            (
                s(f, 1, 21, 1),
                Token::Brace {
                    start: false,
                    brace: Brace::Curly
                }
            )
        );

        assert_eq!(p.next_token()?, (s(f, 1, 22, 0), Token::Newline));
        assert_eq!(p.next_token()?, (s(f, 1, 22, 0), Token::Eof));
        Ok(())
    }
}
