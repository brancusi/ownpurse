//! Classify › Groups (spec classify §3.E): the group list (left; stacked above below 100 columns) and the
//! detail: members in origin → mirror → settlement order, the chain check, a settlement group's running
//! balance and its pair rule on the record's balances, notes. A member's document opens over the detail
//! (Ledger L3's view; Esc returns). Colour means attention: a gap and a failed pair check are warn; ✓ is
//! quiet. Draw never mutates.

use crate::app::App;
use crate::classify::groups::GFocus;
use crate::hit::{Hits, Target};
use crate::layout::{fit_str, pad, width};
use crate::theme::Theme;
use crate::theme_tokens::Token;
use crate::widgets::put;
use ownpurse_core::classify::groups::ChainCheck;
use ownpurse_core::money::fmt;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::text::Span;

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

fn status_token(s: &str) -> Token {
    match s {
        "gap" | "accountant" => Token::Warn,
        _ => Token::Muted,
    }
}

pub fn crumbs(app: &App) -> Vec<String> {
    let mut v = vec![
        app.scope_label(&app.ui.scope),
        "Classify".into(),
        "Groups".into(),
    ];
    if let Some(g) = app.grp_selected() {
        v.push(g.title);
        if let Some((org, _, id)) = &app.ui.classify.groups.doc {
            v.push(format!("{org} {id}"));
        }
    }
    v
}

pub fn status(app: &App) -> String {
    let n = app.grp_rows().len();
    format!(
        "{} · groups · {n} group{}",
        app.scope_label(&app.ui.scope),
        if n == 1 { "" } else { "s" }
    )
}

pub fn draw(app: &App, buf: &mut Buffer, area: Rect, hits: &mut Hits) {
    let th = &app.ui.theme;
    // a member's document, over the detail
    if app.ui.classify.groups.doc.is_some() {
        match app.cl_doc.as_deref() {
            Some(Ok(d)) => crate::views::ledger::draw_l3(app, d, buf, area, hits),
            Some(Err(e)) => crate::views::placeholder(
                app,
                buf,
                area,
                &format!("Could not open the document: {e}"),
            ),
            None => {
                crate::views::placeholder(app, buf, area, "That document is not in the record.")
            }
        }
        return;
    }
    let rows = app.grp_rows();
    if rows.is_empty() {
        crate::views::placeholder(
            app,
            buf,
            area,
            "No groups yet. In the Queue, G adds a line to a new group or one nearby.",
        );
        return;
    }
    let side = area.width >= crate::views::classify::SIDE_BY_SIDE;
    let (list, detail) = if side {
        let lw = (area.width * 36 / 100).max(28);
        (
            Rect::new(area.x, area.y + 1, lw, area.height.saturating_sub(1)),
            Rect::new(area.x + lw + 1, area.y, area.width - lw - 1, area.height),
        )
    } else {
        let lh = (rows.len() as u16 + 1).min(area.height / 3).max(3);
        (
            Rect::new(area.x, area.y + 1, area.width, lh),
            Rect::new(
                area.x,
                area.y + lh + 2,
                area.width,
                area.height.saturating_sub(lh + 2),
            ),
        )
    };
    hits.content = Some(list);
    let focus = app.ui.classify.groups.focus;
    line(
        th,
        buf,
        Rect::new(list.x, area.y, list.width, 1),
        area.y,
        vec![Span::styled(
            th.text("Groups ▸ members"),
            th.fg(Token::Muted),
        )],
    );
    // the list: title · kind, then members and status at the right
    let cur = app.grp_cursor();
    let h = list.height as usize;
    let top = cur.map_or(0, |c| c.saturating_sub(h.saturating_sub(1)));
    for (k, r) in rows.iter().enumerate().skip(top).take(h) {
        let y = list.y + (k - top) as u16;
        let selected = cur == Some(k);
        let base = if selected && focus == GFocus::List {
            th.selection()
        } else {
            ratatui::style::Style::default()
        };
        if selected {
            for x in list.left()..list.right() {
                buf[(x, y)].set_style(base);
            }
            buf.set_span(
                list.x,
                y,
                &Span::styled(th.glyphs().cursor, th.fg(Token::Accent).patch(base)),
                1,
            );
        }
        let right = format!(" {} · {}", r.members, r.status);
        let rw = width(&th.text(&right));
        let room = (list.width as usize).saturating_sub(2 + rw);
        let title = fit_str(&r.title, room, th.glyphs().ellipsis);
        let kind = format!(" · {}", r.kind.replace('_', " "));
        let kind = fit_str(
            &kind,
            room.saturating_sub(width(&title)),
            th.glyphs().ellipsis,
        );
        put(
            buf,
            list.x + 1,
            y,
            vec![
                Span::styled(title.clone(), th.fg(Token::Text).patch(base)),
                Span::styled(th.text(&kind), th.fg(Token::Muted).patch(base)),
            ],
            room as u16,
            th.glyphs().ellipsis,
        );
        put(
            buf,
            list.right().saturating_sub(rw as u16),
            y,
            vec![
                Span::styled(
                    format!(" {} · ", r.members),
                    th.fg(Token::Muted).patch(base),
                ),
                Span::styled(r.status.clone(), th.fg(status_token(&r.status)).patch(base)),
            ],
            rw as u16,
            th.glyphs().ellipsis,
        );
        if selected && focus == GFocus::List {
            th.band_row(buf, Rect::new(list.x, y, list.width, 1));
        }
        hits.add(Rect::new(list.x, y, list.width, 1), Target::Row(k - top));
    }
    // the divider
    let g = th.glyphs();
    if side {
        for y in area.top()..area.bottom() {
            buf[(list.right(), y)]
                .set_symbol(g.vsep)
                .set_style(th.fg(Token::Line));
        }
    } else {
        let y = list.bottom();
        if y < area.bottom() {
            for x in area.left()..area.right() {
                buf[(x, y)].set_symbol(g.rule).set_style(th.fg(Token::Line));
            }
        }
    }
    draw_detail(app, buf, detail);
    if app.ui.classify.picker.is_some() {
        crate::views::classify::draw_picker_pub(app, buf, area, hits);
    }
    if app.ui.classify.note.is_some() {
        crate::views::classify::draw_note_pub(app, buf, area, hits);
    }
}

