//! Phase 8: the Xero write path (v0.13.0). This module is the shared INTERFACE between the package side
//! (plan, seal, export, freeze gate, approvals, op builders: `plan`, `ops`, `delta`, `gate`, `seal`, `export`,
//! `pdf`, `approval`) and the run side (the api crate's write transport, guards, attempt log, run, verify,
//! reconcile, recover and probes). Types only, plus the few pure helpers both sides must agree on byte for
//! byte (the marker, the idempotency key, the allow-list).
//!
//! The design is binding: notes/PHASE8-WRITE-PATH-DESIGN.md. In short:
//! - a [`Package`] is a list of [`Operation`]s, each one [`Op`] (A: create a manual journal, B: recode a
//!   bank-transaction line, C: void a manual journal, D: attach a file to a manual journal) with its live
//!   [`Precondition`]s, its [`ExpectedDelta`]s and its pre-compiled recovery. A package is branch-ready: the
//!   merge step of book branches is this same list of operations with preconditions.
//! - seal = sha256(canon::canonical(package JSON without its `seal` field)); approvals bind to that hash
//!   ([`ApprovalRecord`], stored through [`ApprovalStore`] in the record's append-only `approvals` table).
//! - the run logs every [`Attempt`] BEFORE sending, in `op_events` (chained), and sends through a
//!   [`WriteTransport`] that refuses anything outside [`ALLOWED`].
//!
//! Nothing here performs I/O against Xero. The core never links an HTTP client; the real transport lives in
//! the api crate and the TUI never links that (V1).

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};

/// The package format version.
pub const PACKAGE_V: u32 = 1;

/// At most this many operations per `write run` invocation.
pub const BATCH_MAX: usize = 10;

/// The one-run environment lock: `OWNPURSE_WRITE_RUN=<package hash>` for that one invocation.
pub const RUN_ENV: &str = "OWNPURSE_WRITE_RUN";

// ── the package ─────────────────────────────────────────────────────────────────────────────────────────

/// A write package: what will be done, checked and expected, frozen by its seal. Canonical JSON on disk.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Package {
    /// [`PACKAGE_V`].
    pub v: u32,
    /// A stable id chosen at plan time (`roast-2025-owner-paid-03`); part of every marker.
    pub id: String,
    pub title: String,
    /// The profile it was planned for (the run refuses another).
    pub profile: String,
    /// When `write plan` produced it (ISO UTC).
    pub created_at: String,
    /// The freeze gate's reference (the accountant's adjusting-entry email id, a `decisions` row id) when any operation touches
    /// a frozen entity-year or a prior period; shown in the review export.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub accountant_decision: Option<String>,
    /// The operations, in dependency order.
    pub ops: Vec<Operation>,
    /// Deferred or unknown items: listed for the reviewer, never executable.
    #[serde(default)]
    pub excluded: Vec<Excluded>,
    /// Manual steps after the run (e.g. "set the 2025 lock date in Xero (browser)").
    #[serde(default)]
    pub checklist: Vec<String>,
    /// The package versions this one replaces (an amendment is a new package, a new hash, a new approval).
    #[serde(default)]
    pub supersedes: Vec<String>,
    /// The manifest or branch this package was compiled from, when there is one (absent: written by hand).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    /// sha256 of the canonical package without this field; None until `write seal`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub seal: Option<String>,
}

/// One operation of a package.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Operation {
    /// Unique in the package (`op-01`); with the package id it makes the marker and the idempotency key.
    pub id: String,
    /// The org alias and the tenant it must reach (the tenant guard checks both).
    pub org: String,
    pub tenant_id: String,
    pub op: Op,
    /// The exact request, built at plan time (the run sends this and nothing else).
    pub request: WriteRequest,
    /// Checked live, per operation, right before sending.
    pub preconditions: Vec<Precondition>,
    /// The target's canonical fingerprint after the write (read-back verification); None for a create
    /// (verified through the marker and [`Operation::expected`]).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expected_after: Option<String>,
    /// Report movements this operation causes (every other row must not move).
    #[serde(default)]
    pub expected: Vec<ExpectedDelta>,
    /// The inverse, pre-compiled for review; running it is its own package with its own approval.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recovery: Option<Box<Recovery>>,
    /// Earlier operations of this package this one needs (a blocked dependency stops it).
    #[serde(default)]
    pub depends_on: Vec<String>,
    /// The decisions it implements, with their content hashes.
    #[serde(default)]
    pub decision_refs: Vec<SourceRef>,
    /// The source rows behind it (bank lines, statement rows), with their business-content hashes.
    #[serde(default)]
    pub source_refs: Vec<SourceRef>,
    /// Why the freeze gate holds it (a frozen entity-year or a prior period): the package's
    /// `accountant_decision` covers it. None: not gated.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gated: Option<String>,
    /// Review flags (`reconciled_line`, `split 1/2`, `permanent attachment`).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub flags: Vec<String>,
    /// The audit trail inside Xero: after the operation lands, the run adds the note
    /// `ownpurse package <hash12> op <id>: <why>` to the document's history ([`audit_note`]). `<why>` is
    /// sealed here; None: no note (an [`Op::AddNote`] is itself the note).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub history_note: Option<String>,
}

