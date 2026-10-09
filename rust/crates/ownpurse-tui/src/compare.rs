//! Compare (spec §9 Compare L1 / L2; acceptance H, I; PM P3 decisions): 2–4 orgs side by side, a mode of the
//! Ledger tab. Rows merge only on an identical code or an account_map category (core `tb_compare`); there is
//! no difference column and no cross-org total; each org keeps its own totals. L2 lays one row's postings on
//! one date spine (core `postings_compare`); `m` marks only what the Intercompany engine already matched.
//! Models are prepared in `update` and cached per (orgs, as-of, basis, align, record version).

use crate::app::{App, Effect, Overlay, Scope, Tab};
use crate::ic_nav::{Via, s};
use crate::keymap::Ctx;
use ownpurse_core::balances::Mode;
use ownpurse_core::compare::{self as cmpm, RowStyle};
use rust_decimal::Decimal;
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct CmpState {
    /// Org indexes, in column order.
    pub orgs: Vec<usize>,
    pub align: String,
    pub level: u8,
    /// L1 cursor: row key (kept as an id) and the selected column (0 = account, 1.. = org).
    pub row: Option<String>,
    pub row_idx: usize,
    pub top: usize,
    pub col: usize,
    /// L2: the row whose postings are shown, the spine cursor, range start, IC marks.
    pub l2_row: Option<String>,
    pub l2_idx: usize,
    pub l2_top: usize,
    pub l2_col: usize,
    pub since: Option<String>,
    pub show_matched: bool,
}

/// One prepared compare: the core result for the first `k` orgs (k = 2..=n) and its display rows.
#[derive(Clone, Debug)]
pub struct CmpModel {
    pub orgs: Vec<String>,
    pub result: Value,
    pub rows: Vec<Value>,
    pub columns: Vec<Value>,
}

#[derive(Clone, Debug, Default)]
pub struct CmpView {
    pub key: Option<(Vec<usize>, String, bool, String)>,
    /// by number of orgs shown (2..=n)
    pub models: HashMap<usize, Arc<Result<CmpModel, String>>>,
    pub l2: Option<Arc<Result<Value, String>>>,
    pub matched: HashSet<(String, String)>,
}

impl App {
    pub fn cmp_active(&self) -> bool {
        self.ui.tab == Tab::Ledger && self.ui.cmp.is_some() && self.ui.via.is_none()
    }

    pub fn cmp_ctx(&self) -> Ctx {
        if self.ui.cmp.as_ref().is_some_and(|c| c.level == 2) {
            Ctx::CompareL2
        } else {
            Ctx::CompareL1
        }
    }

    pub fn cmp_aliases(&self) -> Vec<String> {
        self.ui
            .cmp
            .as_ref()
            .map(|c| {
                c.orgs
                    .iter()
                    .map(|&i| self.model.orgs[i].alias.clone())
                    .collect()
            })
            .unwrap_or_default()
    }

    /// The model for `k` orgs shown (the layout picks k from the width).
    pub fn cmp_model(&self, k: usize) -> Option<&CmpModel> {
        self.cmp_view
            .models
            .get(&k)
            .and_then(|m| m.as_ref().as_ref().ok())
    }

    pub fn cmp_error(&self) -> Option<String> {
        let n = self.ui.cmp.as_ref()?.orgs.len();
        self.cmp_view
            .models
            .get(&n)
            .and_then(|m| m.as_ref().as_ref().err().cloned())
    }

    /// Display rows of the full model after the `/` filter (indexes into `rows`).
    pub fn cmp_visible(&self, m: &CmpModel) -> Vec<usize> {
        let f = crate::filter::Filter::parse(&self.ui.filter);
        (0..m.rows.len())
            .filter(|&i| {
                let code = s(&m.rows[i]["code"]);
                f.matches(
                    &format!("{code}\n{}", s(&m.rows[i]["label"])),
                    Some(code.as_str()).filter(|c| !c.is_empty()),
                )
            })
            .collect()
    }

    /// Orgs shown at the current width (the layout: debit/credit → net → fewer orgs).
    pub fn cmp_k(&self) -> usize {
        let Some(c) = self.ui.cmp.as_ref() else {
            return 0;
        };
        let n = c.orgs.len();
        match self.cmp_model(n) {
            Some(m) => {
                crate::views::compare::layout(
                    n,
                    self.ui.size.0,
                    crate::views::compare::amount_width(m),
                )
                .0
            }
            None => n,
        }
    }

