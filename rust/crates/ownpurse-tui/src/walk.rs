//! The narration panel and the walkthrough player (walkthroughs design §2).
//!
//! The panel docks at the bottom of the content (on the right at 160 columns or more). It shows an agent's
//! `narrate`, or the walkthrough step on screen: `Step 3 of 12 · <title>`, the body (`**bold**`, `[term]`
//! glossary links), in lay mode the step's "why this matters", the progress strip (`●○○`, clickable) and
//! the keys.

use crate::app::App;
use crate::hit::{Hits, Target};
use crate::layout::width;
use crate::theme::Theme;
use crate::theme_tokens::Token;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::Span;
use serde::{Deserialize, Serialize};

/// What the narration panel shows.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Narration {
    pub title: String,
    pub body: String,
    /// The agent that narrates (None: a walkthrough step).
    #[serde(default)]
    pub by: Option<String>,
    /// "Why this matters" (lay mode).
    #[serde(default)]
    pub why: Option<String>,
    /// How many lines the panel is scrolled down (`PgDn` / `PgUp`; text is never cut, it scrolls).
    #[serde(default)]
    pub scroll: usize,
}

/// Columns from which the panel docks on the right.
pub const RIGHT_DOCK_COLS: u16 = 160;
/// The right-docked panel's width.
pub const RIGHT_W: u16 = 52;

/// A piece of body text: plain, bold, or a glossary term.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Piece {
    Text(String),
    Bold(String),
    Term(String),
}

/// `**bold**` and `[term]` in a body (unclosed markers stay as text).
pub fn pieces(s: &str) -> Vec<Piece> {
    let mut out = Vec::new();
    let mut rest = s;
    let mut text = String::new();
    while !rest.is_empty() {
        if let Some(r) = rest.strip_prefix("**")
            && let Some(end) = r.find("**")
        {
            if !text.is_empty() {
                out.push(Piece::Text(std::mem::take(&mut text)));
            }
            out.push(Piece::Bold(r[..end].to_string()));
            rest = &r[end + 2..];
            continue;
        }
        if let Some(r) = rest.strip_prefix('[')
            && let Some(end) = r.find(']')
            && end > 0
            && !r[..end].contains('[')
        {
            if !text.is_empty() {
                out.push(Piece::Text(std::mem::take(&mut text)));
            }
            out.push(Piece::Term(r[..end].to_string()));
            rest = &r[end + 1..];
            continue;
        }
        let c = rest.chars().next().unwrap_or(' ');
        text.push(c);
        rest = &rest[c.len_utf8()..];
    }
    if !text.is_empty() {
        out.push(Piece::Text(text));
    }
    out
}

/// The terms a body links to, in order, once each.
pub fn terms(s: &str) -> Vec<String> {
    let mut v: Vec<String> = Vec::new();
    for p in pieces(s) {
        if let Piece::Term(t) = p
            && !v.contains(&t)
        {
            v.push(t);
        }
    }
    v
}

/// A styled word of the body (words keep their piece's style; spaces split them).
#[derive(Clone, Debug)]
struct Word {
    text: String,
    style: Style,
    term: Option<String>,
}

fn words(th: &Theme, body: &str, base: Style, focus: Option<&str>) -> Vec<Vec<Word>> {
    // paragraphs on blank lines or newlines
    body.split('\n')
        .map(|para| {
            let mut out: Vec<Word> = Vec::new();
            for p in pieces(para) {
                let (t, style, term) = match p {
                    Piece::Text(t) => (t, base, None),
                    Piece::Bold(t) => (t, base.add_modifier(Modifier::BOLD), None),
                    Piece::Term(t) => {
                        let mut st = th.fg(Token::Accent).add_modifier(Modifier::UNDERLINED);
                        if focus == Some(t.as_str()) {
                            st = st.add_modifier(Modifier::REVERSED);
                        }
                        (t.clone(), st, Some(t))
                    }
                };
                for (i, w) in t.split(' ').enumerate() {
                    if i > 0 {
                        out.push(Word {
                            text: " ".into(),
                            style: base,
                            term: None,
                        });
                    }
                    if !w.is_empty() {
                        out.push(Word {
                            text: th.text(w),
                            style,
                            term: term.clone(),
                        });
                    }
                }
            }
            out
        })
        .collect()
}

/// Lay words into lines of at most `w` cells (a word longer than a line is cut).
fn lay(paras: Vec<Vec<Word>>, w: usize) -> Vec<Vec<Word>> {
    let mut lines: Vec<Vec<Word>> = Vec::new();
    for para in paras {
        let mut line: Vec<Word> = Vec::new();
        let mut used = 0;
        for word in para {
            let ww = width(&word.text);
            if word.text == " " {
                if used > 0 && used < w {
                    used += 1;
                    line.push(word);
                }
                continue;
            }
            if used + ww > w && used > 0 {
                while line.last().is_some_and(|x| x.text == " ") {
                    line.pop();
                }
                lines.push(std::mem::take(&mut line));
                used = 0;
            }
            used += ww;
            line.push(word);
        }
        while line.last().is_some_and(|x| x.text == " ") {
            line.pop();
        }
        lines.push(line);
    }
    lines
}

/// The panel's rect within the content `area`, and the content left beside or above it.
pub fn split(app: &App, area: Rect) -> (Rect, Option<Rect>) {
    let Some(_) = shown(app) else {
        return (area, None);
    };
    if area.width >= RIGHT_DOCK_COLS {
        let w = RIGHT_W;
        let content = Rect::new(area.x, area.y, area.width - w - 1, area.height);
        let panel = Rect::new(area.right() - w, area.y, w, area.height);
        return (content, Some(panel));
    }
    let h = height(app, area.width).min(area.height.saturating_sub(4));
    let content = Rect::new(area.x, area.y, area.width, area.height - h);
    (
        content,
        Some(Rect::new(area.x, area.bottom() - h, area.width, h)),
    )
}

/// What the panel shows now, if anything.
pub fn shown(app: &App) -> Option<&Narration> {
    app.ui.narration.as_ref()
}

/// The widest the narration's text runs (a readable line; the panel centres it at wide sizes).
pub const TEXT_W: u16 = 100;

fn inner_w(w: u16) -> usize {
    w.min(TEXT_W + 4).saturating_sub(4) as usize
}