/// The four operations of v1. Everything else (deletes, payments, transfers, invoices, contacts, accounts,
/// settings, lock dates) is out of scope and can't be expressed.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Op {
    /// A: create a manual journal (`PUT ManualJournals`), POSTED, NoTax, ShowOnCashBasisReports explicit.
    CreateJournal(Journal),
    /// B: recode one bank-transaction line (`POST BankTransactions/{id}`): account code and/or description
    /// only; every other field must round-trip byte-identical (verified).
    RecodeBankLine(Recode),
    /// C: void a POSTED manual journal (`POST ManualJournals/{id}` with `Status: VOIDED`).
    VoidJournal { journal: JournalRef },
    /// D: attach a file to a manual journal (`PUT ManualJournals/{id}/Attachments/{name}`).
    Attach(Attachment),
    /// E: add a note to a document's history (`PUT {Entity}/{id}/History`). Permanent: Xero can't delete
    /// history.
    AddNote(Note),
    /// F: create a SPEND or RECEIVE bank transaction (`PUT BankTransactions`), AUTHORISED, NoTax, the marker
    /// in its Reference: cash and petty-cash entries that have no bank feed.
    CreateBankTransaction(BankTxn),
    /// G: void an unreconciled bank transaction (`POST BankTransactions/{id}` with `Status: DELETED`, how the
    /// API voids one): the recovery of F.
    VoidBankTransaction { bank_transaction: JournalRef },
    /// H: a transfer between two of the org's bank accounts (`PUT BankTransfers`), the marker in its
    /// Reference. The API can't void a transfer: its recovery is a reverse transfer (permanent history).
    CreateBankTransfer(Transfer),
    /// I: a contact (`PUT Contacts`), name and optional email; refused when the exact name exists.
    CreateContact(NewContact),
    /// J: archive a contact (`POST Contacts/{id}` with `ContactStatus: ARCHIVED`): the recovery of I.
    ArchiveContact { contact: JournalRef },
}

/// A bank transfer this tool creates (op H).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Transfer {
    pub from_account_id: String,
    pub from_account: String,
    pub to_account_id: String,
    pub to_account: String,
    pub amount: Decimal,
    /// YYYY-MM-DD.
    pub date: String,
    /// Ends with the marker.
    pub reference: String,
}

/// A contact this tool creates (op I).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NewContact {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
}

/// A bank transaction this tool creates (op F).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BankTxn {
    /// `SPEND` or `RECEIVE`.
    #[serde(rename = "type")]
    pub kind: String,
    /// The bank account (a BANK account of the org): its AccountID, and how a person reads it.
    pub bank_account_id: String,
    pub bank_account: String,
    /// The contact: by id, or the one an earlier op creates (`created_by`).
    pub contact: JournalRef,
    /// YYYY-MM-DD.
    pub date: String,
    /// Ends with the marker ([`marker`]): the duplicate guard searches for it.
    pub reference: String,
    /// Each line's amount is positive (the type gives the direction).
    pub lines: Vec<JournalLine>,
    /// Always explicit: `NoTax`.
    pub line_amount_types: String,
}

impl BankTxn {
    pub fn total(&self) -> Decimal {
        self.lines.iter().map(|l| l.amount).sum()
    }
}

/// The entities a note can go on (the documents the write path touches).
pub const NOTE_ENTITIES: [&str; 4] = [
    "BankTransactions",
    "ManualJournals",
    "BankTransfers",
    "Contacts",
];

/// The longest note this tool sends (Xero's own limit is larger; a note is cut, never refused).
pub const NOTE_MAX: usize = 250;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Note {
    /// One of [`NOTE_ENTITIES`].
    pub entity: String,
    /// The document: by id, or the one an earlier operation of the package creates.
    pub target: JournalRef,
    /// At most [`NOTE_MAX`] characters.
    pub details: String,
}

/// The history note of a landed operation: `ownpurse package <hash12> op <id>: <why>`, in ASCII (Xero keeps
/// other characters as HTML entities), cut to [`NOTE_MAX`].
pub fn audit_note(package_hash: &str, op_id: &str, why: &str) -> String {
    let full = format!(
        "ownpurse package {} op {op_id}: {why}",
        &package_hash[..package_hash.len().min(12)]
    );
    cut_note(&full)
}

/// A note in printable ASCII ([`crate::text::ascii_fold`]), cut to [`NOTE_MAX`] characters (`...` marks the
/// cut).
pub fn cut_note(s: &str) -> String {
    let s = crate::text::ascii_fold(s);
    if s.chars().count() <= NOTE_MAX {
        s
    } else {
        let mut t: String = s.chars().take(NOTE_MAX - 3).collect();
        t.push_str("...");
        t
    }
}

/// "Open in Xero" for a package: each tenant's ShortCode (from the record), keyed by tenant id. Empty: no
/// links (never a guessed one). Not part of the package: links are built when shown, so the sealed bytes
/// never depend on [`crate::deeplink::PATHS`].
pub type Links = std::collections::BTreeMap<String, String>;

/// The page of the document an operation touches: the bank transaction a recode edits, the journal a void
/// or an attachment targets, or `created` (the journal a create made, once known). None without a ShortCode
/// or when the target is another operation's journal not yet created.
pub fn op_url(o: &Operation, links: &Links, created: Option<&str>) -> Option<String> {
    let sc = links.get(&o.tenant_id)?;
    let (entity, id) = match created {
        Some(id) => (o.op.entity(), id.to_string()),
        None => op_target(o)?,
    };
    crate::deeplink::url_for(sc, entity, "", &id)
}

