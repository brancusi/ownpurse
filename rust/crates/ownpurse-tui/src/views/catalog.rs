//! Classify › Catalog (spec classify §3.D): the list switcher (`Projects · Rules`), the list with its Used
//! column, a toast line (archive instead of delete, V20), the diff band (`3 changes to classify.toml · w write
//! · u discard`), the project and rule editors (the rule editor with its live preview), and the `w` diff with
//! its confirm (Cancel focused). Draw never mutates.

use crate::app::App;
use crate::classify::catalog::{CatList, RuleForm};
use crate::hit::{Hits, Target};
use crate::layout::{ColSpec, Kind, fit_str, width};
use crate::theme::Theme;
use crate::theme_tokens::Token;
use crate::widgets::table::{self, Cell, Table};
use crate::widgets::{centred, fill, put};
use ownpurse_core::classify::config::{Cond, Rule, SetValue};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::text::Span;

pub const PROJECT_COLS: [ColSpec; 5] = [
    ColSpec {
        title: "name",
        short_title: "",
        kind: Kind::Text { min: 12 },
        priority: 9,
    },
    ColSpec {
        title: "id",
        short_title: "",
        kind: Kind::Id,
        priority: 6,
    },
    ColSpec {
        title: "aliases",
        short_title: "",
        kind: Kind::Text { min: 8 },
        priority: 3,
    },
    ColSpec {
        title: "used",
        short_title: "",
        kind: Kind::Amount,
        priority: 9,
    },
    ColSpec {
        title: "status",
        short_title: "",
        kind: Kind::Status,
        priority: 8,
    },
];

/// The Rules list's columns (`ui state` names) and design's widths at 100 columns:
/// `prio 4 · id 20 · when 28 · sets 30 · matches 7 · labelled 8`, then the `archived` chip (when and sets
/// give way when narrower; the id never does).
pub const RULE_WIDTHS: [usize; 7] = [4, 20, 28, 30, 7, 8, 8];

pub const RULE_COLS: [ColSpec; 6] = [
    ColSpec {
        title: "prio",
        short_title: "",
        kind: Kind::Amount,
        priority: 7,
    },
    ColSpec {
        title: "id",
        short_title: "",
        kind: Kind::Text { min: 10 },
        priority: 9,
    },
    ColSpec {
        title: "when",
        short_title: "",
        kind: Kind::Text { min: 10 },
        priority: 5,
    },
    ColSpec {
        title: "sets",
        short_title: "",
        kind: Kind::Text { min: 8 },
        priority: 6,
    },
    ColSpec {
        title: "matches",
        short_title: "",
        kind: Kind::Amount,
        priority: 9,
    },
    ColSpec {
        title: "labelled",
        short_title: "",
        kind: Kind::Amount,
        priority: 8,
    },
];

/// The Rules list with design's fixed widths; `when` and `sets` shrink together when the list is narrower.
fn draw_rules(
    app: &App,
    buf: &mut Buffer,
    area: Rect,
    top: usize,
    cur: Option<usize>,
    hits: &mut Hits,
) {
    let th = &app.ui.theme;
    let g = th.glyphs();
    let mut w = RULE_WIDTHS;
    let need: usize = w.iter().sum::<usize>() + w.len() - 1 + 1;
    let mut over = need.saturating_sub(area.width.saturating_sub(1) as usize);
    for i in [3, 2] {
        let floor = 10;
        let cut = over.min(w[i].saturating_sub(floor));
        w[i] -= cut;
        over -= cut;
    }
    let right = [true, false, false, false, true, true, false];
    let row = |cells: [(String, Token); 7], base: ratatui::style::Style| -> Vec<Span<'static>> {
        let mut v = Vec::new();
        for (k, (t, tok)) in cells.into_iter().enumerate() {
            if k > 0 {
                v.push(Span::styled(" ".to_string(), base));
            }
            v.push(Span::styled(
                crate::layout::pad(&th.text(&t), w[k], g.ellipsis, right[k]),
                th.fg(tok).patch(base),
            ));
        }
        v
    };
    let head = ["prio", "id", "when", "sets", "matches", "labelled", ""]
        .map(|t| (t.to_string(), Token::Muted));
    put(
        buf,
        area.x + 1,
        area.y,
        row(head, ratatui::style::Style::default()),
        area.width.saturating_sub(1),
        g.ellipsis,
    );
    let rules = app.cat_rules();
    let body = (area.height as usize).saturating_sub(1);
    for (k, (r, matches, labelled)) in rules.into_iter().skip(top).take(body).enumerate() {
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
        let tok = if r.archived {
            Token::Muted
        } else {
            Token::Text
        };
        let cells = [
            (r.priority.to_string(), Token::Muted),
            (r.id.clone(), tok),
            (when_text(&r), Token::Muted),
            (sets_text(&r), tok),
            (crate::clock::int(matches as i64), Token::Text),
            (crate::clock::int(labelled as i64), Token::Muted),
            // the status chip at the end (design): ids are never cut for it
            (
                if r.archived { "archived" } else { "" }.to_string(),
                Token::Muted,
            ),
        ];
        put(
            buf,
            area.x + 1,
            y,
            row(cells, base),
            area.width.saturating_sub(1),
            g.ellipsis,
        );
        if selected {
            th.band_row(buf, Rect::new(area.x, y, area.width, 1));
        }
        hits.add(Rect::new(area.x, y, area.width, 1), Target::Row(k));
    }
}

