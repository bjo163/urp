//! Clock abstraction for deterministic business logic and tests.

use chrono::{DateTime, Utc};

/// Supplies the current UTC instant.
pub trait Clock: Send + Sync {
    /// Returns the current UTC instant.
    fn now_utc(&self) -> DateTime<Utc>;
}

/// Production clock backed by the system UTC clock.
#[derive(Clone, Copy, Debug, Default)]
pub struct SystemClock;

impl Clock for SystemClock {
    fn now_utc(&self) -> DateTime<Utc> {
        Utc::now()
    }
}
