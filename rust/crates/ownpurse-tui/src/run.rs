//! Sync and login runs in the app (spec P6 network §3–§5): the confirm copy (core's estimate, the per-org
//! budget rule), the running state fed by the child's progress events, and the result band. Pure state:
//! the runtime starts the child (`Effect::Spawn`) and feeds its events back (`Msg::Run`).

use crate::app::{App, Band, BandKind, Effect, NetAction, Scope};
use crate::clock::{int, plural};
use crate::net::RunEvent;
use ownpurse_core::estimate::Estimate;
use serde_json::Value;

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum RunKind {
    /// `target` is the CLI argument (`design`, `all`, `group:all-co`); `who` the copy (`design`, `4 orgs`).
    Sync {
        target: String,
        who: String,
        multi: bool,
    },
    Login,
}

/// A child running now. Lives in `Ui`, so each event that changes it redraws once (and nothing else does).
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct RunState {
    pub kind: RunKind,
    /// The one-row band is shown (Esc hides it; progress then moves to the status strip).
    pub band_visible: bool,
    pub org: String,
    pub estimate: Option<i64>,
    pub entity: Option<String>,
    pub page: Option<i64>,
    pub calls: i64,
    pub day_remaining: Option<i64>,
    pub wait: Option<(String, i64)>,
    pub browser_timeout: Option<i64>,
    pub done_orgs: i64,
    pub done_calls: i64,
    pub records: i64,
    pub new_versions: i64,
    pub login_orgs: Option<i64>,
    /// Non-JSON stderr, kept for the failure band.
    pub raw: Vec<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ResultLevel {
    Ok,
    Warn,
    Error,
}

/// The result band: non-modal; the next key dismisses it and does its normal action, Esc only dismisses,
/// `y` copies `copy`.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ResultBand {
    pub level: ResultLevel,
    pub lines: Vec<String>,
    pub copy: String,
}

impl RunState {
    pub fn new(kind: RunKind) -> RunState {
        RunState {
            kind,
            band_visible: true,
            org: String::new(),
            estimate: None,
            entity: None,
            page: None,
            calls: 0,
            day_remaining: None,
            wait: None,
            browser_timeout: None,
            done_orgs: 0,
            done_calls: 0,
            records: 0,
            new_versions: 0,
            login_orgs: None,
            raw: Vec::new(),
        }
    }

    /// `sync design`, `sync all`, `sync group:all-co`, `login` (palette suffix, status strip).
    pub fn label(&self) -> String {
        match &self.kind {
            RunKind::Sync { target, .. } => format!("sync {target}"),
            RunKind::Login => "login".into(),
        }
    }

    fn who(&self) -> String {
        match &self.kind {
            RunKind::Sync { .. } if !self.org.is_empty() => self.org.clone(),
            RunKind::Sync { who, .. } => who.clone(),
            RunKind::Login => String::new(),
        }
    }

    fn where_(&self) -> String {
        match (&self.entity, self.page) {
            (Some(e), Some(p)) => format!("{e} p{p}"),
            (Some(e), None) => e.clone(),
            _ => "starting".into(),
        }
    }

    /// The band (spec §4): `⇅ Syncing design · BankTransactions p2 · 9 of ≈ 15 calls · 929 left`.
    pub fn band_text(&self) -> String {
        match &self.kind {
            RunKind::Login => match self.browser_timeout {
                Some(t) => format!(
                    "⇅ Waiting for you to approve in the browser · times out after {}",
                    minutes(t)
                ),
                None => "⇅ Logging in · opening your browser".into(),
            },
            RunKind::Sync { .. } => {
                let who = self.who();
                if let Some((reason, secs)) = &self.wait {
                    return match reason.as_str() {
                        "minute_limit" => {
                            format!(
                                "⇅ Syncing {who} · pausing {secs} s for Xero's per-minute limit"
                            )
                        }
                        "throttle" => format!("⇅ Syncing {who} · pacing calls, {secs} s"),
                        _ => format!("⇅ Syncing {who} · Xero didn't answer, retrying in {secs} s"),
                    };
                }
                let calls = match self.estimate {
                    Some(e) => format!("{} of ≈ {}", self.calls, plural(e, "call", "calls")),
                    None => plural(self.calls, "call", "calls"),
                };
                let left = self
                    .day_remaining
                    .map_or(String::new(), |d| format!(" · {} left", int(d)));
                format!("⇅ Syncing {who} · {} · {calls}{left}", self.where_())
            }
        }
    }

