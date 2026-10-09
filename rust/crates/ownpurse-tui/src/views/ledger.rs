//! Tab 2: Ledger (spec §9). L1 trial balance on the basis (`h` toggles year ⇄ full history, `!` keeps only
//! accounts that differ from Xero), L2 one account's postings with a running balance (`f` sets the first
//! day), L3 one document (`o` opens it in Xero, `y` copies its id). The basis strip is pinned under the
//! breadcrumb and recomputed every draw from the basis facts (V6). Models come prepared (`ledger_nav`).

use crate::app::{App, Scope};
use crate::hit::Hits;
use crate::layout::{ColSpec, Kind, width, wrap};
use crate::ledgers::{DocModel, L1Model, L2Model};
use crate::theme_tokens::Token;
use crate::widgets::strip;
use crate::widgets::table::{self, Cell, Step, Table};
use ownpurse_core::fmt;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::text::Span;
use rust_decimal::Decimal;

// ── L1 ───────────────────────────────────────────────────────────────────────────────────────────

pub const L1_FLAG: usize = 0;
pub const L1_CODE: usize = 1;
pub const L1_ACCOUNT: usize = 2;
pub const L1_CLASS: usize = 3;

pub const L1_COLS: [ColSpec; 6] = [
    ColSpec {
        title: "!",
        short_title: "",
        kind: Kind::Status,
        priority: 9,
    },
    ColSpec {
        title: "code",
        short_title: "",
        kind: Kind::Id,
        priority: 3,
    },
    ColSpec {
        title: "account",
        short_title: "",
        kind: Kind::Text { min: 8 },
        priority: 9,
    },
    ColSpec {
        title: "class",
        short_title: "",
        kind: Kind::Text { min: 5 },
        priority: 2,
    },
    ColSpec {
        title: "debit",
        short_title: "",
        kind: Kind::Amount,
        priority: 9,
    },
    ColSpec {
        title: "credit",
        short_title: "",
        kind: Kind::Amount,
        priority: 9,
    },
];

/// Spec §4.3 Ledger L1: account shortens to 16 first, then class hides, then code.
pub const L1_STEPS: [Step; 3] = [
    Step::Shrink(L1_ACCOUNT, 16),
    Step::Hide(L1_CLASS),
    Step::Hide(L1_CODE),
];

/// With the `!` filter on, L1 shows what Parity used to (C11): Xero's balance and the difference, Dr/Cr.
pub const L1_DIFF_COLS: [ColSpec; 8] = [
    L1_COLS[0],
    L1_COLS[1],
    L1_COLS[2],
    L1_COLS[3],
    L1_COLS[4],
    L1_COLS[5],
    ColSpec {
        title: "xero",
        short_title: "",
        kind: Kind::Amount,
        priority: 9,
    },
    ColSpec {
        title: "difference",
        short_title: "",
        kind: Kind::Amount,
        priority: 9,
    },
];

/// The same at ≥ 100 columns, where the difference column says what it is.
pub const L1_DIFF_COLS_WIDE: [ColSpec; 8] = [
    L1_COLS[0],
    L1_COLS[1],
    L1_COLS[2],
    L1_COLS[3],
    L1_COLS[4],
    L1_COLS[5],
    L1_DIFF_COLS[6],
    ColSpec {
        title: "local − xero",
        short_title: "",
        kind: Kind::Amount,
        priority: 9,
    },
];

/// C11: class hides first to make room, then the account shortens to 16, then code hides.
pub const L1_DIFF_STEPS: [Step; 3] = [
    Step::Hide(L1_CLASS),
    Step::Shrink(L1_ACCOUNT, 16),
    Step::Hide(L1_CODE),
];

// ── L2 ───────────────────────────────────────────────────────────────────────────────────────────

pub const L2_SOURCE: usize = 1;
pub const L2_REFERENCE: usize = 2;
pub const L2_CONTACT: usize = 3;
pub const L2_DESCRIPTION: usize = 4;

