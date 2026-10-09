//! Tab 6 Classify (spec classify §1, §3.A): the mode sub-bar and the Queue. The data comes from
//! `core::classify::view` through [`Provider`] (the core's `Workspace`). Reads happen in `update`/draw from cached rows; every write is an `Effect::Classify` the loop
//! runs against the store, then `Msg::Classified` reloads the rows (V14: the store is the only thing written).
//! The TUI never writes Xero and never links the api crate (V1).

pub mod catalog;
pub mod groups;

use crate::app::{App, Effect, Msg, Scope, Tab};
use ownpurse_core::classify::config::ClassifyConfig;
use ownpurse_core::classify::request::{ApplyOutcome, ClassifyRequest};
use ownpurse_core::classify::{
    Actor, Anchor, BatchId, BulkPreview, Confidence, Filter, LineDetail, QueueCounts, QueueRow,
    Result as CResult, Sort, Workspace, WriteCtx, WriteOutcome,
};
use rust_decimal::Decimal;
use std::cell::RefCell;
use std::rc::Rc;

/// What the TUI needs from the classification layer (implemented by the core's `Workspace`).
pub trait Provider {
    fn config(&self) -> &ClassifyConfig;
    /// The store revision (a write moves it; caches key on it).
    fn rev(&self) -> i64;
    fn parse_filter(&self, text: &str) -> CResult<Filter>;
    fn counts(&self) -> QueueCounts;
    /// Counts inside the scope for a filter (its status ignored): total, open, stale.
    fn counts_for(&self, f: &Filter) -> QueueCounts;
    fn queue(&self, f: &Filter, sort: Sort) -> Vec<QueueRow>;
    fn detail(&self, a: &Anchor) -> Option<LineDetail>;
    fn account_choices(&self, org: &str) -> Vec<(String, String)>;
    fn bulk_preview(&self, f: &Filter, threshold: Confidence) -> BulkPreview;
    fn accept(&mut self, a: &Anchor, w: &WriteCtx) -> CResult<WriteOutcome>;
    fn set_label(&mut self, a: &Anchor, ty: &str, v: &str, w: &WriteCtx) -> CResult<WriteOutcome>;
    fn add_note(&mut self, a: &Anchor, text: &str, w: &WriteCtx) -> CResult<WriteOutcome>;
    fn undo_line(&mut self, a: &Anchor, w: &WriteCtx) -> CResult<WriteOutcome>;
    fn bulk_accept(&mut self, f: &Filter, t: Confidence, w: &WriteCtx) -> CResult<WriteOutcome>;
    /// Pick up events other writers appended (an agent, the CLI); true if anything changed.
    fn refresh(&mut self) -> bool;
    /// Groups worth offering for a line (`G`): those with a member in the same org within 31 days, by id.
    fn groups_near(&self, a: &Anchor) -> Vec<(String, String)>;
    fn add_to_group(&mut self, group: &str, a: &Anchor, w: &WriteCtx) -> CResult<WriteOutcome>;
    /// Every group, by id.
    fn groups_list(&self) -> Vec<groups::GroupRow>;
    fn group_detail(&self, id: &str) -> Option<ownpurse_core::classify::groups::GroupDetail>;
    fn suggest_members(&self, a: &Anchor)
    -> Vec<ownpurse_core::classify::groups::MemberSuggestion>;
    fn pair_check(
        &self,
        id: &str,
        record: &ownpurse_core::view::View<'_>,
        orgs: &[(String, String)],
    ) -> Option<ownpurse_core::classify::groups::PairCheck>;
    /// Add a member, or change its role.
    fn group_member(
        &mut self,
        group: &str,
        a: &Anchor,
        role: Option<&str>,
        w: &WriteCtx,
    ) -> CResult<WriteOutcome>;
    fn group_remove(&mut self, group: &str, a: &Anchor, w: &WriteCtx) -> CResult<WriteOutcome>;
    fn group_status(&mut self, group: &str, status: &str, w: &WriteCtx) -> CResult<WriteOutcome>;
    fn group_note(&mut self, group: &str, text: &str, w: &WriteCtx) -> CResult<WriteOutcome>;
    /// A new group of `kind` titled `title`, with the line as its first member.
    fn new_group(
        &mut self,
        kind: &str,
        title: &str,
        a: &Anchor,
        w: &WriteCtx,
    ) -> CResult<WriteOutcome>;
    /// The live config's hash (a hand edit or a write changes it; caches key on it).
    fn config_hash(&self) -> String;
    /// Lines and groups referencing an item (the Catalog's Used column).
    fn used(&self, item: &ownpurse_core::classify::config::ItemRef) -> usize;
    /// What a rule would match in the record today: the count, 5 samples, and how many of those lines
    /// already carry a label a person set (kept: rules never overwrite them).
    fn would_match(&self, rule: &ownpurse_core::classify::config::Rule) -> Matched;
    /// Lines whose CURRENT label came from this rule (accepted from its suggestion).
    fn labelled(&self, rule_id: &str) -> usize;
    fn stage(
        &self,
        edits: &[ownpurse_core::classify::config::ConfigEdit],
    ) -> CResult<ownpurse_core::classify::config::Staged>;
    fn write_config(
        &mut self,
        staged: &ownpurse_core::classify::config::Staged,
        w: &WriteCtx,
    ) -> CResult<WriteOutcome>;
    /// Re-read `classify.toml` after a hand edit (the last valid config stays live on error).
    fn reload_config(&mut self) -> ownpurse_core::classify::config::Reload;
    /// The lines an agent request would touch (checked for `user_busy` before it is applied).
    fn resolve_targets(&self, req: &ClassifyRequest) -> CResult<Vec<Anchor>>;
    /// Apply an agent request: the core's single write path (`actor = agent:<client>`, one batch).
    fn apply(&mut self, req: &ClassifyRequest) -> CResult<ApplyOutcome>;
    /// A group's title (notices name groups by title).
    fn group_title(&self, id: &str) -> Option<String>;
    /// `U`: the most recent agent batch not yet undone, as one undo event attributed to the user.
    fn undo_batch(&mut self, w: &WriteCtx) -> CResult<WriteOutcome>;
    /// Activity's "Undo this write?": these events.
    fn undo_events(&mut self, ids: &[i64], w: &WriteCtx) -> CResult<WriteOutcome>;
}

