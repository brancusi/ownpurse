//! The queue model the CLI and the TUI share. A [`Workspace`] holds the classifiable lines (read from the
//! record once), the live config and the open store; every read is a plain function of those three, and every
//! write goes through the store (an event) or the config file (atomic write + a `config_written` event).
//!
//! Every write takes a [`WriteCtx`]: the actor (`human`, `agent:<name>`), the batch (one send) and an optional
//! caption. Agent sends are limited to 50 lines unless `allow_bulk` (`too_many`).
//!
//! STABLE INTERFACE (v0.11.0): the TUI builds on these types and signatures; changes are additive only.

use super::anchor::{Anchor, Fingerprint, Line};
use super::config::{
    Choice, ClassifyConfig, ConfigEdit, ConfigFile, ConfigIssue, ItemRef, LabelKind, Reload, Rule,
    Staged,
};
use super::rules::WouldMatch;
use super::store::{
    EventKind, GROUP_STATUSES, Group, Label, NoteTarget, Store, Suggestion, event_anchors,
    evidence_json, now_iso,
};
use super::{Actor, BatchId, ClassifyError, Confidence, Evidence, Glyph, Result, Source};
use rust_decimal::Decimal;
use serde_json::{Value, json};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

/// Agent sends may write at most this many lines without `allow_bulk`.
pub const BULK_LIMIT: usize = 50;

/// Where the config and the store live: `ownpurse.json` `"classify": {"config": …, "store": …}`, relative to
/// the config file's folder; defaults `classify.toml` and `db/classify.sqlite`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClassifyPaths {
    pub config: PathBuf,
    pub store: PathBuf,
}

impl ClassifyPaths {
    pub fn resolve(config: &crate::config::Config) -> ClassifyPaths {
        let get = |k: &str, dflt: &str| -> PathBuf {
            let v = config
                .settings
                .get_str(&format!("classify.{k}"))
                .unwrap_or(dflt);
            let p = Path::new(v);
            if p.is_absolute() {
                p.to_path_buf()
            } else {
                config.root.join(p)
            }
        };
        ClassifyPaths {
            config: get("config", "classify.toml"),
            store: get("store", "db/classify.sqlite"),
        }
    }
}

/// Who writes, as part of which send.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WriteCtx {
    pub actor: Actor,
    pub batch: BatchId,
    pub caption: Option<String>,
    /// Lift the 50-line limit for agent sends (`--allow-bulk`).
    pub allow_bulk: bool,
}

impl WriteCtx {
    pub fn new(actor: Actor, batch: BatchId) -> WriteCtx {
        WriteCtx {
            actor,
            batch,
            caption: None,
            allow_bulk: false,
        }
    }
    pub fn caption(mut self, c: impl Into<String>) -> WriteCtx {
        self.caption = Some(c.into());
        self
    }
}

/// What a write did.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct WriteOutcome {
    /// Event ids appended (empty when nothing changed).
    pub events: Vec<i64>,
    /// Lines touched.
    pub lines: usize,
    /// Past-tense summary for the notice: `set project Harvest Blend launch on 12 lines`.
    pub summary: String,
    /// The store revision after the write.
    pub rev: i64,
}

/// One cell of the queue's suggestion column: a confirmed label or the best suggestion.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cell {
    pub value: String,
    /// Display name (`Harvest Blend launch`).
    pub name: String,
    pub source: Source,
    pub confidence: Option<Confidence>,
    /// A confirmed label (true) or a suggestion (false).
    pub confirmed: bool,
}

/// `project / bearer / treatment`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Triple {
    pub project: Option<Cell>,
    pub bearer: Option<Cell>,
    pub treatment: Option<Cell>,
}

/// One row of the queue.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QueueRow {
    pub anchor: Anchor,
    pub org: String,
    pub date: String,
    pub payee: String,
    pub description: String,
    pub amount: Decimal,
    /// Account code.
    pub account: String,
    pub suggestion: Triple,
    /// The weakest shown cell's source: a suggestion's R/J, else a confirmation's H/A, else `·`.
    pub glyph: Glyph,
    /// Lowest confidence among the unconfirmed suggested cells; `None` when nothing is suggested.
    pub confidence: Option<Confidence>,
    /// Any label on the line is stale (V19).
    pub stale: bool,
    /// A required label type is unconfirmed, or a label is stale.
    pub open: bool,
}

/// A confirmed label as the detail pane shows it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LabelState {
    pub value: String,
    pub name: String,
    pub source: Source,
    pub actor: Actor,
    pub confidence: Option<Confidence>,
    pub evidence: Vec<Evidence>,
    pub at: String,
    pub event_id: i64,
    pub caption: Option<String>,
    pub stale: bool,
    /// When stale: `(field, then, now)` (the old value stays visible).
    pub changed: Vec<(&'static str, String, String)>,
}

/// A suggestion as the detail pane shows it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SuggestionState {
    pub value: String,
    pub name: String,
    pub source: Source,
    pub confidence: Confidence,
    pub evidence: Vec<Evidence>,
    /// The rule id when the source is a rule.
    pub rule: Option<String>,
}

/// One label type on one line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LabelView {
    pub label_type: String,
    pub title: String,
    pub kind: LabelKind,
    pub required: bool,
    pub label: Option<LabelState>,
    pub suggestion: Option<SuggestionState>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NoteView {
    pub event_id: i64,
    pub at: String,
    pub actor: Actor,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GroupMembership {
    pub group_id: String,
    pub kind: String,
    pub title: String,
    pub status: String,
    pub role: Option<String>,
}

/// Everything the detail pane shows for one line.
#[derive(Debug, Clone, PartialEq)]
pub struct LineDetail {
    pub line: Line,
    /// In config label-type order (archived types only when the line has a label of that type).
    pub labels: Vec<LabelView>,
    pub notes: Vec<NoteView>,
    pub groups: Vec<GroupMembership>,
    /// Rules that match the line now, highest priority first.
    pub rules_fired: Vec<String>,
    /// Group kinds proposed by rules: `(rule, kind)`.
    pub group_kinds: Vec<(String, String)>,
}

/// Which lines the queue shows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Status {
    /// Lines with an unconfirmed required label type, or a stale label (default).
    #[default]
    Open,
    All,
    /// Lines with a stale label.
    Stale,
}

/// `since:` in a filter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Since {
    Today,
    Date(String),
}

/// A parsed filter: `org:roast acct:53 month:2025-09 project:harvest conf<0.7 by:agent since:today status:open`.
/// Tokens AND together; repeated `org:`/`acct:` tokens OR within their kind; bare words search payee and
/// description (case-insensitive). Additional keys: `year:2025`, `bearer:<id>`, `treatment:<id>`,
/// `<type>:<id>` for any label type, `conf>=0.9`, `stale`, `open`, `all`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Filter {
    pub orgs: Vec<String>,
    /// Account code prefixes.
    pub accounts: Vec<String>,
    /// `YYYY-MM`.
    pub month: Option<String>,
    /// `YYYY`.
    pub year: Option<String>,
    /// `(label type, choice id)`: matches a confirmed label or, failing that, the suggestion.
    pub labels: Vec<(String, String)>,
    /// `conf<x`: lines whose row confidence is below x.
    pub conf_below: Option<Confidence>,
    /// `conf>=x`.
    pub conf_at_least: Option<Confidence>,
    /// `by:agent`, `by:human`, `by:agent:<name>`, `by:rule:<id>`: lines with a label/suggestion from them.
    pub by: Option<String>,
    pub since: Option<Since>,
    pub status: Status,
    pub text: Vec<String>,
}

fn filter_err(tok: &str, why: &str) -> ClassifyError {
    ClassifyError::new("filter", format!("filter token '{tok}': {why}"))
        .hint("e.g. org:roast acct:53 month:2025-09 project:harvest conf<0.7 by:agent since:today status:open")
}

fn parse_conf(tok: &str, s: &str) -> Result<Confidence> {
    match s.parse::<Decimal>() {
        Ok(d) if d >= Decimal::ZERO && d <= Decimal::ONE => Ok(d.normalize()),
        _ => Err(filter_err(tok, "expected a number between 0 and 1")),
    }
}

impl Filter {
    /// Parse filter text; errors are `filter` (exit 2) and name the token.
    pub fn parse(text: &str) -> Result<Filter> {
        let mut f = Filter::default();
        for tok in text.split_whitespace() {
            if let Some(v) = tok.strip_prefix("conf<") {
                f.conf_below = Some(parse_conf(tok, v)?);
                continue;
            }
            if let Some(v) = tok.strip_prefix("conf>=") {
                f.conf_at_least = Some(parse_conf(tok, v)?);
                continue;
            }
            match tok {
                "open" => {
                    f.status = Status::Open;
                    continue;
                }
                "all" => {
                    f.status = Status::All;
                    continue;
                }
                "stale" => {
                    f.status = Status::Stale;
                    continue;
                }
                _ => {}
            }
            let Some((k, v)) = tok.split_once(':') else {
                f.text.push(tok.to_string());
                continue;
            };
            if v.is_empty() {
                return Err(filter_err(tok, "missing value"));
            }
            match k {
                "org" => f.orgs.extend(v.split(',').map(str::to_string)),
                "acct" | "account" => f.accounts.extend(v.split(',').map(str::to_string)),
                "month" => {
                    let ok = v.len() == 7 && crate::dates::parse_iso(&format!("{v}-01")).is_some();
                    if !ok {
                        return Err(filter_err(tok, "expected YYYY-MM"));
                    }
                    f.month = Some(v.to_string());
                }
                "year" => {
                    if v.len() != 4 || !v.chars().all(|c| c.is_ascii_digit()) {
                        return Err(filter_err(tok, "expected YYYY"));
                    }
                    f.year = Some(v.to_string());
                }
                "by" => {
                    let ok =
                        ["agent", "human", "rule", "jev"].contains(&v) || Actor::parse(v).is_ok();
                    if !ok {
                        return Err(filter_err(
                            tok,
                            "expected agent, human, rule, agent:<name> or rule:<id>",
                        ));
                    }
                    f.by = Some(v.to_string());
                }
                "since" => {
                    f.since = Some(if v == "today" {
                        Since::Today
                    } else if crate::dates::parse_iso(v).is_some() && v.len() == 10 {
                        Since::Date(v.to_string())
                    } else {
                        return Err(filter_err(tok, "expected today or YYYY-MM-DD"));
                    });
                }
                "status" => {
                    f.status = match v {
                        "open" => Status::Open,
                        "all" => Status::All,
                        "stale" => Status::Stale,
                        _ => return Err(filter_err(tok, "expected open, all or stale")),
                    }
                }
                ty => f.labels.push((ty.to_string(), v.to_string())),
            }
        }
        Ok(f)
    }

