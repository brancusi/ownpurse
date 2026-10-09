//! The user's own config (v0.12.0): `~/.config/ownpurse/` (or `$XDG_CONFIG_HOME/ownpurse/`), holding
//! `keys.toml` (key bindings layered over the built-in table) and `themes/*.toml` (user themes). ONE layer
//! for both: discovery, ONE OS-event watcher on the folder (no timer: V13), TOML parsed with `toml_edit`
//! (spans give every error a file:line), keep-last-valid per file, and a revision hash that goes into the
//! exported state.
//!
//! STABLE INTERFACE (Worker T consumes it; changes are additive only):
//! - [`UserConfig`] — what is loaded now: [`KeysFile`] and the themes by name, plus a revision.
//! - [`ConfigEvent`] — what a change on disk produced: `Reloaded` (with what changed) or `Invalid`.
//! - [`UserConfigWatcher::spawn`] — the watcher; it calls back with every [`ConfigEvent`] (the binary
//!   forwards them into its loop channel, as it does for `classify.toml`).
//! - [`ThemeValidator`] — T's semantic check of a theme file (expressions, `extends`, token names); a
//!   failing file is `Invalid` and the last valid version of it stays live.
//!
//! Tests never touch the real folder: [`config_dir`] takes the environment, and every test passes a temp dir.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// The keys file's name inside the config folder.
pub const KEYS_FILE: &str = "keys.toml";
/// The themes folder inside the config folder.
pub const THEMES_DIR: &str = "themes";
/// The keys file schema version this build reads.
pub const KEYS_V: i64 = 1;

/// One problem in a user config file, with its place (`themes/aram-warm.toml:9: semantic.bg: …`).
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ConfigIssue {
    /// Relative to the config folder: `keys.toml`, `themes/aram-warm.toml`.
    pub file: String,
    /// 1-based; 0 when unknown.
    pub line: usize,
    /// The dotted path of the offending entry (`bindings[2].key`), or `(syntax)` / `(file)`.
    pub path: String,
    pub message: String,
}

impl std::fmt::Display for ConfigIssue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.line > 0 {
            write!(
                f,
                "{}:{}: {}: {}",
                self.file, self.line, self.path, self.message
            )
        } else {
            write!(f, "{}: {}: {}", self.file, self.path, self.message)
        }
    }
}

/// One `[[bindings]]` entry: `key = "cmd+["`, `action = "history.back"`, `context = "global"`.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct KeyBinding {
    pub key: String,
    pub action: String,
    /// A keymap context name as `ownpurse keys` prints it (`global`, `view.ledger.l1`, …).
    pub context: String,
    /// Where it is in keys.toml (1-based).
    pub line: usize,
}

/// One `[[unbind]]` entry: a default binding to remove (`key`, `context`).
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct KeyUnbind {
    pub key: String,
    pub context: String,
    pub line: usize,
}

/// `keys.toml` as written (validated against the built-in table by the keymap layer).
#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct KeysFile {
    pub v: i64,
    pub bindings: Vec<KeyBinding>,
    pub unbind: Vec<KeyUnbind>,
    /// SHA-256 hex (first 16) of the file's text; empty when there is no file.
    pub hash: String,
    /// The name errors use: `keys.toml`, or the `tui.keys_file` path as configured.
    #[serde(default)]
    pub file: String,
}

impl KeysFile {
    /// The name its issues carry.
    pub fn name(&self) -> &str {
        if self.file.is_empty() {
            KEYS_FILE
        } else {
            &self.file
        }
    }
}

/// One user theme file, syntax-checked; its meaning is T's (`ThemeValidator`).
#[derive(Clone, Debug)]
pub struct ThemeFile {
    /// The file stem (`aram-warm`); `[meta].name` must match it (checked by T).
    pub name: String,
    /// Relative to the config folder: `themes/aram-warm.toml`.
    pub file: String,
    pub text: String,
    /// The parsed TOML, with spans (`toml_edit`): use [`line_of`] for an error's line.
    pub doc: toml_edit::Document<String>,
    /// SHA-256 hex (first 16) of `text`.
    pub hash: String,
}

impl PartialEq for ThemeFile {
    fn eq(&self, o: &ThemeFile) -> bool {
        self.file == o.file && self.hash == o.hash
    }
}

/// T's semantic check of a theme file against the others (for `extends`). Err → `Invalid`.
pub type ThemeValidator = fn(&ThemeFile, &BTreeMap<String, ThemeFile>) -> Result<(), ConfigIssue>;

/// Everything in the user's config folder, as last validly loaded.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct UserConfig {
    /// The folder (`~/.config/ownpurse`); None = no user config (built-ins only).
    pub dir: Option<PathBuf>,
    pub keys: KeysFile,
    pub themes: BTreeMap<String, ThemeFile>,
    /// A hash over every loaded file's hash: the exported `keymap.revision` / `theme.revision` derive from
    /// it, and it changes exactly when some file's valid content does.
    pub revision: String,
}

