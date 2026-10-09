//! Ethereum: JSON-RPC responses as a wallet connector saves them, one file per transaction (the
//! transaction, its receipt and its block), plus daily prices as CSV.
//!
//! Companions: `mapping` (the wallet: chain, address, time zone offsets, an address book, the price file's
//! columns) and `tokens` (a token list: contract → symbol, decimals).
//!
//! One observation per on-chain transaction touching the wallet (`obs.kind/chain-transaction`, with the block
//! time as `obs/instant` and the date in the wallet's time zone as `obs/date`), and one child leg per
//! commodity that moved: the ether value, each ERC-20 `Transfer` to or from the wallet (identity
//! `<chain>:<txhash>:<log index>`), and the fee when the wallet sent the transaction (`obs.kind/chain-fee`).
//! Gas paid by someone else is not the wallet's and is not a leg. Every log in the receipt is mirrored raw
//! (`eth.log/*`), so an `Approval` or a contract's own event is kept even though it moves nothing.
//!
//! Quantities are hex integers of the smallest unit: parsed as integers and scaled by the commodity's
//! decimals (18 for ether, 6 for USDC), never through a float. A `uint256` past what a decimal holds (28
//! digits) is kept as its hex text; the maximum (`2^256 - 1`, "unlimited" in an approval) is named as such.
//!
//! A CSV main file is prices: `price/*` facts, one per commodity per day.

use crate::snapshot::Snapshot;
use crate::{IngestError, Input, Pressure, Projection, Projector, temp};
use ownpurse_facts::value::parse_decimal;
use ownpurse_facts::{Date, Db, Instant, Kw, ValIn, Value, time};
use rust_decimal::Decimal;
use serde_json::{Map, Value as Json, json};
use std::collections::BTreeMap;

pub struct Eth;

const ID: &str = "eth.rpc@1";
const TRANSFER: &str = "0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef";
const APPROVAL: &str = "0x8c5be1e5ebec7d5bd14f71427d1e84f3dd0314c0f7b2291e5b200ac8c7c3b925";

fn k(s: &'static str) -> Kw {
    Kw::lit(s)
}

fn install() -> Vec<Json> {
    let defs = json!([
        {"ident": "eth.tx/hash", "type": "string"},
        {"ident": "eth.tx/block", "type": "int"},
        {"ident": "eth.tx/from", "type": "string"},
        {"ident": "eth.tx/to", "type": "string"},
        {"ident": "eth.tx/gas-used", "type": "int"},
        {"ident": "eth.tx/effective-gas-price", "type": "decimal", "doc": "Wei per unit of gas"},
        {"ident": "eth.tx/fee-payer", "type": "string", "doc": "Who paid the gas: the sender"},
        {"ident": "eth.leg/from", "type": "string"},
        {"ident": "eth.leg/to", "type": "string"},
        {"ident": "eth.leg/units", "type": "decimal", "doc": "The integer quantity in the commodity's smallest unit (wei, USDC's millionths)"},
        {"ident": "eth.log/id", "type": "string", "unique": "identity", "doc": "<chain>:<txhash>:<log index>"},
        {"ident": "eth.log/tx", "type": "ref", "doc": "The transaction observation the log is in"},
        {"ident": "eth.log/index", "type": "int"},
        {"ident": "eth.log/address", "type": "string", "doc": "The contract that emitted it"},
        {"ident": "eth.log/topics", "type": "string", "doc": "The topics, in order, space-separated"},
        {"ident": "eth.log/data", "type": "string", "doc": "The data, as hex"},
        {"ident": "eth.log/event", "type": "keyword", "doc": "eth.event/transfer, eth.event/approval; absent when not known"},
        {"ident": "eth.log/value", "type": "decimal", "doc": "The decoded uint256, in integer units, when it fits a decimal"},
        {"ident": "eth.log/unlimited", "type": "bool", "doc": "The value is 2^256 - 1: an unlimited allowance"},
        {"ident": "eth.log/text", "type": "string", "doc": "The data read as text, when it is printable text"},
        {"ident": "eth.token/address", "type": "string"},
        {"ident": "eth.token/chain-id", "type": "int"}
    ]);
    defs.as_array().cloned().unwrap_or_default()
}

