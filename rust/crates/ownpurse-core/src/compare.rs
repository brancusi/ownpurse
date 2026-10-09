//! Side-by-side compare of 2–4 organisations' trial balances (and one aligned row's postings).
//!
//! Rows merge across orgs only on an identical account code (`align="code"`) or an explicit `account_map`
//! category (`align="map"`). Nothing else merges — no name matching — and nothing is netted, eliminated,
//! differenced or totalled across orgs: each org keeps its own columns and its own totals. Orgs with different
//! base currencies or fiscal years are refused. No network.
//!
//! Output is a `serde_json::Value` with a fixed key order. Decimals are emitted as their exact text, scale
//! preserved.

use crate::balances::{Basis, BasisKind};
use crate::ledger::{Account, Chart, Ledger, Posting};
use crate::money;
use rust_decimal::Decimal;
use serde_json::{Map, Value, json};
use std::collections::{BTreeSet, HashMap};

/// `0.00`: zero with two decimals (the scale shows in the output text).
pub const D0: Decimal = Decimal::from_parts(0, 0, 0, false, 2);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompareRefused(pub String);

impl std::fmt::Display for CompareRefused {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for CompareRefused {}

#[derive(Debug)]
pub enum CompareError {
    Align(String),
    Shape(String),
}

impl std::fmt::Display for CompareError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CompareError::Align(_) => f.write_str("align must be 'code' or 'map'"),
            CompareError::Shape(s) => write!(f, "compare result: {s}"),
        }
    }
}

impl std::error::Error for CompareError {}

/// A decimal's exact text.
pub fn dstr(d: Decimal) -> Value {
    Value::String(d.to_string())
}

fn opt_dstr(d: Option<Decimal>) -> Value {
    d.map_or(Value::Null, dstr)
}

/// Insertion-ordered map with dict semantics for repeated keys (first position, last value).
fn dict_keys(orgs: &[String]) -> Vec<String> {
    let mut seen = BTreeSet::new();
    orgs.iter()
        .filter(|o| seen.insert(o.as_str()))
        .cloned()
        .collect()
}

fn fy_of(o: &Value) -> (i64, i64) {
    let get = |k: &str, dflt: i64| match o.get(k) {
        Some(Value::Number(n)) => n.as_i64().filter(|v| *v != 0).unwrap_or(dflt),
        Some(Value::String(s)) if !s.is_empty() => s.parse().unwrap_or(dflt),
        _ => dflt,
    };
    (
        get("FinancialYearEndMonth", 12),
        get("FinancialYearEndDay", 31),
    )
}

/// `organisation(alias)` → the org's Organisation payload. Refuses naming the orgs.
pub fn check_compatible(
    orgs: &[String],
    organisation: impl Fn(&str) -> Value,
) -> Result<(), CompareRefused> {
    let keys = dict_keys(orgs);
    let cur: Vec<(String, String)> = keys
        .iter()
        .map(|o| {
            let c = organisation(o)
                .get("BaseCurrency")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string();
            (o.clone(), c)
        })
        .collect();
    let distinct: BTreeSet<&str> = cur
        .iter()
        .map(|(_, c)| c.as_str())
        .filter(|c| !c.is_empty())
        .collect();
    if distinct.len() > 1 {
        return Err(CompareRefused(format!(
            "Cannot compare different base currencies: {}",
            cur.iter()
                .map(|(o, c)| format!("{o} {}", if c.is_empty() { "?" } else { c }))
                .collect::<Vec<_>>()
                .join(", ")
        )));
    }
    let fy: Vec<(String, (i64, i64))> = keys
        .iter()
        .map(|o| (o.clone(), fy_of(&organisation(o))))
        .collect();
    if fy.iter().map(|(_, f)| f).collect::<BTreeSet<_>>().len() > 1 {
        return Err(CompareRefused(format!(
            "Cannot compare different fiscal years: {}",
            fy.iter()
                .map(|(o, (m, d))| format!("{o} ends {d}/{m}"))
                .collect::<Vec<_>>()
                .join(", ")
        )));
    }
    Ok(())
}

fn categories(account_map: &Value) -> Option<&Map<String, Value>> {
    account_map.get("categories").and_then(Value::as_object)
}

fn ref_str(r: &Value) -> String {
    crate::intercompany::py_str(r)
}

