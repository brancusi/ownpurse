//! `ownpurse-tui`: terminal setup and teardown, theme/glyph detection, the synchronized-output probe,
//! the event loop (input thread → one channel → `update` → draw only when dirty).
//!
//!     ownpurse-tui [--config ownpurse.json] [--profile name] [--offline]     (config found as the CLI finds it)
//!     ownpurse-tui --record <record.sqlite> --orgs alias=tenant,... [--group name=a+b] [--offline]
//!     ownpurse-tui --keys [markdown|json|conflicts]
//!     ownpurse-tui ... --snapshot 80x24 [--script '2 ] ]']      one plain-text frame to stdout
//!     ownpurse-tui ... --dump-states <dir> [--sizes 80x24,100x30,140x40] [--theme ledger-dark]
//!
//! Config comes from core's loader (as the CLI); `--record`/`--orgs` run without one. Network actions run the installed `ownpurse` CLI as a child process.

use crossterm::event::{self, Event, KeyEventKind, MouseButton, MouseEventKind};
use crossterm::{execute, terminal};
use ownpurse_tui::app::{App, Effect, Msg};
use ownpurse_tui::model::{Model, Source};
use ownpurse_tui::quiet::Quiet;
use ownpurse_tui::runtime::{Clicks, Runtime};
use ownpurse_tui::theme::Theme;
use ownpurse_tui::{clock, keymap, snapshot};
use ratatui::Terminal;
use std::io::Write;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::time::Duration;

struct Args {
    record: Option<PathBuf>,
    config: Option<PathBuf>,
    orgs: Vec<(String, String)>,
    groups: Vec<(String, Vec<String>)>,
    offline: bool,
    theme: Option<String>,
    glyphs: Option<String>,
    profile: Option<String>,
    now: Option<i64>,
    mouse: bool,
    keys: Option<String>,
    /// `--echo-keys`: print each key as keys.toml spells it (to check ⌘ keys reach the TUI); q twice quits.
    echo_keys: bool,
    /// `--check-theme`: the theme, accents and intensities as configured, with every warning (v0.12.0).
    check_theme: bool,
    snapshot: Option<(u16, u16)>,
    script: String,
    dump: Option<PathBuf>,
    sizes: Vec<(u16, u16)>,
    only: Option<String>,
    /// `--format txt,html,ansi` for `--dump-states` (txt alone writes into the dir; several use subdirs).
    formats: Vec<String>,
    /// `--no-agent`: no agent bridge socket (as `tui.agent: false`).
    no_agent: bool,
    /// `--label`: this session's label in `ownpurse ui ls` (as `tui.label`).
    label: Option<String>,
    /// `--state FILE`: render from a canonical state (with `--snapshot`); `--allow-record-drift`,
    /// `--allow-config-drift`.
    state: Option<PathBuf>,
    allow_drift: bool,
    allow_config_drift: bool,
    /// `--allow-classify-drift`: render though the classification store moved since the state was saved.
    allow_classify_drift: bool,
    /// `--allow-manifest-drift`: render a state whose manifest moved since it was saved.
    allow_manifest_drift: bool,
    /// `--out FILE` for `--snapshot` (else stdout).
    out: Option<PathBuf>,
    /// `--check-state FILE`: the upgrade pre-flight (import, render, compare with the live frame; headless).
    check_state: Option<PathBuf>,
    /// `--resume FILE [--resume-reply FD]`: continue an upgraded session (the file is deleted once read).
    resume: Option<PathBuf>,
    resume_reply: Option<i32>,
    /// `--walkthrough ID`: start playing a walkthrough (interactive).
    walkthrough: Option<String>,
    /// `--walkthrough-validate ID`: lay out every step headlessly and report (JSON lines; exit 4 on findings).
    walk_validate: Option<String>,
    /// `--walkthrough-render ID --out DIR [--format txt,svg,png] [--sizes WxH,…]`: every step's frame.
    walk_render: Option<String>,
}

const USAGE: &str = "usage: ownpurse-tui [--config ownpurse.json] [--profile name] | --record <record.sqlite> --orgs alias=tenant,... [--group name=a+b] \
[--offline] [--theme ledger-dark|ledger-light|ledger-dark-256|ledger-light-256|ansi|no-color] [--glyphs unicode|ascii] \
[--profile name] [--now ISO] [--no-mouse] [--no-agent]\n       ownpurse-tui --keys [markdown|json|conflicts]   |   ownpurse-tui --echo-keys   |   ownpurse-tui --check-theme\n       \
... --snapshot WxH [--script KEYS]   |   ... --dump-states DIR [--sizes 80x24,...]";

fn size(s: &str) -> Result<(u16, u16), String> {
    let (w, h) = s.split_once('x').ok_or(format!("bad size {s}"))?;
    Ok((
        w.parse().map_err(|_| format!("bad size {s}"))?,
        h.parse().map_err(|_| format!("bad size {s}"))?,
    ))
}