impl Provider for Workspace {
    fn config(&self) -> &ClassifyConfig {
        Workspace::config(self)
    }
    fn rev(&self) -> i64 {
        Workspace::rev(self)
    }
    fn parse_filter(&self, text: &str) -> CResult<Filter> {
        Filter::parse(text)
    }
    fn counts(&self) -> QueueCounts {
        Workspace::counts(self)
    }
    fn counts_for(&self, f: &Filter) -> QueueCounts {
        Workspace::counts_for(self, f)
    }
    fn queue(&self, f: &Filter, sort: Sort) -> Vec<QueueRow> {
        Workspace::queue(self, f, sort)
    }
    fn detail(&self, a: &Anchor) -> Option<LineDetail> {
        Workspace::detail(self, a)
    }
    fn account_choices(&self, org: &str) -> Vec<(String, String)> {
        Workspace::account_choices(self, org).to_vec()
    }
    fn bulk_preview(&self, f: &Filter, t: Confidence) -> BulkPreview {
        Workspace::bulk_preview(self, f, t)
    }
    fn accept(&mut self, a: &Anchor, w: &WriteCtx) -> CResult<WriteOutcome> {
        Workspace::accept(self, a, w)
    }
    fn set_label(&mut self, a: &Anchor, ty: &str, v: &str, w: &WriteCtx) -> CResult<WriteOutcome> {
        Workspace::set_label(self, a, ty, v, w)
    }
    fn add_note(&mut self, a: &Anchor, text: &str, w: &WriteCtx) -> CResult<WriteOutcome> {
        Workspace::add_note(
            self,
            &ownpurse_core::classify::store::NoteTarget::Line(a.clone()),
            text,
            w,
        )
    }
    fn undo_line(&mut self, a: &Anchor, w: &WriteCtx) -> CResult<WriteOutcome> {
        Workspace::undo_line(self, a, w)
    }
    fn bulk_accept(&mut self, f: &Filter, t: Confidence, w: &WriteCtx) -> CResult<WriteOutcome> {
        Workspace::bulk_accept(self, f, t, w)
    }
    fn refresh(&mut self) -> bool {
        Workspace::refresh(self).unwrap_or(false)
    }
    fn groups_near(&self, a: &Anchor) -> Vec<(String, String)> {
        let Some(line) = Workspace::line(self, a) else {
            return Vec::new();
        };
        let day = |d: &str| {
            ownpurse_core::dates::parse_iso(d)
                .map(|(y, m, dd)| ownpurse_core::dates::days_from_civil(y, m, dd))
        };
        let here = day(&line.date);
        let near = |m: &Anchor| {
            Workspace::line(self, m).is_some_and(|l| {
                l.org == line.org
                    && match (here, day(&l.date)) {
                        (Some(x), Some(y)) => (x - y).abs() <= 31,
                        _ => false,
                    }
            })
        };
        Workspace::groups(self)
            .into_iter()
            .filter(|g| g.members.iter().any(|m| near(&m.anchor)))
            .map(|g| {
                let n = g.members.len();
                (
                    g.id.clone(),
                    format!(
                        "{} · {} · {n} line{}",
                        g.title,
                        g.kind.replace('_', " "),
                        if n == 1 { "" } else { "s" }
                    ),
                )
            })
            .collect()
    }
    fn groups_list(&self) -> Vec<groups::GroupRow> {
        Workspace::groups(self)
            .into_iter()
            .map(|g| groups::GroupRow {
                id: g.id.clone(),
                kind: g.kind.clone(),
                title: g.title.clone(),
                status: g.status.clone(),
                members: g.members.len(),
            })
            .collect()
    }
    fn group_detail(&self, id: &str) -> Option<ownpurse_core::classify::groups::GroupDetail> {
        Workspace::group_detail(self, id)
    }
    fn suggest_members(
        &self,
        a: &Anchor,
    ) -> Vec<ownpurse_core::classify::groups::MemberSuggestion> {
        Workspace::suggest_members(self, a)
    }
    fn pair_check(
        &self,
        id: &str,
        record: &ownpurse_core::view::View<'_>,
        orgs: &[(String, String)],
    ) -> Option<ownpurse_core::classify::groups::PairCheck> {
        Workspace::pair_check(self, id, record, orgs).ok().flatten()
    }
    fn group_member(
        &mut self,
        group: &str,
        a: &Anchor,
        role: Option<&str>,
        w: &WriteCtx,
    ) -> CResult<WriteOutcome> {
        Workspace::add_member(self, group, a, role, w)
    }
    fn group_remove(&mut self, group: &str, a: &Anchor, w: &WriteCtx) -> CResult<WriteOutcome> {
        Workspace::remove_member(self, group, a, w)
    }
    fn group_status(&mut self, group: &str, status: &str, w: &WriteCtx) -> CResult<WriteOutcome> {
        Workspace::set_group_status(self, group, status, w)
    }
    fn group_note(&mut self, group: &str, text: &str, w: &WriteCtx) -> CResult<WriteOutcome> {
        Workspace::add_note(
            self,
            &ownpurse_core::classify::store::NoteTarget::Group(group.to_string()),
            text,
            w,
        )
    }
    fn add_to_group(&mut self, group: &str, a: &Anchor, w: &WriteCtx) -> CResult<WriteOutcome> {
        Workspace::add_member(self, group, a, None, w)
    }
    fn new_group(
        &mut self,
        kind: &str,
        title: &str,
        a: &Anchor,
        w: &WriteCtx,
    ) -> CResult<WriteOutcome> {
        let (mut first, id) = Workspace::create_group(self, kind, title, w)?;
        let second = Workspace::add_member(self, &id, a, None, w)?;
        first.events.extend(second.events);
        first.rev = second.rev;
        first.summary = format!("added the line to a new {kind} group \"{title}\"");
        Ok(first)
    }
    fn config_hash(&self) -> String {
        Workspace::config_file(self).hash.clone()
    }
    fn used(&self, item: &ownpurse_core::classify::config::ItemRef) -> usize {
        Workspace::used(self, item)
    }
    fn would_match(&self, rule: &ownpurse_core::classify::config::Rule) -> Matched {
        use ownpurse_core::classify::{Source, rules};
        let cfg = Workspace::config(self);
        let labels = &Workspace::store(self).state().labels;
        let mut m = Matched::default();
        for l in Workspace::lines(self) {
            if rules::matches(cfg, rule, l).is_none() {
                continue;
            }
            m.count += 1;
            if m.sample.len() < 5 {
                m.sample.push(l.anchor.clone());
            }
            let by_person = cfg.label_types.iter().any(|t| {
                labels
                    .get(&(l.anchor.clone(), t.id.clone()))
                    .is_some_and(|x| x.source == Source::Human)
            });
            m.kept += usize::from(by_person);
        }
        m
    }
    fn labelled(&self, rule_id: &str) -> usize {
        let src = ownpurse_core::classify::Source::Rule(rule_id.to_string());
        let anchors: std::collections::HashSet<&Anchor> = Workspace::store(self)
            .state()
            .labels
            .iter()
            .filter(|(_, l)| l.source == src)
            .map(|((a, _), _)| a)
            .collect();
        anchors.len()
    }
    fn stage(
        &self,
        edits: &[ownpurse_core::classify::config::ConfigEdit],
    ) -> CResult<ownpurse_core::classify::config::Staged> {
        Workspace::stage_config(self, edits)
    }
    fn write_config(
        &mut self,
        staged: &ownpurse_core::classify::config::Staged,
        w: &WriteCtx,
    ) -> CResult<WriteOutcome> {
        Workspace::write_config(self, staged, w)
    }
    fn reload_config(&mut self) -> ownpurse_core::classify::config::Reload {
        Workspace::reload_config(self)
    }
    fn resolve_targets(&self, req: &ClassifyRequest) -> CResult<Vec<Anchor>> {
        Workspace::resolve_targets(self, req)
    }
    fn apply(&mut self, req: &ClassifyRequest) -> CResult<ApplyOutcome> {
        Workspace::apply(self, req)
    }
    fn undo_batch(&mut self, w: &WriteCtx) -> CResult<WriteOutcome> {
        Workspace::undo_batch(self, w)
    }
    fn group_title(&self, id: &str) -> Option<String> {
        Workspace::groups(self)
            .into_iter()
            .find(|g| g.id == id)
            .map(|g| g.title.clone())
    }
    fn undo_events(&mut self, ids: &[i64], w: &WriteCtx) -> CResult<WriteOutcome> {
        Workspace::undo_events(self, ids, w)
    }
}

/// The `G` picker's first row: start a new group (then choose its kind).
pub const NEW_GROUP: &str = "+new";

/// The five modes (sub-bar order).
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize,
)]
pub enum Mode {
    #[default]
    Queue,
    Groups,
    Audition,
    Tree,
    Catalog,
}

impl Mode {
    pub const ALL: [Mode; 5] = [
        Mode::Queue,
        Mode::Groups,
        Mode::Audition,
        Mode::Tree,
        Mode::Catalog,
    ];
    pub fn label(self) -> &'static str {
        match self {
            Mode::Queue => "Queue",
            Mode::Groups => "Groups",
            Mode::Audition => "Audition",
            Mode::Tree => "Tree",
            Mode::Catalog => "Catalog",
        }
    }
    pub fn word(self) -> &'static str {
        match self {
            Mode::Queue => "queue",
            Mode::Groups => "groups",
            Mode::Audition => "audition",
            Mode::Tree => "tree",
            Mode::Catalog => "catalog",
        }
    }
    fn index(self) -> usize {
        Mode::ALL.iter().position(|m| *m == self).unwrap_or(0)
    }
}

/// The `b` / `t` / `s` picker: a label type's choices (or the line's org chart), typed to filter.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct LabelPicker {
    pub label_type: String,
    pub query: String,
    pub sel: usize,
}

/// The `n` note editor (one line; `⌃E` multi-line).
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct NoteEdit {
    pub text: String,
    pub multi: bool,
}

/// The `A` bulk-accept preview: threshold in hundredths, the focused button (0 = Cancel).
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BulkUi {
    pub threshold: u8,
    pub focus: usize,
}

