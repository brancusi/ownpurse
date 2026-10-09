//! The canonical view state (v0.10.2, round-trip law V21): everything that affects the frame, in one
//! serialisable block, so a fresh app built from it (plus the same record) draws the same frame, byte for
//! byte. `ui state` carries it as `canonical`; `ownpurse-tui --state FILE --snapshot WxH` and
//! `ownpurse ui screen --from-state FILE` render from it alone.
//!
//! What it holds: the whole `Ui` (tab, scope, as-of, basis, filter, drill state and cursors per scope,
//! overlays and their queries, the confirm band and its focus, prompts and the caret, the running child and
//! the result band, notes, history, caption, size, theme), the agent's footprint, the view revision, and
//! the environment the render reads: the clock (`now`, and `model_now` the record was read at), offline,
//! the version label, the CLI path and version, the path base, the label, the update notice, and the
//! record version (the chain's last sequence number) it was derived from.
//!
//! Importing against another record version is refused (`record_changed`, exit 4) unless the caller
//! allows drift; then the app renders anyway and marks itself drifted (a header chip and `drift` in state).
//!
//! Scopes are stored by ALIAS in the canonical JSON (`{"Org": "roast"}`, `{"Group": "all-co"}`, Compare's
//! and a via chain's orgs too) and resolved against the current config on import. The config parts that
//! affect rendering are fingerprinted (`config`: org aliases with tenant ids, groups, account_map; never
//! secrets, never paths). A pure reorder of orgs or groups is tolerated (same orgs, same frame); any other
//! difference (an org removed, renamed, added or on another tenant; groups or account_map changed) is
//! refused, `config_changed`, exit 4, naming what differs, unless config drift is allowed: then aliases still
//! in the config resolve, missing ones fall back to `all`, and the app is marked drifted.

use crate::app::{App, Ui};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// The canonical state's schema version. Every release reads the previous release's states (gate rule:
/// tests/fixtures/state/v<previous>/ must import), migrating older schemas step by step (`MIGRATIONS`).
/// v0.10.2 and v0.10.3 both write v1.
pub const STATE_V: u32 = 1;

/// One schema step: a state at `from` becomes a state at `from + 1` (JSON level, before deserialising).
pub type Migration = fn(Value) -> Result<Value, String>;

/// The schema steps, oldest first: `(from, step)`. v1 is the first schema, so there are none yet; a
/// v2 adds `(1, v1_to_v2)` here and freezes its own fixture.
pub const MIGRATIONS: &[(u32, Migration)] = &[];

/// Bring a canonical block to `STATE_V`, or say why it can't be (never a silent reset): a newer schema
/// than this build, or an older one with no migration path.
pub fn migrate(mut block: Value) -> Result<Value, ImportError> {
    let Some(mut v) = block
        .get("v")
        .and_then(Value::as_u64)
        .and_then(|v| u32::try_from(v).ok())
    else {
        return Err(ImportError::Bad(
            "Not a canonical state: no schema version `v`.".into(),
        ));
    };
    if v > STATE_V {
        return Err(ImportError::Bad(format!(
            "State schema v{v} was written by a newer build (this build reads up to v{STATE_V}); it can't be migrated back."
        )));
    }
    while v < STATE_V {
        let Some((_, step)) = MIGRATIONS.iter().find(|(from, _)| *from == v) else {
            return Err(ImportError::Bad(format!(
                "State schema v{v} can't be migrated to v{STATE_V} (no migration from v{v})."
            )));
        };
        block = step(block).map_err(|e| {
            ImportError::Bad(format!("State schema v{v} could not be migrated: {e}"))
        })?;
        v += 1;
        block["v"] = Value::from(v);
    }
    Ok(block)
}

/// Bumped when the derivation of rows from the record changes (so a state names how its rows were made).
pub const DERIVATION: u32 = 1;

/// A newer installed build (footer and state).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Update {
    pub version: String,
    /// `restart` in v0.10.2 (`upgrade_in_place` and `install` come later).
    pub action: String,
}

