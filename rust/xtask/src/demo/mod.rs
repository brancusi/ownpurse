//! The demo fixture generator: every committed demo and engine fixture is built here, in Rust, from two declarative
//! specs committed next to the fixtures:
//!
//! * `rust/tests/fixtures/engine.json`: the engine fixture (`record.sqlite`, org `one` = the ledger fixture,
//!   `hq`/`co` = the intercompany fixture) and the CLI fixture built on it (`cli/record.sqlite`,
//!   `cli/ownpurse.json`);
//! * `rust/tests/fixtures/demo/world.json`: the demo world (four invented orgs) and its state variants.
//!
//! Records go through the production `Writer` (append-only tables, SHA-256 chain) with the clock pinned, then
//! switch to rollback-journal mode and `VACUUM`, so the same spec always gives byte-identical files (the gate
//! test `tests/demo_regen.rs` regenerates into a temp folder and compares). The derived goldens (ledger, parity,
//! intercompany, compare, estimate) are computed by the Rust core on the generated records: they are
//! regression goldens (see `rust/tests/fixtures/demo/README.md`).
//!
//! Zero network: nothing here reads a token, a config outside the specs, or any record but its own output.

use ownpurse_core::canon::canonical;
use ownpurse_core::compare::{self as cmp, RowStyle};
use ownpurse_core::intercompany::{self as ic, IcContext};
use ownpurse_core::{Basis, BasisKind, Chart, Ledger, Mode, Record, resolve, trial_balance};
use ownpurse_xero::store::{Writer, pin_clock};
use rusqlite::types::Value as Sql;
use rust_decimal::Decimal;
use serde_json::{Map, Value, json};
use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};
use std::str::FromStr;

pub mod classify;
pub mod states;

pub type Result<T> = std::result::Result<T, String>;

fn err<E: std::fmt::Display>(ctx: &str) -> impl Fn(E) -> String + '_ {
    move |e| format!("{ctx}: {e}")
}

/// The committed fixtures folder (`rust/tests/fixtures`).
pub fn fixtures_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../tests/fixtures")
}

/// Load a spec file as JSON (numbers keep their text: `arbitrary_precision`).
pub fn load_spec(path: &Path) -> Result<Value> {
    let text = std::fs::read_to_string(path).map_err(err(&path.display().to_string()))?;
    serde_json::from_str(&text).map_err(err(&path.display().to_string()))
}

/// A money value as a JSON number with exactly this text (`"4618.27"` → `4618.27`).
fn num(text: &str) -> Result<Value> {
    serde_json::from_str::<Value>(text)
        .ok()
        .filter(Value::is_number)
        .ok_or_else(|| format!("not an amount: {text:?}"))
}

fn dec(text: &str) -> Result<Decimal> {
    Decimal::from_str(text).map_err(|e| format!("not an amount: {text:?} ({e})"))
}

fn s(v: &Value) -> Result<&str> {
    v.as_str()
        .ok_or_else(|| format!("expected a string, got {v}"))
}

fn arr(v: &Value) -> Result<&Vec<Value>> {
    v.as_array()
        .ok_or_else(|| format!("expected an array, got {v}"))
}

fn obj(v: &Value) -> Result<&Map<String, Value>> {
    v.as_object()
        .ok_or_else(|| format!("expected an object, got {v}"))
}

/// JSON as the fixtures store it: one-space indent, UTF-8 as is, trailing newline.
pub fn pretty(v: &Value) -> String {
    let mut buf = Vec::new();
    let fmt = serde_json::ser::PrettyFormatter::with_indent(b" ");
    let mut ser = serde_json::Serializer::with_formatter(&mut buf, fmt);
    serde::Serialize::serialize(v, &mut ser).expect("json");
    String::from_utf8(buf).expect("utf-8") + "\n"
}

/// Where the generator writes: a root folder and the relative paths written, in order.
pub struct Out {
    pub root: PathBuf,
    pub files: Vec<String>,
}

impl Out {
    pub fn new(root: &Path) -> Out {
        Out {
            root: root.to_path_buf(),
            files: vec![],
        }
    }

    pub fn path(&mut self, rel: &str) -> Result<PathBuf> {
        let p = self.root.join(rel);
        if let Some(d) = p.parent() {
            std::fs::create_dir_all(d).map_err(err(&d.display().to_string()))?;
        }
        self.files.push(rel.to_string());
        Ok(p)
    }

    pub fn json(&mut self, rel: &str, v: &Value) -> Result<()> {
        let p = self.path(rel)?;
        std::fs::write(&p, pretty(v)).map_err(err(&p.display().to_string()))
    }
}

// ── document expansion (compact spec rows → Xero-shaped payloads) ────────────────────────────────────────

/// `[id, code, name, type, class, system?, {extra}?]` → an Account.
fn account(row: &[Value]) -> Result<Value> {
    let mut m = Map::new();
    m.insert("AccountID".into(), row[0].clone());
    m.insert("Code".into(), row[1].clone());
    m.insert("Name".into(), row[2].clone());
    m.insert("Type".into(), row[3].clone());
    m.insert("Class".into(), row[4].clone());
    m.insert("Status".into(), json!("ACTIVE"));
    m.insert(
        "SystemAccount".into(),
        row.get(5).cloned().unwrap_or(Value::Null),
    );
    if let Some(extra) = row.get(6) {
        for (k, v) in obj(extra)? {
            m.insert(k.clone(), v.clone());
        }
    }
    Ok(Value::Object(m))
}

/// `[n, type, date, bank, code, amount, contact, description?]` → a one-line, reconciled, no-tax
/// BankTransaction `bt-<n>`.
fn bank_txn(row: &[Value], amounts: &mut Vec<String>) -> Result<Value> {
    let amount = s(&row[5])?;
    amounts.push(amount.to_string());
    let mut line = Map::new();
    line.insert("AccountCode".into(), row[4].clone());
    line.insert("LineAmount".into(), num(amount)?);
    line.insert("TaxAmount".into(), json!(0));
    if let Some(d) = row.get(7).filter(|d| !d.is_null()) {
        line.insert("Description".into(), d.clone());
    }
    Ok(json!({
        "BankTransactionID": format!("bt-{}", s(&row[0])?),
        "Type": row[1],
        "Status": "AUTHORISED",
        "LineAmountTypes": "NoTax",
        "DateString": format!("{}T00:00:00", s(&row[2])?),
        "BankAccount": {"AccountID": row[3]},
        "Contact": {"Name": row[6]},
        "Total": num(amount)?,
        "IsReconciled": true,
        "LineItems": [Value::Object(line)],
    }))
}