/// The Classify tab's view state (canonical: `ui.classify`, V21). The cursor is the selected line's anchor
/// (navigation rule 4). `audition_set` is reserved for the Audition mode.
#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct ClassifyUi {
    pub mode: Mode,
    pub filter: String,
    pub selected_anchor: Option<String>,
    pub audition_set: Option<String>,
    /// The list's first drawn row.
    pub top: usize,
    /// `p` was pressed: the next digit picks a project.
    pub pending_project: bool,
    pub picker: Option<LabelPicker>,
    pub note: Option<NoteEdit>,
    pub bulk: Option<BulkUi>,
    /// The line the last write went to, until the cursor moves: `u` undoes there (accept moves on).
    pub last_written: Option<String>,
    /// The Catalog mode: its lists, the staged edits, the open editor or diff.
    pub catalog: catalog::CatalogUi,
    /// The Groups mode: the selected group and member, focus, an open member document.
    pub groups: groups::GroupsUi,
}

impl ClassifyUi {
    /// The parts history keeps (editors closed).
    pub fn settled(&self) -> ClassifyUi {
        ClassifyUi {
            pending_project: false,
            picker: None,
            note: None,
            bulk: None,
            last_written: None,
            catalog: catalog::CatalogUi {
                editor: None,
                diff: None,
                toast: None,
                ..self.catalog.clone()
            },
            ..self.clone()
        }
    }
    /// An editor, picker or preview is open (agent sends are refused `user_busy`).
    pub fn busy(&self) -> Option<&'static str> {
        if self.picker.is_some() {
            Some("in a Classify picker")
        } else if self.note.is_some() {
            Some("writing a note")
        } else if self.bulk.is_some() {
            Some("in the bulk-accept preview")
        } else if self.pending_project {
            Some("choosing a project")
        } else {
            self.catalog_busy()
        }
    }
}

/// One write, run by the loop (`Effect::Classify`), attributed (`actor`) and batched (one key = one batch).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Write {
    pub op: Op,
    pub actor: String,
    pub batch: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Op {
    Accept(String),
    SetLabel {
        anchor: String,
        label_type: String,
        value: String,
    },
    Note(String, String),
    Undo(String),
    AddToGroup {
        anchor: String,
        group: String,
    },
    NewGroup {
        anchor: String,
        kind: String,
        title: String,
    },
    /// Write the Catalog's staged edits to `classify.toml` (diff confirmed first).
    Config(Vec<catalog::CatEdit>),
    /// Groups: add a member, or change its role.
    GroupMember {
        group: String,
        anchor: String,
        role: Option<String>,
    },
    GroupRemove {
        group: String,
        anchor: String,
    },
    GroupStatus {
        group: String,
        status: String,
    },
    GroupNote {
        group: String,
        text: String,
    },
    /// An agent's request (bridge `kind: "classify"`; its actor and batch come from the request).
    Agent(Box<ClassifyRequest>),
    /// `U`: undo the latest agent batch (attributed to the user).
    UndoAgentBatch,
    /// Activity's confirm: undo these events of an older agent write.
    UndoEvents(Vec<i64>),
    /// Bulk-accept the filter (with the scope's orgs) at a threshold in hundredths.
    Bulk {
        filter: String,
        orgs: Vec<String>,
        threshold: u8,
    },
}

type RowsKey = (i64, String, String, String);
type MatchKey = (String, String);
type Matches = (i64, std::collections::HashMap<MatchKey, Matched>);

/// A rule's preview (see [`Provider::would_match`]).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Matched {
    pub count: usize,
    pub sample: Vec<Anchor>,
    /// Matched lines that already carry a label a person set (kept).
    pub kept: usize,
}

/// The opened classification layer and its derived caches (rows per store revision, filter and scope).
pub struct Data {
    pub provider: Box<dyn Provider>,
    rows: RefCell<Option<(RowsKey, Rows)>>,
    counts: RefCell<Option<(i64, QueueCounts)>>,
    /// Rule previews ("would match N"), per (config hash, rule) at one store revision.
    matches: RefCell<Matches>,
    /// The config changed (hand edit or write): the counts are recomputed once.
    pub config_moved: std::cell::Cell<bool>,
    /// The record (read-only), for the Groups pair rule's ledger balances.
    record: Option<ownpurse_core::Record>,
    /// Pair checks per (store rev, group).
    pairs: RefCell<
        std::collections::HashMap<
            (i64, String),
            Option<ownpurse_core::classify::groups::PairCheck>,
        >,
    >,
}

/// The Queue's rows and the parsed filter (or why it doesn't parse).
#[derive(Clone)]
pub struct Rows {
    pub rows: Rc<Vec<QueueRow>>,
    pub error: Option<String>,
    /// The scope's open lines before the filter (the strip's "of N").
    pub total: usize,
    /// The widest amount as shown (the list's amount column).
    pub amount_w: usize,
    /// Every listed line is in one year (the list may show dates as `MM-DD`).
    pub one_year: bool,
}

impl Data {
    pub fn new(provider: Box<dyn Provider>) -> Data {
        Data {
            provider,
            rows: RefCell::new(None),
            counts: RefCell::new(None),
            matches: RefCell::new((i64::MIN, Default::default())),
            config_moved: std::cell::Cell::new(false),
            record: None,
            pairs: RefCell::new(Default::default()),
        }
    }

    /// A group's pair-rule check on the record's balances (cached per store revision).
    pub fn pair_check(
        &self,
        id: &str,
        orgs: &[(String, String)],
    ) -> Option<ownpurse_core::classify::groups::PairCheck> {
        let rec = self.record.as_ref()?;
        let key = (self.provider.rev(), id.to_string());
        if let Some(v) = self.pairs.borrow().get(&key) {
            return v.clone();
        }
        let v = self
            .provider
            .pair_check(id, &ownpurse_core::view::View::main(rec), orgs);
        self.pairs.borrow_mut().insert(key, v.clone());
        v
    }

    /// A rule's preview, cached per config hash and store revision (the Catalog draws every rule's count).
    pub fn cached_would_match(&self, rule: &ownpurse_core::classify::config::Rule) -> Matched {
        let rev = self.provider.rev();
        let key = (self.provider.config_hash(), format!("{rule:?}"));
        let mut m = self.matches.borrow_mut();
        if m.0 != rev {
            *m = (rev, Default::default());
        }
        if let Some(v) = m.1.get(&key) {
            return v.clone();
        }
        let v = self.provider.would_match(rule);
        m.1.insert(key, v.clone());
        v
    }
}

/// Open the classification layer for a source: the core's workspace when `classify.toml` exists (it reads
/// the record, never writes it). None: not set up (the tab says how).
pub fn open(source: &crate::model::Source) -> Option<Data> {
    let paths = source.classify.as_ref()?;
    if !paths.config.exists() {
        return None;
    }
    let record = ownpurse_core::Record::open(&source.record).ok()?;
    let w = Workspace::open(
        paths,
        &ownpurse_core::view::View::main(&record),
        &source.orgs,
    )
    .ok()?;
    let mut d = Data::new(Box::new(w));
    d.record = Some(record);
    Some(d)
}

/// Exact `0.xx` from hundredths.
pub fn hundredths(n: u8) -> Decimal {
    Decimal::new(i64::from(n), 2)
}

/// A confidence as the list shows it: `.84`, `1.0`.
pub fn conf_text(c: Confidence) -> String {
    let r = c.round_dp(2);
    if r >= Decimal::ONE {
        "1.0".into()
    } else {
        let s = format!("{r:.2}");
        s.trim_start_matches('0').to_string()
    }
}

/// The 3-cell bar: ≥ .90 three, ≥ .70 two, else one.
pub fn conf_bar(c: Confidence, ascii: bool) -> String {
    let n = if c >= Decimal::new(90, 2) {
        3
    } else if c >= Decimal::new(70, 2) {
        2
    } else {
        1
    };
    let (on, off) = if ascii { ("#", "-") } else { ("▰", "▱") };
    format!("{}{}", on.repeat(n), off.repeat(3 - n))
}

impl App {
    pub fn cl_active(&self) -> bool {
        self.ui.tab == Tab::Classify
    }

    pub fn cl_queue(&self) -> bool {
        self.cl_active() && self.ui.classify.mode == Mode::Queue && self.classify.is_some()
    }

    /// The scope's org aliases (None = all).
    fn cl_scope_orgs(&self) -> Option<Vec<String>> {
        match self.ui.scope {
            Scope::All => None,
            ref s => Some(
                self.members(s)
                    .into_iter()
                    .map(|i| self.model.orgs[i].alias.clone())
                    .collect(),
            ),
        }
    }

