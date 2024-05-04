#![warn(warnings)]
#![allow(clippy::collapsible_else_if)] // Why does clippy insist on making code less readable?
use std::marker::PhantomData;

use logix_vfs::LogixVfs;

use crate::{
    error::{ParseError, Result, Wanted},
    token::{Brace, Delim, Token},
    type_trait::Value,
    LogixParser, LogixType,
};

enum State {
    Init,
    ValueParsed,
    GotDelim { newline: bool, comma: bool },
}

pub struct ParseDelimited<'p, 'fs, 'f, FS: LogixVfs, T: LogixType> {
    p: &'p mut LogixParser<'fs, 'f, FS>,
    while_parsing: &'static str,
    _phantom: PhantomData<T>,
    state: State,
}

impl<'p, 'fs, 'f, FS: LogixVfs, T: LogixType> ParseDelimited<'p, 'fs, 'f, FS, T> {
    pub(super) fn new(p: &'p mut LogixParser<'fs, 'f, FS>, while_parsing: &'static str) -> Self {
        Self {
            p,
            while_parsing,
            _phantom: PhantomData,
            state: State::Init,
        }
    }

    pub fn skip_until_next(&mut self) -> Result<bool> {
        loop {
            let mut parse_value = false;
            let return_here = self
                .p
                .forked(|p| -> Result<Option<()>> {
                    match p.next_token()? {
                        (_, Token::Brace { start: false, .. }) => match self.state {
                            State::Init | State::ValueParsed | State::GotDelim { .. } => Ok(None),
                        },
                        (span, token @ Token::Delim(Delim::Comma)) => match self.state {
                            State::Init | State::GotDelim { comma: true, .. } => {
                                Err(ParseError::UnexpectedToken {
                                    span,
                                    while_parsing: self.while_parsing,
                                    got_token: token.token_type_name(),
                                    wanted: Wanted::ItemOrEnd,
                                })
                            }
                            State::ValueParsed => {
                                self.state = State::GotDelim {
                                    newline: false,
                                    comma: true,
                                };
                                Ok(Some(()))
                            }
                            State::GotDelim {
                                newline,
                                comma: false,
                            } => {
                                self.state = State::GotDelim {
                                    newline,
                                    comma: true,
                                };
                                Ok(Some(()))
                            }
                        },
                        (_, Token::Newline(false)) => match self.state {
                            State::Init => Ok(Some(())),
                            State::ValueParsed => {
                                self.state = State::GotDelim {
                                    newline: true,
                                    comma: false,
                                };
                                Ok(Some(()))
                            }
                            State::GotDelim { newline: _, comma } => {
                                self.state = State::GotDelim {
                                    newline: true,
                                    comma,
                                };
                                Ok(Some(()))
                            }
                        },
                        (span, token @ (Token::Newline(true) | Token::Delim(..))) => {
                            Err(ParseError::UnexpectedToken {
                                span,
                                while_parsing: self.while_parsing,
                                got_token: token.token_type_name(),
                                wanted: Wanted::ItemOrEnd,
                            })
                        }
                        (_, Token::Comment(..)) => Ok(Some(())),
                        (
                            span,
                            token @ (Token::Ident(..)
                            | Token::Action(..)
                            | Token::Literal(..)
                            | Token::Brace { start: true, .. }),
                        ) => match self.state {
                            State::Init | State::GotDelim { .. } => {
                                parse_value = true;
                                Ok(None)
                            }
                            State::ValueParsed => Err(ParseError::UnexpectedToken {
                                span,
                                while_parsing: self.while_parsing,
                                got_token: token.token_type_name(),
                                wanted: Wanted::ItemDelim,
                            }),
                        },
                    }
                })?
                .is_none();

            if return_here {
                return Ok(parse_value);
            }
        }
    }

    pub fn next_item(&mut self) -> Result<Option<Value<T>>> {
        if self.skip_until_next()? {
            let value = T::logix_parse(self.p)?;
            self.state = State::ValueParsed;
            Ok(Some(value))
        } else {
            Ok(None)
        }
    }

    pub fn req_next_item(&mut self) -> Result<Value<T>> {
        if let Some(ret) = self.next_item()? {
            Ok(ret)
        } else {
            let (span, token) = self.p.peek_token().unwrap();
            Err(ParseError::UnexpectedToken {
                span,
                while_parsing: self.while_parsing,
                got_token: token.token_type_name(),
                wanted: Wanted::Item,
            })
        }
    }

