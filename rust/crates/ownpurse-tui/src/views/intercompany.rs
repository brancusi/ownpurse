//! Tab 3: Intercompany (spec §9; acceptance F, G). L1 pairs (side A, side B party, difference, residual,
//! status; `s` sorts, `/` filters; the residual column never hides, V7), L2 the bridge of one pair (side
//! panels, the bridge table with the residual row in total style, one-sided items with their "why", known
//! items and context; `a` / `b` open a side's accounts in the Ledger), L3 an item's document in its own org,
//! drawn by the Ledger's L3 widget (M5). The strip is pinned under the breadcrumb (V6). Models are core's
//! `intercompany::build` result, prepared in `update` (`ic_nav`).

use crate::app::App;
use crate::clock::plural;
use crate::hit::{Hits, Target};
use crate::ic_nav::{IcRow, SORTS, dec, s};
use crate::layout::{ColSpec, Kind, fit_str, width, wrap};
use crate::theme_tokens::Token;
use crate::widgets::strip;
use crate::widgets::table::{self, Cell, Step, Table};
use ownpurse_core::{dr_cr, fmt};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Modifier;
use ratatui::text::Span;
use rust_decimal::Decimal;
use serde_json::Value;

pub const L1_PAIR: usize = 0;
pub const L1_A: usize = 1;
pub const L1_B: usize = 2;
pub const L1_RESIDUAL: usize = 4;
pub const L1_STATUS: usize = 5;

pub const L1_COLS: [ColSpec; 6] = [
    ColSpec {
        title: "pair",
        short_title: "",
        kind: Kind::Status,
        priority: 9,
    },
    ColSpec {
        title: "side A",
        short_title: "",
        kind: Kind::Text { min: 8 },
        priority: 2,
    },
    ColSpec {
        title: "side B (party)",
        short_title: "",
        kind: Kind::Text { min: 8 },
        priority: 3,
    },
    ColSpec {
        title: "difference",
        short_title: "",
        kind: Kind::Amount,
        priority: 9,
    },
    ColSpec {
        title: "residual",
        short_title: "",
        kind: Kind::Amount,
        priority: 9,
    },
    ColSpec {
        title: "status",
        short_title: "",
        kind: Kind::Status,
        priority: 9,
    },
];

/// Spec §4.3 Intercompany L1: side A → side B labels hide → pair name shortens to 18. Difference, residual
/// and status never hide (V7, V8).
/// Below that the pair shrinks
/// to 10; the pair column itself never hides.
pub const L1_STEPS: [Step; 4] = [
    Step::Hide(L1_A),
    Step::Hide(L1_B),
    Step::Shrink(L1_PAIR, 18),
    Step::Shrink(L1_PAIR, 10),
];

pub const ITEM_COLS: [ColSpec; 7] = [
    ColSpec {
        title: "side",
        short_title: "",
        kind: Kind::Status,
        priority: 9,
    },
    ColSpec {
        title: "date",
        short_title: "",
        kind: Kind::Status,
        priority: 9,
    },
    ColSpec {
        title: "org",
        short_title: "",
        kind: Kind::Text { min: 4 },
        priority: 1,
    },
    ColSpec {
        title: "account",
        short_title: "",
        kind: Kind::Text { min: 8 },
        priority: 5,
    },
    ColSpec {
        title: "contact",
        short_title: "",
        kind: Kind::Text { min: 6 },
        priority: 2,
    },
    ColSpec {
        title: "amount",
        short_title: "",
        kind: Kind::Amount,
        priority: 9,
    },
    ColSpec {
        title: "why",
        short_title: "",
        kind: Kind::Text { min: 10 },
        priority: 4,
    },
];

/// Items: org hides → contact hides → why shortens → account shortens (the full why is on the detail line).
pub const ITEM_STEPS: [Step; 4] = [
    Step::Hide(2),
    Step::Hide(4),
    Step::Shrink(6, 28),
    Step::Shrink(3, 16),
];

fn drcr(v: Option<Decimal>) -> String {
    v.map_or("—".into(), dr_cr)
}

fn amount(v: Option<Decimal>) -> String {
    v.map_or("—".into(), fmt)
}