/// Rendered from a state exported against another record version and/or config.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Drift {
    /// The record version the state came from, and the record's now (equal when only the config drifted).
    pub from: i64,
    pub to: i64,
    /// What differs in the config, when it drifted (`orgs: a,b,c → a,c`).
    #[serde(default)]
    pub config: Option<String>,
    /// The classification store moved (`--allow-classify-drift`): `[from, to]` store revisions.
    #[serde(default)]
    pub classify: Option<[i64; 2]>,
    /// The shown manifest moved (`--allow-manifest-drift`): `[from, to]` head revisions (0 = gone).
    #[serde(default)]
    pub manifest: Option<[i64; 2]>,
}

/// The config a state was made under: the parts that affect rendering.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConfigPrint {
    /// Ordered `(alias, tenant id)`.
    pub orgs: Vec<(String, String)>,
    /// Ordered `(name, members)`.
    pub groups: Vec<(String, Vec<String>)>,
    /// FNV-1a 64 of the canonical JSON of the account map (Compare's `map` alignment).
    pub account_map: String,
    /// FNV-1a 64 of the canonical JSON of all three (a quick equality check; the lists say what).
    pub hash: String,
}

fn fnv(text: &str) -> String {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in text.bytes() {
        h ^= u64::from(b);
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    format!("{h:016x}")
}

impl ConfigPrint {
    pub fn of(source: &crate::model::Source) -> ConfigPrint {
        let account_map = fnv(&ownpurse_core::canon::canonical(&source.account_map));
        let hash = fnv(&ownpurse_core::canon::canonical(&serde_json::json!({
            "orgs": source.orgs, "groups": source.groups, "account_map": account_map,
        })));
        ConfigPrint {
            orgs: source.orgs.clone(),
            groups: source.groups.clone(),
            account_map,
            hash,
        }
    }

    /// The same orgs (alias → tenant), groups (name → members) and account map, in any order.
    fn same_sets(&self, other: &ConfigPrint) -> bool {
        let sorted = |mut v: Vec<(String, String)>| {
            v.sort();
            v
        };
        let gsorted = |mut v: Vec<(String, Vec<String>)>| {
            v.sort();
            v
        };
        sorted(self.orgs.clone()) == sorted(other.orgs.clone())
            && gsorted(self.groups.clone()) == gsorted(other.groups.clone())
            && self.account_map == other.account_map
    }

    /// What differs, for the refusal: `orgs: a,b,c → a,c`, other tenants, groups, account map.
    fn diff(&self, now: &ConfigPrint) -> String {
        let mut parts = Vec::new();
        let names = |v: &[(String, String)]| {
            v.iter()
                .map(|(a, _)| a.as_str())
                .collect::<Vec<_>>()
                .join(",")
        };
        if names(&self.orgs) != names(&now.orgs) {
            parts.push(format!(
                "orgs: {} → {}",
                names(&self.orgs),
                names(&now.orgs)
            ));
        }
        let moved: Vec<&str> = self
            .orgs
            .iter()
            .filter(|(a, t)| now.orgs.iter().any(|(b, u)| a == b && t != u))
            .map(|(a, _)| a.as_str())
            .collect();
        if !moved.is_empty() {
            parts.push(format!("other tenant for {}", moved.join(",")));
        }
        let g = |v: &[(String, Vec<String>)]| {
            let mut v = v.to_vec();
            v.sort();
            v.iter()
                .map(|(n, m)| format!("{n}={}", m.join("+")))
                .collect::<Vec<_>>()
                .join(",")
        };
        if g(&self.groups) != g(&now.groups) {
            parts.push(format!("groups: {} → {}", g(&self.groups), g(&now.groups)));
        }
        if self.account_map != now.account_map {
            parts.push("account_map changed".into());
        }
        parts.join("; ")
    }
}

/// Which drift an import accepts (`--allow-record-drift`, `--allow-config-drift`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AllowDrift {
    pub record: bool,
    pub config: bool,
    /// `--allow-classify-drift`: render though labels changed since the state was saved.
    pub classify: bool,
    /// `--allow-manifest-drift`: render though the shown manifest changed since the state was saved.
    pub manifest: bool,
    /// A live handover (`ui upgrade`), not a file import: the record and the store may have moved since the
    /// export (another process wrote); the view is restored on the current data, unmarked. Config drift is
    /// still refused unless `config`.
    pub live: bool,
}

