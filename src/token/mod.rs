use std::fmt;

mod comment;
mod parse;
mod string;
pub use parse::{parse_token, ParseRes, TokenError};

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
    /// Indicates a newline, argument is true if it is the last one (aka EOF)
    Newline(bool),
}

impl<'a> Token<'a> {
    pub fn token_type_name(&self) -> &'static str {
        match self {
            Self::Ident(_) => "identifier",
            Self::Literal(Literal::Str(..)) => "string",
            Self::Brace {
                start: false,
                brace: Brace::Paren,
            } => "`)`",
            Self::Brace {
                start: false,
                brace: Brace::Curly,
            } => "`}`",
            Self::Delim(Delim::Comma) => "`,`",
            Self::Newline(false) => "newline",
            Self::Newline(true) => "end of file",
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
            Self::Brace { .. } | Self::Delim(..) | Self::Newline(..) => {
                write!(f, "{}", self.token_type_name())
            }
        }
    }
}
