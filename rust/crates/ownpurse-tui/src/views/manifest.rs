//! Tab 7: Manifest (design spec manifests §2–§4; branches §4). On main: every manifest (main first). On a
//! manifest: the stage strip, the command panel (planned / approved: the exact CLI command with the full
//! hash, `y` copies) or the apply status line, then Entries · Issues · Diff · Log (`,` `.`), or an entry's
//! generic document diff (Enter). Prepared in `update` (`App::prepare_manifest`); draw only lays it out.
//! There is no apply action anywhere here: apply is CLI-only.

use crate::app::App;
use crate::hit::Hits;
use crate::layout::{ColSpec, Kind, fit_str, width};
use crate::manifest_tab::{Mode, Prompt};
use crate::theme::Theme;
use crate::theme_tokens::Token;
use crate::widgets::table::{self, Cell, Table};
use crate::widgets::{band, fill, put};
use ownpurse_core::manifest::stage::{EntryStatus, STAGES, Stage};
use ownpurse_core::money::fmt;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Modifier;
use ratatui::text::Span;
use rust_decimal::Decimal;

pub fn crumbs(app: &App) -> Vec<String> {
    let mut v = vec!["Manifest".to_string()];
    match app.ui.manifest.as_deref() {
        None => v.push("manifests".into()),
        Some(b) => {
            v.push(if app.ui.theme.ascii {
                format!("[{b}]")
            } else {
                format!("⎇ {b}")
            });
            v.push(app.mf_mode().label().to_string());
            if let Some(d) = app.mf_diff() {
                v.push(format!("entry {}", d.seq));
                v.push(crate::views::diff::crumb(&d));
            }
        }
    }
    v
}

pub fn status(app: &App) -> String {
    match app.manifest() {
        None => format!("{} manifests", app.mf_list().len().saturating_sub(1)),
        Some(m) => format!(
            "⎇ {} · {}",
            m.name,
            crate::layout::plural_changes(m.active().count())
        ),
    }
}

