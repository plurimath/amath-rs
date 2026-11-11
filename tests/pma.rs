//! Dialect P (pma): every accepted and refused corpus row against the Ruby
//! reference `Amath.parse_model(input, dialect: :pma)` output
//! (tests/fixtures/pma-reference.json, generated from amath 71df055).
#![cfg(feature = "models")]
use amath::pma::{self, PmaError, CATEGORIES};
use serde_json::Value;

fn reference() -> Value {
    serde_json::from_str(include_str!("fixtures/pma-reference.json")).unwrap()
}

#[test]
fn accepts_every_reference_row_with_the_reference_term() {
    let r = reference();
    let rows = r["accepted"].as_array().unwrap();
    assert_eq!(rows.len(), 38);
    for row in rows {
        let input = row["input"].as_str().unwrap();
        let model =
            pma::parse_model(input, ".").unwrap_or_else(|e| panic!("{}: {input}: {e}", row["id"]));
        let json: Value = serde_json::from_str(&pma::to_portable_json(&model).unwrap()).unwrap();
        assert_eq!(json["term"], row["term"], "{}: {input}", row["id"]);
    }
}

#[test]
fn refuses_every_reference_row_with_category_and_offsets() {
    let r = reference();
    let rows = r["refused"].as_array().unwrap();
    assert_eq!(rows.len(), 60);
    let mut seen = std::collections::BTreeSet::new();
    for row in rows {
        let input = row["input"].as_str().unwrap();
        match pma::parse_model(input, ".") {
            Err(PmaError::Refused(refusal)) => {
                seen.insert(refusal.category);
                let got = (
                    refusal.category,
                    refusal.byte_offset.map(|b| b as u64),
                    refusal.char_offset.map(|c| c as u64),
                );
                let want = (
                    row["category"].as_str().unwrap(),
                    row["byte_offset"].as_u64(),
                    row["char_offset"].as_u64(),
                );
                assert_eq!(got, want, "{}: {input}", row["id"]);
                assert_eq!(refusal.input, input);
            }
            other => panic!("{}: {input}: expected a refusal, got {other:?}", row["id"]),
        }
    }
    assert_eq!(
        seen.len(),
        CATEGORIES.len(),
        "a reference row for every category"
    );
}

#[test]
fn has_no_decimal_option_and_no_dialect_l_fallback() {
    assert!(matches!(
        pma::parse_model("unit(m)", ","),
        Err(PmaError::UnsupportedDecimal(_))
    ));
    // Dialect L input is not pma: refused, never parsed by the dialect L artifact.
    assert!(matches!(
        pma::parse_model("x^2", "."),
        Err(PmaError::Refused(_))
    ));
    let legacy = amath::Parser::new().unwrap();
    assert!(legacy.parse_model("x^2", ".").is_ok());
}

#[test]
fn deep_brackets_are_refused_without_a_stack_error() {
    let inputs = [
        "(".repeat(5000),
        format!("unit({}m{})", "(".repeat(5000), ")".repeat(5000)),
        format!("diff({}", "(".repeat(5000)),
    ];
    for input in inputs {
        // Like the reference: a ParseError refusal, not a nesting/budget abort.
        assert!(
            matches!(pma::parse_model(&input, "."), Err(PmaError::Refused(_))),
            "{}",
            &input[..20]
        );
    }
}