    /// The model the view shows (cursors are resolved in it).
    pub(crate) fn cmp_full_pub(&self) -> Option<&CmpModel> {
        self.cmp_full()
    }

    fn cmp_full(&self) -> Option<&CmpModel> {
        self.cmp_model(self.cmp_k())
            .or_else(|| self.cmp_model(self.ui.cmp.as_ref()?.orgs.len()))
    }

    pub fn cmp_cursor(&self) -> Option<usize> {
        let m = self.cmp_full()?;
        let vis = self.cmp_visible(m);
        if vis.is_empty() {
            return None;
        }
        let c = self.ui.cmp.as_ref()?;
        c.row
            .as_ref()
            .and_then(|k| vis.iter().position(|&i| s(&m.rows[i]["key"]) == *k))
            .or(Some(c.row_idx.min(vis.len() - 1)))
    }

    pub fn cmp_selected_row(&self) -> Option<&Value> {
        let m = self.cmp_full()?;
        let c = self.cmp_cursor()?;
        Some(&m.rows[self.cmp_visible(m)[c]])
    }

    pub fn cmp_l2_row(&self) -> Option<&Value> {
        let key = self.ui.cmp.as_ref()?.l2_row.clone()?;
        self.cmp_full()?.rows.iter().find(|r| s(&r["key"]) == key)
    }

    pub fn cmp_l2(&self) -> Option<&Value> {
        self.cmp_view
            .l2
            .as_ref()
            .and_then(|v| v.as_ref().as_ref().ok())
    }

    pub fn cmp_has_row(&self) -> bool {
        match self.ui.cmp.as_ref().map(|c| c.level) {
            Some(2) => self
                .cmp_l2()
                .is_some_and(|v| v["spine"].as_array().is_some_and(|a| !a.is_empty())),
            Some(_) => self.cmp_cursor().is_some(),
            None => false,
        }
    }

    /// `v` can open Compare: an org with a record, or all / a group (2–4 members open directly, else picker).
    pub fn can_compare(&self) -> bool {
        self.ui.via.is_none()
            && (self.ledger_org().is_some() || !matches!(self.ui.scope, Scope::Org(_)))
    }

    /// `v`: under all or a group of 2–4 orgs open directly; otherwise the multi-select picker.
    pub(crate) fn compare_open(&mut self) {
        if self.ui.via.is_some() || self.ui.cmp.is_some() && self.ui.tab == Tab::Ledger {
            return;
        }
        let members = self.members(&self.ui.scope);
        if !matches!(self.ui.scope, Scope::Org(_)) && (2..=4).contains(&members.len()) {
            self.compare_start(members);
            return;
        }
        let chosen = match self.ui.scope {
            Scope::Org(i) => vec![i],
            _ => vec![],
        };
        self.ui.overlay = Some(Overlay::Picker {
            query: String::new(),
            sel: 0,
            multi: true,
            chosen,
        });
    }

