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
            let resource = match &e {
                crate::typed::ModelError::Parse(crate::Error::Runtime(
                    parsanol::parg::PargError::RecursionLimitExceeded { depth, max_depth },
                )) => Some(("recursion_limit", *depth, *max_depth)),
                crate::typed::ModelError::Parse(crate::Error::Runtime(
                    parsanol::parg::PargError::CallDepthLimitExceeded { depth, max_depth },
                )) => Some(("call_depth_limit", *depth, *max_depth)),
                _ => None,
            };
            if let Some((category, depth, max_depth)) = resource {
                return json!({"abi_version":1,"ok":false,"error":{"category":category,"depth":depth,"max_depth":max_depth,"message":e.to_string(),"byte_offset":null,"offset_source":"preprocessed_input"}});
            }
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
fn pma_response(input: &str, decimal: &str) -> Value {
    use crate::pma::PmaError;
    match crate::pma::parse_model(input, decimal) {
        Ok(model) => match crate::pma::to_portable_json(&model) {
            Ok(model) => {
                json!({"abi_version":1,"ok":true,"dialect":"pma","model":serde_json::from_str::<Value>(&model).expect("generated JSON codec")})
            }
            Err(e) => {
                json!({"abi_version":1,"ok":false,"error":{"category":"model_error","message":e.to_string()}})
            }
        },
        Err(PmaError::Refused(r)) => {
            json!({"abi_version":1,"ok":false,"error":{"category":"parse_error","refusal_category":r.category,"message":r.to_string(),"byte_offset":r.byte_offset,"char_offset":r.char_offset,"offset_source":"caller_input"}})
        }
        Err(PmaError::UnsupportedDecimal(d)) => {
            json!({"abi_version":1,"ok":false,"error":{"category":"invalid_argument","message":PmaError::UnsupportedDecimal(d).to_string()}})
        }
        Err(PmaError::Resource(e)) => {
            let (category, depth, max_depth) = match &e {
                parsanol::parg::PargError::RecursionLimitExceeded { depth, max_depth } => {
                    ("recursion_limit", *depth, *max_depth)
                }
                parsanol::parg::PargError::CallDepthLimitExceeded { depth, max_depth } => {
                    ("call_depth_limit", *depth, *max_depth)
                }
                _ => ("internal_error", 0, 0),
            };
            json!({"abi_version":1,"ok":false,"error":{"category":category,"depth":depth,"max_depth":max_depth,"message":e.to_string(),"byte_offset":null,"offset_source":"caller_input"}})
        }
        Err(e @ PmaError::Artifact(_)) => {
            json!({"abi_version":1,"ok":false,"error":{"category":"artifact_error","message":e.to_string()}})
        }
        Err(e) => {
            json!({"abi_version":1,"ok":false,"error":{"category":"model_error","message":e.to_string()}})
        }
    }
}
/// Dialect-selecting model parse (additive to ABI v1). `dialect` is "legacy"
/// (dialect L, identical to `amath_parse_model_json_v1`) or "pma" (dialect P).
///
/// # Safety
/// Each nonempty buffer must point to readable memory of its declared length.
#[no_mangle]
pub unsafe extern "C" fn amath_parse_model_dialect_json_v1(
    input: *const u8,
    input_len: usize,
    decimal: *const u8,
    decimal_len: usize,
    dialect: *const u8,
    dialect_len: usize,
) -> *mut c_char {
    let reply=catch_unwind(AssertUnwindSafe(||{
        match(text(input,input_len),text(decimal,decimal_len),text(dialect,dialect_len)){
            (Ok(input),Ok(decimal),Ok(dialect))=>match crate::pma::Dialect::from_name(dialect){
                Some(crate::pma::Dialect::Legacy)=>response(input,decimal),
                Some(crate::pma::Dialect::Pma)=>pma_response(input,decimal),
                None=>json!({"abi_version":1,"ok":false,"error":{"category":"invalid_argument","message":format!("unknown dialect {dialect:?}; expected \"legacy\" or \"pma\"")}}),
            },
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
