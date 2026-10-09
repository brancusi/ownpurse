//! CLI output goldens on the committed CLI fixture (`rust/tests/fixtures/cli`, built by `cargo run -p xtask --
//! demo fixtures` from `rust/tests/fixtures/engine.json`). They are regression goldens written by the CLI itself:
//! `OWNPURSE_BLESS=1 cargo test -p ownpurse-cli --test cli_golden bless` rewrites `goldens/` from `CASES`.
//! For every case the binary must give the same exit code, the same JSON documents (`--json`; key order
//! checked, decimals compared numerically, strings exactly) and byte-identical text output. One key is ignored: `generated_at` in the `ic --json` envelope (the wall-clock minute the
//! report was made). In `tb --compare` rows the per-org `members` / `member_ids` / `names` lists are compared
//! as unordered (see `unorder_members`). The rewrites are the fixture folder path, which both sides print as
//! `<FIXTURES>`, and `usage`'s estimated calls-left cell (see `unmark_estimates`).

use rust_decimal::Decimal;
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::process::Command;

fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/cli")
}

fn config_path() -> PathBuf {
    fixtures().join("ownpurse.json")
}

/// Run the binary against the fixture config; (exit code, stdout with the fixture folder as `<FIXTURES>`).
fn run(args: &[&str]) -> (i32, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_ownpurse"))
        .args(args)
        .env("OWNPURSE_CONFIG", config_path())
        .env_remove("OWNPURSE_PROFILE")
        .env_remove("OWNPURSE_STORE")
        .env_remove("OWNPURSE_CLIENT_ID")
        .env_remove("XDG_CONFIG_HOME")
        .env_remove("XDG_DATA_HOME")
        .env("HOME", "/home/demo")
        .output()
        .expect("run ownpurse");
    let dir = fixtures().display().to_string();
    let text = String::from_utf8(out.stdout)
        .expect("utf-8 stdout")
        .replace(&dir, "<FIXTURES>");
    (out.status.code().unwrap_or(-1), text)
}

/// The documented wording differences. Hints for commands not available yet say they are coming instead of
/// naming them: the goldens' `` — `ownpurse report X TB --date D` caches it (1 call per org).`` and their
/// `ownpurse init` hints are rewritten to the current texts before comparing.
fn network_hint_str(s: &str) -> String {
    let mut out = s.to_string();
    // the ic hint: " — `ownpurse report … TB --date …` caches it (1 call per org)."
    while let Some(i) = out.find(" — `ownpurse report ") {
        let tail = "` caches it (1 call per org).";
        let Some(j) = out[i..].find(tail) else { break };
        out.replace_range(
            i..i + j + tail.len(),
            " — fetching it needs `ownpurse report` (coming in v0.9.x), 1 call per org.",
        );
    }
    let create = ownpurse_core::config::CREATE_CONFIG;
    out = out.replace("Run `ownpurse init --client-id <id>` first.", create);
    out = out.replace(
        "Run `ownpurse init --client-id <id>` to create a config.",
        create,
    );
    out = out.replace(
        "New setup: ownpurse init --client-id <id>",
        &format!("New setup: {create}"),
    );
    out
}

fn network_hint(v: &Value) -> Value {
    match v {
        Value::String(s) => Value::String(network_hint_str(s)),
        Value::Array(a) => Value::Array(a.iter().map(network_hint).collect()),
        Value::Object(m) => Value::Object(
            m.iter()
                .map(|(k, v)| (k.clone(), network_hint(v)))
                .collect(),
        ),
        other => other.clone(),
    }
}

fn docs(text: &str) -> Vec<Value> {
    serde_json::Deserializer::from_str(text)
        .into_iter::<Value>()
        .collect::<Result<_, _>>()
        .unwrap_or_else(|e| panic!("not JSON ({e}):\n{text}"))
}

fn as_dec(v: &Value) -> Option<Decimal> {
    let t = match v {
        Value::String(s) => s.clone(),
        Value::Number(n) => n.to_string(),
        _ => return None,
    };
    Decimal::from_str_exact(&t)
        .or_else(|_| Decimal::from_scientific(&t))
        .ok()
}

