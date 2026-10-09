//! Row H-1 (spec §3.2, §6): the view's status on the left, keys from the keymap on the right, fitted.
//! While an overlay, a prompt or the confirm band is open, the footer belongs to it.

use crate::app::{App, Overlay};
use crate::hit::{Hits, Target};
use crate::keymap::{self, Hint};
use crate::layout::{fit_str, width};
use crate::theme_tokens::Token;
use crate::widgets::fill;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::text::Span;

/// One footer item as drawn: (key text, label or "", action or None for a plain note).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Item {
    pub keys: String,
    pub label: String,
    pub action: Option<&'static str>,
}

impl Item {
    fn width(&self, sep: usize) -> usize {
        width(&self.keys)
            + if self.label.is_empty() {
                0
            } else {
                1 + width(&self.label)
            }
            + sep
    }
}

/// The Classify Queue's task hints in the order they drop when the footer is short (design ruling).
pub const CLASSIFY_DROP: [&str; 5] = [
    "classify.skip",
    "classify.note",
    "classify.group",
    "classify.treatment",
    "classify.bearer",
];

/// The footer's items for this frame at `cols` columns, best first; the palette hint is pinned last.
pub fn items(app: &App, cols: u16) -> Vec<Item> {
    let th = &app.ui.theme;
    let ctxs = app.stack();
    let hints: Vec<Hint> = keymap::footer_in(&app.keymap, &ctxs, &|w| app.holds(w));
    let globals: Vec<&'static str> = hints
        .iter()
        .filter(|h| h.global)
        .map(|h| h.action)
        .collect();
    let narrow = cols < 100;
    let mut out: Vec<Item> = Vec::new();
    let mut pinned = None;
    for h in hints {
        // spec §6: global keys are key-only below 100 columns only; at 100+ every key keeps its label
        let key_only = narrow && h.global && h.action != "palette.open";
        let label = app.label_for(h.action, h.label);
        let keys = if h.action == "classify.project_digit" {
            if th.ascii { "1-9" } else { "1–9" }.to_string()
        } else {
            h.keys.clone()
        };
        let item = Item {
            keys: th.text(&keys),
            label: if key_only {
                String::new()
            } else {
                th.text(label)
            },
            action: Some(h.action),
        };
        if h.action == "palette.open" {
            pinned = Some(item);
        } else {
            out.push(item);
        }
    }
    // the Classify Queue (ruling 4): the task keys in task order, then the protected `q` and palette hint
    if ctxs.first() == Some(&crate::keymap::Ctx::ClassifyQueue) {
        let order = [
            "drill.in",
            "classify.project",
            "classify.bearer",
            "classify.treatment",
            "classify.group",
            "classify.note",
            "classify.skip",
            "filter.open",
            "app.quit",
        ];
        let mut sorted: Vec<Item> = order
            .iter()
            .filter_map(|a| out.iter().find(|i| i.action == Some(*a)).cloned())
            .collect();
        // the labels stay at every width here: the task list drops from its right end instead
        for it in sorted.iter_mut() {
            if it.label.is_empty()
                && let Some(h) = keymap::footer_in(&app.keymap, &ctxs, &|w| app.holds(w))
                    .into_iter()
                    .find(|h| Some(h.action) == it.action)
            {
                it.label = th.text(h.label);
            }
        }
        out = sorted;
    }
    // design pass 2: at most five keys, then `? more` (help lists the rest), then the palette hint
    if app.ui.overlay.is_none() && ctxs.first() != Some(&crate::keymap::Ctx::ClassifyQueue) {
        let help = out
            .iter()
            .position(|i| i.action == Some("help.open"))
            .map(|k| out.remove(k));
        // this screen's keys first, then the global ones
        out.sort_by_key(|i| i.action.is_some_and(|a| globals.contains(&a)));
        out.truncate(5);
        if let Some(mut h) = help {
            if !h.label.is_empty() {
                h.label = "more".into();
            }
            out.push(h);
        }
    }
    if matches!(
        app.ui.overlay,
        Some(Overlay::Palette { .. } | Overlay::Picker { .. })
    ) {
        out.push(Item {
            keys: String::new(),
            label: "type to filter".into(),
            action: None,
        });
    }
    out.extend(pinned);
    out
}

/// The left-hand status: a transient note, else the view's one line.
pub fn status(app: &App) -> (String, Token) {
    if let Some(Overlay::Picker {
        multi: true,
        chosen,
        ..
    }) = &app.ui.overlay
    {
        let names: Vec<&str> = chosen
            .iter()
            .map(|&i| app.model.orgs[i].alias.as_str())
            .collect();
        let tok = if (2..=4).contains(&chosen.len()) {
            Token::Muted
        } else {
            Token::Warn
        };
        return (
            format!("({} selected: {})", chosen.len(), names.join(", ")),
            tok,
        );
    }
    // the agent's notice leads the strip until the user's next key
    if let Some((t, acted)) = app.agent_notice_text() {
        return (t, if acted { Token::Info } else { Token::Muted });
    }
    if let Some(n) = &app.ui.note {
        return (n.clone(), Token::Info);
    }
    // band hidden: the run's progress leads the strip (spec P6 §4)
    if let Some(r) = app.ui.run.as_ref().filter(|r| !r.band_visible) {
        return (
            format!("{}   ·   {}", r.strip_text(), crate::views::status(app)),
            Token::Text,
        );
    }
    // design pass 2: no routine status line (the breadcrumb and bands carry it); what the next key does
    // (an Enter hint) and the differs filter's count still show
    let s = crate::views::status(app);
    let keep = s.starts_with('↵')
        || s.starts_with("Enter")
        || (app.ui.tab == crate::app::Tab::Ledger && app.ui.differs_only);
    (if keep { s } else { String::new() }, Token::Muted)
}

