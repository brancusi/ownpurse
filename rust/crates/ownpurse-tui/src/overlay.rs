//! Overlays (walkthroughs design §1, §3): hints, rings and spotlights over our own drawing, placed by
//! caretline-layers and drawn here in our theme.
//!
//! Every frame, after the views drew themselves (recording [`Anchors`](crate::anchors::Anchors)): the row
//! and cell anchors are filled from the frame's row targets, the screen becomes a caretline `Grid` (what text
//! is where, the protected regions no box may cover), `plan` places the layers in `Ui.layers`, and the plan
//! is drawn: dimming, rings, arrows, boxes, edge chips. The plan rides in the frame's `Hits` so replies can
//! say where each layer landed.
//!
//! Layers are owned by `guide` (a walkthrough) or `agent:<name>` (the bridge). An agent's box is attributed
//! (`◆ <name>` in its top border, drawn here) and never covers a protected region: the header writes badge,
//! the manifest stage strip, CHECK issues, the apply command and its hashes, warnings and bands.

use crate::anchors::{Anchors, host_anchor, row_aliases, row_key, row_short};
use crate::app::App;
use crate::hit::{Hits, Target};
use crate::layout::{width, wrap};
use crate::theme::Theme;
use crate::theme_tokens::Token;
use caretline_layers as cl;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use serde_json::{Value, json};

/// The anchors of regions no layer box may cover (they're recorded under these names while drawing).
pub const PROTECTED: [&str; 7] = [
    "chip:writes",
    "chip:stage",
    "panel:issues",
    "panel:apply",
    "panel:warnings",
    "panel:band",
    "panel:check",
];

/// The longest hint text and title an agent may send (characters), and its lines.
pub const TEXT_MAX: usize = 280;
pub const TITLE_MAX: usize = 40;

/// Our policy for agent layers: sized like caretline's suggested defaults, but they persist until cleared
/// (no clock in a frame: a snapshot must not depend on when it was taken), may spotlight (the `spotlight`
/// verb), and never capture.
pub fn limits() -> cl::Limits {
    cl::Limits {
        agent_layers: Some(6),
        body_chars: Some(TEXT_MAX),
        body_lines: Some(6),
        title_chars: Some(TITLE_MAX + 24),
        content_bytes: Some(1_024),
        no_agent_capture: true,
        ..cl::Limits::none()
    }
}

/// The lines a hint shows inside its border: the title (bold) then the wrapped text.
pub fn hint_lines(data: &Value, inner: usize) -> Vec<String> {
    let title = data.get("title").and_then(Value::as_str).unwrap_or("");
    let text = data.get("text").and_then(Value::as_str).unwrap_or("");
    let mut lines: Vec<String> = Vec::new();
    if !title.is_empty() {
        lines.extend(wrap(title, inner));
    }
    if !text.is_empty() {
        for para in text.split('\n') {
            lines.extend(wrap(para, inner));
        }
    }
    lines
}

/// The narrowest hint box worth drawing (borders included): narrower would wrap a word a line.
pub const MIN_BOX_W: u16 = 18;

/// Our `hint` renderer: a bordered box, one column of padding each side, at most the room offered.
pub struct HintBox;

impl cl::Renderer for HintBox {
    fn measure(&self, cx: &cl::MeasureCtx) -> cl::Size {
        let (data, avail) = (cx.data, cx.avail);
        // too little room for a readable box on this side: none (placement tries another side)
        if avail.w < MIN_BOX_W {
            return cl::Size::new(0, 0);
        }
        let inner = avail.w.saturating_sub(4).max(8) as usize;
        let lines = hint_lines(data, inner);
        if lines.is_empty() {
            return cl::Size::new(0, 0);
        }
        let w = lines.iter().map(|l| width(l)).max().unwrap_or(0).min(inner);
        let w = (w + 4).min(avail.w as usize);
        let h = lines.len() + 2;
        // a box that would cut its own text is no box for this side (placement tries the next)
        if h > avail.h as usize {
            return cl::Size::new(0, 0);
        }
        cl::Size::new(w as u16, h as u16)
    }

    fn chip(&self, _data: &Value, _anchor: &cl::Anchor, _off: cl::Off) -> cl::Size {
        cl::Size::new(8, 1)
    }
}

pub fn renderers() -> cl::Renderers {
    cl::Renderers::new().register(cl::HINT, HintBox)
}

fn ratatui_rect(r: cl::Rect) -> Rect {
    Rect::new(r.x, r.y, r.w, r.h)
}

fn cl_rect(r: Rect) -> cl::Rect {
    cl::Rect::new(r.x, r.y, r.width, r.height)
}

/// Rows and cells from the frame's row targets: `row:<anchor>` and `cell:<anchor>/<field>`.
pub fn record_rows(app: &App, hits: &mut Hits) {
    record_rows_in(app, hits, None);
}

