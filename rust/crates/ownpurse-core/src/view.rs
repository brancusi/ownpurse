//! The ONE entry point for reading the books: main (the record) or main + a manifest.
//!
//! `View::open(record, ctx, manifest)` applies the manifest's active entries, in order, to the record's
//! documents (a document-level overlay; the record is never written). Each entry is a JSON Patch against
//! the NORMALISED document as it was when recorded (`before_fp`):
//! - main still equals that "before": the entry applies, and the view holds "after" (denormalised);
//! - main already equals the entry's "after" (the write landed and was read back): the entry is POSTED and
//!   the overlay does nothing for it;
//! - main moved anywhere else: a CONFLICT, listed with old against new, never applied.
//!
//! Overlaid documents carry `_xa_manifest {seq, kind, actor, …}` (and changed lines their own mark), which
//! the ledger builder turns into `Posting.proposed`; a void's postings become the ledger's ghosts.
//!
//! Everything else on `Record` (reports, the Xero TB snapshots, sync runs) reads main: parity always
//! compares main with Xero. Documents are only read through `View::payloads` (a test greps for any other
//! caller).

use crate::manifest::normalise::{denormalise, fingerprint, normalise};
use crate::manifest::rules::{Matched, Shape, classify};
use crate::manifest::{Entry, Manifest, Result};
use crate::record::Record;
use serde_json::{Value, json};
use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;

/// The orgs a view resolves (`alias → tenant`) and the accountant gate (`[(alias, through)]`).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Ctx {
    pub orgs: Vec<(String, String)>,
    pub gate: Vec<(String, String)>,
}

impl Ctx {
    pub fn tenant(&self, alias: &str) -> Option<&str> {
        self.orgs
            .iter()
            .find(|(a, _)| a == alias)
            .map(|(_, t)| t.as_str())
    }
    pub fn alias(&self, tid: &str) -> Option<&str> {
        self.orgs
            .iter()
            .find(|(_, t)| t == tid)
            .map(|(a, _)| a.as_str())
    }
}

/// One reason an entry no longer applies: what moved, as it was when recorded and as it is now.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reason {
    pub what: String,
    pub old: String,
    pub new: String,
}

/// An entry that conflicts with the current main (not applied).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Conflict {
    pub seq: i64,
    pub reasons: Vec<Reason>,
}

/// An applied entry: what it touched, its before and after (normalised), and its marks.
#[derive(Debug, Clone, PartialEq)]
pub struct Applied {
    pub seq: i64,
    pub org: String,
    pub tenant: String,
    /// `create`, `change` or `void` (the mark kind).
    pub kind: &'static str,
    /// What it compiles to (a rule's op, or not writable).
    pub matched: Matched,
    /// `(entity, xero id)`; a new document is `new:<seq>`.
    pub doc: (String, String),
    /// The dated effect (the document's date after the change).
    pub date: String,
    /// Normalised before (None: a new document) and after.
    pub before: Option<Value>,
    pub after: Value,
    /// AZ-2025 / prior-period and without an accountant decision: blocks planning.
    pub needs_accountant: bool,
    /// Gated (whether or not referenced).
    pub gated: bool,
    /// Touches a reconciled bank transaction.
    pub reconciled_line: bool,
    pub accountant: Option<String>,
}

#[derive(Debug, Default)]
struct Patch {
    replace: BTreeMap<String, Value>,
    add: Vec<Value>,
}

/// The manifest side of a view.
#[derive(Debug, Default)]
pub struct Overlay {
    pub name: String,
    pub head_rev: i64,
    pub base: i64,
    pub applied: Vec<Applied>,
    pub conflicts: Vec<Conflict>,
    /// Entries whose "after" main already holds (landed and read back): no-ops.
    pub posted: Vec<i64>,
    patches: HashMap<(String, String), Patch>,
}

