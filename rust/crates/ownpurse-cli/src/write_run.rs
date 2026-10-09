//! `ownpurse write run | verify | reconcile | recover | status | probe` (Phase 8, the run side). The package
//! side (plan, seal, export, approve-*) is `write.rs`. Every command here that can write needs the three
//! locks (profile `allow_writes`, `OWNPURSE_WRITE_RUN=<hash>` for this one invocation, a recorded approval).

use crate::app::{App, CliError, Res};
use crate::out::Out;
use ownpurse_core::write::{BATCH_MAX, Package, RUN_ENV, WriteError, exit, seal};
use ownpurse_xero::TokenStore;
use ownpurse_xero::store::Writer;
use ownpurse_xero::write::run::Replay;
use ownpurse_xero::write::run::{self, Ctx, seal_of};
use ownpurse_xero::write::{RecordApprovals, UreqWire, WriteSettings, XeroWriter};
use serde_json::{Value, json};
use std::path::Path;

fn setting(app: &App, key: &str) -> String {
    app.config.settings.get_str(key).unwrap_or("").to_string()
}

fn load(pkg: &Path) -> Res<Package> {
    Ok(seal::load(pkg)?)
}

fn settings(app: &App) -> WriteSettings {
    let p = &app.config.profile;
    WriteSettings {
        profile: p.name.clone(),
        api_base: setting(app, "xero.api_base"),
        connections_url: setting(app, "xero.connections_url"),
        orgs: p.orgs.clone(),
        forbid_tenants: p.forbid_tenants.clone(),
        require_tenant_name: p.require_tenant_name.clone(),
        allow_writes: p.allow_writes,
        loopback_only: WriteSettings::loopback_from_env(),
    }
}

fn tokens(app: &App) -> TokenStore {
    TokenStore::new(
        &app.config.profile.token_file,
        app.config.profile.client_id.as_str(),
    )
    .with_token_url(setting(app, "xero.token_url"))
}

fn refuse_offline(app: &App, cmd: &str) -> Res<()> {
    if app.g.offline {
        return Err(CliError::new(
            "offline",
            exit::WRITES_DISABLED,
            format!("`write {cmd}` needs Xero, and --offline forbids network calls."),
        ));
    }
    Ok(())
}

/// Run `f` with a run context on this profile's record, token and transport.
fn with_ctx<T>(
    app: &App,
    pkg: &Path,
    dry_run: bool,
    batch: usize,
    f: impl FnOnce(&mut Ctx<'_>) -> Result<T, WriteError>,
) -> Res<T> {
    let store = Writer::open(app.config.store())?;
    let tokens = tokens(app);
    let mut x = XeroWriter::new(
        settings(app),
        &store,
        &tokens,
        Box::new(UreqWire::default()),
    );
    let approvals = RecordApprovals { store: &store };
    let mut ctx = Ctx {
        transport: &mut x,
        store: &store,
        approvals: &approvals,
        profile: app.config.profile.name.clone(),
        allow_writes: app.config.profile.allow_writes,
        run_env: std::env::var(RUN_ENV).ok(),
        base_dir: pkg
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| ".".into()),
        batch,
        dry_run,
        require_tenant_name: app.config.profile.require_tenant_name.clone(),
    };
    Ok(f(&mut ctx)?)
}

fn emit(app: &App, v: Value) {
    let mut out = Out::new(app.g.json);
    if let Some(m) = v.as_object() {
        if app.g.json {
            for (k, x) in m {
                out.field(k, x.clone());
            }
        } else {
            text_fields(&mut out, m);
        }
    }
    out.emit();
}

/// A nested value as text a person reads: scalars as they are, anything deeper as compact JSON (never a
/// debug dump).
fn cell(v: &Value) -> Value {
    match v {
        Value::Object(_) | Value::Array(_) => Value::String(ownpurse_core::canon::canonical(v)),
        other => other.clone(),
    }
}

