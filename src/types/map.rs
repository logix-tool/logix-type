use indexmap::IndexMap;
use logix_vfs::LogixVfs;

use crate::{
    error::{Result, Warn},
    parser::LogixParser,
    token::{Brace, Token},
    type_trait::{LogixTypeDescriptor, LogixValueDescriptor, Value},
    types::ShortStr,
    LogixType,
};

pub type Map<V> = IndexMap<ShortStr, V>;

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
