//! Stage ABI v1. Inputs are borrowed UTF-8 buffers for the duration of the call.
//! Returned NUL-terminated JSON belongs to the caller; free it exactly once with
//! `amath_free_json_v1`. Never pass a foreign or already-freed pointer to free.
use serde_json::{json, Value};
use std::{
    ffi::{c_char, CString},
    panic::{catch_unwind, AssertUnwindSafe},
    slice, str,
};
#[no_mangle]
pub extern "C" fn amath_abi_version() -> u32 {
    1
}
unsafe fn text<'a>(ptr: *const u8, len: usize) -> Result<&'a str, &'static str> {
    let bytes = if len == 0 {
        &[]
    } else {
        if ptr.is_null() {
            return Err("null nonempty buffer");
        }
        slice::from_raw_parts(ptr, len)
    };
    str::from_utf8(bytes).map_err(|_| "input must be valid UTF-8")
}
fn response(input: &str, decimal: &str) -> Value {
    let parser = match crate::Parser::new() {
        Ok(p) => p,
        Err(e) => {
            return json!({"abi_version":1,"ok":false,"error":{"category":"artifact_error","message":e.to_string()}})
        }
    };
    match parser.parse_model(input, decimal) {
        Ok(model) => match crate::typed::to_portable_json(&model) {
            Ok(model) => {
                json!({"abi_version":1,"ok":true,"model":serde_json::from_str::<Value>(&model).expect("generated JSON codec")})
            }
            Err(e) => {
                json!({"abi_version":1,"ok":false,"error":{"category":"model_error","message":e.to_string()}})
            }
        },
        Err(e) => {
            let (category, offset) = match &e {
                crate::typed::ModelError::Parse(crate::Error::UnsupportedDecimal(_)) => {
                    ("invalid_argument", None)
                }
                crate::typed::ModelError::Parse(error) => ("parse_error", error.byte_offset()),
                _ => ("model_error", None),
            };
            json!({"abi_version":1,"ok":false,"error":{"category":category,"message":e.to_string(),"byte_offset":offset,"offset_source":"preprocessed_input"}})
        }
    }
}
/// # Safety
/// Each nonempty buffer must point to readable memory of its declared length.
#[no_mangle]
pub unsafe extern "C" fn amath_parse_model_json_v1(
    input: *const u8,
    input_len: usize,
    decimal: *const u8,
    decimal_len: usize,
) -> *mut c_char {
    let reply=catch_unwind(AssertUnwindSafe(||{
        match(text(input,input_len),text(decimal,decimal_len)){
            (Ok(input),Ok(decimal))=>response(input,decimal),
            _=>json!({"abi_version":1,"ok":false,"error":{"category":"invalid_argument","message":"null buffer or invalid UTF-8"}}),
        }
    })).unwrap_or_else(|_|json!({"abi_version":1,"ok":false,"error":{"category":"internal_error","message":"native operation panicked"}}));
    CString::new(reply.to_string())
        .expect("JSON escapes NUL")
        .into_raw()
}
/// # Safety
/// `ptr` must be null or an unfreed pointer returned by this library's ABI v1.
#[no_mangle]
pub unsafe extern "C" fn amath_free_json_v1(ptr: *mut c_char) {
    if !ptr.is_null() {
        drop(CString::from_raw(ptr));
    }
}