/// `record_rows`, each row trimmed to its content (first to last glyph drawn) when the frame is at hand:
/// placement and arrows aim at the text, not at the blank end of a wide row.
pub fn record_rows_in(app: &App, hits: &mut Hits, buf: Option<&Buffer>) {
    let targets: Vec<(Rect, Target)> = hits.targets.clone();
    for (full, t) in targets {
        let rect = buf
            .and_then(|b| text_extent(b, cl_rect(full)))
            .map_or(full, ratatui_rect);
        match t {
            Target::Row(i) => {
                if let Some((a, _, _)) = app.row_anchor_pub(i) {
                    hits.anchors.row(&row_short(&a), rect);
                    hits.anchors.alias(&format!("row:{}", row_key(&a)), rect);
                    for al in row_aliases(&a) {
                        hits.anchors.alias(&format!("row:{al}"), rect);
                    }
                }
            }
            Target::Cell(i, c) => {
                if let Some((a, f, _)) = app.row_anchor_pub(i) {
                    let key = row_short(&a);
                    let col = f
                        .as_object()
                        .and_then(|m| m.keys().nth(c).cloned())
                        .unwrap_or_else(|| c.to_string());
                    hits.anchors.cell(&key, &col, rect);
                    // a row drawn as cells (Compare, Overview parity) is a row too
                    hits.anchors.alias(&format!("row:{key}"), rect);
                }
            }
            _ => {}
        }
    }
}

/// Row anchors the layers ask for that aren't drawn: off screen above or below when the row exists in the
/// screen's whole list (never computed for anything else).
fn record_off_rows(app: &App, hits: &mut Hits, area: Rect) {
    let wanted: Vec<String> = app
        .ui
        .layers
        .layers
        .iter()
        .flat_map(|l| l.anchor.iter())
        .filter_map(|a| match a {
            cl::Anchor::Host { kind, key } if kind == "row" => Some(format!("row:{key}")),
            _ => None,
        })
        .filter(|n| hits.anchors.get(n).is_none())
        .collect();
    if wanted.is_empty() {
        return;
    }
    let drawn: Vec<usize> = hits
        .targets
        .iter()
        .filter_map(|(_, t)| match t {
            Target::Row(i) | Target::Cell(i, _) => Some(*i),
            _ => None,
        })
        .collect();
    let top = app.screen_top_pub();
    let first = top + drawn.iter().copied().min().unwrap_or(0);
    let total = app.total_rows();
    for (n, a, _, _) in app.rows_slice(0..total) {
        let names: Vec<String> = std::iter::once(row_key(&a))
            .chain(row_aliases(&a))
            .map(|k| format!("row:{k}"))
            .collect();
        if let Some(name) = names.iter().find(|n| wanted.contains(n)) {
            let name = name.clone();
            let off = if n - 1 < first {
                cl::Off::Above {
                    x: Some(area.x + 2),
                }
            } else {
                cl::Off::Below {
                    x: Some(area.x + 2),
                }
            };
            hits.anchors.put_off(&name, off);
        }
    }
}

/// The screen as placement sees it: every drawn glyph is text; the protected regions are taken.
pub fn grid(buf: &Buffer, area: Rect, anchors: &Anchors) -> cl::Grid {
    let full = buf.area;
    let mut g = cl::Grid::new(full.width, full.height).with_area(cl_rect(area));
    for y in full.top()..full.bottom() {
        for x in full.left()..full.right() {
            let s = buf[(x, y)].symbol();
            if !s.trim().is_empty() {
                g.mark_text(x, y, s);
            }
        }
    }
    let protect: Vec<cl::Rect> = PROTECTED
        .iter()
        .filter_map(|n| anchors.get(n))
        .flat_map(|r| r.rects.iter().copied())
        .collect();
    g.with_protect(protect)
}

/// Plan the layers over this frame and draw them. `area` is where layers may go (between the tab rule and
/// the footer).
pub fn draw(app: &App, buf: &mut Buffer, area: Rect, hits: &mut Hits) {
    hits.layer_area = Some(area);
    // no layers: no overlay work at all (row anchors are derived on demand: `App::anchor_names`)
    if app.ui.layers.layers.is_empty() {
        return;
    }
    record_rows_in(app, hits, Some(buf));
    record_ranges(app, hits);
    record_off_rows(app, hits, area);
    let plan = plan_avoiding(app, buf, area, &hits.anchors);
    paint(&app.ui.theme, buf, &plan, &app.ui.layers);
    hits.plan = Some(plan);
}

fn set(buf: &mut Buffer, x: u16, y: u16, s: &str, style: Style) {
    if x < buf.area.right() && y < buf.area.bottom() {
        buf[(x, y)].set_symbol(s).set_style(style);
    }
}

