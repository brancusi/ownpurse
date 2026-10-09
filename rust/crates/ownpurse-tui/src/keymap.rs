//! The one keymap (spec §6): the kit's `BINDINGS` / `ACTIONS` table drives dispatch, the footer, help,
//! the palette and the `keys` export. Nothing else binds keys.

pub use crate::keymap_table::{ACTIONS, Action, BINDINGS, Binding, Ctx, When};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Key {
    pub code: KeyCode,
    pub mods: KeyModifiers,
}

const MODS: KeyModifiers = KeyModifiers::CONTROL
    .union(KeyModifiers::ALT)
    .union(KeyModifiers::SUPER)
    .union(KeyModifiers::SHIFT);

impl Key {
    /// The event as the table compares it (exact matching: `d` is only bare `d`). Shift lives in the
    /// character for printable keys (`G`, `!`, `?`); BackTab is `shift-tab`.
    pub fn of(k: &KeyEvent) -> Key {
        let mut mods = k.modifiers & MODS;
        let mut code = k.code;
        if code == KeyCode::BackTab {
            code = KeyCode::Tab;
            mods |= KeyModifiers::SHIFT;
        }
        if let KeyCode::Char(c) = code {
            if mods.contains(KeyModifiers::SHIFT) && c.is_ascii_lowercase() {
                code = KeyCode::Char(c.to_ascii_uppercase());
            }
            mods.remove(KeyModifiers::SHIFT);
            if mods.contains(KeyModifiers::CONTROL) {
                code = KeyCode::Char(c.to_ascii_lowercase());
            }
        }
        Key { code, mods }
    }

    /// The kit notation: `q`, `G`, `!`, `space`, `ctrl-p`, `shift-tab`, `enter`, `pagedown`.
    pub fn parse(s: &str) -> Option<Key> {
        let mut mods = KeyModifiers::NONE;
        let mut rest = s;
        // `ctrl-p` (the table) or `ctrl+p` (keys.toml); `cmd` = `super` (⌘ on macOS)
        'mods: loop {
            for (name, m) in [
                ("ctrl", KeyModifiers::CONTROL),
                ("alt", KeyModifiers::ALT),
                ("shift", KeyModifiers::SHIFT),
                ("cmd", KeyModifiers::SUPER),
                ("super", KeyModifiers::SUPER),
            ] {
                for sep in ['-', '+'] {
                    if let Some(r) = rest
                        .strip_prefix(name)
                        .and_then(|r| r.strip_prefix(sep))
                        .filter(|r| !r.is_empty())
                    {
                        mods |= m;
                        rest = r;
                        continue 'mods;
                    }
                }
            }
            break;
        }
        let code = match rest {
            "space" => KeyCode::Char(' '),
            "enter" => KeyCode::Enter,
            "esc" => KeyCode::Esc,
            "tab" => KeyCode::Tab,
            "backspace" => KeyCode::Backspace,
            "up" => KeyCode::Up,
            "down" => KeyCode::Down,
            "left" => KeyCode::Left,
            "right" => KeyCode::Right,
            "home" => KeyCode::Home,
            "end" => KeyCode::End,
            "pageup" => KeyCode::PageUp,
            "pagedown" => KeyCode::PageDown,
            _ if rest.chars().count() == 1 => KeyCode::Char(rest.chars().next()?),
            _ => return None,
        };
        let mut k = Key { code, mods };
        if let KeyCode::Char(c) = k.code
            && k.mods.contains(KeyModifiers::SHIFT)
        {
            k.code = KeyCode::Char(c.to_ascii_uppercase());
            k.mods.remove(KeyModifiers::SHIFT);
        }
        Some(k)
    }

    /// As keys.toml spells it: `cmd+[`, `ctrl+p`, `shift+tab`, `enter`, `G`.
    pub fn spec(&self) -> String {
        let mut s = String::new();
        for (m, name) in [
            (KeyModifiers::CONTROL, "ctrl+"),
            (KeyModifiers::ALT, "alt+"),
            (KeyModifiers::SHIFT, "shift+"),
            (KeyModifiers::SUPER, "cmd+"),
        ] {
            if self.mods.contains(m) {
                s.push_str(name);
            }
        }
        s.push_str(&match self.code {
            KeyCode::Char(' ') => "space".into(),
            KeyCode::Char(c) => c.to_string(),
            KeyCode::Enter => "enter".into(),
            KeyCode::Esc => "esc".into(),
            KeyCode::Tab => "tab".into(),
            KeyCode::Backspace => "backspace".into(),
            KeyCode::Up => "up".into(),
            KeyCode::Down => "down".into(),
            KeyCode::Left => "left".into(),
            KeyCode::Right => "right".into(),
            KeyCode::Home => "home".into(),
            KeyCode::End => "end".into(),
            KeyCode::PageUp => "pageup".into(),
            KeyCode::PageDown => "pagedown".into(),
            other => format!("{other:?}").to_lowercase(),
        });
        s
    }

    /// The event that would produce this key (for key scripts and tests).
    pub fn event(&self) -> KeyEvent {
        if self.code == KeyCode::Tab && self.mods.contains(KeyModifiers::SHIFT) {
            return KeyEvent::new(KeyCode::BackTab, KeyModifiers::SHIFT);
        }
        KeyEvent::new(self.code, self.mods)
    }

    /// How hints show it: `⌃P`, `⇧Tab`, `Enter`, `↑`, `G`, `space`.
    pub fn display(&self) -> String {
        let mut s = String::new();
        if self.mods.contains(KeyModifiers::CONTROL) {
            s.push('⌃');
        }
        if self.mods.contains(KeyModifiers::ALT) {
            s.push('⌥');
        }
        if self.mods.contains(KeyModifiers::SHIFT) {
            s.push('⇧');
        }
        if self.mods.contains(KeyModifiers::SUPER) {
            s.push('⌘');
        }
        let chord = !self.mods.is_empty();
        s.push_str(&match self.code {
            KeyCode::Char(' ') => "space".into(),
            KeyCode::Char(c) if chord => c.to_uppercase().to_string(),
            KeyCode::Char(c) => c.to_string(),
            KeyCode::Enter => "Enter".into(),
            KeyCode::Esc => "Esc".into(),
            KeyCode::Tab => "Tab".into(),
            KeyCode::Backspace => "⌫".into(),
            KeyCode::Up => "↑".into(),
            KeyCode::Down => "↓".into(),
            KeyCode::Left => "←".into(),
            KeyCode::Right => "→".into(),
            KeyCode::Home => "Home".into(),
            KeyCode::End => "End".into(),
            KeyCode::PageUp => "PgUp".into(),
            KeyCode::PageDown => "PgDn".into(),
            other => format!("{other:?}"),
        });
        s
    }
}