/// Text mode: scalars as fields, objects flattened one level (`stopped.error`), lists of objects as tables
/// (their object cells flattened one level too: `detail.method`, `detail.path`).
fn text_fields(out: &mut Out, m: &serde_json::Map<String, Value>) {
    for (k, x) in m {
        match x {
            Value::Object(o) => {
                for (k2, y) in o {
                    out.field(&format!("{k}.{k2}"), cell(y));
                }
            }
            Value::Array(a) if !a.is_empty() && a.iter().all(Value::is_object) => {
                let rows: Vec<serde_json::Map<String, Value>> = a
                    .iter()
                    .filter_map(Value::as_object)
                    .map(|r| {
                        let mut flat = serde_json::Map::new();
                        for (c, y) in r {
                            match y {
                                Value::Object(o) => {
                                    for (c2, z) in o {
                                        flat.insert(format!("{c}.{c2}"), cell(z));
                                    }
                                }
                                other => {
                                    flat.insert(c.clone(), cell(other));
                                }
                            }
                        }
                        flat
                    })
                    .collect();
                let mut cols: Vec<String> = vec![];
                for r in &rows {
                    for c in r.keys() {
                        if !cols.contains(c) {
                            cols.push(c.clone());
                        }
                    }
                }
                out.table_cols(k, cols, rows, None);
            }
            other => {
                out.field(k, cell(other));
            }
        }
    }
}

/// `write run <pkg> [--batch N] [--dry-run]`
pub fn run_cmd(app: &App, pkg: &Path, batch: Option<usize>, dry_run: bool) -> Res<()> {
    refuse_offline(app, "run")?;
    let p = load(pkg)?;
    let rep = with_ctx(app, pkg, dry_run, batch.unwrap_or(BATCH_MAX), |c| {
        run::run(&p, c)
    })?;
    let mut v = rep.to_json();
    v["dry_run"] = json!(dry_run);
    emit(app, v);
    match rep.stopped {
        Some(e) => Err(e.into()),
        None => Ok(()),
    }
}

/// `write verify <pkg>`
pub fn verify_cmd(app: &App, pkg: &Path) -> Res<()> {
    refuse_offline(app, "verify")?;
    let p = load(pkg)?;
    let v = with_ctx(app, pkg, true, BATCH_MAX, |c| run::verify(&p, c))?;
    let ok = v["ok"] == true;
    let outcome = v["outcome"].as_str().unwrap_or("").to_string();
    if ok {
        emit(app, v);
        return Ok(());
    }
    // not verified: the exit code says why (the answer above has the details)
    let (code, why) = match outcome.as_str() {
        "discrepancy" => (
            exit::DISCREPANCY,
            "the package landed but something differs from what was expected",
        ),
        "uncertain" => (
            exit::UNCERTAIN,
            "an attempt is uncertain: reconcile it first",
        ),
        "rejected" => (
            exit::PRECONDITION,
            "an operation was refused; this package can't be verified as landed",
        ),
        _ => (exit::PRECONDITION, "not every operation has landed yet"),
    };
    // one answer: the error, carrying everything verify found
    let mut e = CliError::new(&outcome, code, why);
    if let Some(m) = v.as_object() {
        for (k, x) in m {
            if k != "outcome" {
                e.detail.push((k.clone(), x.clone()));
            }
        }
    }
    let rejected: Vec<String> = v["targets"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|t| Some(format!("{}: {}", t["op"].as_str()?, t["error"].as_str()?)))
        .collect();
    if !rejected.is_empty() {
        e.message = format!("{why}: {}", rejected.join("; "));
    }
    Err(e)
}

/// `write reconcile <pkg> <op>`
pub fn reconcile_cmd(app: &App, pkg: &Path, op: &str) -> Res<()> {
    refuse_offline(app, "reconcile")?;
    let p = load(pkg)?;
    let v = with_ctx(app, pkg, true, BATCH_MAX, |c| run::reconcile(&p, op, c))?;
    emit(app, v);
    Ok(())
}

/// `write status <pkg>` (record only, no network).
pub fn status_cmd(app: &mut App, pkg: &Path) -> Res<()> {
    let p = load(pkg)?;
    let links = ownpurse_core::write::links(app.record()?, &p);
    let store = Writer::open(app.config.store())?;
    let mut v = run::status_with(&p, &store, &links)?;
    v["sealed"] = json!(p.seal.as_deref() == Some(seal_of(&p).as_str()));
    emit(app, v);
    Ok(())
}

