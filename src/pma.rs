//! pma (dialect P, "plurimath ascii"): the typed `PmaDocument` model, selected
//! explicitly with [`Dialect::Pma`]. A port of the Ruby reference
//! `Amath::Pma` (amath lib/amath/pma.rb): it parses with its own producer
//! artifact (`artifacts/pma.json`) and never falls back to dialect L. The text
//! of `unit(...)` is parsed by the asciiunits artifact whose checksum
//! `artifacts/pma.checksums.txt` records, and a quantity assertion must name a
//! unitsdb quantity from the sealed `artifacts/pma.quantities.json`.
//!
//! Every refusal is a [`PmaRefusal`] whose `category` names the rule the input
//! breaks ([`CATEGORIES`]), with the parser's position as a caller-input UTF-8
//! byte offset and the matching character offset.
use crate::generated::{json_codecs, models::Amath_Model_PmaDocument};
use parsanol::parg::bindings_artifact::PargBindingsArtifact;
use parsanol::portable::parser::ParserCompatibility as Policy;
use parsanol::{PargArtifact, PargError};
use serde_json::{Map, Value};
use sha2::{Digest, Sha256};
use std::{fmt, sync::OnceLock};

pub const ENTRY: &str = "document";
pub const UNITS_ENTRY: &str = "expression";

/// The refusal categories of plurimath docs/pma.adoc "Strict error taxonomy",
/// plus `no-production`.
pub const CATEGORIES: [&str; 14] = [
    "reserved-word",
    "adjacency",
    "constructor-arity",
    "constructor-argument",
    "measured-literal",
    "plus-minus-spelling",
    "shared-exponent",
    "relative",
    "distribution-tag",
    "concise",
    "unit-expression",
    "unit-attachment",
    "quantity-assertion",
    "no-production",
];

/// Which typed model `parse_model_dialect` builds. `Legacy` is dialect L, the
/// existing `Parser::parse_model`; `Pma` is dialect P.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Dialect {
    Legacy,
    Pma,
}
impl Dialect {
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "legacy" | "L" => Some(Self::Legacy),
            "pma" | "P" => Some(Self::Pma),
            _ => None,
        }
    }
}

/// A refused pma input: `category` is one of [`CATEGORIES`]; `byte_offset` is
/// a UTF-8 byte offset in the caller input (pma has no preprocessing) and
/// `char_offset` the character offset of the same position.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PmaRefusal {
    pub input: String,
    pub category: &'static str,
    pub byte_offset: Option<usize>,
    pub char_offset: Option<usize>,
}
impl fmt::Display for PmaRefusal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match (self.byte_offset, self.char_offset) {
            (Some(b), Some(c)) => write!(
                f,
                "pma input rejected at character {c} (byte {b}): {}",
                self.category
            ),
            _ => write!(f, "pma input rejected: {}", self.category),
        }
    }
}

#[derive(Debug)]
pub enum PmaError {
    Refused(PmaRefusal),
    /// pma has no `decimal:` option in version 1.
    UnsupportedDecimal(String),
    /// A parser budget abort (recursion/call depth), not a refusal.
    Resource(PargError),
    Artifact(String),
    Codec(Box<lutaml_rs::contracts::ContractError>),
}
impl fmt::Display for PmaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Refused(r) => write!(f, "{r}"),
            Self::UnsupportedDecimal(d) => write!(
                f,
                "dialect pma has no decimal option in version 1 (got {d:?}); numbers use the full stop"
            ),
            Self::Resource(e) => write!(f, "{e}"),
            Self::Artifact(m) => write!(f, "pma artifact error: {m}"),
            Self::Codec(e) => write!(f, "{e:?}"),
        }
    }
}
impl std::error::Error for PmaError {}

struct Pma {
    artifact: PargArtifact,
    bindings: PargBindingsArtifact,
    units: PargArtifact,
    quantities: Map<String, Value>,
}

