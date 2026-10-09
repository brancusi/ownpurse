//! The operation builders (design §1): each turns a requested change into an [`Operation`] with the exact
//! request, the preconditions the run re-checks live, the expected report movements (the same ledger rules
//! as every read: [`delta::moves`]) and a pre-compiled recovery. Nothing here sends anything.
//!
//! - OP A [`create_journal`]: a POSTED manual journal, `LineAmountTypes: NoTax` and `ShowOnCashBasisReports`
//!   always explicit, the narration ending with the marker `XA:<package>:<op>`; split into balanced parts
//!   when a line limit applies (each part balances through the named split account, which nets to zero).
//! - OP B [`recode_bank_line`]: one bank-transaction line's account code and/or description, by its
//!   `LineItemID` (never by position); the request echoes the whole document so every other field
//!   round-trips; a reconciled line is flagged (Xero may refuse the edit).
//! - OP C [`void_journal`]: a POSTED journal → VOIDED; recovery re-creates it from the before-snapshot.
//! - OP D [`attach`]: a PDF/CSV/TXT (≤ 10 MB) on a journal, existing or created by an earlier op; bound by
//!   the bytes' sha256.

use super::delta::{self, DocKind};
use super::{
    Attachment, BankTxn, Body, ExpectedDelta, Journal, JournalLine, JournalRef, Method, NewContact,
    Note, Op, Operation, Precondition, Recode, Recovery, Transfer, WriteError, WriteRequest, exit,
    fingerprint, marker, reversal_marker, sha256_hex,
};
use crate::dates::{doc_date, parse_iso};
use crate::ledger::{Account, Chart};
use crate::money::money;
use crate::record::Row;
use rust_decimal::Decimal;
use serde_json::{Map, Value, json};
use std::str::FromStr;

/// Xero's attachment size limit.
pub const MAX_ATTACHMENT: u64 = 10 * 1024 * 1024;

pub(crate) fn usage(m: impl Into<String>) -> WriteError {
    WriteError::new("usage", exit::USAGE, m)
}

/// What every builder needs about the org.
#[derive(Default)]
pub struct OrgCtx {
    pub package_id: String,
    pub org: String,
    pub tenant_id: String,
    /// The org's Accounts payloads (from the record).
    pub accounts: Vec<Value>,
    /// The later of the org's period and end-of-year lock dates (None: unlocked).
    pub lock: Option<String>,
    /// The org's ShortCode (from the record), for "Open in Xero" links in refusals; None: no link.
    pub short_code: Option<String>,
}

impl OrgCtx {
    fn period_open(&self, date: &str) -> Result<Precondition, WriteError> {
        if let Some(l) = &self.lock
            && date <= l.as_str()
        {
            return Err(WriteError::new(
                "precondition",
                exit::PRECONDITION,
                format!(
                    "{}: {date} is inside the lock date {l}; Xero refuses it",
                    self.org
                ),
            ));
        }
        Ok(Precondition::PeriodOpen { date: date.into() })
    }

    /// The account for a code: exactly one, ACTIVE, and not a bank account (bank lines come only from bank
    /// transactions).
    pub fn account(&self, code: &str) -> Result<Account, WriteError> {
        let n = self
            .accounts
            .iter()
            .filter(|a| a.get("Code").and_then(Value::as_str) == Some(code))
            .count();
        if n == 0 {
            return Err(usage(format!("{}: no account with code {code}", self.org)));
        }
        if n > 1 {
            return Err(usage(format!(
                "{}: the code {code} is not unique",
                self.org
            )));
        }
        let a = Chart::new(&self.accounts)
            .by_code(code)
            .cloned()
            .ok_or_else(|| usage(format!("{}: no account with code {code}", self.org)))?;
        if a.status != "ACTIVE" {
            return Err(usage(format!(
                "{}: account {code} is {}, not ACTIVE",
                self.org, a.status
            )));
        }
        if a.kind == "BANK" {
            return Err(usage(format!(
                "{}: {code} is a bank account; a journal or a recode can't post to it",
                self.org
            )));
        }
        Ok(a)
    }

    fn moves(
        &self,
        kind: DocKind,
        before: Option<&Value>,
        after: Option<&Value>,
    ) -> Result<Vec<ExpectedDelta>, WriteError> {
        delta::moves(&self.accounts, kind, before, after)
            .map_err(|e| usage(format!("{}: {e}", self.org)))
    }
}

fn num(d: Decimal) -> Value {
    Value::Number(serde_json::Number::from_str(&super::money_text(d)).unwrap_or_else(|_| 0.into()))
}

fn date_ok(d: &str) -> Result<(), WriteError> {
    if d.len() == 10 && parse_iso(d).is_some() {
        Ok(())
    } else {
        Err(usage(format!("{d:?} is not a YYYY-MM-DD date")))
    }
}

/// Local marks (`_xa*`) never leave the machine.
fn strip_marks(doc: &Value) -> Value {
    match doc {
        Value::Object(m) => Value::Object(
            m.iter()
                .filter(|(k, _)| !k.starts_with("_xa"))
                .map(|(k, v)| (k.clone(), strip_marks(v)))
                .collect(),
        ),
        Value::Array(a) => Value::Array(a.iter().map(strip_marks).collect()),
        o => o.clone(),
    }
}

/// What this tool writes into Xero's free text (narrations, references, descriptions) is printable ASCII:
/// Xero keeps other characters as HTML entities, so what comes back would differ from what was sealed.
fn fold(t: &str) -> String {
    crate::text::ascii_fold(t)
}

fn s<'a>(v: &'a Value, k: &str) -> &'a str {
    v.get(k).and_then(Value::as_str).unwrap_or("")
}

// ---------------------------------------------------------------- requests (shared with recovery packages)

