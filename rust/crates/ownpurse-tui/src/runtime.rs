//! The loop's core, generic over the backend so tests and benches drive it exactly as the terminal does:
//! `send(Msg)` runs `update` and its effects; `frame()` draws only when the app is dirty (V13).

use crate::app::{App, Effect, Msg, update};
use crate::hit::Hits;
use crate::model::Model;
use crate::ui;
use ratatui::Terminal;
use ratatui::backend::Backend;
use std::time::{Duration, Instant};

pub struct Runtime<B: Backend> {
    pub terminal: Terminal<B>,
    pub app: App,
    pub draws: u64,
    pub quit: bool,
    /// Network effects seen (V2: the shell produces none; the API crate will execute them).
    pub network_effects: u64,
    /// Effects the terminal loop runs outside the frame ($EDITOR, browser, clipboard), oldest first.
    pub external: Vec<Effect>,
    /// Starts a child (sync / login) whose events come back through the loop as `Msg::Run`. None: a
    /// start fails with a message (tests and benches that never run a child).
    pub spawner: Option<Spawner>,
    /// Children started and record reloads done (acceptance tests: one reload per run, after it ends).
    pub spawns: u64,
    pub reloads: u64,
    /// The installed binary to watch for a newer build (None: no update detection, e.g. state renders).
    pub update_probe: Option<std::path::PathBuf>,
    /// Its identity when last read (mtime, size, inode: a reinstall that keeps the mtime is still seen), and
    /// how many times `--version` ran (tests). Only a successful read is remembered: a build that didn't
    /// answer is asked again at the next interaction.
    pub update_mtime: Option<(std::time::SystemTime, u64, u64)>,
    pub update_reads: u64,
    /// A `--version` read running on its own thread (a fresh binary's first exec can stall for seconds
    /// while the OS checks it; the loop never waits for it): the file's identity, the answer, the start.
    pub update_pending: Option<UpdateRead>,
    /// An upgrade asked for while a child runs: applies when it ends (after its reload).
    pub upgrade_deferred: bool,
    /// A handover is due: the terminal loop performs it (`upgrade::handover`) after this batch of events.
    pub upgrade_due: bool,
}

/// A pending `--version` read of the installed build.
pub struct UpdateRead {
    pub id: (std::time::SystemTime, u64, u64),
    pub rx: std::sync::mpsc::Receiver<Option<String>>,
    pub started: std::time::Instant,
}

/// `(program, args) -> Ok(()) once running | Err(os error)`.
pub type Spawner = Box<dyn FnMut(&std::path::Path, &[String]) -> Result<(), String>>;

impl<B: Backend> Runtime<B> {
    pub fn new(mut terminal: Terminal<B>, mut app: App) -> Runtime<B> {
        let a = terminal.get_frame().area();
        app.ui.size = (a.width, a.height);
        app.repaint();
        app.dirty = true;
        Runtime {
            terminal,
            app,
            draws: 0,
            quit: false,
            network_effects: 0,
            external: Vec::new(),
            spawner: None,
            spawns: 0,
            reloads: 0,
            update_probe: None,
            update_mtime: None,
            update_reads: 0,
            update_pending: None,
            upgrade_deferred: false,
            upgrade_due: false,
        }
    }

    pub fn send(&mut self, msg: Msg) {
        // the update check rides on interaction and the existing minute tick (no timer of its own)
        let minute = self.app.now.div_euclid(60);
        let check = match &msg {
            Msg::Key(_) | Msg::Resize(..) | Msg::Mouse { .. } => true,
            Msg::Tick(t) => t.div_euclid(60) != minute,
            _ => false,
        };
        let new_minute = matches!(&msg, Msg::Tick(t) if t.div_euclid(60) != minute);
        for e in update(&mut self.app, msg) {
            self.effect(e);
        }
        if check {
            self.check_update();
        } else if self.update_pending.is_some() {
            self.poll_update(); // a read's answer is picked up with the next event (no timer of its own)
        }
        // a dropped file event is caught on the minute tick (no timer of its own)
        if new_minute {
            self.external_check();
        }
        // a deferred upgrade applies once the child has ended (its reload ran in the effects above)
        if self.upgrade_deferred && self.app.ui.run.is_none() {
            self.upgrade_deferred = false;
            self.check_update();
            if self.upgradable().is_some() {
                self.upgrade_due = true;
            }
        }
    }

