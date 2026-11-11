use amath::Parser;
use serde_json::Value;
fn normalize(v: &Value) -> Value {
    match v {
        Value::Array(a) => Value::Array(a.iter().map(normalize).collect()),
        Value::Object(m) => {
            let mut m = m
                .iter()
                .map(|(k, v)| (k.clone(), normalize(v)))
                .collect::<serde_json::Map<String, Value>>();
            if let Some(Value::Array(pairs)) = m.get_mut("hash") {
                pairs.sort_by_key(|v| v[0].to_string());
            }
            Value::Object(m)
        }
        _ => v.clone(),
    }
}
#[test]
fn pinned_stage_sample() {
    let parser = Parser::new().unwrap();
    let cases: Value = serde_json::from_str(include_str!("fixtures/raw-smoke.json")).unwrap();
    for row in cases.as_array().unwrap() {
        let input = row["input"]["str"].as_str().unwrap();
        let decimal = row["options"]["decimal"].as_str().unwrap_or(".");
        let result = parser.parse_with_decimal(input, decimal);
        if row["expected"].get("refusal").is_some() {
            assert!(result.is_err(), "{}: expected refusal", row["id"]);
        } else {
            let parsed = result.unwrap_or_else(|e| panic!("{}: {}", row["id"], e));
            assert_eq!(
                parsed.parsed_input,
                row["expected"]["preprocessed"]["str"].as_str().unwrap(),
                "{} preprocessing",
                row["id"]
            );
            assert_eq!(
                normalize(&parsed.canonical()),
                normalize(&row["expected"]["tree"]),
                "{} tree",
                row["id"]
            );
        }
    }
}
