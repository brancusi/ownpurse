//! Classify › Catalog (spec classify §3.D, §6): the Projects and Rules lists with their Used counts, the
//! project and rule editors (the rule editor with its live "would match N lines" preview), and the staged
//! changes: nothing touches `classify.toml` until `w` shows the unified diff and its confirm (Cancel focused)
//! writes it through the core (`stage_config` → `write_config`, atomic replace + a `config_written` event).
//! A delete of something in use becomes an archive (V20), said in a toast before anything is written.

use super::{ClassifyUi, Mode, Op};
use crate::app::{App, Effect};
use ownpurse_core::classify::config::{Choice, Cond, ConfigEdit, ItemRef, Rule, SetValue};
use rust_decimal::Decimal;
use std::str::FromStr;

/// Which list (`←` / `→`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum CatList {
    #[default]
    Projects,
    Rules,
}

impl CatList {
    pub fn label(self) -> &'static str {
        match self {
            CatList::Projects => "Projects",
            CatList::Rules => "Rules",
        }
    }
}

/// A rule as its editor shows it: one text field per condition kind it edits, comma lists where a
/// condition takes several values. Conditions the form doesn't edit (amount, date, sign) are kept as they
/// were in `keep` (rendered read-only).
#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct RuleForm {
    pub id: String,
    pub priority: String,
    pub alias_of: String,
    pub payee: String,
    pub payee_has: String,
    pub description_has: String,
    pub account_in: String,
    pub org_in: String,
    pub project: String,
    pub bearer: String,
    pub treatment: String,
    pub confidence: String,
    /// The original rule's id when editing (None: a new rule).
    pub editing: Option<String>,
}

/// One staged edit (serialisable: it is part of the view state, V21).
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum CatEdit {
    AddProject {
        id: String,
        name: String,
        aliases: Vec<String>,
    },
    EditProject {
        id: String,
        name: String,
        aliases: Vec<String>,
    },
    RemoveProject {
        id: String,
    },
    AddRule(RuleForm),
    EditRule(RuleForm),
    RemoveRule {
        id: String,
    },
    SetPriority {
        id: String,
        priority: i64,
    },
}

/// The open editor: labelled text fields and the focused one.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct CatEditor {
    /// `project` or `rule`.
    pub kind: String,
    /// The project id / rule id being edited (None: new).
    pub editing: Option<String>,
    pub fields: Vec<(String, String)>,
    pub focus: usize,
    /// Why the last save was refused (shown under the fields).
    pub error: Option<String>,
}

/// The `w` diff preview and its confirm (focus 0 = Cancel).
#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DiffUi {
    pub focus: usize,
    pub top: usize,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct CatalogUi {
    pub list: CatList,
    /// The cursor per list (ids; navigation rule 4).
    pub sel_project: Option<String>,
    pub sel_rule: Option<String>,
    pub staged: Vec<CatEdit>,
    /// The config hash the staged edits were made on: a hand edit since then refuses the write once.
    pub base_hash: Option<String>,
    pub editor: Option<CatEditor>,
    pub diff: Option<DiffUi>,
    /// A one-line notice above the diff band (`used by 37 lines: archived instead`), until the next key.
    pub toast: Option<String>,
}

pub const PROJECT_FIELDS: [&str; 3] = ["id", "name", "aliases"];
pub const RULE_FIELDS: [&str; 12] = [
    "id",
    "priority",
    "alias of",
    "payee",
    "payee has",
    "description has",
    "account in",
    "org in",
    "set project",
    "set bearer",
    "set treatment",
    "confidence",
];

fn list(s: &str) -> Vec<String> {
    s.split(',')
        .map(str::trim)
        .filter(|x| !x.is_empty())
        .map(str::to_string)
        .collect()
}

