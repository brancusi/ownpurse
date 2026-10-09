//! Tab 6: Classify (spec classify §1, §3.A; design rulings 2026-10-06). The mode sub-bar, then the Queue:
//! a virtualised list and the detail pane (list 67 / detail 32 at 100 columns; stacked under the list below
//! 100), the filter strip, the b/t/s pickers, the note editor and the bulk-accept preview. Colour means
//! attention: confidence and ✓ are quiet; only stale labels and refusals take warn/error. Draw never mutates.

use crate::app::App;
use crate::classify::{Mode, conf_bar, conf_text};
use crate::hit::{Hits, Target};
use crate::layout::{ColSpec, Kind, fit_str, pad, width, wrap};
use crate::theme::Theme;
use crate::theme_tokens::Token;
use crate::widgets::table::Cell;
use crate::widgets::{centred, fill, put};
use ownpurse_core::classify::{Cell as SCell, Glyph, QueueRow};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::text::Span;

/// The Queue's columns as `ui state` names them (every column the list can show).
pub const COLS: [ColSpec; 7] = [
    ColSpec {
        title: "org",
        short_title: "",
        kind: Kind::Text { min: 6 },
        priority: 6,
    },
    ColSpec {
        title: "date",
        short_title: "",
        kind: Kind::Id,
        priority: 5,
    },
    ColSpec {
        title: "payee",
        short_title: "",
        kind: Kind::Text { min: 14 },
        priority: 7,
    },
    ColSpec {
        title: "amount",
        short_title: "",
        kind: Kind::Amount,
        priority: 9,
    },
    ColSpec {
        title: "account",
        short_title: "",
        kind: Kind::Id,
        priority: 3,
    },
    ColSpec {
        title: "suggestion",
        short_title: "",
        kind: Kind::Text { min: 10 },
        priority: 8,
    },
    ColSpec {
        title: "conf",
        short_title: "",
        kind: Kind::Status,
        priority: 9,
    },
];

/// The detail pane sits beside the list from this width; below it stacks under the list.
pub const SIDE_BY_SIDE: u16 = 100;
/// From this width the list shows the account column, the full date and the confidence bar.
pub const WIDE: u16 = 140;

/// The project's display name; `no project` for the `none` choice.
fn project_name(c: &SCell) -> String {
    if c.value == "none" {
        "no project".into()
    } else {
        c.name.clone()
    }
}

/// The payee, or the description where there is none (journals).
pub fn payee_text(r: &QueueRow) -> String {
    if r.payee.trim().is_empty() {
        r.description.clone()
    } else {
        r.payee.clone()
    }
}

/// The suggestion cell (ruling 2): the project NAME (at most `name_w` cells) plus `+n` for the other
/// suggested labels (`Harvest Blen… +2`); `no project +2`; `· none` with nothing suggested; `stale …` when a
/// label went stale. `detail` false drops the `+n`; `glyph` prefixes the source (`R`, `J`, `H`, `A`).
pub fn suggestion_cell(
    r: &QueueRow,
    name_w: usize,
    detail: bool,
    glyph: bool,
    ellipsis: &str,
) -> (String, Token) {
    let s = &r.suggestion;
    let others = [&s.bearer, &s.treatment]
        .iter()
        .filter(|c| c.is_some())
        .count();
    let Some(p) = s.project.as_ref() else {
        if others == 0 {
            return ("· none".into(), Token::Muted);
        }
        let mut t = if detail {
            format!("no project +{others}")
        } else {
            "no project".to_string()
        };
        if glyph {
            t = format!("{} {t}", r.glyph.char());
        }
        if r.stale {
            return (format!("stale {t}"), Token::Warn);
        }
        return (t, Token::Muted);
    };
    let name = fit_str(&project_name(p), name_w, ellipsis);
    let mut t = if detail && others > 0 {
        format!("{name} +{others}")
    } else {
        name
    };
    if glyph {
        t = format!("{} {t}", r.glyph.char());
    }
    if r.stale {
        return (format!("stale {t}"), Token::Warn);
    }
    let tok = match (r.glyph, p.value.as_str()) {
        (_, "none") => Token::Muted,
        (Glyph::Agent, _) => Token::Info,
        _ => Token::Text,
    };
    (t, tok)
}

/// The conf cell: `.84` (`▰▰▱ .84` with the bar), `✓` confirmed, `stale` (`!` when narrow), or `—`.
pub fn conf_cell(r: &QueueRow, bar: bool, ascii: bool) -> (String, Token) {
    if r.stale {
        return (if bar { "stale" } else { "!" }.into(), Token::Warn);
    }
    match r.confidence {
        Some(c) if bar => (
            format!("{} {}", conf_bar(c, ascii), conf_text(c)),
            Token::Muted,
        ),
        Some(c) => (conf_text(c), Token::Muted),
        // confirmed: ✓ by a person (quiet), A by an agent (the agent colour, spec §5a)
        None if r.glyph == Glyph::Agent => ("A".into(), Token::Info),
        None if r.glyph == Glyph::Human => ("✓".into(), Token::Muted),
        None => ("—".into(), Token::Muted),
    }
}

