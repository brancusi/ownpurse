//! keys.toml (v0.12.0): the built-in table with the user's bindings layered on, validated with file:line,
//! swapped whole on a hot reload (footer, help, palette and `keys` follow), kept on an invalid save,
//! exported in the state (V21), ⌘ (SUPER) parsing and dispatch. Temp dirs only.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ownpurse_tui::app::{App, Msg, Tab, update};
use ownpurse_tui::keymap::{self, Ctx, Key, Keymap};
use ownpurse_tui::snapshot::{self, fixture_app, row_text};
use ownpurse_tui::theme;
use ownpurse_tui::user_config::{ConfigEvent, Loader, UserConfigWatcher, parse_keys};

fn keys(text: &str) -> ownpurse_tui::user_config::KeysFile {
    parse_keys(text).unwrap()
}

const CMD_HISTORY: &str = "v = 1\n\n[[bindings]]\nkey = \"cmd+[\"\naction = \"history.back\"\n\n[[bindings]]\nkey = \"cmd+]\"\naction = \"history.forward\"\n";

#[test]
fn cmd_keys_parse_and_match_super_events() {
    let k = Key::parse("cmd+[").unwrap();
    assert_eq!(k.mods, KeyModifiers::SUPER);
    assert_eq!(Key::parse("super-[").unwrap(), k);
    assert_eq!(k.display(), "⌘[");
    assert_eq!(
        Key::of(&KeyEvent::new(KeyCode::Char('['), KeyModifiers::SUPER)),
        k
    );
    assert_eq!(Key::parse("ctrl+shift+tab"), Key::parse("ctrl-shift-tab"));
    assert!(Key::parse("hyper+x").is_none());
    assert_eq!(Key::parse("ctrl+p"), Key::parse("ctrl-p"));
}

#[test]
fn bindings_layer_over_the_table_and_unbind_removes() {
    let km = Keymap::build(&keys(CMD_HISTORY)).unwrap();
    assert_eq!(km.overrides, 2);
    let back = km
        .bindings
        .iter()
        .find(|b| b.action == "history.back" && b.ctx == Ctx::Global)
        .unwrap();
    let specs: Vec<&str> = back.keys.iter().map(|k| k.spec.as_str()).collect();
    assert_eq!(specs, ["alt-left", "cmd+["], "added beside the default");
    assert_eq!(back.keys[1].from, Some(3));
    assert!(km.has_cmd());
    // rebind quit: unbind q, bind ctrl+q; the footer keeps its place
    let km = Keymap::build(&keys(
        "v = 1\n[[unbind]]\nkey = \"q\"\n[[bindings]]\nkey = \"ctrl+q\"\naction = \"app.quit\"\n",
    ))
    .unwrap();
    let quit = km.bindings.iter().find(|b| b.action == "app.quit").unwrap();
    assert_eq!(quit.keys.len(), 1);
    assert_eq!(quit.keys[0].spec, "ctrl+q");
    assert_eq!(quit.footer, Some(1));
    assert!(keymap::lookup_in(&km, &[Ctx::Global], Key::parse("q").unwrap(), &|_| true).is_none());
    // a palette action without a default key gets its label
    let km = Keymap::build(&keys(
        "v = 1\n[[bindings]]\nkey = \"ctrl+t\"\naction = \"theme.cycle\"\n",
    ))
    .unwrap();
    let t = km
        .bindings
        .iter()
        .find(|b| b.action == "theme.cycle")
        .unwrap();
    assert_eq!(t.label, "Change theme");
}

