//! A table with declared column kinds and a declared degrade order (spec §4.3). The table's `steps`
//! (hide a column, switch a column to its short form, shrink a text column to a floor) run in order until
//! the natural widths fit; then text columns shorten with `…` to their `min`, then text and id columns hide,
//! lowest priority first. Amount and status columns never truncate or hide (V8). The cursor row has the
//! `▌` gutter (a glyph, not only paint; V12); the cursor cell (when the table has one) is underlined.

use crate::hit::{Hits, Target};
use crate::layout::{ColSpec, Kind, fit_columns, pad, width};
use crate::theme::Theme;
use crate::theme_tokens::Token;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Modifier;
use ratatui::text::Span;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Cell {
    pub text: String,
    /// The column's short form (`2 differ` for `2 accounts differ`), used after a `Step::Short`.
    pub short: Option<String>,
    pub token: Token,
}

impl Cell {
    pub fn new(text: impl Into<String>, token: Token) -> Cell {
        Cell {
            text: text.into(),
            short: None,
            token,
        }
    }
    pub fn short(mut self, s: impl Into<String>) -> Cell {
        self.short = Some(s.into());
        self
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Step {
    Hide(usize),
    Short(usize),
    /// Shrink a text column to at most this many cells (with `…`).
    Shrink(usize, usize),
}

pub const GAP: usize = 2;

/// The fitted layout: per column its width (None = hidden) and whether it shows its short form.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Fitted {
    pub widths: Vec<Option<usize>>,
    pub short: Vec<bool>,
}

fn title_of(c: &ColSpec, short: bool) -> &'static str {
    if short && !c.short_title.is_empty() {
        c.short_title
    } else {
        c.title
    }
}

fn cell_text(c: &Cell, short: bool) -> &str {
    if short {
        c.short.as_deref().unwrap_or(&c.text)
    } else {
        &c.text
    }
}

/// Fit the columns to `avail` cells (after the 2-cell gutter).
pub fn fit(
    th: &Theme,
    cols: &[ColSpec],
    rows: &[Vec<Cell>],
    steps: &[Step],
    avail: usize,
) -> Fitted {
    let refs: Vec<&[Cell]> = rows.iter().map(Vec::as_slice).collect();
    fit_marked(th, cols, &refs, steps, avail, None, &[])
}

/// `fit`, reserving `mark` = (column, cells) for the cursor-cell mark.
pub fn fit_marked(
    th: &Theme,
    cols: &[ColSpec],
    rows: &[&[Cell]],
    steps: &[Step],
    avail: usize,
    mark: Option<(usize, usize)>,
    pre_hidden: &[usize],
) -> Fitted {
    let n = cols.len();
    let mut hidden: Vec<bool> = (0..n).map(|i| pre_hidden.contains(&i)).collect();
    let mut short = vec![false; n];
    let mut cap: Vec<Option<usize>> = vec![None; n];
    let natural = |short: &[bool], cap: &[Option<usize>]| -> Vec<usize> {
        (0..n)
            .map(|i| {
                let w = rows
                    .iter()
                    .map(|r| width(&th.text(cell_text(&r[i], short[i]))))
                    .chain([width(title_of(&cols[i], short[i]))])
                    .max()
                    .unwrap_or(0);
                let w = w + mark.filter(|(c, _)| *c == i).map_or(0, |(_, m)| m);
                cap[i].map_or(w, |c| w.min(c.max(width(title_of(&cols[i], short[i])))))
            })
            .collect()
    };
    let total = |nat: &[usize], hidden: &[bool]| {
        let vis: Vec<usize> = (0..n).filter(|&i| !hidden[i]).map(|i| nat[i]).collect();
        vis.iter().sum::<usize>() + GAP * vis.len().saturating_sub(1)
    };
    let mut nat = natural(&short, &cap);
    for step in steps {
        if total(&nat, &hidden) <= avail {
            break;
        }
        match *step {
            Step::Hide(i) => hidden[i] = true,
            Step::Short(i) => short[i] = true,
            Step::Shrink(i, c) => cap[i] = Some(c),
        }
        nat = natural(&short, &cap);
    }
    let visible: Vec<usize> = (0..n).filter(|&i| !hidden[i]).collect();
    let vcols: Vec<ColSpec> = visible.iter().map(|&i| cols[i]).collect();
    let vnat: Vec<usize> = visible.iter().map(|&i| nat[i]).collect();
    let fitted = fit_columns(&vcols, &vnat, avail, GAP);
    let mut widths = vec![None; n];
    for (k, &i) in visible.iter().enumerate() {
        widths[i] = fitted[k];
    }
    Fitted { widths, short }
}

