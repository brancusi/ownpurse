//! v0.12.0: themes, per-scope accents, the cosmetic overlay (theme spec v2; V22–V25).

use ownpurse_tui::app::{App, Msg};
use ownpurse_tui::keymap::Key;
use ownpurse_tui::quiet::{Counting, Quiet};
use ownpurse_tui::runtime::Runtime;
use ownpurse_tui::snapshot::{self, fixture_app};
use ownpurse_tui::theme;
use ownpurse_tui::theme_tokens::Token;
use ratatui::layout::Rect;
use ratatui::{Terminal, TerminalOptions, Viewport};
use serde_json::{Value, json};

type Rt = Runtime<Quiet<Counting>>;

fn rt() -> Rt {
    let backend = Quiet::fixed(Counting::default(), true, 100, 30);
    let terminal = Terminal::with_options(
        backend,
        TerminalOptions {
            viewport: Viewport::Fixed(Rect::new(0, 0, 100, 30)),
        },
    )
    .unwrap();
    let mut rt = Runtime::new(terminal, fixture_app(theme::LEDGER_DARK, false));
    rt.frame().unwrap();
    rt
}

fn send(rt: &mut Rt, verbs: &[&[&str]]) -> Value {
    rt.agent(&json!({"v": 1, "kind": "view", "op": "send", "verbs": verbs}))
}

fn state(rt: &mut Rt) -> Value {
    rt.agent(&json!({"v": 1, "kind": "view", "op": "state"}))["state"].clone()
}

fn key(rt: &mut Rt, k: &str) {
    rt.send(Msg::Key(Key::parse(k).unwrap().event()));
    rt.frame().unwrap();
}

fn round_trip(a: &App) -> App {
    let text = a.view_state(a.rev).canonical();
    App::from_state_json(&text, a.source.clone(), false).unwrap()
}

fn frame(a: &mut App) -> String {
    let buf = snapshot::render(a, 100, 30);
    snapshot::styled(&buf)
}

// ── V25 the cosmetic overlay (spec §2.7) ────────────────────────────────────────────────────────────

#[test]
fn v25_overlay_set_clear_and_round_trip() {
    let mut rt = rt();
    let band = rt.app.ui.theme.rgb(Token::HeaderBand);
    let r = send(
        &mut rt,
        &[&[
            "overlay",
            "set",
            "header.band.bg=#3a2a20",
            "knob.selection=0.02",
        ]],
    );
    assert_eq!(r["ok"], true, "{r}");
    assert_eq!(rt.app.ui.theme.rgb(Token::HeaderBand), 0x3A2A20);
    let st = state(&mut rt);
    assert_eq!(
        st["theme"]["overlay"]["tokens"],
        json!({"header.band.bg": "#3a2a20", "knob.selection": "0.02"})
    );
    assert_eq!(st["theme"]["revision"], "1");
    assert_eq!(st["theme"]["knobs"]["selection"], "0.02");
    // round-trips through the canonical state, colours derived again
    let mut b = round_trip(&rt.app);
    b.repaint();
    assert_eq!(b.ui.theme_overlay, rt.app.ui.theme_overlay);
    assert_eq!(b.ui.theme.rgb(Token::HeaderBand), 0x3A2A20);
    // clear one, then all
    let r = send(&mut rt, &[&["overlay", "clear", "knob.selection"]]);
    assert_eq!(r["ok"], true, "{r}");
    assert_eq!(rt.app.ui.theme_overlay.len(), 1);
    let r = send(&mut rt, &[&["overlay", "clear"]]);
    assert_eq!(r["ok"], true, "{r}");
    assert!(rt.app.ui.theme_overlay.is_empty());
    assert_eq!(
        rt.app.ui.theme.rgb(Token::HeaderBand),
        band,
        "back to the theme's band"
    );
    assert_eq!(state(&mut rt)["theme"]["revision"], "3");
}

#[test]
fn v25_bad_token_exits_2_with_the_valid_names() {
    let mut rt = rt();
    for (verbs, needle) in [
        (
            vec!["overlay", "set", "header.bnd.bg=#000000"],
            "header.band.bg",
        ),
        (vec!["overlay", "set", "accent=#12"], "#rrggbb"),
        (vec!["overlay", "set", "knob.nope=0.1"], "header_band"),
        (vec!["accent", "teal"], "harbor"),
        (vec!["theme", "nope"], "ledger-dark"),
    ] {
        let ui = rt.app.ui.clone();
        let r = send(&mut rt, &[&verbs]);
        assert_eq!(
            (r["error"].as_str(), r["exit"].as_i64()),
            (Some("bad_token"), Some(2)),
            "{r}"
        );
        assert!(
            r["message"].as_str().unwrap().contains(needle),
            "{needle}: {r}"
        );
        assert_eq!(rt.app.ui, ui, "nothing applied");
    }
}

