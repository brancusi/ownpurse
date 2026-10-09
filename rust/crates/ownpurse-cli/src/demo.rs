//! One-command, offline demonstration in the real TUI. The driver uses the same guarded
//! view-only bridge as an agent; Caretline still owns the walkthrough and overlay primitives.
use crate::app::{CliError, Res};
use ownpurse_core::bridge::{self as wire, Dirs};
use ownpurse_core::manifest::{Refs, Store, compile, ops};
use ownpurse_core::record::Record;
use serde_json::{Value, json};
use std::io::IsTerminal;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::{Duration, Instant};

const RECORD: &[u8] = include_bytes!("../../../tests/fixtures/demo/record.sqlite");
const CONFIG: &str = include_str!("../../../tests/fixtures/demo/ownpurse.json");
const TOUR: &str = include_str!("../../../tests/fixtures/walkthroughs/correction.json");
const STEPS: u64 = 10;

fn err(e: impl std::fmt::Display) -> CliError {
    CliError::runtime("demo", e.to_string())
}

/// Only a directory exclusively created by this process is ever removed.
struct World(PathBuf);
impl Drop for World {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
impl World {
    fn new() -> Res<Self> {
        // A short private path also keeps the Unix socket out of the shared long-path fallback.
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(err)?
            .as_nanos();
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let n = NEXT.fetch_add(1, Ordering::Relaxed);
        let root =
            PathBuf::from("/tmp").join(format!("xa-demo-{}-{stamp}-{n}", std::process::id()));
        let mut builder = std::fs::DirBuilder::new();
        #[cfg(unix)]
        {
            use std::os::unix::fs::DirBuilderExt;
            builder.mode(0o700);
        }
        builder.create(&root).map_err(err)?;
        let world = Self(root);
        world.prepare()?;
        Ok(world)
    }

    fn dirs(&self) -> Dirs {
        Dirs {
            state: self.0.join("state/ownpurse"),
            sockets: self.0.join("run/ownpurse"),
        }
    }

