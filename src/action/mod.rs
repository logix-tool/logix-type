use std::path::PathBuf;

use logix_vfs::LogixVfs;

use crate::{
    error::{IncludeError, ParseError, Result},
    loader::CachedFile,
    parser::LogixParser,
    span::SourceSpan,
    token::{Action, Brace},
    type_trait::Value,
    LogixType,
};

pub(crate) fn for_include<FS: LogixVfs>(
    span: SourceSpan,
    p: &mut LogixParser<FS>,
) -> Result<Value<(PathBuf, CachedFile)>> {
    let path = p
        .req_wrapped("@include", Brace::Paren, PathBuf::logix_parse)
        .map(|v| v.join_with_span(span))?;
    let file = p.open_file(&path.value.value).map_err(|error| {
        dbg!(ParseError::IncludeError {
            span: path.span.clone(),
            while_parsing: "string",
            error: IncludeError::Open(error),
        })
    })?;
    Ok(path.map(|p| (p.value, file)))
}

pub fn for_string_data<FS: LogixVfs>(
    action: Action,
    span: SourceSpan,
    p: &mut LogixParser<FS>,
) -> Result<Value<String>> {
    match action {
        Action::Include => {
            let file = for_include(span, p)?;
            Ok(Value {
                span: file.span,
                value: std::str::from_utf8(file.value.1.data())
                    .map_err(|e| ParseError::IncludeError {
                        span: SourceSpan::from_pos(&file.value.1, e.valid_up_to()),
                        while_parsing: "string",
                        error: IncludeError::NotUtf8,
                    })?
                    .into(),
            })
        }
    }
}