    /// The version a handover would go to: a newer installed build that can take over in place.
    pub fn upgradable(&self) -> Option<String> {
        self.app
            .update_available
            .as_ref()
            .filter(|u| u.action == crate::upgrade::ACTION)
            .map(|u| u.version.clone())
    }

    /// An upgrade request (palette or agent): up to date, deferred while a child runs, or due now.
    pub fn upgrade_ask(&mut self) -> crate::upgrade::Ask {
        use crate::upgrade::Ask;
        // under the bridge's 10 s answer: a build still being checked by the OS is "up to date" for now
        self.settle_update(std::time::Duration::from_secs(8));
        let Some(to) = self.upgradable() else {
            return Ask::UpToDate;
        };
        // never a handover to the build that is already running (same version and build id)
        if to == crate::app::full_version(&self.app.version_label, self.app.build_label.as_deref())
        {
            self.app.update_available = None;
            self.app.dirty = true;
            return Ask::UpToDate;
        }
        if let Some(r) = &self.app.ui.run {
            self.upgrade_deferred = true;
            self.app.ui.note = Some(crate::upgrade::deferred_note(&to, &r.label()));
            self.app.dirty = true;
            return Ask::Deferred(to);
        }
        self.upgrade_due = true;
        Ask::Go(to)
    }

    /// Another process wrote the record or the classification store (a file event, or the minute tick):
    /// refresh only when a revision on disk moved (an event for this TUI's own write draws nothing).
    pub fn external_check(&mut self) {
        let store = self
            .app
            .source
            .classify
            .as_ref()
            .filter(|_| self.app.classify.is_some())
            .map(|c| c.store.clone());
        let manifests = self.app.source.manifests.as_ref().map(|p| p.store.clone());
        let disk = crate::external::read_all(
            &self.app.source.record,
            store.as_deref(),
            manifests.as_deref(),
        );
        if disk != self.app.shown_revs() {
            self.send(Msg::External(disk));
        }
    }

    /// `ownpurse.json` was saved (the watcher; also before a live upgrade): re-read it and apply a valid
    /// change with the view kept, or show why it can't be used (the last valid config stays live).
    pub fn config_check(&mut self) {
        match crate::project_config::load(&self.app.source) {
            Ok(None) => {
                // valid again (or unchanged): a band about it goes
                if self
                    .app
                    .ui
                    .result
                    .as_ref()
                    .is_some_and(|r| r.copy.starts_with(crate::project_config::BAND))
                {
                    self.app.ui.result = None;
                    self.app.dirty = true;
                    self.app.rev += 1;
                }
            }
            Ok(Some(src)) => match crate::project_config::apply(&self.app, src) {
                Ok(app) => self.app = app,
                Err(why) => {
                    self.app.ui.result = Some(crate::project_config::invalid_band(&why));
                    self.app.dirty = true;
                    self.app.rev += 1;
                }
            },
            Err(why) => {
                let band = crate::project_config::invalid_band(&why);
                if self.app.ui.result.as_ref().map(|r| &r.copy) != Some(&band.copy) {
                    self.app.ui.result = Some(band);
                    self.app.dirty = true;
                    self.app.rev += 1;
                }
            }
        }
    }

