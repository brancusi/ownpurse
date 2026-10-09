//! The build identity, as the TUI has it (`crates/ownpurse-tui/build.rs`): `OWNPURSE_FULL_VERSION` is
//! `0.12.1+g6dc723f` (`.dirty` with uncommitted changes), or the version alone without git. A set
//! `OWNPURSE_BUILD_ID` wins. `ownpurse --version` prints it.

use std::process::Command;

fn git(args: &[&str]) -> Option<String> {
    let out = Command::new("git").args(args).output().ok()?;
    out.status
        .success()
        .then(|| String::from_utf8_lossy(&out.stdout).trim().to_string())
}

fn main() {
    let version = std::env::var("CARGO_PKG_VERSION").unwrap_or_default();
    println!("cargo:rerun-if-env-changed=OWNPURSE_BUILD_ID");
    let id = std::env::var("OWNPURSE_BUILD_ID").ok().or_else(|| {
        if let Some(dir) = git(&["rev-parse", "--git-dir"]) {
            for f in ["HEAD", "index"] {
                println!("cargo:rerun-if-changed={dir}/{f}");
            }
            if let Some(r) = git(&["symbolic-ref", "-q", "HEAD"])
                && let Some(common) = git(&["rev-parse", "--git-common-dir"])
            {
                println!("cargo:rerun-if-changed={common}/{r}");
            }
        }
        println!("cargo:rerun-if-changed=src");
        println!("cargo:rerun-if-changed=../ownpurse-core/src");
        println!("cargo:rerun-if-changed=../ownpurse-xero/src");
        let sha = git(&["rev-parse", "--short=7", "HEAD"])?;
        let dirty = git(&["status", "--porcelain", "--untracked-files=no", "--", ".."])
            .is_some_and(|s| !s.is_empty());
        Some(format!("g{sha}{}", if dirty { ".dirty" } else { "" }))
    });
    match id.filter(|i| !i.is_empty()) {
        Some(i) => println!("cargo:rustc-env=OWNPURSE_FULL_VERSION={version}+{i}"),
        None => println!("cargo:rustc-env=OWNPURSE_FULL_VERSION={version}"),
    }
}
