//! Local double-entry ledger rebuilt from mirrored source documents (all exact decimals). LEDGER_VERSION
//! 2: debits positive, credits negative; every document's
//! postings sum to zero or the document is reported as unbalanced.
//!
//!   BankTransactions  SPEND*/RECEIVE* (AUTHORISED): bank account vs line accounts (+ tax);
//!                     *-TRANSFER legs are skipped (BankTransfers carries the transfer)
//!   BankTransfers     Dr to-bank, Cr from-bank
//!   ManualJournals    POSTED lines (positive LineAmount = debit)
//!   Invoices          AUTHORISED/PAID: ACCREC Dr receivables / ACCPAY Cr payables vs lines (+ tax)
//!   CreditNotes       AUTHORISED/PAID: the reverse of the matching invoice type
//!   Payments          AUTHORISED: bank vs receivables/payables by PaymentType
//!
//! Cash basis (`LedgerBasis::Cash`, Xero's `paymentsOnly`): invoices and credit notes post nothing; each payment
//! posts the bank leg against the document's own lines (and tax), in proportion to the amount paid, at the
//! payment date. Manual journals with `ShowOnCashBasisReports: false` are left out.

use crate::dates::doc_date;
use crate::money::{MoneyError, ZERO, money};
use crate::record::RecordError;
use rust_decimal::Decimal;
use serde_json::Value;
use std::collections::{BTreeMap, HashMap};

pub const LEDGER_VERSION: &str = "2";

/// Accrual (Xero's default) or cash (`paymentsOnly`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LedgerBasis {
    #[default]
    Accrual,
    Cash,
}

/// An invoice or credit note held back on cash basis until its payments post it.
#[derive(Debug, Clone)]
struct CashDoc {
    lines: Vec<Entry>,
    number: String,
    contact: String,
}
pub const UNCODED: &str = "(uncoded)";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Account {
    pub account_id: String,
    pub code: String,
    pub name: String,
    pub kind: String, // Xero `Type` (BANK, CURRENT, EXPENSE, …)
    pub class: String,
    pub system: String,
    pub status: String,
}

impl Account {
    pub fn label(&self) -> String {
        if self.code.is_empty() {
            self.name.clone()
        } else {
            format!("{} ({})", self.name, self.code)
        }
    }
    pub fn is_pl(&self) -> bool {
        self.class == "REVENUE" || self.class == "EXPENSE"
    }
}

#[derive(Debug, Clone, Default)]
pub struct Chart {
    pub by_id: HashMap<String, Account>,
    pub by_code: HashMap<String, String>,
    pub by_system: HashMap<String, String>,
    /// Account ids in the record's order (by Xero id), for deterministic iteration.
    pub order: Vec<String>,
}

fn s(p: &Value, k: &str) -> String {
    p.get(k).and_then(Value::as_str).unwrap_or("").to_string()
}

impl Chart {
    pub fn new(rows: &[Value]) -> Chart {
        let mut c = Chart::default();
        for p in rows {
            let a = Account {
                account_id: s(p, "AccountID"),
                code: s(p, "Code"),
                name: s(p, "Name"),
                kind: s(p, "Type"),
                class: s(p, "Class"),
                system: s(p, "SystemAccount"),
                status: s(p, "Status"),
            };
            if !a.code.is_empty() {
                c.by_code.insert(a.code.clone(), a.account_id.clone());
            }
            if !a.system.is_empty() {
                c.by_system.insert(a.system.clone(), a.account_id.clone());
            }
            c.order.push(a.account_id.clone());
            c.by_id.insert(a.account_id.clone(), a);
        }
        c
    }

    pub fn resolve(&self, code: Option<&str>, account_id: Option<&str>) -> String {
        if let Some(id) = account_id.filter(|i| !i.is_empty() && self.by_id.contains_key(*i)) {
            return id.to_string();
        }
        if let Some(id) = code
            .filter(|c| !c.is_empty())
            .and_then(|c| self.by_code.get(c))
        {
            return id.clone();
        }
        let key = code
            .filter(|c| !c.is_empty())
            .or(account_id.filter(|i| !i.is_empty()))
            .unwrap_or("");
        format!("{UNCODED}:{key}")
    }

