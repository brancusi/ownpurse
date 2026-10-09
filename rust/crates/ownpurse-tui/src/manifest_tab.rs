//! Tab 7 Manifest (design spec manifests §2–§4; branches §4): on main, the manifest list; on a manifest,
//! four modes (Entries · Issues · Diff · Log, `,` and `.`), the stage strip and the command panel, the
//! generic document diff of an entry (Enter), and the prompts (new manifest, note, edit, the plan confirm).
//! Draw lives in `views::manifest` and `views::diff`. There is NO apply action here (apply is CLI-only).

use serde::{Deserialize, Serialize};

/// The mode on a manifest.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Mode {
    #[default]
    Entries,
    Issues,
    Diff,
    Log,
    /// Every manifest (main first): the list main shows, and `b` / "Switch manifest…" on a manifest.
    Manifests,
}

impl Mode {
    pub const ALL: [Mode; 4] = [Mode::Entries, Mode::Issues, Mode::Diff, Mode::Log];
    pub fn word(self) -> &'static str {
        match self {
            Mode::Entries => "entries",
            Mode::Issues => "issues",
            Mode::Diff => "diff",
            Mode::Log => "log",
            Mode::Manifests => "manifests",
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Mode::Entries => "Entries",
            Mode::Issues => "Issues",
            Mode::Diff => "Diff",
            Mode::Log => "Log",
            Mode::Manifests => "Manifests",
        }
    }
}

/// An open prompt on tab 7 (sealed; types).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Prompt {
    /// `N`: a new manifest's name, then why (`field` 0 / 1).
    New {
        name: String,
        why: String,
        field: u8,
    },
    /// `n`: note an entry; an `accountant:` prefix sets the accountant reference, anything else is a note.
    Note { seq: i64, text: String },
    /// `e`: edit a new-journal entry (`CODE AMOUNT, CODE AMOUNT | narration`).
    Edit { seq: i64, text: String },
    /// `P`: the plan confirm (`[Cancel] [Plan]`, Cancel focused; no single-letter confirm).
    Plan { focus: u8 },
}

/// The open diff view's own state.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiffUi {
    /// The entry whose diff is open.
    pub seq: i64,
    /// `z`: every field (no collapsing).
    #[serde(default)]
    pub expanded: bool,
    /// `J`: the raw RFC 6902 ops.
    #[serde(default)]
    pub patch: bool,
    /// The focused row (a field or line path), from an issue jump or the cursor.
    #[serde(default)]
    pub focus: Option<String>,
    /// The focused row flashes once in the selection colour after an issue jump.
    #[serde(default)]
    pub flash: bool,
}

/// Tab 7's state (in the canonical state, so V21 and upgrades keep it).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ManifestUi {
    pub mode: Mode,
    /// The list cursor (entries, issues, log, or the manifest list), by row position.
    pub sel: usize,
    pub top: usize,
    #[serde(default)]
    pub prompt: Option<Prompt>,
    /// An entry's document diff open over the list (Enter; Esc returns).
    #[serde(default)]
    pub diff: Option<DiffUi>,
}

use crate::app::{App, Effect, Tab};
use ownpurse_core::manifest::check::{Issue, Report};
use ownpurse_core::manifest::fields::{self, DocDiff, Mark};
use ownpurse_core::manifest::ops::{Guard, JLine, Target};
use ownpurse_core::manifest::stage::EntryStatus;
use ownpurse_core::manifest::{Entry, Kind, Manifest, ManifestError};

/// One row of the entry list (conflicts and errors first, then by number; undone and edited stay, dimmed).
#[derive(Clone, Debug, PartialEq)]
pub struct EntryRow {
    pub entry: Entry,
    /// `+`, `~`, `⊘`, `≠` (a conflict), or empty (posted; undone).
    pub marker: &'static str,
    /// `2 fields · 1 line`, `new`.
    pub changes: String,
    /// `✗1 !2` or `✓`.
    pub badge: String,
    pub conflict: bool,
    pub status: EntryStatus,
}

impl EntryRow {
    pub fn word(&self) -> String {
        if let Some(n) = self.entry.replaced_by {
            return format!("edited → {n}");
        }
        if self.entry.undone {
            return "undone".into();
        }
        if self.conflict {
            return "≠ conflict".into();
        }
        String::new()
    }
}

