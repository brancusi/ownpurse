//! The view state (v0.10.1): ONE serialisable struct derived from the App (the Model and the view), so
//! the bridge never builds state JSON by hand and later versions can extend it (and import it). Its JSON
//! is `serde_json` of these structs; `canonical()` is core's canonical form (sorted keys, exact digits).
//! Only what the Model holds and the screen shows: visible rows only, never secrets.

use crate::app::{App, BandKind, Overlay, Prompt, Tab};
use crate::model::Level;
use serde::Serialize;
use serde_json::Value;

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ViewState {
    /// The state schema version (`canonical::STATE_V`).
    pub v: u32,
    /// The view revision this state belongs to.
    pub rev: u64,
    /// The running build (`CARGO_PKG_VERSION`; the footer's last item).
    pub version: String,
    /// The build identity (`g6dc723f`, `.dirty` with uncommitted changes); null for a build without git.
    pub build: Option<String>,
    /// A newer installed build: `{version, action}` (`restart` in v0.10.2), else null.
    pub update_available: Option<crate::canonical::Update>,
    /// This session's label (`--label`, `tui.label`, agent `label`).
    pub label: Option<String>,
    /// The clock the frame shows and the one the record was read at (UTC, ISO).
    pub clock: Clock,
    /// The record version the rows were derived from.
    pub record: crate::canonical::RecordVersion,
    /// Set when rendered from a state exported against another record version (`--allow-record-drift`).
    pub drift: Option<crate::canonical::Drift>,
    pub profile: String,
    /// overview | ledger | intercompany | activity | system
    pub tab: &'static str,
    pub level: u8,
    /// orgs | trial balance | postings | document | compare | compare postings | pairs | bridge | log |
    /// by endpoint | settings
    pub mode: &'static str,
    /// all | <alias> | group:<name>
    pub scope: String,
    pub as_of: String,
    /// year | history
    pub basis: &'static str,
    pub filter: String,
    /// Whether a filter means something on this screen (else `filter` is refused, `not_applicable`).
    pub filter_applies: bool,
    /// Rows the filter lets through (null without a filter or where it doesn't apply).
    pub filter_matches: Option<usize>,
    /// Rows before the filter on this screen (null where a filter doesn't apply).
    pub filter_total: Option<usize>,
    pub differs_only: bool,
    /// Intercompany's sort (null elsewhere).
    pub sort: Option<&'static str>,
    pub breadcrumb: Vec<String>,
    pub compare: Option<CompareState>,
    /// The selected row's anchor (or the open document's).
    pub selected: Option<Value>,
    /// The selected row's key label as `select` would name it (`1060 Tallowmere Green-Bean Card`); null with no row selected.
    pub selected_label: Option<String>,
    /// The rows on screen, top to bottom (or the whole list / a slice, when asked: `Rows`).
    pub rows: Vec<RowState>,
    pub view: ViewWindow,
    /// `rows` holds fewer than `view.total_rows` (the rest are off screen; `--all-rows` / `--rows a..b`).
    pub truncated: bool,
    pub strip: StripState,
    pub size: Size,
    pub theme: ThemeState,
    /// Derived from the scope (never stored): what the accent colour means here.
    pub accent: AccentState,
    /// Reserved: the keymap file's revision (null until v0.11.0).
    pub keymap: KeymapState,
    /// help | palette | picker
    pub overlay: Option<&'static str>,
    pub prompt: Option<PromptState>,
    pub band: Option<BandState>,
    /// The text being typed (a prompt, or the palette/picker query), with its caret.
    pub input: Option<InputState>,
    /// The command palette, when open: its query and highlighted entry.
    pub palette: Option<PaletteState>,
    /// The footer's transient note (until the next key).
    pub note: Option<String>,
    /// The agent's notice in the strip (`◆ agent: …` / skipped), until the user's next key.
    pub notice: Option<String>,
    /// The agent's caption: `{text, by, rev}`, null when cleared.
    pub caption: Option<crate::history::Caption>,
    /// The last run's result band, until the next key.
    pub result: Option<ResultState>,
    /// Navigation history (Alt+← / Alt+→): how far each way, and where the next step lands.
    pub history: HistoryState,
    /// The running sync or login (`sync design`, `login`).
    pub running: Option<RunningState>,
    /// What the user is in, when a send would be refused `user_busy`.
    pub busy: Option<String>,
    /// How many times this session refreshed for other processes' writes (record, store, config).
    pub data_refreshes: u64,
    /// The Classify tab (null on the other tabs): mode, filter, cursor, counts and any open editor.
    pub classify: Option<ClassifyState>,
    /// The shown manifest (null on main): name, stage, head_rev, entries, issues and the apply progress.
    pub manifest: Option<ManifestStateView>,
    /// Overlay layers (hints, rings, spotlights): id, owner, anchor and where each landed on this frame.
    pub layers: Vec<Value>,
    /// The overlay anchors drawn on this frame (`row:code=6140`, `chip:stage`, `diff:/Date`, …): what
    /// `hint`, `highlight` and `spotlight` can point at here.
    pub anchors: Vec<String>,
    /// The narration panel (an agent's `narrate`, or the walkthrough step), when shown.
    pub narration: Option<crate::walk::Narration>,
    /// The walkthrough playing: `{id, title, step, of, step_id, failed_checks}`.
    pub walkthrough: Option<Value>,
    /// A recording in progress: `{id, steps}`.
    pub recording: Option<Value>,
    /// Lay mode (`L`): names first, codes hidden, why-it-matters lines.
    pub lay_mode: bool,
    /// The open explainer panel (null with none): its spec, title, the parts' ids (`viz:<id>` anchors),
    /// scroll, and the error when it could not be computed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub viz: Option<VizStateView>,
    /// The importable canonical block (`ownpurse-tui --state`, `ui screen --from-state`). Agents read the
    /// fields above; this block is for restoring the exact view.
    pub canonical: Value,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ManifestIssues {
    pub errors: usize,
    pub warnings: usize,
    pub acknowledged: usize,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ManifestApply {
    pub sent: usize,
    pub landed: usize,
    pub verified: usize,
    pub rejected: usize,
    pub uncertain: usize,
}

/// `manifest {name, stage, head_rev, entries, issues {errors, warnings, acknowledged}, apply {…}}` (spec
/// manifests §5). There is no apply verb: agents read the stage and the commands, the owner runs them.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ManifestStateView {
    pub name: String,
    /// draft | review | check | planned | approved | applying | applied
    pub stage: &'static str,
    /// The header chip's short form (`7 changes · 1 issue`, `check ✗ 2`, `applying 3/7`).
    pub chip: String,
    pub head_rev: i64,
    /// Active entries.
    pub entries: usize,
    pub issues: ManifestIssues,
    pub apply: ManifestApply,
    /// The planned package's hash (planned onward).
    pub package_hash: Option<String>,
    /// The commands (planned / approved), exactly as the CLI prints them.
    pub approve_command: Option<String>,
    pub apply_command: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ClassifyState {
    /// queue | groups | audition | tree | catalog
    pub mode: &'static str,
    pub filter: String,
    /// Why the filter doesn't parse (the Queue then lists the unfiltered scope).
    pub filter_error: Option<String>,
    /// The cursor's line (`org:doc_type:doc_id:line_id`).
    pub selected_anchor: Option<String>,
    /// Reserved for Audition.
    pub audition_set: Option<String>,
    /// Across the store: lines still open, and lines with a stale label.
    pub open: Option<usize>,
    pub stale: Option<usize>,
    /// bearer | treatment | should_be_account picker, `note`, `bulk`, `project` (after `p`), or null.
    pub editor: Option<String>,
    /// The bulk preview, when open: threshold, lines, total.
    pub bulk: Option<Value>,
    /// Whether Jev may run (V16; always false in v0.11.0).
    pub jev_enabled: bool,
    /// The classification store's revision: agent writes send it as `expect_store_rev`.
    pub store_rev: i64,
    /// Agent writes this session not yet undone (`U` undoes the latest).
    pub agent_writes: usize,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Clock {
    pub now: String,
    pub model_now: String,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ThemeState {
    /// The built-in, or the user theme in use.
    pub name: String,
    /// unicode | ascii
    pub glyphs: &'static str,
    /// color | nocolor
    pub color: &'static str,
    /// Increments on each applied theme change (overlay, switch, user file reload); null before any.
    pub revision: Option<String>,
    /// The agent's cosmetic overlay (theme spec v2 §2.7): token → value, applied over the theme.
    pub overlay: ThemeOverlay,
    /// The effective accent intensities (`tui.accent_intensity` with any overlay `knob.*`), as decimals.
    pub knobs: serde_json::Map<String, Value>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct ThemeOverlay {
    pub tokens: serde_json::Map<String, Value>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct AccentState {
    /// org | pair | compare | group | all
    pub scope_kind: &'static str,
    /// The accent family the scope resolves to (an org's own, or the cross accent).
    pub token: String,
    /// The orgs in scope, each with its own accent family.
    pub members: Vec<AccentMember>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct AccentMember {
    pub alias: String,
    pub accent: String,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct KeymapState {
    /// The keys.toml content hash the keymap was built from; null for the built-in table.
    pub revision: Option<String>,
    /// Keys keys.toml adds or removes.
    pub overrides: usize,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct InputState {
    /// filter | as_of | range | palette | picker
    pub field: &'static str,
    pub text: String,
    /// In characters from the start.
    pub caret: usize,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct PaletteState {
    pub query: String,
    pub highlighted: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ResultState {
    /// ok | warn | error
    pub level: &'static str,
    pub lines: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct HistoryState {
    pub back: usize,
    pub forward: usize,
    /// Where `back` lands (the view's breadcrumb and filter), when there is one.
    pub back_to: Option<String>,
    pub forward_to: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct RunningState {
    /// `sync design`, `login`.
    pub label: String,
    /// The band's progress line (the last event).
    pub progress: String,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct RowState {
    /// 1-based place in the screen's whole (filtered) list: the `#n` of `select`.
    pub row: usize,
    pub anchor: Value,
    /// One key per column (`agent::field_key` of the header), valued as shown, plus raw extras.
    pub fields: Value,
    pub selected: bool,
}

/// Which part of the list is on screen (row numbers as in `rows[].row`).
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ViewWindow {
    pub first_visible: Option<usize>,
    pub last_visible: Option<usize>,
    pub total_rows: usize,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct CompareState {
    pub orgs: Vec<String>,
    pub align: String,
    pub level: u8,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct StripState {
    pub matches_xero: Option<bool>,
    pub record_age: String,
    pub record_level: &'static str,
    pub calls_left: i64,
    pub calls_left_estimated: bool,
    pub calls_limit: i64,
    pub calls_level: &'static str,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Size {
    pub cols: u16,
    pub rows: u16,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct PromptState {
    /// filter | as_of | range
    pub kind: &'static str,
    pub text: String,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct BandState {
    /// confirm | refusal
    pub kind: &'static str,
    pub text: String,
    pub buttons: Vec<String>,
    /// The focused button (0 = Cancel / Close).
    pub focus: usize,
}

/// Which rows a state carries: those on screen (the default), the whole list, or rows `a..=b` (1-based,
/// the `#n` of `select`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rows {
    Visible,
    All,
    Range(usize, usize),
}

impl Rows {
    /// `all` or `a..b` (also `a-b`); None for anything else.
    pub fn parse(s: &str) -> Option<Rows> {
        if s == "all" {
            return Some(Rows::All);
        }
        let (a, b) = s.split_once("..").or_else(|| s.split_once('-'))?;
        let (a, b) = (a.trim().parse().ok()?, b.trim().parse().ok()?);
        (a >= 1 && b >= a).then_some(Rows::Range(a, b))
    }
}

impl ViewState {
    /// Core's canonical JSON (sorted keys, compact, exact digits).
    pub fn canonical(&self) -> String {
        ownpurse_core::canon::canonical(&serde_json::to_value(self).unwrap_or(Value::Null))
    }
}

fn level_word(l: Level) -> &'static str {
    match l {
        Level::Ok => "ok",
        Level::Warn => "warn",
        Level::Err => "error",
    }
}

impl App {
    /// The view state for revision `rev`, derived from the Model and the last frame's rows.
    pub fn view_state(&self, rev: u64) -> ViewState {
        self.view_state_rows(rev, Rows::Visible)
    }

    /// The view state with the chosen rows (`Rows::All` / `Range` read the whole derived list; the view
    /// window still says what is on screen).
    pub fn view_state_rows(&self, rev: u64, which: Rows) -> ViewState {
        let f = self.header_facts();
        let tab = match self.ui.tab {
            Tab::Overview => "overview",
            Tab::Ledger => "ledger",
            Tab::Intercompany => "intercompany",
            Tab::Activity => "activity",
            Tab::System => "system",
            Tab::Classify => "classify",
            Tab::Manifest => "manifest",
        };
        let (level, mode) = match self.ui.tab {
            Tab::Ledger if self.cmp_active() => {
                let l = self.ui.cmp.as_ref().map_or(1, |c| c.level);
                (
                    l,
                    if l == 2 {
                        "compare postings"
                    } else {
                        "compare"
                    },
                )
            }
            Tab::Ledger => {
                let l = self.ledger_level();
                (
                    l,
                    match l {
                        2 => "postings",
                        3 => "document",
                        _ => "trial balance",
                    },
                )
            }
            Tab::Intercompany => {
                let l = self.ic_level();
                (
                    l,
                    match l {
                        2 => "bridge",
                        3 => "document",
                        _ => "pairs",
                    },
                )
            }
            Tab::Activity if self.ui.activity.grouped => (1, "by endpoint"),
            Tab::Activity => (1, "log"),
            Tab::System if self.ui.system.open_pkg.is_some() => (2, "package operations"),
            Tab::System => (1, "settings"),
            Tab::Overview => (1, "orgs"),
            Tab::Classify => (1, self.ui.classify.mode.word()),
            Tab::Manifest => (1, self.mf_mode().word()),
        };
        let numbered = self.visible_rows_numbered();
        let view = ViewWindow {
            first_visible: numbered.first().map(|r| r.3),
            last_visible: numbered.last().map(|r| r.3),
            total_rows: self.total_rows(),
        };
        let rows: Vec<RowState> = match which {
            Rows::Visible => numbered
                .into_iter()
                .map(|(t, anchor, fields, row)| RowState {
                    row,
                    anchor,
                    fields,
                    selected: self.is_selected(t),
                })
                .collect(),
            Rows::All | Rows::Range(..) => {
                let range = match which {
                    Rows::Range(a, b) => a - 1..b.min(view.total_rows),
                    _ => 0..view.total_rows,
                };
                self.rows_slice(range)
                    .into_iter()
                    .map(|(row, anchor, fields, selected)| RowState {
                        row,
                        anchor,
                        fields,
                        selected,
                    })
                    .collect()
            }
        };
        let truncated = rows.len() < view.total_rows;
        let fresh = self
            .model
            .freshness(&self.members(&self.ui.scope), self.now);
        let counts = self.filter_counts();
        ViewState {
            v: crate::canonical::STATE_V,
            rev,
            version: self.version_label.clone(),
            build: self.build_label.clone(),
            update_available: self.update_available.clone(),
            label: self.label.clone(),
            clock: Clock {
                now: ownpurse_core::bridge::utc_stamp(self.now),
                model_now: ownpurse_core::bridge::utc_stamp(self.model_now),
            },
            record: self.canonical().record,
            drift: self.drift.clone(),
            profile: self.profile.clone(),
            tab,
            level,
            mode,
            scope: self.scope_label(&self.ui.scope),
            as_of: self.ui.as_of.clone(),
            basis: if self.ui.full_history {
                "history"
            } else {
                "year"
            },
            filter: self.ui.filter.clone(),
            filter_applies: self.filter_applies().is_ok(),
            filter_matches: counts
                .filter(|_| !self.ui.filter.is_empty())
                .map(|(n, _)| n),
            filter_total: counts.map(|(_, t)| t),
            differs_only: self.ui.differs_only,
            sort: (self.ui.tab == Tab::Intercompany)
                .then(|| crate::ic_nav::SORTS[self.ui.ic_sort % crate::ic_nav::SORTS.len()]),
            breadcrumb: crate::views::crumbs(self),
            compare: match (&self.ui.cmp, self.cmp_active()) {
                (Some(c), true) => Some(CompareState {
                    orgs: c.orgs.iter().map(|&i| self.org_alias(i)).collect(),
                    align: c.align.clone(),
                    level: c.level,
                }),
                _ => None,
            },
            selected: self.selected_anchor(),
            selected_label: self.selected_label(),
            rows,
            view,
            truncated,
            strip: StripState {
                matches_xero: match self.ui.tab {
                    Tab::Ledger if !self.cmp_active() => {
                        self.l1_model().and_then(|m| m.basis.matches)
                    }
                    _ => None,
                },
                record_age: fresh.full(),
                record_level: level_word(fresh.level),
                calls_left: f.left,
                calls_left_estimated: f.left_estimated,
                calls_limit: f.limit,
                calls_level: level_word(f.quota_level),
            },
            size: Size {
                cols: self.ui.size.0,
                rows: self.ui.size.1,
            },
            theme: ThemeState {
                name: self
                    .ui
                    .theme_user
                    .as_ref()
                    .map_or_else(|| self.ui.theme.name().to_string(), |(n, _)| n.clone()),
                glyphs: if self.ui.theme.ascii {
                    "ascii"
                } else {
                    "unicode"
                },
                color: if matches!(self.ui.theme.mode, crate::theme::Mode::NoColor) {
                    "nocolor"
                } else {
                    "color"
                },
                revision: (self.ui.theme_revision > 0).then(|| self.ui.theme_revision.to_string()),
                overlay: ThemeOverlay {
                    tokens: self
                        .ui
                        .theme_overlay
                        .iter()
                        .map(|(k, v)| (k.clone(), Value::String(v.clone())))
                        .collect(),
                },
                knobs: {
                    let k = self.ui.theme.palette().knobs;
                    crate::theme::def::KNOB_NAMES
                        .iter()
                        .map(|n| (n.to_string(), k.text(n)))
                        .collect()
                },
            },
            accent: self.accent(),
            keymap: KeymapState {
                revision: self.keymap.revision.clone(),
                overrides: self.keymap.overrides,
            },
            overlay: self.ui.overlay.as_ref().map(|o| match o {
                Overlay::Help => "help",
                Overlay::Palette { .. } => "palette",
                Overlay::Picker { .. } => "picker",
            }),
            prompt: self.ui.prompt.as_ref().map(|p| match p {
                Prompt::Filter(t) => PromptState {
                    kind: "filter",
                    text: t.clone(),
                },
                Prompt::AsOf(t) => PromptState {
                    kind: "as_of",
                    text: t.clone(),
                },
                Prompt::Range(t) => PromptState {
                    kind: "range",
                    text: t.clone(),
                },
            }),
            band: self.ui.band.as_ref().map(|b| BandState {
                kind: match b.kind {
                    BandKind::Ask { .. } => "confirm",
                    BandKind::Refusal => "refusal",
                },
                text: b.text.clone(),
                buttons: b.buttons(),
                focus: b.focus,
            }),
            input: self.input_state(),
            palette: match &self.ui.overlay {
                Some(Overlay::Palette { query, sel }) => Some(PaletteState {
                    query: query.clone(),
                    highlighted: self
                        .palette_filtered(query)
                        .get(*sel)
                        .map(|e| e.label.clone()),
                }),
                _ => None,
            },
            note: self.ui.note.clone(),
            notice: self.agent_notice_text().map(|(t, _)| t),
            caption: self.ui.caption.clone(),
            result: self.ui.result.as_ref().map(|r| ResultState {
                level: match r.level {
                    crate::run::ResultLevel::Ok => "ok",
                    crate::run::ResultLevel::Warn => "warn",
                    crate::run::ResultLevel::Error => "error",
                },
                lines: r.lines.clone(),
            }),
            history: HistoryState {
                back: self.ui.history.back.len(),
                forward: self.ui.history.forward.len(),
                back_to: self.ui.history.back.last().map(|s| self.snap_summary(s)),
                forward_to: self.ui.history.forward.last().map(|s| self.snap_summary(s)),
            },
            running: self.ui.run.as_ref().map(|r| RunningState {
                label: r.label(),
                progress: r.band_text(),
            }),
            busy: self.agent_busy(),
            data_refreshes: self.data_refreshes,
            // whenever classification is set up: agents need `store_rev` for their writes from any tab
            classify: self.classify.is_some().then(|| self.classify_state()),
            manifest: self.manifest_state_view(),
            layers: self
                .ui
                .layers
                .layers
                .iter()
                .map(|l| {
                    let mut v =
                        crate::overlay::landed(self.hits.plan.as_ref(), l, &self.hits.anchors);
                    v["owner"] = serde_json::to_value(&l.owner).unwrap_or_default();
                    v
                })
                .collect(),
            anchors: self.anchor_names(),
            narration: self.ui.narration.clone(),
            walkthrough: self.ui.walk.as_ref().map(|w| {
                serde_json::json!({"id": w.file().id, "title": w.file().title, "step": w.at() + 1,
                    "of": w.len(), "step_id": w.tour.step, "failed_checks": w.failed})
            }),
            recording: self
                .ui
                .recording
                .as_ref()
                .map(|r| serde_json::json!({"id": r.file.id, "steps": r.file.steps.len()})),
            lay_mode: self.ui.lay_mode,
            viz: self.viz_state_view(),
            canonical: self.canonical_value(),
        }
    }
}

impl App {
    /// The Classify tab as data (derived; the canonical block carries `ui.classify`).
    pub fn classify_state(&self) -> ClassifyState {
        let c = &self.ui.classify;
        let counts = self.cl_counts();
        ClassifyState {
            mode: c.mode.word(),
            filter: c.filter.clone(),
            filter_error: self.cl_rows().error,
            selected_anchor: self
                .cl_selected()
                .map(|r| r.anchor.to_string())
                .or_else(|| c.selected_anchor.clone()),
            audition_set: c.audition_set.clone(),
            open: counts.map(|k| k.open),
            stale: counts.map(|k| k.stale),
            editor: if let Some(p) = &c.picker {
                Some(p.label_type.clone())
            } else if c.note.is_some() {
                Some("note".into())
            } else if c.bulk.is_some() {
                Some("bulk".into())
            } else if c.pending_project {
                Some("project".into())
            } else {
                None
            },
            bulk: self.cl_bulk_preview().map(|p| {
                serde_json::json!({
                    "threshold": crate::classify::conf_text(p.threshold),
                    "lines": p.anchors.len(),
                    "total": ownpurse_core::money::fmt(p.total),
                    "total_value": crate::agent::value_of(&ownpurse_core::money::fmt(p.total)),
                })
            }),
            jev_enabled: self
                .classify
                .as_ref()
                .is_some_and(|d| d.provider.config().jev.enabled),
            store_rev: self.classify.as_ref().map_or(0, |d| d.provider.rev()),
            agent_writes: self.agent.writes.iter().filter(|w| !w.undone).count(),
        }
    }

    /// The text being typed and its caret, if an input is active.
    pub fn input_state(&self) -> Option<InputState> {
        if self.ui.overlay.is_none() && self.ui.band.is_none() && self.ui.tab == Tab::Classify {
            let c = &self.ui.classify;
            let found = c
                .picker
                .as_ref()
                .map(|p| ("classify_picker", &p.query))
                .or_else(|| c.note.as_ref().map(|n| ("note", &n.text)));
            if let Some((field, text)) = found {
                let n = text.chars().count();
                return Some(InputState {
                    field,
                    text: text.clone(),
                    caret: self.ui.caret.unwrap_or(n).min(n),
                });
            }
        }
        let (field, text) = match (&self.ui.overlay, &self.ui.prompt) {
            (Some(Overlay::Palette { query, .. }), _) => ("palette", query),
            (Some(Overlay::Picker { query, .. }), _) => ("picker", query),
            (None, Some(Prompt::Filter(t))) => ("filter", t),
            (None, Some(Prompt::AsOf(t))) => ("as_of", t),
            (None, Some(Prompt::Range(t))) => ("range", t),
            _ => return None,
        };
        let n = text.chars().count();
        Some(InputState {
            field,
            text: text.clone(),
            caret: self.ui.caret.unwrap_or(n).min(n),
        })
    }

    /// The scope accent (theme spec v2 §2.3): ONE derivation, a pure function of the scope (Compare, an open
    /// Intercompany pair and a via chain count as the pair or compare they show). Never stored; every renderer
    /// reads the accent through it (`repaint`).
    pub fn accent(&self) -> AccentState {
        let aliases: Vec<String> = self.model.orgs.iter().map(|o| o.alias.clone()).collect();
        let fam = crate::theme::accent::assign(&aliases, &self.source.accents);
        let member = |alias: String| AccentMember {
            accent: fam.get(&alias).cloned().unwrap_or_else(|| "harbor".into()),
            alias,
        };
        let cross = crate::theme::accent::cross(&self.source.accents);
        // a pair the screen shows: Compare with two orgs, an open Intercompany pair, a via chain from a bridge
        let pair_orgs = |id: &str| -> Option<Vec<String>> {
            let r = self.ic_result()?;
            r["pairs"]
                .as_array()?
                .iter()
                .find(|p| p["id"] == id)
                .map(|p| {
                    vec![
                        p["a_org"].as_str().unwrap_or("").to_string(),
                        p["b_org"].as_str().unwrap_or("").to_string(),
                    ]
                })
        };
        let (scope_kind, members): (&'static str, Vec<String>) = if let Some(v) = self.viz_orgs() {
            // an open explainer panel: the orgs it is about (their accents colour its parts)
            v
        } else if self.cmp_active() {
            let orgs: Vec<String> = self.ui.cmp.as_ref().map_or(vec![], |c| {
                c.orgs.iter().map(|&i| self.org_alias(i)).collect()
            });
            (if orgs.len() == 2 { "pair" } else { "compare" }, orgs)
        } else if let Some(p) = self
            .ui
            .via
            .as_ref()
            .filter(|_| self.ui.tab == Tab::Ledger)
            .and_then(|v| pair_orgs(&v.pair))
        {
            ("pair", p)
        } else if self.ui.tab == Tab::Intercompany
            && self.ic_level() >= 2
            && let Some(p) = self.ic_pair()
        {
            (
                "pair",
                vec![
                    p["a_org"].as_str().unwrap_or("").to_string(),
                    p["b_org"].as_str().unwrap_or("").to_string(),
                ],
            )
        } else {
            let m: Vec<String> = self
                .members(&self.ui.scope)
                .into_iter()
                .map(|i| self.org_alias(i))
                .collect();
            let kind = match self.ui.scope {
                crate::app::Scope::All => "all",
                crate::app::Scope::Org(_) => "org",
                crate::app::Scope::Group(_) => "group",
            };
            (kind, m)
        };
        let members: Vec<AccentMember> = members.into_iter().map(member).collect();
        let token = if scope_kind == "org" {
            members.first().map_or(cross.clone(), |m| m.accent.clone())
        } else {
            cross
        };
        // `ui send accent …` (overlay `accent`) wins over the derived accent until cleared (spec §5)
        let token = self
            .ui
            .theme_overlay
            .get("accent")
            .cloned()
            .unwrap_or(token);
        AccentState {
            scope_kind,
            token,
            members,
        }
    }

    /// Resolve the theme for the current inputs (built-in, knobs, the derived scope accent, the overlay) into
    /// `ui.theme.pal`. Called whenever the view may have changed (`prepare`) and before every render.
    pub fn repaint(&mut self) {
        let def = match self.user_theme() {
            Some(d) => {
                self.ui.theme.dark = d.dark;
                d
            }
            None => {
                let name = if self.ui.theme.dark {
                    "ledger-dark"
                } else {
                    "ledger-light"
                };
                crate::theme::def::builtin(name).unwrap_or_default()
            }
        };
        let dark = self.ui.theme.dark;
        let acc = self.accent();
        let members: Vec<String> = acc.members.iter().map(|m| m.accent.clone()).collect();
        let knobs = self.source.knobs[usize::from(!dark)];
        let pal =
            crate::theme::def::resolve(&def, knobs, &acc.token, &members, &self.ui.theme_overlay);
        self.ui.theme.pal = Some(pal);
    }

    /// A history entry's summary (its breadcrumb and filter).
    fn snap_summary(&self, s: &crate::agent::ViewSnap) -> String {
        let mut parts = vec![s.tab.label().to_string(), self.scope_label(&s.scope)];
        if s.tab == Tab::Ledger {
            let key = match &s.via {
                Some(v) => format!("via:{}:{}", v.pair, self.org_alias(v.org)),
                None => self.scope_label(&s.scope),
            };
            if let Some(n) = s.ledger.get(&key) {
                match n.level {
                    2 => parts.push(format!(
                        "postings {}",
                        n.account.clone().unwrap_or_default()
                    )),
                    3 => parts.push("document".into()),
                    _ => parts.push("trial balance".into()),
                }
            }
        }
        if !s.filter.is_empty() {
            parts.push(format!("filter “{}”", s.filter));
        }
        parts.join(" › ")
    }
}

impl App {
    /// The user theme in use (`ui.theme_user`), parsed from the text the state carries (cached by text).
    /// None: a built-in, or the text no longer parses (the built-in it would extend is drawn instead).
    pub fn user_theme(&self) -> Option<crate::theme::def::ThemeDef> {
        let (name, text) = self.ui.theme_user.as_ref()?;
        let mut c = self.theme_cache.borrow_mut();
        if c.as_ref().map(|(t, _)| t) != Some(text) {
            let file = format!("{}/{name}.toml", crate::user_config::THEMES_DIR);
            *c = Some((
                text.clone(),
                crate::theme::user::parse(&file, name, text).ok(),
            ));
        }
        c.as_ref().and_then(|(_, d)| d.clone())
    }

    /// The configured user theme (`tui.theme`, or the one in use) after the user config (re)loads: applied
    /// once it is valid, re-applied when its file changes (`theme.revision` increments, one note).
    pub fn sync_user_theme(&mut self) {
        let want = self
            .ui
            .theme_user
            .as_ref()
            .map(|(n, _)| n.clone())
            .or_else(|| {
                self.source
                    .theme_pref
                    .clone()
                    .filter(|p| crate::theme::def::builtin(p).is_none())
            });
        let Some(n) = want else { return };
        let Some(f) = self.user_config.themes.get(&n) else {
            return;
        };
        if self.ui.theme_user.as_ref().map(|(_, t)| t) == Some(&f.text) {
            return;
        }
        let reload = self.ui.theme_user.is_some();
        self.ui.theme_user = Some((n.clone(), f.text.clone()));
        self.ui.theme_revision += 1;
        if reload {
            self.ui.note = Some(format!("theme {n} reloaded"));
        }
    }
}

impl App {
    /// The manifest block of the view state.
    pub fn manifest_state_view(&self) -> Option<ManifestStateView> {
        let m = self.manifest()?;
        let f = self.manifest_facts();
        let n = m.active().count();
        let (stage, chip, issues, apply) = match &f {
            Some(f) => {
                let (chip, _) = f
                    .status
                    .stage
                    .short(n, f.report.errors() + f.report.unacked());
                let a = &f.status.apply;
                (
                    f.status.stage.word(),
                    chip,
                    ManifestIssues {
                        errors: f.report.errors(),
                        warnings: f.report.warnings(),
                        acknowledged: f.report.acknowledged(),
                    },
                    ManifestApply {
                        sent: a.sent,
                        landed: a.landed,
                        verified: a.verified,
                        rejected: a.rejected,
                        uncertain: a.uncertain,
                    },
                )
            }
            None => (
                "draft",
                crate::layout::plural_changes(n),
                ManifestIssues {
                    errors: 0,
                    warnings: 0,
                    acknowledged: 0,
                },
                ManifestApply {
                    sent: 0,
                    landed: 0,
                    verified: 0,
                    rejected: 0,
                    uncertain: 0,
                },
            ),
        };
        let cmds = self.manifest_commands();
        let approved = f.as_ref().is_some_and(|f| {
            !matches!(
                f.status.stage,
                ownpurse_core::manifest::stage::Stage::Planned
            )
        });
        Some(ManifestStateView {
            name: m.name.clone(),
            stage,
            chip,
            head_rev: m.head_rev,
            entries: n,
            issues,
            apply,
            package_hash: m.plan().map(|p| p.hash.clone()),
            approve_command: cmds
                .as_ref()
                .filter(|_| !approved)
                .map(|c| c.approve.clone()),
            apply_command: cmds.map(|c| c.apply),
        })
    }
}

/// `viz` in the state: what the explainer panel shows.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct VizStateView {
    /// bridge | flow | t_account | timeline | before_after | checklist
    pub kind: &'static str,
    pub spec: Value,
    pub title: Option<String>,
    /// Every part's id, in drawing order (`viz:<id>` anchors for overlays).
    pub parts: Vec<String>,
    pub scroll: u16,
    pub error: Option<String>,
}

impl App {
    pub fn viz_state_view(&self) -> Option<VizStateView> {
        let open = self.ui.viz.as_ref()?;
        let (title, parts, error) = match self.viz_model() {
            Some(Ok(m)) => (Some(m.title().to_string()), m.ids(), None),
            Some(Err(e)) => (None, vec![], Some(e.clone())),
            None => (None, vec![], None),
        };
        Some(VizStateView {
            kind: open.spec.kind(),
            spec: serde_json::to_value(&open.spec).unwrap_or_default(),
            title,
            parts,
            scroll: open.scroll,
            error,
        })
    }
}
