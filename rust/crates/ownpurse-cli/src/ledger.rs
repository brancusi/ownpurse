//! Ledger commands: `gl`, `tb`, `parity`, record-only: Xero trial
//! balances come from cached report snapshots, never from the network.

use crate::app::{App, CliError, Res};
use crate::cmds::{Row, cents_zero};
use crate::obj;
use crate::out::{dec, opt_dec};
use ownpurse_core::balances::{Balances, Basis, BasisKind, Mode, compare, trial_balance};
use ownpurse_core::dates::fiscal_year_start;
use ownpurse_core::record::tb_params;
use ownpurse_core::{Chart, Ledger, resolve, xero_tb_balances};
use rust_decimal::Decimal;
use serde_json::Value;
use std::collections::{BTreeSet, HashSet};

fn ledger_for(app: &mut App, tid: &str) -> Res<Ledger> {
    app.ensure_ledger(tid)?;
    let v = app.view()?;
    Ok(Ledger::build(&v, tid)?)
}

/// One line describing the basis.
pub fn describe(b: &Basis) -> String {
    let mut s = if b.kind == BasisKind::XeroOpening {
        format!(
            "Opening from Xero TB {} + {} activity",
            b.opening_date.as_deref().unwrap_or("None"),
            b.as_of.chars().take(4).collect::<String>()
        )
    } else {
        "Local full-history balances".to_string()
    };
    match b.matches_xero() {
        Some(true) => s.push_str(" · matches Xero ✓"),
        Some(false) => s.push_str(&format!(" · {} accounts differ from Xero", b.diffs.len())),
        None => {}
    }
    s
}

/// A cached Xero TrialBalance: never fetches.
fn cached_tb(app: &mut App, tid: &str, date: &str) -> Res<(Value, String)> {
    let params = tb_params(date);
    match app.record()?.report(tid, "TrialBalance", &params)? {
        Some(r) => Ok((r.payload, r.fetched_at)),
        None => Err(
            CliError::usage(format!("No cached TrialBalance snapshot for {params}."))
                .hint("Drop --offline/--cached."),
        ),
    }
}

fn fy_start_for(app: &mut App, tid: &str, as_of: &str) -> Res<String> {
    let (m, d) = app.record()?.fy_end(tid)?;
    fiscal_year_start(as_of, m, d).ok_or_else(|| CliError::internal(format!("not a date: {as_of}")))
}

/// Local balances at `as_of`, optionally opening from Xero's TB at `since`.
fn local_tb(app: &mut App, tid: &str, as_of: &str, since: Option<&str>) -> Res<(Ledger, Balances)> {
    let lb = ledger_for(app, tid)?;
    let fy_start = fy_start_for(app, tid, as_of)?;
    let opening = match since {
        Some(s) => Some(xero_tb_balances(&cached_tb(app, tid, s)?.0)?.0),
        None => None,
    };
    let bal = trial_balance(
        &lb.postings,
        &lb.chart,
        as_of,
        &fy_start,
        opening.as_ref(),
        since,
    );
    Ok((lb, bal))
}

fn account_matcher(chart: &Chart, text: Option<&str>) -> Option<HashSet<String>> {
    let t = text?.to_lowercase();
    let exact: HashSet<String> = chart
        .by_id
        .values()
        .filter(|a| {
            t == a.code.to_lowercase()
                || t == a.account_id.to_lowercase()
                || t == a.name.to_lowercase()
        })
        .map(|a| a.account_id.clone())
        .collect();
    if !exact.is_empty() {
        return Some(exact);
    }
    Some(
        chart
            .by_id
            .values()
            .filter(|a| a.label().to_lowercase().contains(&t))
            .map(|a| a.account_id.clone())
            .collect(),
    )
}

pub fn today_utc() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let (y, m, d) = ownpurse_core::dates::civil_from_days((secs / 86_400) as i64);
    ownpurse_core::dates::iso(y, m, d)
}

