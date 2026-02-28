use std::{fs::read, path::PathBuf};

use protoview_lib::parse_proto;

#[test]
fn test_parse_proto_file() {
    let content = read(PathBuf::from("tests/resources/test_proto.bin")).expect("File exists");

    let parsed = parse_proto(&content);

    let expected_hex =
        hex::decode("0802120568656c6c6f1801").expect("Hex string is static and valid");
    let expected = parse_proto(&expected_hex);

    assert!(parsed.is_ok());
    assert_eq!(expected, parsed);
}
