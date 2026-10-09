//! Intercompany navigation and models (spec §9 tab 3): L1 pairs → L2 bridge → L3 an item's document in its
//! own org (inside tab 3; the scope is unchanged). State per scope, cursors kept as ids (pair id; item
//! source id). `a` / `b` open a side's accounts in the Ledger "via" the pair; Esc there returns to the bridge.
//! The engine result is core's `intercompany::build`, cached per (as-of, record version) in `Books`.

use crate::app::{App, Effect, Tab};
use crate::books::Shared;
use crate::keymap::Ctx;
use crate::ledgers::{self, DocModel};
use ownpurse_core::intercompany as icm;
use rust_decimal::Decimal;
use serde_json::Value;
use std::cmp::Ordering;

/// Sort modes for `s`: never drops rows.
pub const SORTS: [&str; 3] = ["status", "difference", "pair"];

#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct IcNav {
    /// 1 pairs · 2 bridge of `pair` · 3 document `doc`
    pub level: u8,
    pub pair: Option<String>,
    pub l1_idx: usize,
    pub l1_top: usize,
    pub item: Option<String>,
    pub item_idx: usize,
    pub item_top: usize,
    /// (org alias, source, document id, kind)
    pub doc: Option<(String, String, String, String)>,
}

/// Viewing another org's Ledger from a bridge (`a` / `b`): the pair, the org, and the side's accounts.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Via {
    pub pair: String,
    pub label: String,
    pub org: usize,
    pub only: Vec<String>,
    /// The Ledger level the via chain entered at (Esc there returns to the bridge).
    pub entry_level: u8,
    /// Entered from Compare (Esc returns to Compare on the Ledger tab, not to a bridge).
    pub from_compare: bool,
}

/// The prepared Intercompany models (looked up in `Books` by `prepare`).
#[derive(Clone, Debug, Default)]
pub struct IcView {
    pub as_of: String,
    pub result: Option<Shared<Value>>,
    pub doc: Option<Shared<DocModel>>,
}

/// One L1 row, parsed from the engine's pair report.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IcRow {
    pub id: String,
    pub label: String,
    pub a_org: String,
    pub b_org: String,
    pub a_net: Option<Decimal>,
    pub b_party_net: Option<Decimal>,
    pub difference: Option<Decimal>,
    pub residual: Option<Decimal>,
    pub status: String,
    pub reason: String,
}

pub fn dec(v: &Value) -> Option<Decimal> {
    v.as_str().and_then(|s| ownpurse_core::money::parse(s).ok())
}

pub fn s(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        Value::Null => String::new(),
        other => other.to_string(),
    }
}

impl IcRow {
    pub fn of(p: &Value) -> IcRow {
        IcRow {
            id: s(&p["id"]),
            label: s(&p["label"]),
            a_org: s(&p["a_org"]),
            b_org: s(&p["b_org"]),
            a_net: dec(&p["a_net"]),
            b_party_net: dec(&p["b_party_net"]),
            difference: dec(&p["difference"]),
            residual: dec(&p["residual"]),
            status: s(&p["status"]),
            reason: s(&p["reason"]),
        }
    }

    /// Status words (F5): `✓ matches`, `bridged`, `unexplained`, `— not compared`, `not supported: <reason>`.
    pub fn status_text(&self) -> String {
        match self.status.as_str() {
            "matches" => "✓ matches".into(),
            "not compared" => "— not compared".into(),
            // the reason goes in the refusal band and the L1 detail line
            "not supported" => "not supported".into(),
            other => other.into(),
        }
    }

    /// The refusal for a pair that can't be opened (currency / fiscal-year copy only when that's the
    /// reason; an unknown org says what to check).
    pub fn refusal(&self) -> String {
        if self.status == "not compared" {
            let reason = if self.reason.is_empty() {
                self.status.clone()
            } else {
                self.reason.clone()
            };
            return format!(
                "{}: {reason}. ⌃P → Refetch Xero trial balance to compare it.",
                self.label
            );
        }
        if let Some(orgs) = self.reason.strip_prefix("unknown org ") {
            let list = orgs
                .split(", ")
                .map(|o| format!("`{o}`"))
                .collect::<Vec<_>>()
                .join(", ");
            let (noun, verb) = if orgs.contains(", ") {
                ("orgs", "aren't")
            } else {
                ("org", "isn't")
            };
            return format!(
                "{}: {noun} {list} {verb} connected or configured. Check the pair in ownpurse.json.",
                self.label
            );
        }
        if self.reason.contains("currenc") || self.reason.contains("fiscal") {
            return format!(
                "{}: {}. Cross-org views refuse different currencies or fiscal years.",
                self.label, self.reason
            );
        }
        format!(
            "{}: {}.",
            self.label,
            if self.reason.is_empty() {
                &self.status
            } else {
                &self.reason
            }
        )
    }

