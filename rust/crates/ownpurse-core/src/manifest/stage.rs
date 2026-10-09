//! The lifecycle (design §5): `draft → review → check → planned → approved → applying → applied`, derived,
//! never stored: from the manifest (entries, acknowledgements, its plan), its check, and what the record
//! says about the planned package (approvals, write attempts, read-backs). An entry is POSTED only when the
//! record's read-back equals its "after" (the view then skips it); a landed write that hasn't been read back
//! keeps its proposed marks.

use super::Manifest;
use super::check::Report;
use super::compile::op_id;
use crate::record::Record;
use crate::view::Overlay;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Stage {
    /// No entries yet.
    Draft,
    /// Entries with warnings not yet acknowledged.
    Review,
    /// Errors: planning is blocked.
    CheckFailed {
        errors: usize,
    },
    /// Checked clean: ready to plan.
    Checked,
    Planned,
    Approved,
    Applying {
        done: usize,
        total: usize,
    },
    /// A write was rejected or is uncertain: stopped at that entry.
    Stopped {
        at: i64,
        landed: usize,
        rejected: usize,
        not_sent: usize,
        verified: usize,
    },
    /// Every entry posted (read back): read-only.
    Applied,
}

impl Stage {
    /// The index in the strip (draft 0 … applied 6).
    pub fn index(&self) -> usize {
        match self {
            Stage::Draft => 0,
            Stage::Review => 1,
            Stage::CheckFailed { .. } | Stage::Checked => 2,
            Stage::Planned => 3,
            Stage::Approved => 4,
            Stage::Applying { .. } | Stage::Stopped { .. } => 5,
            Stage::Applied => 6,
        }
    }
    pub fn word(&self) -> &'static str {
        STAGES[self.index()]
    }
    /// The header chip's short form (after the manifest name), and whether it is an error.
    pub fn short(&self, entries: usize, issues: usize) -> (String, bool) {
        match self {
            Stage::Draft | Stage::Review | Stage::Checked => {
                let mut s = format!("{entries} change{}", if entries == 1 { "" } else { "s" });
                if issues > 0 {
                    s.push_str(&format!(
                        " · {issues} issue{}",
                        if issues == 1 { "" } else { "s" }
                    ));
                }
                (s, false)
            }
            Stage::CheckFailed { errors } => (format!("check ✗ {errors}"), true),
            Stage::Planned => ("planned".into(), false),
            Stage::Approved => ("approved".into(), false),
            Stage::Applying { done, total } => (format!("applying {done}/{total}"), false),
            Stage::Stopped { .. } => ("stopped ✗".into(), true),
            Stage::Applied => ("applied ✓".into(), false),
        }
    }
    /// Read-only (applied).
    pub fn read_only(&self) -> bool {
        matches!(self, Stage::Applied)
    }
}

pub const STAGES: [&str; 7] = [
    "draft", "review", "check", "planned", "approved", "applying", "applied",
];