impl RuleForm {
    pub fn of(r: &Rule) -> RuleForm {
        let mut f = RuleForm {
            id: r.id.clone(),
            priority: r.priority.to_string(),
            confidence: r.confidence.normalize().to_string(),
            editing: Some(r.id.clone()),
            ..RuleForm::default()
        };
        for c in &r.when {
            match c {
                Cond::DescriptionHasAlias(t) => f.alias_of = t.clone(),
                Cond::Payee(p) => f.payee = p.clone(),
                Cond::PayeeHas(v) => f.payee_has = v.join(", "),
                Cond::DescriptionHas(v) => f.description_has = v.join(", "),
                Cond::AccountIn(v) => f.account_in = v.join(", "),
                Cond::OrgIn(v) => f.org_in = v.join(", "),
                _ => {}
            }
        }
        for (k, v) in &r.set {
            let v = match v {
                SetValue::Choice(c) => c.clone(),
                SetValue::Matched => "$matched".into(),
            };
            match k.as_str() {
                "project" => f.project = v,
                "bearer" => f.bearer = v,
                "treatment" => f.treatment = v,
                _ => {}
            }
        }
        f
    }

    pub fn fields(&self) -> Vec<(String, String)> {
        let v = [
            &self.id,
            &self.priority,
            &self.alias_of,
            &self.payee,
            &self.payee_has,
            &self.description_has,
            &self.account_in,
            &self.org_in,
            &self.project,
            &self.bearer,
            &self.treatment,
            &self.confidence,
        ];
        RULE_FIELDS
            .iter()
            .zip(v)
            .map(|(k, v)| (k.to_string(), v.clone()))
            .collect()
    }

    pub fn from_fields(fields: &[(String, String)], editing: Option<String>) -> RuleForm {
        let get = |k: &str| {
            fields
                .iter()
                .find(|(n, _)| n == k)
                .map(|(_, v)| v.trim().to_string())
                .unwrap_or_default()
        };
        RuleForm {
            id: get("id"),
            priority: get("priority"),
            alias_of: get("alias of"),
            payee: get("payee"),
            payee_has: get("payee has"),
            description_has: get("description has"),
            account_in: get("account in"),
            org_in: get("org in"),
            project: get("set project"),
            bearer: get("set bearer"),
            treatment: get("set treatment"),
            confidence: get("confidence"),
            editing,
        }
    }

    /// The rule, with the original's conditions the form doesn't edit kept (`base`), or why it isn't one.
    pub fn rule(&self, base: Option<&Rule>) -> Result<Rule, String> {
        if self.id.is_empty()
            || !self
                .id
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
        {
            return Err("id: letters, digits, - and _ only".into());
        }
        let priority = if self.priority.is_empty() {
            50
        } else {
            self.priority
                .parse::<i64>()
                .map_err(|_| format!("priority: a whole number, got `{}`", self.priority))?
        };
        let confidence = if self.confidence.is_empty() {
            Decimal::ONE
        } else {
            Decimal::from_str(&self.confidence)
                .ok()
                .filter(|c| *c >= Decimal::ZERO && *c <= Decimal::ONE)
                .ok_or_else(|| format!("confidence: 0 to 1, got `{}`", self.confidence))?
        };
        let mut when = Vec::new();
        if !self.alias_of.is_empty() {
            when.push(Cond::DescriptionHasAlias(self.alias_of.clone()));
        }
        if !self.payee.is_empty() {
            when.push(Cond::Payee(self.payee.clone()));
        }
        for (v, mk) in [
            (&self.payee_has, Cond::PayeeHas as fn(Vec<String>) -> Cond),
            (&self.description_has, Cond::DescriptionHas),
            (&self.account_in, Cond::AccountIn),
            (&self.org_in, Cond::OrgIn),
        ] {
            let l = list(v);
            if !l.is_empty() {
                when.push(mk(l));
            }
        }
        if let Some(b) = base {
            when.extend(
                b.when
                    .iter()
                    .filter(|c| {
                        matches!(
                            c,
                            Cond::AmountBetween(..)
                                | Cond::DateBetween(..)
                                | Cond::Sign(_)
                                | Cond::PayeeIsOrg(_)
                                | Cond::BankTransfer(_)
                        )
                    })
                    .cloned(),
            );
        }
        if when.is_empty() {
            return Err("a rule needs at least one condition".into());
        }
        let mut set = Vec::new();
        for (k, v) in [
            ("project", &self.project),
            ("bearer", &self.bearer),
            ("treatment", &self.treatment),
        ] {
            if v.is_empty() {
                continue;
            }
            set.push((
                k.to_string(),
                if v == "$matched" {
                    SetValue::Matched
                } else {
                    SetValue::Choice(v.clone())
                },
            ));
        }
        if set.is_empty() && base.and_then(|b| b.group_kind.clone()).is_none() {
            return Err("a rule sets at least one label".into());
        }
        Ok(Rule {
            id: self.id.clone(),
            priority,
            when,
            set,
            group_kind: base.and_then(|b| b.group_kind.clone()),
            confidence,
            archived: false,
        })
    }
}

