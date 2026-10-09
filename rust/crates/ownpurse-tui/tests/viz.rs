//! Explainer panels (walkthroughs §4) on the demo world: every kind at 140×40, 100×30 and 80×24 in the dark,
//! light and plain themes (snapshots), the plain theme pure ASCII, every part anchored (`viz:<id>`, on or
//! off screen), the agent verb `viz` (cosmetic, refused cleanly on a bad spec), the palette's Explain entries,
//! `T` / `M` on a Ledger account, Esc, and V21 with a panel open.

use ownpurse_core::viz::Spec;
use ownpurse_tui::app::{App, Msg};
use ownpurse_tui::keymap::Key;
use ownpurse_tui::quiet::{Counting, Quiet};
use ownpurse_tui::runtime::Runtime;
use ownpurse_tui::snapshot::{self, demo_app, styled};
use ownpurse_tui::theme::{self, Theme};
use ratatui::layout::Rect;
use ratatui::{Terminal, TerminalOptions, Viewport};
use serde_json::{Value, json};

const SIZES: [(u16, u16); 3] = [(140, 40), (100, 30), (80, 24)];

fn themes() -> [(&'static str, Theme); 3] {
    [
        ("ledger-dark", theme::LEDGER_DARK),
        ("ledger-light", theme::LEDGER_LIGHT),
        ("plain", theme::NO_COLOR_ASCII),
    ]
}

/// A temp manifest store with one journal on hold (1470 / 4610), shown: the before → after panel's input.
fn with_manifest(a: &mut App) {
    use ownpurse_core::manifest::Refs;
    use ownpurse_core::manifest::ops::{self, Guard, JLine, Target};
    let ctx = a.manifest_ctx();
    let rec = ownpurse_core::Record::open(&a.source.record).unwrap();
    let store = a.manifest_store().expect("demo manifests store");
    ops::create(store, &rec, &ctx, "human", "fees", "viz test").unwrap();
    let lines = [
        JLine {
            account: "1470".into(),
            description: String::new(),
            amount: "100.13".parse().unwrap(),
        },
        JLine {
            account: "4610".into(),
            description: String::new(),
            amount: "-100.13".parse().unwrap(),
        },
    ];
    let after = ops::journal_doc("2025-12-20", "fee true-up", true, &lines);
    let t = Target {
        entity: "ManualJournals".into(),
        org: "hold".into(),
        id: None,
    };
    ops::add(
        store,
        &rec,
        &ctx,
        "fees",
        "human",
        &t,
        &after,
        "",
        &Refs::default(),
        None,
        &Guard::default(),
    )
    .unwrap();
    a.switch_manifest(Some("fees"), false).unwrap();
}

fn specs() -> Vec<(&'static str, Value)> {
    vec![
        ("bridge", json!({"kind": "bridge", "pair": "hold-design"})),
        (
            "bridge-residual",
            json!({"kind": "bridge", "pair": "design-roast"}),
        ),
        ("flow", json!({"kind": "flow"})),
        ("flow-paid", json!({"kind": "flow", "basis": "paid"})),
        (
            "t-account",
            json!({"kind": "t_account", "org": "hold", "account": "1470"}),
        ),
        (
            "t-account-many",
            json!({"kind": "t_account", "org": "roast", "account": "1010", "top": 4}),
        ),
        (
            "timeline",
            json!({"kind": "timeline", "org": "hold", "account": "1470"}),
        ),
        (
            "before-after",
            json!({"kind": "before_after", "org": "hold"}),
        ),
        (
            "checklist",
            json!({"kind": "checklist", "file": "../viz/close-status-demo.json"}),
        ),
    ]
}

fn open(theme: Theme, name: &str, spec: &Value) -> App {
    let mut a = demo_app(theme, false);
    if name == "before-after" {
        with_manifest(&mut a);
    }
    a.viz_open(Spec::from_value(spec).unwrap());
    a
}

/// The panel's explanatory lines as drawn (its `prose` rects, read back from a fresh render).
fn prose_lines(a: &App, m: &ownpurse_core::viz::Model, w: u16, h: u16) -> Vec<String> {
    let mut buf = ratatui::buffer::Buffer::empty(Rect::new(0, 0, w, h));
    let area = Rect::new(0, 4, w, h - 5);
    let tok = |o: &str| a.viz_org_token(o);
    let scroll = a.ui.viz.as_ref().map_or(0, |v| v.scroll);
    let p = ownpurse_tui::widgets::viz::draw(m, &a.ui.theme, &tok, &mut buf, area, scroll);
    assert!(!p.prose.is_empty(), "every panel has explanatory text");
    p.prose
        .iter()
        .map(|r| {
            (r.x..r.right())
                .map(|x| buf[(x, r.y)].symbol().to_string())
                .collect::<String>()
                .trim_end()
                .to_string()
        })
        .collect()
}

#[test]
fn every_kind_renders_in_every_theme_and_size() {
    let mut n = 0;
    for (name, spec) in specs() {
        for (tname, th) in themes() {
            for (w, h) in SIZES {
                let mut a = open(th, name, &spec);
                let buf = snapshot::render(&mut a, w, h);
                let m = a.viz_model().unwrap().as_ref().unwrap_or_else(|e| {
                    panic!("{name}: {e}");
                });
                let txt = snapshot::text(&buf);
                if tname == "plain" {
                    assert!(
                        txt.is_ascii(),
                        "{name} {w}x{h}: plain is pure ASCII:\n{txt}"
                    );
                }
                // every part is anchored, on screen or off it
                let hits = ownpurse_tui::ui::render(&a, &mut buf.clone(), Rect::new(0, 0, w, h));
                for id in m.ids() {
                    assert!(
                        hits.anchors.get(&format!("viz:{id}")).is_some(),
                        "{name} {w}x{h} {tname}: viz:{id} not anchored"
                    );
                }
                // explanatory text wraps, never cut: no prose line ends in an ellipsis, and the lead is whole
                let prose = prose_lines(&a, m, w, h);
                for l in &prose {
                    assert!(
                        !l.ends_with('…') && !l.ends_with("..."),
                        "{name} {w}x{h} {tname}: cut sentence {l:?}"
                    );
                }
                let joined = prose.join(" ");
                let lead = ownpurse_tui::widgets::viz::t(&a.ui.theme, &m.lead());
                assert!(
                    joined.contains(&lead),
                    "{name} {w}x{h} {tname}: lead not whole in {joined:?}"
                );
                insta::assert_snapshot!(format!("viz-{name}-{w}x{h}-{tname}"), styled(&buf));
                n += 1;
            }
        }
    }
    assert_eq!(n, 9 * 3 * 3);
}

#[test]
fn bridge_waterfall_reads_left_to_right() {
    let mut a = open(theme::LEDGER_DARK, "bridge", &specs()[0].1);
    let txt = snapshot::text(&snapshot::render(&mut a, 140, 40));
    assert!(txt.contains("Why hold and design disagree by 418.27, item by item"));
    assert!(txt.contains("Annual return filing for the parent"));
    assert!(txt.contains("✓ 0.00"), "a 0.00 residual shows ✓");
    let mut a = open(theme::NO_COLOR_ASCII, "bridge", &specs()[0].1);
    let txt = snapshot::text(&snapshot::render(&mut a, 140, 40));
    assert!(txt.contains("ok 0.00"), "{txt}");
    let mut a = open(theme::LEDGER_DARK, "bridge-residual", &specs()[1].1);
    let txt = snapshot::text(&snapshot::render(&mut a, 100, 30));
    assert!(txt.contains("263.94 left unexplained"), "{txt}");
}

#[test]
fn esc_closes_and_tab_change_closes() {
    let mut a = open(theme::LEDGER_DARK, "flow", &specs()[2].1);
    snapshot::render(&mut a, 100, 30);
    assert!(a.ui.viz.is_some());
    snapshot::run_script(&mut a, "esc", 100, 30);
    assert!(a.ui.viz.is_none());
    let mut a = open(theme::LEDGER_DARK, "flow", &specs()[2].1);
    snapshot::run_script(&mut a, "3", 100, 30);
    assert!(a.ui.viz.is_none());
    assert_eq!(a.ui.tab, ownpurse_tui::app::Tab::Intercompany);
}

#[test]
fn scrolling_moves_the_body_and_marks_parts_off_screen() {
    let mut a = open(theme::LEDGER_DARK, "checklist", &specs()[8].1);
    snapshot::render(&mut a, 80, 24);
    snapshot::run_script(&mut a, "down down down end", 80, 24);
    let s = a.ui.viz.as_ref().unwrap().scroll;
    assert!(s > 0, "scrolled");
    let mut buf = ratatui::buffer::Buffer::empty(Rect::new(0, 0, 80, 24));
    let hits = ownpurse_tui::ui::render(&a, &mut buf, Rect::new(0, 0, 80, 24));
    let off = hits
        .anchors
        .get("viz:group/deadlines")
        .expect("the first group is anchored");
    assert!(off.rects.is_empty(), "scrolled above: no rect");
}

#[test]
fn t_and_m_on_a_ledger_account() {
    let mut a = demo_app(theme::LEDGER_DARK, false);
    snapshot::run_script(
        &mut a,
        ": s w i t c h space h o l d enter 2 / 1 4 7 0 enter T",
        100,
        30,
    );
    match &a.ui.viz.as_ref().expect("T opens the T-account").spec {
        Spec::TAccount { org, account, .. } => {
            assert_eq!((org.as_str(), account.as_str()), ("hold", "1470"))
        }
        s => panic!("{s:?}"),
    }
    snapshot::run_script(&mut a, "esc enter M", 100, 30);
    assert!(matches!(
        a.ui.viz
            .as_ref()
            .expect("M on postings opens the timeline")
            .spec,
        Spec::Timeline { .. }
    ));
}

#[test]
fn the_palette_lists_explain_entries() {
    let mut a = demo_app(theme::LEDGER_DARK, false);
    snapshot::render(&mut a, 100, 30);
    let labels: Vec<String> = a
        .palette_filtered("explain")
        .into_iter()
        .map(|e| e.label)
        .collect();
    assert!(
        labels
            .iter()
            .any(|l| l.contains("bridge Brackenridge ↔ Quillfeather"))
    );
    assert_eq!(labels.iter().filter(|l| l.contains("bridge")).count(), 4);
    assert!(labels.iter().any(|l| l.contains("who owes whom")));
    snapshot::run_script(&mut a, ": e x p l a i n space o w e s enter", 100, 30);
    assert!(matches!(a.ui.viz.as_ref().unwrap().spec, Spec::Flow { .. }));
}

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
    let mut rt = Runtime::new(terminal, demo_app(theme::LEDGER_DARK, false));
    rt.frame().unwrap();
    rt
}

