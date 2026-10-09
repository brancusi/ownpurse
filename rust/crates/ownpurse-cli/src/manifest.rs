//! `ownpurse manifest …`: collect changes to the books as document before → after entries (JSON Patch on
//! normalised documents), review them as a diff, CHECK them locally, and PLAN them into the sealed write
//! package. Nothing here calls Xero. `manifest approve` and `manifest apply` are thin wrappers over
//! `write approve-record` and `write run` with the same locks (apply still needs
//! `OWNPURSE_WRITE_RUN=<hash>`); the command lines come from the core's one builder.

use crate::app::{App, CliError, Res};
use crate::obj;
use crate::out::{Out, dec};
use ownpurse_core::manifest::check::{self, Options, Report};
use ownpurse_core::manifest::ops::{self, Guard, JLine, Target};
use ownpurse_core::manifest::stage::{self, EntryStatus};
use ownpurse_core::manifest::{Entry, Kind, Manifest, Refs, Store, compile, effects};
use ownpurse_core::view::View;
use rust_decimal::Decimal;
use serde_json::{Value, json};
use std::path::{Path, PathBuf};
use std::str::FromStr;

/// The writer's flags: who (`--client`), the head they read (`--expect-rev`), `--overwrite`.
#[derive(Debug, Clone, Default)]
pub struct WriteFlags {
    pub client: Option<String>,
    pub expect_rev: Option<i64>,
    pub overwrite: bool,
}

impl WriteFlags {
    fn actor(&self) -> String {
        match &self.client {
            Some(c) if !c.trim().is_empty() => format!("agent:{}", c.trim()),
            _ => "human".into(),
        }
    }
    fn guard(&self, manifest: &str) -> Guard {
        Guard {
            expect_rev: self.expect_rev,
            expect_manifest: Some(manifest.to_string()),
            overwrite: self.overwrite,
        }
    }
}

fn store(app: &App) -> Res<Store> {
    Ok(Store::open(&app.manifest_paths().store)?)
}

/// The manifest a write targets: `--manifest`, else the current one; main has nothing to change.
fn target(app: &App) -> Res<String> {
    app.manifest_name().ok_or_else(|| {
        CliError::usage("You are on main: changes go into a manifest.")
            .hint("`ownpurse manifest new <name>` creates one and switches to it.")
    })
}

fn open(app: &App) -> Res<Manifest> {
    let name = target(app)?;
    Ok(store(app)?.open_manifest(&name)?)
}

/// The as-of date for effects: `--date`, else the latest cached Xero TB date over the orgs.
fn as_of(app: &mut App, date: Option<String>) -> Res<String> {
    if let Some(d) = date {
        return Ok(d);
    }
    let orgs = app.config.profile.orgs.clone();
    let rec = app.record()?;
    let mut best: Option<String> = None;
    for (_, tid) in &orgs {
        if let Some(d) = rec.latest_tb_date(tid)? {
            best = Some(best.map_or(d.clone(), |b: String| b.max(d)));
        }
    }
    best.ok_or_else(|| {
        CliError::usage("No cached Xero TB to date the effects: pass --date YYYY-MM-DD.")
    })
}

fn ic_config(app: &App) -> Value {
    json!({
        "parties": app.config.settings.get("parties").cloned().unwrap_or(json!({})),
        "intercompany": app.config.settings.get("intercompany").cloned().unwrap_or(json!({})),
    })
}

/// The effects and the check of a manifest.
fn checked(app: &mut App, m: &Manifest, date: Option<String>) -> Res<(effects::Diff, Report)> {
    let as_of = as_of(app, date)?;
    let ctx = app.view_ctx();
    let ic = ic_config(app);
    let zero = ownpurse_core::manifest::zero_accounts(&app.config);
    let line_limit = ownpurse_core::manifest::journal_line_limit(&app.config);
    let rec = app.record()?;
    let d = effects::diff(rec, &ctx, m, &as_of, Some(&ic), &zero)?;
    let r = check::check(
        rec,
        &ctx,
        m,
        &Options {
            as_of,
            ic_config: Some(ic),
            zero_accounts: zero,
            line_limit,
        },
        Some(&d),
    )?;
    Ok((d, r))
}

fn entry_summary(e: &Entry) -> String {
    let n = e.paths().len();
    let what = if e.is_new() {
        "new".to_string()
    } else {
        format!("{n} change{}", if n == 1 { "" } else { "s" })
    };
    format!("{} {} {} · {what}", e.org, short_entity(&e.entity), e.id)
}

fn short_entity(entity: &str) -> &str {
    match entity {
        "ManualJournals" => "journal",
        "BankTransactions" => "bank txn",
        "Invoices" => "invoice",
        "Contacts" => "contact",
        other => other,
    }
}

fn emit_added(app: &App, manifest: &str, seq: i64) -> Res<()> {
    let st = store(app)?;
    let m = st.open_manifest(manifest)?;
    let e = m.entry(seq);
    let mut out = app.out();
    out.field("manifest", manifest)
        .field("entry", seq)
        .field("head_rev", m.head_rev);
    if let Some(e) = e {
        out.field("target", entry_summary(e));
        if let Some(r) = e.replaces {
            out.field("replaces", r);
        }
        if app.g.json {
            out.field("patch", e.patch.clone());
        } else {
            out.field(
                "patch",
                ownpurse_core::manifest::patch_lines(e, None).join("\n"),
            );
        }
    }
    out.hint(format!(
        "`ownpurse manifest check` lists issues; `ownpurse manifest undo {seq}` takes it back."
    ));
    out.emit();
    Ok(())
}