impl Overlay {
    pub fn needs_accountant(&self) -> usize {
        self.applied.iter().filter(|a| a.needs_accountant).count()
    }
    pub fn conflict(&self, seq: i64) -> Option<&Conflict> {
        self.conflicts.iter().find(|c| c.seq == seq)
    }
    pub fn applied(&self, seq: i64) -> Option<&Applied> {
        self.applied.iter().find(|a| a.seq == seq)
    }
    pub fn is_posted(&self, seq: i64) -> bool {
        self.posted.contains(&seq)
    }
}

/// Main, or main + one manifest.
#[derive(Clone)]
pub struct View<'r> {
    record: &'r Record,
    overlay: Option<Arc<Overlay>>,
}

impl std::ops::Deref for View<'_> {
    type Target = Record;
    fn deref(&self) -> &Record {
        self.record
    }
}

/// The id field of an entity's documents.
pub fn id_field(entity: &str) -> &'static str {
    match entity {
        "BankTransactions" => "BankTransactionID",
        "ManualJournals" => "ManualJournalID",
        "Invoices" => "InvoiceID",
        "CreditNotes" => "CreditNoteID",
        "BankTransfers" => "BankTransferID",
        "Payments" => "PaymentID",
        "Accounts" => "AccountID",
        "Contacts" => "ContactID",
        _ => "",
    }
}

/// The id of a document payload in its entity.
pub fn doc_id(entity: &str, p: &Value) -> String {
    p.get(id_field(entity))
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string()
}

/// The entity a document type code lives in.
pub fn entity_of(doc_type: &str) -> Option<&'static str> {
    Some(match doc_type {
        "bt" => "BankTransactions",
        "mj" => "ManualJournals",
        "inv" | "bill" => "Invoices",
        _ => return None,
    })
}

/// The lines array key of a document.
pub fn lines_key(p: &Value) -> &'static str {
    if p.get("JournalLines").is_some() {
        "JournalLines"
    } else {
        "LineItems"
    }
}

/// The marker a compiled create carries: `XA:<package id>:e<seq>`.
pub fn create_marker(package_id: &str, seq: i64) -> String {
    crate::write::marker(package_id, &format!("e{seq}"))
}

/// The line keys an entry's patch touches, and whether it touches anything outside the lines.
fn touched(e: &Entry) -> (Vec<String>, bool) {
    let mut lines = vec![];
    let mut header = false;
    for p in e.paths() {
        let s: Vec<&str> = p.trim_start_matches('/').split('/').collect();
        if s.len() >= 2 && matches!(s[0], "JournalLines" | "LineItems") {
            if !lines.contains(&s[1].to_string()) {
                lines.push(s[1].to_string());
            }
        } else if s.first() != Some(&"Attachments") {
            header = true;
        }
    }
    (lines, header)
}

fn sorted_lines(d: &Value) -> Vec<(String, String)> {
    let mut v: Vec<(String, String)> = d
        .get("JournalLines")
        .or_else(|| d.get("LineItems"))
        .and_then(Value::as_object)
        .into_iter()
        .flatten()
        .map(|(_, l)| {
            (
                l["AccountCode"].as_str().unwrap_or("").to_string(),
                l["LineAmount"].as_str().unwrap_or("").to_string(),
            )
        })
        .collect();
    v.sort();
    v
}

