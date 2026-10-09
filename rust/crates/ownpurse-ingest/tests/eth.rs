//! The roastery's Ethereum wallet: three transactions as JSON-RPC responses, a token list, daily prices.
//! Exact quantities from hex, legs per commodity, fees, identity, block time against the books' date, and a
//! uint256 no decimal can hold.

mod common;

use common::*;
use ownpurse_facts::{Date, Db, Instant, Kw, Value};
use ownpurse_ingest::sources::eth::{Eth, U256, scaled, u256};
use rust_decimal::Decimal;

const SRC: &str = "eth:0x51ab7c3e";
const WITH: &[(&str, &str)] = &[
    ("mapping", "eth/wallet.json"),
    ("tokens", "eth/token-list.json"),
];
const TX1: &str = "ethereum:0x7d3e5a1c9b2f4e608a1d3c5b7e9f0a2c4d6e8b1a3c5d7e9f0b2a4c6d8e0f1a2b";
const TX2: &str = "ethereum:0x2a9c4e6b8d0f1a3c5e7b9d1f3a5c7e9b0d2f4a6c8e0b1d3f5a7c9e2b4d6f8a0c";
const TX3: &str = "ethereum:0xb5d7f9a1c3e5b7d9f1a3c5e7b9d1f3a5c7e9b1d3f5a7c9e1b3d5f7a9c1e3b5d7";
const USDC: &str = "USDC@ethereum:0x0c5d7e2f9a1b3c4d6e8f0a2b4c6d8e0f1a3b5c79";

fn one(db: &Db, file: &str) -> Applied {
    ingest(
        db,
        &Eth,
        SRC,
        &format!("eth/{file}"),
        WITH,
        "2026-04-01T08:00:00Z",
    )
}

fn all() -> Applied {
    let mut r = one(&core(), "2026-03-02-eth-in.rpc.json");
    for f in [
        "2026-03-21-usdc-in.rpc.json",
        "2026-03-26-usdc-out.rpc.json",
        "prices-2026-03.csv",
    ] {
        r = one(&r.db, f);
    }
    r
}

fn dec(s: &str) -> Value {
    Value::Dec(s.parse::<Decimal>().unwrap())
}

fn obs(db: &Db, id: &str, attr: &'static str) -> Option<Value> {
    get(db, "obs/id", id, attr)
}

fn code(db: &Db, id: &str) -> Option<Value> {
    let c = obs(db, id, "obs/commodity")?.as_ref()?;
    db.get(c, &Kw::lit("commodity/code")).cloned()
}

#[test]
fn every_file_loads_and_reloading_changes_nothing() {
    let r = all();
    assert_eq!(
        find(
            &r.db,
            "obs/kind",
            Value::Kw(Kw::lit("obs.kind/chain-transaction"))
        )
        .len(),
        3
    );
    for f in [
        "2026-03-02-eth-in.rpc.json",
        "2026-03-21-usdc-in.rpc.json",
        "2026-03-26-usdc-out.rpc.json",
        "prices-2026-03.csv",
    ] {
        assert_eq!(one(&r.db, f).tx, None, "{f}");
    }
}

#[test]
fn ether_in_is_exact_and_its_gas_is_not_ours() {
    let r = all();
    let leg = format!("{TX1}:value");
    assert_eq!(
        obs(&r.db, &leg, "obs/amount"),
        Some(dec("0.050000000000000000"))
    );
    assert_eq!(code(&r.db, &leg), Some(Value::str("ETH")));
    assert_eq!(
        obs(&r.db, &leg, "obs/counterparty"),
        Some(Value::str("Exchange (hot wallet)"))
    );
    assert_eq!(
        obs(&r.db, &format!("{TX1}:fee"), "obs/amount"),
        None,
        "the exchange paid the gas"
    );
    assert_eq!(
        obs(&r.db, TX1, "obs/date"),
        Some(Value::Date(Date::parse("2026-03-02").unwrap()))
    );
    let eth =
        r.db.lookup(&Kw::lit("commodity/code"), &Value::str("ETH"))
            .unwrap();
    assert_eq!(
        r.db.get(eth, &Kw::lit("commodity/scale")),
        Some(&Value::Int(18))
    );
}