    pub fn system(&self, name: &str) -> String {
        self.by_system
            .get(name)
            .cloned()
            .unwrap_or_else(|| format!("{UNCODED}:system:{name}"))
    }

    /// The account, or a placeholder named after the id.
    pub fn get(&self, account_id: &str) -> Account {
        self.by_id
            .get(account_id)
            .cloned()
            .unwrap_or_else(|| Account {
                account_id: account_id.into(),
                code: String::new(),
                name: account_id.into(),
                kind: String::new(),
                class: String::new(),
                system: String::new(),
                status: String::new(),
            })
    }

    pub fn is_pl(&self, account_id: &str) -> bool {
        self.by_id.get(account_id).is_some_and(Account::is_pl)
    }

    pub fn by_code(&self, code: &str) -> Option<&Account> {
        self.by_code.get(code).and_then(|id| self.by_id.get(id))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Posting {
    pub date: String,
    pub source: &'static str,
    pub source_id: String,
    pub line: usize,
    pub account_id: String,
    pub amount: Decimal, // debit positive, credit negative
    pub description: String,
    pub contact: String,
    pub reference: String,
    pub kind: String, // human label: Spend, Receive, Transfer, Journal, Invoice, …
    /// Proposed by a manifest: which entry, of what kind. None on main.
    pub proposed: Option<Proposed>,
}

/// A posting proposed by a manifest entry (`View` marks the documents; the builder carries the mark).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Proposed {
    /// The entry number (`entry.add` event id).
    pub seq: i64,
    /// `create`, `change` or `void`.
    pub kind: &'static str,
    /// Added by an agent (`A+`).
    pub agent: bool,
    /// For a recode: the account id the line had on main.
    pub was: Option<String>,
}

impl Proposed {
    /// The gutter glyph: `+`, `~` or `⊘`.
    pub fn glyph(&self) -> &'static str {
        match self.kind {
            "create" => "+",
            "change" => "~",
            _ => "⊘",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Unbalanced {
    pub source: &'static str,
    pub id: String,
    pub date: String,
    pub imbalance: Decimal,
}

#[derive(Debug, Clone, Default)]
pub struct Ledger {
    pub chart: Chart,
    pub postings: Vec<Posting>,
    /// The postings of documents a manifest proposes to void (shown struck through; never in balances).
    pub ghosts: Vec<Posting>,
    /// Main's postings of the documents a manifest CHANGES (not voids, not new ones): with the view's
    /// postings of those documents, they give the change's exact effect.
    pub replaced: Vec<Posting>,
    pub unbalanced: Vec<Unbalanced>,
    pub skipped: BTreeMap<String, usize>,
    pub basis: LedgerBasis,
    cash_docs: HashMap<String, CashDoc>,
}

#[derive(Debug)]
pub enum LedgerError {
    Record(RecordError),
    Money {
        source: &'static str,
        id: String,
        error: MoneyError,
    },
}

impl std::fmt::Display for LedgerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LedgerError::Record(e) => e.fmt(f),
            LedgerError::Money { source, id, error } => write!(f, "{source} {id}: {error}"),
        }
    }
}

impl std::error::Error for LedgerError {}

impl From<RecordError> for LedgerError {
    fn from(e: RecordError) -> Self {
        LedgerError::Record(e)
    }
}

/// Title case for Xero type codes (e.g. `SPEND-TRANSFER` → `Spend-Transfer`).
fn title(t: &str) -> String {
    let mut out = String::with_capacity(t.len());
    let mut up = true;
    for ch in t.chars() {
        if ch.is_alphabetic() {
            if up {
                out.extend(ch.to_uppercase())
            } else {
                out.extend(ch.to_lowercase())
            }
            up = false;
        } else {
            out.push(ch);
            up = true;
        }
    }
    out
}

fn kind_label(t: &str) -> String {
    match t {
        "SPEND" => "Spend".into(),
        "RECEIVE" => "Receive".into(),
        "SPEND-OVERPAYMENT" | "RECEIVE-OVERPAYMENT" => "Overpayment".into(),
        "SPEND-PREPAYMENT" | "RECEIVE-PREPAYMENT" => "Prepayment".into(),
        "ACCREC" => "Invoice".into(),
        "ACCPAY" => "Bill".into(),
        "ACCRECCREDIT" | "ACCPAYCREDIT" => "Credit note".into(),
        other => title(other),
    }
}