fn parse_args() -> Result<Args, String> {
    let mut a = Args {
        record: None,
        config: None,
        orgs: Vec::new(),
        groups: Vec::new(),
        offline: false,
        theme: None,
        glyphs: None,
        profile: None,
        now: None,
        mouse: true,
        keys: None,
        echo_keys: false,
        check_theme: false,
        snapshot: None,
        script: std::env::var("OWNPURSE_TUI_KEYS").unwrap_or_default(),
        dump: None,
        sizes: vec![(80, 24), (100, 30), (140, 40)],
        only: None,
        formats: vec!["txt".into()],
        no_agent: false,
        label: None,
        state: None,
        allow_drift: false,
        allow_config_drift: false,
        allow_classify_drift: false,
        allow_manifest_drift: false,
        out: None,
        check_state: None,
        resume: None,
        resume_reply: None,
        walkthrough: None,
        walk_validate: None,
        walk_render: None,
    };
    let mut it = std::env::args().skip(1).peekable();
    while let Some(arg) = it.next() {
        let mut val = |name: &str| it.next().ok_or(format!("{name} needs a value"));
        match arg.as_str() {
            "--record" => a.record = Some(PathBuf::from(val("--record")?)),
            "--orgs" => {
                for pair in val("--orgs")?.split(',').filter(|p| !p.is_empty()) {
                    let (k, v) = pair
                        .split_once('=')
                        .ok_or(format!("--orgs wants alias=tenant, got {pair}"))?;
                    a.orgs.push((k.trim().into(), v.trim().into()));
                }
            }
            "--group" => {
                let g = val("--group")?;
                let (k, v) = g
                    .split_once('=')
                    .ok_or(format!("--group wants name=a+b, got {g}"))?;
                a.groups.push((
                    k.trim().into(),
                    v.split('+').map(|m| m.trim().to_string()).collect(),
                ));
            }
            "--offline" => a.offline = true,
            "--echo-keys" => a.echo_keys = true,
            "--check-theme" => a.check_theme = true,
            "--theme" => a.theme = Some(val("--theme")?),
            "--glyphs" => a.glyphs = Some(val("--glyphs")?),
            "--profile" => a.profile = Some(val("--profile")?),
            "--config" => a.config = Some(PathBuf::from(val("--config")?)),
            "--now" => {
                let v = val("--now")?;
                a.now = Some(clock::parse_utc(&v).ok_or(format!("--now: not a timestamp: {v}"))?);
            }
            "--no-mouse" => a.mouse = false,
            "--no-agent" => a.no_agent = true,
            "--label" => a.label = Some(val("--label")?),
            "--walkthrough" => a.walkthrough = Some(val("--walkthrough")?),
            "--walkthrough-validate" => a.walk_validate = Some(val("--walkthrough-validate")?),
            "--walkthrough-render" => a.walk_render = Some(val("--walkthrough-render")?),
            "--state" => a.state = Some(PathBuf::from(val("--state")?)),
            "--allow-record-drift" => a.allow_drift = true,
            "--allow-config-drift" => a.allow_config_drift = true,
            "--allow-classify-drift" => a.allow_classify_drift = true,
            "--allow-manifest-drift" => a.allow_manifest_drift = true,
            "--check-state" => a.check_state = Some(PathBuf::from(val("--check-state")?)),
            "--resume" => a.resume = Some(PathBuf::from(val("--resume")?)),
            "--resume-reply" => {
                let v = val("--resume-reply")?;
                a.resume_reply = Some(
                    v.parse()
                        .map_err(|_| format!("--resume-reply: not a descriptor: {v}"))?,
                );
            }
            "--out" => a.out = Some(PathBuf::from(val("--out")?)),
            "--keys" => {
                let fmt = match it.peek() {
                    Some(f) if !f.starts_with("--") => it.next().unwrap_or_default(),
                    _ => "markdown".into(),
                };
                a.keys = Some(fmt);
            }
            "--snapshot" => a.snapshot = Some(size(&val("--snapshot")?)?),
            "--script" => a.script = val("--script")?,
            "--only" => a.only = Some(val("--only")?),
            "--format" => {
                a.formats = val("--format")?
                    .split(',')
                    .map(|f| f.trim().to_string())
                    .filter(|f| !f.is_empty())
                    .collect();
                if let Some(bad) = a
                    .formats
                    .iter()
                    .find(|f| !["txt", "text", "html", "ansi", "svg", "png"].contains(&f.as_str()))
                {
                    return Err(format!(
                        "--format: unknown {bad} (txt, html, ansi; svg and png with --state)"
                    ));
                }
            }
            "--dump-states" => a.dump = Some(PathBuf::from(val("--dump-states")?)),
            "--sizes" => {
                a.sizes = val("--sizes")?
                    .split(',')
                    .map(size)
                    .collect::<Result<_, _>>()?
            }
            "-h" | "--help" => return Err(USAGE.into()),
            "-V" | "--version" => {
                println!("ownpurse-tui {}", ownpurse_tui::app::build_version());
                std::process::exit(0);
            }
            other => return Err(format!("unknown argument {other}\n{USAGE}")),
        }
    }
    Ok(a)
}

fn env(k: &str) -> Option<String> {
    std::env::var(k).ok()
}

fn theme_of(a: &Args) -> Result<Theme, String> {
    let mut t = match &a.theme {
        None => Theme::detect(&env),
        Some(n) if n == "auto" => Theme::detect(&env),
        Some(n) => Theme::named(n).ok_or(format!("unknown theme {n}"))?,
    };
    match a.glyphs.as_deref() {
        None | Some("auto") => {
            if a.theme.is_some() {
                t.ascii = !ownpurse_tui::glyphs::utf8_locale(&env);
            }
        }
        Some("ascii") => t.ascii = true,
        Some("unicode") => t.ascii = false,
        Some(g) => return Err(format!("unknown glyphs {g}")),
    }
    Ok(t)
}

/// Synchronized output: `OWNPURSE_SYNC=0|1` overrides; else the terminal's DECRQM 2026 answer; else
/// its identity (terminals without the mode ignore it anyway).
fn sync_supported(probed: Option<bool>) -> bool {
    if let Some(v) = env("OWNPURSE_SYNC") {
        return v == "1";
    }
    if let Some(p) = probed {
        return p;
    }
    let prog = env("TERM_PROGRAM").unwrap_or_default();
    let term = env("TERM").unwrap_or_default();
    matches!(
        prog.as_str(),
        "WezTerm" | "ghostty" | "iTerm.app" | "tmux" | "vscode"
    ) || term.contains("kitty")
        || term.contains("ghostty")
        || term.contains("wezterm")
}

static HANGUP: AtomicBool = AtomicBool::new(false);

extern "C" fn on_signal(_: libc::c_int) {
    HANGUP.store(true, Ordering::SeqCst);
}

/// SIGTERM / SIGHUP set a flag the loop checks; the loop restores the terminal on its way out.
fn catch_signals() {
    // SAFETY: the handler only stores to an atomic (async-signal-safe).
    unsafe {
        libc::signal(libc::SIGTERM, on_signal as *const () as libc::sighandler_t);
        libc::signal(libc::SIGHUP, on_signal as *const () as libc::sighandler_t);
    }
}

/// Set while an external program ($EDITOR) owns the terminal; the input thread stops reading.
static INPUT_PAUSED: AtomicBool = AtomicBool::new(false);

/// Effects outside the frame: $EDITOR (terminal suspended and restored), the browser, OSC 52 copy.
fn run_external<W: Write>(rt: &mut Runtime<Quiet<W>>, mouse: bool) -> std::io::Result<()> {
    for e in std::mem::take(&mut rt.external) {
        match e {
            Effect::Edit(path) => {
                let editor = ownpurse_tui::system::editor_command(&rt.app.source.sys.editor, &env);
                INPUT_PAUSED.store(true, Ordering::SeqCst);
                restore();
                let status = std::process::Command::new(&editor[0])
                    .args(&editor[1..])
                    .arg(&path)
                    .status();
                setup_modes(mouse, true)?;
                INPUT_PAUSED.store(false, Ordering::SeqCst);
                rt.terminal.clear()?;
                rt.app.ui.note = Some(match status {
                    Ok(s) if s.success() => "editor closed · restart to apply".into(),
                    Ok(s) => format!("{} exited with {s}", editor[0]),
                    Err(err) => format!("could not start {}: {err}", editor[0]),
                });
                rt.app.dirty = true;
            }
            Effect::Open(url) => {
                let cmd = ownpurse_tui::system::open_command(&rt.app.source.sys.open_command);
                if let Err(err) = std::process::Command::new(&cmd[0])
                    .args(&cmd[1..])
                    .arg(&url)
                    .stdin(std::process::Stdio::null())
                    .stdout(std::process::Stdio::null())
                    .stderr(std::process::Stdio::null())
                    .spawn()
                {
                    rt.app.ui.note = Some(format!("could not open {url} with {}: {err}", cmd[0]));
                    rt.app.dirty = true;
                }
            }
            Effect::Copy(text) => {
                let mut out = std::io::stdout();
                out.write_all(ownpurse_tui::system::osc52(&text).as_bytes())?;
                out.flush()?;
            }
            _ => {}
        }
    }
    Ok(())
}