/// Structural equality: same key order, strings exact unless both are decimals (then numeric equality).
fn same(path: &str, a: &Value, b: &Value, errors: &mut Vec<String>) {
    match (a, b) {
        (Value::Object(x), Value::Object(y)) => {
            let kx: Vec<&String> = x.keys().collect();
            let ky: Vec<&String> = y.keys().collect();
            if kx != ky {
                errors.push(format!("{path}: keys got {kx:?} != golden {ky:?}"));
                return;
            }
            for (k, v) in x {
                same(&format!("{path}.{k}"), v, &y[k], errors);
            }
        }
        (Value::Array(x), Value::Array(y)) => {
            if x.len() != y.len() {
                errors.push(format!(
                    "{path}: length got {} != golden {}",
                    x.len(),
                    y.len()
                ));
                return;
            }
            for (i, (p, q)) in x.iter().zip(y).enumerate() {
                same(&format!("{path}[{i}]"), p, q, errors);
            }
        }
        (Value::String(p), Value::String(q)) if p == q => {}
        (Value::Number(_), Value::Number(_)) | (Value::String(_), Value::String(_)) => {
            match (as_dec(a), as_dec(b)) {
                (Some(p), Some(q)) if p == q => {}
                _ => errors.push(format!("{path}: got {a} != golden {b}")),
            }
        }
        _ if a == b => {}
        _ => errors.push(format!("{path}: got {a} != golden {b}")),
    }
}

/// `tb --compare` rows list each org's member accounts in the order the org's trial-balance rows were fed to
/// the engine. The goldens were made with a feed order that starts with a *set* of differing accounts (hash
/// order, randomised per process), not stable even run to run; the engine now feeds differing accounts then
/// balances, both by account id. Membership is what matters, so these per-org lists are compared as sorted multisets.
fn unorder_members(doc: &mut Value) {
    let Some(rows) = doc.get_mut("rows").and_then(Value::as_array_mut) else {
        return;
    };
    for r in rows {
        for k in ["members", "member_ids", "names"] {
            if let Some(per_org) = r.get_mut(k).and_then(Value::as_object_mut) {
                for list in per_org.values_mut() {
                    if let Some(a) = list.as_array_mut() {
                        a.sort_by_key(|v| v.to_string());
                    }
                }
            }
        }
    }
}

/// The documented `usage` difference: a calls-left figure that is the local
/// fallback is marked as an estimate. Only that cell is rewritten back to the golden's bare number before
/// comparing: `--json` drops `day_remaining_estimated` from quota rows, text turns `≈ N (estimated)` into `N`.
fn unmark_estimates(docs: &mut [Value]) {
    for d in docs {
        for row in d["quota"].as_array_mut().into_iter().flatten() {
            if let Some(m) = row.as_object_mut() {
                m.shift_remove("day_remaining_estimated");
            }
        }
    }
}

fn unmark_estimates_text(text: &str) -> String {
    let mut out = String::new();
    let mut rest = text;
    while let Some(i) = rest.find("≈ ") {
        out.push_str(&rest[..i]);
        let tail = &rest[i + "≈ ".len()..];
        match tail.find(" (estimated)") {
            Some(j) if tail[..j].bytes().all(|b| b.is_ascii_digit() || b == b'-') => {
                out.push_str(&tail[..j]);
                rest = &tail[j + " (estimated)".len()..];
            }
            _ => {
                out.push_str("≈ ");
                rest = tail;
            }
        }
    }
    out.push_str(rest);
    out
}

const T: &str = "--date";

