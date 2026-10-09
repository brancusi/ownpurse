//! Agent writes to the classification layer (spec §5a): the bridge request `kind: "classify"` (schema `cv` 1)
//! and the single write path both the CLI and the TUI use, [`Workspace::apply`].
//!
//! Request: `{"v":1, "kind":"classify", "cv":1, "op":…, "client":"claude", "expect_store_rev":82,
//! "caption":"…"?, "allow_bulk":false?, "overwrite":false?, …op fields}`. Targets are `"anchors":[…]` or
//! `"filter":"…"` (resolved inside the config scope; `status:open` unless the filter says otherwise).
//!
//! Rules: the store revision must match (`view_changed`, exit 4, naming the store rev); a caption follows the
//! bridge caption rules (`bad_caption`, exit 2); more than 50 lines need `allow_bulk` (`too_many`, exit 2);
//! a type a human confirmed is skipped (listed in `skipped`, reason `human_label`) unless `overwrite`;
//! an agent undoes only agent events; a non-dry config write needs `"confirm": true`. Every write is one
//! batch, `actor = agent:<client>`. Xero is never written (V18).

use super::anchor::Anchor;
use super::config::{Choice, Cond, ConfigEdit, Rule, SetValue, Sign};
use super::store::{EventKind, NoteTarget};
use super::view::{Filter, Status, Workspace, WriteCtx, label_json, with_caption};
use super::{Actor, BatchId, ClassifyError, Result, Source};
use rust_decimal::Decimal;
use serde_json::{Map, Value, json};

/// The classify request schema version.
pub const CV: i64 = 1;

/// The ops, in documentation order.
pub const OPS: [&str; 10] = [
    "set",
    "clear",
    "accept",
    "note",
    "group_create",
    "group_add",
    "group_remove",
    "group_status",
    "undo",
    "config",
];

/// Which lines a request touches.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Targets {
    Anchors(Vec<Anchor>),
    /// Filter text, resolved inside the scope; `status:open` unless it says otherwise.
    Filter(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UndoWhat {
    /// The latest agent batch not yet undone.
    Batch,
    /// These events (agent events only).
    Events(Vec<i64>),
    /// The last agent write on this line.
    Line(Anchor),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Op {
    Set {
        targets: Targets,
        /// `(label type, value)`, in request order.
        labels: Vec<(String, String)>,
    },
    Clear {
        targets: Targets,
        types: Vec<String>,
    },
    Accept {
        targets: Targets,
    },
    Note {
        target: NoteTarget,
        text: String,
    },
    GroupCreate {
        kind: String,
        title: String,
    },
    GroupAdd {
        group: String,
        anchors: Vec<Anchor>,
        role: Option<String>,
    },
    GroupRemove {
        group: String,
        anchors: Vec<Anchor>,
    },
    GroupStatus {
        group: String,
        status: String,
    },
    Undo(UndoWhat),
    Config {
        edits: Vec<ConfigEdit>,
        dry_run: bool,
        confirm: bool,
    },
}

impl Op {
    pub fn name(&self) -> &'static str {
        match self {
            Op::Set { .. } => "set",
            Op::Clear { .. } => "clear",
            Op::Accept { .. } => "accept",
            Op::Note { .. } => "note",
            Op::GroupCreate { .. } => "group_create",
            Op::GroupAdd { .. } => "group_add",
            Op::GroupRemove { .. } => "group_remove",
            Op::GroupStatus { .. } => "group_status",
            Op::Undo(_) => "undo",
            Op::Config { .. } => "config",
        }
    }
}

/// One classify request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClassifyRequest {
    pub op: Op,
    /// The agent's name: `actor = agent:<client>`.
    pub client: String,
    /// The store revision the agent last read (`store_rev`); required.
    pub expect_store_rev: i64,
    pub caption: Option<String>,
    pub allow_bulk: bool,
    /// Replace labels a human confirmed (otherwise they are skipped).
    pub overwrite: bool,
}

/// A line/type the request left alone.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Skipped {
    pub anchor: Anchor,
    pub label_type: String,
    /// `human_label`.
    pub reason: &'static str,
}

/// What a request did.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ApplyOutcome {
    pub op: String,
    pub batch: String,
    pub events: Vec<i64>,
    pub lines: usize,
    pub anchors: Vec<Anchor>,
    pub skipped: Vec<Skipped>,
    /// Past tense, for the notice: `set project Harvest Blend launch on 12 lines (2 skipped: confirmed by you)`.
    pub summary: String,
    pub store_rev: i64,
    /// `group_create`: the new group id.
    pub group: Option<String>,
    /// `config`: the unified diff (written, or only previewed when `dry_run`).
    pub diff: Option<String>,
    pub dry_run: bool,
}

impl ApplyOutcome {
    /// The bridge response: `{"v":1,"ok":true,"kind":"classify",…}`.
    pub fn to_json(&self) -> Value {
        let mut m = Map::new();
        m.insert("v".into(), json!(crate::bridge::PROTOCOL));
        m.insert("ok".into(), json!(true));
        m.insert("kind".into(), json!("classify"));
        m.insert("op".into(), json!(self.op));
        m.insert("batch".into(), json!(self.batch));
        m.insert("events".into(), json!(self.events));
        m.insert("lines".into(), json!(self.lines));
        m.insert(
            "anchors".into(),
            json!(
                self.anchors
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
            ),
        );
        m.insert(
            "skipped".into(),
            Value::Array(
                self.skipped
                    .iter()
                    .map(|s| json!({"anchor": s.anchor.to_string(), "type": s.label_type, "reason": s.reason}))
                    .collect(),
            ),
        );
        m.insert("summary".into(), json!(self.summary));
        m.insert("store_rev".into(), json!(self.store_rev));
        // the uniform per-verb effects list every agent reply carries (one entry: this op)
        let skipped: Vec<Value> = self
            .skipped
            .iter()
            .map(|s| json!({"anchor": s.anchor.to_string(), "type": s.label_type, "reason": s.reason}))
            .collect();
        let mut eff =
            json!({"verb": self.op, "changed": !self.events.is_empty(), "what": self.summary});
        if !skipped.is_empty() {
            eff["skipped"] = Value::Array(skipped);
        }
        m.insert("effects".into(), json!([eff]));
        if let Some(g) = &self.group {
            m.insert("group".into(), json!(g));
        }
        if let Some(d) = &self.diff {
            m.insert("diff".into(), json!(d));
            m.insert("dry_run".into(), json!(self.dry_run));
        }
        Value::Object(m)
    }
}

