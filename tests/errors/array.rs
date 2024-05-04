use super::*;

#[test]
fn too_small_array() {
    let mut l = Loader::init().with_file("test.logix", "[\n  1\n  2\n  3\n]".as_bytes());
    let e = l.parse_file::<[i32; 4]>("test.logix");

    assert_eq!(
        e,
        ParseError::UnexpectedToken {
            span: l.span("test.logix", 5, 0, 1),
            while_parsing: "Array",
            got_token: "`]`",
            wanted: Wanted::Item
        }
    );

    assert_eq!(
        debval(&e),
        concat!(
            "\n",
            "error: Unexpected `]` while parsing `Array`\n",
            "   ---> test.logix:5:0\n",
            "    |\n",
            "  4 |   3\n",
            "  5 | ]\n",
            "    | ^ expected item\n",
        )
    );

    assert_eq!(
        disval(&e),
        "Unexpected `]` while parsing `Array`, expected item in test.logix:5:0"
    );
}

#[test]
fn too_big_array() {
    let mut l = Loader::init().with_file("test.logix", "[\n  1\n  2\n  3\n]".as_bytes());
    let e = l.parse_file::<[i32; 2]>("test.logix");

    assert_eq!(
        e,
        ParseError::UnexpectedToken {
            span: l.span("test.logix", 4, 2, 1),
            while_parsing: "Array",
            got_token: "number",
            wanted: Wanted::Token(Token::Brace {
                start: false,
                brace: Brace::Square
            }),
        }
    );

    assert_eq!(
        debval(&e),
        concat!(
            "\n",
            "error: Unexpected number while parsing `Array`\n",
            "   ---> test.logix:4:2\n",
            "    |\n",
            "  3 |   2\n",
            "  4 |   3\n",
            "    |   ^ expected `]`\n",
            "  5 | ]\n",
        )
    );

    assert_eq!(
        disval(&e),
        "Unexpected number while parsing `Array`, expected `]` in test.logix:4:2"
    );
}

#[test]
fn truncated_array_last_item() {
    let mut l = Loader::init().with_file("test.logix", "[\n  1\n  2\n  3\n".as_bytes());
    let e = l.parse_file::<[i32; 3]>("test.logix");

    assert_eq!(
        e,
        ParseError::UnexpectedToken {
            span: l.span("test.logix", 4, 3, 0),
            while_parsing: "Array",
            got_token: "end of file",
            wanted: Wanted::ItemOrEnd,
        }
    );

    assert_eq!(
        debval(&e),
        concat!(
            "\n",
            "error: Unexpected end of file while parsing `Array`\n",
            "   ---> test.logix:4:3\n",
            "    |\n",
            "  3 |   2\n",
            "  4 |   3\n",
            "    |    ^ expected item or end\n",
        )
    );

    assert_eq!(
        disval(&e),
        "Unexpected end of file while parsing `Array`, expected item or end in test.logix:4:3"
    );
}

#[test]
fn truncated_array_penultimate_item() {
    let mut l = Loader::init().with_file("test.logix", "[\n  1\n  2\n  3\n".as_bytes());
    let e = l.parse_file::<[i32; 4]>("test.logix");

    assert_eq!(
        e,
        ParseError::UnexpectedToken {
            span: l.span("test.logix", 4, 3, 0),
            while_parsing: "Array",
            got_token: "end of file",
            wanted: Wanted::ItemOrEnd
        }
    );

    assert_eq!(
        debval(&e),
        concat!(
            "\n",
            "error: Unexpected end of file while parsing `Array`\n",
            "   ---> test.logix:4:3\n",
            "    |\n",
            "  3 |   2\n",
            "  4 |   3\n",
            "    |    ^ expected item or end\n",
        )
    );

    assert_eq!(
        disval(&e),
        "Unexpected end of file while parsing `Array`, expected item or end in test.logix:4:3"
    );
}