pub fn new(app: &mut App, name: &str, why: &str, f: &WriteFlags) -> Res<()> {
    let mut st = store(app)?;
    let ctx = app.view_ctx();
    let rec = app.record()?;
    let id = ops::create(&mut st, rec, &ctx, &f.actor(), name, why)?;
    app.manifest_paths().set_current(Some(name))?;
    let mut out = app.out();
    out.field("manifest", name)
        .field("created", true)
        .field("current", true)
        .field("head_rev", id);
    out.hint("Add entries: `manifest journal`, `manifest recode`, `manifest void`, `manifest attach`, or `manifest add --after <file>`.");
    out.emit();
    Ok(())
}

pub fn list(app: &mut App) -> Res<()> {
    let st = store(app)?;
    let cur = app.manifest_name();
    let mut rows = vec![obj![
        ("name", "main"),
        ("current", cur.is_none()),
        ("stage", "—"),
        ("entries", 0),
        ("errors", 0),
        ("warnings", 0),
        ("created_by", ""),
    ]];
    for m in st.manifests() {
        let (stage, errors, warnings) = if m.discarded {
            ("discarded".to_string(), 0, 0)
        } else {
            let (_, r) = checked(app, &m, None)?;
            let ctx = app.view_ctx();
            let rec = app.record()?;
            let v = View::open(rec, &ctx, Some(&m))?;
            let s = stage::status(rec, &m, &r, v.overlay());
            (s.stage.word().to_string(), r.errors(), r.unacked())
        };
        rows.push(obj![
            ("name", m.name.as_str()),
            ("current", cur.as_deref() == Some(m.name.as_str())),
            ("stage", stage),
            ("entries", m.active().count()),
            ("errors", errors),
            ("warnings", warnings),
            ("created_by", m.created_by.as_str()),
        ]);
    }
    let mut out = app.out();
    out.table(
        "manifests",
        &[
            "name",
            "current",
            "stage",
            "entries",
            "errors",
            "warnings",
            "created_by",
        ],
        rows,
        None,
    );
    out.emit();
    Ok(())
}

pub fn switch(app: &mut App, name: &str) -> Res<()> {
    if name != "main" {
        store(app)?.open_manifest(name)?;
    }
    app.manifest_paths()
        .set_current(Some(name).filter(|n| *n != "main"))?;
    let mut out = app.out();
    out.field("current", name);
    out.emit();
    Ok(())
}

fn status_word(s: &EntryStatus) -> String {
    match s {
        EntryStatus::None => String::new(),
        EntryStatus::Queued => "queued".into(),
        EntryStatus::Sending => "sending".into(),
        EntryStatus::Landed(t) => format!(
            "landed{} · verifying",
            t.as_ref()
                .map(|t| format!(" · {}", &t[..t.len().min(8)]))
                .unwrap_or_default()
        ),
        EntryStatus::Posted => "posted".into(),
        EntryStatus::Rejected(m) => format!("rejected: {m}"),
        EntryStatus::Uncertain => "uncertain: no response · do not retry (write reconcile)".into(),
        EntryStatus::Manual => "browser step (the checklist)".into(),
    }
}

