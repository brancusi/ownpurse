//! The files a check reads: the repository's tracked files, or every file under a folder.

use std::path::{Path, PathBuf};

/// The repository root (the folder holding `.git`).
pub fn root() -> PathBuf {
    let out = std::process::Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .expect("git");
    PathBuf::from(String::from_utf8_lossy(&out.stdout).trim())
}

/// One file to scan: the name to report and its bytes.
pub struct Item {
    pub name: String,
    pub bytes: Vec<u8>,
}

/// Every tracked file (paths relative to the root).
pub fn tracked(root: &Path) -> Vec<Item> {
    let out = std::process::Command::new("git")
        .args(["ls-files", "-z"])
        .current_dir(root)
        .output()
        .expect("git ls-files");
    String::from_utf8_lossy(&out.stdout)
        .split('\0')
        .filter(|n| !n.is_empty())
        .filter_map(|n| {
            let bytes = std::fs::read(root.join(n)).ok()?; // deleted in the work tree: skip
            Some(Item {
                name: n.to_string(),
                bytes,
            })
        })
        .collect()
}

/// Every file under `dir`, recursively (names as given plus the relative path).
pub fn under(dir: &Path) -> Vec<Item> {
    let mut out = Vec::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(p) = stack.pop() {
        if p.is_dir() {
            if let Ok(rd) = std::fs::read_dir(&p) {
                stack.extend(rd.flatten().map(|e| e.path()));
            }
        } else if let Ok(bytes) = std::fs::read(&p) {
            out.push(Item {
                name: p.display().to_string(),
                bytes,
            });
        }
    }
    out.sort_by(|a, b| a.name.cmp(&b.name));
    out
}

/// Text (UTF-8 without NUL) or binary.
pub fn text(bytes: &[u8]) -> Option<&str> {
    if bytes.contains(&0) {
        return None;
    }
    std::str::from_utf8(bytes).ok()
}

/// The 1-based line of a byte offset.
pub fn line_of(bytes: &[u8], offset: usize) -> usize {
    bytes[..offset.min(bytes.len())]
        .iter()
        .filter(|b| **b == b'\n')
        .count()
        + 1
}

/// Where a hit is: `file:line` in text, `file@offset` in a binary file.
pub fn place(item: &Item, offset: usize) -> String {
    match text(&item.bytes) {
        Some(_) => format!("{}:{}", item.name, line_of(&item.bytes, offset)),
        None => format!("{}@{offset}", item.name),
    }
}

/// A temp folder for the self-tests.
pub fn temp(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("xtask-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).expect("temp dir");
    d
}