pub const L2_COLS: [ColSpec; 8] = [
    ColSpec {
        title: "date",
        short_title: "",
        kind: Kind::Status,
        priority: 9,
    },
    ColSpec {
        title: "source",
        short_title: "src",
        kind: Kind::Status,
        priority: 6,
    },
    ColSpec {
        title: "reference",
        short_title: "",
        kind: Kind::Text { min: 6 },
        priority: 1,
    },
    ColSpec {
        title: "contact",
        short_title: "",
        kind: Kind::Text { min: 6 },
        priority: 4,
    },
    ColSpec {
        title: "description",
        short_title: "",
        kind: Kind::Text { min: 8 },
        priority: 2,
    },
    ColSpec {
        title: "debit",
        short_title: "",
        kind: Kind::Amount,
        priority: 9,
    },
    ColSpec {
        title: "credit",
        short_title: "",
        kind: Kind::Amount,
        priority: 9,
    },
    ColSpec {
        title: "balance",
        short_title: "",
        kind: Kind::Amount,
        priority: 9,
    },
];

/// Spec §4.3 Ledger L2: reference → description → source label shortens (`Spend` → `Sp`) → contact to 10.
pub const L2_STEPS: [Step; 4] = [
    Step::Hide(L2_REFERENCE),
    Step::Hide(L2_DESCRIPTION),
    Step::Short(L2_SOURCE),
    Step::Shrink(L2_CONTACT, 10),
];

// ── L3 lines ─────────────────────────────────────────────────────────────────────────────────────

pub const L3_COLS: [ColSpec; 4] = [
    ColSpec {
        title: "description",
        short_title: "",
        kind: Kind::Text { min: 6 },
        priority: 3,
    },
    ColSpec {
        title: "account",
        short_title: "",
        kind: Kind::Text { min: 6 },
        priority: 6,
    },
    ColSpec {
        title: "amount",
        short_title: "",
        kind: Kind::Amount,
        priority: 9,
    },
    ColSpec {
        title: "tax",
        short_title: "",
        kind: Kind::Amount,
        priority: 1,
    },
];

/// Spec §4.3 Ledger L3 lines: tax hides → description shortens; account shortens to code + 8.
pub const L3_STEPS: [Step; 3] = [Step::Hide(3), Step::Shrink(0, 12), Step::Shrink(1, 14)];

fn amt(v: Decimal, debit: bool) -> String {
    if debit && v > Decimal::ZERO {
        fmt(v)
    } else if !debit && v < Decimal::ZERO {
        fmt(-v)
    } else {
        String::new() // blank means zero (spec §4.2)
    }
}

fn org_name(app: &App) -> String {
    if let Some(v) = &app.ui.via {
        return app.model.orgs[v.org].alias.clone();
    }
    match app.ui.scope {
        Scope::Org(i) => app.model.orgs[i].alias.clone(),
        _ => app.scope_label(&app.ui.scope),
    }
}

/// Breadcrumb after the chain origin: `roast › Trial balance at 2025-12-31 (accrual) › Checking (090) › Spend 2025-10-21`.
pub fn crumbs(app: &App) -> Vec<String> {
    let mut v = Vec::new();
    if let Some(via) = &app.ui.via {
        v.push(via.label.clone());
        v.push("bridge".into());
    }
    v.push(org_name(app));
    if app.ledger_org().is_none() {
        v.push("Ledger".into());
        return v;
    }
    v.push(format!("Trial balance at {} (accrual)", app.ui.as_of));
    let level = app.ledger_level();
    if level >= 2
        && let Some(m) = app.l2_model()
    {
        v.push(m.account.clone());
    }
    if level == 3
        && let Some(d) = app.doc_model()
    {
        v.push(doc_crumb(app, d));
    }
    v
}

