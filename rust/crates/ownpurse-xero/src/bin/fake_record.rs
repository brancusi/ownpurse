//! Test helper for the TUI's fake CLI (`rust/tests/fixtures/fake-cli/ownpurse`): appends to a record COPY
//! the rows a real `ownpurse sync <org>` run of that scenario writes, through the production `Writer`
//! (append-only tables, SHA-256 chain), with timestamps at a pinned clock so snapshots are stable.
//!
//!     ownpurse-fake-record <record> <scenario> <tenant>[,<tenant>…] <at>
//!
//! - `ok` (each tenant): 15 calls — the 14 entities in sync order, BankTransactions over 2 pages — each an
//!   `http_log` row with the budget headers (`X-DayLimit-Remaining` counting down from the last reported
//!   figure that day, else the 1,000 default), one `sync_runs` row per entity (25 records), and 3 new
//!   `versions` of BankTransactions;
//! - `limit`: the first call answered 429 `X-Rate-Limit-Problem: day` with `X-DayLimit-Remaining: 0`, and
//!   the failed run (`BudgetExhausted`);
//! - `error`: the first call answered 401 twice (the retry after a refresh), and the failed run
//!   (`NeedsLogin`);
//! - `write` (the first tenant): a Phase 8 write as `write seal` + `approve-demo` + `run` leave it: a
//!   registered package, its demo standing approval, the attempt log (sending, confirmed) and the created
//!   journal read back, run by the production run engine against the in-process loopback fake Xero.
//!
//! `<at>` is `YYYY-MM-DDTHH:MM:SSZ`: the run finishes then. Never point this at a real record: it is not
//! installed by install.sh and refuses a path that isn't under the system temp folder.

use ownpurse_core::canon::canonical;
use ownpurse_xero::http::py_dumps_sorted;
use ownpurse_xero::store::{Writer, iso_micros, new_id, sha256_hex};
use ownpurse_xero::sync::{entities, xero_datetime};
use rusqlite::types::Value as Sql;
use serde_json::Value;
use std::collections::HashMap;
use std::time::Duration;

const PAGE_SIZE: i64 = 1000;

fn parse_at(s: &str) -> u64 {
    let (date, time) = s
        .trim_end_matches('Z')
        .split_once('T')
        .expect("at: date T time");
    let (y, m, d) = ownpurse_core::dates::parse_iso(date).expect("at: date");
    let hms: Vec<i64> = time
        .split(':')
        .map(|x| x.parse().expect("at: time"))
        .collect();
    let secs = ownpurse_core::dates::days_from_civil(y, m, d) * 86400
        + hms[0] * 3600
        + hms.get(1).copied().unwrap_or(0) * 60
        + hms.get(2).copied().unwrap_or(0);
    secs as u64
}

fn stamp(secs: u64, millis: u64) -> String {
    iso_micros(Duration::from_secs(secs) + Duration::from_millis(millis))
}

struct Run<'a> {
    w: &'a Writer,
    tenant: &'a str,
    at: u64,
    left: i64,
    minute: i64,
}

