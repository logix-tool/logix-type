use super::*;

#[test]
fn not_valid_number() {
    let mut l = Loader::init().with_file("test.logix", b"Struct {\n  aaa: \"aa\"\n}");
    let e = l.parse_struct("test.logix");

    assert_eq!(
        e,
        ParseError::UnexpectedToken {
            span: l.span("test.logix", 2, 7, 4),
            while_parsing: "u32",
            got_token: "string",
            wanted: Wanted::LitNum("unsigned integer")
        }
    );

    assert_eq!(
        debval(&e),
        concat!(
            "\n",
            "error: Unexpected string while parsing `u32`\n",
            "   ---> test.logix:2:7\n",
            "    |\n",
            "  1 | Struct {\n",
            "  2 |   aaa: \"aa\"\n",
            "    |        ^^^^ expected unsigned integer\n",
            "  3 | }\n",
        )
    );

    assert_eq!(
        disval(&e),
        "Unexpected string while parsing `u32`, expected unsigned integer in test.logix:2:7"
    );
}
