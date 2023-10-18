use logix_type::{error::Result, LogixLoader, Map, Str};
use logix_vfs::RelFs;

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
    type_str: Str,
    type_string: String,
    type_map_int: Map<i32>,
    type_map_str: Map<String>,
    type_map_named_struct: Map<NamedNode>,
    type_map_unnamed_struct: Map<UnnamedNode>,
}

#[derive(logix_type::LogixType, PartialEq, Debug)]
struct NamedNode {
    s: Str,
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
        type_map_int: vec![
            (Str::new("key1"), 8),
            (Str::new("key2"), -12),
            (Str::new("key3"), 0),
        ]
        .into_iter()
        .collect(),
        type_map_str: vec![
            (Str::new("key4"), "Hi, space!".to_string()),
            (Str::new("key5"), "Yo, multiverse!".to_string()),
            (Str::new("key6"), "Sup, dimension!".to_string()),
        ]
        .into_iter()
        .collect(),
        type_map_named_struct: vec![
            (
                Str::new("key7"),
                NamedNode {
                    s: Str::new("Ahoy, planet!"),
                    v: 78,
                },
            ),
            (
                Str::new("key8"),
                NamedNode {
                    s: Str::new("Namaste, cosmos!"),
                    v: -689,
                },
            ),
            (
                Str::new("key9"),
                NamedNode {
                    s: Str::new("G'day, cluster!"),
                    v: 597,
                },
            ),
        ]
        .into_iter()
        .collect(),
        type_map_unnamed_struct: vec![
            (
                Str::new("key10"),
                UnnamedNode("Howdy-do, domain!".into(), 409),
            ),
            (
                Str::new("key11"),
                UnnamedNode("Hi-ho, space-time!".into(), 632),
            ),
            (
                Str::new("key12"),
                UnnamedNode("Bonjour, infinity!".into(), 2471),
            ),
        ]
        .into_iter()
        .collect(),
    }
}

#[test]
fn load() -> Result<()> {
    let expected = expected_root();
    let mut loader = LogixLoader::new(RelFs::new("tests/include"));
    let got: Root = loader.load_file("all-types.logix")?;
    assert_eq!(expected, got);
    Ok(())
}
