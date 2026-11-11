use amath::Parser;
use serde_json::Value;
#[test]
fn frozen_raw_smoke_legacy_trees_and_offsets() {
    let parser = Parser::new().unwrap();
    let rows: Value = serde_json::from_str(include_str!("fixtures/raw-smoke.json")).unwrap();
    for row in rows.as_array().unwrap() {
        let input = row["input"]["str"].as_str().unwrap();
        let decimal = row["options"]["decimal"].as_str().unwrap_or(".");
        let result = parser.parse_legacy_with_decimal(input, decimal);
        if let Some(refusal) = row["expected"].get("refusal") {
            let error = result.unwrap_err();
            if let Some(offset) = refusal["byte_offset"].as_u64() {
                assert_eq!(error.byte_offset(), Some(offset as usize), "{}", row["id"]);
            }
        } else {
            let parsed = result.unwrap_or_else(|e| panic!("{}: {}", row["id"], e));
            assert_eq!(
                parsed.parsed_input,
                row["expected"]["preprocessed"]["str"].as_str().unwrap(),
                "{}",
                row["id"]
            );
            // Hash pair order and every slice byte/character offset are part
            // of the fixture. Do not sort or normalize away differences.
            assert_eq!(parsed.canonical(), row["expected"]["tree"], "{}", row["id"]);
        }
    }
}

#[test]
fn legacy_offsets_are_grammar_bytes_not_plurimath_preprocessed_characters() {
    // Frozen testsuite 0d50db7 rejections.yaml records `error.index` 2,4,4,7: characters in
    // Plurimath's preprocessed text (`{:`->U+2112 etc.). This crate does no AsciiMath
    // preprocessing, so the same failing `/` is reported as a UTF-8 byte offset in the
    // unchanged input. The old corpus offsets are not rewritten.
    let parser = Parser::new().unwrap();
    for (text, byte, plurimath_preprocessed, plurimath_index) in [
        ("{:a/", 3, "\u{2112}a/", 2),
        ("{:x:}a/", 6, "\u{2112}x\u{211B}a/", 4),
        ("(:x:)y/", 6, "\u{1455}x\u{1450}y/", 4),
        (
            "{:a:}(:b:)c/",
            11,
            "\u{2112}a\u{211B}\u{1455}b\u{1450}c/",
            7,
        ),
    ] {
        let error = parser.parse_legacy(text).unwrap_err();
        assert_eq!(error.byte_offset(), Some(byte), "{text}");
        assert_eq!(&text[byte..byte + 1], "/", "{text}");
        assert_eq!(
            plurimath_preprocessed.chars().nth(plurimath_index),
            Some('/'),
            "{text}"
        );
    }
}

#[test]
fn legacy_errors_reparse_causal_tree_without_changing_native_api() {
    let parser = Parser::new().unwrap();
    for (text, native, legacy) in [("a/", 2, 1), ("α a/", 5, 4)] {
        assert_eq!(parser.parse(text).unwrap_err().byte_offset(), Some(native));
        assert_eq!(
            parser.parse_legacy(text).unwrap_err().byte_offset(),
            Some(legacy)
        );
    }
}
