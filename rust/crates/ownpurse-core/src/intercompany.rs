//! Intercompany pairs from config: both sides of every org pair, the difference, and a bridge.
//!
//! Balances come from cached Xero trial balances. When one side's account is shared by several parties (one
//! loan account holding two owners), the other parties' components are attributed from documents (contact,
//! journal narration, or a cross-org match) and removed. The bridge splits the difference into the prior-year
//! opening difference and this year's movement, then explains the movement with items:
//!
//! * `b_unexplained` — counterparty postings this year with no party contact whose counterpart is missing
//!   from the other org or recorded outside the paired accounts
//! * `match_both` — postings on either side this year with no matching (opposite, same amount) entry on the
//!   other side
//!
//! Anything the record cannot explain is reported as an unexplained residual, never filled in. No network:
//! cached reports and documents only. The output is a `serde_json::Value` with the same keys, key order and
//! string formats as the CLI's JSON (money as 2-dp strings, `None` → `null`).

use crate::balances::{Balances, XeroTbError, xero_tb_balances};
use crate::dates::{add_days, days_from_civil, fiscal_year_start, iso, parse_iso};
use crate::ledger::{Ledger, LedgerError, Posting};
use crate::money::{self, ZERO};
use crate::record::RecordError;
use regex::{Regex, RegexBuilder};
use rust_decimal::Decimal;
use serde_json::{Map, Value, json};
use std::cell::RefCell;
use std::cmp::Ordering;
use std::collections::{BTreeSet, HashMap};
use std::sync::Arc;

pub const STATUSES: [&str; 5] = [
    "unexplained",
    "not compared",
    "not supported",
    "bridged",
    "matches",
];
pub const SIGN_CONVENTION: &str =
    "net debit = positive; credits negative; amounts are decimal strings";
const BASIS: &str = "Xero TB balances (cached); party components and items from documents";

#[derive(Debug)]
pub enum IcError {
    Record(RecordError),
    Ledger(LedgerError),
    XeroTb(XeroTbError),
    Regex(regex::Error),
    Config(String),
}

impl std::fmt::Display for IcError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IcError::Record(e) => e.fmt(f),
            IcError::Ledger(e) => e.fmt(f),
            IcError::XeroTb(e) => e.fmt(f),
            IcError::Regex(e) => write!(f, "parties: bad regex: {e}"),
            IcError::Config(s) => write!(f, "intercompany config: {s}"),
        }
    }
}

impl std::error::Error for IcError {}

impl From<RecordError> for IcError {
    fn from(e: RecordError) -> Self {
        IcError::Record(e)
    }
}
impl From<LedgerError> for IcError {
    fn from(e: LedgerError) -> Self {
        IcError::Ledger(e)
    }
}
impl From<XeroTbError> for IcError {
    fn from(e: XeroTbError) -> Self {
        IcError::XeroTb(e)
    }
}

pub type Result<T> = std::result::Result<T, IcError>;

/// Money text: `None` → `null`, else 2 dp (banker's rounding).
pub fn money_str(v: Option<Decimal>) -> Value {
    v.map_or(Value::Null, |d| Value::String(money::money_str(d)))
}

fn ms(d: Decimal) -> Value {
    Value::String(money::money_str(d))
}

fn day_number(d: &str) -> Option<i64> {
    parse_iso(d).map(|(y, m, dd)| days_from_civil(y, m, dd))
}

/// `|d1 - d2| <= days` in calendar days. Unparseable dates never match.
pub fn near(d1: &str, d2: &str, days: Decimal) -> bool {
    match (day_number(d1), day_number(d2)) {
        (Some(a), Some(b)) => Decimal::from((a - b).abs()) <= days,
        _ => false,
    }
}

/// Truthiness of a JSON value (null, false, 0, "" and empty containers are false).
pub fn truthy(v: Option<&Value>) -> bool {
    match v {
        None | Some(Value::Null) => false,
        Some(Value::Bool(b)) => *b,
        Some(Value::Number(n)) => money::parse(&n.to_string()).is_ok_and(|d| !d.is_zero()),
        Some(Value::String(s)) => !s.is_empty(),
        Some(Value::Array(a)) => !a.is_empty(),
        Some(Value::Object(o)) => !o.is_empty(),
    }
}

fn py_repr_str(s: &str) -> String {
    let q = if s.contains('\'') && !s.contains('"') {
        '"'
    } else {
        '\''
    };
    let mut out = String::with_capacity(s.len() + 2);
    out.push(q);
    for ch in s.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c == q => {
                out.push('\\');
                out.push(c);
            }
            c => out.push(c),
        }
    }
    out.push(q);
    out
}

/// The repr of a JSON-decoded value (dicts, lists, strings, numbers, None, booleans).
pub fn py_repr(v: &Value) -> String {
    match v {
        Value::String(s) => py_repr_str(s),
        Value::Object(o) => format!(
            "{{{}}}",
            o.iter()
                .map(|(k, v)| format!("{}: {}", py_repr_str(k), py_repr(v)))
                .collect::<Vec<_>>()
                .join(", ")
        ),
        other => py_str(other),
    }
}

/// The text of a JSON-decoded value.
pub fn py_str(v: &Value) -> String {
    match v {
        Value::Null => "None".into(),
        Value::Bool(true) => "True".into(),
        Value::Bool(false) => "False".into(),
        Value::Number(n) => n.to_string(),
        Value::String(s) => s.clone(),
        Value::Array(a) => format!("[{}]", a.iter().map(py_repr).collect::<Vec<_>>().join(", ")),
        Value::Object(_) => py_repr(v),
    }
}

