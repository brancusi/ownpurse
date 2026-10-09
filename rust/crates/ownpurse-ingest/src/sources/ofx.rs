//! OFX / QFX bank statements: OFX 1.x SGML (leaf elements without closing tags) and OFX 2.x XML.
//!
//! Each `STMTTRN` is one observation whose identity is its FITID within the account, so an overlapping
//! statement names the same lines. `DTPOSTED` keeps its local date (the part before the zone bracket); the
//! raw text is kept beside it. `LEDGERBAL` and `AVAILBAL` become `obs.kind/balance` observations of the
//! statement, and the ledger balance is checked against the running balances of any CSV download of the
//! same account already in the log.

use super::bank_csv::{
    AccountSpec, Line, account, date_of, dec_of, earlier, k, kwv, line, money, status,
};
use crate::facts;
use crate::{IngestError, Input, Pressure, Projection, Projector};
use ownpurse_facts::value::parse_decimal;
use ownpurse_facts::{Date, Db, TxData, ValIn, Value};
use rust_decimal::Decimal;
use serde_json::{Value as Json, json};
use std::collections::{BTreeMap, BTreeSet};

pub struct Ofx;

impl Projector for Ofx {
    fn id(&self) -> &'static str {
        "bank.ofx@1"
    }

    fn reads(&self) -> &'static str {
        "OFX / QFX bank statements"
    }

    fn project(&self, db: &Db, input: &Input<'_>) -> Result<Projection, IngestError> {
        let name = input.file.name;
        let mut pressure = Vec::new();
        let text = match std::str::from_utf8(input.file.bytes) {
            Ok(t) => t.to_string(),
            Err(_) => {
                pressure.push(Pressure::new(
                    "encoding",
                    "file",
                    "not UTF-8; read as Windows-1252 single bytes",
                ));
                input.file.bytes.iter().map(|&b| char::from(b)).collect()
            }
        };
        let root = parse(&text).map_err(|e| IngestError(format!("{name}: {e}")))?;
        let mut st = Statements {
            db,
            input,
            tx: TxData::new(),
            pressure,
            zones: BTreeSet::new(),
            times: BTreeSet::new(),
        };
        let mut found = 0;
        for (path, stmt) in root.find_all("", &["STMTRS", "CCSTMTRS"]) {
            st.statement(&path, stmt)?;
            found += 1;
        }
        if found == 0 {
            return Err(IngestError(format!(
                "{name}: no STMTRS or CCSTMTRS (is this a bank or card statement?)"
            )));
        }
        if !st.zones.is_empty() {
            let zones: Vec<String> = st.zones.iter().cloned().collect();
            st.press(
                if zones.len() > 1 {
                    "zone-mixed"
                } else {
                    "local-date"
                },
                "DTPOSTED",
                format!(
                    "dates carry zone brackets {}; each line keeps its local date and the raw text",
                    zones.join(", ")
                ),
            );
        }
        if st.times.len() == 1 && st.times.contains("120000") {
            st.press(
                "placeholder-time",
                "DTPOSTED",
                "every line is posted at 12:00:00, a placeholder; no instant is recorded",
            );
        }
        Ok(Projection {
            install: install(),
            tx: st.tx,
            pressure: st.pressure,
        })
    }
}

fn install() -> Vec<Json> {
    vec![
        json!({"ident": "bank.ofx/fitid", "type": "string",
               "doc": "The bank's id for the line, unique within the account (not across accounts)"}),
        json!({"ident": "bank.ofx/trntype", "type": "string", "doc": "CREDIT, DEBIT, CHECK, POS, XFER, …"}),
        json!({"ident": "bank.ofx/memo", "type": "string"}),
        json!({"ident": "bank.ofx/dtposted", "type": "string",
               "doc": "DTPOSTED as written, with its zone bracket: 20260302120000[-5:EST]"}),
        json!({"ident": "bank.ofx/balance-kind", "type": "keyword",
               "doc": "bank.ofx.balance/ledger or bank.ofx.balance/available"}),
    ]
}

// ---------------------------------------------------------------------------------------------------------
// the element tree

#[derive(Debug, Default)]
struct Node {
    name: String,
    text: Option<String>,
    children: Vec<Node>,
}