/// Contexts whose input is sealed: an unbound key types (filters, prompts) or does nothing.
pub fn sealed(c: Ctx) -> bool {
    matches!(
        c,
        Ctx::Confirm
            | Ctx::Picker
            | Ctx::Palette
            | Ctx::Help
            | Ctx::PromptFilter
            | Ctx::PromptAsOf
            | Ctx::PromptRange
            | Ctx::ClassifyProject
            | Ctx::ClassifyPicker
            | Ctx::ClassifyNote
            | Ctx::ClassifyBulk
            | Ctx::ClassifyEditor
            | Ctx::ClassifyDiff
            | Ctx::ManifestPrompt
    )
}

/// Contexts where an unbound printable key is typed into a field.
pub fn types_text(c: Ctx) -> bool {
    matches!(
        c,
        Ctx::Picker
            | Ctx::Palette
            | Ctx::PromptFilter
            | Ctx::PromptAsOf
            | Ctx::PromptRange
            | Ctx::ClassifyPicker
            | Ctx::ClassifyNote
            | Ctx::ClassifyEditor
            | Ctx::ManifestPrompt
    )
}

pub fn action(id: &str) -> Option<&'static Action> {
    ACTIONS.iter().find(|a| a.id == id)
}

pub fn is_network(id: &str) -> bool {
    action(id).is_some_and(|a| a.network)
}

/// One key of an effective binding: its spec as written, the parsed key, and where it came from.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BoundKey {
    pub spec: String,
    pub key: Key,
    /// The keys.toml line it came from (None = the built-in table).
    pub from: Option<usize>,
}

/// One binding of the effective keymap (the built-in table with keys.toml layered on).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Bound {
    pub keys: Vec<BoundKey>,
    pub action: &'static str,
    pub ctx: Ctx,
    pub when: When,
    pub label: &'static str,
    pub group: &'static str,
    pub footer: Option<u8>,
}

