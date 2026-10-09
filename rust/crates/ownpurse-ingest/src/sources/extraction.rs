//! An extractor's reading of a PDF or image: bank statements, checks, receipts.
//!
//! The main file is an extraction (JSON, format `extraction@1`) written by an agent or an OCR tool; the file
//! it reads is the companion with role `reads`, and its SHA-256 must be the one the extraction names. Every
//! field in an extraction carries the text as read, the parsed value, the page, a box and a confidence.
//!
//! Provenance is per field: each field becomes a **reading** entity (`reading/*`) that names the entity and
//! the attribute it supports, the raw text, the region (page and box) and the confidence. The observation
//! carries the parsed values in the shared vocabulary (`obs/*`) plus every field's region in `obs/region`,
//! and `obs/confidence` is the lowest confidence among the readings of its `obs/*` attributes.
//!
//! The extraction is checked against itself (opening + lines = closing, summary counts and totals, running
//! balances, the check's amount in words, the receipt's totals). A check that fails is pressure; nothing is
//! silently fixed. The projector writes no links: the facts carry what a rule needs to tie witnesses.

use crate::facts::fingerprint;
use crate::snapshot::Snapshot;
use crate::{FileIn, IngestError, Input, Pressure, Projection, Projector};
use ownpurse_facts::value::parse_decimal;
use ownpurse_facts::{Date, Db, Hash, Instant, Kw, ValIn, Value};
use rust_decimal::Decimal;
use serde_json::{Map, Value as Json};
use std::collections::BTreeMap;

pub struct Extraction;

/// Fields below this confidence are named in the pressure so a person can confirm them.
const LOW: Decimal = Decimal::from_parts(7, 0, 0, false, 1);

impl Projector for Extraction {
    fn id(&self) -> &'static str {
        "extraction@1"
    }

    fn reads(&self) -> &'static str {
        "An extractor's reading of a PDF or image (bank statement, check, receipt), with the file it reads"
    }

    fn project(&self, db: &Db, input: &Input<'_>) -> Result<Projection, IngestError> {
        let name = input.file.name;
        let body: Json = serde_json::from_slice(input.file.bytes)
            .map_err(|e| IngestError(format!("{name}: not JSON ({e})")))?;
        let ex = Ex::parse(&body).map_err(|e| IngestError(format!("{name}: {e}")))?;
        let reads = input.companion("reads").ok_or_else(|| {
            IngestError(format!(
                "{name}: an extraction needs the file it reads (a companion with role \"reads\")"
            ))
        })?;
        if reads.hash != ex.reads_sha {
            return Err(IngestError(format!(
                "{name}: the extraction reads {} but the file given is {} ({}); give the file it was made from",
                ex.reads_sha.hex(),
                reads.hash.hex(),
                reads.name
            )));
        }
        let mut b = Builder::new(db, input, reads, &ex);
        b.envelope();
        match ex.kind.as_str() {
            "bank-statement" => b.statement()?,
            "check" => b.check()?,
            "receipt" => b.receipt()?,
            other => {
                return Err(IngestError(format!(
                    "{name}: kind {other:?} is not one this projector reads (bank-statement, check, receipt)"
                )));
            }
        }
        b.low_confidence();
        let tx = b.snap.diff(db, &owns);
        Ok(Projection {
            install: schema(),
            tx,
            pressure: b.pressure,
        })
    }
}

/// What a re-read of an extraction may retract: the observations, readings and extras it wrote, and the
/// document facts it states. Never `reading/review` (a person's), never shared accounts, files or sources.
fn owns(a: &Kw) -> bool {
    let ns = a.ns();
    (ns == "reading" && a.as_str() != "reading/review")
        || ns == "obs"
        || ns.starts_with("extraction")
        || matches!(
            a.as_str(),
            "doc/kind" | "doc/period-start" | "doc/period-end" | "doc/external-id"
        )
}

fn schema() -> Vec<Json> {
    let d = |ident: &str, ty: &str, doc: &str| serde_json::json!({"ident": ident, "type": ty, "doc": doc});
    let u = |ident: &str, ty: &str, doc: &str| serde_json::json!({"ident": ident, "type": ty, "unique": "identity", "doc": doc});
    vec![
        u(
            "reading/id",
            "string",
            "One field as an extractor read it: <entity id>#<attribute>#<field>",
        ),
        d(
            "reading/of",
            "ref",
            "The entity the field supports (an observation, a document, an account)",
        ),
        d(
            "reading/attr",
            "keyword",
            "The attribute the field supports, e.g. :obs/amount",
        ),
        d(
            "reading/field",
            "string",
            "The extractor's own name for the field, e.g. tip, checks-paid.amount",
        ),
        d(
            "reading/text",
            "string",
            "The text as read, e.g. \"$3,215.47\"",
        ),
        d("reading/region", "ref", "Where the text is: a page and box"),
        d(
            "reading/confidence",
            "decimal",
            "0 to 1, the extractor's confidence in this field",
        ),
        d(
            "reading/handwritten",
            "bool",
            "The field was written by hand",
        ),
        d(
            "reading/derivation",
            "string",
            "How the value was got when it is not the text parsed (a year from the statement period)",
        ),
        d(
            "reading/review",
            "keyword",
            "A person's verdict: reading.review/confirmed, reading.review/corrected, reading.review/rejected",
        ),
        d(
            "extraction/extractor",
            "string",
            "Who read the file: agent:<name>, ocr:<tool>",
        ),
        d("extraction/extractor-version", "string", ""),
        d(
            "extraction/read-at",
            "instant",
            "When the extractor read the file",
        ),
        d(
            "extraction.region/width",
            "decimal",
            "Width of a page region, in its unit",
        ),
        d("extraction.region/height", "decimal", ""),
        d(
            "extraction.region/unit",
            "string",
            "pt (PDF points, 1/72 in) or px; boxes on this page are in this unit, origin top-left",
        ),
        d(
            "extraction.region/rotation",
            "decimal",
            "Degrees the content is turned on the page (a photo); boxes are axis-aligned bounds",
        ),
        d("extraction.statement/deposits-count", "int", ""),
        d(
            "extraction.statement/deposits-total",
            "decimal",
            "As printed in the summary",
        ),
        d("extraction.statement/withdrawals-count", "int", ""),
        d(
            "extraction.statement/withdrawals-total",
            "decimal",
            "As printed in the summary, a magnitude",
        ),
        d(
            "extraction.line/running-balance",
            "decimal",
            "The balance a statement prints after a line",
        ),
        d(
            "extraction.balance/side",
            "keyword",
            "extraction.balance.side/opening (start of the date) or extraction.balance.side/closing (end of it)",
        ),
        d(
            "extraction.check/amount-words",
            "string",
            "The legal amount, in words",
        ),
        d(
            "extraction.check/signer",
            "string",
            "The name the signature reads as",
        ),
        d("extraction.check/micr", "string", "The MICR line as read"),
        d("extraction.check/micr-routing", "string", ""),
        d("extraction.check/micr-account", "string", ""),
        d("extraction.check/micr-serial", "string", ""),
        d(
            "extraction.receipt/subtotal",
            "decimal",
            "As printed, the merchant's side (positive)",
        ),
        d("extraction.receipt/tax", "decimal", ""),
        d(
            "extraction.receipt/printed-total",
            "decimal",
            "The total the merchant printed, before the tip",
        ),
        d("extraction.receipt/tip", "decimal", "Usually handwritten"),
        d(
            "extraction.receipt/time",
            "string",
            "Local time of day as printed (HH:MM); the receipt gives no zone",
        ),
        d("extraction.receipt/card-kind", "string", "DEBIT, VISA, …"),
        d(
            "extraction.receipt/card-mask",
            "string",
            "The card digits the receipt shows",
        ),
    ]
}

