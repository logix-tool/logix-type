use std::{fmt, io::Read, path::Path, sync::Arc};

use bstr::ByteSlice;
use indexmap::IndexMap;
use logix_vfs::LogixVfs;

use crate::{error::ParseError, parser::LogixParser, token::Token, type_trait::LogixType};

#[derive(Clone, PartialEq, Eq)]
pub(crate) struct CachedFile {
    path: Arc<Path>,
    data: Arc<[u8]>,
}

impl CachedFile {
    pub(crate) fn empty() -> CachedFile {
        Self {
            path: Arc::from(Path::new("")),
            data: Arc::from(b"".as_slice()),
        }
    }

    #[cfg(test)]
    pub(crate) fn from_slice(path: impl AsRef<Path>, data: &[u8]) -> CachedFile {
        Self {
            path: Arc::from(Path::new(path.as_ref())),
            data: Arc::from(data),
        }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn lines(&self) -> bstr::Lines {
        self.data.lines()
    }

    pub fn data(&self) -> &[u8] {
        &self.data
    }
}

impl fmt::Debug for CachedFile {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("CachedFile").field(&self.path).finish()
    }
}

#[derive(Debug)]
pub struct LogixLoader<FS: LogixVfs> {
    fs: FS,
    files: IndexMap<Arc<Path>, Arc<[u8]>>,
    tmp: Vec<u8>,
}

impl<FS: LogixVfs> LogixLoader<FS> {
    pub fn new(fs: FS) -> Self {
        Self {
            fs,
            files: IndexMap::new(),
            tmp: Vec::with_capacity(0x10000),
        }
    }

    pub(crate) fn get_file(&self, path: impl AsRef<Path>) -> Option<CachedFile> {
        let (key, value) = self.files.get_key_value(path.as_ref())?;

        Some(CachedFile {
            path: key.clone(),
            data: value.clone(),
        })
    }

    pub(crate) fn open_file(&mut self, path: impl AsRef<Path>) -> Result<CachedFile, ParseError> {
        match self
            .files
            .entry(Arc::<Path>::from(self.fs.canonicalize_path(path.as_ref())?))
        {
            indexmap::map::Entry::Vacant(entry) => {
                let path = entry.key().clone();
                self.tmp.clear();
                let mut r = self.fs.open_file(entry.key())?;
                r.read_to_end(&mut self.tmp)
                    .map_err(|e| logix_vfs::Error::from_io(entry.key().to_path_buf(), e))?;
                let data = entry.insert(Arc::from(self.tmp.as_slice())).clone();
                Ok(CachedFile { path, data })
            }
            indexmap::map::Entry::Occupied(entry) => Ok(CachedFile {
                path: entry.key().clone(),
                data: entry.get().clone(),
            }),
        }
    }

    pub fn load_file<T: LogixType>(&mut self, path: impl AsRef<Path>) -> Result<T, ParseError> {
        let file = self.open_file(path)?;
        let mut p = LogixParser::new(self, &file);

        let ret = T::logix_parse(&mut p)?;

        // This will either skip any newlines and comments, or return EOF
        p.req_newline(T::descriptor().name)?;

        // From now on EOF should always be returned
        p.req_token(T::descriptor().name, Token::Newline(true))?;

        Ok(ret.value)
    }
}

#[cfg(test)]
mod tests {
    use logix_vfs::RelFs;

    use super::*;

    #[test]
    fn basics() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(tmp.path().join("test.logix"), b"10").unwrap();

        format!(
            "{:?}",
            CachedFile {
                path: Arc::from(Path::new("a")),
                data: Arc::from(b"".as_slice()),
            }
        );

        let mut loader = LogixLoader::new(RelFs::new(tmp.path()));
        loader.load_file::<u32>("test.logix").unwrap();
        loader.load_file::<u32>("test.logix").unwrap(); // Twice to test cache
    }
}
