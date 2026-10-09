//! Explainer visualisations (walkthroughs design §4): pure models computed from the books, never typed-in
//! numbers. Each model is serialisable (the CLI prints it, the TUI draws it) and every element carries an
//! `id` (the TUI registers it as the anchor `viz:<id>`, so a walkthrough layer can point at it).
//!
//! Numbers come through the same entry points the trial balance and the ledger use: the `View` (main, or
//! main + a manifest), the ledger builder, the balance basis and the intercompany engine. Exact decimals
//! throughout; every list has a deterministic order.
//!
//! Kinds:
//! - `bridge`: an intercompany gap as a waterfall: the difference, the reconciling items (the `ic` engine's
//!   bridge: the opening difference and the movement items), the residual;
//! - `flow`: money between orgs: per configured pair, who owes whom at a date (`owed`) or what moved over a
//!   period (`paid`);
//! - `t_account`: one account as a T: opening, the largest debits and credits (the rest summed), closing;
//! - `timeline`: one account's month-end balances and each month's movement, months with journals marked;
//! - `before_after`: what a manifest changes in one org: TB lines and statement totals before → after;
//! - `checklist`: the project's close worklist (deadlines, questions by owner, agent work, postings).

use crate::balances::{Basis, Mode};
use crate::dates::{add_days, days_from_civil, fiscal_year_start, iso, parse_iso};
use crate::intercompany::{self as icm, IcContext};
use crate::ledger::{Ledger, Posting};
use crate::money::{self, ZERO, fmt};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize, Serializer};
use serde_json::Value;
use std::collections::BTreeMap;
use std::sync::Arc;

// ── the spec ─────────────────────────────────────────────────────────────────────────────────────

/// `owed`: balances at a date; `paid`: movements over a period.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FlowBasis {
    #[default]
    Owed,
    Paid,
}

/// What to show: `{"kind": "bridge", "pair": "hold-design"}` and the like. Missing dates default to the
/// view's as-of date (`to`) and the fiscal-year start (`from`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Spec {
    Bridge {
        pair: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        as_of: Option<String>,
    },
    Flow {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        orgs: Option<Vec<String>>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        as_of: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        from: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        to: Option<String>,
        #[serde(default)]
        basis: FlowBasis,
    },
    TAccount {
        org: String,
        account: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        from: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        to: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        top: Option<usize>,
    },
    Timeline {
        org: String,
        account: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        from: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        to: Option<String>,
    },
    BeforeAfter {
        /// The manifest's name (None: the current one).
        #[serde(default, skip_serializing_if = "Option::is_none")]
        manifest: Option<String>,
        org: String,
    },
    Checklist {
        /// Relative to the project config's folder.
        file: String,
    },
}

/// The kinds, in the order the docs and the palette list them.
pub const KINDS: [&str; 6] = [
    "bridge",
    "flow",
    "t_account",
    "timeline",
    "before_after",
    "checklist",
];

/// Largest debits / credits a T-account lists by default (the rest is one "N more" line).
pub const TOP_DEFAULT: usize = 5;

impl Spec {
    /// Parse a JSON spec (the agent verb's argument, the CLI's `--spec`). The error names what is wrong.
    pub fn parse(text: &str) -> Result<Spec, VizError> {
        let v: Value = serde_json::from_str(text)
            .map_err(|e| VizError::usage(format!("viz spec is not JSON: {e}")))?;
        Spec::from_value(&v)
    }

    pub fn from_value(v: &Value) -> Result<Spec, VizError> {
        let kind = v.get("kind").and_then(Value::as_str).unwrap_or("");
        if !KINDS.contains(&kind) {
            return Err(VizError::usage(format!(
                "viz kind `{kind}` is not one of {}.",
                KINDS.join(", ")
            )));
        }
        let s: Spec = serde_json::from_value(v.clone())
            .map_err(|e| VizError::usage(format!("viz spec ({kind}): {e}")))?;
        s.check()?;
        Ok(s)
    }

    /// Dates are ISO days; `top` is 1–50; a period's `from` is not after its `to`.
    pub fn check(&self) -> Result<(), VizError> {
        let date = |d: &Option<String>, what: &str| -> Result<(), VizError> {
            match d {
                Some(d) if d.len() != 10 || parse_iso(d).is_none() => Err(VizError::usage(
                    format!("viz {what} needs a date YYYY-MM-DD, got `{d}`."),
                )),
                _ => Ok(()),
            }
        };
        let period = |f: &Option<String>, t: &Option<String>| -> Result<(), VizError> {
            date(f, "from")?;
            date(t, "to")?;
            if let (Some(f), Some(t)) = (f, t)
                && f > t
            {
                return Err(VizError::usage(format!(
                    "viz period runs backwards: from {f} is after to {t}."
                )));
            }
            Ok(())
        };
        match self {
            Spec::Bridge { pair, as_of } => {
                if pair.is_empty() {
                    return Err(VizError::usage("viz bridge needs a `pair`."));
                }
                date(as_of, "as_of")
            }
            Spec::Flow {
                as_of, from, to, ..
            } => {
                date(as_of, "as_of")?;
                period(from, to)
            }
            Spec::TAccount {
                from, to, top, org, ..
            } => {
                if org.is_empty() {
                    return Err(VizError::usage("viz t_account needs an `org`."));
                }
                if top.is_some_and(|n| !(1..=50).contains(&n)) {
                    return Err(VizError::usage("viz t_account `top` is 1–50."));
                }
                period(from, to)
            }
            Spec::Timeline { from, to, .. } => period(from, to),
            Spec::BeforeAfter { org, .. } if org.is_empty() => {
                Err(VizError::usage("viz before_after needs an `org`."))
            }
            Spec::Checklist { file } if file.is_empty() => {
                Err(VizError::usage("viz checklist needs a `file`."))
            }
            _ => Ok(()),
        }
    }

    pub fn kind(&self) -> &'static str {
        match self {
            Spec::Bridge { .. } => "bridge",
            Spec::Flow { .. } => "flow",
            Spec::TAccount { .. } => "t_account",
            Spec::Timeline { .. } => "timeline",
            Spec::BeforeAfter { .. } => "before_after",
            Spec::Checklist { .. } => "checklist",
        }
    }

    /// The canonical JSON text (key order fixed by the type).
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_default()
    }
}

/// Why a model could not be computed: `usage` (the spec), `not_found` (a pair, org or account), `record`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VizError {
    pub code: &'static str,
    pub message: String,
}

impl VizError {
    pub fn usage(m: impl Into<String>) -> VizError {
        VizError {
            code: "usage",
            message: m.into(),
        }
    }
    pub fn not_found(m: impl Into<String>) -> VizError {
        VizError {
            code: "not_found",
            message: m.into(),
        }
    }
    pub fn record(m: impl std::fmt::Display) -> VizError {
        VizError {
            code: "record",
            message: m.to_string(),
        }
    }
}

impl std::fmt::Display for VizError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for VizError {}

fn ser_money<S: Serializer>(v: &Decimal, s: S) -> Result<S::Ok, S::Error> {
    s.serialize_str(&money::money_str(*v))
}

fn ser_money_opt<S: Serializer>(v: &Option<Decimal>, s: S) -> Result<S::Ok, S::Error> {
    match v {
        Some(v) => s.serialize_str(&money::money_str(*v)),
        None => s.serialize_none(),
    }
}