pub struct GlArgs {
    pub org: String,
    pub since: Option<String>,
    pub until: Option<String>,
    pub account: Option<String>,
    pub full_history: bool,
    pub source: Option<String>,
    pub search: Option<String>,
}

pub fn gl(app: &mut App, a: &GlArgs) -> Res<()> {
    let tids = app.resolve(&a.org)?;
    let mut rows: Vec<Row> = Vec::new();
    let mut debit = cents_zero();
    let mut credit = cents_zero();
    let mut single: Option<String> = None;
    let mut basis_note = String::new();
    let mut opening = cents_zero();
    let mut since_out: Option<String> = None;
    let mut running = cents_zero();
    for tid in &tids {
        let lb = ledger_for(app, tid)?;
        let matched = account_matcher(&lb.chart, a.account.as_deref());
        if let (Some(acc), Some(m)) = (&a.account, &matched)
            && m.is_empty()
        {
            return Err(CliError::usage(format!("No account matches '{acc}'."))
                .hint(format!("Run `ownpurse accounts {}`.", a.org)));
        }
        if let Some(m) = &matched
            && m.len() == 1
            && tids.len() == 1
        {
            single = m.iter().next().cloned();
        }
        running = cents_zero();
        let mut since = a.since.clone();
        if let Some(acct) = &single {
            let as_of = a.until.clone().unwrap_or_else(today_utc);
            let fy = app.record()?.fy_end(tid)?;
            let mode = if a.full_history {
                Mode::History
            } else {
                Mode::Auto
            };
            let b = resolve(app.record()?, &lb, tid, &as_of, fy, mode)?;
            basis_note = if b.kind == BasisKind::XeroOpening {
                describe(&b)
            } else {
                "full local history (may differ from Xero where setup balances exist)".into()
            };
            if b.kind == BasisKind::XeroOpening && since.is_none() {
                since = Some(b.fy_start.clone());
            }
            running = match &since {
                Some(s) => b.opening_for(&lb, acct, s),
                None => cents_zero(),
            };
            opening = running;
        }
        since_out = since.clone();
        for p in &lb.postings {
            if let Some(m) = &matched
                && !m.contains(&p.account_id)
            {
                continue;
            }
            if let Some(s) = &a.source
                && p.source.to_lowercase() != s.to_lowercase()
            {
                continue;
            }
            if a.until.as_deref().is_some_and(|u| p.date.as_str() > u) {
                continue;
            }
            if since.as_deref().is_some_and(|s| p.date.as_str() < s) {
                continue;
            }
            if let Some(s) = &a.search {
                let hay = format!("{} {} {}", p.description, p.contact, p.reference).to_lowercase();
                if !hay.contains(&s.to_lowercase()) {
                    continue;
                }
            }
            let acct = lb.chart.get(&p.account_id);
            running += p.amount;
            if p.amount > Decimal::ZERO {
                debit += p.amount;
            }
            if p.amount < Decimal::ZERO {
                credit += -p.amount;
            }
            rows.push(obj![
                ("org", app.alias_of(tid)),
                ("date", p.date.as_str()),
                ("account", acct.label()),
                (
                    "debit",
                    opt_dec((p.amount > Decimal::ZERO).then_some(p.amount))
                ),
                (
                    "credit",
                    opt_dec((p.amount < Decimal::ZERO).then_some(-p.amount))
                ),
                ("balance", dec(running)),
                ("source", p.source),
                ("contact", p.contact.as_str()),
                (
                    "description",
                    p.description.chars().take(60).collect::<String>()
                ),
                ("reference", p.reference.as_str()),
                ("source_id", p.source_id.as_str()),
            ]);
        }
    }
    let mut cols = vec!["org", "date", "account", "debit", "credit"];
    if single.is_some() {
        cols.push("balance");
    }
    cols.extend(["source", "contact", "description", "source_id"]);
    let shown = app.limit(&rows);
    let mut out = app.out();
    crate::manifest::view_note(&mut out, app);
    out.field("debits", dec(debit))
        .field("credits", dec(credit))
        .field("net", dec(debit - credit));
    if single.is_some() {
        out.field("from", since_out)
            .field("balance_basis", basis_note)
            .field("opening_balance", dec(opening))
            .field("closing_balance", dec(running));
    }
    let n = shown.len();
    out.table("postings", &cols, shown, Some(rows.len()));
    if n < rows.len() {
        out.hint(format!(
            "Showing {n} of {}; add --full (or --limit N) to see more, --json for everything.",
            rows.len()
        ));
    }
    out.hint(
        "Debits positive / credits negative internally; `ownpurse show <org> <source> <source_id>` \
         opens the source document.",
    );
    out.emit();
    Ok(())
}

