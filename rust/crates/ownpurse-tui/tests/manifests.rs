//! Manifests in the TUI (design spec manifests §1–§6): the generic diff view, tab 7 (Entries · Issues ·
//! Diff · Log), the stage strip, the command panel, the apply statuses and the header chip, on a temp copy
//! of the demo record and a temp manifest store (the fixtures are never written). Snapshots: the 18 states
//! of spec §5 at 100×30 (dark, light, plain) and 80×24 (dark). Acceptance: spec §6, items 1–8.
//!
//! Review with `cargo insta review` (or `INSTA_UPDATE=always cargo test -p ownpurse-tui --test manifests`).

use ownpurse_core::manifest::ops::{self, Guard, JLine, Target};
use ownpurse_core::manifest::{Refs, Store, compile};
use ownpurse_core::record::Record;
use ownpurse_tui::app::{App, Msg, Tab, update};
use ownpurse_tui::keymap::{self, Key};
use ownpurse_tui::snapshot::{self, demo_app_at, styled};
use ownpurse_tui::theme::{self, Theme};
use rusqlite::{Connection, params};
use serde_json::{Value, json};
use std::path::{Path, PathBuf};

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

fn tid(org: &str) -> String {
    tenants().into_iter().find(|(a, _)| a == org).unwrap().1
}

fn ctx() -> ownpurse_core::view::Ctx {
    ownpurse_core::view::Ctx {
        orgs: tenants(),
        gate: vec![("hold".into(), "2025-12-31".into())],
    }
}

/// A temp world: a record copy and a manifest store.
struct World {
    dir: PathBuf,
    store: Store,
}