/// The effective keymap: what dispatch, the footer, help, the palette and `keys` read.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Keymap {
    pub bindings: Vec<Bound>,
    /// The keys.toml content hash it was built from (None = the built-in table alone).
    pub revision: Option<String>,
    /// Keys added or removed by keys.toml.
    pub overrides: usize,
    /// The keys.toml it was built from (exported in the state, so a render needs no file).
    pub file: crate::user_config::KeysFile,
}

impl Keymap {
    /// The built-in table.
    pub fn defaults() -> Keymap {
        Keymap {
            bindings: BINDINGS
                .iter()
                .map(|b| Bound {
                    keys: b
                        .keys
                        .iter()
                        .filter_map(|k| {
                            Key::parse(k).map(|key| BoundKey {
                                spec: (*k).to_string(),
                                key,
                                from: None,
                            })
                        })
                        .collect(),
                    action: b.action,
                    ctx: b.ctx,
                    when: b.when,
                    label: b.label,
                    group: b.group,
                    footer: b.footer,
                })
                .collect(),
            revision: None,
            overrides: 0,
            file: Default::default(),
        }
    }

    /// The built-in table with `file` layered on: `[[unbind]]` removes a default key from its context;
    /// `[[bindings]]` adds a key to the action's binding in that context (or a new binding that inherits the
    /// action's label, group, `when` and, when it replaces an unbound default, its footer place). Refused,
    /// naming the keys.toml line: an unknown action or context, a bad key spec, a network action, the
    /// confirm band's keys, an unbind of a key that isn't bound there, and any conflict the built-in table's
    /// rules forbid (one key, two meanings in a layer or across the stack).
    pub fn build(
        file: &crate::user_config::KeysFile,
    ) -> Result<Keymap, crate::user_config::ConfigIssue> {
        let issue = |line: usize, path: String, message: String| crate::user_config::ConfigIssue {
            file: file.name().to_string(),
            line,
            path,
            message,
        };
        let mut km = Keymap::defaults();
        let mut removed: Vec<Bound> = Vec::new();
        for (i, u) in file.unbind.iter().enumerate() {
            let path = format!("unbind[{i}]");
            let ctx = ctx_of_name(&u.context)
                .ok_or_else(|| issue(u.line, format!("{path}.context"), unknown_ctx(&u.context)))?;
            let key = Key::parse(&u.key)
                .ok_or_else(|| issue(u.line, format!("{path}.key"), bad_key(&u.key)))?;
            if ctx == Ctx::Confirm {
                return Err(issue(
                    u.line,
                    format!("{path}.context"),
                    CONFIRM_SEALED.into(),
                ));
            }
            let Some(b) = km
                .bindings
                .iter_mut()
                .find(|b| b.ctx == ctx && b.keys.iter().any(|k| k.key == key))
            else {
                return Err(issue(
                    u.line,
                    format!("{path}.key"),
                    format!("`{}` is not bound in {}", u.key, u.context),
                ));
            };
            b.keys.retain(|k| k.key != key);
            km.overrides += 1;
            if b.keys.is_empty() {
                removed.push(b.clone());
            }
        }
        km.bindings.retain(|b| !b.keys.is_empty());
        for (i, u) in file.bindings.iter().enumerate() {
            let path = format!("bindings[{i}]");
            let ctx = ctx_of_name(&u.context)
                .ok_or_else(|| issue(u.line, format!("{path}.context"), unknown_ctx(&u.context)))?;
            let key = Key::parse(&u.key)
                .ok_or_else(|| issue(u.line, format!("{path}.key"), bad_key(&u.key)))?;
            if ctx == Ctx::Confirm || u.action.starts_with("confirm.") {
                return Err(issue(
                    u.line,
                    format!("{path}.action"),
                    CONFIRM_SEALED.into(),
                ));
            }
            if is_network(&u.action) {
                return Err(issue(
                    u.line,
                    format!("{path}.action"),
                    format!(
                        "`{}` reaches Xero: network actions have no keys (V3); it stays in the palette",
                        u.action
                    ),
                ));
            }
            let Some(action) = known_action(&u.action) else {
                return Err(issue(
                    u.line,
                    format!("{path}.action"),
                    format!(
                        "unknown action `{}`{}",
                        u.action,
                        nearest_action(&u.action)
                            .map(|a| format!(" (did you mean `{a}`?)"))
                            .unwrap_or_default()
                    ),
                ));
            };
            let bk = BoundKey {
                spec: u.key.clone(),
                key,
                from: Some(u.line),
            };
            km.overrides += 1;
            if let Some(b) = km
                .bindings
                .iter_mut()
                .find(|b| b.ctx == ctx && b.action == action)
            {
                if !b.keys.iter().any(|k| k.key == key) {
                    b.keys.push(bk);
                }
                continue;
            }
            let like = removed
                .iter()
                .find(|b| b.ctx == ctx && b.action == action)
                .cloned();
            let any = BINDINGS.iter().find(|b| b.action == action);
            km.bindings.push(Bound {
                keys: vec![bk],
                action,
                ctx,
                when: like.as_ref().map_or_else(
                    || {
                        any.map_or(When::Always, |b| {
                            if b.ctx == ctx { b.when } else { When::Always }
                        })
                    },
                    |b| b.when,
                ),
                label: like
                    .as_ref()
                    .map(|b| b.label)
                    .or(any.map(|b| b.label))
                    .or(crate::keymap::action(action).map(|a| a.label))
                    .unwrap_or(action),
                group: like
                    .as_ref()
                    .map(|b| b.group)
                    .or(any.map(|b| b.group))
                    .unwrap_or("Custom"),
                footer: like.and_then(|b| b.footer),
            });
        }
        let problems = conflicts_in(&km);
        if let Some((line, msg)) = problems.into_iter().next() {
            return Err(issue(line.unwrap_or(0), "bindings".into(), msg));
        }
        km.revision = Some(file.hash.clone()).filter(|h| !h.is_empty());
        km.file = file.clone();
        Ok(km)
    }

