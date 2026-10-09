//! Ledger navigation (spec §5): L1 → L2 → L3 per scope, cursors kept as ids (account id; source id + line)
//! and resolved back to rows after a reload, as-of or basis change (nearest neighbour if the id vanished).
//! Esc pops one level; at L1 it returns to the chain's origin (Overview) when there is one.

use crate::app::{App, Effect, Prompt, Scope, Tab};
use crate::books::Shared;
use crate::keymap::Ctx;
use crate::ledgers::{self, DocModel, L1Model, L2Model};

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct LedgerNav {
    /// 1 trial balance · 2 postings of `account` · 3 document `doc`
    pub level: u8,
    pub l1_sel: Option<String>,
    pub l1_idx: usize,
    pub l1_top: usize,
    pub account: Option<String>,
    pub l2_sel: Option<(String, usize)>,
    pub l2_idx: usize,
    pub l2_top: usize,
    pub doc: Option<(String, String)>,
    /// `f`: the L2 range's first day (None = the fiscal-year start).
    pub since: Option<String>,
    /// The trial balance's filter when it drilled into postings: restored on the way back up (§12).
    #[serde(default)]
    pub l1_filter: Option<String>,
}

impl Default for LedgerNav {
    fn default() -> Self {
        LedgerNav {
            level: 1,
            l1_sel: None,
            l1_idx: 0,
            l1_top: 0,
            account: None,
            l2_sel: None,
            l2_idx: 0,
            l2_top: 0,
            doc: None,
            since: None,
            l1_filter: None,
        }
    }
}

/// The prepared models (looked up in `Books` by `prepare`; draw only reads them).
#[derive(Clone, Debug, Default)]
pub struct LedgerView {
    pub l1: Option<Shared<L1Model>>,
    pub l2: Option<Shared<L2Model>>,
    pub doc: Option<Shared<DocModel>>,
}

fn ok<T>(s: &Option<Shared<T>>) -> Option<&T> {
    s.as_ref().and_then(|m| m.as_ref().as_ref().ok())
}

impl App {
    fn nav_key(&self) -> String {
        match &self.ui.via {
            Some(v) => format!("via:{}:{}", v.pair, self.model.orgs[v.org].alias),
            None => self.scope_label(&self.ui.scope),
        }
    }

    /// The current scope's Ledger state.
    pub fn nav(&self) -> LedgerNav {
        self.ui
            .ledger
            .get(&self.nav_key())
            .cloned()
            .unwrap_or_default()
    }

    pub(crate) fn nav_mut_pub(&mut self) -> &mut LedgerNav {
        self.nav_mut()
    }

    fn nav_mut(&mut self) -> &mut LedgerNav {
        let k = self.nav_key();
        self.ui.ledger.entry(k).or_default()
    }

    pub fn ledger_org(&self) -> Option<usize> {
        if let Some(v) = &self.ui.via {
            return self.model.orgs[v.org].last_sync.is_some().then_some(v.org);
        }
        match self.ui.scope {
            Scope::Org(i) if self.model.orgs[i].last_sync.is_some() => Some(i),
            _ => None,
        }
    }

    pub fn ledger_level(&self) -> u8 {
        if self.ledger_org().is_some() {
            self.nav().level
        } else {
            1
        }
    }

    pub fn ledger_ctx(&self) -> Ctx {
        match self.ledger_level() {
            3 => Ctx::LedgerL3,
            2 => Ctx::LedgerL2,
            _ => Ctx::LedgerL1,
        }
    }

    pub fn l1_model(&self) -> Option<&L1Model> {
        ok(&self.ledger_view.l1)
    }
    pub fn l2_model(&self) -> Option<&L2Model> {
        ok(&self.ledger_view.l2)
    }
    pub fn doc_model(&self) -> Option<&DocModel> {
        ok(&self.ledger_view.doc)
    }