    /// Open Compare on these orgs: refused (persistently, naming the orgs) on a currency or fiscal-year
    /// mismatch or a missing record. From Ledger L2 it opens the postings of the row holding that account.
    pub fn compare_start(&mut self, orgs: Vec<usize>) {
        let aliases: Vec<String> = orgs
            .iter()
            .map(|&i| self.model.orgs[i].alias.clone())
            .collect();
        if let Some(books) = &self.books {
            let rec = books.record();
            let tenants: HashMap<String, String> = orgs
                .iter()
                .map(|&i| {
                    (
                        self.model.orgs[i].alias.clone(),
                        self.model.orgs[i].tenant.clone(),
                    )
                })
                .collect();
            if let Err(e) = cmpm::check_compatible(&aliases, |o| {
                tenants
                    .get(o)
                    .and_then(|t| rec.organisation(t).ok())
                    .unwrap_or(Value::Null)
            }) {
                self.refuse(format!("{}. The compare view does not open.", e.0));
                return;
            }
        }
        let missing: Vec<&str> = orgs
            .iter()
            .filter(|&&i| self.model.orgs[i].last_sync.is_none())
            .map(|&i| self.model.orgs[i].alias.as_str())
            .collect();
        if !missing.is_empty() {
            let t = format!(
                "No local record for {} yet. ⌃P → Sync first.",
                missing.join(", ")
            );
            self.refuse(t);
            return;
        }
        // the Ledger L2 account (if `v` was pressed there)
        let origin = match (self.ledger_org(), self.ledger_level()) {
            (Some(o), 2) => self
                .nav()
                .account
                .map(|a| (self.model.orgs[o].alias.clone(), a)),
            _ => None,
        };
        self.ui.cmp = Some(CmpState {
            orgs,
            align: self.source.compare_align.clone(),
            level: 1,
            row: None,
            row_idx: 0,
            top: 0,
            col: 0,
            l2_row: None,
            l2_idx: 0,
            l2_top: 0,
            l2_col: 1,
            since: None,
            show_matched: false,
        });
        self.ui.tab = Tab::Ledger;
        self.prepare_compare();
        if let Some((org, account)) = origin {
            let key = self.cmp_full().and_then(|m| {
                m.rows
                    .iter()
                    .find(|r| {
                        r["section"] == "aligned"
                            && r["member_ids"][org.as_str()]
                                .as_array()
                                .is_some_and(|a| a.iter().any(|x| s(x) == account))
                    })
                    .map(|r| s(&r["key"]))
            });
            if let (Some(k), Some(c)) = (key, self.ui.cmp.as_mut()) {
                c.row = Some(k.clone());
                c.l2_row = Some(k);
                c.level = 2;
            }
        }
    }

    fn cmp_page(&self) -> usize {
        self.hits
            .content
            .map_or(10, |r| (r.height as usize).saturating_sub(4).max(1))
    }

    fn cmp_select(&mut self, pos: usize) {
        let Some(m) = self.cmp_full() else { return };
        let vis = self.cmp_visible(m);
        if vis.is_empty() {
            return;
        }
        let pos = pos.min(vis.len() - 1);
        let key = s(&m.rows[vis[pos]]["key"]);
        let page = self.cmp_page();
        if let Some(c) = self.ui.cmp.as_mut() {
            c.row = Some(key);
            c.row_idx = pos;
            if pos < c.top {
                c.top = pos;
            } else if pos >= c.top + page {
                c.top = pos + 1 - page;
            }
        }
    }

    fn cmp_spine_len(&self) -> usize {
        self.cmp_l2()
            .and_then(|v| v["spine"].as_array().map(Vec::len))
            .unwrap_or(0)
    }

    fn cmp_select_l2(&mut self, pos: usize) {
        let n = self.cmp_spine_len();
        if n == 0 {
            return;
        }
        let pos = pos.min(n - 1);
        let page = self.cmp_page();
        if let Some(c) = self.ui.cmp.as_mut() {
            c.l2_idx = pos;
            if pos < c.l2_top {
                c.l2_top = pos;
            } else if pos >= c.l2_top + page {
                c.l2_top = pos + 1 - page;
            }
        }
    }

    /// A click on a row or cell (draw registers absolute positions): select it, and the column.
    pub(crate) fn cmp_click(&mut self, i: usize, col: Option<usize>) {
        let Some(c) = self.ui.cmp.clone() else { return };
        if c.level == 2 {
            self.cmp_select_l2(i);
        } else {
            self.cmp_select(i);
        }
        if let (Some(col), Some(c)) = (col, self.ui.cmp.as_mut()) {
            if c.level == 2 {
                c.l2_col = col.max(1);
            } else {
                c.col = col;
            }
        }
    }

    /// The (org index, account ids) under the L1 cursor when an org column is selected.
    fn cmp_cell_target(&self) -> Option<(usize, Vec<String>)> {
        let c = self.ui.cmp.as_ref()?;
        if c.col == 0 {
            return None;
        }
        let org = *c.orgs.get(c.col - 1)?;
        let row = self.cmp_selected_row()?;
        let ids: Vec<String> = row["member_ids"][self.model.orgs[org].alias.as_str()]
            .as_array()
            .map(|a| a.iter().map(s).collect())
            .unwrap_or_default();
        (!ids.is_empty()).then_some((org, ids))
    }