/// A refusal as the bridge's error document (`error`, `message`, `help`, `exit`).
pub fn error_response(e: &ClassifyError) -> Value {
    let help: Vec<&str> = e.hints.iter().map(String::as_str).collect();
    crate::bridge::error_doc(e.code, e.exit_code(), e.message.clone(), &help)
}

fn usage(m: impl Into<String>) -> ClassifyError {
    ClassifyError::new("usage", m)
}

fn s<'v>(v: &'v Value, k: &str) -> Option<&'v str> {
    v.get(k).and_then(Value::as_str)
}

fn need_str(v: &Value, k: &str, op: &str) -> Result<String> {
    s(v, k)
        .filter(|x| !x.is_empty())
        .map(str::to_string)
        .ok_or_else(|| usage(format!("op `{op}` needs \"{k}\"")))
}

fn strings(v: &Value, k: &str) -> Result<Vec<String>> {
    match v.get(k) {
        None | Some(Value::Null) => Ok(vec![]),
        Some(Value::String(x)) => Ok(vec![x.clone()]),
        Some(Value::Array(a)) => a
            .iter()
            .map(|x| {
                x.as_str()
                    .map(str::to_string)
                    .ok_or_else(|| usage(format!("\"{k}\" holds strings")))
            })
            .collect(),
        Some(_) => Err(usage(format!("\"{k}\" is a string or an array of strings"))),
    }
}

fn anchors(v: &Value, k: &str) -> Result<Vec<Anchor>> {
    strings(v, k)?.iter().map(|a| Anchor::parse(a)).collect()
}

fn targets(v: &Value, op: &str) -> Result<Targets> {
    match (v.get("anchors"), s(v, "filter")) {
        (Some(_), Some(_)) => Err(usage(format!(
            "op `{op}`: give \"anchors\" or \"filter\", not both"
        ))),
        (Some(_), None) => {
            let a = anchors(v, "anchors")?;
            if a.is_empty() {
                return Err(usage(format!("op `{op}`: \"anchors\" is empty")));
            }
            Ok(Targets::Anchors(a))
        }
        (None, Some(f)) => Ok(Targets::Filter(f.to_string())),
        (None, None) => Err(usage(format!("op `{op}` needs \"anchors\" or \"filter\""))),
    }
}

fn targets_json(t: &Targets, m: &mut Map<String, Value>) {
    match t {
        Targets::Anchors(a) => {
            m.insert(
                "anchors".into(),
                json!(a.iter().map(ToString::to_string).collect::<Vec<_>>()),
            );
        }
        Targets::Filter(f) => {
            m.insert("filter".into(), json!(f));
        }
    }
}

fn dec_text(v: &Value) -> Option<String> {
    match v {
        Value::String(x) => Some(x.clone()),
        Value::Number(n) => Some(n.to_string()),
        _ => None,
    }
}

fn pair_of(v: &Value) -> Option<(String, String)> {
    let a = v.as_array().filter(|a| a.len() == 2)?;
    Some((dec_text(&a[0])?, dec_text(&a[1])?))
}

/// A rule from JSON (the same keys as `[[rules]]` in classify.toml). Full validation happens when the edit
/// is staged (the new file must parse).
pub fn rule_from_json(v: &Value) -> Result<Rule> {
    let id = need_str(v, "id", "rule")?;
    let mut when = vec![];
    let w = v
        .get("when")
        .and_then(Value::as_object)
        .ok_or_else(|| usage("a rule needs \"when\" (an object)"))?;
    for (k, x) in w {
        let list = |x: &Value| -> Result<Vec<String>> { strings(&json!({"x": x}), "x") };
        let c = match k.as_str() {
            "description_has_alias" => Cond::DescriptionHasAlias(x.as_str().unwrap_or("").into()),
            "payee" => Cond::Payee(x.as_str().unwrap_or("").into()),
            "payee_has" => Cond::PayeeHas(list(x)?),
            "description_has" => Cond::DescriptionHas(list(x)?),
            "account_in" => Cond::AccountIn(list(x)?),
            "org_in" => Cond::OrgIn(list(x)?),
            "amount_between" => {
                let (a, b) = pair_of(x).ok_or_else(|| usage("amount_between is [low, high]"))?;
                let p = |t: &str| crate::money::parse(t).map_err(|e| usage(e.to_string()));
                Cond::AmountBetween(p(&a)?, p(&b)?)
            }
            "date_between" => {
                let (a, b) = pair_of(x).ok_or_else(|| usage("date_between is [from, to]"))?;
                Cond::DateBetween(a, b)
            }
            "sign" => match x.as_str() {
                Some("in") => Cond::Sign(Sign::In),
                Some("out") => Cond::Sign(Sign::Out),
                _ => return Err(usage("sign is \"in\" or \"out\"")),
            },
            "payee_is_org" => Cond::PayeeIsOrg(x.as_str().unwrap_or("").into()),
            "bank_transfer" => Cond::BankTransfer(
                x.as_bool()
                    .ok_or_else(|| usage("bank_transfer is true or false"))?,
            ),
            other => return Err(usage(format!("unknown rule condition \"{other}\""))),
        };
        when.push(c);
    }
    let mut set = vec![];
    let mut group_kind = None;
    for (k, x) in v
        .get("set")
        .and_then(Value::as_object)
        .ok_or_else(|| usage("a rule needs \"set\" (an object)"))?
    {
        let val = x
            .as_str()
            .ok_or_else(|| usage("rule set values are strings"))?;
        if k == "group_kind" {
            group_kind = Some(val.to_string());
        } else if val == "$matched" {
            set.push((k.clone(), SetValue::Matched));
        } else {
            set.push((k.clone(), SetValue::Choice(val.to_string())));
        }
    }
    let confidence = match v.get("confidence") {
        None | Some(Value::Null) => Decimal::ONE,
        Some(x) => dec_text(x)
            .and_then(|t| t.parse::<Decimal>().ok())
            .ok_or_else(|| usage("confidence is a number between 0 and 1"))?,
    };
    Ok(Rule {
        id,
        priority: v.get("priority").and_then(Value::as_i64).unwrap_or(0),
        when,
        set,
        group_kind,
        confidence,
        archived: v.get("archived").and_then(Value::as_bool).unwrap_or(false),
    })
}

