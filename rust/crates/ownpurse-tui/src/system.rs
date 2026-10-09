//! What the System tab (spec §9 row 5, merge M3) shows, prepared at load: the config facts (path,
//! profile, store, token file; every setting with its source, from core's layered loader), the org rows
//! from the record (plan, record count, last sync) and the expanded links (per app, org and bank
//! account; core `links`). Login state is not read here: the TUI never opens the token file.

use ownpurse_core::Record;
use ownpurse_core::config::{Config, Settings};
use ownpurse_core::links::{self, Link, Values};
use serde_json::Value;
use std::path::{Component, Path, PathBuf};

/// The config side of System (kept in `Source`, so `r` rebuilds with it).
#[derive(Clone, Debug, PartialEq)]
pub struct SysConfig {
    /// The config file (None = built without one: flags or environment).
    pub path: Option<PathBuf>,
    pub profile: String,
    pub client_id: String,
    pub token_file: Option<PathBuf>,
    /// (dotted key, display value, source) for every setting, then the profile's client id and orgs.
    pub settings: Vec<(String, String, String)>,
    pub link_templates: Vec<Value>,
    pub redirect_uri: String,
    pub read_scopes: usize,
    pub write_scopes: usize,
    /// `tui.editor` (empty = $VISUAL / $EDITOR / vi).
    pub editor: String,
    /// `tui.open_command` (empty = `open` on macOS, else `xdg-open`): runs "Open in Xero" links.
    pub open_command: String,
}

/// A setting value as one line: lists of scalars joined with `, `, strings bare, the rest as JSON.
pub fn show_value(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        Value::Array(a) if a.iter().all(|x| !x.is_object() && !x.is_array()) => a
            .iter()
            .map(|x| x.as_str().map_or_else(|| x.to_string(), str::to_string))
            .collect::<Vec<_>>()
            .join(", "),
        other => other.to_string(),
    }
}

fn count(v: Option<&Value>) -> usize {
    v.and_then(Value::as_array).map_or(0, Vec::len)
}

impl SysConfig {
    fn from_settings(st: &Settings) -> SysConfig {
        SysConfig {
            path: None,
            profile: "default".into(),
            client_id: String::new(),
            token_file: None,
            settings: st
                .flat()
                .into_iter()
                .map(|(k, v, o)| (k, show_value(&v), o))
                .collect(),
            link_templates: st
                .get("links")
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default(),
            redirect_uri: st.redirect_uri(),
            read_scopes: count(st.get("xero.scopes.read")),
            write_scopes: count(st.get("xero.scopes.write")),
            editor: st.get_str("tui.editor").unwrap_or("").to_string(),
            open_command: st.get_str("tui.open_command").unwrap_or("").to_string(),
        }
    }

    /// No config: the shipped defaults (source `defaults`).
    pub fn bare() -> SysConfig {
        SysConfig::from_settings(&Settings::default_settings())
    }

    pub fn from_config(cfg: &Config) -> SysConfig {
        let mut s = SysConfig::from_settings(&cfg.settings);
        s.path = cfg.path.clone();
        s.profile = cfg.profile.name.clone();
        s.client_id = cfg.profile.client_id.clone();
        s.token_file = Some(cfg.profile.token_file.clone());
        let origin = format!("profile:{}", cfg.profile.name);
        s.settings.push((
            "profile.client_id".into(),
            cfg.profile.client_id.clone(),
            origin.clone(),
        ));
        for (a, t) in &cfg.profile.orgs {
            s.settings
                .push((format!("profile.orgs.{a}"), t.clone(), origin.clone()));
        }
        s
    }
}

/// One org on System › Connection.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SysOrg {
    pub alias: String,
    pub name: String,
    pub plan: String,
    pub short_code: String,
    pub tenant: String,
    pub records: i64,
    pub last_sync: Option<String>,
}

/// The record side of System, built with the model.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SysData {
    pub orgs: Vec<SysOrg>,
    pub links: Vec<Link>,
    /// The write packages the record has seen (Phase 8), newest first, with their stages.
    pub packages: Vec<ownpurse_core::write::view::PackageRow>,
    /// Load generation (view caches key on it).
    pub generation: u64,
}

