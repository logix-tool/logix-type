#![deny(warnings, clippy::all)]
#![allow(clippy::len_without_is_empty)]

mod action;
pub mod error;
mod loader;
mod parser;
mod span;
mod string;
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

// NOTE(2023.10): This is a work-around to test that compilation works
/// ```
/// use logix_type::LogixType;
///
/// #[derive(LogixType)]
/// struct Hello {
///     a: u32,
///     b: u32,
/// }
///```
// NOTE(2023.10): This is a work-around to test that compilation fails
/// ```compile_fail
/// use logix_type::LogixType;
///
/// #[derive(LogixType)]
/// union Hello {
///     a: u32,
///     b: u32,
/// }
/// ```
struct _Dummy;