/// Every golden case: name → argv (after `--offline`). `bless` writes one golden per case.
const CASES: &[(&str, &[&str])] = &[
    // accounts
    ("accounts-one", &["accounts", "one"]),
    ("accounts-all", &["accounts", "all"]),
    ("accounts-one-bank", &["accounts", "one", "--bank"]),
    (
        "accounts-one-search",
        &["accounts", "one", "--search", "ACC"],
    ),
    (
        "accounts-one-type",
        &["accounts", "one", "--type", "expense", "--archived"],
    ),
    ("accounts-one-full", &["accounts", "one", "--full"]),
    ("accounts-group-pair", &["accounts", "group:pair"]),
    ("accounts-limit", &["accounts", "all", "--limit", "3"]),
    // org
    ("org-all", &["org", "all"]),
    ("org-one", &["org", "one"]),
    // contacts
    ("contacts-one", &["contacts", "one"]),
    ("contacts-one-archived", &["contacts", "one", "--archived"]),
    ("contacts-one-search", &["contacts", "one", "zo"]),
    ("contacts-hq-never-synced", &["contacts", "hq"]),
    // txns
    ("txns-one", &["txns", "one"]),
    (
        "txns-one-range",
        &["txns", "one", "--from", "2025-01-01", "--to", "2025-01-31"],
    ),
    ("txns-one-search", &["txns", "one", "--search", "café"]),
    (
        "txns-one-deleted",
        &[
            "txns",
            "one",
            "--deleted",
            "--from",
            "2025-03-01",
            "--to",
            "2025-03-31",
        ],
    ),
    ("txns-one-status", &["txns", "one", "--status", "deleted"]),
    ("txns-one-account", &["txns", "one", "--account", "savings"]),
    (
        "txns-all",
        &["txns", "all", "--from", "2024-01-01", "--to", "2024-12-31"],
    ),
    ("txns-pair", &["txns", "hq,co"]),
    ("txns-one-limit", &["txns", "one", "--limit", "5"]),
    // journals / transfers
    ("journals-one", &["journals", "one"]),
    ("journals-all", &["journals", "all"]),
    (
        "journals-one-status",
        &["journals", "one", "--status", "posted"],
    ),
    (
        "journals-one-search",
        &["journals", "one", "--search", "ACCRUAL"],
    ),
    ("transfers-one", &["transfers", "one"]),
    ("transfers-all", &["transfers", "all"]),
    // list
    ("list-invoices", &["list", "one", "Invoices"]),
    (
        "list-invoices-paid",
        &["list", "one", "invoices", "--status", "paid"],
    ),
    ("list-payments", &["list", "one", "Payments"]),
    ("list-creditnotes", &["list", "one", "CreditNotes"]),
    ("list-accounts", &["list", "one", "Accounts"]),
    ("list-organisation", &["list", "all", "Organisation"]),
    (
        "list-banktxns-range",
        &[
            "list",
            "one",
            "BankTransactions",
            "--from",
            "2025-06-01",
            "--to",
            "2025-06-01",
        ],
    ),
    (
        "list-banktxns-fields",
        &[
            "list",
            "one",
            "BankTransactions",
            "--fields",
            "Contact.Name,Total,LineItems.0,Reference",
            "--search",
            "zoë",
        ],
    ),
    ("list-journals-pair", &["list", "hq,co", "ManualJournals"]),
    ("list-transfers", &["list", "one", "BankTransfers"]),
    ("list-unknown-entity", &["list", "one", "Bogus"]),
    ("list-contacts-never-synced", &["list", "co", "Contacts"]),
    // show
    ("show-txn", &["show", "one", "BankTransactions", "zz-rcv"]),
    ("show-account", &["show", "one", "accounts", "card"]),
    (
        "show-ambiguous",
        &["show", "one", "BankTransactions", "bt-1"],
    ),
    ("show-missing", &["show", "one", "BankTransactions", "nope"]),
    ("show-multi-org", &["show", "all", "Accounts", "x"]),
    // open (local; the fixture orgs have no ShortCode: no link, never a guessed one)
    (
        "open-no-short-code",
        &["open", "one", "BankTransactions", "zz-rcv"],
    ),
    // gl
    ("gl-one", &["gl", "one"]),
    (
        "gl-one-account",
        &["gl", "one", "--account", "400", "--to", "2025-12-31"],
    ),
    (
        "gl-one-account-range",
        &[
            "gl",
            "one",
            "--account",
            "Checking",
            "--from",
            "2025-03-01",
            "--to",
            "2025-04-30",
        ],
    ),
    (
        "gl-one-account-history",
        &[
            "gl",
            "one",
            "--account",
            "supplies",
            "--full-history",
            "--to",
            "2025-06-30",
        ],
    ),
    (
        "gl-one-substring",
        &["gl", "one", "--account", "ount", "--to", "2025-12-31"],
    ),
    (
        "gl-one-source",
        &["gl", "one", "--source", "manualjournals"],
    ),
    (
        "gl-one-search",
        &["gl", "one", "--search", "joinery", "--to", "2026-12-31"],
    ),
    ("gl-pair", &["gl", "hq,co"]),
    ("gl-pair-account", &["gl", "hq,co", "--account", "2475"]),
    (
        "gl-hq-account",
        &["gl", "hq", "--account", "1405", "--to", "2025-12-31"],
    ),
    (
        "gl-no-match",
        &["gl", "one", "--account", "nothing-like-this"],
    ),
    // tb
    ("tb-one-2025", &["tb", "one", T, "2025-12-31"]),
    ("tb-one-2025-06", &["tb", "one", T, "2025-06-30"]),
    ("tb-one-2024", &["tb", "one", T, "2024-12-31"]),
    (
        "tb-one-history",
        &["tb", "one", T, "2025-12-31", "--full-history"],
    ),
    (
        "tb-one-since",
        &["tb", "one", T, "2025-12-31", "--since", "2024-12-31"],
    ),
    ("tb-one-2026", &["tb", "one", T, "2026-12-31", "--full"]),
    ("tb-all", &["tb", "all", T, "2025-12-31"]),
    ("tb-hq", &["tb", "hq", T, "2025-12-31"]),
    (
        "tb-pair-history",
        &[
            "tb",
            "group:pair",
            T,
            "2025-06-30",
            "--full-history",
            "--full",
        ],
    ),
    (
        "tb-since-uncached",
        &["tb", "one", T, "2025-12-31", "--since", "2023-12-31"],
    ),
    ("tb-unknown-group", &["tb", "group:nope", T, "2025-12-31"]),
    ("tb-unknown-org", &["tb", "nobody", T, "2025-12-31"]),
    // parity
    (
        "parity-one",
        &["parity", "one", T, "2025-12-31", "--cached"],
    ),
    (
        "parity-one-since",
        &["parity", "one", T, "2025-12-31", "--since", "2024-12-31"],
    ),
    (
        "parity-one-2024",
        &["parity", "one", T, "2024-12-31", "--cached"],
    ),
    (
        "parity-all",
        &["parity", "all", T, "2025-12-31", "--cached"],
    ),
    (
        "parity-pair-since",
        &[
            "parity",
            "group:pair",
            T,
            "2025-12-31",
            "--since",
            "2024-12-31",
            "--cached",
        ],
    ),
    (
        "parity-uncached",
        &["parity", "hq", T, "2025-06-30", "--cached"],
    ),
    (
        "parity-strict",
        &["parity", "one", T, "2025-12-31", "--cached", "--strict"],
    ),
    (
        "parity-strict-ok",
        &["parity", "hq", T, "2025-12-31", "--cached", "--strict"],
    ),
    // usage / config
    ("usage", &["usage", "--since", "2026-01-01"]),
    (
        "usage-wide",
        &["usage", "--since", "2025-01-01", "--calls", "2"],
    ),
    ("usage-full", &["usage", "--since", "2025-01-01", "--full"]),
    ("config", &["config"]),
    ("config-full", &["config", "--full"]),
    ("config-path", &["config", "path"]),
    ("config-links", &["config", "links"]),
    ("config-links-full", &["config", "links", "--full"]),
    ("profile-other", &["--profile", "other", "config", "path"]),
    // intercompany
    ("ic-all", &["ic", "--as-of", "2025-12-31"]),
    ("ic-pair", &["ic", "hq-co", "--as-of", "2025-12-31"]),
    (
        "ic-pair-both",
        &["ic", "hq-co-both", "--as-of", "2025-12-31"],
    ),
    ("ic-pair-one", &["ic", "one-co", "--as-of", "2025-12-31"]),
    ("ic-not-compared", &["ic", "--as-of", "2025-06-30"]),
    (
        "ic-pair-not-compared",
        &["ic", "hq-co", "--as-of", "2024-06-30"],
    ),
    ("ic-unknown-pair", &["ic", "nope", "--as-of", "2025-12-31"]),
    ("ic-no-pairs", &["--profile", "other", "ic"]),
    // tb --compare
    (
        "compare-code",
        &["tb", "hq,co", "--compare", T, "2025-12-31"],
    ),
    (
        "compare-map",
        &[
            "tb",
            "hq,co",
            "--compare",
            "--align",
            "map",
            T,
            "2025-12-31",
        ],
    ),
    (
        "compare-three-map",
        &[
            "tb",
            "one,hq,co",
            "--compare",
            "--align",
            "map",
            T,
            "2025-12-31",
        ],
    ),
    (
        "compare-three-code-history",
        &[
            "tb",
            "one,hq,co",
            "--compare",
            T,
            "2025-06-30",
            "--full-history",
        ],
    ),
    (
        "compare-group",
        &[
            "tb",
            "group:pair",
            "--compare",
            "--align",
            "code",
            T,
            "2024-12-31",
            "--limit",
            "3",
        ],
    ),
    (
        "compare-refuse-currency",
        &[
            "--profile",
            "wide",
            "tb",
            "hq,eur",
            "--compare",
            T,
            "2025-12-31",
        ],
    ),
    (
        "compare-refuse-fy",
        &[
            "--profile",
            "wide",
            "tb",
            "hq,june",
            "--compare",
            T,
            "2025-12-31",
        ],
    ),
    (
        "compare-too-few",
        &["tb", "one", "--compare", T, "2025-12-31"],
    ),
    (
        "compare-too-many",
        &[
            "--profile",
            "wide",
            "tb",
            "all",
            "--compare",
            T,
            "2025-12-31",
        ],
    ),
    ("profile-unknown", &["--profile", "nope", "accounts", "one"]),
];

