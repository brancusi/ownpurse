//! Manifest operations with their guards (create, add an entry, edit, undo, annotate, acknowledge, discard)
//! and the entry PRODUCERS: a journal, a recode, a void, an attachment, or any document "after". Shared by
//! the CLI, the TUI and the agent bridge, so every writer applies the same rules. Producers only make an
//! "after"; the entry is always `diff(normalise(main before), normalise(after))`. An entry on a document
//! that already has an active entry REPLACES it (one entry per document, combined), so two entries never
//! fight over one document.
//!
//! Business rules (balance, accounts, lock dates, the accountant gate…) are NOT enforced here: `check`
//! reports them as issues, and errors block `plan`. Nothing here calls Xero, approves or applies.

use super::normalise::normalise;
use super::{Kind, Manifest, ManifestError, Refs, Result, Store, entry_payload, name_ok};
use crate::record::Record;
use crate::view::{Ctx, View, id_field, lines_key};
use rust_decimal::Decimal;
use serde_json::{Value, json};

/// The usual write guards: `expect_rev` (the manifest head the writer last read; refused `view_changed` if
/// it moved), `expect_manifest` (the writer believes it is on this manifest), and `overwrite` (an agent
/// changing another actor's manifest).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Guard {
    pub expect_rev: Option<i64>,
    pub expect_manifest: Option<String>,
    pub overwrite: bool,
}

fn check(m: &Manifest, actor: &str, g: &Guard, owner_rule: bool) -> Result<()> {
    if let Some(e) = &g.expect_manifest
        && e != &m.name
    {
        return Err(ManifestError::new(
            "view_changed",
            format!(
                "Expected manifest `{e}`, but the change targets `{}`.",
                m.name
            ),
        ));
    }
    if let Some(r) = g.expect_rev
        && r != m.head_rev
    {
        return Err(ManifestError::new(
            "view_changed",
            format!(
                "Manifest `{}` moved since you read it (rev {r} → {}). Nothing was changed.",
                m.name, m.head_rev
            ),
        ));
    }
    if owner_rule && actor.starts_with("agent:") && actor != m.created_by && !g.overwrite {
        return Err(ManifestError::new(
            "forbidden",
            format!(
                "Manifest `{}` was created by {}: pass overwrite to change it as {actor}.",
                m.name, m.created_by
            ),
        ));
    }
    Ok(())
}

/// The record version a new manifest starts from: the highest version id over the orgs.
pub fn record_base(record: &Record, ctx: &Ctx) -> Result<i64> {
    let mut base = 0;
    for (_, tid) in &ctx.orgs {
        base = base.max(record.max_version_id(tid)?);
    }
    Ok(base)
}

/// `manifest new`.
pub fn create(
    store: &mut Store,
    record: &Record,
    ctx: &Ctx,
    actor: &str,
    name: &str,
    why: &str,
) -> Result<i64> {
    if !name_ok(name) {
        return Err(ManifestError::new(
            "usage",
            format!(
                "Manifest name `{name}`: use lowercase letters, digits and `-` (up to 40), not `main`."
            ),
        ));
    }
    if store.manifest(name).is_some() {
        return Err(ManifestError::new(
            "usage",
            format!("A manifest named `{name}` already exists (names are never reused)."),
        ));
    }
    let base = record_base(record, ctx)?;
    store.append(actor, name, Kind::Create, json!({"why": why, "base": base}))
}

/// What a producer changes.
#[derive(Debug, Clone, PartialEq)]
pub struct Target {
    pub entity: String,
    pub org: String,
    /// None: a new document.
    pub id: Option<String>,
}

/// The document as the manifest sees it now (main + its entries), normalised. None: no such document.
pub fn current(view: &View<'_>, ctx: &Ctx, t: &Target) -> Result<Option<Value>> {
    let Some(id) = &t.id else { return Ok(None) };
    let tid = tenant(ctx, &t.org)?;
    Ok(view
        .document(tid, &t.entity, id)?
        .map(|(_, d)| normalise(&d)))
}

fn tenant<'c>(ctx: &'c Ctx, org: &str) -> Result<&'c str> {
    ctx.tenant(org)
        .ok_or_else(|| ManifestError::new("usage", format!("Unknown org `{org}`.")))
}