/// `[n, date, narration, [[code, amount], …]]` → a posted ManualJournal `mj-<n>`.
fn journal(row: &[Value], amounts: &mut Vec<String>) -> Result<Value> {
    let mut lines = vec![];
    for l in arr(&row[3])? {
        let l = arr(l)?;
        let amount = s(&l[1])?;
        amounts.push(amount.to_string());
        lines.push(json!({"AccountCode": l[0], "LineAmount": num(amount)?, "TaxAmount": 0}));
    }
    Ok(json!({
        "ManualJournalID": format!("mj-{}", s(&row[0])?),
        "Status": "POSTED",
        "DateString": format!("{}T00:00:00", s(&row[1])?),
        "Narration": row[2],
        "JournalLines": lines,
    }))
}

const MONEY_KEYS: [&str; 4] = ["LineAmount", "TaxAmount", "Total", "Amount"];

/// Every non-zero money value inside a raw payload (for the irregular-cents rule).
fn raw_amounts(v: &Value, out: &mut Vec<String>) {
    match v {
        Value::Object(m) => {
            for (k, x) in m {
                if MONEY_KEYS.contains(&k.as_str()) && x.is_number() {
                    let t = x.to_string();
                    if dec(&t).is_ok_and(|d| !d.is_zero()) {
                        out.push(t);
                    }
                } else {
                    raw_amounts(x, out);
                }
            }
        }
        Value::Array(a) => a.iter().for_each(|x| raw_amounts(x, out)),
        _ => {}
    }
}

fn substitute(v: &Value, i: usize) -> Value {
    match v {
        Value::String(t) => Value::String(t.replace("{i}", &i.to_string())),
        Value::Array(a) => Value::Array(a.iter().map(|x| substitute(x, i)).collect()),
        Value::Object(m) => Value::Object(
            m.iter()
                .map(|(k, x)| (k.clone(), substitute(x, i)))
                .collect(),
        ),
        other => other.clone(),
    }
}

/// One entity's spec items: raw payloads, compact rows (Accounts, BankTransactions, ManualJournals) or
/// `{"repeat": n, "from": k, "template": {…}}` (`{i}` in strings runs k, k+1, …).
fn expand(entity: &str, items: &[Value], amounts: &mut Vec<String>) -> Result<Vec<Value>> {
    let mut out = vec![];
    for it in items {
        match it {
            Value::Array(row) => out.push(match entity {
                "Accounts" => account(row)?,
                "BankTransactions" => bank_txn(row, amounts)?,
                "ManualJournals" => journal(row, amounts)?,
                _ => return Err(format!("{entity}: no compact form")),
            }),
            Value::Object(m) if m.contains_key("repeat") => {
                let n = m["repeat"].as_u64().ok_or("repeat: a count")? as usize;
                let from = m["from"].as_u64().ok_or("repeat: from")? as usize;
                for i in from..from + n {
                    let doc = substitute(&m["template"], i);
                    raw_amounts(&doc, amounts);
                    out.push(doc);
                }
            }
            Value::Object(_) => {
                raw_amounts(it, amounts);
                out.push(it.clone());
            }
            other => return Err(format!("{entity}: unexpected item {other}")),
        }
    }
    Ok(out)
}

fn id_key(entity: &str) -> Result<&'static str> {
    Ok(match entity {
        "Accounts" => "AccountID",
        "Organisation" => "OrganisationID",
        "BankTransactions" => "BankTransactionID",
        "BankTransfers" => "BankTransferID",
        "ManualJournals" => "ManualJournalID",
        "Invoices" => "InvoiceID",
        "CreditNotes" => "CreditNoteID",
        "Payments" => "PaymentID",
        "Contacts" => "ContactID",
        _ => return Err(format!("unknown entity {entity}")),
    })
}

// ── record writing ─────────────────────────────────────────────────────────────────────────────────────

fn remove_record(path: &Path) {
    for suffix in ["", "-wal", "-shm", "-journal"] {
        let _ = std::fs::remove_file(format!("{}{suffix}", path.display()));
    }
}

struct Rec {
    w: Writer,
    path: PathBuf,
    known: HashMap<(String, String), HashMap<String, (i64, String)>>,
}

fn t(v: &str) -> Sql {
    Sql::Text(v.into())
}

fn opt_t(v: Option<&str>) -> Sql {
    v.map_or(Sql::Null, t)
}

fn opt_i(v: Option<i64>) -> Sql {
    v.map_or(Sql::Null, Sql::Integer)
}

impl Rec {
    fn create(path: &Path) -> Result<Rec> {
        remove_record(path);
        let w = Writer::open(path).map_err(err("open record"))?;
        Ok(Rec {
            w,
            path: path.to_path_buf(),
            known: HashMap::new(),
        })
    }