/// First `account_map.categories` entry naming this account (by code or {"name": …}) for this org, else the
/// first whose `default` matches `type:<TYPE>` or `class:<CLASS>`.
pub fn category(account: &Account, org: &str, account_map: &Value) -> Option<String> {
    let cats = categories(account_map)?;
    for (name, spec) in cats {
        let refs = spec
            .get(org)
            .and_then(Value::as_array)
            .map(Vec::as_slice)
            .unwrap_or(&[]);
        for r in refs {
            let hit = match r {
                Value::Object(o) => o.get("name").and_then(Value::as_str) == Some(&account.name),
                other => !account.code.is_empty() && ref_str(other) == account.code,
            };
            if hit {
                return Some(name.clone());
            }
        }
    }
    for (name, spec) in cats {
        let Some(d) = spec.get("default").and_then(Value::as_str) else {
            continue;
        };
        if d.strip_prefix("type:") == Some(account.kind.as_str())
            || d.strip_prefix("class:") == Some(account.class.as_str())
        {
            return Some(name.clone());
        }
    }
    None
}

/// One trial-balance row as `tb_compare` reads it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TbRow {
    pub account_id: String,
    pub balance: Decimal,
    /// `"!"` when the account differs from Xero's TB, else `""`.
    pub flag: String,
}

/// One org's column input: rows, total debits, total credits, and the basis they come from.
#[derive(Debug, Clone)]
pub struct TbColumn<'a> {
    pub rows: Vec<TbRow>,
    pub debit: Decimal,
    pub credit: Decimal,
    pub basis: &'a Basis,
}

/// Which front end's trial-balance rows to reproduce.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RowStyle {
    /// The TUI's trial balance rows: sorted by (class, code, name); accounts only Xero holds at `Decimal("0.00")`.
    Tui,
    /// `tb --compare`: rows unsorted (differing accounts first, then balances by account id); accounts only Xero holds at `Decimal("0")`.
    Cli,
}

/// Trial-balance rows on a basis, as the TUI (or CLI) feeds `tb_compare`.
pub fn tb_rows<'a>(ledger: &Ledger, basis: &'a Basis, style: RowStyle) -> TbColumn<'a> {
    let zero = if style == RowStyle::Tui {
        D0
    } else {
        Decimal::ZERO
    };
    let differing: BTreeSet<&str> = basis.diffs.iter().map(|d| d.account_id.as_str()).collect();
    let mut shown: Vec<(String, Decimal)> = vec![];
    let mut pos: HashMap<String, usize> = HashMap::new();
    for d in &basis.diffs {
        if !pos.contains_key(&d.account_id) {
            pos.insert(d.account_id.clone(), shown.len());
            shown.push((d.account_id.clone(), zero));
        }
    }
    for (k, v) in &basis.balances {
        match pos.get(k) {
            Some(&i) => shown[i].1 = *v,
            None => {
                pos.insert(k.clone(), shown.len());
                shown.push((k.clone(), *v));
            }
        }
    }
    if style == RowStyle::Tui {
        shown.sort_by_cached_key(|(aid, _)| {
            let a = ledger.chart.get(aid);
            (a.class, a.code, a.name)
        });
    }
    let mut dr = D0;
    let mut cr = D0;
    let rows = shown
        .into_iter()
        .map(|(aid, v)| {
            if v > Decimal::ZERO {
                dr = money::add(dr, v);
            } else if v < Decimal::ZERO {
                cr = money::add(cr, -v);
            }
            let flag = if differing.contains(aid.as_str()) {
                "!"
            } else {
                ""
            };
            TbRow {
                account_id: aid,
                balance: v,
                flag: flag.into(),
            }
        })
        .collect();
    TbColumn {
        rows,
        debit: dr,
        credit: cr,
        basis,
    }
}

const CLASS_ORDER: [&str; 5] = ["ASSET", "LIABILITY", "EQUITY", "REVENUE", "EXPENSE"];

fn class_order(c: &str) -> usize {
    CLASS_ORDER.iter().position(|x| *x == c).unwrap_or(9)
}

struct Entry {
    account_id: String,
    code: String,
    account: String,
    label: String,
    class: String,
    balance: Decimal,
    flag: String,
}

impl Entry {
    fn value(&self) -> Value {
        json!({"account_id": self.account_id, "code": self.code, "account": self.account,
               "label": self.label, "class": self.class, "balance": dstr(self.balance), "flag": self.flag})
    }
}

