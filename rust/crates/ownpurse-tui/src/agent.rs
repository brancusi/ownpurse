//! The agent bridge, app side (v0.10, view-only). An agent reads the view
//! state, takes screenshots and sends view verbs through `ownpurse ui …`; the runtime (`bridge.rs`) owns
//! the socket. Everything here is pure app state:
//! - the view state as data (`state_json`): tab, scope, as-of, basis, filter, breadcrumb, the selected row
//!   and every visible row as anchors, the strip facts, size, overlay/band; never off-screen rows, never
//!   secrets;
//! - the verbs (allow-list in core `bridge::VERBS`), applied through `update` (`Msg::Agent`) with the same
//!   actions, clicks and prompt edits the keys use;
//! - what the user sees: the header mark (`◇ agent` / `◆ agent`), the notice in the status strip, `` ` `` to go
//!   back one step, and one Activity row per read, screenshot and send.

use crate::activity::LogRow;
use crate::app::{App, Effect, OvCol, Overlay, Prompt, Scope, Tab};
use crate::hit::Target;
use crate::ic_nav::{IcRow, s};
use serde_json::{Map, Value, json};

/// Seconds without a request after which the header stops showing "connected" (checked on the minute tick;
/// no timer of its own).
pub const CONNECTED_SECS: i64 = 300;

/// Activity ids for agent rows: above any `http_log` id, so they sort as the newest.
const LOG_ID_BASE: i64 = 1 << 40;

/// The parts of the view an agent change can move, kept so `` ` `` can put them back (one step).
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ViewSnap {
    pub(crate) tab: Tab,
    pub(crate) scope: Scope,
    pub(crate) filter: String,
    pub(crate) as_of: String,
    pub(crate) overview_sel: Option<String>,
    pub(crate) overview_top: usize,
    pub(crate) overview_col: OvCol,
    pub(crate) chain: Vec<crate::app::Origin>,
    pub(crate) full_history: bool,
    pub(crate) differs_only: bool,
    pub(crate) ledger: std::collections::BTreeMap<String, crate::ledger_nav::LedgerNav>,
    pub(crate) ic: std::collections::BTreeMap<String, crate::ic_nav::IcNav>,
    pub(crate) ic_sort: usize,
    pub(crate) via: Option<crate::ic_nav::Via>,
    pub(crate) cmp: Option<crate::compare::CmpState>,
    pub(crate) activity: crate::views::activity::ActivityUi,
    pub(crate) system: crate::views::system::SystemUi,
    /// The Classify tab's mode, filter and cursor (editors closed).
    #[serde(default)]
    pub(crate) classify: crate::classify::ClassifyUi,
}

impl ViewSnap {
    pub fn of(u: &crate::app::Ui) -> ViewSnap {
        ViewSnap {
            tab: u.tab,
            scope: u.scope.clone(),
            filter: u.filter.clone(),
            as_of: u.as_of.clone(),
            overview_sel: u.overview_sel.clone(),
            overview_top: u.overview_top,
            overview_col: u.overview_col,
            chain: u.chain.clone(),
            full_history: u.full_history,
            differs_only: u.differs_only,
            ledger: u.ledger.clone(),
            ic: u.ic.clone(),
            ic_sort: u.ic_sort,
            via: u.via.clone(),
            cmp: u.cmp.clone(),
            activity: u.activity.clone(),
            system: u.system.clone(),
            classify: u.classify.settled(),
        }
    }

    /// Put this view back (history moves): the view fields, with any editor closed and the caret reset.
    pub fn restore_view(self, u: &mut crate::app::Ui) {
        self.restore(u);
        u.caret = None;
    }

    fn restore(self, u: &mut crate::app::Ui) {
        u.tab = self.tab;
        u.scope = self.scope;
        u.filter = self.filter;
        u.as_of = self.as_of;
        u.overview_sel = self.overview_sel;
        u.overview_top = self.overview_top;
        u.overview_col = self.overview_col;
        u.chain = self.chain;
        u.full_history = self.full_history;
        u.differs_only = self.differs_only;
        u.ledger = self.ledger;
        u.ic = self.ic;
        u.ic_sort = self.ic_sort;
        u.via = self.via;
        u.cmp = self.cmp;
        u.activity = self.activity;
        u.system = self.system;
        u.classify = self.classify;
        u.prompt = None;
    }
}

/// The status-strip notice.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Notice {
    /// `◆ agent: opened Ledger · scope design` (latest send; past tense).
    Changed(Vec<String>),
    /// `◇ agent: change skipped, you moved first` (a stale send was refused).
    Skipped,
    /// `◆ agent: set project Harvest Blend launch on 12 lines · U undo` (a classify write; its caption, when
    /// it had one). Persists until the view changes.
    Wrote(crate::classify::WroteNotice),
    /// `agent write undone · 12 lines` (dim; until the next key).
    Undone(String),
}

/// The agent's footprint in the app (outside `Ui`, so it never moves the view revision).
#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct AgentUi {
    /// A client has talked to this TUI recently (header `◇ agent`).
    pub connected: bool,
    /// When the last request came (the clock's seconds).
    pub last_seen: i64,
    /// A send or a screenshot since the user's last key (header `◆ agent`).
    pub acted: bool,
    pub notice: Option<Notice>,
    /// The view before the first change the user hasn't seen yet (`` ` `` restores it).
    pub undo: Option<ViewSnap>,
    /// A send was refused while the user was in an overlay: say so when it closes.
    pub skipped_pending: bool,
    /// One Activity row per read, screenshot and send (kept across record reloads).
    pub log: Vec<LogRow>,
    /// The outcome of the last applied verb (`Msg::Agent`): a description of what changed, or an error.
    /// Transient within one request; never part of the canonical state.
    #[serde(skip)]
    pub outcome: Option<Result<Option<String>, (String, String)>>,
    /// The outcome of the last send (`Msg::AgentSend`): what changed, or the refusal. Transient.
    #[serde(skip)]
    pub batch: Option<Result<Vec<String>, (String, String)>>,
    /// Per verb of the last send, in order: what it changed (`None`: nothing). Transient.
    #[serde(skip)]
    pub effects: Vec<Option<String>>,
    /// The last refused send: which verb (0-based) failed, and a help line specific to it. Transient.
    #[serde(skip)]
    pub failed: Option<(usize, Option<String>)>,
    /// The agent's classify writes this session (Activity rows; `U` and "Undo this write?").
    #[serde(default)]
    pub writes: Vec<crate::classify::AgentWrite>,
    /// The answer to the classify request in flight (the bridge reads it). Transient.
    #[serde(skip)]
    pub classify_reply: Option<Value>,
    /// That request's caption (it replaces the notice text). Transient.
    #[serde(skip)]
    pub pending_caption: Option<String>,
    /// The user's Queue cursor when that request arrived. Transient.
    #[serde(skip)]
    pub pending_cursor: Option<usize>,
    /// That request (its op and labels compose the notice). Transient.
    #[serde(skip)]
    pub pending_request: Option<Box<ownpurse_core::classify::request::ClassifyRequest>>,
}

/// One view verb, resolved against the model.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Verb {
    Tab(Tab),
    Scope(Scope),
    AsOf(String),
    Filter(String),
    ClearFilter,
    /// A visible row: `#3` (its number on screen), an anchor as JSON, or text (a field value such as an
    /// account code, else a substring; a bare number matching nothing is the row number).
    Select(String),
    Drill,
    /// Up one level (Esc).
    Up,
    /// Navigation history (Alt+← / Alt+→).
    HistBack,
    HistForward,
    /// The footer caption (None = clear).
    Caption(Option<String>),
    /// This session's label (`ui ls`).
    Label(String),
    /// true = full history.
    Basis(bool),
    Differs(bool),
    Compare(Vec<usize>),
    Reload,
    /// "Open in Xero": the selected document's page in the browser (`o` on a document; a link, no API call).
    Open,
    /// Cosmetic (theme spec v2 §2.7): switch this session's theme (built-in or user; config untouched).
    Theme(String),
    /// Cosmetic: override the scope accent (a family or #hex; `overlay.tokens.accent`).
    Accent(String),
    /// Cosmetic: set overlay tokens (`token=value`).
    OverlaySet(Vec<(String, String)>),
    /// Show a manifest (None = main).
    Manifest(Option<String>),
    /// Tab 7's mode (entries, issues, diff, log).
    Mode(crate::manifest_tab::Mode),
    /// Cosmetic: clear the named overlay tokens, or all when empty.
    OverlayClear(Vec<String>),
    /// Overlays, narration and walkthroughs (`hint`, `highlight`, `spotlight`, `clear-hints`, `narrate`,
    /// `walkthrough …`).
    Guide(crate::overlay::GuideVerb),
    /// Cosmetic (walkthroughs §4): open an explainer panel (None closes it).
    Viz(Option<Box<ownpurse_core::viz::Spec>>),
}

impl Verb {
    /// Cosmetic verbs move no focus and touch no input: never refused for `user_busy`.
    pub fn cosmetic(&self) -> bool {
        match self {
            Verb::Guide(g) => g.cosmetic(),
            v => matches!(
                v,
                Verb::Theme(_)
                    | Verb::Accent(_)
                    | Verb::OverlaySet(_)
                    | Verb::OverlayClear(_)
                    | Verb::Viz(_)
            ),
        }
    }
}

fn usage(msg: impl Into<String>) -> (String, String) {
    ("usage".into(), msg.into())
}

impl App {
    // ── what blocks a send ───────────────────────────────────────────────────────────────────────