/// One row as `ui state` exports it: every column, valued in full (the screen may cut them).
pub fn row_cells(rows: &[QueueRow], ascii: bool) -> Vec<Vec<Cell>> {
    rows.iter()
        .map(|r| {
            let (conf, tok) = conf_cell(r, false, ascii);
            let (sugg, stok) = suggestion_cell(r, usize::MAX / 4, true, false, "…");
            vec![
                Cell::new(r.org.clone(), Token::Text),
                Cell::new(r.date.clone(), Token::Muted),
                Cell::new(payee_text(r), Token::Text),
                Cell::new(ownpurse_core::money::fmt(r.amount), Token::Text),
                Cell::new(r.account.clone(), Token::Muted),
                Cell::new(sugg, stok),
                Cell::new(conf, tok),
            ]
        })
        .collect()
}

/// The list's column widths at one width (ruling 1). Separators are one space; the first cell is the
/// cursor gutter.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ListCols {
    pub org: usize,
    /// 10 = `2025-03-04`, 5 = `03-04`.
    pub date: usize,
    pub payee: usize,
    pub amount: usize,
    /// 0 = hidden.
    pub account: usize,
    pub sugg: usize,
    /// The `+n` after the project name.
    pub detail: bool,
    pub glyph: bool,
    pub bar: bool,
    pub conf: usize,
}

impl ListCols {
    fn total(&self) -> usize {
        let cols = [
            self.org,
            self.date,
            self.payee,
            self.amount,
            self.account,
            self.sugg,
            self.conf,
        ];
        let shown = cols.iter().filter(|w| **w > 0).count();
        1 + cols.iter().sum::<usize>() + shown.saturating_sub(1)
    }
}

/// The columns for a list `w` cells wide in a frame `frame_w` wide; `one_year`: every listed line is in
/// one year, so the 100–139 list may show `MM-DD` (else the full date, payee trimmed toward 14).
/// - from 140: the full set (date, account, bar), the rest to payee and suggestion;
/// - 100–139: `org 6 · date 5 · payee ≥ 20 · amount 11 · suggestion ≥ 15 · conf 4`;
/// - below 100 (stacked, full width): the full set, degrading bar → account → year in date → suggestion
///   detail; payee never below 14, amount never below 11.
pub fn list_cols(w: usize, frame_w: u16, amount_w: usize, one_year: bool) -> ListCols {
    let amount = amount_w.max(11);
    let full = ListCols {
        org: 6,
        date: 10,
        payee: 14,
        amount,
        account: 4,
        sugg: 15,
        detail: true,
        glyph: true,
        bar: true,
        conf: 7,
    };
    // several years listed: the full date, its 5 extra cells taken from payee (never below 14)
    let (mid_date, mid_payee) = if one_year { (5, 20) } else { (10, 15) };
    let mid = ListCols {
        date: mid_date,
        payee: mid_payee,
        account: 0,
        glyph: false,
        bar: false,
        conf: 4,
        ..full
    };
    let mut c = if frame_w >= WIDE {
        full
    } else if frame_w >= SIDE_BY_SIDE {
        mid
    } else {
        // the degrade order below 100
        let steps = [
            ListCols {
                bar: false,
                glyph: false,
                conf: 4,
                ..full
            },
            ListCols {
                bar: false,
                glyph: false,
                conf: 4,
                account: 0,
                ..full
            },
            ListCols {
                bar: false,
                glyph: false,
                conf: 4,
                account: 0,
                date: 5,
                ..full
            },
            ListCols {
                bar: false,
                glyph: false,
                conf: 4,
                account: 0,
                date: 5,
                detail: false,
                sugg: 10,
                ..full
            },
        ];
        std::iter::once(full)
            .chain(steps)
            .find(|c| c.total() <= w)
            .unwrap_or(steps[3])
    };
    // spare cells: payee first (up to 28), then the suggestion
    // keep one clear cell before the divider when there is room to spare
    let spare = w.saturating_sub(c.total()).saturating_sub(1);
    let to_payee = spare.min(28usize.saturating_sub(c.payee));
    c.payee += to_payee;
    c.sugg += spare - to_payee;
    c
}

/// Footer status: nothing in the Queue (the task keys use the row; ruling 4); elsewhere the mode.
pub fn status(app: &App) -> String {
    let scope = app.scope_label(&app.ui.scope);
    let mode = app.ui.classify.mode;
    if app.classify.is_none() {
        return format!("{scope} · classify · not set up");
    }
    if mode == Mode::Groups {
        return crate::views::groups::status(app);
    }
    if mode == Mode::Catalog {
        let c = &app.ui.classify.catalog;
        return format!("{scope} · catalog · {}", c.list.label().to_lowercase());
    }
    if mode != Mode::Queue {
        return format!("{scope} · {} · coming in this release", mode.word());
    }
    String::new()
}