static KITTY: AtomicBool = AtomicBool::new(false);

/// Every mode we set, released on every exit path (normal, signal, panic).
fn restore() {
    let mut out = std::io::stdout();
    let _ = out.write_all(b"\x1b[?2026l\x1b[?1006l\x1b[?1002l\x1b[?1000l");
    if KITTY.load(Ordering::SeqCst) {
        let _ = execute!(out, event::PopKeyboardEnhancementFlags);
    }
    let _ = execute!(
        out,
        event::DisableBracketedPaste,
        crossterm::cursor::Show,
        terminal::LeaveAlternateScreen
    );
    let _ = terminal::disable_raw_mode();
    let _ = out.flush();
}

/// Before exec (in-place upgrade): every mode the new process sets again, popped; raw mode left so the
/// new process saves the cooked settings as the ones to restore at exit. The alternate screen is KEPT (no
/// flash of the shell's screen; the new process draws over it and leaves it when it quits).
fn release_for_exec() -> std::io::Result<()> {
    let mut out = std::io::stdout();
    out.write_all(b"\x1b[?2026l\x1b[?1006l\x1b[?1002l\x1b[?1000l")?;
    if KITTY.swap(false, Ordering::SeqCst) {
        execute!(out, event::PopKeyboardEnhancementFlags)?;
    }
    execute!(out, event::DisableBracketedPaste)?;
    out.flush()?;
    terminal::disable_raw_mode()
}

fn setup(mouse: bool, alternate: bool) -> std::io::Result<()> {
    setup_modes(mouse, alternate)?;
    let prev = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        restore();
        ownpurse_tui::bridge::cleanup();
        prev(info);
    }));
    catch_signals();
    Ok(())
}

/// Raw mode, the alternate screen (unless already in it: a resumed upgrade), bracketed paste, keyboard
/// enhancement, mouse reporting.
fn setup_modes(mouse: bool, alternate: bool) -> std::io::Result<()> {
    terminal::enable_raw_mode()?;
    let mut out = std::io::stdout();
    if alternate {
        execute!(out, terminal::EnterAlternateScreen)?;
    }
    execute!(out, event::EnableBracketedPaste)?;
    if terminal::supports_keyboard_enhancement().unwrap_or(false) {
        execute!(
            out,
            event::PushKeyboardEnhancementFlags(
                event::KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES
            )
        )?;
        KITTY.store(true, Ordering::SeqCst);
    }
    if mouse {
        // press/release (1000), drag (1002), SGR coordinates (1006); no hover in the shell.
        out.write_all(b"\x1b[?1000h\x1b[?1002h\x1b[?1006h")?;
        out.flush()?;
    }
    Ok(())
}

fn make_app(a: &Args, theme: Theme, offline: bool) -> Result<App, String> {
    let source = match &a.record {
        // --record + --orgs: no config (no pairs, no account_map, default limits)
        Some(record) => {
            if a.orgs.is_empty() {
                return Err(format!("--orgs is required with --record\n{USAGE}"));
            }
            let mut s = Source::bare(record.clone(), a.orgs.clone(), a.groups.clone());
            if let Some(p) = &a.profile {
                s.profile = p.clone();
            }
            s
        }
        // the config (ownpurse.json, found as the CLI finds it, or --config)
        None => {
            let env = ownpurse_core::config::Env::process();
            let cfg = ownpurse_core::config::load(&env, a.profile.as_deref(), a.config.as_deref())
                .map_err(|e| e.to_string())?;
            let mut s = Source::from_config(&cfg);
            // the CLI the app runs gets the same config when the TUI was given one
            s.config_arg = a
                .config
                .clone()
                .map(|p| std::fs::canonicalize(&p).unwrap_or(p));
            s.groups.extend(a.groups.iter().cloned());
            s
        }
    };
    let now = a.now.unwrap_or_else(clock::env_now);
    let model = Model::open(&source, now).map_err(|e| e.to_string())?;
    Ok(App::new(model, source, theme, offline, now))
}

