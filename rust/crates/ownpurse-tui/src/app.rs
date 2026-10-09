//! The application state and its one pure transition: `update(&mut App, Msg) -> Vec<Effect>` (Elm-style).
//! Draw never mutates; effects (reload, quit) are run by the caller. No effect
//! ever reaches Xero from the shell: network palette actions show the confirm-band placeholder (M10).

use crate::books::{Books, OverviewModel};
use crate::hit::{Hits, Target};
use crate::keymap::{self, Ctx, Key, When};
use crate::layout::HeaderFacts;
use crate::model::{Level, Model, Source};
use crate::theme::Theme;
use crossterm::event::{KeyCode, KeyEvent, MouseButton, MouseEvent, MouseEventKind};
use ratatui::layout::Position;
use std::sync::Arc;

/// The build's version: the crate version, or `OWNPURSE_BUILD_VERSION` at build time (the live suite builds a
/// second, "newer" binary this way to test `ui upgrade` across two real builds).
pub const VERSION: &str = match option_env!("OWNPURSE_BUILD_VERSION") {
    Some(v) => v,
    None => env!("CARGO_PKG_VERSION"),
};

/// The build identity (`build.rs`: the git commit, `.dirty` with uncommitted changes); None without git.
pub const BUILD: Option<&str> = option_env!("OWNPURSE_BUILD_ID");

/// `0.12.1+g6dc723f` (semver build metadata), or the version alone without a build id.
pub fn build_version() -> String {
    full_version(VERSION, BUILD)
}