/// Parts of the `Ui` that hold view states by alias themselves (a walkthrough and its view before, a
/// recording, the walkthrough list): never remapped.
const OPAQUE: [&str; 3] = ["walk", "recording", "walk_list"];

/// Rewrite every scope / org reference in a canonical JSON block: `{"Org": x}`, `{"Group": x}`, Compare's
/// `orgs`, a via chain's `org`. `org`/`group` map one reference; None drops it (scope → `all`).
fn map_refs(
    v: &mut Value,
    org: &dyn Fn(&Value) -> Option<Value>,
    group: &dyn Fn(&Value) -> Option<Value>,
) {
    match v {
        Value::Object(m) => {
            if m.len() == 1 {
                if let Some(x) = m.get("Org").cloned() {
                    *v = match org(&x) {
                        Some(y) => serde_json::json!({"Org": y}),
                        None => Value::String("All".into()),
                    };
                    return;
                }
                if let Some(x) = m.get("Group").cloned() {
                    *v = match group(&x) {
                        Some(y) => serde_json::json!({"Group": y}),
                        None => Value::String("All".into()),
                    };
                    return;
                }
            }
            // Compare (`align` + `orgs`) and a via chain (`pair` + `org`)
            if m.contains_key("align")
                && let Some(Value::Array(os)) = m.get_mut("orgs")
            {
                let kept: Vec<Value> = os.iter().filter_map(org).collect();
                *os = kept;
            }
            // the scope picker's multi-select (Compare): org indexes
            if m.contains_key("multi")
                && let Some(Value::Array(os)) = m.get_mut("chosen")
            {
                let kept: Vec<Value> = os.iter().filter_map(org).collect();
                *os = kept;
            }
            if m.contains_key("pair")
                && m.contains_key("only")
                && let Some(x) = m.get("org").cloned()
            {
                match org(&x) {
                    Some(y) => {
                        m.insert("org".into(), y);
                    }
                    None => {
                        m.insert("org".into(), Value::Null);
                    }
                }
            }
            for (k, x) in m.iter_mut() {
                // walkthroughs and recordings hold view states by alias already (their own scopes): untouched
                if OPAQUE.contains(&k.as_str()) {
                    continue;
                }
                map_refs(x, org, group);
            }
        }
        Value::Array(a) => {
            for x in a {
                map_refs(x, org, group);
            }
        }
        _ => {}
    }
}