impl Node {
    fn child(&self, name: &str) -> Option<&Node> {
        self.children.iter().find(|c| c.name == name)
    }

    fn leaf(&self, name: &str) -> Option<&str> {
        self.child(name)
            .and_then(|c| c.text.as_deref())
            .filter(|t| !t.is_empty())
    }

    /// Every descendant named one of `names`, with its element path: `/OFX/…/STMTTRN[3]`.
    fn find_all<'n>(&'n self, base: &str, names: &[&str]) -> Vec<(String, &'n Node)> {
        let mut out = Vec::new();
        let mut seen: BTreeMap<&str, usize> = BTreeMap::new();
        for c in &self.children {
            let n = seen.entry(c.name.as_str()).or_insert(0);
            *n += 1;
            let many = self.children.iter().filter(|x| x.name == c.name).count() > 1;
            let path = if many {
                format!("{base}/{}[{n}]", c.name)
            } else {
                format!("{base}/{}", c.name)
            };
            if names.contains(&c.name.as_str()) {
                out.push((path.clone(), c));
            }
            out.extend(c.find_all(&path, names));
        }
        out
    }
}

fn unescape(t: &str) -> String {
    t.replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&nbsp;", " ")
        .replace("&amp;", "&")
}

/// OFX 1.x SGML or 2.x XML into a tree. The header block (`KEY:VALUE` lines, or `<?xml …?>` and
/// `<?OFX …?>`) is skipped; a leaf is an element followed by text, closed or not; an aggregate closes with
/// `</NAME>`, and a close for an element not open is ignored.
fn parse(text: &str) -> Result<Node, String> {
    let start = text.find('<').ok_or("no elements (is this an OFX file?)")?;
    let header = &text[..start];
    if !header.trim().is_empty() && !header.contains("OFXHEADER") {
        return Err("no OFXHEADER block before the first element".into());
    }
    let mut stack: Vec<Node> = vec![Node::default()];
    let mut rest = &text[start..];
    while let Some(lt) = rest.find('<') {
        let after = &rest[lt + 1..];
        let gt = after.find('>').ok_or("an element is not closed with '>'")?;
        let tag = after[..gt].trim();
        let tail = &after[gt + 1..];
        let next = tail.find('<').unwrap_or(tail.len());
        let body = tail[..next].trim();
        rest = &tail[next..];
        if tag.starts_with('?') || tag.starts_with('!') {
            continue;
        }
        if let Some(name) = tag.strip_prefix('/') {
            // a closing tag: XML-style close of a leaf just read, or the end of an aggregate
            if let Some(pos) = stack.iter().rposition(|n| n.name == name)
                && pos > 0
            {
                while stack.len() > pos {
                    let done = stack.pop().unwrap_or_default();
                    if let Some(parent) = stack.last_mut() {
                        parent.children.push(done);
                    }
                }
            }
            continue;
        }
        let name = tag.split_whitespace().next().unwrap_or("").to_string();
        if body.is_empty() {
            // an aggregate, unless an XML leaf is closed straight away (`<MEMO></MEMO>`)
            if rest.starts_with(&format!("</{name}>")) {
                rest = &rest[name.len() + 3..];
                if let Some(p) = stack.last_mut() {
                    p.children.push(Node {
                        name,
                        text: Some(String::new()),
                        children: Vec::new(),
                    });
                }
                continue;
            }
            stack.push(Node {
                name,
                text: None,
                children: Vec::new(),
            });
        } else {
            if let Some(p) = stack.last_mut() {
                p.children.push(Node {
                    name: name.clone(),
                    text: Some(unescape(body)),
                    children: Vec::new(),
                });
            }
            if rest.starts_with(&format!("</{name}>")) {
                rest = &rest[name.len() + 3..];
            }
        }
    }
    while stack.len() > 1 {
        let done = stack.pop().unwrap_or_default();
        if let Some(parent) = stack.last_mut() {
            parent.children.push(done);
        }
    }
    stack.pop().ok_or_else(|| "empty".to_string())
}

