pub mod error;
mod parser;
mod token;
mod type_trait;

use std::{path::Path, sync::Arc};

use error::Result;
use logix_vfs::LogixVfs;

pub use logix_mold_derive::LogixType;
pub use type_trait::LogixType;

pub struct LogixMold<FS> {
    fs: FS,
}

impl<FS: LogixVfs> LogixMold<FS> {
    pub fn new(fs: FS) -> Self {
        Self { fs }
    }

    pub fn load_file<T: LogixType>(&self, path: impl AsRef<Path>) -> Result<T> {
        let path = Arc::<Path>::from(path.as_ref());
        let file = self.fs.open_file(&path)?;
        let mut p = parser::LogixParser::new(path, file);

        let ret = T::logix_parse(&mut p)?;

        match p.next_token()? {
            Some(unk) => todo!("{unk:?}"),
            None => Ok(ret),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate as logix_mold;

    #[test]
    fn basics() {
        #[derive(logix_mold_derive::LogixType, Debug, PartialEq)]
        struct Root {
            some_string: String,
        }
        let root = tempfile::tempdir().unwrap();
        std::fs::write(
            root.path().join("config.logix"),
            r#"
Root {
  some_string: "hello"
}
        "#
            .trim(),
        )
        .unwrap();
        let mold = LogixMold::new(logix_vfs::RelFs::new(root.path()));

        assert_eq!(
            mold.load_file::<Root>("config.logix").unwrap(),
            Root {
                some_string: "hello".into(),
            }
        );
    }
}