    fn effect(&mut self, e: Effect) {
        match e {
            Effect::Quit => self.quit = true,
            Effect::Reload => {
                self.reloads += 1;
                match Model::open(&self.app.source, self.app.now) {
                    Ok(m) => {
                        let books = crate::books::Books::open(&self.app.source).ok();
                        self.send(Msg::Reloaded(Box::new((m, books))))
                    }
                    Err(err) => {
                        self.app.ui.note = Some(format!("reload failed: {err}"));
                        self.app.dirty = true;
                    }
                }
            }
            Effect::Network(_) => self.network_effects += 1,
            e @ (Effect::Edit(_) | Effect::Open(_) | Effect::Copy(_)) => self.external.push(e),
            Effect::Spawn { program, args } => {
                self.spawns += 1;
                let r = match self.spawner.as_mut() {
                    Some(f) => f(&program, &args),
                    None => Err("no process runner in this context".into()),
                };
                if let Err(e) = r {
                    self.send(Msg::Run(crate::net::RunEvent::SpawnFailed(e)));
                }
            }
            Effect::Upgrade => {
                if self.upgrade_ask() == crate::upgrade::Ask::UpToDate {
                    self.app.ui.note = Some(format!("v{} is up to date", self.app.version_label));
                    self.app.dirty = true;
                }
            }
            Effect::Classify(w) => {
                // the classification store only (V14); the rows reload from it on `Classified`
                let m = crate::classify::perform(&mut self.app, w);
                self.send(m);
            }
            Effect::SaveWalkthrough(w) => {
                let r = match self.app.source.walkthroughs.clone() {
                    Some(dir) => ownpurse_core::walkthrough::write(&dir, &w),
                    None => Err("no walkthroughs folder (it lives beside ownpurse.json)".into()),
                };
                self.app.ui.note = Some(match &r {
                    Ok(p) => format!("saved {}", self.app.paths.show(p)),
                    Err(e) => format!("walkthrough not saved: {e}"),
                });
                self.app.walk_saved = Some(r);
                self.app.dirty = true;
            }
            Effect::CliVersion => {
                self.app.cli_version = Some(
                    self.app
                        .cli
                        .as_deref()
                        .and_then(crate::net::cli_version)
                        .unwrap_or_else(|| "version unknown".into()),
                );
                self.app.dirty = true;
            }
        }
    }

    /// The update check: stat the installed binary; on a new mtime, read its `--version` once; a newer
    /// version than the running one shows in the footer (`upgrade_in_place`: `⬆ vX ready · ⌃P Upgrade`).
    /// Writes nothing unless that changes.
    pub fn check_update(&mut self) {
        if self.update_pending.is_some() {
            self.poll_update();
            return;
        }
        let Some(p) = self.update_probe.clone() else {
            return;
        };
        use std::os::unix::fs::MetadataExt;
        let id = std::fs::metadata(&p)
            .ok()
            .and_then(|m| m.modified().ok().map(|t| (t, m.len(), m.ino())));
        let Some(id) = id else { return };
        if Some(id) == self.update_mtime {
            return;
        }
        self.update_reads += 1;
        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let _ = tx.send(crate::net::tui_version_within(&p, VERSION_READ_TIMEOUT));
        });
        self.update_pending = Some(UpdateRead {
            id,
            rx,
            started: std::time::Instant::now(),
        });
        self.poll_update();
    }

    /// Take a finished `--version` read: a newer build shows in the footer; only a good answer is
    /// remembered (a build that didn't answer is asked again at the next interaction).
    pub fn poll_update(&mut self) {
        let Some(r) = &self.update_pending else {
            return;
        };
        let found = match r.rx.try_recv() {
            Ok(v) => v,
            Err(std::sync::mpsc::TryRecvError::Empty)
                if r.started.elapsed()
                    < VERSION_READ_TIMEOUT + std::time::Duration::from_secs(5) =>
            {
                return;
            }
            Err(_) => None,
        };
        let id = r.id;
        self.update_pending = None;
        let Some(found) = found else { return };
        self.update_mtime = Some(id);
        let next = Some(found)
            .filter(|v| differs_newer(v, &crate::app::build_version()))
            .map(|version| crate::canonical::Update {
                version,
                action: crate::upgrade::ACTION.into(),
            });
        if next != self.app.update_available {
            self.app.update_available = next;
            self.app.dirty = true;
        }
    }

    /// Wait (at most `max`) for a running `--version` read: an upgrade request, and tests, need the answer.
    pub fn settle_update(&mut self, max: std::time::Duration) {
        let end = std::time::Instant::now() + max;
        self.check_update();
        while self.update_pending.is_some() && std::time::Instant::now() < end {
            std::thread::sleep(std::time::Duration::from_millis(10));
            self.poll_update();
        }
    }

    /// Draw if dirty. Returns whether a frame was drawn.
    pub fn frame(&mut self) -> Result<bool, B::Error> {
        if !self.app.dirty {
            return Ok(false);
        }
        let app = &self.app;
        let mut hits = Hits::default();
        self.terminal.draw(|f| hits = ui::draw(f, app))?;
        self.app.hits = hits;
        self.app.dirty = false;
        self.draws += 1;
        Ok(true)
    }
}