/// Write one render in each format: `<name>.txt` / `.html` / `.ans`, in subdirs when several formats.
fn write_render(
    dir: &std::path::Path,
    formats: &[String],
    name: &str,
    theme: &Theme,
    buf: &ratatui::buffer::Buffer,
) -> Result<(), String> {
    for f in formats {
        let sub = if formats.len() == 1 && f == "txt" {
            dir.to_path_buf()
        } else {
            dir.join(f)
        };
        std::fs::create_dir_all(&sub).map_err(|e| e.to_string())?;
        let (ext, body) = match f.as_str() {
            "html" => ("html", ownpurse_tui::render::html(buf, theme, name)),
            "ansi" => ("ans", ownpurse_tui::render::ansi(buf)),
            _ => ("txt", snapshot::text(buf)),
        };
        std::fs::write(sub.join(format!("{name}.{ext}")), body).map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// `--echo-keys`: the keys this terminal sends, as keys.toml spells them (`cmd+[`), one per line, with keyboard
/// enhancement on when the terminal supports it. Nothing is read or written but the terminal.
fn echo_keys() -> Result<(), String> {
    let e = |x: std::io::Error| x.to_string();
    terminal::enable_raw_mode().map_err(e)?;
    let mut out = std::io::stdout();
    let enhanced = terminal::supports_keyboard_enhancement().unwrap_or(false);
    if enhanced {
        execute!(
            out,
            event::PushKeyboardEnhancementFlags(
                event::KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES
            )
        )
        .map_err(e)?;
    }
    let say = |out: &mut std::io::Stdout, s: &str| {
        let _ = write!(out, "{s}\r\n");
        let _ = out.flush();
    };
    say(
        &mut out,
        &format!(
            "keyboard enhancement: {} · press keys (try ⌘[ and ⌘]); q twice quits",
            if enhanced {
                "on (⌘ is reported)"
            } else {
                "not supported (⌘ keys can't reach a TUI here)"
            }
        ),
    );
    let mut last_q = false;
    let res = loop {
        match event::read() {
            Ok(Event::Key(k)) if k.kind != event::KeyEventKind::Release => {
                let key = keymap::Key::of(&k);
                say(
                    &mut out,
                    &format!("{}    ({:?} {:?})", key.spec(), k.code, k.modifiers),
                );
                let q = key == keymap::Key::parse("q").expect("q");
                if q && last_q {
                    break Ok(());
                }
                last_q = q;
            }
            Ok(_) => {}
            Err(x) => break Err(x.to_string()),
        }
    };
    if enhanced {
        let _ = execute!(out, event::PopKeyboardEnhancementFlags);
    }
    let _ = terminal::disable_raw_mode();
    res
}

fn run(a: Args) -> Result<(), String> {
    if a.echo_keys {
        return echo_keys();
    }
    if let Some(fmt) = &a.keys {
        // the EFFECTIVE keymap: the built-in table with the user's keys.toml (read, never written)
        let km = match user_keys_dir() {
            Some(dir) => {
                let mut l = user_config_loader(dir);
                let mut km = keymap::default_map().clone();
                for ev in l.reload() {
                    match ev {
                        ownpurse_tui::user_config::ConfigEvent::Reloaded(c, _) => {
                            if let Ok(k) = keymap::Keymap::build(&c.keys) {
                                km = k;
                            }
                        }
                        ownpurse_tui::user_config::ConfigEvent::Invalid(i) => {
                            eprintln!("warning: {i} (the built-in keymap is shown)")
                        }
                    }
                }
                km
            }
            None => keymap::default_map().clone(),
        };
        match fmt.as_str() {
            "markdown" => print!("{}", keymap::to_markdown_in(&km)),
            "json" => print!("{}", keymap::to_json_in(&km)),
            "conflicts" => {
                let c: Vec<String> = keymap::conflicts_in(&km)
                    .into_iter()
                    .map(|(_, m)| m)
                    .collect();
                for line in &c {
                    println!("{line}");
                }
                if !c.is_empty() {
                    return Err(format!("{} conflicts", c.len()));
                }
            }
            other => {
                return Err(format!(
                    "--keys: unknown format {other} (markdown, json, conflicts)"
                ));
            }
        }
        return Ok(());
    }
    let theme = theme_of(&a)?;
    if let Some(dir) = &a.dump {
        std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
        let make = |t: Theme, offline: bool| make_app(&a, t, offline || a.offline).expect("app");
        make_app(&a, theme, a.offline)?; // fail early with a message
        let mut n = 0;
        let only = a.only.clone().unwrap_or_default();
        for st in snapshot::shell_states()
            .into_iter()
            .filter(|s| only.split(',').any(|p| s.name.starts_with(p)))
        {
            for &(w, h) in &a.sizes {
                let sizes: &[(u16, u16)] = st.sizes;
                let size = if st.name == "too-small" {
                    sizes[0]
                } else {
                    (w, h)
                };
                if st.name != "too-small" && !sizes.contains(&(w, h)) {
                    continue;
                }
                if !st.renders(theme.name(), size) {
                    continue;
                }
                let mut app = snapshot::state_app_with(&st, &make, theme, size.0, size.1);
                // captures carry the real version (state setups may swap in a pinned test app)
                app.version_label = ownpurse_tui::app::VERSION.to_string();
                let buf = snapshot::render(&mut app, size.0, size.1);
                let name = format!("{}-{}x{}-{}", st.name, size.0, size.1, theme.name());
                write_render(dir, &a.formats, &name, &theme, &buf)?;
                n += 1;
                if st.name == "too-small" {
                    break;
                }
            }
        }
        // the Activity and System states (demo fixture variants)
        for st in snapshot::tab_states()
            .into_iter()
            .filter(|s| only.split(',').any(|p| s.name.starts_with(p)))
        {
            for &(w, h) in a.sizes.iter().filter(|s| st.sizes.contains(s)) {
                let mut app = snapshot::tab_state_app(&st, theme, w, h);
                let buf = snapshot::render(&mut app, w, h);
                let name = format!("{}-{w}x{h}-{}", st.name, theme.name());
                write_render(dir, &a.formats, &name, &theme, &buf)?;
                n += 1;
            }
        }
        println!("{n} snapshots in {}", dir.display());
        return Ok(());
    }
    let mut app = make_app(&a, theme, a.offline)?;
    if let Some(l) = a.label.clone().filter(|l| !l.trim().is_empty()) {
        app.label = Some(l);
    }
    if let Some(path) = &a.state {
        return render_state(&a, app.source.clone(), path);
    }
    if a.walk_validate.is_some() || a.walk_render.is_some() {
        return walkthrough_cli(&a, theme, &app);
    }
    if let Some(id) = &a.walkthrough {
        let dir = app
            .source
            .walkthroughs
            .clone()
            .ok_or("no walkthroughs folder (it lives beside ownpurse.json)")?;
        let w = ownpurse_core::walkthrough::read(&dir, id)?;
        let mut fx = Vec::new();
        app.walk_start(w, &mut fx)?;
    }
    // the upgrade pre-flight (run by the OLD build, headless): 0 = this build restores the view exactly
    if a.check_theme {
        return check_theme(app);
    }
    if let Some(path) = &a.check_state {
        // the "can I start" dry run: the user config folder (keys, themes) loads as at start-up (a bad file
        // is reported, never fatal; a panic would fail the pre-flight with the old process still running)
        if let Some(dir) = user_keys_dir() {
            let _ = user_config_loader(dir).reload();
        }
        let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
        match ownpurse_tui::upgrade::check(&text, app.source.clone()) {
            Ok(None) => {
                println!("ok: the state restores the live frame exactly");
                return Ok(());
            }
            Ok(Some((from, to))) => {
                println!(
                    "ok: render_epoch {} ({from} → {to}): the state is kept and this build renders it consistently",
                    ownpurse_tui::upgrade::EPOCH_CHANGED
                );
                return Ok(());
            }
            Err(why) => {
                println!("error: preflight_failed\nmessage: {why}");
                std::process::exit(1);
            }
        }
    }
    if let Some((w, h)) = a.snapshot {
        snapshot::run_script(&mut app, &a.script, w, h);
        let buf = snapshot::render(&mut app, w, h);
        print!("{}", snapshot::text(&buf));
        return Ok(());
    }
    // an upgraded session: the exported view, restored exactly (the file goes as soon as it's read)
    let resuming = a.resume.is_some();
    let mut resumed: Option<Result<String, String>> = None;
    let mut resumed_epoch: Option<(u32, u32)> = None;
    if let Some(path) = &a.resume {
        let text = std::fs::read_to_string(path);
        resumed_epoch = text
            .as_ref()
            .ok()
            .and_then(|t| ownpurse_tui::upgrade::epoch_change(t));
        let _ = std::fs::remove_file(path);
        let r = text
            .map_err(|e| format!("could not read the saved view: {e}"))
            .and_then(|t| {
                ownpurse_tui::upgrade::resume(&t, app.source.clone(), &app)
                    .map_err(|e| format!("{}: {}", e.code(), e.message()))
            });
        resumed = Some(match r {
            Ok((restored, from)) => {
                app = restored;
                Ok(from)
            }
            Err(why) => {
                // never a silent reset: a fresh view that says so
                app.ui.note = Some(format!(
                    "updated to v{} · the view could not be restored ({why})",
                    app.version_label
                ));
                Err(why)
            }
        });
    }
    // once ever (a marker under $XDG_STATE_HOME/ownpurse/), not per session
    if !resuming && ownpurse_tui::probe::terminal_note_due(&env, &|p| p.exists()) {
        app.ui.note = Some(
            "Terminal.app can tear on big redraws · Ghostty, iTerm2, WezTerm or kitty recommended"
                .into(),
        );
        ownpurse_tui::probe::mark_terminal_note(&env);
    }
    let pinned = a.now;
    // the same clock as at start: --now, else OWNPURSE_NOW, else the system clock
    let now = move || pinned.unwrap_or_else(clock::env_now);
    // a resumed session is already in the alternate screen (the old build kept it)
    setup(a.mouse, !resuming).map_err(|e| e.to_string())?;
    // OSC 11 (background) and DECRQM 2026 (synchronized output), in raw mode before the input thread
    // starts; no answer within the timeout keeps the environment-based guesses. Not on resume: the theme
    // comes from the state and synchronized output from the old build (keys typed meanwhile stay unread).
    #[cfg(unix)]
    let probed = if resuming {
        ownpurse_tui::probe::Probe::default()
    } else {
        ownpurse_tui::probe::run(
            &mut std::io::stdout(),
            &mut ownpurse_tui::probe::stdin_reader(),
            ownpurse_tui::probe::TIMEOUT,
        )
    };
    #[cfg(not(unix))]
    let probed = ownpurse_tui::probe::Probe::default();
    let auto_theme = a.theme.as_deref().is_none_or(|t| t == "auto");
    if !resuming
        && auto_theme
        && env("COLORFGBG").is_none()
        && matches!(
            app.ui.theme.mode,
            ownpurse_tui::theme::Mode::Truecolor | ownpurse_tui::theme::Mode::Indexed
        )
        && let Some(dark) = probed.dark
    {
        app.ui.theme.dark = dark;
    }
    let sync = sync_supported(probed.sync);
    let result = (|| -> std::io::Result<()> {
        let terminal = Terminal::new(Quiet::new(std::io::stdout(), sync))?;
        let mut rt = Runtime::new(terminal, app);
        // update detection: this binary on disk (a reinstall replaces it while we run)
        let exe = std::env::current_exe().ok();
        rt.update_probe = exe.clone();
        rt.check_update();
        rt.update_reads = 0;
        // one channel for terminal input and the running child's events (spec P6 §2)
        let (tx, rx) = mpsc::channel::<LoopEvent>();
        let run_tx = tx.clone();
        rt.spawner = Some(Box::new(move |program, args| {
            let tx = std::sync::Mutex::new(run_tx.clone());
            ownpurse_tui::net::spawn(program, args, move |ev| {
                if let Ok(t) = tx.lock() {
                    let _ = t.send(LoopEvent::Run(ev));
                }
            })
        }));
        // the user's config folder (keys.toml, themes/): loaded now, then ONE watcher feeding the loop
        rt.app.cmd_keys = KITTY.load(Ordering::SeqCst);
        let user_tx = std::sync::Mutex::new(tx.clone());
        let keys_file = rt.app.source.keys_file.clone();
        let _user_watch = user_keys_dir().and_then(|dir| {
            let mut loader = user_config_loader(dir);
            if let Some((path, shown)) = keys_file {
                loader = loader.with_keys_file(path, shown);
            }
            for ev in loader.reload() {
                rt.app.user_config_event(ev);
            }
            rt.app.cmd_notice();
            ownpurse_tui::user_config::UserConfigWatcher::spawn(loader, move |ev| {
                if let Ok(t) = user_tx.lock() {
                    let _ = t.send(LoopEvent::UserConfig(ev));
                }
            })
        });
        // classify.toml hand edits: OS file events through the same channel (no timer; V13)
        let watch_tx = std::sync::Mutex::new(tx.clone());
        let _classify_watch = rt
            .app
            .source
            .classify
            .as_ref()
            .filter(|_| rt.app.classify.is_some())
            .and_then(|p| {
                ownpurse_tui::classify::watch(&p.config, move || {
                    if let Ok(t) = watch_tx.lock() {
                        let _ = t.send(LoopEvent::ClassifyConfig);
                    }
                })
            });
        // other processes' writes to the record and the classification store (a CLI sync, `classify
        // suggest`): OS file events on their folders, through the same channel (no timer; V13)
        let data_tx = std::sync::Mutex::new(tx.clone());
        let mut data_files = vec![rt.app.source.record.clone()];
        if rt.app.classify.is_some()
            && let Some(c) = rt.app.source.classify.as_ref()
        {
            data_files.push(c.store.clone());
        }
        // the manifest store (an agent's or the CLI's `manifest …` writes show at once)
        if let Some(m) = rt.app.source.manifests.as_ref() {
            data_files.push(m.store.clone());
        }
        let _data_watch = ownpurse_tui::external::watch(&data_files, move || {
            if let Ok(t) = data_tx.lock() {
                let _ = t.send(LoopEvent::DataChanged);
            }
        });
        // the project config (ownpurse.json): saves are re-read live (the same OS events; no timer)
        let cfg_tx = std::sync::Mutex::new(tx.clone());
        let _config_watch = rt.app.source.config_file.clone().and_then(|f| {
            ownpurse_tui::external::watch(&[f], move || {
                if let Ok(t) = cfg_tx.lock() {
                    let _ = t.send(LoopEvent::ProjectConfig);
                }
            })
        });
        // the agent bridge (v0.10): the TUI is the server; requests come through the same channel
        let agent_on = ownpurse_tui::bridge::wanted(a.no_agent, &rt.app.source);
        let profile = rt.app.profile.clone();
        let agent_tx = tx.clone();
        let start_bridge = move || -> Result<ownpurse_tui::bridge::Server, String> {
            let agent_tx = agent_tx.clone();
            ownpurse_tui::bridge::Server::start(
                ownpurse_core::bridge::Dirs::process(),
                &profile,
                Box::new(|| INPUT_PAUSED.load(Ordering::SeqCst)),
                Box::new(move |r| agent_tx.send(LoopEvent::Agent(r)).is_ok()),
            )
            .map_err(|e| e.to_string())
        };
        let mut bridge = if agent_on {
            match start_bridge() {
                Ok(s) => Some(s),
                Err(e) => {
                    rt.app.ui.note = Some(format!("agent bridge off: {e}"));
                    None
                }
            }
        } else {
            None
        };
        if let Some(b) = &bridge {
            b.sync(&rt.app); // label, scope, tab, theme from the start
        }
        // the agent that asked for the upgrade hears how it went, on the connection the old build passed on
        if let Some(fd) = a.resume_reply.filter(|fd| *fd > 2) {
            use std::os::fd::FromRawFd;
            // SAFETY: the old build cleared FD_CLOEXEC on this connected socket and passed its number; it
            // is ours alone now and closed when the stream drops.
            let mut conn = unsafe { std::os::unix::net::UnixStream::from_raw_fd(fd) };
            let doc = match &resumed {
                Some(Ok(from)) => ownpurse_tui::upgrade::upgraded_doc(
                    from,
                    &ownpurse_tui::app::full_version(
                        &rt.app.version_label,
                        rt.app.build_label.as_deref(),
                    ),
                    rt.app.rev,
                    resumed_epoch,
                ),
                Some(Err(why)) => ownpurse_core::bridge::error_doc(
                    "resume_failed",
                    1,
                    format!(
                        "Upgraded to v{}, but the view could not be restored ({why}). The user sees a fresh view that says so.",
                        rt.app.version_label
                    ),
                    &[],
                ),
                None => serde_json::Value::Null,
            };
            let mut line = ownpurse_core::canon::canonical(&doc);
            line.push('\n');
            let _ = conn.write_all(line.as_bytes());
        }
        std::thread::spawn(move || {
            loop {
                // paused while $EDITOR owns the terminal (or an upgrade hands over), so its keys never
                // reach the TUI
                if INPUT_PAUSED.load(Ordering::SeqCst) {
                    std::thread::sleep(Duration::from_millis(20));
                    continue;
                }
                match event::poll(Duration::from_millis(50)) {
                    Ok(true) => match event::read() {
                        Ok(ev) => {
                            if tx.send(LoopEvent::Term(ev)).is_err() {
                                break;
                            }
                        }
                        Err(_) => break,
                    },
                    Ok(false) => {}
                    Err(_) => break,
                }
            }
        });
        let mut clicks = Clicks::default();
        // an agent's upgrade request waiting for the handover (answered by it, or by the new build)
        let mut asked: Option<ownpurse_tui::bridge::AgentRequest> = None;
        // `ui wait` requests in flight: answered after the event that makes them hold, or at their deadline
        let mut waits: Vec<ownpurse_tui::bridge::Wait> = Vec::new();
        let mut carry: std::collections::VecDeque<LoopEvent> = std::collections::VecDeque::new();
        let mut last_tick = std::time::Instant::now();
        loop {
            rt.frame()?;
            if rt.quit || HANGUP.load(Ordering::SeqCst) {
                return Ok(());
            }
            let now_i = std::time::Instant::now();
            waits.retain(|w| {
                let held = rt.wait_holds(&w.cond);
                if held || now_i >= w.deadline {
                    let _ = w.reply.send(rt.wait_answer(w, held));
                    false
                } else {
                    true
                }
            });
            // Wake at least every 500 ms for signals and the minute tick (sooner while a wait has a deadline);
            // idle writes nothing (V13).
            let wake = waits
                .iter()
                .map(|w| w.deadline.saturating_duration_since(now_i))
                .min()
                .map_or(Duration::from_millis(500), |d| {
                    d.min(Duration::from_millis(500))
                });
            // events held over from a reload's drain go first (no waiting while some are queued)
            let got = match carry.pop_front() {
                Some(ev) => Ok(ev),
                None => rx.recv_timeout(wake),
            };
            match got {
                Ok(first) => {
                    let mut pending = vec![first];
                    pending.extend(carry.drain(..));
                    pending.extend(rx.try_iter());
                    // a burst of file events (a sync writes many rows) is one check
                    let mut data_changed = false;
                    let mut config_changed = false;
                    for ev in pending {
                        match ev {
                            LoopEvent::Term(ev) => handle(&mut rt, &mut clicks, ev),
                            LoopEvent::Run(ev) => rt.send(Msg::Run(ev)),
                            LoopEvent::ClassifyConfig => rt.send(Msg::ClassifyConfigChanged),
                            LoopEvent::DataChanged => data_changed = true,
                            LoopEvent::ProjectConfig => config_changed = true,
                            LoopEvent::UserConfig(ev) => rt.send(Msg::UserConfig(ev)),
                            LoopEvent::Agent(r) if r.doc["op"] == "wait" => {
                                if let Some(w) = rt.wait_start(&r.doc, r.reply) {
                                    waits.push(w);
                                }
                            }
                            LoopEvent::Agent(r) => {
                                let answer = rt.agent(&r.doc);
                                if answer["status"] == "upgrading" {
                                    asked = Some(r);
                                } else {
                                    let _ = r.reply.send(answer);
                                }
                            }
                        }
                        if rt.quit {
                            break;
                        }
                    }
                    // an editor's save is several events: one re-read
                    if config_changed && !rt.quit {
                        rt.config_check();
                    }
                    if data_changed && !rt.quit {
                        rt.external_check();
                        // a sync elsewhere writes in bursts: what arrived during that reload is coalesced into
                        // one more check now (no timer); other events wait for the next pass, in order
                        let mut again = false;
                        for ev in rx.try_iter() {
                            match ev {
                                LoopEvent::DataChanged => again = true,
                                LoopEvent::ProjectConfig => rt.config_check(),
                                other => carry.push_back(other),
                            }
                        }
                        if again {
                            rt.external_check();
                        }
                    }
                    run_external(&mut rt, a.mouse)?;
                    if rt.upgrade_due {
                        rt.upgrade_due = false;
                        let from = ownpurse_tui::app::full_version(
                            &rt.app.version_label,
                            rt.app.build_label.as_deref(),
                        );
                        let to = rt.upgradable().unwrap_or_default();
                        let ask = asked.take();
                        let outcome = upgrade_now(
                            &mut rt,
                            exe.clone(),
                            ask.as_ref().and_then(|r| r.conn.as_ref()),
                            &mut bridge,
                            &start_bridge,
                            a.mouse,
                            sync,
                        );
                        if let Some(r) = ask {
                            let _ = r.reply.send(outcome.doc(&from, &to));
                        }
                    }
                    if let Some(b) = &bridge {
                        b.sync(&rt.app);
                    }
                }
                // a wait's deadline wakes the loop early: the tick still comes every 500 ms
                Err(mpsc::RecvTimeoutError::Timeout)
                    if last_tick.elapsed() < Duration::from_millis(500) => {}
                Err(mpsc::RecvTimeoutError::Timeout) => {
                    last_tick = std::time::Instant::now();
                    rt.send(Msg::Tick(now()))
                }
                Err(mpsc::RecvTimeoutError::Disconnected) => return Ok(()),
            }
        }
    })();
    restore();
    result.map_err(|e| e.to_string())
}

/// The terminal side of an in-place upgrade (`upgrade::Handover`).
struct TermHandover<'a> {
    bridge: &'a mut Option<ownpurse_tui::bridge::Server>,
    start_bridge: &'a dyn Fn() -> Result<ownpurse_tui::bridge::Server, String>,
    mouse: bool,
    had_bridge: bool,
}

impl ownpurse_tui::upgrade::Handover for TermHandover<'_> {
    fn release(&mut self) -> std::io::Result<()> {
        release_for_exec()?;
        // dropping the server removes the socket file and the registry entry (the new build registers again)
        self.had_bridge = self.bridge.take().is_some();
        Ok(())
    }

    fn exec(
        &mut self,
        program: &std::path::Path,
        args: &[String],
        env: &[(String, String)],
    ) -> std::io::Error {
        use std::os::unix::process::CommandExt;
        std::process::Command::new(program)
            .args(args)
            .envs(env.iter().map(|(k, v)| (k.as_str(), v.as_str())))
            .exec()
    }

    fn reenter(&mut self) -> std::io::Result<()> {
        setup_modes(self.mouse, false)?;
        if self.had_bridge && self.bridge.is_none() {
            match (self.start_bridge)() {
                Ok(s) => *self.bridge = Some(s),
                Err(e) => return Err(std::io::Error::other(format!("agent bridge: {e}"))),
            }
        }
        Ok(())
    }
}

