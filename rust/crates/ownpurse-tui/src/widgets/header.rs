//! Row 0 (spec §3.1, V5): fitted, never wrapped; protected items never truncate.

use crate::app::{App, level_token};
use crate::hit::{Hits, Target};
use crate::layout::{self, Part};
use crate::theme_tokens::Token;
use crate::widgets::fill;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::text::Span;

pub fn draw(app: &App, buf: &mut Buffer, area: Rect, hits: &mut Hits) {
    let th = &app.ui.theme;
    let bar = th.header_band();
    fill(buf, area, bar);
    // the knob-driven header variants (truecolor / 256): a gradient fading to the right, or a pair split in
    // the two orgs' accents
    let k = th.palette().knobs;
    let acc = app.accent();
    let mix = crate::theme::color::mix;
    let top = th.rgb(Token::Surface);
    let painted_bg = |x: u16| -> Option<ratatui::style::Style> {
        let w = area.width.max(1);
        if k.pair_split && acc.scope_kind == "pair" {
            let m = if x - area.x < w / 2 {
                Token::Member1
            } else {
                Token::Member2
            };
            Some(th.bg_rgb(mix(top, th.rgb(m), k.header_band)))
        } else if k.header_gradient {
            let t = (u32::from(k.header_band) * u32::from(w - (x - area.x)) / u32::from(w)) as u16;
            Some(th.bg_rgb(mix(top, th.rgb(Token::Accent), t)))
        } else {
            None
        }
    };
    let custom = painted_bg(area.x).is_some();
    let parts = layout::header(&app.header_facts(), area.width as usize, th.ascii);
    let chips = app.scope_chips();
    let mut x = area.x;
    for (text, part) in parts {
        let style = match part {
            Part::Product | Part::Sep => th.fg(Token::Muted),
            // v0.12.0: the quiet read-only mark (`header.readonly.fg`) and `offline` as an exception
            Part::Badge => th.fg(Token::Text4),
            Part::BadgeOffline => th.fg(Token::Warn),
            Part::Scope => th.fg(Token::Accent),
            Part::AsOf => th.fg(Token::Text2),
            Part::Fresh(l) => match l {
                crate::model::Level::Ok => th.fg(Token::Muted),
                other => th.fg(level_token(other)),
            },
            Part::Quota(l) => th.fg(level_token(l)),
            Part::Agent(false) => th.fg(Token::Muted),
            Part::Agent(true) => th.fg(Token::Info),
            Part::Drift => th
                .fg(Token::Warn)
                .add_modifier(ratatui::style::Modifier::BOLD),
            Part::Classify(false) => th.fg(Token::Muted),
            Part::Classify(true) => th.fg(Token::Warn),
            Part::ManifestName => th
                .fg(Token::Proposed)
                .add_modifier(ratatui::style::Modifier::BOLD),
            Part::ManifestCount => th.fg(Token::Muted),
            Part::ManifestError => th.fg(Token::Error),
            Part::ManifestWarn => th.fg(Token::Warn),
            Part::Gap => ratatui::style::Style::default(),
        };
        let w = layout::width(&text) as u16;
        if x + w > area.right() {
            break; // never reached when the fit holds (V5 property test)
        }
        let chip = (part == Part::Scope)
            .then(|| {
                chips
                    .iter()
                    .find(|c| th.text(&layout::chip_text(c)) == text)
            })
            .flatten();
        if let Some(chip) = chip {
            // the scope chip: each piece in its own token (the dots in the members' accents)
            let mut cx = x;
            for (t, tok) in chip {
                let t = th.text(t);
                let tw = layout::width(&t) as u16;
                buf.set_span(cx, area.y, &Span::styled(t, bar.patch(th.fg(*tok))), tw);
                cx += tw;
            }
        } else {
            buf.set_span(x, area.y, &Span::styled(text, bar.patch(style)), w);
        }
        if part == Part::Scope {
            hits.add(Rect::new(x, area.y, w, 1), Target::Action("scope.picker"));
        }
        // the header's chips as overlay anchors (`chip:writes` is protected: no box covers it)
        let name = match part {
            Part::Scope => Some("scope"),
            Part::AsOf => Some("asof"),
            Part::Badge | Part::BadgeOffline => Some("writes"),
            Part::Fresh(_) => Some("fresh"),
            Part::Quota(_) => Some("quota"),
            Part::Agent(_) => Some("agent"),
            Part::Drift => Some("drift"),
            Part::Classify(_) => Some("classify"),
            Part::ManifestName | Part::ManifestCount | Part::ManifestError | Part::ManifestWarn => {
                Some("manifest")
            }
            _ => None,
        };
        if let Some(n) = name {
            hits.anchors.chip(n, Rect::new(x, area.y, w, 1));
        }
        x += w;
    }
    if custom {
        for xx in area.left()..area.right() {
            if let Some(bg) = painted_bg(xx)
                && let Some(c) = buf.cell_mut((xx, area.y))
            {
                c.set_style(bg);
            }
        }
    }
}