    fn via_compare(&mut self, org: usize, ids: Vec<String>, doc: Option<(String, String)>) {
        let entry = if doc.is_some() {
            3
        } else if ids.len() == 1 {
            2
        } else {
            1
        };
        self.ui.via = Some(Via {
            pair: "compare".into(),
            label: "compare".into(),
            org,
            only: ids.clone(),
            entry_level: entry,
            from_compare: true,
        });
        let nav = self.nav_mut_pub();
        *nav = Default::default();
        nav.level = entry;
        if entry >= 2 {
            nav.account = ids.first().cloned();
        }
        nav.doc = doc;
    }

    /// Compare actions; false lets the Ledger / global handlers run.
    pub(crate) fn compare_action(&mut self, action: &str, _fx: &mut Vec<Effect>) -> bool {
        let Some(c) = self.ui.cmp.clone() else {
            return false;
        };
        let n_orgs = c.orgs.len();
        match action {
            "row.prev" | "row.next" | "page.prev" | "page.next" | "row.first" | "row.last" => {
                let (n, cur) = if c.level == 2 {
                    (self.cmp_spine_len(), c.l2_idx)
                } else {
                    (
                        self.cmp_full().map_or(0, |m| self.cmp_visible(m).len()),
                        self.cmp_cursor().unwrap_or(0),
                    )
                };
                let page = self.cmp_page();
                let pos = match action {
                    "row.prev" => cur.saturating_sub(1),
                    "row.next" => cur + 1,
                    "page.prev" => cur.saturating_sub(page),
                    "page.next" => cur + page,
                    "row.first" => 0,
                    _ => n.saturating_sub(1),
                };
                if c.level == 2 {
                    self.cmp_select_l2(pos)
                } else {
                    self.cmp_select(pos)
                }
                true
            }
            "cell.prev" | "cell.next" => {
                if let Some(c) = self.ui.cmp.as_mut() {
                    let (col, lo) = if c.level == 2 {
                        (&mut c.l2_col, 1)
                    } else {
                        (&mut c.col, 0)
                    };
                    *col = if action == "cell.next" {
                        (*col + 1).min(n_orgs)
                    } else {
                        col.saturating_sub(1).max(lo)
                    };
                }
                true
            }
            "drill.in" if c.level == 1 => {
                if let Some((org, ids)) = self.cmp_cell_target() {
                    // an amount: that org's ledger (via compare; Esc returns here)
                    self.via_compare(org, ids, None);
                } else if let Some(row) = self.cmp_selected_row() {
                    let key = s(&row["key"]);
                    if let Some(c) = self.ui.cmp.as_mut() {
                        if c.l2_row.as_deref() != Some(key.as_str()) {
                            c.l2_idx = 0;
                            c.l2_top = 0;
                        }
                        c.row = Some(key.clone());
                        c.l2_row = Some(key);
                        c.level = 2;
                    }
                }
                true
            }
            "drill.in" => {
                // a posting: that org's document
                let Some(org) = c.orgs.get(c.l2_col.saturating_sub(1)).copied() else {
                    return true;
                };
                let alias = self.model.orgs[org].alias.clone();
                let cell = self
                    .cmp_l2()
                    .and_then(|v| v["spine"].as_array().and_then(|a| a.get(c.l2_idx)).cloned())
                    .map(|l| l["cells"][alias.as_str()].clone());
                match cell {
                    Some(p) if !p.is_null() => {
                        let ids = vec![s(&p["account_id"])];
                        self.via_compare(org, ids, Some((s(&p["source"]), s(&p["source_id"]))));
                    }
                    _ => {
                        self.ui.note = Some(format!(
                            "no {alias} posting on this row (←/→ picks the org)"
                        ))
                    }
                }
                true
            }
            "drill.back" => {
                if let Some(c) = self.ui.cmp.as_mut() {
                    if c.level == 2 {
                        c.level = 1;
                    } else {
                        self.ui.cmp = None; // back to the Ledger where `v` was pressed
                    }
                }
                true
            }
            "compare.align" => {
                if let Some(c) = self.ui.cmp.as_mut() {
                    c.align = if c.align == "code" {
                        "map".into()
                    } else {
                        "code".into()
                    };
                    c.row = None;
                    c.row_idx = 0;
                    c.top = 0;
                }
                true
            }
            "compare.col_next" | "compare.col_prev" | "compare.col_drop" => {
                if c.col == 0 {
                    self.ui.note = Some("pick an org column first (←/→)".into());
                    return true;
                }
                let i = c.col - 1;
                if let Some(c) = self.ui.cmp.as_mut() {
                    match action {
                        "compare.col_next" if i + 1 < c.orgs.len() => {
                            c.orgs.swap(i, i + 1);
                            c.col += 1;
                        }
                        "compare.col_prev" if i > 0 => {
                            c.orgs.swap(i, i - 1);
                            c.col -= 1;
                        }
                        "compare.col_drop" if c.orgs.len() > 2 => {
                            c.orgs.remove(i);
                            c.col = c.col.min(c.orgs.len());
                        }
                        "compare.col_drop" => {
                            self.ui.note = Some("Compare needs at least 2 orgs".into())
                        }
                        _ => {}
                    }
                }
                true
            }
            "compare.ic_marks" => {
                if let Some(c) = self.ui.cmp.as_mut() {
                    c.show_matched = !c.show_matched;
                }
                true
            }
            "range.edit" if c.level == 2 => {
                let since = self.cmp_l2().map(|v| s(&v["since"])).unwrap_or_default();
                self.ui.prompt = Some(crate::app::Prompt::Range(c.since.clone().unwrap_or(since)));
                true
            }
            "basis.toggle" => {
                self.ui.full_history = !self.ui.full_history;
                true
            }
            _ => false,
        }
    }

