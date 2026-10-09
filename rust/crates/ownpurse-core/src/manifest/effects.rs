//! What a manifest changes against main (the effects, design §3): per org, the trial-balance accounts that
//! move, the statement totals and the suspense / must-be-zero accounts; the intercompany residuals; and the
//! facts the write path will face. Computed by the SAME ledger engine that reads the books (the view's
//! document-level overlay), so nothing here re-implements accounting.

use super::{Manifest, ManifestError, Result};
use crate::ledger::Ledger;
use crate::record::Record;
use crate::view::{Ctx, View};
use rust_decimal::Decimal;
use serde_json::Value;
use std::collections::BTreeMap;

/// One account whose balance differs between main and the manifest.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccountDiff {
    pub account_id: String,
    pub code: String,
    pub name: String,
    pub class: String,
    pub main: Decimal,
    pub manifest: Decimal,
}

impl AccountDiff {
    pub fn delta(&self) -> Decimal {
        self.manifest - self.main
    }
}

/// One org's effect: TB rows that move, the statement totals, and its suspense / must-be-zero accounts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrgDiff {
    pub org: String,
    pub tenant: String,
    pub accounts: Vec<AccountDiff>,
    /// (label, main, manifest) for assets, liabilities, equity, revenue, expenses, net profit.
    pub totals: Vec<(&'static str, Decimal, Decimal)>,
    /// Suspense and must-be-zero accounts: every one, moved or not.
    pub zero: Vec<AccountDiff>,
}

/// One intercompany pair's residual on main and with the manifest.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeDiff {
    pub pair: String,
    pub label: String,
    pub main: Option<Decimal>,
    pub manifest: Option<Decimal>,
}

/// The facts the write path will face (what a plan compiles to).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PlanFacts {
    pub journals: usize,
    pub voids: usize,
    pub recodes: usize,
    pub attachments: usize,
    pub not_writable: usize,
    /// Recodes of reconciled lines carried out by the monthly reclass journals.
    pub reclasses: usize,
    /// Recodes of reconciled lines left to the browser (checklist steps).
    pub browser_steps: usize,
    pub reconciled_line_edits: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diff {
    pub manifest: String,
    pub as_of: String,
    pub entries: usize,
    pub conflicts: usize,
    pub needs_accountant: usize,
    pub orgs: Vec<OrgDiff>,
    pub bridges: Vec<BridgeDiff>,
    pub facts: PlanFacts,
}

/// Balances on the default basis (Xero opening + activity, as the Ledger shows them).
fn basis_balances(
    record: &Record,
    ledger: &Ledger,
    tid: &str,
    as_of: &str,
) -> std::result::Result<BTreeMap<String, Decimal>, ManifestError> {
    let fy = record.fy_end(tid)?;
    crate::balances::resolve(record, ledger, tid, as_of, fy, crate::balances::Mode::Auto)
        .map(|b| b.balances)
        .map_err(|e| ManifestError::new("record", e.to_string()))
}

fn is_zero_account(a: &crate::ledger::Account, extra: &[String]) -> bool {
    a.system == "SUSPENSE" || a.name.to_lowercase().contains("suspense") || extra.contains(&a.code)
}

