//! Tab 5: System (spec §9 row 5, merge M3): three sections in one scrolling list.
//! **Connection**: the token is managed by the CLI (`ownpurse login`; the TUI never reads it), scopes from the
//! config, and one row per org with plan, record count and last sync. **Settings**: every value with its
//! source (`defaults`, `config`, `profile:<name>`); an empty `tui.as_of` shows its default; `e` opens the
//! config in $EDITOR (an effect the loop runs). **Links**: per app, org and bank account; Enter / `o`
//! opens, `y` copies (OSC 52). `/` filters across all three. The cursor is kept as a row key.

use crate::app::{App, Effect, Tab};
use crate::clock;
use crate::hit::{Hits, Target};
use crate::keymap::When;
use crate::layout::{ColSpec, Kind, fit_str, pad, width};
use crate::model::Level;
use crate::theme_tokens::Token;
use crate::widgets::table::{self, Cell, Step};
use crate::widgets::{put, strip};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::text::Span;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SystemUi {
    /// The selected row's key (`org:<alias>`, `set:<key>`, `link:<n>`).
    pub sel: Option<String>,
    /// First visible line of the list.
    pub top: usize,
    /// The write package (hash) whose operations are listed under it (Writes, Enter toggles).
    pub open_pkg: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Section {
    Connection,
    Settings,
    Links,
    /// Write packages (Phase 8): listed only when the record has any.
    Writes,
}

impl Section {
    pub const ALL: [Section; 4] = [
        Section::Connection,
        Section::Settings,
        Section::Links,
        Section::Writes,
    ];
    pub fn title(self) -> &'static str {
        match self {
            Section::Connection => "Connection",
            Section::Settings => "Settings",
            Section::Links => "Links",
            Section::Writes => "Writes",
        }
    }
    pub fn cols(self) -> &'static [ColSpec] {
        match self {
            Section::Connection => &ORG_COLS,
            Section::Settings => &SET_COLS,
            Section::Links => &LINK_COLS,
            Section::Writes => &WRITE_COLS,
        }
    }
    fn steps(self) -> &'static [Step] {
        match self {
            Section::Connection => &[Step::Hide(5), Step::Hide(2)],
            _ => &[],
        }
    }
}

const fn col(title: &'static str, kind: Kind, priority: u8) -> ColSpec {
    ColSpec {
        title,
        short_title: "",
        kind,
        priority,
    }
}

pub const ORG_COLS: [ColSpec; 6] = [
    col("org", Kind::Text { min: 4 }, 9),
    col("name", Kind::Text { min: 8 }, 4),
    col("plan", Kind::Text { min: 4 }, 3),
    col("records", Kind::Amount, 9),
    col("last sync", Kind::Status, 9),
    col("tenant id", Kind::Id, 1),
];

pub const SET_COLS: [ColSpec; 3] = [
    col("key", Kind::Text { min: 14 }, 8),
    col("value", Kind::Text { min: 10 }, 6),
    col("source", Kind::Status, 9),
];

pub const WRITE_COLS: [ColSpec; 6] = [
    col("package", Kind::Text { min: 12 }, 9),
    col("hash", Kind::Id, 3),
    col("stage", Kind::Status, 9),
    col("ops", Kind::Text { min: 5 }, 8),
    // the verify outcome as the record shows it (verified / rejected / uncertain / incomplete; `—` before a run)
    col("verify", Kind::Status, 7),
    col("approval", Kind::Text { min: 8 }, 4),
];

pub const LINK_COLS: [ColSpec; 2] = [
    col("link", Kind::Text { min: 18 }, 9),
    col("url", Kind::Text { min: 16 }, 5),
];

/// A selectable row: its key, section, cells and the detail band's text.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Item {
    pub key: String,
    pub section: Section,
    pub cells: Vec<Cell>,
    pub detail: String,
    /// The link's url (Links only).
    pub url: Option<String>,
}

/// One line of the list.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Line {
    Heading(Section),
    Note(String, Token),
    Header(Section),
    Item(usize),
    Blank,
}

pub fn owns(app: &App) -> bool {
    app.ui.tab == Tab::System && app.ui.overlay.is_none()
}

/// `(default: last FY end → 2025-12-31)` for an empty `tui.as_of` (acceptance L3).
pub fn as_of_default(app: &App) -> String {
    let first = (!app.model.orgs.is_empty()).then_some(0);
    format!(
        "(default: last FY end → {})",
        app.model.default_as_of(first, app.now)
    )
}