fn line(th: &Theme, buf: &mut Buffer, area: Rect, y: u16, spans: Vec<Span<'static>>) {
    if y < area.bottom() {
        let spans = spans
            .into_iter()
            .map(|s| Span::styled(th.text(&s.content), s.style))
            .collect();
        put(
            buf,
            area.x + 2,
            y,
            spans,
            area.width.saturating_sub(4),
            th.glyphs().ellipsis,
        );
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

/// The stage strip (spec §3): done `text.3` with ✓, the current bold `proposed` after `▸`, a failing stage
/// `✗ n` in `err`, the future in `text.4`; narrow widths collapse done to `✓` and the future to `·`.
pub fn strip_spans(th: &Theme, stage: &Stage, width_cells: usize) -> Vec<Span<'static>> {
    let cur = stage.index();
    let ascii = th.glyphs().arrow == "->";
    let tick = th.glyphs().ok;
    let cue = if ascii { ">" } else { "▸" };
    let sep = if ascii {
        " - ".to_string()
    } else {
        format!(" {} ", th.glyphs().crumb)
    };
    let current = |s: &Stage| -> (String, bool) {
        match s {
            Stage::CheckFailed { errors } => (format!("check ✗ {errors}"), true),
            Stage::Checked => (format!("check {tick}"), false),
            Stage::Stopped {
                at,
                landed,
                rejected,
                not_sent,
                verified,
            } => (
                format!(
                    "applying ✗ stopped at entry {at} · {landed} landed · {rejected} rejected · {not_sent} not sent · {verified} verified"
                ),
                true,
            ),
            Stage::Applying { done, total } => (format!("applying {done}/{total}"), false),
            Stage::Applied => (format!("applied {tick}"), false),
            other => (other.word().to_string(), false),
        }
    };
    let (cur_text, cur_err) = current(stage);
    let cur_style = if cur_err {
        th.fg(Token::Error).add_modifier(Modifier::BOLD)
    } else {
        th.fg(Token::Proposed).add_modifier(Modifier::BOLD)
    };
    // stopped: the counts are what matters, so the strip is only `▸ applying ✗ stopped …` (the done stages'
    // ticks would push "verified" off a 100-column screen)
    if matches!(stage, Stage::Stopped { .. }) {
        return vec![Span::styled(format!("{cue} {cur_text}"), cur_style)];
    }
    let full: usize = STAGES
        .iter()
        .enumerate()
        .map(|(i, w)| if i == cur { width(&cur_text) + 2 } else { width(w) + 2 } + 3)
        .sum();
    let narrow = full > width_cells;
    let mut out = vec![];
    for (i, w) in STAGES.iter().enumerate() {
        if i > 0 {
            out.push(Span::styled(
                if narrow { " ".to_string() } else { sep.clone() },
                th.fg(Token::Text4),
            ));
        }
        if i < cur {
            out.push(Span::styled(
                if narrow {
                    tick.to_string()
                } else {
                    format!("{w} {tick}")
                },
                th.fg(Token::Muted),
            ));
        } else if i == cur {
            out.push(Span::styled(format!("{cue} {cur_text}"), cur_style));
        } else {
            out.push(Span::styled(
                if narrow {
                    th.glyphs().sep.to_string()
                } else {
                    (*w).to_string()
                },
                th.fg(Token::Text4),
            ));
        }
    }
    out
}

/// The command panel (planned / approved; spec §4, review 8): the command(s) from the core's one builder,
/// one line each with the hash abbreviated (`27b8b424…7c`), never wrapped; `y` copies the FULL command.
/// Returns its height.
fn command_panel(app: &App, stage: &Stage, buf: &mut Buffer, area: Rect) -> u16 {
    let th = &app.ui.theme;
    let Some(c) = app.manifest_commands() else {
        return 0;
    };
    let mut lines: Vec<(String, String)> = vec![];
    match stage {
        Stage::Planned => {
            lines.push(("Approve: ".into(), c.approve.clone()));
            lines.push(("Then apply: ".into(), c.apply.clone()));
        }
        Stage::Approved => lines.push(("Apply: ".into(), c.apply.clone())),
        _ => return 0,
    }
    let raised = th.raised();
    let mut rows: Vec<Vec<Span<'static>>> = vec![vec![
        Span::styled(
            "Apply from your own terminal".to_string(),
            raised.patch(th.heading()),
        ),
        Span::styled(
            if matches!(stage, Stage::Planned) {
                "   the owner approves first (refused in agent sessions) · y copies the full command".to_string()
            } else {
                "   y copies the full command".to_string()
            },
            raised.patch(th.fg(Token::Muted)),
        ),
    ]];
    let hash = app
        .manifest()
        .and_then(|m| m.plan())
        .map(|p| p.hash.clone())
        .unwrap_or_default();
    for (label, cmd) in lines {
        let shown = th.text(&ownpurse_core::manifest::compile::abbreviate(&cmd, &hash));
        rows.push(vec![
            Span::styled(label, raised.patch(th.fg(Token::Muted))),
            Span::styled(shown, raised.patch(th.fg(Token::Text))),
        ]);
    }
    let h = rows.len() as u16;
    for (i, r) in rows.into_iter().enumerate() {
        let y = area.y + i as u16;
        if y >= area.bottom() {
            break;
        }
        fill(buf, Rect::new(area.x, y, area.width, 1), raised);
        let r: Vec<Span<'static>> = r
            .into_iter()
            .map(|s| Span::styled(th.text(&s.content), s.style))
            .collect();
        put(
            buf,
            area.x + 2,
            y,
            r,
            area.width.saturating_sub(3),
            th.glyphs().ellipsis,
        );
    }
    h
}

/// The mode strip on a manifest: `Entries   Issues   Diff   Log` with the active one in text.1.
fn modes(app: &App, buf: &mut Buffer, area: Rect) {
    let th = &app.ui.theme;
    let mut spans = vec![];
    for m in Mode::ALL {
        let on = app.mf_mode() == m;
        let mut label = m.label().to_string();
        if m == Mode::Issues
            && let Some(f) = app.manifest_facts()
        {
            let e = f.report.errors();
            let w = f.report.unacked();
            if e > 0 {
                label.push_str(&format!(" ✗{e}"));
            }
            if w > 0 {
                label.push_str(&format!(" !{w}"));
            }
        }
        spans.push(Span::styled(
            label,
            if on {
                th.heading()
            } else {
                th.fg(Token::Muted)
            },
        ));
        spans.push(Span::raw("   "));
    }
    spans.push(Span::styled(
        ", . mode · b manifests · P plan".to_string(),
        th.fg(Token::Text4),
    ));
    line(th, buf, area, area.y, spans);
}