/// `Spend 2025-10-21`: the posting's kind and the document date (never an entity name; E2).
fn doc_crumb(app: &App, d: &DocModel) -> String {
    let kind = app
        .l2_model()
        .and_then(|m| {
            m.rows
                .iter()
                .find(|r| r.source_id == d.id)
                .map(|r| r.kind.clone())
        })
        .unwrap_or_else(|| d.source.clone());
    format!("{kind} {}", d.date()).trim().to_string()
}

pub fn status(app: &App) -> String {
    let org = org_name(app);
    if app.ledger_org().is_none() {
        return format!("{org} · ledger");
    }
    match app.ledger_level() {
        // the count leads, so it survives truncation
        1 if app.ui.differs_only => app
            .l1_model()
            .map_or(format!("{org} · trial balance"), |m| {
                let n = m.rows.iter().filter(|r| r.differs).count();
                let verb = if n == 1 { "differs" } else { "differ" };
                format!("{org} · {n} {verb} · trial balance")
            }),
        2 => app.l2_model().map_or(format!("{org} · postings"), |m| {
            format!(
                "{org} · {} · {}",
                m.account,
                crate::clock::plural(m.rows.len() as i64, "posting", "postings")
            )
        }),
        3 => app.doc_model().map_or(format!("{org} · document"), |d| {
            format!("{org} · {}", doc_crumb(app, d))
        }),
        _ => app
            .l1_model()
            .map_or(format!("{org} · trial balance"), |m| {
                format!(
                    "{org} · trial balance · {}",
                    crate::clock::plural(m.rows.len() as i64, "account", "accounts")
                )
            }),
    }
}

/// Lines of text (wrapped, never cut) from the second row of `area`; returns rows used.
fn message(app: &App, buf: &mut Buffer, area: Rect, lines: &[(String, Token)]) -> u16 {
    let th = &app.ui.theme;
    let mut y = area.y + 1;
    for (text, tok) in lines {
        for l in wrap(&th.text(text), area.width.saturating_sub(2) as usize) {
            if y >= area.bottom() {
                return y - area.y;
            }
            buf.set_span(
                area.x + 1,
                y,
                &Span::styled(l, th.fg(*tok)),
                area.width.saturating_sub(2),
            );
            y += 1;
        }
    }
    y - area.y
}

/// The pinned strip (V6) for the current level: (lines, token), recomputed every draw.
pub fn strip_lines(app: &App) -> Vec<(String, Token)> {
    let mut v = level_strip(app);
    if let Some(c) = app.filter_chip() {
        v.push((c, Token::Info));
    }
    v
}

fn level_strip(app: &App) -> Vec<(String, Token)> {
    match app.ledger_level() {
        1 => app.l1_model().map_or(vec![], |m| {
            // a healthy basis is a short state at the breadcrumb's right (`✓ matches Xero`); the sentence is a
            // detail (`i`). One that needs a look keeps its strip (design pass 2).
            let mut v = if m.basis.ok() && !app.ui.details {
                vec![]
            } else {
                vec![(
                    m.basis.line(),
                    if m.basis.ok() { Token::Ok } else { Token::Warn },
                )]
            };
            if app.ui.differs_only {
                let n = m.rows.iter().filter(|r| r.differs).count();
                v.push((
                    format!("showing: differs from Xero ({n}) · difference = local − Xero"),
                    Token::Warn,
                ));
            }
            v
        }),
        2 => app.l2_model().map_or(vec![], |m| {
            vec![
                (
                    format!(
                        "opening {} {} · debits {} · credits {} · closing {}",
                        fmt(m.opening),
                        m.opening_source,
                        fmt(m.debits),
                        fmt(m.credits),
                        fmt(m.closing)
                    ),
                    if m.opening_from_xero {
                        Token::Ok
                    } else {
                        Token::Warn
                    },
                ),
                (
                    format!("postings from {} to {}", m.since, m.as_of),
                    Token::Muted,
                ),
            ]
        }),
        _ => vec![],
    }
}