/// `manifest show [<seq>] [--patch]`: the entries and the stage, or one entry's field diff.
pub fn show(app: &mut App, seq: Option<i64>, raw_patch: bool) -> Res<()> {
    let json_out = app.g.json;
    let m = open(app)?;
    let (_, report) = checked(app, &m, None)?;
    let ctx = app.view_ctx();
    let rec = app.record()?;
    let v = View::open(rec, &ctx, Some(&m))?;
    let ov = v.overlay().expect("a manifest view");
    let st = stage::status(rec, &m, &report, Some(ov));
    if let Some(seq) = seq {
        let e = m.entry(seq).ok_or_else(|| {
            CliError::new("not_found", 2, format!("No entry {seq} on `{}`.", m.name))
        })?;
        let a = ov.applied(seq);
        let mut out = Out::new(json_out);
        out.field("manifest", m.name.as_str())
            .field("entry", seq)
            .field("target", entry_summary(e))
            .field("actor", e.actor.as_str())
            .field("why", e.why.as_str())
            .field(
                "status",
                if !e.active() {
                    "undone".into()
                } else if ov.is_posted(seq) {
                    "posted".into()
                } else if ov.conflict(seq).is_some() {
                    "conflict".to_string()
                } else {
                    "proposed".into()
                },
            );
        if let Some(r) = &e.refs.accountant_decision {
            out.field("accountant_decision", r.as_str());
        }
        if raw_patch {
            if json_out {
                out.field("patch", e.patch.clone());
            } else {
                out.field(
                    "patch",
                    ownpurse_core::manifest::patch_lines(e, a.and_then(|a| a.before.as_ref()))
                        .join("\n"),
                );
            }
        }
        let mut rows = vec![];
        for op in e.ops() {
            let path = op.path().to_string();
            let (kind, new) = match &op {
                json_patch::PatchOperation::Add(x) => ("+", x.value.clone()),
                json_patch::PatchOperation::Remove(_) => ("−", Value::Null),
                json_patch::PatchOperation::Replace(x) => ("~", x.value.clone()),
                _ => ("?", Value::Null),
            };
            let old = a
                .and_then(|a| a.before.as_ref())
                .and_then(|b| b.pointer(&path))
                .cloned()
                .unwrap_or(Value::Null);
            rows.push(obj![
                ("op", kind),
                ("field", check::label(&path)),
                ("path", path.as_str()),
                ("before", short(&old)),
                ("after", short(&new)),
            ]);
        }
        out.table(
            "fields",
            &["op", "field", "path", "before", "after"],
            rows,
            None,
        );
        let issues: Vec<_> = report
            .for_entry(seq)
            .map(|i| {
                obj![
                    ("severity", i.severity.word()),
                    ("code", i.code),
                    ("field", i.field.as_str()),
                    ("message", i.message.as_str()),
                    ("hint", i.hint.as_str()),
                    (
                        "acknowledged",
                        i.ack
                            .as_ref()
                            .map(|a| format!("{} {}", a.actor, a.at.get(..16).unwrap_or(&a.at)))
                            .unwrap_or_default()
                    ),
                ]
            })
            .collect();
        out.table(
            "issues",
            &[
                "severity",
                "code",
                "field",
                "message",
                "hint",
                "acknowledged",
            ],
            issues,
            None,
        );
        out.emit();
        return Ok(());
    }
    let rows: Vec<_> = m
        .entries
        .iter()
        .filter(|e| e.replaced_by.is_none())
        .map(|e| {
            let marker = if !e.active() {
                "undone"
            } else if ov.is_posted(e.seq) {
                ""
            } else {
                ov.applied(e.seq).map_or("≠", |a| match a.kind {
                    "create" => "+",
                    "void" => "⊘",
                    _ => "~",
                })
            };
            obj![
                ("seq", e.seq),
                ("marker", marker),
                ("org", e.org.as_str()),
                ("entity", short_entity(&e.entity)),
                ("target", e.id.as_str()),
                (
                    "changes",
                    if e.is_new() {
                        "new".to_string()
                    } else {
                        format!("{}", e.paths().len())
                    }
                ),
                (
                    "issues",
                    if e.active() {
                        report.entry_badge(e.seq)
                    } else {
                        String::new()
                    }
                ),
                ("actor", if e.by_agent() { "A" } else { "H" }),
                ("apply", status_word(st.of(e.seq))),
            ]
        })
        .collect();
    let (short_form, _) = st
        .stage
        .short(m.active().count(), report.errors() + report.unacked());
    let mut out = Out::new(json_out);
    out.field("manifest", m.name.as_str())
        .field("stage", st.stage.word())
        .field("chip", short_form)
        .field("head_rev", m.head_rev)
        .field("issues", report.summary());
    if st.plan_stale {
        out.field(
            "plan",
            "stale: the manifest changed after planning; plan again",
        );
    }
    if let Some(p) = m.plan() {
        let c = compile::commands(&m.name, &p.hash);
        out.field("package", p.package.as_str())
            .field("hash", p.hash.as_str());
        match st.stage {
            stage::Stage::Planned => {
                out.field("approve", c.approve.as_str())
                    .field("then_apply", c.apply.as_str());
            }
            stage::Stage::Approved => {
                out.field("apply", c.apply.as_str());
            }
            _ => {
                let a = &st.apply;
                out.field(
                    "applying",
                    format!(
                        "{} of {} · {} verified · {} landed · {} rejected · {} uncertain",
                        a.sent, a.total, a.verified, a.landed, a.rejected, a.uncertain
                    ),
                );
            }
        }
    }
    out.table(
        "entries",
        &[
            "seq", "marker", "org", "entity", "target", "changes", "issues", "actor", "apply",
        ],
        rows,
        None,
    );
    out.emit();
    Ok(())
}

fn short(v: &Value) -> String {
    match v {
        Value::Null => "—".into(),
        Value::String(s) => s.clone(),
        o => {
            let t = o.to_string();
            if t.chars().count() > 60 {
                format!("{}…", t.chars().take(59).collect::<String>())
            } else {
                t
            }
        }
    }
}

pub fn log(app: &mut App) -> Res<()> {
    let name = target(app)?;
    let st = store(app)?;
    let m = st
        .manifest(&name)
        .ok_or_else(|| CliError::new("not_found", 2, format!("No manifest named `{name}`.")))?;
    let rows: Vec<_> = st
        .events()
        .iter()
        .filter(|e| e.manifest == name)
        .map(|e| {
            let summary = match e.kind {
                Kind::EntryAdd => m.entry(e.id).map(entry_summary).unwrap_or_default(),
                Kind::EntryUndo => format!(
                    "{} entry {}",
                    if e.payload["undone"] == true {
                        "undo"
                    } else {
                        "restore"
                    },
                    e.payload["seq"]
                ),
                Kind::EntryAnnotate => format!("annotate entry {}", e.payload["seq"]),
                Kind::IssueAck => format!(
                    "{} {} on entry {}",
                    if e.payload["acked"] == false {
                        "withdraw ack of"
                    } else {
                        "acknowledge"
                    },
                    e.payload["code"].as_str().unwrap_or(""),
                    e.payload["seq"]
                ),
                Kind::Planned => format!(
                    "planned · sha256 {}",
                    e.payload["hash"].as_str().unwrap_or("")
                ),
                Kind::Create => format!("created · {}", e.payload["why"].as_str().unwrap_or("")),
                Kind::Discard => "discarded".into(),
                Kind::Resolve => format!(
                    "resolve {} → {}",
                    e.payload["seq"]
                        .as_i64()
                        .map_or("default".to_string(), |s| format!("entry {s}")),
                    e.payload["resolution"].as_str().unwrap_or("")
                ),
            };
            obj![
                ("seq", e.id),
                ("at", e.at.get(..19).unwrap_or(&e.at)),
                ("actor", e.actor.as_str()),
                ("kind", e.kind.as_str()),
                ("summary", summary),
            ]
        })
        .collect();
    let mut out = app.out();
    out.field("manifest", name.as_str());
    out.table(
        "log",
        &["seq", "at", "actor", "kind", "summary"],
        rows,
        None,
    );
    out.emit();
    Ok(())
}