/// Clean up after dropped references: a Compare with fewer than two orgs closes; a via chain without its
/// org ends.
fn tidy(v: &mut Value) {
    match v {
        Value::Object(m) => {
            for k in ["cmp", "via"] {
                let drop = match m.get(k) {
                    Some(Value::Object(c)) if k == "cmp" => c
                        .get("orgs")
                        .and_then(Value::as_array)
                        .is_some_and(|a| a.len() < 2),
                    Some(Value::Object(c)) => c.get("org").is_some_and(Value::is_null),
                    _ => false,
                };
                if drop {
                    m.insert(k.into(), Value::Null);
                }
            }
            for (k, x) in m.iter_mut() {
                if OPAQUE.contains(&k.as_str()) {
                    continue;
                }
                tidy(x);
            }
        }
        Value::Array(a) => a.iter_mut().for_each(tidy),
        _ => {}
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecordVersion {
    /// The integrity chain's last sequence number when the rows were derived.
    pub chain_seq: i64,
    /// `DERIVATION` of the build that derived them.
    pub derivation: u32,
    /// The classification store's revision (its last event id) when the Classify rows were derived; null
    /// without a store (v0.11.0; older states have none and skip the check).
    #[serde(default)]
    pub classify_rev: Option<i64>,
}

/// The environment the render reads.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Env {
    /// The clock the frame shows (ages, "just now").
    pub now: i64,
    /// The clock the Model was read at (today's counts, the last minute's calls).
    pub model_now: i64,
    pub offline: bool,
    pub version_label: String,
    pub profile: String,
    pub cli: Option<String>,
    pub cli_version: Option<String>,
    pub cli_shown: Option<String>,
    pub paths: crate::system::PathBase,
    pub label: Option<String>,
    pub update_available: Option<Update>,
    pub drift: Option<Drift>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Canonical {
    pub v: u32,
    pub record: RecordVersion,
    /// The config fingerprint (orgs, groups, account map) the state was made under.
    pub config: ConfigPrint,
    pub env: Env,
    pub rev: u64,
    pub ui: Ui,
    pub agent: crate::agent::AgentUi,
    /// The keys.toml the effective keymap was built from (v0.12.0; empty = the built-in table). A render
    /// from state uses it, never the file on disk, so renders reproduce without the file.
    #[serde(default)]
    pub keymap: crate::user_config::KeysFile,
    /// The render epoch of the build that wrote the state (v0.12.0; absent before: derived from
    /// `env.version_label`, epoch 0 below 0.12.0). An upgrade across epochs checks the state, not the frame.
    #[serde(default)]
    pub render_epoch: Option<u32>,
    /// The shown manifest's head revision when the state was taken (v0.13.0; the name is `ui.manifest`). A render
    /// from state refuses `manifest_changed` when the manifest moved since (`--allow-manifest-drift` renders anyway).
    #[serde(default)]
    pub manifest_rev: Option<i64>,
}

/// Why a state can't be imported.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ImportError {
    /// Not a canonical state (bad JSON, missing fields, other schema version): exit 2.
    Bad(String),
    /// Exported against another record version: exit 4 unless drift is allowed.
    RecordChanged { from: i64, to: i64 },
    /// Exported under another config (beyond a reorder): exit 4 unless config drift is allowed.
    ConfigChanged(String),
    /// The classification store moved since the state was saved (labels written): exit 4 unless
    /// classify drift is allowed.
    ClassifyChanged { from: i64, to: i64 },
    /// The shown manifest moved (or is gone) since the state was saved: exit 4 unless manifest drift is allowed.
    ManifestChanged { name: String, from: i64, to: i64 },
}

impl ImportError {
    pub fn code(&self) -> &'static str {
        match self {
            ImportError::Bad(_) => "bad_state",
            ImportError::RecordChanged { .. } => "record_changed",
            ImportError::ConfigChanged(_) => "config_changed",
            ImportError::ClassifyChanged { .. } => "classify_changed",
            ImportError::ManifestChanged { .. } => "manifest_changed",
        }
    }
    pub fn exit(&self) -> i32 {
        match self {
            ImportError::Bad(_) => 2,
            ImportError::RecordChanged { .. }
            | ImportError::ConfigChanged(_)
            | ImportError::ClassifyChanged { .. }
            | ImportError::ManifestChanged { .. } => 4,
        }
    }
    pub fn message(&self) -> String {
        match self {
            ImportError::Bad(m) => m.clone(),
            ImportError::RecordChanged { from, to } => format!(
                "The state was exported against record version {from}; the record is now at {to}. Nothing was rendered."
            ),
            ImportError::ConfigChanged(what) => format!(
                "The state was exported under another config ({what}). Nothing was rendered."
            ),
            ImportError::ClassifyChanged { from, to } => format!(
                "The classification store changed since this state was saved: rev {from} → {to}. Nothing was rendered."
            ),
            ImportError::ManifestChanged { name, from, to } => {
                if *to == 0 {
                    format!(
                        "Manifest `{name}` is gone (discarded or not in this store) since this state was saved. Nothing was rendered."
                    )
                } else {
                    format!(
                        "Manifest `{name}` changed since this state was saved: rev {from} → {to}. Nothing was rendered."
                    )
                }
            }
        }
    }
}

