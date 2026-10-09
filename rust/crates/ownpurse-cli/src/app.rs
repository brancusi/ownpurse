//! Shared command context: config, the read-only record, org resolution, structured errors.
//!
//! Read commands are record-only and never read the OAuth token file (only sync/login/orgs do): orgs resolve by configured alias (or `all` / `group:<name>` / comma lists), and an unaliased
//! tenant is labelled by its first 8 characters.

use crate::out::Out;
use ownpurse_core::Record;
use ownpurse_core::config::{Config, ConfigError, Env};
use serde_json::{Map, Value};

/// A structured failure.
#[derive(Debug, Clone)]
pub struct CliError {
    pub code: String,
    pub message: String,
    pub hints: Vec<String>,
    pub detail: Vec<(String, Value)>,
    pub exit: i32,
}

impl CliError {
    pub fn new(code: &str, exit: i32, message: impl Into<String>) -> CliError {
        CliError {
            code: code.into(),
            message: message.into(),
            hints: vec![],
            detail: vec![],
            exit,
        }
    }
    pub fn usage(message: impl Into<String>) -> CliError {
        CliError::new("usage", 2, message)
    }
    pub fn runtime(code: &str, message: impl Into<String>) -> CliError {
        CliError::new(code, 1, message)
    }
    pub fn hint(mut self, h: impl Into<String>) -> CliError {
        self.hints.push(h.into());
        self
    }
    /// Commands the record-only Rust CLI does not implement (network, token, or file-writing commands).
    pub fn not_available(cmd: &str) -> CliError {
        CliError::new(
            "not_available",
            2,
            format!("`ownpurse {cmd}` is not yet available in Rust; coming in v0.9.x."),
        )
    }
    pub fn internal(e: impl std::fmt::Display) -> CliError {
        CliError::runtime("error", format!("Internal error: {e}")).hint(
            "This is a bug in ownpurse; rerun with OWNPURSE_DEBUG=1 for a traceback.".to_string(),
        )
    }

    /// Print the error (axi shape).
    pub fn emit(&self, as_json: bool) {
        let mut out = Out::new(as_json);
        out.field("error", self.code.as_str())
            .field("message", self.message.as_str());
        for (k, v) in &self.detail {
            if as_json {
                out.field(k, v.clone());
            } else {
                out.field(k, text_detail(k, v));
            }
        }
        for h in &self.hints {
            out.hint(h.clone());
        }
        out.emit();
    }
}

/// A detail as a person reads it: `--expect` predicates as `level = 2` / `level is 1` (one per line), other
/// nested objects as compact JSON (never a debug dump); scalars and lists of scalars as they are.
fn text_detail(key: &str, v: &Value) -> Value {
    let pred = |p: &Value| -> Option<String> {
        let path = p["path"].as_str()?;
        let shown = |x: &Value| match x {
            Value::String(s) => s.clone(),
            Value::Null => "nothing".into(),
            other => other.to_string(),
        };
        Some(match key {
            "expected" => format!(
                "{path} {} {}",
                p["op"].as_str().unwrap_or("="),
                shown(&p["value"])
            ),
            _ => format!(
                "{path} is {}",
                shown(if p["text"].is_null() {
                    &p["value"]
                } else {
                    &p["text"]
                })
            ),
        })
    };
    match v {
        Value::Array(a) if matches!(key, "expected" | "actual") => {
            let lines: Option<Vec<String>> = a.iter().map(pred).collect();
            lines.map_or_else(
                || Value::String(v.to_string()),
                |l| Value::String(l.join("; ")),
            )
        }
        Value::Object(_) => Value::String(v.to_string()),
        Value::Array(a) if a.iter().any(|x| x.is_object() || x.is_array()) => {
            Value::String(v.to_string())
        }
        other => other.clone(),
    }
}

impl From<ConfigError> for CliError {
    fn from(e: ConfigError) -> Self {
        CliError {
            code: e.code.into(),
            exit: e.exit_code(),
            message: e.message,
            hints: e.hints,
            detail: e.detail,
        }
    }
}

macro_rules! internal_from {
    ($($t:ty),*) => {$(
        impl From<$t> for CliError {
            fn from(e: $t) -> Self { CliError::internal(e) }
        }
    )*};
}
internal_from!(
    ownpurse_core::record::RecordError,
    ownpurse_core::ledger::LedgerError,
    ownpurse_core::balances::BasisError,
    ownpurse_core::balances::XeroTbError,
    rusqlite::Error
);

pub type Res<T> = Result<T, CliError>;

/// Global flags (valid before or after the subcommand).
#[derive(Debug, Clone, Default)]
pub struct Global {
    pub profile: Option<String>,
    pub json: bool,
    pub full: bool,
    pub limit: Option<usize>,
    /// `--offline`: network commands refuse (read commands never call Xero anyway).
    pub offline: bool,
    /// `--manifest <name|main>`: read main + this manifest (default: `manifests.current`, else main).
    pub manifest: Option<String>,
}