/// Hand over to the newer installed build now (`upgrade::handover`); returns only when it didn't happen.
fn upgrade_now(
    rt: &mut Runtime<Quiet<std::io::Stdout>>,
    exe: Option<PathBuf>,
    conn: Option<&std::os::unix::net::UnixStream>,
    bridge: &mut Option<ownpurse_tui::bridge::Server>,
    start_bridge: &dyn Fn() -> Result<ownpurse_tui::bridge::Server, String>,
    mouse: bool,
    sync: bool,
) -> ownpurse_tui::upgrade::Outcome {
    use ownpurse_tui::upgrade::{self, Outcome};
    use std::os::fd::AsRawFd;
    let Some(program) = exe else {
        let o = Outcome::ExecFailed("this binary's path is unknown".into());
        rt.app.ui.note = Some(upgrade::failed_note(&o, &rt.app.version_label, ""));
        rt.app.dirty = true;
        return o;
    };
    // keys typed from here on stay in the terminal for whichever build reads next
    INPUT_PAUSED.store(true, Ordering::SeqCst);
    // the agent's connection survives exec: the new build answers on it
    let cloexec = |fd: i32, on: bool| {
        // SAFETY: fcntl on a descriptor we own; only the close-on-exec flag changes.
        unsafe {
            let flags = libc::fcntl(fd, libc::F_GETFD);
            if flags >= 0 {
                let next = if on {
                    flags | libc::FD_CLOEXEC
                } else {
                    flags & !libc::FD_CLOEXEC
                };
                libc::fcntl(fd, libc::F_SETFD, next);
            }
        }
    };
    let reply_fd = conn.map(|c| c.as_raw_fd());
    if let Some(fd) = reply_fd {
        cloexec(fd, false);
    }
    let mut env = vec![(
        "OWNPURSE_SYNC".to_string(),
        if sync { "1" } else { "0" }.to_string(),
    )];
    if let Some(b) = bridge.as_ref() {
        env.push((upgrade::STARTED_ENV.to_string(), b.started()));
    }
    // the new build names where it came from (a different build of the same version shows the builds)
    env.push((
        upgrade::FROM_ENV.to_string(),
        ownpurse_tui::app::full_version(&rt.app.version_label, rt.app.build_label.as_deref()),
    ));
    let plan = upgrade::Plan {
        program,
        args: upgrade::base_args(std::env::args().skip(1)),
        dir: ownpurse_core::bridge::Dirs::process().sockets,
        reply_fd,
        env,
        preflight_timeout: upgrade::PREFLIGHT_TIMEOUT,
    };
    let mut h = TermHandover {
        bridge,
        start_bridge,
        mouse,
        had_bridge: false,
    };
    let o = upgrade::handover(rt, &plan, &mut h);
    if let Some(fd) = reply_fd {
        cloexec(fd, true);
    }
    if matches!(o, Outcome::ExecFailed(_)) {
        let _ = rt.terminal.clear();
    }
    rt.app.dirty = true;
    INPUT_PAUSED.store(false, Ordering::SeqCst);
    o
}

