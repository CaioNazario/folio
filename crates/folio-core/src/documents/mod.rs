//! Documents: files of the workspace that Folio recognizes (spec 04).

mod conversions;
mod counts;
pub(crate) mod file_facts;
mod format;
pub(crate) mod index;
mod ops;
mod signature;
#[cfg(test)]
pub(crate) mod test_docs;

pub use conversions::ConversionLink;
pub use counts::WorkspaceTotals;
pub use format::{Format, is_ignored_name};
pub use index::Document;
pub use signature::{Signature, check_signature};
