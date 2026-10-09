//! The live checks.

use super::world::{Bins, World};
use serde_json::Value;

/// `xtask agent-drive explore`: start the TUI and print the screen and state (development aid).
pub fn explore(bins: Bins) -> Result<(), String> {
    if std::env::var_os("XAD_FS").is_some() {
        let mut w = World::new("fs", bins)?;
        w.start(100, 30, &[])?;
        let canon = w.cli(&["--json", "ui", "state"]).stdout;
        let p = w.root.join("s.json");
        std::fs::write(&p, canon).map_err(|e| e.to_string())?;
        let o = w.cli(&[
            "--json",
            "ui",
            "screen",
            "--from-state",
            &p.display().to_string(),
            "--format",
            "text",
        ]);
        println!(
            "code {} out {} err {}",
            o.code,
            &o.stdout[..o.stdout.len().min(900)],
            o.stderr
        );
        return Ok(());
    }
    let mut w = World::new("explore", bins)?;
    w.start(100, 30, &[])?;
    println!("{}", w.screen().text());
    println!("{}", w.cli(&["--json", "ui", "state"]).stdout);
    Ok(())
}

pub fn trim_lines(s: &str) -> Vec<String> {
    let mut v: Vec<String> = s.lines().map(|l| l.trim_end().to_string()).collect();
    while v.last().is_some_and(|l| l.is_empty()) {
        v.pop();
    }
    v
}

/// The first differing line between two screens, for a failure message.
pub fn first_diff(a: &[String], b: &[String]) -> Option<String> {
    for i in 0..a.len().max(b.len()) {
        let x = a.get(i).map(String::as_str).unwrap_or("<none>");
        let y = b.get(i).map(String::as_str).unwrap_or("<none>");
        if x != y {
            return Some(format!("line {i}:\n  terminal: {x:?}\n  model:    {y:?}"));
        }
    }
    None
}

/// The collected failures of a run.
#[derive(Default)]
pub struct Report {
    pub steps: usize,
    pub failures: Vec<String>,
}

impl Report {
    pub fn fail(&mut self, step: &str, why: impl Into<String>) {
        self.failures.push(format!("[{step}] {}", why.into()));
    }
    pub fn check(&mut self, ok: bool, step: &str, why: impl FnOnce() -> String) {
        if !ok {
            self.fail(step, why());
        }
    }
}

/// The display texts of a row (its field values as shown), for finding it on the terminal.
fn row_texts(row: &Value) -> Vec<String> {
    let mut out = vec![];
    if let Some(f) = row.get("fields").and_then(Value::as_object) {
        for (k, v) in f {
            if k.ends_with("_value") || k.ends_with("_estimated") {
                continue;
            }
            if let Value::String(s) = v {
                let t = s.trim().trim_end_matches('…').trim();
                if t.chars().count() >= 2 {
                    // a long value may be cut on screen: its first 8 characters are enough
                    out.push(t.chars().take(8).collect());
                }
            }
        }
    }
    out
}

fn score(line: &str, texts: &[String]) -> usize {
    texts.iter().filter(|t| line.contains(t.as_str())).count()
}

/// `score` on the part left of the first `│`: a detail pane beside a list repeats the selected line's facts
/// on other screen rows, which can outscore the row itself.
fn score_left(line: &str, texts: &[String]) -> usize {
    score(line.split('│').next().unwrap_or(line), texts)
}

/// The invariants after every step: the terminal shows exactly what the TUI says it shows.
pub fn verify(w: &World, r: &mut Report, step: &str) -> Value {
    r.steps += 1;
    w.settle();
    // what the terminal showed when the reads below were served (a read is logged in Activity and may
    // mark the header, so the terminal can move on right after it)
    let shown_before = trim_lines(&w.screen().text());
    let since = w.pty.as_ref().map_or(0, |p| p.snapshot().frames);
    // a frame the bridge drew must be one the terminal showed, before the reads or since (on a loaded
    // machine the bytes arrive late: wait for it, never assume a speed)
    let shown = |model: &[String]| -> bool {
        shown_before == model
            || w.pty.as_ref().is_some_and(|p| {
                p.wait_for(std::time::Duration::from_secs(10), |t| {
                    trim_lines(&t.text()) == model
                        || t.history
                            .iter()
                            .any(|(n, x)| *n > since && trim_lines(x) == model)
                })
            })
    };
    let canon = w.cli(&["--json", "ui", "state"]);
    let state = canon.json();
    if state.is_null() {
        r.fail(
            step,
            format!("ui state returned no JSON: {}", canon.stdout.trim()),
        );
        return state;
    }
    // 0. the state follows its versioned schema (docs/schema/ui-state.v1.json)
    for p in super::schema::check(&super::schema::load("ui-state.v1.json"), &state, "state") {
        r.fail(step, format!("schema: {p}"));
    }
    w.settle();
    // 1. the bridge's frame is the terminal's frame (waiting for the terminal to finish drawing it: on a
    //    loaded machine the bytes arrive late; the check is that it gets there, not how fast)
    let shot = w
        .cli(&["--json", "ui", "screen", "--format", "text"])
        .json();
    let model = trim_lines(shot["body"].as_str().unwrap_or(""));
    let live = if shown(&model) {
        model.clone()
    } else {
        trim_lines(&w.screen().text())
    };
    if live != model {
        r.fail(
            step,
            format!(
                "ui screen differs from the terminal: {}",
                first_diff(&live, &model).unwrap_or_default()
            ),
        );
    }
    // 2. size
    r.check(
        state["size"]["cols"].as_u64() == Some(w.cols as u64)
            && state["size"]["rows"].as_u64() == Some(w.rows as u64),
        step,
        || format!("size {} vs terminal {}x{}", state["size"], w.cols, w.rows),
    );
    // 3. rows: on-screen rows match the view window, and each is on the terminal, in order
    let rows = state["rows"].as_array().cloned().unwrap_or_default();
    let first = state["view"]["first_visible"].as_u64().unwrap_or(0);
    let last = state["view"]["last_visible"].as_u64().unwrap_or(0);
    if !rows.is_empty() && state["truncated"] != true {
        r.check(
            rows.len() as u64 == last.saturating_sub(first) + 1,
            step,
            || format!("{} rows but view {first}..{last}", rows.len()),
        );
    }
    let mut from_line = 0usize;
    for row in &rows {
        let texts = row_texts(row);
        if texts.is_empty() {
            continue;
        }
        let need = texts.len().div_ceil(2);
        match (from_line..live.len()).find(|&i| score(&live[i], &texts) >= need) {
            Some(i) => from_line = i + 1,
            None => {
                r.fail(
                    step,
                    format!(
                        "row {} {:?} is not on the terminal (in order)",
                        row["row"], texts
                    ),
                );
                break;
            }
        }
    }
    // 4. the cursor (anywhere in the whole list) is on screen, and the selected row is the marked line
    let all = w.state(&["--all-rows"]);
    if let Some(cur) = all["rows"]
        .as_array()
        .into_iter()
        .flatten()
        .find(|x| x["selected"] == true)
    {
        // compare within the same read (each read is logged, so Activity grows between reads)
        let n = cur["row"].as_u64().unwrap_or(0);
        let (f2, l2) = (
            all["view"]["first_visible"].as_u64().unwrap_or(0),
            all["view"]["last_visible"].as_u64().unwrap_or(0),
        );
        // (below the minimum size nothing is listed: no window to be in)
        let too_small = w.cols < 60 || w.rows < 20;
        r.check(too_small || (n >= f2 && n <= l2), step, || {
            format!("the cursor is on row {n}, off the screen ({f2}..{l2})")
        });
    }
    let marked_n = rows.iter().filter(|x| x["selected"] == true).count();
    r.check(
        rows.is_empty()
            || marked_n == 1
            || state["selected"].is_null() && live.iter().all(|l| !l.starts_with('▌')),
        step,
        || {
            format!(
                "{marked_n} selected rows on screen, selected {}",
                state["selected"]
            )
        },
    );
    if let Some(sel) = rows.iter().find(|x| x["selected"] == true) {
        let texts = row_texts(sel);
        let marked: Vec<&String> = live.iter().filter(|l| l.starts_with('▌')).collect();
        // the marked line is the selected row's: no other visible row explains it better (twins tie), and
        // it carries at least half of the row's texts. (Asking the other way round, "is the marked line
        // the best match for the row", fails where a detail pane or a legend repeats the row's facts.)
        let need = texts.len().div_ceil(2);
        let ok = texts.is_empty()
            || marked.iter().any(|m| {
                let mine = score(m, &texts);
                let left = score_left(m, &texts);
                (mine >= need || left >= need)
                    && rows.iter().filter(|o| o["selected"] != true).all(|o| {
                        let t = row_texts(o);
                        score(m, &t) <= mine || score_left(m, &t) <= left
                    })
            });
        r.check(ok, step, || {
            format!("selected row {texts:?} is not the marked line; marked: {marked:?}")
        });
        let n = sel["row"].as_u64().unwrap_or(0);
        r.check(n >= first && n <= last, step, || {
            format!("selected row {n} outside view {first}..{last}")
        });
    }
    // 5. a fresh TUI built from the state draws the same frame
    let path = w.root.join("state.json");
    if std::fs::write(&path, &canon.stdout).is_ok() {
        let p = path.display().to_string();
        let o = w.cli(&[
            "--json",
            "ui",
            "screen",
            "--from-state",
            &p,
            "--format",
            "text",
        ]);
        let body = match o.json()["body"].as_str() {
            Some(b) => b.to_string(),
            None => {
                r.fail(step, "ui screen --from-state --json did not answer JSON");
                o.stdout.clone()
            }
        };
        let from = trim_lines(&body);
        if o.code != 0 {
            r.fail(step, format!("--from-state failed: {}", o.stdout.trim()));
        } else if from != live && !shown(&from) {
            r.fail(
                step,
                format!(
                    "--from-state differs from the terminal: {}",
                    first_diff(&live, &from).unwrap_or_default()
                ),
            );
        }
    }
    state
}

pub fn all(bins: Bins, _args: &[String]) -> Result<(), String> {
    let started = std::time::Instant::now();
    let mut r = Report::default();
    type Case = fn(&Bins, &mut Report) -> Result<(), String>;
    let cases: &[(&str, Case)] = &[
        ("walk_tabs", walk_tabs),
        ("verbs", verbs),
        ("discovery", discovery),
        ("guards", guards),
        ("classify_writes", classify_writes),
        ("captions", captions),
        ("select_anywhere", select_anywhere),
        ("expects", expects),
        ("waits", waits),
        ("upgrade", upgrade),
        ("upgrade_same_version", upgrade_same_version),
        ("regressions", regressions),
        ("bulk_cap", bulk_cap),
        ("sync_running", sync_running),
        ("snippet", snippet),
        ("stale_queue", stale_queue),
        ("too_small", too_small),
        ("burst", burst),
        ("config_reload", config_reload),
        ("external", external),
        ("writes", writes),
        ("manifests", manifests),
        ("walkthroughs", walkthroughs),
        ("upgrade_after_external", upgrade_after_external),
        ("upgrade_after_config_change", upgrade_after_config_change),
        ("concurrency", concurrency),
        ("failures", failures),
        ("soak", |b, r| {
            let seeds: u64 = std::env::var("OWNPURSE_LIVE_SOAK")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(3);
            soak(b, r, seeds, 60)
        }),
        ("latency", latency),
    ];
    // OWNPURSE_LIVE_ONLY=name[,name…] runs only those cases (for working on one; the gate runs all)
    let only = std::env::var("OWNPURSE_LIVE_ONLY").ok();
    for (name, case) in cases {
        if only
            .as_deref()
            .is_some_and(|o| !o.split(',').any(|x| x == *name))
        {
            continue;
        }
        case(&bins, &mut r)?;
    }
    println!(
        "agent-drive: {} steps, {} failures, {:.1}s",
        r.steps,
        r.failures.len(),
        started.elapsed().as_secs_f64()
    );
    for f in &r.failures {
        println!("FAIL {f}");
    }
    if r.failures.is_empty() {
        Ok(())
    } else {
        Err(format!("{} live failures", r.failures.len()))
    }
}

/// Every tab, drill and back out, with the invariants after each step.
fn walk_tabs(bins: &Bins, r: &mut Report) -> Result<(), String> {
    let mut w = World::new("walk", bins.clone())?;
    w.start(100, 30, &[])?;
    verify(&w, r, "start");
    for tab in [
        "ledger",
        "intercompany",
        "activity",
        "system",
        "classify",
        "overview",
    ] {
        let o = w.send(&["tab", tab]);
        r.check(o.code == 0, &format!("tab {tab}"), || o.stdout.clone());
        verify(&w, r, &format!("tab {tab}"));
        let o = w.send(&["drill"]);
        verify(&w, r, &format!("{tab} drill ({})", o.code));
        let o = w.send(&["up"]);
        verify(&w, r, &format!("{tab} up ({})", o.code));
    }
    Ok(())
}