/// The open document diff, prepared for draw.
#[derive(Clone, Debug, PartialEq)]
pub struct DiffDoc {
    pub seq: i64,
    pub entity: String,
    pub org: String,
    pub id: String,
    pub actor: String,
    /// `create`, `change` or `void`.
    pub kind: &'static str,
    pub diff: DocDiff,
    pub issues: Vec<Issue>,
    /// One line per patch op (`J`).
    pub patch: Vec<String>,
    /// The entry no longer applies (main moved): old vs new.
    pub conflict: Option<ownpurse_core::view::Conflict>,
    pub status: EntryStatus,
    /// How a reconciled line's change is carried out (`via reclass journal …`, `via the browser …`).
    pub via: Option<String>,
    /// The document is a reconciled bank transaction (a chip on the title row, even collapsed).
    pub reconciled: bool,
}

impl DiffDoc {
    /// The rows the cursor walks, in draw order: field paths, then line paths.
    pub fn paths(&self, expanded: bool) -> Vec<String> {
        let mut v: Vec<String> = visible_fields(&self.diff, expanded, self.kind == "create")
            .into_iter()
            .filter_map(|r| r.map(|f| f.path.clone()))
            .collect();
        if let Some(l) = &self.diff.lines {
            v.extend(
                visible_lines(&l.rows, expanded, self.kind == "create")
                    .into_iter()
                    .filter_map(|r| r.map(|x| x.path.clone())),
            );
        }
        v
    }
}

/// The fields shown: every changed one, at most 2 unchanged around each change, `None` = `… n unchanged`.
pub fn visible_fields(d: &DocDiff, expanded: bool, new: bool) -> Vec<Option<&fields::FieldRow>> {
    collapse(&d.fields, |f| f.mark != Mark::Same, 2, expanded || new)
}

/// The lines shown: every changed one, at most 3 unchanged (in total), `None` = `… n unchanged lines`.
pub fn visible_lines(
    rows: &[fields::LineRow],
    expanded: bool,
    new: bool,
) -> Vec<Option<&fields::LineRow>> {
    if expanded || new {
        return rows.iter().map(Some).collect();
    }
    let mut out = vec![];
    let mut same_shown = 0;
    let mut hidden = false;
    for r in rows {
        if r.mark != Mark::Same || same_shown < 3 {
            if r.mark == Mark::Same {
                same_shown += 1;
            }
            out.push(Some(r));
        } else if !hidden {
            hidden = true;
            out.push(None);
        }
    }
    out
}

fn collapse<T>(rows: &[T], changed: impl Fn(&T) -> bool, ctx: usize, all: bool) -> Vec<Option<&T>> {
    if all || rows.iter().all(&changed) {
        return rows.iter().map(Some).collect();
    }
    let near: Vec<bool> = (0..rows.len())
        .map(|i| {
            let lo = i.saturating_sub(ctx);
            let hi = (i + ctx).min(rows.len().saturating_sub(1));
            (lo..=hi).any(|j| changed(&rows[j]))
        })
        .collect();
    let mut out = vec![];
    let mut gap = false;
    for (i, r) in rows.iter().enumerate() {
        if near[i] {
            out.push(Some(r));
            gap = false;
        } else if !gap {
            out.push(None);
            gap = true;
        }
    }
    out
}

/// How many unchanged fields a collapsed gap hides (draw says `… n unchanged fields`).
pub fn hidden_count(d: &DocDiff, expanded: bool, new: bool) -> usize {
    let shown = visible_fields(d, expanded, new)
        .iter()
        .filter(|r| r.is_some())
        .count();
    d.fields.len() - shown
}

impl App {
    /// The mode tab 7 shows: the manifest list on main.
    pub fn mf_mode(&self) -> Mode {
        if self.ui.manifest.is_none() {
            Mode::Manifests
        } else {
            self.ui.manifest_tab.mode
        }
    }

    pub fn mf_entries_mode(&self) -> bool {
        self.ui.tab == Tab::Manifest
            && self.mf_mode() == Mode::Entries
            && self.ui.manifest_tab.diff.is_none()
    }

    pub fn mf_issues_mode(&self) -> bool {
        self.ui.tab == Tab::Manifest
            && self.mf_mode() == Mode::Issues
            && self.ui.manifest_tab.diff.is_none()
    }

    pub fn mf_diff_open(&self) -> bool {
        self.ui.tab == Tab::Manifest && self.ui.manifest_tab.diff.is_some()
    }

    fn report(&self) -> Report {
        self.manifest_facts()
            .map(|f| f.report.clone())
            .unwrap_or_default()
    }