pub fn draw(app: &App, buf: &mut Buffer, area: Rect, hits: &mut Hits) {
    if app.cmp_active() {
        super::compare::draw(app, buf, area, hits);
        return;
    }
    let org = match (&app.ui.via, &app.ui.scope) {
        (Some(v), _) => v.org,
        (None, Scope::Org(i)) => *i,
        _ => {
            let aliases: Vec<&str> = app.model.orgs.iter().map(|o| o.alias.as_str()).collect();
            message(
                app,
                buf,
                area,
                &[(
                    format!(
                        "Ledger needs one org: press g or ] to choose ({}).",
                        aliases.join(", ")
                    ),
                    Token::Muted,
                )],
            );
            return;
        }
    };
    if app.ledger_org().is_none() {
        let a = &app.model.orgs[org].alias;
        message(
            app,
            buf,
            area,
            &[(
                format!("No local record for {a} yet. ⌃P → Sync {a}."),
                Token::Error,
            )],
        );
        return;
    }
    let th = &app.ui.theme;
    // V6: the strip is pinned under the breadcrumb; it wraps (1–3 rows) and is never cut.
    let mut area = area;
    for (text, tok) in strip_lines(app) {
        let h = strip::height(&text, area.width).min(area.height);
        strip::draw(th, buf, Rect { height: h, ..area }, &text, tok);
        area.y += h;
        area.height -= h;
    }
    match app.ledger_level() {
        1 => match app.ledger_view.l1.as_deref() {
            Some(Ok(m)) => draw_l1(app, m, buf, area, hits),
            Some(Err(e)) => {
                message(
                    app,
                    buf,
                    area,
                    &[(format!("Could not build the ledger: {e}"), Token::Error)],
                );
            }
            None => {}
        },
        2 => match app.ledger_view.l2.as_deref() {
            Some(Ok(m)) => draw_l2(app, m, buf, area, hits),
            Some(Err(e)) => {
                message(
                    app,
                    buf,
                    area,
                    &[(format!("Could not read postings: {e}"), Token::Error)],
                );
            }
            None => {}
        },
        _ => match app.ledger_view.doc.as_deref() {
            Some(Ok(d)) => draw_l3(app, d, buf, area, hits),
            Some(Err(e)) => {
                message(
                    app,
                    buf,
                    area,
                    &[(format!("Could not read the document: {e}"), Token::Error)],
                );
            }
            None => {}
        },
    }
}

pub fn l1_cells(m: &L1Model, rows: &[usize], diff_cols: bool) -> Vec<Vec<Cell>> {
    rows.iter()
        .map(|&i| {
            let r = &m.rows[i];
            let mut v = vec![
                // `!` differs from Xero; `Δ` (proposed) differs from main on the shown manifest
                match (r.differs, r.main.is_some()) {
                    (true, true) => Cell::new("!Δ", Token::Proposed),
                    (false, true) => Cell::new("Δ", Token::Proposed),
                    (true, false) => Cell::new("!", Token::Warn),
                    _ => Cell::new("", Token::Warn),
                },
                Cell::new(r.code.clone(), Token::Muted),
                Cell::new(r.name.clone(), Token::Text),
                Cell::new(r.class.clone(), Token::Muted),
                Cell::new(amt(r.balance, true), Token::Text),
                Cell::new(amt(r.balance, false), Token::Text),
            ];
            if diff_cols {
                let dc = |d: Option<Decimal>| d.map(ownpurse_core::dr_cr).unwrap_or_default();
                v.push(Cell::new(dc(r.xero), Token::Text));
                v.push(Cell::new(dc(r.difference), Token::Warn));
            }
            v
        })
        .collect()
}

