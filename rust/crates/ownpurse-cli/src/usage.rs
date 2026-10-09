//! `usage`: API calls, quotas, endpoints and sync runs, read from the
//! record's logs. Read-only; the record's `http_log` is only counted, never written.

use crate::app::{App, Res};
use crate::cmds::{Row, g, truthy};
use crate::obj;
use ownpurse_core::dates::{civil_from_days, iso};
use serde_json::Value;

/// A SQLite value as shown (int, float, text, None; blobs as `b'..'` text).
pub fn sql_value(v: rusqlite::types::ValueRef<'_>) -> Value {
    use rusqlite::types::ValueRef;
    match v {
        ValueRef::Null => Value::Null,
        ValueRef::Integer(i) => Value::from(i),
        ValueRef::Real(f) => serde_json::Number::from_f64(f)
            .map(Value::Number)
            .unwrap_or(Value::Null),
        ValueRef::Text(t) => Value::String(String::from_utf8_lossy(t).into_owned()),
        ValueRef::Blob(b) => Value::String(format!("b'{}'", String::from_utf8_lossy(b))),
    }
}

/// Now in UTC as (seconds since the epoch, microseconds).
fn now() -> (i64, u32) {
    let d = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    (d.as_secs() as i64, d.subsec_micros())
}

/// `%Y-%m-%dT%H:%M:%S.%fZ` for an epoch time.
fn iso_micro(secs: i64, micros: u32) -> String {
    let days = secs.div_euclid(86_400);
    let rem = secs.rem_euclid(86_400);
    let (y, m, d) = civil_from_days(days);
    format!(
        "{}T{:02}:{:02}:{:02}.{micros:06}Z",
        iso(y, m, d),
        rem / 3600,
        (rem % 3600) / 60,
        rem % 60
    )
}

fn date_of_epoch(secs: i64) -> String {
    let (y, m, d) = civil_from_days(secs.div_euclid(86_400));
    iso(y, m, d)
}

fn is_hex(s: &str) -> bool {
    s.chars().all(|c| c.is_ascii_hexdigit())
}

/// `'BankTransactions/3f…/History'` → `'BankTransactions/{id}/History'` (any 8-4-4-4-12 hex id).
pub fn endpoint(path: &str) -> String {
    let p = path.split('?').next().unwrap_or("");
    let chars: Vec<char> = p.chars().collect();
    let mut out = String::with_capacity(p.len());
    let mut i = 0;
    while i < chars.len() {
        if i + 36 <= chars.len() {
            let cand: String = chars[i..i + 36].iter().collect();
            let parts: Vec<&str> = cand.split('-').collect();
            if parts.len() == 5
                && [8, 4, 4, 4, 12]
                    .iter()
                    .zip(&parts)
                    .all(|(n, s)| s.len() == *n && is_hex(s))
            {
                out.push_str("{id}");
                i += 36;
                continue;
            }
        }
        out.push(chars[i]);
        i += 1;
    }
    out
}

fn half_even_div(num: i64, den: i64) -> i64 {
    let q = num.div_euclid(den);
    let r = num.rem_euclid(den);
    match (2 * r).cmp(&den) {
        std::cmp::Ordering::Greater => q + 1,
        std::cmp::Ordering::Less => q,
        std::cmp::Ordering::Equal => q + (q & 1),
    }
}

