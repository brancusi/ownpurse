//! What the record says about writes, for readers that never link the api crate (the TUI): every write
//! attempt (`op_events` with its operation), and every package registered at seal with its stage derived
//! from approvals, attempts and verification reports. Read-only; tables a record never needed (`packages`,
//! `approvals`) simply read as empty.

use crate::Record;
use serde_json::Value;

/// A package's stage, derived (never stored).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Stage {
    Sealed,
    Approved,
    Running,
    /// Refused by Xero or a precondition, or an attempt is uncertain (`write reconcile`).
    Stopped,
    Landed,
    Verified,
    /// A landed `<id>-recovery` package undid it.
    Recovered,
}

impl Stage {
    pub fn as_str(self) -> &'static str {
        match self {
            Stage::Sealed => "sealed",
            Stage::Approved => "approved",
            Stage::Running => "running",
            Stage::Stopped => "stopped",
            Stage::Landed => "landed",
            Stage::Verified => "verified",
            Stage::Recovered => "recovered",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PackageRow {
    pub hash: String,
    pub id: String,
    pub title: String,
    pub profile: String,
    pub ops: i64,
    /// Operations confirmed (landed) so far.
    pub landed: i64,
    pub sealed_at: String,
    /// The manifest or branch it came from.
    pub source_ref: Option<String>,
    pub stage: Stage,
    /// How it was approved (`demo standing approval`, `owner at his terminal` …), if it was.
    pub approval: Option<String>,
    /// `(op id, kind, org, marker)`.
    pub op_list: Vec<(String, String, String, String)>,
    /// Each op's page in Xero ("Open in Xero", same order as `op_list`): the document it touches, or the one
    /// it created once landed. None: no ShortCode, a package registered before links, or not created yet.
    pub op_urls: Vec<Option<String>>,
    /// The last `write verify` outcome the record keeps (`verified`, `discrepancy` …); None: never verified.
    pub verify: Option<String>,
    /// Each op's last attempt state (`confirmed`, `rejected` …), `""` before any.
    pub op_states: Vec<String>,
}

impl PackageRow {
    pub fn hash12(&self) -> &str {
        &self.hash[..self.hash.len().min(12)]
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AttemptRow {
    /// `op_events.id`.
    pub id: i64,
    pub at: String,
    /// `<hash12>:<op id>`.
    pub op_key: String,
    pub tenant: String,
    pub kind: String,
    /// sending | confirmed | rejected | uncertain | reconciled (`replay` attempts say so in `attempt`).
    pub state: String,
    pub http_status: Option<i64>,
    /// The Xero document it wrote, when known.
    pub target_id: Option<String>,
    /// `replay` / `replay_changed` for a probe re-send.
    pub attempt: Option<String>,
    /// The registered package's title and id, when known.
    pub package: Option<(String, String)>,
}

impl AttemptRow {
    pub fn op_id(&self) -> &str {
        self.op_key.split_once(':').map_or("", |(_, o)| o)
    }
}

fn has_table(r: &Record, name: &str) -> bool {
    r.conn()
        .query_row(
            "SELECT 1 FROM sqlite_master WHERE type='table' AND name=?1",
            [name],
            |_| Ok(()),
        )
        .is_ok()
}

/// A `packages` row: hash, package id, title, profile, ops, sealed at, source ref, ops JSON.
type Registered = (
    String,
    String,
    String,
    String,
    i64,
    String,
    Option<String>,
    String,
);

fn registered(r: &Record) -> Vec<Registered> {
    if !has_table(r, "packages") {
        return vec![];
    }
    let Ok(mut st) = r.conn().prepare(
        "SELECT hash, package_id, title, profile, ops, sealed_at, source_ref, ops_json FROM packages ORDER BY id",
    ) else {
        return vec![];
    };
    st.query_map([], |x| {
        Ok((
            x.get(0)?,
            x.get(1)?,
            x.get(2)?,
            x.get(3)?,
            x.get(4)?,
            x.get(5)?,
            x.get(6)?,
            x.get(7)?,
        ))
    })
    .map(|rows| rows.flatten().collect())
    .unwrap_or_default()
}

/// The last attempt state per operation of a package (by its hash's first 12 characters).
fn last_states(r: &Record, hash12: &str) -> Vec<(String, String, Value)> {
    let Ok(mut st) = r.conn().prepare(
        "SELECT e.op_id, e.state, e.detail FROM op_events e \
         JOIN (SELECT op_id, MAX(seq) AS s FROM op_events WHERE op_id LIKE ?1 || ':%' GROUP BY op_id) m \
           ON e.op_id = m.op_id AND e.seq = m.s",
    ) else {
        return vec![];
    };
    st.query_map([hash12], |x| {
        Ok((
            x.get::<_, String>(0)?,
            x.get::<_, String>(1)?,
            x.get::<_, Option<String>>(2)?,
        ))
    })
    .map(|rows| {
        rows.flatten()
            .map(|(o, s, d)| {
                (
                    o,
                    s,
                    d.and_then(|d| serde_json::from_str(&d).ok())
                        .unwrap_or(Value::Null),
                )
            })
            .collect()
    })
    .unwrap_or_default()
}

fn approval_of(r: &Record, hash: &str) -> Option<String> {
    if !has_table(r, "approvals") {
        return None;
    }
    let via: String = r
        .conn()
        .query_row(
            "SELECT via FROM approvals WHERE hash = ?1 ORDER BY id DESC LIMIT 1",
            [hash],
            |x| x.get(0),
        )
        .ok()?;
    Some(super::Via::parse(&via).map_or(via.clone(), |v| v.label().to_string()))
}

fn verified(r: &Record, hash: &str) -> bool {
    r.conn()
        .query_row(
            "SELECT 1 FROM reports WHERE report = 'TrialBalance' AND params LIKE '%\"phase\":\"after\"%' \
             AND params LIKE '%' || ?1 || '%' LIMIT 1",
            [hash],
            |_| Ok(()),
        )
        .is_ok()
}

/// The last outcome `write verify` recorded for a package (the lazy `verifications` table).
pub fn last_verify(r: &Record, hash: &str) -> Option<String> {
    if !has_table(r, "verifications") {
        return None;
    }
    r.conn()
        .query_row(
            "SELECT outcome FROM verifications WHERE hash = ?1 ORDER BY id DESC LIMIT 1",
            [hash],
            |x| x.get::<_, String>(0),
        )
        .ok()
}

/// The document an op created, from the `operations` table (absent before the run writes the row).
fn created_target(r: &Record, op_key: &str) -> Option<String> {
    if !has_table(r, "operations") {
        return None;
    }
    r.conn()
        .query_row(
            "SELECT target_id FROM operations WHERE op_id = ?1 AND target_id IS NOT NULL AND target_id <> '' \
             ORDER BY rowid DESC LIMIT 1",
            [op_key],
            |x| x.get::<_, String>(0),
        )
        .ok()
}

/// Every registered package, newest first, with its stage.
pub fn packages(r: &Record) -> Vec<PackageRow> {
    let mut out: Vec<PackageRow> = registered(r)
        .into_iter()
        .map(
            |(hash, id, title, profile, ops, sealed_at, source_ref, ops_json)| {
                let h12 = &hash[..hash.len().min(12)];
                let states = last_states(r, h12);
                let landed = states
                    .iter()
                    .filter(|(_, s, d)| {
                        s == "confirmed" || (s == "reconciled" && d["landed"] == true)
                    })
                    .count() as i64;
                let stuck = states
                    .iter()
                    .any(|(_, s, _)| matches!(s.as_str(), "rejected" | "uncertain"));
                let sending = states.iter().any(|(_, s, _)| s == "sending");
                let approval = approval_of(r, &hash);
                let stage = if stuck {
                    Stage::Stopped
                } else if ops > 0 && landed >= ops {
                    if verified(r, &hash) {
                        Stage::Verified
                    } else {
                        Stage::Landed
                    }
                } else if sending || landed > 0 {
                    Stage::Running
                } else if approval.is_some() {
                    Stage::Approved
                } else {
                    Stage::Sealed
                };
                let raw: Vec<Value> = serde_json::from_str::<Value>(&ops_json)
                    .ok()
                    .and_then(|v| v.as_array().cloned())
                    .unwrap_or_default();
                let op_list = raw
                    .iter()
                    .map(|o| {
                        let s = |k: &str| o[k].as_str().unwrap_or("").to_string();
                        (s("op"), s("kind"), s("org"), s("marker"))
                    })
                    .collect();
                let state_of = |op: &str| {
                    states
                        .iter()
                        .find(|(k, ..)| k.split_once(':').map(|x| x.1) == Some(op))
                };
                let op_states = raw
                    .iter()
                    .map(|o| {
                        state_of(o["op"].as_str().unwrap_or(""))
                            .map(|(_, s, _)| s.clone())
                            .unwrap_or_default()
                    })
                    .collect();
                let op_urls = raw
                    .iter()
                    .map(|o| {
                        let tenant = o["tenant"].as_str()?;
                        let entity = o["entity"].as_str()?;
                        let op = o["op"].as_str().unwrap_or("");
                        let id = o["target"].as_str().map(str::to_string).or_else(|| {
                            created_target(r, &format!("{h12}:{op}")).or_else(|| {
                                state_of(op).and_then(|(_, _, d)| {
                                    d["target_id"].as_str().map(str::to_string)
                                })
                            })
                        })?;
                        let sc = crate::deeplink::short_code(r, tenant)?;
                        crate::deeplink::url_for(&sc, entity, "", &id)
                    })
                    .collect();
                let verify = last_verify(r, &hash);
                PackageRow {
                    hash,
                    id,
                    title,
                    profile,
                    ops,
                    landed,
                    sealed_at,
                    source_ref,
                    stage,
                    approval,
                    op_list,
                    op_urls,
                    op_states,
                    verify,
                }
            },
        )
        .collect();
    // a package undone by a landed `<id>-recovery` package
    let recovered: Vec<String> = out
        .iter()
        .filter(|p| p.stage >= Stage::Landed && p.id.ends_with("-recovery"))
        .map(|p| p.id.trim_end_matches("-recovery").to_string())
        .collect();
    for p in &mut out {
        if p.stage >= Stage::Landed && recovered.contains(&p.id) {
            p.stage = Stage::Recovered;
        }
    }
    // one row per hash (a package registered again keeps its first row)
    let mut seen = std::collections::HashSet::new();
    out.retain(|p| seen.insert(p.hash.clone()));
    out.reverse();
    out
}

/// Every write attempt, newest first (at most `limit`).
pub fn attempts(r: &Record, limit: i64) -> Vec<AttemptRow> {
    let pk: Vec<(String, String, String)> = registered(r)
        .into_iter()
        .map(|(h, id, title, ..)| (h[..h.len().min(12)].to_string(), id, title))
        .collect();
    let Ok(mut st) = r.conn().prepare(
        "SELECT e.id, e.at, e.op_id, o.tenant_id, o.kind, e.state, e.http_status, e.detail, o.target_id \
         FROM op_events e JOIN operations o ON o.op_id = e.op_id ORDER BY e.id DESC LIMIT ?1",
    ) else {
        return vec![];
    };
    st.query_map([limit.max(1)], |x| {
        Ok((
            x.get::<_, i64>(0)?,
            x.get::<_, String>(1)?,
            x.get::<_, String>(2)?,
            x.get::<_, String>(3)?,
            x.get::<_, String>(4)?,
            x.get::<_, String>(5)?,
            x.get::<_, Option<i64>>(6)?,
            x.get::<_, Option<String>>(7)?,
            x.get::<_, Option<String>>(8)?,
        ))
    })
    .map(|rows| {
        rows.flatten()
            .map(
                |(id, at, op_key, tenant, kind, state, http_status, detail, target)| {
                    let d: Value = detail
                        .and_then(|d| serde_json::from_str(&d).ok())
                        .unwrap_or(Value::Null);
                    let h12 = op_key.split(':').next().unwrap_or("").to_string();
                    AttemptRow {
                        id,
                        at,
                        tenant,
                        kind,
                        state,
                        http_status,
                        target_id: d["target_id"].as_str().map(str::to_string).or(target),
                        attempt: d["attempt"].as_str().map(str::to_string),
                        package: pk
                            .iter()
                            .find(|(h, ..)| *h == h12)
                            .map(|(_, id, t)| (t.clone(), id.clone())),
                        op_key,
                    }
                },
            )
            .collect()
    })
    .unwrap_or_default()
}

/// The registered package whose marker a narration carries (`XA:<package id>:<op id>`): its title, id and
/// hash, and the op.
pub fn package_of_marker(r: &Record, narration: &str) -> Option<(String, String, String, String)> {
    let i = narration.find("XA:")?;
    let m: String = narration[i + 3..]
        .chars()
        .take_while(|c| !c.is_whitespace())
        .collect();
    let (pkg, op) = m.rsplit_once(':')?;
    registered(r)
        .into_iter()
        .rev()
        .find(|x| x.1 == pkg)
        .map(|x| (x.2, x.1, x.0, op.to_string()))
}

/// How the package with this hash was approved, if it was (readers outside this module: manifests).
pub fn approval_for(r: &Record, hash: &str) -> Option<String> {
    approval_of(r, hash)
}

/// The latest attempt state of each operation of the package whose hash starts with `hash12`:
/// `(op key "<hash12>:<op id>", state, detail)`.
pub fn op_states(r: &Record, hash12: &str) -> Vec<(String, String, Value)> {
    last_states(r, hash12)
}

/// Whether the package's after-run verification report is in the record.
pub fn is_verified(r: &Record, hash: &str) -> bool {
    verified(r, hash)
}