fn draw_detail(app: &App, buf: &mut Buffer, area: Rect) {
    let th = &app.ui.theme;
    let Some(d) = app.grp_detail() else { return };
    let sep = || Span::styled(th.text(" · "), th.fg(Token::Muted));
    let mut y = area.y;
    let g = &d.group;
    line(
        th,
        buf,
        area,
        y,
        vec![
            Span::styled(g.title.clone(), th.heading()),
            sep(),
            Span::styled(g.kind.replace('_', " "), th.fg(Token::Muted)),
            sep(),
            Span::styled(g.status.clone(), th.fg(status_token(&g.status))),
            sep(),
            Span::styled(g.id.clone(), th.fg(Token::Muted)),
        ],
    );
    y += 2;
    // members: role · org · date · account · amount · description
    let w = area.width.saturating_sub(2) as usize;
    let widths = [10usize, 6, 10, 5, 12];
    let fixed: usize = widths.iter().sum::<usize>() + widths.len();
    let desc_w = w.saturating_sub(fixed + 1).max(6);
    let ell = th.glyphs().ellipsis;
    let head = format!(
        "{} {} {} {} {} {}",
        pad("role", widths[0], ell, false),
        pad("org", widths[1], ell, false),
        pad("date", widths[2], ell, false),
        pad("acct", widths[3], ell, false),
        pad("amount", widths[4], ell, true),
        "description"
    );
    line(
        th,
        buf,
        area,
        y,
        vec![Span::styled(head, th.fg(Token::Muted))],
    );
    y += 1;
    let mcur = app.grp_member_cursor(&d);
    let focus = app.ui.classify.groups.focus == GFocus::Members;
    for (k, m) in d.members.iter().enumerate() {
        if y + 3 >= area.bottom() {
            break;
        }
        let on = focus && mcur == Some(k);
        let base = if on {
            th.selection()
        } else {
            ratatui::style::Style::default()
        };
        if on {
            for x in area.left()..area.right() {
                buf[(x, y)].set_style(base);
            }
            buf.set_span(
                area.x,
                y,
                &Span::styled(th.glyphs().cursor, th.fg(Token::Accent).patch(base)),
                1,
            );
        }
        let role = m.role.clone().unwrap_or_else(|| "—".into());
        let desc = if m.missing {
            "not in the record any more".to_string()
        } else if m.description.is_empty() {
            m.payee.clone()
        } else {
            m.description.clone()
        };
        line(
            th,
            buf,
            area,
            y,
            vec![
                Span::styled(
                    format!("{} ", pad(&role, widths[0], ell, false)),
                    th.fg(Token::Muted).patch(base),
                ),
                Span::styled(
                    format!("{} ", pad(&m.org, widths[1], ell, false)),
                    th.fg(Token::Text).patch(base),
                ),
                Span::styled(
                    format!("{} ", pad(&m.date, widths[2], ell, false)),
                    th.fg(Token::Muted).patch(base),
                ),
                Span::styled(
                    format!("{} ", pad(&m.account, widths[3], ell, false)),
                    th.fg(Token::Muted).patch(base),
                ),
                Span::styled(
                    format!("{} ", pad(&fmt(m.amount), widths[4], ell, true)),
                    th.fg(Token::Text).patch(base),
                ),
                Span::styled(
                    fit_str(&desc, desc_w, ell),
                    th.fg(if m.missing { Token::Warn } else { Token::Text })
                        .patch(base),
                ),
            ],
        );
        if on {
            th.band_row(buf, Rect::new(area.x, y, area.width, 1));
        }
        y += 1;
    }
    y += 1;
    // the chain check (many-to-one never claims a line-level match, §10.9)
    let tok = match &d.check {
        ChainCheck::OneToOne { .. } | ChainCheck::NotApplicable => Token::Muted,
        ChainCheck::Gap { .. } => Token::Warn,
        ChainCheck::ManyToOne { .. } => Token::Text,
    };
    // the check: the core's primary line and, for many-to-one, its share line; wrapped, never cut
    let label = "Check  ";
    let room = (area.width as usize).saturating_sub(2 + label.len());
    let (primary, second) = d.check.text();
    // one phrase at every width, as the CLI reads it (the core's text): `… → 1 mirror (x)` over
    // `explains N% of the mirror · no per-line match`
    // wrapped, never cut
    let mut rows: Vec<String> = crate::layout::wrap(&th.text(&primary), room);
    if let Some(s2) = second {
        rows.extend(crate::layout::wrap(&th.text(&s2), room));
    }
    // never cut: the primary line and the share line, each wrapped
    for (k, r) in rows.iter().enumerate() {
        line(
            th,
            buf,
            area,
            y,
            vec![
                Span::styled(
                    if k == 0 {
                        label.to_string()
                    } else {
                        " ".repeat(label.len())
                    },
                    th.fg(Token::Muted),
                ),
                Span::styled(r.clone(), th.fg(tok)),
            ],
        );
        y += 1;
    }
    // a settlement group: the running balance, and its pair rule on the record's balances
    if !d.running.is_empty() && y + 2 < area.bottom() {
        y += 1;
        line(
            th,
            buf,
            area,
            y,
            vec![Span::styled("Running balance".to_string(), th.heading())],
        );
        y += 1;
        // one row per settlement (its legs paired by the core): the core's text, wrapped, never cut
        let room = (area.width as usize).saturating_sub(2);
        for st in &d.running {
            for w in crate::layout::wrap(&th.text(&st.text()), room) {
                if y >= area.bottom() {
                    break;
                }
                let tok = if st.lone_org.is_some() {
                    Token::Warn
                } else {
                    Token::Text
                };
                line(th, buf, area, y, vec![Span::styled(w, th.fg(tok))]);
                y += 1;
            }
        }
    }
    if let Some(p) = app.grp_pair_check()
        && y + 2 < area.bottom()
    {
        y += 1;
        line(
            th,
            buf,
            area,
            y,
            vec![
                Span::styled("Pair rule  ".to_string(), th.fg(Token::Muted)),
                Span::styled(p.title.clone(), th.fg(Token::Text)),
            ],
        );
        y += 1;
        for r in &p.rows {
            if y >= area.bottom() {
                break;
            }
            // `= 0.00 ✓`, `≠ 3,281.19: in transit 07-14 → 07-15` (calm: ok), or a real gap (warn).
            // The label without its anchor; the text after the amounts, else on its own wrapped line.
            let what: String = r
                .label
                .split(' ')
                .filter(|w| !w.contains(':'))
                .collect::<Vec<_>>()
                .join(" ");
            let head = format!("{} {:>11} {:>12}  ", r.date, fmt(r.a), fmt(r.b));
            let full = (area.width as usize).saturating_sub(2);
            let text = th.text(&r.text());
            let tok = if r.ok { Token::Muted } else { Token::Warn };
            let mut out: Vec<Vec<Span<'static>>> = Vec::new();
            if width(&head) + width(&text) <= full && what.len() <= 6 {
                out.push(vec![
                    Span::styled(head.clone(), th.fg(Token::Muted)),
                    Span::styled(text.clone(), th.fg(tok)),
                ]);
            } else {
                // the reason below says what it is (`in transit …`): the label isn't repeated
                let label = if text.contains(&what) {
                    String::new()
                } else {
                    what.clone()
                };
                out.push(vec![
                    Span::styled(head.clone(), th.fg(Token::Muted)),
                    Span::styled(label, th.fg(Token::Muted)),
                ]);
                for w in crate::layout::wrap(&text, full.saturating_sub(2)) {
                    out.push(vec![Span::styled(format!("  {w}"), th.fg(tok))]);
                }
            }
            for l in out {
                if y >= area.bottom() {
                    break;
                }
                line(th, buf, area, y, l);
                y += 1;
            }
        }
    }
    for n in &d.notes {
        if y + 1 >= area.bottom() {
            break;
        }
        y += 1;
        line(
            th,
            buf,
            area,
            y,
            vec![
                Span::styled(
                    format!("Note  {} {}: ", n.at.get(..10).unwrap_or(&n.at), n.actor),
                    th.fg(Token::Muted),
                ),
                Span::styled(n.text.replace('\n', " / "), th.fg(Token::Text)),
            ],
        );
    }
}