/// The Xero ManualJournal body of a journal.
pub fn journal_doc(j: &Journal) -> Value {
    json!({
        "Narration": j.narration,
        "Date": j.date,
        "Status": "POSTED",
        "LineAmountTypes": j.line_amount_types,
        "ShowOnCashBasisReports": j.show_on_cash_basis_reports,
        "JournalLines": j.lines.iter().map(|l| {
            let mut m = Map::new();
            m.insert("LineAmount".into(), num(l.amount));
            m.insert("AccountCode".into(), Value::String(l.account_code.clone()));
            if !l.description.is_empty() {
                m.insert("Description".into(), Value::String(fold(&l.description)));
            }
            Value::Object(m)
        }).collect::<Vec<_>>(),
    })
}

/// OP A's request: `PUT ManualJournals`.
pub fn journal_request(j: &Journal) -> WriteRequest {
    WriteRequest {
        method: Method::Put,
        path: "ManualJournals".into(),
        body: Body::Json {
            json: json!({ "ManualJournals": [journal_doc(j)] }),
        },
    }
}

/// OP C's request: `POST ManualJournals/{id}` with `Status: VOIDED`.
pub fn void_request(id: &str) -> WriteRequest {
    WriteRequest {
        method: Method::Post,
        path: format!("ManualJournals/{id}"),
        body: Body::Json {
            json: json!({ "ManualJournals": [{ "ManualJournalID": id, "Status": "VOIDED" }] }),
        },
    }
}

/// OP D's request: `PUT ManualJournals/{id}/Attachments/{name}` (`{id}` stays literal for a journal an
/// earlier op creates; the run fills it).
pub fn attach_request(a: &Attachment) -> WriteRequest {
    let id = match &a.journal {
        JournalRef::Id(id) => id.clone(),
        JournalRef::CreatedBy(_) => "{id}".into(),
    };
    WriteRequest {
        method: Method::Put,
        path: format!("{}/{id}/Attachments/{}", a.entity, a.file_name),
        body: Body::File {
            sha256: a.sha256.clone(),
            mime: a.mime.clone(),
        },
    }
}

/// The document as the recode leaves it, and the body sent (the whole document echoed; the recoded line
/// names its account by code only, so Xero resolves the id). Errors when the line isn't there.
pub fn recoded(
    doc: &Value,
    r: &Recode,
    new_account_id: Option<&str>,
) -> Result<(Value, Value), WriteError> {
    let doc = strip_marks(doc);
    let lines = doc
        .get("LineItems")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let ix = lines
        .iter()
        .position(|x| s(x, "LineItemID") == r.line_item_id)
        .ok_or_else(|| {
            usage(format!(
                "bank transaction {} has no line {}",
                r.bank_transaction_id, r.line_item_id
            ))
        })?;
    let mut after_line = lines[ix].clone();
    let mut sent_line = lines[ix].clone();
    if let Some(code) = &r.to_account_code {
        for l in [&mut after_line, &mut sent_line] {
            if let Some(o) = l.as_object_mut() {
                o.insert("AccountCode".into(), Value::String(code.clone()));
            }
        }
        if let Some(o) = sent_line.as_object_mut() {
            o.remove("AccountID");
        }
        if let (Some(o), Some(id)) = (after_line.as_object_mut(), new_account_id)
            && o.contains_key("AccountID")
        {
            o.insert("AccountID".into(), Value::String(id.into()));
        }
    }
    if let Some(d) = &r.to_description {
        for l in [&mut after_line, &mut sent_line] {
            if let Some(o) = l.as_object_mut() {
                o.insert("Description".into(), Value::String(d.clone()));
            }
        }
    }
    let with = |line: Value| {
        let mut d = doc.clone();
        if let Some(a) = d.get_mut("LineItems").and_then(Value::as_array_mut) {
            a[ix] = line;
        }
        d
    };
    Ok((with(after_line), with(sent_line)))
}

/// OP B's request: `POST BankTransactions/{id}` with the echoed document.
pub fn recode_request(id: &str, sent: Value) -> WriteRequest {
    WriteRequest {
        method: Method::Post,
        path: format!("BankTransactions/{id}"),
        body: Body::Json {
            json: json!({ "BankTransactions": [sent] }),
        },
    }
}

fn op(ctx: &OrgCtx, id: &str, o: Op, request: WriteRequest) -> Operation {
    Operation {
        id: id.into(),
        org: ctx.org.clone(),
        tenant_id: ctx.tenant_id.clone(),
        op: o,
        request,
        preconditions: vec![],
        expected_after: None,
        expected: vec![],
        recovery: None,
        depends_on: vec![],
        decision_refs: vec![],
        source_refs: vec![],
        gated: None,
        flags: vec![],
        history_note: None,
    }
}

/// The dated accounting effect of an operation (the journal's date, or the target's), for the freeze gate.
/// An attachment has none.
pub fn effect_date(o: &Operation) -> Option<&str> {
    match &o.op {
        Op::CreateJournal(j) => Some(&j.date),
        Op::CreateBankTransaction(b) => Some(&b.date),
        Op::CreateBankTransfer(t) => Some(&t.date),
        Op::Attach(_) | Op::AddNote(_) => None,
        _ => o.preconditions.iter().find_map(|p| match p {
            Precondition::PeriodOpen { date } => Some(date.as_str()),
            _ => None,
        }),
    }
}

// ---------------------------------------------------------------- OP A: create a manual journal

#[derive(Debug, Clone, PartialEq)]
pub struct JournalSpec {
    pub id: String,
    pub date: String,
    /// Without the marker (added here).
    pub narration: String,
    /// `ShowOnCashBasisReports` (required: never left to the API default).
    pub cash_basis: bool,
    pub lines: Vec<JournalLine>,
    /// Xero's per-journal line limit, once Milestone 1 has measured it (None: no split).
    pub max_lines: Option<usize>,
    /// The account each split part balances through (required when a split happens).
    pub split_account: Option<String>,
}

/// The ledger's view of a journal body (it reads `DateString` and needs an id).
fn as_posted(doc: &Value) -> Value {
    let mut d = doc.clone();
    if let Some(o) = d.as_object_mut() {
        let date = o.get("Date").cloned().unwrap_or(Value::Null);
        o.insert("DateString".into(), date);
        o.insert("ManualJournalID".into(), Value::String("(new)".into()));
    }
    d
}