    /// What the user is in right now, if it should refuse an agent send: an editor, overlay or
    /// confirm band. The text completes "The user is … Nothing was changed."
    pub fn agent_busy(&self) -> Option<String> {
        if let Some(p) = &self.ui.prompt {
            return Some(
                match p {
                    Prompt::Filter(_) => "typing in the filter",
                    Prompt::AsOf(_) => "typing an as-of date",
                    Prompt::Range(_) => "typing a date range",
                }
                .into(),
            );
        }
        if let Some(o) = &self.ui.overlay {
            return Some(
                match o {
                    Overlay::Help => "reading help",
                    Overlay::Palette { .. } => "in the command palette",
                    Overlay::Picker { .. } => "in the scope picker",
                }
                .into(),
            );
        }
        if self.ui.band.is_some() {
            return Some("looking at a confirm band".into());
        }
        if self.ui.tab == Tab::Classify
            && let Some(b) = self.ui.classify.busy()
        {
            return Some(b.into());
        }
        None
    }

    // ── verbs ────────────────────────────────────────────────────────────────────────────────────

    /// Resolve one checked verb (`core::bridge::check_verb` passed) against this model.
    pub fn parse_verb(&self, words: &[String]) -> Result<Verb, (String, String)> {
        let arg = words.get(1).map(String::as_str).unwrap_or("");
        Ok(match words[0].as_str() {
            "tab" => Verb::Tab(match arg.to_lowercase().as_str() {
                "1" | "overview" => Tab::Overview,
                "2" | "ledger" => Tab::Ledger,
                "3" | "intercompany" | "ic" => Tab::Intercompany,
                "4" | "activity" => Tab::Activity,
                "5" | "system" => Tab::System,
                "6" | "classify" => Tab::Classify,
                "7" | "manifest" => Tab::Manifest,
                _ => {
                    return Err(usage(format!(
                        "Unknown tab `{arg}`: 1–7 or overview, ledger, intercompany, activity, system, classify, manifest."
                    )));
                }
            }),
            "hint" | "highlight" | "spotlight" | "clear-hints" | "narrate" | "walkthrough" => {
                Verb::Guide(self.parse_guide_verb(words)?)
            }
            "manifest" => Verb::Manifest((arg != "main").then(|| arg.to_string())),
            "mode" => Verb::Mode(match arg {
                "entries" => crate::manifest_tab::Mode::Entries,
                "issues" => crate::manifest_tab::Mode::Issues,
                "diff" => crate::manifest_tab::Mode::Diff,
                "log" => crate::manifest_tab::Mode::Log,
                _ => {
                    return Err(usage(format!(
                        "Unknown mode `{arg}`: entries, issues, diff or log."
                    )));
                }
            }),
            "scope" => Verb::Scope(self.parse_scope(arg)?),
            "asof" => {
                if arg.len() != 10 || ownpurse_core::dates::parse_iso(arg).is_none() {
                    return Err(usage(format!(
                        "as of needs a date YYYY-MM-DD, got `{arg}`."
                    )));
                }
                Verb::AsOf(arg.to_string())
            }
            "filter" => Verb::Filter(arg.to_string()),
            "clear-filter" => Verb::ClearFilter,
            "select" => Verb::Select(arg.to_string()),
            "drill" => Verb::Drill,
            "up" => Verb::Up,
            "back" => Verb::HistBack,
            "forward" => Verb::HistForward,
            "caption" if arg == "clear" => Verb::Caption(None),
            "caption" => {
                if let Some(e) = crate::history::caption_error(arg) {
                    return Err(("bad_caption".into(), e));
                }
                Verb::Caption(Some(arg.to_string()))
            }
            "label" => {
                if let Some(e) = crate::history::caption_error(arg).filter(|_| !arg.is_empty()) {
                    return Err(("usage".into(), e.replace("caption", "label")));
                }
                Verb::Label(arg.trim().to_string())
            }
            "basis" => Verb::Basis(match arg {
                "year" => false,
                "history" | "full" => true,
                _ => return Err(usage(format!("basis is `year` or `history`, got `{arg}`."))),
            }),
            "differs" => Verb::Differs(match arg {
                "on" | "true" | "1" => true,
                "off" | "false" | "0" => false,
                _ => return Err(usage(format!("differs is `on` or `off`, got `{arg}`."))),
            }),
            "compare" => {
                let mut orgs = Vec::new();
                for a in arg.split(',').map(str::trim).filter(|a| !a.is_empty()) {
                    match self.model.orgs.iter().position(|o| o.alias == a) {
                        Some(i) if !orgs.contains(&i) => orgs.push(i),
                        Some(_) => {}
                        None => return Err(usage(format!("Unknown org `{a}`."))),
                    }
                }
                if !(2..=4).contains(&orgs.len()) {
                    return Err(usage("compare takes 2–4 orgs, comma-separated."));
                }
                Verb::Compare(orgs)
            }
            "reload" => Verb::Reload,
            "open" => Verb::Open,
            "viz" if arg == "close" => Verb::Viz(None),
            "viz" => Verb::Viz(Some(Box::new(
                ownpurse_core::viz::Spec::parse(arg)
                    .map_err(|e| (e.code.to_string(), e.message))?,
            ))),
            "theme" => {
                let user = self.user_config.themes.contains_key(arg);
                if crate::theme::Theme::named(arg).is_none() && !user {
                    let mut names: Vec<String> = crate::theme::def::BUILTINS
                        .iter()
                        .map(|s| s.to_string())
                        .collect();
                    names.extend(self.user_config.themes.keys().cloned());
                    return Err((
                        "bad_token".into(),
                        format!("Unknown theme `{arg}`. Themes: {}.", names.join(", ")),
                    ));
                }
                if user && let Some(f) = self.user_config.themes.get(arg) {
                    crate::theme::user::parse(&f.file, &f.name, &f.text).map_err(|i| {
                        (
                            "bad_token".to_string(),
                            format!("Theme `{arg}` is invalid: {i}."),
                        )
                    })?;
                }
                Verb::Theme(arg.to_string())
            }
            "accent" => {
                self.check_overlay_entry("accent", arg)?;
                Verb::Accent(arg.to_string())
            }
            "overlay" => match arg {
                "set" => {
                    let mut pairs = Vec::new();
                    for w in &words[2..] {
                        let (k, v) = w.split_once('=').unwrap_or((w, ""));
                        self.check_overlay_entry(k.trim(), v.trim())?;
                        pairs.push((k.trim().to_string(), v.trim().to_string()));
                    }
                    Verb::OverlaySet(pairs)
                }
                _ => {
                    let names: Vec<String> = words[2..].to_vec();
                    for n in &names {
                        if !self.ui.theme_overlay.contains_key(n) {
                            self.check_overlay_name(n)?;
                        }
                    }
                    Verb::OverlayClear(names)
                }
            },
            other => return Err(usage(format!("Unknown verb `{other}`."))),
        })
    }