/// `OWNPURSE_BLESS=1 cargo test -p ownpurse-cli --test cli_golden bless`: rewrite `goldens/` from `CASES` with
/// this build's output (the ic envelope's wall-clock `generated_at` dropped, usage estimates unmarked).
#[test]
fn bless() {
    if std::env::var("OWNPURSE_BLESS").as_deref() != Ok("1") {
        return;
    }
    let dir = fixtures().join("goldens");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    for (name, argv) in CASES {
        let mut full: Vec<&str> = vec!["--offline"];
        full.extend(argv.iter().copied());
        let mut jargs = vec!["--json"];
        jargs.extend(full.iter().copied());
        let (code, jtext) = run(&jargs);
        let (tcode, mut ttext) = run(&full);
        assert_eq!(
            code, tcode,
            "{name}: exit codes differ between --json and text"
        );
        let mut got = docs(&jtext);
        for d in got.iter_mut() {
            if let Some(m) = d.as_object_mut() {
                m.shift_remove("generated_at");
            }
        }
        if name.starts_with("usage") {
            unmark_estimates(&mut got);
            ttext = unmark_estimates_text(&ttext);
        }
        let g = serde_json::json!({"argv": full, "exit": code, "json": got, "text": ttext});
        let mut buf = Vec::new();
        let fmt = serde_json::ser::PrettyFormatter::with_indent(b" ");
        let mut ser = serde_json::Serializer::with_formatter(&mut buf, fmt);
        serde::Serialize::serialize(&g, &mut ser).unwrap();
        buf.push(b'\n');
        std::fs::write(dir.join(format!("{name}.json")), buf).unwrap();
    }
}

