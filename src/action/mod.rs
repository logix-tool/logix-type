use std::path::PathBuf;

use logix_vfs::LogixVfs;

use crate::{
    error::{IncludeError, ParseError, Result},
    parser::LogixParser,
    span::SourceSpan,
    token::{Action, Brace},
    type_trait::Value,
    LogixType,
};

pub fn for_string_data<FS: LogixVfs>(
    action: Action,
    span: SourceSpan,
    p: &mut LogixParser<FS>,
) -> Result<Value<String>> {
    match action {
        Action::Include => {
            let path = p.req_wrapped("@include", Brace::Paren, PathBuf::logix_parse)?;
            let file = p.open_file(&path.value)?;
            Ok(Value {
                span: span.join(&path.span),
                value: std::str::from_utf8(file.data())
                    .map_err(|e| ParseError::IncludeError {
                        span: SourceSpan::from_pos(&file, e.valid_up_to()),
                        while_parsing: "string",
                        error: IncludeError::NotUtf8,
                    })?
                    .into(),
            })
        }
    }
}
