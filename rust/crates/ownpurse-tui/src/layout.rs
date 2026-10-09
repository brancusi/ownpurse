//! Cell-width text (spec §4.1), the header fit (§3.1, V5) and table column fitting (§4.3).
//! The grapheme cluster is the unit; `fit` is the only function that truncates.

use crate::model::Level;
use ratatui::text::Span;
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

/// Display width in cells, summed per grapheme cluster (what ratatui draws).
pub fn width(s: &str) -> usize {
    s.graphemes(true).map(UnicodeWidthStr::width).sum()
}

pub fn spans_width(spans: &[Span<'_>]) -> usize {
    spans.iter().map(|s| width(&s.content)).sum()
}

/// Truncate styled spans to `max` cells, ending with `ellipsis` (counted at its own width) when cut.
/// Never splits a grapheme cluster; a wide cluster that would straddle the edge is dropped.
pub fn fit(spans: Vec<Span<'static>>, max: usize, ellipsis: &str) -> Vec<Span<'static>> {
    if spans_width(&spans) <= max {
        return spans;
    }
    let ell_w = width(ellipsis);
    if max < ell_w {
        return Vec::new();
    }
    let budget = max - ell_w;
    let mut out = Vec::new();
    let mut used = 0;
    let mut last_style = spans.first().map(|s| s.style).unwrap_or_default();
    'outer: for span in spans {
        let mut kept = String::new();
        for g in span.content.graphemes(true) {
            let w = UnicodeWidthStr::width(g);
            if used + w > budget {
                if !kept.is_empty() {
                    out.push(Span::styled(kept, span.style));
                }
                last_style = span.style;
                break 'outer;
            }
            used += w;
            kept.push_str(g);
        }
        last_style = span.style;
        if !kept.is_empty() {
            out.push(Span::styled(kept, span.style));
        }
    }
    out.push(Span::styled(ellipsis.to_string(), last_style));
    out
}

/// `fit` for plain text.
pub fn fit_str(s: &str, max: usize, ellipsis: &str) -> String {
    fit(vec![Span::raw(s.to_string())], max, ellipsis)
        .into_iter()
        .map(|s| s.content.into_owned())
        .collect()
}

/// Pad (or `fit`) plain text to exactly `w` cells.
pub fn pad(s: &str, w: usize, ellipsis: &str, right: bool) -> String {
    let t = fit_str(s, w, ellipsis);
    let gap = " ".repeat(w.saturating_sub(width(&t)));
    if right { gap + &t } else { t + &gap }
}

/// Word-wrap to `max` cells per line (refusal texts and strips wrap; they're never cut). A word longer
/// than a line is broken at grapheme boundaries.
pub fn wrap(s: &str, max: usize) -> Vec<String> {
    let max = max.max(1);
    let mut lines = Vec::new();
    let mut line = String::new();
    for word in s.split(' ') {
        let candidate = if line.is_empty() {
            word.to_string()
        } else {
            format!("{line} {word}")
        };
        if width(&candidate) <= max {
            line = candidate;
            continue;
        }
        if !line.is_empty() {
            lines.push(std::mem::take(&mut line));
        }
        let mut rest = String::new();
        for g in word.graphemes(true) {
            if width(&rest) + width(g) > max {
                lines.push(std::mem::take(&mut rest));
            }
            rest.push_str(g);
        }
        line = rest;
    }
    if !line.is_empty() || lines.is_empty() {
        lines.push(line);
    }
    lines
}

// ── table columns (§4.3) ──────────────────────────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    /// Right-aligned money or counts: never truncated, never hidden (V8).
    Amount,
    /// Never truncated, never hidden.
    Status,
    /// Fixed width; hides early.
    Id,
    /// Shortens with `…` down to `min`, then hides.
    Text { min: usize },
}

#[derive(Clone, Copy, Debug)]
pub struct ColSpec {
    pub title: &'static str,
    /// The title once the column shows its short form (`year` for `parity (year)`); "" = same title.
    pub short_title: &'static str,
    pub kind: Kind,
    /// Higher = more important. Lowest priority shortens first, then hides first.
    pub priority: u8,
}

