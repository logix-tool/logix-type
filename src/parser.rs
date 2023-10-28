use std::ops::Range;

use crate::{
    error::{ParseError, Result, Wanted, Warn},
    loader::{CachedFile, LogixLoader},
    span::SourceSpan,
    token::{parse_token, Brace, Delim, ParseRes, Token},
    type_trait::Value,
    LogixType,
};
use bstr::ByteSlice;
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
            last_was_newline: true,
            eof: false,
        }
    }

    pub fn warning(&self, warning: Warn) -> Result<()> {
        // TODO(2023.10): Make it possible to allow warnings
        Err(ParseError::Warning(warning))
    }

    fn cur_span(&self, range: Range<usize>) -> SourceSpan {
        SourceSpan::new(
            self.file,
            self.cur_pos + range.start,
            self.cur_ln,
            self.cur_col + range.start,
            range.len(),
        )
    }

    pub fn next_token(&mut self) -> Result<(SourceSpan, Token)> {
        if self.eof {
            return Ok((self.cur_span(0..0), Token::Newline(true)));
        }

        'outer: loop {
            let last_was_newline = std::mem::take(&mut self.last_was_newline);

            'ignore_token: loop {
                let buf = &self.file.data()[self.cur_pos..];
                let (span, token) = {
                    let ParseRes {
                        len,
                        range,
                        lines,
                        token,
                    } = parse_token(buf);
                    let span = self.cur_span(range);

                    self.cur_pos += len;
                    if lines > 0 {
                        debug_assert_ne!(len, 0);

                        self.cur_ln += lines;
                        if len == 1 {
                            self.cur_col = 0;
                        } else {
                            self.cur_col = self.file.data()[..self.cur_pos]
                                .lines()
                                .next_back()
                                .unwrap()
                                .len();
                        }
                    } else {
                        self.cur_col += len;
                    }
                    (span, token)
                };

                return match token {
                    Ok(
                        token @ (Token::Ident(..)
                        | Token::Brace { .. }
                        | Token::Delim(..)
                        | Token::Literal(..)),
                    ) => Ok((span, token)),
                    Ok(Token::Newline(eof)) => {
                        self.last_was_newline = true;
                        if !eof && last_was_newline {
                            continue 'outer;
                        }
                        self.eof = eof;
                        Ok((span, Token::Newline(eof)))
                    }
                    Ok(Token::Comment(_)) => {
                        continue 'ignore_token;
                    }
                    Err(error) => Err(ParseError::TokenError { span, error }),
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

                self.req_newline(while_parsing)?;

                Ok(Some((key, value)))
            }
            (
                _,
                Token::Brace {
                    start: false,
                    brace,
                },
            ) if brace == end_brace => Ok(None),
            (span, got_token) => Err(ParseError::UnexpectedToken {
                span,
                while_parsing,
                wanted: Wanted::Ident,
                got_token: got_token.token_type_name(),
            }),
        }
    }

    pub fn req_newline(&mut self, while_parsing: &'static str) -> Result<()> {
        match self.next_token()? {
            (_, Token::Newline(..)) => Ok(()),
            (span, got_token) => Err(ParseError::UnexpectedToken {
                span,
                while_parsing,
                wanted: Wanted::Token(Token::Newline(false)),
                got_token: got_token.token_type_name(),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use logix_vfs::RelFs;

    use crate::token::{Brace, Literal, StrTag};

    use super::*;

    fn s(file: &CachedFile, pos: usize, ln: usize, start: usize, len: usize) -> SourceSpan {
        SourceSpan::new(&file, pos, ln, start, len)
    }

    #[test]
    fn basics() -> Result<()> {
        let root = tempfile::tempdir().unwrap();
        std::fs::write(root.path().join("test.logix"), b"Hello { world: \"!!!\" }").unwrap();

        let mut loader = LogixLoader::new(RelFs::new(root.path()));
        let f = loader.open_file("test.logix")?;
        let f = &f;
        let mut p = LogixParser::new(&mut loader, f);

        assert_eq!(p.next_token()?, (s(f, 0, 1, 0, 5), Token::Ident("Hello")));
        assert_eq!(
            p.next_token()?,
            (
                s(f, 6, 1, 6, 1),
                Token::Brace {
                    start: true,
                    brace: Brace::Curly
                }
            )
        );
        assert_eq!(p.next_token()?, (s(f, 8, 1, 8, 5), Token::Ident("world")));
        assert_eq!(
            p.next_token()?,
            (s(f, 13, 1, 13, 1), Token::Delim(Delim::Colon))
        );
        assert_eq!(
            p.next_token()?,
            (
                s(f, 15, 1, 15, 5),
                Token::Literal(Literal::Str(StrTag::Raw, "!!!"))
            )
        );
        assert_eq!(
            p.next_token()?,
            (
                s(f, 21, 1, 21, 1),
                Token::Brace {
                    start: false,
                    brace: Brace::Curly
                }
            )
        );

        assert_eq!(p.next_token()?, (s(f, 22, 1, 22, 0), Token::Newline(true)));

        assert_eq!(p.next_token()?, (s(f, 22, 1, 22, 0), Token::Newline(true))); // A second time to trigger additional code
        Ok(())
    }
}
