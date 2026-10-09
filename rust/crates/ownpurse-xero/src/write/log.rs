//! The attempt log and the approvals, in the record (append-only, chained):
//! - `operations`: one row per (package hash, op), registered before its first attempt;
//! - `op_events`: every state, `sending` appended durably BEFORE the request;
//! - `approvals`: the owner's approvals bound to a package hash ([`RecordApprovals`], the `ApprovalStore`).

use crate::store::{Writer, now_iso};
use ownpurse_core::write::{
    ApprovalRecord, ApprovalScope, ApprovalStore, Attempt, AttemptState, Operation, Via,
    WriteError, exit, idempotency_key,
};
use rusqlite::OptionalExtension;
use rusqlite::types::Value as Sql;
use serde_json::{Value, json};

fn db(e: impl std::fmt::Display) -> WriteError {
    WriteError::new("record", exit::RUNTIME, format!("record: {e}"))
}

/// `operations.op_id` of an operation of a package: `<hash 12>:<op id>`.
pub fn op_key(package_hash: &str, op_id: &str) -> String {
    format!("{}:{op_id}", &package_hash[..package_hash.len().min(12)])
}

pub struct OpLog<'a> {
    pub store: &'a Writer,
}

impl OpLog<'_> {
    /// Register the operation (idempotent): the request, its target and its decision refs.
    pub fn register(
        &self,
        package_hash: &str,
        op: &Operation,
        target_id: Option<&str>,
    ) -> Result<String, WriteError> {
        let key = op_key(package_hash, &op.id);
        let exists: Option<String> = self
            .store
            .conn()
            .query_row(
                "SELECT op_id FROM operations WHERE op_id = ?1",
                [&key],
                |r| r.get(0),
            )
            .optional()
            .map_err(db)?;
        if exists.is_some() {
            return Ok(key);
        }
        let request = serde_json::to_value(&op.request).map_err(db)?;
        let refs = serde_json::to_value(&op.decision_refs).map_err(db)?;
        self.store
            .append(
                "operations",
                &[
                    ("op_id", Sql::Text(key.clone())),
                    ("created_at", Sql::Text(now_iso())),
                    ("tenant_id", Sql::Text(op.tenant_id.clone())),
                    ("kind", Sql::Text(op.op.kind().into())),
                    ("target_entity", Sql::Text(op.op.entity().into())),
                    (
                        "target_id",
                        target_id.map(|t| Sql::Text(t.into())).unwrap_or(Sql::Null),
                    ),
                    (
                        "fingerprint",
                        Sql::Text(idempotency_key(package_hash, &op.id)),
                    ),
                    (
                        "request",
                        Sql::Text(ownpurse_core::canon::canonical(&request)),
                    ),
                    ("expected_before_version", Sql::Null),
                    (
                        "decision_refs",
                        Sql::Text(ownpurse_core::canon::canonical(&refs)),
                    ),
                    ("approval_ref", Sql::Text(package_hash.into())),
                    ("supersedes_op", Sql::Null),
                ],
            )
            .map_err(|e| db(e.message()))?;
        Ok(key)
    }

    /// Append one state (the next seq for this op). Returns the attempt as recorded.
    pub fn event(
        &self,
        op_key: &str,
        state: AttemptState,
        idem: Option<&str>,
        http_status: Option<u16>,
        detail: Value,
    ) -> Result<Attempt, WriteError> {
        let seq: i64 = self
            .store
            .conn()
            .query_row(
                "SELECT COALESCE(MAX(seq), 0) + 1 FROM op_events WHERE op_id = ?1",
                [op_key],
                |r| r.get(0),
            )
            .map_err(db)?;
        let at = now_iso();
        self.store
            .append(
                "op_events",
                &[
                    ("op_id", Sql::Text(op_key.into())),
                    ("seq", Sql::Integer(seq)),
                    ("at", Sql::Text(at.clone())),
                    ("state", Sql::Text(state.as_str().into())),
                    (
                        "idempotency_key",
                        idem.map(|k| Sql::Text(k.into())).unwrap_or(Sql::Null),
                    ),
                    (
                        "http_status",
                        http_status
                            .map(|s| Sql::Integer(i64::from(s)))
                            .unwrap_or(Sql::Null),
                    ),
                    (
                        "detail",
                        Sql::Text(ownpurse_core::canon::canonical(&detail)),
                    ),
                ],
            )
            .map_err(|e| db(e.message()))?;
        Ok(Attempt {
            op_id: op_key.into(),
            seq,
            at,
            state,
            idempotency_key: idem.map(str::to_string),
            http_status,
            detail,
        })
    }

    /// Every attempt of an op, in order.
    pub fn attempts(&self, op_key: &str) -> Result<Vec<Attempt>, WriteError> {
        let mut st = self
            .store
            .conn()
            .prepare(
                "SELECT seq, at, state, idempotency_key, http_status, detail FROM op_events \
                 WHERE op_id = ?1 ORDER BY seq",
            )
            .map_err(db)?;
        let rows = st
            .query_map([op_key], |r| {
                Ok((
                    r.get::<_, i64>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, String>(2)?,
                    r.get::<_, Option<String>>(3)?,
                    r.get::<_, Option<i64>>(4)?,
                    r.get::<_, Option<String>>(5)?,
                ))
            })
            .map_err(db)?;
        let mut out = vec![];
        for row in rows {
            let (seq, at, state, key, status, detail) = row.map_err(db)?;
            let state: AttemptState = serde_json::from_value(Value::String(state)).map_err(db)?;
            out.push(Attempt {
                op_id: op_key.into(),
                seq,
                at,
                state,
                idempotency_key: key,
                http_status: status.map(|s| s as u16),
                detail: detail
                    .and_then(|d| serde_json::from_str(&d).ok())
                    .unwrap_or(Value::Null),
            });
        }
        Ok(out)
    }

    /// The last attempt's state, if any.
    pub fn last(&self, op_key: &str) -> Result<Option<Attempt>, WriteError> {
        Ok(self.attempts(op_key)?.pop())
    }
}

