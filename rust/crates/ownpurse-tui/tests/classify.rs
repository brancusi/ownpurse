//! Tab 6 Classify, the Queue (v0.11.0 slice b; spec classify §3.A, §7, §8; design rulings 2026-10-06).
//! Snapshot grid: 100×30 in ledger-dark, ledger-light and nocolor-ascii, plus 80×24 dark. Behaviour: keys,
//! writes through `update` as `Effect::Classify` (actor human), undo, bulk accept, p+digit, pickers, the
//! note editor, the Jev refusal (V16), the header item, the footer's task order, agents, V13 idle.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ownpurse_tui::app::{App, Effect, Msg, Tab, update};
use ownpurse_tui::classify::{self, Op};
use ownpurse_tui::keymap::Key;
use ownpurse_tui::snapshot::{
    self, classify_states, demo_app, header_problems, row_text, run_script, styled, tab_state_app,
    text,
};
use ownpurse_tui::theme::{self, Theme};
use ownpurse_tui::views::classify::{ListCols, list_cols};
use ratatui::layout::Rect;
use ratatui::{Terminal, TerminalOptions, Viewport};
use serde_json::Value;

fn grid() -> Vec<(&'static str, Theme, (u16, u16))> {
    vec![
        ("ledger-dark", theme::LEDGER_DARK, (100, 30)),
        ("ledger-light", theme::LEDGER_LIGHT, (100, 30)),
        ("nocolor-ascii", theme::NO_COLOR_ASCII, (100, 30)),
        ("ledger-dark", theme::LEDGER_DARK, (80, 24)),
    ]
}

#[test]
fn classify_snapshots() {
    let mut problems = Vec::new();
    let mut n = 0;
    for st in classify_states() {
        for (name, th, (w, h)) in grid() {
            let mut app = tab_state_app(&st, th, w, h);
            let buf = snapshot::render(&mut app, w, h);
            let id = format!("{}-{w}x{h}-{name}", st.name);
            for p in header_problems(&app, &row_text(&buf, 0)) {
                problems.push(format!("{id}: {p}"));
            }
            insta::assert_snapshot!(id, styled(&buf));
            n += 1;
        }
    }
    assert!(problems.is_empty(), "{}", problems.join("\n"));
    assert!(n >= 40, "{n}");
}

fn app_at(script: &str, w: u16, h: u16) -> App {
    let mut a = demo_app(theme::LEDGER_DARK, false);
    run_script(&mut a, script, w, h);
    a
}

fn key(a: &mut App, k: &str) -> Vec<Effect> {
    update(a, Msg::Key(Key::parse(k).unwrap().event()))
}

fn screen(a: &mut App) -> String {
    let (w, h) = a.ui.size;
    text(&snapshot::render(a, w, h))
}

#[test]
fn six_opens_the_tab_and_the_queue_lists_stale_lines_first_then_largest() {
    let mut a = app_at("6", 100, 30);
    assert_eq!(a.ui.tab, Tab::Classify);
    let rows = a.cl_rows().rows;
    assert!(rows.len() > 20 && rows.iter().all(|r| r.open));
    // Sort::StaleFirst: every stale line before the rest, each part largest amount first
    let stale = |r: &ownpurse_core::classify::QueueRow| r.stale;
    assert!(rows.iter().any(stale), "the demo has stale lines");
    assert!(rows.windows(2).all(|w| stale(&w[0]) >= stale(&w[1])));
    assert!(
        rows.windows(2)
            .all(|w| stale(&w[0]) != stale(&w[1]) || w[0].amount.abs() >= w[1].amount.abs())
    );
    let t = screen(&mut a);
    assert!(t.contains("Queue ") && t.contains("Groups") && t.contains("Catalog"));
    assert!(t.lines().nth(1).unwrap().contains("6 Classify"));
}

#[test]
fn accept_is_an_effect_attributed_to_human_and_moves_on() {
    let mut a = app_at("6", 100, 30);
    // past the stale lines (first in the queue): an open line with nothing confirmed yet
    let stale = a.cl_rows().rows.iter().take_while(|r| r.stale).count();
    for _ in 0..stale {
        key(&mut a, "down");
    }
    let first = a.cl_selected().unwrap().anchor.to_string();
    let rev = a.classify.as_ref().unwrap().provider.rev();
    let fx = key(&mut a, "enter");
    // nothing is written inside update: the write is an effect the loop runs
    assert_eq!(a.classify.as_ref().unwrap().provider.rev(), rev);
    let w = fx
        .iter()
        .find_map(|e| match e {
            Effect::Classify(w) => Some(w.clone()),
            _ => None,
        })
        .expect("a classify write");
    assert_eq!(w.actor, "human");
    assert_eq!(w.op, Op::Accept(first.clone()));
    // the cursor already moved to the next line
    assert_ne!(
        a.ui.classify.selected_anchor.as_deref(),
        Some(first.as_str())
    );
    classify::run_effects(&mut a, fx);
    assert!(a.classify.as_ref().unwrap().provider.rev() > rev);
    // every suggested label on the line is now confirmed, by a human
    let an = ownpurse_core::classify::Anchor::parse(&first).unwrap();
    let d = a.classify.as_ref().unwrap().provider.detail(&an).unwrap();
    let confirmed: Vec<_> = d.labels.iter().filter_map(|l| l.label.as_ref()).collect();
    assert!(!confirmed.is_empty());
    assert!(confirmed.iter().all(|l| l.actor.is_human()));
    assert!(a.ui.note.as_deref().unwrap().ends_with("· u undo"));
    // `u` right after: the accept is undone and the cursor goes back to that line
    let fx = key(&mut a, "u");
    classify::run_effects(&mut a, fx);
    let d = a.classify.as_ref().unwrap().provider.detail(&an).unwrap();
    assert!(d.labels.iter().all(|l| l.label.is_none()), "undone");
    assert_eq!(
        a.ui.classify.selected_anchor.as_deref(),
        Some(first.as_str())
    );
}

#[test]
fn a_accepts_like_enter_and_x_skips_without_a_write() {
    let mut a = app_at("6", 100, 30);
    let fx = key(&mut a, "x");
    assert!(fx.is_empty());
    assert_eq!(a.cl_cursor(), Some(1));
    let fx = key(&mut a, "a");
    assert!(matches!(fx.as_slice(), [Effect::Classify(_)]));
}