struct Cell {
    exists: bool,
    balance: Option<Decimal>,
    members: Vec<Entry>,
}

type OrgNames = (String, Vec<String>);

struct Group {
    align: &'static str,
    key_value: String,
    label: String,
    class: String,
    cells: Vec<(String, Cell)>,
    /// code alignment: (names differ, the name(s) each org uses for the code)
    names: Option<(bool, Vec<OrgNames>)>,
}

impl Group {
    fn code(&self) -> &str {
        if self.align == "code" {
            &self.key_value
        } else {
            ""
        }
    }
    fn cell_mut(&mut self, org: &str) -> Option<&mut Cell> {
        self.cells
            .iter_mut()
            .find(|(o, _)| o == org)
            .map(|(_, c)| c)
    }
}

/// `tbs[org]` as `tb_rows` returns; `charts[org]` the org's chart. Returns `{align, orgs, columns, aligned,
/// only}`; every amount stays per org.
pub fn tb_compare(
    orgs: &[String],
    tbs: &HashMap<String, TbColumn>,
    charts: &HashMap<String, &Chart>,
    align: &str,
    account_map: &Value,
) -> Result<Value, CompareError> {
    if align != "code" && align != "map" {
        return Err(CompareError::Align(align.into()));
    }
    let missing = |what: &str, o: &str| CompareError::Shape(format!("no {what} for org {o}"));
    let chart = |o: &str| charts.get(o).copied().ok_or_else(|| missing("chart", o));
    let tb = |o: &str| tbs.get(o).ok_or_else(|| missing("trial balance", o));
    let keys = dict_keys(orgs);
    let mut groups: Vec<Group> = vec![];
    let mut only: Vec<(String, Vec<Entry>)> = keys.iter().map(|o| (o.clone(), vec![])).collect();
    for org in orgs {
        for r in &tb(org)?.rows {
            let a = chart(org)?.get(&r.account_id);
            let key: Option<(&'static str, String)> = if align == "code" {
                let mut n = 0;
                for o in orgs {
                    if !a.code.is_empty() && chart(o)?.by_code.contains_key(&a.code) {
                        n += 1;
                    }
                }
                (!a.code.is_empty() && n > 1).then(|| ("code", a.code.clone()))
            } else {
                category(&a, org, account_map).map(|c| ("map", c))
            };
            let entry = Entry {
                account_id: a.account_id.clone(),
                code: a.code.clone(),
                account: a.name.clone(),
                label: a.label(),
                class: a.class.clone(),
                balance: r.balance,
                flag: r.flag.clone(),
            };
            let Some((kind, kv)) = key else {
                if let Some((_, list)) = only.iter_mut().find(|(o, _)| o == org) {
                    list.push(entry);
                }
                continue;
            };
            let gi = match groups
                .iter()
                .position(|g| g.align == kind && g.key_value == kv)
            {
                Some(i) => i,
                None => {
                    let label = if kind == "map" {
                        categories(account_map)
                            .and_then(|c| c.get(&kv))
                            .and_then(|s| s.get("label"))
                            .filter(|l| crate::intercompany::truthy(Some(l)))
                            .map(ref_str)
                            .unwrap_or_else(|| kv.clone())
                    } else {
                        a.name.clone()
                    };
                    groups.push(Group {
                        align: kind,
                        key_value: kv,
                        label,
                        class: a.class.clone(),
                        cells: vec![],
                        names: None,
                    });
                    groups.len() - 1
                }
            };
            let g = &mut groups[gi];
            if g.cell_mut(org).is_none() {
                g.cells.push((
                    org.clone(),
                    Cell {
                        exists: true,
                        balance: Some(D0),
                        members: vec![],
                    },
                ));
            }
            let cell = g.cell_mut(org).expect("cell just ensured");
            cell.balance = Some(money::add(cell.balance.unwrap_or(D0), r.balance));
            cell.members.push(entry);
        }
    }
    for g in &mut groups {
        for org in orgs {
            if g.cell_mut(org).is_none() {
                let exists = g.align == "code" && chart(org)?.by_code.contains_key(&g.key_value);
                g.cells.push((
                    org.clone(),
                    Cell {
                        exists,
                        balance: exists.then_some(D0),
                        members: vec![],
                    },
                ));
            }
        }
        if g.align == "code" {
            let mut names: Vec<String> = vec![];
            for o in orgs {
                if let Some(a) = chart(o)?.by_code(&g.key_value)
                    && !names.contains(&a.name)
                {
                    names.push(a.name.clone());
                }
            }
            g.label = names.join(" / ");
            let mut per = vec![];
            for o in &keys {
                per.push((
                    o.clone(),
                    chart(o)?
                        .by_code(&g.key_value)
                        .map(|a| vec![a.name.clone()])
                        .unwrap_or_default(),
                ));
            }
            g.names = Some((names.len() > 1, per));
        }
        let mut cells = std::mem::take(&mut g.cells);
        g.cells = keys
            .iter()
            .filter_map(|o| {
                cells
                    .iter()
                    .position(|(x, _)| x == o)
                    .map(|i| cells.swap_remove(i))
            })
            .collect();
    }
    groups.sort_by(|a, b| {
        (class_order(&a.class), a.code(), a.label.as_str()).cmp(&(
            class_order(&b.class),
            b.code(),
            b.label.as_str(),
        ))
    });
    for (_, list) in &mut only {
        list.sort_by(|a, b| {
            (class_order(&a.class), &a.code, &a.account).cmp(&(
                class_order(&b.class),
                &b.code,
                &b.account,
            ))
        });
    }
    let aligned: Vec<Value> = groups
        .iter()
        .map(|g| {
            let mut m = Map::new();
            m.insert("key".into(), format!("{}:{}", g.align, g.key_value).into());
            m.insert("align".into(), g.align.into());
            m.insert("code".into(), g.code().into());
            m.insert("label".into(), g.label.clone().into());
            m.insert("class".into(), g.class.clone().into());
            m.insert(
                "cells".into(),
                g.cells
                    .iter()
                    .map(|(o, c)| {
                        (
                            o.clone(),
                            json!({"exists": c.exists, "balance": opt_dstr(c.balance),
                                   "members": c.members.iter().map(Entry::value).collect::<Vec<_>>()}),
                        )
                    })
                    .collect::<Map<_, _>>()
                    .into(),
            );
            if let Some((differ, per)) = &g.names {
                m.insert("names_differ".into(), (*differ).into());
                m.insert(
                    "names".into(),
                    per.iter()
                        .map(|(o, n)| (o.clone(), json!(n)))
                        .collect::<Map<_, _>>()
                        .into(),
                );
            }
            Value::Object(m)
        })
        .collect();
    let mut columns = vec![];
    for org in orgs {
        let t = tb(org)?;
        columns.push(
            json!({"org": org, "basis": basis_state(t.basis), "debit": dstr(t.debit),
                            "credit": dstr(t.credit), "balanced": t.debit == t.credit}),
        );
    }
    let only_v: Map<String, Value> = only
        .iter()
        .map(|(o, list)| {
            (
                o.clone(),
                Value::Array(list.iter().map(Entry::value).collect()),
            )
        })
        .collect();
    Ok(
        json!({"align": align, "orgs": orgs, "columns": columns, "aligned": aligned, "only": only_v}),
    )
}

