//! Compare L1 / L2 (spec §9; acceptance H, I). One column per org (debit/credit sub-fields, the org centred
//! over both, a faint `│` between orgs; net at medium widths; fewer orgs with "showing n of m" when even net
//! doesn't fit), `≠` on code-merged rows whose names differ, `── only in <org> ──` sections, `—` where the
//! account doesn't exist, blank for zero, and a per-org totals row only. No difference column, no cross-org
//! total (V9). L2: one row's postings on one date spine, per-org opening and closing; no pairing inferred.

use crate::app::App;
use crate::clock::plural;
use crate::compare::CmpModel;
use crate::hit::{Hits, Target};
use crate::ic_nav::{dec, s};
use crate::layout::{fit_str, pad, width};
use crate::theme::Mode;
use crate::theme_tokens::Token;
use crate::widgets::strip;
use ownpurse_core::{dr_cr, fmt};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::Span;
use serde_json::Value;

/// (orgs shown, net only, note) for `n` orgs at `w` columns, amounts `amount_w` wide:
/// 140+: up to 4 with debit/credit; 100–139: net with 4; below 100: 2, net; then tightened so amounts
/// never truncate: net first, then fewer orgs, and say so (H9).
pub fn layout(n: usize, w: u16, amount_w: usize) -> (usize, bool, String) {
    let w = w as usize;
    let (mut shown, mut net) = if w >= 140 {
        (n.min(4), false)
    } else if w >= 100 {
        (n.min(4), n == 4)
    } else {
        (n.min(2), true)
    };
    // gutter + code + account (at least 16) + per-org blocks, exactly as drawn
    let fixed = 2 + 6 + 2 + 16;
    let cw = amount_w;
    let fits = |k: usize, as_net: bool| fixed + k * (2 + if as_net { cw } else { 2 * cw + 2 }) <= w;
    if !net && !fits(shown, false) {
        net = true;
    }
    while shown > 2 && !fits(shown, true) {
        shown -= 1;
    }
    let note = if shown < n {
        format!("widen the terminal to compare more orgs (showing {shown} of {n})")
    } else if w < 100 {
        "widen the terminal to compare more orgs".to_string()
    } else {
        String::new()
    };
    (shown, net, note)
}

/// The amount sub-field width: the widest amount across rows (at least 10), so `(987,654.32)` decides it,
/// and room for each org's total with its ` ✓`.
pub fn amount_width(m: &CmpModel) -> usize {
    let mut w = 10;
    for r in &m.rows {
        if let Some(cells) = r["cells"].as_object() {
            for c in cells.values() {
                for k in ["debit", "credit", "net"] {
                    if let Some(v) = dec(&c[k]) {
                        w = w.max(width(&fmt(v.abs())));
                    }
                }
            }
        }
    }
    for c in &m.columns {
        for k in ["debit", "credit"] {
            if let Some(v) = dec(&c[k]) {
                w = w.max(width(&fmt(v)) + if k == "credit" { 2 } else { 0 });
            }
        }
    }
    w
}

fn current(app: &App, w: u16) -> Option<(&CmpModel, usize, bool, String, usize)> {
    let c = app.ui.cmp.as_ref()?;
    let n = c.orgs.len();
    let full = app.cmp_model(n)?;
    let aw = amount_width(full);
    let (k, net, note) = layout(n, w, aw);
    let m = app.cmp_model(k).unwrap_or(full);
    Some((m, k, net, note, aw))
}

pub fn crumbs(app: &App) -> Vec<String> {
    let Some(c) = app.ui.cmp.as_ref() else {
        return vec![];
    };
    if c.level == 2
        && let (Some(row), Some(sp)) = (app.cmp_l2_row(), app.cmp_l2())
    {
        return vec![
            "Ledger".into(),
            "compare".into(),
            format!(
                "{} · {} → {}",
                s(&row["label"]),
                s(&sp["since"]),
                s(&sp["until"])
            ),
        ];
    }
    vec![
        "Ledger".into(),
        format!("compare trial balances at {} (accrual)", app.ui.as_of),
    ]
}