    /// The status strip's left segment while the band is hidden: `⇅ sync design · BankTransactions p2 · 9 calls`.
    pub fn strip_text(&self) -> String {
        match &self.kind {
            RunKind::Login => match self.browser_timeout {
                Some(_) => "⇅ login · waiting for the browser".into(),
                None => "⇅ login".into(),
            },
            RunKind::Sync { target, multi, .. } => {
                let who = if *multi && !self.org.is_empty() {
                    format!("{target} ({})", self.org)
                } else {
                    target.clone()
                };
                if let Some((_, secs)) = &self.wait {
                    return format!("⇅ sync {who} · waiting {secs} s");
                }
                format!(
                    "⇅ sync {who} · {} · {}",
                    self.where_(),
                    plural(self.calls, "call", "calls")
                )
            }
        }
    }

    /// One progress event (unknown `ev` values are no-ops). Non-JSON lines are kept as raw text.
    pub fn on_line(&mut self, line: &str) {
        let Ok(v) = serde_json::from_str::<Value>(line) else {
            if !line.trim().is_empty() {
                self.raw.push(line.to_string());
            }
            return;
        };
        let s = |k: &str| v[k].as_str().map(str::to_string);
        let n = |k: &str| v[k].as_i64();
        match v["ev"].as_str().unwrap_or("") {
            "start" => {
                self.org = s("org").unwrap_or_default();
                self.estimate = n("estimate");
                // another org's budget: its figure comes with its first call
                self.day_remaining = None;
                self.entity = None;
                self.page = None;
                self.calls = 0;
                self.wait = None;
            }
            "entity" => {
                self.entity = s("entity");
                self.page = n("page");
                self.calls = n("calls").unwrap_or(self.calls);
                if let Some(d) = n("day_remaining") {
                    self.day_remaining = Some(d);
                }
                self.wait = None;
            }
            "wait" => {
                self.wait = Some((s("reason").unwrap_or_default(), n("seconds").unwrap_or(0)));
            }
            "browser" => self.browser_timeout = Some(n("timeout").unwrap_or(900)),
            "done" => {
                if let Some(o) = n("orgs").filter(|_| v.get("org").is_none()) {
                    self.login_orgs = Some(o);
                } else {
                    self.done_orgs += 1;
                    self.done_calls += n("calls").unwrap_or(self.calls);
                    self.records += n("records").unwrap_or(0);
                    self.new_versions += n("new_versions").unwrap_or(0);
                    self.wait = None;
                }
            }
            _ => {}
        }
    }

    /// The result band for the child's exit (spec §5).
    pub fn result(&self, code: Option<i32>, stdout: &str, elapsed_secs: u64) -> ResultBand {
        let doc: Option<Value> = stdout
            .lines()
            .find(|l| l.trim_start().starts_with('{'))
            .and_then(|_| serde_json::from_str(stdout.trim()).ok())
            .or_else(|| {
                // the first JSON document on stdout
                serde_json::Deserializer::from_str(stdout)
                    .into_iter::<Value>()
                    .next()
                    .and_then(Result::ok)
            });
        let is_sync = matches!(self.kind, RunKind::Sync { .. });
        if code == Some(0) {
            let line = match &self.kind {
                RunKind::Login => format!(
                    "✓ Logged in · {} connected",
                    plural(self.login_orgs.unwrap_or(0), "org", "orgs")
                ),
                RunKind::Sync { who, multi, .. } => {
                    let calls = if self.done_orgs > 0 {
                        self.done_calls
                    } else {
                        doc.as_ref()
                            .and_then(|d| d["calls_used"].as_i64())
                            .unwrap_or(0)
                    };
                    let who = if *multi {
                        plural(self.done_orgs.max(1), "org", "orgs")
                    } else {
                        who.clone()
                    };
                    format!(
                        "✓ Synced {who} · {} · {}, {} · {} · record reloaded",
                        plural(calls, "call", "calls"),
                        plural(self.records, "record", "records"),
                        plural(self.new_versions, "new version", "new versions"),
                        elapsed(elapsed_secs)
                    )
                }
            };
            return ResultBand {
                level: ResultLevel::Ok,
                copy: line.clone(),
                lines: vec![line],
            };
        }
        // a failure: the CLI's error verbatim (`error:` code, `message:`, then `help[]`)
        let (mut lines, level) = match doc.as_ref().filter(|d| d.get("error").is_some()) {
            Some(d) => {
                let code = d["error"].as_str().unwrap_or("error");
                let msg = d["message"].as_str().unwrap_or("");
                let limit = code == "daily_limit";
                let mut first = format!("✗ error: {code} · {msg}");
                if limit && is_sync {
                    first.push_str(" · record reloaded");
                }
                let mut v = vec![first];
                v.extend(
                    d["help"]
                        .as_array()
                        .into_iter()
                        .flatten()
                        .filter_map(Value::as_str)
                        .map(str::to_string),
                );
                (
                    v,
                    if limit {
                        ResultLevel::Warn
                    } else {
                        ResultLevel::Error
                    },
                )
            }
            None => {
                let mut v = vec![format!(
                    "✗ ownpurse exited with {}",
                    code.map_or("a signal".to_string(), |c| format!("status {c}"))
                )];
                v.extend(self.raw.iter().cloned());
                (v, ResultLevel::Error)
            }
        };
        let copy = lines.join("\n");
        lines.truncate(4); // the first line plus at most 3 help rows; `y` copies everything
        ResultBand { level, lines, copy }
    }
}

