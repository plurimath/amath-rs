use serde_json::Value;

pub fn ordered_without_slice_metadata(value: &Value) -> Value {
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
/// Preserve values and array order, but do not promote a generator's sorted
/// object member order into a parser API contract.
pub fn plain_values(value: &Value) -> Value {
    match value {
        Value::Object(map)
            if map.len() == 5
                && ["offset", "length", "line", "column", "value"]
                    .iter()
                    .all(|key| map.contains_key(*key)) =>
        {
            value["value"].clone()
        }
        Value::Object(map) => Value::Object(
            map.iter()
                .map(|(key, child)| (key.clone(), plain_values(child)))
                .collect(),
        ),
        Value::Array(rows) => Value::Array(rows.iter().map(plain_values).collect()),
        _ => value.clone(),
    }
}
