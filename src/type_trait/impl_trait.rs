use std::path::PathBuf;

use super::{
    Brace, Literal, LogixParser, LogixType, LogixTypeDescriptor, LogixValueDescriptor, LogixVfs,
    Map, ParseError, Result, Str, StrTag, Token, Value, Wanted, Warn,
};

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
                    (span, Token::Literal(Literal::Str(StrTag::Raw, value))) => Value {
                        value: <$type>::from(value),
                        span,
                    },
                    (span, Token::Literal(Literal::Str(StrTag::Esc, value))) => {
                        Value {
                            value: crate::string::esc::decode_str(value)
                                .map_err(|(off, len, error)| {
                                    ParseError::StrEscError { span: span.with_off(off, len), error }
                                })?
                                .into(),
                            span,
                        }
                    }
                    (span, Token::Literal(Literal::Str(StrTag::Txt, value))) => {
                        Value {
                            value: crate::string::txt::decode_str(value).into(),
                            span,
                        }
                    }
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

impl_for_str!(String, Str);

impl LogixType for PathBuf {
    fn descriptor() -> &'static LogixTypeDescriptor {
        static RET: LogixTypeDescriptor = LogixTypeDescriptor {
            name: "path",
            doc: "a valid path",
            value: LogixValueDescriptor::Native,
        };
        &RET
    }

    fn default_value() -> Option<Self> {
        None
    }

    fn logix_parse<FS: LogixVfs>(p: &mut LogixParser<FS>) -> Result<Value<Self>> {
        Ok(match p.next_token()? {
            (span, Token::Literal(Literal::Str(StrTag::Raw, value))) => Value {
                value: PathBuf::from(value),
                span,
            },
            (span, Token::Literal(Literal::Str(StrTag::Esc, value))) => Value {
                value: crate::string::esc::decode_str(value)
                    .map_err(|(off, len, error)| ParseError::StrEscError {
                        span: span.with_off(off, len),
                        error,
                    })?
                    .into(),
                span,
            },
            (span, token) => {
                return Err(ParseError::UnexpectedToken {
                    span,
                    while_parsing: "path",
                    wanted: Wanted::LitStr,
                    got_token: token.token_type_name(),
                })
            }
        })
    }
}

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

impl<T: LogixType> LogixType for Map<T> {
    fn descriptor() -> &'static LogixTypeDescriptor {
        static RET: LogixTypeDescriptor = LogixTypeDescriptor {
            name: "string",
            doc: "a valid utf-8 string",
            value: LogixValueDescriptor::Native,
        };
        &RET
    }

    fn default_value() -> Option<Self> {
        Some(Self::new())
    }

    fn logix_parse<FS: LogixVfs>(p: &mut LogixParser<FS>) -> Result<Value<Self>> {
        let mut map = Map::new();

        let start = p.req_token(
            "map",
            Token::Brace {
                start: true,
                brace: Brace::Curly,
            },
        )?;
        p.req_token("map", Token::Newline(false))?;

        while let Some((key, value)) = p.read_key_value("map", Brace::Curly)? {
            if let (i, Some(_)) = map.insert_full(key.value, value.value) {
                p.warning(Warn::DuplicateMapEntry {
                    span: key.span,
                    key: map.get_index(i).unwrap().0.clone(),
                })?;
            }
        }

        Ok(Value {
            value: map,
            span: start,
        })
    }
}

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
