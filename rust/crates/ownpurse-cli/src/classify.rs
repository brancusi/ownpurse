//! `ownpurse classify …`: the classification layer from the command line. Reads the record (read-only) and
//! writes only the classification store (`db/classify.sqlite`) and config (`classify.toml`); nothing here calls
//! Xero (V14, V18). Jev is not enabled in this release: `--engine jev` refuses with `jev_not_enabled`.

use crate::app::{App, CliError, Res};
use crate::obj;
use crate::out::{dec, opt_dec};
use ownpurse_core::classify::config::{ConfigFile, plural};
use ownpurse_core::classify::request::ClassifyRequest;
use ownpurse_core::classify::view::{Cell, ClassifyPaths, Filter, QueueRow, Sort, Workspace};
use ownpurse_core::classify::{Actor, Anchor, BatchId, ClassifyError, WriteCtx};
use serde_json::{Map, Value};

impl From<ClassifyError> for CliError {
    fn from(e: ClassifyError) -> Self {
        let mut c = CliError::new(e.code, e.exit_code(), e.message);
        c.hints = e.hints;
        if c.hints.is_empty() {
            // every refusal says what to do next (the table in core::bridge)
            c.hints
                .push(ownpurse_core::bridge::default_help(e.code).to_string());
        }
        c
    }
}

fn filter_of(words: &[String]) -> Res<Filter> {
    Ok(Filter::parse(&words.join(" "))?)
}

fn open(app: &mut App) -> Res<Workspace> {
    let paths = ClassifyPaths::resolve(&app.config);
    let orgs = app.config.profile.orgs.clone();
    let v = app.view()?;
    Workspace::open(&paths, &v, &orgs).map_err(|e| {
        let mut c = CliError::from(e);
        if c.code == "config" {
            c = c.hint(format!(
                "The classification config is {} (`classify.config` in ownpurse.json); check it with `ownpurse classify rules check`.",
                paths.config.display()
            ));
        }
        c
    })
}

/// A batch id for one CLI call: the store clock plus the process id.
fn batch() -> BatchId {
    BatchId::new(format!(
        "cli-{}-{}",
        ownpurse_core::classify::store::now_iso(),
        std::process::id()
    ))
}

/// The refusal for `--engine jev` (V16): Jev is not enabled in this release.
pub fn jev_not_enabled() -> CliError {
    CliError::new(
        "jev_not_enabled",
        2,
        "Jev is not enabled: nothing was sent.",
    )
    .hint("This release runs rules only: `ownpurse classify suggest --engine rules`.")
}

pub fn suggest(app: &mut App, words: &[String], engine: &str, dry_run: bool) -> Res<()> {
    if engine == "jev" {
        return Err(jev_not_enabled());
    }
    let filter = filter_of(words)?;
    let mut ws = open(app)?;
    let (sum, rev) = if dry_run {
        (ws.suggest_dry_run(&filter), ws.rev())
    } else {
        let w = WriteCtx::new(Actor::rule("engine"), batch());
        let (sum, out) = ws.suggest(&filter, &w)?;
        (sum, out.rev)
    };
    let mut o = app.out();
    o.field("engine", "rules")
        .field("filter", filter.to_text())
        .field(
            "scope",
            ws.config()
                .queue
                .scope
                .clone()
                .map_or(Value::Null, Value::from),
        )
        .field("dry_run", dry_run)
        .field("lines", sum.lines)
        .field(
            if dry_run {
                "would_suggest"
            } else {
                "suggested"
            },
            sum.suggested,
        )
        .field(if dry_run { "would_write" } else { "events" }, sum.events)
        .field("store_rev", rev);
    if dry_run {
        let types: Vec<String> = ws
            .config()
            .label_types
            .iter()
            .map(|t| t.id.clone())
            .collect();
        let mut cols = vec!["rule".to_string(), "matched".to_string()];
        cols.extend(types.iter().cloned());
        let rows = sum
            .rules
            .iter()
            .map(|r| {
                let mut m = obj![("rule", r.rule.as_str()), ("matched", r.lines)];
                for t in &types {
                    let n = r.labels.iter().find(|(k, _)| k == t).map_or(0, |(_, n)| *n);
                    m.insert(t.clone(), Value::from(n));
                }
                m
            })
            .collect();
        o.table_cols("rules", cols, rows, None);
        o.hint("Nothing was written. Run without --dry-run to store these suggestions.");
        o.emit();
        return Ok(());
    }
    let rows: Vec<Map<String, Value>> = sum
        .by_rule
        .iter()
        .map(|(r, n)| obj![("rule", r.as_str()), ("lines", *n)])
        .collect();
    o.table("by_rule", &["rule", "lines"], rows, None);
    o.hint("Review them: `ownpurse classify queue` (or the TUI's Classify tab).");
    o.emit();
    Ok(())
}

