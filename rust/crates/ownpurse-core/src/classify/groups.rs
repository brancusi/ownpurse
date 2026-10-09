//! The Groups read model (spec §3.E): a group's members in origin → mirror → settlement order, the chain check
//! (one-to-one, many-to-one, gaps; never a line-level claim for many-to-one, §10.9), the running balance of a
//! settlement group, the config-driven pair rule (`[groups.pair_rules.<id>]`) checked on the record's ledger
//! balances, and member suggestions (same amount within ±7 days in another org).

use super::Result;
use super::anchor::Anchor;
use super::config::{PairRule, PairSide, PairSign};
use super::store::{Group, NoteTarget};
use super::view::{NoteView, Workspace};
use crate::money::fmt;
use rust_decimal::Decimal;
use std::collections::HashMap;

/// One member row: `role · org · date · account · amount · description`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemberView {
    pub anchor: Anchor,
    pub role: Option<String>,
    pub org: String,
    pub date: String,
    pub account: String,
    pub account_name: String,
    pub amount: Decimal,
    pub payee: String,
    pub description: String,
    /// The line is no longer in the record (the member is kept, its facts are blank).
    pub missing: bool,
}

/// The chain check of a group.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChainCheck {
    /// One origin, one mirror that cancel (signed: `origin + mirror = 0`, the mirror books the other side).
    /// `same_sign` is always false here (kept for compatibility).
    OneToOne {
        origin: Decimal,
        mirror: Decimal,
        same_sign: bool,
    },
    /// Something is missing or differs: `mirror missing`, `origin missing`, `amount differs by 12.34`.
    Gap { reason: String },
    /// N origins against one mirror: only the share of the mirror their SIGNED net explains, never a line-level
    /// match.
    ManyToOne {
        origins: usize,
        /// The signed net of the origins.
        origin_sum: Decimal,
        /// The mirror, signed.
        mirror: Decimal,
        /// Percent of the mirror the net explains (`|net| / |mirror|`), one decimal.
        share_pct: Decimal,
    },
    /// No origin or mirror roles (e.g. a split or a trip): nothing to check.
    NotApplicable,
}

impl ChainCheck {
    /// The detail-pane lines: a primary line and, for many-to-one, a second (the share). E.g.
    /// `origin 1,148.63 = mirror 1,148.63 ✓`; `mirror missing`;
    /// `2 origins net (682.21) → 1 mirror (2,764.18)` +
    /// `explains 24.7% of the mirror · no per-line match` (one phrase at every width).
    pub fn text(&self) -> (String, Option<String>) {
        match self {
            ChainCheck::OneToOne { origin, mirror, .. } => (
                format!(
                    "origin {} = mirror {} ✓",
                    fmt(origin.abs()),
                    fmt(mirror.abs())
                ),
                None,
            ),
            ChainCheck::Gap { reason } => (reason.clone(), None),
            ChainCheck::ManyToOne {
                origins,
                origin_sum,
                mirror,
                share_pct,
            } => (
                format!(
                    "{origins} origins net {} → 1 mirror {}",
                    fmt(*origin_sum),
                    fmt(*mirror)
                ),
                Some(format!(
                    "explains {share_pct}% of the mirror · no per-line match"
                )),
            ),
            ChainCheck::NotApplicable => {
                ("no origin or mirror roles: nothing to check".into(), None)
            }
        }
    }
    pub fn ok(&self) -> bool {
        matches!(self, ChainCheck::OneToOne { .. })
    }
}

/// One settlement in a running balance. The two legs of one settlement (same size, opposite signs, different
/// orgs, within `[groups] settle_window_days`, default the same date) are ONE step; a leg without its partner
/// is a step of its own, marked `lone_org`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BalanceStep {
    /// The first leg (by date).
    pub anchor: Anchor,
    pub date: String,
    /// The settlement's size.
    pub amount: Decimal,
    pub before: Decimal,
    pub after: Decimal,
    /// Every leg of this settlement (one or two).
    pub legs: Vec<Anchor>,
    /// A lone leg: the org whose leg this is (its counterparty's leg is not in the group).
    pub lone_org: Option<String>,
    /// The first leg's date (`date` is the later leg's, when the settlement is applied). Equal for a lone leg.
    pub start_date: String,
}