impl Run<'_> {
    fn call(&mut self, secs: u64, path: &str, params: &[(String, Value)], status: i64, day: bool) {
        let (day_remaining, problem) = match status {
            200 => {
                self.left -= 1;
                (Sql::Integer(self.left), Sql::Null)
            }
            429 => (Sql::Integer(0), Sql::Text("day".into())),
            _ => (Sql::Null, Sql::Null),
        };
        self.minute -= 1;
        let min = if day || status == 200 {
            Sql::Integer(self.minute)
        } else {
            Sql::Null
        };
        self.w
            .append(
                "http_log",
                &[
                    ("at", Sql::Text(stamp(secs, 120))),
                    ("tenant_id", Sql::Text(self.tenant.into())),
                    ("method", Sql::Text("GET".into())),
                    ("path", Sql::Text(path.into())),
                    (
                        "params",
                        if params.is_empty() {
                            Sql::Null
                        } else {
                            Sql::Text(py_dumps_sorted(params))
                        },
                    ),
                    ("status", Sql::Integer(status)),
                    ("duration_ms", Sql::Integer(240)),
                    ("day_remaining", day_remaining),
                    ("min_remaining", min),
                    ("problem", problem),
                    ("op_id", Sql::Null),
                ],
            )
            .expect("append http_log");
    }

    #[allow(clippy::too_many_arguments)]
    fn sync_run(
        &self,
        id: String,
        entity: &str,
        mode: &str,
        started: u64,
        outcome: &str,
        calls: i64,
        fetched: i64,
        new_versions: i64,
        high_water: Option<String>,
        detail: Option<String>,
    ) {
        self.w
            .append(
                "sync_runs",
                &[
                    ("id", Sql::Text(id.clone())),
                    ("tenant_id", Sql::Text(self.tenant.into())),
                    ("entity", Sql::Text(entity.into())),
                    ("mode", Sql::Text(mode.into())),
                    ("started_at", Sql::Text(stamp(started, 0))),
                    ("finished_at", Sql::Text(stamp(self.at, 0))),
                    ("outcome", Sql::Text(outcome.into())),
                    ("calls", Sql::Integer(calls)),
                    ("fetched", Sql::Integer(fetched)),
                    ("new_versions", Sql::Integer(new_versions)),
                    ("high_water", high_water.map(Sql::Text).unwrap_or(Sql::Null)),
                    ("detail", detail.map(Sql::Text).unwrap_or(Sql::Null)),
                ],
            )
            .expect("append sync_runs");
    }

    fn failed(&mut self, status: i64, class: &str, message: &str) {
        let first = &entities()[0];
        let started = self.at - 2;
        self.call(started, first.name, &[], status, status == 429);
        if status == 401 {
            self.call(started + 1, first.name, &[], status, false);
        }
        let detail = py_dumps_sorted(&[
            ("error".into(), Value::from(class)),
            ("message".into(), Value::from(message)),
        ]);
        self.sync_run(
            new_id().expect("uuid"),
            first.name,
            "full",
            started,
            "error",
            0,
            0,
            0,
            None,
            Some(detail),
        );
    }

    fn ok(&mut self) {
        // records per entity (25 in all) and the BankTransactions that change (3 new versions)
        let fetched: HashMap<&str, i64> = [
            ("Organisation", 1),
            ("Accounts", 20),
            ("BankTransactions", 4),
        ]
        .into_iter()
        .collect();
        let mut secs = self.at - 15;
        for e in entities() {
            let hw: Option<String> = self.w.high_water(self.tenant, e.name).expect("high water");
            // Syncer's rule (without the 7-day age check: the run is pinned to the fixture's day)
            let last_full = self
                .w
                .last_sync(self.tenant, e.name, Some("full"))
                .expect("last sync");
            let mode = if (e.paged || e.name == "Contacts") && hw.is_some() && last_full.is_some() {
                "incremental"
            } else {
                "full"
            };
            let started = secs;
            let pages = if e.name == "BankTransactions" { 2 } else { 1 };
            for page in 1..=pages {
                let mut p: Vec<(String, Value)> = e
                    .params
                    .iter()
                    .map(|(k, v)| (k.to_string(), Value::from(*v)))
                    .collect();
                if e.paged {
                    p.push(("page".into(), Value::from(page)));
                    p.push(("pageSize".into(), Value::from(PAGE_SIZE)));
                    if e.order {
                        p.push(("order".into(), Value::from("UpdatedDateUTC ASC")));
                    }
                }
                self.call(secs, e.name, &p, 200, false);
                secs += 1;
            }
            let id = new_id().expect("uuid");
            let (new_versions, high) = if e.name == "BankTransactions" {
                self.new_versions(e.name, 3, &id)
            } else {
                (0, hw)
            };
            self.sync_run(
                id,
                e.name,
                mode,
                started,
                "ok",
                pages,
                fetched.get(e.name).copied().unwrap_or(0),
                new_versions,
                high,
                None,
            );
        }
    }

    /// Re-record `n` current documents with `UpdatedDateUTC` at the run's time (`record_version`,
    /// with the pinned `observed_at`).
    fn new_versions(&self, entity: &str, n: usize, run_id: &str) -> (i64, Option<String>) {
        let mut st = self
            .w
            .conn()
            .prepare(
                "SELECT xero_id, version, status, payload FROM current WHERE tenant_id=?1 AND entity=?2 \
                 ORDER BY xero_id LIMIT ?3",
            )
            .expect("current");
        let docs: Vec<(String, i64, Option<String>, String)> = st
            .query_map((self.tenant, entity, n as i64), |r| {
                Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?))
            })
            .expect("current rows")
            .collect::<Result<_, _>>()
            .expect("current rows");
        let updated = format!("/Date({}000+0000)/", self.at);
        let upd = xero_datetime(&updated);
        for (xid, version, status, payload) in &docs {
            let mut v: Value = serde_json::from_str(payload).expect("payload json");
            v["UpdatedDateUTC"] = Value::from(updated.clone());
            let text = canonical(&v);
            let opt = |s: Option<&str>| s.map(|x| Sql::Text(x.into())).unwrap_or(Sql::Null);
            self.w
                .append(
                    "versions",
                    &[
                        ("tenant_id", Sql::Text(self.tenant.into())),
                        ("entity", Sql::Text(entity.into())),
                        ("xero_id", Sql::Text(xid.clone())),
                        ("version", Sql::Integer(version + 1)),
                        ("content_hash", Sql::Text(sha256_hex(&text))),
                        ("status", opt(status.as_deref())),
                        ("xero_updated_utc", opt(upd.as_deref())),
                        ("observed_at", Sql::Text(stamp(self.at, 0))),
                        ("source", Sql::Text("sync".into())),
                        ("sync_run_id", Sql::Text(run_id.into())),
                        ("caused_by_op", Sql::Null),
                        ("payload", Sql::Text(text)),
                    ],
                )
                .expect("append versions");
        }
        (docs.len() as i64, upd)
    }
}

