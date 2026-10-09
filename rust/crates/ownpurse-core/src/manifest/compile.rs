//! PLAN (design §5): compile a checked manifest into the EXISTING sealed write package. Each entry becomes
//! the write `Op` its rule names (built by `write::ops`, with the live preconditions, expected deltas and
//! recovery the write path already uses); `Package.source` names the manifest. Refused while CHECK has
//! errors or unacknowledged warnings. Planning is local: it writes the package files and a
//! `manifest.planned` event; approving and applying stay with the write commands (CLI only).
//!
//! [`commands`] is the ONE builder of the approve / apply command lines; the CLI prints it and the TUI
//! shows it, byte for byte.

use super::check::Report;
use super::normalise::dec;
use super::rules::Matched;
use super::{Manifest, ManifestError, Result};
use crate::record::Record;
use crate::view::{Ctx, View};
use crate::write::ops::{self, AttachTo, JournalSpec, OrgCtx, RecodeSpec};
use crate::write::plan::{PlanConfig, lock_date, prior_before};
use crate::write::{JournalLine, Operation, PACKAGE_V, Package, SourceRef, gate};
use serde_json::Value;

/// The package id a manifest compiles to at a content revision: `<name>-r<rev>`.
pub fn package_id_of(name: &str, rev: i64) -> String {
    format!("{name}-r{rev}")
}

/// `manifest:<name>@r<rev>` (the record's `packages.source_ref`).
pub fn source_of(name: &str, rev: i64) -> String {
    format!("manifest:{name}@r{rev}")
}

/// The op id of an entry (`e<seq>`), and of its attachments (`e<seq>-a<k>`).
pub fn op_id(seq: i64) -> String {
    format!("e{seq}")
}

/// The command lines for a planned manifest: approve (the owner, at his own terminal) and apply (from a
/// terminal, with the one-run lock). The full hash is in both.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Commands {
    pub approve: String,
    pub apply: String,
}

pub fn commands(name: &str, hash: &str) -> Commands {
    Commands {
        approve: format!(
            "ownpurse manifest approve {name} --package {hash} --response-file <your-response.txt> --i-am-the-owner"
        ),
        apply: format!(
            "{}={hash} ownpurse manifest apply {name} --package {hash}",
            crate::write::RUN_ENV
        ),
    }
}

fn err(e: crate::write::WriteError) -> ManifestError {
    ManifestError::new(
        "plan",
        if e.help.is_empty() {
            e.message
        } else {
            format!("{} ({})", e.message, e.help.join("; "))
        },
    )
}

fn row(record: &Record, tid: &str, entity: &str, id: &str) -> Result<crate::record::Row> {
    record
        .current_rows(tid, entity)?
        .into_iter()
        .find(|r| r.xero_id == id)
        .ok_or_else(|| ManifestError::new("plan", format!("{entity} {id} is not in the record")))
}

