//! Terminal probes at start (spec §2.2, §8): the OSC 11 background query (dark or light for theme `auto`)
//! and the DECRQM 2026 query (synchronized output). Both are sent together with a DA1 request, which every
//! terminal answers, and the reply is read until the DA1 answer or a short timeout, so nothing ever hangs.
//! No reply → the caller keeps its terminal-name and environment fallback. Parsers are pure and unit-tested;
//! the reader is injected, so tests never touch a terminal.
//!
//! Also here: the Terminal.app note's "shown once ever" marker under `$XDG_STATE_HOME/ownpurse/`.

use std::io::Write;
use std::path::PathBuf;
use std::time::{Duration, Instant};

/// OSC 11 (background colour?), DECRQM ?2026 (synchronized output?), then DA1 as the sentinel.
pub const QUERY: &[u8] = b"\x1b]11;?\x1b\\\x1b[?2026$p\x1b[c";

/// The spec's budget for the background query.
pub const TIMEOUT: Duration = Duration::from_millis(50);

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Probe {
    /// Some(true) dark background, Some(false) light, None no answer.
    pub dark: Option<bool>,
    /// Some(true) the terminal knows mode 2026, Some(false) it said it doesn't, None no answer.
    pub sync: Option<bool>,
}

fn hex_channel(s: &str) -> Option<u64> {
    if s.is_empty() || s.len() > 4 {
        return None;
    }
    let v = u64::from_str_radix(s, 16).ok()?;
    // scale 1–4 hex digits to 16 bits
    let max = (1u64 << (4 * s.len())) - 1;
    Some(v * 0xFFFF / max)
}

/// `ESC ] 11 ; rgb:RRRR/GGGG/BBBB (BEL | ESC \)` → dark (relative luminance below one half). Integer
/// arithmetic only (Rec. 709 weights ×10 000).
pub fn parse_osc11(reply: &[u8]) -> Option<bool> {
    let text = String::from_utf8_lossy(reply);
    let start = text.find("]11;")? + 4;
    let rest = &text[start..];
    let end = rest.find(['\x07', '\x1b']).unwrap_or(rest.len());
    let spec = rest[..end].trim();
    let body = spec
        .strip_prefix("rgba:")
        .or_else(|| spec.strip_prefix("rgb:"))?;
    let parts: Vec<&str> = body.split('/').collect();
    if parts.len() < 3 {
        return None;
    }
    let (r, g, b) = (
        hex_channel(parts[0])?,
        hex_channel(parts[1])?,
        hex_channel(parts[2])?,
    );
    let lum = 2126 * r + 7152 * g + 722 * b; // up to 10 000 × 0xFFFF
    Some(lum < 5000 * 0xFFFF)
}

/// `ESC [ ? 2026 ; Ps $ y` → Ps 1 or 2 (set / reset): supported; 0 or 4 (unknown / permanently reset): not.
pub fn parse_decrqm(reply: &[u8], mode: u32) -> Option<bool> {
    let text = String::from_utf8_lossy(reply);
    let key = format!("[?{mode};");
    let start = text.find(&key)? + key.len();
    let rest = &text[start..];
    let end = rest.find("$y")?;
    match rest[..end].trim() {
        "1" | "2" => Some(true),
        "0" | "4" => Some(false),
        "3" => Some(true), // permanently set
        _ => None,
    }
}

/// The DA1 answer (`ESC [ ? … c`) arrived: everything sent before it has been answered too.
pub fn has_da1(reply: &[u8]) -> bool {
    let text = String::from_utf8_lossy(reply);
    text.match_indices("\x1b[?").any(|(i, _)| {
        let tail = &text[i + 3..];
        let end = tail.find(|c: char| !(c.is_ascii_digit() || c == ';'));
        end.is_some_and(|e| tail[e..].starts_with('c'))
    })
}