// ------------------------------------------------------------------------------------------------ format

#[derive(Debug, Clone)]
struct Field {
    name: String,
    /// JSON pointer to the field in the extraction
    at: String,
    text: String,
    value: String,
    page: i64,
    bbox: Option<[Decimal; 4]>,
    confidence: Option<Decimal>,
    handwritten: bool,
}

#[derive(Debug, Clone)]
struct Group {
    kind: String,
    at: String,
    page: Option<i64>,
    bbox: Option<[Decimal; 4]>,
    fields: BTreeMap<String, Field>,
}

impl Group {
    fn get(&self, name: &str) -> Option<&Field> {
        self.fields.get(name)
    }
}

#[derive(Debug, Clone)]
struct Page {
    page: i64,
    width: Decimal,
    height: Decimal,
    unit: String,
    rotation: Option<Decimal>,
}

#[derive(Debug, Clone)]
struct Ex {
    extractor: String,
    version: Option<String>,
    read_at: Option<Instant>,
    reads_sha: Hash,
    kind: String,
    pages: Vec<Page>,
    fields: Group,
    balances: Vec<Group>,
    items: Vec<Group>,
}

fn num(j: &Json) -> Option<Decimal> {
    match j {
        Json::Number(n) => parse_decimal(&n.to_string()).ok(),
        Json::String(t) => parse_decimal(t).ok(),
        _ => None,
    }
}

fn bbox(j: Option<&Json>) -> Option<[Decimal; 4]> {
    let a = j?.as_array()?;
    if a.len() != 4 {
        return None;
    }
    Some([num(&a[0])?, num(&a[1])?, num(&a[2])?, num(&a[3])?])
}

fn text(o: &Map<String, Json>, k: &str) -> Option<String> {
    o.get(k).and_then(Json::as_str).map(str::to_string)
}

impl Ex {
    fn parse(j: &Json) -> Result<Ex, String> {
        let o = j.as_object().ok_or("an extraction is a JSON object")?;
        if let Some(f) = o.get("format").and_then(Json::as_str)
            && f != "extraction@1"
        {
            return Err(format!(
                "format {f:?} is not extraction@1; this projector reads extraction@1"
            ));
        }
        let ext = o
            .get("extractor")
            .and_then(Json::as_object)
            .ok_or("needs \"extractor\" (who read the file)")?;
        let reads = o
            .get("reads")
            .and_then(Json::as_object)
            .ok_or("needs \"reads\" (the file it read, with its sha256)")?;
        let sha = text(reads, "sha256").ok_or("\"reads\" needs \"sha256\"")?;
        let reads_sha = Hash::parse(&sha).ok_or("\"reads.sha256\" is not a SHA-256 in hex")?;
        let pages = o
            .get("pages")
            .and_then(Json::as_array)
            .into_iter()
            .flatten()
            .filter_map(|p| {
                let p = p.as_object()?;
                Some(Page {
                    page: p.get("page")?.as_i64()?,
                    width: num(p.get("width")?)?,
                    height: num(p.get("height")?)?,
                    unit: text(p, "unit").unwrap_or_default(),
                    rotation: p.get("rotation_deg").and_then(num),
                })
            })
            .collect();
        let group = |g: &Json, at: String| -> Result<Group, String> {
            let g = g
                .as_object()
                .ok_or_else(|| format!("{at}: not an object"))?;
            let fields = g
                .get("fields")
                .and_then(Json::as_object)
                .ok_or_else(|| format!("{at}: needs \"fields\""))?;
            Ok(Group {
                kind: text(g, "kind")
                    .or_else(|| text(g, "name"))
                    .unwrap_or_default(),
                page: g.get("page").and_then(Json::as_i64),
                bbox: bbox(g.get("box")),
                fields: Self::fields(fields, &format!("{at}/fields"))?,
                at,
            })
        };
        let fields = o
            .get("fields")
            .and_then(Json::as_object)
            .map(|f| Self::fields(f, "/fields"))
            .transpose()?
            .unwrap_or_default();
        let list = |k: &str| -> Result<Vec<Group>, String> {
            o.get(k)
                .and_then(Json::as_array)
                .into_iter()
                .flatten()
                .enumerate()
                .map(|(i, g)| group(g, format!("/{k}/{i}")))
                .collect()
        };
        Ok(Ex {
            extractor: text(ext, "id").ok_or("\"extractor\" needs \"id\"")?,
            version: text(ext, "version"),
            read_at: text(ext, "read_at").and_then(|t| Instant::parse(&t).ok()),
            reads_sha,
            kind: text(o, "kind").ok_or("needs \"kind\"")?,
            pages,
            fields: Group {
                kind: String::new(),
                at: String::new(),
                page: None,
                bbox: None,
                fields,
            },
            balances: list("balances")?,
            items: list("items")?,
        })
    }