fn load() -> Result<Pma, PmaError> {
    let art = |e: PargError| PmaError::Artifact(e.to_string());
    let artifact = PargArtifact::from_json(include_str!("../artifacts/pma.json")).map_err(art)?;
    let bindings = PargBindingsArtifact::from_json(
        include_str!("../artifacts/pma.bindings.rs.json"),
        Some("rs"),
    )
    .map_err(art)?;
    // The asciiunits artifact must be the one the pma artifact was composed with.
    let checksums = include_str!("../artifacts/pma.checksums.txt");
    let expected = checksums
        .lines()
        .find_map(|l| l.strip_prefix("asciiunits_artifact_checksum "))
        .and_then(|rest| rest.split_whitespace().next())
        .ok_or_else(|| {
            PmaError::Artifact("pma.checksums.txt records no asciiunits artifact".into())
        })?;
    let units_json = include_str!("../artifacts/asciiunits.unitsml.json");
    let actual = serde_json::from_str::<Value>(units_json)
        .ok()
        .and_then(|v| {
            v.get("checksum")
                .and_then(Value::as_str)
                .map(str::to_string)
        })
        .unwrap_or_default();
    if actual != expected {
        return Err(PmaError::Artifact(format!(
            "asciiunits artifact has checksum {actual}, the pma artifact was composed with {expected}"
        )));
    }
    let units = PargArtifact::from_json(units_json).map_err(art)?;
    let doc: Value = serde_json::from_str(include_str!("../artifacts/pma.quantities.json"))
        .map_err(|e| PmaError::Artifact(format!("quantities: {e}")))?;
    let map = doc
        .get("quantities")
        .and_then(Value::as_object)
        .cloned()
        .ok_or_else(|| PmaError::Artifact("quantities: no quantities map".into()))?;
    let digest = Sha256::digest(Value::Object(map.clone()).to_string().as_bytes());
    let seal = format!(
        "sha256:{}",
        digest
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>()
    );
    if doc.get("sha256").and_then(Value::as_str) != Some(seal.as_str()) {
        return Err(PmaError::Artifact(
            "pma.quantities.json does not match its recorded sha256".into(),
        ));
    }
    Ok(Pma {
        artifact,
        bindings,
        units,
        quantities: map,
    })
}

fn pma() -> Result<&'static Pma, PmaError> {
    static CELL: OnceLock<Result<Pma, String>> = OnceLock::new();
    CELL.get_or_init(|| load().map_err(|e| e.to_string()))
        .as_ref()
        .map_err(|m| PmaError::Artifact(m.clone()))
}

/// Characters of `text[..byte]`, counting a split character once (Ruby
/// `byteslice(0, byte).scrub.length`).
fn char_offset(text: &str, byte: usize) -> usize {
    String::from_utf8_lossy(&text.as_bytes()[..byte.min(text.len())])
        .chars()
        .count()
}

fn refusal(text: &str, byte: Option<usize>, category: &'static str) -> PmaError {
    PmaError::Refused(PmaRefusal {
        input: text.to_string(),
        category,
        byte_offset: byte,
        char_offset: byte.map(|b| char_offset(text, b)),
    })
}

fn is_resource(e: &PargError) -> bool {
    matches!(
        e,
        PargError::RecursionLimitExceeded { .. } | PargError::CallDepthLimitExceeded { .. }
    )
}

/// Strict native parse; on a syntax rejection the position is the Parslet-style
/// causal one (the reference's `ErrorPosition`), like `Parser::parse_legacy`.
fn shape(
    artifact: &PargArtifact,
    entry: &str,
    text: &str,
) -> Result<Value, Result<Option<usize>, PargError>> {
    match artifact.parse_shape_with_compatibility(entry, text, Policy::Strict) {
        Ok(v) => Ok(v),
        Err(e) if is_resource(&e) => Err(Err(e)),
        Err(PargError::ParseWire { offset, .. }) => {
            match artifact.parse_shape_with_compatibility(entry, text, Policy::Parslet) {
                Err(PargError::ParseWire { offset: causal, .. }) => Err(Ok(Some(causal))),
                _ => Err(Ok(Some(offset))),
            }
        }
        Err(e) => Err(Err(e)),
    }
}