pub fn status(app: &App) -> String {
    let Some(c) = app.ui.cmp.as_ref() else {
        return String::new();
    };
    if c.level == 2 {
        let n = app
            .cmp_l2()
            .and_then(|v| v["spine"].as_array().map(Vec::len))
            .unwrap_or(0) as i64;
        return format!("compare · {}", plural(n, "row", "rows"));
    }
    let rows = app
        .cmp_model(c.orgs.len())
        .map_or(0, |m| app.cmp_visible(m).len()) as i64;
    format!(
        "compare · {} · align {} · {}",
        plural(c.orgs.len() as i64, "org", "orgs"),
        c.align,
        plural(rows, "row", "rows")
    )
}

/// The strip lines (H2, H5, H6, H9): basis per org, the note, the selected row's names, the map note.
pub fn strip_lines(app: &App, w: u16) -> Vec<(String, Token)> {
    let mut v = level_strip(app, w);
    if let Some(c) = app.filter_chip() {
        v.push((c, Token::Info));
    }
    v
}

fn level_strip(app: &App, w: u16) -> Vec<(String, Token)> {
    let Some((m, _k, _net, note, _)) = current(app, w) else {
        return vec![];
    };
    let c = app.ui.cmp.as_ref().expect("compare open");
    let mut v = Vec::new();
    if c.level == 2 {
        v.push((
            format!(
                "postings side by side · no pairing inferred{}",
                if note.is_empty() {
                    String::new()
                } else {
                    format!(" · {note}")
                }
            ),
            Token::Text,
        ));
        let marked = app.cmp_matched_shown();
        v.push((
            if c.show_matched {
                format!(
                    "m: highlighting existing Intercompany matches · {} marked ≡ (none inferred here) · f sets the range for every column · Esc back",
                    marked
                )
            } else {
                "m marks postings the Intercompany engine matched · f sets the range for every column · Esc back".into()
            },
            Token::Muted,
        ));
        return v;
    }
    // (org, source, state, record age) per shown column
    let facts: Vec<(String, String, String, String)> = m
        .columns
        .iter()
        .map(|col| {
            let o = s(&col["org"]);
            let age = app
                .model
                .orgs
                .iter()
                .position(|x| x.alias == o)
                .map(|i| app.model.freshness(&[i], app.now).age_token())
                .unwrap_or_default();
            (
                o,
                s(&col["basis"]["source"]),
                s(&col["basis"]["state"]),
                age,
            )
        })
        .collect();
    let narrow = w < 100;
    // The basis, collapsed at every width: `basis: all ✓ TB 2024-12-31 + 2025 · record 2h` when
    // every shown org agrees; otherwise orgs are listed, with their source and age only when they differ
    // from the common ones.
    let basis = {
        let first = facts.first().cloned();
        let all_same = first.as_ref().is_some_and(|f| {
            facts
                .iter()
                .all(|x| (&x.1, &x.2, &x.3) == (&f.1, &f.2, &f.3))
        });
        match first {
            Some((_, src, st, age)) if all_same && facts.len() > 1 => {
                format!("basis: all {st} {src} · record {age}")
            }
            Some((_, src0, _, age0)) => {
                let parts: Vec<String> = facts
                    .iter()
                    .map(|(o, src, st, age)| {
                        if (src, age) == (&src0, &age0) {
                            format!("{o} {st}")
                        } else {
                            format!("{o} {st} ({src} · record {age})")
                        }
                    })
                    .collect();
                format!("basis: {} ({src0} · record {age0})", parts.join(" · "))
            }
            None => String::new(),
        }
    };
    let align = if narrow {
        format!("align: {} (a: code ⇄ map)", c.align)
    } else {
        format!("align: {} (a toggles: code ⇄ map)", c.align)
    };
    // design pass 2: one line, `align: code` left and the basis state right (`✓ all TB+25`); the full basis
    // is a detail (`i`)
    let short_state = {
        let st: Vec<String> = m
            .columns
            .iter()
            .map(|col| {
                let b = &col["basis"];
                let short = if b["kind"] == "xero-opening" {
                    format!("TB+{}", app.ui.as_of.get(2..4).unwrap_or(""))
                } else {
                    "history".into()
                };
                format!("{} {short}", s(&b["state"]))
            })
            .collect();
        match st.first() {
            Some(f) if st.iter().all(|x| x == f) => {
                let (state, short) = f.rsplit_once(' ').unwrap_or((f.as_str(), ""));
                if state == "✓" {
                    format!("✓ all {short}")
                } else {
                    format!("all {short} · {state}")
                }
            }
            _ => st.join(" · "),
        }
    };
    if !app.ui.details && note.is_empty() {
        v.push((format!("align: {}\t{short_state}", c.align), Token::Muted));
    } else if !app.ui.details {
        let full = format!("align: {} · {note}", c.align);
        v.push((full, Token::Warn));
    }
    if app.ui.details {
        if !note.is_empty() {
            // the width note joins the align line (warn); the basis gets its own
            let full = format!("align: {} · {note}", c.align);
            let line = if crate::layout::width(&full) + 2 <= w as usize {
                full
            } else {
                // very narrow: the short form keeps the strip within 3 rows (H9's full text from 80 up)
                let short = note
                    .split_once("(showing ")
                    .map(|(_, r)| format!("showing {} (widen for more)", r.trim_end_matches(')')))
                    .unwrap_or_else(|| "widen for more orgs".into());
                format!("align: {} · {short}", c.align)
            };
            v.push((line, Token::Warn));
            v.push((basis, Token::Muted));
        } else {
            let joined = format!("{align} · {basis}");
            if crate::layout::width(&app.ui.theme.text(&joined)) + 2 <= w as usize {
                v.push((joined, Token::Muted));
            } else {
                v.push((align, Token::Muted));
                v.push((basis, Token::Muted));
            }
        }
    }
    if c.align == "map"
        && app.source.account_map["categories"]
            .as_object()
            .is_none_or(|o| o.is_empty())
    {
        v.push((
            if narrow {
                // one row at 80
                "No account_map: nothing merges; every account sits in its org's section.".into()
            } else {
                "No account_map configured: nothing merges; every account sits in its org's section."
                    .into()
            },
            Token::Warn,
        ));
    }
    // the selected row's per-org names (H5): the focus line under the row now (design pass 2); the strip
    // keeps them only under `i`
    if !app.ui.details {
        return v;
    }
    if let Some(row) = app.cmp_selected_row() {
        let names: Vec<String> = row["names"]
            .as_object()
            .map(|o| {
                o.iter()
                    .filter_map(|(org, n)| {
                        let n: Vec<String> = n
                            .as_array()
                            .map(|a| a.iter().map(s).collect())
                            .unwrap_or_default();
                        (!n.is_empty()).then(|| format!("{org} {}", n.join(", ")))
                    })
                    .collect()
            })
            .unwrap_or_default();
        let code = s(&row["code"]);
        if row["section"] == "aligned" && !names.is_empty() {
            let head = if code.is_empty() {
                s(&row["label"])
            } else {
                code
            };
            // one row, cut with … (the column headers carry each org; design)
            let line = format!("{head} · {}", names.join(" · "));
            let th = &app.ui.theme;
            v.push((
                crate::layout::fit_str(
                    &th.text(&line),
                    (w as usize).saturating_sub(2),
                    th.glyphs().ellipsis,
                ),
                if row["names_differ"] == true {
                    Token::Warn
                } else {
                    Token::Muted
                },
            ));
        } else if !narrow {
            // below 100 the footer and help carry the keys
            v.push(("Enter: amount → that org's ledger · account → postings side by side · < > reorder · x drop".into(), Token::Muted));
        }
    }
    v
}

