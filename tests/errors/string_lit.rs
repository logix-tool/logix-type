use super::*;

fn invalid_utf8_basic<T: LogixType + fmt::Debug>() {
    let mut l = Loader::init().with_file(
        "test.logix",
        b"GenStruct {\n  aaa: 20\n  bbbb: \"aa\x8e\"\n}",
    );
    let e = l.parse_file::<GenStruct<T>>("test.logix");

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
fn invalid_utf8_basic_string() {
    invalid_utf8_basic::<String>();
}

#[test]
fn invalid_utf8_basic_str() {
    invalid_utf8_basic::<Str>();
}

#[test]
fn invalid_utf8_basic_pathbuf() {
    invalid_utf8_basic::<PathBuf>();
}

fn invalid_utf8_txt<T: LogixType + fmt::Debug>() {
    let mut l = Loader::init().with_file(
        "test.logix",
        b"GenStruct {\n  aaa: 20\n  bbbb: #txt\"aa\x8e\"#\n}",
    );
    let e = l.parse_file::<GenStruct<T>>("test.logix");

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

#[test]
fn invalid_utf8_txt_string() {
    invalid_utf8_txt::<String>();
}

#[test]
fn invalid_utf8_txt_str() {
    invalid_utf8_txt::<Str>();
}

#[test]
fn invalid_utf8_txt_pathbuf() {
    invalid_utf8_txt::<PathBuf>();
}

#[test]
fn unknown_tag() {
    let mut l = Loader::init().with_file(
        "test.logix",
        b"Struct {\n  aaa: 20\n  bbbb: #invalid\"aa\"#\n}",
    );
    let e = l.parse_struct("test.logix");

    assert_eq!(
        e,
        ParseError::TokenError {
            span: l.span("test.logix", 3, 8, 8),
            error: TokenError::UnknownStrTag("invalid".into()),
        }
    );

    assert_eq!(
        debval(&e),
        concat!(
            "\n",
            "error: Failed to parse input\n",
            "   ---> test.logix:3:8\n",
            "    |\n",
            "  2 |   aaa: 20\n",
            "  3 |   bbbb: #invalid\"aa\"#\n",
            "    |         ^^^^^^^^ unknown string tag `invalid`\n",
            "  4 | }\n",
        )
    );

    assert_eq!(
        disval(&e),
        "Failed to parse input, unknown string tag `invalid` in test.logix:3:8"
    );
}

#[test]
fn unterminated_eol() {
    let mut l = Loader::init().with_file("test.logix", b"Struct {\n  aaa: 20\n  bbbb: \"aa\n}");
    let e = l.parse_struct("test.logix");

    assert_eq!(
        e,
        ParseError::TokenError {
            span: l.span("test.logix", 3, 11, 1),
            error: TokenError::MissingStringTerminator,
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
            "  3 |   bbbb: \"aa\n",
            "    |            ^ unexpected end of the string, expected `\"`\n",
            "  4 | }\n",
        )
    );

    assert_eq!(
        disval(&e),
        "Failed to parse input, unexpected end of the string, expected `\"` in test.logix:3:11"
    );
}

#[test]
fn unterminated_eof() {
    let mut l = Loader::init().with_file("test.logix", b"Struct {\n  aaa: 20\n  bbbb: \"aa");
    let e = l.parse_struct("test.logix");

    assert_eq!(
        e,
        ParseError::TokenError {
            span: l.span("test.logix", 3, 11, 0),
            error: TokenError::MissingStringTerminator,
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
            "  3 |   bbbb: \"aa\n",
            "    |            ^ unexpected end of the string, expected `\"`\n",
        )
    );

    assert_eq!(
        disval(&e),
        "Failed to parse input, unexpected end of the string, expected `\"` in test.logix:3:11"
    );
}

#[test]
fn not_a_string() {
    let mut l = Loader::init().with_file("test.logix", b"Struct {\n  aaa: 20\n  bbbb: #txt(aa)\n}");
    let e = l.parse_struct("test.logix");

    assert_eq!(
        e,
        ParseError::TokenError {
            span: l.span("test.logix", 3, 8, 1),
            error: TokenError::UnexpectedChar('#'),
        }
    );

    assert_eq!(
        debval(&e),
        concat!(
            "\n",
            "error: Failed to parse input\n",
            "   ---> test.logix:3:8\n",
            "    |\n",
            "  2 |   aaa: 20\n",
            "  3 |   bbbb: #txt(aa)\n",
            "    |         ^ unexpected character '#'\n",
            "  4 | }\n",
        )
    );

    assert_eq!(
        disval(&e),
        "Failed to parse input, unexpected character '#' in test.logix:3:8"
    );
}

#[test]
fn invalid_tag() {
    let mut l = Loader::init().with_file("test.logix", b"Struct {\n  aaa: 20\n  bbbb: #%\"aa\"\n}");
    let e = l.parse_struct("test.logix");

    assert_eq!(
        e,
        ParseError::TokenError {
            span: l.span("test.logix", 3, 8, 1),
            error: TokenError::UnexpectedChar('#'),
        }
    );

    assert_eq!(
        debval(&e),
        concat!(
            "\n",
            "error: Failed to parse input\n",
            "   ---> test.logix:3:8\n",
            "    |\n",
            "  2 |   aaa: 20\n",
            "  3 |   bbbb: #%\"aa\"\n",
            "    |         ^ unexpected character '#'\n",
            "  4 | }\n",
        )
    );

    assert_eq!(
        disval(&e),
        "Failed to parse input, unexpected character '#' in test.logix:3:8"
    );
}

#[test]
fn unterminated_tag() {
    let mut l = Loader::init().with_file("test.logix", b"Struct {\n  aaa: 20\n  bbbb: #hello");
    let e = l.parse_struct("test.logix");

    assert_eq!(
        e,
        ParseError::TokenError {
            span: l.span("test.logix", 3, 8, 1),
            error: TokenError::UnexpectedChar('#'),
        }
    );

    assert_eq!(
        debval(&e),
        concat!(
            "\n",
            "error: Failed to parse input\n",
            "   ---> test.logix:3:8\n",
            "    |\n",
            "  2 |   aaa: 20\n",
            "  3 |   bbbb: #hello\n",
            "    |         ^ unexpected character '#'\n",
        )
    );

    assert_eq!(
        disval(&e),
        "Failed to parse input, unexpected character '#' in test.logix:3:8"
    );
}