/// `manifest diff`: the effects against main (TB, statements, bridges, must-be-zero accounts).
pub fn diff(app: &mut App, date: Option<String>, csv: bool) -> Res<()> {
    let m = open(app)?;
    let (d, _) = checked(app, &m, date)?;
    let mut tb = vec![];
    let mut totals = vec![];
    let mut zero_rows = vec![];
    for o in &d.orgs {
        for a in &o.accounts {
            tb.push(obj![
                ("org", o.org.as_str()),
                ("code", a.code.as_str()),
                ("account", a.name.as_str()),
                ("main", dec(a.main)),
                ("manifest", dec(a.manifest)),
                ("delta", dec(a.delta())),
            ]);
        }
        for (label, mm, mf) in &o.totals {
            if mm != mf {
                totals.push(obj![
                    ("org", o.org.as_str()),
                    ("line", *label),
                    ("main", dec(*mm)),
                    ("manifest", dec(*mf)),
                    ("delta", dec(*mf - *mm)),
                ]);
            }
        }
        for a in &o.zero {
            zero_rows.push(obj![
                ("org", o.org.as_str()),
                ("code", a.code.as_str()),
                ("account", a.name.as_str()),
                ("main", dec(a.main)),
                ("manifest", dec(a.manifest)),
                ("zero", a.manifest.is_zero()),
            ]);
        }
    }
    let bridges: Vec<_> = d
        .bridges
        .iter()
        .filter(|x| x.main != x.manifest)
        .map(|x| {
            obj![
                ("pair", x.pair.as_str()),
                ("label", x.label.as_str()),
                ("main", x.main.map(dec).unwrap_or(Value::Null)),
                ("manifest", x.manifest.map(dec).unwrap_or(Value::Null)),
                ("zero", x.manifest.is_some_and(|v| v.is_zero())),
            ]
        })
        .collect();
    if csv {
        println!("section,org,code,account,main,manifest,delta");
        let s = |v: &Value| match v {
            Value::String(s) => s.clone(),
            Value::Null => String::new(),
            o => o.to_string(),
        };
        let q = |t: &str| format!("\"{}\"", t.replace('"', "\"\""));
        for r in &tb {
            println!(
                "tb,{},{},{},{},{},{}",
                s(&r["org"]),
                s(&r["code"]),
                q(&s(&r["account"])),
                s(&r["main"]),
                s(&r["manifest"]),
                s(&r["delta"])
            );
        }
        for r in &totals {
            println!(
                "statement,{},,{},{},{},{}",
                s(&r["org"]),
                s(&r["line"]),
                s(&r["main"]),
                s(&r["manifest"]),
                s(&r["delta"])
            );
        }
        for r in &zero_rows {
            println!(
                "must_be_zero,{},{},{},{},{},",
                s(&r["org"]),
                s(&r["code"]),
                q(&s(&r["account"])),
                s(&r["main"]),
                s(&r["manifest"])
            );
        }
        for r in &bridges {
            println!(
                "ic_bridge,,{},{},{},{},",
                s(&r["pair"]),
                q(&s(&r["label"])),
                s(&r["main"]),
                s(&r["manifest"])
            );
        }
        return Ok(());
    }
    let f = &d.facts;
    let mut out = app.out();
    out.field("manifest", d.manifest.as_str())
        .field("as_of", d.as_of.as_str())
        .field("entries", d.entries)
        .field("conflicts", d.conflicts)
        .field(
            "plan_facts",
            format!(
                "{} journals would post (reversed only by a reversing journal) · {} voids · {} recodes ({} on reconciled lines) · {} attachments · {} not writable",
                f.journals, f.voids, f.recodes, f.reconciled_line_edits, f.attachments, f.not_writable
            ),
        );
    out.table(
        "trial_balance",
        &["org", "code", "account", "main", "manifest", "delta"],
        tb,
        None,
    );
    out.table(
        "statements",
        &["org", "line", "main", "manifest", "delta"],
        totals,
        None,
    );
    out.table(
        "intercompany",
        &["pair", "label", "main", "manifest", "zero"],
        bridges,
        None,
    );
    out.table(
        "must_be_zero",
        &["org", "code", "account", "main", "manifest", "zero"],
        zero_rows,
        None,
    );
    out.hint("Nothing here is in Xero: `manifest check`, then `manifest plan` seals a package for approval.");
    out.emit();
    Ok(())
}