/// What changed in one reload.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Change {
    Keys,
    /// A theme added, changed or removed (by name).
    Theme(String),
}

/// The watcher's report of a change on disk.
#[derive(Clone, Debug, PartialEq)]
pub enum ConfigEvent {
    /// Valid: the new config (other files keep their last valid version) and what changed.
    Reloaded(Box<UserConfig>, Vec<Change>),
    /// A file doesn't parse or validate: its previous valid version stays live. The issue names
    /// file:line; it clears on the next valid save (a `Reloaded` mentioning that file).
    Invalid(ConfigIssue),
}

/// The config folder: `$XDG_CONFIG_HOME/ownpurse`, else `<home>/.config/ownpurse`.
pub fn config_dir(xdg_config_home: Option<&Path>, home: Option<&Path>) -> Option<PathBuf> {
    match xdg_config_home.filter(|p| p.is_absolute()) {
        Some(x) => Some(x.join("ownpurse")),
        None => home.map(|h| h.join(".config").join("ownpurse")),
    }
}

/// The folder for this process (`XDG_CONFIG_HOME`, `HOME`).
pub fn process_config_dir() -> Option<PathBuf> {
    let xdg = std::env::var_os("XDG_CONFIG_HOME").map(PathBuf::from);
    let home = std::env::var_os("HOME").map(PathBuf::from);
    config_dir(xdg.as_deref(), home.as_deref())
}

/// The 1-based line of a byte offset in `text`.
pub fn line_of(text: &str, offset: usize) -> usize {
    text[..offset.min(text.len())].matches('\n').count() + 1
}

/// The first 16 hex digits of SHA-256.
pub fn short_hash(text: &str) -> String {
    use sha2::{Digest, Sha256};
    Sha256::digest(text.as_bytes())
        .iter()
        .take(8)
        .map(|b| format!("{b:02x}"))
        .collect()
}

fn revision_of(keys: &KeysFile, themes: &BTreeMap<String, ThemeFile>) -> String {
    let mut all = format!("keys:{}", keys.hash);
    for (n, t) in themes {
        all.push_str(&format!("\ntheme:{n}:{}", t.hash));
    }
    short_hash(&all)
}

/// Parse keys.toml text into a [`KeysFile`] (structure only; the keymap layer checks the meaning).
pub fn parse_keys(text: &str) -> Result<KeysFile, ConfigIssue> {
    parse_keys_named(text, KEYS_FILE)
}

/// [`parse_keys`] for a keys file shown under another name (`tui.keys_file`).
pub fn parse_keys_named(text: &str, name: &str) -> Result<KeysFile, ConfigIssue> {
    let issue = |line: usize, path: &str, message: String| ConfigIssue {
        file: name.into(),
        line,
        path: path.into(),
        message,
    };
    let doc = toml_edit::Document::parse(text.to_string()).map_err(|e| {
        let line = e.span().map_or(0, |s| line_of(text, s.start));
        issue(line, "(syntax)", e.message().to_string())
    })?;
    let at = |span: Option<std::ops::Range<usize>>| span.map_or(0, |s| line_of(text, s.start));
    let v = match doc.get("v") {
        Some(item) => item
            .as_integer()
            .ok_or_else(|| issue(at(item.span()), "v", "expected an integer (1)".into()))?,
        None => return Err(issue(1, "v", "missing `v = 1`".into())),
    };
    if v != KEYS_V {
        return Err(issue(
            at(doc.get("v").and_then(|i| i.span())),
            "v",
            format!("version {v} is not supported (this build reads v = {KEYS_V})"),
        ));
    }
    let mut out = KeysFile {
        v,
        hash: short_hash(text),
        file: if name == KEYS_FILE {
            String::new()
        } else {
            name.into()
        },
        ..KeysFile::default()
    };
    for (k, item) in doc.iter() {
        if !matches!(k, "v" | "bindings" | "unbind") {
            return Err(issue(
                at(item.span()),
                k,
                "unknown key (keys.toml has v, [[bindings]] and [[unbind]])".into(),
            ));
        }
    }
    let tables = |name: &str| -> Result<Vec<(usize, toml_edit::Table)>, ConfigIssue> {
        match doc.get(name) {
            None => Ok(vec![]),
            Some(toml_edit::Item::ArrayOfTables(a)) => {
                Ok(a.iter().map(|t| (at(t.span()), t.clone())).collect())
            }
            Some(other) => Err(issue(
                at(other.span()),
                name,
                format!("expected [[{name}]] tables"),
            )),
        }
    };
    let field =
        |t: &toml_edit::Table, line: usize, path: &str, k: &str| -> Result<String, ConfigIssue> {
            match t.get(k) {
                Some(i) => i.as_str().map(str::to_string).ok_or_else(|| {
                    issue(
                        at(i.span()).max(line),
                        &format!("{path}.{k}"),
                        "expected a string".into(),
                    )
                }),
                None => Err(issue(
                    line,
                    &format!("{path}.{k}"),
                    format!("missing `{k}`"),
                )),
            }
        };
    for (i, (line, t)) in tables("bindings")?.into_iter().enumerate() {
        let p = format!("bindings[{i}]");
        let context = match t.get("context") {
            None => "global".to_string(),
            Some(_) => field(&t, line, &p, "context")?,
        };
        out.bindings.push(KeyBinding {
            key: field(&t, line, &p, "key")?,
            action: field(&t, line, &p, "action")?,
            context,
            line,
        });
    }
    for (i, (line, t)) in tables("unbind")?.into_iter().enumerate() {
        let p = format!("unbind[{i}]");
        let context = match t.get("context") {
            None => "global".to_string(),
            Some(_) => field(&t, line, &p, "context")?,
        };
        out.unbind.push(KeyUnbind {
            key: field(&t, line, &p, "key")?,
            context,
            line,
        });
    }
    Ok(out)
}

