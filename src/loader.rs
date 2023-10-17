use std::{
    io::Read,
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicU32, Ordering},
        Arc,
    },
};

use bstr::ByteSlice;
use indexmap::{IndexMap, IndexSet};
use logix_vfs::LogixVfs;

use crate::{error::ParseError, LogixType, __private::LogixParser};

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct FileId {
    loader: u32,
    file: u32,
}

pub(crate) struct CachedFile {
    id: FileId,
    data: Arc<[u8]>,
}

impl CachedFile {
    pub fn lines(&self) -> bstr::LinesWithTerminator {
        self.data.lines_with_terminator()
    }
}

pub struct LogixLoader<FS: LogixVfs> {
    id: u32,
    fs: FS,
    files: IndexMap<PathBuf, Arc<[u8]>>,
    tmp: Vec<u8>,
}

impl<FS: LogixVfs> LogixLoader<FS> {
    pub fn new(fs: FS) -> Self {
        static ID: AtomicU32 = AtomicU32::new(0);
        Self {
            id: ID.fetch_add(1, Ordering::SeqCst) + 1,
            fs,
            files: IndexMap::new(),
            tmp: Vec::with_capacity(0x10000),
        }
    }

    pub(crate) fn open_file_by_id(&self, id: FileId) -> CachedFile {
        assert_eq!(self.id, id.loader);
        self.files
            .get_index(id.file.try_into().unwrap())
            .map(|data| CachedFile {
                id,
                data: data.1.clone(),
            })
            .unwrap()
    }

    pub(crate) fn open_file(&mut self, path: impl AsRef<Path>) -> Result<CachedFile, ParseError> {
        match self.files.entry(self.fs.canonicalize_path(path.as_ref())?) {
            indexmap::map::Entry::Vacant(entry) => {
                let id = FileId {
                    loader: self.id,
                    file: entry.index().try_into().unwrap(),
                };
                self.tmp.clear();
                let mut r = self.fs.open_file(entry.key())?;
                r.read_to_end(&mut self.tmp)
                    .map_err(ParseError::read_error)?;
                let data = entry.insert(Arc::from(self.tmp.as_slice())).clone();
                Ok(CachedFile { id, data })
            }
            indexmap::map::Entry::Occupied(entry) => {
                let id = FileId {
                    loader: self.id,
                    file: entry.index().try_into().unwrap(),
                };
                Ok(CachedFile {
                    id,
                    data: entry.get().clone(),
                })
            }
        }
    }

    pub fn load_file<T: LogixType>(&mut self, path: impl AsRef<Path>) -> Result<T, ParseError> {
        let file = self.open_file(path)?;
        let mut p = LogixParser::new(self, file);

        let ret = T::logix_parse(&mut p)?;

        match p.next_token()? {
            Some(unk) => todo!("{unk:?}"),
            None => Ok(ret.value),
        }
    }
}
