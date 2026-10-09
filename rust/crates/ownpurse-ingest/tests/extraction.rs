//! extraction@1: a bank statement PDF, a check image and a receipt photo, each read by an extractor.

mod common;

use common::{at, core, find, ingest, pressure_kinds, samples, source};
use ownpurse_facts::encode::sha256;
use ownpurse_facts::{Db, Eid, Kw, Value};
use ownpurse_ingest::sources::extraction::Extraction;
use ownpurse_ingest::{FileIn, IngestError, Input, Plan};
use rust_decimal::Decimal;

const BANK: &str = "bank:ridgeway-4821";
const RECEIPTS: &str = "receipts:tallowmere";
const STATEMENT: &str = "bank/statement-2026-03.extraction.json";
const STATEMENT_PDF: &str = "bank/statement-2026-03.pdf";
const CHECK: &str = "bank/check-1042.extraction.json";
const CHECK_PNG: &str = "bank/check-1042.png";
const RECEIPT: &str = "receipts/copper-pot-2026-03-12.extraction.json";
const RECEIPT_JPG: &str = "receipts/copper-pot-2026-03-12.jpg";

const DOC: &str = "bank:ridgeway-4821/statement/ridgeway:checking:4821/2026-03-01..2026-03-31";
const CHECK_OBS: &str = "bank:ridgeway-4821/check/ridgeway:checking:4821/1042/check";

fn d(s: &str) -> Decimal {
    ownpurse_facts::value::parse_decimal(s).unwrap()
}

fn kw(s: &'static str) -> Kw {
    Kw::lit(s)
}

fn statement(db: &Db) -> common::Applied {
    ingest(
        db,
        &Extraction,
        BANK,
        STATEMENT,
        &[("reads", STATEMENT_PDF)],
        "2026-04-03T10:15:00Z",
    )
}

fn check(db: &Db) -> common::Applied {
    ingest(
        db,
        &Extraction,
        BANK,
        CHECK,
        &[("reads", CHECK_PNG)],
        "2026-03-04T16:02:00Z",
    )
}

fn receipt(db: &Db) -> common::Applied {
    ingest(
        db,
        &Extraction,
        RECEIPTS,
        RECEIPT,
        &[("reads", RECEIPT_JPG)],
        "2026-03-13T08:30:00Z",
    )
}

/// All three, in the order a person would bring them in: statement, check, receipt.
fn all() -> (Db, Plan, Plan, Plan) {
    let s = statement(&core());
    let c = check(&s.db);
    let r = receipt(&c.db);
    (r.db, s.plan, c.plan, r.plan)
}

/// Plan straight from bytes, for extractions the test changes.
fn plan_bytes(db: &Db, src: &str, main: &[u8], reads: &[u8]) -> Result<Plan, IngestError> {
    let input = Input {
        file: FileIn {
            hash: sha256(main),
            name: "edited.extraction.json",
            media_type: "",
            bytes: main,
            role: "main",
        },
        with: vec![FileIn {
            hash: sha256(reads),
            name: "read.bin",
            media_type: "",
            bytes: reads,
            role: "reads",
        }],
        source: source(src),
        observed_at: at("2026-04-05T00:00:00Z"),
    };
    ownpurse_ingest::plan(db, &Extraction, &input)
}

fn read(f: &str) -> Vec<u8> {
    std::fs::read(samples().join(f)).unwrap()
}

fn obs(db: &Db, id: &str) -> Eid {
    db.lookup(&kw("obs/id"), &Value::str(id))
        .unwrap_or_else(|| panic!("no observation {id}"))
}

fn dec_of(db: &Db, e: Eid, a: &'static str) -> Decimal {
    db.get(e, &kw(a))
        .and_then(Value::as_dec)
        .unwrap_or_else(|| panic!("{e} has no {a}"))
}

fn str_of(db: &Db, e: Eid, a: &'static str) -> String {
    db.get(e, &kw(a))
        .map(Value::text)
        .unwrap_or_else(|| panic!("{e} has no {a}"))
}

/// The readings behind one attribute of an entity.
fn readings(db: &Db, e: Eid, attr: &str) -> Vec<Eid> {
    find(db, "reading/of", Value::Ref(e))
        .into_iter()
        .filter(|r| {
            db.get(*r, &kw("reading/attr"))
                .and_then(Value::as_kw)
                .map(Kw::as_str)
                == Some(attr)
        })
        .collect()
}

fn one_reading(db: &Db, e: Eid, attr: &str, field: &str) -> Eid {
    readings(db, e, attr)
        .into_iter()
        .find(|r| db.get(*r, &kw("reading/field")).and_then(Value::as_str) == Some(field))
        .unwrap_or_else(|| panic!("no reading of {attr} from field {field}"))
}