/// `write recover <pkg> [op] [--out FILE]`: the recovery package (unsealed; never executed here).
pub fn recover_cmd(app: &App, pkg: &Path, op: Option<&str>, out: Option<&Path>) -> Res<()> {
    refuse_offline(app, "recover")?;
    let p = load(pkg)?;
    let rec = with_ctx(app, pkg, true, BATCH_MAX, |c| run::recover(&p, op, c))?;
    let dest = out.map(Path::to_path_buf).unwrap_or_else(|| {
        let stem = pkg
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("package");
        pkg.with_file_name(format!("{stem}.recovery.json"))
    });
    seal::save(&dest, &rec)?;
    emit(
        app,
        json!({"recovery": dest.display().to_string(), "ops": rec.ops.iter().map(|o| json!({"op": o.id, "kind": o.op.kind(), "flags": o.flags})).collect::<Vec<_>>(),
               "excluded": rec.excluded.iter().map(|e| json!({"what": e.what, "why": e.why})).collect::<Vec<_>>(),
               "next": format!("ownpurse write seal {}  (then its own approval)", dest.display())}),
    );
    Ok(())
}

/// `write probe-replay <pkg> <op> [--changed]` (Milestone 1, the Demo Company only).
pub fn replay_cmd(app: &App, pkg: &Path, op: &str, changed: bool) -> Res<()> {
    refuse_offline(app, "probe-replay")?;
    let p = load(pkg)?;
    let kind = if changed {
        Replay::Changed
    } else {
        Replay::Same
    };
    let v = with_ctx(app, pkg, false, 1, |c| run::replay(&p, op, kind, c))?;
    emit(app, v);
    Ok(())
}

/// The Milestone 1 probes (`write probe <name>`): plan inputs built from the synced demo record.
pub const PROBES: [&str; 8] = [
    "recode",
    "void-after-attachment",
    "idempotency",
    "line-limit",
    "history-note",
    "bank-transaction",
    "bank-transfer",
    "contact",
];