fn draw_l1(app: &App, m: &L1Model, buf: &mut Buffer, area: Rect, hits: &mut Hits) {
    let th = &app.ui.theme;
    let vis = app.l1_visible();
    let diff = app.ui.differs_only;
    let rows = l1_cells(m, &vis, diff);
    // With the filter on, the totals are the whole TB's, not the listed rows' (C11).
    let mut total = vec![
        Cell::new("", Token::Text),
        Cell::new("", Token::Text),
        Cell::new(
            // C11: with any filter on (differs or `/`), the totals are still the whole TB's
            if diff || !app.ui.filter.is_empty() {
                "Total (all accounts)"
            } else {
                "Total"
            },
            Token::Text,
        ),
        Cell::new("", Token::Text),
        Cell::new(fmt(m.debit), Token::Text),
        Cell::new(fmt(m.credit), Token::Text),
    ];
    if diff {
        total.extend([Cell::new("", Token::Text), Cell::new("", Token::Text)]);
    }
    let (cols, steps): (&[ColSpec], &[Step]) = if diff {
        if area.width >= 100 {
            (&L1_DIFF_COLS_WIDE, &L1_DIFF_STEPS)
        } else {
            (&L1_DIFF_COLS, &L1_DIFF_STEPS)
        }
    } else {
        (&L1_COLS, &L1_STEPS)
    };
    // one row under the totals for `balanced ✓` when it doesn't fit after the credit total
    let t_area = Rect {
        height: area.height.saturating_sub(1),
        ..area
    };
    hits.content = Some(t_area);
    let nav = app.nav();
    let grouped = app.source.view.group_tb_by_class;
    let groups = if grouped { l1_groups(m, &vis) } else { vec![] };
    let pos = table::draw(
        th,
        buf,
        t_area,
        &Table {
            cols,
            rows: &rows,
            steps,
            top: nav.l1_top,
            cursor: app.l1_cursor(),
            cursor_col: None,
            footer: Some(&total),
            // the class is the band now; lay mode hides the codes (names lead)
            hidden: match (grouped, app.ui.lay_mode) {
                (true, true) => &[L1_CLASS, L1_CODE],
                (true, false) => &[L1_CLASS],
                (false, true) => &[L1_CODE],
                (false, false) => &[],
            },
            focus: None,
            groups: &groups,
        },
        hits,
    );
    let (word, tok) = if m.debit == m.credit {
        ("balanced ✓".to_string(), Token::Ok)
    } else {
        (
            format!("out of balance by {}", fmt(m.debit - m.credit)),
            Token::Error,
        )
    };
    let word = th.text(&word);
    let w = width(&word) as u16;
    if let Some((cx, cw)) = pos.iter().rev().flatten().next().copied() {
        let after = cx + cw + 2;
        let (x, y) = if after + w < area.right() {
            (after, t_area.bottom().saturating_sub(1))
        } else {
            (area.right().saturating_sub(w + 1), t_area.bottom())
        };
        if y < area.bottom() {
            buf.set_span(x, y, &Span::styled(word, th.fg(tok)), w);
        }
    }
    if vis.is_empty() {
        let msg = if !app.ui.filter.is_empty() {
            "No account matches the filter."
        } else if app.ui.differs_only {
            "No account differs from Xero."
        } else {
            "No balances at this date."
        };
        buf.set_span(
            area.x + 2,
            area.y + 1,
            &Span::styled(msg, th.fg(Token::Muted)),
            area.width.saturating_sub(2),
        );
    }
}