    fn version(&mut self, tenant: &str, entity: &str, doc: &Value) -> Result<()> {
        let key = id_key(entity)?;
        let xid = s(&doc[key]).map_err(|e| format!("{entity}.{key}: {e}"))?;
        let status = doc
            .get("Status")
            .or_else(|| doc.get("ContactStatus"))
            .and_then(Value::as_str);
        let known = self
            .known
            .entry((tenant.to_string(), entity.to_string()))
            .or_default();
        self.w
            .record_version(tenant, entity, xid, doc, status, None, "sync", None, known)
            .map_err(err("record_version"))?;
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn sync_run(
        &self,
        id: &str,
        tenant: &str,
        entity: &str,
        mode: &str,
        at: &str,
        calls: i64,
        fetched: i64,
        new_versions: i64,
    ) -> Result<()> {
        self.w
            .append(
                "sync_runs",
                &[
                    ("id", t(id)),
                    ("tenant_id", t(tenant)),
                    ("entity", t(entity)),
                    ("mode", t(mode)),
                    ("started_at", t(at)),
                    ("finished_at", t(at)),
                    ("outcome", t("ok")),
                    ("calls", Sql::Integer(calls)),
                    ("fetched", Sql::Integer(fetched)),
                    ("new_versions", Sql::Integer(new_versions)),
                    ("high_water", Sql::Null),
                    ("detail", Sql::Null),
                ],
            )
            .map_err(err("sync_runs"))?;
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn call(
        &self,
        at: &str,
        tenant: Option<&str>,
        method: &str,
        path: &str,
        status: Option<i64>,
        ms: Option<i64>,
        day_left: Option<i64>,
        min_left: Option<i64>,
        problem: Option<&str>,
    ) -> Result<()> {
        self.w
            .append(
                "http_log",
                &[
                    ("at", t(at)),
                    ("tenant_id", opt_t(tenant)),
                    ("method", t(method)),
                    ("path", t(path)),
                    ("params", Sql::Null),
                    ("status", opt_i(status)),
                    ("duration_ms", opt_i(ms)),
                    ("day_remaining", opt_i(day_left)),
                    ("min_remaining", opt_i(min_left)),
                    ("problem", opt_t(problem)),
                    ("op_id", Sql::Null),
                ],
            )
            .map_err(err("http_log"))?;
        Ok(())
    }

    /// Cached Xero trial balances at `dates`, computed from the documents already written, plus Xero-only
    /// amounts (`extra`: setup balances that are in no document, or a deliberate parity difference).
    fn trial_balances(
        &self,
        tenant: &str,
        dates: &[String],
        extra: &dyn Fn(&str) -> BTreeMap<String, Decimal>,
        fetched_at: &str,
    ) -> Result<()> {
        let rec = Record::open(&self.path).map_err(err("record"))?;
        let l =
            Ledger::build(&ownpurse_core::view::View::main(&rec), tenant).map_err(err("ledger"))?;
        for date in dates {
            let fy = format!("{}-01-01", &date[..4]);
            let mut tb = trial_balance(&l.postings, &l.chart, date, &fy, None, None);
            for (aid, amt) in extra(date) {
                *tb.entry(aid).or_insert(Decimal::ZERO) += amt;
            }
            tb.retain(|_, v| !v.is_zero());
            self.w
                .append(
                    "reports",
                    &[
                        ("tenant_id", t(tenant)),
                        ("report", t("TrialBalance")),
                        (
                            "params",
                            t(&canonical(&json!({"date": date, "paymentsOnly": "false"}))),
                        ),
                        ("fetched_at", t(fetched_at)),
                        ("content_hash", t("x")),
                        ("payload", t(&canonical(&xero_tb(&tb)))),
                    ],
                )
                .map_err(err("reports"))?;
        }
        Ok(())
    }

    /// Close the writer, VACUUM and checkpoint: one self-contained, deterministic file. `wal` keeps the
    /// write-ahead-log mode the production Writer leaves (the engine and demo
    /// records); otherwise the file switches to a rollback journal (the CLI fixture).
    fn finish(self, wal: bool) -> Result<()> {
        let path = self.path.clone();
        drop(self.w);
        let c = rusqlite::Connection::open(&path).map_err(err("reopen"))?;
        if !wal {
            c.pragma_update(None, "journal_mode", "DELETE")
                .map_err(err("journal_mode"))?;
        }
        c.execute_batch("VACUUM").map_err(err("vacuum"))?;
        if wal {
            c.execute_batch("PRAGMA wal_checkpoint(TRUNCATE)")
                .map_err(err("checkpoint"))?;
        }
        drop(c);
        for suffix in ["-wal", "-shm", "-journal"] {
            let _ = std::fs::remove_file(format!("{}{suffix}", path.display()));
        }
        Ok(())
    }
}

/// Balances rendered the way Xero's TrialBalance report carries them (YTD columns, account attributes).
fn xero_tb(balances: &BTreeMap<String, Decimal>) -> Value {
    let rows: Vec<Value> = balances
        .iter()
        .map(|(aid, v)| {
            let dr = if v.is_sign_positive() && !v.is_zero() {
                v.to_string()
            } else {
                String::new()
            };
            let cr = if v.is_sign_negative() {
                (-*v).to_string()
            } else {
                String::new()
            };
            let attr = json!([{"Id": "account", "Value": aid}]);
            let cells: Vec<Value> = [aid.as_str(), "", "", &dr, &cr]
                .iter()
                .map(|c| json!({"Value": c, "Attributes": attr}))
                .collect();
            json!({"RowType": "Row", "Cells": cells})
        })
        .collect();
    json!({"Reports": [{"ReportTitles": ["Trial Balance"], "Rows": [
        {"RowType": "Header", "Cells": [{"Value": "Account"}, {"Value": "Debit"}, {"Value": "Credit"},
                                        {"Value": "YTD Debit"}, {"Value": "YTD Credit"}]},
        {"RowType": "Section", "Title": "All", "Rows": rows}]}]})
}

fn strings(v: &Value) -> Result<Vec<String>> {
    arr(v)?.iter().map(|x| s(x).map(str::to_string)).collect()
}

// ── the engine fixture (record.sqlite + goldens) and the CLI fixture ───────────────────────────────────

/// Write one engine org's documents (and its sync runs when `synced`), then its cached TBs.
fn engine_org(r: &mut Rec, alias: &str, tenant: &str, o: &Value, now: &str) -> Result<()> {
    let mut amounts = vec![];
    let docs = obj(&o["docs"])?;
    if o["synced"].as_bool() == Some(true) {
        for ent in SYNCED {
            r.sync_run(
                &format!("load-{tenant}-{ent}"),
                tenant,
                ent,
                "full",
                now,
                0,
                0,
                0,
            )?;
        }
    }
    for (ent, items) in docs {
        for doc in expand(ent, arr(items)?, &mut amounts)? {
            r.version(tenant, ent, &doc)?;
        }
    }
    let adjust = o.get("xero_tb_adjust").cloned().unwrap_or(json!({}));
    let extra = |date: &str| -> BTreeMap<String, Decimal> {
        adjust
            .get(date)
            .and_then(Value::as_object)
            .map(|m| {
                m.iter()
                    .map(|(k, v)| {
                        (
                            k.clone(),
                            dec(v.as_str().unwrap_or("0")).unwrap_or_default(),
                        )
                    })
                    .collect()
            })
            .unwrap_or_default()
    };
    r.trial_balances(
        tenant,
        &strings(&o["tb_dates"])?,
        &extra,
        &format!("{now}Z"),
    )
    .map_err(|e| format!("{alias}: {e}"))
}

const SYNCED: [&str; 8] = [
    "Organisation",
    "Accounts",
    "BankTransactions",
    "BankTransfers",
    "ManualJournals",
    "Invoices",
    "CreditNotes",
    "Payments",
];

fn tenants_of(spec: &Value) -> Result<Vec<(String, String)>> {
    obj(&spec["tenants"])?
        .iter()
        .map(|(a, t)| Ok((a.clone(), s(t)?.to_string())))
        .collect()
}

/// The engine record: org `one` then the intercompany orgs, all at the pinned clock.
fn engine_record(spec: &Value, path: &Path) -> Result<Rec> {
    let now = s(&spec["now"])?;
    pin_clock(Some(&format!("{now}Z")));
    let mut r = Rec::create(path)?;
    for (alias, tenant) in tenants_of(spec)? {
        engine_org(&mut r, &alias, &tenant, &spec["orgs"][&alias], now)?;
    }
    Ok(r)
}

/// Every amount the engine spec writes (for the irregular-cents rule).
pub fn engine_amounts(spec: &Value) -> Result<Vec<String>> {
    let mut amounts = vec![];
    for (alias, _) in tenants_of(spec)? {
        let o = &spec["orgs"][&alias];
        for (ent, items) in obj(&o["docs"])? {
            expand(ent, arr(items)?, &mut amounts)?;
        }
        for v in obj(o.get("xero_tb_adjust").unwrap_or(&json!({})))?.values() {
            for a in obj(v)?.values() {
                amounts.push(s(a)?.to_string());
            }
        }
    }
    for items in obj(&spec["cli"]["extra"])?.values() {
        for (ent, docs) in obj(items)? {
            expand(ent, arr(docs)?, &mut amounts)?;
        }
    }
    collect_known_items(&spec["goldens"]["ic_config"], &mut amounts);
    collect_known_items(&spec["cli"]["config"], &mut amounts);
    Ok(amounts)
}

fn collect_known_items(v: &Value, out: &mut Vec<String>) {
    match v {
        Value::Object(m) => {
            if let Some(items) = m.get("known_items").and_then(Value::as_array) {
                for k in items {
                    match &k["amount"] {
                        Value::String(a) => out.push(a.clone()),
                        Value::Number(n) => out.push(n.to_string()),
                        _ => {}
                    }
                }
            }
            m.values().for_each(|x| collect_known_items(x, out));
        }
        Value::Array(a) => a.iter().for_each(|x| collect_known_items(x, out)),
        _ => {}
    }
}

/// `record.sqlite`, `tenants.json` and the engine goldens (`golden-*`, `ic-*`, `compare-*`).
pub fn build_engine(spec: &Value, out: &mut Out) -> Result<()> {
    let path = out.path("record.sqlite")?;
    let r = engine_record(spec, &path)?;
    r.finish(true)?;
    pin_clock(None);
    let tenants = tenants_of(spec)?;
    out.json("tenants.json", &spec["tenants"])?;
    let rec = Record::open(&path).map_err(err("record"))?;
    for (alias, tid) in &tenants {
        for as_of in strings(&spec["goldens"]["ledger_as_of"])? {
            out.json(
                &format!("golden-{alias}-{as_of}.json"),
                &ledger_golden(&rec, tid, &as_of)?,
            )?;
        }
    }
    let tmap: HashMap<String, String> = tenants.iter().cloned().collect();
    for as_of in strings(&spec["goldens"]["ic_as_of"])? {
        out.json(
            &format!("ic-{as_of}.json"),
            &ic_golden(&rec, &tenants, &spec["goldens"]["ic_config"], &as_of)?,
        )?;
    }
    for c in arr(&spec["goldens"]["compare"])? {
        let orgs = strings(&c["orgs"])?;
        let align = s(&c["align"])?;
        let history = c.get("mode").and_then(Value::as_str) == Some("history");
        let name = format!(
            "compare-{}-{align}{}.json",
            orgs.join("+"),
            if history { "-history" } else { "" }
        );
        let g = compare_golden(
            &rec,
            &tmap,
            &orgs,
            "2025-12-31",
            align,
            &spec["goldens"]["compare_map"],
            if history { Mode::History } else { Mode::Auto },
            c["postings_rows"].as_u64().unwrap_or(4) as usize,
        )?;
        out.json(&name, &g)?;
    }
    Ok(())
}

/// `cli/record.sqlite` (the engine record plus sync runs for `one`, contacts, documents that change no
/// trial balance, two Organisation-only orgs and an old call log) and `cli/ownpurse.json`.
pub fn build_cli(spec: &Value, out: &mut Out) -> Result<()> {
    let path = out.path("cli/record.sqlite")?;
    let mut r = engine_record(spec, &path)?;
    let now = s(&spec["now"])?;
    let cli = &spec["cli"];
    let mut tenants: HashMap<String, String> = tenants_of(spec)?.into_iter().collect();
    for (a, t) in obj(&cli["tenants"])? {
        tenants.insert(a.clone(), s(t)?.to_string());
    }
    let one = &tenants[s(&cli["sync_runs"]["org"])?];
    for ent in strings(&cli["sync_runs"]["entities"])? {
        r.sync_run(&format!("cli-{one}-{ent}"), one, &ent, "full", now, 1, 3, 2)?;
    }
    let mut amounts = vec![];
    for (alias, ents) in obj(&cli["extra"])? {
        let tid = tenants
            .get(alias)
            .ok_or_else(|| format!("cli: no tenant for {alias}"))?
            .clone();
        for (ent, items) in obj(ents)? {
            for doc in expand(ent, arr(items)?, &mut amounts)? {
                r.version(&tid, ent, &doc)?;
            }
        }
    }
    for row in arr(&cli["http_log"])? {
        let row = arr(row)?;
        let tenant = row[1].as_str().map(|a| tenants[a].as_str());
        r.call(
            s(&row[0])?,
            tenant,
            s(&row[2])?,
            s(&row[3])?,
            row[4].as_i64(),
            row[5].as_i64(),
            row[6].as_i64(),
            None,
            row[7].as_str(),
        )?;
    }
    r.finish(false)?;
    pin_clock(None);
    out.json("cli/ownpurse.json", &cli["config"])
}

// ── the demo world ─────────────────────────────────────────────────────────────────────────────────────

/// What one record variant changes (see `variants` in world.json).
struct Variant<'a> {
    name: &'a str,
    spec: &'a Value,
}

impl Variant<'_> {
    fn str_of(&self, key: &str) -> Option<&str> {
        self.spec.get(key).and_then(Value::as_str)
    }
}