/// A decimal for a config value: strings and integers exactly; a JSON float (the config is read with
/// plain `json.loads`) becomes the exact binary value, so it equals a decimal amount only when that amount is
/// exactly representable as a double. `Ok(None)` = a float with no exact decimal equal to any posting amount.
pub fn py_decimal(v: &Value) -> Result<Option<Decimal>> {
    let bad = || IcError::Config(format!("not a decimal: {v}"));
    match v {
        Value::String(s) => money::parse(s).map(Some).map_err(|_| bad()),
        Value::Number(n) => {
            let text = n.to_string();
            let d = money::parse(&text).map_err(|_| bad())?;
            let is_float = text.contains(['.', 'e', 'E']);
            Ok(if !is_float || exact_double(d) {
                Some(d)
            } else {
                None
            })
        }
        _ => Err(bad()),
    }
}

/// Whether a decimal is exactly a binary64 value (mantissa ≤ 53 bits after removing the power of two).
fn exact_double(d: Decimal) -> bool {
    let d = d.normalize();
    let scale = d.scale();
    let mut n = d.mantissa().unsigned_abs();
    // d = n / 10^s = (n / 5^s) / 2^s: exact iff 5^s divides n and the odd part fits in 53 bits.
    for _ in 0..scale {
        if !n.is_multiple_of(5) {
            return false;
        }
        n /= 5;
    }
    while n != 0 && n.is_multiple_of(2) {
        n /= 2;
    }
    n < (1u128 << 53)
}

fn int_of(v: &Value) -> Result<i64> {
    let bad = || IcError::Config(format!("not an integer: {v}"));
    let text = match v {
        Value::Number(n) => n.to_string(),
        Value::String(s) => s.trim().to_string(),
        _ => return Err(bad()),
    };
    if let Ok(i) = text.parse::<i64>() {
        return Ok(i);
    }
    if matches!(v, Value::Number(_)) {
        let d = money::parse(&text).map_err(|_| bad())?;
        return i64::try_from(d.trunc()).map_err(|_| bad());
    }
    Err(bad())
}

fn days_of(v: Option<&Value>, default: i64) -> Result<Decimal> {
    match v {
        None => Ok(Decimal::from(default)),
        Some(Value::Number(n)) => money::parse(&n.to_string())
            .map_err(|_| IcError::Config(format!("not a number of days: {n}"))),
        Some(other) => Err(IcError::Config(format!("not a number of days: {other}"))),
    }
}

fn str_field<'v>(v: &'v Value, k: &str) -> Result<&'v str> {
    v.get(k)
        .and_then(Value::as_str)
        .ok_or_else(|| IcError::Config(format!("missing string `{k}` in {v}")))
}

fn list_field<'v>(v: &'v Value, k: &str) -> &'v [Value] {
    v.get(k)
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or(&[])
}

struct TbSnap {
    balances: Balances,
    fetched_at: String,
}

/// (org, as_of) → cached Xero TB (None = not cached).
type TbCache = HashMap<(String, String), Option<Arc<TbSnap>>>;

type LedgerFn<'r> = Box<dyn Fn(&str) -> Result<Arc<Ledger>> + 'r>;

/// Read-only view of the record for the engine: ledgers, cached TBs, account lookup, party rules.
pub struct IcContext<'r> {
    /// Main, or main + a branch (the bridges then show the branch's residuals).
    record: crate::view::View<'r>,
    /// alias → tenant id
    pub tenants: HashMap<String, String>,
    pub parties: Value,
    pub pairs: Vec<Value>,
    pub as_of: String,
    ledger_fn: Option<LedgerFn<'r>>,
    ledgers: RefCell<HashMap<String, Arc<Ledger>>>,
    tbs: RefCell<TbCache>,
    orgs: RefCell<HashMap<String, Arc<Value>>>,
    regexes: RefCell<HashMap<String, Arc<Regex>>>,
}

