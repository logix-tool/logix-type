use std::{fmt, io::Read, path::Path, sync::Arc};

use bstr::ByteSlice;
use indexmap::IndexMap;
use logix_vfs::LogixVfs;

use crate::{error::ParseError, LogixType, __private::LogixParser, token::Token};

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct FileId {
    loader: u32,
    file: u32,
}

#[derive(Clone, PartialEq, Eq)]
pub(crate) struct CachedFile {
    path: Arc<Path>,
    data: Arc<[u8]>,
}

impl CachedFile {
    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn lines(&self) -> bstr::Lines {
        self.data.lines()
    }
}

impl fmt::Debug for CachedFile {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("CachedFile").field(&self.path).finish()
    }
}

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
                    .map_err(ParseError::read_error)?;
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

        p.req_token(T::DESCRIPTOR.name, Token::Newline)?;

        match p.next_token()? {
            (_, Token::Eof) => Ok(ret.value),
            unk => todo!("{unk:?}"),
        }
    }
}
