//! Search by file name in the open workspace (spec 06).

mod normalize;
mod query;
mod rank;
mod statement;

pub use normalize::{normalize_file_name, normalize_name};
pub use query::{MAX_RESULTS, ProjectFilter, SearchQuery};
pub(crate) use rank::has_word_starting_with;