#[test]
fn every_invalid_case_is_refused_with_file_and_line() {
    let cases = [
        (
            "v = 1\n[[bindings]]\nkey = \"ctrl+z\"\naction = \"histroy.back\"\n",
            2,
            "unknown action `histroy.back` (did you mean `history.back`?)",
        ),
        (
            "v = 1\n[[bindings]]\nkey = \"hyper+x\"\naction = \"history.back\"\n",
            2,
            "bad key `hyper+x`",
        ),
        (
            "v = 1\n[[bindings]]\nkey = \"q\"\naction = \"history.back\"\n",
            2,
            "q bound twice in global",
        ),
        (
            "v = 1\n\n\n[[bindings]]\nkey = \"ctrl+s\"\naction = \"net.sync_all\"\n",
            4,
            "network actions have no keys",
        ),
        (
            "v = 1\n[[bindings]]\nkey = \"y\"\naction = \"confirm.activate\"\ncontext = \"confirm\"\n",
            2,
            "confirm band's keys are fixed",
        ),
        (
            "v = 1\n[[unbind]]\nkey = \"enter\"\ncontext = \"confirm\"\n",
            2,
            "confirm band's keys are fixed",
        ),
        (
            "v = 1\n[[bindings]]\nkey = \"x\"\naction = \"history.back\"\ncontext = \"view.nope\"\n",
            2,
            "unknown context `view.nope`",
        ),
        (
            "v = 1\n[[unbind]]\nkey = \"ctrl+z\"\n",
            2,
            "`ctrl+z` is not bound in global",
        ),
        // a view key reused globally for another meaning
        (
            "v = 1\n[[bindings]]\nkey = \"h\"\naction = \"history.back\"\n",
            2,
            "h means history.back globally and basis.toggle",
        ),
    ];
    for (text, line, msg) in cases {
        let k = parse_keys(text).unwrap();
        let e = Keymap::build(&k).unwrap_err();
        assert_eq!(e.file, "keys.toml");
        assert_eq!(e.line, line, "{text}: {e}");
        assert!(e.message.contains(msg), "{text}: {e}");
    }
    assert!(
        parse_keys("v = 7\n")
            .unwrap_err()
            .message
            .contains("not supported")
    );
}

#[test]
fn history_moves_with_the_cmd_keys() {
    let mut a = fixture_app(theme::LEDGER_DARK, false);
    a.set_keys(&keys(CMD_HISTORY)).unwrap();
    snapshot::run_script(&mut a, "2 3", 100, 30);
    assert_eq!(a.ui.tab, Tab::Intercompany);
    let _ = update(
        &mut a,
        Msg::Key(KeyEvent::new(KeyCode::Char('['), KeyModifiers::SUPER)),
    );
    assert_eq!(a.ui.tab, Tab::Ledger, "cmd+[ is back");
    let _ = update(
        &mut a,
        Msg::Key(KeyEvent::new(KeyCode::Char(']'), KeyModifiers::SUPER)),
    );
    assert_eq!(a.ui.tab, Tab::Intercompany, "cmd+] is forward");
}

fn foot(a: &mut App) -> String {
    let (w, h) = a.ui.size;
    row_text(&snapshot::render(a, w, h), h - 1)
}