/// Parse one theme file's TOML (syntax only).
pub fn parse_theme(file: &str, name: &str, text: &str) -> Result<ThemeFile, ConfigIssue> {
    let doc = toml_edit::Document::parse(text.to_string()).map_err(|e| ConfigIssue {
        file: file.into(),
        line: e.span().map_or(0, |s| line_of(text, s.start)),
        path: "(syntax)".into(),
        message: e.message().to_string(),
    })?;
    Ok(ThemeFile {
        name: name.into(),
        file: file.into(),
        text: text.into(),
        doc,
        hash: short_hash(text),
    })
}

/// Validates a keys file against the keymap (unknown action, bad key spec, conflicts, network actions,
/// confirm-band keys). Set by the keymap layer; `None` = structure only.
pub type KeysValidator = fn(&KeysFile) -> Result<(), ConfigIssue>;

/// The loader: the current valid config, re-read file by file on each change (keep-last-valid).
pub struct Loader {
    pub dir: PathBuf,
    /// keys.toml, or the `tui.keys_file` setting's file.
    pub keys_path: PathBuf,
    /// How errors name it (`keys.toml`, or the setting as written).
    pub keys_name: String,
    pub keys_validator: Option<KeysValidator>,
    pub theme_validator: Option<ThemeValidator>,
    pub current: UserConfig,
    /// The issue per file that is currently invalid (cleared by its next valid save).
    pub invalid: BTreeMap<String, ConfigIssue>,
}

impl Loader {
    pub fn new(
        dir: PathBuf,
        keys_validator: Option<KeysValidator>,
        theme_validator: Option<ThemeValidator>,
    ) -> Loader {
        Loader {
            keys_path: dir.join(KEYS_FILE),
            keys_name: KEYS_FILE.into(),
            current: UserConfig {
                dir: Some(dir.clone()),
                revision: revision_of(&KeysFile::default(), &BTreeMap::new()),
                ..UserConfig::default()
            },
            dir,
            keys_validator,
            theme_validator,
            invalid: BTreeMap::new(),
        }
    }

    /// Read keys from another file (`tui.keys_file`), named in errors as `shown`; the watcher watches its
    /// folder too.
    pub fn with_keys_file(mut self, path: PathBuf, shown: String) -> Loader {
        self.keys_path = path;
        self.keys_name = shown;
        self
    }