fn owns(a: &Kw) -> bool {
    match a.as_str() {
        "obs/region" => false,
        "doc/kind" | "doc/external-id" => true,
        _ => matches!(a.ns(), "obs" | "eth.tx" | "eth.leg" | "eth.log" | "price"),
    }
}

impl Projector for Eth {
    fn id(&self) -> &'static str {
        ID
    }

    fn reads(&self) -> &'static str {
        "Ethereum JSON-RPC responses (transaction, receipt, block) per transaction, or daily prices as CSV; \
         with the wallet mapping and a token list"
    }

    fn project(&self, db: &Db, input: &Input<'_>) -> Result<Projection, IngestError> {
        let bad = |m: String| IngestError(format!("{}: {m}", input.file.name));
        let json_of = |role: &str| -> Result<Json, IngestError> {
            let f = input
                .companion(role)
                .ok_or_else(|| bad(format!("needs a {role} (--with {role}=<file>)")))?;
            serde_json::from_slice(f.bytes).map_err(|e| bad(format!("{role}: not JSON ({e})")))
        };
        let wallet = Wallet::read(&json_of("mapping")?, &json_of("tokens")?).map_err(bad)?;
        let mut p = Chain {
            input,
            w: &wallet,
            snap: Snapshot::new(),
            pressure: Vec::new(),
        };
        let first = input
            .file
            .bytes
            .iter()
            .find(|b| !b.is_ascii_whitespace())
            .copied();
        if matches!(first, Some(b'[' | b'{')) {
            let body: Json = serde_json::from_slice(input.file.bytes)
                .map_err(|e| bad(format!("not JSON ({e})")))?;
            p.transaction(&body).map_err(bad)?;
        } else {
            p.prices().map_err(bad)?;
        }
        let tx = p.snap.diff(db, &owns);
        Ok(Projection {
            install: install(),
            tx,
            pressure: p.pressure,
        })
    }
}

struct Token {
    symbol: String,
    name: String,
    decimals: u32,
}

struct Wallet {
    chain: String,
    chain_id: i64,
    address: String,
    name: String,
    native: Token,
    /// (from, offset seconds), sorted
    offsets: Vec<(Instant, i64)>,
    book: BTreeMap<String, String>,
    tokens: BTreeMap<String, Token>,
    price_cols: [String; 4],
}

fn offset_seconds(text: &str) -> Option<i64> {
    let (sign, rest) = match text.as_bytes().first()? {
        b'-' => (-1, &text[1..]),
        b'+' => (1, &text[1..]),
        _ => return None,
    };
    let (h, m) = rest.split_once(':')?;
    Some(sign * (h.parse::<i64>().ok()? * 3600 + m.parse::<i64>().ok()? * 60))
}