/// Every line the panel's body has at width `w`: the body, then (lay mode) "Why this matters: …".
fn panel_lines(app: &App, w: u16, base: Style, muted: Style) -> Vec<Vec<Word>> {
    let Some(n) = shown(app) else { return vec![] };
    let th = &app.ui.theme;
    let inner = inner_w(w);
    let focus = app.ui.glossary_focus.as_deref();
    let mut lines = lay(words(th, &n.body, base, focus), inner);
    if app.ui.lay_mode
        && let Some(why) = &n.why
    {
        for l in crate::layout::wrap(&th.text(&format!("Why this matters: {why}")), inner) {
            lines.push(vec![Word {
                text: l,
                style: muted,
                term: None,
            }]);
        }
    }
    lines
}

/// The most body lines the bottom panel takes: about 40% of the screen (then it scrolls).
fn max_body(app: &App) -> usize {
    ((app.ui.size.1 as usize) * 40 / 100)
        .saturating_sub(2)
        .max(2)
}

/// The panel's body lines and how many show at once (its width and room as `split` lays it out).
pub fn body_fit(app: &App) -> (usize, usize) {
    let (w, h) = app.ui.size;
    let (pw, room) = if w >= RIGHT_DOCK_COLS {
        (RIGHT_W, (h as usize).saturating_sub(8))
    } else {
        (w, max_body(app))
    };
    let n = panel_lines(app, pw, Style::default(), Style::default()).len();
    (n, n.min(room).max(1))
}

/// The bottom panel's height: a rule with the title, the body (all of it up to about 40% of the screen,
/// then it scrolls), the strip/keys line.
pub fn height(app: &App, w: u16) -> u16 {
    let n = panel_lines(app, w, Style::default(), Style::default()).len();
    let body = n.min(max_body(app)).max(1) as u16;
    1 + body + 1
}

/// Draw the panel into `rect`.
pub fn draw(app: &App, buf: &mut Buffer, rect: Rect, hits: &mut Hits) {
    let Some(n) = shown(app) else { return };
    let th = &app.ui.theme;
    let g = th.glyphs();
    crate::widgets::fill(buf, rect, th.raised());
    hits.anchors.panel("narration", rect);
    let right = rect.width < app.ui.size.0;
    // the text column: at most TEXT_W wide, centred (the rule spans the panel)
    let tw = rect.width.min(TEXT_W + 4);
    let tb = Rect::new(rect.x + (rect.width - tw) / 2, rect.y, tw, rect.height);
    // the title rule: `── Step 3 of 12 · Title ──…` (an agent's: `── ◆ name · Title`)
    let rule = th.fg(Token::Line).patch(th.raised());
    for x in rect.left()..rect.right() {
        buf[(x, rect.top())].set_symbol(g.rule).set_style(rule);
    }
    let mut head = String::new();
    if let Some(w) = app.ui.walk.as_ref() {
        head.push_str(&format!("Step {} of {} · ", w.at() + 1, w.len()));
    }
    if let Some(by) = &n.by {
        head.push_str(&format!("◆ {by} · "));
    }
    head.push_str(&n.title);
    let tok = if n.by.is_some() {
        Token::Info
    } else {
        Token::Accent
    };
    crate::widgets::put(
        buf,
        tb.x + 2,
        rect.y,
        vec![Span::styled(
            th.text(&format!(" {head} ")),
            th.fg(tok).patch(th.raised()).add_modifier(Modifier::BOLD),
        )],
        tb.width.saturating_sub(4),
        g.ellipsis,
    );
    let inner = inner_w(rect.width);
    let base = th.fg(Token::Text).patch(th.raised());
    let lines = panel_lines(app, tb.width, base, th.fg(Token::Muted).patch(th.raised()));
    let _ = inner;
    let last = rect.bottom() - 1;
    let max_lines = (last - rect.y - 1) as usize;
    let scroll = n.scroll.min(lines.len().saturating_sub(max_lines));
    let more_below = scroll + max_lines < lines.len();
    // the strip and the keys sit right under the text (a tall right-docked panel keeps them by the words)
    let shown_lines = lines.len().saturating_sub(scroll).min(max_lines) as u16;
    let foot_y = (rect.y + 1 + shown_lines).min(last);
    for (k, line) in lines.iter().skip(scroll).take(max_lines).enumerate() {
        let y = rect.y + 1 + k as u16;
        let mut x = tb.x + 2;
        for w in line {
            let ww = width(&w.text) as u16;
            if x + ww > tb.right().saturating_sub(1) {
                break;
            }
            buf.set_string(x, y, &w.text, w.style);
            if let Some(t) = &w.term {
                hits.add(Rect::new(x, y, ww, 1), Target::Term(term_index(app, t)));
            }
            x += ww;
        }
    }
    let _ = right;
    // the last line: the progress strip and the keys (a walkthrough), or how to close an agent's panel
    let keys_style = th.fg(Token::Muted).patch(th.raised());
    // the text never ends cut off: what is below (or above) says so, PgDn / PgUp scroll it
    let more = match (more_below, scroll > 0) {
        (true, _) => Some(th.text("↓ more · PgDn")),
        (false, true) => Some(th.text("↑ PgUp")),
        _ => None,
    };
    let mut more_w = 0;
    if let Some(m) = &more {
        more_w = width(m) as u16 + 3;
        let mx = tb.right().saturating_sub(2 + width(m) as u16);
        buf.set_string(mx, foot_y, m, th.fg(Token::Accent).patch(th.raised()));
    }
    if let Some(w) = app.ui.walk.as_ref() {
        let mut x = tb.x + 2;
        let total = w.len();
        let room = tb.width.saturating_sub(4) as usize;
        let strip_w = (total * 2).min(room / 2);
        let per = if total * 2 <= strip_w { 2 } else { 1 };
        let shown = strip_w / per;
        let first = if w.at() >= shown {
            w.at() + 1 - shown
        } else {
            0
        };
        hits.anchors
            .chip("walk-strip", Rect::new(x, foot_y, strip_w as u16, 1));
        for i in first..(first + shown).min(total) {
            let (dot, st) = if i == w.at() {
                ("●", th.fg(Token::Accent).patch(th.raised()))
            } else if i < w.at() {
                ("●", th.fg(Token::Muted).patch(th.raised()))
            } else {
                ("○", keys_style)
            };
            buf.set_string(x, foot_y, th.text(dot), st);
            hits.add(Rect::new(x, foot_y, per as u16, 1), Target::WalkStep(i));
            x += per as u16;
        }
        let keys = th.text("→ next · ← back · t contents · Esc leave");
        let keys = if app.ui.lay_mode {
            keys
        } else {
            th.text("→ next · ← back · Esc leave")
        };
        let kx = tb.right().saturating_sub(2 + more_w + width(&keys) as u16);
        if kx > x + 2 {
            buf.set_string(kx, foot_y, keys, keys_style);
        }
    } else {
        let keys = th.text("Esc closes");
        let kx = tb.right().saturating_sub(2 + more_w + width(&keys) as u16);
        buf.set_string(kx, foot_y, keys, keys_style);
    }
}

