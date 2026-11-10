#![allow(dead_code)]
use serde_json::{json, Map, Value};
use std::collections::HashMap;
pub fn is_input_ref(map: &Map<String, Value>) -> bool {
    map.get("offset").is_some_and(Value::is_u64)
        && map.get("length").is_some_and(Value::is_u64)
        && map.keys().all(|k| {
            matches!(
                k.as_str(),
                "offset" | "length" | "value" | "line" | "column"
            )
        })
}

fn is_tagged_array(map: &Map<String, Value>) -> bool {
    map.len() == 2
        && map.get("tag").is_some_and(Value::is_string)
        && map.get("items").is_some_and(Value::is_array)
}

fn is_plain_hash(value: &Value) -> bool {
    matches!(value, Value::Object(m) if !is_input_ref(m))
}

/// Flatten parsanol-tree/v2 TaggedArray nodes the way the Parslet-style
/// binders do: `repetition` is an array, `maybe` is its item or nil,
/// `sequence` merges hashes (captures win over bare spans), drops nils,
/// and a lone item is itself.
pub fn flatten_shape(value: &Value) -> Value {
    match value {
        Value::Array(items) => Value::Array(items.iter().map(flatten_shape).collect()),
        Value::Object(map) if is_input_ref(map) => value.clone(),
        Value::Object(map) if is_tagged_array(map) => {
            let items: Vec<Value> = map["items"]
                .as_array()
                .unwrap()
                .iter()
                .map(flatten_shape)
                .collect();
            let tag = map["tag"].as_str().unwrap().trim_start_matches(':');
            match tag {
                "repetition" => Value::Array(items),
                "maybe" => items.into_iter().next().unwrap_or(Value::Null),
                "sequence" => fold_sequence(items),
                other => panic!("parsanol-tree/v2: unknown TaggedArray tag {other:?}"),
            }
        }
        Value::Object(map) => Value::Object(
            map.iter()
                .map(|(k, v)| (k.clone(), flatten_shape(v)))
                .collect(),
        ),
        other => other.clone(),
    }
}

fn fold_sequence(items: Vec<Value>) -> Value {
    let present: Vec<Value> = items.into_iter().filter(|i| !i.is_null()).collect();
    if present.is_empty() {
        return Value::Null;
    }
    if present.len() == 1 {
        return present.into_iter().next().unwrap();
    }
    let hashes = present.iter().filter(|i| is_plain_hash(i)).count();
    let refs_or_hashes = present.iter().all(|i| matches!(i, Value::Object(_)));
    if hashes > 0 && refs_or_hashes {
        let mut merged = Map::new();
        for item in present.iter().filter(|i| is_plain_hash(i)) {
            merged.extend(item.as_object().unwrap().clone());
        }
        return Value::Object(merged);
    }
    Value::Array(present)
}

/// `value` (a parsanol-tree/v2 node) in canonical tagged form. `source`
/// is the grammar input the slices point into.
pub fn dump_canonical(value: &Value, source: &str, renames: &HashMap<String, String>) -> Value {
    fn dump(v: &Value, source: &str, renames: &HashMap<String, String>) -> Value {
        match v {
            Value::Null => json!({"nil": true}),
            Value::Bool(b) => json!({"bool": b}),
            Value::Number(n) => match n.as_i64() {
                Some(i) => json!({"int": i.to_string()}),
                None => panic!("no canonical form for non-integer {n}"),
            },
            Value::String(s) => json!({"str": s}),
            Value::Array(items) => {
                json!({"array": items.iter().map(|i| dump(i, source, renames)).collect::<Vec<_>>()})
            }
            Value::Object(map) if is_input_ref(map) => {
                let offset = map["offset"].as_u64().unwrap() as usize;
                let text = match map.get("value").and_then(Value::as_str) {
                    Some(text) => text.to_string(),
                    None => {
                        let length = map["length"].as_u64().unwrap() as usize;
                        source[offset..].chars().take(length).collect()
                    }
                };
                let char_offset = source.get(..offset).map(|s| s.chars().count());
                json!({"slice": text, "byte_offset": offset, "char_offset": char_offset})
            }
            Value::Object(map) => {
                let pairs: Vec<Value> = map
                    .iter()
                    .map(|(k, item)| {
                        let key = match renames.get(k) {
                            Some(renamed) => json!({"str": renamed}),
                            None => json!({"sym": k}),
                        };
                        json!([key, dump(item, source, renames)])
                    })
                    .collect();
                json!({"hash": pairs})
            }
        }
    }
    dump(&flatten_shape(value), source, renames)
}
