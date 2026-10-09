//! `ownpurse classify …` goldens and the V14 write harness, on a temp copy of the demo fixture
//! (`rust/tests/fixtures/demo`: record, `classify.toml`, `db/classify.sqlite`).
//!
//! Goldens are text (`tests/fixtures/demo/classify-goldens/<case>.txt`: the argv, the exit code, then stdout with
//! the temp folder written as `<DEMO>`). Each case runs on a fresh copy. Re-bless with
//! `OWNPURSE_BLESS=1 cargo test -p ownpurse-cli --test classify bless`.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;

fn demo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/demo")
}

fn goldens() -> PathBuf {
    demo().join("classify-goldens")
}

/// A fresh copy of the demo world with its classification files.
fn copy(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("xa-classify-cli-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(d.join("db")).unwrap();
    for f in ["ownpurse.json", "record.sqlite", "classify.toml"] {
        std::fs::copy(demo().join(f), d.join(f)).unwrap();
    }
    std::fs::copy(
        demo().join("db/classify.sqlite"),
        d.join("db/classify.sqlite"),
    )
    .unwrap();
    d
}

fn run(dir: &Path, args: &[&str]) -> (i32, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_ownpurse"))
        .args(args)
        .current_dir(dir)
        .env("OWNPURSE_CONFIG", dir.join("ownpurse.json"))
        .env_remove("OWNPURSE_PROFILE")
        .env_remove("OWNPURSE_STORE")
        .env_remove("OWNPURSE_CLIENT_ID")
        .env_remove("XDG_CONFIG_HOME")
        .env_remove("XDG_DATA_HOME")
        .env("OWNPURSE_NOW", "2026-10-05T09:46Z")
        .env("HOME", "/home/demo")
        .output()
        .expect("run ownpurse");
    let text = String::from_utf8(out.stdout)
        .expect("utf-8")
        .replace(&dir.display().to_string(), "<DEMO>");
    (out.status.code().unwrap_or(-1), text)
}

/// A case may first edit the copied config.
type Setup = fn(&Path);

fn no_setup(_: &Path) {}

fn bad_config(d: &Path) {
    let p = d.join("classify.toml");
    let t = std::fs::read_to_string(&p).unwrap();
    let t = t
        .replace(
            "set = { bearer = \"roast\", treatment = \"intercompany\" }",
            "set = { bearer = \"rost\", treatment = \"intercompany\" }",
        )
        .replace("payee_has = [\"fuel\"]", "payee_hass = [\"fuel\"]");
    std::fs::write(p, t).unwrap();
}

fn jev_on(d: &Path) {
    let p = d.join("classify.toml");
    let t = std::fs::read_to_string(&p).unwrap();
    std::fs::write(p, t.replace("enabled = false", "enabled = true")).unwrap();
}

/// A new rule, so `suggest` has something to write.
fn new_rule(d: &Path) {
    let p = d.join("classify.toml");
    let t = std::fs::read_to_string(&p).unwrap();
    let rule = "[[rules]]\nid = \"r-green-beans\"\npriority = 75\nwhen = { account_in = [\"5020\"] }\nset = { project = \"none\", treatment = \"unknown\" }\nconfidence = 0.65\n\n";
    std::fs::write(
        p,
        t.replace(
            "[[rules]]\nid = \"r-fuel\"",
            &format!("{rule}[[rules]]\nid = \"r-fuel\""),
        ),
    )
    .unwrap();
}

fn scope_2025(d: &Path) {
    let p = d.join("classify.toml");
    let t = std::fs::read_to_string(&p).unwrap();
    std::fs::write(
        p,
        t.replace(
            "threshold = 0.90",
            "threshold = 0.90\nscope = \"year:2025\"",
        ),
    )
    .unwrap();
}

/// No store yet: reads must not create one.
fn no_store(d: &Path) {
    std::fs::remove_dir_all(d.join("db")).unwrap();
}

/// Sixty more lab lines in one bank transaction (a version appended to the copied record), so a filter can
/// pass the 50-line agent limit.
fn sixty_lines(d: &Path) {
    let c = rusqlite::Connection::open(d.join("record.sqlite")).unwrap();
    let items: Vec<serde_json::Value> = (0..60)
        .map(|i| serde_json::json!({"AccountCode": "5510", "Description": format!("Sample cup lot {i}"), "LineAmount": "3.17", "TaxAmount": 0}))
        .collect();
    let doc = serde_json::json!({"BankAccount": {"AccountID": "l-card"}, "BankTransactionID": "bt-499",
        "Contact": {"Name": "Ridgeback Lab Glassware"}, "DateString": "2025-06-03T00:00:00", "IsReconciled": false,
        "LineAmountTypes": "NoTax", "LineItems": items, "Status": "AUTHORISED", "Total": "190.20", "Type": "SPEND"});
    c.execute(
        "INSERT INTO versions (tenant_id, entity, xero_id, version, content_hash, status, observed_at, source, payload) \
         VALUES ('1ab00000-0000-4000-8000-0000000000d4', 'BankTransactions', 'bt-499', 1, 't', 'AUTHORISED', '2026-10-05T07:46:00Z', 'sync', ?1)",
        [doc.to_string()],
    )
    .unwrap();
}

fn bad_pair_org(d: &Path) {
    let p = d.join("classify.toml");
    let t = std::fs::read_to_string(&p).unwrap();
    std::fs::write(
        p,
        t.replace("a = { org = \"hold\"", "a = { org = \"holding\""),
    )
    .unwrap();
}

const W: [&str; 4] = ["--expect-store-rev", "82", "--client", "claude"];

const CASES: &[(&str, &[&str], Setup)] = &[
    (
        "rules-check-pair-org",
        &["classify", "rules", "check"],
        bad_pair_org,
    ),
    ("queue-pair-org", &["classify", "queue"], bad_pair_org),
    (
        "write-group-create",
        &[
            "classify",
            "group",
            "create",
            "split",
            "Cupping kit split",
            W[0],
            W[1],
            W[2],
            W[3],
        ],
        no_setup,
    ),
    (
        "write-group-remove",
        &[
            "classify",
            "group",
            "remove",
            "g70",
            "roast:bt:bt-108:L1",
            W[0],
            W[1],
            W[2],
            W[3],
        ],
        no_setup,
    ),
    (
        "write-group-status",
        &[
            "classify", "group", "status", "g70", "complete", W[0], W[1], W[2], W[3],
        ],
        no_setup,
    ),
    (
        "write-refused-group-status",
        &[
            "classify", "group", "status", "g70", "finished", W[0], W[1], W[2], W[3],
        ],
        no_setup,
    ),
    (
        "write-refused-group-remove-nonmember",
        &[
            "classify",
            "group",
            "remove",
            "g70",
            "lab:bt:bt-401:L1",
            W[0],
            W[1],
            W[2],
            W[3],
        ],
        no_setup,
    ),
    (
        "write-set-anchors",
        &[
            "classify",
            "set",
            "roast:bt:bt-104:L1",
            "roast:bt:bt-109:L1",
            "project=harvest",
            "treatment=unknown",
            W[0],
            W[1],
            W[2],
            W[3],
            "--caption",
            "Green coffee for the launch",
        ],
        no_setup,
    ),
    (
        "write-set-filter-skips-human",
        &[
            "classify",
            "set",
            "org:design",
            "status:all",
            "project=refit",
            W[0],
            W[1],
            W[2],
            W[3],
        ],
        no_setup,
    ),
    (
        "write-set-overwrite-json",
        &[
            "classify",
            "set",
            "design:bt:bt-303:L1",
            "project=refit",
            W[0],
            W[1],
            W[2],
            W[3],
            "--overwrite",
            "--json",
        ],
        no_setup,
    ),
    (
        "write-refused-view-changed",
        &[
            "classify",
            "set",
            "roast:bt:bt-104:L1",
            "project=harvest",
            "--expect-store-rev",
            "80",
            "--client",
            "claude",
        ],
        no_setup,
    ),
    (
        "write-refused-view-changed-json",
        &[
            "classify",
            "set",
            "roast:bt:bt-104:L1",
            "project=harvest",
            "--expect-store-rev",
            "80",
            "--client",
            "claude",
            "--json",
        ],
        no_setup,
    ),
    (
        "write-refused-too-many",
        &[
            "classify",
            "set",
            "org:lab",
            "project=cupping",
            W[0],
            W[1],
            W[2],
            W[3],
        ],
        sixty_lines,
    ),
    (
        "write-allow-bulk",
        &[
            "classify",
            "set",
            "org:lab",
            "project=cupping",
            W[0],
            W[1],
            W[2],
            W[3],
            "--allow-bulk",
            "--limit",
            "3",
        ],
        sixty_lines,
    ),
    (
        "write-refused-bad-caption",
        &[
            "classify",
            "note",
            "lab:bt:bt-402:L1",
            "ok",
            W[0],
            W[1],
            W[2],
            W[3],
            "--caption",
            "two\u{1b}[31m",
        ],
        no_setup,
    ),
    (
        "write-refused-long-caption",
        &[
            "classify",
            "note",
            "lab:bt:bt-402:L1",
            "ok",
            W[0],
            W[1],
            W[2],
            W[3],
            "--caption",
            "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.",
        ],
        no_setup,
    ),
    (
        "write-refused-no-rev",
        &[
            "classify",
            "set",
            "roast:bt:bt-104:L1",
            "project=harvest",
            "--client",
            "claude",
        ],
        no_setup,
    ),
    (
        "write-refused-no-client",
        &[
            "classify",
            "set",
            "roast:bt:bt-104:L1",
            "project=harvest",
            "--expect-store-rev",
            "82",
        ],
        no_setup,
    ),
    (
        "write-refused-no-labels",
        &[
            "classify",
            "set",
            "roast:bt:bt-104:L1",
            W[0],
            W[1],
            W[2],
            W[3],
        ],
        no_setup,
    ),
    (
        "write-refused-unknown-choice",
        &[
            "classify",
            "set",
            "roast:bt:bt-104:L1",
            "project=harvset",
            W[0],
            W[1],
            W[2],
            W[3],
        ],
        no_setup,
    ),
    (
        "write-refused-unknown-line",
        &[
            "classify",
            "set",
            "roast:bt:bt-999:L1",
            "project=harvest",
            W[0],
            W[1],
            W[2],
            W[3],
        ],
        no_setup,
    ),
    (
        "write-refused-session",
        &[
            "classify",
            "set",
            "roast:bt:bt-104:L1",
            "project=harvest",
            W[0],
            W[1],
            W[2],
            W[3],
            "--session",
            "s-none",
        ],
        no_setup,
    ),
    (
        "write-note",
        &[
            "classify",
            "note",
            "lab:bt:bt-402:L1",
            "Founding members, see the register",
            W[0],
            W[1],
            W[2],
            W[3],
        ],
        no_setup,
    ),
    (
        "write-note-group",
        &[
            "classify",
            "note",
            "g76",
            "Two visits, one invoice",
            W[0],
            W[1],
            W[2],
            W[3],
        ],
        no_setup,
    ),
    (
        "write-group-add",
        &[
            "classify",
            "group",
            "add",
            "g76",
            "roast:bt:bt-110:L1",
            "--role",
            "origin",
            W[0],
            W[1],
            W[2],
            W[3],
        ],
        no_setup,
    ),
    (
        "write-refused-group-role",
        &[
            "classify",
            "group",
            "add",
            "g76",
            "roast:bt:bt-110:L1",
            "--role",
            "witness",
            W[0],
            W[1],
            W[2],
            W[3],
        ],
        no_setup,
    ),
    (
        "write-undo-batch",
        &["classify", "undo", "--batch", W[0], W[1], W[2], W[3]],
        no_setup,
    ),
    (
        "write-undo-line-agent",
        &[
            "classify",
            "undo",
            "lab:bt:bt-401:L1",
            W[0],
            W[1],
            W[2],
            W[3],
        ],
        no_setup,
    ),
    (
        "write-refused-undo-human-line",
        &[
            "classify",
            "undo",
            "design:bt:bt-303:L1",
            W[0],
            W[1],
            W[2],
            W[3],
        ],
        no_setup,
    ),
    (
        "write-refused-undo-human-events",
        &["classify", "undo", "--events", "70", W[0], W[1], W[2], W[3]],
        no_setup,
    ),
    (
        "write-refused-undo-ambiguous",
        &[
            "classify", "undo", "--batch", "--events", "62", W[0], W[1], W[2], W[3],
        ],
        no_setup,
    ),
    (
        "write-config-dry-run",
        &[
            "classify",
            "config",
            "add-project",
            "market",
            "Holiday market stall",
            "--alias",
            "market stall",
            "--dry-run",
            W[0],
            W[1],
            W[2],
            W[3],
        ],
        no_setup,
    ),
    (
        "write-refused-config-no-confirm",
        &[
            "classify",
            "config",
            "add-project",
            "market",
            "Holiday market stall",
            W[0],
            W[1],
            W[2],
            W[3],
        ],
        no_setup,
    ),
    (
        "write-config-confirm",
        &[
            "classify",
            "config",
            "add-project",
            "market",
            "Holiday market stall",
            "--confirm",
            W[0],
            W[1],
            W[2],
            W[3],
        ],
        no_setup,
    ),
    (
        "write-refused-config-duplicate",
        &[
            "classify",
            "config",
            "add-project",
            "harvest",
            "Again",
            "--confirm",
            W[0],
            W[1],
            W[2],
            W[3],
        ],
        no_setup,
    ),
    (
        "queue-design-counts",
        &["classify", "queue", "org:design"],
        no_setup,
    ),
    (
        "queue-scope-2025",
        &["classify", "queue", "--limit", "3"],
        scope_2025,
    ),
    (
        "queue-scope-2025-json-lab",
        &["classify", "queue", "org:lab", "--json"],
        scope_2025,
    ),
    (
        "queue-no-store",
        &["classify", "queue", "--limit", "3"],
        no_store,
    ),
    (
        "suggest-dry-run",
        &["classify", "suggest", "--dry-run", "status:all"],
        new_rule,
    ),
    (
        "suggest-dry-run-json",
        &["classify", "suggest", "--dry-run", "--json"],
        no_store,
    ),
    (
        "export-csv-scope",
        &["classify", "export", "--csv", "org:hold"],
        scope_2025,
    ),
    ("rules-check", &["classify", "rules", "check"], no_setup),
    (
        "rules-check-json",
        &["classify", "rules", "check", "--json"],
        no_setup,
    ),
    (
        "rules-check-bad",
        &["classify", "rules", "check"],
        bad_config,
    ),
    (
        "rules-check-jev-on",
        &["classify", "rules", "check"],
        jev_on,
    ),
    ("queue", &["classify", "queue", "--full"], no_setup),
    (
        "queue-roast-json",
        &["classify", "queue", "org:roast", "status:all", "--json"],
        no_setup,
    ),
    (
        "queue-stale",
        &["classify", "queue", "status:stale"],
        no_setup,
    ),
    (
        "queue-agent",
        &["classify", "queue", "by:agent", "status:all"],
        no_setup,
    ),
    (
        "queue-date-limit",
        &[
            "classify",
            "queue",
            "year:2025",
            "conf<0.9",
            "--sort",
            "date",
            "--limit",
            "5",
        ],
        no_setup,
    ),
    (
        "queue-text-search",
        &["classify", "queue", "proofs", "status:all"],
        no_setup,
    ),
    (
        "queue-bad-filter",
        &["classify", "queue", "month:2025-13"],
        no_setup,
    ),
    ("queue-bad-config", &["classify", "queue"], bad_config),
    (
        "suggest-nothing-new",
        &["classify", "suggest", "status:all"],
        no_setup,
    ),
    (
        "suggest-new-rule",
        &["classify", "suggest", "status:all"],
        new_rule,
    ),
    (
        "suggest-new-rule-json",
        &["classify", "suggest", "--json"],
        new_rule,
    ),
    (
        "suggest-jev",
        &["classify", "suggest", "--engine", "jev"],
        no_setup,
    ),
    (
        "suggest-jev-json",
        &["classify", "suggest", "--engine", "jev", "--json"],
        no_setup,
    ),
    ("export-csv", &["classify", "export", "--csv"], no_setup),
    (
        "export-csv-design",
        &["classify", "export", "--csv", "org:design"],
        no_setup,
    ),
    ("export-no-format", &["classify", "export"], no_setup),
    ("help", &["classify", "--help"], no_setup),
];

fn render(argv: &[&str], code: i32, out: &str) -> String {
    format!("$ ownpurse {}\nexit {code}\n{out}", argv.join(" "))
}

fn actual() -> BTreeMap<String, String> {
    let mut m = BTreeMap::new();
    for (name, argv, setup) in CASES {
        let d = copy(name);
        setup(&d);
        let (code, out) = run(&d, argv);
        m.insert(name.to_string(), render(argv, code, &out));
        let _ = std::fs::remove_dir_all(&d);
    }
    m
}

#[test]
fn bless() {
    if std::env::var("OWNPURSE_BLESS").as_deref() != Ok("1") {
        return;
    }
    let _ = std::fs::remove_dir_all(goldens());
    std::fs::create_dir_all(goldens()).unwrap();
    for (name, text) in actual() {
        std::fs::write(goldens().join(format!("{name}.txt")), text).unwrap();
    }
}

#[test]
fn goldens_match() {
    if std::env::var("OWNPURSE_BLESS").as_deref() == Ok("1") {
        return;
    }
    let got = actual();
    let mut names: Vec<String> = std::fs::read_dir(goldens())
        .expect("classify-goldens/ (bless first)")
        .map(|e| {
            e.unwrap()
                .file_name()
                .to_string_lossy()
                .trim_end_matches(".txt")
                .to_string()
        })
        .collect();
    names.sort();
    assert_eq!(
        names,
        got.keys().cloned().collect::<Vec<_>>(),
        "goldens and CASES disagree (re-bless)"
    );
    for (name, text) in &got {
        let want = std::fs::read_to_string(goldens().join(format!("{name}.txt"))).unwrap();
        assert!(
            want == *text,
            "classify golden {name} differs:\n--- want\n{want}\n--- got\n{text}"
        );
    }
}

#[test]
fn jev_never_reaches_the_network() {
    // the refusal happens before the config, record or store are opened: it works with none of them
    let d = std::env::temp_dir().join(format!("xa-classify-cli-nojev-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    std::fs::write(d.join("ownpurse.json"), "{}").unwrap();
    let (code, out) = run(&d, &["classify", "suggest", "--engine", "jev", "--json"]);
    assert_eq!(code, 2);
    assert!(out.contains("\"error\": \"jev_not_enabled\""), "{out}");
    let left: Vec<_> = std::fs::read_dir(&d).unwrap().collect();
    assert_eq!(left.len(), 1, "nothing written");
    let _ = std::fs::remove_dir_all(&d);
}

/// Every file under `dir`: (size, mtime, bytes hash).
fn tree(dir: &Path) -> BTreeMap<String, (u64, std::time::SystemTime, u64)> {
    fn walk(base: &Path, d: &Path, m: &mut BTreeMap<String, (u64, std::time::SystemTime, u64)>) {
        for e in std::fs::read_dir(d).unwrap() {
            let e = e.unwrap();
            let p = e.path();
            if p.is_dir() {
                walk(base, &p, m);
            } else {
                let md = e.metadata().unwrap();
                let bytes = std::fs::read(&p).unwrap();
                use std::hash::{Hash, Hasher};
                let mut h = std::collections::hash_map::DefaultHasher::new();
                bytes.hash(&mut h);
                let rel = p.strip_prefix(base).unwrap().display().to_string();
                m.insert(rel, (md.len(), md.modified().unwrap(), h.finish()));
            }
        }
    }
    let mut m = BTreeMap::new();
    walk(dir, dir, &mut m);
    m
}

/// V14: the classify commands write the classification store and config only. The record is read-only (its
/// bytes never change; SQLite's `-wal`/`-shm` reader files beside it are the record reader's, as for every
/// read command).
#[test]
fn v14_writes_only_the_classification_files() {
    let d = copy("v14");
    new_rule(&d);
    let before = tree(&d);
    for (_, argv, _) in CASES {
        let _ = run(&d, argv);
    }
    let after = tree(&d);
    let mut written: Vec<String> = after
        .iter()
        .filter(|(k, v)| before.get(*k) != Some(v))
        .map(|(k, _)| k.clone())
        .collect();
    written.extend(before.keys().filter(|k| !after.contains_key(*k)).cloned());
    let allowed = |p: &str| {
        p == "db/classify.sqlite"
            || p == "classify.toml"
            || p.starts_with("classify.toml.~")
            || p == "record.sqlite-wal"
            || p == "record.sqlite-shm"
    };
    assert!(
        written.iter().all(|p| allowed(p)),
        "unexpected writes: {written:?}"
    );
    assert!(
        written.contains(&"db/classify.sqlite".to_string()),
        "suggest wrote the store"
    );
    assert_eq!(
        std::fs::read(d.join("record.sqlite")).unwrap(),
        std::fs::read(demo().join("record.sqlite")).unwrap(),
        "the record is never written"
    );
    let _ = std::fs::remove_dir_all(&d);
}

/// A lone read never creates the store (it reads as empty), and a dry run writes nothing.
#[test]
fn reads_and_dry_runs_create_no_store() {
    let d = copy("lazy");
    no_store(&d);
    for argv in [
        &["classify", "queue"][..],
        &["classify", "export", "--csv"],
        &["classify", "suggest", "--dry-run"],
        &["classify", "rules", "check"],
    ] {
        let (code, _) = run(&d, argv);
        assert_eq!(code, 0, "{argv:?}");
        assert!(!d.join("db").exists(), "{argv:?} created the store");
    }
    let (code, _) = run(&d, &["classify", "suggest"]);
    assert_eq!(code, 0);
    assert!(
        d.join("db/classify.sqlite").exists(),
        "the first write creates it"
    );
    let _ = std::fs::remove_dir_all(&d);
}
