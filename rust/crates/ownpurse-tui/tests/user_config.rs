//! The user config layer (v0.12.0): ~/.config/ownpurse discovery, keys.toml and themes/*.toml parsed with
//! file:line errors, keep-last-valid per file, the revision, and the one OS-event watcher. Temp dirs only:
//! nothing here reads or writes the real config folder.

use ownpurse_tui::user_config::{
    Change, ConfigEvent, Loader, UserConfigWatcher, config_dir, parse_keys,
};
use std::path::{Path, PathBuf};

fn temp(tag: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("xa-userconfig-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(d.join("themes")).unwrap();
    d
}

#[test]
fn the_folder_is_xdg_then_home() {
    assert_eq!(
        config_dir(Some(Path::new("/x/cfg")), Some(Path::new("/h"))),
        Some(PathBuf::from("/x/cfg/ownpurse"))
    );
    assert_eq!(
        config_dir(Some(Path::new("relative")), Some(Path::new("/h"))),
        Some(PathBuf::from("/h/.config/ownpurse")),
        "a relative XDG_CONFIG_HOME is ignored (the spec)"
    );
    assert_eq!(config_dir(None, None), None);
}

#[test]
fn keys_parse_with_lines_and_reject_with_file_line() {
    let k = parse_keys(
        "v = 1\n\n[[bindings]]\nkey = \"cmd+[\"\naction = \"history.back\"\n\n[[unbind]]\nkey = \"q\"\n",
    )
    .unwrap();
    assert_eq!(k.bindings[0].key, "cmd+[");
    assert_eq!(k.bindings[0].context, "global");
    assert_eq!(k.bindings[0].line, 3);
    assert_eq!(k.unbind[0].line, 7);
    let e = parse_keys("v = 1\n[[bindings]]\nkey = \"x\"\n").unwrap_err();
    assert_eq!(
        e.to_string(),
        "keys.toml:2: bindings[0].action: missing `action`"
    );
    let e = parse_keys("v = 2\n").unwrap_err();
    assert!(
        e.to_string()
            .starts_with("keys.toml:1: v: version 2 is not supported"),
        "{e}"
    );
    let e = parse_keys("v = 1\n[[bindings]\n").unwrap_err();
    assert_eq!((e.line, e.path.as_str()), (2, "(syntax)"));
}

#[test]
fn reload_keeps_the_last_valid_file_and_reports_only_changes() {
    let d = temp("reload");
    let mut l = Loader::new(d.clone(), None, None);
    let empty_rev = l.current.revision.clone();
    assert!(l.reload().is_empty(), "nothing there: nothing changed");
    std::fs::write(
        d.join("keys.toml"),
        "v = 1\n[[bindings]]\nkey = \"cmd+[\"\naction = \"history.back\"\n",
    )
    .unwrap();
    std::fs::write(d.join("themes/warm.toml"), "[meta]\nname = \"warm\"\n").unwrap();
    let ev = l.reload();
    match &ev[..] {
        [ConfigEvent::Reloaded(c, ch)] => {
            assert_eq!(c.keys.bindings.len(), 1);
            assert!(c.themes.contains_key("warm"));
            assert!(ch.contains(&Change::Keys) && ch.contains(&Change::Theme("warm".into())));
            assert_ne!(c.revision, empty_rev);
        }
        other => panic!("{other:?}"),
    }
    assert!(l.reload().is_empty(), "an echo changes nothing");
    let rev = l.current.revision.clone();
    // a bad save: the old keys stay, file:line reported once
    std::fs::write(
        d.join("keys.toml"),
        "v = 1\n[[bindings]]\nkey = \"cmd+]\"\n",
    )
    .unwrap();
    let ev = l.reload();
    assert!(
        matches!(&ev[..], [ConfigEvent::Invalid(i)] if i.file == "keys.toml" && i.line == 2),
        "{ev:?}"
    );
    assert_eq!(l.current.keys.bindings[0].key, "cmd+[");
    assert_eq!(l.current.revision, rev);
    assert!(
        l.reload().is_empty(),
        "the same problem is not reported twice"
    );
    // fixed: reloaded
    std::fs::write(
        d.join("keys.toml"),
        "v = 1\n[[bindings]]\nkey = \"cmd+]\"\naction = \"history.forward\"\n",
    )
    .unwrap();
    assert!(matches!(&l.reload()[..], [ConfigEvent::Reloaded(..)]));
    // a theme removed
    std::fs::remove_file(d.join("themes/warm.toml")).unwrap();
    assert!(
        matches!(&l.reload()[..], [ConfigEvent::Reloaded(c, ch)] if !c.themes.contains_key("warm") && ch == &vec![Change::Theme("warm".into())])
    );
}

#[test]
fn the_watcher_sees_a_save_and_an_atomic_rename_save() {
    let d = temp("watch");
    let (tx, rx) = std::sync::mpsc::channel();
    let tx = std::sync::Mutex::new(tx);
    let _w = UserConfigWatcher::spawn(Loader::new(d.clone(), None, None), move |e| {
        let _ = tx.lock().unwrap().send(e);
    })
    .expect("watcher");
    std::thread::sleep(std::time::Duration::from_millis(300));
    std::fs::write(d.join("keys.toml"), "v = 1\n").unwrap();
    let got = rx
        .recv_timeout(std::time::Duration::from_secs(60))
        .expect("an event");
    assert!(matches!(got, ConfigEvent::Reloaded(..)), "{got:?}");
    // an editor's atomic save: write a temp file, rename over keys.toml
    std::fs::write(
        d.join(".keys.toml.swp"),
        "v = 1\n[[bindings]]\nkey = \"cmd+[\"\naction = \"history.back\"\n",
    )
    .unwrap();
    std::fs::rename(d.join(".keys.toml.swp"), d.join("keys.toml")).unwrap();
    let got = loop {
        let e = rx
            .recv_timeout(std::time::Duration::from_secs(60))
            .expect("an event after rename");
        if let ConfigEvent::Reloaded(c, _) = &e
            && !c.keys.bindings.is_empty()
        {
            break e;
        }
    };
    assert!(matches!(got, ConfigEvent::Reloaded(..)));
    let _ = std::fs::remove_dir_all(&d);
}
