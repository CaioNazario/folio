//! Folio business logic: workspace, documents, conversion and index.
//!
//! This crate does not depend on Tauri and is testable without running the app.

pub mod bridge_error;
pub mod clock;
pub mod db;
pub mod display;
pub mod documents;
mod error;
pub mod favorites;
pub mod ids;
pub mod indexing;
pub mod integrity;
pub mod model;
pub mod paths;
pub mod preferences;
pub mod readers;
pub mod recents;
pub mod search;
pub mod services;
#[cfg(test)]
mod testing;
pub mod workspace;

pub use error::{Error, Result};
