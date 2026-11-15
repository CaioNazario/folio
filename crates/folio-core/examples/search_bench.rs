//! Spec 06: how long a search by name takes in a large workspace.
//!
//! Usage: `cargo run --release -p folio-core --example search_bench -- <base dir> <documents>...`
//!
//! For each count, generates a workspace under `<base dir>` with that many
//! small text documents, named from a list of accented words so queries hit
//! many of them, indexes it, opens some of them, then runs each query several
//! times and reports the slowest run. The goal is under 50 ms per search.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};

use folio_core::clock::SystemClock;
use folio_core::db::GlobalDb;
use folio_core::documents::Format;
use folio_core::ids::UuidV7Ids;
use folio_core::indexing::ScanOptions;
use folio_core::integrity::SystemTrash;
use folio_core::paths::RelativePath;
use folio_core::search::{ProjectFilter, SearchQuery};
use folio_core::services::Services;
use folio_core::workspace::{FolioDirStore, Workspace};

const FILES_PER_FOLDER: usize = 100;
const RUNS: usize = 20;
const WORDS: [&str; 8] = [
    "Relatório",
    "Trabalho",
    "Anotações",
    "Contrato",
    "Fatura",
    "Currículo",
    "Prova",
    "Resumo",
];

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let (base, counts) = args
        .split_first()
        .expect("usage: search_bench <base dir> <documents>...");
    for count in counts {
        let count: usize = count.parse().expect("document count");
        bench(Path::new(base), count);
    }
}

fn bench(base: &Path, documents: usize) {
    let home = base.join(format!("search-home-{documents}"));
    let root = home.join("Folio");
    let _ = fs::remove_dir_all(&home);
    generate(&root, documents);
    let services = services(&home);
    let workspace = Workspace::open(&root, &services).expect("open workspace");
    let options = ScanOptions {
        remove_temp_files: true,
    };
    workspace.full_scan(options, &mut |_| {}).expect("scan");
    open_some(&workspace);
    println!("\n== {documents} documents (slowest of {RUNS} runs)");
    for (label, query) in queries() {
        report(&workspace, label, &query);
    }
    drop(workspace);
    let _ = fs::remove_dir_all(&home);
}

fn queries() -> Vec<(&'static str, SearchQuery)> {
    let text = |text: &str| SearchQuery {
        text: text.to_string(),
        ..SearchQuery::default()
    };
    let mut queries = vec![
        ("one letter (matches all)", text("a")),
        ("prefix of a word", text("relat")),
        ("two words", text("trab 7")),
        ("no match", text("zzzz")),
    ];
    queries.extend(filter_queries());
    queries
}

fn filter_queries() -> [(&'static str, SearchQuery); 2] {
    let project = RelativePath::from_bytes("Projeto 3").expect("path");
    let markdown = SearchQuery {
        formats: vec![Format::Md],
        ..SearchQuery::default()
    };
    let in_project = SearchQuery {
        text: "prova".to_string(),
        project: Some(ProjectFilter::Project(project)),
        ..SearchQuery::default()
    };
    [
        ("filter only (format)", markdown),
        ("text + project", in_project),
    ]
}

fn report(workspace: &Workspace, label: &str, query: &SearchQuery) {
    let mut slowest = Duration::ZERO;
    let mut found = 0;
    for _ in 0..RUNS {
        let start = Instant::now();
        found = workspace.search(query).expect("search").len();
        slowest = slowest.max(start.elapsed());
    }
    println!("{label:<28} {slowest:>10.1?}  ({found} results)");
}

/// Opens one document in fifty, so the recency ordering has work to do.
fn open_some(workspace: &Workspace) {
    let all = workspace
        .search(&SearchQuery {
            formats: vec![Format::Txt],
            ..SearchQuery::default()
        })
        .expect("search");
    for doc in all.iter().step_by(50) {
        workspace.record_opened(doc.id).expect("record opened");
    }
}

fn services(home: &Path) -> Services {
    let clock = Arc::new(SystemClock);
    let ids = Arc::new(UuidV7Ids);
    let global =
        GlobalDb::open(&home.join(".global.db"), clock.clone(), ids.clone()).expect("global db");
    Services {
        home: home.to_path_buf(),
        global,
        store: Box::new(FolioDirStore),
        clock,
        ids,
        trash: Arc::new(SystemTrash),
    }
}

/// Text documents spread over projects of 10 folders each.
fn generate(root: &Path, documents: usize) {
    for index in 0..documents {
        let dir = folder_of(root, index);
        fs::create_dir_all(&dir).expect("create folder");
        fs::write(dir.join(name_of(index)), b"Texto\n").expect("write document");
    }
}

fn name_of(index: usize) -> String {
    let word = WORDS[index % WORDS.len()];
    let extension = if index.is_multiple_of(2) { "txt" } else { "md" };
    format!("{word} {index} v{}.{extension}", index % 10)
}

fn folder_of(root: &Path, index: usize) -> PathBuf {
    let folder = index / FILES_PER_FOLDER;
    root.join(format!("Projeto {}", folder / 10))
        .join(format!("Pasta {}", folder % 10))
}