/// Draw a plan in our theme: dimming, rings, arrows, boxes with attribution, edge chips.
pub fn paint(th: &Theme, buf: &mut Buffer, p: &cl::Plan, layers: &cl::Layers) {
    let ascii = th.ascii;
    // dimming: the text quiets to `text.4` (NO_COLOR: DIM)
    if !p.spots.is_empty() {
        let dim = th.spotlight_dim();
        for y in buf.area.top()..buf.area.bottom() {
            for x in buf.area.left()..buf.area.right() {
                if p.dimmed(x, y) {
                    let c = &mut buf[(x, y)];
                    c.set_style(Style::default().remove_modifier(Modifier::BOLD).patch(dim));
                }
            }
        }
        // no colour to dim with: a gutter mark beside every row the spotlight leaves lit
        if th.palette_free() {
            let mark = if ascii { "|" } else { "┃" };
            for sp in &p.spots {
                for hole in &sp.holes {
                    let x = hole.x.saturating_sub(1);
                    for y in hole.y..hole.bottom() {
                        set(buf, x, y, mark, th.fg(Token::Accent));
                    }
                }
            }
        }
    }
    for l in &p.layers {
        let agent = l.owner.is_agent();
        let tok = if agent { Token::Info } else { Token::Accent };
        let edge = th.fg(tok).patch(th.raised());
        // ring: the anchor's cells in the accent tint, underlined (NO_COLOR: reversed); ASCII marks its ends
        for r in &l.ring {
            let r = ratatui_rect(*r);
            for y in r.top()..r.bottom() {
                for x in r.left()..r.right() {
                    if x < buf.area.right() && y < buf.area.bottom() {
                        let c = &mut buf[(x, y)];
                        let s = if th.palette_free() {
                            Style::default().add_modifier(Modifier::REVERSED)
                        } else {
                            th.bg(Token::AccentTint).add_modifier(Modifier::UNDERLINED)
                        };
                        c.set_style(s);
                    }
                }
                if ascii {
                    if r.left() > 0 && buf[(r.left() - 1, y)].symbol() == " " {
                        set(buf, r.left() - 1, y, "[", th.fg(tok));
                    }
                    if r.right() < buf.area.right() && buf[(r.right(), y)].symbol() == " " {
                        set(buf, r.right(), y, "]", th.fg(tok));
                    }
                }
            }
        }
        if let Some(rt) = &l.route {
            let n = rt.steps.len();
            for (k, s) in rt.steps.iter().enumerate() {
                use cl::Dir;
                let g = if k + 1 == n {
                    match (s.leave, ascii) {
                        (Dir::Up, false) => "▲",
                        (Dir::Down, false) => "▼",
                        (Dir::Left, false) => "◀",
                        (Dir::Right, false) => "▶",
                        (Dir::Up, true) => "^",
                        (Dir::Down, true) => "v",
                        (Dir::Left, true) => "<",
                        (Dir::Right, true) => ">",
                    }
                } else if s.enter == s.leave {
                    match (matches!(s.enter, Dir::Up | Dir::Down), ascii) {
                        (true, false) => "│",
                        (false, false) => "─",
                        (true, true) => "|",
                        (false, true) => "-",
                    }
                } else if ascii {
                    "+"
                } else {
                    match (s.enter, s.leave) {
                        (Dir::Right, Dir::Down) | (Dir::Up, Dir::Left) => "╮",
                        (Dir::Left, Dir::Down) | (Dir::Up, Dir::Right) => "╭",
                        (Dir::Right, Dir::Up) | (Dir::Down, Dir::Left) => "╯",
                        _ => "╰",
                    }
                };
                set(buf, s.x, s.y, g, th.fg(tok));
            }
        }
        if let Some(c) = l.chip {
            let c = ratatui_rect(c);
            let arrow = match l.anchor.as_ref().and_then(|a| a.off) {
                Some(cl::Off::Above { .. }) => th.text("↑ here"),
                Some(cl::Off::Below { .. }) => th.text("↓ here"),
                Some(cl::Off::Left { .. }) => th.text("← here"),
                _ => th.text("→ here"),
            };
            let s = format!(" {arrow} ");
            crate::widgets::fill(buf, c, th.raised());
            crate::widgets::put(
                buf,
                c.x,
                c.y,
                vec![ratatui::text::Span::styled(s, edge)],
                c.width,
                th.glyphs().ellipsis,
            );
        }
        let (Some(r), Some(content)) = (l.rect, layers.get(&l.id).and_then(|x| x.content.as_ref()))
        else {
            continue;
        };
        let r = ratatui_rect(r);
        let fill = th.fg(Token::Text).patch(th.raised());
        crate::widgets::fill(buf, r, th.raised());
        let inner = r.width.saturating_sub(4) as usize;
        let lines = hint_lines(&content.data, inner.max(1));
        let has_title = content
            .data
            .get("title")
            .and_then(Value::as_str)
            .is_some_and(|t| !t.is_empty());
        if l.mode == Some(cl::Mode::Strip) {
            let s = lines.join(" ");
            crate::widgets::put(
                buf,
                r.x + 1,
                r.y,
                vec![ratatui::text::Span::styled(th.text(&s), fill)],
                r.width.saturating_sub(1),
                th.glyphs().ellipsis,
            );
            continue;
        }
        if r.width < 4 || r.height < 2 {
            continue;
        }
        let (h, v, tl, tr, bl, br) = if ascii {
            ("-", "|", "+", "+", "+", "+")
        } else {
            ("─", "│", "╭", "╮", "╰", "╯")
        };
        let (x1, y1) = (r.right() - 1, r.bottom() - 1);
        for x in r.left() + 1..x1 {
            set(buf, x, r.top(), h, edge);
            set(buf, x, y1, h, edge);
        }
        for y in r.top() + 1..y1 {
            set(buf, r.left(), y, v, edge);
            set(buf, x1, y, v, edge);
        }
        set(buf, r.left(), r.top(), tl, edge);
        set(buf, x1, r.top(), tr, edge);
        set(buf, r.left(), y1, bl, edge);
        set(buf, x1, y1, br, edge);
        // where the arrow (or a docked chip) meets the border
        if let Some(rt) = &l.route {
            let j = match (rt.attach.edge, ascii) {
                (_, true) => "+",
                (cl::Edge::Top, false) => "┴",
                (cl::Edge::Bottom, false) => "┬",
                (cl::Edge::Left, false) => "┤",
                (cl::Edge::Right, false) => "├",
            };
            set(buf, rt.junction.0, rt.junction.1, j, edge);
        }
        for (k, line) in lines.iter().enumerate().take(r.height as usize - 2) {
            let style = if k == 0 && has_title {
                th.fg(tok).patch(th.raised()).add_modifier(Modifier::BOLD)
            } else {
                fill
            };
            buf.set_stringn(r.x + 2, r.y + 1 + k as u16, th.text(line), inner, style);
        }
    }
}