fn locator(db: &Db, reading: Eid) -> String {
    let region = db
        .get(reading, &kw("reading/region"))
        .and_then(Value::as_ref)
        .unwrap();
    str_of(db, region, "region/locator")
}

fn check_failures(p: &Plan) -> Vec<String> {
    p.pressure
        .iter()
        .filter(|x| x.kind == "check-failed")
        .map(|x| x.line())
        .collect()
}

#[test]
fn all_three_load_and_check_themselves() {
    let (db, s, c, r) = all();
    for p in [&s, &c, &r] {
        assert!(check_failures(p).is_empty(), "{:?}", check_failures(p));
    }
    // one document per file, of the right kind
    for (id, kind) in [
        (DOC.to_string(), "doc.kind/bank-statement"),
        (
            "bank:ridgeway-4821/check/ridgeway:checking:4821/1042".to_string(),
            "doc.kind/check",
        ),
        (
            format!(
                "receipts:tallowmere/receipt/{}",
                sha256(&read(RECEIPT_JPG)).hex()
            ),
            "doc.kind/receipt",
        ),
    ] {
        let e = db
            .lookup(&kw("doc/id"), &Value::str(&id))
            .unwrap_or_else(|| panic!("no document {id}"));
        assert_eq!(str_of(&db, e, "doc/kind"), kind);
        // the document names both the extraction and the file it reads
        assert_eq!(
            db.values(e, db.attr(&kw("doc/file")).unwrap().id).count(),
            2
        );
    }
    let doc = db.lookup(&kw("doc/id"), &Value::str(DOC)).unwrap();
    assert_eq!(str_of(&db, doc, "doc/period-start"), "2026-03-01");
    assert_eq!(str_of(&db, doc, "doc/period-end"), "2026-03-31");
    // the shared account identity, reached by all three
    let acct = db
        .lookup(&kw("account/id"), &Value::str("ridgeway:checking:4821"))
        .unwrap();
    assert_eq!(str_of(&db, acct, "account/mask"), "4821");
    assert_eq!(str_of(&db, acct, "account/name"), "Operating checking");
    assert_eq!(
        str_of(&db, acct, "account/institution"),
        "Ridgeway Federal Bank"
    );
    let usd = db
        .get(acct, &kw("account/commodity"))
        .and_then(Value::as_ref)
        .unwrap();
    assert_eq!(str_of(&db, usd, "commodity/code"), "USD");
    for id in [format!("{DOC}/line/5"), CHECK_OBS.to_string()] {
        assert_eq!(
            db.get(obs(&db, &id), &kw("obs/account")),
            Some(&Value::Ref(acct))
        );
    }
    let rec = find(&db, "obs/kind", Value::Kw(kw("obs.kind/receipt")));
    assert_eq!(rec.len(), 1);
    assert_eq!(db.get(rec[0], &kw("obs/account")), Some(&Value::Ref(acct)));
    // the extractor is on the extraction file
    let ex = db
        .lookup(&kw("file/sha256"), &Value::Hash(sha256(&read(STATEMENT))))
        .unwrap();
    assert_eq!(str_of(&db, ex, "extraction/extractor"), "agent:tally");
    // the statement says where its years came from
    assert!(pressure_kinds(&s).contains(&"missing-year".to_string()));
    assert!(pressure_kinds(&s).contains(&"embedded-copy".to_string()));
}

#[test]
fn reingest_changes_nothing() {
    let s = statement(&core());
    assert!(s.tx.is_some());
    assert!(statement(&s.db).tx.is_none());
    let c = check(&s.db);
    assert!(check(&c.db).tx.is_none());
    let r = receipt(&c.db);
    assert!(receipt(&r.db).tx.is_none());
}

#[test]
fn an_extraction_of_another_file_is_refused() {
    let e = plan_bytes(&core(), BANK, &read(STATEMENT), &read(CHECK_PNG)).unwrap_err();
    assert!(e.0.contains("give the file it was made from"), "{e}");
    // and one with no file at all
    let main = read(STATEMENT);
    let input = Input {
        file: FileIn {
            hash: sha256(&main),
            name: STATEMENT,
            media_type: "",
            bytes: &main,
            role: "main",
        },
        with: vec![],
        source: source(BANK),
        observed_at: at("2026-04-05T00:00:00Z"),
    };
    assert!(ownpurse_ingest::plan(&core(), &Extraction, &input).is_err());
}