pub fn draw(app: &App, buf: &mut Buffer, area: Rect, hits: &mut Hits) {
    let th = &app.ui.theme;
    if area.height < 4 {
        return;
    }
    let mut area = area;
    if let Some(p) = &app.ui.manifest_tab.prompt {
        let h = prompt(app, p, buf, area);
        area.y += h;
        area.height = area.height.saturating_sub(h);
    }
    if app.ui.manifest.is_none() || app.mf_mode() == Mode::Manifests {
        draw_list(app, buf, area, hits);
        return;
    }
    // the stage strip, then the command panel or the apply status line
    if let Some(f) = app.manifest_facts() {
        let spans = strip_spans(th, &f.status.stage, area.width.saturating_sub(4) as usize);
        line(th, buf, area, area.y, spans);
        // the stage strip and (below) the apply panel and status: protected from overlay boxes
        hits.anchors
            .chip("stage", Rect::new(area.x, area.y, area.width, 1));
        area.y += 1;
        area.height = area.height.saturating_sub(1);
        if f.status.plan_stale {
            line(
                th,
                buf,
                area,
                area.y,
                vec![Span::styled(
                    "the manifest changed after planning: plan again (P)".to_string(),
                    th.fg(Token::Warn),
                )],
            );
            area.y += 1;
            area.height = area.height.saturating_sub(1);
        }
        let h = if app.ui.manifest_tab.diff.is_some() {
            0
        } else {
            command_panel(app, &f.status.stage, buf, area)
        };
        if h > 0 {
            hits.anchors
                .panel("apply", Rect::new(area.x, area.y, area.width, h));
        }
        area.y += h;
        area.height = area.height.saturating_sub(h);
        if matches!(f.status.stage, Stage::Applying { .. } | Stage::Applied) {
            let a = &f.status.apply;
            hits.anchors
                .panel("apply", Rect::new(area.x, area.y, area.width, 1));
            line(
                th,
                buf,
                area,
                area.y,
                vec![Span::styled(
                    format!(
                        "applying · {} of {} · {} verified · {} landed · {} rejected · {} uncertain",
                        a.sent, a.total, a.verified, a.landed, a.rejected, a.uncertain
                    ),
                    th.fg(Token::Muted),
                )],
            );
            area.y += 1;
            area.height = area.height.saturating_sub(1);
        }
    }
    if let (Some(doc), Some(ui)) = (app.mf_diff(), app.ui.manifest_tab.diff.as_ref()) {
        crate::views::diff::draw(app, &doc, ui, buf, area, hits);
        return;
    }
    modes(app, buf, area);
    area.y += 2;
    area.height = area.height.saturating_sub(2);
    match app.mf_mode() {
        Mode::Entries => draw_entries(app, buf, area, hits),
        Mode::Issues => {
            hits.anchors.panel("issues", area);
            draw_issues(app, buf, area, hits)
        }
        Mode::Diff => draw_effects(app, buf, area),
        Mode::Log => draw_log(app, buf, area, hits),
        Mode::Manifests => {}
    }
}

/// The open prompt (one or two lines on the raised surface). Returns the rows it took.
fn prompt(app: &App, p: &Prompt, buf: &mut Buffer, area: Rect) -> u16 {
    let th = &app.ui.theme;
    let raised = th.raised();
    if let Prompt::Plan { focus } = p {
        let n = app.manifest().map_or(0, |m| m.active().count());
        fill(buf, Rect::new(area.x, area.y, area.width, 1), raised);
        let text = format!(
            "Plan {}: seal {} entr{} into a write package (local; nothing is sent to Xero)  ",
            app.ui.manifest.as_deref().unwrap_or(""),
            n,
            if n == 1 { "y" } else { "ies" }
        );
        let mut spans = vec![Span::styled(text, raised.patch(th.fg(Token::Text)))];
        for (i, b) in ["Cancel", "Plan"].iter().enumerate() {
            let st = if i as u8 == *focus {
                th.focused_button()
            } else {
                raised.patch(th.fg(Token::Muted))
            };
            spans.push(Span::styled(format!("[ {b} ]"), st));
            spans.push(Span::styled(" ".to_string(), raised));
        }
        put(
            buf,
            area.x + 2,
            area.y,
            spans,
            area.width.saturating_sub(3),
            th.glyphs().ellipsis,
        );
        return 2;
    }
    let rows: Vec<(String, String, bool)> = match p {
        Prompt::New { name, why, field } => vec![
            ("New manifest name".into(), name.clone(), *field == 0),
            ("why".into(), why.clone(), *field == 1),
        ],
        Prompt::Note { seq, text } => vec![(
            format!("Note entry {seq} (accountant: <ref> or a note)"),
            text.clone(),
            true,
        )],
        Prompt::Edit { seq, text } => vec![(
            format!("Edit entry {seq} (CODE AMOUNT, … | narration)"),
            text.clone(),
            true,
        )],
        Prompt::Plan { .. } => vec![],
    };
    for (i, (label, text, focus)) in rows.iter().enumerate() {
        let y = area.y + i as u16;
        fill(buf, Rect::new(area.x, y, area.width, 1), raised);
        let head = format!("{label} › ");
        let x = area.x + 2;
        buf.set_span(
            x,
            y,
            &Span::styled(head.clone(), raised.patch(th.fg(Token::Muted))),
            area.width,
        );
        let x = x + width(&head) as u16;
        let w = area.right().saturating_sub(x + 2);
        let t = fit_str(text, w as usize, th.glyphs().ellipsis);
        buf.set_span(
            x,
            y,
            &Span::styled(t.clone(), raised.patch(th.fg(Token::Text))),
            w,
        );
        if *focus {
            crate::widgets::caret(
                buf,
                x,
                y,
                &t,
                app.ui.caret,
                raised.patch(th.fg(Token::Text)),
                w,
            );
        }
    }
    rows.len() as u16 + 1
}

