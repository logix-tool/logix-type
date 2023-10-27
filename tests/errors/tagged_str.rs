use super::*;

fn unterminated_eof_tagged(tag: StrTag, tag_str: &str) {
    let mut l = Loader::init().with_file(
        "test.logix",
        format!("Struct {{\n  aaa: 20\n  bbbb: #{tag_str}\"aa").as_bytes(),
    );
    let e = l.parse_struct("test.logix");

    assert_eq!(
        e,
        ParseError::TokenError {
            span: l.span("test.logix", 3, 15, 0),
            error: TokenError::MissingTaggedStringTerminator {
                tag,
                suffix: StrTagSuffix::new(1),
            },
        }
    );

    assert_eq!(
        debval(&e),
        [
            format!("\n"),
            format!("error: Failed to parse input\n"),
            format!("   ---> test.logix:3:15\n"),
            format!("    |\n"),
            format!("  2 |   aaa: 20\n"),
            format!("  3 |   bbbb: #{tag_str}\"aa\n"),
            format!(
                "    |             {:>tag_len$}^ unexpected end of `#{tag_str}` string, expected `\"#`\n",
                "",
                tag_len=tag_str.len(),
            ),
        ]
        .into_iter()
        .collect::<String>(),
    );

    assert_eq!(
        disval(&e),
        format!("Failed to parse input, unexpected end of `#{tag_str}` string, expected `\"#` in test.logix:3:15"),
    );
}

#[test]
fn unterminated_eof_raw() {
    unterminated_eof_tagged(StrTag::Raw, "raw");
}

#[test]
fn unterminated_eof_esc() {
    unterminated_eof_tagged(StrTag::Esc, "esc");
}

#[test]
fn unterminated_eof_txt() {
    unterminated_eof_tagged(StrTag::Txt, "txt");
}