    /// The entry rows: conflicts and errored entries first, then by number.
    pub fn mf_rows(&self) -> Vec<EntryRow> {
        let Some(m) = self.manifest() else {
            return vec![];
        };
        let ov = self.manifest_overlay();
        let facts = self.manifest_facts();
        let report = self.report();
        let mut rows: Vec<EntryRow> =
            m.entries
                .iter()
                .filter(|e| e.replaced_by.is_none())
                .map(|e| {
                    let a = ov.and_then(|o| o.applied(e.seq));
                    let posted = ov.is_some_and(|o| o.is_posted(e.seq));
                    let conflict = ov.is_some_and(|o| o.conflict(e.seq).is_some());
                    let marker = if !e.active() || posted {
                        ""
                    } else if conflict {
                        "≠"
                    } else {
                        match a.map(|a| a.kind) {
                            Some("create") => "+",
                            Some("void") => "⊘",
                            _ => "~",
                        }
                    };
                    let changes = match a {
                        _ if e.is_new() => "new".to_string(),
                        Some(a) => fields::doc_diff(&e.entity, a.kind, a.before.as_ref(), &a.after)
                            .summary(),
                        None => {
                            let n = e.paths().len();
                            format!("{n} change{}", if n == 1 { "" } else { "s" })
                        }
                    };
                    EntryRow {
                        marker,
                        changes,
                        badge: if e.active() {
                            report.entry_badge(e.seq)
                        } else {
                            String::new()
                        },
                        conflict,
                        status: facts
                            .as_ref()
                            .map(|f| f.status.of(e.seq).clone())
                            .unwrap_or(EntryStatus::None),
                        entry: e.clone(),
                    }
                })
                .collect();
        rows.sort_by_key(|r| {
            let errors = report.for_entry(r.entry.seq).any(|i| i.blocking());
            (!(r.conflict || (errors && r.entry.active())), r.entry.seq)
        });
        rows
    }

    /// The issue rows: errors first, then warnings.
    pub fn mf_issues(&self) -> Vec<Issue> {
        self.report().issues
    }

    /// The manifest list rows (main first, then open manifests, then history).
    pub fn mf_list(&self) -> Vec<Option<Manifest>> {
        let mut v: Vec<Manifest> = self
            .manifests
            .store
            .as_ref()
            .map(|s| s.manifests())
            .unwrap_or_default();
        v.sort_by_key(|b| (b.discarded, b.created_at.clone()));
        std::iter::once(None)
            .chain(v.into_iter().map(Some))
            .collect()
    }

    fn mf_len(&self) -> usize {
        if let Some(d) = &self.ui.manifest_tab.diff {
            return self.mf_diff().map_or(0, |doc| doc.paths(d.expanded).len());
        }
        match self.mf_mode() {
            Mode::Entries => self.mf_rows().len(),
            Mode::Issues => self.mf_issues().len(),
            Mode::Manifests => self.mf_list().len(),
            Mode::Log => self.mf_log().len(),
            Mode::Diff => 0,
        }
    }

    pub fn mf_has_row(&self) -> bool {
        self.mf_len() > 0
    }

    pub fn mf_cursor(&self) -> Option<usize> {
        let n = self.mf_len();
        (n > 0).then(|| self.ui.manifest_tab.sel.min(n - 1))
    }

    pub fn mf_selected(&self) -> Option<EntryRow> {
        if self.mf_mode() != Mode::Entries {
            return None;
        }
        self.mf_cursor()
            .and_then(|i| self.mf_rows().into_iter().nth(i))
    }

    pub fn mf_selected_issue(&self) -> Option<Issue> {
        if let Some(d) = &self.ui.manifest_tab.diff {
            let doc = self.mf_diff()?;
            let path = d.focus.clone()?;
            return doc
                .issues
                .iter()
                .find(|i| {
                    !i.blocking() && (i.path == path || i.path.starts_with(&format!("{path}/")))
                })
                .cloned();
        }
        if self.mf_mode() != Mode::Issues {
            return None;
        }
        self.mf_cursor()
            .and_then(|i| self.mf_issues().into_iter().nth(i))
    }

    /// The selected issue (or the diff's focused field) is an unacknowledged warning.
    pub fn mf_on_warning(&self) -> bool {
        self.mf_selected_issue()
            .is_some_and(|i| !i.blocking() && i.ack.is_none())
    }