    /// The range prompt while Compare L2 is open (sets the range for every column).
    pub(crate) fn compare_range(&mut self, action: &str, text: &str) -> bool {
        if !self.cmp_active() || self.ui.cmp.as_ref().is_none_or(|c| c.level != 2) {
            return false;
        }
        match action {
            "prompt.apply" => {
                let t = text.trim().to_string();
                if t.len() == 10
                    && ownpurse_core::dates::parse_iso(&t).is_some()
                    && t <= self.ui.as_of
                {
                    if let Some(c) = self.ui.cmp.as_mut() {
                        c.since = Some(t);
                    }
                    self.ui.prompt = None;
                } else {
                    self.ui.note = Some(format!(
                        "range needs a first day: YYYY-MM-DD, up to {}",
                        self.ui.as_of
                    ));
                }
                true
            }
            "prompt.cancel" => {
                if let Some(c) = self.ui.cmp.as_mut() {
                    c.since = None;
                }
                self.ui.prompt = None;
                true
            }
            _ => false,
        }
    }

    /// Build (or look up) the compare models for every k = 2..=n and the L2 spine.
    pub(crate) fn prepare_compare(&mut self) {
        let Some(c) = self.ui.cmp.clone() else {
            self.cmp_view = CmpView::default();
            return;
        };
        if self.ui.tab != Tab::Ledger {
            return;
        }
        let history = self.ui.full_history;
        let key = (
            c.orgs.clone(),
            self.ui.as_of.clone(),
            history,
            c.align.clone(),
        );
        if self.cmp_view.key.as_ref() != Some(&key) {
            let mut models = HashMap::new();
            for k in 2..=c.orgs.len() {
                models.insert(
                    k,
                    Arc::new(self.build_compare(&c.orgs[..k], history, &c.align)),
                );
            }
            self.cmp_view = CmpView {
                key: Some(key),
                models,
                l2: None,
                matched: HashSet::new(),
            };
        }
        // L2: the spine for the open row (all orgs; the view shows the first k)
        self.cmp_view.l2 = None;
        if c.level == 2
            && let Some(row) = self.cmp_l2_row().cloned()
        {
            self.cmp_view.l2 = Some(Arc::new(self.build_spine(&c, &row, history)));
            self.cmp_view.matched = self.ic_matched(&c);
        }
    }

