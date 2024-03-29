#![deny(warnings, clippy::all)]
#![allow(clippy::len_without_is_empty)]

mod action;
pub mod error;
mod loader;
mod parser;
mod span;
mod string;
pub mod token;
pub mod type_trait;
pub mod types;

pub use crate::{loader::LogixLoader, parser::LogixParser};
pub use logix_type_derive::LogixType;
pub use type_trait::LogixType;

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