/// Add (or, on a document that already has an active entry, replace) the entry that turns main's document
/// into `after` (raw or normalised; any entity). `replaces` edits a given entry explicitly.
#[allow(clippy::too_many_arguments)]
pub fn add(
    store: &mut Store,
    record: &Record,
    ctx: &Ctx,
    manifest: &str,
    actor: &str,
    t: &Target,
    after: &Value,
    why: &str,
    refs: &Refs,
    replaces: Option<i64>,
    g: &Guard,
) -> Result<i64> {
    let m = store.open_manifest(manifest)?;
    check(&m, actor, g, false)?;
    let tid = tenant(ctx, &t.org)?.to_string();
    let before = match &t.id {
        None => None,
        Some(id) if id.starts_with("new:") => None,
        Some(id) => Some(
            record
                .current_rows(&tid, &t.entity)?
                .into_iter()
                .find(|r| &r.xero_id == id)
                .map(|r| r.payload)
                .ok_or_else(|| {
                    ManifestError::new(
                        "not_found",
                        format!("No {} document {id} in {}'s record.", t.entity, t.org),
                    )
                })?,
        ),
    };
    // one entry per document: an active entry on it is replaced (combined), unless told otherwise
    let doc_id = t.id.clone().unwrap_or_default();
    let replaces = match replaces {
        Some(r) => {
            let old = m.entry(r).ok_or_else(|| {
                ManifestError::new("not_found", format!("No entry {r} on `{manifest}`."))
            })?;
            if old.replaced_by.is_some() {
                return Err(ManifestError::new(
                    "usage",
                    format!("Entry {r} was already edited; edit its newest version."),
                ));
            }
            Some(r)
        }
        None => m.on_doc(&t.org, &t.entity, &doc_id).map(|e| e.seq),
    };
    let mut refs = refs.clone();
    let mut why = why.to_string();
    if let Some(r) = replaces.and_then(|r| m.entry(r)) {
        // an edit keeps the references and reason unless new ones are given
        if refs.accountant_decision.is_none() {
            refs.accountant_decision = r.refs.accountant_decision.clone();
        }
        if refs.sources.is_empty() {
            refs.sources = r.refs.sources.clone();
        }
        if why.is_empty() {
            why = r.why.clone();
        }
    }
    let mut a = normalise(after);
    if let (Some(o), Some(id)) = (a.as_object_mut(), &t.id)
        && id.starts_with("new:")
    {
        o.remove(id_field(&t.entity));
    }
    let id = t.id.as_deref().filter(|i| !i.starts_with("new:"));
    let mut payload = entry_payload(
        &t.entity,
        &t.org,
        id,
        before.as_ref(),
        &a,
        &why,
        &refs,
        replaces,
    );
    // a reconciled bank line can't be edited through the API: the entry records how its change is carried
    // out (the edited entry's choice, else the manifest's default: a reclass journal)
    let reconciled = t.entity == "BankTransactions"
        && before
            .as_ref()
            .and_then(|b| b.get("IsReconciled"))
            .and_then(Value::as_bool)
            == Some(true);
    if reconciled {
        let r = replaces
            .and_then(|r| m.entry(r))
            .and_then(|e| e.resolution.clone())
            .unwrap_or_else(|| m.default_resolution.clone());
        payload["resolution"] = json!(r);
    }
    if payload["patch"].as_array().is_some_and(|p| p.is_empty()) {
        return Err(ManifestError::new(
            "usage",
            "That changes nothing on main (the document already looks like that).",
        ));
    }
    store.append(actor, manifest, Kind::EntryAdd, payload)
}

// ── producers ──────────────────────────────────────────────────────────────────────────────────

/// One line of a new journal: a signed amount (debit positive, credit negative).
#[derive(Debug, Clone, PartialEq)]
pub struct JLine {
    pub account: String,
    pub description: String,
    pub amount: Decimal,
}