/// `20260302120000.000[-5:EST]` → (local date, time digits, zone bracket).
fn ofx_date(t: &str) -> Option<(Date, String, Option<String>)> {
    let (body, zone) = match t.find('[') {
        Some(i) => (&t[..i], Some(t[i..].to_string())),
        None => (t, None),
    };
    let d = body.get(..8)?;
    let date = Date::parse(&format!("{}-{}-{}", &d[..4], &d[4..6], &d[6..8])).ok()?;
    let time: String = body
        .get(8..)
        .unwrap_or("")
        .chars()
        .take_while(char::is_ascii_digit)
        .collect();
    Some((date, time, zone))
}

fn amount(t: &str) -> Result<Decimal, String> {
    let t = t.trim();
    let t = t.strip_prefix('+').unwrap_or(t);
    // some banks write a decimal comma
    let t = if t.contains(',') && !t.contains('.') {
        t.replace(',', ".")
    } else {
        t.to_string()
    };
    parse_decimal(&t).map_err(|e| e.0)
}

// ---------------------------------------------------------------------------------------------------------
// statements

struct Statements<'a, 'i> {
    db: &'a Db,
    input: &'a Input<'i>,
    tx: TxData,
    pressure: Vec<Pressure>,
    zones: BTreeSet<String>,
    times: BTreeSet<String>,
}

impl Statements<'_, '_> {
    fn press(&mut self, kind: &str, at: impl Into<String>, detail: impl Into<String>) {
        self.pressure.push(Pressure::new(kind, at, detail));
    }

    fn src(&self) -> &str {
        &self.input.source.id
    }

    /// `bank:ridgeway-4821` and CHECKING …4821 → `ridgeway:checking:4821`. OFX names an account only by its
    /// number, so the institution part comes from the source id.
    fn account_spec(&mut self, path: &str, stmt: &Node) -> Result<AccountSpec, IngestError> {
        let from = stmt
            .child("BANKACCTFROM")
            .or_else(|| stmt.child("CCACCTFROM"))
            .ok_or_else(|| {
                IngestError(format!(
                    "{}: {path} names no account (BANKACCTFROM)",
                    self.input.file.name
                ))
            })?;
        let acctid = from.leaf("ACCTID").unwrap_or("");
        let digits: String = acctid.chars().filter(char::is_ascii_digit).collect();
        if digits.len() < 4 {
            return Err(IngestError(format!(
                "{}: ACCTID {acctid:?} has fewer than four digits",
                self.input.file.name
            )));
        }
        let mask = digits[digits.len() - 4..].to_string();
        let kind = match from.leaf("ACCTTYPE") {
            Some(t) => t.to_ascii_lowercase(),
            None if from.name == "CCACCTFROM" => "card".into(),
            None => "account".into(),
        };
        let inst = self
            .src()
            .split_once(':')
            .map_or(self.src(), |(_, r)| r)
            .split('-')
            .next()
            .unwrap_or("")
            .to_string();
        let id = format!("{inst}:{kind}:{mask}");
        if self.db.lookup(&k("account/id"), &Value::str(&id)).is_none() {
            self.press(
                "account-identity",
                format!("{path}/{}/ACCTID", from.name),
                format!(
                    "account {id} is new to the log; its id was built from the source id and ACCTTYPE, since OFX gives only the number"
                ),
            );
        }
        Ok(AccountSpec {
            id,
            mask,
            name: None,
            institution: None,
            commodity: stmt.leaf("CURDEF").unwrap_or("USD").to_string(),
            scale: Some(2),
        })
    }

