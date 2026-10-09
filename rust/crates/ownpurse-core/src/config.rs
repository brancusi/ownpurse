//! Configuration and profile selection.
//!
//! Lookup order for the config file (first hit wins):
//!   1. `$OWNPURSE_CONFIG`
//!   2. `ownpurse.json` in the current directory or any parent (project config)
//!   3. `$XDG_CONFIG_HOME/ownpurse/config.json` (default `~/.config/ownpurse/config.json`, the user config)
//!
//! With no file at all, a single "default" profile is built from `$OWNPURSE_CLIENT_ID`; with neither, the
//! error lists where it looked.
//!
//! Effective settings are layered: shipped `defaults.json` ← the config file's top-level sections ← the
//! selected profile's sections. Dicts merge recursively; lists and scalars replace; `extra_links` appends
//! to `links`. Every leaf remembers its origin (`defaults`, `config`, `profile:<name>`).
//!
//! Relative `store` paths resolve against the config file's directory. The token file path is computed for
//! display only: nothing in Rust reads it.

use serde_json::{Map, Value};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub const CONFIG_NAME: &str = "ownpurse.json";

/// Old name kept for callers; see `COMING`. The command for what the Rust CLI does not run yet (init, report fetch, q, history, attachments): every hint
/// that names one of them uses it, so nobody is sent to a Rust command that answers `not_available`. sync,
/// login and orgs are Rust commands since Phase 6 and their hints say `ownpurse`. The frozen CLI
/// goldens' wording (`ownpurse init …`) differs only by this prefix; the golden test rewrites it.
pub const NETWORK_CLI: &str = "ownpurse";
/// What a hint says about a command that is not available yet.
pub const COMING: &str = "coming in v0.9.x";
/// How to create a config until `init` is ported.
pub const CREATE_CONFIG: &str =
    "Create ownpurse.json from ownpurse.example.json (`ownpurse init` is coming in v0.9.x).";

/// The shipped defaults (`defaults.json`).
pub const DEFAULTS_JSON: &str = include_str!("../defaults.json");

/// Mergeable settings sections.
pub const SECTIONS: [&str; 15] = [
    "branches",
    "manifests",
    "walkthroughs",
    "classify",
    "xero",
    "limits",
    "sync",
    "tui",
    "groups",
    "parties",
    "intercompany",
    "account_map",
    "links",
    "extra_links",
    "instructions",
];

/// A structured configuration failure. `code` is `config` (runtime, exit 1) or `usage` (exit 2).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigError {
    pub code: &'static str,
    pub message: String,
    pub hints: Vec<String>,
    /// Extra structured fields (e.g. `looked_in`), in output order.
    pub detail: Vec<(String, Value)>,
}