/// A new manual journal (normalised): POSTED, NoTax, `ShowOnCashBasisReports` explicit, lines keyed `n<k>`.
pub fn journal_doc(date: &str, narration: &str, cash_basis: bool, lines: &[JLine]) -> Value {
    let mut ls = vec![];
    for l in lines {
        let mut o = serde_json::Map::new();
        o.insert("AccountCode".into(), json!(l.account));
        if !l.description.is_empty() {
            o.insert("Description".into(), json!(l.description));
        }
        o.insert("LineAmount".into(), json!(l.amount.to_string()));
        ls.push(Value::Object(o));
    }
    normalise(&json!({
        "Date": date,
        "Narration": narration,
        "Status": "POSTED",
        "LineAmountTypes": "NoTax",
        "ShowOnCashBasisReports": cash_basis,
        "JournalLines": ls,
    }))
}

/// The key of a line named by a user: a `LineItemID`, `n<k>`, or `L<k>` (the k-th line).
pub fn line_key(doc: &Value, line: &str) -> Option<String> {
    let lines = doc.get(lines_key_norm(doc))?.as_object()?;
    if lines.contains_key(line) {
        return Some(line.to_string());
    }
    let k: usize = line.strip_prefix('L')?.parse().ok()?;
    lines.keys().nth(k.checked_sub(1)?).cloned()
}

fn lines_key_norm(doc: &Value) -> &'static str {
    if doc.get("JournalLines").is_some() {
        "JournalLines"
    } else {
        "LineItems"
    }
}

/// `after` for a recode: one line's account and/or description changed.
pub fn recode_doc(
    doc: &Value,
    line: &str,
    account: Option<&str>,
    description: Option<&str>,
) -> Result<Value> {
    let key = line_key(doc, line)
        .ok_or_else(|| ManifestError::new("usage", format!("No line {line} on that document.")))?;
    let lk = lines_key_norm(doc);
    let mut a = doc.clone();
    let li = &mut a[lk][&key];
    if let Some(c) = account {
        li["AccountCode"] = json!(c);
    }
    if let Some(d) = description {
        li["Description"] = json!(d);
    }
    Ok(a)
}

/// `after` for a void.
pub fn void_doc(doc: &Value) -> Value {
    let mut a = doc.clone();
    a["Status"] = json!("VOIDED");
    a
}

/// `after` with a file attached (bound by its sha256; PDF, CSV or TXT).
pub fn attach_doc(doc: &Value, file: &std::path::Path) -> Result<Value> {
    let name = file
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| ManifestError::new("usage", "the file has no name"))?
        .to_string();
    let bytes = std::fs::read(file)
        .map_err(|e| ManifestError::new("usage", format!("{}: {e}", file.display())))?;
    let mime = crate::write::ops::mime_of(&name).unwrap_or("application/octet-stream");
    let mut a = doc.clone();
    if a.get("Attachments").is_none() {
        a["Attachments"] = json!({});
    }
    a["Attachments"][&name] = json!({
        "FileName": name,
        "MimeType": mime,
        "ContentLength": bytes.len(),
        "Sha256": crate::write::sha256_hex(&bytes),
        "Path": file.display().to_string(),
    });
    Ok(a)
}

/// A document's lines' key field for display (`JournalLines` / `LineItems`).
pub fn lines_of(doc: &Value) -> &'static str {
    lines_key(doc)
}

// ── undo, annotate, acknowledge, discard ────────────────────────────────────────────────────────

/// Undo an entry, or restore it (`undone: false`). Reversible, so no confirm.
pub fn undo(
    store: &mut Store,
    manifest: &str,
    actor: &str,
    seq: i64,
    undone: bool,
    g: &Guard,
) -> Result<i64> {
    let m = store.open_manifest(manifest)?;
    check(&m, actor, g, true)?;
    let c = m.entry(seq).ok_or_else(|| {
        ManifestError::new("not_found", format!("No entry {seq} on `{manifest}`."))
    })?;
    if c.undone == undone {
        return Err(ManifestError::new(
            "usage",
            format!(
                "Entry {seq} is already {}.",
                if undone { "undone" } else { "in the manifest" }
            ),
        ));
    }
    if !undone
        && let Some(other) = m.on_doc(&c.org, &c.entity, &c.id)
        && !c.id.starts_with("new:")
    {
        return Err(ManifestError::new(
            "usage",
            format!(
                "Entry {} already changes that document; undo it first (one entry per document).",
                other.seq
            ),
        ));
    }
    store.append(
        actor,
        manifest,
        Kind::EntryUndo,
        json!({"seq": seq, "undone": undone}),
    )
}

