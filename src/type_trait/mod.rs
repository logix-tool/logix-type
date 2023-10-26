mod impl_trait;

pub use crate::{
    error::{ParseError, Result, Wanted, Warn},
    parser::LogixParser,
    span::SourceSpan,
    token::{Brace, Delim, Literal, StrTag, StrTagSuffix, Token},
    Map, Str,
};
pub use logix_vfs::LogixVfs;

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

    fn logix_parse<FS: LogixVfs>(p: &mut LogixParser<FS>) -> Result<Value<Self>>;
}
