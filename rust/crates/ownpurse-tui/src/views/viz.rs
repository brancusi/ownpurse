//! The explainer panel (walkthroughs design §4): a visualisation opened over the current tab's content
//! area. `ui.viz` holds the spec (in the canonical state, so a state with a panel open round-trips, V21);
//! the model is computed in `prepare` from the books (the same view, ledgers, basis and intercompany engine
//! the tabs read, so a shown manifest applies) and cached per (spec, as-of, record, manifest). Esc closes it;
//! ↑/↓ scroll it; changing tab or scope closes it.
//!
//! Opened by: the agent verb `viz <spec-json>` (cosmetic), the palette's "Explain: …" entries, and `T` /
//! `M` on a Ledger account (its T-account / its timeline).

use crate::app::{App, Effect, PalEntry, PalRun, Tab};
use crate::hit::Hits;
use crate::theme_tokens::Token;
use ownpurse_core::intercompany::{IcContext, IcError};
use ownpurse_core::viz::{self, Extra, Model, Spec};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

/// The open panel: its spec and the body's scroll.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct VizOpen {
    pub spec: Spec,
    #[serde(default)]
    pub scroll: u16,
}

/// The prepared model (key, result).
pub type VizView = Option<(String, Arc<Result<Model, String>>)>;

impl App {
    /// The open panel's model (prepared in `update`).
    pub fn viz_model(&self) -> Option<&Result<Model, String>> {
        self.ui.viz.as_ref()?;
        self.viz_view.as_ref().map(|(_, m)| m.as_ref())
    }

    /// Open a panel (replacing one already open).
    pub fn viz_open(&mut self, spec: Spec) {
        self.ui.viz = Some(VizOpen { spec, scroll: 0 });
        self.ui.overlay = None;
        self.prepare_viz();
    }

    pub fn viz_close(&mut self) {
        self.ui.viz = None;
        self.viz_view = None;
    }

    fn viz_key(&self, spec: &Spec) -> String {
        format!(
            "{}|{}|{}|{}:{}|{}",
            spec.to_json(),
            self.ui.as_of,
            self.model.record_version,
            self.ui.manifest.as_deref().unwrap_or(""),
            self.manifest_head(),
            self.viz_today(),
        )
    }

    /// Today (UTC) from the app's clock: the checklist's days left.
    pub fn viz_today(&self) -> String {
        let (y, m, d) = ownpurse_core::dates::civil_from_days(self.now.div_euclid(86_400));
        ownpurse_core::dates::iso(y, m, d)
    }

    /// Compute (or reuse) the open panel's model.
    pub(crate) fn prepare_viz(&mut self) {
        let Some(open) = self.ui.viz.clone() else {
            self.viz_view = None;
            return;
        };
        let key = self.viz_key(&open.spec);
        if self.viz_view.as_ref().is_some_and(|(k, _)| *k == key) {
            return;
        }
        let r = self.compute_viz(&open.spec);
        self.viz_view = Some((key, Arc::new(r)));
    }

    /// The manifest effects a `before_after` panel reads: the shown manifest's (already computed for the
    /// Manifest tab) or a named one's.
    fn viz_effects(
        &mut self,
        name: Option<&str>,
    ) -> Result<ownpurse_core::manifest::effects::Diff, String> {
        let name = name
            .map(str::to_string)
            .or_else(|| self.ui.manifest.clone())
            .ok_or("No manifest shown: name one in the spec (`manifest`) or show one first.")?;
        if self.ui.manifest.as_deref() == Some(name.as_str()) {
            self.prepare_manifest();
            if let Some(f) = self.manifest_facts() {
                return f.diff.clone();
            }
        }
        let m = self
            .manifest_store()
            .and_then(|s| s.manifest(&name))
            .ok_or_else(|| format!("No manifest named `{name}`."))?;
        let ctx = self.manifest_ctx();
        let ic = serde_json::json!({
            "parties": self.source.ic_config.get("parties").cloned().unwrap_or_default(),
            "intercompany": self.source.ic_config.get("intercompany").cloned().unwrap_or_default(),
        });
        let books = self.books.as_ref().ok_or("The local record is not open.")?;
        ownpurse_core::manifest::effects::diff(
            books.record(),
            &ctx,
            &m,
            &self.ui.as_of,
            Some(&ic),
            &self.source.zero_accounts,
        )
        .map_err(|e| e.message)
    }