pub fn probe(bins: Bins, script: &[&str]) -> Result<(), String> {
    let mut w = World::new("probe", bins)?;
    let rows: u16 = std::env::var("XAD_ROWS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(30);
    w.start(100, rows, &[])?;
    for s in script {
        let args: Vec<&str> = s.split(' ').collect();
        let o = w.send(&args);
        let j = o.json();
        println!(
            "$ send {s} -> {} {} changed={} err={} msg={}",
            o.code, j["rev"], j["changed"], j["error"], j["message"]
        );
        println!("effects {}", j["effects"]);
        println!("{}", w.screen().text());
        let st = w.state(&[]);
        println!(
            "tab={} level={} breadcrumb={} sel={} view={} filter={}",
            st["tab"], st["level"], st["breadcrumb"], st["selected"], st["view"], st["filter"]
        );
        if let Some(r) = st["rows"].as_array().and_then(|r| r.first()) {
            println!("row0 {}", r);
        }
    }
    Ok(())
}

fn st_rev(w: &World) -> i64 {
    w.state(&[])["rev"].as_i64().unwrap_or(-1)
}

/// A step: send, check the exit code, verify the invariants; returns the reply.
fn step(w: &World, r: &mut Report, args: &[&str], want: i32) -> Value {
    let name = args.join(" ");
    let o = w.send(args);
    let j = o.json();
    r.check(o.code == want, &name, || {
        format!("exit {} (wanted {want}): {}", o.code, o.stdout.trim())
    });
    r.check(!j.is_null(), &name, || {
        format!("--json reply is not JSON: {}", o.stdout.trim())
    });
    if o.code != 0 {
        check_error(r, &name, &j);
    }
    if o.code == 0 {
        r.check(
            j["effects"].as_array().is_some_and(|e| !e.is_empty()),
            &name,
            || format!("no effects: {}", o.stdout.trim()),
        );
        r.check(
            j["summary"]["rev"] == j["rev"] && !j["summary"]["tab"].is_null(),
            &name,
            || format!("no summary: {}", j["summary"]),
        );
    }
    let st = verify(w, r, &name);
    // a history move says where it landed
    if want == 0
        && let Some(c) = j["changed"]
            .as_array()
            .and_then(|c| c.first())
            .and_then(Value::as_str)
        && (c.starts_with("back → ") || c.starts_with("forward → "))
    {
        let crumbs: Vec<String> = st["breadcrumb"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(|x| x.as_str().map(str::to_string))
            .collect();
        let mut place = crumbs.join(" › ");
        if st["filter"].as_str().is_some_and(|f| !f.is_empty()) {
            place = format!("{place} › filter “{}”", st["filter"].as_str().unwrap_or(""));
        }
        r.check(c.ends_with(&place), &name, || {
            format!("the history notice says {c:?} but the view is {place:?}")
        });
    }
    j
}

/// Every view verb on every tab and level.
fn verbs(bins: &Bins, r: &mut Report) -> Result<(), String> {
    let mut w = World::new("verbs", bins.clone())?;
    w.start(100, 30, &[])?;
    // overview
    step(&w, r, &["select", "#2"], 0);
    step(&w, r, &["select", "design"], 0);
    step(&w, r, &["select", r#"{"org":"lab"}"#], 0);
    step(&w, r, &["drill"], 0);
    step(&w, r, &["back"], 0);
    step(&w, r, &["forward"], 0);
    step(&w, r, &["up"], 0);
    // ledger, every level
    step(&w, r, &["tab", "ledger", "scope", "roast"], 0);
    step(&w, r, &["asof", "2025-06-30"], 0);
    step(&w, r, &["asof", "2025-12-31"], 0);
    step(&w, r, &["basis", "history"], 0);
    step(&w, r, &["differs", "on"], 0);
    step(&w, r, &["differs", "off"], 0);
    step(&w, r, &["basis", "year"], 0);
    step(&w, r, &["filter", "Operating"], 0);
    step(&w, r, &["clear-filter"], 0);
    step(&w, r, &["filter", "code:1010..1060"], 0);
    step(&w, r, &["select", r#"{"account_code":"1060"}"#], 0);
    step(&w, r, &["clear-filter"], 0);
    step(&w, r, &["select", "1010"], 0);
    step(&w, r, &["drill"], 0);
    step(&w, r, &["select", "#2"], 0);
    step(&w, r, &["drill"], 0);
    step(&w, r, &["up"], 0);
    step(&w, r, &["back"], 0);
    step(&w, r, &["back"], 0);
    step(&w, r, &["forward"], 0);
    step(&w, r, &["up"], 0);
    step(&w, r, &["up"], 0);
    step(&w, r, &["compare", "roast,hold"], 0);
    step(&w, r, &["drill"], 0);
    step(&w, r, &["up"], 0);
    step(&w, r, &["up"], 0);
    step(&w, r, &["scope", "all"], 0);
    // intercompany
    step(&w, r, &["tab", "intercompany"], 0);
    step(&w, r, &["select", "#2"], 0);
    step(&w, r, &["drill"], 0);
    step(&w, r, &["up"], 0);
    // activity and system
    step(&w, r, &["tab", "activity"], 0);
    step(&w, r, &["filter", "lab"], 0);
    step(&w, r, &["clear-filter"], 0);
    step(&w, r, &["tab", "system"], 0);
    step(&w, r, &["select", "#2"], 0);
    // classify
    step(&w, r, &["tab", "classify"], 0);
    step(&w, r, &["filter", "org:roast"], 0);
    step(&w, r, &["select", "#2"], 0);
    step(&w, r, &["clear-filter"], 0);
    // the rest
    step(&w, r, &["reload"], 0);
    step(&w, r, &["caption", "Demo walk · 4 orgs"], 0);
    step(&w, r, &["label", "books"], 0);
    step(&w, r, &["tab", "overview"], 0);
    Ok(())
}

/// `ui ls` and `--session` by id, pid, label and scope; ambiguous and stale sessions.
fn discovery(bins: &Bins, r: &mut Report) -> Result<(), String> {
    let mut a = World::new("disc", bins.clone())?;
    a.start(100, 30, &["--label", "books"])?;
    let ls = a.cli(&["--json", "ui", "ls"]).json();
    let s = ls["sessions"][0].clone();
    for key in [
        s["id"].as_str().unwrap_or("").to_string(),
        s["pid"].to_string(),
        "books".to_string(),
        "scope:all".to_string(),
    ] {
        let o = a.cli(&["--json", "ui", "state", "--session", &key]);
        r.check(o.code == 0, &format!("--session {key}"), || {
            o.stdout.clone()
        });
    }
    let o = a.cli(&["--json", "ui", "state", "--session", "nope"]);
    r.check(o.code == 2, "--session nope", || {
        format!("exit {}: {}", o.code, o.stdout)
    });
    r.check(!o.json()["help"].is_null(), "--session nope", || {
        format!("no help line: {}", o.stdout)
    });
    // a second TUI on the same registry: unqualified commands become ambiguous
    let mut b = World::new("disc2", bins.clone())?;
    b.root = a.root.clone();
    let spawned = super::pty::Pty::spawn(
        &b.bins.tui,
        &["--label".into(), "other".into()],
        &a.env(),
        &a.work(),
        100,
        30,
    )
    .map_err(|e| e.to_string())?;
    b.pty = Some(spawned);
    let start = std::time::Instant::now();
    while a.cli(&["--json", "ui", "ls"]).json()["sessions"]
        .as_array()
        .map_or(0, Vec::len)
        < 2
    {
        if start.elapsed().as_secs() > 60 {
            r.fail("two sessions", "the second TUI never registered");
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
    let o = a.cli(&["--json", "ui", "state"]);
    r.check(o.code == 2, "ambiguous", || {
        format!("exit {}: {}", o.code, o.stdout)
    });
    let o = a.cli(&["--json", "ui", "state", "--session", "other"]);
    r.check(o.code == 0, "--session other", || o.stdout.clone());
    // SIGKILL leaves a stale entry; ls prunes it
    b.pty().kill(libc::SIGKILL);
    b.pty().wait_exit(std::time::Duration::from_secs(30));
    let ls = a.cli(&["--json", "ui", "ls"]).json();
    let n = ls["sessions"].as_array().map_or(0, Vec::len);
    r.check(n == 1, "stale pruned", || {
        format!("{n} sessions after SIGKILL: {ls}")
    });
    let o = a.cli(&["--json", "ui", "state"]);
    r.check(o.code == 0, "after prune", || o.stdout.clone());
    b.root = std::path::PathBuf::from("/nonexistent-xad");
    Ok(())
}

/// Guards: stale rev, the user typing, forbidden verbs, atomic refusal.
fn guards(bins: &Bins, r: &mut Report) -> Result<(), String> {
    let mut w = World::new("guards", bins.clone())?;
    w.start(100, 30, &[])?;
    step(&w, r, &["tab", "ledger", "scope", "roast"], 0);
    let rev = st_rev(&w);
    let stale = (rev - 1).to_string();
    let j = step(&w, r, &["--expect-rev", &stale, "select", "#2"], 4);
    r.check(j["error"] == "view_changed", "stale rev", || j.to_string());
    r.check(st_rev(&w) == rev, "stale rev", || "the rev moved".into());
    let j = step(
        &w,
        r,
        &["--expect-rev", &rev.to_string(), "select", "#2"],
        0,
    );
    r.check(j["rev"].as_i64() == Some(rev + 1), "expect-rev ok", || {
        j.to_string()
    });
    // forbidden verbs: exit 2, nothing changes
    for bad in ["sync", "quit", "confirm", "edit", "frobnicate"] {
        let before = st_rev(&w);
        let screen = trim_lines(&w.screen().text());
        let j = step(&w, r, &[bad], 2);
        r.check(
            !j["help"].is_null() || j["help"].as_array().is_some_and(|h| !h.is_empty()),
            bad,
            || j.to_string(),
        );
        r.check(st_rev(&w) == before, bad, || "the rev moved".into());
        r.check(trim_lines(&w.screen().text()) == screen, bad, || {
            "the screen changed".into()
        });
    }
    // atomic: a bad verb later in the list applies nothing
    let before = st_rev(&w);
    let tab_before = w.state(&[])["tab"].clone();
    step(&w, r, &["tab", "overview", "bogus"], 2);
    r.check(
        st_rev(&w) == before && w.state(&[])["tab"] == tab_before,
        "atomic",
        || "a refused send applied a verb".into(),
    );
    step(&w, r, &["tab", "overview", "asof", "not-a-date"], 2);
    r.check(w.state(&[])["tab"] == tab_before, "atomic asof", || {
        "a refused send applied a verb".into()
    });
    // the user typing: a prompt is open, so the agent waits
    w.keys(b"/");
    let j = step(&w, r, &["select", "#3"], 4);
    r.check(j["error"] == "user_busy", "user_busy", || j.to_string());
    w.keys(b"\x1b");
    step(&w, r, &["select", "#3"], 0);
    Ok(())
}

fn store_rev(w: &World) -> String {
    w.state(&[])["classify"]["store_rev"].to_string()
}

/// Agent writes in Classify, routed through the running TUI.
fn classify_writes(bins: &Bins, r: &mut Report) -> Result<(), String> {
    let mut w = World::new("cw", bins.clone())?;
    w.start(100, 30, &[])?;
    step(&w, r, &["tab", "classify"], 0);
    let sr = store_rev(&w);
    let o = w.cli(&[
        "--json",
        "classify",
        "set",
        "roast:bt:bt-104:L1",
        "project=harvest",
        "--expect-store-rev",
        &sr,
        "--client",
        "claude",
        "--caption",
        "Green coffee for the launch",
    ]);
    w.settle();
    r.check(o.code == 0, "classify set (routed)", || {
        format!("exit {}: {}", o.code, o.stdout)
    });
    let st = verify(&w, r, "after classify set");
    r.check(
        st["classify"]["agent_writes"].as_i64() == Some(1),
        "agent_writes",
        || st["classify"].to_string(),
    );
    let screen = w.screen().text();
    r.check(
        screen.contains("◆ agent") && screen.contains("U undo"),
        "write notice",
        || format!("no agent notice:\n{screen}"),
    );
    // a stale store rev is refused
    let o = w.cli(&[
        "--json",
        "classify",
        "set",
        "roast:bt:bt-109:L1",
        "project=harvest",
        "--expect-store-rev",
        &sr,
        "--client",
        "claude",
    ]);
    r.check(
        o.code == 4 && o.json()["error"] == "view_changed",
        "stale store rev",
        || format!("exit {}: {}", o.code, o.stdout),
    );
    // human labels are kept
    let sr = store_rev(&w);
    let o = w.cli(&[
        "--json",
        "classify",
        "set",
        "design:bt:bt-303:L1",
        "project=refit",
        "--expect-store-rev",
        &sr,
        "--client",
        "claude",
    ]);
    r.check(o.code == 0, "set on a human label", || o.stdout.clone());
    let skipped = o.json()["skipped"].as_array().map_or(0, Vec::len);
    r.check(skipped == 1, "human label kept", || o.stdout.clone());
    verify(&w, r, "after human-label set");
    // U undoes the latest agent batch
    w.keys(b"U");
    let st = verify(&w, r, "after U");
    let screen = w.screen().text();
    r.check(screen.contains("undone"), "U notice", || {
        format!("no undo notice:\n{screen}")
    });
    let _ = st;
    // undo through the CLI too
    let sr = store_rev(&w);
    let o = w.cli(&[
        "--json",
        "classify",
        "undo",
        "--batch",
        "--expect-store-rev",
        &sr,
        "--client",
        "claude",
    ]);
    r.check(o.code == 0, "classify undo --batch", || o.stdout.clone());
    verify(&w, r, "after undo --batch");
    // the user has a picker open on a line: a write to it waits
    step(&w, r, &["select", r#"{"anchor":"roast:bt:bt-104:L1"}"#], 0);
    let sel = w.state(&[])["classify"]["selected_anchor"]
        .as_str()
        .unwrap_or("")
        .to_string();
    w.keys(b"b");
    let sr = store_rev(&w);
    let o = w.cli(&[
        "--json",
        "classify",
        "set",
        &sel,
        "bearer=roast",
        "--expect-store-rev",
        &sr,
        "--client",
        "claude",
    ]);
    r.check(
        o.code == 4 && o.json()["error"] == "user_busy",
        "write under an open picker",
        || format!("exit {}: {}", o.code, o.stdout),
    );
    w.keys(b"\x1b");
    verify(&w, r, "picker closed");
    Ok(())
}

fn footer(w: &World) -> String {
    let t = w.screen();
    t.line(t.rows - 1)
}

/// Captions and notices: shown, kept by selection moves, cleared by a view change.
fn captions(bins: &Bins, r: &mut Report) -> Result<(), String> {
    let mut w = World::new("cap", bins.clone())?;
    w.start(100, 30, &[])?;
    step(&w, r, &["tab", "ledger", "scope", "roast"], 0);
    let ft = footer(&w);
    r.check(
        ft.contains("◆ agent: opened Ledger"),
        "notice shows",
        || ft.clone(),
    );
    step(
        &w,
        r,
        &["--caption", "Operating cash, roast", "select", "#3"],
        0,
    );
    let st = w.state(&[]);
    r.check(
        st["caption"]["text"] == "Operating cash, roast",
        "caption in state",
        || st["caption"].to_string(),
    );
    let ft = footer(&w);
    r.check(
        ft.contains("◆ Operating cash, roast"),
        "caption shows",
        || ft.clone(),
    );
    step(&w, r, &["select", "#4"], 0);
    r.check(
        w.state(&[])["caption"]["text"] == "Operating cash, roast",
        "caption kept by select",
        || "cleared".into(),
    );
    // the user moving the selection keeps it too
    w.keys(b"j");
    verify(&w, r, "user j");
    r.check(
        !w.state(&[])["caption"].is_null(),
        "caption kept by user selection",
        || "cleared".into(),
    );
    step(&w, r, &["drill"], 0);
    let st = w.state(&[]);
    r.check(st["caption"].is_null(), "caption cleared by drill", || {
        st["caption"].to_string()
    });
    r.check(
        !footer(&w).contains("Operating cash"),
        "caption gone from footer",
        || footer(&w),
    );
    step(&w, r, &["caption", "x"], 0);
    step(&w, r, &["caption", "clear"], 0);
    r.check(w.state(&[])["caption"].is_null(), "caption clear", || {
        "still set".into()
    });
    let j = step(&w, r, &["caption", "bad\u{1b}[31m"], 2);
    r.check(j["error"] == "bad_caption", "bad_caption", || j.to_string());
    Ok(())
}

/// Two agents at once, rapid fire, and sends during a user's typing; replies never lost, revs monotonic.
fn concurrency(bins: &Bins, r: &mut Report) -> Result<(), String> {
    let mut w = World::new("conc", bins.clone())?;
    w.start(100, 30, &[])?;
    step(&w, r, &["tab", "ledger", "scope", "roast"], 0);
    let w = std::sync::Arc::new(w);
    let mut handles = vec![];
    for k in 0..2 {
        let w2 = w.clone();
        handles.push(std::thread::spawn(move || {
            let mut revs = vec![];
            let mut bad = vec![];
            for i in 0..25 {
                let row = format!("#{}", 1 + (i + k) % 8);
                let o = w2.cli(&["--json", "ui", "send", "select", &row]);
                if o.code != 0 {
                    bad.push(format!("exit {}: {}", o.code, o.stdout.trim()));
                } else {
                    revs.push(o.json()["rev"].as_i64().unwrap_or(-1));
                }
            }
            (revs, bad)
        }));
    }
    let mut all = vec![];
    for h in handles {
        let (revs, bad) = h.join().map_err(|_| "a sender panicked".to_string())?;
        for b in bad {
            r.fail("two agents", b);
        }
        r.check(revs.windows(2).all(|p| p[0] <= p[1]), "two agents", || {
            format!("revs went backwards: {revs:?}")
        });
        all.extend(revs);
    }
    w.settle();
    verify(&w, r, "after two agents");
    // 100 sends in a row
    let start = st_rev(&w);
    let mut last = start;
    for i in 0..100 {
        let o = w.cli(&["--json", "ui", "send", "select", &format!("#{}", 1 + i % 8)]);
        let rev = o.json()["rev"].as_i64().unwrap_or(-1);
        if o.code != 0 || rev < last {
            r.fail(
                "rapid fire",
                format!("send {i}: exit {} rev {rev} after {last}", o.code),
            );
            break;
        }
        last = rev;
    }
    w.settle();
    verify(&w, r, "after rapid fire");
    let _ = all;
    Ok(())
}

/// Failure modes: a garbage or oversized request, a client gone mid-request, a resize mid-request, the TUI
/// exiting with a request in flight.
fn failures(bins: &Bins, r: &mut Report) -> Result<(), String> {
    use std::io::{Read, Write};
    let mut w = World::new("fail", bins.clone())?;
    w.start(100, 30, &[])?;
    let sock = w.cli(&["--json", "ui", "ls"]).json()["sessions"][0]["socket"]
        .as_str()
        .unwrap_or("")
        .to_string();
    let ask = |payload: &[u8]| -> String {
        match std::os::unix::net::UnixStream::connect(&sock) {
            Ok(mut c) => {
                let _ = c.set_read_timeout(Some(std::time::Duration::from_secs(5)));
                let _ = c.write_all(payload);
                let _ = c.shutdown(std::net::Shutdown::Write);
                let mut out = String::new();
                let _ = c.read_to_string(&mut out);
                out
            }
            Err(e) => format!("connect: {e}"),
        }
    };
    for (name, payload) in [
        ("garbage", b"not json at all\n".to_vec()),
        (
            "wrong version",
            b"{\"v\": 99, \"kind\": \"view\", \"op\": \"state\"}\n".to_vec(),
        ),
        (
            "unknown kind",
            b"{\"v\": 1, \"kind\": \"bogus\", \"op\": \"x\"}\n".to_vec(),
        ),
        ("empty", b"\n".to_vec()),
    ] {
        let a = ask(&payload);
        let j: Value = serde_json::from_str(a.trim()).unwrap_or(Value::Null);
        r.check(j["ok"] == false && j["error"].is_string(), name, || {
            format!("answer: {a:?}")
        });
    }
    let mut big = vec![b'x'; (1 << 20) + 10];
    big.push(b'\n');
    let a = ask(&big);
    let j: Value = serde_json::from_str(a.trim()).unwrap_or(Value::Null);
    r.check(j["ok"] == false, "oversized", || {
        format!("answer: {:?}", &a[..a.len().min(200)])
    });
    // a client that connects and goes away without a word, or half a request
    for _ in 0..5 {
        if let Ok(c) = std::os::unix::net::UnixStream::connect(&sock) {
            drop(c);
        }
        if let Ok(mut c) = std::os::unix::net::UnixStream::connect(&sock) {
            let _ = c.write_all(b"{\"v\": 1, \"kind\": \"view\", \"op\"");
            drop(c);
        }
    }
    let o = w.cli(&["--json", "ui", "state"]);
    r.check(o.code == 0, "alive after bad clients", || o.stdout.clone());
    // a client that connects and says nothing never holds up another agent
    let silent = std::os::unix::net::UnixStream::connect(&sock).ok();
    let o = w.cli(&["--json", "ui", "send", "select", "#2"]);
    r.check(
        // a server that answered one connection at a time would hold this send for the silent client's
        // whole 5 s read timeout: the bound is that structure, not a speed assumption
        o.code == 0 && o.elapsed.as_millis() < 4500,
        "send beside a silent client",
        || format!("exit {} after {:?}: {}", o.code, o.elapsed, o.stdout.trim()),
    );
    drop(silent);
    verify(&w, r, "after bad clients");
    // resize while requests arrive
    let w = std::sync::Arc::new(std::sync::Mutex::new(w));
    let w2 = w.clone();
    let sender = std::thread::spawn(move || {
        let mut bad = vec![];
        for i in 0..30 {
            let o = {
                let g = w2.lock().unwrap_or_else(|e| e.into_inner());
                g.cli(&["--json", "ui", "send", "select", &format!("#{}", 1 + i % 4)])
            };
            if o.code != 0 {
                bad.push(o.stdout);
            }
            std::thread::sleep(std::time::Duration::from_millis(3));
        }
        bad
    });
    for (c, rr) in [(90u16, 28u16), (120, 40), (80, 24), (100, 30)] {
        let mut g = w.lock().unwrap_or_else(|e| e.into_inner());
        let _ = g.pty().resize(c, rr);
        g.cols = c;
        g.rows = rr;
        drop(g);
        std::thread::sleep(std::time::Duration::from_millis(30));
    }
    for b in sender.join().map_err(|_| "sender panicked".to_string())? {
        r.fail("send during resize", b);
    }
    let mut g = w.lock().unwrap_or_else(|e| e.into_inner());
    g.settle();
    verify(&g, r, "after resizes");
    // the TUI quits with a request in flight: the client gets an error, not a hang
    let cli = g.bins.cli.clone();
    let env = g.env();
    let work = g.work();
    let pending = std::thread::spawn(move || {
        std::process::Command::new(cli)
            .args(["--json", "ui", "send", "select", "#2"])
            .current_dir(work)
            .env_clear()
            .envs(env)
            .output()
    });
    g.pty().kill(libc::SIGTERM);
    let st = g.pty().wait_exit(std::time::Duration::from_secs(30));
    r.check(st.is_some(), "SIGTERM exit", || {
        "the TUI did not exit".into()
    });
    let out = pending.join().map_err(|_| "pending panicked".to_string())?;
    if let Ok(o) = out {
        let code = o.status.code().unwrap_or(-1);
        r.check(code == 0 || code == 1, "request in flight at exit", || {
            format!("exit {code}")
        });
    }
    let ls = g.cli(&["--json", "ui", "ls"]).json();
    r.check(
        ls["sessions"].as_array().is_none_or(Vec::is_empty),
        "registry after SIGTERM",
        || ls.to_string(),
    );
    Ok(())
}

/// A small deterministic RNG (xorshift).
struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
    fn pick<'a, T>(&mut self, v: &'a [T]) -> &'a T {
        &v[(self.next() % v.len() as u64) as usize]
    }
}

/// The seeded soak: random verbs, user keys and resizes; the invariants after every step.
fn soak(bins: &Bins, r: &mut Report, seeds: u64, steps: usize) -> Result<(), String> {
    let verbs: &[&[&str]] = &[
        &["tab", "overview"],
        &["tab", "ledger"],
        &["tab", "intercompany"],
        &["tab", "activity"],
        &["tab", "system"],
        &["tab", "classify"],
        &["scope", "roast"],
        &["scope", "design"],
        &["scope", "all"],
        &["drill"],
        &["up"],
        &["back"],
        &["forward"],
        &["select", "#1"],
        &["select", "#2"],
        &["select", "#5"],
        &["filter", "a"],
        &["filter", "Operating"],
        &["clear-filter"],
        &["basis", "history"],
        &["basis", "year"],
        &["differs", "on"],
        &["differs", "off"],
        &["asof", "2025-06-30"],
        &["asof", "2025-12-31"],
        &["caption", "soak"],
        &["reload"],
    ];
    let keys: &[&[u8]] = &[
        b"j", b"k", b"\r", b"\x1b", b"1", b"2", b"3", b"6", b"]", b"[", b"\x1b[B", b"\x1b[A",
    ];
    let sizes: &[(u16, u16)] = &[(100, 30), (80, 24), (140, 40), (120, 35)];
    for seed in 1..=seeds {
        let mut rng = Rng(0x9E37_79B9_7F4A_7C15 ^ seed.wrapping_mul(0x2545_F491_4F6C_DD1D));
        let mut w = World::new(&format!("soak{seed}"), bins.clone())?;
        w.start(100, 30, &[])?;
        let mut last_rev = st_rev(&w);
        for i in 0..steps {
            let roll = rng.next() % 10;
            let name;
            if roll < 7 {
                let v = rng.pick(verbs);
                name = format!("seed {seed} step {i}: send {}", v.join(" "));
                let o = w.send(v);
                if o.code != 0 && o.code != 2 && o.code != 4 {
                    r.fail(&name, format!("exit {}: {}", o.code, o.stdout.trim()));
                }
                if o.json().is_null() {
                    r.fail(&name, format!("reply is not JSON: {}", o.stdout.trim()));
                }
            } else if roll < 9 {
                let k = rng.pick(keys);
                name = format!("seed {seed} step {i}: key {:?}", String::from_utf8_lossy(k));
                w.keys(k);
            } else {
                let (c, rr) = *rng.pick(sizes);
                name = format!("seed {seed} step {i}: resize {c}x{rr}");
                let _ = w.pty().resize(c, rr);
                w.cols = c;
                w.rows = rr;
                w.settle();
            }
            // a key may have opened a prompt or overlay: close it so the agent can read the state
            let st = verify(&w, r, &name);
            let rev = st["rev"].as_i64().unwrap_or(-1);
            r.check(rev >= last_rev, &name, || {
                format!("rev went back {last_rev} → {rev}")
            });
            last_rev = rev;
            if w.pty
                .as_mut()
                .is_some_and(|p| p.child.try_wait().ok().flatten().is_some())
            {
                r.fail(&name, "the TUI exited");
                break;
            }
        }
    }
    Ok(())
}

fn pct(v: &mut [u128], p: usize) -> u128 {
    v.sort_unstable();
    v[((v.len() * p).div_ceil(100))
        .saturating_sub(1)
        .min(v.len() - 1)]
}

/// p50/p99 per verb, end to end (CLI process start included).
fn latency(bins: &Bins, r: &mut Report) -> Result<(), String> {
    let mut w = World::new("lat", bins.clone())?;
    w.start(100, 30, &[])?;
    w.send(&["tab", "ledger", "scope", "roast"]);
    let cases: &[(&str, &[&str], u128)] = &[
        ("ui state", &["--json", "ui", "state"], 20),
        (
            "ui state (lean)",
            &["--json", "ui", "state", "--rows", "1..3"],
            20,
        ),
        ("ui ls", &["--json", "ui", "ls"], 20),
        ("send select", &["--json", "ui", "send", "select", "#2"], 50),
        (
            "send filter",
            &["--json", "ui", "send", "filter", "Operating"],
            50,
        ),
        (
            "send clear-filter",
            &["--json", "ui", "send", "clear-filter"],
            50,
        ),
        (
            "send drill+up",
            &["--json", "ui", "send", "drill", "up"],
            50,
        ),
        (
            "send tab",
            &["--json", "ui", "send", "tab", "overview", "tab", "ledger"],
            50,
        ),
    ];
    println!("latency (ms, end to end, 40 runs):");
    println!("  {:<20} {:>6} {:>6} {:>8}", "call", "p50", "p99", "budget");
    let gate = std::env::var("OWNPURSE_LIVE_LATENCY_GATE").is_ok_and(|v| v == "1");
    for (name, args, budget) in cases {
        let mut v: Vec<u128> = (0..40).map(|_| w.cli(args).elapsed.as_micros()).collect();
        let ms = |us: u128| format!("{}.{}", us / 1000, us % 1000 / 100);
        let p50 = ms(pct(&mut v, 50));
        let p99v = pct(&mut v, 99);
        let p99 = ms(p99v);
        println!("  {name:<20} {p50:>6} {p99:>6} {budget:>8}");
        if gate && p99v > budget * 2 * 1000 {
            r.fail(name, format!("p99 {p99} ms > 2 × {budget} ms"));
        }
    }
    Ok(())
}

/// `select` reaches rows off the screen (anchor, `#n`, text), scrolls to them, and `drill` acts on them.
fn select_anywhere(bins: &Bins, r: &mut Report) -> Result<(), String> {
    let mut w = World::new("selany", bins.clone())?;
    w.start(100, 20, &[])?;
    step(&w, r, &["tab", "ledger", "scope", "roast"], 0);
    let st = w.state(&["--all-rows"]);
    let rows = st["rows"].as_array().cloned().unwrap_or_default();
    let last_visible = st["view"]["last_visible"].as_u64().unwrap_or(0) as usize;
    if rows.len() <= last_visible {
        r.fail(
            "select anywhere",
            format!(
                "the TB fits the screen ({} rows): the test needs a longer list",
                rows.len()
            ),
        );
        return Ok(());
    }
    let far = rows.last().cloned().unwrap_or(Value::Null);
    let code = far["anchor"]["account_code"]
        .as_str()
        .unwrap_or("")
        .to_string();
    let anchor = format!(r#"{{"account_code":"{code}"}}"#);
    let j = step(&w, r, &["select", &anchor, "drill"], 0);
    let crumbs = j["state"]["breadcrumb"].to_string();
    r.check(
        j["state"]["level"] == 2 && crumbs.contains(&code),
        "select off-screen + drill",
        || crumbs.clone(),
    );
    step(&w, r, &["up"], 0);
    let st = w.state(&[]);
    r.check(
        st["selected"]["account_code"] == code.as_str(),
        "selection kept after up",
        || st["selected"].to_string(),
    );
    // #n in the whole list, and back to the top
    step(&w, r, &["select", "#1"], 0);
    let n = rows.len().to_string();
    step(&w, r, &["select", &format!("#{n}")], 0);
    r.check(
        w.state(&[])["selected"]["account_code"] == code.as_str(),
        "#n off-screen",
        || "wrong row".into(),
    );
    // text over the whole list
    let name = far["fields"]["account"].as_str().unwrap_or("").to_string();
    step(&w, r, &["select", "#1"], 0);
    step(&w, r, &["select", &name], 0);
    r.check(
        w.state(&[])["selected"]["account_code"] == code.as_str(),
        "text off-screen",
        || "wrong row".into(),
    );
    // not in the list at all
    let j = step(&w, r, &["select", r#"{"account_code":"0000"}"#], 2);
    r.check(j["error"] == "not_found", "not_found", || j.to_string());
    // a long list: the classify queue and activity
    step(&w, r, &["tab", "classify"], 0);
    let all = w.state(&["--all-rows"]);
    if let Some(last) = all["rows"].as_array().and_then(|v| v.last()) {
        let a = last["anchor"].to_string();
        step(&w, r, &["select", &a], 0);
        let st = w.state(&[]);
        r.check(
            st["selected"] == last["anchor"],
            "classify off-screen",
            || st["selected"].to_string(),
        );
    }
    step(&w, r, &["tab", "activity"], 0);
    let total = w.state(&[])["view"]["total_rows"].as_u64().unwrap_or(0);
    if total > 20 {
        step(&w, r, &["select", &format!("#{}", total - 3)], 0);
    }
    Ok(())
}

/// `--expect` intent guards: checked with the send, atomically; a mismatch lists expected vs actual.
fn expects(bins: &Bins, r: &mut Report) -> Result<(), String> {
    let mut w = World::new("expect", bins.clone())?;
    w.start(100, 30, &[])?;
    step(&w, r, &["tab", "ledger", "scope", "roast"], 0);
    step(
        &w,
        r,
        &[
            "--expect",
            "tab=ledger",
            "--expect",
            "level=1",
            "--expect",
            "scope=roast",
            "select",
            "1010",
            "drill",
        ],
        0,
    );
    let rev = st_rev(&w);
    // the incident: the agent means to caption L2, but the view is back on the TB
    step(&w, r, &["up"], 0);
    let before = w.state(&[]);
    let j = step(
        &w,
        r,
        &[
            "--expect",
            "level=2",
            "--expect",
            "breadcrumb~Operating Account",
            "caption",
            "Operating cash postings",
        ],
        4,
    );
    r.check(j["error"] == "view_mismatch", "view_mismatch", || {
        j.to_string()
    });
    r.check(
        j["expected"][0]["path"] == "level" && j["actual"][0]["value"] == 1,
        "expected vs actual",
        || j.to_string(),
    );
    // the text-mode refusal says what was expected and what is there
    let o = w.cli(&["ui", "send", "--expect", "level=2", "up"]);
    let text = format!("{}{}", o.stdout, o.stderr);
    r.check(
        o.code == 4 && text.contains("level") && text.contains('2') && text.contains('1'),
        "view_mismatch in text mode",
        || text.clone(),
    );
    let after = w.state(&[]);
    r.check(
        after["caption"].is_null() && after["rev"] == before["rev"],
        "mismatch applies nothing",
        || after["caption"].to_string(),
    );
    // dotted paths, substring, inequality; with --expect-rev too
    let rev2 = st_rev(&w).to_string();
    step(
        &w,
        r,
        &[
            "--expect-rev",
            &rev2,
            "--expect",
            "selected.account_code=1010",
            "--expect",
            "breadcrumb~trial balance",
            "--expect",
            "filter!=x",
            "select",
            "#2",
        ],
        0,
    );
    let j = step(
        &w,
        r,
        &["--expect", "selected.account_code=9999", "select", "#3"],
        4,
    );
    r.check(
        j["actual"][0]["text"] == w.state(&[])["selected"]["account_code"],
        "actual value",
        || j.to_string(),
    );
    // effects say when a verb did nothing
    step(&w, r, &["scope", "all"], 0);
    let j = step(&w, r, &["drill"], 0);
    r.check(
        j["effects"][0]["changed"] == false,
        "drill on nothing",
        || j["effects"].to_string(),
    );
    let j = step(
        &w,
        r,
        &["scope", "roast", "select", "#5", "select", "#5"],
        0,
    );
    let e: Vec<bool> = j["effects"]
        .as_array()
        .into_iter()
        .flatten()
        .map(|x| x["changed"] == true)
        .collect();
    r.check(e == [true, true, false], "per-verb effects", || {
        j["effects"].to_string()
    });
    // lean reads
    let o = w.cli(&[
        "--json",
        "ui",
        "state",
        "--fields",
        "tab,level,breadcrumb,view,selected.account_code",
    ]);
    let j = o.json();
    let keys: Vec<&String> = j
        .as_object()
        .map(|m| m.keys().collect())
        .unwrap_or_default();
    r.check(
        keys == [
            "tab",
            "level",
            "breadcrumb",
            "view",
            "selected.account_code",
        ] && j["tab"] == "ledger",
        "--fields",
        || o.stdout.clone(),
    );
    // a bad predicate is a usage error
    let o = w.send(&["--expect", "no operator", "select", "#1"]);
    r.check(
        o.code == 2 && !o.json()["help"].is_null(),
        "bad predicate",
        || o.stdout.clone(),
    );
    let _ = rev;
    Ok(())
}

/// `xtask agent-drive dump-states DIR`: the state on a spread of screens (schema work).
pub fn dump_states(bins: Bins, dir: &str) -> Result<(), String> {
    let mut w = World::new("dump", bins)?;
    w.start(100, 30, &[])?;
    let scripts: &[&[&str]] = &[
        &["tab", "overview"],
        &["tab", "ledger"],
        &["scope", "roast"],
        &["drill"],
        &["drill"],
        &["up", "up", "compare", "roast,hold"],
        &["drill"],
        &["up", "up", "tab", "intercompany"],
        &["drill"],
        &["tab", "activity"],
        &["tab", "system"],
        &["tab", "classify"],
        &["caption", "x"],
        &["filter", "zz"],
    ];
    std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    for (i, s) in scripts.iter().enumerate() {
        w.send(s);
        let o = w.cli(&["--json", "ui", "state"]);
        std::fs::write(format!("{dir}/{i:02}.json"), o.stdout).map_err(|e| e.to_string())?;
    }
    w.keys(b"/");
    let o = w.cli(&["--json", "ui", "state"]);
    std::fs::write(format!("{dir}/99.json"), o.stdout).map_err(|e| e.to_string())?;
    Ok(())
}

/// An error reply follows docs/schema/ui-error.v1.json (and so has a help line).
pub fn check_error(r: &mut Report, name: &str, j: &Value) {
    let code = j["error"].as_str().unwrap_or("");
    r.check(
        ownpurse_core::bridge::UI_ERRORS.iter().any(|e| e.0 == code),
        name,
        || format!("error code `{code}` is not in the documented table"),
    );
    for p in super::schema::check(&super::schema::load("ui-error.v1.json"), j, "error") {
        r.fail(name, format!("error schema: {p} in {j}"));
    }
}

/// `ui wait --until …`: answered by the TUI when the condition holds (no polling), or `timeout` at the deadline.
fn waits(bins: &Bins, r: &mut Report) -> Result<(), String> {
    let mut w = World::new("wait", bins.clone())?;
    w.start(100, 30, &[])?;
    let o = w.cli(&["--json", "ui", "wait", "--until", "idle"]);
    r.check(o.code == 0 && o.json()["held"] == true, "wait idle", || {
        o.stdout.clone()
    });
    // immediate: answered on arrival (the TUI's own count, not the machine's speed)
    r.check(
        o.json()["waited_ms"].as_u64().is_some_and(|ms| ms < 1000),
        "wait idle is immediate",
        || o.stdout.clone(),
    );
    // rev>N: blocks until another agent changes the view
    let rev = st_rev(&w);
    let w = std::sync::Arc::new(w);
    let w2 = w.clone();
    let later = std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(400));
        w2.cli(&["--json", "ui", "send", "tab", "ledger"])
    });
    let o = w.cli(&[
        "--json",
        "ui",
        "wait",
        "--until",
        &format!("rev>{rev}"),
        "--timeout",
        "60",
    ]);
    let _ = later.join();
    let j = o.json();
    r.check(
        o.code == 0 && j["held"] == true && j["summary"]["tab"] == "ledger",
        "wait rev>N",
        || o.stdout.clone(),
    );
    let ms = o.elapsed.as_millis();
    // it blocked until the change (held, not a timeout); no upper bound: a loaded machine is slow
    r.check(ms >= 300, "wait rev>N blocked", || format!("{ms} ms"));
    // a timeout says where things are
    let o = w.cli(&[
        "--json",
        "ui",
        "wait",
        "--until",
        "rev>99999",
        "--timeout",
        "1",
    ]);
    let j = o.json();
    r.check(
        o.code == 4 && j["error"] == "timeout" && j["actual"]["rev"].is_number(),
        "wait timeout",
        || o.stdout.clone(),
    );
    check_error(r, "wait timeout", &j);
    let ms = o.elapsed.as_millis();
    r.check(ms >= 900, "wait timeout not early", || format!("{ms} ms"));
    // store_rev>N: a classify write lands
    let sr = w.state(&[])["classify"]["store_rev"].as_i64().unwrap_or(0);
    let w3 = w.clone();
    let write = std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(300));
        w3.cli(&[
            "--json",
            "classify",
            "set",
            "roast:bt:bt-104:L1",
            "project=harvest",
            "--expect-store-rev",
            &sr.to_string(),
            "--client",
            "claude",
        ])
    });
    let o = w.cli(&[
        "--json",
        "ui",
        "wait",
        "--until",
        &format!("store_rev>{sr}"),
        "--timeout",
        "10",
    ]);
    let wo = write.join().map_err(|_| "writer panicked".to_string())?;
    r.check(wo.code == 0, "write during wait", || wo.stdout.clone());
    r.check(
        o.code == 0 && o.json()["held"] == true,
        "wait store_rev>N",
        || o.stdout.clone(),
    );
    // a bad condition
    let o = w.cli(&["--json", "ui", "wait", "--until", "soon"]);
    r.check(o.code == 2, "wait bad condition", || o.stdout.clone());
    check_error(r, "wait bad condition", &o.json());
    // two waits at once, both answered
    let rev = st_rev(&w);
    let (a, b) = (w.clone(), w.clone());
    let ta = std::thread::spawn(move || {
        a.cli(&[
            "--json",
            "ui",
            "wait",
            "--until",
            &format!("rev>{rev}"),
            "--timeout",
            "10",
        ])
    });
    let tb = std::thread::spawn(move || {
        b.cli(&[
            "--json",
            "ui",
            "wait",
            "--until",
            &format!("rev>={}", rev + 1),
            "--timeout",
            "10",
        ])
    });
    std::thread::sleep(std::time::Duration::from_millis(300));
    let o = w.cli(&["--json", "ui", "send", "tab", "overview"]);
    r.check(o.code == 0, "send during two waits", || o.stdout.clone());
    for (n, t) in [("first", ta), ("second", tb)] {
        let o = t.join().map_err(|_| "waiter panicked".to_string())?;
        r.check(o.code == 0, &format!("{n} concurrent wait"), || {
            o.stdout.clone()
        });
    }
    w.settle();
    verify(&w, r, "after waits");
    Ok(())
}

/// `ui upgrade` across two real builds: the running TUI (a copy of this build) is replaced on disk by a newer
/// build, notices it, and hands over in place with the view kept exactly.
fn upgrade(bins: &Bins, r: &mut Report) -> Result<(), String> {
    let Some(bumped) = bins.bumped.clone() else {
        r.fail(
            "upgrade",
            "no second build (--bumped PATH): the upgrade case did not run",
        );
        return Ok(());
    };
    let mut b = bins.clone();
    let mut w = World::new("upg", b.clone())?;
    let local = w.root.join("bin/ownpurse-tui");
    std::fs::create_dir_all(w.root.join("bin")).map_err(|e| e.to_string())?;
    std::fs::copy(&bins.tui, &local).map_err(|e| e.to_string())?;
    b.tui = local.clone();
    w.bins = b;
    w.start(100, 30, &[])?;
    step(&w, r, &["tab", "ledger", "scope", "roast"], 0);
    step(&w, r, &["select", "1060", "drill"], 0);
    step(&w, r, &["caption", "Card spend before the upgrade"], 0);
    let before = w.state(&[]);
    let pid = w.cli(&["--json", "ui", "ls"]).json()["sessions"][0]["pid"].clone();
    // up to date: nothing newer on disk yet
    let o = w.cli(&["--json", "ui", "upgrade"]);
    r.check(
        o.json()["status"] == "up_to_date",
        "upgrade up to date",
        || o.stdout.clone(),
    );
    // install the newer build over the running binary (as a reinstall does: write aside, rename)
    let tmp = w.root.join("bin/.ownpurse-tui.new");
    std::fs::copy(&bumped, &tmp).map_err(|e| e.to_string())?;
    std::fs::rename(&tmp, &local).map_err(|e| e.to_string())?;
    let st = wait_update(&w);
    r.check(
        st["update_available"]["version"].is_string(),
        "update noticed",
        || format!("{}\n{}", st["update_available"], w.screen().text()),
    );
    let o = w.cli(&["--json", "ui", "upgrade"]);
    let j = o.json();
    r.check(
        o.code == 0 && j["status"] == "upgraded",
        "ui upgrade",
        || o.stdout.clone(),
    );
    w.settle();
    let ls = w.cli(&["--json", "ui", "ls"]).json();
    r.check(
        ls["sessions"][0]["pid"] == pid,
        "same pid after upgrade",
        || ls.to_string(),
    );
    r.check(
        ls["sessions"][0]["version"] != before["version"],
        "new version registered",
        || ls.to_string(),
    );
    let after = verify(&w, r, "after upgrade");
    for k in [
        "tab",
        "level",
        "scope",
        "breadcrumb",
        "selected",
        "filter",
        "as_of",
    ] {
        r.check(after[k] == before[k], &format!("upgrade keeps {k}"), || {
            format!("{} → {}", before[k], after[k])
        });
    }
    r.check(
        after["caption"]["text"] == before["caption"]["text"],
        "upgrade keeps the caption",
        || after["caption"].to_string(),
    );
    step(&w, r, &["up"], 0);
    Ok(())
}

/// `1,234.56` / `(1,234.56)` → the number's text without grouping (`-1234.56`).
fn display_number(t: &str) -> Option<String> {
    let t = t.trim();
    if t.is_empty() {
        return None;
    }
    let neg = t.starts_with('(') && t.ends_with(')');
    let core: String = t
        .trim_matches(['(', ')'])
        .chars()
        .filter(|c| *c != ',')
        .collect();
    core.parse::<rust_decimal::Decimal>()
        .ok()
        .map(|d| if neg { (-d).to_string() } else { d.to_string() })
}

/// One live case per previously fixed issue (the PM's list).
fn regressions(bins: &Bins, r: &mut Report) -> Result<(), String> {
    let mut w = World::new("regr", bins.clone())?;
    w.start(100, 30, &[])?;
    // forward/back with nothing to go to: exit 0, note no_history, the effect says so
    let j = step(&w, r, &["back"], 0);
    r.check(
        j["note"] == "no_history" && j["effects"][0]["changed"] == false,
        "back no_history",
        || j.to_string(),
    );
    let j = step(&w, r, &["forward"], 0);
    r.check(j["note"] == "no_history", "forward no_history", || {
        j.to_string()
    });
    // the filter applies on every screen where it means something (and is refused where it doesn't)
    for (setup, q) in [
        (vec!["tab", "overview"], "design"),
        (vec!["tab", "ledger", "scope", "roast"], "Operating"),
        (vec!["select", "1010", "drill"], "Larkspur"),
        (vec!["up", "tab", "intercompany"], "Tallowmere"),
        (vec!["tab", "activity"], "lab"),
        (vec!["tab", "system"], "roast"),
    ] {
        step(&w, r, &setup, 0);
        let j = step(&w, r, &["filter", q], 0);
        let st = &j["state"];
        r.check(
            st["filter"] == q && st["filter_applies"] == true,
            &format!("filter {q}"),
            || format!("filter {} applies {}", st["filter"], st["filter_applies"]),
        );
        let rows = st["rows"].as_array().cloned().unwrap_or_default();
        r.check(!rows.is_empty(), &format!("filter {q} rows"), || {
            "no rows match".into()
        });
        step(&w, r, &["clear-filter"], 0);
    }
    step(
        &w,
        r,
        &[
            "tab", "ledger", "scope", "roast", "select", "1010", "drill", "drill",
        ],
        0,
    );
    let j = step(&w, r, &["filter", "x"], 2);
    r.check(
        j["error"] == "not_applicable",
        "filter on a document",
        || j.to_string(),
    );
    step(&w, r, &["up", "up"], 0);
    // the _value twins equal their display strings
    let st = w.state(&["--all-rows"]);
    let mut twins = 0;
    for row in st["rows"].as_array().into_iter().flatten() {
        let Some(f) = row["fields"].as_object() else {
            continue;
        };
        for (k, v) in f {
            let Some(base) = k.strip_suffix("_value") else {
                continue;
            };
            let shown = f.get(base).and_then(Value::as_str).unwrap_or("");
            match (display_number(shown), v) {
                (Some(d), Value::Number(n)) => {
                    twins += 1;
                    let want = rust_decimal::Decimal::from_str_exact(&d).ok();
                    let got = rust_decimal::Decimal::from_str_exact(&n.to_string()).ok();
                    r.check(want.is_some() && want == got, "value twin", || {
                        format!("{k}: shown {shown:?} vs {n}")
                    });
                }
                (None, Value::Null) => {}
                (a, b) => r.fail("value twin", format!("{k}: shown {shown:?} ({a:?}) vs {b}")),
            }
        }
    }
    r.check(twins > 10, "value twins present", || {
        format!("only {twins} twins")
    });
    // rows, view, truncated and paging
    let all = w.state(&["--all-rows"]);
    let total = all["view"]["total_rows"].as_u64().unwrap_or(0) as usize;
    r.check(
        all["rows"].as_array().map_or(0, Vec::len) == total,
        "--all-rows",
        || format!("{} of {total}", all["rows"]),
    );
    let page = w.state(&["--rows", "2..4"]);
    let nums: Vec<u64> = page["rows"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|x| x["row"].as_u64())
        .collect();
    r.check(nums == [2, 3, 4], "--rows 2..4", || format!("{nums:?}"));
    // compact text output of a send
    let o = w.cli(&["ui", "send", "select", "#3"]);
    w.settle();
    let lines = o.stdout.lines().count();
    r.check(
        o.code == 0 && o.stdout.contains("rev") && o.stdout.contains("changed") && lines <= 14,
        "compact send output",
        || o.stdout.clone(),
    );
    // label + --session
    step(&w, r, &["label", "close-review"], 0);
    let o = w.cli(&[
        "--json",
        "ui",
        "state",
        "--session",
        "close-review",
        "--fields",
        "tab",
    ]);
    r.check(o.code == 0, "--session by a label set by the agent", || {
        o.stdout.clone()
    });
    // drift: a saved state refuses to draw after a label is written (classify_changed) or the record moves
    let saved = w.root.join("saved.json");
    std::fs::write(&saved, w.cli(&["--json", "ui", "state"]).stdout).map_err(|e| e.to_string())?;
    let sp = saved.display().to_string();
    let sr = w.state(&[])["classify"]["store_rev"].to_string();
    let o = w.cli(&[
        "--json",
        "classify",
        "set",
        "roast:bt:bt-104:L1",
        "project=harvest",
        "--expect-store-rev",
        &sr,
        "--client",
        "claude",
    ]);
    r.check(o.code == 0, "label for drift", || o.stdout.clone());
    let o = w.cli(&[
        "--json",
        "ui",
        "screen",
        "--from-state",
        &sp,
        "--format",
        "text",
    ]);
    r.check(
        o.code == 4 && o.json()["error"] == "classify_changed",
        "classify_changed",
        || o.stdout.clone(),
    );
    check_error(r, "classify_changed", &o.json());
    let o = w.cli(&[
        "--json",
        "ui",
        "screen",
        "--from-state",
        &sp,
        "--format",
        "text",
        "--allow-classify-drift",
    ]);
    r.check(o.code == 0, "--allow-classify-drift", || o.stdout.clone());
    let saved2 = w.root.join("saved2.json");
    std::fs::write(&saved2, w.cli(&["--json", "ui", "state"]).stdout).map_err(|e| e.to_string())?;
    {
        let c = rusqlite::Connection::open(w.work().join("record.sqlite"))
            .map_err(|e| e.to_string())?;
        c.execute(
            "INSERT INTO versions (tenant_id, entity, xero_id, version, content_hash, status, observed_at, source, payload) \
             SELECT tenant_id, entity, xero_id, version + 1, 'drift', status, '2026-10-05T09:50:00Z', 'sync', payload FROM current \
             WHERE entity = 'BankTransactions' AND xero_id = 'bt-104'",
            [],
        )
        .map_err(|e| e.to_string())?;
        // a sync chains every row it appends; the record version is the chain's last seq
        c.execute(
            "INSERT INTO chain (seq, at, tbl, row_id, row_hash, prev_hash, hash) \
             SELECT seq + 1, '2026-10-05T09:50:00Z', 'versions', (SELECT MAX(id) FROM versions), 'drift', hash, 'drift' \
             FROM chain ORDER BY seq DESC LIMIT 1",
            [],
        )
        .map_err(|e| e.to_string())?;
    }
    let o = w.cli(&[
        "--json",
        "ui",
        "screen",
        "--from-state",
        &saved2.display().to_string(),
        "--format",
        "text",
    ]);
    r.check(
        o.code == 4 && o.json()["error"] == "record_changed",
        "record_changed",
        || o.stdout.clone(),
    );
    check_error(r, "record_changed", &o.json());
    Ok(())
}

/// The 50-line cap on agent writes through the TUI: refused atomically (too_many, exit 2, store and screen
/// unchanged), then written with --allow-bulk; human labels survive the bulk write.
fn bulk_cap(bins: &Bins, r: &mut Report) -> Result<(), String> {
    let mut w = World::new("bulk", bins.clone())?;
    w.add_bulk_lines(60)?;
    w.start(100, 30, &[])?;
    step(&w, r, &["tab", "classify"], 0);
    let before = w.state(&[]);
    let screen_before = w.screen().text();
    let sr = before["classify"]["store_rev"].to_string();
    let o = w.cli(&[
        "--json",
        "classify",
        "set",
        "Brindlewick",
        "project=harvest",
        "--expect-store-rev",
        &sr,
        "--client",
        "claude",
    ]);
    w.settle();
    let j = o.json();
    r.check(
        o.code == 2 && j["error"] == "too_many",
        "bulk refused",
        || format!("exit {}: {}", o.code, o.stdout),
    );
    check_error(r, "too_many", &j);
    let after = w.state(&[]);
    r.check(
        after["classify"]["store_rev"] == before["classify"]["store_rev"]
            && after["rev"] == before["rev"],
        "bulk refusal is atomic",
        || format!("{} -> {}", before["classify"], after["classify"]),
    );
    r.check(
        w.screen().text() == screen_before,
        "bulk refusal leaves the screen",
        || w.screen().text(),
    );
    let o = w.cli(&[
        "--json",
        "classify",
        "set",
        "Brindlewick",
        "project=harvest",
        "--expect-store-rev",
        &sr,
        "--client",
        "claude",
        "--allow-bulk",
    ]);
    w.settle();
    let j = o.json();
    r.check(o.code == 0, "--allow-bulk", || {
        format!("exit {}: {}", o.code, o.stdout)
    });
    r.check(
        j["lines"] == 60 && j["effects"][0]["changed"] == true,
        "bulk wrote 60 lines",
        || j.to_string(),
    );
    verify(&w, r, "after bulk write");
    Ok(())
}

/// A sync the user starts (the fake CLI: no network, the real CLI's events, rows appended to the world's
/// record copy when the fake writer is built) while agents keep driving: sends work mid-run, `running` is
/// reported, `ui wait --until running=none` returns when it ends, the record reloads and the screen matches.
fn sync_running(bins: &Bins, r: &mut Report) -> Result<(), String> {
    let mut b = bins.clone();
    let mut w = World::new("sync", b.clone())?;
    let sbin = w.root.join("sbin");
    std::fs::create_dir_all(&sbin).map_err(|e| e.to_string())?;
    let local = sbin.join("ownpurse-tui");
    std::fs::copy(&bins.tui, &local).map_err(|e| e.to_string())?;
    // the TUI runs the CLI next to its own binary: the fake
    std::fs::copy(
        crate::demo::fixtures_dir().join("fake-cli/ownpurse"),
        sbin.join("ownpurse"),
    )
    .map_err(|e| e.to_string())?;
    b.tui = local;
    w.bins = b;
    let tenants: Value = serde_json::from_str(
        &std::fs::read_to_string(w.work().join("tenants.json")).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    w.extra_env = vec![
        ("OWNPURSE_FAKE_SCENARIO".into(), "slow".into()),
        ("OWNPURSE_FAKE_DELAY".into(), "0.12".into()),
        (
            "OWNPURSE_FAKE_MARKER".into(),
            w.root.join("marker").display().to_string(),
        ),
        // the fake writer only writes record copies under the temp dir; the world lives under /tmp
        ("TMPDIR".into(), "/tmp".into()),
    ];
    let writer = bins.cli.with_file_name("ownpurse-fake-record");
    let writes = writer.is_file();
    if writes {
        w.extra_env.extend([
            (
                "OWNPURSE_FAKE_RECORD".to_string(),
                w.work().join("record.sqlite").display().to_string(),
            ),
            (
                "OWNPURSE_FAKE_WRITER".to_string(),
                writer.display().to_string(),
            ),
            (
                "OWNPURSE_FAKE_TENANTS".to_string(),
                tenants["design"].as_str().unwrap_or_default().to_string(),
            ),
        ]);
    } else {
        r.fail(
            "sync record",
            format!("{} is not built: the run appends no rows", writer.display()),
        );
    }
    w.start(100, 30, &[])?;
    let rev0 = w.state(&[])["record"].clone();
    // the user starts the sync from the palette and confirms
    for k in [&b"\x10"[..], b"sync design", b"\r", b"\t\r"] {
        w.keys(k);
    }
    let ok = w.pty().wait_for(std::time::Duration::from_secs(60), |t| {
        t.text().contains("Syncing design")
    });
    r.check(ok, "sync started", || w.screen().text());
    let st = w.state(&[]);
    r.check(st["running"].is_object(), "running reported", || {
        format!("running {}\n{}", st["running"], w.screen().text())
    });
    // agents keep driving mid-run, from two threads, and every send answers
    let w = std::sync::Arc::new(w);
    let mut hs = Vec::new();
    for t in 0..2 {
        let w2 = w.clone();
        hs.push(std::thread::spawn(move || {
            let mut bad = Vec::new();
            for i in 0..10 {
                let verbs: &[&str] = if (i + t) % 2 == 0 {
                    &["tab", "ledger"]
                } else {
                    &["tab", "activity"]
                };
                let mut a = vec!["--json", "ui", "send"];
                a.extend_from_slice(verbs);
                let o = w2.cli(&a);
                // a conflict between the two agents is a correct answer; anything else is not
                if o.code != 0 && o.json()["error"] != "view_changed" {
                    bad.push(format!("exit {}: {}", o.code, o.stdout));
                }
            }
            bad
        }));
    }
    for h in hs {
        let bad = h.join().unwrap_or_else(|_| vec!["thread panicked".into()]);
        r.check(bad.is_empty(), "sends mid-sync", || bad.join("\n"));
    }
    let mid = w.state(&[]);
    r.check(
        mid["running"].is_object(),
        "still running after the sends",
        || format!("running {}", mid["running"]),
    );
    let o = w.cli(&[
        "--json",
        "ui",
        "wait",
        "--until",
        "running=none",
        "--timeout",
        "60",
    ]);
    let j = o.json();
    r.check(
        o.code == 0 && j["held"] == true,
        "wait running=none",
        || o.stdout.clone(),
    );
    w.settle();
    let marker = std::fs::read_to_string(w.root.join("marker")).unwrap_or_default();
    r.check(marker.trim() == "done slow", "the run completed", || {
        marker.clone()
    });
    let st = verify(&w, r, "after the sync");
    r.check(st["running"].is_null(), "running cleared", || {
        st["running"].to_string()
    });
    if writes {
        r.check(
            st["record"] != rev0,
            "record reloaded after the sync",
            || format!("{rev0} → {}", st["record"]),
        );
    }
    let screen = w.screen().text();
    r.check(
        screen.contains("Synced design"),
        "sync result shown",
        || screen.clone(),
    );
    Ok(())
}

/// Split one shell line into words (double and single quotes; no expansion), dropping a trailing comment.
fn shell_words(line: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut quote: Option<char> = None;
    let mut any = false;
    let mut prev_space = true;
    for c in line.chars() {
        match quote {
            Some(q) if c == q => quote = None,
            Some(_) => cur.push(c),
            None if c == '"' || c == '\'' => {
                quote = Some(c);
                any = true;
            }
            None if c == '#' && prev_space => break,
            None if c.is_whitespace() => {
                if any || !cur.is_empty() {
                    out.push(std::mem::take(&mut cur));
                }
                any = false;
            }
            None => cur.push(c),
        }
        prev_space = c.is_whitespace() && quote.is_none();
    }
    if any || !cur.is_empty() {
        out.push(cur);
    }
    out
}

/// Every example in the agent skill snippet (docs/agent-bridge-skill-snippet.md), run live in order against a
/// running TUI: each must work as written (only `--expect-rev N` is replaced by the rev just read, and output
/// files go to the world's folder).
fn snippet(bins: &Bins, r: &mut Report) -> Result<(), String> {
    let doc = std::fs::read_to_string(
        crate::demo::fixtures_dir().join("../../../docs/agent-bridge-skill-snippet.md"),
    )
    .map_err(|e| format!("skill snippet: {e}"))?;
    let block = doc
        .split("```bash")
        .nth(1)
        .and_then(|b| b.split("```").next())
        .ok_or("the skill snippet has no bash block")?;
    let mut w = World::new("snip", bins.clone())?;
    w.start(100, 30, &[])?;
    let mut ran = 0;
    for line in block.lines() {
        let words = shell_words(line);
        if words.first().map(String::as_str) != Some("ownpurse") {
            continue;
        }
        let rev = st_rev(&w).to_string();
        let mut args: Vec<String> = Vec::new();
        let mut it = words[1..].iter();
        while let Some(a) = it.next() {
            args.push(a.clone());
            if a == "--expect-rev" {
                it.next();
                args.push(rev.clone());
            } else if a == "--out" {
                let f = it.next().cloned().unwrap_or_default();
                args.push(w.root.join(f).display().to_string());
            }
        }
        let refs: Vec<&str> = args.iter().map(String::as_str).collect();
        let o = w.cli(&refs);
        w.settle();
        ran += 1;
        r.steps += 1;
        r.check(
            o.code == 0,
            &format!(
                "snippet: {}",
                line.split("  #").next().unwrap_or(line).trim()
            ),
            || format!("exit {}: {}{}", o.code, o.stdout, o.stderr),
        );
        if refs.contains(&"send") {
            verify(&w, r, "snippet send");
        }
    }
    r.check(ran >= 8, "snippet examples ran", || format!("only {ran}"));
    Ok(())
}

/// Another process writes while the TUI is open (§7.9): a CLI classify write that bypasses the TUI
/// (`--direct`), a `classify suggest`, and a sync from another terminal (the fake writer appending to the
/// record). Each reaches the TUI by the file event: the state's revisions move, the rows follow, the screen
/// matches and says what arrived; idle afterwards draws nothing.
fn external(bins: &Bins, r: &mut Report) -> Result<(), String> {
    let mut w = World::new("ext", bins.clone())?;
    w.add_bulk_lines(12)?;
    w.start(100, 30, &[])?;
    step(&w, r, &["tab", "classify"], 0);
    let st = w.state(&[]);
    let sr = st["classify"]["store_rev"].as_i64().unwrap_or(0);
    // 1. a direct write (the store only; the TUI is not asked)
    let o = w.cli(&[
        "--json",
        "classify",
        "set",
        "roast:bt:bt-bulk:L3",
        "project=harvest",
        "--expect-store-rev",
        &sr.to_string(),
        "--client",
        "other",
        "--direct",
    ]);
    r.check(o.code == 0, "direct classify write", || {
        format!("exit {}: {}", o.code, o.stdout)
    });
    let o = w.cli(&[
        "--json",
        "ui",
        "wait",
        "--until",
        &format!("store_rev>{sr}"),
        "--timeout",
        "60",
    ]);
    r.check(
        o.code == 0 && o.json()["held"] == true,
        "the TUI noticed the store write",
        || o.stdout.clone(),
    );
    w.settle();
    let st = verify(&w, r, "after an external store write");
    let sr2 = st["classify"]["store_rev"].as_i64().unwrap_or(0);
    r.check(sr2 > sr, "store_rev moved", || format!("{sr} → {sr2}"));
    r.check(st["tab"] == "classify", "the view is kept", || {
        st["tab"].to_string()
    });
    let screen = w.screen().text();
    r.check(screen.contains("labels updated"), "store notice", || {
        screen.clone()
    });
    // the queue rows follow (the labelled line is no longer open, or shows its label)
    let all = w.state(&["--all-rows"]);
    let line = all["rows"].as_array().into_iter().flatten().find(|x| {
        x["anchor"].to_string().contains("bt-bulk") && x["anchor"].to_string().contains("L3")
    });
    r.check(
        line.is_none_or(|l| l.to_string().contains("Harvest") || l.to_string().contains("harvest")),
        "queue row follows",
        || format!("{line:?}"),
    );
    // 2. classify suggest from another terminal
    let o = w.cli(&["--json", "classify", "suggest"]);
    r.check(o.code == 0, "classify suggest", || {
        format!("exit {}: {}{}", o.code, o.stdout, o.stderr)
    });
    let after = w
        .cli(&["--json", "classify", "queue", "--limit", "1"])
        .json()["store_rev"]
        .as_i64()
        .unwrap_or(0);
    r.check(after > sr2, "suggest wrote suggestions", || {
        format!("store_rev {sr2} → {after}")
    });
    {
        let o = w.cli(&[
            "--json",
            "ui",
            "wait",
            "--until",
            &format!("store_rev>={after}"),
            "--timeout",
            "60",
        ]);
        r.check(
            o.code == 0 && o.json()["held"] == true,
            "the TUI noticed the suggestions",
            || o.stdout.clone(),
        );
        w.settle();
        verify(&w, r, "after classify suggest");
        let screen = w.screen().text();
        r.check(
            screen.contains("new suggestion"),
            "suggestion notice",
            || screen.clone(),
        );
    }
    // 3. a sync from another terminal: the fake writer appends a run's rows to the record
    let writer = bins.cli.with_file_name("ownpurse-fake-record");
    if !writer.is_file() {
        r.fail(
            "external record",
            format!("{} is not built", writer.display()),
        );
        return Ok(());
    }
    let tenants: Value = serde_json::from_str(
        &std::fs::read_to_string(w.work().join("tenants.json")).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    let rec = w.state(&[])["record"]["chain_seq"].as_i64().unwrap_or(0);
    let out = std::process::Command::new(&writer)
        .args([
            w.work().join("record.sqlite").display().to_string(),
            "ok".into(),
            tenants["design"].as_str().unwrap_or_default().to_string(),
            "2026-10-05T09:47:00Z".into(),
        ])
        .env("TMPDIR", "/tmp")
        .output()
        .map_err(|e| e.to_string())?;
    r.check(out.status.success(), "external sync wrote", || {
        String::from_utf8_lossy(&out.stderr).into_owned()
    });
    let o = w.cli(&[
        "--json",
        "ui",
        "wait",
        "--until",
        &format!("record>{rec}"),
        "--timeout",
        "60",
    ]);
    r.check(
        o.code == 0 && o.json()["held"] == true,
        "the TUI noticed the record write",
        || o.stdout.clone(),
    );
    w.settle();
    let st = verify(&w, r, "after an external sync");
    r.check(
        st["record"]["chain_seq"].as_i64().unwrap_or(0) > rec,
        "record version moved",
        || st["record"].to_string(),
    );
    r.check(
        st["tab"] == "classify",
        "the view is kept after the reload",
        || st["tab"].to_string(),
    );
    let screen = w.screen().text();
    r.check(screen.contains("record updated"), "record notice", || {
        screen.clone()
    });
    // V13: nothing changed since → nothing drawn (the minute tick may redraw ages; within a second, nothing)
    w.settle();
    let before = w.pty.as_ref().map_or(0, |p| p.bytes());
    std::thread::sleep(std::time::Duration::from_millis(800));
    let after = w.pty.as_ref().map_or(0, |p| p.bytes());
    r.check(
        after == before,
        "idle after the refresh draws nothing",
        || format!("{} bytes", after - before),
    );
    // the TUI's own (routed) write doesn't trigger a second refresh or notice
    let sr3 = w.state(&[])["classify"]["store_rev"].to_string();
    let o = w.cli(&[
        "--json",
        "classify",
        "set",
        "roast:bt:bt-bulk:L4",
        "project=harvest",
        "--expect-store-rev",
        &sr3,
        "--client",
        "claude",
    ]);
    r.check(o.code == 0, "routed write", || o.stdout.clone());
    w.settle();
    let screen = w.screen().text();
    r.check(
        !screen.contains("labels updated"),
        "own write is not announced as external",
        || screen.clone(),
    );
    Ok(())
}

/// §7.10: a CLI classify write after the TUI started, then `ui upgrade` straight away: upgraded (never
/// classify_changed), the view kept, the new store_rev in the state.
fn upgrade_after_external(bins: &Bins, r: &mut Report) -> Result<(), String> {
    let Some(bumped) = bins.bumped.clone() else {
        r.fail("upgrade after external", "no second build (--bumped PATH)");
        return Ok(());
    };
    let mut b = bins.clone();
    let mut w = World::new("upgx", b.clone())?;
    let local = w.root.join("bin/ownpurse-tui");
    std::fs::create_dir_all(w.root.join("bin")).map_err(|e| e.to_string())?;
    std::fs::copy(&bins.tui, &local).map_err(|e| e.to_string())?;
    b.tui = local.clone();
    w.bins = b;
    w.start(100, 30, &[])?;
    step(
        &w,
        r,
        &["tab", "ledger", "scope", "roast", "select", "1060", "drill"],
        0,
    );
    let before = w.state(&[]);
    let tmp = w.root.join("bin/.ownpurse-tui.new");
    std::fs::copy(&bumped, &tmp).map_err(|e| e.to_string())?;
    std::fs::rename(&tmp, &local).map_err(|e| e.to_string())?;
    let sr = before["classify"]["store_rev"].as_i64().unwrap_or(0);
    let o = w.cli(&[
        "--json",
        "classify",
        "set",
        "roast:bt:bt-104:L1",
        "project=harvest",
        "--expect-store-rev",
        &sr.to_string(),
        "--client",
        "other",
        "--direct",
    ]);
    r.check(o.code == 0, "direct write before upgrade", || {
        o.stdout.clone()
    });
    let o = w.cli(&["--json", "ui", "upgrade"]);
    let j = o.json();
    r.check(
        o.code == 0 && j["status"] == "upgraded",
        "upgrade after an external write",
        || o.stdout.clone(),
    );
    w.settle();
    let after = verify(&w, r, "after upgrade with external write");
    for k in ["tab", "level", "scope", "breadcrumb", "selected"] {
        r.check(after[k] == before[k], &format!("upgrade keeps {k}"), || {
            format!("{} → {}", before[k], after[k])
        });
    }
    r.check(
        after["classify"]["store_rev"].as_i64().unwrap_or(0) > sr,
        "new store_rev after upgrade",
        || after["classify"].to_string(),
    );
    r.check(
        after["drift"].is_null(),
        "no drift marker after a live upgrade",
        || after["drift"].to_string(),
    );
    Ok(())
}

/// A config change, then `ui upgrade` straight away (before or after the watcher's re-read): the old build
/// picks the change up first, so the upgrade goes ahead on the new config (a removed org in scope falls back
/// to all). The race left (a save between export and check) still fails with config_changed: unit-tested.
fn upgrade_after_config_change(bins: &Bins, r: &mut Report) -> Result<(), String> {
    let Some(bumped) = bins.bumped.clone() else {
        r.fail(
            "upgrade after config change",
            "no second build (--bumped PATH)",
        );
        return Ok(());
    };
    let mut b = bins.clone();
    let mut w = World::new("upgo", b.clone())?;
    let local = w.root.join("bin/ownpurse-tui");
    std::fs::create_dir_all(w.root.join("bin")).map_err(|e| e.to_string())?;
    std::fs::copy(&bins.tui, &local).map_err(|e| e.to_string())?;
    b.tui = local.clone();
    w.bins = b;
    w.start(100, 30, &[])?;
    step(&w, r, &["tab", "ledger", "scope", "lab"], 0);
    edit_config(&w, |c| remove_org(c, "lab"))?;
    let tmp = w.root.join("bin/.ownpurse-tui.new");
    std::fs::copy(&bumped, &tmp).map_err(|e| e.to_string())?;
    std::fs::rename(&tmp, &local).map_err(|e| e.to_string())?;
    let o = w.cli(&["--json", "ui", "upgrade"]);
    r.check(
        o.code == 0 && o.json()["status"] == "upgraded",
        "upgrade on the new config",
        || o.stdout.clone(),
    );
    w.settle();
    let after = verify(&w, r, "after upgrade on a new config");
    r.check(
        after["scope"] == "all" && after["tab"] == "ledger",
        "scope fell back to all, tab kept",
        || format!("{} {}", after["scope"], after["tab"]),
    );
    Ok(())
}

/// The Classify queue in the TUI: stale lines first, then the largest amount (Sort::StaleFirst, as the CLI);
/// `status:stale` lists exactly the stale lines, and the counts agree with the rows.
fn stale_queue(bins: &Bins, r: &mut Report) -> Result<(), String> {
    let mut w = World::new("stale", bins.clone())?;
    w.start(100, 30, &[])?;
    step(&w, r, &["tab", "classify"], 0);
    let st = w.state(&["--all-rows"]);
    let rows: Vec<Value> = st["rows"].as_array().cloned().unwrap_or_default();
    let stale = |x: &Value| x["fields"]["stale"] == true;
    let amount = |x: &Value| {
        rust_decimal::Decimal::from_str_exact(&x["fields"]["amount_value"].to_string())
            .map(|d| d.abs())
            .unwrap_or_default()
    };
    let n_stale = rows.iter().filter(|x| stale(x)).count();
    r.check(n_stale > 0, "the demo queue has stale lines", || {
        format!("{} rows", rows.len())
    });
    r.check(
        rows.iter().take(n_stale).all(stale),
        "stale lines first",
        || {
            rows.iter()
                .map(|x| x["fields"]["stale"].to_string())
                .collect::<Vec<_>>()
                .join(",")
        },
    );
    let ordered = rows
        .windows(2)
        .all(|p| stale(&p[0]) != stale(&p[1]) || amount(&p[0]) >= amount(&p[1]));
    r.check(ordered, "then the largest amount first", || {
        rows.iter()
            .map(|x| x["fields"]["amount"].to_string())
            .collect::<Vec<_>>()
            .join(",")
    });
    r.check(
        st["selected"]["anchor"]
            == rows
                .first()
                .map_or(Value::Null, |x| x["anchor"]["anchor"].clone()),
        "the cursor starts on the first stale line",
        || st["selected"].to_string(),
    );
    let header_stale = st["classify"]["stale"].as_u64().unwrap_or(0) as usize;
    r.check(
        header_stale == n_stale,
        "header stale count = stale rows",
        || format!("{header_stale} vs {n_stale}"),
    );
    // status:stale: exactly those, rows and counts alike
    let j = step(&w, r, &["filter", "status:stale"], 0);
    let st = w.state(&["--all-rows"]);
    let total = st["view"]["total_rows"].as_u64().unwrap_or(0) as usize;
    let got: Vec<Value> = st["rows"].as_array().cloned().unwrap_or_default();
    r.check(
        total == n_stale && got.iter().all(stale),
        "status:stale lists the stale lines",
        || format!("{total} rows: {j}"),
    );
    let screen = w.screen().text();
    r.check(
        screen.contains(&format!("{n_stale} stale")),
        "the header still counts them",
        || screen.clone(),
    );
    step(&w, r, &["clear-filter"], 0);
    let st = w.state(&[]);
    r.check(
        st["view"]["total_rows"].as_u64() == Some(rows.len() as u64),
        "clear-filter brings the queue back",
        || st["view"].to_string(),
    );
    Ok(())
}

/// A write lands while the TUI runs (the production run engine against the loopback fake Xero, appending to
/// the world's record): the TUI notices by the file event, says what landed, Activity lists the attempts and
/// System lists the package with its stage; the screen matches the state throughout.
fn writes(bins: &Bins, r: &mut Report) -> Result<(), String> {
    let writer = bins.cli.with_file_name("ownpurse-fake-record");
    if !writer.is_file() {
        r.fail("writes", format!("{} is not built", writer.display()));
        return Ok(());
    }
    let mut w = World::new("writes", bins.clone())?;
    w.start(100, 30, &[])?;
    step(&w, r, &["tab", "activity"], 0);
    let rec = w.state(&[])["record"]["chain_seq"].as_i64().unwrap_or(0);
    let out = std::process::Command::new(&writer)
        .args([
            w.work().join("record.sqlite").display().to_string(),
            "write".into(),
            "c0ffee00-0000-4000-8000-0000000000a1".into(),
            "2026-10-05T09:45:00Z".into(),
        ])
        .env("TMPDIR", "/tmp")
        .output()
        .map_err(|e| e.to_string())?;
    r.check(out.status.success(), "a write appended", || {
        String::from_utf8_lossy(&out.stderr).into_owned()
    });
    let o = w.cli(&[
        "--json",
        "ui",
        "wait",
        "--until",
        &format!("record>{rec}"),
        "--timeout",
        "60",
    ]);
    r.check(
        o.code == 0 && o.json()["held"] == true,
        "the TUI noticed the write",
        || o.stdout.clone(),
    );
    w.settle();
    let st = verify(&w, r, "after a write landed");
    let screen = w.screen().text();
    r.check(
        screen.contains("write: 1 op landed · XA:fixture-write-01"),
        "write notice",
        || screen.lines().last().unwrap_or("").to_string(),
    );
    let rows = w.state(&["--all-rows"]);
    let attempts = rows["rows"]
        .as_array()
        .into_iter()
        .flatten()
        .filter(|x| x.to_string().contains("create_journal"))
        .count();
    r.check(
        attempts == 2,
        "two attempt rows (sending, confirmed)",
        || format!("{attempts}: {}", st["rows"]),
    );
    step(&w, r, &["tab", "system", "filter", "Owner"], 0);
    let screen = w.screen().text();
    r.check(
        screen.contains("Writes") && screen.contains("landed"),
        "System lists the package",
        || screen.clone(),
    );
    // v0.13.1: select by the row's own anchor (key=value form), drill lists the ops and says so, a drill
    // that means nothing is refused, up folds them
    let st = w.state(&["--all-rows"]);
    let key = st["rows"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|x| x["anchor"]["system_item"].as_str())
        .find(|k| k.starts_with("pkg:"))
        .unwrap_or("")
        .to_string();
    r.check(!key.is_empty(), "the package row's anchor", || {
        st["rows"].to_string()
    });
    step(
        &w,
        r,
        &["clear-filter", "select", &format!("system_item={key}")],
        0,
    );
    let j = step(&w, r, &["drill"], 0);
    r.check(
        j["changed"][0]
            .as_str()
            .is_some_and(|c| c.starts_with("listed the 1 operation of")),
        "drill says it listed the operations",
        || j.to_string(),
    );
    let st = w.state(&[]);
    let screen = w.screen().text();
    r.check(
        st["level"] == 2 && screen.contains("› op-1 create_journal"),
        "the operations are on screen at level 2",
        || format!("level {} · {screen}", st["level"]),
    );
    let j = step(&w, r, &["drill"], 2);
    r.check(
        j["error"] == "not_applicable",
        "drill on an operation is refused",
        || j.to_string(),
    );
    let j = step(&w, r, &["up"], 0);
    r.check(
        j["summary"]["level"] == 1 && j["summary"]["selected"]["system_item"] == key.as_str(),
        "up folds them, back on the package",
        || j.to_string(),
    );
    Ok(())
}

/// The update check after a reinstall: each `ui state` is an interaction that re-reads the installed build
/// (bounded; a first exec of a fresh binary can be slow while the OS checks it), so poll for up to 20 s.
fn wait_update(w: &World) -> Value {
    let end = std::time::Instant::now() + std::time::Duration::from_secs(20);
    loop {
        let st = w.state(&[]);
        if st["update_available"]["version"].is_string() || std::time::Instant::now() > end {
            return st;
        }
        std::thread::sleep(std::time::Duration::from_millis(250));
    }
}

/// Two dev builds of the same version (different commits): the newer install is noticed and `ui upgrade`
/// takes it (state and `ui ls` show the build id); the same build again is up to date.
fn upgrade_same_version(bins: &Bins, r: &mut Report) -> Result<(), String> {
    let Some(other) = bins.same_version.clone() else {
        r.fail(
            "upgrade same version",
            "no second build of the same version (--same-version PATH)",
        );
        return Ok(());
    };
    // the case needs a genuinely different build id, whatever the git state (the gate bakes `gfeed001`)
    let ver = |p: &std::path::Path| {
        std::process::Command::new(p)
            .arg("--version")
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
            .unwrap_or_default()
    };
    let (mine, theirs) = (ver(&bins.tui), ver(&other));
    if !theirs.ends_with("+gfeed001") || mine == theirs {
        r.fail(
            "upgrade same version",
            format!("the second build must be the same version with build id gfeed001 (OWNPURSE_BUILD_ID=gfeed001): {mine:?} vs {theirs:?}"),
        );
        return Ok(());
    }
    let mut b = bins.clone();
    let mut w = World::new("upgb", b.clone())?;
    let local = w.root.join("bin/ownpurse-tui");
    std::fs::create_dir_all(w.root.join("bin")).map_err(|e| e.to_string())?;
    std::fs::copy(&bins.tui, &local).map_err(|e| e.to_string())?;
    b.tui = local.clone();
    w.bins = b;
    w.start(100, 30, &[])?;
    step(&w, r, &["tab", "ledger", "scope", "roast"], 0);
    let before = w.state(&[]);
    let o = w.cli(&["--json", "ui", "upgrade"]);
    r.check(
        o.json()["status"] == "up_to_date",
        "the same build is up to date",
        || o.stdout.clone(),
    );
    // a reinstall of the IDENTICAL build (a new file, same version and build id): still up to date, and
    // the running TUI keeps answering (never a handover to itself)
    let tmp = w.root.join("bin/.ownpurse-tui.new");
    std::fs::copy(&bins.tui, &tmp).map_err(|e| e.to_string())?;
    std::fs::rename(&tmp, &local).map_err(|e| e.to_string())?;
    let o = w.cli(&["--json", "ui", "upgrade"]);
    r.check(
        o.code == 0 && o.json()["status"] == "up_to_date",
        "a reinstalled identical build is up to date",
        || o.stdout.clone(),
    );
    let st = verify(&w, r, "after asking to upgrade to the identical build");
    r.check(st["tab"] == "ledger", "the TUI still answers", || {
        st.to_string()
    });
    let tmp = w.root.join("bin/.ownpurse-tui.new");
    std::fs::copy(&other, &tmp).map_err(|e| e.to_string())?;
    std::fs::rename(&tmp, &local).map_err(|e| e.to_string())?;
    let st = wait_update(&w);
    r.check(
        st["update_available"]["version"]
            .as_str()
            .is_some_and(|v| v.contains("+gfeed001")),
        "a different build of the same version is noticed",
        || st["update_available"].to_string(),
    );
    let o = w.cli(&["--json", "ui", "upgrade"]);
    r.check(
        o.code == 0 && o.json()["status"] == "upgraded",
        "ui upgrade to the other build",
        || o.stdout.clone(),
    );
    w.settle();
    let after = verify(&w, r, "after a same-version upgrade");
    r.check(
        after["version"] == before["version"] && after["build"] == "gfeed001",
        "same version, new build",
        || {
            format!(
                "{} {} → {} {}",
                before["version"], before["build"], after["version"], after["build"]
            )
        },
    );
    let ls = w.cli(&["--json", "ui", "ls"]).json();
    r.check(
        ls["sessions"][0]["version"]
            .as_str()
            .is_some_and(|v| v.ends_with("+gfeed001")),
        "ui ls shows the build",
        || ls.to_string(),
    );
    let screen = w.screen().text();
    r.check(
        screen.contains("+gfeed001"),
        "the updated note names the builds",
        || screen.lines().last().unwrap_or("").to_string(),
    );
    Ok(())
}

/// Below the minimum size the screen shows only the size warning: view-changing sends and classify writes are
/// refused (too_small, exit 4, nothing changed); reads, screenshots and cosmetic verbs still answer.
fn too_small(bins: &Bins, r: &mut Report) -> Result<(), String> {
    let mut w = World::new("small", bins.clone())?;
    w.start(100, 30, &[])?;
    step(&w, r, &["tab", "ledger", "scope", "roast"], 0);
    w.pty().resize(50, 15).map_err(|e| e.to_string())?;
    w.cols = 50;
    w.rows = 15;
    w.settle();
    let ok = w.pty().wait_for(std::time::Duration::from_secs(10), |t| {
        t.text().contains("needs at least")
    });
    r.check(ok, "size warning shown", || w.screen().text());
    let before = w.state(&[]);
    r.check(
        before["size"]["cols"] == 50,
        "state reads at a small size",
        || before["size"].to_string(),
    );
    let j = step(&w, r, &["select", "#3"], 4);
    r.check(
        j["error"] == "too_small" && j["actual"]["cols"] == 50 && j["expected"]["cols"] == 60,
        "send refused too_small",
        || j.to_string(),
    );
    let sr = before["classify"]["store_rev"].to_string();
    let o = w.cli(&[
        "--json",
        "classify",
        "set",
        "roast:bt:bt-104:L1",
        "project=harvest",
        "--expect-store-rev",
        &sr,
        "--client",
        "claude",
    ]);
    let jc = o.json();
    r.check(
        o.code == 4 && jc["error"] == "too_small",
        "classify write refused too_small",
        || o.stdout.clone(),
    );
    check_error(r, "too_small", &jc);
    let after = w.state(&[]);
    r.check(
        after["rev"] == before["rev"]
            && after["classify"]["store_rev"] == before["classify"]["store_rev"]
            && after["selected"] == before["selected"],
        "nothing changed",
        || format!("{} → {}", before["selected"], after["selected"]),
    );
    let o = w.cli(&["--json", "ui", "screen", "--format", "text"]);
    r.check(o.code == 0, "screenshot at a small size", || {
        o.stdout.clone()
    });
    let o = w.cli(&["ui", "send", "select", "#3"]);
    r.check(
        o.code == 4 && format!("{}{}", o.stdout, o.stderr).contains("too_small"),
        "text-mode refusal",
        || o.stdout.clone(),
    );
    // back to a usable size: the same send works
    w.pty().resize(100, 30).map_err(|e| e.to_string())?;
    w.cols = 100;
    w.rows = 30;
    w.settle();
    step(&w, r, &["select", "#3"], 0);
    Ok(())
}

/// A sync in another terminal writes in bursts: the TUI coalesces them (at most two refreshes per burst, no
/// timer), ends on the final record version with the right rows, and is idle (0 bytes) afterwards.
fn burst(bins: &Bins, r: &mut Report) -> Result<(), String> {
    let mut w = World::new("burst", bins.clone())?;
    w.start(100, 30, &[])?;
    step(
        &w,
        r,
        &["tab", "ledger", "scope", "roast", "select", "1010", "drill"],
        0,
    );
    let before = w.state(&[]);
    let n0 = before["data_refreshes"].as_u64().unwrap_or(0);
    let last = w.sync_pages(25)?;
    let o = w.cli(&[
        "--json",
        "ui",
        "wait",
        "--until",
        &format!("record>={last}"),
        "--timeout",
        "60",
    ]);
    r.check(
        o.code == 0 && o.json()["held"] == true,
        "the burst reached the TUI",
        || o.stdout.clone(),
    );
    w.settle();
    let st = verify(&w, r, "after a burst");
    let n = st["data_refreshes"].as_u64().unwrap_or(0) - n0;
    r.check(
        (1..=2).contains(&n),
        "at most two refreshes per burst",
        || format!("{n} refreshes"),
    );
    r.check(
        st["record"]["chain_seq"].as_i64() == Some(last),
        "the final record version",
        || st["record"].to_string(),
    );
    r.check(
        st["breadcrumb"] == before["breadcrumb"],
        "the view is kept",
        || st["breadcrumb"].to_string(),
    );
    // the account's rows show the last page's amount (4,276.31 + 24 × 13.37 = 4,597.19)
    let all = w.state(&["--all-rows"]);
    let shows = all["rows"]
        .as_array()
        .into_iter()
        .flatten()
        .any(|x| x.to_string().contains("4,597.19"));
    r.check(shows, "rows from the last page", || all["rows"].to_string());
    // idle: nothing drawn
    w.settle();
    let b0 = w.pty.as_ref().map_or(0, |p| p.bytes());
    std::thread::sleep(std::time::Duration::from_millis(800));
    let b1 = w.pty.as_ref().map_or(0, |p| p.bytes());
    r.check(b1 == b0, "idle after the burst draws nothing", || {
        format!("{} bytes", b1 - b0)
    });
    Ok(())
}

fn edit_config(w: &World, f: impl FnOnce(&mut Value)) -> Result<(), String> {
    let cfg = w.work().join("ownpurse.json");
    let mut c: Value =
        serde_json::from_str(&std::fs::read_to_string(&cfg).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
    f(&mut c);
    // as an editor saves: write aside, rename over
    let tmp = w.work().join(".ownpurse.json.tmp");
    std::fs::write(
        &tmp,
        serde_json::to_string_pretty(&c).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    std::fs::rename(&tmp, &cfg).map_err(|e| e.to_string())
}

fn remove_org(c: &mut Value, alias: &str) {
    if let Some(o) = c["profiles"]["demo"]["orgs"].as_object_mut() {
        o.remove(alias);
    }
    if let Some(gs) = c["groups"].as_object_mut() {
        for g in gs.values_mut() {
            if let Some(a) = g["orgs"].as_array_mut() {
                a.retain(|x| x != alias);
            }
        }
    }
}

/// `ownpurse.json` saved while the TUI runs: valid changes apply with the view kept and a notice; a removed
/// org in scope falls back to all; an invalid file keeps the last valid config behind an error band, which a
/// valid save clears.
fn config_reload(bins: &Bins, r: &mut Report) -> Result<(), String> {
    let mut w = World::new("cfg", bins.clone())?;
    w.start(100, 30, &[])?;
    step(
        &w,
        r,
        &["tab", "ledger", "scope", "roast", "select", "1060"],
        0,
    );
    let before = w.state(&[]);
    let n0 = before["data_refreshes"].as_u64().unwrap_or(0);
    let wait_refresh = |w: &World, n: u64| -> Value {
        let start = std::time::Instant::now();
        loop {
            let st = w.state(&[]);
            if st["data_refreshes"].as_u64().unwrap_or(0) >= n || start.elapsed().as_secs() > 60 {
                w.settle();
                return st;
            }
            std::thread::sleep(std::time::Duration::from_millis(20));
        }
    };
    // 1. a new group and another accent for roast: applied, view kept
    let accent_before = before["accent"].clone();
    let other = if accent_before.to_string().contains("harbor") {
        "ember"
    } else {
        "harbor"
    };
    edit_config(&w, |c| {
        c["groups"]["duo"] = serde_json::json!({"orgs": ["roast", "hold"]});
        c["tui"]["accents"] = serde_json::json!({"roast": other});
    })?;
    wait_refresh(&w, n0 + 1);
    let st = verify(&w, r, "after a config save");
    r.check(
        st["scope"] == before["scope"]
            && st["selected"] == before["selected"]
            && st["breadcrumb"] == before["breadcrumb"],
        "the view is kept",
        || {
            format!(
                "{} {} → {} {}",
                before["scope"], before["selected"], st["scope"], st["selected"]
            )
        },
    );
    r.check(st["accent"] != accent_before, "the accent applies", || {
        format!("{accent_before} → {}", st["accent"])
    });
    let screen = w.screen().text();
    r.check(screen.contains("config reloaded"), "config notice", || {
        screen.clone()
    });
    step(&w, r, &["scope", "group:duo"], 0);
    // 2. the org in scope is removed: scope → all, said so
    step(&w, r, &["scope", "lab"], 0);
    let n1 = w.state(&[])["data_refreshes"].as_u64().unwrap_or(0);
    edit_config(&w, |c| remove_org(c, "lab"))?;
    wait_refresh(&w, n1 + 1);
    let st = verify(&w, r, "after removing the org in scope");
    r.check(
        st["scope"] == "all" && st["tab"] == "ledger",
        "scope falls back to all",
        || format!("{} {}", st["scope"], st["tab"]),
    );
    let screen = w.screen().text();
    r.check(screen.contains("lab is gone"), "fallback notice", || {
        screen.clone()
    });
    let j = step(&w, r, &["scope", "lab"], 2);
    r.check(
        j["error"].is_string(),
        "the removed org is no longer a scope",
        || j.to_string(),
    );
    // 3. an invalid file: the last valid config stays, behind an error band
    let n2 = w.state(&[])["data_refreshes"].as_u64().unwrap_or(0);
    let cfg = w.work().join("ownpurse.json");
    let good = std::fs::read_to_string(&cfg).map_err(|e| e.to_string())?;
    std::fs::write(&cfg, "{ \"profiles\": ").map_err(|e| e.to_string())?;
    let ok = w.pty().wait_for(std::time::Duration::from_secs(30), |t| {
        t.text().contains("previous config stays live")
    });
    r.check(ok, "error band for an invalid config", || w.screen().text());
    // (no --from-state render here: a file render reads the config, which is invalid on disk)
    let st = w.state(&[]);
    r.check(
        st["data_refreshes"].as_u64() == Some(n2),
        "nothing reloaded from an invalid file",
        || st["data_refreshes"].to_string(),
    );
    let o = w.send(&["scope", "group:duo"]);
    r.check(o.code == 0, "the last valid config keeps working", || {
        o.stdout.clone()
    });
    // a valid save clears the band
    std::fs::write(&cfg, &good).map_err(|e| e.to_string())?;
    let ok = w.pty().wait_for(std::time::Duration::from_secs(30), |t| {
        !t.text().contains("previous config stays live")
    });
    r.check(ok, "a valid save clears the band", || w.screen().text());
    w.settle();
    verify(&w, r, "after fixing the config");
    // idle: nothing drawn
    let b0 = w.pty.as_ref().map_or(0, |p| p.bytes());
    std::thread::sleep(std::time::Duration::from_millis(800));
    let b1 = w.pty.as_ref().map_or(0, |p| p.bytes());
    r.check(b1 == b0, "idle after config reloads draws nothing", || {
        format!("{} bytes", b1 - b0)
    });
    Ok(())
}

/// Manifests, driven like an agent: the CLI builds a manifest (`--client`), the live TUI picks each write up
/// (the store watcher) and shows it; the check flags an issue, the agent fixes it, plans, and the TUI shows the
/// planned stage with the exact commands. `apply` and `approve` are never bridge verbs; nothing calls Xero.
fn manifests(bins: &Bins, r: &mut Report) -> Result<(), String> {
    let mut w = World::new("manifests", bins.clone())?;
    w.start(120, 34, &[])?;
    let c = |w: &World, args: &[&str]| {
        let mut a = vec!["--json", "manifest"];
        a.extend_from_slice(args);
        a.extend_from_slice(&["--client", "tester"]);
        w.cli(&a)
    };
    let o = c(&w, &["new", "dec-live", "live acceptance"]);
    r.check(o.code == 0, "manifest new", || o.stderr.clone() + &o.stdout);
    step(&w, r, &["manifest", "dec-live", "tab", "7"], 0);
    let wait_rev = |w: &World, at_least: i64| {
        for _ in 0..100 {
            let st = w.state(&[]);
            if st["manifest"]["head_rev"].as_i64().unwrap_or(0) >= at_least {
                return st;
            }
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
        w.state(&[])
    };
    let o = c(
        &w,
        &[
            "journal",
            "--org",
            "design",
            "--date",
            "2026-01-31",
            "--narration",
            "Reclass management charges",
            "--cash-basis",
            "yes",
            "--line",
            "6420=263.94:charge",
            "--line",
            "2420=-263.94",
        ],
    );
    r.check(o.code == 0, "an agent's journal entry", || o.stdout.clone());
    let o = c(&w, &["recode", "roast:bt:bt-110#L1", "--account", "5029"]);
    r.check(o.code == 0, "an agent's recode entry", || o.stdout.clone());
    let recode_seq = o.json()["entry"].as_i64().unwrap_or(0);
    let st = wait_rev(&w, recode_seq);
    w.settle();
    r.check(
        st["manifest"]["entries"] == 2
            && st["manifest"]["issues"]["errors"].as_i64().unwrap_or(0) >= 1,
        "the TUI picked the entries up and the check flags the unknown account",
        || st["manifest"].to_string(),
    );
    let screen = w.screen().text();
    r.check(
        screen.contains("check ✗"),
        "the stage strip and chip show the failing check",
        || screen.clone(),
    );
    // the agent fixes it: undo the bad recode, recode to a real account (a reclass journal: bt-110 is reconciled)
    let o = c(&w, &["undo", &recode_seq.to_string()]);
    r.check(o.code == 0, "undo the bad recode", || o.stdout.clone());
    let o = c(&w, &["recode", "roast:bt:bt-110#L1", "--account", "5020"]);
    let fix = o.json()["entry"].as_i64().unwrap_or(0);
    let st = wait_rev(&w, fix);
    r.check(
        st["manifest"]["issues"]["errors"] == 0,
        "the check is clean after the fix",
        || st["manifest"].to_string(),
    );
    let o = c(&w, &["plan"]);
    r.check(o.code == 0, "manifest plan", || {
        o.stdout.clone() + &o.stderr
    });
    let plan = o.json();
    let hash = plan["hash"].as_str().unwrap_or("").to_string();
    let st = wait_rev(&w, fix + 1);
    w.settle();
    r.check(
        st["manifest"]["stage"] == "planned"
            && st["manifest"]["apply_command"].as_str() == plan["then_apply"].as_str()
            && st["manifest"]["approve_command"].as_str() == plan["approve"].as_str(),
        "the TUI shows the planned stage with the CLI's exact commands",
        || format!("{} vs {plan}", st["manifest"]),
    );
    let screen = w.screen().text();
    r.check(
        screen.contains(&format!(
            "{}…{}",
            &hash[..8.min(hash.len())],
            &hash[hash.len().saturating_sub(2)..]
        )),
        "the command panel shows the abbreviated hash",
        || screen.clone(),
    );
    for verb in ["apply", "approve"] {
        let o = w.send(&[verb]);
        r.check(
            o.code == 2 && o.json()["error"] == "forbidden",
            &format!("`{verb}` is never an agent verb"),
            || o.stdout.clone(),
        );
    }
    Ok(())
}

/// Walkthroughs (walkthroughs design §3): an agent places a hint on a ledger row (the reply says where it
/// landed), records a three-step walkthrough, then plays the file back and jumps to step 2: the frame
/// matches the one recorded.
fn walkthroughs(bins: &Bins, r: &mut Report) -> Result<(), String> {
    let mut w = World::new("walkthroughs", bins.clone())?;
    w.start(100, 30, &[])?;
    let send = |w: &World, args: &[&str]| {
        let mut a = vec!["--client", "tester"];
        a.extend_from_slice(args);
        w.send(&a)
    };
    // the content rows: no header (the agent mark) and no panel title (it names the agent, or the step)
    let body = |w: &World| -> Vec<String> {
        let t = w.screen().text();
        let lines: Vec<String> = t.lines().map(|l| l.trim_end().to_string()).collect();
        let end = lines
            .iter()
            .position(|l| l.starts_with("──") && l.contains(" · "))
            .unwrap_or(lines.len());
        lines[1..end].to_vec()
    };
    let o = send(&w, &["scope", "roast", "tab", "2"]);
    r.check(o.code == 0, "open the roastery's ledger", || {
        o.stdout.clone()
    });
    let o = send(
        &w,
        &[
            "hint",
            "row:code=6140",
            "The igniter is in this balance.",
            "title=Here",
        ],
    );
    let j = o.json();
    r.check(
        o.code == 0
            && j["layers"][0]["landed"] == "on_screen"
            && j["layers"][0]["anchor"] == "row:code=6140",
        "the hint landed on the row, and the reply says so",
        || o.stdout.clone(),
    );
    let screen = w.screen().text();
    r.check(
        screen.contains("◆ tester · Here") && screen.contains("The igniter is in this balance."),
        "the hint is drawn, attributed to the agent",
        || screen.clone(),
    );
    let o = send(&w, &["hint", "row:code=9999", "nowhere"]);
    r.check(
        o.json()["layers"][0]["landed"] == "not_found",
        "a hint on a row not on screen says not found, and why",
        || o.stdout.clone(),
    );
    let o = send(&w, &["clear-hints"]);
    r.check(o.code == 0, "clear-hints", || o.stdout.clone());
    // record three pages
    let o = send(
        &w,
        &["walkthrough", "record", "start", "live", "title=Live"],
    );
    r.check(o.code == 0, "record start", || o.stdout.clone());
    let o = send(&w, &["narrate", "One", "The roastery's [trial balance]."]);
    r.check(o.code == 0, "narrate page 1", || o.stdout.clone());
    let o = send(&w, &["highlight", "row:code=6140"]);
    r.check(o.code == 0, "highlight on page 1", || o.stdout.clone());
    let o = send(&w, &["walkthrough", "record", "step"]);
    r.check(o.code == 0, "record step 1", || o.stdout.clone());
    let o = send(&w, &["clear-hints"]);
    r.check(o.code == 0, "clear the highlight", || o.stdout.clone());
    let o = send(&w, &["tab", "3"]);
    r.check(o.code == 0, "to Intercompany", || o.stdout.clone());
    let o = send(&w, &["narrate", "Two", "Money between the companies."]);
    r.check(o.code == 0, "narrate page 2", || o.stdout.clone());
    let o = send(&w, &["walkthrough", "record", "step"]);
    r.check(o.code == 0, "record step 2", || o.stdout.clone());
    let page2 = body(&w);
    let o = send(&w, &["tab", "4"]);
    r.check(o.code == 0, "to Activity", || o.stdout.clone());
    let o = send(&w, &["narrate", "Three", "The calls this session made."]);
    r.check(o.code == 0, "page 3", || o.stdout.clone());
    let o = send(&w, &["walkthrough", "record", "step"]);
    r.check(o.code == 0, "record step 3", || o.stdout.clone());
    let o = send(&w, &["walkthrough", "record", "stop"]);
    let saved = o.json()["saved"].as_str().unwrap_or("").to_string();
    r.check(
        o.code == 0
            && saved.ends_with("walkthroughs/live.json")
            && std::path::Path::new(&saved).exists(),
        "record stop writes walkthroughs/live.json",
        || o.stdout.clone(),
    );
    let o = w.cli(&["--json", "walkthrough", "show", "live"]);
    r.check(
        o.code == 0 && o.json()["steps"].as_array().map_or(0, Vec::len) == 3,
        "the CLI reads the recorded file (3 steps)",
        || o.stdout.clone(),
    );
    // play it back from elsewhere, jump to step 2
    let o = send(&w, &["tab", "1"]);
    r.check(o.code == 0, "back to Overview", || o.stdout.clone());
    let o = send(&w, &["narrate", "clear"]);
    r.check(o.code == 0, "close the narration", || o.stdout.clone());
    let o = send(&w, &["walkthrough", "play", "live"]);
    r.check(o.code == 0, "play", || o.stdout.clone());
    let o = send(&w, &["walkthrough", "step", "2"]);
    r.check(o.code == 0, "jump to step 2", || o.stdout.clone());
    let st = w.state(&[]);
    r.check(
        st["walkthrough"]["step"] == 2 && st["walkthrough"]["of"] == 3,
        "the state says step 2 of 3",
        || st["walkthrough"].to_string(),
    );
    let played = body(&w);
    r.check(
        played == page2,
        "step 2 played back is the frame recorded",
        || first_diff(&played, &page2).unwrap_or_default(),
    );
    let o = send(&w, &["walkthrough", "stop"]);
    r.check(o.code == 0, "stop", || o.stdout.clone());
    for verb in ["apply", "approve"] {
        let o = w.send(&["walkthrough", verb]);
        r.check(
            o.code == 2,
            &format!("`walkthrough {verb}` is refused"),
            || o.stdout.clone(),
        );
    }
    Ok(())
}
