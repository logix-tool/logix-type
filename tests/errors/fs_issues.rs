use logix_vfs::LogixVfs;

use super::*;
use std::{
    collections::HashMap,
    io::{Error, ErrorKind},
    sync::Mutex,
};

#[derive(Debug)]
struct FailFs {
    files: Mutex<HashMap<&'static str, FailFile>>,
}

impl LogixVfs for FailFs {
    type RoFile = FailFile;

    fn canonicalize_path(
        &self,
        path: &std::path::Path,
    ) -> Result<std::path::PathBuf, logix_vfs::Error> {
        Ok(path.into())
    }

    fn open_file(&self, path: &std::path::Path) -> Result<Self::RoFile, logix_vfs::Error> {
        Ok(self
            .files
            .lock()
            .unwrap()
            .remove(path.to_str().unwrap())
            .unwrap())
    }
}

#[derive(Debug)]
struct FailFile {
    res: Vec<std::io::Result<Vec<u8>>>,
}

impl std::io::Read for FailFile {
    fn read(&mut self, _buf: &mut [u8]) -> std::io::Result<usize> {
        Err(self.res.remove(0).unwrap_err())
    }
}

#[test]
fn read_error() {
    let mut loader = LogixLoader::new(FailFs {
        files: Mutex::new(
            [(
                "test.logix",
                FailFile {
                    res: vec![Err(Error::new(ErrorKind::Other, "sorry"))],
                },
            )]
            .into_iter()
            .collect(),
        ),
    });
    println!("{loader:#?}");

    let e = loader.load_file::<String>("test.logix").unwrap_err();

    println!("{e}\n{e:?}");

    assert_eq!(
        e,
        ParseError::FsError(logix_vfs::Error::Other("sorry".into()))
    );
}