pub fn crumbs(app: &App) -> Vec<String> {
    if !app.ic_configured() {
        return vec!["Intercompany".into()];
    }
    match (app.ic_level(), app.ic_pair()) {
        (2, Some(p)) => vec![s(&p["label"]), format!("bridge at {}", app.ui.as_of)],
        (3, Some(p)) => {
            let mut v = vec![s(&p["label"]), "bridge".into()];
            let nav = app.ic_nav();
            if let Some((org, _, id, kind)) = &nav.doc {
                v.push(org.clone());
                if let Some(it) = app.ic_items().iter().find(|i| s(&i["source_id"]) == *id) {
                    v.push(s(&it["account"]));
                }
                let date = app.ic_doc().map_or("", |d| d.date()).to_string();
                v.push(format!("{kind} {date}").trim().to_string());
            }
            v
        }
        _ => vec!["Intercompany".into(), "pairs".into()],
    }
}

pub fn status(app: &App) -> String {
    if !app.ic_configured() {
        return "intercompany · no pairs configured".into();
    }
    match (app.ic_level(), app.ic_pair()) {
        (2, Some(p)) => {
            let n = p["movement_items"].as_array().map_or(0, Vec::len) as i64;
            format!("{} · bridge · {}", s(&p["id"]), plural(n, "item", "items"))
        }
        (3, Some(p)) => format!(
            "{} · document · via {}",
            app.ic_nav().doc.map(|d| d.0).unwrap_or_default(),
            s(&p["id"])
        ),
        _ => format!(
            "intercompany · {} · sort: {}",
            plural(app.ic_rows().len() as i64, "pair", "pairs"),
            SORTS[app.ui.ic_sort % SORTS.len()]
        ),
    }
}

/// `Intercompany at … · balances: cached Xero TBs (oldest fetched 07:46 UTC) · items: documents` (F1), then
/// the not-compared note in warn.
pub fn l1_strip(app: &App) -> Vec<(String, Token)> {
    let pairs = app.ic_scoped();
    let fetched: Vec<String> = pairs
        .iter()
        .filter_map(|p| p["tb_fetched"].as_object())
        .flat_map(|o| o.values().filter_map(Value::as_str).map(str::to_string))
        .collect();
    let mut v = vec![(
        match fetched.iter().min() {
            Some(t) => format!(
                "Intercompany at {} · balances: cached Xero TBs (oldest fetched {} UTC) · items: documents",
                app.ui.as_of,
                t.get(11..16).unwrap_or("")
            ),
            None => format!(
                "Intercompany at {} · balances: cached Xero TBs · items: documents",
                app.ui.as_of
            ),
        },
        Token::Muted,
    )];
    let nc: Vec<&&Value> = pairs
        .iter()
        .filter(|p| p["status"] == "not compared")
        .collect();
    if !nc.is_empty() {
        let mut orgs: Vec<String> = nc
            .iter()
            .flat_map(|p| {
                p["missing_tb"]["orgs"]
                    .as_array()
                    .cloned()
                    .unwrap_or_default()
            })
            .map(|o| s(&o))
            .collect();
        orgs.sort();
        orgs.dedup();
        let mut dates: Vec<String> = nc.iter().map(|p| s(&p["missing_tb"]["date"])).collect();
        dates.sort();
        dates.dedup();
        v.push((
            format!(
                "not compared for {}: no cached Xero TB at {} (⌃P → Refetch)",
                orgs.join(", "),
                dates.join(", ")
            ),
            Token::Warn,
        ));
    }
    v
}

/// `residual 0.00 ✓ · balances: Xero TB design 07:46 · roast 07:46 · items from documents` (G1); the residual
/// is in error and without ✓ when not zero.
pub fn bridge_strip(app: &App, p: &Value) -> Vec<Span<'static>> {
    let th = &app.ui.theme;
    let res = dec(&p["residual"]).unwrap_or_default();
    let tbs: Vec<String> = p["tb_fetched"]
        .as_object()
        .map(|o| {
            o.iter()
                .map(|(org, t)| {
                    format!(
                        "{org} {}",
                        t.as_str().and_then(|t| t.get(11..16)).unwrap_or("—")
                    )
                })
                .collect()
        })
        .unwrap_or_default();
    vec![
        Span::styled("residual ", th.fg(Token::Text)),
        if res.is_zero() {
            Span::styled(th.text(&format!("{} ✓", fmt(res))), th.fg(Token::Ok))
        } else {
            Span::styled(fmt(res), th.fg(Token::Error).add_modifier(Modifier::BOLD))
        },
        Span::styled(
            th.text(&format!(
                " · balances: Xero TB {} · items from documents",
                tbs.join(" · ")
            )),
            th.fg(Token::Muted),
        ),
    ]
}