/// Split `lines` into parts of at most `max` lines that each balance through `split` (the split lines net
/// to zero across the parts).
fn split_lines(
    lines: &[JournalLine],
    max: usize,
    split: &str,
    narration: &str,
) -> Vec<Vec<JournalLine>> {
    let per = max.saturating_sub(1).max(1);
    let mut parts: Vec<Vec<JournalLine>> = lines.chunks(per).map(<[JournalLine]>::to_vec).collect();
    let n = parts.len();
    for (i, p) in parts.iter_mut().enumerate() {
        let net: Decimal = p.iter().map(|l| l.amount).sum();
        if !net.is_zero() {
            p.push(JournalLine {
                account_code: split.into(),
                amount: -net,
                description: format!("split {}/{n} of: {narration}", i + 1),
            });
        }
    }
    parts
}

fn journal_lines(doc: &Value) -> Vec<JournalLine> {
    doc.get("JournalLines")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .map(|l| JournalLine {
            account_code: s(l, "AccountCode").into(),
            amount: money(l.get("LineAmount")).unwrap_or_default(),
            description: s(l, "Description").into(),
        })
        .collect()
}

pub fn create_journal(ctx: &OrgCtx, spec: &JournalSpec) -> Result<Vec<Operation>, WriteError> {
    date_ok(&spec.date)?;
    if spec.narration.trim().is_empty() {
        return Err(usage(format!("{}: a journal needs a narration", spec.id)));
    }
    if spec.lines.len() < 2 {
        return Err(usage(format!(
            "{}: a journal needs at least two lines",
            spec.id
        )));
    }
    for l in &spec.lines {
        if l.amount.is_zero() || l.amount.round_dp(2) != l.amount {
            return Err(usage(format!(
                "{}: line {} {}: amounts are non-zero with at most 2 decimals",
                spec.id, l.account_code, l.amount
            )));
        }
        ctx.account(&l.account_code)?;
    }
    let net: Decimal = spec.lines.iter().map(|l| l.amount).sum();
    if !net.is_zero() {
        return Err(usage(format!(
            "{}: the lines don't balance (debits − credits = {})",
            spec.id,
            super::money_text(net)
        )));
    }
    let open = ctx.period_open(&spec.date)?;
    let parts = match spec.max_lines {
        Some(max) if spec.lines.len() > max => {
            let split = spec.split_account.as_deref().ok_or_else(|| {
                usage(format!(
                    "{}: {} lines exceed the {max}-line limit; name a split_account to split it",
                    spec.id,
                    spec.lines.len()
                ))
            })?;
            ctx.account(split)?;
            split_lines(&spec.lines, max, split, &spec.narration)
        }
        _ => vec![spec.lines.clone()],
    };
    let n = parts.len();
    let mut out = vec![];
    for (i, lines) in parts.into_iter().enumerate() {
        let id = if n == 1 {
            spec.id.clone()
        } else {
            format!("{}.{}", spec.id, i + 1)
        };
        let m = marker(&ctx.package_id, &id);
        let narration = if n == 1 {
            format!("{} - {m}", fold(&spec.narration))
        } else {
            format!("{} (part {}/{n}) - {m}", fold(&spec.narration), i + 1)
        };
        let mut codes: Vec<String> = lines.iter().map(|l| l.account_code.clone()).collect();
        codes.sort();
        codes.dedup();
        let j = Journal {
            date: spec.date.clone(),
            narration,
            lines,
            line_amount_types: "NoTax".into(),
            show_on_cash_basis_reports: spec.cash_basis,
        };
        let rev = Journal {
            narration: format!(
                "Reversal of {} - {}",
                fold(&spec.narration),
                reversal_marker(&id)
            ),
            lines: j
                .lines
                .iter()
                .map(|l| JournalLine {
                    amount: -l.amount,
                    ..l.clone()
                })
                .collect(),
            ..j.clone()
        };
        let mut o = op(ctx, &id, Op::CreateJournal(j.clone()), journal_request(&j));
        o.preconditions = vec![
            Precondition::MarkerAbsent { marker: m },
            Precondition::AccountsActive { codes },
            open.clone(),
        ];
        o.expected = ctx.moves(
            DocKind::ManualJournal,
            None,
            Some(&as_posted(&journal_doc(&j))),
        )?;
        o.recovery = Some(Box::new(Recovery {
            note: format!(
                "a reversing journal (negated lines, same date, marker {}); voiding the created journal is preferable only if no report has been exported or relied on since",
                reversal_marker(&id)
            ),
            op: Some(Op::CreateJournal(rev)),
        }));
        if n > 1 {
            o.flags.push(format!("split {}/{n}", i + 1));
        }
        out.push(o);
    }
    Ok(out)
}

// ---------------------------------------------------------------- OP B: recode a bank-transaction line

#[derive(Debug, Clone, PartialEq)]
pub struct RecodeSpec {
    pub id: String,
    pub line_item_id: String,
    pub account: Option<String>,
    pub description: Option<String>,
}

