//! `classify.toml`: the catalog (label types and their choices), rules, the queue's required types, group
//! kinds and roles, the Jev switch (must be off in v0.11.0) and the decision tree (validated only as far as
//! needed to accept the spec's example; not executed in this release).
//!
//! Parsing keeps spans, so every problem reads `classify.toml:42: rules[3].set.bearer = "fa2": unknown choice
//! (did you mean "fa"?)`. Edits go through [`ConfigEdit`] on a format-preserving document (`toml_edit`: hand
//! comments survive), are staged with a unified diff, and are written atomically (tmp, fsync, rotate the old
//! file to `classify.toml.~N~`, keep 20, rename). A write is refused if the file changed on disk since it was
//! loaded. Removing an item that is in use becomes archiving it (V20).

use super::{ClassifyError, Confidence, Result};
use rust_decimal::Decimal;
use std::path::{Path, PathBuf};

/// The fixed allow-list of line fields a Jev request may carry (V17).
pub const JEV_FIELDS: [&str; 4] = ["description", "payee", "amount", "date"];
/// Numbered backups kept beside the config.
pub const KEEP_BACKUPS: usize = 20;

/// One validation problem, with its place in the file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigIssue {
    /// The file name as shown (`classify.toml`).
    pub file: String,
    /// 1-based line, 0 when unknown.
    pub line: usize,
    /// Dotted path, e.g. `rules[3].set.bearer`.
    pub path: String,
    pub reason: String,
}

impl std::fmt::Display for ConfigIssue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // a reason that quotes the value reads `path = "x": why`; any other reads `path: why`
        let sep = if self.reason.starts_with("= ") {
            " "
        } else {
            ": "
        };
        if self.line > 0 {
            write!(
                f,
                "{}:{}: {}{sep}{}",
                self.file, self.line, self.path, self.reason
            )
        } else {
            write!(f, "{}: {}{sep}{}", self.file, self.path, self.reason)
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LabelKind {
    /// Choices listed in the file.
    Choice,
    /// Choices are the line's own org chart (`kind = "account"`).
    Account,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Choice {
    pub id: String,
    pub name: String,
    pub aliases: Vec<String>,
    /// Archived choices are hidden from pickers but stay valid for history (V20).
    pub archived: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LabelType {
    pub id: String,
    pub title: String,
    pub kind: LabelKind,
    /// In file order (the order shown in pickers and the detail pane; `p`+digit picks the N-th active one).
    pub choices: Vec<Choice>,
    pub archived: bool,
}

impl LabelType {
    pub fn choice(&self, id: &str) -> Option<&Choice> {
        self.choices.iter().find(|c| c.id == id)
    }
    /// Active (not archived) choices, in order.
    pub fn active(&self) -> impl Iterator<Item = &Choice> {
        self.choices.iter().filter(|c| !c.archived)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sign {
    /// Money in (amount > 0).
    In,
    /// Money out (amount < 0).
    Out,
}

/// One rule condition. All conditions of a rule must hold (AND).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Cond {
    /// Any alias (or the name) of any choice of this label type, whole word, case-insensitive; the matched
    /// choice is `$matched`.
    DescriptionHasAlias(String),
    /// Payee equals (case-insensitive).
    Payee(String),
    /// Payee contains any of these (case-insensitive).
    PayeeHas(Vec<String>),
    /// Description contains any of these (case-insensitive).
    DescriptionHas(Vec<String>),
    /// Account code is one of these.
    AccountIn(Vec<String>),
    /// Org alias is one of these.
    OrgIn(Vec<String>),
    /// `lo <= amount <= hi` (signed).
    AmountBetween(Decimal, Decimal),
    /// `from <= date <= to` (ISO, inclusive).
    DateBetween(String, String),
    Sign(Sign),
    /// The payee is one of the configured orgs (by its organisation name or legal name, ignoring case,
    /// punctuation and suffixes like Ltd, LLC, LLP, Inc, Co., GmbH): an alias, or `any` for any configured org.
    PayeeIsOrg(String),
    /// The line is (true) or is not (false) a bank transfer leg: a `SPEND-TRANSFER`/`RECEIVE-TRANSFER`
    /// transaction, or a line posted to a bank account.
    BankTransfer(bool),
}

impl Cond {
    /// The TOML key of this condition.
    pub fn key(&self) -> &'static str {
        match self {
            Cond::DescriptionHasAlias(_) => "description_has_alias",
            Cond::Payee(_) => "payee",
            Cond::PayeeHas(_) => "payee_has",
            Cond::DescriptionHas(_) => "description_has",
            Cond::AccountIn(_) => "account_in",
            Cond::OrgIn(_) => "org_in",
            Cond::AmountBetween(..) => "amount_between",
            Cond::DateBetween(..) => "date_between",
            Cond::Sign(_) => "sign",
            Cond::PayeeIsOrg(_) => "payee_is_org",
            Cond::BankTransfer(_) => "bank_transfer",
        }
    }
}

/// What a rule sets for one key.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SetValue {
    /// A choice id of the label type.
    Choice(String),
    /// `$matched`: the choice found by `description_has_alias`.
    Matched,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rule {
    pub id: String,
    /// Higher runs first; for each label type the highest-priority matching rule wins.
    pub priority: i64,
    pub when: Vec<Cond>,
    /// label type → value, in file order.
    pub set: Vec<(String, SetValue)>,
    /// `set = { group_kind = "split" }`: proposes a group of this kind (not a label).
    pub group_kind: Option<String>,
    /// Default 1.0.
    pub confidence: Confidence,
    pub archived: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JevConfig {
    /// Must be false in v0.11.0 (`jev_not_enabled` otherwise).
    pub enabled: bool,
    pub fields: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QueueConfig {
    /// Label types a line needs confirmed to leave the queue (default project, bearer, treatment).
    pub required: Vec<String>,
    /// Bulk-accept default threshold (default 0.90).
    pub threshold: Confidence,
    /// `scope = "year:2025"`: a filter ANDed into everything (queue, counts, suggest, bulk, export,
    /// would-match). Its status token is ignored. Default: no scope.
    pub scope: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GroupsConfig {
    pub kinds: Vec<String>,
    pub roles: Vec<String>,
    /// `[groups.pair_rules.<id>]`: the balance rule a settlement group's running balance is checked against.
    pub pair_rules: Vec<PairRule>,
    /// `[groups] settle_window_days`: how many days apart two legs of one settlement may be (default 7, as
    /// member suggestions: a bank transfer often lands a day or two later).
    pub settle_window_days: i64,
}

/// One side of a pair rule: an org and the accounts whose balances add (`plus`) or subtract (`minus`).
/// Codes may be ranges, `"121-131"` (numeric codes, inclusive).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PairSide {
    pub org: String,
    pub plus: Vec<String>,
    pub minus: Vec<String>,
}

impl PairSide {
    /// +1, -1 or 0: how an account code counts on this side.
    pub fn weight(&self, code: &str) -> i32 {
        if self.plus.iter().any(|p| code_in(code, p)) {
            1
        } else if self.minus.iter().any(|p| code_in(code, p)) {
            -1
        } else {
            0
        }
    }
}

/// `code` equals `pat`, or lies in the numeric range `lo-hi`.
pub fn code_in(code: &str, pat: &str) -> bool {
    if code == pat {
        return true;
    }
    let Some((lo, hi)) = pat.split_once('-') else {
        return false;
    };
    match (
        code.parse::<u64>(),
        lo.trim().parse::<u64>(),
        hi.trim().parse::<u64>(),
    ) {
        (Ok(c), Ok(l), Ok(h)) => l <= c && c <= h,
        _ => false,
    }
}

/// `a + b = 0` (`opposite`, the default: a receivable against a payable) or `a = b` (`same`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PairSign {
    Opposite,
    Same,
}

/// A config-driven balance rule between two orgs, e.g. side A's receivables less a clearing account against
/// side B's loan account.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PairRule {
    pub id: String,
    pub title: String,
    pub a: PairSide,
    pub b: PairSide,
    pub sign: PairSign,
}

/// A tree node as far as v0.11.0 reads it (validated, never executed).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TreeNode {
    pub id: String,
    /// `noul`, `choice` or `score`.
    pub node_type: String,
    pub instructions: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TreeConfig {
    pub root: String,
    pub nodes: Vec<TreeNode>,
}

/// The validated configuration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClassifyConfig {
    pub version: i64,
    pub jev: JevConfig,
    /// In file order.
    pub label_types: Vec<LabelType>,
    /// In file order (use [`ClassifyConfig::rules_by_priority`] to evaluate).
    pub rules: Vec<Rule>,
    pub queue: QueueConfig,
    pub groups: GroupsConfig,
    pub tree: Option<TreeConfig>,
}

impl ClassifyConfig {
    pub fn label_type(&self, id: &str) -> Option<&LabelType> {
        self.label_types.iter().find(|t| t.id == id)
    }
    pub fn rule(&self, id: &str) -> Option<&Rule> {
        self.rules.iter().find(|r| r.id == id)
    }
    /// Active rules, highest priority first (ties: file order).
    pub fn rules_by_priority(&self) -> Vec<&Rule> {
        let mut v: Vec<&Rule> = self.rules.iter().filter(|r| !r.archived).collect();
        v.sort_by_key(|r| std::cmp::Reverse(r.priority));
        v
    }
    /// The display name of a value (`harvest` → `Harvest Blend launch`); the value itself when unknown.
    pub fn choice_name<'a>(&'a self, label_type: &str, value: &'a str) -> &'a str {
        self.label_type(label_type)
            .and_then(|t| t.choice(value))
            .map(|c| c.name.as_str())
            .unwrap_or(value)
    }
}

/// What an item is, for usage counts and archive-instead-of-delete (V20).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ItemRef {
    Choice { label_type: String, id: String },
    LabelType(String),
    Rule(String),
}

/// One catalog or rule edit. Each produces a diff when staged.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigEdit {
    /// Add a choice (a project is a choice of `project`).
    AddChoice {
        label_type: String,
        choice: Choice,
    },
    /// Change a choice's name and/or aliases.
    EditChoice {
        label_type: String,
        id: String,
        name: Option<String>,
        aliases: Option<Vec<String>>,
    },
    ArchiveChoice {
        label_type: String,
        id: String,
    },
    /// Un-archive.
    RestoreChoice {
        label_type: String,
        id: String,
    },
    /// Delete: becomes `ArchiveChoice` when the choice is used (V20).
    RemoveChoice {
        label_type: String,
        id: String,
    },
    AddRule(Rule),
    /// Replace the rule with the same id.
    EditRule(Rule),
    ArchiveRule {
        id: String,
    },
    /// Delete: becomes `ArchiveRule` when the rule has fired on any line (V20).
    RemoveRule {
        id: String,
    },
    SetPriority {
        id: String,
        priority: i64,
    },
}

