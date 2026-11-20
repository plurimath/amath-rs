//! Native byte reference for the focused WASM smoke inputs; run by WASM CI only.
use amath::{generated::json_codecs, typed, Parser};
use serde_json::{json, Value};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cases: Value = serde_json::from_str(include_str!("../tests/fixtures/wasm-smoke.json"))?;
    let parser = Parser::new()?;
    let mut rows = Vec::new();
    for case in cases.as_array().unwrap() {
        let input = case["input"].as_str().unwrap();
        let decimal = case["decimal"].as_str().unwrap();
        let parsed = parser.parse_with_decimal(input, decimal)?;
        let raw =
            json!({"preprocessed": parsed.parsed_input, "tree": parsed.canonical()}).to_string();
        let model = typed::to_portable_json(&parser.parse_model(input, decimal)?)?;
        let decoded = json_codecs::read_Amath_Model_Document_json(&model)?;
        let serialized = typed::to_portable_json(&decoded)?;
        rows.push(json!({"input": input, "decimal": decimal, "raw": raw,
            "model": model, "serialized": serialized}));
    }
    println!("{}", serde_json::to_string(&rows)?);
    Ok(())
}
