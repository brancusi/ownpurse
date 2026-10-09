//! How many calls a sync would make, from the record alone. Read-only:
//! `sync_runs` (last full run, high water) and `current` counts. Used by the TUI's confirm band and its
//! "Needs attention" lines (`⌃P → Sync roast (≈ 14 calls)`).

use crate::record::{Record, Result};
use rusqlite::{OptionalExtension, params};

/// One Xero entity as the syncer pages it (name, `paged`, `full_extra_passes`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Entity {
    pub name: &'static str,
    pub paged: bool,
    /// Extra full-sweep passes (Invoices: a second pass for DELETED,VOIDED).
    pub full_extra_passes: usize,
}

const fn e(name: &'static str, paged: bool, full_extra_passes: usize) -> Entity {
    Entity {
        name,
        paged,
        full_extra_passes,
    }
}

/// `REFERENCE + TRANSACTIONS`, in sync order (the default set `estimate` covers).
pub const ENTITIES: [Entity; 14] = [
    e("Organisation", false, 0),
    e("Accounts", false, 0),
    e("TaxRates", false, 0),
    e("TrackingCategories", false, 0),
    e("Currencies", false, 0),
    e("Contacts", true, 0),
    e("BankTransactions", true, 0),
    e("BankTransfers", false, 0),
    e("ManualJournals", true, 0),
    e("Invoices", true, 1),
    e("CreditNotes", true, 0),
    e("Payments", true, 0),
    e("Overpayments", true, 0),
    e("Prepayments", true, 0),
];

/// Settings the estimate reads (`sync.page_size`, `sync.full_sweep_days`; defaults 1000 and 7).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EstimateSettings {
    pub page_size: i64,
    pub full_every_seconds: i64,
}

impl Default for EstimateSettings {
    fn default() -> Self {
        EstimateSettings {
            page_size: 1000,
            full_every_seconds: 7 * 86_400,
        }
    }
}

impl EstimateSettings {
    pub fn from_settings(s: &crate::config::Settings) -> EstimateSettings {
        let d = EstimateSettings::default();
        let int = |k: &str| {
            s.get(k).and_then(|v| {
                v.as_i64()
                    .or_else(|| v.as_str().and_then(|t| t.trim().parse().ok()))
            })
        };
        EstimateSettings {
            page_size: int("sync.page_size")
                .filter(|n| *n > 0)
                .unwrap_or(d.page_size),
            full_every_seconds: int("sync.full_sweep_days")
                .map(|days| days * 86_400)
                .unwrap_or(d.full_every_seconds),
        }
    }
}

/// Seconds since the epoch for `YYYY-MM-DDTHH:MM:SS…` (the first 19 characters, UTC).
fn epoch(ts: &str) -> Option<i64> {
    let (y, m, d) = crate::dates::parse_iso(ts)?;
    let t = ts.get(11..19)?;
    let hh: i64 = t.get(0..2)?.parse().ok()?;
    let mm: i64 = t.get(3..5)?.parse().ok()?;
    let ss: i64 = t.get(6..8)?.parse().ok()?;
    Some(crate::dates::days_from_civil(y, m, d) * 86_400 + hh * 3600 + mm * 60 + ss)
}

/// Calls a sync of `entities` (default: all fourteen) would make at `now` (seconds since the epoch):
/// 1 per unpaged table; for a paged table 1 when an incremental run is due (a full run within
/// `full_every` and a high water mark), else `ceil(records / page_size)` (at least 1) plus its extra passes.
/// `None` when the org has never synced successfully (no basis for an estimate). `full` forces full sweeps.
pub fn estimate(
    record: &Record,
    tenant_id: &str,
    entities: Option<&[Entity]>,
    full: bool,
    settings: EstimateSettings,
    now: i64,
) -> Result<Option<i64>> {
    let conn = record.conn();
    let synced: Option<i64> = conn
        .query_row(
            "SELECT 1 FROM sync_runs WHERE tenant_id=?1 AND outcome='ok' LIMIT 1",
            params![tenant_id],
            |r| r.get(0),
        )
        .optional()?;
    if synced.is_none() {
        return Ok(None);
    }
    let mut total = 0i64;
    for e in entities.unwrap_or(&ENTITIES) {
        if !e.paged {
            total += 1;
            continue;
        }
        let last_full: Option<String> = conn
            .query_row(
                "SELECT finished_at FROM sync_runs WHERE tenant_id=?1 AND entity=?2 AND outcome='ok' \
                 AND mode='full' ORDER BY finished_at DESC LIMIT 1",
                params![tenant_id, e.name],
                |r| r.get(0),
            )
            .optional()?;
        let hw: Option<String> = conn.query_row(
            "SELECT MAX(high_water) FROM sync_runs WHERE tenant_id=?1 AND entity=?2 AND outcome='ok'",
            params![tenant_id, e.name],
            |r| r.get(0),
        )?;
        let recent = last_full
            .as_deref()
            .and_then(epoch)
            .is_some_and(|t| now - t < settings.full_every_seconds);
        if !full && recent && hw.is_some() {
            total += 1;
            continue;
        }
        let n: i64 = conn.query_row(
            "SELECT COUNT(*) FROM current WHERE tenant_id=?1 AND entity=?2",
            params![tenant_id, e.name],
            |r| r.get(0),
        )?;
        let pages = (n + settings.page_size - 1) / settings.page_size;
        total += pages.max(1) + e.full_extra_passes as i64;
    }
    Ok(Some(total))
}

