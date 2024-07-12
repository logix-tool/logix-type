use std::{path::PathBuf, rc::Rc, sync::Arc};

use logix_type::{
    error::Result,
    types::{Data, ExecutablePath, FullPath, Map, NameOnlyPath, RelPath, ShortStr, ValidPath},
    LogixLoader, LogixType,
};
use logix_vfs::{LogixVfs, RelFs};

static ALL_TYPES_FILE: &str = include_str!("include/all-types.logix");

#[derive(logix_type::LogixType, PartialEq, Debug)]
enum Enum {
    Unit,
    Unnamed(u32),
    Named { v: u32 },
}

#[derive(logix_type::LogixType, PartialEq, Debug)]
struct Unit;

#[derive(logix_type::LogixType, PartialEq, Debug)]
struct Root {
    type_i8: i8,
    type_u8: u8,
    type_i16: i16,
    type_u16: u16,
    type_i32: i32,
    type_u32: u32,
    type_i64: i64,
    type_u64: u64,
    type_str: ShortStr,
    type_string: String,
    type_path: PathBuf,
    type_enum: Map<Enum>,
    type_unit: Unit,
    type_map_int: Map<i32>,
    type_map_str: Map<String>,
    type_map_named_struct: Map<NamedNode>,
    type_map_unnamed_struct: Map<UnnamedNode>,
    txt_str: String,
    esc_str: String,
    very_long_escape1: String,
    very_long_escape2: String,
    tagged_strings: Map<String>,
    included_string: String,
    data_by_path_str: Data<String>,
    data_inline_str: Data<String>,
    opt_i32_none: Option<i32>,
    opt_i32_set: Option<i32>,
    full_path: FullPath,
    rel_path1: RelPath,
    rel_path2: RelPath,
    name_only_path: NameOnlyPath,
    valid_path_full: ValidPath,
    valid_path_rel: ValidPath,
    valid_path_name: ValidPath,
    executable1: ExecutablePath,
    executable2: ExecutablePath,
    dyn_array: Vec<u32>,
    fixed_array: [u32; 3],
    arc_map: Map<u32, Arc<str>>,
    rc_map: Map<u32, Rc<str>>,
    box_map: Map<u32, Box<str>>,
}

#[derive(logix_type::LogixType, PartialEq, Debug)]
struct NamedNode {
    s: ShortStr,
    v: i32,
}

#[derive(logix_type::LogixType, PartialEq, Debug)]
struct UnnamedNode(String, u32);

fn expected_root() -> Root {
    Root {
        type_i8: -42,
        type_u8: 42,
        type_i16: -1337,
        type_u16: 1337,
        type_i32: -69696,
        type_u32: 69696,
        type_i64: -7202218937,
        type_u64: 7202218937,
        type_str: "Hello, world!".into(),
        type_string: "Howdy, universe!".into(),
        type_path: "hello.txt".into(),
        type_enum: [
            (ShortStr::from("unit"), Enum::Unit),
            (ShortStr::from("unnamed"), Enum::Unnamed(10)),
            (ShortStr::from("named"), Enum::Named { v: 20 }),
        ].into(),
        type_unit: Unit,
        type_map_int: [
            (ShortStr::from("key1"), 8),
            (ShortStr::from("key2"), -12),
            (ShortStr::from("key3"), 0),
        ].into(),
        type_map_str: [
            (ShortStr::from("key4"), "Hi, space!".to_string()),
            (ShortStr::from("key5"), "Yo, multiverse!".to_string()),
            (ShortStr::from("key6"), "Sup, dimension!".to_string()),
        ].into(),
        type_map_named_struct: [
            (
                ShortStr::from("key7"),
                NamedNode {
                    s: ShortStr::from("Ahoy, planet!"),
                    v: 78,
                },
            ),
            (
                ShortStr::from("key8"),
                NamedNode {
                    s: ShortStr::from("Namaste, cosmos!"),
                    v: -689,
                },
            ),
            (
                ShortStr::from("key9"),
                NamedNode {
                    s: ShortStr::from("G'day, cluster!"),
                    v: 597,
                },
            ),
        ].into(),
        type_map_unnamed_struct: [
            (
                ShortStr::from("key10"),
                UnnamedNode("Howdy-do, domain!".into(), 409),
            ),
            (
                ShortStr::from("key11"),
                UnnamedNode("Hi-ho, space-time!".into(), 632),
            ),
            (
                ShortStr::from("key12"),
                UnnamedNode("Bonjour, infinity!".into(), 2471),
            ),
        ].into(),
        txt_str: concat!(
            "Good day there mister!",
            "\n",
            "This text is quite long, and contains more than two paragraphs. It follows similar wrapping rules as",
            "markdown, so a single line-break is only used to make the paragraph easier to read.",
            "\n",
            "The prefix is also removed and so is the first and last newline",
        ).into(),
        esc_str: "LF: \n, Tab: \t, CR: \r, Unicode: \u{a4}, Backslash: \\, Quote: \", Hex: \x20".into(),
        very_long_escape1: "it works".into(),
        very_long_escape2: "it \"################################## works".into(),
        tagged_strings: [
            (ShortStr::from("raw"), "this is \\n raw".to_owned()),
            (ShortStr::from("esc"), "this is \n esc".to_owned()),
            (ShortStr::from("txt"), "this is \\n txt".to_owned()),
        ].into(),
        included_string: "Hello, this is a plain text file\n".into(),
        data_by_path_str: Data::ByPath("text-file.txt".into()),
        data_inline_str: Data::Inline("inline string".into()),
        opt_i32_none: None,
        opt_i32_set: Some(99),
        full_path: "/hello/world.txt".try_into().unwrap(),
        rel_path1: "hello/world.txt".try_into().unwrap(),
        rel_path2: "world.txt".try_into().unwrap(),
        name_only_path: "world.txt".try_into().unwrap(),
        valid_path_full: "/hello/world.txt".try_into().unwrap(),
        valid_path_rel: "hello/world.txt".try_into().unwrap(),
        valid_path_name: "world.txt".try_into().unwrap(),
        executable1: "logix".try_into().unwrap(),
        executable2: "/usr/bin/logix".try_into().unwrap(),
        dyn_array: vec![1, 1, 2, 2, 3, 3],
        fixed_array: [1, 2, 3],
        arc_map: [(Arc::from("a"), 16)].into(),
        rc_map: [(Rc::from("b"), 54)].into(),
        box_map: [(Box::from("c"), 32)].into(),
    }
}