type Entry = (String, Decimal, String, Option<Proposed>);

struct Builder<'a> {
    l: &'a mut Ledger,
    /// The document's branch mark (a proposed journal or void), given to every unmarked entry.
    doc_mark: Option<Proposed>,
    /// A void proposed by a manifest: the postings go to `ghosts`.
    ghost: bool,
}

/// The manifest mark a document or line carries (`_xa_manifest`, set only by `View`).
fn mark_of(chart: &Chart, v: &Value) -> Option<Proposed> {
    let m = v.get("_xa_manifest")?;
    let kind = match m.get("kind").and_then(Value::as_str)? {
        "create" => "create",
        "change" => "change",
        "void" => "void",
        _ => return None,
    };
    Some(Proposed {
        seq: m.get("seq").and_then(Value::as_i64).unwrap_or(0),
        kind,
        agent: m
            .get("actor")
            .and_then(Value::as_str)
            .is_some_and(|a| a.starts_with("agent:")),
        was: m
            .get("was_account")
            .and_then(Value::as_str)
            .map(|c| chart.resolve(Some(c), None)),
    })
}

/// A document's status for the ledger: a manifest void keeps its original status here (its postings become
/// ghosts), everything else reads `Status`.
fn status_of(p: &Value) -> Option<&str> {
    match p.get("_xa_manifest") {
        Some(m) if m.get("kind").and_then(Value::as_str) == Some("void") => {
            m.get("status").and_then(Value::as_str)
        }
        _ => opt_str(p, "Status"),
    }
}

impl<'a> Builder<'a> {
    fn for_doc(l: &'a mut Ledger, p: &Value) -> Builder<'a> {
        let doc_mark = mark_of(&l.chart, p);
        let ghost = doc_mark.as_ref().is_some_and(|m| m.kind == "void");
        Builder { l, doc_mark, ghost }
    }
}

impl Builder<'_> {
    #[allow(clippy::too_many_arguments)]
    fn emit(
        &mut self,
        source: &'static str,
        sid: &str,
        d: &str,
        entries: Vec<Entry>,
        contact: &str,
        reference: &str,
        kind: &str,
    ) {
        let total: Decimal = entries.iter().map(|e| e.1).sum();
        if total != ZERO && !self.ghost {
            self.l.unbalanced.push(Unbalanced {
                source,
                id: sid.into(),
                date: d.into(),
                imbalance: total,
            });
        }
        for (i, (acct, amt, desc, mark)) in entries.into_iter().enumerate() {
            if amt != ZERO {
                let target = if self.ghost {
                    &mut self.l.ghosts
                } else {
                    &mut self.l.postings
                };
                target.push(Posting {
                    date: d.into(),
                    source,
                    source_id: sid.into(),
                    line: i,
                    account_id: acct,
                    amount: amt,
                    description: desc,
                    contact: contact.into(),
                    reference: reference.into(),
                    kind: kind.into(),
                    proposed: mark.or_else(|| self.doc_mark.clone()),
                });
            }
        }
    }
}

fn opt_str<'v>(p: &'v Value, k: &str) -> Option<&'v str> {
    p.get(k).and_then(Value::as_str)
}

fn nested<'v>(p: &'v Value, k: &str, inner: &str) -> Option<&'v str> {
    p.get(k).and_then(|o| o.get(inner)).and_then(Value::as_str)
}

/// The documents a ledger is built from (current versions, in the record's order).
#[derive(Debug, Clone, Default)]
pub struct Documents {
    pub accounts: Vec<Value>,
    pub bank_transactions: Vec<Value>,
    pub bank_transfers: Vec<Value>,
    pub manual_journals: Vec<Value>,
    pub invoices: Vec<Value>,
    pub credit_notes: Vec<Value>,
    pub payments: Vec<Value>,
}

