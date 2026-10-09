//! Generic, public repository hygiene (no private data involved):
//! - no files of the retired toolchain (`*.py`, its project file, its lock file) and no script the gate runs
//!   (rust/scripts, .githooks, .github/workflows, install.sh) invoking that interpreter or its runner;
//! - no SQLite `-wal` / `-shm` side files;
//! - no real paths: a user home (`/Users/<name>`, `/home/<name>` other than the synthetic `demo` and `me`) or the
//!   real token file path outside the fakes;
//! - no UUID that `rust/scripts/uuid-allow.txt` doesn't list (text and binary fixtures alike);
//! - every tracked config (`ownpurse*.json`) uses allow-listed (synthetic) tenant ids and a placeholder
//!   client id, never one shaped like a real Xero client id (32 hex characters).

use crate::files::{self, Item};
use regex::bytes::Regex;
use std::collections::HashSet;
use std::path::Path;

/// The retired toolchain's names, built from parts so this file never matches its own search.
fn retired() -> (String, String, String) {
    (
        ["py", "thon"].concat(),
        ["py", "project.toml"].concat(),
        ["u", "v.lock"].concat(),
    )
}

/// Paths where a real-looking token path is expected (the fakes that print one).
const FAKES: [&str; 1] = ["rust/tests/fixtures/fake-cli/"];

pub struct Rules {
    uuid: Regex,
    home: Regex,
    token_path: Regex,
    invoke: Regex,
    allowed_uuids: HashSet<String>,
}

impl Rules {
    pub fn new(allowed_uuids: HashSet<String>) -> Rules {
        let (py, _, _) = retired();
        Rules {
            uuid: Regex::new(
                r"(?i)\b[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}\b",
            )
            .expect("uuid regex"),
            home: Regex::new(r"/(?:Users|home)/([A-Za-z][A-Za-z0-9._-]*)").expect("home regex"),
            token_path: Regex::new(r"~/\.xero/tokens\.json").expect("token path"),
            invoke: Regex::new(&format!(
                r"(?i)\b{py}[0-9.]*\b|\bu{v}\s+(?:run|sync|pip|build)\b|\.{v}env\b|\.py\b",
                v = "v"
            ))
            .expect("invoke regex"),
            allowed_uuids,
        }
    }
}

/// The allow-listed UUIDs (lower case; `#` comments).
pub fn allowed_uuids(root: &Path) -> HashSet<String> {
    let text =
        std::fs::read_to_string(root.join("rust/scripts/uuid-allow.txt")).unwrap_or_default();
    parse_allow(&text)
}

fn parse_allow(text: &str) -> HashSet<String> {
    text.lines()
        .map(|l| l.split('#').next().unwrap_or("").trim().to_lowercase())
        .filter(|l| !l.is_empty())
        .collect()
}

/// Every finding in these files (empty: clean).
pub fn check(items: &[Item], rules: &Rules) -> Vec<String> {
    let (_, project_file, lock_file) = retired();
    let mut hits = Vec::new();
    for it in items {
        let base = it.name.rsplit('/').next().unwrap_or(&it.name);
        if it.name.ends_with(".py") || base == project_file || base == lock_file {
            hits.push(format!("{}: a file of the retired toolchain", it.name));
        }
        if it.name.ends_with(".sqlite-wal") || it.name.ends_with(".sqlite-shm") {
            hits.push(format!("{}: a SQLite side file", it.name));
        }
        let gate_script = it.name.starts_with("rust/scripts/")
            || it.name.starts_with(".githooks/")
            || it.name.starts_with(".github/workflows/")
            || it.name == "install.sh";
        if gate_script && let Some(m) = rules.invoke.find(&it.bytes) {
            hits.push(format!(
                "{}: invokes the retired toolchain",
                files::place(it, m.start())
            ));
        }
        for c in rules.home.captures_iter(&it.bytes) {
            let who = String::from_utf8_lossy(&c[1]).to_string();
            if who != "demo" && who != "me" {
                let m = c.get(0).map_or(0, |m| m.start());
                hits.push(format!("{}: a user home path", files::place(it, m)));
            }
        }
        if !FAKES.iter().any(|f| it.name.starts_with(f))
            && let Some(m) = rules.token_path.find(&it.bytes)
        {
            hits.push(format!(
                "{}: a real token file path",
                files::place(it, m.start())
            ));
        }
        for m in rules.uuid.find_iter(&it.bytes) {
            let u = String::from_utf8_lossy(m.as_bytes()).to_lowercase();
            if !rules.allowed_uuids.contains(&u) {
                hits.push(format!(
                    "{}: UUID not in uuid-allow.txt ({}…)",
                    files::place(it, m.start()),
                    &u[..8]
                ));
            }
        }
        if (base.starts_with("ownpurse") && base.ends_with(".json"))
            && let Some(t) = files::text(&it.bytes)
            && let Ok(v) = serde_json::from_str::<serde_json::Value>(t)
            && v.get("profiles").is_some()
        {
            hits.extend(config_hits(&it.name, &v, rules));
        }
    }
    hits
}

