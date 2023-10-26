use logix_type::error::TokenError;

use super::*;

#[test]
fn stray_slash() {
    let mut l = Loader::init().with_file(
        "test.logix",
        b"Struct {\n  aaa: 20\n  bbbb: \"aa\"\n} / hello",
    );
    let e = l.parse_struct("test.logix");

    assert_eq!(
        e,
        ParseError::TokenError {
            span: l.span("test.logix", 4, 2, 1),
            error: TokenError::UnexpectedChar('/'),
        }
    );

    assert_eq!(
        debval(&e),
        concat!(
            "\n",
            "error: Failed to parse input\n",
            "   ---> test.logix:4:2\n",
            "    |\n",
            "  3 |   bbbb: \"aa\"\n",
            "  4 | } / hello\n",
            "    |   ^ unexpected character '/'\n",
        )
    );

    assert_eq!(
        disval(&e),
        "Failed to parse input, unexpected character '/' in test.logix:4:2"
    );
}

#[test]
fn not_terminated() {
    let mut l = Loader::init().with_file(
        "test.logix",
        b"Struct {\n  aaa: 20\n  bbbb: \"aa\"\n} /* hello",
    );
    let e = l.parse_struct("test.logix");

    assert_eq!(
        e,
        ParseError::TokenError {
            span: l.span("test.logix", 4, 10, 1),
            error: TokenError::MissingCommentTerminator,
        }
    );

    assert_eq!(
        debval(&e),
        concat!(
            "\n",
            "error: Failed to parse input\n",
            "   ---> test.logix:4:10\n",
            "    |\n",
            "  3 |   bbbb: \"aa\"\n",
            "  4 | } /* hello\n",
            "    |           ^ unexpected end of file, expected `*/`\n",
        )
    );

    assert_eq!(
        disval(&e),
        "Failed to parse input, unexpected end of file, expected `*/` in test.logix:4:10"
    );
}