impl BalanceStep {
    /// `2025-08-08 settled 1,148.63  1,148.63 → 0.00`, `2025-07-15 settled (in transit from 07-14) …`, or
    /// `… settled (roast leg only) …`.
    pub fn text(&self) -> String {
        let what = match &self.lone_org {
            Some(o) => format!("settled ({o} leg only)"),
            None if self.start_date != self.date => {
                format!("settled (in transit from {})", md(&self.start_date))
            }
            None => "settled".into(),
        };
        format!(
            "{} {what} {}  {} → {}",
            self.date,
            fmt(self.amount),
            fmt(self.before),
            fmt(self.after)
        )
    }
}

/// Everything the Groups detail shows.
#[derive(Debug, Clone, PartialEq)]
pub struct GroupDetail {
    pub group: Group,
    /// origin → mirror → settlement → no role, then by date.
    pub members: Vec<MemberView>,
    pub check: ChainCheck,
    /// Settlement groups: the open amount (Σ origins, else the mirror) before and after each settlement, by
    /// date. Empty without settlements.
    pub running: Vec<BalanceStep>,
    /// The pair rule that applies (both its orgs are among the members), if any. Check it with
    /// [`Workspace::pair_check`].
    pub pair_rule: Option<String>,
    pub notes: Vec<NoteView>,
}

/// One date of a pair-rule check.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PairRow {
    /// `before <anchor>` / `after <anchor>`.
    pub label: String,
    pub date: String,
    pub a: Decimal,
    pub b: Decimal,
    /// `a + b` (opposite) or `a - b` (same): zero when the rule holds.
    pub difference: Decimal,
    pub ok: bool,
    /// Why it differs: `in transit 07-14 → 07-15` (expected while a settlement's second leg is on its way;
    /// `ok` stays true), `hold leg of this settlement is missing`, `… (it posts on 2025-07-15)`, or
    /// `difference unexplained`. `None` when the rule holds.
    pub reason: Option<String>,
}