impl ConfigError {
    fn config(message: impl Into<String>, hints: Vec<String>) -> Self {
        ConfigError {
            code: "config",
            message: message.into(),
            hints,
            detail: vec![],
        }
    }
    fn usage(message: impl Into<String>, hints: Vec<String>) -> Self {
        ConfigError {
            code: "usage",
            message: message.into(),
            hints,
            detail: vec![],
        }
    }
    pub fn exit_code(&self) -> i32 {
        if self.code == "usage" { 2 } else { 1 }
    }
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for ConfigError {}

/// The process environment, injectable for tests.
#[derive(Debug, Clone)]
pub struct Env {
    pub vars: HashMap<String, String>,
    pub cwd: PathBuf,
    pub home: PathBuf,
}

impl Env {
    /// The real environment.
    pub fn process() -> Env {
        let vars: HashMap<String, String> = std::env::vars().collect();
        let home = vars
            .get("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("/"));
        Env {
            cwd: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
            vars,
            home,
        }
    }

    /// A variable, treating empty as unset .
    pub fn var(&self, k: &str) -> Option<&str> {
        self.vars
            .get(k)
            .map(String::as_str)
            .filter(|v| !v.is_empty())
    }

    pub fn config_home(&self) -> PathBuf {
        self.var("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| self.home.join(".config"))
            .join("ownpurse")
    }

    pub fn data_home(&self) -> PathBuf {
        self.var("XDG_DATA_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| self.home.join(".local").join("share"))
            .join("ownpurse")
    }

    /// `Path.expanduser()` for a leading `~`.
    pub fn expand_user(&self, p: &str) -> PathBuf {
        if p == "~" {
            self.home.clone()
        } else if let Some(rest) = p.strip_prefix("~/") {
            self.home.join(rest)
        } else {
            PathBuf::from(p)
        }
    }
}

/// Effective settings with the origin of every leaf.
#[derive(Debug, Clone)]
pub struct Settings {
    pub data: Map<String, Value>,
    pub origins: HashMap<String, String>,
}

fn join(path: &str, k: &str) -> String {
    if path.is_empty() {
        k.to_string()
    } else {
        format!("{path}.{k}")
    }
}

fn mark(value: &Value, origin: &str, origins: &mut HashMap<String, String>, path: &str) {
    if let Value::Object(m) = value {
        for (k, v) in m {
            mark(v, origin, origins, &join(path, k));
        }
    } else {
        origins.insert(path.to_string(), origin.to_string());
    }
}

fn merge(
    base: Option<&Value>,
    over: &Value,
    origin: &str,
    origins: &mut HashMap<String, String>,
    path: &str,
) -> Value {
    if let (Some(Value::Object(b)), Value::Object(o)) = (base, over) {
        let mut out = b.clone();
        for (k, v) in o {
            let merged = merge(b.get(k), v, origin, origins, &join(path, k));
            out.insert(k.clone(), merged);
        }
        return Value::Object(out);
    }
    mark(over, origin, origins, path);
    over.clone()
}

/// The shipped defaults as a JSON object.
pub fn defaults() -> Map<String, Value> {
    match serde_json::from_str(DEFAULTS_JSON) {
        Ok(Value::Object(m)) => m,
        _ => unreachable!("defaults.json is a JSON object (checked by tests)"),
    }
}

impl Settings {
    /// defaults ← `file_raw` sections ← `profile_raw` sections.
    pub fn build(
        file_raw: Option<&Map<String, Value>>,
        profile_name: Option<&str>,
        profile_raw: Option<&Map<String, Value>>,
    ) -> Settings {
        let mut origins = HashMap::new();
        let mut data = defaults();
        mark(&Value::Object(data.clone()), "defaults", &mut origins, "");
        let empty = Map::new();
        let profile_origin = format!("profile:{}", profile_name.unwrap_or("None"));
        for (origin, layer) in [
            ("config", file_raw.unwrap_or(&empty)),
            (profile_origin.as_str(), profile_raw.unwrap_or(&empty)),
        ] {
            for section in SECTIONS {
                if let Some(over) = layer.get(section) {
                    let merged = merge(data.get(section), over, origin, &mut origins, section);
                    data.insert(section.to_string(), merged);
                }
            }
        }
        let mut links: Vec<Value> = match data.get("links") {
            Some(Value::Array(a)) => a.clone(),
            _ => vec![],
        };
        if let Some(Value::Array(extra)) = data.shift_remove("extra_links") {
            links.extend(extra);
        }
        data.insert("links".into(), Value::Array(links));
        Settings { data, origins }
    }

    pub fn default_settings() -> Settings {
        Settings::build(None, None, None)
    }

    /// Dotted lookup (`xero.scopes.read`).
    pub fn get(&self, dotted: &str) -> Option<&Value> {
        let mut parts = dotted.split('.');
        let mut cur = self.data.get(parts.next()?)?;
        for part in parts {
            cur = cur.as_object()?.get(part)?;
        }
        Some(cur)
    }

    pub fn get_str(&self, dotted: &str) -> Option<&str> {
        self.get(dotted).and_then(Value::as_str)
    }

    pub fn origin(&self, dotted: &str) -> &str {
        self.origins
            .get(dotted)
            .map(String::as_str)
            .unwrap_or("defaults")
    }

    /// An integer limit (`limits.<name>`).
    pub fn limit(&self, name: &str) -> Option<i64> {
        match self.get(&format!("limits.{name}"))? {
            Value::Number(n) => n
                .as_i64()
                .or_else(|| n.to_string().split('.').next()?.parse().ok()),
            Value::String(s) => s.trim().parse().ok(),
            Value::Bool(b) => Some(i64::from(*b)),
            _ => None,
        }
    }

    pub fn redirect_uri(&self) -> String {
        self.get_str("xero.redirect_uri").unwrap_or("").to_string()
    }

    /// group name → member org aliases (config `groups.<name>.orgs`), in config order.
    pub fn groups(&self) -> Vec<(String, Vec<String>)> {
        let Some(Value::Object(g)) = self.get("groups") else {
            return vec![];
        };
        g.iter()
            .map(|(name, v)| {
                let members = v
                    .get("orgs")
                    .and_then(Value::as_array)
                    .map(|a| {
                        a.iter()
                            .map(|m| {
                                m.as_str()
                                    .map(str::to_string)
                                    .unwrap_or_else(|| m.to_string())
                            })
                            .collect()
                    })
                    .unwrap_or_default();
                (name.clone(), members)
            })
            .collect()
    }

    /// (dotted key, value, origin) for every leaf except links / instructions, then one row per group.
    pub fn flat(&self) -> Vec<(String, Value, String)> {
        fn walk(obj: &Value, path: &str, s: &Settings, rows: &mut Vec<(String, Value, String)>) {
            if let Value::Object(m) = obj {
                for (k, v) in m {
                    walk(v, &join(path, k), s, rows);
                }
            } else {
                rows.push((path.to_string(), obj.clone(), s.origin(path).to_string()));
            }
        }
        let mut rows = Vec::new();
        for (k, v) in &self.data {
            if matches!(k.as_str(), "links" | "instructions" | "groups") {
                continue;
            }
            walk(v, k, self, &mut rows);
        }
        for (name, members) in self.groups() {
            let key = format!("groups.{name}.orgs");
            let origin = self.origin(&key).to_string();
            rows.push((
                key,
                Value::Array(members.into_iter().map(Value::String).collect()),
                origin,
            ));
        }
        rows
    }
}

/// One OAuth app + one token file + one local record.
#[derive(Debug, Clone)]
pub struct Profile {
    pub name: String,
    pub client_id: String,
    /// Displayed by `config path`; never read by Rust.
    pub token_file: PathBuf,
    pub store: PathBuf,
    /// alias → tenant id, in config order.
    pub orgs: Vec<(String, String)>,
    pub allow_writes: bool,
    pub require_tenant_name: Option<String>,
    pub forbid_tenants: Vec<String>,
    /// `history_notes` as configured (None: the default, see [`Profile::history_notes`]).
    pub history_notes_setting: Option<bool>,
}

impl Profile {
    pub fn tenant(&self, alias: &str) -> Option<&str> {
        self.orgs
            .iter()
            .find(|(a, _)| a == alias)
            .map(|(_, t)| t.as_str())
    }

    /// Whether each landed write adds its audit note to the document's history in Xero: the profile's
    /// `history_notes`, else on for a real profile and off for a Demo Company one (its `require_tenant_name`).
    pub fn history_notes(&self) -> bool {
        self.history_notes_setting.unwrap_or_else(|| {
            !self
                .require_tenant_name
                .as_deref()
                .is_some_and(|n| n.contains("Demo Company"))
        })
    }

    /// First alias configured for a tenant id.
    pub fn alias(&self, tenant_id: &str) -> Option<&str> {
        self.orgs
            .iter()
            .find(|(_, t)| t == tenant_id)
            .map(|(a, _)| a.as_str())
    }
}

#[derive(Debug, Clone)]
pub struct Config {
    /// The config file, or None when built from the environment only.
    pub path: Option<PathBuf>,
    pub root: PathBuf,
    pub profile: Profile,
    pub settings: Settings,
}

impl Config {
    pub fn store(&self) -> &Path {
        &self.profile.store
    }
}

/// The config file to use, or None.
pub fn find_config(env: &Env, start: Option<&Path>) -> Result<Option<PathBuf>, ConfigError> {
    if let Some(p) = env.var("OWNPURSE_CONFIG") {
        let p = env.expand_user(p);
        if !p.is_file() {
            return Err(ConfigError::config(
                format!("OWNPURSE_CONFIG points to a missing file: {}", p.display()),
                vec![],
            ));
        }
        return Ok(Some(p));
    }
    let start = start.unwrap_or(&env.cwd);
    let here = start.canonicalize().unwrap_or_else(|_| start.to_path_buf());
    for d in here.ancestors() {
        let c = d.join(CONFIG_NAME);
        if c.is_file() {
            return Ok(Some(c));
        }
    }
    let user = env.config_home().join("config.json");
    Ok(user.is_file().then_some(user))
}

fn str_of(v: Option<&Value>) -> Option<String> {
    match v? {
        Value::Null => None,
        Value::String(s) if s.is_empty() => None,
        Value::String(s) => Some(s.clone()),
        Value::Bool(false) => None,
        other => Some(other.to_string()),
    }
}

/// Truthiness of a JSON value (null, false, 0, "" and empty containers are false).
fn truthy(v: Option<&Value>) -> bool {
    match v {
        None | Some(Value::Null) | Some(Value::Bool(false)) => false,
        Some(Value::String(s)) => !s.is_empty(),
        Some(Value::Array(a)) => !a.is_empty(),
        Some(Value::Object(o)) => !o.is_empty(),
        Some(Value::Number(n)) => {
            crate::money::parse(&n.to_string()).map_or(true, |d| !d.is_zero())
        }
        Some(Value::Bool(true)) => true,
    }
}

/// Load the configuration and select a profile.
pub fn load(env: &Env, profile: Option<&str>, path: Option<&Path>) -> Result<Config, ConfigError> {
    let path = match path {
        Some(p) => Some(p.to_path_buf()),
        None => find_config(env, None)?,
    };
    let (raw, root) = match &path {
        None => {
            if env.var("OWNPURSE_CLIENT_ID").is_none() {
                let here = &env.cwd;
                let user = env.config_home().join("config.json");
                let mut e = ConfigError::config(
                    format!(
                        "No ownpurse config found (looked for {CONFIG_NAME} in {} and its parent folders, then {}).",
                        here.display(),
                        user.display()
                    ),
                    vec![
                        "cd into your project folder (the one containing ownpurse.json) and run the command again."
                            .into(),
                        "Or point at it: export OWNPURSE_CONFIG=/path/to/ownpurse.json".into(),
                        format!("New setup: {CREATE_CONFIG}"),
                    ],
                );
                e.detail.push((
                    "looked_in".into(),
                    Value::Array(vec![
                        Value::String(here.join(CONFIG_NAME).display().to_string()),
                        Value::String("(parent folders)".into()),
                        Value::String(user.display().to_string()),
                    ]),
                ));
                return Err(e);
            }
            let raw = serde_json::json!({"default_profile": "default",
                "profiles": {"default": {"client_id": env.var("OWNPURSE_CLIENT_ID").unwrap_or("")}}});
            (raw, env.cwd.clone())
        }
        Some(p) => {
            let text = std::fs::read_to_string(p).map_err(|e| {
                ConfigError::config(format!("{} could not be read ({e}).", p.display()), vec![])
            })?;
            let raw: Value = serde_json::from_str(&text).map_err(|e| {
                ConfigError::config(format!("{} is not valid JSON ({e}).", p.display()), vec![])
            })?;
            let root = p.parent().map(Path::to_path_buf).unwrap_or_default();
            (raw, root)
        }
    };
    let empty = Map::new();
    let raw_map = raw.as_object().unwrap_or(&empty);
    let name: String = profile
        .map(str::to_string)
        .or_else(|| env.var("OWNPURSE_PROFILE").map(str::to_string))
        .or_else(|| {
            raw_map.get("default_profile").map(|v| {
                v.as_str()
                    .map(str::to_string)
                    .unwrap_or_else(|| v.to_string())
            })
        })
        .unwrap_or_else(|| "default".into());
    let profiles = raw_map
        .get("profiles")
        .and_then(Value::as_object)
        .unwrap_or(&empty);
    let Some(p) = profiles.get(&name).and_then(Value::as_object) else {
        let names: Vec<&str> = profiles.keys().map(String::as_str).collect();
        return Err(ConfigError::usage(
            format!("Unknown profile '{name}'."),
            vec![format!(
                "Profiles: {}",
                if names.is_empty() {
                    "none".to_string()
                } else {
                    names.join(", ")
                }
            )],
        ));
    };
    if !truthy(p.get("client_id")) {
        return Err(ConfigError::config(
            format!("Profile '{name}' has no client_id."),
            vec![
                CREATE_CONFIG.to_string(),
                "Create a 'mobile or desktop' (PKCE) app at developer.xero.com/app/manage.".into(),
            ],
        ));
    }
    let store = env
        .var("OWNPURSE_STORE")
        .map(str::to_string)
        .or_else(|| str_of(p.get("store")))
        .or_else(|| str_of(raw_map.get("store")));
    let mut store_path = match store {
        Some(s) => env.expand_user(&s),
        None => env.data_home().join(format!("{name}.sqlite")),
    };
    if !store_path.is_absolute() {
        store_path = root.join(store_path);
    }
    let token = str_of(p.get("token_file")).unwrap_or_else(|| {
        env.config_home()
            .join("tokens")
            .join(format!("{name}.json"))
            .display()
            .to_string()
    });
    // Per-profile sections, plus the legacy per-profile keys mapped onto the xero section.
    let mut prof_over: Map<String, Value> = p
        .iter()
        .filter(|(k, _)| SECTIONS.contains(&k.as_str()))
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();
    let mut legacy = Map::new();
    if let Some(r) = p.get("redirect_uri") {
        legacy.insert("redirect_uri".into(), r.clone());
    }
    if let Some(s) = p.get("scopes") {
        legacy.insert("scopes".into(), serde_json::json!({"read": s}));
    }
    if !legacy.is_empty() {
        if let Some(Value::Object(x)) = prof_over.get("xero") {
            for (k, v) in x {
                legacy.insert(k.clone(), v.clone());
            }
        }
        prof_over.insert("xero".into(), Value::Object(legacy));
    }
    let settings = Settings::build(Some(raw_map), Some(&name), Some(&prof_over));
    let orgs = p
        .get("orgs")
        .and_then(Value::as_object)
        .map(|m| {
            m.iter()
                .map(|(a, t)| {
                    (
                        a.clone(),
                        t.as_str()
                            .map(str::to_string)
                            .unwrap_or_else(|| t.to_string()),
                    )
                })
                .collect()
        })
        .unwrap_or_default();
    let profile = Profile {
        client_id: str_of(p.get("client_id")).unwrap_or_default(),
        token_file: env.expand_user(&token),
        store: store_path,
        orgs,
        allow_writes: truthy(p.get("allow_writes")),
        require_tenant_name: str_of(p.get("require_tenant_name")),
        history_notes_setting: p.get("history_notes").and_then(Value::as_bool),
        forbid_tenants: p
            .get("forbid_tenants")
            .and_then(Value::as_array)
            .map(|a| {
                a.iter()
                    .filter_map(|v| v.as_str().map(str::to_string))
                    .collect()
            })
            .unwrap_or_default(),
        name,
    };
    check_write_guard(&profile)?;
    Ok(Config {
        path,
        root,
        settings,
        profile,
    })
}

/// The tenant guard, at config load (Phase 8): a profile that allows writes must not list a forbidden
/// tenant among its own orgs, and must name each org's tenant once. (`require_tenant_name` is checked live,
/// against the connected tenant, before the first write of each run.)
fn check_write_guard(p: &Profile) -> Result<(), ConfigError> {
    if !p.allow_writes {
        return Ok(());
    }
    let overlap: Vec<&str> = p
        .orgs
        .iter()
        .filter(|(_, t)| p.forbid_tenants.contains(t))
        .map(|(a, _)| a.as_str())
        .collect();
    if !overlap.is_empty() {
        return Err(ConfigError::config(
            format!(
                "Profile '{}' sets allow_writes but its orgs include forbidden tenants: {}.",
                p.name,
                overlap.join(", ")
            ),
            vec!["Remove those orgs from the profile, or allow_writes; never both.".into()],
        ));
    }
    if p.orgs.is_empty() {
        return Err(ConfigError::config(
            format!("Profile '{}' sets allow_writes but lists no orgs.", p.name),
            vec!["A writing profile names exactly the orgs it may write to.".into()],
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Tmp(PathBuf);
    impl Tmp {
        fn new(tag: &str) -> Tmp {
            let p = std::env::temp_dir()
                .join(format!("ownpurse-config-test-{tag}-{}", std::process::id()));
            let _ = std::fs::remove_dir_all(&p);
            std::fs::create_dir_all(&p).unwrap();
            Tmp(p.canonicalize().unwrap())
        }
    }
    impl Drop for Tmp {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    fn env(cwd: &Path, home: &Path, vars: &[(&str, &str)]) -> Env {
        Env {
            vars: vars
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            cwd: cwd.to_path_buf(),
            home: home.to_path_buf(),
        }
    }

    const CFG: &str = r#"{
      "default_profile": "live",
      "limits": {"calls_per_day": 500},
      "groups": {"pair": {"orgs": ["a", "b"]}},
      "extra_links": [{"name": "Extra", "url": "https://example.com"}],
      "profiles": {
        "live": {"client_id": "cid", "orgs": {"a": "T-A", "b": "T-B"}, "store": "db/x.sqlite",
                 "limits": {"warn_day_remaining": 7}, "redirect_uri": "http://localhost:9/cb",
                 "tui": {"pinned_orgs": ["a"]}},
        "other": {"client_id": "cid2", "token_file": "~/t.json"},
        "nocid": {"orgs": {}}
      }
    }"#;

    #[test]
    fn defaults_parse_and_have_every_section() {
        let d = defaults();
        for s in [
            "xero",
            "limits",
            "sync",
            "tui",
            "links",
            "instructions",
            "groups",
        ] {
            assert!(d.contains_key(s), "{s}");
        }
    }

    #[test]
    fn layers_and_origins() {
        let t = Tmp::new("layers");
        let proj = t.0.join("proj");
        let sub = proj.join("a").join("b");
        std::fs::create_dir_all(&sub).unwrap();
        std::fs::write(proj.join(CONFIG_NAME), CFG).unwrap();
        let e = env(&sub, &t.0, &[]);
        let c = load(&e, None, None).unwrap();
        assert_eq!(c.path.as_deref(), Some(proj.join(CONFIG_NAME).as_path()));
        assert_eq!(c.profile.name, "live");
        assert_eq!(c.profile.store, proj.join("db/x.sqlite"));
        assert_eq!(
            c.profile.token_file,
            t.0.join(".config/ownpurse/tokens/live.json")
        );
        assert_eq!(c.profile.tenant("b"), Some("T-B"));
        assert_eq!(c.profile.alias("T-A"), Some("a"));
        let s = &c.settings;
        assert_eq!(s.limit("calls_per_day"), Some(500));
        assert_eq!(s.origin("limits.calls_per_day"), "config");
        assert_eq!(s.limit("warn_day_remaining"), Some(7));
        assert_eq!(s.origin("limits.warn_day_remaining"), "profile:live");
        assert_eq!(s.origin("limits.calls_per_minute"), "defaults");
        assert_eq!(s.redirect_uri(), "http://localhost:9/cb");
        assert_eq!(s.origin("xero.redirect_uri"), "profile:live");
        assert_eq!(s.origin("tui.pinned_orgs"), "profile:live");
        assert_eq!(
            s.groups(),
            vec![("pair".into(), vec!["a".into(), "b".into()])]
        );
        let links = s.get("links").unwrap().as_array().unwrap();
        assert_eq!(links.last().unwrap()["name"], "Extra");
        assert!(!s.data.contains_key("extra_links"));
        // flat: groups last, links/instructions absent, order follows the defaults.
        let flat = s.flat();
        assert_eq!(flat.first().unwrap().0, "xero.auth_url");
        assert_eq!(flat.last().unwrap().0, "groups.pair.orgs");
        assert!(flat.iter().all(|r| !r.0.starts_with("links")));
    }

    #[test]
    fn env_overrides_profile_and_store() {
        let t = Tmp::new("env");
        let cfg = t.0.join("c.json");
        std::fs::write(&cfg, CFG).unwrap();
        let e = env(
            &t.0,
            &t.0,
            &[
                ("OWNPURSE_CONFIG", cfg.to_str().unwrap()),
                ("OWNPURSE_PROFILE", "other"),
                ("OWNPURSE_STORE", "/abs/rec.sqlite"),
            ],
        );
        let c = load(&e, None, None).unwrap();
        assert_eq!(c.profile.name, "other");
        assert_eq!(c.profile.store, PathBuf::from("/abs/rec.sqlite"));
        assert_eq!(c.profile.token_file, t.0.join("t.json"));
        // explicit profile beats the environment
        let c = load(&e, Some("live"), None).unwrap();
        assert_eq!(c.profile.name, "live");
    }

    #[test]
    fn errors_name_the_problem() {
        let t = Tmp::new("errors");
        let e = env(&t.0, &t.0, &[]);
        let err = load(&e, None, None).unwrap_err();
        assert_eq!(err.code, "config");
        assert!(
            err.message
                .starts_with("No ownpurse config found (looked for ownpurse.json in ")
        );
        assert_eq!(err.hints.len(), 3);
        assert_eq!(err.detail[0].0, "looked_in");
        let e2 = env(&t.0, &t.0, &[("OWNPURSE_CONFIG", "/nope/x.json")]);
        assert_eq!(
            load(&e2, None, None).unwrap_err().message,
            "OWNPURSE_CONFIG points to a missing file: /nope/x.json"
        );
        let cfg = t.0.join(CONFIG_NAME);
        std::fs::write(&cfg, CFG).unwrap();
        let e3 = env(&t.0, &t.0, &[]);
        let u = load(&e3, Some("zzz"), None).unwrap_err();
        assert_eq!((u.code, u.exit_code()), ("usage", 2));
        assert_eq!(u.message, "Unknown profile 'zzz'.");
        assert_eq!(u.hints, vec!["Profiles: live, other, nocid".to_string()]);
        let n = load(&e3, Some("nocid"), None).unwrap_err();
        assert_eq!(n.message, "Profile 'nocid' has no client_id.");
        std::fs::write(&cfg, "{not json").unwrap();
        assert!(
            load(&e3, None, None)
                .unwrap_err()
                .message
                .contains("is not valid JSON")
        );
    }

    #[test]
    fn client_id_from_environment_without_a_file() {
        let t = Tmp::new("envonly");
        let e = env(&t.0, &t.0, &[("OWNPURSE_CLIENT_ID", "abc")]);
        let c = load(&e, None, None).unwrap();
        assert!(c.path.is_none());
        assert_eq!(c.profile.name, "default");
        assert_eq!(c.profile.client_id, "abc");
        assert_eq!(
            c.profile.store,
            t.0.join(".local/share/ownpurse/default.sqlite")
        );
    }

    #[test]
    fn user_config_is_the_last_resort() {
        let t = Tmp::new("user");
        let home = t.0.join("home");
        std::fs::create_dir_all(home.join(".config/ownpurse")).unwrap();
        std::fs::write(home.join(".config/ownpurse/config.json"), CFG).unwrap();
        let e = env(&t.0, &home, &[]);
        let c = load(&e, None, None).unwrap();
        assert_eq!(c.path.unwrap(), home.join(".config/ownpurse/config.json"));
        // relative store resolves against the config file's folder
        assert_eq!(c.profile.store, home.join(".config/ownpurse/db/x.sqlite"));
    }

    #[test]
    fn a_writing_profile_may_not_list_a_forbidden_tenant() {
        let t = Tmp::new("write-guard");
        let cfg = t.0.join("ownpurse.json");
        let write = |p: serde_json::Value| {
            std::fs::write(
                &cfg,
                serde_json::json!({"profiles": {"demo": p}}).to_string(),
            )
            .unwrap();
        };
        let e = env(&t.0, &t.0, &[]);
        write(
            serde_json::json!({"client_id": "c", "orgs": {"demo": "t-1", "real": "t-2"},
                     "forbid_tenants": ["t-2"], "allow_writes": true}),
        );
        let err = load(&e, Some("demo"), Some(&cfg)).unwrap_err();
        assert!(
            err.message.contains("forbidden tenants: real"),
            "{}",
            err.message
        );
        // without allow_writes the same profile loads (reads keep the old behaviour)
        write(
            serde_json::json!({"client_id": "c", "orgs": {"demo": "t-1", "real": "t-2"}, "forbid_tenants": ["t-2"]}),
        );
        assert!(load(&e, Some("demo"), Some(&cfg)).is_ok());
        write(serde_json::json!({"client_id": "c", "allow_writes": true}));
        assert!(
            load(&e, Some("demo"), Some(&cfg))
                .unwrap_err()
                .message
                .contains("lists no orgs")
        );
        write(
            serde_json::json!({"client_id": "c", "orgs": {"demo": "t-1"}, "forbid_tenants": ["t-2"],
                     "allow_writes": true, "require_tenant_name": "Demo Company (US)"}),
        );
        assert!(
            load(&e, Some("demo"), Some(&cfg))
                .unwrap()
                .profile
                .allow_writes
        );
    }
}