pub fn recode_bank_line(
    ctx: &OrgCtx,
    target: &Row,
    spec: &RecodeSpec,
) -> Result<Operation, WriteError> {
    let doc = strip_marks(&target.payload);
    let bt = target.xero_id.clone();
    let status = s(&doc, "Status");
    if status != "AUTHORISED" {
        return Err(usage(format!(
            "{}: bank transaction {bt} is {status}; only AUTHORISED ones are recoded",
            spec.id
        )));
    }
    let t = s(&doc, "Type");
    if !matches!(t, "SPEND" | "RECEIVE") {
        return Err(usage(format!(
            "{}: a {t} bank transaction is not recoded in v1 (SPEND and RECEIVE only)",
            spec.id
        )));
    }
    if spec.account.is_none() && spec.description.is_none() {
        return Err(usage(format!("{}: nothing to change", spec.id)));
    }
    let lines = doc
        .get("LineItems")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    if lines.is_empty() || lines.iter().any(|l| s(l, "LineItemID").is_empty()) {
        return Err(usage(format!(
            "{}: bank transaction {bt} has lines without a LineItemID in the record, and a line is never chosen by position",
            spec.id
        ))
        .help(format!(
            "re-sync the org (`ownpurse sync {}`); line ids come with the full document",
            ctx.org
        )));
    }
    let old = lines
        .iter()
        .find(|l| s(l, "LineItemID") == spec.line_item_id)
        .ok_or_else(|| {
            usage(format!(
                "{}: bank transaction {bt} has no line {}",
                spec.id, spec.line_item_id
            ))
        })?;
    let from_code = s(old, "AccountCode").to_string();
    let from_desc = s(old, "Description").to_string();
    let new_acct = match &spec.account {
        Some(c) => Some(ctx.account(c)?),
        None => None,
    };
    let r = Recode {
        bank_transaction_id: bt.clone(),
        line_item_id: spec.line_item_id.clone(),
        from_account_code: from_code.clone(),
        from_description: from_desc.clone(),
        to_account_code: spec.account.clone().filter(|c| *c != from_code),
        to_description: spec
            .description
            .as_deref()
            .map(fold)
            .filter(|d| *d != from_desc),
    };
    if r.to_account_code.is_none() && r.to_description.is_none() {
        return Err(usage(format!(
            "{}: line {} already has that account and description",
            spec.id, spec.line_item_id
        )));
    }
    // Xero refuses any edit of a reconciled bank transaction (400 "cannot be edited as it has been
    // reconciled with a Bank Statement", seen on the Demo Company): refused here, never sent
    if doc.get("IsReconciled").and_then(Value::as_bool) == Some(true) {
        return Err(WriteError::new(
            "reconciled_line",
            exit::USAGE,
            format!("{}: bank transaction {bt} is reconciled; Xero refuses to edit a reconciled transaction", spec.id),
        )
        .help(match ctx.short_code.as_deref().and_then(|c| crate::deeplink::url_for(c, "BankTransactions", "", &bt)) {
            Some(u) => format!("Use a reclass journal, or the browser path (unreconcile, edit, reconcile again): {u}"),
            None => "Use a reclass journal, or the browser path (unreconcile, edit, reconcile again).".into(),
        }));
    }
    let (after, sent) = recoded(&doc, &r, new_acct.as_ref().map(|a| a.account_id.as_str()))?;
    let date = doc_date(&doc);
    let open = ctx.period_open(&date)?;
    let mut o = op(
        ctx,
        &spec.id,
        Op::RecodeBankLine(r.clone()),
        recode_request(&bt, sent),
    );
    o.preconditions.push(Precondition::Target {
        entity: "BankTransactions".into(),
        id: bt.clone(),
        fingerprint: fingerprint(&doc),
        version: Some(target.version),
        status: status.into(),
        before: Some(super::business_view(&doc)),
    });
    if let Some(c) = &r.to_account_code {
        o.preconditions.push(Precondition::AccountsActive {
            codes: vec![c.clone()],
        });
    }
    o.preconditions.push(open);
    o.expected_after = Some(fingerprint(&after));
    o.expected = ctx.moves(DocKind::BankTransaction, Some(&doc), Some(&after))?;
    o.recovery = Some(Box::new(Recovery {
        note: "the inverse recode back to the line as it was (never a void: the payment is real money)"
            .into(),
        op: Some(Op::RecodeBankLine(Recode {
            bank_transaction_id: bt,
            line_item_id: r.line_item_id.clone(),
            from_account_code: r.to_account_code.clone().unwrap_or(from_code.clone()),
            from_description: r.to_description.clone().unwrap_or(from_desc.clone()),
            to_account_code: r.to_account_code.as_ref().map(|_| from_code),
            to_description: r.to_description.as_ref().map(|_| from_desc),
        })),
    }));
    Ok(o)
}

// ---------------------------------------------------------------- OP C: void a manual journal

pub fn void_journal(ctx: &OrgCtx, op_id: &str, target: &Row) -> Result<Operation, WriteError> {
    let doc = strip_marks(&target.payload);
    let id = target.xero_id.clone();
    let status = s(&doc, "Status");
    if status != "POSTED" {
        return Err(usage(format!(
            "{op_id}: journal {id} is {status}; only POSTED journals are voided"
        )));
    }
    let date = doc_date(&doc);
    let open = ctx.period_open(&date)?;
    let mut after = doc.clone();
    if let Some(o) = after.as_object_mut() {
        o.insert("Status".into(), Value::String("VOIDED".into()));
    }
    let narration = s(&doc, "Narration").to_string();
    let cash = doc
        .get("ShowOnCashBasisReports")
        .and_then(Value::as_bool)
        .unwrap_or(true);
    let lat = Some(s(&doc, "LineAmountTypes"))
        .filter(|x| !x.is_empty())
        .unwrap_or("NoTax");
    let redo = Journal {
        date: date.clone(),
        narration: format!(
            "{} - re-created - {}",
            fold(&narration),
            reversal_marker(op_id)
        ),
        lines: journal_lines(&doc),
        line_amount_types: lat.into(),
        show_on_cash_basis_reports: cash,
    };
    let mut o = op(
        ctx,
        op_id,
        Op::VoidJournal {
            journal: JournalRef::Id(id.clone()),
        },
        void_request(&id),
    );
    o.preconditions = vec![
        Precondition::Target {
            entity: "ManualJournals".into(),
            id: id.clone(),
            fingerprint: fingerprint(&doc),
            version: Some(target.version),
            status: status.into(),
            before: Some(super::business_view(&doc)),
        },
        open,
    ];
    o.expected_after = Some(fingerprint(&after));
    o.expected = ctx.moves(DocKind::ManualJournal, Some(&doc), Some(&after))?;
    o.recovery = Some(Box::new(Recovery {
        note: "a voided journal can't be un-voided: re-create it from the stored before-snapshot"
            .into(),
        op: Some(Op::CreateJournal(redo)),
    }));
    Ok(o)
}