    /// Any cmd (⌘) binding (the "no keyboard enhancement" notice is shown only then).
    pub fn has_cmd(&self) -> bool {
        self.bindings
            .iter()
            .flat_map(|b| &b.keys)
            .any(|k| k.key.mods.contains(KeyModifiers::SUPER))
    }
}

const CONFIRM_SEALED: &str =
    "the confirm band's keys are fixed (focus + Enter, Esc; V3) and can't be changed";

fn unknown_ctx(c: &str) -> String {
    format!(
        "unknown context `{c}` (one of: {})",
        ALL_CTX
            .iter()
            .map(|c| ctx_name(*c))
            .collect::<Vec<_>>()
            .join(", ")
    )
}

fn bad_key(k: &str) -> String {
    format!(
        "bad key `{k}`: modifiers ctrl, alt, shift, cmd (or super) joined with + to a key: a character, space, enter, esc, tab, backspace, up, down, left, right, home, end, pageup, pagedown"
    )
}

/// Every context, in table order (for names and messages).
pub const ALL_CTX: [Ctx; 35] = [
    Ctx::Global,
    Ctx::Overview,
    Ctx::LedgerL1,
    Ctx::LedgerL2,
    Ctx::LedgerL3,
    Ctx::IcL1,
    Ctx::IcL2,
    Ctx::IcL3,
    Ctx::CompareL1,
    Ctx::CompareL2,
    Ctx::Combined,
    Ctx::Activity,
    Ctx::System,
    Ctx::Confirm,
    Ctx::Picker,
    Ctx::Palette,
    Ctx::Help,
    Ctx::PromptFilter,
    Ctx::PromptAsOf,
    Ctx::PromptRange,
    Ctx::ClassifyMode,
    Ctx::ClassifyQueue,
    Ctx::ClassifyProject,
    Ctx::ClassifyPicker,
    Ctx::ClassifyNote,
    Ctx::ClassifyBulk,
    Ctx::ClassifyCatalog,
    Ctx::ClassifyEditor,
    Ctx::ClassifyDiff,
    Ctx::ClassifyGroups,
    Ctx::ManifestMode,
    Ctx::ManifestEntries,
    Ctx::ManifestIssues,
    Ctx::ManifestDiff,
    Ctx::ManifestPrompt,
];

/// A context by the name `keys` prints (`global`, `view.ledger.l1`, …).
pub fn ctx_of_name(name: &str) -> Option<Ctx> {
    ALL_CTX.iter().copied().find(|c| ctx_name(*c) == name)
}

/// An action id the table or the palette knows (as its static string).
pub fn known_action(id: &str) -> Option<&'static str> {
    BINDINGS
        .iter()
        .map(|b| b.action)
        .chain(ACTIONS.iter().map(|a| a.id))
        .find(|a| *a == id)
}