impl App {
    /// The canonical block.
    pub fn canonical(&self) -> Canonical {
        Canonical {
            v: STATE_V,
            record: RecordVersion {
                chain_seq: self
                    .drift
                    .as_ref()
                    .map_or(self.model.record_version, |d| d.from),
                derivation: DERIVATION,
                // a drifted state keeps the revision it was taken at (it stays drifted)
                classify_rev: self
                    .drift
                    .as_ref()
                    .and_then(|d| d.classify)
                    .map(|[from, _]| from)
                    .or_else(|| self.classify.as_ref().map(|d| d.provider.rev())),
            },
            config: ConfigPrint::of(&self.source),
            env: Env {
                now: self.now,
                model_now: self.model_now,
                offline: self.offline,
                version_label: self.version_label.clone(),
                profile: self.profile.clone(),
                cli: self.cli.as_ref().map(|p| p.display().to_string()),
                cli_version: self.cli_version.clone(),
                cli_shown: self.cli_shown.clone(),
                paths: self.paths.clone(),
                label: self.label.clone(),
                update_available: self.update_available.clone(),
                drift: self.drift.clone(),
            },
            rev: self.rev,
            ui: self.ui.clone(),
            agent: self.agent.clone(),
            keymap: self.keymap.file.clone(),
            render_epoch: Some(crate::upgrade::RENDER_EPOCH),
            manifest_rev: self.ui.manifest.as_ref().map(|_| self.manifest_head()),
        }
    }

    /// The canonical block as JSON, scopes and orgs by alias (never config indexes).
    pub fn canonical_value(&self) -> Value {
        let mut v = serde_json::to_value(self.canonical()).unwrap_or(Value::Null);
        let orgs: Vec<String> = self.source.orgs.iter().map(|(a, _)| a.clone()).collect();
        let groups: Vec<String> = self.model.groups.iter().map(|g| g.name.clone()).collect();
        let by = |names: Vec<String>| {
            move |x: &Value| {
                x.as_u64()
                    .and_then(|i| names.get(i as usize))
                    .map(|n| Value::String(n.clone()))
            }
        };
        if let Some(m) = v.as_object_mut() {
            for k in ["ui", "agent"] {
                if let Some(x) = m.get_mut(k) {
                    map_refs(x, &by(orgs.clone()), &by(groups.clone()));
                }
            }
        }
        // the scope picker's cursor is a row of a config-ordered list: also name the scope under it
        if let Some(crate::app::Overlay::Picker { query, sel, .. }) = &self.ui.overlay
            && let Some(e) = self.picker_entries(query).get(*sel)
            && let Some(p) = v
                .pointer_mut("/ui/overlay/Picker")
                .and_then(Value::as_object_mut)
        {
            p.insert("at".into(), Value::String(self.scope_label(&e.scope)));
        }
        v
    }

    /// The canonical block as core's canonical JSON (sorted keys, exact digits).
    pub fn canonical_json(&self) -> String {
        ownpurse_core::canon::canonical(&self.canonical_value())
    }