fn temp_loader() -> (tempfile::TempDir, LogixLoader<impl LogixVfs>) {
    let dir = tempfile::tempdir().unwrap();
    let fs = RelFs::new(dir.path());
    std::fs::write(
        dir.path().join("text-file.txt"),
        include_str!("include/text-file.txt"),
    )
    .unwrap();
    std::fs::write(
        dir.path().join("all-files.logix"),
        include_str!("include/all-types.logix"),
    )
    .unwrap();

    (dir, LogixLoader::new(fs))
}

fn load_and_compare(loader: &mut LogixLoader<impl LogixVfs>) -> Result<()> {
    let expected = expected_root();
    let Root {
        type_i8,
        type_u8,
        type_i16,
        type_u16,
        type_i32,
        type_u32,
        type_i64,
        type_u64,
        type_str,
        type_string,
        type_path,
        type_enum,
        type_unit,
        type_map_int,
        type_map_str,
        type_map_named_struct,
        type_map_unnamed_struct,
        txt_str,
        esc_str,
        very_long_escape1,
        very_long_escape2,
        tagged_strings,
        included_string,
        data_by_path_str,
        data_inline_str,
        opt_i32_none,
        opt_i32_set,
        full_path,
        rel_path1,
        rel_path2,
        name_only_path,
        valid_path_full,
        valid_path_rel,
        valid_path_name,
        executable1,
        executable2,
        dyn_array,
        fixed_array,
        arc_map,
        rc_map,
        box_map,
    } = loader.load_file("all-types.logix")?;
    assert_eq!(type_i8, expected.type_i8);
    assert_eq!(type_u8, expected.type_u8);
    assert_eq!(type_i16, expected.type_i16);
    assert_eq!(type_u16, expected.type_u16);
    assert_eq!(type_i32, expected.type_i32);
    assert_eq!(type_u32, expected.type_u32);
    assert_eq!(type_i64, expected.type_i64);
    assert_eq!(type_u64, expected.type_u64);
    assert_eq!(type_str, expected.type_str);
    assert_eq!(type_unit, expected.type_unit);
    assert_eq!(type_enum, expected.type_enum);
    assert_eq!(type_string, expected.type_string);
    assert_eq!(type_path, expected.type_path);
    assert_eq!(type_map_int, expected.type_map_int);
    assert_eq!(type_map_str, expected.type_map_str);
    assert_eq!(type_map_named_struct, expected.type_map_named_struct);
    assert_eq!(type_map_unnamed_struct, expected.type_map_unnamed_struct);

    {
        let mut exp_it = expected.txt_str.split_inclusive('\n').peekable();
        let mut got_it = txt_str.split_inclusive('\n').peekable();
        let mut ln = 0;
        while exp_it.peek().is_some() || got_it.peek().is_some() {
            ln += 1;
            assert_eq!(exp_it.next(), got_it.next(), "Mismatch on line {ln}")
        }
    }

    assert_eq!(esc_str, expected.esc_str);
    assert_eq!(very_long_escape1, expected.very_long_escape1);
    assert_eq!(very_long_escape2, expected.very_long_escape2);

    {
        let mut exp_it = expected.tagged_strings.iter().peekable();
        let mut got_it = tagged_strings.iter().peekable();
        while exp_it.peek().is_some() || got_it.peek().is_some() {
            assert_eq!(exp_it.next(), got_it.next())
        }
    }

    assert_eq!(included_string, expected.included_string);
    assert_eq!(data_by_path_str, expected.data_by_path_str);
    assert_eq!(data_inline_str, expected.data_inline_str);

    assert_eq!(opt_i32_none, expected.opt_i32_none);
    assert_eq!(opt_i32_set, expected.opt_i32_set);

    assert_eq!(full_path, expected.full_path);
    assert_eq!(rel_path1, expected.rel_path1);
    assert_eq!(rel_path2, expected.rel_path2);
    assert_eq!(name_only_path, expected.name_only_path);
    assert_eq!(valid_path_full, expected.valid_path_full);
    assert_eq!(valid_path_rel, expected.valid_path_rel);
    assert_eq!(valid_path_name, expected.valid_path_name);

    assert_eq!(executable1, expected.executable1);
    assert_eq!(executable2, expected.executable2);

    assert_eq!(dyn_array, expected.dyn_array);
    assert_eq!(fixed_array, expected.fixed_array);

    assert_eq!(arc_map, expected.arc_map);
    assert_eq!(rc_map, expected.rc_map);
    assert_eq!(box_map, expected.box_map);

    Ok(())
}