pub fn rule_to_json(r: &Rule) -> Value {
    let mut when = Map::new();
    for c in &r.when {
        let v = match c {
            Cond::DescriptionHasAlias(x) | Cond::Payee(x) => json!(x),
            Cond::PayeeHas(x) | Cond::DescriptionHas(x) | Cond::AccountIn(x) | Cond::OrgIn(x) => {
                json!(x)
            }
            Cond::AmountBetween(a, b) => json!([a.to_string(), b.to_string()]),
            Cond::DateBetween(a, b) => json!([a, b]),
            Cond::Sign(Sign::In) => json!("in"),
            Cond::Sign(Sign::Out) => json!("out"),
            Cond::PayeeIsOrg(s) => json!(s),
            Cond::BankTransfer(b) => json!(b),
        };
        when.insert(c.key().into(), v);
    }
    let mut set = Map::new();
    for (k, v) in &r.set {
        set.insert(
            k.clone(),
            json!(match v {
                SetValue::Choice(c) => c.as_str(),
                SetValue::Matched => "$matched",
            }),
        );
    }
    if let Some(g) = &r.group_kind {
        set.insert("group_kind".into(), json!(g));
    }
    json!({"id": r.id, "priority": r.priority, "when": when, "set": set,
           "confidence": r.confidence.to_string(), "archived": r.archived})
}

/// A config edit from JSON: `{"edit": "add_choice", "label_type": "project", "id": …, "name": …, "aliases": […]}`.
pub fn edit_from_json(v: &Value) -> Result<ConfigEdit> {
    let edit = need_str(v, "edit", "config")?;
    let lt = || need_str(v, "label_type", &edit);
    let id = || need_str(v, "id", &edit);
    Ok(match edit.as_str() {
        "add_choice" => ConfigEdit::AddChoice {
            label_type: lt()?,
            choice: Choice {
                id: id()?,
                name: need_str(v, "name", &edit)?,
                aliases: strings(v, "aliases")?,
                archived: false,
            },
        },
        "edit_choice" => ConfigEdit::EditChoice {
            label_type: lt()?,
            id: id()?,
            name: s(v, "name").map(str::to_string),
            aliases: v
                .get("aliases")
                .map(|_| strings(v, "aliases"))
                .transpose()?,
        },
        "archive_choice" => ConfigEdit::ArchiveChoice {
            label_type: lt()?,
            id: id()?,
        },
        "restore_choice" => ConfigEdit::RestoreChoice {
            label_type: lt()?,
            id: id()?,
        },
        "remove_choice" => ConfigEdit::RemoveChoice {
            label_type: lt()?,
            id: id()?,
        },
        "add_rule" => ConfigEdit::AddRule(rule_from_json(
            v.get("rule")
                .ok_or_else(|| usage("add_rule needs \"rule\""))?,
        )?),
        "edit_rule" => ConfigEdit::EditRule(rule_from_json(
            v.get("rule")
                .ok_or_else(|| usage("edit_rule needs \"rule\""))?,
        )?),
        "archive_rule" => ConfigEdit::ArchiveRule { id: id()? },
        "remove_rule" => ConfigEdit::RemoveRule { id: id()? },
        "set_priority" => ConfigEdit::SetPriority {
            id: id()?,
            priority: v
                .get("priority")
                .and_then(Value::as_i64)
                .ok_or_else(|| usage("set_priority needs \"priority\" (an integer)"))?,
        },
        other => return Err(usage(format!("unknown config edit \"{other}\""))),
    })
}

pub fn edit_to_json(e: &ConfigEdit) -> Value {
    match e {
        ConfigEdit::AddChoice { label_type, choice } => json!({"edit": "add_choice",
            "label_type": label_type, "id": choice.id, "name": choice.name, "aliases": choice.aliases}),
        ConfigEdit::EditChoice {
            label_type,
            id,
            name,
            aliases,
        } => {
            let mut m = json!({"edit": "edit_choice", "label_type": label_type, "id": id});
            if let Some(n) = name {
                m["name"] = json!(n);
            }
            if let Some(a) = aliases {
                m["aliases"] = json!(a);
            }
            m
        }
        ConfigEdit::ArchiveChoice { label_type, id } => {
            json!({"edit": "archive_choice", "label_type": label_type, "id": id})
        }
        ConfigEdit::RestoreChoice { label_type, id } => {
            json!({"edit": "restore_choice", "label_type": label_type, "id": id})
        }
        ConfigEdit::RemoveChoice { label_type, id } => {
            json!({"edit": "remove_choice", "label_type": label_type, "id": id})
        }
        ConfigEdit::AddRule(r) => json!({"edit": "add_rule", "rule": rule_to_json(r)}),
        ConfigEdit::EditRule(r) => json!({"edit": "edit_rule", "rule": rule_to_json(r)}),
        ConfigEdit::ArchiveRule { id } => json!({"edit": "archive_rule", "id": id}),
        ConfigEdit::RemoveRule { id } => json!({"edit": "remove_rule", "id": id}),
        ConfigEdit::SetPriority { id, priority } => {
            json!({"edit": "set_priority", "id": id, "priority": priority})
        }
    }
}

fn valid_client(c: &str) -> bool {
    !c.is_empty()
        && c.len() <= 40
        && c.chars()
            .all(|x| x.is_ascii_alphanumeric() || matches!(x, '-' | '_' | '.'))
}

