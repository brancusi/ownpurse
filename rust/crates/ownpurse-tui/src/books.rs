//! The books the views show, computed with core and cached (spec §14): ledgers per org, bases per
//! (org, as-of, basis), intercompany results per as-of. A `Books` lives for one record version: `r` (reload)
//! opens the record again and starts a fresh `Books`, so every cache is keyed by the record version too.
//! All reads are local SQLite through core's read-only `Record`; nothing here can reach Xero.

use crate::clock::plural;
use crate::model::{Model, Source};

/// `≈ 15 calls` / `calls: unknown` (never synced: no basis for an estimate).
/// The attention line's tail after "⌃P → Sync <org>": ` (≈ 15 calls).`, ` (calls: unknown).`, or for core's
/// first-sync floor ` · ≈ 15+ calls, first full sync.`.
pub fn est_attn(e: Option<ownpurse_core::estimate::Estimate>) -> String {
    match e {
        Some(e @ ownpurse_core::estimate::Estimate::FirstSyncFloor { .. }) => {
            let l = e.label();
            let l = match l.split_once(" (") {
                Some((head, note)) => format!("{head}, {}", note.trim_end_matches(')')),
                None => l,
            };
            format!(" · {l}.")
        }
        other => format!(" ({}).", est_label(other)),
    }
}

/// The sync estimate as the confirm shows it (core `estimate_or_floor`; one number everywhere).
pub fn est_label(e: Option<ownpurse_core::estimate::Estimate>) -> String {
    e.map_or("calls: unknown".into(), |e| e.label())
}

use ownpurse_core::balances::{Basis, BasisKind, Mode, resolve};
use ownpurse_core::intercompany::{self as icm, IcContext, IcError};
use ownpurse_core::{Ledger, Record};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

pub type Shared<T> = Arc<Result<T, String>>;

pub struct Books {
    record: Record,
    /// The manifest side: None on main. Set when the books open on a manifest; a manifest change opens
    /// fresh caches, so every cache below is keyed by the manifest revision too.
    overlay: Option<Arc<ownpurse_core::view::Overlay>>,
    ledgers: HashMap<String, Result<Arc<Ledger>, String>>,
    /// On a manifest: main's ledgers and bases too (parity and the Δ marks compare against them).
    main_ledgers: HashMap<String, Result<Arc<Ledger>, String>>,
    main_bases: HashMap<(String, String, bool), Shared<Basis>>,
    bases: HashMap<(String, String, bool), Shared<Basis>>,
    ic: HashMap<String, Shared<Value>>,
    tb_fetched: HashMap<(String, String), Option<String>>,
    /// Ledger views (crate::ledgers): L1 per (org, as-of, history), L2 per (org, account, as-of, since,
    /// history), documents per (org, source, id), postings indexed by account per org.
    pub(crate) l1: HashMap<(usize, String, bool), Shared<crate::ledgers::L1Model>>,
    pub(crate) l2: HashMap<(usize, String, String, String, bool), Shared<crate::ledgers::L2Model>>,
    pub(crate) docs: HashMap<(usize, String, String), Shared<crate::ledgers::DocModel>>,
    pub(crate) by_account: HashMap<usize, Arc<HashMap<String, Vec<usize>>>>,
}

impl Books {
    pub fn open(source: &Source) -> Result<Books, String> {
        let record = Record::open(&source.record).map_err(|e| e.to_string())?;
        Ok(Books {
            record,
            overlay: None,
            ledgers: HashMap::new(),
            main_ledgers: HashMap::new(),
            main_bases: HashMap::new(),
            bases: HashMap::new(),
            ic: HashMap::new(),
            tb_fetched: HashMap::new(),
            l1: HashMap::new(),
            l2: HashMap::new(),
            docs: HashMap::new(),
            by_account: HashMap::new(),
        })
    }

    pub fn record(&self) -> &Record {
        &self.record
    }