    /// The filter with the scope applied (orgs outside the scope never show).
    fn cl_filter_scoped(&self, d: &Data, text: &str) -> CResult<Filter> {
        let mut f = d.provider.parse_filter(text)?;
        if let Some(scope) = self.cl_scope_orgs() {
            if f.orgs.is_empty() {
                f.orgs = scope;
            } else {
                f.orgs.retain(|o| scope.contains(o));
                if f.orgs.is_empty() {
                    f.orgs.push("\u{0}none".into());
                }
            }
        }
        Ok(f)
    }

    /// The Queue's rows (cached per store revision, filter and scope).
    pub fn cl_rows(&self) -> Rows {
        let Some(d) = self.classify.as_ref() else {
            return Rows {
                rows: Rc::new(Vec::new()),
                error: None,
                total: 0,
                amount_w: 0,
                one_year: true,
            };
        };
        let key = (
            d.provider.rev(),
            self.ui.classify.filter.clone(),
            format!("{:?}", self.ui.scope),
            d.provider.config_hash(),
        );
        if let Some((k, r)) = d.rows.borrow().as_ref()
            && *k == key
        {
            return r.clone();
        }
        // the strip's "of N" needs the unfiltered count only when a filter is typed
        let base = if self.ui.classify.filter.trim().is_empty() {
            None
        } else {
            Some(
                // the scope's open lines, counted (no rows built, no sort)
                self.cl_filter_scoped(d, "")
                    .map(|f| d.provider.counts_for(&f).open)
                    .unwrap_or(0),
            )
        };
        let (rows, error) = match self.cl_filter_scoped(d, &self.ui.classify.filter) {
            Ok(f) => (d.provider.queue(&f, Sort::StaleFirst), None),
            Err(e) => (
                d.provider.queue(
                    &self.cl_filter_scoped(d, "").unwrap_or_default(),
                    Sort::StaleFirst,
                ),
                Some(e.message),
            ),
        };
        // the widest amount is the largest positive or the largest negative (its parentheses): format two
        let widest = |neg: bool| {
            rows.iter()
                .filter(|r| r.amount.is_sign_negative() == neg)
                .map(|r| r.amount.abs())
                .max()
                .map_or(0, |a| {
                    ownpurse_core::money::fmt(if neg { -a } else { a }).len()
                })
        };
        let amount_w = widest(false).max(widest(true));
        let one_year = rows
            .windows(2)
            .all(|w| w[0].date.get(..4) == w[1].date.get(..4));
        let total = base.unwrap_or(rows.len());
        let r = Rows {
            rows: Rc::new(rows),
            error,
            total,
            amount_w,
            one_year,
        };
        *d.rows.borrow_mut() = Some((key, r.clone()));
        r
    }

    /// The cursor's row (the selected anchor; the first row when it isn't listed).
    pub fn cl_cursor(&self) -> Option<usize> {
        let rows = self.cl_rows().rows;
        if rows.is_empty() {
            return None;
        }
        Some(
            self.ui
                .classify
                .selected_anchor
                .as_ref()
                .and_then(|a| rows.iter().position(|r| r.anchor.to_string() == *a))
                .unwrap_or(0),
        )
    }

    pub fn cl_amount_width(&self) -> usize {
        self.cl_rows().amount_w
    }

    pub fn cl_selected(&self) -> Option<QueueRow> {
        let rows = self.cl_rows().rows;
        self.cl_cursor().and_then(|i| rows.get(i).cloned())
    }

    pub fn cl_detail(&self) -> Option<LineDetail> {
        let r = self.cl_selected()?;
        self.classify.as_ref()?.provider.detail(&r.anchor)
    }

    fn cl_select(&mut self, pos: usize) {
        let rows = self.cl_rows().rows;
        if let Some(r) = rows.get(pos.min(rows.len().saturating_sub(1))) {
            self.ui.classify.selected_anchor = Some(r.anchor.to_string());
        }
        self.cl_keep_visible();
    }

    pub fn cl_page(&self) -> usize {
        self.hits
            .content
            .map_or(10, |r| (r.height as usize).saturating_sub(1).max(1))
    }

    fn cl_keep_visible(&mut self) {
        let Some(cur) = self.cl_cursor() else { return };
        let h = self.cl_page();
        let top = &mut self.ui.classify.top;
        if cur < *top {
            *top = cur;
        } else if cur >= *top + h {
            *top = cur + 1 - h;
        }
    }

    /// The first drawn row: the stored top, moved just enough to show the cursor (pure; draw-safe).
    pub fn cl_top(&self, h: usize) -> usize {
        let n = self.cl_rows().rows.len();
        let mut top = self.ui.classify.top.min(n.saturating_sub(1));
        if let Some(cur) = self.cl_cursor() {
            if cur < top {
                top = cur;
            } else if h > 0 && cur >= top + h {
                top = cur + 1 - h;
            }
        }
        top
    }

    /// The anchor the cursor lands on after this line leaves the queue (the next, else the previous).
    fn cl_next_anchor(&self) -> Option<String> {
        let rows = self.cl_rows().rows;
        let cur = self.cl_cursor()?;
        rows.get(cur + 1)
            .or_else(|| cur.checked_sub(1).and_then(|p| rows.get(p)))
            .map(|r| r.anchor.to_string())
    }

    pub(crate) fn cl_write_pub(&mut self, op: Op, fx: &mut Vec<Effect>) {
        self.cl_write(op, fx);
    }

    fn cl_write(&mut self, op: Op, fx: &mut Vec<Effect>) {
        self.ui.classify.last_written = match &op {
            Op::Accept(a)
            | Op::Note(a, _)
            | Op::Undo(a)
            | Op::SetLabel { anchor: a, .. }
            | Op::AddToGroup { anchor: a, .. }
            | Op::NewGroup { anchor: a, .. } => Some(a.clone()),
            Op::Bulk { .. }
            | Op::Config(_)
            | Op::GroupMember { .. }
            | Op::GroupRemove { .. }
            | Op::GroupStatus { .. }
            | Op::GroupNote { .. }
            | Op::Agent(_)
            | Op::UndoAgentBatch
            | Op::UndoEvents(_) => None,
        };
        fx.push(Effect::Classify(Write {
            op,
            actor: "human".into(),
            batch: format!("tui:{}", self.rev + 1),
        }));
    }

    /// Choices for a picker (label type choices, or the line's org chart), filtered by the query.
    pub fn cl_picker_rows(&self) -> Vec<(String, String)> {
        let (Some(p), Some(d)) = (&self.ui.classify.picker, self.classify.as_ref()) else {
            return Vec::new();
        };
        if p.label_type.starts_with("g_") {
            return self.grp_picker_rows(&p.label_type, &p.query);
        }
        let cfg = d.provider.config();
        let all: Vec<(String, String)> = match p.label_type.as_str() {
            "group" => std::iter::once((NEW_GROUP.to_string(), "New group…".to_string()))
                .chain(
                    self.cl_selected()
                        .map(|r| d.provider.groups_near(&r.anchor))
                        .unwrap_or_default(),
                )
                .collect(),
            "group_kind" => cfg
                .groups
                .kinds
                .iter()
                .map(|k| (k.clone(), k.replace('_', " ")))
                .collect(),
            _ => match cfg.label_type(&p.label_type) {
                Some(t) if t.kind == ownpurse_core::classify::config::LabelKind::Account => self
                    .cl_selected()
                    .map(|r| d.provider.account_choices(&r.org))
                    .unwrap_or_default(),
                Some(t) => t.active().map(|c| (c.id.clone(), c.name.clone())).collect(),
                None => Vec::new(),
            },
        };
        let q = p.query.to_lowercase();
        all.into_iter()
            .filter(|(id, name)| {
                q.is_empty() || id.to_lowercase().contains(&q) || name.to_lowercase().contains(&q)
            })
            .collect()
    }

    /// Project choices in detail-pane order (`p`+digit picks the N-th).
    pub fn cl_projects(&self) -> Vec<(String, String)> {
        self.classify
            .as_ref()
            .and_then(|d| d.provider.config().label_type("project"))
            .map(|t| t.active().map(|c| (c.id.clone(), c.name.clone())).collect())
            .unwrap_or_default()
    }