fn send(rt: &mut Rt, verbs: &[&[&str]]) -> Value {
    rt.agent(&json!({"v": 1, "kind": "view", "op": "send", "verbs": verbs}))
}

#[test]
fn the_agent_verb_opens_closes_and_refuses_bad_specs() {
    let mut rt = rt();
    let r = send(
        &mut rt,
        &[&["viz", r#"{"kind":"bridge","pair":"hold-lab"}"#]],
    );
    assert_eq!(r["ok"], true, "{r}");
    let st = rt.agent(&json!({"v": 1, "kind": "view", "op": "state"}));
    assert_eq!(st["state"]["viz"]["kind"], "bridge", "{st}");
    let parts: Vec<&str> = st["state"]["viz"]["parts"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap())
        .collect();
    assert_eq!(parts, ["start", "item/1", "residual"]);
    // a bad spec and an unknown pair change nothing
    let r = send(&mut rt, &[&["viz", r#"{"kind":"pie"}"#]]);
    assert_eq!(
        (r["ok"].as_bool(), r["error"].as_str()),
        (Some(false), Some("usage"))
    );
    let r = send(&mut rt, &[&["viz", r#"{"kind":"bridge","pair":"nope"}"#]]);
    assert_eq!(r["error"], "not_found", "{r}");
    assert!(matches!(
        &rt.app.ui.viz.as_ref().unwrap().spec,
        Spec::Bridge { pair, .. } if pair == "hold-lab"
    ));
    // cosmetic: not refused while the user types
    rt.send(Msg::Key(Key::parse("/").unwrap().event()));
    let r = send(&mut rt, &[&["viz", "close"]]);
    assert_eq!(r["ok"], true, "{r}");
    assert!(rt.app.ui.viz.is_none());
}

#[test]
fn v21_a_state_with_a_panel_open_round_trips() {
    for (name, spec) in specs() {
        let mut a = open(theme::LEDGER_DARK, name, &spec);
        snapshot::run_script(&mut a, "down", 100, 30);
        let live = snapshot::render(&mut a, 100, 30);
        let text = a.view_state(a.rev).canonical();
        let mut b = App::from_state_json(&text, a.source.clone(), false).expect("import");
        assert_eq!(
            b.view_state(b.rev).canonical(),
            text,
            "{name}: export → import → export"
        );
        let again = snapshot::render(&mut b, 100, 30);
        assert_eq!(
            snapshot::text(&again),
            snapshot::text(&live),
            "{name}: the imported state draws the live frame"
        );
        assert_eq!(again, live, "{name}: cell for cell");
    }
}

/// The longest run of blank cells between two content cells on any row of `r`.
fn widest_gap(buf: &ratatui::buffer::Buffer, r: Rect) -> (u16, usize) {
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

/// Wide terminals (v0.15.0): every panel keeps a readable width, centred; no row of it has a run of more
/// than 40 blank cells between two pieces of content (the flow's lifelines stop after its last edge).
#[test]
fn panels_stay_readable_on_wide_terminals() {
    for (name, spec) in specs() {
        for (w, h) in [(200, 50), (320, 80)] {
            let mut a = open(theme::LEDGER_DARK, name, &spec);
            let buf = snapshot::render(&mut a, w, h);
            let p = a.hits.anchors.rect("panel:viz").unwrap();
            let (y, gap) = widest_gap(&buf, p);
            assert!(
                gap <= 40,
                "{name} {w}x{h}: a {gap}-cell gap on row {y}:\n{}",
                snapshot::text(&buf)
            );
            insta::assert_snapshot!(format!("viz-{name}-{w}x{h}-ledger-dark"), styled(&buf));
        }
    }
}