fn notes(app: &App, section: Section) -> Vec<(String, Token)> {
    let sys = &app.source.sys;
    match section {
        Section::Connection => vec![
            (
                format!(
                    "token: managed by the CLI (`ownpurse login`){}",
                    sys.token_file.as_ref().map_or(String::new(), |t| format!(
                        " · token file {}",
                        app.paths.show(t)
                    ))
                ),
                Token::Muted,
            ),
            (
                // the CLI the app runs for sync and login (spec P6 §2)
                match &app.cli {
                    Some(p) => format!(
                        "cli: {}{}",
                        app.cli_shown.clone().unwrap_or_else(|| app.paths.show(p)),
                        app.cli_version
                            .as_deref()
                            .filter(|v| !v.is_empty())
                            .map_or(String::new(), |v| format!(" ({v})"))
                    ),
                    None => format!("cli: not found · {}", crate::net::NO_CLI),
                },
                if app.cli.is_some() {
                    Token::Muted
                } else {
                    Token::Warn
                },
            ),
            (scopes_note(sys, true), Token::Muted),
        ],
        Section::Settings => vec![(
            format!(
                "Config: {} · profile {} · store {} · e edits; restart to apply",
                sys.path
                    .as_ref()
                    .map_or("(none — environment defaults)".to_string(), |p| {
                        app.paths.show(p)
                    }),
                app.source.profile,
                app.paths.show(&app.source.record)
            ),
            Token::Muted,
        )],
        Section::Links | Section::Writes => Vec::new(),
    }
}

/// Below this width the scopes note takes its short form.
pub const SCOPES_WIDE: u16 = 100;

/// `scopes: 15 read · 3 write configured (never requested by this TUI)`; narrow: `… · 3 write (unused)`.
pub fn scopes_note(sys: &crate::system::SysConfig, wide: bool) -> String {
    if wide {
        format!(
            "scopes: {} read · {} write configured (never requested by this TUI)",
            sys.read_scopes, sys.write_scopes
        )
    } else {
        format!(
            "scopes: {} read · {} write (unused)",
            sys.read_scopes, sys.write_scopes
        )
    }
}