/// The document an operation touches that exists before the run: the bank transaction a recode edits, or
/// the journal a void or an attachment names by id. None for a create, or another op's journal.
pub fn op_target(o: &Operation) -> Option<(&'static str, String)> {
    match &o.op {
        Op::RecodeBankLine(r) => Some(("BankTransactions", r.bank_transaction_id.clone())),
        Op::VoidJournal {
            journal: JournalRef::Id(id),
        } => Some(("ManualJournals", id.clone())),
        Op::Attach(a) => match &a.journal {
            JournalRef::Id(id) => Some((o.op.entity(), id.clone())),
            JournalRef::CreatedBy(_) => None,
        },
        Op::AddNote(n) => match &n.target {
            JournalRef::Id(id) => Some((o.op.entity(), id.clone())),
            JournalRef::CreatedBy(_) => None,
        },
        Op::VoidBankTransaction {
            bank_transaction: JournalRef::Id(id),
        } => Some(("BankTransactions", id.clone())),
        Op::ArchiveContact {
            contact: JournalRef::Id(id),
        } => Some(("Contacts", id.clone())),
        _ => None,
    }
}

/// [`Links`] for a package's tenants, from the record.
pub fn links(record: &crate::Record, p: &Package) -> Links {
    p.ops
        .iter()
        .filter_map(|o| {
            crate::deeplink::short_code(record, &o.tenant_id).map(|c| (o.tenant_id.clone(), c))
        })
        .collect()
}

impl Op {
    /// `create_journal`, `recode_bank_line`, `void_journal`, `attach` (the `operations.kind` column).
    pub fn kind(&self) -> &'static str {
        match self {
            Op::CreateJournal(_) => "create_journal",
            Op::RecodeBankLine(_) => "recode_bank_line",
            Op::VoidJournal { .. } => "void_journal",
            Op::Attach(_) => "attach",
            Op::AddNote(_) => "add_note",
            Op::CreateBankTransaction(_) => "create_bank_transaction",
            Op::VoidBankTransaction { .. } => "void_bank_transaction",
            Op::CreateBankTransfer(_) => "create_bank_transfer",
            Op::CreateContact(_) => "create_contact",
            Op::ArchiveContact { .. } => "archive_contact",
        }
    }
    /// The Xero entity it writes (`operations.target_entity`).
    pub fn entity(&self) -> &'static str {
        match self {
            Op::RecodeBankLine(_)
            | Op::CreateBankTransaction(_)
            | Op::VoidBankTransaction { .. } => "BankTransactions",
            Op::Attach(a) if a.entity == "BankTransactions" => "BankTransactions",
            Op::CreateBankTransfer(_) => "BankTransfers",
            Op::CreateContact(_) | Op::ArchiveContact { .. } => "Contacts",
            Op::AddNote(n) => NOTE_ENTITIES
                .iter()
                .copied()
                .find(|e| *e == n.entity)
                .unwrap_or("ManualJournals"),
            _ => "ManualJournals",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Journal {
    /// YYYY-MM-DD.
    pub date: String,
    /// Ends with the marker ([`marker`]): the duplicate guard searches for it.
    pub narration: String,
    pub lines: Vec<JournalLine>,
    /// Always explicit: `NoTax` in v1.
    pub line_amount_types: String,
    /// Always explicit.
    pub show_on_cash_basis_reports: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct JournalLine {
    pub account_code: String,
    pub description: String,
    /// Debit positive, credit negative (Xero's `LineAmount`).
    pub amount: Decimal,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Recode {
    pub bank_transaction_id: String,
    pub line_item_id: String,
    /// The line as it is now (the precondition) and as it will be; None = unchanged.
    pub from_account_code: String,
    pub from_description: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub to_account_code: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub to_description: Option<String>,
}

/// A journal by Xero id, or the one an earlier operation of the same package creates.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JournalRef {
    Id(String),
    CreatedBy(String),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Attachment {
    /// The document: a journal (by id or the one an earlier op creates) or, with `entity`, a bank
    /// transaction.
    pub journal: JournalRef,
    /// `ManualJournals` (the default, left out of the package) or `BankTransactions`.
    #[serde(
        default = "manual_journals",
        skip_serializing_if = "is_manual_journals"
    )]
    pub entity: String,
    /// The name Xero shows (`supporting-2025-03.csv`); PDF, CSV or TXT, at most 10 MB.
    pub file_name: String,
    pub mime: String,
    /// The bytes' sha256 (the package binds the file, not its path).
    pub sha256: String,
    pub bytes: u64,
    /// Where the file is at plan time (re-hashed before sending).
    pub path: String,
}

fn manual_journals() -> String {
    "ManualJournals".into()
}

fn is_manual_journals(e: &str) -> bool {
    e == "ManualJournals"
}

/// The entities a file can be attached to.
pub const ATTACH_ENTITIES: [&str; 2] = ["ManualJournals", "BankTransactions"];

/// The recovery of an operation (design §4): an inverse [`Op`] and why, or a manual step.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Recovery {
    pub note: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub op: Option<Op>,
}

/// A decision or source row an operation rests on, with the hash of its content when planned.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceRef {
    pub id: String,
    pub content_hash: String,
}

/// An item listed for the reviewer that will not run.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Excluded {
    pub what: String,
    pub why: String,
}