pub fn draw(app: &App, buf: &mut Buffer, area: Rect, hits: &mut Hits) {
    let th = &app.ui.theme;
    let bar = th.footer_band();
    fill(buf, area, bar);
    // Two spaces between hints in the main footer (spec §6); " · " only inside the finder and help footers.
    let sep = if app.ui.overlay.is_some() {
        th.text(" · ")
    } else {
        "  ".to_string()
    };
    let sep_w = width(&sep);
    let mut items = items(app, area.width);
    let caption = app.caption_parts(area.width, area.width as usize);
    if caption.is_some() && area.width < 80 {
        items.clear(); // caption · version at 60 columns
    }
    let (status, status_tok) = status(app);
    let mut status = th.text(&status);
    // the upgrade note on show (the user's note, not an agent notice), in the form this width uses:
    // below 80 columns its remedy drops "then" (`r reload, retry`)
    let shown_note: Option<String> = app
        .ui
        .note
        .as_deref()
        .filter(|n| crate::upgrade::is_note(n) && th.text(n) == status)
        .map(|n| {
            if area.width < 80 {
                crate::upgrade::narrow_note(n)
            } else {
                n.to_string()
            }
        });
    if let Some(n) = &shown_note {
        status = th.text(n);
    }
    // Keys must fit with 3 spaces after the status. The status keeps 12 cells (it shortens with …), or all of
    // an Enter hint (`↵ Ledger · differs · full history`), which says what the next key does.
    let keep = match &caption {
        Some(parts) => parts.iter().map(|(t, _)| width(t)).sum(),
        None => crate::views::status_keep(app, &status),
    };
    // the running version: the last item, faintest text tone, protected (key hints drop before it)
    // …and, when a newer build is installed, where to get it after it
    // the build id is a detail (`i`): `v0.12.1+g6dc723f`
    let running = match app.build_label.as_deref().filter(|_| app.ui.details) {
        Some(b) => format!("v{}+{b}", app.version_label),
        None => format!("v{}", app.version_label),
    };
    let update = app.update_available.as_ref().map(|u| {
        let ready = u.action == crate::upgrade::ACTION;
        th.text(&match area.width {
            w if w >= 100 && ready => format!("  ⬆ v{} ready · ⌃P Upgrade", u.version),
            w if w >= 100 => format!("  ⬆ v{} installed · quit and restart to use it", u.version),
            w if w >= 80 && ready => format!("  ⬆ v{} · ⌃P Upgrade", u.version),
            w if w >= 80 => format!("  ⬆ v{} · restart", u.version),
            _ => format!(" ⬆ {}", u.version),
        })
    });
    // the shown manifest, protected immediately left of the version (design spec branches §2)
    let chip = app.manifest().map(|b| {
        let n = crate::layout::middle_trunc(&b.name, 16);
        if th.ascii || matches!(th.mode, crate::theme::Mode::NoColor) {
            format!("[{n}]  ")
        } else {
            format!("⎇ {n}  ")
        }
    });
    let chip_s = chip.clone().unwrap_or_default();
    let version = format!("{chip_s}{running}{}", update.as_deref().unwrap_or(""));
    let version_w = width(&version) + 2;
    let status_now = if caption.is_some() {
        keep
    } else {
        width(&status).min(keep)
    };
    let mut budget = (area.width as usize).saturating_sub(2 + 3 + status_now + version_w);
    let total = |v: &[Item]| {
        v.iter()
            .map(|i| i.width(sep_w))
            .sum::<usize>()
            .saturating_sub(sep_w)
    };
    let ready = app
        .update_available
        .as_ref()
        .is_some_and(|u| u.action == crate::upgrade::ACTION);
    if ready && caption.is_none() && app.ui.overlay.is_none() {
        // the update notice already says `⌃P Upgrade`: the palette hint isn't protected then. Order:
        // optional hints, the palette hint, an upgrade note's reason, then `q` (a status message outranks
        // key hints), and only then is the note cut.
        while total(&items) > budget {
            let Some(idx) = items.iter().rposition(|i| i.action != Some("app.quit")) else {
                break;
            };
            items.remove(idx);
        }
        if total(&items) > budget
            && let Some(short) = shown_note.as_deref().and_then(crate::upgrade::short_note)
        {
            status = th.text(&short);
            budget = (area.width as usize).saturating_sub(2 + 3 + width(&status) + version_w);
        }
        if total(&items) > budget && shown_note.is_some() {
            items.clear();
        }
    } else {
        let classify = app.stack().first() == Some(&crate::keymap::Ctx::ClassifyQueue);
        while total(&items) > budget && items.len() > usize::from(caption.is_none()) {
            // drop from the end, keeping the pinned palette hint (the last item); in the Classify Queue
            // `q Quit` is protected too and the task list drops from its right end
            let idx = if classify {
                // Skip → Note → Group → Treatment → Bearer; Accept, Project, Filter, q and ⌃P always stay
                match CLASSIFY_DROP
                    .iter()
                    .find_map(|a| items.iter().position(|i| i.action == Some(*a)))
                {
                    Some(i) => i,
                    // an agent's notice outranks the task keys too (it says what just changed)
                    None if app.agent_notice_text().is_some() && items.len() > 1 => items.len() - 2,
                    None => break,
                }
            } else {
                // (with a caption the loop may reach the palette hint itself: it goes last)
                items.len().saturating_sub(2)
            };
            items.remove(idx);
        }
        // an upgrade note still short of room: the pinned palette hint goes key-only (`⌃P`) too
        if shown_note.is_some()
            && total(&items) > budget
            && let Some(last) = items.last_mut()
            && last.action == Some("palette.open")
        {
            last.label.clear();
        }
    }
    let keys_w = total(&items).min(area.width as usize);
    let vx = area.right().saturating_sub(1 + width(&version) as u16);
    let cw = width(&chip_s) as u16;
    if let Some(c) = &chip {
        buf.set_span(
            vx,
            area.y,
            &Span::styled(
                c.clone(),
                th.fg(Token::Proposed)
                    .add_modifier(ratatui::style::Modifier::BOLD)
                    .patch(bar),
            ),
            cw,
        );
    }
    let vx = vx + cw;
    buf.set_span(
        vx,
        area.y,
        &Span::styled(running.clone(), th.fg(Token::Text4).patch(bar)),
        width(&running) as u16,
    );
    if let Some(u) = &update {
        buf.set_span(
            vx + width(&running) as u16,
            area.y,
            &Span::styled(u.clone(), th.fg(Token::Accent).patch(bar)),
            width(u) as u16,
        );
    }
    let mut x = area
        .right()
        .saturating_sub(1 + version_w as u16 + keys_w as u16);
    // with no status to show, the keys start at the left inset (design pass 2)
    if status.is_empty()
        && caption.is_none()
        && app.agent_notice_text().is_none()
        && app.ui.overlay.is_none()
    {
        x = area.x + 2;
    }
    // 3 cells between the status and the keys; with no keys left, the version's own 2-cell padding is the gap
    let gap = if items.is_empty() { 0 } else { 3 };
    let status_w = (x.saturating_sub(area.x + 1 + gap)) as usize;
    // the agent's caption (replacing its notice): ◆ in the agent colour, the text, the back hint
    if let Some(parts) = app.caption_parts(area.width, status_w) {
        let mut cx = area.x + 1;
        for (t, tok) in parts {
            let w = width(&t) as u16;
            buf.set_span(cx, area.y, &Span::styled(t, th.fg(tok).patch(bar)), w);
            cx += w;
        }
    } else {
        // the agent's notice keeps its lead and `` · ` back `` hint; what changed loses its middle
        let fitted = match app.agent_notice_fitted(status_w) {
            Some((t, _)) => t,
            None => {
                // an upgrade note too long for its room drops its reason before anything is cut
                let short = shown_note
                    .as_deref()
                    .filter(|_| width(&status) > status_w)
                    .and_then(crate::upgrade::short_note)
                    .map(|s| th.text(&s));
                fit_str(
                    short.as_ref().unwrap_or(&status),
                    status_w,
                    th.glyphs().ellipsis,
                )
            }
        };
        buf.set_span(
            area.x + 1,
            area.y,
            &Span::styled(fitted, th.fg(status_tok).patch(bar)),
            status_w as u16,
        );
    }
    for (n, it) in items.iter().enumerate() {
        let start = x;
        if !it.keys.is_empty() {
            let w = width(&it.keys) as u16;
            buf.set_span(
                x,
                area.y,
                &Span::styled(it.keys.clone(), th.fg(Token::Key).patch(bar)),
                w,
            );
            x += w;
            if !it.label.is_empty() {
                x += 1;
            }
        }
        if !it.label.is_empty() {
            let w = width(&it.label) as u16;
            buf.set_span(
                x,
                area.y,
                &Span::styled(it.label.clone(), th.fg(Token::Muted).patch(bar)),
                w,
            );
            x += w;
        }
        if let Some(a) = it.action {
            hits.add(Rect::new(start, area.y, x - start, 1), Target::Action(a));
        }
        if n + 1 < items.len() {
            buf.set_span(
                x,
                area.y,
                &Span::styled(sep.clone(), th.fg(Token::Muted).patch(bar)),
                sep_w as u16,
            );
            x += sep_w as u16;
        }
    }
}