/// Every selectable row, in list order (before the filter).
pub fn items(app: &App) -> Vec<Item> {
    let mut out = Vec::new();
    for o in &app.model.system.orgs {
        let (sync, tok) = match &o.last_sync {
            None => ("never synced".to_string(), Token::Error),
            Some(at) => {
                let i = app.model.orgs.iter().position(|m| m.alias == o.alias);
                let f = i.map(|i| app.model.freshness(&[i], app.now));
                let age = f.as_ref().map_or(0, |f| f.age);
                let stale = f.as_ref().is_some_and(|f| f.level == Level::Warn);
                (
                    format!(
                        "{} UTC · {} ago{}",
                        clock::hhmm(at),
                        clock::ago(age),
                        if stale { " stale" } else { "" }
                    ),
                    if stale { Token::Warn } else { Token::Text },
                )
            }
        };
        out.push(Item {
            key: format!("org:{}", o.alias),
            section: Section::Connection,
            detail: format!(
                "{} · {} · tenant {} · short code {}",
                o.alias,
                o.name,
                o.tenant,
                if o.short_code.is_empty() {
                    "—"
                } else {
                    &o.short_code
                }
            ),
            cells: vec![
                Cell::new(o.alias.clone(), Token::Text),
                Cell::new(o.name.clone(), Token::Muted),
                Cell::new(if o.plan.is_empty() { "—" } else { &o.plan }, Token::Muted),
                Cell::new(clock::int(o.records), Token::Text),
                Cell::new(sync, tok),
                Cell::new(o.tenant.clone(), Token::Muted),
            ],
            url: None,
        });
    }
    for (k, v, o) in &app.source.sys.settings {
        let shown = if k == "tui.as_of" && v.is_empty() {
            as_of_default(app)
        } else {
            v.clone()
        };
        let tok = if o == "defaults" {
            Token::Muted
        } else {
            Token::Info
        };
        out.push(Item {
            key: format!("set:{k}"),
            section: Section::Settings,
            detail: format!("{k} = {shown} (source: {o})"),
            cells: vec![
                Cell::new(k.clone(), Token::Text),
                Cell::new(shown, Token::Text),
                Cell::new(o.clone(), tok),
            ],
            url: None,
        });
    }
    for (n, l) in app.model.system.links.iter().enumerate() {
        out.push(Item {
            key: format!("link:{n}"),
            section: Section::Links,
            detail: if l.description.is_empty() {
                l.url.clone()
            } else {
                format!("{} · {}", l.description, l.url)
            },
            cells: vec![
                Cell::new(l.name.clone(), Token::Text),
                Cell::new(l.url.clone(), Token::Info),
            ],
            url: Some(l.url.clone()),
        });
    }
    for p in &app.model.system.packages {
        use ownpurse_core::write::view::Stage;
        let tok = match p.stage {
            Stage::Stopped => Token::Warn,
            Stage::Verified | Stage::Landed => Token::Text,
            Stage::Recovered => Token::Muted,
            _ => Token::Info,
        };
        let ops: Vec<String> = p
            .op_list
            .iter()
            .map(|(id, kind, org, _)| format!("{id} {kind} ({org})"))
            .collect();
        out.push(Item {
            key: format!("pkg:{}", p.hash),
            section: Section::Writes,
            detail: format!(
                "{} · sha256 {} · {} · XA:{}{}",
                p.title,
                p.hash12(),
                ops.join(", "),
                p.id,
                p.source_ref
                    .as_deref()
                    .map_or(String::new(), |s| format!(" · from {s}"))
            ),
            cells: vec![
                Cell::new(p.title.clone(), Token::Text),
                Cell::new(p.hash12().to_string(), Token::Muted),
                Cell::new(p.stage.as_str(), tok),
                Cell::new(format!("{}/{}", p.landed, p.ops), Token::Text),
                {
                    let (w, t) = verify_outcome(app, p);
                    Cell::new(w, t)
                },
                Cell::new(
                    p.approval.clone().unwrap_or_else(|| "—".into()),
                    Token::Muted,
                ),
            ],
            url: None,
        });
        // Enter on a package lists its operations, each with its "Open in Xero" link (Enter / `o` opens it)
        if app.ui.system.open_pkg.as_deref() == Some(p.hash.as_str()) {
            for (i, (id, kind, org, marker)) in p.op_list.iter().enumerate() {
                let url = p.op_urls.get(i).cloned().flatten();
                let state = p.op_states.get(i).cloned().unwrap_or_default();
                out.push(Item {
                    key: format!("pkgop:{}:{id}", p.hash),
                    section: Section::Writes,
                    // the package's title too: a filter that shows the package shows its operations
                    detail: match &url {
                        Some(u) => format!("{} › {id} {kind} ({org}) · {marker} · {u}", p.title),
                        None => format!(
                            "{} › {id} {kind} ({org}) · {marker} · no Xero link yet (created by the run, or no ShortCode synced)",
                            p.title
                        ),
                    },
                    cells: vec![
                        Cell::new(format!("  › {id} {kind}"), Token::Text),
                        Cell::new(org.clone(), Token::Muted),
                        Cell::new(
                            if state.is_empty() { "—".to_string() } else { state.clone() },
                            match state.as_str() {
                                "rejected" => Token::Error,
                                "uncertain" => Token::Warn,
                                "confirmed" | "reconciled" => Token::Ok,
                                _ => Token::Muted,
                            },
                        ),
                        Cell::new("", Token::Muted),
                        Cell::new("", Token::Muted),
                        Cell::new(
                            if url.is_some() { "open in Xero" } else { "no link" },
                            if url.is_some() { Token::Info } else { Token::Muted },
                        ),
                    ],
                    url,
                });
            }
        }
    }
    out
}

fn matches(item: &Item, f: &crate::filter::Filter) -> bool {
    let hay = item
        .cells
        .iter()
        .map(|c| c.text.as_str())
        .chain([item.detail.as_str()])
        .collect::<Vec<_>>()
        .join("\n");
    f.matches(&hay, None)
}

/// Selectable System rows before the `/` filter (the chip's "of N").
pub fn item_total(app: &App) -> usize {
    items(app).len()
}

pub type Layout = Rc<(Vec<Item>, Vec<Line>)>;
type FitsKey = (u64, String, i64, usize, bool);
type LayoutKey = (u64, String, i64);

