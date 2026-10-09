//! Mirror Xero entities into the append-only record.
//!
//! Full pull for small settings tables; If-Modified-Since + paging for transaction tables, with a periodic
//! full sweep (some edits don't bump UpdatedDateUTC). Rows are never removed locally: deletions arrive as
//! status changes, and ids a full sweep no longer sees are reported as `vanished` in the sync run. Content
//! hashes are stable, so a sync after an earlier sync of the same data records nothing new.

use crate::error::ApiError;
use crate::http::XeroHttp;
use crate::store::{Writer, now_iso};
use ownpurse_core::canon::json_str;
use rusqlite::types::Value as Sql;
use serde_json::{Map, Value};
use std::time::{SystemTime, UNIX_EPOCH};

pub const DAY: i64 = 86400;
pub const PAGE_SIZE: i64 = 1000;

#[derive(Debug, Clone)]
pub struct Entity {
    pub name: &'static str,
    pub id_field: &'static str,
    pub list_key: Option<&'static str>,
    pub paged: bool,
    pub order: bool,
    pub status_field: &'static str,
    pub params: &'static [(&'static str, &'static str)],
    pub full_extra_passes: &'static [&'static [(&'static str, &'static str)]],
}

impl Entity {
    pub fn key(&self) -> &'static str {
        self.list_key.unwrap_or(self.name)
    }
}

const fn e(name: &'static str, id_field: &'static str) -> Entity {
    Entity {
        name,
        id_field,
        list_key: None,
        paged: false,
        order: true,
        status_field: "Status",
        params: &[],
        full_extra_passes: &[],
    }
}

/// The entities, in sync order, with their settings.
pub fn entities() -> Vec<Entity> {
    vec![
        Entity {
            list_key: Some("Organisations"),
            status_field: "OrganisationStatus",
            ..e("Organisation", "OrganisationID")
        },
        e("Accounts", "AccountID"),
        e("TaxRates", "TaxType"),
        Entity {
            params: &[("includeArchived", "true")],
            ..e("TrackingCategories", "TrackingCategoryID")
        },
        Entity {
            status_field: "",
            ..e("Currencies", "Code")
        },
        Entity {
            status_field: "",
            ..e("BankTransfers", "BankTransferID")
        },
        Entity {
            paged: true,
            status_field: "ContactStatus",
            params: &[("includeArchived", "true")],
            ..e("Contacts", "ContactID")
        },
        Entity {
            paged: true,
            ..e("BankTransactions", "BankTransactionID")
        },
        // Xero ignores paging on ManualJournals when `order` is sent (tap-xero workaround)
        Entity {
            paged: true,
            order: false,
            ..e("ManualJournals", "ManualJournalID")
        },
        Entity {
            paged: true,
            full_extra_passes: &[&[("Statuses", "DELETED,VOIDED")]],
            ..e("Invoices", "InvoiceID")
        },
        Entity {
            paged: true,
            ..e("CreditNotes", "CreditNoteID")
        },
        Entity {
            paged: true,
            ..e("Payments", "PaymentID")
        },
        Entity {
            paged: true,
            ..e("Overpayments", "OverpaymentID")
        },
        Entity {
            paged: true,
            ..e("Prepayments", "PrepaymentID")
        },
    ]
}

pub const REFERENCE: [&str; 5] = [
    "Organisation",
    "Accounts",
    "TaxRates",
    "TrackingCategories",
    "Currencies",
];
pub const TRANSACTIONS: [&str; 9] = [
    "Contacts",
    "BankTransactions",
    "BankTransfers",
    "ManualJournals",
    "Invoices",
    "CreditNotes",
    "Payments",
    "Overpayments",
    "Prepayments",
];

/// Case-insensitive lookup.
pub fn entity(name: &str) -> Result<Entity, ApiError> {
    entities()
        .into_iter()
        .find(|x| x.name.eq_ignore_ascii_case(name))
        .ok_or_else(|| {
            let all: Vec<&str> = entities().iter().map(|x| x.name).collect();
            ApiError::new(
                "usage",
                format!("Unknown entity '{name}'."),
                format!("Entities: {}", all.join(", ")),
            )
        })
}

