//! Folio business logic: workspace, documents, conversion and index.
//!
//! This crate does not depend on Tauri and is testable without running the app.

pub mod clock;
pub mod db;
pub mod documents;
mod error;
pub mod ids;
pub mod integrity;
pub mod paths;
pub mod services;
#[cfg(test)]
mod testing;
pub mod workspace;

pub use error::{Error, Result};
