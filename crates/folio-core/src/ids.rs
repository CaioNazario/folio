//! Id generation is injected so tests get predictable ids.

use uuid::Uuid;

/// Source of new stable ids (UUID v7 in production).
pub trait IdGenerator: Send + Sync {
    fn new_id(&self) -> Uuid;
}

/// Time-ordered random UUID v7 ids.
pub struct UuidV7Ids;

impl IdGenerator for UuidV7Ids {
    fn new_id(&self) -> Uuid {
        Uuid::now_v7()
    }
}