fn demo_orgs(spec: &Value) -> Result<Vec<&Value>> {
    Ok(arr(&spec["orgs"])?.iter().collect())
}

/// The demo tenants, in config order.
pub fn demo_tenants(spec: &Value) -> Result<Vec<(String, String)>> {
    demo_orgs(spec)?
        .iter()
        .map(|o| Ok((s(&o["alias"])?.to_string(), s(&o["tenant"])?.to_string())))
        .collect()
}

/// The demo org's documents by entity (Organisation, Accounts, BankTransactions, ManualJournals).
fn demo_docs(o: &Value, amounts: &mut Vec<String>) -> Result<Vec<(&'static str, Vec<Value>)>> {
    let alias = s(&o["alias"])?;
    let mut org = Map::new();
    org.insert("OrganisationID".into(), json!(format!("o-{alias}")));
    org.insert("BaseCurrency".into(), json!("USD"));
    org.insert("FinancialYearEndMonth".into(), json!(12));
    org.insert("FinancialYearEndDay".into(), json!(31));
    for (k, v) in obj(&o["organisation"])? {
        org.insert(k.clone(), v.clone());
    }
    Ok(vec![
        ("Organisation", vec![Value::Object(org)]),
        (
            "Accounts",
            expand("Accounts", arr(&o["accounts"])?, amounts)?,
        ),
        (
            "BankTransactions",
            expand("BankTransactions", arr(&o["bank_transactions"])?, amounts)?,
        ),
        (
            "ManualJournals",
            expand("ManualJournals", arr(&o["manual_journals"])?, amounts)?,
        ),
    ])
}