    pub fn req_end(&mut self, end_brace: Brace) -> Result<()> {
        let parse_value = self.skip_until_next()?;
        let (span, token) = self.p.peek_token().unwrap();

        if !parse_value
            && token
                == (Token::Brace {
                    start: false,
                    brace: end_brace,
                })
        {
            Ok(())
        } else {
            Err(ParseError::UnexpectedToken {
                span,
                while_parsing: self.while_parsing,
                got_token: token.token_type_name(),
                wanted: Wanted::Token(Token::Brace {
                    start: false,
                    brace: Brace::Square,
                }),
            })
        }
    }
}

impl<'p, 'fs, 'f, FS: LogixVfs, T: LogixType> Iterator for ParseDelimited<'p, 'fs, 'f, FS, T> {
    type Item = Result<Value<T>>;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_item().transpose()
    }
}

#[cfg(test)]
mod tests {
    use std::fmt;

    use crate::{parser::tests::run_test, token::Brace};

    use super::*;

    fn test_vec<T: LogixType + fmt::Debug>(src: &str) -> Result<Vec<T>> {
        run_test(src, |p, _| {
            p.req_wrapped("list1", Brace::Square, |p| -> Result<_> {
                p.parse_delimited("list2")
                    .map(|r| r.map(|v| v.value))
                    .collect()
            })
        })
        .map(|r| r.value)
    }

    #[test]
    fn basics() -> Result<()> {
        assert_eq!(test_vec::<i32>("[]")?, vec![]);
        assert_eq!(test_vec::<i32>("[\n]")?, vec![]);
        assert_eq!(
            test_vec::<i32>("[,]").unwrap_err().to_string(),
            "Unexpected `,` while parsing `list2`, expected item or end in test.logix:1:1"
        );
        assert_eq!(
            test_vec::<i32>("[\n,]").unwrap_err().to_string(),
            "Unexpected `,` while parsing `list2`, expected item or end in test.logix:2:0"
        );
        assert_eq!(
            test_vec::<i32>("[,\n]").unwrap_err().to_string(),
            "Unexpected `,` while parsing `list2`, expected item or end in test.logix:1:1"
        );
        assert_eq!(
            test_vec::<i32>("[").unwrap_err().to_string(),
            "Unexpected end of file while parsing `list2`, expected item or end in test.logix:1:1"
        );
        assert_eq!(
            test_vec::<i32>("[\n").unwrap_err().to_string(),
            "Unexpected end of file while parsing `list2`, expected item or end in test.logix:1:1"
        );
        assert_eq!(
            test_vec::<i32>("[0").unwrap_err().to_string(),
            "Unexpected end of file while parsing `list2`, expected item or end in test.logix:1:2"
        );
        assert_eq!(
            test_vec::<i32>("[0\n").unwrap_err().to_string(),
            "Unexpected end of file while parsing `list2`, expected item or end in test.logix:1:2"
        );
        assert_eq!(
            test_vec::<i32>("[0,").unwrap_err().to_string(),
            "Unexpected end of file while parsing `list2`, expected item or end in test.logix:1:3"
        );
        assert_eq!(
            test_vec::<i32>("[0,\n").unwrap_err().to_string(),
            "Unexpected end of file while parsing `list2`, expected item or end in test.logix:1:3"
        );
        assert_eq!(
            test_vec::<i32>("[0 1]").unwrap_err().to_string(),
            "Unexpected number while parsing `list2`, expected delimiter in test.logix:1:3"
        );
        assert_eq!(test_vec::<i32>("[10]")?, vec![10]);
        assert_eq!(test_vec::<i32>("[10,]")?, vec![10]);
        assert_eq!(test_vec::<i32>("[10\n]")?, vec![10]);
        assert_eq!(test_vec::<i32>("[10\n,]")?, vec![10]);
        assert_eq!(test_vec::<i32>("[10,\n]")?, vec![10]);
        assert_eq!(test_vec::<i32>("[10,11]")?, vec![10, 11]);
        assert_eq!(test_vec::<i32>("[10\n11]")?, vec![10, 11]);
        assert_eq!(test_vec::<i32>("[10,\n11]")?, vec![10, 11]);
        assert_eq!(test_vec::<i32>("[10\n,11]")?, vec![10, 11]);
        assert_eq!(test_vec::<i32>("[10\n,11,]")?, vec![10, 11]);
        assert_eq!(test_vec::<i32>("[10\n,11\n,]")?, vec![10, 11]);
        assert_eq!(test_vec::<i32>("[10\n,11\n,\n]")?, vec![10, 11]);
        Ok(())
    }
}
