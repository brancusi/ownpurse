//! Shared by the binary's tests: a log in a temporary folder, driven through the real `ownpurse-facts` binary.
#![allow(dead_code)]

pub use serde_json::Value as Json;
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn samples() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../samples")
}

pub struct Log {
    _dir: tempfile::TempDir,
    pub path: PathBuf,
}

impl Log {
    pub fn new() -> Log {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("books.sqlite");
        Log { _dir: dir, path }
    }

    pub fn run(&self, args: &[&str]) -> (bool, String, String) {
        let out = Command::new(env!("CARGO_BIN_EXE_ownpurse-facts"))
            .arg("--log")
            .arg(&self.path)
            .args(["--at", "2026-04-02T10:00:00Z"])
            .args(args)
            .output()
            .unwrap();
        (
            out.status.success(),
            String::from_utf8_lossy(&out.stdout).into_owned(),
            String::from_utf8_lossy(&out.stderr).into_owned(),
        )
    }

    pub fn ok(&self, args: &[&str]) -> String {
        let (ok, out, err) = self.run(args);
        assert!(ok, "{args:?} failed: {err}");
        out
    }

    pub fn json(&self, args: &[&str]) -> Json {
        let mut a = vec!["--json"];
        a.extend_from_slice(args);
        serde_json::from_str(&self.ok(&a)).unwrap()
    }

    /// A manifest with just the Xero files, written next to the samples so its paths resolve.
    pub fn load_xero(&self) {
        let full: Json =
            serde_json::from_str(&std::fs::read_to_string(samples().join("ingest.json")).unwrap())
                .unwrap();
        let files: Vec<Json> = full["files"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|f| f["source"] == "xero:tallowmere")
            .map(|f| {
                let mut f = f.clone();
                let p = samples().join(f["file"].as_str().unwrap());
                f["file"] = Json::String(p.display().to_string());
                f
            })
            .collect();
        let m = serde_json::json!({"sources": full["sources"], "files": files});
        let path = self.path.with_extension("manifest.json");
        std::fs::write(&path, m.to_string()).unwrap();
        self.ok(&["load", path.to_str().unwrap()]);
    }
}

impl Log {
    /// The whole month: an empty log, every file in `samples/ingest.json`, then the link rules.
    pub fn month() -> Log {
        let log = Log::new();
        log.ok(&["init"]);
        log.ok(&["load", sample("ingest.json").as_str()]);
        log.ok(&["link"]);
        log
    }

    /// Rows of a query file in `samples/queries/`.
    pub fn query(&self, file: &str) -> Vec<Json> {
        let out = self.json(&["sql", "-f", sample(&format!("queries/{file}")).as_str()]);
        out["rows"].as_array().unwrap().clone()
    }
}

/// A path under `samples/`, as text.
pub fn sample(rel: &str) -> String {
    samples().join(rel).display().to_string()
}
