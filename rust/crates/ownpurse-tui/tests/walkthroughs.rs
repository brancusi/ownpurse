//! Overlays and walkthroughs (walkthroughs design §1–§3): hints, rings and spotlights over our own drawing
//! (placed by caretline-layers), the agent verbs and their landing replies, the narration panel, the
//! player (deterministic: play, jump and reload give the same frame), record mode, the glossary and lay
//! mode, `validate`, and the demo walkthrough "How a correction gets made" on a demo manifest. Everything
//! runs on a temp copy of the demo record and a temp manifest store (the fixtures are never written).
//!
//! Review snapshots with `cargo insta review` (or `INSTA_UPDATE=always cargo test -p ownpurse-tui --test
//! walkthroughs`).

use ownpurse_core::manifest::ops::{self, Guard, Target};
use ownpurse_core::manifest::{Refs, Store, compile};
use ownpurse_core::record::Record;
use ownpurse_tui::app::App;
use ownpurse_tui::snapshot::{self, agent_send_result, demo_app_at, styled};
use ownpurse_tui::theme::{self, Theme};
use serde_json::{Value, json};
use std::path::PathBuf;

fn demo() -> PathBuf {
    snapshot::demo_dir()
}

fn tenants() -> Vec<(String, String)> {
    let t: Value =
        serde_json::from_str(&std::fs::read_to_string(demo().join("tenants.json")).unwrap())
            .unwrap();
    let mut v: Vec<(String, String)> = t
        .as_object()
        .unwrap()
        .iter()
        .map(|(a, v)| (a.clone(), v.as_str().unwrap().to_string()))
        .collect();
    v.sort();
    v
}

fn ctx() -> ownpurse_core::view::Ctx {
    ownpurse_core::view::Ctx {
        orgs: tenants(),
        gate: vec![("hold".into(), "2025-12-31".into())],
    }
}

/// A temp world: a copy of the demo record, a manifest store, a walkthroughs folder.
struct World {
    dir: PathBuf,
    store: Store,
}