thread_local! {
    /// The prepared list keyed by (load generation, filter, minute): the settings, links and org rows are
    /// formatted once per load (and once a minute, for ages), not on every arrival, key or redraw.
    static LAYOUT: RefCell<Option<(LayoutKey, Layout)>> = const { RefCell::new(None) };
    /// Per-section column fits, keyed by the layout key plus width and glyph set.
    static FITS: RefCell<Option<(FitsKey, Rc<Vec<table::Fitted>>)>> = const { RefCell::new(None) };
}

thread_local! {
    static BUILDS: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
}

/// How many times this thread rebuilt the System list or its column fits (tests: arrival reuses them).
pub fn builds() -> u64 {
    BUILDS.with(std::cell::Cell::get)
}

fn built() {
    BUILDS.with(|b| b.set(b.get() + 1));
}

fn layout_key(app: &App) -> (u64, String, i64) {
    (
        app.model.system.generation,
        // the filter, and the package whose operations are listed (both change the rows)
        format!(
            "{}\u{0}{}",
            app.ui.filter.to_lowercase(),
            app.ui.system.open_pkg.as_deref().unwrap_or("")
        ),
        app.now.div_euclid(60),
    )
}

/// The visible items (through the filter) and the list's lines (cached; see `LAYOUT`).
pub fn layout(app: &App) -> Layout {
    let key = layout_key(app);
    LAYOUT.with(|c| {
        if let Some((k, l)) = c.borrow().as_ref()
            && *k == key
        {
            return l.clone();
        }
        built();
        let l = Rc::new(build_layout(app));
        *c.borrow_mut() = Some((key, l.clone()));
        l
    })
}

fn fits(app: &App, items: &[Item], avail: usize) -> Rc<Vec<table::Fitted>> {
    let th = &app.ui.theme;
    let (g, f, m) = layout_key(app);
    let key = (g, f, m, avail, th.ascii);
    FITS.with(|c| {
        if let Some((k, v)) = c.borrow().as_ref()
            && *k == key
        {
            return v.clone();
        }
        built();
        let v: Rc<Vec<table::Fitted>> = Rc::new(
            Section::ALL
                .iter()
                .map(|&s| {
                    let rows: Vec<Vec<Cell>> = items
                        .iter()
                        .filter(|i| i.section == s)
                        .map(|i| i.cells.clone())
                        .collect();
                    table::fit(th, s.cols(), &rows, s.steps(), avail)
                })
                .collect(),
        );
        *c.borrow_mut() = Some((key, v.clone()));
        v
    })
}

fn build_layout(app: &App) -> (Vec<Item>, Vec<Line>) {
    let q = app.ui.filter.to_lowercase();
    let f = crate::filter::Filter::parse(&app.ui.filter);
    let all: Vec<Item> = items(app).into_iter().filter(|i| matches(i, &f)).collect();
    let mut lines = Vec::new();
    for s in Section::ALL {
        let idx: Vec<usize> = (0..all.len()).filter(|&i| all[i].section == s).collect();
        let notes: Vec<(String, Token)> = notes(app, s)
            .into_iter()
            .filter(|(t, _)| f.matches(t, None))
            .collect();
        if (!q.is_empty() || s == Section::Writes) && idx.is_empty() && notes.is_empty() {
            continue;
        }
        if !lines.is_empty() {
            lines.push(Line::Blank);
        }
        lines.push(Line::Heading(s));
        lines.extend(notes.into_iter().map(|(t, k)| Line::Note(t, k)));
        if !idx.is_empty() {
            lines.push(Line::Header(s));
            lines.extend(idx.into_iter().map(Line::Item));
        }
    }
    (all, lines)
}

/// The cursor as an index into the visible items (the key resolved; the first item when it vanished).
pub fn cursor(app: &App, items: &[Item]) -> Option<usize> {
    if items.is_empty() {
        return None;
    }
    Some(
        app.ui
            .system
            .sel
            .as_ref()
            .and_then(|k| items.iter().position(|i| &i.key == k))
            .unwrap_or(0),
    )
}

pub fn selected(app: &App) -> Option<Item> {
    let lay = layout(app);
    let items = &lay.0;
    let c = cursor(app, items)?;
    items.get(c).cloned()
}

fn page(app: &App) -> usize {
    app.hits.content.map_or(10, |r| (r.height as usize).max(1))
}

