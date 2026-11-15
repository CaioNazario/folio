//! Spike S6 (spec 05): how long a workspace takes to index.
//!
//! Usage: `cargo run --release -p folio-core --example scan_bench -- <base dir> <documents>...`
//!
//! For each count, generates a workspace under `<base dir>` with that many
//! documents (PDF, DOCX, MD and TXT in equal parts) and as many non-documents,
//! 100 files per folder, then times each stage separately. Run it on the disk
//! being measured: `/tmp` is often tmpfs, which is memory, not disk.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};

use folio_core::clock::SystemClock;
use folio_core::db::GlobalDb;
use folio_core::documents::{Format, check_signature};
use folio_core::ids::UuidV7Ids;
use folio_core::indexing::ScanOptions;
use folio_core::integrity::SystemTrash;
use folio_core::services::Services;
use folio_core::workspace::{FolioDirStore, Workspace};

const FILES_PER_FOLDER: usize = 100;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let (base, counts) = args
        .split_first()
        .expect("usage: scan_bench <base dir> <documents>...");
    for count in counts {
        let count: usize = count.parse().expect("document count");
        bench(Path::new(base), count);
    }
}

fn bench(base: &Path, documents: usize) {
    let home = base.join(format!("home-{documents}"));
    let root = home.join("Folio");
    let _ = fs::remove_dir_all(&home);
    let generated = timed(|| generate(&root, documents));
    println!("\n== {documents} documents + {documents} other files (generated in {generated:.1?})");
    report("walk + stat (std only)", timed(|| walk_and_stat(&root)));
    report("signature check only", timed(|| check_signatures(&root)));
    bench_workspace(&home, &root);
    let _ = fs::remove_dir_all(&home);
}

/// The stages that go through the core, as the app runs them on opening.
fn bench_workspace(home: &Path, root: &Path) {
    let services = services(home);
    let workspace = Workspace::open(root, &services).expect("open workspace");
    let opening = ScanOptions {
        remove_temp_files: true,
    };
    report(
        "first full scan (empty index)",
        timed(|| scan(&workspace, opening)),
    );
    report(
        "second full scan (no changes)",
        timed(|| scan(&workspace, opening)),
    );
    report("hash pass (all documents)", timed(|| hash(&workspace)));
}

fn report(stage: &str, elapsed: Duration) {
    println!("{stage:<32} {elapsed:>10.1?}");
}

fn timed(work: impl FnOnce()) -> Duration {
    let start = Instant::now();
    work();
    start.elapsed()
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

fn scan(workspace: &Workspace, options: ScanOptions) {
    let summary = workspace.full_scan(options, &mut |_| {}).expect("scan");
    println!("{:<32} {summary:?}", "");
}

fn hash(workspace: &Workspace) {
    let hashed = workspace.hash_stale_documents().expect("hash");
    println!("{:<32} hashed {hashed}", "");
}

/// Documents and non-documents spread over projects of 10 folders each.
fn generate(root: &Path, documents: usize) {
    for index in 0..documents {
        let dir = folder_of(root, index);
        fs::create_dir_all(&dir).expect("create folder");
        write_document(&dir, index);
        fs::write(dir.join(format!("foto-{index}.png")), vec![0x89; 2048]).expect("write other");
    }
}

fn folder_of(root: &Path, index: usize) -> PathBuf {
    let folder = index / FILES_PER_FOLDER;
    root.join(format!("Projeto {}", folder / 10))
        .join(format!("Pasta {}", folder % 10))
}

fn write_document(dir: &Path, index: usize) {
    let (name, content) = match index % 4 {
        0 => (format!("doc-{index}.pdf"), pdf()),
        1 => (format!("doc-{index}.docx"), docx()),
        2 => (format!("doc-{index}.md"), text(b"# Notas\n")),
        _ => (format!("doc-{index}.txt"), text(b"Texto\n")),
    };
    fs::write(dir.join(name), content).expect("write document");
}

fn pdf() -> Vec<u8> {
    [b"%PDF-1.7\n".as_slice(), &vec![b'x'; 20 * 1024]].concat()
}

fn text(start: &[u8]) -> Vec<u8> {
    [start, &vec![b'a'; 4 * 1024]].concat()
}

/// A small DOCX-shaped ZIP: the central directory is what the check reads.
fn docx() -> Vec<u8> {
    let mut writer = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
    let options =
        zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);
    for name in [
        "[Content_Types].xml",
        "word/document.xml",
        "word/styles.xml",
    ] {
        writer.start_file(name, options).expect("zip entry");
        writer
            .write_all(&vec![b'<'; 8 * 1024])
            .expect("zip content");
    }
    writer.finish().expect("zip").into_inner()
}

fn walk_and_stat(root: &Path) {
    let mut pending = vec![root.to_path_buf()];
    while let Some(dir) = pending.pop() {
        for entry in fs::read_dir(&dir).expect("read dir") {
            let path = entry.expect("entry").path();
            let metadata = fs::metadata(&path).expect("stat");
            if metadata.is_dir() && !path.ends_with(".folio") {
                pending.push(path);
            }
        }
    }
}

fn check_signatures(root: &Path) {
    let mut pending = vec![root.to_path_buf()];
    while let Some(dir) = pending.pop() {
        for entry in fs::read_dir(&dir).expect("read dir") {
            let path = entry.expect("entry").path();
            if path.is_dir() {
                pending.push(path);
            } else if let Some(format) = Format::from_name(path.file_name().expect("name")) {
                check_signature(format, &path);
            }
        }
    }
}