    fn statement(&mut self, path: &str, stmt: &Node) -> Result<(), IngestError> {
        let spec = self.account_spec(path, stmt)?;
        let input = self.input;
        let (acct, cur) = account(&mut self.tx, input, &spec);
        let list = stmt.child("BANKTRANLIST");
        let span = |tag: &str| list.and_then(|l| l.leaf(tag)).and_then(ofx_date);
        let (start, end) = (span("DTSTART"), span("DTEND"));
        // the statement (balances belong to it) and the feed its lines belong to
        let period = match (&start, &end) {
            (Some(a), Some(b)) => format!("{}..{}", a.0.as_str(), b.0.as_str()),
            _ => input.file.hash.hex(),
        };
        let stmt_id = format!("{}|ofx-statement|{}|{period}", self.src(), spec.id);
        let sdoc = facts::document(&mut self.tx, input, &stmt_id, "bank-ofx-statement");
        if let (Some(a), Some(b)) = (&start, &end) {
            self.tx
                .add(&sdoc, &k("doc/period-start"), Value::Date(a.0.clone()));
            self.tx
                .add(&sdoc, &k("doc/period-end"), Value::Date(b.0.clone()));
        }
        let feed_id = format!("{}|ofx|{}", self.src(), spec.id);
        let feed = facts::document(&mut self.tx, input, &feed_id, "bank-activity");
        let mut lines: Vec<(Date, Decimal)> = Vec::new();
        let trns = list
            .map(|l| l.find_all(&format!("{path}/BANKTRANLIST"), &["STMTTRN"]))
            .unwrap_or_default();
        for (at, t) in trns {
            let Some(fitid) = t.leaf("FITID") else {
                self.press(
                    "no-identity",
                    &at,
                    "a line without a FITID is skipped; OFX requires one",
                );
                continue;
            };
            let Some(raw_date) = t.leaf("DTPOSTED") else {
                self.press("unreadable", &at, "no DTPOSTED");
                continue;
            };
            let Some((date, time, zone)) = ofx_date(raw_date) else {
                self.press("unreadable", &at, format!("DTPOSTED {raw_date:?}"));
                continue;
            };
            self.zones.extend(zone);
            self.times.insert(time);
            let amt = match t.leaf("TRNAMT").map(amount) {
                Some(Ok(a)) => a,
                Some(Err(e)) => {
                    self.press("unreadable", &at, format!("TRNAMT: {e}"));
                    continue;
                }
                None => {
                    self.press("unreadable", &at, "no TRNAMT");
                    continue;
                }
            };
            let trntype = t.leaf("TRNTYPE").unwrap_or("OTHER");
            let out = matches!(
                trntype,
                "DEBIT" | "CHECK" | "FEE" | "SRVCHG" | "ATM" | "PAYMENT" | "DIRECTDEBIT" | "CASH"
            );
            let inn = matches!(trntype, "CREDIT" | "DEP" | "DIRECTDEP" | "INT" | "DIV");
            if (out && amt.is_sign_positive() && !amt.is_zero()) || (inn && amt.is_sign_negative())
            {
                self.press(
                    "sign",
                    &at,
                    format!(
                        "TRNTYPE {trntype} but TRNAMT {amt}; the signed amount is kept as written"
                    ),
                );
            }
            let name = t.leaf("NAME").or_else(|| t.leaf("PAYEE")).unwrap_or("");
            let memo = t.leaf("MEMO");
            if name.chars().count() == 32 {
                let full = memo.filter(|m| m.starts_with(name) && m.len() > name.len());
                self.press(
                    "truncated",
                    &at,
                    match full {
                        Some(m) => format!("NAME {name:?} is cut at 32 characters; MEMO has {m:?}"),
                        None => {
                            format!("NAME {name:?} is 32 characters, OFX's limit; it may be cut")
                        }
                    },
                );
            }
            let id = format!("{}|ofx|{}|{fitid}", self.src(), spec.id);
            if let Some(e) = self.db.lookup(&k("obs/id"), &Value::str(&id)) {
                let was = (date_of(self.db, e), dec_of(self.db, e, "obs/amount"));
                if was != (Some(date.clone()), Some(amt)) {
                    self.press(
                        "fitid-reused",
                        &at,
                        format!(
                            "FITID {fitid} was {} {} in an earlier file and is {} {} here; the newer reading replaces it",
                            was.0.map(|d| d.as_str().to_string()).unwrap_or_default(),
                            was.1.map(money).unwrap_or_default(),
                            date.as_str(),
                            money(amt)
                        ),
                    );
                }
            }
            let region = facts::region(&mut self.tx, &input.file.hash, &at);
            let o = line(
                &mut self.tx,
                Line {
                    id,
                    doc: &feed,
                    region,
                    account: &acct,
                    commodity: &cur,
                    mask: &spec.mask,
                    date: date.clone(),
                    amount: amt,
                    description: name.to_string(),
                    match_text: name.to_string(),
                    reference: t.leaf("CHECKNUM").map(str::to_string),
                    status: status("posted"),
                },
            );
            self.tx.add(&o, &k("bank.ofx/fitid"), Value::str(fitid));
            self.tx.add(&o, &k("bank.ofx/trntype"), Value::str(trntype));
            self.tx
                .add(&o, &k("bank.ofx/dtposted"), Value::str(raw_date));
            if let Some(m) = memo {
                self.tx.add(&o, &k("bank.ofx/memo"), Value::str(m));
            }
            lines.push((date, amt));
        }
        let total: Decimal = lines.iter().map(|(_, a)| *a).sum();
        for (tag, kind) in [("LEDGERBAL", "ledger"), ("AVAILBAL", "available")] {
            let Some(b) = stmt.child(tag) else { continue };
            let at = format!("{path}/{tag}");
            let (Some(amt), Some((asof, _, _))) = (
                b.leaf("BALAMT").and_then(|t| amount(t).ok()),
                b.leaf("DTASOF").and_then(ofx_date),
            ) else {
                self.press("unreadable", &at, "BALAMT or DTASOF missing or unreadable");
                continue;
            };
            let id = format!("{}|ofx|{}|{tag}|{}", self.src(), spec.id, asof.as_str());
            let o = facts::observation(&mut self.tx, &id, "balance");
            self.tx.add(&o, &k("obs/doc"), ValIn::Ref(sdoc.clone()));
            let region = facts::region(&mut self.tx, &input.file.hash, &at);
            self.tx.add(&o, &k("obs/region"), ValIn::Ref(region));
            self.tx.add(&o, &k("obs/account"), ValIn::Ref(acct.clone()));
            self.tx
                .add(&o, &k("obs/commodity"), ValIn::Ref(cur.clone()));
            self.tx.add(&o, &k("obs/date"), Value::Date(asof.clone()));
            self.tx.add(&o, &k("obs/amount"), Value::Dec(amt));
            if let Some(v) = kwv(&format!("bank.ofx.balance/{kind}")) {
                self.tx.add(&o, &k("bank.ofx/balance-kind"), v);
            }
            if kind == "ledger" {
                self.check_ledger(&at, &spec.id, &asof, amt, total);
            }
        }
        Ok(())
    }

