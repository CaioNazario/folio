//! Time is injected so tests never depend on the real clock.

use std::time::{SystemTime, UNIX_EPOCH};

/// Source of the current time, in milliseconds since the Unix epoch (UTC).
pub trait Clock: Send + Sync {
    fn now_millis(&self) -> i64;
}

/// The operating system clock.
pub struct SystemClock;

impl Clock for SystemClock {
    fn now_millis(&self) -> i64 {
        let elapsed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default();
        i64::try_from(elapsed.as_millis()).unwrap_or(i64::MAX)
    }
}
