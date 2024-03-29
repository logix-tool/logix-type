use std::borrow::Cow;

use crate::{
    error::{ParseError, Result},
    span::SourceSpan,
    token::StrTag,
};

mod esc;
mod txt;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct StrLit<'a> {
    tag: StrTag,
    value: &'a str,
}

impl<'a> StrLit<'a> {
    pub fn new(tag: StrTag, value: &'a str) -> Self {
        Self { tag, value }
    }

    pub fn decode_str(&self, span: &SourceSpan) -> Result<Cow<'a, str>> {
        match self.tag {
            StrTag::Raw => Ok(Cow::Borrowed(self.value)),
            StrTag::Esc => crate::string::esc::decode_str(self.value)
                .map(Cow::Owned)
                .map_err(|(off, len, error)| ParseError::StrEscError {
                    span: span.with_off(off, len),
                    error,
                }),
            StrTag::Txt => Ok(Cow::Owned(crate::string::txt::decode_str(self.value))),
        }
    }
}