#[test]
fn p_then_a_digit_sets_the_project_and_digits_still_switch_tabs() {
    let mut a = app_at("6 j", 100, 30);
    let anchor = a.cl_selected().unwrap().anchor.to_string();
    key(&mut a, "p");
    assert!(a.ui.classify.pending_project);
    assert_eq!(
        a.view_state(a.rev).classify.unwrap().editor.as_deref(),
        Some("project")
    );
    let fx = key(&mut a, "2");
    assert_eq!(a.ui.tab, Tab::Classify, "the digit after p is the project");
    assert!(!a.ui.classify.pending_project);
    match fx.as_slice() {
        [Effect::Classify(w)] => assert_eq!(
            w.op,
            Op::SetLabel {
                anchor,
                label_type: "project".into(),
                value: "refit".into(),
            }
        ),
        other => panic!("{other:?}"),
    }
    // without p, 2 is the Ledger tab
    key(&mut a, "2");
    assert_eq!(a.ui.tab, Tab::Ledger);
    // p then any other key cancels
    let mut a = app_at("6 p", 100, 30);
    let fx = key(&mut a, "z");
    assert!(fx.is_empty() && !a.ui.classify.pending_project && a.ui.tab == Tab::Classify);
}

#[test]
fn pickers_filter_by_typing_and_set_the_chosen_value() {
    let mut a = app_at("6 b s a l", 100, 30);
    assert_eq!(
        a.cl_picker_rows(),
        vec![("lab".to_string(), "Saltmarsh Cupping Lab".to_string())]
    );
    let st = a.view_state(a.rev);
    assert_eq!(st.input.as_ref().unwrap().field, "classify_picker");
    assert_eq!(st.busy.as_deref(), Some("in a Classify picker"));
    let fx = key(&mut a, "enter");
    match fx.as_slice() {
        [Effect::Classify(w)] => assert!(matches!(
            &w.op,
            Op::SetLabel { label_type, value, .. } if label_type == "bearer" && value == "lab"
        )),
        other => panic!("{other:?}"),
    }
    assert!(a.ui.classify.picker.is_none());
    // `s` lists the line's own org chart
    let mut a = app_at("6 s", 100, 30);
    let org = a.cl_selected().unwrap().org;
    let codes = a.cl_picker_rows();
    assert!(!codes.is_empty());
    assert!(screen(&mut a).contains("Should-be account"));
    let _ = org;
    // Esc closes without a write
    let fx = key(&mut a, "esc");
    assert!(fx.is_empty() && a.ui.classify.picker.is_none());
}

#[test]
fn the_note_editor_is_one_line_with_ctrl_e_for_multi_line() {
    let mut a = app_at("6 n h i", 100, 30);
    assert_eq!(a.view_state(a.rev).input.unwrap().field, "note");
    let fx = key(&mut a, "enter");
    assert!(
        matches!(fx.as_slice(), [Effect::Classify(w)] if matches!(&w.op, Op::Note(_, t) if t == "hi"))
    );
    let mut a = app_at("6 n a ctrl-e", 100, 30);
    assert!(a.ui.classify.note.as_ref().unwrap().multi);
    let fx = key(&mut a, "enter");
    assert!(fx.is_empty(), "Enter breaks the line in multi-line");
    key(&mut a, "b");
    assert_eq!(a.ui.classify.note.as_ref().unwrap().text, "a\nb");
    let fx = key(&mut a, "ctrl-s");
    assert!(
        matches!(fx.as_slice(), [Effect::Classify(w)] if matches!(&w.op, Op::Note(_, t) if t == "a\nb"))
    );
    // an empty note isn't written
    let mut a = app_at("6 n", 100, 30);
    assert!(key(&mut a, "enter").is_empty());
}

#[test]
fn bulk_preview_focuses_cancel_and_never_includes_stale_or_low_lines() {
    let mut a = app_at("6 A", 100, 30);
    let b = a.ui.classify.bulk.clone().unwrap();
    assert_eq!(
        (b.threshold, b.focus),
        (90, 0),
        "default 0.90, Cancel focused"
    );
    let p = a.cl_bulk_preview().unwrap();
    let rows = a.cl_rows().rows;
    for an in &p.anchors {
        let r = rows.iter().find(|r| &r.anchor == an).unwrap();
        assert!(!r.stale && r.confidence.unwrap() >= classify::hundredths(90));
    }
    let t = screen(&mut a);
    assert!(
        t.contains(&format!("Accept {} lines", p.anchors.len())),
        "{t}"
    );
    // Enter on Cancel writes nothing
    assert!(key(&mut a, "enter").is_empty());
    assert!(a.ui.classify.bulk.is_none());
    // a lower threshold lists more; tab moves focus; Enter accepts them all in one write
    let mut a = app_at("6 A - -", 100, 30);
    assert_eq!(a.ui.classify.bulk.as_ref().unwrap().threshold, 80);
    let n = a.cl_bulk_preview().unwrap().anchors.len();
    assert!(n > p.anchors.len());
    key(&mut a, "tab");
    let fx = key(&mut a, "enter");
    assert!(
        matches!(fx.as_slice(), [Effect::Classify(w)] if matches!(w.op, Op::Bulk { threshold: 80, .. }))
    );
    let open = a.cl_counts().unwrap().open;
    classify::run_effects(&mut a, fx);
    assert_eq!(a.cl_counts().unwrap().open, open - n);
}

#[test]
fn stale_labels_show_stale_with_the_old_and_new_values() {
    let mut a = app_at("6 / s t a l e enter", 100, 30);
    let rows = a.cl_rows().rows;
    assert!(!rows.is_empty() && rows.iter().all(|r| r.stale));
    let t = screen(&mut a);
    assert!(t.contains("line changed: amount"), "{t}");
    assert!(t.matches("stale").count() >= 3);
}

#[test]
fn the_filter_strip_counts_and_bad_filters_say_why() {
    let mut a = app_at("6 / o r g : r o a s t enter", 100, 30);
    assert!(a.cl_rows().rows.iter().all(|r| r.org == "roast"));
    assert!(screen(&mut a).contains("filter: \"org:roast\" · "));
    let a = app_at("6 / c o n f < x enter", 100, 30);
    let st = a.view_state(a.rev).classify.unwrap();
    assert!(st.filter_error.unwrap().contains("conf<"));
    // the scope narrows the queue as well
    let a = app_at("6 ]", 100, 30);
    let org = a.scope_label(&a.ui.scope);
    assert!(a.cl_rows().rows.iter().all(|r| r.org == org));
    // empty
    let mut a = app_at("6 / z z z q q enter", 100, 30);
    assert!(screen(&mut a).contains("Nothing to classify in this filter."));
}

#[test]
fn jev_is_palette_only_disabled_and_refused_while_not_enabled() {
    let mut a = app_at("6 ctrl-p j e v", 100, 30);
    let t = screen(&mut a);
    assert!(
        t.contains("Classify: Ask Jev") && t.contains("Jev not enabled"),
        "{t}"
    );
    key(&mut a, "enter");
    let b = a.ui.band.as_ref().expect("a refusal band");
    assert!(b.text.starts_with("Jev not enabled"));
    assert!(!a.view_state(a.rev).classify.unwrap().jev_enabled);
    // offline: not offered at all
    let mut a = demo_app(theme::LEDGER_DARK, true);
    run_script(&mut a, "6", 100, 30);
    assert!(!a.palette_entries().iter().any(|e| e.label.contains("Jev")));
    // and never bound to a key
    assert!(
        !ownpurse_tui::keymap::BINDINGS
            .iter()
            .any(|b| b.action == "classify.jev")
    );
}