/// Every amount the demo spec writes (for the irregular-cents rule).
pub fn demo_amounts(spec: &Value) -> Result<Vec<String>> {
    let mut amounts = vec![];
    for o in demo_orgs(spec)? {
        demo_docs(o, &mut amounts)?;
        for v in obj(&o["setup"])?.values() {
            amounts.push(s(v)?.to_string());
        }
    }
    Ok(amounts)
}

fn demo_record(spec: &Value, v: &Variant, path: &Path) -> Result<()> {
    let sync_at = s(&spec["sync_at"])?;
    let mut r = Rec::create(path)?;
    let mut amounts = vec![];
    for o in demo_orgs(spec)? {
        let alias = s(&o["alias"])?;
        let tid = s(&o["tenant"])?;
        if v.str_of("never") == Some(alias) {
            continue;
        }
        let when = if v.str_of("stale") == Some(alias) {
            s(&spec["stale_at"])?
        } else {
            sync_at
        };
        pin_clock(Some(&format!("{when}Z")));
        let docs = demo_docs(o, &mut amounts)?;
        let count = |ent: &str| -> i64 {
            docs.iter()
                .find(|(e, _)| *e == ent)
                .map_or(0, |(_, d)| d.len() as i64)
        };
        for (k, ent) in strings(&spec["synced_entities"])?.iter().enumerate() {
            let n = count(ent);
            r.sync_run(
                &format!("demo-{alias}-{ent}"),
                tid,
                ent,
                if k < 2 { "full" } else { "incremental" },
                when,
                1 + (k as i64) % 3,
                n,
                n,
            )?;
        }
        for (ent, items) in &docs {
            for doc in items {
                r.version(tid, ent, doc)?;
            }
        }
        let mut dates = strings(&spec["tb_dates"])?;
        if let Some(no) = v.spec.get("no_tb").filter(|n| n["org"] == alias) {
            dates.retain(|d| d != s(&no["date"]).unwrap_or(""));
        }
        let setup: BTreeMap<String, Decimal> = obj(&o["setup"])?
            .iter()
            .map(|(k, a)| Ok((k.clone(), dec(s(a)?)?)))
            .collect::<Result<_>>()?;
        r.trial_balances(tid, &dates, &|_| setup.clone(), &format!("{when}Z"))?;
    }
    if let Some(extra) = v.str_of("extra_org") {
        pin_clock(Some(&format!("{sync_at}Z")));
        let e = &spec["extra_orgs"][extra];
        let tid = s(&e["tenant"])?;
        let mut org = Map::new();
        org.insert("OrganisationID".into(), json!(tid));
        for (k, x) in obj(&e["organisation"])? {
            org.insert(k.clone(), x.clone());
        }
        r.version(tid, "Organisation", &Value::Object(org))?;
        r.sync_run(
            &format!("demo-{extra}-Organisation"),
            tid,
            "Organisation",
            "full",
            sync_at,
            1,
            1,
            1,
        )?;
    }
    demo_http_log(&r, spec, v)?;
    r.finish(true)?;
    pin_clock(None);
    Ok(())
}

/// Today's call log (2026-10-05) per org, with the last `X-DayLimit-Remaining` = 1,000 − calls; yesterday a
/// login and one 503; one 404 and one 429 today; the variants shift what their state needs.
fn demo_http_log(r: &Rec, spec: &Value, v: &Variant) -> Result<()> {
    let h = &spec["http_log"];
    let tenants = demo_tenants(spec)?;
    let tid_of = |a: &str| {
        tenants
            .iter()
            .find(|(x, _)| x == a)
            .map(|(_, t)| t.clone())
            .unwrap_or_default()
    };
    let calls = |a: &str| h["calls"][a].as_i64().unwrap_or(0);
    let endpoints = strings(&h["endpoints"])?;
    let lost = s(&h["server_error_org"])?;
    let log = |at: &str,
               tenant: Option<&str>,
               method: &str,
               path: &str,
               status: i64,
               ms: i64,
               left: Option<i64>,
               problem: Option<&str>| {
        r.call(
            at,
            tenant,
            method,
            path,
            Some(status),
            Some(ms),
            left,
            left.map(|_| 55),
            problem,
        )
    };
    log(
        "2026-10-04T18:02:11.000000Z",
        None,
        "POST",
        "connect/token",
        200,
        412,
        None,
        None,
    )?;
    log(
        "2026-10-04T18:02:13.000000Z",
        None,
        "GET",
        "connections",
        200,
        233,
        None,
        None,
    )?;
    log(
        "2026-10-04T18:05:40.000000Z",
        Some(&tid_of(lost)),
        "GET",
        "BankTransactions?page=1",
        503,
        30012,
        Some(991),
        Some("Service Unavailable"),
    )?;
    let stale = v.str_of("stale");
    if let Some(a) = stale {
        let tid = tid_of(a);
        for i in 0..calls(a) {
            log(
                &format!("2026-10-03T07:45:{i:02}.000000Z"),
                Some(&tid),
                "GET",
                &endpoints[i as usize % endpoints.len()],
                200,
                200 + i,
                Some(1000 - (i + 1)),
                None,
            )?;
        }
    }
    let low = v.spec.get("lowbudget");
    let mut sec = 0;
    for (alias, tid) in &tenants {
        if v.str_of("never") == Some(alias) || stale == Some(alias) {
            continue;
        }
        let n = calls(alias);
        for i in 0..n {
            let mut left = Some(1000 - (i + 1));
            if let Some(l) = low.filter(|l| l["org"] == alias.as_str()) {
                left = Some(l["left"].as_i64().unwrap_or(0) + (n - i - 1));
            }
            let mut path = endpoints[i as usize % endpoints.len()].clone();
            let (mut status, mut problem) = (200, None);
            if alias == s(&h["not_found_org"])? && i == n - 2 {
                path = format!(
                    "BankTransactions/{}-1111-4111-8111-111111111111/Attachments",
                    &tid[..8]
                );
                (status, problem) = (404, Some("Not Found"));
            }
            if alias == s(&h["rate_limited_org"])? && i == n - 3 {
                (status, problem, left) = (429, Some("Rate limited (minute)"), None);
            }
            let t = 43 * 60 + 37 + sec;
            let at = format!(
                "2026-10-05T07:{:02}:{:02}.{:03}000Z",
                t / 60,
                t % 60,
                (i * 37) % 1000
            );
            log(
                &at,
                Some(tid),
                "GET",
                &path,
                status,
                180 + (i * 53) % 700,
                left,
                problem,
            )?;
            sec += 1;
        }
    }
    let _ = v.name;
    Ok(())
}

