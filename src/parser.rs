use crate::{
    error::{ParseError, Result, SourceSpan, Warn},
    loader::{CachedFile, LogixLoader},
    token::{Brace, Token, TokenState},
    type_trait::Value,
    LogixType,
};
use logix_vfs::LogixVfs;
use smol_str::SmolStr;

pub struct LogixParser<'fs, 'f, FS: LogixVfs> {
    _loader: &'fs mut LogixLoader<FS>,
    file: &'f CachedFile,
    lines: bstr::Lines<'f>,
    cur_line: &'f [u8],
    cur_col: usize,
    cur_ln: usize,
    state: TokenState,
}

impl<'fs, 'f, FS: LogixVfs> LogixParser<'fs, 'f, FS> {
    pub(crate) fn new(loader: &'fs mut LogixLoader<FS>, file: &'f CachedFile) -> Self {
        Self {
            _loader: loader,
            file,
            lines: file.lines(),
            cur_line: b"",
            cur_col: 0,
            cur_ln: 0,
            state: TokenState::Normal,
        }
    }

    pub fn next_token(&mut self) -> Result<Option<(SourceSpan, Token)>> {
        Ok(self
            .raw_next_token(true)?
            .map(|(span, _, token)| (span, token)))
    }

    pub fn warning(&self, warning: Warn) -> Result<()> {
        // TODO(2023.10): Make it possible to allow warnings
        Err(ParseError::Warning(warning))
    }

    fn raw_next_token(&mut self, advance: bool) -> Result<Option<(SourceSpan, usize, Token)>> {
        while self.cur_col == self.cur_line.len() {
            if let Some(line) = self.lines.next() {
                self.cur_line = line;
                self.cur_col = 0;
                self.cur_ln += 1;
            } else {
                return Ok(None);
            }
        }

        match self.state.parse_token(&self.cur_line[self.cur_col..]) {
            Ok(Some((range, size, token))) => {
                let span = SourceSpan::new(
                    &self.file,
                    self.cur_ln,
                    self.cur_col + range.start,
                    range.len(),
                );
                if advance {
                    self.cur_col += size;
                }
                Ok(Some((span, size, token)))
            }
            unk => todo!("{unk:?}"),
        }
    }

    pub fn read_key_value<T: LogixType>(
        &mut self,
        end_brace: Brace,
    ) -> Result<Option<(Value<SmolStr>, Value<T>)>> {
        match self.next_token()? {
            Some((span, Token::Ident(key))) => {
                let key = Value {
                    value: SmolStr::new(key),
                    span,
                };

                match self.next_token()? {
                    Some((_, Token::Colon)) => {}
                    unk => todo!("{unk:#?}"),
                }

                let value = T::logix_parse(self)?;

                match self.next_token()? {
                    Some((_, Token::Newline)) => {}
                    unk => todo!("{unk:#?}"),
                }

                Ok(Some((key, value)))
            }
            Some((_, Token::BraceEnd(brace))) if brace == end_brace => Ok(None),
            unk => todo!("{unk:#?}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use logix_vfs::RelFs;

    use crate::token::Brace;

    use super::*;

    fn s(file: &CachedFile, ln: usize, start: usize, len: usize) -> SourceSpan {
        SourceSpan::new(&file, ln, start, len)
    }

    #[test]
    fn basics() -> Result<()> {
        let root = tempfile::tempdir().unwrap();
        std::fs::write(root.path().join("test.logix"), b"Hello { world: \"!!!\" }").unwrap();

        let mut loader = LogixLoader::new(RelFs::new(root.path()));
        let f = loader.open_file("test.logix")?;
        let f = &f;
        let mut p = LogixParser::new(&mut loader, f);

        assert_eq!(
            p.next_token()?,
            Some((s(f, 1, 0, 5), Token::Ident("Hello")))
        );
        assert_eq!(
            p.next_token()?,
            Some((s(f, 1, 6, 1), Token::BraceStart(Brace::Curly)))
        );
        assert_eq!(
            p.next_token()?,
            Some((s(f, 1, 8, 5), Token::Ident("world")))
        );
        assert_eq!(p.next_token()?, Some((s(f, 1, 13, 1), Token::Colon)));
        assert_eq!(
            p.next_token()?,
            Some((
                s(f, 1, 16, 3),
                Token::LitStrChunk {
                    chunk: "!!!",
                    last: true
                }
            ))
        );
        assert_eq!(
            p.next_token()?,
            Some((s(f, 1, 21, 1), Token::BraceEnd(Brace::Curly)))
        );
        assert_eq!(p.next_token()?, None);
        Ok(())
    }
}
