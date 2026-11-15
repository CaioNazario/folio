//! Folio business logic: workspace, documents, conversion and index.
//!
//! This crate does not depend on Tauri and is testable without running the app.

mod error;
pub mod integrity;

pub use error::{Error, Result};
