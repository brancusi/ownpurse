//! Frame assembly (spec §3): header · tabs · rule · crumb · content · [confirm band] · footer, then the
//! open overlay. Pure over `&App`; returns the frame's hit targets.

use crate::app::{App, Overlay, Prompt, Tab};
use crate::hit::{Hits, Target};
use crate::layout::{fit_str, width};
use crate::theme_tokens::Token;
use crate::widgets::{self, confirm, footer, header, help, palette, picker, tabs};
use ratatui::Frame;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::text::Span;

pub const MIN_W: u16 = 60;
pub const MIN_H: u16 = 20;

pub fn draw(f: &mut Frame, app: &App) -> Hits {
    let area = f.area();
    render(app, f.buffer_mut(), area)
}

/// Draw into a buffer (the terminal frame, snapshots and benches all come through here).
pub fn render(app: &App, buf: &mut Buffer, area: Rect) -> Hits {
    let mut hits = Hits::default();
    let th = &app.ui.theme;
    widgets::fill(buf, area, th.base());
    if area.width < MIN_W || area.height < MIN_H {
        let msg = th.text(&format!(
            "ownpurse needs at least {MIN_W}×{MIN_H} · now {}×{}",
            area.width, area.height
        ));
        let msg = fit_str(&msg, area.width as usize, th.glyphs().ellipsis);
        let x = area.x + (area.width.saturating_sub(width(&msg) as u16)) / 2;
        let y = area.y + area.height / 2;
        buf.set_span(x, y, &Span::styled(msg, th.fg(Token::Text)), area.width);
        return hits;
    }
    let row = |y: u16| Rect::new(area.x, area.y + y, area.width, 1);
    header::draw(app, buf, row(0), &mut hits);
    tabs::draw(app, buf, row(1), row(2), &mut hits);
    crumb(app, buf, row(3), &mut hits);
    let footer_y = area.bottom() - 1;
    // the band slot (H-2): the confirm / refusal band, else the run's band or its result (non-modal)
    let band_h = match (&app.ui.band, &app.ui.run, &app.ui.result) {
        (Some(b), _, _) => confirm::height(th, b, area.width),
        (None, Some(r), _) if r.band_visible => 1,
        (None, None, Some(res)) => crate::widgets::runband::result_height(th, res, area.width),
        _ => 0,
    };
    let content = Rect::new(
        area.x,
        area.y + 4,
        area.width,
        footer_y.saturating_sub(area.y + 4 + band_h),
    );
    let (content, panel) = crate::walk::split(app, content);
    crate::views::draw(app, buf, content, &mut hits);
    if let Some(p) = panel {
        crate::walk::draw(app, buf, p, &mut hits);
    }
    let slot = Rect::new(area.x, footer_y - band_h, area.width, band_h);
    match (&app.ui.band, &app.ui.run, &app.ui.result) {
        (Some(b), _, _) => confirm::draw(th, b, buf, slot, &mut hits),
        (None, Some(r), _) if r.band_visible => crate::widgets::runband::draw_run(th, r, buf, slot),
        (None, None, Some(res)) => crate::widgets::runband::draw_result(th, res, buf, slot),
        _ => {}
    }
    footer::draw(app, buf, row(area.height - 1), &mut hits);
    if band_h > 0 {
        hits.anchors.panel("band", slot);
    }
    // tabs as chips (`chip:tab-ledger`), then the overlay layers over everything but the header and footer
    let tabs: Vec<(Rect, Tab)> = hits
        .targets
        .iter()
        .filter_map(|(r, t)| match t {
            Target::Tab(tab) => Some((*r, *tab)),
            _ => None,
        })
        .collect();
    for (r, t) in tabs {
        hits.anchors
            .chip(&format!("tab-{}", t.label().to_lowercase()), r);
    }
    // layers go over the views (from the tab row down to the content's end), never over the narration
    // panel, a band or the footer
    let layer_area = Rect::new(
        area.x,
        area.y + 3,
        content.right().saturating_sub(area.x),
        content.bottom().saturating_sub(area.y + 3),
    );
    crate::overlay::draw(app, buf, layer_area, &mut hits);
    crate::walk::draw_popups(app, buf, layer_area, &mut hits);
    match &app.ui.overlay {
        Some(Overlay::Help) => help::draw(app, buf, area, &mut hits),
        Some(Overlay::Palette { .. }) => palette::draw(app, buf, area, &mut hits),
        Some(Overlay::Picker { .. }) => picker::draw(app, buf, area, &mut hits),
        None => {}
    }
    hits
}