/// The first visible line: the stored top, moved just enough to show the cursor's line (draw-safe).
fn visible_top(app: &App, lines: &[Line], cur: Option<usize>, h: usize) -> usize {
    let mut top = app.ui.system.top.min(lines.len().saturating_sub(1));
    if let Some(c) = cur
        && let Some(l) = lines.iter().position(|l| *l == Line::Item(c))
    {
        // keep the section heading in view when the cursor is among its first rows
        let head = lines[..l]
            .iter()
            .rposition(|x| matches!(x, Line::Heading(_)))
            .filter(|&p| l - p <= 4)
            .unwrap_or(l);
        if head < top {
            top = head;
        } else if h > 1 && l >= top + h - 1 {
            top = l + 2 - h;
        }
    }
    top
}

fn select(app: &mut App, pos: usize) {
    let lay = layout(app);
    let (items, lines) = (&lay.0, &lay.1);
    if let Some(it) = items.get(pos.min(items.len().saturating_sub(1))) {
        app.ui.system.sel = Some(it.key.clone());
    }
    let cur = cursor(app, items);
    app.ui.system.top = visible_top(app, lines, cur, page(app));
}

pub fn handle(app: &mut App, action: &str, fx: &mut Vec<Effect>) -> bool {
    if !owns(app) || app.ui.band.is_some() || app.ui.prompt.is_some() {
        return false;
    }
    match action {
        "system.edit_config" => {
            match &app.source.sys.path {
                Some(p) => {
                    fx.push(Effect::Edit(p.clone()));
                    app.ui.note = Some(format!("editing {} · restart to apply", app.paths.show(p)));
                }
                None => {
                    app.ui.note = Some(
                        "No config file to edit: create ownpurse.json from ownpurse.example.json \
                         (`ownpurse init` is coming in v0.9.x)."
                            .into(),
                    )
                }
            }
            true
        }
        // Enter on a write package lists its operations and moves to the first (Enter again folds them)
        "drill.in" if selected(app).is_some_and(|i| i.key.starts_with("pkg:")) => {
            let key = selected(app).map(|i| i.key).unwrap_or_default();
            let hash = key.trim_start_matches("pkg:").to_string();
            if app.ui.system.open_pkg.as_deref() == Some(hash.as_str()) {
                app.ui.system.open_pkg = None;
            } else {
                app.ui.system.open_pkg = Some(hash.clone());
                app.ui.note = Some(
                    "operations listed · Enter or o on one opens it in Xero · Esc folds".into(),
                );
                let lay = layout(app);
                let first = format!("pkgop:{hash}:");
                if let Some(i) = lay.0.iter().position(|it| it.key.starts_with(&first)) {
                    select(app, i);
                }
            }
            true
        }
        // Esc with a package's operations listed folds them and goes back to the package
        "drill.back" if app.ui.system.open_pkg.is_some() => {
            let hash = app.ui.system.open_pkg.take().unwrap_or_default();
            let lay = layout(app);
            let key = format!("pkg:{hash}");
            if let Some(i) = lay.0.iter().position(|it| it.key == key) {
                select(app, i);
            }
            true
        }
        "link.open" | "link.copy" | "drill.in" => {
            if let Some(url) = selected(app).and_then(|i| i.url) {
                if action == "link.copy" {
                    app.ui.note = Some(format!("copied {url}"));
                    fx.push(Effect::Copy(url));
                } else {
                    app.ui.note = Some(format!("opening {url}"));
                    fx.push(Effect::Open(url));
                }
            }
            true
        }
        a => {
            let lay = layout(app);
            let items = &lay.0;
            let n = items.len();
            let cur = cursor(app, items).unwrap_or(0);
            match super::activity::moved(a, cur, n, page(app).saturating_sub(4).max(1)) {
                Some(pos) if n > 0 => {
                    select(app, pos);
                    true
                }
                Some(_) => true,
                None => false,
            }
        }
    }
}

pub fn holds(app: &App, w: When) -> Option<bool> {
    if app.ui.tab != Tab::System {
        return None;
    }
    let section = || selected(app).map(|i| i.section);
    match w {
        When::HasRow => Some(selected(app).is_some()),
        When::SettingsSection => Some(section() == Some(Section::Settings)),
        // a Links row, or a write operation with its "Open in Xero" link
        When::LinksSection => Some(
            section() == Some(Section::Links) || selected(app).is_some_and(|i| i.url.is_some()),
        ),
        _ => None,
    }
}