fn main() {
    let a: Vec<String> = std::env::args().skip(1).collect();
    let [record, scenario, tenants, at] = [&a[0], &a[1], &a[2], &a[3]];
    let tmp = std::env::temp_dir()
        .canonicalize()
        .unwrap_or_else(|_| std::env::temp_dir());
    let real = std::path::Path::new(record)
        .canonicalize()
        .expect("record path");
    assert!(
        real.starts_with(&tmp) || real.starts_with(std::env::temp_dir()),
        "ownpurse-fake-record only writes record copies under {}, got {}",
        tmp.display(),
        real.display()
    );
    let at = parse_at(at);
    let w = Writer::open(&real).expect("open record copy");
    let day = stamp(at, 0)[..10].to_string();
    for tenant in tenants.split(',') {
        let left: i64 = w
            .conn()
            .query_row(
                "SELECT day_remaining FROM http_log WHERE tenant_id=?1 AND day_remaining IS NOT NULL \
                 AND substr(at,1,10)=?2 ORDER BY id DESC LIMIT 1",
                (tenant, &day),
                |r| r.get(0),
            )
            .unwrap_or(1000);
        let mut run = Run {
            w: &w,
            tenant,
            at,
            left,
            minute: 60,
        };
        match scenario.as_str() {
            "ok" | "all" | "slow" | "wait" => run.ok(),
            "limit" => run.failed(
                429,
                "BudgetExhausted",
                "Xero daily call limit reached for this organisation.",
            ),
            "error" => run.failed(401, "NeedsLogin", "Xero rejected the access token."),
            "write" => {
                ownpurse_xero::store::pin_clock(Some(&stamp(at, 0)));
                write_scenario(&w, tenant, &stamp(at, 0)[..10]);
                break;
            }
            _ => {}
        }
    }
    // readers that open the copy read-only see everything without the WAL
    let _ = w.conn().execute_batch("PRAGMA wal_checkpoint(TRUNCATE)");
}

