use std::{ops::Range, str::from_utf8};

const IDENT1: ByteSet = ByteSet(concat!(
    "abcdefghijklmnopqrstuvwxyz",
    "ABCDEFGHIJKLMNOPQRSTUVWXYZ",
    "0123456789",
    "_-",
));

pub(super) struct ByteSet(&'static str);

use bstr::ByteSlice;

use super::{Brace, Delim, Literal, Token};

#[derive(Debug, thiserror::Error)]
pub enum TokenError {
    #[error("String literal is not valid utf-8")]
    LitStrNotUtf8,
}

#[derive(Debug)]
pub struct ParseRes<'a> {
    /// How much to skip before the next parse_token call
    pub len: usize,
    /// The range that contains the token
    pub range: Range<usize>,
    /// Number of lines skipped, normally 0, but set for multi-line comments and strings
    pub lines: usize,
    /// The current token
    pub token: Result<Token<'a>, TokenError>,
}

impl<'a> ParseRes<'a> {
    pub(super) fn new(range: Range<usize>, token: Token<'a>) -> Self {
        Self::new_res(range, 0, Ok(token))
    }

    pub(super) fn new_res(
        range: Range<usize>,
        extra: usize,
        token: Result<Token<'a>, TokenError>,
    ) -> ParseRes<'a> {
        Self {
            len: range.end + extra,
            range,
            lines: 0,
            token,
        }
    }

    pub(super) fn new_lines(
        buf: &[u8],
        range: Range<usize>,
        extra: usize,
        token: Result<Token<'a>, TokenError>,
    ) -> Self {
        let lines = buf[range.start..range.end + extra].find_iter(b"\n").count();
        Self {
            len: range.end + extra,
            range,
            lines,
            token,
        }
    }

    pub(super) fn take_byteset(
        buf: &'a [u8],
        start: usize,
        byteset: ByteSet,
        f: impl FnOnce(&'a str) -> Token,
    ) -> Self {
        let len = buf[start..]
            .find_not_byteset(byteset.0)
            .unwrap_or_else(|| buf.len() - start);
        let end = start + len;
        ParseRes::new(start..end, f(from_utf8(&buf[start..end]).unwrap()))
    }

    fn new_brace(pos: usize, start: bool, brace: Brace) -> Self {
        Self::new(pos..pos + 1, Token::Brace { start, brace })
    }
}

pub fn parse_token<'a>(buf: &'a [u8]) -> ParseRes<'a> {
    let start = buf.find_not_byteset(b" \t").unwrap_or(0);

    match buf.get(start) {
        Some(b'a'..=b'z' | b'A'..=b'Z' | b'_') => {
            ParseRes::take_byteset(buf, start, IDENT1, Token::Ident)
        }
        Some(b'-' | b'0'..=b'9') => {
            ParseRes::take_byteset(buf, start, ByteSet("0123456789-._"), |s| {
                Token::Literal(Literal::Num(s))
            })
        }
        Some(b'/') => super::comment::parse_comment(buf, start),
        Some(b'{') => ParseRes::new_brace(start, true, Brace::Curly),
        Some(b'}') => ParseRes::new_brace(start, false, Brace::Curly),
        Some(b'(') => ParseRes::new_brace(start, true, Brace::Paren),
        Some(b')') => ParseRes::new_brace(start, false, Brace::Paren),
        Some(b'[') => ParseRes::new_brace(start, true, Brace::Square),
        Some(b']') => ParseRes::new_brace(start, false, Brace::Square),
        Some(b'<') => ParseRes::new_brace(start, true, Brace::Angle),
        Some(b'>') => ParseRes::new_brace(start, false, Brace::Angle),
        Some(b':') => ParseRes::new(start..start + 1, Token::Delim(Delim::Colon)),
        Some(b',') => ParseRes::new(start..start + 1, Token::Delim(Delim::Comma)),
        Some(b'\n') => {
            let off = buf[start..]
                .find_not_byteset("\r\n \t")
                .unwrap_or_else(|| buf.len() - start);
            ParseRes::new_lines(
                buf,
                start..start,
                off,
                Ok(Token::Newline(buf.len() == start + off)),
            )
        }
        Some(b'"') => super::string::parse_basic(buf, start),
        Some(b'#') => super::string::parse_tagged(buf, start),
        Some(unk) => todo!("{unk:?}"),
        None => ParseRes::new(buf.len()..buf.len(), Token::Newline(true)),
    }
    /*
    Self::Comment(level) => {
        let start = buf.len() - it.as_bytes().len();
        let mut cur = start;

        while let Some(off) = buf[cur..].find_byteset(b"/*") {
            match buf.get(cur + off..cur + off + 2) {
                Some(b"/*") => {
                    *level += 1;
                    cur += off + 2;
                }
                Some(b"*/
    ") if *level == 0 => {
                            cur += off;
                            *self = Self::Normal;
                            return Ok((
                                start..cur,
                                cur + 2,
                                Token::CommentChunk {
                                    chunk: from_utf8(&buf[start..cur]).unwrap(),
                                    last: true,
                                },
                            ));
                        }
                        Some(b"*/") => {
                            *level -= 1;
                            cur += off + 2;
                        }
                        _ => {
                            cur += off + 1;
                        }
                    }
                }

                // Need to look at the next line
                return Ok((
                    start..buf.len(),
                    buf.len(), // TODO(2023.10): Not sure this is correct
                    Token::CommentChunk {
                        chunk: from_utf8(it.as_bytes()).unwrap(),
                        last: false,
                    },
                ));
            }
            &mut Self::TaggedStr(ref suff, tag) => {
                let start = buf.len() - it.as_bytes().len();

                if let Some(end) = it.as_bytes().find(suff.as_bytes()) {
                    let suff_len = suff.len();
                    *self = Self::Normal;
                    return Ok((
                        start..start + end,
                        start + end + suff_len,
                        Token::TaggedStrChunk {
                            tag,
                            chunk: from_utf8(&buf[start..end]).unwrap(),
                            last: true,
                        },
                    ));
                } else {
                    return Ok((
                        start..buf.len(),
                        buf.len(),
                        Token::TaggedStrChunk {
                            tag,
                            chunk: from_utf8(&buf[start..]).unwrap(),
                            last: false,
                        },
                    ));
                }
            }
*/
}
