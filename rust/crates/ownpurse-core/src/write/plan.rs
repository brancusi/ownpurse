//! `ownpurse write plan <ops.json>`: compile the requested changes against the local record into a package.
//! Every operation is built by [`ops`] (exact request, preconditions, expected effects, recovery). The freeze
//! gate holds frozen entity-years (`branches.accountant_gate`) and prior periods unless the package carries
//! the accountant's reference; a create that would duplicate a journal already in the record is refused.
//! The plan reads only the record (no network); the run re-checks every precondition live.
//!
//! The input:
//! ```json
//! {"id": "fixes-2025-v1", "title": "2025 owner-paid corrections",
//!  "accountant_decision": "AJE-2 (the accountant's email of 2026-10-01)",
//!  "ops": [
//!   {"kind": "create_journal", "id": "op-1", "org": "design", "date": "2025-12-31",
//!    "narration": "…", "cash_basis": true,
//!    "lines": [{"account": "6290", "amount": "263.94", "description": "…"},
//!              {"account": "2430", "amount": "-263.94"}],
//!    "decisions": [{"id": "18", "content": {…}}]},
//!   {"kind": "recode_bank_line", "id": "op-2", "org": "lab", "bank_transaction": "<id>",
//!    "line_item": "<LineItemID>", "account": "5020"},
//!   {"kind": "void_journal", "id": "op-3", "org": "lab", "journal": "<ManualJournalID>"},
//!   {"kind": "attach", "id": "op-4", "org": "design", "journal": "op:op-1", "file": "support.csv"}],
//!  "excluded": [{"what": "…", "why": "…"}],
//!  "checklist": ["…"]}
//! ```