/// Compile the manifest (its current content) into an unsealed package. `report` is its check.
pub fn compile(
    record: &Record,
    ctx: &Ctx,
    cfg: &PlanConfig,
    m: &Manifest,
    report: &Report,
    now: &str,
) -> Result<Package> {
    if report.errors() > 0 {
        return Err(ManifestError::new(
            "check",
            format!(
                "{} error{} block planning (ownpurse manifest check)",
                report.errors(),
                if report.errors() == 1 { "" } else { "s" }
            ),
        ));
    }
    if report.unacked() > 0 {
        return Err(ManifestError::new(
            "review",
            format!(
                "{} warning{} need acknowledging before planning (ownpurse manifest ack)",
                report.unacked(),
                if report.unacked() == 1 { "" } else { "s" }
            ),
        ));
    }
    let view = View::open(record, ctx, Some(m))?;
    let ov = view.overlay().expect("a manifest view");
    if ov.applied.is_empty() {
        return Err(ManifestError::new(
            "usage",
            "nothing to plan: the manifest has no entries to apply",
        ));
    }
    let pkg_id = package_id_of(&m.name, m.content_rev);
    let mut out: Vec<Operation> = vec![];
    let mut refs: Vec<String> = vec![];
    // reconciled-line recodes: one reclass journal per org per month (CLAUDE.md: one journal per month per
    // entity); browser resolutions become checklist steps
    let mut reclass: std::collections::BTreeMap<(String, String), Reclass> = Default::default();
    let mut browser: Vec<String> = vec![];
    for a in &ov.applied {
        let Some(e) = m.entry(a.seq) else { continue };
        let ctx_o = OrgCtx {
            package_id: pkg_id.clone(),
            org: a.org.clone(),
            tenant_id: a.tenant.clone(),
            accounts: View::main(record).payloads(&a.tenant, "Accounts")?,
            lock: lock_date(record, &a.tenant).map_err(err)?,
            short_code: crate::deeplink::short_code(record, &a.tenant),
        };
        let id = op_id(e.seq);
        let mut built: Vec<Operation> = vec![];
        let attachments = |after: &Value, before: Option<&Value>| -> Vec<std::path::PathBuf> {
            let had = before
                .and_then(|b| b.get("Attachments"))
                .and_then(Value::as_object);
            after
                .get("Attachments")
                .and_then(Value::as_object)
                .into_iter()
                .flatten()
                .filter(|(n, _)| !had.is_some_and(|h| h.contains_key(*n)))
                .filter_map(|(_, v)| v.get("Path").and_then(Value::as_str).map(Into::into))
                .collect()
        };
        match &a.matched {
            Matched::Op("reclass_journal") => {
                let before = a
                    .before
                    .as_ref()
                    .map(|b| super::normalise::denormalise(b, None));
                let after = super::normalise::denormalise(&a.after, a.before.as_ref());
                let moves = crate::write::delta::moves(
                    &ctx_o.accounts,
                    crate::write::delta::DocKind::BankTransaction,
                    before.as_ref(),
                    Some(&after),
                )
                .map_err(|x| ManifestError::new("plan", format!("entry {}: {x}", e.seq)))?;
                let key = e
                    .paths()
                    .iter()
                    .find_map(|p| {
                        p.trim_start_matches('/')
                            .split('/')
                            .nth(1)
                            .map(str::to_string)
                    })
                    .unwrap_or_default();
                let g = reclass
                    .entry((a.org.clone(), a.date.chars().take(7).collect()))
                    .or_insert_with(|| Reclass {
                        tenant: a.tenant.clone(),
                        ctx: OrgCtx {
                            package_id: pkg_id.clone(),
                            org: a.org.clone(),
                            tenant_id: a.tenant.clone(),
                            accounts: ctx_o.accounts.clone(),
                            lock: ctx_o.lock.clone(),
                            short_code: ctx_o.short_code.clone(),
                        },
                        ..Default::default()
                    });
                g.date = g.date.clone().max(a.date.clone());
                g.seqs.push(e.seq);
                g.what.push(format!("{}#{key}", e.id));
                for mv in moves {
                    g.lines.push(JournalLine {
                        account_code: mv.code.clone(),
                        description: format!(
                            "reclass {} line {key} ({}) {}",
                            e.id,
                            a.date,
                            if mv.amount > rust_decimal::Decimal::ZERO {
                                "to"
                            } else {
                                "from"
                            }
                        ),
                        amount: mv.amount,
                    });
                }
                g.refs.push(SourceRef {
                    id: format!("{}#{}", source_of(&m.name, m.content_rev), e.seq),
                    content_hash: entry_hash(e),
                });
                if let Some(r) = &e.refs.accountant_decision {
                    g.accountant.push(r.clone());
                }
                continue;
            }
            Matched::Op("browser_recode") => {
                let d = super::fields::doc_diff(&e.entity, "change", a.before.as_ref(), &a.after);
                let what: Vec<String> = d
                    .lines
                    .iter()
                    .flat_map(|l| l.rows.iter())
                    .filter(|r| r.mark != super::fields::Mark::Same)
                    .map(|r| {
                        let c = &r.cells[0];
                        format!(
                            "line {} account {} → {}",
                            r.key,
                            c.old.as_deref().unwrap_or(""),
                            c.new.as_deref().unwrap_or("")
                        )
                    })
                    .collect();
                browser.push(format!(
                    "in Xero (browser), {}: unreconcile bank transaction {} ({}), change {}, then re-reconcile it (manifest {} entry {})",
                    a.org,
                    e.id,
                    a.date,
                    what.join("; "),
                    m.name,
                    e.seq
                ));
                continue;
            }
            Matched::Unresolved { message, .. } => {
                return Err(ManifestError::new(
                    "check",
                    format!("entry {}: {message}", e.seq),
                ));
            }
            Matched::Op("create_journal") => {
                let payload = super::normalise::denormalise(&a.after, None);
                let lines: Vec<JournalLine> = payload["JournalLines"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .map(|l| JournalLine {
                        account_code: l["AccountCode"].as_str().unwrap_or("").into(),
                        description: l["Description"].as_str().unwrap_or("").into(),
                        amount: dec(l, "LineAmount"),
                    })
                    .collect();
                let mut v = ops::create_journal(
                    &ctx_o,
                    &JournalSpec {
                        id: id.clone(),
                        date: a.after["Date"].as_str().unwrap_or("").into(),
                        narration: a.after["Narration"].as_str().unwrap_or("").into(),
                        cash_basis: a.after["ShowOnCashBasisReports"].as_bool().unwrap_or(true),
                        lines,
                        max_lines: None,
                        split_account: None,
                    },
                )
                .map_err(err)?;
                for (k, path) in attachments(&a.after, None).into_iter().enumerate() {
                    v.push(
                        ops::attach(
                            &ctx_o,
                            &format!("{id}-a{}", k + 1),
                            AttachTo::CreatedBy(id.clone()),
                            &path,
                        )
                        .map_err(err)?,
                    );
                }
                built.extend(v);
            }
            Matched::Op("void_recreate") => {
                // void the original, then create the corrected journal (it names what it replaces)
                let r = row(record, &a.tenant, "ManualJournals", &e.id)?;
                let mut void = ops::void_journal(&ctx_o, &format!("{id}-void"), &r).map_err(err)?;
                void.flags
                    .push(format!("void + re-create: the replacement is {id}"));
                built.push(void);
                let payload = super::normalise::denormalise(&a.after, a.before.as_ref());
                let lines: Vec<JournalLine> = payload["JournalLines"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .map(|l| JournalLine {
                        account_code: l["AccountCode"].as_str().unwrap_or("").into(),
                        description: l["Description"].as_str().unwrap_or("").into(),
                        amount: dec(l, "LineAmount"),
                    })
                    .collect();
                let narration = a.after["Narration"].as_str().unwrap_or("").to_string();
                let mut v = ops::create_journal(
                    &ctx_o,
                    &JournalSpec {
                        id: id.clone(),
                        date: a.after["Date"].as_str().unwrap_or("").into(),
                        narration: format!("{narration} (replaces {})", e.id),
                        cash_basis: a.after["ShowOnCashBasisReports"].as_bool().unwrap_or(true),
                        lines,
                        max_lines: None,
                        split_account: None,
                    },
                )
                .map_err(err)?;
                for op in &mut v {
                    op.depends_on.push(format!("{id}-void"));
                    op.flags
                        .push(format!("void + re-create: replaces {}", e.id));
                }
                built.extend(v);
            }
            Matched::Op("create_bank_transaction") => {
                let payload = super::normalise::denormalise(&a.after, None);
                let lines: Vec<JournalLine> = payload["LineItems"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .map(|l| JournalLine {
                        account_code: l["AccountCode"].as_str().unwrap_or("").into(),
                        description: l["Description"].as_str().unwrap_or("").into(),
                        amount: dec(l, "LineAmount"),
                    })
                    .collect();
                let s = |p: &str| {
                    payload
                        .pointer(p)
                        .and_then(Value::as_str)
                        .unwrap_or("")
                        .to_string()
                };
                let bank = Some(s("/BankAccount/AccountID"))
                    .filter(|x| !x.is_empty())
                    .unwrap_or_else(|| s("/BankAccount/Code"));
                let contact = Some(s("/Contact/ContactID"))
                    .filter(|x| !x.is_empty())
                    .unwrap_or_else(|| s("/Contact/Name"));
                let contact = crate::write::plan::resolve_contact(
                    record,
                    &a.tenant,
                    &[],
                    &a.org,
                    &id,
                    &contact,
                )
                .map_err(err)?;
                built.push(
                    ops::create_bank_transaction(
                        &ctx_o,
                        &ops::BankTxnSpec {
                            id: id.clone(),
                            kind: s("/Type"),
                            bank_account: bank,
                            contact,
                            date: a.after["Date"].as_str().unwrap_or("").into(),
                            reference: s("/Reference"),
                            lines,
                        },
                    )
                    .map_err(err)?,
                );
            }
            Matched::Op("create_bank_transfer") => {
                let payload = super::normalise::denormalise(&a.after, None);
                let s = |p: &str| {
                    payload
                        .pointer(p)
                        .and_then(Value::as_str)
                        .unwrap_or("")
                        .to_string()
                };
                let acct = |base: &str| {
                    Some(s(&format!("/{base}/AccountID")))
                        .filter(|x| !x.is_empty())
                        .unwrap_or_else(|| s(&format!("/{base}/Code")))
                };
                built.push(
                    ops::create_bank_transfer(
                        &ctx_o,
                        &id,
                        &acct("FromBankAccount"),
                        &acct("ToBankAccount"),
                        dec(&payload, "Amount"),
                        a.after["Date"].as_str().unwrap_or(""),
                        &s("/Reference"),
                    )
                    .map_err(err)?,
                );
            }
            Matched::Op("create_contact") => {
                let payload = super::normalise::denormalise(&a.after, None);
                built.push(
                    ops::create_contact(
                        &ctx_o,
                        &id,
                        payload["Name"].as_str().unwrap_or(""),
                        payload["EmailAddress"].as_str(),
                    )
                    .map_err(err)?,
                );
            }
            Matched::Op("void_bank_transaction") => {
                let r = row(record, &a.tenant, "BankTransactions", &e.id)?;
                built.push(ops::void_bank_transaction(&ctx_o, &id, &r).map_err(err)?);
            }
            Matched::Op("void_journal") => {
                let r = row(record, &a.tenant, "ManualJournals", &e.id)?;
                built.push(ops::void_journal(&ctx_o, &id, &r).map_err(err)?);
            }
            Matched::Op("attach") => {
                let r = row(record, &a.tenant, &e.entity, &e.id)?;
                for (k, path) in attachments(&a.after, a.before.as_ref())
                    .into_iter()
                    .enumerate()
                {
                    let oid = if k == 0 {
                        id.clone()
                    } else {
                        format!("{id}-a{}", k + 1)
                    };
                    let to = if e.entity == "BankTransactions" {
                        AttachTo::BankTransaction(&r)
                    } else {
                        AttachTo::Journal(&r)
                    };
                    built.push(ops::attach(&ctx_o, &oid, to, &path).map_err(err)?);
                }
            }
            Matched::Op("add_note") => {
                let shape_notes: Vec<String> = a
                    .after
                    .get("History")
                    .and_then(Value::as_array)
                    .into_iter()
                    .flatten()
                    .skip(
                        a.before
                            .as_ref()
                            .and_then(|b| b.get("History"))
                            .and_then(Value::as_array)
                            .map_or(0, Vec::len),
                    )
                    .filter_map(|n| n.get("Details").and_then(Value::as_str).map(str::to_string))
                    .collect();
                for (k, details) in shape_notes.iter().enumerate() {
                    let oid = if k == 0 {
                        id.clone()
                    } else {
                        format!("{id}-n{}", k + 1)
                    };
                    built.push(
                        ops::add_note(
                            &ctx_o,
                            &oid,
                            &e.entity,
                            crate::write::JournalRef::Id(e.id.clone()),
                            details,
                        )
                        .map_err(err)?,
                    );
                }
            }
            Matched::Op("recode_bank_line") => {
                let r = row(record, &a.tenant, "BankTransactions", &e.id)?;
                let key = e
                    .paths()
                    .iter()
                    .find_map(|p| {
                        p.trim_start_matches('/')
                            .split('/')
                            .nth(1)
                            .map(str::to_string)
                    })
                    .unwrap_or_default();
                let before_line = a.before.as_ref().map(|b| b["LineItems"][&key].clone());
                let after_line = &a.after["LineItems"][&key];
                let changed = |f: &str| {
                    let n = after_line.get(f).and_then(Value::as_str);
                    let o = before_line
                        .as_ref()
                        .and_then(|b| b.get(f))
                        .and_then(Value::as_str);
                    (n != o).then(|| n.unwrap_or("").to_string())
                };
                let line_item_id = before_line
                    .as_ref()
                    .and_then(|l| l.get("LineItemID"))
                    .and_then(Value::as_str)
                    .unwrap_or(&key)
                    .to_string();
                built.push(
                    ops::recode_bank_line(
                        &ctx_o,
                        &r,
                        &RecodeSpec {
                            id: id.clone(),
                            line_item_id,
                            account: changed("AccountCode"),
                            description: changed("Description"),
                        },
                    )
                    .map_err(err)?,
                );
            }
            Matched::Op(other) => {
                return Err(ManifestError::new(
                    "plan",
                    format!("entry {}: no builder for {other}", e.seq),
                ));
            }
            Matched::NotWritable { message, .. } => {
                return Err(ManifestError::new(
                    "check",
                    format!("entry {}: {message}", e.seq),
                ));
            }
        }
        let prior = prior_before(record, &a.tenant).map_err(err)?;
        let entry_hash = entry_hash(e);
        for mut op in built {
            op.gated = ops::effect_date(&op)
                .and_then(|d| gate::needs_accountant(&a.org, d, &cfg.gates, prior.as_deref()));
            op.decision_refs.push(SourceRef {
                id: format!("{}#{}", source_of(&m.name, m.content_rev), e.seq),
                content_hash: entry_hash.clone(),
            });
            for s in &e.refs.sources {
                op.source_refs.push(SourceRef {
                    id: s.clone(),
                    content_hash: String::new(),
                });
            }
            if op.gated.is_some()
                && let Some(r) = &e.refs.accountant_decision
                && !refs.contains(r)
            {
                refs.push(r.clone());
            }
            out.push(op);
        }
    }
    for ((org, month), g) in reclass {
        let id = reclass_op_id(&org, &month);
        let narration = format!(
            "Reclass of reconciled bank lines, {month}: {}",
            g.what.join(", ")
        );
        let v = ops::create_journal(
            &g.ctx,
            &JournalSpec {
                id: id.clone(),
                date: g.date.clone(),
                narration,
                cash_basis: true,
                lines: g.lines.clone(),
                max_lines: None,
                split_account: None,
            },
        )
        .map_err(err)?;
        let prior = prior_before(record, &g.tenant).map_err(err)?;
        for mut op in v {
            op.gated = ops::effect_date(&op)
                .and_then(|d| gate::needs_accountant(&org, d, &cfg.gates, prior.as_deref()));
            op.decision_refs.extend(g.refs.iter().cloned());
            op.flags.push(format!(
                "reclass journal for {} reconciled line{}",
                g.seqs.len(),
                if g.seqs.len() == 1 { "" } else { "s" }
            ));
            if op.gated.is_some() {
                for r in &g.accountant {
                    if !refs.contains(r) {
                        refs.push(r.clone());
                    }
                }
            }
            out.push(op);
        }
    }
    if out.is_empty() {
        return Err(ManifestError::new(
            "usage",
            "nothing to apply through the API: every entry is a browser step (see the checklist with `manifest show`)",
        ));
    }
    let mut checklist = vec![];
    checklist.extend(browser);
    let years: std::collections::BTreeSet<(String, String)> = out
        .iter()
        .filter(|o| o.gated.is_some())
        .filter_map(|o| ops::effect_date(o).map(|d| (o.org.clone(), d.chars().take(4).collect())))
        .collect();
    for (org, y) in years {
        checklist.push(format!(
            "after posting the {y} corrections, set the {y} lock date for {org} in Xero (browser; the API can't set lock dates)"
        ));
    }
    if cfg.history_notes {
        let title = if m.why.is_empty() {
            format!("manifest {}", m.name)
        } else {
            m.why.clone()
        };
        crate::write::plan::with_history_notes(&mut out, &title);
    }
    Ok(Package {
        v: PACKAGE_V,
        id: pkg_id,
        title: if m.why.is_empty() {
            format!("manifest {}", m.name)
        } else {
            m.why.clone()
        },
        profile: cfg.profile.clone(),
        created_at: now.into(),
        accountant_decision: (!refs.is_empty()).then(|| refs.join("; ")),
        ops: out,
        excluded: vec![],
        checklist,
        supersedes: m.planned.iter().map(|p| p.hash.clone()).collect(),
        seal: None,
        source: Some(source_of(&m.name, m.content_rev)),
    })
}

/// One org-month's reclass journal being collected.
#[derive(Default)]
struct Reclass {
    tenant: String,
    ctx: OrgCtx,
    /// The latest line date in the month (the journal's date).
    date: String,
    seqs: Vec<i64>,
    /// `bt-110#n1`, …
    what: Vec<String>,
    lines: Vec<JournalLine>,
    refs: Vec<SourceRef>,
    accountant: Vec<String>,
}

/// The op id of an org-month's reclass journal: `reclass-<org>-<yyyymm>`.
pub fn reclass_op_id(org: &str, date_or_month: &str) -> String {
    let ym: String = date_or_month
        .chars()
        .take(7)
        .filter(|c| c.is_ascii_digit())
        .collect();
    format!("reclass-{org}-{ym}")
}

/// The content hash of an entry (its decision reference in the package).
pub fn entry_hash(e: &super::Entry) -> String {
    crate::write::sha256_hex(
        crate::canon::canonical(&serde_json::json!({
            "entity": e.entity, "org": e.org, "id": e.id, "before_fp": e.before_fp, "patch": e.patch,
        }))
        .as_bytes(),
    )
}

/// A command with its hash abbreviated for display (`27b8b424…7c`); the copy and the state keep the full one.
pub fn abbreviate(cmd: &str, hash: &str) -> String {
    if hash.len() < 12 {
        return cmd.to_string();
    }
    cmd.replace(hash, &format!("{}…{}", &hash[..8], &hash[hash.len() - 2..]))
}
