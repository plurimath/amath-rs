#[path = "support/raw_profiles.rs"]
mod raw_profiles;
use amath::Parser;
use raw_profiles::{ordered_without_slice_metadata, plain_values};
use serde_json::Value;

#[test]
fn sorted_archive_keys_and_actual_current_member_order_have_distinct_contracts() {
    let archived: Value =
        serde_json::from_str(include_str!("fixtures/full-legacy-values.json")).unwrap();
    let current: Value =
        serde_json::from_str(include_str!("fixtures/current-raw-a4ed231-v1.json")).unwrap();
    let old = &archived["cases"][0];
    let now = &current["cases"][0];
    assert_eq!(old["id"], "colour-named");
    assert_eq!(old["input"], now["input"]);
    assert_eq!(plain_values(&old["tree"]), plain_values(&now["tree"]));
    assert_ne!(
        ordered_without_slice_metadata(&old["tree"]),
        ordered_without_slice_metadata(&now["tree"])
    );
    let parsed = Parser::new()
        .unwrap()
        .parse_legacy(old["input"].as_str().unwrap())
        .unwrap();
    assert_eq!(plain_values(&parsed.tree), plain_values(&old["tree"]));
    assert_eq!(
        ordered_without_slice_metadata(&parsed.tree),
        ordered_without_slice_metadata(&now["tree"])
    );
}

#[test]
fn legacy_value_comparison_preserves_capture_values_and_array_order() {
    let value = serde_json::json!({"expr": ["a", "b"]});
    assert_ne!(
        plain_values(&value),
        plain_values(&serde_json::json!({"expr": ["b", "a"]}))
    );
    assert_ne!(
        plain_values(&value),
        plain_values(&serde_json::json!({"other": ["a", "b"]}))
    );
    assert_ne!(
        plain_values(&value),
        plain_values(&serde_json::json!({"expr": ["a", "c"]}))
    );
}

#[test]
fn current_member_order_matches_representative_capture_families() {
    let corpus: Value =
        serde_json::from_str(include_str!("fixtures/current-raw-a4ed231-v1.json")).unwrap();
    let parser = Parser::new().unwrap();
    for id in [
        "colour-named",
        "fence-round-single",
        "fence-over-number",
        "font-bold",
        "font-mixed",
        "frac-explicit",
        "left-right-around-frac",
        "matrix-column",
        "mod-simple",
        "nary-sum-bounded",
        "partial-sqrt-unclosed",
    ] {
        let row = corpus["cases"]
            .as_array()
            .unwrap()
            .iter()
            .find(|row| row["id"] == id)
            .unwrap();
        let parsed = parser.parse_legacy(row["input"].as_str().unwrap()).unwrap();
        assert_eq!(
            ordered_without_slice_metadata(&parsed.tree),
            ordered_without_slice_metadata(&row["tree"]),
            "{id}"
        );
    }
}