pub struct TbArgs {
    pub org: String,
    pub date: String,
    pub since: Option<String>,
    pub full_history: bool,
}

pub fn tb(app: &mut App, a: &TbArgs) -> Res<()> {
    let tids = app.resolve(&a.org)?;
    let mut rows: Vec<Row> = Vec::new();
    let mut notes: Vec<String> = Vec::new();
    let mut debit = cents_zero();
    let mut credit = cents_zero();
    for tid in &tids {
        let alias = app.alias_of(tid);
        let (lb, bal, differing, note): (Ledger, Balances, BTreeSet<String>, String);
        if let Some(since) = &a.since {
            let (l, b) = local_tb(app, tid, &a.date, Some(since))?;
            let mut diff = BTreeSet::new();
            if let Some(cur) = app.record()?.tb_snapshot(tid, &a.date)? {
                let xb = xero_tb_balances(&cur.payload)?.0;
                diff = compare(&b, &xb).into_iter().map(|d| d.account_id).collect();
            }
            (lb, bal, differing, note) = (
                l,
                b,
                diff,
                format!("opening from Xero TB {since} + later activity"),
            );
        } else {
            let l = ledger_for(app, tid)?;
            let fy = app.record()?.fy_end(tid)?;
            let mode = if a.full_history {
                Mode::History
            } else {
                Mode::Auto
            };
            let b = resolve(app.record()?, &l, tid, &a.date, fy, mode)?;
            let diff: BTreeSet<String> = b.diffs.iter().map(|d| d.account_id.clone()).collect();
            let n = if b.kind == BasisKind::XeroOpening {
                describe(&b)
            } else {
                "full local history".to_string()
            };
            if b.kind == BasisKind::History && b.matches_xero() != Some(true) {
                notes.push(format!(
                    "{alias}: full local history {} — run `ownpurse parity {alias} --date {} --since <prior year-end>` to cache Xero's TBs.",
                    if b.diffs.is_empty() {
                        "not compared (no cached Xero TB)".to_string()
                    } else {
                        format!("differs from Xero in {} accounts (flag !)", b.diffs.len())
                    },
                    a.date
                ));
            }
            (lb, bal, differing, note) = (l, b.balances, diff, n);
        }
        if !differing.is_empty() && a.since.is_some() {
            notes.push(format!(
                "{alias}: {} accounts differ from Xero (flag !).",
                differing.len()
            ));
        }
        // Xero-only accounts appear at zero, flagged.
        let mut shown: Vec<(String, Decimal)> = differing
            .iter()
            .filter(|k| !bal.contains_key(*k))
            .map(|k| (k.clone(), Decimal::ZERO))
            .collect();
        shown.extend(bal.iter().map(|(k, v)| (k.clone(), *v)));
        shown.sort_by_cached_key(|(k, _)| {
            let acct = lb.chart.get(k);
            (acct.class, acct.code, acct.name)
        });
        for (acct_id, amount) in shown {
            let acct = lb.chart.get(&acct_id);
            let (dr, cr) = if amount.is_zero() {
                (None, None)
            } else if amount > Decimal::ZERO {
                (Some(amount), None)
            } else {
                (None, Some(-amount))
            };
            debit += dr.unwrap_or_default();
            credit += cr.unwrap_or_default();
            rows.push(obj![
                ("org", alias.as_str()),
                (
                    "flag",
                    if differing.contains(&acct_id) {
                        "!"
                    } else {
                        ""
                    }
                ),
                ("account", acct.label()),
                ("class", acct.class.as_str()),
                ("debit", opt_dec(dr)),
                ("credit", opt_dec(cr)),
                ("account_id", acct_id.as_str()),
            ]);
        }
        notes.insert(0, format!("{alias}: {note}"));
    }
    let n_orgs = tids.len();
    let mut out = app.out();
    crate::manifest::view_note(&mut out, app);
    out.field("as_of", a.date.as_str())
        .field("accounting_basis", "accrual");
    if notes.len() > 1 {
        out.field(
            "balance_basis",
            Value::Array(
                notes
                    .iter()
                    .take(n_orgs)
                    .cloned()
                    .map(Value::String)
                    .collect(),
            ),
        );
    } else {
        out.field("balance_basis", notes[0].as_str());
    }
    out.field("total_debit", dec(debit))
        .field("total_credit", dec(credit))
        .field("balanced", debit == credit);
    let shown = app.limit(&rows);
    let mut cols = vec!["org", "account", "class", "debit", "credit"];
    if rows
        .iter()
        .any(|r| r.get("flag").and_then(Value::as_str) == Some("!"))
    {
        cols.insert(1, "flag");
    }
    if app.g.full {
        cols.push("account_id");
    }
    let n = shown.len();
    out.table("trial_balance", &cols, shown, Some(rows.len()));
    if n < rows.len() {
        out.hint(format!(
            "Showing {n} of {}; add --full (or --limit N) to see more, --json for everything.",
            rows.len()
        ));
    }
    for extra in notes.iter().skip(n_orgs) {
        out.hint(extra.clone());
    }
    out.hint(
        "`--full-history` shows balances from local documents only; `ownpurse parity` checks against Xero.",
    );
    out.emit();
    Ok(())
}