/// The closest known action by edit distance (≤ 3), for "did you mean".
fn nearest_action(id: &str) -> Option<&'static str> {
    fn dist(a: &str, b: &str) -> usize {
        let b: Vec<char> = b.chars().collect();
        let mut prev: Vec<usize> = (0..=b.len()).collect();
        for (i, ca) in a.chars().enumerate() {
            let mut cur = vec![i + 1];
            for (j, cb) in b.iter().enumerate() {
                cur.push(
                    (prev[j] + usize::from(ca != *cb))
                        .min(prev[j + 1] + 1)
                        .min(cur[j] + 1),
                );
            }
            prev = cur;
        }
        prev[b.len()]
    }
    BINDINGS
        .iter()
        .map(|b| b.action)
        .chain(ACTIONS.iter().map(|a| a.id))
        .map(|a| (dist(id, a), a))
        .filter(|(d, _)| *d <= 3)
        .min_by_key(|(d, _)| *d)
        .map(|(_, a)| a)
}

/// The built-in keymap, built once.
pub fn default_map() -> &'static Keymap {
    static M: std::sync::OnceLock<Keymap> = std::sync::OnceLock::new();
    M.get_or_init(Keymap::defaults)
}

/// The binding a key resolves to in a context stack (top first); `holds` decides `when`.
pub fn lookup_in<'k>(
    km: &'k Keymap,
    ctxs: &[Ctx],
    key: Key,
    holds: &dyn Fn(When) -> bool,
) -> Option<&'k Bound> {
    for &c in ctxs {
        for b in km.bindings.iter().filter(|b| b.ctx == c) {
            if b.keys.iter().any(|k| k.key == key) && holds(b.when) {
                return Some(b);
            }
        }
        if sealed(c) {
            break;
        }
    }
    None
}

pub fn lookup(ctxs: &[Ctx], key: Key, holds: &dyn Fn(When) -> bool) -> Option<&'static Bound> {
    lookup_in(default_map(), ctxs, key, holds)
}

/// The current keys of an action in a context stack, as hints show them.
pub fn keys_for_in(
    km: &Keymap,
    ctxs: &[Ctx],
    action: &str,
    holds: &dyn Fn(When) -> bool,
) -> Option<String> {
    ctxs.iter().find_map(|&c| {
        km.bindings
            .iter()
            .find(|b| b.ctx == c && b.action == action && holds(b.when))
            .and_then(|b| b.keys.first())
            .map(|k| k.key.display())
    })
}

pub fn keys_for(ctxs: &[Ctx], action: &str, holds: &dyn Fn(When) -> bool) -> Option<String> {
    keys_for_in(default_map(), ctxs, action, holds)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Hint {
    pub keys: String,
    pub label: &'static str,
    pub action: &'static str,
    pub rank: u8,
    pub global: bool,
}

/// Footer hints, best first (spec §6): ranked bindings from `ctxs` whose `when` holds and that a higher
/// context doesn't shadow; bindings sharing a rank and a label merge (`↑↓ move`).
pub fn footer_in(km: &Keymap, ctxs: &[Ctx], holds: &dyn Fn(When) -> bool) -> Vec<Hint> {
    let mut seen: Vec<Key> = Vec::new();
    let mut out: Vec<(usize, Hint)> = Vec::new();
    for &c in ctxs {
        for b in km.bindings.iter().filter(|b| b.ctx == c) {
            if !holds(b.when) {
                continue;
            }
            let keys: Vec<Key> = b.keys.iter().map(|k| k.key).collect();
            let Some(first) = keys.first().copied() else {
                continue;
            };
            if seen.contains(&first) {
                continue;
            }
            seen.extend(keys.iter().copied());
            let Some(rank) = b.footer else { continue };
            let shown = first.display();
            if let Some((_, h)) = out
                .iter_mut()
                .find(|(_, h)| h.rank == rank && h.label == b.label)
            {
                h.keys.push_str(&shown);
                continue;
            }
            let order = out.len();
            out.push((
                order,
                Hint {
                    keys: shown,
                    label: b.label,
                    action: b.action,
                    rank,
                    global: c == Ctx::Global,
                },
            ));
        }
    }
    out.sort_by_key(|(o, h)| (h.rank, *o));
    out.into_iter().map(|(_, h)| h).collect()
}

pub fn footer(ctxs: &[Ctx], holds: &dyn Fn(When) -> bool) -> Vec<Hint> {
    footer_in(default_map(), ctxs, holds)
}

/// Help (`?`): the stack's live bindings by group, view contexts first, then global. A key from keys.toml
/// carries a `*`.
pub fn help_in<'k>(
    km: &'k Keymap,
    ctxs: &[Ctx],
    holds: &dyn Fn(When) -> bool,
) -> Vec<(&'static str, Vec<(String, &'k Bound)>)> {
    let mut groups: Vec<(&'static str, Vec<(String, &'k Bound)>)> = Vec::new();
    let mut seen: Vec<Key> = Vec::new();
    for &c in ctxs {
        for b in km.bindings.iter().filter(|b| b.ctx == c) {
            if !holds(b.when) {
                continue;
            }
            let keys: Vec<&BoundKey> = b.keys.iter().filter(|k| !seen.contains(&k.key)).collect();
            if keys.is_empty() {
                continue;
            }
            seen.extend(keys.iter().map(|k| k.key));
            let shown = keys
                .iter()
                .map(|k| {
                    if k.from.is_some() {
                        format!("{}*", k.key.display())
                    } else {
                        k.key.display()
                    }
                })
                .collect::<Vec<_>>()
                .join(" ");
            match groups.iter_mut().find(|(g, _)| *g == b.group) {
                Some((_, rows)) => rows.push((shown, b)),
                None => groups.push((b.group, vec![(shown, b)])),
            }
        }
    }
    groups
}