/// A term's index in the shown body's term list (the click target carries it).
fn term_index(app: &App, t: &str) -> usize {
    shown(app)
        .map(|n| terms(&n.body))
        .and_then(|v| v.iter().position(|x| x == t))
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bold_and_terms() {
        assert_eq!(
            pieces("A **[debit]** is [debit] and [credit]. [x"),
            vec![
                Piece::Text("A ".into()),
                Piece::Bold("[debit]".into()),
                Piece::Text(" is ".into()),
                Piece::Term("debit".into()),
                Piece::Text(" and ".into()),
                Piece::Term("credit".into()),
                Piece::Text(". [x".into()),
            ]
        );
        assert_eq!(terms("[a] [b] [a]"), vec!["a", "b"]);
    }
}

/// The walkthrough being played (in the canonical view state, so a render from state shows the same step).
/// Above the frame: the failed-checks band, the glossary definition (`?`/click), the table of contents.
pub fn draw_popups(app: &App, buf: &mut Buffer, area: Rect, hits: &mut Hits) {
    let th = &app.ui.theme;
    let g = th.glyphs();
    if let Some(w) = app.ui.walk.as_ref()
        && !w.failed.is_empty()
    {
        // the crumb row: never silently wrong numbers
        let y = area.y + 2;
        let r = Rect::new(area.x, y, area.width, 1);
        crate::widgets::fill(buf, r, th.bg(Token::WarnTint));
        let s = th.text(&format!(
            " ✗ The books changed since this walkthrough was written: {} · the numbers shown are today's",
            w.failed.join(" · ")
        ));
        crate::widgets::put(
            buf,
            area.x + 1,
            y,
            vec![Span::styled(
                s,
                th.fg(Token::Warn).patch(th.bg(Token::WarnTint)),
            )],
            area.width.saturating_sub(2),
            g.ellipsis,
        );
        hits.anchors.panel("warnings", r);
    }
    if let Some(list) = &app.ui.walk_list {
        let w = area.width.saturating_sub(8).min(76);
        let h = (list.items.len() as u16 + 3).min(area.height.saturating_sub(2));
        let rect = crate::widgets::centred(area, w, h, None);
        let inner = crate::widgets::overlay_box_clear(buf, rect, "Walkthroughs", th);
        let rows = inner.height.saturating_sub(1) as usize;
        let first = list.sel.saturating_sub(rows.saturating_sub(1));
        for (k, i) in (first..list.items.len()).take(rows).enumerate() {
            let it = &list.items[i];
            let right = match &it.error {
                Some(_) => th.text("can't be read"),
                None => th.text(&format!("{} steps · {}", it.steps, it.created)),
            };
            let rw = width(&right);
            let y = inner.y + k as u16;
            let st = if i == list.sel {
                th.selection()
            } else {
                th.fg(Token::Text).patch(th.raised())
            };
            let r = Rect::new(inner.x, y, inner.width, 1);
            crate::widgets::fill(buf, r, st);
            let tw = (inner.width as usize).saturating_sub(rw + 4);
            buf.set_stringn(inner.x + 1, y, th.text(&it.title), tw, st);
            let rst = if i == list.sel {
                st
            } else {
                th.fg(Token::Muted).patch(th.raised())
            };
            buf.set_string(inner.right().saturating_sub(1 + rw as u16), y, right, rst);
        }
        let keys = th.text("Enter play · Esc close");
        buf.set_string(
            inner.right().saturating_sub(1 + width(&keys) as u16),
            inner.bottom().saturating_sub(1),
            keys,
            th.fg(Token::Muted).patch(th.raised()),
        );
        hits.overlay = Some(rect);
        hits.anchors.panel("walkthroughs", rect);
        return;
    }
    if let Some(term) = &app.ui.glossary_focus {
        let (title, mut body) = match app.glossary.get(term) {
            Some(e) => (e.term.clone(), vec![e.text.clone()]),
            None => (term.clone(), vec!["Not in the glossary.".to_string()]),
        };
        if let Some(e) = app.glossary.get(term) {
            for x in &e.examples {
                body.push(format!("In your books: {x}"));
            }
        }
        let w = area.width.saturating_sub(8).min(64);
        let inner = w.saturating_sub(4) as usize;
        let mut lines: Vec<String> = Vec::new();
        for (i, b) in body.iter().enumerate() {
            if i > 0 {
                lines.push(String::new());
            }
            lines.extend(crate::layout::wrap(&th.text(b), inner));
        }
        let h = (lines.len() as u16 + 3).min(area.height.saturating_sub(2));
        let rect = crate::widgets::centred(area, w, h, None);
        let inner_r = crate::widgets::overlay_box_clear(buf, rect, &title, th);
        for (k, l) in lines
            .iter()
            .enumerate()
            .take(inner_r.height.saturating_sub(1) as usize)
        {
            let st = if l.starts_with("In your books") {
                th.fg(Token::Muted).patch(th.raised())
            } else {
                th.fg(Token::Text).patch(th.raised())
            };
            buf.set_stringn(inner_r.x + 1, inner_r.y + k as u16, l, inner, st);
        }
        let close = th.text("Esc closes");
        buf.set_string(
            inner_r.right().saturating_sub(1 + width(&close) as u16),
            inner_r.bottom().saturating_sub(1),
            close,
            th.fg(Token::Muted).patch(th.raised()),
        );
        hits.overlay = Some(rect);
        hits.anchors.panel("glossary", rect);
        return;
    }
    if let Some(w) = app.ui.walk.as_ref()
        && let Some(cur) = w.toc
    {
        let n = w.len();
        let width_box = area.width.saturating_sub(8).min(60);
        let h = (n as u16 + 2).min(area.height.saturating_sub(2));
        let rect = crate::widgets::centred(area, width_box, h, None);
        let inner = crate::widgets::overlay_box_clear(buf, rect, &w.file().title, th);
        let rows = inner.height as usize;
        let first = cur.saturating_sub(rows.saturating_sub(1));
        for (k, i) in (first..n).take(rows).enumerate() {
            let s = &w.file().steps[i];
            let title = ownpurse_core::walkthrough::step_title(s);
            let mark = if i == w.at() { "●" } else { " " };
            let line = th.text(&format!("{mark} {:>2}  {title}", i + 1));
            let y = inner.y + k as u16;
            let st = if i == cur {
                th.selection()
            } else {
                th.fg(Token::Text).patch(th.raised())
            };
            let r = Rect::new(inner.x, y, inner.width, 1);
            crate::widgets::fill(buf, r, st);
            buf.set_stringn(inner.x + 1, y, line, inner.width as usize - 2, st);
            hits.add(r, Target::WalkStep(i));
        }
        hits.overlay = Some(rect);
        hits.anchors.panel("contents", rect);
    }
}