/// A click on a list line: rows register `Target::Row(<item index>)` (headings and notes don't).
pub fn click_row(app: &mut App, i: usize, clicks: u8, fx: &mut Vec<Effect>) {
    select(app, i);
    if clicks >= 2 {
        handle(app, "drill.in", fx);
    }
}

pub fn wheel(app: &mut App, down: bool) -> bool {
    if !owns(app) {
        return false;
    }
    let lay = layout(app);
    let (items, lines) = (&lay.0, &lay.1);
    let h = page(app);
    let top = visible_top(app, lines, cursor(app, items), h);
    let new_top = if down {
        (top + 1).min(lines.len().saturating_sub(h))
    } else {
        top.saturating_sub(1)
    };
    app.ui.system.top = new_top;
    // the wheel never moves the selection unless it left the view
    if let Some(c) = cursor(app, items)
        && let Some(l) = lines.iter().position(|x| *x == Line::Item(c))
        && (l < new_top || l >= new_top + h)
    {
        let visible = lines[new_top..(new_top + h).min(lines.len())]
            .iter()
            .filter_map(|x| match x {
                Line::Item(i) => Some(*i),
                _ => None,
            });
        let pick = if l < new_top {
            visible.min()
        } else {
            visible.max()
        };
        if let Some(p) = pick {
            app.ui.system.sel = Some(items[p].key.clone());
        }
    }
    true
}

pub fn status(app: &App) -> String {
    let lay = layout(app);
    let items = &lay.0;
    let count = |s: Section| items.iter().filter(|i| i.section == s).count();
    format!(
        "system · {} orgs · {} settings · {} links",
        count(Section::Connection),
        count(Section::Settings),
        count(Section::Links)
    )
}