pub fn crumbs(app: &App) -> Vec<String> {
    if app.ui.classify.mode == Mode::Groups && app.classify.is_some() {
        return crate::views::groups::crumbs(app);
    }
    vec![
        app.scope_label(&app.ui.scope),
        "Classify".into(),
        app.ui.classify.mode.label().into(),
    ]
}
fn sub_bar(app: &App, buf: &mut Buffer, area: Rect, hits: &mut Hits) {
    let th = &app.ui.theme;
    let sep = th.text(" · ");
    let open = app.cl_counts().map(|c| c.open);
    let mut x = area.x + 1;
    for (i, m) in Mode::ALL.iter().enumerate() {
        if i > 0 {
            x = put(
                buf,
                x,
                area.y,
                vec![Span::styled(sep.clone(), th.fg(Token::Muted))],
                area.right().saturating_sub(x),
                th.glyphs().ellipsis,
            );
        }
        let label = match (m, open) {
            (Mode::Queue, Some(n)) => format!("Queue {}", crate::clock::int(n as i64)),
            _ => m.label().to_string(),
        };
        let active = *m == app.ui.classify.mode;
        let style = if active {
            th.fg(Token::Accent)
                .add_modifier(crate::theme_tokens::styles::HEADING)
        } else {
            th.fg(Token::Text)
        };
        let x0 = x;
        x = put(
            buf,
            x,
            area.y,
            vec![Span::styled(label, style)],
            area.right().saturating_sub(x),
            th.glyphs().ellipsis,
        );
        hits.add(
            Rect::new(x0, area.y, x.saturating_sub(x0), 1),
            Target::Action(match m {
                Mode::Queue => "classify.queue",
                Mode::Groups => "classify.groups",
                Mode::Audition => "classify.audition",
                Mode::Tree => "classify.tree",
                Mode::Catalog => "classify.catalog",
            }),
        );
    }
}

/// The filter strip: the facets in effect, or the typed filter with its count, or why it doesn't parse.
pub fn strip_text(app: &App) -> (String, Token) {
    let rows = app.cl_rows();
    let q = &app.ui.classify.filter;
    if let Some(e) = &rows.error {
        return (format!("filter: \"{q}\" · {e}"), Token::Warn);
    }
    if q.trim().is_empty() {
        let org = match app.ui.scope {
            crate::app::Scope::All => "all".to_string(),
            ref s => app.scope_label(s),
        };
        return (
            format!("org: {org} · account: any · month: any · project: any · status: open"),
            Token::Muted,
        );
    }
    (
        crate::filter::chip(q, rows.rows.len(), rows.total),
        Token::Muted,
    )
}