/// An anchor argument: a name (`row:code=6140`, `chip:stage`, `diff:/Date`), `#n` (the n-th visible row),
/// or a bare row anchor (`code=6140`).
pub fn parse_anchor(app: &App, arg: &str) -> Result<cl::Anchor, String> {
    if let Some(n) = arg.strip_prefix('#') {
        let n: usize = n
            .parse()
            .map_err(|_| format!("`{arg}`: a row number is #1, #2, …"))?;
        let rows = app.visible_rows();
        let (_, a, _) = rows
            .get(n.wrapping_sub(1))
            .ok_or_else(|| format!("There is no visible row #{n} ({} shown).", rows.len()))?;
        return host_anchor(&format!("row:{}", row_short(a))).ok_or_else(|| "no anchor".into());
    }
    if arg.starts_with('{') {
        let v: Value = serde_json::from_str(arg).map_err(|e| format!("`{arg}`: {e}"))?;
        // a caretline anchor (`{"host": …}`) or a row's anchor object (`{"code": "6140"}`)
        if let Ok(a) = serde_json::from_value::<cl::Anchor>(v.clone()) {
            return Ok(a);
        }
        return host_anchor(&format!("row:{}", row_key(&v))).ok_or_else(|| "no anchor".into());
    }
    match crate::anchors::split(arg) {
        Some((kind, _)) if ["row", "cell", "diff", "chip", "panel", "viz"].contains(&kind) => {
            host_anchor(arg).ok_or_else(|| "no anchor".into())
        }
        _ if arg.contains('=') => {
            host_anchor(&format!("row:{arg}")).ok_or_else(|| "no anchor".into())
        }
        _ => Err(format!(
            "`{arg}` is not an anchor: row:<anchor>, cell:<row>/<column>, diff:<path>, chip:<name>, panel:<name>, viz:<id>, or #n."
        )),
    }
}

/// The text form of a caretline anchor (`Host` → `kind:key`; others as JSON).
pub fn anchor_name(a: &cl::Anchor) -> String {
    match a {
        cl::Anchor::Host { kind, key } => format!("{kind}:{key}"),
        o => serde_json::to_string(o).unwrap_or_default(),
    }
}

/// Where a layer landed in a plan, for replies and `ui state`: `{"layer", "anchor", "landed": "on_screen"
/// | "off_above" | "off_below" | "not_found", "rect"?, "box"?, "reason"?}`.
pub fn landed(plan: Option<&cl::Plan>, layer: &cl::Layer, anchors: &Anchors) -> Value {
    let name = layer.anchor.first().map(anchor_name).unwrap_or_default();
    let mut v = json!({"layer": layer.id, "anchor": name});
    let Some(p) = plan else {
        v["landed"] = json!("not_found");
        v["reason"] = json!("nothing drawn yet");
        return v;
    };
    if p.missing.contains(&layer.id) {
        v["landed"] = json!("not_found");
        v["reason"] = json!(if anchors.is_empty() {
            "nothing on this screen can be pointed at".to_string()
        } else {
            format!("`{name}` is not on this screen (tab, level or filter)")
        });
        return v;
    }
    let Some(pl) = p.layers.iter().find(|l| l.id == layer.id) else {
        v["landed"] = json!("not_found");
        v["reason"] = json!("hidden");
        return v;
    };
    match pl.anchor.as_ref() {
        Some(r) if !r.rects.is_empty() => {
            let b = r.bounds();
            v["landed"] = json!("on_screen");
            v["rect"] = json!([b.x, b.y, b.w, b.h]);
        }
        Some(r) => {
            v["landed"] = json!(match r.off {
                Some(cl::Off::Above { .. }) => "off_above",
                Some(cl::Off::Below { .. }) => "off_below",
                Some(cl::Off::Left { .. }) => "off_left",
                _ => "off_right",
            });
        }
        None => {
            v["landed"] = json!("on_screen");
        }
    }
    if let Some(r) = pl.rect {
        v["box"] = json!([r.x, r.y, r.w, r.h]);
        if pl.mode == Some(cl::Mode::Strip) {
            v["strip"] = json!(true);
        }
    }
    if let Some(n) = pl.no_arrow {
        v["no_arrow"] = serde_json::to_value(n).unwrap_or_default();
    }
    v
}

/// The overlay, narration and walkthrough verbs of the bridge (walkthroughs design §3).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GuideVerb {
    Hint {
        anchor: cl::Anchor,
        title: Option<String>,
        text: String,
        arrow: bool,
        ring: bool,
        spotlight: bool,
        /// What the box and its arrow keep off (`avoid=<anchor>,…`).
        avoid: Vec<cl::Anchor>,
    },
    Highlight(cl::Anchor),
    Spotlight(Vec<cl::Anchor>),
    /// A layer id, or every agent layer (None / `all`).
    Clear(Option<String>),
    Narrate(Option<(String, String)>),
    Play(Box<ownpurse_core::walkthrough::Walkthrough>),
    /// 0-based.
    Step(usize),
    Next,
    Prev,
    Stop,
    RecordStart {
        id: String,
        title: String,
    },
    RecordStep,
    RecordStop,
}