/// What to show before a sync: the estimate, or (where there is none, an org never synced) an explicit
/// floor. Shared by the CLI and the TUI.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Estimate {
    /// The estimate from the record.
    Calls(i64),
    /// Never synced: with no records each table costs at least 1 call, plus its extra full passes. A minimum,
    /// shown as `≈ N+ calls (first full sync)`; a big org pages more.
    FirstSyncFloor { calls: i64, subset: bool },
}

impl Estimate {
    /// The figure (for sums across orgs).
    pub fn calls(self) -> i64 {
        match self {
            Estimate::Calls(n) | Estimate::FirstSyncFloor { calls: n, .. } => n,
        }
    }
    /// The budget rule: refuse only when this figure exceeds the calls left (a floor is a minimum, so it is
    /// refused only when even the minimum does not fit).
    pub fn exceeds(self, left: i64) -> bool {
        self.calls() > left
    }
    /// `≈ 15 calls`, `≈ 15+ calls (first full sync)`, `≈ 3+ calls (first sync, 1+ per entity)`.
    pub fn label(self) -> String {
        let calls = |n: i64| if n == 1 { "call" } else { "calls" };
        match self {
            Estimate::Calls(n) => format!("≈ {n} {}", calls(n)),
            Estimate::FirstSyncFloor {
                calls: n,
                subset: false,
            } => {
                format!("≈ {n}+ {} (first full sync)", calls(n))
            }
            Estimate::FirstSyncFloor {
                calls: n,
                subset: true,
            } => {
                format!("≈ {n}+ {} (first sync, 1+ per entity)", calls(n))
            }
        }
    }
}

/// The floor for a never-synced org: 1 per table plus extra full passes (15 for all fourteen tables).
pub fn first_sync_floor(entities: Option<&[Entity]>) -> i64 {
    entities
        .unwrap_or(&ENTITIES)
        .iter()
        .map(|e| 1 + e.full_extra_passes as i64)
        .sum()
}

/// `estimate`, or the first-sync floor where there is no estimate.
pub fn estimate_or_floor(
    record: &Record,
    tenant_id: &str,
    entities: Option<&[Entity]>,
    full: bool,
    settings: EstimateSettings,
    now: i64,
) -> Result<Estimate> {
    Ok(
        match estimate(record, tenant_id, entities, full, settings, now)? {
            Some(n) => Estimate::Calls(n),
            None => Estimate::FirstSyncFloor {
                calls: first_sync_floor(entities),
                subset: entities.is_some_and(|e| e.len() < ENTITIES.len()),
            },
        },
    )
}

/// The entities named (case-insensitive), in the order given; `None` for an unknown name.
pub fn entities_named(names: &[&str]) -> Option<Vec<Entity>> {
    names
        .iter()
        .map(|n| {
            ENTITIES
                .iter()
                .copied()
                .find(|e| e.name.eq_ignore_ascii_case(n))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_sync_floor_full_subset_and_labels() {
        assert_eq!(
            first_sync_floor(None),
            15,
            "14 tables + Invoices' extra pass"
        );
        let sub = entities_named(&["accounts", "Invoices", "BankTransactions"]).unwrap();
        assert_eq!(first_sync_floor(Some(&sub)), 4);
        assert_eq!(entities_named(&["Nope"]), None);
        assert_eq!(
            Estimate::FirstSyncFloor {
                calls: 15,
                subset: false
            }
            .label(),
            "≈ 15+ calls (first full sync)"
        );
        assert_eq!(
            Estimate::FirstSyncFloor {
                calls: 4,
                subset: true
            }
            .label(),
            "≈ 4+ calls (first sync, 1+ per entity)"
        );
        assert_eq!(Estimate::Calls(1).label(), "≈ 1 call");
        assert_eq!(Estimate::Calls(15).label(), "≈ 15 calls");
    }

    #[test]
    fn floor_is_refused_only_when_the_minimum_does_not_fit() {
        let f = Estimate::FirstSyncFloor {
            calls: 15,
            subset: false,
        };
        assert!(!f.exceeds(15), "exactly enough is allowed");
        assert!(!f.exceeds(1000));
        assert!(f.exceeds(14), "the minimum itself does not fit");
        assert!(Estimate::Calls(16).exceeds(15));
    }

    #[test]
    fn epoch_parses_record_timestamps() {
        assert_eq!(epoch("1970-01-02T00:00:01"), Some(86_401));
        assert_eq!(
            epoch("2026-10-05T07:46:00.123Z"),
            epoch("2026-10-05T07:46:00")
        );
        assert_eq!(epoch("2026-10-05"), None);
    }

    #[test]
    fn entity_table_is_the_sync_table() {
        assert_eq!(ENTITIES.iter().filter(|e| e.paged).count(), 8);
        assert_eq!(
            ENTITIES.iter().map(|e| e.full_extra_passes).sum::<usize>(),
            1
        );
    }
}