#[test]
fn a_reload_swaps_the_keymap_and_the_footer_follows_and_a_bad_save_keeps_it() {
    let mut a = fixture_app(theme::LEDGER_DARK, false);
    snapshot::run_script(&mut a, "", 140, 40);
    assert!(foot(&mut a).contains("q Quit"));
    let d = std::env::temp_dir().join(format!("xa-keys-reload-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    let mut l = Loader::new(d.clone(), Some(|k| Keymap::build(k).map(|_| ())), None);
    std::fs::write(
        d.join("keys.toml"),
        "v = 1\n[[unbind]]\nkey = \"q\"\n[[bindings]]\nkey = \"ctrl+q\"\naction = \"app.quit\"\n",
    )
    .unwrap();
    let rev = a.rev;
    for ev in l.reload() {
        let _ = update(&mut a, Msg::UserConfig(ev));
    }
    assert!(a.rev > rev);
    assert_eq!(a.ui.note.as_deref(), Some("keys reloaded (2 overrides)"));
    let f = foot(&mut a);
    assert!(f.contains("⌃Q Quit") && !f.contains("q Quit"), "{f}");
    let st = a.view_state(a.rev);
    assert_eq!(st.keymap.overrides, 2);
    assert!(st.keymap.revision.is_some());
    // a bad save: the keymap stays, the band names file:line
    std::fs::write(
        d.join("keys.toml"),
        "v = 1\n[[bindings]]\nkey = \"ctrl+q\"\naction = \"nope\"\n",
    )
    .unwrap();
    for ev in l.reload() {
        let _ = update(&mut a, Msg::UserConfig(ev));
    }
    let band = a.ui.result.clone().expect("the error band");
    assert!(
        band.lines[0].starts_with("keys.toml:2: bindings[0].action: unknown action `nope`"),
        "{band:?}"
    );
    assert_eq!(a.keymap.overrides, 2, "the previous keymap stays live");
    // fixed: the band goes
    std::fs::write(d.join("keys.toml"), CMD_HISTORY).unwrap();
    for ev in l.reload() {
        let _ = update(&mut a, Msg::UserConfig(ev));
    }
    assert!(a.ui.result.is_none());
    // help marks keys.toml keys with *
    snapshot::run_script(&mut a, "?", 140, 40);
    assert!(snapshot::text(&snapshot::render(&mut a, 140, 40)).contains("⌘[*"));
    let _ = std::fs::remove_dir_all(&d);
}

#[test]
fn the_keymap_round_trips_in_the_state_and_wins_over_the_file() {
    let mut a = fixture_app(theme::LEDGER_DARK, false);
    a.set_keys(&keys(
        "v = 1\n[[unbind]]\nkey = \"q\"\n[[bindings]]\nkey = \"ctrl+q\"\naction = \"app.quit\"\n",
    ))
    .unwrap();
    snapshot::run_script(&mut a, "2", 100, 30);
    let live = snapshot::render(&mut a, 100, 30);
    let text = a.view_state(a.rev).canonical();
    let mut b = App::from_state_json(&text, a.source.clone(), false).expect("imports");
    assert_eq!(b.keymap.overrides, 2);
    assert_eq!(snapshot::render(&mut b, 100, 30), live);
    assert_eq!(b.view_state(b.rev).canonical(), text);
    // a state from before v0.12.0 (no keymap block) renders with the built-in table
    let mut v: serde_json::Value = serde_json::from_str(&text).unwrap();
    v["canonical"].as_object_mut().unwrap().remove("keymap");
    let c = App::from_state_json(&v.to_string(), a.source.clone(), false).expect("imports");
    assert_eq!(c.keymap.overrides, 0);
}

#[test]
fn the_markdown_marks_overrides_and_is_unchanged_without_them() {
    assert_eq!(
        keymap::to_markdown_in(keymap::default_map()),
        keymap::to_markdown()
    );
    let km = Keymap::build(&keys(CMD_HISTORY)).unwrap();
    let md = keymap::to_markdown_in(&km);
    assert!(md.contains("Effective keymap: the built-in table with 2 overrides from keys.toml"));
    assert!(md.contains("`⌘[` keys.toml:3"), "{md}");
    assert!(keymap::to_json_in(&km).contains("\"from\":[{\"key\":\"cmd+[\",\"line\":3}]"));
}

#[test]
fn without_keyboard_enhancement_a_cmd_binding_gets_a_notice() {
    let mut a = fixture_app(theme::LEDGER_DARK, false);
    a.set_keys(&keys(CMD_HISTORY)).unwrap();
    a.cmd_notice();
    assert_eq!(
        a.ui.note.as_deref(),
        Some("Cmd bindings won't work in this terminal (no keyboard enhancement)")
    );
    let mut b = fixture_app(theme::LEDGER_DARK, false);
    b.cmd_keys = true;
    b.set_keys(&keys(CMD_HISTORY)).unwrap();
    b.cmd_notice();
    assert!(b.ui.note.is_none());
}

#[test]
fn the_watcher_drives_a_live_swap_end_to_end() {
    let d = std::env::temp_dir().join(format!("xa-keys-live-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    let (tx, rx) = std::sync::mpsc::channel();
    let tx = std::sync::Mutex::new(tx);
    let _w = UserConfigWatcher::spawn(
        Loader::new(d.clone(), Some(|k| Keymap::build(k).map(|_| ())), None),
        move |e| {
            let _ = tx.lock().unwrap().send(e);
        },
    )
    .unwrap();
    std::thread::sleep(std::time::Duration::from_millis(300));
    std::fs::write(d.join(".tmp-keys"), CMD_HISTORY).unwrap();
    std::fs::rename(d.join(".tmp-keys"), d.join("keys.toml")).unwrap();
    let ev = rx
        .recv_timeout(std::time::Duration::from_secs(60))
        .expect("event");
    let mut a = fixture_app(theme::LEDGER_DARK, false);
    let _ = update(&mut a, Msg::UserConfig(ev.clone()));
    assert!(matches!(ev, ConfigEvent::Reloaded(..)));
    assert!(a.keymap.has_cmd());
    let _ = std::fs::remove_dir_all(&d);
}

/// A byte counter for the terminal (V13).
#[derive(Clone, Default)]
struct Counting(std::rc::Rc<std::cell::RefCell<usize>>);

impl std::io::Write for Counting {
    fn write(&mut self, b: &[u8]) -> std::io::Result<usize> {
        *self.0.borrow_mut() += b.len();
        Ok(b.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

#[test]
fn v13_idle_writes_nothing_with_the_watcher_active() {
    use ratatui::layout::Rect;
    use ratatui::{Terminal, TerminalOptions, Viewport};
    let d = std::env::temp_dir().join(format!("xa-keys-idle-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    std::fs::write(d.join("keys.toml"), CMD_HISTORY).unwrap();
    let (tx, rx) = std::sync::mpsc::channel();
    let tx = std::sync::Mutex::new(tx);
    let _w = UserConfigWatcher::spawn(Loader::new(d.clone(), None, None), move |e| {
        let _ = tx.lock().unwrap().send(e);
    })
    .unwrap();
    let out = Counting::default();
    let backend = ownpurse_tui::quiet::Quiet::fixed(out.clone(), true, 100, 30);
    let terminal = Terminal::with_options(
        backend,
        TerminalOptions {
            viewport: Viewport::Fixed(Rect::new(0, 0, 100, 30)),
        },
    )
    .unwrap();
    let mut rt =
        ownpurse_tui::runtime::Runtime::new(terminal, fixture_app(theme::LEDGER_DARK, false));
    rt.frame().unwrap();
    let n = *out.0.borrow();
    let t0 = rt.app.now;
    for i in 1..=20 {
        rt.send(Msg::Tick(t0 + i / 4));
        assert!(!rt.frame().unwrap(), "redraw while idle");
    }
    // nothing changed on disk: the watcher woke nothing
    assert!(
        rx.recv_timeout(std::time::Duration::from_millis(500))
            .is_err()
    );
    assert_eq!(*out.0.borrow(), n, "idle wrote bytes");
    let _ = std::fs::remove_dir_all(&d);
}

#[test]
fn tui_keys_file_points_elsewhere_and_errors_name_it() {
    let base = std::env::temp_dir().join(format!("xa-keys-file-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&base);
    let (dir, other) = (base.join("cfg"), base.join("dotfiles"));
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::create_dir_all(&other).unwrap();
    // a keys.toml in the folder is ignored when the setting names another file
    std::fs::write(
        dir.join("keys.toml"),
        "v = 1\n[[bindings]]\nkey = \"ctrl+z\"\naction = \"app.quit\"\n",
    )
    .unwrap();
    std::fs::write(other.join("my-keys.toml"), CMD_HISTORY).unwrap();
    let mut l = Loader::new(dir.clone(), Some(|k| Keymap::build(k).map(|_| ())), None)
        .with_keys_file(other.join("my-keys.toml"), "dotfiles/my-keys.toml".into());
    let ev = l.reload();
    assert!(
        matches!(&ev[..], [ConfigEvent::Reloaded(c, _)] if c.keys.bindings.len() == 2),
        "{ev:?}"
    );
    std::fs::write(
        other.join("my-keys.toml"),
        "v = 1\n[[bindings]]\nkey = \"q\"\naction = \"history.back\"\n",
    )
    .unwrap();
    let ev = l.reload();
    assert!(
        matches!(&ev[..], [ConfigEvent::Invalid(i)] if i.to_string().starts_with("dotfiles/my-keys.toml:2: ")),
        "{ev:?}"
    );
    // the watcher sees saves there too
    let (tx, rx) = std::sync::mpsc::channel();
    let tx = std::sync::Mutex::new(tx);
    let loader = Loader::new(dir.clone(), None, None)
        .with_keys_file(other.join("my-keys.toml"), "dotfiles/my-keys.toml".into());
    let _w = UserConfigWatcher::spawn(loader, move |e| {
        let _ = tx.lock().unwrap().send(e);
    })
    .unwrap();
    std::thread::sleep(std::time::Duration::from_millis(300));
    while rx.try_recv().is_ok() {}
    std::fs::write(other.join("my-keys.toml"), CMD_HISTORY).unwrap();
    assert!(rx.recv_timeout(std::time::Duration::from_secs(60)).is_ok());
    let _ = std::fs::remove_dir_all(&base);
}

#[test]
fn spec_round_trips_through_parse() {
    for s in [
        "cmd+[",
        "cmd+]",
        "ctrl+p",
        "shift+tab",
        "alt+left",
        "enter",
        "G",
        "space",
        "ctrl+alt+cmd+x",
        "pagedown",
    ] {
        let k = Key::parse(s).unwrap();
        assert_eq!(Key::parse(&k.spec()), Some(k), "{s} → {}", k.spec());
    }
    assert_eq!(
        Key::of(&KeyEvent::new(KeyCode::Char(']'), KeyModifiers::SUPER)).spec(),
        "cmd+]"
    );
}