    pub fn openable(&self) -> bool {
        !matches!(self.status.as_str(), "not compared" | "not supported")
    }
}

impl App {
    fn ic_key(&self) -> String {
        self.scope_label(&self.ui.scope)
    }

    pub fn ic_nav(&self) -> IcNav {
        let mut n = self.ui.ic.get(&self.ic_key()).cloned().unwrap_or_default();
        n.level = n.level.max(1);
        n
    }

    fn ic_nav_mut(&mut self) -> &mut IcNav {
        let k = self.ic_key();
        let n = self.ui.ic.entry(k).or_default();
        n.level = n.level.max(1);
        n
    }

    pub fn ic_configured(&self) -> bool {
        self.source.has_pairs()
    }

    pub fn ic_result(&self) -> Option<&Value> {
        self.ic_view
            .result
            .as_ref()
            .and_then(|r| r.as_ref().as_ref().ok())
    }

    pub fn ic_error(&self) -> Option<&str> {
        self.ic_view
            .result
            .as_ref()
            .and_then(|r| r.as_ref().as_ref().err().map(String::as_str))
    }

    /// Pair reports in scope (all: every pair; org: either side; group: both sides in the group).
    pub fn ic_scoped(&self) -> Vec<&Value> {
        let Some(r) = self.ic_result() else {
            return vec![];
        };
        let members: Vec<String> = self
            .members(&self.ui.scope)
            .into_iter()
            .map(|i| self.model.orgs[i].alias.clone())
            .collect();
        let scope = self.scope_label(&self.ui.scope);
        let config = self.source.ic_config["intercompany"]["pairs"]
            .as_array()
            .cloned()
            .unwrap_or_default();
        r["pairs"]
            .as_array()
            .map(|a| {
                a.iter()
                    .filter(|p| {
                        config
                            .iter()
                            .find(|c| c["id"] == p["id"])
                            .is_none_or(|c| icm::in_scope(c, &members, &scope))
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    /// L1 rows: scoped, sorted by the current mode (stable), then the `/` filter. Never drops a row otherwise.
    pub fn ic_rows(&self) -> Vec<&Value> {
        let mut rows = self.ic_scoped();
        let cmp = icm::sort_cmp(SORTS[self.ui.ic_sort % SORTS.len()]);
        rows.sort_by(|a, b| cmp(a, b));
        let f = crate::filter::Filter::parse(&self.ui.filter);
        rows.into_iter()
            .filter(|p| {
                let hay = [&p["id"], &p["label"], &p["a_org"], &p["b_org"]]
                    .iter()
                    .map(|v| s(v))
                    .collect::<Vec<_>>()
                    .join("\n");
                f.matches(&hay, None)
            })
            .collect()
    }

    pub fn ic_l1_cursor(&self) -> Option<usize> {
        let rows = self.ic_rows();
        if rows.is_empty() {
            return None;
        }
        let nav = self.ic_nav();
        nav.pair
            .as_ref()
            .and_then(|id| rows.iter().position(|p| s(&p["id"]) == *id))
            .or(Some(nav.l1_idx.min(rows.len() - 1)))
    }

    /// The open pair (L2 / L3), if it is still in scope and comparable.
    pub fn ic_pair(&self) -> Option<&Value> {
        let id = self.ic_nav().pair?;
        self.ic_scoped()
            .into_iter()
            .find(|p| s(&p["id"]) == id && IcRow::of(p).openable())
    }

    pub fn ic_level(&self) -> u8 {
        let n = self.ic_nav();
        if n.level >= 2 && self.ic_pair().is_none() {
            1 // as-of or scope changed and the pair can no longer be opened
        } else {
            n.level
        }
    }

    pub fn ic_ctx(&self) -> Ctx {
        match self.ic_level() {
            3 => Ctx::IcL3,
            2 => Ctx::IcL2,
            _ => Ctx::IcL1,
        }
    }

    pub fn ic_items(&self) -> Vec<Value> {
        self.ic_pair()
            .and_then(|p| p["movement_items"].as_array().cloned())
            .unwrap_or_default()
    }

    pub fn ic_item_cursor(&self) -> Option<usize> {
        let items = self.ic_items();
        if items.is_empty() {
            return None;
        }
        let nav = self.ic_nav();
        nav.item
            .as_ref()
            .and_then(|id| items.iter().position(|i| s(&i["source_id"]) == *id))
            .or(Some(nav.item_idx.min(items.len() - 1)))
    }

    pub fn ic_doc(&self) -> Option<&DocModel> {
        self.ic_view
            .doc
            .as_ref()
            .and_then(|d| d.as_ref().as_ref().ok())
    }

    pub fn ic_has_row(&self) -> bool {
        match self.ic_level() {
            1 => !self.ic_rows().is_empty(),
            2 => !self.ic_items().is_empty(),
            _ => false,
        }
    }

    fn ic_page(&self) -> usize {
        self.hits
            .content
            .map_or(5, |r| (r.height as usize).saturating_sub(1).max(1))
    }

    fn ic_select_l1(&mut self, pos: usize) {
        let rows = self.ic_rows();
        if rows.is_empty() {
            return;
        }
        let pos = pos.min(rows.len() - 1);
        let id = s(&rows[pos]["id"]);
        let page = self.ic_page();
        let n = self.ic_nav_mut();
        n.pair = Some(id);
        n.l1_idx = pos;
        if pos < n.l1_top {
            n.l1_top = pos;
        } else if pos >= n.l1_top + page {
            n.l1_top = pos + 1 - page;
        }
    }

    fn ic_select_item(&mut self, pos: usize) {
        let items = self.ic_items();
        if items.is_empty() {
            return;
        }
        let pos = pos.min(items.len() - 1);
        let id = s(&items[pos]["source_id"]);
        let page = self.ic_page();
        let n = self.ic_nav_mut();
        n.item = Some(id);
        n.item_idx = pos;
        if pos < n.item_top {
            n.item_top = pos;
        } else if pos >= n.item_top + page {
            n.item_top = pos + 1 - page;
        }
    }

    pub(crate) fn ic_click_row(&mut self, i: usize) {
        match self.ic_level() {
            1 => {
                let top = self.ic_nav().l1_top;
                self.ic_select_l1(top + i)
            }
            2 => {
                let top = self.ic_nav().item_top;
                self.ic_select_item(top + i)
            }
            _ => {}
        }
    }

    pub(crate) fn ic_wheel(&mut self, down: bool) {
        let level = self.ic_level();
        let (n, cur) = match level {
            1 => (self.ic_rows().len(), self.ic_l1_cursor()),
            2 => (self.ic_items().len(), self.ic_item_cursor()),
            _ => return,
        };
        let page = self.ic_page();
        let max_top = n.saturating_sub(page);
        let nav = self.ic_nav_mut();
        let top = if level == 1 {
            &mut nav.l1_top
        } else {
            &mut nav.item_top
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
                    self.ic_select_l1(p)
                } else {
                    self.ic_select_item(p)
                }
            }
        }
    }

    /// Intercompany actions; false lets the global handler run (Esc at L1 → the chain origin / filter).
    pub(crate) fn ic_action(&mut self, action: &str, fx: &mut Vec<Effect>) -> bool {
        let level = self.ic_level();
        match action {
            "row.prev" | "row.next" | "page.prev" | "page.next" | "row.first" | "row.last" => {
                let (n, cur) = match level {
                    1 => (self.ic_rows().len(), self.ic_l1_cursor().unwrap_or(0)),
                    2 => (self.ic_items().len(), self.ic_item_cursor().unwrap_or(0)),
                    _ => return true,
                };
                let page = self.ic_page();
                let pos = match action {
                    "row.prev" => cur.saturating_sub(1),
                    "row.next" => cur + 1,
                    "page.prev" => cur.saturating_sub(page),
                    "page.next" => cur + page,
                    "row.first" => 0,
                    _ => n.saturating_sub(1),
                };
                if level == 1 {
                    self.ic_select_l1(pos)
                } else {
                    self.ic_select_item(pos)
                }
                true
            }
            "drill.in" => {
                match level {
                    1 => {
                        let Some(c) = self.ic_l1_cursor() else {
                            return true;
                        };
                        let row = IcRow::of(self.ic_rows()[c]);
                        if !row.openable() {
                            self.refuse(row.refusal());
                            return true;
                        }
                        let n = self.ic_nav_mut();
                        if n.pair.as_deref() != Some(row.id.as_str()) {
                            n.item = None;
                            n.item_idx = 0;
                            n.item_top = 0;
                        }
                        n.pair = Some(row.id);
                        n.l1_idx = c;
                        n.level = 2;
                    }
                    2 => {
                        let Some(c) = self.ic_item_cursor() else {
                            return true;
                        };
                        let it = self.ic_items()[c].clone();
                        let kind = {
                            let k = s(&it["kind"]);
                            if k.is_empty() { s(&it["source"]) } else { k }
                        };
                        let n = self.ic_nav_mut();
                        n.item = Some(s(&it["source_id"]));
                        n.item_idx = c;
                        n.doc = Some((s(&it["org"]), s(&it["source"]), s(&it["source_id"]), kind));
                        n.level = 3;
                    }
                    _ => {}
                }
                true
            }
            "drill.back" if level > 1 => {
                self.ic_nav_mut().level = level - 1;
                true
            }
            "ic.sort" => {
                self.ui.ic_sort = (self.ui.ic_sort + 1) % SORTS.len();
                true
            }
            "ic.side_a" | "ic.side_b" if level == 2 => {
                let Some(p) = self.ic_pair().cloned() else {
                    return true;
                };
                let a = action == "ic.side_a";
                let org = s(if a { &p["a_org"] } else { &p["b_org"] });
                let ids: Vec<String> = p[if a { "a_account_ids" } else { "b_account_ids" }]
                    .as_array()
                    .map(|v| v.iter().map(s).collect())
                    .unwrap_or_default();
                let Some(oi) = self.model.orgs.iter().position(|o| o.alias == org) else {
                    self.ui.note = Some(format!("{org} is not an org in this profile"));
                    return true;
                };
                let entry = if ids.len() == 1 { 2 } else { 1 };
                self.ui.via = Some(Via {
                    pair: s(&p["id"]),
                    label: s(&p["label"]),
                    org: oi,
                    only: ids.clone(),
                    entry_level: entry,
                    from_compare: false,
                });
                let nav = self.nav_mut_pub();
                *nav = Default::default();
                nav.level = entry;
                if entry == 2 {
                    nav.account = ids.first().cloned();
                }
                self.ui.tab = Tab::Ledger;
                self.ui.prompt = None;
                true
            }
            "doc.open_xero" | "doc.copy_id" => {
                let (link, id) = match level {
                    3 => match self.ic_doc() {
                        Some(d) => (d.link.as_ref().map(|l| l.url.clone()), d.id.clone()),
                        None => return true,
                    },
                    2 => match self.ic_item_cursor() {
                        Some(c) => {
                            let it = &self.ic_items()[c];
                            let source = s(&it["source"]);
                            let id = s(&it["source_id"]);
                            let link = self
                                .source
                                .links
                                .as_array()
                                .and_then(|ls| {
                                    ls.iter().find(|l| {
                                        l["per"] == "document" && s(&l["source"]) == source
                                    })
                                })
                                .map(|l| s(&l["url"]).replace("{document_id}", &id));
                            (link, id)
                        }
                        None => return true,
                    },
                    _ => return false,
                };
                if action == "doc.copy_id" {
                    self.ui.note = Some(format!("copied {id}"));
                    fx.push(Effect::Copy(id));
                } else if let Some(url) = link {
                    self.ui.note = Some("opened in your browser (link unverified)".into());
                    fx.push(Effect::Open(url));
                } else {
                    self.ui.note = Some(
                        "No link template for this document type (links, per: document).".into(),
                    );
                }
                true
            }
            _ => false,
        }
    }

    /// Look up the engine result (cached per as-of in `Books`) and the L3 document.
    pub(crate) fn prepare_ic(&mut self) {
        if self.ui.tab != Tab::Intercompany && self.ui.tab != Tab::Overview {
            return;
        }
        if !self.ic_configured() {
            self.ic_view = IcView::default();
            return;
        }
        let nav = self
            .ui
            .ic
            .get(&self.scope_label(&self.ui.scope))
            .cloned()
            .unwrap_or_default();
        let Some(books) = self.books.as_mut() else {
            return;
        };
        let result = books.ic(&self.model, &self.source, &self.ui.as_of);
        let doc = match (&nav.doc, nav.level == 3) {
            (Some((org, source, id, _)), true) => self
                .model
                .orgs
                .iter()
                .position(|o| &o.alias == org)
                .map(|oi| ledgers::doc(books, &self.model, &self.source.links, oi, source, id)),
            _ => None,
        };
        self.ic_view = IcView {
            as_of: self.ui.as_of.clone(),
            result: Some(result),
            doc,
        };
    }
}

/// Order helper for tests: the comparator the L1 uses.
pub fn order(mode: &str) -> impl Fn(&Value, &Value) -> Ordering {
    icm::sort_cmp(mode)
}