    /// The bulk preview at the current threshold.
    pub fn cl_bulk_preview(&self) -> Option<BulkPreview> {
        let b = self.ui.classify.bulk.as_ref()?;
        let d = self.classify.as_ref()?;
        let f = self.cl_filter_scoped(d, &self.ui.classify.filter).ok()?;
        Some(d.provider.bulk_preview(&f, hundredths(b.threshold)))
    }

    pub fn cl_counts(&self) -> Option<QueueCounts> {
        let d = self.classify.as_ref()?;
        let rev = d.provider.rev();
        if let Some((r, c)) = *d.counts.borrow()
            && r == rev
            && !d.config_moved.get()
        {
            return Some(c);
        }
        let c = d.provider.counts();
        *d.counts.borrow_mut() = Some((rev, c));
        d.config_moved.set(false);
        Some(c)
    }

    pub fn cl_text_mut(&mut self) -> Option<&mut String> {
        if self.ui.classify.catalog.editor.is_some() {
            return self.cat_text_mut();
        }
        let c = &mut self.ui.classify;
        if let Some(p) = c.picker.as_mut() {
            return Some(&mut p.query);
        }
        c.note.as_mut().map(|n| &mut n.text)
    }

    /// Keys while the tab is open. Returns false for anything the shell handles.
    pub fn cl_action(&mut self, action: &str, fx: &mut Vec<Effect>) -> bool {
        if !self.cl_active() || self.ui.overlay.is_some() || self.ui.band.is_some() {
            return false;
        }
        match action {
            "classify.mode_next" | "classify.mode_prev" => {
                let i = self.ui.classify.mode.index();
                let n = Mode::ALL.len();
                let j = if action == "classify.mode_next" {
                    (i + 1) % n
                } else {
                    (i + n - 1) % n
                };
                self.ui.classify.mode = Mode::ALL[j];
                return true;
            }
            "classify.cancel" => {
                let c = &mut self.ui.classify;
                c.pending_project = false;
                c.picker = None;
                c.note = None;
                c.bulk = None;
                return true;
            }
            _ => {}
        }
        if self.ui.classify.mode == Mode::Catalog {
            return self.cat_action(action, fx);
        }
        if self.ui.classify.mode == Mode::Groups && self.classify.is_some() {
            if self.ui.classify.picker.is_some() {
                return self.cl_picker_action(action, fx);
            }
            if self.ui.classify.note.is_some() {
                return self.cl_note_action(action, fx);
            }
            return self.grp_action(action, fx);
        }
        if self.ui.classify.mode != Mode::Queue || self.classify.is_none() {
            return false;
        }
        // sealed editors first
        if self.ui.classify.bulk.is_some() {
            return self.cl_bulk_action(action, fx);
        }
        if self.ui.classify.picker.is_some() {
            return self.cl_picker_action(action, fx);
        }
        if self.ui.classify.note.is_some() {
            return self.cl_note_action(action, fx);
        }
        if self.ui.prompt.is_some() {
            return false;
        }
        let sel = self.cl_selected().map(|r| r.anchor.to_string());
        match action {
            "drill.in" | "classify.accept" => {
                if let Some(a) = sel {
                    let next = self.cl_next_anchor();
                    self.cl_write(Op::Accept(a), fx);
                    if next.is_some() {
                        self.ui.classify.selected_anchor = next;
                    }
                    self.cl_keep_visible();
                }
            }
            "classify.skip" => {
                self.ui.classify.last_written = None;
                if let Some(c) = self.cl_cursor() {
                    self.cl_select(c + 1);
                }
            }
            "classify.undo" => {
                // right after a write that moved on (accept), `u` undoes it and goes back to its line
                let target = self.ui.classify.last_written.clone().or(sel);
                if let Some(a) = target {
                    self.ui.classify.selected_anchor = Some(a.clone());
                    self.cl_write(Op::Undo(a), fx);
                }
            }
            "classify.project" if sel.is_some() => self.ui.classify.pending_project = true,
            "classify.bearer" | "classify.treatment" | "classify.account" | "classify.group"
                if sel.is_some() =>
            {
                let ty = match action {
                    "classify.bearer" => "bearer",
                    "classify.treatment" => "treatment",
                    "classify.group" => "group",
                    _ => "should_be_account",
                };
                self.ui.classify.picker = Some(LabelPicker {
                    label_type: ty.into(),
                    query: String::new(),
                    sel: 0,
                });
            }
            "classify.note" if sel.is_some() => {
                self.ui.classify.note = Some(NoteEdit {
                    text: String::new(),
                    multi: false,
                })
            }
            "classify.bulk" => {
                let t = self
                    .classify
                    .as_ref()
                    .map(|d| d.provider.config().queue.threshold)
                    .unwrap_or(Decimal::new(90, 2));
                let t = u8::try_from((t * Decimal::from(100)).round().mantissa()).unwrap_or(90);
                self.ui.classify.bulk = Some(BulkUi {
                    threshold: t.clamp(50, 100),
                    focus: 0,
                });
            }
            "classify.make_rule" => self.cat_make_rule(),
            "classify.project" | "classify.bearer" | "classify.treatment" | "classify.account"
            | "classify.group" | "classify.note" => self.ui.note = Some("No line selected.".into()),
            a => {
                let n = self.cl_rows().rows.len();
                let page = self.cl_page();
                match crate::views::activity::moved(a, self.cl_cursor().unwrap_or(0), n, page) {
                    Some(pos) if n > 0 => {
                        self.ui.classify.last_written = None;
                        self.cl_select(pos)
                    }
                    Some(_) => {}
                    None => return false,
                }
            }
        }
        true
    }

    /// `p` then a digit: the N-th project (the detail pane's order).
    pub fn cl_project_digit(&mut self, c: char, fx: &mut Vec<Effect>) {
        self.ui.classify.pending_project = false;
        let n = c.to_digit(10).unwrap_or(0) as usize;
        let projects = self.cl_projects();
        let (Some(r), Some((id, _))) = (
            self.cl_selected(),
            n.checked_sub(1).and_then(|i| projects.get(i)),
        ) else {
            self.ui.note = Some(format!("No project {n} (1 to {}).", projects.len()));
            return;
        };
        self.cl_write(
            Op::SetLabel {
                anchor: r.anchor.to_string(),
                label_type: "project".into(),
                value: id.clone(),
            },
            fx,
        );
    }

    fn cl_picker_action(&mut self, action: &str, fx: &mut Vec<Effect>) -> bool {
        let n = self.cl_picker_rows().len();
        match action {
            "classify.pick_prev" | "classify.pick_next" => {
                if let Some(p) = self.ui.classify.picker.as_mut()
                    && n > 0
                {
                    p.sel = if action == "classify.pick_next" {
                        (p.sel + 1) % n
                    } else {
                        (p.sel + n - 1) % n
                    };
                }
            }
            "classify.pick"
                if self
                    .ui
                    .classify
                    .picker
                    .as_ref()
                    .is_some_and(|p| p.label_type.starts_with("g_")) =>
            {
                let rows = self.cl_picker_rows();
                if let Some(p) = self.ui.classify.picker.clone() {
                    match rows.get(p.sel.min(n.saturating_sub(1))) {
                        Some((id, _)) => {
                            self.ui.classify.picker = None;
                            let id = id.clone();
                            self.grp_pick(&p.label_type, &id, fx);
                        }
                        None => self.ui.note = Some("Nothing matches; Esc closes.".into()),
                    }
                }
            }
            "classify.pick" => {
                let rows = self.cl_picker_rows();
                let p = self.ui.classify.picker.clone();
                if let (Some(p), Some(r)) = (p, self.cl_selected()) {
                    match rows.get(p.sel.min(n.saturating_sub(1))) {
                        Some((id, _)) if p.label_type == "group" && id == NEW_GROUP => {
                            // a new group: its kind next
                            self.ui.classify.picker = Some(LabelPicker {
                                label_type: "group_kind".into(),
                                query: String::new(),
                                sel: 0,
                            });
                        }
                        Some((id, _)) => {
                            self.ui.classify.picker = None;
                            let anchor = r.anchor.to_string();
                            let op = match p.label_type.as_str() {
                                "group" => Op::AddToGroup {
                                    anchor,
                                    group: id.clone(),
                                },
                                "group_kind" => Op::NewGroup {
                                    anchor,
                                    kind: id.clone(),
                                    title: format!(
                                        "{} {}",
                                        crate::views::classify::payee_text(&r),
                                        r.date
                                    ),
                                },
                                _ => Op::SetLabel {
                                    anchor,
                                    label_type: p.label_type,
                                    value: id.clone(),
                                },
                            };
                            self.cl_write(op, fx);
                        }
                        None => self.ui.note = Some("Nothing matches; Esc closes.".into()),
                    }
                }
            }
            _ => return false,
        }
        true
    }