pub struct ParityArgs {
    pub org: String,
    pub date: String,
    pub since: Option<String>,
    pub strict: bool,
}

pub fn parity(app: &mut App, a: &ParityArgs) -> Res<()> {
    let tids = app.resolve(&a.org)?;
    let mut summary: Vec<Row> = Vec::new();
    let mut diffs_out: Vec<Row> = Vec::new();
    for tid in &tids {
        let (lb, local) = local_tb(app, tid, &a.date, a.since.as_deref())?;
        let (payload, fetched_at) = cached_tb(app, tid, &a.date)?;
        let (xero, labels) = xero_tb_balances(&payload)?;
        let diffs = compare(&local, &xero);
        let uncoded = lb
            .postings
            .iter()
            .filter(|p| p.account_id.starts_with("(uncoded)"))
            .count();
        let accounts: BTreeSet<&String> = local.keys().chain(xero.keys()).collect();
        let net: Decimal = diffs.iter().fold(cents_zero(), |acc, d| acc + d.difference);
        let alias = app.alias_of(tid);
        summary.push(obj![
            ("org", alias.as_str()),
            ("match", diffs.is_empty()),
            ("accounts", accounts.len()),
            ("differences", diffs.len()),
            ("net_difference", dec(net)),
            ("postings", lb.postings.len()),
            ("unbalanced_docs", lb.unbalanced.len()),
            ("uncoded_postings", uncoded),
            ("xero_tb_fetched", fetched_at),
        ]);
        for d in &diffs {
            let label = labels
                .get(&d.account_id)
                .filter(|l| !l.is_empty())
                .cloned()
                .unwrap_or_else(|| lb.chart.get(&d.account_id).label());
            diffs_out.push(obj![
                ("org", alias.as_str()),
                ("account", label),
                ("local", dec(d.local)),
                ("xero", dec(d.xero)),
                ("difference", dec(d.difference)),
                ("account_id", d.account_id.as_str()),
            ]);
        }
    }
    let all_match = summary
        .iter()
        .all(|s| s.get("match").and_then(Value::as_bool) == Some(true));
    let mut out = app.out();
    crate::manifest::view_note(&mut out, app);
    out.field("as_of", a.date.as_str())
        .field("match", all_match);
    if let Some(s) = &a.since {
        out.field("opening_from_xero_tb", s.as_str());
    }
    out.table(
        "parity",
        &[
            "org",
            "match",
            "accounts",
            "differences",
            "net_difference",
            "postings",
            "unbalanced_docs",
            "uncoded_postings",
        ],
        summary,
        None,
    );
    let shown = app.limit(&diffs_out);
    out.table(
        "differences",
        &["org", "account", "local", "xero", "difference"],
        shown,
        Some(diffs_out.len()),
    );
    if !diffs_out.is_empty() && a.since.is_none() {
        out.hint(
            "Differences without --since often come from opening/conversion balances: rerun with \
             --since <prior year-end> to compare only the activity in between.",
        );
    }
    if !diffs_out.is_empty() {
        out.hint(
            "Drill in with `ownpurse gl <org> --account <code|name> --from <date> --to <date>`.",
        );
    }
    out.emit();
    if a.strict && !diffs_out.is_empty() {
        return Err(CliError::runtime(
            "parity_mismatch",
            format!(
                "{} account(s) differ from Xero's trial balance.",
                diffs_out.len()
            ),
        ));
    }
    Ok(())
}