/// Counts presses for double-clicks (same cell within 400 ms).
#[derive(Default)]
pub struct Clicks {
    last: Option<(Instant, u16, u16)>,
    count: u8,
}

impl Clicks {
    pub fn press(&mut self, x: u16, y: u16) -> u8 {
        let now = Instant::now();
        self.count = match self.last {
            Some((t, lx, ly)) if lx == x && ly == y && now - t < Duration::from_millis(400) => {
                self.count.saturating_add(1)
            }
            _ => 1,
        };
        self.last = Some((now, x, y));
        self.count
    }
}

/// The installed build `found` should replace the running `mine` (both `version[+build]`): a newer semver;
/// or, at the same semver, a different build id (a dev build of the same version: another commit, or a
/// release replaced by a dev build). Two releases (no build ids) compare by semver alone.
/// How long the update check waits for the installed build's `--version`.
pub const VERSION_READ_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(20);

pub fn differs_newer(found: &str, mine: &str) -> bool {
    let split = |v: &str| {
        let v = v.trim().trim_start_matches('v');
        match v.split_once('+') {
            Some((s, b)) => (s.to_string(), Some(b.to_string())),
            None => (v.to_string(), None),
        }
    };
    let ((fv, fb), (mv, mb)) = (split(found), split(mine));
    if newer(&fv, &mv) {
        return true;
    }
    if newer(&mv, &fv) {
        return false;
    }
    (fb.is_some() || mb.is_some()) && fb != mb
}

/// `a` is a newer version than `b` (numeric dotted parts; a pre-release suffix sorts before its release;
/// `+build` metadata is ignored).
pub fn newer(a: &str, b: &str) -> bool {
    fn parts(v: &str) -> (Vec<u64>, bool) {
        let v = v.trim().trim_start_matches('v');
        let v = v.split_once('+').map_or(v, |(s, _)| s);
        let (core, pre) = v.split_once('-').map_or((v, false), |(c, _)| (c, true));
        (
            core.split('.').map(|p| p.parse().unwrap_or(0)).collect(),
            pre,
        )
    }
    let ((pa, prea), (pb, preb)) = (parts(a), parts(b));
    match pa.cmp(&pb) {
        std::cmp::Ordering::Greater => true,
        std::cmp::Ordering::Less => false,
        std::cmp::Ordering::Equal => preb && !prea,
    }
}

#[cfg(test)]
mod tests {
    use super::newer;

    #[test]
    fn versions_compare_numerically() {
        assert!(newer("0.10.3", "0.10.2"));
        assert!(newer("0.10.10", "0.10.9"));
        assert!(!newer("0.10.2", "0.10.2"));
        assert!(!newer("0.9.9", "0.10.0"));
        assert!(newer("0.10.2", "0.10.2-rc.1"));
        assert!(!newer("0.10.2-rc.1", "0.10.2"));
        assert!(!newer("0.12.1+g6dc723f", "0.12.1"));
    }

    #[test]
    fn a_different_build_of_the_same_version_is_an_upgrade() {
        use super::differs_newer;
        assert!(differs_newer("0.12.1+g6dc723f", "0.12.1+g5aa1401"));
        assert!(differs_newer("0.12.1+g6dc723f", "0.12.1"));
        assert!(differs_newer("0.12.1", "0.12.1+g6dc723f"));
        assert!(!differs_newer("0.12.1+g6dc723f", "0.12.1+g6dc723f"));
        assert!(!differs_newer("0.12.1", "0.12.1"), "releases: semver only");
        assert!(differs_newer("0.12.2", "0.12.1+g6dc723f"));
        assert!(
            !differs_newer("0.12.0+g1111111", "0.12.1+g2222222"),
            "an older version never"
        );
    }
}
