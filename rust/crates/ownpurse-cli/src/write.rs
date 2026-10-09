//! `ownpurse write …`, the package side (Phase 8): plan, seal, export, and the owner's approval (record at a
//! TTY, import from his chat transcript, check). None of these calls Xero: plan reads the local record, the
//! approvals go into the record's append-only `approvals` table. The run side (run, verify, reconcile,
//! recover, status, probe) is in `write_run`.

use crate::app::{App, CliError, Res};
use ownpurse_core::write::approval::{self, TtyApproval};
use ownpurse_core::write::plan::{self, PlanConfig};
use ownpurse_core::write::{Package, WriteError, export, seal};
use ownpurse_xero::store::Writer;
use ownpurse_xero::write::RecordApprovals;
use std::io::{BufRead, IsTerminal, Write};
use std::path::{Path, PathBuf};

impl From<WriteError> for CliError {
    fn from(e: WriteError) -> Self {
        let mut c = CliError::new(&e.code, e.exit, e.message);
        for h in e.help {
            c = c.hint(h);
        }
        c
    }
}

fn now() -> String {
    ownpurse_core::classify::store::now_iso()
}

fn gates(app: &App) -> Vec<(String, String)> {
    ownpurse_core::manifest::accountant_gate(&app.config)
}

/// `<dir>/<stem>.package.json` beside the input, unless `--out`.
fn package_path(input: &Path, out: Option<&Path>) -> PathBuf {
    if let Some(o) = out {
        return o.to_path_buf();
    }
    let stem = input
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("ops")
        .trim_end_matches(".ops");
    input.with_file_name(format!("{stem}.package.json"))
}

fn summary(app: &App, p: &Package, out: &mut crate::out::Out) {
    let rows = p
        .ops
        .iter()
        .map(|o| {
            let mut m = serde_json::Map::new();
            m.insert("op".into(), o.id.clone().into());
            m.insert("kind".into(), o.op.kind().into());
            m.insert("org".into(), o.org.clone().into());
            m.insert("what".into(), export::summary(o).into());
            m.insert(
                "gate".into(),
                if o.gated.is_some() { "accountant" } else { "" }.into(),
            );
            m.insert("flags".into(), o.flags.join("; ").into());
            m
        })
        .collect::<Vec<_>>();
    let n = rows.len();
    out.table(
        "ops",
        &["op", "kind", "org", "what", "gate", "flags"],
        app.limit(&rows),
        Some(n),
    );
}

/// `write plan <ops.json> [--out <package.json>]`: compile against the local record; writes an unsealed
/// package.
pub fn plan_cmd(app: &mut App, input: &Path, out_path: Option<&Path>) -> Res<()> {
    let inp = plan::load_input(input)?;
    let cfg = PlanConfig {
        profile: app.config.profile.name.clone(),
        orgs: app.config.profile.orgs.clone(),
        gates: gates(app),
        history_notes: app.config.profile.history_notes(),
    };
    let base = input
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."));
    let base = if base.as_os_str().is_empty() {
        PathBuf::from(".")
    } else {
        base
    };
    let now = now();
    let p = plan::plan(app.record()?, &cfg, &inp, &base, &now)?;
    let path = package_path(input, out_path);
    seal::save(&path, &p)?;
    let mut out = app.out();
    out.field("package", path.display().to_string())
        .field("id", p.id.as_str())
        .field("sealed", false)
        .field("ops", p.ops.len());
    if let Some(a) = &p.accountant_decision {
        out.field("accountant_decision", a.as_str());
    }
    summary(app, &p, &mut out);
    out.hint(format!(
        "Review it, then `ownpurse write seal {}` (prints the hash and writes the PDF and CSV for the accountant).",
        path.display()
    ));
    out.emit();
    Ok(())
}

/// The export files beside the package: `<package stem>-<hash 12>.pdf|csv`.
fn export_paths(pkg: &Path, hash: &str, dir: Option<&Path>) -> (PathBuf, PathBuf) {
    let stem = pkg
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("package")
        .trim_end_matches(".package");
    let dir = dir
        .map(Path::to_path_buf)
        .or_else(|| pkg.parent().map(Path::to_path_buf))
        .unwrap_or_else(|| PathBuf::from("."));
    let base = format!("{stem}-{}", &hash[..12]);
    (
        dir.join(format!("{base}.pdf")),
        dir.join(format!("{base}.csv")),
    )
}