/// The walkthrough being played: caretline-tour's progress (the tour itself inside it, so a render from state
/// never reads the file) plus the player's own bits. In the canonical view state.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct WalkUi {
    pub tour: caretline_tour::TourState,
    /// The view from before the walkthrough started (`Esc` restores it): the view keys, by alias.
    #[serde(default)]
    pub prior: Option<serde_json::Value>,
    /// The table of contents is open (`t`), its cursor.
    #[serde(default)]
    pub toc: Option<usize>,
    /// `g` typed, then digits: the step number being typed.
    #[serde(default)]
    pub goto: Option<String>,
    /// The step's `checks` that failed when it was shown (the "books changed" band).
    #[serde(default)]
    pub failed: Vec<String>,
}

impl WalkUi {
    /// The step shown (0-based).
    pub fn at(&self) -> usize {
        self.tour.index().unwrap_or(0)
    }

    pub fn len(&self) -> usize {
        self.tour.tour.as_ref().map_or(0, |t| t.steps.len())
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn file(&self) -> &Walkthrough {
        static EMPTY: std::sync::OnceLock<Walkthrough> = std::sync::OnceLock::new();
        self.tour
            .tour
            .as_ref()
            .unwrap_or_else(|| EMPTY.get_or_init(|| wt::new("none", "")))
    }
}

/// The view keys of the canonical `Ui` a step sets (everything that decides what is on screen, nothing
/// about the session: size, theme, layers, the walkthrough itself).
pub const VIEW_KEYS: [&str; 18] = [
    "tab",
    "scope",
    "as_of",
    "filter",
    "full_history",
    "differs_only",
    "overview_sel",
    "overview_top",
    "overview_col",
    "ledger",
    "ic",
    "ic_sort",
    "via",
    "cmp",
    "activity",
    "system",
    "manifest",
    "manifest_tab",
];

/// A walkthrough in the `W` list.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ListItem {
    pub id: String,
    pub title: String,
    pub steps: usize,
    pub created: String,
    /// Why it can't be played (a bad file), if so.
    #[serde(default)]
    pub error: Option<String>,
    /// The parsed file (played on Enter).
    #[serde(default)]
    pub file: Option<Walkthrough>,
}

/// The `W` list.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct WalkList {
    pub items: Vec<ListItem>,
    pub sel: usize,
}

/// A recording in progress (`walkthrough record start`): the steps so far.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Recording {
    pub file: Walkthrough,
}

use crate::app::Effect;
use caretline_tour::{TourEffect, TourOp};
use ownpurse_core::walkthrough::{self as wt, StepHost, Walkthrough};
use serde_json::{Value, json};

/// The neutral view a step's view applies over: the start of the app (Overview, all orgs, no filter, no
/// drill), keeping the as-of date and the shown manifest from `prior`.
fn neutral(prior: &Value) -> Value {
    let mut v = prior.clone();
    let set = |v: &mut Value, k: &str, x: Value| {
        if let Some(m) = v.as_object_mut() {
            m.insert(k.into(), x);
        }
    };
    set(&mut v, "tab", json!("Overview"));
    set(&mut v, "scope", json!("All"));
    set(&mut v, "filter", json!(""));
    set(&mut v, "full_history", json!(false));
    set(&mut v, "differs_only", json!(false));
    set(&mut v, "overview_top", json!(0));
    set(&mut v, "overview_col", json!("Org"));
    set(&mut v, "ledger", json!({}));
    set(&mut v, "ic", json!({}));
    set(&mut v, "ic_sort", json!(0));
    set(&mut v, "via", Value::Null);
    set(&mut v, "cmp", Value::Null);
    v
}

impl App {
    /// The view part of the canonical `Ui` (aliases, never indexes), for recording and restoring.
    pub fn view_part(&self) -> Value {
        let ui = self.canonical_value()["ui"].clone();
        let mut m = serde_json::Map::new();
        for k in VIEW_KEYS {
            if let Some(v) = ui.get(k) {
                m.insert(k.into(), v.clone());
            }
        }
        Value::Object(m)
    }

    /// Set the view keys from `part` (aliases) over the current `Ui`; the rest of the session stays.
    pub fn set_view_part(&mut self, part: &Value) -> Result<(), String> {
        let mut ui = self.canonical_value()["ui"].clone();
        if let (Some(u), Some(p)) = (ui.as_object_mut(), part.as_object()) {
            for (k, v) in p {
                if VIEW_KEYS.contains(&k.as_str()) {
                    u.insert(k.clone(), v.clone());
                } else {
                    return Err(format!(
                        "`{k}` is not a view key (a step sets {}).",
                        VIEW_KEYS.join(", ")
                    ));
                }
            }
        }
        self.ui = crate::canonical::ui_from_value(self, ui)?;
        self.invalidate();
        self.manifest_sync();
        self.prepare();
        Ok(())
    }