impl Wallet {
    fn read(m: &Json, tokens: &Json) -> Result<Wallet, String> {
        let s = |key: &str| m.get(key).and_then(Json::as_str).map(str::to_string);
        let need = |key: &str| s(key).ok_or_else(|| format!("wallet mapping needs \"{key}\""));
        let chain_id = m.get("chain_id").and_then(Json::as_i64).unwrap_or(1);
        let mut offsets = Vec::new();
        for o in m
            .get("utc_offsets")
            .and_then(Json::as_array)
            .into_iter()
            .flatten()
        {
            let from = o
                .get("from")
                .and_then(Json::as_str)
                .and_then(|t| Instant::parse(t).ok());
            let off = o
                .get("offset")
                .and_then(Json::as_str)
                .and_then(offset_seconds);
            match (from, off) {
                (Some(f), Some(x)) => offsets.push((f, x)),
                _ => return Err(format!("wallet mapping: bad utc_offsets entry {o}")),
            }
        }
        offsets.sort();
        let native = m.get("native");
        let ns = |key: &str, d: &str| {
            native
                .and_then(|n| n.get(key))
                .and_then(Json::as_str)
                .unwrap_or(d)
                .to_string()
        };
        let mut list = BTreeMap::new();
        for t in tokens
            .get("tokens")
            .and_then(Json::as_array)
            .into_iter()
            .flatten()
        {
            if t.get("chainId").and_then(Json::as_i64) != Some(chain_id) {
                continue;
            }
            let (Some(addr), Some(symbol), Some(dec)) = (
                t.get("address").and_then(Json::as_str),
                t.get("symbol").and_then(Json::as_str),
                t.get("decimals").and_then(Json::as_u64),
            ) else {
                continue;
            };
            list.insert(
                addr.to_ascii_lowercase(),
                Token {
                    symbol: symbol.to_string(),
                    name: t
                        .get("name")
                        .and_then(Json::as_str)
                        .unwrap_or(symbol)
                        .to_string(),
                    decimals: u32::try_from(dec).map_err(|_| "token decimals out of range")?,
                },
            );
        }
        let pc = |key: &str| {
            m.get("prices")
                .and_then(|p| p.get("columns"))
                .and_then(|c| c.get(key))
                .and_then(Json::as_str)
                .unwrap_or(key)
                .to_string()
        };
        Ok(Wallet {
            chain: need("chain")?,
            chain_id,
            address: need("address")?.to_ascii_lowercase(),
            name: s("name").unwrap_or_else(|| "Wallet".into()),
            native: Token {
                symbol: ns("symbol", "ETH"),
                name: ns("name", "Ether"),
                decimals: native
                    .and_then(|n| n.get("decimals"))
                    .and_then(Json::as_u64)
                    .and_then(|d| u32::try_from(d).ok())
                    .unwrap_or(18),
            },
            offsets,
            book: m
                .get("address_book")
                .and_then(Json::as_object)
                .map(|b| {
                    b.iter()
                        .filter_map(|(a, n)| {
                            n.as_str().map(|n| (a.to_ascii_lowercase(), n.to_string()))
                        })
                        .collect()
                })
                .unwrap_or_default(),
            tokens: list,
            price_cols: [pc("date"), pc("base"), pc("quote"), pc("close")],
        })
    }

    /// The wallet holder's calendar date for a block time, from the mapping's offsets.
    fn local_date(&self, unix: i64, at: &Instant) -> Option<Date> {
        let off = self
            .offsets
            .iter()
            .rev()
            .find(|(from, _)| from <= at)
            .map(|(_, o)| *o)?;
        Some(Date::from_days((unix + off).div_euclid(86_400)))
    }

    fn name_of(&self, addr: &str) -> Option<&str> {
        self.book.get(addr).map(String::as_str)
    }
}

/// A `uint256` as the RPC gives it (`0x…` hex).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum U256 {
    Fits(u128),
    /// 2^256 - 1: an unlimited allowance
    Max,
    /// more than 128 bits, and not the maximum
    TooBig,
}

pub fn u256(hex: &str) -> Option<U256> {
    let h = hex.strip_prefix("0x").unwrap_or(hex);
    if h.is_empty() || !h.bytes().all(|b| b.is_ascii_hexdigit()) || h.len() > 64 {
        return None;
    }
    if h.len() == 64 && h.bytes().all(|b| b == b'f' || b == b'F') {
        return Some(U256::Max);
    }
    let digits = h.trim_start_matches('0');
    if digits.len() > 32 {
        return Some(U256::TooBig);
    }
    Some(U256::Fits(if digits.is_empty() {
        0
    } else {
        u128::from_str_radix(digits, 16).ok()?
    }))
}

/// Integer units scaled by `decimals`, exactly: `1250000000` at 6 → `1250.000000`. `None` past what a
/// decimal holds (96 bits of mantissa, about 28 digits).
pub fn scaled(units: u128, decimals: u32) -> Option<Decimal> {
    Decimal::try_from_i128_with_scale(i128::try_from(units).ok()?, decimals).ok()
}

fn hex_int(o: &Map<String, Json>, f: &str) -> Option<u128> {
    match u256(o.get(f)?.as_str()?)? {
        U256::Fits(n) => Some(n),
        _ => None,
    }
}

fn s<'j>(o: &'j Map<String, Json>, f: &str) -> Option<&'j str> {
    o.get(f).and_then(Json::as_str)
}

/// The address in a 32-byte topic.
fn topic_address(t: &str) -> Option<String> {
    let h = t.strip_prefix("0x")?;
    (h.len() == 64).then(|| format!("0x{}", h[24..].to_ascii_lowercase()))
}

