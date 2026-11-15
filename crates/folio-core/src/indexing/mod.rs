//! Keeping the index in step with the disk (spec 05). The disk is always the
//! truth; the index only catches up with it.

mod batch;
mod disk;
pub(crate) mod fs_id;
mod hashing;
mod purge;
mod reconcile;
mod reconnect;
mod scan;

pub use purge::MISSING_RETENTION_MS;
pub use reconcile::ReconcileSummary;
pub use scan::{ScanOptions, ScanProgress};