    /// The canonical text (round-trips through `parse`).
    pub fn to_text(&self) -> String {
        let mut out: Vec<String> = vec![];
        out.extend(self.orgs.iter().map(|o| format!("org:{o}")));
        out.extend(self.accounts.iter().map(|a| format!("acct:{a}")));
        out.extend(self.year.iter().map(|y| format!("year:{y}")));
        out.extend(self.month.iter().map(|m| format!("month:{m}")));
        out.extend(self.labels.iter().map(|(k, v)| format!("{k}:{v}")));
        out.extend(self.conf_below.iter().map(|c| format!("conf<{c}")));
        out.extend(self.conf_at_least.iter().map(|c| format!("conf>={c}")));
        out.extend(self.by.iter().map(|b| format!("by:{b}")));
        out.extend(self.since.iter().map(|s| match s {
            Since::Today => "since:today".to_string(),
            Since::Date(d) => format!("since:{d}"),
        }));
        match self.status {
            Status::Open => {}
            Status::All => out.push("status:all".into()),
            Status::Stale => out.push("status:stale".into()),
        }
        out.extend(self.text.iter().cloned());
        out.join(" ")
    }
}

/// Queue order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Sort {
    /// Largest absolute amount first (default); ties by date, then anchor.
    #[default]
    AmountDesc,
    DateAsc,
    DateDesc,
    /// Stale lines first (they need a look before anything else), then as `AmountDesc`. The CLI's default.
    StaleFirst,
}

/// Header counts: `classify: 214 open · 3 stale`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct QueueCounts {
    pub total: usize,
    pub open: usize,
    pub stale: usize,
}

/// The bulk-accept preview: lines in the filter whose every required type has a suggestion ≥ threshold, not
/// stale, not already labelled (§10.8).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct BulkPreview {
    pub threshold: Confidence,
    pub anchors: Vec<Anchor>,
    /// Sum of the lines' amounts.
    pub total: Decimal,
    /// `(choice id, lines)`, by count descending.
    pub by_project: Vec<(String, usize)>,
    pub by_bearer: Vec<(String, usize)>,
}

/// A `classify suggest` run.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SuggestSummary {
    /// Lines considered (in the filter).
    pub lines: usize,
    /// Lines with at least one new or changed suggestion written.
    pub suggested: usize,
    /// Suggestion events written.
    pub events: usize,
    /// `(rule id, lines it suggested on)`, by count descending then id.
    pub by_rule: Vec<(String, usize)>,
    /// Per rule: lines it matched and the labels per type it writes (new or changed), by rule priority.
    pub rules: Vec<RuleStat>,
    /// True when nothing was appended (`--dry-run`).
    pub dry_run: bool,
}

/// One rule in a suggest run.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RuleStat {
    pub rule: String,
    /// Lines in the filter the rule matched.
    pub lines: usize,
    /// `(label type, suggestions written or to write)`, in label-type order.
    pub labels: Vec<(String, usize)>,
}

/// The classification workspace.
pub struct Workspace {
    config: ConfigFile,
    store: Store,
    lines: Vec<Line>,
    /// `(org alias, [(code, name)])` charts for the should-be-account picker.
    charts: Vec<(String, Vec<(String, String)>)>,
    index: HashMap<Anchor, usize>,
    /// Rows and scope membership per line, rebuilt lazily: in full when the config or an undo changes things,
    /// otherwise only for the lines new events touched (a write re-reads a handful of rows, not 5,000).
    cache: std::cell::RefCell<RowCache>,
    /// The configured org aliases when opened from a record (pair rules are checked against them).
    aliases: Option<Vec<String>>,
}

/// The materialised queue rows (one per line, in line order) and whether each is inside the scope.
#[derive(Default)]
struct RowCache {
    valid: bool,
    rev: i64,
    config_hash: String,
    rows: Vec<QueueRow>,
    in_scope: Vec<bool>,
}

fn not_found(what: impl Into<String>) -> ClassifyError {
    ClassifyError::new("not_found", what)
}

fn usage(what: impl Into<String>) -> ClassifyError {
    ClassifyError::new("usage", what)
}

/// Payload helpers.
pub(crate) fn label_json(
    anchor: &Anchor,
    ty: &str,
    value: &str,
    source: &Source,
    confidence: Option<Confidence>,
    evidence: &[Evidence],
    fp: &Fingerprint,
) -> Value {
    json!({
        "anchor": anchor.to_string(), "type": ty, "value": value, "source": source.to_string(),
        "confidence": confidence.map(|c| c.to_string()), "evidence": evidence_json(evidence),
        "fingerprint": fp.to_json(),
    })
}

pub(crate) fn with_caption(mut v: Value, w: &WriteCtx) -> Value {
    if let (Some(c), Some(m)) = (&w.caption, v.as_object_mut()) {
        m.insert("caption".into(), Value::String(c.clone()));
    }
    v
}

/// The source a confirmation by `actor` records: agents sign their labels; a human keeps the suggestion's.
fn confirm_source(actor: &Actor, suggested: &Source) -> Source {
    match actor.as_str().strip_prefix("agent:") {
        Some(n) => Source::Agent(n.to_string()),
        None => suggested.clone(),
    }
}

fn set_source(actor: &Actor) -> Source {
    match actor.as_str().strip_prefix("agent:") {
        Some(n) => Source::Agent(n.to_string()),
        None => Source::Human,
    }
}

fn count_desc(m: HashMap<String, usize>) -> Vec<(String, usize)> {
    let mut v: Vec<(String, usize)> = m.into_iter().collect();
    v.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    v
}

impl Workspace {
    /// Open from the record (read-only) for these orgs (`(alias, tenant_id)`), the config and the store.
    pub fn open(
        paths: &ClassifyPaths,
        record: &crate::view::View<'_>,
        orgs: &[(String, String)],
    ) -> Result<Workspace> {
        let config = ConfigFile::load(&paths.config).map_err(config_error)?;
        let aliases: Vec<String> = orgs.iter().map(|(a, _)| a.clone()).collect();
        let issues = super::config::check_orgs(&config.text, &config.name(), &aliases);
        if !issues.is_empty() {
            return Err(config_error(issues));
        }
        let store = Store::open_lazy(&paths.store)?;
        let lines = super::anchor::enumerate(record, orgs)?;
        let mut charts = vec![];
        for (alias, tid) in orgs {
            let mut c: Vec<(String, String)> = record
                .payloads(tid, "Accounts")?
                .iter()
                .filter(|a| a.get("Status").and_then(Value::as_str) != Some("ARCHIVED"))
                .filter_map(|a| {
                    let code = a.get("Code").and_then(Value::as_str)?;
                    let name = a.get("Name").and_then(Value::as_str).unwrap_or("");
                    (!code.is_empty()).then(|| (code.to_string(), name.to_string()))
                })
                .collect();
            c.sort();
            charts.push((alias.clone(), c));
        }
        let mut w = Workspace::from_parts(config, store, lines, charts);
        w.aliases = Some(aliases);
        Ok(w)
    }

    /// Build from parts (tests, benches).
    pub fn from_parts(
        config: ConfigFile,
        store: Store,
        lines: Vec<Line>,
        charts: Vec<(String, Vec<(String, String)>)>,
    ) -> Workspace {
        let index = lines
            .iter()
            .enumerate()
            .map(|(i, l)| (l.anchor.clone(), i))
            .collect();
        Workspace {
            config,
            store,
            lines,
            charts,
            index,
            cache: Default::default(),
            aliases: None,
        }
    }

    /// Drop the cached rows (tests that change lines in place).
    #[cfg(test)]
    pub(crate) fn invalidate_cache(&mut self) {
        self.cache.get_mut().valid = false;
    }

    fn scope_ok(&self, scope: &Option<Filter>, line: &Line, row: &QueueRow) -> bool {
        scope
            .as_ref()
            .is_none_or(|s| self.filter_matches(s, line, row))
    }

    /// The anchors whose rows new events may change; `None` means rebuild everything (an undo).
    fn touched(&self, from_rev: i64) -> Option<HashSet<Anchor>> {
        let start = usize::try_from(from_rev).ok()?;
        let mut out = HashSet::new();
        for e in self.store.events().get(start..)? {
            match e.kind {
                EventKind::Undo | EventKind::ConfigWritten => return None,
                EventKind::Suggestion => {
                    for s in e.payload["suggestions"].as_array().into_iter().flatten() {
                        if let Some(a) = s["anchor"].as_str().and_then(|a| Anchor::parse(a).ok()) {
                            out.insert(a);
                        }
                    }
                }
                _ => out.extend(event_anchors(e)),
            }
        }
        Some(out)
    }