/// `write probe <name> [--org ALIAS] [--lines N] [--out DIR]`: write `<name>.ops.json` (and any file it
/// attaches) for `write plan`, from the synced record of a Demo Company profile; print the next commands.
pub fn probe_cmd(
    app: &mut App,
    name: &str,
    org: Option<&str>,
    lines: Option<usize>,
    out: Option<&Path>,
) -> Res<()> {
    let p = app.config.profile.clone();
    if !p
        .require_tenant_name
        .as_deref()
        .is_some_and(|n| n.contains("Demo Company"))
    {
        return Err(CliError::new(
            "not_a_demo",
            exit::USAGE,
            "The probes run only on a profile whose require_tenant_name names a Demo Company.",
        )
        .hint("Run them from the Demo Company's own folder and profile."));
    }
    let alias = match org {
        Some(a) => a.to_string(),
        None if p.orgs.len() == 1 => p.orgs[0].0.clone(),
        None => {
            return Err(CliError::usage(
                "This profile has several orgs: pass --org.",
            ));
        }
    };
    let tenant = p
        .tenant(&alias)
        .ok_or_else(|| CliError::usage(format!("{alias} is not an org of profile {}.", p.name)))?
        .to_string();
    let dir = out
        .map(Path::to_path_buf)
        .unwrap_or_else(|| "probes".into());
    std::fs::create_dir_all(&dir)
        .map_err(|e| CliError::usage(format!("{}: {e}", dir.display())))?;
    let rec = app.record()?;
    let accounts = ownpurse_core::view::View::main(rec).payloads(&tenant, "Accounts")?;
    let expense: Vec<String> = accounts
        .iter()
        .filter(|a| a["Status"] == "ACTIVE" && (a["Class"] == "EXPENSE" || a["Type"] == "EXPENSE"))
        .filter_map(|a| {
            a["Code"]
                .as_str()
                .filter(|c| !c.is_empty())
                .map(str::to_string)
        })
        .collect();
    if expense.len() < 2 {
        return Err(CliError::usage(
            "The record has fewer than two active expense accounts: sync the demo org first.",
        ));
    }
    // today (UTC), from the same clock the record uses
    let today = ownpurse_xero::store::now_iso()[..10].to_string();
    let stamp = today.replace('-', "");
    let journal = |id: &str, n: usize, narration: &str, extra: Value| -> Value {
        // n-1 lines of 0.01 on one expense account, balanced on another (tiny, irregular, in an open period)
        let mut ls: Vec<Value> = (1..n)
            .map(|i| json!({"account": expense[0], "amount": "0.01", "description": format!("probe line {i}")}))
            .collect();
        let total = (n - 1) as i64;
        ls.push(json!({"account": expense[1], "amount": format!("-{}.{:02}", total / 100, total % 100), "description": "probe balance"}));
        let mut v = json!({"kind": "create_journal", "id": id, "org": alias, "date": today, "narration": narration,
                           "cash_basis": true, "lines": ls});
        if let (Some(m), Some(e)) = (v.as_object_mut(), extra.as_object()) {
            for (k, x) in e {
                m.insert(k.clone(), x.clone());
            }
        }
        v
    };
    let (ops, files): (Vec<Value>, Vec<(String, String)>) = match name {
        // probed on the Demo Company (2026-10-07): Xero refuses any edit of a reconciled transaction
        "recode-reconciled" => {
            return Err(CliError::usage(
                "Xero refuses to edit a reconciled bank transaction (probed on the Demo Company): `write plan` refuses it too.",
            )
            .hint("Use the `recode` probe (an unreconciled line), or a reclass journal for a reconciled one."));
        }
        "recode" => {
            let bts = ownpurse_core::view::View::main(rec).payloads(&tenant, "BankTransactions")?;
            let pick = bts.iter().find_map(|t| {
                if t["Status"] != "AUTHORISED" || t["IsReconciled"] == true || t["Type"] != "SPEND"
                {
                    return None;
                }
                let l = t["LineItems"]
                    .as_array()?
                    .iter()
                    .find(|l| l["LineItemID"].as_str().is_some_and(|s| !s.is_empty()))?;
                let code = l["AccountCode"].as_str()?;
                let alt = expense.iter().find(|c| *c != code)?;
                Some((
                    t["BankTransactionID"].as_str()?.to_string(),
                    l["LineItemID"].as_str()?.to_string(),
                    alt.clone(),
                ))
            });
            let Some((bt, li, alt)) = pick else {
                return Err(CliError::usage(
                    "No unreconciled SPEND line with a LineItemID in the record: sync BankTransactions first.",
                ));
            };
            (
                vec![
                    json!({"kind": "recode_bank_line", "id": "op-1", "org": alias, "bank_transaction": bt, "line_item": li, "account": alt}),
                ],
                vec![],
            )
        }
        "void-after-attachment" => {
            let csv = format!("probe,date,amount\nvoid-after-attachment,{today},0.01\n");
            (
                vec![
                    journal(
                        "op-1",
                        2,
                        "Milestone 1 probe: void after attachment",
                        json!({}),
                    ),
                    json!({"kind": "attach", "id": "op-2", "org": alias, "journal": "op:op-1", "file": "probe-support.csv"}),
                    json!({"kind": "void_journal", "id": "op-3", "org": alias, "journal": "op:op-1", "depends_on": ["op-2"]}),
                ],
                vec![("probe-support.csv".into(), csv)],
            )
        }
        "idempotency" => (
            vec![journal(
                "op-1",
                2,
                "Milestone 1 probe: Idempotency-Key replay",
                json!({"probe_replay": true}),
            )],
            vec![],
        ),
        "line-limit" => {
            let n = lines.unwrap_or(100).max(2);
            (
                vec![journal(
                    "op-1",
                    n,
                    &format!("Milestone 1 probe: {n} journal lines"),
                    json!({"max_lines": n}),
                )],
                vec![],
            )
        }
        // the audit trail: a journal (its own audit note, the package forcing history_notes on) and an
        // explicit note on it (op E); the run reads each back from the journal's history
        "history-note" => (
            vec![
                journal("op-1", 2, "API coverage probe: history notes", json!({})),
                json!({"kind": "add_note", "id": "op-2", "org": alias, "entity": "ManualJournals",
                       "document": "op:op-1", "details": "ownpurse probe: an explicit note (op E) after the audit note"}),
            ],
            vec![],
        ),
        // a 0.01 SPEND on the first active bank account, to an existing contact; its recovery (write recover)
        // voids it with Status DELETED: verifies on the Demo Company that this is how the API voids one
        "bank-transaction" => {
            let bank = accounts
                .iter()
                .find(|a| a["Type"] == "BANK" && a["Status"] == "ACTIVE")
                .and_then(|a| a["AccountID"].as_str())
                .map(str::to_string);
            let contact = ownpurse_core::view::View::main(rec)
                .payloads(&tenant, "Contacts")?
                .iter()
                .find(|c| c["ContactStatus"] == "ACTIVE")
                .and_then(|c| c["ContactID"].as_str())
                .map(str::to_string);
            let (Some(bank), Some(contact)) = (bank, contact) else {
                return Err(CliError::usage(
                    "The record has no active bank account or contact: sync Accounts and Contacts first.",
                ));
            };
            (
                vec![
                    json!({"kind": "create_bank_transaction", "id": "op-1", "org": alias, "type": "SPEND",
                            "bank_account": bank, "contact": contact, "date": today,
                            "reference": "API coverage probe: bank transaction",
                            "lines": [{"account": expense[0], "amount": "0.01", "description": "probe"}],
                            "allow_duplicate": true}),
                ],
                vec![],
            )
        }
        // 0.01 between the first two active bank accounts; its recovery is a reverse transfer
        "bank-transfer" => {
            let banks: Vec<String> = accounts
                .iter()
                .filter(|a| a["Type"] == "BANK" && a["Status"] == "ACTIVE")
                .filter_map(|a| a["AccountID"].as_str().map(str::to_string))
                .take(2)
                .collect();
            let [from, to] = banks.as_slice() else {
                return Err(CliError::usage(
                    "The record has fewer than two active bank accounts: sync Accounts first.",
                ));
            };
            (
                vec![
                    json!({"kind": "create_bank_transfer", "id": "op-1", "org": alias, "from": from,
                            "to": to, "amount": "0.01", "date": today,
                            "reference": "API coverage probe: transfer", "allow_duplicate": true}),
                ],
                vec![],
            )
        }
        // a contact named for today; its recovery archives it
        "contact" => (
            vec![json!({"kind": "create_contact", "id": "op-1", "org": alias,
                        "name": format!("ownpurse probe contact {stamp}")})],
            vec![],
        ),
        other => {
            return Err(CliError::usage(format!(
                "No probe {other}. Probes: {}.",
                PROBES.join(", ")
            )));
        }
    };
    let id = format!("probe-{name}-{stamp}");
    let mut input = json!({"id": id, "title": format!("Milestone 1 probe: {name}"), "ops": ops,
                       "checklist": ["Demo Company only; reset the Demo Company afterwards if you like"]});
    if name == "history-note" {
        input["history_notes"] = json!(true);
    }
    let file = dir.join(format!("{name}.ops.json"));
    std::fs::write(
        &file,
        serde_json::to_string_pretty(&input).unwrap_or_default(),
    )
    .map_err(|e| CliError::usage(format!("{}: {e}", file.display())))?;
    for (n, text) in &files {
        std::fs::write(dir.join(n), text).map_err(|e| CliError::usage(format!("{n}: {e}")))?;
    }
    let pkg = dir.join(format!("{name}.package.json"));
    let mut next = vec![
        format!("ownpurse write plan {}", file.display()),
        format!("ownpurse write seal {}", pkg.display()),
        // a Demo Company profile (the only one the probes run on): the owner's standing approval
        format!("ownpurse write approve-demo {}", pkg.display()),
        format!(
            "OWNPURSE_WRITE_RUN=<hash> ownpurse write run {} --dry-run",
            pkg.display()
        ),
        format!(
            "OWNPURSE_WRITE_RUN=<hash> ownpurse write run {}",
            pkg.display()
        ),
        format!("ownpurse write verify {}", pkg.display()),
    ];
    match name {
        "bank-transfer" => next.push(format!(
            "ownpurse write recover {}   (the reverse transfer; both stay in the accounts' history)",
            pkg.display()
        )),
        "contact" => next.push(format!(
            "ownpurse write recover {}   (archives the contact)",
            pkg.display()
        )),
        "bank-transaction" => next.push(format!(
            "ownpurse write recover {}   (the void: Status DELETED; its own seal and approval, then run and verify it)",
            pkg.display()
        )),
        "history-note" => next.push(
            "ownpurse history <org> ManualJournals <the journal id from write status>   (both notes, read back)"
                .into(),
        ),
        "recode" => next.push(format!(
            "ownpurse write recover {}   (the inverse recode, its own seal and approval)",
            pkg.display()
        )),
        "idempotency" => {
            next.push(format!(
                "OWNPURSE_WRITE_RUN=<hash> ownpurse write probe-replay {} op-1",
                pkg.display()
            ));
            next.push(format!(
                "OWNPURSE_WRITE_RUN=<hash> ownpurse write probe-replay {} op-1 --changed",
                pkg.display()
            ));
        }
        _ => {}
    }
    next.push(format!("ownpurse write status {}", pkg.display()));
    if app.g.json {
        emit(
            app,
            json!({"probe": name, "input": file.display().to_string(), "next": next}),
        );
    } else {
        // the next commands as a numbered list a person can follow
        println!("probe: {name}\ninput: {}\nnext:", file.display());
        for (i, c) in next.iter().enumerate() {
            println!("  {}. {c}", i + 1);
        }
    }
    Ok(())
}