fn goldens() -> Vec<(String, Value)> {
    let mut out: Vec<(String, Value)> = std::fs::read_dir(fixtures().join("goldens"))
        .expect("goldens folder")
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().is_some_and(|x| x == "json"))
        .map(|p| {
            let g: Value = serde_json::from_str(&std::fs::read_to_string(&p).unwrap()).unwrap();
            (p.file_stem().unwrap().to_string_lossy().to_string(), g)
        })
        .collect();
    out.sort_by(|a, b| a.0.cmp(&b.0));
    out
}

fn http_calls() -> i64 {
    let rec = ownpurse_core::Record::open(fixtures().join("record.sqlite")).unwrap();
    rec.http_calls().unwrap()
}

fn record_bytes() -> Vec<u8> {
    std::fs::read(fixtures().join("record.sqlite")).unwrap()
}

#[test]
fn every_command_matches_the_goldens_json_and_text() {
    let calls_before = http_calls();
    let bytes_before = record_bytes();
    let cases = goldens();
    assert!(cases.len() >= 100, "only {} goldens", cases.len());
    let names: Vec<&str> = cases.iter().map(|(n, _)| n.as_str()).collect();
    let mut want: Vec<&str> = CASES.iter().map(|(n, _)| *n).collect();
    want.sort();
    assert_eq!(names, want, "goldens/ and CASES disagree (re-bless)");
    let mut failures: Vec<String> = Vec::new();
    for (name, g) in &cases {
        let argv: Vec<String> = g["argv"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap().to_string())
            .collect();
        let exit = g["exit"].as_i64().unwrap() as i32;
        // JSON
        let mut jargs: Vec<&str> = vec!["--json"];
        jargs.extend(argv.iter().map(String::as_str));
        let (code, text) = run(&jargs);
        if code != exit {
            failures.push(format!("{name}: exit got {code} != golden {exit}\n{text}"));
            continue;
        }
        let mut errors = Vec::new();
        let mut got = docs(&text);
        let mut want_json = network_hint(&g["json"]);
        if name.starts_with("compare-") {
            for d in got
                .iter_mut()
                .chain(want_json.as_array_mut().into_iter().flatten())
            {
                unorder_members(d);
            }
        }
        if name.starts_with("ic-") {
            for d in got
                .iter_mut()
                .chain(want_json.as_array_mut().into_iter().flatten())
            {
                if let Some(m) = d.as_object_mut() {
                    m.shift_remove("generated_at");
                }
            }
        }
        if name.starts_with("usage") {
            unmark_estimates(&mut got);
        }
        same("$", &Value::Array(got), &want_json, &mut errors);
        for e in errors.iter().take(5) {
            failures.push(format!("{name} (json): {e}"));
        }
        // Text
        let targs: Vec<&str> = argv.iter().map(String::as_str).collect();
        let (tcode, ttext) = run(&targs);
        for bad in [
            ["ownpurse", "-py"].concat(),
            ["Py", "thon"].concat(),
            "for now".to_string(),
            ["Not in the ", "Rust CLI"].concat(),
        ] {
            if ttext.contains(bad.as_str()) {
                failures.push(format!(
                    "{name}: output mentions {bad:?} (a retired tool, or a promise)"
                ));
            }
        }
        let ttext = if name.starts_with("usage") {
            unmark_estimates_text(&ttext)
        } else {
            ttext
        };
        let want_owned = network_hint_str(g["text"].as_str().unwrap());
        let want = want_owned.as_str();
        if tcode != exit {
            failures.push(format!("{name} (text): exit got {tcode} != golden {exit}"));
        } else if ttext != want {
            let line = ttext
                .lines()
                .zip(want.lines())
                .position(|(a, b)| a != b)
                .unwrap_or_else(|| ttext.lines().count().min(want.lines().count()));
            failures.push(format!(
                "{name} (text): first difference at line {line}:\n  got:    {:?}\n  golden: {:?}",
                ttext.lines().nth(line),
                want.lines().nth(line)
            ));
        }
    }
    assert!(
        failures.is_empty(),
        "{} of {} cases differ:\n{}",
        failures.len(),
        cases.len(),
        failures.join("\n")
    );
    assert_eq!(http_calls(), calls_before, "no command may touch http_log");
    assert!(
        record_bytes() == bytes_before,
        "the record file must be byte-identical after every command"
    );
    eprintln!("{} golden cases equal (json + text)", cases.len());
}

