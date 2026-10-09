//! The network commands (Phase 6): `sync`, `login` and `orgs`, over the `ownpurse-xero` crate. They are the only commands that call Xero or touch the token
//! file; every read command stays record-only. `login --write` (Phase 8) asks for the write scopes only for a profile with `allow_writes`.

use crate::app::{App, CliError, Res};
use crate::obj;
use ownpurse_core::config::COMING;
use ownpurse_xero::ApiError;
use ownpurse_xero::form::UreqForm;
use ownpurse_xero::http::{HttpSettings, UreqGet, XeroHttp};
use ownpurse_xero::store::Writer;
use ownpurse_xero::sync::{REFERENCE, Syncer, TRANSACTIONS, entity};
use ownpurse_xero::{TokenStore, auth};
use serde_json::{Map, Value};
use std::time::Duration;

impl From<ApiError> for CliError {
    fn from(e: ApiError) -> Self {
        let mut c = CliError::new(e.code, e.exit_code(), e.message.clone());
        c.hints = e.hints.clone();
        if let Some(b) = e.body {
            c.detail.push(("body".into(), Value::from(b)));
        }
        c
    }
}

fn setting(app: &App, key: &str) -> String {
    app.config.settings.get_str(key).unwrap_or("").to_string()
}

fn int_setting(app: &App, key: &str, default: i64) -> i64 {
    app.config
        .settings
        .get(key)
        .and_then(|v| {
            v.as_i64()
                .or_else(|| v.as_str().and_then(|s| s.parse().ok()))
        })
        .unwrap_or(default)
}

fn refuse_offline(app: &App, cmd: &str) -> Res<()> {
    if app.g.offline {
        return Err(CliError::new(
            "offline",
            1,
            format!("`{cmd}` needs Xero, and --offline forbids network calls."),
        )
        .hint("Drop --offline to run it."));
    }
    Ok(())
}

fn tokens(app: &App) -> TokenStore {
    TokenStore::new(
        &app.config.profile.token_file,
        app.config.profile.client_id.as_str(),
    )
    .with_token_url(setting(app, "xero.token_url"))
}

fn http_settings(app: &App) -> HttpSettings {
    HttpSettings {
        profile: app.config.profile.name.clone(),
        api_base: setting(app, "xero.api_base"),
        connections_url: setting(app, "xero.connections_url"),
        minute_soft_limit: app.config.settings.limit("minute_soft_limit").unwrap_or(55),
        forbid_tenants: app.config.profile.forbid_tenants.clone(),
    }
}

/// Check the profile's tenants against the token's connections.
fn check_profile_tenants(app: &App, tenants: &[Map<String, Value>]) -> Res<()> {
    let p = &app.config.profile;
    let name = |t: &Map<String, Value>| {
        t.get("tenantName")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string()
    };
    let id = |t: &Map<String, Value>| {
        t.get("tenantId")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string()
    };
    let bad: Vec<String> = tenants
        .iter()
        .filter(|t| p.forbid_tenants.contains(&id(t)))
        .map(name)
        .collect();
    if !bad.is_empty() {
        return Err(CliError::usage(format!(
            "Profile '{}' is connected to forbidden orgs: {}.",
            p.name,
            bad.join(", ")
        ))
        .hint("Disconnect them in Xero (Settings → Connected apps) before using this profile."));
    }
    if let Some(req) = &p.require_tenant_name
        && tenants.iter().any(|t| name(t) != *req)
    {
        return Err(CliError::usage(format!(
            "Profile '{}' may only connect to '{req}'.",
            p.name
        )));
    }
    Ok(())
}

/// A `--progress jsonl` event: one compact JSON object per line on stderr. Write errors are ignored, so a run
/// whose reader went away (the TUI quit) still completes and records its result.
fn event(on: bool, fields: Map<String, Value>) {
    if on {
        use std::io::Write;
        let mut e = std::io::stderr().lock();
        let _ = writeln!(e, "{}", Value::Object(fields));
        let _ = e.flush();
    }
}

#[derive(Default)]
struct Progress {
    org: String,
    calls: i64,
}