pub fn help(
    ctxs: &[Ctx],
    holds: &dyn Fn(When) -> bool,
) -> Vec<(&'static str, Vec<(String, &'static Bound)>)> {
    help_in(default_map(), ctxs, holds)
}

pub fn ctx_name(c: Ctx) -> &'static str {
    match c {
        Ctx::Global => "global",
        Ctx::Overview => "view.overview",
        Ctx::LedgerL1 => "view.ledger.l1",
        Ctx::LedgerL2 => "view.ledger.l2",
        Ctx::LedgerL3 => "view.ledger.l3",
        Ctx::IcL1 => "view.ic.l1",
        Ctx::IcL2 => "view.ic.l2",
        Ctx::IcL3 => "view.ic.l3",
        Ctx::CompareL1 => "view.compare.l1",
        Ctx::CompareL2 => "view.compare.l2",
        Ctx::Combined => "view.combined",
        Ctx::Activity => "view.activity",
        Ctx::System => "view.system",
        Ctx::Confirm => "confirm",
        Ctx::Picker => "overlay.picker",
        Ctx::Palette => "overlay.palette",
        Ctx::Help => "overlay.help",
        Ctx::PromptFilter => "prompt.filter",
        Ctx::PromptAsOf => "prompt.as_of",
        Ctx::PromptRange => "prompt.range",
        Ctx::ClassifyMode => "view.classify",
        Ctx::ClassifyQueue => "view.classify.queue",
        Ctx::ClassifyProject => "view.classify.queue.project",
        Ctx::ClassifyPicker => "view.classify.queue.picker",
        Ctx::ClassifyNote => "view.classify.queue.note",
        Ctx::ClassifyBulk => "view.classify.queue.bulk",
        Ctx::ClassifyCatalog => "view.classify.catalog",
        Ctx::ClassifyEditor => "view.classify.catalog.editor",
        Ctx::ClassifyDiff => "view.classify.catalog.diff",
        Ctx::ClassifyGroups => "view.classify.groups",
        Ctx::Viz => "view.viz",
        Ctx::ManifestMode => "view.manifest",
        Ctx::ManifestEntries => "view.manifest.entries",
        Ctx::ManifestIssues => "view.manifest.issues",
        Ctx::ManifestDiff => "view.manifest.diff",
        Ctx::ManifestPrompt => "view.manifest.prompt",
    }
}

fn when_name(w: When) -> &'static str {
    match w {
        When::Always => "",
        When::HasRow => "has_row",
        When::HasDocument => "has_document",
        When::ScopeIsOrg => "scope_is_org",
        When::ScopeIsGroup => "scope_is_group",
        When::UsesAsOf => "uses_as_of",
        When::LinksSection => "links_section",
        When::SettingsSection => "settings_section",
        When::NotOffline => "not_offline",
        When::ConfirmAsk => "confirm_ask",
        When::MultiSelect => "multi_select",
        When::HasRecord => "has_record",
        When::CanCompare => "can_compare",
        When::NoteMulti => "note_multi",
        When::Staged => "staged",
        When::AgentWrites => "agent_writes",
        When::RulesList => "rules_list",
        When::OnManifest => "on_manifest",
        When::OnWarning => "on_warning",
        When::HasCommand => "has_command",
    }
}