/// The demo config (`ownpurse.json`), or the intercompany variant's (`ownpurse-ic.json`).
pub fn demo_config(spec: &Value, ic_variant: bool) -> Result<Value> {
    let c = &spec["config"];
    let mut orgs = Map::new();
    for (a, t) in demo_tenants(spec)? {
        orgs.insert(a, json!(t));
    }
    let mut pairs = arr(&spec["pairs"])?.clone();
    let mut profile = obj(&c["profile"])?.clone();
    if ic_variant {
        profile.insert("store".into(), json!("record-ic.sqlite"));
        for (a, e) in obj(&spec["extra_orgs"])? {
            orgs.insert(a.clone(), e["tenant"].clone());
        }
        pairs.extend(arr(&spec["ic_variant_pairs"])?.iter().cloned());
    }
    // key order as the earlier fixtures: client_id, store, token_file, orgs, extra_links
    let mut p = Map::new();
    for k in ["client_id", "store", "token_file"] {
        p.insert(k.into(), profile[k].clone());
    }
    p.insert("orgs".into(), Value::Object(orgs));
    for (k, x) in profile {
        p.entry(k).or_insert(x);
    }
    let groups: Map<String, Value> = obj(&c["groups"])?
        .iter()
        .map(|(g, m)| (g.clone(), json!({"orgs": m})))
        .collect();
    let mut out = json!({
        "default_profile": c["default_profile"],
        "profiles": {s(&c["default_profile"])?: p},
        "groups": groups,
        "tui": c["tui"],
    });
    // manifests (the accountant gate and the must-be-zero accounts), when the spec has them
    if !c["manifests"].is_null()
        && let Some(m) = out.as_object_mut()
    {
        m.insert("manifests".into(), c["manifests"].clone());
    }
    if let Some(m) = out.as_object_mut() {
        m.insert("parties".into(), spec["parties"].clone());
        m.insert("intercompany".into(), json!({"pairs": pairs}));
        m.insert("account_map".into(), spec["account_map"].clone());
    }
    Ok(out)
}

/// The demo records (every variant), tenants.json, both configs and the demo goldens.
pub fn build_demo(spec: &Value, out: &mut Out) -> Result<()> {
    let variants = obj(&spec["variants"])?;
    for (name, vs) in variants {
        let v = Variant { name, spec: vs };
        let path = out.path(&format!("demo/{}", s(&vs["file"])?))?;
        demo_record(spec, &v, &path)?;
    }
    let tenants = demo_tenants(spec)?;
    let mut tj = Map::new();
    for (a, t) in &tenants {
        tj.insert(a.clone(), json!(t));
    }
    out.json("demo/tenants.json", &Value::Object(tj))?;
    let cfg = demo_config(spec, false)?;
    let cfg_ic = demo_config(spec, true)?;
    out.json("demo/ownpurse.json", &cfg)?;
    out.json("demo/ownpurse-ic.json", &cfg_ic)?;
    let as_of = s(&spec["goldens"]["as_of"])?;
    let dir = out.root.join("demo");
    // intercompany: the variant record with its config, then the main record
    let mut ic_tenants = tenants.clone();
    for (a, e) in obj(&spec["extra_orgs"])? {
        ic_tenants.push((a.clone(), s(&e["tenant"])?.to_string()));
    }
    let rec_ic = Record::open(dir.join("record-ic.sqlite")).map_err(err("record-ic"))?;
    out.json(
        &format!("demo/ic-variant-{as_of}.json"),
        &ic_golden(&rec_ic, &ic_tenants, &cfg_ic, as_of)?,
    )?;
    let rec = Record::open(dir.join("record.sqlite")).map_err(err("record"))?;
    out.json(
        &format!("demo/ic-{as_of}.json"),
        &ic_golden(&rec, &tenants, &cfg, as_of)?,
    )?;
    let mut parity = Map::new();
    for (alias, tid) in &tenants {
        let l =
            Ledger::build(&ownpurse_core::view::View::main(&rec), tid).map_err(err("ledger"))?;
        let mut per = Map::new();
        for (name, mode) in [("auto", Mode::Auto), ("history", Mode::History)] {
            let b = basis(&rec, &l, tid, as_of, mode)?;
            per.insert(
                name.into(),
                json!({"kind": kind(&b), "matches_xero": b.matches_xero(), "differs": b.diffs.len()}),
            );
        }
        parity.insert(alias.clone(), Value::Object(per));
    }
    out.json(&format!("demo/parity-{as_of}.json"), &Value::Object(parity))?;
    Ok(())
}

/// `estimate` (core) on the demo variants and the CLI fixture, clock pinned: `demo/estimate-<date>.json`.
pub fn build_estimate(world: &Value, engine: &Value, out: &mut Out) -> Result<()> {
    use ownpurse_core::estimate::{EstimateSettings, estimate};
    let now_text = s(&world["goldens"]["estimate_now"])?;
    let (date, time) = now_text
        .trim_end_matches('Z')
        .split_once('T')
        .ok_or("estimate_now")?;
    let (y, m, d) = ownpurse_core::dates::parse_iso(date).ok_or("estimate_now date")?;
    let (hh, mm) = time.split_once(':').ok_or("estimate_now time")?;
    let now = ownpurse_core::dates::days_from_civil(y, m, d) * 86_400
        + hh.parse::<i64>().map_err(err("hh"))? * 3600
        + mm.parse::<i64>().map_err(err("mm"))? * 60;
    let per = |rec: &Record, tenants: &[(String, String)]| -> Result<Value> {
        let mut o = Map::new();
        for (alias, tid) in tenants {
            let e = estimate(rec, tid, None, false, EstimateSettings::default(), now)
                .map_err(err("estimate"))?;
            let f = estimate(rec, tid, None, true, EstimateSettings::default(), now)
                .map_err(err("estimate"))?;
            o.insert(alias.clone(), json!({"estimate": e, "full": f}));
        }
        Ok(Value::Object(o))
    };
    let tenants = demo_tenants(world)?;
    let mut variants = Map::new();
    for v in strings(&world["goldens"]["estimate_variants"])? {
        let file = s(&world["variants"][&v]["file"])?;
        let rec = Record::open(out.root.join("demo").join(file)).map_err(err(file))?;
        variants.insert(v.clone(), per(&rec, &tenants)?);
    }
    let mut cli_tenants = tenants_of(engine)?;
    cli_tenants.push((
        "eur".into(),
        s(&engine["cli"]["tenants"]["eur"])?.to_string(),
    ));
    let rec = Record::open(out.root.join("cli/record.sqlite")).map_err(err("cli record"))?;
    let cli = per(&rec, &cli_tenants)?;
    out.json(
        &format!("demo/estimate-{date}.json"),
        &json!({"now": now_text, "variants": variants, "cli": cli}),
    )
}

