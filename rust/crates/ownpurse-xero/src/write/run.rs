//! `write run`, `write verify`, `write reconcile`, `write status`: the execution side (design §3).
//!
//! A run needs three locks (the profile's `allow_writes`, `OWNPURSE_WRITE_RUN=<hash>` for this one
//! invocation, an approval recorded for the hash) plus the scopes its operations need. It then takes at most
//! [`BATCH_MAX`] pending operations in order, and for each: checks its preconditions live, appends
//! `sending` (durably, before the request), sends with the stable Idempotency-Key, appends the outcome, and
//! reads the target back. It stops at the first refusal, discrepancy or uncertain attempt. Nothing is ever
//! re-sent after an uncertain attempt until `reconcile` has proved by read-back whether it landed.

use super::log::{OpLog, op_key};
use crate::store::Writer;
use ownpurse_core::write::{
    ApprovalStore, AttemptState, BATCH_MAX, Body, JournalRef, Op, Operation, Package, Precondition,
    RUN_ENV, Response, Send, TransportError, WriteError, WriteTransport, exit, fingerprint,
    idempotency_key, marker, sha256_hex,
};
use rusqlite::types::Value as Sql;
use serde_json::{Value, json};
use std::path::Path;

/// The seal of a package (`core::write::seal::hash`: sha256 of its canonical JSON without `seal`).
pub fn seal_of(p: &Package) -> String {
    ownpurse_core::write::seal::hash(p).unwrap_or_default()
}

/// The scope an operation needs.
pub fn scope_for(op: &Op) -> &'static str {
    match op {
        Op::CreateJournal(_) | Op::VoidJournal { .. } => "accounting.manualjournals",
        Op::RecodeBankLine(_) => "accounting.banktransactions",
        Op::Attach(_) => "accounting.attachments",
        Op::AddNote(_) => note_scope(op.entity()),
        Op::CreateBankTransaction(_)
        | Op::VoidBankTransaction { .. }
        | Op::CreateBankTransfer(_) => "accounting.banktransactions",
        Op::CreateContact(_) | Op::ArchiveContact { .. } => "accounting.contacts",
    }
}

/// The scope a history note on an entity needs (the entity's own write scope).
pub fn note_scope(entity: &str) -> &'static str {
    match entity {
        "ManualJournals" => "accounting.manualjournals",
        "Contacts" => "accounting.contacts",
        _ => "accounting.banktransactions",
    }
}

/// Where an operation stands, from its attempt log.
#[derive(Clone, Debug, PartialEq)]
pub enum OpStatus {
    Pending,
    /// Landed: the target's Xero id.
    Done(Option<String>),
    Uncertain,
    Rejected(Value),
}

pub struct Ctx<'a> {
    pub transport: &'a mut dyn WriteTransport,
    pub store: &'a Writer,
    pub approvals: &'a dyn ApprovalStore,
    pub profile: String,
    pub allow_writes: bool,
    /// `OWNPURSE_WRITE_RUN` as set for this invocation.
    pub run_env: Option<String>,
    /// Attachment paths resolve against this (the package file's folder).
    pub base_dir: std::path::PathBuf,
    pub batch: usize,
    pub dry_run: bool,
    /// The profile's `require_tenant_name` (the demo-only probes need it).
    pub require_tenant_name: Option<String>,
}

impl OpStatus {
    /// pending | landed | uncertain | rejected
    pub fn word(&self) -> &'static str {
        match self {
            OpStatus::Pending => "pending",
            OpStatus::Done(_) => "landed",
            OpStatus::Uncertain => "uncertain",
            OpStatus::Rejected(_) => "rejected",
        }
    }

    /// Why it was rejected, in words (Xero's errors, or the precondition's reason).
    pub fn reason(&self) -> Option<String> {
        let OpStatus::Rejected(d) = self else {
            return None;
        };
        let errors: Vec<String> = d["errors"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(|e| e.as_str().map(str::to_string))
            .collect();
        Some(if !errors.is_empty() {
            errors.join("; ")
        } else if let Some(r) = d["reason"].as_str() {
            r.to_string()
        } else {
            ownpurse_core::canon::canonical(d)
        })
    }
}

/// What a run did, per operation, and why it stopped.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct RunReport {
    pub hash: String,
    /// (op id, outcome word, detail)
    pub ops: Vec<(String, String, Value)>,
    pub stopped: Option<WriteError>,
    pub remaining: usize,
}

impl RunReport {
    pub fn to_json(&self) -> Value {
        json!({
            "hash": self.hash,
            "ops": self.ops.iter().map(|(id, w, d)| json!({"op": id, "outcome": w, "detail": d})).collect::<Vec<_>>(),
            "stopped": self.stopped.as_ref().map(|e| json!({"error": e.code, "message": e.message, "exit": e.exit})),
            "remaining": self.remaining,
        })
    }
}

fn err(code: &str, exit: i32, msg: impl Into<String>) -> WriteError {
    WriteError::new(code, exit, msg)
}

/// The status of every operation, from the log.
pub fn statuses(store: &Writer, hash: &str, p: &Package) -> Result<Vec<OpStatus>, WriteError> {
    let log = OpLog { store };
    p.ops
        .iter()
        .map(|op| {
            let attempts = log.attempts(&op_key(hash, &op.id))?;
            let Some(last) = attempts.last() else {
                return Ok(OpStatus::Pending);
            };
            Ok(match last.state {
                AttemptState::Confirmed => {
                    OpStatus::Done(last.detail["target_id"].as_str().map(str::to_string))
                }
                AttemptState::Reconciled if last.detail["landed"] == true => {
                    OpStatus::Done(last.detail["target_id"].as_str().map(str::to_string))
                }
                AttemptState::Reconciled | AttemptState::Planned | AttemptState::Approved => {
                    OpStatus::Pending
                }
                AttemptState::Sending | AttemptState::Uncertain => OpStatus::Uncertain,
                AttemptState::Rejected => OpStatus::Rejected(last.detail.clone()),
            })
        })
        .collect()
}

/// The three locks, the seal and the approval scope; the ops the approvals cover.
pub fn check_locks(p: &Package, ctx: &Ctx<'_>) -> Result<(String, Vec<bool>), WriteError> {
    let hash = seal_of(p);
    if p.seal.as_deref() != Some(hash.as_str()) {
        return Err(err(
            "hash_mismatch",
            exit::NOT_APPROVED,
            "The package is not sealed, or it changed since it was sealed.",
        )
        .help("Seal it again (`ownpurse write seal`); a new hash needs a new approval."));
    }
    // the freeze gate again (plan enforces it; a hand-edited package can't slip past at run time)
    if let Some(o) = p.ops.iter().find(|o| o.gated.is_some())
        && p.accountant_decision.as_deref().is_none_or(str::is_empty)
    {
        return Err(err(
            "freeze_gate",
            exit::USAGE,
            format!(
                "{} is held by the freeze gate and the package names no accountant decision.",
                o.id
            ),
        ));
    }
    if p.profile != ctx.profile {
        return Err(err(
            "usage",
            exit::USAGE,
            format!(
                "The package was planned for profile '{}', not '{}'.",
                p.profile, ctx.profile
            ),
        ));
    }
    if !ctx.dry_run {
        if !ctx.allow_writes {
            return Err(err(
                "writes_disabled",
                exit::WRITES_DISABLED,
                format!("Profile '{}' does not set allow_writes.", ctx.profile),
            ));
        }
        if ctx.run_env.as_deref() != Some(hash.as_str()) {
            return Err(err(
                "writes_disabled",
                exit::WRITES_DISABLED,
                format!("{RUN_ENV} is not set to this package's hash for this invocation."),
            )
            .help(format!(
                "Run it as `{RUN_ENV}={hash} ownpurse write run …` (one invocation only)."
            )));
        }
    }
    let approvals = ctx.approvals.approvals(&hash)?;
    if approvals.is_empty() {
        return Err(err(
            "not_approved",
            exit::NOT_APPROVED,
            format!("No approval is recorded for {}.", &hash[..12]),
        )
        .help("The owner records it with `ownpurse write approve-record` (an agent never can)."));
    }
    let covered: Vec<bool> = p
        .ops
        .iter()
        .map(|o| approvals.iter().any(|a| a.scope.covers(&o.id)))
        .collect();
    Ok((hash, covered))
}

fn parse_body(resp: &Response, entity: &str) -> Option<Value> {
    resp.body.get(entity).and_then(|a| a.get(0)).cloned()
}

fn validation_errors(v: &Value) -> Vec<String> {
    let mut out = vec![];
    fn walk(v: &Value, out: &mut Vec<String>) {
        match v {
            Value::Object(m) => {
                if let Some(a) = m.get("ValidationErrors").and_then(Value::as_array) {
                    for e in a {
                        if let Some(s) = e["Message"].as_str() {
                            out.push(s.to_string());
                        }
                    }
                }
                for x in m.values() {
                    walk(x, out);
                }
            }
            Value::Array(a) => a.iter().for_each(|x| walk(x, out)),
            _ => {}
        }
    }
    walk(v, &mut out);
    out.sort();
    out.dedup();
    out
}

/// Xero's error answer in words: `PostDataInvalidException (14): Error converting value …`, then each
/// `Elements[].ValidationErrors[].Message`. Plain text bodies are kept (cut at 400 characters).
pub fn xero_errors(v: &Value) -> Vec<String> {
    let mut out = vec![];
    if let Some(m) = v.get("Message").and_then(Value::as_str) {
        let ty = v.get("Type").and_then(Value::as_str).unwrap_or("");
        let n = v
            .get("ErrorNumber")
            .map(|x| x.to_string())
            .filter(|x| x != "null");
        out.push(match (ty.is_empty(), n) {
            (false, Some(n)) => format!("{ty} ({n}): {m}"),
            (false, None) => format!("{ty}: {m}"),
            (true, Some(n)) => format!("({n}) {m}"),
            (true, None) => m.to_string(),
        });
    } else if let Some(t) = v.as_str().filter(|t| !t.is_empty()) {
        out.push(t.chars().take(400).collect());
    } else if let Some(d) = v.get("Detail").and_then(Value::as_str) {
        out.push(format!(
            "{}: {d}",
            v.get("Title").and_then(Value::as_str).unwrap_or("error")
        ));
    }
    for e in validation_errors(v) {
        if !out.contains(&e) {
            out.push(e);
        }
    }
    out
}