/// Per-column header facts: where balances come from and whether they match Xero.
pub fn basis_state(b: &Basis) -> Value {
    let (kind, src) = match b.kind {
        BasisKind::XeroOpening => (
            "xero-opening",
            format!(
                "TB {} + {}",
                b.opening_date.as_deref().unwrap_or("None"),
                b.as_of.chars().take(4).collect::<String>()
            ),
        ),
        BasisKind::History => ("history", "full history".to_string()),
    };
    let (state, level) = match b.matches_xero() {
        Some(true) => ("✓".to_string(), "ok"),
        Some(false) => (format!("differs: {}", b.diffs.len()), "warn"),
        None => ("not compared".to_string(), "warn"),
    };
    json!({"source": src, "state": state, "level": level, "kind": kind,
           "opening_date": b.opening_date, "matches_xero": b.matches_xero()})
}

fn posting_value(p: &Posting) -> Value {
    json!({"date": p.date, "kind": if p.kind.is_empty() { p.source } else { p.kind.as_str() },
           "contact": p.contact, "amount": dstr(p.amount), "source": p.source, "source_id": p.source_id,
           "account_id": p.account_id})
}

/// One date-ordered spine; each org's postings for its account(s) sit in its column on their date row.
/// Same-day postings stack. No pairing is inferred.
pub fn postings_compare(
    orgs: &[String],
    ledgers: &HashMap<String, &Ledger>,
    account_ids: &HashMap<String, Vec<String>>,
    since: &str,
    until: &str,
    openings: &HashMap<String, Decimal>,
) -> Result<Value, CompareError> {
    let empty: Vec<String> = vec![];
    let mut per: Vec<(String, Vec<&Posting>)> = vec![];
    for org in orgs {
        let ids: BTreeSet<&str> = account_ids
            .get(org)
            .unwrap_or(&empty)
            .iter()
            .map(String::as_str)
            .collect();
        let l = ledgers
            .get(org)
            .ok_or_else(|| CompareError::Shape(format!("no ledger for org {org}")))?;
        let mut ps: Vec<&Posting> = l
            .postings
            .iter()
            .filter(|p| {
                ids.contains(p.account_id.as_str())
                    && since <= p.date.as_str()
                    && p.date.as_str() <= until
            })
            .collect();
        ps.sort_by(|a, b| (&a.date, &a.source_id).cmp(&(&b.date, &b.source_id)));
        per.push((org.clone(), ps));
    }
    // dict semantics: a repeated org's later list replaces the earlier one
    let of = |o: &str| -> &Vec<&Posting> {
        &per.iter()
            .rev()
            .find(|(x, _)| x == o)
            .expect("every org listed")
            .1
    };
    let dates: BTreeSet<&str> = per
        .iter()
        .flat_map(|(_, ps)| ps.iter().map(|p| p.date.as_str()))
        .collect();
    let keys = dict_keys(orgs);
    let mut spine = vec![];
    for d in dates {
        let day: Vec<(String, Vec<&Posting>)> = keys
            .iter()
            .map(|o| {
                (
                    o.clone(),
                    of(o)
                        .iter()
                        .copied()
                        .filter(|p| p.date == d)
                        .collect::<Vec<_>>(),
                )
            })
            .collect();
        let n = day.iter().map(|(_, v)| v.len()).max().unwrap_or(0);
        for i in 0..n {
            let cells: Map<String, Value> = day
                .iter()
                .map(|(o, v)| {
                    (
                        o.clone(),
                        v.get(i).map_or(Value::Null, |p| posting_value(p)),
                    )
                })
                .collect();
            spine.push(json!({"date": if i == 0 { d } else { "" }, "cells": cells}));
        }
    }
    let mut cols = vec![];
    for org in orgs {
        let opening = openings.get(org).copied().unwrap_or(D0);
        let ps = of(org);
        let total = ps.iter().fold(D0, |acc, p| money::add(acc, p.amount));
        cols.push(json!({"org": org, "account_ids": account_ids.get(org).unwrap_or(&empty),
                         "opening": dstr(opening), "closing": dstr(money::add(opening, total)), "count": ps.len()}));
    }
    Ok(json!({"since": since, "until": until, "columns": cols, "spine": spine}))
}

