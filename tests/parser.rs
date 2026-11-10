use amath::{Error, Parser};
#[test]
fn parse_and_decimal_contract() {
    let parser = Parser::new().unwrap();
    let parsed = parser.parse("x+1").unwrap();
    assert!(!parsed.canonical().is_null());
    assert!(parser.parse_with_decimal("1,5", ",").is_ok());
    assert!(matches!(
        parser.parse_with_decimal("x", "bad"),
        Err(Error::UnsupportedDecimal(_))
    ));
}
