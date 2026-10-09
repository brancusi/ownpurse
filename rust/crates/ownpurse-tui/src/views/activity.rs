//! Tab 4: Activity (spec §9 row 4, merge M2). A budget summary (today's calls and the per-minute rate,
//! then one line per org: calls today, calls left with a bar, last sync), then the HTTP log from the
//! record with status colours that always carry their code (V12) and a detail band for the selected
//! row. `G` groups the log by endpoint (calls, errors, avg ms, last). `/` filters; the scope picks the
//! orgs (all = every row, including identity calls).

use crate::activity::{EndpointRow, LogRow, StatusClass, by_endpoint};
use crate::app::{App, Effect, Scope, Tab, level_token};
use crate::clock::{self, parse_utc};
use crate::hit::Hits;
use crate::keymap::When;
use crate::layout::{ColSpec, Kind, fit_str, width};
use crate::model::Level;
use crate::theme::Theme;
use crate::theme_tokens::Token;
use crate::widgets::table::{self, Cell, Step, Table};
use crate::widgets::{put, strip};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::text::Span;
use std::cell::RefCell;
use std::rc::Rc;

/// Activity state: cursors are ids (navigation rule 4), the log row id or the endpoint.
#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ActivityUi {
    pub grouped: bool,
    pub sel: Option<i64>,
    pub ep_sel: Option<String>,
    pub top: usize,
}

/// Log columns (spec §4.3 Activity: method → ms → endpoint shortens; time loses its date part).
pub const LOG_COLS: [ColSpec; 7] = [
    ColSpec {
        title: "time",
        short_title: "",
        kind: Kind::Status,
        priority: 9,
    },
    ColSpec {
        title: "org",
        short_title: "",
        kind: Kind::Text { min: 5 },
        priority: 8,
    },
    ColSpec {
        title: "method",
        short_title: "",
        kind: Kind::Id,
        priority: 1,
    },
    ColSpec {
        title: "endpoint",
        short_title: "",
        kind: Kind::Text { min: 12 },
        priority: 4,
    },
    ColSpec {
        title: "result",
        short_title: "",
        kind: Kind::Status,
        priority: 9,
    },
    ColSpec {
        title: "ms",
        short_title: "",
        kind: Kind::Id,
        priority: 2,
    },
    ColSpec {
        title: "day left",
        short_title: "",
        kind: Kind::Amount,
        priority: 9,
    },
];

/// The declared hide order before any text shortens: method, then ms (the endpoint shortens after).
pub const LOG_STEPS: &[Step] = &[Step::Hide(2), Step::Hide(5)];

/// Grouped by endpoint (`G`).
pub const EP_COLS: [ColSpec; 5] = [
    ColSpec {
        title: "endpoint",
        short_title: "",
        kind: Kind::Text { min: 12 },
        priority: 9,
    },
    ColSpec {
        title: "calls",
        short_title: "",
        kind: Kind::Amount,
        priority: 9,
    },
    ColSpec {
        title: "errors",
        short_title: "",
        kind: Kind::Amount,
        priority: 9,
    },
    ColSpec {
        title: "avg ms",
        short_title: "",
        kind: Kind::Amount,
        priority: 9,
    },
    ColSpec {
        title: "last",
        short_title: "",
        kind: Kind::Status,
        priority: 9,
    },
];

/// Below this width the time column drops the date of older rows.
pub const DATE_COLUMNS: u16 = 100;

pub fn owns(app: &App) -> bool {
    app.ui.tab == Tab::Activity && app.ui.overlay.is_none()
}

/// Tenants of the current scope (`None` = every row, including identity calls).
fn scope_tenants(app: &App) -> Option<Vec<&str>> {
    match app.ui.scope {
        Scope::All => None,
        ref s => Some(
            app.members(s)
                .into_iter()
                .map(|i| app.model.orgs[i].tenant.as_str())
                .collect(),
        ),
    }
}

fn in_scope(tenants: &Option<Vec<&str>>, r: &LogRow) -> bool {
    // the agent bridge's rows belong to the session, not an org: every scope shows them
    r.is_agent()
        || tenants
            .as_ref()
            .is_none_or(|ts| r.tenant.as_deref().is_some_and(|t| ts.contains(&t)))
}

type Prepared = (Rc<Vec<usize>>, Rc<Vec<EndpointRow>>);
type PreparedKey = (u64, String, String);
type CellsKey = (PreparedKey, (bool, bool), i64);
type LogCells = (Rc<Vec<Vec<Cell>>>, Rc<Vec<Cell>>);