pub fn l2_cells(m: &L2Model, rows: &[usize]) -> Vec<Vec<Cell>> {
    rows.iter()
        .map(|&i| {
            let r = &m.rows[i];
            // a proposed posting carries its glyph before the date, in `proposed` (the row takes the proposed tint)
            let date = match &r.mark {
                Some(g) => Cell::new(format!("{g} {}", r.date), Token::Proposed),
                None => Cell::new(r.date.clone(), Token::Text),
            };
            let money = |d: Decimal, dr: bool| {
                let t = amt(d, dr);
                if r.ghost && !t.is_empty() {
                    format!("{t} (void)")
                } else {
                    t
                }
            };
            vec![
                date,
                Cell::new(r.kind.clone(), Token::Muted)
                    .short(r.kind.chars().take(2).collect::<String>()),
                Cell::new(r.reference.clone(), Token::Muted),
                Cell::new(r.contact.clone(), Token::Text),
                Cell::new(r.description.clone(), Token::Muted),
                Cell::new(
                    money(r.amount, true),
                    if r.ghost { Token::Muted } else { Token::Text },
                ),
                Cell::new(
                    money(r.amount, false),
                    if r.ghost { Token::Muted } else { Token::Text },
                ),
                Cell::new(fmt(r.balance), Token::Text),
            ]
        })
        .collect()
}

fn draw_l2(app: &App, m: &L2Model, buf: &mut Buffer, area: Rect, hits: &mut Hits) {
    let th = &app.ui.theme;
    let rows = l2_cells(m, &app.l2_visible());
    hits.content = Some(area);
    let nav = app.nav();
    table::draw(
        th,
        buf,
        area,
        &Table {
            cols: &L2_COLS,
            rows: &rows,
            steps: &L2_STEPS,
            top: nav.l2_top,
            cursor: app.l2_cursor(),
            cursor_col: None,
            footer: None,
            hidden: &[],
            focus: None,
            groups: &[],
        },
        hits,
    );
    if rows.is_empty() {
        buf.set_span(
            area.x + 2,
            area.y + 1,
            &Span::styled(
                if app.ui.filter.is_empty() {
                    "No postings in this range."
                } else {
                    "No posting matches the filter."
                },
                th.fg(Token::Muted),
            ),
            area.width.saturating_sub(2),
        );
    }
}