    fn parse_scope(&self, arg: &str) -> Result<Scope, (String, String)> {
        if arg == "all" {
            return Ok(Scope::All);
        }
        if let Some(g) = arg.strip_prefix("group:") {
            return self
                .model
                .groups
                .iter()
                .position(|x| x.name == g)
                .map(Scope::Group)
                .ok_or_else(|| usage(format!("Unknown group `{g}`.")));
        }
        self.model
            .orgs
            .iter()
            .position(|o| o.alias == arg)
            .map(Scope::Org)
            .ok_or_else(|| {
                usage(format!(
                    "Unknown scope `{arg}`: all, group:<name> or one of {}.",
                    self.model
                        .orgs
                        .iter()
                        .map(|o| o.alias.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                ))
            })
    }

    /// Apply one verb (called from `update` for `Msg::Agent`): the same actions, clicks and prompt edits
    /// a key or a click runs. Sets `agent.outcome` to what changed (past tense) or the error.
    /// Whether a verb means something on the current screen (a verb that can't show its change is
    /// refused, never applied silently): Err(message) → `not_applicable`, exit 2.
    pub fn verb_applies(&self, v: &Verb) -> Result<(), String> {
        let screen = || crate::views::crumbs(self).join(" › ");
        match v {
            Verb::Filter(_) | Verb::ClearFilter => self
                .filter_applies()
                .map_err(|s| format!("A filter doesn't apply on {s} ({}).", screen())),
            Verb::Basis(_) => {
                let ok = self.ui.tab == Tab::Ledger
                    && (self.cmp_active()
                        || (self.ledger_org().is_some() && self.ledger_level() <= 2));
                if ok {
                    Ok(())
                } else {
                    Err(format!(
                        "The basis applies on the Ledger trial balance, postings and Compare, not on {}.",
                        screen()
                    ))
                }
            }
            Verb::Differs(_) => {
                let ok = self.ui.tab == Tab::Ledger
                    && !self.cmp_active()
                    && self.ledger_org().is_some()
                    && self.ledger_level() == 1;
                if ok {
                    Ok(())
                } else {
                    Err(format!(
                        "`differs` applies on the Ledger trial balance only, not on {}.",
                        screen()
                    ))
                }
            }
            // System: drill opens a write package's operations (Enter); a link opens with `open`
            Verb::Drill if self.ui.tab == Tab::System => {
                match crate::views::system::selected(self).map(|i| i.key) {
                    Some(k) if k.starts_with("pkg:") => Ok(()),
                    Some(k) if k.starts_with("pkgop:") || k.starts_with("link:") => Err(format!(
                        "`drill` lists a write package's operations; this row is a link: send `open` to open it ({}).",
                        screen()
                    )),
                    _ => Err(format!(
                        "`drill` applies to a write package row on System (Writes), not this row ({}).",
                        screen()
                    )),
                }
            }
            Verb::Up if self.ui.tab == Tab::System && self.ui.system.open_pkg.is_none() => Err(
                format!("Nothing to go up from on {}.", screen()),
            ),
            Verb::Open => self.open_link().map(|_| ()).ok_or_else(|| {
                format!(
                    "`open` needs an open document with a Xero link (Ledger or Intercompany, `drill` into one), not {}.",
                    screen()
                )
            }),
            // the header names the compared orgs while Compare is open: a scope change wouldn't show
            Verb::Scope(_) if self.cmp_active() => Err(format!(
                "Compare is open on {}: the scope applies after `up` closes it.",
                self.ui
                    .cmp
                    .as_ref()
                    .map(|c| c
                        .orgs
                        .iter()
                        .map(|&i| self.org_alias(i))
                        .collect::<Vec<_>>()
                        .join(", "))
                    .unwrap_or_default()
            )),
            _ => Ok(()),
        }
    }

    /// A whole send in ONE update (so one revision): each verb through `apply_verb`, with the frame's rows
    /// refreshed between verbs (an offscreen render; `select` reads the rows the previous verb produced).
    /// Atomic: if any verb is refused, the view is put back and nothing is applied (caption included).
    pub(crate) fn apply_batch(&mut self, verbs: Vec<Verb>, fx: &mut Vec<Effect>) {
        let ui0 = self.ui.clone();
        let label0 = self.label.clone();
        let mut changes = Vec::new();
        let mut err = None;
        self.agent.effects.clear();
        self.agent.failed = None;
        for (i, v) in verbs.into_iter().enumerate() {
            if i > 0 {
                let (w, h) = self.ui.size;
                let dirty = self.dirty;
                self.prepare();
                crate::snapshot::render(self, w, h);
                self.dirty = dirty;
            }
            self.apply_verb(v, fx);
            match self.agent.outcome.take() {
                Some(Ok(Some(d))) => {
                    self.agent.effects.push(Some(d.clone()));
                    changes.push(d)
                }
                Some(Ok(None)) | None => self.agent.effects.push(None),
                Some(Err(e)) => {
                    let help = self.agent.failed.take().and_then(|(_, h)| h);
                    self.agent.failed = Some((i, help));
                    err = Some(e);
                    break;
                }
            }
        }
        self.agent.batch = Some(match err {
            Some((code, msg)) => {
                self.ui = ui0;
                self.label = label0;
                fx.retain(|e| !matches!(e, Effect::Reload | Effect::Open(_)));
                Err((code, msg))
            }
            None => Ok(changes),
        });
    }

    pub(crate) fn apply_verb(&mut self, v: Verb, fx: &mut Vec<Effect>) {
        if let Verb::Guide(g) = v {
            let r = self.apply_guide(g, fx);
            self.prepare();
            self.agent.outcome = Some(r);
            return;
        }
        if let Err(msg) = self.verb_applies(&v) {
            self.agent.outcome = Some(Err(("not_applicable".into(), msg)));
            return;
        }
        let before = ViewSnap::of(&self.ui);
        let selected_before = self.selected_anchor();
        let res: Result<(), (String, String)> = match &v {
            Verb::Guide(_) => Ok(()),
            Verb::Tab(t) => {
                self.run_action(
                    match t {
                        Tab::Overview => "tab.overview",
                        Tab::Ledger => "tab.ledger",
                        Tab::Intercompany => "tab.intercompany",
                        Tab::Activity => "tab.activity",
                        Tab::System => "tab.system",
                        Tab::Classify => "tab.classify",
                        Tab::Manifest => "tab.manifest",
                    },
                    fx,
                );
                Ok(())
            }
            // the scope picker's choice (`picker.choose` → set_scope)
            Verb::Scope(sc) => {
                self.set_scope_pub(sc.clone());
                Ok(())
            }
            // the as-of prompt: open, replace the text, apply
            Verb::AsOf(d) => {
                self.run_action("asof.edit", fx);
                self.ui.prompt = Some(Prompt::AsOf(String::new()));
                for c in d.chars() {
                    self.type_char_pub(c);
                }
                self.run_action("prompt.apply", fx);
                Ok(())
            }
            // the filter prompt: clear, open, type, apply
            Verb::Filter(t) => {
                self.run_action("filter.clear", fx);
                self.run_action("filter.open", fx);
                for c in t.chars().filter(|c| !c.is_control()) {
                    self.type_char_pub(c);
                }
                self.run_action("prompt.apply", fx);
                Ok(())
            }
            Verb::ClearFilter => {
                self.run_action("filter.clear", fx);
                Ok(())
            }
            Verb::Select(q) => match self.find_visible_row(q) {
                // already the selected row: nothing to do (no invisible state change, no rev bump)
                Some(t) if self.is_selected(t) => Ok(()),
                Some(t) => {
                    self.click_pub(t, fx);
                    Ok(())
                }
                // anywhere else in the list: move the cursor there as the user's keys would (the view
                // scrolls to it, and a following `drill` acts on it)
                None => match self.find_list_row(q) {
                    Some(idx) if self.move_cursor_to(idx, fx) => Ok(()),
                    Some(_) => Err((
                        "not_found".into(),
                        format!("Row `{q}` is in the list but the cursor can't reach it here."),
                    )),
                    // the Ledger across several orgs lists nothing to select (it asks for one org)
                    None if self.ui.tab == Tab::Ledger
                        && !matches!(self.ui.scope, Scope::Org(_)) =>
                    {
                        self.agent.failed = Some((
                            0,
                            Some("Ledger needs a single-org scope; send `scope <org>` first (e.g. `scope <org> select <row>` in one send).".into()),
                        ));
                        Err((
                            "not_found".into(),
                            format!(
                                "The Ledger lists no rows for scope {}: it shows one org at a time.",
                                self.scope_label(&self.ui.scope)
                            ),
                        ))
                    }
                    None => Err((
                        "not_found".into(),
                        format!(
                            "No row in the current list matches `{q}` ({} rows; the filter and scope narrow it).",
                            self.total_rows()
                        ),
                    )),
                },
            },
            Verb::Drill => {
                self.run_action("drill.in", fx);
                Ok(())
            }
            Verb::Up => {
                self.run_action("drill.back", fx);
                Ok(())
            }
            Verb::HistBack | Verb::HistForward => {
                let back = matches!(v, Verb::HistBack);
                self.run_action(
                    if back {
                        "history.back"
                    } else {
                        "history.forward"
                    },
                    fx,
                );
                if ViewSnap::of(&self.ui) == before {
                    // never silent: the answer says `no_history`, the footer says it too (no new revision)
                    self.agent_no_history = Some(if back {
                        "nothing to go back to"
                    } else {
                        "nothing to go forward to"
                    });
                } else {
                    self.ui.note = None; // the agent's notice says it
                }
                Ok(())
            }
            Verb::Caption(t) => {
                self.ui.caption = t.as_ref().map(|t| crate::history::Caption {
                    text: t.clone(),
                    by: self.agent_client.clone().unwrap_or_else(|| "agent".into()),
                    rev: self.rev + 1,
                    fresh: true,
                });
                Ok(())
            }
            Verb::Label(l) => {
                self.label = (!l.is_empty()).then(|| l.clone());
                Ok(())
            }
            Verb::Manifest(name) => self
                .switch_manifest(name.as_deref(), false)
                .map_err(|e| (e.code.to_string(), e.message)),
            Verb::Mode(m) => {
                if self.ui.tab != Tab::Manifest || self.ui.manifest.is_none() {
                    Err((
                        "not_applicable".into(),
                        "`mode` applies on tab 7 with a manifest shown.".into(),
                    ))
                } else {
                    self.ui.manifest_tab.mode = *m;
                    self.ui.manifest_tab.diff = None;
                    self.ui.manifest_tab.sel = 0;
                    Ok(())
                }
            }
            Verb::Basis(h) => {
                if self.ui.full_history != *h {
                    self.run_action("basis.toggle", fx);
                }
                Ok(())
            }
            Verb::Differs(on) => {
                if self.ui.differs_only != *on {
                    self.run_action("filter.differs", fx);
                }
                Ok(())
            }
            Verb::Compare(orgs) => {
                if self.ui.tab != Tab::Ledger {
                    self.run_action("tab.ledger", fx);
                }
                self.compare_start(orgs.clone());
                Ok(())
            }
            Verb::Reload => {
                self.run_action("record.reload", fx);
                Ok(())
            }
            Verb::Open => {
                self.run_action(
                    if self.ui.tab == Tab::System {
                        "link.open"
                    } else {
                        "doc.open_xero"
                    },
                    fx,
                );
                Ok(())
            }
            Verb::Theme(_) | Verb::Accent(_) | Verb::OverlaySet(_) | Verb::OverlayClear(_) => {
                self.apply_cosmetic(&v);
                Ok(())
            }
            Verb::Viz(None) => {
                self.viz_close();
                Ok(())
            }
            Verb::Viz(Some(spec)) => {
                let before = self.ui.viz.clone();
                self.viz_open((**spec).clone());
                match self.viz_model() {
                    Some(Err(e)) => {
                        let e = e.clone();
                        self.ui.viz = before;
                        self.viz_view = None;
                        self.prepare_viz();
                        Err(("not_found".into(), format!("viz: {e}")))
                    }
                    _ => Ok(()),
                }
            }
        };
        // a verb never leaves an editor open behind it (a cosmetic one never touches it)
        if !v.cosmetic() && matches!(self.ui.prompt, Some(Prompt::AsOf(_) | Prompt::Filter(_))) {
            self.ui.prompt = None;
        }
        // the description reads the models of the view the verb landed on (a history move to a document,
        // for one): prepare them first (cached, so a no-op when nothing moved)
        self.prepare();
        self.agent.outcome = Some(match res {
            Err(e) => Err(e),
            // selecting the row that is already selected changes nothing visible
            Ok(()) if matches!(v, Verb::Select(_)) && self.selected_anchor() == selected_before => {
                Ok(None)
            }
            Ok(()) => Ok(self.describe(&v, &before, fx)),
        });
    }

    /// What a verb changed, past tense; None when it changed nothing.
    fn describe(&self, v: &Verb, before: &ViewSnap, fx: &[Effect]) -> Option<String> {
        if matches!(v, Verb::Open) {
            return fx.iter().find_map(|e| match e {
                Effect::Open(u) => Some(format!("opened {u} in the browser")),
                _ => None,
            });
        }
        if matches!(v, Verb::Reload) {
            return fx
                .contains(&Effect::Reload)
                .then(|| "reloaded the record".into());
        }
        match v {
            Verb::Caption(Some(t)) => return Some(format!("caption “{t}”")),
            Verb::Caption(None) => return Some("cleared the caption".into()),
            Verb::Label(l) => return Some(format!("label “{l}”")),
            Verb::Manifest(Some(n)) => return Some(format!("showing manifest {n}")),
            Verb::Manifest(None) => return Some("showing main".into()),
            Verb::Mode(m) => return Some(format!("manifest mode {}", m.word())),
            Verb::Theme(n) => return Some(format!("theme {n}")),
            Verb::Accent(a) => return Some(format!("accent {a}")),
            Verb::OverlaySet(p) => {
                return Some(format!(
                    "overlay {}",
                    p.iter()
                        .map(|(k, v)| format!("{k}={v}"))
                        .collect::<Vec<_>>()
                        .join(" ")
                ));
            }
            Verb::OverlayClear(n) if n.is_empty() => {
                return Some("cleared the theme overlay".into());
            }
            Verb::OverlayClear(n) => return Some(format!("cleared overlay {}", n.join(" "))),
            Verb::Viz(Some(_)) => {
                return self
                    .viz_model()
                    .and_then(|m| m.as_ref().ok())
                    .map(|m| format!("opened explainer: {}", m.title()));
            }
            Verb::Viz(None) => return Some("closed the explainer".into()),
            _ => {}
        }
        if ViewSnap::of(&self.ui) == *before {
            return None;
        }
        Some(match v {
            Verb::Tab(t) => format!("opened {}", t.label()),
            Verb::Scope(sc) => format!("scope {}", self.scope_label(sc)),
            Verb::AsOf(d) => format!("as of {d}"),
            // typographic quotes: the CLI's text output never doubles them (TOON quoting)
            Verb::Filter(t) => format!("filter “{t}” on {}", self.ui.tab.label()),
            Verb::ClearFilter => "cleared the filter".into(),
            Verb::Select(_) => format!(
                "selected {}",
                self.selected_label().unwrap_or_else(|| "a row".into())
            ),
            Verb::Drill => self.drilled_to(),
            Verb::Up => "went up a level".into(),
            Verb::HistBack => format!("back → {}", self.view_summary()),
            Verb::HistForward => format!("forward → {}", self.view_summary()),
            Verb::Caption(_)
            | Verb::Label(_)
            | Verb::Open
            | Verb::Manifest(_)
            | Verb::Mode(_)
            | Verb::Theme(_)
            | Verb::Accent(_)
            | Verb::OverlaySet(_)
            | Verb::OverlayClear(_)
            | Verb::Guide(_)
            | Verb::Viz(_) => String::new(),
            Verb::Basis(h) => if *h { "full history" } else { "year basis" }.into(),
            Verb::Differs(on) => if *on {
                "only accounts that differ"
            } else {
                "all accounts"
            }
            .into(),
            Verb::Compare(orgs) => format!(
                "compared {}",
                orgs.iter()
                    .map(|&i| self.model.orgs[i].alias.as_str())
                    .collect::<Vec<_>>()
                    .join(" · ")
            ),
            Verb::Reload => "reloaded the record".into(),
        })
    }

    fn drilled_to(&self) -> String {
        match self.ui.tab {
            Tab::System => match self
                .ui
                .system
                .open_pkg
                .as_deref()
                .and_then(|h| self.model.system.packages.iter().find(|p| p.hash == h))
            {
                Some(p) => format!(
                    "listed the {} operation{} of {}",
                    p.op_list.len(),
                    if p.op_list.len() == 1 { "" } else { "s" },
                    p.title
                ),
                None => "folded the package's operations".into(),
            },
            Tab::Ledger if self.cmp_active() => "drilled to postings".into(),
            Tab::Ledger => match self.ledger_level() {
                2 => "drilled to postings".into(),
                3 => "opened the document".into(),
                _ => "drilled to Ledger".into(),
            },
            Tab::Intercompany => match self.ic_level() {
                2 => "opened the bridge".into(),
                3 => "opened the document".into(),
                _ => "drilled in".into(),
            },
            _ => "drilled in".into(),
        }
    }

    // ── what the user sees ───────────────────────────────────────────────────────────────────────────

    /// A request arrived (any op): the header shows the agent as connected.
    pub fn agent_seen(&mut self) {
        if !self.agent.connected {
            self.agent.connected = true;
            self.dirty = true;
        }
        self.agent.last_seen = self.now;
    }

    /// A send changed the view: announce it (latest send) and keep the view before the first unseen change.
    pub fn agent_announce(&mut self, changes: Vec<String>, before: ViewSnap) {
        let unseen = matches!(self.agent.notice, Some(Notice::Changed(_)));
        // a send that moved nothing (a caption, a label) offers no way back
        let moved = ViewSnap::of(&self.ui) != before;
        if moved && (!unseen || self.agent.undo.is_none()) {
            self.agent.undo = Some(before);
        }
        self.agent.notice = Some(Notice::Changed(changes));
        self.agent.acted = true;
        self.dirty = true;
    }

    /// A screenshot left the terminal: `◆ agent` until the user's next key.
    pub fn agent_acted(&mut self) {
        if !self.agent.acted {
            self.agent.acted = true;
            self.dirty = true;
        }
    }

    /// A stale send was refused: the dim notice, unless the user is in an overlay (then when it closes).
    pub fn agent_skipped(&mut self) {
        if self.agent_busy().is_some() {
            self.agent.skipped_pending = true;
        } else {
            self.agent.notice = Some(Notice::Skipped);
            self.agent.undo = None;
            self.dirty = true;
        }
    }

    /// the user's key (before it acts). Returns true when the key is consumed: Esc dismisses the notice only;
    /// `` ` `` goes back to the view before the agent's change. Any other key dismisses and acts.
    pub(crate) fn agent_key(&mut self, esc: bool, backtick: bool) -> bool {
        let had = self.agent.acted || self.agent.notice.is_some();
        self.agent.acted = false;
        let mut consumed = false;
        // a classify write's notice stays until the view changes (Esc dismisses it; `U` undoes it)
        if matches!(self.agent.notice, Some(Notice::Wrote(_))) && !esc {
            if had {
                self.dirty = true;
            }
            return false;
        }
        if self.agent.notice.is_some() && self.agent_busy().is_none() {
            if backtick && let Some(snap) = self.agent.undo.take() {
                snap.restore(&mut self.ui);
                consumed = true;
            } else if esc {
                consumed = true;
            }
            self.agent.notice = None;
            self.agent.undo = None;
        }
        if had {
            self.dirty = true;
        }
        consumed
    }

    /// After every `update`: a refusal that waited for an overlay to close is shown now; a connection
    /// with no request for `CONNECTED_SECS` is no longer shown (on the minute tick only).
    pub(crate) fn agent_after_update(&mut self) {
        if self.agent.skipped_pending && self.agent_busy().is_none() {
            self.agent.skipped_pending = false;
            self.agent.notice = Some(Notice::Skipped);
            self.dirty = true;
        }
        if self.agent.connected && self.now - self.agent.last_seen > CONNECTED_SECS {
            self.agent.connected = false;
            self.dirty = true;
        }
    }

    /// The header mark: None, Some(false) connected, Some(true) acted.
    pub fn agent_mark(&self) -> Option<bool> {
        if self.agent.acted {
            Some(true)
        } else if self.agent.connected {
            Some(false)
        } else {
            None
        }
    }

    /// The status strip's notice: (text, acted). NO_COLOR/ASCII: `* agent: …` and `agent: change skipped…`.
    pub fn agent_notice_text(&self) -> Option<(String, bool)> {
        self.agent_notice_parts()
            .map(|(head, body, tail, acted)| (format!("{head}{body}{tail}"), acted))
    }

    /// The notice as (lead, what changed, tail, acted): `◆ agent: ` · changes · `` · ` back ``. NO_COLOR/ASCII
    /// leads: `* agent: ` (acted) and `- agent: ` (skipped).
    fn agent_notice_parts(&self) -> Option<(String, String, String, bool)> {
        let plain =
            self.ui.theme.ascii || matches!(self.ui.theme.mode, crate::theme::Mode::NoColor);
        match self.agent.notice.as_ref()? {
            Notice::Changed(c) => Some((
                format!("{} agent: ", if plain { "*" } else { "◆" }),
                c.join(" · "),
                if self.agent.undo.is_some() {
                    " · ` back".into()
                } else {
                    String::new()
                },
                true,
            )),
            Notice::Skipped => Some((
                format!("{} agent: ", if plain { "-" } else { "◇" }),
                "change skipped, you moved first".into(),
                String::new(),
                false,
            )),
            Notice::Wrote(n) => Some((
                format!("{} agent: ", if plain { "*" } else { "◆" }),
                n.fitted("", usize::MAX, false)
                    .trim_end_matches(" · U undo")
                    .to_string(),
                " · U undo".into(),
                true,
            )),
            Notice::Undone(t) => Some((String::new(), t.clone(), String::new(), false)),
        }
    }

    /// The notice fitted to `w` cells (theme glyphs applied): the lead and the `` · ` back `` hint always
    /// stay; what changed loses its middle.
    pub fn agent_notice_fitted(&self, w: usize) -> Option<(String, bool)> {
        use crate::layout::width;
        let th = &self.ui.theme;
        // a classify write's notice fits by its own segments (never cut mid-word)
        if let Some(Notice::Wrote(n)) = &self.agent.notice {
            let plain =
                self.ui.theme.ascii || matches!(self.ui.theme.mode, crate::theme::Mode::NoColor);
            let lead = th.text(&format!("{} agent: ", if plain { "*" } else { "◆" }));
            return Some((n.fitted(&lead, w, th.ascii), true));
        }
        let (head, body, tail, acted) = self.agent_notice_parts()?;
        let (head, body, tail) = (th.text(&head), th.text(&body), th.text(&tail));
        let full = format!("{head}{body}{tail}");
        if width(&full) <= w {
            return Some((full, acted));
        }
        let ell = th.glyphs().ellipsis;
        let room = w.saturating_sub(width(&head) + width(&tail));
        Some((
            format!("{head}{}{tail}", middle_cut(&body, room, ell)),
            acted,
        ))
    }

    /// One Activity row (`agent · read state`, `agent · screenshot png`, `agent · send tab=2 scope=design`).
    pub fn agent_log(&mut self, what: &str) {
        // arriving never takes the cursor: a cursor on "the first row" stays on that row, not the new one
        if self.ui.activity.sel.is_none()
            && let Some(first) = crate::views::activity::rows(self).first().map(|r| r.id)
        {
            self.ui.activity.sel = Some(first);
        }
        let id = LOG_ID_BASE + self.agent.log.len() as i64;
        let at = format!(
            "{}.000000Z",
            ownpurse_core::bridge::utc_stamp(self.now).trim_end_matches('Z')
        );
        let mut row = LogRow {
            id,
            at,
            tenant: None,
            org: "agent".into(),
            method: "agent".into(),
            path: what.to_string(),
            endpoint: what.to_string(),
            status: None,
            ms: None,
            day_left: None,
            min_left: None,
            problem: None,
            hay: String::new(),
        };
        row.hay = row.haystack();
        self.agent.log.push(row.clone());
        self.model.activity.log.insert(0, row);
        self.model.activity.generation = crate::activity::next_generation();
        if self.ui.tab == Tab::Activity {
            self.dirty = true;
        }
    }

    /// After a record reload: the agent's Activity rows come back (they live in memory only).
    pub(crate) fn agent_relog(&mut self) {
        if self.agent.log.is_empty() {
            return;
        }
        let mut log = self.agent.log.clone();
        log.reverse();
        log.append(&mut self.model.activity.log);
        log.sort_by(|a, b| b.at.cmp(&a.at).then(b.id.cmp(&a.id)));
        self.model.activity.log = log;
        self.model.activity.generation = crate::activity::next_generation();
    }

    // ── the view state ───────────────────────────────────────────────────────────────────────────

    /// The visible rows: (hit target, anchor, fields, row number), top to bottom, from the last frame's row
    /// targets. The row number is the 1-based position in the screen's whole (filtered) list, the `#n` of
    /// `select`.
    pub fn visible_rows(&self) -> Vec<(Target, Value, Value)> {
        self.visible_rows_numbered()
            .into_iter()
            .map(|(t, a, f, _)| (t, a, f))
            .collect()
    }

    pub fn visible_rows_numbered(&self) -> Vec<(Target, Value, Value, usize)> {
        let mut drawn: Vec<usize> = self
            .hits
            .targets
            .iter()
            .filter_map(|(_, t)| match t {
                Target::Row(i) => Some(*i),
                // Compare registers its rows as cells (one per org column)
                Target::Cell(i, _) if self.cmp_active() => Some(*i),
                _ => None,
            })
            .collect();
        drawn.dedup();
        drawn
            .into_iter()
            .filter_map(|i| {
                let (a, f, n) = self.row_anchor(i)?;
                Some((Target::Row(i), a, f, n + 1))
            })
            .collect()
    }

    /// Rows in the screen's whole (filtered) list, for `view.total_rows`.
    pub fn total_rows(&self) -> usize {
        if self.ui.tab == Tab::Classify {
            return if self.cl_queue() {
                self.cl_rows().rows.len()
            } else {
                0
            };
        }
        if crate::views::activity::owns(self) {
            return crate::views::activity::shown_len(self);
        }
        if crate::views::system::owns(self) {
            return crate::views::system::layout(self).0.len();
        }
        match self.ui.tab {
            Tab::Overview => self.overview_rows().len(),
            Tab::Ledger if self.cmp_active() => match self.ui.cmp.as_ref().map(|c| c.level) {
                Some(2) => 0,
                _ => self.cmp_full_pub().map_or(0, |m| self.cmp_visible(m).len()),
            },
            Tab::Ledger => match self.ledger_level() {
                1 => self.l1_visible().len(),
                2 => self.l2_visible().len(),
                _ => 0,
            },
            Tab::Intercompany => match self.ic_level() {
                1 => self.ic_rows().len(),
                2 => self.ic_items().len(),
                _ => 0,
            },
            Tab::Manifest if self.mf_entries_mode() => self.mf_rows().len(),
            _ => 0,
        }
    }

    pub(crate) fn org_alias(&self, i: usize) -> String {
        self.model
            .orgs
            .get(i)
            .map(|o| o.alias.clone())
            .unwrap_or_default()
    }

    /// The first drawn row's place in the whole list (each view's own scroll; System and Compare register
    /// whole-list positions already).
    fn screen_top(&self) -> usize {
        if self.ui.tab == Tab::Classify {
            return self.cl_top(self.cl_page());
        }
        if crate::views::activity::owns(self) {
            return crate::views::activity::visible_top_pub(self);
        }
        if crate::views::system::owns(self) {
            return 0;
        }
        match self.ui.tab {
            Tab::Overview => self.ui.overview_top,
            Tab::Ledger if self.cmp_active() => 0,
            Tab::Ledger => match self.ledger_level() {
                1 => self.nav().l1_top,
                2 => self.nav().l2_top,
                _ => 0,
            },
            Tab::Intercompany => match self.ic_level() {
                1 => self.ic_nav().l1_top,
                2 => self.ic_nav().item_top,
                _ => 0,
            },
            Tab::Manifest if self.mf_entries_mode() => self.ui.manifest_tab.top,
            _ => 0,
        }
    }

    /// The cursor's place in the whole list.
    fn cursor_abs(&self) -> Option<usize> {
        if self.ui.tab == Tab::Classify {
            return self.cl_queue().then(|| self.cl_cursor()).flatten();
        }
        if crate::views::activity::owns(self) {
            return crate::views::activity::cursor(self);
        }
        if crate::views::system::owns(self) {
            let lay = crate::views::system::layout(self);
            return crate::views::system::cursor(self, &lay.0);
        }
        match self.ui.tab {
            Tab::Overview => self.overview_cursor(),
            Tab::Ledger if self.cmp_active() => match self.ui.cmp.as_ref().map(|c| c.level) {
                Some(2) => self.ui.cmp.as_ref().map(|c| c.l2_idx),
                _ => self.cmp_cursor(),
            },
            Tab::Ledger => match self.ledger_level() {
                1 => self.l1_cursor(),
                2 => self.l2_cursor(),
                _ => None,
            },
            Tab::Intercompany => match self.ic_level() {
                1 => self.ic_l1_cursor(),
                2 => self.ic_item_cursor(),
                _ => None,
            },
            Tab::Manifest if self.mf_entries_mode() => {
                let n = self.mf_rows().len();
                (n > 0).then(|| self.ui.manifest_tab.sel.min(n - 1))
            }
            _ => None,
        }
    }

    /// Drawn row `i` (as its click handler numbers it): anchor, fields, place in the whole list.
    pub(crate) fn row_anchor_pub(&self, i: usize) -> Option<(Value, Value, usize)> {
        self.row_anchor(i)
    }

    pub(crate) fn screen_top_pub(&self) -> usize {
        self.screen_top()
    }

    fn row_anchor(&self, i: usize) -> Option<(Value, Value, usize)> {
        self.row_abs(self.screen_top() + i)
    }

    /// Rows `range` (0-based, end exclusive) of the screen's whole (filtered) list, in screen order,
    /// whether drawn or not: (row number, anchor, fields, selected). Read-only (`ui state --all-rows`).
    pub fn rows_slice(&self, range: std::ops::Range<usize>) -> Vec<(usize, Value, Value, bool)> {
        let cur = self.cursor_abs();
        range
            .filter_map(|n| {
                let (a, f, n) = self.row_abs(n)?;
                Some((n + 1, a, f, cur == Some(n)))
            })
            .collect()
    }

    /// Whole-list position `i`: anchor, fields (one per column the screen's table has; `field_key` of each
    /// header) and the position.
    fn row_abs(&self, i: usize) -> Option<(Value, Value, usize)> {
        use crate::views::{activity, intercompany as ic, ledger, overview, system};
        if self.ui.tab == Tab::Classify {
            return self.classify_row_abs(i);
        }
        if activity::owns(self) {
            let n = i;
            if self.ui.activity.grouped {
                let eps = activity::endpoints(self);
                let e = eps.get(n)?;
                let cells = activity::ep_cells(self, std::slice::from_ref(e));
                return Some((
                    json!({"endpoint": e.endpoint}),
                    fields(&activity::EP_COLS, &cells[0], &[]),
                    n,
                ));
            }
            let rows = activity::rows(self);
            let r = *rows.get(n)?;
            let cells = activity::log_cells(self, &[r], true);
            return Some((
                json!({"log_id": r.id}),
                fields(&activity::LOG_COLS, &cells[0], &[("at", json!(r.at))]),
                n,
            ));
        }
        if system::owns(self) {
            let lay = system::layout(self);
            let it = lay.0.get(i)?;
            return Some((
                json!({"system_item": it.key}),
                fields(
                    it.section.cols(),
                    &it.cells,
                    &[("section", json!(it.section.title()))],
                ),
                i,
            ));
        }
        match self.ui.tab {
            Tab::Overview => {
                let ov = self.overview.as_deref()?;
                let n = i;
                let rows = self.overview_rows();
                let o = &self.model.orgs[*rows.get(n)?];
                let cells = overview::rows(self, ov);
                Some((
                    json!({"org": o.alias}),
                    fields(
                        &overview::COLS,
                        cells.get(n)?,
                        &[
                            ("name", json!(o.name)),
                            ("calls_left_estimated", json!(!o.left_known)),
                        ],
                    ),
                    n,
                ))
            }
            Tab::Ledger if self.cmp_active() => self.compare_anchor(i),
            Tab::Ledger => {
                let org = self.ledger_org().map(|o| self.org_alias(o));
                match self.ledger_level() {
                    1 => {
                        let m = self.l1_model()?;
                        let vis = self.l1_visible();
                        let n = i;
                        let idx = *vis.get(n)?;
                        let r = &m.rows[idx];
                        let diff = self.ui.differs_only;
                        let cells = ledger::l1_cells(m, &[idx], diff);
                        let cols: &[crate::layout::ColSpec] = if diff {
                            &ledger::L1_DIFF_COLS
                        } else {
                            &ledger::L1_COLS
                        };
                        Some((
                            json!({"org": org, "account_code": r.code, "account_id": r.account_id}),
                            fields(
                                cols,
                                &cells[0],
                                &[
                                    ("differs", json!(r.differs)),
                                    ("balance", json!(r.balance.to_string())),
                                ],
                            ),
                            n,
                        ))
                    }
                    2 => {
                        let m = self.l2_model()?;
                        let n = i;
                        let idx = *self.l2_visible().get(n)?;
                        let r = &m.rows[idx];
                        let code = self
                            .l1_model()
                            .and_then(|l1| l1.rows.iter().find(|x| x.account_id == m.account_id))
                            .map(|x| x.code.clone());
                        let cells = ledger::l2_cells(m, &[idx]);
                        Some((
                            json!({
                                "org": org,
                                "doc_type": r.source,
                                "doc_id": r.source_id,
                                "line_id": r.line,
                                "account_code": code,
                            }),
                            fields(
                                &ledger::L2_COLS,
                                &cells[0],
                                &[
                                    // signed: debit positive, credit negative (the code's convention)
                                    ("amount", json!(r.amount.to_string())),
                                    ("balance_value", json!(r.balance.to_string())),
                                ],
                            ),
                            n,
                        ))
                    }
                    _ => None,
                }
            }
            Tab::Intercompany => match self.ic_level() {
                1 => {
                    let rows = self.ic_rows();
                    let n = i;
                    let p = *rows.get(n)?;
                    let r = IcRow::of(p);
                    let cells = ic::l1_cells(&[p]);
                    Some((
                        json!({"pair": r.id}),
                        fields(&ic::L1_COLS, &cells[0], &[]),
                        n,
                    ))
                }
                2 => {
                    let items = self.ic_items();
                    let n = i;
                    let it = items.get(n)?;
                    let cells = ic::item_cells(std::slice::from_ref(it));
                    Some((
                        json!({
                            "pair": self.ic_nav().pair,
                            "org": s(&it["org"]),
                            "doc_type": s(&it["source"]),
                            "doc_id": s(&it["source_id"]),
                        }),
                        fields(&ic::ITEM_COLS, &cells[0], &[]),
                        n,
                    ))
                }
                _ => None,
            },
            Tab::Manifest if self.mf_entries_mode() => {
                let rows = self.mf_rows();
                let r = rows.get(i)?;
                let e = &r.entry;
                Some((
                    json!({"entry": e.seq, "target": e.id}),
                    json!({"seq": e.seq, "org": e.org, "entity": e.entity, "target": e.id,
                           "change": r.changes, "issues": r.badge, "by": e.actor}),
                    i,
                ))
            }
            _ => None,
        }
    }

    /// A Queue row: the anchor (`org`, `doc_type`, `doc_id`, `line_id` and the whole `anchor`), every column
    /// as shown, `amount_value` and `conf_value` (a number), and the raw labels.
    fn classify_row_abs(&self, i: usize) -> Option<(Value, Value, usize)> {
        use ownpurse_core::classify::Anchor;
        if !self.cl_queue() {
            return None;
        }
        let rows = self.cl_rows().rows;
        let r = rows.get(i)?;
        let cells = crate::views::classify::row_cells(std::slice::from_ref(r), self.ui.theme.ascii);
        let anchor = match &r.anchor {
            Anchor::Line {
                org,
                doc_type,
                doc_id,
                line_id,
            } => json!({
                "anchor": r.anchor.to_string(),
                "org": org,
                "doc_type": doc_type.code(),
                "doc_id": doc_id,
                "line_id": line_id,
            }),
            a => json!({"anchor": a.to_string()}),
        };
        let cell = |c: &Option<ownpurse_core::classify::Cell>| {
            c.as_ref().map_or(Value::Null, |c| json!(c.value))
        };
        let conf_value = r
            .confidence
            .and_then(|c| c.round_dp(2).to_string().parse::<serde_json::Number>().ok())
            .map_or(Value::Null, Value::Number);
        Some((
            anchor,
            fields(
                &crate::views::classify::COLS,
                &cells[0],
                &[
                    ("conf_value", conf_value),
                    ("project", cell(&r.suggestion.project)),
                    ("bearer", cell(&r.suggestion.bearer)),
                    ("treatment", cell(&r.suggestion.treatment)),
                    ("source", json!(r.glyph.char().to_string())),
                    ("description", json!(r.description)),
                    ("stale", json!(r.stale)),
                    ("open", json!(r.open)),
                ],
            ),
            i,
        ))
    }

    fn compare_anchor(&self, i: usize) -> Option<(Value, Value, usize)> {
        let c = self.ui.cmp.as_ref()?;
        if c.level == 2 {
            return None; // the postings spine registers no rows (scroll and select by key only)
        }
        let m = self.cmp_full_pub()?;
        let vis = self.cmp_visible(m);
        let r = &m.rows[*vis.get(i)?];
        // the columns: code, account, then per org debit and credit (or net when narrow)
        let mut f = Map::new();
        f.insert("code".into(), json!(s(&r["code"])));
        f.insert("account".into(), json!(s(&r["label"])));
        for o in &m.orgs {
            let cell = &r["cells"][o.as_str()];
            let k = field_key(o);
            if cell["exists"] != true {
                f.insert(format!("{k}_debit"), json!("—"));
                f.insert(format!("{k}_credit"), json!("—"));
                f.insert(format!("{k}_net"), json!("—"));
                continue;
            }
            let amt = |v: &Value| {
                crate::ic_nav::dec(v)
                    .map(ownpurse_core::money::fmt)
                    .unwrap_or_default()
            };
            f.insert(format!("{k}_debit"), json!(amt(&cell["debit"])));
            f.insert(format!("{k}_credit"), json!(amt(&cell["credit"])));
            f.insert(format!("{k}_net"), json!(amt(&cell["net"])));
        }
        // the machine twins (v0.10.3): `<org>_debit_value` … from the display strings above
        let keys: Vec<String> = f.keys().cloned().collect();
        for k in keys {
            if k.ends_with("_debit") || k.ends_with("_credit") || k.ends_with("_net") {
                let twin = value_of(f[&k].as_str().unwrap_or(""));
                f.insert(format!("{k}_value"), twin);
            }
        }
        Some((
            json!({"compare_row": s(&r["key"]), "account_code": s(&r["code"])}),
            Value::Object(f),
            i,
        ))
    }

    /// The selected row's anchor (any level, including a document at Ledger/IC L3).
    /// The "Open in Xero" link of the document on screen (Ledger or Intercompany L3), if it has one.
    pub fn open_link(&self) -> Option<String> {
        if self.ui.tab == Tab::System {
            return crate::views::system::selected(self).and_then(|i| i.url);
        }
        let d = match self.ui.tab {
            Tab::Ledger if self.ledger_level() == 3 && !self.cmp_active() => self.doc_model()?,
            Tab::Intercompany if self.ic_level() == 3 => self.ic_doc()?,
            _ => return None,
        };
        d.link.as_ref().map(|l| l.url.clone())
    }

    pub fn selected_anchor(&self) -> Option<Value> {
        match self.ui.tab {
            Tab::Ledger if self.ledger_level() == 3 && !self.cmp_active() => {
                let d = self.nav().doc?;
                Some(json!({
                    "org": self.ledger_org().map(|o| self.org_alias(o)),
                    "doc_type": d.0,
                    "doc_id": d.1,
                }))
            }
            Tab::Intercompany if self.ic_level() == 3 => {
                let d = self.ic_nav().doc?;
                Some(json!({"org": d.0, "doc_type": d.1, "doc_id": d.2}))
            }
            _ => self
                .visible_rows()
                .into_iter()
                .find(|(t, _, _)| self.is_selected(*t))
                .map(|(_, a, _)| a),
        }
    }

    /// The row the cursor is on (drawn numbering, as `row_anchor`).
    pub(crate) fn is_selected(&self, t: Target) -> bool {
        let Target::Row(i) = t else { return false };
        self.cursor_abs() == Some(self.screen_top() + i)
    }

    /// A short label for the selected row (the notice's "selected 1460 Term Loan to Tallowmere").
    pub fn selected_label(&self) -> Option<String> {
        if self.ui.tab == Tab::Classify {
            let r = self.cl_selected()?;
            return Some(format!(
                "{} {} {}",
                r.org,
                crate::views::classify::payee_text(&r),
                ownpurse_core::money::fmt(r.amount)
            ));
        }
        let (_, a, f) = self
            .visible_rows()
            .into_iter()
            .find(|(t, _, _)| self.is_selected(*t))?;
        let txt = |k: &str| f.get(k).and_then(Value::as_str).map(str::to_string);
        Some(
            match (txt("code"), txt("account").or_else(|| txt("name"))) {
                (Some(c), Some(n)) if !c.is_empty() => format!("{c} {n}"),
                (_, Some(n)) => n,
                _ => txt("reference")
                    .filter(|r| !r.is_empty())
                    .or_else(|| txt("pair"))
                    .or_else(|| txt("label"))
                    .or_else(|| txt("endpoint"))
                    .or_else(|| a.as_object().and_then(|m| m.values().next()).map(s))
                    .unwrap_or_default(),
            },
        )
    }

    /// `select` beyond the screen: the whole-list position of the first row matching `q` (`#n` is the row
    /// number in the whole list; an anchor JSON or text match as for visible rows).
    fn find_list_row(&self, q: &str) -> Option<usize> {
        let q = q.trim();
        let total = self.total_rows();
        let rows: Vec<(usize, Value, Value)> = (0..total)
            .filter_map(|i| self.row_abs(i).map(|(a, f, n)| (n, a, f)))
            .collect();
        if let Some(n) = q.strip_prefix('#') {
            let n: usize = n.parse().ok()?;
            return rows.iter().find(|r| r.0 + 1 == n).map(|r| r.0);
        }
        if let Some(want) = anchor_query(q) {
            return rows
                .iter()
                .find(|(_, a, _)| {
                    want.iter()
                        .all(|(k, v)| a.get(k).is_some_and(|x| x == v || s(x) == s(v)))
                })
                .map(|r| r.0);
        }
        let needle = q.to_lowercase();
        let exact = |v: &Value| {
            v.as_object()
                .is_some_and(|m| m.values().any(|x| s(x).to_lowercase() == needle))
        };
        rows.iter()
            .find(|(_, a, f)| exact(a) || exact(f))
            .or_else(|| {
                rows.iter()
                    .find(|(_, a, f)| format!("{a} {f}").to_lowercase().contains(&needle))
            })
            .map(|r| r.0)
            .or_else(|| {
                q.parse::<usize>()
                    .ok()
                    .and_then(|n| rows.iter().find(|r| r.0 + 1 == n).map(|r| r.0))
            })
    }

    /// Move the cursor to whole-list position `idx` with the user's own moves (first, then next), so the
    /// view scrolls exactly as it would for keys. True when it got there.
    fn move_cursor_to(&mut self, idx: usize, fx: &mut Vec<Effect>) -> bool {
        if self.cursor_abs() == Some(idx) {
            return true;
        }
        self.run_action("row.first", fx);
        let mut guard = self.total_rows() + 1;
        while self.cursor_abs().is_some_and(|c| c < idx) && guard > 0 {
            let before = self.cursor_abs();
            self.run_action("row.next", fx);
            if self.cursor_abs() == before {
                break;
            }
            guard -= 1;
        }
        self.cursor_abs() == Some(idx)
    }

    /// `select` resolves to a visible row's hit target: `#3`, an anchor (JSON object, every key must match),
    /// else text (case-insensitive; an exact field value first, then a substring, then a bare row number).
    fn find_visible_row(&self, q: &str) -> Option<Target> {
        let numbered = self.visible_rows_numbered();
        let rows: Vec<(Target, Value, Value)> = numbered
            .iter()
            .map(|(t, a, f, _)| (*t, a.clone(), f.clone()))
            .collect();
        let q = q.trim();
        // `#n` / a bare n: the row numbered n in `state.rows` (its place in the whole list; must be visible)
        let row_no = |n: &str| {
            n.parse::<usize>()
                .ok()
                .and_then(|n| numbered.iter().find(|r| r.3 == n).map(|r| r.0))
        };
        if let Some(n) = q.strip_prefix('#') {
            return row_no(n);
        }
        if let Some(want) = anchor_query(q) {
            return rows
                .iter()
                .find(|(_, a, _)| {
                    want.iter()
                        .all(|(k, v)| a.get(k).is_some_and(|x| x == v || s(x) == s(v)))
                })
                .map(|r| r.0);
        }
        // text: an exact anchor or field value first (`270` is account 270), then a substring; a bare
        // number that matches nothing is the row number
        let needle = q.to_lowercase();
        let exact = |v: &Value| {
            v.as_object()
                .is_some_and(|m| m.values().any(|x| s(x).to_lowercase() == needle))
        };
        rows.iter()
            .find(|(_, a, f)| exact(a) || exact(f))
            .or_else(|| {
                rows.iter()
                    .find(|(_, a, f)| format!("{a} {f}").to_lowercase().contains(&needle))
            })
            .map(|r| r.0)
            .or_else(|| row_no(q))
    }

    /// The post-send summary every reply carries: where the view is (tab, level, breadcrumb, filter), what is
    /// selected, the visible window and the revision.
    pub fn state_summary(&self, rev: u64) -> Value {
        let st = self.state_json(rev);
        let mut m = serde_json::Map::new();
        for k in [
            "rev",
            "tab",
            "level",
            "scope",
            "breadcrumb",
            "filter",
            "selected",
            "view",
            "caption",
        ] {
            m.insert(k.into(), st.get(k).cloned().unwrap_or(Value::Null));
        }
        if self.ui.tab == Tab::Classify {
            m.insert("store_rev".into(), st["classify"]["store_rev"].clone());
        }
        Value::Object(m)
    }

    /// The view state as JSON (`ownpurse ui state`): `state::ViewState`, serialised.
    pub fn state_json(&self, rev: u64) -> Value {
        serde_json::to_value(self.view_state(rev)).unwrap_or(Value::Null)
    }

    /// The view state with the chosen rows (`state::Rows`), as JSON.
    pub fn state_json_rows(&self, rev: u64, which: crate::state::Rows) -> Value {
        serde_json::to_value(self.view_state_rows(rev, which)).unwrap_or(Value::Null)
    }
}

/// A row anchor asked for by `select`: JSON (`{"account_code":"1010"}`) or `key=value` pairs joined by `,`
/// (`system_item=pkg:<hash>`, `org=roast,doc_id=bt-1`), keys as `ui state` shows them (lowercase and `_`).
pub fn anchor_query(q: &str) -> Option<serde_json::Map<String, Value>> {
    if let Ok(Value::Object(m)) = serde_json::from_str::<Value>(q) {
        return Some(m);
    }
    let mut m = serde_json::Map::new();
    for part in q.split(',') {
        let (k, v) = part.split_once('=')?;
        let k = k.trim();
        if k.is_empty()
            || !k
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
        {
            return None;
        }
        m.insert(k.into(), Value::String(v.trim().into()));
    }
    (!m.is_empty()).then_some(m)
}

/// A column header as a state field key (stable snake_case): `parity (year)` → `parity_year`,
/// `TB fetched` → `tb_fetched`, `calls left` → `calls_left`, `side B (party)` → `side_b_party`; the
/// TB's flag column `!` → `flag`, and its wide `local − xero` → `difference` (the narrow title).
pub fn field_key(title: &str) -> String {
    match title {
        "!" => return "flag".into(),
        "local − xero" => return "difference".into(),
        _ => {}
    }
    let mut out = String::new();
    for ch in title.to_lowercase().chars() {
        if ch.is_alphanumeric() {
            out.push(ch);
        } else if !out.ends_with('_') && !out.is_empty() {
            out.push('_');
        }
    }
    out.trim_end_matches('_').to_string()
}

/// Amount columns that hold counts, not money (no `_value` twin: they are counts as shown).
const COUNT_COLUMNS: [&str; 5] = ["day left", "calls", "errors", "avg ms", "calls left"];

/// A money column: an amount column that isn't a count.
pub fn is_money(c: &crate::layout::ColSpec) -> bool {
    matches!(c.kind, crate::layout::Kind::Amount) && !COUNT_COLUMNS.contains(&c.title)
}

/// The number a money display string shows (v0.10.3 machine twins): `7,036.41` → 7036.41,
/// `(7,036.41)` / `−7,036.41` → −7036.41, `Dr 7,036.41` → 7036.41, `Cr 7,036.41` → −7036.41 (with or
/// without a leading org alias: `roast Cr 5,318.47`). None when the cell shows no amount (``, `—`).
pub fn display_value(text: &str) -> Option<rust_decimal::Decimal> {
    let mut t = text.trim();
    let mut neg = false;
    let words: Vec<&str> = t.split_whitespace().collect();
    if let Some(i) = words.iter().position(|w| *w == "Dr" || *w == "Cr")
        && i + 2 == words.len()
    {
        neg = words[i] == "Cr";
        t = words[i + 1];
    }
    if let Some(inner) = t.strip_prefix('(').and_then(|x| x.strip_suffix(')')) {
        neg = !neg;
        t = inner;
    }
    if let Some(rest) = t.strip_prefix('−').or_else(|| t.strip_prefix('-')) {
        neg = !neg;
        t = rest;
    }
    let t = t.strip_prefix('+').unwrap_or(t);
    let digits = t.replace(',', "");
    let (int, frac) = digits.split_once('.')?;
    if int.is_empty()
        || !int.bytes().all(|b| b.is_ascii_digit())
        || frac.len() != 2
        || !frac.bytes().all(|b| b.is_ascii_digit())
    {
        return None;
    }
    let v = ownpurse_core::money::parse(&digits).ok()?;
    Some(if neg { -v } else { v })
}

/// The JSON twin of a display string: an exact JSON number, or null when the cell shows no amount.
pub fn value_of(text: &str) -> Value {
    display_value(text)
        .and_then(|d| d.to_string().parse::<serde_json::Number>().ok())
        .map_or(Value::Null, Value::Number)
}

/// One field per column (`field_key(title)` → the cell's full text; a money column also gets its
/// `<key>_value` twin), then extras for keys not taken.
fn fields(
    cols: &[crate::layout::ColSpec],
    cells: &[crate::widgets::table::Cell],
    extra: &[(&str, Value)],
) -> Value {
    let mut m = Map::new();
    for (c, cell) in cols.iter().zip(cells) {
        let k = field_key(c.title);
        m.insert(k.clone(), json!(cell.text));
        // a money column, or a text column that shows a side's balance (`roast Cr 5,318.47`)
        let side = cell.text.contains(" Dr ") || cell.text.contains(" Cr ");
        if is_money(c) || (side && display_value(&cell.text).is_some()) {
            m.insert(format!("{k}_value"), value_of(&cell.text));
        }
    }
    for (k, v) in extra {
        if m.contains_key(*k) {
            continue;
        }
        m.insert(k.to_string(), v.clone());
        // a raw signed decimal extra (`balance`, `amount`) gets its exact-number twin too
        if let Some(t) = v.as_str()
            && !m.contains_key(&format!("{k}_value"))
            && display_value(t).is_some()
        {
            m.insert(format!("{k}_value"), value_of(t));
        }
    }
    Value::Object(m)
}

/// `text` cut to `w` cells by removing its middle (`opened Led…that differ`); empty when not even the
/// ellipsis fits.
pub fn middle_cut(text: &str, w: usize, ell: &str) -> String {
    use crate::layout::width;
    use unicode_segmentation::UnicodeSegmentation;
    if width(text) <= w {
        return text.to_string();
    }
    let Some(room) = w.checked_sub(width(ell)) else {
        return String::new();
    };
    let g: Vec<&str> = text.graphemes(true).collect();
    let (mut left, mut right) = (String::new(), Vec::new());
    let (mut lw, mut rw) = (0, 0);
    let (mut i, mut j) = (0, g.len());
    // alternate: the left half gets the extra cell
    loop {
        let mut moved = false;
        if i < j && lw <= rw && lw + rw + width(g[i]) <= room {
            lw += width(g[i]);
            left.push_str(g[i]);
            i += 1;
            moved = true;
        }
        if i < j && rw < lw && lw + rw + width(g[j - 1]) <= room {
            rw += width(g[j - 1]);
            right.push(g[j - 1]);
            j -= 1;
            moved = true;
        }
        if !moved {
            break;
        }
    }
    right.reverse();
    format!("{}{ell}{}", left.trim_end(), right.concat().trim_start())
}

#[cfg(test)]
mod tests {
    use super::middle_cut;

