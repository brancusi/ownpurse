//! The Ledger tab's models (spec §9): L1 trial balance on a basis, L2 postings of one account with a running
//! balance, L3 one document. Built with core from the record, cached in `Books` (one record version), and
//! prepared in `update`; draw only lays them out. Same numbers as the CLI's `tb` / `gl` / `show`.

use crate::books::{Books, Shared};
use crate::model::Model;
use ownpurse_core::balances::{Basis, BasisKind};
use ownpurse_core::money::{self, money};
use ownpurse_core::{Ledger, fmt};
use rust_decimal::Decimal;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TbRow {
    pub account_id: String,
    pub differs: bool,
    pub code: String,
    pub name: String,
    pub class: String,
    pub balance: Decimal,
    /// For an account that differs from Xero's TB at as-of: Xero's balance and local − Xero (C11).
    pub xero: Option<Decimal>,
    pub difference: Option<Decimal>,
    /// On a manifest, when this account's balance differs from main's: main's balance (the `Δ` mark).
    pub main: Option<Decimal>,
}

/// The basis strip's facts (V6): recomputed into text every draw.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BasisFacts {
    pub kind: BasisKind,
    pub as_of: String,
    pub fy_start: String,
    pub opening_date: Option<String>,
    /// Some(true) matches, Some(false) differs, None not compared.
    pub matches: Option<bool>,
    pub differs: usize,
}

impl BasisFacts {
    pub fn of(b: &Basis) -> BasisFacts {
        BasisFacts {
            kind: b.kind,
            as_of: b.as_of.clone(),
            fy_start: b.fy_start.clone(),
            opening_date: b.opening_date.clone(),
            matches: b.matches_xero(),
            differs: b.diffs.len(),
        }
    }

    /// The strip line.
    pub fn line(&self) -> String {
        let n = crate::clock::plural(self.differs as i64, "account", "accounts");
        if self.kind == BasisKind::XeroOpening {
            let base = format!(
                "Opening from Xero TB {} + {} activity",
                self.opening_date.as_deref().unwrap_or(""),
                self.as_of.get(..4).unwrap_or("")
            );
            return match self.matches {
                Some(true) => format!("{base} · matches Xero ✓"),
                Some(false) => format!("{base} · differs from Xero at {}: {n}", self.as_of),
                None => format!(
                    "{base} · not compared: no cached Xero TB at {} (⌃P → Refetch)",
                    self.as_of
                ),
            };
        }
        let verb = if self.differs == 1 {
            "differs"
        } else {
            "differ"
        };
        match self.matches {
            Some(false) => format!(
                "Local full-history balances: {n} {verb} from Xero (marked !; usually pre-{} setup balances). \
                 ⌃P → Refetch TB to open from Xero.",
                self.fy_start.get(..4).unwrap_or("")
            ),
            Some(true) => "Local full-history balances · matches Xero ✓".into(),
            None => {
                "Local full-history balances — no cached Xero TB to compare; may differ from Xero. \
                     ⌃P → Refetch TB to open from Xero."
                    .into()
            }
        }
    }

