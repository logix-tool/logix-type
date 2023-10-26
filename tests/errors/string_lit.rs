use logix_type::error::TokenError;

use super::*;

#[test]
fn invalid_utf8_basic() {
    let mut l =
        Loader::init().with_file("test.logix", b"Struct {\n  aaa: 20\n  bbbb: \"aa\x8e\"\n}");
    let e = l.parse_struct("test.logix");

    assert_eq!(
        e,
        ParseError::TokenError {
            span: l.span("test.logix", 3, 11, 1),
            error: TokenError::LitStrNotUtf8,
        }
    );

    assert_eq!(
        debval(&e),
        concat!(
            "\n",
            "error: Failed to parse input\n",
            "   ---> test.logix:3:11\n",
            "    |\n",
            "  2 |   aaa: 20\n",
            "  3 |   bbbb: \"aa\u{fffd}\"\n",
            "    |            ^ invalid utf-8 sequence\n",
            "  4 | }\n",
        )
    );

    assert_eq!(
        disval(&e),
        "Failed to parse input, invalid utf-8 sequence in test.logix:3:11"
    );
}

#[test]
fn invalid_utf8_txt() {
    let mut l = Loader::init().with_file(
        "test.logix",
        b"Struct {\n  aaa: 20\n  bbbb: #txt\"aa\x8e\"#\n}",
    );
    let e = l.parse_struct("test.logix");

    assert_eq!(
        e,
        ParseError::TokenError {
            span: l.span("test.logix", 3, 16, 1),
            error: TokenError::LitStrNotUtf8,
        }
    );

    assert_eq!(
        debval(&e),
        concat!(
            "\n",
            "error: Failed to parse input\n",
            "   ---> test.logix:3:16\n",
            "    |\n",
            "  2 |   aaa: 20\n",
            "  3 |   bbbb: #txt\"aa\u{fffd}\"#\n",
            "    |                 ^ invalid utf-8 sequence\n",
            "  4 | }\n",
        )
    );

    assert_eq!(
        disval(&e),
        "Failed to parse input, invalid utf-8 sequence in test.logix:3:16"
    );
}
