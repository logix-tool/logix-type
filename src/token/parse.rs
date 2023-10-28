use std::{ops::Range, str::from_utf8};

const IDENT1: ByteSet = ByteSet(concat!(
    "abcdefghijklmnopqrstuvwxyz",
    "ABCDEFGHIJKLMNOPQRSTUVWXYZ",
    "0123456789",
    "_-",
));

use bstr::ByteSlice;

use super::{Brace, ByteSet, Delim, Literal, Token, TokenError};

#[derive(Debug, PartialEq)]
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
            .unwrap_or(buf.len() - start);
        let end = start + len;
        ParseRes::new(start..end, f(from_utf8(&buf[start..end]).unwrap()))
    }

    fn new_brace(pos: usize, start: bool, brace: Brace) -> Self {
        Self::new(pos..pos + 1, Token::Brace { start, brace })
    }
}

pub fn parse_token(buf: &[u8]) -> ParseRes {
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
        Some(b'/') => {
            if let Some(ret) = super::comment::parse_comment(buf, start) {
                ret
            } else {
                ParseRes::new_res(start..start + 1, 0, Err(TokenError::UnexpectedChar('/')))
            }
        }
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
                .unwrap_or(buf.len() - start);
            ParseRes::new_lines(
                buf,
                start..start,
                off,
                Ok(Token::Newline(buf.len() == start + off)),
            )
        }
        Some(b'"') => super::string::parse_basic(buf, start),
        Some(b'#') => {
            if let Some(ret) = super::string::parse_tagged(buf, start) {
                ret
            } else {
                ParseRes::new_res(start..start + 1, 0, Err(TokenError::UnexpectedChar('#')))
            }
        }
        _ => {
            if let Some((_, off, chr)) = buf[start..].char_indices().next() {
                ParseRes::new_res(start..start + off, 0, Err(TokenError::UnexpectedChar(chr)))
            } else {
                ParseRes::new(buf.len()..buf.len(), Token::Newline(true))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basics() {
        assert_eq!(
            parse_token(b"{"),
            ParseRes {
                len: 1,
                range: 0..1,
                lines: 0,
                token: Ok(Token::Brace {
                    start: true,
                    brace: Brace::Curly
                }),
            }
        );
    }
}
