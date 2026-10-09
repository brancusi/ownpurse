//! ownpurse TUI default keymap (pasted from the design kit, `rust-design-kit/keymap.rs`; extensions marked `// shell:`). Design source: ownpurse-rust-tui-spec.md §6, §15–16.
//! One table drives dispatch, the footer, help, the palette and `ownpurse keys`. Nothing else binds keys.
//!
//! Key notation: lowercase names; chords `ctrl-p`; `shift-tab`; printable keys as themselves ("!", "[", ":").
//! A plain key matches only with no ctrl/alt (exact matching). Uppercase "G" means shift-g.
//! `footer`: 1 shows first; None = never in the footer (still in help and palette).
//! `when`: the condition for the binding to be live AND shown (footer, help).
//! Network actions (`network: true` in ACTIONS) MUST have no binding here (test `network_has_no_keys`).

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ctx {
    Global,
    Overview,
    LedgerL1,
    LedgerL2,
    LedgerL3,
    IcL1,
    IcL2,
    IcL3,
    CompareL1,
    CompareL2,
    Combined,
    Activity,
    System,
    Confirm,
    Picker,
    Palette,
    Help,
    PromptFilter,
    PromptAsOf,
    PromptRange,
    ClassifyMode,    // shell: v0.11.0 tab 6, every mode (`,` `.`)
    ClassifyQueue,   // shell: the Queue list
    ClassifyProject, // shell: `p` pressed, a digit picks the project (sealed)
    ClassifyPicker,  // shell: the b / t / s picker (sealed; types)
    ClassifyNote,    // shell: the note editor (sealed; types)
    ClassifyBulk,    // shell: the bulk-accept preview and its confirm (sealed)
    ClassifyCatalog, // shell: the Catalog lists (Projects, Rules)
    ClassifyEditor,  // shell: the project / rule editor (sealed; types)
    ClassifyDiff,    // shell: the classify.toml diff and its confirm (sealed)
    ClassifyGroups,  // shell: the Groups mode (list + detail)
    ManifestMode,    // shell: tab 7 Manifest, every mode (`,` `.`, `b`, `N`, `c`, `P`, `y`)
    ManifestEntries, // shell: the entry list (Enter diff, u, e, n)
    ManifestIssues,  // shell: the Issues list (Enter jumps to the field, a acknowledges)
    ManifestDiff,    // shell: an entry's document diff (z, J, a)
    ManifestPrompt,  // shell: a tab 7 prompt (new manifest, note, edit; sealed; types)
    Viz,             // shell: an explainer panel is open (Esc closes, the row keys scroll)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum When {
    Always,
    HasRow,
    HasDocument,
    ScopeIsOrg,
    ScopeIsGroup,
    UsesAsOf,
    LinksSection,
    SettingsSection,
    NotOffline,
    ConfirmAsk, // shell: the band offers a choice (Cancel + action); a refusal has only Close
    MultiSelect, // shell: the picker is in multi-select mode (Compare)
    HasRecord,  // shell: the Ledger's org has a local record (keys that need its books)
    CanCompare, // shell: `v` can open Compare (an org with a record, or all / a group of 2–4)
    NoteMulti,  // shell: the Classify note editor is multi-line (`⌃S` saves)
    Staged,     // shell: the Catalog has staged classify.toml changes
    AgentWrites, // shell: Classify is open, or an agent's classify write is announced (`U` undoes it)
    RulesList,   // shell: the Catalog shows its Rules list
    OnManifest,  // shell: this session shows a manifest (not main)
    OnWarning, // shell: the selected issue (or the diff's focused field) is an unacknowledged warning
    HasCommand, // shell: the manifest is planned or approved: the command panel shows a command to copy
}

pub struct Binding {
    pub keys: &'static [&'static str],
    pub action: &'static str,
    pub ctx: Ctx,
    pub when: When,
    pub label: &'static str,
    pub group: &'static str,
    pub footer: Option<u8>,
}

pub struct Action {
    pub id: &'static str,
    pub label: &'static str,
    pub palette: bool,
    pub network: bool,
    pub cli: Option<&'static str>,
}

use Ctx::*;
use When::*;

