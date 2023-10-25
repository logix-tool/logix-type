use std::fmt;

use logix_type::{
    error::{ParseError, SourceSpan, Wanted},
    LogixLoader, LogixType,
    __private::{Delim, Token},
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

    fn parse_file<T: LogixType + std::fmt::Debug>(&mut self, name: &str) -> ParseError {
        println!("\n\nParsing {}\n", T::DESCRIPTOR.name);

        let e = self.loader.load_file::<T>(name).unwrap_err();

        println!("**** DISPLAY START ****\n{e}\n**** DISPLAY END ******\n");
        println!("*** DEBUG START ******\n{e:?}\n**** DEBUG END ********\n");
        e
    }

    fn parse_struct(&mut self, name: &str) -> ParseError {
        self.parse_file::<Struct>(name)
    }

    fn parse_tuple(&mut self, name: &str) -> ParseError {
        self.parse_file::<Tuple>(name)
    }
}

#[derive(LogixType, PartialEq, Debug)]
struct Struct {
    aaa: u32,
    bbbb: String,
}

#[derive(LogixType, PartialEq, Debug)]
struct Tuple(u32, String);

fn debval(s: &impl fmt::Debug) -> String {
    strip_ansi_escapes::strip_str(format!("{s:?}"))
}

fn disval(s: &impl fmt::Display) -> String {
    strip_ansi_escapes::strip_str(s.to_string())
}