impl PairRow {
    /// `= 0.00 ✓` or `≠ 3,281.19: hold leg of this settlement is missing`.
    pub fn text(&self) -> String {
        match &self.reason {
            None => format!("= {} ✓", fmt(self.difference)),
            Some(r) => format!("≠ {}: {r}", fmt(self.difference.abs())),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PairCheck {
    pub rule: String,
    pub title: String,
    pub rows: Vec<PairRow>,
}

/// A candidate member: same amount (by size) within ±7 days, in another org.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemberSuggestion {
    pub anchor: Anchor,
    pub org: String,
    pub date: String,
    pub amount: Decimal,
    pub payee: String,
    pub description: String,
    pub days_apart: i64,
}

fn role_rank(r: Option<&str>) -> u8 {
    match r {
        Some("origin") => 0,
        Some("mirror") => 1,
        Some("settlement") => 2,
        _ => 3,
    }
}

fn days(d: &str) -> Option<i64> {
    crate::dates::parse_iso(d).map(|(y, m, dd)| crate::dates::days_from_civil(y, m, dd))
}

impl Workspace {
    /// The Groups detail for one group.
    pub fn group_detail(&self, id: &str) -> Option<GroupDetail> {
        let group = self.store().state().groups.get(id)?.clone();
        let mut members: Vec<MemberView> = group
            .members
            .iter()
            .map(|m| match self.line(&m.anchor) {
                Some(l) => MemberView {
                    anchor: m.anchor.clone(),
                    role: m.role.clone(),
                    org: l.org.clone(),
                    date: l.date.clone(),
                    account: l.account_code.clone(),
                    account_name: l.account_name.clone(),
                    amount: l.amount,
                    payee: l.payee.clone(),
                    description: l.description.clone(),
                    missing: false,
                },
                None => MemberView {
                    anchor: m.anchor.clone(),
                    role: m.role.clone(),
                    org: m.anchor.org().unwrap_or("").to_string(),
                    date: String::new(),
                    account: String::new(),
                    account_name: String::new(),
                    amount: Decimal::ZERO,
                    payee: String::new(),
                    description: String::new(),
                    missing: true,
                },
            })
            .collect();
        members.sort_by(|a, b| {
            role_rank(a.role.as_deref())
                .cmp(&role_rank(b.role.as_deref()))
                .then_with(|| a.date.cmp(&b.date))
                .then_with(|| a.anchor.cmp(&b.anchor))
        });
        let with = |r: &str| -> Vec<&MemberView> {
            members
                .iter()
                .filter(|m| m.role.as_deref() == Some(r) && !m.missing)
                .collect()
        };
        let (origins, mirrors, settlements) = (with("origin"), with("mirror"), with("settlement"));
        let check = chain_check(&origins, &mirrors);
        let running = running_balance(
            &origins,
            &mirrors,
            &settlements,
            self.config().groups.settle_window_days,
        );
        let orgs: Vec<&str> = members.iter().map(|m| m.org.as_str()).collect();
        let pair_rule = self
            .config()
            .groups
            .pair_rules
            .iter()
            .find(|r| orgs.contains(&r.a.org.as_str()) && orgs.contains(&r.b.org.as_str()))
            .map(|r| r.id.clone());
        let notes = self
            .store()
            .state()
            .notes
            .iter()
            .filter(|n| n.target == NoteTarget::Group(id.to_string()))
            .map(|n| NoteView {
                event_id: n.event_id,
                at: n.at.clone(),
                actor: n.actor.clone(),
                text: n.text.clone(),
            })
            .collect();
        Some(GroupDetail {
            group,
            members,
            check,
            running,
            pair_rule,
            notes,
        })
    }

    /// Check a group's pair rule on the record's ledger balances (Xero opening + activity, as the Ledger tab),
    /// before and after each settlement. `orgs` maps aliases to tenant ids. `None` when no rule applies or the
    /// group has no settlements.
    pub fn pair_check(
        &self,
        id: &str,
        record: &crate::view::View<'_>,
        orgs: &[(String, String)],
    ) -> Result<Option<PairCheck>> {
        let Some(d) = self.group_detail(id) else {
            return Ok(None);
        };
        let Some(rule) = d
            .pair_rule
            .as_ref()
            .and_then(|r| self.config().groups.pair_rules.iter().find(|x| &x.id == r))
        else {
            return Ok(None);
        };
        if d.running.is_empty() {
            return Ok(None);
        }
        let mut cache: HashMap<String, crate::Ledger> = HashMap::new();
        let mut rows = vec![];
        let mut seen: Vec<String> = vec![];
        for step in &d.running {
            // before the first leg, in transit (after the first leg, when the second lands later), after the
            // second; a date already evaluated is not repeated
            let prev = crate::dates::add_days(&step.start_date, -1).unwrap_or_default();
            let mut points = vec![(format!("before {}", step.anchor), prev, false)];
            if step.start_date != step.date {
                points.push((
                    format!("in transit {}", step.anchor),
                    step.start_date.clone(),
                    true,
                ));
            }
            points.push((format!("after {}", step.anchor), step.date.clone(), false));
            for (label, date, transit) in points {
                if seen.contains(&date) {
                    continue;
                }
                seen.push(date.clone());
                let a = side_balance(record, orgs, &rule.a, &date, &mut cache)?;
                let b = side_balance(record, orgs, &rule.b, &date, &mut cache)?;
                let difference = match rule.sign {
                    PairSign::Opposite => a + b,
                    PairSign::Same => a - b,
                };
                let in_transit = transit && difference.abs() == step.amount;
                let reason = (!difference.is_zero()).then(|| {
                    if in_transit {
                        format!("in transit {} → {}", md(&step.start_date), md(&step.date))
                    } else {
                        pair_reason(&d.running, rule, difference, &date)
                    }
                });
                rows.push(PairRow {
                    label,
                    date,
                    a,
                    b,
                    difference,
                    // an expected in-transit gap is not a mismatch
                    ok: difference.is_zero() || in_transit,
                    reason,
                });
            }
        }
        Ok(Some(PairCheck {
            rule: rule.id.clone(),
            title: rule.title.clone(),
            rows,
        }))
    }

    /// Lines that could join a group with this line: the same amount (by size) within ±7 days in another
    /// org, nearest first. Lines already in a group with it are left out.
    pub fn suggest_members(&self, anchor: &Anchor) -> Vec<MemberSuggestion> {
        let Some(line) = self.line(anchor) else {
            return vec![];
        };
        let Some(d0) = days(&line.date) else {
            return vec![];
        };
        let together: Vec<&Anchor> = self
            .store()
            .state()
            .groups
            .values()
            .filter(|g| g.members.iter().any(|m| &m.anchor == anchor))
            .flat_map(|g| g.members.iter().map(|m| &m.anchor))
            .collect();
        let mut out: Vec<MemberSuggestion> = self
            .lines()
            .iter()
            .filter(|l| l.org != line.org && l.amount.abs() == line.amount.abs())
            .filter(|l| !together.contains(&&l.anchor))
            .filter_map(|l| {
                let apart = (days(&l.date)? - d0).abs();
                (apart <= 7).then(|| MemberSuggestion {
                    anchor: l.anchor.clone(),
                    org: l.org.clone(),
                    date: l.date.clone(),
                    amount: l.amount,
                    payee: l.payee.clone(),
                    description: l.description.clone(),
                    days_apart: apart,
                })
            })
            .collect();
        out.sort_by(|a, b| {
            a.days_apart
                .cmp(&b.days_apart)
                .then_with(|| a.anchor.cmp(&b.anchor))
        });
        out
    }

    /// The pair rules from config (Catalog/Groups display).
    pub fn pair_rules(&self) -> &[PairRule] {
        &self.config().groups.pair_rules
    }
}

fn chain_check(origins: &[&MemberView], mirrors: &[&MemberView]) -> ChainCheck {
    match (origins.len(), mirrors.len()) {
        (0, 0) => ChainCheck::NotApplicable,
        (_, 0) => ChainCheck::Gap {
            reason: "mirror missing".into(),
        },
        (0, _) => ChainCheck::Gap {
            reason: "origin missing".into(),
        },
        (1, 1) => {
            let (o, m) = (origins[0].amount, mirrors[0].amount);
            let net = o + m;
            if net.is_zero() {
                ChainCheck::OneToOne {
                    origin: o,
                    mirror: m,
                    same_sign: false,
                }
            } else if o == m {
                ChainCheck::Gap {
                    reason: format!(
                        "origin {} and mirror {} move the same way: a mirror books the other side",
                        fmt(o),
                        fmt(m)
                    ),
                }
            } else {
                ChainCheck::Gap {
                    reason: format!(
                        "origin {} ≠ mirror {}: amount differs by {}",
                        fmt(o),
                        fmt(m),
                        fmt(net.abs())
                    ),
                }
            }
        }
        (n, 1) => {
            let net: Decimal = origins.iter().map(|m| m.amount).sum();
            let mirror = mirrors[0].amount;
            let share_pct = if mirror.is_zero() {
                Decimal::new(0, 1)
            } else {
                let mut p = (net.abs() * Decimal::from(100) / mirror.abs()).round_dp(1);
                p.rescale(1);
                p
            };
            ChainCheck::ManyToOne {
                origins: n,
                origin_sum: net,
                mirror,
                share_pct,
            }
        }
        (_, m) => ChainCheck::Gap {
            reason: format!("{m} mirrors: a chain has one mirror; cannot check"),
        },
    }
}

/// Pair the settlement legs (same size, opposite signs, different orgs, within `window` days) and run the open
/// amount down once per settlement. The open amount starts at the size of the origins' signed net (else the
/// mirror's).
fn md(d: &str) -> &str {
    d.get(5..).unwrap_or(d)
}

/// Pair the settlement legs and run the open amount down ONCE per settlement, on the later leg's date. Two legs
/// pair when they are the same size with opposite signs, in different orgs, at most `window` days apart; pairs
/// are chosen one-to-one, greedily by the smallest date gap (ties by anchor order). A leg without a partner is a
/// step of its own. The open amount starts at the size of the origins' signed net (else the mirror's).
fn running_balance(
    origins: &[&MemberView],
    mirrors: &[&MemberView],
    settlements: &[&MemberView],
    window: i64,
) -> Vec<BalanceStep> {
    if settlements.is_empty() {
        return vec![];
    }
    let mut st: Vec<&MemberView> = settlements.to_vec();
    st.sort_by(|a, b| a.date.cmp(&b.date).then_with(|| a.anchor.cmp(&b.anchor)));
    let mut cands: Vec<(i64, usize, usize)> = vec![];
    for i in 0..st.len() {
        for j in i + 1..st.len() {
            let (a, b) = (st[i], st[j]);
            if a.org == b.org || a.amount.is_zero() || b.amount != -a.amount {
                continue;
            }
            if let (Some(x), Some(y)) = (days(&a.date), days(&b.date))
                && (y - x).abs() <= window
            {
                cands.push(((y - x).abs(), i, j));
            }
        }
    }
    cands.sort_by(|p, q| {
        p.0.cmp(&q.0)
            .then_with(|| st[p.1].anchor.cmp(&st[q.1].anchor))
            .then_with(|| st[p.2].anchor.cmp(&st[q.2].anchor))
    });
    let mut partner: Vec<Option<usize>> = vec![None; st.len()];
    for (_, i, j) in cands {
        if partner[i].is_none() && partner[j].is_none() {
            partner[i] = Some(j);
            partner[j] = Some(i);
        }
    }
    // one step per settlement: (applied date, first date, first leg index, legs)
    let mut steps: Vec<(String, String, usize, Vec<usize>)> = vec![];
    for i in 0..st.len() {
        match partner[i] {
            Some(j) if j < i => continue,
            Some(j) => steps.push((st[j].date.clone(), st[i].date.clone(), i, vec![i, j])),
            None => steps.push((st[i].date.clone(), st[i].date.clone(), i, vec![i])),
        }
    }
    steps.sort_by(|a, b| {
        a.0.cmp(&b.0)
            .then_with(|| st[a.2].anchor.cmp(&st[b.2].anchor))
    });
    let mut open: Decimal = if origins.is_empty() {
        mirrors.iter().map(|m| m.amount).sum::<Decimal>().abs()
    } else {
        origins.iter().map(|m| m.amount).sum::<Decimal>().abs()
    };
    let mut out = vec![];
    for (date, start, first, legs) in steps {
        let leg = st[first];
        let before = open;
        open -= leg.amount.abs();
        out.push(BalanceStep {
            anchor: leg.anchor.clone(),
            date,
            amount: leg.amount.abs(),
            before,
            after: open,
            lone_org: (legs.len() == 1).then(|| leg.org.clone()),
            legs: legs.iter().map(|&k| st[k].anchor.clone()).collect(),
            start_date: start,
        });
    }
    out
}

/// Why a pair-rule row differs: a lone settlement leg of the same size already posted by `date` means the other
/// side's leg is missing (or posts later); anything else is unexplained.
fn pair_reason(
    running: &[BalanceStep],
    rule: &PairRule,
    difference: Decimal,
    date: &str,
) -> String {
    let posted = running
        .iter()
        .find(|s| s.lone_org.is_some() && s.amount == difference.abs() && s.date.as_str() <= date);
    let Some(s) = posted else {
        return "difference unexplained".into();
    };
    let mine = s.lone_org.as_deref().unwrap_or("");
    let other = if mine == rule.a.org {
        rule.b.org.as_str()
    } else {
        rule.a.org.as_str()
    };
    let later = running.iter().find(|x| {
        x.lone_org.as_deref() == Some(other) && x.amount == s.amount && x.date.as_str() > date
    });
    match later {
        Some(x) => format!(
            "{other} leg of this settlement is missing (it posts on {})",
            x.date
        ),
        None => format!("{other} leg of this settlement is missing"),
    }
}

fn side_balance(
    record: &crate::view::View<'_>,
    orgs: &[(String, String)],
    side: &PairSide,
    date: &str,
    cache: &mut HashMap<String, crate::Ledger>,
) -> Result<Decimal> {
    let Some((_, tid)) = orgs.iter().find(|(a, _)| *a == side.org) else {
        return Err(super::ClassifyError::new(
            "config",
            format!("pair rule org \"{}\" is not a configured org", side.org),
        ));
    };
    if !cache.contains_key(tid) {
        let l = crate::Ledger::build(record, tid)
            .map_err(|e| super::ClassifyError::new("record", e.to_string()))?;
        cache.insert(tid.clone(), l);
    }
    let ledger = &cache[tid];
    let fy = record.fy_end(tid)?;
    let basis =
        crate::balances::resolve(record, ledger, tid, date, fy, crate::balances::Mode::Auto)
            .map_err(|e| super::ClassifyError::new("record", e.to_string()))?;
    let mut total = Decimal::ZERO;
    for (acct, bal) in &basis.balances {
        let code = ledger.chart.get(acct).code;
        match side.weight(&code) {
            1 => total += *bal,
            -1 => total -= *bal,
            _ => {}
        }
    }
    Ok(total)
}
