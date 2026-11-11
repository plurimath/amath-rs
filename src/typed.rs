//! Stage-specific adapter into upstream generated Lutaml models and codecs.
use crate::{
    generated::{json_codecs, models::Amath_Model_Document},
    Error, Parser,
};
use serde_json::{json, Value};
use std::{collections::HashMap, fmt};
#[derive(Debug)]
pub enum ModelError {
    Parse(Error),
    Codec(Box<lutaml_rs::contracts::ContractError>),
}
impl fmt::Display for ModelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Parse(e) => write!(f, "{e}"),
            Self::Codec(e) => write!(f, "{e:?}"),
        }
    }
}
impl std::error::Error for ModelError {}
fn strings(value: &Value, source: &str, catalog: &HashMap<String, Value>) -> Value {
    match value {
        Value::Array(a) => Value::Array(a.iter().map(|v| strings(v, source, catalog)).collect()),
        Value::Object(map) if crate::replay::is_input_ref(map) => {
            if let Some(text) = map.get("value").and_then(Value::as_str) {
                return Value::String(text.into());
            }
            let offset = map["offset"].as_u64().unwrap() as usize;
            let length = map["length"].as_u64().unwrap() as usize;
            Value::String(source[offset..].chars().take(length).collect())
        }
        Value::Object(map) => {
            let mut object = map
                .iter()
                .map(|(k, v)| (k.clone(), strings(v, source, catalog)))
                .collect::<serde_json::Map<String, Value>>();
            if let Some(spelling) = object
                .get("spelling")
                .and_then(Value::as_str)
                .map(str::to_string)
            {
                let stripped = spelling.trim_matches(|c| {
                    matches!(c, '\0' | '\t' | '\n' | '\x0b' | '\x0c' | '\r' | ' ')
                });
                let found = catalog
                    .get(&spelling)
                    .or_else(|| catalog.get(stripped).map(|v| v.get("padded").unwrap_or(v)));
                object.insert(
                    "value".into(),
                    found
                        .and_then(|v| v.get("value"))
                        .cloned()
                        .unwrap_or_else(|| Value::String(spelling.clone())),
                );
                object.insert(
                    "kind".into(),
                    found
                        .and_then(|v| v.get("kind"))
                        .cloned()
                        .unwrap_or_else(|| Value::String("unlisted".into())),
                );
            }
            Value::Object(object)
        }
        _ => value.clone(),
    }
}
impl Parser {
    /// The dialect P (`pma`) typed model; explicit, never selected implicitly.
    /// `decimal` must be "." (pma has no decimal option in version 1).
    pub fn parse_pma_model(
        &self,
        input: &str,
        decimal: &str,
    ) -> Result<crate::generated::models::Amath_Model_PmaDocument, crate::pma::PmaError> {
        crate::pma::parse_model(input, decimal)
    }
    pub fn parse_model(
        &self,
        input: &str,
        decimal: &str,
    ) -> Result<Amath_Model_Document, ModelError> {
        let parsed = self
            .parse_with_decimal(input, decimal)
            .map_err(ModelError::Parse)?;
        let catalog = serde_json::from_str(include_str!("../artifacts/catalog.json"))
            .expect("embedded stage catalog");
        let mut wire = if input.is_empty() {
            json!({})
        } else {
            strings(&parsed.bound, &parsed.parsed_input, &catalog)
        };
        wire.as_object_mut()
            .expect("declared bindings object")
            .insert("decimal".into(), Value::String(decimal.into()));
        json_codecs::read_Amath_Model_Document_json(&wire.to_string()).map_err(ModelError::Codec)
    }
}
pub fn to_portable_json(model: &Amath_Model_Document) -> Result<String, ModelError> {
    json_codecs::write_Amath_Model_Document_json(model).map_err(ModelError::Codec)
}