#[rustfmt::skip]
pub const BINDINGS: &[Binding] = &[
    // ── global (open context; views fall through to it) ─────────────────────────────────────────────
    Binding { keys: &["1"], action: "tab.overview",     ctx: Global, when: Always, label: "Overview",     group: "Tabs",  footer: None },
    Binding { keys: &["2"], action: "tab.ledger",       ctx: Global, when: Always, label: "Ledger",       group: "Tabs",  footer: None },
    Binding { keys: &["3"], action: "tab.intercompany", ctx: Global, when: Always, label: "Intercompany", group: "Tabs",  footer: None },
    Binding { keys: &["4"], action: "tab.activity",     ctx: Global, when: Always, label: "Activity",     group: "Tabs",  footer: None },
    Binding { keys: &["5"], action: "tab.system",       ctx: Global, when: Always, label: "System",       group: "Tabs",  footer: None },
    Binding { keys: &["6"], action: "tab.classify",     ctx: Global, when: Always, label: "Classify",     group: "Tabs",  footer: None }, // shell: v0.11.0
    Binding { keys: &["7"], action: "tab.manifest",       ctx: Global, when: Always, label: "Manifest",     group: "Tabs",  footer: None }, // shell: v0.13.0
    Binding { keys: &["tab"],       action: "tab.next", ctx: Global, when: Always, label: "next tab",     group: "Tabs",  footer: None },
    Binding { keys: &["shift-tab"], action: "tab.prev", ctx: Global, when: Always, label: "prev tab",     group: "Tabs",  footer: None },
    Binding { keys: &["q"],      action: "app.quit",       ctx: Global, when: Always,   label: "Quit",    group: "App",   footer: Some(1) },
    Binding { keys: &["/"],      action: "filter.open",    ctx: Global, when: Always,   label: "Filter",  group: "Find",  footer: Some(2) },
    Binding { keys: &["]"],      action: "scope.next",     ctx: Global, when: Always,   label: "Org ▸",   group: "Scope", footer: Some(3) },
    Binding { keys: &["["],      action: "scope.prev",     ctx: Global, when: Always,   label: "Org ◂",   group: "Scope", footer: None },
    Binding { keys: &["g"],      action: "scope.picker",   ctx: Global, when: Always,   label: "Scope",   group: "Scope", footer: Some(4) },
    Binding { keys: &["d"],      action: "asof.edit",      ctx: Global, when: UsesAsOf, label: "As of",   group: "Scope", footer: Some(5) },
    Binding { keys: &["alt-left"],  action: "history.back",    ctx: Global, when: Always, label: "Back",    group: "Move",  footer: None }, // shell: v0.10.2 navigation history
    Binding { keys: &["alt-right"], action: "history.forward", ctx: Global, when: Always, label: "Forward", group: "Move",  footer: None },
    Binding { keys: &["U"],      action: "classify.undo_agent", ctx: Global, when: AgentWrites, label: "Undo agent", group: "Classify", footer: None }, // shell: v0.11.0 (spec classify §5a)
    Binding { keys: &["i"],      action: "view.details",   ctx: Global, when: Always,   label: "Details", group: "View",  footer: Some(6) }, // shell: v0.12.0 the focused view's details layer
    Binding { keys: &["W"],      action: "walk.open",      ctx: Global, when: Always,   label: "Walkthroughs", group: "View", footer: None }, // shell: v0.14.0 walkthroughs
    Binding { keys: &["L"],      action: "view.lay",       ctx: Global, when: Always,   label: "Lay mode", group: "View", footer: None }, // shell: v0.14.0 lay / informed
    Binding { keys: &["r"],      action: "record.reload",  ctx: Global, when: Always,   label: "Reload",  group: "App",   footer: Some(7) },
    Binding { keys: &["?"],      action: "help.open",      ctx: Global, when: Always,   label: "Help",    group: "App",   footer: Some(8) },
    Binding { keys: &["ctrl-p", ":"], action: "palette.open", ctx: Global, when: Always, label: "palette", group: "App", footer: Some(9) }, // pinned right edge
    // movement and drill (every table view)
    Binding { keys: &["up", "k"],     action: "row.prev",  ctx: Global, when: HasRow, label: "up",     group: "Move", footer: None },
    Binding { keys: &["down", "j"],   action: "row.next",  ctx: Global, when: HasRow, label: "down",   group: "Move", footer: None },
    Binding { keys: &["pageup"],      action: "page.prev", ctx: Global, when: HasRow, label: "page up",   group: "Move", footer: None },
    Binding { keys: &["pagedown"],    action: "page.next", ctx: Global, when: HasRow, label: "page down", group: "Move", footer: None },
    Binding { keys: &["home"],        action: "row.first", ctx: Global, when: HasRow, label: "first",  group: "Move", footer: None },
    Binding { keys: &["end"],         action: "row.last",  ctx: Global, when: HasRow, label: "last",   group: "Move", footer: None },
    Binding { keys: &["enter"],       action: "drill.in",  ctx: Global, when: HasRow, label: "open",   group: "Move", footer: None },
    Binding { keys: &["esc"],         action: "drill.back",ctx: Global, when: Always, label: "back",   group: "Move", footer: None },

    // ── Overview ───────────────────────────────────────────────────────────────────────────────────
    // shell: ←/→ move between the org cell and the two parity cells (M1: Enter on a parity cell drills with its basis)
    Binding { keys: &["left"],  action: "cell.prev", ctx: Overview, when: HasRow, label: "cell ◂", group: "Move", footer: None },
    Binding { keys: &["right"], action: "cell.next", ctx: Overview, when: HasRow, label: "cell ▸", group: "Move", footer: None },

    // ── Ledger ─────────────────────────────────────────────────────────────────────────────────────
    Binding { keys: &["h"], action: "basis.toggle",   ctx: LedgerL1, when: HasRecord,    label: "Full history", group: "View", footer: Some(6) }, // label flips to "Year basis"
    Binding { keys: &["!"], action: "filter.differs", ctx: LedgerL1, when: HasRecord,    label: "Differs",      group: "View", footer: Some(7) },
    Binding { keys: &["v"], action: "compare.open",   ctx: LedgerL1, when: CanCompare,   label: "Compare",      group: "View", footer: Some(6) },
    // `c` combined.open returns with Phase 5 (Combined); nothing unbuilt is reachable (PM, 2026-10-05).
    Binding { keys: &["h"], action: "basis.toggle",   ctx: LedgerL2, when: Always,       label: "Full history", group: "View", footer: Some(6) },
    Binding { keys: &["f"], action: "range.edit",     ctx: LedgerL2, when: Always,       label: "Range",        group: "View", footer: Some(5) },
    Binding { keys: &["v"], action: "compare.open",   ctx: LedgerL2, when: CanCompare,   label: "Compare",      group: "View", footer: Some(7) },
    Binding { keys: &["T"], action: "viz.t_account",  ctx: LedgerL1, when: HasRow,       label: "T-account",    group: "Explain", footer: None }, // shell: walkthroughs §4
    Binding { keys: &["M"], action: "viz.timeline",   ctx: LedgerL1, when: HasRow,       label: "Timeline",     group: "Explain", footer: None },
    Binding { keys: &["T"], action: "viz.t_account",  ctx: LedgerL2, when: Always,       label: "T-account",    group: "Explain", footer: None },
    Binding { keys: &["M"], action: "viz.timeline",   ctx: LedgerL2, when: Always,       label: "Timeline",     group: "Explain", footer: None },
    Binding { keys: &["o"], action: "doc.open_xero",  ctx: LedgerL3, when: HasDocument,  label: "Open",         group: "Doc",  footer: Some(5) },
    Binding { keys: &["y"], action: "doc.copy_id",    ctx: LedgerL3, when: HasDocument,  label: "Copy",         group: "Doc",  footer: Some(6) },

    // ── an explainer panel (walkthroughs §4) ─────────────────────────────────────────────────────────
    Binding { keys: &["esc"],  action: "drill.back", ctx: Viz, when: Always, label: "Close",  group: "Explain", footer: Some(1) },
    Binding { keys: &["down"], action: "row.next",   ctx: Viz, when: Always, label: "Scroll", group: "Explain", footer: Some(2) },

    // ── Intercompany ───────────────────────────────────────────────────────────────────────────────
    Binding { keys: &["s"], action: "ic.sort",        ctx: IcL1, when: HasRow,      label: "Sort",   group: "View", footer: Some(6) },
    Binding { keys: &["a"], action: "ic.side_a",      ctx: IcL2, when: Always,      label: "A",      group: "View", footer: Some(6) },
    Binding { keys: &["b"], action: "ic.side_b",      ctx: IcL2, when: Always,      label: "B",      group: "View", footer: Some(7) },
    Binding { keys: &["o"], action: "doc.open_xero",  ctx: IcL2, when: HasRow,      label: "Open",   group: "Doc",  footer: Some(5) },
    Binding { keys: &["y"], action: "doc.copy_id",    ctx: IcL2, when: HasRow,      label: "Copy",   group: "Doc",  footer: None },
    Binding { keys: &["o"], action: "doc.open_xero",  ctx: IcL3, when: HasDocument, label: "Open",   group: "Doc",  footer: Some(5) },
    Binding { keys: &["y"], action: "doc.copy_id",    ctx: IcL3, when: HasDocument, label: "Copy",   group: "Doc",  footer: Some(6) },

    // ── Compare ────────────────────────────────────────────────────────────────────────────────────
    Binding { keys: &["a"], action: "compare.align",    ctx: CompareL1, when: Always, label: "Align",    group: "View", footer: Some(5) },
    Binding { keys: &[">"], action: "compare.col_next", ctx: CompareL1, when: Always, label: "Col ▸",    group: "View", footer: Some(7) },
    Binding { keys: &["<"], action: "compare.col_prev", ctx: CompareL1, when: Always, label: "Col ◂",    group: "View", footer: None },
    Binding { keys: &["x"], action: "compare.col_drop", ctx: CompareL1, when: Always, label: "Drop col", group: "View", footer: Some(8) },
    // shell: ←/→ choose the account column or an org column (Enter: account → postings, amount → that org's ledger)
    Binding { keys: &["left"],  action: "cell.prev", ctx: CompareL1, when: HasRow, label: "column ◂", group: "Move", footer: None },
    Binding { keys: &["right"], action: "cell.next", ctx: CompareL1, when: HasRow, label: "column ▸", group: "Move", footer: None },
    Binding { keys: &["h"], action: "basis.toggle",     ctx: CompareL1, when: Always, label: "Full history", group: "View", footer: Some(6) },
    Binding { keys: &["left"],  action: "cell.prev", ctx: CompareL2, when: HasRow, label: "column ◂", group: "Move", footer: None },
    Binding { keys: &["right"], action: "cell.next", ctx: CompareL2, when: HasRow, label: "column ▸", group: "Move", footer: None },
    Binding { keys: &["h"], action: "basis.toggle",     ctx: CompareL2, when: Always, label: "Full history", group: "View", footer: Some(7) },
    Binding { keys: &["m"], action: "compare.ic_marks", ctx: CompareL2, when: Always, label: "IC matches", group: "View", footer: Some(6) },
    Binding { keys: &["f"], action: "range.edit",       ctx: CompareL2, when: Always, label: "Range",    group: "View", footer: Some(5) },

    // ── Combined ───────────────────────────────────────────────────────────────────────────────────
    // Phase 5: Binding { keys: &["t"], action: "combined.statement",  ctx: Combined, when: Always, label: "TB/BS/P&L",       group: "View", footer: Some(5) },
    // Phase 5: Binding { keys: &["n"], action: "combined.not_elim",   ctx: Combined, when: Always, label: "Not eliminated",  group: "View", footer: Some(7) },
    // Phase 5: Binding { keys: &["<"], action: "combined.page_prev",  ctx: Combined, when: Always, label: "Orgs ◂",          group: "View", footer: None },
    // Phase 5: Binding { keys: &[">"], action: "combined.page_next",  ctx: Combined, when: Always, label: "Orgs ▸",          group: "View", footer: None },

    // ── Activity / System ──────────────────────────────────────────────────────────────────────────
    Binding { keys: &["G"], action: "activity.group_endpoint", ctx: Activity, when: Always,          label: "By endpoint", group: "View", footer: Some(6) },
    Binding { keys: &["e"], action: "system.edit_config",      ctx: System,   when: SettingsSection, label: "Edit config", group: "App",  footer: Some(5) },
    Binding { keys: &["o"], action: "link.open",               ctx: System,   when: LinksSection,    label: "Open",        group: "Doc",  footer: Some(5) },
    Binding { keys: &["y"], action: "link.copy",               ctx: System,   when: LinksSection,    label: "Copy",        group: "Doc",  footer: Some(6) },

    // ── Classify (v0.11.0; spec classify §3.A, §7) ──────────────────────────────────────────────────────
    // shell: Enter is the row's primary action everywhere (`drill.in`); in the Queue it accepts, then moves on.
    Binding { keys: &["enter"], action: "drill.in",            ctx: ClassifyQueue, when: HasRow, label: "Accept",     group: "Classify", footer: Some(1) },
    Binding { keys: &["a"],     action: "classify.accept",     ctx: ClassifyQueue, when: HasRow, label: "Accept",     group: "Classify", footer: None },
    Binding { keys: &["p"],     action: "classify.project",    ctx: ClassifyQueue, when: HasRow, label: "Project",    group: "Classify", footer: Some(2) },
    Binding { keys: &["b"],     action: "classify.bearer",     ctx: ClassifyQueue, when: HasRow, label: "Bearer",     group: "Classify", footer: Some(3) },
    Binding { keys: &["t"],     action: "classify.treatment",  ctx: ClassifyQueue, when: HasRow, label: "Treatment",  group: "Classify", footer: Some(4) },
    Binding { keys: &["s"],     action: "classify.account",    ctx: ClassifyQueue, when: HasRow, label: "Should-be",  group: "Classify", footer: None },
    Binding { keys: &["G"],     action: "classify.group",      ctx: ClassifyQueue, when: HasRow, label: "Group",      group: "Classify", footer: Some(5) }, // design: G (g stays the scope picker)
    Binding { keys: &["n"],     action: "classify.note",       ctx: ClassifyQueue, when: HasRow, label: "Note",       group: "Classify", footer: Some(5) },
    Binding { keys: &["x"],     action: "classify.skip",       ctx: ClassifyQueue, when: HasRow, label: "Skip",       group: "Classify", footer: Some(6) },
    Binding { keys: &["u"],     action: "classify.undo",       ctx: ClassifyQueue, when: HasRow, label: "Undo",       group: "Classify", footer: Some(6) },
    Binding { keys: &["A"],     action: "classify.bulk",       ctx: ClassifyQueue, when: Always, label: "Bulk accept", group: "Classify", footer: Some(7) },
    Binding { keys: &["R"],     action: "classify.make_rule",  ctx: ClassifyQueue, when: HasRow, label: "Make rule (coming)", group: "Coming", footer: None },
    Binding { keys: &["."],     action: "classify.mode_next",  ctx: ClassifyMode,  when: Always, label: "Mode ▸",     group: "View",     footer: Some(8) },
    Binding { keys: &[","],     action: "classify.mode_prev",  ctx: ClassifyMode,  when: Always, label: "Mode ◂",     group: "View",     footer: None },
    // sealed: `p` then 1–9; the pickers and the note editor type; the bulk preview confirms with Cancel focused
    Binding { keys: &["1", "2", "3", "4", "5", "6", "7", "8", "9"], action: "classify.project_digit", ctx: ClassifyProject, when: Always, label: "project", group: "Classify", footer: Some(1) },
    Binding { keys: &["esc"],   action: "classify.cancel",     ctx: ClassifyProject, when: Always, label: "cancel",   group: "Classify", footer: Some(2) },
    Binding { keys: &["enter"], action: "classify.pick",       ctx: ClassifyPicker,  when: Always, label: "set",      group: "Picker",   footer: Some(1) },
    Binding { keys: &["esc"],   action: "classify.cancel",     ctx: ClassifyPicker,  when: Always, label: "cancel",   group: "Picker",   footer: Some(2) },
    Binding { keys: &["up", "ctrl-k"],   action: "classify.pick_prev", ctx: ClassifyPicker, when: Always, label: "move", group: "Picker", footer: Some(3) },
    Binding { keys: &["down", "ctrl-j"], action: "classify.pick_next", ctx: ClassifyPicker, when: Always, label: "move", group: "Picker", footer: Some(3) },
    Binding { keys: &["enter"],  action: "classify.note_enter", ctx: ClassifyNote, when: Always,    label: "save",       group: "Note", footer: Some(1) }, // multi-line: a new line
    Binding { keys: &["ctrl-s"], action: "classify.note_save",  ctx: ClassifyNote, when: NoteMulti, label: "save",       group: "Note", footer: Some(2) },
    Binding { keys: &["ctrl-e"], action: "classify.note_multi", ctx: ClassifyNote, when: Always,    label: "multi-line", group: "Note", footer: Some(3) },
    Binding { keys: &["esc"],    action: "classify.cancel",     ctx: ClassifyNote, when: Always,    label: "cancel",     group: "Note", footer: Some(4) },
    Binding { keys: &["enter"],  action: "classify.bulk_go",    ctx: ClassifyBulk, when: Always, label: "choose",      group: "Confirm", footer: Some(1) }, // runs the FOCUSED button; Cancel by default
    Binding { keys: &["tab", "right", "shift-tab", "left"], action: "classify.bulk_focus", ctx: ClassifyBulk, when: Always, label: "next button", group: "Confirm", footer: Some(2) },
    Binding { keys: &["+", "="], action: "classify.bulk_more",  ctx: ClassifyBulk, when: Always, label: "threshold",   group: "Confirm", footer: Some(3) },
    Binding { keys: &["-"],      action: "classify.bulk_less",  ctx: ClassifyBulk, when: Always, label: "threshold",   group: "Confirm", footer: Some(3) },
    Binding { keys: &["esc"],    action: "classify.cancel",     ctx: ClassifyBulk, when: Always, label: "cancel",      group: "Confirm", footer: Some(4) },
    // Groups (spec classify §3.E): ←→ list/members · Enter open · R role (r is the global reload) · m add member
    // · n note · S status · D remove member
    Binding { keys: &["right"], action: "groups.focus_next", ctx: ClassifyGroups, when: Always, label: "members ▸", group: "Groups", footer: None },
    Binding { keys: &["left"],  action: "groups.focus_prev", ctx: ClassifyGroups, when: Always, label: "list ◂",    group: "Groups", footer: None },
    Binding { keys: &["enter"], action: "drill.in",          ctx: ClassifyGroups, when: HasRow, label: "Open",      group: "Groups", footer: Some(1) },
    Binding { keys: &["R"],     action: "groups.role",       ctx: ClassifyGroups, when: HasRow, label: "Role",      group: "Groups", footer: Some(2) },
    Binding { keys: &["m"],     action: "groups.add_member", ctx: ClassifyGroups, when: HasRow, label: "Add member", group: "Groups", footer: Some(2) },
    Binding { keys: &["n"],     action: "groups.note",       ctx: ClassifyGroups, when: HasRow, label: "Note",      group: "Groups", footer: Some(3) },
    Binding { keys: &["S"],     action: "groups.status",     ctx: ClassifyGroups, when: HasRow, label: "Status",    group: "Groups", footer: Some(3) },
    Binding { keys: &["D"],     action: "groups.remove",     ctx: ClassifyGroups, when: HasRow, label: "Remove",    group: "Groups", footer: Some(4) },
    // Catalog (spec classify §3.D, §7): ←→ list · N new · Enter edit · D archive · ⌥↑↓ priority · w write · u discard
    Binding { keys: &["right"], action: "catalog.list_next", ctx: ClassifyCatalog, when: Always, label: "list ▸",   group: "Catalog", footer: None },
    Binding { keys: &["left"],  action: "catalog.list_prev", ctx: ClassifyCatalog, when: Always, label: "list ◂",   group: "Catalog", footer: None },
    Binding { keys: &["enter"], action: "drill.in",          ctx: ClassifyCatalog, when: HasRow, label: "Edit",     group: "Catalog", footer: Some(1) },
    Binding { keys: &["N"],     action: "catalog.new",       ctx: ClassifyCatalog, when: Always, label: "New",      group: "Catalog", footer: Some(1) },
    Binding { keys: &["D"],     action: "catalog.archive",   ctx: ClassifyCatalog, when: HasRow, label: "Archive",  group: "Catalog", footer: Some(2) },
    Binding { keys: &["w"],     action: "catalog.apply",     ctx: ClassifyCatalog, when: Staged, label: "Write",    group: "Catalog", footer: Some(1) },
    Binding { keys: &["u"],     action: "catalog.discard",   ctx: ClassifyCatalog, when: Staged, label: "Discard",  group: "Catalog", footer: Some(2) },
    Binding { keys: &["alt-up"],   action: "catalog.priority_up",   ctx: ClassifyCatalog, when: RulesList, label: "priority", group: "Catalog", footer: None },
    Binding { keys: &["alt-down"], action: "catalog.priority_down", ctx: ClassifyCatalog, when: RulesList, label: "priority", group: "Catalog", footer: None },
    Binding { keys: &["enter"],  action: "catalog.save",       ctx: ClassifyEditor, when: Always, label: "stage",   group: "Editor", footer: Some(1) },
    Binding { keys: &["tab", "down"],       action: "catalog.field_next", ctx: ClassifyEditor, when: Always, label: "field", group: "Editor", footer: Some(2) },
    Binding { keys: &["shift-tab", "up"],   action: "catalog.field_prev", ctx: ClassifyEditor, when: Always, label: "field", group: "Editor", footer: None },
    Binding { keys: &["esc"],    action: "catalog.cancel",     ctx: ClassifyEditor, when: Always, label: "cancel",  group: "Editor", footer: Some(3) },
    Binding { keys: &["enter"],  action: "catalog.diff_go",    ctx: ClassifyDiff, when: Always, label: "choose",      group: "Confirm", footer: Some(1) }, // the FOCUSED button; Cancel by default
    Binding { keys: &["tab", "right", "shift-tab", "left"], action: "catalog.diff_focus", ctx: ClassifyDiff, when: Always, label: "next button", group: "Confirm", footer: Some(2) },
    Binding { keys: &["down", "j"], action: "row.next",  ctx: ClassifyDiff, when: Always, label: "scroll", group: "Confirm", footer: Some(3) },
    Binding { keys: &["up", "k"],   action: "row.prev",  ctx: ClassifyDiff, when: Always, label: "scroll", group: "Confirm", footer: Some(3) },
    Binding { keys: &["pagedown"],  action: "page.next", ctx: ClassifyDiff, when: Always, label: "page",   group: "Confirm", footer: None },
    Binding { keys: &["pageup"],    action: "page.prev", ctx: ClassifyDiff, when: Always, label: "page",   group: "Confirm", footer: None },
    Binding { keys: &["esc"],    action: "catalog.cancel",     ctx: ClassifyDiff, when: Always, label: "cancel",      group: "Confirm", footer: Some(4) },

    // ── tab 7 Manifest (design spec manifests §2; branches §4) ────────────────────────────────────
    // There is NO apply key, palette entry or agent verb: apply is CLI-only (PM ruling, 7 Oct).
    Binding { keys: &[","], action: "manifest.mode_prev", ctx: ManifestMode, when: OnManifest, label: "mode ◂",  group: "Manifest", footer: None },
    Binding { keys: &["."], action: "manifest.mode_next", ctx: ManifestMode, when: OnManifest, label: "mode ▸",  group: "Manifest", footer: None },
    Binding { keys: &["b"], action: "manifest.picker",    ctx: ManifestMode, when: Always,     label: "Switch",  group: "Manifest", footer: Some(3) },
    Binding { keys: &["N"], action: "manifest.new",       ctx: ManifestMode, when: Always,     label: "New",     group: "Manifest", footer: Some(2) },
    Binding { keys: &["c"], action: "manifest.check",     ctx: ManifestMode, when: OnManifest, label: "Check",   group: "Manifest", footer: None },
    Binding { keys: &["P"], action: "manifest.plan",      ctx: ManifestMode, when: OnManifest, label: "Plan",    group: "Manifest", footer: Some(2) },
    Binding { keys: &["y"], action: "manifest.copy_command", ctx: ManifestMode, when: HasCommand, label: "copy command", group: "Manifest", footer: Some(1) },
    Binding { keys: &["enter"], action: "drill.in",       ctx: ManifestEntries, when: HasRow, label: "Diff",  group: "Manifest", footer: Some(1) },
    Binding { keys: &["u"], action: "manifest.undo",      ctx: ManifestEntries, when: HasRow, label: "Undo",  group: "Manifest", footer: Some(2) },
    Binding { keys: &["e"], action: "manifest.edit",      ctx: ManifestEntries, when: HasRow, label: "Edit",  group: "Manifest", footer: Some(3) },
    Binding { keys: &["n"], action: "manifest.annotate",  ctx: ManifestEntries, when: HasRow, label: "Note",  group: "Manifest", footer: Some(3) },
    Binding { keys: &["enter"], action: "drill.in",       ctx: ManifestIssues, when: HasRow, label: "Go to field", group: "Manifest", footer: Some(1) },
    Binding { keys: &["a"], action: "manifest.ack",       ctx: ManifestIssues, when: OnWarning, label: "Acknowledge", group: "Manifest", footer: Some(1) },
    Binding { keys: &["z"], action: "manifest.collapse",  ctx: ManifestDiff, when: Always, label: "All fields", group: "Diff", footer: Some(2) },
    Binding { keys: &["J"], action: "manifest.patch_view", ctx: ManifestDiff, when: Always, label: "Patch", group: "Diff", footer: Some(2) },
    Binding { keys: &["a"], action: "manifest.ack",       ctx: ManifestDiff, when: OnWarning, label: "Acknowledge", group: "Diff", footer: Some(1) },
    Binding { keys: &["enter"], action: "manifest.prompt_save", ctx: ManifestPrompt, when: Always, label: "save",   group: "Prompt", footer: Some(1) },
    Binding { keys: &["tab"],   action: "manifest.prompt_next", ctx: ManifestPrompt, when: Always, label: "field",  group: "Prompt", footer: Some(2) },
    Binding { keys: &["esc"],   action: "manifest.prompt_cancel", ctx: ManifestPrompt, when: Always, label: "cancel", group: "Prompt", footer: Some(3) },

    // ── Confirm band (SEALED; spec V3: no single-letter confirm) ──────────────────────────────────
    Binding { keys: &["tab", "right"],      action: "confirm.focus_next", ctx: Confirm, when: ConfirmAsk, label: "next button", group: "Confirm", footer: Some(3) },
    Binding { keys: &["shift-tab", "left"], action: "confirm.focus_prev", ctx: Confirm, when: ConfirmAsk, label: "prev button", group: "Confirm", footer: None },
    Binding { keys: &["enter"],             action: "confirm.activate",   ctx: Confirm, when: Always, label: "choose",      group: "Confirm", footer: Some(1) }, // runs the FOCUSED button; Cancel by default
    Binding { keys: &["esc"],               action: "confirm.cancel",     ctx: Confirm, when: Always, label: "cancel",      group: "Confirm", footer: Some(2) },

    // ── Overlays (SEALED; printable keys type into the filter) ─────────────────────────────────────
    Binding { keys: &["enter"], action: "picker.choose",   ctx: Picker,  when: Always, label: "switch",  group: "Picker",  footer: Some(1) },
    Binding { keys: &["space"], action: "picker.toggle",   ctx: Picker,  when: MultiSelect, label: "toggle",  group: "Picker",  footer: Some(2) }, // multi-select mode only (Compare)
    Binding { keys: &["esc"],   action: "overlay.close",   ctx: Picker,  when: Always, label: "cancel",  group: "Picker",  footer: Some(3) },
    Binding { keys: &["up", "ctrl-k"],   action: "picker.prev", ctx: Picker, when: Always, label: "move", group: "Picker", footer: Some(4) }, // shell: ↑↓ move (acceptance M1),
    Binding { keys: &["down", "ctrl-j"], action: "picker.next", ctx: Picker, when: Always, label: "move", group: "Picker", footer: Some(4) },
    Binding { keys: &["enter"], action: "palette.run",     ctx: Palette, when: Always, label: "run",     group: "Palette", footer: Some(1) },
    Binding { keys: &["esc"],   action: "overlay.close",   ctx: Palette, when: Always, label: "close",   group: "Palette", footer: Some(2) },
    Binding { keys: &["up", "ctrl-k"],   action: "palette.prev", ctx: Palette, when: Always, label: "move", group: "Palette", footer: Some(3) }, // shell: ↑↓ move,
    Binding { keys: &["down", "ctrl-j"], action: "palette.next", ctx: Palette, when: Always, label: "move", group: "Palette", footer: Some(3) },
    Binding { keys: &["esc", "?", "q"], action: "overlay.close", ctx: Help, when: Always, label: "close", group: "Help", footer: Some(1) },
    Binding { keys: &["enter"], action: "prompt.apply",  ctx: PromptAsOf,   when: Always, label: "apply",  group: "Prompt", footer: Some(1) },
    Binding { keys: &["esc"],   action: "prompt.cancel", ctx: PromptAsOf,   when: Always, label: "cancel", group: "Prompt", footer: Some(2) },
    Binding { keys: &["enter"], action: "prompt.apply",  ctx: PromptRange,  when: Always, label: "apply",  group: "Prompt", footer: Some(1) },
    Binding { keys: &["esc"],   action: "prompt.cancel", ctx: PromptRange,  when: Always, label: "reset",  group: "Prompt", footer: Some(2) },
    Binding { keys: &["esc"],   action: "filter.clear",  ctx: PromptFilter, when: Always, label: "clear",  group: "Prompt", footer: Some(1) },
    Binding { keys: &["enter"], action: "prompt.apply",  ctx: PromptFilter, when: Always, label: "keep",   group: "Prompt", footer: Some(2) }, // shell: close the prompt, keep the filter
];

