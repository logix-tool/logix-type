use super::*;

fn stray_token(token: &str, token_str: &'static str) {
    stray_token_after_struct(token, token_str);
    stray_token_in_map("", token, token_str, Wanted::Ident, "identifier");
    stray_token_in_map(
        "  bb: 10 ",
        token,
        token_str,
        Wanted::Token(Token::Newline(false)),
        "newline",
    );
}

fn stray_token_after_struct(token: &str, token_str: &'static str) {
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
            format!("    | {} expected end of file\n", "^".repeat(token.len())),
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

fn stray_token_in_map(
    prefix: &str,
    token: &str,
    token_str: &'static str,
    wanted: Wanted,
    wanted_str: &str,
) {
    if token == "}" {
        return;
    }

    let mut l = Loader::init().with_file(
        "test.logix",
        format!("{{\n  aaa: 10\n{prefix}{token}}}\n").as_bytes(),
    );
    let e = l.parse_file::<Map<i32>>("test.logix");
    let prefix_len = prefix.len();

    assert_eq!(
        e,
        ParseError::UnexpectedToken {
            span: l.span("test.logix", 3, prefix_len, token.len()),
            while_parsing: "map",
            wanted,
            got_token: token_str,
        }
    );

    assert_eq!(
        debval(&e),
        [
            format!("\n"),
            format!("error: Unexpected {token_str} while parsing `map`\n"),
            format!("   ---> test.logix:3:{prefix_len}\n"),
            format!("    |\n"),
            format!("  2 |   aaa: 10\n"),
            format!("  3 | {prefix}{token}}}\n"),
            format!(
                "    | {:prefix_len$}{} expected {wanted_str}\n",
                "",
                "^".repeat(token.len())
            ),
        ]
        .into_iter()
        .collect::<String>(),
    );

    assert_eq!(
        disval(&e),
        format!(
            "Unexpected {token_str} while parsing `map`, expected {wanted_str} in test.logix:3:{prefix_len}"
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