    fn cl_note_action(&mut self, action: &str, fx: &mut Vec<Effect>) -> bool {
        match action {
            "classify.note_multi" => {
                if let Some(n) = self.ui.classify.note.as_mut() {
                    n.multi = !n.multi;
                }
            }
            "classify.note_enter" if self.ui.classify.note.as_ref().is_some_and(|n| n.multi) => {
                self.type_char_pub('\n');
            }
            "classify.note_enter" | "classify.note_save" => {
                let text = self
                    .ui
                    .classify
                    .note
                    .as_ref()
                    .map(|n| n.text.trim().to_string())
                    .unwrap_or_default();
                if text.is_empty() {
                    self.ui.note = Some("A note needs text; Esc closes.".into());
                } else if self.ui.classify.mode == Mode::Groups {
                    if let Some(g) = self.grp_selected() {
                        self.ui.classify.note = None;
                        self.cl_write(Op::GroupNote { group: g.id, text }, fx);
                    }
                } else if let Some(r) = self.cl_selected() {
                    self.ui.classify.note = None;
                    self.cl_write(Op::Note(r.anchor.to_string(), text), fx);
                }
            }
            _ => return false,
        }
        true
    }

    fn cl_bulk_action(&mut self, action: &str, fx: &mut Vec<Effect>) -> bool {
        let Some(b) = self.ui.classify.bulk.as_mut() else {
            return false;
        };
        match action {
            "classify.bulk_more" => b.threshold = (b.threshold + 5).min(100),
            "classify.bulk_less" => b.threshold = b.threshold.saturating_sub(5).max(50),
            "classify.bulk_focus" => b.focus = 1 - b.focus.min(1),
            "classify.bulk_go" => {
                let b = b.clone();
                if b.focus == 0 {
                    self.ui.classify.bulk = None;
                    return true;
                }
                let n = self.cl_bulk_preview().map_or(0, |p| p.anchors.len());
                self.ui.classify.bulk = None;
                if n == 0 {
                    self.ui.note = Some("No line is at or above the threshold.".into());
                    return true;
                }
                let orgs = self.cl_scope_orgs().unwrap_or_default();
                self.cl_write(
                    Op::Bulk {
                        filter: self.ui.classify.filter.clone(),
                        orgs,
                        threshold: b.threshold,
                    },
                    fx,
                );
            }
            _ => return false,
        }
        true
    }
}

/// Run one write against the store (the loop's `Effect::Classify`); the answer goes back as
/// `Msg::Classified`.
pub fn perform(app: &mut App, w: Write) -> Msg {
    // the core caches its rows: after a write the Queue simply re-reads (≈ 1.6 ms at 5,000 lines)
    perform_write(app, w)
}

fn perform_write(app: &mut App, w: Write) -> Msg {
    let Some(d) = app.classify.as_mut() else {
        return Msg::Classified(Err("Classification isn't set up.".into()));
    };
    let human = || {
        WriteCtx::new(
            Actor::parse(&w.actor).unwrap_or_else(|_| Actor::human()),
            BatchId::new(w.batch.clone()),
        )
    };
    match &w.op {
        Op::Agent(req) => return Msg::AgentWrote(Box::new(d.provider.apply(req))),
        Op::UndoAgentBatch => {
            return Msg::AgentUndone(d.provider.undo_batch(&human()).map_err(|e| e.message), None);
        }
        Op::UndoEvents(ids) => {
            return Msg::AgentUndone(
                d.provider.undo_events(ids, &human()).map_err(|e| e.message),
                Some(ids.clone()),
            );
        }
        _ => {}
    }
    if let Op::Config(staged) = &w.op {
        let actor = Actor::parse(&w.actor).unwrap_or_else(|_| Actor::human());
        let ctx = WriteCtx::new(actor, BatchId::new(w.batch.clone()));
        let r = catalog::config_edits(staged, d.provider.config())
            .and_then(|e| d.provider.stage(&e).map_err(|e| e.message))
            .and_then(|s| d.provider.write_config(&s, &ctx).map_err(|e| e.message));
        d.config_moved.set(true);
        return Msg::ConfigWritten(r.map(|o| o.summary));
    }
    let actor = Actor::parse(&w.actor).unwrap_or_else(|_| Actor::human());
    let ctx = WriteCtx::new(actor, BatchId::new(w.batch));
    let anchor = |s: &str| Anchor::parse(s).map_err(|e| e.message);
    let r: Result<WriteOutcome, String> = match &w.op {
        Op::Config(_) | Op::Agent(_) | Op::UndoAgentBatch | Op::UndoEvents(_) => {
            Err("unreachable: answered above".into())
        }
        Op::Accept(a) => anchor(a).and_then(|a| d.provider.accept(&a, &ctx).map_err(|e| e.message)),
        Op::SetLabel {
            anchor: a,
            label_type,
            value,
        } => anchor(a).and_then(|a| {
            d.provider
                .set_label(&a, label_type, value, &ctx)
                .map_err(|e| e.message)
        }),
        Op::Note(a, t) => {
            anchor(a).and_then(|a| d.provider.add_note(&a, t, &ctx).map_err(|e| e.message))
        }
        Op::Undo(a) => {
            anchor(a).and_then(|a| d.provider.undo_line(&a, &ctx).map_err(|e| e.message))
        }
        Op::GroupMember {
            group,
            anchor: a,
            role,
        } => anchor(a).and_then(|a| {
            d.provider
                .group_member(group, &a, role.as_deref(), &ctx)
                .map_err(|e| e.message)
        }),
        Op::GroupRemove { group, anchor: a } => anchor(a).and_then(|a| {
            d.provider
                .group_remove(group, &a, &ctx)
                .map_err(|e| e.message)
        }),
        Op::GroupStatus { group, status } => d
            .provider
            .group_status(group, status, &ctx)
            .map_err(|e| e.message),
        Op::GroupNote { group, text } => d
            .provider
            .group_note(group, text, &ctx)
            .map_err(|e| e.message),
        Op::AddToGroup { anchor: a, group } => anchor(a).and_then(|a| {
            d.provider
                .add_to_group(group, &a, &ctx)
                .map_err(|e| e.message)
        }),
        Op::NewGroup {
            anchor: a,
            kind,
            title,
        } => anchor(a).and_then(|a| {
            d.provider
                .new_group(kind, title, &a, &ctx)
                .map_err(|e| e.message)
        }),
        Op::Bulk {
            filter,
            orgs,
            threshold,
        } => d
            .provider
            .parse_filter(filter)
            .map_err(|e| e.message)
            .and_then(|mut f| {
                if !orgs.is_empty() {
                    if f.orgs.is_empty() {
                        f.orgs = orgs.clone();
                    } else {
                        f.orgs.retain(|o| orgs.contains(o));
                    }
                }
                d.provider
                    .bulk_accept(&f, hundredths(*threshold), &ctx)
                    .map_err(|e| e.message)
            }),
    };
    Msg::Classified(r.map(|o| o.summary))
}

/// Run the classify writes among `fx` (snapshot scripts and tests; the runtime does the same).
pub fn run_effects(app: &mut App, fx: Vec<Effect>) {
    for e in fx {
        if let Effect::Classify(w) = e {
            let m = perform(app, w);
            let more = crate::app::update(app, m);
            run_effects(app, more);
        }
    }
}

impl App {
    /// A click on drawn row `pos` (whole-list position).
    pub(crate) fn cl_click(&mut self, pos: usize) {
        self.ui.classify.last_written = None;
        self.cl_select(pos);
    }