pub fn rules_check(app: &mut App) -> Res<()> {
    let paths = ClassifyPaths::resolve(&app.config);
    let aliases: Vec<String> = app
        .config
        .profile
        .orgs
        .iter()
        .map(|(a, _)| a.clone())
        .collect();
    let loaded = ConfigFile::load(&paths.config).and_then(|f| {
        let issues = ownpurse_core::classify::config::check_orgs(&f.text, &f.name(), &aliases);
        if issues.is_empty() {
            Ok(f)
        } else {
            Err(issues)
        }
    });
    match loaded {
        Ok(f) => {
            let c = &f.config;
            let mut o = app.out();
            o.field("config", paths.config.display().to_string())
                .field("ok", true)
                .field("label_types", c.label_types.len())
                .field(
                    "choices",
                    c.label_types.iter().map(|t| t.choices.len()).sum::<usize>(),
                )
                .field("rules", c.rules.len())
                .field("required", Value::from(c.queue.required.clone()))
                .field(
                    "jev",
                    if c.jev.enabled {
                        "enabled"
                    } else {
                        "not enabled"
                    },
                );
            o.emit();
            Ok(())
        }
        Err(issues) => {
            let n = issues.len();
            let mut e = CliError::runtime(
                "config",
                format!(
                    "{} has {n} {}.",
                    paths.config.display(),
                    plural(n, "problem")
                ),
            );
            for i in &issues {
                e = e.hint(i.to_string());
            }
            Err(e)
        }
    }
}

fn cell_text(c: &Option<Cell>) -> String {
    c.as_ref().map_or("·".to_string(), |c| c.value.clone())
}

/// `project / bearer / treatment`, `·` for a missing cell, `· none` when nothing is suggested at all.
fn triple_text(r: &QueueRow) -> String {
    let s = &r.suggestion;
    if s.project.is_none() && s.bearer.is_none() && s.treatment.is_none() {
        return "· none".into();
    }
    format!(
        "{} / {} / {}",
        cell_text(&s.project),
        cell_text(&s.bearer),
        cell_text(&s.treatment)
    )
}

fn cell_json(c: &Option<Cell>) -> Value {
    match c {
        None => Value::Null,
        Some(c) => Value::Object(obj![
            ("value", c.value.as_str()),
            ("name", c.name.as_str()),
            ("source", c.source.to_string()),
            ("confidence", opt_dec(c.confidence)),
            ("confirmed", c.confirmed),
        ]),
    }
}

fn row_json(r: &QueueRow) -> Map<String, Value> {
    obj![
        ("anchor", r.anchor.to_string()),
        ("org", r.org.as_str()),
        ("date", r.date.as_str()),
        ("payee", r.payee.as_str()),
        ("description", r.description.as_str()),
        ("amount", dec(r.amount)),
        ("account", r.account.as_str()),
        (
            "suggestion",
            Value::Object(obj![
                ("project", cell_json(&r.suggestion.project)),
                ("bearer", cell_json(&r.suggestion.bearer)),
                ("treatment", cell_json(&r.suggestion.treatment)),
            ])
        ),
        ("source", r.glyph.char().to_string()),
        ("confidence", opt_dec(r.confidence)),
        ("stale", r.stale),
        ("open", r.open),
    ]
}

