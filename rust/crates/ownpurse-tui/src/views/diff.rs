//! The ONE generic document diff (design spec manifests §1): any entity, from the normalised before/after
//! (`core::manifest::fields`). A title (`~ / + new / ⊘ void`, the counts, the entry and actor), field rows
//! (`+ − ~` markers; changed text with word highlights, money with Δ, enums and dates old → new), the lines
//! table (changed cells inline, or `− old` / `+ new` rows when they don't fit; balanced ✓ / ✗), collapsing
//! (`z`), the raw patch (`J`), and each issue under the field it concerns. Draw only; the model is prepared.

use crate::app::App;
use crate::hit::{Hits, Target};
use crate::layout::width;
use crate::manifest_tab::{DiffDoc, DiffUi, hidden_count, visible_fields, visible_lines};
use crate::theme::Theme;
use crate::theme_tokens::Token;
use crate::widgets::{band, fill, put};
use ownpurse_core::manifest::check::Issue;
use ownpurse_core::manifest::fields::{FieldRow, LineCell, LineRow, Mark, ValueKind, entity_label};
use ownpurse_core::manifest::stage::EntryStatus;
use ownpurse_core::money::fmt;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::Span;
use rust_decimal::Decimal;

const LABEL_W: usize = 14;

fn struck(th: &Theme) -> Style {
    th.fg(Token::Muted).add_modifier(Modifier::CROSSED_OUT)
}

fn money_text(s: &str) -> String {
    match ownpurse_core::money::parse(s) {
        Ok(d) if !s.is_empty() => fmt(d),
        _ => s.to_string(),
    }
}

fn signed(d: Decimal) -> String {
    if d > Decimal::ZERO {
        format!("+{}", fmt(d))
    } else if d < Decimal::ZERO {
        format!("−{}", fmt(-d))
    } else {
        "0.00".into()
    }
}

fn short_id(id: &str) -> String {
    if id.chars().count() > 10 && !id.starts_with("new:") {
        format!("{}…", id.chars().take(4).collect::<String>())
    } else {
        id.to_string()
    }
}

/// Word-level spans of a changed text: removed words struck in `text.3`, inserted words in `proposed` on
/// the proposed tint, the rest in `text.1`.
pub fn word_spans(th: &Theme, old: &str, new: &str) -> Vec<Span<'static>> {
    let d = similar::TextDiff::from_words(old, new);
    // without colour the strike-through and the tint don't show: removed words read `[-…-]`, inserted `{+…+}`
    let plain = matches!(th.mode, crate::theme::Mode::NoColor);
    let mut out: Vec<Span<'static>> = vec![];
    let mut last: Option<similar::ChangeTag> = None;
    for c in d.iter_all_changes() {
        let t = c.value().to_string();
        let tag = c.tag();
        // a removed word followed straight by an inserted one: keep them apart
        if last == Some(similar::ChangeTag::Delete) && tag == similar::ChangeTag::Insert && !plain {
            out.push(Span::styled(" ".to_string(), th.fg(Token::Text)));
        }
        let (text, st) = match tag {
            similar::ChangeTag::Equal => (t, th.fg(Token::Text)),
            similar::ChangeTag::Delete if plain && !t.trim().is_empty() => {
                (format!("[-{t}-]"), th.fg(Token::Muted))
            }
            similar::ChangeTag::Insert if plain && !t.trim().is_empty() => {
                (format!("{{+{t}+}}"), th.fg(Token::Text))
            }
            similar::ChangeTag::Delete if plain => (String::new(), th.fg(Token::Muted)),
            similar::ChangeTag::Insert if plain => (t, th.fg(Token::Text)),
            similar::ChangeTag::Delete => (t, struck(th)),
            similar::ChangeTag::Insert => {
                (t, th.bg(Token::ProposedTint).patch(th.fg(Token::Proposed)))
            }
        };
        out.push(Span::styled(text, st));
        last = Some(tag);
    }
    out
}

