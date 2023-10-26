use super::*;

fn stray_token(token: &str, token_str: &'static str) {
    let mut l = Loader::init().with_file(
        "test.logix",
        format!("Struct {{\n  aaa: 10\n  bbbb: \"hola\"\n}}\n\n{token}").as_bytes(),
    );
    let e = l.parse_struct("test.logix");

    assert_eq!(
        e,
        ParseError::UnexpectedToken {
            span: l.span("test.logix", 6, 0, token.len()),
            while_parsing: "Struct",
            wanted: Wanted::Token(Token::Newline(true)),
            got_token: token_str,
        }
    );

    assert_eq!(
        debval(&e),
        [
            format!("\n"),
            format!("error: Unexpected {token_str} while parsing `Struct`\n"),
            format!("   ---> test.logix:6:0\n"),
            format!("    |\n"),
            format!("  5 | \n"),
            format!("  6 | {token}\n"),
            format!("    | {} Expected end of file\n", "^".repeat(token.len())),
        ]
        .into_iter()
        .collect::<String>(),
    );

    assert_eq!(
        disval(&e),
        format!(
            "Unexpected {token_str} while parsing `Struct`, expected end of file in test.logix:6:0"
        )
    );
}

#[test]
fn stray_curly_brace() {
    stray_token("{", "`{`");
    stray_token("}", "`}`");
}

#[test]
fn stray_parens() {
    stray_token("(", "`(`");
    stray_token(")", "`)`");
}

#[test]
fn stray_square_brackets() {
    stray_token("[", "`[`");
    stray_token("]", "`]`");
}

#[test]
fn stray_angle_brackets() {
    stray_token("<", "`<`");
    stray_token(">", "`>`");
}

#[test]
fn stray_delims() {
    stray_token(":", "`:`");
    stray_token(",", "`,`");
}

#[test]
fn stray_string() {
    stray_token("\"hello\"", "string");
}

#[test]
fn stray_number() {
    stray_token("1337", "number");
    stray_token("133_7", "number");
    stray_token("-1337", "number");
    stray_token("-13_37", "number");
}