    fn fields(o: &Map<String, Json>, at: &str) -> Result<BTreeMap<String, Field>, String> {
        let mut out = BTreeMap::new();
        for (name, f) in o {
            let here = format!("{at}/{name}");
            let f = f
                .as_object()
                .ok_or_else(|| format!("{here}: not an object"))?;
            out.insert(
                name.clone(),
                Field {
                    name: name.clone(),
                    text: text(f, "text").unwrap_or_default(),
                    value: match f.get("value") {
                        Some(Json::String(s)) => s.clone(),
                        Some(Json::Number(n)) => n.to_string(),
                        _ => return Err(format!("{here}: needs a \"value\"")),
                    },
                    page: f
                        .get("page")
                        .and_then(Json::as_i64)
                        .ok_or_else(|| format!("{here}: needs a \"page\""))?,
                    bbox: bbox(f.get("box")),
                    confidence: f.get("confidence").and_then(num),
                    handwritten: f
                        .get("handwritten")
                        .and_then(Json::as_bool)
                        .unwrap_or(false),
                    at: here,
                },
            );
        }
        Ok(out)
    }
}

// ------------------------------------------------------------------------------------------------ build

fn k(s: &'static str) -> Kw {
    Kw::lit(s)
}

fn r(key: &str) -> ValIn {
    ValIn::Ref(crate::temp(key))
}

fn money(d: Decimal) -> String {
    let neg = d.is_sign_negative() && !d.is_zero();
    let t = format!("{:.2}", d.abs());
    let (int, frac) = t.split_once('.').unwrap_or((&t, "00"));
    let mut g = String::new();
    for (i, c) in int.chars().enumerate() {
        if i > 0 && (int.len() - i) % 3 == 0 {
            g.push(',');
        }
        g.push(c);
    }
    format!("{}{g}.{frac}", if neg { "-" } else { "" })
}

struct Builder<'a, 'i> {
    db: &'a Db,
    input: &'a Input<'i>,
    reads: &'a FileIn<'i>,
    ex: &'a Ex,
    snap: Snapshot,
    pressure: Vec<Pressure>,
    /// lowest confidence among the `obs/*` readings of each observation key
    conf: BTreeMap<String, Decimal>,
    /// fields that went into a reading, for the low-confidence pass
    read: Vec<Field>,
}

/// A statement line as projected, for the checks-paid table to find.
struct Line {
    key: String,
    id: String,
    amount: Decimal,
    date: Option<Date>,
    text: String,
}

/// Where a reading points: the snapshot key of the entity, its id (for the reading's id), and whether it is
/// an observation (which also collects the region in `obs/region`).
struct Target<'t> {
    key: &'t str,
    id: &'t str,
    obs: bool,
}

impl<'a, 'i> Builder<'a, 'i> {
    fn new(db: &'a Db, input: &'a Input<'i>, reads: &'a FileIn<'i>, ex: &'a Ex) -> Self {
        Builder {
            db,
            input,
            reads,
            ex,
            snap: Snapshot::new(),
            pressure: Vec::new(),
            conf: BTreeMap::new(),
            read: Vec::new(),
        }
    }

    fn press(&mut self, kind: &str, at: &str, detail: impl Into<String>) {
        self.pressure.push(Pressure::new(kind, at, detail));
    }

    fn add(&mut self, key: &str, a: &'static str, v: impl Into<ValIn>) {
        self.snap.add(key, &k(a), v);
    }

    fn src(&self) -> &str {
        &self.input.source.id
    }

    fn file_key(&mut self, h: &Hash) -> String {
        let key = format!("file:{}", h.hex());
        self.snap.refer(&key, &k("file/sha256"), Value::Hash(*h));
        key
    }

    /// The extraction file (who read what, when) and the page regions of the file it reads (their size).
    fn envelope(&mut self) {
        let src = format!("source:{}", self.src());
        let sid = self.src().to_string();
        self.snap.refer(&src, &k("source/id"), Value::str(&sid));
        let main = self.file_key(&self.input.file.hash);
        self.add(
            &main,
            "extraction/extractor",
            Value::str(&self.ex.extractor),
        );
        if let Some(v) = &self.ex.version {
            self.add(&main, "extraction/extractor-version", Value::str(v));
        }
        if let Some(t) = &self.ex.read_at {
            self.add(&main, "extraction/read-at", Value::Instant(t.clone()));
        }
        self.file_key(&self.reads.hash);
        for p in &self.ex.pages {
            let key = self.region(&format!("page:{}", p.page));
            self.add(&key, "extraction.region/width", Value::Dec(p.width));
            self.add(&key, "extraction.region/height", Value::Dec(p.height));
            self.add(&key, "extraction.region/unit", Value::str(&p.unit));
            if let Some(rot) = p.rotation {
                self.add(&key, "extraction.region/rotation", Value::Dec(rot));
            }
        }
    }

    /// A region of the file the extraction reads, by locator.
    fn region(&mut self, locator: &str) -> String {
        let hex = self.reads.hash.hex();
        let id = format!("{hex}#{locator}");
        let key = format!("region:{id}");
        self.snap.ent(&key, &k("region/id"), Value::str(&id));
        let f = self.file_key(&self.reads.hash);
        self.add(&key, "region/file", r(&f));
        self.add(&key, "region/locator", Value::str(locator));
        if let Some(p) = locator
            .strip_prefix("page:")
            .and_then(|t| t.split('/').next())
            .and_then(|t| t.parse::<i64>().ok())
        {
            self.add(&key, "region/page", Value::Int(p));
        }
        key
    }

    fn boxed(&mut self, page: i64, bbox: Option<[Decimal; 4]>) -> String {
        match bbox {
            Some(b) => {
                let c: Vec<String> = b.iter().map(|d| d.normalize().to_string()).collect();
                self.region(&format!("page:{page}/box:{}", c.join(",")))
            }
            None => self.region(&format!("page:{page}")),
        }
    }

    /// The document: one per file read, with both the extraction and the file it reads in `doc/file`.
    fn document(&mut self, id: &str, kind: &'static str) -> String {
        let key = format!("doc:{id}");
        self.snap.ent(&key, &k("doc/id"), Value::str(id));
        if let Ok(kw) = Kw::new(&format!("doc.kind/{kind}")) {
            self.add(&key, "doc/kind", Value::Kw(kw));
        }
        let src = format!("source:{}", self.src());
        self.add(&key, "doc/source", r(&src));
        let main = self.file_key(&self.input.file.hash);
        let reads = self.file_key(&self.reads.hash);
        self.add(&key, "doc/file", r(&main));
        self.add(&key, "doc/file", r(&reads));
        key
    }

    fn observation(&mut self, id: &str, kind: &str, doc: &str) -> String {
        let key = format!("obs:{id}");
        self.snap.ent(&key, &k("obs/id"), Value::str(id));
        if let Ok(kw) = Kw::new(&format!("obs.kind/{kind}")) {
            self.add(&key, "obs/kind", Value::Kw(kw));
        }
        self.add(&key, "obs/doc", r(doc));
        key
    }