#[test]
fn v25_cosmetic_sends_are_not_refused_while_the_user_types() {
    let mut rt = rt();
    state(&mut rt);
    key(&mut rt, "/");
    key(&mut rt, "d");
    let r = send(&mut rt, &[&["tab", "ledger"]]);
    assert_eq!(r["error"], "user_busy", "a view verb still is");
    let r = send(&mut rt, &[&["accent", "iris"]]);
    assert_eq!(r["ok"], true, "{r}");
    assert_eq!(rt.app.ui.theme_overlay["accent"], "iris");
    // the half-typed filter is untouched
    assert!(matches!(&rt.app.ui.prompt, Some(ownpurse_tui::app::Prompt::Filter(t)) if t == "d"));
}

#[test]
fn v25_an_overlay_breaking_contrast_is_warned_not_refused() {
    let mut rt = rt();
    let r = send(&mut rt, &[&["overlay", "set", "text.3=#222222"]]);
    assert_eq!(r["ok"], true, "{r}");
    let w = r["contrast_warnings"].as_array().expect("warnings");
    assert!(
        w.iter()
            .any(|w| w.as_str().unwrap().starts_with("text.3 on bg")),
        "{r}"
    );
    let r = send(&mut rt, &[&["overlay", "clear"]]);
    assert!(r.get("contrast_warnings").is_none(), "{r}");
}

#[test]
fn v25_theme_switch_is_session_only() {
    let mut rt = rt();
    let r = send(&mut rt, &[&["theme", "ledger-light"]]);
    assert_eq!(r["ok"], true, "{r}");
    assert!(!rt.app.ui.theme.dark);
    assert_eq!(state(&mut rt)["theme"]["name"], "ledger-light");
}

// ── V23 the accent comes from the state only ───────────────────────────────────────────────────────

#[test]
fn v23_changing_the_accent_token_in_an_imported_state_changes_the_frame() {
    let mut a = fixture_app(theme::LEDGER_DARK, false);
    snapshot::agent_send(&mut a, &[&["scope", "design"]], 100, 30);
    let mut b = round_trip(&a);
    let before = frame(&mut b);
    // the state's accent token is the overlay's `accent` when set: edit it in the exported state
    let mut st: Value = serde_json::from_str(&a.view_state(a.rev).canonical()).unwrap();
    st["canonical"]["ui"]["theme_overlay"] = json!({"accent": "iris"});
    let mut c = App::from_state_json(&st.to_string(), a.source.clone(), false).unwrap();
    let after = frame(&mut c);
    assert_ne!(before, after, "the accent token drives the frame");
    assert_eq!(c.view_state(c.rev).accent.token, "iris");
    // and the text is the same: colour only (T1)
    assert_eq!(
        snapshot::text(&snapshot::render(&mut b, 100, 30)),
        snapshot::text(&snapshot::render(&mut c, 100, 30))
    );
}

#[test]
fn v23_accent_follows_the_scope_kind() {
    for (verbs, kind, token) in [
        (vec![vec!["scope", "all"]], "all", "harbor"),
        (vec![vec!["scope", "roast"]], "org", "sea"),
        (vec![vec!["scope", "group:ops"]], "group", "harbor"),
        (vec![vec!["compare", "roast,design"]], "pair", "harbor"),
        (
            vec![vec!["compare", "roast,design,lab"]],
            "compare",
            "harbor",
        ),
    ] {
        let mut a = fixture_app(theme::LEDGER_DARK, false);
        let v: Vec<&[&str]> = verbs.iter().map(|v| v.as_slice()).collect();
        snapshot::agent_send(&mut a, &v, 100, 30);
        let acc = a.view_state(a.rev).accent;
        assert_eq!(
            (acc.scope_kind, acc.token.as_str()),
            (kind, token),
            "{verbs:?}"
        );
        assert_eq!(round_trip(&a).view_state(a.rev).accent, acc, "{kind}");
    }
}

