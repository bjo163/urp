//! Versioned domain events and transactional outbox persistence.
//!
//! This crate writes event rows using a caller-owned SQL transaction. It does
//! not publish, claim, retry, or perform external side effects; the worker
//! dispatcher owns those responsibilities.

mod outbox;

pub use outbox::{
    AppendOutcome, EventActor, EventEnvelope, EventRecord, EventStatus, OutboxError,
    OutboxRepository,
};