#[test]
fn just_load() -> Result<()> {
    Root::descriptor(); // Try to load the descriptor
    load_and_compare(&mut LogixLoader::new(RelFs::new("tests/include")))
}

#[test]
fn starting_line_comment() -> Result<()> {
    let (dir, mut l) = temp_loader();
    std::fs::write(
        dir.path().join("all-types.logix"),
        format!("// Start with a line-comment\n{ALL_TYPES_FILE}"),
    )
    .unwrap();
    load_and_compare(&mut l)
}

#[test]
fn terminating_line_comment() -> Result<()> {
    let (dir, mut l) = temp_loader();
    std::fs::write(
        dir.path().join("all-types.logix"),
        format!("{ALL_TYPES_FILE} // End in a line-comment"),
    )
    .unwrap();
    load_and_compare(&mut l)
}

#[test]
fn starting_multiline_comment() -> Result<()> {
    let (dir, mut l) = temp_loader();
    std::fs::write(
        dir.path().join("all-types.logix"),
        format!("/*\nStart with a multi-line comment\n*/ {ALL_TYPES_FILE}"),
    )
    .unwrap();
    load_and_compare(&mut l)
}

#[test]
fn terminating_multiline_comment() -> Result<()> {
    let (dir, mut l) = temp_loader();
    std::fs::write(
        dir.path().join("all-types.logix"),
        format!("{ALL_TYPES_FILE} /*\nEnd in a multi-line comment\n*/"),
    )
    .unwrap();
    load_and_compare(&mut l)
}

#[test]
fn starting_eols() -> Result<()> {
    let (dir, mut l) = temp_loader();
    std::fs::write(
        dir.path().join("all-types.logix"),
        format!("\n\n\n\n\n\n\n{ALL_TYPES_FILE}"),
    )
    .unwrap();
    load_and_compare(&mut l)
}

#[test]
fn terminating_eols() -> Result<()> {
    let (dir, mut l) = temp_loader();
    std::fs::write(
        dir.path().join("all-types.logix"),
        format!("{ALL_TYPES_FILE}\n\n\n\n\n\n\n"),
    )
    .unwrap();
    load_and_compare(&mut l)
}

#[test]
fn starting_spaces() -> Result<()> {
    let (dir, mut l) = temp_loader();
    std::fs::write(
        dir.path().join("all-types.logix"),
        format!("   \n  \n \t\t  \n  \n\n\n\n{ALL_TYPES_FILE}"),
    )
    .unwrap();
    load_and_compare(&mut l)
}

#[test]
fn terminating_spaces() -> Result<()> {
    let (dir, mut l) = temp_loader();
    std::fs::write(
        dir.path().join("all-types.logix"),
        format!("{ALL_TYPES_FILE}   \n  \n \t\t  \n  \n\n\n\n"),
    )
    .unwrap();
    load_and_compare(&mut l)
}