impl Ledger {
    /// Build the ledger for one tenant from the record.
    pub fn build(view: &crate::view::View<'_>, tenant_id: &str) -> Result<Ledger, LedgerError> {
        Ledger::build_basis(view, tenant_id, LedgerBasis::Accrual)
    }

    /// Build the ledger for one tenant on a basis (accrual or cash).
    pub fn build_basis(
        view: &crate::view::View<'_>,
        tenant_id: &str,
        basis: LedgerBasis,
    ) -> Result<Ledger, LedgerError> {
        let docs = |e: &str| view.payloads(tenant_id, e);
        let mut l = Ledger::from_documents_basis(
            &Documents {
                accounts: docs("Accounts")?,
                bank_transactions: docs("BankTransactions")?,
                bank_transfers: docs("BankTransfers")?,
                manual_journals: docs("ManualJournals")?,
                invoices: docs("Invoices")?,
                credit_notes: docs("CreditNotes")?,
                payments: docs("Payments")?,
            },
            basis,
        )?;
        if view.overlay().is_some() {
            // main's version of every document the manifest changes (a void's postings are the ghosts)
            let main = crate::view::View::main(view.record());
            let changed = |e: &str| -> Result<Vec<Value>, LedgerError> {
                let (rep, _) = view.patched(tenant_id, e);
                let ids: Vec<String> = rep
                    .iter()
                    .filter(|(_, d)| {
                        d.get("_xa_manifest")
                            .and_then(|m| m.get("kind"))
                            .and_then(Value::as_str)
                            != Some("void")
                    })
                    .map(|(id, _)| id.clone())
                    .collect();
                if ids.is_empty() {
                    return Ok(vec![]);
                }
                Ok(main
                    .payloads(tenant_id, e)?
                    .into_iter()
                    .filter(|d| ids.contains(&crate::view::doc_id(e, d)))
                    .collect())
            };
            let before = Ledger::from_documents_basis(
                &Documents {
                    accounts: docs("Accounts")?,
                    bank_transactions: changed("BankTransactions")?,
                    bank_transfers: changed("BankTransfers")?,
                    manual_journals: changed("ManualJournals")?,
                    invoices: changed("Invoices")?,
                    credit_notes: changed("CreditNotes")?,
                    payments: changed("Payments")?,
                },
                basis,
            )?;
            l.replaced = before.postings;
        }
        Ok(l)
    }

    /// Build a ledger from documents already in memory (same rules as `build`).
    pub fn from_documents(d: &Documents) -> Result<Ledger, LedgerError> {
        Ledger::from_documents_basis(d, LedgerBasis::Accrual)
    }

    /// Build a ledger from documents already in memory on a basis.
    pub fn from_documents_basis(d: &Documents, basis: LedgerBasis) -> Result<Ledger, LedgerError> {
        let mut l = Ledger {
            chart: Chart::new(&d.accounts),
            basis,
            ..Default::default()
        };
        l.add_bank_transactions(&d.bank_transactions)?;
        l.add_transfers(&d.bank_transfers)?;
        l.add_journals(&d.manual_journals)?;
        l.add_invoices(&d.invoices, "Invoices")?;
        l.add_invoices(&d.credit_notes, "CreditNotes")?;
        l.add_payments(&d.payments)?;
        l.postings.sort_by(|a, b| {
            (&a.date, a.source, &a.source_id, a.line).cmp(&(
                &b.date,
                b.source,
                &b.source_id,
                b.line,
            ))
        });
        Ok(l)
    }

    fn m(source: &'static str, id: &str, v: Option<&Value>) -> Result<Decimal, LedgerError> {
        money(v).map_err(|error| LedgerError::Money {
            source,
            id: id.into(),
            error,
        })
    }