// ── preconditions and expected effects ─────────────────────────────────────────────────────────────────

/// A live check before an operation is sent; any failure ends it `precondition_failed` and stops the batch.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "check", rename_all = "snake_case")]
pub enum Precondition {
    /// The target as planned: [`fingerprint`] of its canonical business fields, plus its version (the record's)
    /// and status.
    Target {
        entity: String,
        id: String,
        fingerprint: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        version: Option<i64>,
        status: String,
        /// The document that was fingerprinted ([`business_view`]): what a mismatch is diffed against.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        before: Option<Value>,
    },
    /// For a create: no journal carries this marker (the real duplicate guard; Idempotency-Key retention is
    /// time-bounded on Xero's side).
    MarkerAbsent { marker: String },
    /// Each account code exists once in the org's chart and is ACTIVE.
    AccountsActive { codes: Vec<String> },
    /// The date is after the org's lock dates.
    PeriodOpen { date: String },
    /// For an attachment: the journal has no attachment with this name and content yet (checked live with
    /// GET `ManualJournals/{id}/Attachments`).
    AttachmentAbsent { file_name: String, sha256: String },
    /// The document exists (a note needs nothing else of it: no fingerprint, it may change meanwhile).
    Exists { entity: String, id: String },
    /// For a bank transaction create (or, with `entity: BankTransfers`, a transfer): none carries this
    /// marker in its Reference (checked live).
    ReferenceAbsent {
        marker: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        entity: Option<String>,
    },
    /// For a contact create: no contact has exactly this name (checked live).
    ContactNameAbsent { name: String },
}

/// Which report section a delta lands in.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Section {
    Bs,
    Pl,
}

/// One account's expected movement (trial balance), debit positive. Every account not listed must not move.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExpectedDelta {
    pub account_id: String,
    pub code: String,
    pub name: String,
    pub section: Section,
    pub amount: Decimal,
}

/// The fields a target fingerprint leaves out: Xero's own bookkeeping, and anything this tool adds (`_xa*`).
pub const FINGERPRINT_DROPS: [&str; 2] = ["UpdatedDateUTC", "HasAttachments"];

/// A document's business fields in one normalised shape, the same whether it came from a list GET (as a
/// sync stores it) or a GET by id (as a run reads it): Xero adds fields on one endpoint only (contact
/// details, account names, `Url`, …), writes numbers as `6.00` or `6`, and dates as `/Date(…)/` or ISO. A
/// bank transaction or a manual journal keeps the fields a write can change or must keep (ids, type, status,
/// date, contact and bank account ids, reference, reconciled, amounts, and per line its id, account,
/// description and amounts); any other document keeps everything but [`FINGERPRINT_DROPS`] and `_xa*`.
/// Numbers go through exact decimals; null, empty strings, empty lists and empty objects are dropped.
pub fn business_view(doc: &Value) -> Value {
    fn num(v: &Value) -> Option<Value> {
        let t = match v {
            Value::Number(n) => n.to_string(),
            Value::String(s) => s.clone(),
            _ => return None,
        };
        let d: Decimal = t.trim().parse().ok()?;
        Some(Value::String(d.normalize().to_string()))
    }
    fn tidy(v: &Value) -> Option<Value> {
        match v {
            Value::Null => None,
            Value::String(s) if s.is_empty() => None,
            Value::Number(_) => num(v),
            Value::Array(a) => {
                let a: Vec<Value> = a.iter().filter_map(tidy).collect();
                (!a.is_empty()).then_some(Value::Array(a))
            }
            Value::Object(m) => {
                let m: serde_json::Map<String, Value> = m
                    .iter()
                    .filter(|(k, _)| {
                        !FINGERPRINT_DROPS.contains(&k.as_str()) && !k.starts_with("_xa")
                    })
                    .filter_map(|(k, v)| tidy(v).map(|v| (k.clone(), v)))
                    .collect();
                (!m.is_empty()).then_some(Value::Object(m))
            }
            other => Some(other.clone()),
        }
    }
    fn pick(doc: &Value, keys: &[&str]) -> serde_json::Map<String, Value> {
        let mut m = serde_json::Map::new();
        for k in keys {
            if let Some(v) = doc.get(*k).and_then(tidy) {
                m.insert((*k).into(), v);
            }
        }
        m
    }
    let lines = |key: &str, fields: &[&str]| -> Option<Value> {
        let a: Vec<Value> = doc
            .get(key)?
            .as_array()?
            .iter()
            .map(|l| {
                let mut m = pick(l, fields);
                for k in ["LineAmount", "TaxAmount", "Quantity", "UnitAmount"] {
                    // a zero tax amount is absent on some endpoints: zero and absent are the same
                    if let Some(n) = l
                        .get(k)
                        .and_then(num)
                        .filter(|n| k != "TaxAmount" || n != "0")
                    {
                        m.insert(k.into(), n);
                    }
                }
                Value::Object(m)
            })
            .collect();
        (!a.is_empty()).then_some(Value::Array(a))
    };
    let mut m = if doc.get("BankTransactionID").is_some() {
        let mut m = pick(
            doc,
            &[
                "BankTransactionID",
                "Type",
                "Status",
                "Reference",
                "IsReconciled",
                "LineAmountTypes",
                "CurrencyCode",
            ],
        );
        for k in ["SubTotal", "TotalTax", "Total"] {
            if let Some(n) = doc
                .get(k)
                .and_then(num)
                .filter(|n| k != "TotalTax" || n != "0")
            {
                m.insert(k.into(), n);
            }
        }
        if let Some(c) = doc.pointer("/Contact/ContactID").and_then(tidy) {
            m.insert("ContactID".into(), c);
        }
        if let Some(b) = doc.pointer("/BankAccount/AccountID").and_then(tidy) {
            m.insert("BankAccountID".into(), b);
        }
        if let Some(l) = lines("LineItems", &["LineItemID", "AccountCode", "Description"]) {
            m.insert("LineItems".into(), l);
        }
        m
    } else if doc.get("ManualJournalID").is_some() {
        let mut m = pick(
            doc,
            &[
                "ManualJournalID",
                "Status",
                "Narration",
                "LineAmountTypes",
                "ShowOnCashBasisReports",
            ],
        );
        if let Some(l) = lines("JournalLines", &["AccountCode", "Description"]) {
            m.insert("JournalLines".into(), l);
        }
        m
    } else {
        return tidy(doc).unwrap_or(Value::Null);
    };
    let date = doc
        .get("DateString")
        .and_then(Value::as_str)
        .map(|s| s.chars().take(10).collect::<String>())
        .or_else(|| {
            doc.get("Date")
                .and_then(Value::as_str)
                .and_then(crate::dates::xero_date)
        });
    if let Some(d) = date {
        m.insert("Date".into(), Value::String(d));
    }
    Value::Object(m)
}