    #[test]
    fn middle_cut_keeps_both_ends() {
        let t = "opened Ledger · scope design · only accounts that differ";
        let c = middle_cut(t, 30, "…");
        assert!(crate::layout::width(&c) <= 30, "{c}");
        assert!(
            c.starts_with("opened Ledger") && c.ends_with("that differ"),
            "{c}"
        );
        assert!(c.contains('…'));
        assert_eq!(middle_cut(t, 200, "…"), t);
        assert_eq!(middle_cut(t, 0, "..."), "");
    }
}

/// Caption text fitted by segment: drop whole ` · ` segments from the middle (keeping
/// the first and the last, the gap marked `…`); only if those two don't fit, end-truncate the first.
pub fn caption_fit(text: &str, w: usize, ell: &str) -> String {
    use crate::layout::{fit_str, width};
    if width(text) <= w {
        return text.to_string();
    }
    let segs: Vec<&str> = text.split(" · ").collect();
    if segs.len() >= 2 {
        let (first, last) = (segs[0], segs[segs.len() - 1]);
        // keep as many leading middle segments as fit
        for keep in (0..segs.len().saturating_sub(2)).rev() {
            let mut parts: Vec<&str> = segs[..=keep].to_vec();
            parts.push(ell);
            parts.push(last);
            let t = parts.join(" · ");
            if width(&t) <= w {
                return t;
            }
        }
        let tail = format!(" · {ell} · {last}");
        if width(&tail) < w {
            return format!("{}{tail}", fit_str(first, w - width(&tail), ell));
        }
    }
    fit_str(text, w, ell)
}

impl App {
    /// The caption's footer parts (glyph, text, back hint), fitted for a footer `cols` wide with `room`
    /// cells for the left segment; None without a caption or while something else owns the footer.
    pub fn caption_parts(
        &self,
        cols: u16,
        room: usize,
    ) -> Option<Vec<(String, crate::theme_tokens::Token)>> {
        use crate::layout::width;
        use crate::theme_tokens::Token;
        let c = self.ui.caption.as_ref()?;
        if self.ui.overlay.is_some() || self.ui.prompt.is_some() || self.ui.band.is_some() {
            return None;
        }
        let th = &self.ui.theme;
        let plain = th.ascii || matches!(th.mode, crate::theme::Mode::NoColor);
        let glyph = if plain { "*" } else { "◆" }.to_string();
        let cap = match cols {
            c if c >= 140 => 84,
            c if c >= 100 => 58,
            c if c >= 80 => 44,
            _ => 30,
        };
        let back = self.agent.undo.is_some();
        let hint: Vec<(String, Token)> = match (back, cols) {
            (true, c) if c >= 100 => vec![
                (th.text(" · "), Token::Muted),
                ("`".into(), Token::Text),
                (" back".into(), Token::Muted),
            ],
            (true, c) if c >= 80 => vec![(th.text(" · "), Token::Muted), ("`".into(), Token::Text)],
            _ => vec![],
        };
        let hint_w: usize = hint.iter().map(|(t, _)| width(t)).sum();
        let text_room = cap.min(room.saturating_sub(width(&glyph) + 1 + hint_w));
        let text = caption_fit(&th.text(&c.text), text_room, th.glyphs().ellipsis);
        let mut v = vec![
            (glyph, Token::Info),
            (" ".into(), Token::Text),
            // the caption reads in `text.2` (the v0.10.2 review)
            (text, Token::Text2),
        ];
        v.extend(hint);
        Some(v)
    }
}

#[cfg(test)]
mod caption_tests {
    use super::caption_fit;