/// L3 (shared with IC L3 later, M5): fields | attachments and history. Side by side from 120 columns,
/// split at min(W·60%, 96); stacked below (E7).
pub fn draw_l3(app: &App, d: &DocModel, buf: &mut Buffer, area: Rect, hits: &mut Hits) {
    // a document the shown manifest changes: the ONE generic diff (spec manifests §1), focused on what changed
    if let Some((_, seq, _)) = &d.proposed
        && let Some(doc) = app.mf_diff_of(*seq)
    {
        let ui = crate::manifest_tab::DiffUi {
            seq: *seq,
            ..Default::default()
        };
        crate::views::diff::draw(app, &doc, &ui, buf, area, hits);
        return;
    }
    let th = &app.ui.theme;
    let ell = th.glyphs().ellipsis;
    let split = area.width >= 120;
    let left_w = if split {
        (area.width * 60 / 100).min(96)
    } else {
        area.width
    };
    let left = Rect {
        width: left_w,
        ..area
    };
    let mut y = area.y;
    let put = |buf: &mut Buffer, x: u16, y: u16, w: u16, text: &str, tok: Token| {
        let t = crate::layout::fit_str(&th.text(text), w.saturating_sub(2) as usize, ell);
        buf.set_span(x + 1, y, &Span::styled(t, th.fg(tok)), w.saturating_sub(2));
    };
    let field = |k: &str| {
        d.fields
            .iter()
            .find(|(f, _)| f == k)
            .map(|(_, v)| v.as_str())
    };
    // the document leads with what happened (design pass 2): `9,104.57 received from Larkspur Grocers`
    let kind = field("Type").unwrap_or("");
    let contact = field("Contact").unwrap_or("");
    let total = field("Total").unwrap_or("");
    let verb = match kind.to_lowercase().as_str() {
        k if k.starts_with("receive") => Some("received from"),
        k if k.starts_with("spend") => Some("paid to"),
        k if k.contains("sales invoice") || k.contains("invoice (sales)") => Some("invoiced to"),
        k if k.contains("bill") => Some("billed by"),
        _ => None,
    };
    let lead = match verb {
        Some(v) if !total.is_empty() && !contact.is_empty() => format!("{total}  {v} {contact}"),
        _ => [kind, contact, total]
            .iter()
            .filter(|s| !s.is_empty())
            .copied()
            .collect::<Vec<_>>()
            .join(" · "),
    };
    // the rest folds into one quiet line: type, status, date, reconciliation, amounts
    let mut facts: Vec<String> = Vec::new();
    for (k, v) in &d.fields {
        match k.as_str() {
            "Contact" | "Total" => {}
            "Type" if verb.is_some() => facts.push(v.clone()),
            "Type" => {}
            "Reconciled" => facts.push(if v == "yes" {
                "reconciled ✓".into()
            } else {
                format!("reconciled: {v}")
            }),
            _ => facts.push(v.clone()),
        }
    }
    for (text, tok) in [(lead, Token::Text), (facts.join(" · "), Token::Muted)] {
        if y >= area.bottom() {
            return;
        }
        for l in wrap(&th.text(&text), left.width.saturating_sub(4) as usize)
            .into_iter()
            .take(2)
        {
            if y < area.bottom() {
                put(buf, left.x + 1, y, left.width.saturating_sub(1), &l, tok);
                y += 1;
            }
        }
    }
    y += 1;
    if y < area.bottom() {
        crate::widgets::band(th, buf, Rect::new(left.x, y, left.width, 1), "Lines", None);
        y += 1;
    }
    let lines: Vec<Vec<Cell>> = d
        .lines
        .iter()
        .map(|l| {
            vec![
                Cell::new(l.description.clone(), Token::Text),
                Cell::new(l.account.clone(), Token::Muted),
                Cell::new(l.amount.clone(), Token::Text),
                Cell::new(l.tax.clone(), Token::Muted),
            ]
        })
        .collect();
    let lines_h = (lines.len() as u16 + 1).min(area.bottom().saturating_sub(y));
    if lines_h > 0 {
        table::draw(
            th,
            buf,
            Rect::new(left.x, y, left.width, lines_h),
            &Table {
                cols: &L3_COLS,
                rows: &lines,
                steps: &L3_STEPS,
                top: 0,
                cursor: None,
                cursor_col: None,
                footer: None,
                hidden: &[],
                focus: None,
                groups: &[],
            },
            hits,
        );
        y += lines_h + 1;
    }
    // attachments and history: the right pane, or below
    let (mut ry, rx, rw) = if split {
        let rx = area.x + left_w + 1;
        (area.y, rx, area.right().saturating_sub(rx))
    } else {
        (y, area.x, area.width)
    };
    // (text, token, kind): 0 a line, 1 a band, 2 a details row on the raised surface
    let mut block: Vec<(String, Token, u8)> =
        vec![("Attachments and history".into(), Token::Text, 1)];
    let nothing = d.attachments.is_none() && d.history.is_none();
    if nothing {
        block.push((
            "Not fetched yet. ⌃P → Fetch attachments or Fetch history · 1 call each".into(),
            Token::Muted,
            0,
        ));
    } else {
        match &d.attachments {
            None => block.push((
                "Attachments not fetched yet. ⌃P → Fetch attachments (1 call)".into(),
                Token::Muted,
                0,
            )),
            Some(a) if a.is_empty() => {
                block.push(("No attachments.".into(), Token::Muted, 0));
            }
            Some(a) => {
                for (name, bytes) in a {
                    block.push((
                        format!("{name}  {} bytes", crate::clock::int(*bytes)),
                        Token::Text,
                        0,
                    ));
                }
            }
        }
        match &d.history {
            None => block.push((
                "History not fetched yet. ⌃P → Fetch history (1 call)".into(),
                Token::Muted,
                0,
            )),
            Some(h) => {
                for (when, change, by) in h {
                    block.push((format!("{when}  {change}  {by}"), Token::Text, 0));
                }
            }
        }
    }
    // the details layer (`i`): the document's id, the record version, the Xero link
    if app.ui.details {
        block.push((String::new(), Token::Text, 0));
        block.push(("Details".into(), Token::Text, 1));
        let link = match &d.link {
            Some(l) if l.verified => "o opens it in Xero".to_string(),
            Some(_) => "o opens it in Xero · link not yet verified".to_string(),
            None => "no link template for this document type (links, per: document)".to_string(),
        };
        let local = d
            .local
            .as_ref()
            .map_or("not in the local record".into(), |(v, at)| {
                format!(
                    "local v{v} · synced {} UTC",
                    at.get(..16).unwrap_or(at).replace('T', " ")
                )
            });
        for (k, v) in [
            ("document", format!("{} {}", d.source, d.id)),
            ("record", local),
            ("Xero link", link),
        ] {
            block.push((format!("{k:<12}{v}"), Token::Text2, 2));
        }
    }
    for (text, tok, kind) in &block {
        for l in wrap(&th.text(text), rw.saturating_sub(4) as usize) {
            if ry >= area.bottom() {
                return;
            }
            match kind {
                1 => {
                    let note = (l == "Details").then(|| ("i hide".to_string(), Token::Muted));
                    crate::widgets::band(th, buf, Rect::new(rx, ry, rw, 1), &l, note);
                }
                2 => {
                    let raised = th.raised();
                    crate::widgets::fill(buf, Rect::new(rx, ry, rw, 1), raised);
                    let (k, v) = l.split_at(l.len().min(12));
                    buf.set_line(
                        rx + 2,
                        ry,
                        &ratatui::text::Line::from(vec![
                            Span::styled(k.to_string(), raised.patch(th.fg(Token::Muted))),
                            Span::styled(v.to_string(), raised.patch(th.fg(*tok))),
                        ]),
                        rw.saturating_sub(3),
                    );
                }
                _ => {
                    buf.set_span(
                        rx + 2,
                        ry,
                        &Span::styled(l, th.fg(*tok)),
                        rw.saturating_sub(3),
                    );
                }
            }
            ry += 1;
        }
    }
}