/// Parses `input` as pma and builds the typed `PmaDocument`.
pub fn parse_model(input: &str, decimal: &str) -> Result<Amath_Model_PmaDocument, PmaError> {
    if decimal != "." {
        return Err(PmaError::UnsupportedDecimal(decimal.to_string()));
    }
    let pma = pma()?;
    let tree = match shape(&pma.artifact, ENTRY, input) {
        Ok(tree) => tree,
        Err(Err(e)) => return Err(PmaError::Resource(e)),
        Err(Ok(offset)) => return Err(refusal(input, offset, refusal_category(input))),
    };
    check_units(pma, &tree, input)?;
    let mut bound = pma
        .bindings
        .apply(&pma.artifact, ENTRY, &tree)
        .map_err(|e| PmaError::Artifact(e.to_string()))?;
    if let Some(term) = bound.get_mut("term") {
        resolve_quantities(pma, term);
    }
    let mut wire = Map::new();
    if let Some(term) = bound.get("term") {
        wire.insert("term".into(), term.clone());
    }
    json_codecs::read_Amath_Model_PmaDocument_json(&Value::Object(wire).to_string())
        .map_err(PmaError::Codec)
}

pub fn to_portable_json(model: &Amath_Model_PmaDocument) -> Result<String, PmaError> {
    json_codecs::write_Amath_Model_PmaDocument_json(model).map_err(PmaError::Codec)
}

/// Byte offset of a raw-tree slice in `text`.
fn slice_start(slice: &Value) -> Option<usize> {
    slice
        .get("offset")
        .and_then(Value::as_u64)
        .map(|o| o as usize)
}
fn slice_text(slice: &Value) -> Option<&str> {
    slice.get("value").and_then(Value::as_str)
}

/// Every `unit` capture of a raw tree ({literal} or {empty_list}).
fn units_in<'a>(node: &'a Value, out: &mut Vec<&'a Map<String, Value>>) {
    match node {
        Value::Object(map) => {
            for (key, value) in map {
                match (key.as_str(), value) {
                    ("unit", Value::Object(unit)) => out.push(unit),
                    _ => units_in(value, out),
                }
            }
        }
        Value::Array(items) => items.iter().for_each(|v| units_in(v, out)),
        _ => {}
    }
}

/// Every unit(...) text must be a non-empty asciiunits unit expression, and a
/// quantity assertion after it must name a unitsdb quantity.
fn check_units(pma: &Pma, tree: &Value, text: &str) -> Result<(), PmaError> {
    let mut units = Vec::new();
    units_in(tree, &mut units);
    for unit in units {
        let slice = unit.get("literal").or_else(|| unit.get("empty_list"));
        let start = slice.and_then(slice_start);
        let literal = match unit.get("literal").and_then(slice_text) {
            Some(l) => l,
            None => return Err(refusal(text, start, "unit-expression")),
        };
        match shape(&pma.units, UNITS_ENTRY, literal) {
            Ok(_) => {}
            Err(Err(e)) => return Err(PmaError::Resource(e)),
            Err(Ok(inner)) => {
                return Err(refusal(
                    text,
                    inner.and_then(|i| start.map(|s| s + i)),
                    "unit-expression",
                ))
            }
        }
        if let Some(name) = unit.get("quantity") {
            let known = slice_text(name).is_some_and(|n| pma.quantities.contains_key(n));
            if !known {
                return Err(refusal(text, slice_start(name), "quantity-assertion"));
            }
        }
    }
    Ok(())
}

/// Replaces the quantity name of every Unit of the bound term by the unitsml id
/// it resolves to (`check_units` has refused unknown names).
fn resolve_quantities(pma: &Pma, term: &mut Value) {
    let paths: [&[&str]; 3] = [
        &["unit"],
        &["uncertainty", "unit"],
        &["diff", "uncertainty", "unit"],
    ];
    for path in paths {
        let mut node = Some(&mut *term);
        for key in path {
            node = node.and_then(|n| n.get_mut(*key));
        }
        if let Some(Value::Object(unit)) = node {
            if let Some(Value::String(name)) = unit.get("quantity") {
                if let Some(id) = pma.quantities.get(name).cloned() {
                    unit.insert("quantity".into(), id);
                }
            }
        }
    }
}

// ---- Refusal categories: a port of Amath::Pma::Refusal (first matching rule). ----

fn is_alpha(c: Option<char>) -> bool {
    c.is_some_and(|c| c.is_ascii_alphabetic())
}
fn is_ws(c: char) -> bool {
    matches!(c, ' ' | '\t' | '\n' | '\r' | '\x0b' | '\x0c')
}
const CALL_WORDS: [&str; 5] = ["unit", "ustd", "uexp", "ucov", "diff"];