/// The detail pane's lines for the selected row. Side by side: facts, labels, the project keys (`p1 …`
/// one per line), why. Stacked (below 100; ruling 5): the labels first, then one facts line, then account
/// and description as rows remain; the project keys only while `p` is pressed.
pub fn detail_lines(app: &App, w: usize, stacked: bool) -> Vec<Vec<Span<'static>>> {
    let th = &app.ui.theme;
    let Some(d) = app.cl_detail() else {
        return vec![];
    };
    let ell = th.glyphs().ellipsis;
    let sep = || Span::styled(th.text(" · "), th.fg(Token::Muted));
    let l = &d.line;
    let header = vec![
        Span::styled(
            format!("{} {}", cap(l.doc_type.label()), short_id(&l.doc_id)),
            th.heading(),
        ),
        sep(),
        Span::styled(l.org.clone(), th.fg(Token::Text)),
    ];
    let amount = ownpurse_core::money::fmt(l.amount);
    let payee = if l.payee.trim().is_empty() {
        l.description.clone()
    } else {
        l.payee.clone()
    };
    // the label rows: title, then the status column (fixed width), then the name, which alone is cut
    let title_w = 10;
    let mut labels: Vec<Vec<Span<'static>>> = Vec::new();
    for lv in &d.labels {
        let title = match lv.label_type.as_str() {
            "should_be_account" => "Should-be".to_string(),
            _ => lv.title.clone(),
        };
        let (status, tok, name) = match (&lv.label, &lv.suggestion) {
            (Some(lab), _) if lab.stale => ("stale".to_string(), Token::Warn, lab.name.clone()),
            (Some(lab), _) => match &lab.source {
                ownpurse_core::classify::Source::Agent(_) => {
                    ("A ✓".to_string(), Token::Info, lab.name.clone())
                }
                s => (
                    format!("{} ✓", s.glyph().char()),
                    Token::Muted,
                    lab.name.clone(),
                ),
            },
            (None, Some(s)) => (
                format!("{} {}", s.source.glyph().char(), conf_text(s.confidence)),
                Token::Muted,
                s.name.clone(),
            ),
            (None, None) => (String::new(), Token::Muted, "—".to_string()),
        };
        let status = th.text(&status);
        let padn = 6usize.saturating_sub(width(&status));
        labels.push(vec![
            Span::styled(format!("{title:<title_w$} "), th.fg(Token::Muted)),
            Span::styled(format!("{status}{} ", " ".repeat(padn)), th.fg(tok)),
            Span::styled(
                name,
                th.fg(if lv.label.is_some() || lv.suggestion.is_some() {
                    Token::Text
                } else {
                    Token::Muted
                }),
            ),
        ]);
    }
    // the stale label's old fingerprint against the line now (V19)
    let mut stale: Vec<Vec<Span<'static>>> = Vec::new();
    if let Some(lab) = d
        .labels
        .iter()
        .filter_map(|l| l.label.as_ref())
        .find(|l| l.stale)
    {
        for (field, then, now) in &lab.changed {
            let fmt = |v: &str| {
                if *field == "amount" {
                    ownpurse_core::money::parse(v)
                        .map(ownpurse_core::money::fmt)
                        .unwrap_or_else(|_| v.to_string())
                } else {
                    v.to_string()
                }
            };
            let text = th.text(&format!(
                "line changed: {field} {} → {}",
                fmt(then),
                fmt(now)
            ));
            for t in wrap(&text, w).into_iter().take(2) {
                stale.push(vec![Span::styled(t, th.fg(Token::Warn))]);
            }
        }
    }
    // `p`+digit (ruling 3): one per line, cut, never wrapped
    let keys: Vec<Vec<Span<'static>>> = app
        .cl_projects()
        .iter()
        .take(9)
        .enumerate()
        .map(|(i, (_, n))| {
            vec![
                Span::styled(format!("p{} ", i + 1), th.fg(Token::Text)),
                Span::styled(fit_str(n, w.saturating_sub(3), ell), th.fg(Token::Muted)),
            ]
        })
        .collect();
    let account = vec![Span::styled(
        format!("{} {}", l.account_code, l.account_name),
        th.fg(Token::Text),
    )];
    let mut why: Vec<Vec<Span<'static>>> = Vec::new();
    if let Some(s) = d.labels.iter().find_map(|l| {
        l.suggestion
            .as_ref()
            .filter(|_| l.label.is_none())
            .and_then(|s| s.rule.clone().map(|r| (r, s.evidence.clone())))
    }) {
        let ev =
            s.1.iter()
                .map(|e| format!("{} \"{}\"", e.condition, e.matched))
                .collect::<Vec<_>>()
                .join(", ");
        why.push(vec![
            Span::styled("Rule      ".to_string(), th.fg(Token::Muted)),
            Span::styled(s.0, th.fg(Token::Text)),
            sep(),
            Span::styled(ev, th.fg(Token::Muted)),
        ]);
    } else if let Some(r) = d.rules_fired.first() {
        why.push(vec![
            Span::styled("Rule      ".to_string(), th.fg(Token::Muted)),
            Span::styled(r.clone(), th.fg(Token::Text)),
        ]);
    }
    for g in &d.groups {
        why.push(vec![
            Span::styled("Group     ".to_string(), th.fg(Token::Muted)),
            Span::styled(g.title.clone(), th.fg(Token::Text)),
            sep(),
            Span::styled(
                format!("{} {}", g.kind, g.role.clone().unwrap_or_default()),
                th.fg(Token::Muted),
            ),
        ]);
    }
    for n in &d.notes {
        let head = format!(
            "Note      {} {}: ",
            n.at.get(..10).unwrap_or(&n.at),
            n.actor
        );
        let text = format!("{head}{}", n.text.replace('\n', " / "));
        for (k, t) in wrap(&text, w).into_iter().take(3).enumerate() {
            why.push(vec![Span::styled(
                t,
                th.fg(if k == 0 { Token::Muted } else { Token::Text }),
            )]);
        }
    }
    let description: Vec<Vec<Span<'static>>> = wrap(&l.description, w)
        .into_iter()
        .take(2)
        .map(|t| vec![Span::styled(t, th.fg(Token::Muted))])
        .collect();
    let mut out = vec![header];
    if stacked {
        out.extend(labels.into_iter().take(3));
        out.extend(stale);
        out.push(vec![
            Span::styled(l.date.clone(), th.fg(Token::Muted)),
            sep(),
            Span::styled(payee, th.fg(Token::Text)),
            sep(),
            Span::styled(amount, th.fg(Token::Text)),
        ]);
        if app.ui.classify.pending_project {
            out.extend(keys);
        }
        out.push(account);
        out.extend(description);
        out.extend(why);
    } else {
        out.push(vec![
            Span::styled(l.date.clone(), th.fg(Token::Muted)),
            sep(),
            Span::styled(payee, th.fg(Token::Text)),
        ]);
        out.extend(description);
        let mut facts = vec![Span::styled(amount, th.fg(Token::Text)), sep()];
        facts.extend(account);
        out.push(facts);
        if let Some(rec) = l.reconciled {
            out.push(vec![Span::styled(
                th.text(if rec {
                    "reconciled ✓"
                } else {
                    "not reconciled"
                }),
                th.fg(Token::Muted),
            )]);
        }
        out.push(vec![]);
        out.extend(labels);
        out.extend(stale);
        out.push(vec![]);
        out.extend(keys);
        out.push(vec![]);
        out.extend(why);
    }
    out
}

fn cap(s: &str) -> String {
    let mut c = s.chars();
    c.next()
        .map(|f| f.to_uppercase().collect::<String>() + c.as_str())
        .unwrap_or_default()
}

/// A document id as the pane shows it (UUIDs keep their first 8).
fn short_id(id: &str) -> String {
    if id.len() > 12 && id.contains('-') {
        id.chars().take(8).collect()
    } else {
        id.to_string()
    }
}