/// The canonical fingerprint of a document: sha256 of its [`business_view`] as canonical JSON. Version and
/// status are compared alongside ([`Precondition::Target`]).
pub fn fingerprint(doc: &Value) -> String {
    sha256_hex(crate::canon::canonical(&business_view(doc)).as_bytes())
}

/// An RFC 6902 JSON Patch from `before` to `after` (what changed, for a person and for `--json`).
pub fn patch(before: &Value, after: &Value) -> Value {
    serde_json::to_value(json_patch::diff(before, after)).unwrap_or(Value::Null)
}

// ── shared helpers (both sides must agree byte for byte) ───────────────────────────────────────────────

/// Two decimals for the write path (requests, reviews, plans): a zero is always `0.00`, never `-0.00` (the
/// record's own `money_str` keeps a signed zero for byte compatibility; nothing written or reviewed may).
pub fn money_text(v: Decimal) -> String {
    let q = v.round_dp_with_strategy(2, rust_decimal::RoundingStrategy::MidpointNearestEven);
    if q.is_zero() {
        return "0.00".into();
    }
    crate::money::money_str(q)
}

pub fn sha256_hex(bytes: &[u8]) -> String {
    let mut h = Sha256::new();
    h.update(bytes);
    h.finalize().iter().map(|b| format!("{b:02x}")).collect()
}

/// The narration marker of an operation: `XA:<package_id>:<op_id>`.
pub fn marker(package_id: &str, op_id: &str) -> String {
    format!("XA:{package_id}:{op_id}")
}

/// The marker of a reversing journal (design §4): `XA-REV:<op_id>`.
pub fn reversal_marker(op_id: &str) -> String {
    format!("XA-REV:{op_id}")
}

/// The Idempotency-Key of an operation: sha256(package hash ∥ op id), stable across re-sends.
pub fn idempotency_key(package_hash: &str, op_id: &str) -> String {
    sha256_hex(format!("{package_hash}{op_id}").as_bytes())
}

// ── the transport ───────────────────────────────────────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Method {
    Put,
    Post,
}

/// The whole allow-list: (method, path pattern; `{id}` and `{name}` are one path segment). Checked before any
/// byte leaves the machine; `DELETE` doesn't exist in the transport.
pub const ALLOWED: [(Method, &str); 13] = [
    (Method::Put, "ManualJournals"),
    (Method::Post, "ManualJournals/{id}"),
    (Method::Put, "BankTransactions"),
    (Method::Put, "BankTransfers"),
    (Method::Put, "Contacts"),
    (Method::Post, "Contacts/{id}"),
    (Method::Post, "BankTransactions/{id}"),
    (Method::Put, "ManualJournals/{id}/Attachments/{name}"),
    (Method::Put, "BankTransactions/{id}/Attachments/{name}"),
    // history notes (the audit trail, and Op E)
    (Method::Put, "ManualJournals/{id}/History"),
    (Method::Put, "BankTransactions/{id}/History"),
    (Method::Put, "BankTransfers/{id}/History"),
    (Method::Put, "Contacts/{id}/History"),
];

/// Whether `(method, path)` matches the allow-list (`path` relative to `api.xro/2.0/`, no query).
pub fn allowed(method: Method, path: &str) -> bool {
    let segs: Vec<&str> = path.split('/').collect();
    ALLOWED.iter().any(|(m, pat)| {
        let ps: Vec<&str> = pat.split('/').collect();
        *m == method
            && ps.len() == segs.len()
            && ps.iter().zip(&segs).all(|(p, s)| {
                if p.starts_with('{') {
                    !s.is_empty() && !s.contains(['?', '#', '\\']) && *s != "." && *s != ".."
                } else {
                    p == s
                }
            })
    })
}