fn shown_keys(b: &Bound) -> String {
    b.keys
        .iter()
        .map(|k| format!("`{}`", k.key.display()))
        .collect::<Vec<_>>()
        .join(" ")
}

/// `keys --markdown`: the guide's keys page, generated from the table.
pub fn to_markdown() -> String {
    to_markdown_in(default_map())
}

/// `keys --markdown` for an effective keymap: with keys.toml overrides, a `From` column names their line.
pub fn to_markdown_in(km: &Keymap) -> String {
    let marked = km.overrides > 0;
    let mut out = String::from(
        "# ownpurse keys\n\nGenerated from the keymap table (the only source for the footer, help and palette).\n",
    );
    if marked {
        out.push_str(&format!(
            "\nEffective keymap: the built-in table with {} override{} from keys.toml (revision {}).\n",
            km.overrides,
            if km.overrides == 1 { "" } else { "s" },
            km.revision.as_deref().unwrap_or("")
        ));
    }
    let mut ctxs: Vec<Ctx> = Vec::new();
    for b in &km.bindings {
        if !ctxs.contains(&b.ctx) {
            ctxs.push(b.ctx);
        }
    }
    for c in ctxs {
        if marked {
            out.push_str(&format!(
                "\n## {}{}\n\n| Keys | Action | Label | When | Footer | From |\n|---|---|---|---|---|---|\n",
                ctx_name(c),
                if sealed(c) { " (sealed)" } else { "" }
            ));
        } else {
            out.push_str(&format!(
                "\n## {}{}\n\n| Keys | Action | Label | When | Footer |\n|---|---|---|---|---|\n",
                ctx_name(c),
                if sealed(c) { " (sealed)" } else { "" }
            ));
        }
        for b in km.bindings.iter().filter(|b| b.ctx == c) {
            let from = b
                .keys
                .iter()
                .filter_map(|k| {
                    k.from
                        .map(|l| format!("`{}` keys.toml:{l}", k.key.display()))
                })
                .collect::<Vec<_>>()
                .join(", ");
            out.push_str(&format!(
                "| {} | `{}` | {} | {} | {} |{}\n",
                shown_keys(b).replace('|', "\\|"),
                b.action,
                b.label,
                when_name(b.when),
                b.footer.map(|r| r.to_string()).unwrap_or_default(),
                if marked {
                    format!(" {from} |")
                } else {
                    String::new()
                }
            ));
        }
    }
    out.push_str("\n## Palette only (`⌃P`)\n\nNetwork actions have no keys (V3): they confirm first, with focus on Cancel.\n\n| Action | Label | Network | CLI |\n|---|---|---|---|\n");
    for a in ACTIONS {
        out.push_str(&format!(
            "| `{}` | {} | {} | {} |\n",
            a.id,
            a.label,
            if a.network { "yes" } else { "" },
            a.cli.map(|c| format!("`{c}`")).unwrap_or_default()
        ));
    }
    out
}