#[test]
fn a_tampered_statement_is_reported_not_fixed() {
    let mut j: serde_json::Value = serde_json::from_slice(&read(STATEMENT)).unwrap();
    // the dinner misread by a dime; the closing balance and the running balances no longer follow
    j["items"][4]["fields"]["amount"]["value"] = "-186.30".into();
    j["fields"]["deposits_count"]["value"] = "4".into();
    let p = plan_bytes(
        &core(),
        BANK,
        &serde_json::to_vec(&j).unwrap(),
        &read(STATEMENT_PDF),
    )
    .unwrap();
    let fails = check_failures(&p);
    assert!(
        fails
            .iter()
            .any(|l| l.starts_with("check-failed | /balances | opening 24,310.55")),
        "{fails:?}"
    );
    assert!(
        fails
            .iter()
            .any(|l| l.contains("/items/4/fields/balance") && l.contains("running balance")),
        "{fails:?}"
    );
    assert!(
        fails.iter().any(|l| l.contains("/fields/deposits_count")),
        "{fails:?}"
    );
    // nothing was fixed: the line still says what the extraction said
    let db = core()
        .with(&p.install, &at("2026-04-05T00:00:00Z"))
        .unwrap()
        .db_after;
    let p = plan_bytes(
        &db,
        BANK,
        &serde_json::to_vec(&j).unwrap(),
        &read(STATEMENT_PDF),
    )
    .unwrap();
    let db = db
        .with(&p.tx, &at("2026-04-05T00:00:00Z"))
        .unwrap()
        .db_after;
    assert_eq!(
        dec_of(&db, obs(&db, &format!("{DOC}/line/5")), "obs/amount"),
        d("-186.30")
    );
    // a missing checks-paid partner is reported too
    j["items"][13]["fields"]["number"]["value"] = "1043".into();
    let p = plan_bytes(
        &db,
        BANK,
        &serde_json::to_vec(&j).unwrap(),
        &read(STATEMENT_PDF),
    )
    .unwrap();
    assert!(
        check_failures(&p)
            .iter()
            .any(|l| l.contains("no line reads \"CHECK 1043\""))
    );
}

#[test]
fn a_tampered_check_and_receipt_are_reported() {
    let mut j: serde_json::Value = serde_json::from_slice(&read(CHECK)).unwrap();
    j["fields"]["amount_words"]["value"] = "Three thousand two hundred fifty and 47/100".into();
    let p = plan_bytes(
        &core(),
        BANK,
        &serde_json::to_vec(&j).unwrap(),
        &read(CHECK_PNG),
    )
    .unwrap();
    assert!(
        check_failures(&p)
            .iter()
            .any(|l| l.contains("3,250.47") && l.contains("3,215.47")),
        "{:?}",
        p.pressure
    );

    let mut j: serde_json::Value = serde_json::from_slice(&read(RECEIPT)).unwrap();
    j["fields"]["tip"]["value"] = "26.00".into();
    let p = plan_bytes(
        &core(),
        RECEIPTS,
        &serde_json::to_vec(&j).unwrap(),
        &read(RECEIPT_JPG),
    )
    .unwrap();
    assert!(
        check_failures(&p)
            .iter()
            .any(|l| l.contains("/fields/total") && l.contains("tip 26.00")),
        "{:?}",
        p.pressure
    );
}

