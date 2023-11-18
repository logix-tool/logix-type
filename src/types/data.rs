use std::path::PathBuf;

use logix_vfs::LogixVfs;

use crate::{
    error::Result,
    parser::LogixParser,
    token::{Action, Token},
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

    fn default_value() -> Option<Self> {
        None
    }

    fn logix_parse<FS: LogixVfs>(p: &mut LogixParser<FS>) -> Result<Value<Self>> {
        if let Some(ret) = p.forked(|p| match p.next_token()? {
            (span, Token::Action(Action::Include)) => {
                let file = crate::action::for_include(span, p)?;
                Ok(Some(file.map(|f| Data::ByPath(f.0))))
            }
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