/// Positions (char index) where a call word starts, not preceded by a letter.
fn call_words(chars: &[char]) -> Vec<(usize, usize)> {
    let mut out = Vec::new();
    for i in 0..chars.len() {
        if i > 0 && is_alpha(Some(chars[i - 1])) {
            continue;
        }
        for w in CALL_WORDS {
            let wc: Vec<char> = w.chars().collect();
            if chars[i..].starts_with(&wc) {
                out.push((i, i + wc.len()));
            }
        }
    }
    out
}
/// A standalone word (no letter before or after).
fn has_word(chars: &[char], word: &str) -> bool {
    let wc: Vec<char> = word.chars().collect();
    (0..chars.len()).any(|i| {
        chars[i..].starts_with(&wc)
            && !(i > 0 && is_alpha(Some(chars[i - 1])))
            && !is_alpha(chars.get(i + wc.len()).copied())
    })
}
fn skip_ws(chars: &[char], mut i: usize) -> usize {
    while i < chars.len() && is_ws(chars[i]) {
        i += 1;
    }
    i
}
/// `\d\(WS*\d` — a concise uncertainty such as `1.23(4)`.
fn concise(chars: &[char]) -> bool {
    (0..chars.len().saturating_sub(1)).any(|i| {
        chars[i].is_ascii_digit()
            && chars[i + 1] == '('
            && chars
                .get(skip_ws(chars, i + 2))
                .is_some_and(|c| c.is_ascii_digit())
    })
}

pub fn refusal_category(text: &str) -> &'static str {
    let chars: Vec<char> = text.chars().collect();
    if has_word(&chars, "quantity") {
        return "reserved-word";
    }
    if text.contains("::") {
        return "quantity-assertion";
    }
    if concise(&chars) {
        return "concise";
    }
    let bare: Vec<char> = mask_units(&chars);
    let s: String = bare.iter().collect();
    if s.contains("+-") || s.contains('±') || has_word(&bare, "pm") {
        return "plus-minus-spelling";
    }
    if adjacency(&bare) {
        return "adjacency";
    }
    if call_words(&bare)
        .iter()
        .any(|&(_, end)| !matches!(bare.get(end), Some(c) if c.is_ascii_alphabetic() || *c == '('))
    {
        return "reserved-word";
    }
    if s.contains('~') {
        return "distribution-tag";
    }
    if attachment(&bare, &s) {
        return "unit-attachment";
    }
    {
        let i = skip_ws(&bare, 0);
        if bare[i..].starts_with(&['d', 'i', 'f', 'f', '(']) && !s.contains("unit(") {
            return "no-production";
        }
    }
    if close_then_e(&bare, true) {
        return "shared-exponent";
    }
    match constructor(&bare) {
        Some(Ctor::Arity) => return "constructor-arity",
        Some(Ctor::Argument) => return "constructor-argument",
        None => {}
    }
    if (0..bare.len().saturating_sub(1))
        .any(|i| bare[i].is_ascii_digit() && matches!(bare[i + 1], 'e' | 'E'))
    {
        return "shared-exponent";
    }
    if s.contains('%') {
        return "relative";
    }
    if s.contains("+/-") {
        return "measured-literal";
    }
    if s.contains("unit(") {
        return "unit-expression";
    }
    "no-production"
}

/// `\)WS*e` (any_ws) or `\)WS+e[+-]?\d` (adjacency form, any_ws = false).
fn close_then_e(bare: &[char], any_ws: bool) -> bool {
    (0..bare.len()).any(|i| {
        if bare[i] != ')' {
            return false;
        }
        let j = skip_ws(bare, i + 1);
        if any_ws {
            return bare.get(j) == Some(&'e');
        }
        if j == i + 1 || bare.get(j) != Some(&'e') {
            return false;
        }
        let mut k = j + 1;
        if matches!(bare.get(k), Some('+' | '-')) {
            k += 1;
        }
        bare.get(k).is_some_and(|c| c.is_ascii_digit())
    })
}