/// A tracked config: synthetic tenants, a placeholder client id.
fn config_hits(name: &str, v: &serde_json::Value, rules: &Rules) -> Vec<String> {
    let mut hits = Vec::new();
    let real_client = regex::Regex::new(r"^[0-9A-Fa-f]{32}$").expect("client regex");
    for (pname, p) in v["profiles"].as_object().into_iter().flatten() {
        for (alias, tenant) in p["orgs"].as_object().into_iter().flatten() {
            let t = tenant.as_str().unwrap_or("").to_lowercase();
            if !rules.allowed_uuids.contains(&t) {
                hits.push(format!(
                    "{name}: profile {pname} org {alias}: tenant id not synthetic (uuid-allow.txt)"
                ));
            }
        }
        if let Some(c) = p["client_id"].as_str()
            && real_client.is_match(c)
        {
            hits.push(format!(
                "{name}: profile {pname}: client_id looks real (use a placeholder)"
            ));
        }
    }
    hits
}

pub fn run() -> i32 {
    let root = files::root();
    let rules = Rules::new(allowed_uuids(&root));
    let items = files::tracked(&root);
    let hits = check(&items, &rules);
    for h in &hits {
        println!("hygiene: {h}");
    }
    if hits.is_empty() {
        println!("hygiene: OK ({} tracked files)", items.len());
        0
    } else {
        println!("hygiene: {} finding(s)", hits.len());
        1
    }
}

/// Planted problems are each caught; a clean tree passes.
pub fn self_test() -> i32 {
    let (py, project_file, lock_file) = retired();
    let ok_uuid = "00000000-0000-0000-0000-000000000000";
    let rules = Rules::new(parse_allow(ok_uuid));
    let item = |name: &str, body: &str| Item {
        name: name.into(),
        bytes: body.as_bytes().to_vec(),
    };
    let planted = vec![
        (item("tools/gen.py", "x = 1"), "retired toolchain"),
        (item(&project_file, "[project]"), "retired toolchain"),
        (item(&format!("sub/{lock_file}"), ""), "retired toolchain"),
        (
            item("rust/scripts/x.sh", &format!("{py}3 scripts/gen")),
            "invokes",
        ),
        (
            item(".githooks/pre-push", &["u", "v run thing"].concat()),
            "invokes",
        ),
        (item("rust/tests/fixtures/a.sqlite-wal", ""), "side file"),
        (
            item("docs/a.md", &["see /Us", "ers/someone/x"].concat()),
            "home path",
        ),
        (
            item("docs/c.md", &["token at ~/.xe", "ro/tokens.json"].concat()),
            "token file",
        ),
        (
            item(
                "docs/d.md",
                &["id 12345678-1234-", "4234-8234-123456789abc"].concat(),
            ),
            "UUID",
        ),
        (
            item(
                "rust/x/ownpurse.json",
                &format!(
                    r#"{{"profiles": {{"p": {{"client_id": "{}", "orgs": {{"a": "{}"}}}}}}}}"#,
                    "0123456789ABCDEF".repeat(2),
                    ["aaaaaaaa-0000-", "4000-8000-000000000000"].concat()
                ),
            ),
            "client_id",
        ),
        (
            item(
                "rust/y/ownpurse.json",
                &format!(
                    r#"{{"profiles": {{"p": {{"orgs": {{"a": "{}"}}}}}}}}"#,
                    ["aaaaaaaa-0000-", "4000-8000-000000000000"].concat()
                ),
            ),
            "tenant id",
        ),
    ];
    let mut ok = true;
    for (it, want) in &planted {
        let hits = check(std::slice::from_ref(it), &rules);
        if !hits.iter().any(|h| h.contains(want)) {
            println!(
                "hygiene self-test: {} ({want}) NOT caught: {hits:?}",
                it.name
            );
            ok = false;
        }
    }
    let clean = vec![
        item(
            "docs/ok.md",
            "HOME=/home/demo and /home/me; tenant 00000000-0000-0000-0000-000000000000",
        ),
        item(
            "rust/z/ownpurse.json",
            &format!(
                r#"{{"profiles": {{"p": {{"client_id": "YOUR_XERO_APP_CLIENT_ID", "orgs": {{"a": "{ok_uuid}"}}}}}}}}"#
            ),
        ),
        item(
            "rust/tests/fixtures/fake-cli/ownpurse",
            &["echo ~/.xe", "ro/tokens.json"].concat(),
        ),
        item("rust/scripts/check.sh", "cargo test --workspace"),
    ];
    let hits = check(&clean, &rules);
    if !hits.is_empty() {
        println!("hygiene self-test: clean files flagged: {hits:?}");
        ok = false;
    }
    println!("hygiene self-test: {}", if ok { "OK" } else { "FAILED" });
    if ok { 0 } else { 1 }
}

#[cfg(test)]
mod tests {
    #[test]
    fn self_test_passes() {
        assert_eq!(super::self_test(), 0);
    }
}