fn civil(secs: i64) -> String {
    let (days, rem) = (secs.div_euclid(DAY), secs.rem_euclid(DAY));
    let (y, m, d) = ownpurse_core::dates::civil_from_days(days);
    format!(
        "{y:04}-{m:02}-{d:02}T{:02}:{:02}:{:02}",
        rem / 3600,
        rem % 3600 / 60,
        rem % 60
    )
}

fn parse_secs(s: &str) -> Option<i64> {
    // YYYY-MM-DDTHH:MM:SS (anything after is ignored by the callers that need it)
    let b = s.as_bytes();
    if b.len() < 19 || b[4] != b'-' || b[7] != b'-' || b[13] != b':' || b[16] != b':' {
        return None;
    }
    let n = |r: std::ops::Range<usize>| s.get(r).and_then(|x| x.parse::<i64>().ok());
    let days = ownpurse_core::dates::days_from_civil(n(0..4)?, n(5..7)? as u32, n(8..10)? as u32);
    Some(days * DAY + n(11..13)? * 3600 + n(14..16)? * 60 + n(17..19)?)
}

/// A Xero date: `/Date(ms±zzzz)/` or ISO text → `YYYY-MM-DDTHH:MM:SSZ` (UTC); other text unchanged.
pub fn xero_datetime(value: &str) -> Option<String> {
    if value.is_empty() {
        return None;
    }
    if let Some(inner) = value
        .strip_prefix("/Date(")
        .and_then(|r| r.strip_suffix(")/"))
    {
        let split = inner[1.min(inner.len())..]
            .find(['+', '-'])
            .map(|i| i + 1)
            .unwrap_or(inner.len());
        let (ms, off) = inner.split_at(split);
        let off_ok =
            off.is_empty() || (off.len() == 5 && off[1..].bytes().all(|c| c.is_ascii_digit()));
        let digits = ms.strip_prefix('-').unwrap_or(ms);
        let ms_ok = !digits.is_empty() && digits.bytes().all(|c| c.is_ascii_digit());
        if let (true, true, Ok(ms)) = (off_ok, ms_ok, ms.parse::<i64>()) {
            return Some(format!("{}Z", civil(ms.div_euclid(1000))));
        }
        // not the /Date()/ pattern and not ISO text either, so the text is returned unchanged
    }
    // ISO: naive = UTC; Z or ±HH:MM offsets converted to UTC; fractions dropped
    let Some(base) = parse_secs(value) else {
        return Some(value.to_string());
    };
    let rest = &value[19..];
    let rest = rest
        .strip_prefix('.')
        .map(|r| r.trim_start_matches(|c: char| c.is_ascii_digit()))
        .unwrap_or(rest);
    let offset = match rest {
        "" | "Z" => 0,
        r if (r.starts_with('+') || r.starts_with('-')) && r.len() == 6 && &r[3..4] == ":" => {
            let sign = if r.starts_with('-') { -1 } else { 1 };
            match (r[1..3].parse::<i64>(), r[4..6].parse::<i64>()) {
                (Ok(h), Ok(m)) => sign * (h * 3600 + m * 60),
                _ => return Some(value.to_string()),
            }
        }
        _ => return Some(value.to_string()),
    };
    Some(format!("{}Z", civil(base - offset)))
}

fn now_secs() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

/// Seconds since a `finished_at` (first 19 characters, UTC).
pub fn age_seconds(finished_at: &str) -> i64 {
    parse_secs(finished_at)
        .map(|t| now_secs() - t)
        .unwrap_or(i64::MAX)
}

/// JSON in insertion order, ASCII-escaped, `", "` / `": "` separators.
fn py_dumps(pairs: &[(&str, Value)]) -> String {
    let mut out = String::from("{");
    for (i, (k, v)) in pairs.iter().enumerate() {
        if i > 0 {
            out.push_str(", ");
        }
        json_str(&mut out, k, true);
        out.push_str(": ");
        dump_value(&mut out, v);
    }
    out.push('}');
    out
}

fn dump_value(out: &mut String, v: &Value) {
    match v {
        Value::String(s) => json_str(out, s, true),
        Value::Array(a) => {
            out.push('[');
            for (i, x) in a.iter().enumerate() {
                if i > 0 {
                    out.push_str(", ");
                }
                dump_value(out, x);
            }
            out.push(']');
        }
        other => out.push_str(&other.to_string()),
    }
}