/// OP C on the journal an earlier op of this package creates (e.g. a probe: create → attach → void). The
/// journal doesn't exist at plan time: no target precondition (the run fills `{id}` from the create's
/// confirmed response); the expected movement is the create's, negated.
pub fn void_created(
    ctx: &OrgCtx,
    op_id: &str,
    create: &Operation,
) -> Result<Operation, WriteError> {
    let Op::CreateJournal(j) = &create.op else {
        return Err(usage(format!(
            "{op_id}: op:{} is not a journal create",
            create.id
        )));
    };
    let open = ctx.period_open(&j.date)?;
    let mut o = op(
        ctx,
        op_id,
        Op::VoidJournal {
            journal: JournalRef::CreatedBy(create.id.clone()),
        },
        void_request("{id}"),
    );
    o.preconditions.push(open);
    o.depends_on.push(create.id.clone());
    o.expected = create
        .expected
        .iter()
        .map(|e| ExpectedDelta {
            amount: -e.amount,
            ..e.clone()
        })
        .collect();
    let redo = Journal {
        narration: format!(
            "{} - re-created - {}",
            fold(&j.narration),
            reversal_marker(op_id)
        ),
        ..j.clone()
    };
    o.recovery = Some(Box::new(Recovery {
        note: "a voided journal can't be un-voided: re-create it from the package's own journal"
            .into(),
        op: Some(Op::CreateJournal(redo)),
    }));
    Ok(o)
}

// ---------------------------------------------------------------- OP D: attach a file to a journal

// ---------------------------------------------------------------- OP F/G: bank transactions

impl OrgCtx {
    /// The org's bank account by code, AccountID or exact name: exactly one, ACTIVE, type BANK.
    pub fn bank_account(&self, key: &str) -> Result<(String, String), WriteError> {
        let hits: Vec<&Value> = self
            .accounts
            .iter()
            .filter(|a| {
                s(a, "Type") == "BANK"
                    && (s(a, "Code") == key || s(a, "AccountID") == key || s(a, "Name") == key)
            })
            .collect();
        match hits.as_slice() {
            [a] if s(a, "Status") == "ACTIVE" => Ok((
                s(a, "AccountID").to_string(),
                format!("{} {}", s(a, "Code"), s(a, "Name"))
                    .trim()
                    .to_string(),
            )),
            [a] => Err(usage(format!(
                "{}: bank account {key} is {}, not ACTIVE",
                self.org,
                s(a, "Status")
            ))),
            [] => Err(usage(format!(
                "{}: no bank account {key} (a code, AccountID or exact name of a BANK account)",
                self.org
            ))),
            _ => Err(usage(format!(
                "{}: bank account {key} is ambiguous",
                self.org
            ))),
        }
    }
}

/// What a bank transaction create asks for (the plan input, resolved).
#[derive(Debug, Clone, PartialEq)]
pub struct BankTxnSpec {
    pub id: String,
    /// SPEND or RECEIVE.
    pub kind: String,
    pub bank_account: String,
    /// A ContactID, or `op:<id>` for the contact an earlier op creates.
    pub contact: JournalRef,
    pub date: String,
    /// Without the marker (added here).
    pub reference: String,
    pub lines: Vec<JournalLine>,
}

/// The Xero body of a bank transaction (a created contact's id stays `{op:<id>}` until the run fills it).
pub fn bank_txn_doc(b: &BankTxn) -> Value {
    let contact = match &b.contact {
        JournalRef::Id(id) => id.clone(),
        JournalRef::CreatedBy(op) => format!("{{op:{op}}}"),
    };
    json!({
        "Type": b.kind,
        "Contact": {"ContactID": contact},
        "BankAccount": {"AccountID": b.bank_account_id},
        "Date": b.date,
        "Reference": b.reference,
        "Status": "AUTHORISED",
        "LineAmountTypes": b.line_amount_types,
        "LineItems": b.lines.iter().map(|l| {
            let mut m = Map::new();
            if !l.description.is_empty() {
                m.insert("Description".into(), Value::String(fold(&l.description)));
            }
            m.insert("Quantity".into(), json!(1));
            m.insert("UnitAmount".into(), num(l.amount));
            m.insert("LineAmount".into(), num(l.amount));
            m.insert("AccountCode".into(), Value::String(l.account_code.clone()));
            Value::Object(m)
        }).collect::<Vec<_>>(),
    })
}

/// OP F's request: `PUT BankTransactions`.
pub fn bank_txn_request(b: &BankTxn) -> WriteRequest {
    WriteRequest {
        method: Method::Put,
        path: "BankTransactions".into(),
        body: Body::Json {
            json: json!({ "BankTransactions": [bank_txn_doc(b)] }),
        },
    }
}

/// OP G's request: `POST BankTransactions/{id}` with `Status: DELETED` (the API's void).
pub fn void_bank_request(id: &str) -> WriteRequest {
    WriteRequest {
        method: Method::Post,
        path: format!("BankTransactions/{id}"),
        body: Body::Json {
            json: json!({ "BankTransactions": [{ "BankTransactionID": id, "Status": "DELETED" }] }),
        },
    }
}

/// The ledger's view of a bank transaction this tool would create (for the expected movement).
fn bank_as_posted(b: &BankTxn) -> Value {
    json!({
        "BankTransactionID": "(new)",
        "Type": b.kind,
        "Status": "AUTHORISED",
        "BankAccount": {"AccountID": b.bank_account_id},
        "DateString": b.date,
        "LineAmountTypes": b.line_amount_types,
        "Total": num(b.total()),
        "LineItems": b.lines.iter().map(|l| json!({"AccountCode": l.account_code, "LineAmount": num(l.amount)})).collect::<Vec<_>>(),
    })
}