#[test]
fn the_header_item_shows_on_the_tab_and_drops_when_narrow() {
    let mut a = app_at("6", 140, 40);
    assert!(row_text(&snapshot::render(&mut a, 140, 40), 0).contains("classify: "));
    let mut a = app_at("1", 140, 40);
    assert!(!row_text(&snapshot::render(&mut a, 140, 40), 0).contains("classify:"));
    let mut a = app_at("6", 80, 24);
    let row0 = row_text(&snapshot::render(&mut a, 80, 24), 0);
    assert!(!row0.contains("classify:"), "dropped at step 4: {row0}");
    assert!(header_problems(&a, &row0).is_empty());
}

#[test]
fn the_footer_lists_the_task_keys_and_drops_skip_note_treatment_bearer_first() {
    let foot = |w: u16| {
        let mut a = app_at("6", w, 30);
        row_text(&snapshot::render(&mut a, w, 30), 29)
    };
    let wide = foot(160);
    for k in [
        "Enter Accept",
        "p Project",
        "b Bearer",
        "t Treatment",
        "n Note",
        "x Skip",
        "/ Filter",
        "q Quit",
        "⌃P palette",
    ] {
        assert!(wide.contains(k), "{k}: {wide}");
    }
    let order: Vec<usize> = [
        "Enter Accept",
        "p Project",
        "b Bearer",
        "t Treatment",
        "n Note",
        "x Skip",
        "/ Filter",
        "q Quit",
    ]
    .iter()
    .map(|k| wide.find(k).unwrap())
    .collect();
    assert!(order.windows(2).all(|w| w[0] < w[1]), "{wide}");
    for w in [60u16, 80, 100, 120] {
        let f = foot(w);
        for keep in ["Enter Accept", "p Project", "/ Filter", "q Quit", "⌃P"] {
            assert!(f.contains(keep), "{w}: {keep} in {f}");
        }
        // drop order: Skip, Note, Treatment, Bearer
        let has = |k: &str| f.contains(k);
        if has("n Note") {
            assert!(has("t Treatment") && has("b Bearer"), "{w}: {f}");
        }
        if has("t Treatment") {
            assert!(has("b Bearer"), "{w}: {f}");
        }
        if has("x Skip") {
            assert!(has("n Note"), "{w}: {f}");
        }
    }
    let f = foot(100);
    assert!(f.contains("t Treatment") && !f.contains("x Skip"), "{f}");
}

#[test]
fn list_columns_follow_the_ruling() {
    // 100: list 67 = org 6 · date 5 · payee 20 · amount 11 · suggestion 15 · conf 4 (one year)
    let c = list_cols(67, 100, 9, true);
    assert_eq!(
        (
            c.org, c.date, c.payee, c.amount, c.account, c.sugg, c.conf, c.bar
        ),
        (6, 5, 20, 11, 0, 15, 4, false)
    );
    // several years: the full date, taken from payee (never below 14)
    let c = list_cols(67, 100, 9, false);
    assert_eq!((c.date, c.payee, c.sugg), (10, 15, 15));
    // 140: account, full date and the bar
    let c = list_cols(93, 140, 9, false);
    assert!(c.account > 0 && c.date == 10 && c.bar && c.payee >= 14);
    // below 100 (stacked): bar → account → year → suggestion detail; payee ≥ 14, amount ≥ 11
    let at = |w| list_cols(w, 80, 9, false);
    assert!(at(79).bar);
    let degrade: Vec<ListCols> = (56..79).rev().map(at).collect();
    for c in &degrade {
        assert!(c.payee >= 14 && c.amount >= 11);
    }
    let first_no = |f: &dyn Fn(&ListCols) -> bool| (56..79).rev().find(|w| !f(&at(*w)));
    let (bar, acct, year, detail) = (
        first_no(&|c| c.bar),
        first_no(&|c| c.account > 0),
        first_no(&|c| c.date == 10),
        first_no(&|c| c.detail),
    );
    assert!(
        bar >= acct && acct >= year && year >= detail,
        "{bar:?} {acct:?} {year:?} {detail:?}"
    );
}

#[test]
fn the_date_shows_the_year_only_when_the_list_spans_years() {
    let mut a = app_at("6", 100, 30);
    let t = screen(&mut a);
    assert!(
        t.contains("2024-") || t.contains("2025-"),
        "several years: full dates\n{t}"
    );
    let mut a = app_at("6 / m o n t h : 2 0 2 5 - 0 6 enter", 100, 30);
    let row = row_text(&snapshot::render(&mut a, 100, 30), 7);
    assert!(row.contains(" 06-"), "one year: MM-DD: {row}");
}

#[test]
fn stacked_detail_puts_the_labels_first_then_one_facts_line() {
    let mut a = app_at("6", 80, 24);
    let t = screen(&mut a);
    let lines: Vec<&str> = t.lines().collect();
    let p = lines.iter().position(|l| l.contains("Project ")).unwrap();
    let facts = lines
        .iter()
        .position(|l| l.contains(" · ") && l.contains("Halberd"))
        .unwrap_or(99);
    assert!(lines[p + 1].contains("Bearer") && lines[p + 2].contains("Treatment"));
    assert!(facts > p + 2, "{t}");
    // the project keys show only while p is pressed at 80
    assert!(!t.contains("p1 "));
    let mut a = app_at("6 p", 80, 24);
    assert!(screen(&mut a).contains("p1 "));
    // at 100 they are always there, one per line
    let mut a = app_at("6", 100, 30);
    let t = screen(&mut a);
    assert!(t.contains("p1 Harvest Blend launch") && t.contains("p2 Wayfinding refit"));
}

#[test]
fn agents_open_the_tab_filter_and_select_and_are_refused_while_a_picker_is_open() {
    let mut a = demo_app(theme::LEDGER_DARK, false);
    snapshot::agent_send(
        &mut a,
        &[
            &["tab", "classify"],
            &["filter", "org:lab"],
            &["select", "#2"],
        ],
        100,
        30,
    );
    assert_eq!(a.ui.tab, Tab::Classify);
    assert_eq!(a.ui.classify.filter, "org:lab");
    let st: Value = a.state_json(a.rev);
    assert_eq!(st["tab"], "classify");
    assert_eq!(st["mode"], "queue");
    assert_eq!(st["classify"]["filter"], "org:lab");
    let rows = st["rows"].as_array().unwrap();
    assert!(!rows.is_empty());
    let sel = rows.iter().find(|r| r["selected"] == true).unwrap();
    assert_eq!(sel["row"], 2);
    let f = &sel["fields"];
    for k in [
        "org",
        "date",
        "payee",
        "amount",
        "account",
        "suggestion",
        "conf",
        "project",
        "source",
    ] {
        assert!(f.get(k).is_some(), "{k}: {f}");
    }
    assert!(f["amount_value"].is_number());
    assert!(f["conf_value"].is_number() || f["conf_value"].is_null());
    assert!(
        sel["anchor"]["anchor"]
            .as_str()
            .unwrap()
            .starts_with("lab:")
    );
    assert_eq!(st["classify"]["selected_anchor"], sel["anchor"]["anchor"]);
    assert!(st["view"]["total_rows"].as_u64().unwrap() >= rows.len() as u64);
    // a picker open: the user is busy
    run_script(&mut a, "b", 100, 30);
    assert!(a.agent_busy().is_some());
}