/// Printable text in a hex data word, padded with zeros (a contract's bytes32 reference).
fn ascii(data: &str) -> Option<String> {
    let h = data.strip_prefix("0x")?;
    let bytes: Option<Vec<u8>> = (0..h.len() / 2)
        .map(|i| u8::from_str_radix(h.get(2 * i..2 * i + 2)?, 16).ok())
        .collect();
    let bytes = bytes?;
    let end = bytes.iter().rposition(|b| *b != 0)? + 1;
    let text = &bytes[..end];
    (text.len() >= 3 && text.iter().all(|b| b.is_ascii_graphic() || *b == b' '))
        .then(|| String::from_utf8_lossy(text).into_owned())
}

struct Chain<'a, 'i> {
    input: &'a Input<'i>,
    w: &'a Wallet,
    snap: Snapshot,
    pressure: Vec<Pressure>,
}

struct Leg {
    id: String,
    kind: &'static str,
    amount: Decimal,
    units: u128,
    commodity: String,
    from: String,
    to: String,
    region: String,
}

impl Chain<'_, '_> {
    fn press(&mut self, kind: &str, at: impl Into<String>, detail: impl Into<String>) {
        self.pressure.push(Pressure::new(kind, at, detail));
    }

    fn add(&mut self, key: &str, a: &'static str, v: impl Into<ValIn>) {
        self.snap.add(key, &k(a), v);
    }

    fn src(&self) -> &str {
        &self.input.source.id
    }

    fn source_key(&mut self) -> String {
        let key = format!("source:{}", self.src());
        self.snap
            .refer(&key, &k("source/id"), Value::str(self.src()));
        key
    }

    fn file_key(&mut self) -> String {
        let h = self.input.file.hash;
        let key = format!("file:{}", h.hex());
        self.snap.refer(&key, &k("file/sha256"), Value::Hash(h));
        key
    }

    fn region(&mut self, pointer: &str) -> String {
        let h = self.input.file.hash;
        let id = format!("{}#{pointer}", h.hex());
        let key = format!("region:{id}");
        self.snap.ent(&key, &k("region/id"), Value::str(&id));
        let fk = self.file_key();
        self.add(&key, "region/file", ValIn::Ref(temp(&fk)));
        self.add(&key, "region/locator", Value::str(pointer));
        key
    }

    /// The native coin: `ETH`. Commodities only accrete: a re-read never retracts a name someone else gave.
    fn native(&mut self) -> String {
        let code = self.w.native.symbol.clone();
        let key = format!("commodity:{code}");
        self.snap.ent(&key, &k("commodity/code"), Value::str(&code));
        self.add(&key, "commodity/name", Value::str(&self.w.native.name));
        self.add(
            &key,
            "commodity/kind",
            Value::Kw(k("commodity.kind/crypto")),
        );
        self.add(
            &key,
            "commodity/scale",
            Value::Int(i64::from(self.w.native.decimals)),
        );
        key
    }

    /// A token: `USDC@ethereum:0x…`. The symbol is a label; the chain and contract are the identity.
    fn token(&mut self, contract: &str) -> Option<(String, u32)> {
        let t = self.w.tokens.get(contract)?;
        let code = format!("{}@{}:{contract}", t.symbol, self.w.chain);
        let (name, decimals) = (t.name.clone(), t.decimals);
        let key = format!("commodity:{code}");
        self.snap.ent(&key, &k("commodity/code"), Value::str(&code));
        self.add(&key, "commodity/name", Value::str(&name));
        self.add(&key, "commodity/kind", Value::Kw(k("commodity.kind/token")));
        self.add(&key, "commodity/scale", Value::Int(i64::from(decimals)));
        self.add(&key, "eth.token/address", Value::str(contract));
        self.add(&key, "eth.token/chain-id", Value::Int(self.w.chain_id));
        Some((key, decimals))
    }

    fn wallet_account(&mut self) -> String {
        let id = format!("{}:{}", self.w.chain, self.w.address);
        let key = format!("account:{id}");
        self.snap.ent(&key, &k("account/id"), Value::str(&id));
        self.add(&key, "account/name", Value::str(&self.w.name));
        self.add(&key, "account/type", Value::Kw(k("account.type/wallet")));
        let src = self.source_key();
        self.add(&key, "account/source", ValIn::Ref(temp(&src)));
        key
    }