/// The `write` scenario: one package with one journal (two expense accounts of the tenant's chart, 147.38),
/// sealed, registered, approved (demo standing) and run through the production engine against the loopback
/// fake Xero, so the record holds exactly what a real write leaves.
fn write_scenario(w: &Writer, tenant: &str, date: &str) {
    use ownpurse_core::write::*;
    use ownpurse_xero::write::fake::{FakeXero, State};
    use ownpurse_xero::write::run::{self, Ctx};
    use ownpurse_xero::write::{RecordApprovals, UreqWire, WriteSettings, XeroWriter, log};
    let rec = ownpurse_core::Record::open(&w.path).expect("record");
    let accounts = ownpurse_core::view::View::main(&rec)
        .payloads(tenant, "Accounts")
        .unwrap_or_default();
    let chart: Vec<(String, String, String)> = accounts
        .iter()
        .filter_map(|a| {
            Some((
                a["Code"].as_str().filter(|c| !c.is_empty())?.to_string(),
                a["Name"].as_str().unwrap_or("").to_string(),
                a["Class"]
                    .as_str()
                    .or(a["Type"].as_str())
                    .unwrap_or("EXPENSE")
                    .to_string(),
            ))
        })
        .collect();
    let exp: Vec<&String> = chart
        .iter()
        .filter(|c| c.2 == "EXPENSE")
        .map(|c| &c.0)
        .collect();
    let (dr, cr) = (exp[0].clone(), exp[1].clone());
    let mut st = State::default();
    let refs: Vec<(&str, &str, &str)> = chart
        .iter()
        .map(|(a, b, c)| (a.as_str(), b.as_str(), c.as_str()))
        .collect();
    st.add_tenant(tenant, "Demo Company (US)", &refs);
    let fake = FakeXero::start(st).expect("fake xero");
    let tmp = std::env::temp_dir().join(format!("xa-fake-write-{}", std::process::id()));
    let _ = std::fs::create_dir_all(&tmp);
    let tok = tmp.join("tokens.json");
    std::fs::write(
        &tok,
        r#"{"access_token":"AT","refresh_token":"RT","obtained_at":4000000000,"expires_in":1800,"scope":"accounting.manualjournals"}"#,
    )
    .expect("token");
    let tokens = ownpurse_xero::TokenStore::new(&tok, "CLIENT");
    let pid = "fixture-write-01";
    let m = marker(pid, "op-1");
    let narration = format!("Owner-paid cupping supplies · {m}");
    let j = Journal {
        date: date.to_string(),
        narration: narration.clone(),
        lines: vec![
            JournalLine {
                account_code: dr.clone(),
                description: "tins".into(),
                amount: "147.38".parse().unwrap_or_default(),
            },
            JournalLine {
                account_code: cr.clone(),
                description: "tins".into(),
                amount: "-147.38".parse().unwrap_or_default(),
            },
        ],
        line_amount_types: "NoTax".into(),
        show_on_cash_basis_reports: true,
    };
    let p = Package {
        v: PACKAGE_V,
        id: pid.into(),
        title: "Owner-paid cupping supplies".into(),
        profile: "demo".into(),
        created_at: stamp_now(),
        accountant_decision: None,
        ops: vec![Operation {
            id: "op-1".into(),
            org: "demo".into(),
            tenant_id: tenant.into(),
            request: ops::journal_request(&j),
            op: Op::CreateJournal(j),
            preconditions: vec![Precondition::MarkerAbsent { marker: m }],
            expected_after: None,
            expected: vec![],
            recovery: None,
            depends_on: vec![],
            decision_refs: vec![],
            source_refs: vec![],
            gated: None,
            flags: vec![],
            history_note: None,
        }],
        excluded: vec![],
        checklist: vec![],
        supersedes: vec![],
        source: None,
        seal: None,
    };
    let p = seal::seal(p).expect("seal");
    let hash = p.seal.clone().unwrap_or_default();
    log::register_package(w, &p, &hash).expect("register");
    let now = stamp_now();
    RecordApprovals { store: w }
        .record(&ApprovalRecord {
            hash: hash.clone(),
            verbatim: "Play freely on the Demo Company; it can be reset.".into(),
            via: Via::DemoStanding,
            where_: "standing-approval.txt".into(),
            at: now.clone(),
            scope: ApprovalScope::All(AllOps::All),
            recorded_at: now,
        })
        .expect("approve");
    let settings = WriteSettings {
        profile: "demo".into(),
        api_base: fake.api_base(),
        connections_url: fake.connections_url(),
        orgs: vec![("demo".into(), tenant.into())],
        forbid_tenants: vec![],
        require_tenant_name: None,
        allow_writes: true,
        loopback_only: true,
    };
    let mut x = XeroWriter::new(settings, w, &tokens, Box::new(UreqWire::default()));
    let approvals = RecordApprovals { store: w };
    let mut ctx = Ctx {
        transport: &mut x,
        store: w,
        approvals: &approvals,
        profile: "demo".into(),
        allow_writes: true,
        run_env: Some(hash),
        base_dir: tmp.clone(),
        batch: BATCH_MAX,
        dry_run: false,
        require_tenant_name: None,
    };
    let rep = run::run(&p, &mut ctx).expect("run");
    assert!(rep.stopped.is_none(), "{:?}", rep.stopped);
    let _ = std::fs::remove_dir_all(&tmp);
}

fn stamp_now() -> String {
    ownpurse_xero::store::now_iso()
}