#[test]
fn modes_switch_with_comma_and_dot_and_the_others_are_placeholders() {
    let mut a = app_at("6 . .", 100, 30);
    assert_eq!(a.ui.classify.mode, classify::Mode::Audition);
    assert!(screen(&mut a).contains("Audition: coming in this release"));
    key(&mut a, ",");
    key(&mut a, ",");
    key(&mut a, ",");
    assert_eq!(a.ui.classify.mode, classify::Mode::Catalog);
    // the palette has the modes
    let mut a = app_at("1 ctrl-p c l a s s i f y space q u e u e enter", 100, 30);
    assert_eq!(
        (a.ui.tab, a.ui.classify.mode),
        (Tab::Classify, classify::Mode::Queue)
    );
    let _ = screen(&mut a);
}

#[test]
fn a_state_round_trips_over_the_store_and_one_from_before_a_write_is_refused() {
    let mut a = app_at("6", 100, 30);
    let before = a.view_state(a.rev).canonical();
    let fx = key(&mut a, "enter");
    classify::run_effects(&mut a, fx);
    let live = snapshot::render(&mut a, 100, 30);
    // V21 over the store: the state after the write imports (same store revision) and draws the same frame
    let after = a.view_state(a.rev).canonical();
    let mut b = App::from_state_json(&after, a.source.clone(), false).expect("imports");
    assert_eq!(snapshot::render(&mut b, 100, 30), live);
    // the state from before the write names an older store revision: refused, never drawn with other rows
    let err = App::from_state_json(&before, a.source.clone(), true)
        .err()
        .expect("refused even with record drift allowed");
    assert_eq!(err.code(), "classify_changed");
    assert_eq!(err.exit(), 4);
    assert!(
        err.message()
            .contains("classification store changed since this state was saved: rev ")
    );
    // its own allowance: rendered, marked `≠ labels changed`, and it stays drifted on re-export
    let allow = ownpurse_tui::canonical::AllowDrift {
        record: false,
        config: false,
        classify: true,
        manifest: false,
        live: false,
    };
    let mut c = App::from_state_json_with(&before, a.source.clone(), allow).expect("allowed");
    let row0 = row_text(&snapshot::render(&mut c, 100, 30), 0);
    assert!(row0.contains("≠ labels changed"), "{row0}");
    let again = c.view_state(c.rev).canonical();
    assert_eq!(
        App::from_state_json(&again, a.source.clone(), false)
            .err()
            .map(|e| e.code()),
        Some("classify_changed")
    );
}

/// A writer that counts bytes (V13).
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
fn v13_the_queue_idles_at_zero_bytes() {
    let out = Counting::default();
    let backend = ownpurse_tui::quiet::Quiet::fixed(out.clone(), true, 100, 30);
    let terminal = Terminal::with_options(
        backend,
        TerminalOptions {
            viewport: Viewport::Fixed(Rect::new(0, 0, 100, 30)),
        },
    )
    .unwrap();
    let mut rt = ownpurse_tui::runtime::Runtime::new(terminal, demo_app(theme::LEDGER_DARK, false));
    rt.frame().unwrap();
    rt.send(Msg::Key(Key::parse("6").unwrap().event()));
    rt.frame().unwrap();
    let n = *out.0.borrow();
    let t0 = rt.app.now;
    for i in 1..=20 {
        rt.send(Msg::Tick(t0 + i / 4));
        assert!(!rt.frame().unwrap(), "redraw while idle");
    }
    assert_eq!(*out.0.borrow(), n, "idle wrote bytes");
    // an accept through the runtime: the store write, the reload, one frame
    let draws = rt.draws;
    rt.send(Msg::Key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE)));
    assert!(rt.frame().unwrap());
    assert!(!rt.frame().unwrap());
    assert_eq!(rt.draws, draws + 1);
    assert!(rt.app.ui.note.as_deref().unwrap().contains("accepted"));
}

#[test]
fn after_writes_the_queue_equals_a_fresh_query() {
    use ownpurse_core::classify::{Filter, Sort};
    let mut a = app_at("6", 100, 30);
    for k in ["enter", "enter", "u", "j", "p", "3", "x", "b"] {
        let fx = key(&mut a, k);
        classify::run_effects(&mut a, fx);
    }
    let fx = key(&mut a, "enter"); // picks the first bearer in the open picker
    classify::run_effects(&mut a, fx);
    let cached = a.cl_rows().rows;
    let d = a.classify.as_ref().unwrap();
    let fresh = d
        .provider
        .queue(&Filter::parse("").unwrap(), Sort::StaleFirst);
    assert_eq!(cached.as_ref(), &fresh);
    assert_eq!(a.cl_counts().unwrap(), d.provider.counts());
}

#[test]
fn capital_g_adds_the_line_to_a_new_or_nearby_group() {
    let mut a = app_at("6 G", 100, 30);
    assert_eq!(a.cl_picker_rows()[0].1, "New group…");
    assert!(screen(&mut a).contains("Add to group"));
    // New group… → its kind → one write: create the group with the line in it
    key(&mut a, "enter");
    let kinds = a.cl_picker_rows();
    assert!(!kinds.is_empty());
    let fx = key(&mut a, "enter");
    let anchor = a.cl_selected().unwrap().anchor;
    assert!(
        matches!(fx.as_slice(), [Effect::Classify(w)] if matches!(&w.op, Op::NewGroup { kind, .. } if *kind == kinds[0].0))
    );
    classify::run_effects(&mut a, fx);
    assert!(
        a.ui.note.as_deref().unwrap().contains("new"),
        "{:?}",
        a.ui.note
    );
    let d = a
        .classify
        .as_ref()
        .unwrap()
        .provider
        .detail(&anchor)
        .unwrap();
    assert!(!d.groups.is_empty());
    // the same line now offers that group too
    run_script(&mut a, "G", 100, 30);
    assert!(a.cl_picker_rows().len() >= 2);
    // `g` stays the scope picker
    let a = app_at("6 g", 100, 30);
    assert!(a.ui.classify.picker.is_none() && a.ui.overlay.is_some());
}

// ── Catalog (spec classify §3.D, §6) ─────────────────────────────────────────────────────────────

fn config_text(a: &App) -> String {
    std::fs::read_to_string(&a.source.classify.as_ref().unwrap().config).unwrap()
}

fn run(a: &mut App, keys: &str) {
    for k in keys.split_whitespace() {
        let fx = key(a, k);
        classify::run_effects(a, fx);
    }
}