/// `{id}` filled in the path and anywhere in a JSON body (a void of a journal this package creates).
fn resolve_request(
    r: &ownpurse_core::write::WriteRequest,
    id: Option<&str>,
) -> ownpurse_core::write::WriteRequest {
    fn fill(v: &Value, id: &str) -> Value {
        match v {
            Value::String(s) => Value::String(s.replace("{id}", id)),
            Value::Array(a) => Value::Array(a.iter().map(|x| fill(x, id)).collect()),
            Value::Object(m) => {
                Value::Object(m.iter().map(|(k, x)| (k.clone(), fill(x, id))).collect())
            }
            other => other.clone(),
        }
    }
    let mut out = r.clone();
    if let Some(id) = id {
        out.path = r.path.replace("{id}", id);
        if let Body::Json { json } = &r.body {
            out.body = Body::Json {
                json: fill(json, id),
            };
        }
    }
    out
}

/// `{op:<id>}` filled anywhere in a JSON body with the document that earlier op created (a contact a bank
/// transaction names, a transaction a transfer names).
fn resolve_created(
    r: &ownpurse_core::write::WriteRequest,
    done: &dyn Fn(&str) -> Option<String>,
) -> ownpurse_core::write::WriteRequest {
    fn fill(v: &Value, done: &dyn Fn(&str) -> Option<String>) -> Value {
        match v {
            Value::String(s) if s.starts_with("{op:") && s.ends_with('}') => {
                match done(&s[4..s.len() - 1]) {
                    Some(id) => Value::String(id),
                    None => v.clone(),
                }
            }
            Value::Array(a) => Value::Array(a.iter().map(|x| fill(x, done)).collect()),
            Value::Object(m) => {
                Value::Object(m.iter().map(|(k, x)| (k.clone(), fill(x, done))).collect())
            }
            other => other.clone(),
        }
    }
    let mut out = r.clone();
    if let Body::Json { json } = &r.body {
        out.body = Body::Json {
            json: fill(json, done),
        };
    }
    out
}

/// A placeholder left in an outgoing request (`{id}`, an `op:` reference): never sent.
fn unresolved(r: &ownpurse_core::write::WriteRequest) -> Option<&'static str> {
    let body = match &r.body {
        Body::Json { json } => ownpurse_core::canon::canonical(json),
        Body::File { .. } => String::new(),
    };
    if r.path.contains("{id}") || body.contains("{id}") {
        Some("`{id}`")
    } else if r.path.contains("{op:") || body.contains("{op:") {
        Some("a `{op:…}` reference")
    } else if r.path.contains("op:") || body.contains("\"op:") {
        Some("an `op:` reference")
    } else {
        None
    }
}

fn get(t: &mut dyn WriteTransport, tenant: &str, path: &str) -> Result<Response, WriteError> {
    t.get(tenant, path).map_err(|e| transport_err(&e, false))
}

fn transport_err(e: &TransportError, sent: bool) -> WriteError {
    match e {
        TransportError::NotAllowed(m) => err("not_allowed", exit::USAGE, m.clone()),
        TransportError::Refused { code, message } => {
            let x = match code.as_str() {
                "writes_disabled" | "tenant_guard" | "not_loopback" => exit::WRITES_DISABLED,
                _ => exit::RUNTIME,
            };
            err(code, x, message.clone())
        }
        TransportError::NotSent(m) => err("network", exit::RUNTIME, m.clone()),
        TransportError::AfterSend(m) if sent => err("uncertain", exit::UNCERTAIN, m.clone()),
        TransportError::AfterSend(m) => err("network", exit::RUNTIME, m.clone()),
    }
}

/// A JSON number or numeric string as a decimal (exact digits; never through f64).
fn decimal(v: &Value) -> rust_decimal::Decimal {
    match v {
        Value::Number(n) => n.to_string().parse().unwrap_or_default(),
        Value::String(s) => s.parse().unwrap_or_default(),
        _ => rust_decimal::Decimal::ZERO,
    }
}

/// Append what Xero now holds for an operation's target to the record (source `readback`, `caused_by_op`
/// set), so the local record and the TUI follow a write without waiting for a sync. Unchanged content adds
/// nothing (the content hash, as in a sync). Returns whether a version was added.
fn record_target(
    t: &mut dyn WriteTransport,
    store: &Writer,
    op: &Operation,
    target_id: Option<&str>,
    op_key: &str,
) -> Result<bool, WriteError> {
    let Some(id) = target_id.filter(|i| !i.is_empty()) else {
        return Ok(false);
    };
    // a note: the document's history is what changed
    if let Op::AddNote(n) = &op.op {
        let r = get(t, &op.tenant_id, &format!("{}/{id}/History", n.entity))?;
        let Some(recs) = r.body["HistoryRecords"].as_array() else {
            return Ok(false);
        };
        return crate::history::record(store, &op.tenant_id, &n.entity, id, recs)
            .map_err(|e| err("record", exit::RUNTIME, e.message().to_string()));
    }
    let entity = op.op.entity();
    let r = get(t, &op.tenant_id, &format!("{entity}/{id}"))?;
    let Some(doc) = parse_body(&r, entity) else {
        return Ok(false);
    };
    let rec = |e: crate::ApiError| err("record", exit::RUNTIME, e.message().to_string());
    let mut known = store.latest_hashes(&op.tenant_id, entity).map_err(rec)?;
    let upd = doc["UpdatedDateUTC"]
        .as_str()
        .and_then(crate::sync::xero_datetime);
    store
        .record_version_by(
            &op.tenant_id,
            entity,
            id,
            &doc,
            doc["Status"].as_str(),
            upd.as_deref(),
            "readback",
            None,
            Some(op_key),
            &mut known,
        )
        .map_err(rec)
}

/// A JSON Patch as lines a person reads: `replace /LineItems/0/AccountCode: "652" → "6310"`.
fn patch_lines(p: &Value) -> Vec<String> {
    p.as_array()
        .into_iter()
        .flatten()
        .map(|o| {
            let path = o["path"].as_str().unwrap_or("");
            match o["op"].as_str().unwrap_or("") {
                "add" => format!("added {path} = {}", o["value"]),
                "remove" => format!("removed {path}"),
                op => format!("{op} {path} → {}", o["value"]),
            }
        })
        .collect()
}

/// URL-encoded `where` for a marker search.
pub fn marker_query(m: &str) -> String {
    let w = format!("Narration.Contains(\"{m}\")");
    let enc: String = w
        .bytes()
        .map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                (b as char).to_string()
            }
            _ => format!("%{b:02X}"),
        })
        .collect();
    format!("ManualJournals?where={enc}")
}

fn marker_hits(
    t: &mut dyn WriteTransport,
    tenant: &str,
    m: &str,
) -> Result<Vec<Value>, WriteError> {
    let r = get(t, tenant, &marker_query(m))?;
    Ok(r.body["ManualJournals"]
        .as_array()
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter(|j| j["Narration"].as_str().is_some_and(|n| n.contains(m)))
        .collect())
}

/// The bank transactions whose Reference carries a marker (GET `BankTransactions?where=Reference.Contains`).
fn reference_hits(
    t: &mut dyn WriteTransport,
    tenant: &str,
    m: &str,
) -> Result<Vec<Value>, WriteError> {
    reference_hits_in(t, tenant, "BankTransactions", m)
}

/// The documents of an entity (BankTransactions, BankTransfers) whose Reference carries a marker.
fn reference_hits_in(
    t: &mut dyn WriteTransport,
    tenant: &str,
    entity: &str,
    m: &str,
) -> Result<Vec<Value>, WriteError> {
    let r = get(t, tenant, &reference_query(entity, m))?;
    Ok(r.body[entity]
        .as_array()
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter(|b| b["Reference"].as_str().is_some_and(|n| n.contains(m)))
        .collect())
}

/// The marker search on an entity's Reference (`BankTransactions?where=Reference.Contains("…")`).
pub fn reference_query(entity: &str, m: &str) -> String {
    marker_query(m)
        .replacen("ManualJournals?", &format!("{entity}?"), 1)
        .replacen("Narration.Contains", "Reference.Contains", 1)
}

/// The pauses between marker searches when reconciling (Xero's search index can lag a fresh create):
/// `OWNPURSE_SEARCH_BACKOFF_MS` (`2000,3000,5000` by default: about 10 s).
fn search_backoff() -> Vec<u64> {
    std::env::var("OWNPURSE_SEARCH_BACKOFF_MS")
        .ok()
        .map(|v| v.split(',').filter_map(|x| x.trim().parse().ok()).collect())
        .unwrap_or_else(|| vec![2000, 3000, 5000])
}

fn search_tries() -> usize {
    search_backoff().len() + 1
}

/// A search retried with [`search_backoff`] until it finds something (never concludes "absent" from one
/// empty answer).
fn with_backoff(
    mut f: impl FnMut() -> Result<Vec<Value>, WriteError>,
) -> Result<Vec<Value>, WriteError> {
    let mut hits = f()?;
    for ms in search_backoff() {
        if !hits.is_empty() {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(ms));
        hits = f()?;
    }
    Ok(hits)
}

/// The contacts with exactly this name (GET `Contacts?where=Name=="…"`, then an exact compare).
fn contacts_named(
    t: &mut dyn WriteTransport,
    tenant: &str,
    name: &str,
) -> Result<Vec<Value>, WriteError> {
    let w = format!("Name==\"{}\"", name.replace('"', "\\\""));
    let enc: String = w
        .bytes()
        .map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                (b as char).to_string()
            }
            _ => format!("%{b:02X}"),
        })
        .collect();
    let r = get(t, tenant, &format!("Contacts?where={enc}"))?;
    Ok(r.body["Contacts"]
        .as_array()
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter(|c| c["Name"] == name)
        .collect())
}