pub struct Table<'a> {
    pub cols: &'a [ColSpec],
    pub rows: &'a [Vec<Cell>],
    pub steps: &'a [Step],
    pub top: usize,
    pub cursor: Option<usize>,
    /// The cursor cell's column (underlined on the cursor row).
    pub cursor_col: Option<usize>,
    /// A totals row pinned at the bottom (bold, a `line` rule above), included in the width fit.
    pub footer: Option<&'a [Cell]>,
    /// Columns hidden before fitting (the details layer's columns while `i` is off).
    pub hidden: &'a [usize],
    /// The quiet second line under the selected row (design pass 2 tier 2, `text.3`), when there is room.
    pub focus: Option<String>,
    /// Group bands (design pass 2: trial balance classes): before the row at this position, a band with the
    /// label and its net (right-aligned in `text.3`). The cursor stays on screen: the draw starts lower
    /// when the bands would push it off.
    pub groups: &'a [(usize, String, String)],
}

/// Where each visible column was drawn: (x, width).
pub type Columns = Vec<Option<(u16, u16)>>;

/// Draw the header row and the rows from `top`. Returns the column positions.
pub fn draw(th: &Theme, buf: &mut Buffer, area: Rect, t: &Table, hits: &mut Hits) -> Columns {
    let mut pos: Columns = vec![None; t.cols.len()];
    if area.height == 0 {
        return pos;
    }
    let g = th.glyphs();
    let avail = (area.width as usize).saturating_sub(2);
    // The cursor cell shows a `▸` mark where colour can't carry it (NO_COLOR, ASCII; V12).
    let marked = matches!(th.mode, crate::theme::Mode::NoColor) || th.ascii;
    let mark = th.text("▸ ");
    let mut all: Vec<&[Cell]> = t.rows.iter().map(Vec::as_slice).collect();
    all.extend(t.footer);
    let f = fit_marked(
        th,
        t.cols,
        &all,
        t.steps,
        avail,
        t.cursor_col.filter(|_| marked).map(|c| (c, width(&mark))),
        t.hidden,
    );
    let right = |c: &ColSpec| matches!(c.kind, Kind::Amount);
    let mut x = area.x + 2;
    for (i, c) in t.cols.iter().enumerate() {
        let Some(cw) = f.widths[i] else { continue };
        let title = pad(title_of(c, f.short[i]), cw, g.ellipsis, right(c));
        buf.set_span(
            x,
            area.y,
            &Span::styled(title, th.fg(Token::Muted)),
            cw as u16,
        );
        pos[i] = Some((x, cw as u16));
        x += (cw + GAP) as u16;
    }
    let footer_h = if t.footer.is_some() { 2 } else { 0 };
    let body = area.height.saturating_sub(1 + footer_h) as usize;
    if let Some(cells) = t.footer.filter(|_| area.height > 2) {
        let ry = area.bottom() - 2;
        let line = th.fg(Token::Line);
        for xx in area.left() + 2..area.right() {
            buf[(xx, ry)].set_symbol(g.rule).set_style(line);
        }
        for (i, c) in t.cols.iter().enumerate() {
            let (Some(cw), Some((cx, _))) = (f.widths[i], pos[i]) else {
                continue;
            };
            let text = pad(
                &th.text(cell_text(&cells[i], f.short[i])),
                cw,
                g.ellipsis,
                right(c),
            );
            let style = th.fg(cells[i].token).add_modifier(Modifier::BOLD);
            buf.set_span(cx, area.bottom() - 1, &Span::styled(text, style), cw as u16);
        }
    }
    // The body as display lines: group bands, rows, and the focus line under the selected row. The draw
    // starts at `top`, or lower when bands and the focus line would push the cursor off screen.
    #[derive(Clone, Copy)]
    enum Line {
        Band(usize),
        Row(usize),
        Focus(usize),
    }
    let group_at = |n: usize| t.groups.iter().position(|(at, _, _)| *at == n);
    let lines_from = |top: usize| -> Vec<Line> {
        let mut v = Vec::new();
        // the band of the group the first row is in, even mid-group (it stays on top)
        if let Some(g) = t.groups.iter().rposition(|(at, _, _)| *at <= top)
            && t.groups[g].0 < top
        {
            v.push(Line::Band(g));
        }
        for n in top..t.rows.len() {
            if let Some(g) = group_at(n) {
                v.push(Line::Band(g));
            }
            v.push(Line::Row(n));
            if t.focus.is_some() && t.cursor == Some(n) {
                v.push(Line::Focus(n));
            }
            if v.len() >= body + 2 {
                break;
            }
        }
        v
    };
    let mut top = t.top;
    let mut lines = lines_from(top);
    if let Some(c) = t.cursor {
        let fits = |lines: &[Line]| {
            lines
                .iter()
                .take(body)
                .any(|l| matches!(l, Line::Row(n) if *n == c))
                && (t.focus.is_none()
                    || lines
                        .iter()
                        .take(body)
                        .any(|l| matches!(l, Line::Focus(n) if *n == c))
                    || body < 3)
        };
        while top < c && !fits(&lines) {
            top += 1;
            lines = lines_from(top);
        }
    }
    let x0 = pos.iter().flatten().nth(1).map_or(area.x + 2, |(x, _)| *x);
    for (k, line) in lines.into_iter().take(body).enumerate() {
        let y = area.y + 1 + k as u16;
        let n = match line {
            Line::Band(g) => {
                let (_, label, net) = &t.groups[g];
                let note = (!net.is_empty()).then(|| (net.clone(), Token::Muted));
                // the net ends where the table does (the last column's right edge)
                let end = pos.iter().flatten().last().map(|(x, w)| x + w);
                crate::widgets::band_to(
                    th,
                    buf,
                    Rect::new(area.x, y, area.width, 1),
                    label,
                    note,
                    end,
                );
                continue;
            }
            Line::Focus(n) => {
                let sel = th.selection();
                for xx in area.left()..area.right() {
                    buf[(xx, y)].set_style(sel);
                }
                let text = crate::layout::fit_str(
                    &th.text(t.focus.as_deref().unwrap_or("")),
                    area.right().saturating_sub(x0 + 1) as usize,
                    g.ellipsis,
                );
                let w = width(&text) as u16;
                buf.set_span(
                    x0,
                    y,
                    &Span::styled(text, sel.patch(th.fg(Token::Muted))),
                    w,
                );
                th.band_row(buf, Rect::new(area.x, y, area.width, 1));
                hits.add(Rect::new(area.x, y, area.width, 1), Target::Row(n - t.top));
                continue;
            }
            Line::Row(n) => n,
        };
        let row = &t.rows[n];
        let selected = t.cursor == Some(n);
        // problem rows (knobs `status_warn` / `status_err`, off by default): a faint tint; the row keeps its
        // glyphs and words
        let tint = if selected {
            None
        } else if row
            .iter()
            .any(|c| c.token == Token::Error && !c.text.trim().is_empty())
        {
            (th.palette().knobs.status_err > 0).then(|| th.bg(Token::StatusErrRow))
        } else if row
            .iter()
            .any(|c| c.token == Token::Warn && !c.text.trim().is_empty())
        {
            (th.palette().knobs.status_warn > 0).then(|| th.bg(Token::StatusWarnRow))
        } else if row
            .iter()
            .any(|c| c.token == Token::Proposed && !c.text.trim().is_empty())
        {
            // a proposed row (its `+ ~ ⊘` glyph in `proposed`): the faint proposed tint (knob `proposed_rows`;
            // 0 turns it off, the glyph stays). A selected row uses the selection only.
            (th.palette().knobs.proposed_rows > 0).then(|| th.bg(Token::ProposedTint))
        } else {
            None
        };
        if let Some(bg) = tint {
            crate::widgets::fill(buf, Rect::new(area.x, y, area.width, 1), bg);
        }
        if selected {
            let sel = th.selection();
            for xx in area.left()..area.right() {
                buf[(xx, y)].set_style(sel);
            }
            buf.set_span(area.x, y, &Span::styled(g.cursor, th.fg(Token::Accent)), 1);
        }
        for (i, c) in t.cols.iter().enumerate() {
            let (Some(cw), Some((cx, _))) = (f.widths[i], pos[i]) else {
                continue;
            };
            let is_cursor_cell = selected && t.cursor_col == Some(i);
            let mut text = th.text(cell_text(&row[i], f.short[i]));
            if is_cursor_cell && marked {
                text = format!("{mark}{text}");
            }
            let cell = pad(&text, cw, g.ellipsis, right(c));
            let mut style = th.fg(row[i].token);
            if let Some(bg) = tint {
                style = bg.patch(style);
            }
            if selected {
                style = style.patch(th.selection());
                if is_cursor_cell {
                    // Inverse within the row: REVERSED on a painted row; in ANSI and NO_COLOR the row stays
                    // one continuous reverse span (v0.12.0), so the cell is underlined and bold instead.
                    style = if matches!(
                        th.mode,
                        crate::theme::Mode::Ansi | crate::theme::Mode::NoColor
                    ) {
                        style.add_modifier(Modifier::UNDERLINED | Modifier::BOLD)
                    } else {
                        style.add_modifier(Modifier::REVERSED | Modifier::BOLD)
                    };
                }
            }
            buf.set_span(cx, y, &Span::styled(cell, style), cw as u16);
        }
        if selected {
            th.band_row(buf, Rect::new(area.x, y, area.width, 1));
        }
        // the row's position from the app's `top` (the app adds its top back)
        hits.add(Rect::new(area.x, y, area.width, 1), Target::Row(n - t.top));
    }
    pos
}