/// A rule's conditions in one line: `payee "Hollowpine…" · alias of project`.
pub fn when_text(r: &Rule) -> String {
    r.when
        .iter()
        .map(|c| match c {
            Cond::DescriptionHasAlias(t) => format!("alias of {t}"),
            Cond::Payee(p) => format!("payee \"{p}\""),
            Cond::PayeeHas(v) => format!("payee has {}", v.join("|")),
            Cond::DescriptionHas(v) => format!("description has {}", v.join("|")),
            Cond::AccountIn(v) => format!("account {}", v.join(",")),
            Cond::OrgIn(v) => format!("org {}", v.join(",")),
            Cond::AmountBetween(a, b) => format!("amount {a}..{b}"),
            Cond::DateBetween(a, b) => format!("date {a}..{b}"),
            Cond::Sign(s) => format!("sign {s:?}").to_lowercase(),
            Cond::PayeeIsOrg(o) => format!("payee is org {o}"),
            Cond::BankTransfer(true) => "bank transfer".into(),
            Cond::BankTransfer(false) => "not a bank transfer".into(),
        })
        .collect::<Vec<_>>()
        .join(" · ")
}

/// What a rule sets: `project=$matched bearer=roast`.
pub fn sets_text(r: &Rule) -> String {
    let mut v: Vec<String> = r
        .set
        .iter()
        .map(|(k, s)| match s {
            SetValue::Choice(c) => format!("{k}={c}"),
            SetValue::Matched => format!("{k}=$matched"),
        })
        .collect();
    if let Some(g) = &r.group_kind {
        v.push(format!("group={g}"));
    }
    v.join(" ")
}

/// The diff band's text, or None with nothing staged.
pub fn band_text(app: &App) -> Option<String> {
    let n = app.ui.classify.catalog.staged.len();
    (n > 0).then(|| {
        format!(
            "{n} change{} to classify.toml · w write · u discard",
            if n == 1 { "" } else { "s" }
        )
    })
}