impl<'r> IcContext<'r> {
    /// `config` is `{"parties": {...}, "intercompany": {"pairs": [...]}}` (missing sections are empty).
    pub fn new(
        record: crate::view::View<'r>,
        tenants: HashMap<String, String>,
        config: &Value,
        as_of: &str,
    ) -> IcContext<'r> {
        let parties = config
            .get("parties")
            .filter(|v| truthy(Some(v)))
            .cloned()
            .unwrap_or_else(|| json!({}));
        let pairs = config
            .get("intercompany")
            .and_then(|i| i.get("pairs"))
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        IcContext {
            record,
            tenants,
            parties,
            pairs,
            as_of: as_of.into(),
            ledger_fn: None,
            ledgers: RefCell::new(HashMap::new()),
            tbs: RefCell::new(HashMap::new()),
            orgs: RefCell::new(HashMap::new()),
            regexes: RefCell::new(HashMap::new()),
        }
    }

    /// The view the context reads (main, or main + a manifest).
    pub fn view(&self) -> &crate::view::View<'r> {
        &self.record
    }

    /// The same view, config and ledgers at another as-of date (the ledgers are shared; TBs re-read).
    pub fn at(&self, as_of: &str) -> IcContext<'_> {
        let mut c = IcContext {
            record: self.record.clone(),
            tenants: self.tenants.clone(),
            parties: self.parties.clone(),
            pairs: self.pairs.clone(),
            as_of: as_of.into(),
            ledger_fn: None,
            ledgers: RefCell::new(HashMap::new()),
            tbs: RefCell::new(HashMap::new()),
            orgs: RefCell::new(HashMap::new()),
            regexes: RefCell::new(HashMap::new()),
        };
        c.ledger_fn = Some(Box::new(move |org: &str| self.ledger(org)));
        c
    }

    /// Use the caller's ledgers (e.g. the TUI's cache) instead of building them.
    pub fn with_ledgers(mut self, f: impl Fn(&str) -> Result<Arc<Ledger>> + 'r) -> Self {
        self.ledger_fn = Some(Box::new(f));
        self
    }

    fn tenant(&self, org: &str) -> Result<&str> {
        self.tenants
            .get(org)
            .map(String::as_str)
            .ok_or_else(|| IcError::Config(format!("unknown org {org}")))
    }

    pub fn ledger(&self, org: &str) -> Result<Arc<Ledger>> {
        if let Some(f) = &self.ledger_fn {
            return f(org);
        }
        if let Some(l) = self.ledgers.borrow().get(org) {
            return Ok(l.clone());
        }
        let l = Arc::new(Ledger::build(&self.record, self.tenant(org)?)?);
        self.ledgers.borrow_mut().insert(org.into(), l.clone());
        Ok(l)
    }

    pub fn organisation(&self, org: &str) -> Result<Arc<Value>> {
        if let Some(o) = self.orgs.borrow().get(org) {
            return Ok(o.clone());
        }
        let o = Arc::new(self.record.organisation(self.tenant(org)?)?);
        self.orgs.borrow_mut().insert(org.into(), o.clone());
        Ok(o)
    }

    pub fn fy_end(&self, org: &str) -> Result<(u32, u32)> {
        Ok(self.record.fy_end(self.tenant(org)?)?)
    }

    fn tb(&self, org: &str, as_of: &str) -> Result<Option<Arc<TbSnap>>> {
        let key = (org.to_string(), as_of.to_string());
        if let Some(t) = self.tbs.borrow().get(&key) {
            return Ok(t.clone());
        }
        let mut snap = match self.record.tb_snapshot(self.tenant(org)?, as_of)? {
            Some(r) => Some(TbSnap {
                balances: xero_tb_balances(&r.payload)?.0,
                fetched_at: r.fetched_at,
            }),
            None => None,
        };
        // on a branch, the Xero TB is main's: the branch's proposed effect up to as-of is added on top
        if self.record.overlay().is_some()
            && let Some(s) = snap.as_mut()
        {
            let l = self.ledger(org)?;
            let (m, d) = self.record.fy_end(self.tenant(org)?)?;
            let fy = crate::dates::fiscal_year_start(as_of, m, d).unwrap_or_default();
            for (acct, amt) in crate::balances::manifest_prior_delta(&l, as_of, as_of, &fy) {
                *s.balances.entry(acct).or_insert(ZERO) += amt;
            }
        }
        let snap = snap.map(Arc::new);
        self.tbs.borrow_mut().insert(key, snap.clone());
        Ok(snap)
    }

    /// Account id for a config ref: a code (any scalar, compared as text) or `{"name": …}`.
    pub fn account_id(&self, org: &str, r: &Value) -> Result<Option<String>> {
        let l = self.ledger(org)?;
        let chart = &l.chart;
        if let Value::Object(o) = r {
            let name = o.get("name");
            return Ok(chart
                .order
                .iter()
                .filter_map(|id| chart.by_id.get(id))
                .find(|a| name.and_then(Value::as_str) == Some(a.name.as_str()))
                .map(|a| a.account_id.clone()));
        }
        Ok(chart.by_code(&py_str(r)).map(|a| a.account_id.clone()))
    }

    pub fn account_ids(&self, org: &str, refs: &[Value]) -> Result<Vec<String>> {
        let mut out = vec![];
        for r in refs {
            if let Some(id) = self.account_id(org, r)? {
                out.push(id);
            }
        }
        Ok(out)
    }

    pub fn label(&self, org: &str, account_id: &str) -> Result<String> {
        Ok(self.ledger(org)?.chart.get(account_id).label())
    }

    pub fn balance(
        &self,
        org: &str,
        account_ids: &[String],
        as_of: &str,
    ) -> Result<Option<Decimal>> {
        Ok(self.tb(org, as_of)?.map(|t| {
            account_ids
                .iter()
                .map(|a| t.balances.get(a).copied().unwrap_or(ZERO))
                .sum()
        }))
    }

    fn regex(&self, rx: &str) -> Result<Arc<Regex>> {
        if let Some(r) = self.regexes.borrow().get(rx) {
            return Ok(r.clone());
        }
        let r = Arc::new(
            RegexBuilder::new(rx)
                .case_insensitive(true)
                .build()
                .map_err(IcError::Regex)?,
        );
        self.regexes.borrow_mut().insert(rx.into(), r.clone());
        Ok(r)
    }

    /// Case-insensitive regex search over the party's rules for `field` (contact / narration).
    pub fn party_of(&self, text: &str, party: &str, field: &str) -> Result<bool> {
        for rx in list_field(self.parties.get(party).unwrap_or(&Value::Null), field) {
            let rx = rx
                .as_str()
                .ok_or_else(|| IcError::Config(format!("parties.{party}.{field}: {rx}")))?;
            if self.regex(rx)?.is_match(text) {
                return Ok(true);
            }
        }
        Ok(false)
    }

    /// One posting with drill ids (org, account, source type and id) so any cell can cite it.
    pub fn posting_dict(&self, org: &str, p: &Posting) -> Result<Map<String, Value>> {
        let a = self.ledger(org)?.chart.get(&p.account_id);
        let v = json!({
            "org": org, "date": p.date, "account": a.label(), "account_id": p.account_id,
            "account_code": if a.code.is_empty() { Value::Null } else { Value::String(a.code.clone()) },
            "amount": ms(p.amount), "source": p.source, "kind": p.kind, "contact": p.contact,
            "reference": p.reference, "description": p.description, "source_id": p.source_id,
        });
        Ok(match v {
            Value::Object(m) => m,
            _ => unreachable!("json! object"),
        })
    }
}

/// Postings of a ledger on `ids` in [since, until] (ledger order).
fn select<'a>(
    l: &'a Ledger,
    ids: &[String],
    since: Option<&str>,
    until: Option<&str>,
) -> Vec<&'a Posting> {
    let ids: BTreeSet<&str> = ids.iter().map(String::as_str).collect();
    l.postings
        .iter()
        .filter(|p| {
            ids.contains(p.account_id.as_str())
                && since.is_none_or(|s| p.date.as_str() >= s)
                && until.is_none_or(|u| p.date.as_str() <= u)
        })
        .collect()
}