/// The value spans of a field row.
fn value_spans(th: &Theme, f: &FieldRow) -> Vec<Span<'static>> {
    let arrow = format!(" {} ", th.glyphs().arrow);
    let disp = |s: &Option<String>| -> String {
        let s = s.clone().unwrap_or_default();
        if f.kind == ValueKind::Money {
            money_text(&s)
        } else {
            s
        }
    };
    match f.mark {
        Mark::Same => vec![Span::styled(disp(&f.new), th.fg(Token::Text2))],
        Mark::Added => vec![Span::styled(disp(&f.new), th.fg(Token::Text))],
        Mark::Removed => vec![Span::styled(disp(&f.old), struck(th))],
        Mark::Changed => match f.kind {
            ValueKind::Text => word_spans(
                th,
                f.old.as_deref().unwrap_or(""),
                f.new.as_deref().unwrap_or(""),
            ),
            ValueKind::Money => {
                let mut v = vec![
                    Span::styled(disp(&f.old), struck(th)),
                    Span::styled(arrow, th.fg(Token::Muted)),
                    Span::styled(disp(&f.new), th.fg(Token::Text)),
                ];
                if let Some(d) = f.delta() {
                    v.push(Span::styled(
                        format!("  Δ {}", signed(d)),
                        th.fg(Token::Proposed),
                    ));
                }
                v
            }
            _ => vec![
                Span::styled(disp(&f.old), struck(th)),
                Span::styled(arrow, th.fg(Token::Muted)),
                Span::styled(disp(&f.new), th.fg(Token::Text)),
            ],
        },
    }
}

fn marker_span(th: &Theme, m: Mark) -> Span<'static> {
    let g = match m {
        Mark::Removed if th.glyphs().arrow == "->" => "-",
        _ => m.glyph(),
    };
    Span::styled(
        format!("{g} "),
        if m == Mark::Same {
            th.fg(Token::Muted)
        } else {
            th.fg(Token::Proposed)
        },
    )
}

fn issue_spans(th: &Theme, i: &Issue) -> Vec<Span<'static>> {
    match (&i.ack, i.blocking()) {
        (_, true) => vec![Span::styled(
            format!("✗ {}", i.message),
            th.fg(Token::Error),
        )],
        (Some(a), false) => vec![Span::styled(
            format!(
                "✓ acknowledged by {} {} · {}",
                a.actor.trim_start_matches("agent:"),
                a.at.get(11..16).unwrap_or(""),
                i.message
            ),
            th.fg(Token::Muted),
        )],
        (None, false) => vec![
            Span::styled(format!("! {}", i.message), th.fg(Token::Warn)),
            Span::styled("  a acknowledge".to_string(), th.fg(Token::Text4)),
        ],
    }
}

struct Painter<'a> {
    th: &'a Theme,
    buf: &'a mut Buffer,
    area: Rect,
    y: u16,
    /// The first row to draw (scrolling keeps the focused row on screen).
    skip: usize,
    n: usize,
}

impl Painter<'_> {
    fn full(&self) -> bool {
        self.y >= self.area.bottom()
    }
    /// One logical row; `focus` paints it in the selection colour. Returns the y it drew at (None: skipped).
    fn row(&mut self, spans: Vec<Span<'static>>, focus: bool) -> Option<u16> {
        let i = self.n;
        self.n += 1;
        if i < self.skip || self.full() {
            return None;
        }
        let y = self.y;
        if focus {
            let sel = self.th.selection();
            fill(self.buf, Rect::new(self.area.x, y, self.area.width, 1), sel);
            self.buf.set_span(
                self.area.x,
                y,
                &Span::styled(self.th.glyphs().cursor, self.th.fg(Token::Accent)),
                1,
            );
        }
        let spans = spans
            .into_iter()
            .map(|s| {
                let st = if focus {
                    s.style.patch(self.th.selection())
                } else {
                    s.style
                };
                Span::styled(self.th.text(&s.content), st)
            })
            .collect();
        put(
            self.buf,
            self.area.x + 2,
            y,
            spans,
            self.area.width.saturating_sub(3),
            self.th.glyphs().ellipsis,
        );
        self.y += 1;
        Some(y)
    }
    fn band(&mut self, label: &str, right: Option<(String, Token)>) {
        let i = self.n;
        self.n += 1;
        if i < self.skip || self.full() {
            return;
        }
        band(
            self.th,
            self.buf,
            Rect::new(self.area.x, self.y, self.area.width, 1),
            label,
            right,
        );
        self.y += 1;
    }
}