#[test]
fn network_and_unported_commands_refuse_with_exit_2() {
    let calls_before = http_calls();
    for args in [
        vec!["init", "--client-id", "x"],
        vec!["q", "select 1"],
        vec!["verify"],
        vec!["skills"],
        vec![],
        vec!["accounts", "one", "--fresh"],
    ] {
        let mut jargs = vec!["--json"];
        jargs.extend(args.iter().copied());
        let (code, text) = run(&jargs);
        assert_eq!(code, 2, "{args:?}: {text}");
        let d = &docs(&text)[0];
        assert_eq!(d["error"], "not_available", "{args:?}");
        let msg = d["message"].as_str().unwrap();
        assert!(
            msg.starts_with("`ownpurse ")
                && msg.ends_with("` is not yet available in Rust; coming in v0.9.x."),
            "{msg}"
        );
        // text mode has the same shape
        let (code, text) = run(&args);
        assert_eq!(code, 2);
        assert!(
            text.starts_with("error: not_available\nmessage: `ownpurse "),
            "{text}"
        );
    }
    // `tui` is no longer refused: it launches the separate TUI binary (absent here → a usage error naming it).
    let missing = Command::new(env!("CARGO_BIN_EXE_ownpurse"))
        .args(["tui", "--keys", "json"])
        .env("PATH", "/nonexistent")
        .output()
        .unwrap();
    let text = String::from_utf8_lossy(&missing.stdout).to_string()
        + &String::from_utf8_lossy(&missing.stderr);
    assert!(
        text.contains("ownpurse-tui") || missing.status.success(),
        "tui should launch ownpurse-tui or say it is missing: {text}"
    );
    assert_eq!(http_calls(), calls_before);
}