    /// The command panel shows a command (planned or approved).
    pub fn mf_has_command(&self) -> bool {
        use ownpurse_core::manifest::stage::Stage;
        self.ui.tab == Tab::Manifest
            && self
                .manifest_facts()
                .is_some_and(|f| matches!(f.status.stage, Stage::Planned | Stage::Approved))
    }

    /// The open entry's document diff (the generic component's model).
    pub fn mf_diff(&self) -> Option<DiffDoc> {
        let seq = self.ui.manifest_tab.diff.as_ref()?.seq;
        self.mf_diff_of(seq)
    }

    pub fn mf_diff_of(&self, seq: i64) -> Option<DiffDoc> {
        let m = self.manifest()?;
        let e = m.entry(seq)?;
        let ov = self.manifest_overlay()?;
        let a = ov.applied(seq);
        let report = self.report();
        let (kind, diff, before) = match a {
            Some(a) => (
                a.kind,
                fields::doc_diff(&e.entity, a.kind, a.before.as_ref(), &a.after),
                a.before.clone(),
            ),
            None => {
                // a conflict or an undone / posted entry: show its patch against main now, if it applies
                let tid = self.manifest_ctx().tenant(&e.org)?.to_string();
                let before = if e.is_new() {
                    None
                } else {
                    self.books
                        .as_ref()?
                        .view()
                        .main_document(&tid, &e.entity, &e.id)
                        .ok()
                        .flatten()
                        .map(|(_, d)| ownpurse_core::manifest::normalise::normalise(&d))
                };
                let after = e
                    .after(before.as_ref())
                    .unwrap_or_else(|_| before.clone().unwrap_or_default());
                let kind = if e.is_new() { "create" } else { "change" };
                (
                    kind,
                    fields::doc_diff(&e.entity, kind, before.as_ref(), &after),
                    before,
                )
            }
        };
        Some(DiffDoc {
            seq,
            entity: e.entity.clone(),
            org: e.org.clone(),
            id: e.id.clone(),
            actor: e.actor.clone(),
            kind,
            diff,
            issues: report.for_entry(seq).cloned().collect(),
            patch: ownpurse_core::manifest::patch_lines(e, before.as_ref()),
            conflict: ov.conflict(seq).cloned(),
            reconciled: before
                .as_ref()
                .and_then(|b| b.get("IsReconciled"))
                .and_then(serde_json::Value::as_bool)
                == Some(true),
            via: a.and_then(|a| match &a.matched {
                ownpurse_core::manifest::rules::Matched::Op("reclass_journal") => Some(format!(
                    "via reclass journal {}: Xero refuses edits of reconciled lines, so {}'s {} reclass journal carries this account change (Dr new / Cr old)",
                    ownpurse_core::manifest::compile::reclass_op_id(&a.org, &a.date),
                    a.org,
                    a.date.chars().take(7).collect::<String>()
                )),
                ownpurse_core::manifest::rules::Matched::Op("void_recreate") => Some(format!(
                    "via void + re-create: voids {}, creates its replacement (both in the package; voiding shows in Xero's history)",
                    e.id
                )),
                ownpurse_core::manifest::rules::Matched::Op("browser_recode") => Some(
                    "via the browser: unreconcile, edit, re-reconcile in Xero (a checklist step, never automated)".into(),
                ),
                _ => None,
            }),
            status: self
                .manifest_facts()
                .map(|f| f.status.of(seq).clone())
                .unwrap_or(EntryStatus::None),
        })
    }

    /// The log rows: (seq, at, actor, kind, summary), newest last.
    pub fn mf_log(&self) -> Vec<(i64, String, String, String, String)> {
        let (Some(name), Some(s)) = (self.ui.manifest.as_ref(), self.manifests.store.as_ref())
        else {
            return vec![];
        };
        let m = self.manifest();
        s.events()
            .iter()
            .filter(|e| &e.manifest == name)
            .map(|e| {
                let what = match e.kind {
                    Kind::EntryAdd => m
                        .and_then(|m| m.entry(e.id))
                        .map(|en| {
                            format!("{} {} {}", en.org, fields::entity_label(&en.entity), en.id)
                        })
                        .unwrap_or_default(),
                    Kind::EntryUndo => format!(
                        "{} entry {}",
                        if e.payload["undone"] == true {
                            "undo"
                        } else {
                            "restore"
                        },
                        e.payload["seq"]
                    ),
                    Kind::EntryAnnotate => format!("note on entry {}", e.payload["seq"]),
                    Kind::IssueAck => format!(
                        "{} {} on entry {}",
                        if e.payload["acked"] == false {
                            "withdraw"
                        } else {
                            "acknowledge"
                        },
                        e.payload["code"].as_str().unwrap_or(""),
                        e.payload["seq"]
                    ),
                    Kind::Planned => format!(
                        "planned · sha256 {}…",
                        &e.payload["hash"].as_str().unwrap_or("")
                            [..12.min(e.payload["hash"].as_str().unwrap_or("").len())]
                    ),
                    Kind::Create => {
                        format!("created · {}", e.payload["why"].as_str().unwrap_or(""))
                    }
                    Kind::Discard => "discarded".into(),
                    Kind::Resolve => format!(
                        "resolve {} → {}",
                        e.payload["seq"]
                            .as_i64()
                            .map_or("default".to_string(), |s| format!("entry {s}")),
                        e.payload["resolution"].as_str().unwrap_or("")
                    ),
                };
                (
                    e.id,
                    e.at.get(..16).unwrap_or(&e.at).replace('T', " "),
                    e.actor.clone(),
                    e.kind.as_str().to_string(),
                    what,
                )
            })
            .collect()
    }