fn text_line(th: &Theme, buf: &mut Buffer, area: Rect, y: u16, spans: Vec<Span<'static>>) {
    // the UI's glyphs fold to ASCII in the ASCII set (data text has none of them)
    let spans: Vec<Span<'static>> = spans
        .into_iter()
        .map(|s| Span::styled(th.text(&s.content), s.style))
        .collect();
    if y < area.bottom() {
        put(
            buf,
            area.x + 1,
            y,
            spans,
            area.width.saturating_sub(2),
            th.glyphs().ellipsis,
        );
    }
}

pub fn draw(app: &App, buf: &mut Buffer, area: Rect, hits: &mut Hits) {
    let th = &app.ui.theme;
    if area.height < 3 {
        return;
    }
    sub_bar(app, buf, Rect::new(area.x, area.y, area.width, 1), hits);
    let body = Rect::new(area.x, area.y + 1, area.width, area.height - 1);
    if app.classify.is_none() {
        crate::views::placeholder(
            app,
            buf,
            body,
            "Classification isn't set up: add classify.toml beside ownpurse.json (check it with `ownpurse classify rules check`).",
        );
        return;
    }
    if app.ui.classify.mode == Mode::Catalog {
        crate::views::catalog::draw(app, buf, body, hits);
        return;
    }
    if app.ui.classify.mode == Mode::Groups {
        crate::views::groups::draw(app, buf, body, hits);
        return;
    }
    if app.ui.classify.mode != Mode::Queue {
        crate::views::placeholder(
            app,
            buf,
            body,
            &format!(
                "{}: coming in this release. The Queue is ready now (, and . switch modes).",
                app.ui.classify.mode.label()
            ),
        );
        return;
    }
    if app.ui.classify.bulk.is_some() {
        draw_bulk(app, buf, body, hits);
        return;
    }
    let (strip, tok) = strip_text(app);
    let sh = crate::widgets::strip::height(&strip, body.width).min(2);
    crate::widgets::strip::draw(
        th,
        buf,
        Rect::new(body.x, body.y, body.width, sh),
        &strip,
        tok,
    );
    let main = Rect::new(
        body.x,
        body.y + sh,
        body.width,
        body.height.saturating_sub(sh),
    );
    if main.height < 3 {
        return;
    }
    let side = area.width >= SIDE_BY_SIDE;
    let (list, detail) = if side {
        // 67 of 100 (ruling 1): the list, the divider, the detail
        let lw = area.width * 67 / 100;
        (
            Rect::new(main.x, main.y, lw, main.height),
            Rect::new(main.x + lw + 1, main.y, main.width - lw - 1, main.height),
        )
    } else {
        let lh = (main.height * 11 / 20).max(3);
        (
            Rect::new(main.x, main.y, main.width, lh),
            Rect::new(
                main.x,
                main.y + lh + 1,
                main.width,
                main.height.saturating_sub(lh + 1),
            ),
        )
    };
    draw_list(app, buf, list, hits);
    let line = th.fg(Token::Line);
    let g = th.glyphs();
    if side {
        for y in main.top()..main.bottom() {
            buf[(list.right(), y)].set_symbol(g.vsep).set_style(line);
        }
    } else if list.bottom() < main.bottom() {
        for x in main.left()..main.right() {
            buf[(x, list.bottom())].set_symbol(g.rule).set_style(line);
        }
    }
    let w = detail.width.saturating_sub(2) as usize;
    for (k, l) in detail_lines(app, w, !side).into_iter().enumerate() {
        text_line(th, buf, detail, detail.y + k as u16, l);
    }
    if app.ui.classify.picker.is_some() {
        draw_picker(app, buf, area, hits);
    }
    if app.ui.classify.note.is_some() {
        draw_note(app, buf, area, hits);
    }
}