use super::ops::{self, AttachTo, JournalSpec, OrgCtx, RecodeSpec, usage};
use super::{
    Excluded, JournalLine, JournalRef, Op, Operation, PACKAGE_V, Package, Precondition, SourceRef,
    WriteError, exit, gate, sha256_hex,
};
use crate::canon::canonical;
use crate::dates::{fiscal_year_start, xero_date};
use crate::money;
use crate::record::{Record, Row};
use serde::Deserialize;
use serde_json::Value;
use std::collections::BTreeSet;
use std::path::Path;

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InputRef {
    pub id: String,
    #[serde(default)]
    pub hash: Option<String>,
    #[serde(default)]
    pub content: Option<Value>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InputLine {
    pub account: String,
    pub amount: String,
    #[serde(default)]
    pub description: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum InputOp {
    CreateJournal {
        id: String,
        org: String,
        date: String,
        narration: String,
        /// Required: `ShowOnCashBasisReports` is never left to the API default.
        cash_basis: bool,
        lines: Vec<InputLine>,
        #[serde(default)]
        max_lines: Option<usize>,
        #[serde(default)]
        split_account: Option<String>,
        /// Milestone 1 only: the approval knowingly covers a deliberate re-send of this create under its own
        /// Idempotency-Key (`write probe-replay`, Demo Company only); flagged `probe_replay` in the review.
        #[serde(default)]
        probe_replay: bool,
        #[serde(default)]
        decisions: Vec<InputRef>,
        #[serde(default)]
        sources: Vec<InputRef>,
        #[serde(default)]
        depends_on: Vec<String>,
    },
    RecodeBankLine {
        id: String,
        org: String,
        bank_transaction: String,
        /// The line's `LineItemID` (never a position).
        line_item: String,
        #[serde(default)]
        account: Option<String>,
        #[serde(default)]
        description: Option<String>,
        #[serde(default)]
        decisions: Vec<InputRef>,
        #[serde(default)]
        sources: Vec<InputRef>,
        #[serde(default)]
        depends_on: Vec<String>,
    },
    VoidJournal {
        id: String,
        org: String,
        journal: String,
        #[serde(default)]
        decisions: Vec<InputRef>,
        #[serde(default)]
        sources: Vec<InputRef>,
        #[serde(default)]
        depends_on: Vec<String>,
    },
    /// A SPEND or RECEIVE bank transaction (cash, petty cash: entries with no bank feed).
    CreateBankTransaction {
        id: String,
        org: String,
        /// `SPEND` or `RECEIVE`.
        #[serde(rename = "type")]
        kind: String,
        /// The bank account: code, AccountID or exact name of a BANK account.
        bank_account: String,
        /// A ContactID, an exact contact name (resolved in the record; never created implicitly), or
        /// `op:<id>` for the contact an earlier op creates.
        contact: String,
        date: String,
        #[serde(default)]
        reference: String,
        /// Positive amounts (the type gives the direction).
        lines: Vec<InputLine>,
        /// The same bank, date, amount and contact already in the record is refused unless set.
        #[serde(default)]
        allow_duplicate: bool,
        #[serde(default)]
        decisions: Vec<InputRef>,
        #[serde(default)]
        sources: Vec<InputRef>,
        #[serde(default)]
        depends_on: Vec<String>,
    },
    /// A transfer between two of the org's bank accounts (petty cash funding). Permanent history: the recovery
    /// is a reverse transfer.
    CreateBankTransfer {
        id: String,
        org: String,
        /// Bank accounts: code, AccountID or exact name.
        from: String,
        to: String,
        amount: String,
        date: String,
        #[serde(default)]
        reference: String,
        /// The same accounts, amount and date already in the record is refused unless set.
        #[serde(default)]
        allow_duplicate: bool,
        #[serde(default)]
        decisions: Vec<InputRef>,
        #[serde(default)]
        sources: Vec<InputRef>,
        #[serde(default)]
        depends_on: Vec<String>,
    },
    /// A contact (name, optional email); refused when one has exactly this name.
    CreateContact {
        id: String,
        org: String,
        name: String,
        #[serde(default)]
        email: Option<String>,
        #[serde(default)]
        decisions: Vec<InputRef>,
        #[serde(default)]
        sources: Vec<InputRef>,
        #[serde(default)]
        depends_on: Vec<String>,
    },
    /// Archive a contact, by id or `op:<id>`.
    ArchiveContact {
        id: String,
        org: String,
        contact: String,
        #[serde(default)]
        decisions: Vec<InputRef>,
        #[serde(default)]
        sources: Vec<InputRef>,
        #[serde(default)]
        depends_on: Vec<String>,
    },
    /// Void an unreconciled bank transaction (Status DELETED), by id or `op:<id>`.
    VoidBankTransaction {
        id: String,
        org: String,
        bank_transaction: String,
        #[serde(default)]
        decisions: Vec<InputRef>,
        #[serde(default)]
        sources: Vec<InputRef>,
        #[serde(default)]
        depends_on: Vec<String>,
    },
    /// A note in a document's history (`PUT {Entity}/{id}/History`): permanent.
    AddNote {
        id: String,
        org: String,
        /// BankTransactions, ManualJournals, BankTransfers or Contacts.
        entity: String,
        /// The document's id, or `op:<id>` for the one an earlier op of this package creates.
        document: String,
        /// At most 250 characters.
        details: String,
        #[serde(default)]
        decisions: Vec<InputRef>,
        #[serde(default)]
        sources: Vec<InputRef>,
        #[serde(default)]
        depends_on: Vec<String>,
    },
    Attach {
        id: String,
        org: String,
        /// The document's id (a ManualJournalID, or a BankTransactionID with `entity`), or `op:<id>` for the
        /// journal an earlier op of this package creates.
        journal: String,
        /// `ManualJournals` (the default) or `BankTransactions`.
        #[serde(default)]
        entity: Option<String>,
        /// Relative to the input file's folder.
        file: String,
        #[serde(default)]
        decisions: Vec<InputRef>,
        #[serde(default)]
        sources: Vec<InputRef>,
        #[serde(default)]
        depends_on: Vec<String>,
    },
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PlanInput {
    pub id: String,
    #[serde(default)]
    pub title: String,
    /// The accountant's reference covering every gated operation.
    #[serde(default)]
    pub accountant_decision: Option<String>,
    pub ops: Vec<InputOp>,
    #[serde(default)]
    pub excluded: Vec<Excluded>,
    #[serde(default)]
    pub checklist: Vec<String>,
    #[serde(default)]
    pub supersedes: Vec<String>,
    /// Overrides the profile's `history_notes` for this package (the `history-note` probe sets it).
    #[serde(default)]
    pub history_notes: Option<bool>,
}

/// What the plan needs from the configuration.
pub struct PlanConfig {
    pub profile: String,
    /// alias → tenant id (the active profile's orgs).
    pub orgs: Vec<(String, String)>,
    /// `branches.accountant_gate`: `[(org, through)]`.
    pub gates: Vec<(String, String)>,
    /// Each landed operation adds `ownpurse package <hash12> op <id>: <why>` to its document's history
    /// (the profile's `history_notes`; on by default for real profiles, off for a Demo Company unless set).
    pub history_notes: bool,
}

/// Seal each operation's audit note into the package (`history_note`: the package title and the op's
/// summary); an [`Op::AddNote`] is itself a note and gets none.
pub fn with_history_notes(ops: &mut [Operation], title: &str) {
    for o in ops {
        if matches!(o.op, Op::AddNote(_)) {
            continue;
        }
        let what = super::export::summary(o);
        let why = if title.trim().is_empty() {
            what
        } else {
            format!("{} - {what}", title.trim())
        };
        o.history_note = Some(super::cut_note(&why));
    }
}

impl PlanConfig {
    fn tenant(&self, org: &str) -> Result<&str, WriteError> {
        self.orgs
            .iter()
            .find(|(a, _)| a == org)
            .map(|(_, t)| t.as_str())
            .ok_or_else(|| {
                usage(format!(
                    "{org} is not an org of the profile {}",
                    self.profile
                ))
            })
    }
}

pub fn load_input(path: &Path) -> Result<PlanInput, WriteError> {
    let text = std::fs::read_to_string(path)
        .map_err(|e| WriteError::new("io", exit::RUNTIME, format!("{}: {e}", path.display())))?;
    serde_json::from_str(&text).map_err(|e| usage(format!("{}: {e}", path.display())))
}

fn rec<T>(r: crate::record::Result<T>) -> Result<T, WriteError> {
    r.map_err(|e| WriteError::new("record", exit::RUNTIME, e.to_string()))
}

fn find(record: &Record, tenant: &str, entity: &str, id: &str) -> Result<Row, WriteError> {
    rec(record.current_rows(tenant, entity))?
        .into_iter()
        .find(|r| r.xero_id == id)
        .ok_or_else(|| {
            usage(format!("{entity} {id} is not in the record"))
                .help("sync the org first (`ownpurse sync <org>`)")
        })
}

/// A contact for a bank transaction: a ContactID or an exact name in the record (one, ACTIVE), or `op:<id>`
/// for the contact an earlier op of this package creates. A name is never sent to Xero (Xero would create a
/// new contact for a name it doesn't know).
pub(crate) fn resolve_contact(
    record: &Record,
    tenant: &str,
    out: &[Operation],
    org: &str,
    id: &str,
    contact: &str,
) -> Result<JournalRef, WriteError> {
    if let Some(op) = contact.strip_prefix("op:") {
        if !out.iter().any(|x| x.id == op && x.org == org) {
            return Err(usage(format!(
                "{id}: op:{op} is not an earlier op of this package on {org}"
            )));
        }
        return Ok(JournalRef::CreatedBy(op.into()));
    }
    let rows = rec(record.current_rows(tenant, "Contacts"))?;
    if rows.iter().any(|r| r.xero_id == contact) {
        return Ok(JournalRef::Id(contact.into()));
    }
    let named: Vec<&crate::record::Row> = rows
        .iter()
        .filter(|r| r.payload.get("Name").and_then(Value::as_str) == Some(contact))
        .collect();
    match named.as_slice() {
        [one] if one.payload.get("ContactStatus").and_then(Value::as_str) != Some("ARCHIVED") => {
            Ok(JournalRef::Id(one.xero_id.clone()))
        }
        [_] => Err(usage(format!("{id}: the contact {contact:?} is archived"))),
        [] => Err(usage(format!(
            "{id}: no contact {contact:?} in the record (an id or an exact name)"
        ))
        .help(
            "sync Contacts, or create it first in this package (`create_contact`, then `op:<id>`)",
        )),
        _ => Err(usage(format!(
            "{id}: {} contacts are named {contact:?}: use the ContactID",
            named.len()
        ))),
    }
}

/// The duplicate guard of a bank transaction create (with the marker, checked live): the same bank account,
/// date, total and contact already in the record.
fn bank_duplicate_of(
    record: &Record,
    tenant: &str,
    o: &Operation,
) -> Result<Option<String>, WriteError> {
    let Op::CreateBankTransaction(b) = &o.op else {
        return Ok(None);
    };
    let JournalRef::Id(contact) = &b.contact else {
        return Ok(None);
    };
    let total = b.total();
    for r in rec(record.current_rows(tenant, "BankTransactions"))? {
        let p = &r.payload;
        let same = p.get("Status").and_then(Value::as_str) == Some("AUTHORISED")
            && p.get("Type").and_then(Value::as_str) == Some(b.kind.as_str())
            && p.pointer("/BankAccount/AccountID").and_then(Value::as_str)
                == Some(b.bank_account_id.as_str())
            && p.pointer("/Contact/ContactID").and_then(Value::as_str) == Some(contact.as_str())
            && crate::dates::doc_date(p) == b.date
            && money::money(p.get("Total")).ok() == Some(total);
        if same {
            return Ok(Some(format!(
                "{} {} {} on {}",
                r.xero_id,
                b.kind,
                super::money_text(total),
                b.date
            )));
        }
    }
    Ok(None)
}

/// The duplicate guard of a transfer create (with the marker, checked live): the same accounts, amount and
/// date already in the record.
fn transfer_duplicate_of(
    record: &Record,
    tenant: &str,
    o: &Operation,
) -> Result<Option<String>, WriteError> {
    let Op::CreateBankTransfer(t) = &o.op else {
        return Ok(None);
    };
    for r in rec(record.current_rows(tenant, "BankTransfers"))? {
        let p = &r.payload;
        let same = p
            .pointer("/FromBankAccount/AccountID")
            .and_then(Value::as_str)
            == Some(t.from_account_id.as_str())
            && p.pointer("/ToBankAccount/AccountID")
                .and_then(Value::as_str)
                == Some(t.to_account_id.as_str())
            && crate::dates::doc_date(p) == t.date
            && money::money(p.get("Amount")).ok() == Some(t.amount);
        if same {
            return Ok(Some(format!(
                "{} {} on {}",
                r.xero_id,
                super::money_text(t.amount),
                t.date
            )));
        }
    }
    Ok(None)
}

/// The later of the org's period and end-of-year lock dates.
pub fn lock_date(record: &Record, tenant: &str) -> Result<Option<String>, WriteError> {
    let org = rec(record.organisation(tenant))?;
    let d = |k: &str| {
        org.get(k)
            .and_then(Value::as_str)
            .map(|s| xero_date(s).unwrap_or_else(|| s.chars().take(10).collect()))
            .filter(|s| s.len() == 10)
    };
    Ok([d("PeriodLockDate"), d("EndOfYearLockDate")]
        .into_iter()
        .flatten()
        .max())
}

/// Prior period: before the financial year containing the org's latest cached trial balance.
pub fn prior_before(record: &Record, tenant: &str) -> Result<Option<String>, WriteError> {
    let latest: Option<String> = record
        .conn()
        .query_row(
            // sync-cached trial balances only: a write run's evidence snapshots (`phase`, `xa_package`) and
            // any implausible date (past today + 1 year, e.g. an old sentinel row) never set the period
            "SELECT max(json_extract(params, '$.date')) FROM reports \
             WHERE tenant_id = ?1 AND report = 'TrialBalance' \
               AND json_extract(params, '$.phase') IS NULL AND json_extract(params, '$.xa_package') IS NULL \
               AND json_extract(params, '$.date') <= date('now', '+1 year')",
            [tenant],
            |r| r.get(0),
        )
        .map_err(|e| WriteError::new("record", exit::RUNTIME, e.to_string()))?;
    let Some(latest) = latest else {
        return Ok(None);
    };
    let (m, d) = rec(record.fy_end(tenant))?;
    Ok(fiscal_year_start(&latest, m, d))
}

fn refs(rs: &[InputRef]) -> Result<Vec<SourceRef>, WriteError> {
    rs.iter()
        .map(|r| {
            let content_hash = match (&r.hash, &r.content) {
                (Some(h), None) => h.clone(),
                (None, Some(c)) => sha256_hex(canonical(c).as_bytes()),
                _ => {
                    return Err(usage(format!(
                        "reference {}: give exactly one of hash or content",
                        r.id
                    )));
                }
            };
            Ok(SourceRef {
                id: r.id.clone(),
                content_hash,
            })
        })
        .collect()
}

/// An existing POSTED journal this create would duplicate: one carrying its marker, or one with the same
/// date, accounts and amounts.
fn duplicate_of(record: &Record, o: &Operation) -> Result<Option<String>, WriteError> {
    let Op::CreateJournal(j) = &o.op else {
        return Ok(None);
    };
    let marker = o.preconditions.iter().find_map(|p| match p {
        Precondition::MarkerAbsent { marker } => Some(marker.clone()),
        _ => None,
    });
    let shape = |lines: &[JournalLine]| {
        let mut v: Vec<(String, String)> = lines
            .iter()
            .map(|l| (l.account_code.clone(), super::money_text(l.amount)))
            .collect();
        v.sort();
        v
    };
    let want = shape(&j.lines);
    for r in rec(record.current_rows(&o.tenant_id, "ManualJournals"))? {
        if r.payload.get("Status").and_then(Value::as_str) != Some("POSTED") {
            continue;
        }
        let narr = r
            .payload
            .get("Narration")
            .and_then(Value::as_str)
            .unwrap_or("");
        if let Some(m) = &marker
            && narr.contains(m.as_str())
        {
            return Ok(Some(format!("{} carries the marker {m}", r.xero_id)));
        }
        let lines: Vec<JournalLine> = r
            .payload
            .get("JournalLines")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .map(|l| JournalLine {
                account_code: l
                    .get("AccountCode")
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    .into(),
                description: String::new(),
                amount: money::money(l.get("LineAmount")).unwrap_or_default(),
            })
            .collect();
        if crate::dates::doc_date(&r.payload) == j.date && shape(&lines) == want {
            return Ok(Some(format!(
                "{} has the same date, accounts and amounts",
                r.xero_id
            )));
        }
    }
    Ok(None)
}

fn id_ok(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 64
        && s.chars()
            .all(|c| c.is_ascii_alphanumeric() || "-_.".contains(c))
}

/// Compile `input` into an unsealed package. `base_dir` resolves attachment paths; `now` is the ISO time.
pub fn plan(
    record: &Record,
    cfg: &PlanConfig,
    input: &PlanInput,
    base_dir: &Path,
    now: &str,
) -> Result<Package, WriteError> {
    if !id_ok(&input.id) {
        return Err(usage(
            "id: 1–64 letters, digits, '-', '_' or '.' (it is part of every marker)",
        ));
    }
    if input.ops.is_empty() {
        return Err(usage("the input has no operations"));
    }
    let mut out: Vec<Operation> = vec![];
    for o in &input.ops {
        let (id, org, ds, ss, deps) = match o {
            InputOp::CreateJournal {
                id,
                org,
                decisions,
                sources,
                depends_on,
                ..
            }
            | InputOp::RecodeBankLine {
                id,
                org,
                decisions,
                sources,
                depends_on,
                ..
            }
            | InputOp::VoidJournal {
                id,
                org,
                decisions,
                sources,
                depends_on,
                ..
            }
            | InputOp::Attach {
                id,
                org,
                decisions,
                sources,
                depends_on,
                ..
            }
            | InputOp::AddNote {
                id,
                org,
                decisions,
                sources,
                depends_on,
                ..
            }
            | InputOp::CreateBankTransaction {
                id,
                org,
                decisions,
                sources,
                depends_on,
                ..
            }
            | InputOp::VoidBankTransaction {
                id,
                org,
                decisions,
                sources,
                depends_on,
                ..
            }
            | InputOp::CreateBankTransfer {
                id,
                org,
                decisions,
                sources,
                depends_on,
                ..
            }
            | InputOp::CreateContact {
                id,
                org,
                decisions,
                sources,
                depends_on,
                ..
            }
            | InputOp::ArchiveContact {
                id,
                org,
                decisions,
                sources,
                depends_on,
                ..
            } => (id, org, decisions, sources, depends_on),
        };
        if !id_ok(id) || id.contains('.') {
            return Err(usage(format!(
                "op id {id:?}: letters, digits, '-' and '_' ('.' is reserved for split parts)"
            )));
        }
        if out
            .iter()
            .any(|x| x.id == *id || x.id.starts_with(&format!("{id}.")))
        {
            return Err(usage(format!("op id {id} is used twice")));
        }
        let tenant = cfg.tenant(org)?.to_string();
        let ctx = OrgCtx {
            package_id: input.id.clone(),
            org: org.clone(),
            tenant_id: tenant.clone(),
            accounts: rec(crate::view::View::main(record).payloads(&tenant, "Accounts"))?,
            lock: lock_date(record, &tenant)?,
            short_code: crate::deeplink::short_code(record, &tenant),
        };
        let built: Vec<Operation> = match o {
            InputOp::CreateJournal {
                date,
                narration,
                cash_basis,
                lines,
                max_lines,
                split_account,
                probe_replay,
                ..
            } => {
                let lines = lines
                    .iter()
                    .map(|l| {
                        Ok(JournalLine {
                            account_code: l.account.clone(),
                            amount: money::parse(&l.amount).map_err(|_| {
                                usage(format!("{id}: {:?} is not an amount", l.amount))
                            })?,
                            description: l.description.clone(),
                        })
                    })
                    .collect::<Result<Vec<_>, WriteError>>()?;
                let v = ops::create_journal(
                    &ctx,
                    &JournalSpec {
                        id: id.clone(),
                        date: date.clone(),
                        narration: narration.clone(),
                        cash_basis: *cash_basis,
                        lines,
                        max_lines: *max_lines,
                        split_account: split_account.clone(),
                    },
                )?;
                let mut v = v;
                if *probe_replay {
                    for op in &mut v {
                        op.flags.push("probe_replay".into());
                    }
                }
                for op in &v {
                    if let Some(why) = duplicate_of(record, op)? {
                        return Err(WriteError::new(
                            "precondition",
                            exit::PRECONDITION,
                            format!(
                                "{}: it would duplicate a journal already in Xero: {why}",
                                op.id
                            ),
                        ));
                    }
                }
                v
            }
            InputOp::RecodeBankLine {
                bank_transaction,
                line_item,
                account,
                description,
                ..
            } => {
                let row = find(record, &tenant, "BankTransactions", bank_transaction)?;
                vec![ops::recode_bank_line(
                    &ctx,
                    &row,
                    &RecodeSpec {
                        id: id.clone(),
                        line_item_id: line_item.clone(),
                        account: account.clone(),
                        description: description.clone(),
                    },
                )?]
            }
            InputOp::VoidJournal { journal, .. } => {
                if let Some(op) = journal.strip_prefix("op:") {
                    let create = created_by(&out, op, org, id)?;
                    vec![ops::void_created(&ctx, id, create)?]
                } else {
                    let row = find(record, &tenant, "ManualJournals", journal)?;
                    vec![ops::void_journal(&ctx, id, &row)?]
                }
            }
            InputOp::CreateBankTransaction {
                kind,
                bank_account,
                contact,
                date,
                reference,
                lines,
                allow_duplicate,
                ..
            } => {
                let contact = resolve_contact(record, &tenant, &out, org, id, contact)?;
                let lines = lines
                    .iter()
                    .map(|l| {
                        Ok(JournalLine {
                            account_code: l.account.clone(),
                            amount: money::parse(&l.amount).map_err(|_| {
                                usage(format!("{id}: {:?} is not an amount", l.amount))
                            })?,
                            description: l.description.clone(),
                        })
                    })
                    .collect::<Result<Vec<_>, WriteError>>()?;
                let o = ops::create_bank_transaction(
                    &ctx,
                    &ops::BankTxnSpec {
                        id: id.clone(),
                        kind: kind.clone(),
                        bank_account: bank_account.clone(),
                        contact,
                        date: date.clone(),
                        reference: reference.clone(),
                        lines,
                    },
                )?;
                if !*allow_duplicate && let Some(why) = bank_duplicate_of(record, &tenant, &o)? {
                    return Err(WriteError::new(
                        "precondition",
                        exit::PRECONDITION,
                        format!("{id}: it would duplicate a bank transaction already in Xero: {why}"),
                    )
                    .help("set \"allow_duplicate\": true if it really is a second, separate transaction"));
                }
                vec![o]
            }
            InputOp::CreateBankTransfer {
                from,
                to,
                amount,
                date,
                reference,
                allow_duplicate,
                ..
            } => {
                let amount = money::parse(amount)
                    .map_err(|_| usage(format!("{id}: {amount:?} is not an amount")))?;
                let o = ops::create_bank_transfer(&ctx, id, from, to, amount, date, reference)?;
                if !*allow_duplicate && let Some(why) = transfer_duplicate_of(record, &tenant, &o)?
                {
                    return Err(WriteError::new(
                        "precondition",
                        exit::PRECONDITION,
                        format!("{id}: it would duplicate a transfer already in Xero: {why}"),
                    )
                    .help(
                        "set \"allow_duplicate\": true if it really is a second, separate transfer",
                    ));
                }
                vec![o]
            }
            InputOp::CreateContact { name, email, .. } => {
                let taken = rec(record.current_rows(&tenant, "Contacts"))?
                    .into_iter()
                    .find(|r| r.payload.get("Name").and_then(Value::as_str) == Some(name.trim()));
                if let Some(r) = taken {
                    return Err(usage(format!(
                        "{id}: a contact named {:?} exists ({}); Xero names are unique",
                        name.trim(),
                        r.xero_id
                    ))
                    .help("use it by name or id in the bank transaction"));
                }
                if out.iter().any(|x| {
                    matches!(&x.op, Op::CreateContact(c) if c.name == name.trim()) && x.org == *org
                }) {
                    return Err(usage(format!(
                        "{id}: this package already creates {:?}",
                        name.trim()
                    )));
                }
                vec![ops::create_contact(&ctx, id, name, email.as_deref())?]
            }
            InputOp::ArchiveContact { contact, .. } => {
                let target = match contact.strip_prefix("op:") {
                    Some(op) => {
                        if !out.iter().any(|x| x.id == op && x.org == *org) {
                            return Err(usage(format!(
                                "{id}: op:{op} is not an earlier op of this package on {org}"
                            )));
                        }
                        JournalRef::CreatedBy(op.into())
                    }
                    None => {
                        find(record, &tenant, "Contacts", contact)?;
                        JournalRef::Id(contact.clone())
                    }
                };
                vec![ops::archive_contact(&ctx, id, target)]
            }
            InputOp::VoidBankTransaction {
                bank_transaction, ..
            } => match bank_transaction.strip_prefix("op:") {
                Some(op) => {
                    let create = out
                        .iter()
                        .find(|x| x.id == op && x.org == *org)
                        .ok_or_else(|| {
                            usage(format!(
                                "{id}: op:{op} is not an earlier op of this package on {org}"
                            ))
                        })?;
                    vec![ops::void_created_bank(&ctx, id, create)?]
                }
                None => {
                    let row = find(record, &tenant, "BankTransactions", bank_transaction)?;
                    vec![ops::void_bank_transaction(&ctx, id, &row)?]
                }
            },
            InputOp::AddNote {
                entity,
                document,
                details,
                ..
            } => {
                let target = match document.strip_prefix("op:") {
                    Some(op) => {
                        if !out.iter().any(|x| x.id == op && x.org == *org) {
                            return Err(usage(format!(
                                "{id}: op:{op} is not an earlier op of this package on {org}"
                            )));
                        }
                        JournalRef::CreatedBy(op.into())
                    }
                    None => {
                        let e = super::NOTE_ENTITIES
                            .iter()
                            .find(|e| e.eq_ignore_ascii_case(entity))
                            .copied()
                            .unwrap_or("");
                        if !e.is_empty() {
                            find(record, &tenant, e, document)?;
                        }
                        JournalRef::Id(document.clone())
                    }
                };
                vec![ops::add_note(&ctx, id, entity, target, details)?]
            }
            InputOp::Attach {
                journal,
                file,
                entity,
                ..
            } => {
                let path = base_dir.join(file);
                let entity = match entity.as_deref() {
                    None => "ManualJournals",
                    Some(e) => super::ATTACH_ENTITIES
                        .iter()
                        .copied()
                        .find(|x| x.eq_ignore_ascii_case(e))
                        .ok_or_else(|| {
                            usage(format!(
                                "{id}: files attach to {}, not {e}",
                                super::ATTACH_ENTITIES.join(" or ")
                            ))
                        })?,
                };
                if entity == "BankTransactions" {
                    let row = find(record, &tenant, entity, journal)?;
                    vec![ops::attach(
                        &ctx,
                        id,
                        AttachTo::BankTransaction(&row),
                        &path,
                    )?]
                } else if let Some(op) = journal.strip_prefix("op:") {
                    created_by(&out, op, org, id)?;
                    let voided = out.iter().any(|x| {
                        matches!(&x.op, Op::VoidJournal { journal: JournalRef::CreatedBy(c) } if c == op)
                    });
                    if voided {
                        return Err(usage(format!(
                            "{id}: op:{op} is voided earlier in this package; attach before the void"
                        )));
                    }
                    vec![ops::attach(
                        &ctx,
                        id,
                        AttachTo::CreatedBy(op.into()),
                        &path,
                    )?]
                } else {
                    let row = find(record, &tenant, "ManualJournals", journal)?;
                    vec![ops::attach(&ctx, id, AttachTo::Journal(&row), &path)?]
                }
            }
        };
        let decision_refs = refs(ds)?;
        let source_refs = refs(ss)?;
        let prior = prior_before(record, &tenant)?;
        for mut op in built {
            for d in deps {
                if !out.iter().any(|x| x.id == *d) {
                    return Err(usage(format!(
                        "{}: depends on {d}, which is not an earlier op",
                        op.id
                    )));
                }
                if !op.depends_on.contains(d) {
                    op.depends_on.push(d.clone());
                }
            }
            op.decision_refs = decision_refs.clone();
            op.source_refs = source_refs.clone();
            op.gated = ops::effect_date(&op)
                .and_then(|d| gate::needs_accountant(org, d, &cfg.gates, prior.as_deref()));
            out.push(op);
        }
    }
    // the freeze gate: a gated op needs the accountant's reference
    let gated: Vec<String> = out
        .iter()
        .filter_map(|o| o.gated.as_ref().map(|g| format!("{}: {g}", o.id)))
        .collect();
    let reference = input
        .accountant_decision
        .as_deref()
        .map(str::trim)
        .filter(|r| !r.is_empty());
    if !gated.is_empty() {
        match reference {
            Some(r) if gate::reference_ok(r) => {}
            _ => {
                let mut e = WriteError::new(
                    "freeze_gate",
                    exit::USAGE,
                    format!(
                        "{} operation{} change a frozen or prior period, and the package has no accountant's reference",
                        gated.len(),
                        if gated.len() == 1 { "" } else { "s" }
                    ),
                )
                .help("add \"accountant_decision\": \"<the accountant's reference: an adjusting-entry id, an email, a decisions row>\" to the input");
                for g in gated {
                    e = e.help(g);
                }
                return Err(e);
            }
        }
    }
    // lock-date reminders: every (org, year) the package corrects in a frozen or prior period
    let mut checklist = input.checklist.clone();
    let years: BTreeSet<(String, String)> = out
        .iter()
        .filter(|o| o.gated.is_some())
        .filter_map(|o| ops::effect_date(o).map(|d| (o.org.clone(), d.chars().take(4).collect())))
        .collect();
    for (org, y) in years {
        let step = format!(
            "after posting the {y} corrections, set the {y} lock date for {org} in Xero (browser; the API can't set lock dates)"
        );
        if !checklist.contains(&step) {
            checklist.push(step);
        }
    }
    if input.history_notes.unwrap_or(cfg.history_notes) {
        with_history_notes(&mut out, &input.title);
    }
    Ok(Package {
        v: PACKAGE_V,
        id: input.id.clone(),
        title: if input.title.is_empty() {
            input.id.clone()
        } else {
            input.title.clone()
        },
        profile: cfg.profile.clone(),
        created_at: now.into(),
        accountant_decision: reference.map(str::to_string),
        ops: out,
        excluded: input.excluded.clone(),
        checklist,
        supersedes: input.supersedes.clone(),
        source: None,
        seal: None,
    })
}

/// The earlier, unsplit journal create of `org` that `op:<op>` names (attach and void may target it).
fn created_by<'a>(
    out: &'a [Operation],
    op: &str,
    org: &str,
    id: &str,
) -> Result<&'a Operation, WriteError> {
    out.iter()
        .find(|x| x.id == op && x.org == org && matches!(x.op, Op::CreateJournal(_)))
        .ok_or_else(|| {
            usage(format!(
                "{id}: op:{op} is not an earlier (unsplit) journal create of {org} in this package"
            ))
        })
}