impl World {
    fn new(name: &str) -> World {
        static N: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let n = N.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let dir = std::env::temp_dir().join(format!("xa-tui-wt-{name}-{}-{n}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("db")).unwrap();
        std::fs::create_dir_all(dir.join("walkthroughs")).unwrap();
        for ext in ["", "-wal", "-shm"] {
            let f = demo().join(format!("record.sqlite{ext}"));
            if f.exists() {
                std::fs::copy(&f, dir.join(format!("record.sqlite{ext}"))).unwrap();
            }
        }
        let store = Store::open(&dir.join("db/manifests.sqlite")).unwrap();
        World { dir, store }
    }

    fn record(&self) -> Record {
        Record::open(self.dir.join("record.sqlite")).unwrap()
    }

    /// The demo manifest: the burner igniter (bt-110, a reconciled card payment) moves from 6140 to 6310,
    /// resolved as a reclass journal, then planned.
    fn correction(mut self) -> World {
        let rec = self.record();
        ops::create(
            &mut self.store,
            &rec,
            &ctx(),
            "human",
            "fix-igniter",
            "the igniter belongs in cupping-room supplies",
        )
        .unwrap();
        let t = Target {
            entity: "BankTransactions".into(),
            org: "roast".into(),
            id: Some("bt-110".into()),
        };
        let m = self.store.manifest("fix-igniter").unwrap();
        let v = ownpurse_core::view::View::open(&rec, &ctx(), Some(&m)).unwrap();
        let d = ops::current(&v, &ctx(), &t).unwrap().unwrap();
        let after = ops::recode_doc(&d, "n1", Some("6310"), None).unwrap();
        ops::add(
            &mut self.store,
            &rec,
            &ctx(),
            "fix-igniter",
            "human",
            &t,
            &after,
            "the igniter is for the sample roaster in the cupping room",
            &Refs::default(),
            None,
            &Guard::default(),
        )
        .unwrap();
        ops::resolve(
            &mut self.store,
            "fix-igniter",
            "human",
            None,
            "reclass",
            &Guard::default(),
        )
        .unwrap();
        // plan it (the sealed package beside the store)
        let m = self.store.manifest("fix-igniter").unwrap();
        let rep =
            ownpurse_core::manifest::check::check(&rec, &ctx(), &m, &Default::default(), None)
                .unwrap();
        let cfg = ownpurse_core::write::plan::PlanConfig {
            profile: "demo".into(),
            orgs: tenants(),
            gates: ctx().gate,
            history_notes: false,
        };
        let pkg = compile::compile(&rec, &ctx(), &cfg, &m, &rep, "2026-10-07T12:00:00Z").unwrap();
        let sealed = ownpurse_core::write::seal::seal(pkg).unwrap();
        let hash = sealed.seal.clone().unwrap();
        let path = self
            .dir
            .join(format!("db/packages/{}.package.json", sealed.id));
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        ownpurse_core::write::seal::save(&path, &sealed).unwrap();
        ops::planned(
            &mut self.store,
            "fix-igniter",
            "human",
            &path.display().to_string(),
            &hash,
            m.content_rev,
        )
        .unwrap();
        self
    }

    fn app(&self, th: Theme) -> App {
        let mut a = demo_app_at(
            th,
            &self.dir.join("record.sqlite"),
            &self.dir.join("db/manifests.sqlite"),
        );
        a.source.walkthroughs = Some(self.dir.join("walkthroughs"));
        a
    }
}

fn dark() -> Theme {
    Theme::named("ledger-dark").unwrap()
}

fn send(a: &mut App, verbs: &[&[&str]]) -> Result<Vec<String>, (String, String)> {
    agent_send_result(a, verbs, a.ui.size.0, a.ui.size.1)
}

fn text(a: &mut App, w: u16, h: u16) -> String {
    snapshot::text(&snapshot::render(a, w, h))
}

fn demo_file() -> PathBuf {
    demo()
        .parent()
        .unwrap()
        .join("walkthroughs/correction.json")
}

fn demo_walkthrough() -> ownpurse_core::walkthrough::Walkthrough {
    ownpurse_core::walkthrough::parse(&std::fs::read_to_string(demo_file()).unwrap()).unwrap()
}

/// The correction world with the demo walkthrough in its folder.
fn correction(name: &str) -> World {
    let w = World::new(name).correction();
    std::fs::copy(demo_file(), w.dir.join("walkthroughs/correction.json")).unwrap();
    w
}

fn ascii_only(name: &str, t: &str) {
    let bad: Vec<char> = t.chars().filter(|c| !c.is_ascii()).collect();
    assert!(bad.is_empty(), "{name}: non-ASCII {bad:?}:\n{t}");
}

const GRID: [(&str, Theme, u16, u16); 4] = [
    ("ledger-dark", theme::LEDGER_DARK, 100, 30),
    ("ledger-light", theme::LEDGER_LIGHT, 100, 30),
    ("nocolor-ascii", theme::NO_COLOR_ASCII, 100, 30),
    ("ledger-dark", theme::LEDGER_DARK, 80, 24),
];

type Setup = fn(&mut App);

/// The overlay states (walkthroughs design §1, §3): each set up by agent sends on the demo world.
fn overlay_states() -> Vec<(&'static str, Setup)> {
    vec![
        ("hint-row", |a| {
            send(a, &[&["scope", "roast"], &["tab", "2"]]).unwrap();
            send(
                a,
                &[&[
                    "hint",
                    "row:code=6140",
                    "The igniter is in this balance.",
                    "--title",
                    "Here",
                ]],
            )
            .unwrap();
        }),
        ("highlight-chip", |a| {
            send(a, &[&["scope", "roast"], &["tab", "2"]]).unwrap();
            send(a, &[&["highlight", "chip:asof"]]).unwrap();
        }),
        ("spotlight-rows", |a| {
            send(a, &[&["scope", "roast"], &["tab", "2"]]).unwrap();
            send(a, &[&["spotlight", "row:code=6140", "row:code=6310"]]).unwrap();
        }),
        ("hint-off-screen", |a| {
            send(a, &[&["scope", "roast"], &["tab", "2"]]).unwrap();
            send(
                a,
                &[&[
                    "narrate",
                    "Interest",
                    "The term loan's interest is further down: the hint waits at the edge, pointing the way.\nScroll to it and it follows.",
                ]],
            )
            .unwrap();
            send(
                a,
                &[&["hint", "row:code=6830", "Interest on the term loan."]],
            )
            .unwrap();
        }),
        ("narrate", |a| {
            send(a, &[&["scope", "roast"], &["tab", "2"]]).unwrap();
            send(
                a,
                &[&[
                    "narrate",
                    "The trial balance",
                    "Every account with its balance. Each [debit] has a matching [credit], so the two columns agree.",
                ]],
            )
            .unwrap();
        }),
        ("glossary", |a| {
            send(a, &[&["scope", "roast"], &["tab", "2"]]).unwrap();
            send(a, &[&["narrate", "Debits", "A [debit] is the left side."]]).unwrap();
            a.ui.glossary_focus = Some("debit".into());
        }),
        ("lay-mode", |a| {
            send(a, &[&["scope", "roast"], &["tab", "2"]]).unwrap();
            let _ = ownpurse_tui::app::update(a, ownpurse_tui::app::Msg::Action("view.lay"));
        }),
    ]
}

#[test]
fn overlay_snapshots() {
    let w = World::new("snap");
    for (name, setup) in overlay_states() {
        for (tn, th, cw, ch) in GRID {
            let mut a = w.app(th);
            snapshot::render(&mut a, cw, ch);
            setup(&mut a);
            let buf = snapshot::render(&mut a, cw, ch);
            if tn == "nocolor-ascii" {
                ascii_only(name, &snapshot::text(&buf));
            }
            insta::assert_snapshot!(format!("overlay__{name}-{cw}x{ch}-{tn}"), styled(&buf));
        }
    }
}

/// The demo walkthrough, every step, at 100x30 (dark, plain) and 80x24 (dark).
#[test]
fn demo_walkthrough_snapshots() {
    let w = correction("demo-snap");
    let grid: [(&str, Theme, u16, u16); 3] = [
        ("ledger-dark", theme::LEDGER_DARK, 100, 30),
        ("nocolor-ascii", theme::NO_COLOR_ASCII, 100, 30),
        ("ledger-dark", theme::LEDGER_DARK, 80, 24),
    ];
    for (tn, th, cw, ch) in grid {
        let mut a = w.app(th);
        snapshot::render(&mut a, cw, ch);
        send(&mut a, &[&["walkthrough", "play", "correction"]]).unwrap();
        let n = a.ui.walk.as_ref().unwrap().len();
        for i in 0..n {
            if i > 0 {
                send(&mut a, &[&["walkthrough", "next"]]).unwrap();
            }
            let buf = snapshot::render(&mut a, cw, ch);
            let id = a.ui.walk.as_ref().unwrap().tour.step.clone().unwrap();
            if tn == "nocolor-ascii" {
                ascii_only(&id, &snapshot::text(&buf));
            }
            insta::assert_snapshot!(
                format!("walk__correction-{:02}-{id}-{cw}x{ch}-{tn}", i + 1),
                styled(&buf)
            );
        }
    }
}

/// Acceptance: an agent's hint reply says where the layer landed; one on a row not on this screen says
/// why; attribution is drawn; `clear-hints` removes only agents' layers.
#[test]
fn hints_report_where_they_landed() {
    let w = World::new("landed");
    let mut a = w.app(dark());
    snapshot::render(&mut a, 100, 30);
    send(&mut a, &[&["scope", "roast"], &["tab", "2"]]).unwrap();
    send(&mut a, &[&["hint", "row:code=6140", "here"]]).unwrap();
    let st = a.state_json(a.rev);
    let l = &st["layers"][0];
    assert_eq!(l["landed"], "on_screen", "{st:#}");
    assert_eq!(l["anchor"], "row:code=6140");
    assert!(
        st["anchors"]
            .as_array()
            .unwrap()
            .iter()
            .any(|x| x == "row:code=6140")
    );
    assert!(text(&mut a, 100, 30).contains("◆ agent"));
    // a row this screen doesn't have
    send(&mut a, &[&["hint", "row:code=9999", "nowhere"]]).unwrap();
    let st = a.state_json(a.rev);
    let l = st["layers"]
        .as_array()
        .unwrap()
        .iter()
        .find(|l| l["anchor"] == "row:code=9999")
        .unwrap()
        .clone();
    assert_eq!(l["landed"], "not_found");
    assert!(l["reason"].as_str().unwrap().contains("not on this screen"));
    // bad anchors and too-long texts are refused before anything changes
    let e = send(&mut a, &[&["hint", "nothing", "x"]]).unwrap_err();
    assert_eq!(e.0, "usage");
    let long = "x".repeat(300);
    let e = send(&mut a, &[&["hint", "row:code=6140", &long]]).unwrap_err();
    assert_eq!(e.0, "too_long");
    send(&mut a, &[&["clear-hints"]]).unwrap();
    assert!(a.ui.layers.layers.is_empty());
}

/// Acceptance: an agent's box never covers a protected region (the writes badge, the stage strip, the
/// apply panel).
#[test]
fn agent_boxes_never_cover_protected_regions() {
    let w = World::new("protect").correction();
    for (cw, ch) in [(140, 40), (100, 30), (80, 24)] {
        let mut a = w.app(dark());
        snapshot::render(&mut a, cw, ch);
        send(&mut a, &[&["manifest", "fix-igniter"], &["tab", "7"]]).unwrap();
        for anchor in ["chip:writes", "chip:stage", "panel:apply", "row:entry=2"] {
            send(
                &mut a,
                &[&["hint", anchor, "A hint with a few words in it."]],
            )
            .unwrap();
        }
        snapshot::render(&mut a, cw, ch);
        let plan = a.hits.plan.clone().unwrap();
        for n in ownpurse_tui::overlay::PROTECTED {
            for r in a
                .hits
                .anchors
                .get(n)
                .map(|r| r.rects.clone())
                .unwrap_or_default()
            {
                for l in &plan.layers {
                    if let Some(b) = l.rect {
                        assert!(!b.intersects(&r), "{cw}x{ch}: {} covers {n}", l.id);
                    }
                }
            }
        }
    }
}

/// Acceptance: the demo walkthrough validates (caretline-tour's lint and plan_steps at 140x40, 100x30 and
/// 80x24, the checks, the glossary links, the protected regions) with no errors.
#[test]
fn the_demo_walkthrough_validates() {
    let tour = demo_walkthrough();
    assert!(
        ownpurse_core::walkthrough::check(&tour).is_empty(),
        "{:?}",
        ownpurse_core::walkthrough::check(&tour)
    );
    let w = correction("validate");
    let found = ownpurse_tui::walk::validate(&|| w.app(dark()), &tour);
    let errors: Vec<_> = found.iter().filter(|f| f.level == "error").collect();
    assert!(errors.is_empty(), "{errors:#?}");
}

/// Acceptance: play to a step, jump straight to it, and reload from the exported state: the same frame.
#[test]
fn play_jump_and_reload_give_identical_frames() {
    let w = correction("determinism");
    for (cw, ch) in [(140, 40), (100, 30), (80, 24)] {
        for target in 1..9 {
            let mut played = w.app(dark());
            snapshot::render(&mut played, cw, ch);
            send(&mut played, &[&["walkthrough", "play", "correction"]]).unwrap();
            for _ in 0..target {
                send(&mut played, &[&["walkthrough", "next"]]).unwrap();
            }
            let a = text(&mut played, cw, ch);
            let mut jumped = w.app(dark());
            snapshot::render(&mut jumped, cw, ch);
            send(&mut jumped, &[&["walkthrough", "play", "correction"]]).unwrap();
            let n = (target + 1).to_string();
            send(&mut jumped, &[&["walkthrough", "step", &n]]).unwrap();
            let b = text(&mut jumped, cw, ch);
            assert_eq!(a, b, "{cw}x{ch} step {n}: played vs jumped");
            let state = played.state_json(played.rev).to_string();
            let mut reloaded = App::from_state_json(&state, played.source.clone(), false).unwrap();
            let c = text(&mut reloaded, cw, ch);
            assert_eq!(a, c, "{cw}x{ch} step {n}: played vs reloaded");
        }
    }
}

/// Acceptance: the player's keys: → / ← / g<n> / t + Enter / Home / End / [ ] / r / Esc restores the view.
#[test]
fn player_keys_move_and_esc_restores() {
    use ownpurse_tui::app::{Msg, update};
    let w = correction("keys");
    let mut a = w.app(dark());
    snapshot::render(&mut a, 100, 30);
    send(&mut a, &[&["scope", "design"], &["tab", "3"]]).unwrap();
    let before = text(&mut a, 100, 30);
    let key = |a: &mut App, k: &str| {
        for m in snapshot::token_msgs(k).unwrap() {
            update(a, m);
        }
        snapshot::render(a, 100, 30);
    };
    let at = |a: &App| a.ui.walk.as_ref().map(|w| w.at() + 1);
    send(&mut a, &[&["walkthrough", "play", "correction"]]).unwrap();
    assert_eq!(at(&a), Some(1));
    key(&mut a, "right");
    key(&mut a, "n");
    assert_eq!(at(&a), Some(3));
    key(&mut a, "left");
    assert_eq!(at(&a), Some(2));
    key(&mut a, "end");
    assert_eq!(at(&a), Some(10));
    key(&mut a, "home");
    assert_eq!(at(&a), Some(1));
    key(&mut a, "]");
    key(&mut a, "]");
    key(&mut a, "[");
    assert_eq!(at(&a), Some(2));
    key(&mut a, "g");
    key(&mut a, "5");
    key(&mut a, "enter");
    assert_eq!(at(&a), Some(5));
    key(&mut a, "t");
    key(&mut a, "down");
    key(&mut a, "enter");
    assert_eq!(at(&a), Some(6));
    // the strip is clickable
    let strip = a
        .hits
        .targets
        .iter()
        .find(|(_, t)| *t == ownpurse_tui::hit::Target::WalkStep(2))
        .map(|(r, _)| *r)
        .unwrap();
    for m in snapshot::token_msgs(&format!("<click:{},{}>", strip.x, strip.y)).unwrap() {
        update(&mut a, m);
    }
    assert_eq!(at(&a), Some(3));
    // `?` opens the first term's definition; any key closes it
    key(&mut a, "home");
    key(&mut a, "?");
    assert!(a.ui.glossary_focus.is_some());
    key(&mut a, "x");
    assert!(a.ui.glossary_focus.is_none());
    key(&mut a, "esc");
    assert!(a.ui.walk.is_none());
    assert!(a.ui.layers.layers.is_empty());
    let _ = Msg::Tick(0);
    // the agent's header mark and notice aside (the person pressed keys since), the same frame
    let body = |t: &str| t.lines().skip(1).take(28).collect::<Vec<_>>().join("\n");
    assert_eq!(
        body(&text(&mut a, 100, 30)),
        body(&before),
        "Esc restores the view from before"
    );
}

/// Acceptance: a failing check shows the "books changed" band, never silently wrong numbers.
#[test]
fn a_failing_check_shows_the_band() {
    let w = correction("drift");
    let mut tour = demo_walkthrough();
    let h = tour.steps[0].host.as_mut().unwrap();
    h["checks"] = json!(["amount(roast, 6140, 2025-12-31) = 1.00"]);
    let mut a = w.app(theme::NO_COLOR_ASCII);
    snapshot::render(&mut a, 100, 30);
    let mut fx = vec![];
    a.walk_start(tour, &mut fx).unwrap();
    let t = text(&mut a, 100, 30);
    assert!(
        t.contains("The books changed since this walkthrough was written"),
        "{t}"
    );
    let st = a.state_json(a.rev);
    assert!(
        st["walkthrough"]["failed_checks"][0]
            .as_str()
            .unwrap()
            .contains("now 898.06"),
        "{st:#}"
    );
}

/// Acceptance: record mode writes a walkthrough file that plays back to the same frames.
#[test]
fn record_then_play_back() {
    let w = correction("record");
    let mut a = w.app(dark());
    snapshot::render(&mut a, 100, 30);
    send(
        &mut a,
        &[&["walkthrough", "record", "start", "mine", "--title", "Mine"]],
    )
    .unwrap();
    send(&mut a, &[&["scope", "roast"], &["tab", "2"]]).unwrap();
    send(&mut a, &[&["narrate", "Roastery", "The [trial balance]."]]).unwrap();
    send(&mut a, &[&["hint", "row:code=6140", "This one."]]).unwrap();
    send(&mut a, &[&["walkthrough", "record", "step"]]).unwrap();
    let first = text(&mut a, 100, 30);
    send(&mut a, &[&["clear-hints"]]).unwrap();
    send(&mut a, &[&["highlight", "chip:asof"]]).unwrap();
    send(
        &mut a,
        &[&["narrate", "The date", "Balances at the year end."]],
    )
    .unwrap();
    send(&mut a, &[&["walkthrough", "record", "step"]]).unwrap();
    // the recording is written by the runtime's effect; here, by hand from the state
    let file = a.ui.recording.as_ref().unwrap().file.clone();
    let mut fx = vec![];
    let _ = a.apply_guide(ownpurse_tui::overlay::GuideVerb::RecordStop, &mut fx);
    assert!(matches!(
        fx.last(),
        Some(ownpurse_tui::app::Effect::SaveWalkthrough(_))
    ));
    ownpurse_core::walkthrough::write(&w.dir.join("walkthroughs"), &file).unwrap();
    let back = ownpurse_core::walkthrough::read(&w.dir.join("walkthroughs"), "mine").unwrap();
    assert_eq!(back.steps.len(), 2);
    let mut b = w.app(dark());
    snapshot::render(&mut b, 100, 30);
    send(&mut b, &[&["walkthrough", "play", "mine"]]).unwrap();
    let played = text(&mut b, 100, 30);
    // the recorded step shows the same view and the hint (as the guide's now: no agent attribution)
    for line in ["roast", "6140", "This one.", "Roastery"] {
        assert!(played.contains(line), "{line} missing:\n{played}");
    }
    assert!(first.contains("◆ agent"));
}

/// Recording preserves Caretline's placement rule, including the default for older walkthroughs.
#[test]
fn recording_preserves_arrow_head_rule() {
    use caretline_layers::{Anchor, HeadRule, Layer};
    use ownpurse_core::walkthrough::Place;

    let old: Place = serde_json::from_value(json!({"arrow": true})).unwrap();
    assert_eq!(old.head, HeadRule::Any);
    for head in [HeadRule::Any, HeadRule::OnAnchorRows] {
        let layer = Layer::new(Anchor::Host {
            kind: "row".into(),
            key: "code=6140".into(),
        })
        .with_arrow()
        .with_head(head);
        let step = ownpurse_tui::walk::step_layer(&layer);
        assert_eq!(step.place.head, head);
        let wire = serde_json::to_value(&step.place).unwrap();
        if head == HeadRule::Any {
            assert!(wire.get("head").is_none(), "default stays off the wire");
        } else {
            assert_eq!(wire["head"], "on_anchor_rows");
        }
        let back: Place = serde_json::from_value(wire).unwrap();
        assert_eq!(back.head, head);
    }
}

/// Walkthrough verbs never write: a step's view uses view verbs only.
#[test]
fn steps_use_view_verbs_only() {
    let w = correction("viewonly");
    let mut tour = demo_walkthrough();
    tour.steps[0].host = Some(json!({"view": {"verbs": [["walkthrough", "stop"]]}}));
    let mut a = w.app(dark());
    let mut fx = vec![];
    let e = a.walk_start(tour, &mut fx).unwrap_err();
    assert!(e.contains("not a view verb"), "{e}");
}

/// Review fix 1: narration, why-lines and glossary text are never cut: at 80x24 every step's panel text,
/// read across its pages (PgDn), is the whole source text.
#[test]
fn no_narration_text_is_lost_at_80x24() {
    use ownpurse_tui::app::update;
    let w = correction("no-cut");
    let tour = demo_walkthrough();
    let norm = |s: &str| {
        s.replace("**", "")
            .replace(['[', ']'], "")
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
    };
    let mut a = w.app(dark());
    snapshot::render(&mut a, 80, 24);
    send(&mut a, &[&["walkthrough", "play", "correction"]]).unwrap();
    assert!(a.ui.lay_mode, "the demo is for a lay audience");
    for (i, s) in tour.steps.iter().enumerate() {
        let n = (i + 1).to_string();
        send(&mut a, &[&["walkthrough", "step", &n]]).unwrap();
        let host = ownpurse_core::walkthrough::StepHost::of(s);
        let mut want = ownpurse_core::walkthrough::step_text(s);
        if let Some(why) = &host.why {
            want.push_str(&format!(" Why this matters: {why}"));
        }
        let mut got: Vec<String> = Vec::new();
        for _page in 0..10 {
            let buf = snapshot::render(&mut a, 80, 24);
            let p = a.hits.anchors.rect("panel:narration").unwrap();
            for y in p.y + 1..p.bottom() - 1 {
                got.push(snapshot::row_text(&buf, y));
            }
            let more = snapshot::row_text(&buf, p.bottom() - 1).contains("more");
            if !more {
                break;
            }
            for m in snapshot::token_msgs("pagedown").unwrap() {
                update(&mut a, m);
            }
        }
        // pages overlap at the end (the last page fills up): drop lines already read
        let mut seen: Vec<String> = Vec::new();
        for l in got {
            let l = l.trim().to_string();
            if l.is_empty() || seen.last() == Some(&l) || seen.contains(&l) {
                continue;
            }
            seen.push(l);
        }
        assert_eq!(norm(&seen.join(" ")), norm(&want), "step {n} lost text");
    }
}

/// Soft avoid (caretline-layers' `avoid.diff` and `avoid.table` cases, on our screens): a hint at a diff
/// line keeps off the via band and the title it explains; a hint at an Intercompany pair keeps its box and
/// arrow off the other pairs' text. Neither box covers an avoid cell.
#[test]
fn avoid_snapshots() {
    let w = World::new("avoid").correction();
    type Case = (&'static str, &'static [&'static [&'static str]]);
    let cases: [Case; 2] = [
        (
            "avoid-diff",
            &[
                &["manifest", "fix-igniter"],
                &["tab", "7"],
                &["select", "bt-110"],
                &["drill"],
            ],
        ),
        ("avoid-table", &[&["tab", "3"]]),
    ];
    for (name, view) in cases {
        for (tn, th, cw, ch) in [
            ("ledger-dark", theme::LEDGER_DARK, 100, 30),
            ("ledger-dark", theme::LEDGER_DARK, 80, 24),
            ("nocolor-ascii", theme::NO_COLOR_ASCII, 100, 30),
        ] {
            let mut a = w.app(th);
            snapshot::render(&mut a, cw, ch);
            send(&mut a, view).unwrap();
            let hint: Vec<&str> = if name == "avoid-diff" {
                vec![
                    "hint",
                    "diff:/LineItems/n1",
                    "Same line, new account.",
                    "title=The change",
                    "avoid=panel:via,panel:diff-title",
                ]
            } else {
                vec![
                    "hint",
                    "#1",
                    "The recharges between the two companies.",
                    "title=This pair",
                ]
            };
            send(&mut a, &[hint.as_slice()]).unwrap();
            let buf = snapshot::render(&mut a, cw, ch);
            let p = a.hits.plan.clone().unwrap();
            assert!(
                p.layers
                    .iter()
                    .all(|l| l.covers_avoid == 0 && l.rect.is_some()),
                "{name} {cw}x{ch}: {:?}",
                p.layers
            );
            if tn == "nocolor-ascii" {
                ascii_only(name, &snapshot::text(&buf));
            }
            insta::assert_snapshot!(format!("overlay__{name}-{cw}x{ch}-{tn}"), styled(&buf));
        }
    }
}

/// The longest run of blank cells between two content cells on any row of `r`.
fn widest_gap(buf: &ratatui::buffer::Buffer, r: ratatui::layout::Rect) -> (u16, usize) {
    let mut worst = (0, 0);
    for y in r.y..r.bottom() {
        let row: Vec<bool> = (r.x..r.right())
            .map(|x| !buf[(x, y)].symbol().trim().is_empty())
            .collect();
        let (Some(a), Some(b)) = (row.iter().position(|c| *c), row.iter().rposition(|c| *c)) else {
            continue;
        };
        let mut run = 0;
        for c in &row[a..=b] {
            if *c {
                run = 0;
            } else {
                run += 1;
                if run > worst.1 {
                    worst = (y, run);
                }
            }
        }
    }
    worst
}

/// Wide terminals (v0.15.0): the demo walkthrough and the overlay states at 200x50 and 320x80. The
/// narration and explainer panels keep a readable width: no row has a run of more than 40 blank cells
/// between two pieces of content.
#[test]
fn wide_terminals() {
    let w = correction("wide");
    for (cw, ch) in [(200, 50), (320, 80)] {
        let mut a = w.app(dark());
        snapshot::render(&mut a, cw, ch);
        send(&mut a, &[&["walkthrough", "play", "correction"]]).unwrap();
        let n = a.ui.walk.as_ref().unwrap().len();
        for i in 0..n {
            if i > 0 {
                send(&mut a, &[&["walkthrough", "next"]]).unwrap();
            }
            let buf = snapshot::render(&mut a, cw, ch);
            let id = a.ui.walk.as_ref().unwrap().tour.step.clone().unwrap();
            for p in ["panel:narration", "panel:viz"] {
                if let Some(r) = a.hits.anchors.rect(p) {
                    let (y, gap) = widest_gap(&buf, r);
                    assert!(
                        gap <= 40,
                        "{id} {cw}x{ch} {p}: a {gap}-cell gap on row {y}:\n{}",
                        snapshot::text(&buf)
                    );
                }
            }
            insta::assert_snapshot!(
                format!("walk__correction-{:02}-{id}-{cw}x{ch}-ledger-dark", i + 1),
                styled(&buf)
            );
        }
        let world = World::new("wide-overlays");
        for (name, setup) in overlay_states() {
            let mut a = world.app(dark());
            snapshot::render(&mut a, cw, ch);
            setup(&mut a);
            let buf = snapshot::render(&mut a, cw, ch);
            if let Some(r) = a.hits.anchors.rect("panel:narration") {
                let (y, gap) = widest_gap(&buf, r);
                assert!(gap <= 40, "{name} {cw}x{ch}: a {gap}-cell gap on row {y}");
            }
            insta::assert_snapshot!(
                format!("overlay__{name}-{cw}x{ch}-ledger-dark"),
                styled(&buf)
            );
        }
    }
}

/// Every layer whose anchor is a table row has its arrow head on that row (beside it, pointing along it),
/// never on a neighbouring row; and its box doesn't start far right of the row's text.
fn heads_on_their_rows(a: &App, what: &str) {
    let p = a.hits.plan.clone().expect("a plan");
    for pl in &p.layers {
        let row = a
            .ui
            .layers
            .get(&pl.id)
            .and_then(|l| l.anchor.first().cloned())
            .is_some_and(
                |x| matches!(x, caretline_layers::Anchor::Host { ref kind, .. } if kind == "row"),
            );
        if !row {
            continue;
        }
        assert!(
            !ownpurse_tui::overlay::head_off_row(pl),
            "{what}: layer {} has its arrow head off its row: {pl:?}",
            pl.id
        );
        if let (Some(b), Some(an)) = (pl.rect, pl.anchor.as_ref()) {
            let end = an.rects.iter().map(|r| r.right()).max().unwrap_or(0);
            assert!(
                b.x <= end + 12,
                "{what}: layer {} box at column {} is far from the row's text (ends {end})",
                pl.id,
                b.x
            );
        }
    }
}

/// The live overlay review at 214x59 (v0.15.0): a hint on a trial-balance row (anchored on the row's text,
/// not its blank end), a walkthrough step on an Intercompany pair (the head on that row, not the next),
/// and a range spotlight (one hole from the first row to the last).
#[test]
fn overlay_review_214x59() {
    let (cw, ch) = (214, 59);
    let w = correction("review-214");
    // 1. a hint on a TB row
    let mut a = w.app(dark());
    snapshot::render(&mut a, cw, ch);
    send(&mut a, &[&["scope", "roast"], &["tab", "2"]]).unwrap();
    send(
        &mut a,
        &[&[
            "hint",
            "row:code=6140",
            "The igniter is in this balance.",
            "title=Here",
        ]],
    )
    .unwrap();
    let buf = snapshot::render(&mut a, cw, ch);
    heads_on_their_rows(&a, "tb hint");
    insta::assert_snapshot!("overlay__review-tb-row-214x59-ledger-dark", styled(&buf));
    // 2. a walkthrough step on an IC pair row
    let mut a = w.app(dark());
    snapshot::render(&mut a, cw, ch);
    send(&mut a, &[&["tab", "3"]]).unwrap();
    let pair = a.visible_rows()[1].1["pair"].as_str().unwrap().to_string();
    let tour = ownpurse_core::walkthrough::parse(
        &json!({"id": "ic", "title": "IC", "meta": {"audience": "lay"}, "step": [{
            "id": "pair",
            "narration": {"title": "One pair", "text": "Two companies' books about the same money."},
            "anchor": {"host": {"kind": "row", "key": format!("pair={pair}")}},
            "data": {"title": "This pair", "text": "Both sides should agree."},
            "place": {"arrow": true, "ring": true},
            "host": {"view": {"verbs": [["tab", "3"]]}}
        }]})
        .to_string(),
    )
    .unwrap();
    let mut fx = vec![];
    a.walk_start(tour, &mut fx).unwrap();
    let buf = snapshot::render(&mut a, cw, ch);
    heads_on_their_rows(&a, "ic pair step");
    insta::assert_snapshot!("overlay__review-ic-pair-214x59-ledger-dark", styled(&buf));
    // 3. a range spotlight
    let mut a = w.app(dark());
    snapshot::render(&mut a, cw, ch);
    send(&mut a, &[&["scope", "roast"], &["tab", "2"]]).unwrap();
    send(&mut a, &[&["spotlight", "row:code=5020..6830"]]).unwrap();
    let buf = snapshot::render(&mut a, cw, ch);
    let p = a.hits.plan.clone().unwrap();
    let holes: Vec<_> = p.spots.iter().flat_map(|s| s.holes.clone()).collect();
    let r6290 = a.hits.anchors.rect("row:code=6290").unwrap();
    assert!(
        holes.iter().any(|h| h.y <= r6290.y && h.bottom() > r6290.y),
        "the rows inside the range are lit: {holes:?}"
    );
    insta::assert_snapshot!(
        "overlay__review-range-spotlight-214x59-ledger-dark",
        styled(&buf)
    );
}
