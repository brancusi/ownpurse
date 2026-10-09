//! Helpers every projector uses to name the shared things: files, sources, regions, commodities, accounts,
//! documents and links. Each one asserts the thing's identity, so naming it twice is naming it once.

use crate::{Input, SourceSpec, temp};
use ownpurse_facts::{EntRef, Hash, Kw, TxData, ValIn, Value};
use rust_decimal::Decimal;

fn k(s: &'static str) -> Kw {
    Kw::lit(s)
}

pub fn file_ref(h: &Hash) -> ValIn {
    ValIn::Ref(temp(&format!("file:{}", h.hex())))
}

pub fn source_ref(s: &SourceSpec) -> ValIn {
    ValIn::Ref(temp(&format!("source:{}", s.id)))
}

/// The files and the source, with what is known about them.
pub fn envelope(input: &Input<'_>) -> TxData {
    let mut tx = TxData::new();
    let src = temp(&format!("source:{}", input.source.id));
    tx.add(&src, &k("source/id"), Value::str(&input.source.id));
    if let Ok(kind) = Kw::new(&format!("source.kind/{}", input.source.kind)) {
        tx.add(&src, &k("source/kind"), Value::Kw(kind));
    }
    tx.add(&src, &k("source/name"), Value::str(&input.source.name));
    for f in std::iter::once(&input.file).chain(&input.with) {
        let e = file(&mut tx, &f.hash);
        tx.add(&e, &k("file/name"), Value::str(f.name));
        tx.add(&e, &k("file/media-type"), Value::str(f.media_type));
        tx.add(
            &e,
            &k("file/size"),
            Value::Int(i64::try_from(f.bytes.len()).unwrap_or(i64::MAX)),
        );
        if f.role == "main" {
            continue;
        }
        // an extraction or a mapping is read alongside the main file; an extraction is derived from the
        // file it reads
        if f.role == "reads" {
            let main = file(&mut tx, &input.file.hash);
            tx.add(&main, &k("file/derived-from"), ValIn::Ref(e));
        }
    }
    tx
}

/// A drawer file by hash.
pub fn file(tx: &mut TxData, h: &Hash) -> EntRef {
    let e = temp(&format!("file:{}", h.hex()));
    tx.add(&e, &k("file/sha256"), Value::Hash(*h));
    e
}

/// A place in a file. `locator`: a JSON pointer, `row:12`, `cell:Budget!C5`, `page:2`, `page:2/box:x0,y0,x1,y1`.
pub fn region(tx: &mut TxData, file_hash: &Hash, locator: &str) -> EntRef {
    let id = format!("{}#{locator}", file_hash.hex());
    let e = temp(&format!("region:{id}"));
    tx.add(&e, &k("region/id"), Value::str(&id));
    let f = file(tx, file_hash);
    tx.add(&e, &k("region/file"), ValIn::Ref(f));
    tx.add(&e, &k("region/locator"), Value::str(locator));
    if let Some(page) = locator
        .strip_prefix("page:")
        .and_then(|r| r.split('/').next())
        .and_then(|p| p.parse().ok())
    {
        tx.add(&e, &k("region/page"), Value::Int(page));
    }
    e
}

/// A commodity by code: `USD`, `ETH`, `USDC@ethereum:0xa0b8…`.
pub fn commodity(tx: &mut TxData, code: &str, scale: Option<i64>) -> EntRef {
    let e = temp(&format!("commodity:{code}"));
    tx.add(&e, &k("commodity/code"), Value::str(code));
    if let Some(s) = scale {
        tx.add(&e, &k("commodity/scale"), Value::Int(s));
    }
    e
}

/// An account (at a source, or in a book) by its id.
pub fn account(tx: &mut TxData, id: &str) -> EntRef {
    let e = temp(&format!("account:{id}"));
    tx.add(&e, &k("account/id"), Value::str(id));
    e
}

/// A document by id, with its kind, source and the file this version came in.
pub fn document(tx: &mut TxData, input: &Input<'_>, id: &str, kind: &str) -> EntRef {
    let e = temp(&format!("doc:{id}"));
    tx.add(&e, &k("doc/id"), Value::str(id));
    if let Ok(kind) = Kw::new(&format!("doc.kind/{kind}")) {
        tx.add(&e, &k("doc/kind"), Value::Kw(kind));
    }
    tx.add(&e, &k("doc/source"), source_ref(&input.source));
    tx.add(&e, &k("doc/file"), file_ref(&input.file.hash));
    e
}

/// An observation by id, with its kind.
pub fn observation(tx: &mut TxData, id: &str, kind: &str) -> EntRef {
    let e = temp(&format!("obs:{id}"));
    tx.add(&e, &k("obs/id"), Value::str(id));
    if let Ok(kind) = Kw::new(&format!("obs.kind/{kind}")) {
        tx.add(&e, &k("obs/kind"), Value::Kw(kind));
    }
    e
}

/// A link between two entities. Its id is `<kind>|<from key>|<to key>`, so proposing it again is a no-op.
pub fn link(
    tx: &mut TxData,
    kind: &str,
    from: (&EntRef, &str),
    to: (&EntRef, &str),
    rule: &str,
) -> EntRef {
    let id = format!("{kind}|{}|{}", from.1, to.1);
    let e = temp(&format!("link:{id}"));
    tx.add(&e, &k("link/id"), Value::str(&id));
    if let Ok(kw) = Kw::new(&format!("link.kind/{kind}")) {
        tx.add(&e, &k("link/kind"), Value::Kw(kw));
    }
    tx.add(&e, &k("link/from"), ValIn::Ref(from.0.clone()));
    tx.add(&e, &k("link/to"), ValIn::Ref(to.0.clone()));
    tx.add(&e, &k("link/rule"), Value::str(rule));
    e
}

pub fn dec(d: Decimal) -> Value {
    Value::Dec(d)
}

/// `account|date|amount|text`: what link rules match witnesses of one event on. The text is upper-cased and
/// stripped to letters and digits, since every bank abbreviates differently.
pub fn fingerprint(account: &str, date: &str, amount: Decimal, text: &str) -> String {
    let t: String = text
        .chars()
        .filter(char::is_ascii_alphanumeric)
        .map(|c| c.to_ascii_uppercase())
        .collect();
    format!("{account}|{date}|{}|{t}", amount.normalize())
}