pub struct CompareArgs {
    pub org: String,
    pub date: String,
    pub full_history: bool,
    pub align: Option<String>,
}

/// `tb a,b --compare`: 2–4 orgs side by side; rows merge only by code or
/// account_map category, nothing is added or differenced across orgs.
pub fn tb_compare(app: &mut App, a: &CompareArgs) -> Res<()> {
    use ownpurse_core::compare::{self as cmpm, RowStyle};
    use std::collections::HashMap;
    let tids = app.resolve(&a.org)?;
    let aliases: Vec<String> = tids.iter().map(|t| app.alias_of(t)).collect();
    if !(2..=4).contains(&tids.len()) {
        return Err(CliError::usage(format!(
            "--compare needs 2–4 orgs; '{}' names {}.",
            a.org,
            tids.len()
        ))
        .hint("Pass a comma list or a group, e.g. `ownpurse tb roast,design --compare`."));
    }
    let mut orgs_payload: HashMap<String, Value> = HashMap::new();
    for (tid, alias) in tids.iter().zip(&aliases) {
        orgs_payload
            .entry(alias.clone())
            .or_insert(app.record()?.organisation(tid)?);
    }
    if let Err(e) = cmpm::check_compatible(&aliases, |o| {
        orgs_payload.get(o).cloned().unwrap_or(Value::Null)
    }) {
        return Err(
            CliError::usage(e.0).hint("Compare orgs with the same base currency and fiscal year.")
        );
    }
    let mut ledgers: Vec<Ledger> = Vec::new();
    let mut bases: Vec<Basis> = Vec::new();
    for tid in &tids {
        let lb = ledger_for(app, tid)?;
        let fy = app.record()?.fy_end(tid)?;
        let mode = if a.full_history {
            Mode::History
        } else {
            Mode::Auto
        };
        bases.push(resolve(app.record()?, &lb, tid, &a.date, fy, mode)?);
        ledgers.push(lb);
    }
    let mut tbs = HashMap::new();
    let mut charts = HashMap::new();
    for ((alias, lb), b) in aliases.iter().zip(&ledgers).zip(&bases) {
        tbs.insert(alias.clone(), cmpm::tb_rows(lb, b, RowStyle::Cli));
        charts.insert(alias.clone(), &lb.chart);
    }
    let st = &app.config.settings;
    let align = a
        .align
        .clone()
        .or_else(|| {
            st.get_str("tui.compare_default_align")
                .filter(|s| !s.is_empty())
                .map(str::to_string)
        })
        .unwrap_or_else(|| "code".into());
    let account_map = st
        .get("account_map")
        .filter(|v| crate::cmds::truthy(v))
        .cloned()
        .unwrap_or_else(|| Value::Object(Default::default()));
    let res = cmpm::tb_compare(&aliases, &tbs, &charts, &align, &account_map)
        .map_err(CliError::internal)?;
    let rows = cmpm::flatten(&res).map_err(CliError::internal)?;
    let columns: Vec<Value> = res["columns"].as_array().cloned().unwrap_or_default();
    let text = |v: &Value| crate::out::py_str(v);
    let mut out = app.out();
    crate::manifest::view_note(&mut out, app);
    out.field("as_of", a.date.as_str())
        .field("accounting_basis", "accrual")
        .field("align", align.as_str());
    if app.g.json {
        out.field(
            "columns",
            Value::Array(
                columns
                    .iter()
                    .map(|c| {
                        Value::Object(obj![
                            ("org", c["org"].clone()),
                            (
                                "basis",
                                format!(
                                    "{} {}",
                                    text(&c["basis"]["source"]),
                                    text(&c["basis"]["state"])
                                )
                            ),
                            ("total_debit", c["debit"].clone()),
                            ("total_credit", c["credit"].clone()),
                            ("balanced", c["balanced"].clone()),
                        ])
                    })
                    .collect(),
            ),
        );
        let rows: Vec<Row> = rows
            .into_iter()
            .map(|r| r.as_object().cloned().unwrap_or_default())
            .collect();
        out.table(
            "rows",
            &["section", "code", "label", "class", "cells", "members"],
            rows,
            None,
        );
    } else {
        out.field(
            "columns",
            Value::Array(
                columns
                    .iter()
                    .map(|c| {
                        Value::String(format!(
                            "{}: {} {} · debit {} credit {}",
                            text(&c["org"]),
                            text(&c["basis"]["source"]),
                            text(&c["basis"]["state"]),
                            text(&c["debit"]),
                            text(&c["credit"])
                        ))
                    })
                    .collect(),
            ),
        );
        let mut cols: Vec<String> = vec!["section".into(), "code".into(), "account".into()];
        cols.extend(aliases.iter().cloned());
        let flat: Vec<Row> = rows
            .iter()
            .map(|r| {
                let mut d = obj![
                    ("section", r["section"].clone()),
                    ("code", r["code"].clone()),
                    ("account", r["label"].clone()),
                ];
                for o in &aliases {
                    let c = &r["cells"][o.as_str()];
                    let v = if c["exists"] != Value::Bool(true) {
                        "—".to_string()
                    } else {
                        let net = ownpurse_core::money::parse(&text(&c["net"])).unwrap_or_default();
                        if net.is_zero() {
                            String::new()
                        } else {
                            text(&c["net"])
                        }
                    };
                    d.insert(o.clone(), Value::String(v));
                }
                d
            })
            .collect();
        let total = flat.len();
        out.table_cols("rows", cols, app.limit(&flat), Some(total));
        out.hint(
            "Cells are each org's own net balance (debit positive); — = the account does not exist there. \
             Rows merge only on an identical code (or an account_map category with --align map); nothing \
             is added or differenced across orgs.",
        );
    }
    out.emit();
    Ok(())
}

