//! The docs-site inputs, from the DEMO fixture only (synthetic orgs and ids):
//!
//!     cargo run -q -p xtask -- docs-inputs <out-dir>
//!
//! Writes install.md, config.md, scopes.md, cli.md (the CLI reference with a real example under each command),
//! doctor.txt, examples/config-min.json, examples/quickstart.txt, keys.md and renders/{ledger-dark,ledger-light}.
//! Every command runs in a sandbox: a temp copy of the demo config and record, HOME set to the sandbox, a fake
//! token file (mode 600) and PATH holding only the built binaries, so no real path, record or token is
//! reachable. The hygiene rules and the private denylist then run on the output.

use crate::files;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;

const AS_OF: &str = "2025-12-31";

/// command → example argv (after `ownpurse`); short with --limit
fn examples() -> Vec<(&'static str, Vec<&'static str>)> {
    vec![
        ("doctor", vec!["doctor"]),
        ("accounts", vec!["accounts", "roast", "--limit", "8"]),
        ("org", vec!["org", "roast"]),
        ("txns", vec!["txns", "roast", "--limit", "5"]),
        ("journals", vec!["journals", "roast", "--limit", "5"]),
        ("transfers", vec!["transfers", "roast", "--limit", "5"]),
        ("show", vec!["show", "roast", "Accounts", "<ID>"]),
        ("list", vec!["list", "roast", "Invoices", "--limit", "5"]),
        ("gl", vec!["gl", "roast", "--limit", "8"]),
        ("tb", vec!["tb", "roast", "--date", AS_OF]),
        ("parity", vec!["parity", "roast", "--date", AS_OF]),
        ("usage", vec!["usage"]),
        ("config", vec!["config", "path"]),
        ("ic", vec!["ic"]),
    ]
}

struct Sandbox {
    dir: PathBuf,
    home: PathBuf,
    proj: PathBuf,
}