#[test]
fn catalog_stages_shows_the_diff_and_writes_only_after_the_confirm() {
    let mut a = app_at("6 ,", 100, 30);
    assert_eq!(a.ui.classify.mode, classify::Mode::Catalog);
    let before = config_text(&a);
    run(&mut a, "N R o a s t e r y space r e f i t enter");
    assert_eq!(a.ui.classify.catalog.staged.len(), 1);
    assert_eq!(config_text(&a), before, "staged only: nothing on disk");
    assert!(screen(&mut a).contains("1 change to classify.toml · w write · u discard"));
    // w: the diff, Cancel focused; Enter on Cancel writes nothing
    run(&mut a, "w");
    let t = screen(&mut a);
    assert!(
        t.contains("+  { id = \"roastery-refit\", name = \"Roastery refit\" }"),
        "{t}"
    );
    assert_eq!(a.ui.classify.catalog.diff.as_ref().unwrap().focus, 0);
    run(&mut a, "enter");
    assert_eq!(config_text(&a), before);
    assert_eq!(a.ui.classify.catalog.staged.len(), 1, "still staged");
    // w, Tab, Enter: written through the core (atomic replace + a config_written event)
    let rev = a.classify.as_ref().unwrap().provider.rev();
    run(&mut a, "w tab enter");
    assert!(config_text(&a).contains("Roastery refit"));
    assert!(a.ui.classify.catalog.staged.is_empty());
    assert!(
        a.ui.note.as_deref().unwrap().starts_with("wrote "),
        "{:?}",
        a.ui.note
    );
    assert!(
        a.classify.as_ref().unwrap().provider.rev() > rev,
        "a config_written event"
    );
    assert!(a.cat_projects().iter().any(|p| p.1 == "Roastery refit"));
    // the backup the core keeps
    let dir = a
        .source
        .classify
        .as_ref()
        .unwrap()
        .config
        .parent()
        .unwrap()
        .to_path_buf();
    assert!(
        std::fs::read_dir(dir)
            .unwrap()
            .flatten()
            .any(|e| e.file_name().to_string_lossy().contains(".~"))
    );
    // u discards what is staged
    run(&mut a, "N x enter u");
    assert!(a.ui.classify.catalog.staged.is_empty());
}

#[test]
fn deleting_a_used_project_archives_it_and_says_so_first() {
    let mut a = app_at("6 ,", 100, 30);
    let (id, _, _, used, _) = a.cat_projects()[0].clone();
    assert!(used > 0);
    run(&mut a, "D");
    let toast = a.ui.classify.catalog.toast.clone().unwrap();
    assert!(toast.contains("archived instead"), "{toast}");
    let staged = a.cat_staged().unwrap().unwrap();
    assert!(!staged.archived_instead.is_empty());
    run(&mut a, "w tab enter");
    let p = a
        .cat_projects()
        .into_iter()
        .find(|p| p.0 == id)
        .expect("still there (V20)");
    assert!(p.4, "archived");
}

#[test]
fn rule_editor_previews_matches_and_stages_a_valid_rule_only() {
    let mut a = app_at("6 , right enter", 100, 30);
    let m = a.cat_preview().unwrap().unwrap();
    let n = m.count;
    assert!(n > 0 && !m.sample.is_empty() && m.sample.len() <= 5);
    let t = screen(&mut a);
    assert!(t.contains(&format!("would match {n} line")));
    assert!(
        t.contains(&format!(
            "{n} lines · {} already labelled by a person (kept)",
            m.kept
        )),
        "{t}"
    );
    // an invalid confidence is refused in the editor, nothing staged
    run(
        &mut a,
        "shift-tab backspace backspace backspace backspace 7 enter",
    );
    assert!(
        a.ui.classify
            .catalog
            .editor
            .as_ref()
            .unwrap()
            .error
            .as_deref()
            .unwrap()
            .contains("confidence")
    );
    assert!(a.ui.classify.catalog.staged.is_empty());
    run(&mut a, "backspace 0 . 7 enter");
    assert!(a.ui.classify.catalog.editor.is_none());
    assert_eq!(a.ui.classify.catalog.staged.len(), 1);
    let s = a.cat_staged().unwrap().unwrap();
    assert!(s.diff.contains("+confidence = 0.7"), "{}", s.diff);
}

#[test]
fn r_in_the_queue_opens_a_prefilled_rule_with_its_preview() {
    let mut a = app_at("6 j", 100, 30);
    let r = a.cl_selected().unwrap();
    run(&mut a, "R");
    assert_eq!(a.ui.classify.mode, classify::Mode::Catalog);
    let ed = a.ui.classify.catalog.editor.clone().unwrap();
    let get = |k: &str| ed.fields.iter().find(|(n, _)| n == k).unwrap().1.clone();
    assert_eq!(get("payee"), r.payee);
    assert!(get("id").starts_with("r-"));
    let n = a.cat_preview().unwrap().unwrap().count;
    assert!(n >= 1, "matches at least its own line");
    run(&mut a, "enter");
    assert!(matches!(
        a.ui.classify.catalog.staged.as_slice(),
        [classify::catalog::CatEdit::AddRule(_)]
    ));
    assert!(a.agent_busy().is_none());
    run(&mut a, "N");
    assert_eq!(a.agent_busy().as_deref(), Some("editing the catalog"));
}

#[test]
fn hand_edits_reload_live_or_show_the_error_and_keep_the_last_valid_config() {
    let mut a = app_at("6", 100, 30);
    snapshot::hand_edit_valid(&mut a);
    let band = a.ui.result.clone().expect("a band");
    assert!(band.lines[0].starts_with("classify.toml changed on disk · reloaded · "));
    assert!(
        a.cl_projects()
            .iter()
            .any(|p| p.1 == "Wayfinding refit, phase 2")
    );
    let mut a = app_at("6", 100, 30);
    let names = a.cl_projects();
    snapshot::hand_edit_invalid(&mut a);
    let band = a.ui.result.clone().expect("a band");
    assert!(
        band.lines[0].starts_with("classify.toml:"),
        "{:?}",
        band.lines
    );
    assert!(band.lines[0].contains("did you mean"));
    assert_eq!(a.cl_projects(), names, "the last valid config stays live");
    // our own write's echo, or any event that changed nothing: no redraw, no revision (V13)
    let rev = a.rev;
    a.dirty = false;
    let _ = update(&mut a, Msg::ClassifyConfigChanged);
    assert!(!a.dirty);
    assert_eq!(a.rev, rev);
}

#[test]
fn a_hand_edit_while_changes_are_staged_refuses_the_write_once() {
    let mut a = app_at("6 , N R o a s t e r y enter", 100, 30);
    snapshot::hand_edit_valid(&mut a);
    run(&mut a, "w tab enter");
    let b = a.ui.band.clone().expect("refusal");
    assert!(
        b.text.starts_with("classify.toml changed on disk"),
        "{}",
        b.text
    );
    assert!(!config_text(&a).contains("Roastery"));
    assert_eq!(a.ui.classify.catalog.staged.len(), 1, "kept");
    run(&mut a, "esc w tab enter");
    let t = config_text(&a);
    assert!(
        t.contains("Roastery") && t.contains("phase 2"),
        "applied to the new file"
    );
}