    /// The wheel scrolls the list; the selection moves only if it left the view.
    pub(crate) fn cl_wheel(&mut self, down: bool) {
        let n = self.cl_rows().rows.len();
        let h = self.cl_page();
        let top = self.cl_top(h);
        let new_top = if down {
            (top + 1).min(n.saturating_sub(h))
        } else {
            top.saturating_sub(1)
        };
        self.ui.classify.top = new_top;
        if let Some(cur) = self.cl_cursor() {
            if cur < new_top {
                self.cl_select(new_top);
            } else if cur >= new_top + h {
                self.cl_select(new_top + h - 1);
            }
        }
    }
}

impl App {
    /// `classify.toml` changed on disk (the watcher): reload it. Valid → the band "classify.toml changed on
    /// disk · reloaded · N changes"; invalid → the error band with file:line, the last valid config live;
    /// unchanged (our own write) → nothing at all (V13).
    pub fn cl_config_changed(&mut self) -> bool {
        use crate::run::{ResultBand, ResultLevel};
        use ownpurse_core::classify::config::Reload;
        let Some(d) = self.classify.as_mut() else {
            return false;
        };
        match d.provider.reload_config() {
            Reload::Unchanged => false,
            Reload::Reloaded { changes } => {
                d.config_moved.set(true);
                let line = format!(
                    "classify.toml changed on disk · reloaded · {changes} change{}",
                    if changes == 1 { "" } else { "s" }
                );
                self.ui.result = Some(ResultBand {
                    level: ResultLevel::Ok,
                    lines: vec![line.clone()],
                    copy: line,
                });
                true
            }
            Reload::Invalid(issues) => {
                let mut lines: Vec<String> = issues.iter().take(2).map(|i| i.to_string()).collect();
                if issues.len() > 2 {
                    lines.push(format!("… and {} more", issues.len() - 2));
                }
                lines.push("the last valid classify.toml stays live; fix the file and save".into());
                let band = Some(ResultBand {
                    level: ResultLevel::Error,
                    copy: issues
                        .iter()
                        .map(|i| i.to_string())
                        .collect::<Vec<_>>()
                        .join("\n"),
                    lines,
                });
                // the same problems again (another save of the same bad file): nothing new to show
                let new = self.ui.result != band;
                self.ui.result = band;
                new
            }
        }
    }
}

/// Watch `classify.toml` for hand edits (spec classify §6): the OS file events on its folder (an editor's
/// atomic save replaces the file), filtered to that file name, call `changed`. No timer and no polling: an
/// idle TUI writes nothing (V13). Keep the returned watcher alive for as long as events are wanted.
pub fn watch(
    config: &std::path::Path,
    changed: impl Fn() + Send + 'static,
) -> Option<notify::RecommendedWatcher> {
    use notify::{EventKind, RecursiveMode, Watcher};
    let dir = config.parent()?.to_path_buf();
    let name = config.file_name()?.to_os_string();
    let mut w = notify::recommended_watcher(move |res: notify::Result<notify::Event>| {
        let Ok(ev) = res else { return };
        if matches!(ev.kind, EventKind::Access(_)) {
            return;
        }
        if ev
            .paths
            .iter()
            .any(|p| p.file_name() == Some(name.as_os_str()))
        {
            changed();
        }
    })
    .ok()?;
    w.watch(&dir, RecursiveMode::NonRecursive).ok()?;
    Some(w)
}

/// An agent's write as Activity keeps it: its row id, the events (for "Undo this write?"), what it did.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct AgentWrite {
    pub log_id: i64,
    pub events: Vec<i64>,
    pub summary: String,
    pub lines: usize,
    pub undone: bool,
}

impl App {
    /// An agent's classify request (bridge `kind: "classify"`), through `update` like the user's keys.
    /// Refused `user_busy` (exit 4) when it touches a line the user has open in a picker, the note editor or
    /// the bulk preview, or a config op while the Catalog editor or diff is open; else the write is an
    /// effect the loop runs (`Workspace::apply`) and `Msg::AgentWrote` reports it.
    pub(crate) fn cl_agent_request(&mut self, req: ClassifyRequest, fx: &mut Vec<Effect>) {
        let Some(d) = self.classify.as_ref() else {
            self.agent.classify_reply = Some(ownpurse_core::bridge::error_doc(
                "not_set_up",
                1,
                "Classification isn't set up in this TUI (no classify.toml). Nothing was changed.",
                &[],
            ));
            return;
        };
        let targets = match d.provider.resolve_targets(&req) {
            Ok(t) => t,
            Err(e) => {
                self.agent.classify_reply =
                    Some(ownpurse_core::classify::request::error_response(&e));
                return;
            }
        };
        let c = &self.ui.classify;
        let open_line = (c.picker.is_some() || c.note.is_some() || c.pending_project)
            .then(|| self.cl_selected().map(|r| r.anchor))
            .flatten();
        let mut busy: Option<&str> = None;
        if let Some(a) = &open_line
            && targets.contains(a)
        {
            busy = c.busy();
        }
        if busy.is_none()
            && c.bulk.is_some()
            && let Some(p) = self.cl_bulk_preview()
            && p.anchors.iter().any(|a| targets.contains(a))
        {
            busy = Some("in the bulk-accept preview over these lines");
        }
        if busy.is_none() && matches!(req.op, ownpurse_core::classify::request::Op::Config { .. }) {
            busy = c.catalog_busy();
        }
        if let Some(b) = busy {
            self.agent_skipped();
            self.agent_log(&format!("classify {} · refused, user busy", req.op.name()));
            self.agent.classify_reply = Some(ownpurse_core::bridge::error_doc(
                "user_busy",
                4,
                format!("The user is {b}. Nothing was changed."),
                &[
                    "Read `ownpurse ui state` again when they are done, and resend if it still makes sense.",
                ],
            ));
            return;
        }
        self.agent.pending_caption = req.caption.clone();
        self.agent.pending_request = Some(Box::new(req.clone()));
        // where the user's cursor is now (the rows change once the write lands)
        self.agent.pending_cursor = self.cl_cursor();
        fx.push(Effect::Classify(Write {
            op: Op::Agent(Box::new(req)),
            actor: String::new(),
            batch: String::new(),
        }));
    }

    /// The loop applied an agent request: answer the bridge, and show it (the notice, Activity, the rows).
    /// The user's cursor stays where it was (the same row when its line left the queue).
    pub(crate) fn cl_agent_wrote(
        &mut self,
        r: ownpurse_core::classify::Result<ApplyOutcome>,
        caption: Option<String>,
    ) {
        let before = self.agent.pending_cursor.take();
        let anchor = self.ui.classify.selected_anchor.clone();
        match r {
            Err(e) => {
                self.agent_log(&format!("classify · refused, {}", e.code));
                self.agent.classify_reply =
                    Some(ownpurse_core::classify::request::error_response(&e));
            }
            Ok(o) => {
                self.agent.classify_reply = Some(o.to_json());
                if o.events.is_empty() {
                    self.agent_log(&format!("classify {} · {}", o.op, o.summary));
                    return;
                }
                self.agent_log(&format!("classify {} · {}", o.op, o.summary));
                let log_id = self.agent.log.last().map_or(0, |r| r.id);
                self.agent.writes.push(AgentWrite {
                    log_id,
                    events: o.events.clone(),
                    summary: o.summary.clone(),
                    lines: o.lines,
                    undone: false,
                });
                let req = self.agent.pending_request.take();
                let n = match (caption, req) {
                    (Some(c), _) => WroteNotice::caption(c),
                    (None, Some(r)) => self.compose_notice(&r, &o),
                    (None, None) => WroteNotice {
                        prefix: o.summary.clone(),
                        ..WroteNotice::default()
                    },
                };
                self.agent.notice = Some(crate::agent::Notice::Wrote(n));
                self.agent.acted = true;
                if let Some(d) = self.classify.as_ref() {
                    d.config_moved.set(true);
                }
                // arriving never takes the cursor: its line, else the same place in the list
                if let Some(a) = anchor
                    && !self
                        .cl_rows()
                        .rows
                        .iter()
                        .any(|r| r.anchor.to_string() == a)
                    && let Some(i) = before
                {
                    let rows = self.cl_rows().rows;
                    if let Some(r) = rows.get(i.min(rows.len().saturating_sub(1))) {
                        self.ui.classify.selected_anchor = Some(r.anchor.to_string());
                    }
                }
                self.dirty = true;
                self.rev += 1;
            }
        }
    }