/// Attach an accountant decision reference and/or a note to an entry.
pub fn annotate(
    store: &mut Store,
    manifest: &str,
    actor: &str,
    seq: i64,
    accountant: Option<&str>,
    note: Option<&str>,
    g: &Guard,
) -> Result<i64> {
    let m = store.open_manifest(manifest)?;
    check(&m, actor, g, false)?;
    m.entry(seq).ok_or_else(|| {
        ManifestError::new("not_found", format!("No entry {seq} on `{manifest}`."))
    })?;
    if accountant.is_none() && note.is_none() {
        return Err(ManifestError::new(
            "usage",
            "annotate needs --accountant <ref> and/or --note <text>",
        ));
    }
    let mut p = json!({"seq": seq});
    if let Some(a) = accountant {
        p["accountant"] = json!(a);
    }
    if let Some(n) = note {
        p["note"] = json!(n);
    }
    store.append(actor, manifest, Kind::EntryAnnotate, p)
}

/// Acknowledge a warning (or withdraw the acknowledgement): an undoable event with the actor.
#[allow(clippy::too_many_arguments)]
pub fn ack(
    store: &mut Store,
    manifest: &str,
    actor: &str,
    seq: i64,
    code: &str,
    path: &str,
    acked: bool,
    g: &Guard,
) -> Result<i64> {
    let m = store.open_manifest(manifest)?;
    check(&m, actor, g, false)?;
    if acked == m.acked(seq, code, path).is_some() {
        return Err(ManifestError::new(
            "usage",
            if acked {
                "That warning is already acknowledged."
            } else {
                "That warning isn't acknowledged."
            },
        ));
    }
    store.append(
        actor,
        manifest,
        Kind::IssueAck,
        json!({"seq": seq, "code": code, "path": path, "acked": acked}),
    )
}

/// Discard a manifest (a marker; its history stays).
pub fn discard(store: &mut Store, manifest: &str, actor: &str, g: &Guard) -> Result<i64> {
    let m = store.open_manifest(manifest)?;
    check(&m, actor, g, true)?;
    store.append(actor, manifest, Kind::Discard, json!({}))
}

/// Record that a package was sealed from the manifest at its current content.
pub fn planned(
    store: &mut Store,
    manifest: &str,
    actor: &str,
    package: &str,
    hash: &str,
    from_rev: i64,
) -> Result<i64> {
    store.open_manifest(manifest)?;
    store.append(
        actor,
        manifest,
        Kind::Planned,
        json!({"package": package, "hash": hash, "from_rev": from_rev}),
    )
}

/// Choose how a reconciled bank line's recode is carried out: `reclass` (a reclass journal) or `browser`
/// (unreconcile → edit → re-reconcile in Xero, a checklist step). `seq` None sets the manifest's default.
pub fn resolve(
    store: &mut Store,
    manifest: &str,
    actor: &str,
    seq: Option<i64>,
    resolution: &str,
    g: &Guard,
) -> Result<i64> {
    let m = store.open_manifest(manifest)?;
    check(&m, actor, g, false)?;
    if !matches!(resolution, "reclass" | "browser" | "void-recreate") {
        return Err(ManifestError::new(
            "usage",
            format!(
                "resolution `{resolution}`: reclass or browser (reconciled bank lines), void-recreate (posted journals)"
            ),
        ));
    }
    if seq.is_none() && resolution == "void-recreate" {
        return Err(ManifestError::new(
            "usage",
            "void-recreate is chosen per entry, never as a default (voiding shows in Xero's history)",
        ));
    }
    if let Some(s) = seq {
        m.entry(s).ok_or_else(|| {
            ManifestError::new("not_found", format!("No entry {s} on `{manifest}`."))
        })?;
    }
    store.append(
        actor,
        manifest,
        Kind::Resolve,
        json!({"seq": seq, "resolution": resolution}),
    )
}
