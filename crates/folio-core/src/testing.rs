//! Fakes shared by the unit tests.

use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::sync::atomic::{AtomicI64, AtomicU64, Ordering};

use uuid::Uuid;

use crate::clock::Clock;
use crate::ids::IdGenerator;
use crate::integrity::Trash;

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

/// A trash that records what was sent to it and deletes nothing.
#[derive(Default)]
pub struct RecordingTrash(Mutex<Vec<PathBuf>>);

impl RecordingTrash {
    pub fn sent(&self) -> Vec<PathBuf> {
        self.0.lock().unwrap().clone()
    }
}

impl Trash for RecordingTrash {
    fn send(&self, path: &Path) -> Result<(), String> {
        self.0.lock().unwrap().push(path.to_path_buf());
        Ok(())
    }
}