/// The error class name recorded for an error code (sync_runs `detail.error`).
fn py_class(code: &str) -> &'static str {
    match code {
        "needs_login" => "NeedsLogin",
        "daily_limit" => "BudgetExhausted",
        "rate_limited" => "RateLimited",
        "forbidden" => "Forbidden",
        "not_found" => "NotFound",
        "usage" => "UsageError",
        "integrity" => "IntegrityError",
        "offline" => "Offline",
        _ => "ApiError",
    }
}

pub struct Syncer<'a> {
    pub store: &'a Writer,
    pub http: &'a XeroHttp<'a>,
    pub page_size: i64,
    pub overlap_seconds: i64,
    pub full_every: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyncResult {
    pub entity: String,
    pub mode: String,
    pub fetched: usize,
    pub new_versions: usize,
    pub calls: i64,
    pub vanished: usize,
}

impl<'a> Syncer<'a> {
    pub fn new(store: &'a Writer, http: &'a XeroHttp<'a>) -> Syncer<'a> {
        Syncer {
            store,
            http,
            page_size: PAGE_SIZE,
            overlap_seconds: 5,
            full_every: 7 * DAY,
        }
    }

    fn fetch(
        &self,
        tenant_id: &str,
        e: &Entity,
        extra: &[(&str, &str)],
        ims: Option<&str>,
    ) -> Result<(Vec<Value>, i64), ApiError> {
        let base: Vec<(String, Value)> = {
            // {**e.params, **params}: later keys override, first position kept
            let mut m: Map<String, Value> = Map::new();
            for (k, v) in e.params.iter().chain(extra.iter()) {
                m.insert(k.to_string(), Value::from(*v));
            }
            m.into_iter().collect()
        };
        let take = |data: &Value| -> Vec<Value> {
            data.get(e.key())
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default()
        };
        if !e.paged {
            let data = self.http.get(tenant_id, e.name, &base, ims)?;
            return Ok((take(&data), 1));
        }
        let (mut items, mut calls, mut page) = (Vec::new(), 0i64, 1i64);
        loop {
            let mut p = base.clone();
            p.push(("page".into(), Value::from(page)));
            p.push(("pageSize".into(), Value::from(self.page_size)));
            if e.order {
                p.push(("order".into(), Value::from("UpdatedDateUTC ASC")));
            }
            let data = self.http.get(tenant_id, e.name, &p, ims)?;
            calls += 1;
            let chunk = take(&data);
            let n = chunk.len() as i64;
            items.extend(chunk);
            match data
                .get("pagination")
                .and_then(|x| x.get("pageCount"))
                .and_then(Value::as_i64)
            {
                Some(count) if page >= count => break,
                Some(_) => {}
                None if n < self.page_size => break,
                None => {}
            }
            page += 1;
        }
        Ok((items, calls))
    }

    /// Sync one entity (full or incremental). Errors are recorded as a failed run.
    pub fn sync(&self, tenant_id: &str, e: &Entity, full: bool) -> Result<SyncResult, ApiError> {
        let started = now_iso();
        let last_full = self.store.last_sync(tenant_id, e.name, Some("full"))?;
        let hw = self.store.high_water(tenant_id, e.name)?;
        let incremental = (e.paged || e.name == "Contacts")
            && !full
            && hw.is_some()
            && last_full
                .as_deref()
                .is_some_and(|f| age_seconds(f) < self.full_every);
        let mode = if incremental { "incremental" } else { "full" };
        let ims: Option<String> = if incremental {
            hw.as_deref()
                .and_then(parse_secs)
                .map(|t| civil(t - self.overlap_seconds))
        } else {
            None
        };
        let run_id = crate::store::new_id()?;
        let mut calls = 0i64;
        let result = (|| -> Result<SyncResult, ApiError> {
            let mut fetched: Map<String, Value> = Map::new();
            let mut passes: Vec<&[(&str, &str)]> = vec![&[]];
            if mode == "full" {
                passes.extend(e.full_extra_passes.iter().copied());
            }
            for extra in passes {
                let (items, n) = self.fetch(tenant_id, e, extra, ims.as_deref())?;
                calls += n;
                for it in items {
                    let id = match it.get(e.id_field) {
                        Some(Value::String(s)) => s.clone(),
                        Some(other) => other.to_string(),
                        None => {
                            return Err(ApiError::api(
                                format!("{} item without {}.", e.name, e.id_field),
                                "",
                            ));
                        }
                    };
                    fetched.insert(id, it);
                }
            }
            self.store.tx(|| {
                let mut known = self.store.latest_hashes(tenant_id, e.name)?;
                let before: Vec<String> = known.keys().cloned().collect();
                let (mut new_versions, mut high) = (0usize, hw.clone());
                for (xid, it) in &fetched {
                    let upd = it
                        .get("UpdatedDateUTC")
                        .and_then(Value::as_str)
                        .and_then(xero_datetime);
                    if let Some(u) = &upd
                        && high.as_ref().is_none_or(|h| u > h)
                    {
                        high = Some(u.clone());
                    }
                    let status = if e.status_field.is_empty() {
                        None
                    } else {
                        it.get(e.status_field).and_then(Value::as_str)
                    };
                    if self.store.record_version(
                        tenant_id,
                        e.name,
                        xid,
                        it,
                        status,
                        upd.as_deref(),
                        "sync",
                        Some(&run_id),
                        &mut known,
                    )? {
                        new_versions += 1;
                    }
                }
                let mut vanished: Vec<String> = if mode == "full" {
                    before
                        .into_iter()
                        .filter(|id| !fetched.contains_key(id))
                        .collect()
                } else {
                    vec![]
                };
                vanished.sort();
                let detail = if !vanished.is_empty() || ims.is_some() {
                    let v: Vec<Value> = vanished
                        .iter()
                        .take(200)
                        .map(|s| Value::from(s.as_str()))
                        .collect();
                    Sql::Text(py_dumps(&[
                        ("vanished", Value::Array(v)),
                        ("ims", ims.clone().map(Value::from).unwrap_or(Value::Null)),
                    ]))
                } else {
                    Sql::Null
                };
                self.store.append(
                    "sync_runs",
                    &[
                        ("id", Sql::Text(run_id.clone())),
                        ("tenant_id", Sql::Text(tenant_id.into())),
                        ("entity", Sql::Text(e.name.into())),
                        ("mode", Sql::Text(mode.into())),
                        ("started_at", Sql::Text(started.clone())),
                        ("finished_at", Sql::Text(now_iso())),
                        ("outcome", Sql::Text("ok".into())),
                        ("calls", Sql::Integer(calls)),
                        ("fetched", Sql::Integer(fetched.len() as i64)),
                        ("new_versions", Sql::Integer(new_versions as i64)),
                        (
                            "high_water",
                            high.clone().map(Sql::Text).unwrap_or(Sql::Null),
                        ),
                        ("detail", detail),
                    ],
                )?;
                Ok(SyncResult {
                    entity: e.name.into(),
                    mode: mode.into(),
                    fetched: fetched.len(),
                    new_versions,
                    calls,
                    vanished: vanished.len(),
                })
            })
        })();
        if let Err(err) = &result {
            let message: String = err.message.chars().take(500).collect();
            self.store.append(
                "sync_runs",
                &[
                    ("id", Sql::Text(run_id.clone())),
                    ("tenant_id", Sql::Text(tenant_id.into())),
                    ("entity", Sql::Text(e.name.into())),
                    ("mode", Sql::Text(mode.into())),
                    ("started_at", Sql::Text(started)),
                    ("finished_at", Sql::Text(now_iso())),
                    ("outcome", Sql::Text("error".into())),
                    ("calls", Sql::Integer(calls)),
                    ("fetched", Sql::Integer(0)),
                    ("new_versions", Sql::Integer(0)),
                    ("high_water", Sql::Null),
                    (
                        "detail",
                        Sql::Text(py_dumps(&[
                            ("error", Value::from(py_class(err.code))),
                            ("message", Value::from(message)),
                        ])),
                    ),
                ],
            )?;
        }
        result
    }
}