fn line(th: &Theme, buf: &mut Buffer, area: Rect, y: u16, spans: Vec<Span<'static>>) {
    if y < area.bottom() {
        let spans = spans
            .into_iter()
            .map(|s| Span::styled(th.text(&s.content), s.style))
            .collect();
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
    let c = &app.ui.classify.catalog;
    if area.height < 4 {
        return;
    }
    if c.diff.is_some() {
        draw_diff(app, buf, area, hits);
        return;
    }
    // the list switcher
    let sep = th.text(" · ");
    let mut x = area.x + 1;
    for (i, l) in [CatList::Projects, CatList::Rules].iter().enumerate() {
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
        let active = *l == c.list;
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
            vec![Span::styled(l.label().to_string(), style)],
            area.right().saturating_sub(x),
            th.glyphs().ellipsis,
        );
        hits.add(
            Rect::new(x0, area.y, x - x0, 1),
            Target::Action(if active {
                "drill.back"
            } else if *l == CatList::Rules {
                "catalog.list_next"
            } else {
                "catalog.list_prev"
            }),
        );
    }
    put(
        buf,
        x,
        area.y,
        vec![Span::styled(th.text("   ← → list"), th.fg(Token::Muted))],
        area.right().saturating_sub(x),
        th.glyphs().ellipsis,
    );
    // the bottom: toast, then the diff band
    let band = band_text(app);
    let bottom = u16::from(band.is_some()) + u16::from(c.toast.is_some());
    let t_area = Rect::new(
        area.x,
        area.y + 2,
        area.width,
        area.height.saturating_sub(2 + bottom),
    );
    hits.content = Some(t_area);
    let cur = app.cat_cursor();
    let h = (t_area.height as usize).saturating_sub(1);
    let top = cur.map_or(0, |c| c.saturating_sub(h.saturating_sub(1)));
    match c.list {
        CatList::Projects => {
            let rows: Vec<Vec<Cell>> = app
                .cat_projects()
                .into_iter()
                .map(|(id, name, aliases, used, archived)| {
                    vec![
                        Cell::new(name, if archived { Token::Muted } else { Token::Text }),
                        Cell::new(id, Token::Muted),
                        Cell::new(aliases, Token::Muted),
                        Cell::new(crate::clock::int(used as i64), Token::Text),
                        Cell::new(if archived { "archived" } else { "active" }, Token::Muted),
                    ]
                })
                .collect();
            let t = Table {
                cols: &PROJECT_COLS,
                rows: &rows,
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
        }
        CatList::Rules => draw_rules(app, buf, t_area, top, cur, hits),
    }
    let mut y = t_area.bottom();
    if let Some(t) = &c.toast {
        line(
            th,
            buf,
            area,
            y,
            vec![Span::styled(t.clone(), th.fg(Token::Warn))],
        );
        y += 1;
    }
    if let Some(b) = band {
        let r = Rect::new(area.x, y, area.width, 1);
        fill(buf, r, th.raised());
        line(
            th,
            buf,
            area,
            y,
            vec![Span::styled(b, th.fg(Token::Text).patch(th.raised()))],
        );
        hits.add(r, Target::Action("catalog.apply"));
    }
    if c.editor.is_some() {
        draw_editor(app, buf, area, hits);
    }
}

fn draw_editor(app: &App, buf: &mut Buffer, area: Rect, _hits: &mut Hits) {
    let th = &app.ui.theme;
    let Some(ed) = &app.ui.classify.catalog.editor else {
        return;
    };
    let rule = ed.kind == "rule";
    let title = match (&ed.editing, rule) {
        (Some(id), true) => format!("Rule {id}"),
        (None, true) => "New rule".to_string(),
        (Some(id), false) => format!("Project {id}"),
        (None, false) => "New project".to_string(),
    };
    let h = (ed.fields.len() as u16 + 5).min(area.height);
    // the rule editor spans the content width (its 1-column margins reach the screen edges)
    let rect = if rule {
        Rect::new(area.x + 1, area.y, area.width.saturating_sub(2), h)
    } else {
        centred(area, area.width.saturating_sub(4).min(64), h, Some(0))
    };
    let inner = crate::widgets::overlay_box_clear(buf, rect, &title, th);
    if inner.height < 2 {
        return;
    }
    let raised = th.raised();
    // fields on the left (the whole box for a project), the preview on the right (rules)
    let left_w = if rule && inner.width >= 70 {
        inner.width * 9 / 20
    } else {
        inner.width
    };
    let label_w = 16usize;
    for (k, (name, value)) in ed.fields.iter().enumerate() {
        let y = inner.y + k as u16;
        if y >= inner.bottom().saturating_sub(2) {
            break;
        }
        let focused = k == ed.focus;
        let base = if focused { th.selection() } else { raised };
        if focused {
            fill(buf, Rect::new(inner.x, y, left_w, 1), base);
            buf.set_span(
                inner.x,
                y,
                &Span::styled(th.glyphs().cursor, th.fg(Token::Accent).patch(base)),
                1,
            );
        }
        let label = format!("{name:<label_w$}");
        buf.set_span(
            inner.x + 2,
            y,
            &Span::styled(label, th.fg(Token::Muted).patch(base)),
            label_w as u16,
        );
        let vx = inner.x + 2 + label_w as u16 + 1;
        let room = left_w.saturating_sub(label_w as u16 + 4) as usize;
        let shown = if value.is_empty() && !focused {
            "—".to_string()
        } else {
            fit_str(value, room.saturating_sub(1), th.glyphs().ellipsis)
        };
        let tw = width(&shown) as u16;
        buf.set_span(
            vx,
            y,
            &Span::styled(shown.clone(), th.fg(Token::Text).patch(base)),
            tw,
        );
        if focused {
            crate::widgets::caret(buf, vx, y, &shown, app.ui.caret, base, tw);
        }
    }
    let foot_y = inner.bottom().saturating_sub(1);
    let foot = match &ed.error {
        Some(e) => Span::styled(th.text(e), th.fg(Token::Error).patch(raised)),
        None => Span::styled(
            th.text(if rule {
                "comma lists: payee has, description has, account in, org in · $matched sets the alias's project"
            } else {
                "aliases: a comma list (whole words, any case)"
            }),
            th.fg(Token::Muted).patch(raised),
        ),
    };
    put(
        buf,
        inner.x + 1,
        foot_y,
        vec![foot],
        inner.width.saturating_sub(2),
        th.glyphs().ellipsis,
    );
    if !rule || left_w == inner.width {
        return;
    }
    // the live preview: would match N lines, 5 samples (V: rules only, no Jev)
    let px = inner.x + left_w + 1;
    for y in inner.y..foot_y {
        buf[(px - 1, y)]
            .set_symbol(th.glyphs().vsep)
            .set_style(th.fg(Token::Line).patch(raised));
    }
    let pw = inner.right().saturating_sub(px + 1);
    let put_at = |buf: &mut Buffer, y: u16, spans: Vec<Span<'static>>| {
        put(buf, px + 1, y, spans, pw, th.glyphs().ellipsis);
    };
    match app.cat_preview() {
        Some(Ok(m)) => {
            let n = m.count;
            put_at(
                buf,
                inner.y,
                vec![
                    Span::styled(th.text("would match "), th.fg(Token::Muted).patch(raised)),
                    Span::styled(
                        format!(
                            "{} line{}",
                            crate::clock::int(n as i64),
                            if n == 1 { "" } else { "s" }
                        ),
                        th.heading().patch(raised),
                    ),
                ],
            );
            // design: `11 lines · 3 already labelled by a person (kept)`
            put_at(
                buf,
                inner.y + 1,
                vec![Span::styled(
                    th.text(&format!(
                        "{} line{} · {} already labelled by a person (kept)",
                        crate::clock::int(n as i64),
                        if n == 1 { "" } else { "s" },
                        crate::clock::int(m.kept as i64)
                    )),
                    th.fg(Token::Muted).patch(raised),
                )],
            );
            let d = app.classify.as_ref();
            let lines: Vec<_> = m
                .sample
                .iter()
                .take(5)
                .filter_map(|a| d.and_then(|d| d.provider.detail(a)).map(|x| x.line))
                .collect();
            // MM-DD when every sample is in one year (the Queue's rule)
            let one_year = lines
                .windows(2)
                .all(|w| w[0].date.get(..4) == w[1].date.get(..4));
            for (k, l) in lines.into_iter().enumerate() {
                let date = if one_year {
                    l.date.get(5..).unwrap_or(&l.date).to_string()
                } else {
                    l.date.clone()
                };
                put_at(
                    buf,
                    inner.y + 3 + k as u16,
                    vec![
                        Span::styled(format!("{date} "), th.fg(Token::Muted).patch(raised)),
                        Span::styled(
                            format!("{:>11} ", ownpurse_core::money::fmt(l.amount)),
                            th.fg(Token::Text).patch(raised),
                        ),
                        Span::styled(
                            if l.payee.is_empty() {
                                l.description
                            } else {
                                l.payee
                            },
                            th.fg(Token::Text).patch(raised),
                        ),
                    ],
                );
            }
        }
        Some(Err(why)) => put_at(
            buf,
            inner.y,
            vec![Span::styled(
                th.text(&format!("not a rule yet: {why}")),
                th.fg(Token::Muted).patch(raised),
            )],
        ),
        None => {}
    }
    let f = RuleForm::from_fields(&ed.fields, ed.editing.clone());
    let kept: Vec<String> = ed
        .editing
        .as_deref()
        .and_then(|id| {
            app.classify
                .as_ref()
                .and_then(|d| d.provider.config().rule(id))
        })
        .map(|r| {
            r.when
                .iter()
                .filter(|c| {
                    matches!(
                        c,
                        Cond::AmountBetween(..)
                            | Cond::DateBetween(..)
                            | Cond::Sign(_)
                            | Cond::PayeeIsOrg(_)
                            | Cond::BankTransfer(_)
                    )
                })
                .map(|c| {
                    when_text(&Rule {
                        when: vec![c.clone()],
                        ..r.clone()
                    })
                })
                .collect()
        })
        .unwrap_or_default();
    let _ = f;
    if !kept.is_empty() && foot_y > inner.y + 8 {
        put_at(
            buf,
            foot_y - 1,
            vec![Span::styled(
                th.text(&format!("kept as is: {}", kept.join(" · "))),
                th.fg(Token::Muted).patch(raised),
            )],
        );
    }
}

