use std::str::from_utf8;

use bstr::ByteSlice;

use super::{Literal, ParseRes, StrTag, Token, TokenError};

pub fn parse_basic<'a>(buf: &'a [u8], start: usize) -> ParseRes<'a> {
    let mut pos = start + 1;
    let mut tag = StrTag::Raw;

    while let Some(off) = buf[pos..].find_byteset(b"\\\"\n") {
        pos += off;
        match buf[pos] {
            b'"' => {
                let token = from_utf8(&buf[start + 1..pos])
                    .map(|value| Token::Literal(Literal::Str(tag, value)))
                    .map_err(|_| TokenError::LitStrNotUtf8);
                return ParseRes::new_res(start..pos + 1, 0, token);
            }
            b'\\' => {
                tag = StrTag::Esc;
                match buf.get(pos + 1) {
                    Some(b't' | b'n' | b'r') => pos += 2,
                    Some(b'u') => todo!("unicode"),
                    Some(unk) => todo!("{unk:?}"),
                    None => todo!("unexpected end of file"),
                }
            }
            b'\n' => todo!("Unexpected end of string"),
            unk => unreachable!("{unk:?} ({:?}) is not in byteset", char::from(unk)),
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
        let token = from_utf8(value)
            .map(|v| Token::Literal(Literal::Str(tag, v)))
            .map_err(|_| TokenError::LitStrNotUtf8);
        ParseRes::new_lines(buf, start..pos + value.len() + suffix.len(), 0, token)
    } else {
        todo!("unexpected eof")
    }
}