impl ClassifyRequest {
    /// Parse a request document (the envelope's `v`/`kind` are checked too when present).
    pub fn from_json(v: &Value) -> Result<ClassifyRequest> {
        if let Some(k) = s(v, "kind").filter(|k| *k != "classify") {
            return Err(usage(format!("kind `{k}` is not a classify request")));
        }
        match v.get("cv").and_then(Value::as_i64) {
            Some(CV) => {}
            other => {
                return Err(usage(format!(
                    "classify schema version {} is not supported (this build speaks cv {CV})",
                    other.map_or("(missing)".to_string(), |x| x.to_string())
                )));
            }
        }
        let op = s(v, "op").unwrap_or("");
        let client = s(v, "client").unwrap_or("").to_string();
        if !valid_client(&client) {
            return Err(usage(
                "\"client\" is required: letters, digits, - _ . (at most 40), e.g. \"claude\"",
            ));
        }
        let expect_store_rev = v
            .get("expect_store_rev")
            .and_then(Value::as_i64)
            .ok_or_else(|| {
                usage("\"expect_store_rev\" is required (the store_rev you last read)")
                    .hint("Read it with `ownpurse classify queue --json` (store_rev).")
            })?;
        let opv = match op {
            "set" => {
                let labels = v
                    .get("labels")
                    .and_then(Value::as_object)
                    .filter(|m| !m.is_empty())
                    .ok_or_else(|| usage("op `set` needs \"labels\": {\"<type>\": \"<id>\"}"))?
                    .iter()
                    .map(|(k, x)| {
                        x.as_str()
                            .map(|x| (k.clone(), x.to_string()))
                            .ok_or_else(|| usage("label values are strings"))
                    })
                    .collect::<Result<Vec<_>>>()?;
                Op::Set {
                    targets: targets(v, op)?,
                    labels,
                }
            }
            "clear" => {
                let types = strings(v, "types")?;
                if types.is_empty() {
                    return Err(usage("op `clear` needs \"types\""));
                }
                Op::Clear {
                    targets: targets(v, op)?,
                    types,
                }
            }
            "accept" => Op::Accept {
                targets: targets(v, op)?,
            },
            "note" => {
                let target = match (s(v, "anchor"), s(v, "group")) {
                    (Some(a), None) => NoteTarget::Line(Anchor::parse(a)?),
                    (None, Some(g)) => NoteTarget::Group(g.to_string()),
                    _ => return Err(usage("op `note` needs \"anchor\" or \"group\"")),
                };
                Op::Note {
                    target,
                    text: need_str(v, "text", op)?,
                }
            }
            "group_create" => Op::GroupCreate {
                // `group_kind`, not `kind`: `kind` is the envelope's ("classify")
                kind: need_str(v, "group_kind", op)?,
                title: need_str(v, "title", op)?,
            },
            "group_add" => Op::GroupAdd {
                group: need_str(v, "group", op)?,
                anchors: anchors(v, "anchors")?,
                role: s(v, "role").map(str::to_string),
            },
            "group_remove" => Op::GroupRemove {
                group: need_str(v, "group", op)?,
                anchors: anchors(v, "anchors")?,
            },
            "group_status" => Op::GroupStatus {
                group: need_str(v, "group", op)?,
                status: need_str(v, "status", op)?,
            },
            "undo" => Op::Undo(match s(v, "what").unwrap_or("batch") {
                "batch" => UndoWhat::Batch,
                "events" => {
                    let ids: Vec<i64> = v
                        .get("events")
                        .and_then(Value::as_array)
                        .map(|a| a.iter().filter_map(Value::as_i64).collect())
                        .unwrap_or_default();
                    if ids.is_empty() {
                        return Err(usage("undo \"events\" needs event ids"));
                    }
                    UndoWhat::Events(ids)
                }
                "line" => UndoWhat::Line(Anchor::parse(&need_str(v, "anchor", op)?)?),
                other => {
                    return Err(usage(format!(
                        "undo \"what\" is batch, events or line (not {other})"
                    )));
                }
            }),
            "config" => {
                let edits = v
                    .get("edits")
                    .and_then(Value::as_array)
                    .filter(|a| !a.is_empty())
                    .ok_or_else(|| usage("op `config` needs \"edits\": [ … ]"))?
                    .iter()
                    .map(edit_from_json)
                    .collect::<Result<Vec<_>>>()?;
                Op::Config {
                    edits,
                    dry_run: v.get("dry_run").and_then(Value::as_bool).unwrap_or(false),
                    confirm: v.get("confirm").and_then(Value::as_bool).unwrap_or(false),
                }
            }
            other => {
                return Err(usage(format!(
                    "unknown classify op `{other}` (ops: {})",
                    OPS.join(", ")
                )));
            }
        };
        Ok(ClassifyRequest {
            op: opv,
            client,
            expect_store_rev,
            caption: s(v, "caption").map(str::to_string),
            allow_bulk: v
                .get("allow_bulk")
                .and_then(Value::as_bool)
                .unwrap_or(false),
            overwrite: v.get("overwrite").and_then(Value::as_bool).unwrap_or(false),
        })
    }

    /// The request document, envelope included (`{"v":1,"kind":"classify","cv":1,"op":…}`).
    pub fn to_json(&self) -> Value {
        let mut m = Map::new();
        m.insert("cv".into(), json!(CV));
        m.insert("client".into(), json!(self.client));
        m.insert("expect_store_rev".into(), json!(self.expect_store_rev));
        if let Some(c) = &self.caption {
            m.insert("caption".into(), json!(c));
        }
        if self.allow_bulk {
            m.insert("allow_bulk".into(), json!(true));
        }
        if self.overwrite {
            m.insert("overwrite".into(), json!(true));
        }
        match &self.op {
            Op::Set { targets, labels } => {
                targets_json(targets, &mut m);
                let l: Map<String, Value> =
                    labels.iter().map(|(k, v)| (k.clone(), json!(v))).collect();
                m.insert("labels".into(), Value::Object(l));
            }
            Op::Clear { targets, types } => {
                targets_json(targets, &mut m);
                m.insert("types".into(), json!(types));
            }
            Op::Accept { targets } => targets_json(targets, &mut m),
            Op::Note { target, text } => {
                match target {
                    NoteTarget::Line(a) => m.insert("anchor".into(), json!(a.to_string())),
                    NoteTarget::Group(g) => m.insert("group".into(), json!(g)),
                };
                m.insert("text".into(), json!(text));
            }
            Op::GroupCreate { kind, title } => {
                m.insert("group_kind".into(), json!(kind));
                m.insert("title".into(), json!(title));
            }
            Op::GroupAdd {
                group,
                anchors,
                role,
            } => {
                m.insert("group".into(), json!(group));
                targets_json(&Targets::Anchors(anchors.clone()), &mut m);
                if let Some(r) = role {
                    m.insert("role".into(), json!(r));
                }
            }
            Op::GroupRemove { group, anchors } => {
                m.insert("group".into(), json!(group));
                targets_json(&Targets::Anchors(anchors.clone()), &mut m);
            }
            Op::GroupStatus { group, status } => {
                m.insert("group".into(), json!(group));
                m.insert("status".into(), json!(status));
            }
            Op::Undo(w) => match w {
                UndoWhat::Batch => {
                    m.insert("what".into(), json!("batch"));
                }
                UndoWhat::Events(ids) => {
                    m.insert("what".into(), json!("events"));
                    m.insert("events".into(), json!(ids));
                }
                UndoWhat::Line(a) => {
                    m.insert("what".into(), json!("line"));
                    m.insert("anchor".into(), json!(a.to_string()));
                }
            },
            Op::Config {
                edits,
                dry_run,
                confirm,
            } => {
                m.insert(
                    "edits".into(),
                    Value::Array(edits.iter().map(edit_to_json).collect()),
                );
                m.insert("dry_run".into(), json!(dry_run));
                m.insert("confirm".into(), json!(confirm));
            }
        }
        crate::bridge::request_doc("classify", self.op.name(), m)
    }

    fn ctx(&self, batch: &str) -> WriteCtx {
        WriteCtx {
            actor: Actor::agent(&self.client),
            batch: BatchId::new(batch),
            caption: self.caption.clone(),
            allow_bulk: self.allow_bulk,
        }
    }
}

fn plural(n: usize, w: &str) -> String {
    super::config::plural(n, w)
}

