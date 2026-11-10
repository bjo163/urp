//! HTTP middleware for request tracing and boundary metadata.

mod trace;

pub use trace::{RequestTrace, attach_trace_context};
