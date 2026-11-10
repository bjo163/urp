//! HTTP request extractors that preserve the canonical URP error contract.

pub mod validated_json;

pub use validated_json::{ValidateRequest, ValidatedJson};
