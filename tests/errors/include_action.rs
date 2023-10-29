use super::*;

#[test]
fn invalid_utf8() {
    #[derive(LogixType, Debug)]
    struct Struct0 {}
    let mut l = Loader::init()
        .with_file("test.txt", b"Hello \x80 World")
        .with_file("test.logix", b"@include(\"test.txt\")");
    let e = l.parse_file::<String>("test.logix");

    assert_eq!(
        e,
        ParseError::IncludeError {
            span: l.span("test.txt", 1, 6, 1),
            while_parsing: "string",
            error: IncludeError::NotUtf8
        }
    );

    assert_eq!(
        debval(&e),
        concat!(
            "\n",
            "error: Failed to include file as `string`\n",
            "   ---> test.txt:1:6\n",
            "    |\n",
            "  1 | Hello \u{fffd} World\n",
            "    |       ^ invalid utf-8 sequence\n",
        )
    );

    assert_eq!(
        disval(&e),
        "Failed to include file as `string`, invalid utf-8 sequence in test.txt:1:6"
    );
}