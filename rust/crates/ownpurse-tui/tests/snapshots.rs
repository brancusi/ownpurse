//! Snapshot grid (spec §10, §13): every shell state × size × theme mode, against the committed fixture
//! record with the clock pinned (`snapshot::FIXTURE_NOW`). XS (60×20) snapshots keep only the header row.
//! Each snapshot also runs the V5 header check and the V12 NO_COLOR word/glyph checks.
//!
//! Review changes with `cargo insta review` (or `INSTA_UPDATE=always cargo test` to accept).

use ownpurse_tui::snapshot::{
    self, header_problems, row_text, shell_states, state_app, styled, themes,
};

/// `\b1 (accounts|calls|pairs|lines|items)\b`.
fn singular_problems(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    for noun in ["accounts", "calls", "pairs", "lines", "items"] {
        let pat = format!("1 {noun}");
        for (i, _) in text.match_indices(&pat) {
            let before = text[..i].chars().next_back();
            let after = text[i + pat.len()..].chars().next();
            let word_start = before.is_none_or(|c| !(c.is_alphanumeric() || c == ',' || c == '.'));
            let word_end = after.is_none_or(|c| !c.is_alphanumeric());
            if word_start && word_end {
                out.push(format!("\"{pat}\""));
            }
        }
    }
    out
}

/// `\b\d{4,}\.\d{2}\b` without thousands separators: an amount that skipped the formatter.
fn unformatted_amounts(text: &str) -> Vec<String> {
    let b = text.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i < b.len() {
        if b[i].is_ascii_digit()
            && (i == 0 || !(b[i - 1].is_ascii_digit() || b[i - 1] == b',' || b[i - 1] == b'.'))
        {
            let start = i;
            while i < b.len() && b[i].is_ascii_digit() {
                i += 1;
            }
            let run = i - start;
            if run >= 4
                && i + 2 < b.len() + 1
                && b.get(i) == Some(&b'.')
                && b.get(i + 1).is_some_and(u8::is_ascii_digit)
                && b.get(i + 2).is_some_and(u8::is_ascii_digit)
                && !b.get(i + 3).is_some_and(u8::is_ascii_digit)
            {
                out.push(String::from_utf8_lossy(&b[start..i + 3]).into_owned());
            }
        } else {
            i += 1;
        }
    }
    out
}

#[test]
fn unformatted_amount_scan() {
    assert_eq!(unformatted_amounts("found 1013.57 · x"), vec!["1013.57"]);
    assert!(unformatted_amounts("1,013.57 2025-12-31 v0.9.0 999.00 bt-108 07:46").is_empty());
}

#[test]
fn singular_scan_catches_copy_bugs() {
    assert_eq!(singular_problems("x 1 accounts differ").len(), 1);
    assert!(singular_problems("11 accounts · 1,001 calls · 21 pairs").is_empty());
}

#[test]
fn shell_snapshots() {
    let mut problems = Vec::new();
    let mut count = 0;
    for st in shell_states() {
        for (theme_name, theme) in themes() {
            for &(w, h) in st.sizes {
                if !st.renders(theme_name, (w, h)) {
                    continue;
                }
                let mut app = state_app(&st, theme, w, h);
                let buf = snapshot::render(&mut app, w, h);
                let name = format!("{}-{w}x{h}-{theme_name}", st.name);
                if st.name != "too-small" {
                    for p in header_problems(&app, &row_text(&buf, 0)) {
                        problems.push(format!("{name}: {p}"));
                    }
                }
                for p in singular_problems(&snapshot::text(&buf)) {
                    problems.push(format!("{name}: {p}"));
                }
                for p in unformatted_amounts(&snapshot::text(&buf)) {
                    problems.push(format!("{name}: unformatted amount {p}"));
                }
                let shown = if w == 60 {
                    snapshot::crop(&buf, 1)
                } else {
                    buf
                };
                insta::assert_snapshot!(name, styled(&shown));
                count += 1;
            }
        }
    }
    assert!(problems.is_empty(), "{}", problems.join("\n"));
    assert!(count >= 200, "{count} snapshots");
}

/// Snapshots render a fixed version label (like the pinned clock), so a version bump changes none of them.
#[test]
fn snapshots_never_carry_the_crate_version() {
    use ownpurse_tui::app::VERSION;
    let mut a = snapshot::state_app(
        &shell_states()[0],
        ownpurse_tui::theme::LEDGER_DARK,
        140,
        40,
    );
    // the version lives in the footer, never in the header
    let buf = snapshot::render(&mut a, 140, 40);
    let (row0, foot) = (row_text(&buf, 0), row_text(&buf, 39));
    assert!(
        foot.trim_end()
            .ends_with(&format!("v{}", snapshot::TEST_VERSION)),
        "{foot}"
    );
    assert!(
        !row0.contains(&format!("v{}", snapshot::TEST_VERSION)),
        "{row0}"
    );
    assert!(!foot.contains(&format!("v{VERSION}")) || VERSION == snapshot::TEST_VERSION);
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/snapshots");
    for e in std::fs::read_dir(dir).unwrap().flatten() {
        let t = std::fs::read_to_string(e.path()).unwrap_or_default();
        assert!(
            !t.contains(&format!("v{VERSION} ")),
            "{} carries the crate version",
            e.path().display()
        );
    }
}

