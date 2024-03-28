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

impl<T> Value<T> {
    pub fn map<R>(self, f: impl FnOnce(T) -> R) -> Value<R> {
        Value {
            span: self.span,
            value: f(self.value),
        }
    }

    pub fn join_with_span(mut self, span: SourceSpan) -> Self {
        self.span = self.span.join(&span);
        self
    }
}

pub enum LogixValueDescriptor {
    /// A native type that can be specified by a literal
    Native,
    /// Describes a tuple of various types
    Tuple {
        members: Vec<&'static LogixTypeDescriptor>,
    },
    /// Describes the named members of a struct
    Struct {
        members: Vec<(&'static str, &'static LogixTypeDescriptor)>,
    },
    /// Describes the variants of an enum
    Enum { variants: Vec<LogixTypeDescriptor> },
}

pub struct LogixTypeDescriptor {
    pub name: &'static str,
    pub doc: &'static str,
    pub value: LogixValueDescriptor,
}

pub trait LogixType: Sized {
    fn descriptor() -> &'static LogixTypeDescriptor;
    fn default_value() -> Option<Self>;
    fn logix_parse<FS: LogixVfs>(p: &mut LogixParser<FS>) -> Result<Value<Self>>;
}