fn row_text(r: &QueueRow) -> Map<String, Value> {
    obj![
        ("anchor", r.anchor.to_string()),
        ("date", r.date.as_str()),
        ("payee", r.payee.as_str()),
        ("amount", ownpurse_core::money::fmt(r.amount)),
        ("account", r.account.as_str()),
        ("suggestion", triple_text(r)),
        ("src", r.glyph.char().to_string()),
        (
            "conf",
            r.confidence.map(|c| c.to_string()).unwrap_or_default()
        ),
        ("stale", if r.stale { "stale" } else { "" }),
    ]
}

pub fn queue(app: &mut App, words: &[String], sort: Option<&str>) -> Res<()> {
    let filter = filter_of(words)?;
    let sort = match sort {
        Some("date") => Sort::DateAsc,
        Some("amount") => Sort::AmountDesc,
        _ => Sort::StaleFirst,
    };
    let ws = open(app)?;
    let rows = ws.queue(&filter, sort);
    let here = ws.counts_for(&filter);
    let all = ws.counts();
    let shown = app.limit(&rows);
    let mut o = app.out();
    o.field("filter", filter.to_text())
        .field(
            "scope",
            ws.config()
                .queue
                .scope
                .clone()
                .map_or(Value::Null, Value::from),
        )
        .field("store_rev", ws.rev())
        .field("lines", here.total)
        .field("open", here.open)
        .field("stale", here.stale)
        .field("total_lines", all.total)
        .field("total_open", all.open)
        .field("total_stale", all.stale);
    if app.g.json {
        o.table(
            "rows",
            &[],
            shown
                .iter()
                .map(|r| {
                    let mut m = row_json(r);
                    if r.stale {
                        let ch: Vec<Value> = ws
                            .stale_changes(&r.anchor)
                            .into_iter()
                            .map(|(t, f, a, b)| serde_json::json!({"type": t, "field": f, "then": a, "now": b}))
                            .collect();
                        m.insert("stale_changes".into(), Value::Array(ch));
                    }
                    m
                })
                .collect(),
            Some(rows.len()),
        );
    } else {
        o.table(
            "rows",
            &[
                "anchor",
                "date",
                "payee",
                "amount",
                "account",
                "suggestion",
                "src",
                "conf",
                "stale",
            ],
            shown
                .iter()
                .map(|r| {
                    let mut m = row_text(r);
                    if r.stale {
                        m.insert(
                            "stale".into(),
                            Value::from(format!("stale: {}", ws.stale_fields(&r.anchor))),
                        );
                    }
                    m
                })
                .collect(),
            Some(rows.len()),
        );
        if shown.len() < rows.len() {
            o.hint("Show every row with --full.");
        }
    }
    o.emit();
    Ok(())
}