#[test]
fn the_watcher_fires_on_a_save_of_classify_toml_only() {
    let paths = snapshot::classify_copy(&snapshot::demo_dir());
    let (tx, rx) = std::sync::mpsc::channel();
    let tx = std::sync::Mutex::new(tx);
    let _w = classify::watch(&paths.config, move || {
        let _ = tx.lock().unwrap().send(());
    })
    .expect("watcher");
    std::thread::sleep(std::time::Duration::from_millis(300));
    // another file in the folder: nothing
    std::fs::write(paths.config.with_file_name("other.txt"), "x").unwrap();
    std::thread::sleep(std::time::Duration::from_millis(500));
    while rx.try_recv().is_ok() {}
    std::fs::write(paths.config.with_file_name("unrelated.toml"), "y").unwrap();
    assert!(
        rx.recv_timeout(std::time::Duration::from_millis(800))
            .is_err()
    );
    // the file itself: an event
    let t = std::fs::read_to_string(&paths.config).unwrap();
    std::fs::write(&paths.config, format!("{t}\n# edited\n")).unwrap();
    assert!(rx.recv_timeout(std::time::Duration::from_secs(60)).is_ok());
}

/// Overlays never let the view behind show through (design, 2026-10-06): inside each box's rectangle plus
/// its 1-column margin, and in any sliver of 1–2 columns left between that and the screen edge, no cell
/// carries the view's content. "The view behind" is the same state with the overlay closed (Esc).
#[test]
fn overlays_clear_their_area_and_leave_no_slivers() {
    let states = [
        "6 j b",
        "6 j t",
        "6 j s",
        "6 j G",
        "6 j n a b c",
        "6 , enter",
        "6 , N",
        "6 , right enter",
        "6 j R",
        // v0.12.0: the shell's overlays too (palette, picker, help)
        "ctrl-p",
        "g",
        "?",
    ];
    for script in states {
        for (w, h) in [(100u16, 30u16), (80, 24), (140, 40)] {
            let mut a = app_at(script, w, h);
            let with = snapshot::render(&mut a, w, h);
            let mut b = app_at(&format!("{script} esc"), w, h);
            let behind = snapshot::render(&mut b, w, h);
            let sym =
                |buf: &ratatui::buffer::Buffer, x: u16, y: u16| buf[(x, y)].symbol().to_string();
            // the box: its corners
            let (mut top, mut left, mut bottom, mut right) = (None, None, None, None);
            for y in 0..h {
                for x in 0..w {
                    match sym(&with, x, y).as_str() {
                        "╭" | "+" if top.is_none() && sym(&with, x + 1, y) == "─" => {
                            top = Some(y);
                            left = Some(x);
                        }
                        "╯" => {
                            bottom = Some(y);
                            right = Some(x);
                        }
                        _ => {}
                    }
                }
            }
            let (Some(t), Some(l), Some(bm), Some(r)) = (top, left, bottom, right) else {
                panic!("{script} {w}x{h}: no box");
            };
            let mut x0 = l.saturating_sub(1);
            let mut x1 = (r + 1).min(w - 1);
            if x0 <= 2 {
                x0 = 0;
            }
            if w - 1 - x1 <= 2 {
                x1 = w - 1;
            }
            for y in t..=bm {
                for x in x0..=x1 {
                    if (l..=r).contains(&x) {
                        continue; // the box itself draws there
                    }
                    let s = sym(&with, x, y);
                    assert!(
                        s.trim().is_empty() || s != sym(&behind, x, y),
                        "{script} {w}x{h}: ({x},{y}) shows {s:?} from the view behind\n{}",
                        text(&with)
                    );
                }
            }
        }
    }
}

// ── agent writes (spec classify §5a) ─────────────────────────────────────────────────────────────

use serde_json::json;

#[test]
fn an_agent_write_lands_attributed_with_a_notice_an_activity_row_and_the_a_glyph() {
    let mut a = app_at(
        "6 / o r g : d e s i g n space s t a t u s : a l l enter",
        100,
        30,
    );
    let rev = a.classify.as_ref().unwrap().provider.rev();
    assert_eq!(a.view_state(a.rev).classify.unwrap().store_rev, rev);
    let target = snapshot::rows_by_amount(&mut a)[1].anchor.to_string();
    let reply = snapshot::agent_classify(
        &mut a,
        json!({"op": "set", "anchors": [target],
            "labels": {"project": "refit", "bearer": "design", "treatment": "owner-paid"}}),
    );
    assert_eq!(reply["ok"], true, "{reply}");
    assert!(reply["store_rev"].as_i64().unwrap() > rev);
    let an = ownpurse_core::classify::Anchor::parse(&target).unwrap();
    let d = a.classify.as_ref().unwrap().provider.detail(&an).unwrap();
    // types a person confirmed are skipped (kept); the rest are the agent's
    assert!(
        d.labels
            .iter()
            .filter_map(|l| l.label.as_ref())
            .any(|l| l.actor.as_str() == "agent:claude")
    );
    assert!(
        d.labels
            .iter()
            .filter_map(|l| l.label.as_ref())
            .all(|l| l.actor.as_str() == "agent:claude" || l.actor.is_human())
    );
    let t = screen(&mut a);
    assert!(
        t.contains("◆ agent: project Wayfinding refit +2 on 1 line") && t.contains("· U undo"),
        "{t}"
    );
    assert_eq!(a.agent.writes.len(), 1);
    assert!(
        a.agent
            .log
            .iter()
            .any(|r| r.path.starts_with("classify set · "))
    );
    let row = a
        .cl_rows()
        .rows
        .iter()
        .position(|r| r.anchor == an)
        .unwrap();
    let row_text = t.lines().nth(7 + row).unwrap();
    assert!(
        row_text.trim_end().ends_with("A   │") || row_text.contains(" A "),
        "{row_text}"
    );
    // the notice stays through cursor moves, and goes with the view
    run(&mut a, "j");
    assert!(a.agent_notice_text().is_some());
    run(&mut a, "1");
    assert!(a.agent_notice_text().is_none());
    // `by:agent since:today` lists what agents wrote
    let mut b = a;
    run(&mut b, "6");
    b.ui.classify.filter = "by:agent since:today status:all".into();
    assert!(b.cl_rows().rows.iter().any(|r| r.anchor == an));
}

#[test]
fn a_caption_replaces_the_notice_text_and_keeps_u_undo() {
    let mut a = app_at("6", 100, 30);
    let t = snapshot::rows_by_amount(&mut a)[2].anchor.to_string();
    snapshot::agent_classify(
        &mut a,
        json!({"op": "note", "anchor": t, "text": "checked the invoice", "caption": "noted the Halberd line"}),
    );
    let (n, _) = a.agent_notice_text().unwrap();
    assert!(n.ends_with("noted the Halberd line · U undo"), "{n}");
}

