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
    a: u32,
    b: String,
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
            "error: Unexpected end of file while parsing Struct\n",
            "   ---> test.logix:1:0\n",
            "    |\n",
        )
    );

    assert_eq!(
        disval(&e),
        "Unexpected end of file while parsing Struct, expected `Struct`"
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
                Token::Ident("a"),
                Token::Ident("b")
            ]),
            got_token: "end of file",
        }
    );

    assert_eq!(
        debval(&e),
        concat!(
            "\n",
            "error: Unexpected end of file while parsing Struct\n",
            "   ---> test.logix:1:8\n",
            "    |\n",
            "  1 | Struct {\n",
            "    |          Expected one of `}`, `a`, or `b`\n",
        )
    );

    assert_eq!(
        disval(&e),
        "Unexpected end of file while parsing Struct, expected one of `}`, `a`, or `b`"
    );
}