// ── goldens (computed by the Rust core; regression goldens) ─────────────────────────────────────────────

fn basis(rec: &Record, l: &Ledger, tid: &str, as_of: &str, mode: Mode) -> Result<Basis> {
    let fy = rec.fy_end(tid).map_err(err("fy_end"))?;
    resolve(rec, l, tid, as_of, fy, mode).map_err(err("resolve"))
}

fn kind(b: &Basis) -> &'static str {
    match b.kind {
        BasisKind::XeroOpening => "xero-opening",
        BasisKind::History => "history",
    }
}

fn d(v: Decimal) -> Value {
    Value::String(v.to_string())
}

/// Postings, unbalanced documents, skipped counts and both bases (`rust/crates/ownpurse-core/tests/golden.rs`).
pub fn ledger_golden(rec: &Record, tid: &str, as_of: &str) -> Result<Value> {
    let l = Ledger::build(&ownpurse_core::view::View::main(rec), tid).map_err(err("ledger"))?;
    let postings: Vec<Value> = l
        .postings
        .iter()
        .map(|p| {
            json!({"date": p.date, "source": p.source, "source_id": p.source_id, "line": p.line,
                   "account_id": p.account_id, "amount": d(p.amount), "description": p.description,
                   "contact": p.contact, "reference": p.reference, "kind": p.kind})
        })
        .collect();
    let unbalanced: Vec<Value> = l
        .unbalanced
        .iter()
        .map(|u| json!({"source": u.source, "id": u.id, "date": u.date, "imbalance": d(u.imbalance)}))
        .collect();
    let mut bases = Map::new();
    for (name, mode) in [("auto", Mode::Auto), ("history", Mode::History)] {
        let b = basis(rec, &l, tid, as_of, mode)?;
        let balances: Map<String, Value> =
            b.balances.iter().map(|(k, v)| (k.clone(), d(*v))).collect();
        let diffs: Vec<Value> = b
            .diffs
            .iter()
            .map(|x| {
                json!({"account_id": x.account_id, "local": d(x.local), "xero": d(x.xero),
                       "difference": d(x.difference)})
            })
            .collect();
        let opening_for: Map<String, Value> = b
            .balances
            .keys()
            .map(|aid| (aid.clone(), d(b.opening_for(&l, aid, &b.fy_start))))
            .collect();
        bases.insert(
            name.into(),
            json!({"kind": kind(&b), "fy_start": b.fy_start, "opening_date": b.opening_date,
                   "matches_xero": b.matches_xero(), "balances": balances, "diffs": diffs,
                   "opening_for": opening_for}),
        );
    }
    Ok(
        json!({"tenant_id": tid, "as_of": as_of, "postings": postings, "unbalanced": unbalanced,
              "skipped": l.skipped, "bases": bases}),
    )
}

/// The full `intercompany::build` result plus checks, the envelope, every sort order and a one-pair subset
/// (`rust/crates/ownpurse-core/tests/golden_ic.rs`).
pub fn ic_golden(
    rec: &Record,
    tenants: &[(String, String)],
    config: &Value,
    as_of: &str,
) -> Result<Value> {
    let tmap: HashMap<String, String> = tenants.iter().cloned().collect();
    let ctx = IcContext::new(ownpurse_core::view::View::main(rec), tmap, config, as_of);
    let result = ic::build(&ctx, None).map_err(err("ic"))?;
    let pairs = result["pairs"].as_array().cloned().unwrap_or_default();
    let mut sorted = Map::new();
    for m in ["status", "difference", "pair"] {
        let mut v = pairs.clone();
        v.sort_by(ic::sort_cmp(m));
        sorted.insert(
            m.into(),
            Value::Array(v.iter().map(|p| p["id"].clone()).collect()),
        );
    }
    let subset = match pairs.first() {
        Some(p) => {
            let ids = vec![s(&p["id"])?.to_string()];
            ic::build(&ctx, Some(&ids)).map_err(err("ic subset"))?
        }
        None => Value::Null,
    };
    let tj: Map<String, Value> = tenants.iter().map(|(a, t)| (a.clone(), json!(t))).collect();
    Ok(
        json!({"as_of": as_of, "tenants": tj, "config": config, "result": result,
              "checks": ic::checks(&result),
              "envelope": ic::envelope(&result, "record.sqlite", "2026-10-05T00:00:00Z"),
              "sorted": sorted, "subset": subset}),
    )
}

/// `tb_compare` + `flatten` over the TUI's trial-balance rows, and `postings_compare` for the first aligned
/// rows (`rust/crates/ownpurse-core/tests/golden_compare.rs`).
#[allow(clippy::too_many_arguments)]
pub fn compare_golden(
    rec: &Record,
    tenants: &HashMap<String, String>,
    orgs: &[String],
    as_of: &str,
    align: &str,
    account_map: &Value,
    mode: Mode,
    postings_rows: usize,
) -> Result<Value> {
    let mut ledgers: HashMap<String, Ledger> = HashMap::new();
    let mut bases: HashMap<String, Basis> = HashMap::new();
    for o in orgs {
        let tid = &tenants[o];
        let l = Ledger::build(&ownpurse_core::view::View::main(rec), tid).map_err(err("ledger"))?;
        let b = basis(rec, &l, tid, as_of, mode)?;
        ledgers.insert(o.clone(), l);
        bases.insert(o.clone(), b);
    }
    let tbs: HashMap<String, cmp::TbColumn> = orgs
        .iter()
        .map(|o| {
            (
                o.clone(),
                cmp::tb_rows(&ledgers[o], &bases[o], RowStyle::Tui),
            )
        })
        .collect();
    let charts: HashMap<String, &Chart> = orgs
        .iter()
        .map(|o| (o.clone(), &ledgers[o].chart))
        .collect();
    let result =
        cmp::tb_compare(orgs, &tbs, &charts, align, account_map).map_err(err("compare"))?;
    let rows = cmp::flatten(&result).map_err(err("flatten"))?;
    let since = bases[&orgs[0]].fy_start.clone();
    let lrefs: HashMap<String, &Ledger> = ledgers.iter().map(|(k, v)| (k.clone(), v)).collect();
    let mut postings = vec![];
    for r in rows
        .iter()
        .filter(|r| r["section"] == "aligned")
        .take(postings_rows)
    {
        let ids: HashMap<String, Vec<String>> = obj(&r["member_ids"])?
            .iter()
            .map(|(o, v)| Ok((o.clone(), strings(v)?)))
            .collect::<Result<_>>()?;
        let openings: HashMap<String, Decimal> = orgs
            .iter()
            .map(|o| {
                let total = ids.get(o).into_iter().flatten().fold(cmp::D0, |acc, aid| {
                    ownpurse_core::money::add(acc, bases[o].opening_for(&ledgers[o], aid, &since))
                });
                (o.clone(), total)
            })
            .collect();
        let ov: Map<String, Value> = orgs
            .iter()
            .map(|o| (o.clone(), cmp::dstr(openings[o])))
            .collect();
        let pc = cmp::postings_compare(orgs, &lrefs, &ids, &since, as_of, &openings)
            .map_err(err("postings_compare"))?;
        postings.push(json!({"key": r["key"], "openings": ov, "result": pc}));
    }
    let tj: Map<String, Value> = orgs
        .iter()
        .map(|o| (o.clone(), json!(tenants[o])))
        .collect();
    Ok(json!({"orgs": orgs, "as_of": as_of,
              "mode": if mode == Mode::History { "history" } else { "auto" },
              "align": align, "tenants": tj, "account_map": account_map, "result": result,
              "flatten": rows, "postings": postings}))
}