/// The list (virtualised: only the window's rows are formatted; the amount width is the widest amount).
fn draw_list(app: &App, buf: &mut Buffer, area: Rect, hits: &mut Hits) {
    let th = &app.ui.theme;
    let g = th.glyphs();
    hits.content = Some(area);
    let rows = app.cl_rows().rows;
    let amount_w = app.cl_amount_width();
    let c = list_cols(
        area.width as usize,
        app.ui.size.0,
        amount_w,
        app.cl_rows().one_year,
    );
    let muted = th.fg(Token::Muted);
    // header: the titles over their columns
    let titles: Vec<(usize, &str, bool)> = vec![
        (c.org, "org", false),
        (c.date, "date", false),
        (c.payee, "payee", false),
        (c.amount, "amount", true),
        (c.account, "acct", false),
        (c.sugg, "suggestion", false),
        (c.conf, "conf", false),
    ];
    let row_spans = |cells: Vec<(String, Token, bool)>, base: ratatui::style::Style| {
        let mut spans: Vec<Span<'static>> = Vec::new();
        let mut first = true;
        for ((w, _, _), (text, tok, right)) in titles.iter().zip(cells) {
            if *w == 0 {
                continue;
            }
            if !first {
                spans.push(Span::styled(" ".to_string(), base));
            }
            first = false;
            spans.push(Span::styled(
                pad(&th.text(&text), *w, g.ellipsis, right),
                th.fg(tok).patch(base),
            ));
        }
        spans
    };
    let head: Vec<(String, Token, bool)> = titles
        .iter()
        .map(|(_, t, r)| (t.to_string(), Token::Muted, *r))
        .collect();
    put(
        buf,
        area.x + 1,
        area.y,
        row_spans(head, muted),
        area.width.saturating_sub(1),
        g.ellipsis,
    );
    if rows.is_empty() {
        text_line(
            th,
            buf,
            area,
            area.y + 2,
            vec![Span::styled(
                "Nothing to classify in this filter.".to_string(),
                th.fg(Token::Muted),
            )],
        );
        return;
    }
    let body = (area.height as usize).saturating_sub(1);
    let top = app.cl_top(body);
    let cur = app.cl_cursor();
    // the project name leaves room for ` +n` (and the glyph when shown)
    let name_w = c
        .sugg
        .saturating_sub(3 + if c.glyph { 2 } else { 0 })
        .max(4);
    for (k, r) in rows.iter().skip(top).take(body).enumerate() {
        let y = area.y + 1 + k as u16;
        let selected = cur == Some(top + k);
        let base = if selected {
            th.selection()
        } else {
            ratatui::style::Style::default()
        };
        if selected {
            for x in area.left()..area.right() {
                buf[(x, y)].set_style(th.selection());
            }
            buf.set_span(
                area.x,
                y,
                &Span::styled(g.cursor, th.fg(Token::Accent).patch(base)),
                1,
            );
        }
        let (sugg, stok) = suggestion_cell(r, name_w, c.detail, c.glyph, g.ellipsis);
        let (conf, ctok) = conf_cell(r, c.bar, th.ascii);
        let date = if c.date >= 10 {
            r.date.clone()
        } else {
            r.date.get(5..).unwrap_or("").to_string()
        };
        let cells = vec![
            (r.org.clone(), Token::Text, false),
            (date, Token::Muted, false),
            (payee_text(r), Token::Text, false),
            (ownpurse_core::money::fmt(r.amount), Token::Text, true),
            (r.account.clone(), Token::Muted, false),
            (sugg, stok, false),
            (conf, ctok, false),
        ];
        put(
            buf,
            area.x + 1,
            y,
            row_spans(cells, base),
            area.width.saturating_sub(1),
            g.ellipsis,
        );
        if selected {
            th.band_row(buf, Rect::new(area.x, y, area.width, 1));
        }
        hits.add(Rect::new(area.x, y, area.width, 1), Target::Row(k));
    }
}
fn draw_picker(app: &App, buf: &mut Buffer, area: Rect, _hits: &mut Hits) {
    let th = &app.ui.theme;
    let Some(p) = &app.ui.classify.picker else {
        return;
    };
    let rows = app.cl_picker_rows();
    let r = app.cl_selected();
    let title = match p.label_type.as_str() {
        "should_be_account" => "Should-be account".to_string(),
        "group" => "Add to group".to_string(),
        "g_role" => "Role".to_string(),
        "g_status" => "Status".to_string(),
        "g_member" => "Add member".to_string(),
        "group_kind" => "New group: kind".to_string(),
        "bearer" => "Bearer".to_string(),
        "treatment" => "Treatment".to_string(),
        t => t.to_string(),
    };
    let title = match (&r, app.ui.classify.mode == Mode::Groups) {
        (_, true) => match app.grp_selected() {
            Some(g) => format!("{title} · {}", g.title),
            None => title,
        },
        (Some(r), false) => format!(
            "{title} · {} {}",
            r.payee,
            ownpurse_core::money::fmt(r.amount)
        ),
        (None, false) => title,
    };
    // adding a member searches every line: a wider box
    let w = if p.label_type == crate::classify::groups::PICK_MEMBER {
        80
    } else {
        52
    }
    .min(area.width.saturating_sub(4));
    let h = (rows.len().max(1) as u16 + 4).min(area.height.saturating_sub(1));
    let rect = centred(area, w, h, Some(2));
    let inner = crate::widgets::overlay_box_clear(buf, rect, &title, th);
    if inner.height < 3 {
        return;
    }
    let hint = if p.label_type == crate::classify::groups::PICK_MEMBER {
        "same amount ±7 days in another org first · type an org, date, amount or payee"
    } else {
        "type to filter"
    };
    crate::widgets::palette::draw_query(th, buf, inner, &p.query, hint, app.ui.caret);
    let raised = th.raised();
    let current = r.as_ref().and_then(|r| match p.label_type.as_str() {
        "bearer" => r
            .suggestion
            .bearer
            .as_ref()
            .filter(|c| c.confirmed)
            .map(|c| c.value.clone()),
        "treatment" => r
            .suggestion
            .treatment
            .as_ref()
            .filter(|c| c.confirmed)
            .map(|c| c.value.clone()),
        _ => None,
    });
    let list = Rect::new(inner.x, inner.y + 2, inner.width, inner.height - 2);
    if rows.is_empty() {
        buf.set_span(
            list.x + 2,
            list.y,
            &Span::styled(
                if p.label_type == crate::classify::groups::PICK_MEMBER && p.query.is_empty() {
                    "No line of the same amount within 7 days; type to search."
                } else {
                    "Nothing matches."
                },
                th.fg(Token::Muted).patch(raised),
            ),
            list.width.saturating_sub(2),
        );
        return;
    }
    let sel = p.sel.min(rows.len() - 1);
    let first = sel.saturating_sub(list.height.saturating_sub(1) as usize);
    for (k, (i, (id, name))) in rows
        .iter()
        .enumerate()
        .skip(first)
        .take(list.height as usize)
        .enumerate()
    {
        let y = list.y + k as u16;
        let on = i == sel;
        let base = if on { th.selection() } else { raised };
        if on {
            fill(buf, Rect::new(list.x, y, list.width, 1), base);
            buf.set_span(
                list.x,
                y,
                &Span::styled(th.glyphs().cursor, th.fg(Token::Accent).patch(base)),
                1,
            );
        }
        // groups: `title · kind · N lines` only (the id is in the detail)
        let mut spans = if matches!(
            p.label_type.as_str(),
            "group" | "group_kind" | "g_role" | "g_status" | "g_member"
        ) {
            vec![Span::styled(name.clone(), th.fg(Token::Text).patch(base))]
        } else {
            vec![
                Span::styled(format!("{id:<12} "), th.fg(Token::Muted).patch(base)),
                Span::styled(name.clone(), th.fg(Token::Text).patch(base)),
            ]
        };
        if current.as_deref() == Some(id.as_str()) {
            spans.push(Span::styled(
                th.text("  ● current"),
                th.fg(Token::Muted).patch(base),
            ));
        }
        put(
            buf,
            list.x + 2,
            y,
            spans,
            list.width.saturating_sub(3),
            th.glyphs().ellipsis,
        );
    }
}

