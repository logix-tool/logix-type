pub mod error;
mod parser;
mod token;
mod type_trait;

use std::{path::Path, sync::Arc};

use error::Result;
use logix_vfs::LogixVfs;

pub use logix_type_derive::LogixType;
pub use type_trait::LogixType;

// Used by macros, no semver guaranties are made here
#[doc(hidden)]
pub mod __private {
    pub use crate::type_trait::*;
}

pub type Map<V> = indexmap::IndexMap<Str, V>;
pub type Str = smol_str::SmolStr;

pub fn load_file<T: LogixType, FS: LogixVfs>(fs: Arc<FS>, path: impl AsRef<Path>) -> Result<T, FS> {
    let path = Arc::<Path>::from(path.as_ref());
    let file = fs.open_file(&path)?;
    let mut p = parser::LogixParser::new(path, fs.clone(), file);

    let ret = T::logix_parse(&mut p)?;

    match p.next_token()? {
        Some(unk) => todo!("{unk:?}"),
        None => Ok(ret.value),
    }
}
