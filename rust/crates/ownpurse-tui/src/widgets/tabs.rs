//! Rows 1–2: the tabs on the tab band and the rule under them (`━` under the active tab in the scope
//! accent, so it reads without colour; V12). v0.12.0 (design pass 2): numbers in `text.4`, inactive labels in
//! `text.3`, the active one in `text.1`; a 2-cell inset and 3-cell gaps when they fit.

use crate::app::{App, Tab};
use crate::hit::{Hits, Target};
use crate::layout::width;
use crate::theme_tokens::Token;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::text::Span;

pub fn draw(app: &App, buf: &mut Buffer, tabs_row: Rect, rule_row: Rect, hits: &mut Hits) {
    let th = &app.ui.theme;
    let g = th.glyphs();
    let band = th.bg(Token::TabsBand);
    crate::widgets::fill(buf, tabs_row, band);
    // on a manifest the hairline is dashed in `proposed` (spec branches §2): the quiet "this is a draft" cue;
    // ASCII and plain renders draw `- - -`
    let on_manifest = app.ui.manifest.is_some();
    let (sym, line) = if on_manifest {
        (
            if g.rule == "-" { "- " } else { "╌" },
            th.fg(Token::Proposed),
        )
    } else {
        (g.rule, th.fg(Token::Line))
    };
    for (i, x) in (rule_row.left()..rule_row.right()).enumerate() {
        let s = if sym == "- " {
            if i % 2 == 0 { "-" } else { " " }
        } else {
            sym
        };
        buf[(x, rule_row.y)].set_symbol(s).set_style(line);
    }
    let full: Vec<(String, &str)> = Tab::ALL
        .iter()
        .enumerate()
        .map(|(i, t)| (format!("{} ", i + 1), t.label()))
        .collect();
    // the spacing is fitted on the first six tabs (the v0.12 set), so their row stays byte-identical across
    // versions (the frozen-frame rule); tab 7 is appended when it fits (key 7 always works)
    let first: usize = full.iter().take(6).map(|(n, l)| width(n) + width(l)).sum();
    let (mut inset, mut gap) = if first + 2 + 3 * 5 <= tabs_row.width as usize {
        (2u16, 3u16)
    } else {
        (1, 2)
    };
    // the active tab is never dropped: when it would fall off the row, the labels shorten (`1 Ovw`), and
    // only then are inactive tabs dropped from the right
    let fits_upto = |labels: &[(String, &str)], inset: u16, gap: u16, upto: usize| {
        let w: usize = labels
            .iter()
            .take(upto + 1)
            .map(|(n, l)| width(n) + width(l))
            .sum();
        inset as usize + w + gap as usize * upto <= tabs_row.width as usize
    };
    let active_i = app.ui.tab.index();
    let mut labels = full.clone();
    if !fits_upto(&labels, inset, gap, active_i) {
        labels = Tab::ALL
            .iter()
            .enumerate()
            .map(|(i, t)| (format!("{} ", i + 1), t.short_label()))
            .collect();
        inset = 1;
        gap = 2;
    }
    // still too narrow: keep the active tab, drop inactive tabs from the right
    let mut keep: Vec<usize> = (0..labels.len()).collect();
    while keep.len() > 1 {
        let w: usize = keep
            .iter()
            .map(|&i| width(&labels[i].0) + width(labels[i].1))
            .sum();
        if inset as usize + w + gap as usize * (keep.len() - 1) <= tabs_row.width as usize {
            break;
        }
        match keep.iter().rposition(|&i| i != active_i) {
            Some(p) => {
                keep.remove(p);
            }
            None => break,
        }
    }
    let mut x = tabs_row.x + inset;
    for (i, t) in Tab::ALL.iter().enumerate() {
        if !keep.contains(&i) {
            continue;
        }
        let (num, label) = &labels[i];
        let w = (width(num) + width(label)) as u16;
        if x + w > tabs_row.right() {
            break;
        }
        let active = *t == app.ui.tab;
        buf.set_span(
            x,
            tabs_row.y,
            &Span::styled(num.clone(), band.patch(th.fg(Token::Text4))),
            w,
        );
        let style = if active {
            th.fg(Token::Text)
                .add_modifier(crate::theme_tokens::styles::HEADING)
        } else {
            th.fg(Token::Muted)
        };
        buf.set_span(
            x + width(num) as u16,
            tabs_row.y,
            &Span::styled(*label, band.patch(style)),
            w,
        );
        if active {
            for xx in x..x + w {
                buf[(xx, rule_row.y)]
                    .set_symbol(g.rule_heavy)
                    .set_style(th.fg(Token::Accent));
            }
        }
        hits.add(Rect::new(x, tabs_row.y, w, 1), Target::Tab(*t));
        x += w + gap;
    }
}