    #[test]
    fn segments_drop_from_the_middle() {
        let t = "Green-bean card spend, 2025 · 3 lines · 1,990.90";
        assert_eq!(caption_fit(t, 80, "…"), t);
        assert_eq!(
            caption_fit(t, 45, "…"),
            "Green-bean card spend, 2025 · … · 1,990.90"
        );
        let c = caption_fit(t, 30, "…");
        assert!(
            c.ends_with(" · … · 1,990.90") && crate::layout::width(&c) <= 30,
            "{c}"
        );
        assert_eq!(
            caption_fit("one segment that is long", 10, "…"),
            "one segme…"
        );
    }
}

// ── the cosmetic theme verbs (theme spec v2 §2.7) ──────────────────────────────────────────────────

impl App {
    /// The theme the overlay applies over: the user theme in use, else the built-in for `ui.theme.dark`.
    pub fn active_theme_def(&self) -> crate::theme::def::ThemeDef {
        self.user_theme().unwrap_or_else(|| {
            crate::theme::def::builtin(if self.ui.theme.dark {
                "ledger-dark"
            } else {
                "ledger-light"
            })
            .unwrap_or_default()
        })
    }

    /// One `token=value` entry: `bad_token` (exit 2) with the valid names for the prefix otherwise.
    fn check_overlay_entry(&self, key: &str, value: &str) -> Result<(), (String, String)> {
        crate::theme::def::check_overlay(&self.active_theme_def(), key, value)
            .map_err(|e| ("bad_token".into(), format!("{e}.")))
    }