/// Row 3: the breadcrumb (left) and this view's record age (right), or the open prompt.
fn crumb(app: &App, buf: &mut Buffer, area: Rect, hits: &mut Hits) {
    let th = &app.ui.theme;
    let g = th.glyphs();
    if let Some(p) = &app.ui.prompt {
        let (label, text) = match p {
            Prompt::Filter(t) => ("Filter", t),
            Prompt::AsOf(t) => ("As of", t),
            Prompt::Range(t) => ("From", t),
        };
        let head = th.text(&format!("{label} › "));
        let x = area.x + 2;
        buf.set_span(
            x,
            area.y,
            &Span::styled(head.clone(), th.fg(Token::Muted)),
            area.width,
        );
        let x = x + width(&head) as u16;
        let t = fit_str(
            text,
            area.right().saturating_sub(x + 2) as usize,
            g.ellipsis,
        );
        let w = width(&t) as u16;
        buf.set_span(x, area.y, &Span::styled(t.clone(), th.fg(Token::Text)), w);
        crate::widgets::caret(buf, x, area.y, &t, app.ui.caret, th.fg(Token::Text), w);
        return;
    }
    // the breadcrumb tint (knob `breadcrumb`, off by default)
    if th.palette().knobs.breadcrumb > 0 {
        widgets::fill(buf, area, th.bg(Token::BreadcrumbBand));
    }
    let members = app.members(&app.ui.scope);
    let fresh = app.model.freshness(&members, app.now);
    let right = th.text(&fresh.full());
    let right_w = width(&right) as u16;
    let original = crate::views::crumbs(app);
    let sep = th.text(&format!(" {} ", g.crumb));
    let total = |c: &[String]| {
        c.iter().map(|s| width(&th.text(s))).sum::<usize>()
            + width(&sep) * c.len().saturating_sub(1)
    };
    // Middle segments become … first, so where you came from and what you're looking at stay (D1);
    // if that still doesn't fit, the record age on the right goes (it's in the header too).
    let compact = |max: usize| {
        let mut c = original.clone();
        let mut i = 1;
        while total(&c) > max && i + 1 < c.len() {
            c[i] = g.ellipsis.to_string();
            i += 1;
        }
        c.dedup_by(|a, b| a == b && a == g.ellipsis);
        c
    };
    let mut left_max = area.width.saturating_sub(right_w + 5);
    let mut crumbs = compact(left_max as usize);
    // the record age is a detail now (design pass 2): the header names it when stale; `i` shows it here
    let mut show_right = app.ui.details;
    if total(&crumbs) > left_max as usize {
        show_right = false;
        left_max = area.width.saturating_sub(3);
        crumbs = compact(left_max as usize);
    }
    let mut spans = Vec::new();
    for (i, c) in crumbs.iter().enumerate() {
        if i > 0 {
            spans.push(Span::styled(sep.clone(), th.fg(Token::Muted)));
        }
        let last = i + 1 == crumbs.len();
        spans.push(Span::styled(
            th.text(c),
            if last {
                th.fg(Token::Text)
            } else {
                th.fg(Token::Muted)
            },
        ));
    }
    widgets::put(buf, area.x + 2, area.y, spans, left_max, g.ellipsis);
    if crumbs.len() > 1 && app.ui.chain.last().is_some_and(|o| o.to == app.ui.tab) {
        let w = width(&crumbs[0]) as u16;
        hits.add(
            Rect::new(area.x + 2, area.y, w, 1),
            Target::Action("drill.back"),
        );
    }
    if !show_right && let Some((s, tok)) = crate::views::crumb_state(app) {
        let s = th.text(&s);
        let w = width(&s) as u16;
        if total(&crumbs) as u16 + w + 6 < area.width {
            buf.set_span(
                area.right() - 2 - w,
                area.y,
                &Span::styled(s, th.fg(tok)),
                w,
            );
        }
    }
    if show_right && right_w + 4 < area.width {
        let tok = match fresh.level {
            crate::model::Level::Ok => Token::Muted,
            l => crate::app::level_token(l),
        };
        buf.set_span(
            area.right() - 1 - right_w,
            area.y,
            &Span::styled(right, th.fg(tok)),
            right_w,
        );
    }
}