    /// Re-read everything; returns the events (one `Reloaded` if any valid file changed, plus an
    /// `Invalid` per newly invalid file). Unchanged files produce nothing (V13: an echo draws nothing).
    pub fn reload(&mut self) -> Vec<ConfigEvent> {
        let mut changes = Vec::new();
        let mut events = Vec::new();
        let mut next = self.current.clone();
        // keys.toml
        let kname = self.keys_name.clone();
        let keys = match std::fs::read_to_string(&self.keys_path) {
            Err(_) => Ok(KeysFile::default()),
            Ok(text) => parse_keys_named(&text, &kname)
                .and_then(|k| self.keys_validator.map_or(Ok(()), |f| f(&k)).map(|_| k)),
        };
        match keys {
            Ok(k) => {
                let was_invalid = self.invalid.remove(&kname).is_some();
                if k != next.keys {
                    next.keys = k;
                    changes.push(Change::Keys);
                } else if was_invalid {
                    changes.push(Change::Keys);
                }
            }
            Err(i) => {
                if self.invalid.get(&kname) != Some(&i) {
                    self.invalid.insert(kname.clone(), i.clone());
                    events.push(ConfigEvent::Invalid(i));
                }
            }
        }
        // themes/*.toml
        let tdir = self.dir.join(THEMES_DIR);
        let mut found: BTreeMap<String, String> = BTreeMap::new();
        if let Ok(rd) = std::fs::read_dir(&tdir) {
            for e in rd.flatten() {
                let p = e.path();
                if p.extension().is_some_and(|x| x == "toml")
                    && let (Some(stem), Ok(text)) = (
                        p.file_stem().and_then(|s| s.to_str()),
                        std::fs::read_to_string(&p),
                    )
                {
                    found.insert(stem.to_string(), text);
                }
            }
        }
        let removed: Vec<String> = next
            .themes
            .keys()
            .filter(|n| !found.contains_key(*n))
            .cloned()
            .collect();
        for n in removed {
            next.themes.remove(&n);
            self.invalid.remove(&format!("{THEMES_DIR}/{n}.toml"));
            changes.push(Change::Theme(n));
        }
        for (name, text) in &found {
            let file = format!("{THEMES_DIR}/{name}.toml");
            let parsed = parse_theme(&file, name, text).and_then(|t| {
                self.theme_validator
                    .map_or(Ok(()), |f| f(&t, &next.themes))
                    .map(|_| t)
            });
            match parsed {
                Ok(t) => {
                    let was_invalid = self.invalid.remove(&file).is_some();
                    if next.themes.get(name) != Some(&t) || was_invalid {
                        next.themes.insert(name.clone(), t);
                        changes.push(Change::Theme(name.clone()));
                    }
                }
                Err(i) => {
                    if self.invalid.get(&file) != Some(&i) {
                        self.invalid.insert(file, i.clone());
                        events.push(ConfigEvent::Invalid(i));
                    }
                }
            }
        }
        if !changes.is_empty() {
            next.revision = revision_of(&next.keys, &next.themes);
            self.current = next.clone();
            events.insert(0, ConfigEvent::Reloaded(Box::new(next), changes));
        }
        events
    }
}

/// The watcher: OS file events (FSEvents / inotify) on the config folder and its themes folder, filtered
/// to `keys.toml` and `themes/*.toml`, each burst re-read by the [`Loader`] on the watcher's own thread.
/// No timer and no polling. Dropping it stops watching.
pub struct UserConfigWatcher {
    _watcher: notify::RecommendedWatcher,
}

impl UserConfigWatcher {
    /// Watch `loader.dir` (created when missing, so a first keys.toml is seen); every event the loader
    /// produces goes to `send`. Returns None when the OS watcher can't start (the TUI then runs on what it
    /// loaded at start).
    pub fn spawn(
        mut loader: Loader,
        send: impl Fn(ConfigEvent) + Send + 'static,
    ) -> Option<UserConfigWatcher> {
        use notify::{EventKind, RecursiveMode, Watcher};
        std::fs::create_dir_all(&loader.dir).ok()?;
        // OS events carry canonical paths (macOS: /private/var/… for /var/…): compare against those
        let dir = std::fs::canonicalize(&loader.dir).ok()?;
        // keys.toml, or `tui.keys_file` (its folder is watched too when it lives elsewhere)
        let keys_dir = loader
            .keys_path
            .parent()
            .and_then(|p| std::fs::canonicalize(p).ok());
        let keys = match (&keys_dir, loader.keys_path.file_name()) {
            (Some(d), Some(n)) => d.join(n),
            _ => dir.join(KEYS_FILE),
        };
        let themes = dir.join(THEMES_DIR);
        let canon = |p: &Path| -> PathBuf {
            match (
                p.parent().and_then(|d| std::fs::canonicalize(d).ok()),
                p.file_name(),
            ) {
                (Some(d), Some(n)) => d.join(n),
                _ => p.to_path_buf(),
            }
        };
        let lock = std::sync::Mutex::new(());
        let mut w = notify::recommended_watcher(move |res: notify::Result<notify::Event>| {
            let Ok(ev) = res else { return };
            if matches!(ev.kind, EventKind::Access(_)) {
                return;
            }
            let ours = ev.paths.iter().map(|p| canon(p)).any(|p| {
                p == keys
                    || p == themes
                    || (p.parent() == Some(themes.as_path())
                        && p.extension().is_some_and(|x| x == "toml"))
            });
            if !ours {
                return;
            }
            let _g = lock.lock();
            for e in loader.reload() {
                send(e);
            }
        })
        .ok()?;
        // the folder itself (an editor's atomic save replaces keys.toml) and everything under it (themes/)
        w.watch(&dir, RecursiveMode::Recursive).ok()?;
        if let Some(kd) = keys_dir.filter(|kd| !kd.starts_with(&dir)) {
            w.watch(&kd, RecursiveMode::NonRecursive).ok()?;
        }
        Some(UserConfigWatcher { _watcher: w })
    }
}