/// `manifest check`: every issue (errors block plan; warnings need acknowledging).
pub fn check_cmd(app: &mut App, date: Option<String>) -> Res<()> {
    let m = open(app)?;
    let (_, r) = checked(app, &m, date)?;
    let rows: Vec<_> = r
        .issues
        .iter()
        .map(|i| {
            obj![
                ("severity", i.severity.glyph()),
                ("entry", i.seq),
                ("org", i.org.as_str()),
                ("code", i.code),
                ("field", i.field.as_str()),
                ("path", i.path.as_str()),
                ("message", i.message.as_str()),
                ("hint", i.hint.as_str()),
                (
                    "acknowledged",
                    i.ack.as_ref().map(|a| a.actor.clone()).unwrap_or_default()
                ),
            ]
        })
        .collect();
    let mut out = app.out();
    out.field("manifest", m.name.as_str())
        .field("errors", r.errors())
        .field("warnings", r.warnings())
        .field("acknowledged", r.acknowledged())
        .field("summary", r.summary());
    out.table(
        "issues",
        &[
            "severity",
            "entry",
            "org",
            "code",
            "field",
            "path",
            "message",
            "hint",
            "acknowledged",
        ],
        rows,
        None,
    );
    if r.unacked() > 0 {
        out.hint("Acknowledge a warning: `ownpurse manifest ack <entry> <code> --path <path>`.");
    }
    out.emit();
    Ok(())
}

/// `manifest ack <entry> <code> [--path P] [--withdraw]`.
pub fn ack(
    app: &mut App,
    seq: i64,
    code: &str,
    path: Option<String>,
    withdraw: bool,
    f: &WriteFlags,
) -> Res<()> {
    let m = open(app)?;
    let path = match path {
        Some(p) => p,
        None => {
            let (_, r) = checked(app, &m, None)?;
            let hits: Vec<_> = r
                .for_entry(seq)
                .filter(|i| i.code == code && !i.blocking())
                .collect();
            match hits.as_slice() {
                [one] => one.path.clone(),
                [] => {
                    return Err(CliError::usage(format!(
                        "No `{code}` warning on entry {seq}."
                    )));
                }
                _ => {
                    return Err(CliError::usage(format!(
                        "Entry {seq} has several `{code}` warnings: pass --path ({}).",
                        hits.iter()
                            .map(|i| i.path.as_str())
                            .collect::<Vec<_>>()
                            .join(", ")
                    )));
                }
            }
        }
    };
    let mut st = store(app)?;
    let id = ops::ack(
        &mut st,
        &m.name,
        &f.actor(),
        seq,
        code,
        &path,
        !withdraw,
        &f.guard(&m.name),
    )?;
    let mut out = app.out();
    out.field("manifest", m.name.as_str())
        .field(
            if withdraw {
                "withdrawn"
            } else {
                "acknowledged"
            },
            format!("{code} on entry {seq} ({path})"),
        )
        .field("head_rev", id);
    out.emit();
    Ok(())
}

/// `manifest plan`: compile the checked manifest into the write package, seal it, write the review
/// export, and print the approve / apply commands.
pub fn plan(app: &mut App, date: Option<String>, f: &WriteFlags) -> Res<()> {
    let m = open(app)?;
    let (_, report) = checked(app, &m, date)?;
    let ctx = app.view_ctx();
    let cfg = ownpurse_core::write::plan::PlanConfig {
        profile: app.config.profile.name.clone(),
        orgs: app.config.profile.orgs.clone(),
        gates: ownpurse_core::manifest::accountant_gate(&app.config),
        history_notes: app.config.profile.history_notes(),
    };
    let now = ownpurse_core::classify::store::now_iso();
    let rec = app.record()?;
    let pkg = compile::compile(rec, &ctx, &cfg, &m, &report, &now)?;
    let dir = app.manifest_paths().packages;
    std::fs::create_dir_all(&dir)
        .map_err(|e| CliError::runtime("io", format!("{}: {e}", dir.display())))?;
    let path = dir.join(format!("{}.package.json", pkg.id));
    ownpurse_core::write::seal::save(&path, &pkg)?;
    let (pkg, hash, pdf, csv) = crate::write::seal_file(app, &path, None)?;
    let mut st = store(app)?;
    ops::planned(
        &mut st,
        &m.name,
        &f.actor(),
        &path.display().to_string(),
        &hash,
        m.content_rev,
    )?;
    let c = compile::commands(&m.name, &hash);
    let mut out = app.out();
    out.field("manifest", m.name.as_str())
        .field("package", path.display().to_string())
        .field("hash", hash.as_str())
        .field("ops", pkg.ops.len())
        .field("pdf", pdf.display().to_string())
        .field("csv", csv.display().to_string())
        .field("approve", c.approve.as_str())
        .field("then_apply", c.apply.as_str());
    out.hint("Nothing was sent to Xero. The owner approves at his own terminal (the approve line), then the apply line runs it.");
    out.emit();
    Ok(())
}

/// The planned package of a manifest, checked against the hash the caller quotes.
fn planned_package(app: &App, name: &str, hash: &str) -> Res<PathBuf> {
    let m = store(app)?.open_manifest(name)?;
    let p = m.plan().ok_or_else(|| {
        CliError::new(
            "not_approved",
            3,
            format!("`{name}` has no current plan (ownpurse manifest plan)."),
        )
    })?;
    if p.hash != hash {
        return Err(CliError::new(
            "hash_mismatch",
            3,
            format!(
                "`{name}`'s current plan is {}…, not {}…",
                &p.hash[..12.min(p.hash.len())],
                &hash[..12.min(hash.len())]
            ),
        ));
    }
    Ok(PathBuf::from(&p.package))
}