    fn prepare(&self) -> Res<()> {
        for dir in [
            "home",
            "config",
            "state",
            "run",
            "db/packages",
            "walkthroughs",
        ] {
            std::fs::create_dir_all(self.0.join(dir)).map_err(err)?;
        }
        std::fs::write(self.0.join("record.sqlite"), RECORD).map_err(err)?;
        let mut config: Value = serde_json::from_str(CONFIG).map_err(err)?;
        config["tui"]["agent"] = json!(true);
        config["tui"]["paint_bg"] = json!(true);
        std::fs::write(
            self.0.join("ownpurse.json"),
            serde_json::to_vec_pretty(&config).map_err(err)?,
        )
        .map_err(err)?;
        let mut tour: Value = serde_json::from_str(TOUR).map_err(err)?;
        for step in tour["step"]
            .as_array_mut()
            .ok_or_else(|| err("demo steps"))?
        {
            if step["host"].get("view").is_none() {
                step["host"]["view"] = json!({"verbs": []});
            }
            step["host"]["view"]["verbs"]
                .as_array_mut()
                .ok_or_else(|| err("demo view verbs"))?
                .insert(0, json!(["asof", "2025-12-31"]));
        }
        tour["step"][0]["place"]["spotlight"] = json!(true);
        tour["step"][0]["place"]["head"] = json!("on_anchor_rows");
        tour["step"][8]["place"]["head"] = json!("on_anchor_rows");
        // Dense diffs and protected stage/command areas: narrate beside a ring, not over them.
        for n in [2, 3, 6, 7] {
            let step = tour["step"][n]
                .as_object_mut()
                .ok_or_else(|| err("demo step"))?;
            step.remove("data");
            step.remove("avoid");
            step.insert("place".into(), json!({"ring": true}));
        }
        std::fs::write(
            self.0.join("walkthroughs/correction.json"),
            serde_json::to_vec_pretty(&tour).map_err(err)?,
        )
        .map_err(err)?;

        let orgs: Vec<(String, String)> = config["profiles"]["demo"]["orgs"]
            .as_object()
            .ok_or_else(|| err("demo orgs"))?
            .iter()
            .map(|(alias, id)| (alias.clone(), id.as_str().unwrap_or_default().into()))
            .collect();
        let ctx = ownpurse_core::view::Ctx {
            orgs: orgs.clone(),
            gate: vec![("hold".into(), "2025-12-31".into())],
        };
        let record = Record::open(self.0.join("record.sqlite")).map_err(err)?;
        let mut store = Store::open(&self.0.join("db/manifests.sqlite")).map_err(err)?;
        ops::create(
            &mut store,
            &record,
            &ctx,
            "human",
            "fix-igniter",
            "Synthetic demo: the igniter belongs in cupping-room supplies",
        )
        .map_err(err)?;
        let target = ops::Target {
            entity: "BankTransactions".into(),
            org: "roast".into(),
            id: Some("bt-110".into()),
        };
        let manifest = store
            .manifest("fix-igniter")
            .ok_or_else(|| err("demo manifest missing"))?;
        let view = ownpurse_core::view::View::open(&record, &ctx, Some(&manifest)).map_err(err)?;
        let before = ops::current(&view, &ctx, &target)
            .map_err(err)?
            .ok_or_else(|| err("demo payment missing"))?;
        let after = ops::recode_doc(&before, "n1", Some("6310"), None).map_err(err)?;
        ops::add(
            &mut store,
            &record,
            &ctx,
            "fix-igniter",
            "human",
            &target,
            &after,
            "Synthetic demo: move the 214.77 igniter",
            &Refs::default(),
            None,
            &ops::Guard::default(),
        )
        .map_err(err)?;
        ops::resolve(
            &mut store,
            "fix-igniter",
            "human",
            None,
            "reclass",
            &ops::Guard::default(),
        )
        .map_err(err)?;
        let manifest = store
            .manifest("fix-igniter")
            .ok_or_else(|| err("demo manifest missing"))?;
        let check = ownpurse_core::manifest::check::check(
            &record,
            &ctx,
            &manifest,
            &Default::default(),
            None,
        )
        .map_err(err)?;
        let plan_config = ownpurse_core::write::plan::PlanConfig {
            profile: "demo".into(),
            orgs,
            gates: ctx.gate.clone(),
            history_notes: false,
        };
        let package = compile::compile(
            &record,
            &ctx,
            &plan_config,
            &manifest,
            &check,
            "2026-10-08T12:00:00Z",
        )
        .map_err(err)?;
        let sealed = ownpurse_core::write::seal::seal(package).map_err(err)?;
        let path = self
            .0
            .join(format!("db/packages/{}.package.json", sealed.id));
        ownpurse_core::write::seal::save(&path, &sealed).map_err(err)?;
        ops::planned(
            &mut store,
            "fix-igniter",
            "human",
            &path.display().to_string(),
            sealed
                .seal
                .as_deref()
                .ok_or_else(|| err("demo seal missing"))?,
            manifest.content_rev,
        )
        .map_err(err)?;
        Ok(())
    }

