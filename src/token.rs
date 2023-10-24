use std::{fmt, ops::Range};

use bstr::ByteSlice;

#[derive(Debug, PartialEq, Eq)]
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

#[derive(Debug, PartialEq, Eq)]
pub enum Token<'a> {
    Ident(&'a str),
    LitStrChunk { chunk: &'a str, last: bool },
    LitNumber(&'a str),
    BraceStart(Brace),
    BraceEnd(Brace),
    Colon,
    Comma,
    Newline,
    CommentChunk { chunk: &'a str, last: bool },
    Eof,
}
impl<'a> Token<'a> {
    pub fn token_type_name(&self) -> &'static str {
        match self {
            Self::Ident(_) => "identifier",
            //Self::LitStrChunk { .. } => "string literal",
            Self::BraceEnd(Brace::Paren) => "`)`",
            Self::BraceEnd(Brace::Curly) => "`}`",
            Self::Comma => "`,`",
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
            Self::LitStrChunk { .. } => todo!(),
            Self::LitNumber(..) => todo!(),
            Self::CommentChunk { .. } => todo!(),
            Self::BraceEnd(..)
            | Self::BraceStart(..)
            | Self::Colon
            | Self::Comma
            | Self::Newline
            | Self::Eof => write!(f, "{}", self.token_type_name()),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum TokenError {
    #[error("String literal is not valid utf-8")]
    LitStrNotUtf8,
}

pub(crate) enum TokenState {
    Normal,
    LitStr,
    Comment(usize),
}

impl TokenState {
    pub fn parse_token<'a>(
        &mut self,
        buf: &'a [u8],
    ) -> std::result::Result<(Range<usize>, usize, Token<'a>), TokenError> {
        let mut it = buf.char_indices();

        loop {
            match self {
                Self::Normal => {
                    return Ok(match it.next() {
                        Some((start, _, 'a'..='z' | 'A'..='Z' | '_')) => {
                            if let Some((end, _, _)) = it.find(
                                |(_, _, c)| !matches!(c, 'a'..='z' | 'A'..='Z' | '0'..='9' | '_' | '-'),
                            ) {
                                (
                                    start..end,
                                    end,
                                    Token::Ident(std::str::from_utf8(&buf[start..end]).unwrap()),
                                )
                            } else {
                                todo!()
                            }
                        }
                        Some((start, _, '-' | '0'..='9')) => {
                            let cur = &buf[start..];
                            match cur.find_not_byteset(b"0123456789-._").unwrap_or(buf.len()) {
                                0 => todo!(),
                                pos => {
                                    let end = start + pos;
                                    (
                                        start..end,
                                        end,
                                        Token::LitNumber(std::str::from_utf8(&cur[..pos]).unwrap()),
                                    )
                                }
                            }
                        }
                        Some((start, end, '/')) => {
                            if let Some(comment) = it.as_bytes().strip_prefix(b"/") {
                                let end = end + comment.len() + 1;
                                (start..end, end, Token::CommentChunk { chunk: std::str::from_utf8(comment).unwrap(), last: true })
                            } else if it.as_bytes().starts_with(b"*") {
                                it.next();
                                *self = Self::Comment(0);
                                continue;
                            } else {
                                todo!()
                            }
                        }
                        Some((start, end, '{')) => (start..end, skip_whitespace(end, it), Token::BraceStart(Brace::Curly)),
                        Some((start, end, '}')) => (start..end, skip_whitespace(end, it), Token::BraceEnd(Brace::Curly)),
                        Some((start, end, '(')) => (start..end, skip_whitespace(end, it), Token::BraceStart(Brace::Paren)),
                        Some((start, end, ')')) => (start..end, skip_whitespace(end, it), Token::BraceEnd(Brace::Paren)),
                        Some((start, end, ':')) => (start..end, skip_whitespace(end, it), Token::Colon),
                        Some((start, end, ',')) => (start..end, skip_whitespace(end, it), Token::Comma),
                        Some((_, _, '"')) => {
                            *self = TokenState::LitStr;
                            continue;
                        }
                        Some((_, _, ' ')) => continue,
                        Some(unk) => todo!("{unk:?}"),
                        None => todo!(),
                    });
                }
                Self::LitStr => {
                    let start = buf.len() - it.as_bytes().len();

                    if let Some(pos) = it.as_bytes().find_byteset(b"\\\"") {
                        let end = start + pos;
                        match buf[end] {
                            b'"' => {
                                let data = &buf[start..end];
                                *self = Self::Normal;
                                return Ok((
                                    start..end,
                                    end + 1,
                                    Token::LitStrChunk {
                                        chunk: std::str::from_utf8(data)
                                            .map_err(|_| TokenError::LitStrNotUtf8)?,
                                        last: true,
                                    },
                                ));
                            }
                            b'\\' => todo!(),
                            unk => unreachable!("{unk:?} is not in byteset"),
                        }
                    } else {
                        todo!()
                    }
                }
                Self::Comment(level) => {
                    let start = buf.len() - it.as_bytes().len();
                    let mut cur = start;

                    while let Some(off) = buf[cur..].find_byteset(b"/*") {
                        match buf.get(cur + off..cur + off + 2) {
                            Some(b"/*") => {
                                *level += 1;
                                cur += off + 2;
                            }
                            Some(b"*/") if *level == 0 => {
                                cur += off;
                                *self = Self::Normal;
                                return Ok((
                                    start..cur,
                                    cur + 2,
                                    Token::CommentChunk {
                                        chunk: std::str::from_utf8(&buf[start..cur]).unwrap(),
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
                        buf.len(),
                        Token::CommentChunk {
                            chunk: std::str::from_utf8(it.as_bytes()).unwrap(),
                            last: false,
                        },
                    ));
                }
            }
        }
    }
}

fn skip_whitespace(end: usize, it: bstr::CharIndices) -> usize {
    let (_, actual_end, _) = it
        .take_while(|(_, _, c)| c.is_whitespace() && !matches!(c, '\r' | '\n'))
        .last()
        .unwrap_or((0, end, '\n'));
    actual_end
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