/// The issues on a path (and, for a line row, on its fields), excluding the document-level ones.
fn issues_on<'a>(doc: &'a DiffDoc, path: &str, line: bool) -> Vec<&'a Issue> {
    doc.issues
        .iter()
        .filter(|i| {
            !i.path.is_empty()
                && (i.path == path || (line && i.path.starts_with(&format!("{path}/"))))
        })
        .collect()
}

/// The title's kind word and colour.
pub fn title(doc: &DiffDoc) -> (String, String) {
    let ent = entity_label(&doc.entity);
    match doc.kind {
        "create" => ("+".into(), format!("new {ent} · {} · not in Xero", doc.org)),
        "void" => (
            "⊘".into(),
            format!("void · {ent} · {} · {}", doc.org, short_id(&doc.id)),
        ),
        _ => (
            "~".into(),
            format!("{} · {} · {}", cap(&ent), doc.org, short_id(&doc.id)),
        ),
    }
}

fn cap(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
        None => String::new(),
    }
}

/// The diff's crumb (`~ manual journal mj-401`).
pub fn crumb(doc: &DiffDoc) -> String {
    let (g, _) = title(doc);
    format!("{g} {} {}", entity_label(&doc.entity), short_id(&doc.id))
}

pub fn draw(app: &App, doc: &DiffDoc, ui: &DiffUi, buf: &mut Buffer, area: Rect, hits: &mut Hits) {
    let th = &app.ui.theme;
    if area.height < 3 {
        return;
    }
    // the title block: kind, entity, org, id · the counts · entry n · H/A actor
    let (g, t) = title(doc);
    let counts = match doc.kind {
        "create" => format!(
            "{} fields · {} lines",
            doc.diff.fields.len(),
            doc.diff.lines.as_ref().map_or(0, |l| l.rows.len())
        ),
        _ => format!("changed: {}", doc.diff.summary()),
    };
    let who = match doc.actor.strip_prefix("agent:") {
        Some(a) => format!("entry {} · A {a}", doc.seq),
        None => format!("entry {} · H", doc.seq),
    };
    let mut left = vec![
        Span::styled(
            format!("{g} "),
            th.fg(Token::Proposed).add_modifier(Modifier::BOLD),
        ),
        Span::styled(t, th.heading()),
    ];
    if doc.reconciled {
        // shown even when the fields are collapsed (review 6)
        left.push(Span::styled(
            if th.ascii {
                "  [reconciled]".to_string()
            } else {
                "  reconciled".to_string()
            },
            th.fg(Token::Warn),
        ));
    }
    left.push(Span::styled(format!("   {counts}"), th.fg(Token::Muted)));
    let left: Vec<Span<'static>> = left
        .into_iter()
        .map(|s| Span::styled(th.text(&s.content), s.style))
        .collect();
    let who = th.text(&who);
    let lw: usize = left.iter().map(|s| width(&s.content)).sum();
    put(
        buf,
        area.x + 2,
        area.y,
        left,
        area.width.saturating_sub(3),
        th.glyphs().ellipsis,
    );
    hits.anchors.panel(
        "diff-title",
        Rect::new(
            area.x + 2,
            area.y,
            (lw as u16).min(area.width.saturating_sub(3)),
            1,
        ),
    );
    let ww = width(&who);
    if (area.width as usize) > lw + ww + 6 {
        buf.set_span(
            area.right().saturating_sub(ww as u16 + 1),
            area.y,
            &Span::styled(who, th.fg(Token::Muted)),
            ww as u16,
        );
    }
    let mut y = area.y + 1;
    let mut extra: Vec<(Vec<Span<'static>>, Option<Style>)> = vec![];
    if doc.kind == "void" {
        extra.push((
            vec![Span::styled(
                "this document would be voided in Xero (reversible only by re-creating it)"
                    .to_string(),
                th.fg(Token::Proposed),
            )],
            Some(th.bg(Token::ProposedTint)),
        ));
    }
    let mut via_at = None;
    if let Some(v) = &doc.via {
        via_at = Some(extra.len());
        extra.push((vec![Span::styled(v.clone(), th.fg(Token::Proposed))], None));
    }
    if let Some(c) = &doc.conflict {
        let why: Vec<String> = c
            .reasons
            .iter()
            .map(|r| format!("{} {} → {}", r.what, r.old, r.new))
            .collect();
        extra.push((
            vec![Span::styled(
                format!(
                    "≠ not applied: main moved since this entry was made ({})",
                    why.join("; ")
                ),
                th.fg(Token::Error),
            )],
            None,
        ));
    }
    match &doc.status {
        EntryStatus::Landed(t) => extra.push((
            vec![Span::styled(
                format!(
                    "✓ landed{} · verifying (the marks stay until the read-back agrees)",
                    t.as_ref().map(|t| format!(" · {}", short_id(t))).unwrap_or_default()
                ),
                th.fg(Token::Ok),
            )],
            None,
        )),
        EntryStatus::Posted => extra.push((
            vec![Span::styled("✓ posted · the read-back equals this after".to_string(), th.fg(Token::Muted))],
            None,
        )),
        EntryStatus::Rejected(m) => extra.push((
            vec![Span::styled(format!("✗ rejected: {m}"), th.fg(Token::Error))],
            None,
        )),
        EntryStatus::Uncertain => extra.push((
            vec![Span::styled(
                "? uncertain: no response · do not retry (the read-back decides: ownpurse write reconcile)".to_string(),
                th.fg(Token::Warn),
            )],
            None,
        )),
        _ => {}
    }
    for i in doc.issues.iter().filter(|i| i.path.is_empty()) {
        extra.push((issue_spans(th, i), None));
    }
    // the bands wrap to at most two lines (review 3): what they say is never cut off unread
    for (k, (spans, bg)) in extra.into_iter().enumerate() {
        let room = (area.width as usize).saturating_sub(4).max(10);
        let spans: Vec<Span<'static>> = spans
            .into_iter()
            .map(|s| Span::styled(th.text(&s.content), s.style))
            .collect();
        for line in two_lines(spans, room) {
            if y >= area.bottom() {
                return;
            }
            if let Some(bg) = bg {
                fill(buf, Rect::new(area.x, y, area.width, 1), bg);
            }
            put(
                buf,
                area.x + 2,
                y,
                line,
                area.width.saturating_sub(3),
                th.glyphs().ellipsis,
            );
            if via_at == Some(k) {
                hits.anchors
                    .panel("via", Rect::new(area.x, y, area.width, 1));
            }
            y += 1;
        }
    }
    let body = Rect::new(area.x, y, area.width, area.bottom().saturating_sub(y));
    if ui.patch {
        // `J`: the raw RFC 6902 ops, one per line
        let mut p = Painter {
            th,
            buf,
            area: body,
            y: body.y,
            skip: 0,
            n: 0,
        };
        p.band("Patch (RFC 6902)", Some(("J back".into(), Token::Muted)));
        for l in &doc.patch {
            p.row(vec![Span::styled(l.clone(), th.fg(Token::Text2))], false);
        }
        return;
    }
    let new = doc.kind == "create";
    let paths = doc.paths(ui.expanded);
    let focus_i = ui.focus.as_ref().and_then(|f| {
        paths
            .iter()
            .position(|p| p == f || f.starts_with(&format!("{p}/")))
    });
    // keep the focused row on screen: estimate its logical row and skip from there
    let skip = focus_i.map_or(0, |i| (i + 2).saturating_sub(body.height as usize / 2));
    let mut p = Painter {
        th,
        buf,
        area: body,
        y: body.y,
        skip,
        n: 0,
    };
    let mut row_i = 0usize;
    let mut drawn: Vec<(u16, usize)> = vec![];
    for r in visible_fields(&doc.diff, ui.expanded, new) {
        let Some(f) = r else {
            let hidden = hidden_count(&doc.diff, ui.expanded, new);
            p.row(
                vec![Span::styled(
                    format!(
                        "  … {hidden} unchanged field{} · z to expand",
                        if hidden == 1 { "" } else { "s" }
                    ),
                    th.fg(Token::Text4),
                )],
                false,
            );
            continue;
        };
        let focus = focus_i == Some(row_i);
        let mut spans = vec![marker_span(th, f.mark)];
        let label = format!(
            "{:<w$}",
            crate::layout::fit_str(&f.label, LABEL_W, "…"),
            w = LABEL_W
        );
        spans.push(Span::styled(
            format!("{label} "),
            if f.mark == Mark::Same || !f.known {
                th.fg(Token::Muted)
            } else {
                th.fg(Token::Text2)
            },
        ));
        let vals = value_spans(th, f);
        // long text wraps under the label (at most 4 lines, then `… z to expand`)
        let vw: usize = vals.iter().map(|s| width(&s.content)).sum();
        let room = (body.width as usize).saturating_sub(LABEL_W + 6);
        if f.kind == ValueKind::Text && vw > room && room > 10 {
            if let Some(y) = p.row(spans, focus) {
                drawn.push((y, row_i));
            }
            let wrapped = wrap_spans(vals, room);
            let cap = if ui.expanded { usize::MAX } else { 4 };
            for (k, line) in wrapped.iter().enumerate() {
                if k == cap {
                    p.row(
                        vec![Span::styled(
                            format!("{:w$}… z to expand", "", w = LABEL_W + 2),
                            th.fg(Token::Text4),
                        )],
                        false,
                    );
                    break;
                }
                let mut l = vec![Span::raw(format!("{:w$}", "", w = LABEL_W + 2))];
                l.extend(line.clone());
                p.row(l, focus);
            }
        } else {
            spans.extend(vals);
            if let Some(y) = p.row(spans, focus) {
                drawn.push((y, row_i));
            }
        }
        for i in issues_on(doc, &f.path, false) {
            let pad = LABEL_W + 3;
            for line in two_lines(
                issue_spans(th, i),
                (body.width as usize).saturating_sub(pad + 4).max(10),
            ) {
                let mut l = vec![Span::raw(format!("{:w$}", "", w = pad))];
                l.extend(line);
                if let Some(y) = p.row(l, false) {
                    hits.anchors
                        .panel("check", Rect::new(area.x, y, area.width, 1));
                }
            }
        }
        row_i += 1;
    }
    if let Some(lines) = &doc.diff.lines {
        p.band("Lines", None);
        // column widths: account, description (flex), and the two amounts
        let amt_w = 14usize;
        let acct_w = 16usize;
        let desc_w = (body.width as usize)
            .saturating_sub(2 + 2 + acct_w + 2 * amt_w + 6)
            .max(8);
        let widths = [acct_w, desc_w, amt_w, amt_w];
        let header: Vec<Span<'static>> = std::iter::once(Span::raw("  ".to_string()))
            .chain(lines.columns.iter().enumerate().map(|(i, c)| {
                let w = widths[i.min(3)];
                let t = if c.kind == ValueKind::Money {
                    format!("{:>w$} ", c.label)
                } else {
                    format!("{:<w$} ", c.label)
                };
                Span::styled(t, th.fg(Token::Muted))
            }))
            .collect();
        p.row(header, false);
        for r in visible_lines(&lines.rows, ui.expanded, new) {
            let Some(r) = r else {
                let hidden = lines.rows.len()
                    - visible_lines(&lines.rows, ui.expanded, new)
                        .iter()
                        .filter(|x| x.is_some())
                        .count();
                p.row(
                    vec![Span::styled(
                        format!("  … {hidden} unchanged lines · z to expand"),
                        th.fg(Token::Text4),
                    )],
                    false,
                );
                continue;
            };
            let focus = focus_i == Some(row_i);
            let void = doc.kind == "void";
            let fits = line_fits(r, &widths);
            if r.mark == Mark::Changed && !fits {
                // two rows: − old values (struck), then + new values with the changed cells highlighted
                let old = line_spans(th, r, &widths, Side::Old);
                let mut a = vec![Span::styled("− ".to_string(), th.fg(Token::Proposed))];
                a.extend(old);
                if let Some(y) = p.row(a, focus) {
                    drawn.push((y, row_i));
                }
                let mut b = vec![Span::styled("+ ".to_string(), th.fg(Token::Proposed))];
                b.extend(line_spans(th, r, &widths, Side::New));
                p.row(b, focus);
            } else {
                let mut a = vec![marker_span(th, if void { Mark::Removed } else { r.mark })];
                a.extend(line_spans(
                    th,
                    r,
                    &widths,
                    if void || r.mark == Mark::Removed {
                        Side::Struck
                    } else {
                        Side::Inline
                    },
                ));
                if let Some(y) = p.row(a, focus) {
                    drawn.push((y, row_i));
                }
            }
            for i in issues_on(doc, &r.path, true) {
                for line in two_lines(
                    issue_spans(th, i),
                    (body.width as usize).saturating_sub(8).max(10),
                ) {
                    let mut l = vec![Span::raw("    ".to_string())];
                    l.extend(line);
                    if let Some(y) = p.row(l, false) {
                        hits.anchors
                            .panel("check", Rect::new(area.x, y, area.width, 1));
                    }
                }
            }
            row_i += 1;
        }
        // totals
        let t = &lines.totals;
        let mut tot = vec![Span::raw("  ".to_string())];
        if t.removed {
            tot.push(Span::styled(
                format!(
                    "− would remove: debits {} · credits {}",
                    fmt(t.debits),
                    fmt(t.credits)
                ),
                struck(th),
            ));
        } else {
            tot.push(Span::styled(
                format!("debits {} · credits {} ", fmt(t.debits), fmt(t.credits)),
                th.fg(Token::Text2),
            ));
            match t.imbalance {
                Some(d) if d.is_zero() => {
                    tot.push(Span::styled(th.glyphs().ok.to_string(), th.fg(Token::Ok)))
                }
                Some(d) => tot.push(Span::styled(
                    format!("✗ unbalanced: Δ {}", fmt(d.abs())),
                    th.fg(Token::Error),
                )),
                None => {}
            }
        }
        if t.imbalance.is_none() && !t.removed {
            tot = vec![
                Span::raw("  ".to_string()),
                Span::styled(
                    format!("total {}", fmt(t.debits - t.credits)),
                    th.fg(Token::Text2),
                ),
            ];
        }
        p.row(tot, false);
    }
    // a click on a drawn row focuses it (the row index is its position in `paths`)
    for (y, i) in drawn {
        let r = Rect::new(area.x, y, area.width, 1);
        hits.add(r, Target::Row(i));
        // the field or line as an overlay anchor: `diff:/JournalLines/n1`, `diff:/Date`
        if let Some(path) = paths.get(i) {
            hits.anchors.diff(path, r);
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
enum Side {
    Old,
    New,
    Inline,
    Struck,
}

fn cell_text(c: &LineCell, kind: ValueKind, side: Side) -> (Option<String>, Option<String>) {
    let f = |s: &Option<String>| {
        s.as_ref().map(|t| {
            if kind == ValueKind::Money {
                money_text(t)
            } else {
                t.clone()
            }
        })
    };
    match side {
        Side::Old | Side::Struck => (f(&c.old).or(f(&c.new)), None),
        Side::New => (None, f(&c.new)),
        Side::Inline => (f(&c.old), f(&c.new)),
    }
}

fn line_fits(r: &LineRow, widths: &[usize; 4]) -> bool {
    r.cells.iter().enumerate().all(|(i, c)| {
        let w = widths[i.min(3)];
        if !c.changed() {
            return true;
        }
        let o = c.old.as_deref().unwrap_or("");
        let n = c.new.as_deref().unwrap_or("");
        width(o) + width(n) + 3 <= w
    })
}

fn line_spans(th: &Theme, r: &LineRow, widths: &[usize; 4], side: Side) -> Vec<Span<'static>> {
    let mut out = vec![];
    let arrow = format!(" {} ", th.glyphs().arrow);
    for (i, c) in r.cells.iter().enumerate() {
        let w = widths[i.min(3)];
        let kind = if i >= 2 {
            ValueKind::Money
        } else {
            ValueKind::Text
        };
        let right = kind == ValueKind::Money;
        let pad = |s: &str, w: usize| -> String {
            let s = crate::layout::fit_str(s, w, "…");
            if right {
                format!("{s:>w$}")
            } else {
                format!("{s:<w$}")
            }
        };
        let (old, new) = cell_text(c, kind, side);
        match side {
            Side::Struck => out.push(Span::styled(
                pad(&old.unwrap_or_default(), w) + " ",
                struck(th),
            )),
            Side::Old => {
                let st = if c.changed() {
                    struck(th)
                } else {
                    th.fg(Token::Muted)
                };
                out.push(Span::styled(pad(&old.unwrap_or_default(), w) + " ", st));
            }
            Side::New => {
                let st = if c.changed() {
                    th.bg(Token::ProposedTint).patch(th.fg(Token::Proposed))
                } else {
                    th.fg(Token::Text)
                };
                out.push(Span::styled(pad(&new.unwrap_or_default(), w) + " ", st));
            }
            Side::Inline => {
                if c.changed() && r.mark == Mark::Changed {
                    let o = old.unwrap_or_default();
                    let n = new.unwrap_or_default();
                    let used = width(&o) + width(&arrow) + width(&n);
                    let fill_w = w.saturating_sub(used);
                    if right {
                        out.push(Span::raw(" ".repeat(fill_w)));
                    }
                    out.push(Span::styled(o, struck(th)));
                    out.push(Span::styled(arrow.clone(), th.fg(Token::Muted)));
                    out.push(Span::styled(n, th.fg(Token::Text)));
                    out.push(Span::raw(if right {
                        " ".to_string()
                    } else {
                        " ".repeat(fill_w + 1)
                    }));
                } else {
                    let st = if r.mark == Mark::Same {
                        th.fg(Token::Text2)
                    } else {
                        th.fg(Token::Text)
                    };
                    out.push(Span::styled(
                        pad(&new.or(old).unwrap_or_default(), w) + " ",
                        st,
                    ));
                }
            }
        }
    }
    out
}

/// Wrap spans to `w` cells per line (word boundaries where possible).
fn wrap_spans(spans: Vec<Span<'static>>, w: usize) -> Vec<Vec<Span<'static>>> {
    let mut lines: Vec<Vec<Span<'static>>> = vec![vec![]];
    let mut used = 0usize;
    for s in spans {
        for word in s.content.split_inclusive(' ') {
            let ww = width(word);
            if used + ww > w && used > 0 {
                lines.push(vec![]);
                used = 0;
            }
            lines
                .last_mut()
                .expect("one line")
                .push(Span::styled(word.to_string(), s.style));
            used += ww;
        }
    }
    lines
}

/// Wrap to at most two lines; the second ends with `…` when there is more (the Issues detail shows it all).
fn two_lines(spans: Vec<Span<'static>>, w: usize) -> Vec<Vec<Span<'static>>> {
    let mut lines = wrap_spans(spans, w);
    if lines.len() > 2 {
        lines.truncate(2);
        if let Some(last) = lines.last_mut() {
            let style = last.last().map(|s| s.style).unwrap_or_default();
            // make room for the ellipsis
            let mut used: usize = last.iter().map(|s| width(&s.content)).sum();
            while used + 2 > w {
                match last.pop() {
                    Some(s) => used -= width(&s.content),
                    None => break,
                }
            }
            last.push(Span::styled(" …".to_string(), style));
        }
    }
    lines
}