#[test]
fn usdc_in_is_exact_and_dated_where_the_books_are_kept() {
    let r = all();
    let leg = format!("{TX2}:203");
    assert_eq!(obs(&r.db, &leg, "obs/amount"), Some(dec("1250.000000")));
    assert_eq!(code(&r.db, &leg), Some(Value::str(USDC)));
    assert_eq!(
        obs(&r.db, &leg, "obs/counterparty"),
        Some(Value::str("Fernhill Grocers"))
    );
    let usdc =
        r.db.lookup(&Kw::lit("commodity/code"), &Value::str(USDC))
            .unwrap();
    assert_eq!(
        r.db.get(usdc, &Kw::lit("commodity/scale")),
        Some(&Value::Int(6))
    );
    // 03:41 UTC on the 22nd is 22:41 on the 21st in Chicago: the date is the books' date, the instant exact
    assert_eq!(
        obs(&r.db, TX2, "obs/instant"),
        Some(Value::Instant(
            Instant::parse("2026-03-22T03:41:23Z").unwrap()
        ))
    );
    assert_eq!(
        obs(&r.db, TX2, "obs/date"),
        Some(Value::Date(Date::parse("2026-03-21").unwrap()))
    );
    // the checkout contract's own event is mirrored raw, and its data reads as the invoice number
    assert_eq!(
        get(&r.db, "eth.log/id", &format!("{TX2}:205"), "eth.log/text"),
        Some(Value::str("INV-0418"))
    );
}

#[test]
fn usdc_out_pays_its_fee_in_ether() {
    let r = all();
    assert_eq!(
        obs(&r.db, &format!("{TX3}:143"), "obs/amount"),
        Some(dec("-1000.000000"))
    );
    assert_eq!(
        obs(&r.db, &format!("{TX3}:143"), "obs/counterparty"),
        Some(Value::str("Exchange (deposit address)"))
    );
    // 46,109 gas × 23,418,770,219 wei = 1,079,816,076,027,871 wei
    let fee = format!("{TX3}:fee");
    assert_eq!(
        obs(&r.db, &fee, "obs/amount"),
        Some(dec("-0.001079816076027871"))
    );
    assert_eq!(
        obs(&r.db, &fee, "eth.leg/units"),
        Some(dec("1079816076027871"))
    );
    assert_eq!(code(&r.db, &fee), Some(Value::str("ETH")));
    let tx = r.db.lookup(&Kw::lit("obs/id"), &Value::str(TX3)).unwrap();
    assert_eq!(obs(&r.db, &fee, "obs/parent"), Some(Value::Ref(tx)));
}

#[test]
fn the_unlimited_approval_is_kept_without_a_number() {
    let r = all();
    let id = format!("{TX2}:204");
    assert_eq!(
        get(&r.db, "eth.log/id", &id, "eth.log/event"),
        Some(Value::Kw(Kw::lit("eth.event/approval")))
    );
    assert_eq!(
        get(&r.db, "eth.log/id", &id, "eth.log/unlimited"),
        Some(Value::Bool(true))
    );
    assert_eq!(get(&r.db, "eth.log/id", &id, "eth.log/value"), None);
    assert_eq!(
        get(&r.db, "eth.log/id", &id, "eth.log/data"),
        Some(Value::str(&format!("0x{}", "f".repeat(64))))
    );
    assert_eq!(
        obs(&r.db, &id, "obs/amount"),
        None,
        "an allowance moves nothing"
    );
    // the parsing underneath
    assert_eq!(u256(&format!("0x{}", "f".repeat(64))), Some(U256::Max));
    assert_eq!(u256(&format!("0x1{}", "0".repeat(40))), Some(U256::TooBig));
    assert_eq!(u256("0x4a817c80"), Some(U256::Fits(1_250_000_000)));
    assert_eq!(scaled(u128::MAX, 0), None, "past 28 digits");
    assert_eq!(
        scaled(79_228_162_514_264_337_593_543_950_335, 0).map(|d| d.to_string()),
        Some("79228162514264337593543950335".into())
    );
}

#[test]
fn the_usdc_out_transaction_is_one_event_in_two_commodities() {
    let r = one(&core(), "2026-03-26-usdc-out.rpc.json");
    assert!(pressure_kinds(&r.plan).contains(&"multi-commodity".to_string()));
    let tx = r.db.lookup(&Kw::lit("obs/id"), &Value::str(TX3)).unwrap();
    let legs = find(&r.db, "obs/parent", Value::Ref(tx));
    assert_eq!(legs.len(), 2);
}

#[test]
fn prices_load_as_facts() {
    let r = all();
    let p = |c: &str, d: &str| format!("{SRC}/{c}/USD/{d}");
    assert_eq!(
        get(&r.db, "price/id", &p("ETH", "2026-03-02"), "price/value"),
        Some(dec("2214.09"))
    );
    assert_eq!(
        get(&r.db, "price/id", &p(USDC, "2026-03-21"), "price/value"),
        Some(dec("0.9997"))
    );
    assert_eq!(
        get(&r.db, "price/id", &p("ETH", "2026-03-31"), "price/at"),
        Some(Value::Instant(
            Instant::parse("2026-04-01T00:00:00Z").unwrap()
        ))
    );
    let n =
        r.db.attr(&Kw::lit("price/id"))
            .map(|a| r.db.attr_values(a.id).count())
            .unwrap();
    assert_eq!(n, 62, "31 days × ETH and USDC");
}