    /// One field as read: a reading entity naming the attribute it supports, and the region it was read
    /// from. `derivation` says how the value was got when it is more than the text parsed.
    fn reading(
        &mut self,
        t: &Target<'_>,
        attr: &str,
        f: &Field,
        label: &str,
        derivation: Option<&str>,
    ) {
        let rid = format!("{}#{attr}#{label}", t.id);
        let key = format!("reading:{rid}");
        self.snap.ent(&key, &k("reading/id"), Value::str(&rid));
        self.add(&key, "reading/of", r(t.key));
        if let Ok(a) = Kw::new(attr) {
            self.add(&key, "reading/attr", Value::Kw(a));
        }
        self.add(&key, "reading/field", Value::str(label));
        self.add(&key, "reading/text", Value::str(&f.text));
        let region = self.boxed(f.page, f.bbox);
        self.add(&key, "reading/region", r(&region));
        if let Some(c) = f.confidence {
            self.add(&key, "reading/confidence", Value::Dec(c));
            if t.obs && attr.starts_with("obs/") {
                let e = self.conf.entry(t.key.to_string()).or_insert(c);
                if c < *e {
                    *e = c;
                }
            }
        }
        if f.handwritten {
            self.add(&key, "reading/handwritten", Value::Bool(true));
        }
        if let Some(d) = derivation {
            self.add(&key, "reading/derivation", Value::str(d));
        }
        if t.obs {
            self.add(t.key, "obs/region", r(&region));
        }
        self.read.push(f.clone());
    }

    /// Set an attribute from a field and record the reading behind it.
    fn put(&mut self, t: &Target<'_>, attr: &'static str, v: Value, f: &Field, label: &str) {
        self.add(t.key, attr, v);
        self.reading(t, attr, f, label, None);
    }

    fn finish_confidence(&mut self) {
        for (key, c) in std::mem::take(&mut self.conf) {
            self.add(&key, "obs/confidence", Value::Dec(c));
        }
    }

    fn dec(&mut self, f: &Field) -> Option<Decimal> {
        match parse_decimal(&f.value) {
            Ok(d) => Some(d),
            Err(_) => {
                self.press("unparsed", &f.at, format!("{:?} is not a decimal", f.value));
                None
            }
        }
    }

    fn date(&mut self, f: &Field) -> Option<Date> {
        match Date::parse(&f.value) {
            Ok(d) => Some(d),
            Err(_) => {
                self.press("unparsed", &f.at, format!("{:?} is not a date", f.value));
                None
            }
        }
    }

    fn field(&mut self, g: &Group, name: &str) -> Option<Field> {
        let f = g.get(name).cloned();
        if f.is_none() {
            let at = if g.at.is_empty() { "/fields" } else { &g.at };
            self.press(
                "missing",
                at,
                format!("no {name} field; the observation goes without it"),
            );
        }
        f
    }

    fn usd(&mut self) -> String {
        let key = "commodity:USD".to_string();
        self.snap
            .refer(&key, &k("commodity/code"), Value::str("USD"));
        key
    }