    /// The row cache, brought up to the store revision and config.
    fn rows(&self) -> std::cell::Ref<'_, RowCache> {
        {
            let mut c = self.cache.borrow_mut();
            let rev = self.store.rev();
            let fresh = c.valid && c.config_hash == self.config.hash;
            let touched = if fresh && c.rev != rev {
                self.touched(c.rev)
            } else {
                None
            };
            let scope = self.scope();
            if fresh && c.rev == rev {
                // up to date
            } else if let Some(t) = touched {
                for a in t {
                    if let Some(&i) = self.index.get(&a) {
                        let r = self.row(&self.lines[i]);
                        c.in_scope[i] = self.scope_ok(&scope, &self.lines[i], &r);
                        c.rows[i] = r;
                    }
                }
                c.rev = rev;
            } else {
                // lines with no label or suggestion at all (most of a fresh year) get the empty row without
                // the per-type lookups
                let st = self.store.state();
                let mut busy: HashSet<&Anchor> = st.labels.keys().map(|(a, _)| a).collect();
                busy.extend(st.suggestions.keys().map(|(a, _)| a));
                let rows: Vec<QueueRow> = self
                    .lines
                    .iter()
                    .map(|l| {
                        if busy.contains(&l.anchor) {
                            self.row(l)
                        } else {
                            self.empty_row(l)
                        }
                    })
                    .collect();
                c.in_scope = self
                    .lines
                    .iter()
                    .zip(&rows)
                    .map(|(l, r)| self.scope_ok(&scope, l, r))
                    .collect();
                c.rows = rows;
                c.rev = rev;
                c.config_hash = self.config.hash.clone();
                c.valid = true;
            }
        }
        self.cache.borrow()
    }

    pub fn config(&self) -> &ClassifyConfig {
        &self.config.config
    }
    pub fn config_file(&self) -> &ConfigFile {
        &self.config
    }
    pub fn store(&self) -> &Store {
        &self.store
    }
    pub(crate) fn store_mut(&mut self) -> &mut Store {
        &mut self.store
    }
    pub fn lines(&self) -> &[Line] {
        &self.lines
    }
    pub fn line(&self, anchor: &Anchor) -> Option<&Line> {
        self.index.get(anchor).map(|&i| &self.lines[i])
    }
    /// The store revision (last event id); agents send it as `--expect-rev`.
    pub fn rev(&self) -> i64 {
        self.store.rev()
    }
    /// Today (`YYYY-MM-DD`, the store clock; pinned in tests).
    pub fn today(&self) -> String {
        now_iso().chars().take(10).collect()
    }
    /// Refuse with `view_changed` (exit 4) unless the store is still at `rev`.
    pub fn expect_rev(&self, rev: i64) -> Result<()> {
        if self.rev() == rev {
            Ok(())
        } else {
            Err(ClassifyError::new(
                "view_changed",
                format!(
                    "the classify store moved from store rev {rev} to {} (expect_store_rev is stale; re-read `classify queue --json` store_rev)",
                    self.rev()
                ),
            ))
        }
    }

    /// Pick up events other writers appended; true if anything changed.
    pub fn refresh(&mut self) -> Result<bool> {
        self.store.refresh()
    }
    /// Re-read `classify.toml` after a hand edit (the last valid config stays live on error).
    pub fn reload_config(&mut self) -> Reload {
        let name = self.config.name();
        match &self.aliases {
            Some(a) => {
                let a = a.clone();
                self.config
                    .reload_checked(&|t| super::config::check_orgs(t, &name, &a))
            }
            None => self.config.reload(),
        }
    }

    // ---- reads ----

    pub(crate) fn label(&self, a: &Anchor, ty: &str) -> Option<&Label> {
        self.store.state().labels.get(&(a.clone(), ty.to_string()))
    }
    fn suggestion(&self, a: &Anchor, ty: &str) -> Option<&Suggestion> {
        self.store
            .state()
            .suggestions
            .get(&(a.clone(), ty.to_string()))
    }
    /// A label is stale when the line it was set on changed (or is gone).
    pub(crate) fn is_stale(&self, l: &Label) -> bool {
        self.line(&l.anchor)
            .is_none_or(|line| line.fingerprint != l.fingerprint)
    }
    fn labels_of(&self, a: &Anchor) -> impl Iterator<Item = &Label> {
        self.config
            .config
            .label_types
            .iter()
            .filter_map(move |t| self.label(a, &t.id))
    }

    fn cell(&self, line: &Line, ty: &str) -> Option<Cell> {
        let name = |v: &str| self.config().choice_name(ty, v).to_string();
        if let Some(l) = self.label(&line.anchor, ty) {
            return Some(Cell {
                value: l.value.clone(),
                name: name(&l.value),
                source: l.source.clone(),
                confidence: l.confidence,
                confirmed: true,
            });
        }
        self.suggestion(&line.anchor, ty).map(|s| Cell {
            value: s.value.clone(),
            name: name(&s.value),
            source: s.source.clone(),
            confidence: Some(s.confidence),
            confirmed: false,
        })
    }

    /// The row of a line with no label and no suggestion.
    fn empty_row(&self, line: &Line) -> QueueRow {
        QueueRow {
            anchor: line.anchor.clone(),
            org: line.org.clone(),
            date: line.date.clone(),
            payee: line.payee.clone(),
            description: line.description.clone(),
            amount: line.amount,
            account: line.account_code.clone(),
            suggestion: Triple::default(),
            glyph: Glyph::None,
            confidence: None,
            stale: false,
            open: !self.config().queue.required.is_empty(),
        }
    }

    fn row(&self, line: &Line) -> QueueRow {
        let suggestion = Triple {
            project: self.cell(line, "project"),
            bearer: self.cell(line, "bearer"),
            treatment: self.cell(line, "treatment"),
        };
        let cells = [
            &suggestion.project,
            &suggestion.bearer,
            &suggestion.treatment,
        ];
        let unconfirmed: Vec<&Cell> = cells
            .iter()
            .filter_map(|c| c.as_ref())
            .filter(|c| !c.confirmed)
            .collect();
        let confidence = unconfirmed.iter().filter_map(|c| c.confidence).min();
        let glyph = if let Some(c) = unconfirmed.first() {
            c.source.glyph()
        } else {
            let mut g = Glyph::None;
            for t in ["project", "bearer", "treatment"] {
                if let Some(l) = self.label(&line.anchor, t) {
                    if l.actor.is_agent() {
                        g = Glyph::Agent;
                        break;
                    }
                    g = Glyph::Human;
                }
            }
            g
        };
        let stale = self.labels_of(&line.anchor).any(|l| self.is_stale(l));
        let missing = self
            .config()
            .queue
            .required
            .iter()
            .any(|t| self.label(&line.anchor, t).is_none());
        QueueRow {
            anchor: line.anchor.clone(),
            org: line.org.clone(),
            date: line.date.clone(),
            payee: line.payee.clone(),
            description: line.description.clone(),
            amount: line.amount,
            account: line.account_code.clone(),
            suggestion,
            glyph,
            confidence,
            stale,
            open: missing || stale,
        }
    }

    fn by_matches(&self, by: &str, a: &Anchor) -> bool {
        let actor_ok = |act: &Actor| match by {
            "agent" => act.is_agent(),
            "human" => act.is_human(),
            "rule" | "jev" => act.as_str().starts_with(&format!("{by}:")),
            _ => act.as_str() == by,
        };
        let source_ok = |s: &Source| match by {
            "rule" => matches!(s, Source::Rule(_)),
            "jev" => matches!(s, Source::Jev(_)),
            _ => s.to_string() == by,
        };
        self.labels_of(a)
            .any(|l| actor_ok(&l.actor) || source_ok(&l.source))
            || self
                .config()
                .label_types
                .iter()
                .filter_map(|t| self.suggestion(a, &t.id))
                .any(|s| source_ok(&s.source))
            || self
                .store
                .state()
                .writes_by_anchor
                .get(a)
                .is_some_and(|ids| {
                    ids.iter()
                        .filter_map(|id| self.store.event(*id))
                        .any(|e| actor_ok(&e.actor))
                })
    }

    fn since_matches(&self, since: &Since, a: &Anchor) -> bool {
        let day = match since {
            Since::Today => self.today(),
            Since::Date(d) => d.clone(),
        };
        self.store
            .state()
            .writes_by_anchor
            .get(a)
            .is_some_and(|ids| {
                ids.iter()
                    .filter_map(|id| self.store.event(*id))
                    .any(|e| e.at.get(..10).is_some_and(|d| d >= day.as_str()))
            })
    }

    fn filter_matches(&self, f: &Filter, line: &Line, row: &QueueRow) -> bool {
        if !f.orgs.is_empty() && !f.orgs.contains(&line.org) {
            return false;
        }
        if !f.accounts.is_empty()
            && !f
                .accounts
                .iter()
                .any(|a| line.account_code.starts_with(a.as_str()))
        {
            return false;
        }
        if f.month.as_ref().is_some_and(|m| line.month() != m) {
            return false;
        }
        if f.year
            .as_ref()
            .is_some_and(|y| !line.date.starts_with(y.as_str()))
        {
            return false;
        }
        for (ty, v) in &f.labels {
            let have = self
                .label(&line.anchor, ty)
                .map(|l| l.value.as_str())
                .or_else(|| self.suggestion(&line.anchor, ty).map(|s| s.value.as_str()));
            if have != Some(v.as_str()) {
                return false;
            }
        }
        if let Some(c) = f.conf_below
            && !row.confidence.is_some_and(|x| x < c)
        {
            return false;
        }
        if let Some(c) = f.conf_at_least
            && !row.confidence.is_some_and(|x| x >= c)
        {
            return false;
        }
        if let Some(by) = &f.by
            && !self.by_matches(by, &line.anchor)
        {
            return false;
        }
        if let Some(s) = &f.since
            && !self.since_matches(s, &line.anchor)
        {
            return false;
        }
        let status_ok = match f.status {
            Status::Open => row.open,
            Status::All => true,
            Status::Stale => row.stale,
        };
        if !status_ok {
            return false;
        }
        f.text.iter().all(|t| {
            let t = t.to_lowercase();
            line.payee.to_lowercase().contains(&t)
                || line.description.to_lowercase().contains(&t)
                || line.anchor.to_string().to_lowercase().contains(&t)
        })
    }

    /// The config's `[queue] scope` as a filter (its status is ignored); `None` when unset. Every read and
    /// write over lines ANDs it in: queue, counts, suggest, bulk, export, would-match.
    pub fn scope(&self) -> Option<Filter> {
        let s = self.config().queue.scope.as_deref()?;
        Filter::parse(s).ok().map(|f| Filter {
            status: Status::All,
            ..f
        })
    }

    /// The lines inside the scope, each with its row.
    fn scoped(&self) -> Vec<(&Line, QueueRow)> {
        let c = self.rows();
        self.lines
            .iter()
            .enumerate()
            .filter(|(i, _)| c.in_scope[*i])
            .map(|(i, l)| (l, c.rows[i].clone()))
            .collect()
    }

    /// Header counts over the whole scope: `classify: 214 open · 3 stale`.
    pub fn counts(&self) -> QueueCounts {
        self.counts_for(&Filter {
            status: Status::All,
            ..Filter::default()
        })
    }

    /// Counts for a filter inside the scope, status included: `total` is exactly the rows `queue(filter)`
    /// returns, `open` and `stale` how many of those are in each state.
    pub fn counts_for(&self, filter: &Filter) -> QueueCounts {
        let f = filter.clone();
        let mut c = QueueCounts::default();
        let cache = self.rows();
        let plain = f
            == Filter {
                status: Status::All,
                ..Filter::default()
            };
        for (i, r) in cache.rows.iter().enumerate() {
            if cache.in_scope[i] && (plain || self.filter_matches(&f, &self.lines[i], r)) {
                c.total += 1;
                c.open += usize::from(r.open);
                c.stale += usize::from(r.stale);
            }
        }
        c
    }

    pub fn queue(&self, filter: &Filter, sort: Sort) -> Vec<QueueRow> {
        let cache = self.rows();
        let mut idx: Vec<usize> = (0..cache.rows.len())
            .filter(|&i| {
                cache.in_scope[i] && self.filter_matches(filter, &self.lines[i], &cache.rows[i])
            })
            .collect();
        let rs = &cache.rows;
        match sort {
            Sort::AmountDesc => idx.sort_by(|&x, &y| {
                let (a, b) = (&rs[x], &rs[y]);
                b.amount
                    .abs()
                    .cmp(&a.amount.abs())
                    .then_with(|| a.date.cmp(&b.date))
                    .then_with(|| a.anchor.cmp(&b.anchor))
            }),
            // stale labels first, then the largest amounts
            Sort::StaleFirst => idx.sort_by(|&x, &y| {
                let (a, b) = (&rs[x], &rs[y]);
                b.stale
                    .cmp(&a.stale)
                    .then_with(|| b.amount.abs().cmp(&a.amount.abs()))
                    .then_with(|| a.date.cmp(&b.date))
                    .then_with(|| a.anchor.cmp(&b.anchor))
            }),
            Sort::DateAsc => idx.sort_by(|&x, &y| {
                let (a, b) = (&rs[x], &rs[y]);
                a.date.cmp(&b.date).then_with(|| a.anchor.cmp(&b.anchor))
            }),
            Sort::DateDesc => idx.sort_by(|&x, &y| {
                let (a, b) = (&rs[x], &rs[y]);
                b.date.cmp(&a.date).then_with(|| a.anchor.cmp(&b.anchor))
            }),
        }
        idx.into_iter().map(|i| rs[i].clone()).collect()
    }

    /// What changed under a line's stale labels: `(label type, field, then, now)`, in label-type order;
    /// empty when nothing is stale. The queue shows `stale` plus the fields (`stale: amount`).
    pub fn stale_changes(&self, anchor: &Anchor) -> Vec<(String, &'static str, String, String)> {
        let Some(line) = self.line(anchor) else {
            return vec![];
        };
        let mut out = vec![];
        for l in self.labels_of(anchor) {
            if self.is_stale(l) {
                for (field, then, now) in l.fingerprint.diff(&line.fingerprint) {
                    out.push((l.label_type.clone(), field, then, now));
                }
            }
        }
        out
    }

    /// The changed fields of a stale line, once each: `amount, date`.
    pub fn stale_fields(&self, anchor: &Anchor) -> String {
        let mut seen: Vec<&str> = vec![];
        for (_, f, _, _) in self.stale_changes(anchor) {
            let f = if f == "description_hash" {
                "description"
            } else {
                f
            };
            if !seen.contains(&f) {
                seen.push(f);
            }
        }
        seen.join(", ")
    }

    pub fn detail(&self, anchor: &Anchor) -> Option<LineDetail> {
        let line = self.line(anchor)?.clone();
        let cfg = self.config();
        let mut labels = vec![];
        for t in &cfg.label_types {
            let label = self.label(anchor, &t.id);
            if t.archived && label.is_none() {
                continue;
            }
            let name = |v: &str| match t.kind {
                LabelKind::Account => self
                    .account_choices(&line.org)
                    .iter()
                    .find(|(c, _)| c == v)
                    .map(|(c, n)| format!("{c} {n}"))
                    .unwrap_or_else(|| v.to_string()),
                LabelKind::Choice => cfg.choice_name(&t.id, v).to_string(),
            };
            labels.push(LabelView {
                label_type: t.id.clone(),
                title: t.title.clone(),
                kind: t.kind,
                required: cfg.queue.required.contains(&t.id),
                label: label.map(|l| {
                    let stale = self.is_stale(l);
                    LabelState {
                        value: l.value.clone(),
                        name: name(&l.value),
                        source: l.source.clone(),
                        actor: l.actor.clone(),
                        confidence: l.confidence,
                        evidence: l.evidence.clone(),
                        at: l.at.clone(),
                        event_id: l.event_id,
                        caption: l.caption.clone(),
                        stale,
                        changed: if stale {
                            l.fingerprint.diff(&line.fingerprint)
                        } else {
                            vec![]
                        },
                    }
                }),
                suggestion: self.suggestion(anchor, &t.id).map(|s| SuggestionState {
                    value: s.value.clone(),
                    name: name(&s.value),
                    source: s.source.clone(),
                    confidence: s.confidence,
                    evidence: s.evidence.clone(),
                    rule: match &s.source {
                        Source::Rule(r) => Some(r.clone()),
                        _ => None,
                    },
                }),
            });
        }
        let notes = self
            .store
            .state()
            .notes
            .iter()
            .filter(|n| n.target == NoteTarget::Line(anchor.clone()))
            .map(|n| NoteView {
                event_id: n.event_id,
                at: n.at.clone(),
                actor: n.actor.clone(),
                text: n.text.clone(),
            })
            .collect();
        let groups = self
            .store
            .state()
            .groups
            .values()
            .filter_map(|g| {
                let m = g.members.iter().find(|m| &m.anchor == anchor)?;
                Some(GroupMembership {
                    group_id: g.id.clone(),
                    kind: g.kind.clone(),
                    title: g.title.clone(),
                    status: g.status.clone(),
                    role: m.role.clone(),
                })
            })
            .collect();
        let ev = super::rules::evaluate(cfg, &line);
        Some(LineDetail {
            line,
            labels,
            notes,
            groups,
            rules_fired: ev.fired,
            group_kinds: ev.group_kinds,
        })
    }

    /// Active choices of a label type, in picker order (`p`+digit picks the N-th).
    pub fn choices(&self, label_type: &str) -> Vec<&Choice> {
        self.config()
            .label_type(label_type)
            .map(|t| t.active().collect())
            .unwrap_or_default()
    }
    /// The should-be-account picker: an org's chart `(code, name)`, by code.
    pub fn account_choices(&self, org: &str) -> &[(String, String)] {
        self.charts
            .iter()
            .find(|(o, _)| o == org)
            .map_or(&[][..], |(_, c)| c.as_slice())
    }
    /// Groups, by id.
    pub fn groups(&self) -> Vec<&Group> {
        let mut v: Vec<&Group> = self.store.state().groups.values().collect();
        v.sort_by_key(|g| g.id.trim_start_matches('g').parse::<i64>().unwrap_or(0));
        v
    }
    /// Lines and groups referencing an item (the Catalog's Used column).
    pub fn used(&self, item: &ItemRef) -> usize {
        let st = self.store.state();
        let mut anchors: HashSet<&Anchor> = HashSet::new();
        match item {
            ItemRef::Choice { label_type, id } => {
                for ((a, t), l) in &st.labels {
                    if t == label_type && &l.value == id {
                        anchors.insert(a);
                    }
                }
                for ((a, t), s) in &st.suggestions {
                    if t == label_type && &s.value == id {
                        anchors.insert(a);
                    }
                }
            }
            ItemRef::LabelType(ty) => {
                for (a, t) in st.labels.keys().chain(st.suggestions.keys()) {
                    if t == ty {
                        anchors.insert(a);
                    }
                }
            }
            ItemRef::Rule(id) => {
                let src = Source::Rule(id.clone());
                for ((a, _), l) in &st.labels {
                    if l.source == src {
                        anchors.insert(a);
                    }
                }
                for ((a, _), s) in &st.suggestions {
                    if s.source == src {
                        anchors.insert(a);
                    }
                }
            }
        }
        anchors.len()
    }
    /// The rule preview: how many lines `rule` would match, with 5 samples.
    pub fn would_match(&self, rule: &Rule) -> WouldMatch {
        match self.scope() {
            None => super::rules::would_match(self.config(), rule, &self.lines),
            Some(_) => {
                let lines: Vec<Line> = self.scoped().into_iter().map(|(l, _)| l.clone()).collect();
                super::rules::would_match(self.config(), rule, &lines)
            }
        }
    }

    fn bulk_candidates(&self, filter: &Filter, threshold: Confidence) -> Vec<&Line> {
        let req = &self.config().queue.required;
        let f = Filter {
            status: Status::All,
            ..filter.clone()
        };
        self.scoped()
            .into_iter()
            .filter(|(l, r)| {
                if r.stale || !self.filter_matches(&f, l, r) {
                    return false;
                }
                // a line with any confirmed required label is not a bulk candidate ("not already labelled")
                !req.is_empty()
                    && req.iter().all(|t| {
                        self.label(&l.anchor, t).is_none()
                            && self
                                .suggestion(&l.anchor, t)
                                .is_some_and(|s| s.confidence >= threshold)
                    })
            })
            .map(|(l, _)| l)
            .collect()
    }

    pub fn bulk_preview(&self, filter: &Filter, threshold: Confidence) -> BulkPreview {
        let lines = self.bulk_candidates(filter, threshold);
        let mut by_project: HashMap<String, usize> = HashMap::new();
        let mut by_bearer: HashMap<String, usize> = HashMap::new();
        for l in &lines {
            if let Some(s) = self.suggestion(&l.anchor, "project") {
                *by_project.entry(s.value.clone()).or_default() += 1;
            }
            if let Some(s) = self.suggestion(&l.anchor, "bearer") {
                *by_bearer.entry(s.value.clone()).or_default() += 1;
            }
        }
        BulkPreview {
            threshold,
            anchors: lines.iter().map(|l| l.anchor.clone()).collect(),
            total: lines.iter().map(|l| l.amount).sum(),
            by_project: count_desc(by_project),
            by_bearer: count_desc(by_bearer),
        }
    }

    // ---- writes (each one store event or more, attributed and undoable) ----

    fn check_bulk(w: &WriteCtx, n: usize) -> Result<()> {
        if w.actor.is_agent() && n > BULK_LIMIT && !w.allow_bulk {
            return Err(ClassifyError::new(
                "too_many",
                format!(
                    "this send would write {n} lines; the limit is {BULK_LIMIT} without --allow-bulk"
                ),
            ));
        }
        Ok(())
    }

    pub(crate) fn need_line(&self, a: &Anchor) -> Result<&Line> {
        self.line(a)
            .ok_or_else(|| not_found(format!("no classifiable line {a}")))
    }

    pub(crate) fn outcome(&self, events: Vec<i64>, lines: usize, summary: String) -> WriteOutcome {
        WriteOutcome {
            events,
            lines,
            summary,
            rev: self.rev(),
        }
    }

    pub(crate) fn line_title(&self, a: &Anchor) -> String {
        match self.line(a) {
            Some(l) => {
                let who = if l.payee.is_empty() {
                    &l.description
                } else {
                    &l.payee
                };
                format!("{who} {}", crate::money::fmt(l.amount.abs()))
            }
            None => a.to_string(),
        }
    }

    /// The `label_set` entries accepting a line's suggestions.
    pub(crate) fn accept_entries(&self, line: &Line, actor: &Actor) -> Vec<Value> {
        let mut out = vec![];
        for t in &self.config().label_types {
            let Some(s) = self.suggestion(&line.anchor, &t.id) else {
                continue;
            };
            if self
                .label(&line.anchor, &t.id)
                .is_some_and(|l| !self.is_stale(l))
            {
                continue;
            }
            out.push(label_json(
                &line.anchor,
                &t.id,
                &s.value,
                &confirm_source(actor, &s.source),
                Some(s.confidence),
                &s.evidence,
                &line.fingerprint,
            ));
        }
        out
    }

    /// Confirm every suggested label on the line (types already confirmed are left alone).
    pub fn accept(&mut self, anchor: &Anchor, w: &WriteCtx) -> Result<WriteOutcome> {
        let line = self.need_line(anchor)?.clone();
        let entries = self.accept_entries(&line, &w.actor);
        if entries.is_empty() {
            return Ok(self.outcome(vec![], 0, "nothing to accept".into()));
        }
        let n = entries.len();
        let ids = self.store.append(
            &w.actor,
            &w.batch,
            vec![(
                EventKind::LabelSet,
                with_caption(json!({"labels": entries}), w),
            )],
        )?;
        let summary = format!(
            "accepted {n} {} on {}",
            super::config::plural(n, "label"),
            self.line_title(anchor)
        );
        Ok(self.outcome(ids, 1, summary))
    }

    pub(crate) fn check_value(&self, line: &Line, label_type: &str, value: &str) -> Result<()> {
        let t = self.config().label_type(label_type).ok_or_else(|| {
            let names: Vec<&str> = self
                .config()
                .label_types
                .iter()
                .map(|t| t.id.as_str())
                .collect();
            let m = match super::config::did_you_mean(label_type, names) {
                Some(c) => format!(" (did you mean \"{c}\"?)"),
                None => String::new(),
            };
            usage(format!("unknown label type \"{label_type}\"{m}"))
        })?;
        match t.kind {
            LabelKind::Choice => match t.choice(value) {
                Some(c) if !c.archived => Ok(()),
                Some(_) => Err(usage(format!("{label_type} \"{value}\" is archived"))),
                None => {
                    let m =
                        match super::config::did_you_mean(value, t.active().map(|c| c.id.as_str()))
                        {
                            Some(c) => format!(" (did you mean \"{c}\"?)"),
                            None => String::new(),
                        };
                    Err(usage(format!("unknown {label_type} \"{value}\"{m}")))
                }
            },
            LabelKind::Account => {
                let chart = self.account_choices(&line.org);
                if chart.is_empty() || chart.iter().any(|(c, _)| c == value) {
                    Ok(())
                } else {
                    Err(usage(format!("{} has no account {value}", line.org)))
                }
            }
        }
    }

    /// Set one label (a choice id; for `kind = "account"` types an account code of the line's org).
    pub fn set_label(
        &mut self,
        anchor: &Anchor,
        label_type: &str,
        value: &str,
        w: &WriteCtx,
    ) -> Result<WriteOutcome> {
        self.set_labels(std::slice::from_ref(anchor), label_type, value, w)
    }

    /// Set one label on many lines in one send (agents: "reclassify these as Harvest Blend").
    pub fn set_labels(
        &mut self,
        anchors: &[Anchor],
        label_type: &str,
        value: &str,
        w: &WriteCtx,
    ) -> Result<WriteOutcome> {
        let mut seen: HashSet<&Anchor> = HashSet::new();
        let anchors: Vec<&Anchor> = anchors.iter().filter(|a| seen.insert(*a)).collect();
        Self::check_bulk(w, anchors.len())?;
        let mut items = vec![];
        let mut n = 0;
        for a in anchors.iter().copied() {
            let line = self.need_line(a)?;
            self.check_value(line, label_type, value)?;
            if self
                .label(a, label_type)
                .is_some_and(|l| l.value == value && !self.is_stale(l))
            {
                continue;
            }
            n += 1;
            let entry = label_json(
                a,
                label_type,
                value,
                &set_source(&w.actor),
                None,
                &[],
                &line.fingerprint,
            );
            items.push((
                EventKind::LabelSet,
                with_caption(json!({"labels": [entry]}), w),
            ));
        }
        let ids = self.store.append(&w.actor, &w.batch, items)?;
        let title = self
            .config()
            .label_type(label_type)
            .map(|t| t.title.to_lowercase())
            .unwrap_or_default();
        let name = self.config().choice_name(label_type, value).to_string();
        let summary = if n == 1 && anchors.len() == 1 {
            format!("set {title} {name} on {}", self.line_title(anchors[0]))
        } else {
            format!(
                "set {title} {name} on {n} {}",
                super::config::plural(n, "line")
            )
        };
        Ok(self.outcome(ids, n, summary))
    }

    pub fn clear_label(
        &mut self,
        anchor: &Anchor,
        label_type: &str,
        w: &WriteCtx,
    ) -> Result<WriteOutcome> {
        self.need_line(anchor)?;
        if self.label(anchor, label_type).is_none() {
            return Err(not_found(format!("{anchor} has no {label_type} label")));
        }
        let ids = self.store.append(
            &w.actor,
            &w.batch,
            vec![(
                EventKind::LabelCleared,
                with_caption(
                    json!({"labels": [{"anchor": anchor.to_string(), "type": label_type}]}),
                    w,
                ),
            )],
        )?;
        let summary = format!("cleared {label_type} on {}", self.line_title(anchor));
        Ok(self.outcome(ids, 1, summary))
    }

    pub fn add_note(
        &mut self,
        target: &NoteTarget,
        text: &str,
        w: &WriteCtx,
    ) -> Result<WriteOutcome> {
        let text = text.trim();
        if text.is_empty() {
            return Err(usage("a note needs text"));
        }
        let (t, title, lines) = match target {
            NoteTarget::Line(a) => {
                self.need_line(a)?;
                (json!({"anchor": a.to_string()}), self.line_title(a), 1)
            }
            NoteTarget::Group(g) => {
                let grp = self
                    .store
                    .state()
                    .groups
                    .get(g)
                    .ok_or_else(|| not_found(format!("no group {g}")))?;
                (json!({"group": g}), format!("group \"{}\"", grp.title), 0)
            }
        };
        let ids = self.store.append(
            &w.actor,
            &w.batch,
            vec![(
                EventKind::NoteAdded,
                with_caption(json!({"target": t, "text": text}), w),
            )],
        )?;
        Ok(self.outcome(ids, lines, format!("added a note to {title}")))
    }

    /// Undo the last active write on this line, whoever made it (`u`).
    pub fn undo_line(&mut self, anchor: &Anchor, w: &WriteCtx) -> Result<WriteOutcome> {
        let last = self
            .store
            .state()
            .writes_by_anchor
            .get(anchor)
            .and_then(|v| v.last().copied())
            .ok_or_else(|| not_found(format!("nothing to undo on {anchor}")))?;
        self.undo_events(&[last], w)
    }

    /// Undo the most recent agent batch not yet undone (`U`); pressing again walks further back.
    pub fn undo_batch(&mut self, w: &WriteCtx) -> Result<WriteOutcome> {
        let undone = self.store.undone();
        let batch = self
            .store
            .events()
            .iter()
            .rev()
            .find(|e| e.actor.is_agent() && e.kind != EventKind::Undo && !undone.contains(&e.id))
            .map(|e| e.batch.clone())
            .ok_or_else(|| not_found("no agent write to undo"))?;
        let ids: Vec<i64> = self
            .store
            .events()
            .iter()
            .filter(|e| {
                e.batch == batch
                    && e.actor.is_agent()
                    && e.kind != EventKind::Undo
                    && !undone.contains(&e.id)
            })
            .map(|e| e.id)
            .collect();
        let mut out = self.undo_events(&ids, w)?;
        out.summary = format!(
            "agent write undone · {} {}",
            out.lines,
            super::config::plural(out.lines, "line")
        );
        Ok(out)
    }

    /// Undo these events (Activity: an older batch). Config writes restore the previous file.
    pub fn undo_events(&mut self, ids: &[i64], w: &WriteCtx) -> Result<WriteOutcome> {
        if ids.is_empty() {
            return Err(not_found("nothing to undo"));
        }
        let undone = self.store.undone();
        let mut lines: HashSet<Anchor> = HashSet::new();
        let mut configs = vec![];
        for id in ids {
            let e = self
                .store
                .event(*id)
                .ok_or_else(|| not_found(format!("no event {id}")))?;
            if undone.contains(id) {
                return Err(usage(format!("event {id} is already undone")));
            }
            lines.extend(event_anchors(e));
            if e.kind == EventKind::ConfigWritten {
                configs.push((
                    e.id,
                    e.payload
                        .get("before_hash")
                        .and_then(Value::as_str)
                        .unwrap_or("")
                        .to_string(),
                ));
            }
        }
        // newest config write first: each restore puts back the file that write replaced
        configs.sort_by_key(|c| std::cmp::Reverse(c.0));
        for (_, before) in &configs {
            self.config.restore(before)?;
        }
        let mut sorted = ids.to_vec();
        sorted.sort_unstable();
        let ev = self.store.append(
            &w.actor,
            &w.batch,
            vec![(EventKind::Undo, with_caption(json!({"events": sorted}), w))],
        )?;
        let n = lines.len();
        let summary = if !configs.is_empty() && n == 0 {
            format!("restored the previous {}", self.config.name())
        } else if n == 1 {
            let a = lines.iter().next().cloned();
            format!(
                "undid the last write on {}",
                a.map(|a| self.line_title(&a)).unwrap_or_default()
            )
        } else {
            format!(
                "undid {} {} on {n} {}",
                sorted.len(),
                super::config::plural(sorted.len(), "write"),
                super::config::plural(n, "line")
            )
        };
        Ok(self.outcome(ev, n, summary))
    }

    /// Accept every line of [`Workspace::bulk_preview`].
    pub fn bulk_accept(
        &mut self,
        filter: &Filter,
        threshold: Confidence,
        w: &WriteCtx,
    ) -> Result<WriteOutcome> {
        let lines: Vec<Line> = self
            .bulk_candidates(filter, threshold)
            .into_iter()
            .cloned()
            .collect();
        Self::check_bulk(w, lines.len())?;
        let mut items = vec![];
        for l in &lines {
            let entries = self.accept_entries(l, &w.actor);
            if !entries.is_empty() {
                items.push((
                    EventKind::LabelSet,
                    with_caption(json!({"labels": entries}), w),
                ));
            }
        }
        let n = items.len();
        let ids = self.store.append(&w.actor, &w.batch, items)?;
        Ok(self.outcome(
            ids,
            n,
            format!("accepted {n} {}", super::config::plural(n, "line")),
        ))
    }

    /// What a suggest run over `filter` (inside the scope) would write: the summary and the events.
    fn plan_suggestions(&self, filter: &Filter) -> (SuggestSummary, Vec<(EventKind, Value)>) {
        let mut sum = SuggestSummary::default();
        let mut by_rule: HashMap<String, usize> = HashMap::new();
        let mut stats: HashMap<String, (usize, HashMap<String, usize>)> = HashMap::new();
        let mut items = vec![];
        for (line, row) in self.scoped() {
            if !self.filter_matches(filter, line, &row) {
                continue;
            }
            sum.lines += 1;
            let ev = super::rules::evaluate(self.config(), line);
            for r in &ev.fired {
                stats.entry(r.clone()).or_default().0 += 1;
            }
            let mut entries = vec![];
            let mut rules_here: HashSet<&str> = HashSet::new();
            for t in &self.config().label_types {
                let hit = ev.hits.iter().find(|h| h.label_type == t.id);
                let cur = self.suggestion(&line.anchor, &t.id);
                match (hit, cur) {
                    (Some(h), cur) => {
                        rules_here.insert(h.rule.as_str());
                        let src = Source::Rule(h.rule.clone());
                        let same = cur.is_some_and(|c| {
                            c.value == h.value
                                && c.source == src
                                && c.confidence == h.confidence
                                && c.fingerprint == line.fingerprint
                        });
                        if !same {
                            *stats
                                .entry(h.rule.clone())
                                .or_default()
                                .1
                                .entry(t.id.clone())
                                .or_default() += 1;
                            entries.push(label_json(
                                &line.anchor,
                                &t.id,
                                &h.value,
                                &src,
                                Some(h.confidence),
                                &h.evidence,
                                &line.fingerprint,
                            ));
                        }
                    }
                    // a rule suggestion that no rule makes any more is withdrawn
                    (None, Some(c)) if matches!(c.source, Source::Rule(_)) => entries.push(json!({
                        "anchor": line.anchor.to_string(), "type": t.id, "value": "",
                    })),
                    _ => {}
                }
            }
            for r in rules_here {
                *by_rule.entry(r.to_string()).or_default() += 1;
            }
            if !entries.is_empty() {
                sum.suggested += 1;
                items.push((EventKind::Suggestion, json!({"suggestions": entries})));
            }
        }
        sum.events = items.len();
        sum.by_rule = count_desc(by_rule);
        for r in self.config().rules_by_priority() {
            if let Some((lines, labels)) = stats.remove(&r.id) {
                let labels = self
                    .config()
                    .label_types
                    .iter()
                    .filter_map(|t| labels.get(&t.id).map(|n| (t.id.clone(), *n)))
                    .collect();
                sum.rules.push(RuleStat {
                    rule: r.id.clone(),
                    lines,
                    labels,
                });
            }
        }
        (sum, items)
    }

    /// What [`Workspace::suggest`] would write, without appending anything (`--dry-run`).
    pub fn suggest_dry_run(&self, filter: &Filter) -> SuggestSummary {
        let (mut sum, _) = self.plan_suggestions(filter);
        sum.dry_run = true;
        sum
    }

    /// Run the rules over the lines in `filter` and write new or changed suggestions (actor `rule:*`).
    pub fn suggest(
        &mut self,
        filter: &Filter,
        w: &WriteCtx,
    ) -> Result<(SuggestSummary, WriteOutcome)> {
        let (sum, items) = self.plan_suggestions(filter);
        let n = items.len();
        let ids = self.store.append(&w.actor, &w.batch, items)?;
        let out = self.outcome(
            ids,
            n,
            format!("suggested on {n} {}", super::config::plural(n, "line")),
        );
        Ok((sum, out))
    }

    // ---- groups ----

    /// Returns the outcome and the new group id.
    pub fn create_group(
        &mut self,
        kind: &str,
        title: &str,
        w: &WriteCtx,
    ) -> Result<(WriteOutcome, String)> {
        if !self.config().groups.kinds.iter().any(|k| k == kind) {
            return Err(usage(format!(
                "unknown group kind \"{kind}\" (kinds: {})",
                self.config().groups.kinds.join(", ")
            )));
        }
        if title.trim().is_empty() {
            return Err(usage("a group needs a title"));
        }
        let ids = self.store.append(
            &w.actor,
            &w.batch,
            vec![(
                EventKind::GroupCreated,
                with_caption(json!({"kind": kind, "title": title.trim()}), w),
            )],
        )?;
        let id = format!("g{}", ids[0]);
        let out = self.outcome(ids, 0, format!("created group \"{}\"", title.trim()));
        Ok((out, id))
    }

    fn need_group(&self, group: &str) -> Result<&Group> {
        self.store
            .state()
            .groups
            .get(group)
            .ok_or_else(|| not_found(format!("no group {group}")))
    }

    /// Add (or re-role) a member.
    pub fn add_member(
        &mut self,
        group: &str,
        anchor: &Anchor,
        role: Option<&str>,
        w: &WriteCtx,
    ) -> Result<WriteOutcome> {
        let title = self.need_group(group)?.title.clone();
        self.need_line(anchor)?;
        if let Some(r) = role
            && !self.config().groups.roles.iter().any(|x| x == r)
        {
            return Err(usage(format!(
                "unknown role \"{r}\" (roles: {})",
                self.config().groups.roles.join(", ")
            )));
        }
        let ids = self.store.append(
            &w.actor,
            &w.batch,
            vec![(
                EventKind::GroupMemberAdded,
                with_caption(
                    json!({"group": group, "anchor": anchor.to_string(), "role": role}),
                    w,
                ),
            )],
        )?;
        Ok(self.outcome(ids, 1, format!("added 1 line to group \"{title}\"")))
    }

    pub fn remove_member(
        &mut self,
        group: &str,
        anchor: &Anchor,
        w: &WriteCtx,
    ) -> Result<WriteOutcome> {
        let g = self.need_group(group)?;
        let title = g.title.clone();
        if !g.members.iter().any(|m| &m.anchor == anchor) {
            return Err(not_found(format!("{anchor} is not in group {group}")));
        }
        let ids = self.store.append(
            &w.actor,
            &w.batch,
            vec![(
                EventKind::GroupMemberRemoved,
                with_caption(json!({"group": group, "anchor": anchor.to_string()}), w),
            )],
        )?;
        Ok(self.outcome(ids, 1, format!("removed 1 line from group \"{title}\"")))
    }

    pub fn set_group_status(
        &mut self,
        group: &str,
        status: &str,
        w: &WriteCtx,
    ) -> Result<WriteOutcome> {
        let title = self.need_group(group)?.title.clone();
        if !GROUP_STATUSES.contains(&status) {
            return Err(usage(format!(
                "unknown status \"{status}\" (statuses: {})",
                GROUP_STATUSES.join(", ")
            )));
        }
        let ids = self.store.append(
            &w.actor,
            &w.batch,
            vec![(
                EventKind::GroupStatus,
                with_caption(json!({"group": group, "status": status}), w),
            )],
        )?;
        Ok(self.outcome(ids, 0, format!("set group \"{title}\" to {status}")))
    }

    // ---- config ----

    /// Stage catalog/rule edits: the diff to preview (nothing written).
    pub fn stage_config(&self, edits: &[ConfigEdit]) -> Result<Staged> {
        self.config.stage(edits, &|i| self.used(i))
    }

    /// Write a staged change (refused with `conflict` if the file changed on disk) and record a
    /// `config_written` event `{before_hash, after_hash, summary, backup}`.
    pub fn write_config(&mut self, staged: &Staged, w: &WriteCtx) -> Result<WriteOutcome> {
        let backup = self.config.write(staged)?;
        let name = self.config.name();
        let payload = json!({
            "before_hash": staged.base_hash, "after_hash": staged.new_hash, "summary": staged.summary,
            "backup": backup.file_name().map(|s| s.to_string_lossy().into_owned()), "diff": staged.diff,
        });
        let ids = self.store.append(
            &w.actor,
            &w.batch,
            vec![(EventKind::ConfigWritten, with_caption(payload, w))],
        )?;
        let summary = format!("{} to {name}", staged.summary.join("; "));
        Ok(self.outcome(ids, 0, summary))
    }
}