    /// `W` / palette "Walkthroughs…": the walkthroughs in the project folder.
    pub fn walk_list_open(&mut self) {
        let Some(dir) = self.source.walkthroughs.clone() else {
            self.refuse(
                "No walkthroughs folder: walkthroughs live beside ownpurse.json (`walkthroughs.dir`)."
                    .into(),
            );
            return;
        };
        let items: Vec<ListItem> = wt::list(&dir)
            .into_iter()
            .map(|(id, w)| match w {
                Ok(w) => ListItem {
                    title: w.title.clone(),
                    steps: w.steps.len(),
                    created: wt::Meta::of(&w).created.unwrap_or_default(),
                    error: None,
                    id,
                    file: Some(w),
                },
                Err(e) => ListItem {
                    title: id.clone(),
                    steps: 0,
                    created: String::new(),
                    error: Some(e),
                    id,
                    file: None,
                },
            })
            .collect();
        if items.is_empty() {
            self.refuse(format!(
                "No walkthroughs yet in {} (an agent records one with `walkthrough record`).",
                self.paths.show(&dir)
            ));
            return;
        }
        self.ui.walk_list = Some(WalkList { items, sel: 0 });
    }

    /// Start playing `w` at its first step; the view from before comes back on `Esc`.
    pub fn walk_start(&mut self, w: Walkthrough, fx: &mut Vec<Effect>) -> Result<(), String> {
        if w.steps.is_empty() {
            return Err(format!("Walkthrough `{}` has no steps.", w.id));
        }
        let prior = match self.ui.walk.as_ref() {
            Some(old) => old.prior.clone(),
            None => Some(self.view_part()),
        };
        if wt::Meta::of(&w).audience.as_deref() == Some("lay") {
            self.ui.lay_mode = true;
        }
        let mut st = WalkUi {
            prior,
            ..Default::default()
        };
        let effects = caretline_tour::apply(&mut st.tour, TourOp::Start(Box::new(w)), 0)
            .map_err(|e| e.to_string())?;
        self.ui.walk = Some(Box::new(st));
        self.ui.walk_list = None;
        self.walk_effects(effects, fx)
    }

    /// A tour op on the walkthrough playing, then its effects.
    pub fn walk_op(&mut self, op: TourOp, fx: &mut Vec<Effect>) -> Result<(), String> {
        let Some(w) = self.ui.walk.as_mut() else {
            return Err("No walkthrough is playing.".into());
        };
        let effects = caretline_tour::apply(&mut w.tour, op, 0).map_err(|e| e.to_string())?;
        self.walk_effects(effects, fx)
    }

    /// caretline-tour's effects, in order: entering a step applies its view (over the neutral view: the
    /// same step always gives the same frame, however it was reached), its checks and narration; its layers
    /// replace the guide's; the end restores the view from before.
    fn walk_effects(
        &mut self,
        effects: Vec<TourEffect>,
        fx: &mut Vec<Effect>,
    ) -> Result<(), String> {
        for e in effects {
            match e {
                // the view is applied with the step change (a step with no host still resets the view)
                TourEffect::Host { .. } => {}
                TourEffect::Layers { layers } => {
                    caretline_tour::replace_guide(&mut self.ui.layers, layers, 0)
                        .map_err(|r| r.to_string())?;
                }
                TourEffect::Step { .. } => self.walk_enter(fx)?,
                TourEffect::Ended { .. } => self.walk_ended(),
            }
        }
        Ok(())
    }

    /// The current step's view, checks and narration.
    fn walk_enter(&mut self, fx: &mut Vec<Effect>) -> Result<(), String> {
        let Some(w) = self.ui.walk.as_ref() else {
            return Ok(());
        };
        let Some(step) = w.tour.current().cloned() else {
            return Ok(());
        };
        let host = StepHost::of(&step);
        let prior = w.prior.clone().unwrap_or_else(|| self.view_part());
        let mut part = neutral(&prior);
        let view = host.view.clone().unwrap_or(Value::Null);
        if let Some(st) = view.get("state").and_then(Value::as_object) {
            for (k, v) in st {
                part[k] = v.clone();
            }
        }
        self.set_view_part(&part)?;
        // the panel and the band first: they decide the room the step's view verbs scroll in
        self.ui.narration = Some(Narration {
            title: wt::step_title(&step),
            body: wt::step_text(&step),
            by: None,
            why: host.why.clone(),
            scroll: 0,
        });
        self.ui.glossary_focus = None;
        let failed = self.walk_checks(&host.checks);
        if let Some(w) = self.ui.walk.as_mut() {
            w.toc = None;
            w.goto = None;
            w.failed = failed;
        }
        if let Some(verbs) = view.get("verbs").and_then(Value::as_array) {
            for words in verbs {
                let words: Vec<String> = match words {
                    Value::Array(a) => a
                        .iter()
                        .map(|x| match x {
                            Value::String(s) => s.clone(),
                            o => o.to_string(),
                        })
                        .collect(),
                    Value::String(s) => s.split_whitespace().map(str::to_string).collect(),
                    _ => continue,
                };
                // the frame the verb acts on (a page, the visible rows) is this step's own, never the
                // frame before it: render it first, as the bridge does between verbs
                self.settle_frame();
                let verb = self.parse_view_verb(&words)?;
                self.apply_verb(verb, fx);
                if let Some(Err((_, m))) = self.agent.outcome.take() {
                    return Err(format!("step {}: `{}`: {m}", step.id, words.join(" ")));
                }
            }
        }
        // the step's visualisation (computed from the ledger now), or none: a step without one closes it
        let viz = match &host.viz {
            Some(v) => Some(Box::new(
                ownpurse_core::viz::Spec::parse(&v.to_string())
                    .map_err(|e| format!("step {}: viz: {}", step.id, e.message))?,
            )),
            None => None,
        };
        if viz.is_some() || self.ui.viz.is_some() {
            self.settle_frame();
            self.apply_verb(crate::agent::Verb::Viz(viz), fx);
            if let Some(Err((_, m))) = self.agent.outcome.take() {
                return Err(format!("step {}: {m}", step.id));
            }
        }
        Ok(())
    }