fn csv_cell(s: &str) -> String {
    if s.contains([',', '"', '\n', '\r']) {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

pub fn export(app: &mut App, words: &[String], csv: bool) -> Res<()> {
    if !csv {
        return Err(CliError::usage("Pass --csv (the only export format).")
            .hint("e.g. `ownpurse classify export --csv status:all > labels.csv`"));
    }
    let mut filter = filter_of(words)?;
    if !words
        .iter()
        .any(|w| w.starts_with("status:") || matches!(w.as_str(), "open" | "all" | "stale"))
    {
        filter.status = ownpurse_core::classify::Status::All;
    }
    let ws = open(app)?;
    let types: Vec<String> = ws
        .config()
        .label_types
        .iter()
        .map(|t| t.id.clone())
        .collect();
    let mut head = vec![
        "anchor",
        "org",
        "date",
        "payee",
        "description",
        "amount",
        "account",
    ]
    .into_iter()
    .map(str::to_string)
    .collect::<Vec<_>>();
    for t in &types {
        head.push(t.clone());
        head.push(format!("{t}_source"));
    }
    head.extend(["stale", "notes"].map(str::to_string));
    let mut out = head.join(",");
    out.push('\n');
    for r in ws.queue(&filter, Sort::DateAsc) {
        let Some(d) = ws.detail(&r.anchor) else {
            continue;
        };
        let mut cells = vec![
            r.anchor.to_string(),
            r.org.clone(),
            r.date.clone(),
            r.payee.clone(),
            r.description.clone(),
            r.amount.to_string(),
            r.account.clone(),
        ];
        for t in &types {
            let lv = d.labels.iter().find(|l| &l.label_type == t);
            match lv.and_then(|l| l.label.as_ref()) {
                Some(l) => {
                    cells.push(l.value.clone());
                    cells.push(l.source.to_string());
                }
                None => {
                    cells.push(String::new());
                    cells.push(String::new());
                }
            }
        }
        cells.push(if r.stale { "true" } else { "" }.into());
        cells.push(
            d.notes
                .iter()
                .map(|n| n.text.clone())
                .collect::<Vec<_>>()
                .join(" | "),
        );
        out.push_str(
            &cells
                .iter()
                .map(|c| csv_cell(c))
                .collect::<Vec<_>>()
                .join(","),
        );
        out.push('\n');
    }
    use std::io::Write;
    let mut so = std::io::stdout().lock();
    let _ = so.write_all(out.as_bytes());
    let _ = so.flush();
    Ok(())
}

// ── agent writes (spec §5a) ─────────────────────────────────────────────────────────────────────────────

/// The flags every agent write carries.
#[derive(Debug, Clone, Default)]
pub struct AgentFlags {
    pub expect_store_rev: Option<i64>,
    pub client: Option<String>,
    pub caption: Option<String>,
    pub allow_bulk: bool,
    pub overwrite: bool,
    pub session: Option<String>,
    pub direct: bool,
}

/// The agent write commands.
pub enum AgentOp {
    Set {
        words: Vec<String>,
    },
    Note {
        target: String,
        text: String,
    },
    GroupAdd {
        group: String,
        anchors: Vec<String>,
        role: Option<String>,
    },
    GroupCreate {
        group_kind: String,
        title: String,
    },
    GroupRemove {
        group: String,
        anchors: Vec<String>,
    },
    GroupStatus {
        group: String,
        status: String,
    },
    Undo {
        batch: bool,
        anchor: Option<String>,
        events: Vec<i64>,
    },
    AddProject {
        id: String,
        name: String,
        aliases: Vec<String>,
        dry_run: bool,
        confirm: bool,
    },
}

fn targets_json(words: &[String]) -> Res<(&'static str, Value)> {
    let parsed: Vec<_> = words.iter().map(|w| Anchor::parse(w)).collect();
    if !words.is_empty() && parsed.iter().all(Result::is_ok) {
        return Ok(("anchors", Value::from(words.to_vec())));
    }
    Ok(("filter", Value::from(words.join(" "))))
}

fn build_request(op: &AgentOp, f: &AgentFlags) -> Res<Value> {
    let rev = f.expect_store_rev.ok_or_else(|| {
        CliError::usage("--expect-store-rev is required for a write (the store_rev you last read).")
            .hint("Read it with `ownpurse classify queue --json` (store_rev).")
    })?;
    let client = f.client.clone().ok_or_else(|| {
        CliError::usage("--client is required for a write (your name, e.g. --client claude).")
    })?;
    let mut m = Map::new();
    m.insert("v".into(), Value::from(1));
    m.insert("kind".into(), Value::from("classify"));
    m.insert(
        "cv".into(),
        Value::from(ownpurse_core::classify::request::CV),
    );
    m.insert("client".into(), Value::from(client));
    m.insert("expect_store_rev".into(), Value::from(rev));
    if let Some(c) = &f.caption {
        m.insert("caption".into(), Value::from(c.as_str()));
    }
    if f.allow_bulk {
        m.insert("allow_bulk".into(), Value::from(true));
    }
    if f.overwrite {
        m.insert("overwrite".into(), Value::from(true));
    }
    let op_name = match op {
        AgentOp::Set { words } => {
            let (labels, targets): (Vec<&String>, Vec<&String>) =
                words.iter().partition(|w| w.contains('='));
            if labels.is_empty() {
                return Err(CliError::usage(
                    "Give at least one label: <type>=<id>, e.g. project=harvest.",
                ));
            }
            if targets.is_empty() {
                return Err(CliError::usage(
                    "Give the lines: anchors, or a filter such as `org:roast month:2025-09`.",
                ));
            }
            let mut l = Map::new();
            for x in labels {
                let (k, v) = x.split_once('=').unwrap_or_default();
                l.insert(k.to_string(), Value::from(v));
            }
            m.insert("labels".into(), Value::Object(l));
            let t: Vec<String> = targets.into_iter().cloned().collect();
            let (k, v) = targets_json(&t)?;
            m.insert(k.into(), v);
            "set"
        }
        AgentOp::Note { target, text } => {
            if target.starts_with('g') && Anchor::parse(target).is_err() {
                m.insert("group".into(), Value::from(target.as_str()));
            } else {
                m.insert("anchor".into(), Value::from(target.as_str()));
            }
            m.insert("text".into(), Value::from(text.as_str()));
            "note"
        }
        AgentOp::GroupAdd {
            group,
            anchors,
            role,
        } => {
            m.insert("group".into(), Value::from(group.as_str()));
            m.insert("anchors".into(), Value::from(anchors.clone()));
            if let Some(r) = role {
                m.insert("role".into(), Value::from(r.as_str()));
            }
            "group_add"
        }
        AgentOp::GroupCreate { group_kind, title } => {
            m.insert("group_kind".into(), Value::from(group_kind.as_str()));
            m.insert("title".into(), Value::from(title.as_str()));
            "group_create"
        }
        AgentOp::GroupRemove { group, anchors } => {
            m.insert("group".into(), Value::from(group.as_str()));
            m.insert("anchors".into(), Value::from(anchors.clone()));
            "group_remove"
        }
        AgentOp::GroupStatus { group, status } => {
            m.insert("group".into(), Value::from(group.as_str()));
            m.insert("status".into(), Value::from(status.as_str()));
            "group_status"
        }
        AgentOp::Undo {
            batch,
            anchor,
            events,
        } => {
            match (batch, anchor, events.is_empty()) {
                (true, None, true) => {
                    m.insert("what".into(), Value::from("batch"));
                }
                (false, Some(a), true) => {
                    m.insert("what".into(), Value::from("line"));
                    m.insert("anchor".into(), Value::from(a.as_str()));
                }
                (false, None, false) => {
                    m.insert("what".into(), Value::from("events"));
                    m.insert("events".into(), Value::from(events.clone()));
                }
                _ => {
                    return Err(CliError::usage(
                        "Undo one thing: --batch (the latest agent batch), an anchor (your last write on it) or --events 3,4.",
                    ));
                }
            }
            "undo"
        }
        AgentOp::AddProject {
            id,
            name,
            aliases,
            dry_run,
            confirm,
        } => {
            if !dry_run && !confirm {
                return Err(CliError::usage(
                    "A config write needs --confirm (run with --dry-run first to see the diff).",
                ));
            }
            m.insert(
                "edits".into(),
                serde_json::json!([{"edit": "add_choice", "label_type": "project", "id": id,
                                    "name": name, "aliases": aliases}]),
            );
            m.insert("dry_run".into(), Value::from(*dry_run));
            m.insert("confirm".into(), Value::from(*confirm));
            "config"
        }
    };
    m.insert("op".into(), Value::from(op_name));
    Ok(Value::Object(m))
}

/// A running TUI for this profile to route through, unless --direct.
fn route(app: &App, f: &AgentFlags) -> Res<Option<ownpurse_core::bridge::Session>> {
    use ownpurse_core::bridge as wire;
    if f.direct {
        return Ok(None);
    }
    let all = wire::sessions(&wire::Dirs::process());
    if let Some(key) = &f.session {
        return wire::find_session(&all, key)
            .cloned()
            .map(Some)
            .map_err(|_| CliError::usage(format!("No single live TUI session matches `{key}`.")));
    }
    let mine: Vec<_> = all
        .into_iter()
        .filter(|s| s.profile == app.config.profile.name)
        .collect();
    match mine.len() {
        0 => Ok(None),
        1 => Ok(mine.into_iter().next()),
        n => Err(CliError::usage(format!(
            "{n} TUI sessions run this profile: pass --session <id|label>, or --direct to write the store."
        ))),
    }
}

fn response_error(resp: &Value) -> CliError {
    let mut e = CliError::new(
        resp["error"].as_str().unwrap_or("error"),
        resp["exit"].as_i64().map_or(1, |x| x as i32),
        resp["message"].as_str().unwrap_or("The write was refused."),
    );
    for h in resp["help"].as_array().into_iter().flatten() {
        if let Some(h) = h.as_str() {
            e = e.hint(h);
        }
    }
    e
}

/// Run one agent write: through the running TUI when one serves this profile (so `user_busy` and the
/// notice/undo apply), else directly on the store.
pub fn agent_write(app: &mut App, op: AgentOp, f: &AgentFlags) -> Res<()> {
    let doc = build_request(&op, f)?;
    let req = ClassifyRequest::from_json(&doc)?;
    let resp = match route(app, f)? {
        Some(s) => {
            let r = ownpurse_core::bridge::request(
                std::path::Path::new(&s.socket),
                &req.to_json(),
                std::time::Duration::from_secs(15),
            )
            .map_err(|e| {
                CliError::runtime("bridge", format!("Could not reach TUI {}: {e}", s.pid))
                    .hint("Retry with --direct to write the store without the TUI.")
            })?;
            if r["ok"] != true {
                return Err(response_error(&r));
            }
            r
        }
        None => {
            let mut ws = open(app)?;
            ws.apply(&req)?.to_json()
        }
    };
    emit_outcome(app, &resp);
    Ok(())
}

fn emit_outcome(app: &App, r: &Value) {
    let mut o = app.out();
    for k in [
        "op",
        "summary",
        "batch",
        "lines",
        "store_rev",
        "group",
        "dry_run",
    ] {
        if let Some(v) = r.get(k).filter(|v| !v.is_null()) {
            o.field(k, v.clone());
        }
    }
    o.field("events", r["events"].clone());
    let anchors: Vec<Map<String, Value>> = r["anchors"]
        .as_array()
        .into_iter()
        .flatten()
        .map(|a| obj![("anchor", a.clone())])
        .collect();
    if app.g.json {
        o.field("anchors", r["anchors"].clone());
        o.field("skipped", r["skipped"].clone());
        o.field("effects", r["effects"].clone());
        if let Some(v) = r.get("summary_view") {
            o.field("view", v.clone());
        }
        if let Some(d) = r.get("diff") {
            o.field("diff", d.clone());
        }
    } else {
        if !anchors.is_empty() {
            o.table("anchors", &["anchor"], anchors, None);
        }
        let skipped: Vec<Map<String, Value>> = r["skipped"]
            .as_array()
            .into_iter()
            .flatten()
            .map(|s| {
                obj![
                    ("anchor", s["anchor"].clone()),
                    ("type", s["type"].clone()),
                    ("reason", s["reason"].clone())
                ]
            })
            .collect();
        if !skipped.is_empty() {
            o.table("skipped", &["anchor", "type", "reason"], skipped, None);
            o.hint("Lines you confirmed are never overwritten by an agent unless --overwrite.");
        }
    }
    o.emit();
    // the diff as it is (a unified diff reads badly folded into one field)
    if !app.g.json
        && let Some(d) = r.get("diff").and_then(Value::as_str)
    {
        print!("diff:\n{d}");
    }
}