/// A staged change: what would be written, and its diff. Nothing is on disk until [`ConfigFile::write`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Staged {
    /// Hash of the file the edits were staged on (the write refuses if the disk differs).
    pub base_hash: String,
    pub new_text: String,
    pub new_hash: String,
    /// Unified diff, `--- classify.toml` / `+++ classify.toml`.
    pub diff: String,
    /// One line per edit, e.g. `added project "Harvest Blend launch"`.
    pub summary: Vec<String>,
    /// Removals turned into archives because the item is used (V20): `(item, uses)`.
    pub archived_instead: Vec<(ItemRef, usize)>,
}

/// A loaded config file: the validated config plus the exact text it came from.
#[derive(Debug, Clone)]
pub struct ConfigFile {
    pub path: PathBuf,
    pub text: String,
    /// SHA-256 hex of `text`.
    pub hash: String,
    pub config: ClassifyConfig,
}

/// The result of a hand-edit reload.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Reload {
    Unchanged,
    /// Valid: reloaded; number of changed lines.
    Reloaded {
        changes: usize,
    },
    /// Invalid: the last valid config stays live.
    Invalid(Vec<ConfigIssue>),
}

pub fn sha256_hex(text: &str) -> String {
    use sha2::{Digest, Sha256};
    Sha256::digest(text.as_bytes())
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

// ---------------------------------------------------------------------------------------------------------
// parsing and validation
// ---------------------------------------------------------------------------------------------------------

use toml_edit::{Item, TableLike, Value as TVal};

struct Ctx<'a> {
    file: &'a str,
    text: &'a str,
    issues: Vec<ConfigIssue>,
}

impl Ctx<'_> {
    fn line(&self, span: Option<std::ops::Range<usize>>) -> usize {
        span.map_or(0, |r| {
            self.text
                .get(..r.start.min(self.text.len()))
                .map_or(0, |s| s.matches('\n').count() + 1)
        })
    }
    fn issue(
        &mut self,
        span: Option<std::ops::Range<usize>>,
        path: &str,
        reason: impl Into<String>,
    ) {
        let line = self.line(span);
        self.issues.push(ConfigIssue {
            file: self.file.to_string(),
            line,
            path: path.to_string(),
            reason: reason.into(),
        });
    }
}

fn item_span(i: &Item) -> Option<std::ops::Range<usize>> {
    match i {
        Item::Value(v) => v.span(),
        Item::Table(t) => t.span(),
        Item::ArrayOfTables(a) => a.span(),
        Item::None => None,
    }
}

/// The span of a table-like's first value (array-of-tables entries carry no span of their own).
fn first_span(t: &dyn TableLike) -> Option<std::ops::Range<usize>> {
    t.iter().find_map(|(_, i)| item_span(i))
}

fn repr_text(v: &TVal) -> String {
    match v {
        TVal::String(s) => s.value().clone(),
        TVal::Integer(f) => f.display_repr().into_owned(),
        TVal::Float(f) => f.display_repr().into_owned(),
        TVal::Datetime(f) => f.display_repr().into_owned(),
        TVal::Boolean(b) => b.value().to_string(),
        _ => v.to_string().trim().to_string(),
    }
}

/// A string or an array of strings.
fn strings(ctx: &mut Ctx, i: &Item, path: &str) -> Vec<String> {
    match i {
        Item::Value(TVal::String(s)) => vec![s.value().clone()],
        Item::Value(TVal::Array(a)) => {
            let mut out = vec![];
            for (k, v) in a.iter().enumerate() {
                match v.as_str() {
                    Some(s) => out.push(s.to_string()),
                    None => ctx.issue(v.span(), &format!("{path}[{k}]"), "expected a string"),
                }
            }
            out
        }
        _ => {
            ctx.issue(
                item_span(i),
                path,
                "expected a string or an array of strings",
            );
            vec![]
        }
    }
}

fn confidence(ctx: &mut Ctx, i: &Item, path: &str) -> Option<Confidence> {
    let text = match i {
        Item::Value(v @ (TVal::Float(_) | TVal::Integer(_) | TVal::String(_))) => repr_text(v),
        _ => {
            ctx.issue(item_span(i), path, "expected a number between 0 and 1");
            return None;
        }
    };
    match text.parse::<Decimal>() {
        Ok(d) if d >= Decimal::ZERO && d <= Decimal::ONE => Some(d.normalize()),
        _ => {
            ctx.issue(
                item_span(i),
                path,
                format!("{text}: expected a number between 0 and 1"),
            );
            None
        }
    }
}

fn levenshtein(a: &str, b: &str) -> usize {
    let b: Vec<char> = b.chars().collect();
    let mut prev: Vec<usize> = (0..=b.len()).collect();
    for (i, ca) in a.chars().enumerate() {
        let mut cur = vec![i + 1];
        for (j, cb) in b.iter().enumerate() {
            let sub = prev[j] + usize::from(ca != *cb);
            cur.push(sub.min(prev[j + 1] + 1).min(cur[j] + 1));
        }
        prev = cur;
    }
    prev[b.len()]
}

/// `unknown choice (did you mean "fa"?)`.
pub fn did_you_mean<'a>(
    word: &str,
    candidates: impl IntoIterator<Item = &'a str>,
) -> Option<&'a str> {
    let w = word.to_lowercase();
    candidates
        .into_iter()
        .map(|c| (levenshtein(&w, &c.to_lowercase()), c))
        .filter(|(d, c)| {
            *d <= 2.max(c.len() / 3)
                || c.to_lowercase().starts_with(&w)
                || w.starts_with(&c.to_lowercase())
        })
        .min_by_key(|(d, _)| *d)
        .map(|(_, c)| c)
}

fn unknown(what: &str, word: &str, candidates: Vec<&str>) -> String {
    match did_you_mean(word, candidates) {
        Some(c) => format!("unknown {what} (did you mean \"{c}\"?)"),
        None => format!("unknown {what}"),
    }
}

fn valid_id(s: &str) -> bool {
    !s.is_empty()
        && s.chars()
            .all(|c| c.is_alphanumeric() || matches!(c, '-' | '_' | '.'))
}

const DEFAULT_REQUIRED: [&str; 3] = ["project", "bearer", "treatment"];
const DEFAULT_KINDS: [&str; 6] = [
    "intercompany_chain",
    "split",
    "trip",
    "accountant_question",
    "owner_paid",
    "settlement",
];
const DEFAULT_ROLES: [&str; 3] = ["origin", "mirror", "settlement"];