impl GuideVerb {
    /// Overlays and narration are cosmetic: they move no focus and touch no input.
    pub fn cosmetic(&self) -> bool {
        matches!(
            self,
            GuideVerb::Hint { .. }
                | GuideVerb::Highlight(_)
                | GuideVerb::Spotlight(_)
                | GuideVerb::Clear(_)
                | GuideVerb::Narrate(_)
        )
    }
}

impl App {
    /// Parse a guide verb's words (`core::bridge::check_verb` passed).
    pub fn parse_guide_verb(&self, words: &[String]) -> Result<GuideVerb, (String, String)> {
        let usage = |m: String| ("usage".to_string(), m);
        let args = &words[1..];
        Ok(match words[0].as_str() {
            "hint" => {
                let anchor = parse_anchor(self, &args[0]).map_err(usage)?;
                let (mut title, mut text) = (None, Vec::new());
                let (mut arrow, mut ring, mut spotlight) = (true, true, false);
                let mut avoid = Vec::new();
                let mut i = 1;
                while i < args.len() {
                    match args[i].as_str() {
                        "--title" => {
                            title = Some(
                                args.get(i + 1)
                                    .cloned()
                                    .ok_or_else(|| usage("`--title` needs a title.".into()))?,
                            );
                            i += 1;
                        }
                        // the bare forms (`title=…`, `no-arrow`, …) pass through shells and the CLI's own flag parser
                        t if t.starts_with("title=") => {
                            title = Some(t["title=".len()..].to_string())
                        }
                        t if t.starts_with("avoid=") => {
                            for a in t["avoid=".len()..].split(',').filter(|a| !a.is_empty()) {
                                avoid.push(parse_anchor(self, a).map_err(usage)?);
                            }
                        }
                        "--arrow" | "arrow" => arrow = true,
                        "--no-arrow" | "no-arrow" => arrow = false,
                        "--ring" | "ring" => ring = true,
                        "--no-ring" | "no-ring" => ring = false,
                        "--spotlight" | "spotlight" => spotlight = true,
                        f if f.starts_with("--") => {
                            return Err(usage(format!(
                                "Unknown `hint` option `{f}`: title=T, avoid=A,B, no-arrow, no-ring, spotlight (or --title T, --arrow, --no-arrow, --ring, --no-ring, --spotlight)."
                            )));
                        }
                        w => text.push(w.to_string()),
                    }
                    i += 1;
                }
                let text = text.join(" ");
                if text.chars().count() > TEXT_MAX || text.split('\n').count() > 6 {
                    return Err((
                        "too_long".into(),
                        format!("A hint's text is at most {TEXT_MAX} characters and 6 lines."),
                    ));
                }
                if title
                    .as_ref()
                    .is_some_and(|t| t.chars().count() > TITLE_MAX)
                {
                    return Err((
                        "too_long".into(),
                        format!("A hint's title is at most {TITLE_MAX} characters."),
                    ));
                }
                GuideVerb::Hint {
                    anchor,
                    title,
                    text,
                    arrow,
                    ring,
                    spotlight,
                    avoid,
                }
            }
            "highlight" => GuideVerb::Highlight(parse_anchor(self, &args[0]).map_err(usage)?),
            "spotlight" => GuideVerb::Spotlight(
                args.iter()
                    .map(|a| parse_anchor(self, a))
                    .collect::<Result<_, _>>()
                    .map_err(usage)?,
            ),
            "clear-hints" => {
                GuideVerb::Clear(args.first().filter(|a| a.as_str() != "all").cloned())
            }
            "narrate" if args[0] == "clear" && args.len() == 1 => GuideVerb::Narrate(None),
            "narrate" => {
                if args[0].chars().count() > 80 || args[1].chars().count() > 1_000 {
                    return Err((
                        "too_long".into(),
                        "A narration's title is at most 80 characters and its body 1,000.".into(),
                    ));
                }
                GuideVerb::Narrate(Some((args[0].clone(), args[1].clone())))
            }
            "walkthrough" => match (args[0].as_str(), args.get(1).map(String::as_str)) {
                ("play", Some(id)) => {
                    let dir = self.source.walkthroughs.clone().ok_or_else(|| {
                        (
                            "not_found".to_string(),
                            "No walkthroughs folder (it lives beside ownpurse.json).".to_string(),
                        )
                    })?;
                    let w = ownpurse_core::walkthrough::read(&dir, id)
                        .map_err(|e| ("not_found".to_string(), e))?;
                    GuideVerb::Play(Box::new(w))
                }
                ("step", Some(n)) => GuideVerb::Step(n.parse::<usize>().unwrap_or(1) - 1),
                ("next", _) => GuideVerb::Next,
                ("prev", _) => GuideVerb::Prev,
                ("stop", _) => GuideVerb::Stop,
                ("record", Some("start")) => {
                    let id = args[2].clone();
                    if !ownpurse_core::walkthrough::valid_id(&id) {
                        return Err(usage(format!(
                            "`{id}` is not a walkthrough id (letters, digits, - and _)."
                        )));
                    }
                    let bare = args
                        .iter()
                        .find_map(|a| a.strip_prefix("title=").map(str::to_string));
                    let title = match (args.iter().position(|a| a == "--title"), bare) {
                        (_, Some(t)) => t,
                        (Some(k), None) => args
                            .get(k + 1)
                            .cloned()
                            .ok_or_else(|| usage("`--title` needs a title.".into()))?,
                        (None, None) => id.clone(),
                    };
                    GuideVerb::RecordStart { id, title }
                }
                ("record", Some("step")) => GuideVerb::RecordStep,
                ("record", Some("stop")) => GuideVerb::RecordStop,
                _ => return Err(usage("Unknown `walkthrough` form.".into())),
            },
            other => return Err(usage(format!("Unknown verb `{other}`."))),
        })
    }

