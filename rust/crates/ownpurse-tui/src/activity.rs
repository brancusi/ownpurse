//! What the Activity tab (spec §9 row 4, merge M2) reads from the record: the HTTP log (`http_log`, newest
//! first, at most `LOG_ROWS`), today's totals and the busiest minute per org. Read-only and local; built
//! with the model (start and `r`), so drawing and filtering never touch SQLite.

use crate::clock::{self, parse_utc};
use crate::model::OrgInfo;
use ownpurse_core::Record;

/// The `tui.log_rows` default (the setting is honoured: `Source::log_rows`).
pub const LOG_ROWS: i64 = 500;
/// `limits.minute_soft_limit` and `limits.calls_per_minute` defaults.
pub const MINUTE_SOFT: i64 = 55;
pub const MINUTE_HARD: i64 = 60;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StatusClass {
    /// 1xx–3xx: dim (`muted`).
    Ok,
    /// 4xx except 429: `warn`.
    Client,
    /// 429, 5xx, no status (network problem) or a problem on a success: `error`.
    Error,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct LogRow {
    pub id: i64,
    pub at: String,
    pub tenant: Option<String>,
    /// The org alias, the tenant id's first 8 characters when not configured, or `(identity)`.
    pub org: String,
    pub method: String,
    pub path: String,
    pub endpoint: String,
    pub status: Option<i64>,
    pub ms: Option<i64>,
    pub day_left: Option<i64>,
    pub min_left: Option<i64>,
    pub problem: Option<String>,
    /// Lower-cased text the `/` filter searches (prepared at load).
    pub hay: String,
}

impl LogRow {
    /// An agent bridge row (read, screenshot, send; in memory only, never a Xero call).
    pub fn is_agent(&self) -> bool {
        self.method == "agent"
    }

    /// A write attempt (Phase 8; `problem` holds its state).
    pub fn is_write(&self) -> bool {
        self.method == "write"
    }

    pub fn class(&self) -> StatusClass {
        if self.is_agent() {
            return if self.path.contains("refused") {
                StatusClass::Client
            } else {
                StatusClass::Ok
            };
        }
        if self.is_write() {
            return match self.problem.as_deref() {
                Some("confirmed" | "reconciled") => StatusClass::Ok,
                Some("rejected") => StatusClass::Client,
                _ => StatusClass::Error,
            };
        }
        match self.status {
            None => StatusClass::Error,
            Some(429) => StatusClass::Error,
            Some(s) if s >= 500 => StatusClass::Error,
            Some(s) if s >= 400 => StatusClass::Client,
            Some(_) if self.problem.is_some() => StatusClass::Error,
            Some(_) => StatusClass::Ok,
        }
    }

    /// The status cell: the code, or the problem (`—` when neither).
    pub fn status_text(&self) -> String {
        if self.is_agent() {
            return if self.path.contains("refused") {
                "refused".into()
            } else {
                "ok".into()
            };
        }
        if self.is_write() {
            return self.problem.clone().unwrap_or_default();
        }
        match (self.status, &self.problem) {
            (Some(s), _) => s.to_string(),
            (None, Some(p)) => p.clone(),
            (None, None) => "—".into(),
        }
    }

    pub fn is_error(&self) -> bool {
        if self.is_write() {
            return self.class() != StatusClass::Ok;
        }
        !self.is_agent() && self.status.is_none_or(|s| s >= 400)
    }

    /// `07:45:12` today; `10-04 18:02:11` (or `18:02:11` when `with_date` is false) on other days.
    pub fn time(&self, today: &str, with_date: bool) -> String {
        let hms = self.at.get(11..19).unwrap_or("").to_string();
        if self.at.get(..10) == Some(today) || !with_date {
            hms
        } else {
            format!("{} {hms}", self.at.get(5..10).unwrap_or(""))
        }
    }

    /// Lower-cased text the `/` filter searches.
    pub fn haystack(&self) -> String {
        if self.is_write() {
            // a write attempt: its op, kind and package title are searchable too
            return format!(
                "{} {} write {} {} {}",
                self.at,
                self.org,
                self.endpoint,
                self.path,
                self.status_text()
            )
            .to_lowercase();
        }
        format!(
            "{} {} {} {} {} {}",
            self.at,
            self.org,
            self.method,
            self.path,
            self.status_text(),
            self.problem.as_deref().unwrap_or("")
        )
        .to_lowercase()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EndpointRow {
    pub endpoint: String,
    pub calls: i64,
    pub errors: i64,
    pub avg_ms: i64,
    pub last: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Activity {
    /// Newest first.
    pub log: Vec<LogRow>,
    /// Calls in the last minute before `now`, per configured org (model order).
    pub last_minute: Vec<i64>,
    pub minute_soft: i64,
    pub minute_hard: i64,
    /// Load generation: view caches key on it, so `r` (a new load) invalidates them.
    pub generation: u64,
}

static GENERATION: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);

/// A fresh generation number for a load (shared by Activity and System).
pub fn next_generation() -> u64 {
    GENERATION.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
}

fn is_uuid(s: &str) -> bool {
    let b = s.as_bytes();
    b.len() == 36
        && b.iter().enumerate().all(|(i, c)| match i {
            8 | 13 | 18 | 23 => *c == b'-',
            _ => c.is_ascii_hexdigit(),
        })
}

/// `BankTransactions/3f…/History?x=1` → `BankTransactions/{id}/History`.
pub fn endpoint(path: &str) -> String {
    let path = path.split('?').next().unwrap_or("");
    path.split('/')
        .map(|seg| if is_uuid(seg) { "{id}" } else { seg })
        .collect::<Vec<_>>()
        .join("/")
}

/// Endpoint statistics over `rows` (calls, errors, average ms rounded half up, latest call), busiest first.
pub fn by_endpoint<'a>(rows: impl IntoIterator<Item = &'a LogRow>) -> Vec<EndpointRow> {
    let mut out: Vec<(EndpointRow, i64)> = Vec::new();
    for r in rows {
        let i = match out.iter().position(|(e, _)| e.endpoint == r.endpoint) {
            Some(i) => i,
            None => {
                out.push((
                    EndpointRow {
                        endpoint: r.endpoint.clone(),
                        calls: 0,
                        errors: 0,
                        avg_ms: 0,
                        last: String::new(),
                    },
                    0,
                ));
                out.len() - 1
            }
        };
        let (e, ms) = &mut out[i];
        e.calls += 1;
        e.errors += i64::from(r.is_error());
        *ms += r.ms.unwrap_or(0);
        if r.at > e.last {
            e.last = r.at.clone();
        }
    }
    let mut rows: Vec<EndpointRow> = out
        .into_iter()
        .map(|(mut e, ms)| {
            e.avg_ms = (2 * ms + e.calls) / (2 * e.calls);
            e
        })
        .collect();
    rows.sort_by(|a, b| b.calls.cmp(&a.calls).then(a.endpoint.cmp(&b.endpoint)));
    rows
}

impl Activity {
    pub fn load(record: &Record, orgs: &[OrgInfo], now: i64, rows: i64) -> Activity {
        let label = |t: &Option<String>| match t {
            None => "(identity)".to_string(),
            Some(t) => orgs
                .iter()
                .find(|o| &o.tenant == t)
                .map(|o| o.alias.clone())
                .unwrap_or_else(|| t.chars().take(8).collect()),
        };
        let mut log = Vec::new();
        if let Ok(mut stmt) = record.conn().prepare(
            "SELECT id, at, tenant_id, method, path, status, duration_ms, day_remaining, min_remaining, problem \
             FROM http_log ORDER BY id DESC LIMIT ?1",
        ) && let Ok(rows) = stmt.query_map((rows.max(1),), |r| {
            Ok((
                r.get::<_, i64>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, Option<String>>(2)?,
                r.get::<_, String>(3)?,
                r.get::<_, String>(4)?,
                r.get::<_, Option<i64>>(5)?,
                r.get::<_, Option<i64>>(6)?,
                r.get::<_, Option<i64>>(7)?,
                r.get::<_, Option<i64>>(8)?,
                r.get::<_, Option<String>>(9)?,
            ))
        }) {
            for (id, at, tenant, method, path, status, ms, day_left, min_left, problem) in
                rows.flatten()
            {
                let mut row = LogRow {
                    id,
                    org: label(&tenant),
                    endpoint: endpoint(&path),
                    at,
                    tenant,
                    method,
                    path,
                    status,
                    ms,
                    day_left,
                    min_left,
                    problem,
                    hay: String::new(),
                };
                row.hay = row.haystack();
                log.push(row);
            }
        }
        // every write attempt (Phase 8), as its own row: the op, its kind, its state, the Xero id it wrote
        let writes = ownpurse_core::write::view::attempts(record, rows);
        let merge = !writes.is_empty();
        for a in writes {
            let tenant = Some(a.tenant.clone());
            let what = match &a.attempt {
                Some(x) => format!("{} {}", a.kind, x),
                None => a.kind.clone(),
            };
            // the endpoint column: the op and what it did; the path (detail): its document and package
            let ep = match &a.package {
                Some((title, _)) => format!("{} {what} · {title}", a.op_id()),
                None => format!("{} {what}", a.op_id()),
            };
            let mut path = a.op_key.clone();
            if let Some(t) = &a.target_id {
                path.push_str(&format!(" · {t}"));
            }
            if let Some((_, id)) = &a.package {
                path.push_str(&format!(" · XA:{id}:{}", a.op_id()));
            }
            let mut row = LogRow {
                // negative: never the id of an http_log row
                id: -a.id,
                org: label(&tenant),
                endpoint: ep,
                at: a.at.clone(),
                tenant,
                method: "write".into(),
                path,
                status: a.http_status,
                ms: None,
                day_left: None,
                min_left: None,
                problem: Some(a.state.clone()),
                hay: String::new(),
            };
            row.hay = row.haystack();
            log.push(row);
        }
        if merge {
            log.sort_by(|a, b| b.at.cmp(&a.at).then(b.id.abs().cmp(&a.id.abs())));
            log.truncate(rows.max(1) as usize);
        }
        let last_minute = orgs
            .iter()
            .map(|o| {
                log.iter()
                    .filter(|r| r.tenant.as_deref() == Some(o.tenant.as_str()))
                    .filter(|r| parse_utc(&r.at).is_some_and(|t| t > now - 60 && t <= now))
                    .count() as i64
            })
            .collect();
        Activity {
            log,
            last_minute,
            minute_soft: MINUTE_SOFT,
            minute_hard: MINUTE_HARD,
            generation: next_generation(),
        }
    }

    /// Today's date (UTC) for `now`.
    pub fn today(now: i64) -> String {
        clock::date_of(now)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(id: i64, path: &str, status: Option<i64>, ms: i64, problem: Option<&str>) -> LogRow {
        LogRow {
            id,
            at: format!("2026-10-05T07:45:{:02}.000000Z", id % 60),
            tenant: None,
            org: "roast".into(),
            method: "GET".into(),
            path: path.into(),
            endpoint: endpoint(path),
            status,
            ms: Some(ms),
            day_left: None,
            min_left: None,
            problem: problem.map(str::to_string),
            hay: String::new(),
        }
    }

    #[test]
    fn endpoints_normalise_ids_and_queries() {
        assert_eq!(
            endpoint("BankTransactions/c0ffee00-0000-4000-8000-0000000000a1/History?page=2"),
            "BankTransactions/{id}/History"
        );
        assert_eq!(
            endpoint("Reports/TrialBalance?date=2025-12-31"),
            "Reports/TrialBalance"
        );
        assert_eq!(endpoint("connect/token"), "connect/token");
    }

    #[test]
    fn status_classes() {
        assert_eq!(row(1, "a", Some(200), 1, None).class(), StatusClass::Ok);
        assert_eq!(
            row(1, "a", Some(404), 1, Some("Not Found")).class(),
            StatusClass::Client
        );
        assert_eq!(row(1, "a", Some(429), 1, None).class(), StatusClass::Error);
        assert_eq!(row(1, "a", Some(503), 1, None).class(), StatusClass::Error);
        assert_eq!(
            row(1, "a", None, 1, Some("timeout")).class(),
            StatusClass::Error
        );
        assert_eq!(
            row(1, "a", None, 1, Some("timeout")).status_text(),
            "timeout"
        );
        assert_eq!(row(1, "a", None, 1, None).status_text(), "—");
    }

    #[test]
    fn grouping_counts_errors_and_rounds_half_up() {
        let rows = [
            row(1, "Accounts", Some(200), 100, None),
            row(2, "Accounts", Some(500), 101, None),
            row(3, "Organisation", Some(200), 7, None),
        ];
        let g = by_endpoint(rows.iter());
        assert_eq!(g[0].endpoint, "Accounts");
        assert_eq!((g[0].calls, g[0].errors, g[0].avg_ms), (2, 1, 101));
        assert_eq!(g[0].last, rows[1].at);
        assert_eq!((g[1].calls, g[1].errors, g[1].avg_ms), (1, 0, 7));
    }
}