#[test]
fn an_agent_write_to_a_line_the_user_has_open_is_refused_user_busy() {
    let mut a = app_at("6 j b", 100, 30);
    let open = a.cl_selected().unwrap().anchor.to_string();
    let other = a.cl_rows().rows[5].anchor.to_string();
    let rev = a.classify.as_ref().unwrap().provider.rev();
    let reply = snapshot::agent_classify(&mut a, json!({"op": "accept", "anchors": [open]}));
    assert_eq!(reply["error"], "user_busy", "{reply}");
    assert_eq!(reply["exit"], 4);
    assert_eq!(
        a.classify.as_ref().unwrap().provider.rev(),
        rev,
        "nothing written"
    );
    assert!(a.ui.classify.picker.is_some(), "the picker stays open");
    // a line the user isn't on lands, without moving the user
    let sel = a.ui.classify.selected_anchor.clone();
    let reply = snapshot::agent_classify(&mut a, json!({"op": "accept", "anchors": [other]}));
    assert_eq!(reply["ok"], true, "{reply}");
    assert!(a.ui.classify.picker.is_some());
    assert_eq!(a.ui.classify.selected_anchor, sel);
    // a stale store revision: view_changed
    let reply = snapshot::agent_classify(
        &mut a,
        json!({"op": "accept", "anchors": [other], "expect_store_rev": 1}),
    );
    assert_eq!(reply["error"], "view_changed", "{reply}");
}

#[test]
fn when_the_users_line_leaves_the_queue_the_cursor_keeps_its_place() {
    let mut a = app_at("6 j j", 100, 30);
    let mine = a.cl_selected().unwrap().anchor.to_string();
    let reply = snapshot::agent_classify(
        &mut a,
        json!({"op": "set", "anchors": [mine],
            "labels": {"project": "refit", "bearer": "design", "treatment": "owner-paid"}}),
    );
    assert_eq!(reply["ok"], true, "{reply}");
    assert_eq!(a.cl_cursor(), Some(2), "same place in the list");
}

#[test]
fn capital_u_undoes_agent_batches_one_by_one() {
    let mut a = app_at("6", 100, 30);
    let rows = a.cl_rows().rows;
    let (x, y) = (rows[3].anchor.to_string(), rows[4].anchor.to_string());
    snapshot::agent_classify(&mut a, json!({"op": "accept", "anchors": [x]}));
    snapshot::agent_classify(&mut a, json!({"op": "accept", "anchors": [y]}));
    assert_eq!(a.view_state(a.rev).classify.unwrap().agent_writes, 2);
    run(&mut a, "U");
    let (n, _) = a.agent_notice_text().unwrap();
    assert!(n.starts_with("agent write undone · 1 line"), "{n}");
    run(&mut a, "U");
    assert_eq!(a.view_state(a.rev).classify.unwrap().agent_writes, 0);
    let d = a.classify.as_ref().unwrap();
    for an in [&x, &y] {
        let det = d
            .provider
            .detail(&ownpurse_core::classify::Anchor::parse(an).unwrap())
            .unwrap();
        assert!(
            det.labels
                .iter()
                .filter_map(|l| l.label.as_ref())
                .all(|l| !l.actor.is_agent()),
            "{an} undone"
        );
    }
    run(&mut a, "U");
    assert_eq!(a.ui.note.as_deref(), Some("No agent write to undo."));
}

#[test]
fn activity_offers_undo_this_write_with_cancel_focused() {
    let mut a = app_at("6", 100, 30);
    let x = a.cl_rows().rows[3].anchor.to_string();
    snapshot::agent_classify(&mut a, json!({"op": "accept", "anchors": [x]}));
    run(&mut a, "4");
    // the newest row is the write
    let b = {
        run(&mut a, "home enter");
        a.ui.band.clone().expect("the confirm")
    };
    assert!(b.text.starts_with("Undo this write? "), "{}", b.text);
    assert_eq!(b.focus, 0, "Cancel focused");
    run(&mut a, "enter");
    assert!(
        a.agent.writes.iter().all(|w| !w.undone),
        "Cancel undid nothing"
    );
    run(&mut a, "home enter tab enter");
    assert!(a.agent.writes[0].undone);
}