/// Wrapped styled spans (the strip never cuts): break at span-word boundaries.
fn put_wrapped(buf: &mut Buffer, area: Rect, spans: Vec<Span<'static>>) -> u16 {
    let w = area.width.saturating_sub(4) as usize;
    let mut words: Vec<(String, ratatui::style::Style)> = Vec::new();
    for sp in spans {
        for (i, word) in sp.content.split(' ').enumerate() {
            if i > 0 {
                words.push((" ".into(), sp.style));
            }
            if !word.is_empty() {
                words.push((word.to_string(), sp.style));
            }
        }
    }
    let (mut x, mut y) = (0usize, 0u16);
    for (word, style) in words {
        let ww = width(&word);
        if word == " " && x == 0 {
            continue;
        }
        if x + ww > w && x > 0 {
            y += 1;
            x = 0;
            if word == " " {
                continue;
            }
        }
        if y >= area.height {
            break;
        }
        let t = fit_str(&word, w, "…");
        buf.set_span(
            area.x + 2 + x as u16,
            area.y + y,
            &Span::styled(t, style),
            (w - x.min(w)) as u16,
        );
        x += ww;
    }
    y + 1
}

fn message(app: &App, buf: &mut Buffer, area: Rect, text: &str, tok: Token) {
    let th = &app.ui.theme;
    for (i, l) in wrap(&th.text(text), area.width.saturating_sub(2) as usize)
        .into_iter()
        .enumerate()
    {
        let y = area.y + 1 + i as u16;
        if y >= area.bottom() {
            break;
        }
        buf.set_span(
            area.x + 1,
            y,
            &Span::styled(l, th.fg(tok)),
            area.width.saturating_sub(2),
        );
    }
}

pub fn draw(app: &App, buf: &mut Buffer, area: Rect, hits: &mut Hits) {
    if !app.ic_configured() {
        message(
            app,
            buf,
            area,
            "No intercompany pairs configured. Add \"intercompany.pairs\" to ownpurse.json (docs/ROADMAP.md §2).",
            Token::Muted,
        );
        return;
    }
    if let Some(e) = app.ic_error() {
        message(
            app,
            buf,
            area,
            &format!("Intercompany could not be computed: {e}"),
            Token::Error,
        );
        return;
    }
    match (app.ic_level(), app.ic_pair()) {
        (2, Some(p)) => draw_l2(app, p, buf, area, hits),
        (3, Some(p)) => {
            let mut area = area;
            let h = put_wrapped(buf, area, bridge_strip(app, p)).min(area.height);
            area.y += h;
            area.height -= h;
            match app.ic_view.doc.as_deref() {
                Some(Ok(d)) => super::ledger::draw_l3(app, d, buf, area, hits),
                Some(Err(e)) => message(
                    app,
                    buf,
                    area,
                    &format!("Could not read the document: {e}"),
                    Token::Error,
                ),
                None => message(
                    app,
                    buf,
                    area,
                    "The document's org is not in this profile.",
                    Token::Muted,
                ),
            }
        }
        _ => draw_l1(app, buf, area, hits),
    }
}

pub fn l1_cells(rows: &[&Value]) -> Vec<Vec<Cell>> {
    rows.iter()
        .map(|p| {
            let r = IcRow::of(p);
            let side = |org: &str, net: Option<Decimal>| match net {
                Some(n) if !n.is_zero() => format!("{org} {}", dr_cr(n)),
                _ => org.to_string(),
            };
            let (res_tok, st_tok) = match r.status.as_str() {
                "unexplained" => (Token::Error, Token::Error),
                "not compared" | "not supported" => (Token::Text, Token::Warn),
                "matches" => (Token::Text, Token::Ok),
                _ => (Token::Text, Token::Text),
            };
            vec![
                Cell::new(r.label.clone(), Token::Text),
                Cell::new(side(&r.a_org, r.a_net), Token::Muted),
                Cell::new(side(&r.b_org, r.b_party_net), Token::Muted),
                Cell::new(amount(r.difference), Token::Text),
                Cell::new(amount(r.residual), res_tok),
                Cell::new(r.status_text(), st_tok),
            ]
        })
        .collect()
}