/// The `w` diff: the unified diff of classify.toml, what it does, and the confirm (Cancel focused).
fn draw_diff(app: &App, buf: &mut Buffer, area: Rect, hits: &mut Hits) {
    let th = &app.ui.theme;
    let Some(d) = &app.ui.classify.catalog.diff else {
        return;
    };
    let staged = app.cat_staged();
    let (diff, summary, archived): (Vec<String>, Vec<String>, Vec<String>) = match &staged {
        Some(Ok(s)) => (
            s.diff.lines().map(str::to_string).collect(),
            s.summary.clone(),
            s.archived_instead
                .iter()
                .map(|(i, n)| format!("{i:?} is used by {n}: archived instead"))
                .collect(),
        ),
        Some(Err(e)) => (vec![], vec![format!("can't be written: {e}")], vec![]),
        None => (vec![], vec![], vec![]),
    };
    line(
        th,
        buf,
        area,
        area.y,
        vec![
            Span::styled("classify.toml".to_string(), th.heading()),
            Span::styled(
                th.text(&format!(" · {}", summary.join("; "))),
                th.fg(Token::Muted),
            ),
        ],
    );
    let mut y = area.y + 1;
    for a in &archived {
        line(
            th,
            buf,
            area,
            y,
            vec![Span::styled(a.clone(), th.fg(Token::Warn))],
        );
        y += 1;
    }
    let bar_y = area.bottom().saturating_sub(1);
    let body = bar_y.saturating_sub(y + 1) as usize;
    let top = d.top.min(diff.len().saturating_sub(body));
    for (k, l) in diff.iter().skip(top).take(body).enumerate() {
        let tok = if l.starts_with("+++") || l.starts_with("---") || l.starts_with("@@") {
            Token::Muted
        } else if l.starts_with('+') {
            Token::Ok
        } else if l.starts_with('-') {
            Token::Error
        } else {
            Token::Text
        };
        line(
            th,
            buf,
            area,
            y + 1 + k as u16,
            vec![Span::styled(l.clone(), th.fg(tok))],
        );
    }
    // the confirm bar
    let bar = Rect::new(area.x, bar_y, area.width, 1);
    fill(buf, bar, th.raised());
    let n = app.ui.classify.catalog.staged.len();
    let q = format!(
        "Write {n} change{} to classify.toml?",
        if n == 1 { "" } else { "s" }
    );
    buf.set_span(
        bar.x + 1,
        bar.y,
        &Span::styled(q, th.fg(Token::Text).patch(th.raised())),
        bar.width / 2,
    );
    let labels = ["Cancel", "Write classify.toml"];
    let texts: Vec<String> = labels
        .iter()
        .enumerate()
        .map(|(i, l)| {
            if i == d.focus {
                format!("{}{l} ", th.glyphs().cursor)
            } else {
                format!(" {l} ")
            }
        })
        .collect();
    let total: u16 = texts.iter().map(|t| width(t) as u16 + 1).sum();
    let mut x = bar.right().saturating_sub(total + 1);
    for (i, t) in texts.iter().enumerate() {
        let style = if i == d.focus {
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