    /// Draw the current state off screen at its size, so the next view verb sees this frame's rows and
    /// pages (deterministic: a pure render of the state).
    fn settle_frame(&mut self) {
        let (w, h) = self.ui.size;
        let area = ratatui::layout::Rect::new(0, 0, w, h);
        let mut buf = ratatui::buffer::Buffer::empty(area);
        self.hits = crate::ui::render(self, &mut buf, area);
    }

    /// The walkthrough ended (stopped, or past its last step): its layers and narration go, the view from
    /// before it returns.
    fn walk_ended(&mut self) {
        let prior = self.ui.walk.take().and_then(|w| w.prior);
        let _ = caretline_tour::replace_guide(&mut self.ui.layers, vec![], 0);
        if self.ui.viz.is_some() {
            self.viz_close();
        }
        self.ui.narration = None;
        self.ui.glossary_focus = None;
        if let Some(p) = prior
            && let Err(e) = self.set_view_part(&p)
        {
            self.ui.note = Some(format!("the view from before could not be restored: {e}"));
        }
    }

    /// Show step `i` (clamped).
    pub fn walk_go(&mut self, i: usize, fx: &mut Vec<Effect>) -> Result<(), String> {
        let Some(w) = self.ui.walk.as_ref() else {
            return Err("No walkthrough is playing.".into());
        };
        let i = i.min(w.len().saturating_sub(1));
        let id = w.file().steps[i].id.clone();
        self.walk_op(TourOp::To(id), fx)
    }

    /// The step's `checks` that don't hold on the record now (each with what it is now).
    pub fn walk_checks(&self, checks: &[String]) -> Vec<String> {
        if checks.is_empty() {
            return vec![];
        }
        let Some(rec) = self.books.as_ref().map(|b| b.record()) else {
            return vec!["the record can't be read".into()];
        };
        checks
            .iter()
            .filter_map(|c| match wt::parse_check(c) {
                Err(e) => Some(e),
                Ok(chk) => wt::eval_check(&chk, rec, &self.source.orgs)
                    .err()
                    .map(|now| format!("{c} ({now})")),
            })
            .collect()
    }

    /// Leave the walkthrough (`Esc`, `walkthrough stop`).
    pub fn walk_stop(&mut self) {
        let mut fx = Vec::new();
        if self.walk_op(TourOp::Stop, &mut fx).is_err() {
            self.walk_ended();
        }
    }

    /// A view verb from a step (`tab 2`, `select 6140`, …): only the bridge's view verbs, never the
    /// overlay, narration, walkthrough or theme ones.
    fn parse_view_verb(&self, words: &[String]) -> Result<crate::agent::Verb, String> {
        let name = words.first().map(String::as_str).unwrap_or("");
        if !ownpurse_core::bridge::VIEW_ONLY.contains(&name) {
            return Err(format!(
                "`{name}` is not a view verb a walkthrough step may use ({}).",
                ownpurse_core::bridge::VIEW_ONLY.join(", ")
            ));
        }
        ownpurse_core::bridge::check_verb(words).map_err(|(_, m)| m)?;
        self.parse_verb(words).map_err(|(_, m)| m)
    }

    /// The step as recorded now: the narration shown, the view, the layers on screen (guide's and agents').
    pub fn record_step(&self) -> Result<wt::Step, String> {
        let Some(rec) = self.ui.recording.as_ref() else {
            return Err("Not recording: `walkthrough record start <id>` first.".into());
        };
        let mut n = rec.file.steps.len() + 1;
        while rec.file.index(&format!("s{n}")).is_some() {
            n += 1;
        }
        let id = format!("s{n}");
        let mut step = wt::Step::new(&id);
        // what a recorded step is about, kept clear on playback: the selected row and the diff shown
        let mut about: Vec<caretline_layers::Anchor> = Vec::new();
        if let Some(sel) = self.selected_anchor() {
            about.extend(crate::anchors::host_anchor(&format!(
                "row:{}",
                crate::anchors::row_short(&sel)
            )));
        }
        for n in self.hits.anchors.names() {
            if n.starts_with("diff:") || n == "panel:via" {
                about.extend(crate::anchors::host_anchor(n));
            }
        }
        for (i, l) in self.ui.layers.layers.iter().enumerate() {
            let mut sl = step_layer(l);
            sl.id = format!("{id}/{i}");
            if sl.avoid.is_empty() {
                sl.avoid = about
                    .iter()
                    .filter(|a| !l.anchor.contains(a))
                    .cloned()
                    .map(wt::StepAnchor::At)
                    .collect();
            }
            step.layers.push(sl);
        }
        let narr = self.ui.narration.clone().unwrap_or_default();
        step.narration = (!narr.title.is_empty() || !narr.body.is_empty()).then(|| wt::Narration {
            title: (!narr.title.is_empty()).then(|| narr.title.clone()),
            text: narr.body.clone(),
        });
        step.host = StepHost {
            view: Some(json!({"state": self.view_part()})),
            viz: None,
            checks: vec![],
            why: narr.why.clone(),
        }
        .to_value();
        Ok(step)
    }