    /// Main, or main + the manifest these books are open on: the one way the views read documents.
    pub fn view(&self) -> ownpurse_core::view::View<'_> {
        ownpurse_core::view::View::from_parts(&self.record, self.overlay.clone())
    }

    /// Open the manifest side (fresh caches per manifest revision).
    pub fn set_manifest(
        &mut self,
        ctx: &ownpurse_core::view::Ctx,
        manifest: Option<&ownpurse_core::manifest::Manifest>,
    ) -> Result<(), String> {
        let v = ownpurse_core::view::View::open(&self.record, ctx, manifest)
            .map_err(|e| e.to_string())?;
        self.overlay = v.overlay_arc();
        // every cache was computed for the previous manifest side
        self.ledgers.clear();
        self.bases.clear();
        self.ic.clear();
        self.l1.clear();
        self.l2.clear();
        self.docs.clear();
        self.by_account.clear();
        Ok(())
    }

    pub fn overlay(&self) -> Option<&ownpurse_core::view::Overlay> {
        self.overlay.as_deref()
    }

    /// The org's ledger, built once per record version.
    pub fn ledger(&mut self, model: &Model, org: usize) -> Result<Arc<Ledger>, String> {
        let o = &model.orgs[org];
        if let Some(l) = self.ledgers.get(&o.alias) {
            return l.clone();
        }
        let l = Ledger::build(&self.view(), &o.tenant)
            .map(Arc::new)
            .map_err(|e| e.to_string());
        self.ledgers.insert(o.alias.clone(), l.clone());
        l
    }

    /// Main's ledger for an org (the same as `ledger` on main).
    pub fn main_ledger(&mut self, model: &Model, org: usize) -> Result<Arc<Ledger>, String> {
        if self.overlay.is_none() {
            return self.ledger(model, org);
        }
        let o = &model.orgs[org];
        if let Some(l) = self.main_ledgers.get(&o.alias) {
            return l.clone();
        }
        let l = Ledger::build(&ownpurse_core::view::View::main(&self.record), &o.tenant)
            .map(Arc::new)
            .map_err(|e| e.to_string());
        self.main_ledgers.insert(o.alias.clone(), l.clone());
        l
    }

    /// Main's basis (the same as `basis` on main).
    pub fn main_basis(
        &mut self,
        model: &Model,
        org: usize,
        as_of: &str,
        history: bool,
    ) -> Shared<Basis> {
        if self.overlay.is_none() {
            return self.basis(model, org, as_of, history);
        }
        let key = (model.orgs[org].alias.clone(), as_of.to_string(), history);
        if let Some(b) = self.main_bases.get(&key) {
            return b.clone();
        }
        let o = &model.orgs[org];
        let mode = if history { Mode::History } else { Mode::Auto };
        let b = Arc::new(self.main_ledger(model, org).and_then(|l| {
            resolve(&self.record, &l, &o.tenant, as_of, o.fy_end, mode).map_err(|e| e.to_string())
        }));
        self.main_bases.insert(key, b.clone());
        b
    }

    /// The basis for (org, as-of, full history?), resolved once.
    pub fn basis(
        &mut self,
        model: &Model,
        org: usize,
        as_of: &str,
        history: bool,
    ) -> Shared<Basis> {
        let key = (model.orgs[org].alias.clone(), as_of.to_string(), history);
        if let Some(b) = self.bases.get(&key) {
            return b.clone();
        }
        let o = &model.orgs[org];
        let mode = if history { Mode::History } else { Mode::Auto };
        let b = Arc::new(self.ledger(model, org).and_then(|l| {
            resolve(&self.record, &l, &o.tenant, as_of, o.fy_end, mode).map_err(|e| e.to_string())
        }));
        self.bases.insert(key, b.clone());
        b
    }

    /// When the cached Xero TB at `as_of` was fetched (`YYYY-MM-DD HH:MM`), if there is one.
    pub fn tb_fetched(&mut self, model: &Model, org: usize, as_of: &str) -> Option<String> {
        let o = &model.orgs[org];
        let key = (o.alias.clone(), as_of.to_string());
        if let Some(v) = self.tb_fetched.get(&key) {
            return v.clone();
        }
        let v = self
            .record
            .tb_snapshot(&o.tenant, as_of)
            .ok()
            .flatten()
            .map(|r| {
                r.fetched_at
                    .get(..16)
                    .unwrap_or(&r.fetched_at)
                    .replace('T', " ")
            });
        self.tb_fetched.insert(key, v.clone());
        v
    }

    /// The intercompany engine's result at `as_of` (every configured pair), using the cached ledgers.
    pub fn ic(&mut self, model: &Model, source: &Source, as_of: &str) -> Shared<Value> {
        if let Some(r) = self.ic.get(as_of) {
            return r.clone();
        }
        for i in 0..model.orgs.len() {
            let _ = self.ledger(model, i);
        }
        let ledgers: HashMap<String, Arc<Ledger>> = self
            .ledgers
            .iter()
            .filter_map(|(k, v)| v.as_ref().ok().map(|l| (k.clone(), l.clone())))
            .collect();
        let tenants: HashMap<String, String> = source.orgs.iter().cloned().collect();
        let r = {
            let ctx = IcContext::new(self.view(), tenants, &source.ic_config, as_of).with_ledgers(
                move |org: &str| {
                    ledgers
                        .get(org)
                        .cloned()
                        .ok_or_else(|| IcError::Config(format!("no ledger for {org}")))
                },
            );
            Arc::new(icm::build(&ctx, None).map_err(|e| e.to_string()))
        };
        self.ic.insert(as_of.to_string(), r.clone());
        r
    }
}

