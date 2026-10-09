//! Shared by the API tests: temp folders and the frozen reference artifacts.
#![allow(dead_code)]

use std::path::{Path, PathBuf};

pub fn temp_dir(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("ownpurse-xero-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

/// Committed reference artifacts (rust/tests/fixtures/frozen/): records, call logs and sync results the
/// writer, the HTTP layer and the syncer must keep reproducing byte for byte. They are never regenerated.
pub fn frozen(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures/frozen")
        .join(name)
}

/// A frozen text artifact.
pub fn frozen_text(name: &str) -> String {
    let p = frozen(name);
    std::fs::read_to_string(&p)
        .unwrap_or_else(|e| panic!("missing frozen artifact {} ({e})", p.display()))
}

/// A private copy of a frozen SQLite record at `dest`.
pub fn frozen_record(name: &str, dest: &Path) {
    let p = frozen(name);
    std::fs::copy(&p, dest)
        .unwrap_or_else(|e| panic!("missing frozen record {} ({e})", p.display()));
}