/// Check one precondition live; `Err(why)` when it doesn't hold.
fn check_pre(
    t: &mut dyn WriteTransport,
    op: &Operation,
    pre: &Precondition,
    resolved: &dyn Fn(&str) -> String,
) -> Result<Result<(), (String, Value)>, WriteError> {
    let tenant = &op.tenant_id;
    let plain = |r: Result<(), String>| r.map_err(|w| (w, Value::Null));
    Ok(match pre {
        Precondition::Target {
            entity,
            id,
            fingerprint: fp,
            status,
            before,
            ..
        } => {
            let id = resolved(id);
            let r = get(t, tenant, &format!("{entity}/{id}"))?;
            match parse_body(&r, entity) {
                None => plain(Err(format!("{entity}/{id} not found (HTTP {})", r.status))),
                Some(doc) if doc["Status"].as_str() != Some(status.as_str()) => plain(Err(
                    format!("{entity}/{id} is {} (expected {status})", doc["Status"]),
                )),
                Some(doc) if fingerprint(&doc) != *fp => {
                    // what changed, as a JSON Patch from the planned copy to the live document
                    let live = ownpurse_core::write::business_view(&doc);
                    match before {
                        Some(b) => {
                            let diff = ownpurse_core::write::patch(b, &live);
                            let lines = patch_lines(&diff);
                            Err((
                                format!(
                                    "{entity}/{id} changed since the package was planned: {}",
                                    if lines.is_empty() {
                                        "(no field differs in the business view; the fingerprint rule changed: re-plan)".to_string()
                                    } else {
                                        lines.join("; ")
                                    }
                                ),
                                json!({"before": b, "live": live, "patch": diff}),
                            ))
                        }
                        None => Err((
                            format!(
                                "{entity}/{id} changed since the package was planned (the package holds no planned copy to diff: re-plan it)"
                            ),
                            json!({"live": live}),
                        )),
                    }
                }
                Some(_) => Ok(()),
            }
        }
        Precondition::MarkerAbsent { marker: m } => plain({
            let hits = marker_hits(t, tenant, m)?;
            if hits.is_empty() {
                Ok(())
            } else {
                Err(format!(
                    "{} journal(s) already carry the marker {m}",
                    hits.len()
                ))
            }
        }),
        Precondition::AccountsActive { codes } => plain({
            let r = get(t, tenant, "Accounts")?;
            let accts = r.body["Accounts"].as_array().cloned().unwrap_or_default();
            let mut bad = vec![];
            for c in codes {
                let found: Vec<&Value> = accts.iter().filter(|a| a["Code"] == c.as_str()).collect();
                if found.len() != 1 || found[0]["Status"] != "ACTIVE" {
                    bad.push(c.clone());
                }
            }
            if bad.is_empty() {
                Ok(())
            } else {
                Err(format!(
                    "accounts not unique and active: {}",
                    bad.join(", ")
                ))
            }
        }),
        Precondition::AttachmentAbsent { file_name, .. } => plain({
            // Xero lists names and sizes (not content hashes): a same-named file already there blocks it
            let id = resolved("{id}");
            let r = get(t, tenant, &format!("{}/{id}/Attachments", op.op.entity()))?;
            let there = r.body["Attachments"]
                .as_array()
                .into_iter()
                .flatten()
                .any(|a| a["FileName"] == file_name.as_str());
            if there {
                Err(format!("{file_name} is already attached to {id}"))
            } else {
                Ok(())
            }
        }),
        Precondition::ReferenceAbsent { marker: m, entity } => plain({
            let e = entity.as_deref().unwrap_or("BankTransactions");
            let hits = reference_hits_in(t, tenant, e, m)?
                .into_iter()
                .filter(|b| b["Status"] != "DELETED")
                .count();
            if hits == 0 {
                Ok(())
            } else {
                Err(format!("{hits} {e} already carry the marker {m}"))
            }
        }),
        Precondition::ContactNameAbsent { name } => plain({
            let hits = contacts_named(t, tenant, name)?;
            if hits.is_empty() {
                Ok(())
            } else {
                Err(format!(
                    "a contact named {name:?} exists ({})",
                    hits[0]["ContactID"].as_str().unwrap_or("")
                ))
            }
        }),
        Precondition::Exists { entity, id } => plain({
            let id = resolved(id);
            let r = get(t, tenant, &format!("{entity}/{id}"))?;
            if parse_body(&r, entity).is_some() {
                Ok(())
            } else {
                Err(format!("{entity}/{id} not found (HTTP {})", r.status))
            }
        }),
        Precondition::PeriodOpen { date } => plain({
            let r = get(t, tenant, "Organisation")?;
            let org = &r.body["Organisations"][0];
            let lock = ["PeriodLockDate", "EndOfYearLockDate"]
                .iter()
                .filter_map(|k| org[*k].as_str().and_then(ownpurse_core::dates::xero_date))
                .max();
            match lock {
                Some(l) if date.as_str() <= l.as_str() => {
                    Err(format!("{date} is on or before the lock date {l}"))
                }
                _ => Ok(()),
            }
        }),
    })
}

/// Read the operation's target back after a confirmed write: Ok(target id) or the discrepancy.
fn read_back(
    t: &mut dyn WriteTransport,
    pkg_id: &str,
    op: &Operation,
    target_id: Option<&str>,
    voided_later: bool,
) -> Result<Result<Option<String>, String>, WriteError> {
    let tenant = &op.tenant_id;
    Ok(match &op.op {
        Op::CreateJournal(j) => {
            // by the id the create answered with (Xero's search index can lag a fresh create); the marker
            // search only when the id isn't known
            let m = marker(pkg_id, &op.id);
            let hits = match target_id.filter(|i| !i.is_empty()) {
                Some(id) => vec![json!({ "ManualJournalID": id })],
                None => marker_hits(t, tenant, &m)?,
            };
            match hits.as_slice() {
                [one] => {
                    // a list GET (the marker search) carries no JournalLines in real Xero: read the journal
                    // itself, and compare what came back line for line with what was sent
                    let id = one["ManualJournalID"]
                        .as_str()
                        .unwrap_or_default()
                        .to_string();
                    let r = get(t, tenant, &format!("ManualJournals/{id}"))?;
                    let Some(full) = parse_body(&r, "ManualJournals") else {
                        return Ok(Err(format!(
                            "ManualJournals/{id} can't be read back (HTTP {})",
                            r.status
                        )));
                    };
                    let key = |code: &str, amt: rust_decimal::Decimal| {
                        format!("{code} {}", ownpurse_core::write::money_text(amt))
                    };
                    let mut got: Vec<String> = full["JournalLines"]
                        .as_array()
                        .into_iter()
                        .flatten()
                        .map(|l| {
                            key(
                                l["AccountCode"].as_str().unwrap_or(""),
                                decimal(&l["LineAmount"]),
                            )
                        })
                        .collect();
                    let mut sent: Vec<String> = j
                        .lines
                        .iter()
                        .map(|l| key(&l.account_code, l.amount))
                        .collect();
                    got.sort();
                    sent.sort();
                    if !full["Narration"].as_str().unwrap_or("").contains(&m) {
                        Err(format!("ManualJournals/{id} doesn't carry the marker {m}"))
                    } else if got != sent {
                        Err(format!(
                            "the journal's lines differ from what was sent: {got:?} vs {sent:?}"
                        ))
                    } else if full["Status"] != "POSTED"
                        && !(voided_later && full["Status"] == "VOIDED")
                    {
                        Err(format!("the journal is {}", full["Status"]))
                    } else {
                        Ok(Some(id))
                    }
                }
                [] => Err(format!(
                    "no journal carries the marker (GET {})",
                    marker_query(&m)
                )),
                more => Err(format!(
                    "{} journals carry the marker (a duplicate)",
                    more.len()
                )),
            }
        }
        Op::Attach(a) => {
            let id = target_id.unwrap_or_default();
            let r = get(t, tenant, &format!("{}/{id}/Attachments", a.entity))?;
            let n = r.body["Attachments"]
                .as_array()
                .into_iter()
                .flatten()
                .filter(|x| {
                    x["FileName"] == a.file_name.as_str()
                        && x["ContentLength"].as_u64() == Some(a.bytes)
                })
                .count();
            if n >= 1 {
                Ok(Some(id.to_string()))
            } else {
                Err(format!("{} is not attached", a.file_name))
            }
        }
        Op::CreateBankTransaction(b) => {
            let m = marker(pkg_id, &op.id);
            let found = match target_id.filter(|i| !i.is_empty()) {
                Some(id) => vec![json!({ "BankTransactionID": id })],
                None => reference_hits(t, tenant, &m)?,
            };
            let hits: Vec<Value> = found
                .into_iter()
                .filter(|x| x["Status"] != "DELETED" || voided_later)
                .collect();
            match hits.as_slice() {
                [one] => {
                    let id = one["BankTransactionID"]
                        .as_str()
                        .unwrap_or_default()
                        .to_string();
                    let r = get(t, tenant, &format!("BankTransactions/{id}"))?;
                    let Some(full) = parse_body(&r, "BankTransactions") else {
                        return Ok(Err(format!(
                            "BankTransactions/{id} can't be read back (HTTP {})",
                            r.status
                        )));
                    };
                    let key = |code: &str, amt: rust_decimal::Decimal| {
                        format!("{code} {}", ownpurse_core::write::money_text(amt))
                    };
                    let mut got: Vec<String> = full["LineItems"]
                        .as_array()
                        .into_iter()
                        .flatten()
                        .map(|l| {
                            key(
                                l["AccountCode"].as_str().unwrap_or(""),
                                decimal(&l["LineAmount"]),
                            )
                        })
                        .collect();
                    let mut sent: Vec<String> = b
                        .lines
                        .iter()
                        .map(|l| key(&l.account_code, l.amount))
                        .collect();
                    got.sort();
                    sent.sort();
                    if !full["Reference"].as_str().unwrap_or("").contains(&m) {
                        Err(format!(
                            "BankTransactions/{id} doesn't carry the marker {m}"
                        ))
                    } else if got != sent {
                        Err(format!(
                            "the transaction's lines differ from what was sent: {got:?} vs {sent:?}"
                        ))
                    } else if full["Type"] != b.kind.as_str() {
                        Err(format!(
                            "the transaction is {} (sent {})",
                            full["Type"], b.kind
                        ))
                    } else if full["Status"] != "AUTHORISED"
                        && !(voided_later && full["Status"] == "DELETED")
                    {
                        Err(format!("the transaction is {}", full["Status"]))
                    } else {
                        Ok(Some(id))
                    }
                }
                [] => Err(format!(
                    "no bank transaction carries the marker (GET {})",
                    reference_query("BankTransactions", &m)
                )),
                more => Err(format!(
                    "{} bank transactions carry the marker (a duplicate)",
                    more.len()
                )),
            }
        }
        Op::CreateBankTransfer(tr) => {
            let m = marker(pkg_id, &op.id);
            let hits = match target_id.filter(|i| !i.is_empty()) {
                Some(id) => vec![json!({ "BankTransferID": id })],
                None => reference_hits_in(t, tenant, "BankTransfers", &m)?,
            };
            match hits.as_slice() {
                [one] => {
                    let id = one["BankTransferID"]
                        .as_str()
                        .unwrap_or_default()
                        .to_string();
                    let r = get(t, tenant, &format!("BankTransfers/{id}"))?;
                    match parse_body(&r, "BankTransfers") {
                        None => Err(format!(
                            "BankTransfers/{id} can't be read back (HTTP {})",
                            r.status
                        )),
                        Some(d) if !d["Reference"].as_str().unwrap_or("").contains(&m) => {
                            Err(format!("BankTransfers/{id} doesn't carry the marker {m}"))
                        }
                        Some(d) if decimal(&d["Amount"]) != tr.amount => Err(format!(
                            "the transfer is {} (sent {})",
                            d["Amount"],
                            ownpurse_core::write::money_text(tr.amount)
                        )),
                        Some(d)
                            if d["FromBankAccount"]["AccountID"] != tr.from_account_id.as_str()
                                || d["ToBankAccount"]["AccountID"] != tr.to_account_id.as_str() =>
                        {
                            Err("the transfer's accounts differ from what was sent".into())
                        }
                        Some(_) => Ok(Some(id)),
                    }
                }
                [] => Err(format!(
                    "no bank transfer carries the marker (GET {})",
                    reference_query("BankTransfers", &m)
                )),
                more => Err(format!(
                    "{} bank transfers carry the marker (a duplicate)",
                    more.len()
                )),
            }
        }
        Op::CreateContact(c) => {
            let hits = match target_id.filter(|i| !i.is_empty()) {
                Some(id) => {
                    let r = get(t, tenant, &format!("Contacts/{id}"))?;
                    parse_body(&r, "Contacts")
                        .filter(|d| d["Name"] == c.name.as_str())
                        .into_iter()
                        .collect()
                }
                None => contacts_named(t, tenant, &c.name)?,
            };
            match hits.as_slice() {
                [one] if one["ContactStatus"] != "ARCHIVED" || voided_later => {
                    Ok(one["ContactID"].as_str().map(str::to_string))
                }
                [one] => Err(format!("the contact is {}", one["ContactStatus"])),
                [] => Err(format!("no contact is named {:?}", c.name)),
                more => Err(format!("{} contacts are named {:?}", more.len(), c.name)),
            }
        }
        Op::ArchiveContact { .. } => {
            let id = target_id.unwrap_or_default();
            let r = get(t, tenant, &format!("Contacts/{id}"))?;
            match parse_body(&r, "Contacts") {
                None => Err(format!("Contacts/{id} can't be read back")),
                Some(d) if d["ContactStatus"] != "ARCHIVED" => {
                    Err(format!("Contacts/{id} is {}", d["ContactStatus"]))
                }
                Some(_) => Ok(Some(id.to_string())),
            }
        }
        Op::VoidBankTransaction { .. } => {
            let id = target_id.unwrap_or_default();
            let r = get(t, tenant, &format!("BankTransactions/{id}"))?;
            match parse_body(&r, "BankTransactions") {
                None => Err(format!("BankTransactions/{id} can't be read back")),
                Some(doc) if doc["Status"] != "DELETED" => {
                    Err(format!("BankTransactions/{id} is {}", doc["Status"]))
                }
                Some(_) => Ok(Some(id.to_string())),
            }
        }
        Op::AddNote(n) => {
            let id = target_id.unwrap_or_default();
            match note_landed(t, tenant, &n.entity, id, &n.details)? {
                true => Ok(Some(id.to_string())),
                false => Err(format!("the note is not in {}/{id}'s history", n.entity)),
            }
        }
        Op::VoidJournal { .. } | Op::RecodeBankLine(_) => {
            let entity = op.op.entity();
            let id = target_id.unwrap_or_default();
            let r = get(t, tenant, &format!("{entity}/{id}"))?;
            match (parse_body(&r, entity), &op.expected_after) {
                (None, _) => Err(format!("{entity}/{id} can't be read back")),
                (Some(doc), Some(want)) if fingerprint(&doc) != *want => {
                    Err(format!("{entity}/{id} differs from the expected result"))
                }
                (Some(doc), None)
                    if matches!(op.op, Op::VoidJournal { .. }) && doc["Status"] != "VOIDED" =>
                {
                    Err(format!("{entity}/{id} is {}", doc["Status"]))
                }
                (Some(_), _) => Ok(Some(id.to_string())),
            }
        }
    })
}