/// The staged edits as the core's edits (rules completed from the live config).
pub fn config_edits(
    staged: &[CatEdit],
    cfg: &ownpurse_core::classify::config::ClassifyConfig,
) -> Result<Vec<ConfigEdit>, String> {
    staged
        .iter()
        .map(|e| {
            Ok(match e {
                CatEdit::AddProject { id, name, aliases } => ConfigEdit::AddChoice {
                    label_type: "project".into(),
                    choice: Choice {
                        id: id.clone(),
                        name: name.clone(),
                        aliases: aliases.clone(),
                        archived: false,
                    },
                },
                CatEdit::EditProject { id, name, aliases } => ConfigEdit::EditChoice {
                    label_type: "project".into(),
                    id: id.clone(),
                    name: Some(name.clone()),
                    aliases: Some(aliases.clone()),
                },
                CatEdit::RemoveProject { id } => ConfigEdit::RemoveChoice {
                    label_type: "project".into(),
                    id: id.clone(),
                },
                CatEdit::AddRule(f) => ConfigEdit::AddRule(f.rule(None)?),
                CatEdit::EditRule(f) => {
                    let base = f.editing.as_deref().and_then(|id| cfg.rule(id));
                    ConfigEdit::EditRule(f.rule(base)?)
                }
                CatEdit::RemoveRule { id } => ConfigEdit::RemoveRule { id: id.clone() },
                CatEdit::SetPriority { id, priority } => ConfigEdit::SetPriority {
                    id: id.clone(),
                    priority: *priority,
                },
            })
        })
        .collect()
}

impl ClassifyUi {
    pub fn catalog_busy(&self) -> Option<&'static str> {
        if self.catalog.editor.is_some() {
            Some("editing the catalog")
        } else if self.catalog.diff.is_some() {
            Some("reviewing a classify.toml diff")
        } else {
            None
        }
    }
}

/// The ASCII spelling of an accented Latin letter (lowercase).
fn translit(c: char) -> Option<&'static str> {
    let c = c.to_lowercase().next().unwrap_or(c);
    Some(match c {
        'à' | 'á' | 'â' | 'ã' | 'ä' | 'å' | 'ā' => "a",
        'ç' | 'ć' | 'č' => "c",
        'è' | 'é' | 'ê' | 'ë' | 'ē' => "e",
        'ì' | 'í' | 'î' | 'ï' | 'ī' => "i",
        'ñ' | 'ń' => "n",
        'ò' | 'ó' | 'ô' | 'õ' | 'ö' | 'ø' | 'ō' => "o",
        'ù' | 'ú' | 'û' | 'ü' | 'ū' => "u",
        'ý' | 'ÿ' => "y",
        'š' | 'ś' => "s",
        'ž' | 'ź' | 'ż' => "z",
        'ß' => "ss",
        'æ' => "ae",
        'œ' => "oe",
        _ => return None,
    })
}

/// A slug for ids: `Harvest Blend launch` → `harvest-blend-launch`.
pub fn slug(s: &str) -> String {
    let mut out = String::new();
    for c in s.chars() {
        if c.is_ascii_alphanumeric() {
            out.push(c.to_ascii_lowercase());
        } else if let Some(t) = translit(c) {
            // accented Latin letters transliterate (`Cafés` → `cafes`)
            out.push_str(t);
        } else if c.is_alphanumeric() {
            // other letters outside ASCII: dropped, never a word break
        } else if !out.ends_with('-') && !out.is_empty() {
            out.push('-');
        }
    }
    out.trim_end_matches('-').chars().take(24).collect()
}

