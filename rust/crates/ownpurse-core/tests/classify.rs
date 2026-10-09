//! Classification on the committed demo fixture (`rust/tests/fixtures/demo`: record, `classify.toml`,
//! `db/classify.sqlite`), always on a temp copy: V19 (a changed line renders its label stale, value kept),
//! V14 (only the classification files are written), the fixture's mixed states, two writers on one store.

use ownpurse_core::Record;
use ownpurse_core::classify::config::{Choice, ConfigEdit};
use ownpurse_core::classify::store::NoteTarget;
use ownpurse_core::classify::view::{ClassifyPaths, Filter, Sort, Workspace, WriteCtx};
use ownpurse_core::classify::{Actor, Anchor, BatchId, DocType};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

fn demo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/demo")
}

fn copy(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("xa-classify-core-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(d.join("db")).unwrap();
    for f in [
        "record.sqlite",
        "classify.toml",
        "db/classify.sqlite",
        "tenants.json",
    ] {
        std::fs::copy(demo().join(f), d.join(f)).unwrap();
    }
    d
}

fn orgs(d: &Path) -> Vec<(String, String)> {
    let t: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(d.join("tenants.json")).unwrap()).unwrap();
    let mut v: Vec<(String, String)> = t
        .as_object()
        .unwrap()
        .iter()
        .map(|(a, t)| (a.clone(), t.as_str().unwrap().to_string()))
        .collect();
    v.sort();
    v
}

fn open(d: &Path) -> Workspace {
    let rec = Record::open(d.join("record.sqlite")).unwrap();
    let paths = ClassifyPaths {
        config: d.join("classify.toml"),
        store: d.join("db/classify.sqlite"),
    };
    Workspace::open(&paths, &ownpurse_core::view::View::main(&rec), &orgs(d)).unwrap()
}

fn bt(org: &str, id: &str) -> Anchor {
    Anchor::line(org, DocType::BankTransaction, id, "L1")
}

#[test]
fn the_demo_store_has_mixed_states() {
    let d = copy("mixed");
    let w = open(&d);
    assert_eq!(w.lines().len(), 49);
    assert_eq!(w.store().verify().unwrap() as i64, w.rev());
    let c = w.counts();
    assert_eq!((c.total, c.open, c.stale), (49, 38, 2));
    let all = w.queue(&Filter::parse("status:all").unwrap(), Sort::AmountDesc);
    let glyphs: std::collections::HashSet<char> = all.iter().map(|r| r.glyph.char()).collect();
    for g in ['R', 'H', 'A'] {
        assert!(glyphs.contains(&g), "glyph {g} in the fixture");
    }
    assert!(
        all.iter()
            .any(|r| r.confidence.is_some_and(|c| c < "0.6".parse().unwrap()))
    );
    let detail = w.detail(&bt("design", "bt-303")).unwrap();
    assert_eq!(detail.groups[0].role.as_deref(), Some("origin"));
    assert_eq!(w.groups().len(), 2);
    let undone = w.store().undone();
    assert_eq!(undone.len(), 1, "one mistake undone in the fixture");
    let _ = std::fs::remove_dir_all(&d);
}