    /// L1 rows after the `!` filter (indexes into the model's rows).
    pub fn l1_visible(&self) -> Vec<usize> {
        let Some(m) = self.l1_model() else {
            return vec![];
        };
        let only = self
            .ui
            .via
            .as_ref()
            .map(|v| &v.only)
            .filter(|o| !o.is_empty());
        let f = crate::filter::Filter::parse(&self.ui.filter);
        let mut v: Vec<usize> = (0..m.rows.len())
            .filter(|&i| !self.ui.differs_only || m.rows[i].differs)
            .filter(|&i| only.is_none_or(|o| o.contains(&m.rows[i].account_id)))
            .filter(|&i| {
                let r = &m.rows[i];
                f.matches(&format!("{}\n{}", r.code, r.name), Some(&r.code))
            })
            .collect();
        // under class bands (`tui.view.group_tb_by_class`): the accounting order of the classes, the
        // record's order within each
        if self.source.view.group_tb_by_class {
            v.sort_by_key(|&i| crate::views::ledger::class_rank(&m.rows[i].class));
        }
        v
    }

    /// L1 rows before the `/` filter (its "of N" total).
    pub fn l1_unfiltered(&self) -> usize {
        let Some(m) = self.l1_model() else { return 0 };
        let only = self
            .ui
            .via
            .as_ref()
            .map(|v| &v.only)
            .filter(|o| !o.is_empty());
        (0..m.rows.len())
            .filter(|&i| !self.ui.differs_only || m.rows[i].differs)
            .filter(|&i| only.is_none_or(|o| o.contains(&m.rows[i].account_id)))
            .count()
    }

    /// L2 postings through the `/` filter (indexes into the model's rows).
    pub fn l2_visible(&self) -> Vec<usize> {
        let Some(m) = self.l2_model() else {
            return vec![];
        };
        let f = crate::filter::Filter::parse(&self.ui.filter);
        (0..m.rows.len())
            .filter(|&i| {
                let r = &m.rows[i];
                f.matches(
                    &format!(
                        "{}\n{}\n{}\n{}\n{}\n{}",
                        r.date, r.kind, r.source, r.reference, r.contact, r.description
                    ),
                    None,
                )
            })
            .collect()
    }

    /// The L1 cursor (position in `l1_visible`): the selected id, else the nearest surviving row.
    pub fn l1_cursor(&self) -> Option<usize> {
        let m = self.l1_model()?;
        let vis = self.l1_visible();
        if vis.is_empty() {
            return None;
        }
        let nav = self.nav();
        nav.l1_sel
            .as_ref()
            .and_then(|id| vis.iter().position(|&i| &m.rows[i].account_id == id))
            .or(Some(nav.l1_idx.min(vis.len() - 1)))
    }

    /// The L2 cursor (position in `l2_visible`).
    pub fn l2_cursor(&self) -> Option<usize> {
        let m = self.l2_model()?;
        let vis = self.l2_visible();
        if vis.is_empty() {
            return None;
        }
        let nav = self.nav();
        nav.l2_sel
            .as_ref()
            .and_then(|(id, line)| {
                vis.iter()
                    .position(|&i| &m.rows[i].source_id == id && m.rows[i].line == *line)
            })
            .or(Some(nav.l2_idx.min(vis.len() - 1)))
    }

    pub fn ledger_has_row(&self) -> bool {
        match self.ledger_level() {
            1 => !self.l1_visible().is_empty(),
            2 => !self.l2_visible().is_empty(),
            _ => false,
        }
    }

    pub fn ledger_has_doc(&self) -> bool {
        self.ledger_level() == 3 && self.doc_model().is_some()
    }

    fn page(&self) -> usize {
        self.hits
            .content
            .map_or(10, |r| (r.height as usize).saturating_sub(3).max(1))
    }

    fn select_l1(&mut self, pos: usize) {
        let vis = self.l1_visible();
        let Some(m) = self.l1_model() else { return };
        if vis.is_empty() {
            return;
        }
        let pos = pos.min(vis.len() - 1);
        let id = m.rows[vis[pos]].account_id.clone();
        let page = self.page();
        let nav = self.nav_mut();
        nav.l1_sel = Some(id);
        nav.l1_idx = pos;
        if pos < nav.l1_top {
            nav.l1_top = pos;
        } else if pos >= nav.l1_top + page {
            nav.l1_top = pos + 1 - page;
        }
    }