pub fn full_version(version: &str, build: Option<&str>) -> String {
    match build.filter(|b| !b.is_empty()) {
        Some(b) => format!("{version}+{b}"),
        None => version.to_string(),
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum Tab {
    Overview,
    Ledger,
    Intercompany,
    Activity,
    System,
    Classify,
    Manifest,
}

impl Tab {
    pub const ALL: [Tab; 7] = [
        Tab::Overview,
        Tab::Ledger,
        Tab::Intercompany,
        Tab::Activity,
        Tab::System,
        Tab::Classify,
        Tab::Manifest,
    ];
    pub fn label(self) -> &'static str {
        match self {
            Tab::Overview => "Overview",
            Tab::Ledger => "Ledger",
            Tab::Intercompany => "Intercompany",
            Tab::Activity => "Activity",
            Tab::System => "System",
            Tab::Classify => "Classify",
            Tab::Manifest => "Manifest",
        }
    }
    /// The narrow tab label (the active tab is never dropped: the labels shorten first).
    pub fn short_label(self) -> &'static str {
        match self {
            Tab::Overview => "Ovw",
            Tab::Ledger => "Ldg",
            Tab::Intercompany => "IC",
            Tab::Activity => "Act",
            Tab::System => "Sys",
            Tab::Classify => "Cls",
            Tab::Manifest => "Mfst",
        }
    }
    pub fn index(self) -> usize {
        Tab::ALL.iter().position(|t| *t == self).unwrap_or(0)
    }
    pub fn ctx(self) -> Ctx {
        match self {
            Tab::Overview => Ctx::Overview,
            Tab::Ledger => Ctx::LedgerL1,
            Tab::Intercompany => Ctx::IcL1,
            Tab::Activity => Ctx::Activity,
            Tab::System => Ctx::System,
            Tab::Classify => Ctx::ClassifyMode,
            Tab::Manifest => Ctx::ManifestMode,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Scope {
    All,
    Org(usize),
    Group(usize),
}

/// The palette's network actions (spec §9 Palette; kit `ACTIONS` with `network: true`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum NetAction {
    SyncOrg(usize),
    SyncAll,
    /// `ownpurse sync group:<name>` (palette under a group scope).
    SyncGroup(usize),
    /// Not a network action: the quit confirm while a run is going (the child keeps running).
    QuitWhileRunning,
    Login,
    RefetchTb,
    FetchHistory,
    FetchAttach,
    /// `ownpurse classify suggest --engine jev` (V16: refused while Jev is not enabled).
    ClassifyJev,
    /// Not a network action: Activity's "Undo this write?" for an agent's classify write (its log id).
    UndoAgentWrite(i64),
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum BandKind {
    /// `[Cancel] [<label>]`, focus on Cancel (V3).
    Ask { action: NetAction, label: String },
    /// Persistent, error colour, one `Close` button (budget, offline, the pre-API placeholder).
    Refusal,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Band {
    pub kind: BandKind,
    pub text: String,
    pub focus: usize,
}

impl Band {
    pub fn buttons(&self) -> Vec<String> {
        match &self.kind {
            BandKind::Ask { label, .. } => vec!["Cancel".into(), label.clone()],
            BandKind::Refusal => vec!["Close".into()],
        }
    }
    pub(crate) fn refusal(text: String) -> Band {
        Band {
            kind: BandKind::Refusal,
            text,
            focus: 0,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Overlay {
    Help,
    Palette {
        query: String,
        sel: usize,
    },
    Picker {
        query: String,
        sel: usize,
        multi: bool,
        chosen: Vec<usize>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Prompt {
    Filter(String),
    AsOf(String),
    /// Ledger L2 `f`: the range's first day (the last is the as-of date).
    Range(String),
}

/// Where a drill chain started (navigation rule 3: Esc returns there, not to the tab's root).
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Origin {
    pub from: Tab,
    pub to: Tab,
    pub scope: Scope,
    pub overview_sel: Option<String>,
    pub overview_col: OvCol,
}

/// Everything a frame shows that keys and clicks change. `update` compares it before and after to know
/// whether a redraw is due (V13: only a state-changing Msg redraws).
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Ui {
    pub tab: Tab,
    pub scope: Scope,
    pub overlay: Option<Overlay>,
    pub band: Option<Band>,
    pub prompt: Option<Prompt>,
    pub filter: String,
    pub as_of: String,
    /// Overview cursor, kept as the org alias (navigation rule 4), not a row index.
    pub overview_sel: Option<String>,
    pub overview_top: usize,
    pub chain: Vec<Origin>,
    /// A one-line status for the footer's left side until the next key.
    pub note: Option<String>,
    pub theme: Theme,
    pub size: (u16, u16),
    pub full_history: bool,
    pub differs_only: bool,
    /// A sync or login child running now (spec P6 §4).
    pub run: Option<crate::run::RunState>,
    /// The last run's result band, until the next key (spec P6 §5).
    pub result: Option<crate::run::ResultBand>,
    /// The Overview cell under the cursor (M1: a parity cell drills with its basis).
    pub overview_col: OvCol,
    /// Ledger drill state per scope (navigation rule 6), cursors kept as ids (rule 4).
    pub ledger: std::collections::BTreeMap<String, crate::ledger_nav::LedgerNav>,
    /// Intercompany drill state per scope (pair id, item source id).
    pub ic: std::collections::BTreeMap<String, crate::ic_nav::IcNav>,
    /// Intercompany L1 sort (index into `ic_nav::SORTS`).
    pub ic_sort: usize,
    /// Viewing another org's Ledger from a bridge (`a` / `b`) or from Compare.
    pub via: Option<crate::ic_nav::Via>,
    /// Compare (Ledger `v`): the orgs, alignment, level and cursors.
    pub cmp: Option<crate::compare::CmpState>,
    /// Activity tab state (grouping, cursor by log id / endpoint).
    pub activity: crate::views::activity::ActivityUi,
    /// System tab state (cursor by row key).
    pub system: crate::views::system::SystemUi,
    /// The caret in the active text input (prompt, palette or picker query), in characters; None = at the
    /// end (v0.10.2: in-progress input is part of the state).
    #[serde(default)]
    pub caret: Option<usize>,
    /// Browser-style navigation history (Alt+← / Alt+→; agent `back` / `forward`), bounded.
    #[serde(default)]
    pub history: crate::history::History,
    /// The agent's caption in the footer; cleared with the view identity.
    #[serde(default)]
    pub caption: Option<crate::history::Caption>,
    /// The Classify tab (v0.11.0): mode, filter, selected anchor, audition set, and its open editors.
    #[serde(default)]
    pub classify: crate::classify::ClassifyUi,
    /// The theme overlay (theme spec v2 §2.7): session-only token overrides set by agents (`overlay set`,
    /// `accent`), `token → value`. Cosmetic; round-trips through the state.
    #[serde(default)]
    pub theme_overlay: std::collections::BTreeMap<String, String>,
    /// Bumped on every applied theme change (overlay, theme switch, reload): `theme.revision` in the state.
    #[serde(default)]
    pub theme_revision: u64,
    /// A user theme in use (`themes/<name>.toml`), by name, with the file's text as it was when applied: a
    /// render from state uses this text, never the file on disk. None: the built-in for `theme`.
    #[serde(default)]
    pub theme_user: Option<(String, String)>,
    /// The details layer (`i`; design pass 2 tier 3): IDs, provenance sentences, budgets, versions.
    #[serde(default)]
    pub details: bool,
    /// The manifest this session shows; None = main. Its head revision travels in the canonical
    /// state beside it (`canonical.manifest_rev`).
    #[serde(default)]
    pub manifest: Option<String>,
    /// Tab 7 Manifest: its mode (entries, issues, diff, log), list cursor, the open prompt and the open diff.
    #[serde(default)]
    pub manifest_tab: crate::manifest_tab::ManifestUi,
    /// Overlay layers (hints, rings, spotlights; caretline-layers): a walkthrough's (owner `guide`) and agents'.
    #[serde(default)]
    pub layers: caretline_layers::Layers,
    /// The narration panel: an agent's `narrate`, or the walkthrough step shown.
    #[serde(default)]
    pub narration: Option<crate::walk::Narration>,
    /// The walkthrough being played.
    #[serde(default)]
    pub walk: Option<Box<crate::walk::WalkUi>>,
    /// Lay mode (`L`): account names lead, codes hide, "why this matters" lines show.
    #[serde(default)]
    pub lay_mode: bool,
    /// The glossary term whose definition is open (`?` or a click on a `[term]`).
    #[serde(default)]
    pub glossary_focus: Option<String>,
    /// The `W` list of walkthroughs.
    #[serde(default)]
    pub walk_list: Option<crate::walk::WalkList>,
    /// A walkthrough being recorded (`walkthrough record start`).
    #[serde(default)]
    pub recording: Option<crate::walk::Recording>,
    /// The open explainer panel (walkthroughs §4): its spec and scroll; None = no panel.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub viz: Option<crate::views::viz::VizOpen>,
}

/// Overview cells that drill differently (spec §5.2, M1).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum OvCol {
    #[default]
    Org,
    ParityYear,
    ParityHistory,
}

pub struct App {
    pub ui: Ui,
    pub model: Model,
    pub source: Source,
    pub offline: bool,
    pub now: i64,
    pub profile: String,
    /// Hit targets of the last frame (mouse).
    pub hits: Hits,
    /// A redraw is due.
    pub dirty: bool,
    /// What paths are shown relative to (cwd, home); pinned in snapshots so they don't depend on the checkout.
    pub paths: crate::system::PathBase,
    /// This build's identity (`BUILD`; None in snapshots): `i` details and the state show it.
    pub build_label: Option<String>,
    /// The version the header shows (`CARGO_PKG_VERSION`; a fixed label in snapshot tests, like the clock).
    pub version_label: String,
    /// The `ownpurse` CLI this app runs for sync and login: next to the app first, then PATH (spec P6 §2).
    pub cli: Option<std::path::PathBuf>,
    /// Its `--version`, read once when System is first shown.
    pub cli_version: Option<String>,
    /// How System shows `cli` when not its real path (snapshots pin `~/.local/bin/ownpurse`, the installed
    /// location, while the tests run the fake next to the fixtures).
    pub cli_shown: Option<String>,
    /// The record's books and their caches (None if the record could not be opened).
    pub books: Option<Books>,
    pub books_error: Option<String>,
    /// The prepared Overview for the current as-of (built in `update`, read by draw).
    pub overview: Option<Arc<OverviewModel>>,
    /// The prepared Ledger models for the current scope / as-of / basis / drill.
    pub ledger_view: crate::ledger_nav::LedgerView,
    /// The prepared Intercompany models.
    pub ic_view: crate::ic_nav::IcView,
    /// The prepared Compare models.
    pub cmp_view: crate::compare::CmpView,
    /// The open explainer panel's model (prepared from the books in `update`).
    pub viz_view: crate::views::viz::VizView,
    /// The agent bridge's footprint: header mark, notice, one-step back, Activity rows (v0.10).
    pub agent: crate::agent::AgentUi,
    /// The clock value the Model was loaded at (record ages and today's counts derive from it).
    pub model_now: i64,
    /// This session's label (`--label`, `tui.label`, agent `label`): shown in `ui ls`.
    pub label: Option<String>,
    /// A newer installed `ownpurse-tui` (footer and state; checked at start, on the minute tick and on keys).
    pub update_available: Option<crate::canonical::Update>,
    /// Rendered from a state exported against a different record version (`--allow-record-drift`).
    pub drift: Option<crate::canonical::Drift>,
    /// Refreshes for other processes' writes this session (`ui state` `data_refreshes`; not canonical).
    pub data_refreshes: u64,
    /// The client name the current agent request carried (captions say who set them).
    pub agent_client: Option<String>,
    /// An agent `back` / `forward` found nothing to go to (the answer says `note: no_history`).
    pub agent_no_history: Option<&'static str>,
    /// The view revision: +1 on every applied Msg that changes the view (a key, a click, an agent verb,
    /// a reload). `ownpurse ui send --expect-rev` is refused when it moved.
    pub rev: u64,
    /// Set while a history move runs (it changes the view without pushing a new entry).
    pub(crate) hist_moved: bool,
    /// The classification layer (labels, suggestions, notes; v0.11.0). None: not set up.
    pub classify: Option<crate::classify::Data>,
    /// Groups: the member document open over the detail (prepared; drawn as Ledger L3).
    pub cl_doc: Option<crate::books::Shared<crate::ledgers::DocModel>>,
    /// The effective keymap: the built-in table with ~/.config/ownpurse/keys.toml layered on (v0.12.0).
    /// Swapped whole on a valid reload; exported in the state (its keys.toml) so a render needs no file.
    pub keymap: std::sync::Arc<crate::keymap::Keymap>,
    /// The user config as last validly loaded (keys.toml, themes; Worker T reads `themes`).
    pub user_config: crate::user_config::UserConfig,
    /// The parsed user theme for `ui.theme_user` (keyed by its text), so a repaint doesn't re-parse.
    pub theme_cache: std::cell::RefCell<Option<(String, Option<crate::theme::def::ThemeDef>)>>,
    /// The terminal reports ⌘ (keyboard enhancement is on); false in tests and plain terminals.
    pub cmd_keys: bool,
    /// Manifests: the store and the shown manifest, materialised (`ui.manifest` is the state).
    pub manifests: crate::manifest::ManifestState,
    /// The glossary behind `[term]` links: the shipped default plus the project's `glossary.toml`.
    pub glossary: ownpurse_core::glossary::Glossary,
    /// Layers the current agent send pushed (its reply says where each landed). Transient.
    pub layers_touched: Vec<String>,
    /// A walkthrough the last send saved (`walkthrough record stop`): its path, or why not. Transient.
    pub walk_saved: Option<Result<std::path::PathBuf, String>>,
}

pub enum Msg {
    Key(KeyEvent),
    Mouse {
        ev: MouseEvent,
        clicks: u8,
    },
    Resize(u16, u16),
    /// The clock (seconds since the epoch). Redraws only when the minute changes.
    Tick(i64),
    /// The record re-read (`r`): a fresh model and fresh books (every cache starts again).
    Reloaded(Box<(Model, Option<Books>)>),
    /// Run a keymap action directly (tests, key scripts).
    Action(&'static str),
    /// A running child's stderr line, its exit, or a failed start.
    Run(crate::net::RunEvent),
    /// An agent's view verb (the bridge; allow-listed, view-only), applied with the same actions as keys.
    Agent(crate::agent::Verb),
    /// A whole agent send: its verbs in one update (one revision), atomic.
    AgentSend(Vec<crate::agent::Verb>),
    /// A classification write finished (`Effect::Classify`): its summary, or why it was refused.
    Classified(Result<String, String>),
    /// The Catalog's staged edits were written to `classify.toml` (summary), or refused (why).
    ConfigWritten(Result<String, String>),
    /// `classify.toml` changed on disk (the watcher; no timer): reload it.
    ClassifyConfigChanged,
    /// The user config folder changed (keys.toml, themes/*.toml; the one watcher, no timer).
    UserConfig(crate::user_config::ConfigEvent),
    /// Another process moved the record or the store (`external`): these are the revisions on disk.
    External(crate::external::DiskRevs),
    /// An agent's classify request (bridge `kind: "classify"`).
    AgentClassify(Box<ownpurse_core::classify::request::ClassifyRequest>),
    /// The loop applied it (`Workspace::apply`).
    AgentWrote(
        Box<ownpurse_core::classify::Result<ownpurse_core::classify::request::ApplyOutcome>>,
    ),
    /// `U` / Activity's undo of an agent write finished (the events Activity named, if any).
    AgentUndone(
        Result<ownpurse_core::classify::WriteOutcome, String>,
        Option<Vec<i64>>,
    ),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Effect {
    Quit,
    /// Re-read the local record (zero calls).
    Reload,
    /// A request to Xero. Nothing in the shell produces it (V2); the API crate will, after the confirm band.
    Network(NetAction),
    /// Open the config file in $EDITOR (System › Settings, `e`); the loop suspends the terminal.
    Edit(std::path::PathBuf),
    /// Open a URL in the browser (System › Links Enter / `o`; Ledger L3 `o`; a Xero web page, not an API call).
    Open(String),
    /// Copy text to the clipboard via OSC 52 (System › Links and Ledger L3, `y`).
    Copy(String),
    /// Run the installed CLI as a child (sync / login; spec P6 §2). The runtime starts it and streams its
    /// events back as `Msg::Run`.
    Spawn {
        program: std::path::PathBuf,
        args: Vec<String>,
    },
    /// `<cli> --version`, once, for System's `cli:` row.
    CliVersion,
    /// Upgrade in place to the newer installed build (palette `Upgrade to vX`; the loop hands over).
    Upgrade,
    /// Append to the classification store (a label, a note, an undo; never Xero). The loop runs it and
    /// answers `Msg::Classified`.
    Classify(crate::classify::Write),
    /// Write a recorded walkthrough to the walkthroughs folder (`walkthrough record stop`; local files only).
    SaveWalkthrough(Box<ownpurse_core::walkthrough::Walkthrough>),
}

/// Exact refusal texts (acceptance M8, M10).
pub const OFFLINE_REFUSAL: &str =
    "Offline mode: network actions are disabled. Restart without --offline to use them.";
pub const NOT_AVAILABLE: &str = "Not available yet (coming in v0.9.x).";
/// V16: Jev runs only once `classify.jev.enabled` is true and a key is configured; never in v0.11.0.
pub const JEV_NOT_ENABLED: &str =
    "Jev not enabled: classify.toml [jev] enabled = false. Suggestions come from rules only.";

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PalRun {
    Net(NetAction),
    Scope(Scope),
    Action(&'static str),
    /// "Explain: …": open an explainer panel.
    Viz(Box<ownpurse_core::viz::Spec>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PalEntry {
    pub label: String,
    pub keys: Option<String>,
    pub network: bool,
    pub run: PalRun,
    /// A muted suffix after the label (`· running: sync design` while a
    /// child runs, when every ⇅ entry is disabled).
    pub suffix: Option<String>,
    /// Disabled while a child runs (one at a time): the whole row draws muted.
    pub disabled: bool,
}

/// The header's calls-left facts (see `App::header_quota`).
struct HeaderQuota {
    left: i64,
    level: Level,
    estimated: bool,
    busy: bool,
    run: Option<crate::layout::RunSeg>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PickEntry {
    pub scope: Scope,
    pub label: String,
    pub detail: String,
    pub current: bool,
}

impl App {
    /// The revisions this TUI shows (compared with the disk by `Runtime::external_check`); unknown where a
    /// database is absent.
    pub fn shown_revs(&self) -> crate::external::DiskRevs {
        crate::external::DiskRevs {
            record: Some(self.model.record_version),
            store: self.classify.as_ref().map(|d| d.provider.rev()),
            manifests: self
                .source
                .manifests
                .as_ref()
                .filter(|p| p.store.exists())
                .map(|_| self.manifests.store.as_ref().map_or(0, |s| s.rev())),
        }
    }

    /// Another process wrote: the record moved → the normal reload (the store with it); only the store moved →
    /// pick up its events. The view is kept; the note says what arrived. While this TUI's own sync runs, the
    /// record waits for its reload at the end.
    fn on_external(&mut self, disk: crate::external::DiskRevs, fx: &mut Vec<Effect>) {
        let shown = self.shown_revs();
        let store_path = self.source.classify.as_ref().map(|c| c.store.clone());
        let record_moved =
            disk.record.is_some() && disk.record != shown.record && self.ui.run.is_none();
        let store_moved =
            disk.store.is_some() && shown.store.is_some() && disk.store != shown.store;
        let manifests_moved = disk.manifests.is_some() && disk.manifests != shown.manifests;
        if manifests_moved && !record_moved && !store_moved {
            // another process (an agent, the CLI) wrote a manifest: re-read it, re-attach the shown one
            self.data_refreshes += 1;
            self.manifest_sync();
            self.prepare();
            self.dirty = true;
            self.rev += 1;
            let n = disk.manifests.unwrap_or(0) - shown.manifests.unwrap_or(0);
            if n > 0 {
                self.ui.note = Some(format!(
                    "manifest updated · {n} event{}",
                    if n == 1 { "" } else { "s" }
                ));
            }
            return;
        }
        if manifests_moved {
            self.manifest_sync();
        }
        if !record_moved && !store_moved {
            return;
        }
        let from = crate::external::DiskRevs {
            record: if record_moved {
                shown.record
            } else {
                disk.record
            },
            store: shown.store,
            manifests: shown.manifests,
        };
        let note =
            crate::external::describe(&self.source.record, store_path.as_deref(), from, disk);
        self.data_refreshes += 1;
        if record_moved {
            fx.push(Effect::Reload);
        } else if let Some(d) = self.classify.as_mut() {
            d.provider.refresh();
            self.prepare();
            self.dirty = true;
            self.rev += 1;
        }
        if !note.is_empty() {
            self.ui.note = Some(note);
            // the newest news leads the strip (an older agent notice would hide it; ` still undoes)
            self.agent.notice = None;
            self.dirty = true;
        }
    }

    pub fn new(model: Model, source: Source, theme: Theme, offline: bool, now: i64) -> App {
        let first = (!model.orgs.is_empty()).then_some(0);
        // `tui.as_of`: a date, `today` (the clock's date), or unset (the last fiscal year end)
        let as_of = match source.as_of.as_deref() {
            Some("today") => crate::clock::date_of(now),
            Some(d) => d.to_string(),
            None => model.default_as_of(first, now),
        };
        let overview_sel = model.orgs.first().map(|o| o.alias.clone());
        let (books, books_error) = match Books::open(&source) {
            Ok(b) => (Some(b), None),
            Err(e) => (None, Some(e)),
        };
        let profile = source.profile.clone();
        let mut app = App {
            ui: Ui {
                tab: Tab::Overview,
                scope: Scope::All,
                overlay: None,
                band: None,
                prompt: None,
                filter: String::new(),
                as_of,
                overview_sel,
                overview_top: 0,
                chain: Vec::new(),
                note: None,
                theme,
                size: (80, 24),
                full_history: false,
                differs_only: false,
                run: None,
                result: None,
                overview_col: OvCol::Org,
                ledger: Default::default(),
                ic: Default::default(),
                ic_sort: 0,
                via: None,
                cmp: None,
                activity: Default::default(),
                system: Default::default(),
                caret: None,
                history: Default::default(),
                caption: None,
                classify: Default::default(),
                theme_overlay: Default::default(),
                theme_revision: 0,
                theme_user: None,
                details: source.view.details,
                manifest: source.manifests.as_ref().and_then(|p| p.current()),
                manifest_tab: Default::default(),
                layers: Default::default(),
                narration: None,
                walk: None,
                lay_mode: false,
                glossary_focus: None,
                walk_list: None,
                recording: None,
                viz: None,
            },
            model,
            source,
            offline,
            now,
            profile,
            hits: Hits::default(),
            dirty: true,
            paths: crate::system::PathBase::process(),
            version_label: VERSION.to_string(),
            build_label: BUILD.map(str::to_string),
            cli: crate::net::resolve_cli_for_process(),
            cli_version: None,
            cli_shown: None,
            books,
            books_error,
            overview: None,
            ledger_view: Default::default(),
            ic_view: Default::default(),
            cmp_view: Default::default(),
            viz_view: None,
            agent: Default::default(),
            model_now: now,
            label: None, // set below from the source
            update_available: None,
            drift: None,
            data_refreshes: 0,
            agent_client: None,
            agent_no_history: None,
            rev: 0,
            hist_moved: false,
            classify: None,
            cl_doc: None,
            keymap: std::sync::Arc::new(crate::keymap::default_map().clone()),
            user_config: Default::default(),
            theme_cache: Default::default(),
            cmd_keys: false,
            manifests: Default::default(),
            glossary: Default::default(),
            layers_touched: vec![],
            walk_saved: None,
        };
        app.classify = crate::classify::open(&app.source);
        app.label = app.source.label.clone();
        app.glossary = ownpurse_core::glossary::Glossary::load(app.source.glossary_dir.as_deref())
            .unwrap_or_else(|_| ownpurse_core::glossary::Glossary::default_set());
        app.manifest_sync();
        app.prepare();
        app
    }

    /// Prepare the models the current view draws (spec §14: no work in draw). Cached per (as-of) inside
    /// one `Books`, i.e. per record version; a tab, scope or drill change only looks them up.
    pub fn prepare(&mut self) {
        self.repaint();
        self.prepare_cl_doc();
        self.prepare_manifest();
        self.prepare_ledger();
        self.prepare_ic();
        self.prepare_compare();
        self.prepare_viz();
        let Some(books) = self.books.as_mut() else {
            return;
        };
        if self.ui.tab == Tab::Overview
            && self.overview.as_ref().is_none_or(|o| {
                o.as_of != self.ui.as_of || o.now.div_euclid(60) != self.now.div_euclid(60)
            })
        {
            self.overview = Some(Arc::new(OverviewModel::build(
                books,
                &self.model,
                &self.source,
                &self.ui.as_of,
                self.now,
            )));
        }
    }

    /// Groups' open member document (from the record; cached in the books like Ledger L3's).
    fn prepare_cl_doc(&mut self) {
        let want = (self.ui.tab == Tab::Classify
            && self.ui.classify.mode == crate::classify::Mode::Groups)
            .then(|| self.ui.classify.groups.doc.clone())
            .flatten();
        self.cl_doc = match (want, self.books.as_mut()) {
            (Some((org, entity, id)), Some(books)) => self
                .model
                .orgs
                .iter()
                .position(|o| o.alias == org)
                .map(|oi| {
                    crate::ledgers::doc(books, &self.model, &self.source.links, oi, &entity, &id)
                }),
            _ => None,
        };
    }

    /// Drop prepared models (after a change to the model itself, e.g. in tests and on reload).
    pub fn invalidate(&mut self) {
        self.overview = None;
        self.ledger_view = Default::default();
        self.ic_view = Default::default();
        self.cmp_view = Default::default();
        self.prepare();
        self.dirty = true;
    }

    // ── scope ────────────────────────────────────────────────────────────────────────────────────

    pub fn members(&self, scope: &Scope) -> Vec<usize> {
        match scope {
            Scope::All => (0..self.model.orgs.len()).collect(),
            Scope::Org(i) => vec![*i],
            Scope::Group(g) => self
                .model
                .groups
                .get(*g)
                .map(|g| g.members.clone())
                .unwrap_or_default(),
        }
    }

    pub fn scope_label(&self, scope: &Scope) -> String {
        match scope {
            Scope::All => "all".into(),
            Scope::Org(i) => self.model.orgs[*i].alias.clone(),
            Scope::Group(g) => format!("group:{}", self.model.groups[*g].name),
        }
    }

    /// Every scope in `[` `]` order: all, each org, each group.
    pub fn scopes(&self) -> Vec<Scope> {
        let mut v = vec![Scope::All];
        v.extend((0..self.model.orgs.len()).map(Scope::Org));
        v.extend((0..self.model.groups.len()).map(Scope::Group));
        v
    }

    /// The header scope chip (theme spec v2 §2.3) in its full, compact and minimal forms, each as styled
    /// pieces. The name always carries the meaning (T1): dots only add the members' accents, and the plain
    /// rendering (NO_COLOR) is a bracketed name with no dots. Colour comes only from the derived accent state.
    pub fn scope_chips(&self) -> [Vec<crate::layout::ChipSpan>; 3] {
        use crate::theme_tokens::Token;
        let acc = self.accent();
        let plain = matches!(self.ui.theme.mode, crate::theme::Mode::NoColor);
        let names: Vec<String> = acc.members.iter().map(|m| m.alias.clone()).collect();
        let s = |t: &str, k: Token| (t.to_string(), k);
        let dots = || -> Vec<crate::layout::ChipSpan> {
            (0..names.len().min(4))
                .map(|i| s("●", Token::member(i)))
                .collect()
        };
        let caret = s(" ▾", Token::Muted);
        let n = names.len();
        let chip = |label: &str| -> Vec<crate::layout::ChipSpan> {
            vec![s(&format!("[{label}]"), Token::Text), caret.clone()]
        };
        let mut chips = match acc.scope_kind {
            "org" => {
                let a = names.first().cloned().unwrap_or_default();
                let v = if plain {
                    chip(&a)
                } else {
                    vec![
                        s("●", Token::Accent),
                        s(" ", Token::Muted),
                        s(&a, Token::Accent),
                        caret.clone(),
                    ]
                };
                [v.clone(), v, vec![]]
            }
            "pair" => {
                let (a, b) = (
                    names.first().cloned().unwrap_or_default(),
                    names.get(1).cloned().unwrap_or_default(),
                );
                let v = if plain {
                    vec![s(&format!("[{a} ↔ {b}]"), Token::Text)]
                } else {
                    vec![
                        s("●", Token::member(0)),
                        s(" ", Token::Muted),
                        s(&a, Token::member(0)),
                        s(" ↔ ", Token::Muted),
                        s("●", Token::member(1)),
                        s(" ", Token::Muted),
                        s(&b, Token::member(1)),
                    ]
                };
                [v.clone(), v.clone(), v]
            }
            "compare" => {
                let list = names.join(" · ");
                if plain {
                    [
                        chip(&format!("compare: {list}")),
                        chip(&format!("compare ({n})")),
                        vec![],
                    ]
                } else {
                    let mut full = dots();
                    full.extend([
                        s(" ", Token::Muted),
                        s(&format!("compare: {list}"), Token::Accent),
                        caret.clone(),
                    ]);
                    let mut compact = dots();
                    compact.extend([
                        s(" ", Token::Muted),
                        s("compare", Token::Accent),
                        s(&format!(" ({n})"), Token::Muted),
                        caret.clone(),
                    ]);
                    let min = vec![
                        s("compare", Token::Accent),
                        s(&format!(" ({n})"), Token::Muted),
                        caret.clone(),
                    ];
                    [full, compact, min]
                }
            }
            "group" => {
                let name = match self.ui.scope {
                    Scope::Group(g) => self.model.groups[g].name.clone(),
                    _ => String::new(),
                };
                if plain {
                    [
                        chip(&format!("group:{name} ({n})")),
                        chip(&format!("{name} ({n})")),
                        vec![],
                    ]
                } else {
                    let mut full = vec![
                        s(&format!("group:{name}"), Token::Accent),
                        s(&format!(" ({n}) "), Token::Muted),
                    ];
                    full.extend(dots());
                    full.push(caret.clone());
                    let mut compact = vec![
                        s(&name, Token::Accent),
                        s(&format!(" ({n}) "), Token::Muted),
                    ];
                    compact.extend(dots());
                    compact.push(caret.clone());
                    // the last resort drops the dots (the name carries the meaning, T1)
                    let min = vec![
                        s(&name, Token::Accent),
                        s(&format!(" ({n})"), Token::Muted),
                        caret.clone(),
                    ];
                    [full, compact, min]
                }
            }
            _ => {
                if plain {
                    [chip("all organisations"), chip("all"), vec![]]
                } else {
                    let mut full = vec![s("All organisations ", Token::Accent)];
                    full.extend(dots());
                    full.push(caret.clone());
                    let mut compact = vec![s("all ", Token::Accent)];
                    compact.extend(dots());
                    compact.push(caret.clone());
                    let min = vec![s("all", Token::Accent), caret.clone()];
                    [full, compact, min]
                }
            }
        };
        // Another org's books from a bridge (Ledger via `a`/`b`, IC L3) when the pair is not resolved: name it (E6).
        let via = match self.ui.tab {
            Tab::Ledger => self.ui.via.as_ref().map(|v| v.pair.clone()),
            Tab::Intercompany if self.ic_level() == 3 => self.ic_nav().pair,
            _ => None,
        };
        if let Some(p) = via.filter(|_| acc.scope_kind != "pair") {
            for c in chips.iter_mut().take(2) {
                c.push(s(&format!(" · via {p}"), Token::Muted));
            }
            chips[2] = vec![s(&format!("via {p} ▾"), Token::Muted)];
        }
        chips
    }

    pub fn header_facts(&self) -> HeaderFacts {
        // Compare: the compared orgs (oldest record, lowest calls left) and `compare: a · b ▾` (H1)
        let cmp = self.ui.cmp.as_ref().filter(|_| self.cmp_active());
        let members = cmp.map_or_else(|| self.members(&self.ui.scope), |c| c.orgs.clone());
        let fresh = self.model.freshness(&members, self.now);
        let q = self.header_quota(&members);
        let [full, compact, min] = self.scope_chips();
        let scope_full = crate::layout::chip_text(&full);
        let scope_compact = crate::layout::chip_text(&compact);
        let scope_min = crate::layout::chip_text(&min);
        let _ = cmp;
        HeaderFacts {
            product: format!("ownpurse · {} · v{}", self.profile, self.version_label),
            offline: self.offline,
            scope_full,
            scope_compact,
            scope_min,
            as_of: self.ui.as_of.clone(),
            fresh_full: fresh.full(),
            fresh_compact: fresh.compact(),
            fresh_min: fresh.min(),
            fresh_level: fresh.level,
            left: q.left,
            limit: self.model.limit,
            quota_level: q.level,
            left_estimated: q.estimated,
            busy: q.busy,
            run: q.run,
            agent: self.agent_mark(),
            plain: matches!(self.ui.theme.mode, crate::theme::Mode::NoColor),
            show_age: self.source.view.record_age_in_header == "always"
                || fresh.level != crate::model::Level::Ok,
            show_quota: match self.source.view.budget_in_header {
                "always" => true,
                "never" => false,
                _ => q.level != crate::model::Level::Ok || self.offline,
            },
            manifest: self.manifest_chip(),
            writes: self.source.allow_writes,
            drift: self.drift.as_ref().map(|d| {
                if d.manifest.is_some()
                    && d.from == d.to
                    && d.config.is_none()
                    && d.classify.is_none()
                {
                    "≠ manifest changed"
                } else {
                    crate::layout::drift_text(
                        d.from != d.to,
                        d.config.is_some(),
                        d.classify.is_some() || d.manifest.is_some(),
                    )
                }
            }),
            classify: self
                .cl_counts()
                .filter(|_| self.ui.tab == Tab::Classify)
                .map(|c| (c.open, c.stale)),
        }
    }

    /// The header's calls-left figure: the lowest of the scope's orgs, with the running org's
    /// live `day_remaining` in place of its recorded figure. Never two figures for one org: if the figure
    /// shown belongs to the running org it carries `⇅` in place; otherwise a separate `⇅ <org> · N left`
    /// segment names the running org. A login (no org) puts `⇅` on the figure.
    fn header_quota(&self, members: &[usize]) -> HeaderQuota {
        use crate::run::RunKind;
        let run = self.ui.run.as_ref();
        let running: Option<(Option<usize>, String, Option<i64>)> =
            run.and_then(|r| match &r.kind {
                RunKind::Login => None,
                RunKind::Sync { target, .. } => {
                    let who = if r.org.is_empty() {
                        target.clone()
                    } else {
                        r.org.clone()
                    };
                    let i = self.model.orgs.iter().position(|o| o.alias == who);
                    Some((i, who, r.day_remaining))
                }
            });
        let live = |i: usize| -> Option<i64> {
            running
                .as_ref()
                .filter(|(ri, _, _)| *ri == Some(i))
                .and_then(|(_, _, d)| *d)
        };
        let eff = |i: usize| live(i).unwrap_or(self.model.orgs[i].left);
        let left = members
            .iter()
            .map(|&i| eff(i))
            .min()
            .unwrap_or(self.model.limit);
        let tied: Vec<usize> = members
            .iter()
            .copied()
            .filter(|&i| eff(i) == left)
            .collect();
        let estimated = !tied.is_empty()
            && tied
                .iter()
                .all(|&i| live(i).is_none() && !self.model.orgs[i].left_known);
        let mine = running
            .as_ref()
            .and_then(|(ri, _, _)| *ri)
            .is_some_and(|ri| tied.contains(&ri));
        let login = run.is_some_and(|r| matches!(r.kind, RunKind::Login));
        let seg = match &running {
            Some((ri, who, d)) if !mine => {
                let rec = ri.map(|i| &self.model.orgs[i]);
                let left = d.or(rec.map(|o| o.left));
                Some(crate::layout::RunSeg {
                    level: left.map_or(Level::Ok, |l| self.model.left_level(l)),
                    who: who.clone(),
                    left,
                    estimated: d.is_none() && rec.is_some_and(|o| !o.left_known),
                })
            }
            _ => None,
        };
        HeaderQuota {
            left,
            level: self.model.left_level(left),
            estimated,
            busy: login || mine,
            run: seg,
        }
    }

    // ── keymap context ───────────────────────────────────────────────────────────────────────────

    /// The context stack, top first. Overlays, the confirm band and prompts are sealed.
    pub fn stack(&self) -> Vec<Ctx> {
        if let Some(o) = &self.ui.overlay {
            return vec![match o {
                Overlay::Help => Ctx::Help,
                Overlay::Palette { .. } => Ctx::Palette,
                Overlay::Picker { .. } => Ctx::Picker,
            }];
        }
        if self.ui.band.is_some() {
            return vec![Ctx::Confirm];
        }
        if self.cl_active() {
            let c = &self.ui.classify;
            if c.bulk.is_some() {
                return vec![Ctx::ClassifyBulk];
            }
            if c.picker.is_some() {
                return vec![Ctx::ClassifyPicker];
            }
            if c.note.is_some() {
                return vec![Ctx::ClassifyNote];
            }
            if c.pending_project {
                return vec![Ctx::ClassifyProject];
            }
            if c.catalog.diff.is_some() {
                return vec![Ctx::ClassifyDiff];
            }
            if c.catalog.editor.is_some() {
                return vec![Ctx::ClassifyEditor];
            }
        }
        if self.ui.tab == Tab::Manifest && self.ui.manifest_tab.prompt.is_some() {
            return vec![Ctx::ManifestPrompt];
        }
        match &self.ui.prompt {
            Some(Prompt::Filter(_)) => vec![Ctx::PromptFilter],
            Some(Prompt::AsOf(_)) => vec![Ctx::PromptAsOf],
            Some(Prompt::Range(_)) => vec![Ctx::PromptRange],
            None => self.view_stack(),
        }
    }

    /// The view's stack (what help lists and what palette keys are shown for).
    pub fn view_stack(&self) -> Vec<Ctx> {
        if self.ui.viz.is_some() {
            return vec![Ctx::Viz, Ctx::Global];
        }
        let view = match self.ui.tab {
            Tab::Ledger if self.cmp_active() => self.cmp_ctx(),
            Tab::Ledger => self.ledger_ctx(),
            Tab::Intercompany => self.ic_ctx(),
            Tab::Classify if self.cl_queue() => {
                return vec![Ctx::ClassifyQueue, Ctx::ClassifyMode, Ctx::Global];
            }
            Tab::Classify
                if self.ui.classify.mode == crate::classify::Mode::Catalog
                    && self.classify.is_some() =>
            {
                return vec![Ctx::ClassifyCatalog, Ctx::ClassifyMode, Ctx::Global];
            }
            Tab::Classify
                if self.ui.classify.mode == crate::classify::Mode::Groups
                    && self.classify.is_some() =>
            {
                return vec![Ctx::ClassifyGroups, Ctx::ClassifyMode, Ctx::Global];
            }
            Tab::Manifest if self.mf_diff_open() => {
                return vec![Ctx::ManifestDiff, Ctx::ManifestMode, Ctx::Global];
            }
            Tab::Manifest if self.mf_entries_mode() => {
                return vec![Ctx::ManifestEntries, Ctx::ManifestMode, Ctx::Global];
            }
            Tab::Manifest if self.mf_issues_mode() => {
                return vec![Ctx::ManifestIssues, Ctx::ManifestMode, Ctx::Global];
            }
            t => t.ctx(),
        };
        vec![view, Ctx::Global]
    }

    pub fn holds(&self, w: When) -> bool {
        if let Some(v) =
            crate::views::activity::holds(self, w).or_else(|| crate::views::system::holds(self, w))
        {
            return v;
        }
        match w {
            When::Always => true,
            When::HasRow if self.ui.viz.is_some() => true,
            When::HasRow => match self.ui.tab {
                Tab::Overview => !self.overview_rows().is_empty(),
                Tab::Ledger if self.cmp_active() => self.cmp_has_row(),
                Tab::Ledger => self.ledger_has_row(),
                Tab::Intercompany => self.ic_has_row(),
                Tab::Classify if self.ui.classify.mode == crate::classify::Mode::Catalog => {
                    self.cat_cursor().is_some()
                }
                Tab::Classify if self.ui.classify.mode == crate::classify::Mode::Groups => {
                    self.grp_cursor().is_some()
                }
                Tab::Classify => self.cl_queue() && !self.cl_rows().rows.is_empty(),
                Tab::Manifest => self.mf_has_row(),
                _ => false,
            },
            When::HasDocument => {
                (self.ui.tab == Tab::Ledger && self.ledger_has_doc())
                    || (self.ui.tab == Tab::Intercompany
                        && self.ic_level() == 3
                        && self.ic_doc().is_some())
            }
            When::LinksSection | When::SettingsSection => false,
            When::ScopeIsOrg => matches!(self.ui.scope, Scope::Org(_)),
            When::ScopeIsGroup => matches!(self.ui.scope, Scope::Group(_)),
            When::UsesAsOf => matches!(self.ui.tab, Tab::Ledger | Tab::Intercompany),
            When::NotOffline => !self.offline,
            When::ConfirmAsk => matches!(
                self.ui.band,
                Some(Band {
                    kind: BandKind::Ask { .. },
                    ..
                })
            ),
            When::HasRecord => self.ledger_org().is_some(),
            When::CanCompare => self.can_compare(),
            When::NoteMulti => self.ui.classify.note.as_ref().is_some_and(|n| n.multi),
            When::Staged => !self.ui.classify.catalog.staged.is_empty(),
            When::AgentWrites => {
                self.classify.is_some()
                    && (self.ui.tab == Tab::Classify
                        || matches!(self.agent.notice, Some(crate::agent::Notice::Wrote(_))))
            }
            When::RulesList => {
                self.ui.classify.catalog.list == crate::classify::catalog::CatList::Rules
            }
            When::MultiSelect => {
                matches!(self.ui.overlay, Some(Overlay::Picker { multi: true, .. }))
            }
            When::OnManifest => self.ui.manifest.is_some(),
            When::OnWarning => self.mf_on_warning(),
            When::HasCommand => self.mf_has_command(),
        }
    }

    /// The footer label of a binding (the basis toggle names its target mode).
    pub fn label_for(&self, action: &str, label: &'static str) -> &'static str {
        let refusal = matches!(
            self.ui.band,
            Some(Band {
                kind: BandKind::Refusal,
                ..
            })
        );
        match action {
            "basis.toggle" if self.ui.full_history => "Year basis",
            // the multi-select picker opens Compare; Enter is disabled outside 2–4
            "picker.choose" => match &self.ui.overlay {
                Some(Overlay::Picker {
                    multi: true,
                    chosen,
                    ..
                }) => {
                    if (2..=4).contains(&chosen.len()) {
                        "compare"
                    } else {
                        "compare (choose 2–4)"
                    }
                }
                _ => label,
            },
            // a refusal's only button is Close: Enter and Esc both close it
            "confirm.activate" | "confirm.cancel" if refusal => "close",
            "classify.note_enter" if self.ui.classify.note.as_ref().is_some_and(|n| n.multi) => {
                "new line"
            }
            "classify.note_multi" if self.ui.classify.note.as_ref().is_some_and(|n| n.multi) => {
                "one line"
            }
            _ => label,
        }
    }

    // ── rows ─────────────────────────────────────────────────────────────────────────────────────

    /// Overview rows (org indexes) after the `/` filter.
    pub fn overview_rows(&self) -> Vec<usize> {
        let f = crate::filter::Filter::parse(&self.ui.filter);
        (0..self.model.orgs.len())
            .filter(|&i| {
                let o = &self.model.orgs[i];
                f.matches(&format!("{}\n{}", o.alias, o.name), None)
            })
            .collect()
    }

    pub fn overview_cursor(&self) -> Option<usize> {
        let rows = self.overview_rows();
        let sel = self.ui.overview_sel.as_ref()?;
        rows.iter()
            .position(|&i| &self.model.orgs[i].alias == sel)
            .or((!rows.is_empty()).then_some(0))
    }

    fn select_overview_row(&mut self, pos: usize) {
        let rows = self.overview_rows();
        if let Some(&i) = rows.get(pos.min(rows.len().saturating_sub(1))) {
            self.ui.overview_sel = Some(self.model.orgs[i].alias.clone());
            self.keep_cursor_visible();
        }
    }

    fn content_height(&self) -> usize {
        self.hits
            .content
            .map_or(10, |r| (r.height as usize).saturating_sub(1).max(1))
    }

    fn keep_cursor_visible(&mut self) {
        let Some(cur) = self.overview_cursor() else {
            return;
        };
        let h = self.content_height();
        if cur < self.ui.overview_top {
            self.ui.overview_top = cur;
        } else if cur >= self.ui.overview_top + h {
            self.ui.overview_top = cur + 1 - h;
        }
    }

    // ── palette and picker ───────────────────────────────────────────────────────────────────────

    /// Palette entries (kit `ACTIONS` with `palette: true`, templates expanded per org and group).
    /// Network entries are hidden under `--offline` (M4).
    pub fn palette_entries(&self) -> Vec<PalEntry> {
        let stack = self.view_stack();
        let keys = |id: &str| keymap::keys_for_in(&self.keymap, &stack, id, &|w| self.holds(w));
        let mut out = Vec::new();
        for a in keymap::ACTIONS.iter().filter(|a| a.palette) {
            if a.network && self.offline {
                continue;
            }
            let running = self
                .ui
                .run
                .as_ref()
                .map(|r| format!("· running: {}", r.label()));
            let net = |label: String, n: NetAction| PalEntry {
                label,
                keys: None,
                network: true,
                // `· running: …` while a child runs; unported actions carry no suffix
                suffix: running.clone(),
                run: PalRun::Net(n),
                disabled: running.is_some(),
            };
            match a.id {
                "net.sync_org" => {
                    for (i, o) in self.model.orgs.iter().enumerate() {
                        out.push(net(format!("Sync {}…", o.alias), NetAction::SyncOrg(i)));
                    }
                }
                "net.sync_all" => {
                    out.push(net(a.label.into(), NetAction::SyncAll));
                    if let Scope::Group(g) = self.ui.scope {
                        out.push(net(
                            format!("Sync group:{}…", self.model.groups[g].name),
                            NetAction::SyncGroup(g),
                        ));
                    }
                }
                "net.login" => out.push(net(a.label.into(), NetAction::Login)),
                "net.refetch_tb" => out.push(net(a.label.into(), NetAction::RefetchTb)),
                // V16: listed so it can be found, but disabled and labelled while Jev is not enabled
                "classify.jev" => {
                    let enabled = self
                        .classify
                        .as_ref()
                        .is_some_and(|d| d.provider.config().jev.enabled);
                    let mut e = net(a.label.into(), NetAction::ClassifyJev);
                    // a running child says so first (one at a time); else why it can't run
                    if !enabled && running.is_none() {
                        e.suffix = Some("· Jev not enabled".into());
                        e.disabled = true;
                    }
                    out.push(e);
                }
                // Only with a document open.
                "net.fetch_history" if self.holds(When::HasDocument) => {
                    out.push(net(a.label.into(), NetAction::FetchHistory))
                }
                "net.fetch_attach" if self.holds(When::HasDocument) => {
                    out.push(net(a.label.into(), NetAction::FetchAttach))
                }
                "net.fetch_history" | "net.fetch_attach" => {}
                // only while a newer build can take over in place (spec: version footer §2)
                "app.upgrade" => {
                    if let Some(u) = self
                        .update_available
                        .as_ref()
                        .filter(|u| u.action == crate::upgrade::ACTION)
                    {
                        out.push(PalEntry {
                            label: format!("Upgrade to v{}", u.version),
                            keys: None,
                            network: false,
                            suffix: running.clone().map(|r| format!("{r} · applies after it")),
                            run: PalRun::Action(a.id),
                            disabled: false,
                        });
                    }
                }
                // The combined view needs a group scope.
                "combined.open" if !self.holds(When::ScopeIsGroup) => {}
                "scope.switch_to" => {
                    for (i, o) in self.model.orgs.iter().enumerate() {
                        out.push(PalEntry {
                            label: format!("Switch to {}", o.alias),
                            keys: None,
                            network: false,
                            suffix: None,
                            run: PalRun::Scope(Scope::Org(i)),
                            disabled: false,
                        });
                    }
                }
                "scope.switch_group" => {
                    for (g, grp) in self.model.groups.iter().enumerate() {
                        out.push(PalEntry {
                            label: format!("Switch to group:{}", grp.name),
                            keys: None,
                            network: false,
                            suffix: None,
                            run: PalRun::Scope(Scope::Group(g)),
                            disabled: false,
                        });
                    }
                }
                id => out.push(PalEntry {
                    label: a.label.into(),
                    keys: keys(id),
                    network: false,
                    suffix: None,
                    run: PalRun::Action(a.id),
                    disabled: false,
                }),
            }
        }
        out.extend(self.viz_palette());
        out
    }

    /// The palette after its filter: every query word must appear in the label (case-insensitive).
    pub fn palette_filtered(&self, query: &str) -> Vec<PalEntry> {
        let words: Vec<String> = query.split_whitespace().map(str::to_lowercase).collect();
        self.palette_entries()
            .into_iter()
            .filter(|e| {
                let l = e.label.to_lowercase();
                words.iter().all(|w| l.contains(w.as_str()))
            })
            .collect()
    }

    /// Picker rows, ranked when filtered: alias > alias prefix > short code > name word > fuzzy.
    pub fn picker_entries(&self, query: &str) -> Vec<PickEntry> {
        let mut rows: Vec<(u8, PickEntry)> = Vec::new();
        let q = query.trim().to_lowercase();
        let subseq = |hay: &str| {
            let mut it = hay.chars();
            q.chars().all(|c| it.any(|h| h == c))
        };
        for s in self.scopes() {
            let label = self.scope_label(&s);
            let (detail, short, name) = match &s {
                Scope::Org(i) => {
                    let o = &self.model.orgs[*i];
                    let fresh = self.model.freshness(&[*i], self.now);
                    (
                        format!(
                            "{} · {} · {}",
                            o.name,
                            fresh.full().trim_start_matches("record "),
                            crate::layout::quota_text(o.left, self.model.limit, !o.left_known, 0)
                        ),
                        o.short_code.to_lowercase(),
                        o.name.to_lowercase(),
                    )
                }
                _ => (
                    format!("{} orgs", self.members(&s).len()),
                    String::new(),
                    String::new(),
                ),
            };
            let l = label.to_lowercase();
            let bare = l.trim_start_matches("group:");
            let score = if q.is_empty() {
                0
            } else if l == q || bare == q {
                1
            } else if l.starts_with(&q) || bare.starts_with(&q) {
                2
            } else if !short.is_empty() && short.starts_with(&q) {
                3
            } else if name.split_whitespace().any(|w| w.starts_with(&q)) {
                4
            } else if subseq(&l) || subseq(&name) {
                5
            } else {
                continue;
            };
            rows.push((
                score,
                PickEntry {
                    current: s == self.ui.scope,
                    scope: s,
                    label,
                    detail,
                },
            ));
        }
        rows.sort_by_key(|(s, _)| *s);
        rows.into_iter().map(|(_, e)| e).collect()
    }

    // ── network actions (never reach Xero from the shell) ────────────────────────────────────────

    /// The pre-API placeholder (acceptance M10): the exact refusal text for each network action.
    pub fn placeholder_text(&self, a: NetAction) -> String {
        match a {
            NetAction::SyncOrg(i) => {
                format!("Sync with `ownpurse sync {}`", self.model.orgs[i].alias)
            }
            NetAction::SyncAll => "Sync with `ownpurse sync all`".into(),
            NetAction::Login => "Log in with `ownpurse login`".into(),
            NetAction::RefetchTb => NOT_AVAILABLE.into(),
            NetAction::FetchHistory | NetAction::FetchAttach => NOT_AVAILABLE.into(),
            NetAction::SyncGroup(g) => format!(
                "Sync with `ownpurse sync group:{}`",
                self.model.groups[g].name
            ),
            NetAction::QuitWhileRunning => String::new(),
            NetAction::ClassifyJev => JEV_NOT_ENABLED.into(),
            NetAction::UndoAgentWrite(_) => String::new(),
        }
    }

    /// A network action was chosen: offline refusal wins (M8); otherwise the placeholder band (M10).
    /// Neither produces an `Effect::Network`.
    /// A persistent refusal in the confirm band (error colour, one Close).
    pub(crate) fn refuse(&mut self, text: String) {
        self.ui.overlay = None;
        self.ui.band = Some(Band::refusal(text));
    }

    pub fn request_network(&mut self, a: NetAction) {
        self.ui.overlay = None;
        if self.offline {
            self.ui.band = Some(Band::refusal(OFFLINE_REFUSAL.to_string()));
            return;
        }
        if let Some(r) = &self.ui.run {
            // one child at a time: a disabled entry only flashes the status line
            self.ui.note = Some(format!("running: {}", r.label()));
            return;
        }
        match a {
            NetAction::SyncOrg(_)
            | NetAction::SyncAll
            | NetAction::SyncGroup(_)
            | NetAction::Login => {
                if self.cli.is_none() {
                    self.refuse(crate::net::NO_CLI.into());
                } else {
                    self.confirm_run(a);
                }
            }
            _ => self.ui.band = Some(Band::refusal(self.placeholder_text(a))),
        }
    }

    /// The confirm band for a network action: the real sync / login confirm.
    pub fn open_confirm(&mut self, a: NetAction, _estimate: Option<i64>) {
        self.confirm_run(a);
    }

    // ── actions ──────────────────────────────────────────────────────────────────────────────────

    fn set_tab(&mut self, t: Tab) {
        if t != Tab::Ledger {
            self.ui.via = None; // a via chain lives on the Ledger only
        }
        self.ui.tab = t;
        // Arriving never takes the cursor (navigation rule 1).
        self.ui.prompt = None;
    }

    fn set_scope(&mut self, s: Scope) {
        self.ui.scope = s;
    }

    pub(crate) fn set_scope_pub(&mut self, s: Scope) {
        self.set_scope(s);
    }

    pub(crate) fn set_tab_pub(&mut self, t: Tab) {
        self.set_tab(t);
    }

    pub(crate) fn type_char_pub(&mut self, c: char) {
        self.type_char(c);
    }

    pub(crate) fn click_pub(&mut self, t: Target, fx: &mut Vec<Effect>) {
        self.click(t, 1, fx);
    }

    fn phase4(&mut self, what: &str) {
        self.ui.note = Some(format!("{what}: coming in Phase 4"));
    }

    pub fn run_action(&mut self, action: &str, fx: &mut Vec<Effect>) {
        if let Some(Prompt::Range(text)) = self.ui.prompt.clone()
            && self.compare_range(action, &text)
        {
            return;
        }
        if self.cmp_active() && self.ui.prompt.is_none() && self.compare_action(action, fx) {
            return;
        }
        if self.viz_action(action, fx) {
            return;
        }
        if self.cl_action(action, fx) {
            return;
        }
        if self.mf_action(action, fx) {
            return;
        }
        if self.ui.tab == Tab::Ledger && self.ledger_action(action, fx) {
            return;
        }
        if self.ui.tab == Tab::Intercompany && self.ic_action(action, fx) {
            return;
        }
        if keymap::is_network(action) {
            // Never bound to a key (V3); reachable only from the palette via `PalRun::Net`.
            return;
        }
        if crate::views::activity::handle(self, action, fx)
            || crate::views::system::handle(self, action, fx)
        {
            return;
        }
        match action {
            "tab.overview" => self.set_tab(Tab::Overview),
            "tab.ledger" => self.set_tab(Tab::Ledger),
            "tab.intercompany" => self.set_tab(Tab::Intercompany),
            "tab.activity" => self.set_tab(Tab::Activity),
            "tab.system" => self.set_tab(Tab::System),
            "tab.classify" => self.set_tab(Tab::Classify),
            "tab.manifest" => self.set_tab(Tab::Manifest),
            // `U`: the latest agent batch, undone as one event attributed to the user (repeatable)
            "classify.undo_agent" => {
                if self.agent.writes.iter().any(|w| !w.undone) {
                    self.cl_write_pub(crate::classify::Op::UndoAgentBatch, fx);
                } else {
                    self.ui.note = Some("No agent write to undo.".into());
                }
            }
            "tab.next" => self.set_tab(Tab::ALL[(self.ui.tab.index() + 1) % Tab::ALL.len()]),
            "tab.prev" => {
                self.set_tab(Tab::ALL[(self.ui.tab.index() + Tab::ALL.len() - 1) % Tab::ALL.len()])
            }
            // the palette's mode entries
            "classify.queue" | "classify.groups" | "classify.audition" | "classify.tree"
            | "classify.catalog" => {
                self.set_tab(Tab::Classify);
                self.ui.classify.mode = match action {
                    "classify.groups" => crate::classify::Mode::Groups,
                    "classify.audition" => crate::classify::Mode::Audition,
                    "classify.tree" => crate::classify::Mode::Tree,
                    "classify.catalog" => crate::classify::Mode::Catalog,
                    _ => crate::classify::Mode::Queue,
                };
            }
            "app.quit" if self.ui.run.is_some() => self.confirm_quit_running(),
            "app.quit" => fx.push(Effect::Quit),
            "app.upgrade" => fx.push(Effect::Upgrade),
            // a filter that can't show on this screen is refused, never set silently (v0.10.1)
            "filter.open" => match self.filter_applies() {
                Ok(()) if self.cl_active() => {
                    self.ui.prompt = Some(Prompt::Filter(self.ui.classify.filter.clone()))
                }
                Ok(()) => self.ui.prompt = Some(Prompt::Filter(self.ui.filter.clone())),
                Err(s) => self.ui.note = Some(format!("A filter doesn't apply on {s}.")),
            },
            "scope.next" | "scope.prev" => {
                let all = self.scopes();
                let i = all.iter().position(|s| *s == self.ui.scope).unwrap_or(0);
                let n = all.len();
                let j = if action == "scope.next" {
                    (i + 1) % n
                } else {
                    (i + n - 1) % n
                };
                self.set_scope(all[j].clone());
            }
            "scope.picker" => {
                let sel = self
                    .picker_entries("")
                    .iter()
                    .position(|e| e.current)
                    .unwrap_or(0);
                self.ui.overlay = Some(Overlay::Picker {
                    query: String::new(),
                    sel,
                    multi: false,
                    chosen: Vec::new(),
                });
            }
            "asof.edit" => self.ui.prompt = Some(Prompt::AsOf(self.ui.as_of.clone())),
            "record.reload" => fx.push(Effect::Reload),
            "history.back" | "history.forward" => {
                let moved = if action == "history.back" {
                    self.history_back()
                } else {
                    self.history_forward()
                };
                self.hist_moved = true;
                if moved {
                    self.ui.note = Some(format!(
                        "{} → {}",
                        if action == "history.back" {
                            "back"
                        } else {
                            "forward"
                        },
                        self.view_summary()
                    ));
                } else {
                    self.ui.note = Some(if action == "history.back" {
                        "Nothing to go back to".into()
                    } else {
                        "Nothing to go forward to".into()
                    });
                }
            }
            "help.open" => self.ui.overlay = Some(Overlay::Help),
            "palette.open" => {
                self.ui.overlay = Some(Overlay::Palette {
                    query: String::new(),
                    sel: 0,
                })
            }
            "row.prev" | "row.next" | "page.prev" | "page.next" | "row.first" | "row.last" => {
                let cur = self.overview_cursor().unwrap_or(0);
                let n = self.overview_rows().len();
                let page = self.content_height();
                let pos = match action {
                    "row.prev" => cur.saturating_sub(1),
                    "row.next" => cur + 1,
                    "page.prev" => cur.saturating_sub(page),
                    "page.next" => cur + page,
                    "row.first" => 0,
                    _ => n.saturating_sub(1),
                };
                self.select_overview_row(pos);
            }
            "cell.prev" | "cell.next" => {
                let cols = [OvCol::Org, OvCol::ParityYear, OvCol::ParityHistory];
                let i = cols
                    .iter()
                    .position(|c| *c == self.ui.overview_col)
                    .unwrap_or(0);
                let j = if action == "cell.next" {
                    (i + 1).min(2)
                } else {
                    i.saturating_sub(1)
                };
                self.ui.overview_col = cols[j];
            }
            "drill.in" => {
                if self.ui.tab == Tab::Overview
                    && let Some(i) = self.overview_cursor().map(|p| self.overview_rows()[p])
                {
                    self.ui.chain.push(Origin {
                        from: Tab::Overview,
                        to: Tab::Ledger,
                        scope: self.ui.scope.clone(),
                        overview_sel: self.ui.overview_sel.clone(),
                        overview_col: self.ui.overview_col,
                    });
                    self.ui.scope = Scope::Org(i);
                    // M1: a parity cell lands on Ledger L1 with `!` on and the cell's basis.
                    match self.ui.overview_col {
                        OvCol::Org => {}
                        OvCol::ParityYear => {
                            self.ui.differs_only = true;
                            self.ui.full_history = false;
                        }
                        OvCol::ParityHistory => {
                            self.ui.differs_only = true;
                            self.ui.full_history = true;
                        }
                    }
                    self.nav_mut_pub().level = 1; // a chain lands on L1
                    self.set_tab(Tab::Ledger);
                }
            }
            "drill.back" => {
                if self.ui.chain.last().is_some_and(|o| o.to == self.ui.tab) {
                    let o = self.ui.chain.pop().expect("checked");
                    self.ui.scope = o.scope;
                    self.ui.overview_sel = o.overview_sel;
                    self.ui.overview_col = o.overview_col;
                    if o.overview_col != OvCol::Org {
                        self.ui.differs_only = false;
                        self.ui.full_history = false;
                    }
                    self.set_tab(o.from);
                } else if self.cl_active() {
                    self.ui.classify.filter.clear();
                } else if !self.ui.filter.is_empty() {
                    self.ui.filter.clear();
                }
            }
            "basis.toggle" => {
                self.ui.full_history = !self.ui.full_history;
                self.phase4(if self.ui.full_history {
                    "Full history"
                } else {
                    "Year basis"
                });
            }
            "filter.differs" => {
                self.ui.differs_only = !self.ui.differs_only;
                self.phase4("Differs filter");
            }
            "compare.open" => self.compare_open(),
            "combined.open" => self.phase4("Combined view"),
            "walk.open" => {
                self.ui.overlay = None;
                self.walk_list_open();
            }
            "view.lay" => {
                self.ui.overlay = None;
                self.ui.lay_mode = !self.ui.lay_mode;
                self.ui.note = Some(
                    if self.ui.lay_mode {
                        "lay mode: names first, why-it-matters lines"
                    } else {
                        "informed mode: codes and amounts"
                    }
                    .into(),
                );
            }
            "view.details" => {
                self.ui.details = !self.ui.details;
                self.ui.overlay = None;
            }
            "theme.overlay.clear" => {
                self.ui.overlay = None;
                if !self.ui.theme_overlay.is_empty() {
                    self.ui.theme_overlay.clear();
                    self.ui.theme_revision += 1;
                    self.ui.note = Some("theme overlay cleared".into());
                }
            }
            "theme.cycle" => {
                self.ui.theme = self.ui.theme.cycle();
                self.ui.overlay = None;
                self.ui.note = Some(format!("theme: {}", self.ui.theme.name()));
            }
            // confirm band (sealed; V3)
            "confirm.focus_next" | "confirm.focus_prev" => {
                if let Some(b) = &mut self.ui.band {
                    let n = b.buttons().len();
                    b.focus = (b.focus
                        + if action == "confirm.focus_next" {
                            1
                        } else {
                            n - 1
                        })
                        % n;
                }
            }
            "confirm.activate" => {
                if let Some(b) = self.ui.band.take()
                    && let (BandKind::Ask { action, .. }, 1) = (&b.kind, b.focus)
                {
                    match *action {
                        NetAction::QuitWhileRunning => fx.push(Effect::Quit),
                        NetAction::UndoAgentWrite(id) => {
                            if let Some(w) = self.agent.writes.iter().find(|w| w.log_id == id) {
                                let ids = w.events.clone();
                                self.cl_write_pub(crate::classify::Op::UndoEvents(ids), fx);
                            }
                        }
                        a => self.start_run(a, fx),
                    }
                }
            }
            "confirm.cancel" => self.ui.band = None,
            // overlays
            "overlay.close" => self.ui.overlay = None,
            "picker.prev" | "picker.next" | "palette.prev" | "palette.next" => {
                let n = match &self.ui.overlay {
                    Some(Overlay::Picker { query, .. }) => self.picker_entries(query).len(),
                    Some(Overlay::Palette { query, .. }) => self.palette_filtered(query).len(),
                    _ => 0,
                };
                let down = action.ends_with("next");
                if let Some(Overlay::Picker { sel, .. } | Overlay::Palette { sel, .. }) =
                    &mut self.ui.overlay
                    && n > 0
                {
                    *sel = if down {
                        (*sel + 1) % n
                    } else {
                        (*sel + n - 1) % n
                    };
                }
            }
            "picker.toggle" => {
                // multi-select (Compare): an org toggles; all or a group toggles its members
                if let Some(Overlay::Picker {
                    query, sel, chosen, ..
                }) = &self.ui.overlay
                    && let Some(scope) = self
                        .picker_entries(query)
                        .get(*sel)
                        .map(|e| e.scope.clone())
                {
                    let mut chosen = chosen.clone();
                    let members = self.members(&scope);
                    if members.iter().all(|m| chosen.contains(m)) {
                        chosen.retain(|c| !members.contains(c));
                    } else {
                        for m in members {
                            if !chosen.contains(&m) {
                                chosen.push(m);
                            }
                        }
                    }
                    if let Some(Overlay::Picker { chosen: c, .. }) = &mut self.ui.overlay {
                        *c = chosen;
                    }
                }
            }
            "picker.choose"
                if matches!(self.ui.overlay, Some(Overlay::Picker { multi: true, .. })) =>
            {
                if let Some(Overlay::Picker { chosen, .. }) = &self.ui.overlay {
                    let chosen = chosen.clone();
                    if (2..=4).contains(&chosen.len()) {
                        self.ui.overlay = None;
                        self.compare_start(chosen);
                    } else {
                        self.ui.note = Some(format!(
                            "Compare opens with 2–4 orgs ({} selected): space toggles",
                            chosen.len()
                        ));
                    }
                }
            }
            "picker.choose" => {
                if let Some(Overlay::Picker { query, sel, .. }) = &self.ui.overlay
                    && let Some(e) = self.picker_entries(query).get(*sel).cloned()
                {
                    self.ui.overlay = None;
                    self.set_scope(e.scope);
                }
            }
            "palette.run" => {
                if let Some(Overlay::Palette { query, sel }) = &self.ui.overlay
                    && let Some(e) = self.palette_filtered(query).get(*sel).cloned()
                {
                    self.ui.overlay = None;
                    match e.run {
                        PalRun::Net(n) => self.request_network(n),
                        PalRun::Scope(s) => self.set_scope(s),
                        PalRun::Action(a) => self.run_action(a, fx),
                        PalRun::Viz(spec) => self.viz_open(*spec),
                    }
                }
            }
            // prompts
            "prompt.apply" => match self.ui.prompt.take() {
                Some(Prompt::AsOf(text)) => {
                    let t = text.trim().to_string();
                    if t.len() == 10 && ownpurse_core::dates::parse_iso(&t).is_some() {
                        self.ui.as_of = t;
                    } else {
                        self.ui.prompt = Some(Prompt::AsOf(text));
                        self.ui.note = Some("as of needs a date: YYYY-MM-DD".into());
                    }
                }
                Some(Prompt::Filter(_) | Prompt::Range(_)) | None => {}
            },
            "prompt.cancel" => self.ui.prompt = None,
            "filter.clear" if self.cl_active() => {
                self.ui.classify.filter.clear();
                self.ui.classify.top = 0;
                self.ui.prompt = None;
            }
            "filter.clear" => {
                self.ui.filter.clear();
                self.ui.prompt = None;
            }
            // Phase 4 views
            other => {
                let label = keymap::BINDINGS
                    .iter()
                    .find(|b| b.action == other)
                    .map_or(other, |b| b.label);
                self.phase4(label);
            }
        }
    }

    /// The active input's text, mutably (a prompt, or the palette/picker query).
    fn input_text(&mut self) -> Option<&mut String> {
        if self.ui.overlay.is_none()
            && self.ui.band.is_none()
            && self.ui.tab == Tab::Manifest
            && self.ui.manifest_tab.prompt.is_some()
        {
            return self.mf_prompt_text();
        }
        if self.ui.overlay.is_none()
            && self.ui.band.is_none()
            && self.ui.tab == Tab::Classify
            && (self.ui.classify.picker.is_some()
                || self.ui.classify.note.is_some()
                || self.ui.classify.catalog.editor.is_some())
        {
            return self.cl_text_mut();
        }
        match (&mut self.ui.overlay, &mut self.ui.prompt) {
            (Some(Overlay::Palette { query, .. } | Overlay::Picker { query, .. }), _) => {
                Some(query)
            }
            (None, Some(Prompt::Filter(t) | Prompt::AsOf(t) | Prompt::Range(t))) => Some(t),
            _ => None,
        }
    }

    /// After the input's text changed: the filter follows its prompt live; finders restart at the top.
    fn input_changed(&mut self) {
        if self.ui.overlay.is_none()
            && self.ui.tab == Tab::Classify
            && let Some(p) = self.ui.classify.picker.as_mut()
        {
            p.sel = 0;
            return;
        }
        match (&mut self.ui.overlay, &self.ui.prompt) {
            (Some(Overlay::Palette { sel, .. } | Overlay::Picker { sel, .. }), _) => *sel = 0,
            (None, Some(Prompt::Filter(t))) if self.ui.tab == Tab::Classify => {
                self.ui.classify.filter = t.clone();
                self.ui.classify.top = 0;
            }
            (None, Some(Prompt::Filter(t))) => {
                self.ui.filter = t.clone();
                self.ui.overview_top = 0;
            }
            _ => {}
        }
    }

    fn type_char(&mut self, c: char) {
        let caret = self.ui.caret;
        let Some(text) = self.input_text() else {
            return;
        };
        let n = text.chars().count();
        let at = caret.unwrap_or(n).min(n);
        let byte = text.char_indices().nth(at).map_or(text.len(), |(b, _)| b);
        text.insert(byte, c);
        self.ui.caret = caret.map(|_| at + 1);
        self.input_changed();
    }

    fn backspace(&mut self) {
        let caret = self.ui.caret;
        let Some(text) = self.input_text() else {
            return;
        };
        let n = text.chars().count();
        let at = caret.unwrap_or(n).min(n);
        if at == 0 {
            return;
        }
        let byte = text
            .char_indices()
            .nth(at - 1)
            .map_or(text.len(), |(b, _)| b);
        text.remove(byte);
        self.ui.caret = caret.map(|_| at - 1);
        self.input_changed();
    }

    /// ←/→/Home/End inside a text input move the caret (None = at the end).
    fn move_caret(&mut self, code: KeyCode) {
        let caret = self.ui.caret;
        let Some(text) = self.input_text() else {
            return;
        };
        let n = text.chars().count();
        let at = caret.unwrap_or(n).min(n);
        let to = match code {
            KeyCode::Left => at.saturating_sub(1),
            KeyCode::Right => (at + 1).min(n),
            KeyCode::Home => 0,
            _ => n,
        };
        self.ui.caret = (to < n).then_some(to);
    }

    fn on_key(&mut self, ev: KeyEvent, fx: &mut Vec<Effect>) {
        self.ui.note = None;
        let key = Key::of(&ev);
        // the agent's notice and header mark clear on the user's next key (Esc only dismisses; ` goes back)
        let esc = key.code == KeyCode::Esc && key.mods.is_empty();
        let backtick = key.code == KeyCode::Char('`') && key.mods.is_empty();
        // Esc dismisses the agent's caption (and only that), as it dismisses its notice
        if esc
            && self.ui.caption.is_some()
            && self.ui.overlay.is_none()
            && self.ui.prompt.is_none()
            && self.ui.band.is_none()
        {
            self.ui.caption = None;
            self.agent.notice = None;
            self.agent.acted = false;
            self.dirty = true;
            return;
        }
        if self.agent_key(esc, backtick) {
            return;
        }
        // the walkthrough player, the glossary definition and the walkthrough list take their keys first
        if self.walk_key(&key, fx) {
            return;
        }
        // the run's bands never take focus (spec P6 §4–§5)
        if self.ui.overlay.is_none() && self.ui.band.is_none() && self.ui.prompt.is_none() {
            let esc = key.code == KeyCode::Esc && key.mods.is_empty();
            if let Some(r) = self.ui.result.take() {
                if esc {
                    return; // Esc only dismisses
                }
                if key.code == KeyCode::Char('y') && key.mods.is_empty() {
                    fx.push(Effect::Copy(r.copy));
                    return;
                }
                // any other key dismisses and does its normal action
            } else if esc && self.ui.run.as_ref().is_some_and(|r| r.band_visible) {
                if let Some(r) = self.ui.run.as_mut() {
                    r.band_visible = false; // Esc hides the band; it never stops the run
                }
                return;
            }
        }
        let ctxs = self.stack();
        let bound =
            keymap::lookup_in(&self.keymap, &ctxs, key, &|w| self.holds(w)).map(|b| b.action);
        if let Some(a) = bound {
            // `p` then a digit: the digit names the project (the one binding covers 1–9)
            if a == "classify.project_digit" {
                if let KeyCode::Char(c) = key.code {
                    self.cl_project_digit(c, fx);
                }
                return;
            }
            self.run_action(a, fx);
            return;
        }
        // `p` pending: any other key cancels it (and does nothing else)
        if ctxs[0] == Ctx::ClassifyProject {
            self.ui.classify.pending_project = false;
            return;
        }
        if keymap::types_text(ctxs[0]) {
            match key.code {
                KeyCode::Char(c) if key.mods.is_empty() => self.type_char(c),
                KeyCode::Backspace => self.backspace(),
                KeyCode::Left | KeyCode::Right | KeyCode::Home | KeyCode::End
                    if key.mods.is_empty() =>
                {
                    self.move_caret(key.code)
                }
                _ => {}
            }
        }
    }

    fn click(&mut self, t: Target, clicks: u8, fx: &mut Vec<Effect>) {
        match t {
            Target::Action(a) => self.run_action(a, fx),
            Target::Tab(t) => self.set_tab(t),
            Target::Term(i) => {
                let t = self
                    .ui
                    .narration
                    .as_ref()
                    .and_then(|n| crate::walk::terms(&n.body).get(i).cloned());
                self.ui.glossary_focus = t;
            }
            Target::WalkStep(i) => self.walk_jump(i, fx),
            Target::Row(i) if self.ui.tab == Tab::Intercompany => {
                self.ic_click_row(i);
                if clicks >= 2 {
                    self.run_action("drill.in", fx);
                }
            }
            Target::Cell(i, col) if self.cmp_active() => {
                self.cmp_click(i, Some(col));
                if clicks >= 2 {
                    self.run_action("drill.in", fx);
                }
            }
            Target::Row(i) if self.cmp_active() => {
                self.cmp_click(i, None);
                if clicks >= 2 {
                    self.run_action("drill.in", fx);
                }
            }
            Target::Row(i) if self.ui.tab == Tab::Ledger => {
                self.ledger_click_row(i);
                if clicks >= 2 {
                    self.run_action("drill.in", fx);
                }
            }
            Target::Row(i)
                if self.ui.tab == Tab::Classify
                    && self.ui.classify.mode == crate::classify::Mode::Groups
                    && self.ui.overlay.is_none() =>
            {
                self.grp_click_row(i);
            }
            Target::Row(i)
                if self.ui.tab == Tab::Classify
                    && self.ui.classify.mode == crate::classify::Mode::Catalog
                    && self.ui.overlay.is_none() =>
            {
                self.cat_click_row(i);
                if clicks >= 2 {
                    self.run_action("drill.in", fx);
                }
            }
            Target::Row(i) if self.cl_queue() && self.ui.overlay.is_none() => {
                let top = self.cl_top(self.cl_page());
                self.cl_click(top + i);
            }
            Target::Row(i) if crate::views::activity::owns(self) => {
                crate::views::activity::click_row(self, i, clicks, fx)
            }
            Target::Row(i) if crate::views::system::owns(self) => {
                crate::views::system::click_row(self, i, clicks, fx)
            }
            Target::Row(i) if self.ui.tab == Tab::Manifest && self.ui.overlay.is_none() => {
                self.ui.manifest_tab.sel = self.ui.manifest_tab.top + i;
                if clicks >= 2 {
                    self.run_action("drill.in", fx);
                }
            }
            Target::Row(i) => {
                self.select_overview_row(self.ui.overview_top + i);
                if clicks >= 2 {
                    self.run_action("drill.in", fx);
                }
            }
            Target::Cell(i, col) => {
                self.select_overview_row(self.ui.overview_top + i);
                self.ui.overview_col = match col {
                    2 => OvCol::ParityYear,
                    3 => OvCol::ParityHistory,
                    _ => OvCol::Org,
                };
                if clicks >= 2 {
                    self.run_action("drill.in", fx);
                }
            }
            Target::PaletteRow(i) => {
                if let Some(Overlay::Palette { sel, .. }) = &mut self.ui.overlay {
                    *sel = i;
                }
                self.run_action("palette.run", fx);
            }
            Target::PickerRow(i) => {
                if let Some(Overlay::Picker { sel, .. }) = &mut self.ui.overlay {
                    *sel = i;
                }
                self.run_action("picker.choose", fx);
            }
            Target::HelpRow(a) => {
                self.ui.overlay = None;
                self.run_action(a, fx);
            }
            Target::Button(i)
                if self.ui.band.is_none() && self.ui.classify.catalog.diff.is_some() =>
            {
                if let Some(d) = self.ui.classify.catalog.diff.as_mut() {
                    d.focus = i.min(1);
                }
                self.run_action("catalog.diff_go", fx);
            }
            Target::Button(i) if self.ui.band.is_none() && self.ui.classify.bulk.is_some() => {
                if let Some(b) = self.ui.classify.bulk.as_mut() {
                    b.focus = i.min(1);
                }
                self.run_action("classify.bulk_go", fx);
            }
            Target::Button(i) => {
                if let Some(b) = &mut self.ui.band {
                    b.focus = i;
                }
                self.run_action("confirm.activate", fx);
            }
        }
    }

    fn on_mouse(&mut self, ev: MouseEvent, clicks: u8, fx: &mut Vec<Effect>) {
        let p = Position {
            x: ev.column,
            y: ev.row,
        };
        match ev.kind {
            MouseEventKind::Down(MouseButton::Left) => {
                if self.ui.overlay.is_some() {
                    // A click outside an overlay closes it and does nothing underneath.
                    if self.hits.overlay.is_none_or(|r| !r.contains(p)) {
                        self.ui.overlay = None;
                        return;
                    }
                }
                let Some(t) = self.hits.at(p.x, p.y) else {
                    return;
                };
                if self.ui.band.is_some() && !matches!(t, Target::Button(_)) {
                    return; // the band is modal: only its buttons answer
                }
                // a Classify picker, note or preview is modal too (its buttons answer; tabs and rows don't)
                if self.ui.tab == Tab::Classify
                    && self.ui.classify.busy().is_some()
                    && !matches!(t, Target::Button(_))
                {
                    return;
                }
                self.click(t, clicks, fx);
            }
            MouseEventKind::ScrollDown | MouseEventKind::ScrollUp
                if self.ui.overlay.is_none() && self.ui.tab == Tab::Intercompany =>
            {
                self.ic_wheel(ev.kind == MouseEventKind::ScrollDown);
            }
            MouseEventKind::ScrollDown | MouseEventKind::ScrollUp
                if self.ui.overlay.is_none() && self.ui.tab == Tab::Ledger =>
            {
                self.ledger_wheel(ev.kind == MouseEventKind::ScrollDown);
            }
            MouseEventKind::ScrollDown | MouseEventKind::ScrollUp
                if self.ui.overlay.is_none() && self.cl_queue() =>
            {
                self.cl_wheel(ev.kind == MouseEventKind::ScrollDown);
            }
            MouseEventKind::ScrollDown | MouseEventKind::ScrollUp => {
                let down = ev.kind == MouseEventKind::ScrollDown;
                if self.ui.overlay.is_none()
                    && (crate::views::activity::wheel(self, down)
                        || crate::views::system::wheel(self, down))
                {
                    return;
                }
                if self.ui.overlay.is_some() || self.ui.tab != Tab::Overview {
                    return;
                }
                let n = self.overview_rows().len();
                let h = self.content_height();
                let max_top = n.saturating_sub(h);
                self.ui.overview_top = if ev.kind == MouseEventKind::ScrollDown {
                    (self.ui.overview_top + 1).min(max_top)
                } else {
                    self.ui.overview_top.saturating_sub(1)
                };
                // The wheel never moves the selection unless it left the view.
                if let Some(cur) = self.overview_cursor() {
                    let top = self.ui.overview_top;
                    if cur < top {
                        self.select_overview_row(top);
                    } else if cur >= top + h {
                        self.select_overview_row(top + h - 1);
                    }
                }
            }
            _ => {}
        }
    }
}

/// The one transition. Marks the app dirty only if what a frame shows changed.
pub fn update(app: &mut App, msg: Msg) -> Vec<Effect> {
    let before = app.ui.clone();
    let id_before = app.identity();
    let snap_before = crate::agent::ViewSnap::of(&app.ui);
    app.hist_moved = false;
    let input_before = app.input_state().map(|i| i.field);
    let mut fx = Vec::new();
    match msg {
        Msg::Key(ev) => app.on_key(ev, &mut fx),
        Msg::Mouse { ev, clicks } => app.on_mouse(ev, clicks, &mut fx),
        Msg::Resize(w, h) => app.ui.size = (w, h),
        Msg::Tick(t) => {
            if t.div_euclid(60) != app.now.div_euclid(60) {
                app.dirty = true;
            }
            app.now = t;
        }
        Msg::Reloaded(fresh) => {
            let (model, books) = *fresh;
            let n = model.orgs.len();
            let g = model.groups.len();
            app.model = model;
            app.books = books;
            // fresh books: the manifest side is re-attached (the record moved; changes are re-checked)
            app.manifests.attached = None;
            app.manifest_sync();
            match app.ui.scope {
                Scope::Org(i) if i >= n => app.ui.scope = Scope::All,
                Scope::Group(i) if i >= g => app.ui.scope = Scope::All,
                _ => {}
            }
            app.overview = None;
            app.ledger_view = Default::default();
            app.ic_view = Default::default();
            app.cmp_view = Default::default();
            // the classification store too: events another writer appended (zero calls; no write)
            if let Some(d) = app.classify.as_mut() {
                d.provider.refresh();
            }
            app.model_now = app.now;
            app.prepare();
            app.agent_relog();
            app.dirty = true;
            app.rev += 1;
        }
        Msg::Action(a) => {
            app.ui.note = None;
            app.run_action(a, &mut fx);
        }
        Msg::Run(ev) => app.on_run(ev, &mut fx),
        Msg::Agent(v) => app.apply_verb(v, &mut fx),
        Msg::AgentSend(vs) => app.apply_batch(vs, &mut fx),
        Msg::Classified(r) => {
            app.ui.note = Some(match r {
                Ok(s) => format!("{s} · u undo"),
                Err(e) => format!("Not written: {e}"),
            });
            app.dirty = true;
            app.rev += 1;
        }
        Msg::ConfigWritten(r) => {
            match r {
                Ok(s) => {
                    let c = &mut app.ui.classify.catalog;
                    c.staged.clear();
                    c.base_hash = None;
                    app.ui.note = Some(format!("wrote {s}"));
                }
                Err(e) => app.refuse(format!(
                    "classify.toml was not written: {e}. Your changes stay staged (w retries, u discards)."
                )),
            }
            app.dirty = true;
            app.rev += 1;
        }
        Msg::AgentClassify(req) => app.cl_agent_request(*req, &mut fx),
        Msg::AgentWrote(r) => {
            let caption = app.agent.pending_caption.take();
            app.cl_agent_wrote(*r, caption);
        }
        Msg::AgentUndone(r, ids) => app.cl_agent_undone(r, ids),
        Msg::UserConfig(ev) => app.user_config_event(ev),
        Msg::External(disk) => app.on_external(disk, &mut fx),
        Msg::ClassifyConfigChanged => {
            if app.cl_config_changed() {
                app.dirty = true;
                app.rev += 1;
            }
        }
    }
    app.agent_after_update();
    // navigation history and the caption follow the view identity (v0.10.2)
    if app.identity() != id_before {
        if !app.hist_moved {
            app.history_push(snap_before);
        }
        if app.ui.caption.as_ref().is_some_and(|c| !c.fresh) {
            app.ui.caption = None;
        }
        // a classify write's notice lasts as long as the view it landed on
        if matches!(app.agent.notice, Some(crate::agent::Notice::Wrote(_))) {
            app.agent.notice = None;
        }
    }
    if let Some(c) = app.ui.caption.as_mut() {
        c.fresh = false;
    }
    // a new or closed input starts with the caret at the end
    if app.input_state().map(|i| i.field) != input_before || input_before.is_none() {
        app.ui.caret = None;
    }
    // System's `cli:` row: `--version` once, when System is first shown (never on other navigation)
    if app.ui.tab == Tab::System && app.cli.is_some() && app.cli_version.is_none() {
        app.cli_version = Some(String::new());
        fx.push(Effect::CliVersion);
    }
    if app.ui != before {
        app.dirty = true;
        // a one-line note alone (a refusal, `Nothing to go back to`) is not a view change: no new revision
        let mut same = app.ui.clone();
        same.note = before.note.clone();
        if same != before {
            app.rev += 1;
        }
        app.prepare();
    }
    fx
}

/// Freshness level → the token that colours it (V12: the words carry it too).
pub fn level_token(l: Level) -> crate::theme_tokens::Token {
    match l {
        Level::Ok => crate::theme_tokens::Token::Text,
        Level::Warn => crate::theme_tokens::Token::Warn,
        Level::Err => crate::theme_tokens::Token::Error,
    }
}

/// The band an invalid user config file shows (it stays until that file's next valid save).
const USER_CONFIG_BAND: &str = "user config";

impl App {
    /// The keymap the state says (a render from state needs no keys.toml): defaults when empty or invalid.
    pub fn set_keys(
        &mut self,
        file: &crate::user_config::KeysFile,
    ) -> Result<(), crate::user_config::ConfigIssue> {
        let km = if file.bindings.is_empty() && file.unbind.is_empty() {
            crate::keymap::Keymap {
                file: file.clone(),
                revision: Some(file.hash.clone()).filter(|h| !h.is_empty()),
                ..crate::keymap::default_map().clone()
            }
        } else {
            crate::keymap::Keymap::build(file)?
        };
        self.keymap = Arc::new(km);
        Ok(())
    }

    /// A user-config event (the watcher, or the first load): a valid keys.toml swaps the keymap whole
    /// (footer, help, palette and `keys` follow on the next frame) with the note `keys reloaded (N
    /// overrides)`; an invalid file keeps the previous one and shows a band naming file:line, until that
    /// file's next valid save. Themes are kept for the theme layer.
    pub fn user_config_event(&mut self, ev: crate::user_config::ConfigEvent) {
        use crate::run::{ResultBand, ResultLevel};
        use crate::user_config::{Change, ConfigEvent};
        match ev {
            ConfigEvent::Reloaded(cfg, changes) => {
                let keys_changed = changes.contains(&Change::Keys);
                self.user_config = *cfg;
                self.sync_user_theme();
                if keys_changed {
                    let file = self.user_config.keys.clone();
                    match self.set_keys(&file) {
                        Ok(()) => {
                            let n = self.keymap.overrides;
                            self.ui.note = Some(format!(
                                "keys reloaded ({n} override{})",
                                if n == 1 { "" } else { "s" }
                            ));
                            self.cmd_notice();
                        }
                        Err(i) => self.user_config_event(ConfigEvent::Invalid(i)),
                    }
                }
                // a band about a file that is now valid goes
                if self
                    .ui
                    .result
                    .as_ref()
                    .is_some_and(|r| r.copy.starts_with(USER_CONFIG_BAND))
                    && changes.iter().any(|c| match c {
                        Change::Keys => self.ui.result.as_ref().is_some_and(|r| {
                            r.lines
                                .first()
                                .is_some_and(|l| l.starts_with(self.user_config.keys.name()))
                        }),
                        Change::Theme(n) => self.ui.result.as_ref().is_some_and(|r| {
                            r.lines
                                .first()
                                .is_some_and(|l| l.starts_with(&format!("themes/{n}.toml")))
                        }),
                    })
                {
                    self.ui.result = None;
                }
            }
            ConfigEvent::Invalid(i) => {
                let what = if !i.file.starts_with(crate::user_config::THEMES_DIR) {
                    "the previous key bindings stay live"
                } else {
                    "the previous theme stays live"
                };
                self.ui.result = Some(ResultBand {
                    level: ResultLevel::Error,
                    lines: vec![i.to_string(), format!("{what}; fix the file and save")],
                    copy: format!("{USER_CONFIG_BAND}: {i}"),
                });
            }
        }
        self.dirty = true;
        self.rev += 1;
    }

    /// Once: a ⌘ binding in a terminal that can't report ⌘.
    pub fn cmd_notice(&mut self) {
        if !self.cmd_keys && self.keymap.has_cmd() {
            self.ui.note =
                Some("Cmd bindings won't work in this terminal (no keyboard enhancement)".into());
        }
    }
}