/// Spec §6: below 100 columns global keys are key-only; at 100+ every footer key shows its label (or drops
/// out with it), never a bare key.
#[test]
fn footer_keys_keep_labels_from_100_columns() {
    for st in shell_states().into_iter().filter(|s| {
        [
            "overview",
            "overview-group",
            "activity",
            "ledger-roast",
            "system",
        ]
        .contains(&s.name)
    }) {
        let mut a = snapshot::state_app(&st, ownpurse_tui::theme::LEDGER_DARK, 100, 30);
        for w in 60..=220u16 {
            let buf = snapshot::render(&mut a, w, 30);
            let last = row_text(&buf, 29);
            let words: Vec<&str> = last
                .split("  ")
                .map(str::trim)
                .filter(|x| !x.is_empty())
                .collect();
            for bare in ["q", "/", "]", "?", "g", "r", "d"] {
                let has_bare = words.contains(&bare);
                if w >= 100 {
                    assert!(!has_bare, "{} {w}: bare `{bare}` in {last:?}", st.name);
                }
            }
            if w >= 100 && last.contains(" q ") || last.contains("  q Quit") {
                assert!(last.contains("q Quit"), "{} {w}: {last}", st.name);
            }
        }
    }
}

/// Overlay boxes in a frame (inclusive corners: left, top, right, bottom), unicode or ASCII.
fn boxes(buf: &ratatui::buffer::Buffer) -> Vec<(u16, u16, u16, u16)> {
    let sym = |x: u16, y: u16| -> &str {
        if x < buf.area.right() && y < buf.area.bottom() {
            buf[(x, y)].symbol()
        } else {
            ""
        }
    };
    let mut out = Vec::new();
    for y in buf.area.top()..buf.area.bottom() {
        for x in buf.area.left()..buf.area.right() {
            let tl = sym(x, y) == "╭"
                || (sym(x, y) == "+" && sym(x + 1, y) == "-" && sym(x, y + 1) == "|");
            if !tl {
                continue;
            }
            // walk right along the top edge to the top-right corner, then down to the bottom-right
            let mut r = x + 1;
            while r < buf.area.right() && !matches!(sym(r, y), "╮" | "+") {
                r += 1;
            }
            let mut b = y + 1;
            while b < buf.area.bottom() && !matches!(sym(r, b), "╯" | "+") {
                b += 1;
            }
            if r < buf.area.right() && b < buf.area.bottom() {
                out.push((x, y, r, b));
            }
        }
    }
    out
}

/// The reversed cells of one body row (NO_COLOR) per region (outside the overlay boxes, and inside each
/// box): their spans (start, end) and whether any reversed cell is DIM. Cells under a box don't break a
/// span of the row behind it.
fn reverse_spans(buf: &ratatui::buffer::Buffer, y: u16) -> Vec<(Vec<(u16, u16)>, bool)> {
    use ratatui::style::Modifier;
    let bx: Vec<_> = boxes(buf)
        .into_iter()
        .filter(|b| (b.1..=b.3).contains(&y))
        .collect();
    let mut regions = vec![None];
    regions.extend(bx.iter().map(|b| Some(*b)));
    regions
        .into_iter()
        .map(|reg| {
            let mut spans: Vec<(u16, u16)> = Vec::new();
            let mut dim = false;
            for x in buf.area.left()..buf.area.right() {
                let under = bx
                    .iter()
                    .find(|b| (b.0.saturating_sub(1)..=b.2 + 1).contains(&x));
                let mine = match reg {
                    None => under.is_none(),
                    Some(b) => x > b.0 && x < b.2,
                };
                if !mine {
                    if reg.is_none()
                        && let Some(s) = spans.last_mut()
                        && s.1 == x
                    {
                        s.1 = x + 1; // transparent: the box covers this cell
                    }
                    continue;
                }
                let c = &buf[(x, y)];
                if c.modifier.contains(Modifier::REVERSED) {
                    dim |= c.modifier.contains(Modifier::DIM);
                    match spans.last_mut() {
                        Some(s) if s.1 == x => s.1 = x + 1,
                        _ => spans.push((x, x + 1)),
                    }
                }
            }
            (spans, dim)
        })
        .collect()
}

/// v0.12.0 (design review): in NO_COLOR a selected row is ONE continuous reverse span with no DIM cell
/// inside it (DIM on a reversed cell dims its background in many terminals and splits the band), on every
/// state's body rows (the header badges and the footer are their own spans).
#[test]
fn nocolor_selected_row_is_one_reverse_span() {
    let th = ownpurse_tui::theme::NO_COLOR_ASCII;
    let (w, h) = (100u16, 30u16);
    let mut problems = Vec::new();
    let mut check = |name: &str, buf: &ratatui::buffer::Buffer| {
        for y in 3..h.saturating_sub(1) {
            for (spans, dim) in reverse_spans(buf, y) {
                if spans.len() > 1 || dim {
                    problems.push(format!("{name} row {y}: {spans:?} dim={dim}"));
                }
            }
        }
    };
    for st in shell_states() {
        if !st.sizes.contains(&(w, h)) && !st.sizes.contains(&(80, 24)) {
            continue;
        }
        let mut app = state_app(&st, th, w, h);
        let buf = snapshot::render(&mut app, w, h);
        check(st.name, &buf);
    }
    for st in snapshot::tab_states() {
        let mut app = snapshot::tab_state_app(&st, th, w, h);
        let buf = snapshot::render(&mut app, w, h);
        check(st.name, &buf);
    }
    assert!(problems.is_empty(), "{}", problems.join("\n"));
}