/// What the manifest changes against main at `as_of`: per org, the TB accounts that move, the statement
/// totals and the suspense accounts (`zero_accounts`: alias → extra must-be-zero codes); the intercompany
/// residuals (`ic_config` as the CLI reads it: `{parties, intercompany}`), and the plan facts.
pub fn diff(
    record: &Record,
    ctx: &Ctx,
    manifest: &Manifest,
    as_of: &str,
    ic_config: Option<&Value>,
    zero_accounts: &BTreeMap<String, Vec<String>>,
) -> Result<Diff> {
    let main = View::main(record);
    let view = View::open(record, ctx, Some(manifest))?;
    let ov = view.overlay().expect("a manifest view");
    let mut orgs = vec![];
    for (alias, tid) in &ctx.orgs {
        let lm =
            Ledger::build(&main, tid).map_err(|e| ManifestError::new("record", e.to_string()))?;
        let lb =
            Ledger::build(&view, tid).map_err(|e| ManifestError::new("record", e.to_string()))?;
        let bm = basis_balances(record, &lm, tid, as_of)?;
        let bb = basis_balances(record, &lb, tid, as_of)?;
        let mut ids: Vec<&String> = bm.keys().chain(bb.keys()).collect();
        ids.sort();
        ids.dedup();
        let row = |id: &String| {
            let a = lb.chart.get(id);
            AccountDiff {
                account_id: id.clone(),
                code: a.code.clone(),
                name: a.name.clone(),
                class: a.class.clone(),
                main: bm.get(id).copied().unwrap_or_default(),
                manifest: bb.get(id).copied().unwrap_or_default(),
            }
        };
        let accounts: Vec<AccountDiff> = ids
            .iter()
            .map(|id| row(id))
            .filter(|r| r.main != r.manifest)
            .collect();
        let extra = zero_accounts.get(alias).cloned().unwrap_or_default();
        let mut zero: Vec<AccountDiff> = lb
            .chart
            .order
            .iter()
            .filter(|id| is_zero_account(&lb.chart.get(id), &extra))
            .map(row)
            .collect();
        zero.sort_by(|a, b| a.code.cmp(&b.code));
        let sum = |bal: &BTreeMap<String, Decimal>, class: &str| -> Decimal {
            bal.iter()
                .filter(|(id, _)| lb.chart.get(id).class == class)
                .map(|(_, v)| *v)
                .sum()
        };
        let mut totals = vec![];
        for (label, class, sign) in [
            ("assets", "ASSET", 1),
            ("liabilities", "LIABILITY", -1),
            ("equity", "EQUITY", -1),
            ("revenue", "REVENUE", -1),
            ("expenses", "EXPENSE", 1),
        ] {
            let s = Decimal::from(sign);
            totals.push((label, s * sum(&bm, class), s * sum(&bb, class)));
        }
        let net = |bal: &BTreeMap<String, Decimal>| -(sum(bal, "REVENUE") + sum(bal, "EXPENSE"));
        totals.push(("net", net(&bm), net(&bb)));
        if !accounts.is_empty() || ov.applied.iter().any(|a| &a.tenant == tid) || !zero.is_empty() {
            orgs.push(OrgDiff {
                org: alias.clone(),
                tenant: tid.clone(),
                accounts,
                totals,
                zero,
            });
        }
    }
    let mut bridges = vec![];
    if let Some(cfg) = ic_config {
        let tenants: std::collections::HashMap<String, String> = ctx.orgs.iter().cloned().collect();
        let run = |v: View<'_>| -> Option<Value> {
            let c = crate::intercompany::IcContext::new(v, tenants.clone(), cfg, as_of);
            crate::intercompany::build(&c, None).ok()
        };
        if let (Some(m), Some(b)) = (run(View::main(record)), run(view.clone())) {
            let empty = vec![];
            for pm in m["pairs"].as_array().unwrap_or(&empty) {
                let id = pm["id"].as_str().unwrap_or("").to_string();
                let pb = b["pairs"]
                    .as_array()
                    .and_then(|a| a.iter().find(|p| p["id"] == pm["id"]));
                // what is left to explain: nothing when the sides match, else the bridge's residual
                let d = |p: &Value| -> Option<Decimal> {
                    if p["status"] == "matches" {
                        return Some(Decimal::ZERO);
                    }
                    let v = &p["residual"];
                    crate::money::money(Some(v)).ok().filter(|_| !v.is_null())
                };
                bridges.push(BridgeDiff {
                    label: pm["label"].as_str().unwrap_or(&id).to_string(),
                    pair: id,
                    main: d(pm),
                    manifest: pb.and_then(d),
                });
            }
        }
    }
    let mut facts = PlanFacts::default();
    for a in &ov.applied {
        match &a.matched {
            super::rules::Matched::Op("create_journal") => facts.journals += 1,
            super::rules::Matched::Op("void_journal") => facts.voids += 1,
            super::rules::Matched::Op("void_recreate") => {
                facts.voids += 1;
                facts.journals += 1;
            }
            super::rules::Matched::Op("recode_bank_line") => facts.recodes += 1,
            super::rules::Matched::Op("reclass_journal") => facts.reclasses += 1,
            super::rules::Matched::Op("browser_recode") => facts.browser_steps += 1,
            super::rules::Matched::Unresolved { .. } => facts.not_writable += 1,
            super::rules::Matched::Op(_) => facts.attachments += 1,
            super::rules::Matched::NotWritable { .. } => facts.not_writable += 1,
        }
        if a.reconciled_line && matches!(a.matched, super::rules::Matched::Op("recode_bank_line")) {
            facts.reconciled_line_edits += 1;
        }
    }
    Ok(Diff {
        manifest: manifest.name.clone(),
        as_of: as_of.into(),
        entries: manifest.active().count(),
        conflicts: ov.conflicts.len(),
        needs_accountant: ov.needs_accountant(),
        orgs,
        bridges,
        facts,
    })
}