/// The account cell: the joined names when they fit; for a code-merged row whose names differ and don't
/// fit, the first org's name plus `≠` (the strip carries every name; design).
pub fn row_label(r: &Value, w: usize) -> String {
    let label = s(&r["label"]);
    if r["names_differ"] != true || width(&label) <= w {
        return label;
    }
    let first = r["names"]
        .as_object()
        .and_then(|o| {
            o.values()
                .filter_map(|n| n.as_array())
                .find_map(|a| a.first().map(s))
        })
        .unwrap_or(label);
    format!("{first} ≠")
}

fn cell_style(app: &App, base: Style, selected_row: bool, selected_cell: bool) -> Style {
    let th = &app.ui.theme;
    let mut st = base;
    if selected_row {
        st = st.patch(th.selection());
        if selected_cell {
            st = if matches!(th.mode, Mode::Ansi | Mode::NoColor) {
                // the row stays one continuous reverse span (v0.12.0): the cell is underlined and bold
                st.add_modifier(Modifier::UNDERLINED | Modifier::BOLD)
            } else {
                st.add_modifier(Modifier::REVERSED | Modifier::BOLD)
            };
        }
    }
    st
}

pub fn draw(app: &App, buf: &mut Buffer, area: Rect, hits: &mut Hits) {
    let th = &app.ui.theme;
    if let Some(e) = app.cmp_error() {
        let t = fit_str(
            &format!("Compare could not be built: {e}"),
            area.width.saturating_sub(2) as usize,
            "…",
        );
        buf.set_span(
            area.x + 1,
            area.y + 1,
            &Span::styled(t, th.fg(Token::Error)),
            area.width.saturating_sub(2),
        );
        return;
    }
    let mut area = area;
    for (text, tok) in strip_lines(app, area.width) {
        let h = strip::height(&th.text(&text), area.width).min(area.height);
        strip::draw(th, buf, Rect { height: h, ..area }, &text, tok);
        area.y += h;
        area.height -= h;
    }
    let Some((m, k, net, _, aw)) = current(app, area.width) else {
        return;
    };
    let c = app.ui.cmp.as_ref().expect("compare open");
    if c.level == 2 {
        draw_l2(app, m, k, aw, buf, area, hits);
    } else {
        draw_l1(app, m, k, net, aw, buf, area, hits);
    }
}