/// `manifest approve <name> --package <hash> --response-file F --i-am-the-owner` = `write approve-record`.
pub fn approve(
    app: &mut App,
    name: &str,
    hash: &str,
    response: &Path,
    i_am_owner: bool,
) -> Res<()> {
    let pkg = planned_package(app, name, hash)?;
    crate::write::approve_record(app, &pkg, response, i_am_owner, None)
}

/// `manifest apply <name> --package <hash>` = `write run` (the same locks: OWNPURSE_WRITE_RUN=<hash>).
pub fn apply(
    app: &mut App,
    name: &str,
    hash: &str,
    batch: Option<usize>,
    dry_run: bool,
) -> Res<()> {
    let pkg = planned_package(app, name, hash)?;
    crate::write_run::run_cmd(app, &pkg, batch, dry_run)
}

pub fn undo(app: &mut App, seq: Option<i64>, restore: bool, f: &WriteFlags) -> Res<()> {
    let name = target(app)?;
    let mut st = store(app)?;
    let m = st.open_manifest(&name)?;
    let seq = match seq {
        Some(s) => s,
        None => m
            .entries
            .iter()
            .rev()
            .find(|c| if restore { c.undone } else { c.active() })
            .map(|c| c.seq)
            .ok_or_else(|| CliError::usage("No entry to undo in this manifest."))?,
    };
    let id = ops::undo(&mut st, &name, &f.actor(), seq, !restore, &f.guard(&name))?;
    let mut out = app.out();
    out.field("manifest", name.as_str())
        .field(if restore { "restored" } else { "undid" }, seq)
        .field("head_rev", id);
    out.emit();
    Ok(())
}

pub fn discard(app: &mut App, name: Option<String>, f: &WriteFlags) -> Res<()> {
    let name = match name {
        Some(n) => n,
        None => target(app)?,
    };
    let mut st = store(app)?;
    ops::discard(&mut st, &name, &f.actor(), &f.guard(&name))?;
    if app.manifest_paths().current().as_deref() == Some(name.as_str()) {
        app.manifest_paths().set_current(None)?;
    }
    let mut out = app.out();
    out.field("discarded", name.as_str())
        .field("current", "main");
    out.hint("Its history stays: `ownpurse manifest log --manifest <name>`.");
    out.emit();
    Ok(())
}

pub fn annotate(
    app: &mut App,
    seq: i64,
    accountant: Option<String>,
    note: Option<String>,
    f: &WriteFlags,
) -> Res<()> {
    let name = target(app)?;
    let mut st = store(app)?;
    let id = ops::annotate(
        &mut st,
        &name,
        &f.actor(),
        seq,
        accountant.as_deref(),
        note.as_deref(),
        &f.guard(&name),
    )?;
    let mut out = app.out();
    out.field("manifest", name.as_str())
        .field("annotated", seq)
        .field("head_rev", id);
    out.emit();
    Ok(())
}

// ── producers ──────────────────────────────────────────────────────────────────────────────────

/// Add (or combine onto the document's entry) the entry whose "after" is `after`.
fn put(
    app: &mut App,
    t: Target,
    after: Value,
    why: &str,
    accountant: Option<String>,
    replaces: Option<i64>,
    f: &WriteFlags,
) -> Res<()> {
    let name = target(app)?;
    let mut st = store(app)?;
    let ctx = app.view_ctx();
    let refs = Refs {
        accountant_decision: accountant,
        ..Default::default()
    };
    let rec = app.record()?;
    let seq = ops::add(
        &mut st,
        rec,
        &ctx,
        &name,
        &f.actor(),
        &t,
        &after,
        why,
        &refs,
        replaces,
        &f.guard(&name),
    )?;
    emit_added(app, &name, seq)
}

/// The document as the manifest sees it now (normalised).
fn current(app: &mut App, t: &Target) -> Res<Value> {
    let m = open(app)?;
    let ctx = app.view_ctx();
    let rec = app.record()?;
    let v = View::open(rec, &ctx, Some(&m))?;
    ops::current(&v, &ctx, t)?.ok_or_else(|| {
        CliError::new(
            "not_found",
            2,
            format!(
                "No {} document {} in {}'s record.",
                t.entity,
                t.id.as_deref().unwrap_or(""),
                t.org
            ),
        )
    })
}

fn entity_code(code: &str) -> Res<&'static str> {
    ownpurse_core::view::entity_of(code)
        .or(match code {
            "ManualJournals" => Some("ManualJournals"),
            "BankTransactions" => Some("BankTransactions"),
            _ => None,
        })
        .ok_or_else(|| CliError::usage(format!("`{code}`: use mj, bt, inv or bill")))
}

/// `org:type:id` (type mj, bt, inv, bill).
fn doc_ref(s: &str) -> Res<Target> {
    let parts: Vec<&str> = s.split(':').collect();
    let [org, typ, id] = parts.as_slice() else {
        return Err(CliError::usage(format!(
            "`{s}`: expected org:type:id (type mj, bt, inv or bill)"
        )));
    };
    Ok(Target {
        entity: entity_code(typ)?.to_string(),
        org: (*org).into(),
        id: Some((*id).into()),
    })
}