/// One entry's write status while a planned package applies.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EntryStatus {
    /// Not part of an apply (no plan, or the plan is stale).
    None,
    Queued,
    Sending,
    /// Xero accepted it (`MJ 9f2c…` when known); read-back pending.
    Landed(Option<String>),
    /// The record's read-back equals the entry's "after".
    Posted,
    Rejected(String),
    Uncertain,
    /// Carried out in the browser (a checklist step): posted once a sync reads the edit back.
    Manual,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ApplyCounts {
    pub total: usize,
    pub sent: usize,
    pub landed: usize,
    pub verified: usize,
    pub rejected: usize,
    pub uncertain: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Status {
    pub stage: Stage,
    /// A plan exists but the manifest changed after it (a new plan is needed).
    pub plan_stale: bool,
    pub apply: ApplyCounts,
    /// Per active entry.
    pub entries: Vec<(i64, EntryStatus)>,
}

impl Status {
    pub fn of(&self, seq: i64) -> &EntryStatus {
        self.entries
            .iter()
            .find(|(s, _)| *s == seq)
            .map_or(&EntryStatus::None, |(_, e)| e)
    }
}

/// The stage and per-entry status of a manifest.
pub fn status(record: &Record, m: &Manifest, report: &Report, overlay: Option<&Overlay>) -> Status {
    let active: Vec<i64> = m.active().map(|e| e.seq).collect();
    let posted = |s: i64| overlay.is_some_and(|o| o.is_posted(s));
    let Some(plan) = m.plan() else {
        let stage = if active.is_empty() {
            Stage::Draft
        } else if report.errors() > 0 {
            Stage::CheckFailed {
                errors: report.errors(),
            }
        } else if report.unacked() > 0 {
            Stage::Review
        } else {
            Stage::Checked
        };
        return Status {
            stage,
            plan_stale: m.planned.is_some(),
            apply: ApplyCounts::default(),
            entries: active.iter().map(|s| (*s, EntryStatus::None)).collect(),
        };
    };
    let h12 = &plan.hash[..plan.hash.len().min(12)];
    let states = crate::write::view::op_states(record, h12);
    let mut counts = ApplyCounts {
        total: active.len(),
        ..Default::default()
    };
    let mut entries = vec![];
    let mut stopped_at = None;
    for s in &active {
        let id = op_id(*s);
        let applied = overlay.and_then(|o| o.applied(*s));
        // a reconciled line's recode rides on its org-month reclass journal
        let reclass = applied.and_then(|a| {
            matches!(a.matched, super::rules::Matched::Op("reclass_journal"))
                .then(|| super::compile::reclass_op_id(&a.org, &a.date))
        });
        let browser = applied
            .is_some_and(|a| matches!(a.matched, super::rules::Matched::Op("browser_recode")));
        let mine: Vec<&(String, String, serde_json::Value)> = states
            .iter()
            .filter(|(k, _, _)| {
                k.split_once(':').is_some_and(|(_, o)| {
                    o == id
                        || o.starts_with(&format!("{id}-"))
                        || o.starts_with(&format!("{id}."))
                        || reclass
                            .as_deref()
                            .is_some_and(|r| o == r || o.starts_with(&format!("{r}.")))
                })
            })
            .collect();
        let st = if posted(*s) {
            EntryStatus::Posted
        } else if browser {
            EntryStatus::Manual
        } else if let Some((_, _, d)) = mine.iter().find(|(_, st, _)| st == "rejected") {
            let msg = d["message"]
                .as_str()
                .or_else(|| d["error"].as_str())
                .or_else(|| d["note"].as_str())
                .unwrap_or("refused")
                .to_string();
            EntryStatus::Rejected(msg)
        } else if mine.iter().any(|(_, st, _)| st == "uncertain") {
            EntryStatus::Uncertain
        } else if mine.iter().any(|(_, st, _)| st == "sending") {
            EntryStatus::Sending
        } else if !mine.is_empty()
            && mine
                .iter()
                .all(|(_, st, d)| st == "confirmed" || (st == "reconciled" && d["landed"] == true))
        {
            EntryStatus::Landed(
                mine.iter()
                    .find_map(|(_, _, d)| d["target_id"].as_str().map(str::to_string)),
            )
        } else {
            EntryStatus::Queued
        };
        match &st {
            EntryStatus::Posted => {
                counts.verified += 1;
                counts.sent += 1;
            }
            EntryStatus::Landed(_) => {
                counts.landed += 1;
                counts.sent += 1;
            }
            EntryStatus::Rejected(_) => {
                counts.rejected += 1;
                counts.sent += 1;
                stopped_at.get_or_insert(*s);
            }
            EntryStatus::Uncertain => {
                counts.uncertain += 1;
                counts.sent += 1;
                stopped_at.get_or_insert(*s);
            }
            EntryStatus::Sending => counts.sent += 1,
            _ => {}
        }
        entries.push((*s, st));
    }
    let stage = if let Some(at) = stopped_at {
        Stage::Stopped {
            at,
            landed: counts.landed,
            rejected: counts.rejected + counts.uncertain,
            not_sent: counts.total - counts.sent,
            verified: counts.verified,
        }
    } else if counts.total > 0 && counts.verified == counts.total {
        Stage::Applied
    } else if counts.sent > 0 {
        Stage::Applying {
            done: counts.landed + counts.verified,
            total: counts.total,
        }
    } else if crate::write::view::approval_for(record, &plan.hash).is_some() {
        Stage::Approved
    } else {
        Stage::Planned
    };
    Status {
        stage,
        plan_stale: false,
        apply: counts,
        entries,
    }
}