    /// The prompt's text field (typing goes here).
    pub fn mf_prompt_text(&mut self) -> Option<&mut String> {
        match self.ui.manifest_tab.prompt.as_mut()? {
            Prompt::New { name, why, field } => Some(if *field == 0 { name } else { why }),
            Prompt::Note { text, .. } | Prompt::Edit { text, .. } => Some(text),
            Prompt::Plan { .. } => None,
        }
    }

    /// Open an entry's diff (optionally focused on a field, flashing once).
    pub fn mf_open_diff(&mut self, seq: i64, focus: Option<String>) {
        let flash = focus.is_some();
        self.ui.manifest_tab.diff = Some(DiffUi {
            seq,
            expanded: false,
            patch: false,
            focus: focus.clone(),
            flash,
        });
        // the cursor lands on the focused row; a collapsed field is expanded so it shows
        if let (Some(p), Some(doc)) = (focus, self.mf_diff()) {
            let mut paths = doc.paths(false);
            if !paths
                .iter()
                .any(|x| x == &p || p.starts_with(&format!("{x}/")))
            {
                if let Some(d) = self.ui.manifest_tab.diff.as_mut() {
                    d.expanded = true;
                }
                paths = doc.paths(true);
            }
            let i = paths
                .iter()
                .position(|x| x == &p || p.starts_with(&format!("{x}/")))
                .unwrap_or(0);
            self.ui.manifest_tab.sel = i;
        } else {
            self.ui.manifest_tab.sel = 0;
        }
    }

