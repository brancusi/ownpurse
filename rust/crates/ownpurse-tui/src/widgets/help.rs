//! Help (`?`, spec §6): the current stack's live bindings by group, then global, straight from the
//! keymap table. Rows flow into columns; a click on a row closes help and runs its action.

use crate::app::App;
use crate::hit::{Hits, Target};
use crate::keymap;
use crate::layout::{fit_str, pad};
use crate::theme_tokens::Token;
use crate::widgets::centred;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::text::Span;

enum Row {
    Heading(&'static str),
    Key(String, String, &'static str),
}

const KEY_W: usize = 11;
const COL_W: usize = 30;

/// Pack group blocks into columns of at most `h` rows (a blank row between blocks in one column).
/// Returns the columns, or None if a block is taller than `h`.
fn pack(blocks: &[Vec<Row>], h: usize) -> Option<Vec<Vec<usize>>> {
    let mut cols: Vec<Vec<usize>> = vec![];
    let mut used = 0;
    for (i, b) in blocks.iter().enumerate() {
        if b.len() > h {
            return None;
        }
        match cols.last_mut() {
            Some(c) if used + 1 + b.len() <= h => {
                c.push(i);
                used += 1 + b.len();
            }
            _ => {
                cols.push(vec![i]);
                used = b.len();
            }
        }
    }
    Some(cols)
}

pub fn draw(app: &App, buf: &mut Buffer, area: Rect, hits: &mut Hits) {
    let th = &app.ui.theme;
    let stack = app.view_stack();
    let mut blocks: Vec<Vec<Row>> = Vec::new();
    for (group, items) in keymap::help_in(&app.keymap, &stack, &|w| app.holds(w)) {
        let mut b = vec![Row::Heading(group)];
        for (keys, bd) in items {
            b.push(Row::Key(
                th.text(&keys),
                th.text(app.label_for(bd.action, bd.label)),
                bd.action,
            ));
        }
        blocks.push(b);
    }
    let max_w = area.width.saturating_sub(6) as usize;
    let max_h = area.height.saturating_sub(4).saturating_sub(2).max(1) as usize;
    let max_cols = (max_w.saturating_sub(3) / COL_W).max(1);
    // Each group stays in one column and no heading sits alone at a column's foot: the shortest height
    // whose packing fits the columns we have.
    let tallest = blocks.iter().map(Vec::len).max().unwrap_or(1);
    let (h, cols) = (tallest.min(max_h)..=max_h)
        .find_map(|h| {
            pack(&blocks, h)
                .filter(|c| c.len() <= max_cols)
                .map(|c| (h, c))
        })
        .unwrap_or_else(|| {
            // too small: one block per column, cut at the bottom with a note
            let c: Vec<Vec<usize>> = (0..blocks.len().min(max_cols)).map(|i| vec![i]).collect();
            (max_h, c)
        });
    let ncols = cols.len().max(1);
    let box_w = (ncols * COL_W + 3).min(max_w) as u16;
    let box_h = (h + 2) as u16;
    let rect = centred(area, box_w, box_h, None);
    let inner =
        crate::widgets::overlay_box_clear(buf, rect, &format!("Keys · {}", app.ui.tab.label()), th);
    hits.overlay = Some(rect);
    let raised = th.raised();
    let ell = th.glyphs().ellipsis;
    let shown: usize = cols.iter().flatten().count();
    for (ci, col) in cols.iter().enumerate() {
        let x = inner.x + 1 + (ci * COL_W) as u16;
        let w = (COL_W - 1).min((inner.right().saturating_sub(x)) as usize);
        let mut line = 0usize;
        for (k, &bi) in col.iter().enumerate() {
            if k > 0 {
                line += 1;
            }
            for row in &blocks[bi] {
                if line >= inner.height as usize {
                    break;
                }
                let y = inner.y + line as u16;
                match row {
                    Row::Heading(g) => {
                        buf.set_span(
                            x,
                            y,
                            &Span::styled(fit_str(g, w, ell), th.heading().patch(raised)),
                            w as u16,
                        );
                    }
                    Row::Key(keys, label, action) => {
                        let k = pad(keys, KEY_W.min(w), ell, false);
                        buf.set_span(
                            x,
                            y,
                            &Span::styled(k, th.fg(Token::Text).patch(raised)),
                            KEY_W as u16,
                        );
                        let lw = w.saturating_sub(KEY_W + 1);
                        buf.set_span(
                            x + KEY_W as u16 + 1,
                            y,
                            &Span::styled(
                                fit_str(label, lw, ell),
                                th.fg(Token::Muted).patch(raised),
                            ),
                            lw as u16,
                        );
                        hits.add(Rect::new(x, y, w as u16, 1), Target::HelpRow(action));
                    }
                }
                line += 1;
            }
        }
    }
    if shown < blocks.len() && inner.height > 0 {
        let more = th.text(&format!("… {} more groups (widen)", blocks.len() - shown));
        let y = inner.bottom() - 1;
        buf.set_span(
            inner.x + 1,
            y,
            &Span::styled(
                fit_str(&more, inner.width as usize - 2, ell),
                th.fg(Token::Muted).patch(raised),
            ),
            inner.width - 2,
        );
    }
}