    fn command(&self, program: &Path) -> Command {
        let mut cmd = Command::new(program);
        // Inherited project settings never select real stores, tokens, clocks or endpoints.
        for (key, _) in std::env::vars_os() {
            if key.to_string_lossy().starts_with("OWNPURSE_") {
                cmd.env_remove(key);
            }
        }
        cmd.current_dir(&self.0)
            .env("HOME", self.0.join("home"))
            .env("XDG_CONFIG_HOME", self.0.join("config"))
            .env("XDG_STATE_HOME", self.0.join("state"))
            .env("XDG_RUNTIME_DIR", self.0.join("run"))
            .args([
                "--config",
                "ownpurse.json",
                "--profile",
                "demo",
                "--offline",
                "--theme",
                "ledger-dark",
                "--label",
                "Scripted mock-data demo",
                "--now",
                "2026-10-08T12:00:00Z",
                "--walkthrough",
                "correction",
            ]);
        cmd
    }
}

fn request(socket: &Path, op: &str, fields: Value) -> Result<Value, String> {
    // A legitimate redraw can outlive two seconds when terminal output is backpressured.
    // Use the regular CLI bridge policy, including its bounded transport deadline.
    request_with_timeout(socket, op, fields, crate::ui::TIMEOUT)
}

fn request_with_timeout(
    socket: &Path,
    op: &str,
    fields: Value,
    timeout: Duration,
) -> Result<Value, String> {
    let fields = fields.as_object().cloned().unwrap_or_default();
    wire::request(socket, &wire::request_doc("view", op, fields), timeout)
}

fn pause(stop: &AtomicBool, delay: Duration) -> bool {
    let until = Instant::now() + delay;
    while Instant::now() < until {
        if stop.load(Ordering::Acquire) {
            return false;
        }
        std::thread::sleep(
            Duration::from_millis(50).min(until.saturating_duration_since(Instant::now())),
        );
    }
    !stop.load(Ordering::Acquire)
}

/// A view change from the person wins: a guarded send fails and autoplay never fights them.
fn drive(socket: &Path, delay: Duration, stop: &AtomicBool) -> Result<(), String> {
    let deadline = Instant::now() + Duration::from_secs(15);
    let first = loop {
        if stop.load(Ordering::Acquire) {
            return Ok(());
        }
        if socket.exists()
            && let Ok(response) = request(socket, "state", json!({}))
            && response["ok"] == true
        {
            break response;
        }
        if Instant::now() >= deadline {
            return Err("demo bridge did not become ready".into());
        }
        if !pause(stop, Duration::from_millis(50)) {
            return Ok(());
        }
    };
    let mut rev = first["state"]["rev"]
        .as_u64()
        .ok_or("demo state has no rev")?;
    for step in 1..=STEPS {
        if step > 1 && !pause(stop, delay) {
            return Ok(());
        }
        let caption = if step == STEPS {
            "Demo complete · mock data only · nothing posted · explore with ← / →, q quits"
                .to_string()
        } else {
            format!(
                "Scripted demo {step}/{STEPS} · mock data only · any view change takes control · q quits"
            )
        };
        let response = request(
            socket,
            "send",
            json!({
                "client": "demo-guide", "expect_rev": rev,
                "verbs": [["walkthrough", "step", step.to_string()], ["caption", caption]]
            }),
        )?;
        if response["ok"] != true {
            if matches!(
                response["error"].as_str(),
                Some("view_changed" | "user_busy" | "view_mismatch")
            ) {
                let _ = request(
                    socket,
                    "send",
                    json!({"client":"demo-guide", "verbs":[["caption","Demo paused · you have control · ← / → navigate, q quits"]]}),
                );
                return Ok(());
            }
            return Err(response["message"]
                .as_str()
                .unwrap_or("demo view refused")
                .into());
        }
        rev = response["state"]["rev"]
            .as_u64()
            .ok_or("demo send has no rev")?;
    }
    Ok(())
}

pub fn run(step_seconds: u64) -> Res<()> {
    if !std::io::stdin().is_terminal() || !std::io::stdout().is_terminal() {
        return Err(CliError::runtime(
            "demo_terminal_required",
            "The scripted demo needs an interactive terminal.",
        )
        .hint("Run `ownpurse demo` in your terminal; it uses mock data only."));
    }
    let program = std::env::current_exe()
        .map_err(err)?
        .parent()
        .ok_or_else(|| err("CLI location missing"))?
        .join("ownpurse-tui");
    if !program.is_file() {
        return Err(CliError::runtime(
            "demo_tui_missing",
            "The matching ownpurse-tui binary must be installed beside ownpurse.",
        )
        .hint("Build/install both binaries, then run `ownpurse demo`."));
    }
    let world = World::new()?;
    eprintln!("Demo: isolated mock data in {}", world.0.display());
    eprintln!(
        "Autoplay: {STEPS} steps, {step_seconds}s each. Move the view to take control; q quits."
    );
    let mut child = world.command(&program).spawn().map_err(err)?;
    let socket = world.dirs().socket_for(child.id());
    let stop = Arc::new(AtomicBool::new(false));
    let driver_stop = stop.clone();
    let driver = std::thread::spawn(move || {
        let result = drive(&socket, Duration::from_secs(step_seconds), &driver_stop);
        if let Err(message) = &result
            && !driver_stop.load(Ordering::Acquire)
        {
            let _ = request(
                &socket,
                "send",
                json!({"client":"demo-guide", "verbs":[["caption", format!("Demo paused: {message} · you can still explore or quit")]]}),
            );
        }
        result
    });
    let status = child.wait();
    stop.store(true, Ordering::Release);
    let result = driver.join().map_err(|_| err("demo driver panicked"))?;
    let status: ExitStatus = status.map_err(err)?;
    if !status.success() {
        return Err(err(format!("demo TUI exited with {status}")));
    }
    // On a normal early quit the socket may disappear while a send is in flight.
    if let Err(message) = result {
        eprintln!("demo: {message}");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mock_world_is_self_contained_and_cleaned_up() {
        let root;
        {
            let world = World::new().unwrap();
            root = world.0.clone();
            assert_eq!(std::fs::read(root.join("record.sqlite")).unwrap(), RECORD);
            let tour =
                ownpurse_core::walkthrough::read(&root.join("walkthroughs"), "correction").unwrap();
            assert_eq!(tour.steps.len(), STEPS as usize);
            assert!(ownpurse_core::walkthrough::check(&tour).is_empty());
            let store = Store::open(&root.join("db/manifests.sqlite")).unwrap();
            let manifest = store.manifest("fix-igniter").unwrap();
            assert_eq!(manifest.content_rev, 3);
            assert!(world.dirs().socket_for(12345).as_os_str().len() < wire::SOCKET_PATH_MAX);
            let cmd = world.command(Path::new("/synthetic/ownpurse-tui"));
            let args: Vec<_> = cmd
                .get_args()
                .map(|x| x.to_string_lossy().into_owned())
                .collect();
            assert!(args.iter().any(|a| a == "--offline"));
            assert_eq!(cmd.get_current_dir(), Some(root.as_path()));
            assert!(
                root.join("db/packages/fix-igniter-r3.package.json")
                    .is_file()
            );
        }
        assert!(!root.exists());
    }

    #[cfg(unix)]
    fn fake_bridge(takeover: bool) -> Vec<Value> {
        use std::io::{BufRead, BufReader, Write};
        use std::os::unix::net::UnixListener;
        let world = World::new().unwrap();
        let socket = world.0.join("test.sock");
        let listener = UnixListener::bind(&socket).unwrap();
        listener.set_nonblocking(true).unwrap();
        let count = if takeover { 4 } else { 11 };
        let server = std::thread::spawn(move || {
            let mut requests = vec![];
            let deadline = Instant::now() + Duration::from_secs(5);
            while requests.len() < count {
                let (mut connection, _) = match listener.accept() {
                    Ok(c) => c,
                    Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        assert!(Instant::now() < deadline, "driver did not finish");
                        std::thread::sleep(Duration::from_millis(1));
                        continue;
                    }
                    Err(e) => panic!("{e}"),
                };
                connection.set_nonblocking(false).unwrap();
                connection
                    .set_read_timeout(Some(Duration::from_secs(2)))
                    .unwrap();
                let mut line = String::new();
                BufReader::new(&connection).read_line(&mut line).unwrap();
                requests.push(serde_json::from_str::<Value>(&line).unwrap());
                let n = requests.len();
                let response = if takeover && n == 3 {
                    json!({"ok":false, "error":"view_changed", "message":"person moved"})
                } else {
                    json!({"ok":true, "state":{"rev":n + 1}})
                };
                writeln!(connection, "{response}").unwrap();
            }
            requests
        });
        drive(&socket, Duration::ZERO, &AtomicBool::new(false)).unwrap();
        server.join().unwrap()
    }

    #[cfg(unix)]
    #[test]
    fn autoplay_sends_every_step_through_the_guarded_view_bridge() {
        let requests = fake_bridge(false);
        assert_eq!(requests[0]["op"], "state");
        for (n, request) in requests.iter().skip(1).enumerate() {
            assert_eq!(request["kind"], "view");
            assert_eq!(request["op"], "send");
            assert_eq!(request["expect_rev"], n + 2);
            assert_eq!(
                request["verbs"][0],
                json!(["walkthrough", "step", (n + 1).to_string()])
            );
        }
        assert!(
            requests[10]["verbs"][1][1]
                .as_str()
                .unwrap()
                .contains("Demo complete")
        );
    }

    #[cfg(unix)]
    #[test]
    fn a_persons_view_change_stops_autoplay_without_retrying() {
        let requests = fake_bridge(true);
        assert_eq!(requests.len(), 4);
        assert!(requests[3].get("expect_rev").is_none());
        assert_eq!(requests[3]["verbs"].as_array().unwrap().len(), 1);
        assert!(
            requests[3]["verbs"][0][1]
                .as_str()
                .unwrap()
                .contains("you have control")
        );
    }

    #[cfg(unix)]
    fn delayed_request(
        delay: Duration,
        timeout: Option<Duration>,
    ) -> (Result<Value, String>, Duration) {
        use std::io::{BufRead, BufReader, Write};
        use std::os::unix::net::UnixListener;
        let world = World::new().unwrap();
        let socket = world.0.join("slow.sock");
        let listener = UnixListener::bind(&socket).unwrap();
        let (release, hold) = std::sync::mpsc::channel();
        let server = std::thread::spawn(move || {
            let (mut connection, _) = listener.accept().unwrap();
            connection
                .set_read_timeout(Some(Duration::from_secs(5)))
                .unwrap();
            let mut line = String::new();
            BufReader::new(&connection).read_line(&mut line).unwrap();
            let doc: Value = serde_json::from_str(&line).unwrap();
            assert_eq!(doc["op"], "send");
            assert_eq!(doc["verbs"][0], json!(["walkthrough", "step", "3"]));
            if timeout.is_some() {
                // Hold the reply until the client returns: the timeout test cannot race a fast server.
                hold.recv_timeout(Duration::from_secs(5)).unwrap();
            } else {
                std::thread::sleep(delay);
            }
            // A genuinely timed-out client closes its connection; a late reply must not be retried.
            let _ = writeln!(connection, "{}", json!({"ok":true, "state":{"rev":3}}));
        });
        let start = Instant::now();
        let fields = json!({"verbs":[["walkthrough", "step", "3"]]});
        let result = match timeout {
            Some(limit) => request_with_timeout(&socket, "send", fields, limit),
            None => request(&socket, "send", fields),
        };
        let elapsed = start.elapsed();
        let _ = release.send(());
        server.join().unwrap();
        (result, elapsed)
    }

    #[cfg(unix)]
    #[test]
    fn a_slow_legitimate_reply_uses_the_regular_cli_bridge_budget() {
        let (reply, elapsed) = delayed_request(Duration::from_millis(2_100), None);
        assert_eq!(reply.unwrap()["state"]["rev"], 3);
        assert!(elapsed >= Duration::from_secs(2));
    }

    #[cfg(unix)]
    #[test]
    fn a_genuine_transport_timeout_remains_an_error_without_retry() {
        let (reply, _) =
            delayed_request(Duration::from_millis(100), Some(Duration::from_millis(20)));
        assert!(reply.unwrap_err().contains("no answer from the TUI"));
    }

    #[test]
    fn cancellation_does_not_wait_for_the_step_delay() {
        let stopped = AtomicBool::new(true);
        assert!(!pause(&stopped, Duration::from_secs(60)));
    }
}
