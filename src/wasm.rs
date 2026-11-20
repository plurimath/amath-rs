//! Optional JavaScript transport over the existing stage parser and model codecs.
use crate::{generated::json_codecs, typed, Parser};
use wasm_bindgen::prelude::*;

fn js_error(error: impl std::fmt::Display) -> JsValue {
    JsValue::from_str(&error.to_string())
}

/// A reusable parser using the crate's verified PARG artifacts and Rust bindings.
#[wasm_bindgen(js_name = StageParser)]
pub struct WasmParser {
    parser: Parser,
}

#[wasm_bindgen(js_class = StageParser)]
impl WasmParser {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Result<WasmParser, JsValue> {
        Ok(Self {
            parser: Parser::new().map_err(js_error)?,
        })
    }

    /// Parse to the generated model's portable JSON, using native strict policy.
    pub fn parse(&self, input: &str, decimal: &str) -> Result<String, JsValue> {
        let model = self.parser.parse_model(input, decimal).map_err(js_error)?;
        typed::to_portable_json(&model).map_err(js_error)
    }

    /// Return preprocessed text and the canonical tagged raw tree as JSON.
    #[wasm_bindgen(js_name = parseRaw)]
    pub fn parse_raw(&self, input: &str, decimal: &str) -> Result<String, JsValue> {
        let parsed = self
            .parser
            .parse_with_decimal(input, decimal)
            .map_err(js_error)?;
        Ok(serde_json::json!({
            "preprocessed": parsed.parsed_input,
            "tree": parsed.canonical(),
        })
        .to_string())
    }

    /// Decode, validate and write portable model JSON through the generated codecs.
    /// The existing writer may omit empty members; this is not a source-text writer.
    pub fn serialize(&self, json: &str) -> Result<String, JsValue> {
        let model = json_codecs::read_Amath_Model_Document_json(json)
            .map_err(|error| JsValue::from_str(&format!("{error:?}")))?;
        typed::to_portable_json(&model).map_err(js_error)
    }
}