    fn compute_viz(&mut self, spec: &Spec) -> Result<Model, String> {
        let effects = match spec {
            Spec::BeforeAfter { manifest, .. } => Some(self.viz_effects(manifest.as_deref())?),
            _ => None,
        };
        let base_dir = self
            .source
            .config_file
            .as_ref()
            .and_then(|p| p.parent().map(|d| d.to_path_buf()));
        let today = self.viz_today();
        let orgs: Vec<(String, String)> = self
            .model
            .orgs
            .iter()
            .map(|o| (o.alias.clone(), o.name.clone()))
            .collect();
        let books = self.books.as_mut().ok_or("The local record is not open.")?;
        let mut ledgers = HashMap::new();
        for i in 0..self.model.orgs.len() {
            if let Ok(l) = books.ledger(&self.model, i) {
                ledgers.insert(self.model.orgs[i].alias.clone(), l);
            }
        }
        let tenants: HashMap<String, String> = self.source.orgs.iter().cloned().collect();
        let ctx = IcContext::new(
            books.view(),
            tenants,
            &self.source.ic_config,
            &self.ui.as_of,
        )
        .with_ledgers(move |org: &str| {
            ledgers
                .get(org)
                .cloned()
                .ok_or_else(|| IcError::Config(format!("{org} has no local record")))
        });
        let extra = Extra {
            today,
            base_dir,
            effects: effects.as_ref(),
            orgs,
        };
        viz::compute(&ctx, spec, &extra).map_err(|e| e.message)
    }

    /// The orgs the open panel is about (the scope accent's members while it is open).
    pub fn viz_orgs(&self) -> Option<(&'static str, Vec<String>)> {
        let v = self.ui.viz.as_ref()?;
        Some(match &v.spec {
            Spec::Bridge { pair, .. } => {
                let p = self
                    .source
                    .ic_config
                    .get("intercompany")
                    .and_then(|i| i.get("pairs"))
                    .and_then(|p| p.as_array())
                    .and_then(|a| a.iter().find(|x| x["id"].as_str() == Some(pair)))?;
                (
                    "pair",
                    vec![
                        p["a"]["org"].as_str().unwrap_or("").to_string(),
                        p["b"]["org"].as_str().unwrap_or("").to_string(),
                    ],
                )
            }
            Spec::Flow { orgs, .. } => {
                let all: Vec<String> = self.model.orgs.iter().map(|o| o.alias.clone()).collect();
                let v: Vec<String> = match orgs {
                    Some(w) => all.into_iter().filter(|a| w.contains(a)).collect(),
                    None => all,
                };
                (if v.len() == 2 { "pair" } else { "compare" }, v)
            }
            Spec::TAccount { org, .. }
            | Spec::Timeline { org, .. }
            | Spec::BeforeAfter { org, .. } => ("org", vec![org.clone()]),
            Spec::Checklist { .. } => return None,
        })
    }

    /// An org's accent token while the panel is open: its place among the panel's orgs (`member.N`).
    pub fn viz_org_token(&self, alias: &str) -> Token {
        let members = self.accent().members;
        match members.iter().position(|m| m.alias == alias) {
            Some(0) => Token::Member1,
            Some(1) => Token::Member2,
            Some(2) => Token::Member3,
            Some(3) => Token::Member4,
            _ => Token::Accent,
        }
    }

