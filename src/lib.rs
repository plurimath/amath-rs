//! Parse shared PARG artifacts to raw stage trees. No typed model or writer API.
#[cfg(feature = "ffi")]
pub mod ffi;
mod languages;
#[cfg(feature="models")]
#[rustfmt::skip]
pub mod generated;
mod replay;
#[cfg(feature = "models")]
pub mod typed;
use parsanol::parg::bindings_artifact::PargBindingsArtifact;
use parsanol::{PargArtifact, PargError};
use serde_json::Value;
use std::{collections::HashMap, fmt};

pub const LANGUAGE: &str = "asciimath";
pub const ENGINE: &str = "parsanol-rust-native/0.20.7";

#[derive(Debug)]
pub enum Error {
    Runtime(PargError),
    UnsupportedDecimal(String),
    NoFormula,
    MissingCapture(&'static str),
    InvalidWrappers(serde_json::Error),
}
impl From<PargError> for Error {
    fn from(e: PargError) -> Self {
        Self::Runtime(e)
    }
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Runtime(e) => write!(f, "{e}"),
            Self::UnsupportedDecimal(s) => write!(f, "unsupported decimal marker {s:?}"),
            Self::NoFormula => write!(f, "input has no formula"),
            Self::MissingCapture(s) => write!(f, "artifact lacks capture {s}"),
            Self::InvalidWrappers(e) => write!(f, "invalid wrapper table: {e}"),
        }
    }
}
impl std::error::Error for Error {}
impl Error {
    /// UTF-8 byte offset in the preprocessed grammar input; never caller-coordinate remapping.
    pub fn byte_offset(&self) -> Option<usize> {
        match self {
            Self::Runtime(PargError::ParseWire { offset, .. }) => Some(*offset),
            _ => None,
        }
    }
}

/// Raw tree slices refer to `parsed_input`, which may differ from the caller input.
#[derive(Debug, Clone)]
pub struct Parsed {
    pub input: String,
    pub parsed_input: String,
    pub decimal: String,
    pub tree: Value,
    /// Declared Rust binding output; a bound JSON map, not a typed Lutaml model.
    pub bound: Value,
}
impl Parsed {
    /// Canonical tagged stage JSON, preserving UTF-8 byte and character offsets.
    pub fn canonical(&self) -> Value {
        let language = languages::stage_language(LANGUAGE);
        let renames = language
            .renames
            .iter()
            .map(|(a, b)| (a.to_string(), b.to_string()))
            .collect();
        replay::dump_canonical(&self.tree, &self.parsed_input, &renames)
    }
}

/// Reusable verified grammar and declared Rust bindings. Construction checks artifact checksums.
pub struct Parser {
    artifact: PargArtifact,
    bindings: PargBindingsArtifact,
    wrappers: Option<HashMap<String, String>>,
}
impl Parser {
    pub fn new() -> Result<Self, Error> {
        let artifact = PargArtifact::from_json(include_str!("../artifacts/asciimath.json"))?;
        let bindings = PargBindingsArtifact::from_json(
            include_str!("../artifacts/asciimath.bindings.rs.json"),
            Some("rs"),
        )?;
        let wrappers = Some(
            serde_json::from_value(
                serde_json::from_str::<Value>(include_str!("../artifacts/asciimath.wrappers.json"))
                    .map_err(Error::InvalidWrappers)?["wrappers"]
                    .clone(),
            )
            .map_err(Error::InvalidWrappers)?,
        );
        Ok(Self {
            artifact,
            bindings,
            wrappers,
        })
    }
    pub fn parse(&self, input: &str) -> Result<Parsed, Error> {
        self.parse_with_decimal(input, ".")
    }
    pub fn parse_with_decimal(&self, input: &str, decimal: &str) -> Result<Parsed, Error> {
        self.parse_with_policy(
            input,
            decimal,
            parsanol::portable::parser::ParserCompatibility::Strict,
        )
    }
    /// Legacy stage policy, preserving the producer's diagnostic boundary.
    /// Error offsets refer to preprocessed input, like the raw Ruby producer.
    pub fn parse_legacy(&self, input: &str) -> Result<Parsed, Error> {
        self.parse_legacy_with_decimal(input, ".")
    }
    pub fn parse_legacy_with_decimal(&self, input: &str, decimal: &str) -> Result<Parsed, Error> {
        let native = self.parse_with_decimal(input, decimal);
        if matches!(&native, Err(Error::Runtime(PargError::ParseWire { .. }))) {
            let causal = self.parse_with_policy(
                input,
                decimal,
                parsanol::portable::parser::ParserCompatibility::Parslet,
            );
            if matches!(&causal, Err(Error::Runtime(PargError::ParseWire { .. }))) {
                return causal;
            }
        }
        native
    }
    fn parse_with_policy(
        &self,
        input: &str,
        decimal: &str,
        policy: parsanol::portable::parser::ParserCompatibility,
    ) -> Result<Parsed, Error> {
        let lang = languages::stage_language(LANGUAGE);
        let entry = lang
            .decimal_entries
            .iter()
            .find(|(d, _)| *d == decimal)
            .map(|(_, e)| *e)
            .ok_or_else(|| Error::UnsupportedDecimal(decimal.to_string()))?;
        let parsed_input = match lang.preprocess {
            Some(preprocess) => preprocess(input).ok_or(Error::NoFormula)?,
            None => input.to_string(),
        };
        let shape = self
            .artifact
            .parse_shape_with_compatibility(entry, &parsed_input, policy)?;
        // Apply authoritative bindings even for the asciimath root mapping.
        let bound = self.bindings.apply(&self.artifact, entry, &shape)?;
        let raw_source = if lang.bindings { &bound } else { &shape };
        let tree = raw_source
            .get(lang.capture)
            .ok_or(Error::MissingCapture(lang.capture))?;
        let unwrapped = languages::unwrap(tree, self.wrappers.as_ref().unwrap(), &parsed_input);
        let tree = languages::renest(&unwrapped, &parsed_input);
        Ok(Parsed {
            input: input.to_string(),
            parsed_input,
            decimal: decimal.to_string(),
            tree,
            bound,
        })
    }
}
pub fn parse(input: &str) -> Result<Parsed, Error> {
    Parser::new()?.parse(input)
}

/// Parse using the stage producer's explicit legacy raw policy.
pub fn parse_legacy(input: &str) -> Result<Parsed, Error> {
    Parser::new()?.parse_legacy(input)
}