    fn transaction(&mut self, body: &Json) -> Result<(), String> {
        let calls = body.as_array().ok_or("want an array of RPC exchanges")?;
        let result = |method: &str| -> Option<(usize, &Map<String, Json>)> {
            calls.iter().enumerate().find_map(|(i, c)| {
                (c.get("method").and_then(Json::as_str) == Some(method))
                    .then(|| {
                        c.pointer("/response/result")
                            .and_then(Json::as_object)
                            .map(|r| (i, r))
                    })
                    .flatten()
            })
        };
        let (_, tx) =
            result("eth_getTransactionByHash").ok_or("no eth_getTransactionByHash result")?;
        let (ri, receipt) =
            result("eth_getTransactionReceipt").ok_or("no eth_getTransactionReceipt result")?;
        let (_, block) = result("eth_getBlockByNumber").ok_or("no eth_getBlockByNumber result")?;
        let hash = s(tx, "hash")
            .ok_or("transaction has no hash")?
            .to_ascii_lowercase();
        let from = s(tx, "from").unwrap_or("").to_ascii_lowercase();
        let to = s(tx, "to").unwrap_or("").to_ascii_lowercase();
        let me = self.w.address.clone();
        let chain = self.w.chain.clone();
        let tid = format!("{chain}:{hash}");

        let unix = hex_int(block, "timestamp")
            .and_then(|t| i64::try_from(t).ok())
            .ok_or("block has no timestamp")?;
        let instant = Instant::from_unix_millis(unix * 1000);
        let ok = s(receipt, "status") == Some("0x1");

        let dk = format!("doc:{tid}");
        self.snap.ent(&dk, &k("doc/id"), Value::str(&tid));
        self.add(&dk, "doc/kind", Value::Kw(k("doc.kind/eth-transaction")));
        self.add(&dk, "doc/external-id", Value::str(&hash));
        let (src, file) = (self.source_key(), self.file_key());
        self.add(&dk, "doc/source", ValIn::Ref(temp(&src)));
        self.add(&dk, "doc/file", ValIn::Ref(temp(&file)));

        let acct = self.wallet_account();
        let ok_key = format!("obs:{tid}");
        self.snap.ent(&ok_key, &k("obs/id"), Value::str(&tid));
        self.add(
            &ok_key,
            "obs/kind",
            Value::Kw(k("obs.kind/chain-transaction")),
        );
        self.add(&ok_key, "obs/doc", ValIn::Ref(temp(&dk)));
        self.add(&ok_key, "obs/account", ValIn::Ref(temp(&acct)));
        self.add(&ok_key, "obs/instant", Value::Instant(instant.clone()));
        match self.w.local_date(unix, &instant) {
            Some(d) => {
                if d != instant.date() {
                    self.press(
                        "date",
                        tid.clone(),
                        format!(
                            "the block time {} is {} in UTC but {} where the books are kept; obs/date is the local date",
                            instant.as_str(),
                            instant.date().as_str(),
                            d.as_str()
                        ),
                    );
                }
                self.add(&ok_key, "obs/date", Value::Date(d));
            }
            None => self.press(
                "date",
                tid.clone(),
                "no UTC offset in the mapping covers this block time; no obs/date",
            ),
        }
        self.add(&ok_key, "obs/reference", Value::str(&hash));
        self.add(
            &ok_key,
            "obs/status",
            Value::Kw(k(if ok {
                "obs.status/success"
            } else {
                "obs.status/failed"
            })),
        );
        self.add(&ok_key, "eth.tx/hash", Value::str(&hash));
        if let Some(n) = hex_int(tx, "blockNumber").and_then(|n| i64::try_from(n).ok()) {
            self.add(&ok_key, "eth.tx/block", Value::Int(n));
        }
        self.add(&ok_key, "eth.tx/from", Value::str(&from));
        if !to.is_empty() {
            self.add(&ok_key, "eth.tx/to", Value::str(&to));
        }
        self.add(&ok_key, "eth.tx/fee-payer", Value::str(&from));
        let gas = hex_int(receipt, "gasUsed");
        let price = hex_int(receipt, "effectiveGasPrice");
        if let Some(g) = gas.and_then(|g| i64::try_from(g).ok()) {
            self.add(&ok_key, "eth.tx/gas-used", Value::Int(g));
        }
        if let Some(p) = price.and_then(|p| scaled(p, 0)) {
            self.add(&ok_key, "eth.tx/effective-gas-price", Value::Dec(p));
        }

        let mut legs: Vec<Leg> = Vec::new();
        let native = self.native();
        let decimals = self.w.native.decimals;
        // the ether value, when the transaction succeeded
        if let Some(v) = hex_int(tx, "value").filter(|v| *v > 0 && ok)
            && (from == me || to == me)
        {
            let q = scaled(v, decimals).ok_or("ether value too large for a decimal")?;
            legs.push(Leg {
                id: format!("{tid}:value"),
                kind: "chain-transfer",
                amount: if to == me { q } else { -q },
                units: v,
                commodity: native.clone(),
                from: from.clone(),
                to: to.clone(),
                region: "/0/response/result/value".into(),
            });
        }
        // the fee: the sender's, failed or not
        if from == me {
            match (gas, price) {
                (Some(g), Some(p)) => {
                    let wei = g.checked_mul(p).ok_or("fee overflows")?;
                    let q = scaled(wei, decimals).ok_or("fee too large for a decimal")?;
                    legs.push(Leg {
                        id: format!("{tid}:fee"),
                        kind: "chain-fee",
                        amount: -q,
                        units: wei,
                        commodity: native.clone(),
                        from: from.clone(),
                        to: String::new(),
                        region: format!("/{ri}/response/result/gasUsed"),
                    });
                    if receipt.contains_key("l1Fee") {
                        self.press(
                            "fee",
                            tid.clone(),
                            "the receipt has an l1Fee (a rollup): it is part of the fee and is not added",
                        );
                    }
                }
                _ => self.press(
                    "fee",
                    tid.clone(),
                    "no gasUsed or effectiveGasPrice: the fee is unknown",
                ),
            }
        }

        // the logs: every one mirrored, transfers touching the wallet become legs
        let logs: Vec<Json> = receipt
            .get("logs")
            .and_then(Json::as_array)
            .cloned()
            .unwrap_or_default();
        let mut commodities = BTreeMap::new();
        for (i, lg) in logs.iter().enumerate() {
            let Some(lg) = lg.as_object() else { continue };
            let idx = hex_int(lg, "logIndex").unwrap_or(0);
            let lid = format!("{tid}:{idx}");
            let pointer = format!("/{ri}/response/result/logs/{i}");
            let contract = s(lg, "address").unwrap_or("").to_ascii_lowercase();
            let topics: Vec<String> = lg
                .get("topics")
                .and_then(Json::as_array)
                .into_iter()
                .flatten()
                .filter_map(Json::as_str)
                .map(str::to_ascii_lowercase)
                .collect();
            let data = s(lg, "data").unwrap_or("0x").to_ascii_lowercase();
            let lk = format!("log:{lid}");
            self.snap.ent(&lk, &k("eth.log/id"), Value::str(&lid));
            self.add(&lk, "eth.log/tx", ValIn::Ref(temp(&ok_key)));
            self.add(
                &lk,
                "eth.log/index",
                Value::Int(i64::try_from(idx).unwrap_or(0)),
            );
            self.add(&lk, "eth.log/address", Value::str(&contract));
            self.add(&lk, "eth.log/topics", Value::str(&topics.join(" ")));
            self.add(&lk, "eth.log/data", Value::str(&data));
            let event = match topics.first().map(String::as_str) {
                Some(TRANSFER) if topics.len() == 3 => Some("transfer"),
                Some(APPROVAL) if topics.len() == 3 => Some("approval"),
                _ => None,
            };
            let Some(event) = event else {
                let text = ascii(&data);
                if let Some(t) = &text {
                    self.add(&lk, "eth.log/text", Value::str(t));
                }
                self.press(
                    "unknown-event",
                    pointer,
                    format!(
                        "an event from {contract} this projector cannot decode; mirrored raw{}",
                        text.map(|t| format!(" (its data reads as text: {t:?})"))
                            .unwrap_or_default()
                    ),
                );
                continue;
            };
            if let Ok(kw) = Kw::new(&format!("eth.event/{event}")) {
                self.add(&lk, "eth.log/event", Value::Kw(kw));
            }
            let value = u256(&data);
            match value {
                Some(U256::Fits(n)) => match scaled(n, 0) {
                    Some(d) => self.add(&lk, "eth.log/value", Value::Dec(d)),
                    None => self.press(
                        "precision",
                        pointer.clone(),
                        format!(
                            "{n} is past the 28 digits a decimal holds; kept as hex in eth.log/data"
                        ),
                    ),
                },
                Some(U256::Max) => {
                    self.add(&lk, "eth.log/unlimited", Value::Bool(true));
                    self.press(
                        "precision",
                        pointer.clone(),
                        "2^256 - 1 (78 digits) does not fit a decimal (28 digits): stored as eth.log/unlimited, \
                         the hex kept in eth.log/data",
                    );
                }
                Some(U256::TooBig) => self.press(
                    "precision",
                    pointer.clone(),
                    "a uint256 past 128 bits; kept as hex in eth.log/data",
                ),
                None => self.press("unreadable", pointer.clone(), "the data is not one uint256"),
            }
            let (a, b) = (
                topic_address(&topics[1]).unwrap_or_default(),
                topic_address(&topics[2]).unwrap_or_default(),
            );
            if event == "approval" {
                if a == me {
                    self.press(
                        "approval",
                        pointer,
                        "the wallet allowed a spender to move its tokens: nothing moved, so no leg; mirrored",
                    );
                }
                continue;
            }
            if a != me && b != me {
                continue;
            }
            let Some(U256::Fits(units)) = value else {
                self.press(
                    "precision",
                    pointer,
                    "a transfer amount that does not fit; no leg",
                );
                continue;
            };
            let Some((ck, decimals)) = self.token(&contract) else {
                self.press(
                    "unknown-token",
                    pointer,
                    format!(
                        "{contract} is not in the token list: its decimals are unknown, so no leg"
                    ),
                );
                continue;
            };
            let q = scaled(units, decimals).ok_or("token amount too large for a decimal")?;
            commodities.insert(ck.clone(), ());
            legs.push(Leg {
                id: lid,
                kind: "chain-transfer",
                amount: if b == me { q } else { -q },
                units,
                commodity: ck,
                from: a,
                to: b,
                region: pointer,
            });
        }
        if legs.iter().any(|l| l.commodity == native) && !commodities.is_empty() {
            self.press(
                "multi-commodity",
                tid.clone(),
                "one transaction moves two commodities (a token, and ether for the fee): legs per commodity under one observation",
            );
        }
        if legs.is_empty() {
            self.press(
                "no-legs",
                tid.clone(),
                "the transaction touches the wallet but moves nothing it holds",
            );
        }

        for (i, l) in legs.iter().enumerate() {
            let key = format!("obs:{}", l.id);
            self.snap.ent(&key, &k("obs/id"), Value::str(&l.id));
            if let Ok(kw) = Kw::new(&format!("obs.kind/{}", l.kind)) {
                self.add(&key, "obs/kind", Value::Kw(kw));
            }
            self.add(&key, "obs/parent", ValIn::Ref(temp(&ok_key)));
            self.add(&key, "obs/doc", ValIn::Ref(temp(&dk)));
            self.add(&key, "obs/index", Value::Int(i64::try_from(i).unwrap_or(0)));
            self.add(&key, "obs/account", ValIn::Ref(temp(&acct)));
            self.add(&key, "obs/amount", Value::Dec(l.amount));
            self.add(&key, "obs/commodity", ValIn::Ref(temp(&l.commodity)));
            self.add(&key, "obs/instant", Value::Instant(instant.clone()));
            if let Some(scaled_units) = scaled(l.units, 0) {
                self.add(&key, "eth.leg/units", Value::Dec(scaled_units));
            }
            self.add(&key, "eth.leg/from", Value::str(&l.from));
            if !l.to.is_empty() {
                self.add(&key, "eth.leg/to", Value::str(&l.to));
            }
            let other = if l.from == me { &l.to } else { &l.from };
            if let Some(n) = self.w.name_of(other) {
                let n = n.to_string();
                self.add(&key, "obs/counterparty", Value::str(&n));
            } else if !other.is_empty() {
                let o = other.clone();
                self.add(&key, "obs/counterparty", Value::str(&o));
            }
            let rk = self.region(&l.region);
            self.add(&key, "obs/region", ValIn::Ref(temp(&rk)));
        }
        Ok(())
    }