/// The note editor: one line (Enter saves), or multi-line after ⌃E (Enter breaks the line, ⌃S saves).
fn draw_note(app: &App, buf: &mut Buffer, area: Rect, _hits: &mut Hits) {
    let th = &app.ui.theme;
    let Some(n) = &app.ui.classify.note else {
        return;
    };
    let r = app.cl_selected();
    let title = match (&r, app.ui.classify.mode == Mode::Groups) {
        (_, true) => app
            .grp_selected()
            .map_or_else(|| "Note".into(), |g| format!("Note · {}", g.title)),
        (Some(r), false) => format!("Note · {} {}", r.payee, ownpurse_core::money::fmt(r.amount)),
        (None, false) => "Note".into(),
    };
    let w = 64.min(area.width.saturating_sub(4));
    let inner_w = w.saturating_sub(4) as usize;
    let lines: Vec<String> = if n.multi {
        n.text
            .split('\n')
            .flat_map(|l| {
                let w = wrap(l, inner_w);
                if w.is_empty() { vec![String::new()] } else { w }
            })
            .collect()
    } else {
        vec![n.text.clone()]
    };
    let body_h = if n.multi {
        (lines.len() as u16).clamp(4, 8)
    } else {
        1
    };
    let rect = centred(area, w, body_h + 4, Some(3));
    let inner = crate::widgets::overlay_box_clear(buf, rect, &title, th);
    if inner.height < 2 {
        return;
    }
    let raised = th.raised();
    let hint = if n.multi {
        "multi-line · Enter new line · ⌃S save"
    } else {
        "one line · ⌃E multi-line"
    };
    buf.set_span(
        inner.x + 1,
        inner.y,
        &Span::styled(th.text(hint), th.fg(Token::Muted).patch(raised)),
        inner.width.saturating_sub(2),
    );
    let first = lines.len().saturating_sub(body_h as usize);
    let shown: Vec<&String> = lines.iter().skip(first).collect();
    let last = shown.len().saturating_sub(1);
    for (k, l) in shown.iter().enumerate() {
        let y = inner.y + 2 + k as u16;
        if y >= inner.bottom() {
            break;
        }
        let t = if n.multi {
            (*l).clone()
        } else {
            // one line: the end of the text stays visible
            let room = inner_w.saturating_sub(1);
            let cs: Vec<char> = l.chars().collect();
            let mut s: String = String::new();
            let mut wsum = 0;
            for c in cs.iter().rev() {
                let cw = width(&c.to_string());
                if wsum + cw > room {
                    break;
                }
                wsum += cw;
                s.insert(0, *c);
            }
            s
        };
        let tw = width(&t) as u16;
        buf.set_span(
            inner.x + 1,
            y,
            &Span::styled(t.clone(), th.fg(Token::Text).patch(raised)),
            tw,
        );
        if k == last {
            crate::widgets::caret(buf, inner.x + 1, y, &t, None, raised, tw);
        }
    }
}

