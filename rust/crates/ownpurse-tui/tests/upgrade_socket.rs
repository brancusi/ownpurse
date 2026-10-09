//! Socket re-registration across an in-place upgrade (v0.10.3 §6): exec keeps the pid, so the new build's
//! entry has the same id, pid and socket path, and (through `upgrade::STARTED_ENV`) the same start time.
//! Its own test binary: it sets an environment variable, which nothing else here may race with.

use ownpurse_core::bridge::{self as wire, Dirs};
use std::path::PathBuf;

fn short_dirs(name: &str) -> Dirs {
    let root = PathBuf::from(format!("/tmp/xab-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    Dirs {
        state: root.join("s"),
        sockets: root.join("r"),
    }
}

fn start(dirs: &Dirs) -> ownpurse_tui::bridge::Server {
    ownpurse_tui::bridge::Server::start(
        dirs.clone(),
        "demo",
        Box::new(|| false),
        Box::new(|_| true),
    )
    .unwrap()
}

#[test]
fn the_new_build_registers_the_same_session_again() {
    let dirs = short_dirs("resock");
    let old = start(&dirs);
    let before = wire::sessions(&dirs);
    assert_eq!(before.len(), 1);
    let started = old.started();
    assert_eq!(before[0].started, started);
    // the handover closes the socket and unregisters (dropping the server)
    drop(old);
    assert!(wire::sessions(&dirs).is_empty(), "unregistered before exec");
    // the new process: same pid, the start time passed on
    // SAFETY: this test binary runs this one test; nothing else reads the environment concurrently.
    unsafe { std::env::set_var(ownpurse_tui::upgrade::STARTED_ENV, &started) };
    let new = start(&dirs);
    let after = wire::sessions(&dirs);
    assert_eq!(after.len(), 1);
    let (a, b) = (&before[0], &after[0]);
    assert_eq!(
        (&b.id, b.pid, &b.socket, &b.started),
        (&a.id, a.pid, &a.socket, &a.started)
    );
    assert!(std::path::Path::new(&b.socket).exists());
    drop(new);
    assert!(wire::sessions(&dirs).is_empty());
}