    /// Keys while a panel is open (true = handled). Esc closes, the row keys scroll, a tab or scope change
    /// closes it and goes on; `T` / `M` on a Ledger account open its T-account / timeline.
    pub(crate) fn viz_action(&mut self, action: &str, _fx: &mut Vec<Effect>) -> bool {
        match action {
            "viz.t_account" | "viz.timeline" => {
                match self.ledger_account_spec(action == "viz.timeline") {
                    Some(s) => self.viz_open(s),
                    None => {
                        self.ui.note = Some(
                            "Select an account first (Ledger trial balance or postings).".into(),
                        )
                    }
                }
                return true;
            }
            _ => {}
        }
        if self.ui.viz.is_none() {
            return false;
        }
        let max = if action.starts_with("row.") || action.starts_with("page.") {
            self.viz_max_scroll()
        } else {
            0
        };
        let page = self.ui.size.1.saturating_sub(10).max(1);
        let scroll = |app: &mut App, f: &dyn Fn(u16) -> u16| {
            if let Some(o) = app.ui.viz.as_mut() {
                o.scroll = f(o.scroll).min(max);
            }
        };
        match action {
            "drill.back" => {
                self.viz_close();
                true
            }
            "row.prev" => {
                scroll(self, &|s| s.saturating_sub(1));
                true
            }
            "row.next" => {
                scroll(self, &|s| s.saturating_add(1));
                true
            }
            "page.prev" => {
                scroll(self, &|s| s.saturating_sub(page));
                true
            }
            "page.next" => {
                scroll(self, &|s| s.saturating_add(page));
                true
            }
            "row.first" => {
                scroll(self, &|_| 0);
                true
            }
            "row.last" => {
                scroll(self, &|_| u16::MAX);
                true
            }
            "drill.in" | "cell.prev" | "cell.next" => true,
            "filter.open" => {
                self.ui.note =
                    Some("A filter doesn't apply on an explainer panel (Esc closes it).".into());
                true
            }
            a if a.starts_with("tab.") || a.starts_with("scope.") || a.starts_with("history.") => {
                self.viz_close();
                false
            }
            _ => false,
        }
    }

    /// How far the open panel's body scrolls (from its last offscreen render at the current size).
    pub fn viz_max_scroll(&self) -> u16 {
        let (w, h) = self.ui.size;
        let Some(Ok(m)) = self.viz_model() else {
            return 0;
        };
        let mut buf = Buffer::empty(Rect::new(0, 0, w, h));
        let area = Rect::new(0, 4, w, h.saturating_sub(5));
        let tok = |a: &str| self.viz_org_token(a);
        let p = crate::widgets::viz::draw(m, &self.ui.theme, &tok, &mut buf, area, 0);
        p.rows.saturating_sub(p.visible)
    }

    /// The selected Ledger account as a T-account or timeline spec (from, to: the postings range).
    pub fn ledger_account_spec(&self, timeline: bool) -> Option<Spec> {
        if self.ui.tab != Tab::Ledger || self.cmp_active() {
            return None;
        }
        let org = self.ledger_org()?;
        let alias = self.model.orgs[org].alias.clone();
        let nav = self.nav();
        let account = match self.ledger_level() {
            1 => {
                let m = self.l1_model()?;
                let vis = self.l1_visible();
                let r = &m.rows[*vis.get(self.l1_cursor()?)?];
                if r.code.is_empty() {
                    r.account_id.clone()
                } else {
                    r.code.clone()
                }
            }
            _ => {
                let id = nav.account.clone()?;
                self.l1_model()
                    .and_then(|m| m.rows.iter().find(|r| r.account_id == id))
                    .map(|r| r.code.clone())
                    .filter(|c| !c.is_empty())
                    .unwrap_or(id)
            }
        };
        let from = nav.since.clone();
        let to = Some(self.ui.as_of.clone());
        Some(if timeline {
            Spec::Timeline {
                org: alias,
                account,
                from,
                to,
            }
        } else {
            Spec::TAccount {
                org: alias,
                account,
                from,
                to,
                top: None,
            }
        })
    }

