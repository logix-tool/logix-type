pub mod error;
mod loader;
mod parser;
mod token;
mod type_trait;

pub use crate::loader::LogixLoader;
pub use logix_type_derive::LogixType;
pub use type_trait::LogixType;

// Used by macros, no semver guaranties are made here
#[doc(hidden)]
pub mod __private {
    pub use crate::type_trait::*;
}

pub type Map<V> = indexmap::IndexMap<Str, V>;
pub type Str = smol_str::SmolStr;