    /// Line accounts (+ tax to the sales-tax system account) with sign +1 debit / −1 credit.
    fn lines(
        &self,
        source: &'static str,
        id: &str,
        doc: &Value,
        sign: i64,
    ) -> Result<Vec<Entry>, LedgerError> {
        let mut out = Vec::new();
        let lat = opt_str(doc, "LineAmountTypes");
        let sign = Decimal::from(sign);
        for li in doc
            .get("LineItems")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            let (net, tax) = Self::net(source, id, li, lat)?;
            let acct = self
                .chart
                .resolve(opt_str(li, "AccountCode"), opt_str(li, "AccountID"));
            out.push((
                acct,
                sign * net,
                opt_str(li, "Description").unwrap_or("").into(),
                mark_of(&self.chart, li),
            ));
            if tax != ZERO {
                out.push((self.chart.system("GST"), sign * tax, "tax".into(), None));
            }
        }
        Ok(out)
    }

    fn net(
        source: &'static str,
        id: &str,
        line: &Value,
        amount_types: Option<&str>,
    ) -> Result<(Decimal, Decimal), LedgerError> {
        let amount = Self::m(source, id, line.get("LineAmount"))?;
        let tax = Self::m(source, id, line.get("TaxAmount"))?;
        Ok((
            if amount_types == Some("Inclusive") {
                amount - tax
            } else {
                amount
            },
            tax,
        ))
    }

    fn skip(&mut self, key: String) {
        *self.skipped.entry(key).or_default() += 1;
    }

    fn add_bank_transactions(&mut self, docs: &[Value]) -> Result<(), LedgerError> {
        for p in docs {
            let t = opt_str(p, "Type").unwrap_or("");
            let status = status_of(p);
            if status != Some("AUTHORISED") || t.ends_with("-TRANSFER") {
                self.skip(format!("BankTransactions:{t}:{}", status.unwrap_or("None")));
                continue;
            }
            let id = opt_str(p, "BankTransactionID").unwrap_or("");
            let sign: i64 = if matches!(t, "SPEND" | "SPEND-OVERPAYMENT" | "SPEND-PREPAYMENT") {
                1
            } else {
                -1
            };
            let bank = self
                .chart
                .resolve(None, nested(p, "BankAccount", "AccountID"));
            let mut entries = self.lines("BankTransactions", id, p, sign)?;
            let total = Self::m("BankTransactions", id, p.get("Total"))?;
            entries.push((bank, Decimal::from(-sign) * total, "bank".into(), None));
            let contact = nested(p, "Contact", "Name").unwrap_or("").to_string();
            let reference = opt_str(p, "Reference").unwrap_or("").to_string();
            let d = doc_date(p);
            Builder::for_doc(self, p).emit(
                "BankTransactions",
                id,
                &d,
                entries,
                &contact,
                &reference,
                &kind_label(t),
            );
        }
        Ok(())
    }

    fn add_transfers(&mut self, docs: &[Value]) -> Result<(), LedgerError> {
        for p in docs {
            let status = status_of(p).unwrap_or("AUTHORISED");
            if status != "AUTHORISED" {
                self.skip(format!("BankTransfers:{status}"));
                continue;
            }
            let id = opt_str(p, "BankTransferID").unwrap_or("");
            let amt = Self::m("BankTransfers", id, p.get("Amount"))?;
            let frm = self
                .chart
                .resolve(None, nested(p, "FromBankAccount", "AccountID"));
            let to = self
                .chart
                .resolve(None, nested(p, "ToBankAccount", "AccountID"));
            let reference = opt_str(p, "Reference").unwrap_or("").to_string();
            let d = doc_date(p);
            Builder::for_doc(self, p).emit(
                "BankTransfers",
                id,
                &d,
                vec![
                    (to, amt, "transfer in".into(), None),
                    (frm, -amt, "transfer out".into(), None),
                ],
                "",
                &reference,
                "Transfer",
            );
        }
        Ok(())
    }

    fn add_journals(&mut self, docs: &[Value]) -> Result<(), LedgerError> {
        for p in docs {
            let status = status_of(p);
            if status != Some("POSTED") {
                self.skip(format!("ManualJournals:{}", status.unwrap_or("None")));
                continue;
            }
            if self.basis == LedgerBasis::Cash
                && p.get("ShowOnCashBasisReports").and_then(Value::as_bool) == Some(false)
            {
                self.skip("ManualJournals:not on cash basis".into());
                continue;
            }
            let id = opt_str(p, "ManualJournalID").unwrap_or("");
            let lat = Some(
                opt_str(p, "LineAmountTypes")
                    .filter(|s| !s.is_empty())
                    .unwrap_or("NoTax"),
            );
            let mut entries = Vec::new();
            for li in p
                .get("JournalLines")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
            {
                let (net, tax) = Self::net("ManualJournals", id, li, lat)?;
                entries.push((
                    self.chart
                        .resolve(opt_str(li, "AccountCode"), opt_str(li, "AccountID")),
                    net,
                    opt_str(li, "Description").unwrap_or("").into(),
                    mark_of(&self.chart, li),
                ));
                if tax != ZERO {
                    entries.push((self.chart.system("GST"), tax, "tax".into(), None));
                }
            }
            let narration = opt_str(p, "Narration").unwrap_or("").to_string();
            let d = doc_date(p);
            Builder::for_doc(self, p).emit(
                "ManualJournals",
                id,
                &d,
                entries,
                "",
                &narration,
                "Journal",
            );
        }
        Ok(())
    }

    fn add_invoices(&mut self, docs: &[Value], source: &'static str) -> Result<(), LedgerError> {
        let credit = source == "CreditNotes";
        for p in docs {
            let status = status_of(p);
            if !matches!(status, Some("AUTHORISED") | Some("PAID")) {
                self.skip(format!("{source}:{}", status.unwrap_or("None")));
                continue;
            }
            let t = opt_str(p, "Type").unwrap_or("");
            let id = opt_str(p, if credit { "CreditNoteID" } else { "InvoiceID" }).unwrap_or("");
            let sales = if credit {
                t == "ACCRECCREDIT"
            } else {
                t == "ACCREC"
            };
            let control = self
                .chart
                .system(if sales { "DEBTORS" } else { "CREDITORS" });
            let sign: i64 = match (credit, sales) {
                (false, true) | (true, false) => -1,
                _ => 1,
            };
            let mut entries = self.lines(source, id, p, sign)?;
            if self.basis == LedgerBasis::Cash {
                let doc = CashDoc {
                    lines: entries,
                    number: opt_str(
                        p,
                        if credit {
                            "CreditNoteNumber"
                        } else {
                            "InvoiceNumber"
                        },
                    )
                    .filter(|s| !s.is_empty())
                    .or_else(|| opt_str(p, "Reference"))
                    .unwrap_or("")
                    .to_string(),
                    contact: nested(p, "Contact", "Name").unwrap_or("").to_string(),
                };
                self.cash_docs.insert(id.to_string(), doc);
                continue;
            }
            let total = Self::m(source, id, p.get("Total"))?;
            entries.push((
                control,
                Decimal::from(-sign) * total,
                "control".into(),
                None,
            ));
            let number = opt_str(
                p,
                if credit {
                    "CreditNoteNumber"
                } else {
                    "InvoiceNumber"
                },
            )
            .filter(|s| !s.is_empty())
            .or_else(|| opt_str(p, "Reference"))
            .unwrap_or("")
            .to_string();
            let contact = nested(p, "Contact", "Name").unwrap_or("").to_string();
            let kind = if credit {
                "Credit note".to_string()
            } else if t == "ACCREC" || t == "ACCPAY" {
                kind_label(t)
            } else {
                "Invoice".into()
            };
            let d = doc_date(p);
            Builder::for_doc(self, p).emit(source, id, &d, entries, &contact, &number, &kind);
        }
        Ok(())
    }

    fn add_payments(&mut self, docs: &[Value]) -> Result<(), LedgerError> {
        for p in docs {
            let status = status_of(p);
            if status != Some("AUTHORISED") {
                self.skip(format!("Payments:{}", status.unwrap_or("None")));
                continue;
            }
            let id = opt_str(p, "PaymentID").unwrap_or("");
            let ptype = opt_str(p, "PaymentType").unwrap_or("");
            let receivable = ptype.starts_with("AR") || ptype == "ACCRECPAYMENT";
            let control = self
                .chart
                .system(if receivable { "DEBTORS" } else { "CREDITORS" });
            let money_in = matches!(
                ptype,
                "ACCRECPAYMENT"
                    | "APCREDITPAYMENT"
                    | "APOVERPAYMENTPAYMENT"
                    | "APPREPAYMENTPAYMENT"
            );
            let amt = Self::m("Payments", id, p.get("Amount"))?;
            let bank = self.chart.resolve(
                nested(p, "Account", "Code"),
                nested(p, "Account", "AccountID"),
            );
            let s = if money_in {
                Decimal::ONE
            } else {
                -Decimal::ONE
            };
            let reference = opt_str(p, "Reference").unwrap_or("").to_string();
            let d = doc_date(p);
            let paid = nested(p, "Invoice", "InvoiceID")
                .or_else(|| nested(p, "CreditNote", "CreditNoteID"))
                .and_then(|i| self.cash_docs.get(i))
                .cloned();
            if let Some(doc) = paid {
                // cash basis: the document's lines, in proportion to this payment, against the bank
                let mut entries = vec![(bank, s * amt, "payment".to_string(), None)];
                entries.extend(apportion(&doc.lines, -s * amt));
                Builder::for_doc(self, p).emit(
                    "Payments",
                    id,
                    &d,
                    entries,
                    &doc.contact,
                    &doc.number,
                    "Payment",
                );
                continue;
            }
            Builder::for_doc(self, p).emit(
                "Payments",
                id,
                &d,
                vec![
                    (bank, s * amt, "payment".into(), None),
                    (control, -s * amt, ptype.into(), None),
                ],
                "",
                &reference,
                "Payment",
            );
        }
        Ok(())
    }
}