// ── entry points ───────────────────────────────────────────────────────────────────────────────────────

/// Generate every fixture from the specs in `spec_dir` (the committed `rust/tests/fixtures`) into `out_dir`.
/// Returns the relative paths written.
pub fn generate(spec_dir: &Path, out_dir: &Path) -> Result<Vec<String>> {
    // the Writer's pinned clock is process-wide: one generation at a time (tests run in threads)
    static ONE_AT_A_TIME: std::sync::Mutex<()> = std::sync::Mutex::new(());
    let _guard = ONE_AT_A_TIME.lock().unwrap_or_else(|e| e.into_inner());
    let engine = load_spec(&spec_dir.join("engine.json"))?;
    let world = load_spec(&spec_dir.join("demo/world.json"))?;
    let mut out = Out::new(out_dir);
    build_engine(&engine, &mut out)?;
    build_cli(&engine, &mut out)?;
    build_demo(&world, &mut out)?;
    classify::build_classify(&world, &mut out)?;
    build_estimate(&world, &engine, &mut out)?;
    // reading the WAL-mode records (goldens) leaves -shm/-wal beside them: never leave any behind
    for f in out.files.iter().filter(|f| f.ends_with(".sqlite")) {
        for suffix in ["-wal", "-shm"] {
            let _ = std::fs::remove_file(out_dir.join(format!("{f}{suffix}")));
        }
    }
    Ok(out.files)
}

/// The irregular-cents rule: a money amount never ends in .00 or .50 (so no invented figure can echo a
/// round amount in anyone's real books). Returns the offending amounts.
pub fn round_amounts(amounts: &[String]) -> Vec<String> {
    amounts
        .iter()
        .filter(|a| match dec(a) {
            Ok(v) => {
                let cents = v.abs() * Decimal::from(100);
                v.is_zero()
                    || !cents.fract().is_zero()
                    || (cents % Decimal::from(100)).is_zero()
                    || (cents % Decimal::from(100)) == Decimal::from(50)
            }
            Err(_) => true,
        })
        .cloned()
        .collect()
}

/// Regenerate into a temp folder and compare every file with the committed one, byte for byte.
pub fn check(fixtures: &Path) -> Result<usize> {
    let tmp = std::env::temp_dir().join(format!("xtask-check-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&tmp);
    let files = generate(fixtures, &tmp)?;
    let mut bad = vec![];
    for f in &files {
        let a = std::fs::read(tmp.join(f)).map_err(err(f))?;
        match std::fs::read(fixtures.join(f)) {
            Ok(b) if a == b => {}
            Ok(_) => bad.push(format!("{f} differs from its spec")),
            Err(_) => bad.push(format!("{f} is missing")),
        }
    }
    let _ = std::fs::remove_dir_all(&tmp);
    if bad.is_empty() {
        Ok(files.len())
    } else {
        Err(format!(
            "{} (run `cargo run -p xtask -- demo fixtures`)",
            bad.join("; ")
        ))
    }
}

/// `xtask demo fixtures | states | check`.
pub fn run(args: &[String]) -> Result<()> {
    let fixtures = fixtures_dir();
    let print = |files: Vec<String>| {
        for f in &files {
            println!("wrote rust/tests/fixtures/{f}");
        }
    };
    match args.first().map(String::as_str) {
        Some("fixtures") => generate(&fixtures, &fixtures).map(print),
        Some("states") => states::write(&fixtures).map(print),
        Some("check") => check(&fixtures).map(|n| println!("{n} fixture files equal their spec")),
        Some("big") => big(&fixtures, &args[1..]),
        _ => Err("usage: xtask demo fixtures | states | check | big <dir> [--docs N]".into()),
    }
}

/// `xtask demo big <dir> [--docs N]`: a synthetic large world for performance work (never committed): the demo
/// record plus N invented documents (default 10,000), the demo `classify.toml` and store, and an
/// `ownpurse.json` pointing at them. Then e.g. `OWNPURSE_CONFIG=<dir>/ownpurse.json ownpurse classify queue`.
fn big(fixtures: &Path, args: &[String]) -> Result<()> {
    let dir = PathBuf::from(
        args.first()
            .ok_or("usage: xtask demo big <dir> [--docs N]")?,
    );
    let docs = match args.iter().position(|a| a == "--docs") {
        Some(i) => args
            .get(i + 1)
            .and_then(|n| n.parse().ok())
            .ok_or("--docs needs a number")?,
        None => 10_000,
    };
    std::fs::create_dir_all(dir.join("db")).map_err(err("big dir"))?;
    let demo = fixtures.join("demo");
    let lines = ownpurse_core::classify::synth::big_record(
        &demo.join("record.sqlite"),
        &dir.join("record.sqlite"),
        docs,
    )
    .map_err(err("synthetic record"))?;
    for f in ["classify.toml", "ownpurse.json", "tenants.json"] {
        std::fs::copy(demo.join(f), dir.join(f)).map_err(err(f))?;
    }
    std::fs::copy(
        demo.join("db/classify.sqlite"),
        dir.join("db/classify.sqlite"),
    )
    .map_err(err("store"))?;
    println!(
        "wrote {} ({docs} documents, {lines} lines added)",
        dir.display()
    );
    Ok(())
}