const LIST_COLS: [ColSpec; 6] = [
    ColSpec {
        title: "manifest",
        short_title: "",
        kind: Kind::Text { min: 8 },
        priority: 9,
    },
    ColSpec {
        title: "stage",
        short_title: "",
        kind: Kind::Status,
        priority: 9,
    },
    ColSpec {
        title: "entries",
        short_title: "n",
        kind: Kind::Amount,
        priority: 8,
    },
    ColSpec {
        title: "base",
        short_title: "",
        kind: Kind::Status,
        priority: 3,
    },
    ColSpec {
        title: "created by",
        short_title: "by",
        kind: Kind::Text { min: 5 },
        priority: 5,
    },
    ColSpec {
        title: "why",
        short_title: "",
        kind: Kind::Text { min: 6 },
        priority: 4,
    },
];

fn draw_list(app: &App, buf: &mut Buffer, area: Rect, hits: &mut Hits) {
    let th = &app.ui.theme;
    let list = app.mf_list();
    let open = list.iter().flatten().filter(|m| !m.discarded).count();
    band(
        th,
        buf,
        Rect::new(area.x, area.y, area.width, 1),
        "Manifests",
        Some((format!("{open} open · N new · Enter switch"), Token::Muted)),
    );
    let cur = app.ui.manifest.clone();
    let shown_stage = app
        .manifest_facts()
        .map(|f| f.status.stage.word().to_string());
    let rows: Vec<Vec<Cell>> = list
        .iter()
        .map(|m| match m {
            None => vec![
                Cell::new(
                    if cur.is_none() {
                        "main (shown)"
                    } else {
                        "main"
                    },
                    Token::Text,
                ),
                Cell::new("synced from Xero", Token::Muted),
                Cell::new("—", Token::Muted),
                Cell::new("the record", Token::Muted),
                Cell::new("", Token::Muted),
                Cell::new("", Token::Muted),
            ],
            Some(m) => {
                let shown = cur.as_deref() == Some(m.name.as_str());
                let tok = if m.discarded {
                    Token::Text4
                } else {
                    Token::Text
                };
                let stage = if m.discarded {
                    "discarded".to_string()
                } else if shown {
                    shown_stage.clone().unwrap_or_default()
                } else if m.plan().is_some() {
                    "planned".into()
                } else {
                    "draft".into()
                };
                vec![
                    Cell::new(
                        format!("⎇ {}{}", m.name, if shown { " (shown)" } else { "" }),
                        if m.discarded {
                            Token::Text4
                        } else {
                            Token::Proposed
                        },
                    ),
                    Cell::new(stage, tok),
                    Cell::new(m.active().count().to_string(), tok),
                    Cell::new(format!("v{}", m.base), Token::Muted),
                    Cell::new(m.created_by.clone(), Token::Muted),
                    Cell::new(m.why.clone(), Token::Muted),
                ]
            }
        })
        .collect();
    let t_area = Rect::new(
        area.x,
        area.y + 1,
        area.width,
        area.height.saturating_sub(1),
    );
    hits.content = Some(t_area);
    table::draw(
        th,
        buf,
        t_area,
        &Table {
            cols: &LIST_COLS,
            rows: &rows,
            steps: &[],
            top: 0,
            cursor: app.mf_cursor(),
            cursor_col: None,
            footer: None,
            hidden: &[],
            focus: None,
            groups: &[],
        },
        hits,
    );
}

const ENTRY_COLS: [ColSpec; 9] = [
    // give up width in this order (review 4): change, by, issues, entity; never org or target before those
    ColSpec {
        title: "seq",
        short_title: "#",
        kind: Kind::Status,
        priority: 9,
    },
    ColSpec {
        title: " ",
        short_title: "",
        kind: Kind::Status,
        priority: 9,
    },
    ColSpec {
        title: "org",
        short_title: "",
        kind: Kind::Text { min: 6 },
        priority: 9,
    },
    ColSpec {
        title: "entity",
        short_title: "",
        kind: Kind::Status,
        priority: 6,
    },
    ColSpec {
        title: "target",
        short_title: "",
        kind: Kind::Text { min: 10 },
        priority: 9,
    },
    ColSpec {
        title: "change",
        short_title: "",
        kind: Kind::Text { min: 6 },
        priority: 3,
    },
    ColSpec {
        title: "issues",
        short_title: "",
        kind: Kind::Status,
        priority: 5,
    },
    ColSpec {
        title: "by",
        short_title: "",
        kind: Kind::Status,
        priority: 4,
    },
    ColSpec {
        title: "apply",
        short_title: "",
        kind: Kind::Text { min: 10 },
        priority: 8,
    },
];

