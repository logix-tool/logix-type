use bstr::ByteSlice;

use crate::{string::StrLit, types::ShortStr};

use super::{Literal, ParseRes, StrTag, StrTagSuffix, Token, TokenError};

fn parse_utf8<'a>(
    start: usize,
    end: usize,
    value: &'a [u8],
    f: impl FnOnce(&'a str) -> ParseRes<'a>,
) -> ParseRes<'a> {
    std::str::from_utf8(value).map(f).unwrap_or_else(|e| {
        let new_start = start + e.valid_up_to();
        let extra = (start + end + 1) - new_start;
        ParseRes::new_res(
            new_start..new_start + 1,
            extra,
            Err(TokenError::LitStrNotUtf8),
        )
    })
}

pub fn parse_basic(buf: &[u8], start: usize) -> ParseRes {
    let mut pos = start + 1;
    let mut tag = StrTag::Raw;

    while let Some(off) = buf[pos..].find_byteset(b"\\\"\n") {
        pos += off;
        match buf[pos] {
            b'"' => {
                return parse_utf8(start + 1, pos, &buf[start + 1..pos], |value| {
                    ParseRes::new(
                        start..pos + 1,
                        Token::Literal(Literal::Str(StrLit::new(tag, value))),
                    )
                });
            }
            b'\\' => {
                tag = StrTag::Esc;
                pos += 2;
            }
            unk => {
                assert_eq!(unk, b'\n');
                return ParseRes::new_res(
                    pos..pos + 1,
                    0,
                    Err(TokenError::MissingStringTerminator),
                );
            }
        }
    }

    ParseRes::new_res(
        buf.len()..buf.len(),
        0,
        Err(TokenError::MissingStringTerminator),
    )
}

pub fn parse_tagged(buf: &[u8], start: usize) -> Option<ParseRes> {
    let num_hashes = buf[start..].find_not_byteset(b"#").unwrap();
    let suffix = StrTagSuffix::new(num_hashes);

    let mut pos = start + num_hashes;

    let tag = if let Some((off, tag)) = StrTag::from_prefix(&buf[pos..]) {
        pos += off;
        tag
    } else {
        let end = buf[pos..].find_not_byteset(StrTag::VALID.0)? + pos;
        let tag = ShortStr::from(std::str::from_utf8(&buf[pos..end]).unwrap());
        match buf[end] {
            b'"' => {
                return Some(ParseRes::new_res(
                    pos - 1..end,
                    0,
                    Err(TokenError::UnknownStrTag(tag)),
                ));
            }
            // NOTE(2023.10): It is not a string, return None to let the caller figure out what to do
            _ => return None,
        }
    };

    if let Some((value, _)) = buf[pos..].split_once_str(&suffix) {
        let end = pos + value.len() + suffix.len();
        return Some(parse_utf8(start + pos, end, value, |v| {
            ParseRes::new_lines(
                buf,
                start..end,
                0,
                Ok(Token::Literal(Literal::Str(StrLit::new(tag, v)))),
            )
        }));
    }

    Some(ParseRes::new_res(
        buf.len()..buf.len(),
        0,
        Err(TokenError::MissingTaggedStringTerminator { tag, suffix }),
    ))
}
