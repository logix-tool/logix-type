use std::fmt;

use logix_type::{
    error::{ParseError, SourceSpan, Wanted},
    LogixLoader, LogixType,
    __private::{Brace, Token},
};
use logix_vfs::RelFs;

struct Loader {
    root: tempfile::TempDir,
    loader: LogixLoader<RelFs>,
}

impl Loader {
    fn init() -> Self {
        let root = tempfile::tempdir().unwrap();
        let loader = LogixLoader::new(RelFs::new(root.path()));

        Self { root, loader }
    }

    fn with_file(self, path: &str, data: &[u8]) -> Self {
        let path = self.root.path().join(path);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, data).unwrap();
        self
    }

    fn span(&self, path: &str, line: usize, col: usize, len: usize) -> SourceSpan {
        SourceSpan::new_for_test(&self.loader, path, line, col, len)
    }
}

#[derive(LogixType, PartialEq, Debug)]
struct Struct {
    aaa: u32,
    bbbb: String,
}

fn debval(s: &impl fmt::Debug) -> String {
    strip_ansi_escapes::strip_str(format!("{s:?}"))
}

fn disval(s: &impl fmt::Display) -> String {
    strip_ansi_escapes::strip_str(s.to_string())
}

#[test]
fn empty_file() {
    let mut l = Loader::init().with_file("test.logix", b"");
    let e = l.loader.load_file::<Struct>("test.logix").unwrap_err();

    assert_eq!(
        e,
        ParseError::UnexpectedToken {
            span: l.span("test.logix", 1, 0, 0),
            while_parsing: "Struct",
            wanted: Wanted::Token(Token::Ident("Struct")),
            got_token: "end of file",
        }
    );

    assert_eq!(
        debval(&e),
        concat!(
            "\n",
            "error: Unexpected end of file while parsing `Struct`\n",
            "   ---> test.logix:1:0\n",
            "    |\n",
        )
    );

    assert_eq!(
        disval(&e),
        "Unexpected end of file while parsing `Struct`, expected `Struct` in test.logix:1:0"
    );
}

#[test]
fn unclosed_curly_brace() {
    let mut l = Loader::init().with_file("test.logix", b"Struct {");
    let e = l.loader.load_file::<Struct>("test.logix").unwrap_err();

    assert_eq!(
        e,
        ParseError::UnexpectedToken {
            span: l.span("test.logix", 1, 8, 0),
            while_parsing: "Struct",
            wanted: Wanted::Tokens(&[
                Token::BraceEnd(Brace::Curly),
                Token::Ident("aaa"),
                Token::Ident("bbbb")
            ]),
            got_token: "end of file",
        }
    );

    assert_eq!(
        debval(&e),
        concat!(
            "\n",
            "error: Unexpected end of file while parsing `Struct`\n",
            "   ---> test.logix:1:8\n",
            "    |\n",
            "  1 | Struct {\n",
            "    |          Expected one of `}`, `aaa`, or `bbbb`\n",
        )
    );

    assert_eq!(
        disval(&e),
        "Unexpected end of file while parsing `Struct`, expected one of `}`, `aaa`, or `bbbb` in test.logix:1:8"
    );
}

#[test]
fn no_newline() {
    let mut l = Loader::init().with_file("test.logix", b"Struct {}");
    let e = l.loader.load_file::<Struct>("test.logix").unwrap_err();

    assert_eq!(
        e,
        ParseError::UnexpectedToken {
            span: l.span("test.logix", 1, 8, 1),
            while_parsing: "Struct",
            wanted: Wanted::Token(Token::Newline),
            got_token: "`}`",
        }
    );

    assert_eq!(
        debval(&e),
        concat!(
            "\n",
            "error: Unexpected `}` while parsing `Struct`\n",
            "   ---> test.logix:1:8\n",
            "    |\n",
            "  1 | Struct {}\n",
            "    |         ^ Expected `<newline>`\n",
        )
    );

    assert_eq!(
        disval(&e),
        "Unexpected `}` while parsing `Struct`, expected `<newline>` in test.logix:1:8"
    );
}

#[test]
fn no_members() {
    let mut l = Loader::init().with_file("test.logix", b"Struct {\n}");
    let e = l.loader.load_file::<Struct>("test.logix").unwrap_err();

    assert_eq!(
        e,
        ParseError::MissingMember {
            span: l.span("test.logix", 2, 0, 1),
            type_name: "Struct",
            member: "aaa",
        }
    );

    assert_eq!(
        debval(&e),
        concat!(
            "\n",
            "error: Missing member while parsing `Struct`\n",
            "   ---> test.logix:2:0\n",
            "    |\n",
            "  1 | Struct {\n",
            "  2 | }\n",
            "    | ^ Expected `aaa`\n",
        )
    );

    assert_eq!(
        disval(&e),
        "Missing member `aaa` while parsing `Struct` in test.logix:2:0"
    );
}

#[test]
fn one_member_a() {
    let mut l = Loader::init().with_file("test.logix", b"Struct {\n  aaa: 10\n}");
    let e = l.loader.load_file::<Struct>("test.logix").unwrap_err();

    assert_eq!(
        e,
        ParseError::MissingMember {
            span: l.span("test.logix", 3, 0, 1),
            type_name: "Struct",
            member: "bbbb",
        }
    );

    assert_eq!(
        debval(&e),
        concat!(
            "\n",
            "error: Missing member while parsing `Struct`\n",
            "   ---> test.logix:3:0\n",
            "    |\n",
            "  2 |   aaa: 10\n",
            "  3 | }\n",
            "    | ^ Expected `bbbb`\n",
        )
    );

    assert_eq!(
        disval(&e),
        "Missing member `bbbb` while parsing `Struct` in test.logix:3:0"
    );
}

#[test]
fn one_member_b() {
    let mut l = Loader::init().with_file("test.logix", b"Struct {\n  bbbb: \"yo\"\n}");
    let e = l.loader.load_file::<Struct>("test.logix").unwrap_err();

    assert_eq!(
        e,
        ParseError::MissingMember {
            span: l.span("test.logix", 3, 0, 1),
            type_name: "Struct",
            member: "aaa",
        }
    );

    assert_eq!(
        debval(&e),
        concat!(
            "\n",
            "error: Missing member while parsing `Struct`\n",
            "   ---> test.logix:3:0\n",
            "    |\n",
            "  2 |   bbbb: \"yo\"\n",
            "  3 | }\n",
            "    | ^ Expected `aaa`\n",
        )
    );

    assert_eq!(
        disval(&e),
        "Missing member `aaa` while parsing `Struct` in test.logix:3:0"
    );
}

#[test]
fn duplicate_member() {
    let mut l = Loader::init().with_file("test.logix", b"Struct {\n  aaa: 20\n  aaa: 30\n}");
    let e = l.loader.load_file::<Struct>("test.logix").unwrap_err();

    assert_eq!(
        e,
        ParseError::DuplicateMember {
            span: l.span("test.logix", 3, 2, 3),
            type_name: "Struct",
            member: "aaa",
        }
    );

    assert_eq!(
        debval(&e),
        concat!(
            "\n",
            "error: Duplicate member while parsing `Struct`\n",
            "   ---> test.logix:3:2\n",
            "    |\n",
            "  2 |   aaa: 20\n",
            "  3 |   aaa: 30\n",
            "    |   ^^^ Unexpected `aaa`\n",
            "  4 | }\n",
        )
    );

    assert_eq!(
        disval(&e),
        "Duplicate member `aaa` while parsing `Struct` in test.logix:3:2"
    );
}