/// Xero's documented reset rule (a fixed window that resets at a different time for each tenant).
pub const RESETS: &str = "Xero resets each org's daily limit at its own time.";

fn minutes(secs: i64) -> String {
    if secs % 60 == 0 && secs >= 60 {
        format!("{} min", secs / 60)
    } else {
        format!("{secs} s")
    }
}

/// `0:21`, `1:04`, `12:30`.
pub fn elapsed(secs: u64) -> String {
    format!("{}:{:02}", secs / 60, secs % 60)
}

impl App {
    /// The orgs a sync action covers.
    fn sync_members(&self, a: NetAction) -> Vec<usize> {
        match a {
            NetAction::SyncOrg(i) => vec![i],
            NetAction::SyncAll => (0..self.model.orgs.len()).collect(),
            NetAction::SyncGroup(g) => self.members(&Scope::Group(g)),
            _ => vec![],
        }
    }

    /// The confirm (or refusal) for a sync or login (spec §3). Nothing runs until Enter on the action.
    pub(crate) fn confirm_run(&mut self, a: NetAction) {
        self.ui.overlay = None;
        if matches!(a, NetAction::Login) {
            self.ui.band = Some(Band {
                kind: BandKind::Ask {
                    action: a,
                    label: "Log in".into(),
                },
                text: "Log in to Xero? Opens your browser. Uses no daily budget.".into(),
                focus: 0,
            });
            return;
        }
        let members = self.sync_members(a);
        // the budget rule, per org: refused if any member's estimate exceeds its calls left (the estimated
        // figure counts too; the copy says when it is one). Xero's window resets per tenant, at its own time.
        for &i in &members {
            let o = &self.model.orgs[i];
            if let Some(e) = o.sync_estimate
                && e.exceeds(o.left)
            {
                let left = if o.left_known {
                    format!("{} left", int(o.left))
                } else {
                    format!("{} left, estimated", o.left_fig())
                };
                let t = format!(
                    "Not enough calls left for {} (needs ≈ {}, {left}). {RESETS}",
                    o.alias,
                    e.calls(),
                );
                self.refuse(t);
                return;
            }
        }
        let (text, label) = if let NetAction::SyncOrg(i) = a {
            let o = &self.model.orgs[i];
            let est = if o.left_known { "" } else { " (estimated)" };
            let text = match o.sync_estimate {
                Some(e @ Estimate::Calls(_)) => format!(
                    "Sync {a} from Xero? {} · {} left for {a}{est}.",
                    e.label(),
                    o.left_fig(),
                    a = o.alias
                ),
                Some(e) => format!(
                    "Sync {} from Xero? {} · {} left{est}.",
                    o.alias,
                    e.label(),
                    o.left_fig()
                ),
                None => format!(
                    "Sync {} from Xero? calls: unknown · {} left{est}.",
                    o.alias,
                    o.left_fig()
                ),
            };
            (text, "Sync".to_string())
        } else {
            let n = members.len() as i64;
            let ests: Vec<Option<Estimate>> = members
                .iter()
                .map(|&i| self.model.orgs[i].sync_estimate)
                .collect();
            let sum: i64 = ests.iter().flatten().map(|e| e.calls()).sum();
            let est = if ests.iter().any(Option::is_none) {
                "calls: unknown".to_string()
            } else if ests
                .iter()
                .flatten()
                .any(|e| matches!(e, Estimate::FirstSyncFloor { .. }))
            {
                format!("≈ {sum}+ calls")
            } else {
                format!("≈ {}", plural(sum, "call", "calls"))
            };
            let lowest = members
                .iter()
                .min_by_key(|&&i| self.model.orgs[i].left)
                .copied();
            let low = lowest.map_or(String::new(), |i| {
                let o = &self.model.orgs[i];
                format!(
                    " · lowest left: {} {}{}",
                    o.alias,
                    o.left_fig(),
                    if o.left_known { "" } else { " (estimated)" }
                )
            });
            (
                format!("Sync {} from Xero? {est}{low}.", plural(n, "org", "orgs")),
                format!("Sync {}", plural(n, "org", "orgs")),
            )
        };
        self.ui.band = Some(Band {
            kind: BandKind::Ask { action: a, label },
            text,
            focus: 0,
        });
    }