/// `lines` scaled so they sum to `target` (each rounded to the cent, half away from zero; the rounding
/// remainder goes to the largest line).
fn apportion(lines: &[Entry], target: Decimal) -> Vec<Entry> {
    let whole: Decimal = lines.iter().map(|e| e.1).sum();
    if whole == ZERO {
        return Vec::new();
    }
    let mut out: Vec<Entry> = lines
        .iter()
        .map(|(a, amt, desc, mark)| {
            let v = (*amt * target / whole)
                .round_dp_with_strategy(2, rust_decimal::RoundingStrategy::MidpointAwayFromZero);
            (a.clone(), v, desc.clone(), mark.clone())
        })
        .collect();
    let rest = target - out.iter().map(|e| e.1).sum::<Decimal>();
    if rest != ZERO
        && let Some(big) = out.iter_mut().max_by_key(|e| e.1.abs())
    {
        big.1 += rest;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn apportion_keeps_the_total() {
        let l = |a: &str, v: i64| (a.to_string(), Decimal::new(v, 2), String::new(), None);
        let lines = vec![l("a", -10000), l("b", -5000), l("tax", -1)];
        let got = apportion(&lines, Decimal::new(-10000, 2));
        assert_eq!(
            got.iter().map(|e| e.1).sum::<Decimal>(),
            Decimal::new(-10000, 2)
        );
        assert_eq!(got[0].1, Decimal::new(-6666, 2));
        assert_eq!(got[1].1, Decimal::new(-3333, 2));
    }

    #[test]
    fn titles_type_codes() {
        assert_eq!(title("SPEND-TRANSFER"), "Spend-Transfer");
        assert_eq!(kind_label("RECEIVE-PREPAYMENT"), "Prepayment");
        assert_eq!(kind_label("ACCPAY"), "Bill");
    }

    #[test]
    fn chart_resolves_ids_and_placeholders() {
        let c = Chart::new(&[
            serde_json::json!({"AccountID": "a1", "Code": "2410", "Name": "Loans", "Type": "CURRLIAB",
                                                  "Class": "LIABILITY", "SystemAccount": "", "Status": "ACTIVE"}),
            serde_json::json!({"AccountID": "a2", "Code": "", "Name": "GST", "SystemAccount": "GST"}),
        ]);
        assert_eq!(c.resolve(Some("2410"), None), "a1");
        assert_eq!(c.resolve(None, Some("a1")), "a1");
        assert_eq!(c.resolve(Some("999"), None), "(uncoded):999");
        assert_eq!(c.resolve(None, None), "(uncoded):");
        assert_eq!(c.system("GST"), "a2");
        assert_eq!(c.system("DEBTORS"), "(uncoded):system:DEBTORS");
        assert_eq!(c.get("zz").name, "zz");
        assert_eq!(c.get("a1").label(), "Loans (2410)");
    }
}