/// Send the query and read until the DA1 answer or `timeout`. `read` returns the bytes available within
/// the time it's given (None = nothing came).
pub fn run(
    out: &mut dyn Write,
    read: &mut dyn FnMut(Duration) -> Option<Vec<u8>>,
    timeout: Duration,
) -> Probe {
    if out.write_all(QUERY).and_then(|_| out.flush()).is_err() {
        return Probe::default();
    }
    let started = Instant::now();
    let mut buf = Vec::new();
    while !has_da1(&buf) {
        let left = timeout.saturating_sub(started.elapsed());
        if left.is_zero() {
            break;
        }
        match read(left) {
            Some(bytes) if !bytes.is_empty() => buf.extend(bytes),
            _ => break,
        }
    }
    Probe {
        dark: parse_osc11(&buf),
        sync: parse_decrqm(&buf, 2026),
    }
}

/// Read stdin with a timeout (raw mode must be on): poll(2) then read(2).
#[cfg(unix)]
pub fn stdin_reader() -> impl FnMut(Duration) -> Option<Vec<u8>> {
    |d: Duration| {
        let mut fds = libc::pollfd {
            fd: 0,
            events: libc::POLLIN,
            revents: 0,
        };
        let ms = d.as_millis().min(i32::MAX as u128) as i32;
        // SAFETY: one valid pollfd for stdin; the buffer outlives the read.
        let ready = unsafe { libc::poll(&mut fds, 1, ms) };
        if ready <= 0 {
            return None;
        }
        let mut buf = [0u8; 256];
        let n = unsafe { libc::read(0, buf.as_mut_ptr().cast(), buf.len()) };
        (n > 0).then(|| buf[..n as usize].to_vec())
    }
}

// ── the Terminal.app note, shown once ever ─────────────────────────────────────────────────────────

/// `$XDG_STATE_HOME/ownpurse/terminal-app-note`, else `~/.local/state/ownpurse/terminal-app-note`.
pub fn terminal_note_marker(env: &dyn Fn(&str) -> Option<String>) -> Option<PathBuf> {
    let base = env("XDG_STATE_HOME")
        .filter(|v| !v.is_empty())
        .map(PathBuf::from)
        .or_else(|| {
            env("HOME")
                .filter(|h| !h.is_empty())
                .map(|h| PathBuf::from(h).join(".local").join("state"))
        })?;
    Some(base.join("ownpurse").join("terminal-app-note"))
}

/// Show the note now? Only in Terminal.app and only if the marker isn't there yet.
pub fn terminal_note_due(
    env: &dyn Fn(&str) -> Option<String>,
    exists: &dyn Fn(&std::path::Path) -> bool,
) -> bool {
    env("TERM_PROGRAM").as_deref() == Some("Apple_Terminal")
        && terminal_note_marker(env).is_some_and(|m| !exists(&m))
}

