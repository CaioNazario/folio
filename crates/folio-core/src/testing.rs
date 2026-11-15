//! Fakes shared by the unit tests.

use std::sync::atomic::{AtomicI64, AtomicU64, Ordering};

use uuid::Uuid;

use crate::clock::Clock;
use crate::ids::IdGenerator;

/// A clock that only moves when told to.
pub struct ManualClock(AtomicI64);

impl ManualClock {
    pub fn at(millis: i64) -> Self {
        Self(AtomicI64::new(millis))
    }

    pub fn set(&self, millis: i64) {
        self.0.store(millis, Ordering::SeqCst);
    }
}

impl Clock for ManualClock {
    fn now_millis(&self) -> i64 {
        self.0.load(Ordering::SeqCst)
    }
}

/// Ids 1, 2, 3... as UUIDs.
#[derive(Default)]
pub struct SequentialIds(AtomicU64);

impl IdGenerator for SequentialIds {
    fn new_id(&self) -> Uuid {
        Uuid::from_u128(u128::from(self.0.fetch_add(1, Ordering::SeqCst) + 1))
    }
}