/// The bulk-accept preview (`A`): every line in the filter whose suggestions are all ≥ the threshold, by
/// project and bearer, the total; the confirm bar with Cancel focused.
fn draw_bulk(app: &App, buf: &mut Buffer, area: Rect, hits: &mut Hits) {
    let th = &app.ui.theme;
    let Some(b) = &app.ui.classify.bulk else {
        return;
    };
    let Some(p) = app.cl_bulk_preview() else {
        return;
    };
    let cfg = app.classify.as_ref().map(|d| d.provider.config());
    let name =
        |ty: &str, v: &str| cfg.map_or_else(|| v.to_string(), |c| c.choice_name(ty, v).to_string());
    let thr = conf_text(crate::classify::hundredths(b.threshold));
    let n = p.anchors.len();
    let mut y = area.y;
    let line =
        |buf: &mut Buffer, y: u16, spans: Vec<Span<'static>>| text_line(th, buf, area, y, spans);
    line(
        buf,
        y,
        vec![
            Span::styled("Bulk accept".to_string(), th.heading()),
            Span::styled(
                th.text(&format!(
                    " · every line in this filter whose suggestions are all ≥ {thr} (- / + change it)"
                )),
                th.fg(Token::Muted),
            ),
        ],
    );
    y += 2;
    line(
        buf,
        y,
        vec![
            Span::styled(
                format!(
                    "{} line{}",
                    crate::clock::int(n as i64),
                    if n == 1 { "" } else { "s" }
                ),
                th.fg(Token::Text)
                    .add_modifier(crate::theme_tokens::styles::HEADING),
            ),
            Span::styled(th.text(" · total "), th.fg(Token::Muted)),
            Span::styled(ownpurse_core::money::fmt(p.total), th.fg(Token::Text)),
        ],
    );
    y += 2;
    let col_w = (area.width.saturating_sub(4) / 2) as usize;
    let pw = col_w.saturating_sub(8);
    let block = |title: &str, ty: &str, items: &[(String, usize)]| -> Vec<String> {
        let mut v = vec![title.to_string()];
        for (id, k) in items {
            let label = fit_str(&name(ty, id), pw, th.glyphs().ellipsis);
            v.push(format!("{label:<pw$} {:>5}", crate::clock::int(*k as i64)));
        }
        v
    };
    let left = block("By project", "project", &p.by_project);
    let right = block("By bearer", "bearer", &p.by_bearer);
    let footer_y = area.bottom().saturating_sub(1);
    for k in 0..left.len().max(right.len()) {
        if y + 2 >= footer_y {
            break;
        }
        let tok = |k: usize| {
            if k == 0 {
                th.fg(Token::Muted)
            } else {
                th.fg(Token::Text)
            }
        };
        if let Some(t) = left.get(k) {
            buf.set_span(
                area.x + 1,
                y,
                &Span::styled(t.clone(), tok(k)),
                col_w as u16,
            );
        }
        if let Some(t) = right.get(k) {
            buf.set_span(
                area.x + 3 + col_w as u16,
                y,
                &Span::styled(t.clone(), tok(k)),
                col_w as u16,
            );
        }
        y += 1;
    }
    if n == 0 {
        line(
            buf,
            y + 1,
            vec![Span::styled(
                th.text(&format!("Nothing in this filter is at or above {thr}.")),
                th.fg(Token::Muted),
            )],
        );
    }
    if footer_y > y {
        line(
            buf,
            footer_y.saturating_sub(1),
            vec![Span::styled(
                "Never included: stale labels, lines below the threshold, lines already labelled."
                    .to_string(),
                th.fg(Token::Muted),
            )],
        );
    }
    // the confirm bar: Cancel focused (V3), the action names the count
    let bar = Rect::new(area.x, footer_y, area.width, 1);
    fill(buf, bar, th.raised());
    let q = th.text(&format!(
        "Accept {n} line{} at ≥ {thr}?",
        if n == 1 { "" } else { "s" }
    ));
    buf.set_span(
        bar.x + 1,
        bar.y,
        &Span::styled(q, th.fg(Token::Text).patch(th.raised())),
        bar.width / 2,
    );
    let labels = [
        "Cancel".to_string(),
        format!("Accept {n} line{}", if n == 1 { "" } else { "s" }),
    ];
    let texts: Vec<String> = labels
        .iter()
        .enumerate()
        .map(|(i, l)| {
            if i == b.focus {
                format!("{}{l} ", th.glyphs().cursor)
            } else {
                format!(" {l} ")
            }
        })
        .collect();
    let total: u16 = texts.iter().map(|t| width(t) as u16 + 1).sum();
    let mut x = bar.right().saturating_sub(total + 1);
    for (i, t) in texts.iter().enumerate() {
        let style = if i == b.focus {
            th.focused_button()
        } else {
            th.fg(Token::Text).patch(th.raised())
        };
        let w = width(t) as u16;
        buf.set_span(x, bar.y, &Span::styled(t.clone(), style), w);
        hits.add(Rect::new(x, bar.y, w, 1), Target::Button(i));
        x += w + 1;
    }
}

/// The pickers and the note editor over other Classify modes (Groups).
pub fn draw_picker_pub(app: &App, buf: &mut Buffer, area: Rect, hits: &mut Hits) {
    draw_picker(app, buf, area, hits);
}

pub fn draw_note_pub(app: &App, buf: &mut Buffer, area: Rect, hits: &mut Hits) {
    draw_note(app, buf, area, hits);
}