    /// Enter on the confirm's action: start the child (spec §2 argv).
    pub(crate) fn start_run(&mut self, a: NetAction, fx: &mut Vec<Effect>) {
        let Some(program) = self.cli.clone() else {
            self.refuse(crate::net::NO_CLI.into());
            return;
        };
        let (cmd, target, kind) = match a {
            NetAction::SyncOrg(i) => {
                let alias = self.model.orgs[i].alias.clone();
                (
                    "sync",
                    Some(alias.clone()),
                    RunKind::Sync {
                        target: alias.clone(),
                        who: alias,
                        multi: false,
                    },
                )
            }
            NetAction::SyncAll => (
                "sync",
                Some("all".to_string()),
                RunKind::Sync {
                    target: "all".into(),
                    who: plural(self.model.orgs.len() as i64, "org", "orgs"),
                    multi: true,
                },
            ),
            NetAction::SyncGroup(g) => {
                let t = format!("group:{}", self.model.groups[g].name);
                (
                    "sync",
                    Some(t.clone()),
                    RunKind::Sync {
                        target: t,
                        who: plural(self.members(&Scope::Group(g)).len() as i64, "org", "orgs"),
                        multi: true,
                    },
                )
            }
            NetAction::Login => ("login", None, RunKind::Login),
            _ => return,
        };
        let args = crate::net::argv(
            cmd,
            target.as_deref(),
            &self.source.profile,
            self.source.config_arg.as_deref(),
        );
        self.ui.run = Some(RunState::new(kind));
        self.ui.result = None;
        fx.push(Effect::Spawn { program, args });
    }

    /// A child's event (spec §4, §5). The record reloads exactly once, after a sync ends (success or failure).
    pub(crate) fn on_run(&mut self, ev: RunEvent, fx: &mut Vec<Effect>) {
        match ev {
            RunEvent::Line(l) => {
                if let Some(r) = self.ui.run.as_mut() {
                    r.on_line(&l);
                }
            }
            RunEvent::Exit {
                code,
                stdout,
                elapsed_secs,
            } => {
                if let Some(r) = self.ui.run.take() {
                    self.ui.result = Some(r.result(code, &stdout, elapsed_secs));
                    if matches!(r.kind, RunKind::Sync { .. }) {
                        fx.push(Effect::Reload);
                    }
                }
            }
            RunEvent::SpawnFailed(e) => {
                self.ui.run = None;
                let line = format!("✗ Couldn't start ownpurse: {e}");
                self.ui.result = Some(ResultBand {
                    level: ResultLevel::Error,
                    copy: line.clone(),
                    lines: vec![line],
                });
            }
        }
    }