fn draw_l1(app: &App, buf: &mut Buffer, area: Rect, hits: &mut Hits) {
    let th = &app.ui.theme;
    let mut area = area;
    for (text, tok) in l1_strip(app) {
        let h = strip::height(&text, area.width).min(area.height);
        strip::draw(th, buf, Rect { height: h, ..area }, &text, tok);
        area.y += h;
        area.height -= h;
    }
    // summary (F2): counts, `N unexplained` in error when N > 0, then the sort
    let scoped = app.ic_scoped();
    let count = |st: &str| scoped.iter().filter(|p| p["status"] == st).count();
    let mut spans = vec![Span::styled(
        plural(scoped.len() as i64, "pair", "pairs"),
        th.fg(Token::Text),
    )];
    for (st, tok) in [
        ("matches", Token::Text),
        ("bridged", Token::Text),
        (
            "unexplained",
            if count("unexplained") > 0 {
                Token::Error
            } else {
                Token::Text
            },
        ),
        ("not compared", Token::Warn),
        ("not supported", Token::Warn),
    ] {
        let n = count(st);
        if n == 0 && matches!(st, "not compared" | "not supported") {
            continue;
        }
        spans.push(Span::styled(th.text(" · "), th.fg(Token::Muted)));
        spans.push(Span::styled(format!("{n} {st}"), th.fg(tok)));
    }
    spans.push(Span::styled(
        format!("   sort: {} (s)", SORTS[app.ui.ic_sort % SORTS.len()]),
        th.fg(Token::Muted),
    ));
    if area.height > 0 {
        let h = put_wrapped(
            buf,
            Rect {
                height: 2.min(area.height),
                ..area
            },
            spans,
        );
        area.y += h;
        area.height -= h;
    }
    if let Some(line) = app.filter_chip().filter(|_| area.height > 0) {
        strip::draw(th, buf, Rect { height: 1, ..area }, &line, Token::Muted);
        area.y += 1;
        area.height -= 1;
    }
    let rows = app.ic_rows();
    let cells = l1_cells(&rows);
    // the detail line: why the selected pair can't be opened (the status column says only the status)
    let detail = app
        .ic_l1_cursor()
        .map(|c| IcRow::of(rows[c]))
        .filter(|r| !r.openable())
        .map(|r| r.refusal());
    if let Some(d) = &detail
        && area.height > 3
    {
        let y = area.bottom() - 1;
        let t = fit_str(
            &th.text(d),
            area.width.saturating_sub(2) as usize,
            th.glyphs().ellipsis,
        );
        buf.set_span(
            area.x + 1,
            y,
            &Span::styled(t, th.fg(Token::Warn)),
            area.width.saturating_sub(2),
        );
        area.height -= 2;
    }
    hits.content = Some(area);
    let nav = app.ic_nav();
    table::draw(
        th,
        buf,
        area,
        &Table {
            cols: &L1_COLS,
            rows: &cells,
            steps: &L1_STEPS,
            top: nav.l1_top,
            cursor: app.ic_l1_cursor(),
            cursor_col: None,
            footer: None,
            hidden: &[],
            focus: None,
            groups: &[],
        },
        hits,
    );
    if rows.is_empty() && area.height > 1 {
        let msg = if !app.ui.filter.is_empty() {
            "No pair matches the filter.".to_string()
        } else {
            let scope = app.scope_label(&app.ui.scope);
            let scope = scope
                .strip_prefix("group:")
                .map_or(scope.clone(), |g| format!("group {g}"));
            format!("No intercompany pairs involve {scope}.")
        };
        buf.set_span(
            area.x + 2,
            area.y + 1,
            &Span::styled(msg, th.fg(Token::Muted)),
            area.width.saturating_sub(2),
        );
    }
}