    /// The ledger balance against the running balances a CSV download of the same account gave for the
    /// same day: the day's closing balance must be one a line shows, and the day's opening plus its lines.
    fn check_ledger(
        &mut self,
        at: &str,
        acct_id: &str,
        asof: &Date,
        ledger: Decimal,
        total: Decimal,
    ) {
        let prefix = format!("{}|csv|", self.src());
        let mut by_day: BTreeMap<Date, Vec<(Decimal, Decimal)>> = BTreeMap::new();
        for e in earlier(self.db, acct_id, &prefix) {
            if let (Some(d), Some(a), Some(b)) = (
                date_of(self.db, e),
                dec_of(self.db, e, "obs/amount"),
                dec_of(self.db, e, "bank.csv/balance-after"),
            ) && d <= *asof
            {
                by_day.entry(d).or_default().push((a, b));
            }
        }
        let Some((day, ls)) = by_day.iter().next_back() else {
            self.press(
                "balance-unchecked",
                at,
                format!(
                    "no other witness of this account's balance yet; the ledger balance {} less this file's lines implies an opening of {}",
                    money(ledger),
                    money(ledger - total)
                ),
            );
            return;
        };
        let sum: Decimal = ls.iter().map(|(a, _)| *a).sum();
        let shown = ls.iter().any(|(_, b)| *b == ledger);
        let adds_up = ls.iter().any(|(a, b)| *b - *a + sum == ledger);
        if !(shown && adds_up) {
            let closing = ls.iter().map(|(_, b)| *b).collect::<Vec<_>>();
            self.press(
                "balance-disagrees",
                at,
                format!(
                    "the ledger balance is {} as of {}, but the CSV's running balances on {} are {}",
                    money(ledger),
                    asof.as_str(),
                    day.as_str(),
                    closing.into_iter().map(money).collect::<Vec<_>>().join(", ")
                ),
            );
        }
    }
}