impl World {
    fn new(name: &str) -> World {
        static N: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let n = N.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let dir = std::env::temp_dir().join(format!("xa-tui-mf-{name}-{}-{n}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("db")).unwrap();
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
    fn create(&mut self, name: &str, why: &str) {
        let rec = self.record();
        ops::create(&mut self.store, &rec, &ctx(), "human", name, why).unwrap();
        ownpurse_core::manifest::Paths::beside(self.dir.join("db/manifests.sqlite"))
            .set_current(Some(name))
            .unwrap();
    }
    fn current(&self, name: &str, t: &Target) -> Value {
        let rec = self.record();
        let m = self.store.manifest(name).unwrap();
        let v = ownpurse_core::view::View::open(&rec, &ctx(), Some(&m)).unwrap();
        ops::current(&v, &ctx(), t).unwrap().unwrap()
    }
    fn put(&mut self, name: &str, actor: &str, t: Target, after: Value, why: &str) -> i64 {
        let rec = self.record();
        ops::add(
            &mut self.store,
            &rec,
            &ctx(),
            name,
            actor,
            &t,
            &after,
            why,
            &Refs::default(),
            None,
            &Guard::default(),
        )
        .unwrap()
    }
    fn journal(&mut self, name: &str, actor: &str) -> i64 {
        let doc = ops::journal_doc(
            "2026-01-31",
            "Reclass management charges",
            true,
            &[
                JLine {
                    account: "6420".into(),
                    description: "charge".into(),
                    amount: "263.94".parse().unwrap(),
                },
                JLine {
                    account: "2420".into(),
                    description: String::new(),
                    amount: "-263.94".parse().unwrap(),
                },
            ],
        );
        self.put(
            name,
            actor,
            target("ManualJournals", "design", None),
            doc,
            "reclass the charge",
        )
    }
    fn void(&mut self, name: &str) -> i64 {
        let t = target("ManualJournals", "lab", Some("mj-401"));
        let d = self.current(name, &t);
        self.put(name, "human", t, ops::void_doc(&d), "wrong FX")
    }
    fn recode(&mut self, name: &str, acct: &str) -> i64 {
        let t = target("BankTransactions", "roast", Some("bt-110"));
        let d = self.current(name, &t);
        let after = ops::recode_doc(&d, "L1", Some(acct), None).unwrap();
        self.put(
            name,
            "agent:helper",
            t,
            after,
            "the igniter is a roaster part",
        )
    }
    fn edit_journal(&mut self, name: &str) -> i64 {
        let t = target("ManualJournals", "design", Some("mj-303"));
        let mut d = self.current(name, &t);
        d["Narration"] = json!("Management charge for August, Brackenridge holding fee");
        d["JournalLines"]["n1"]["LineAmount"] = json!("2864.18");
        d["JournalLines"]["n2"]["LineAmount"] = json!("-2864.18");
        d["JournalLines"]["n2"]["AccountCode"] = json!("2250");
        self.put(name, "human", t, d, "the fee was understated")
    }
    fn plan(&mut self, name: &str) -> String {
        let rec = self.record();
        let m = self.store.manifest(name).unwrap();
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
            name,
            "human",
            &path.display().to_string(),
            &hash,
            m.content_rev,
        )
        .unwrap();
        hash
    }
    fn sql(&self) -> Connection {
        Connection::open(self.dir.join("record.sqlite")).unwrap()
    }
    fn approve(&self, hash: &str) {
        let c = self.sql();
        c.execute_batch(
            "CREATE TABLE IF NOT EXISTS approvals (id INTEGER PRIMARY KEY, hash TEXT, verbatim TEXT, via TEXT, \
             where_given TEXT, at TEXT, scope TEXT, recorded_at TEXT)",
        )
        .unwrap();
        c.execute(
            "INSERT INTO approvals (hash, verbatim, via, where_given, at, scope, recorded_at) VALUES (?1, 'approved', 'tty', 'tty', '2026-10-07T12:30:00Z', '\"all\"', '2026-10-07T12:30:00Z')",
            [hash],
        )
        .unwrap();
    }
    /// A write attempt's latest state for entry `seq` of the package `hash`.
    fn attempt(&self, hash: &str, seq: i64, state: &str, detail: Value) {
        let c = self.sql();
        let op = format!("{}:e{seq}", &hash[..12]);
        let _ = c.execute(
            "INSERT INTO operations (op_id, created_at, tenant_id, kind, target_entity, fingerprint, request) \
             VALUES (?1, '2026-10-07T12:40:00Z', 't', 'x', 'ManualJournals', ?1, '{}')",
            [&op],
        );
        let n: i64 = c
            .query_row(
                "SELECT COALESCE(MAX(seq), 0) FROM op_events WHERE op_id = ?1",
                [&op],
                |r| r.get(0),
            )
            .unwrap();
        c.execute(
            "INSERT INTO op_events (op_id, seq, at, state, detail) VALUES (?1, ?2, '2026-10-07T12:41:00Z', ?3, ?4)",
            params![op, n + 1, state, detail.to_string()],
        )
        .unwrap();
    }
    /// A read-back: a newer version of a document (or a new one) in the record copy.
    fn readback(&self, org: &str, entity: &str, id: &str, payload: Value) {
        let c = self.sql();
        let t = tid(org);
        let v: i64 = c
            .query_row(
                "SELECT COALESCE(MAX(version), 0) FROM versions WHERE tenant_id=?1 AND entity=?2 AND xero_id=?3",
                params![t, entity, id],
                |r| r.get(0),
            )
            .unwrap();
        c.execute(
            "INSERT INTO versions (tenant_id, entity, xero_id, version, content_hash, status, observed_at, source, payload) \
             VALUES (?1, ?2, ?3, ?4, 'x', ?5, '2026-10-07T12:42:00Z', 'readback', ?6)",
            params![t, entity, id, v + 1, payload.get("Status").and_then(Value::as_str), payload.to_string()],
        )
        .unwrap();
    }
    fn raw(&self, org: &str, entity: &str, id: &str) -> Value {
        self.record()
            .current_rows(&tid(org), entity)
            .unwrap()
            .into_iter()
            .find(|r| r.xero_id == id)
            .unwrap()
            .payload
    }
    fn app(&self, th: Theme) -> App {
        demo_app_at(
            th,
            &self.dir.join("record.sqlite"),
            &self.dir.join("db/manifests.sqlite"),
        )
    }
}

fn target(entity: &str, org: &str, id: Option<&str>) -> Target {
    Target {
        entity: entity.into(),
        org: org.into(),
        id: id.map(str::to_string),
    }
}

fn keys(app: &mut App, ks: &str) {
    for k in ks.split_whitespace() {
        update(app, Msg::Key(Key::parse(k).expect(k).event()));
        snapshot::render(app, 100, 30);
    }
}

fn text(app: &mut App, w: u16, h: u16) -> String {
    snapshot::text(&snapshot::render(app, w, h))
}

/// The review world: a new journal, an agent's recode to an unknown account, a void, an edited journal.
fn review(name: &str) -> World {
    let mut w = World::new(name);
    w.create("dec-review", "December review");
    w.journal("dec-review", "human");
    w.recode("dec-review", "5029");
    w.void("dec-review");
    w.edit_journal("dec-review");
    w
}

/// The clean world: a new journal and a void (plannable).
fn clean(name: &str) -> (World, i64, i64) {
    let mut w = World::new(name);
    w.create("dec-cleanup", "December cleanup");
    let j = w.journal("dec-cleanup", "human");
    let v = w.void("dec-cleanup");
    (w, j, v)
}

// ── snapshots (spec §5) ──────────────────────────────────────────────────────────────────────────

type Setup = fn() -> (World, &'static str);

fn states() -> Vec<(&'static str, Setup)> {
    vec![
        ("diff-changed-journal", || (review("s1"), "7 :mj-303 enter")),
        ("diff-new-journal", || (review("s2"), "7 :new enter")),
        ("diff-void", || (review("s3"), "7 :mj-401 enter")),
        ("diff-bank-recode", || {
            let mut w = World::new("s4");
            w.create("dec-review", "December review");
            w.recode("dec-review", "5020");
            (w, "7 :bt-110 enter")
        }),
        ("diff-collapsed", || (review("s5"), "7 :mj-303 enter")),
        ("diff-expanded", || (review("s6"), "7 :mj-303 enter z")),
        ("diff-patch-view", || (review("s7"), "7 :mj-303 enter J")),
        ("manifest-entries", || (review("s8"), "7")),
        ("manifest-issues", || (review("s9"), "7 .")),
        ("manifest-issue-jump", || (review("s10"), "7 . enter")),
        ("manifest-stage-check-failed", || (review("s11"), "7")),
        ("manifest-stage-planned", || {
            let (mut w, ..) = clean("s12");
            w.plan("dec-cleanup");
            (w, "7")
        }),
        ("manifest-apply-command", || {
            let (mut w, ..) = clean("s13");
            let h = w.plan("dec-cleanup");
            w.approve(&h);
            (w, "7")
        }),
        ("manifest-applying", || {
            let (mut w, j, v) = clean("s14");
            let h = w.plan("dec-cleanup");
            w.approve(&h);
            w.attempt(&h, j, "confirmed", json!({"target_id": "9f2c1e00-aaaa"}));
            w.attempt(&h, v, "sending", json!({}));
            (w, "7")
        }),
        ("manifest-stopped", || {
            let (mut w, j, v) = clean("s15");
            let h = w.plan("dec-cleanup");
            w.approve(&h);
            w.attempt(&h, j, "confirmed", json!({"target_id": "9f2c1e00-aaaa"}));
            w.attempt(
                &h,
                v,
                "rejected",
                json!({"message": "The journal is in a locked period"}),
            );
            (w, "7")
        }),
        ("manifest-applied", || {
            let (mut w, j, v) = clean("s16");
            let h = w.plan("dec-cleanup");
            w.approve(&h);
            w.attempt(&h, j, "confirmed", json!({}));
            w.attempt(&h, v, "confirmed", json!({}));
            applied_readbacks(&w, j);
            (w, "7")
        }),
        ("header-manifest-applying", || {
            let (mut w, j, _) = clean("s17");
            let h = w.plan("dec-cleanup");
            w.attempt(&h, j, "confirmed", json!({}));
            (w, "1")
        }),
        ("header-manifest-stopped", || {
            let (mut w, j, _) = clean("s18");
            let h = w.plan("dec-cleanup");
            w.attempt(&h, j, "uncertain", json!({}));
            (w, "1")
        }),
    ]
}

/// The read-backs that make the clean world applied: mj-401 voided, the new journal created with its marker.
fn applied_readbacks(w: &World, j: i64) {
    let mut v = w.raw("lab", "ManualJournals", "mj-401");
    v["Status"] = json!("VOIDED");
    w.readback("lab", "ManualJournals", "mj-401", v);
    let m = w.store.manifest("dec-cleanup").unwrap();
    let pkg = compile::package_id_of("dec-cleanup", m.plan().unwrap().from_rev);
    w.readback(
        "design",
        "ManualJournals",
        "mj-new-1",
        json!({"ManualJournalID": "mj-new-1", "Status": "POSTED", "DateString": "2026-01-31T00:00:00",
               "Narration": format!("Reclass management charges · {}", ownpurse_core::view::create_marker(&pkg, j)),
               "LineAmountTypes": "NoTax", "ShowOnCashBasisReports": true,
               "JournalLines": [{"AccountCode": "6420", "Description": "charge", "LineAmount": 263.94},
                                {"AccountCode": "2420", "LineAmount": -263.94}]}),
    );
}

/// `:<id part>` selects the entry row whose target contains it (the bridge's `select`).
fn run(app: &mut App, script: &str) {
    for k in script.split_whitespace() {
        if let Some(part) = k.strip_prefix(':') {
            let rows = app.mf_rows();
            let ids: Vec<String> = rows.iter().map(|r| r.entry.id.clone()).collect();
            let i = rows
                .iter()
                .position(|r| r.entry.id.contains(part))
                .unwrap_or_else(|| {
                    panic!(
                        "{part} not in {ids:?} (manifest {:?}, tab {:?})",
                        app.ui.manifest, app.ui.tab
                    )
                });
            app.ui.manifest_tab.sel = i;
            snapshot::render(app, 100, 30);
            continue;
        }
        keys(app, k);
    }
}

#[test]
fn snapshots() {
    let grid: [(&str, Theme, u16, u16); 4] = [
        ("ledger-dark", theme::LEDGER_DARK, 100, 30),
        ("ledger-light", theme::LEDGER_LIGHT, 100, 30),
        ("nocolor-ascii", theme::NO_COLOR_ASCII, 100, 30),
        ("ledger-dark", theme::LEDGER_DARK, 80, 24),
    ];
    for (name, setup) in states() {
        let (w, script) = setup();
        for (tn, th, cw, ch) in grid {
            let mut app = w.app(th);
            run(&mut app, script);
            let buf = snapshot::render(&mut app, cw, ch);
            let mut shown = buf.clone();
            if name.starts_with("header-") {
                // the header row only
                shown.resize(ratatui::layout::Rect::new(0, 0, cw, 1));
            }
            if tn == "nocolor-ascii" {
                // review 2: every glyph has its ASCII fallback in the plain renders
                let t = snapshot::text(&shown);
                let bad: Vec<char> = t.chars().filter(|c| !c.is_ascii()).collect();
                assert!(
                    bad.is_empty(),
                    "{name}: non-ASCII {bad:?} in the plain render:\n{t}"
                );
            }
            insta::assert_snapshot!(format!("manifest__{name}-{cw}x{ch}-{tn}"), styled(&shown));
        }
    }
}

// ── acceptance (spec §6) ────────────────────────────────────────────────────────────────────────

/// 1. One diff component renders every entity; unknown fields show their raw path.
#[test]
fn one_diff_component_for_every_entity() {
    use ownpurse_core::manifest::fields::doc_diff;
    let mut w = review("a1");
    let mut app = w.app(theme::NO_COLOR_ASCII);
    run(&mut app, "7 :mj-303 enter");
    assert!(text(&mut app, 100, 30).contains("~ Manual journal - design - mj-303"));
    run(&mut app, "esc :bt-110 enter");
    assert!(text(&mut app, 100, 30).contains("~ Bank transaction - roast - bt-110  [reconciled]"));
    // an unknown entity through the same model: raw-path labels
    let d = doc_diff(
        "Widgets",
        "change",
        Some(&json!({"Url": "a", "Name": "x"})),
        &json!({"Url": "b", "Name": "x"}),
    );
    assert!(d.fields.iter().any(|f| f.label == "/Url" && !f.known));
    let _ = &mut w;
}

/// 2. Every changed field or line carries a glyph: no proposed colour or strike-through without a marker.
#[test]
fn every_change_has_a_glyph() {
    let w = review("a2");
    for script in [
        "7 :mj-303 enter",
        "7 :mj-401 enter",
        "7 :bt-110 enter",
        "7 :new enter",
        "7",
    ] {
        let mut app = w.app(theme::LEDGER_DARK);
        run(&mut app, script);
        let buf = snapshot::render(&mut app, 100, 30);
        let proposed = app
            .ui
            .theme
            .fg(ownpurse_tui::theme_tokens::Token::Proposed)
            .fg;
        for y in 3..buf.area.height {
            let row: String = (0..buf.area.width)
                .map(|x| buf[(x, y)].symbol().to_string())
                .collect();
            let marked = (0..buf.area.width).any(|x| {
                let c = &buf[(x, y)];
                (c.fg == proposed.unwrap_or_default()
                    && !c.symbol().trim().is_empty()
                    && !"⎇".contains(c.symbol()))
                    || c.modifier.contains(ratatui::style::Modifier::CROSSED_OUT)
            });
            if marked {
                let has_glyph = ["+", "−", "-", "~", "⊘", "Δ", "→", "≠", "✗", "!"]
                    .iter()
                    .any(|g| row.contains(g));
                assert!(
                    has_glyph,
                    "{script}: row {y} is marked without a glyph: {row}"
                );
            }
        }
    }
}

/// 3. Word highlights only inside text fields; money shows Δ; enums and dates old → new.
#[test]
fn text_words_money_delta_enums_arrows() {
    let w = review("a3");
    let mut app = w.app(theme::NO_COLOR_ASCII);
    run(&mut app, "7 :mj-303 enter z");
    let t = text(&mut app, 120, 40);
    assert!(t.contains("Narration"), "{t}");
    assert!(t.contains("2,764.18") && t.contains("2,864.18"), "{t}");
    assert!(
        t.contains("[-") && t.contains("{+"),
        "word changes read without colour: {t}"
    );
    let mut app = w.app(theme::NO_COLOR_ASCII);
    run(&mut app, "7 :mj-401 enter");
    assert!(text(&mut app, 100, 30).contains("POSTED -> VOIDED"));
    // money Δ on a money field row
    use ownpurse_core::manifest::fields::doc_diff;
    let d = doc_diff(
        "BankTransactions",
        "change",
        Some(&json!({"Total": "1148.63"})),
        &json!({"Total": "1240.00"}),
    );
    assert_eq!(d.fields[0].delta(), Some("91.37".parse().unwrap()));
}

/// 4. Errors block `P`, warnings need acknowledging, and Enter from Issues lands the cursor on the field.
#[test]
fn errors_block_plan_warnings_ack_issue_jump() {
    let w = review("a4");
    let mut app = w.app(theme::NO_COLOR_ASCII);
    run(&mut app, "7 P");
    assert!(
        app.ui
            .note
            .as_deref()
            .unwrap_or("")
            .contains("block planning")
    );
    assert!(app.ui.manifest_tab.prompt.is_none());
    run(&mut app, ". enter");
    let d = app
        .ui
        .manifest_tab
        .diff
        .clone()
        .expect("the issue opened its entry");
    assert!(d.focus.is_some() && d.flash);
    // a warnings-only world: P refused until acknowledged, then the plan confirm (Cancel focused)
    let mut w2 = World::new("a4b");
    w2.create("dec-review", "x");
    let t = target("ManualJournals", "design", Some("mj-303"));
    let d0 = w2.current("dec-review", &t);
    let f = w2.dir.join("support.csv");
    std::fs::write(&f, "a,b\n1,2\n").unwrap();
    w2.put(
        "dec-review",
        "human",
        t,
        ops::attach_doc(&d0, &f).unwrap(),
        "support",
    );
    let mut app = w2.app(theme::NO_COLOR_ASCII);
    run(&mut app, "7 P");
    assert!(
        app.ui
            .note
            .as_deref()
            .unwrap_or("")
            .contains("acknowledging")
    );
    run(&mut app, ". a");
    run(&mut app, "P");
    assert!(matches!(
        app.ui.manifest_tab.prompt,
        Some(ownpurse_tui::manifest_tab::Prompt::Plan { focus: 0 })
    ));
    run(&mut app, "enter");
    assert!(
        app.manifest().unwrap().plan().is_none(),
        "Enter on the focused Cancel does not plan"
    );
    run(&mut app, "P tab enter");
    assert!(app.manifest().unwrap().plan().is_some());
}

/// 5. A landed entry keeps its proposed marks until verified; a verified one has none and the ledger shows
///    no Δ for it.
#[test]
fn landed_keeps_marks_verified_has_none() {
    let (mut w, j, v) = clean("a5");
    let h = w.plan("dec-cleanup");
    w.approve(&h);
    w.attempt(&h, v, "confirmed", json!({}));
    let app = w.app(theme::NO_COLOR_ASCII);
    let row = app
        .mf_rows()
        .into_iter()
        .find(|r| r.entry.seq == v)
        .unwrap();
    assert_eq!(row.marker, "⊘");
    assert!(matches!(
        row.status,
        ownpurse_core::manifest::stage::EntryStatus::Landed(_)
    ));
    applied_readbacks(&w, j);
    let app = w.app(theme::NO_COLOR_ASCII);
    let row = app
        .mf_rows()
        .into_iter()
        .find(|r| r.entry.seq == v)
        .unwrap();
    assert_eq!(row.marker, "");
    assert_eq!(
        row.status,
        ownpurse_core::manifest::stage::EntryStatus::Posted
    );
    let ov = app.manifest_overlay().unwrap();
    assert!(
        ov.is_posted(v) && ov.is_posted(j) && ov.applied.is_empty(),
        "the overlay is a no-op"
    );
}

/// 6. `uncertain` never offers a retry.
#[test]
fn uncertain_never_offers_retry() {
    let (mut w, j, _) = clean("a6");
    let h = w.plan("dec-cleanup");
    w.attempt(&h, j, "uncertain", json!({}));
    let mut app = w.app(theme::NO_COLOR_ASCII);
    run(&mut app, "7");
    let t = text(&mut app, 120, 30);
    assert!(t.contains("do not retry"), "{t}");
    for b in keymap::BINDINGS {
        assert!(!b.action.contains("retry"), "{}", b.action);
    }
}

/// 7. The stage strip and the header's short form agree in every state.
#[test]
fn strip_and_chip_agree() {
    for (name, setup) in states() {
        let (w, _) = setup();
        let mut app = w.app(theme::NO_COLOR_ASCII);
        app.ui.tab = Tab::Manifest;
        let f = app.manifest_facts().unwrap();
        let (chip, _) = app.manifest_chip().map(|(_, s, e)| (s, e)).unwrap();
        let word = f.status.stage.word();
        let t = text(&mut app, 140, 30);
        let strip = t.lines().nth(4).unwrap_or("");
        assert!(
            strip.contains(&format!("> {}", word.split(' ').next().unwrap())),
            "{name}: {strip}"
        );
        let chip_word = chip.split(' ').next().unwrap();
        let ok = match word {
            "draft" | "review" | "check" => chip.contains("change") || chip.starts_with("check"),
            "applying" => chip.starts_with("applying") || chip.starts_with("stopped"),
            other => chip_word == other || chip.starts_with(other),
        };
        assert!(ok, "{name}: stage {word} but chip {chip}");
        let shown = app.ui.theme.text(&chip);
        assert!(
            t.lines().next().unwrap().contains(&shown),
            "{name}: header lacks {shown}"
        );
    }
}

/// 8. No TUI path starts an apply (no key, no palette entry, no agent verb); the panel's command equals the
///    CLI's builder byte for byte and carries the full hash.
#[test]
fn no_tui_apply_and_the_panel_matches_the_cli() {
    // no manifest action applies or approves (other tabs' own words, like Catalog's `apply` of staged rules, are
    // local edits and unrelated)
    for b in keymap::BINDINGS
        .iter()
        .filter(|b| b.action.starts_with("manifest.") || b.action.starts_with("write."))
    {
        assert!(
            !b.action.contains("apply") && !b.action.contains("approve"),
            "binding {}",
            b.action
        );
    }
    for a in keymap::ACTIONS {
        let manifesty = a.id.starts_with("manifest.") || a.id.starts_with("write.");
        assert!(
            !(manifesty && (a.id.contains("apply") || a.id.contains("approve"))),
            "action {}",
            a.id
        );
        assert!(
            !a.label.to_lowercase().contains("apply manifest"),
            "{}",
            a.label
        );
    }
    for v in ["apply", "approve"] {
        let e = ownpurse_core::bridge::verb_error(v);
        assert_eq!(e.0, "forbidden", "{v}");
        assert!(ownpurse_core::bridge::check_verb(&[v.to_string()]).is_err());
    }
    let (mut w, ..) = clean("a8");
    let h = w.plan("dec-cleanup");
    let mut app = w.app(theme::NO_COLOR_ASCII);
    run(&mut app, "7");
    let c = compile::commands("dec-cleanup", &h);
    assert_eq!(app.manifest_commands().unwrap(), c);
    let s = app.manifest_state_view().unwrap();
    assert_eq!(s.approve_command.as_deref(), Some(c.approve.as_str()));
    assert_eq!(s.apply_command.as_deref(), Some(c.apply.as_str()));
    assert!(c.apply.contains(&h) && c.approve.contains(&h) && h.len() == 64);
    // the panel shows each command on ONE line with the hash abbreviated (review 8); `y` copies it in full
    let t = text(&mut app, 160, 30);
    let short = ownpurse_core::manifest::compile::abbreviate(&c.apply, &h).replace('…', "...");
    assert!(t.contains(&short), "{short}\n{t}");
    assert!(!t.contains(&h), "no full hash on screen");
    // `y` copies the approve command (planned)
    let fx = update(&mut app, Msg::Key(Key::parse("y").unwrap().event()));
    assert!(
        fx.iter()
            .any(|e| matches!(e, ownpurse_tui::app::Effect::Copy(s) if *s == c.approve))
    );
}

/// The demo record and the fixture folder are never written by any of the above.
#[test]
fn fixtures_untouched() {
    assert!(!demo().join("db/manifests.sqlite").exists());
    assert!(!demo().join("db/manifests.current").exists());
    assert!(!demo().join("db/branches.current").exists());
    let _ = Path::new("");
}
