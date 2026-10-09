//! Classify › Groups (spec classify §3.E) on the core's groups read model (`core::classify::groups`): the
//! group list, and the detail: members across orgs (origin → mirror → settlement), the chain check (never a
//! line-level claim for many-to-one, §10.9), the running balance of a settlement group and its pair rule
//! checked on the record's ledger balances, notes. Keys: Enter opens a member's document (Esc returns),
//! `r` role, `m` add member (same amount within ±7 days in another org first), `n` note, `S` status, `D`
//! remove member. Every write is an attributed, undoable store event.

use super::{LabelPicker, Mode, NoteEdit, Op};
use crate::app::{App, Effect};
use ownpurse_core::classify::Anchor;
use ownpurse_core::classify::groups::GroupDetail;

/// Which side has the cursor.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum GFocus {
    #[default]
    List,
    Members,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct GroupsUi {
    /// The selected group's id, and the member cursor (an anchor).
    pub sel: Option<String>,
    pub member: Option<String>,
    pub focus: GFocus,
    /// A member's document, open over the detail: (org, record entity, document id).
    pub doc: Option<(String, String, String)>,
}

/// One row of the group list.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GroupRow {
    pub id: String,
    pub kind: String,
    pub title: String,
    pub status: String,
    pub members: usize,
}

/// The picker's group-side kinds (in `LabelPicker.label_type`).
pub const PICK_ROLE: &str = "g_role";
pub const PICK_STATUS: &str = "g_status";
pub const PICK_MEMBER: &str = "g_member";

impl App {
    pub fn grp_rows(&self) -> Vec<GroupRow> {
        self.classify
            .as_ref()
            .map(|d| d.provider.groups_list())
            .unwrap_or_default()
    }

    pub fn grp_cursor(&self) -> Option<usize> {
        let rows = self.grp_rows();
        (!rows.is_empty()).then(|| {
            self.ui
                .classify
                .groups
                .sel
                .as_ref()
                .and_then(|s| rows.iter().position(|r| &r.id == s))
                .unwrap_or(0)
        })
    }

    pub fn grp_selected(&self) -> Option<GroupRow> {
        let rows = self.grp_rows();
        self.grp_cursor().and_then(|i| rows.get(i).cloned())
    }

    pub fn grp_detail(&self) -> Option<GroupDetail> {
        let g = self.grp_selected()?;
        self.classify.as_ref()?.provider.group_detail(&g.id)
    }

    pub fn grp_member_cursor(&self, d: &GroupDetail) -> Option<usize> {
        (!d.members.is_empty()).then(|| {
            self.ui
                .classify
                .groups
                .member
                .as_ref()
                .and_then(|a| d.members.iter().position(|m| m.anchor.to_string() == *a))
                .unwrap_or(0)
        })
    }

    pub fn grp_member(&self) -> Option<Anchor> {
        let d = self.grp_detail()?;
        let i = self.grp_member_cursor(&d)?;
        d.members.get(i).map(|m| m.anchor.clone())
    }

    /// The pair rule's check on the record's balances (settlement groups whose orgs a rule names).
    pub fn grp_pair_check(&self) -> Option<ownpurse_core::classify::groups::PairCheck> {
        let g = self.grp_selected()?;
        let d = self.classify.as_ref()?;
        d.pair_check(&g.id, &self.source.orgs)
    }

    /// Rows for the group pickers (`r` role, `S` status, `m` add member).
    pub(crate) fn grp_picker_rows(&self, kind: &str, query: &str) -> Vec<(String, String)> {
        let Some(d) = self.classify.as_ref() else {
            return vec![];
        };
        let cfg = d.provider.config();
        let q = query.to_lowercase();
        let keep = |id: &str, name: &str| {
            q.is_empty() || id.to_lowercase().contains(&q) || name.to_lowercase().contains(&q)
        };
        match kind {
            PICK_ROLE => cfg
                .groups
                .roles
                .iter()
                .chain(std::iter::once(&"none".to_string()))
                .map(|r| (r.clone(), r.clone()))
                .filter(|(i, n)| keep(i, n))
                .collect(),
            PICK_STATUS => ownpurse_core::classify::store::GROUP_STATUSES
                .iter()
                .map(|s| (s.to_string(), s.to_string()))
                .filter(|(i, n)| keep(i, n))
                .collect(),
            PICK_MEMBER => {
                let Some(det) = self.grp_detail() else {
                    return vec![];
                };
                let line = |a: &Anchor, org: &str, date: &str, amt, payee: &str, why: &str| {
                    (
                        a.to_string(),
                        format!(
                            "{org} · {date} · {} · {payee}{why}",
                            ownpurse_core::money::fmt(amt)
                        ),
                    )
                };
                // same amount within ±7 days in another org, for each member, first
                let mut out: Vec<(String, String)> = Vec::new();
                for m in &det.members {
                    for s in d.provider.suggest_members(&m.anchor) {
                        let row = line(
                            &s.anchor,
                            &s.org,
                            &s.date,
                            s.amount,
                            &s.payee,
                            &format!(" · {} days apart", s.days_apart),
                        );
                        if !out.iter().any(|(a, _)| *a == row.0) {
                            out.push(row);
                        }
                    }
                }
                // then any line, by org, date, amount or payee, once something is typed
                if !q.is_empty() {
                    for r in d.provider.queue(
                        &ownpurse_core::classify::Filter::parse("status:all").unwrap_or_default(),
                        ownpurse_core::classify::Sort::DateAsc,
                    ) {
                        let row = line(&r.anchor, &r.org, &r.date, r.amount, &r.payee, "");
                        if !out.iter().any(|(a, _)| *a == row.0)
                            && !det.members.iter().any(|m| m.anchor == r.anchor)
                        {
                            out.push(row);
                        }
                    }
                }
                out.into_iter()
                    .filter(|(i, n)| keep(i, n))
                    .take(200)
                    .collect()
            }
            _ => vec![],
        }
    }