/// V19: change a line in a copied record (a new version, as a later sync would append); its label renders
/// stale and keeps its old value.
#[test]
fn v19_a_changed_line_renders_stale() {
    let d = copy("v19");
    let a = bt("design", "bt-303");
    {
        let w = open(&d);
        let l = w.detail(&a).unwrap().labels[0].label.clone().unwrap();
        assert_eq!((l.value.as_str(), l.stale), ("harvest", false));
    }
    let conn = rusqlite::Connection::open(d.join("record.sqlite")).unwrap();
    let (tenant, payload): (String, String) = conn
        .query_row(
            "SELECT tenant_id, payload FROM current WHERE entity='BankTransactions' AND xero_id='bt-303'",
            [],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .unwrap();
    let changed = payload.replace("1148.63", "1184.36");
    assert_ne!(changed, payload);
    conn.execute(
        "INSERT INTO versions (tenant_id, entity, xero_id, version, content_hash, status, observed_at, source, payload) \
         VALUES (?1, 'BankTransactions', 'bt-303', 2, 'test', 'AUTHORISED', '2026-10-06T07:46:00Z', 'sync', ?2)",
        rusqlite::params![tenant, changed],
    )
    .unwrap();
    drop(conn);
    let w = open(&d);
    let rows = w.queue(&Filter::parse("status:stale").unwrap(), Sort::AmountDesc);
    assert!(rows.iter().any(|r| r.anchor == a && r.stale && r.open));
    let l = w.detail(&a).unwrap().labels[0].label.clone().unwrap();
    assert!(l.stale);
    assert_eq!(l.value, "harvest", "the old value stays visible");
    assert_eq!(
        l.changed,
        vec![("amount", "-1148.63".to_string(), "-1184.36".to_string())]
    );
    assert_eq!(w.counts().stale, 3);
    // bulk-accept never takes a stale line
    let p = w.bulk_preview(&Filter::parse("status:all").unwrap(), "0".parse().unwrap());
    assert!(!p.anchors.contains(&a));
    let _ = std::fs::remove_dir_all(&d);
}

fn tree(dir: &Path) -> BTreeMap<String, Vec<u8>> {
    fn walk(base: &Path, d: &Path, m: &mut BTreeMap<String, Vec<u8>>) {
        for e in std::fs::read_dir(d).unwrap() {
            let p = e.unwrap().path();
            if p.is_dir() {
                walk(base, &p, m);
            } else {
                let rel = p.strip_prefix(base).unwrap().display().to_string();
                m.insert(rel, std::fs::read(&p).unwrap());
            }
        }
    }
    let mut m = BTreeMap::new();
    walk(dir, dir, &mut m);
    m
}

/// V14: every kind of classification write lands in the store or the config (plus the config's numbered
/// backup), nowhere else.
#[test]
fn v14_writes_only_the_store_and_the_config() {
    let d = copy("v14");
    let w0 = open(&d);
    drop(w0);
    let before = tree(&d);
    let mut w = open(&d);
    let h = WriteCtx::new(Actor::human(), BatchId::new("v14"));
    let ag = WriteCtx::new(Actor::agent("claude"), BatchId::new("v14-agent")).caption("demo");
    let a = bt("roast", "bt-104");
    w.set_label(&a, "project", "harvest", &h).unwrap();
    w.accept(&bt("roast", "bt-109"), &ag).unwrap();
    w.add_note(
        &NoteTarget::Line(a.clone()),
        "green coffee for the launch",
        &h,
    )
    .unwrap();
    let (_, g) = w.create_group("trip", "Origin trip", &h).unwrap();
    w.add_member(&g, &a, None, &h).unwrap();
    w.undo_line(&a, &h).unwrap();
    w.undo_batch(&h).unwrap();
    let st = w
        .stage_config(&[ConfigEdit::AddChoice {
            label_type: "project".into(),
            choice: Choice {
                id: "market".into(),
                name: "Holiday market stall".into(),
                aliases: vec!["market stall".into()],
                archived: false,
            },
        }])
        .unwrap();
    let out = w.write_config(&st, &ag).unwrap();
    w.undo_events(&out.events, &h).unwrap();
    w.suggest(
        &Filter::default(),
        &WriteCtx::new(Actor::rule("engine"), BatchId::new("s")),
    )
    .unwrap();
    drop(w);
    let after = tree(&d);
    let mut written: Vec<&String> = after
        .iter()
        .filter(|(k, v)| before.get(*k) != Some(v))
        .map(|(k, _)| k)
        .collect();
    written.extend(before.keys().filter(|k| !after.contains_key(*k)));
    for p in &written {
        let ok = p.as_str() == "db/classify.sqlite"
            || p.as_str() == "classify.toml"
            || p.starts_with("classify.toml.~")
            || p.starts_with("record.sqlite-");
        assert!(ok, "unexpected write: {p} (all: {written:?})");
    }
    assert_eq!(after["record.sqlite"], before["record.sqlite"]);
    assert_eq!(
        after["classify.toml"], before["classify.toml"],
        "the config write was undone"
    );
    assert!(after.contains_key("classify.toml.~1~") && after.contains_key("classify.toml.~2~"));
    let _ = std::fs::remove_dir_all(&d);
}

/// The TUI and the CLI may both write: each appends under the lock and re-reads the other's events.
#[test]
fn two_writers_share_one_store() {
    let d = copy("writers");
    let mut a = open(&d);
    let mut b = open(&d);
    let h = WriteCtx::new(Actor::human(), BatchId::new("w"));
    let threads: Vec<_> = (0..2)
        .map(|k| {
            let d = d.clone();
            std::thread::spawn(move || {
                let mut w = open(&d);
                let c = WriteCtx::new(
                    Actor::agent(&format!("w{k}")),
                    BatchId::new(format!("t{k}")),
                );
                for i in 0..20 {
                    w.add_note(
                        &NoteTarget::Line(bt("roast", "bt-112")),
                        &format!("note {k}.{i}"),
                        &c,
                    )
                    .unwrap();
                }
            })
        })
        .collect();
    a.set_label(&bt("lab", "bt-402"), "project", "none", &h)
        .unwrap();
    for t in threads {
        t.join().unwrap();
    }
    b.set_label(&bt("lab", "bt-402"), "treatment", "transfer", &h)
        .unwrap();
    assert!(a.refresh().unwrap());
    assert_eq!(a.rev(), b.rev());
    assert_eq!(a.store().verify().unwrap() as i64, a.rev());
    let d1 = a.detail(&bt("lab", "bt-402")).unwrap();
    assert_eq!(d1.labels[0].label.as_ref().unwrap().value, "none");
    assert_eq!(d1.labels[2].label.as_ref().unwrap().value, "transfer");
    assert_eq!(a.detail(&bt("roast", "bt-112")).unwrap().notes.len(), 41);
    assert_eq!(
        a.queue(
            &Filter::parse("by:agent:w1 status:all").unwrap(),
            Sort::AmountDesc
        )
        .len(),
        1
    );
    let _ = std::fs::remove_dir_all(&d);
}

fn mj(org: &str, id: &str, line: &str) -> Anchor {
    Anchor::line(org, DocType::ManualJournal, id, line)
}

#[test]
fn groups_one_to_one_order_and_a_paired_settlement() {
    use ownpurse_core::classify::groups::ChainCheck;
    let d = copy("groups1");
    let w = open(&d);
    let ids: Vec<String> = w.groups().iter().map(|g| g.id.clone()).collect();
    // the fixture's chain: origin design bt-303 (1,148.63), mirror roast mj-101 L1 +1,148.63, and the
    // repayment's two legs on 2025-08-08: roast bt-108 (1,148.63) and design bt-304 +1,148.63
    let chain = w.group_detail(&ids[0]).unwrap();
    let roles: Vec<Option<&str>> = chain.members.iter().map(|m| m.role.as_deref()).collect();
    assert_eq!(
        roles,
        [
            Some("origin"),
            Some("mirror"),
            Some("settlement"),
            Some("settlement")
        ]
    );
    assert!(chain.check.ok());
    assert!(matches!(chain.check, ChainCheck::OneToOne { .. }));
    assert_eq!(
        chain.check.text(),
        ("origin 1,148.63 = mirror 1,148.63 ✓".to_string(), None)
    );
    assert_eq!(chain.notes.len(), 1);
    // B1: the two legs are ONE settlement, applied once
    assert_eq!(chain.running.len(), 1);
    let s = &chain.running[0];
    assert_eq!(s.legs.len(), 2);
    assert_eq!(s.lone_org, None);
    assert_eq!(s.text(), "2025-08-08 settled 1,148.63  1,148.63 → 0.00");
    // the split group has an origin only
    let split = w.group_detail(&ids[1]).unwrap();
    assert_eq!(
        split.check,
        ChainCheck::Gap {
            reason: "mirror missing".into()
        }
    );
    assert_eq!(split.group.status, "accountant");
    let _ = std::fs::remove_dir_all(&d);
}

#[test]
fn groups_signed_gaps_nets_and_net_zero() {
    use ownpurse_core::classify::groups::ChainCheck;
    let d = copy("groups2");
    let mut w = open(&d);
    let h = WriteCtx::new(Actor::human(), BatchId::new("g"));
    // a size gap, signed
    let (_, g) = w.create_group("intercompany_chain", "Reprint", &h).unwrap();
    w.add_member(&g, &bt("design", "bt-305"), Some("origin"), &h)
        .unwrap();
    w.add_member(&g, &mj("roast", "mj-101", "L1"), Some("mirror"), &h)
        .unwrap();
    let c = w.group_detail(&g).unwrap().check;
    assert!(!c.ok());
    assert_eq!(
        c.text().0,
        "origin (263.94) ≠ mirror 1,148.63: amount differs by 884.69"
    );
    // same size, same direction: not a mirror
    let (_, g1) = w
        .create_group("intercompany_chain", "Two fees", &h)
        .unwrap();
    w.add_member(&g1, &mj("hold", "mj-202", "L1"), Some("origin"), &h)
        .unwrap();
    w.add_member(&g1, &mj("design", "mj-302", "L1"), Some("mirror"), &h)
        .unwrap();
    assert_eq!(
        w.group_detail(&g1).unwrap().check.text().0,
        "origin 2,764.18 and mirror 2,764.18 move the same way: a mirror books the other side"
    );
    // B2: many-to-one uses the SIGNED net; two spends against one remittance
    let (_, g2) = w
        .create_group("owner_paid", "Fees and filings", &h)
        .unwrap();
    w.add_member(&g2, &bt("design", "bt-305"), Some("origin"), &h)
        .unwrap();
    w.add_member(&g2, &bt("design", "bt-307"), Some("origin"), &h)
        .unwrap();
    w.add_member(&g2, &bt("design", "bt-306"), Some("mirror"), &h)
        .unwrap();
    let c = w.group_detail(&g2).unwrap().check;
    assert_eq!(
        c,
        ChainCheck::ManyToOne {
            origins: 2,
            origin_sum: "-682.21".parse().unwrap(),
            mirror: "-2764.18".parse().unwrap(),
            share_pct: "24.7".parse().unwrap(),
        }
    );
    assert_eq!(
        c.text(),
        (
            "2 origins net (682.21) → 1 mirror (2,764.18)".to_string(),
            Some("explains 24.7% of the mirror · no per-line match".to_string())
        )
    );
    assert!(!c.ok(), "many-to-one never claims a line-level match");
    // the two legs of one journal net to zero against a non-zero mirror
    let (_, g3) = w
        .create_group("intercompany_chain", "Autumn fee", &h)
        .unwrap();
    w.add_member(&g3, &mj("design", "mj-301", "L1"), Some("origin"), &h)
        .unwrap();
    w.add_member(&g3, &mj("design", "mj-301", "L2"), Some("origin"), &h)
        .unwrap();
    w.add_member(&g3, &mj("hold", "mj-201", "L2"), Some("mirror"), &h)
        .unwrap();
    let c = w.group_detail(&g3).unwrap().check;
    assert_eq!(
        c.text(),
        (
            "2 origins net 0.00 → 1 mirror (1,906.33)".to_string(),
            Some("explains 0.0% of the mirror · no per-line match".to_string())
        )
    );
    let _ = std::fs::remove_dir_all(&d);
}

#[test]
fn groups_lone_legs_pair_rule_reasons_and_member_suggestions() {
    let d = copy("groups3");
    let mut w = open(&d);
    let h = WriteCtx::new(Actor::human(), BatchId::new("g"));
    // the term loan: the instalment leaves Tallowmere on 07-14 and reaches Brackenridge on 07-15
    let (_, g) = w.create_group("settlement", "Term loan", &h).unwrap();
    w.add_member(&g, &bt("hold", "bt-202"), Some("origin"), &h)
        .unwrap();
    w.add_member(&g, &bt("roast", "bt-106"), Some("mirror"), &h)
        .unwrap();
    w.add_member(&g, &bt("roast", "bt-107"), Some("settlement"), &h)
        .unwrap();
    w.add_member(&g, &bt("hold", "bt-203"), Some("settlement"), &h)
        .unwrap();
    let det = w.group_detail(&g).unwrap();
    assert_eq!(det.pair_rule.as_deref(), Some("hold-roast-loan"));
    assert!(det.check.ok());
    // the two legs are a day apart: ONE settlement (default window ±7 days), applied on the later leg's date
    assert_eq!(det.running.len(), 1);
    assert_eq!(det.running[0].legs.len(), 2);
    assert_eq!(
        det.running[0].text(),
        "2025-07-15 settled (in transit from 07-14) 3,281.19  18,437.62 → 15,156.43"
    );
    let rec = Record::open(d.join("record.sqlite")).unwrap();
    let pc = w
        .pair_check(&g, &ownpurse_core::view::View::main(&rec), &orgs(&d))
        .unwrap()
        .unwrap();
    let rows: Vec<(String, String, bool)> = pc
        .rows
        .iter()
        .map(|r| (r.date.clone(), r.text(), r.ok))
        .collect();
    assert_eq!(
        rows,
        [
            ("2025-07-13".into(), "= 0.00 ✓".into(), true),
            (
                "2025-07-14".into(),
                "≠ 3,281.19: in transit 07-14 → 07-15".into(),
                true
            ),
            ("2025-07-15".into(), "= 0.00 ✓".into(), true),
        ],
        "before the first leg, in transit, after the second: no repeated row"
    );
    // window 0 (configurable): the legs no longer pair; each is applied once and marked
    let p = d.join("classify.toml");
    let t = std::fs::read_to_string(&p).unwrap();
    std::fs::write(
        &p,
        t.replace("[groups]  ", "[groups]\nsettle_window_days = 0\n#"),
    )
    .unwrap();
    assert!(matches!(
        w.reload_config(),
        ownpurse_core::classify::config::Reload::Reloaded { .. }
    ));
    let det = w.group_detail(&g).unwrap();
    assert_eq!(
        det.running.iter().map(|s| s.text()).collect::<Vec<_>>(),
        [
            "2025-07-14 settled (roast leg only) 3,281.19  18,437.62 → 15,156.43",
            "2025-07-15 settled (hold leg only) 3,281.19  15,156.43 → 11,875.24",
        ]
    );
    let pc = w
        .pair_check(&g, &ownpurse_core::view::View::main(&rec), &orgs(&d))
        .unwrap()
        .unwrap();
    assert_eq!(
        pc.rows[1].text(),
        "≠ 3,281.19: hold leg of this settlement is missing (it posts on 2025-07-15)"
    );
    std::fs::write(&p, t).unwrap();
    assert!(matches!(
        w.reload_config(),
        ownpurse_core::classify::config::Reload::Reloaded { .. }
    ));
    // without the hold leg the 07-14 gap stays and nothing explains a later difference
    w.remove_member(&g, &bt("hold", "bt-203"), &h).unwrap();
    let pc = w
        .pair_check(&g, &ownpurse_core::view::View::main(&rec), &orgs(&d))
        .unwrap()
        .unwrap();
    assert_eq!(
        pc.rows[1].text(),
        "≠ 3,281.19: hold leg of this settlement is missing"
    );
    // a difference no leg in the group explains: only the hold leg is a member, but the roast leg posted
    // the day before
    let (_, g2) = w
        .create_group("settlement", "Instalment, one side", &h)
        .unwrap();
    w.add_member(&g2, &bt("hold", "bt-202"), Some("origin"), &h)
        .unwrap();
    w.add_member(&g2, &bt("roast", "bt-106"), Some("mirror"), &h)
        .unwrap();
    w.add_member(&g2, &bt("hold", "bt-203"), Some("settlement"), &h)
        .unwrap();
    let pc = w
        .pair_check(&g2, &ownpurse_core::view::View::main(&rec), &orgs(&d))
        .unwrap()
        .unwrap();
    assert!(
        pc.rows
            .iter()
            .any(|r| r.text() == "≠ 3,281.19: difference unexplained"),
        "{:?}",
        pc.rows
    );
    // member suggestions: the same 2,764.18 within ±7 days in another org
    let s = w.suggest_members(&bt("hold", "bt-204"));
    let got: Vec<String> = s.iter().map(|m| m.anchor.to_string()).collect();
    assert_eq!(got, ["design:bt:bt-306:L1"]);
    assert_eq!(s[0].days_apart, 0);
    assert!(w.suggest_members(&bt("roast", "bt-108")).is_empty());
    let _ = std::fs::remove_dir_all(&d);
}

/// `suggest` is idempotent: with no rule or record change, a second run appends nothing (whatever the filter).
#[test]
fn suggest_twice_appends_nothing_the_second_time() {
    let d = copy("idem");
    let mut w = open(&d);
    let all = Filter::parse("status:all").unwrap();
    let rule = WriteCtx::new(Actor::rule("engine"), BatchId::new("s1"));
    let (first, _) = w.suggest(&all, &rule).unwrap();
    let rev = w.rev();
    for f in ["status:all", "", "org:roast", "status:stale"] {
        let (again, out) = w.suggest(&Filter::parse(f).unwrap(), &rule).unwrap();
        assert_eq!((again.events, out.events.len()), (0, 0), "filter {f:?}");
        assert_eq!(w.rev(), rev);
    }
    assert_eq!(first.events, 0, "the fixture store is already up to date");
    // a fresh store: the first run writes, the second does not
    std::fs::remove_dir_all(d.join("db")).unwrap();
    let mut w = open(&d);
    let (one, _) = w.suggest(&all, &rule).unwrap();
    assert!(one.events > 30);
    let (two, _) = w.suggest(&all, &rule).unwrap();
    assert_eq!(two.events, 0);
    assert_eq!(w.suggest_dry_run(&all).events, 0);
    let _ = std::fs::remove_dir_all(&d);
}

/// A pair rule naming an org that isn't configured is refused at open (file:line) and on a hand-edit reload
/// (the last valid config stays live).
#[test]
fn pair_rule_orgs_are_checked_against_the_configured_aliases() {
    let d = copy("pairorg");
    let p = d.join("classify.toml");
    let good = std::fs::read_to_string(&p).unwrap();
    let bad = good.replace("b = { org = \"roast\"", "b = { org = \"roaster\"");
    assert_ne!(bad, good);
    std::fs::write(&p, &bad).unwrap();
    let rec = Record::open(d.join("record.sqlite")).unwrap();
    let paths = ClassifyPaths {
        config: p.clone(),
        store: d.join("db/classify.sqlite"),
    };
    let e = Workspace::open(&paths, &ownpurse_core::view::View::main(&rec), &orgs(&d))
        .err()
        .unwrap();
    assert_eq!(e.code, "config");
    assert!(
        e.hints[0].starts_with("classify.toml:153: groups.pair_rules.hold-roast-loan.b.org = \"roaster\": unknown org (did you mean \"roast\"?)"),
        "{}",
        e.hints[0]
    );
    std::fs::write(&p, &good).unwrap();
    let mut w = Workspace::open(&paths, &ownpurse_core::view::View::main(&rec), &orgs(&d)).unwrap();
    std::fs::write(&p, &bad).unwrap();
    assert!(matches!(
        w.reload_config(),
        ownpurse_core::classify::config::Reload::Invalid(_)
    ));
    assert_eq!(w.config_file().text, good);
    let _ = std::fs::remove_dir_all(&d);
}

/// `status:` filters the rows AND the counts the same way: no stale labels gives 0 lines, one gives 1; stale
/// lines sort first by default and name the changed field.
#[test]
fn status_filters_rows_and_counts_consistently() {
    let d = copy("status");
    let stale = Filter::parse("status:stale").unwrap();
    {
        let mut w = open(&d);
        let rows = w.queue(&stale, Sort::AmountDesc);
        let c = w.counts_for(&stale);
        assert_eq!((rows.len(), c.total, c.stale), (2, 2, 2));
        // stale first with Sort::StaleFirst (the CLI default), with the changed field
        let all = w.queue(&Filter::parse("status:all").unwrap(), Sort::StaleFirst);
        assert!(!w.queue(&Filter::parse("status:all").unwrap(), Sort::AmountDesc)[0].stale);
        assert!(all[0].stale && all[1].stale && !all[2].stale);
        assert_eq!(w.stale_fields(&all[0].anchor), "amount");
        assert_eq!(w.stale_fields(&all[1].anchor), "date");
        let open_f = Filter::default();
        let c = w.counts_for(&open_f);
        assert_eq!(c.total, w.queue(&open_f, Sort::AmountDesc).len());
        assert_eq!(c.total, c.open, "status:open counts only open lines");
        let all_c = w.counts_for(&Filter::parse("status:all").unwrap());
        assert_eq!(all_c, w.counts());
        // undo one stale label: one left
        let last = w.store().events().last().unwrap().id;
        w.undo_events(&[last], &WriteCtx::new(Actor::human(), BatchId::new("u")))
            .unwrap();
        let c = w.counts_for(&stale);
        assert_eq!(
            (w.queue(&stale, Sort::AmountDesc).len(), c.total, c.stale),
            (1, 1, 1)
        );
    }
    // no stale labels at all: 0 lines
    std::fs::remove_dir_all(d.join("db")).unwrap();
    let w = open(&d);
    let c = w.counts_for(&stale);
    assert_eq!(
        (w.queue(&stale, Sort::AmountDesc).len(), c.total, c.stale),
        (0, 0, 0)
    );
    let _ = std::fs::remove_dir_all(&d);
}

/// Transfer-style rules: account_in + sign, org_in, payee_is_org (a configured org's name, suffixes ignored),
/// bank_transfer and amount_between, on the demo world.
#[test]
fn transfer_style_conditions() {
    use ownpurse_core::classify::config::parse;
    let d = copy("transfer");
    // one bank transfer leg (a version appended to the copied record): Tallowmere moves cash to its card
    let c = rusqlite::Connection::open(d.join("record.sqlite")).unwrap();
    let doc = serde_json::json!({"BankAccount": {"AccountID": "r-bank"}, "BankTransactionID": "bt-190",
        "Contact": {"Name": "Tallowmere Coffee Roasting Co."}, "DateString": "2025-09-29T00:00:00",
        "IsReconciled": true, "LineAmountTypes": "NoTax",
        "LineItems": [{"AccountCode": "1060", "Description": "Card top-up", "LineAmount": "741.13", "TaxAmount": 0}],
        "Status": "AUTHORISED", "Total": "741.13", "Type": "SPEND-TRANSFER"});
    c.execute(
        "INSERT INTO versions (tenant_id, entity, xero_id, version, content_hash, status, observed_at, source, payload) \
         VALUES ('c0ffee00-0000-4000-8000-0000000000a1', 'BankTransactions', 'bt-190', 1, 't', 'AUTHORISED', '2026-10-05T07:46:00Z', 'sync', ?1)",
        [doc.to_string()],
    )
    .unwrap();
    drop(c);
    let w = open(&d);
    let line = |org: &str, id: &str| w.line(&bt(org, id)).unwrap().clone();
    // facts
    assert_eq!(
        line("roast", "bt-106").counterparty_org.as_deref(),
        Some("hold"),
        "\"Brackenridge Lane Holdings\" = hold (Ltd dropped)"
    );
    assert_eq!(
        line("design", "bt-306").counterparty_org.as_deref(),
        Some("hold")
    );
    assert_eq!(
        line("hold", "bt-205").counterparty_org.as_deref(),
        Some("lab"),
        "LLC dropped"
    );
    assert_eq!(line("roast", "bt-101").counterparty_org, None);
    assert_eq!(
        line("roast", "bt-190").counterparty_org,
        None,
        "never the line's own org"
    );
    assert!(line("roast", "bt-190").bank_transfer);
    assert!(!line("roast", "bt-101").bank_transfer);
    let base = std::fs::read_to_string(d.join("classify.toml")).unwrap();
    let count = |when: &str| -> Vec<String> {
        let text = format!(
            "{base}\n[[rules]]\nid = \"r-t\"\nwhen = {{ {when} }}\nset = {{ treatment = \"transfer\" }}\n"
        );
        let cfg = parse(&text, "classify.toml").unwrap_or_else(|e| panic!("{when}: {}", e[0]));
        let r = cfg.rule("r-t").unwrap().clone();
        let wm = w.would_match(&r);
        let mut v: Vec<String> = w
            .lines()
            .iter()
            .filter(|l| ownpurse_core::classify::rules::matches(&cfg, &r, l).is_some())
            .map(|l| l.anchor.to_string())
            .collect();
        v.sort();
        assert_eq!(v.len(), wm.count);
        v
    };
    assert_eq!(
        count("payee_is_org = \"hold\", account_in = [\"2410\"], sign = \"in\""),
        ["roast:bt:bt-106:L1"]
    );
    assert_eq!(
        count("payee_is_org = \"hold\", account_in = [\"2410\"], sign = \"out\""),
        ["roast:bt:bt-107:L1"]
    );
    assert_eq!(
        count("payee_is_org = \"any\", org_in = [\"design\"]"),
        ["design:bt:bt-304:L1", "design:bt:bt-306:L1"]
    );
    assert_eq!(count("bank_transfer = true"), ["roast:bt:bt-190:L1"]);
    assert_eq!(
        count("bank_transfer = true, amount_between = [-1000.07, -500.03], org_in = [\"roast\"]"),
        ["roast:bt:bt-190:L1"]
    );
    assert!(count("bank_transfer = true, amount_between = [0.01, 99.99]").is_empty());
    // a payee_is_org naming no configured org is a config error with file:line
    let bad = format!(
        "{base}\n[[rules]]\nid = \"r-t\"\nwhen = {{ payee_is_org = \"holdco\" }}\nset = {{ treatment = \"transfer\" }}\n"
    );
    let al: Vec<String> = orgs(&d).into_iter().map(|(a, _)| a).collect();
    let issues = ownpurse_core::classify::config::check_orgs(&bad, "classify.toml", &al);
    assert!(
        issues[0]
            .to_string()
            .contains(".when.payee_is_org = \"holdco\": unknown org (did you mean \"hold\"?)"),
        "{}",
        issues[0]
    );
    let _ = std::fs::remove_dir_all(&d);
}