/// The Xero id an operation targets (before sending), resolving `CreatedBy` through earlier results.
fn target_of(op: &Operation, done: &dyn Fn(&str) -> Option<String>) -> Option<String> {
    match &op.op {
        Op::CreateJournal(_) => None,
        Op::RecodeBankLine(r) => Some(r.bank_transaction_id.clone()),
        Op::VoidJournal { journal } => resolve(journal, done),
        Op::Attach(a) => resolve(&a.journal, done),
        Op::AddNote(n) => resolve(&n.target, done),
        Op::CreateBankTransaction(_) | Op::CreateBankTransfer(_) | Op::CreateContact(_) => None,
        Op::VoidBankTransaction { bank_transaction } => resolve(bank_transaction, done),
        Op::ArchiveContact { contact } => resolve(contact, done),
    }
}

/// The entity an operation's audit note goes on (the document it wrote).
pub fn note_entity(o: &Operation) -> &'static str {
    o.op.entity()
}

/// Add `ownpurse package <hash12> op <id>: <why>` to the history of the document an op wrote (its own
/// Idempotency-Key), read the history back into the record, and say how it went. Never an error: the op
/// itself has landed; a missing note is reported, not fatal.
fn add_audit_note(
    t: &mut dyn WriteTransport,
    store: &Writer,
    hash: &str,
    op: &Operation,
    id: &str,
    why: &str,
) -> Value {
    let entity = note_entity(op);
    let text = ownpurse_core::write::audit_note(hash, &op.id, why);
    let request =
        ownpurse_core::write::ops::note_request(entity, &JournalRef::Id(id.into()), &text);
    let idem = idempotency_key(hash, &format!("{}:note", op.id));
    let sent = t.send(&Send {
        tenant_id: &op.tenant_id,
        request: &request,
        path: request.path.clone(),
        idempotency_key: &idem,
        file: None,
    });
    match sent {
        Ok(r) if r.status < 300 => {
            // the history as Xero now holds it, into the record (the TUI's document view shows it)
            if let Ok(h) = get(t, &op.tenant_id, &format!("{entity}/{id}/History"))
                && let Some(recs) = h.body["HistoryRecords"].as_array()
            {
                let _ = crate::history::record(store, &op.tenant_id, entity, id, recs);
            }
            json!({"ok": true, "details": text})
        }
        Ok(r) => {
            json!({"ok": false, "why": format!("HTTP {}: {}", r.status, xero_errors(&r.body).join("; "))})
        }
        Err(e) => json!({"ok": false, "why": format!("{e:?}")}),
    }
}

/// Whether the document's history holds a note with exactly these details (GET `{Entity}/{id}/History`).
fn note_landed(
    t: &mut dyn WriteTransport,
    tenant: &str,
    entity: &str,
    id: &str,
    details: &str,
) -> Result<bool, WriteError> {
    let r = get(t, tenant, &format!("{entity}/{id}/History"))?;
    Ok(r.body["HistoryRecords"]
        .as_array()
        .into_iter()
        .flatten()
        .any(|h| {
            h["Details"]
                .as_str()
                .map(ownpurse_core::text::html_unescape)
                .as_deref()
                == Some(details)
        }))
}

fn resolve(j: &JournalRef, done: &dyn Fn(&str) -> Option<String>) -> Option<String> {
    match j {
        JournalRef::Id(id) => Some(id.clone()),
        JournalRef::CreatedBy(op) => done(op),
    }
}

fn store_report(
    store: &Writer,
    tenant: &str,
    params: &Value,
    body: &Value,
) -> Result<(), WriteError> {
    let payload = ownpurse_core::canon::canonical(body);
    store
        .append(
            "reports",
            &[
                ("tenant_id", Sql::Text(tenant.into())),
                ("report", Sql::Text("TrialBalance".into())),
                ("params", Sql::Text(ownpurse_core::canon::canonical(params))),
                ("fetched_at", Sql::Text(crate::store::now_iso())),
                ("content_hash", Sql::Text(sha256_hex(payload.as_bytes()))),
                ("payload", Sql::Text(payload)),
            ],
        )
        .map_err(|e| err("record", exit::RUNTIME, e.message().to_string()))?;
    Ok(())
}

/// The TrialBalance dates a package's deltas are checked at (the latest op date per tenant).
/// The TrialBalance date each tenant's deltas are checked at: today, or the latest dated effect of the
/// package's operations when that is later (a journal's date; a dependent op takes the date of the journal it
/// acts on; otherwise the op's `PeriodOpen` date). Before and after use the same date, so a later date only
/// adds documents both sides hold. Never a sentinel date.
fn tb_dates(p: &Package) -> Vec<(String, String)> {
    let today = crate::store::now_iso()[..10].to_string();
    let date_of = |o: &Operation| -> Option<String> {
        match &o.op {
            Op::CreateJournal(j) => Some(j.date.clone()),
            Op::VoidJournal {
                journal: JournalRef::CreatedBy(c),
            }
            | Op::Attach(ownpurse_core::write::Attachment {
                journal: JournalRef::CreatedBy(c),
                ..
            }) => p.ops.iter().find(|x| x.id == *c).and_then(|x| match &x.op {
                Op::CreateJournal(j) => Some(j.date.clone()),
                _ => None,
            }),
            _ => o.preconditions.iter().find_map(|pre| match pre {
                Precondition::PeriodOpen { date } => Some(date.clone()),
                _ => None,
            }),
        }
    };
    let mut out: std::collections::BTreeMap<String, String> = Default::default();
    for o in &p.ops {
        let e = out
            .entry(o.tenant_id.clone())
            .or_insert_with(|| today.clone());
        if let Some(d) = date_of(o).filter(|d| ownpurse_core::dates::parse_iso(d).is_some())
            && d > *e
        {
            *e = d;
        }
    }
    out.into_iter().collect()
}

fn tb_path(date: &str) -> String {
    format!("Reports/TrialBalance?date={date}")
}