    /// A pick in a group picker: the write.
    pub(crate) fn grp_pick(&mut self, kind: &str, id: &str, fx: &mut Vec<Effect>) {
        let Some(g) = self.grp_selected() else { return };
        let op = match kind {
            PICK_ROLE => {
                let Some(a) = self.grp_member() else { return };
                Op::GroupMember {
                    group: g.id,
                    anchor: a.to_string(),
                    role: (id != "none").then(|| id.to_string()),
                }
            }
            PICK_STATUS => Op::GroupStatus {
                group: g.id,
                status: id.to_string(),
            },
            PICK_MEMBER => Op::GroupMember {
                group: g.id,
                anchor: id.to_string(),
                role: None,
            },
            _ => return,
        };
        self.cl_write_pub(op, fx);
    }

    /// A click on drawn list row `i` (the list scrolls to keep the cursor in its last row).
    pub(crate) fn grp_click_row(&mut self, i: usize) {
        let h = self.hits.content.map_or(10, |r| (r.height as usize).max(1));
        let top = self
            .grp_cursor()
            .map_or(0, |c| c.saturating_sub(h.saturating_sub(1)));
        self.ui.classify.groups.focus = GFocus::List;
        self.grp_select(top + i);
    }

    fn grp_select(&mut self, pos: usize) {
        let rows = self.grp_rows();
        if let Some(r) = rows.get(pos.min(rows.len().saturating_sub(1))) {
            self.ui.classify.groups.sel = Some(r.id.clone());
            self.ui.classify.groups.member = None;
        }
    }

    /// Groups keys. Returns false for anything the shell handles.
    pub(crate) fn grp_action(&mut self, action: &str, fx: &mut Vec<Effect>) -> bool {
        if self.ui.classify.mode != Mode::Groups || self.classify.is_none() {
            return false;
        }
        let open = |kind: &str| LabelPicker {
            label_type: kind.into(),
            query: String::new(),
            sel: 0,
        };
        let has_member = self.grp_member().is_some();
        let has_group = self.grp_selected().is_some();
        let gu = &mut self.ui.classify.groups;
        match action {
            "drill.back" if gu.doc.is_some() => gu.doc = None,
            "drill.back" if gu.focus == GFocus::Members => gu.focus = GFocus::List,
            "groups.focus_next" => gu.focus = GFocus::Members,
            "groups.focus_prev" => gu.focus = GFocus::List,
            "drill.in" if gu.doc.is_some() => {}
            "drill.in" => match gu.focus {
                GFocus::List => gu.focus = GFocus::Members,
                GFocus::Members => {
                    let Some(a) = self.grp_member() else {
                        return true;
                    };
                    if let Anchor::Line {
                        org,
                        doc_type,
                        doc_id,
                        ..
                    } = a
                    {
                        self.ui.classify.groups.doc =
                            Some((org, doc_type.entity().to_string(), doc_id));
                    }
                }
            },
            "groups.role" if has_member && gu.focus == GFocus::Members => {
                self.ui.classify.picker = Some(open(PICK_ROLE))
            }
            "groups.role" => self.ui.note = Some("Choose a member first (→).".into()),
            "groups.status" if has_group => self.ui.classify.picker = Some(open(PICK_STATUS)),
            "groups.add_member" if has_group => self.ui.classify.picker = Some(open(PICK_MEMBER)),
            "groups.note" if has_group => {
                self.ui.classify.note = Some(NoteEdit {
                    text: String::new(),
                    multi: false,
                })
            }
            "groups.remove" if has_member && gu.focus == GFocus::Members => {
                let (Some(g), Some(a)) = (self.grp_selected(), self.grp_member()) else {
                    return true;
                };
                self.cl_write_pub(
                    Op::GroupRemove {
                        group: g.id,
                        anchor: a.to_string(),
                    },
                    fx,
                );
            }
            "groups.remove" => self.ui.note = Some("Choose a member first (→).".into()),
            "groups.status" | "groups.add_member" | "groups.note" => {
                self.ui.note = Some("No group yet: G in the Queue starts one.".into())
            }
            a => {
                if gu.doc.is_some() {
                    return false;
                }
                match gu.focus {
                    GFocus::List => {
                        let n = self.grp_rows().len();
                        match crate::views::activity::moved(
                            a,
                            self.grp_cursor().unwrap_or(0),
                            n,
                            10,
                        ) {
                            Some(pos) if n > 0 => self.grp_select(pos),
                            Some(_) => {}
                            None => return false,
                        }
                    }
                    GFocus::Members => {
                        let Some(d) = self.grp_detail() else {
                            return false;
                        };
                        let n = d.members.len();
                        let cur = self.grp_member_cursor(&d).unwrap_or(0);
                        match crate::views::activity::moved(a, cur, n, 10) {
                            Some(pos) if n > 0 => {
                                self.ui.classify.groups.member =
                                    Some(d.members[pos.min(n - 1)].anchor.to_string())
                            }
                            Some(_) => {}
                            None => return false,
                        }
                    }
                }
            }
        }
        true
    }
}