fn dec_of(v: &Value) -> Result<Option<Decimal>, CompareError> {
    match v {
        Value::Null => Ok(None),
        Value::String(s) => money::parse(s)
            .map(Some)
            .map_err(|e| CompareError::Shape(e.to_string())),
        other => Err(CompareError::Shape(format!("not a decimal: {other}"))),
    }
}

fn sides(v: Option<Decimal>) -> Value {
    match v {
        None => json!({"exists": false, "debit": null, "credit": null, "net": null}),
        Some(v) => json!({"exists": true,
                          "debit": if v > Decimal::ZERO { dstr(v) } else { Value::Null },
                          "credit": if v < Decimal::ZERO { dstr(-v) } else { Value::Null },
                          "net": dstr(v)}),
    }
}

fn arr(v: &Value) -> &[Value] {
    v.as_array().map(Vec::as_slice).unwrap_or(&[])
}

/// Display rows, shared by the TUI and `tb --compare`: aligned rows first, then an `only in <org>` section per
/// org. Each row has per-org cells {exists, debit, credit, net}; exists=false renders as '—'.
pub fn flatten(result: &Value) -> Result<Vec<Value>, CompareError> {
    let orgs: Vec<String> = arr(&result["orgs"])
        .iter()
        .filter_map(|o| o.as_str().map(String::from))
        .collect();
    let keys = dict_keys(&orgs);
    let mut out = vec![];
    for g in arr(&result["aligned"]) {
        let cell = |o: &str| &g["cells"][o];
        let members: Map<String, Value> = keys
            .iter()
            .map(|o| {
                (
                    o.clone(),
                    arr(&cell(o)["members"])
                        .iter()
                        .map(|m| m["label"].clone())
                        .collect(),
                )
            })
            .collect();
        let member_ids: Map<String, Value> = keys
            .iter()
            .map(|o| {
                (
                    o.clone(),
                    arr(&cell(o)["members"])
                        .iter()
                        .map(|m| m["account_id"].clone())
                        .collect(),
                )
            })
            .collect();
        let names = match g.get("names") {
            Some(n) if crate::intercompany::truthy(Some(n)) => n.clone(),
            _ => Value::Object(
                keys.iter()
                    .map(|o| {
                        let mut seen: Vec<Value> = vec![];
                        for m in arr(&cell(o)["members"]) {
                            if !seen.contains(&m["account"]) {
                                seen.push(m["account"].clone());
                            }
                        }
                        (o.clone(), Value::Array(seen))
                    })
                    .collect(),
            ),
        };
        let mut cells = Map::new();
        for o in &keys {
            cells.insert(o.clone(), sides(dec_of(&cell(o)["balance"])?));
        }
        out.push(
            json!({"section": "aligned", "key": g["key"], "code": g["code"], "label": g["label"],
                        "class": g["class"], "members": members, "member_ids": member_ids,
                        "names_differ": crate::intercompany::truthy(g.get("names_differ")),
                        "names": names, "cells": cells}),
        );
    }
    for org in &keys {
        for e in arr(&result["only"][org.as_str()]) {
            let id = e["account_id"].as_str().unwrap_or("");
            let mut cells = Map::new();
            for o in &keys {
                cells.insert(
                    o.clone(),
                    if o == org {
                        sides(dec_of(&e["balance"])?)
                    } else {
                        sides(None)
                    },
                );
            }
            let member_ids: Map<String, Value> = keys
                .iter()
                .map(|o| (o.clone(), if o == org { json!([id]) } else { json!([]) }))
                .collect();
            out.push(
                json!({"section": format!("only in {org}"), "key": format!("{org}:{id}"),
                            "code": e["code"], "label": e["account"], "class": e["class"],
                            "members": {org.as_str(): [e["label"]]}, "member_ids": member_ids,
                            "account_id": e["account_id"], "org": org, "cells": cells}),
            );
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn refusals_name_the_orgs() {
        let orgs = |o: &str| match o {
            "a" => {
                json!({"BaseCurrency": "USD", "FinancialYearEndMonth": 12, "FinancialYearEndDay": 31})
            }
            "b" => {
                json!({"BaseCurrency": "EUR", "FinancialYearEndMonth": 12, "FinancialYearEndDay": 31})
            }
            _ => {
                json!({"BaseCurrency": "USD", "FinancialYearEndMonth": 6, "FinancialYearEndDay": 30})
            }
        };
        let s = |v: &[&str]| v.iter().map(|x| x.to_string()).collect::<Vec<_>>();
        let e = check_compatible(&s(&["a", "b"]), orgs).unwrap_err();
        assert!(e.0.contains("a USD, b EUR"), "{e}");
        let e = check_compatible(&s(&["a", "c"]), orgs).unwrap_err();
        assert!(e.0.contains("a ends 31/12, c ends 30/6"), "{e}");
        check_compatible(&s(&["a", "a"]), orgs).unwrap();
        check_compatible(&s(&["a", "x"]), |o| {
            if o == "a" {
                json!({})
            } else {
                json!({"BaseCurrency": "USD"})
            }
        })
        .unwrap();
    }

    #[test]
    fn d0_prints_two_decimals() {
        assert_eq!(D0.to_string(), "0.00");
        assert_eq!(
            money::add(D0, money::parse("-7").unwrap()).to_string(),
            "-7.00"
        );
    }

    #[test]
    fn bad_align_is_refused() {
        let r = tb_compare(&[], &HashMap::new(), &HashMap::new(), "name", &Value::Null);
        assert!(matches!(r, Err(CompareError::Align(_))));
    }
}