#[test]
fn every_field_has_its_own_reading_region_and_confidence() {
    let (db, ..) = all();
    let rec = find(&db, "obs/kind", Value::Kw(kw("obs.kind/receipt")))[0];
    let date = one_reading(&db, rec, "obs/date", "date");
    let total = one_reading(&db, rec, "obs/amount", "total");
    let tip = one_reading(&db, rec, "extraction.receipt/tip", "tip");
    // different boxes, different confidence
    assert_ne!(locator(&db, date), locator(&db, total));
    assert!(locator(&db, date).starts_with("page:1/box:"));
    assert_eq!(dec_of(&db, date, "reading/confidence"), d("0.97"));
    assert_eq!(dec_of(&db, total, "reading/confidence"), d("0.81"));
    assert_eq!(dec_of(&db, tip, "reading/confidence"), d("0.62"));
    assert_eq!(
        db.get(tip, &kw("reading/handwritten")),
        Some(&Value::Bool(true))
    );
    assert_eq!(str_of(&db, total, "reading/text"), "186.40");
    // the observation's confidence is the weakest of its obs/* readings (the handwritten total), not the tip
    assert_eq!(dec_of(&db, rec, "obs/confidence"), d("0.81"));
    // every field's region is on the observation too (two readings of the card share one box)
    let on_obs: std::collections::BTreeSet<&Value> = db
        .values(rec, db.attr(&kw("obs/region")).unwrap().id)
        .collect();
    let from_readings: std::collections::BTreeSet<&Value> =
        find(&db, "reading/of", Value::Ref(rec))
            .into_iter()
            .filter_map(|r| db.get(r, &kw("reading/region")))
            .collect();
    assert_eq!(on_obs, from_readings);

    // a statement line: four readings, the date's year derived and saying so
    let line = obs(&db, &format!("{DOC}/line/4"));
    let dr = one_reading(&db, line, "obs/date", "date");
    assert_eq!(str_of(&db, dr, "reading/text"), "03/09");
    assert!(str_of(&db, dr, "reading/derivation").contains("year 2026 from the statement period"));
    // the checks-paid table is a second reading of the same amount, from page 2
    let paid = one_reading(&db, line, "obs/amount", "checks-paid.amount");
    assert!(locator(&db, paid).starts_with("page:2/box:"));
    assert_eq!(str_of(&db, paid, "reading/text"), "3,215.47");
    assert_eq!(readings(&db, line, "obs/amount").len(), 2);
    // page regions know their size and unit, so a box can be drawn
    let pdf = sha256(&read(STATEMENT_PDF)).hex();
    let page = db
        .lookup(&kw("region/id"), &Value::str(&format!("{pdf}#page:1")))
        .unwrap();
    assert_eq!(str_of(&db, page, "extraction.region/unit"), "pt");
    assert_eq!(dec_of(&db, page, "extraction.region/width"), d("612"));
}

#[test]
fn the_receipt_keeps_both_totals() {
    let (db, _, _, r) = all();
    let rec = find(&db, "obs/kind", Value::Kw(kw("obs.kind/receipt")))[0];
    assert_eq!(dec_of(&db, rec, "obs/amount"), d("-186.40"));
    assert_eq!(
        dec_of(&db, rec, "extraction.receipt/printed-total"),
        d("166.40")
    );
    assert_eq!(dec_of(&db, rec, "extraction.receipt/subtotal"), d("152.80"));
    assert_eq!(dec_of(&db, rec, "extraction.receipt/tax"), d("13.60"));
    assert_eq!(dec_of(&db, rec, "extraction.receipt/tip"), d("20.00"));
    assert_eq!(str_of(&db, rec, "obs/date"), "2026-03-12");
    assert_eq!(str_of(&db, rec, "extraction.receipt/time"), "20:47");
    assert_eq!(str_of(&db, rec, "obs/counterparty"), "Copper Pot Bistro");
    assert!(pressure_kinds(&r).contains(&"two-totals".to_string()));
    assert!(pressure_kinds(&r).contains(&"no-zone".to_string()));
    let lines = find(&db, "obs/parent", Value::Ref(rec));
    assert_eq!(lines.len(), 5);
    let sum: Decimal = lines.iter().map(|l| dec_of(&db, *l, "obs/amount")).sum();
    assert_eq!(sum, d("-152.80"));
    // the fingerprint a rule matches on: same text as the bank line, the date it was spent (posted a day later)
    assert_eq!(
        str_of(&db, rec, "obs/fingerprint"),
        ownpurse_ingest::facts::fingerprint(
            "4821",
            "2026-03-12",
            d("-186.40"),
            "Copper Pot Bistro"
        )
    );
}

#[test]
fn the_checks_signer_is_kept_with_its_low_confidence() {
    let (db, _, c, _) = all();
    let ck = obs(&db, CHECK_OBS);
    assert_eq!(str_of(&db, ck, "extraction.check/signer"), "Odile Fenwick");
    let sr = one_reading(&db, ck, "extraction.check/signer", "signer");
    assert_eq!(dec_of(&db, sr, "reading/confidence"), d("0.38"));
    assert!(
        c.pressure
            .iter()
            .any(|p| p.kind == "low-confidence" && p.at == "/fields/signer")
    );
    // the signer does not drag the check down: obs/confidence is the weakest obs/* field (the memo)
    assert_eq!(dec_of(&db, ck, "obs/confidence"), d("0.86"));
    assert_eq!(str_of(&db, ck, "obs/reference"), "1042");
    assert_eq!(str_of(&db, ck, "obs/counterparty"), "Brindle & Oake Supply");
    assert_eq!(str_of(&db, ck, "obs/date"), "2026-03-03");
    assert_eq!(str_of(&db, ck, "extraction.check/micr-serial"), "1042");

    // a person confirms the signer: their own transaction, on the reading; a re-read leaves it alone
    let mut tx = ownpurse_facts::TxData::new();
    tx.add(
        &ownpurse_facts::EntRef::Id(sr),
        &kw("reading/review"),
        Value::Kw(kw("reading.review/confirmed")),
    );
    tx.meta(&kw("tx/actor"), Value::str("person:owner"));
    tx.meta(&kw("tx/reason"), Value::str("I know the signature"));
    let db = db.with(&tx, &at("2026-03-05T09:00:00Z")).unwrap().db_after;
    let again = check(&db);
    assert!(again.tx.is_none());
    assert_eq!(
        again.db.get(sr, &kw("reading/review")),
        Some(&Value::Kw(kw("reading.review/confirmed")))
    );
}

