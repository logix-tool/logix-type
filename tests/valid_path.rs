use logix_type::{
    types::{FullPath, NameOnlyPath, RelPath, ValidPath},
    LogixType,
};
use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
};

fn path_as_ref(v: &impl AsRef<Path>) -> &Path {
    v.as_ref()
}

fn os_str_as_ref(v: &impl AsRef<OsStr>) -> &OsStr {
    v.as_ref()
}

fn path_as_deref(v: &impl std::ops::Deref<Target = Path>) -> &Path {
    v
}

macro_rules! run_basic_tests {
    ($type:ident, $path:literal) => {
        let path = $type::try_from($path).unwrap();
        let want = Path::new($path);

        assert_eq!(path_as_deref(&path), want);
        assert_eq!(path_as_ref(&path), want);
        assert_eq!(os_str_as_ref(&path), want.as_os_str());
        assert_eq!(format!("{path:?}"), format!("{want:?}"));
        assert_eq!(<$type as LogixType>::default_value(), None);
        assert_eq!(PathBuf::from(path).as_path(), want);
    };
}

#[test]
fn full_path_basics() {
    run_basic_tests!(FullPath, "/hello/world.txt");
}

#[test]
fn rel_path_basics1() {
    run_basic_tests!(RelPath, "hello/world.txt");
}

#[test]
fn rel_path_basics2() {
    run_basic_tests!(RelPath, "world.txt");
}

#[test]
fn name_only_path_basics() {
    run_basic_tests!(NameOnlyPath, "world.txt");
}

#[test]
fn valid_path_basics_full() {
    run_basic_tests!(ValidPath, "/hello/world.txt");
}

#[test]
fn valid_path_basics_rel() {
    run_basic_tests!(ValidPath, "hello/world.txt");
}

#[test]
fn valid_path_basics_name_only() {
    run_basic_tests!(ValidPath, "world.txt");
}