/// Widths for each column (`None` = hidden), given the natural widths (widest value or title) and the
/// space available, with `gap` cells between visible columns. Amount and status columns always keep
/// their natural width.
pub fn fit_columns(
    cols: &[ColSpec],
    natural: &[usize],
    avail: usize,
    gap: usize,
) -> Vec<Option<usize>> {
    let mut w: Vec<Option<usize>> = natural.iter().map(|&n| Some(n)).collect();
    let total = |w: &[Option<usize>]| {
        let vis: Vec<usize> = w.iter().flatten().copied().collect();
        vis.iter().sum::<usize>() + gap * vis.len().saturating_sub(1)
    };
    let mut order: Vec<usize> = (0..cols.len()).collect();
    order.sort_by_key(|&i| (cols[i].priority, std::cmp::Reverse(i)));
    // 1. text columns shorten to their min, lowest priority first
    for &i in &order {
        let over = total(&w).saturating_sub(avail);
        if over == 0 {
            return w;
        }
        if let (Kind::Text { min }, Some(cur)) = (cols[i].kind, w[i]) {
            let floor = min.min(cur);
            w[i] = Some(cur - over.min(cur - floor));
        }
    }
    // 2. text and id columns hide, lowest priority first
    for &i in &order {
        if total(&w) <= avail {
            return w;
        }
        if matches!(cols[i].kind, Kind::Text { .. } | Kind::Id) {
            w[i] = None;
        }
    }
    w
}

// ── header (§3.1, V5) ─────────────────────────────────────────────────────────────────────────────

/// Everything row 0 says, in its full, compact and minimal forms.
#[derive(Clone, Debug)]
pub struct HeaderFacts {
    /// `ownpurse · <profile> · v<ver>`
    pub product: String,
    pub offline: bool,
    pub scope_full: String,
    pub scope_compact: String,
    /// The last resort (via another org's books: `via hold-roast ▾`); empty = the compact form.
    pub scope_min: String,
    pub as_of: String,
    pub fresh_full: String,
    pub fresh_compact: String,
    pub fresh_min: String,
    pub fresh_level: Level,
    pub left: i64,
    pub limit: i64,
    pub quota_level: Level,
    /// The figure is the local estimate (no `X-DayLimit-Remaining` today): `≈ 1,000 left (estimated)`.
    pub left_estimated: bool,
    /// A run covers exactly this scope (or is a login): `⇅` before the calls-left figure (spec P6 §4).
    pub busy: bool,
    /// A sync of another org runs: its own segment `⇅ design · 967/1,000 left` after the scope's figure.
    pub run: Option<RunSeg>,
    /// The agent bridge: None, Some(false) connected (`◇ agent`, dim), Some(true) acted
    /// (`◆ agent`, accent). Just before calls-left.
    pub agent: Option<bool>,
    /// NO_COLOR: the agent mark uses the ASCII words (`[agent]` / `[agent*]`), dim and accent being gone.
    pub plain: bool,
    /// Rendered from a state exported against another record version or config: the chip after the badges
    /// (`≠ record changed`, `≠ config changed`, both: `≠ record and config changed`).
    pub drift: Option<&'static str>,
    /// The Classify tab is open: `classify: 214 open · 3 stale` (open, stale); dropped from step 4.
    pub classify: Option<(usize, usize)>,
    /// The record age shows (`tui.view.record_age_in_header`: always, or only when stale).
    pub show_age: bool,
    /// The calls left show (`tui.view.budget_in_header`: always, low (under the warning line) or offline, never).
    pub show_quota: bool,
    /// The profile allows writes (`allow_writes`): the badge says `writes: CLI · enabled` (the TUI itself never
    /// writes to Xero; writes run from the CLI), else the quiet `◦ read-only`.
    pub writes: bool,
    /// The shown manifest (None on main): its name, and the stage's short form (`7 changes · 1 issue`,
    /// `check ✗ 2`, `planned`, `applying 3/7`, `stopped ✗`, `applied ✓`) with whether it is an error.
    pub manifest: Option<(String, String, bool)>,
}

/// The header segment naming the running org.
#[derive(Clone, Debug)]
pub struct RunSeg {
    pub who: String,
    pub left: Option<i64>,
    pub level: Level,
    /// Its figure is the record's local estimate (no live or reported figure yet).
    pub estimated: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Part {
    Product,
    Badge,
    BadgeOffline,
    Scope,
    AsOf,
    Fresh(Level),
    Quota(Level),
    /// The agent mark; true = acted.
    Agent(bool),
    /// The drift chip (a state rendered against another record version or config).
    Drift,
    /// `classify: N open · M stale` (true = something is stale).
    Classify(bool),
    /// The manifest chip: `⎇ name` (proposed, bold), then the stage's short form (quiet; `err` when it is an
    /// error: `check ✗ 2`, `stopped ✗`). Never dropped; it shortens.
    ManifestName,
    ManifestCount,
    ManifestError,
    ManifestWarn,
    Sep,
    Gap,
}

/// The drift chip when both the record and the config drifted.
pub const DRIFT_BOTH: &str = "≠ record and config changed";

/// The drift chip for what drifted: the record, the config, the labels (the classification store).
pub fn drift_text(record: bool, config: bool, labels: bool) -> &'static str {
    match (record, config, labels) {
        (true, true, true) => "≠ record, config and labels changed",
        (true, true, false) => DRIFT_BOTH,
        (true, false, true) => "≠ record and labels changed",
        (false, true, true) => "≠ config and labels changed",
        (false, true, false) => "≠ config changed",
        (false, false, true) => "≠ labels changed",
        _ => "≠ record changed",
    }
}

