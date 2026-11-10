//! Shared primitives for the URP platform.
//!
//! This crate intentionally starts small. Platform identifiers, request
//! context, error taxonomy and clock abstractions are introduced by their
//! dedicated foundation tasks instead of accumulating unrelated helpers here.

/// Public API version used by the Rust application boundary.
pub const API_VERSION: &str = "v1";

/// Product identifier included in service diagnostics.
pub const PRODUCT_NAME: &str = "URP";