impl Workspace {
    /// The lines a request would touch (the TUI checks them for `user_busy` before applying). A filter
    /// resolves inside the scope, `status:open` unless the filter names a status.
    pub fn resolve_targets(&self, req: &ClassifyRequest) -> Result<Vec<Anchor>> {
        let t = match &req.op {
            Op::Set { targets, .. } | Op::Clear { targets, .. } | Op::Accept { targets } => {
                targets.clone()
            }
            Op::GroupAdd { anchors, .. } | Op::GroupRemove { anchors, .. } => {
                Targets::Anchors(anchors.clone())
            }
            Op::Note {
                target: NoteTarget::Line(a),
                ..
            } => Targets::Anchors(vec![a.clone()]),
            Op::Undo(UndoWhat::Line(a)) => Targets::Anchors(vec![a.clone()]),
            _ => return Ok(vec![]),
        };
        match t {
            Targets::Anchors(a) => {
                let mut seen = std::collections::HashSet::new();
                let mut out = vec![];
                for x in a {
                    if self.line(&x).is_none() {
                        return Err(ClassifyError::new(
                            "not_found",
                            format!("no classifiable line {x}"),
                        ));
                    }
                    if seen.insert(x.clone()) {
                        out.push(x);
                    }
                }
                Ok(out)
            }
            Targets::Filter(text) => {
                let mut f = Filter::parse(&text)?;
                let says_status = text
                    .split_whitespace()
                    .any(|w| w.starts_with("status:") || matches!(w, "open" | "all" | "stale"));
                if !says_status {
                    f.status = Status::Open;
                }
                Ok(self
                    .queue(&f, super::view::Sort::DateAsc)
                    .into_iter()
                    .map(|r| r.anchor)
                    .collect())
            }
        }
    }

    /// Apply one agent request: the single write path for the CLI and the TUI bridge. Checks, in order: the
    /// caption, the store revision, the targets and the bulk limit; then writes one batch.
    pub fn apply(&mut self, req: &ClassifyRequest) -> Result<ApplyOutcome> {
        if let Some(c) = &req.caption
            && let Some(why) = crate::bridge::caption_error(c)
        {
            return Err(ClassifyError::new("bad_caption", why));
        }
        // pick up other writers first, so the revision compared is the store's, not this handle's
        self.refresh()?;
        self.expect_rev(req.expect_store_rev)?;
        let anchors = self.resolve_targets(req)?;
        let bulk = matches!(
            req.op,
            Op::Set { .. }
                | Op::Clear { .. }
                | Op::Accept { .. }
                | Op::GroupAdd { .. }
                | Op::GroupRemove { .. }
        );
        if bulk && anchors.len() > super::view::BULK_LIMIT && !req.allow_bulk {
            return Err(ClassifyError::new(
                "too_many",
                format!(
                    "this send would write {} lines; the limit is {} without allow_bulk",
                    anchors.len(),
                    super::view::BULK_LIMIT
                ),
            )
            .hint("Narrow the filter, or pass --allow-bulk."));
        }
        let batch = format!("{}-{}-{}", req.client, super::store::now_iso(), self.rev());
        let w = req.ctx(&batch);
        let mut out = ApplyOutcome {
            op: req.op.name().into(),
            batch: batch.clone(),
            ..ApplyOutcome::default()
        };
        match &req.op {
            Op::Set { labels, .. } => {
                self.apply_labels(&anchors, labels, req.overwrite, &w, &mut out)?
            }
            Op::Clear { types, .. } => {
                self.apply_clear(&anchors, types, req.overwrite, &w, &mut out)?
            }
            Op::Accept { .. } => self.apply_accept(&anchors, req.overwrite, &w, &mut out)?,
            Op::Note { target, text } => {
                let o = self.add_note(target, text, &w)?;
                out.events = o.events;
                out.lines = o.lines;
                out.anchors = anchors;
                out.summary = o.summary;
            }
            Op::GroupCreate { kind, title } => {
                let (o, g) = self.create_group(kind, title, &w)?;
                out.events = o.events;
                out.summary = o.summary;
                out.group = Some(g);
            }
            Op::GroupAdd { group, role, .. } => {
                for a in &anchors {
                    let o = self.add_member(group, a, role.as_deref(), &w)?;
                    out.events.extend(o.events);
                }
                let title = self
                    .groups()
                    .iter()
                    .find(|g| &g.id == group)
                    .map(|g| g.title.clone());
                out.lines = anchors.len();
                out.summary = format!(
                    "added {} {} to group \"{}\"",
                    anchors.len(),
                    plural(anchors.len(), "line"),
                    title.unwrap_or_default()
                );
                out.anchors = anchors;
            }
            Op::GroupRemove { group, .. } => {
                let title = self
                    .groups()
                    .iter()
                    .find(|g| &g.id == group)
                    .map(|g| g.title.clone())
                    .unwrap_or_default();
                for a in &anchors {
                    let o = self.remove_member(group, a, &w)?;
                    out.events.extend(o.events);
                }
                out.lines = anchors.len();
                out.summary = format!(
                    "removed {} {} from group \"{title}\"",
                    anchors.len(),
                    plural(anchors.len(), "line")
                );
                out.anchors = anchors;
            }
            Op::GroupStatus { group, status } => {
                let o = self.set_group_status(group, status, &w)?;
                out.events = o.events;
                out.summary = o.summary;
            }
            Op::Undo(what) => self.apply_undo(what, &w, &mut out)?,
            Op::Config {
                edits,
                dry_run,
                confirm,
            } => {
                let staged = self.stage_config(edits)?;
                out.diff = Some(staged.diff.clone());
                out.dry_run = *dry_run;
                if *dry_run {
                    out.summary = format!(
                        "would change {}: {}",
                        self.config_file().name(),
                        staged.summary.join("; ")
                    );
                } else {
                    if !confirm {
                        return Err(usage(
                            "a config write by an agent needs \"confirm\": true (send dry_run first to see the diff)",
                        ));
                    }
                    let o = self.write_config(&staged, &w)?;
                    out.events = o.events;
                    out.summary = o.summary;
                }
            }
        }
        out.store_rev = self.rev();
        Ok(out)
    }

    /// A human-confirmed label on this type (left alone unless `overwrite`).
    fn human_label(&self, a: &Anchor, ty: &str) -> bool {
        self.label(a, ty).is_some_and(|l| l.actor.is_human())
    }

    fn skipped_suffix(out: &ApplyOutcome) -> String {
        let n = out
            .skipped
            .iter()
            .map(|s| &s.anchor)
            .collect::<std::collections::HashSet<_>>()
            .len();
        if n == 0 {
            String::new()
        } else {
            format!(" ({n} skipped: confirmed by you)")
        }
    }