/// The apply status cell (spec §4).
pub fn status_cell(s: &EntryStatus) -> Cell {
    match s {
        EntryStatus::None => Cell::new("", Token::Muted),
        EntryStatus::Queued => Cell::new("·", Token::Text4),
        EntryStatus::Sending => Cell::new("⇅ sending", Token::Accent),
        EntryStatus::Landed(t) => Cell::new(
            format!(
                "✓ landed{} · verifying",
                t.as_ref()
                    .map(|t| format!(" · MJ {}…", t.chars().take(4).collect::<String>()))
                    .unwrap_or_default()
            ),
            Token::Ok,
        ),
        EntryStatus::Posted => Cell::new("✓ posted", Token::Muted),
        EntryStatus::Rejected(m) => Cell::new(format!("✗ rejected: {m}"), Token::Error),
        EntryStatus::Uncertain => Cell::new("? uncertain: no response · do not retry", Token::Warn),
        EntryStatus::Manual => Cell::new("browser step · checklist", Token::Muted),
    }
}

fn draw_entries(app: &App, buf: &mut Buffer, area: Rect, hits: &mut Hits) {
    let th = &app.ui.theme;
    let rows = app.mf_rows();
    let note = match (app.manifest().and_then(|m| m.plan()), app.manifest_facts()) {
        (Some(p), _) => format!("planned · sha256 {}…", &p.hash[..12.min(p.hash.len())]),
        (None, Some(f)) => f.report.summary(),
        _ => String::new(),
    };
    band(
        th,
        buf,
        Rect::new(area.x, area.y, area.width, 1),
        "Entries",
        Some((note, Token::Muted)),
    );
    if rows.is_empty() {
        line(
            th,
            buf,
            area,
            area.y + 1,
            vec![Span::styled(
                "No entries yet. Add one with `ownpurse manifest journal | recode | void | attach | add`, or through an agent."
                    .to_string(),
                th.fg(Token::Muted),
            )],
        );
        return;
    }
    let applying = app.manifest_facts().is_some_and(|f| {
        matches!(
            f.status.stage,
            Stage::Applying { .. } | Stage::Stopped { .. } | Stage::Applied | Stage::Approved
        )
    });
    let cells: Vec<Vec<Cell>> = rows
        .iter()
        .map(|r| {
            let live = r.entry.active();
            let tok = if live { Token::Text } else { Token::Text4 };
            let agent = r.entry.by_agent();
            let marker = format!(
                "{}{}",
                if agent && !r.marker.is_empty() {
                    "A"
                } else {
                    ""
                },
                r.marker
            );
            let marker_tok = if r.conflict {
                Token::Error
            } else if live && !r.marker.is_empty() {
                Token::Proposed
            } else {
                Token::Text4
            };
            let badge_tok = if r.badge.contains('✗') {
                Token::Error
            } else if r.badge.contains('!') {
                Token::Warn
            } else {
                Token::Muted
            };
            let target = if r.entry.is_new() {
                format!(
                    "new {}",
                    ownpurse_core::manifest::fields::entity_code(&r.entry.entity)
                )
            } else {
                r.entry.id.clone()
            };
            let word = r.word();
            vec![
                Cell::new(r.entry.seq.to_string(), Token::Muted),
                Cell::new(marker, marker_tok),
                Cell::new(r.entry.org.clone(), tok),
                Cell::new(
                    ownpurse_core::manifest::fields::entity_code(&r.entry.entity),
                    Token::Muted,
                ),
                Cell::new(target, tok),
                Cell::new(
                    if word.is_empty() {
                        r.changes.clone()
                    } else {
                        word
                    },
                    tok,
                ),
                Cell::new(r.badge.clone(), badge_tok),
                Cell::new(
                    if agent { "A" } else { "H" },
                    if agent { Token::Info } else { Token::Muted },
                ),
                if applying {
                    status_cell(&r.status)
                } else {
                    Cell::new("", Token::Muted)
                },
            ]
        })
        .collect();
    let hidden: &[usize] = if applying { &[] } else { &[8] };
    let t_area = Rect::new(
        area.x,
        area.y + 1,
        area.width,
        area.height.saturating_sub(1),
    );
    hits.content = Some(t_area);
    table::draw(
        th,
        buf,
        t_area,
        &Table {
            cols: &ENTRY_COLS,
            rows: &cells,
            steps: &[],
            top: app.ui.manifest_tab.top,
            cursor: app.mf_cursor(),
            cursor_col: None,
            footer: None,
            hidden,
            focus: app.mf_selected().and_then(|r| {
                (!r.entry.why.is_empty() || r.entry.refs.accountant_decision.is_some()).then(|| {
                    let mut s = r.entry.why.clone();
                    if let Some(a) = &r.entry.refs.accountant_decision {
                        s.push_str(&format!(
                            "{}⚑ accountant: {a}",
                            if s.is_empty() { "" } else { " · " }
                        ));
                    }
                    s
                })
            }),
            groups: &[],
        },
        hits,
    );
}