pub fn sync(
    app: &mut App,
    org: &str,
    names: &[String],
    reference: bool,
    transactions: bool,
    progress: bool,
) -> Res<()> {
    refuse_offline(app, "sync")?;
    let list: Vec<String> = if !names.is_empty() {
        names.to_vec()
    } else if reference {
        REFERENCE.iter().map(|s| s.to_string()).collect()
    } else if transactions {
        TRANSACTIONS.iter().map(|s| s.to_string()).collect()
    } else {
        REFERENCE
            .iter()
            .chain(TRANSACTIONS.iter())
            .map(|s| s.to_string())
            .collect()
    };
    let ents = list
        .iter()
        .map(|n| entity(n))
        .collect::<Result<Vec<_>, _>>()?;
    let tids = app.resolve(org)?;
    let store = Writer::open(app.config.store())?;
    let tok = tokens(app);
    let state = std::sync::Arc::new(std::sync::Mutex::new(Progress::default()));
    let (s_call, s_wait) = (state.clone(), state.clone());
    let http = XeroHttp::new(
        http_settings(app),
        &store,
        &tok,
        Box::new(UreqGet::default()),
    )
    .with_on_call(Box::new(move |c| {
        if c.tenant_id.is_none() {
            return;
        }
        let mut st = s_call.lock().expect("progress state");
        st.calls += 1;
        event(
            progress,
            obj![
                ("ev", "entity"),
                ("org", st.org.clone()),
                ("entity", c.path.clone()),
                ("page", c.page.map(Value::from).unwrap_or(Value::Null)),
                ("calls", st.calls),
                (
                    "day_remaining",
                    c.day_remaining.map(Value::from).unwrap_or(Value::Null)
                ),
            ],
        );
    }))
    .with_sleep(Box::new(move |secs, reason| {
        let org = s_wait.lock().expect("progress state").org.clone();
        event(
            progress,
            obj![
                ("ev", "wait"),
                ("org", org),
                ("reason", reason),
                ("seconds", secs.ceil() as i64),
            ],
        );
        std::thread::sleep(Duration::from_secs_f64(secs));
    }));
    let record = ownpurse_core::Record::open(app.config.store()).ok();
    let core_ents: Option<Vec<ownpurse_core::estimate::Entity>> =
        if names.is_empty() && !reference && !transactions {
            None
        } else {
            let refs: Vec<&str> = list.iter().map(String::as_str).collect();
            ownpurse_core::estimate::entities_named(&refs)
        };
    let est_settings =
        ownpurse_core::estimate::EstimateSettings::from_settings(&app.config.settings);
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    let mut syncer = Syncer::new(&store, &http);
    syncer.page_size = int_setting(app, "sync.page_size", 1000);
    syncer.overlap_seconds = int_setting(app, "sync.ims_overlap_seconds", 5);
    syncer.full_every = int_setting(app, "sync.full_sweep_days", 7) * 86400;
    let mut rows: Vec<Map<String, Value>> = Vec::new();
    let mut any_error = false;
    let mut any_vanished = false;
    for tid in &tids {
        let alias = app.alias_of(tid);
        {
            let mut st = state.lock().expect("progress state");
            st.org = alias.clone();
            st.calls = 0;
        }
        let est = record.as_ref().and_then(|r| {
            ownpurse_core::estimate::estimate_or_floor(
                r,
                tid,
                core_ents.as_deref(),
                app.g.full,
                est_settings,
                now,
            )
            .ok()
        });
        let mut start = obj![
            ("ev", "start"),
            ("org", alias.clone()),
            ("entities", ents.len() as i64),
            (
                "estimate",
                est.map(|e| Value::from(e.calls())).unwrap_or(Value::Null)
            ),
        ];
        if let Some(e) = est {
            start.insert("estimate_label".into(), Value::from(e.label()));
        }
        event(progress, start);
        let (mut org_records, mut org_new) = (0i64, 0i64);
        for e in &ents {
            match syncer.sync(tid, e, app.g.full) {
                Ok(r) => {
                    any_vanished |= r.vanished > 0;
                    org_records += r.fetched as i64;
                    org_new += r.new_versions as i64;
                    rows.push(obj![
                        ("org", app.alias_of(tid)),
                        ("entity", r.entity),
                        ("mode", r.mode),
                        ("fetched", r.fetched as i64),
                        ("new_versions", r.new_versions as i64),
                        ("calls", r.calls),
                        ("vanished", r.vanished as i64),
                    ]);
                }
                Err(err) if err.code == "forbidden" => {
                    any_error = true;
                    rows.push(obj![
                        ("org", app.alias_of(tid)),
                        ("entity", e.name),
                        ("mode", "skipped"),
                        ("fetched", 0),
                        ("new_versions", 0),
                        ("calls", 1),
                        ("vanished", 0),
                        ("error", "forbidden (scope)"),
                    ]);
                }
                Err(err) => return Err(err.into()),
            }
        }
        let calls = state.lock().expect("progress state").calls;
        event(
            progress,
            obj![
                ("ev", "done"),
                ("org", alias),
                ("calls", calls),
                ("records", org_records),
                ("new_versions", org_new),
            ],
        );
    }
    let mut out = app.out();
    let mut cols = vec![
        "org",
        "entity",
        "mode",
        "fetched",
        "new_versions",
        "vanished",
        "calls",
    ];
    if any_error {
        cols.push("error");
    }
    let used: i64 = rows
        .iter()
        .map(|r| r.get("calls").and_then(Value::as_i64).unwrap_or(0))
        .sum();
    out.table("synced", &cols, rows, None);
    out.field("calls_used", used);
    if any_vanished {
        out.hint(format!(
            "Some ids vanished from Xero since the last full sync (see sync_runs.detail; `ownpurse q` is {COMING})."
        ));
    }
    out.hint("Run `ownpurse accounts <org>`, `ownpurse txns <org>` or `ownpurse tui` to read the record.".to_string());
    out.emit();
    Ok(())
}