/// Parse and validate config text. `file` is the name used in issues.
pub fn parse(text: &str, file: &str) -> std::result::Result<ClassifyConfig, Vec<ConfigIssue>> {
    let mut ctx = Ctx {
        file,
        text,
        issues: vec![],
    };
    let doc = match toml_edit::Document::parse(text) {
        Ok(d) => d,
        Err(e) => {
            let reason = e.message().to_string();
            ctx.issue(e.span(), "(syntax)", reason);
            return Err(ctx.issues);
        }
    };
    let root = doc.as_table();
    let known = [
        "version",
        "jev",
        "label_types",
        "rules",
        "queue",
        "groups",
        "tree",
    ];
    for (k, i) in root.iter() {
        if !known.contains(&k) {
            let r = unknown("key", k, known.to_vec());
            ctx.issue(item_span(i), k, r);
        }
    }
    let version = match root.get("version") {
        None => 1,
        Some(i) => match i.as_integer() {
            Some(1) => 1,
            _ => {
                ctx.issue(item_span(i), "version", "expected 1");
                1
            }
        },
    };
    // [jev]
    let mut jev = JevConfig {
        enabled: false,
        fields: JEV_FIELDS.iter().map(|s| s.to_string()).collect(),
    };
    if let Some(i) = root.get("jev") {
        match i.as_table_like() {
            None => ctx.issue(item_span(i), "jev", "expected a table"),
            Some(t) => {
                if let Some(e) = t.get("enabled") {
                    match e.as_bool() {
                        Some(false) => {}
                        Some(true) => ctx.issue(
                            item_span(e),
                            "jev.enabled",
                            "jev_not_enabled: Jev is not available in this release; set enabled = false",
                        ),
                        None => ctx.issue(item_span(e), "jev.enabled", "expected true or false"),
                    }
                }
                if let Some(f) = t.get("fields") {
                    let fields = strings(&mut ctx, f, "jev.fields");
                    for x in &fields {
                        if !JEV_FIELDS.contains(&x.as_str()) {
                            ctx.issue(
                                item_span(f),
                                "jev.fields",
                                format!("\"{x}\" is not allowed: only description, payee, amount, date may be sent"),
                            );
                        }
                    }
                    jev.fields = fields;
                }
            }
        }
    }
    // [label_types.*]
    let mut label_types: Vec<LabelType> = vec![];
    if let Some(i) = root.get("label_types") {
        match i.as_table_like() {
            None => ctx.issue(item_span(i), "label_types", "expected a table"),
            Some(t) => {
                for (id, ti) in t.iter() {
                    let path = format!("label_types.{id}");
                    let Some(tt) = ti.as_table_like() else {
                        ctx.issue(item_span(ti), &path, "expected a table");
                        continue;
                    };
                    if !valid_id(id) {
                        ctx.issue(first_span(tt), &path, "ids use letters, digits, - _ . only");
                    }
                    label_types.push(label_type(&mut ctx, id, tt, &path));
                }
            }
        }
    }
    // [groups]
    let mut groups = GroupsConfig {
        kinds: DEFAULT_KINDS.iter().map(|s| s.to_string()).collect(),
        roles: DEFAULT_ROLES.iter().map(|s| s.to_string()).collect(),
        pair_rules: vec![],
        settle_window_days: 7,
    };
    if let Some(i) = root.get("groups") {
        match i.as_table_like() {
            None => ctx.issue(item_span(i), "groups", "expected a table"),
            Some(t) => {
                if let Some(k) = t.get("kinds") {
                    groups.kinds = strings(&mut ctx, k, "groups.kinds");
                }
                if let Some(r) = t.get("roles") {
                    groups.roles = strings(&mut ctx, r, "groups.roles");
                }
                if let Some(w) = t.get("settle_window_days") {
                    match w.as_integer() {
                        Some(n) if (0..=31).contains(&n) => groups.settle_window_days = n,
                        _ => ctx.issue(
                            item_span(w),
                            "groups.settle_window_days",
                            "expected a number of days, 0 to 31",
                        ),
                    }
                }
                if let Some(pr) = t.get("pair_rules") {
                    groups.pair_rules = pair_rules(&mut ctx, pr);
                }
            }
        }
    }
    // [queue]
    let mut queue = QueueConfig {
        required: DEFAULT_REQUIRED
            .iter()
            .filter(|r| label_types.iter().any(|t| t.id == **r))
            .map(|s| s.to_string())
            .collect(),
        threshold: Decimal::new(9, 1),
        scope: None,
    };
    if let Some(i) = root.get("queue") {
        match i.as_table_like() {
            None => ctx.issue(item_span(i), "queue", "expected a table"),
            Some(t) => {
                if let Some(r) = t.get("required") {
                    let req = strings(&mut ctx, r, "queue.required");
                    for x in &req {
                        if !label_types.iter().any(|t| &t.id == x) {
                            let m = unknown(
                                "label type",
                                x,
                                label_types.iter().map(|t| t.id.as_str()).collect(),
                            );
                            ctx.issue(item_span(r), "queue.required", format!("\"{x}\": {m}"));
                        }
                    }
                    queue.required = req;
                }
                if let Some(sc) = t.get("scope") {
                    match sc.as_str() {
                        Some(s) => match crate::classify::view::Filter::parse(s) {
                            Ok(_) => {
                                queue.scope = (!s.trim().is_empty()).then(|| s.trim().to_string())
                            }
                            Err(e) => ctx.issue(item_span(sc), "queue.scope", e.message),
                        },
                        None => ctx.issue(
                            item_span(sc),
                            "queue.scope",
                            "expected a filter string, e.g. \"year:2025\"",
                        ),
                    }
                }
                if let Some(th) = t.get("threshold")
                    && let Some(c) = confidence(&mut ctx, th, "queue.threshold")
                {
                    queue.threshold = c;
                }
            }
        }
    }
    // [[rules]]
    let mut rules: Vec<Rule> = vec![];
    if let Some(i) = root.get("rules") {
        match i.as_array_of_tables() {
            None => ctx.issue(item_span(i), "rules", "expected [[rules]] tables"),
            Some(a) => {
                for (k, t) in a.iter().enumerate() {
                    let path = format!("rules[{k}]");
                    if let Some(r) = rule(&mut ctx, t, &path, &label_types, &groups) {
                        if let Some(dup) = rules.iter().find(|x| x.id == r.id) {
                            let id = dup.id.clone();
                            ctx.issue(
                                first_span(t),
                                &format!("{path}.id"),
                                format!("duplicate rule id \"{id}\""),
                            );
                        }
                        rules.push(r);
                    }
                }
            }
        }
    }
    // [tree] (validated, not executed)
    let tree = root.get("tree").and_then(|i| tree(&mut ctx, i));
    if ctx.issues.is_empty() {
        Ok(ClassifyConfig {
            version,
            jev,
            label_types,
            rules,
            queue,
            groups,
            tree,
        })
    } else {
        Err(ctx.issues)
    }
}

fn label_type(ctx: &mut Ctx, id: &str, t: &dyn TableLike, path: &str) -> LabelType {
    let title = t
        .get("title")
        .and_then(Item::as_str)
        .unwrap_or(id)
        .to_string();
    let kind = match t.get("kind") {
        None => LabelKind::Choice,
        Some(k) => match k.as_str() {
            Some("choice") => LabelKind::Choice,
            Some("account") => LabelKind::Account,
            _ => {
                ctx.issue(
                    item_span(k),
                    &format!("{path}.kind"),
                    "expected \"choice\" or \"account\"",
                );
                LabelKind::Choice
            }
        },
    };
    for (k, i) in t.iter() {
        if !["title", "kind", "choices", "archived"].contains(&k) {
            let r = unknown("key", k, vec!["title", "kind", "choices", "archived"]);
            ctx.issue(item_span(i), &format!("{path}.{k}"), r);
        }
    }
    let archived = t.get("archived").and_then(Item::as_bool).unwrap_or(false);
    let mut choices: Vec<Choice> = vec![];
    if let Some(ci) = t.get("choices") {
        if kind == LabelKind::Account {
            ctx.issue(
                item_span(ci),
                &format!("{path}.choices"),
                "an account label type takes its choices from the line's org chart",
            );
        }
        match ci.as_array() {
            None => ctx.issue(
                item_span(ci),
                &format!("{path}.choices"),
                "expected an array",
            ),
            Some(a) => {
                for (k, v) in a.iter().enumerate() {
                    let cpath = format!("{path}.choices[{k}]");
                    let Some(it) = v.as_inline_table() else {
                        ctx.issue(v.span(), &cpath, "expected { id = …, name = … }");
                        continue;
                    };
                    let id = it
                        .get("id")
                        .and_then(TVal::as_str)
                        .unwrap_or("")
                        .to_string();
                    if !valid_id(&id) {
                        ctx.issue(
                            v.span(),
                            &format!("{cpath}.id"),
                            "missing or invalid id (letters, digits, - _ .)",
                        );
                        continue;
                    }
                    if choices.iter().any(|c| c.id == id) {
                        ctx.issue(
                            v.span(),
                            &format!("{cpath}.id"),
                            format!("duplicate choice \"{id}\""),
                        );
                    }
                    for (key, val) in it.iter() {
                        if !["id", "name", "aliases", "archived"].contains(&key) {
                            let r = unknown("key", key, vec!["id", "name", "aliases", "archived"]);
                            ctx.issue(val.span(), &format!("{cpath}.{key}"), r);
                        }
                    }
                    let name = it
                        .get("name")
                        .and_then(TVal::as_str)
                        .unwrap_or(&id)
                        .to_string();
                    let aliases = match it.get("aliases") {
                        None => vec![],
                        Some(TVal::Array(a)) => a
                            .iter()
                            .filter_map(|x| x.as_str().map(str::to_string))
                            .collect(),
                        Some(x) => {
                            ctx.issue(
                                x.span(),
                                &format!("{cpath}.aliases"),
                                "expected an array of strings",
                            );
                            vec![]
                        }
                    };
                    let archived = it.get("archived").and_then(TVal::as_bool).unwrap_or(false);
                    choices.push(Choice {
                        id,
                        name,
                        aliases,
                        archived,
                    });
                }
            }
        }
    }
    LabelType {
        id: id.to_string(),
        title,
        kind,
        choices,
        archived,
    }
}

const COND_KEYS: [&str; 11] = [
    "payee_is_org",
    "bank_transfer",
    "description_has_alias",
    "payee",
    "payee_has",
    "description_has",
    "account_in",
    "org_in",
    "amount_between",
    "date_between",
    "sign",
];