/// Each tenant's ShortCode for the review's "open in Xero" links (none without a record).
fn links_of(app: &mut App, p: &Package) -> ownpurse_core::write::Links {
    if !app.config.store().exists() {
        return Default::default();
    }
    app.record()
        .map(|r| ownpurse_core::write::links(r, p))
        .unwrap_or_default()
}

fn write_exports(
    p: &Package,
    pkg: &Path,
    dir: Option<&Path>,
    links: &ownpurse_core::write::Links,
) -> Res<(PathBuf, PathBuf)> {
    let hash = seal::verify(p)?;
    let (pdf, csv) = export_paths(pkg, &hash, dir);
    let io = |path: &Path, e: std::io::Error| {
        CliError::runtime("io", format!("{}: {e}", path.display()))
    };
    std::fs::write(&pdf, export::pdf_with(p, links)?).map_err(|e| io(&pdf, e))?;
    std::fs::write(&csv, export::csv_with(p, links)?).map_err(|e| io(&csv, e))?;
    Ok((pdf, csv))
}

/// `write seal <package>`: seal in place (a changed package gets a new hash), write the review export, and
/// print the review.
/// Seal a package file in place, register it in the record (when there is one) and write its review
/// export. Returns the sealed package, its hash and the export paths.
pub fn seal_file(
    app: &mut App,
    pkg: &Path,
    dir: Option<&Path>,
) -> Res<(Package, String, PathBuf, PathBuf)> {
    let p = seal::load(pkg)?;
    let p = seal::seal(p)?;
    let hash = p.seal.clone().unwrap_or_default();
    seal::save(pkg, &p)?;
    // the record knows the package from now on (the TUI lists it); no record yet: the run registers it
    if app.config.store().exists() {
        let w = writer(app)?;
        ownpurse_xero::write::log::register_package(&w, &p, &hash)?;
    }
    let links = links_of(app, &p);
    let (pdf, csv) = write_exports(&p, pkg, dir, &links)?;
    Ok((p, hash, pdf, csv))
}

pub fn seal_cmd(app: &mut App, pkg: &Path, dir: Option<&Path>) -> Res<()> {
    let before = seal::load(pkg)?.seal;
    let (p, hash, pdf, csv) = seal_file(app, pkg, dir)?;
    let links = links_of(app, &p);
    let mut out = app.out();
    out.field("package", pkg.display().to_string())
        .field("hash", hash.as_str())
        .field("changed", before.as_deref().is_some_and(|b| b != hash))
        .field("pdf", pdf.display().to_string())
        .field("csv", csv.display().to_string());
    let review = export::lines_with(&p, &links)?;
    if app.g.json {
        // one string in JSON; text mode prints it after the fields, as the lines a person reads
        out.field("review", review.join("\n"));
    }
    summary(app, &p, &mut out);
    out.hint("Nothing was sent to Xero. Send the PDF and CSV to the accountant and the owner; the owner approves by quoting the full hash.");
    out.hint(format!(
        "The owner records it at his own terminal: `ownpurse write approve-record {} --response-file <file> --i-am-the-owner`.",
        pkg.display()
    ));
    out.emit();
    if !app.g.json {
        println!("\nreview:");
        for l in &review {
            println!("  {l}");
        }
    }
    Ok(())
}

/// `write export <package> [--dir <folder>]`: re-write the PDF and CSV of a sealed package.
pub fn export_cmd(app: &mut App, pkg: &Path, dir: Option<&Path>) -> Res<()> {
    let p = seal::load(pkg)?;
    let links = links_of(app, &p);
    let (pdf, csv) = write_exports(&p, pkg, dir, &links)?;
    let mut out = app.out();
    out.field("hash", p.seal.clone().unwrap_or_default())
        .field("pdf", pdf.display().to_string())
        .field("csv", csv.display().to_string());
    out.emit();
    Ok(())
}

fn writer(app: &App) -> Res<Writer> {
    let path = app.config.store();
    if !path.exists() {
        return Err(CliError::usage("The local record does not exist yet.")
            .hint("Run `ownpurse sync all`."));
    }
    Writer::open(path).map_err(|e| CliError::runtime("record", e.to_string()))
}