impl App {
    /// Project rows: (id, name, aliases, used, archived), in file order.
    pub fn cat_projects(&self) -> Vec<(String, String, String, usize, bool)> {
        let Some(d) = self.classify.as_ref() else {
            return vec![];
        };
        let Some(t) = d.provider.config().label_type("project") else {
            return vec![];
        };
        t.choices
            .iter()
            .map(|c| {
                (
                    c.id.clone(),
                    c.name.clone(),
                    c.aliases.join(", "),
                    d.provider.used(&ItemRef::Choice {
                        label_type: "project".into(),
                        id: c.id.clone(),
                    }),
                    c.archived,
                )
            })
            .collect()
    }

    /// Rule rows by priority: (rule, would-match count, lines whose current label came from it).
    pub fn cat_rules(&self) -> Vec<(Rule, usize, usize)> {
        let Some(d) = self.classify.as_ref() else {
            return vec![];
        };
        let mut rules: Vec<&Rule> = d.provider.config().rules.iter().collect();
        rules.sort_by_key(|r| std::cmp::Reverse(r.priority));
        rules
            .into_iter()
            .map(|r| {
                (
                    r.clone(),
                    d.cached_would_match(r).count,
                    d.provider.labelled(&r.id),
                )
            })
            .collect()
    }

    fn cat_len(&self) -> usize {
        match self.ui.classify.catalog.list {
            CatList::Projects => self.cat_projects().len(),
            CatList::Rules => self.cat_rules().len(),
        }
    }

    pub fn cat_cursor(&self) -> Option<usize> {
        let c = &self.ui.classify.catalog;
        match c.list {
            CatList::Projects => {
                let rows = self.cat_projects();
                (!rows.is_empty()).then(|| {
                    c.sel_project
                        .as_ref()
                        .and_then(|s| rows.iter().position(|r| &r.0 == s))
                        .unwrap_or(0)
                })
            }
            CatList::Rules => {
                let rows = self.cat_rules();
                (!rows.is_empty()).then(|| {
                    c.sel_rule
                        .as_ref()
                        .and_then(|s| rows.iter().position(|r| &r.0.id == s))
                        .unwrap_or(0)
                })
            }
        }
    }

    fn cat_select(&mut self, pos: usize) {
        match self.ui.classify.catalog.list {
            CatList::Projects => {
                let rows = self.cat_projects();
                if let Some(r) = rows.get(pos.min(rows.len().saturating_sub(1))) {
                    self.ui.classify.catalog.sel_project = Some(r.0.clone());
                }
            }
            CatList::Rules => {
                let rows = self.cat_rules();
                if let Some(r) = rows.get(pos.min(rows.len().saturating_sub(1))) {
                    self.ui.classify.catalog.sel_rule = Some(r.0.id.clone());
                }
            }
        }
    }

    /// A click on drawn row `i` (the list scrolls to keep the cursor in its last line).
    pub(crate) fn cat_click_row(&mut self, i: usize) {
        let h = self
            .hits
            .content
            .map_or(10, |r| (r.height as usize).saturating_sub(1).max(1));
        let top = self
            .cat_cursor()
            .map_or(0, |c| c.saturating_sub(h.saturating_sub(1)));
        self.cat_select(top + i);
    }

    fn cat_stage(&mut self, e: CatEdit) {
        let hash = self.classify.as_ref().map(|d| d.provider.config_hash());
        let c = &mut self.ui.classify.catalog;
        if c.staged.is_empty() {
            c.base_hash = hash;
        }
        c.staged.push(e);
    }

    /// The staged change as the core would write it (its diff, summary and archive-instead notes).
    pub fn cat_staged(&self) -> Option<Result<ownpurse_core::classify::config::Staged, String>> {
        let d = self.classify.as_ref()?;
        let staged = &self.ui.classify.catalog.staged;
        if staged.is_empty() {
            return None;
        }
        Some(
            config_edits(staged, d.provider.config())
                .and_then(|e| d.provider.stage(&e).map_err(|e| e.message)),
        )
    }