fn pair(ctx: &mut Ctx, i: &Item, path: &str) -> Option<(String, String)> {
    let a = i.as_array().filter(|a| a.len() == 2);
    match a {
        Some(a) => Some((repr_text(a.get(0)?), repr_text(a.get(1)?))),
        None => {
            ctx.issue(item_span(i), path, "expected [low, high]");
            None
        }
    }
}

fn rule(
    ctx: &mut Ctx,
    t: &toml_edit::Table,
    path: &str,
    types: &[LabelType],
    groups: &GroupsConfig,
) -> Option<Rule> {
    let span = first_span(t);
    for (k, i) in t.iter() {
        let keys = ["id", "priority", "when", "set", "confidence", "archived"];
        if !keys.contains(&k) {
            let r = unknown("key", k, keys.to_vec());
            ctx.issue(item_span(i), &format!("{path}.{k}"), r);
        }
    }
    let Some(id) = t.get("id").and_then(Item::as_str).filter(|s| valid_id(s)) else {
        ctx.issue(span.clone(), &format!("{path}.id"), "missing or invalid id");
        return None;
    };
    let priority = match t.get("priority") {
        None => 0,
        Some(p) => p.as_integer().unwrap_or_else(|| {
            ctx.issue(
                item_span(p),
                &format!("{path}.priority"),
                "expected an integer",
            );
            0
        }),
    };
    let conf = match t.get("confidence") {
        None => Some(Decimal::ONE),
        Some(c) => confidence(ctx, c, &format!("{path}.confidence")),
    };
    let archived = t.get("archived").and_then(Item::as_bool).unwrap_or(false);
    let mut when = vec![];
    let mut alias_type: Option<String> = None;
    match t.get("when").and_then(Item::as_table_like) {
        None => ctx.issue(
            span.clone(),
            &format!("{path}.when"),
            "missing: a rule needs at least one condition",
        ),
        Some(w) => {
            if w.is_empty() {
                ctx.issue(
                    span.clone(),
                    &format!("{path}.when"),
                    "empty: a rule needs at least one condition",
                );
            }
            for (k, i) in w.iter() {
                let cpath = format!("{path}.when.{k}");
                let c = match k {
                    "description_has_alias" => {
                        let ty = i.as_str().unwrap_or("").to_string();
                        match types.iter().find(|t| t.id == ty) {
                            Some(t) if t.kind == LabelKind::Choice => {
                                alias_type = Some(ty.clone());
                                Some(Cond::DescriptionHasAlias(ty))
                            }
                            Some(_) => {
                                ctx.issue(
                                    item_span(i),
                                    &cpath,
                                    "an account label type has no aliases",
                                );
                                None
                            }
                            None => {
                                let m = unknown(
                                    "label type",
                                    &ty,
                                    types.iter().map(|t| t.id.as_str()).collect(),
                                );
                                ctx.issue(item_span(i), &cpath, format!("\"{ty}\": {m}"));
                                None
                            }
                        }
                    }
                    "payee" => match i.as_str() {
                        Some(s) => Some(Cond::Payee(s.to_string())),
                        None => {
                            ctx.issue(item_span(i), &cpath, "expected a string");
                            None
                        }
                    },
                    "payee_has" => Some(Cond::PayeeHas(strings(ctx, i, &cpath))),
                    "description_has" => Some(Cond::DescriptionHas(strings(ctx, i, &cpath))),
                    "account_in" => Some(Cond::AccountIn(strings(ctx, i, &cpath))),
                    "org_in" => Some(Cond::OrgIn(strings(ctx, i, &cpath))),
                    "amount_between" => pair(ctx, i, &cpath).and_then(|(a, b)| {
                        match (crate::money::parse(&a), crate::money::parse(&b)) {
                            (Ok(lo), Ok(hi)) if lo <= hi => Some(Cond::AmountBetween(lo, hi)),
                            _ => {
                                ctx.issue(
                                    item_span(i),
                                    &cpath,
                                    "expected [low, high] amounts, low <= high",
                                );
                                None
                            }
                        }
                    }),
                    "date_between" => pair(ctx, i, &cpath).and_then(|(a, b)| {
                        let ok = crate::dates::parse_iso(&a).is_some()
                            && crate::dates::parse_iso(&b).is_some()
                            && a <= b;
                        if ok {
                            Some(Cond::DateBetween(a, b))
                        } else {
                            ctx.issue(
                                item_span(i),
                                &cpath,
                                "expected [\"YYYY-MM-DD\", \"YYYY-MM-DD\"], from <= to",
                            );
                            None
                        }
                    }),
                    "sign" => match i.as_str() {
                        Some("in") => Some(Cond::Sign(Sign::In)),
                        Some("out") => Some(Cond::Sign(Sign::Out)),
                        _ => {
                            ctx.issue(item_span(i), &cpath, "expected \"in\" or \"out\"");
                            None
                        }
                    },
                    "payee_is_org" => match i.as_str() {
                        Some(s) if !s.is_empty() => Some(Cond::PayeeIsOrg(s.to_string())),
                        _ => {
                            ctx.issue(item_span(i), &cpath, "expected an org alias or \"any\"");
                            None
                        }
                    },
                    "bank_transfer" => match i.as_bool() {
                        Some(b) => Some(Cond::BankTransfer(b)),
                        None => {
                            ctx.issue(item_span(i), &cpath, "expected true or false");
                            None
                        }
                    },
                    other => {
                        let m = unknown("condition", other, COND_KEYS.to_vec());
                        ctx.issue(item_span(i), &cpath, m);
                        None
                    }
                };
                when.extend(c);
            }
        }
    }
    let mut set = vec![];
    let mut group_kind = None;
    match t.get("set").and_then(Item::as_table_like) {
        None => ctx.issue(
            span.clone(),
            &format!("{path}.set"),
            "missing: a rule needs something to set",
        ),
        Some(s) => {
            if s.is_empty() {
                ctx.issue(
                    span.clone(),
                    &format!("{path}.set"),
                    "empty: a rule needs something to set",
                );
            }
            for (k, i) in s.iter() {
                let spath = format!("{path}.set.{k}");
                let Some(v) = i.as_str() else {
                    ctx.issue(item_span(i), &spath, "expected a string");
                    continue;
                };
                if k == "group_kind" {
                    if groups.kinds.iter().any(|g| g == v) {
                        group_kind = Some(v.to_string());
                    } else {
                        let m = unknown(
                            "group kind",
                            v,
                            groups.kinds.iter().map(String::as_str).collect(),
                        );
                        ctx.issue(item_span(i), &spath, format!("= \"{v}\": {m}"));
                    }
                    continue;
                }
                let Some(ty) = types.iter().find(|t| t.id == k) else {
                    let mut names: Vec<&str> = types.iter().map(|t| t.id.as_str()).collect();
                    names.push("group_kind");
                    let m = unknown("label type", k, names);
                    ctx.issue(item_span(i), &spath, m);
                    continue;
                };
                if v == "$matched" {
                    if alias_type.as_deref() == Some(k) {
                        set.push((k.to_string(), SetValue::Matched));
                    } else {
                        ctx.issue(
                            item_span(i),
                            &spath,
                            format!("$matched needs when.description_has_alias = \"{k}\""),
                        );
                    }
                    continue;
                }
                if ty.kind == LabelKind::Choice {
                    match ty.choice(v) {
                        None => {
                            let m = unknown(
                                "choice",
                                v,
                                ty.choices.iter().map(|c| c.id.as_str()).collect(),
                            );
                            ctx.issue(item_span(i), &spath, format!("= \"{v}\": {m}"));
                            continue;
                        }
                        Some(c) if c.archived && !archived => {
                            ctx.issue(
                                item_span(i),
                                &spath,
                                format!("= \"{v}\": the choice is archived"),
                            );
                            continue;
                        }
                        _ => {}
                    }
                }
                set.push((k.to_string(), SetValue::Choice(v.to_string())));
            }
        }
    }
    Some(Rule {
        id: id.to_string(),
        priority,
        when,
        set,
        group_kind,
        confidence: conf.unwrap_or(Decimal::ONE),
        archived,
    })
}

fn pair_side(
    ctx: &mut Ctx,
    i: Option<&Item>,
    path: &str,
    span: Option<std::ops::Range<usize>>,
) -> Option<PairSide> {
    let Some(t) = i.and_then(Item::as_table_like) else {
        ctx.issue(
            span,
            path,
            "expected { org = \"…\", plus = [\"…\"], minus = [\"…\"] }",
        );
        return None;
    };
    let org = t
        .get("org")
        .and_then(Item::as_str)
        .unwrap_or("")
        .to_string();
    if org.is_empty() {
        ctx.issue(first_span(t), &format!("{path}.org"), "missing");
    }
    let plus = t
        .get("plus")
        .map(|p| strings(ctx, p, &format!("{path}.plus")))
        .unwrap_or_default();
    let minus = t
        .get("minus")
        .map(|p| strings(ctx, p, &format!("{path}.minus")))
        .unwrap_or_default();
    if plus.is_empty() {
        ctx.issue(
            first_span(t),
            &format!("{path}.plus"),
            "needs at least one account code or range",
        );
    }
    Some(PairSide { org, plus, minus })
}