/// Per column, the cell with the widest text (the hidden sizing row).
fn widest_row(cells: &[Vec<Cell>]) -> Vec<Cell> {
    (0..LOG_COLS.len())
        .map(|i| {
            cells
                .iter()
                .map(|r| &r[i])
                .max_by_key(|c| width(&c.text))
                .cloned()
                .unwrap_or_else(|| Cell::new("", Token::Text))
        })
        .collect()
}

thread_local! {
    /// The filtered log (indexes) and its endpoint grouping, keyed by (load generation, scope, filter):
    /// tab arrival, cursor moves and redraws reuse it; a reload, scope or filter change rebuilds it.
    static PREPARED: RefCell<Option<(PreparedKey, Prepared)>> = const { RefCell::new(None) };
    /// The log's cells for the visible rows' table, keyed by the prepared key plus date form and minute.
    static CELLS: RefCell<Option<(CellsKey, LogCells)>> = const { RefCell::new(None) };
}

thread_local! {
    static BUILDS: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
}

/// How many times this thread rebuilt an Activity view model (filtered log, cells, summary). Tests use it
/// to prove tab arrival and redraws reuse the caches (spec §14).
pub fn builds() -> u64 {
    BUILDS.with(std::cell::Cell::get)
}

fn built() {
    BUILDS.with(|b| b.set(b.get() + 1));
}

fn prepared(app: &App) -> Prepared {
    let key = (
        app.model.activity.generation,
        format!("{:?}", app.ui.scope),
        app.ui.filter.to_lowercase(),
    );
    PREPARED.with(|c| {
        if let Some((k, p)) = c.borrow().as_ref()
            && *k == key
        {
            return p.clone();
        }
        let tenants = scope_tenants(app);
        let f = crate::filter::Filter::parse(&app.ui.filter);
        let log = &app.model.activity.log;
        let idx: Vec<usize> = (0..log.len())
            .filter(|&i| in_scope(&tenants, &log[i]) && f.matches(&log[i].hay, None))
            .collect();
        let eps = by_endpoint(idx.iter().map(|&i| &log[i]));
        built();
        let p = (Rc::new(idx), Rc::new(eps));
        *c.borrow_mut() = Some((key, p.clone()));
        p
    })
}

/// Rows the current mode lists before the `/` filter (the chip's "of N").
pub fn unfiltered_len(app: &App) -> usize {
    let tenants = scope_tenants(app);
    let log = &app.model.activity.log;
    if app.ui.activity.grouped {
        by_endpoint(log.iter().filter(|r| in_scope(&tenants, r))).len()
    } else {
        log.iter().filter(|r| in_scope(&tenants, r)).count()
    }
}

/// The number of rows the current mode lists (through the filter).
pub fn shown_len(app: &App) -> usize {
    len(app)
}

/// Log rows in scope and through the filter, newest first.
pub fn rows(app: &App) -> Vec<&LogRow> {
    let log = &app.model.activity.log;
    prepared(app).0.iter().map(|&i| &log[i]).collect()
}

/// The log grouped by endpoint (`G`), busiest first.
pub fn endpoints(app: &App) -> Rc<Vec<EndpointRow>> {
    prepared(app).1
}

fn row_count(app: &App) -> usize {
    prepared(app).0.len()
}

/// The number of rows the current mode lists.
fn len(app: &App) -> usize {
    if app.ui.activity.grouped {
        endpoints(app).len()
    } else {
        row_count(app)
    }
}

/// The cursor's position in the current mode (the id resolved; the nearest survivor when it vanished).
pub fn cursor(app: &App) -> Option<usize> {
    let a = &app.ui.activity;
    if a.grouped {
        let eps = endpoints(app);
        if eps.is_empty() {
            return None;
        }
        Some(
            a.ep_sel
                .as_ref()
                .and_then(|s| eps.iter().position(|e| &e.endpoint == s))
                .unwrap_or(0),
        )
    } else {
        let rs = rows(app);
        if rs.is_empty() {
            return None;
        }
        // ids are monotonic: a vanished id lands on the nearest older surviving row
        Some(a.sel.map_or(0, |id| {
            rs.iter()
                .position(|r| r.id == id)
                .or_else(|| rs.iter().position(|r| r.id < id))
                .unwrap_or(rs.len() - 1)
        }))
    }
}

fn select(app: &mut App, pos: usize) {
    if app.ui.activity.grouped {
        let eps = endpoints(app);
        if let Some(e) = eps.get(pos.min(eps.len().saturating_sub(1))) {
            app.ui.activity.ep_sel = Some(e.endpoint.clone());
        }
    } else {
        let id = {
            let rs = rows(app);
            rs.get(pos.min(rs.len().saturating_sub(1))).map(|r| r.id)
        };
        if id.is_some() {
            app.ui.activity.sel = id;
        }
    }
    keep_visible(app);
}

