use std::str::from_utf8;

use bstr::ByteSlice;

use super::{ParseRes, Token, TokenError};

pub fn parse_comment<'a>(buf: &'a [u8], start: usize) -> Option<ParseRes<'a>> {
    if let Some(cur) = buf[start..].strip_prefix(b"//") {
        let comment = cur.lines().next().unwrap();
        Some(ParseRes::new(
            start..start + comment.len() + 2,
            Token::Comment(from_utf8(comment.trim()).unwrap()),
        ))
    } else if buf[start..].starts_with(b"/*") {
        let mut end = start + 2;
        let mut level = 0;

        while let Some(off) = buf[end..].find_byteset(b"/*") {
            end += off;

            match buf.get(end..end + 2) {
                Some(b"*/") => {
                    end += 2;
                    if level == 0 {
                        return Some(ParseRes::new_lines(
                            buf,
                            start..end,
                            0,
                            Ok(Token::Comment(
                                from_utf8(buf[start + 2..end - 2].trim()).unwrap(),
                            )),
                        ));
                    } else {
                        level -= 1;
                    }
                }
                Some(b"/*") => {
                    end += 2;
                    level += 1;
                }
                _ => end += 1,
            }
        }

        Some(ParseRes::new_res(
            buf.len()..buf.len() + 1,
            0,
            Err(TokenError::MissingCommentTerminator),
        ))
    } else {
        None
    }
}
