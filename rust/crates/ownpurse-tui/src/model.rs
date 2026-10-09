//! What the shell reads from the record: per org its name, last successful sync and today's calls.
//! Read-only (the record is opened with `SQLITE_OPEN_READ_ONLY` by core); built once at start and on
//! `r` (reload, zero calls). Every query here is local SQLite.

use crate::clock::{self, parse_utc};
use ownpurse_core::Record;
use ownpurse_core::config::Config;
use ownpurse_core::estimate::EstimateSettings;
use ownpurse_core::record::RecordError;
use serde_json::{Value, json};
use std::path::PathBuf;

/// Defaults from `defaults.json` (`limits`, `sync.ttl_seconds.default`).
pub const CALLS_PER_DAY: i64 = 1_000;
pub const WARN_DAY_REMAINING: i64 = 100;
pub const TTL_DEFAULT: i64 = 86_400;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Level {
    Ok,
    Warn,
    Err,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OrgInfo {
    pub alias: String,
    pub tenant: String,
    pub name: String,
    pub short_code: String,
    /// Most recent successful sync (`sync_runs.finished_at`), `None` = never synced.
    pub last_sync: Option<String>,
    pub calls_today: i64,
    /// Calls left: the last `day_remaining` Xero reported today (UTC), else limit − today's calls.
    pub left: i64,
    pub fy_end: (u32, u32),
    /// The sync confirm's figure (core `estimate_or_floor`: the estimate, or the first-sync floor);
    /// None only when the estimator itself fails (`calls: unknown`).
    pub sync_estimate: Option<ownpurse_core::estimate::Estimate>,
    /// Xero reported a day-remaining header today (else `left` is limit − today's calls: an estimate,
    /// marked `≈ … (estimated)` wherever it shows).
    pub left_known: bool,
}

impl OrgInfo {
    /// The calls-left figure as shown: `938`, or `≈ 1,000` when it is the local estimate.
    pub fn left_fig(&self) -> String {
        if self.left_known {
            clock::int(self.left)
        } else {
            format!("≈ {}", clock::int(self.left))
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Group {
    pub name: String,
    pub members: Vec<usize>,
}

/// Where the model comes from (kept so `r` can rebuild it): the record, the orgs and groups, and the
/// config sections the views read (parties and intercompany pairs, account_map, limits, ttl).
#[derive(Clone, Debug, PartialEq)]
pub struct Source {
    pub record: PathBuf,
    pub orgs: Vec<(String, String)>,
    pub groups: Vec<(String, Vec<String>)>,
    /// `{"parties": {...}, "intercompany": {...}}` for the intercompany engine (empty without config).
    pub ic_config: Value,
    pub account_map: Value,
    pub calls_per_day: i64,
    pub warn_day_remaining: i64,
    pub ttl: i64,
    /// `tui.as_of` from the config (None = last fiscal year end).
    pub as_of: Option<String>,
    pub profile: String,
    pub estimate: EstimateSettings,
    /// `tui.log_rows`: how many HTTP log rows Activity reads (default 500).
    pub log_rows: i64,
    /// `tui.compare_default_align`: `code` or `map` when Compare opens (default code).
    pub compare_align: String,
    /// `--config` as the TUI got it (passed on to the CLI it runs); None when the config was found.
    pub config_arg: Option<PathBuf>,
    /// The config file this source was loaded from (watched and re-read live); None without one.
    pub config_file: Option<PathBuf>,
    /// The sha256 of that file's bytes when loaded: a save that changes nothing (or an event for another
    /// file) re-reads nothing.
    pub config_digest: Option<String>,
    /// The `links` list (settings; `per: document` templates open documents in Xero).
    pub links: Value,
    /// The config facts System shows (settings with sources, link templates, paths).
    pub sys: crate::system::SysConfig,
    /// `tui.agent` (default true): the agent bridge socket opens (`--no-agent` turns it off too).
    pub agent: bool,
    /// `tui.label`: this session's label in `ui ls` (`--label` overrides).
    pub label: Option<String>,
    /// `tui.keys_file`: keys.toml elsewhere than the user config folder (resolved against the config's folder),
    /// with the text as configured (errors name it so).
    pub keys_file: Option<(PathBuf, String)>,
    /// Where `classify.toml` and the classify store live (None without a config file).
    pub classify: Option<ownpurse_core::classify::ClassifyPaths>,
    /// Manifests: the store and the current-manifest file (None without a config file), the
    /// accountant gate `[(org, through)]` and the must-be-zero accounts per org.
    pub manifests: Option<ownpurse_core::manifest::Paths>,
    /// The profile's `allow_writes` (the header badge; the TUI never writes to Xero).
    pub allow_writes: bool,
    pub accountant_gate: Vec<(String, String)>,
    pub zero_accounts: std::collections::BTreeMap<String, Vec<String>>,
    /// Xero's per-journal line limit, once measured (`manifests.journal_line_limit`).
    pub journal_line_limit: Option<usize>,
    /// The walkthroughs folder (`walkthroughs.dir`; None without a config file) and the folder whose
    /// `glossary.toml` adds to the shipped glossary (the config's).
    pub walkthroughs: Option<PathBuf>,
    pub glossary_dir: Option<PathBuf>,
    /// `tui.accents`: org alias (or `cross`) → accent family (theme spec v2 §2.2).
    pub accents: crate::theme::accent::Assignment,
    /// `tui.accent_intensity` (§2.6), resolved for dark and light (defaults where unset).
    pub knobs: [crate::theme::def::Knobs; 2],
    /// Problems in the theme settings (unknown accent or knob values): shown in System › Settings; the
    /// default is used for each.
    pub theme_issues: Vec<String>,
    /// `tui.theme`: a theme name (built-in or user); None for `auto` or unset.
    pub theme_pref: Option<String>,
    /// `tui.view`: the focused view's visibility rules.
    pub view: View,
    /// `tui.checklist`: the close worklist the palette's "Explain: the close worklist" opens (a JSON file,
    /// relative to the project config's folder).
    pub checklist: Option<String>,
}

/// The sha256 of a file's bytes (None when it can't be read).
pub fn file_digest(p: &std::path::Path) -> Option<String> {
    std::fs::read(p)
        .ok()
        .map(|b| ownpurse_core::write::sha256_hex(&b))
}

/// `tui.view` (theme spec v2 §1.3): what the focused view shows, as config rather than colour.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct View {
    /// `i` starts on (IDs, provenance sentences, budgets, versions).
    pub details: bool,
    /// always | low | never.
    pub budget_in_header: &'static str,
    /// always | stale.
    pub record_age_in_header: &'static str,
    /// The quiet second line under the selected row.
    pub focus_line: bool,
    /// A healthy ✓ in `text.3`, not `ok`.
    pub quiet_ok: bool,
    /// Trial balance rows under class bands.
    pub group_tb_by_class: bool,
    /// comfortable | compact.
    pub density: &'static str,
}

impl Default for View {
    fn default() -> View {
        View {
            details: false,
            budget_in_header: "low",
            record_age_in_header: "stale",
            focus_line: true,
            quiet_ok: true,
            group_tb_by_class: true,
            density: "comfortable",
        }
    }
}

/// `tui.view` from settings (unknown values keep the default, with an issue).
pub fn view_settings(st: &ownpurse_core::config::Settings, issues: &mut Vec<String>) -> View {
    let mut v = View::default();
    let Some(m) = st.get("tui.view").and_then(Value::as_object) else {
        return v;
    };
    for (k, val) in m {
        let word = val.as_str().unwrap_or("");
        let flag = val.as_bool();
        let ok = match (k.as_str(), flag) {
            ("details", Some(b)) => {
                v.details = b;
                true
            }
            ("focus_line", Some(b)) => {
                v.focus_line = b;
                true
            }
            ("quiet_ok", Some(b)) => {
                v.quiet_ok = b;
                true
            }
            ("group_tb_by_class", Some(b)) => {
                v.group_tb_by_class = b;
                true
            }
            ("budget_in_header", _) => match word {
                "always" => {
                    v.budget_in_header = "always";
                    true
                }
                "low" => {
                    v.budget_in_header = "low";
                    true
                }
                "never" => {
                    v.budget_in_header = "never";
                    true
                }
                _ => false,
            },
            ("record_age_in_header", _) => match word {
                "always" => {
                    v.record_age_in_header = "always";
                    true
                }
                "stale" => {
                    v.record_age_in_header = "stale";
                    true
                }
                _ => false,
            },
            ("density", _) => match word {
                "comfortable" => {
                    v.density = "comfortable";
                    true
                }
                "compact" => {
                    v.density = "compact";
                    true
                }
                _ => false,
            },
            _ => false,
        };
        if !ok {
            issues.push(format!(
                "tui.view.{k} = {val}: not a valid value; the default is used"
            ));
        }
    }
    v
}

/// `tui.accents` and `tui.accent_intensity` from settings: (assignment, [dark, light] knobs, issues).
pub fn theme_settings(
    st: &ownpurse_core::config::Settings,
) -> (
    crate::theme::accent::Assignment,
    [crate::theme::def::Knobs; 2],
    Vec<String>,
) {
    use crate::theme::def::Knobs;
    let mut issues = Vec::new();
    let mut accents = crate::theme::accent::Assignment::new();
    if let Some(m) = st.get("tui.accents").and_then(Value::as_object) {
        for (k, v) in m {
            match v.as_str() {
                Some(f) if crate::theme::accent::is_family(f) => {
                    accents.insert(k.clone(), f.to_string());
                }
                _ => issues.push(format!(
                    "tui.accents.{k} = {v}: unknown accent (families: {})",
                    crate::theme::accent::FAMILIES.join(", ")
                )),
            }
        }
    }
    let mut knobs = [Knobs::defaults(true), Knobs::defaults(false)];
    if let Some(m) = st.get("tui.accent_intensity").and_then(Value::as_object) {
        for (k, v) in m {
            let text = |v: &Value| match v {
                Value::String(s) => s.clone(),
                other => other.to_string(),
            };
            let (d, l) = match v {
                Value::Object(o) => (o.get("dark").map(text), o.get("light").map(text)),
                other => (Some(text(other)), Some(text(other))),
            };
            for (slot, val) in [(0, d), (1, l)] {
                if let Some(val) = val
                    && let Err(e) = knobs[slot].set(k, &val)
                {
                    issues.push(format!("tui.accent_intensity.{k}: {e}"));
                }
            }
        }
    }
    issues.dedup();
    (accents, knobs, issues)
}

impl Source {
    /// From flags only (no config): no pairs, no account_map, default limits.
    pub fn bare(
        record: PathBuf,
        orgs: Vec<(String, String)>,
        groups: Vec<(String, Vec<String>)>,
    ) -> Source {
        Source {
            record,
            orgs,
            groups,
            ic_config: json!({}),
            account_map: json!({}),
            calls_per_day: CALLS_PER_DAY,
            warn_day_remaining: WARN_DAY_REMAINING,
            ttl: TTL_DEFAULT,
            as_of: None,
            profile: "default".into(),
            estimate: EstimateSettings::default(),
            log_rows: crate::activity::LOG_ROWS,
            compare_align: "code".into(),
            config_arg: None,
            config_file: None,
            config_digest: None,
            links: ownpurse_core::config::Settings::default_settings()
                .get("links")
                .cloned()
                .unwrap_or_else(|| json!([])),
            sys: crate::system::SysConfig::bare(),
            agent: true,
            label: None,
            classify: None,
            manifests: None,
            allow_writes: false,
            accountant_gate: vec![],
            zero_accounts: Default::default(),
            journal_line_limit: None,
            walkthroughs: None,
            glossary_dir: None,
            checklist: None,
            keys_file: None,
            accents: Default::default(),
            knobs: [
                crate::theme::def::Knobs::defaults(true),
                crate::theme::def::Knobs::defaults(false),
            ],
            theme_issues: vec![],
            theme_pref: None,
            view: View::default(),
        }
    }

    /// From a loaded config (core `config::load`): the profile's record, orgs, groups and sections.
    pub fn from_config(cfg: &Config) -> Source {
        let st = &cfg.settings;
        let (accents, knobs, mut theme_issues) = theme_settings(st);
        let view = view_settings(st, &mut theme_issues);
        let section = |k: &str| {
            st.get(k)
                .filter(|v| v.as_object().is_some_and(|o| !o.is_empty()))
                .cloned()
                .unwrap_or_else(|| json!({}))
        };
        Source {
            record: cfg.store().to_path_buf(),
            orgs: cfg.profile.orgs.clone(),
            groups: st.groups(),
            ic_config: json!({"parties": section("parties"), "intercompany": section("intercompany")}),
            account_map: section("account_map"),
            calls_per_day: st.limit("calls_per_day").unwrap_or(CALLS_PER_DAY),
            warn_day_remaining: st.limit("warn_day_remaining").unwrap_or(WARN_DAY_REMAINING),
            ttl: st
                .get("sync.ttl_seconds.default")
                .and_then(Value::as_i64)
                .unwrap_or(TTL_DEFAULT),
            as_of: st
                .get_str("tui.as_of")
                .filter(|s| !s.is_empty())
                .map(str::to_string),
            profile: cfg.profile.name.clone(),
            estimate: EstimateSettings::from_settings(st),
            log_rows: st
                .get("tui.log_rows")
                .and_then(|v| {
                    v.as_i64()
                        .or_else(|| v.as_str().and_then(|t| t.trim().parse().ok()))
                })
                .filter(|n| *n > 0)
                .unwrap_or(crate::activity::LOG_ROWS),
            compare_align: st
                .get_str("tui.compare_default_align")
                .filter(|a| *a == "code" || *a == "map")
                .unwrap_or("code")
                .to_string(),
            config_arg: None,
            config_file: cfg.path.clone(),
            config_digest: cfg.path.as_ref().and_then(|p| file_digest(p)),
            links: st.get("links").cloned().unwrap_or_else(|| json!([])),
            sys: crate::system::SysConfig::from_config(cfg),
            agent: st
                .get("tui.agent")
                .map(|v| {
                    v.as_bool()
                        .unwrap_or_else(|| !matches!(v.as_str(), Some("false" | "0" | "off")))
                })
                .unwrap_or(true),
            label: st
                .get_str("tui.label")
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(str::to_string),
            classify: Some(ownpurse_core::classify::ClassifyPaths::resolve(cfg)),
            manifests: Some(ownpurse_core::manifest::Paths::resolve(cfg)),
            allow_writes: cfg.profile.allow_writes,
            accountant_gate: ownpurse_core::manifest::accountant_gate(cfg),
            zero_accounts: ownpurse_core::manifest::zero_accounts(cfg),
            journal_line_limit: ownpurse_core::manifest::journal_line_limit(cfg),
            walkthroughs: Some(ownpurse_core::walkthrough::dir(cfg)),
            glossary_dir: Some(cfg.root.clone()),
            checklist: st
                .get_str("tui.checklist")
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(str::to_string),
            keys_file: st
                .get_str("tui.keys_file")
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(|s| {
                    let p = std::path::Path::new(s);
                    let full = if p.is_absolute() {
                        p.to_path_buf()
                    } else {
                        cfg.root.join(p)
                    };
                    (full, s.to_string())
                }),
            accents,
            knobs,
            theme_issues,
            theme_pref: st
                .get_str("tui.theme")
                .map(str::trim)
                .filter(|s| !s.is_empty() && *s != "auto")
                .map(str::to_string),
            view,
        }
    }

    pub fn has_pairs(&self) -> bool {
        self.ic_config["intercompany"]["pairs"]
            .as_array()
            .is_some_and(|a| !a.is_empty())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Model {
    pub orgs: Vec<OrgInfo>,
    pub groups: Vec<Group>,
    pub limit: i64,
    pub warn: i64,
    pub ttl: i64,
    /// The HTTP log and minute counts for the Activity tab.
    pub activity: crate::activity::Activity,
    /// System's org rows and expanded links.
    pub system: crate::system::SysData,
    /// The record's version: the integrity chain's last sequence number (every append moves it), so a
    /// canonical state names exactly which record its rows were derived from.
    pub record_version: i64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Fresh {
    /// Oldest last-sync across the scope; `None` when any member has never synced.
    pub at: Option<String>,
    pub age: i64,
    pub level: Level,
}

impl Fresh {
    /// `record 07:46 UTC · 2h ago [stale]` / `no local record`.
    pub fn full(&self) -> String {
        match &self.at {
            None => "no local record".into(),
            Some(at) => format!(
                "record {} UTC · {}{}",
                clock::hhmm(at),
                self.ago_words(),
                self.stale_suffix()
            ),
        }
    }
    /// `07:46Z · 2h [stale]` (header degrade step 2).
    pub fn compact(&self) -> String {
        match &self.at {
            None => "no local record".into(),
            Some(at) => format!(
                "{}Z · {}{}",
                clock::hhmm(at),
                self.age_token(),
                self.stale_suffix()
            ),
        }
    }
    /// `2h [stale]` / `no record` (shell degrade step 6; the age itself never goes).
    pub fn min(&self) -> String {
        match &self.at {
            None => "no record".into(),
            Some(_) => format!("{}{}", self.age_token(), self.stale_suffix()),
        }
    }
    /// The age token the V5 check looks for.
    pub fn age_token(&self) -> String {
        match &self.at {
            // "no local record" and its minimal form "no record" both end in "record"
            None => "record".into(),
            // under a minute (right after a sync): `just now` (spec P6 net-done-ok)
            Some(_) if self.age < 60 => "just now".into(),
            Some(_) => clock::ago(self.age),
        }
    }
    /// `2h ago` / `just now`.
    fn ago_words(&self) -> String {
        if self.age < 60 {
            "just now".into()
        } else {
            format!("{} ago", clock::ago(self.age))
        }
    }
    fn stale_suffix(&self) -> &'static str {
        if self.level == Level::Warn {
            " stale"
        } else {
            ""
        }
    }
}

fn count(record: &Record, sql: &str, tenant: &str, arg: &str) -> i64 {
    record
        .conn()
        .query_row(sql, (tenant, arg), |r| r.get::<_, i64>(0))
        .unwrap_or(0)
}

impl Model {
    pub fn load(record: &Record, source: &Source, now: i64) -> Result<Model, RecordError> {
        let today = clock::date_of(now);
        let mut orgs = Vec::with_capacity(source.orgs.len());
        for (alias, tenant) in &source.orgs {
            let org = record.organisation(tenant)?;
            let text = |k: &str| {
                org.get(k)
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string()
            };
            let calls_today = count(
                record,
                "SELECT COUNT(*) FROM http_log WHERE tenant_id=?1 AND at >= ?2",
                tenant,
                &today,
            );
            let last: Option<(String, i64)> = record
                .conn()
                .query_row(
                    "SELECT at, day_remaining FROM http_log WHERE tenant_id=?1 AND day_remaining IS NOT NULL \
                     ORDER BY id DESC LIMIT 1",
                    (tenant,),
                    |r| Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?)),
                )
                .ok();
            let (left, left_known) = match last {
                Some((at, rem)) if at.get(..10) == Some(today.as_str()) => (rem, true),
                _ => (source.calls_per_day - calls_today, false),
            };
            let name = text("Name");
            orgs.push(OrgInfo {
                alias: alias.clone(),
                tenant: tenant.clone(),
                name: if name.is_empty() { alias.clone() } else { name },
                short_code: text("ShortCode"),
                last_sync: record.last_sync(tenant)?,
                calls_today,
                left,
                fy_end: record.fy_end(tenant)?,
                sync_estimate: ownpurse_core::estimate::estimate_or_floor(
                    record,
                    tenant,
                    None,
                    false,
                    source.estimate,
                    now,
                )
                .ok(),
                left_known,
            });
        }
        let groups = source
            .groups
            .iter()
            .map(|(name, members)| Group {
                name: name.clone(),
                members: members
                    .iter()
                    .filter_map(|m| orgs.iter().position(|o| &o.alias == m))
                    .collect(),
            })
            .collect();
        let activity = crate::activity::Activity::load(record, &orgs, now, source.log_rows);
        let system = crate::system::SysData::load(record, &source.orgs, &source.sys);
        let record_version = record
            .conn()
            .query_row("SELECT COALESCE(MAX(seq), 0) FROM chain", [], |r| {
                r.get::<_, i64>(0)
            })
            .unwrap_or(0);
        Ok(Model {
            record_version,
            system,
            activity,
            orgs,
            groups,
            limit: source.calls_per_day,
            warn: source.warn_day_remaining,
            ttl: source.ttl,
        })
    }

    pub fn open(source: &Source, now: i64) -> Result<Model, RecordError> {
        let record = Record::open(&source.record)?;
        Model::load(&record, source, now)
    }

    /// Oldest freshness across `members`.
    pub fn freshness(&self, members: &[usize], now: i64) -> Fresh {
        let times: Vec<Option<&String>> = members
            .iter()
            .map(|&i| self.orgs[i].last_sync.as_ref())
            .collect();
        if times.is_empty() || times.iter().any(Option::is_none) {
            return Fresh {
                at: None,
                age: 0,
                level: Level::Err,
            };
        }
        let oldest = times.into_iter().flatten().min().cloned();
        let age = oldest.as_deref().and_then(parse_utc).map_or(0, |t| now - t);
        Fresh {
            at: oldest,
            age,
            level: if age > self.ttl {
                Level::Warn
            } else {
                Level::Ok
            },
        }
    }

    /// Lowest calls-left across `members` and its level (error below 10 %, warn below the warn mark).
    pub fn quota(&self, members: &[usize]) -> (i64, Level) {
        let left = members
            .iter()
            .map(|&i| self.orgs[i].left)
            .min()
            .unwrap_or(self.limit);
        (left, self.left_level(left))
    }

    /// A calls-left figure's level: error below 10 % of the limit, warn below the warn mark.
    pub fn left_level(&self, left: i64) -> Level {
        if left * 10 < self.limit {
            Level::Err
        } else if left < self.warn {
            Level::Warn
        } else {
            Level::Ok
        }
    }

    /// Default as-of: the last fiscal year end before today of the first org.
    pub fn default_as_of(&self, first: Option<usize>, now: i64) -> String {
        let today = clock::date_of(now);
        let year: i64 = today[..4].parse().unwrap_or(1970);
        let (m, d) = first.map_or((12, 31), |i| self.orgs[i].fy_end);
        let this_year = ownpurse_core::dates::iso(year, m, d);
        if this_year < today {
            this_year
        } else {
            ownpurse_core::dates::iso(year - 1, m, d)
        }
    }
}