    /// Daily closes: `date,base,quote,close`. One `price` per base per day, at the end of the UTC day.
    fn prices(&mut self) -> Result<(), String> {
        let mut rdr = csv::ReaderBuilder::new()
            .trim(csv::Trim::All)
            .from_reader(self.input.file.bytes);
        let headers = rdr.headers().map_err(|e| e.to_string())?.clone();
        let col = |name: &str| {
            headers
                .iter()
                .position(|h| h == name)
                .ok_or_else(|| format!("no {name:?} column"))
        };
        let cols = &self.w.price_cols;
        let (dc, bc, qc, cc) = (
            col(&cols[0])?,
            col(&cols[1])?,
            col(&cols[2])?,
            col(&cols[3])?,
        );
        let src = self.source_key();
        self.press(
            "price-date",
            "prices",
            "a daily close has a date; price/at is an instant, so it is the end of that UTC day (00:00 the next day)",
        );
        self.press(
            "price-source",
            "prices",
            "the prices came with the wallet's files, so price/source is the wallet's source, not the feed",
        );
        // a symbol is not an identity: resolve it through the token list, and say when it is ambiguous
        let mut by_symbol: BTreeMap<String, Vec<String>> = BTreeMap::new();
        for (addr, t) in &self.w.tokens {
            by_symbol
                .entry(t.symbol.clone())
                .or_default()
                .push(addr.clone());
        }
        for (i, rec) in rdr.records().enumerate() {
            let rec = rec.map_err(|e| e.to_string())?;
            let at = format!("row:{}", i + 2);
            let get = |c: usize| rec.get(c).unwrap_or("").to_string();
            let (date, base, quote, close) = (get(dc), get(bc), get(qc), get(cc));
            let Ok(d) = Date::parse(&date) else {
                self.press("unreadable", at, format!("{date:?} is not a date"));
                continue;
            };
            let Ok(value) = parse_decimal(&close) else {
                self.press("unreadable", at, format!("{close:?} is not a price"));
                continue;
            };
            let ck = if base == self.w.native.symbol {
                self.native()
            } else {
                match by_symbol.get(&base).map(Vec::as_slice) {
                    Some([one]) => {
                        let one = one.clone();
                        let Some((ck, _)) = self.token(&one) else {
                            continue;
                        };
                        ck
                    }
                    Some(_) => {
                        self.press(
                            "symbol",
                            at,
                            format!("{base} names more than one token; not read"),
                        );
                        continue;
                    }
                    None => {
                        self.press(
                            "symbol",
                            at,
                            format!("{base} is not in the token list; not read"),
                        );
                        continue;
                    }
                }
            };
            let qk = format!("commodity:{quote}");
            self.snap
                .refer(&qk, &k("commodity/code"), Value::str(&quote));
            let code = ck.trim_start_matches("commodity:").to_string();
            let pid = format!("{}/{code}/{quote}/{}", self.src(), d.as_str());
            let pk = format!("price:{pid}");
            self.snap.ent(&pk, &k("price/id"), Value::str(&pid));
            self.add(&pk, "price/commodity", ValIn::Ref(temp(&ck)));
            self.add(&pk, "price/quote", ValIn::Ref(temp(&qk)));
            self.add(&pk, "price/value", Value::Dec(value));
            let next = time::days_from_civil(
                d.as_str()[..4].parse().unwrap_or(1970),
                d.as_str()[5..7].parse().unwrap_or(1),
                d.as_str()[8..].parse().unwrap_or(1),
            ) + 1;
            let end = Instant::from_unix_millis(next * 86_400_000);
            self.add(&pk, "price/at", Value::Instant(end));
            self.add(&pk, "price/source", ValIn::Ref(temp(&src)));
        }
        Ok(())
    }
}