#[test]
fn the_bridge_accepts_classify_ops_only() {
    use ownpurse_tui::bridge::precheck;
    assert!(precheck(r#"{"v":1,"kind":"classify","op":"set"}"#).is_ok());
    let e = precheck(r#"{"v":1,"kind":"classify","op":"drop_table"}"#).unwrap_err();
    assert_eq!(e["error"], "usage");
}

#[test]
fn the_agent_write_notice_is_built_from_segments_and_never_cut_mid_word() {
    use ownpurse_tui::classify::WroteNotice;
    let n = WroteNotice {
        prefix: "project ".into(),
        name: Some("Wayfinding refit".into()),
        suffix: " on 2 lines".into(),
        kept: 1,
    };
    let lead = "◆ agent: ";
    assert_eq!(
        n.fitted(lead, 200, false),
        "◆ agent: project Wayfinding refit on 2 lines · 1 kept (yours) · U undo"
    );
    // the name shortens first …
    assert_eq!(
        n.fitted(lead, 66, false),
        "◆ agent: project Wayfinding… on 2 lines · 1 kept (yours) · U undo"
    );
    // … then the kept segment goes; `· U undo` never does
    assert_eq!(
        n.fitted(lead, 50, false),
        "◆ agent: project Wayfinding… on 2 lines · U undo"
    );
    for w in 20..80 {
        let t = n.fitted(lead, w, false);
        assert!(t.ends_with(" · U undo"), "{w}: {t}");
        for word in t
            .trim_end_matches(" · U undo")
            .trim_start_matches(lead)
            .split(' ')
        {
            let bare = word.trim_end_matches('…');
            assert!(
                [
                    "project",
                    "Wayfinding",
                    "refit",
                    "on",
                    "2",
                    "lines",
                    "·",
                    "1",
                    "kept",
                    "(yours)",
                    ""
                ]
                .contains(&bare),
                "{w}: `{word}` cut mid-word in {t}"
            );
        }
    }
    // the other verbs' copy
    let mut a = app_at("6 ,", 100, 30);
    snapshot::agent_writes_config(&mut a);
    let foot = |a: &mut App, w: u16, h: u16| row_text(&snapshot::render(a, w, h), h - 1);
    assert!(
        foot(&mut a, 100, 30)
            .contains("◆ agent: project \"Tasting tour\" added to classify.toml · U undo"),
        "{}",
        foot(&mut a, 100, 30)
    );
    // the labels state at 100 and 80 (as rendered)
    let mut a = app_at(
        "6 / o r g : d e s i g n space s t a t u s : a l l enter",
        100,
        30,
    );
    snapshot::agent_writes_labels(&mut a);
    assert!(
        foot(&mut a, 100, 30)
            .contains("◆ agent: project Wayfinding… +2 on 2 lines · 1 kept (yours) · U undo")
    );
    assert!(foot(&mut a, 80, 24).contains("◆ agent: project Wayfinding… +2 on 2 lines · U undo"));
    let mut a = app_at("6", 100, 30);
    let t = snapshot::rows_by_amount(&mut a)[2].anchor.to_string();
    snapshot::agent_classify(
        &mut a,
        serde_json::json!({"op": "note", "anchor": t, "text": "x"}),
    );
    let f = foot(&mut a, 100, 30);
    assert!(
        f.contains("◆ agent: note on ") && f.contains(" · U undo"),
        "{f}"
    );
}

// ── Groups (spec classify §3.E) ──────────────────────────────────────────────────────────────────

#[test]
fn groups_lists_groups_and_checks_their_chains() {
    let mut a = app_at("6 .", 100, 30);
    assert_eq!(a.ui.classify.mode, classify::Mode::Groups);
    assert_eq!(a.grp_rows().len(), 2);
    let t = screen(&mut a);
    assert!(
        t.contains("Hollowpine proofs recharge") && t.contains("origin 1,148.63 = mirror 1,148.63"),
        "{t}"
    );
    snapshot::group_gap(&mut a);
    assert!(screen(&mut a).contains("Check  mirror missing"));
    snapshot::group_aggregate(&mut a);
    let t = screen(&mut a);
    assert!(
        t.contains("2 origins net (3,693.56) → 1 mirror (18,437.62)"),
        "{t}"
    );
    assert!(t.contains("explains 20.0% of the mirror"), "{t}");
    snapshot::group_running(&mut a);
    let t = screen(&mut a);
    assert!(
        t.contains("Running balance") && t.contains("18,437.62 → 15,156.43"),
        "{t}"
    );
    assert!(
        t.contains("Pair rule  Term loan: Brackenridge 1460 = Tallowmere 2410"),
        "{t}"
    );
    let p = a.grp_pair_check().unwrap();
    // both legs are members: the rule holds before, in transit and after
    assert!(p.rows.iter().all(|r| r.ok), "the record's own numbers");
}

#[test]
fn enter_opens_a_members_document_and_esc_comes_back() {
    let mut a = app_at("6 . enter", 100, 30);
    assert_eq!(
        a.ui.classify.groups.focus,
        classify::groups::GFocus::Members
    );
    run(&mut a, "enter");
    assert!(a.ui.classify.groups.doc.is_some());
    assert!(
        matches!(a.cl_doc.as_deref(), Some(Ok(_))),
        "the record has it"
    );
    let t = screen(&mut a);
    assert!(t.contains("Bag artwork") || t.contains("Contact"), "{t}");
    assert_eq!(a.ui.tab, Tab::Classify, "stays in the Classify chain");
    run(&mut a, "esc");
    assert!(a.ui.classify.groups.doc.is_none());
    run(&mut a, "esc");
    assert_eq!(a.ui.classify.groups.focus, classify::groups::GFocus::List);
}

#[test]
fn groups_keys_write_role_member_note_status_and_removal() {
    let mut a = app_at("6 .", 100, 30);
    let id = snapshot::make_group(
        &mut a,
        "intercompany_chain",
        "Equipment advance",
        &[("hold", "-6912.48", "origin")],
    );
    let members = |a: &App| {
        a.classify
            .as_ref()
            .unwrap()
            .provider
            .group_detail(&id)
            .unwrap()
            .members
    };
    // m: the suggested member (same amount, same day, another org)
    run(&mut a, "right m");
    let rows = a.cl_picker_rows();
    assert!(
        rows[0].1.starts_with("lab · 2025-02-26 · 6,912.48"),
        "{rows:?}"
    );
    run(&mut a, "enter");
    assert_eq!(members(&a).len(), 2);
    // R: its role
    run(&mut a, "down R m i r r o r enter");
    assert!(
        members(&a)
            .iter()
            .any(|m| m.role.as_deref() == Some("mirror"))
    );
    assert!(screen(&mut a).contains("origin 6,912.48 = mirror 6,912.48"));
    // n: a note on the group
    run(&mut a, "n s a m e space d a y enter");
    let d = a
        .classify
        .as_ref()
        .unwrap()
        .provider
        .group_detail(&id)
        .unwrap();
    assert_eq!(d.notes.last().unwrap().text, "same day");
    assert!(d.notes.last().unwrap().actor.is_human());
    // S: status
    run(&mut a, "S c o m p enter");
    assert_eq!(a.grp_selected().unwrap().status, "complete");
    // D: remove the member under the cursor
    run(&mut a, "D");
    assert_eq!(members(&a).len(), 1);
}

#[test]
fn every_group_shows_its_own_id() {
    let mut a = app_at("6 .", 100, 30);
    snapshot::group_gap(&mut a);
    snapshot::group_aggregate(&mut a);
    snapshot::group_running(&mut a);
    let rows = a.grp_rows();
    assert!(rows.len() >= 5);
    let mut seen = std::collections::HashSet::new();
    for (i, g) in rows.iter().enumerate() {
        a.ui.classify.groups.sel = Some(g.id.clone());
        let t = screen(&mut a);
        let header = t.lines().nth(5).unwrap_or("");
        assert!(
            header.contains(&format!("· {}", g.id)),
            "{i}: {} not in {header:?}",
            g.id
        );
        assert!(seen.insert(g.id.clone()), "ids are distinct");
    }
}

#[test]
fn a_paired_settlement_is_one_calm_row_and_the_check_takes_two_lines_at_most() {
    let mut a = app_at("6 .", 100, 30);
    snapshot::group_running(&mut a);
    let t = screen(&mut a);
    assert!(
        t.contains("2025-07-15 settled (in transit from 07-14) 3,281.19"),
        "{t}"
    );
    assert_eq!(
        t.matches("settled").count(),
        1,
        "the two legs are one settlement"
    );
    let p = a.grp_pair_check().unwrap();
    assert!(p.rows.iter().all(|r| r.ok), "in transit is ok");
    assert!(t.contains("in transit 07-14 → 07-15"), "{t}");
    for (w, h) in [(100u16, 30u16), (80, 24)] {
        let mut a = app_at("6 .", w, h);
        snapshot::group_aggregate(&mut a);
        let t = screen(&mut a);
        let lines: Vec<&str> = t.lines().collect();
        let i = lines.iter().position(|l| l.contains("Check  ")).unwrap();
        assert!(
            lines[i].contains("→ 1 mirror (18,437.62)") && !lines[i].contains("cannot check"),
            "{w}: {t}"
        );
        assert!(
            lines[i + 1].contains("explains 20.0% of the mirror · no per-line match"),
            "{w}: {t}"
        );
        assert!(!lines[i + 2].contains("match"), "{w}: two lines at most");
        assert!(!t.contains('…') || !lines[i].contains('…'), "never cut");
    }
}