fn pair_rules(ctx: &mut Ctx, i: &Item) -> Vec<PairRule> {
    let mut out = vec![];
    let Some(t) = i.as_table_like() else {
        ctx.issue(
            item_span(i),
            "groups.pair_rules",
            "expected a table of rules",
        );
        return out;
    };
    for (id, ri) in t.iter() {
        let path = format!("groups.pair_rules.{id}");
        let Some(r) = ri.as_table_like() else {
            ctx.issue(item_span(ri), &path, "expected a table");
            continue;
        };
        let span = first_span(r);
        let title = r
            .get("title")
            .and_then(Item::as_str)
            .unwrap_or(id)
            .to_string();
        let sign = match r.get("sign").and_then(Item::as_str) {
            None | Some("opposite") => PairSign::Opposite,
            Some("same") => PairSign::Same,
            Some(_) => {
                ctx.issue(
                    span.clone(),
                    &format!("{path}.sign"),
                    "expected \"opposite\" or \"same\"",
                );
                PairSign::Opposite
            }
        };
        let a = pair_side(ctx, r.get("a"), &format!("{path}.a"), span.clone());
        let b = pair_side(ctx, r.get("b"), &format!("{path}.b"), span);
        if let (Some(a), Some(b)) = (a, b) {
            out.push(PairRule {
                id: id.to_string(),
                title,
                a,
                b,
                sign,
            });
        }
    }
    out
}

fn tree(ctx: &mut Ctx, i: &Item) -> Option<TreeConfig> {
    let Some(t) = i.as_table_like() else {
        ctx.issue(item_span(i), "tree", "expected a table");
        return None;
    };
    let root = t
        .get("root")
        .and_then(Item::as_str)
        .unwrap_or("")
        .to_string();
    let mut nodes = vec![];
    if let Some(n) = t.get("nodes").and_then(Item::as_table_like) {
        for (id, ni) in n.iter() {
            let path = format!("tree.nodes.{id}");
            let Some(nt) = ni.as_table_like() else {
                ctx.issue(item_span(ni), &path, "expected a table");
                continue;
            };
            let node_type = nt
                .get("type")
                .and_then(Item::as_str)
                .unwrap_or("")
                .to_string();
            if !["noul", "choice", "score"].contains(&node_type.as_str()) {
                ctx.issue(
                    first_span(nt),
                    &format!("{path}.type"),
                    "expected \"noul\", \"choice\" or \"score\"",
                );
            }
            let instructions = nt
                .get("instructions")
                .and_then(Item::as_str)
                .unwrap_or("")
                .to_string();
            if instructions.is_empty() {
                ctx.issue(first_span(nt), &format!("{path}.instructions"), "missing");
            }
            nodes.push(TreeNode {
                id: id.to_string(),
                node_type,
                instructions,
            });
        }
    }
    if !root.is_empty() && !nodes.iter().any(|n| n.id == root) {
        let m = unknown("node", &root, nodes.iter().map(|n| n.id.as_str()).collect());
        ctx.issue(
            t.get("root").and_then(item_span),
            "tree.root",
            format!("\"{root}\": {m}"),
        );
    }
    Some(TreeConfig { root, nodes })
}

/// Checks that need the configured orgs (`ownpurse.json` aliases): every pair rule side names one. Issues
/// carry the file and line of the `org` value.
pub fn check_orgs(text: &str, file: &str, aliases: &[String]) -> Vec<ConfigIssue> {
    let mut ctx = Ctx {
        file,
        text,
        issues: vec![],
    };
    let Ok(doc) = toml_edit::Document::parse(text) else {
        return vec![];
    };
    let rules = doc
        .as_table()
        .get("groups")
        .and_then(|g| g.get("pair_rules"))
        .and_then(Item::as_table_like);
    if let Some(rs) = doc
        .as_table()
        .get("rules")
        .and_then(Item::as_array_of_tables)
    {
        for (k, t) in rs.iter().enumerate() {
            let Some(o) = t.get("when").and_then(|w| w.get("payee_is_org")) else {
                continue;
            };
            let org = o.as_str().unwrap_or("");
            if org != "any" && !org.is_empty() && !aliases.iter().any(|a| a == org) {
                let m = unknown("org", org, aliases.iter().map(String::as_str).collect());
                ctx.issue(
                    item_span(o),
                    &format!("rules[{k}].when.payee_is_org"),
                    format!("= \"{org}\": {m} (an alias in ownpurse.json, or \"any\")"),
                );
            }
        }
    }
    for (id, r) in rules.into_iter().flat_map(|t| t.iter()) {
        for side in ["a", "b"] {
            let Some(o) = r.get(side).and_then(|s| s.get("org")) else {
                continue;
            };
            let org = o.as_str().unwrap_or("");
            if !org.is_empty() && !aliases.iter().any(|a| a == org) {
                let m = unknown("org", org, aliases.iter().map(String::as_str).collect());
                ctx.issue(
                    item_span(o),
                    &format!("groups.pair_rules.{id}.{side}.org"),
                    format!("= \"{org}\": {m} (the orgs are the aliases in ownpurse.json)"),
                );
            }
        }
    }
    ctx.issues
}

// ---------------------------------------------------------------------------------------------------------
// diff
// ---------------------------------------------------------------------------------------------------------

/// A unified diff (3 lines of context) between two texts.
pub fn unified_diff(a: &str, b: &str, name: &str) -> String {
    let x: Vec<&str> = a.lines().collect();
    let y: Vec<&str> = b.lines().collect();
    // LCS table (config files are small)
    let (n, m) = (x.len(), y.len());
    let mut l = vec![vec![0u32; m + 1]; n + 1];
    for i in (0..n).rev() {
        for j in (0..m).rev() {
            l[i][j] = if x[i] == y[j] {
                l[i + 1][j + 1] + 1
            } else {
                l[i + 1][j].max(l[i][j + 1])
            };
        }
    }
    // ops: (' ' | '-' | '+', line, a index, b index)
    let mut ops: Vec<(char, &str)> = vec![];
    let (mut i, mut j) = (0, 0);
    while i < n || j < m {
        if i < n && j < m && x[i] == y[j] {
            ops.push((' ', x[i]));
            i += 1;
            j += 1;
        } else if i < n && (j == m || l[i + 1][j] >= l[i][j + 1]) {
            ops.push(('-', x[i]));
            i += 1;
        } else {
            ops.push(('+', y[j]));
            j += 1;
        }
    }
    if ops.iter().all(|o| o.0 == ' ') {
        return String::new();
    }
    let mut out = format!("--- {name}\n+++ {name}\n");
    let ctxn = 3;
    let changed: Vec<usize> = ops
        .iter()
        .enumerate()
        .filter(|(_, o)| o.0 != ' ')
        .map(|(k, _)| k)
        .collect();
    let mut k = 0;
    while k < changed.len() {
        let start = changed[k].saturating_sub(ctxn);
        let mut end = changed[k];
        while k < changed.len() && changed[k] <= end + 2 * ctxn {
            end = changed[k];
            k += 1;
        }
        let end = (end + ctxn + 1).min(ops.len());
        // line numbers
        let a0 = ops[..start].iter().filter(|o| o.0 != '+').count();
        let b0 = ops[..start].iter().filter(|o| o.0 != '-').count();
        let alen = ops[start..end].iter().filter(|o| o.0 != '+').count();
        let blen = ops[start..end].iter().filter(|o| o.0 != '-').count();
        out.push_str(&format!("@@ -{},{alen} +{},{blen} @@\n", a0 + 1, b0 + 1));
        for (c, line) in &ops[start..end] {
            out.push(*c);
            out.push_str(line);
            out.push('\n');
        }
    }
    out
}

// ---------------------------------------------------------------------------------------------------------
// edits
// ---------------------------------------------------------------------------------------------------------

fn cfg_err(msg: impl Into<String>) -> ClassifyError {
    ClassifyError::new("config", msg)
}

fn str_array(items: &[String]) -> toml_edit::Array {
    let mut a = toml_edit::Array::new();
    for s in items {
        a.push(s.as_str());
    }
    a
}

fn num_value(d: &Decimal) -> TVal {
    let text = d.normalize().to_string();
    text.parse::<TVal>()
        .unwrap_or_else(|_| TVal::from(text.clone()))
}

fn choice_value(c: &Choice) -> TVal {
    let mut t = toml_edit::InlineTable::new();
    t.insert("id", c.id.as_str().into());
    t.insert("name", c.name.as_str().into());
    if !c.aliases.is_empty() {
        t.insert("aliases", TVal::Array(str_array(&c.aliases)));
    }
    if c.archived {
        t.insert("archived", true.into());
    }
    TVal::InlineTable(t)
}

