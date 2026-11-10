#![cfg(feature = "models")]
use amath::{
    generated::json_codecs::read_Amath_Model_Document_json, typed::to_portable_json, Parser,
};
#[test]
fn generated_model_symbol_and_codec_contract() {
    let parser = Parser::new().unwrap();
    for (input, decimal) in [
        ("alpha+1", "."),
        ("sqrt(x)", "."),
        ("a/b", "."),
        ("1,5", ","),
        ("", "."),
    ] {
        let model = parser.parse_model(input, decimal).unwrap();
        let encoded = to_portable_json(&model).unwrap();
        let decoded = read_Amath_Model_Document_json(&encoded).unwrap();
        assert_eq!(model, decoded, "{input}");
        let json: serde_json::Value = serde_json::from_str(&encoded).unwrap();
        assert_eq!(json["decimal"], decimal);
        if input.is_empty() {
            assert!(json.get("expression").is_none());
        }
        if input == "alpha+1" {
            assert!(encoded.contains('α'));
            assert!(encoded.contains("symbol"));
        }
    }
    assert!(parser.parse_model("x", "bad").is_err());
}