    fn agent_name(&self) -> String {
        self.agent_client
            .clone()
            .filter(|c| !c.is_empty())
            .unwrap_or_else(|| "agent".into())
    }

    fn push_agent_layer(&mut self, layer: cl::Layer) -> Result<String, (String, String)> {
        let who = self.agent_name();
        let now_ms = u64::try_from(self.now).unwrap_or(0).saturating_mul(1000);
        let applied = cl::apply(
            &mut self.ui.layers,
            cl::LayerOp::Push(layer),
            Some(&who),
            now_ms,
            &limits(),
        )
        .map_err(|r| ("refused".to_string(), r.to_string()))?;
        let id = applied.layer.unwrap_or_default();
        self.layers_touched.push(id.clone());
        Ok(id)
    }

    /// Apply a guide verb: what it did (past tense), or why not.
    pub fn apply_guide(
        &mut self,
        v: GuideVerb,
        fx: &mut Vec<crate::app::Effect>,
    ) -> Result<Option<String>, (String, String)> {
        let who = self.agent_name();
        let na = |m: String| ("not_applicable".to_string(), m);
        match v {
            GuideVerb::Hint {
                anchor,
                title,
                text,
                arrow,
                ring,
                spotlight,
                avoid,
            } => {
                let t = match &title {
                    Some(t) => format!("◆ {who} · {t}"),
                    None => format!("◆ {who}"),
                };
                let name = anchor_name(&anchor);
                let mut l = cl::Layer::new(anchor).with_content(cl::Content::hint(Some(&t), &text));
                l.arrow = arrow;
                if ring {
                    l.ring = Some(cl::Ring::default());
                }
                l.avoid = avoid;
                if spotlight {
                    l.spotlight = Some(cl::Spotlight::default());
                }
                let id = self.push_agent_layer(l)?;
                Ok(Some(format!("hint {id} on {name}")))
            }
            GuideVerb::Highlight(anchor) => {
                let name = anchor_name(&anchor);
                let id = self.push_agent_layer(cl::Layer::new(anchor).with_ring())?;
                Ok(Some(format!("highlight {id} on {name}")))
            }
            GuideVerb::Spotlight(anchors) => {
                let mut out = Vec::new();
                for a in anchors {
                    let name = anchor_name(&a);
                    let mut l = cl::Layer::new(a).with_ring();
                    l.spotlight = Some(cl::Spotlight {
                        holes: vec![cl::Part::Anchor],
                    });
                    let id = self.push_agent_layer(l)?;
                    out.push(format!("{id} on {name}"));
                }
                Ok(Some(format!("spotlight {}", out.join(", "))))
            }
            GuideVerb::Clear(id) => {
                let sel = match &id {
                    Some(i) => {
                        let Some(l) = self.ui.layers.get(i) else {
                            return Err((
                                "not_found".into(),
                                format!("No layer `{i}` (`ui state` lists them under `layers`)."),
                            ));
                        };
                        if !l.owner.is_agent() {
                            return Err(na(format!(
                                "Layer `{i}` is the walkthrough's: `walkthrough stop` leaves it."
                            )));
                        }
                        cl::Selector::Layer(i.clone())
                    }
                    None => cl::Selector::All(true),
                };
                // agents clear agents' layers only (a walkthrough's stay)
                let before: Vec<String> = self
                    .ui
                    .layers
                    .layers
                    .iter()
                    .filter(|l| !l.owner.is_agent())
                    .map(|l| l.id.clone())
                    .collect();
                let mut kept = self.ui.layers.clone();
                kept.layers.retain(|l| before.contains(&l.id));
                let mut ag = self.ui.layers.clone();
                ag.layers.retain(|l| l.owner.is_agent());
                let r = cl::apply(&mut ag, cl::LayerOp::Pop(sel), None, 0, &cl::Limits::none())
                    .map_err(|r| ("refused".to_string(), r.to_string()))?;
                kept.layers.extend(ag.layers);
                kept.next = ag.next.max(kept.next);
                self.ui.layers = kept;
                Ok((!r.popped.is_empty()).then(|| format!("cleared {}", r.popped.join(", "))))
            }
            GuideVerb::Narrate(Some((title, body))) => {
                if self.ui.walk.is_some() {
                    return Err(na(
                        "A walkthrough is playing: its steps narrate (`walkthrough stop` first)."
                            .into(),
                    ));
                }
                self.ui.narration = Some(crate::walk::Narration {
                    title: title.clone(),
                    body,
                    by: Some(who),
                    why: None,
                    scroll: 0,
                });
                Ok(Some(format!("narrating “{title}”")))
            }
            GuideVerb::Narrate(None) => {
                if self.ui.walk.is_some() {
                    return Err(na(
                        "A walkthrough is playing: `walkthrough stop` closes it.".into(),
                    ));
                }
                Ok(self
                    .ui
                    .narration
                    .take()
                    .map(|_| "closed the narration".into()))
            }
            GuideVerb::Play(w) => {
                let (id, n) = (w.id.clone(), w.steps.len());
                self.walk_start(*w, fx).map_err(na)?;
                Ok(Some(format!("playing walkthrough {id} ({n} steps)")))
            }
            GuideVerb::Step(_) | GuideVerb::Next | GuideVerb::Prev if self.ui.walk.is_none() => {
                Err(na(
                    "No walkthrough is playing (`walkthrough play <id>`).".into()
                ))
            }
            GuideVerb::Step(i) => {
                let n = self.ui.walk.as_ref().map_or(0, |w| w.len());
                if i >= n {
                    return Err(("usage".into(), format!("There are {n} steps.")));
                }
                self.walk_go(i, fx).map_err(na)?;
                Ok(Some(format!("step {} of {n}", i + 1)))
            }
            GuideVerb::Next | GuideVerb::Prev => {
                let (at, n) = self.ui.walk.as_ref().map_or((0, 0), |w| (w.at(), w.len()));
                let to = if matches!(v, GuideVerb::Next) {
                    (at + 1).min(n - 1)
                } else {
                    at.saturating_sub(1)
                };
                if to == at {
                    return Ok(None);
                }
                self.walk_go(to, fx).map_err(na)?;
                Ok(Some(format!("step {} of {n}", to + 1)))
            }
            GuideVerb::Stop => {
                if self.ui.walk.is_none() {
                    return Ok(None);
                }
                self.walk_stop();
                Ok(Some("left the walkthrough".into()))
            }
            GuideVerb::RecordStart { id, title } => {
                if self.ui.recording.is_some() {
                    return Err(na(
                        "Already recording (`walkthrough record stop` first).".into()
                    ));
                }
                let created = crate::clock::date_of(self.now);
                self.ui.recording = Some(crate::walk::Recording {
                    file: {
                        let mut w = ownpurse_core::walkthrough::new(&id, &title);
                        w.meta = ownpurse_core::walkthrough::Meta {
                            author: Some(who),
                            created: Some(created),
                            record: None,
                            audience: Some(
                                if self.ui.lay_mode { "lay" } else { "informed" }.into(),
                            ),
                        }
                        .to_value();
                        w
                    },
                });
                Ok(Some(format!("recording walkthrough {id}")))
            }
            GuideVerb::RecordStep => {
                let step = self.record_step().map_err(na)?;
                let n = {
                    let r = self
                        .ui
                        .recording
                        .as_mut()
                        .ok_or_else(|| na("Not recording.".into()))?;
                    r.file.steps.push(step);
                    r.file.steps.len()
                };
                Ok(Some(format!("recorded step {n}")))
            }
            GuideVerb::RecordStop => {
                let Some(r) = self.ui.recording.take() else {
                    return Err(na("Not recording.".into()));
                };
                if r.file.steps.is_empty() {
                    return Err(na(
                        "Nothing recorded: `walkthrough record step` after each page.".into(),
                    ));
                }
                let id = r.file.id.clone();
                let n = r.file.steps.len();
                fx.push(crate::app::Effect::SaveWalkthrough(Box::new(r.file)));
                Ok(Some(format!("saved walkthrough {id} ({n} steps)")))
            }
        }
    }
}
/// A rect trimmed to the text drawn in it (a row's cells to its last glyph), so a box can sit in the blank
/// space beside a table and an arrow can come in from there.
fn text_extent(buf: &Buffer, r: cl::Rect) -> Option<cl::Rect> {
    let mut first = None;
    let mut last = None;
    for y in r.y..r.bottom().min(buf.area.bottom()) {
        for x in r.x..r.right().min(buf.area.right()) {
            if !buf[(x, y)].symbol().trim().is_empty() {
                first = Some(first.map_or(x, |f: u16| f.min(x)));
                last = Some(last.map_or(x, |l: u16| l.max(x)));
            }
        }
    }
    Some(cl::Rect::new(first?, r.y, last? + 1 - first?, r.h))
}