#[test]
fn argument_errors_are_structured_usage_errors() {
    for args in [
        vec!["tb", "one"],
        vec!["accounts"],
        vec!["tb", "one", "--date", "2025-12-31", "--align", "zzz"],
        vec!["bogus-command"],
    ] {
        let mut jargs = vec!["--json"];
        jargs.extend(args.iter().copied());
        let (code, text) = run(&jargs);
        assert_eq!(code, 2, "{args:?}: {text}");
        assert_eq!(docs(&text)[0]["error"], "usage", "{args:?}");
    }
}

#[test]
fn single_org_commands_refuse_multi_org_scopes() {
    let (code, text) = run(&["--json", "show", "group:pair", "Accounts", "x"]);
    assert_eq!(code, 2);
    let d = &docs(&text)[0];
    assert_eq!(
        d["message"],
        "'group:pair' names 2 orgs; this command works on one."
    );
    assert_eq!(d["help"][0], "Pick one of: hq, co");
}

#[test]
fn missing_config_lists_where_it_looked() {
    let dir = std::env::temp_dir().join(format!("ownpurse-rs-noconfig-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_ownpurse"))
        .args(["--json", "accounts", "one"])
        .current_dir(&dir)
        .env_remove("OWNPURSE_CONFIG")
        .env_remove("OWNPURSE_CLIENT_ID")
        .env("HOME", &dir)
        .env("XDG_CONFIG_HOME", dir.join("cfg"))
        .output()
        .unwrap();
    let _ = std::fs::remove_dir_all(&dir);
    assert_eq!(out.status.code(), Some(1));
    let d = &docs(&String::from_utf8(out.stdout).unwrap())[0];
    assert_eq!(d["error"], "config");
    assert!(
        d["message"]
            .as_str()
            .unwrap()
            .starts_with("No ownpurse config found")
    );
    assert_eq!(d["looked_in"].as_array().unwrap().len(), 3);
}
