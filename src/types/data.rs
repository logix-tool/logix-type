use std::path::PathBuf;

use logix_vfs::LogixVfs;

use crate::{
    error::Result,
    parser::LogixParser,
    token::{Brace, Token},
    type_trait::{LogixTypeDescriptor, Value},
    LogixType,
};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Data<T> {
    ByPath(PathBuf),
    Inline(T),
}

impl<T: LogixType> LogixType for Data<T> {
    fn descriptor() -> &'static LogixTypeDescriptor {
        T::descriptor()
    }

    fn logix_parse<FS: LogixVfs>(p: &mut LogixParser<FS>) -> Result<Value<Self>> {
        if let Some(ret) = p.forked(|p| match p.next_token()? {
            (_, Token::Ident("ByPath")) => p
                .req_wrapped("ByPath", Brace::Paren, PathBuf::logix_parse)
                .map(|Value { span, value }| {
                    Some(Value {
                        span,
                        value: Self::ByPath(value),
                    })
                }),
            _ => Ok(None),
        })? {
            Ok(ret)
        } else {
            T::logix_parse(p).map(|Value { span, value }| Value {
                span,
                value: Self::Inline(value),
            })
        }
    }
}
