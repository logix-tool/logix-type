use std::{borrow::Cow, ffi::OsStr, fmt, path::Path};

use logix_vfs::LogixVfs;
use smol_str::SmolStr;

use crate::{
    error::{ParseError, Result, Wanted},
    parser::LogixParser,
    token::{Literal, Token},
    type_trait::{LogixTypeDescriptor, LogixValueDescriptor, Value},
    LogixType,
};

/// Represents a short string, will not need allocation for typical identifiers
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ShortStr {
    value: SmolStr,
}

impl From<String> for ShortStr {
    fn from(value: String) -> Self {
        Self {
            value: value.into(),
        }
    }
}

impl From<SmolStr> for ShortStr {
    fn from(value: SmolStr) -> Self {
        Self { value }
    }
}

impl<'a> From<&'a str> for ShortStr {
    fn from(value: &'a str) -> Self {
        Self {
            value: value.into(),
        }
    }
}

impl<'a> From<Cow<'a, str>> for ShortStr {
    fn from(value: Cow<'a, str>) -> Self {
        Self {
            value: value.into(),
        }
    }
}

impl AsRef<Path> for ShortStr {
    fn as_ref(&self) -> &Path {
        self.value.as_str().as_ref()
    }
}

impl AsRef<OsStr> for ShortStr {
    fn as_ref(&self) -> &OsStr {
        self.value.as_str().as_ref()
    }
}

impl AsRef<str> for ShortStr {
    fn as_ref(&self) -> &str {
        self.value.as_str()
    }
}

impl std::ops::Deref for ShortStr {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl fmt::Display for ShortStr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self.value.as_str(), f)
    }
}

impl fmt::Debug for ShortStr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self.value.as_str(), f)
    }
}

macro_rules! impl_for_str {
    ($($type:ty),+) => {$(
        impl LogixType for $type {
            fn descriptor() -> &'static LogixTypeDescriptor {
                &LogixTypeDescriptor {
                    name: "string",
                    doc: "a valid utf-8 string",
                    value: LogixValueDescriptor::Native,
                }
            }

            fn default_value() -> Option<Self> {
                None
            }

            fn logix_parse<FS: LogixVfs>(p: &mut LogixParser<FS>) -> Result<Value<Self>> {
                Ok(match p.next_token()? {
                    (span, Token::Literal(Literal::Str(value))) => Value {
                        value: <$type>::from(value.decode_str(&span)?),
                        span,
                    },
                    (span, Token::Action(name)) => crate::action::for_string_data(name, span, p)
                        .map(|Value { span, value }| Value { span, value: value.into() })?,
                    (span, token) => return Err(ParseError::UnexpectedToken {
                        span,
                        while_parsing: "string",
                        wanted: Wanted::LitStr,
                        got_token: token.token_type_name(),
                    }),
                })
            }
        }
    )*};
}

impl_for_str!(String, ShortStr);