pub fn draw(app: &App, buf: &mut Buffer, area: Rect, hits: &mut Hits) {
    let th = &app.ui.theme;
    let g = th.glyphs();
    let lay = layout(app);
    let (items, lines) = (&lay.0, &lay.1);
    let mut area = area;
    if let Some(text) = app.filter_chip() {
        let h = strip::height(&text, area.width).min(area.height);
        strip::draw(th, buf, Rect { height: h, ..area }, &text, Token::Muted);
        area.y += h;
        area.height -= h;
    }
    let cur = cursor(app, items);
    let detail = cur.map(|c| th.text(&items[c].detail));
    let band_h: u16 = if detail.is_some() { 2 } else { 0 };
    let list_h = area.height.saturating_sub(band_h);
    if list_h == 0 {
        return;
    }
    let list = Rect::new(area.x, area.y, area.width, list_h);
    hits.content = Some(list);
    let avail = (area.width as usize).saturating_sub(2);
    // per-section column widths over all of the section's (filtered) rows, so scrolling never re-fits
    let fits = fits(app, items, avail);
    let sec_i = |s: Section| Section::ALL.iter().position(|x| *x == s).unwrap_or(0);
    let top = visible_top(app, lines, cur, list_h as usize);
    if items.is_empty() && !app.ui.filter.is_empty() {
        buf.set_span(
            area.x + 1,
            area.y,
            &Span::styled("Nothing matches the filter.", th.fg(Token::Muted)),
            area.width.saturating_sub(2),
        );
    }
    let cells_line =
        |buf: &mut Buffer, y: u16, s: Section, texts: &[(String, Token)], sel: bool| {
            let f = &fits[sec_i(s)];
            let mut x = area.x + 2;
            for (k, c) in s.cols().iter().enumerate() {
                let Some(cw) = f.widths[k] else { continue };
                let (t, tok) = &texts[k];
                let right = matches!(c.kind, Kind::Amount);
                let mut style = th.fg(*tok);
                if sel {
                    style = style.patch(th.selection());
                }
                let cell = pad(&th.text(t), cw, g.ellipsis, right);
                buf.set_span(x, y, &Span::styled(cell, style), cw as u16);
                x += (cw + table::GAP) as u16;
            }
        };
    // a section scrolled past its heading keeps the heading pinned on the first row
    let pinned = lines.get(top).and_then(|l| match l {
        Line::Heading(_) => None,
        _ => lines[..top].iter().rev().find_map(|x| match x {
            Line::Heading(s) => Some(*s),
            _ => None,
        }),
    });
    let first = if let Some(s) = pinned {
        crate::widgets::band(
            th,
            buf,
            Rect::new(area.x, list.y, area.width, 1),
            &format!("{} (continued)", s.title()),
            None,
        );
        1
    } else {
        0
    };
    for (n, line) in lines
        .iter()
        .enumerate()
        .skip(top)
        .take((list_h as usize).saturating_sub(first))
    {
        let y = list.y + first as u16 + (n - top) as u16;
        match line {
            Line::Blank => {}
            Line::Heading(s) => {
                crate::widgets::band(
                    th,
                    buf,
                    Rect::new(area.x, y, area.width, 1),
                    s.title(),
                    None,
                );
            }
            Line::Note(t, tok) => {
                let narrow;
                let t = if area.width < SCOPES_WIDE && t.starts_with("scopes:") {
                    narrow = scopes_note(&app.source.sys, false);
                    &narrow
                } else {
                    t
                };
                put(
                    buf,
                    area.x + 2,
                    y,
                    vec![Span::styled(th.text(t), th.fg(*tok))],
                    area.width.saturating_sub(3),
                    g.ellipsis,
                );
            }
            Line::Header(s) => {
                let texts: Vec<(String, Token)> = s
                    .cols()
                    .iter()
                    .map(|c| (c.title.to_string(), Token::Muted))
                    .collect();
                cells_line(buf, y, *s, &texts, false);
            }
            Line::Item(i) => {
                let it = &items[*i];
                let sel = cur == Some(*i);
                if sel {
                    let st = th.selection();
                    for xx in area.left()..area.right() {
                        buf[(xx, y)].set_style(st);
                    }
                    buf.set_span(area.x, y, &Span::styled(g.cursor, th.fg(Token::Accent)), 1);
                }
                let texts: Vec<(String, Token)> =
                    it.cells.iter().map(|c| (c.text.clone(), c.token)).collect();
                cells_line(buf, y, it.section, &texts, sel);
                if sel {
                    th.band_row(buf, Rect::new(area.x, y, area.width, 1));
                }
                hits.add(Rect::new(area.x, y, area.width, 1), Target::Row(*i));
            }
        }
    }
    if let Some(d) = detail {
        let ry = list.bottom();
        let rule = g.rule.repeat(area.width as usize);
        buf.set_span(
            area.x,
            ry,
            &Span::styled(rule, th.fg(Token::Line)),
            area.width,
        );
        let w = area.width.saturating_sub(2) as usize;
        let shown = if width(&d) > w {
            fit_str(&d, w, g.ellipsis)
        } else {
            d
        };
        buf.set_span(
            area.x + 1,
            ry + 1,
            &Span::styled(shown, th.fg(Token::Text)),
            area.width.saturating_sub(2),
        );
    }
}

/// A package's verify outcome as far as the record shows it: `verified` (every op landed and the after-run
/// report is in the record), `rejected` / `uncertain` (a stopped attempt), `incomplete` (landed or running
/// without the verification yet), or `—` before any attempt. The outcome `write verify` recorded wins
/// (`discrepancy` comes only from there).
fn verify_outcome(app: &App, p: &ownpurse_core::write::view::PackageRow) -> (&'static str, Token) {
    use ownpurse_core::write::view::Stage;
    // what `write verify` concluded, when the record keeps it (v0.13.0), before what the stage implies
    match p.verify.as_deref() {
        Some("verified") if p.stage >= Stage::Landed => return ("verified", Token::Ok),
        Some("discrepancy") => return ("discrepancy", Token::Error),
        Some("rejected") => return ("rejected", Token::Error),
        Some("uncertain") => return ("uncertain", Token::Warn),
        _ => {}
    }
    match p.stage {
        Stage::Verified | Stage::Recovered => ("verified", Token::Ok),
        Stage::Stopped => {
            let states = app
                .books
                .as_ref()
                .map(|b| ownpurse_core::write::view::op_states(b.record(), p.hash12()))
                .unwrap_or_default();
            if states.iter().any(|(_, s, _)| s == "rejected") {
                ("rejected", Token::Error)
            } else {
                ("uncertain", Token::Warn)
            }
        }
        Stage::Landed | Stage::Running => ("incomplete", Token::Warn),
        Stage::Sealed | Stage::Approved => ("—", Token::Muted),
    }
}