/// A rule as a `[[rules]]` table.
pub fn rule_table(r: &Rule) -> toml_edit::Table {
    let mut t = toml_edit::Table::new();
    t.insert("id", toml_edit::value(r.id.as_str()));
    t.insert("priority", toml_edit::value(r.priority));
    let mut when = toml_edit::InlineTable::new();
    for c in &r.when {
        let v: TVal = match c {
            Cond::DescriptionHasAlias(s) | Cond::Payee(s) => s.as_str().into(),
            Cond::PayeeHas(v) | Cond::DescriptionHas(v) | Cond::AccountIn(v) | Cond::OrgIn(v) => {
                TVal::Array(str_array(v))
            }
            Cond::AmountBetween(lo, hi) => {
                let mut a = toml_edit::Array::new();
                a.push(num_value(lo));
                a.push(num_value(hi));
                TVal::Array(a)
            }
            Cond::DateBetween(a, b) => TVal::Array(str_array(&[a.clone(), b.clone()])),
            Cond::Sign(Sign::In) => "in".into(),
            Cond::Sign(Sign::Out) => "out".into(),
            Cond::PayeeIsOrg(s) => s.as_str().into(),
            Cond::BankTransfer(b) => (*b).into(),
        };
        when.insert(c.key(), v);
    }
    t.insert("when", toml_edit::value(when));
    let mut set = toml_edit::InlineTable::new();
    for (k, v) in &r.set {
        let s = match v {
            SetValue::Choice(c) => c.as_str(),
            SetValue::Matched => "$matched",
        };
        set.insert(k, s.into());
    }
    if let Some(g) = &r.group_kind {
        set.insert("group_kind", g.as_str().into());
    }
    t.insert("set", toml_edit::value(set));
    if r.confidence != Decimal::ONE {
        t.insert(
            "confidence",
            toml_edit::Item::Value(num_value(&r.confidence)),
        );
    }
    if r.archived {
        t.insert("archived", toml_edit::value(true));
    }
    t
}

fn choices_mut<'d>(
    doc: &'d mut toml_edit::DocumentMut,
    ty: &str,
) -> Result<&'d mut toml_edit::Array> {
    let lt = doc
        .get_mut("label_types")
        .and_then(|i| i.get_mut(ty))
        .ok_or_else(|| cfg_err(format!("no label type \"{ty}\"")))?;
    let tl = lt
        .as_table_like_mut()
        .ok_or_else(|| cfg_err(format!("label_types.{ty} is not a table")))?;
    if !tl.contains_key("choices") {
        tl.insert("choices", toml_edit::value(toml_edit::Array::new()));
    }
    tl.get_mut("choices")
        .and_then(Item::as_array_mut)
        .ok_or_else(|| cfg_err(format!("label_types.{ty}.choices is not an array")))
}

fn choice_index(a: &toml_edit::Array, id: &str) -> Option<usize> {
    a.iter().position(|v| {
        v.as_inline_table()
            .and_then(|t| t.get("id"))
            .and_then(TVal::as_str)
            == Some(id)
    })
}

fn rules_mut(doc: &mut toml_edit::DocumentMut) -> Result<&mut toml_edit::ArrayOfTables> {
    if !doc.contains_key("rules") {
        doc.insert(
            "rules",
            Item::ArrayOfTables(toml_edit::ArrayOfTables::new()),
        );
    }
    doc.get_mut("rules")
        .and_then(Item::as_array_of_tables_mut)
        .ok_or_else(|| cfg_err("rules is not [[rules]]"))
}

fn rule_index(a: &toml_edit::ArrayOfTables, id: &str) -> Option<usize> {
    a.iter()
        .position(|t| t.get("id").and_then(Item::as_str) == Some(id))
}

fn set_choice_flag(
    doc: &mut toml_edit::DocumentMut,
    ty: &str,
    id: &str,
    archived: bool,
) -> Result<()> {
    let a = choices_mut(doc, ty)?;
    let k = choice_index(a, id).ok_or_else(|| cfg_err(format!("no choice \"{id}\" in {ty}")))?;
    let t = a
        .get_mut(k)
        .and_then(TVal::as_inline_table_mut)
        .ok_or_else(|| cfg_err("choice is not an inline table"))?;
    if archived {
        t.insert("archived", true.into());
    } else {
        t.remove("archived");
    }
    Ok(())
}

fn set_rule_flag(doc: &mut toml_edit::DocumentMut, id: &str) -> Result<()> {
    let a = rules_mut(doc)?;
    let k = rule_index(a, id).ok_or_else(|| cfg_err(format!("no rule \"{id}\"")))?;
    if let Some(t) = a.get_mut(k) {
        t.insert("archived", toml_edit::value(true));
    }
    Ok(())
}

/// Push onto an array keeping a one-element-per-line layout when the array already has one.
fn push_layout(a: &mut toml_edit::Array, mut v: TVal) {
    let multiline = a.iter().any(|x| {
        x.decor()
            .prefix()
            .and_then(|p| p.as_str())
            .is_some_and(|p| p.contains('\n'))
    }) || a.trailing().as_str().is_some_and(|t| t.contains('\n'));
    if multiline {
        let indent = a
            .iter()
            .last()
            .and_then(|x| {
                x.decor()
                    .prefix()
                    .and_then(|p| p.as_str())
                    .map(str::to_string)
            })
            .unwrap_or_else(|| "\n  ".into());
        v.decor_mut().set_prefix(indent);
        v.decor_mut().set_suffix("");
        a.push_formatted(v);
        a.set_trailing_comma(true);
    } else {
        a.push(v);
    }
}

fn apply_edit(
    doc: &mut toml_edit::DocumentMut,
    cfg: &ClassifyConfig,
    e: &ConfigEdit,
    used: &dyn Fn(&ItemRef) -> usize,
    archived_instead: &mut Vec<(ItemRef, usize)>,
) -> Result<String> {
    let title = |ty: &str| {
        cfg.label_type(ty)
            .map(|t| t.title.to_lowercase())
            .unwrap_or_else(|| ty.to_string())
    };
    Ok(match e {
        ConfigEdit::AddChoice { label_type, choice } => {
            if !valid_id(&choice.id) {
                return Err(cfg_err(format!("invalid id \"{}\"", choice.id)));
            }
            let a = choices_mut(doc, label_type)?;
            if choice_index(a, &choice.id).is_some() {
                return Err(cfg_err(format!(
                    "{label_type} already has a choice \"{}\"",
                    choice.id
                )));
            }
            push_layout(a, choice_value(choice));
            format!("added {} \"{}\"", title(label_type), choice.name)
        }
        ConfigEdit::EditChoice {
            label_type,
            id,
            name,
            aliases,
        } => {
            let a = choices_mut(doc, label_type)?;
            let k = choice_index(a, id)
                .ok_or_else(|| cfg_err(format!("no choice \"{id}\" in {label_type}")))?;
            let t = a
                .get_mut(k)
                .and_then(TVal::as_inline_table_mut)
                .ok_or_else(|| cfg_err("choice is not an inline table"))?;
            if let Some(n) = name {
                t.insert("name", n.as_str().into());
            }
            if let Some(al) = aliases {
                if al.is_empty() {
                    t.remove("aliases");
                } else {
                    t.insert("aliases", TVal::Array(str_array(al)));
                }
            }
            format!("edited {} \"{id}\"", title(label_type))
        }
        ConfigEdit::ArchiveChoice { label_type, id } => {
            set_choice_flag(doc, label_type, id, true)?;
            format!("archived {} \"{id}\"", title(label_type))
        }
        ConfigEdit::RestoreChoice { label_type, id } => {
            set_choice_flag(doc, label_type, id, false)?;
            format!("restored {} \"{id}\"", title(label_type))
        }
        ConfigEdit::RemoveChoice { label_type, id } => {
            let item = ItemRef::Choice {
                label_type: label_type.clone(),
                id: id.clone(),
            };
            let n = used(&item);
            let in_rules = cfg.rules.iter().any(|r| {
                r.set
                    .iter()
                    .any(|(k, v)| k == label_type && *v == SetValue::Choice(id.clone()))
            });
            if n > 0 || in_rules {
                set_choice_flag(doc, label_type, id, true)?;
                archived_instead.push((item, n));
                format!(
                    "archived {} \"{id}\" (used by {n} {}, so not deleted)",
                    title(label_type),
                    plural(n, "line")
                )
            } else {
                let a = choices_mut(doc, label_type)?;
                let k = choice_index(a, id)
                    .ok_or_else(|| cfg_err(format!("no choice \"{id}\" in {label_type}")))?;
                a.remove(k);
                format!("removed {} \"{id}\"", title(label_type))
            }
        }
        ConfigEdit::AddRule(r) => {
            let a = rules_mut(doc)?;
            if rule_index(a, &r.id).is_some() {
                return Err(cfg_err(format!("a rule \"{}\" exists", r.id)));
            }
            a.push(rule_table(r));
            format!("added rule \"{}\"", r.id)
        }
        ConfigEdit::EditRule(r) => {
            let a = rules_mut(doc)?;
            let k = rule_index(a, &r.id).ok_or_else(|| cfg_err(format!("no rule \"{}\"", r.id)))?;
            let mut t = rule_table(r);
            if let Some(old) = a.get(k) {
                *t.decor_mut() = old.decor().clone();
            }
            a.replace(k, t);
            format!("edited rule \"{}\"", r.id)
        }
        ConfigEdit::ArchiveRule { id } => {
            set_rule_flag(doc, id)?;
            format!("archived rule \"{id}\"")
        }
        ConfigEdit::RemoveRule { id } => {
            let item = ItemRef::Rule(id.clone());
            let n = used(&item);
            if n > 0 {
                set_rule_flag(doc, id)?;
                archived_instead.push((item, n));
                format!(
                    "archived rule \"{id}\" (used by {n} {}, so not deleted)",
                    plural(n, "line")
                )
            } else {
                let a = rules_mut(doc)?;
                let k = rule_index(a, id).ok_or_else(|| cfg_err(format!("no rule \"{id}\"")))?;
                a.remove(k);
                format!("removed rule \"{id}\"")
            }
        }
        ConfigEdit::SetPriority { id, priority } => {
            let a = rules_mut(doc)?;
            let k = rule_index(a, id).ok_or_else(|| cfg_err(format!("no rule \"{id}\"")))?;
            if let Some(t) = a.get_mut(k) {
                t.insert("priority", toml_edit::value(*priority));
            }
            format!("set rule \"{id}\" priority {priority}")
        }
    })
}