const ISSUE_COLS: [ColSpec; 6] = [
    ColSpec {
        title: " ",
        short_title: "",
        kind: Kind::Status,
        priority: 9,
    },
    ColSpec {
        title: "entry",
        short_title: "#",
        kind: Kind::Status,
        priority: 9,
    },
    ColSpec {
        title: "org",
        short_title: "",
        kind: Kind::Text { min: 6 },
        priority: 6,
    },
    ColSpec {
        title: "field",
        short_title: "",
        kind: Kind::Text { min: 16 },
        priority: 8,
    },
    ColSpec {
        title: "issue",
        short_title: "",
        kind: Kind::Text { min: 30 },
        priority: 9,
    },
    ColSpec {
        title: "hint",
        short_title: "",
        kind: Kind::Text { min: 18 },
        priority: 4,
    },
];

fn draw_issues(app: &App, buf: &mut Buffer, area: Rect, hits: &mut Hits) {
    let th = &app.ui.theme;
    let issues = app.mf_issues();
    let f = app.manifest_facts();
    let summary = f.as_ref().map(|f| f.report.summary()).unwrap_or_default();
    let err = f.as_ref().is_some_and(|f| f.report.errors() > 0);
    line(
        th,
        buf,
        area,
        area.y,
        vec![Span::styled(
            summary,
            if err {
                th.fg(Token::Error)
            } else {
                th.fg(Token::Muted)
            },
        )],
    );
    if issues.is_empty() {
        return;
    }
    let first_warn = issues.iter().position(|i| !i.blocking());
    let mut groups: Vec<(usize, String, String)> = vec![];
    let ne = issues.iter().filter(|i| i.blocking()).count();
    if ne > 0 {
        groups.push((0, "Errors · block planning".into(), ne.to_string()));
    }
    if let Some(w) = first_warn {
        groups.push((
            w,
            "Warnings · acknowledge in review".into(),
            (issues.len() - w).to_string(),
        ));
    }
    let cells: Vec<Vec<Cell>> = issues
        .iter()
        .map(|i| {
            let (g, tok) = if i.blocking() {
                ("✗", Token::Error)
            } else if i.ack.is_some() {
                ("✓", Token::Muted)
            } else {
                ("!", Token::Warn)
            };
            let hint = match (&i.ack, i.blocking()) {
                (Some(a), false) => format!(
                    "acknowledged by {} {}",
                    a.actor.trim_start_matches("agent:"),
                    a.at.get(11..16).unwrap_or("")
                ),
                (None, false) => format!("a acknowledge · {}", i.hint),
                _ => i.hint.clone(),
            };
            vec![
                Cell::new(g, tok),
                Cell::new(
                    if i.seq > 0 {
                        i.seq.to_string()
                    } else {
                        "—".into()
                    },
                    Token::Muted,
                ),
                Cell::new(i.org.clone(), Token::Muted),
                Cell::new(i.field.clone(), Token::Text2),
                Cell::new(
                    i.message.clone(),
                    if i.ack.is_some() {
                        Token::Muted
                    } else {
                        Token::Text
                    },
                ),
                Cell::new(hint, Token::Muted),
            ]
        })
        .collect();
    // the selected issue in full under the list (review 3): the message and the hint, wrapped
    let detail_h: u16 = if area.height >= 12 { 4 } else { 0 };
    let t_area = Rect::new(
        area.x,
        area.y + 1,
        area.width,
        area.height.saturating_sub(1 + detail_h),
    );
    hits.content = Some(t_area);
    table::draw(
        th,
        buf,
        t_area,
        &Table {
            cols: &ISSUE_COLS,
            rows: &cells,
            steps: &[],
            top: app.ui.manifest_tab.top,
            cursor: app.mf_cursor(),
            cursor_col: None,
            footer: None,
            hidden: &[],
            focus: None,
            groups: &groups,
        },
        hits,
    );
    if detail_h > 0
        && let Some(i) = app.mf_cursor().and_then(|c| issues.get(c))
    {
        let mut y = t_area.bottom();
        band(
            th,
            buf,
            Rect::new(area.x, y, area.width, 1),
            &format!("{} · entry {} · {}", i.severity.word(), i.seq, i.field),
            Some(("Enter goes to the field".into(), Token::Muted)),
        );
        y += 1;
        let text = format!("{} · {}", i.message, i.hint);
        for l in wrap_words(&text, (area.width as usize).saturating_sub(4))
            .into_iter()
            .take(3)
        {
            line(th, buf, area, y, vec![Span::styled(l, th.fg(Token::Text2))]);
            y += 1;
        }
    }
}