/// `write run`: the next batch of pending operations.
pub fn run(p: &Package, ctx: &mut Ctx<'_>) -> Result<RunReport, WriteError> {
    let (hash, covered) = check_locks(p, ctx)?;
    let mut rep = RunReport {
        hash: hash.clone(),
        ..Default::default()
    };
    let status = statuses(ctx.store, &hash, p)?;
    if let Some(i) = status.iter().position(|s| *s == OpStatus::Uncertain) {
        return Err(err(
            "uncertain",
            exit::UNCERTAIN,
            format!(
                "Operation {} has an uncertain attempt; nothing is sent until it is reconciled.",
                p.ops[i].id
            ),
        )
        .help(format!(
            "Run `ownpurse write reconcile <package> {}` (a read-back decides).",
            p.ops[i].id
        )));
    }
    if !ctx.dry_run {
        let granted = ctx.transport.scopes();
        // each op's scope, and the scope of its audit note (the entity's own)
        let missing: Vec<&str> = p
            .ops
            .iter()
            .flat_map(|o| {
                let note = o.history_note.as_ref().map(|_| note_scope(note_entity(o)));
                std::iter::once(scope_for(&o.op)).chain(note)
            })
            .filter(|s| !granted.iter().any(|g| g == s))
            .collect();
        if let Some(s) = missing.first() {
            return Err(err(
                "writes_disabled",
                exit::WRITES_DISABLED,
                format!("The token lacks the scope {s}."),
            )
            .help("The owner grants them with `ownpurse login --write` (a browser consent); a run never asks."));
        }
    }
    let mut results: std::collections::BTreeMap<String, Option<String>> = Default::default();
    for (o, s) in p.ops.iter().zip(&status) {
        if let OpStatus::Done(id) = s {
            results.insert(o.id.clone(), id.clone());
        }
    }
    let pending: Vec<usize> = (0..p.ops.len())
        .filter(|&i| matches!(status[i], OpStatus::Pending) && covered[i])
        .collect();
    if let Some(i) = (0..p.ops.len()).find(|&i| matches!(status[i], OpStatus::Rejected(_))) {
        return Err(err(
            "rejected",
            exit::PRECONDITION,
            format!(
                "Operation {} was refused earlier; this package can't continue.",
                p.ops[i].id
            ),
        )
        .help("Plan an amended package (a new hash, a new approval)."));
    }
    let batch = ctx.batch.clamp(1, BATCH_MAX);
    rep.remaining = pending.len().saturating_sub(batch);
    let log = OpLog { store: ctx.store };
    // the package as the record knows it (seal registers it; this is the backstop)
    if !ctx.dry_run {
        super::log::register_package(ctx.store, p, &hash)?;
    }
    // the evidence before the first write: each tenant's TrialBalance at the package's date
    if !ctx.dry_run && !pending.is_empty() && results.is_empty() {
        for (tenant, date) in tb_dates(p) {
            let r = get(ctx.transport, &tenant, &tb_path(&date))?;
            store_report(
                ctx.store,
                &tenant,
                &json!({"date": date, "xa_package": hash, "phase": "before"}),
                &r.body,
            )?;
        }
    }
    let mut dry_planned: Vec<String> = vec![];
    for &i in pending.iter().take(batch) {
        let op = &p.ops[i];
        // what it waits for: its listed dependencies and the journal an earlier op creates
        let mut deps: Vec<String> = op.depends_on.clone();
        if let Op::VoidJournal {
            journal: JournalRef::CreatedBy(d),
        }
        | Op::Attach(ownpurse_core::write::Attachment {
            journal: JournalRef::CreatedBy(d),
            ..
        })
        | Op::VoidBankTransaction {
            bank_transaction: JournalRef::CreatedBy(d),
        }
        | Op::ArchiveContact {
            contact: JournalRef::CreatedBy(d),
        } = &op.op
        {
            deps.push(d.clone());
        }
        deps.sort();
        deps.dedup();
        let blocked: Vec<&String> = deps.iter().filter(|d| !results.contains_key(*d)).collect();
        if ctx.dry_run && !blocked.is_empty() && blocked.iter().all(|d| dry_planned.contains(*d)) {
            // in a dry run the earlier op didn't really run: show it, unchecked
            rep.ops.push((
                op.id.clone(),
                "after_dependency".into(),
                json!({"method": op.request.method, "path": op.request.path, "waits_for": blocked, "expected": op.expected}),
            ));
            dry_planned.push(op.id.clone());
            continue;
        }
        if !blocked.is_empty() {
            rep.stopped = Some(err(
                "blocked",
                exit::PRECONDITION,
                format!("{} waits for {:?}", op.id, blocked),
            ));
            break;
        }
        let done = |id: &str| results.get(id).cloned().flatten();
        let target = target_of(op, &done);
        let resolved = |id: &str| {
            if id == "{id}" {
                target.clone().unwrap_or_default()
            } else {
                id.to_string()
            }
        };
        let key = op_key(&hash, &op.id);
        // live preconditions
        let mut failed = None;
        for pre in &op.preconditions {
            if let Err(f) = check_pre(ctx.transport, op, pre, &resolved)? {
                failed = Some(f);
                break;
            }
        }
        if ctx.dry_run {
            rep.ops.push((
                op.id.clone(),
                if failed.is_some() { "precondition_failed" } else { "would_send" }.into(),
                json!({"method": op.request.method, "path": resolve_request(&op.request, target.as_deref()).path,
                       "body": resolve_request(&op.request, target.as_deref()).body, "history_note": op.history_note, "precondition": failed.as_ref().map(|f| &f.0),
                       "diff": failed.as_ref().map(|f| &f.1).filter(|d| !d.is_null()), "expected": op.expected}),
            ));
            if failed.is_some() {
                break;
            }
            dry_planned.push(op.id.clone());
            continue;
        }
        log.register(&hash, op, target.as_deref())?;
        if let Some((why, diff)) = failed {
            // the live document and the diff are evidence in the attempt log
            log.event(
                &key,
                AttemptState::Rejected,
                None,
                None,
                json!({"stage": "precondition", "reason": why, "diff": diff}),
            )?;
            rep.ops.push((
                op.id.clone(),
                "precondition_failed".into(),
                json!({"reason": why, "diff": diff}),
            ));
            rep.stopped = Some(err(
                "precondition_failed",
                exit::PRECONDITION,
                format!("{}: {why}", op.id),
            ));
            break;
        }
        // the request as it goes out: `{id}` filled everywhere (path AND body), from the journal it acts on,
        // and `{op:<id>}` from the documents earlier ops created
        let request = resolve_created(&resolve_request(&op.request, target.as_deref()), &done);
        let path = request.path.clone();
        if let Some(left) = unresolved(&request) {
            log.event(
                &key,
                AttemptState::Rejected,
                None,
                None,
                json!({"stage": "request", "reason": format!("unresolved {left} in the request")}),
            )?;
            rep.stopped = Some(err(
                "precondition_failed",
                exit::PRECONDITION,
                format!(
                    "{}: the request still holds {left} (its journal isn't known); nothing was sent",
                    op.id
                ),
            ));
            break;
        }
        let file = match &op.op {
            Op::Attach(a) => {
                let p = Path::new(&a.path);
                // absolute; else as planned (relative to where plan ran); else beside the package
                let p = if p.is_absolute() || p.exists() {
                    p.to_path_buf()
                } else {
                    ctx.base_dir.join(p)
                };
                match std::fs::read(&p) {
                    Ok(b) => Some(b),
                    Err(e) => {
                        log.event(
                            &key,
                            AttemptState::Rejected,
                            None,
                            None,
                            json!({"stage": "file", "reason": e.to_string()}),
                        )?;
                        rep.stopped = Some(err(
                            "precondition_failed",
                            exit::PRECONDITION,
                            format!("{}: {}: {e}", op.id, a.path),
                        ));
                        break;
                    }
                }
            }
            _ => None,
        };
        if let (Some(b), Body::File { sha256, .. }) = (&file, &op.request.body)
            && sha256_hex(b) != *sha256
        {
            log.event(
                &key,
                AttemptState::Rejected,
                None,
                None,
                json!({"stage": "file", "reason": "the file changed since sealing"}),
            )?;
            rep.stopped = Some(err(
                "precondition_failed",
                exit::PRECONDITION,
                format!("{}: the attachment changed since sealing", op.id),
            ));
            break;
        }
        let idem = idempotency_key(&hash, &op.id);
        // durably BEFORE the request
        log.event(
            &key,
            AttemptState::Sending,
            Some(&idem),
            None,
            json!({"path": path}),
        )?;
        let sent = ctx.transport.send(&Send {
            tenant_id: &op.tenant_id,
            request: &request,
            path: path.clone(),
            idempotency_key: &idem,
            file: file.as_deref(),
        });
        let resp = match sent {
            Ok(r) => r,
            Err(e @ TransportError::AfterSend(_)) => {
                log.event(
                    &key,
                    AttemptState::Uncertain,
                    Some(&idem),
                    None,
                    json!({"reason": format!("{e:?}")}),
                )?;
                rep.ops
                    .push((op.id.clone(), "uncertain".into(), json!(format!("{e:?}"))));
                rep.stopped = Some(transport_err(&e, true));
                break;
            }
            Err(e) => {
                log.event(
                    &key,
                    AttemptState::Rejected,
                    Some(&idem),
                    None,
                    json!({"stage": "transport", "reason": format!("{e:?}")}),
                )?;
                rep.ops
                    .push((op.id.clone(), "not_sent".into(), json!(format!("{e:?}"))));
                rep.stopped = Some(transport_err(&e, false));
                break;
            }
        };
        let errors = validation_errors(&resp.body);
        if resp.status >= 500 || (resp.status < 300 && !errors.is_empty()) {
            log.event(
                &key,
                AttemptState::Uncertain,
                Some(&idem),
                Some(resp.status),
                json!({"response": resp.body}),
            )?;
            rep.ops.push((
                op.id.clone(),
                "uncertain".into(),
                json!({"status": resp.status, "errors": errors}),
            ));
            rep.stopped = Some(
                err(
                    "uncertain",
                    exit::UNCERTAIN,
                    format!(
                        "{}: HTTP {}{}",
                        op.id,
                        resp.status,
                        if errors.is_empty() {
                            String::new()
                        } else {
                            format!(" with {}", errors.join("; "))
                        }
                    ),
                )
                .help(format!(
                    "Run `ownpurse write reconcile <package> {}` before anything else.",
                    op.id
                )),
            );
            break;
        }
        if resp.status >= 300 {
            // Xero's error: its Type, ErrorNumber and Message, then every validation error
            let errors = xero_errors(&resp.body);
            log.event(
                &key,
                AttemptState::Rejected,
                Some(&idem),
                Some(resp.status),
                json!({"stage": "xero", "errors": errors, "response": resp.body}),
            )?;
            rep.ops.push((
                op.id.clone(),
                "rejected".into(),
                json!({"status": resp.status, "errors": errors}),
            ));
            rep.stopped = Some(err(
                "rejected",
                exit::RUNTIME,
                format!(
                    "{}: Xero refused it (HTTP {}): {}",
                    op.id,
                    resp.status,
                    errors.join("; ")
                ),
            ));
            break;
        }
        // the target's id from the answer (a create), else the one we wrote to
        let new_id = match &op.op {
            Op::CreateJournal(_) => parse_body(&resp, "ManualJournals")
                .and_then(|j| j["ManualJournalID"].as_str().map(str::to_string)),
            Op::CreateBankTransaction(_) => parse_body(&resp, "BankTransactions")
                .and_then(|j| j["BankTransactionID"].as_str().map(str::to_string)),
            Op::CreateBankTransfer(_) => parse_body(&resp, "BankTransfers")
                .and_then(|j| j["BankTransferID"].as_str().map(str::to_string)),
            Op::CreateContact(_) => parse_body(&resp, "Contacts")
                .and_then(|j| j["ContactID"].as_str().map(str::to_string)),
            _ => target.clone(),
        };
        let back = read_back(ctx.transport, &p.id, op, new_id.as_deref(), false)?;
        let ok = back.is_ok();
        // the audit trail inside Xero: the note on the document it wrote (never blocks the op: it landed)
        let note = match (&op.history_note, new_id.as_deref()) {
            (Some(why), Some(id)) if ok => {
                Some(add_audit_note(ctx.transport, ctx.store, &hash, op, id, why))
            }
            (Some(_), _) => {
                Some(json!({"ok": false, "why": "not added: the write wasn't read back"}))
            }
            _ => None,
        };
        log.event(&key, AttemptState::Confirmed, Some(&idem), Some(resp.status),
            json!({"target_id": new_id, "readback": match &back { Ok(_) => json!({"ok": true}), Err(w) => json!({"ok": false, "why": w}) }, "note": note}))?;
        if ok {
            record_target(ctx.transport, ctx.store, op, new_id.as_deref(), &key)?;
        }
        results.insert(op.id.clone(), new_id.clone());
        rep.ops.push((
            op.id.clone(),
            "confirmed".into(),
            json!({"target_id": new_id, "readback_ok": ok}),
        ));
        if let Err(why) = back {
            rep.stopped = Some(err("discrepancy", exit::DISCREPANCY, format!("{}: {why}", op.id))
                .help("Stopped: dependent operations are blocked. Inspect with `ownpurse write verify`."));
            break;
        }
    }
    Ok(rep)
}

