use std::{fmt, ops::Range, str::from_utf8};

use bstr::ByteSlice;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum StrTag {
    /// The string can be used as is, no pre-processing needed
    Raw,
    /// The string contain basic backslash escaped data
    Esc,
    /// The string is in nicely wrapped multi-line format
    /// * No escapes
    /// * Remove common leading whitespace
    /// * Trim the end of the string
    /// * Remove all single newlines and replace them by space (paragraph)
    Txt,
}

impl StrTag {
    fn from_prefix(buf: &[u8]) -> Option<(usize, Self)> {
        if buf.starts_with(b"txt\"") {
            Some((4, Self::Txt))
        } else {
            None
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Brace {
    /// Curly braces `{}`
    Curly,
    /// Parenthesis `()`
    Paren,
    /*
    /// Square brackets `[]`
    Square,
    /// AAngle brackets `<>`
    Angle,
    */
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Delim {
    Colon,
    Comma,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Literal<'a> {
    Str(StrTag, &'a str),
    Num(&'a str),
}

#[derive(Debug, PartialEq, Eq)]
pub enum Token<'a> {
    Ident(&'a str),
    Literal(Literal<'a>),
    TaggedStrChunk {
        tag: StrTag,
        chunk: &'a str,
        last: bool,
    },
    Brace {
        start: bool,
        brace: Brace,
    },
    Delim(Delim),
    Comment(&'a str),
    Newline,
    Eof,
}

impl<'a> Token<'a> {
    pub fn token_type_name(&self) -> &'static str {
        match self {
            Self::Ident(_) => "identifier",
            Self::Literal(Literal::Str(..)) => "string",
            Self::Brace {
                start: true,
                brace: Brace::Paren,
            } => "`)`",
            Self::Brace {
                start: false,
                brace: Brace::Curly,
            } => "`}`",
            Self::Delim(Delim::Comma) => "`,`",
            Self::Newline => "newline",
            //Self::CommentChunk { .. } => "comment",
            Self::Eof => "end of file",
            unk => todo!("{unk:?}"),
        }
    }
}

impl<'a> fmt::Display for Token<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Ident(value) => write!(f, "`{value}`"),
            Self::Literal(..) => todo!(),
            Self::TaggedStrChunk { .. } => todo!(),
            Self::Comment(..) => todo!(),
            Self::Brace { .. } | Self::Delim(..) | Self::Newline | Self::Eof => {
                write!(f, "{}", self.token_type_name())
            }
        }
    }
}

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
    fn new(range: Range<usize>, token: Token<'a>) -> Self {
        Self::new_res(range, 0, Ok(token))
    }

    fn new_res(
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
}

pub fn parse_token<'a>(buf: &'a [u8]) -> ParseRes<'a> {
    let mut it = if let Some(pos) = buf.find_not_byteset(b" \t") {
        buf.char_indices().skip(pos)
    } else {
        buf.char_indices().skip(0)
    };

    match it.next() {
        Some((start, _, 'a'..='z' | 'A'..='Z' | '_')) => {
            if let Some((end, _, _)) =
                it.find(|(_, _, c)| !matches!(c, 'a'..='z' | 'A'..='Z' | '0'..='9' | '_' | '-'))
            {
                ParseRes::new(
                    start..end,
                    Token::Ident(from_utf8(&buf[start..end]).unwrap()),
                )
            } else {
                todo!()
            }
        }
        Some((start, _, '-' | '0'..='9')) => {
            let cur = &buf[start..];
            match cur.find_not_byteset(b"0123456789-._").unwrap_or(buf.len()) {
                0 => todo!(),
                pos => ParseRes::new(
                    start..start + pos,
                    Token::Literal(Literal::Num(from_utf8(&cur[..pos]).unwrap())),
                ),
            }
        }
        Some((start, _, '/')) => {
            if let Some(cur) = buf[start..].strip_prefix(b"//") {
                let comment = cur.lines().next().unwrap();
                ParseRes::new(
                    start..start + comment.len() + 2,
                    Token::Comment(from_utf8(comment.trim()).unwrap()),
                )
            } else if let Some(comment) = buf[start..].strip_prefix(b"/*") {
                let mut cur = &buf[start + 2..];
                let mut level = 0;

                while let Some(off) = cur.find_byteset(b"/*") {
                    match cur.get(off..off + 2) {
                        Some(b"*/") => if level == 0 {
                            todo!("level += 1"),
                        } else {
                            level -= 1;
                            todo!()
                        },
                        Some(b"*/") => {
                            level += 1;
                            todo!()
                        }
                        Some(b"/*") => todo!("level += 1"),
                        unk => todo!("{unk:?}"),
                    }
                }

                todo!("{comment:?}")
            } else {
                todo!()
            }
        }
        Some((start, end, '{')) => ParseRes::new(
            start..end,
            Token::Brace {
                start: true,
                brace: Brace::Curly,
            },
        ),
        Some((start, end, '}')) => ParseRes::new(
            start..end,
            Token::Brace {
                start: false,
                brace: Brace::Curly,
            },
        ),
        Some((start, end, '(')) => ParseRes::new(
            start..end,
            Token::Brace {
                start: true,
                brace: Brace::Paren,
            },
        ),
        Some((start, end, ')')) => ParseRes::new(
            start..end,
            Token::Brace {
                start: false,
                brace: Brace::Paren,
            },
        ),
        Some((start, end, ':')) => ParseRes::new(start..end, Token::Delim(Delim::Colon)),
        Some((start, end, ',')) => ParseRes::new(start..end, Token::Delim(Delim::Comma)),
        Some((start, end, '\n')) => ParseRes::new(start..end, Token::Newline),
        Some((start, _, '"')) => {
            let mut cur = &buf[start + 1..];
            let mut tag = StrTag::Raw;

            while let Some(pos) = cur.find_byteset(b"\\\"\\n") {
                match cur[pos] {
                    b'"' => {
                        let token = from_utf8(&cur[..pos])
                            .map(|value| Token::Literal(Literal::Str(tag, value)))
                            .map_err(|_| TokenError::LitStrNotUtf8);
                        return ParseRes::new_res(start..start + pos + 2, 0, token);
                    }
                    b'\\' => {
                        tag = StrTag::Esc;
                        match cur.get(pos + 1) {
                            Some(b't' | b'n' | b'r') => {}
                            Some(b'u') => todo!("unicode"),
                            Some(unk) => todo!("{unk:?}"),
                            None => todo!("unexpected end of file"),
                        }
                    }
                    b'\n' => todo!("Unexpected end of string"),
                    unk => unreachable!("{unk:?} is not in byteset"),
                }
                cur = &cur[pos..];
            }

            todo!("Unexpected end of file?")
        }
        Some((start, _, '#')) => {
            let cur = buf[start..].trim_start_with(|c| matches!(c, '#'));
            let prefix = &buf[start..buf.len() - cur.len()];
            todo!("{cur:?}, {prefix:?}")
        }
        Some(unk) => todo!("{unk:?}"),
        None => ParseRes::new(buf.len()..buf.len(), Token::Eof),
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

#[cfg(test)]
mod tests {
    use super::*;

    const REPLACEMENT_CHAR_BYTES: &[u8] = "\u{fffd}".as_bytes();

    #[test]
    fn bstr_assumptions() {
        assert_eq!(REPLACEMENT_CHAR_BYTES, b"\xef\xbf\xbd");

        assert_eq!(
            b"abc\x80\xef\xbf\xbd\xf0\x9f\x98"
                .as_bytes()
                .char_indices()
                .collect::<Vec<_>>(),
            vec![
                (0, 1, 'a'),
                (1, 2, 'b'),
                (2, 3, 'c'),
                (3, 4, char::REPLACEMENT_CHARACTER),
                (4, 7, char::REPLACEMENT_CHARACTER),
                (7, 10, char::REPLACEMENT_CHARACTER),
            ]
        );

        assert_eq!(
            REPLACEMENT_CHAR_BYTES.chars().collect::<Vec<_>>(),
            vec!['\u{fffd}']
        );
    }
}