#[allow(clippy::too_many_arguments)]
fn draw_l1(
    app: &App,
    m: &CmpModel,
    k: usize,
    net: bool,
    aw: usize,
    buf: &mut Buffer,
    area: Rect,
    hits: &mut Hits,
) {
    let th = &app.ui.theme;
    let g = th.glyphs();
    let c = app.ui.cmp.as_ref().expect("compare open");
    let orgs: Vec<String> = m.orgs.iter().take(k).cloned().collect();
    let cw_amt = aw;
    let block = 2 + if net { cw_amt } else { 2 * cw_amt + 2 };
    let code_w = 6;
    let acct_w = (area.width as usize)
        .saturating_sub(2 + code_w + 2 + k * block)
        .max(8);
    let x_code = area.x + 2;
    let x_acct = x_code + code_w as u16 + 2;
    let x_org = |i: usize| x_acct + acct_w as u16 + (i * block) as u16;
    let rule_st = th.fg(Token::Line);
    let cols: Vec<&Value> = orgs
        .iter()
        .map(|o| {
            m.columns
                .iter()
                .find(|x| s(&x["org"]) == *o)
                .unwrap_or(&Value::Null)
        })
        .collect();
    let muted: Vec<bool> = cols
        .iter()
        .map(|col| col["basis"]["matches_xero"].is_null())
        .collect();
    // three header rows: org · basis state · sub-fields (H3)
    if area.height < 6 {
        return;
    }
    for (i, o) in orgs.iter().enumerate() {
        let x = x_org(i);
        let b = &cols[i]["basis"];
        let short = if b["kind"] == "xero-opening" {
            format!("TB+{}", app.ui.as_of.get(2..4).unwrap_or(""))
        } else {
            "history".into()
        };
        let state = format!("{short} {}", s(&b["state"]));
        let st_tok = if b["level"] == "warn" {
            Token::Warn
        } else {
            Token::Muted
        };
        for (y, t, st) in [
            (area.y, o.clone(), th.heading()),
            (area.y + 1, th.text(&state), th.fg(st_tok)),
        ] {
            buf.set_span(x, y, &Span::styled(g.vsep, rule_st), 1);
            let t = fit_str(&t, block - 1, g.ellipsis);
            let off = (block - 1 - width(&t)) / 2;
            buf.set_span(
                x + 1 + off as u16,
                y,
                &Span::styled(t, st),
                (block - 1) as u16,
            );
        }
        buf.set_span(x, area.y + 2, &Span::styled(g.vsep, rule_st), 1);
        let sub = if net {
            pad("net", cw_amt, g.ellipsis, true)
        } else {
            format!(
                "{}  {}",
                pad("debit", cw_amt, g.ellipsis, true),
                pad("credit", cw_amt, g.ellipsis, true)
            )
        };
        buf.set_span(
            x + 2,
            area.y + 2,
            &Span::styled(sub, th.fg(Token::Muted)),
            (block - 2) as u16,
        );
    }
    buf.set_span(
        x_code,
        area.y + 2,
        &Span::styled("code", th.fg(Token::Muted)),
        code_w as u16,
    );
    buf.set_span(
        x_acct,
        area.y + 2,
        &Span::styled("account", th.fg(Token::Muted)),
        acct_w as u16,
    );
    // body lines: section separators and rows
    let vis = app.cmp_visible(m);
    let full = m;
    let mut lines: Vec<Option<usize>> = Vec::new(); // None = separator, Some(pos in vis)
    let mut seps: Vec<String> = Vec::new();
    let mut section = String::new();
    for (pos, &ri) in vis.iter().enumerate() {
        let r = &full.rows[ri];
        let sec = s(&r["section"]);
        if sec != section {
            section = sec.clone();
            if sec != "aligned" {
                lines.push(None);
                seps.push(sec);
            }
        }
        lines.push(Some(pos));
    }
    let body = area.height.saturating_sub(3 + 2) as usize;
    let cursor = app.cmp_cursor();
    // the focus line (the selected row's per-org names) right under it
    const FOCUS: usize = usize::MAX;
    let focus = names_line(app).filter(|_| app.ui.cmp.as_ref().is_some_and(|c| c.level == 1));
    if focus.is_some()
        && let Some(i) = cursor.and_then(|cp| lines.iter().position(|l| *l == Some(cp)))
    {
        lines.insert(i + 1, Some(FOCUS));
    }
    let cur_line = cursor
        .and_then(|cp| lines.iter().position(|l| *l == Some(cp)))
        .unwrap_or(0);
    let cur_line = cur_line + usize::from(focus.is_some());
    let top = cur_line.saturating_sub(body.saturating_sub(1));
    let mut sep_i = lines[..top.min(lines.len())]
        .iter()
        .filter(|l| l.is_none())
        .count();
    hits.content = Some(Rect::new(area.x, area.y + 3, area.width, body as u16));
    for (dy, line) in lines.iter().enumerate().skip(top).take(body) {
        let y = area.y + 3 + (dy - top) as u16;
        if *line == Some(FOCUS) {
            if let Some((t, tok)) = &focus {
                let sel = th.selection();
                for xx in area.left()..area.right() {
                    buf[(xx, y)].set_style(sel);
                }
                let t = fit_str(
                    &th.text(t),
                    area.right().saturating_sub(x_acct + 1) as usize,
                    g.ellipsis,
                );
                let w = width(&t) as u16;
                buf.set_span(x_acct, y, &Span::styled(t, sel.patch(th.fg(*tok))), w);
                th.band_row(buf, Rect::new(area.x, y, area.width, 1));
            }
            continue;
        }
        let y = area.y + 3 + (dy - top) as u16;
        let Some(pos) = *line else {
            let t = th.text(&format!("── {} ──", seps[sep_i]));
            sep_i += 1;
            buf.set_span(
                x_acct,
                y,
                &Span::styled(t, th.fg(Token::Muted)),
                acct_w as u16,
            );
            continue;
        };
        let r = &full.rows[vis[pos]];
        let sel = cursor == Some(pos);
        if sel {
            for xx in area.left()..area.right() {
                buf[(xx, y)].set_style(th.selection());
            }
            buf.set_span(area.x, y, &Span::styled(g.cursor, th.fg(Token::Accent)), 1);
        }
        let differ = r["names_differ"] == true;
        let code = format!(
            "{}{}",
            s(&r["code"]),
            if differ {
                th.text(" ≠")
            } else {
                String::new()
            }
        );
        let code_tok = if differ { Token::Warn } else { Token::Muted };
        buf.set_span(
            x_code,
            y,
            &Span::styled(
                pad(&code, code_w, g.ellipsis, false),
                cell_style(app, th.fg(code_tok), sel, false),
            ),
            code_w as u16,
        );
        buf.set_span(
            x_acct,
            y,
            &Span::styled(
                pad(&th.text(&row_label(r, acct_w)), acct_w, g.ellipsis, false),
                cell_style(app, th.fg(Token::Text), sel, c.col == 0),
            ),
            acct_w as u16,
        );
        for (i, o) in orgs.iter().enumerate() {
            let x = x_org(i);
            buf.set_span(x, y, &Span::styled(g.vsep, rule_st), 1);
            let cell = &r["cells"][o.as_str()];
            let parts: Vec<String> = if cell["exists"] != true {
                if net {
                    vec!["—".into()]
                } else {
                    vec!["—".into(), "—".into()]
                }
            } else if net {
                vec![
                    dec(&cell["net"])
                        .filter(|v| !v.is_zero())
                        .map(fmt)
                        .unwrap_or_default(),
                ]
            } else {
                vec![
                    dec(&cell["debit"]).map(fmt).unwrap_or_default(),
                    dec(&cell["credit"]).map(fmt).unwrap_or_default(),
                ]
            };
            let text = parts
                .iter()
                .map(|p| pad(&th.text(p), cw_amt, g.ellipsis, true))
                .collect::<Vec<_>>()
                .join("  ");
            let tok = if muted[i] || cell["exists"] != true {
                Token::Muted
            } else {
                Token::Text
            };
            let st = cell_style(app, th.fg(tok), sel, c.col == i + 1);
            // the org's column in its own accent (knob `compare_columns`, off by default): identity only
            let st = if !sel && th.palette().knobs.compare_columns > 0 {
                let bg = th.bg(Token::member_column(i));
                crate::widgets::fill(buf, Rect::new(x + 1, y, (block - 1) as u16, 1), bg);
                bg.patch(st)
            } else {
                st
            };
            buf.set_span(x + 2, y, &Span::styled(text, st), (block - 2) as u16);
            hits.add(
                Rect::new(x + 1, y, block as u16 - 1, 1),
                Target::Cell(pos, i + 1),
            );
        }
        if sel {
            th.band_row(buf, Rect::new(area.x, y, area.width, 1));
        }
        hits.add(
            Rect::new(area.x, y, x_org(0) - area.x, 1),
            Target::Cell(pos, 0),
        );
    }
    // the per-org totals row (no cross-org total, V9)
    let ty = area.y + 3 + body as u16;
    if ty + 1 < area.bottom() + 1 {
        for xx in area.x + 2..area.right() {
            buf[(xx, ty)].set_symbol(g.rule).set_style(rule_st);
        }
        let y = ty + 1;
        if y < area.bottom() {
            buf.set_span(
                x_acct,
                y,
                &Span::styled(
                    pad("Total (each org's own)", acct_w, g.ellipsis, false),
                    th.heading(),
                ),
                acct_w as u16,
            );
            for (i, col) in cols.iter().enumerate() {
                let x = x_org(i);
                buf.set_span(x, y, &Span::styled(g.vsep, rule_st), 1);
                let balanced = col["balanced"] == true;
                let text = if net {
                    let t = if balanced {
                        th.text("balanced ✓")
                    } else {
                        fmt(dec(&col["debit"]).unwrap_or_default()
                            - dec(&col["credit"]).unwrap_or_default())
                    };
                    pad(&t, cw_amt, g.ellipsis, true)
                } else {
                    let d = dec(&col["debit"]).map(fmt).unwrap_or_default();
                    let cr = format!(
                        "{}{}",
                        dec(&col["credit"]).map(fmt).unwrap_or_default(),
                        if balanced {
                            th.text(" ✓")
                        } else {
                            String::new()
                        }
                    );
                    format!(
                        "{}  {}",
                        pad(&d, cw_amt, g.ellipsis, true),
                        pad(&cr, cw_amt, g.ellipsis, true)
                    )
                };
                let tok = if balanced { Token::Text } else { Token::Error };
                buf.set_span(
                    x + 2,
                    y,
                    &Span::styled(text, th.fg(tok).add_modifier(Modifier::BOLD)),
                    (block - 2) as u16,
                );
            }
        }
    }
}