/// `write reconcile <op>`: settle an uncertain attempt by read-back, never by re-sending.
pub fn reconcile(p: &Package, op_id: &str, ctx: &mut Ctx<'_>) -> Result<Value, WriteError> {
    let hash = seal_of(p);
    let op = p.ops.iter().find(|o| o.id == op_id).ok_or_else(|| {
        err(
            "usage",
            exit::USAGE,
            format!("No operation {op_id} in the package."),
        )
    })?;
    let log = OpLog { store: ctx.store };
    let key = op_key(&hash, op_id);
    let last = log.last(&key)?;
    if !matches!(
        last.as_ref().map(|a| a.state),
        Some(AttemptState::Uncertain | AttemptState::Sending)
    ) {
        return Err(err(
            "usage",
            exit::USAGE,
            format!("{op_id} has no uncertain attempt to reconcile."),
        ));
    }
    let status = statuses(ctx.store, &hash, p)?;
    let results: std::collections::BTreeMap<String, Option<String>> = p
        .ops
        .iter()
        .zip(&status)
        .filter_map(|(o, s)| match s {
            OpStatus::Done(id) => Some((o.id.clone(), id.clone())),
            _ => None,
        })
        .collect();
    let done = |id: &str| results.get(id).cloned().flatten();
    let target = target_of(op, &done);
    let (landed, detail) = match &op.op {
        Op::CreateJournal(_) => {
            let hits =
                with_backoff(|| marker_hits(ctx.transport, &op.tenant_id, &marker(&p.id, &op.id)))?;
            match hits.len() {
                0 => (
                    false,
                    json!({"landed": false, "evidence": format!("no journal carries the marker (searched {} times: GET {})", search_tries(), marker_query(&marker(&p.id, &op.id)))}),
                ),
                1 => (
                    true,
                    json!({"landed": true, "target_id": hits[0]["ManualJournalID"], "evidence": "one journal carries the marker"}),
                ),
                n => {
                    log.event(&key, AttemptState::Reconciled, None, None,
                        json!({"landed": true, "duplicates": n, "target_id": hits[0]["ManualJournalID"]}))?;
                    return Err(err(
                        "discrepancy",
                        exit::DISCREPANCY,
                        format!("{n} journals carry the marker of {op_id}: a duplicate landed."),
                    )
                    .help("Plan a recovery package for the extra journal(s)."));
                }
            }
        }
        Op::CreateBankTransfer(_) => {
            let hits = with_backoff(|| {
                reference_hits_in(
                    ctx.transport,
                    &op.tenant_id,
                    "BankTransfers",
                    &marker(&p.id, &op.id),
                )
            })?;
            match hits.len() {
                0 => (
                    false,
                    json!({"landed": false, "evidence": format!("no bank transfer carries the marker (searched {} times: GET {})", search_tries(), reference_query("BankTransfers", &marker(&p.id, &op.id)))}),
                ),
                1 => (
                    true,
                    json!({"landed": true, "target_id": hits[0]["BankTransferID"], "evidence": "one bank transfer carries the marker"}),
                ),
                n => {
                    log.event(&key, AttemptState::Reconciled, None, None,
                        json!({"landed": true, "duplicates": n, "target_id": hits[0]["BankTransferID"]}))?;
                    return Err(err(
                        "discrepancy",
                        exit::DISCREPANCY,
                        format!(
                            "{n} bank transfers carry the marker of {op_id}: a duplicate landed."
                        ),
                    )
                    .help("Plan a recovery package (a reverse transfer) for the extra one(s)."));
                }
            }
        }
        Op::CreateContact(c) => {
            let hits = with_backoff(|| contacts_named(ctx.transport, &op.tenant_id, &c.name))?;
            match hits.first() {
                None => (
                    false,
                    json!({"landed": false, "evidence": "no contact has the name"}),
                ),
                Some(h) => (
                    true,
                    json!({"landed": true, "target_id": h["ContactID"], "evidence": "a contact has the name"}),
                ),
            }
        }
        Op::CreateBankTransaction(_) => {
            let hits: Vec<Value> = with_backoff(|| {
                reference_hits(ctx.transport, &op.tenant_id, &marker(&p.id, &op.id))
            })?
            .into_iter()
            .filter(|b| b["Status"] != "DELETED")
            .collect();
            match hits.len() {
                0 => (
                    false,
                    json!({"landed": false, "evidence": format!("no bank transaction carries the marker (searched {} times: GET {})", search_tries(), reference_query("BankTransactions", &marker(&p.id, &op.id)))}),
                ),
                1 => (
                    true,
                    json!({"landed": true, "target_id": hits[0]["BankTransactionID"], "evidence": "one bank transaction carries the marker"}),
                ),
                n => {
                    log.event(&key, AttemptState::Reconciled, None, None,
                        json!({"landed": true, "duplicates": n, "target_id": hits[0]["BankTransactionID"]}))?;
                    return Err(err(
                        "discrepancy",
                        exit::DISCREPANCY,
                        format!(
                            "{n} bank transactions carry the marker of {op_id}: a duplicate landed."
                        ),
                    )
                    .help("Plan a recovery package (void) for the extra transaction(s)."));
                }
            }
        }
        _ => match read_back(ctx.transport, &p.id, op, target.as_deref(), false)? {
            Ok(id) => (
                true,
                json!({"landed": true, "target_id": id, "evidence": "the read-back matches the expected result"}),
            ),
            Err(why) => {
                // unchanged since planning means it did not land
                let mut unchanged = true;
                for pre in &op.preconditions {
                    if let Precondition::Target { .. } = pre {
                        let r = check_pre(ctx.transport, op, pre, &|id: &str| {
                            if id == "{id}" {
                                target.clone().unwrap_or_default()
                            } else {
                                id.to_string()
                            }
                        })?;
                        unchanged &= r.is_ok();
                    }
                }
                if unchanged
                    && matches!(
                        op.op,
                        Op::RecodeBankLine(_)
                            | Op::VoidJournal { .. }
                            | Op::VoidBankTransaction { .. }
                    )
                {
                    (
                        false,
                        json!({"landed": false, "evidence": "the target is unchanged since planning"}),
                    )
                } else if matches!(op.op, Op::Attach(_)) {
                    (false, json!({"landed": false, "evidence": why}))
                } else {
                    return Err(err(
                        "discrepancy",
                        exit::DISCREPANCY,
                        format!("{op_id}: neither landed nor unchanged ({why})."),
                    )
                    .help("Inspect the target in Xero; plan a recovery package if needed."));
                }
            }
        },
    };
    log.event(&key, AttemptState::Reconciled, None, None, detail.clone())?;
    Ok(json!({"op": op_id, "landed": landed, "detail": detail}))
}