fn scope_list(scope: Option<&str>) -> Vec<String> {
    scope
        .map(|s| {
            s.split(',')
                .map(str::trim)
                .filter(|x| !x.is_empty())
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default()
}

fn env_names() -> Vec<String> {
    std::env::vars_os()
        .filter_map(|(k, _)| k.into_string().ok())
        .collect()
}

fn recorded(app: &App, a: &ownpurse_core::write::ApprovalRecord, id: i64) {
    let mut out = app.out();
    out.field("approval", id)
        .field("hash", a.hash.as_str())
        .field("via", a.via.as_str())
        .field("where", a.where_.as_str())
        .field("at", a.at.as_str())
        .field(
            "scope",
            serde_json::to_value(&a.scope).unwrap_or(serde_json::Value::Null),
        )
        .field("verbatim", a.verbatim.as_str());
    out.hint("Recorded in the record's append-only approvals table. `ownpurse write approve-check <package>` confirms it.");
    out.emit();
}

/// `write approve-record <package> --response-file <file> --i-am-the-owner [--scope op-1,op-2]`: the owner,
/// at his own terminal. Refuses agent sessions (agent markers in the environment, or no TTY).
pub fn approve_record(
    app: &mut App,
    pkg: &Path,
    response_file: &Path,
    i_am_owner: bool,
    scope: Option<&str>,
) -> Res<()> {
    let tty = std::io::stdin().is_terminal() && std::io::stdout().is_terminal();
    let names = env_names();
    // refuse before reading anything else or prompting
    approval::refuse_agent(tty, &names)?;
    let p = seal::load(pkg)?;
    let hash = seal::verify(&p)?;
    let verbatim = std::fs::read_to_string(response_file)
        .map_err(|e| CliError::runtime("io", format!("{}: {e}", response_file.display())))?;
    print!(
        "Package {} ({} operations)\nsha256 {hash}\nType the hash (at least its first 12 characters) to record your approval: ",
        p.id,
        p.ops.len()
    );
    let _ = std::io::stdout().flush();
    let mut typed = String::new();
    std::io::stdin()
        .lock()
        .read_line(&mut typed)
        .map_err(|e| CliError::runtime("io", e.to_string()))?;
    let a = approval::record_tty(&TtyApproval {
        package: &p,
        verbatim: verbatim.trim_end(),
        typed: &typed,
        i_am_owner,
        tty,
        env_names: names,
        scope: scope_list(scope),
        now: now(),
    })?;
    let w = writer(app)?;
    let id = ownpurse_core::write::ApprovalStore::record(&mut RecordApprovals { store: &w }, &a)?;
    recorded(app, &a, id);
    Ok(())
}

/// `write approve-import <package> --transcript <session.jsonl> --session <id> [--scope …]`: the owner's
/// typed chat message quoting the full hash, from his own session transcript.
pub fn approve_import(
    app: &mut App,
    pkg: &Path,
    transcript: &Path,
    session: &str,
    scope: Option<&str>,
) -> Res<()> {
    let p = seal::load(pkg)?;
    let a = approval::record_chat(&p, transcript, session, &scope_list(scope), &now())?;
    let w = writer(app)?;
    let id = ownpurse_core::write::ApprovalStore::record(&mut RecordApprovals { store: &w }, &a)?;
    recorded(app, &a, id);
    Ok(())
}

/// `write approve-check <package>`: exit 0 with the approvals for its (recomputed) hash, else 3.
pub fn approve_check(app: &mut App, pkg: &Path) -> Res<()> {
    let p = seal::load(pkg)?;
    let w = writer(app)?;
    let found = approval::check(&RecordApprovals { store: &w }, &p)?;
    let uncovered: Vec<String> = p
        .ops
        .iter()
        .filter(|o| !approval::covers(&found, &o.id))
        .map(|o| o.id.clone())
        .collect();
    let rows = found
        .iter()
        .map(|a| {
            let mut m = serde_json::Map::new();
            m.insert(
                "via".into(),
                serde_json::to_value(a.via).unwrap_or_default(),
            );
            m.insert("source".into(), a.via.label().into());
            m.insert("at".into(), a.at.clone().into());
            m.insert(
                "scope".into(),
                serde_json::to_value(&a.scope).unwrap_or_default(),
            );
            m.insert("where".into(), a.where_.clone().into());
            m
        })
        .collect::<Vec<_>>();
    let mut out = app.out();
    out.field("hash", p.seal.clone().unwrap_or_default())
        .field("approved", true)
        .field("uncovered_ops", uncovered.join(", "));
    out.table("approvals", &["via", "at", "scope", "where"], rows, None);
    out.emit();
    Ok(())
}
