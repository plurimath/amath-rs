//! Replays a pinned plurimath-testsuite asciimath generation extracted by
//! `.github/scripts/extract-corpus.py`. Set `AMATH_CORPUS_CASES` to the
//! extracted JSON; `AMATH_CORPUS_REPORT` optionally receives every outcome.
//!
//! Raw-tree rows compare the legacy raw parser's preprocessing, canonical tree
//! and refusal byte offsets. Typed-model rows (feature `models`) decode the
//! expected Ruby model through the generated codec and compare it with
//! `Parser::parse_model` (the producer's `Amath.parse_model` route) as typed
//! values. AsciiMath has no stage preprocessing, so refusal offsets are caller
//! input byte offsets on both routes. The expected class names are dropped by
//! the projection: every model field has one declared class, so the generated
//! struct types carry the class identity.
use amath::Parser;
use serde_json::{json, Value};
fn raw_failure(parser: &Parser, row: &Value) -> Option<String> {
    let input = row["input"]["str"].as_str().unwrap();
    let decimal = row["options"]["decimal"].as_str().unwrap_or(".");
    let result = parser.parse_legacy_with_decimal(input, decimal);
    if let Some(refusal) = row["expected"].get("refusal") {
        return match result {
            Ok(parsed) => Some(format!("accepted: {}", parsed.canonical())),
            Err(error) => {
                let expected = refusal["byte_offset"].as_u64().map(|v| v as usize);
                (error.byte_offset() != expected).then(|| {
                    format!(
                        "refusal offset {:?} != {:?}: {error}",
                        error.byte_offset(),
                        expected
                    )
                })
            }
        };
    }
    match result {
        Err(error) => Some(format!("refused: {error}")),
        Ok(parsed) => {
            let preprocessed = row["expected"]["preprocessed"]["str"].as_str();
            if preprocessed.is_some_and(|text| text != parsed.parsed_input) {
                Some(format!("preprocessed {:?}", parsed.parsed_input))
            } else if parsed.canonical() != row["expected"]["tree"] {
                Some(format!("tree {}", parsed.canonical()))
            } else {
                None
            }
        }
    }
}

/// Ruby `{node, fields}` / `{str}` / `{array}` model wire to the declared
/// portable JSON shape read by the generated codecs.
#[cfg(feature = "models")]
fn portable(node: &Value) -> Value {
    if let Some(text) = node.get("str") {
        return text.clone();
    }
    if let Some(items) = node.get("array") {
        return Value::Array(items.as_array().unwrap().iter().map(portable).collect());
    }
    let class = node["node"].as_str().unwrap();
    assert!(
        class.starts_with("Amath::Model::"),
        "unexpected model class {class}"
    );
    let mut object = serde_json::Map::new();
    for pair in node["fields"].as_array().unwrap() {
        object.insert(pair[0].as_str().unwrap().into(), portable(&pair[1]));
    }
    Value::Object(object)
}

#[cfg(feature = "models")]
fn typed_failure(parser: &Parser, row: &Value) -> Option<String> {
    use amath::generated::json_codecs::read_Amath_Model_Document_json;
    use amath::typed::ModelError;
    let input = row["input"]["str"].as_str().unwrap();
    let decimal = row["options"]["decimal"].as_str().unwrap_or(".");
    let result = parser.parse_model(input, decimal);
    if let Some(refusal) = row["expected"].get("refusal") {
        let expected = refusal["byte_offset"].as_u64().map(|v| v as usize);
        return match result {
            Ok(model) => Some(format!("accepted: {model:?}")),
            Err(ModelError::Parse(error)) if error.byte_offset() == expected => None,
            Err(error) => Some(format!("refusal {error} (expected offset {expected:?})")),
        };
    }
    let expected =
        match read_Amath_Model_Document_json(&portable(&row["expected"]["tree"]).to_string()) {
            Ok(model) => model,
            Err(error) => return Some(format!("expected model does not decode: {error:?}")),
        };
    match result {
        Err(error) => Some(format!("refused: {error}")),
        Ok(model) if model != expected => Some(format!("model {model:?}")),
        Ok(_) => None,
    }
}

#[cfg(not(feature = "models"))]
fn typed_failure(_: &Parser, _: &Value) -> Option<String> {
    panic!("typed-model rows need --features models")
}

fn replay() {
    let path = std::env::var("AMATH_CORPUS_CASES").expect("AMATH_CORPUS_CASES");
    let text = std::fs::read_to_string(path).unwrap();
    let corpus: Value = serde_json::from_str(&text).unwrap();
    let parser = Parser::new().unwrap();
    let rows = corpus["cases"].as_array().unwrap();
    // Append-only supersession, as amath's testsuite_parity_spec: a row whose
    // id has an -r2 sibling (first "-r2" removed) is kept for history and only
    // the sibling runs. Generation 5 re-homes the bareword uncertainty calls
    // to the quoted channel (amath#17) this way.
    let superseded: std::collections::BTreeSet<String> = rows
        .iter()
        .filter_map(|row| row["id"].as_str())
        .filter(|id| id.contains("-r2"))
        .map(|id| id.replacen("-r2", "", 1))
        .collect();
    let mut outcomes = Vec::new();
    let mut failures = 0usize;
    let mut skipped = Vec::new();
    for row in rows {
        let id = row["id"].as_str().unwrap();
        if superseded.contains(id) {
            skipped.push(id.to_owned());
            continue;
        }
        let started = std::time::Instant::now();
        let failure = match row["boundary"].as_str().unwrap() {
            "raw-tree" => raw_failure(&parser, row),
            "typed-model" => typed_failure(&parser, row),
            other => panic!("{}: undeclared boundary {other}", row["id"]),
        };
        let millis = started.elapsed().as_millis() as u64;
        if millis >= 2000 {
            eprintln!("slow row {} [{}]: {millis} ms", row["id"], row["boundary"]);
        }
        failures += usize::from(failure.is_some());
        let failure = failure.map(|f| f.chars().take(800).collect::<String>());
        outcomes.push(json!({"id": row["id"], "boundary": row["boundary"], "failure": failure, "millis": millis}));
    }
    assert_eq!(
        skipped.len(),
        superseded.len(),
        "every -r2 row must supersede an existing original"
    );
    let report = json!({
        "generation": corpus["generation"],
        "revision": corpus["revision"],
        "executed": outcomes.len(),
        "failed": failures,
        "superseded": skipped,
        "outcomes": outcomes,
    });
    if let Ok(out) = std::env::var("AMATH_CORPUS_REPORT") {
        std::fs::write(out, serde_json::to_string_pretty(&report).unwrap()).unwrap();
    }
    let shown: Vec<_> = outcomes
        .iter()
        .filter(|o| !o["failure"].is_null())
        .take(20)
        .collect();
    assert_eq!(
        failures,
        0,
        "{failures} of {} rows failed: {shown:#?}",
        outcomes.len()
    );
}

#[test]
#[ignore = "pinned generation replay runs in platform CI"]
fn pinned_generation_replay() {
    // The larger stack is for the test's own recursive JSON values. Default-stack
    // parser depth behaviour is checked separately by
    // .github/scripts/check-native-depth.py.
    std::thread::Builder::new()
        .stack_size(64 << 20)
        .spawn(replay)
        .unwrap()
        .join()
        .unwrap();
}