/// Where a number came from: the document (org, entity, Xero id).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DocRef {
    pub org: String,
    pub source: String,
    pub id: String,
}

// ── the models ───────────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Model {
    Bridge(Bridge),
    Flow(Flow),
    TAccount(TAccount),
    Timeline(Timeline),
    BeforeAfter(BeforeAfter),
    Checklist(Checklist),
}

impl Model {
    pub fn title(&self) -> &str {
        match self {
            Model::Bridge(m) => &m.title,
            Model::Flow(m) => &m.title,
            Model::TAccount(m) => &m.title,
            Model::Timeline(m) => &m.title,
            Model::BeforeAfter(m) => &m.title,
            Model::Checklist(m) => &m.title,
        }
    }

    /// One plain-English line saying what the panel shows (the subtitle under the title).
    pub fn lead(&self) -> String {
        match self {
            Model::Bridge(b) => format!(
                "Why {} and {} disagree by {}, item by item",
                b.a_org,
                b.b_org,
                fmt(b.start.amount.abs())
            ),
            Model::Flow(f) => match f.basis {
                FlowBasis::Owed => format!(
                    "Who owes whom at {}",
                    f.as_of.as_deref().unwrap_or("the date")
                ),
                FlowBasis::Paid => format!(
                    "Who paid or charged whom from {} to {}",
                    f.from.as_deref().unwrap_or(""),
                    f.to.as_deref().unwrap_or("")
                ),
            },
            Model::TAccount(t) => format!(
                "Money into (debit) and out of (credit) {}, {} to {}",
                t.account, t.from, t.to
            ),
            Model::Timeline(t) => format!("How the balance of {} moved, month by month", t.account),
            Model::BeforeAfter(b) => format!(
                "What manifest {} would change in {}'s books",
                b.manifest, b.org
            ),
            Model::Checklist(_) => {
                "Where the close stands: what is done and what waits on whom".into()
            }
        }
    }

    /// Every element's id (the `viz:<id>` anchors), in drawing order.
    pub fn ids(&self) -> Vec<String> {
        match self {
            Model::Bridge(m) => std::iter::once(m.start.id.clone())
                .chain(m.items.iter().map(|i| i.id.clone()))
                .chain(std::iter::once(m.end.id.clone()))
                .collect(),
            Model::Flow(m) => m
                .nodes
                .iter()
                .map(|n| n.id.clone())
                .chain(m.edges.iter().map(|e| e.id.clone()))
                .collect(),
            Model::TAccount(m) => {
                let mut v = vec!["opening".to_string()];
                v.extend(m.debits.iter().map(|e| e.id.clone()));
                v.extend(m.debits_more.iter().map(|e| e.id.clone()));
                v.extend(m.credits.iter().map(|e| e.id.clone()));
                v.extend(m.credits_more.iter().map(|e| e.id.clone()));
                v.push("closing".into());
                v
            }
            Model::Timeline(m) => m.months.iter().map(|x| x.id.clone()).collect(),
            Model::BeforeAfter(m) => m
                .lines
                .iter()
                .map(|l| l.id.clone())
                .chain(m.totals.iter().map(|t| t.id.clone()))
                .collect(),
            Model::Checklist(m) => m
                .groups
                .iter()
                .flat_map(|g| {
                    std::iter::once(g.id.clone()).chain(g.items.iter().map(|i| i.id.clone()))
                })
                .collect(),
        }
    }
}