/// `write verify`: re-read each landed target, then the TrialBalance movement against the expected deltas
/// (every account not listed must not move), per tenant; plus a warning when no lock date is set.
pub fn verify(p: &Package, ctx: &mut Ctx<'_>) -> Result<Value, WriteError> {
    let hash = seal_of(p);
    let status = statuses(ctx.store, &hash, p)?;
    let mut targets = vec![];
    let mut problems = vec![];
    let mut done_ops = vec![];
    for (op, s) in p.ops.iter().zip(&status) {
        match s {
            OpStatus::Done(id) => {
                done_ops.push(op);
                // a journal this package creates and later voids reads back VOIDED
                let voided_later = p.ops.iter().zip(&status).any(|(o, s)| {
                    matches!(s, OpStatus::Done(_))
                        && matches!(&o.op, Op::VoidJournal { journal: JournalRef::CreatedBy(c) }
                            | Op::VoidBankTransaction { bank_transaction: JournalRef::CreatedBy(c) }
                            | Op::ArchiveContact { contact: JournalRef::CreatedBy(c) } if *c == op.id)
                });
                match read_back(ctx.transport, &p.id, op, id.as_deref(), voided_later)? {
                    Ok(_) => {
                        let added = record_target(
                            ctx.transport,
                            ctx.store,
                            op,
                            id.as_deref(),
                            &op_key(&hash, &op.id),
                        )?;
                        // `recorded`: the record holds what Xero has for this op (a read-back version, from
                        // the run or from now); `new_version`: this verify added one (the document moved)
                        let recorded = ctx
                            .store
                            .conn()
                            .query_row(
                                "SELECT COUNT(*) FROM versions WHERE caused_by_op = ?1",
                                [op_key(&hash, &op.id)],
                                |r| r.get::<_, i64>(0),
                            )
                            .unwrap_or(0)
                            > 0;
                        targets.push(json!({"op": op.id, "ok": true, "recorded": recorded, "new_version": added}));
                    }
                    Err(w) => {
                        targets.push(json!({"op": op.id, "ok": false, "why": w}));
                        problems.push(format!("{}: {w}", op.id));
                    }
                }
            }
            other => targets.push(
                json!({"op": op.id, "ok": false, "status": other.word(), "error": other.reason()}),
            ),
        }
    }
    let mut reports = vec![];
    let mut warnings = vec![];
    for (tenant, _) in tb_dates(p) {
        // the run's own before-report, at the date it was taken (the after-report uses the same date)
        let before: Option<(String, String)> = ctx
            .store
            .conn()
            .query_row(
                "SELECT payload, json_extract(params, '$.date') FROM reports WHERE tenant_id = ?1 AND report = 'TrialBalance' \
                 AND json_extract(params, '$.xa_package') = ?2 AND json_extract(params, '$.phase') = 'before' ORDER BY id LIMIT 1",
                rusqlite::params![tenant, hash],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .ok();
        let Some((before, date)) = before else {
            warnings.push(format!("{tenant}: no before-report (nothing ran yet)"));
            continue;
        };
        // a before-report from an older build with a sentinel date: never sent to Xero again
        let now = crate::store::now_iso();
        let horizon = format!(
            "{}{}",
            now[..4].parse::<i32>().unwrap_or(9998) + 1,
            &now[4..10]
        );
        let plausible = date <= horizon;
        if !plausible || ownpurse_core::dates::parse_iso(&date).is_none() {
            warnings.push(format!(
                "{tenant}: the before-report has an unusable date ({date}); the trial-balance deltas aren't checked"
            ));
            continue;
        }
        let after = get(ctx.transport, &tenant, &tb_path(&date))?;
        store_report(
            ctx.store,
            &tenant,
            &json!({"date": date, "xa_package": hash, "phase": "after"}),
            &after.body,
        )?;
        let b: Value = serde_json::from_str(&before).unwrap_or(Value::Null);
        let tb = |v: &Value| {
            ownpurse_core::xero_tb_balances(v)
                .map(|x| x.0)
                .map_err(|e| err("report", exit::RUNTIME, e.to_string()))
        };
        let (b, a) = (tb(&b)?, tb(&after.body)?);
        let mut moved: std::collections::BTreeMap<String, rust_decimal::Decimal> =
            Default::default();
        for k in b.keys().chain(a.keys()) {
            let d = a.get(k).copied().unwrap_or_default() - b.get(k).copied().unwrap_or_default();
            if !d.is_zero() {
                moved.insert(k.clone(), d);
            }
        }
        let mut want: std::collections::BTreeMap<String, rust_decimal::Decimal> =
            Default::default();
        for op in done_ops.iter().filter(|o| o.tenant_id == tenant) {
            for d in &op.expected {
                *want.entry(d.account_id.clone()).or_default() += d.amount;
            }
        }
        want.retain(|_, v| !v.is_zero());
        let mut diffs = vec![];
        for k in want
            .keys()
            .chain(moved.keys())
            .collect::<std::collections::BTreeSet<_>>()
        {
            let (w, m) = (
                want.get(k).copied().unwrap_or_default(),
                moved.get(k).copied().unwrap_or_default(),
            );
            if w != m {
                diffs.push(
                    json!({"account_id": k, "expected": w.to_string(), "moved": m.to_string()}),
                );
            }
        }
        if !diffs.is_empty() {
            problems.push(format!(
                "{tenant}: the trial balance moved differently from the expected deltas"
            ));
        }
        reports.push(json!({"tenant": tenant, "date": date, "differences": diffs}));
        let org = get(ctx.transport, &tenant, "Organisation")?;
        let o = &org.body["Organisations"][0];
        if o["PeriodLockDate"].as_str().is_none_or(str::is_empty)
            && o["EndOfYearLockDate"].as_str().is_none_or(str::is_empty)
        {
            warnings.push(format!("{tenant}: no lock date is set; after posting a closed year's corrections, set it in Xero (browser)"));
        }
    }
    // the package's outcome: verified only when every op landed and nothing differs
    let outcome = if status.iter().any(|s| matches!(s, OpStatus::Rejected(_))) {
        "rejected"
    } else if status.contains(&OpStatus::Uncertain) {
        "uncertain"
    } else if status.contains(&OpStatus::Pending) {
        "incomplete"
    } else if !problems.is_empty() {
        "discrepancy"
    } else {
        "verified"
    };
    // the record keeps what verify concluded (System › Writes shows it; `discrepancy` can't be derived)
    {
        use rusqlite::types::Value as Sql;
        let db = |e: crate::ApiError| err("record", exit::RUNTIME, e.message());
        ctx.store.ensure_verifications().map_err(db)?;
        ctx.store
            .append(
                "verifications",
                &[
                    ("hash", Sql::Text(hash.clone())),
                    ("outcome", Sql::Text(outcome.into())),
                    ("problems_json", Sql::Text(json!(problems).to_string())),
                    ("verified_at", Sql::Text(crate::store::now_iso())),
                ],
            )
            .map_err(db)?;
    }
    Ok(
        json!({"hash": hash, "outcome": outcome, "ok": outcome == "verified", "targets": targets,
              "reports": reports, "warnings": warnings, "problems": problems}),
    )
}

/// `write status`: every operation's attempts.
pub fn status(p: &Package, store: &Writer) -> Result<Value, WriteError> {
    status_with(p, store, &ownpurse_core::write::Links::new())
}

/// [`status`] with each operation's "Open in Xero" `url` (its target, or the journal it created).
pub fn status_with(
    p: &Package,
    store: &Writer,
    links: &ownpurse_core::write::Links,
) -> Result<Value, WriteError> {
    let hash = seal_of(p);
    let log = OpLog { store };
    let st = statuses(store, &hash, p)?;
    let ops: Vec<Value> = p
        .ops
        .iter()
        .zip(st)
        .map(|(o, s)| {
            let attempts = log.attempts(&op_key(&hash, &o.id)).unwrap_or_default();
            let created = match (&o.op, &s) {
                (Op::CreateJournal(_), OpStatus::Done(id)) => id.as_deref(),
                _ => None,
            };
            let url = ownpurse_core::write::op_url(o, links, created);
            json!({"op": o.id, "kind": o.op.kind(), "status": s.word(), "error": s.reason(), "url": url,
                   "target_id": match &s { OpStatus::Done(id) => json!(id), _ => Value::Null },
                   "attempts": attempts.iter().map(|a| json!({"seq": a.seq, "at": a.at, "state": a.state, "http_status": a.http_status, "attempt": a.detail.get("attempt")})).collect::<Vec<_>>()})
        })
        .collect();
    let chain = store
        .verify()
        .map(|(n, head)| json!({"entries": n, "head": head}))
        .unwrap_or_else(|e| json!({"error": e.message()}));
    let approvals: Vec<Value> = super::log::RecordApprovals { store }
        .approvals(&hash)
        .unwrap_or_default()
        .iter()
        .map(|a| json!({"via": a.via.as_str(), "source": a.via.label(), "at": a.at, "scope": a.scope}))
        .collect();
    Ok(json!({"hash": hash, "ops": ops, "approvals": approvals, "chain": chain}))
}

/// `write recover <pkg> [op]`: a recovery package for operations that landed (all of them, or one), built
/// from each operation's pre-compiled recovery and a fresh read of its target. It is never executed here:
/// it is sealed and approved on its own. An attachment has no API recovery (listed as excluded).
pub fn recover(p: &Package, only: Option<&str>, ctx: &mut Ctx<'_>) -> Result<Package, WriteError> {
    use ownpurse_core::write::{Excluded, ops};
    let hash = seal_of(p);
    let status = statuses(ctx.store, &hash, p)?;
    let rec_id = format!("{}-recovery", p.id);
    let mut out = Package {
        v: p.v,
        id: rec_id.clone(),
        title: format!("Recovery of {}", p.title),
        profile: p.profile.clone(),
        created_at: crate::store::now_iso(),
        accountant_decision: p.accountant_decision.clone(),
        ops: vec![],
        excluded: vec![],
        checklist: p.checklist.clone(),
        supersedes: vec![],
        source: None,
        seal: None,
    };
    for (op, s) in p.ops.iter().zip(&status).rev() {
        if only.is_some_and(|o| o != op.id) {
            continue;
        }
        let OpStatus::Done(target) = s else {
            if only.is_some() {
                return Err(err(
                    "usage",
                    exit::USAGE,
                    format!("{} has not landed ({s:?}); nothing to recover.", op.id),
                ));
            }
            continue;
        };
        let Some(r) = op.recovery.as_ref() else {
            out.excluded.push(Excluded {
                what: format!("{} ({})", op.id, op.op.kind()),
                why: "no recovery was pre-compiled for it".into(),
            });
            continue;
        };
        let Some(rop) = r.op.clone() else {
            out.excluded.push(Excluded {
                what: format!("{} ({})", op.id, op.op.kind()),
                why: r.note.clone(),
            });
            continue;
        };
        let id = format!("rec-{}", op.id);
        let negated: Vec<_> = op
            .expected
            .iter()
            .map(|d| {
                let mut d = d.clone();
                d.amount = -d.amount;
                d
            })
            .collect();
        let mut o = Operation {
            id: id.clone(),
            org: op.org.clone(),
            tenant_id: op.tenant_id.clone(),
            op: rop.clone(),
            request: ops::void_request(""),
            preconditions: vec![],
            expected_after: None,
            expected: negated,
            recovery: None,
            depends_on: vec![],
            decision_refs: op.decision_refs.clone(),
            source_refs: op.source_refs.clone(),
            gated: op.gated.clone(),
            flags: vec![format!("recovers {}", op.id)],
            // the recovery leaves its own audit note when the original did
            history_note: op.history_note.as_ref().map(|_| {
                ownpurse_core::write::cut_note(&format!("recovery of {} ({})", op.id, p.title))
            }),
        };
        match rop {
            Op::CreateJournal(mut j) => {
                let m = marker(&rec_id, &id);
                if !j.narration.contains(&m) {
                    j.narration = format!("{} · {m}", j.narration);
                }
                let mut codes: Vec<String> =
                    j.lines.iter().map(|l| l.account_code.clone()).collect();
                codes.sort();
                codes.dedup();
                o.preconditions = vec![
                    Precondition::MarkerAbsent { marker: m },
                    Precondition::AccountsActive { codes },
                    Precondition::PeriodOpen {
                        date: j.date.clone(),
                    },
                ];
                o.request = ops::journal_request(&j);
                o.op = Op::CreateJournal(j);
            }
            Op::RecodeBankLine(inv) => {
                let r = get(
                    ctx.transport,
                    &op.tenant_id,
                    &format!("BankTransactions/{}", inv.bank_transaction_id),
                )?;
                let doc = parse_body(&r, "BankTransactions").ok_or_else(|| {
                    err(
                        "not_found",
                        exit::PRECONDITION,
                        format!("BankTransactions/{} can't be read", inv.bank_transaction_id),
                    )
                })?;
                let acct_id = match &inv.to_account_code {
                    Some(code) => {
                        let a = get(ctx.transport, &op.tenant_id, "Accounts")?;
                        a.body["Accounts"]
                            .as_array()
                            .into_iter()
                            .flatten()
                            .find(|x| x["Code"] == code.as_str())
                            .and_then(|x| x["AccountID"].as_str().map(str::to_string))
                    }
                    None => None,
                };
                let (after, sent) = ops::recoded(&doc, &inv, acct_id.as_deref())?;
                o.preconditions = vec![Precondition::Target {
                    entity: "BankTransactions".into(),
                    id: inv.bank_transaction_id.clone(),
                    fingerprint: fingerprint(&doc),
                    version: None,
                    status: doc["Status"].as_str().unwrap_or("").into(),
                    before: Some(ownpurse_core::write::business_view(&doc)),
                }];
                o.expected_after = Some(fingerprint(&after));
                o.request = ops::recode_request(&inv.bank_transaction_id, sent);
                o.op = Op::RecodeBankLine(inv);
            }
            Op::VoidJournal { journal } => {
                let jid = match &journal {
                    JournalRef::Id(x) => x.clone(),
                    JournalRef::CreatedBy(_) => target.clone().unwrap_or_default(),
                };
                let r = get(
                    ctx.transport,
                    &op.tenant_id,
                    &format!("ManualJournals/{jid}"),
                )?;
                let doc = parse_body(&r, "ManualJournals").ok_or_else(|| {
                    err(
                        "not_found",
                        exit::PRECONDITION,
                        format!("ManualJournals/{jid} can't be read"),
                    )
                })?;
                o.preconditions = vec![Precondition::Target {
                    entity: "ManualJournals".into(),
                    id: jid.clone(),
                    fingerprint: fingerprint(&doc),
                    version: None,
                    status: "POSTED".into(),
                    before: Some(ownpurse_core::write::business_view(&doc)),
                }];
                o.request = ops::void_request(&jid);
                o.op = Op::VoidJournal {
                    journal: JournalRef::Id(jid),
                };
            }
            Op::VoidBankTransaction { bank_transaction } => {
                let bid = match &bank_transaction {
                    JournalRef::Id(x) => x.clone(),
                    JournalRef::CreatedBy(_) => target.clone().unwrap_or_default(),
                };
                let r = get(
                    ctx.transport,
                    &op.tenant_id,
                    &format!("BankTransactions/{bid}"),
                )?;
                let doc = parse_body(&r, "BankTransactions").ok_or_else(|| {
                    err(
                        "not_found",
                        exit::PRECONDITION,
                        format!("BankTransactions/{bid} can't be read"),
                    )
                })?;
                o.preconditions = vec![Precondition::Target {
                    entity: "BankTransactions".into(),
                    id: bid.clone(),
                    fingerprint: fingerprint(&doc),
                    version: None,
                    status: "AUTHORISED".into(),
                    before: Some(ownpurse_core::write::business_view(&doc)),
                }];
                o.request = ops::void_bank_request(&bid);
                o.op = Op::VoidBankTransaction {
                    bank_transaction: JournalRef::Id(bid),
                };
            }
            Op::CreateBankTransfer(mut tr) => {
                // the reverse transfer, under the recovery package's own marker
                let m = marker(&rec_id, &id);
                if !tr.reference.contains(&m) {
                    tr.reference = format!("{} - {m}", tr.reference);
                }
                o.preconditions = vec![
                    Precondition::ReferenceAbsent {
                        marker: m,
                        entity: Some("BankTransfers".into()),
                    },
                    Precondition::PeriodOpen {
                        date: tr.date.clone(),
                    },
                ];
                o.request = ops::transfer_request(&tr);
                o.op = Op::CreateBankTransfer(tr);
                o.flags
                    .push("permanent history: a transfer can't be voided through the API".into());
            }
            Op::ArchiveContact { contact } => {
                let cid = match &contact {
                    JournalRef::Id(x) => x.clone(),
                    JournalRef::CreatedBy(_) => target.clone().unwrap_or_default(),
                };
                o.preconditions = vec![Precondition::Exists {
                    entity: "Contacts".into(),
                    id: cid.clone(),
                }];
                o.request = ops::archive_contact_request(&cid);
                o.op = Op::ArchiveContact {
                    contact: JournalRef::Id(cid),
                };
            }
            Op::Attach(_)
            | Op::AddNote(_)
            | Op::CreateBankTransaction(_)
            | Op::CreateContact(_) => {
                continue;
            }
        }
        out.ops.push(o);
    }
    if out.ops.is_empty() && out.excluded.is_empty() {
        return Err(err(
            "usage",
            exit::USAGE,
            "Nothing has landed: there is nothing to recover.",
        ));
    }
    Ok(out)
}

/// The flag a plan sets on an operation whose approval knowingly covers a deliberate re-send (the
/// Idempotency-Key probe); it appears in the review export.
pub const PROBE_REPLAY_FLAG: &str = "probe_replay";

/// What `probe-replay` re-sends: the exact request, or the same request with a changed narration (the
/// marker kept), both under the operation's own Idempotency-Key.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Replay {
    Same,
    Changed,
}

