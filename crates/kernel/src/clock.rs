//! Clock abstraction for deterministic business logic and tests.

use std::sync::{Arc, RwLock};

use chrono::{DateTime, Duration, Utc};
use thiserror::Error;

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

/// A deterministic, thread-safe clock intended for tests and simulations.
#[derive(Clone, Debug)]
pub struct FixedClock {
    instant: Arc<RwLock<DateTime<Utc>>>,
}

/// Errors from mutating a deterministic clock.
#[derive(Debug, Error, Clone, Copy, PartialEq, Eq)]
pub enum ClockError {
    /// Advancing the clock would exceed the supported date-time range.
    #[error("clock advance exceeds supported date-time range")]
    Overflow,
}

impl FixedClock {
    /// Creates a fixed clock at the supplied UTC instant.
    pub fn new(instant: DateTime<Utc>) -> Self {
        Self {
            instant: Arc::new(RwLock::new(instant)),
        }
    }

    /// Replaces the current instant with an explicit UTC instant.
    pub fn set(&self, instant: DateTime<Utc>) {
        let mut current = self
            .instant
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        *current = instant;
    }

    /// Advances the clock by a signed duration.
    pub fn advance_by(&self, duration: Duration) -> Result<DateTime<Utc>, ClockError> {
        let mut current = self
            .instant
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let next = current
            .checked_add_signed(duration)
            .ok_or(ClockError::Overflow)?;
        *current = next;
        Ok(next)
    }
}

impl Clock for FixedClock {
    fn now_utc(&self) -> DateTime<Utc> {
        *self
            .instant
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}

#[cfg(test)]
mod tests {
    use chrono::{Duration, TimeZone, Utc};

    use super::{Clock, ClockError, FixedClock, SystemClock};

    #[test]
    fn fixed_clock_can_be_frozen_and_advanced() {
        let start = Utc.with_ymd_and_hms(2026, 11, 11, 0, 0, 0).single();
        assert!(start.is_some());
        let start = start.unwrap_or_else(Utc::now);
        let clock = FixedClock::new(start);

        assert_eq!(clock.now_utc(), start);
        let advanced = clock.advance_by(Duration::minutes(15));
        assert!(advanced.is_ok());
        assert_eq!(clock.now_utc(), start + Duration::minutes(15));
    }

    #[test]
    fn fixed_clock_can_be_reset() {
        let first = Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).single();
        let second = Utc.with_ymd_and_hms(2026, 2, 1, 0, 0, 0).single();
        assert!(first.is_some() && second.is_some());
        let clock = FixedClock::new(first.unwrap_or_else(Utc::now));
        let next = second.unwrap_or_else(Utc::now);

        clock.set(next);
        assert_eq!(clock.now_utc(), next);
    }

    #[test]
    fn fixed_clock_reports_overflow() {
        let clock = FixedClock::new(DateTimeMax::value());
        assert_eq!(
            clock.advance_by(Duration::seconds(1)),
            Err(ClockError::Overflow)
        );
    }

    #[test]
    fn system_clock_returns_utc() {
        let now = SystemClock.now_utc();
        assert_eq!(now.offset(), &Utc);
    }

    struct DateTimeMax;

    impl DateTimeMax {
        fn value() -> chrono::DateTime<Utc> {
            chrono::DateTime::<Utc>::MAX_UTC
        }
    }
}