/// One bar of a bridge.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Step {
    pub id: String,
    pub label: String,
    /// A second line in plain words (where it is booked, why it is an item).
    pub detail: String,
    #[serde(serialize_with = "ser_money")]
    pub amount: Decimal,
    /// What is left to explain after this bar.
    #[serde(serialize_with = "ser_money")]
    pub running: Decimal,
    pub sources: Vec<DocRef>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Bridge {
    pub title: String,
    pub pair: String,
    pub a_org: String,
    pub b_org: String,
    pub as_of: String,
    /// The engine's status: matches, bridged, unexplained, not compared, not supported.
    pub status: String,
    /// What each side's books say, in words.
    pub a_says: String,
    pub b_says: String,
    /// The difference to explain (the first bar).
    pub start: Step,
    pub items: Vec<Step>,
    /// What is left (the last bar): 0.00 when fully explained.
    pub end: Step,
    #[serde(serialize_with = "ser_money")]
    pub residual: Decimal,
    /// Why there are no bars (not compared / not supported).
    pub reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Node {
    pub id: String,
    pub org: String,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Edge {
    pub id: String,
    pub pair: String,
    pub label: String,
    /// The money goes from → to (owed: the debtor → the creditor; paid: who paid or charged → whom).
    pub from: String,
    pub to: String,
    #[serde(serialize_with = "ser_money_opt")]
    pub amount: Option<Decimal>,
    /// "roast owes hold 15,568.80".
    pub words: String,
    /// The other side's view when it differs (owed), or the reverse leg (paid).
    pub note: Option<String>,
    pub status: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Flow {
    pub title: String,
    pub basis: FlowBasis,
    pub as_of: Option<String>,
    pub from: Option<String>,
    pub to: Option<String>,
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TEntry {
    pub id: String,
    pub date: String,
    pub label: String,
    pub detail: String,
    #[serde(serialize_with = "ser_money")]
    pub amount: Decimal,
    /// The rest summed: how many postings this line stands for (1 for a posting).
    pub count: usize,
    pub source: Option<DocRef>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TAccount {
    pub title: String,
    pub org: String,
    pub code: String,
    pub account: String,
    pub class: String,
    pub from: String,
    pub to: String,
    /// Net debit (positive) or credit (negative), as the ledger signs it.
    #[serde(serialize_with = "ser_money")]
    pub opening: Decimal,
    pub opening_source: String,
    pub debits: Vec<TEntry>,
    pub debits_more: Option<TEntry>,
    pub credits: Vec<TEntry>,
    pub credits_more: Option<TEntry>,
    #[serde(serialize_with = "ser_money")]
    pub debit_total: Decimal,
    #[serde(serialize_with = "ser_money")]
    pub credit_total: Decimal,
    #[serde(serialize_with = "ser_money")]
    pub closing: Decimal,
    pub debit_count: usize,
    pub credit_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Month {
    pub id: String,
    /// `2025-03`.
    pub month: String,
    /// The last day shown (the month-end, or `to` in the last month).
    pub end: String,
    #[serde(serialize_with = "ser_money")]
    pub balance: Decimal,
    #[serde(serialize_with = "ser_money")]
    pub movement: Decimal,
    pub postings: usize,
    /// Manual journals that landed on the account this month.
    pub journals: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Timeline {
    pub title: String,
    pub org: String,
    pub code: String,
    pub account: String,
    pub from: String,
    pub to: String,
    #[serde(serialize_with = "ser_money")]
    pub opening: Decimal,
    pub months: Vec<Month>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct BaLine {
    pub id: String,
    /// `tb` (an account), `bs` or `pl` (a statement total).
    pub section: String,
    pub code: String,
    pub label: String,
    #[serde(serialize_with = "ser_money")]
    pub before: Decimal,
    #[serde(serialize_with = "ser_money")]
    pub after: Decimal,
    #[serde(serialize_with = "ser_money")]
    pub delta: Decimal,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct BeforeAfter {
    pub title: String,
    pub manifest: String,
    pub org: String,
    pub as_of: String,
    pub entries: usize,
    pub conflicts: usize,
    /// Trial-balance accounts that move (net debit, as the ledger signs it).
    pub lines: Vec<BaLine>,
    /// Balance sheet and P&L totals (natural signs: assets, liabilities, equity, revenue, expenses, net).
    pub totals: Vec<BaLine>,
}

/// How an item's status reads (the colour and the glyph; the word itself is always shown verbatim).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Tone {
    Done,
    Open,
    Blocked,
    Late,
    Soon,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CheckItem {
    pub id: String,
    pub key: String,
    pub text: String,
    /// The status word from the file, verbatim (a deadline: `12 days left`, `due 3 days ago`, `filed 2026-09-10`).
    pub status: String,
    pub tone: Tone,
    pub detail: Option<String>,
    pub days_left: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CheckGroup {
    pub id: String,
    pub title: String,
    pub items: Vec<CheckItem>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Checklist {
    pub title: String,
    pub file: String,
    pub year: Option<String>,
    pub comment: Option<String>,
    pub today: String,
    pub groups: Vec<CheckGroup>,
    /// Status word → count over every item (deadlines excluded).
    pub counts: BTreeMap<String, usize>,
    /// Other top-level sections of the file, named (not drawn).
    pub also: Vec<String>,
}

// ── bridge ───────────────────────────────────────────────────────────────────────────────────────

fn dec(v: &Value) -> Option<Decimal> {
    v.as_str().and_then(|s| money::parse(s).ok())
}

fn sv(v: &Value) -> String {
    v.as_str().unwrap_or("").to_string()
}

/// `x owes y 1.00` from one side's net debit on its paired accounts (positive: the other side owes it).
fn owes(me: &str, other: &str, net_debit: Decimal) -> String {
    if net_debit > ZERO {
        format!("{other} owes {me} {}", fmt(net_debit))
    } else if net_debit < ZERO {
        format!("{me} owes {other} {}", fmt(-net_debit))
    } else {
        format!("nothing owed between {me} and {other}")
    }
}

/// A bridge from one pair of the intercompany engine's result (`ic::build` → `pairs[i]`).
pub fn bridge(pair: &Value) -> Bridge {
    let a = sv(&pair["a_org"]);
    let b = sv(&pair["b_org"]);
    let as_of = sv(&pair["as_of"]);
    let status = sv(&pair["status"]);
    let label = pair["label"]
        .as_str()
        .map(str::to_string)
        .unwrap_or_else(|| format!("{a} ↔ {b}"));
    let diff = dec(&pair["difference"]);
    let a_net = dec(&pair["a_net"]);
    let b_net = dec(&pair["b_party_net"]);
    let a_says = a_net.map_or_else(
        || format!("{a}: no balance to compare"),
        |n| format!("{a}'s books: {}", owes(&a, &b, n)),
    );
    let b_says = b_net.map_or_else(
        || format!("{b}: no balance to compare"),
        |n| format!("{b}'s books: {}", owes(&b, &a, n)),
    );
    let mut items = vec![];
    let start_amt = diff.unwrap_or(ZERO);
    let mut running = start_amt;
    let reason = pair["reason"].as_str().map(str::to_string);
    if let Some(open) = dec(&pair["opening_difference"]).filter(|d| *d != ZERO) {
        running -= open;
        items.push(Step {
            id: format!("item/{}", items.len() + 1),
            label: format!(
                "Carried in from before {}",
                pair["year_start"]
                    .as_str()
                    .and_then(|y| y.get(..4))
                    .unwrap_or("this year")
            ),
            detail: format!(
                "the books already disagreed at {} (a prior-period difference)",
                pair["prior"].as_str().unwrap_or("the prior year-end")
            ),
            amount: open,
            running,
            sources: vec![],
        });
    }
    for it in pair["movement_items"].as_array().into_iter().flatten() {
        let amt = dec(&it["amount"]).unwrap_or(ZERO);
        running -= amt;
        let what = [
            &it["description"],
            &it["reference"],
            &it["contact"],
            &it["kind"],
        ]
        .into_iter()
        .map(sv)
        .find(|s| !s.trim().is_empty())
        .unwrap_or_else(|| "(no description)".into());
        let org = sv(&it["org"]);
        let mut detail = format!("{} · {org} · {}", sv(&it["date"]), sv(&it["kind"]));
        if let Some(c) = it["contact"]
            .as_str()
            .filter(|c| !c.is_empty() && *c != what)
        {
            detail.push_str(&format!(" · {c}"));
        }
        if let Some(w) = it["why"].as_str() {
            detail.push_str(&format!(" · {w}"));
        }
        items.push(Step {
            id: format!("item/{}", items.len() + 1),
            label: what,
            detail,
            amount: amt,
            running,
            sources: vec![DocRef {
                org,
                source: sv(&it["source"]),
                id: sv(&it["source_id"]),
            }],
        });
    }
    let residual = match dec(&pair["residual"]) {
        Some(r) if status != "matches" => r,
        _ => running,
    };
    let start = Step {
        id: "start".into(),
        label: format!("Difference at {as_of}"),
        detail: format!("{a}'s side plus {b}'s side; should be 0.00"),
        amount: start_amt,
        running: start_amt,
        sources: vec![],
    };
    let end = Step {
        id: "residual".into(),
        label: if reason.is_some() {
            "Not compared".into()
        } else {
            "Unexplained".into()
        },
        detail: if residual == ZERO {
            "fully explained".into()
        } else {
            "no document explains this part yet".into()
        },
        amount: residual,
        running: residual,
        sources: vec![],
    };
    Bridge {
        title: label,
        pair: sv(&pair["id"]),
        a_org: a,
        b_org: b,
        as_of,
        status,
        a_says,
        b_says,
        start,
        items,
        end,
        residual,
        reason,
    }
}

// ── flow ─────────────────────────────────────────────────────────────────────────────────────────

fn in_orgs(orgs: &[String], o: &str) -> bool {
    orgs.iter().any(|x| x == o)
}

/// Balances owed between orgs at the result's date, one edge per configured pair (`ic::build` result).
/// `orgs`: `(alias, name)` in the order to draw them; pairs outside them are left out.
pub fn flow_owed(result: &Value, orgs: &[(String, String)]) -> Flow {
    let aliases: Vec<String> = orgs.iter().map(|o| o.0.clone()).collect();
    let mut edges = vec![];
    for p in result["pairs"].as_array().into_iter().flatten() {
        let (a, b) = (sv(&p["a_org"]), sv(&p["b_org"]));
        if !in_orgs(&aliases, &a) || !in_orgs(&aliases, &b) {
            continue;
        }
        let a_net = dec(&p["a_net"]);
        let b_net = dec(&p["b_party_net"]);
        let status = sv(&p["status"]);
        let (from, to, amount, words) = match a_net {
            Some(n) if n >= ZERO => (b.clone(), a.clone(), Some(n), owes(&a, &b, n)),
            Some(n) => (a.clone(), b.clone(), Some(-n), owes(&a, &b, n)),
            None => (
                b.clone(),
                a.clone(),
                None,
                format!(
                    "not compared: {}",
                    p["reason"].as_str().unwrap_or("no balance")
                ),
            ),
        };
        let note = match (a_net, b_net) {
            (Some(x), Some(y)) if x + y != ZERO => Some(format!(
                "{b}'s books show {} · a gap of {}",
                fmt(y.abs()),
                fmt((x + y).abs())
            )),
            _ => None,
        };
        edges.push(Edge {
            id: format!("edge/{}", sv(&p["id"])),
            pair: sv(&p["id"]),
            label: sv(&p["label"]),
            from,
            to,
            amount,
            words,
            note,
            status,
        });
    }
    Flow {
        title: "Money between the orgs: what is owed".into(),
        basis: FlowBasis::Owed,
        as_of: result["as_of"].as_str().map(str::to_string),
        from: None,
        to: None,
        nodes: nodes(orgs),
        edges,
    }
}

fn nodes(orgs: &[(String, String)]) -> Vec<Node> {
    orgs.iter()
        .map(|(a, n)| Node {
            id: format!("node/{a}"),
            org: a.clone(),
            name: n.clone(),
        })
        .collect()
}

/// What moved between orgs over [from, to], per configured pair, from side a's paired accounts: a debit
/// there is value from a to b (a paid, lent or charged b, or repaid what it owed b), a credit is value
/// from b to a. Two edges per pair (each direction that moved).
pub fn flow_paid(
    ctx: &IcContext,
    orgs: &[(String, String)],
    from: &str,
    to: &str,
) -> Result<Flow, VizError> {
    let aliases: Vec<String> = orgs.iter().map(|o| o.0.clone()).collect();
    let mut edges = vec![];
    for p in &ctx.pairs {
        let (a, b) = (sv(&p["a"]["org"]), sv(&p["b"]["org"]));
        if !in_orgs(&aliases, &a) || !in_orgs(&aliases, &b) {
            continue;
        }
        let id = sv(&p["id"]);
        let label = p["label"]
            .as_str()
            .map(str::to_string)
            .unwrap_or_else(|| format!("{a} ↔ {b}"));
        if !ctx.tenants.contains_key(&a) || !ctx.tenants.contains_key(&b) {
            continue;
        }
        let refs = p["a"]["accounts"].as_array().cloned().unwrap_or_default();
        let ids = ctx.account_ids(&a, &refs).map_err(VizError::record)?;
        let l = ctx.ledger(&a).map_err(VizError::record)?;
        let (mut dr, mut cr, mut ndr, mut ncr) = (ZERO, ZERO, 0usize, 0usize);
        for x in l.postings.iter().filter(|x| {
            ids.contains(&x.account_id) && x.date.as_str() >= from && x.date.as_str() <= to
        }) {
            if x.amount > ZERO {
                dr = money::add(dr, x.amount);
                ndr += 1;
            } else {
                cr = money::add(cr, -x.amount);
                ncr += 1;
            }
        }
        for (dir, f, t, amt, n) in [("ab", &a, &b, dr, ndr), ("ba", &b, &a, cr, ncr)] {
            if n == 0 {
                continue;
            }
            edges.push(Edge {
                id: format!("edge/{id}/{dir}"),
                pair: id.clone(),
                label: label.clone(),
                from: f.clone(),
                to: t.clone(),
                amount: Some(amt),
                words: format!("{f} paid or charged {t} {}", fmt(amt)),
                note: Some(format!(
                    "{} in {a}'s books",
                    if n == 1 {
                        "1 entry".to_string()
                    } else {
                        format!("{n} entries")
                    }
                )),
                status: "moved".into(),
            });
        }
    }
    Ok(Flow {
        title: "Money between the orgs: what moved".into(),
        basis: FlowBasis::Paid,
        as_of: None,
        from: Some(from.into()),
        to: Some(to.into()),
        nodes: nodes(orgs),
        edges,
    })
}

// ── t_account and timeline ───────────────────────────────────────────────────────────────────────

fn entry_label(p: &Posting) -> String {
    // a bank account's own leg carries the placeholder description `bank`: name the counterparty instead
    let desc = if p.description == "bank" {
        ""
    } else {
        p.description.as_str()
    };
    [desc, &p.contact, &p.reference, &p.kind]
        .into_iter()
        .find(|s| !s.trim().is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| "(no description)".into())
}

fn entry_detail(p: &Posting) -> String {
    let kind = if p.kind.is_empty() {
        p.source.to_string()
    } else {
        p.kind.clone()
    };
    match p.contact.as_str() {
        "" => kind,
        c if *c == entry_label(p) => kind,
        c => format!("{kind} · {c}"),
    }
}

/// An account id from a code or an id.
pub fn account_id(ledger: &Ledger, account: &str) -> Option<String> {
    ledger
        .chart
        .by_code(account)
        .map(|a| a.account_id.clone())
        .or_else(|| {
            ledger
                .chart
                .by_id
                .contains_key(account)
                .then(|| account.to_string())
        })
}

/// The account's postings in [from, to], date order (ties in ledger order).
fn postings_in<'a>(ledger: &'a Ledger, id: &str, from: &str, to: &str) -> Vec<&'a Posting> {
    let mut v: Vec<&Posting> = ledger
        .postings
        .iter()
        .filter(|p| p.account_id == id && p.date.as_str() >= from && p.date.as_str() <= to)
        .collect();
    v.sort_by(|a, b| a.date.cmp(&b.date));
    v
}

fn opening_source(basis: &Basis, from: &str) -> String {
    let xero = basis.kind == crate::balances::BasisKind::XeroOpening
        && basis.opening_date.as_deref().is_some_and(|od| from > od);
    if xero {
        format!(
            "from Xero TB {}",
            basis.opening_date.as_deref().unwrap_or("")
        )
    } else {
        "from local history".into()
    }
}

/// One account as a T over [from, to]: the opening on the basis (as the Ledger's postings view opens), the
/// `top` largest debits and credits (ties by date, then document), the rest summed, and the closing.
pub fn t_account(
    org: &str,
    ledger: &Ledger,
    basis: &Basis,
    account: &str,
    from: &str,
    to: &str,
    top: usize,
) -> Result<TAccount, VizError> {
    let id = account_id(ledger, account)
        .ok_or_else(|| VizError::not_found(format!("{org} has no account `{account}`.")))?;
    let acct = ledger.chart.get(&id);
    let opening = basis.opening_for(ledger, &id, from);
    let ps = postings_in(ledger, &id, from, to);
    let side = |debit: bool| -> (Vec<TEntry>, Option<TEntry>, Decimal, usize) {
        let mut v: Vec<&Posting> = ps
            .iter()
            .copied()
            .filter(|p| (p.amount > ZERO) == debit && p.amount != ZERO)
            .collect();
        let total = money::sum(v.iter().map(|p| p.amount.abs()), ZERO);
        let n = v.len();
        v.sort_by(|a, b| {
            b.amount
                .abs()
                .cmp(&a.amount.abs())
                .then_with(|| a.date.cmp(&b.date))
                .then_with(|| a.source_id.cmp(&b.source_id))
                .then_with(|| a.line.cmp(&b.line))
        });
        let tag = if debit { "debit" } else { "credit" };
        let shown: Vec<TEntry> = v
            .iter()
            .take(top)
            .enumerate()
            .map(|(i, p)| TEntry {
                id: format!("{tag}/{}", i + 1),
                date: p.date.clone(),
                label: entry_label(p),
                detail: entry_detail(p),
                amount: p.amount.abs(),
                count: 1,
                source: Some(DocRef {
                    org: org.into(),
                    source: p.source.into(),
                    id: p.source_id.clone(),
                }),
            })
            .collect();
        let rest: Vec<&&Posting> = v.iter().skip(top).collect();
        let more = (!rest.is_empty()).then(|| TEntry {
            id: format!("{tag}/more"),
            date: String::new(),
            label: format!(
                "{} more {tag}{}",
                rest.len(),
                if rest.len() == 1 { "" } else { "s" }
            ),
            detail: String::new(),
            amount: money::sum(rest.iter().map(|p| p.amount.abs()), ZERO),
            count: rest.len(),
            source: None,
        });
        (shown, more, total, n)
    };
    let (debits, debits_more, debit_total, debit_count) = side(true);
    let (credits, credits_more, credit_total, credit_count) = side(false);
    let closing = money::add(money::add(opening, debit_total), -credit_total);
    Ok(TAccount {
        title: format!("{} · {org}", acct.label()),
        org: org.into(),
        code: acct.code.clone(),
        account: acct.name.clone(),
        class: acct.class.clone(),
        from: from.into(),
        to: to.into(),
        opening,
        opening_source: opening_source(basis, from),
        debits,
        debits_more,
        credits,
        credits_more,
        debit_total,
        credit_total,
        closing,
        debit_count,
        credit_count,
    })
}

/// The last day of `date`'s month.
fn month_end(y: i64, m: u32) -> String {
    let (ny, nm) = if m == 12 { (y + 1, 1) } else { (y, m + 1) };
    add_days(&iso(ny, nm, 1), -1).unwrap_or_default()
}

/// One account's balance at each month-end in [from, to] (the last month ends at `to`), the month's
/// movement, and the manual journals that landed on it.
pub fn timeline(
    org: &str,
    ledger: &Ledger,
    basis: &Basis,
    account: &str,
    from: &str,
    to: &str,
) -> Result<Timeline, VizError> {
    let id = account_id(ledger, account)
        .ok_or_else(|| VizError::not_found(format!("{org} has no account `{account}`.")))?;
    let acct = ledger.chart.get(&id);
    let opening = basis.opening_for(ledger, &id, from);
    let ps = postings_in(ledger, &id, from, to);
    let (Some((mut y, mut m, _)), Some(_)) = (parse_iso(from), parse_iso(to)) else {
        return Err(VizError::usage("timeline needs dates YYYY-MM-DD"));
    };
    let mut months = vec![];
    let mut bal = opening;
    let mut start = from.to_string();
    loop {
        let end = month_end(y, m).min(to.to_string());
        let in_month: Vec<&&Posting> = ps
            .iter()
            .filter(|p| p.date.as_str() >= start.as_str() && p.date.as_str() <= end.as_str())
            .collect();
        let mv = money::sum(in_month.iter().map(|p| p.amount), ZERO);
        bal = money::add(bal, mv);
        let mut docs: Vec<&str> = in_month
            .iter()
            .filter(|p| p.source == "ManualJournals")
            .map(|p| p.source_id.as_str())
            .collect();
        docs.dedup();
        let key = format!("{y:04}-{m:02}");
        months.push(Month {
            id: format!("month/{key}"),
            month: key,
            end: end.clone(),
            balance: bal,
            movement: mv,
            postings: in_month.len(),
            journals: docs.len(),
        });
        if end.as_str() >= to || months.len() >= 240 {
            break;
        }
        (y, m) = if m == 12 { (y + 1, 1) } else { (y, m + 1) };
        start = iso(y, m, 1);
    }
    Ok(Timeline {
        title: format!("{} · {org} · month by month", acct.label()),
        org: org.into(),
        code: acct.code.clone(),
        account: acct.name.clone(),
        from: from.into(),
        to: to.into(),
        opening,
        months,
    })
}

// ── before_after ─────────────────────────────────────────────────────────────────────────────────

/// One org's lines from a manifest's effects (`manifest::effects::diff`, the code the CHECK step uses).
pub fn before_after(
    diff: &crate::manifest::effects::Diff,
    org: &str,
) -> Result<BeforeAfter, VizError> {
    let od = diff.orgs.iter().find(|o| o.org == org);
    let lines = od
        .map(|o| {
            o.accounts
                .iter()
                .map(|a| BaLine {
                    id: format!(
                        "line/{}",
                        if a.code.is_empty() {
                            &a.account_id
                        } else {
                            &a.code
                        }
                    ),
                    section: "tb".into(),
                    code: a.code.clone(),
                    label: a.name.clone(),
                    before: a.main,
                    after: a.manifest,
                    delta: a.delta(),
                })
                .collect()
        })
        .unwrap_or_default();
    let totals = od
        .map(|o| {
            o.totals
                .iter()
                .filter(|(_, b, a)| b != a)
                .map(|(label, b, a)| BaLine {
                    id: format!("total/{label}"),
                    section: if matches!(*label, "assets" | "liabilities" | "equity") {
                        "bs".into()
                    } else {
                        "pl".into()
                    },
                    code: String::new(),
                    label: match *label {
                        "net" => "Net profit".to_string(),
                        l => {
                            let mut c = l.chars();
                            c.next()
                                .map(|f| f.to_uppercase().collect::<String>() + c.as_str())
                                .unwrap_or_default()
                        }
                    },
                    before: *b,
                    after: *a,
                    delta: *a - *b,
                })
                .collect()
        })
        .unwrap_or_default();
    Ok(BeforeAfter {
        title: format!("Manifest {} · {org}", diff.manifest),
        manifest: diff.manifest.clone(),
        org: org.into(),
        as_of: diff.as_of.clone(),
        entries: diff.entries,
        conflicts: diff.conflicts,
        lines,
        totals,
    })
}

// ── checklist ────────────────────────────────────────────────────────────────────────────────────

const DONE: [&str; 9] = [
    "done",
    "answered",
    "closed",
    "resolved",
    "complete",
    "completed",
    "posted",
    "filed",
    "verified",
];
const OPEN: [&str; 7] = [
    "open",
    "todo",
    "waiting",
    "pending",
    "prepared",
    "in_progress",
    "asked",
];
const BLOCKED: [&str; 4] = ["blocked", "parked", "on_hold", "stuck"];

fn tone(status: &str) -> Tone {
    let s = status.to_lowercase().replace([' ', '-'], "_");
    if DONE.contains(&s.as_str()) {
        Tone::Done
    } else if OPEN.contains(&s.as_str()) {
        Tone::Open
    } else if BLOCKED.contains(&s.as_str()) {
        Tone::Blocked
    } else {
        Tone::Unknown
    }
}

fn text_of(v: &Value) -> Option<String> {
    match v {
        Value::Null => None,
        Value::String(s) if s.is_empty() => None,
        Value::String(s) => Some(s.clone()),
        Value::Bool(b) => Some(b.to_string()),
        Value::Number(n) => Some(n.to_string()),
        other => Some(other.to_string()),
    }
}

fn days_between(from: &str, to: &str) -> Option<i64> {
    let (y1, m1, d1) = parse_iso(from)?;
    let (y2, m2, d2) = parse_iso(to)?;
    Some(days_from_civil(y2, m2, d2) - days_from_civil(y1, m1, d1))
}

/// A unique id per item: `item/<file id>`, or `item/<section>-<n>` when the file has none or repeats one.
fn item_id(seen: &mut Vec<String>, section: &str, n: usize, v: &Value) -> (String, String) {
    let key = text_of(&v["id"]).unwrap_or_else(|| format!("{section}-{n}"));
    let mut id = format!("item/{key}");
    if seen.contains(&id) {
        id = format!("item/{section}-{n}");
    }
    seen.push(id.clone());
    (id, key)
}

/// The close worklist from the project's JSON (`{comment, year, deadlines, questions, agent_items, postings,
/// …}`): Deadlines (days left from `today`), one "Waiting on <owner>" group per question owner (in the
/// order they first appear), Agent work, then Postings by state. Every item is kept; an owner or state the groups don't name gets its own group; status words are
/// shown verbatim.
pub fn checklist(doc: &Value, file: &str, today: &str) -> Result<Checklist, VizError> {
    let Some(obj) = doc.as_object() else {
        return Err(VizError::usage(format!(
            "{file}: a checklist is a JSON object."
        )));
    };
    let mut seen = vec![];
    let mut groups: Vec<CheckGroup> = vec![];
    let mut counts: BTreeMap<String, usize> = BTreeMap::new();
    let push =
        |groups: &mut Vec<CheckGroup>, gid: String, title: String, item: CheckItem| match groups
            .iter_mut()
            .find(|g| g.id == gid)
        {
            Some(g) => g.items.push(item),
            None => groups.push(CheckGroup {
                id: gid,
                title,
                items: vec![item],
            }),
        };
    let arr = |k: &str| -> Vec<Value> {
        obj.get(k)
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default()
    };
    // deadlines
    let deadlines = arr("deadlines");
    if !deadlines.is_empty() {
        groups.push(CheckGroup {
            id: "group/deadlines".into(),
            title: "Deadlines".into(),
            items: vec![],
        });
    }
    for (n, d) in deadlines.iter().enumerate() {
        let (id, key) = item_id(&mut seen, "deadline", n + 1, d);
        let entity = text_of(&d["entity"]).unwrap_or_default();
        let form = text_of(&d["form"]).unwrap_or_default();
        let due = text_of(&d["due"]);
        let ext = text_of(&d["extended"]).filter(|e| parse_iso(e).is_some());
        let when = ext.clone().or(due.clone());
        let days = when.as_deref().and_then(|w| days_between(today, w));
        // `filed: true` (with `filed_on`) closes a deadline; `false` or absent leaves the days wording
        let filed = d["filed"].as_bool() == Some(true);
        let filed_on = text_of(&d["filed_on"]);
        let status = match days {
            _ if filed => match &filed_on {
                Some(on) => format!("filed {on}"),
                None => "filed".to_string(),
            },
            Some(0) => "due today".to_string(),
            Some(1) => "1 day left".to_string(),
            Some(n) if n > 1 => format!("{n} days left"),
            Some(-1) => "due 1 day ago".to_string(),
            Some(n) => format!("due {} days ago", -n),
            None => "no date".to_string(),
        };
        let t = match days {
            _ if filed => Tone::Done,
            Some(n) if n < 0 => Tone::Late,
            Some(n) if n <= 14 => Tone::Soon,
            Some(_) => Tone::Open,
            None => Tone::Unknown,
        };
        let mut detail = vec![];
        if let Some(d) = &due {
            detail.push(format!("due {d}"));
        }
        if let Some(e) = &ext {
            detail.push(format!("extended to {e}"));
        } else if let Some(e) = text_of(&d["extended"]) {
            detail.push(format!("extended: {e}"));
        }
        match &d["confirmed"] {
            Value::Bool(true) => detail.push("date confirmed".into()),
            Value::Bool(false) => detail.push("date not confirmed".into()),
            v => {
                if let Some(c) = text_of(v) {
                    detail.push(format!("confirmed: {c}"));
                }
            }
        }
        if let Some(note) = text_of(&d["note"]) {
            detail.push(note);
        }
        let text = [entity, form]
            .into_iter()
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join(" · ");
        push(
            &mut groups,
            "group/deadlines".into(),
            "Deadlines".into(),
            CheckItem {
                id,
                key: key.clone(),
                text: if text.is_empty() { key } else { text },
                status,
                tone: t,
                detail: (!detail.is_empty()).then(|| detail.join(" · ")),
                days_left: days,
            },
        );
    }
    // questions by owner, then the agent's own items
    // one group per owner, in the order owners first appear; the agent's work after the people's
    let owner_group = |owner: &str| -> (String, String) {
        match owner.to_lowercase().as_str() {
            "agent" => ("group/agent".into(), "Agent work".into()),
            "" => ("group/unassigned".into(), "No owner".into()),
            o => {
                let mut c = o.chars();
                let name = c
                    .next()
                    .map(|f| f.to_uppercase().collect::<String>() + c.as_str())
                    .unwrap_or_default();
                (format!("group/owner-{o}"), format!("Waiting on {name}"))
            }
        }
    };
    let mut owners: Vec<(String, String)> = vec![];
    for q in arr("questions").iter().chain(arr("agent_items").iter()) {
        let g = owner_group(&text_of(&q["owner"]).unwrap_or_default());
        if !owners.contains(&g) {
            owners.push(g);
        }
    }
    if !arr("agent_items").is_empty() && !owners.iter().any(|o| o.0 == "group/agent") {
        owners.push(owner_group("agent"));
    }
    owners.sort_by_key(|o| o.0 == "group/agent");
    for (id, title) in owners {
        groups.push(CheckGroup {
            id,
            title,
            items: vec![],
        });
    }
    for (section, list) in [
        ("question", arr("questions")),
        ("agent", arr("agent_items")),
    ] {
        for (n, q) in list.iter().enumerate() {
            let (id, key) = item_id(&mut seen, section, n + 1, q);
            let owner = text_of(&q["owner"]).unwrap_or_else(|| {
                if section == "agent" {
                    "agent".into()
                } else {
                    String::new()
                }
            });
            let (gid, title) = owner_group(&owner);
            let status = text_of(&q["status"]).unwrap_or_else(|| "(no status)".into());
            *counts.entry(status.clone()).or_default() += 1;
            let mut detail = vec![];
            if section == "agent" && owner.to_lowercase() != "agent" {
                detail.push(format!("owner {owner}"));
            }
            for k in ["note", "answer", "blocks"] {
                if let Some(t) = text_of(&q[k]) {
                    detail.push(format!("{k}: {t}"));
                }
            }
            push(
                &mut groups,
                gid,
                title,
                CheckItem {
                    id,
                    key: key.clone(),
                    text: text_of(&q["text"]).unwrap_or(key),
                    tone: tone(&status),
                    status,
                    detail: (!detail.is_empty()).then(|| detail.join(" · ")),
                    days_left: None,
                },
            );
        }
    }
    // postings by state: the known states in close order, then any other in the order first seen
    let postings = arr("postings");
    let mut states: Vec<String> = vec![];
    for k in ["posted", "prepared", "parked"] {
        if postings
            .iter()
            .any(|p| text_of(&p["state"]).as_deref() == Some(k))
        {
            states.push(k.into());
        }
    }
    for p in &postings {
        let s = text_of(&p["state"]).unwrap_or_else(|| "(no state)".into());
        if !states.contains(&s) {
            states.push(s);
        }
    }
    for s in &states {
        groups.push(CheckGroup {
            id: format!("group/postings-{s}"),
            title: format!("Postings: {s}"),
            items: vec![],
        });
    }
    for (n, p) in postings.iter().enumerate() {
        let (id, key) = item_id(&mut seen, "posting", n + 1, p);
        let state = text_of(&p["state"]).unwrap_or_else(|| "(no state)".into());
        *counts.entry(state.clone()).or_default() += 1;
        let mut detail = vec![];
        if let Some(e) = text_of(&p["entity"]) {
            detail.push(e);
        }
        let paths = p["paths"].as_array().map_or(0, Vec::len);
        if paths > 0 {
            detail.push(format!("{paths} file{}", if paths == 1 { "" } else { "s" }));
        }
        if let Some(v) = p["verify"].as_object()
            && !v.is_empty()
        {
            detail.push(
                v.get("status")
                    .or_else(|| v.get("result"))
                    .and_then(text_of)
                    .map_or_else(|| "verify: recorded".into(), |s| format!("verify: {s}")),
            );
        }
        push(
            &mut groups,
            format!("group/postings-{state}"),
            format!("Postings: {state}"),
            CheckItem {
                id,
                key: key.clone(),
                text: text_of(&p["text"]).unwrap_or(key),
                tone: tone(&state),
                status: state,
                detail: (!detail.is_empty()).then(|| detail.join(" · ")),
                days_left: None,
            },
        );
    }
    groups.retain(|g| !g.items.is_empty());
    let known = [
        "comment",
        "year",
        "deadlines",
        "questions",
        "agent_items",
        "postings",
    ];
    let also: Vec<String> = obj
        .keys()
        .filter(|k| !known.contains(&k.as_str()))
        .cloned()
        .collect();
    let year = text_of(obj.get("year").unwrap_or(&Value::Null));
    Ok(Checklist {
        title: match &year {
            Some(y) => format!("Close worklist {y}"),
            None => "Close worklist".into(),
        },
        file: file.into(),
        year,
        comment: text_of(obj.get("comment").unwrap_or(&Value::Null)),
        today: today.into(),
        groups,
        counts,
        also,
    })
}

// ── computing a spec over the books ──────────────────────────────────────────────────────────────

/// What `compute` reads besides the intercompany context: today (checklist), the project folder (relative
/// checklist paths), and the manifest effects for `before_after` (computed by the caller with
/// `manifest::effects::diff`, as the CHECK step does).
#[derive(Default)]
pub struct Extra<'a> {
    pub today: String,
    pub base_dir: Option<std::path::PathBuf>,
    pub effects: Option<&'a crate::manifest::effects::Diff>,
    /// `(alias, Xero name)` in config order (flow's nodes); empty: the context's aliases, sorted.
    pub orgs: Vec<(String, String)>,
}

/// The period a spec covers: `to` (default: the context's as-of) and `from` (default: the fiscal-year start
/// of `to` in `org`).
pub fn period(
    ctx: &IcContext,
    org: &str,
    from: Option<&str>,
    to: Option<&str>,
) -> Result<(String, String), VizError> {
    let to = to.unwrap_or(&ctx.as_of).to_string();
    let from = match from {
        Some(f) => f.to_string(),
        None => {
            let (m, d) = ctx.fy_end(org).map_err(VizError::record)?;
            fiscal_year_start(&to, m, d)
                .ok_or_else(|| VizError::usage(format!("not a date: {to}")))?
        }
    };
    Ok((from, to))
}

/// The org's ledger and its balance basis at `to` (Xero opening TB + activity when cached).
pub fn books(ctx: &IcContext, org: &str, to: &str) -> Result<(Arc<Ledger>, Basis), VizError> {
    let tid = ctx
        .tenants
        .get(org)
        .cloned()
        .ok_or_else(|| VizError::not_found(format!("Unknown org `{org}`.")))?;
    let l = ctx.ledger(org).map_err(VizError::record)?;
    let fy = ctx.fy_end(org).map_err(VizError::record)?;
    let b = crate::balances::resolve(ctx.view(), &l, &tid, to, fy, Mode::Auto)
        .map_err(VizError::record)?;
    Ok((l, b))
}

/// Compute a spec's model. The context's as-of is the default date; `ctx` decides main or a manifest.
pub fn compute(ctx: &IcContext, spec: &Spec, extra: &Extra) -> Result<Model, VizError> {
    spec.check()?;
    let orgs = if extra.orgs.is_empty() {
        let mut v: Vec<(String, String)> =
            ctx.tenants.keys().map(|a| (a.clone(), a.clone())).collect();
        v.sort();
        v
    } else {
        extra.orgs.clone()
    };
    match spec {
        Spec::Bridge { pair, as_of } => {
            let known: Vec<String> = ctx.pairs.iter().map(|p| sv(&p["id"])).collect();
            if !known.contains(pair) {
                return Err(VizError::not_found(format!(
                    "Unknown pair `{pair}`. Pairs: {}.",
                    known.join(", ")
                )));
            }
            let r = with_as_of(ctx, as_of.as_deref(), |c| {
                icm::build(c, Some(std::slice::from_ref(pair))).map_err(VizError::record)
            })?;
            let p = r["pairs"]
                .as_array()
                .and_then(|a| a.first())
                .ok_or_else(|| VizError::not_found(format!("pair `{pair}`")))?;
            Ok(Model::Bridge(bridge(p)))
        }
        Spec::Flow {
            orgs: want,
            as_of,
            from,
            to,
            basis,
        } => {
            let mut nodes = orgs.clone();
            if let Some(w) = want {
                for a in w {
                    if !nodes.iter().any(|n| &n.0 == a) {
                        return Err(VizError::not_found(format!("Unknown org `{a}`.")));
                    }
                }
                nodes.retain(|n| w.contains(&n.0));
            }
            match basis {
                FlowBasis::Owed => {
                    let r = with_as_of(ctx, as_of.as_deref().or(to.as_deref()), |c| {
                        icm::build(c, None).map_err(VizError::record)
                    })?;
                    Ok(Model::Flow(flow_owed(&r, &nodes)))
                }
                FlowBasis::Paid => {
                    let first = nodes.first().map(|n| n.0.clone()).unwrap_or_default();
                    let (f, t) = period(
                        ctx,
                        &first,
                        from.as_deref(),
                        to.as_deref().or(as_of.as_deref()),
                    )?;
                    Ok(Model::Flow(flow_paid(ctx, &nodes, &f, &t)?))
                }
            }
        }
        Spec::TAccount {
            org,
            account,
            from,
            to,
            top,
        } => {
            let (f, t) = period(ctx, org, from.as_deref(), to.as_deref())?;
            let (l, b) = books(ctx, org, &t)?;
            Ok(Model::TAccount(t_account(
                org,
                &l,
                &b,
                account,
                &f,
                &t,
                top.unwrap_or(TOP_DEFAULT),
            )?))
        }
        Spec::Timeline {
            org,
            account,
            from,
            to,
        } => {
            let (f, t) = period(ctx, org, from.as_deref(), to.as_deref())?;
            let (l, b) = books(ctx, org, &t)?;
            Ok(Model::Timeline(timeline(org, &l, &b, account, &f, &t)?))
        }
        Spec::BeforeAfter { org, .. } => {
            let d = extra.effects.ok_or_else(|| {
                VizError::not_found("No manifest to compare: show or name one first.")
            })?;
            Ok(Model::BeforeAfter(before_after(d, org)?))
        }
        Spec::Checklist { file } => {
            let path = match &extra.base_dir {
                Some(b) => b.join(file),
                None => std::path::PathBuf::from(file),
            };
            let text = std::fs::read_to_string(&path)
                .map_err(|e| VizError::not_found(format!("checklist {}: {e}", path.display())))?;
            let doc: Value = serde_json::from_str(&text)
                .map_err(|e| VizError::usage(format!("checklist {file}: not JSON: {e}")))?;
            Ok(Model::Checklist(checklist(&doc, file, &extra.today)?))
        }
    }
}

/// Run `f` on the context at another as-of date (a fresh context over the same view and ledgers).
fn with_as_of<T>(
    ctx: &IcContext,
    as_of: Option<&str>,
    f: impl FnOnce(&IcContext) -> Result<T, VizError>,
) -> Result<T, VizError> {
    match as_of {
        Some(d) if d != ctx.as_of => {
            let c = ctx.at(d);
            f(&c)
        }
        _ => f(ctx),
    }
}

// ── plain text (the CLI's text output; the TUI draws its own) ─────────────────────────────────────

fn amt(v: Decimal) -> String {
    fmt(v)
}

impl Model {
    /// The model as plain lines, amounts right-aligned in one column.
    pub fn text(&self) -> Vec<String> {
        let mut out = vec![self.title().to_string(), format!("  {}", self.lead())];
        let row = |l: &str, a: String| format!("  {l:<52} {a:>16}");
        match self {
            Model::Bridge(b) => {
                out.push(format!("  {}", b.a_says));
                out.push(format!("  {}", b.b_says));
                if let Some(r) = &b.reason {
                    out.push(format!("  not compared: {r}"));
                    return out;
                }
                out.push(row(&b.start.label, amt(b.start.amount)));
                for i in &b.items {
                    out.push(row(&format!("− {}", i.label), amt(i.amount)));
                    out.push(format!("      {}", i.detail));
                }
                out.push(row(&b.end.label, amt(b.end.amount)));
            }
            Model::Flow(f) => {
                for n in &f.nodes {
                    out.push(format!("  [{}] {}", n.org, n.name));
                }
                for e in &f.edges {
                    out.push(format!("  {} → {}  {}", e.from, e.to, e.words));
                    if let Some(n) = &e.note {
                        out.push(format!("      {n}"));
                    }
                }
            }
            Model::TAccount(t) => {
                out.push(format!("  {} to {}", t.from, t.to));
                out.push(row(
                    &format!("Opening ({})", t.opening_source),
                    money::dr_cr(t.opening),
                ));
                out.push(format!(
                    "  Debits ({}): {}",
                    t.debit_count,
                    amt(t.debit_total)
                ));
                for e in t.debits.iter().chain(t.debits_more.iter()) {
                    out.push(row(&format!("{} {}", e.date, e.label), amt(e.amount)));
                }
                out.push(format!(
                    "  Credits ({}): {}",
                    t.credit_count,
                    amt(t.credit_total)
                ));
                for e in t.credits.iter().chain(t.credits_more.iter()) {
                    out.push(row(&format!("{} {}", e.date, e.label), amt(e.amount)));
                }
                out.push(row("Closing", money::dr_cr(t.closing)));
            }
            Model::Timeline(t) => {
                out.push(row(&format!("Opening {}", t.from), money::dr_cr(t.opening)));
                for m in &t.months {
                    out.push(format!(
                        "  {}  {:>16}  {:>16}{}",
                        m.month,
                        money::dr_cr(m.balance),
                        amt(m.movement),
                        if m.journals > 0 {
                            format!("  journal ×{}", m.journals)
                        } else {
                            String::new()
                        }
                    ));
                }
            }
            Model::BeforeAfter(b) => {
                for l in b.lines.iter().chain(b.totals.iter()) {
                    out.push(format!(
                        "  {:<8} {:<36} {:>14} → {:>14}  Δ {:>12}",
                        l.code,
                        l.label,
                        amt(l.before),
                        amt(l.after),
                        amt(l.delta)
                    ));
                }
            }
            Model::Checklist(c) => {
                for g in &c.groups {
                    out.push(format!("  {}", g.title));
                    for i in &g.items {
                        out.push(format!("    [{}] {} · {}", i.status, i.key, i.text));
                    }
                }
                if !c.also.is_empty() {
                    out.push(format!("  also in the file: {}", c.also.join(", ")));
                }
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn spec_round_trip_and_errors() {
        let s = Spec::parse(r#"{"kind":"bridge","pair":"hold-design"}"#).unwrap();
        assert_eq!(s.to_json(), r#"{"kind":"bridge","pair":"hold-design"}"#);
        assert_eq!(Spec::parse(&s.to_json()).unwrap(), s);
        let t =
            Spec::parse(r#"{"kind":"t_account","org":"hold","account":"1470","top":3}"#).unwrap();
        assert_eq!(t.kind(), "t_account");
        assert_eq!(Spec::parse(r#"{"kind":"pie"}"#).unwrap_err().code, "usage");
        assert!(Spec::parse(r#"{"kind":"bridge","pair":"x","nope":1}"#).is_err());
        assert!(
            Spec::parse(r#"{"kind":"timeline","org":"a","account":"1","from":"2025-12-01","to":"2025-01-01"}"#)
                .is_err()
        );
        assert!(Spec::parse(r#"{"kind":"flow","basis":"paid"}"#).is_ok());
        assert!(Spec::parse(r#"{"kind":"flow","basis":"both"}"#).is_err());
    }

    #[test]
    fn checklist_keeps_every_item_and_unknown_statuses() {
        let doc = json!({
            "comment": "c", "year": 2025,
            "deadlines": [{"id": "d1", "entity": "hold", "form": "1120", "due": "2026-10-15",
                           "extended": null, "confirmed": true, "note": ""}],
            "questions": [
                {"id": "q1", "owner": "maren", "status": "open", "text": "a?"},
                {"id": "q2", "owner": "tobias", "status": "answered", "text": "k?"},
                {"id": "q3", "owner": "someone", "status": "mulling", "text": "s?"},
                {"owner": "maren", "status": "open", "text": "no id"}
            ],
            "agent_items": [{"id": "g1", "owner": "agent", "status": "doing", "text": "g"}],
            "postings": [
                {"id": "p1", "state": "posted", "entity": "hold", "text": "p", "paths": [], "verify": {}},
                {"id": "p2", "state": "drafted", "entity": "lab", "text": "q", "paths": ["a"], "verify": {}}
            ],
            "crypto": {}, "key_balances": {}
        });
        let c = checklist(&doc, "x.json", "2026-10-05").unwrap();
        let n: usize = c.groups.iter().map(|g| g.items.len()).sum();
        assert_eq!(n, 8);
        let titles: Vec<&str> = c.groups.iter().map(|g| g.title.as_str()).collect();
        assert_eq!(
            titles,
            [
                "Deadlines",
                "Waiting on Maren",
                "Waiting on Tobias",
                "Waiting on Someone",
                "Agent work",
                "Postings: posted",
                "Postings: drafted"
            ]
        );
        assert_eq!(c.groups[0].items[0].status, "10 days left");
        assert_eq!(c.groups[0].items[0].tone, Tone::Soon);
        let mulling = &c.groups[3].items[0];
        assert_eq!(
            (mulling.status.as_str(), mulling.tone),
            ("mulling", Tone::Unknown)
        );
        assert_eq!(c.also, ["crypto", "key_balances"]);
        assert_eq!(c.groups[1].items[1].id, "item/question-4");
    }

    #[test]
    fn timeline_month_ends() {
        assert_eq!(month_end(2025, 2), "2025-02-28");
        assert_eq!(month_end(2024, 2), "2024-02-29");
        assert_eq!(month_end(2025, 12), "2025-12-31");
    }
}