    /// A fresh app from a canonical state (plus the same source/record), rendered once at its size so its
    /// derived rows exist. `c` holds config indexes (already resolved from aliases); `config_drift` says
    /// what differed in the config when the import tolerated it.
    fn from_canonical(
        c: Canonical,
        source: crate::model::Source,
        allow: AllowDrift,
        config_drift: Option<String>,
        picker_at: Option<String>,
    ) -> Result<App, ImportError> {
        if c.v != STATE_V {
            return Err(ImportError::Bad(format!(
                "State schema v{} is not supported (this build reads v{STATE_V}).",
                c.v
            )));
        }
        // A state already rendered under config drift stays drifted (and needs the same allowance).
        let config_drift =
            config_drift.or_else(|| c.env.drift.as_ref().and_then(|d| d.config.clone()));
        if let Some(what) = &config_drift
            && !allow.config
        {
            return Err(ImportError::ConfigChanged(what.clone()));
        }
        let model = crate::model::Model::open(&source, c.env.model_now)
            .map_err(|e| ImportError::Bad(format!("Could not read the record: {e}")))?;
        let to = model.record_version;
        let from = c.env.drift.as_ref().map_or(c.record.chain_seq, |d| d.from);
        if from != to && !allow.record && !allow.live {
            return Err(ImportError::RecordChanged { from, to });
        }
        // (a config that drifted lists other lines: only the same config compares store revisions)
        let app_config_same = config_drift.is_none();
        let mut app = App::new(model, source, c.ui.theme, c.env.offline, c.env.model_now);
        // the classification store moved since the state was taken (a label written): its rows would differ
        let now_rev = app.classify.as_ref().map(|d| d.provider.rev());
        let mut classify_drift = None;
        if let (Some(then), Some(now)) = (c.record.classify_rev, now_rev)
            && then != now
            && app_config_same
        {
            if !allow.classify && !allow.live {
                return Err(ImportError::ClassifyChanged {
                    from: then,
                    to: now,
                });
            }
            if !allow.live {
                classify_drift = Some([then, now]);
            }
        }
        // live: data freshness is not drift (the view is shown on the current data, unmarked)
        let record_drift = from != to && !allow.live;
        let drift =
            (record_drift || config_drift.is_some() || classify_drift.is_some()).then_some(Drift {
                from,
                to,
                config: config_drift,
                classify: classify_drift,
                manifest: None,
            });
        app.now = c.env.now;
        app.model_now = c.env.model_now;
        app.profile = c.env.profile;
        // the build id isn't in the state: it's this build's when the state came from this version
        app.build_label = if c.env.version_label == crate::app::VERSION {
            crate::app::BUILD.map(str::to_string)
        } else {
            None
        };
        app.version_label = c.env.version_label;
        app.cli = c.env.cli.map(std::path::PathBuf::from);
        app.cli_version = c.env.cli_version;
        app.cli_shown = c.env.cli_shown;
        app.paths = c.env.paths;
        app.label = c.env.label;
        app.update_available = c.env.update_available;
        app.drift = drift;
        app.rev = c.rev;
        app.ui = c.ui;
        // the shown manifest: re-read and attached; a manifest that moved since the state was taken is drift
        let wanted = app.ui.manifest.clone();
        app.manifests.attached = None;
        app.manifest_sync();
        if let Some(name) = wanted {
            let now = app.manifest().map_or(0, |b| b.head_rev);
            let then = c.manifest_rev.unwrap_or(now);
            if now != then && !allow.live {
                if !allow.manifest {
                    return Err(ImportError::ManifestChanged {
                        name,
                        from: then,
                        to: now,
                    });
                }
                let d = app.drift.get_or_insert(Drift {
                    from: to,
                    to,
                    config: None,
                    classify: None,
                    manifest: None,
                });
                d.manifest = Some([then, now]);
                if now == 0 {
                    // gone: render as it was named (the chip says so), on main
                    app.ui.manifest = None;
                }
            }
        }
        if let Some(cmp) = app.ui.cmp.as_mut() {
            cmp.col = cmp.col.min(cmp.orgs.len());
        }
        // the picker's cursor follows its scope (a reordered config lists the scopes in another order)
        if let Some(at) = picker_at
            && let Some(crate::app::Overlay::Picker { query, .. }) = &app.ui.overlay
        {
            let entries = app.picker_entries(query);
            let found = entries.iter().position(|e| app.scope_label(&e.scope) == at);
            let last = entries.len().saturating_sub(1);
            if let Some(crate::app::Overlay::Picker { sel, .. }) = app.ui.overlay.as_mut() {
                *sel = found.unwrap_or((*sel).min(last));
            }
        }
        app.agent = c.agent;
        // the state's keymap wins over any keys.toml (a state that carries a bad one renders with defaults)
        let _ = app.set_keys(&c.keymap);
        app.agent_relog();
        app.invalidate();
        let (w, h) = app.ui.size;
        crate::snapshot::render(&mut app, w, h);
        Ok(app)
    }