pub fn usage(app: &mut App, since: Option<&str>, days: Option<i64>, recent: i64) -> Res<()> {
    let (secs, micros) = now();
    let today = date_of_epoch(secs);
    let mut since = since.map(str::to_string);
    if let Some(d) = days.filter(|d| *d != 0) {
        since = Some(date_of_epoch(secs - d * 86_400));
    }
    let since = since.unwrap_or_else(|| today.clone());
    let orgs = app.config.profile.orgs.clone();
    let label = |t: &Value| -> String {
        match t.as_str() {
            Some(s) => orgs
                .iter()
                .find(|(_, tid)| tid == s)
                .map(|(a, _)| a.clone())
                .unwrap_or_else(|| s.chars().take(8).collect()),
            None => "(identity)".into(),
        }
    };
    let st = &app.config.settings;
    let limit_day = st.limit("calls_per_day").unwrap_or(1000);
    let warn = st.limit("warn_day_remaining").unwrap_or(100);
    let hour_ago = iso_micro(secs - 3600, micros);
    let minute_ago = iso_micro(secs - 60, micros);
    let as_json = app.g.json;
    let db = app.record()?.conn();

    let mut tenants: Vec<Value> = orgs.iter().map(|(_, t)| Value::String(t.clone())).collect();
    {
        let mut q = db.prepare("SELECT DISTINCT tenant_id FROM http_log WHERE at >= ?")?;
        let mut rows = q.query([&since])?;
        while let Some(r) = rows.next()? {
            let v = sql_value(r.get_ref(0)?);
            if !tenants.contains(&v) {
                tenants.push(v);
            }
        }
    }
    let count = |sql: &str, t: &Value, arg: Option<&str>| -> Res<Value> {
        let tv: Option<&str> = t.as_str();
        let v = match arg {
            Some(a) => db.query_row(sql, rusqlite::params![tv, a], |r| {
                Ok(sql_value(r.get_ref(0)?))
            })?,
            None => db.query_row(sql, rusqlite::params![tv], |r| Ok(sql_value(r.get_ref(0)?)))?,
        };
        Ok(v)
    };
    let mut per_org: Vec<Row> = Vec::new();
    for t in &tenants {
        let today_calls = count(
            "SELECT COUNT(*) FROM http_log WHERE tenant_id IS ? AND at >= ?",
            t,
            Some(&today),
        )?;
        let last: Option<(String, Option<i64>)> = {
            let mut q = db.prepare(
                "SELECT at, day_remaining FROM http_log WHERE tenant_id IS ? AND day_remaining IS NOT \
                 NULL ORDER BY id DESC LIMIT 1",
            )?;
            let mut rows = q.query(rusqlite::params![t.as_str()])?;
            match rows.next()? {
                Some(r) => Some((r.get(0)?, r.get(1)?)),
                None => None,
            }
        };
        let remaining = last
            .as_ref()
            .filter(|(at, _)| at.chars().take(10).collect::<String>() == today)
            .and_then(|(_, rem)| *rem);
        let has_tenant = truthy(t);
        let est: Option<i64> = match remaining {
            Some(r) => Some(r),
            None if has_tenant => Some(limit_day - today_calls.as_i64().unwrap_or(0)),
            None => None,
        };
        let row = obj![
            ("org", label(t)),
            (
                "calls_window",
                count(
                    "SELECT COUNT(*) FROM http_log WHERE tenant_id IS ? AND at >= ?",
                    t,
                    Some(&since)
                )?
            ),
            ("calls_today", today_calls),
            (
                "last_hour",
                count(
                    "SELECT COUNT(*) FROM http_log WHERE tenant_id IS ? AND at >= ?",
                    t,
                    Some(&hour_ago)
                )?
            ),
            (
                "last_minute",
                count(
                    "SELECT COUNT(*) FROM http_log WHERE tenant_id IS ? AND at >= ?",
                    t,
                    Some(&minute_ago)
                )?
            ),
            ("day_remaining", est),
            ("day_limit", if has_tenant { Some(limit_day) } else { None }),
            (
                "errors",
                count(
                    "SELECT COUNT(*) FROM http_log WHERE tenant_id IS ? AND at >= ? AND \
                     (status IS NULL OR status >= 400)",
                    t,
                    Some(&since)
                )?
            ),
            (
                "rate_limited",
                count(
                    "SELECT COUNT(*) FROM http_log WHERE tenant_id IS ? AND at >= ? AND status = 429",
                    t,
                    Some(&since)
                )?
            ),
            (
                "last_call",
                count("SELECT MAX(at) FROM http_log WHERE tenant_id IS ?", t, None)?
            ),
            ("warn", est.is_some_and(|e| e <= warn)),
        ];
        // no X-DayLimit-Remaining today: the figure is the local estimate (limit − calls today), marked as one.
        let estimated = remaining.is_none() && est.is_some();
        per_org.push(if estimated {
            mark_estimated(row, as_json)
        } else {
            row
        });
    }

    // endpoints, in first-seen order then by call count (stable).
    let mut eps: Vec<(String, i64, i64, i64, String)> = Vec::new();
    {
        let mut q =
            db.prepare("SELECT path, status, duration_ms, at FROM http_log WHERE at >= ?")?;
        let mut rows = q.query([&since])?;
        while let Some(r) = rows.next()? {
            let path: String = r.get(0)?;
            let status: Option<i64> = r.get(1)?;
            let ms: Option<i64> = r.get(2)?;
            let at: String = r.get(3)?;
            let key = endpoint(&path);
            let idx = match eps.iter().position(|e| e.0 == key) {
                Some(i) => i,
                None => {
                    eps.push((key, 0, 0, 0, String::new()));
                    eps.len() - 1
                }
            };
            let e = &mut eps[idx];
            e.1 += 1;
            e.2 += i64::from(status.is_none_or(|s| s >= 400));
            e.3 += ms.unwrap_or(0);
            if at > e.4 {
                e.4 = at;
            }
        }
    }
    eps.sort_by_key(|e| -e.1);
    let endpoints: Vec<Row> = eps
        .into_iter()
        .map(|(k, calls, errors, ms, last)| {
            obj![
                ("endpoint", k),
                ("calls", calls),
                ("errors", errors),
                (
                    "avg_ms",
                    if calls > 0 {
                        half_even_div(ms, calls)
                    } else {
                        0
                    }
                ),
                ("last", last),
            ]
        })
        .collect();

    let mut syncs: Vec<Row> = Vec::new();
    {
        let mut q = db.prepare(
            "SELECT tenant_id, entity, COUNT(*) AS runs, SUM(fetched) AS records, SUM(new_versions) AS new_versions, \
             SUM(calls) AS calls, SUM(outcome='error') AS failed, MAX(finished_at) AS last \
             FROM sync_runs WHERE finished_at >= ? GROUP BY tenant_id, entity ORDER BY tenant_id, entity",
        )?;
        let names: Vec<String> = q.column_names().iter().map(|s| s.to_string()).collect();
        let mut rows = q.query([&since])?;
        while let Some(r) = rows.next()? {
            let mut m = Row::new();
            for (i, n) in names.iter().enumerate() {
                m.insert(n.clone(), sql_value(r.get_ref(i)?));
            }
            let org = label(&g(&m, "tenant_id"));
            m.insert("org".into(), Value::String(org));
            syncs.push(m);
        }
    }
    let mut by_org: Vec<Row> = Vec::new();
    for r in &syncs {
        let org = g(r, "org");
        let idx = match by_org.iter().position(|o| g(o, "org") == org) {
            Some(i) => i,
            None => {
                by_org.push(obj![
                    ("org", org.clone()),
                    ("entities", 0),
                    ("runs", 0),
                    ("records", 0),
                    ("new_versions", 0),
                    ("calls", 0),
                    ("failed", 0),
                    ("last", ""),
                ]);
                by_org.len() - 1
            }
        };
        let o = &mut by_org[idx];
        let bump = |o: &mut Row, k: &str, by: i64| {
            let cur = o.get(k).and_then(Value::as_i64).unwrap_or(0);
            o.insert(k.into(), Value::from(cur + by));
        };
        bump(o, "entities", 1);
        for k in ["runs", "records", "new_versions", "calls", "failed"] {
            bump(o, k, r.get(k).and_then(Value::as_i64).unwrap_or(0));
        }
        let last = r
            .get("last")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
        if last.as_str() > o.get("last").and_then(Value::as_str).unwrap_or("") {
            o.insert("last".into(), Value::String(last));
        }
    }

    let mut calls: Vec<Row> = Vec::new();
    {
        let mut q = db.prepare(
            "SELECT at, tenant_id, method, path, status, duration_ms, day_remaining, problem FROM http_log \
             WHERE at >= ? ORDER BY id DESC LIMIT ?",
        )?;
        let names: Vec<String> = q.column_names().iter().map(|s| s.to_string()).collect();
        let mut rows = q.query(rusqlite::params![since, recent])?;
        while let Some(r) = rows.next()? {
            let mut m = Row::new();
            for (i, n) in names.iter().enumerate() {
                m.insert(n.clone(), sql_value(r.get_ref(i)?));
            }
            let org = label(&g(&m, "tenant_id"));
            let ep = endpoint(m.get("path").and_then(Value::as_str).unwrap_or(""));
            m.insert("org".into(), Value::String(org));
            m.insert("endpoint".into(), Value::String(ep));
            calls.push(m);
        }
    }
    let one = |sql: &str| -> Res<Value> {
        Ok(db.query_row(sql, [&since], |r| Ok(sql_value(r.get_ref(0)?)))?)
    };
    let errors_total: i64 = per_org
        .iter()
        .map(|o| o.get("errors").and_then(Value::as_i64).unwrap_or(0))
        .sum();
    let totals = obj![
        ("calls", one("SELECT COUNT(*) FROM http_log WHERE at >= ?")?),
        (
            "records_fetched",
            one("SELECT COALESCE(SUM(fetched),0) FROM sync_runs WHERE finished_at >= ?")?
        ),
        (
            "new_versions",
            one("SELECT COUNT(*) FROM versions WHERE observed_at >= ?")?
        ),
        (
            "report_snapshots",
            one("SELECT COUNT(*) FROM reports WHERE fetched_at >= ?")?
        ),
        ("errors", errors_total),
    ];

    let low: Vec<String> = per_org
        .iter()
        .filter(|o| o.get("warn").and_then(Value::as_bool) == Some(true))
        .map(|o| crate::cmds::gs(o, "org"))
        .collect();
    let full = app.g.full || app.g.json;
    let mut out = app.out();
    out.field("since", since.as_str())
        .field("totals", Value::Object(totals));
    out.table(
        "quota",
        &[
            "org",
            "calls_today",
            "day_remaining",
            "day_limit",
            "last_hour",
            "last_minute",
            "calls_window",
            "errors",
            "rate_limited",
            "last_call",
        ],
        per_org,
        None,
    );
    out.table(
        "endpoints",
        &["endpoint", "calls", "errors", "avg_ms", "last"],
        endpoints,
        None,
    );
    if full {
        out.table(
            "syncs",
            &[
                "org",
                "entity",
                "runs",
                "records",
                "new_versions",
                "calls",
                "failed",
                "last",
            ],
            syncs,
            None,
        );
    } else {
        out.table(
            "syncs",
            &[
                "org",
                "entities",
                "runs",
                "records",
                "new_versions",
                "calls",
                "failed",
                "last",
            ],
            by_org,
            None,
        );
    }
    out.table(
        "recent_calls",
        &[
            "at",
            "org",
            "method",
            "endpoint",
            "status",
            "duration_ms",
            "day_remaining",
        ],
        calls,
        None,
    );
    if !low.is_empty() {
        out.hint(format!(
            "Low daily budget: {} — prefer --offline until the window resets.",
            low.join(", ")
        ));
    }
    out.hint("`--days 7` widens the window; `ownpurse tui` shows this live.");
    out.emit();
    Ok(())
}

/// A quota row whose `day_remaining` is the local estimate: `--json` keeps the number and adds
/// `"day_remaining_estimated": true` after it; text shows `≈ 1000 (estimated)`.
fn mark_estimated(row: Row, json: bool) -> Row {
    let mut out = Row::new();
    for (k, v) in row {
        if k == "day_remaining" {
            if json {
                out.insert(k, v);
                out.insert("day_remaining_estimated".into(), Value::Bool(true));
            } else {
                out.insert(
                    k,
                    Value::String(format!("≈ {} (estimated)", crate::out::cell(&v))),
                );
            }
        } else {
            out.insert(k, v);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn endpoints_replace_ids() {
        assert_eq!(
            endpoint("BankTransactions/3f2504e0-4f89-11d3-9a0c-0305e82c3301/History?x=1"),
            "BankTransactions/{id}/History"
        );
        assert_eq!(endpoint("Accounts"), "Accounts");
    }

    #[test]
    fn rounding_and_times() {
        assert_eq!(half_even_div(5, 2), 2);
        assert_eq!(half_even_div(7, 2), 4);
        assert_eq!(half_even_div(10, 3), 3);
        assert_eq!(iso_micro(86_400 + 3661, 5), "1970-01-02T01:01:01.000005Z");
    }
}