pub const DEFAULT_LIMIT: usize = 50;

pub struct App {
    pub g: Global,
    pub config: Config,
    record: Option<Record>,
}

impl App {
    pub fn new(g: Global) -> Res<App> {
        let config = ownpurse_core::config::load(&Env::process(), g.profile.as_deref(), None)?;
        Ok(App {
            g,
            config,
            record: None,
        })
    }

    /// Where the manifest store, the current-manifest file and the packages are.
    pub fn manifest_paths(&self) -> ownpurse_core::manifest::Paths {
        ownpurse_core::manifest::Paths::resolve(&self.config)
    }

    /// The orgs and the accountant gate a view needs.
    pub fn view_ctx(&self) -> ownpurse_core::view::Ctx {
        ownpurse_core::view::Ctx {
            orgs: self.config.profile.orgs.clone(),
            gate: ownpurse_core::manifest::accountant_gate(&self.config),
        }
    }

    /// The manifest the read commands use: `--manifest`, else `manifests.current`, else main (None).
    pub fn manifest_name(&self) -> Option<String> {
        self.g
            .manifest
            .clone()
            .or_else(|| self.manifest_paths().current())
            .filter(|n| n != "main")
    }

    /// The open manifest the read commands use (None = main).
    pub fn current_manifest(&self) -> Res<Option<ownpurse_core::manifest::Manifest>> {
        let Some(name) = self.manifest_name() else {
            return Ok(None);
        };
        let store = ownpurse_core::manifest::Store::open(&self.manifest_paths().store)?;
        Ok(Some(store.open_manifest(&name)?))
    }