impl SysData {
    pub fn load(record: &Record, orgs: &[(String, String)], cfg: &SysConfig) -> SysData {
        let mut rows = Vec::new();
        let mut org_vals = Vec::new();
        let mut banks = Vec::new();
        for (alias, tenant) in orgs {
            let o = record.organisation(tenant).unwrap_or(Value::Null);
            let text = |k: &str| o.get(k).and_then(Value::as_str).unwrap_or("").to_string();
            let records = record
                .conn()
                .query_row(
                    "SELECT COUNT(*) FROM current WHERE tenant_id=?1",
                    (tenant,),
                    |r| r.get::<_, i64>(0),
                )
                .unwrap_or(0);
            let name = text("Name");
            let row = SysOrg {
                alias: alias.clone(),
                name: if name.is_empty() { alias.clone() } else { name },
                plan: text("Class"),
                short_code: text("ShortCode"),
                tenant: tenant.clone(),
                records,
                last_sync: record.last_sync(tenant).ok().flatten(),
            };
            let base: Values = [
                ("org", row.alias.clone()),
                ("org_name", row.name.clone()),
                ("short_code", row.short_code.clone()),
                ("tenant_id", row.tenant.clone()),
            ]
            .into_iter()
            .map(|(k, v)| (k.to_string(), v))
            .collect();
            for a in ownpurse_core::view::View::main(record)
                .payloads(tenant, "Accounts")
                .unwrap_or_default()
            {
                let s = |k: &str| a.get(k).and_then(Value::as_str).unwrap_or("");
                if s("Type") == "BANK" && s("Status") == "ACTIVE" {
                    let mut b = base.clone();
                    b.insert("account_id".into(), s("AccountID").into());
                    b.insert("account_name".into(), s("Name").into());
                    banks.push(b);
                }
            }
            org_vals.push(base);
            rows.push(row);
        }
        let ctx: Values = [
            ("client_id", cfg.client_id.clone()),
            ("redirect_uri", cfg.redirect_uri.clone()),
            ("profile", cfg.profile.clone()),
        ]
        .into_iter()
        .map(|(k, v)| (k.to_string(), v))
        .collect();
        SysData {
            orgs: rows,
            links: links::expand(&cfg.link_templates, &ctx, &org_vals, &banks),
            packages: ownpurse_core::write::view::packages(record),
            generation: crate::activity::next_generation(),
        }
    }
}

/// Lexically normalised (`a/b/../c` → `a/c`).
fn normalise(p: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for c in p.components() {
        match c {
            Component::ParentDir if out.file_name().is_some() => {
                out.pop();
            }
            Component::CurDir => {}
            other => out.push(other),
        }
    }
    out
}

/// A path as the UI shows it: relative when it's under `cwd`, `~/…` when it's under the home directory,
/// absolute otherwise. A relative path (as configured) is shown as is.
pub fn show_path(p: &Path, cwd: &Path, home: Option<&Path>) -> String {
    if p.is_relative() {
        return p.display().to_string();
    }
    let p = normalise(p);
    if let Ok(rest) = p.strip_prefix(normalise(cwd))
        && !rest.as_os_str().is_empty()
    {
        return rest.display().to_string();
    }
    if let Some(rest) = home.and_then(|h| p.strip_prefix(normalise(h)).ok()) {
        return format!("~/{}", rest.display());
    }
    p.display().to_string()
}

/// The working directory and home the UI resolves paths against (`App::paths`; pinned in snapshots).
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct PathBase {
    pub cwd: PathBuf,
    pub home: Option<PathBuf>,
}

impl PathBase {
    /// The process's cwd and `$HOME`.
    pub fn process() -> PathBase {
        PathBase {
            cwd: std::env::current_dir().unwrap_or_default(),
            home: std::env::var_os("HOME").map(PathBuf::from),
        }
    }

    pub fn show(&self, p: &Path) -> String {
        show_path(p, &self.cwd, self.home.as_deref())
    }
}

/// The editor to run: `tui.editor`, else `$VISUAL`, else `$EDITOR`, else `vi`, split on whitespace
/// (so `code -w` works).
pub fn editor_command(configured: &str, env: &dyn Fn(&str) -> Option<String>) -> Vec<String> {
    let chosen = [Some(configured.to_string()), env("VISUAL"), env("EDITOR")]
        .into_iter()
        .flatten()
        .find(|s| !s.trim().is_empty())
        .unwrap_or_else(|| "vi".into());
    chosen.split_whitespace().map(str::to_string).collect()
}

