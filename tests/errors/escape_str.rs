use super::*;

fn escape_str<T: LogixType + fmt::Debug>(
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
        format!("GenStruct {{\n  aaa: 10\n  bbbb: {esc_str}\n}}").as_bytes(),
    );
    let e = l.parse_file::<GenStruct<T>>("test.logix");

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

fn escape_hex<T: LogixType + fmt::Debug>() {
    escape_str::<T>(
        r#""\xf""#,
        1,
        3,
        " ^^^",
        EscStrError::TruncatedHex,
        "got truncated hex escape code",
    );
    escape_str::<T>(
        r#""\xfk""#,
        1,
        4,
        " ^^^^",
        EscStrError::InvalidHex,
        "got invalid hex escape code",
    );
}

#[test]
fn escape_hex_string() {
    escape_hex::<String>();
}

#[test]
fn escape_hex_str() {
    escape_hex::<Str>();
}

#[test]
fn escape_hex_pathbuf() {
    escape_hex::<PathBuf>();
}

fn escape_unicode<T: LogixType + fmt::Debug>() {
    escape_str::<T>(
        r#""\u{z}""#,
        1,
        5,
        " ^^^^^",
        EscStrError::InvalidUnicodeHex,
        "got invalid unicode hex escape code",
    );

    escape_str::<T>(
        r#""\u{}""#,
        1,
        4,
        " ^^^^",
        EscStrError::InvalidUnicodeHex,
        "got invalid unicode hex escape code",
    );

    escape_str::<T>(
        r#""\u{1234z}""#,
        1,
        9,
        " ^^^^^^^^^",
        EscStrError::InvalidUnicodeHex,
        "got invalid unicode hex escape code",
    );

    escape_str::<T>(
        r#""\u{ffff0000}""#,
        1,
        12,
        " ^^^^^^^^^^^^",
        EscStrError::InvalidUnicodePoint(0xffff0000),
        "the code point U+ffff0000 is invalid",
    );

    escape_str::<T>(
        r#""\u{ffff00000}""#,
        1,
        11,
        " ^^^^^^^^^^^",
        EscStrError::InvalidUnicodeMissingEndBrace,
        "got invalid unicode escape, expected `}`",
    );

    escape_str::<T>(
        r#""\u{fff""#,
        1,
        6,
        " ^^^^^^",
        EscStrError::InvalidUnicodeMissingEndBrace,
        "got invalid unicode escape, expected `}`",
    );

    escape_str::<T>(
        r#""\u10""#,
        1,
        3,
        " ^^^",
        EscStrError::InvalidUnicodeMissingStartBrace,
        "got invalid unicode escape, expected `{`",
    );
}

#[test]
fn escape_unicode_string() {
    escape_unicode::<String>();
}

#[test]
fn escape_unicode_str() {
    escape_unicode::<Str>();
}

#[test]
fn escape_unicode_pathbuf() {
    escape_unicode::<PathBuf>();
}

fn escape_char<T: LogixType + fmt::Debug>() {
    escape_str::<T>(
        r#""\k""#,
        1,
        2,
        " ^^",
        EscStrError::InvalidEscapeChar('k'),
        "got invalid escape character 'k'",
    );
}

#[test]
fn escape_char_string() {
    escape_char::<String>();
}

#[test]
fn escape_char_str() {
    escape_char::<Str>();
}

#[test]
fn escape_char_pathbuf() {
    escape_char::<PathBuf>();
}