/// What reaches the loop: terminal input, or a running child's event.
enum LoopEvent {
    Term(Event),
    Run(ownpurse_tui::net::RunEvent),
    /// An agent bridge request (answered by the loop through its reply channel).
    Agent(ownpurse_tui::bridge::AgentRequest),
    /// `classify.toml` changed on disk (the file watcher).
    ClassifyConfig,
    /// The record or the classification store was written (by anyone; the check compares revisions).
    DataChanged,
    /// `ownpurse.json` was saved.
    ProjectConfig,
    /// The user config folder changed (keys.toml, themes/*.toml).
    UserConfig(ownpurse_tui::user_config::ConfigEvent),
}

/// The user config folder: `OWNPURSE_CONFIG_DIR` (tests), else `$XDG_CONFIG_HOME/ownpurse`, else
/// `~/.config/ownpurse`.
fn user_keys_dir() -> Option<std::path::PathBuf> {
    std::env::var_os("OWNPURSE_CONFIG_DIR")
        .map(std::path::PathBuf::from)
        .or_else(ownpurse_tui::user_config::process_config_dir)
}

/// The loader with the keymap's validation (themes: built-ins only until the theme layer plugs in).
fn user_config_loader(dir: std::path::PathBuf) -> ownpurse_tui::user_config::Loader {
    ownpurse_tui::user_config::Loader::new(
        dir,
        Some(|k| keymap::Keymap::build(k).map(|_| ())),
        Some(ownpurse_tui::theme::user::validate),
    )
}