/// `manifest add --entity E --org O [--id ID] (--after FILE | --patch FILE)`: any document, generically.
#[allow(clippy::too_many_arguments)]
pub fn add(
    app: &mut App,
    entity: &str,
    org: &str,
    id: Option<String>,
    after: Option<String>,
    patch: Option<String>,
    why: &str,
    accountant: Option<String>,
    f: &WriteFlags,
) -> Res<()> {
    let t = Target {
        entity: entity.into(),
        org: org.into(),
        id,
    };
    let read = |p: &str| -> Res<Value> {
        let text = std::fs::read_to_string(p).map_err(|e| CliError::usage(format!("{p}: {e}")))?;
        serde_json::from_str(&text).map_err(|e| CliError::usage(format!("{p}: {e}")))
    };
    let after = match (after, patch) {
        (Some(a), None) => read(&a)?,
        (None, Some(p)) => {
            let ops_v = read(&p)?;
            let base = if t.id.is_some() {
                current(app, &t)?
            } else {
                json!({})
            };
            let mut doc = base;
            let ops: json_patch::Patch = serde_json::from_value(ops_v)
                .map_err(|e| CliError::usage(format!("{p}: not an RFC 6902 patch ({e})")))?;
            json_patch::patch(&mut doc, &ops.0)
                .map_err(|e| CliError::usage(format!("{p}: the patch doesn't apply ({e})")))?;
            doc
        }
        _ => {
            return Err(CliError::usage(
                "pass exactly one of --after <file> or --patch <file>",
            ));
        }
    };
    put(app, t, after, why, accountant, None, f)
}

/// `--line CODE=AMOUNT[:description]` (debit positive, credit negative; or `Dr`/`Cr` prefixes).
fn parse_line(s: &str) -> Res<JLine> {
    let bad = || {
        CliError::usage(format!(
            "--line `{s}`: expected CODE=AMOUNT[:description] (debit +, credit −, e.g. 6290=1200.00 or 2430=-1200.00)"
        ))
    };
    let (code, rest) = s.split_once('=').ok_or_else(bad)?;
    let (amt, desc) = rest.split_once(':').unwrap_or((rest, ""));
    let amt = amt.trim();
    let (sign, num) = if let Some(n) = amt.strip_prefix("Cr ").or_else(|| amt.strip_prefix("cr ")) {
        (-1, n)
    } else if let Some(n) = amt.strip_prefix("Dr ").or_else(|| amt.strip_prefix("dr ")) {
        (1, n)
    } else {
        (1, amt)
    };
    let amount = Decimal::from_str(num.trim()).map_err(|_| bad())? * Decimal::from(sign);
    Ok(JLine {
        account: code.trim().to_string(),
        description: desc.trim().to_string(),
        amount,
    })
}

#[derive(Debug, Clone, Default)]
pub struct JournalFlags {
    pub org: Option<String>,
    pub date: Option<String>,
    pub narration: Option<String>,
    pub cash_basis: Option<String>,
    pub lines: Vec<String>,
    pub file: Option<String>,
    pub why: Option<String>,
    pub accountant: Option<String>,
}

impl JournalFlags {
    /// `(org, normalised journal)`.
    fn build(&self) -> Res<(String, Value)> {
        let mut org = self.org.clone();
        let mut date = self.date.clone();
        let mut narration = self.narration.clone();
        let mut cash = self.cash_basis.clone();
        let mut lines: Vec<JLine> = vec![];
        if let Some(path) = &self.file {
            let text = std::fs::read_to_string(path)
                .map_err(|e| CliError::usage(format!("{path}: {e}")))?;
            if path.ends_with(".json") {
                let v: Value = serde_json::from_str(&text)
                    .map_err(|e| CliError::usage(format!("{path}: {e}")))?;
                org = org.or(v["org"].as_str().map(str::to_string));
                date = date.or(v["date"].as_str().map(str::to_string));
                narration = narration.or(v["narration"].as_str().map(str::to_string));
                cash = cash.or(v["cash_basis"]
                    .as_bool()
                    .map(|b| if b { "yes".into() } else { "no".into() }));
                for l in v["lines"].as_array().into_iter().flatten() {
                    let amt = match &l["amount"] {
                        Value::String(s) => s.clone(),
                        o => o.to_string(),
                    };
                    lines.push(JLine {
                        account: l["account"].as_str().unwrap_or("").into(),
                        description: l["description"].as_str().unwrap_or("").into(),
                        amount: Decimal::from_str(&amt)
                            .map_err(|_| CliError::usage(format!("{path}: amount {amt:?}")))?,
                    });
                }
            } else {
                for (i, row) in text.lines().enumerate() {
                    let cols: Vec<&str> = row.split(',').map(str::trim).collect();
                    if i == 0
                        && cols
                            .first()
                            .is_some_and(|c| c.eq_ignore_ascii_case("account"))
                    {
                        continue;
                    }
                    if cols.len() < 3 || row.trim().is_empty() {
                        continue;
                    }
                    lines.push(parse_line(&format!("{}={}:{}", cols[0], cols[2], cols[1]))?);
                }
            }
        }
        for l in &self.lines {
            lines.push(parse_line(l)?);
        }
        let cash = match cash.as_deref() {
            Some("yes" | "true") => true,
            Some("no" | "false") => false,
            _ => {
                return Err(CliError::usage(
                    "--cash-basis yes|no is required (Xero's ShowOnCashBasisReports; never left to the default).",
                ));
            }
        };
        let org = org.ok_or_else(|| CliError::usage("--org is required"))?;
        let date = date.ok_or_else(|| CliError::usage("--date YYYY-MM-DD is required"))?;
        Ok((
            org,
            ops::journal_doc(&date, &narration.unwrap_or_default(), cash, &lines),
        ))
    }
}