    /// Tab 7's keys, the palette's manifest actions and the list's navigation. True when handled.
    pub fn mf_action(&mut self, action: &str, fx: &mut Vec<Effect>) -> bool {
        let _ = &fx;
        match action {
            "manifest.switch" => {
                self.ui.overlay = None;
                self.set_tab_pub(Tab::Manifest);
                self.ui.manifest_tab.mode = Mode::Manifests;
                self.ui.manifest_tab.diff = None;
                self.ui.manifest_tab.sel = 0;
                return true;
            }
            "manifest.make_default" => {
                self.ui.overlay = None;
                let name = self.ui.manifest.clone();
                if let Some(p) = &self.source.manifests
                    && p.set_current(name.as_deref()).is_ok()
                {
                    self.ui.note = Some(format!(
                        "default manifest: {} (the CLI follows it)",
                        name.as_deref().unwrap_or("main")
                    ));
                }
                return true;
            }
            "manifest.to_main" => {
                self.ui.overlay = None;
                let _ = self.switch_manifest(None, false);
                self.ui.note = Some("showing main".into());
                return true;
            }
            _ => {}
        }
        if self.ui.tab != Tab::Manifest || self.ui.overlay.is_some() || self.ui.band.is_some() {
            return false;
        }
        // the prompt (sealed)
        if self.ui.manifest_tab.prompt.is_some() {
            match action {
                "manifest.prompt_cancel" => self.ui.manifest_tab.prompt = None,
                "manifest.prompt_next" => match self.ui.manifest_tab.prompt.as_mut() {
                    Some(Prompt::New { field, .. }) => *field = 1 - *field,
                    Some(Prompt::Plan { focus }) => *focus = 1 - *focus,
                    _ => {}
                },
                "manifest.prompt_save" => self.mf_prompt_save(),
                _ => return false,
            }
            self.ui.caret = None;
            return true;
        }
        let n = self.mf_len();
        match action {
            "manifest.mode_next" | "manifest.mode_prev" if self.ui.manifest.is_some() => {
                let all = Mode::ALL;
                let i = all
                    .iter()
                    .position(|m| *m == self.ui.manifest_tab.mode)
                    .unwrap_or(0);
                let j = if action == "manifest.mode_next" {
                    (i + 1) % all.len()
                } else {
                    (i + all.len() - 1) % all.len()
                };
                self.ui.manifest_tab.mode = all[j];
                self.ui.manifest_tab.diff = None;
                self.ui.manifest_tab.sel = 0;
                self.ui.manifest_tab.top = 0;
            }
            "manifest.picker" => {
                self.ui.manifest_tab.mode = Mode::Manifests;
                self.ui.manifest_tab.diff = None;
                self.ui.manifest_tab.sel = 0;
            }
            "manifest.new" => {
                self.ui.manifest_tab.prompt = Some(Prompt::New {
                    name: String::new(),
                    why: String::new(),
                    field: 0,
                });
            }
            "manifest.check" => {
                self.manifests.facts = None;
                self.prepare_manifest();
                if let Some(f) = self.manifest_facts() {
                    self.ui.note = Some(format!("checked · {}", f.report.summary()));
                }
            }
            "manifest.plan" => {
                let report = self.report();
                if report.errors() > 0 {
                    self.ui.note = Some(format!(
                        "Not planned: {} error{} block planning (Issues, `.`)",
                        report.errors(),
                        if report.errors() == 1 { "" } else { "s" }
                    ));
                } else if report.unacked() > 0 {
                    self.ui.note = Some(format!(
                        "Not planned: {} warning{} need acknowledging (Issues, `a`)",
                        report.unacked(),
                        if report.unacked() == 1 { "" } else { "s" }
                    ));
                } else if self.manifest().is_some_and(|m| m.active().count() == 0) {
                    self.ui.note = Some("Not planned: the manifest has no entries".into());
                } else {
                    self.ui.manifest_tab.prompt = Some(Prompt::Plan { focus: 0 });
                }
            }
            "manifest.copy_command" => {
                if let Some(c) = self.manifest_commands() {
                    let approved = self.manifest_facts().is_some_and(|f| {
                        f.status.stage == ownpurse_core::manifest::stage::Stage::Approved
                    });
                    let text = if approved { c.apply } else { c.approve };
                    fx.push(Effect::Copy(text));
                    self.ui.note = Some(if approved {
                        "copied the apply command · run it in your own terminal".into()
                    } else {
                        "copied the approve command · the owner runs it at his own terminal".into()
                    });
                }
            }
            "row.prev" | "row.next" | "page.prev" | "page.next" | "row.first" | "row.last" => {
                let cur = self.mf_cursor().unwrap_or(0);
                let page = 10;
                let pos = match action {
                    "row.prev" => cur.saturating_sub(1),
                    "row.next" => cur + 1,
                    "page.prev" => cur.saturating_sub(page),
                    "page.next" => cur + page,
                    "row.first" => 0,
                    _ => n.saturating_sub(1),
                };
                self.ui.manifest_tab.sel = pos.min(n.saturating_sub(1));
                if self.ui.manifest_tab.diff.is_some()
                    && let Some(doc) = self.mf_diff()
                {
                    let exp = self
                        .ui
                        .manifest_tab
                        .diff
                        .as_ref()
                        .is_some_and(|d| d.expanded);
                    let p = doc.paths(exp).get(self.ui.manifest_tab.sel).cloned();
                    if let Some(d) = self.ui.manifest_tab.diff.as_mut() {
                        d.focus = p;
                        d.flash = false;
                    }
                }
            }
            "drill.in" if self.ui.manifest_tab.diff.is_none() => self.mf_enter(),
            "drill.back" if self.ui.manifest_tab.diff.is_some() => {
                let seq = self.ui.manifest_tab.diff.take().map(|d| d.seq);
                // back on the entry it came from
                if self.ui.manifest_tab.mode == Mode::Entries
                    && let Some(i) = self.mf_rows().iter().position(|r| Some(r.entry.seq) == seq)
                {
                    self.ui.manifest_tab.sel = i;
                } else {
                    self.ui.manifest_tab.sel = 0;
                }
            }
            "drill.back"
                if self.ui.manifest.is_some() && self.ui.manifest_tab.mode == Mode::Manifests =>
            {
                self.ui.manifest_tab.mode = Mode::Entries;
                self.ui.manifest_tab.sel = 0;
            }
            "manifest.collapse" if self.ui.manifest_tab.diff.is_some() => {
                if let Some(d) = self.ui.manifest_tab.diff.as_mut() {
                    d.expanded = !d.expanded;
                }
            }
            "manifest.patch_view" if self.ui.manifest_tab.diff.is_some() => {
                if let Some(d) = self.ui.manifest_tab.diff.as_mut() {
                    d.patch = !d.patch;
                }
            }
            "manifest.ack" => {
                if let Some(i) = self.mf_selected_issue().filter(|i| !i.blocking()) {
                    let acked = i.ack.is_none();
                    match self.manifest_ack(
                        i.seq,
                        i.code,
                        &i.path,
                        acked,
                        "human",
                        &Guard::default(),
                    ) {
                        Ok(_) => {
                            self.ui.note = Some(if acked {
                                format!(
                                    "acknowledged {} on entry {} · a again to withdraw",
                                    i.code, i.seq
                                )
                            } else {
                                format!(
                                    "withdrew the acknowledgement of {} on entry {}",
                                    i.code, i.seq
                                )
                            })
                        }
                        Err(e) => self.ui.note = Some(e.message),
                    }
                }
            }
            "manifest.undo" => {
                if let Some(r) = self.mf_selected() {
                    let undone = !r.entry.undone;
                    match self.manifest_undo(r.entry.seq, undone, "human", &Guard::default()) {
                        Ok(_) => {
                            self.ui.note = Some(if undone {
                                format!(
                                    "undid entry {} · {} {} · u again to restore",
                                    r.entry.seq, r.entry.org, r.entry.id
                                )
                            } else {
                                format!("restored entry {}", r.entry.seq)
                            })
                        }
                        Err(e) => self.ui.note = Some(e.message),
                    }
                }
            }
            "manifest.annotate" => {
                if let Some(r) = self.mf_selected() {
                    let gated = self
                        .manifest_overlay()
                        .and_then(|o| o.applied(r.entry.seq))
                        .is_some_and(|a| a.gated);
                    let text = match (&r.entry.refs.accountant_decision, gated) {
                        (Some(a), _) => format!("accountant: {a}"),
                        (None, true) => "accountant: ".into(),
                        _ => r.entry.note.clone().unwrap_or_default(),
                    };
                    self.ui.manifest_tab.prompt = Some(Prompt::Note {
                        seq: r.entry.seq,
                        text,
                    });
                }
            }
            "manifest.edit" => {
                if let Some(r) = self.mf_selected() {
                    match self.mf_diff_of(r.entry.seq) {
                        Some(doc) if r.entry.is_new() && r.entry.entity == "ManualJournals" => {
                            let after = self
                                .manifest_overlay()
                                .and_then(|o| o.applied(r.entry.seq))
                                .map(|a| a.after.clone())
                                .unwrap_or_default();
                            let ls: Vec<String> = after["JournalLines"]
                                .as_object()
                                .into_iter()
                                .flatten()
                                .map(|(_, l)| {
                                    format!(
                                        "{} {}",
                                        l["AccountCode"].as_str().unwrap_or(""),
                                        l["LineAmount"].as_str().unwrap_or("")
                                    )
                                })
                                .collect();
                            let _ = doc;
                            self.ui.manifest_tab.prompt = Some(Prompt::Edit {
                                seq: r.entry.seq,
                                text: format!(
                                    "{} | {}",
                                    ls.join(", "),
                                    after["Narration"].as_str().unwrap_or("")
                                ),
                            });
                        }
                        _ => {
                            self.ui.note = Some(
                                "New journals are edited here; change other documents with a new entry (it combines)."
                                    .into(),
                            )
                        }
                    }
                }
            }
            _ => return false,
        }
        true
    }

