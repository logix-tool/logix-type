use std::io::Read;

pub use crate::{
    error::{ParseError, Result, SourceSpan},
    parser::LogixParser,
    token::{Brace, Token},
};

pub struct Value<T> {
    pub value: T,
    pub span: SourceSpan,
}

pub enum LogixValueDescriptor {
    Native,
    Tuple {
        members: &'static [&'static LogixTypeDescriptor],
    },
    Struct {
        members: &'static [(&'static str, &'static LogixTypeDescriptor)],
    },
    Enum {
        variants: &'static [&'static LogixTypeDescriptor],
    },
}

pub struct LogixTypeDescriptor {
    pub name: &'static str,
    pub doc: &'static str,
    pub value: LogixValueDescriptor,
}

pub trait LogixType: Sized {
    const DESCRIPTOR: &'static LogixTypeDescriptor;

    fn logix_parse<R: Read>(p: &mut LogixParser<R>) -> Result<Value<Self>>;
}

impl LogixType for String {
    const DESCRIPTOR: &'static LogixTypeDescriptor = &LogixTypeDescriptor {
        name: "string",
        doc: "a valid utf-8 string",
        value: LogixValueDescriptor::Native,
    };

    fn logix_parse<R: Read>(p: &mut LogixParser<R>) -> Result<Value<Self>> {
        Ok(match p.next_token()? {
            Some((span, Token::LitStrChunk { chunk, last: true })) => Value {
                value: String::from(chunk),
                span,
            },
            unk => todo!("{unk:#?}"),
        })
    }
}

impl LogixType for i32 {
    const DESCRIPTOR: &'static LogixTypeDescriptor = &LogixTypeDescriptor {
        name: "i32",
        doc: "a 32bit signed integer",
        value: LogixValueDescriptor::Native,
    };

    fn logix_parse<R: Read>(p: &mut LogixParser<R>) -> Result<Value<Self>> {
        Ok(match p.next_token()? {
            Some((span, Token::LitDigit(num))) => Value {
                value: num.parse().unwrap(),
                span,
            },
            unk => todo!("{unk:#?}"),
        })
    }
}