/// Record that the note was shown (best effort: a read-only home just shows it again next time).
pub fn mark_terminal_note(env: &dyn Fn(&str) -> Option<String>) {
    if let Some(m) = terminal_note_marker(env) {
        if let Some(dir) = m.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        let _ = std::fs::write(&m, b"shown\n");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn osc11_dark_and_light() {
        assert_eq!(parse_osc11(b"\x1b]11;rgb:1414/1919/1c1c\x07"), Some(true));
        assert_eq!(
            parse_osc11(b"\x1b]11;rgb:f5f5/f7f7/f5f5\x1b\\"),
            Some(false)
        );
        assert_eq!(
            parse_osc11(b"\x1b]11;rgb:ff/ff/ff\x07"),
            Some(false),
            "2-digit channels"
        );
        assert_eq!(
            parse_osc11(b"\x1b]11;rgba:0000/0000/0000/ffff\x07"),
            Some(true)
        );
        assert_eq!(parse_osc11(b"\x1b]11;?\x07"), None);
        assert_eq!(parse_osc11(b"garbage"), None);
    }

    #[test]
    fn decrqm_states() {
        assert_eq!(parse_decrqm(b"\x1b[?2026;2$y", 2026), Some(true));
        assert_eq!(parse_decrqm(b"\x1b[?2026;1$y", 2026), Some(true));
        assert_eq!(parse_decrqm(b"\x1b[?2026;0$y", 2026), Some(false));
        assert_eq!(parse_decrqm(b"\x1b[?2026;4$y", 2026), Some(false));
        assert_eq!(parse_decrqm(b"\x1b[?1049;2$y", 2026), None);
        assert_eq!(parse_decrqm(b"", 2026), None);
    }

    #[test]
    fn a_full_reply_in_pieces() {
        let mut chunks = vec![
            b"\x1b]11;rgb:1b1b/2222/2626\x1b\\".to_vec(),
            b"\x1b[?2026;2$y".to_vec(),
            b"\x1b[?62;22c".to_vec(),
        ]
        .into_iter();
        let mut out = Vec::new();
        let p = run(&mut out, &mut |_| chunks.next(), Duration::from_millis(200));
        assert_eq!(out, QUERY);
        assert_eq!(
            p,
            Probe {
                dark: Some(true),
                sync: Some(true)
            }
        );
    }

    #[test]
    fn no_reply_times_out_without_hanging() {
        let started = Instant::now();
        let mut out = Vec::new();
        let p = run(&mut out, &mut |_| None, TIMEOUT);
        assert_eq!(p, Probe::default());
        assert!(
            started.elapsed() < Duration::from_secs(10),
            "it returned (a loaded machine is slow, never stuck)"
        );
        // a terminal that answers DA1 only (no OSC 11, no DECRQM): stop at once, fall back
        let mut once = Some(b"\x1b[?1;2c".to_vec());
        let p = run(&mut out, &mut |_| once.take(), TIMEOUT);
        assert_eq!(p, Probe::default());
        // a reader that trickles without DA1 stops at the timeout
        let started = Instant::now();
        let p = run(
            &mut out,
            &mut |d| {
                std::thread::sleep(d.min(Duration::from_millis(5)));
                Some(b"x".to_vec())
            },
            Duration::from_millis(30),
        );
        assert_eq!(p, Probe::default());
        assert!(
            started.elapsed() < Duration::from_secs(10),
            "it returned (a loaded machine is slow, never stuck)"
        );
    }

    #[test]
    fn terminal_note_marker_path_and_once() {
        let env = |pairs: &'static [(&'static str, &'static str)]| {
            move |k: &str| {
                pairs
                    .iter()
                    .find(|(a, _)| *a == k)
                    .map(|(_, v)| v.to_string())
            }
        };
        assert_eq!(
            terminal_note_marker(&env(&[("XDG_STATE_HOME", "/s"), ("HOME", "/h")])),
            Some(PathBuf::from("/s/ownpurse/terminal-app-note"))
        );
        assert_eq!(
            terminal_note_marker(&env(&[("HOME", "/h")])),
            Some(PathBuf::from("/h/.local/state/ownpurse/terminal-app-note"))
        );
        let apple = env(&[("TERM_PROGRAM", "Apple_Terminal"), ("HOME", "/h")]);
        assert!(terminal_note_due(&apple, &|_| false));
        assert!(!terminal_note_due(&apple, &|_| true), "shown once ever");
        assert!(!terminal_note_due(
            &env(&[("TERM_PROGRAM", "WezTerm"), ("HOME", "/h")]),
            &|_| false
        ));
        // marking writes only under the given state dir (a temp dir here, never the real home)
        let dir = std::env::temp_dir().join(format!("ownpurse-note-{}", std::process::id()));
        let d = dir.display().to_string();
        let tmp_env = move |k: &str| (k == "XDG_STATE_HOME").then(|| d.clone());
        mark_terminal_note(&tmp_env);
        assert!(dir.join("ownpurse/terminal-app-note").exists());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