    fn mf_enter(&mut self) {
        match self.mf_mode() {
            Mode::Manifests => {
                let list = self.mf_list();
                let Some(row) = self.mf_cursor().and_then(|i| list.into_iter().nth(i)) else {
                    return;
                };
                match row {
                    None => {
                        let _ = self.switch_manifest(None, false);
                        self.ui.note = Some("showing main".into());
                    }
                    Some(m) if !m.discarded => match self.switch_manifest(Some(&m.name), false) {
                        Ok(()) => {
                            self.ui.manifest_tab.mode = Mode::Entries;
                            self.ui.manifest_tab.sel = 0;
                            self.ui.note = Some(format!(
                                "⎇ {} · {}",
                                m.name,
                                crate::layout::plural_changes(m.active().count())
                            ));
                        }
                        Err(e) => self.ui.note = Some(e.message),
                    },
                    Some(m) => {
                        self.ui.note = Some(format!("{} is discarded: read-only history", m.name))
                    }
                }
            }
            Mode::Entries => {
                if let Some(r) = self.mf_selected() {
                    self.mf_open_diff(r.entry.seq, None);
                }
            }
            Mode::Issues => {
                if let Some(i) = self
                    .mf_cursor()
                    .and_then(|c| self.mf_issues().into_iter().nth(c))
                    && i.seq > 0
                {
                    self.mf_open_diff(i.seq, Some(i.path.clone()));
                }
            }
            _ => {}
        }
    }