/// The record's `approvals` table as the `ApprovalStore`.
pub struct RecordApprovals<'a> {
    pub store: &'a Writer,
}

impl ApprovalStore for RecordApprovals<'_> {
    fn record(&mut self, a: &ApprovalRecord) -> Result<i64, WriteError> {
        self.store.ensure_approvals().map_err(|e| db(e.message()))?;
        let scope = serde_json::to_value(&a.scope).map_err(db)?;
        let via = a.via.as_str();
        let id = self
            .store
            .append(
                "approvals",
                &[
                    ("hash", Sql::Text(a.hash.clone())),
                    ("verbatim", Sql::Text(a.verbatim.clone())),
                    ("via", Sql::Text(via.into())),
                    ("where_given", Sql::Text(a.where_.clone())),
                    ("at", Sql::Text(a.at.clone())),
                    ("scope", Sql::Text(ownpurse_core::canon::canonical(&scope))),
                    ("recorded_at", Sql::Text(a.recorded_at.clone())),
                ],
            )
            .map_err(|e| db(e.message()))?;
        id.parse().map_err(db)
    }

    fn approvals(&self, hash: &str) -> Result<Vec<ApprovalRecord>, WriteError> {
        if !self.store.has_approvals().map_err(|e| db(e.message()))? {
            return Ok(vec![]);
        }
        let mut st = self
            .store
            .conn()
            .prepare(
                "SELECT hash, verbatim, via, where_given, at, scope, recorded_at FROM approvals \
                 WHERE hash = ?1 ORDER BY id",
            )
            .map_err(db)?;
        let rows = st
            .query_map([hash], |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, String>(2)?,
                    r.get::<_, String>(3)?,
                    r.get::<_, String>(4)?,
                    r.get::<_, String>(5)?,
                    r.get::<_, String>(6)?,
                ))
            })
            .map_err(db)?;
        let mut out = vec![];
        for row in rows {
            let (hash, verbatim, via, where_, at, scope, recorded_at) = row.map_err(db)?;
            out.push(ApprovalRecord {
                hash,
                verbatim,
                via: Via::parse(&via).unwrap_or(Via::Chat),
                where_,
                at,
                scope: serde_json::from_str::<ApprovalScope>(&scope).map_err(db)?,
                recorded_at,
            });
        }
        Ok(out)
    }
}

/// A short JSON note for an event's detail.
pub fn note(what: &str) -> Value {
    json!({"note": what})
}

/// Register a sealed package in the record's `packages` table (created on first use), once per hash: what
/// the TUI lists (title, stage, op counts) and what links a package to its manifest or branch.
pub fn register_package(
    store: &Writer,
    p: &ownpurse_core::write::Package,
    hash: &str,
) -> Result<bool, WriteError> {
    let rec = |e: crate::ApiError| db(e.message());
    store.ensure_packages().map_err(rec)?;
    let seen: Option<i64> = store
        .conn()
        .query_row("SELECT id FROM packages WHERE hash = ?1", [hash], |r| {
            r.get(0)
        })
        .optional()
        .map_err(db)?;
    if seen.is_some() {
        return Ok(false);
    }
    let ops: Vec<Value> = p
        .ops
        .iter()
        .map(|o| {
            // tenant, entity and the pre-existing target: "Open in Xero" from the record (System › Writes)
            json!({"op": o.id, "kind": o.op.kind(), "org": o.org,
                   "marker": ownpurse_core::write::marker(&p.id, &o.id),
                   "tenant": o.tenant_id, "entity": o.op.entity(),
                   "target": ownpurse_core::write::op_target(o).map(|(_, id)| id)})
        })
        .collect();
    store
        .append(
            "packages",
            &[
                ("hash", Sql::Text(hash.into())),
                ("package_id", Sql::Text(p.id.clone())),
                ("title", Sql::Text(p.title.clone())),
                ("profile", Sql::Text(p.profile.clone())),
                ("ops", Sql::Integer(p.ops.len() as i64)),
                (
                    "ops_json",
                    Sql::Text(ownpurse_core::canon::canonical(&Value::Array(ops))),
                ),
                (
                    "source_ref",
                    p.source.clone().map(Sql::Text).unwrap_or(Sql::Null),
                ),
                ("sealed_at", Sql::Text(now_iso())),
            ],
        )
        .map_err(rec)?;
    Ok(true)
}