    /// A token name to clear (any token or knob, set or not).
    fn check_overlay_name(&self, key: &str) -> Result<(), (String, String)> {
        let ok = crate::theme_tokens::Token::from_name(key).is_some()
            || key
                .strip_prefix("knob.")
                .is_some_and(|k| crate::theme::def::KNOB_NAMES.contains(&k));
        if ok {
            Ok(())
        } else {
            // the same message as an unknown token in `set`
            self.check_overlay_entry(key, "#000000")
        }
    }

    /// Apply a cosmetic verb: one theme change, `theme.revision` increments.
    pub(crate) fn apply_cosmetic(&mut self, v: &Verb) {
        match v {
            Verb::Theme(name) => {
                if let Some(t) = crate::theme::Theme::named(name) {
                    self.ui.theme = crate::theme::Theme {
                        ascii: self.ui.theme.ascii,
                        paint_bg: self.ui.theme.paint_bg,
                        ..t
                    };
                    self.ui.theme_user = None;
                } else if let Some(f) = self.user_config.themes.get(name) {
                    self.ui.theme_user = Some((name.clone(), f.text.clone()));
                }
            }
            Verb::Accent(a) => {
                self.ui.theme_overlay.insert("accent".into(), a.clone());
            }
            Verb::OverlaySet(pairs) => {
                for (k, v) in pairs {
                    self.ui.theme_overlay.insert(k.clone(), v.clone());
                }
            }
            Verb::OverlayClear(names) if names.is_empty() => self.ui.theme_overlay.clear(),
            Verb::OverlayClear(names) => {
                for n in names {
                    self.ui.theme_overlay.remove(n);
                }
            }
            _ => return,
        }
        self.ui.theme_revision += 1;
        self.repaint();
    }