    fn build_compare(
        &mut self,
        orgs: &[usize],
        history: bool,
        align: &str,
    ) -> Result<CmpModel, String> {
        let as_of = self.ui.as_of.clone();
        let aliases: Vec<String> = orgs
            .iter()
            .map(|&i| self.model.orgs[i].alias.clone())
            .collect();
        let books = self.books.as_mut().ok_or("no record")?;
        let mut ledgers = Vec::new();
        let mut bases = Vec::new();
        for &o in orgs {
            ledgers.push(books.ledger(&self.model, o)?);
            let b = books.basis(&self.model, o, &as_of, history);
            bases.push(b.as_ref().clone()?);
        }
        let mut tbs = HashMap::new();
        let mut charts = HashMap::new();
        for ((alias, l), b) in aliases.iter().zip(&ledgers).zip(&bases) {
            tbs.insert(alias.clone(), cmpm::tb_rows(l, b, RowStyle::Tui));
            charts.insert(alias.clone(), &l.chart);
        }
        let result = cmpm::tb_compare(&aliases, &tbs, &charts, align, &self.source.account_map)
            .map_err(|e| e.to_string())?;
        let rows = cmpm::flatten(&result).map_err(|e| e.to_string())?;
        let columns = result["columns"].as_array().cloned().unwrap_or_default();
        Ok(CmpModel {
            orgs: aliases,
            result,
            rows,
            columns,
        })
    }

    fn build_spine(&mut self, c: &CmpState, row: &Value, history: bool) -> Result<Value, String> {
        let as_of = self.ui.as_of.clone();
        let aliases: Vec<String> = c
            .orgs
            .iter()
            .map(|&i| self.model.orgs[i].alias.clone())
            .collect();
        let books = self.books.as_mut().ok_or("no record")?;
        let first = books.basis(&self.model, c.orgs[0], &as_of, history);
        let since = c
            .since
            .clone()
            .or_else(|| first.as_ref().as_ref().ok().map(|b| b.fy_start.clone()))
            .unwrap_or_default();
        let mut ledgers = Vec::new();
        let mut openings = HashMap::new();
        let mut ids: HashMap<String, Vec<String>> = HashMap::new();
        for (&o, alias) in c.orgs.iter().zip(&aliases) {
            let l = books.ledger(&self.model, o)?;
            let b = books.basis(&self.model, o, &as_of, history);
            let b = b.as_ref().as_ref().map_err(String::clone)?;
            let member: Vec<String> = row["member_ids"][alias.as_str()]
                .as_array()
                .map(|a| a.iter().map(s).collect())
                .unwrap_or_default();
            let opening = member.iter().fold(cmpm::D0, |acc, a| {
                ownpurse_core::money::add(acc, b.opening_for(&l, a, &since))
            });
            openings.insert(alias.clone(), opening);
            ids.insert(alias.clone(), member);
            ledgers.push(l);
        }
        let lrefs: HashMap<String, &ownpurse_core::Ledger> = aliases
            .iter()
            .cloned()
            .zip(ledgers.iter().map(|l| &**l))
            .collect();
        cmpm::postings_compare(&aliases, &lrefs, &ids, &since, &as_of, &openings)
            .map_err(|e| e.to_string())
    }

    /// (org, source id) of postings the Intercompany engine itself matched, for pairs with both orgs shown.
    /// Read from the engine's output; Compare never pairs anything (I3).
    fn ic_matched(&mut self, c: &CmpState) -> HashSet<(String, String)> {
        let mut out = HashSet::new();
        if !self.source.has_pairs() {
            return out;
        }
        let aliases: Vec<String> = c
            .orgs
            .iter()
            .map(|&i| self.model.orgs[i].alias.clone())
            .collect();
        let Some(books) = self.books.as_mut() else {
            return out;
        };
        let r = books.ic(&self.model, &self.source, &self.ui.as_of);
        if let Ok(r) = r.as_ref() {
            for p in r["pairs"].as_array().into_iter().flatten() {
                if aliases.contains(&s(&p["a_org"])) && aliases.contains(&s(&p["b_org"])) {
                    for m in p["matched"].as_array().into_iter().flatten() {
                        for side in ["a", "b"] {
                            out.insert((s(&m[side]["org"]), s(&m[side]["source_id"])));
                        }
                    }
                }
            }
        }
        out
    }
}

/// Dr/Cr text of a decimal string (`—` when null).
pub fn net(v: &Value) -> Option<Decimal> {
    crate::ic_nav::dec(v)
}

/// Mode helper re-exported for benches.
pub fn mode(history: bool) -> Mode {
    if history { Mode::History } else { Mode::Auto }
}