impl App {
    /// How many shown postings the IC engine matched (for the L2 strip).
    pub fn cmp_matched_shown(&self) -> usize {
        let Some(sp) = self.cmp_l2() else { return 0 };
        let mut n = 0;
        for line in sp["spine"].as_array().into_iter().flatten() {
            if let Some(cells) = line["cells"].as_object() {
                for (o, p) in cells {
                    if !p.is_null()
                        && self
                            .cmp_view
                            .matched
                            .contains(&(o.clone(), s(&p["source_id"])))
                    {
                        n += 1;
                    }
                }
            }
        }
        n
    }
}

fn draw_l2(
    app: &App,
    m: &CmpModel,
    k: usize,
    aw: usize,
    buf: &mut Buffer,
    area: Rect,
    hits: &mut Hits,
) {
    let th = &app.ui.theme;
    let g = th.glyphs();
    let c = app.ui.cmp.as_ref().expect("compare open");
    let (Some(sp), Some(row)) = (app.cmp_l2(), app.cmp_l2_row()) else {
        return;
    };
    let orgs: Vec<String> = m.orgs.iter().take(k).cloned().collect();
    let mut y = area.y;
    // per-org opening / closing (I1)
    for col in sp["columns"].as_array().into_iter().flatten() {
        let o = s(&col["org"]);
        if !orgs.contains(&o) || y >= area.bottom() {
            continue;
        }
        let accts: Vec<String> = row["members"][o.as_str()]
            .as_array()
            .map(|a| a.iter().map(s).collect())
            .unwrap_or_default();
        let t = format!(
            "{o} · {}   opening {}   closing {}   {}",
            if accts.is_empty() {
                "—".into()
            } else {
                accts.join(", ")
            },
            dec(&col["opening"]).map_or("—".into(), dr_cr),
            dec(&col["closing"]).map_or("—".into(), dr_cr),
            plural(col["count"].as_i64().unwrap_or(0), "posting", "postings")
        );
        buf.set_span(
            area.x + 1,
            y,
            &Span::styled(
                fit_str(
                    &th.text(&t),
                    area.width.saturating_sub(2) as usize,
                    g.ellipsis,
                ),
                th.fg(Token::Text),
            ),
            area.width.saturating_sub(2),
        );
        y += 1;
    }
    y += 1;
    if y + 2 > area.bottom() {
        return;
    }
    // the spine: date | per org (what, amount)
    let amt_w = aw;
    let date_w = 10;
    let per = ((area.width as usize).saturating_sub(2 + date_w + 2) / k.max(1)).max(amt_w + 6);
    let what_w = per.saturating_sub(amt_w + 3);
    let x_org = |i: usize| area.x + 2 + date_w as u16 + 2 + (i * per) as u16;
    buf.set_span(
        area.x + 2,
        y,
        &Span::styled("date", th.fg(Token::Muted)),
        date_w as u16,
    );
    for (i, o) in orgs.iter().enumerate() {
        let x = x_org(i);
        buf.set_span(x, y, &Span::styled(g.vsep, th.fg(Token::Line)), 1);
        buf.set_span(
            x + 2,
            y,
            &Span::styled(pad(o, what_w, g.ellipsis, false), th.heading()),
            what_w as u16,
        );
        buf.set_span(
            x + 2 + what_w as u16 + 1,
            y,
            &Span::styled(pad("amount", amt_w, "~", true), th.fg(Token::Muted)),
            amt_w as u16,
        );
    }
    y += 1;
    let spine = sp["spine"].as_array().cloned().unwrap_or_default();
    let body = area.bottom().saturating_sub(y) as usize;
    let top = c.l2_idx.saturating_sub(body.saturating_sub(1));
    hits.content = Some(Rect::new(area.x, y, area.width, body as u16));
    for (i, line) in spine.iter().enumerate().skip(top).take(body) {
        let yy = y + (i - top) as u16;
        let sel = i == c.l2_idx;
        if sel {
            for xx in area.left()..area.right() {
                buf[(xx, yy)].set_style(th.selection());
            }
            buf.set_span(area.x, yy, &Span::styled(g.cursor, th.fg(Token::Accent)), 1);
        }
        buf.set_span(
            area.x + 2,
            yy,
            &Span::styled(
                s(&line["date"]),
                cell_style(app, th.fg(Token::Text), sel, false),
            ),
            date_w as u16,
        );
        for (j, o) in orgs.iter().enumerate() {
            let x = x_org(j);
            buf.set_span(x, yy, &Span::styled(g.vsep, th.fg(Token::Line)), 1);
            let p = &line["cells"][o.as_str()];
            if p.is_null() {
                continue;
            }
            let hit = c.show_matched
                && app
                    .cmp_view
                    .matched
                    .contains(&(o.clone(), s(&p["source_id"])));
            let what = format!(
                "{}{}  {}",
                if hit { "≡ " } else { "" },
                s(&p["kind"]),
                s(&p["contact"])
            );
            let selc = sel && c.l2_col == j + 1;
            buf.set_span(
                x + 2,
                yy,
                &Span::styled(
                    pad(&th.text(&what), what_w, g.ellipsis, false),
                    cell_style(app, th.fg(Token::Text), sel, selc),
                ),
                what_w as u16,
            );
            let amt_st = if hit {
                th.fg(Token::Accent).add_modifier(Modifier::BOLD)
            } else {
                th.fg(Token::Text)
            };
            let amt = dec(&p["amount"]).map(fmt).unwrap_or_default();
            buf.set_span(
                x + 2 + what_w as u16 + 1,
                yy,
                &Span::styled(
                    pad(&amt, amt_w, "~", true),
                    cell_style(app, amt_st, sel, selc),
                ),
                amt_w as u16,
            );
            hits.add(
                Rect::new(x + 1, yy, per as u16 - 1, 1),
                Target::Cell(i, j + 1),
            );
        }
        if sel {
            th.band_row(buf, Rect::new(area.x, yy, area.width, 1));
        }
    }
    if spine.is_empty() {
        buf.set_span(
            area.x + 2,
            y,
            &Span::styled("No postings in this range.", th.fg(Token::Muted)),
            area.width.saturating_sub(2),
        );
    }
}

/// The selected row's per-org account names (H5), for the focus line under it (design pass 2):
/// `1010 · roast Operating Account · design Client Receipts`.
pub fn names_line(app: &App) -> Option<(String, Token)> {
    if !app.source.view.focus_line {
        return None;
    }
    let row = app.cmp_selected_row()?;
    let names: Vec<String> = row["names"]
        .as_object()
        .map(|o| {
            o.iter()
                .filter_map(|(org, n)| {
                    let n: Vec<String> = n
                        .as_array()
                        .map(|a| a.iter().map(s).collect())
                        .unwrap_or_default();
                    (!n.is_empty()).then(|| format!("{org} {}", n.join(", ")))
                })
                .collect()
        })
        .unwrap_or_default();
    if row["section"] != "aligned" || names.is_empty() {
        return None;
    }
    let tok = if row["names_differ"] == true {
        Token::Warn
    } else {
        Token::Muted
    };
    Some((names.join(" · "), tok))
}