/// (posting, party, basis) for every b-side posting up to `until`.
type Attributed<'a> = (&'a Posting, Option<String>, Option<String>);

fn attribute<'a>(
    ctx: &IcContext,
    pair: &Value,
    b_ledger: &'a Ledger,
    b_ids: &[String],
    until: &str,
) -> Result<Vec<Attributed<'a>>> {
    let b = &pair["b"];
    let mut parties: Vec<String> = vec![];
    if truthy(b.get("party")) {
        parties.extend(b.get("party").and_then(Value::as_str).map(String::from));
    }
    for p in list_field(b, "other_parties") {
        if truthy(Some(p)) {
            parties.extend(p.as_str().map(String::from));
        }
    }
    let mut cross: Vec<(String, Arc<Ledger>, Vec<usize>, Decimal)> = vec![];
    for party in &parties {
        let Some(cm) = ctx.parties.get(party).and_then(|p| p.get("cross_match")) else {
            continue;
        };
        if !truthy(Some(cm)) {
            continue;
        }
        let org = str_field(cm, "org")?;
        if !ctx.tenants.contains_key(org) {
            continue;
        }
        let ids = ctx.account_ids(org, list_field(cm, "accounts"))?;
        let l = ctx.ledger(org)?;
        let idset: BTreeSet<&str> = ids.iter().map(String::as_str).collect();
        let cands = l
            .postings
            .iter()
            .enumerate()
            .filter(|(_, p)| idset.contains(p.account_id.as_str()) && p.date.as_str() <= until)
            .map(|(i, _)| i)
            .collect();
        let days = days_of(cm.get("days"), 31)?;
        // dict semantics: a repeated party keeps its first position, last value
        if let Some(e) = cross.iter_mut().find(|e| &e.0 == party) {
            *e = (party.clone(), l, cands, days);
        } else {
            cross.push((party.clone(), l, cands, days));
        }
    }
    let mut out = vec![];
    for p in select(b_ledger, b_ids, None, Some(until)) {
        let mut who: Option<String> = None;
        let mut basis: Option<String> = None;
        for party in &parties {
            if ctx.party_of(&p.contact, party, "contact")? {
                who = Some(party.clone());
                basis = Some("contact".into());
                break;
            }
        }
        if who.is_none() && p.source == "ManualJournals" {
            for party in &parties {
                if ctx.party_of(&p.reference, party, "narration")? {
                    who = Some(party.clone());
                    basis = Some("journal narration".into());
                    break;
                }
            }
        }
        if who.is_none() {
            for (party, l, cands, days) in &cross {
                let hit = cands
                    .iter()
                    .map(|&i| &l.postings[i])
                    .find(|c| c.amount.abs() == p.amount.abs() && near(&c.date, &p.date, *days));
                if let Some(hit) = hit {
                    who = Some(party.clone());
                    let what = if hit.contact.is_empty() {
                        hit.source
                    } else {
                        hit.contact.as_str()
                    };
                    basis = Some(format!("matches {party} {} {what}", hit.date));
                    break;
                }
            }
        }
        out.push((p, who, basis));
    }
    Ok(out)
}