    fn select_l2(&mut self, pos: usize) {
        let vis = self.l2_visible();
        let Some(m) = self.l2_model() else { return };
        if vis.is_empty() {
            return;
        }
        let pos = pos.min(vis.len() - 1);
        let r = &m.rows[vis[pos]];
        let sel = (r.source_id.clone(), r.line);
        let page = self.page();
        let nav = self.nav_mut();
        nav.l2_sel = Some(sel);
        nav.l2_idx = pos;
        if pos < nav.l2_top {
            nav.l2_top = pos;
        } else if pos >= nav.l2_top + page {
            nav.l2_top = pos + 1 - page;
        }
    }

    pub(crate) fn ledger_click_row(&mut self, i: usize) {
        match self.ledger_level() {
            1 => {
                let top = self.nav().l1_top;
                self.select_l1(top + i)
            }
            2 => {
                let top = self.nav().l2_top;
                self.select_l2(top + i)
            }
            _ => {}
        }
    }

    /// The wheel scrolls; the selection moves only if it left the view (spec §7).
    pub(crate) fn ledger_wheel(&mut self, down: bool) {
        let page = self.page();
        let level = self.ledger_level();
        let (n, cur) = match level {
            1 => (self.l1_visible().len(), self.l1_cursor()),
            2 => (self.l2_visible().len(), self.l2_cursor()),
            _ => return,
        };
        let max_top = n.saturating_sub(page);
        let nav = self.nav_mut();
        let top = if level == 1 {
            &mut nav.l1_top
        } else {
            &mut nav.l2_top
        };
        *top = if down {
            (*top + 1).min(max_top)
        } else {
            top.saturating_sub(1)
        };
        let t = *top;
        if let Some(c) = cur {
            let to = if c < t {
                Some(t)
            } else if c >= t + page {
                Some(t + page - 1)
            } else {
                None
            };
            if let Some(p) = to {
                if level == 1 {
                    self.select_l1(p)
                } else {
                    self.select_l2(p)
                }
            }
        }
    }

