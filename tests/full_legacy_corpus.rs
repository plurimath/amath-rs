#[path = "support/raw_profiles.rs"]
mod raw_profiles;
use amath::Parser;
use raw_profiles::{ordered_without_slice_metadata, plain_values};
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

#[test]
#[ignore = "complete frozen legacy values run in platform CI"]
fn full_frozen_legacy_values() {
    let parser = Parser::new().unwrap();
    let corpus: Value =
        serde_json::from_str(include_str!("fixtures/full-legacy-values.json")).unwrap();
    assert_eq!(corpus["cases"].as_array().unwrap().len(), 160);
    // The thirteen source-declared no-tree exclusions remain in the fixture.
    assert_eq!(corpus["excluded"].as_array().unwrap().len(), 13);
    // The frozen producer sorts Hash keys for stable files and explicitly
    // treats parser insertion order as an implementation detail. Its values,
    // arrays, captures and all160 inputs remain asserted verbatim.
    for row in corpus["cases"].as_array().unwrap() {
        let input = row["input"].as_str().unwrap();
        let parsed = parser
            .parse_legacy_with_decimal(input, row["decimal"].as_str().unwrap_or("."))
            .unwrap_or_else(|e| panic!("{}: {}", row["id"], e));
        assert_eq!(
            plain_values(&parsed.tree),
            plain_values(&row["tree"]),
            "{}",
            row["id"]
        );
    }
}

#[test]
#[ignore = "complete current producer profile runs in platform CI"]
fn full_current_producer_member_order() {
    let parser = Parser::new().unwrap();
    let current: Value =
        serde_json::from_str(include_str!("fixtures/current-raw-a4ed231-v1.json")).unwrap();
    let archived: Value =
        serde_json::from_str(include_str!("fixtures/full-legacy-values.json")).unwrap();
    assert_eq!(current["cases"].as_array().unwrap().len(), 160);
    assert_eq!(current["excluded"], archived["excluded"]);
    assert_eq!(current["producer"]["engine_agreement"], true);
    for (row, old) in current["cases"]
        .as_array()
        .unwrap()
        .iter()
        .zip(archived["cases"].as_array().unwrap())
    {
        assert_eq!(row["id"], old["id"]);
        assert_eq!(row["input"], old["input"]);
        let parsed = parser
            .parse_legacy_with_decimal(
                row["input"].as_str().unwrap(),
                row["decimal"].as_str().unwrap(),
            )
            .unwrap();
        assert_eq!(
            ordered_without_slice_metadata(&parsed.tree),
            ordered_without_slice_metadata(&row["tree"]),
            "{}",
            row["id"]
        );
    }
}