impl<'r> View<'r> {
    /// Main only.
    pub fn main(record: &'r Record) -> View<'r> {
        View {
            record,
            overlay: None,
        }
    }

    /// Main + a manifest (None = main). Conflicting entries are listed, never applied.
    pub fn open(record: &'r Record, ctx: &Ctx, manifest: Option<&Manifest>) -> Result<View<'r>> {
        let Some(m) = manifest else {
            return Ok(View::main(record));
        };
        let mut v = View {
            record,
            overlay: Some(Arc::new(Overlay {
                name: m.name.clone(),
                head_rev: m.head_rev,
                base: m.base,
                ..Default::default()
            })),
        };
        let package_id = m
            .planned
            .as_ref()
            .map(|p| crate::manifest::compile::package_id_of(&m.name, p.from_rev));
        for e in m.active() {
            v.entry(ctx, m, e, package_id.as_deref())?;
        }
        Ok(v)
    }

    fn overlay_mut(&mut self) -> &mut Overlay {
        Arc::get_mut(self.overlay.as_mut().expect("a manifest view"))
            .expect("unshared while opening")
    }

    fn conflict(&mut self, seq: i64, what: &str, old: &str, new: &str) {
        self.overlay_mut().conflicts.push(Conflict {
            seq,
            reasons: vec![Reason {
                what: what.into(),
                old: old.into(),
                new: new.into(),
            }],
        });
    }

    /// A created journal already in main (its marker), with the entry's lines: posted.
    fn created_in_main(
        &self,
        tid: &str,
        e: &Entry,
        after: &Value,
        package_id: Option<&str>,
    ) -> Result<bool> {
        let Some(pkg) = package_id else {
            return Ok(false);
        };
        // a transfer carries the marker in its Reference; a contact is its exact name
        if e.entity == "BankTransfers" {
            let marker = create_marker(pkg, e.seq);
            return Ok(self
                .record
                .main_payloads(tid, "BankTransfers")?
                .iter()
                .any(|d| {
                    d.get("Reference")
                        .and_then(Value::as_str)
                        .is_some_and(|r| r.contains(&marker))
                }));
        }
        if e.entity == "Contacts" {
            let name = after.get("Name").and_then(Value::as_str).unwrap_or("");
            return Ok(!name.is_empty()
                && self
                    .record
                    .main_payloads(tid, "Contacts")?
                    .iter()
                    .any(|d| d.get("Name").and_then(Value::as_str) == Some(name)));
        }
        // a journal carries the marker in its Narration, a bank transaction in its Reference
        let (field, live) = match e.entity.as_str() {
            "ManualJournals" => ("Narration", "POSTED"),
            "BankTransactions" => ("Reference", "AUTHORISED"),
            _ => return Ok(false),
        };
        let marker = create_marker(pkg, e.seq);
        for d in self.record.main_payloads(tid, &e.entity)? {
            let text = d.get(field).and_then(Value::as_str).unwrap_or("");
            if text.contains(&marker) && d.get("Status").and_then(Value::as_str) == Some(live) {
                return Ok(sorted_lines(&normalise(&d)) == sorted_lines(after));
            }
        }
        Ok(false)
    }

    fn entry(
        &mut self,
        ctx: &Ctx,
        m: &Manifest,
        e: &Entry,
        package_id: Option<&str>,
    ) -> Result<()> {
        let Some(tid) = ctx.tenant(&e.org).map(str::to_string) else {
            self.conflict(e.seq, "org", &e.org, "not in this profile");
            return Ok(());
        };
        // a note-only entry landed when the document's history (as the record last read it) holds every note
        if let Some(notes) = note_details(&e.patch)
            && let Some((_, h)) =
                self.record
                    .main_document(&tid, "History", &format!("{}/{}", e.entity, e.id))?
            && notes.iter().all(|n| {
                h.as_array().into_iter().flatten().any(|r| {
                    r.get("Details")
                        .and_then(Value::as_str)
                        .map(crate::text::html_unescape)
                        .is_some_and(|d| d == crate::text::ascii_fold(n))
                })
            })
        {
            self.overlay_mut().posted.push(e.seq);
            return Ok(());
        }
        // the precondition: main still holds the "before" the entry was recorded against
        let mut main_raw = None;
        if let Some(fp_old) = &e.before_fp {
            main_raw = self
                .record
                .main_document(&tid, &e.entity, &e.id)?
                .map(|(_, d)| d);
            let main_now = main_raw.as_ref().map(normalise);
            let fp_now = main_now.as_ref().map(fingerprint).unwrap_or_default();
            if &fp_now != fp_old {
                // void + re-create landed: the original is voided and its replacement (the marker) is in main
                if e.resolution.as_deref() == Some("void-recreate")
                    && main_now
                        .as_ref()
                        .and_then(|n| n.get("Status"))
                        .and_then(Value::as_str)
                        == Some("VOIDED")
                    && let Some(pkg) = package_id
                {
                    let marker = create_marker(pkg, e.seq);
                    let landed = self
                        .record
                        .main_payloads(&tid, "ManualJournals")?
                        .iter()
                        .any(|d| {
                            d.get("Status").and_then(Value::as_str) == Some("POSTED")
                                && d.get("Narration")
                                    .and_then(Value::as_str)
                                    .is_some_and(|n| n.contains(&marker))
                        });
                    if landed {
                        self.overlay_mut().posted.push(e.seq);
                        return Ok(());
                    }
                }
                // landed and read back: main holds "after" (the overlay is a no-op)
                let bfp_now = main_raw.as_ref().map(crate::write::fingerprint);
                if main_now.is_some()
                    && (e.after_fp.as_deref() == Some(fp_now.as_str())
                        || (e.after_bfp.is_some() && e.after_bfp == bfp_now))
                {
                    self.overlay_mut().posted.push(e.seq);
                    return Ok(());
                }
                let new_v = match &main_now {
                    Some(n) => format!(
                        "changed since ({})",
                        n.get("Status")
                            .and_then(Value::as_str)
                            .unwrap_or("edited")
                            .to_lowercase()
                    ),
                    None => "gone from the record".into(),
                };
                self.conflict(e.seq, "document", "as recorded", &new_v);
                return Ok(());
            }
        }
        let base_norm = main_raw.as_ref().map(normalise);
        let after = match e.after(base_norm.as_ref()) {
            Ok(a) => a,
            Err(err) => {
                self.conflict(e.seq, "patch", "applies", &err.message);
                return Ok(());
            }
        };
        if e.is_new() && self.created_in_main(&tid, e, &after, package_id)? {
            self.overlay_mut().posted.push(e.seq);
            return Ok(());
        }
        let matched = classify(&Shape::of(e, base_norm.as_ref(), &after));
        // a reconciled line carried out by its org-month reclass journal: posted once that journal is in main
        if matches!(matched, Matched::Op("reclass_journal"))
            && let Some(pkg) = package_id
        {
            let date = base_norm
                .as_ref()
                .and_then(|b| b.get("Date"))
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string();
            let marker =
                crate::write::marker(pkg, &crate::manifest::compile::reclass_op_id(&e.org, &date));
            let landed = self
                .record
                .main_payloads(&tid, "ManualJournals")?
                .iter()
                .any(|d| {
                    d.get("Status").and_then(Value::as_str) == Some("POSTED")
                        && d.get("Narration")
                            .and_then(Value::as_str)
                            .is_some_and(|n| n.contains(&marker))
                });
            if landed {
                self.overlay_mut().posted.push(e.seq);
                return Ok(());
            }
        }
        let was_status = base_norm
            .as_ref()
            .and_then(|b| b.get("Status"))
            .and_then(Value::as_str);
        let void = after.get("Status").and_then(Value::as_str) == Some("VOIDED")
            && was_status.is_some_and(|s| s != "VOIDED");
        let kind: &'static str = if e.is_new() {
            "create"
        } else if void {
            "void"
        } else {
            "change"
        };
        let mut payload = denormalise(&after, base_norm.as_ref());
        let field = id_field(&e.entity);
        if e.is_new() && !field.is_empty() {
            payload[field] = json!(e.id);
            if e.entity == "ManualJournals" {
                if payload.get("Status").is_none() {
                    payload["Status"] = json!("POSTED");
                }
                if payload.get("LineAmountTypes").is_none() {
                    payload["LineAmountTypes"] = json!("NoTax");
                }
            }
        }
        let mark = |extra: Value| {
            let mut mk = json!({"manifest": m.name, "seq": e.seq, "kind": kind, "actor": e.actor});
            if let (Some(mm), Some(x)) = (mk.as_object_mut(), extra.as_object()) {
                for (k, v) in x {
                    mm.insert(k.clone(), v.clone());
                }
            }
            mk
        };
        let (lines, header) = touched(e);
        match kind {
            "create" => payload["_xa_manifest"] = mark(json!({})),
            "void" => {
                payload["_xa_manifest"] = mark(json!({"status": was_status}));
            }
            _ if header => payload["_xa_manifest"] = mark(json!({})),
            _ => {
                // mark the changed lines, with what each had on main
                let key = lines_key(&payload);
                let before_lines = base_norm.as_ref().and_then(|b| b.get(key)).cloned();
                if let Some(arr) = payload.get_mut(key).and_then(Value::as_array_mut) {
                    for (i, li) in arr.iter_mut().enumerate() {
                        let k = crate::manifest::normalise::line_key(li, i);
                        if !lines.contains(&k) {
                            continue;
                        }
                        let old = before_lines.as_ref().and_then(|b| b.get(&k));
                        let was = old
                            .and_then(|l| l.get("AccountCode"))
                            .cloned()
                            .unwrap_or(Value::Null);
                        let was_desc = old
                            .and_then(|l| l.get("Description"))
                            .cloned()
                            .unwrap_or(Value::Null);
                        if let Some(o) = li.as_object_mut() {
                            o.insert(
                                "_xa_manifest".into(),
                                mark(json!({"was_account": was, "was_description": was_desc})),
                            );
                        }
                    }
                }
            }
        }
        let date = crate::dates::doc_date(&payload);
        let old_date = main_raw.as_ref().map(crate::dates::doc_date);
        // a note moves no number: never the accountant's gate
        let gated = note_details(&e.patch).is_none()
            && (self.gated(ctx, &e.org, &tid, &date)?
                || match old_date {
                    Some(d) if d != date => self.gated(ctx, &e.org, &tid, &d)?,
                    _ => false,
                });
        let reconciled = main_raw
            .as_ref()
            .and_then(|b| b.get("IsReconciled"))
            .and_then(Value::as_bool)
            .unwrap_or(false);
        let accountant = e.refs.accountant_decision.clone();
        let ov = self.overlay_mut();
        let p = ov
            .patches
            .entry((tid.clone(), e.entity.clone()))
            .or_default();
        if e.is_new() {
            p.add.push(payload);
        } else {
            p.replace.insert(e.id.clone(), payload);
        }
        ov.applied.push(Applied {
            seq: e.seq,
            org: e.org.clone(),
            tenant: tid,
            kind,
            matched,
            doc: (e.entity.clone(), e.id.clone()),
            date,
            before: base_norm,
            after,
            needs_accountant: gated && accountant.is_none(),
            gated,
            reconciled_line: reconciled,
            accountant,
        });
        Ok(())
    }

    pub fn record(&self) -> &'r Record {
        self.record
    }