pub fn plural(n: usize, word: &str) -> String {
    if n == 1 {
        word.to_string()
    } else {
        format!("{word}s")
    }
}

fn file_name(path: &Path) -> String {
    path.file_name()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| "classify.toml".into())
}

/// `classify.toml.~N~` backups beside `path`, `(N, path)` ascending.
pub fn backups(path: &Path) -> Vec<(u64, PathBuf)> {
    let dir = path
        .parent()
        .filter(|d| !d.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let base = file_name(path);
    let mut out = vec![];
    if let Ok(rd) = std::fs::read_dir(dir) {
        for e in rd.flatten() {
            let n = e.file_name().to_string_lossy().into_owned();
            if let Some(num) = n
                .strip_prefix(&format!("{base}.~"))
                .and_then(|r| r.strip_suffix('~'))
                .and_then(|r| r.parse::<u64>().ok())
            {
                out.push((num, e.path()));
            }
        }
    }
    out.sort();
    out
}

fn io_err(p: &Path, e: std::io::Error) -> ClassifyError {
    ClassifyError::new("config", format!("{}: {e}", p.display()))
}

/// Atomic replace: write `<file>.tmp`, fsync, copy the current file to `<file>.~N~` (keep 20), rename.
fn atomic_write(path: &Path, text: &str) -> Result<PathBuf> {
    use std::io::Write;
    let tmp = path.with_file_name(format!("{}.tmp", file_name(path)));
    {
        let mut f = std::fs::File::create(&tmp).map_err(|e| io_err(&tmp, e))?;
        f.write_all(text.as_bytes()).map_err(|e| io_err(&tmp, e))?;
        f.sync_all().map_err(|e| io_err(&tmp, e))?;
    }
    let existing = backups(path);
    let next = existing.last().map_or(1, |(n, _)| n + 1);
    let backup = path.with_file_name(format!("{}.~{next}~", file_name(path)));
    if path.exists() {
        std::fs::copy(path, &backup).map_err(|e| io_err(&backup, e))?;
    }
    std::fs::rename(&tmp, path).map_err(|e| io_err(path, e))?;
    if let Some(dir) = path.parent().filter(|d| !d.as_os_str().is_empty())
        && let Ok(d) = std::fs::File::open(dir)
    {
        let _ = d.sync_all();
    }
    let all = backups(path);
    if all.len() > KEEP_BACKUPS {
        for (_, p) in &all[..all.len() - KEEP_BACKUPS] {
            let _ = std::fs::remove_file(p);
        }
    }
    Ok(backup)
}

impl ConfigFile {
    /// Load and validate. `Err` carries every issue (code `config`).
    pub fn load(path: &Path) -> std::result::Result<ConfigFile, Vec<ConfigIssue>> {
        let name = file_name(path);
        let text = std::fs::read_to_string(path).map_err(|e| {
            vec![ConfigIssue {
                file: name.clone(),
                line: 0,
                path: "(file)".into(),
                reason: format!("cannot be read ({e})"),
            }]
        })?;
        let config = parse(&text, &name)?;
        Ok(ConfigFile {
            path: path.to_path_buf(),
            hash: sha256_hex(&text),
            text,
            config,
        })
    }

    /// The file name used in messages.
    pub fn name(&self) -> String {
        file_name(&self.path)
    }

    /// Re-read the file after a hand edit; on success `self` is replaced, on failure it is kept (§6).
    pub fn reload(&mut self) -> Reload {
        self.reload_checked(&|_| vec![])
    }

    /// [`ConfigFile::reload`] with an extra check on the new text (e.g. [`check_orgs`]); its issues make the
    /// reload invalid and keep the last valid config.
    pub fn reload_checked(&mut self, extra: &dyn Fn(&str) -> Vec<ConfigIssue>) -> Reload {
        let text = match std::fs::read_to_string(&self.path) {
            Ok(t) => t,
            Err(e) => {
                return Reload::Invalid(vec![ConfigIssue {
                    file: self.name(),
                    line: 0,
                    path: "(file)".into(),
                    reason: format!("cannot be read ({e})"),
                }]);
            }
        };
        let hash = sha256_hex(&text);
        if hash == self.hash {
            return Reload::Unchanged;
        }
        match parse(&text, &self.name()) {
            Ok(config) => {
                let more = extra(&text);
                if !more.is_empty() {
                    return Reload::Invalid(more);
                }
                let changes = unified_diff(&self.text, &text, "x")
                    .lines()
                    .skip(2)
                    .filter(|l| l.starts_with('+') || l.starts_with('-'))
                    .count();
                self.text = text;
                self.hash = hash;
                self.config = config;
                Reload::Reloaded { changes }
            }
            Err(issues) => Reload::Invalid(issues),
        }
    }

    /// Stage edits: apply them to a copy of the document, validate, diff. `used` counts the uses of an item
    /// (labels, suggestions) so removals of used items become archives (V20).
    pub fn stage(&self, edits: &[ConfigEdit], used: &dyn Fn(&ItemRef) -> usize) -> Result<Staged> {
        let mut doc: toml_edit::DocumentMut = self
            .text
            .parse()
            .map_err(|e: toml_edit::TomlError| cfg_err(e.message().to_string()))?;
        let mut summary = vec![];
        let mut archived_instead = vec![];
        for e in edits {
            summary.push(apply_edit(
                &mut doc,
                &self.config,
                e,
                used,
                &mut archived_instead,
            )?);
        }
        let new_text = doc.to_string();
        self.staged_text(new_text, summary, archived_instead)
    }

    fn staged_text(
        &self,
        new_text: String,
        summary: Vec<String>,
        archived_instead: Vec<(ItemRef, usize)>,
    ) -> Result<Staged> {
        if let Err(issues) = parse(&new_text, &self.name()) {
            let mut e = cfg_err(format!(
                "the change would make {} invalid: {}",
                self.name(),
                issues.first().map(ToString::to_string).unwrap_or_default()
            ));
            for i in issues.iter().skip(1) {
                e = e.hint(i.to_string());
            }
            return Err(e);
        }
        Ok(Staged {
            base_hash: self.hash.clone(),
            new_hash: sha256_hex(&new_text),
            diff: unified_diff(&self.text, &new_text, &self.name()),
            new_text,
            summary,
            archived_instead,
        })
    }

    /// Stage restoring an earlier version of the file (config undo).
    pub fn stage_restore(&self, hash: &str) -> Result<Staged> {
        let found = backups(&self.path).into_iter().rev().find_map(|(_, p)| {
            std::fs::read_to_string(&p)
                .ok()
                .filter(|t| sha256_hex(t) == hash)
        });
        let text = found.ok_or_else(|| {
            cfg_err(format!(
                "no backup of {} with that content is left",
                self.name()
            ))
            .hint(format!("{KEEP_BACKUPS} backups are kept"))
        })?;
        self.staged_text(text, vec!["restored the previous version".into()], vec![])
    }

    /// Write a staged change atomically. Refuses (`conflict`) if the file on disk no longer hashes to
    /// `staged.base_hash`. Returns the path of the backup the old file was copied to.
    pub fn write(&mut self, staged: &Staged) -> Result<PathBuf> {
        let disk = std::fs::read_to_string(&self.path).unwrap_or_default();
        if sha256_hex(&disk) != staged.base_hash {
            let mut e = ClassifyError::new(
                "conflict",
                format!("{} changed on disk; reload first", self.name()),
            );
            let d = unified_diff(&self.text, &disk, &self.name());
            if !d.is_empty() {
                e = e.hint(d);
            }
            return Err(e);
        }
        let backup = atomic_write(&self.path, &staged.new_text)?;
        let config = parse(&staged.new_text, &self.name())
            .map_err(|i| cfg_err(i.first().map(ToString::to_string).unwrap_or_default()))?;
        self.text = staged.new_text.clone();
        self.hash = staged.new_hash.clone();
        self.config = config;
        Ok(backup)
    }

    /// Restore the backup whose content hashes to `hash` (config undo), as a normal atomic write.
    pub fn restore(&mut self, hash: &str) -> Result<PathBuf> {
        let staged = self.stage_restore(hash)?;
        self.write(&staged)
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    pub(crate) const SAMPLE: &str = r#"version = 1

[jev]
enabled = false            # stays off in v0.11.0
fields  = ["description", "payee", "amount", "date"]

[label_types.project]
title   = "Project"
choices = [
  { id = "harvest", name = "Harvest Blend launch", aliases = ["Harvest", "HBL"] },
  { id = "refit",   name = "Wayfinding refit",     aliases = ["wayfinding"] },
  { id = "none",    name = "No project" },
]
[label_types.bearer]
title   = "Bearer"
choices = [{ id = "roast", name = "Tallowmere" }, { id = "hold", name = "Brackenridge" }]
[label_types.treatment]
title   = "Treatment"
choices = [{ id = "keep", name = "Keep" }, { id = "intercompany", name = "Intercompany" }]
[label_types.should_be_account]
title   = "Should-be account"
kind    = "account"

# the alias rule
[[rules]]
id = "r-alias"
priority = 100
when  = { description_has_alias = "project" }
set   = { project = "$matched" }

[[rules]]
id = "r-hollowpine"
priority = 90
when  = { payee = "Hollowpine Packaging Works" }
set   = { bearer = "roast", project = "harvest" }
confidence = 0.85

[tree]
root = "n-business"
[tree.nodes.n-business]
type = "noul"
instructions = "Was this payment made for a business?"
criteria = { true = "business", false = "personal" }
yes = "n-project"
no = { set = { treatment = "keep" } }
auto = 0.90
queue_below = 0.50
[tree.nodes.n-project]
type = "choice"
instructions = "Which project?"
branches = { harvest = { set = { bearer = "roast" } }, none = { queue = true } }

[groups]
kinds = ["intercompany_chain", "split"]
roles = ["origin", "mirror", "settlement"]
"#;

    #[test]
    fn parses_the_sample() {
        let c = parse(SAMPLE, "classify.toml").unwrap();
        assert_eq!(c.label_types.len(), 4);
        assert_eq!(c.queue.required, vec!["project", "bearer", "treatment"]);
        assert_eq!(c.queue.threshold.to_string(), "0.9");
        assert_eq!(c.rules[1].confidence.to_string(), "0.85");
        assert_eq!(c.rules_by_priority()[0].id, "r-alias");
        assert_eq!(
            c.label_type("should_be_account").unwrap().kind,
            LabelKind::Account
        );
        assert_eq!(c.tree.as_ref().unwrap().nodes.len(), 2);
    }

    #[test]
    fn errors_carry_file_line_path_and_did_you_mean() {
        let bad = SAMPLE.replace(
            "set   = { bearer = \"roast\", project",
            "set   = { bearer = \"roats\", project",
        );
        let issues = parse(&bad, "classify.toml").unwrap_err();
        assert_eq!(
            issues[0].to_string(),
            "classify.toml:35: rules[1].set.bearer = \"roats\": unknown choice (did you mean \"roast\"?)"
        );
        let syntax = parse("[[rules]\nid=1", "classify.toml").unwrap_err();
        assert!(
            syntax[0]
                .to_string()
                .starts_with("classify.toml:1: (syntax):"),
            "{}",
            syntax[0]
        );
        let jev = parse(
            &SAMPLE.replace("enabled = false", "enabled = true"),
            "classify.toml",
        )
        .unwrap_err();
        assert!(jev[0].reason.starts_with("jev_not_enabled"));
        let field = parse(
            &SAMPLE.replace("\"date\"]", "\"date\", \"org\"]"),
            "classify.toml",
        )
        .unwrap_err();
        assert!(field[0].reason.contains("\"org\" is not allowed"));
        let matched = parse(
            &SAMPLE.replace(
                "set   = { project = \"$matched\" }",
                "set   = { bearer = \"$matched\" }",
            ),
            "c",
        )
        .unwrap_err();
        assert!(matched[0].reason.contains("$matched needs"));
    }

    fn tmpdir(name: &str) -> PathBuf {
        let d =
            std::env::temp_dir().join(format!("xa-classify-config-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    #[test]
    fn stage_write_rotate_conflict_and_restore() {
        let d = tmpdir("write");
        let p = d.join("classify.toml");
        std::fs::write(&p, SAMPLE).unwrap();
        let mut f = ConfigFile::load(&p).unwrap();
        let before = f.hash.clone();
        let st = f
            .stage(
                &[ConfigEdit::AddChoice {
                    label_type: "project".into(),
                    choice: Choice {
                        id: "cupping".into(),
                        name: "Cupping room fit-out".into(),
                        aliases: vec![],
                        archived: false,
                    },
                }],
                &|_| 0,
            )
            .unwrap();
        assert!(
            st.diff
                .contains("+  { id = \"cupping\", name = \"Cupping room fit-out\" },"),
            "{}",
            st.diff
        );
        assert!(
            st.new_text.contains("# stays off in v0.11.0"),
            "comments survive"
        );
        assert_eq!(st.summary, vec!["added project \"Cupping room fit-out\""]);
        let backup = f.write(&st).unwrap();
        assert!(backup.ends_with("classify.toml.~1~"));
        assert_eq!(std::fs::read_to_string(&backup).unwrap(), SAMPLE);
        assert!(
            f.config
                .label_type("project")
                .unwrap()
                .choice("cupping")
                .is_some()
        );
        // conflict: the file changed on disk after staging
        let st2 = f
            .stage(
                &[ConfigEdit::ArchiveRule {
                    id: "r-alias".into(),
                }],
                &|_| 0,
            )
            .unwrap();
        std::fs::write(&p, format!("{}\n# hand edit\n", f.text)).unwrap();
        let e = f.write(&st2).unwrap_err();
        assert_eq!(e.code, "conflict");
        assert!(e.message.contains("changed on disk; reload first"));
        assert!(matches!(f.reload(), Reload::Reloaded { changes: 2 }));
        // restore the original
        f.restore(&before).unwrap();
        assert_eq!(std::fs::read_to_string(&p).unwrap(), SAMPLE);
        assert_eq!(backups(&p).len(), 2);
    }

    #[test]
    fn hand_edit_error_keeps_the_last_valid_config() {
        let d = tmpdir("reload");
        let p = d.join("classify.toml");
        std::fs::write(&p, SAMPLE).unwrap();
        let mut f = ConfigFile::load(&p).unwrap();
        assert_eq!(f.reload(), Reload::Unchanged);
        std::fs::write(
            &p,
            SAMPLE.replace("project = \"harvest\"", "project = \"harvset\""),
        )
        .unwrap();
        let Reload::Invalid(issues) = f.reload() else {
            panic!()
        };
        assert!(issues[0].to_string().contains("did you mean \"harvest\""));
        assert_eq!(f.text, SAMPLE, "last valid config stays live");
        assert!(f.config.rule("r-hollowpine").is_some());
    }

    #[test]
    fn v20_remove_used_choice_archives() {
        let d = tmpdir("v20");
        let p = d.join("classify.toml");
        std::fs::write(&p, SAMPLE).unwrap();
        let f = ConfigFile::load(&p).unwrap();
        let used = |i: &ItemRef| match i {
            ItemRef::Choice { id, .. } if id == "refit" => 37,
            _ => 0,
        };
        let st = f
            .stage(
                &[
                    ConfigEdit::RemoveChoice {
                        label_type: "project".into(),
                        id: "refit".into(),
                    },
                    ConfigEdit::RemoveChoice {
                        label_type: "treatment".into(),
                        id: "intercompany".into(),
                    },
                    ConfigEdit::RemoveRule {
                        id: "r-alias".into(),
                    },
                ],
                &used,
            )
            .unwrap();
        let c = parse(&st.new_text, "c").unwrap();
        assert!(
            c.label_type("project")
                .unwrap()
                .choice("refit")
                .unwrap()
                .archived
        );
        assert!(
            c.label_type("treatment")
                .unwrap()
                .choice("intercompany")
                .is_none()
        );
        assert!(c.rule("r-alias").is_none());
        assert_eq!(st.archived_instead.len(), 1);
        assert_eq!(
            st.summary[0],
            "archived project \"refit\" (used by 37 lines, so not deleted)"
        );
        // a choice a rule sets is archived too
        let st = f
            .stage(
                &[ConfigEdit::RemoveChoice {
                    label_type: "bearer".into(),
                    id: "roast".into(),
                }],
                &|_| 0,
            )
            .unwrap_err();
        assert!(st.message.contains("archived"), "{}", st.message);
    }

    #[test]
    fn rule_round_trips_through_toml() {
        let r = Rule {
            id: "r-x".into(),
            priority: 5,
            when: vec![
                Cond::PayeeHas(vec!["Hollowpine".into()]),
                Cond::AmountBetween("-500.13".parse().unwrap(), "-12.07".parse().unwrap()),
                Cond::DateBetween("2025-01-01".into(), "2025-12-31".into()),
                Cond::Sign(Sign::Out),
                Cond::OrgIn(vec!["roast".into()]),
            ],
            set: vec![("bearer".into(), SetValue::Choice("hold".into()))],
            group_kind: None,
            confidence: "0.75".parse().unwrap(),
            archived: false,
        };
        let f = {
            let d = tmpdir("rt");
            let p = d.join("classify.toml");
            std::fs::write(&p, SAMPLE).unwrap();
            ConfigFile::load(&p).unwrap()
        };
        let st = f.stage(&[ConfigEdit::AddRule(r.clone())], &|_| 0).unwrap();
        let c = parse(&st.new_text, "c").unwrap();
        assert_eq!(c.rule("r-x").unwrap(), &r);
    }

    #[test]
    fn diff_is_unified() {
        let d = unified_diff("a\nb\nc\n", "a\nB\nc\n", "f");
        assert_eq!(d, "--- f\n+++ f\n@@ -1,3 +1,3 @@\n a\n-b\n+B\n c\n");
        assert_eq!(unified_diff("a\n", "a\n", "f"), "");
    }
}