/// Config issues as one error: a count as the message, every issue (`file:line: path: reason`) as a hint.
pub fn config_error(issues: Vec<ConfigIssue>) -> ClassifyError {
    let n = issues.len();
    let file = issues
        .first()
        .map_or_else(|| "classify.toml".to_string(), |i| i.file.clone());
    let mut e = ClassifyError::new(
        "config",
        format!("{file} has {n} {}.", super::config::plural(n, "problem")),
    );
    for i in issues {
        e = e.hint(i.to_string());
    }
    e
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::classify::config::tests::SAMPLE;
    use crate::classify::rules::tests::line;
    use crate::classify::store::pin_clock;

    const NOW: &str = "2026-10-05T09:46:00.000000Z";

    fn ws(name: &str) -> Workspace {
        pin_clock(Some(NOW));
        let d =
            std::env::temp_dir().join(format!("xa-classify-view-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(d.join("db")).unwrap();
        std::fs::write(d.join("classify.toml"), SAMPLE).unwrap();
        let config = ConfigFile::load(&d.join("classify.toml")).unwrap();
        let store = Store::open_lazy(&d.join("db/classify.sqlite")).unwrap();
        let lines = vec![
            line(
                "roast",
                "bt-1",
                "Hollowpine Packaging Works",
                "Bag proofs for HBL",
                "-1148.63",
                "5310",
                "2025-04-17",
            ),
            line(
                "roast",
                "bt-2",
                "Hollowpine Packaging Works",
                "Reprint",
                "-263.94",
                "5310",
                "2025-11-03",
            ),
            line(
                "design",
                "bt-3",
                "Marrowby Signs",
                "Wayfinding panels",
                "-2764.18",
                "6120",
                "2025-08-14",
            ),
            line(
                "lab",
                "bt-4",
                "大阪焙煎研究所",
                "Cupping spoons ☕",
                "-361.72",
                "5510",
                "2025-05-20",
            ),
            line(
                "hold",
                "bt-5",
                "Tallowmere Coffee",
                "Loan repayment",
                "3281.19",
                "1410",
                "2025-07-09",
            ),
        ];
        let charts = vec![(
            "roast".to_string(),
            vec![("5310".to_string(), "Packaging".to_string())],
        )];
        Workspace::from_parts(config, store, lines, charts)
    }

    fn human() -> WriteCtx {
        WriteCtx::new(Actor::human(), BatchId::new("h1"))
    }
    fn agent(b: &str) -> WriteCtx {
        WriteCtx::new(Actor::agent("claude"), BatchId::new(b)).caption("reclassify these")
    }
    fn a(s: &str) -> Anchor {
        let (org, id) = s.split_once('/').unwrap();
        Anchor::line(org, crate::classify::DocType::BankTransaction, id, "L1")
    }
    fn rule_ctx() -> WriteCtx {
        WriteCtx::new(Actor::rule("engine"), BatchId::new("s1"))
    }

    #[test]
    fn filter_parse_round_trip_and_errors() {
        let f = Filter::parse("org:roast acct:53 month:2025-09 project:harvest conf<0.7 by:agent since:today status:stale bags").unwrap();
        assert_eq!(f.orgs, vec!["roast"]);
        assert_eq!(
            f.labels,
            vec![("project".to_string(), "harvest".to_string())]
        );
        assert_eq!(f.conf_below.unwrap().to_string(), "0.7");
        assert_eq!(f.status, Status::Stale);
        assert_eq!(Filter::parse(&f.to_text()).unwrap(), f);
        for bad in [
            "month:2025-13",
            "conf<1.5",
            "since:yesterday",
            "status:done",
            "by:robot",
            "org:",
        ] {
            let e = Filter::parse(bad).unwrap_err();
            assert_eq!((e.code, e.exit_code()), ("filter", 2), "{bad}");
        }
    }

    #[test]
    fn queue_sorts_filters_and_suggests() {
        let mut w = ws("queue");
        let rows = w.queue(&Filter::default(), Sort::AmountDesc);
        let amounts: Vec<String> = rows.iter().map(|r| r.amount.to_string()).collect();
        assert_eq!(
            amounts,
            ["3281.19", "-2764.18", "-1148.63", "-361.72", "-263.94"]
        );
        assert!(rows.iter().all(|r| r.open && r.glyph == Glyph::None));
        let dates: Vec<String> = w
            .queue(&Filter::default(), Sort::DateAsc)
            .iter()
            .map(|r| r.date.clone())
            .collect();
        assert_eq!(dates[0], "2025-04-17");
        let (sum, out) = w.suggest(&Filter::default(), &rule_ctx()).unwrap();
        assert_eq!((sum.lines, sum.suggested, sum.events), (5, 3, 3));
        assert_eq!(
            sum.by_rule,
            vec![("r-alias".to_string(), 2), ("r-hollowpine".to_string(), 2)]
        );
        assert_eq!(out.events.len(), 3);
        // a second run writes nothing new
        let (sum2, _) = w.suggest(&Filter::default(), &rule_ctx()).unwrap();
        assert_eq!(sum2.events, 0);
        let rows = w.queue(&Filter::parse("org:roast").unwrap(), Sort::AmountDesc);
        assert_eq!(rows.len(), 2);
        let r = &rows[0];
        assert_eq!(r.glyph, Glyph::Rule);
        assert_eq!(
            r.suggestion.project.as_ref().unwrap().name,
            "Harvest Blend launch"
        );
        assert_eq!(r.confidence.unwrap().to_string(), "0.85");
        assert_eq!(
            w.queue(&Filter::parse("project:refit").unwrap(), Sort::AmountDesc)
                .len(),
            1
        );
        assert_eq!(
            w.queue(&Filter::parse("conf<0.9").unwrap(), Sort::AmountDesc)
                .len(),
            2
        );
        assert_eq!(
            w.queue(&Filter::parse("acct:55 cupping").unwrap(), Sort::AmountDesc)
                .len(),
            1
        );
        assert_eq!(
            w.queue(&Filter::parse("year:2024").unwrap(), Sort::AmountDesc)
                .len(),
            0
        );
        let d = w.detail(&a("roast/bt-1")).unwrap();
        assert_eq!(d.rules_fired, vec!["r-alias", "r-hollowpine"]);
        assert_eq!(
            d.labels[0].suggestion.as_ref().unwrap().rule.as_deref(),
            Some("r-alias")
        );
        assert_eq!(
            w.counts(),
            QueueCounts {
                total: 5,
                open: 5,
                stale: 0
            }
        );
    }

    #[test]
    fn accept_set_and_undo_line() {
        let mut w = ws("accept");
        w.suggest(&Filter::default(), &rule_ctx()).unwrap();
        let x = a("roast/bt-1");
        let out = w.accept(&x, &human()).unwrap();
        assert_eq!(
            out.summary,
            "accepted 2 labels on Hollowpine Packaging Works 1,148.63"
        );
        w.set_label(&x, "treatment", "keep", &human()).unwrap();
        let r = &w.queue(
            &Filter::parse("status:all org:roast").unwrap(),
            Sort::AmountDesc,
        )[0];
        assert!(!r.open);
        assert_eq!(r.glyph, Glyph::Human);
        let e = w.set_label(&x, "bearer", "rost", &human()).unwrap_err();
        assert!(
            e.message.contains("did you mean \"roast\""),
            "{}",
            e.message
        );
        assert!(
            w.set_label(&x, "should_be_account", "9999", &human())
                .is_err()
        );
        w.set_label(&x, "should_be_account", "5310", &human())
            .unwrap();
        let d = w.detail(&x).unwrap();
        assert_eq!(d.labels[3].label.as_ref().unwrap().name, "5310 Packaging");
        // u: the last write on the line, whoever made it
        w.undo_line(&x, &human()).unwrap();
        assert!(w.detail(&x).unwrap().labels[3].label.is_none());
        w.undo_line(&x, &human()).unwrap();
        assert!(
            w.queue(&Filter::default(), Sort::AmountDesc)
                .iter()
                .any(|r| r.anchor == x)
        );
        assert_eq!(w.store().verify().unwrap(), w.rev() as usize);
    }

    #[test]
    fn v15_undo_round_trip_per_write_kind() {
        let mut w = ws("v15");
        let x = a("lab/bt-4");
        let h = human();
        // label_set
        w.set_label(&x, "project", "none", &h).unwrap();
        let snap = |w: &Workspace| format!("{:?}", w.detail(&x).unwrap().labels);
        let before = snap(&w);
        w.set_label(&x, "project", "harvest", &h).unwrap();
        w.undo_line(&x, &h).unwrap();
        assert_eq!(snap(&w), before);
        // label_cleared
        w.clear_label(&x, "project", &h).unwrap();
        w.undo_line(&x, &h).unwrap();
        assert_eq!(snap(&w), before);
        // note_added
        w.add_note(&NoteTarget::Line(x.clone()), "spoons for the panel", &h)
            .unwrap();
        assert_eq!(w.detail(&x).unwrap().notes.len(), 1);
        w.undo_line(&x, &h).unwrap();
        assert!(w.detail(&x).unwrap().notes.is_empty());
        // group_created, member added/removed, status
        let (o, g) = w.create_group("split", "Cupping kit", &h).unwrap();
        w.add_member(&g, &x, Some("origin"), &h).unwrap();
        assert_eq!(
            w.detail(&x).unwrap().groups[0].role.as_deref(),
            Some("origin")
        );
        w.set_group_status(&g, "complete", &h).unwrap();
        let last = w.rev();
        w.undo_events(&[last], &h).unwrap();
        assert_eq!(w.groups()[0].status, "open");
        w.remove_member(&g, &x, &h).unwrap();
        assert!(w.detail(&x).unwrap().groups.is_empty());
        w.undo_line(&x, &h).unwrap();
        assert_eq!(w.detail(&x).unwrap().groups.len(), 1);
        w.undo_events(&o.events, &h).unwrap();
        assert!(w.groups().is_empty());
        // suggestion
        let (_, s) = w.suggest(&Filter::default(), &rule_ctx()).unwrap();
        w.undo_events(&s.events, &h).unwrap();
        assert!(
            w.queue(&Filter::default(), Sort::AmountDesc)
                .iter()
                .all(|r| r.confidence.is_none())
        );
        // config_written
        let text = w.config_file().text.clone();
        let st = w
            .stage_config(&[ConfigEdit::AddChoice {
                label_type: "project".into(),
                choice: Choice {
                    id: "fitout".into(),
                    name: "Cupping room fit-out".into(),
                    aliases: vec![],
                    archived: false,
                },
            }])
            .unwrap();
        let out = w.write_config(&st, &agent("c1")).unwrap();
        assert_eq!(
            out.summary,
            "added project \"Cupping room fit-out\" to classify.toml"
        );
        assert_eq!(w.choices("project").len(), 4);
        let u = w.undo_batch(&h).unwrap();
        assert!(u.summary.starts_with("agent write undone"));
        assert_eq!(w.config_file().text, text);
        assert_eq!(
            std::fs::read_to_string(&w.config_file().path).unwrap(),
            text
        );
        // undo of an undo
        let undo_id = w.rev();
        w.undo_events(&[undo_id], &h).unwrap();
        assert_eq!(snap(&w), before);
        assert!(w.undo_events(&[undo_id], &h).is_err(), "already undone");
    }

    #[test]
    fn agent_batches_bulk_and_limits() {
        let mut w = ws("bulk");
        w.suggest(&Filter::default(), &rule_ctx()).unwrap();
        let p = w.bulk_preview(&Filter::default(), "0.9".parse().unwrap());
        assert!(
            p.anchors.is_empty(),
            "bearer/treatment missing: nothing to bulk-accept"
        );
        // make treatment suggestions exist by labelling? Use a lower requirement: queue.required = project only
        let text = SAMPLE.replace("[groups]", "[queue]\nrequired = [\"project\"]\n\n[groups]");
        std::fs::write(&w.config_file().path, text).unwrap();
        assert!(matches!(w.reload_config(), Reload::Reloaded { .. }));
        let p = w.bulk_preview(&Filter::default(), "0.9".parse().unwrap());
        assert_eq!(
            p.anchors,
            vec![a("roast/bt-1"), a("design/bt-3")],
            "0.85 lines stay out"
        );
        let p = w.bulk_preview(&Filter::default(), "0.8".parse().unwrap());
        assert_eq!(p.anchors.len(), 3);
        assert_eq!(p.total.to_string(), "-4176.75");
        assert_eq!(
            p.by_project,
            vec![("harvest".to_string(), 2), ("refit".to_string(), 1)]
        );
        // a stale line never bulk-accepts
        w.set_label(&a("roast/bt-2"), "bearer", "hold", &human())
            .unwrap();
        w.lines[1].fingerprint.amount = "-264.13".into();
        w.invalidate_cache();
        let p = w.bulk_preview(&Filter::default(), "0.8".parse().unwrap());
        assert_eq!(p.anchors.len(), 2);
        let out = w
            .bulk_accept(&Filter::default(), "0.8".parse().unwrap(), &agent("b1"))
            .unwrap();
        assert_eq!(out.lines, 2);
        let rows = w.queue(
            &Filter::parse("by:agent since:today status:all").unwrap(),
            Sort::AmountDesc,
        );
        assert_eq!(rows.len(), 2);
        assert!(
            rows.iter()
                .all(|r| r.glyph == Glyph::Agent || r.glyph == Glyph::Rule)
        );
        let d = w.detail(&a("design/bt-3")).unwrap();
        let l = d.labels[0].label.as_ref().unwrap();
        assert_eq!(
            (l.source.to_string().as_str(), l.caption.as_deref()),
            ("agent:claude", Some("reclassify these"))
        );
        w.undo_batch(&human()).unwrap();
        assert!(
            w.queue(
                &Filter::parse("by:agent status:all").unwrap(),
                Sort::AmountDesc
            )
            .is_empty()
        );
        assert_eq!(w.undo_batch(&human()).unwrap_err().code, "not_found");
        // too_many: 51 anchors in one agent send
        let many: Vec<Anchor> = (0..51)
            .map(|i| {
                Anchor::line(
                    "lab",
                    crate::classify::DocType::BankTransaction,
                    &format!("bt-{i}"),
                    "L1",
                )
            })
            .collect();
        let e = w
            .set_labels(&many, "project", "none", &agent("b2"))
            .unwrap_err();
        assert_eq!((e.code, e.exit_code()), ("too_many", 2));
        assert!(e.message.contains("51 lines"));
        let mut ok = agent("b3");
        ok.allow_bulk = true;
        // past the limit: the 51 invented anchors are then refused as unknown lines
        assert_eq!(
            w.set_labels(&many, "project", "none", &ok)
                .unwrap_err()
                .code,
            "not_found"
        );
        let dup: Vec<Anchor> = (0..60).map(|_| a("lab/bt-4")).collect();
        assert_eq!(
            w.set_labels(&dup, "project", "none", &agent("b4"))
                .unwrap()
                .lines,
            1
        );
        assert_eq!(w.expect_rev(w.rev() - 1).unwrap_err().code, "view_changed");
    }

    #[test]
    fn scope_counts_dry_run_and_lazy_store() {
        let mut w = ws("scope");
        assert!(!w.store().exists() || w.rev() == 0);
        let text = SAMPLE.replace(
            "[groups]",
            "[queue]\nscope = \"org:roast,design\"\n\n[groups]",
        );
        std::fs::write(&w.config_file().path, text).unwrap();
        assert!(matches!(w.reload_config(), Reload::Reloaded { .. }));
        assert_eq!(w.scope().unwrap().orgs, vec!["roast", "design"]);
        assert_eq!(
            w.counts(),
            QueueCounts {
                total: 3,
                open: 3,
                stale: 0
            }
        );
        assert_eq!(w.queue(&Filter::default(), Sort::AmountDesc).len(), 3);
        assert_eq!(
            w.queue(&Filter::parse("org:lab").unwrap(), Sort::AmountDesc)
                .len(),
            0
        );
        let c = w.counts_for(&Filter::parse("org:roast status:stale").unwrap());
        assert_eq!((c.total, c.open), (0, 0), "status filters the counts too");
        let c = w.counts_for(&Filter::parse("org:roast").unwrap());
        assert_eq!((c.total, c.open), (2, 2));
        let dry = w.suggest_dry_run(&Filter::default());
        assert!(dry.dry_run);
        assert_eq!((dry.lines, dry.suggested), (3, 3));
        assert_eq!(dry.rules[0].rule, "r-alias");
        assert_eq!(dry.rules[0].labels, vec![("project".to_string(), 2)]);
        assert_eq!(dry.rules[1].rule, "r-hollowpine");
        assert_eq!((dry.rules[1].lines, dry.rules[1].labels.len()), (2, 2));
        assert_eq!(w.rev(), 0, "a dry run appends nothing");
        let (sum, _) = w.suggest(&Filter::default(), &rule_ctx()).unwrap();
        assert_eq!(sum.events, 3, "lab and hold are out of scope");
        let bad = SAMPLE.replace("[groups]", "[queue]\nscope = \"month:2025-13\"\n\n[groups]");
        let issues = crate::classify::config::parse(&bad, "c").unwrap_err();
        assert!(
            issues[0].to_string().contains("queue.scope"),
            "{}",
            issues[0]
        );
    }

    #[test]
    fn lazy_store_is_created_by_the_first_write_only() {
        let d = std::env::temp_dir().join(format!("xa-classify-lazy-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        let p = d.join("db/classify.sqlite");
        let mut s = Store::open_lazy(&p).unwrap();
        assert!(!s.exists() && !d.exists());
        assert_eq!((s.rev(), s.verify().unwrap()), (0, 0));
        assert!(!s.refresh().unwrap());
        s.append(
            &Actor::human(),
            &BatchId::new("b"),
            vec![(
                EventKind::GroupCreated,
                json!({"kind": "split", "title": "t"}),
            )],
        )
        .unwrap();
        assert!(p.exists() && s.exists());
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn incremental_rows_equal_a_full_rebuild() {
        let mut w = ws("cache");
        let all = Filter::parse("status:all").unwrap();
        let snap = |w: &mut Workspace| {
            let inc = (w.queue(&all, Sort::AmountDesc), w.counts());
            w.invalidate_cache();
            let full = (w.queue(&all, Sort::AmountDesc), w.counts());
            assert_eq!(inc, full);
        };
        snap(&mut w);
        w.suggest(&all, &rule_ctx()).unwrap();
        snap(&mut w);
        w.accept(&a("roast/bt-1"), &human()).unwrap();
        snap(&mut w);
        w.set_labels(
            &[a("lab/bt-4"), a("hold/bt-5")],
            "treatment",
            "keep",
            &agent("c1"),
        )
        .unwrap();
        snap(&mut w);
        w.add_note(&NoteTarget::Line(a("lab/bt-4")), "n", &human())
            .unwrap();
        w.clear_label(&a("lab/bt-4"), "treatment", &human())
            .unwrap();
        snap(&mut w);
        w.undo_batch(&human()).unwrap();
        snap(&mut w);
        let text = SAMPLE.replace("[groups]", "[queue]\nscope = \"org:roast,lab\"\n\n[groups]");
        std::fs::write(&w.config_file().path, text).unwrap();
        w.reload_config();
        snap(&mut w);
        assert_eq!(w.counts().total, 3);
    }

    #[test]
    fn v19_stale_keeps_the_old_value() {
        let mut w = ws("stale");
        let x = a("roast/bt-2");
        w.set_label(&x, "project", "harvest", &human()).unwrap();
        w.lines[1].amount = "-264.13".parse().unwrap();
        w.lines[1].fingerprint.amount = "-264.13".into();
        w.invalidate_cache();
        let r = w.queue(&Filter::parse("status:stale").unwrap(), Sort::AmountDesc);
        assert_eq!(r.len(), 1);
        assert!(r[0].stale && r[0].open);
        let l = w.detail(&x).unwrap().labels[0].label.clone().unwrap();
        assert!(l.stale);
        assert_eq!(l.value, "harvest");
        assert_eq!(
            l.changed,
            vec![("amount", "-263.94".to_string(), "-264.13".to_string())]
        );
        assert_eq!(w.counts().stale, 1);
    }

    #[test]
    fn v20_used_choice_archives_and_would_match() {
        let mut w = ws("v20");
        w.set_label(&a("lab/bt-4"), "project", "none", &human())
            .unwrap();
        assert_eq!(
            w.used(&ItemRef::Choice {
                label_type: "project".into(),
                id: "none".into()
            }),
            1
        );
        let st = w
            .stage_config(&[ConfigEdit::RemoveChoice {
                label_type: "project".into(),
                id: "none".into(),
            }])
            .unwrap();
        assert_eq!(st.archived_instead.len(), 1);
        w.write_config(&st, &human()).unwrap();
        assert!(
            w.choices("project").iter().all(|c| c.id != "none"),
            "hidden from pickers"
        );
        assert_eq!(
            w.detail(&a("lab/bt-4")).unwrap().labels[0]
                .label
                .as_ref()
                .unwrap()
                .name,
            "No project"
        );
        let wm = w.would_match(w.config().rule("r-hollowpine").unwrap());
        assert_eq!((wm.count, wm.sample.len()), (2, 2));
    }
}

#[cfg(test)]
mod paths_tests {
    use super::*;

    #[test]
    fn paths_come_from_ownpurse_json_with_defaults() {
        let d = std::env::temp_dir().join(format!("xa-classify-paths-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        let p = d.join("ownpurse.json");
        std::fs::write(
            &p,
            r#"{"default_profile": "p", "profiles": {"p": {"client_id": "x"}}}"#,
        )
        .unwrap();
        let env = crate::config::Env::process();
        let c = crate::config::load(&env, None, Some(&p)).unwrap();
        let got = ClassifyPaths::resolve(&c);
        assert_eq!(got.config, d.join("classify.toml"));
        assert_eq!(got.store, d.join("db/classify.sqlite"));
        std::fs::write(
            &p,
            r#"{"default_profile": "p", "classify": {"store": "state/labels.sqlite"},
                "profiles": {"p": {"client_id": "x", "classify": {"config": "cfg/labels.toml"}}}}"#,
        )
        .unwrap();
        let c = crate::config::load(&env, None, Some(&p)).unwrap();
        let got = ClassifyPaths::resolve(&c);
        assert_eq!(got.config, d.join("cfg/labels.toml"));
        assert_eq!(got.store, d.join("state/labels.sqlite"));
        let _ = std::fs::remove_dir_all(&d);
    }
}