    /// Player keys while a walkthrough plays (or the glossary, the contents or the list is open). True when
    /// the key was the player's.
    pub fn walk_key(&mut self, key: &crate::keymap::Key, fx: &mut Vec<Effect>) -> bool {
        use crossterm::event::KeyCode as K;
        let plain = key.mods.is_empty();
        if self.ui.overlay.is_some() || self.ui.prompt.is_some() || self.ui.band.is_some() {
            return false;
        }
        // the narration panel scrolls when its text is longer than its room (it is never cut)
        if matches!(key.code, K::PageDown | K::PageUp) && plain && self.ui.narration.is_some() {
            let (n, room) = body_fit(self);
            if n > room {
                if let Some(nar) = self.ui.narration.as_mut() {
                    let max = n - room;
                    nar.scroll = if key.code == K::PageDown {
                        (nar.scroll + room).min(max)
                    } else {
                        nar.scroll.saturating_sub(room)
                    };
                }
                return true;
            }
        }
        // the glossary definition: any key closes it
        if self.ui.glossary_focus.is_some() {
            self.ui.glossary_focus = None;
            return true;
        }
        if let Some(list) = self.ui.walk_list.as_mut() {
            match key.code {
                K::Up | K::Char('k') => list.sel = list.sel.saturating_sub(1),
                K::Down | K::Char('j') => {
                    list.sel = (list.sel + 1).min(list.items.len().saturating_sub(1))
                }
                K::Enter => {
                    let it = list.items[list.sel].clone();
                    match it.file {
                        Some(w) => {
                            if let Err(e) = self.walk_start(w, fx) {
                                self.refuse(e);
                            }
                        }
                        None => self.refuse(format!(
                            "{} can't be played: {}",
                            it.id,
                            it.error.unwrap_or_default()
                        )),
                    }
                }
                _ => self.ui.walk_list = None,
            }
            return true;
        }
        let Some(w) = self.ui.walk.as_ref() else {
            if self.ui.narration.is_some() && key.code == K::Esc && plain {
                self.ui.narration = None;
                return true;
            }
            return false;
        };
        let (at, n, toc, goto) = (w.at(), w.len(), w.toc, w.goto.clone());
        if let Some(cur) = toc {
            match key.code {
                K::Up | K::Char('k') => self.walk_toc(cur.saturating_sub(1)),
                K::Down | K::Char('j') => self.walk_toc((cur + 1).min(n - 1)),
                K::Enter => self.walk_jump(cur, fx),
                _ => {
                    if let Some(w) = self.ui.walk.as_mut() {
                        w.toc = None;
                    }
                }
            }
            return true;
        }
        if let Some(g) = goto {
            match key.code {
                K::Char(c) if c.is_ascii_digit() => {
                    if let Some(w) = self.ui.walk.as_mut() {
                        w.goto = Some(format!("{g}{c}"));
                    }
                }
                K::Enter => {
                    let k: usize = g.parse().unwrap_or(0);
                    if k >= 1 {
                        self.walk_jump(k - 1, fx);
                    } else if let Some(w) = self.ui.walk.as_mut() {
                        w.goto = None;
                    }
                }
                _ => {
                    if let Some(w) = self.ui.walk.as_mut() {
                        w.goto = None;
                    }
                }
            }
            return true;
        }
        match key.code {
            // forward stops at the last step (Esc leaves)
            K::Right | K::Char('n') | K::Char(' ') if plain => {
                if at + 1 < n {
                    self.walk_jump(at + 1, fx);
                }
            }
            K::Left | K::Char('p') if plain => {
                if at > 0 {
                    self.walk_jump(at - 1, fx);
                }
            }
            K::Char(']') if plain => self.walk_jump((at + 1).min(n - 1), fx),
            K::Char('[') if plain => self.walk_jump(at.saturating_sub(1), fx),
            K::Home if plain => self.walk_jump(0, fx),
            K::End if plain => self.walk_jump(n - 1, fx),
            K::Char('g') if plain => {
                if let Some(w) = self.ui.walk.as_mut() {
                    w.goto = Some(String::new());
                }
            }
            K::Char('t') if plain => self.walk_toc(at),
            K::Char('r') if plain => self.walk_jump(at, fx),
            K::Char('?') if plain => {
                let first = self
                    .ui
                    .narration
                    .as_ref()
                    .and_then(|n| terms(&n.body).first().cloned());
                match first {
                    Some(t) => self.ui.glossary_focus = Some(t),
                    None => return false,
                }
            }
            K::Esc if plain => self.walk_stop(),
            _ => return false,
        }
        true
    }

    fn walk_toc(&mut self, cur: usize) {
        if let Some(w) = self.ui.walk.as_mut() {
            w.toc = Some(cur);
        }
    }

    /// Jump to step `i`; a step that can't be shown says why (the walkthrough stays where it was).
    pub fn walk_jump(&mut self, i: usize, fx: &mut Vec<Effect>) {
        if let Err(e) = self.walk_go(i, fx) {
            self.refuse(e);
        }
    }
}

/// A caretline layer as a step layer (recording): the agent's attribution leaves the title; a layer with
/// no content (a ring, a spotlight) gets the ring kind.
pub fn step_layer(l: &caretline_layers::Layer) -> wt::StepLayer {
    let (kind, data) = match &l.content {
        Some(c) => {
            let mut d = c.data.clone();
            if let Some(t) = d.get("title").and_then(Value::as_str) {
                let t = strip_by(t);
                if t.is_empty() {
                    if let Some(m) = d.as_object_mut() {
                        m.remove("title");
                    }
                } else {
                    d["title"] = json!(t);
                }
            }
            (Some(c.kind.clone()), d)
        }
        None => (None, Value::Null),
    };
    wt::StepLayer {
        id: String::new(),
        anchor: l.anchor.iter().cloned().map(wt::StepAnchor::At).collect(),
        avoid: l.avoid.iter().cloned().map(wt::StepAnchor::At).collect(),
        kind,
        data,
        place: wt::Place {
            sides: l.place.clone(),
            max_width: l.max_width,
            arrow: l.arrow,
            ring: l.ring.clone(),
            spotlight: l.spotlight.clone(),
            hide_off_screen: l.hide_off_screen,
            head: l.head,
        },
        capture: false,
    }
}

/// `◆ name · Title` → `Title` (`◆ name` → ``).
pub fn strip_by(t: &str) -> String {
    match t.strip_prefix("◆ ") {
        Some(rest) => rest
            .split_once(" · ")
            .map(|(_, x)| x.to_string())
            .unwrap_or_default(),
        None => t.to_string(),
    }
}

/// One finding of `walkthrough validate`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Finding {
    /// 1-based step number and its id.
    pub step: usize,
    pub step_id: String,
    /// `WxH`, or empty for size-independent findings.
    pub size: String,
    /// error | warning
    pub level: &'static str,
    /// caretline-tour's codes (`not_found`, `off_screen`, `no_way`, `unrendered`, …, and its lint) plus ours:
    /// `strip`, `protected`, `check`, `glossary`, `view`.
    pub kind: String,
    pub detail: String,
}

/// The sizes `validate` lays every step out at.
pub const VALIDATE_SIZES: [(u16, u16); 3] = [(140, 40), (100, 30), (80, 24)];