/// One CSV cell (quoted when it holds a comma, quote or newline).
fn csv_cell(s: &str) -> String {
    if s.contains([',', '"', '\n', '\r']) {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

fn money_cell(d: Decimal) -> String {
    ownpurse_core::write::money_text(d)
}

/// `export <org> tb|gl|bs|pl [--date | --from --to] --csv [--out FILE]`: a file for the accountant, from the
/// record (zero calls). `tb` and `gl` come from the local ledger (the same numbers as `tb` / `gl`); `bs` and
/// `pl` are Xero's own reports as `report fetch` cached them (fetch first). Never over an existing file.
#[allow(clippy::too_many_arguments)]
pub fn export(
    app: &mut App,
    org: &str,
    kind: &str,
    date: Option<String>,
    from: Option<String>,
    to: Option<String>,
    xlsx: bool,
    out_path: Option<std::path::PathBuf>,
) -> Res<()> {
    if xlsx {
        return Err(CliError::not_available("export --xlsx")
            .hint("Use --csv (spreadsheets open it); xlsx comes in a later release."));
    }
    let tid = app.resolve_one(org)?;
    let alias = app.alias_of(&tid);
    let fy = app.record()?.fy_end(&tid)?;
    let end = date
        .clone()
        .or(to.clone())
        .or_else(|| ownpurse_core::intercompany::default_as_of(fy, &today_utc()))
        .unwrap_or_else(today_utc);
    let start = from
        .clone()
        .or_else(|| fiscal_year_start(&end, fy.0, fy.1))
        .unwrap_or_default();
    for d in [&end, &start] {
        if !d.is_empty() && ownpurse_core::dates::parse_iso(d).is_none() {
            return Err(CliError::usage(format!("Not a date: {d}.")).hint("Use YYYY-MM-DD."));
        }
    }
    let mut lines: Vec<Vec<String>> = vec![];
    let (what, period) = match kind.to_ascii_lowercase().as_str() {
        "tb" | "trialbalance" => {
            let l = ledger_for(app, &tid)?;
            let b = resolve(app.record()?, &l, &tid, &end, fy, Mode::Auto)?;
            lines.push(
                vec!["code", "account", "class", "debit", "credit"]
                    .into_iter()
                    .map(String::from)
                    .collect(),
            );
            let mut rows: Vec<(String, Decimal)> =
                b.balances.iter().map(|(k, v)| (k.clone(), *v)).collect();
            rows.sort_by_cached_key(|(k, _)| {
                let a = l.chart.get(k);
                (a.class, a.code, a.name)
            });
            let (mut dr, mut cr) = (Decimal::ZERO, Decimal::ZERO);
            for (id, amt) in rows.into_iter().filter(|(_, a)| !a.is_zero()) {
                let a = l.chart.get(&id);
                let (d, c) = if amt > Decimal::ZERO {
                    dr += amt;
                    (money_cell(amt), String::new())
                } else {
                    cr += -amt;
                    (String::new(), money_cell(-amt))
                };
                lines.push(vec![a.code.clone(), a.name.clone(), a.class.clone(), d, c]);
            }
            lines.push(vec![
                String::new(),
                "Total".into(),
                String::new(),
                money_cell(dr),
                money_cell(cr),
            ]);
            ("trial balance", format!("as of {end} ({})", describe(&b)))
        }
        "gl" | "generalledger" => {
            let l = ledger_for(app, &tid)?;
            lines.push(
                vec![
                    "date",
                    "code",
                    "account",
                    "debit",
                    "credit",
                    "source",
                    "source_id",
                    "contact",
                    "reference",
                    "description",
                ]
                .into_iter()
                .map(String::from)
                .collect(),
            );
            let mut ps: Vec<&ownpurse_core::Posting> = l
                .postings
                .iter()
                .filter(|p| p.date.as_str() >= start.as_str() && p.date.as_str() <= end.as_str())
                .collect();
            ps.sort_by(|a, b| {
                (a.date.as_str(), a.source, a.source_id.as_str(), a.line).cmp(&(
                    b.date.as_str(),
                    b.source,
                    b.source_id.as_str(),
                    b.line,
                ))
            });
            for p in ps {
                let a = l.chart.get(&p.account_id);
                let (d, c) = if p.amount > Decimal::ZERO {
                    (money_cell(p.amount), String::new())
                } else {
                    (String::new(), money_cell(-p.amount))
                };
                lines.push(vec![
                    p.date.clone(),
                    a.code.clone(),
                    a.name.clone(),
                    d,
                    c,
                    p.source.to_string(),
                    p.source_id.clone(),
                    p.contact.clone(),
                    p.reference.clone(),
                    p.description.clone(),
                ]);
            }
            ("general ledger", format!("{start} to {end}"))
        }
        k @ ("bs" | "balancesheet" | "pl" | "profitandloss") => {
            let bs = k.starts_with('b');
            let (report, params) = if bs {
                (
                    "BalanceSheet",
                    format!(r#"{{"date":"{end}","paymentsOnly":"false"}}"#),
                )
            } else {
                (
                    "ProfitAndLoss",
                    format!(r#"{{"fromDate":"{start}","paymentsOnly":"false","toDate":"{end}"}}"#),
                )
            };
            let Some(r) = app.record()?.report(&tid, report, &params)? else {
                return Err(CliError::usage(format!(
                    "No cached {report} for {params}: the record holds what `report fetch` brought."
                ))
                .hint(if bs {
                    format!("Run `ownpurse report fetch {alias} balancesheet --date {end}` (one call).")
                } else {
                    format!("Run `ownpurse report fetch {alias} pl --from {start} --to {end}` (one call).")
                }));
            };
            flatten_report(&r.payload["Reports"][0], &mut lines);
            (
                if bs {
                    "balance sheet (Xero)"
                } else {
                    "profit and loss (Xero)"
                },
                format!(
                    "{} · fetched {}",
                    if bs {
                        format!("as of {end}")
                    } else {
                        format!("{start} to {end}")
                    },
                    r.fetched_at
                ),
            )
        }
        other => {
            return Err(
                CliError::usage(format!("Unknown export '{other}'.")).hint("Use tb, gl, bs or pl.")
            );
        }
    };
    let dest = out_path.unwrap_or_else(|| {
        std::path::PathBuf::from(format!("{alias}-{}-{end}.csv", kind.to_ascii_lowercase()))
    });
    if dest.exists() {
        return Err(CliError::usage(format!(
            "{} exists; nothing was overwritten.",
            dest.display()
        ))
        .hint("Pass --out <new path>."));
    }
    let mut text = String::new();
    for l in &lines {
        text.push_str(&l.iter().map(|c| csv_cell(c)).collect::<Vec<_>>().join(","));
        text.push_str("\r\n");
    }
    std::fs::write(&dest, &text)
        .map_err(|e| CliError::runtime("io", format!("{}: {e}", dest.display())))?;
    let mut out = app.out();
    out.field("org", alias.as_str())
        .field("export", what)
        .field("period", period.as_str())
        .field("file", dest.display().to_string())
        .field("rows", (lines.len().saturating_sub(1)) as i64);
    out.hint("Local numbers (tb, gl) match `ownpurse tb` / `gl`; check them against Xero with `ownpurse parity`.");
    out.emit();
    Ok(())
}

/// A Xero report as CSV rows: the header row's cells, then each row (a section's title on a line of its own).
fn flatten_report(rep: &Value, out: &mut Vec<Vec<String>>) {
    let cell = |c: &Value| c["Value"].as_str().unwrap_or("").to_string();
    for r in rep["Rows"].as_array().into_iter().flatten() {
        match r["RowType"].as_str() {
            Some("Header") => out.push(
                r["Cells"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .map(cell)
                    .collect(),
            ),
            Some("Section") => {
                if let Some(t) = r["Title"].as_str().filter(|t| !t.is_empty()) {
                    out.push(vec![t.to_string()]);
                }
                for x in r["Rows"].as_array().into_iter().flatten() {
                    out.push(
                        x["Cells"]
                            .as_array()
                            .into_iter()
                            .flatten()
                            .map(cell)
                            .collect(),
                    );
                }
            }
            _ => out.push(
                r["Cells"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .map(cell)
                    .collect(),
            ),
        }
    }
}