/// Palette-only actions (network) and other actions without keys. `network: true` ⇒ confirm band first,
/// budget check first, hidden under --offline, and NEVER bound to a key.
#[rustfmt::skip]
pub const ACTIONS: &[Action] = &[
    Action { id: "net.sync_org",       label: "Sync <alias>…",                          palette: true, network: true,  cli: Some("ownpurse sync <alias>") },
    Action { id: "net.sync_all",       label: "Sync all orgs…",                         palette: true, network: true,  cli: Some("ownpurse sync all") },
    Action { id: "net.login",          label: "Log in to Xero…",                        palette: true, network: true,  cli: Some("ownpurse login") },
    Action { id: "net.refetch_tb",     label: "Refetch Xero trial balance for parity…", palette: true, network: true,  cli: Some("ownpurse parity --refetch") },
    Action { id: "net.fetch_history",  label: "Fetch history…",                         palette: true, network: true,  cli: None }, // when: has_document
    Action { id: "net.fetch_attach",   label: "Fetch attachments…",                     palette: true, network: true,  cli: None }, // when: has_document
    Action { id: "scope.switch_to",    label: "Switch to <alias>",                      palette: true, network: false, cli: None },
    Action { id: "scope.switch_group", label: "Switch to group:<name>",                 palette: true, network: false, cli: None },
    Action { id: "compare.open",       label: "Compare orgs side by side…",             palette: true, network: false, cli: Some("ownpurse tb <a>,<b> --compare") },
    Action { id: "theme.cycle",        label: "Change theme",                           palette: true, network: false, cli: None },
    Action { id: "view.details",       label: "Show or hide details (IDs, sources, budgets)", palette: true, network: false, cli: None }, // shell: v0.12.0
    Action { id: "walk.open",          label: "Walkthroughs…",                          palette: true, network: false, cli: Some("ownpurse walkthrough list") }, // shell: v0.14.0
    Action { id: "view.lay",           label: "Lay mode / informed mode",               palette: true, network: false, cli: None }, // shell: v0.14.0
    Action { id: "theme.overlay.clear", label: "Clear theme overlay",                  palette: true, network: false, cli: None }, // shell: v0.12.0 (theme spec v2 §2.7)
    Action { id: "app.upgrade",        label: "Upgrade to v<version>",                  palette: true, network: false, cli: Some("ownpurse ui upgrade") },
    Action { id: "classify.queue",     label: "Classify: Queue",                        palette: true, network: false, cli: Some("ownpurse classify queue") },
    Action { id: "classify.groups",    label: "Classify: Groups",                       palette: true, network: false, cli: None },
    Action { id: "classify.audition",  label: "Classify: Audition",                     palette: true, network: false, cli: None },
    Action { id: "classify.tree",      label: "Classify: Tree",                         palette: true, network: false, cli: None },
    Action { id: "classify.catalog",   label: "Classify: Catalog",                      palette: true, network: false, cli: None },
    Action { id: "classify.jev",       label: "Classify: Ask Jev…",                     palette: true, network: true,  cli: Some("ownpurse classify suggest --engine jev") }, // V16: refused while Jev is not enabled
    Action { id: "manifest.switch",    label: "Switch manifest…",                       palette: true, network: false, cli: Some("ownpurse manifest switch <name>") },
    Action { id: "manifest.make_default", label: "Make default manifest (the CLI follows it)", palette: true, network: false, cli: Some("ownpurse manifest switch <name>") },
    Action { id: "manifest.to_main",   label: "Show main (leave the manifest)",         palette: true, network: false, cli: Some("ownpurse manifest switch main") },
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn network_has_no_keys() {
        for a in ACTIONS.iter().filter(|a| a.network) {
            assert!(
                !BINDINGS.iter().any(|b| b.action == a.id),
                "network action {} has a key",
                a.id
            );
        }
    }

    #[test]
    fn confirm_binds_only_focus_enter_esc() {
        let allowed = ["tab", "shift-tab", "left", "right", "enter", "esc"];
        for b in BINDINGS.iter().filter(|b| b.ctx == Ctx::Confirm) {
            for k in b.keys {
                assert!(allowed.contains(k), "confirm binds {k}");
            }
        }
    }

    /// A key may mean only one action within a context, and a view binding may not reuse a global key
    /// for a different action (no key with two meanings across the stack). Tab and shift-tab are the
    /// one sanctioned exception: they cycle tabs globally and move focus in the sealed confirm band.
    #[test]
    fn no_key_has_two_meanings() {
        use std::collections::HashMap;
        let global: HashMap<&str, &str> = BINDINGS
            .iter()
            .filter(|b| b.ctx == Ctx::Global)
            .flat_map(|b| b.keys.iter().map(move |k| (*k, b.action)))
            .collect();
        let sealed = [
            Ctx::Confirm,
            Ctx::Picker,
            Ctx::Palette,
            Ctx::Help,
            Ctx::PromptFilter,
            Ctx::PromptAsOf,
            Ctx::PromptRange,
            Ctx::ClassifyProject,
            Ctx::ClassifyPicker,
            Ctx::ClassifyNote,
            Ctx::ClassifyBulk,
            Ctx::ClassifyEditor,
            Ctx::ClassifyDiff,
            Ctx::ManifestPrompt,
        ];
        for b in BINDINGS
            .iter()
            .filter(|b| b.ctx != Ctx::Global && !sealed.contains(&b.ctx))
        {
            for k in b.keys {
                if let Some(g) = global.get(k) {
                    assert_eq!(
                        *g, b.action,
                        "{k} means {g} globally and {} in {:?}",
                        b.action, b.ctx
                    );
                }
            }
        }
        let mut seen: HashMap<(String, &str), &str> = HashMap::new();
        for b in BINDINGS {
            for k in b.keys {
                let key = (format!("{:?}", b.ctx), *k);
                if let Some(prev) = seen.insert(key.clone(), b.action) {
                    assert_eq!(prev, b.action, "{k} bound twice in {:?}", b.ctx);
                }
            }
        }
    }

    #[test]
    fn copy_is_y_everywhere() {
        for b in BINDINGS
            .iter()
            .filter(|b| b.action.ends_with("copy") || b.action.ends_with("copy_id"))
        {
            assert_eq!(b.keys, &["y"], "{}", b.action);
        }
    }
}