/// The bridge rows (G2), in order: label, value, is the residual row.
pub fn bridge_rows(p: &Value) -> Vec<(String, Option<Decimal>, bool)> {
    let n = p["movement_items"].as_array().map_or(0, Vec::len) as i64;
    let year = s(&p["year_start"]).chars().take(4).collect::<String>();
    vec![
        (
            "Difference at year end".into(),
            dec(&p["difference"]),
            false,
        ),
        (
            "  = prior-year opening difference".into(),
            dec(&p["opening_difference"]),
            false,
        ),
        (
            format!("  + {year} movement difference"),
            dec(&p["movement_difference"]),
            false,
        ),
        (
            format!("      explained by {} below", plural(n, "item", "items")),
            dec(&p["movement_explained"]),
            false,
        ),
        ("Unexplained residual".into(), dec(&p["residual"]), true),
    ]
}

/// (text, token, bold) for one line of a side panel.
type SideLine = (String, Token, bool);

fn side_lines(app: &App, p: &Value, wide: bool, w: usize) -> (Vec<SideLine>, Vec<SideLine>) {
    let accounts = |k: &str| -> String {
        let a = p[k]
            .as_array()
            .map(|a| a.iter().map(s).collect::<Vec<_>>().join(" · "))
            .unwrap_or_default();
        if a.is_empty() {
            "no paired accounts with postings".into()
        } else {
            a
        }
    };
    let party = s(&p["party"]);
    let comp = s(&p["component"]);
    let b_head = if party.is_empty() {
        format!("side B · {}", s(&p["b_org"]))
    } else {
        format!("side B · {} · {party}'s share ({comp})", s(&p["b_org"]))
    };
    let others: Vec<String> = p["other_components"]
        .as_object()
        .map(|o| {
            o.iter()
                .map(|(k, v)| format!("{k} {}", drcr(dec(v))))
                .collect()
        })
        .unwrap_or_default();
    let ell = app.ui.theme.glyphs().ellipsis;
    if wide {
        let a = vec![
            (format!("side A · {}", s(&p["a_org"])), Token::Text, true),
            (drcr(dec(&p["a_net"])), Token::Text, false),
            (
                fit_str(&accounts("a_accounts"), w, ell),
                Token::Muted,
                false,
            ),
        ];
        let mut b = vec![
            (b_head, Token::Text, true),
            (drcr(dec(&p["b_party_net"])), Token::Text, false),
            (
                fit_str(
                    &format!(
                        "{} · account total {}",
                        accounts("b_accounts"),
                        drcr(dec(&p["b_net"]))
                    ),
                    w,
                    ell,
                ),
                Token::Muted,
                false,
            ),
        ];
        if !others.is_empty() {
            b.push((
                fit_str(&format!("other parties: {}", others.join(" · ")), w, ell),
                Token::Muted,
                false,
            ));
        }
        (a, b)
    } else {
        let a = vec![(
            format!(
                "A {} {} · {}",
                s(&p["a_org"]),
                drcr(dec(&p["a_net"])),
                accounts("a_accounts")
            ),
            Token::Text,
            false,
        )];
        let share = if party.is_empty() {
            String::new()
        } else {
            format!(" ({party}'s share)")
        };
        let b = vec![(
            format!(
                "B {} {}{share} · {}",
                s(&p["b_org"]),
                drcr(dec(&p["b_party_net"])),
                accounts("b_accounts")
            ),
            Token::Text,
            false,
        )];
        (a, b)
    }
}

pub fn item_cells(items: &[Value]) -> Vec<Vec<Cell>> {
    items
        .iter()
        .map(|it| {
            vec![
                Cell::new(s(&it["side"]).to_uppercase(), Token::Text),
                Cell::new(s(&it["date"]), Token::Text),
                Cell::new(s(&it["org"]), Token::Muted),
                Cell::new(s(&it["account"]), Token::Text),
                Cell::new(s(&it["contact"]), Token::Muted),
                Cell::new(amount(dec(&it["amount"])), Token::Text),
                Cell::new(s(&it["why"]), Token::Muted),
            ]
        })
        .collect()
}

