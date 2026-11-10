use amath::Parser;
use serde_json::Value;
#[test]
#[ignore = "complete frozen corpus runs in platform CI"]
fn full_frozen_raw_corpus() {
    let parser = Parser::new().unwrap();
    let corpus: Value = serde_json::from_str(include_str!("fixtures/full-raw.json")).unwrap();
    for row in corpus["cases"].as_array().unwrap() {
        let input = row["input"]["str"].as_str().unwrap();
        let result = parser
            .parse_legacy_with_decimal(input, row["options"]["decimal"].as_str().unwrap_or("."));
        if let Some(refusal) = row["expected"].get("refusal") {
            let error = result.unwrap_err();
            assert_eq!(
                error.byte_offset(),
                refusal["byte_offset"].as_u64().map(|v| v as usize),
                "{}",
                row["id"]
            );
        } else {
            let parsed = result.unwrap_or_else(|e| panic!("{}: {}", row["id"], e));
            assert_eq!(
                parsed.parsed_input,
                row["expected"]["preprocessed"]["str"].as_str().unwrap(),
                "{} preprocessing",
                row["id"]
            );
            assert_eq!(
                parsed.canonical(),
                row["expected"]["tree"],
                "{} tree/order/offsets",
                row["id"]
            );
        }
    }
}

fn ordered_without_slice_metadata(value: &Value) -> Value {
    match value {
        Value::Object(map)
            if map.len() == 5
                && ["offset", "length", "line", "column", "value"]
                    .iter()
                    .all(|key| map.contains_key(*key)) =>
        {
            value["value"].clone()
        }
        Value::Object(map) => Value::Array(
            map.iter()
                .map(|(key, child)| serde_json::json!([key, ordered_without_slice_metadata(child)]))
                .collect(),
        ),
        Value::Array(rows) => {
            Value::Array(rows.iter().map(ordered_without_slice_metadata).collect())
        }
        _ => value.clone(),
    }
}
#[test]
#[ignore = "complete frozen legacy values run in platform CI"]
fn full_frozen_legacy_values() {
    let parser = Parser::new().unwrap();
    let corpus: Value =
        serde_json::from_str(include_str!("fixtures/full-legacy-values.json")).unwrap();
    assert_eq!(corpus["cases"].as_array().unwrap().len(), 160);
    // The thirteen source-declared no-tree exclusions remain in the fixture.
    assert_eq!(corpus["excluded"].as_array().unwrap().len(), 13);
    for row in corpus["cases"].as_array().unwrap() {
        let input = row["input"].as_str().unwrap();
        let parsed = parser
            .parse_legacy_with_decimal(input, row["decimal"].as_str().unwrap_or("."))
            .unwrap_or_else(|e| panic!("{}: {}", row["id"], e));
        assert_eq!(
            ordered_without_slice_metadata(&parsed.tree),
            ordered_without_slice_metadata(&row["tree"]),
            "{}",
            row["id"]
        );
    }
}