    /// The palette's "Explain: …" entries: each configured pair's bridge, the money flow (owed and paid),
    /// the selected account's T-account and timeline, the shown manifest's effect per org it touches, and
    /// the close worklist when the config names one (`tui.checklist`).
    pub fn viz_palette(&self) -> Vec<PalEntry> {
        let mut out = vec![];
        let entry = |label: String, spec: Spec| PalEntry {
            label,
            keys: None,
            network: false,
            suffix: None,
            run: PalRun::Viz(Box::new(spec)),
            disabled: false,
        };
        let pairs = self
            .source
            .ic_config
            .get("intercompany")
            .and_then(|i| i.get("pairs"))
            .and_then(|p| p.as_array())
            .cloned()
            .unwrap_or_default();
        for p in &pairs {
            let id = p["id"].as_str().unwrap_or("").to_string();
            let label = p["label"]
                .as_str()
                .map(str::to_string)
                .unwrap_or_else(|| id.clone());
            out.push(entry(
                format!("Explain: bridge {label}"),
                Spec::Bridge {
                    pair: id,
                    as_of: None,
                },
            ));
        }
        out.push(entry(
            "Explain: who owes whom (money flow)".into(),
            Spec::Flow {
                orgs: None,
                as_of: None,
                from: None,
                to: None,
                basis: viz::FlowBasis::Owed,
            },
        ));
        out.push(entry(
            "Explain: what moved between the orgs this year".into(),
            Spec::Flow {
                orgs: None,
                as_of: None,
                from: None,
                to: None,
                basis: viz::FlowBasis::Paid,
            },
        ));
        if let Some(s) = self.ledger_account_spec(false) {
            if let Spec::TAccount { account, org, .. } = &s {
                out.push(entry(
                    format!("Explain: T-account of {org} {account}"),
                    s.clone(),
                ));
            }
            if let Some(tl @ Spec::Timeline { .. }) = self.ledger_account_spec(true)
                && let Spec::TAccount { account, org, .. } = &s
            {
                out.push(entry(
                    format!("Explain: {org} {account} month by month"),
                    tl,
                ));
            }
        }
        if let (Some(name), Some(f)) = (self.ui.manifest.as_ref(), self.manifest_facts())
            && let Ok(d) = &f.diff
        {
            for o in &d.orgs {
                out.push(entry(
                    format!("Explain: what {name} changes in {}", o.org),
                    Spec::BeforeAfter {
                        manifest: Some(name.clone()),
                        org: o.org.clone(),
                    },
                ));
            }
        }
        if let Some(file) = &self.source.checklist {
            out.push(entry(
                "Explain: the close worklist".into(),
                Spec::Checklist { file: file.clone() },
            ));
        }
        out
    }
}

/// Breadcrumb segments while a panel is open: where it opened, then `Explain › <kind>`.
pub fn crumbs(app: &App, under: Vec<String>) -> Vec<String> {
    let mut v = under;
    let kind = app.ui.viz.as_ref().map_or("", |o| match o.spec {
        Spec::Bridge { .. } => "bridge",
        Spec::Flow { .. } => "money flow",
        Spec::TAccount { .. } => "T-account",
        Spec::Timeline { .. } => "timeline",
        Spec::BeforeAfter { .. } => "before → after",
        Spec::Checklist { .. } => "worklist",
    });
    v.push(format!("Explain: {kind}"));
    v
}

/// The footer's status line while a panel is open.
pub fn status(app: &App) -> String {
    match app.viz_model() {
        Some(Ok(m)) => {
            let n = m.ids().len();
            format!("{} · {n} parts · Esc closes", m.title())
        }
        Some(Err(_)) => "Explain: could not compute · Esc closes".into(),
        None => "Esc closes".into(),
    }
}

pub fn draw(app: &App, buf: &mut Buffer, area: Rect, hits: &mut Hits) {
    let th = &app.ui.theme;
    let Some(open) = app.ui.viz.as_ref() else {
        return;
    };
    hits.content = Some(area);
    match app.viz_model() {
        Some(Ok(m)) => {
            let tok = |a: &str| app.viz_org_token(a);
            let p = crate::widgets::viz::draw(m, th, &tok, buf, area, open.scroll);
            for (id, r) in &p.anchors {
                hits.anchors.viz(id, *r);
            }
            for (id, off) in &p.off {
                hits.anchors.put_off(&format!("viz:{id}"), *off);
            }
            hits.anchors.panel("viz", area);
        }
        Some(Err(e)) => {
            let (body, _) = crate::widgets::viz::head(th, buf, area, "Explain", &[]);
            let inner = Rect::new(
                body.x + 2,
                body.y,
                body.width.saturating_sub(4),
                body.height,
            );
            let lines = crate::widgets::viz::wrapped(
                th,
                &format!("Could not compute: {e}"),
                inner.width,
                Token::Error,
            );
            let mut b = crate::widgets::viz::Body::new(inner, 0, lines.len() as u16);
            b.prose(buf, th, inner.x, &lines);
        }
        None => crate::views::placeholder(app, buf, area, "Explain: preparing…"),
    }
}