/// `manifest journal …`: a new manual journal.
pub fn journal(app: &mut App, j: &JournalFlags, f: &WriteFlags) -> Res<()> {
    let (org, doc) = j.build()?;
    let t = Target {
        entity: "ManualJournals".into(),
        org,
        id: None,
    };
    put(
        app,
        t,
        doc,
        j.why.as_deref().unwrap_or(""),
        j.accountant.clone(),
        None,
        f,
    )
}

/// `manifest edit <seq> …`: replace a new-journal entry with an edited one.
pub fn edit(app: &mut App, seq: i64, j: &JournalFlags, f: &WriteFlags) -> Res<()> {
    let m = open(app)?;
    let e = m
        .entry(seq)
        .ok_or_else(|| CliError::new("not_found", 2, format!("No entry {seq}.")))?;
    if !e.is_new() {
        return Err(CliError::usage(
            "`manifest edit` edits new journals; change other documents with recode/void/add (they combine).",
        ));
    }
    let (org, doc) = j.build()?;
    let t = Target {
        entity: "ManualJournals".into(),
        org,
        id: Some(e.id.clone()),
    };
    put(
        app,
        t,
        doc,
        j.why.as_deref().unwrap_or(""),
        j.accountant.clone(),
        Some(seq),
        f,
    )
}

/// `manifest recode org:bt:id#LINE --account C [--description D]`.
pub fn recode(
    app: &mut App,
    line: &str,
    account: Option<String>,
    description: Option<String>,
    why: &str,
    f: &WriteFlags,
) -> Res<()> {
    let (doc, l) = line.split_once('#').ok_or_else(|| {
        CliError::usage(format!(
            "`{line}`: expected org:bt:id#LINE (LINE: a line id, n<k> or L<k>)"
        ))
    })?;
    let t = doc_ref(doc)?;
    let d = current(app, &t)?;
    let after = ops::recode_doc(&d, l, account.as_deref(), description.as_deref())?;
    put(app, t, after, why, None, None, f)
}

/// `manifest void org:mj:id`.
pub fn void(app: &mut App, doc: &str, why: &str, f: &WriteFlags) -> Res<()> {
    let t = doc_ref(doc)?;
    let d = current(app, &t)?;
    put(app, t, ops::void_doc(&d), why, None, None, f)
}

/// `manifest attach org:mj:id <file>`.
pub fn attach(app: &mut App, doc: &str, file: &Path, why: &str, f: &WriteFlags) -> Res<()> {
    let t = doc_ref(doc)?;
    let d = current(app, &t)?;
    let file = std::fs::canonicalize(file)
        .map_err(|e| CliError::usage(format!("{}: {e}", file.display())))?;
    let after = ops::attach_doc(&d, &file)?;
    put(app, t, after, why, None, None, f)
}

/// `manifest note <doc> "<text>"`: a note added to the document's history (the rules table's `add_note`).
pub fn note(app: &mut App, doc: &str, text: &str, why: &str, f: &WriteFlags) -> Res<()> {
    let t = doc_ref(doc)?;
    let d = current(app, &t)?;
    let text = text.trim();
    if text.is_empty() || text.chars().count() > ownpurse_core::write::NOTE_MAX {
        return Err(CliError::usage(format!(
            "A note has 1 to {} characters ({} given).",
            ownpurse_core::write::NOTE_MAX,
            text.chars().count()
        )));
    }
    let mut after = d.clone();
    let mut h = after
        .get("History")
        .and_then(serde_json::Value::as_array)
        .cloned()
        .unwrap_or_default();
    h.push(serde_json::json!({ "Details": text }));
    after["History"] = serde_json::Value::Array(h);
    put(app, t, after, why, None, None, f)
}

/// One line for the read commands: which view they show.
pub fn view_note(out: &mut Out, app: &App) {
    if let Some(m) = app.manifest_name() {
        out.field("view", format!("⎇ {m} (main + manifest; not in Xero)"));
    }
}

/// `manifest resolve <entry|default> reclass|browser`.
pub fn resolve(app: &mut App, entry: &str, how: &str, f: &WriteFlags) -> Res<()> {
    let name = target(app)?;
    let seq = if entry == "default" {
        None
    } else {
        Some(
            entry
                .parse::<i64>()
                .map_err(|_| CliError::usage(format!("`{entry}`: an entry number or `default`")))?,
        )
    };
    let mut st = store(app)?;
    let id = ops::resolve(&mut st, &name, &f.actor(), seq, how, &f.guard(&name))?;
    let mut out = app.out();
    out.field("manifest", name.as_str())
        .field(
            "resolved",
            seq.map_or("default".to_string(), |s| format!("entry {s}")),
        )
        .field("how", how)
        .field("head_rev", id);
    out.emit();
    Ok(())
}