fn adjacency(bare: &[char]) -> bool {
    let word_space_paren = call_words(bare).iter().any(|&(_, end)| {
        let j = skip_ws(bare, end);
        j > end && bare.get(j) == Some(&'(')
    });
    let plus_slash = (0..bare.len()).any(|i| {
        (bare[i] == '+' && {
            let j = skip_ws(bare, i + 1);
            j > i + 1 && bare.get(j) == Some(&'/')
        }) || (bare[i] == '/' && {
            let j = skip_ws(bare, i + 1);
            j > i + 1 && bare.get(j) == Some(&'-')
        })
    });
    word_space_paren || close_then_e(bare, false) || plus_slash
}

enum Ctor {
    Arity,
    Argument,
}
struct Call {
    name: String,
    args: Vec<String>,
}
/// `\A WS* (diff\( WS*)? (ustd|uexp|ucov)\(` and its top-level arguments.
fn constructor_call(bare: &[char]) -> Option<Call> {
    let mut i = skip_ws(bare, 0);
    if bare[i..].starts_with(&['d', 'i', 'f', 'f', '(']) {
        let j = skip_ws(bare, i + 5);
        if ["ustd(", "uexp(", "ucov("]
            .iter()
            .any(|w| bare[j..].starts_with(&w.chars().collect::<Vec<_>>()))
        {
            i = j;
        }
    }
    let name = ["ustd", "uexp", "ucov"]
        .into_iter()
        .find(|w| bare[i..].starts_with(&format!("{w}(").chars().collect::<Vec<_>>()))?;
    let mut args = vec![String::new()];
    let mut depth = 0i32;
    for &c in &bare[i + 5..] {
        if c == ')' && depth == 0 {
            break;
        }
        if c == '(' {
            depth += 1;
        }
        if c == ')' {
            depth -= 1;
        }
        if c == ',' && depth == 0 {
            args.push(String::new());
        } else {
            args.last_mut().unwrap().push(c);
        }
    }
    Some(Call {
        name: name.to_string(),
        args,
    })
}
fn constructor(bare: &[char]) -> Option<Ctor> {
    let call = constructor_call(bare)?;
    let arity: &[usize] = match call.name.as_str() {
        "ustd" => &[2],
        "uexp" => &[3, 4],
        _ => &[4, 5],
    };
    Some(if arity.contains(&call.args.len()) {
        Ctor::Argument
    } else {
        Ctor::Arity
    })
}
/// Two units, a unit inside a constructor call, or a unit after a plain number.
fn attachment(bare: &[char], s: &str) -> bool {
    let units = s.matches("unit(").count();
    if units > 1 {
        return true;
    }
    if units == 0 {
        return false;
    }
    if let Some(call) = constructor_call(bare) {
        if call.args.iter().any(|a| a.contains("unit(")) {
            return true;
        }
    }
    let before = s[..s.find("unit(").unwrap()]
        .trim_end_matches([' ', '\t', '\n', '\r', '\x0b', '\x0c', '\0']);
    let b: Vec<char> = before.chars().collect();
    if !b.last().is_some_and(|c| c.is_ascii_digit()) {
        return false;
    }
    // !before.match?(/\)e[+-]?\d+\z/)
    let mut k = b.len();
    while k > 0 && b[k - 1].is_ascii_digit() {
        k -= 1;
    }
    if k > 0 && matches!(b[k - 1], '+' | '-') {
        k -= 1;
    }
    !(k >= 2 && b[k - 1] == 'e' && b[k - 2] == ')')
}

/// `text` with the content of every unit(...) replaced by `u`, so unit names
/// such as `pm` (picometre) match no other rule.
fn mask_units(text: &[char]) -> Vec<char> {
    let pat: Vec<char> = "unit(".chars().collect();
    let mut out = Vec::with_capacity(text.len());
    let mut i = 0;
    loop {
        let Some(j) = (i..text.len()).find(|&j| text[j..].starts_with(&pat)) else {
            break;
        };
        out.extend_from_slice(&text[i..j]);
        out.extend_from_slice(&pat);
        let mut k = j + 5;
        let mut depth = 0i32;
        while k < text.len() {
            let c = text[k];
            if c == ')' && depth == 0 {
                break;
            }
            if c == '(' {
                depth += 1;
            }
            if c == ')' {
                depth -= 1;
            }
            out.push('u');
            k += 1;
        }
        i = k;
    }
    out.extend_from_slice(&text[i.min(text.len())..]);
    out
}