    /// The rule editor's live preview (what it would match), or why the rule isn't valid yet.
    pub fn cat_preview(&self) -> Option<Result<super::Matched, String>> {
        let ed = self
            .ui
            .classify
            .catalog
            .editor
            .as_ref()
            .filter(|e| e.kind == "rule")?;
        let d = self.classify.as_ref()?;
        let f = RuleForm::from_fields(&ed.fields, ed.editing.clone());
        let base = ed
            .editing
            .as_deref()
            .and_then(|id| d.provider.config().rule(id));
        Some(f.rule(base).map(|r| d.cached_would_match(&r)))
    }

    /// Open the rule editor on a form (`N`, `Enter`, the Queue's `R`).
    pub fn cat_open_rule(&mut self, f: RuleForm) {
        let editing = f.editing.clone();
        self.ui.classify.mode = Mode::Catalog;
        self.ui.classify.catalog.list = CatList::Rules;
        self.ui.classify.catalog.editor = Some(CatEditor {
            kind: "rule".into(),
            editing,
            fields: f.fields(),
            focus: 0,
            error: None,
        });
    }

    /// `R` in the Queue: a rule prefilled from this line (its payee, an alias its description has, and its
    /// labels or suggestions), opened in the Catalog's rule editor with the live preview.
    pub fn cat_make_rule(&mut self) {
        let (Some(r), Some(d)) = (self.cl_selected(), self.classify.as_ref()) else {
            return;
        };
        let cfg = d.provider.config();
        let desc = r.description.to_lowercase();
        let has_alias = cfg.label_type("project").is_some_and(|t| {
            t.active().any(|c| {
                std::iter::once(&c.name)
                    .chain(c.aliases.iter())
                    .any(|a| ownpurse_core::classify::rules::has_word(&desc, &a.to_lowercase()))
            })
        });
        let pick = |c: &Option<ownpurse_core::classify::Cell>| {
            c.as_ref().map(|c| c.value.clone()).unwrap_or_default()
        };
        let mut id = format!("r-{}", slug(&r.payee));
        let taken = |id: &str| cfg.rule(id).is_some();
        if taken(&id) {
            let mut n = 2;
            while taken(&format!("{id}-{n}")) {
                n += 1;
            }
            id = format!("{id}-{n}");
        }
        let f = RuleForm {
            id,
            priority: "60".into(),
            alias_of: if has_alias {
                "project".into()
            } else {
                String::new()
            },
            payee: r.payee.clone(),
            project: if has_alias {
                "$matched".into()
            } else {
                pick(&r.suggestion.project)
            },
            bearer: pick(&r.suggestion.bearer),
            treatment: pick(&r.suggestion.treatment),
            confidence: "0.9".into(),
            ..RuleForm::default()
        };
        self.cat_open_rule(f);
    }

