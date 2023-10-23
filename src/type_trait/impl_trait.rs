use super::{
    Brace, LogixParser, LogixType, LogixTypeDescriptor, LogixValueDescriptor, LogixVfs, Map,
    Result, Str, Token, Value, Warn,
};

macro_rules! impl_for_str {
    ($($type:ty),+) => {$(
        impl LogixType for $type {
            const DESCRIPTOR: &'static LogixTypeDescriptor = &LogixTypeDescriptor {
                name: "string",
                doc: "a valid utf-8 string",
                value: LogixValueDescriptor::Native,
            };

            fn logix_parse<FS: LogixVfs>(p: &mut LogixParser<FS>) -> Result<Value<Self>> {
                Ok(match p.next_token()? {
                    (span, Token::LitStrChunk { chunk, last: true }) => Value {
                        value: <$type>::from(chunk),
                        span,
                    },
                    unk => todo!("{unk:#?}"),
                })
            }
        }
    )*};
}

impl_for_str!(String, Str);

macro_rules! impl_for_int {
    ($signed:literal => $($type:ty),+) => {$(
        impl LogixType for $type {
            const DESCRIPTOR: &'static LogixTypeDescriptor = &LogixTypeDescriptor {
                name: stringify!($type),
                doc: "",
                value: LogixValueDescriptor::Native,
            };

            fn logix_parse<FS: LogixVfs>(p: &mut LogixParser<FS>) -> Result<Value<Self>> {
                Ok(match p.next_token()? {
                    (span, Token::LitNumber(num)) => Value {
                        value: num.parse().unwrap(), // TODO(2023.10): Return a sensible error
                        span,
                    },
                    unk => todo!("{unk:#?}"),
                })
            }
        }
    )*};
}

impl_for_int!("signed" => i8, i16, i32, i64);
impl_for_int!("unsigned" => u8, u16, u32, u64);

impl<T: LogixType> LogixType for Map<T> {
    const DESCRIPTOR: &'static LogixTypeDescriptor = &LogixTypeDescriptor {
        name: "string",
        doc: "a valid utf-8 string",
        value: LogixValueDescriptor::Native,
    };

    fn logix_parse<FS: LogixVfs>(p: &mut LogixParser<FS>) -> Result<Value<Self>> {
        let mut map = Map::new();

        let start = p.req_token("map", Token::BraceStart(Brace::Curly))?;
        p.req_token("map", Token::Newline)?;

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