    fn mf_prompt_save(&mut self) {
        let Some(p) = self.ui.manifest_tab.prompt.clone() else {
            return;
        };
        let r: Result<String, ManifestError> = match p {
            Prompt::New { name, why, .. } => {
                let name = name.trim().to_string();
                self.manifest_create(&name, why.trim(), "human").map(|_| {
                    self.ui.manifest_tab.mode = Mode::Entries;
                    self.ui.manifest_tab.sel = 0;
                    format!("⎇ {name} created · entries stay local until planned, approved and applied (CLI)")
                })
            }
            Prompt::Note { seq, text } => {
                let t = text.trim();
                let (acct, note) = match t.strip_prefix("accountant:") {
                    Some(r) => (Some(r.trim()), None),
                    None => (None, Some(t)),
                };
                self.manifest_annotate(seq, acct, note, "human", &Guard::default())
                    .map(|_| format!("noted entry {seq}"))
            }
            Prompt::Edit { seq, text } => {
                let old = self
                    .manifest_overlay()
                    .and_then(|o| o.applied(seq))
                    .map(|a| (a.org.clone(), a.after.clone()));
                match (old, parse_edit(&text)) {
                    (Some((org, after)), Ok((lines, narration))) => {
                        let doc = ownpurse_core::manifest::ops::journal_doc(
                            after["Date"].as_str().unwrap_or(""),
                            &narration,
                            after["ShowOnCashBasisReports"].as_bool().unwrap_or(true),
                            &lines,
                        );
                        let id = self
                            .manifest()
                            .and_then(|m| m.entry(seq))
                            .map(|e| e.id.clone());
                        let t = Target {
                            entity: "ManualJournals".into(),
                            org,
                            id,
                        };
                        self.manifest_put(&t, &doc, "", Some(seq), "human", &Guard::default())
                            .map(|id| format!("entry {seq} edited → {id}"))
                    }
                    (None, _) => Err(ManifestError::new("usage", "that entry is not in the view")),
                    (_, Err(e)) => Err(ManifestError::new("usage", e)),
                }
            }
            Prompt::Plan { focus } => {
                if focus == 0 {
                    // Cancel is focused: Enter cancels
                    self.ui.manifest_tab.prompt = None;
                    self.ui.note = Some("not planned".into());
                    return;
                }
                self.manifest_plan("human").map(|(_, hash)| {
                    format!(
                        "planned · sha256 {}… · nothing was sent; the panel shows the approve command",
                        &hash[..12]
                    )
                })
            }
        };
        match r {
            Ok(note) => {
                self.ui.manifest_tab.prompt = None;
                self.ui.note = Some(note);
            }
            // the prompt stays open with what was typed; the note says why
            Err(e) => self.ui.note = Some(format!("Not saved: {}", e.message)),
        }
    }
}

/// `CODE AMOUNT, CODE AMOUNT | narration` → the lines and narration of a new journal.
pub fn parse_edit(text: &str) -> Result<(Vec<JLine>, String), String> {
    let (ls, narration) = text.split_once('|').unwrap_or((text, ""));
    let mut lines = vec![];
    for part in ls.split(',').map(str::trim).filter(|p| !p.is_empty()) {
        let mut it = part.split_whitespace();
        let (Some(code), Some(amt)) = (it.next(), it.next()) else {
            return Err(format!("`{part}`: expected CODE AMOUNT"));
        };
        let amount =
            ownpurse_core::money::parse(amt).map_err(|_| format!("`{amt}` is not an amount"))?;
        lines.push(JLine {
            account: code.into(),
            description: String::new(),
            amount,
        });
    }
    Ok((lines, narration.trim().to_string()))
}
