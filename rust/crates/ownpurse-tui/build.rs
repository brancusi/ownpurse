//! The build identity: the git commit this TUI was built from, plus `.dirty` when the work tree had
//! uncommitted changes (`OWNPURSE_BUILD_ID`, e.g. `g6dc723f` / `g6dc723f.dirty`), so two dev builds that share
//! a version can tell each other apart (`ui upgrade`). A set `OWNPURSE_BUILD_ID` wins (tests, packaging);
//! no git (a source tarball): none, and the version alone identifies the build.

use std::process::Command;

fn git(args: &[&str]) -> Option<String> {
    let out = Command::new("git").args(args).output().ok()?;
    out.status
        .success()
        .then(|| String::from_utf8_lossy(&out.stdout).trim().to_string())
}

fn main() {
    println!("cargo:rerun-if-env-changed=OWNPURSE_BUILD_ID");
    if let Ok(id) = std::env::var("OWNPURSE_BUILD_ID") {
        println!("cargo:rustc-env=OWNPURSE_BUILD_ID={id}");
        return;
    }
    // rebuild when the commit or the index moves, and when this crate's or the core's sources change
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
    let Some(sha) = git(&["rev-parse", "--short=7", "HEAD"]) else {
        return;
    };
    let dirty = git(&["status", "--porcelain", "--untracked-files=no", "--", ".."])
        .is_some_and(|s| !s.is_empty());
    println!(
        "cargo:rustc-env=OWNPURSE_BUILD_ID=g{sha}{}",
        if dirty { ".dirty" } else { "" }
    );
}