    /// The quit confirm while a child runs (spec §4); the child keeps running in its own session.
    pub(crate) fn confirm_quit_running(&mut self) {
        let what = match self.ui.run.as_ref().map(|r| r.kind.clone()) {
            Some(RunKind::Sync { target, .. }) if target == "all" => "Sync all orgs".to_string(),
            Some(RunKind::Sync { target, .. }) => format!("Sync {target}"),
            Some(RunKind::Login) => "Log in".into(),
            None => return,
        };
        self.ui.band = Some(Band {
            kind: BandKind::Ask {
                action: NetAction::QuitWhileRunning,
                label: "Quit, let it finish".into(),
            },
            text: format!(
                "{what} is still running. Quit the app and let it finish in the background?"
            ),
            focus: 0,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run() -> RunState {
        RunState::new(RunKind::Sync {
            target: "design".into(),
            who: "design".into(),
            multi: false,
        })
    }

    #[test]
    fn band_copy_follows_events() {
        let mut r = run();
        r.on_line(r#"{"ev":"start","org":"design","entities":14,"estimate":15}"#);
        r.on_line(r#"{"ev":"entity","org":"design","entity":"BankTransactions","page":2,"calls":9,"day_remaining":929}"#);
        assert_eq!(
            r.band_text(),
            "⇅ Syncing design · BankTransactions p2 · 9 of ≈ 15 calls · 929 left"
        );
        assert_eq!(
            r.strip_text(),
            "⇅ sync design · BankTransactions p2 · 9 calls"
        );
        r.on_line(r#"{"ev":"wait","org":"design","reason":"minute_limit","seconds":4}"#);
        assert_eq!(
            r.band_text(),
            "⇅ Syncing design · pausing 4 s for Xero's per-minute limit"
        );
        r.on_line(r#"{"ev":"wait","org":"design","reason":"throttle","seconds":4}"#);
        assert_eq!(r.band_text(), "⇅ Syncing design · pacing calls, 4 s");
        r.on_line(r#"{"ev":"wait","org":"design","reason":"retry","seconds":4}"#);
        assert_eq!(
            r.band_text(),
            "⇅ Syncing design · Xero didn't answer, retrying in 4 s"
        );
        r.on_line(r#"{"ev":"future_event","x":1}"#);
        r.on_line("not json at all");
        assert_eq!(r.raw, vec!["not json at all"]);
        let mut l = RunState::new(RunKind::Login);
        l.on_line(r#"{"ev":"browser","url_shown":false,"timeout":900}"#);
        assert_eq!(
            l.band_text(),
            "⇅ Waiting for you to approve in the browser · times out after 15 min"
        );
    }

    #[test]
    fn result_copy() {
        let mut r = run();
        r.on_line(r#"{"ev":"done","org":"design","calls":15,"records":25,"new_versions":3}"#);
        let b = r.result(Some(0), r#"{"calls_used": 15}"#, 21);
        assert_eq!(
            b.lines,
            ["✓ Synced design · 15 calls · 25 records, 3 new versions · 0:21 · record reloaded"]
        );
        let mut all = RunState::new(RunKind::Sync {
            target: "all".into(),
            who: "4 orgs".into(),
            multi: true,
        });
        for _ in 0..4 {
            all.on_line(r#"{"ev":"done","org":"x","calls":15,"records":18,"new_versions":2}"#);
        }
        let b = all.result(Some(0), "{}", 64);
        assert_eq!(
            b.lines,
            ["✓ Synced 4 orgs · 60 calls · 72 records, 8 new versions · 1:04 · record reloaded"]
        );
        let mut l = RunState::new(RunKind::Login);
        l.on_line(r#"{"ev":"done","orgs":4}"#);
        assert_eq!(
            l.result(Some(0), "{}", 3).lines,
            ["✓ Logged in · 4 orgs connected"]
        );
        let b = run().result(
            Some(1),
            r#"{"error": "daily_limit", "message": "Xero daily call limit reached for this organisation.", "help": ["a", "b"]}"#,
            5,
        );
        assert_eq!(b.level, ResultLevel::Warn);
        assert_eq!(
            b.lines[0],
            "✗ error: daily_limit · Xero daily call limit reached for this organisation. · record reloaded"
        );
        let b = run().result(
            Some(2),
            r#"{"error": "needs_login", "message": "m", "help": ["1", "2", "3", "4"]}"#,
            5,
        );
        assert_eq!(b.level, ResultLevel::Error);
        assert_eq!(b.lines.len(), 4, "first line + 3 help rows");
        assert!(b.copy.ends_with("\n4"), "y copies everything");
        assert_eq!(elapsed(754), "12:34");
    }
}
