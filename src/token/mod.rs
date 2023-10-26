use std::{borrow::Cow, fmt};

use crate::error::TokenError;

mod comment;
mod parse;
mod string;
pub use parse::{parse_token, ParseRes};

struct ByteSet(&'static str);

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
    const VALID: ByteSet = ByteSet("abcdefghijklmnopqrstuvwxyz0123456789-_");

    fn from_prefix(buf: &[u8]) -> Option<(usize, Self)> {
        if buf.starts_with(b"txt\"") {
            Some((4, Self::Txt))
        } else {
            None
        }
    }
}

impl<'a> fmt::Display for StrTag {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Raw => write!(f, "`#raw`"),
            Self::Esc => write!(f, "`#esc`"),
            Self::Txt => write!(f, "`#txt`"),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct StrTagSuffix(Cow<'static, str>);

impl StrTagSuffix {
    pub fn len(&self) -> usize {
        self.0.len()
    }
}

impl StrTagSuffix {
    pub fn new(num_hashes: usize) -> Self {
        static HASHES: &str = "\"################################";
        Self(if num_hashes < HASHES.len() {
            Cow::Borrowed(&HASHES[..num_hashes + 1])
        } else {
            let mut s = String::with_capacity(num_hashes + 1);
            s.push('"');
            s.extend(std::iter::repeat('#').take(num_hashes));
            Cow::Owned(s)
        })
    }
}

impl AsRef<[u8]> for StrTagSuffix {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref().as_ref()
    }
}

impl<'a> fmt::Display for StrTagSuffix {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "`{}`", self.0)
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Brace {
    /// Curly braces `{}`
    Curly,
    /// Parenthesis `()`
    Paren,
    /// Square brackets `[]`
    Square,
    /// AAngle brackets `<>`
    Angle,
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
    Brace {
        start: bool,
        brace: Brace,
    },
    Delim(Delim),
    Comment(&'a str),
    /// Indicates a newline, argument is true if it is the last one (aka EOF)
    Newline(bool),
}

impl<'a> Token<'a> {
    pub fn token_type_name(&self) -> &'static str {
        match self {
            Self::Ident(_) => "identifier",
            Self::Literal(Literal::Str(..)) => "string",
            Self::Literal(Literal::Num(..)) => "number",
            Self::Brace {
                start: true,
                brace: Brace::Paren,
            } => "`(`",
            Self::Brace {
                start: false,
                brace: Brace::Paren,
            } => "`)`",
            Self::Brace {
                start: true,
                brace: Brace::Curly,
            } => "`{`",
            Self::Brace {
                start: false,
                brace: Brace::Curly,
            } => "`}`",
            Self::Brace {
                start: true,
                brace: Brace::Square,
            } => "`[`",
            Self::Brace {
                start: false,
                brace: Brace::Square,
            } => "`]`",
            Self::Brace {
                start: true,
                brace: Brace::Angle,
            } => "`<`",
            Self::Brace {
                start: false,
                brace: Brace::Angle,
            } => "`>`",
            Self::Delim(Delim::Comma) => "`,`",
            Self::Delim(Delim::Colon) => "`:`",
            Self::Newline(false) => "newline",
            Self::Newline(true) => "end of file",
            Self::Comment(..) => "comment",
        }
    }
}

impl<'a> fmt::Display for Token<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Ident(value) => write!(f, "`{value}`"),
            Self::Literal(..) => todo!(),
            Self::Comment(..) => todo!(),
            Self::Brace { .. } | Self::Delim(..) | Self::Newline(..) => {
                write!(f, "{}", self.token_type_name())
            }
        }
    }
}