/// What a step is about, kept clear when anything else fits: the diff's fields and lines, its title and via
/// band, the selected row (each trimmed to its text). A layer's own anchor is never avoided. Layers name
/// their own avoid anchors too (`Layer.avoid`: a walkthrough step's, a hint's `avoid=`).
fn referenced(app: &App, buf: &Buffer, anchors: &Anchors) -> Vec<cl::Rect> {
    let own: Vec<cl::Rect> = app
        .ui
        .layers
        .layers
        .iter()
        .flat_map(|l| l.anchor.iter())
        .filter_map(|a| anchors.get(&anchor_name(a)))
        .flat_map(|r| r.rects.clone())
        .collect();
    let mut out: Vec<cl::Rect> = Vec::new();
    for n in anchors.names() {
        if (n.starts_with("diff:") || n == "panel:via" || n == "panel:diff-title")
            && let Some(r) = anchors.get(n)
        {
            out.extend(r.rects.iter().filter_map(|x| text_extent(buf, *x)));
        }
    }
    if let Some(sel) = app.selected_anchor()
        && let Some(r) = anchors.get(&format!("row:{}", row_short(&sel)))
    {
        out.extend(r.rects.iter().filter_map(|x| text_extent(buf, *x)));
    }
    out.retain(|r| !own.iter().any(|o| o.intersects(r)));
    out
}