fn page(app: &App) -> usize {
    app.hits
        .content
        .map_or(10, |r| (r.height as usize).saturating_sub(1).max(1))
}

fn keep_visible(app: &mut App) {
    let Some(cur) = cursor(app) else { return };
    let h = page(app);
    let top = &mut app.ui.activity.top;
    if cur < *top {
        *top = cur;
    } else if cur >= *top + h {
        *top = cur + 1 - h;
    }
}

/// The new cursor position for a movement action (shared with System).
pub fn moved(action: &str, cur: usize, n: usize, page: usize) -> Option<usize> {
    let last = n.saturating_sub(1);
    Some(match action {
        "row.prev" => cur.saturating_sub(1),
        "row.next" => (cur + 1).min(last),
        "page.prev" => cur.saturating_sub(page),
        "page.next" => (cur + page).min(last),
        "row.first" => 0,
        "row.last" => last,
        _ => return None,
    })
}

/// The tab's actions. Returns false for anything it doesn't own (the shell handles it).
pub fn handle(app: &mut App, action: &str, _fx: &mut [Effect]) -> bool {
    if !owns(app) || app.ui.band.is_some() || app.ui.prompt.is_some() {
        return false;
    }
    match action {
        "activity.group_endpoint" => {
            app.ui.activity.grouped = !app.ui.activity.grouped;
            app.ui.activity.top = 0;
            keep_visible(app);
            true
        }
        // an agent's classify write: "Undo this write?", Cancel focused; other rows have nothing below them
        "drill.in" => {
            if !app.ui.activity.grouped
                && let Some(c) = cursor(app)
                && let Some(r) = rows(app).get(c).map(|r| r.id)
                && let Some(w) = app.agent.writes.iter().find(|w| w.log_id == r && !w.undone)
            {
                let text = format!("Undo this write? {}", w.summary);
                app.ui.band = Some(crate::app::Band {
                    kind: crate::app::BandKind::Ask {
                        action: crate::app::NetAction::UndoAgentWrite(r),
                        label: "Undo".into(),
                    },
                    text,
                    focus: 0,
                });
            }
            true
        }
        a => {
            let n = len(app);
            match moved(a, cursor(app).unwrap_or(0), n, page(app)) {
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
    (app.ui.tab == Tab::Activity && w == When::HasRow).then(|| len(app) > 0)
}

pub fn click_row(app: &mut App, i: usize, _clicks: u8, _fx: &mut [Effect]) {
    let top = visible_top(app, page(app));
    select(app, top + i);
}

pub fn wheel(app: &mut App, down: bool) -> bool {
    if !owns(app) {
        return false;
    }
    let n = len(app);
    let h = page(app);
    let top = visible_top(app, h);
    let new_top = if down {
        (top + 1).min(n.saturating_sub(h))
    } else {
        top.saturating_sub(1)
    };
    app.ui.activity.top = new_top;
    // the wheel never moves the selection unless it left the view
    if let Some(cur) = cursor(app) {
        if cur < new_top {
            select(app, new_top);
        } else if cur >= new_top + h {
            select(app, new_top + h - 1);
        }
    }
    true
}

/// The first visible row as drawn (the agent bridge maps drawn rows to the log with it).
pub fn visible_top_pub(app: &App) -> usize {
    visible_top(app, page(app))
}

/// The first visible row: the stored top, moved just enough to show the cursor (pure; draw-safe).
fn visible_top(app: &App, h: usize) -> usize {
    let n = len(app);
    let mut top = app.ui.activity.top.min(n.saturating_sub(1));
    if let Some(cur) = cursor(app) {
        if cur < top {
            top = cur;
        } else if h > 0 && cur >= top + h {
            top = cur + 1 - h;
        }
    }
    top
}

pub fn status(app: &App) -> String {
    let scope = app.scope_label(&app.ui.scope);
    if app.ui.activity.grouped {
        format!(
            "{scope} · activity · {} endpoints · {} calls",
            endpoints(app).len(),
            clock::int(rows(app).len() as i64)
        )
    } else {
        format!(
            "{scope} · activity · {} calls in the log",
            clock::int(rows(app).len() as i64)
        )
    }
}

// ── budget summary ───────────────────────────────────────────────────────────────────────────────

fn sync_text(app: &App, i: usize, full: bool) -> (String, Level) {
    let f = app.model.freshness(&[i], app.now);
    let stale = if f.level == Level::Warn { " stale" } else { "" };
    let text = match &app.model.orgs[i].last_sync {
        None => "never synced".into(),
        Some(at) if full => format!(
            "last sync {} UTC · {} ago{stale}",
            clock::hhmm(at),
            clock::ago(f.age)
        ),
        Some(_) => format!("synced {} ago{stale}", clock::ago(f.age)),
    };
    (text, f.level)
}

/// The summary (3–5 lines for up to four orgs): `Today (UTC): …` with the per-minute rate, then one
/// line per org in scope (at most four; with more, three and a line naming the rest).
type Summary = Rc<Vec<Vec<Span<'static>>>>;
type SummaryKey = (
    u64,
    String,
    usize,
    &'static str,
    bool,
    i64,
    (u64, u32, bool),
);

thread_local! {
    /// The budget summary keyed by (load generation, scope, width, theme, minute).
    static SUMMARY: RefCell<Option<(SummaryKey, Summary)>> = const { RefCell::new(None) };
}

/// The summary (cached per load, scope, width, theme and minute; see `build_summary`).
pub fn summary(app: &App, w: usize) -> Summary {
    let key = (
        app.model.activity.generation,
        format!("{:?}", app.ui.scope),
        w,
        app.ui.theme.name(),
        app.ui.theme.ascii,
        app.now.div_euclid(60),
        // the resolved colours: an overlay, a user theme or the accent changes them
        (
            app.ui.theme_revision,
            app.ui.theme.rgb(Token::Accent),
            app.ui.details,
        ),
    );
    SUMMARY.with(|c| {
        if let Some((k, v)) = c.borrow().as_ref()
            && *k == key
        {
            return v.clone();
        }
        built();
        let v = Rc::new(build_summary(app, w));
        *c.borrow_mut() = Some((key, v.clone()));
        v
    })
}

fn build_summary(app: &App, w: usize) -> Vec<Vec<Span<'static>>> {
    let th = &app.ui.theme;
    let today = clock::date_of(app.now);
    let members = app.members(&app.ui.scope);
    let act = &app.model.activity;
    let tenants = scope_tenants(app);
    let today_rows: Vec<&LogRow> = act
        .log
        .iter()
        .filter(|r| in_scope(&tenants, r) && r.at.get(..10) == Some(today.as_str()))
        .collect();
    let calls = today_rows.len() as i64;
    let errors = today_rows.iter().filter(|r| r.is_error()).count() as i64;
    let minute = members
        .iter()
        .filter_map(|&i| act.last_minute.get(i))
        .copied()
        .max()
        .unwrap_or(0);
    let sep = th.text(" · ");
    let head = format!("Today (UTC): {} calls", clock::int(calls));
    let errs = format!("{} errors", clock::int(errors));
    let mut per = format!(
        "Per minute: {minute} of {} (soft limit; Xero allows {})",
        act.minute_soft, act.minute_hard
    );
    if minute >= act.minute_soft {
        per.push_str(" · at the soft limit");
    }
    let mut first = vec![Span::styled(head.clone(), th.heading())];
    // the error count goes first when the line is tight (the log shows every error anyway)
    if width(&head) + width(&errs) + width(&per) + 2 * width(&sep) <= w {
        first.push(Span::styled(sep.clone(), th.fg(Token::Muted)));
        first.push(Span::styled(
            errs,
            th.fg(if errors > 0 {
                Token::Error
            } else {
                Token::Muted
            }),
        ));
    }
    first.push(Span::styled(sep.clone(), th.fg(Token::Muted)));
    first.push(Span::styled(
        per,
        th.fg(if minute >= act.minute_soft {
            Token::Error
        } else {
            Token::Text
        }),
    ));
    let mut out = vec![first];

    let shown: Vec<usize> = if members.len() > 4 {
        members[..3].to_vec()
    } else {
        members.clone()
    };
    let limit = app.model.limit;
    // columns line up across orgs: the alias padded to the longest in scope, counts right-aligned
    let alias_w = shown
        .iter()
        .map(|&i| width(&app.model.orgs[i].alias))
        .max()
        .unwrap_or(0);
    let calls_w = shown
        .iter()
        .map(|&i| clock::int(app.model.orgs[i].calls_today).len())
        .max()
        .unwrap_or(0);
    let left_w = shown
        .iter()
        .map(|&i| width(&left_cell(&app.model.orgs[i])))
        .max()
        .unwrap_or(0);
    let texts: Vec<(usize, String, Level)> = shown
        .iter()
        .map(|&i| {
            let o = &app.model.orgs[i];
            let (_, level) = app.model.quota(&[i]);
            let low = if level == Level::Ok { "" } else { " · low" };
            // design pass 2: `roast  ━━───  967 left`; today's count and the last sync are details (`i`)
            if !app.ui.details {
                return (
                    i,
                    format!("{}{}", o.alias, " ".repeat(alias_w - width(&o.alias))),
                    level,
                );
            }
            let text = th.text(&format!(
                "{}{}  {:>calls_w$} {} today · {:>left_w$} left of {}{low}",
                o.alias,
                " ".repeat(alias_w - width(&o.alias)),
                clock::int(o.calls_today),
                if o.calls_today == 1 { "call " } else { "calls" },
                left_cell(o),
                clock::int(limit)
            ));
            (i, text, level)
        })
        .collect();
    let text_w = texts.iter().map(|(_, t, _)| width(t)).max().unwrap_or(0);
    let sync_w = |full: bool| {
        shown
            .iter()
            .map(|&i| width(&th.text(&sync_text(app, i, full).0)))
            .max()
            .unwrap_or(0)
    };
    let sep_w = width(&sep);
    // one form for every line: the richest whose widest line fits (bar cells, sync 2 full / 1 short / 0)
    let forms = [(20, 2), (12, 2), (12, 1), (10, 1), (10, 0), (0, 0)];
    let (bar, sync) = forms
        .iter()
        .copied()
        .find(|&(bar, sync)| {
            let sw = match sync {
                2 => sep_w + sync_w(true),
                1 => sep_w + sync_w(false),
                _ => 0,
            };
            text_w + if bar > 0 { bar + 2 } else { 0 } + sw <= w
        })
        .unwrap_or((0, 0));
    for (i, text, level) in texts {
        let o = &app.model.orgs[i];
        let lt = match level {
            Level::Ok => Token::Text,
            l => level_token(l),
        };
        let mut line = vec![Span::styled(
            format!("{text}{}", " ".repeat(text_w - width(&text))),
            th.fg(lt),
        )];
        if bar > 0 {
            // cells used, rounded up so any spend shows (integer arithmetic; no floats)
            let spent = (limit - o.left).clamp(0, limit.max(1));
            let used =
                ((spent * bar as i64 + limit - 1) / limit.max(1)).clamp(0, bar as i64) as usize;
            let (full, empty) = if th.ascii { ("=", "-") } else { ("━", "─") };
            line.push(Span::raw("  "));
            line.push(Span::styled(
                full.repeat(used),
                th.fg(if level == Level::Ok {
                    Token::Accent
                } else {
                    lt
                }),
            ));
            line.push(Span::styled(empty.repeat(bar - used), th.fg(Token::Line)));
        }
        if !app.ui.details {
            let low = if level == Level::Ok { "" } else { " · low" };
            line.push(Span::styled(
                th.text(&format!("  {} left{low}", left_cell(o))),
                th.fg(lt),
            ));
        }
        if sync > 0 && app.ui.details {
            let (s, l) = sync_text(app, i, sync == 2);
            let tok = match l {
                Level::Ok => Token::Muted,
                l => level_token(l),
            };
            line.push(Span::styled(sep.clone(), th.fg(Token::Muted)));
            line.push(Span::styled(th.text(&s), th.fg(tok)));
        }
        out.push(line);
    }
    if members.len() > 4 {
        let rest: Vec<&str> = members[3..]
            .iter()
            .map(|&i| app.model.orgs[i].alias.as_str())
            .collect();
        out.push(vec![Span::styled(
            format!("+ {} more: {}", rest.len(), rest.join(", ")),
            th.fg(Token::Muted),
        )]);
    }
    out
}

// ── drawing ──────────────────────────────────────────────────────────────────────────────────────

/// Status colours (acceptance K3): 2xx muted, 4xx warn, 429 / 5xx / problem error. The code is the word.
pub fn class_token(c: StatusClass) -> Token {
    match c {
        StatusClass::Ok => Token::Muted,
        StatusClass::Client => Token::Warn,
        StatusClass::Error => Token::Error,
    }
}

pub fn log_cells(app: &App, rows: &[&LogRow], with_date: bool) -> Vec<Vec<Cell>> {
    let today = clock::date_of(app.now);
    let ms_text = |r: &LogRow| r.ms.map_or("—".to_string(), clock::int);
    let ms_w = rows.iter().map(|r| width(&ms_text(r))).max().unwrap_or(1);
    // without the details layer the columns take the design's fixed widths
    let fixed = !app.ui.details;
    let ms_w = if fixed { ms_w.max(4) } else { ms_w };
    let cap = |s: String, w: usize| -> String { if fixed { fit_str(&s, w, "…") } else { s } };
    rows.iter()
        .map(|r| {
            let ms = ms_text(r);
            vec![
                Cell::new(r.time(&today, with_date), Token::Muted),
                Cell::new(cap(r.org.clone(), 8), Token::Text),
                Cell::new(r.method.clone(), Token::Muted),
                // design pass 2: the method shows only when it isn't GET (its column is a detail)
                Cell::new(
                    cap(
                        if app.ui.details || r.method == "GET" || r.method.is_empty() {
                            r.endpoint.clone()
                        } else {
                            format!("{} {}", r.method, r.endpoint)
                        },
                        36,
                    ),
                    Token::Text,
                ),
                Cell::new(
                    cap(result_text(r, app.ui.details), 16),
                    class_token(r.class()),
                ),
                Cell::new(
                    format!("{}{ms}", " ".repeat(ms_w - width(&ms))),
                    Token::Muted,
                ),
                match r.day_left {
                    Some(n) => Cell::new(clock::int(n), Token::Text),
                    None => Cell::new("—", Token::Muted),
                },
            ]
        })
        .collect()
}

fn ago_of(app: &App, at: &str) -> String {
    parse_utc(at).map_or_else(String::new, |t| format!("{} ago", clock::ago(app.now - t)))
}

pub fn ep_cells(app: &App, eps: &[EndpointRow]) -> Vec<Vec<Cell>> {
    eps.iter()
        .map(|e| {
            vec![
                Cell::new(e.endpoint.clone(), Token::Text),
                Cell::new(clock::int(e.calls), Token::Text),
                Cell::new(
                    clock::int(e.errors),
                    if e.errors > 0 {
                        Token::Error
                    } else {
                        Token::Muted
                    },
                ),
                Cell::new(clock::int(e.avg_ms), Token::Text),
                Cell::new(ago_of(app, &e.last), Token::Muted),
            ]
        })
        .collect()
}

/// The detail band for the selected row: the full path, tenant, status with its problem text, timing
/// and the budget headers (two lines; one in the grouped view).
pub fn detail(app: &App) -> Vec<Vec<Span<'static>>> {
    let th = &app.ui.theme;
    let sep = || Span::styled(th.text(" · "), th.fg(Token::Muted));
    let Some(cur) = cursor(app) else {
        return Vec::new();
    };
    if app.ui.activity.grouped {
        let eps = endpoints(app);
        let e = &eps[cur];
        return vec![vec![
            Span::styled(e.endpoint.clone(), th.heading()),
            sep(),
            Span::styled(format!("{} calls", clock::int(e.calls)), th.fg(Token::Text)),
            sep(),
            Span::styled(
                format!(
                    "{} error{}",
                    clock::int(e.errors),
                    if e.errors == 1 { "" } else { "s" }
                ),
                th.fg(if e.errors > 0 {
                    Token::Error
                } else {
                    Token::Muted
                }),
            ),
            sep(),
            Span::styled(
                format!("avg {} ms", clock::int(e.avg_ms)),
                th.fg(Token::Text),
            ),
            sep(),
            Span::styled(
                format!(
                    "last {} UTC ({})",
                    e.last.get(..19).unwrap_or(&e.last).replace('T', " "),
                    ago_of(app, &e.last)
                ),
                th.fg(Token::Muted),
            ),
        ]];
    }
    let rs = rows(app);
    let r = rs[cur];
    let when = r.at.get(..19).unwrap_or(&r.at).replace('T', " ");
    if r.is_write() {
        // a write attempt (Phase 8): from the record's attempt log
        return vec![
            vec![
                Span::styled(format!("{when} UTC"), th.fg(Token::Text)),
                sep(),
                Span::styled(format!("write {}", r.endpoint), th.heading()),
            ],
            vec![Span::styled(
                th.text(&format!(
                    "{} · attempt {}{} · the record's attempt log (write status shows it all)",
                    r.path,
                    r.problem.as_deref().unwrap_or(""),
                    r.status.map_or(String::new(), |s| format!(" · HTTP {s}"))
                )),
                th.fg(Token::Muted),
            )],
        ];
    }
    if r.is_agent() {
        // the agent bridge's own rows: in memory, never a Xero call (and no id that means anything)
        return vec![
            vec![
                Span::styled(format!("{when} UTC"), th.fg(Token::Text)),
                sep(),
                Span::styled(format!("agent {}", r.path), th.heading()),
            ],
            vec![Span::styled(
                th.text("agent bridge (this session) · not a Xero call · not kept in the record"),
                th.fg(Token::Muted),
            )],
        ];
    }
    let who = match &r.tenant {
        Some(t) => format!("{} ({})", r.org, t.chars().take(8).collect::<String>()),
        None => "identity (no tenant)".into(),
    };
    let status = match (&r.status, &r.problem) {
        (Some(s), Some(p)) => format!("status {s} {p}"),
        (Some(s), None) => format!("status {s}"),
        (None, Some(p)) => format!("no status · {p}"),
        (None, None) => "no status".into(),
    };
    let opt = |v: Option<i64>| v.map_or("—".to_string(), clock::int);
    vec![
        vec![
            Span::styled(format!("#{}", r.id), th.fg(Token::Muted)),
            sep(),
            Span::styled(format!("{when} UTC"), th.fg(Token::Text)),
            sep(),
            Span::styled(format!("{} {}", r.method, r.path), th.heading()),
        ],
        vec![
            Span::styled(who, th.fg(Token::Text)),
            sep(),
            Span::styled(th.text(&status), th.fg(class_token(r.class()))),
            sep(),
            Span::styled(format!("{} ms", opt(r.ms)), th.fg(Token::Muted)),
            sep(),
            Span::styled(
                th.text(&format!(
                    "day left {} · minute left {}",
                    opt(r.day_left),
                    opt(r.min_left)
                )),
                th.fg(Token::Muted),
            ),
        ],
    ]
}

fn line(th: &Theme, buf: &mut Buffer, area: Rect, y: u16, spans: Vec<Span<'static>>) {
    if y < area.bottom() {
        put(
            buf,
            area.x + 2,
            y,
            spans,
            area.width.saturating_sub(4),
            th.glyphs().ellipsis,
        );
    }
}

/// Today's calls and errors in scope, and whether the per-minute rate is at its soft limit (the bands'
/// notes; design pass 2).
fn today_counts(app: &App) -> (i64, i64, bool) {
    let today = clock::date_of(app.now);
    let act = &app.model.activity;
    let tenants = scope_tenants(app);
    let rows: Vec<&LogRow> = act
        .log
        .iter()
        .filter(|r| in_scope(&tenants, r) && r.at.get(..10) == Some(today.as_str()))
        .collect();
    let minute = app
        .members(&app.ui.scope)
        .iter()
        .filter_map(|&i| act.last_minute.get(i))
        .copied()
        .max()
        .unwrap_or(0);
    (
        rows.len() as i64,
        rows.iter().filter(|r| r.is_error()).count() as i64,
        minute >= act.minute_soft,
    )
}

pub fn draw(app: &App, buf: &mut Buffer, area: Rect, hits: &mut Hits) {
    let th = &app.ui.theme;
    let w = area.width.saturating_sub(4) as usize;
    let mut y = area.y;
    let sum = summary(app, w);
    // design pass 2: calls left gets its own band with one bar per org (its main home now); the totals
    // line is a detail (`i`), unless the per-minute rate is at its soft limit
    let (calls, errors, at_soft) = today_counts(app);
    crate::widgets::band(
        th,
        buf,
        Rect::new(area.x, y, area.width, 1),
        "Calls left",
        Some(("Xero resets each org at its own time".into(), Token::Muted)),
    );
    y += 1;
    for (k, l) in sum.iter().enumerate() {
        if k == 0 && !(app.ui.details || at_soft) {
            continue;
        }
        line(th, buf, area, y, l.clone());
        y += 1;
    }
    y += app.section_gap();
    if y < area.bottom() {
        crate::widgets::band(
            th,
            buf,
            Rect::new(area.x, y, area.width, 1),
            "Recent calls",
            Some((
                format!(
                    "{} today · {}",
                    clock::int(calls),
                    crate::clock::plural(errors, "error", "errors")
                ),
                if errors > 0 {
                    Token::Warn
                } else {
                    Token::Muted
                },
            )),
        );
        y += 1;
    }
    let grouped = app.ui.activity.grouped;
    if let Some(text) = app.filter_chip() {
        let h = strip::height(&text, area.width).min(area.bottom().saturating_sub(y));
        strip::draw(
            th,
            buf,
            Rect::new(area.x, y, area.width, h),
            &text,
            Token::Muted,
        );
        y += h;
    }
    if y >= area.bottom() {
        return;
    }
    let detail = detail(app);
    let band_h = if detail.is_empty() {
        0
    } else {
        detail.len() as u16 + 1
    };
    let table_h = area.bottom().saturating_sub(y).saturating_sub(band_h);
    if table_h < 2 {
        return;
    }
    let t_area = Rect::new(area.x, y, area.width, table_h);
    hits.content = Some(t_area);
    let top = visible_top(app, (table_h as usize).saturating_sub(1));
    let cur = cursor(app);
    let empty = if grouped {
        let eps = endpoints(app);
        let cells = ep_cells(app, &eps);
        let t = Table {
            cols: &EP_COLS,
            rows: &cells,
            steps: &[],
            top,
            cursor: cur,
            cursor_col: None,
            footer: None,
            hidden: &[],
            focus: None,
            groups: &[],
        };
        table::draw(th, buf, t_area, &t, hits);
        eps.is_empty()
    } else {
        let rs = rows(app);
        // the date of older rows is a detail now (design pass 2: time 9 · org 8 · endpoint 36 · result 16 · ms 6)
        let with_date = area.width >= DATE_COLUMNS && app.ui.details;
        let key = (
            (
                app.model.activity.generation,
                format!("{:?}", app.ui.scope),
                app.ui.filter.to_lowercase(),
            ),
            (with_date, app.ui.details),
            app.now.div_euclid(60),
        );
        let (cells, widest) = CELLS.with(|c| {
            if let Some((k, v)) = c.borrow().as_ref()
                && *k == key
            {
                return v.clone();
            }
            built();
            let cells = log_cells(app, &rs, with_date);
            let mut widest = widest_row(&cells);
            // the design's fixed widths (the cells only truncate; the hidden sizing row sets the width)
            if !app.ui.details {
                for (i, w) in [(1usize, 8usize), (3, 36), (4, 16)] {
                    widest[i] = Cell::new(" ".repeat(w), Token::Text);
                }
            }
            let v = (Rc::new(cells), Rc::new(widest));
            *c.borrow_mut() = Some((key, v.clone()));
            v
        });
        // Only the visible window goes to the table, behind one hidden row holding each column's widest
        // text, so column widths are those of the whole log (stable while scrolling) at a window's cost.
        let body = (table_h as usize).saturating_sub(1);
        let shown: Vec<Vec<Cell>> = if cells.is_empty() {
            Vec::new()
        } else {
            std::iter::once(widest.as_ref().clone())
                .chain(cells.iter().skip(top).take(body).cloned())
                .collect()
        };
        let t = Table {
            cols: &LOG_COLS,
            rows: &shown,
            steps: LOG_STEPS,
            top: 1,
            cursor: cur
                .filter(|&c| c >= top && c < top + body)
                .map(|c| c - top + 1),
            cursor_col: None,
            footer: None,
            hidden: if app.ui.details { &[] } else { &[2, 6] },
            focus: None,
            groups: &[],
        };
        table::draw(th, buf, t_area, &t, hits);
        rs.is_empty()
    };
    if empty {
        let msg = if app.ui.filter.is_empty() {
            "No calls in the log for this scope."
        } else {
            "No call matches the filter."
        };
        line(
            th,
            buf,
            area,
            y + 1,
            vec![Span::styled(
                fit_str(msg, w.saturating_sub(1), th.glyphs().ellipsis),
                th.fg(Token::Muted),
            )],
        );
    }
    if !detail.is_empty() {
        let ry = y + table_h;
        if ry < area.bottom() {
            let rule = th.glyphs().rule.repeat(area.width as usize);
            buf.set_span(
                area.x,
                ry,
                &Span::styled(rule, th.fg(Token::Line)),
                area.width,
            );
        }
        for (k, l) in detail.into_iter().enumerate() {
            line(th, buf, area, ry + 1 + k as u16, l);
        }
    }
}

/// The quota strip's calls-left figure: `938`, or the local estimate `≈ 1,000 (est.)` (one rule everywhere).
fn left_cell(o: &crate::model::OrgInfo) -> String {
    if o.left_known {
        clock::int(o.left)
    } else {
        format!("{} (est.)", o.left_fig())
    }
}

/// A call's result (design pass 2): a quiet `ok`, else the code with what it means (`404 not found`,
/// `429 rate-limited`); the bare code under `i`.
pub fn result_text(r: &LogRow, details: bool) -> String {
    if details || r.is_agent() || r.is_write() {
        return r.status_text();
    }
    match (r.class(), r.status) {
        (StatusClass::Ok, _) => "ok".into(),
        (_, Some(s)) => {
            let what = match s {
                400 => "bad request",
                401 => "unauthorized",
                403 => "forbidden",
                404 => "not found",
                429 => "rate-limited",
                500..=599 => "server error",
                _ => "",
            };
            if what.is_empty() {
                s.to_string()
            } else {
                format!("{s} {what}")
            }
        }
        _ => r.status_text(),
    }
}
