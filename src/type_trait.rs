use std::io::Read;

pub use crate::{
    error::{ParseError, Result},
    parser::LogixParser,
    token::{Brace, Token},
};

pub enum LogixValueDescriptor {
    Native,
    Struct {
        members: &'static [(&'static str, &'static LogixTypeDescriptor)],
    },
}

pub struct LogixTypeDescriptor {
    pub name: &'static str,
    pub doc: &'static str,
    pub value: LogixValueDescriptor,
}

pub trait LogixType: Sized {
    const DESCRIPTOR: &'static LogixTypeDescriptor;

    fn logix_parse<R: Read>(p: &mut LogixParser<R>) -> Result<Self>;
}

impl LogixType for String {
    const DESCRIPTOR: &'static LogixTypeDescriptor = &LogixTypeDescriptor {
        name: "string",
        doc: "a valid utf-8 string",
        value: LogixValueDescriptor::Native,
    };

    fn logix_parse<R: Read>(p: &mut LogixParser<R>) -> Result<Self> {
        Ok(match p.next_token()? {
            Some((_, Token::LitStrChunk { chunk, last: true })) => String::from(chunk),
            unk => todo!("{unk:#?}"),
        })
    }
}