/// The screen for placement with the referenced content marked as soft avoid cells.
pub fn grid_avoiding(app: &App, buf: &Buffer, area: Rect, anchors: &Anchors) -> cl::Grid {
    let mut g = grid(buf, area, anchors);
    for r in referenced(app, buf, anchors) {
        g.avoid(r, cl::AVOID);
    }
    g
}

/// Plan once: the hard protects, plus the referenced content as soft avoid cells (caretline-layers keeps
/// boxes and arrows off them, and off text, whenever anything else fits; `Planned.covers_avoid` says when not).
pub fn plan_avoiding(app: &App, buf: &Buffer, area: Rect, anchors: &Anchors) -> cl::Plan {
    let g = grid_avoiding(app, buf, area, anchors);
    let r = renderers();
    // a table row is pointed at from beside it: the box right of the row's text, the head on the row
    let mut layers = app.ui.layers.clone();
    for l in &mut layers.layers {
        if l.place.is_empty() && on_row(l) {
            l.place = vec![
                cl::Side::Right,
                cl::Side::Left,
                cl::Side::Below,
                cl::Side::Above,
            ];
        }
    }
    let p = cl::plan(&layers, anchors.map(), &g, &r);
    // never a head in another row: a row layer whose arrow ends off its row is placed beside it only
    let off: Vec<String> = p
        .layers
        .iter()
        .filter(|pl| head_off_row(pl) && layers.get(&pl.id).is_some_and(on_row))
        .map(|pl| pl.id.clone())
        .collect();
    if off.is_empty() {
        return p;
    }
    for l in &mut layers.layers {
        if off.contains(&l.id) {
            l.place = vec![cl::Side::Right, cl::Side::Left];
        }
    }
    let q = cl::plan(&layers, anchors.map(), &g, &r);
    let placed = |pl: &cl::Plan| {
        off.iter().all(|id| {
            pl.layers
                .iter()
                .any(|l| &l.id == id && l.rect.is_some() && l.mode == Some(cl::Mode::Box))
        })
    };
    if placed(&q) { q } else { p }
}

/// A layer pointing at a table row (or a row range).
fn on_row(l: &cl::Layer) -> bool {
    matches!(l.anchor.first(), Some(cl::Anchor::Host { kind, .. }) if kind == "row")
}

/// Whether a placed layer's arrow head sits outside its anchor's rows.
pub fn head_off_row(pl: &cl::Planned) -> bool {
    let (Some(rt), Some(a)) = (pl.route.as_ref(), pl.anchor.as_ref()) else {
        return false;
    };
    let Some(head) = rt.steps.last() else {
        return false;
    };
    !a.rects.iter().any(|r| head.y >= r.y && head.y < r.bottom())
}

impl App {
    /// Every overlay anchor on the last frame, row and cell anchors included (derived here, on demand, when
    /// the frame had no layers to place: drawing records them only when it needs them).
    pub fn anchor_names(&self) -> Vec<String> {
        let mut h = self.hits.clone();
        if self.ui.layers.layers.is_empty() {
            record_rows(self, &mut h);
        }
        h.anchors.names().to_vec()
    }
}

/// Range anchors: `row:<k>=<a>..<b>` names every visible row from the row `k=a` to the row `k=b` (in
/// either order), as one rect (a spotlight's single hole). Recorded for the ranges the layers name.
fn record_ranges(app: &App, hits: &mut Hits) {
    let wanted: Vec<String> = app
        .ui
        .layers
        .layers
        .iter()
        .flat_map(|l| l.anchor.iter().chain(l.avoid.iter()))
        .filter_map(|a| match a {
            cl::Anchor::Host { kind, key } if kind == "row" && key.contains("..") => {
                Some(key.clone())
            }
            _ => None,
        })
        .collect();
    for key in wanted {
        let Some((lhs, hi)) = key.split_once("..") else {
            continue;
        };
        let Some((k, lo)) = lhs.split_once('=') else {
            continue;
        };
        let hi = hi.strip_prefix(&format!("{k}=")).unwrap_or(hi);
        let (Some(a), Some(b)) = (
            hits.anchors.rect(&format!("row:{k}={lo}")),
            hits.anchors.rect(&format!("row:{k}={hi}")),
        ) else {
            continue;
        };
        let (top, bottom) = (a.y.min(b.y), a.bottom().max(b.bottom()));
        // every row drawn in between widens the rect to its content
        let mut x0 = a.x.min(b.x);
        let mut x1 = a.right().max(b.right());
        for n in hits.anchors.names().to_vec() {
            if let (true, Some(r)) = (n.starts_with("row:"), hits.anchors.rect(&n))
                && r.y >= top
                && r.bottom() <= bottom
            {
                x0 = x0.min(r.x);
                x1 = x1.max(r.right());
            }
        }
        hits.anchors.put(
            &format!("row:{key}"),
            Rect::new(x0, top, x1.saturating_sub(x0), bottom - top),
        );
    }
}