    /// `U` or Activity's undo finished: the dim notice `agent write undone · 12 lines`.
    pub(crate) fn cl_agent_undone(
        &mut self,
        r: Result<WriteOutcome, String>,
        ids: Option<Vec<i64>>,
    ) {
        match r {
            Ok(o) => {
                // Activity's undo names its write; `U` undid the latest one not yet undone
                let hit = match &ids {
                    Some(ids) => self.agent.writes.iter().rposition(|w| &w.events == ids),
                    None => self.agent.writes.iter().rposition(|w| !w.undone),
                };
                if let Some(i) = hit {
                    self.agent.writes[i].undone = true;
                }
                if let Some(d) = self.classify.as_ref() {
                    d.config_moved.set(true);
                }
                self.agent.notice = Some(crate::agent::Notice::Undone(format!(
                    "agent write undone · {} line{}",
                    o.lines,
                    if o.lines == 1 { "" } else { "s" }
                )));
                self.dirty = true;
                self.rev += 1;
            }
            Err(e) => self.ui.note = Some(format!("Nothing undone: {e}")),
        }
    }
}

/// An agent write's notice, as segments (design, 2026-10-06): `[action + count] · [kept] · [U undo]`, e.g.
/// `◆ agent: project Wayfinding refit on 2 lines · 1 kept (yours) · U undo`. When it doesn't fit, the
/// choice name shortens first (`Wayfinding…`), then the kept segment goes; `· U undo` never does, and
/// nothing is cut mid-word.
#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct WroteNotice {
    /// Before the name: `project `, `note on `, `3 lines to group `.
    pub prefix: String,
    /// The shortenable name: a choice, a payee, a group title (quoted where the copy quotes it).
    pub name: Option<String>,
    /// After the name: ` on 2 lines`, ` added to classify.toml`.
    pub suffix: String,
    /// Lines a person's labels kept (the core skipped those types).
    pub kept: usize,
}

impl WroteNotice {
    /// The agent's caption replaces the text (it keeps `· U undo`); it shortens by whole words.
    pub fn caption(c: String) -> WroteNotice {
        WroteNotice {
            name: Some(c),
            ..WroteNotice::default()
        }
    }

    fn main(&self, name: Option<&str>) -> String {
        format!("{}{}{}", self.prefix, name.unwrap_or(""), self.suffix)
    }

    /// The name shortened to its first `n` words, with `…` when shorter than the whole.
    fn name_words(&self, n: usize, ell: &str) -> Option<String> {
        let name = self.name.as_ref()?;
        let words: Vec<&str> = name.split(' ').collect();
        if n >= words.len() {
            return Some(name.clone());
        }
        let (q0, q1) = if name.starts_with('"') && name.ends_with('"') {
            ("", "\"")
        } else {
            ("", "")
        };
        Some(format!("{q0}{}{ell}{q1}", words[..n].join(" ")))
    }

    /// The whole text (the lead and `· U undo` included), fitted to `w` cells.
    pub fn fitted(&self, lead: &str, w: usize, ascii: bool) -> String {
        use crate::layout::width;
        let fold = |s: &str| crate::glyphs::fold(s, ascii);
        let ell = if ascii { "..." } else { "…" };
        let tail = fold(" · U undo");
        let kept = (self.kept > 0).then(|| fold(&format!(" · {} kept (yours)", self.kept)));
        let words = self.name.as_ref().map_or(0, |n| n.split(' ').count());
        let mut tries: Vec<(Option<String>, bool)> = Vec::new();
        // the whole name, then shorter names; with the kept segment first, then without it
        for with_kept in [true, false] {
            for n in (1..=words.max(1)).rev() {
                tries.push((self.name_words(n, ell), with_kept));
            }
        }
        for (name, with_kept) in &tries {
            if *with_kept && kept.is_none() {
                continue;
            }
            let text = format!(
                "{lead}{}{}{tail}",
                fold(&self.main(name.as_deref())),
                if *with_kept {
                    kept.clone().unwrap_or_default()
                } else {
                    String::new()
                }
            );
            if width(&text) <= w {
                return text;
            }
        }
        // still too long: whole words of the main text, then `· U undo` (never cut mid-word)
        let main = fold(&self.main(self.name_words(1, ell).as_deref()));
        let mut out = String::new();
        for word in main.split(' ') {
            let next = if out.is_empty() {
                word.to_string()
            } else {
                format!("{out} {word}")
            };
            if width(&format!("{lead}{next}{ell}{tail}")) > w {
                break;
            }
            out = next;
        }
        format!("{lead}{out}{ell}{tail}")
    }
}

fn lines_word(n: usize) -> String {
    format!("{n} line{}", if n == 1 { "" } else { "s" })
}

impl App {
    /// The notice for an applied request, from its op and the outcome's structured fields.
    pub(crate) fn compose_notice(&self, req: &ClassifyRequest, o: &ApplyOutcome) -> WroteNotice {
        use ownpurse_core::classify::request::{Op as R, UndoWhat};
        let d = self.classify.as_ref();
        let choice = |ty: &str, v: &str| {
            d.map_or_else(
                || v.to_string(),
                |d| d.provider.config().choice_name(ty, v).to_string(),
            )
        };
        let kept = {
            let mut a: Vec<&Anchor> = o.skipped.iter().map(|s| &s.anchor).collect();
            a.sort();
            a.dedup();
            a.len()
        };
        let on = format!(" on {}", lines_word(o.lines));
        let title = |g: &str| {
            d.and_then(|d| d.provider.group_title(g))
                .map_or_else(|| g.to_string(), |t| format!("\"{t}\""))
        };
        let n = |prefix: String, name: Option<String>, suffix: String| WroteNotice {
            prefix,
            name,
            suffix,
            kept,
        };
        match &req.op {
            R::Set { labels, .. } => match labels.as_slice() {
                [(ty, v)] => n(format!("{ty} "), Some(choice(ty, v)), on),
                // several: the first by name, then `+n` (as the Queue's suggestion column)
                [(ty, v), rest @ ..] => n(
                    format!("{ty} "),
                    Some(choice(ty, v)),
                    format!(" +{}{on}", rest.len()),
                ),
                [] => n("labels".into(), None, on),
            },
            R::Clear { types, .. } => n(format!("cleared {}", types.join(", ")), None, on),
            R::Accept { .. } => n(
                format!("accepted {}", lines_word(o.lines)),
                None,
                String::new(),
            ),
            R::Note { target, .. } => match target {
                ownpurse_core::classify::store::NoteTarget::Line(a) => {
                    let what = d.and_then(|d| d.provider.detail(a)).map(|x| {
                        let who = if x.line.payee.is_empty() {
                            x.line.description.clone()
                        } else {
                            x.line.payee.clone()
                        };
                        format!("{who} {}", ownpurse_core::money::fmt(x.line.amount.abs()))
                    });
                    n("note on ".into(), what, String::new())
                }
                ownpurse_core::classify::store::NoteTarget::Group(g) => {
                    n("note on group ".into(), Some(title(g)), String::new())
                }
            },
            R::GroupCreate { kind, title: t } => n(
                format!("{} group ", kind.replace('_', " ")),
                Some(format!("\"{t}\"")),
                " created".into(),
            ),
            R::GroupAdd { group, .. } => n(
                format!("{} to group ", lines_word(o.lines)),
                Some(title(group)),
                String::new(),
            ),
            R::GroupRemove { group, .. } => n(
                format!("{} out of group ", lines_word(o.lines)),
                Some(title(group)),
                String::new(),
            ),
            R::GroupStatus { group, status } => n(
                "group ".into(),
                Some(title(group)),
                format!(" marked {status}"),
            ),
            R::Undo(w) => n(
                match w {
                    UndoWhat::Batch => "undid its last write".into(),
                    UndoWhat::Events(e) => format!("undid {} events", e.len()),
                    UndoWhat::Line(_) => "undid its last write on a line".into(),
                },
                None,
                String::new(),
            ),
            R::Config { edits, .. } => match edits.as_slice() {
                [ownpurse_core::classify::config::ConfigEdit::AddChoice { label_type, choice }] => {
                    n(
                        format!("{label_type} "),
                        Some(format!("\"{}\"", choice.name)),
                        " added to classify.toml".into(),
                    )
                }
                many => n(
                    format!(
                        "{} change{} to classify.toml",
                        many.len(),
                        if many.len() == 1 { "" } else { "s" }
                    ),
                    None,
                    String::new(),
                ),
            },
        }
    }
}