/// OP F: a SPEND or RECEIVE bank transaction. Preconditions: no transaction carries the marker, the
/// line accounts and the bank account exist once and are ACTIVE, the date is open. Recovery: void it
/// (op G: `Status: DELETED`; refused by Xero once reconciled).
pub fn create_bank_transaction(ctx: &OrgCtx, spec: &BankTxnSpec) -> Result<Operation, WriteError> {
    date_ok(&spec.date)?;
    let kind = spec.kind.to_ascii_uppercase();
    if kind != "SPEND" && kind != "RECEIVE" {
        return Err(usage(format!(
            "{}: a bank transaction is SPEND or RECEIVE, not {}",
            spec.id, spec.kind
        )));
    }
    if spec.lines.is_empty() {
        return Err(usage(format!(
            "{}: a bank transaction needs a line",
            spec.id
        )));
    }
    for l in &spec.lines {
        if l.amount <= Decimal::ZERO || l.amount.round_dp(2) != l.amount {
            return Err(usage(format!(
                "{}: line {} {}: amounts are positive with at most 2 decimals (the type gives the direction)",
                spec.id, l.account_code, l.amount
            )));
        }
        ctx.account(&l.account_code)?;
    }
    let (bank_id, bank_name) = ctx.bank_account(&spec.bank_account)?;
    let open = ctx.period_open(&spec.date)?;
    let m = marker(&ctx.package_id, &spec.id);
    let reference = if spec.reference.trim().is_empty() {
        m.clone()
    } else {
        format!("{} - {m}", fold(spec.reference.trim()))
    };
    let b = BankTxn {
        kind,
        bank_account_id: bank_id,
        bank_account: bank_name,
        contact: spec.contact.clone(),
        date: spec.date.clone(),
        reference,
        lines: spec.lines.clone(),
        line_amount_types: "NoTax".into(),
    };
    let mut codes: Vec<String> = b.lines.iter().map(|l| l.account_code.clone()).collect();
    codes.sort();
    codes.dedup();
    let mut o = op(
        ctx,
        &spec.id,
        Op::CreateBankTransaction(b.clone()),
        bank_txn_request(&b),
    );
    o.preconditions = vec![
        Precondition::ReferenceAbsent {
            marker: m,
            entity: None,
        },
        Precondition::AccountsActive { codes },
        open,
    ];
    if let JournalRef::CreatedBy(c) = &b.contact {
        o.depends_on.push(c.clone());
    }
    o.expected = ctx.moves(DocKind::BankTransaction, None, Some(&bank_as_posted(&b)))?;
    o.recovery = Some(Box::new(Recovery {
        note: "void it (Status DELETED), while it is unreconciled; a reconciled one is unreconciled in the browser first".into(),
        op: Some(Op::VoidBankTransaction {
            bank_transaction: JournalRef::CreatedBy(spec.id.clone()),
        }),
    }));
    Ok(o)
}

/// OP G on an existing bank transaction (from the record): AUTHORISED and unreconciled.
pub fn void_bank_transaction(
    ctx: &OrgCtx,
    op_id: &str,
    target: &Row,
) -> Result<Operation, WriteError> {
    let doc = strip_marks(&target.payload);
    let id = target.xero_id.clone();
    let status = s(&doc, "Status");
    if status != "AUTHORISED" {
        return Err(usage(format!(
            "{op_id}: bank transaction {id} is {status}; only AUTHORISED ones are voided"
        )));
    }
    if doc.get("IsReconciled").and_then(Value::as_bool) == Some(true) {
        return Err(WriteError::new(
            "reconciled_line",
            exit::USAGE,
            format!("{op_id}: bank transaction {id} is reconciled; Xero refuses to void it"),
        )
        .help("Unreconcile it in the browser first, then plan the void again."));
    }
    let date = doc_date(&doc);
    let open = ctx.period_open(&date)?;
    let mut after = doc.clone();
    if let Some(o) = after.as_object_mut() {
        o.insert("Status".into(), Value::String("DELETED".into()));
    }
    let mut o = op(
        ctx,
        op_id,
        Op::VoidBankTransaction {
            bank_transaction: JournalRef::Id(id.clone()),
        },
        void_bank_request(&id),
    );
    o.preconditions = vec![
        Precondition::Target {
            entity: "BankTransactions".into(),
            id: id.clone(),
            fingerprint: fingerprint(&doc),
            version: Some(target.version),
            status: status.into(),
            before: Some(super::business_view(&doc)),
        },
        open,
    ];
    o.expected = ctx.moves(DocKind::BankTransaction, Some(&doc), None)?;
    o.recovery = Some(Box::new(Recovery {
        note: "a deleted bank transaction can't be restored: re-create it from the stored before-snapshot (a new id)".into(),
        op: None,
    }));
    o.flags.push("permanent void".into());
    Ok(o)
}

/// OP G on the bank transaction an earlier op of this package creates (its recovery, or a probe).
pub fn void_created_bank(
    ctx: &OrgCtx,
    op_id: &str,
    create: &Operation,
) -> Result<Operation, WriteError> {
    let Op::CreateBankTransaction(b) = &create.op else {
        return Err(usage(format!(
            "{op_id}: op:{} is not a bank transaction create",
            create.id
        )));
    };
    let open = ctx.period_open(&b.date)?;
    let mut o = op(
        ctx,
        op_id,
        Op::VoidBankTransaction {
            bank_transaction: JournalRef::CreatedBy(create.id.clone()),
        },
        void_bank_request("{id}"),
    );
    o.preconditions.push(open);
    o.depends_on.push(create.id.clone());
    o.expected = create
        .expected
        .iter()
        .map(|e| ExpectedDelta {
            amount: -e.amount,
            ..e.clone()
        })
        .collect();
    o.recovery = Some(Box::new(Recovery {
        note: "re-create it from the package's own transaction (a new id)".into(),
        op: None,
    }));
    Ok(o)
}

// ---------------------------------------------------------------- OP H: a bank transfer

/// OP H's request: `PUT BankTransfers`.
pub fn transfer_request(t: &Transfer) -> WriteRequest {
    WriteRequest {
        method: Method::Put,
        path: "BankTransfers".into(),
        body: Body::Json {
            json: json!({ "BankTransfers": [{
                "FromBankAccount": {"AccountID": t.from_account_id},
                "ToBankAccount": {"AccountID": t.to_account_id},
                "Amount": num(t.amount),
                "Date": t.date,
                "Reference": t.reference,
            }] }),
        },
    }
}