/// Pair opposite postings of the same amount within `days`; return unmatched (a, b). When `pairs` is given,
/// the pairs made are appended to it as (a posting, b posting).
pub fn match_postings<'a>(
    a_list: &[&'a Posting],
    b_list: &[&'a Posting],
    days: i64,
    mut pairs: Option<&mut Vec<(&'a Posting, &'a Posting)>>,
) -> (Vec<&'a Posting>, Vec<&'a Posting>) {
    let days = Decimal::from(days);
    let mut b_left: Vec<&Posting> = b_list.to_vec();
    let mut a_un = vec![];
    for &a in a_list {
        let hit = b_left
            .iter()
            .position(|b| b.amount == -a.amount && near(&a.date, &b.date, days));
        match hit {
            Some(i) => {
                let b = b_left.remove(i);
                if let Some(ps) = pairs.as_deref_mut() {
                    ps.push((a, b));
                }
            }
            None => a_un.push(a),
        }
    }
    (a_un, b_left)
}

fn unsupported(ctx: &IcContext, a_org: &str, b_org: &str) -> Result<Option<String>> {
    let missing: Vec<&str> = [a_org, b_org]
        .into_iter()
        .filter(|o| !ctx.tenants.contains_key(*o))
        .collect();
    if !missing.is_empty() {
        return Ok(Some(format!("unknown org {}", missing.join(", "))));
    }
    let (oa, ob) = (ctx.organisation(a_org)?, ctx.organisation(b_org)?);
    let (ca, cb) = (oa.get("BaseCurrency"), ob.get("BaseCurrency"));
    if truthy(ca) && truthy(cb) && ca != cb {
        return Ok(Some(format!(
            "different base currencies ({a_org} {}, {b_org} {})",
            py_str(ca.unwrap_or(&Value::Null)),
            py_str(cb.unwrap_or(&Value::Null))
        )));
    }
    let (fa, fb) = (ctx.fy_end(a_org)?, ctx.fy_end(b_org)?);
    if fa != fb {
        return Ok(Some(format!(
            "different fiscal years ({a_org} ends {}/{}, {b_org} ends {}/{})",
            fa.1, fa.0, fb.1, fb.0
        )));
    }
    Ok(None)
}

fn set(rep: &mut Map<String, Value>, k: &str, v: impl Into<Value>) {
    rep.insert(k.into(), v.into());
}

fn missing_orgs(pairs: [(&str, Option<Decimal>); 2]) -> Vec<String> {
    pairs
        .iter()
        .filter(|(_, v)| v.is_none())
        .map(|(o, _)| o.to_string())
        .collect()
}

pub fn pair_report(ctx: &IcContext, pair: &Value) -> Result<Value> {
    let a_org = str_field(&pair["a"], "org")?;
    let b_org = str_field(&pair["b"], "org")?;
    let label = match pair.get("label") {
        Some(v) if truthy(Some(v)) => v.clone(),
        _ => Value::String(format!("{a_org} ↔ {b_org}")),
    };
    let party_v = pair["b"].get("party").cloned().unwrap_or(Value::Null);
    let mut rep = Map::new();
    set(
        &mut rep,
        "id",
        pair.get("id").cloned().unwrap_or(Value::Null),
    );
    set(&mut rep, "label", label);
    set(&mut rep, "a_org", a_org);
    set(&mut rep, "b_org", b_org);
    set(&mut rep, "party", party_v.clone());
    set(&mut rep, "as_of", ctx.as_of.as_str());
    set(&mut rep, "residual", Value::Null);
    set(&mut rep, "movement_items", json!([]));
    set(&mut rep, "matched", json!([]));
    set(&mut rep, "known_items", json!([]));
    set(&mut rep, "context", json!([]));
    if let Some(reason) = unsupported(ctx, a_org, b_org)? {
        set(&mut rep, "status", "not supported");
        set(&mut rep, "reason", reason);
        return Ok(Value::Object(rep));
    }
    let bad_date = |d: &str| IcError::Config(format!("not a date: {d}"));
    let (m, d) = ctx.fy_end(a_org)?;
    let fy_start = fiscal_year_start(&ctx.as_of, m, d).ok_or_else(|| bad_date(&ctx.as_of))?;
    let prior = add_days(&fy_start, -1).ok_or_else(|| bad_date(&fy_start))?;
    let a_ids = ctx.account_ids(a_org, list_field(&pair["a"], "accounts"))?;
    let b_ids = ctx.account_ids(b_org, list_field(&pair["b"], "accounts"))?;
    let days = match pair.get("match_days") {
        None => 7,
        Some(v) => int_of(v)?,
    };
    let la = ctx.ledger(a_org)?;
    let lb = ctx.ledger(b_org)?;
    let component_v = pair["b"]
        .get("component")
        .cloned()
        .unwrap_or_else(|| "residual".into());
    let movement_v = pair
        .get("movement")
        .cloned()
        .unwrap_or_else(|| "match_both".into());
    set(&mut rep, "prior", prior.as_str());
    set(&mut rep, "year_start", fy_start.as_str());
    set(&mut rep, "a_account_ids", a_ids.clone());
    set(&mut rep, "b_account_ids", b_ids.clone());
    set(
        &mut rep,
        "a_accounts",
        a_ids
            .iter()
            .map(|x| la.chart.get(x).label())
            .collect::<Vec<_>>(),
    );
    set(
        &mut rep,
        "b_accounts",
        b_ids
            .iter()
            .map(|x| lb.chart.get(x).label())
            .collect::<Vec<_>>(),
    );
    set(&mut rep, "component", component_v.clone());
    set(&mut rep, "movement", movement_v.clone());
    let mut fetched = Map::new();
    for o in [a_org, b_org] {
        fetched.insert(
            o.into(),
            ctx.tb(o, &ctx.as_of)?
                .map_or(Value::Null, |t| t.fetched_at.clone().into()),
        );
    }
    set(&mut rep, "tb_fetched", fetched);
    let a_now = ctx.balance(a_org, &a_ids, &ctx.as_of)?;
    let a_prev = ctx.balance(a_org, &a_ids, &prior)?;
    let b_now = ctx.balance(b_org, &b_ids, &ctx.as_of)?;
    let b_prev = ctx.balance(b_org, &b_ids, &prior)?;
    let (Some(a_now), Some(b_now)) = (a_now, b_now) else {
        let orgs = missing_orgs([(a_org, a_now), (b_org, b_now)]);
        set(&mut rep, "status", "not compared");
        set(
            &mut rep,
            "reason",
            format!("no cached Xero TB at {} for {}", ctx.as_of, orgs.join(", ")),
        );
        set(
            &mut rep,
            "missing_tb",
            json!({"date": ctx.as_of, "orgs": orgs}),
        );
        return Ok(Value::Object(rep));
    };
    let attributed = attribute(ctx, pair, &lb, &b_ids, &ctx.as_of)?;
    // other parties' components, in config order (dict semantics for repeats)
    let mut others: Vec<(String, Decimal, Decimal)> = vec![];
    for party in list_field(&pair["b"], "other_parties") {
        let Some(party) = party.as_str() else {
            continue;
        };
        let mine: Vec<&Posting> = attributed
            .iter()
            .filter(|(_, w, _)| w.as_deref() == Some(party))
            .map(|(p, _, _)| *p)
            .collect();
        let now: Decimal = mine.iter().map(|p| p.amount).sum();
        let prev: Decimal = mine
            .iter()
            .filter(|p| p.date <= prior)
            .map(|p| p.amount)
            .sum();
        if let Some(e) = others.iter_mut().find(|e| e.0 == party) {
            *e = (party.into(), now, prev);
        } else {
            others.push((party.into(), now, prev));
        }
    }
    let attributed_component = component_v.as_str() == Some("attributed");
    let party: Option<&str> = party_v.as_str();
    let mine: Vec<&Posting> = attributed
        .iter()
        .filter(|(_, w, _)| w.as_deref() == party)
        .map(|(p, _, _)| *p)
        .collect();
    let others_now: Decimal = others.iter().map(|o| o.1).sum();
    let b_party_now = if attributed_component {
        mine.iter().map(|p| p.amount).sum()
    } else {
        b_now - others_now
    };
    let diff = a_now + b_party_now;
    set(&mut rep, "a_net", ms(a_now));
    set(&mut rep, "b_net", ms(b_now));
    set(&mut rep, "b_party_net", ms(b_party_now));
    set(
        &mut rep,
        "other_components",
        others
            .iter()
            .map(|(k, now, _)| (k.clone(), ms(*now)))
            .collect::<Map<_, _>>(),
    );
    set(&mut rep, "raw_difference", ms(a_now + b_now));
    set(&mut rep, "difference", ms(diff));
    let (Some(a_prev), Some(b_prev)) = (a_prev, b_prev) else {
        let orgs = missing_orgs([(a_org, a_prev), (b_org, b_prev)]);
        set(&mut rep, "status", "not compared");
        set(
            &mut rep,
            "reason",
            format!(
                "no cached Xero TB at {prior} for {} (bridge needs the prior year-end)",
                orgs.join(", ")
            ),
        );
        set(&mut rep, "missing_tb", json!({"date": prior, "orgs": orgs}));
        return Ok(Value::Object(rep));
    };
    let b_party_prev = if attributed_component {
        mine.iter()
            .filter(|p| p.date <= prior)
            .map(|p| p.amount)
            .sum()
    } else {
        b_prev - others.iter().map(|o| o.2).sum::<Decimal>()
    };
    let opening = a_prev + b_party_prev;
    let movement = diff - opening;
    let mut items: Vec<Map<String, Value>> = vec![];
    let mut made: Vec<(&Posting, &Posting)> = vec![];
    if movement_v.as_str() == Some("b_unexplained") {
        let mut a_by_amt: HashMap<Decimal, Vec<&Posting>> = HashMap::new();
        for p in &la.postings {
            a_by_amt.entry(p.amount.abs()).or_default().push(p);
        }
        let a_set: BTreeSet<&str> = a_ids.iter().map(String::as_str).collect();
        for (p, who, _) in &attributed {
            if p.date < fy_start || who.is_some() {
                continue;
            }
            let hit = a_by_amt.get(&p.amount.abs()).and_then(|cs| {
                cs.iter()
                    .copied()
                    .find(|c| near(&c.date, &p.date, Decimal::from(days)))
            });
            if let Some(hit) = hit
                && a_set.contains(hit.account_id.as_str())
            {
                made.push((hit, p));
                continue;
            }
            let why = match hit {
                Some(hit) => format!(
                    "no party contact; {a_org} records it as {} {} ({}), outside the paired accounts",
                    la.chart.get(&hit.account_id).label(),
                    hit.date,
                    hit.source
                ),
                None => format!("no party contact; no matching entry in {a_org}"),
            };
            let mut item = Map::new();
            item.insert("side".into(), "b".into());
            item.extend(ctx.posting_dict(b_org, p)?);
            item.insert("why".into(), why.into());
            items.push(item);
        }
    } else {
        let exclusive = attributed_component || !others.is_empty();
        let b_year: Vec<&Posting> = attributed
            .iter()
            .filter(|(p, w, _)| {
                p.date >= fy_start
                    && if exclusive && party.is_some_and(|s| !s.is_empty()) {
                        w.as_deref() == party
                    } else {
                        !w.as_deref()
                            .is_some_and(|w| others.iter().any(|o| o.0 == w))
                    }
            })
            .map(|(p, _, _)| *p)
            .collect();
        let a_year = select(&la, &a_ids, Some(&fy_start), Some(&ctx.as_of));
        let (a_un, b_un) = match_postings(&a_year, &b_year, days, Some(&mut made));
        for (side, org, other, list) in [("a", a_org, b_org, &a_un), ("b", b_org, a_org, &b_un)] {
            for p in list {
                let mut item = Map::new();
                item.insert("side".into(), side.into());
                item.extend(ctx.posting_dict(org, p)?);
                item.insert("why".into(), format!("no matching entry in {other}").into());
                items.push(item);
            }
        }
    }
    let key = |i: &Map<String, Value>| {
        (
            i["date"].as_str().unwrap_or("").to_string(),
            i["side"].as_str().unwrap_or("").to_string(),
        )
    };
    items.sort_by_key(key);
    let mut explained = ZERO;
    for i in &items {
        explained += money::parse(i["amount"].as_str().unwrap_or("0"))
            .map_err(|e| IcError::Config(e.to_string()))?;
    }
    let mut known = vec![];
    for k in list_field(pair, "known_items") {
        let org = str_field(k, "org")?;
        let text = k.get("text").cloned().unwrap_or(Value::Null);
        if !ctx.tenants.contains_key(org) {
            known.push(json!({"text": text, "found": []}));
            continue;
        }
        let amount = py_decimal(k.get("amount").unwrap_or(&Value::Null))?;
        let contact = match k.get("contact") {
            None => String::new(),
            Some(v) => v
                .as_str()
                .ok_or_else(|| IcError::Config(format!("known_items contact: {v}")))?
                .to_lowercase(),
        };
        let before = match k.get("before") {
            None => ctx.as_of.clone(),
            Some(v) => v
                .as_str()
                .ok_or_else(|| IcError::Config(format!("known_items before: {v}")))?
                .to_string(),
        };
        let l = ctx.ledger(org)?;
        let mut found = vec![];
        for p in l.postings.iter().filter(|p| {
            amount.is_some_and(|a| p.amount.abs() == a)
                && p.contact.to_lowercase().contains(&contact)
                && p.date < before
                && l.chart.get(&p.account_id).kind != "BANK"
        }) {
            if found.len() == 3 {
                break;
            }
            found.push(Value::Object(ctx.posting_dict(org, p)?));
        }
        known.push(json!({"text": text, "found": found}));
    }
    let mut context = vec![];
    for c in list_field(pair, "context_accounts") {
        let org = str_field(c, "org")?;
        if !ctx.tenants.contains_key(org) {
            continue;
        }
        let acct = c.get("account").unwrap_or(&Value::Null);
        let ids = ctx.account_ids(org, std::slice::from_ref(acct))?;
        let label = match ids.first() {
            Some(id) => ctx.label(org, id)?,
            None => py_str(acct),
        };
        context.push(json!({"org": org, "account": label,
                            "balance": money_str(ctx.balance(org, &ids, &ctx.as_of)?)}));
    }
    let residual = movement - explained;
    let status = if diff == ZERO {
        "matches"
    } else if residual == ZERO {
        "bridged"
    } else {
        "unexplained"
    };
    let matched: Vec<Value> = made
        .iter()
        .map(|(a, b)| {
            json!({"a": {"org": a_org, "source": a.source, "source_id": a.source_id, "date": a.date,
                         "amount": ms(a.amount)},
                   "b": {"org": b_org, "source": b.source, "source_id": b.source_id, "date": b.date,
                         "amount": ms(b.amount)}})
        })
        .collect();
    set(&mut rep, "opening_difference", ms(opening));
    set(&mut rep, "movement_difference", ms(movement));
    set(
        &mut rep,
        "movement_items",
        items.into_iter().map(Value::Object).collect::<Vec<_>>(),
    );
    set(&mut rep, "movement_explained", ms(explained));
    set(&mut rep, "movement_residual", ms(residual));
    set(&mut rep, "residual", ms(residual));
    set(&mut rep, "known_items", known);
    set(&mut rep, "matched", matched);
    set(&mut rep, "context", context);
    set(&mut rep, "a_prev", ms(a_prev));
    set(&mut rep, "b_party_prev", ms(b_party_prev));
    set(&mut rep, "status", status);
    Ok(Value::Object(rep))
}

fn status_of(p: &Value) -> &str {
    p.get("status").and_then(Value::as_str).unwrap_or("")
}

pub fn summary(pairs: &[Value]) -> Value {
    let mut m = Map::new();
    m.insert("pairs".into(), pairs.len().into());
    for s in STATUSES {
        m.insert(
            s.into(),
            pairs.iter().filter(|p| status_of(p) == s).count().into(),
        );
    }
    Value::Object(m)
}

fn absdiff(p: &Value) -> Decimal {
    match p.get("difference").and_then(Value::as_str) {
        Some(s) => -money::parse(s).map(|d| d.abs()).unwrap_or(ZERO),
        None => ZERO,
    }
}

fn label_of(p: &Value) -> &str {
    p.get("label").and_then(Value::as_str).unwrap_or("")
}

/// The sort for `mode`, as a comparator: `status` (unexplained first, then by absolute difference, largest
/// first), `difference`, or `pair` (label, case-insensitive). Use with a stable sort.
pub fn sort_cmp(mode: &str) -> impl Fn(&Value, &Value) -> Ordering {
    let mode = mode.to_string();
    move |a, b| match mode.as_str() {
        "difference" => (absdiff(a), label_of(a)).cmp(&(absdiff(b), label_of(b))),
        "pair" => label_of(a).to_lowercase().cmp(&label_of(b).to_lowercase()),
        _ => {
            let idx = |p: &Value| STATUSES.iter().position(|s| *s == status_of(p));
            (idx(a), absdiff(a), label_of(a)).cmp(&(idx(b), absdiff(b), label_of(b)))
        }
    }
}

/// `all`: every pair · one org: that org on either side · `group:<name>`: both sides in the group.
pub fn in_scope(pair: &Value, members: &[String], scope: &str) -> bool {
    let org = |side: &str| pair[side]["org"].as_str().unwrap_or("");
    if scope == "all" {
        return true;
    }
    if scope.starts_with("group:") {
        return members.iter().any(|m| m == org("a")) && members.iter().any(|m| m == org("b"));
    }
    scope == org("a") || scope == org("b")
}

pub fn build(ctx: &IcContext, pair_ids: Option<&[String]>) -> Result<Value> {
    let mut reps = vec![];
    for p in &ctx.pairs {
        let chosen = match pair_ids {
            Some(ids) if !ids.is_empty() => p
                .get("id")
                .and_then(Value::as_str)
                .is_some_and(|id| ids.iter().any(|x| x == id)),
            _ => true,
        };
        if chosen {
            reps.push(pair_report(ctx, p)?);
        }
    }
    Ok(json!({"as_of": ctx.as_of, "basis": BASIS, "summary": summary(&reps), "pairs": reps}))
}

pub fn checks(result: &Value) -> Value {
    let pairs = result["pairs"].as_array().map(Vec::as_slice).unwrap_or(&[]);
    Value::Array(
        pairs
            .iter()
            .map(|p| {
                let st = status_of(p);
                let detail = match p.get("reason") {
                    Some(r) if truthy(Some(r)) => format!("{st}: {}", py_str(r)),
                    _ => st.to_string(),
                };
                json!({"name": p.get("id").cloned().unwrap_or(Value::Null),
                       "status": if st == "matches" || st == "bridged" { "ok" } else { "warn" },
                       "detail": detail})
            })
            .collect(),
    )
}

/// Same envelope as the project reports: source, checks, sign convention, data.
pub fn envelope(result: &Value, record: &str, generated_at: &str) -> Value {
    let pairs = result["pairs"].as_array().map(Vec::as_slice).unwrap_or(&[]);
    let mut fetched = Map::new();
    let mut orgs = BTreeSet::new();
    for p in pairs {
        if let Some(t) = p.get("tb_fetched").and_then(Value::as_object) {
            for (k, v) in t {
                fetched.insert(k.clone(), v.clone());
            }
        }
        for k in ["a_org", "b_org"] {
            if let Some(o) = p.get(k).and_then(Value::as_str) {
                orgs.insert(o.to_string());
            }
        }
    }
    json!({"report": "ic", "title": "Intercompany pairs", "generated_at": generated_at,
           "source": {"record": record, "xero_tb_fetched": fetched}, "as_of": result["as_of"],
           "basis": "accrual", "orgs": orgs.into_iter().collect::<Vec<_>>(),
           "sign_convention": SIGN_CONVENTION, "checks": checks(result), "data": result})
}

/// The last fiscal year-end before `today` (ISO date).
pub fn default_as_of(fy_end: (u32, u32), today: &str) -> Option<String> {
    let (y, _, _) = parse_iso(today)?;
    let (m, d) = fy_end;
    let this_year = iso(y, m, d);
    Some(if this_year.as_str() < today {
        this_year
    } else {
        iso(y - 1, m, d)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(amount: &str, date: &str) -> Posting {
        Posting {
            date: date.into(),
            source: "BankTransactions",
            source_id: format!("{amount}@{date}"),
            line: 0,
            account_id: "x".into(),
            amount: money::parse(amount).unwrap(),
            description: String::new(),
            contact: String::new(),
            reference: String::new(),
            kind: String::new(),
            proposed: None,
        }
    }

    #[test]
    fn match_pairs_opposite_amounts() {
        let a = [p("10", "2025-01-01"), p("5", "2025-02-01")];
        let b = [p("-10", "2025-01-03"), p("7", "2025-03-01")];
        let (ar, br): (Vec<&Posting>, Vec<&Posting>) = (a.iter().collect(), b.iter().collect());
        let mut made = vec![];
        let (a_un, b_un) = match_postings(&ar, &br, 7, Some(&mut made));
        assert_eq!(
            a_un.iter().map(|x| x.amount).collect::<Vec<_>>(),
            [Decimal::from(5)]
        );
        assert_eq!(
            b_un.iter().map(|x| x.amount).collect::<Vec<_>>(),
            [Decimal::from(7)]
        );
        assert_eq!(made.len(), 1);
    }

    #[test]
    fn scope_rules_and_sort() {
        let prs = [
            json!({"a": {"org": "own"}, "b": {"org": "co"}}),
            json!({"a": {"org": "x"}, "b": {"org": "co"}}),
        ];
        let m = |v: &[&str]| v.iter().map(|s| s.to_string()).collect::<Vec<_>>();
        assert_eq!(
            prs.iter()
                .map(|p| in_scope(p, &m(&["own", "co"]), "group:g"))
                .collect::<Vec<_>>(),
            [true, false]
        );
        assert_eq!(
            prs.iter()
                .map(|p| in_scope(p, &m(&["co"]), "co"))
                .collect::<Vec<_>>(),
            [true, true]
        );
        assert_eq!(
            prs.iter()
                .map(|p| in_scope(p, &m(&["own"]), "own"))
                .collect::<Vec<_>>(),
            [true, false]
        );
        assert!(in_scope(&prs[1], &[], "all"));
        let mut rows = vec![
            json!({"label": "b", "status": "bridged", "difference": "-50.00"}),
            json!({"label": "a", "status": "matches", "difference": "0.00"}),
            json!({"label": "c", "status": "bridged", "difference": "91.37"}),
            json!({"label": "d", "status": "unexplained", "difference": "1.00"}),
            json!({"label": "e", "status": "not compared", "difference": null}),
        ];
        rows.sort_by(sort_cmp("status"));
        let labels = |r: &[Value]| {
            r.iter()
                .map(|x| x["label"].as_str().unwrap().to_string())
                .collect::<Vec<_>>()
        };
        assert_eq!(labels(&rows), ["d", "e", "c", "b", "a"]);
        rows.sort_by(sort_cmp("pair"));
        assert_eq!(labels(&rows), ["a", "b", "c", "d", "e"]);
    }

    #[test]
    fn default_as_of_is_last_year_end_before_today() {
        assert_eq!(default_as_of((12, 31), "2026-10-05").unwrap(), "2025-12-31");
        assert_eq!(default_as_of((12, 31), "2026-12-31").unwrap(), "2025-12-31");
        assert_eq!(default_as_of((6, 30), "2026-10-05").unwrap(), "2026-06-30");
        assert_eq!(default_as_of((6, 30), "2026-06-30").unwrap(), "2025-06-30");
    }

    #[test]
    fn value_text_semantics() {
        assert_eq!(py_str(&json!({"name": "Nope"})), "{'name': 'Nope'}");
        assert_eq!(py_str(&json!("2405")), "2405");
        assert_eq!(py_str(&json!(2475)), "2475");
        assert_eq!(py_repr(&json!("it's")), "\"it's\"");
        assert!(truthy(Some(&json!("x"))) && !truthy(Some(&json!(""))) && !truthy(None));
        assert!(!truthy(Some(&json!(0))) && !truthy(Some(&json!(0.0))));
        let d = |s: &str| money::parse(s).unwrap();
        assert_eq!(py_decimal(&json!("1013.57")).unwrap(), Some(d("1013.57")));
        assert_eq!(py_decimal(&json!(500.0)).unwrap(), Some(d("500")));
        assert_eq!(py_decimal(&json!(12.5)).unwrap(), Some(d("12.5")));
        assert_eq!(py_decimal(&json!(9.99)).unwrap(), None);
        assert_eq!(py_decimal(&json!(1000)).unwrap(), Some(d("1000")));
        assert!(py_decimal(&json!(null)).is_err());
    }

    #[test]
    fn near_counts_calendar_days() {
        let d = Decimal::from;
        assert!(near("2025-01-01", "2025-01-08", d(7)));
        assert!(!near("2025-01-01", "2025-01-09", d(7)));
        assert!(near("2024-12-31", "2024-03-01", d(305)));
        assert!(!near("", "2025-01-01", d(10_000)));
    }
}
