//! The `LogixType` trait and types used to describe the types

mod impl_trait;

use crate::{error::Result, parser::LogixParser, span::SourceSpan};
pub use logix_vfs::LogixVfs;

/// Represents a value and the location in the config file
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

/// Describes a type
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

/// Describes a type in the logix config file
pub struct LogixTypeDescriptor {
    /// Name of the type
    pub name: &'static str,
    /// Documentation for the type
    pub doc: &'static str,
    /// Describes the type itself
    pub value: LogixValueDescriptor,
}

/// This trait is used to represent types that can be stored in a logix config.
pub trait LogixType: Sized {
    /// A description of the type, intended used for documentation and auto-completion
    fn descriptor() -> &'static LogixTypeDescriptor;
    /// If the value is optional, this returns `Some`
    fn default_value() -> Option<Self>;
    /// Parse the value from the given parser state
    fn logix_parse<FS: LogixVfs>(p: &mut LogixParser<FS>) -> Result<Value<Self>>;
}