/// `write approve-demo <package>`: the owner's standing approval for the Demo Company, recorded for this
/// package's hash (`via: demo_standing`, the words from `standing-approval.txt` beside the config). Agents
/// may run it. Refused (`not_a_demo`, exit 2) unless the profile's `require_tenant_name` names a Demo
/// Company AND every org of the package is connected under that name in the token file (no network).
pub fn approve_demo_cmd(app: &App, pkg: &Path) -> Res<()> {
    use ownpurse_core::write::{
        AllOps, ApprovalRecord, ApprovalScope, ApprovalStore, STANDING_APPROVAL_FILE, Via,
    };
    let not_demo = |why: String| {
        CliError::new("not_a_demo", exit::USAGE, why)
            .hint("The standing approval exists only for the Demo Company's own profile.")
    };
    let p = &app.config.profile;
    let Some(want) = p
        .require_tenant_name
        .clone()
        .filter(|n| n.contains("Demo Company"))
    else {
        return Err(not_demo(format!(
            "Profile '{}' does not require a Demo Company tenant.",
            p.name
        )));
    };
    let pkg_v = load(pkg)?;
    let hash = seal::verify(&pkg_v)?;
    if pkg_v.profile != p.name {
        return Err(not_demo(format!(
            "The package was planned for profile '{}', not '{}'.",
            pkg_v.profile, p.name
        )));
    }
    // the connected tenants as the last login/orgs stored them (names only)
    let tok = tokens(app).load()?.unwrap_or_default();
    let connected: Vec<(String, String)> = tok
        .get("tenants")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|t| {
            Some((
                t["tenantId"].as_str()?.to_string(),
                t["tenantName"].as_str()?.to_string(),
            ))
        })
        .collect();
    for o in &pkg_v.ops {
        match connected.iter().find(|(id, _)| *id == o.tenant_id) {
            Some((_, name)) if *name == want => {}
            Some((_, name)) => {
                return Err(not_demo(format!(
                    "{} is connected as '{name}', not '{want}'.",
                    o.org
                )));
            }
            None => {
                return Err(not_demo(format!(
                    "{} is not among the token's connected tenants (run `ownpurse orgs`).",
                    o.org
                )));
            }
        }
    }
    let dir = app
        .config
        .path
        .as_ref()
        .and_then(|c| c.parent())
        .map(Path::to_path_buf)
        .ok_or_else(|| not_demo("No config file: no standing approval.".into()))?;
    let file = dir.join(STANDING_APPROVAL_FILE);
    let words = std::fs::read_to_string(&file)
        .map_err(|e| {
            CliError::new(
                "not_approved",
                exit::NOT_APPROVED,
                format!("No standing approval ({STANDING_APPROVAL_FILE}: {e})."),
            )
            .hint("The owner writes his standing approval for the Demo Company into that file.")
        })?
        .trim()
        .to_string();
    if words.is_empty() {
        return Err(CliError::new(
            "not_approved",
            exit::NOT_APPROVED,
            format!("{STANDING_APPROVAL_FILE} is empty."),
        ));
    }
    let now = ownpurse_xero::store::now_iso();
    let a = ApprovalRecord {
        hash: hash.clone(),
        verbatim: words.clone(),
        via: Via::DemoStanding,
        where_: format!(
            "{STANDING_APPROVAL_FILE} sha256:{}",
            &ownpurse_core::write::sha256_hex(words.as_bytes())[..12]
        ),
        at: now.clone(),
        scope: ApprovalScope::All(AllOps::All),
        recorded_at: now,
    };
    let store = Writer::open(app.config.store())?;
    let id = RecordApprovals { store: &store }.record(&a)?;
    emit(
        app,
        json!({"approval": id, "hash": hash, "via": "demo_standing", "source": Via::DemoStanding.label(),
               "where": a.where_, "scope": "all"}),
    );
    Ok(())
}