fn tenant_rows(
    app: &App,
    tenants: &[Map<String, Value>],
    with_type: bool,
) -> Vec<Map<String, Value>> {
    tenants
        .iter()
        .map(|t| {
            let id = t.get("tenantId").and_then(Value::as_str).unwrap_or("");
            let mut m = obj![
                ("org", app.alias_of(id)),
                ("name", t.get("tenantName").cloned().unwrap_or(Value::Null))
            ];
            if with_type {
                m.insert(
                    "type".into(),
                    t.get("tenantType").cloned().unwrap_or(Value::Null),
                );
            }
            m.insert("tenant_id".into(), Value::from(id));
            m
        })
        .collect()
}

pub fn orgs(app: &mut App) -> Res<()> {
    let tenants: Vec<Map<String, Value>> = if app.g.offline {
        match tokens(app).load()? {
            Some(t) => t
                .get("tenants")
                .and_then(Value::as_array)
                .map(|a| a.iter().filter_map(|x| x.as_object().cloned()).collect())
                .unwrap_or_default(),
            None => vec![],
        }
    } else {
        let store = Writer::open(app.config.store())?;
        let tok = tokens(app);
        XeroHttp::new(
            http_settings(app),
            &store,
            &tok,
            Box::new(UreqGet::default()),
        )
        .connections()?
    };
    check_profile_tenants(app, &tenants)?;
    let mut out = app.out();
    out.table(
        "orgs",
        &["org", "name", "type", "tenant_id"],
        tenant_rows(app, &tenants, true),
        None,
    );
    out.hint("Run `ownpurse org <org>` for plan and lock dates.");
    out.emit();
    Ok(())
}

fn open_browser(url: &str) {
    let opener = if cfg!(target_os = "macos") {
        "open"
    } else {
        "xdg-open"
    };
    let _ = std::process::Command::new(opener)
        .arg(url)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn();
}

