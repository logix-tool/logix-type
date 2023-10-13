use std::ops::Range;

use bstr::ByteSlice;

use crate::error::{ParseError, Result};

#[derive(Debug, PartialEq, Eq)]
pub enum Brace {
    /// Curly braces `{}`
    Curly,
    /*
    /// Parenthesis `()`
    Paren,
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
    BraceStart(Brace),
    BraceEnd(Brace),
    Colon,
    Newline,
}

impl<'a> Token<'a> {
    /// A single token can't be bigger than this
    pub const MAX_LEN: usize = 1024;
}

pub(crate) enum TokenState {
    Normal,
    LitStr,
}

impl TokenState {
    pub fn parse_token<'a>(
        &mut self,
        buf: &'a [u8],
    ) -> Result<Option<(Range<usize>, usize, Token<'a>)>> {
        let mut it = buf.char_indices();

        loop {
            match self {
                Self::Normal => {
                    return Ok(Some(match it.next() {
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
                        Some((start, end, '{')) => (start..end, skip_any_whitespace(end, it), Token::BraceStart(Brace::Curly)),
                        Some((start, end, '}')) => (start..end, skip_any_whitespace(end, it), Token::BraceEnd(Brace::Curly)),
                        Some((start, end, ':')) => (start..end, skip_whitespace(end, it), Token::Colon),
                        Some((start, end, '\r' | '\n')) => (start..end, skip_any_whitespace(end, it), Token::Newline),
                        Some((_, _, '"')) => {
                            *self = TokenState::LitStr;
                            continue;
                        }
                        Some((_, _, ' ')) => continue,
                        Some(unk) => todo!("{unk:?}"),
                        None => return Ok(None),
                    }));
                }
                Self::LitStr => {
                    let start = buf.len() - it.as_bytes().len();

                    if let Some(pos) = it.as_bytes().find_byteset(b"\\\"") {
                        let end = start + pos;
                        match buf[end] {
                            b'"' => {
                                let data = &buf[start..end];
                                *self = Self::Normal;
                                return Ok(Some((
                                    start..end,
                                    end + 1,
                                    Token::LitStrChunk {
                                        chunk: std::str::from_utf8(data)
                                            .map_err(|_| ParseError::LitStrNotUtf8)?,
                                        last: true,
                                    },
                                )));
                            }
                            b'\\' => todo!(),
                            unk => unreachable!("{unk:?} is not in byteset"),
                        }
                    } else {
                        todo!()
                    }
                }
            }
        }
    }
}

fn skip_any_whitespace(end: usize, it: bstr::CharIndices) -> usize {
    let (_, actual_end, _) = it
        .take_while(|(_, _, c)| c.is_whitespace())
        .last()
        .unwrap_or((0, end, '\n'));
    actual_end
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