/// The ledger's view of a transfer this tool would create (for the expected movement).
fn transfer_as_posted(t: &Transfer) -> Value {
    json!({
        "BankTransferID": "(new)",
        "FromBankAccount": {"AccountID": t.from_account_id},
        "ToBankAccount": {"AccountID": t.to_account_id},
        "Amount": num(t.amount),
        "DateString": t.date,
    })
}

/// OP H: a transfer between two of the org's bank accounts (petty cash funded from the bank). The API can't
/// void a transfer: the recovery is a reverse transfer, and both stay in the accounts' history (flagged).
pub fn create_bank_transfer(
    ctx: &OrgCtx,
    op_id: &str,
    from: &str,
    to: &str,
    amount: Decimal,
    date: &str,
    reference: &str,
) -> Result<Operation, WriteError> {
    date_ok(date)?;
    if amount <= Decimal::ZERO || amount.round_dp(2) != amount {
        return Err(usage(format!(
            "{op_id}: a transfer amount is positive with at most 2 decimals"
        )));
    }
    let (from_id, from_name) = ctx.bank_account(from)?;
    let (to_id, to_name) = ctx.bank_account(to)?;
    if from_id == to_id {
        return Err(usage(format!(
            "{op_id}: a transfer needs two different bank accounts"
        )));
    }
    let open = ctx.period_open(date)?;
    let m = marker(&ctx.package_id, op_id);
    let reference = if reference.trim().is_empty() {
        m.clone()
    } else {
        format!("{} - {m}", crate::text::ascii_fold(reference.trim()))
    };
    let t = Transfer {
        from_account_id: from_id.clone(),
        from_account: from_name,
        to_account_id: to_id.clone(),
        to_account: to_name,
        amount,
        date: date.into(),
        reference,
    };
    let mut o = op(
        ctx,
        op_id,
        Op::CreateBankTransfer(t.clone()),
        transfer_request(&t),
    );
    o.preconditions = vec![
        Precondition::ReferenceAbsent {
            marker: m,
            entity: Some("BankTransfers".into()),
        },
        Precondition::Exists {
            entity: "Accounts".into(),
            id: from_id,
        },
        Precondition::Exists {
            entity: "Accounts".into(),
            id: to_id,
        },
        open,
    ];
    o.expected = ctx.moves(DocKind::BankTransfer, None, Some(&transfer_as_posted(&t)))?;
    let back = Transfer {
        from_account_id: t.to_account_id.clone(),
        from_account: t.to_account.clone(),
        to_account_id: t.from_account_id.clone(),
        to_account: t.from_account.clone(),
        reference: format!("reverses {op_id} - {}", reversal_marker(op_id)),
        ..t.clone()
    };
    o.recovery = Some(Box::new(Recovery {
        note: "the API can't void a transfer: a reverse transfer (same amount, the accounts swapped); both stay in the accounts' history".into(),
        op: Some(Op::CreateBankTransfer(back)),
    }));
    o.flags
        .push("permanent history: a transfer can't be voided through the API".into());
    Ok(o)
}

// ---------------------------------------------------------------- OP I/J: contacts

/// OP I's request: `PUT Contacts`.
pub fn contact_request(c: &NewContact) -> WriteRequest {
    let mut m = Map::new();
    m.insert("Name".into(), Value::String(c.name.clone()));
    if let Some(e) = &c.email {
        m.insert("EmailAddress".into(), Value::String(e.clone()));
    }
    WriteRequest {
        method: Method::Put,
        path: "Contacts".into(),
        body: Body::Json {
            json: json!({ "Contacts": [Value::Object(m)] }),
        },
    }
}

/// OP J's request: `POST Contacts/{id}` with `ContactStatus: ARCHIVED`.
pub fn archive_contact_request(id: &str) -> WriteRequest {
    WriteRequest {
        method: Method::Post,
        path: format!("Contacts/{id}"),
        body: Body::Json {
            json: json!({ "Contacts": [{ "ContactID": id, "ContactStatus": "ARCHIVED" }] }),
        },
    }
}

/// OP I: a contact by name (and an optional email). No account moves. Precondition: no contact has
/// exactly this name (checked live; the plan also checks the record). Recovery: archive it (op J).
pub fn create_contact(
    ctx: &OrgCtx,
    op_id: &str,
    name: &str,
    email: Option<&str>,
) -> Result<Operation, WriteError> {
    let name = name.trim();
    if name.is_empty() || name.chars().count() > 255 {
        return Err(usage(format!(
            "{op_id}: a contact name has 1 to 255 characters"
        )));
    }
    let email = email.map(str::trim).filter(|e| !e.is_empty());
    if let Some(e) = email
        && (!e.contains('@') || e.contains(char::is_whitespace))
    {
        return Err(usage(format!("{op_id}: {e:?} is not an email address")));
    }
    let c = NewContact {
        name: name.into(),
        email: email.map(str::to_string),
    };
    let mut o = op(
        ctx,
        op_id,
        Op::CreateContact(c.clone()),
        contact_request(&c),
    );
    o.preconditions = vec![Precondition::ContactNameAbsent { name: name.into() }];
    o.recovery = Some(Box::new(Recovery {
        note: "archive it (a contact can't be deleted once used; archived, it no longer shows)"
            .into(),
        op: Some(Op::ArchiveContact {
            contact: JournalRef::CreatedBy(op_id.into()),
        }),
    }));
    Ok(o)
}

/// OP J on an existing contact (from the record) or on the one an earlier op creates.
pub fn archive_contact(ctx: &OrgCtx, op_id: &str, contact: JournalRef) -> Operation {
    let id = match &contact {
        JournalRef::Id(id) => id.clone(),
        JournalRef::CreatedBy(_) => "{id}".into(),
    };
    let mut o = op(
        ctx,
        op_id,
        Op::ArchiveContact {
            contact: contact.clone(),
        },
        archive_contact_request(&id),
    );
    match contact {
        JournalRef::Id(id) => o.preconditions.push(Precondition::Exists {
            entity: "Contacts".into(),
            id,
        }),
        JournalRef::CreatedBy(c) => o.depends_on.push(c),
    }
    o.recovery = Some(Box::new(Recovery {
        note: "un-archive it in Xero (browser), or a recovery package setting it ACTIVE".into(),
        op: None,
    }));
    o
}