#[test]
fn amounts_are_exact_and_carry_what_rules_match_on() {
    let (db, ..) = all();
    let opening = obs(&db, &format!("{DOC}/balance/opening"));
    let closing = obs(&db, &format!("{DOC}/balance/closing"));
    assert_eq!(dec_of(&db, opening, "obs/amount"), d("24310.55"));
    assert_eq!(dec_of(&db, closing, "obs/amount"), d("13413.22"));
    assert_eq!(
        str_of(&db, closing, "extraction.balance/side"),
        "extraction.balance.side/closing"
    );
    let mut sum = Decimal::ZERO;
    for n in 1..=13 {
        sum += dec_of(&db, obs(&db, &format!("{DOC}/line/{n}")), "obs/amount");
    }
    assert_eq!(d("24310.55") + sum, d("13413.22"));
    let line = obs(&db, &format!("{DOC}/line/4"));
    assert_eq!(dec_of(&db, line, "obs/amount"), d("-3215.47"));
    assert_eq!(str_of(&db, line, "obs/date"), "2026-03-09");
    assert_eq!(str_of(&db, line, "obs/reference"), "1042");
    assert_eq!(
        dec_of(&db, line, "extraction.line/running-balance"),
        d("22931.71")
    );
    assert_eq!(
        str_of(&db, line, "obs/fingerprint"),
        ownpurse_ingest::facts::fingerprint("4821", "2026-03-09", d("-3215.47"), "CHECK 1042")
    );
    assert_eq!(
        dec_of(&db, obs(&db, CHECK_OBS), "obs/amount"),
        d("-3215.47")
    );
    let dinner = obs(&db, &format!("{DOC}/line/5"));
    assert_eq!(dec_of(&db, dinner, "obs/amount"), d("-186.40"));
    assert_eq!(str_of(&db, dinner, "obs/date"), "2026-03-13");
    let doc = db.lookup(&kw("doc/id"), &Value::str(DOC)).unwrap();
    assert_eq!(
        db.get(doc, &kw("extraction.statement/withdrawals-count")),
        Some(&Value::Int(10))
    );
    assert_eq!(
        dec_of(&db, doc, "extraction.statement/deposits-total"),
        d("4165.56")
    );
}

#[test]
fn what_backs_the_dinner_is_a_box_on_a_page() {
    // the walk from an observation to a box: obs -> reading of obs/amount -> region -> file
    let (db, ..) = all();
    let dinner = obs(&db, &format!("{DOC}/line/5"));
    let amount = one_reading(&db, dinner, "obs/amount", "amount");
    let region = db
        .get(amount, &kw("reading/region"))
        .and_then(Value::as_ref)
        .unwrap();
    let file = db
        .get(region, &kw("region/file"))
        .and_then(Value::as_ref)
        .unwrap();
    assert_eq!(
        db.get(file, &kw("file/sha256")),
        Some(&Value::Hash(sha256(&read(STATEMENT_PDF))))
    );
    assert_eq!(db.get(region, &kw("region/page")), Some(&Value::Int(1)));
    let loc = str_of(&db, region, "region/locator");
    let (_, b) = loc.split_once("/box:").unwrap();
    let c: Vec<Decimal> = b.split(',').map(d).collect();
    assert_eq!(c.len(), 4);
    assert!(
        c[0] < c[2] && c[1] < c[3] && c[2] <= d("612") && c[3] <= d("792"),
        "{loc}"
    );
    assert_eq!(str_of(&db, amount, "reading/text"), "-186.40");
}

#[test]
fn a_receipt_with_no_known_card_has_no_account() {
    let r = receipt(&core());
    assert!(pressure_kinds(&r.plan).contains(&"no-identity".to_string()));
    let rec = find(&r.db, "obs/kind", Value::Kw(kw("obs.kind/receipt")))[0];
    assert!(r.db.get(rec, &kw("obs/account")).is_none());
    assert_eq!(str_of(&r.db, rec, "extraction.receipt/card-mask"), "4821");
}
