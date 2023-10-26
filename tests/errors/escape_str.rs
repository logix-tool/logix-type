use super::*;

fn escape_str(
    esc_str: &str,
    col_off: usize,
    col_len: usize,
    underline: &str,
    error: EscStrError,
    err_str: &str,
) {
    let col = 8 + col_off;
    let mut l = Loader::init().with_file(
        "test.logix",
        format!("Struct {{\n  aaa: 10\n  bbbb: {esc_str}\n}}").as_bytes(),
    );
    let e = l.parse_struct("test.logix");

    assert_eq!(
        e,
        ParseError::StrEscError {
            span: l.span("test.logix", 3, col, col_len),
            error,
        }
    );

    assert_eq!(
        debval(&e),
        [
            format!("\n"),
            format!("error: Failed to parse escaped string\n"),
            format!("   ---> test.logix:3:{col}\n"),
            format!("    |\n"),
            format!("  2 |   aaa: 10\n"),
            format!("  3 |   bbbb: {esc_str}\n"),
            format!("    |         {underline} {err_str}\n"),
            format!("  4 | }}\n"),
        ]
        .into_iter()
        .collect::<String>(),
    );

    assert_eq!(
        disval(&e),
        format!("Failed to parse string, {err_str} in test.logix:3:{col}")
    );
}

#[test]
fn escape_hex() {
    escape_str(
        r#""\xf""#,
        1,
        3,
        " ^^^",
        EscStrError::TruncatedHex,
        "got truncated hex escape code",
    );
    escape_str(
        r#""\xfk""#,
        1,
        4,
        " ^^^^",
        EscStrError::InvalidHex,
        "got invalid hex escape code",
    );
}

#[test]
fn escape_unicode() {
    escape_str(
        r#""\u{z}""#,
        1,
        5,
        " ^^^^^",
        EscStrError::InvalidUnicodeHex,
        "got invalid unicode hex escape code",
    );

    escape_str(
        r#""\u{}""#,
        1,
        4,
        " ^^^^",
        EscStrError::InvalidUnicodeHex,
        "got invalid unicode hex escape code",
    );

    escape_str(
        r#""\u{1234z}""#,
        1,
        9,
        " ^^^^^^^^^",
        EscStrError::InvalidUnicodeHex,
        "got invalid unicode hex escape code",
    );

    escape_str(
        r#""\u{ffff0000}""#,
        1,
        12,
        " ^^^^^^^^^^^^",
        EscStrError::InvalidUnicodePoint(0xffff0000),
        "the code point U+ffff0000 is invalid",
    );

    escape_str(
        r#""\u{ffff00000}""#,
        1,
        11,
        " ^^^^^^^^^^^",
        EscStrError::InvalidUnicodeMissingEndBrace,
        "got invalid unicode escape, expected `}`",
    );

    escape_str(
        r#""\u{fff""#,
        1,
        6,
        " ^^^^^^",
        EscStrError::InvalidUnicodeMissingEndBrace,
        "got invalid unicode escape, expected `}`",
    );

    escape_str(
        r#""\u10""#,
        1,
        3,
        " ^^^",
        EscStrError::InvalidUnicodeMissingStartBrace,
        "got invalid unicode escape, expected `{`",
    );
}

#[test]
fn escape_char() {
    escape_str(
        r#""\k""#,
        1,
        2,
        " ^^",
        EscStrError::InvalidEscapeChar('k'),
        "got invalid escape character 'k'",
    );
}