/// A request body: JSON, or a file's bytes (attachments).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Body {
    Json {
        json: Value,
    },
    /// The bytes are read from [`Attachment::path`] and re-hashed at send time; the package stores the hash.
    File {
        sha256: String,
        mime: String,
    },
}

/// One write request, exactly as it will be sent (the transport adds auth and the tenant header).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct WriteRequest {
    pub method: Method,
    /// Relative to `api.xro/2.0/`; for an operation on a journal another op creates, `{id}` stays in the path
    /// and is filled from that op's confirmed response at run time.
    pub path: String,
    pub body: Body,
}

/// What the transport sends: the request plus the run-time facts.
#[derive(Clone, Debug, PartialEq)]
pub struct Send<'a> {
    pub tenant_id: &'a str,
    pub request: &'a WriteRequest,
    /// The resolved path (placeholders filled).
    pub path: String,
    pub idempotency_key: &'a str,
    /// The file's bytes for [`Body::File`].
    pub file: Option<&'a [u8]>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Response {
    pub status: u16,
    pub body: Value,
    /// `Retry-After` seconds on a 429.
    pub retry_after: Option<u64>,
}

/// Why a request didn't produce a response.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TransportError {
    /// Not in [`ALLOWED`] (nothing left the machine).
    NotAllowed(String),
    /// Refused before sending (tenant guard, base URL, missing scope): nothing left the machine.
    Refused { code: String, message: String },
    /// Failed before any byte was sent: safe to report as not attempted.
    NotSent(String),
    /// Failed after the request may have been sent (timeout, connection reset): the attempt is `uncertain`.
    AfterSend(String),
}

/// The write side of the Xero API, plus the reads the run needs (preconditions, read-back, the tenant name).
/// The real one is in the api crate (CLI only); tests use a fake server or [`FakeTransport`].
pub trait WriteTransport {
    /// Send one allow-listed write.
    fn send(&mut self, s: &Send<'_>) -> Result<Response, TransportError>;
    /// GET a path for reads (`ManualJournals/{id}`, `BankTransactions/{id}`, a `where` search for a marker,
    /// `Accounts`, `Organisation`, reports).
    fn get(&mut self, tenant_id: &str, path_and_query: &str) -> Result<Response, TransportError>;
    /// The connected tenant's name (`require_tenant_name`), checked before the first write of a run.
    fn tenant_name(&mut self, tenant_id: &str) -> Result<String, TransportError>;
    /// The token's granted scopes (space-separated in Xero's form).
    fn scopes(&self) -> Vec<String>;
}

/// An in-memory scripted transport for unit tests of either side (the api crate's end-to-end tests use a
/// loopback fake Xero server instead). Answers `send` from `script` in order and records every send; reads
/// come from `gets` by exact path (404 when absent).
#[derive(Debug, Default)]
pub struct FakeTransport {
    pub script: std::collections::VecDeque<Result<Response, TransportError>>,
    pub gets: std::collections::BTreeMap<String, Value>,
    pub tenant_names: std::collections::BTreeMap<String, String>,
    pub granted: Vec<String>,
    /// (tenant, method, resolved path, idempotency key, body) of every send, in order.
    pub sent: Vec<(String, Method, String, String, Option<Value>)>,
}

impl WriteTransport for FakeTransport {
    fn send(&mut self, s: &Send<'_>) -> Result<Response, TransportError> {
        if !allowed(s.request.method, &s.path) {
            return Err(TransportError::NotAllowed(format!(
                "{:?} {}",
                s.request.method, s.path
            )));
        }
        let body = match &s.request.body {
            Body::Json { json } => Some(json.clone()),
            Body::File { .. } => None,
        };
        self.sent.push((
            s.tenant_id.to_string(),
            s.request.method,
            s.path.clone(),
            s.idempotency_key.to_string(),
            body,
        ));
        self.script.pop_front().unwrap_or_else(|| {
            Err(TransportError::NotSent(
                "the fake has no scripted answer".into(),
            ))
        })
    }
    fn get(&mut self, _tenant_id: &str, path_and_query: &str) -> Result<Response, TransportError> {
        Ok(match self.gets.get(path_and_query) {
            Some(v) => Response {
                status: 200,
                body: v.clone(),
                retry_after: None,
            },
            None => Response {
                status: 404,
                body: Value::Null,
                retry_after: None,
            },
        })
    }
    fn tenant_name(&mut self, tenant_id: &str) -> Result<String, TransportError> {
        self.tenant_names
            .get(tenant_id)
            .cloned()
            .ok_or_else(|| TransportError::Refused {
                code: "tenant".into(),
                message: format!("not connected: {tenant_id}"),
            })
    }
    fn scopes(&self) -> Vec<String> {
        self.granted.clone()
    }
}

// ── attempts (op_events) ───────────────────────────────────────────────────────────────────────────────

/// The states of `op_events.state` (the record's CHECK constraint, unchanged).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AttemptState {
    Planned,
    Approved,
    Sending,
    Confirmed,
    Rejected,
    Uncertain,
    Reconciled,
}

impl AttemptState {
    pub fn as_str(self) -> &'static str {
        match self {
            AttemptState::Planned => "planned",
            AttemptState::Approved => "approved",
            AttemptState::Sending => "sending",
            AttemptState::Confirmed => "confirmed",
            AttemptState::Rejected => "rejected",
            AttemptState::Uncertain => "uncertain",
            AttemptState::Reconciled => "reconciled",
        }
    }
}