    pub fn ok(&self) -> bool {
        self.matches == Some(true)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct L1Model {
    pub org: usize,
    pub as_of: String,
    pub history: bool,
    pub rows: Vec<TbRow>,
    pub debit: Decimal,
    pub credit: Decimal,
    pub basis: BasisFacts,
    /// Shown on a manifest: the basis facts compare MAIN with Xero (parity is always main's).
    pub manifest: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GlRow {
    pub date: String,
    pub source: &'static str,
    pub source_id: String,
    pub line: usize,
    pub kind: String,
    pub reference: String,
    pub contact: String,
    pub description: String,
    pub amount: Decimal,
    pub balance: Decimal,
    /// Proposed by the shown manifest: its glyph (`+`, `~`, `⊘`; `A+` for an agent's entry). A void's
    /// postings stay listed (`ghost`) but never move the balance.
    pub mark: Option<String>,
    pub ghost: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct L2Model {
    pub account_id: String,
    pub account: String,
    pub since: String,
    pub as_of: String,
    pub opening: Decimal,
    /// `from Xero TB <date>` or `from local history`.
    pub opening_source: String,
    pub opening_from_xero: bool,
    pub debits: Decimal,
    pub credits: Decimal,
    pub closing: Decimal,
    pub rows: Vec<GlRow>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DocLine {
    pub description: String,
    pub code: String,
    pub account: String,
    pub amount: String,
    pub tax: String,
    /// Recoded on the shown manifest: the account it had on main (`was 6140 Roaster …`).
    pub was: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DocLink {
    pub url: String,
    pub verified: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DocModel {
    pub source: String,
    pub id: String,
    pub fields: Vec<(String, String)>,
    pub lines: Vec<DocLine>,
    /// None = not fetched yet; Some(names and sizes).
    pub attachments: Option<Vec<(String, i64)>>,
    /// None = not fetched yet; Some((when, change, by)).
    pub history: Option<Vec<(String, String, String)>>,
    /// Latest local version and when it was observed.
    pub local: Option<(i64, String)>,
    pub link: Option<DocLink>,
    /// The shown manifest proposes this document: (kind, change seq, actor). A proposed journal has no Xero
    /// link; a void strikes its amounts through.
    pub proposed: Option<(String, i64, String)>,
    /// The accountant gate for that change: Some(None) = needs a decision, Some(Some(ref)) = annotated.
    pub accountant: Option<Option<String>>,
}

impl DocModel {
    pub fn date(&self) -> &str {
        self.fields
            .iter()
            .find(|(k, _)| k == "Date")
            .map_or("", |(_, v)| v.as_str())
    }
}

fn side_total(rows: &[TbRow]) -> (Decimal, Decimal) {
    let (mut dr, mut cr) = (Decimal::ZERO, Decimal::ZERO);
    for r in rows {
        if r.balance > Decimal::ZERO {
            dr = money::add(dr, r.balance);
        } else if r.balance < Decimal::ZERO {
            cr = money::add(cr, -r.balance);
        }
    }
    (dr, cr)
}

/// L1: the trial balance on the basis: accounts Xero alone holds appear at zero,
/// flagged `!`; rows sorted by (class, code, name).
pub fn l1(
    books: &mut Books,
    model: &Model,
    org: usize,
    as_of: &str,
    history: bool,
) -> Shared<L1Model> {
    let key = (org, as_of.to_string(), history);
    if let Some(m) = books.l1.get(&key) {
        return m.clone();
    }
    let basis = books.basis(model, org, as_of, history);
    // on a manifest, `!` compares MAIN with Xero; the manifest's own balances get `Δ` against main
    let on_manifest = books.overlay().is_some();
    let main_basis = on_manifest.then(|| books.main_basis(model, org, as_of, history));
    let built = match (basis.as_ref(), books.ledger(model, org).as_ref()) {
        (Ok(b), Ok(l)) => {
            let mb: Option<&Basis> = main_basis.as_ref().and_then(|m| m.as_ref().as_ref().ok());
            let parity = mb.unwrap_or(b);
            let differing: std::collections::BTreeSet<&str> =
                parity.diffs.iter().map(|d| d.account_id.as_str()).collect();
            let mut shown: Vec<(String, Decimal)> = differing
                .iter()
                .map(|a| (a.to_string(), Decimal::ZERO))
                .collect();
            if let Some(m) = mb {
                for k in m.balances.keys() {
                    if !shown.iter().any(|(a, _)| a == k) {
                        shown.push((k.clone(), Decimal::ZERO));
                    }
                }
            }
            for (k, v) in &b.balances {
                match shown.iter_mut().find(|(a, _)| a == k) {
                    Some(e) => e.1 = *v,
                    None => shown.push((k.clone(), *v)),
                }
            }
            let mut rows: Vec<TbRow> = shown
                .into_iter()
                .map(|(aid, v)| {
                    let a = l.chart.get(&aid);
                    let d = parity.diffs.iter().find(|d| d.account_id == aid);
                    let main = mb
                        .map(|m| m.balances.get(&aid).copied().unwrap_or_default())
                        .filter(|m| *m != v);
                    TbRow {
                        main,
                        xero: d.map(|d| d.xero),
                        difference: d.map(|d| d.difference),
                        differs: differing.contains(aid.as_str()),
                        account_id: aid,
                        code: a.code,
                        name: a.name,
                        class: a.class,
                        balance: v,
                    }
                })
                .collect();
            rows.sort_by(|x, y| (&x.class, &x.code, &x.name).cmp(&(&y.class, &y.code, &y.name)));
            let (debit, credit) = side_total(&rows);
            Ok(L1Model {
                org,
                as_of: as_of.into(),
                history,
                rows,
                debit,
                credit,
                basis: BasisFacts::of(parity),
                manifest: on_manifest,
            })
        }
        (Err(e), _) | (_, Err(e)) => Err(e.clone()),
    };
    let m = Arc::new(built);
    books.l1.insert(key, m.clone());
    m
}

/// Postings by account id (indexes into `Ledger::postings`).
pub type AccountIndex = Arc<HashMap<String, Vec<usize>>>;

fn index(
    books: &mut Books,
    model: &Model,
    org: usize,
) -> Result<(Arc<Ledger>, AccountIndex), String> {
    let l = books.ledger(model, org)?;
    if let Some(ix) = books.by_account.get(&org) {
        return Ok((l, ix.clone()));
    }
    let mut ix: HashMap<String, Vec<usize>> = HashMap::new();
    for (i, p) in l.postings.iter().enumerate() {
        ix.entry(p.account_id.clone()).or_default().push(i);
    }
    let ix = Arc::new(ix);
    books.by_account.insert(org, ix.clone());
    Ok((l, ix))
}

/// L2: postings of one account in [since, as_of] with a running balance from the basis' opening
/// (`since` defaults to the fiscal-year start).
pub fn l2(
    books: &mut Books,
    model: &Model,
    org: usize,
    account_id: &str,
    as_of: &str,
    since: Option<&str>,
    history: bool,
) -> Shared<L2Model> {
    let basis = books.basis(model, org, as_of, history);
    let since = match (since, basis.as_ref()) {
        (Some(s), _) => s.to_string(),
        (None, Ok(b)) => b.fy_start.clone(),
        (None, Err(_)) => String::new(),
    };
    let key = (
        org,
        account_id.to_string(),
        as_of.to_string(),
        since.clone(),
        history,
    );
    if let Some(m) = books.l2.get(&key) {
        return m.clone();
    }
    let built = match (basis.as_ref(), index(books, model, org).as_ref()) {
        (Ok(b), Ok((l, ix))) => {
            let opening = b.opening_for(l, account_id, &since);
            let from_xero = b.kind == BasisKind::XeroOpening
                && b.opening_date
                    .as_deref()
                    .is_some_and(|od| since.as_str() > od);
            let (mut bal, mut dr, mut cr) = (opening, Decimal::ZERO, Decimal::ZERO);
            let mut rows = Vec::new();
            // the account's postings, and a manifest void's struck-through ones, in date order
            let mut items: Vec<(&ownpurse_core::ledger::Posting, bool)> = ix
                .get(account_id)
                .map(Vec::as_slice)
                .unwrap_or(&[])
                .iter()
                .map(|&i| (&l.postings[i], false))
                .chain(
                    l.ghosts
                        .iter()
                        .filter(|g| g.account_id == account_id)
                        .map(|g| (g, true)),
                )
                .collect();
            items.sort_by(|a, b| a.0.date.cmp(&b.0.date));
            for (p, ghost) in items {
                if p.date.as_str() < since.as_str() || p.date.as_str() > as_of {
                    continue;
                }
                if !ghost {
                    bal = money::add(bal, p.amount);
                    if p.amount > Decimal::ZERO {
                        dr = money::add(dr, p.amount);
                    } else {
                        cr = money::add(cr, -p.amount);
                    }
                }
                let mark = p
                    .proposed
                    .as_ref()
                    .map(|m| format!("{}{}", if m.agent { "A" } else { "" }, m.glyph()))
                    .or_else(|| ghost.then(|| "⊘".to_string()));
                rows.push(GlRow {
                    mark,
                    ghost,
                    date: p.date.clone(),
                    source: p.source,
                    source_id: p.source_id.clone(),
                    line: p.line,
                    kind: if p.kind.is_empty() {
                        p.source.to_string()
                    } else {
                        p.kind.clone()
                    },
                    reference: p.reference.clone(),
                    contact: p.contact.clone(),
                    description: p.description.chars().take(60).collect(),
                    amount: p.amount,
                    balance: bal,
                });
            }
            Ok(L2Model {
                account_id: account_id.into(),
                account: l.chart.get(account_id).label(),
                since: since.clone(),
                as_of: as_of.into(),
                opening,
                opening_source: if from_xero {
                    format!("from Xero TB {}", b.opening_date.as_deref().unwrap_or(""))
                } else {
                    "from local history".into()
                },
                opening_from_xero: from_xero,
                debits: dr,
                credits: cr,
                closing: bal,
                rows,
            })
        }
        (Err(e), _) | (_, Err(e)) => Err(e.clone()),
    };
    let m = Arc::new(built);
    books.l2.insert(key, m.clone());
    m
}

fn type_label(t: &str) -> String {
    match t {
        "SPEND" => "Spend money",
        "RECEIVE" => "Receive money",
        "SPEND-TRANSFER" => "Transfer out",
        "RECEIVE-TRANSFER" => "Transfer in",
        "ACCREC" => "Sales invoice",
        "ACCPAY" => "Bill",
        "ACCRECCREDIT" => "Sales credit note",
        "ACCPAYCREDIT" => "Bill credit note",
        other => other,
    }
    .to_string()
}

fn title_case(s: &str) -> String {
    let lower = s.to_lowercase();
    let mut out = String::new();
    let mut up = true;
    for ch in lower.chars() {
        if up && ch.is_alphabetic() {
            out.extend(ch.to_uppercase());
            up = false;
        } else {
            out.push(ch);
        }
        if !ch.is_alphanumeric() {
            up = true;
        }
    }
    out
}

fn text(v: &Value) -> Option<String> {
    match v {
        Value::Null => None,
        Value::String(s) if s.is_empty() => None,
        Value::String(s) => Some(s.clone()),
        other => Some(other.to_string()),
    }
}

/// The first `per: document` link for this source, filled with the document id.
fn link_for(links: &Value, source: &str, id: &str) -> Option<DocLink> {
    links.as_array()?.iter().find_map(|l| {
        (l.get("per").and_then(Value::as_str) == Some("document")
            && l.get("source").and_then(Value::as_str) == Some(source))
        .then(|| DocLink {
            url: l
                .get("url")
                .and_then(Value::as_str)
                .unwrap_or("")
                .replace("{document_id}", id),
            verified: l.get("verified").and_then(Value::as_bool).unwrap_or(true),
        })
    })
}

/// "Open in Xero" through the org switch (core `deeplink`), when the org's ShortCode is known: unverified
/// until the paths are checked live.
fn org_link(short_code: &str, source: &str, kind: Option<&str>, id: &str) -> Option<DocLink> {
    ownpurse_core::deeplink::url_for(short_code, source, kind.unwrap_or(""), id).map(|url| {
        DocLink {
            url,
            verified: false,
        }
    })
}

/// L3: one document as the record holds it: fields, lines, attachments and
/// history when fetched, the local version, the Xero link template.
pub fn doc(
    books: &mut Books,
    model: &Model,
    links: &Value,
    org: usize,
    source: &str,
    id: &str,
) -> Shared<DocModel> {
    let key = (org, source.to_string(), id.to_string());
    if let Some(m) = books.docs.get(&key) {
        return m.clone();
    }
    let tid = model.orgs[org].tenant.clone();
    let ledger = books.ledger(model, org);
    let conn = books.record().conn();
    let built = (|| -> Result<DocModel, String> {
        let mut st = conn
            .prepare("SELECT version, observed_at, payload FROM versions WHERE tenant_id=?1 AND entity=?2 AND xero_id=?3 ORDER BY version")
            .map_err(|e| e.to_string())?;
        let versions: Vec<(i64, String, String)> = st
            .query_map((&tid, source, id), |r| {
                Ok((r.get(0)?, r.get(1)?, r.get(2)?))
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<_, _>>()
            .map_err(|e| e.to_string())?;
        let p: Value = versions
            .last()
            .map(|(_, _, t)| serde_json::from_str(t).unwrap_or(Value::Null))
            .unwrap_or(Value::Null);
        // the shown manifest's version of it (a proposed journal, a recode, a void), else main's
        let view = books.view();
        let (replaced, added) = view.patched(&tid, source);
        let p = replaced
            .into_iter()
            .find(|(k, _)| k == id)
            .map(|(_, v)| v)
            .or_else(|| {
                added
                    .into_iter()
                    .find(|d| ownpurse_core::view::doc_id(source, d) == id)
            })
            .unwrap_or(p);
        let mark = |v: &Value| -> Option<(String, i64, String)> {
            let m = v.get("_xa_manifest")?;
            Some((
                m["kind"].as_str().unwrap_or("").to_string(),
                m["seq"].as_i64().unwrap_or(0),
                m["actor"].as_str().unwrap_or("").to_string(),
            ))
        };
        let line_mark = p
            .get("LineItems")
            .or_else(|| p.get("JournalLines"))
            .and_then(Value::as_array)
            .and_then(|a| a.iter().find_map(mark));
        let proposed = mark(&p).or(line_mark);
        let accountant = proposed.as_ref().and_then(|(_, seq, _)| {
            let o = view.overlay()?;
            let a = o.applied(*seq)?;
            a.gated.then(|| a.accountant.clone())
        });
        let typ = text(&p["Type"]).unwrap_or_else(|| match source {
            "ManualJournals" => "Manual journal".into(),
            "BankTransfers" => "Bank transfer".into(),
            "Payments" => text(&p["PaymentType"]).unwrap_or_else(|| "Payment".into()),
            other => other.into(),
        });
        let bank = text(&p["BankAccount"]["Name"]).or_else(|| text(&p["FromBankAccount"]["Name"]));
        let total = if p.get("Total").is_some() {
            p.get("Total")
        } else {
            p.get("Amount")
        };
        let fields: Vec<(&str, Option<String>)> = vec![
            (
                "Date",
                Some(ownpurse_core::dates::doc_date(&p)).filter(|s| !s.is_empty()),
            ),
            ("Contact", text(&p["Contact"]["Name"])),
            ("Type", Some(type_label(&typ))),
            ("Status", text(&p["Status"]).map(|s| title_case(&s))),
            ("Bank account", bank),
            (
                "Reference",
                text(&p["Reference"]).or_else(|| text(&p["InvoiceNumber"])),
            ),
            ("Narration", text(&p["Narration"])),
            // a journal this tool wrote (Phase 8): its package, by the narration's XA: marker
            (
                "Write package",
                text(&p["Narration"])
                    .and_then(|n| ownpurse_core::write::view::package_of_marker(books.record(), &n))
                    .map(|(title, _pid, hash, op)| {
                        // the narration already carries the marker: name the package's hash and op, and
                        // its title when the narration doesn't show it
                        let short = &hash[..hash.len().min(12)];
                        if p["Narration"].as_str().is_some_and(|n| n.contains(&title)) {
                            format!("written by package {short} ({op})")
                        } else {
                            format!("written by package {title} · {short} ({op})")
                        }
                    }),
            ),
            (
                "Amounts",
                text(&p["LineAmountTypes"]).map(|t| match t.as_str() {
                    "Inclusive" => "tax inclusive".into(),
                    "Exclusive" => "tax exclusive".into(),
                    "NoTax" => "no tax".into(),
                    _ => t,
                }),
            ),
            (
                "Total",
                total
                    .filter(|v| !v.is_null())
                    .map(|v| fmt(money(Some(v)).unwrap_or_default())),
            ),
            (
                "Reconciled",
                p.get("IsReconciled")
                    .and_then(Value::as_bool)
                    .map(|b| if b { "yes" } else { "no" }.to_string()),
            ),
        ];
        let fields = fields
            .into_iter()
            .filter_map(|(k, v)| v.map(|v| (k.to_string(), v)))
            .collect();
        let items = p
            .get("LineItems")
            .and_then(Value::as_array)
            .or_else(|| p.get("JournalLines").and_then(Value::as_array))
            .cloned()
            .unwrap_or_default();
        let lines = items
            .iter()
            .map(|li| {
                let code = text(&li["AccountCode"]).unwrap_or_default();
                let name = ledger
                    .as_ref()
                    .ok()
                    .and_then(|l| l.chart.by_code(&code).map(|a| a.name.clone()));
                DocLine {
                    description: text(&li["Description"])
                        .map(|d| d.chars().take(50).collect())
                        .unwrap_or_else(|| "—".into()),
                    account: match &name {
                        Some(n) => format!("{code} · {n}"),
                        None if code.is_empty() => "—".into(),
                        None => code.clone(),
                    },
                    code,
                    amount: fmt(money(li.get("LineAmount")).unwrap_or_default()),
                    tax: fmt(money(li.get("TaxAmount")).unwrap_or_default()),
                    was: li.get("_xa_manifest").and_then(|m| {
                        let c = m["was_account"].as_str()?.to_string();
                        let n = ledger
                            .as_ref()
                            .ok()
                            .and_then(|l| l.chart.by_code(&c).map(|a| a.name.clone()));
                        Some(match n {
                            Some(n) => format!("{c} {n}"),
                            None => c,
                        })
                    }),
                }
            })
            .collect();
        let mut st = conn
            .prepare("SELECT payload FROM current WHERE tenant_id=?1 AND entity='Attachments' AND xero_id LIKE ?2")
            .map_err(|e| e.to_string())?;
        let att: Vec<(String, i64)> = st
            .query_map((&tid, format!("{source}/{id}/%")), |r| {
                r.get::<_, String>(0)
            })
            .map_err(|e| e.to_string())?
            .filter_map(Result::ok)
            .filter_map(|t| serde_json::from_str::<Value>(&t).ok())
            .map(|a| {
                (
                    text(&a["FileName"]).unwrap_or_default(),
                    a["ContentLength"].as_i64().unwrap_or(0),
                )
            })
            .collect();
        let hist: Option<String> = conn
            .query_row(
                "SELECT payload FROM current WHERE tenant_id=?1 AND entity='History' AND xero_id=?2",
                (&tid, format!("{source}/{id}")),
                |r| r.get(0),
            )
            .ok();
        let history = hist
            .and_then(|t| serde_json::from_str::<Value>(&t).ok())
            .map(|h| {
                h.as_array()
                    .cloned()
                    .unwrap_or_default()
                    .iter()
                    .map(|e| {
                        (
                            text(&e["DateUTCString"])
                                .map(|s| s.chars().take(16).collect())
                                .unwrap_or_default(),
                            ownpurse_core::text::html_unescape(
                                &text(&e["Changes"]).unwrap_or_default(),
                            ),
                            ownpurse_core::text::html_unescape(
                                &text(&e["User"]).unwrap_or_default(),
                            ),
                        )
                    })
                    .collect()
            });
        Ok(DocModel {
            source: source.into(),
            id: id.into(),
            fields,
            lines,
            attachments: (!att.is_empty()).then_some(att),
            history,
            local: versions.last().map(|(v, at, _)| (*v, at.clone())),
            // the org switch when the org's ShortCode is known (core deeplink; opens in the right org),
            // else the configured `per: document` template
            link: org_link(&model.orgs[org].short_code, source, p["Type"].as_str(), id)
                .or_else(|| link_for(links, source, id)),
            proposed,
            accountant,
        })
    })();
    let m = Arc::new(built);
    books.docs.insert(key, m.clone());
    m
}
