//! Runs every conversion over a corpus of real documents (spec 08,
//! "Critério mínimo de qualidade"):
//!
//! ```text
//! cargo run -p folio-core --example conversion_corpus -- <corpus dir> <output dir>
//! ```
//!
//! The corpus is copied into `<output dir>/Folio`, a new workspace, and each
//! document is converted to the three other formats there. One line per
//! conversion goes to stdout: `ok`, or the error. `scripts/check-derivatives.sh`
//! then opens the derivatives in the reference programs.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use folio_core::clock::SystemClock;
use folio_core::db::GlobalDb;
use folio_core::documents::{Document, Format};
use folio_core::ids::UuidV7Ids;
use folio_core::integrity::Trash;
use folio_core::paths::RelativePath;
use folio_core::services::Services;
use folio_core::workspace::{FolioDirStore, Workspace};

struct NoTrash;

impl Trash for NoTrash {
    fn send(&self, path: &Path) -> Result<(), String> {
        Err(format!("refusing to trash {}", path.display()))
    }
}

fn main() {
    let args: Vec<PathBuf> = std::env::args_os().skip(1).map(PathBuf::from).collect();
    let [corpus, output] = args.as_slice() else {
        eprintln!("usage: conversion_corpus <corpus dir> <output dir>");
        std::process::exit(2);
    };
    let root = output.join("Folio");
    fs::create_dir_all(&root).expect("cannot create the output workspace");
    let documents = copy_corpus(corpus, &root);
    let workspace = open_workspace(output, &root);
    for name in documents {
        convert_all(&workspace, &name);
    }
}

/// Copies the files of `corpus` (not its subdirectories) into `root`.
fn copy_corpus(corpus: &Path, root: &Path) -> Vec<String> {
    let entries = fs::read_dir(corpus).expect("cannot read the corpus");
    let files = entries.flatten().filter(|entry| entry.path().is_file());
    let mut names = Vec::new();
    for entry in files {
        let name = entry.file_name().to_string_lossy().into_owned();
        fs::copy(entry.path(), root.join(&name)).expect("cannot copy a corpus file");
        names.push(name);
    }
    names.sort();
    names
}

fn open_workspace(home: &Path, root: &Path) -> Workspace {
    let (clock, ids) = (Arc::new(SystemClock), Arc::new(UuidV7Ids));
    let global = GlobalDb::open(&home.join("global.db"), clock.clone(), ids.clone());
    let services = Services {
        home: home.to_path_buf(),
        global: global.expect("cannot open the global database"),
        store: Box::new(FolioDirStore),
        clock,
        ids,
        trash: Arc::new(NoTrash),
    };
    Workspace::open(root, &services).expect("cannot open the output workspace")
}

/// Converts the document `name` to every other format, printing each result.
fn convert_all(workspace: &Workspace, name: &str) {
    let Some(original) = indexed(workspace, name) else {
        return;
    };
    for target in Format::ALL.into_iter().filter(|f| *f != original.format) {
        let result = workspace.convert(original.id, target, None);
        let outcome = result.map_or_else(|e| format!("FAILED: {e}"), |d| format!("ok {}", d.name));
        println!("{name} -> {}: {outcome}", target.as_str());
    }
}

/// The document `name` as indexed; `None`, with a note, when it is not one.
fn indexed(workspace: &Workspace, name: &str) -> Option<Document> {
    let path = RelativePath::from_bytes(name.as_bytes().to_vec()).ok()?;
    let document = workspace.index_file(&path).ok().flatten();
    if document.is_none() {
        println!("{name}: not a document, skipped");
    }
    document
}