    /// Catalog keys. Returns false for anything the shell handles.
    pub fn cat_action(&mut self, action: &str, fx: &mut Vec<Effect>) -> bool {
        if self.ui.classify.mode != Mode::Catalog || self.classify.is_none() {
            return false;
        }
        self.ui.classify.catalog.toast = None;
        if self.ui.classify.catalog.diff.is_some() {
            return self.cat_diff_action(action, fx);
        }
        if self.ui.classify.catalog.editor.is_some() {
            return self.cat_editor_action(action);
        }
        let c = &mut self.ui.classify.catalog;
        match action {
            "catalog.list_next" | "catalog.list_prev" => {
                c.list = match c.list {
                    CatList::Projects => CatList::Rules,
                    CatList::Rules => CatList::Projects,
                };
            }
            "catalog.new" => match c.list {
                CatList::Projects => {
                    c.editor = Some(CatEditor {
                        kind: "project".into(),
                        editing: None,
                        fields: PROJECT_FIELDS
                            .iter()
                            .map(|k| (k.to_string(), String::new()))
                            .collect(),
                        focus: 1,
                        error: None,
                    })
                }
                CatList::Rules => self.cat_open_rule(RuleForm {
                    id: "r-new".into(),
                    priority: "50".into(),
                    confidence: "0.9".into(),
                    ..RuleForm::default()
                }),
            },
            "drill.in" | "catalog.edit" => match c.list {
                CatList::Projects => {
                    let Some(i) = self.cat_cursor() else {
                        return true;
                    };
                    let (id, name, aliases, _, _) = self.cat_projects()[i].clone();
                    self.ui.classify.catalog.editor = Some(CatEditor {
                        kind: "project".into(),
                        editing: Some(id.clone()),
                        fields: vec![
                            ("id".into(), id),
                            ("name".into(), name),
                            ("aliases".into(), aliases),
                        ],
                        focus: 1,
                        error: None,
                    });
                }
                CatList::Rules => {
                    let Some(i) = self.cat_cursor() else {
                        return true;
                    };
                    let f = RuleForm::of(&self.cat_rules()[i].0);
                    self.cat_open_rule(f);
                }
            },
            "catalog.archive" => {
                let Some(i) = self.cat_cursor() else {
                    return true;
                };
                let (e, used, name) = match self.ui.classify.catalog.list {
                    CatList::Projects => {
                        let p = self.cat_projects()[i].clone();
                        (CatEdit::RemoveProject { id: p.0.clone() }, p.3, p.1)
                    }
                    CatList::Rules => {
                        let r = self.cat_rules()[i].clone();
                        (CatEdit::RemoveRule { id: r.0.id.clone() }, r.2, r.0.id)
                    }
                };
                self.cat_stage(e);
                // V20: in use → archived instead of deleted; said now, before anything is written
                let _ = name;
                // design copy: whole at 100 columns
                self.ui.classify.catalog.toast = Some(if used > 0 {
                    format!(
                        "Used by {used} line{}, so it's archived instead: hidden from pickers, kept in history.",
                        if used == 1 { "" } else { "s" }
                    )
                } else {
                    "Unused, so it will be deleted when you write.".to_string()
                });
            }
            "catalog.priority_up" | "catalog.priority_down" if c.list == CatList::Rules => {
                let Some(i) = self.cat_cursor() else {
                    return true;
                };
                let rules = self.cat_rules();
                let up = action == "catalog.priority_up";
                let j = if up { i.checked_sub(1) } else { Some(i + 1) };
                if let Some(other) = j.and_then(|j| rules.get(j)) {
                    let me = &rules[i].0;
                    let priority = if up {
                        other.0.priority + 1
                    } else {
                        other.0.priority - 1
                    };
                    self.cat_stage(CatEdit::SetPriority {
                        id: me.id.clone(),
                        priority,
                    });
                }
            }
            "catalog.apply" if !c.staged.is_empty() => c.diff = Some(DiffUi::default()),
            "catalog.apply" => self.ui.note = Some("Nothing staged to write.".into()),
            "catalog.discard" if !c.staged.is_empty() => {
                let n = c.staged.len();
                c.staged.clear();
                c.base_hash = None;
                self.ui.note = Some(format!(
                    "discarded {n} staged change{}",
                    if n == 1 { "" } else { "s" }
                ));
            }
            "catalog.discard" => self.ui.note = Some("Nothing staged.".into()),
            a => {
                let n = self.cat_len();
                match crate::views::activity::moved(a, self.cat_cursor().unwrap_or(0), n, 10) {
                    Some(pos) if n > 0 => self.cat_select(pos),
                    Some(_) => {}
                    None => return false,
                }
            }
        }
        true
    }