    /// An app from JSON text (a full `ui state` answer, its `state`, or the canonical block), with
    /// `allow_drift` covering record drift only (config drift refused).
    pub fn from_state_json(
        text: &str,
        source: crate::model::Source,
        allow_drift: bool,
    ) -> Result<App, ImportError> {
        App::from_state_json_with(
            text,
            source,
            AllowDrift {
                record: allow_drift,
                config: false,
                classify: false,
                manifest: false,
                live: false,
            },
        )
    }

    /// `from_state_json` with both allowances: scopes resolve by alias against the current config; a pure
    /// reorder resolves exactly, any other config difference is `config_changed` unless `allow.config`.
    pub fn from_state_json_with(
        text: &str,
        source: crate::model::Source,
        allow: AllowDrift,
    ) -> Result<App, ImportError> {
        let v: Value =
            serde_json::from_str(text).map_err(|e| ImportError::Bad(format!("Not JSON: {e}")))?;
        let block = v
            .get("state")
            .and_then(|s| s.get("canonical"))
            .or_else(|| v.get("canonical"))
            .unwrap_or(&v)
            .clone();
        let mut block = migrate(block)?;
        let then: ConfigPrint = block
            .get("config")
            .cloned()
            .ok_or_else(|| ImportError::Bad("Not a canonical state: missing field `config`".into()))
            .and_then(|c| {
                serde_json::from_value(c)
                    .map_err(|e| ImportError::Bad(format!("Not a canonical state: config: {e}")))
            })?;
        let now = ConfigPrint::of(&source);
        let config_drift = if then.hash == now.hash || then.same_sets(&now) {
            None
        } else {
            Some(then.diff(&now))
        };
        if let Some(what) = &config_drift
            && !allow.config
        {
            return Err(ImportError::ConfigChanged(what.clone()));
        }
        let orgs: Vec<String> = source.orgs.iter().map(|(a, _)| a.clone()).collect();
        let groups: Vec<String> = source.groups.iter().map(|(n, _)| n.clone()).collect();
        let by = |names: Vec<String>| {
            move |x: &Value| {
                x.as_str()
                    .and_then(|a| names.iter().position(|n| n == a))
                    .map(Value::from)
            }
        };
        if let Some(m) = block.as_object_mut() {
            for k in ["ui", "agent"] {
                if let Some(x) = m.get_mut(k) {
                    map_refs(x, &by(orgs.clone()), &by(groups.clone()));
                    if config_drift.is_some() {
                        tidy(x);
                    }
                }
            }
        }
        let picker_at = block
            .pointer_mut("/ui/overlay/Picker")
            .and_then(Value::as_object_mut)
            .and_then(|p| p.remove("at"))
            .and_then(|a| a.as_str().map(str::to_string));
        let c: Canonical = serde_json::from_value(block)
            .map_err(|e| ImportError::Bad(format!("Not a canonical state: {e}")))?;
        App::from_canonical(c, source, allow, config_drift, picker_at)
    }
}

/// A `Ui` from its canonical JSON (scopes and orgs by alias, as `canonical_value` writes them), against
/// this app's config: walkthrough steps set the view this way.
pub(crate) fn ui_from_value(app: &App, mut ui: Value) -> Result<Ui, String> {
    let orgs: Vec<String> = app.source.orgs.iter().map(|(a, _)| a.clone()).collect();
    let groups: Vec<String> = app.source.groups.iter().map(|(n, _)| n.clone()).collect();
    let by = |names: Vec<String>| {
        move |x: &Value| {
            x.as_str()
                .and_then(|a| names.iter().position(|n| n == a))
                .map(Value::from)
        }
    };
    map_refs(&mut ui, &by(orgs), &by(groups));
    tidy(&mut ui);
    serde_json::from_value(ui).map_err(|e| format!("Not a view state: {e}"))
}
