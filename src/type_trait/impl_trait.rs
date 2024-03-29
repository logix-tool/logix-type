use crate::{
    error::{ParseError, Result, Wanted},
    parser::LogixParser,
    token::{Literal, Token},
    type_trait::{LogixTypeDescriptor, LogixValueDescriptor, Value},
    LogixType,
};
use logix_vfs::LogixVfs;

macro_rules! impl_for_int {
    ($signed:literal => $($type:ty),+) => {$(
        impl LogixType for $type {
            fn descriptor() -> &'static LogixTypeDescriptor {
                &LogixTypeDescriptor {
                    name: stringify!($type),
                    doc: "",
                    value: LogixValueDescriptor::Native,
                }
            }

            fn default_value() -> Option<Self> {
                None
            }

            fn logix_parse<FS: LogixVfs>(p: &mut LogixParser<FS>) -> Result<Value<Self>> {
                match p.next_token()? {
                    (span, Token::Literal(Literal::Num(num))) => Ok(Value {
                        value: num.parse().unwrap(), // TODO(2023.10): Return a sensible error
                        span,
                    }),
                    (span, token) => Err(ParseError::UnexpectedToken {
                        span,
                        got_token: token.token_type_name(),
                        wanted: Wanted::LitNum(concat!($signed, " integer")),
                        while_parsing: Self::descriptor().name,
                    }),
                }
            }
        }
    )*};
}

impl_for_int!("signed" => i8, i16, i32, i64);
impl_for_int!("unsigned" => u8, u16, u32, u64);

impl<T: LogixType> LogixType for Option<T> {
    fn descriptor() -> &'static LogixTypeDescriptor {
        T::descriptor()
    }

    fn default_value() -> Option<Self> {
        Some(None)
    }

    fn logix_parse<FS: LogixVfs>(p: &mut LogixParser<FS>) -> Result<Value<Self>> {
        T::logix_parse(p).map(|v| v.map(Some))
    }
}
