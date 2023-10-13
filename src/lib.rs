pub mod error;
mod parser;
mod token;
pub mod type_trait;

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
            None => Ok(ret.value),
        }
    }
}

#[cfg(test)]
mod tests {
    use type_trait::ParseError;

    use super::*;
    use crate as logix_mold;

    #[test]
    fn basics() {
        #[derive(logix_mold_derive::LogixType, Debug, PartialEq)]
        struct Unnamed(String, i32);

        #[derive(logix_mold_derive::LogixType, Debug, PartialEq)]
        enum Hello {
            World { yes: i32 },
        }

        #[derive(logix_mold_derive::LogixType, Debug, PartialEq)]
        struct Root {
            some_string: String,
            sub_struct: Unnamed,
            sub_enum: Hello,
        }
        let root = tempfile::tempdir().unwrap();
        std::fs::write(
            root.path().join("config.logix"),
            r#"
Root {
  some_string: "hello"
  sub_struct: Unnamed("Something", 42)
  sub_enum: Hello::World {
    yes: 269
  }
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
                sub_struct: Unnamed("Something".into(), 42),
                sub_enum: Hello::World { yes: 269 },
            }
        );
    }

    #[test]
    fn duplicate_member() {
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
  some_string: "world"
}
        "#
            .trim(),
        )
        .unwrap();
        let mold = LogixMold::new(logix_vfs::RelFs::new(root.path()));

        match mold.load_file::<Root>("config.logix") {
            Err(ParseError::DuplicateMember {
                type_name: "Root",
                member: "some_string",
            }) => {}
            unk => panic!("Expected duplicate member error: {unk:#?}"),
        }
    }
}