/// The trial balance's class order (assets, liabilities, equity, revenue, expenses, then anything else).
pub fn class_rank(class: &str) -> usize {
    match class.to_uppercase().as_str() {
        "ASSET" | "ASSETS" => 0,
        "LIABILITY" | "LIABILITIES" => 1,
        "EQUITY" => 2,
        "REVENUE" | "INCOME" => 3,
        "EXPENSE" | "EXPENSES" => 4,
        _ => 5,
    }
}

/// A class band's label: `Assets`, `Liabilities`, …
pub fn class_label(class: &str) -> String {
    match class_rank(class) {
        0 => "Assets".into(),
        1 => "Liabilities".into(),
        2 => "Equity".into(),
        3 => "Revenue".into(),
        4 => "Expenses".into(),
        _ => {
            let mut c = class.to_lowercase();
            if let Some(f) = c.get_mut(0..1) {
                f.make_ascii_uppercase();
            }
            c
        }
    }
}

/// The class bands over the visible L1 rows: (position, label, net). The net is Dr as is, Cr in
/// parentheses.
pub fn l1_groups(m: &L1Model, vis: &[usize]) -> Vec<(usize, String, String)> {
    let mut out: Vec<(usize, String, String)> = Vec::new();
    let mut k = 0;
    while k < vis.len() {
        let rank = class_rank(&m.rows[vis[k]].class);
        let start = k;
        let mut net = Decimal::ZERO;
        while k < vis.len() && class_rank(&m.rows[vis[k]].class) == rank {
            net += m.rows[vis[k]].balance;
            k += 1;
        }
        let text = if net < Decimal::ZERO {
            format!("({})", fmt(-net))
        } else {
            fmt(net)
        };
        out.push((start, class_label(&m.rows[vis[start]].class), text));
    }
    out
}
