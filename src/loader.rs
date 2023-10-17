use std::{
    path::{Path, PathBuf},
    sync::atomic::{AtomicU32, Ordering},
};

use indexmap::IndexSet;
use logix_vfs::LogixVfs;

use crate::{error::ParseError, LogixType, __private::LogixParser};

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct FileId {
    loader: u32,
    file: u32,
}

pub struct LogixLoader<FS: LogixVfs> {
    id: u32,
    fs: FS,
    file_ids: IndexSet<PathBuf>,
}

impl<FS: LogixVfs> LogixLoader<FS> {
    pub fn new(fs: FS) -> Self {
        static ID: AtomicU32 = AtomicU32::new(0);
        Self {
            id: ID.fetch_add(1, Ordering::SeqCst) + 1,
            fs,
            file_ids: IndexSet::new(),
        }
    }

    pub(crate) fn open_file_by_id(&self, id: FileId) -> Result<(PathBuf, FS::RoFile), ParseError> {
        assert_eq!(self.id, id.loader);
        let path = self
            .file_ids
            .get_index(id.file.try_into().unwrap())
            .unwrap();
        let file = self.fs.open_file(path)?;
        Ok((path.clone(), file))
    }

    pub(crate) fn open_file(
        &mut self,
        path: impl AsRef<Path>,
    ) -> Result<(FileId, FS::RoFile), ParseError> {
        let id = FileId {
            loader: self.id,
            file: self
                .file_ids
                .insert_full(self.fs.canonicalize_path(path.as_ref())?)
                .0
                .try_into()
                .unwrap(),
        };

        Ok((id, self.fs.open_file(path.as_ref())?))
    }

    pub fn load_file<T: LogixType>(&mut self, path: impl AsRef<Path>) -> Result<T, ParseError> {
        let (id, r) = self.open_file(path)?;
        let mut p = LogixParser::new(self, id, r);

        let ret = T::logix_parse(&mut p)?;

        match p.next_token()? {
            Some(unk) => todo!("{unk:?}"),
            None => Ok(ret.value),
        }
    }
}