fn json_str(s: &str) -> String {
    let mut out = String::from("\"");
    for ch in s.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// `keys --json`.
pub fn to_json() -> String {
    to_json_in(default_map())
}

/// `keys --json` for an effective keymap (a keys.toml key also lists its line under `from`).
pub fn to_json_in(km: &Keymap) -> String {
    let bindings: Vec<String> = km
        .bindings
        .iter()
        .map(|b| {
            let from: Vec<String> = b
                .keys
                .iter()
                .filter_map(|k| k.from.map(|l| format!("{{\"key\":{},\"line\":{l}}}", json_str(&k.spec))))
                .collect();
            format!(
                "{{\"context\":{},\"keys\":[{}],\"action\":{},\"label\":{},\"group\":{},\"when\":{},\"footer\":{}{}}}",
                json_str(ctx_name(b.ctx)),
                b.keys.iter().map(|k| json_str(&k.spec)).collect::<Vec<_>>().join(","),
                json_str(b.action),
                json_str(b.label),
                json_str(b.group),
                json_str(when_name(b.when)),
                b.footer.map_or("null".into(), |r| r.to_string()),
                if from.is_empty() { String::new() } else { format!(",\"from\":[{}]", from.join(",")) }
            )
        })
        .collect();
    let actions: Vec<String> = ACTIONS
        .iter()
        .map(|a| {
            format!(
                "{{\"id\":{},\"label\":{},\"palette\":{},\"network\":{},\"cli\":{}}}",
                json_str(a.id),
                json_str(a.label),
                a.palette,
                a.network,
                a.cli.map_or("null".into(), json_str)
            )
        })
        .collect();
    format!(
        "{{\"bindings\":[{}],\"actions\":[{}]}}\n",
        bindings.join(","),
        actions.join(",")
    )
}

/// `keys --conflicts`: a key bound twice in one context, a view key that means something else
/// globally, or a network action with a key. Empty = clean.
pub fn conflicts() -> Vec<String> {
    conflicts_in(default_map())
        .into_iter()
        .map(|(_, m)| m)
        .collect()
}

/// The conflicts of an effective keymap, each with the keys.toml line involved (None: the built-in table).
pub fn conflicts_in(km: &Keymap) -> Vec<(Option<usize>, String)> {
    let mut out = Vec::new();
    for b in km.bindings.iter().filter(|b| is_network(b.action)) {
        out.push((
            b.keys.iter().find_map(|k| k.from),
            format!("network action {} has a key", b.action),
        ));
    }
    let line = |a: &BoundKey, b: &BoundKey| a.from.or(b.from);
    for b in km
        .bindings
        .iter()
        .filter(|b| b.ctx != Ctx::Global && !sealed(b.ctx))
    {
        for k in &b.keys {
            for g in km
                .bindings
                .iter()
                .filter(|g| g.ctx == Ctx::Global && g.action != b.action)
            {
                if let Some(gk) = g.keys.iter().find(|gk| gk.key == k.key) {
                    out.push((
                        line(k, gk),
                        format!(
                            "{} means {} globally and {} in {}",
                            k.spec,
                            g.action,
                            b.action,
                            ctx_name(b.ctx)
                        ),
                    ));
                }
            }
        }
    }
    for (i, b) in km.bindings.iter().enumerate() {
        for k in &b.keys {
            if let Some(pk) = km.bindings[..i]
                .iter()
                .filter(|p| p.ctx == b.ctx && p.action != b.action)
                .find_map(|p| p.keys.iter().find(|pk| pk.key == k.key))
            {
                out.push((
                    line(k, pk),
                    format!("{} bound twice in {}", k.spec, ctx_name(b.ctx)),
                ));
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_key_in_the_table_parses() {
        for b in BINDINGS {
            for k in b.keys {
                assert!(Key::parse(k).is_some(), "{k}");
            }
        }
    }

    #[test]
    fn exact_matching_and_display() {
        let g = Key::of(&KeyEvent::new(KeyCode::Char('g'), KeyModifiers::SHIFT));
        assert_eq!(Some(g), Key::parse("G"));
        let bang = Key::of(&KeyEvent::new(KeyCode::Char('!'), KeyModifiers::SHIFT));
        assert_eq!(Some(bang), Key::parse("!"));
        let back = Key::of(&KeyEvent::new(KeyCode::BackTab, KeyModifiers::SHIFT));
        assert_eq!(Some(back), Key::parse("shift-tab"));
        let cp = Key::of(&KeyEvent::new(KeyCode::Char('p'), KeyModifiers::CONTROL));
        assert_eq!(Some(cp), Key::parse("ctrl-p"));
        assert_eq!(cp.display(), "⌃P");
        assert_ne!(Key::parse("d"), Key::parse("ctrl-d"));
        assert_eq!(Key::parse("shift-tab").unwrap().display(), "⇧Tab");
    }

    #[test]
    fn table_has_no_conflicts() {
        assert_eq!(conflicts(), Vec::<String>::new());
    }

    #[test]
    fn exports_cover_every_binding_and_action() {
        let md = to_markdown();
        let json = to_json();
        for b in BINDINGS {
            assert!(md.contains(&format!("`{}`", b.action)), "{}", b.action);
            assert!(json.contains(&format!("\"{}\"", b.action)), "{}", b.action);
        }
        for a in ACTIONS {
            assert!(md.contains(a.id) && json.contains(a.id), "{}", a.id);
        }
    }
}