#[test]
fn empty_file() {
    let mut l = Loader::init().with_file("test.logix", b"");
    let e = l.parse_struct("test.logix");

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
fn two_types() {
    let mut l = Loader::init().with_file(
        "test.logix",
        &b"Struct {\n  aaa: 60\n  bbbb: \"red\"\n}\n".repeat(2),
    );
    let e = l.parse_struct("test.logix");

    assert_eq!(
        e,
        ParseError::UnexpectedToken {
            span: l.span("test.logix", 5, 0, 6),
            while_parsing: "Struct",
            wanted: Wanted::Token(Token::Newline(true)),
            got_token: "identifier",
        }
    );

    assert_eq!(
        debval(&e),
        concat!(
            "\n",
            "error: Unexpected identifier while parsing `Struct`\n",
            "   ---> test.logix:5:0\n",
            "    |\n",
            "  4 | }\n",
            "  5 | Struct {\n",
            "    | ^^^^^^ Expected end of file\n",
            "  6 |   aaa: 60\n",
        )
    );

    assert_eq!(
        disval(&e),
        "Unexpected identifier while parsing `Struct`, expected end of file in test.logix:5:0"
    );
}

#[test]
fn unclosed_curly_brace() {
    let mut l = Loader::init().with_file("test.logix", b"Struct {");
    let e = l.parse_struct("test.logix");

    assert_eq!(
        e,
        ParseError::UnexpectedToken {
            span: l.span("test.logix", 1, 8, 0),
            while_parsing: "Struct",
            wanted: Wanted::Token(Token::Newline(false)),
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
            "    |         ^ Expected newline\n",
        )
    );

    assert_eq!(
        disval(&e),
        "Unexpected end of file while parsing `Struct`, expected newline in test.logix:1:8"
    );
}

#[test]
fn no_newline() {
    let mut l = Loader::init().with_file("test.logix", b"Struct {}");
    let e = l.parse_struct("test.logix");

    assert_eq!(
        e,
        ParseError::UnexpectedToken {
            span: l.span("test.logix", 1, 8, 1),
            while_parsing: "Struct",
            wanted: Wanted::Token(Token::Newline(false)),
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
            "    |         ^ Expected newline\n",
        )
    );

    assert_eq!(
        disval(&e),
        "Unexpected `}` while parsing `Struct`, expected newline in test.logix:1:8"
    );
}

#[test]
fn no_members() {
    let mut l = Loader::init().with_file("test.logix", b"Struct {\n}");
    let e = l.parse_struct("test.logix");

    assert_eq!(
        e,
        ParseError::MissingStructMember {
            span: l.span("test.logix", 2, 0, 1),
            type_name: "Struct",
            member: "aaa",
        }
    );

    assert_eq!(
        debval(&e),
        concat!(
            "\n",
            "error: Missing struct member while parsing `Struct`\n",
            "   ---> test.logix:2:0\n",
            "    |\n",
            "  1 | Struct {\n",
            "  2 | }\n",
            "    | ^ Expected `aaa`\n",
        )
    );

    assert_eq!(
        disval(&e),
        "Missing struct member `aaa` while parsing `Struct` in test.logix:2:0"
    );
}

#[test]
fn one_member_a() {
    let mut l = Loader::init().with_file("test.logix", b"Struct {\n  aaa: 10\n}");
    let e = l.parse_struct("test.logix");

    assert_eq!(
        e,
        ParseError::MissingStructMember {
            span: l.span("test.logix", 3, 0, 1),
            type_name: "Struct",
            member: "bbbb",
        }
    );

    assert_eq!(
        debval(&e),
        concat!(
            "\n",
            "error: Missing struct member while parsing `Struct`\n",
            "   ---> test.logix:3:0\n",
            "    |\n",
            "  2 |   aaa: 10\n",
            "  3 | }\n",
            "    | ^ Expected `bbbb`\n",
        )
    );

    assert_eq!(
        disval(&e),
        "Missing struct member `bbbb` while parsing `Struct` in test.logix:3:0"
    );
}

#[test]
fn one_member_b() {
    let mut l = Loader::init().with_file("test.logix", b"Struct {\n  bbbb: \"yo\"\n}");
    let e = l.parse_struct("test.logix");

    assert_eq!(
        e,
        ParseError::MissingStructMember {
            span: l.span("test.logix", 3, 0, 1),
            type_name: "Struct",
            member: "aaa",
        }
    );

    assert_eq!(
        debval(&e),
        concat!(
            "\n",
            "error: Missing struct member while parsing `Struct`\n",
            "   ---> test.logix:3:0\n",
            "    |\n",
            "  2 |   bbbb: \"yo\"\n",
            "  3 | }\n",
            "    | ^ Expected `aaa`\n",
        )
    );

    assert_eq!(
        disval(&e),
        "Missing struct member `aaa` while parsing `Struct` in test.logix:3:0"
    );
}

#[test]
fn duplicate_member() {
    let mut l = Loader::init().with_file("test.logix", b"Struct {\n  aaa: 20\n  aaa: 30\n}");
    let e = l.parse_struct("test.logix");

    assert_eq!(
        e,
        ParseError::DuplicateStructMember {
            span: l.span("test.logix", 3, 2, 3),
            type_name: "Struct",
            member: "aaa",
        }
    );

    assert_eq!(
        debval(&e),
        concat!(
            "\n",
            "error: Duplicate struct member while parsing `Struct`\n",
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
        "Duplicate struct member `aaa` while parsing `Struct` in test.logix:3:2"
    );
}

#[test]
fn one_member_tuple_want_comma() {
    let mut l = Loader::init().with_file("test.logix", b"Tuple(10)");
    let e = l.parse_tuple("test.logix");

    assert_eq!(
        e,
        ParseError::UnexpectedToken {
            span: l.span("test.logix", 1, 8, 1),
            while_parsing: "Tuple",
            got_token: "`)`",
            wanted: Wanted::Token(Token::Delim(Delim::Comma)),
        }
    );

    assert_eq!(
        debval(&e),
        concat!(
            "\n",
            "error: Unexpected `)` while parsing `Tuple`\n",
            "   ---> test.logix:1:8\n",
            "    |\n",
            "  1 | Tuple(10)\n",
            "    |         ^ Expected `,`\n",
        )
    );

    assert_eq!(
        disval(&e),
        "Unexpected `)` while parsing `Tuple`, expected `,` in test.logix:1:8"
    );
}

#[test]
fn one_member_tuple_want_litstr() {
    let mut l = Loader::init().with_file("test.logix", b"Tuple(10, )");
    let e = l.parse_tuple("test.logix");

    assert_eq!(
        e,
        ParseError::UnexpectedToken {
            span: l.span("test.logix", 1, 10, 1),
            while_parsing: "string",
            got_token: "`)`",
            wanted: Wanted::LitStr,
        }
    );

    assert_eq!(
        debval(&e),
        concat!(
            "\n",
            "error: Unexpected `)` while parsing `string`\n",
            "   ---> test.logix:1:10\n",
            "    |\n",
            "  1 | Tuple(10, )\n",
            "    |           ^ Expected string\n",
        )
    );

    assert_eq!(
        disval(&e),
        "Unexpected `)` while parsing `string`, expected string in test.logix:1:10"
    );
}