/// Validate a walkthrough headlessly: caretline-tour's lint, then every step planned at each size by
/// caretline-tour's `plan_steps` over our own frames (the step's view applied and drawn), plus ours: no strip
/// in place of a box, no box over a protected region, every check holds against the record, every glossary
/// link exists. `make` builds a fresh app.
pub fn validate(make: &dyn Fn() -> App, w: &Walkthrough) -> Vec<Finding> {
    let mut out = Vec::new();
    let num = |id: &str| w.index(id).map_or(0, |i| i + 1);
    let level = |l: caretline_tour::Level| match l {
        caretline_tour::Level::Error => "error",
        caretline_tour::Level::Warning => "warning",
    };
    for p in wt::check(w) {
        let id = p.step.clone().unwrap_or_default();
        out.push(Finding {
            step: num(&id),
            step_id: id,
            size: String::new(),
            level: level(p.level),
            kind: p.code.clone(),
            detail: p.detail.clone(),
        });
    }
    let probe = make();
    for (i, s) in w.steps.iter().enumerate() {
        let mut push = |kind: &str, detail: String| {
            out.push(Finding {
                step: i + 1,
                step_id: s.id.clone(),
                size: String::new(),
                level: "error",
                kind: kind.into(),
                detail,
            })
        };
        for t in terms(&wt::step_text(s)) {
            if probe.glossary.get(&t).is_none() {
                push("glossary", format!("[{t}] is not in the glossary"));
            }
        }
        if let Some(h) = &s.host
            && let Err(e) = serde_json::from_value::<StepHost>(h.clone())
        {
            push("view", format!("host: {e}"));
        }
        for f in probe.walk_checks(&StepHost::of(s).checks) {
            push("check", f);
        }
    }
    let renderers = crate::overlay::renderers();
    for (sw, sh) in VALIDATE_SIZES {
        let mut app = make();
        crate::snapshot::render(&mut app, sw, sh);
        let mut fx = Vec::new();
        if let Err(e) = app.walk_start(w.clone(), &mut fx) {
            out.push(Finding {
                step: 1,
                step_id: w.steps.first().map(|s| s.id.clone()).unwrap_or_default(),
                size: format!("{sw}x{sh}"),
                level: "error",
                kind: "view".into(),
                detail: e,
            });
            continue;
        }
        let mut ours: Vec<Finding> = Vec::new();
        let plans = caretline_tour::plan_steps(
            w,
            &[(sw, sh)],
            &renderers,
            &mut |step, w_, h_, go| {
                let i = w.index(&step.id).unwrap_or(0);
                let mut fx = Vec::new();
                if let Err(e) = app.walk_go(i, &mut fx) {
                    ours.push(Finding {
                        step: i + 1,
                        step_id: step.id.clone(),
                        size: format!("{w_}x{h_}"),
                        level: "error",
                        kind: "view".into(),
                        detail: e,
                    });
                    return;
                }
                let buf = crate::snapshot::render(&mut app, w_, h_);
                let _ = &buf;
                let protected: Vec<caretline_layers::Rect> = crate::overlay::PROTECTED
                    .iter()
                    .filter_map(|n| app.hits.anchors.get(n))
                    .flat_map(|r| r.rects.clone())
                    .collect();
                if let Some(p) = app.hits.plan.as_ref() {
                    for l in &p.layers {
                        if l.covers_avoid > 0 {
                            ours.push(Finding {
                                step: i + 1,
                                step_id: step.id.clone(),
                                size: format!("{w_}x{h_}"),
                                level: "warning",
                                kind: "covers_avoid".into(),
                                detail: format!(
                                    "layer {}: its box covers {} cells of what the step explains (nothing clear was in reach)",
                                    l.id, l.covers_avoid
                                ),
                            });
                        }
                        if l.mode == Some(caretline_layers::Mode::Strip) {
                            ours.push(Finding {
                                step: i + 1,
                                step_id: step.id.clone(),
                                size: format!("{w_}x{h_}"),
                                level: "error",
                                kind: "strip".into(),
                                detail: format!(
                                    "layer {}: no room for a box beside its anchor",
                                    l.id
                                ),
                            });
                        }
                        if let Some(b) = l.rect
                            && protected.iter().any(|x| x.intersects(&b))
                        {
                            ours.push(Finding {
                                step: i + 1,
                                step_id: step.id.clone(),
                                size: format!("{w_}x{h_}"),
                                level: "error",
                                kind: "protected".into(),
                                detail: format!(
                                    "layer {}: its box covers a protected region",
                                    l.id
                                ),
                            });
                        }
                    }
                }
                // the scene without our own drawing of the layers: placement sees the view alone
                let saved = app.ui.layers.hidden;
                app.ui.layers.hidden = true;
                let buf = crate::snapshot::render(&mut app, w_, h_);
                let grid = crate::overlay::grid_avoiding(
                    &app,
                    &buf,
                    app.hits.layer_area.unwrap_or(buf.area),
                    &app.hits.anchors,
                );
                go(&grid, app.hits.anchors.map());
                app.ui.layers.hidden = saved;
            },
        );
        for sp in plans {
            for p in sp.problems {
                out.push(Finding {
                    step: num(&sp.step),
                    step_id: sp.step.clone(),
                    size: format!("{}x{}", sp.width, sp.height),
                    level: level(p.level),
                    kind: p.code.clone(),
                    detail: format!(
                        "{}{}",
                        p.layer.map(|l| format!("layer {l}: ")).unwrap_or_default(),
                        p.detail
                    ),
                });
            }
        }
        out.extend(ours);
    }
    out
}

/// Every step's frame at `size` (a fresh app per walkthrough; steps in order): (file stem, buffer).
pub fn render_steps(
    make: &dyn Fn() -> App,
    w: &Walkthrough,
    size: (u16, u16),
) -> Result<Vec<(String, ratatui::buffer::Buffer)>, String> {
    let mut app = make();
    crate::snapshot::render(&mut app, size.0, size.1);
    let mut fx = Vec::new();
    app.walk_start(w.clone(), &mut fx)?;
    let mut out = Vec::new();
    for (i, s) in w.steps.iter().enumerate() {
        app.walk_go(i, &mut fx)?;
        let buf = crate::snapshot::render(&mut app, size.0, size.1);
        out.push((format!("{:02}-{}", i + 1, s.id), buf));
    }
    Ok(out)
}