fn draw_effects(app: &App, buf: &mut Buffer, area: Rect) {
    let th = &app.ui.theme;
    let Some(f) = app.manifest_facts() else {
        crate::views::placeholder(app, buf, area, "Computing the effects…");
        return;
    };
    let d = match &f.diff {
        Ok(d) => d,
        Err(e) => {
            crate::views::placeholder(
                app,
                buf,
                area,
                &format!("Could not compute the effects: {e}"),
            );
            return;
        }
    };
    let mut lines: Vec<(Option<&'static str>, Vec<Span<'static>>)> = vec![];
    let mut top = vec![Span::styled(
        crate::layout::plural_changes(d.entries),
        th.fg(Token::Text),
    )];
    if d.conflicts > 0 {
        top.push(Span::styled(
            format!(
                " · {} conflict{}",
                d.conflicts,
                if d.conflicts == 1 { "" } else { "s" }
            ),
            th.fg(Token::Error),
        ));
    }
    top.push(Span::styled(
        format!(" · at {}", d.as_of),
        th.fg(Token::Muted),
    ));
    lines.push((None, top));
    lines.push((None, vec![]));
    let show_all = app.ui.details;
    lines.push((Some("Trial balance"), vec![]));
    let mut any = false;
    for o in &d.orgs {
        for a in &o.accounts {
            any = true;
            lines.push((
                None,
                vec![
                    Span::styled(format!("{:<7}", o.org), th.fg(Token::Muted)),
                    Span::styled(format!("{:<6}", a.code), th.fg(Token::Muted)),
                    Span::styled(
                        format!("{:<34}", fit_str(&a.name, 33, "…")),
                        th.fg(Token::Text),
                    ),
                    Span::styled(format!("{:>14}", fmt(a.main)), th.fg(Token::Text2)),
                    Span::styled(" → ".to_string(), th.fg(Token::Muted)),
                    Span::styled(format!("{:>14}", fmt(a.manifest)), th.fg(Token::Text)),
                    Span::styled(format!("  Δ {}", signed(a.delta())), th.fg(Token::Proposed)),
                ],
            ));
        }
    }
    if !any {
        lines.push((
            None,
            vec![Span::styled(
                "No account moves.".to_string(),
                th.fg(Token::Muted),
            )],
        ));
    }
    lines.push((None, vec![]));
    lines.push((Some("Balance sheet and P&L"), vec![]));
    for o in &d.orgs {
        for (label, m, b) in &o.totals {
            if m == b && !show_all {
                continue;
            }
            lines.push((
                None,
                vec![
                    Span::styled(format!("{:<7}", o.org), th.fg(Token::Muted)),
                    Span::styled(format!("{label:<40}"), th.fg(Token::Text)),
                    Span::styled(format!("{:>14}", fmt(*m)), th.fg(Token::Text2)),
                    Span::styled(" → ".to_string(), th.fg(Token::Muted)),
                    Span::styled(format!("{:>14}", fmt(*b)), th.fg(Token::Text)),
                    Span::styled(format!("  Δ {}", signed(*b - *m)), th.fg(Token::Proposed)),
                ],
            ));
        }
    }
    let bridges: Vec<_> = d
        .bridges
        .iter()
        .filter(|b| show_all || b.main != b.manifest)
        .collect();
    if !bridges.is_empty() {
        lines.push((None, vec![]));
        lines.push((Some("Intercompany bridges"), vec![]));
        for b in bridges {
            let z = b.manifest.is_some_and(|v| v.is_zero());
            lines.push((
                None,
                vec![
                    Span::styled(
                        format!("{:<47}", fit_str(&b.label, 46, "…")),
                        th.fg(Token::Text),
                    ),
                    Span::styled(
                        format!("{:>14}", b.main.map(fmt).unwrap_or_else(|| "—".into())),
                        th.fg(Token::Text2),
                    ),
                    Span::styled(" → ".to_string(), th.fg(Token::Muted)),
                    Span::styled(
                        format!("{:>14}", b.manifest.map(fmt).unwrap_or_else(|| "—".into())),
                        th.fg(Token::Text),
                    ),
                    Span::styled(
                        if z {
                            format!("  {}", th.glyphs().ok)
                        } else {
                            String::new()
                        },
                        th.fg(Token::Muted),
                    ),
                ],
            ));
        }
    }
    let zero: Vec<_> = d
        .orgs
        .iter()
        .flat_map(|o| o.zero.iter().map(move |a| (o.org.clone(), a)))
        .collect();
    if !zero.is_empty() {
        lines.push((None, vec![]));
        lines.push((Some("Suspense and must-be-zero accounts"), vec![]));
        for (org, a) in zero {
            let z = a.manifest.is_zero();
            lines.push((
                None,
                vec![
                    Span::styled(format!("{org:<7}"), th.fg(Token::Muted)),
                    Span::styled(format!("{:<6}", a.code), th.fg(Token::Muted)),
                    Span::styled(
                        format!("{:<34}", fit_str(&a.name, 33, "…")),
                        th.fg(Token::Text),
                    ),
                    Span::styled(format!("{:>14}", fmt(a.main)), th.fg(Token::Text2)),
                    Span::styled(" → ".to_string(), th.fg(Token::Muted)),
                    Span::styled(format!("{:>14}", fmt(a.manifest)), th.fg(Token::Text)),
                    Span::styled(
                        if z {
                            format!("  {}", th.glyphs().ok)
                        } else {
                            String::new()
                        },
                        th.fg(Token::Muted),
                    ),
                ],
            ));
        }
    }
    let fc = &d.facts;
    lines.push((None, vec![]));
    lines.push((Some("What a plan compiles to"), vec![]));
    lines.push((
        None,
        vec![Span::styled(
            format!(
                "{} journal{} (reversed only by a reversing journal) · {} void{} · {} recode{} ({} on reconciled lines) · {} attachment{} · {} not writable",
                fc.journals,
                if fc.journals == 1 { "" } else { "s" },
                fc.voids,
                if fc.voids == 1 { "" } else { "s" },
                fc.recodes,
                if fc.recodes == 1 { "" } else { "s" },
                fc.reconciled_line_edits,
                fc.attachments,
                if fc.attachments == 1 { "" } else { "s" },
                fc.not_writable,
            ),
            th.fg(Token::Text2),
        )],
    ));
    for (y, (b, spans)) in (area.y..area.bottom()).zip(lines) {
        match b {
            Some(label) => band(th, buf, Rect::new(area.x, y, area.width, 1), label, None),
            None => line(th, buf, area, y, spans),
        }
    }
}