fn draw_l2(app: &App, p: &Value, buf: &mut Buffer, area: Rect, hits: &mut Hits) {
    let th = &app.ui.theme;
    let ell = th.glyphs().ellipsis;
    let mut area = area;
    // V6 / G1: the residual strip, pinned and wrapped, when there is something to explain (or under `i`);
    // a fully explained bridge says so at the breadcrumb's right (design pass 2)
    if app.ui.details {
        let h = put_wrapped(buf, area, bridge_strip(app, p)).min(area.height);
        area.y += h;
        area.height -= h;
    }
    area.y += 1;
    area.height = area.height.saturating_sub(1);
    // side panels: side by side from 100 columns, one line each below
    let wide = area.width >= 100;
    let half = (area.width / 2) as usize;
    let (a, b) = side_lines(app, p, wide, half.saturating_sub(3));
    let put = |buf: &mut Buffer, x: u16, y: u16, w: u16, t: &str, tok: Token, bold: bool| {
        let mut st = th.fg(tok);
        if bold {
            st = st.add_modifier(Modifier::BOLD);
        }
        buf.set_span(
            x + 2,
            y,
            &Span::styled(fit_str(&th.text(t), w.saturating_sub(3) as usize, ell), st),
            w.saturating_sub(3),
        );
    };
    let side_h = if wide { a.len().max(b.len()) as u16 } else { 2 };
    if area.height > side_h + 8 || area.height > 10 {
        if wide {
            for (i, (t, tok, bold)) in a.iter().enumerate() {
                put(buf, area.x, area.y + i as u16, half as u16, t, *tok, *bold);
            }
            for (i, (t, tok, bold)) in b.iter().enumerate() {
                put(
                    buf,
                    area.x + half as u16,
                    area.y + i as u16,
                    area.width - half as u16,
                    t,
                    *tok,
                    *bold,
                );
            }
        } else {
            put(buf, area.x, area.y, area.width, &a[0].0, a[0].1, false);
            put(buf, area.x, area.y + 1, area.width, &b[0].0, b[0].1, false);
        }
        area.y += side_h + 1;
        area.height = area.height.saturating_sub(side_h + 1);
    }
    // the bridge (G2, G3): never hidden; the residual row in total style, error when not zero
    let rows = bridge_rows(p);
    if area.height > 0 {
        crate::widgets::band(th, buf, Rect { height: 1, ..area }, "Bridge", None);
        area.y += 1;
        area.height -= 1;
    }
    let amounts: Vec<String> = rows.iter().map(|(_, v, _)| amount(*v)).collect();
    let amt_w = amounts.iter().map(|a| width(a)).max().unwrap_or(4) as u16;
    let label_w = rows
        .iter()
        .map(|(l, _, _)| width(l) as u16)
        .max()
        .unwrap_or(10)
        .min(area.width.saturating_sub(amt_w + 6));
    let mut y = area.y;
    for (i, ((label, v, total), amt)) in rows.iter().zip(&amounts).enumerate() {
        if *total && y < area.bottom() {
            let line = th.fg(Token::Line);
            for x in area.x + 4..area.x + 4 + label_w + 2 + amt_w {
                buf[(x, y)].set_symbol(th.glyphs().rule).set_style(line);
            }
            y += 1;
        }
        if y >= area.bottom() {
            break;
        }
        let tok = if *total && v.is_some_and(|d| !d.is_zero()) {
            Token::Error
        } else {
            Token::Text
        };
        let mut st = th.fg(tok);
        if *total {
            st = st.add_modifier(Modifier::BOLD);
        }
        buf.set_span(
            area.x + 4,
            y,
            &Span::styled(fit_str(label, label_w as usize, ell), st),
            label_w,
        );
        let ax = area.x + 4 + label_w + 2;
        if *total && v.is_some_and(|d| d.is_zero()) {
            let ok = th.text(" ✓");
            buf.set_span(ax + amt_w, y, &Span::styled(ok, th.fg(Token::Muted)), 2);
        }
        buf.set_span(
            ax + amt_w - width(amt) as u16,
            y,
            &Span::styled(amt.clone(), st),
            amt_w,
        );
        let _ = i;
        y += 1;
    }
    y += app.section_gap();
    // items on one side only (G4)
    let items = app.ic_items();
    let below: u16 = 3; // detail + known + context
    if y < area.bottom() {
        crate::widgets::band(
            th,
            buf,
            Rect::new(area.x, y, area.width, 1),
            "Items on one side only",
            (!items.is_empty()).then(|| (items.len().to_string(), Token::Muted)),
        );
        y += 1;
    }
    // none: one quiet line instead of an empty table (design pass 2)
    if items.is_empty() {
        if y < area.bottom() {
            buf.set_span(
                area.x + 2,
                y,
                &Span::styled("No items on one side.", th.fg(Token::Muted)),
                area.width.saturating_sub(3),
            );
        }
        y += 1;
    }
    let table_h = if items.is_empty() {
        0
    } else {
        (items.len() as u16 + 1)
            .max(2)
            .min(area.bottom().saturating_sub(y + below).max(2))
            .min(area.bottom().saturating_sub(y))
    };
    let t_area = Rect::new(area.x, y, area.width, table_h);
    hits.content = Some(t_area);
    let cells = item_cells(&items);
    let nav = app.ic_nav();
    if table_h > 0 {
        table::draw(
            th,
            buf,
            t_area,
            &Table {
                cols: &ITEM_COLS,
                rows: &cells,
                steps: &ITEM_STEPS,
                top: nav.item_top,
                cursor: app.ic_item_cursor(),
                cursor_col: None,
                footer: None,
                hidden: &[],
                focus: None,
                groups: &[],
            },
            hits,
        );
    }
    y += table_h;
    // detail line, known items, context (G4, G5)
    let mut lines: Vec<(String, Token)> = Vec::new();
    if let Some(c) = app.ic_item_cursor() {
        let it = &items[c];
        lines.push((
            format!(
                "why: {}  ·  {} {}",
                s(&it["why"]),
                s(&it["source"]),
                s(&it["source_id"])
            ),
            Token::Text,
        ));
    }
    // G5: each label once as a heading, its items indented beneath it
    let known = p["known_items"].as_array().cloned().unwrap_or_default();
    // "none" is a detail (`i`); known items and context show when they have content
    if known.is_empty() {
        if app.ui.details {
            lines.push(("known items: none".into(), Token::Muted));
        }
    } else {
        lines.push(("known items:".into(), Token::Text));
    }
    for k in &known {
        match k["found"].as_array().and_then(|f| f.first()) {
            Some(f) => lines.push((
                format!(
                    "  ✓ found {} {} {} {} · {}",
                    s(&f["date"]),
                    s(&f["org"]),
                    s(&f["account"]),
                    amount(dec(&f["amount"])),
                    s(&k["text"])
                ),
                Token::Text,
            )),
            None => lines.push((
                format!("  ! not found in the record · {}", s(&k["text"])),
                Token::Warn,
            )),
        }
    }
    let context = p["context"].as_array().cloned().unwrap_or_default();
    if context.is_empty() {
        if app.ui.details {
            lines.push(("context: none".into(), Token::Muted));
        }
    } else {
        lines.push(("context:".into(), Token::Muted));
    }
    for c in &context {
        lines.push((
            format!(
                "  {} {} {}",
                s(&c["org"]),
                s(&c["account"]),
                drcr(dec(&c["balance"]))
            ),
            Token::Muted,
        ));
    }
    for (text, tok) in lines {
        // indented items keep their indent on every wrapped line
        let indent = text.len() - text.trim_start().len();
        let pad_s = " ".repeat(indent);
        let w = (area.width as usize).saturating_sub(4 + indent);
        for l in wrap(&th.text(text.trim_start()), w)
            .into_iter()
            .map(|l| format!("{pad_s}{l}"))
        {
            if y >= area.bottom() {
                return;
            }
            buf.set_span(
                area.x + 2,
                y,
                &Span::styled(l, th.fg(tok)),
                area.width.saturating_sub(3),
            );
            y += 1;
        }
    }
    let _ = Target::Row(0);
}

/// The bridge leaves nothing unexplained (its residual row is zero).
pub fn explained(p: &Value) -> bool {
    bridge_rows(p)
        .iter()
        .rev()
        .find(|(_, _, total)| *total)
        .is_some_and(|(_, v, _)| v.is_some_and(|d| d.is_zero()))
}

/// The residual's amount as the bridge shows it.
pub fn residual_text(p: &Value) -> String {
    bridge_rows(p)
        .iter()
        .rev()
        .find(|(_, _, total)| *total)
        .map(|(_, v, _)| amount(*v))
        .unwrap_or_default()
}