    /// Main, or main + the current manifest: the ONE way read commands see the books.
    pub fn view(&mut self) -> Res<ownpurse_core::view::View<'_>> {
        let manifest = self.current_manifest()?;
        let ctx = self.view_ctx();
        let rec = self.record()?;
        Ok(ownpurse_core::view::View::open(
            rec,
            &ctx,
            manifest.as_ref(),
        )?)
    }

    /// The local record, opened read-only on first use (never created).
    pub fn record(&mut self) -> Res<&Record> {
        if self.record.is_none() {
            let path = self.config.store().to_path_buf();
            if !path.exists() {
                return Err(CliError::usage("The local record does not exist yet.")
                    .hint("Run `ownpurse sync all`."));
            }
            self.record = Some(Record::open(&path)?);
        }
        Ok(self.record.as_ref().expect("opened above"))
    }

    pub fn out(&self) -> Out {
        Out::new(self.g.json)
    }

    /// The row limit: everything with --full or --json, else the first --limit (default 50) rows.
    pub fn limit<T: Clone>(&self, rows: &[T]) -> Vec<T> {
        if self.g.full || self.g.json {
            return rows.to_vec();
        }
        let n = self.g.limit.filter(|n| *n > 0).unwrap_or(DEFAULT_LIMIT);
        rows.iter().take(n).cloned().collect()
    }

    pub fn alias_of(&self, tenant_id: &str) -> String {
        self.config
            .profile
            .alias(tenant_id)
            .map(str::to_string)
            .unwrap_or_else(|| tenant_id.chars().take(8).collect())
    }

    /// alias | 'all' | 'group:<name>' | comma list of any of these (without a token).
    pub fn resolve(&self, org: &str) -> Res<Vec<String>> {
        if org.contains(',') {
            let mut out: Vec<String> = Vec::new();
            for part in org.split(',') {
                for tid in self.resolve(part.trim())? {
                    if !out.contains(&tid) {
                        out.push(tid);
                    }
                }
            }
            return Ok(out);
        }
        if let Some(name) = org.strip_prefix("group:") {
            let groups = self.config.settings.groups();
            let Some((_, members)) = groups.iter().find(|(n, _)| n == name) else {
                let names: Vec<&str> = groups.iter().map(|(n, _)| n.as_str()).collect();
                return Err(
                    CliError::usage(format!("Unknown group '{name}'.")).hint(format!(
                        "Groups: {}",
                        if names.is_empty() {
                            "none configured".to_string()
                        } else {
                            names.join(", ")
                        }
                    )),
                );
            };
            let mut out = Vec::new();
            for m in members {
                out.extend(self.resolve(m)?);
            }
            return Ok(out);
        }
        let orgs = &self.config.profile.orgs;
        if org == "all" {
            if orgs.is_empty() {
                return Err(CliError::usage("No orgs configured or connected.")
                    .hint("Run `ownpurse login`."));
            }
            return Ok(orgs.iter().map(|(_, t)| t.clone()).collect());
        }
        if let Some(t) = self.config.profile.tenant(org) {
            return Ok(vec![t.to_string()]);
        }
        let known: Vec<&str> = orgs.iter().map(|(a, _)| a.as_str()).collect();
        Err(
            CliError::usage(format!("Unknown or ambiguous org '{org}'.")).hint(format!(
                "Orgs: {}",
                if known.is_empty() {
                    "none".to_string()
                } else {
                    known.join(", ")
                }
            )),
        )
    }

    /// For commands that work on one organisation: refuse a group, list or `all` that names several.
    pub fn resolve_one(&self, org: &str) -> Res<String> {
        let tids = self.resolve(org)?;
        if tids.len() != 1 {
            let names: Vec<String> = tids.iter().map(|t| self.alias_of(t)).collect();
            return Err(CliError::usage(format!(
                "'{org}' names {} orgs; this command works on one.",
                tids.len()
            ))
            .hint(format!("Pick one of: {}", names.join(", "))));
        }
        Ok(tids.into_iter().next().expect("one"))
    }

    /// Latest successful sync of an entity (`finished_at`), or the offline error.
    pub fn ensure(&mut self, tenant_id: &str, entity: &str) -> Res<String> {
        let last: Option<String> = {
            let rec = self.record()?;
            let mut st = rec.conn().prepare_cached(
                "SELECT finished_at FROM sync_runs WHERE tenant_id=?1 AND entity=?2 AND outcome='ok' \
                 ORDER BY finished_at DESC LIMIT 1",
            )?;
            let mut rows = st.query(rusqlite::params![tenant_id, entity])?;
            match rows.next()? {
                Some(r) => Some(r.get(0)?),
                None => None,
            }
        };
        last.ok_or_else(|| {
            CliError::runtime(
                "offline",
                format!("{entity} has never been synced for this org."),
            )
            .hint("Drop --offline, or run `ownpurse sync <org>`.".to_string())
        })
    }

    /// Current payloads with `_status` / `_version` appended, and the sync time.
    pub fn rows_of(
        &mut self,
        tenant_id: &str,
        entity: &str,
    ) -> Res<(Vec<Map<String, Value>>, String)> {
        let as_of = self.ensure(tenant_id, entity)?;
        let rows = self.record()?.current_rows(tenant_id, entity)?;
        let out = rows
            .into_iter()
            .map(|r| {
                let mut m = match r.payload {
                    Value::Object(m) => m,
                    _ => Map::new(),
                };
                m.insert(
                    "_status".into(),
                    r.status.map(Value::String).unwrap_or(Value::Null),
                );
                m.insert("_version".into(), Value::from(r.version));
                m
            })
            .collect();
        Ok((out, as_of))
    }

    /// Oldest last-sync time across the orgs read.
    pub fn as_of_min(&mut self, tids: &[String], entity: &str) -> Res<Option<String>> {
        let mut times = Vec::new();
        for t in tids {
            if let Ok(f) = self.ensure(t, entity) {
                times.push(f);
            }
        }
        Ok(times.into_iter().min())
    }

    /// Ledger source entities must all have been synced.
    pub fn ensure_ledger(&mut self, tenant_id: &str) -> Res<()> {
        for e in [
            "Organisation",
            "Accounts",
            "BankTransactions",
            "BankTransfers",
            "ManualJournals",
            "Invoices",
            "CreditNotes",
            "Payments",
        ] {
            self.ensure(tenant_id, e)?;
        }
        Ok(())
    }
}

impl From<ownpurse_core::manifest::ManifestError> for CliError {
    fn from(e: ownpurse_core::manifest::ManifestError) -> Self {
        let exit = match e.code {
            "usage" | "forbidden" | "not_found" | "patch" => 2,
            "check" | "review" => 4,
            "precondition" | "conflict" | "view_changed" => 4,
            _ => 1,
        };
        CliError::new(e.code, exit, e.message)
    }
}

#[cfg(test)]
mod text_detail_tests {
    use super::text_detail;
    use serde_json::json;

    #[test]
    fn an_expect_refusal_reads_as_words_in_text_mode() {
        let e = json!([{"op": "=", "path": "level", "value": "2"}, {"op": "~", "path": "breadcrumb", "value": "Card"}]);
        let a = json!([{"path": "level", "text": "1", "value": 1}, {"path": "breadcrumb", "text": null, "value": null}]);
        assert_eq!(
            text_detail("expected", &e),
            json!("level = 2; breadcrumb ~ Card")
        );
        assert_eq!(
            text_detail("actual", &a),
            json!("level is 1; breadcrumb is nothing")
        );
        // nested values are compact JSON; scalars and scalar lists are untouched
        assert_eq!(text_detail("x", &json!({"a": 1})), json!("{\"a\":1}"));
        assert_eq!(
            text_detail("anchors", &json!(["a", "b"])),
            json!(["a", "b"])
        );
        assert_eq!(text_detail("rev", &json!(3)), json!(3));
    }
}