/// `write probe-replay <pkg> <op> [--changed]` (Milestone 1, the Demo Company only): re-send a landed
/// journal's request under its own Idempotency-Key and report what Xero did (the marker count shows whether
/// a second journal appeared). Refused (`not_a_demo`, exit 2) before any request is built unless the
/// profile sets `require_tenant_name` and the live tenant's name contains "Demo Company"; then the three
/// locks, and the operation must carry the `probe_replay` flag its approval covered. Logged on the same
/// operation as a `replay` / `replay_changed` attempt; the operation stays landed.
pub fn replay(
    p: &Package,
    op_id: &str,
    kind: Replay,
    ctx: &mut Ctx<'_>,
) -> Result<Value, WriteError> {
    let op = p.ops.iter().find(|o| o.id == op_id).ok_or_else(|| {
        err(
            "usage",
            exit::USAGE,
            format!("No operation {op_id} in the package."),
        )
    })?;
    let not_demo = |why: String| {
        err("not_a_demo", exit::USAGE, why).help(
            "probe-replay runs only against the Demo Company, from its own folder and profile.",
        )
    };
    let Some(_) = ctx.require_tenant_name.as_ref() else {
        return Err(not_demo(
            "the profile does not set require_tenant_name".into(),
        ));
    };
    let live = ctx
        .transport
        .tenant_name(&op.tenant_id)
        .map_err(|e| transport_err(&e, false))?;
    if !live.contains("Demo Company") {
        return Err(not_demo(format!(
            "the connected tenant is '{live}', not a Demo Company"
        )));
    }
    let (hash, covered) = check_locks(p, ctx)?;
    let i = p.ops.iter().position(|o| o.id == op_id).unwrap_or(0);
    if !covered[i] || !op.flags.iter().any(|f| f == PROBE_REPLAY_FLAG) {
        return Err(err(
            "not_approved",
            exit::NOT_APPROVED,
            format!(
                "{op_id} is not marked `{PROBE_REPLAY_FLAG}` in the approved package: its approval doesn't cover a re-send."
            ),
        ));
    }
    let Op::CreateJournal(_) = &op.op else {
        return Err(err(
            "usage",
            exit::USAGE,
            "probe-replay re-sends a journal create only.",
        ));
    };
    let status = statuses(ctx.store, &hash, p)?;
    let OpStatus::Done(target) = status[i].clone() else {
        return Err(err(
            "usage",
            exit::USAGE,
            format!("{op_id} has not landed ({:?}); run it first.", status[i]),
        ));
    };
    let mut request = op.request.clone();
    if kind == Replay::Changed
        && let Body::Json { json } = &mut request.body
        && let Some(n) = json["ManualJournals"][0]["Narration"]
            .as_str()
            .map(str::to_string)
    {
        json["ManualJournals"][0]["Narration"] = json!(format!("{n} (replay, changed)"));
    }
    let attempt = match kind {
        Replay::Same => "replay",
        Replay::Changed => "replay_changed",
    };
    let log = OpLog { store: ctx.store };
    let key = op_key(&hash, op_id);
    let idem = idempotency_key(&hash, op_id);
    log.event(
        &key,
        AttemptState::Sending,
        Some(&idem),
        None,
        json!({"attempt": attempt, "path": request.path}),
    )?;
    let sent = ctx.transport.send(&Send {
        tenant_id: &op.tenant_id,
        request: &request,
        path: request.path.clone(),
        idempotency_key: &idem,
        file: None,
    });
    let (status_code, response) = match &sent {
        Ok(r) => (Some(r.status), r.body.clone()),
        Err(e) => (None, json!({"error": format!("{e:?}")})),
    };
    let hits = marker_hits(ctx.transport, &op.tenant_id, &marker(&p.id, op_id))?;
    let ids: Vec<Value> = hits.iter().map(|j| j["ManualJournalID"].clone()).collect();
    let returned = response["ManualJournals"][0]["ManualJournalID"].clone();
    let detail = json!({"attempt": attempt, "landed": true, "target_id": target, "http_status": status_code,
                        "returned_id": returned, "marker_count": hits.len(), "journals": ids, "response": response});
    // the original stays the operation's result; the replay is evidence
    log.event(
        &key,
        AttemptState::Reconciled,
        Some(&idem),
        status_code,
        detail.clone(),
    )?;
    Ok(
        json!({"op": op_id, "attempt": attempt, "http_status": status_code, "same_journal_returned": returned == json!(target),
              "marker_count": hits.len(), "duplicate": hits.len() > 1,
              "note": if hits.len() > 1 { "a second journal exists: plan a recovery for it" } else { "no second journal" }}),
    )
}