    /// Text and strokes the current palette leaves under 4.5:1 (3:1 for `text.4` and strokes): the CLI
    /// prints these to stderr after an overlay send (overlays are exempt, but warned).
    pub fn contrast_warnings(&self) -> Vec<String> {
        use crate::theme::color::contrast;
        use crate::theme_tokens::Token as T;
        let th = &self.ui.theme;
        if !matches!(
            th.mode,
            crate::theme::Mode::Truecolor | crate::theme::Mode::Indexed
        ) {
            return vec![];
        }
        let surfaces = [
            T::Bg,
            T::Surface,
            T::HeaderBand,
            T::SurfaceSection,
            T::RowSelected,
            T::Raised,
            T::FooterBand,
        ];
        let texts = [
            (T::Text, 4.5),
            (T::Text2, 4.5),
            (T::Muted, 4.5),
            (T::Accent, 4.5),
            (T::Ok, 4.5),
            (T::Warn, 4.5),
            (T::Error, 4.5),
            (T::Info, 4.5),
            (T::Key, 4.5),
            (T::Text4, 3.0),
        ];
        let mut out = Vec::new();
        for s in surfaces {
            for (t, need) in texts {
                let r = contrast(th.rgb(t), th.rgb(s));
                if r < need {
                    out.push(format!(
                        "{} on {} is {:.2}:1 (needs {need}:1)",
                        t.name(),
                        s.name(),
                        r
                    ));
                }
            }
        }
        out
    }
}