    /// A view from an already-opened overlay (cheap; the TUI keeps the overlay for a manifest revision).
    pub fn from_parts(record: &'r Record, overlay: Option<Arc<Overlay>>) -> View<'r> {
        View { record, overlay }
    }

    /// The overlay, shareable (None on main).
    pub fn overlay_arc(&self) -> Option<Arc<Overlay>> {
        self.overlay.clone()
    }

    /// The manifest side, or None on main.
    pub fn overlay(&self) -> Option<&Overlay> {
        self.overlay.as_deref()
    }

    pub fn manifest_name(&self) -> Option<&str> {
        self.overlay.as_deref().map(|o| o.name.as_str())
    }

    /// The current documents of an entity: main's, with the manifest's replacements and additions.
    pub fn payloads(&self, tenant_id: &str, entity: &str) -> crate::record::Result<Vec<Value>> {
        let main = self.record.main_payloads(tenant_id, entity)?;
        let Some(p) = self
            .overlay
            .as_deref()
            .and_then(|o| o.patches.get(&(tenant_id.to_string(), entity.to_string())))
        else {
            return Ok(main);
        };
        let mut out: Vec<Value> = main
            .into_iter()
            .map(|d| match p.replace.get(&doc_id(entity, &d)) {
                Some(r) => r.clone(),
                None => d,
            })
            .collect();
        out.extend(p.add.iter().cloned());
        Ok(out)
    }

    /// The documents this view changes for a tenant and entity: (replaced by id, added).
    pub fn patched(&self, tenant_id: &str, entity: &str) -> (Vec<(String, Value)>, Vec<Value>) {
        match self
            .overlay
            .as_deref()
            .and_then(|o| o.patches.get(&(tenant_id.to_string(), entity.to_string())))
        {
            Some(p) => (
                p.replace
                    .iter()
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect(),
                p.add.clone(),
            ),
            None => (vec![], vec![]),
        }
    }

    /// One current document as this view sees it: `(version on main, payload)`; a new document has
    /// version 0.
    pub fn document(
        &self,
        tenant_id: &str,
        entity: &str,
        id: &str,
    ) -> crate::record::Result<Option<(i64, Value)>> {
        if let Some(p) = self
            .overlay
            .as_deref()
            .and_then(|o| o.patches.get(&(tenant_id.to_string(), entity.to_string())))
        {
            if let Some(d) = p.add.iter().find(|d| doc_id(entity, d) == id) {
                return Ok(Some((0, d.clone())));
            }
            if let Some(d) = p.replace.get(id) {
                let v = self
                    .record
                    .main_document(tenant_id, entity, id)?
                    .map_or(0, |(v, _)| v);
                return Ok(Some((v, d.clone())));
            }
        }
        self.record.main_document(tenant_id, entity, id)
    }

    /// Main's document (never the overlay): what an entry's "before" is.
    pub fn main_document(
        &self,
        tenant_id: &str,
        entity: &str,
        id: &str,
    ) -> crate::record::Result<Option<(i64, Value)>> {
        self.record.main_document(tenant_id, entity, id)
    }

    /// The accountant gate: the org is gated through a date, or the change is dated before the fiscal-year
    /// start of the latest year the books are worked on (the latest cached Xero TB): a prior period.
    pub fn gated(&self, ctx: &Ctx, org: &str, tid: &str, date: &str) -> Result<bool> {
        if ctx
            .gate
            .iter()
            .any(|(o, through)| o == org && date <= through.as_str())
        {
            return Ok(true);
        }
        let Some(last) = self.record.latest_tb_date(tid)? else {
            return Ok(false);
        };
        let (m, d) = self.record.fy_end(tid)?;
        Ok(
            crate::dates::fiscal_year_start(last.get(..10).unwrap_or(""), m, d)
                .is_some_and(|start| date < start.as_str()),
        )
    }
}

/// The notes a note-only patch adds (`add /History/-` `{"Details": …}` or `add /History` `[…]`); None for any
/// other patch.
fn note_details(patch: &Value) -> Option<Vec<String>> {
    let mut out = vec![];
    for op in patch.as_array()? {
        let path = op.get("path").and_then(Value::as_str)?;
        if op.get("op").and_then(Value::as_str) != Some("add") {
            return None;
        }
        let v = op.get("value")?;
        let one = |v: &Value| v.get("Details").and_then(Value::as_str).map(str::to_string);
        if path == "/History" {
            for n in v.as_array()? {
                out.push(one(n)?);
            }
        } else if path.starts_with("/History/") && path.matches('/').count() == 2 {
            out.push(one(v)?);
        } else {
            return None;
        }
    }
    (!out.is_empty()).then_some(out)
}