    fn apply_labels(
        &mut self,
        anchors: &[Anchor],
        labels: &[(String, String)],
        overwrite: bool,
        w: &WriteCtx,
        out: &mut ApplyOutcome,
    ) -> Result<()> {
        let mut items = vec![];
        for a in anchors {
            let line = self.need_line(a)?.clone();
            let mut entries = vec![];
            for (ty, v) in labels {
                self.check_value(&line, ty, v)?;
                if !overwrite && self.human_label(a, ty) {
                    out.skipped.push(Skipped {
                        anchor: a.clone(),
                        label_type: ty.clone(),
                        reason: "human_label",
                    });
                    continue;
                }
                if self
                    .label(a, ty)
                    .is_some_and(|l| &l.value == v && !self.is_stale(l))
                {
                    continue;
                }
                entries.push(label_json(
                    a,
                    ty,
                    v,
                    &Source::Agent(self_client(w)),
                    None,
                    &[],
                    &line.fingerprint,
                ));
            }
            if !entries.is_empty() {
                out.anchors.push(a.clone());
                items.push((
                    EventKind::LabelSet,
                    with_caption(json!({"labels": entries}), w),
                ));
            }
        }
        out.lines = items.len();
        out.events = self.store_mut().append(&w.actor, &w.batch, items)?;
        let what = labels
            .iter()
            .map(|(t, v)| {
                let title = self
                    .config()
                    .label_type(t)
                    .map_or(t.clone(), |x| x.title.to_lowercase());
                format!("{title} {}", self.config().choice_name(t, v))
            })
            .collect::<Vec<_>>()
            .join(", ");
        out.summary = format!(
            "set {what} on {} {}{}",
            out.lines,
            plural(out.lines, "line"),
            Self::skipped_suffix(out)
        );
        Ok(())
    }

    fn apply_clear(
        &mut self,
        anchors: &[Anchor],
        types: &[String],
        overwrite: bool,
        w: &WriteCtx,
        out: &mut ApplyOutcome,
    ) -> Result<()> {
        let mut items = vec![];
        for a in anchors {
            let mut entries = vec![];
            for ty in types {
                if self.label(a, ty).is_none() {
                    continue;
                }
                if !overwrite && self.human_label(a, ty) {
                    out.skipped.push(Skipped {
                        anchor: a.clone(),
                        label_type: ty.clone(),
                        reason: "human_label",
                    });
                    continue;
                }
                entries.push(json!({"anchor": a.to_string(), "type": ty}));
            }
            if !entries.is_empty() {
                out.anchors.push(a.clone());
                items.push((
                    EventKind::LabelCleared,
                    with_caption(json!({"labels": entries}), w),
                ));
            }
        }
        out.lines = items.len();
        out.events = self.store_mut().append(&w.actor, &w.batch, items)?;
        out.summary = format!(
            "cleared {} on {} {}{}",
            types.join(", "),
            out.lines,
            plural(out.lines, "line"),
            Self::skipped_suffix(out)
        );
        Ok(())
    }

    fn apply_accept(
        &mut self,
        anchors: &[Anchor],
        overwrite: bool,
        w: &WriteCtx,
        out: &mut ApplyOutcome,
    ) -> Result<()> {
        let mut items = vec![];
        for a in anchors {
            let line = self.need_line(a)?.clone();
            let mut entries = self.accept_entries(&line, &w.actor);
            if !overwrite {
                entries.retain(|e| {
                    let ty = e["type"].as_str().unwrap_or("");
                    if self.human_label(a, ty) {
                        out.skipped.push(Skipped {
                            anchor: a.clone(),
                            label_type: ty.to_string(),
                            reason: "human_label",
                        });
                        false
                    } else {
                        true
                    }
                });
            }
            if !entries.is_empty() {
                out.anchors.push(a.clone());
                items.push((
                    EventKind::LabelSet,
                    with_caption(json!({"labels": entries}), w),
                ));
            }
        }
        out.lines = items.len();
        out.events = self.store_mut().append(&w.actor, &w.batch, items)?;
        out.summary = format!(
            "accepted the suggestions on {} {}{}",
            out.lines,
            plural(out.lines, "line"),
            Self::skipped_suffix(out)
        );
        Ok(())
    }

    fn apply_undo(&mut self, what: &UndoWhat, w: &WriteCtx, out: &mut ApplyOutcome) -> Result<()> {
        let not_agent = |id: i64, actor: &str| {
            ClassifyError::new(
                "not_found",
                format!("event {id} was written by {actor}: an agent can undo only agent writes"),
            )
        };
        let o = match what {
            UndoWhat::Batch => self.undo_batch(w)?,
            UndoWhat::Events(ids) => {
                for id in ids {
                    let e = self
                        .store()
                        .event(*id)
                        .ok_or_else(|| ClassifyError::new("not_found", format!("no event {id}")))?;
                    if !e.actor.is_agent() {
                        return Err(not_agent(*id, e.actor.as_str()));
                    }
                }
                self.undo_events(ids, w)?
            }
            UndoWhat::Line(a) => {
                let last = self
                    .store()
                    .state()
                    .writes_by_anchor
                    .get(a)
                    .and_then(|ids| {
                        ids.iter()
                            .rev()
                            .copied()
                            .find(|id| self.store().event(*id).is_some_and(|e| e.actor.is_agent()))
                    })
                    .ok_or_else(|| {
                        ClassifyError::new(
                            "not_found",
                            format!("no agent write to undo on {a} (an agent can undo only agent writes)"),
                        )
                    })?;
                self.undo_events(&[last], w)?
            }
        };
        out.events = o.events;
        out.lines = o.lines;
        out.summary = o.summary;
        Ok(())
    }
}