impl Sandbox {
    fn new(bins: &Path, demo: &Path) -> std::io::Result<Sandbox> {
        let dir = files::temp("docs-inputs");
        let home = dir.join("home/demo");
        let proj = home.join("roast-books");
        std::fs::create_dir_all(proj.join("tokens"))?;
        std::fs::create_dir_all(home.join("bin"))?;
        std::fs::copy(demo.join("ownpurse.json"), proj.join("ownpurse.json"))?;
        std::fs::copy(demo.join("record.sqlite"), proj.join("record.sqlite"))?;
        let tok = proj.join("tokens/demo.json");
        std::fs::write(&tok, r#"{"note": "demo token file (not a real token)"}"#)?;
        std::fs::set_permissions(&tok, std::fs::Permissions::from_mode(0o600))?;
        for b in ["ownpurse", "ownpurse-tui"] {
            std::os::unix::fs::symlink(bins.join(b), home.join("bin").join(b))?;
        }
        Ok(Sandbox { dir, home, proj })
    }

    fn cmd(&self, program: &Path) -> Command {
        let mut c = Command::new(program);
        c.env_clear()
            .env("HOME", &self.home)
            .env(
                "PATH",
                format!("{}:/usr/bin:/bin", self.home.join("bin").display()),
            )
            .env("LANG", "en_US.UTF-8")
            .env("NO_COLOR", "1")
            .current_dir(&self.proj);
        c
    }

    /// `ownpurse <argv>` in the sandbox; the sandbox's temp prefix is removed from the output.
    fn run(&self, argv: &[&str]) -> String {
        let out = self
            .cmd(&self.home.join("bin/ownpurse"))
            .args(argv)
            .output()
            .expect("run ownpurse");
        let text = format!(
            "{}{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
        let real = std::fs::canonicalize(&self.dir).unwrap_or(self.dir.clone());
        text.replace(&real.display().to_string(), "")
            .replace(&self.dir.display().to_string(), "")
    }
}

impl Drop for Sandbox {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

fn fence(argv: &[&str], out: &str) -> String {
    format!(
        "```text\n$ ownpurse {}\n{}\n```\n",
        argv.join(" "),
        out.trim_end()
    )
}

fn fail(msg: &str) -> i32 {
    eprintln!("docs-inputs: {msg}");
    1
}

pub fn run(out: &Path) -> i32 {
    let root = files::root();
    let rust = root.join("rust");
    let demo = rust.join("tests/fixtures/demo");
    let build = Command::new(env!("CARGO"))
        .args([
            "build",
            "--quiet",
            "-p",
            "ownpurse-cli",
            "-p",
            "ownpurse-tui",
            "-p",
            "ownpurse-xero",
        ])
        .current_dir(&rust)
        .status();
    if !build.is_ok_and(|s| s.success()) {
        return fail("cargo build failed");
    }
    let bins = rust.join("target/debug");
    if std::fs::create_dir_all(out.join("examples")).is_err() {
        return fail("cannot create the output folder");
    }
    let sb = match Sandbox::new(&bins, &demo) {
        Ok(s) => s,
        Err(e) => return fail(&format!("sandbox: {e}")),
    };

    // an account id for `show`
    let accts = sb.run(&["--json", "accounts", "roast", "--bank"]);
    let first: Option<serde_json::Value> = serde_json::Deserializer::from_str(accts.trim_start())
        .into_iter::<serde_json::Value>()
        .next()
        .and_then(Result::ok);
    let acct_id = first.as_ref().and_then(|d| {
        let rows = d
            .get("accounts")
            .and_then(|v| v.as_array())
            .or_else(|| d.as_object()?.values().find_map(|v| v.as_array()))?;
        rows.iter().find_map(|r| {
            ["id", "account_id", "AccountID"]
                .iter()
                .find_map(|k| r.get(*k).and_then(|v| v.as_str()).map(str::to_string))
        })
    });

    let mut md = sb.run(&["help-markdown"]);
    for (name, argv) in examples() {
        let marker = format!("<!-- example:{name} -->");
        let argv: Vec<String> = if argv.contains(&"<ID>") {
            let Some(id) = &acct_id else {
                md = md.replace(&format!("{marker}\n"), "");
                continue;
            };
            argv.iter()
                .map(|a| {
                    if *a == "<ID>" {
                        id.chars().take(8).collect()
                    } else {
                        a.to_string()
                    }
                })
                .collect()
        } else {
            argv.iter().map(|a| a.to_string()).collect()
        };
        let argv: Vec<&str> = argv.iter().map(String::as_str).collect();
        let ex = format!("Example (demo data):\n\n{}", fence(&argv, &sb.run(&argv)));
        md = md.replace(&marker, &ex);
    }
    md = md.replace(
        "<!-- example:tui -->",
        "See the TUI pages for screens and keys (`keys.md`).",
    );
    let leftover = regex::Regex::new(r"<!-- example:[a-z-]+ -->\n\n").expect("regex");
    md = leftover.replace_all(&md, "").to_string();
    let write = |name: &str, body: String| std::fs::write(out.join(name), body);
    let mut res = write("cli.md", md);

    res = res.and(write(
        "doctor.txt",
        format!("$ ownpurse doctor\n{}\n", sb.run(&["doctor"]).trim_end()),
    ));
    let quick: [&[&str]; 2] = [
        &["tb", "roast", "--date", AS_OF],
        &["parity", "roast", "--date", AS_OF],
    ];
    let mut q = String::new();
    for a in quick {
        q.push_str(&format!(
            "$ ownpurse {}\n{}\n\n",
            a.join(" "),
            sb.run(a).trim_end()
        ));
    }
    q.push_str("$ ownpurse tui\n(opens the terminal UI)\n");
    res = res.and(write("examples/quickstart.txt", q));

    let minimal = serde_json::json!({"default_profile": "demo", "profiles": {"demo": {
        "client_id": "YOUR_XERO_APP_CLIENT_ID", "store": "record.sqlite",
        "orgs": {"roast": "c0ffee00-0000-4000-8000-0000000000a1"}}}});
    let min_text = serde_json::to_string_pretty(&minimal).expect("json");
    let _ = std::fs::write(sb.proj.join("ownpurse.json"), &min_text);
    let check = sb.run(&["tb", "roast", "--date", AS_OF]);
    if check.contains("error:") {
        return fail(&format!(
            "config-min.json is not valid against the fixture:\n{check}"
        ));
    }
    res = res.and(write("examples/config-min.json", format!("{min_text}\n")));
    let _ = std::fs::copy(demo.join("ownpurse.json"), sb.proj.join("ownpurse.json"));

    let keys = sb
        .cmd(&bins.join("ownpurse-tui"))
        .args(["--config"])
        .arg(sb.proj.join("ownpurse.json"))
        .args(["--profile", "demo", "--keys", "markdown"])
        .output();
    match keys {
        Ok(k) => res = res.and(std::fs::write(out.join("keys.md"), k.stdout)),
        Err(e) => return fail(&format!("keys: {e}")),
    }
    drop(sb);

    res = res
        .and(std::fs::copy(root.join("docs/INSTALL.md"), out.join("install.md")).map(|_| ()))
        .and(std::fs::copy(root.join("docs/CONFIG.md"), out.join("config.md")).map(|_| ()));
    let defaults: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(rust.join("crates/ownpurse-core/defaults.json"))
            .unwrap_or_default(),
    )
    .unwrap_or_default();
    let list = |k: &str| -> String {
        defaults["xero"]["scopes"][k]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(|s| s.as_str())
            .map(|s| format!("- `{s}`\n"))
            .collect()
    };
    res = res.and(write(
        "scopes.md",
        format!(
            "# Scopes\n\nLogin requests the read scopes below **plus any scope already granted** on the token \
(consent is additive, so a re-login never drops a permission). `ownpurse login` and the TUI never add a write \
scope.\n\n## Read (requested by `login`)\n\n{}\n## Write (reserved for the approved write protocol; never \
requested by login)\n\n{}",
            list("read"),
            list("write")
        ),
    ));
    if let Err(e) = res {
        return fail(&format!("write: {e}"));
    }

    for theme in ["ledger-dark", "ledger-light"] {
        let st = Command::new(rust.join("scripts/tui-snapshots.sh"))
            .arg(out.join("renders").join(theme))
            .args([theme, "", "txt,html"])
            .status();
        if !st.is_ok_and(|s| s.success()) {
            return fail(&format!("renders ({theme})"));
        }
    }

    // the output must pass the public hygiene rules and the private denylist
    let rules = crate::hygiene::Rules::new(crate::hygiene::allowed_uuids(&root));
    let items = files::under(out);
    let hits = crate::hygiene::check(&items, &rules);
    for h in &hits {
        println!("docs-inputs: {h}");
    }
    let deny = crate::denylist::run(&["--no-tracked".to_string(), out.display().to_string()]);
    println!("docs-inputs: {} files in {}", items.len(), out.display());
    if hits.is_empty() && deny == 0 { 0 } else { 1 }
}