/// The command that opens a link: `tui.open_command` split on whitespace, else the system's opener.
pub fn open_command(configured: &str) -> Vec<String> {
    let c = configured.trim();
    if c.is_empty() {
        vec![
            if cfg!(target_os = "macos") {
                "open"
            } else {
                "xdg-open"
            }
            .to_string(),
        ]
    } else {
        c.split_whitespace().map(str::to_string).collect()
    }
}

/// The OSC 52 sequence that puts `text` on the system clipboard (base64, BEL-terminated).
pub fn osc52(text: &str) -> String {
    const A: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let b = text.as_bytes();
    let mut enc = String::with_capacity(b.len().div_ceil(3) * 4);
    for chunk in b.chunks(3) {
        let n = chunk
            .iter()
            .enumerate()
            .fold(0u32, |acc, (i, &x)| acc | (u32::from(x) << (16 - 8 * i)));
        for i in 0..4 {
            if i <= chunk.len() {
                enc.push(A[((n >> (18 - 6 * i)) & 63) as usize] as char);
            } else {
                enc.push('=');
            }
        }
    }
    format!("\x1b]52;c;{enc}\x07")
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn osc52_is_base64() {
        assert_eq!(osc52(""), "\x1b]52;c;\x07");
        assert_eq!(osc52("f"), "\x1b]52;c;Zg==\x07");
        assert_eq!(osc52("fo"), "\x1b]52;c;Zm8=\x07");
        assert_eq!(osc52("foo"), "\x1b]52;c;Zm9v\x07");
        assert_eq!(
            osc52("https://go.xero.com/?a=1"),
            "\x1b]52;c;aHR0cHM6Ly9nby54ZXJvLmNvbS8/YT0x\x07"
        );
    }

    #[test]
    fn editor_falls_back_in_order() {
        let none = |_: &str| None;
        assert_eq!(editor_command("", &none), ["vi"]);
        let env = |k: &str| (k == "EDITOR").then(|| "nano".to_string());
        assert_eq!(editor_command("", &env), ["nano"]);
        assert_eq!(editor_command("code -w", &env), ["code", "-w"]);
    }

    #[test]
    fn values_display_as_one_line() {
        assert_eq!(show_value(&json!("x")), "x");
        assert_eq!(show_value(&json!(["a", "b", 3])), "a, b, 3");
        assert_eq!(show_value(&json!([])), "");
        assert_eq!(show_value(&json!(500)), "500");
        assert_eq!(show_value(&json!([{"id": 1}])), "[{\"id\":1}]");
    }

    #[test]
    fn paths_are_shown_relative_or_home_abbreviated() {
        let cwd = Path::new("/w/repo/rust");
        assert_eq!(
            show_path(
                Path::new("/w/repo/rust/crates/tui/../../tests/demo/x.json"),
                cwd,
                None
            ),
            "tests/demo/x.json"
        );
        // outside the working directory: never `../../..`; `~/…` under home, else absolute
        assert_eq!(
            show_path(
                Path::new("/home/me/.config/ownpurse/tokens.json"),
                Path::new("/home/me/work/repo"),
                Some(Path::new("/home/me"))
            ),
            "~/.config/ownpurse/tokens.json"
        );
        assert_eq!(
            show_path(
                Path::new("/w/other/x.json"),
                cwd,
                Some(Path::new("/home/me"))
            ),
            "/w/other/x.json"
        );
        assert_eq!(
            show_path(
                Path::new("/home/me/.config/x.json"),
                cwd,
                Some(Path::new("/home/me"))
            ),
            "~/.config/x.json"
        );
        assert_eq!(show_path(Path::new("/etc/x"), cwd, None), "/etc/x");
        assert_eq!(
            show_path(Path::new("tokens/a.json"), cwd, None),
            "tokens/a.json"
        );
    }

    #[test]
    fn defaults_have_sources_and_links() {
        let s = SysConfig::bare();
        assert!(
            s.settings
                .iter()
                .any(|(k, v, o)| k == "tui.as_of" && v.is_empty() && o == "defaults")
        );
        assert_eq!((s.read_scopes, s.write_scopes), (15, 3));
        assert!(!s.link_templates.is_empty());
    }
}
