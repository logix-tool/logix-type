use std::path::Path;

use super::*;

trait PathLike<'a>:
    LogixType
    + fmt::Debug
    + PartialEq
    + TryFrom<&'a str, Error = PathError>
    + TryFrom<&'a Path, Error = PathError>
    + TryFrom<PathBuf, Error = PathError>
    + TryFrom<String, Error = PathError>
{
}

impl<'a> PathLike<'a> for FullPath {}
impl<'a> PathLike<'a> for RelPath {}
impl<'a> PathLike<'a> for NameOnlyPath {}
impl<'a> PathLike<'a> for ValidPath {}
impl<'a> PathLike<'a> for ExecutablePath {}

fn path_test<'a, T: PathLike<'a>>(
    path: &'a str,
    col_off: usize,
    col_len: usize,
    underline: &str,
    error: PathError,
    err_str: &str,
) {
    let col = 8 + col_off;
    let mut l = Loader::init().with_file(
        "test.logix",
        format!("GenStruct {{\n  aaa: 10\n  bbbb: {path:?}\n}}").as_bytes(),
    );
    let e = l.parse_file::<GenStruct<T>>("test.logix");

    assert_eq!(
        e,
        ParseError::PathError {
            span: l.span("test.logix", 3, col, col_len),
            error,
        }
    );

    assert_eq!(
        debval(&e),
        [
            format!("\n"),
            format!("error: Failed to parse path\n"),
            format!("   ---> test.logix:3:{col}\n"),
            format!("    |\n"),
            format!("  2 |   aaa: 10\n"),
            format!("  3 |   bbbb: {path:?}\n"),
            format!("    |         {underline} {err_str}\n"),
            format!("  4 | }}\n"),
        ]
        .into_iter()
        .collect::<String>(),
    );

    assert_eq!(
        disval(&e),
        format!("Failed to parse path, {err_str} in test.logix:3:{col}")
    );

    assert_eq!(T::try_from(path), Err(error));
    assert_eq!(T::try_from(Path::new(path)), Err(error));
    assert_eq!(T::try_from(PathBuf::from(path)), Err(error));
    assert_eq!(T::try_from(String::from(path)), Err(error));
}

fn generic_errors_test<'a, T: PathLike<'a>>(wanted: Wanted) {
    path_test::<T>(
        "",
        0,
        2,
        "^^",
        PathError::EmptyPath,
        "the specified path is empty",
    );

    path_test::<T>(
        "\"",
        0,
        4,
        "^^^^",
        PathError::InvalidChar('"'),
        "the path contains the invalid character '\"'",
    );

    path_test::<T>(
        "\n",
        0,
        4,
        "^^^^",
        PathError::InvalidChar('\n'),
        "the path contains the invalid character '\\n'",
    );

    path_test::<T>(
        "|",
        0,
        3,
        "^^^",
        PathError::InvalidChar('|'),
        "the path contains the invalid character '|'",
    );

    path_test::<T>(
        "'",
        0,
        3,
        "^^^",
        PathError::InvalidChar('\''),
        "the path contains the invalid character '\\''",
    );

    {
        let mut l = Loader::init().with_file("test.logix", format!("0").as_bytes());
        assert_eq!(
            l.parse_file::<T>("test.logix"),
            ParseError::UnexpectedToken {
                span: l.span("test.logix", 1, 0, 1),
                while_parsing: T::descriptor().name,
                got_token: "number",
                wanted
            }
        );
    }
}

#[test]
fn load_full_path() {
    path_test::<FullPath>(
        "hello/world.txt",
        0,
        17,
        "^^^^^^^^^^^^^^^^^",
        PathError::NotAbsolute,
        "expected an absolute path",
    );

    generic_errors_test::<FullPath>(Wanted::FullPath);
}

#[test]
fn load_rel_path() {
    path_test::<RelPath>(
        "/hello/world.txt",
        0,
        18,
        "^^^^^^^^^^^^^^^^^^",
        PathError::NotRelative,
        "expected a relative path",
    );

    generic_errors_test::<RelPath>(Wanted::RelPath);
}

#[test]
fn load_name_only_path() {
    path_test::<NameOnlyPath>(
        "/hello/world.txt",
        0,
        18,
        "^^^^^^^^^^^^^^^^^^",
        PathError::NotName,
        "expected either a file or directory name",
    );

    path_test::<NameOnlyPath>(
        "hello/world.txt",
        0,
        17,
        "^^^^^^^^^^^^^^^^^",
        PathError::NotName,
        "expected either a file or directory name",
    );

    generic_errors_test::<NameOnlyPath>(Wanted::NameOnlyPath);
}

#[test]
fn load_valid_path() {
    generic_errors_test::<ValidPath>(Wanted::ValidPath);
}

#[test]
fn load_executable_path() {
    path_test::<ExecutablePath>(
        "hello/world.txt",
        0,
        17,
        "^^^^^^^^^^^^^^^^^",
        PathError::NotFullOrNameOnly,
        "expected file name or absolute path",
    );

    generic_errors_test::<ExecutablePath>(Wanted::ExecutablePath);
}