/// The drift chip at a degrade step: full; both-drift shortens to `≠ record, config changed` from
/// step 4 (where the scope goes compact); at the last two steps `changed` drops, then `≠` alone (the state's
/// `drift` and `?` help keep the full text).
fn drift_chip(chip: &'static str, step: usize) -> &'static str {
    let chip = match chip {
        DRIFT_BOTH if step >= 4 => "≠ record, config changed",
        "≠ record, config and labels changed" if step >= 4 => "≠ record, config, labels changed",
        "≠ record and labels changed" if step >= 4 => "≠ record, labels changed",
        "≠ config and labels changed" if step >= 4 => "≠ config, labels changed",
        c => c,
    };
    if step < 10 {
        chip
    } else if step < 11 {
        chip.trim_end_matches(" changed")
    } else {
        "≠"
    }
}

/// The degrade steps (spec §3.1 steps 1–5, then the shell's steps 6–10 so that every protected item
/// still fits at 60 columns with OFFLINE, a 4-org compare scope, a stale or missing record and a full
/// quota). Protected items never change after their last listed form.
pub const STEPS: usize = 12;

/// The header at degrade step `step` (0 = full). v0.12.0 (design pass 2, "calm unless something's wrong"):
/// the scope chip and the date always; then the exceptions only when they apply (drift, stale record,
/// offline, classify, the agent, a low or busy budget); the quiet `◦ read-only` at the right end (the fit
/// pads before it).
pub fn header_at(f: &HeaderFacts, step: usize, ascii: bool) -> Vec<(String, Part)> {
    let fold = |s: &str| crate::glyphs::fold(s, ascii);
    // the gap between groups: wide while it fits, then narrower
    let sep = if step < 5 {
        "    ".to_string()
    } else if step < 9 {
        "  ".into()
    } else {
        " ".into()
    };
    let _ = &f.product;
    let mut out: Vec<(String, Part)> = vec![(if step < 9 { "  " } else { " " }.into(), Part::Gap)];
    let scope = if step < 4 {
        &f.scope_full
    } else if step < 11 || f.scope_min.is_empty() {
        &f.scope_compact
    } else {
        &f.scope_min
    };
    out.push((fold(scope), Part::Scope));
    // the manifest chip, right after the scope (design spec branches §2, manifests §3): never dropped, it
    // shortens by width; the stage's short form comes from the core (`Stage::short`), so it always agrees
    // with the stage strip
    if let Some((name, stage, err)) = &f.manifest {
        let short = middle_trunc(name, 16);
        out.push((sep.clone(), Part::Sep));
        let compact = manifest_compact(stage, *err);
        let form = if step < 4 {
            Some(stage.clone())
        } else if step < 9 {
            Some(compact)
        } else if *err {
            Some("✗".to_string())
        } else {
            None
        };
        if f.plain || ascii {
            let text = match &form {
                Some(s) => format!("[manifest {short} · {s}]"),
                None => format!("[manifest {short}]"),
            };
            out.push((fold(&text), Part::ManifestName));
        } else {
            out.push((format!("⎇ {short}"), Part::ManifestName));
            if let Some(s) = form {
                out.push((
                    format!(" · {s}"),
                    if *err {
                        Part::ManifestError
                    } else {
                        Part::ManifestCount
                    },
                ));
            }
        }
    }
    out.push((sep.clone(), Part::Sep));
    let as_of = if step < 3 {
        format!("as of {}", f.as_of)
    } else {
        f.as_of.clone()
    };
    out.push((as_of, Part::AsOf));
    // the exception group, after "as of"
    let ex = |text: String, part: Part, out: &mut Vec<(String, Part)>| {
        out.push((sep.clone(), Part::Sep));
        out.push((text, part));
    };
    if let Some(chip) = f.drift {
        ex(fold(drift_chip(chip, step)), Part::Drift, &mut out);
    }
    if f.offline {
        ex("offline".into(), Part::BadgeOffline, &mut out);
    }
    if f.show_age {
        let fresh = if step < 2 {
            &f.fresh_full
        } else if step < 6 {
            &f.fresh_compact
        } else {
            &f.fresh_min
        };
        ex(fold(fresh), Part::Fresh(f.fresh_level), &mut out);
    }
    if let Some((open, stale)) = f.classify.filter(|_| step < 4) {
        let text = if stale > 0 {
            format!(
                "classify: {} open · {} stale",
                crate::clock::int(open as i64),
                crate::clock::int(stale as i64)
            )
        } else {
            format!("classify: {} open", crate::clock::int(open as i64))
        };
        ex(fold(&text), Part::Classify(stale > 0), &mut out);
    }
    if let Some(acted) = f.agent.filter(|_| step < 10) {
        let mark = if ascii || f.plain {
            if acted { "[agent*]" } else { "[agent]" }.to_string()
        } else {
            let g = if acted { "◆" } else { "◇" };
            if step < 2 {
                format!("{g} agent")
            } else {
                g.to_string()
            }
        };
        ex(mark, Part::Agent(acted), &mut out);
    }
    if f.show_quota || f.busy {
        let quota = quota_text(f.left, f.limit, f.left_estimated, step);
        let quota = if f.busy {
            format!("⇅ {quota}")
        } else {
            quota
        };
        ex(fold(&quota), Part::Quota(f.quota_level), &mut out);
    }
    if let Some(r) = &f.run {
        let seg = match r.left {
            Some(l) => format!(
                "⇅ {} · {}",
                r.who,
                quota_text(l, f.limit, r.estimated, step)
            ),
            None => format!("⇅ {}", r.who),
        };
        ex(fold(&seg), Part::Quota(r.level), &mut out);
    }
    // the quiet read-only mark, last (right-aligned by `header`)
    out.push((sep, Part::Sep));
    let badge = match (f.writes, step) {
        (true, s) if s < 6 => "writes: CLI · enabled",
        (true, _) => "writes: CLI",
        // writes off (the default): the v0.12 mark, unchanged (frozen frames stay byte-identical)
        (false, s) if s < 10 && !(ascii || f.plain) => "◦ read-only",
        (false, _) => "read-only",
    };
    out.push((badge.into(), Part::Badge));
    out.push((if step < 9 { "  " } else { "" }.into(), Part::Gap));
    out
}

/// A calls-left figure at degrade step `step`: `938/1,000 left` → `938 left` → `938`; the local estimate
/// `≈ 1,000 left (estimated)` → `≈ 1,000 (est.)` → `≈1,000` (the last only at the narrowest widths).
pub fn quota_text(left: i64, limit: i64, estimated: bool, step: usize) -> String {
    let l = crate::clock::int(left);
    match (estimated, step) {
        (true, s) if s < 5 => format!("≈ {l} left (estimated)"),
        (true, s) if s < 10 => format!("≈ {l} (est.)"),
        (true, _) => format!("≈{l}"),
        (false, s) if s < 5 => format!("{l}/{} left", crate::clock::int(limit)),
        (false, s) if s < 10 => format!("{l} left"),
        (false, _) => l,
    }
}

/// The first degrade step that fits `avail` cells (the most compact form if none does).
pub fn header(f: &HeaderFacts, avail: usize, ascii: bool) -> Vec<(String, Part)> {
    for step in 0..STEPS {
        let parts = header_at(f, step, ascii);
        let used = parts.iter().map(|(s, _)| width(s)).sum::<usize>();
        if used <= avail {
            return right_align(parts, avail - used);
        }
    }
    header_at(f, STEPS - 1, ascii)
}

/// Pad before the read-only mark so it ends at the right edge (the exceptions stay left).
fn right_align(mut parts: Vec<(String, Part)>, extra: usize) -> Vec<(String, Part)> {
    if let Some(i) = parts.iter().rposition(|(_, p)| *p == Part::Sep) {
        parts[i].0.push_str(&" ".repeat(extra));
    }
    parts
}

/// One styled piece of the header scope chip (the text and its token; theme spec v2 §2.3).
pub type ChipSpan = (String, crate::theme_tokens::Token);

/// The chip's text, for fitting (the pieces joined).
pub fn chip_text(spans: &[ChipSpan]) -> String {
    spans.iter().map(|(t, _)| t.as_str()).collect()
}

/// The manifest chip's middle form: `7 changes · 1 issue` → `7`, `7 · 1` when issues; the stage words
/// (`planned`, `check ✗ 2`, `applying 3/7`, …) stay as they are.
pub fn manifest_compact(stage: &str, err: bool) -> String {
    let mut parts = stage.split(" · ");
    let first = parts.next().unwrap_or("");
    match first.split_once(' ') {
        Some((n, "change" | "changes")) if !err => {
            let issues = parts
                .next()
                .and_then(|p| p.split_whitespace().next())
                .map(|i| format!(" · {i}!"))
                .unwrap_or_default();
            format!("{n}{issues}")
        }
        _ => stage.to_string(),
    }
}

/// `7 changes` / `1 change`.
pub fn plural_changes(n: usize) -> String {
    if n == 1 {
        "1 change".into()
    } else {
        format!("{n} changes")
    }
}

/// A name cut in the middle to `max` cells (`cleanup-…-2025`), whole when it fits.
pub fn middle_trunc(s: &str, max: usize) -> String {
    let n = s.chars().count();
    if n <= max {
        return s.to_string();
    }
    let keep = max.saturating_sub(1);
    let head = keep.div_ceil(2);
    let tail = keep - head;
    let a: String = s.chars().take(head).collect();
    let b: String = s.chars().skip(n - tail).collect();
    format!("{a}…{b}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn drift_chip_uses_words_and_shortens_late() {
        assert_eq!(drift_chip(DRIFT_BOTH, 0), "≠ record and config changed");
        assert_eq!(drift_chip(DRIFT_BOTH, 4), "≠ record, config changed");
        assert_eq!(drift_chip(DRIFT_BOTH, 10), "≠ record, config");
        assert_eq!(drift_chip("≠ record changed", 9), "≠ record changed");
        assert_eq!(drift_chip("≠ record changed", 10), "≠ record");
        assert_eq!(drift_chip("≠ config changed", 11), "≠");
        assert_eq!(
            drift_chip(drift_text(false, false, true), 0),
            "≠ labels changed"
        );
        assert_eq!(drift_chip(drift_text(false, false, true), 10), "≠ labels");
        assert_eq!(
            drift_chip(drift_text(true, false, true), 4),
            "≠ record, labels changed"
        );
    }

    #[test]
    fn fit_is_grapheme_correct() {
        assert_eq!(
            fit_str("SALTMARSH SENSORY PANEL CARD™", 12, "…"),
            "SALTMARSH S…"
        );
        assert_eq!(fit_str("東京銀行株式会社", 7, "…"), "東京銀…");
        assert_eq!(fit_str("東京銀行株式会社", 6, "…"), "東京…");
        assert_eq!(width(&fit_str("café 👩‍👩‍👧 lunch", 8, "…")), 8);
        assert_eq!(fit_str("e\u{301}tude", 3, "~"), "e\u{301}t~");
        assert_eq!(fit_str("short", 10, "…"), "short");
        assert_eq!(fit_str("abc", 0, "…"), "");
    }

    #[test]
    fn wrap_never_cuts() {
        let text = "Not available yet (coming in v0.9.x): `ownpurse parity one --date 2025-12-31 --since 2024-12-31`";
        for w in 10..120 {
            let lines = wrap(text, w);
            assert!(lines.iter().all(|l| width(l) <= w), "{w}");
            let joined: String = lines.concat().split_whitespace().collect();
            let orig: String = text.split_whitespace().collect();
            assert_eq!(joined, orig, "{w}");
        }
    }

    #[test]
    fn columns_degrade_in_declared_order() {
        let cols = [
            ColSpec {
                title: "org",
                short_title: "",
                kind: Kind::Text { min: 6 },
                priority: 9,
            },
            ColSpec {
                title: "name",
                short_title: "",
                kind: Kind::Text { min: 8 },
                priority: 2,
            },
            ColSpec {
                title: "id",
                short_title: "",
                kind: Kind::Id,
                priority: 1,
            },
            ColSpec {
                title: "fetched",
                short_title: "",
                kind: Kind::Status,
                priority: 8,
            },
            ColSpec {
                title: "calls",
                short_title: "",
                kind: Kind::Amount,
                priority: 9,
            },
        ];
        let nat = [10, 30, 8, 22, 11];
        // plenty of room: natural widths
        assert_eq!(fit_columns(&cols, &nat, 200, 2), nat.map(Some).to_vec());
        for avail in 0..120 {
            let w = fit_columns(&cols, &nat, avail, 2);
            // amount and status never shrink or hide (V8)
            assert_eq!(w[3], Some(22));
            assert_eq!(w[4], Some(11));
            // text never below its min while visible
            if let Some(n) = w[1] {
                assert!(n >= 8);
            }
            // the id column (priority 1) hides before name shortens to nothing
            if w[2].is_some() {
                assert!(w[1].is_some());
            }
        }
        // name shortens first (lowest text priority)
        let w = fit_columns(&cols, &nat, 85, 2);
        assert_eq!(w, vec![Some(10), Some(26), Some(8), Some(22), Some(11)]);
    }
}
