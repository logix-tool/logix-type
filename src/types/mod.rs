//! Types that implement the `LogixType` trait

#[macro_use]
mod valid_path;

mod array;
mod data;
mod executable_path;
mod map;
mod string;

pub use self::{
    data::Data,
    executable_path::{ExecutableEnv, ExecutablePath},
    map::Map,
    string::ShortStr,
    valid_path::{FullPath, NameOnlyPath, RelPath, ValidPath},
};