#[test]
fn v23_no_renderer_reads_accents_or_colours_except_through_the_theme() {
    // the lint: outside theme/ (and the one derivation in state.rs), no colour literal, no accent family
    // lookup, no ratatui Color constructor
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/src");
    let mut problems = Vec::new();
    fn walk(dir: &std::path::Path, out: &mut Vec<std::path::PathBuf>) {
        for e in std::fs::read_dir(dir).unwrap().flatten() {
            let p = e.path();
            if p.is_dir() {
                walk(&p, out);
            } else if p.extension().is_some_and(|x| x == "rs") {
                out.push(p);
            }
        }
    }
    let mut files = Vec::new();
    walk(std::path::Path::new(root), &mut files);
    for f in files {
        let rel = f
            .strip_prefix(root)
            .unwrap()
            .to_string_lossy()
            .replace('\\', "/");
        if rel.starts_with("theme/") {
            continue;
        }
        let text = std::fs::read_to_string(&f).unwrap();
        for (n, line) in text.lines().enumerate() {
            let code = line.split("//").next().unwrap_or("");
            let hex = code.match_indices("0x").any(|(i, _)| {
                let h: String = code[i + 2..]
                    .chars()
                    .take_while(char::is_ascii_hexdigit)
                    .collect();
                h.len() == 6
            });
            let css = code.contains("\"#") && code.contains("color");
            let ctor = [
                "Color::Rgb(",
                "Color::Indexed(",
                "Color::Blue",
                "Color::Cyan",
                "Color::Magenta",
            ]
            .iter()
            .any(|c| code.contains(c));
            let family = rel != "state.rs"
                && (code.contains("accent::assign") || code.contains("source.accents"));
            // render.rs converts finished cells to HTML/SVG (it reads colours, never picks them)
            let allowed = rel == "render.rs" || rel == "snapshot.rs";
            if (hex || css || ctor) && !allowed || family {
                problems.push(format!("src/{rel}:{}: {}", n + 1, line.trim()));
            }
        }
    }
    assert!(
        problems.is_empty(),
        "colour or accent outside theme/:\n{}",
        problems.join("\n")
    );
}

// ── user themes (spec §3): extends, file:line, hot reload ──────────────────────────────────────────

fn reloaded(a: &mut App, text: &str) {
    use ownpurse_tui::user_config::{Change, ConfigEvent, UserConfig, parse_theme};
    let f = parse_theme("themes/warm.toml", "warm", text).unwrap();
    let mut cfg = UserConfig::default();
    cfg.themes.insert("warm".into(), f);
    a.user_config_event(ConfigEvent::Reloaded(
        Box::new(cfg),
        vec![Change::Theme("warm".into())],
    ));
    a.repaint();
}

#[test]
fn a_user_theme_applies_reloads_and_keeps_the_last_valid_on_error() {
    let mut a = fixture_app(theme::LEDGER_DARK, false);
    a.source.theme_pref = Some("warm".into());
    let bg0 = a.ui.theme.rgb(Token::Bg);
    reloaded(&mut a, snapshot::DEMO_USER_THEME);
    assert_eq!(a.view_state(a.rev).theme.name, "warm");
    assert_eq!(
        a.ui.theme.rgb(Token::Bg),
        0x161412,
        "extends ledger-dark, bg overridden"
    );
    assert_ne!(bg0, 0x161412);
    let rev = a.ui.theme_revision;
    // a valid save: re-applied, one revision, one note
    reloaded(
        &mut a,
        &snapshot::DEMO_USER_THEME.replace("#161412", "#101010"),
    );
    assert_eq!(a.ui.theme.rgb(Token::Bg), 0x101010);
    assert_eq!(a.ui.theme_revision, rev + 1);
    assert_eq!(a.ui.note.as_deref(), Some("theme warm reloaded"));
    // an invalid save: the validator refuses it with file:line; the previous theme stays
    let bad = "[meta]\nname = \"warm\"\n[semantic]\nbg = \"@slate.95\"\n";
    let f = ownpurse_tui::user_config::parse_theme("themes/warm.toml", "warm", bad).unwrap();
    let e = ownpurse_tui::theme::user::validate(&f, &Default::default()).unwrap_err();
    assert_eq!((e.file.as_str(), e.line), ("themes/warm.toml", 4));
    snapshot::theme_invalid(&mut a);
    a.repaint();
    assert_eq!(
        a.ui.theme.rgb(Token::Bg),
        0x101010,
        "the last valid theme stays"
    );
    assert!(a.ui.result.is_some(), "the error band");
    // a render from state carries the theme text (never reads the file)
    let mut b = round_trip(&a);
    b.repaint();
    assert_eq!(b.ui.theme.rgb(Token::Bg), 0x101010);
}