// ── Overview (tab 1) ─────────────────────────────────────────────────────────────────────────────

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Parity {
    Exact,
    Differs(usize),
    /// No cached Xero TB to compare with (or, for the year basis, no opening TB), or never synced.
    Unchecked,
    Error(String),
}

impl Parity {
    pub fn label(&self) -> String {
        match self {
            Parity::Exact => "✓ exact".into(),
            Parity::Differs(n) if *n == 1 => "1 account differs".into(),
            Parity::Differs(n) => format!("{} differ", plural(*n as i64, "account", "accounts")),
            Parity::Unchecked => "— not checked".into(),
            Parity::Error(e) => format!("error: {e}"),
        }
    }
    pub fn short(&self) -> String {
        match self {
            Parity::Differs(n) if *n == 1 => "1 differs".into(),
            Parity::Differs(n) => format!("{n} differ"),
            other => other.label(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OvRow {
    pub org: usize,
    /// The org has a local record (else issues are unknown: `—`).
    pub synced: bool,
    pub year: Parity,
    pub history: Parity,
    pub unbalanced: usize,
    pub uncoded: usize,
    pub tb_fetched: Option<String>,
}

impl OvRow {
    pub fn issues(&self) -> String {
        if !self.synced {
            return "—".into();
        }
        let mut v = Vec::new();
        if self.unbalanced > 0 {
            v.push(format!("{} unbalanced", self.unbalanced));
        }
        if self.uncoded > 0 {
            v.push(format!("{} uncoded", self.uncoded));
        }
        if v.is_empty() {
            "none".into()
        } else {
            v.join(" · ")
        }
    }
    pub fn issues_short(&self) -> String {
        if !self.synced {
            return "—".into();
        }
        let mut v = Vec::new();
        if self.unbalanced > 0 {
            v.push(format!("{} unbal", self.unbalanced));
        }
        if self.uncoded > 0 {
            v.push(format!("{} uncoded", self.uncoded));
        }
        if v.is_empty() {
            "none".into()
        } else {
            v.join(" · ")
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IcLine {
    /// `3 pairs · 1 match · 2 bridged · 0 unexplained [· n not compared] [· n not supported]`
    pub parts: Vec<String>,
    pub unexplained: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OverviewModel {
    pub as_of: String,
    /// The clock it was built at (freshness lines depend on it): rebuilt when the minute changes, so a
    /// TUI built from the state draws the same lines.
    pub now: i64,
    pub rows: Vec<OvRow>,
    pub ic: Option<IcLine>,
    /// Needs-attention lines (each starts with `!`).
    pub attention: Vec<String>,
}

fn year_parity(b: &Shared<Basis>) -> Parity {
    match b.as_ref() {
        Err(e) => Parity::Error(e.clone()),
        Ok(b) if b.kind != BasisKind::XeroOpening => Parity::Unchecked,
        Ok(b) => parity_of(b),
    }
}

fn parity_of(b: &Basis) -> Parity {
    match b.matches_xero() {
        None => Parity::Unchecked,
        Some(true) => Parity::Exact,
        Some(false) => Parity::Differs(b.diffs.len()),
    }
}

fn count(v: &Value, k: &str) -> usize {
    v.get(k).and_then(Value::as_u64).unwrap_or(0) as usize
}

impl OverviewModel {
    pub fn build(
        books: &mut Books,
        model: &Model,
        source: &Source,
        as_of: &str,
        now: i64,
    ) -> OverviewModel {
        let mut rows = Vec::new();
        let mut attention = Vec::new();
        for (i, o) in model.orgs.iter().enumerate() {
            let fresh = model.freshness(&[i], now);
            let synced = o.last_sync.is_some();
            let (year, history, unbalanced, uncoded) = if synced {
                let y = books.basis(model, i, as_of, false);
                let h = books.basis(model, i, as_of, true);
                let (ub, uc) = match books.ledger(model, i) {
                    Ok(l) => (
                        l.unbalanced.len(),
                        l.postings
                            .iter()
                            .filter(|p| p.account_id.starts_with("(uncoded)"))
                            .count(),
                    ),
                    Err(_) => (0, 0),
                };
                let hp = match h.as_ref() {
                    Ok(b) => parity_of(b),
                    Err(e) => Parity::Error(e.clone()),
                };
                (year_parity(&y), hp, ub, uc)
            } else {
                (Parity::Unchecked, Parity::Unchecked, 0, 0)
            };
            match fresh.level {
                crate::model::Level::Err => attention.push(format!(
                    "! No local record for {a} yet. ⌃P → Sync {a}{}",
                    est_attn(o.sync_estimate),
                    a = o.alias
                )),
                crate::model::Level::Warn => attention.push(format!(
                    "! {a} record is {} old (TTL {}). ⌃P → Sync {a}{}",
                    crate::clock::ago(fresh.age),
                    crate::clock::ago(model.ttl),
                    est_attn(o.sync_estimate),
                    a = o.alias
                )),
                crate::model::Level::Ok => {}
            }
            if o.left < model.warn {
                attention.push(if o.left_known {
                    format!(
                        "! {} has {} left.",
                        o.alias,
                        plural(o.left, "call", "calls")
                    )
                } else {
                    format!(
                        "! {} has ≈ {} left (estimated).",
                        o.alias,
                        plural(o.left, "call", "calls")
                    )
                });
            }
            if let Parity::Differs(n) = year {
                attention.push(format!(
                    "! {} {} activity differs from Xero in {}.",
                    o.alias,
                    as_of.get(..4).unwrap_or(as_of),
                    plural(n as i64, "account", "accounts")
                ));
            }
            if unbalanced > 0 {
                attention.push(format!(
                    "! {} has {} unbalanced.",
                    o.alias,
                    plural(unbalanced as i64, "document", "documents")
                ));
            }
            if uncoded > 0 {
                attention.push(format!("! {} has uncoded postings.", o.alias));
            }
            rows.push(OvRow {
                org: i,
                synced,
                year,
                history,
                unbalanced,
                uncoded,
                tb_fetched: if synced {
                    books.tb_fetched(model, i, as_of)
                } else {
                    None
                },
            });
        }
        let ic = if source.has_pairs() {
            match books.ic(model, source, as_of).as_ref() {
                Ok(r) => {
                    let sm = &r["summary"];
                    let mut parts = vec![
                        plural(count(sm, "pairs") as i64, "pair", "pairs"),
                        plural(count(sm, "matches") as i64, "match", "matches"),
                        format!("{} bridged", count(sm, "bridged")),
                        format!("{} unexplained", count(sm, "unexplained")),
                    ];
                    for k in ["not compared", "not supported"] {
                        if count(sm, k) > 0 {
                            parts.push(format!("{} {k}", count(sm, k)));
                        }
                    }
                    let pairs = r["pairs"].as_array().cloned().unwrap_or_default();
                    let mut by_date: Vec<(String, Vec<&Value>)> = Vec::new();
                    for p in &pairs {
                        let label = p["label"].as_str().unwrap_or("");
                        match p["status"].as_str().unwrap_or("") {
                            "unexplained" => attention.push(format!(
                                "! {label} has an unexplained residual of {}. Open Intercompany (3).",
                                p["residual"]
                                    .as_str()
                                    .and_then(|s| ownpurse_core::money::parse(s).ok())
                                    .map(ownpurse_core::fmt)
                                    .unwrap_or_default()
                            )),
                            "not supported" => attention.push(format!(
                                "! {label} not supported: {}",
                                p["reason"].as_str().unwrap_or("")
                            )),
                            "not compared" => {
                                let day = p["missing_tb"]["date"].as_str().unwrap_or("").to_string();
                                match by_date.iter_mut().find(|(d, _)| *d == day) {
                                    Some((_, g)) => g.push(p),
                                    None => by_date.push((day, vec![p])),
                                }
                            }
                            _ => {}
                        }
                    }
                    for (day, group) in by_date {
                        let mut orgs: Vec<&str> = group
                            .iter()
                            .flat_map(|p| p["missing_tb"]["orgs"].as_array().into_iter().flatten())
                            .filter_map(Value::as_str)
                            .collect();
                        orgs.sort_unstable();
                        orgs.dedup();
                        let who = if group.len() == 1 {
                            group[0]["label"].as_str().unwrap_or("").to_string()
                        } else {
                            plural(group.len() as i64, "pair", "pairs")
                        };
                        let n = orgs.len();
                        attention.push(format!(
                            "! {who} not compared: no cached Xero TB at {day}. ⌃P → Refetch (≈ {})",
                            plural(n as i64, "call", "calls")
                        ));
                    }
                    Some(IcLine {
                        parts,
                        unexplained: count(sm, "unexplained"),
                    })
                }
                Err(e) => {
                    attention.push(format!("! Intercompany could not be computed: {e}"));
                    None
                }
            }
        } else {
            None
        };
        OverviewModel {
            as_of: as_of.to_string(),
            now,
            rows,
            ic,
            attention,
        }
    }
}