fn self_client(w: &WriteCtx) -> String {
    w.actor
        .as_str()
        .strip_prefix("agent:")
        .unwrap_or("agent")
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::classify::config::{ConfigFile, tests::SAMPLE};
    use crate::classify::rules::tests::line;
    use crate::classify::store::{Store, pin_clock};

    fn ws(name: &str, n: usize) -> Workspace {
        pin_clock(Some("2026-10-05T09:46:00.000000Z"));
        let d = std::env::temp_dir().join(format!("xa-classify-req-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        std::fs::write(d.join("classify.toml"), SAMPLE).unwrap();
        let config = ConfigFile::load(&d.join("classify.toml")).unwrap();
        let store = Store::open_lazy(&d.join("db/classify.sqlite")).unwrap();
        let lines = (0..n)
            .map(|i| {
                let payee = if i % 2 == 0 {
                    "Hollowpine Packaging Works"
                } else {
                    "Marrowby Signs"
                };
                line(
                    "roast",
                    &format!("bt-{i}"),
                    payee,
                    "Bag proofs",
                    "-41.27",
                    "5310",
                    "2025-04-17",
                )
            })
            .collect();
        Workspace::from_parts(config, store, lines, vec![])
    }

    fn a(i: usize) -> Anchor {
        Anchor::line(
            "roast",
            crate::classify::DocType::BankTransaction,
            &format!("bt-{i}"),
            "L1",
        )
    }

    fn req(w: &Workspace, op: Value) -> ClassifyRequest {
        let mut v = op;
        v["v"] = json!(1);
        v["kind"] = json!("classify");
        v["cv"] = json!(1);
        v["client"] = json!("claude");
        if v.get("expect_store_rev").is_none() {
            v["expect_store_rev"] = json!(w.rev());
        }
        ClassifyRequest::from_json(&v).unwrap()
    }

    #[test]
    fn json_round_trips_for_every_op() {
        let w = ws("rt", 2);
        for op in [
            json!({"op": "set", "anchors": ["roast:bt:bt-0:L1"], "labels": {"project": "harvest"}, "caption": "c", "overwrite": true}),
            json!({"op": "clear", "filter": "org:roast status:all", "types": ["project"]}),
            json!({"op": "accept", "filter": "org:roast", "allow_bulk": true}),
            json!({"op": "note", "anchor": "roast:bt:bt-0:L1", "text": "t"}),
            json!({"op": "note", "group": "g3", "text": "t"}),
            json!({"op": "group_create", "group_kind": "split", "title": "x"}),
            json!({"op": "group_add", "group": "g3", "anchors": ["roast:bt:bt-0:L1"], "role": "origin"}),
            json!({"op": "group_remove", "group": "g3", "anchors": ["roast:bt:bt-0:L1"]}),
            json!({"op": "group_status", "group": "g3", "status": "gap"}),
            json!({"op": "undo", "what": "batch"}),
            json!({"op": "undo", "what": "events", "events": [3, 4]}),
            json!({"op": "undo", "what": "line", "anchor": "roast:bt:bt-0:L1"}),
            json!({"op": "config", "edits": [
                {"edit": "add_choice", "label_type": "project", "id": "x", "name": "X", "aliases": ["xx"]},
                {"edit": "add_rule", "rule": {"id": "r-n", "priority": 3, "when": {"payee_has": ["a"], "amount_between": ["-9.13", "-1.07"], "sign": "out"}, "set": {"bearer": "hold"}, "confidence": "0.75"}},
                {"edit": "set_priority", "id": "r-n", "priority": 7}], "dry_run": true}),
        ] {
            let r = req(&w, op);
            assert_eq!(
                ClassifyRequest::from_json(&r.to_json()).unwrap(),
                r,
                "{:?}",
                r.op
            );
        }
        for (bad, why) in [
            (
                json!({"v": 1, "kind": "classify", "cv": 2, "op": "set", "client": "c", "expect_store_rev": 0}),
                "cv 1",
            ),
            (
                json!({"cv": 1, "op": "set", "client": "", "expect_store_rev": 0}),
                "client",
            ),
            (
                json!({"cv": 1, "op": "set", "client": "c"}),
                "expect_store_rev",
            ),
            (
                json!({"cv": 1, "op": "frob", "client": "c", "expect_store_rev": 0}),
                "unknown classify op",
            ),
            (
                json!({"cv": 1, "op": "set", "client": "c", "expect_store_rev": 0, "labels": {"project": "x"}}),
                "anchors",
            ),
        ] {
            let e = ClassifyRequest::from_json(&bad).unwrap_err();
            assert_eq!(e.code, "usage");
            assert!(e.message.contains(why), "{} / {why}", e.message);
        }
    }

    #[test]
    fn guards_rev_caption_bulk() {
        let mut w = ws("guards", 60);
        let mut r = req(
            &w,
            json!({"op": "set", "filter": "org:roast", "labels": {"project": "harvest"}}),
        );
        let e = w.apply(&r).unwrap_err();
        assert_eq!((e.code, e.exit_code()), ("too_many", 2));
        assert!(e.message.contains("60 lines"));
        r.allow_bulk = true;
        r.expect_store_rev = 7;
        let e = w.apply(&r).unwrap_err();
        assert_eq!((e.code, e.exit_code()), ("view_changed", 4));
        assert!(e.message.contains("store rev 7"), "{}", e.message);
        r.expect_store_rev = 0;
        r.caption = Some("two\nlines".into());
        let e = w.apply(&r).unwrap_err();
        assert_eq!((e.code, e.exit_code()), ("bad_caption", 2));
        r.caption = Some("x".repeat(121));
        assert_eq!(w.apply(&r).unwrap_err().code, "bad_caption");
        r.caption = Some("Bag proofs are the launch".into());
        let out = w.apply(&r).unwrap();
        assert_eq!(out.lines, 60);
        assert_eq!(out.store_rev, 60);
        let j = out.to_json();
        assert_eq!(j["ok"], true);
        assert_eq!(j["kind"], "classify");
        assert_eq!(j["summary"], "set project Harvest Blend launch on 60 lines");
        let e = error_response(&ClassifyError::new("too_many", "x"));
        assert_eq!(
            (e["error"].as_str(), e["exit"].as_i64()),
            (Some("too_many"), Some(2))
        );
    }

    #[test]
    fn human_labels_are_skipped_unless_overwrite_and_filters_default_to_open() {
        let mut w = ws("human", 4);
        let h = WriteCtx::new(Actor::human(), BatchId::new("h"));
        w.set_label(&a(0), "project", "refit", &h).unwrap();
        // a fully labelled line leaves the open queue
        for (t, v) in [
            ("project", "none"),
            ("bearer", "hold"),
            ("treatment", "keep"),
        ] {
            w.set_label(&a(3), t, v, &h).unwrap();
        }
        let r = req(
            &w,
            json!({"op": "set", "filter": "org:roast", "labels": {"project": "harvest"}}),
        );
        assert_eq!(
            w.resolve_targets(&r).unwrap().len(),
            3,
            "status:open by default"
        );
        let out = w.apply(&r).unwrap();
        assert_eq!(out.lines, 2);
        assert_eq!(out.skipped.len(), 1);
        assert_eq!(
            (out.skipped[0].anchor.clone(), out.skipped[0].reason),
            (a(0), "human_label")
        );
        assert_eq!(
            out.summary,
            "set project Harvest Blend launch on 2 lines (1 skipped: confirmed by you)"
        );
        assert_eq!(out.to_json()["skipped"][0]["type"], "project");
        let d = w.detail(&a(1)).unwrap();
        let l = d.labels[0].label.clone().unwrap();
        assert_eq!(
            (l.source.to_string(), l.actor.to_string()),
            ("agent:claude".into(), "agent:claude".into())
        );
        // agent over agent may overwrite; status:all reaches the closed line too
        let r = req(
            &w,
            json!({"op": "set", "filter": "org:roast status:all", "labels": {"project": "refit"}, "overwrite": true}),
        );
        let out = w.apply(&r).unwrap();
        assert_eq!(
            (out.lines, out.skipped.len()),
            (3, 0),
            "line 0 is already refit"
        );
        assert_eq!(
            w.detail(&a(3)).unwrap().labels[0]
                .label
                .as_ref()
                .unwrap()
                .value,
            "refit"
        );
        // clear and accept skip human labels too
        let r = req(
            &w,
            json!({"op": "clear", "anchors": ["roast:bt:bt-0:L1", "roast:bt:bt-1:L1"], "types": ["project"]}),
        );
        let out = w.apply(&r).unwrap();
        assert_eq!((out.lines, out.skipped.len()), (1, 1));
        let s = crate::classify::view::Filter::parse("status:all").unwrap();
        w.suggest(&s, &WriteCtx::new(Actor::rule("engine"), BatchId::new("s")))
            .unwrap();
        let r = req(
            &w,
            json!({"op": "accept", "anchors": ["roast:bt:bt-0:L1", "roast:bt:bt-2:L1"]}),
        );
        let out = w.apply(&r).unwrap();
        assert!(out.skipped.iter().all(|s| s.anchor == a(0)));
        assert_eq!(
            out.lines, 2,
            "bt-0 still gets its bearer; its project is the human's"
        );
        assert_eq!(
            w.detail(&a(0)).unwrap().labels[0]
                .label
                .as_ref()
                .unwrap()
                .value,
            "refit"
        );
    }

    #[test]
    fn agent_undo_touches_only_agent_writes() {
        let mut w = ws("undo", 3);
        let h = WriteCtx::new(Actor::human(), BatchId::new("h"));
        w.set_label(&a(0), "bearer", "hold", &h).unwrap();
        let r = req(
            &w,
            json!({"op": "undo", "what": "line", "anchor": "roast:bt:bt-0:L1"}),
        );
        let e = w.apply(&r).unwrap_err();
        assert_eq!(e.code, "not_found");
        assert!(e.message.contains("only agent writes"));
        let r = req(&w, json!({"op": "undo", "what": "events", "events": [1]}));
        assert!(
            w.apply(&r)
                .unwrap_err()
                .message
                .contains("written by human")
        );
        // an agent batch over two lines, undone as one
        let r = req(
            &w,
            json!({"op": "set", "anchors": ["roast:bt:bt-1:L1", "roast:bt:bt-2:L1"], "labels": {"project": "harvest", "bearer": "roast"}, "caption": "launch"}),
        );
        let set = w.apply(&r).unwrap();
        assert_eq!(set.events.len(), 2);
        let f = crate::classify::view::Filter::parse("by:agent since:today status:all").unwrap();
        assert_eq!(w.queue(&f, crate::classify::view::Sort::DateAsc).len(), 2);
        // the line undo takes the agent write, not the human one before it
        w.apply(&req(
            &w,
            json!({"op": "set", "anchors": ["roast:bt:bt-0:L1"], "labels": {"project": "refit"}}),
        ))
        .unwrap();
        w.apply(&req(
            &w,
            json!({"op": "undo", "what": "line", "anchor": "roast:bt:bt-0:L1"}),
        ))
        .unwrap();
        let d = w.detail(&a(0)).unwrap();
        assert!(d.labels[0].label.is_none());
        assert_eq!(
            d.labels[1].label.as_ref().unwrap().value,
            "hold",
            "the human label stays"
        );
        // undo batch: the set on lines 1 and 2
        let out = w
            .apply(&req(&w, json!({"op": "undo", "what": "batch"})))
            .unwrap();
        assert_eq!(out.summary, "agent write undone · 2 lines");
        assert!(
            w.queue(&f, crate::classify::view::Sort::DateAsc).is_empty()
                || w.queue(&f, crate::classify::view::Sort::DateAsc)
                    .iter()
                    .all(|r| r.anchor == a(0))
        );
        assert!(w.detail(&a(1)).unwrap().labels[0].label.is_none());
    }

    #[test]
    fn config_needs_confirm_and_dry_run_returns_the_diff() {
        let mut w = ws("config", 1);
        let edits = json!([{"edit": "add_choice", "label_type": "project", "id": "market", "name": "Holiday market stall"}]);
        let dry = w
            .apply(&req(
                &w,
                json!({"op": "config", "edits": edits, "dry_run": true}),
            ))
            .unwrap();
        assert!(
            dry.diff
                .as_ref()
                .unwrap()
                .contains("+  { id = \"market\", name = \"Holiday market stall\" },")
        );
        assert!(dry.events.is_empty() && dry.dry_run);
        assert_eq!(
            dry.summary,
            "would change classify.toml: added project \"Holiday market stall\""
        );
        let e = w
            .apply(&req(&w, json!({"op": "config", "edits": edits})))
            .unwrap_err();
        assert!(e.message.contains("\"confirm\": true"));
        let out = w
            .apply(&req(
                &w,
                json!({"op": "config", "edits": edits, "confirm": true}),
            ))
            .unwrap();
        assert_eq!(out.events.len(), 1);
        assert!(out.diff.is_some());
        assert_eq!(w.choices("project").len(), 4);
        w.apply(&req(&w, json!({"op": "undo", "what": "batch"})))
            .unwrap();
        assert_eq!(w.choices("project").len(), 3);
    }

    #[test]
    fn groups_and_notes_through_apply() {
        let mut w = ws("groups", 2);
        let out = w
            .apply(&req(
                &w,
                json!({"op": "group_create", "group_kind": "split", "title": "Proof run"}),
            ))
            .unwrap();
        let g = out.group.clone().unwrap();
        let out = w
            .apply(&req(&w, json!({"op": "group_add", "group": g, "anchors": ["roast:bt:bt-0:L1", "roast:bt:bt-1:L1"], "role": "origin"})))
            .unwrap();
        assert_eq!(out.summary, "added 2 lines to group \"Proof run\"");
        w.apply(&req(
            &w,
            json!({"op": "group_status", "group": g, "status": "accountant"}),
        ))
        .unwrap();
        w.apply(&req(
            &w,
            json!({"op": "note", "group": g, "text": "ask about the split"}),
        ))
        .unwrap();
        let d = w.group_detail(&g).unwrap();
        assert_eq!(
            (d.members.len(), d.group.status.as_str(), d.notes.len()),
            (2, "accountant", 1)
        );
    }
}