/// One row of the attempt log (`op_events`, chained). `sending` is appended durably BEFORE the request.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Attempt {
    /// `operations.op_id`: `<package hash 12>:<op id>`.
    pub op_id: String,
    pub seq: i64,
    pub at: String,
    pub state: AttemptState,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub idempotency_key: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub http_status: Option<u16>,
    /// The response or the reason (JSON), as evidence.
    #[serde(default)]
    pub detail: Value,
}

// ── approvals ───────────────────────────────────────────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Via {
    /// `write approve-record`, run by the owner at a TTY.
    Tty,
    /// `write approve-import`: a USER-role message from the owner's own session transcript quoting the hash.
    Chat,
    /// `write approve-demo`: the owner's standing approval for the Demo Company only (his words in
    /// `standing-approval.txt` beside the demo config); never valid for any other profile.
    DemoStanding,
}

impl Via {
    /// The stored word (`approvals.via`).
    pub fn as_str(self) -> &'static str {
        match self {
            Via::Tty => "tty",
            Via::Chat => "chat",
            Via::DemoStanding => "demo_standing",
        }
    }
    pub fn parse(s: &str) -> Option<Via> {
        match s {
            "tty" => Some(Via::Tty),
            "chat" => Some(Via::Chat),
            "demo_standing" => Some(Via::DemoStanding),
            _ => None,
        }
    }
    /// How a review or status names it.
    pub fn label(self) -> &'static str {
        match self {
            Via::Tty => "owner at his terminal",
            Via::Chat => "owner's chat message",
            Via::DemoStanding => "demo standing approval",
        }
    }
}

/// The file beside the demo config that holds the owner's standing approval for the Demo Company.
pub const STANDING_APPROVAL_FILE: &str = "standing-approval.txt";

/// Which operations an approval covers.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ApprovalScope {
    /// `"all"`.
    All(AllOps),
    /// A list of op ids.
    Ops(Vec<String>),
}

/// The literal `"all"`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AllOps {
    All,
}

impl ApprovalScope {
    pub fn covers(&self, op_id: &str) -> bool {
        match self {
            ApprovalScope::All(_) => true,
            ApprovalScope::Ops(v) => v.iter().any(|o| o == op_id),
        }
    }
}

/// The owner's approval, bound to a package hash. Append-only, in the record's chain.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApprovalRecord {
    /// The sealed package hash it approves (full sha256).
    pub hash: String,
    /// The owner's words, verbatim.
    pub verbatim: String,
    pub via: Via,
    /// Where it was given: `tty`, or `transcript:<path>#session=<id>`.
    #[serde(rename = "where")]
    pub where_: String,
    /// When it was given (the message's timestamp for chat).
    pub at: String,
    pub scope: ApprovalScope,
    /// When it was recorded.
    pub recorded_at: String,
}

/// Approval storage: the record's append-only `approvals` table (the run side owns the schema; the package
/// side records and reads through this).
pub trait ApprovalStore {
    /// Append one approval; returns its row id. Never replaces or removes.
    fn record(&mut self, a: &ApprovalRecord) -> Result<i64, WriteError>;
    /// Every approval recorded for this hash, oldest first.
    fn approvals(&self, hash: &str) -> Result<Vec<ApprovalRecord>, WriteError>;
}

/// An in-memory [`ApprovalStore`] for tests.
#[derive(Debug, Default)]
pub struct MemApprovals(pub Vec<ApprovalRecord>);

impl ApprovalStore for MemApprovals {
    fn record(&mut self, a: &ApprovalRecord) -> Result<i64, WriteError> {
        self.0.push(a.clone());
        Ok(self.0.len() as i64)
    }
    fn approvals(&self, hash: &str) -> Result<Vec<ApprovalRecord>, WriteError> {
        Ok(self.0.iter().filter(|a| a.hash == hash).cloned().collect())
    }
}

// ── errors and exit codes (design §6) ───────────────────────────────────────────────────────────────────

pub mod exit {
    pub const OK: i32 = 0;
    pub const RUNTIME: i32 = 1;
    pub const USAGE: i32 = 2;
    pub const NOT_APPROVED: i32 = 3;
    pub const PRECONDITION: i32 = 4;
    pub const DISCREPANCY: i32 = 5;
    pub const UNCERTAIN: i32 = 6;
    pub const WRITES_DISABLED: i32 = 7;
}

/// A refusal or failure of any write command: a stable code, a message, what to do, and the exit code.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WriteError {
    pub code: String,
    pub message: String,
    pub help: Vec<String>,
    pub exit: i32,
}

impl WriteError {
    pub fn new(code: &str, exit: i32, message: impl Into<String>) -> WriteError {
        WriteError {
            code: code.into(),
            message: message.into(),
            help: vec![],
            exit,
        }
    }
    pub fn help(mut self, h: impl Into<String>) -> WriteError {
        self.help.push(h.into());
        self
    }
}

impl std::fmt::Display for WriteError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}

impl std::error::Error for WriteError {}