pub fn login(
    app: &mut App,
    timeout: Option<u64>,
    no_browser: bool,
    progress: bool,
    write: bool,
) -> Res<()> {
    refuse_offline(app, "login")?;
    // `--write` (Phase 8): only a profile that allows writes asks for write scopes; refused before the browser
    let write_scopes: Vec<String> = if write {
        if !app.config.profile.allow_writes {
            return Err(CliError::new(
                "writes_disabled",
                2,
                format!(
                    "Profile '{}' does not set allow_writes: `login --write` asks for nothing.",
                    app.config.profile.name
                ),
            )
            .hint("Write scopes belong only to a profile that allows writes (e.g. the Demo Company's)."));
        }
        app.config
            .settings
            .get("xero.scopes.write")
            .and_then(Value::as_array)
            .map(|a| {
                a.iter()
                    .filter_map(|v| v.as_str().map(str::to_string))
                    .collect()
            })
            .unwrap_or_default()
    } else {
        vec![]
    };
    let read: Vec<String> = app
        .config
        .settings
        .get("xero.scopes.read")
        .and_then(Value::as_array)
        .map(|a| {
            a.iter()
                .filter_map(|v| v.as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default();
    let tok = tokens(app);
    let timeout_secs = timeout
        .unwrap_or_else(|| int_setting(app, "xero.login_timeout_seconds", 900).max(1) as u64);
    let token_url = setting(app, "xero.token_url");
    let auth_url = setting(app, "xero.auth_url");
    let redirect = app.config.settings.redirect_uri();
    let opts = auth::Login {
        auth_url: &auth_url,
        token_url: &token_url,
        client_id: &app.config.profile.client_id,
        redirect_uri: &redirect,
        read_scopes: &read,
        write_scopes: &write_scopes,
        timeout: Duration::from_secs(timeout_secs),
        bound: None,
    };
    let as_json = app.g.json;
    auth::login(&opts, &tok, &UreqForm::default(), &mut |url: &str| {
        let mut o = crate::out::Out::new(as_json);
        o.field("status", "awaiting_browser_consent")
            .field("url", url);
        o.hint("Approve in the browser and select every organisation you need.");
        o.emit();
        event(
            progress,
            obj![
                ("ev", "browser"),
                ("url_shown", no_browser),
                ("timeout", timeout_secs as i64),
            ],
        );
        if !no_browser {
            open_browser(url);
        }
    })?;
    let store = Writer::open(app.config.store())?;
    let tenants = XeroHttp::new(
        http_settings(app),
        &store,
        &tok,
        Box::new(UreqGet::default()),
    )
    .connections()?;
    check_profile_tenants(app, &tenants)?;
    let scope: Vec<Value> = tok.granted_scopes()?.into_iter().map(Value::from).collect();
    let connected: Vec<String> = tenants
        .iter()
        .filter_map(|t| {
            t.get("tenantId")
                .and_then(Value::as_str)
                .map(str::to_string)
        })
        .collect();
    let missing: Vec<String> = app
        .config
        .profile
        .orgs
        .iter()
        .filter(|(_, t)| !connected.contains(t))
        .map(|(a, _)| a.clone())
        .collect();
    let mut out = app.out();
    out.field("status", "logged_in")
        .field("scopes", Value::Array(scope));
    out.table(
        "orgs",
        &["org", "name", "tenant_id"],
        tenant_rows(app, &tenants, false),
        None,
    );
    if !missing.is_empty() {
        out.hint(format!(
            "Expected orgs not connected: {} — rerun `ownpurse login` and tick them.",
            missing.join(", ")
        ));
    }
    event(
        progress,
        obj![("ev", "done"), ("orgs", tenants.len() as i64)],
    );
    out.hint("Run `ownpurse sync all` to build the local record.");
    out.emit();
    Ok(())
}

/// `report fetch <org|all> <kind> [--date D | --from F --to T] [--cash]`: one call per org, appended to the
/// record's `reports` (an observation, like a sync). Defaults: the org's last fiscal year-end before today,
/// and that fiscal year for a period report. `parity` / `tb --since` read the latest row for their params.
#[allow(clippy::too_many_arguments)]
pub fn report_fetch(
    app: &mut App,
    org: &str,
    kind: &str,
    date: Option<String>,
    from: Option<String>,
    to: Option<String>,
    cash: bool,
) -> Res<()> {
    use ownpurse_xero::reports::{self, Kind, Request};
    let k = Kind::parse(kind).ok_or_else(|| {
        CliError::usage(format!("Unknown report '{kind}'."))
            .hint("Use trialbalance, balancesheet, pl or banksummary.")
    })?;
    refuse_offline(app, "report fetch")?;
    let tids = app.resolve(org)?;
    // defaults from the record (no call): the last fiscal year-end before today, and its year
    let mut reqs = vec![];
    for tid in &tids {
        let fy = app
            .record()
            .ok()
            .and_then(|r| r.fy_end(tid).ok())
            .unwrap_or((12, 31));
        let end = ownpurse_core::intercompany::default_as_of(fy, &crate::ledger::today_utc());
        let (date, to) = if k.is_period() {
            (None, to.clone().or(end))
        } else {
            (date.clone().or(end), None)
        };
        let from = if k.is_period() {
            from.clone().or_else(|| {
                to.as_deref()
                    .and_then(|t| ownpurse_core::dates::fiscal_year_start(t, fy.0, fy.1))
            })
        } else {
            None
        };
        let req = Request {
            kind: k,
            date,
            from,
            to,
            cash,
        };
        req.params()?; // a bad date is a usage error before any call
        reqs.push((tid.clone(), req));
    }
    let store = Writer::open(app.config.store())?;
    let tok = tokens(app);
    let http = XeroHttp::new(
        http_settings(app),
        &store,
        &tok,
        Box::new(UreqGet::default()),
    );
    let mut rows = vec![];
    for (tid, req) in &reqs {
        let f = reports::fetch(&http, &store, tid, req)?;
        rows.push(obj![
            ("org", app.alias_of(tid)),
            ("report", f.report),
            ("params", f.params),
            ("rows", f.rows as i64),
            (
                "change",
                if f.first {
                    "first"
                } else if f.unchanged {
                    "unchanged"
                } else {
                    "changed"
                }
            ),
            ("fetched_at", f.fetched_at),
            ("content_hash", f.content_hash[..12].to_string()),
        ]);
    }
    let mut out = app.out();
    out.table(
        "reports",
        &[
            "org",
            "report",
            "params",
            "rows",
            "change",
            "fetched_at",
            "content_hash",
        ],
        rows,
        None,
    );
    out.hint(if k == Kind::TrialBalance {
        "Run `ownpurse parity <org> --date <date>` to compare the local trial balance with it."
    } else {
        "Read it with `ownpurse report list <org>`."
    });
    out.emit();
    Ok(())
}

/// A document id as typed: the full id, or a prefix the record resolves to exactly one document (as
/// `show`). A prefix that matches nothing is `not_found`, several `ambiguous`: Xero is never asked with a
/// partial id. A full GUID the record doesn't hold yet is taken as typed (a document not synced yet).
fn doc_id(app: &mut App, tid: &str, entity: &str, id: &str) -> Res<String> {
    let rows = app.record()?.current_rows(tid, entity)?;
    if rows.iter().any(|r| r.xero_id == id) {
        return Ok(id.to_string());
    }
    let hits: Vec<String> = rows
        .iter()
        .filter(|r| r.xero_id.starts_with(id))
        .map(|r| r.xero_id.clone())
        .collect();
    let guid = id.len() == 36 && id.chars().filter(|c| *c == '-').count() == 4;
    match hits.as_slice() {
        [one] => Ok(one.clone()),
        [] if guid => Ok(id.to_string()),
        [] => Err(CliError::new(
            "not_found",
            2,
            format!("No {entity} in the record starts with '{id}'."),
        )
        .hint(format!("Use the full id, or sync {entity} first."))),
        many => Err(CliError::new(
            "ambiguous",
            2,
            format!("'{id}' matches {} {entity}.", many.len()),
        )
        .hint("Use a longer prefix.")),
    }
}

/// `history <org> <entity> <id> [--cached]`: a document's history and notes. ⇅ one call (into the record),
/// or `--cached` / `--offline`: what the record last saw (zero calls).
pub fn history(app: &mut App, org: &str, entity: &str, id: &str, cached: bool) -> Res<()> {
    use ownpurse_xero::history as h;
    let ename = h::entity(entity).ok_or_else(|| {
        CliError::usage(format!("Xero keeps no history for '{entity}'.")).hint(format!(
            "Entities with a history: {}.",
            h::ENTITIES.join(", ")
        ))
    })?;
    let tid = app.resolve_one(org)?;
    let full = doc_id(app, &tid, ename, id)?;
    let id = full.as_str();
    let (records, source) = if cached || app.g.offline {
        let rec = app.record()?;
        let rows = rec.current_rows(&tid, "History")?;
        let k = h::key(ename, id);
        let row = rows.into_iter().find(|r| r.xero_id == k).ok_or_else(|| {
            CliError::usage(format!("The record holds no history for {ename} {id}.")).hint(format!(
                "Run `ownpurse history {org} {ename} {id}` (one call)."
            ))
        })?;
        (
            row.payload.as_array().cloned().unwrap_or_default(),
            "record",
        )
    } else {
        let store = Writer::open(app.config.store())?;
        let tok = tokens(app);
        let http = XeroHttp::new(
            http_settings(app),
            &store,
            &tok,
            Box::new(UreqGet::default()),
        );
        let f = h::fetch(&http, &store, &tid, ename, id)?;
        (
            f.records,
            if f.new_version {
                "xero (new version)"
            } else {
                "xero (unchanged)"
            },
        )
    };
    let rows: Vec<Map<String, Value>> = records
        .iter()
        .map(|r| {
            let (when, change, by, details) = h::line(r);
            obj![
                ("when", when),
                ("change", change),
                ("by", by),
                ("details", details)
            ]
        })
        .collect();
    let mut out = app.out();
    out.field("entity", ename)
        .field("id", id)
        .field("source", source);
    out.table("history", &["when", "change", "by", "details"], rows, None);
    out.hint("Notes from ownpurse read `ownpurse package <hash12> op <id>: …` (the audit trail of each write).");
    out.emit();
    Ok(())
}

/// `attachments <org> <entity> <id> [--cached] [--get NAME [--out PATH]]`: a document's attachments (⇅ one
/// call, into the record; `--cached` / `--offline`: what the record holds), and with `--get` one file's
/// bytes saved to `--out` (default: the file name in the current folder; never over an existing file).
pub fn attachments(
    app: &mut App,
    org: &str,
    entity: &str,
    id: &str,
    cached: bool,
    get: Option<&str>,
    out_path: Option<&std::path::Path>,
) -> Res<()> {
    use ownpurse_xero::attachments as at;
    let ename = at::entity(entity).ok_or_else(|| {
        CliError::usage(format!("Xero keeps no attachments for '{entity}'.")).hint(format!(
            "Entities with attachments: {}.",
            at::ENTITIES.join(", ")
        ))
    })?;
    let tid = app.resolve_one(org)?;
    let full = doc_id(app, &tid, ename, id)?;
    let id = full.as_str();
    if get.is_some() && (cached || app.g.offline) {
        return Err(CliError::usage(
            "A download needs Xero: drop --cached / --offline.",
        ));
    }
    let (items, source) = if cached || app.g.offline {
        let prefix = format!("{ename}/{id}/");
        let rows = app.record()?.current_rows(&tid, "Attachments")?;
        let items: Vec<Value> = rows
            .into_iter()
            .filter(|r| r.xero_id.starts_with(&prefix))
            .map(|r| r.payload)
            .collect();
        (items, "record".to_string())
    } else {
        let store = Writer::open(app.config.store())?;
        let tok = tokens(app);
        let http = XeroHttp::new(
            http_settings(app),
            &store,
            &tok,
            Box::new(UreqGet::default()),
        );
        let (items, added) = at::list(&http, &store, &tid, ename, id)?;
        let mut saved = None;
        if let Some(name) = get {
            if !items.iter().any(|a| a["FileName"] == name) {
                return Err(CliError::usage(format!(
                    "{ename} {id} has no attachment named {name}."
                ))
                .hint("The list shows the names (without --get)."));
            }
            let dest = out_path
                .map(std::path::Path::to_path_buf)
                .unwrap_or_else(|| std::path::PathBuf::from(name));
            if dest.exists() {
                return Err(CliError::usage(format!(
                    "{} exists; nothing was overwritten.",
                    dest.display()
                ))
                .hint("Pass --out <new path>."));
            }
            let bytes = at::download(&http, &tid, ename, id, name)?;
            std::fs::write(&dest, &bytes)
                .map_err(|e| CliError::runtime("io", format!("{}: {e}", dest.display())))?;
            saved = Some((dest.display().to_string(), bytes.len()));
        }
        if let Some((path, n)) = saved {
            let mut out = app.out();
            out.field("entity", ename)
                .field("id", id)
                .field("saved", path.as_str())
                .field("bytes", n as i64);
            out.hint("The file is as Xero holds it; nothing in Xero changed.");
            out.emit();
            return Ok(());
        }
        (
            items,
            if added > 0 {
                format!("xero ({added} new or changed)")
            } else {
                "xero (unchanged)".to_string()
            },
        )
    };
    let rows: Vec<Map<String, Value>> = items
        .iter()
        .map(|a| {
            obj![
                ("file", a["FileName"].clone()),
                ("type", a["MimeType"].clone()),
                ("bytes", a["ContentLength"].clone()),
                ("attachment_id", a["AttachmentID"].clone()),
            ]
        })
        .collect();
    let mut out = app.out();
    out.field("entity", ename)
        .field("id", id)
        .field("source", source.as_str());
    out.table(
        "attachments",
        &["file", "type", "bytes", "attachment_id"],
        rows,
        None,
    );
    out.hint(format!(
        "Save one with `ownpurse attachments {org} {ename} {id} --get <file> --out <path>`."
    ));
    out.emit();
    Ok(())
}