/// `--state FILE --snapshot WxH [--format text|ansi|html|svg|png] [--out FILE]`: the frame of a canonical
/// state, from the state and the record alone (no terminal, no socket). Another record version is refused
/// (`record_changed`, exit 4) unless `--allow-record-drift`; another config (beyond a reorder of orgs or
/// groups) is refused (`config_changed`, exit 4) unless `--allow-config-drift`.
fn render_state(a: &Args, source: Source, path: &std::path::Path) -> Result<(), String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let allow = ownpurse_tui::canonical::AllowDrift {
        record: a.allow_drift,
        config: a.allow_config_drift,
        classify: a.allow_classify_drift,
        manifest: a.allow_manifest_drift,
        live: false,
    };
    let mut app = match App::from_state_json_with(&text, source, allow) {
        Ok(app) => app,
        Err(e) => {
            println!("error: {}\nmessage: {}", e.code(), e.message());
            if e.code() == "record_changed" {
                println!(
                    "help[1]:\n  Pass --allow-record-drift to render against the current record (marked as drifted)."
                );
            }
            if e.code() == "config_changed" {
                println!(
                    "help[1]:\n  Pass --allow-config-drift to render under the current config (marked as drifted)."
                );
            }
            if e.code() == "classify_changed" {
                println!(
                    "help[1]:\n  Pass --allow-classify-drift to render with the current labels (marked as drifted)."
                );
            }
            std::process::exit(e.exit());
        }
    };
    let (w, h) = match a.snapshot {
        Some((w, h)) => (w, h),
        None => app.ui.size,
    };
    if (w, h) != app.ui.size {
        return Err(format!(
            "--snapshot {w}x{h} differs from the state's size {}x{}: a state renders at its own size",
            app.ui.size.0, app.ui.size.1
        ));
    }
    let buf = snapshot::render(&mut app, w, h);
    let th = app.ui.theme;
    let format = a.formats.first().map(String::as_str).unwrap_or("txt");
    let bytes: Vec<u8> = match format {
        "txt" | "text" => snapshot::text(&buf).into_bytes(),
        "ansi" => ownpurse_tui::render::ansi(&buf).into_bytes(),
        "html" => ownpurse_tui::render::html(&buf, &th, "ownpurse").into_bytes(),
        "svg" => ownpurse_tui::render::svg(&buf, &th).into_bytes(),
        "png" => ownpurse_core::raster::png(&ownpurse_tui::render::grid(&buf, &th))?,
        other => return Err(format!("--format {other}: text, ansi, html, svg or png")),
    };
    match &a.out {
        Some(p) => std::fs::write(p, &bytes).map_err(|e| format!("{}: {e}", p.display()))?,
        None => {
            std::io::stdout()
                .write_all(&bytes)
                .map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

/// `--walkthrough-validate` / `--walkthrough-render` (the CLI's `walkthrough validate|render` run these).
fn walkthrough_cli(a: &Args, theme: Theme, app: &App) -> Result<(), String> {
    let dir = app
        .source
        .walkthroughs
        .clone()
        .ok_or("no walkthroughs folder (it lives beside ownpurse.json)")?;
    let make = || {
        let mut x = make_app(a, theme, a.offline).expect("app");
        x.paths = app.paths.clone();
        x
    };
    if let Some(id) = &a.walk_validate {
        let w = ownpurse_core::walkthrough::read(&dir, id)?;
        let found = ownpurse_tui::walk::validate(&make, &w);
        for f in &found {
            println!("{}", serde_json::to_string(f).unwrap_or_default());
        }
        if !found.is_empty() {
            std::process::exit(4);
        }
        return Ok(());
    }
    if let Some(id) = &a.walk_render {
        let w = ownpurse_core::walkthrough::read(&dir, id)?;
        let out = a
            .out
            .clone()
            .ok_or("--walkthrough-render needs --out DIR")?;
        let mut n = 0;
        for &size in &a.sizes {
            for (name, buf) in ownpurse_tui::walk::render_steps(&make, &w, size)? {
                for f in &a.formats {
                    let sub = out.join(format!("{}x{}", size.0, size.1));
                    std::fs::create_dir_all(&sub).map_err(|e| e.to_string())?;
                    let (ext, bytes): (&str, Vec<u8>) = match f.as_str() {
                        "txt" | "text" => ("txt", snapshot::text(&buf).into_bytes()),
                        "svg" => ("svg", ownpurse_tui::render::svg(&buf, &theme).into_bytes()),
                        "png" => (
                            "png",
                            ownpurse_core::raster::png(&ownpurse_tui::render::grid(&buf, &theme))?,
                        ),
                        "html" => (
                            "html",
                            ownpurse_tui::render::html(&buf, &theme, &name).into_bytes(),
                        ),
                        other => return Err(format!("--format {other}: txt, svg, png or html")),
                    };
                    std::fs::write(sub.join(format!("{name}.{ext}")), bytes)
                        .map_err(|e| e.to_string())?;
                    n += 1;
                }
            }
        }
        println!("{n} frames in {}", out.display());
    }
    Ok(())
}

fn handle<W: Write>(rt: &mut Runtime<Quiet<W>>, clicks: &mut Clicks, ev: Event) {
    match ev {
        Event::Key(k) if k.kind == KeyEventKind::Press => rt.send(Msg::Key(k)),
        Event::Mouse(m) => match m.kind {
            MouseEventKind::Down(MouseButton::Left) => {
                let n = clicks.press(m.column, m.row);
                rt.send(Msg::Mouse { ev: m, clicks: n });
            }
            MouseEventKind::ScrollUp | MouseEventKind::ScrollDown => {
                rt.send(Msg::Mouse { ev: m, clicks: 0 })
            }
            _ => {}
        },
        Event::Resize(w, h) => rt.send(Msg::Resize(w, h)),
        // Bracketed paste goes into the focused prompt or finder only.
        Event::Paste(text) if keymap::types_text(rt.app.stack()[0]) => {
            for c in text.chars().filter(|c| !c.is_control()) {
                rt.send(Msg::Key(crossterm::event::KeyEvent::new(
                    crossterm::event::KeyCode::Char(c),
                    crossterm::event::KeyModifiers::NONE,
                )));
            }
        }
        _ => {}
    }
}

fn main() {
    let args = match parse_args() {
        Ok(a) => a,
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(2);
        }
    };
    if let Err(e) = run(args) {
        eprintln!("ownpurse-tui: {e}");
        std::process::exit(1);
    }
}

/// `--check-theme` (theme spec v2 §2.6, §3.2): the theme in use (the user's themes folder read, never
/// written), the intensities over their computed ceilings, settings problems, and text under 4.5:1 on any
/// surface in either darkness. Warnings, not errors: exit 0.
fn check_theme(mut app: ownpurse_tui::app::App) -> Result<(), String> {
    use ownpurse_tui::theme::def::Knobs;
    let mut warnings: Vec<String> = Vec::new();
    if let Some(dir) = user_keys_dir() {
        for ev in user_config_loader(dir).reload() {
            if let ownpurse_tui::user_config::ConfigEvent::Invalid(i) = &ev {
                warnings.push(format!("{i} (the previous theme stays live)"));
            }
            app.user_config_event(ev);
        }
    }
    warnings.extend(app.source.theme_issues.iter().cloned());
    for (i, dark) in [(0usize, true), (1, false)] {
        for k in app.source.knobs[i].over_ceiling(dark) {
            let c = Knobs::ceilings(dark);
            warnings.push(format!(
                "tui.accent_intensity.{k} ({}) is over its ceiling {} on {}: applied, text may read below 4.5:1",
                app.source.knobs[i].text(k),
                c.text(k),
                if dark { "dark" } else { "light" }
            ));
        }
    }
    let name = app
        .ui
        .theme_user
        .as_ref()
        .map_or_else(|| app.ui.theme.name().to_string(), |(n, _)| n.clone());
    for dark in [true, false] {
        app.ui.theme.dark = dark;
        if app.ui.theme_user.is_none() {
            app.ui.theme.mode = ownpurse_tui::theme::Mode::Truecolor;
        }
        app.repaint();
        for w in app.contrast_warnings() {
            warnings.push(format!(
                "{} ({}): {w}",
                name,
                if dark { "dark" } else { "light" }
            ));
        }
    }
    println!("theme: {name}");
    if warnings.is_empty() {
        println!("ok: no warnings");
    } else {
        for w in &warnings {
            println!("warning: {w}");
        }
    }
    Ok(())
}