const LOG_COLS: [ColSpec; 5] = [
    ColSpec {
        title: "seq",
        short_title: "",
        kind: Kind::Status,
        priority: 9,
    },
    ColSpec {
        title: "at",
        short_title: "",
        kind: Kind::Status,
        priority: 6,
    },
    ColSpec {
        title: "actor",
        short_title: "",
        kind: Kind::Text { min: 5 },
        priority: 7,
    },
    ColSpec {
        title: "kind",
        short_title: "",
        kind: Kind::Status,
        priority: 8,
    },
    ColSpec {
        title: "what",
        short_title: "",
        kind: Kind::Text { min: 12 },
        priority: 9,
    },
];

fn draw_log(app: &App, buf: &mut Buffer, area: Rect, hits: &mut Hits) {
    let th = &app.ui.theme;
    band(
        th,
        buf,
        Rect::new(area.x, area.y, area.width, 1),
        "Log",
        Some(("read-only".into(), Token::Muted)),
    );
    let rows: Vec<Vec<Cell>> = app
        .mf_log()
        .into_iter()
        .map(|(seq, at, actor, kind, what)| {
            vec![
                Cell::new(seq.to_string(), Token::Muted),
                Cell::new(at, Token::Muted),
                Cell::new(actor, Token::Text2),
                Cell::new(kind, Token::Muted),
                Cell::new(what, Token::Text),
            ]
        })
        .collect();
    let t_area = Rect::new(
        area.x,
        area.y + 1,
        area.width,
        area.height.saturating_sub(1),
    );
    hits.content = Some(t_area);
    table::draw(
        th,
        buf,
        t_area,
        &Table {
            cols: &LOG_COLS,
            rows: &rows,
            steps: &[],
            top: app.ui.manifest_tab.top,
            cursor: app.mf_cursor(),
            cursor_col: None,
            footer: None,
            hidden: &[],
            focus: None,
            groups: &[],
        },
        hits,
    );
}

/// Wrap at spaces (a word longer than the width is cut), so a command stays readable and selectable.
fn wrap_words(text: &str, w: usize) -> Vec<String> {
    let mut out: Vec<String> = vec![String::new()];
    for word in text.split(' ') {
        let cur = out.last_mut().expect("one line");
        let need = width(cur) + usize::from(!cur.is_empty()) + width(word);
        if need > w && !cur.is_empty() {
            out.push(String::new());
        }
        let cur = out.last_mut().expect("one line");
        if !cur.is_empty() {
            cur.push(' ');
        }
        cur.push_str(word);
    }
    out
}