    /// The bank account behind a mask. One account already in the log with that mask and an institution
    /// (a bank-side account, not a book's) wins. Otherwise the id is made from what the document says
    /// (institution's first word, account type, mask), and the guess is pressure.
    fn account(
        &mut self,
        mask: &str,
        institution: Option<&str>,
        kind_hint: &str,
        at: &str,
    ) -> Option<(String, String)> {
        let found: Vec<String> = self
            .db
            .attr(&k("account/mask"))
            .map(|d| {
                self.db
                    .with_value(d.id, &Value::str(mask))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default()
            .into_iter()
            .filter(|e| {
                self.db.get(*e, &k("account/book")).is_none()
                    && self.db.get(*e, &k("account/institution")).is_some()
            })
            .filter_map(|e| {
                self.db
                    .get(e, &k("account/id"))
                    .and_then(Value::as_str)
                    .map(str::to_string)
            })
            .collect();
        let id = match (found.as_slice(), institution) {
            ([one], _) => one.clone(),
            (_, Some(inst)) => {
                let slug: String = inst
                    .split_whitespace()
                    .next()
                    .unwrap_or("bank")
                    .to_ascii_lowercase();
                let id = format!("{slug}:{kind_hint}:{mask}");
                self.press(
                    "derived-identity",
                    at,
                    format!(
                        "no account with mask {mask} in the log yet ({} found); named it {id} from the document; \
                         a source-to-account binding would say this outright",
                        found.len()
                    ),
                );
                id
            }
            (_, None) => {
                self.press(
                    "no-identity",
                    at,
                    format!(
                        "card ending {mask}: {} bank accounts with that mask in the log; the observation has no obs/account",
                        found.len()
                    ),
                );
                return None;
            }
        };
        let key = format!("account:{id}");
        self.snap.ent(&key, &k("account/id"), Value::str(&id));
        self.add(&key, "account/mask", Value::str(mask));
        let usd = self.usd();
        self.add(&key, "account/commodity", r(&usd));
        Some((key, id))
    }

    fn low_confidence(&mut self) {
        let mut seen = std::collections::BTreeSet::new();
        for f in std::mem::take(&mut self.read) {
            if !seen.insert(f.at.clone()) {
                continue;
            }
            if let Some(c) = f.confidence
                && c < LOW
            {
                self.press(
                    "low-confidence",
                    &f.at,
                    format!(
                        "{} read as {:?} from {:?}{} at confidence {c}; a person should confirm it",
                        f.name,
                        f.value,
                        f.text,
                        if f.handwritten { " (handwritten)" } else { "" }
                    ),
                );
            }
        }
    }

    // -------------------------------------------------------------------------------------- statement

    fn statement(&mut self) -> Result<(), IngestError> {
        let ex = self.ex;
        let df = &ex.fields;
        let (Some(start), Some(end), Some(mask)) = (
            df.get("period_start").cloned(),
            df.get("period_end").cloned(),
            df.get("account_mask").cloned(),
        ) else {
            return Err(IngestError(format!(
                "{}: a bank statement needs period_start, period_end and account_mask",
                self.input.file.name
            )));
        };
        let (Some(ps), Some(pe)) = (self.date(&start), self.date(&end)) else {
            return Err(IngestError(format!(
                "{}: the statement period is not readable",
                self.input.file.name
            )));
        };
        let inst = df.get("institution").map(|f| f.value.clone());
        let Some((acct, acct_id)) = self.account(
            &mask.value,
            inst.as_deref(),
            "checking",
            "/fields/account_mask",
        ) else {
            return Ok(());
        };
        let doc_id = format!(
            "{}/statement/{acct_id}/{}..{}",
            self.src(),
            ps.as_str(),
            pe.as_str()
        );
        let doc = self.document(&doc_id, "bank-statement");
        let dt = Target {
            key: &doc,
            id: &doc_id,
            obs: false,
        };
        self.put(
            &dt,
            "doc/period-start",
            Value::Date(ps.clone()),
            &start,
            "period_start",
        );
        self.put(
            &dt,
            "doc/period-end",
            Value::Date(pe.clone()),
            &end,
            "period_end",
        );
        let at = Target {
            key: &acct,
            id: &acct_id,
            obs: false,
        };
        self.reading(&at, "account/mask", &mask, "account_mask", None);
        if let Some(f) = df.get("institution") {
            self.put(
                &at,
                "account/institution",
                Value::str(&f.value),
                f,
                "institution",
            );
        }
        if let Some(f) = df.get("account_name") {
            self.put(&at, "account/name", Value::str(&f.value), f, "account_name");
        }
        let mut summary: BTreeMap<&str, Decimal> = BTreeMap::new();
        for (name, attr, int) in [
            (
                "deposits_count",
                "extraction.statement/deposits-count",
                true,
            ),
            (
                "deposits_total",
                "extraction.statement/deposits-total",
                false,
            ),
            (
                "withdrawals_count",
                "extraction.statement/withdrawals-count",
                true,
            ),
            (
                "withdrawals_total",
                "extraction.statement/withdrawals-total",
                false,
            ),
        ] {
            let Some(f) = df.get(name).cloned() else {
                continue;
            };
            let Some(d) = self.dec(&f) else { continue };
            summary.insert(name, d);
            let v = if int {
                match i64::try_from(d) {
                    Ok(i) if d.fract().is_zero() => Value::Int(i),
                    _ => {
                        self.press("unparsed", &f.at, format!("{:?} is not a count", f.value));
                        continue;
                    }
                }
            } else {
                Value::Dec(d)
            };
            self.put(&dt, attr, v, &f, name);
        }
        let usd = self.usd();
        self.press(
            "assumed",
            "/fields",
            "amounts carry \"$\" and no currency code; read as USD, the account's commodity",
        );

        // balances
        let mut opening = None;
        let mut closing = None;
        for g in &ex.balances {
            let side = g.kind.as_str();
            if side != "opening" && side != "closing" {
                self.press(
                    "unknown",
                    &g.at,
                    format!("balance {side:?} is neither opening nor closing"),
                );
                continue;
            }
            let id = format!("{doc_id}/balance/{side}");
            let key = self.observation(&id, "balance", &doc);
            let t = Target {
                key: &key,
                id: &id,
                obs: true,
            };
            self.add(&key, "obs/account", r(&acct));
            self.add(&key, "obs/commodity", r(&usd));
            if let Ok(kw) = Kw::new(&format!("extraction.balance.side/{side}")) {
                self.add(&key, "extraction.balance/side", Value::Kw(kw));
            }
            if let Some(f) = self.field(g, "date")
                && let Some(d) = self.date(&f)
            {
                self.put(&t, "obs/date", Value::Date(d), &f, &format!("{side}.date"));
            }
            if let Some(f) = self.field(g, "amount")
                && let Some(a) = self.dec(&f)
            {
                self.put(
                    &t,
                    "obs/amount",
                    Value::Dec(a),
                    &f,
                    &format!("{side}.amount"),
                );
                if side == "opening" {
                    opening = Some(a);
                } else {
                    closing = Some(a);
                }
            }
        }

        // lines
        let mut sum = Decimal::ZERO;
        let mut running = opening;
        let (mut dep_n, mut dep_t, mut wd_n, mut wd_t) = (0i64, Decimal::ZERO, 0i64, Decimal::ZERO);
        let mut derived_years = 0;
        let mut lines: Vec<Line> = Vec::new();
        let mut n = 0;
        for g in ex.items.iter().filter(|g| g.kind == "line") {
            n += 1;
            let id = format!("{doc_id}/line/{n}");
            let key = self.observation(&id, "money-movement", &doc);
            let t = Target {
                key: &key,
                id: &id,
                obs: true,
            };
            self.add(&key, "obs/index", Value::Int(n));
            self.add(&key, "obs/account", r(&acct));
            self.add(&key, "obs/commodity", r(&usd));
            if let Some(page) = g.page.or_else(|| g.fields.values().next().map(|f| f.page)) {
                let row = self.boxed(page, g.bbox);
                self.add(&key, "obs/region", r(&row));
            }
            let date = match self.field(g, "date") {
                Some(f) => {
                    let (d, how) = self.year_from_period(&f, &ps, &pe);
                    if how.is_some() {
                        derived_years += 1;
                    }
                    if let Some(d) = &d {
                        self.add(&key, "obs/date", Value::Date(d.clone()));
                    }
                    self.reading(&t, "obs/date", &f, "date", how.as_deref());
                    d
                }
                None => None,
            };
            let desc = self.field(g, "description");
            if let Some(f) = &desc {
                self.put(
                    &t,
                    "obs/description",
                    Value::str(&f.value),
                    f,
                    "description",
                );
            }
            let Some(af) = self.field(g, "amount") else {
                continue;
            };
            let Some(amount) = self.dec(&af) else {
                continue;
            };
            self.put(&t, "obs/amount", Value::Dec(amount), &af, "amount");
            sum += amount;
            if amount.is_sign_negative() {
                wd_n += 1;
                wd_t -= amount;
            } else {
                dep_n += 1;
                dep_t += amount;
            }
            if let Some(bf) = self.field(g, "balance")
                && let Some(bal) = self.dec(&bf)
            {
                self.put(
                    &t,
                    "extraction.line/running-balance",
                    Value::Dec(bal),
                    &bf,
                    "balance",
                );
                if let Some(prev) = running
                    && prev + amount != bal
                {
                    self.press(
                        "check-failed",
                        &bf.at,
                        format!(
                            "running balance: {} + {} = {}, but the line prints {}",
                            money(prev),
                            money(amount),
                            money(prev + amount),
                            money(bal)
                        ),
                    );
                }
                running = Some(bal);
            }
            let text = desc.map(|f| f.value).unwrap_or_default();
            if let Some(d) = &date {
                let fp = fingerprint(&mask.value, d.as_str(), amount, &text);
                self.add(&key, "obs/fingerprint", Value::str(&fp));
            }
            lines.push(Line {
                key,
                id,
                amount,
                date,
                text,
            });
        }
        if derived_years > 0 {
            self.press(
                "missing-year",
                "/items",
                format!(
                    "{derived_years} line dates print month and day only; the year comes from the period {}..{} \
                     (reading/derivation says so on each)",
                    ps.as_str(),
                    pe.as_str()
                ),
            );
        }

        // checks paid: the same items again, in another table; they add to the line they repeat
        for g in ex.items.iter().filter(|g| g.kind == "check-paid") {
            self.check_paid(g, &lines, &ps, &pe);
        }

        // self-checks
        match (opening, closing) {
            (Some(o), Some(c)) if o + sum != c => self.press(
                "check-failed",
                "/balances",
                format!(
                    "opening {} + lines {} = {}, but the closing balance reads {}",
                    money(o),
                    money(sum),
                    money(o + sum),
                    money(c)
                ),
            ),
            (Some(_), Some(_)) => {}
            _ => self.press(
                "check-skipped",
                "/balances",
                "no opening or closing balance; the lines are not checked",
            ),
        }
        for (name, got) in [
            ("deposits_count", Decimal::from(dep_n)),
            ("deposits_total", dep_t),
            ("withdrawals_count", Decimal::from(wd_n)),
            ("withdrawals_total", wd_t),
        ] {
            if let Some(want) = summary.get(name)
                && *want != got
            {
                self.press(
                    "check-failed",
                    &format!("/fields/{name}"),
                    format!("the summary prints {name} {want}, the lines add up to {got}"),
                );
            }
        }
        self.finish_confidence();
        Ok(())
    }

    /// A statement line's date: a full date as read, or month and day with the year taken from the period.
    fn year_from_period(
        &mut self,
        f: &Field,
        ps: &Date,
        pe: &Date,
    ) -> (Option<Date>, Option<String>) {
        if let Ok(d) = Date::parse(&f.value) {
            return (Some(d), None);
        }
        let Some(md) = f.value.strip_prefix("--") else {
            self.press(
                "unparsed",
                &f.at,
                format!("{:?} is neither a date nor --MM-DD", f.value),
            );
            return (None, None);
        };
        let years = [&ps.as_str()[..4], &pe.as_str()[..4]];
        for y in years {
            if let Ok(d) = Date::parse(&format!("{y}-{md}"))
                && d.as_str() >= ps.as_str()
                && d.as_str() <= pe.as_str()
            {
                let how = format!(
                    "year {y} from the statement period {}..{}; the line prints {:?}",
                    ps.as_str(),
                    pe.as_str(),
                    f.text
                );
                return (Some(d), Some(how));
            }
        }
        self.press(
            "check-failed",
            &f.at,
            format!(
                "{:?} falls outside the statement period {}..{} in every year it could be",
                f.text,
                ps.as_str(),
                pe.as_str()
            ),
        );
        (None, None)
    }

    fn check_paid(&mut self, g: &Group, lines: &[Line], ps: &Date, pe: &Date) {
        let (Some(nf), Some(af)) = (self.field(g, "number"), self.field(g, "amount")) else {
            return;
        };
        let Some(amount) = self.dec(&af) else { return };
        let date = match g.get("date").cloned() {
            Some(f) => self.year_from_period(&f, ps, pe).0,
            None => None,
        };
        let want = format!("CHECK {}", nf.value);
        // a check paid is money out; the table prints it unsigned
        let hit = lines.iter().find(|l| {
            l.amount == -amount
                && l.text.eq_ignore_ascii_case(&want)
                && (date.is_none() || l.date.as_ref() == date.as_ref())
        });
        let Some(Line { key, id, .. }) = hit else {
            self.press(
                "check-failed",
                &g.at,
                format!(
                    "checks paid lists check {} for {}, but no line reads {want:?} for {}",
                    nf.value,
                    money(amount),
                    money(-amount)
                ),
            );
            return;
        };
        let (key, id) = (key.clone(), id.clone());
        let t = Target {
            key: &key,
            id: &id,
            obs: true,
        };
        self.put(
            &t,
            "obs/reference",
            Value::str(&nf.value),
            &nf,
            "checks-paid.number",
        );
        self.reading(
            &t,
            "obs/amount",
            &af,
            "checks-paid.amount",
            Some("printed unsigned in the checks paid table"),
        );
        if let Some(f) = g.get("date").cloned() {
            self.reading(
                &t,
                "obs/date",
                &f,
                "checks-paid.date",
                Some("year from the statement period"),
            );
        }
        if let Some(page) = g.page {
            let row = self.boxed(page, g.bbox);
            self.add(&key, "obs/region", r(&row));
        }
        self.press(
            "embedded-copy",
            &g.at,
            format!(
                "checks paid repeats line {id} (check {}); kept as more readings and regions on that one observation",
                nf.value
            ),
        );
    }

    // ------------------------------------------------------------------------------------------ check

    fn check(&mut self) -> Result<(), IngestError> {
        let g = self.ex.fields.clone();
        let (Some(nf), Some(af)) = (g.get("number").cloned(), g.get("amount").cloned()) else {
            return Err(IngestError(format!(
                "{}: a check needs number and amount",
                self.input.file.name
            )));
        };
        // the account: MICR account number's last four, at the bank printed on the check
        let micr = g.get("micr").cloned();
        let parts: Vec<String> = micr
            .as_ref()
            .map(|f| f.value.split_whitespace().map(str::to_string).collect())
            .unwrap_or_default();
        let mask = parts
            .get(1)
            .filter(|a| a.len() >= 4)
            .map(|a| a[a.len() - 4..].to_string());
        let Some(mask) = mask else {
            return Err(IngestError(format!(
                "{}: the MICR line does not give an account number; the check cannot be tied to an account",
                self.input.file.name
            )));
        };
        let bank = g.get("bank").map(|f| f.value.clone());
        self.press(
            "assumed",
            "/fields/micr",
            "a check is drawn on a checking account; the MICR line does not say which kind",
        );
        let Some((acct, acct_id)) =
            self.account(&mask, bank.as_deref(), "checking", "/fields/micr")
        else {
            return Ok(());
        };
        let doc_id = format!("{}/check/{acct_id}/{}", self.src(), nf.value);
        let doc = self.document(&doc_id, "check");
        self.add(&doc, "doc/external-id", Value::str(&nf.value));
        let usd = self.usd();
        let id = format!("{doc_id}/check");
        let key = self.observation(&id, "check", &doc);
        let t = Target {
            key: &key,
            id: &id,
            obs: true,
        };
        self.add(&key, "obs/account", r(&acct));
        self.add(&key, "obs/commodity", r(&usd));
        if let Some(f) = &micr {
            let at = Target {
                key: &acct,
                id: &acct_id,
                obs: false,
            };
            self.reading(
                &at,
                "account/mask",
                f,
                "micr",
                Some("last four digits of the MICR account number"),
            );
        }
        if let Some(f) = g.get("bank") {
            let at = Target {
                key: &acct,
                id: &acct_id,
                obs: false,
            };
            self.put(&at, "account/institution", Value::str(&f.value), f, "bank");
        }
        self.put(&t, "obs/reference", Value::str(&nf.value), &nf, "number");
        let figure = self.dec(&af);
        if let Some(a) = figure {
            // written by the account holder: money out
            self.put(&t, "obs/amount", Value::Dec(-a), &af, "amount");
        }
        let date = match self.field(&g, "date") {
            Some(f) => {
                let d = self.date(&f);
                if let Some(d) = &d {
                    self.put(&t, "obs/date", Value::Date(d.clone()), &f, "date");
                }
                d
            }
            None => None,
        };
        if let Some(f) = self.field(&g, "payee") {
            self.put(&t, "obs/counterparty", Value::str(&f.value), &f, "payee");
        }
        if let Some(f) = g.get("memo") {
            self.put(&t, "obs/description", Value::str(&f.value), f, "memo");
        }
        if let Some(f) = g.get("signer") {
            self.put(
                &t,
                "extraction.check/signer",
                Value::str(&f.value),
                f,
                "signer",
            );
        }
        if let Some(f) = &micr {
            self.put(&t, "extraction.check/micr", Value::str(&f.text), f, "micr");
            for (i, attr) in [
                "extraction.check/micr-routing",
                "extraction.check/micr-account",
                "extraction.check/micr-serial",
            ]
            .into_iter()
            .enumerate()
            {
                if let Some(p) = parts.get(i) {
                    self.add(&key, attr, Value::str(p));
                }
            }
            if parts.get(2).is_some_and(|s| *s != nf.value) {
                self.press(
                    "check-failed",
                    &f.at,
                    format!(
                        "the MICR serial {:?} is not the check number {:?}",
                        parts[2], nf.value
                    ),
                );
            }
        }
        if let Some(f) = g.get("amount_words") {
            self.put(
                &t,
                "extraction.check/amount-words",
                Value::str(&f.value),
                f,
                "amount_words",
            );
            match (words_amount(&f.value), figure) {
                (Some(w), Some(a)) if w != a => self.press(
                    "check-failed",
                    &f.at,
                    format!(
                        "the amount in words ({:?} = {}) is not the figure {}; the words are the legal amount",
                        f.value,
                        money(w),
                        money(a)
                    ),
                ),
                (None, _) => self.press("unparsed", &f.at, format!("{:?} does not read as an amount", f.value)),
                _ => {}
            }
        }
        if let (Some(d), Some(a)) = (&date, figure) {
            let fp = fingerprint(&mask, d.as_str(), -a, &format!("CHECK {}", nf.value));
            self.add(&key, "obs/fingerprint", Value::str(&fp));
            self.press(
                "two-dates",
                "/fields/date",
                format!(
                    "the check is dated {} (written); the bank posts it when it clears, so its fingerprint will \
                     not equal the bank line's; match on account, obs/reference and amount",
                    d.as_str()
                ),
            );
        }
        self.finish_confidence();
        Ok(())
    }

    // ---------------------------------------------------------------------------------------- receipt

    fn receipt(&mut self) -> Result<(), IngestError> {
        let g = self.ex.fields.clone();
        let doc_id = format!("{}/receipt/{}", self.src(), self.reads.hash.hex());
        let doc = self.document(&doc_id, "receipt");
        let usd = self.usd();
        let id = format!("{doc_id}/receipt");
        let key = self.observation(&id, "receipt", &doc);
        let t = Target {
            key: &key,
            id: &id,
            obs: true,
        };
        self.add(&key, "obs/commodity", r(&usd));
        let mut mask = None;
        if let Some(f) = g.get("card") {
            let kind = f.text.split_whitespace().next().unwrap_or("").to_string();
            self.put(
                &t,
                "extraction.receipt/card-mask",
                Value::str(&f.value),
                f,
                "card",
            );
            if !kind.is_empty() {
                self.add(&key, "extraction.receipt/card-kind", Value::str(&kind));
            }
            if let Some((acct, _)) = self.account(&f.value, None, "card", "/fields/card") {
                self.add(&key, "obs/account", r(&acct));
                self.reading(
                    &t,
                    "obs/account",
                    f,
                    "card",
                    Some("the card's last four digits name a bank account already in the log"),
                );
            }
            mask = Some(f.value.clone());
        }
        let date = match self.field(&g, "date") {
            Some(f) => {
                let d = self.date(&f);
                if let Some(d) = &d {
                    self.put(&t, "obs/date", Value::Date(d.clone()), &f, "date");
                }
                d
            }
            None => None,
        };
        if let Some(f) = g.get("time") {
            self.put(
                &t,
                "extraction.receipt/time",
                Value::str(&f.value),
                f,
                "time",
            );
            self.press(
                "no-zone",
                &f.at,
                format!("the receipt prints {:?} with no time zone; kept as local time text, not obs/instant", f.text),
            );
        }
        let merchant = g.get("merchant").cloned();
        if let Some(f) = &merchant {
            self.put(&t, "obs/counterparty", Value::str(&f.value), f, "merchant");
        }
        if let Some(f) = g.get("auth_code") {
            self.put(&t, "obs/reference", Value::str(&f.value), f, "auth_code");
        }
        let mut amounts: BTreeMap<&str, Decimal> = BTreeMap::new();
        for (name, attr) in [
            ("subtotal", "extraction.receipt/subtotal"),
            ("tax", "extraction.receipt/tax"),
            ("tip", "extraction.receipt/tip"),
            ("printed_total", "extraction.receipt/printed-total"),
        ] {
            if let Some(f) = g.get(name).cloned()
                && let Some(d) = self.dec(&f)
            {
                amounts.insert(name, d);
                self.put(&t, attr, Value::Dec(d), &f, name);
            }
        }
        // the paid total: the handwritten one when there is a tip line, else the printed one
        let paid = match g.get("total").cloned() {
            Some(f) => self.dec(&f).map(|d| (d, f)),
            None => g
                .get("printed_total")
                .cloned()
                .and_then(|f| self.dec(&f).map(|d| (d, f))),
        };
        if let Some((total, f)) = &paid {
            self.put(&t, "obs/amount", Value::Dec(-*total), f, &f.name.clone());
            amounts.insert("total", *total);
        }
        // lines
        let mut lines_sum = Decimal::ZERO;
        let mut n = 0;
        let ex = self.ex;
        for li in ex.items.iter().filter(|i| i.kind == "line") {
            n += 1;
            let lid = format!("{id}/line/{n}");
            let lk = self.observation(&lid, "line", &doc);
            let lt = Target {
                key: &lk,
                id: &lid,
                obs: true,
            };
            self.add(&lk, "obs/parent", r(&key));
            self.add(&lk, "obs/index", Value::Int(n));
            self.add(&lk, "obs/commodity", r(&usd));
            if let Some(page) = li.page {
                let row = self.boxed(page, li.bbox);
                self.add(&lk, "obs/region", r(&row));
            }
            if let Some(f) = self.field(li, "description") {
                self.put(
                    &lt,
                    "obs/description",
                    Value::str(&f.value),
                    &f,
                    "description",
                );
            }
            if let Some(f) = self.field(li, "amount")
                && let Some(a) = self.dec(&f)
            {
                lines_sum += a;
                self.put(&lt, "obs/amount", Value::Dec(-a), &f, "amount");
            }
        }
        // self-checks
        let get = |n: &str| amounts.get(n).copied();
        if let Some(sub) = get("subtotal")
            && n > 0
            && lines_sum != sub
        {
            self.press(
                "check-failed",
                "/fields/subtotal",
                format!(
                    "the lines add up to {}, the subtotal reads {}",
                    money(lines_sum),
                    money(sub)
                ),
            );
        }
        if let (Some(sub), Some(tax), Some(pt)) =
            (get("subtotal"), get("tax"), get("printed_total"))
            && sub + tax != pt
        {
            self.press(
                "check-failed",
                "/fields/printed_total",
                format!(
                    "subtotal {} + tax {} = {}, the printed total reads {}",
                    money(sub),
                    money(tax),
                    money(sub + tax),
                    money(pt)
                ),
            );
        }
        if let (Some(sub), Some(tax), Some(tip), Some(total)) =
            (get("subtotal"), get("tax"), get("tip"), get("total"))
            && sub + tax + tip != total
        {
            self.press(
                "check-failed",
                "/fields/total",
                format!(
                    "subtotal {} + tax {} + tip {} = {}, the handwritten total reads {}",
                    money(sub),
                    money(tax),
                    money(tip),
                    money(sub + tax + tip),
                    money(total)
                ),
            );
        }
        if let (Some(pt), Some(total)) = (get("printed_total"), get("total"))
            && pt != total
        {
            self.press(
                "two-totals",
                "/fields/total",
                format!(
                    "the receipt prints a total of {} and a handwritten total of {}; obs/amount is the paid total, \
                     extraction.receipt/printed-total keeps the other",
                    money(pt),
                    money(total)
                ),
            );
        }
        if let (Some(m), Some(d), Some((total, _))) = (&mask, &date, &paid) {
            let who = merchant.map(|f| f.value).unwrap_or_default();
            let fp = fingerprint(m, d.as_str(), -*total, &who);
            self.add(&key, "obs/fingerprint", Value::str(&fp));
        }
        self.finish_confidence();
        Ok(())
    }
}

/// An amount written in words, as on a check: "Three thousand two hundred fifteen and 47/100".
fn words_amount(text: &str) -> Option<Decimal> {
    let lower = text.to_ascii_lowercase().replace('-', " ");
    let lower = lower.trim_end_matches(" dollars").trim_end_matches(" only");
    let (words, cents) = match lower.rsplit_once(" and ") {
        Some((w, c)) if c.trim().ends_with("/100") => (w, c.trim().trim_end_matches("/100")),
        _ => (lower, "0"),
    };
    let cents: i64 = if cents.trim().eq_ignore_ascii_case("no") {
        0
    } else {
        cents.trim().parse().ok()?
    };
    let mut total: i64 = 0;
    let mut group: i64 = 0;
    for w in words.split_whitespace() {
        let small = match w {
            "zero" => 0,
            "one" => 1,
            "two" => 2,
            "three" => 3,
            "four" => 4,
            "five" => 5,
            "six" => 6,
            "seven" => 7,
            "eight" => 8,
            "nine" => 9,
            "ten" => 10,
            "eleven" => 11,
            "twelve" => 12,
            "thirteen" => 13,
            "fourteen" => 14,
            "fifteen" => 15,
            "sixteen" => 16,
            "seventeen" => 17,
            "eighteen" => 18,
            "nineteen" => 19,
            "twenty" => 20,
            "thirty" => 30,
            "forty" => 40,
            "fifty" => 50,
            "sixty" => 60,
            "seventy" => 70,
            "eighty" => 80,
            "ninety" => 90,
            "hundred" => {
                group *= 100;
                continue;
            }
            "thousand" | "million" => {
                let m = if w == "thousand" { 1_000 } else { 1_000_000 };
                total += group * m;
                group = 0;
                continue;
            }
            "and" => continue,
            _ => return None,
        };
        group += small;
    }
    Some(Decimal::from(total + group) + Decimal::new(cents, 2))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn words() {
        let d = |s: &str| parse_decimal(s).unwrap();
        assert_eq!(
            words_amount("Three thousand two hundred fifteen and 47/100"),
            Some(d("3215.47"))
        );
        assert_eq!(
            words_amount("One hundred and no/100 dollars"),
            Some(d("100"))
        );
        assert_eq!(words_amount("Twenty-one and 05/100"), Some(d("21.05")));
        assert_eq!(
            words_amount("Two million forty thousand and 00/100"),
            Some(d("2040000"))
        );
        assert_eq!(words_amount("Three thousand and some"), None);
    }

    #[test]
    fn money_groups() {
        let d = |s: &str| parse_decimal(s).unwrap();
        assert_eq!(money(d("-3215.47")), "-3,215.47");
        assert_eq!(money(d("24310.55")), "24,310.55");
        assert_eq!(money(d("4.7")), "4.70");
    }
}