    /// Ledger actions. Returns false to let the global handler run (Esc at L1 returns to the chain origin).
    pub(crate) fn ledger_action(&mut self, action: &str, fx: &mut Vec<Effect>) -> bool {
        // prompts first (a range prompt belongs to L2)
        if let Some(Prompt::Range(text)) = self.ui.prompt.clone() {
            match action {
                "prompt.apply" => {
                    let t = text.trim().to_string();
                    if t.len() == 10
                        && ownpurse_core::dates::parse_iso(&t).is_some()
                        && t <= self.ui.as_of
                    {
                        self.nav_mut().since = Some(t);
                        self.ui.prompt = None;
                    } else {
                        self.ui.note = Some(format!(
                            "range needs a first day: YYYY-MM-DD, up to {}",
                            self.ui.as_of
                        ));
                    }
                    return true;
                }
                "prompt.cancel" => {
                    self.nav_mut().since = None;
                    self.ui.prompt = None;
                    return true;
                }
                _ => return false,
            }
        }
        if self.ledger_org().is_none() {
            return false;
        }
        let level = self.ledger_level();
        match action {
            "row.prev" | "row.next" | "page.prev" | "page.next" | "row.first" | "row.last" => {
                let (n, cur) = match level {
                    1 => (self.l1_visible().len(), self.l1_cursor().unwrap_or(0)),
                    2 => (self.l2_visible().len(), self.l2_cursor().unwrap_or(0)),
                    _ => return true,
                };
                let page = self.page();
                let pos = match action {
                    "row.prev" => cur.saturating_sub(1),
                    "row.next" => cur + 1,
                    "page.prev" => cur.saturating_sub(page),
                    "page.next" => cur + page,
                    "row.first" => 0,
                    _ => n.saturating_sub(1),
                };
                if level == 1 {
                    self.select_l1(pos)
                } else {
                    self.select_l2(pos)
                }
                true
            }
            "drill.in" => {
                match level {
                    1 => {
                        if let (Some(m), Some(c)) = (self.l1_model(), self.l1_cursor()) {
                            let id = m.rows[self.l1_visible()[c]].account_id.clone();
                            let idx = c;
                            // the filter chose the account; the postings start unfiltered, and the
                            // filter comes back with the trial balance (restore on return, §12)
                            let f = std::mem::take(&mut self.ui.filter);
                            self.nav_mut().l1_filter = (!f.is_empty()).then_some(f);
                            let nav = self.nav_mut();
                            nav.l1_sel = Some(id.clone());
                            nav.l1_idx = idx;
                            nav.level = 2;
                            if nav.account.as_deref() != Some(id.as_str()) {
                                nav.l2_sel = None;
                                nav.l2_idx = 0;
                                nav.l2_top = 0;
                            }
                            nav.account = Some(id);
                        }
                    }
                    2 => {
                        if let (Some(m), Some(c)) = (self.l2_model(), self.l2_cursor()) {
                            let r = &m.rows[self.l2_visible()[c]];
                            let doc = (r.source.to_string(), r.source_id.clone());
                            let sel = (r.source_id.clone(), r.line);
                            let nav = self.nav_mut();
                            nav.l2_sel = Some(sel);
                            nav.l2_idx = c;
                            nav.doc = Some(doc);
                            nav.level = 3;
                        }
                    }
                    _ => {}
                }
                true
            }
            "drill.back" => {
                // a via chain returns to its bridge from the level it entered at
                if let Some(v) = &self.ui.via
                    && level <= v.entry_level
                {
                    if !v.from_compare {
                        self.ui.tab = Tab::Intercompany;
                    }
                    self.ui.via = None;
                    return true;
                }
                if level > 1 {
                    self.nav_mut().level = level - 1;
                    if level == 2 {
                        self.ui.filter = self.nav_mut().l1_filter.take().unwrap_or_default();
                    }
                    true
                } else {
                    false
                }
            }
            "basis.toggle" => {
                self.ui.full_history = !self.ui.full_history;
                true
            }
            "filter.differs" => {
                self.ui.differs_only = !self.ui.differs_only;
                self.nav_mut().l1_top = 0;
                true
            }
            "range.edit" => {
                let since = self.l2_model().map(|m| m.since.clone()).unwrap_or_default();
                self.ui.prompt = Some(Prompt::Range(self.nav().since.unwrap_or(since)));
                true
            }
            "doc.open_xero" => {
                match self.doc_model().and_then(|d| d.link.clone()) {
                    Some(l) => {
                        fx.push(Effect::Open(l.url));
                        self.ui.note = Some(if l.verified {
                            "opened in your browser".into()
                        } else {
                            "opened in your browser (link unverified)".into()
                        });
                    }
                    None => {
                        self.ui.note = Some(
                            "No link template for this document type (links, per: document)."
                                .into(),
                        )
                    }
                }
                true
            }
            "doc.copy_id" => {
                if let Some(d) = self.doc_model() {
                    let id = d.id.clone();
                    self.ui.note = Some(format!("copied {id}"));
                    fx.push(Effect::Copy(id));
                }
                true
            }
            _ => false,
        }
    }

    /// Look up the Ledger models for the current scope / as-of / basis / drill (cached in `Books`).
    pub(crate) fn prepare_ledger(&mut self) {
        if self.ui.tab != Tab::Ledger {
            return;
        }
        let Some(org) = self.ledger_org() else {
            self.ledger_view = LedgerView::default();
            return;
        };
        let nav = self.nav();
        let Some(books) = self.books.as_mut() else {
            return;
        };
        let history = self.ui.full_history;
        let as_of = self.ui.as_of.clone();
        let l1 = ledgers::l1(books, &self.model, org, &as_of, history);
        let l2 = match (&nav.account, nav.level >= 2) {
            (Some(a), true) => Some(ledgers::l2(
                books,
                &self.model,
                org,
                a,
                &as_of,
                nav.since.as_deref(),
                history,
            )),
            _ => None,
        };
        let doc = match (&nav.doc, nav.level == 3) {
            (Some((s, id)), true) => Some(ledgers::doc(
                books,
                &self.model,
                &self.source.links,
                org,
                s,
                id,
            )),
            _ => None,
        };
        self.ledger_view = LedgerView {
            l1: Some(l1),
            l2,
            doc,
        };
    }
}
