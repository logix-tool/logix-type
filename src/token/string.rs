use bstr::ByteSlice;

use super::{Literal, ParseRes, StrTag, Token, TokenError};

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
pub fn parse_basic<'a>(buf: &'a [u8], start: usize) -> ParseRes<'a> {
    let mut pos = start + 1;
    let mut tag = StrTag::Raw;

    while let Some(off) = buf[pos..].find_byteset(b"\\\"\n") {
        pos += off;
        match buf.get(pos).copied() {
            Some(b'"') => {
                return parse_utf8(start + 1, pos, &buf[start + 1..pos], |value| {
                    ParseRes::new(start..pos + 1, Token::Literal(Literal::Str(tag, value)))
                });
            }
            Some(b'\\') => {
                tag = StrTag::Esc;
                pos += 2;
            }
            Some(b'\n') => todo!("Unexpected end of string"),
            Some(unk) => unreachable!("{unk:?} ({:?}) is not in byteset", char::from(unk)),
            None => todo!("unexpected end of file"),
        }
    }

    todo!("Unexpected end of file?")
}

pub fn parse_tagged<'a>(buf: &'a [u8], start: usize) -> ParseRes<'a> {
    static HASHES: &str = "\"################################";
    let num_hashes = buf[start..].find_not_byteset(b"#").unwrap();
    let suffix = if num_hashes < HASHES.len() {
        HASHES[..num_hashes + 1].as_bytes()
    } else {
        todo!("Too many #")
    };
    let mut pos = start + num_hashes;

    let tag = if let Some((off, tag)) = StrTag::from_prefix(&buf[pos..]) {
        pos += off;
        tag
    } else {
        todo!("No valid tag")
    };

    if let Some((value, _)) = buf[pos..].split_once_str(suffix) {
        let end = pos + value.len() + suffix.len();
        return parse_utf8(start + pos, end, value, |v| {
            ParseRes::new_lines(buf, start..end, 0, Ok(Token::Literal(Literal::Str(tag, v))))
        });
    } else {
        todo!("unexpected eof")
    }
}