// ---------------------------------------------------------------- OP E: a note in a document's history

/// OP E's request: `PUT {Entity}/{id}/History` (`{id}` stays literal for a document an earlier op creates).
pub fn note_request(entity: &str, target: &JournalRef, details: &str) -> WriteRequest {
    let id = match target {
        JournalRef::Id(id) => id.clone(),
        JournalRef::CreatedBy(_) => "{id}".into(),
    };
    WriteRequest {
        method: Method::Put,
        path: format!("{entity}/{id}/History"),
        body: Body::Json {
            json: json!({ "HistoryRecords": [{ "Details": details }] }),
        },
    }
}

/// OP E: a note on an existing document (`target` from the record) or on the one op `created_by` makes.
/// No account moves; no recovery (history can't be deleted): flagged `permanent history`.
pub fn add_note(
    ctx: &OrgCtx,
    op_id: &str,
    entity: &str,
    target: JournalRef,
    details: &str,
) -> Result<Operation, WriteError> {
    let entity = super::NOTE_ENTITIES
        .iter()
        .copied()
        .find(|e| e.eq_ignore_ascii_case(entity))
        .ok_or_else(|| {
            usage(format!(
                "{op_id}: notes go on {}, not {entity}",
                super::NOTE_ENTITIES.join(", ")
            ))
        })?;
    // ASCII, as Xero will hold it (other characters come back as HTML entities)
    let folded = crate::text::ascii_fold(details.trim());
    let details = folded.as_str();
    if details.is_empty() {
        return Err(usage(format!("{op_id}: an empty note")));
    }
    if details.chars().count() > super::NOTE_MAX {
        return Err(usage(format!(
            "{op_id}: a note is at most {} characters ({} given)",
            super::NOTE_MAX,
            details.chars().count()
        )));
    }
    let n = Note {
        entity: entity.into(),
        target: target.clone(),
        details: details.into(),
    };
    let mut o = op(
        ctx,
        op_id,
        Op::AddNote(n),
        note_request(entity, &target, details),
    );
    match &target {
        JournalRef::Id(id) => o.preconditions.push(Precondition::Exists {
            entity: entity.into(),
            id: id.clone(),
        }),
        JournalRef::CreatedBy(c) => o.depends_on.push(c.clone()),
    }
    o.recovery = Some(Box::new(Recovery {
        note: "none: Xero keeps history notes for good; a correcting note can follow".into(),
        op: None,
    }));
    o.flags.push("permanent history".into());
    Ok(o)
}

pub fn mime_of(name: &str) -> Option<&'static str> {
    let ext = name.rsplit_once('.')?.1.to_ascii_lowercase();
    match ext.as_str() {
        "pdf" => Some("application/pdf"),
        "csv" => Some("text/csv"),
        "txt" => Some("text/plain"),
        _ => None,
    }
}

/// The document an attachment goes on.
pub enum AttachTo<'a> {
    /// An existing journal (from the record).
    Journal(&'a Row),
    /// An existing bank transaction (from the record).
    BankTransaction(&'a Row),
    /// The journal an earlier op of this package creates.
    CreatedBy(String),
}

pub fn attach(
    ctx: &OrgCtx,
    op_id: &str,
    to: AttachTo<'_>,
    file: &std::path::Path,
) -> Result<Operation, WriteError> {
    let name = file
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| usage(format!("{op_id}: {} has no file name", file.display())))?
        .to_string();
    let mime = mime_of(&name).ok_or_else(|| {
        usage(format!(
            "{op_id}: {name}: only PDF, CSV or TXT attach (Xero rejects Markdown and others)"
        ))
    })?;
    if !name
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || "._-".contains(c))
    {
        return Err(usage(format!(
            "{op_id}: {name}: use letters, digits, '.', '_' and '-' in attachment names"
        )));
    }
    let bytes = std::fs::read(file).map_err(|e| {
        WriteError::new(
            "io",
            exit::RUNTIME,
            format!("{op_id}: {}: {e}", file.display()),
        )
    })?;
    if bytes.is_empty() {
        return Err(usage(format!("{op_id}: {name} is empty")));
    }
    if bytes.len() as u64 > MAX_ATTACHMENT {
        return Err(usage(format!(
            "{op_id}: {name} is {} bytes; Xero's limit is 10 MB",
            bytes.len()
        )));
    }
    let a = Attachment {
        journal: match &to {
            AttachTo::Journal(r) | AttachTo::BankTransaction(r) => {
                JournalRef::Id(r.xero_id.clone())
            }
            AttachTo::CreatedBy(op) => JournalRef::CreatedBy(op.clone()),
        },
        entity: match &to {
            AttachTo::BankTransaction(_) => "BankTransactions".into(),
            _ => "ManualJournals".into(),
        },
        file_name: name,
        mime: mime.into(),
        sha256: sha256_hex(&bytes),
        bytes: bytes.len() as u64,
        path: file.display().to_string(),
    };
    let mut o = op(ctx, op_id, Op::Attach(a.clone()), attach_request(&a));
    match to {
        AttachTo::Journal(r) | AttachTo::BankTransaction(r) => {
            let doc = strip_marks(&r.payload);
            o.preconditions.push(Precondition::Target {
                entity: a.entity.clone(),
                id: r.xero_id.clone(),
                fingerprint: fingerprint(&doc),
                version: Some(r.version),
                status: s(&doc, "Status").into(),
                before: Some(super::business_view(&doc)),
            });
        }
        AttachTo::CreatedBy(op) => o.depends_on.push(op),
    }
    o.preconditions.push(Precondition::AttachmentAbsent {
        file_name: a.file_name.clone(),
        sha256: a.sha256.clone(),
    });
    o.recovery = Some(Box::new(Recovery {
        note: "none through the API: attachments can't be deleted; a corrected file is uploaded alongside, with a note"
            .into(),
        op: None,
    }));
    o.flags.push("permanent attachment".into());
    Ok(o)
}