pub mod approval;
pub mod delta;
pub mod export;
pub mod gate;
pub mod ops;
pub mod pdf;
pub mod plan;
pub mod seal;
pub mod view;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_allow_list_is_exactly_four_shapes() {
        assert!(allowed(Method::Put, "ManualJournals"));
        assert!(allowed(Method::Post, "ManualJournals/mj-1"));
        assert!(allowed(Method::Post, "BankTransactions/bt-104"));
        assert!(allowed(
            Method::Put,
            "ManualJournals/mj-1/Attachments/support.csv"
        ));
        assert!(!allowed(Method::Post, "ManualJournals"));
        assert!(!allowed(Method::Put, "BankTransactions/bt-104"));
        assert!(!allowed(Method::Post, "Invoices/inv-1"));
        assert!(!allowed(
            Method::Post,
            "BankTransactions/bt-104/Attachments/x.csv"
        ));
        assert!(!allowed(Method::Post, "ManualJournals/.."));
        assert!(!allowed(Method::Post, "ManualJournals/"));
        assert!(!allowed(
            Method::Post,
            "ManualJournals/x?summarizeErrors=false"
        ));
    }

    #[test]
    fn money_text_never_shows_a_signed_zero() {
        for (v, want) in [
            ("-0.00", "0.00"),
            ("-0.001", "0.00"),
            ("0", "0.00"),
            ("-1.005", "-1.00"),
            ("263.94", "263.94"),
        ] {
            assert_eq!(money_text(v.parse().unwrap()), want, "{v}");
        }
    }

    #[test]
    fn markers_and_keys_are_stable() {
        assert_eq!(marker("pkg-1", "op-01"), "XA:pkg-1:op-01");
        assert_eq!(idempotency_key("abc", "op-01"), sha256_hex(b"abcop-01"));
        assert_ne!(
            idempotency_key("abc", "op-01"),
            idempotency_key("abc", "op-02")
        );
    }

    #[test]
    fn the_fingerprint_ignores_bookkeeping_fields() {
        let a = serde_json::json!({"Status": "POSTED", "UpdatedDateUTC": "/Date(1)/", "Lines": [{"_xa_note": 1, "LineAmount": 1.5}]});
        let b = serde_json::json!({"Lines": [{"LineAmount": 1.5}], "UpdatedDateUTC": "/Date(2)/", "Status": "POSTED"});
        assert_eq!(fingerprint(&a), fingerprint(&b));
        let c = serde_json::json!({"Lines": [{"LineAmount": 1.51}], "Status": "POSTED"});
        assert_ne!(fingerprint(&a), fingerprint(&c));
    }

    #[test]
    fn a_package_round_trips_through_json() {
        let p = Package {
            v: PACKAGE_V,
            id: "demo-01".into(),
            title: "Demo".into(),
            profile: "demo".into(),
            created_at: "2026-10-07T12:00:00Z".into(),
            accountant_decision: None,
            ops: vec![Operation {
                id: "op-01".into(),
                org: "demo".into(),
                tenant_id: "t".into(),
                op: Op::CreateJournal(Journal {
                    date: "2025-03-31".into(),
                    narration: "Owner-paid costs · XA:demo-01:op-01".into(),
                    lines: vec![
                        JournalLine {
                            account_code: "6310".into(),
                            description: "x".into(),
                            amount: "147.38".parse().unwrap(),
                        },
                        JournalLine {
                            account_code: "2700".into(),
                            description: "x".into(),
                            amount: "-147.38".parse().unwrap(),
                        },
                    ],
                    line_amount_types: "NoTax".into(),
                    show_on_cash_basis_reports: true,
                }),
                request: WriteRequest {
                    method: Method::Put,
                    path: "ManualJournals".into(),
                    body: Body::Json { json: Value::Null },
                },
                preconditions: vec![Precondition::MarkerAbsent {
                    marker: marker("demo-01", "op-01"),
                }],
                expected_after: None,
                expected: vec![ExpectedDelta {
                    account_id: "a1".into(),
                    code: "6310".into(),
                    name: "Supplies".into(),
                    section: Section::Pl,
                    amount: "147.38".parse().unwrap(),
                }],
                recovery: Some(Box::new(Recovery {
                    note: "reverse".into(),
                    op: None,
                })),
                depends_on: vec![],
                decision_refs: vec![],
                source_refs: vec![],
                gated: None,
                flags: vec![],
                history_note: None,
            }],
            excluded: vec![],
            checklist: vec![],
            supersedes: vec![],
            source: None,
            seal: None,
        };
        let j = serde_json::to_value(&p).unwrap();
        assert_eq!(j["ops"][0]["op"]["kind"], "create_journal");
        assert_eq!(j["ops"][0]["expected"][0]["amount"], "147.38");
        let back: Package = serde_json::from_value(j).unwrap();
        assert_eq!(back, p);
        let s: ApprovalScope = serde_json::from_value(serde_json::json!("all")).unwrap();
        assert!(s.covers("op-09"));
        let s: ApprovalScope = serde_json::from_value(serde_json::json!(["op-01"])).unwrap();
        assert!(s.covers("op-01") && !s.covers("op-02"));
    }

    #[test]
    fn the_fake_transport_refuses_outside_the_allow_list() {
        let mut t = FakeTransport::default();
        let req = WriteRequest {
            method: Method::Post,
            path: "Invoices/x".into(),
            body: Body::Json { json: Value::Null },
        };
        let s = Send {
            tenant_id: "t",
            request: &req,
            path: req.path.clone(),
            idempotency_key: "k",
            file: None,
        };
        assert!(matches!(t.send(&s), Err(TransportError::NotAllowed(_))));
        assert!(t.sent.is_empty());
    }
}