    fn cat_editor_action(&mut self, action: &str) -> bool {
        let Some(ed) = self.ui.classify.catalog.editor.as_mut() else {
            return false;
        };
        let n = ed.fields.len();
        match action {
            "catalog.field_next" => ed.focus = (ed.focus + 1) % n,
            "catalog.field_prev" => ed.focus = (ed.focus + n - 1) % n,
            "catalog.cancel" => self.ui.classify.catalog.editor = None,
            "catalog.save" => {
                let ed = ed.clone();
                let staged = match ed.kind.as_str() {
                    "project" => {
                        let get = |k: &str| {
                            ed.fields
                                .iter()
                                .find(|(n, _)| n == k)
                                .map(|(_, v)| v.trim().to_string())
                                .unwrap_or_default()
                        };
                        let name = get("name");
                        let id = match get("id") {
                            i if i.is_empty() => slug(&name),
                            i => i,
                        };
                        let aliases = list(&get("aliases"));
                        if name.is_empty() {
                            Err("name: a project needs a name".to_string())
                        } else if ed.editing.is_none()
                            && self.cat_projects().iter().any(|p| p.0 == id)
                        {
                            Err(format!("id: `{id}` is taken"))
                        } else {
                            Ok(match &ed.editing {
                                Some(_) => CatEdit::EditProject { id, name, aliases },
                                None => CatEdit::AddProject { id, name, aliases },
                            })
                        }
                    }
                    _ => {
                        let f = RuleForm::from_fields(&ed.fields, ed.editing.clone());
                        let base = ed.editing.as_deref().and_then(|id| {
                            self.classify
                                .as_ref()
                                .and_then(|d| d.provider.config().rule(id))
                                .cloned()
                        });
                        f.rule(base.as_ref()).map(|_| match ed.editing {
                            Some(_) => CatEdit::EditRule(f),
                            None => CatEdit::AddRule(f),
                        })
                    }
                };
                match staged {
                    Ok(e) => {
                        let what = match &e {
                            CatEdit::AddProject { name, .. } => format!("project \"{name}\""),
                            CatEdit::EditProject { name, .. } => format!("project \"{name}\""),
                            CatEdit::AddRule(f) | CatEdit::EditRule(f) => format!("rule {}", f.id),
                            _ => String::new(),
                        };
                        self.ui.classify.catalog.editor = None;
                        self.cat_stage(e);
                        self.ui.note = Some(format!("staged {what} · w write · u discard"));
                    }
                    Err(why) => {
                        if let Some(ed) = self.ui.classify.catalog.editor.as_mut() {
                            ed.error = Some(why);
                        }
                    }
                }
            }
            _ => return false,
        }
        true
    }

    fn cat_diff_action(&mut self, action: &str, fx: &mut Vec<Effect>) -> bool {
        let Some(d) = self.ui.classify.catalog.diff.as_mut() else {
            return false;
        };
        match action {
            "catalog.diff_focus" => d.focus = 1 - d.focus.min(1),
            "catalog.cancel" => self.ui.classify.catalog.diff = None,
            "row.next" | "page.next" => d.top += if action == "row.next" { 1 } else { 10 },
            "row.prev" | "page.prev" => {
                d.top = d
                    .top
                    .saturating_sub(if action == "row.prev" { 1 } else { 10 })
            }
            "catalog.diff_go" => {
                let go = d.focus == 1;
                self.ui.classify.catalog.diff = None;
                if !go {
                    return true;
                }
                // a hand edit since the edits were staged: refuse once, then a second w re-applies them
                let now = self.classify.as_ref().map(|d| d.provider.config_hash());
                let c = &mut self.ui.classify.catalog;
                if c.base_hash.is_some() && c.base_hash != now {
                    c.base_hash = now;
                    let n = c.staged.len();
                    self.refuse(format!(
                        "classify.toml changed on disk since you staged {n} change{}; it was reloaded. They stay staged: w applies them to the new file, u discards them.",
                        if n == 1 { "" } else { "s" }
                    ));
                    return true;
                }
                let staged = self.ui.classify.catalog.staged.clone();
                self.cl_write_pub(Op::Config(staged), fx);
            }
            _ => return false,
        }
        true
    }

    pub fn cat_text_mut(&mut self) -> Option<&mut String> {
        let ed = self.ui.classify.catalog.editor.as_mut()?;
        let f = ed.focus;
        ed.error = None;
        ed.fields.get_mut(f).map(|(_, v)| v)
    }
}
