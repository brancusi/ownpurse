# Edges: documents read by an extractor

What loading a bank statement PDF, a check image and a receipt photo taught us about the fact model. The
projector is `extraction@1` (`rust/crates/ownpurse-ingest/src/sources/extraction.rs`); the samples are
`samples/bank/statement-2026-03.*`, `samples/bank/check-1042.*` and `samples/receipts/copper-pot-2026-03-12.*`;
the tests are `rust/crates/ownpurse-ingest/tests/extraction.rs`. Everything here is synthetic.

## The input

An extraction is JSON (`format: extraction@1`) that an extractor (here `agent:tally`) writes after reading one
file. It names the extractor and version, when it read, the SHA-256 of the file it read, the document kind
(`bank-statement`, `check`, `receipt`), the pages with their size, unit and origin, document-level `fields`,
`balances` and `items`. Every field is the same shape:

```json
{"text": "$3,215.47", "value": "3215.47", "page": 1, "box": [1225, 251, 1395, 280], "confidence": 0.97,
 "handwritten": true}
```

The file it reads goes in as a companion with role `reads`. The projector refuses an extraction whose
`reads.sha256` is not the companion's hash, and one with no companion.

## What the projector writes

| Thing | Facts |
|---|---|
| The extraction file | `extraction/extractor`, `extraction/extractor-version`, `extraction/read-at`; the envelope's `file/derived-from` points at the file it reads |
| Each page | A region `<sha>#page:<n>` with `extraction.region/width`, `/height`, `/unit` (`pt`, `px`), and `/rotation` for a photo |
| The document | `doc/*`, one per file read; `doc/file` names both the extraction and the file it reads; a statement has `doc/period-start`/`-end` and its printed summary as `extraction.statement/*` |
| Statement lines | `obs.kind/money-movement`: date, signed amount, USD, the shared account, description, fingerprint, `extraction.line/running-balance` |
| Balances | `obs.kind/balance` with `extraction.balance/side` opening or closing |
| The check | `obs.kind/check`: written date, amount (negative, money out), payee as `obs/counterparty`, number as `obs/reference`, memo as `obs/description`, signer and MICR parts as `extraction.check/*` |
| The receipt | `obs.kind/receipt`: date, paid total (negative), merchant, auth code as `obs/reference`; subtotal, tax, tip, printed total, time and card as `extraction.receipt/*`; each item an `obs.kind/line` with `obs/parent` |
| Every field | A `reading/*` entity: the entity and attribute it supports, the extractor's field name, the raw text, the region, the confidence, handwritten, and how the value was derived when it is more than the text parsed |

The account is the shared identity `ridgeway:checking:4821` (mask, name, institution, USD). Self-checks that fail
are `check-failed` pressure; nothing is fixed.

## Strains

### Per-field provenance

**What.** The core vocabulary gives an observation `obs/region` (many) and `obs/confidence` (one). A receipt's
date sits in one box, read at 0.97; its handwritten total in another, read at 0.81; the tip in a third, at 0.62.
With only those two attributes you know the receipt came from five boxes and is "0.62 confident", but not which
box holds the total or which figure is the shaky one.

**What the model did.** Each field becomes a reading entity, installed in the `reading` namespace:
`reading/id` (`<entity id>#<attribute>#<field>`, identity), `reading/of` (the observation, document or account it
supports), `reading/attr` (a keyword naming the attribute, e.g. `:obs/amount`), `reading/field`,
`reading/text`, `reading/region`, `reading/confidence`, `reading/handwritten`, `reading/derivation`. The value
itself stays on the observation in the shared vocabulary, so rules and reports never touch readings. `obs/region`
still collects every field's region (and the row's), so "show this item in place" works without readings, and
`obs/confidence` is the lowest confidence among the readings of the observation's `obs/*` attributes. Two
readings may support one attribute: the statement's `CHECK 1042` line has its amount read twice, once in the
transactions table and once in "Checks paid" on page 2.

**What was lost or awkward.** Volume: on the statement, 160 `obs/*` facts carry 906 facts of provenance
(491 reading, 342 region, 73 `obs/region`). The receipt is 56 to 283. That is fine for SQLite at this size but it
is the bulk of the log for any document source. A reading names its attribute by keyword, not by datom, so it
supports "the current value of `obs/amount`" rather than one specific fact; if a person later corrects the value,
the old reading still points at the attribute and only its text shows what the extractor saw. The `reading/of`
reverse lookup needs the VAET index for every query that asks "why this value".

**Options.** (a) Reified readings, as built. (b) One transaction per field, with `tx/region` and `tx/confidence`
as transaction metadata: exact (the provenance is on the very datom) and no extra entities, but a statement becomes
70 transactions, the chain and the log's "one file, one transaction" rule break, and re-reading a file can no
longer be one empty transaction. (c) Tuple values (`obs/amount` = `[value, region, confidence]`): one fact per
field, but every rule must unpack tuples, the shared vocabulary stops being plain, and the engine has no tuple
type. (d) Keep only `obs/region` and `obs/confidence`: cheapest, and loses exactly what a person checking a
receipt needs.

**Recommendation.** Keep reified readings and promote `reading/*` to core. They leave the shared vocabulary plain,
cost nothing to queries that do not ask for provenance, and give the TUI exactly what it needs to draw a box
around one figure. Make readings optional per source: structured sources (Xero, CSV) need none, since the region
of an item is the provenance of every field in it. Revisit volume only if the benchmark says so.

### Regions as strings versus tuples

**What.** A region is `region/locator` = `page:1/box:54,40.5,233.6,55.3`, a string. The box has no unit, no
origin and no page size, so the TUI cannot draw it without going back to the extraction. A photographed receipt is
rotated 2.3 degrees: the extractor can only give an axis-aligned box around a tilted line.

**What the model did.** The locator stays a string (it is the region's identity, and D11 says a finer reading is a
retract and an assert). The page itself is a region (`page:<n>`) carrying `extraction.region/width`, `/height`,
`/unit` and, for a photo, `/rotation`; every box on that page is read in that unit with a top-left origin.
`region/page` is set from the locator, as the shared helper does.

**What was lost or awkward.** Range queries ("readings in the top third of page 2", "what else is in this box")
need the string parsed. Points versus pixels lives on the page region, one hop away. A rotated box over-covers the
text it bounds.

**Options.** (a) Strings plus page-size facts, as built. (b) Numeric attributes on the region: `region/x0`,
`region/y0`, `region/x1`, `region/y1`, with the locator kept as identity. (c) A quad (four points) instead of a box
for photos. (d) Normalise boxes to 0..1 of the page so units disappear.

**Recommendation.** Keep the locator string as identity and add numeric `region/box-x0..y1` plus page size and
unit on the page region in core. Do not normalise: an extractor's own units are what a person can check against the
file. Quads can wait until a photo source needs them; note rotation on the page meanwhile.

### Raw text versus parsed value

**What.** "$3,215.47", "-$15,062.89", "(3)", "03/09", "8:47 PM", "DEBIT XXXXXXXXXXXX4821" are what the
documents say; the facts need 3215.47, 15062.89, 3, 2026-03-09, 20:47, 4821.

**What the model did.** The extraction carries both; the projector trusts `value` and stores `text` on the
reading. Signs are the projector's: a check's figure and a receipt's totals are printed unsigned and become
negative `obs/amount` (money out of the account); the receipt's `extraction.receipt/*` amounts stay as printed.
The statement's withdrawals total is stored as the printed magnitude.

**What was lost or awkward.** Two places can disagree (text "3,215.47", value "3251.47") and nothing checks that
the value is the text parsed; the extractor is trusted on parsing as on reading. The sign rule lives in code, per
document kind.

**Options.** (a) As built. (b) The projector re-parses `text` and raises pressure when it disagrees with `value`.
(c) Drop `value` and parse in the projector only.

**Recommendation.** (b) for money and dates, which have one obvious parse; keep the extractor's `value` for
everything else. The text must always be stored: it is what a person compares against the image.

### Low confidence, and how a person confirms a field

**What.** The check's signer reads as "Odile Fenwick" at 0.38; the receipt's handwritten tip at 0.62. They must
be kept (a later rule or person may need them) but must not look as solid as the printed figures.

**What the model did.** Low fields are kept with their confidence and named in `low-confidence` pressure (below
0.7). `obs/confidence` counts only `obs/*` readings, so the unsure signer (an extra) does not drag the check's
confidence below its memo's 0.86. A person confirms a field by asserting `reading/review :reading.review/confirmed`
on the reading in their own transaction (`tx/actor person:…`, a `tx/reason`). The projector does not own
`reading/review`, so re-reading the same extraction leaves the confirmation in place (tested).

**What was lost or awkward.** A correction is harder than a confirmation. If a person asserts a different
`extraction.check/signer`, a re-read of the extraction diffs its snapshot against the log and retracts the
person's value, because the projector owns the attribute. The snapshot diff cannot tell its own past assertions
from someone else's.

**Options.** (a) Corrections live only on the reading (`reading/corrected-text`, `reading/review :corrected`) and
a rule applies them. (b) The snapshot diff retracts only values that a transaction from the same projector
asserted (look up each current value's transaction and its `tx/projector`). (c) People never edit observations;
they write their own observation of the same item, and links choose.

**Recommendation.** (b) in `snapshot.rs`, since it is right for every snapshot source (a person's fix to a Xero
contact should also survive a re-read until Xero changes it), together with (a)'s `reading/review` for the
verdict. (c) is the cleanest for audit and is worth keeping for disputed amounts.

### Two totals on one receipt

**What.** The receipt prints a TOTAL of 166.40 (subtotal 152.80 + tax 13.60); the diner wrote a tip of 20.00 and
a total of 186.40, which is what the bank charged.

**What the model did.** One observation (D12): `obs/amount` is the paid total, -186.40, read from the handwritten
figure; `extraction.receipt/printed-total` keeps 166.40, with subtotal, tax and tip beside it, each with its own
reading. Self-checks: the lines add to the subtotal, subtotal + tax = printed total, subtotal + tax + tip =
handwritten total. A `two-totals` pressure says which one `obs/amount` holds.

**What was lost or awkward.** Which total is "the amount" is a projector policy. A receipt with no tip line has
only the printed total, and the projector falls back to it; a receipt with a tip but an unreadable handwritten
total would get the printed total silently if the extractor dropped the field.

**Options.** (a) As built. (b) Two observations (an authorisation of 166.40 and a settlement of 186.40), which is
how card networks see it. (c) Make `obs/amount` many-valued with a role.

**Recommendation.** (a), and promote `receipt/subtotal`, `/tax`, `/tip`, `/printed-total` to core so rules for
sales tax and tips can use them across receipt sources. Add pressure when a tip is present and the paid total is
missing rather than falling back.

### Dates without a year

**What.** Statement lines print `03/09`. The extractor reports `--03-09` (ISO 8601 month-day) rather than guessing.

**What the model did.** The year comes from the statement period: the first of the period's years in which the
month-day falls inside the period. The reading keeps the text "03/09" and `reading/derivation` says "year 2026
from the statement period 2026-03-01..2026-03-31"; one `missing-year` pressure counts the lines. A month-day
outside the period in every candidate year is `check-failed`.

**What was lost or awkward.** Nothing for this statement. A statement spanning December to January works; a
posting dated after the period end (banks do this) does not, and is reported rather than placed.

**Recommendation.** Keep it. Derivations belong on the reading, never silently in the value.

### The account behind a document

**What.** A statement says "Ridgeway Federal Bank, Operating checking, account ending 4821"; a check's MICR line
ends in 4821; a receipt says "DEBIT XXXXXXXXXXXX4821". None of them says `ridgeway:checking:4821`.

**What the model did.** It looks for one bank-side account (one with `account/institution` and no `account/book`)
whose `account/mask` is the document's. If there is none, the statement and the check build the id from what they
print (institution's first word, `checking`, mask) and raise `derived-identity`; the receipt, which names no
bank, gets no `obs/account` and raises `no-identity`. In load order (CSV and OFX first) the lookup finds the
account; on its own, the statement derives the same id.

**What was lost or awkward.** The derivation is a guess, and "checking" is assumed for a check. Masks are not
unique: a savings account or a card could end in the same four digits.

**Recommendation.** Put the binding in the manifest: a source declares its account
(`"bank:ridgeway-4821": {"kind": "bank", "account": "ridgeway:checking:4821"}`) and `SourceSpec` carries it.
Receipts need a card-to-account table (a person states it once, as a fact).

### The same item twice in one document

**What.** The statement's "Checks paid" table repeats line 4 (`CHECK 1042`, 03/09, 3,215.47) on page 2.

**What the model did.** No second observation (D12): the table's number, date and amount become more readings
and a second row region on the line, and the number becomes `obs/reference` "1042". The projector checks that a
matching line exists (`check-failed` otherwise) and says so in `embedded-copy` pressure.

**Recommendation.** Keep it; this is the case for readings, since a single `obs/region` could not say which box
gave the check number.

### Two dates on a check

**What.** Check 1042 is dated 03-03 (written) and posted 03-09 (cleared). The receipt is dated 03-12 and posted
03-13.

**What the model did.** Each observation keeps its own document's date. The check's fingerprint uses its written
date and the text `CHECK 1042`, so it will not equal the bank line's; `two-dates` pressure says to match on
account, `obs/reference` and amount. The receipt's fingerprint uses the merchant name, which normalises to the bank
text `COPPERPOTBISTRO`, with the spend date.

**Recommendation.** Fingerprints match one-to-one witnesses of the same posting. Document-to-line rules need a
date window and the reference: check ↔ line on (account, reference, amount, posted date ≥ written date); receipt
↔ line on (account, amount, text, posted date within three days). Both rules can be written from these facts; no
links were written here.

### The document's file

**What.** `facts::document` sets `doc/file` to the main file, which for an extraction is the JSON, not the PDF.

**What the model did.** It builds the document itself and puts both files in `doc/file`.

**Recommendation.** Give the helper the file to name (see below). The document is the PDF; the extraction is one
reading of it, linked by `file/derived-from`.

### Times with no zone

**What.** The receipt prints "8:47 PM" and no zone. `obs/instant` needs one.

**What the model did.** `extraction.receipt/time` keeps "20:47" as text; `no-zone` pressure.

**Recommendation.** A book has a time zone (config); a later rule can assert `obs/instant` from date, time and the
book's zone, with its own reading derivation.

### Who wrote it

**What.** The facts are written by `projector:extraction@1`, but the reading was done by `agent:tally`.

**What the model did.** `tx/actor` is the projector, as for every ingest; the extractor, its version and when it
read are on the extraction file entity, which is in `tx/evidence`.

**Recommendation.** Keep it: the extractor is evidence, not the author of the facts. A person asking "who read
this?" follows `tx/evidence` to the extraction file.

## Show me what backs this posting

The walk, from a posting to a box on a page, with what each step uses (tested from the observation down):

1. posting → its entry (`entry/posting`, reverse);
2. entry → observation through a `link.kind/evidences` link (`link/from`, `link/to`);
3. observation → its other witnesses through `link.kind/same-event` links (the CSV and OFX lines, the receipt);
4. observation → readings (`reading/of`, reverse), filtered by `reading/attr` to the attribute in question
   (`:obs/amount` for "why this amount");
5. reading → `reading/region` → `region/locator` (`page:1/box:…`) and `region/file` → `file/sha256`;
6. the page region `<sha>#page:<n>` gives the size and unit to draw the box; the drawer gives the bytes.

For the dinner the walk ends at `page:1/box:…` on the statement PDF (text "-186.40", 0.99) and, through a
same-event link, at the receipt photo's handwritten total (text "186.40", 0.81). Without readings, step 4 stops
at `obs/region`: every box of the item, with no way to say which is the amount.

## Proposed shared-file changes

**`rust/crates/ownpurse-facts/schema/core.json`**: add

```json
{"ident": "reading/id", "type": "string", "unique": "identity", "doc": "One field as an extractor read it: <entity id>#<attribute>#<field>"},
{"ident": "reading/of", "type": "ref", "doc": "The entity the field supports"},
{"ident": "reading/attr", "type": "keyword", "doc": "The attribute the field supports, e.g. :obs/amount"},
{"ident": "reading/field", "type": "string", "doc": "The extractor's name for the field"},
{"ident": "reading/text", "type": "string", "doc": "The text as read"},
{"ident": "reading/region", "type": "ref"},
{"ident": "reading/confidence", "type": "decimal", "doc": "0 to 1"},
{"ident": "reading/handwritten", "type": "bool"},
{"ident": "reading/derivation", "type": "string", "doc": "How the value was got when it is not the text parsed"},
{"ident": "reading/review", "type": "keyword", "doc": "A person's verdict: reading.review/confirmed, /corrected, /rejected"},
{"ident": "region/width", "type": "decimal", "doc": "On a page region: the page's width in region/unit"},
{"ident": "region/height", "type": "decimal"},
{"ident": "region/unit", "type": "string", "doc": "pt or px; boxes have a top-left origin"},
{"ident": "region/rotation", "type": "decimal", "doc": "Degrees the content is turned (a photo)"},
{"ident": "balance/side", "type": "keyword", "doc": "balance.side/opening (start of obs/date) or balance.side/closing (end of it)"},
{"ident": "receipt/subtotal", "type": "decimal"},
{"ident": "receipt/tax", "type": "decimal"},
{"ident": "receipt/tip", "type": "decimal"},
{"ident": "receipt/printed-total", "type": "decimal", "doc": "The merchant's total before the tip; obs/amount is what was paid"},
{"ident": "check/amount-words", "type": "string"},
{"ident": "check/signer", "type": "string"}
```

and change the doc of `obs/confidence` to "0 to 1: the lowest confidence among the readings of this observation's
obs/* attributes". The `extraction.*` attributes then map one to one (`extraction.balance/side` →
`balance/side`, and so on); the statement summary, running balance, MICR parts, receipt time and card stay in
`extraction.*`.

**`rust/crates/ownpurse-ingest/src/facts.rs`**:

- `document(tx, input, id, kind)` → add `document_of(tx, input, id, kind, file: &Hash)` (or a `file` argument)
  so a document can name the file it is, not the extraction;
- `region(tx, hash, locator)` → add `region_box(tx, hash, page, [Decimal; 4])` that formats the locator the one
  way (`page:<n>/box:<x0>,<y0>,<x1>,<y1>`, decimals normalised) so two projectors reading the same box name the
  same region;
- add `reading(tx, of: (&EntRef, &str), attr, text, region, confidence)` once `reading/*` is core.

**`rust/crates/ownpurse-ingest/src/snapshot.rs`**: in `diff`, retract a current value only when the transaction
that asserted it has the same `tx/projector` as this read, so a person's correction survives a re-read.

**`rust/crates/ownpurse-ingest/src/lib.rs`**: `SourceSpec` gains `account: Option<String>`, read from the
manifest's `sources` entry, so bank documents name their account instead of deriving it.

**`samples/ingest.json`**: add to `sources."bank:ridgeway-4821"` the key `"account": "ridgeway:checking:4821"`,
and append the entries of `samples/bank/manifest-extraction.json` (statement, then check) after the bank CSV and
OFX entries, then the entry of `samples/receipts/manifest.json`.

**`docs/plans/FACTS.md`**, "Bank statements and evidence": the walk gains a step, observation → readings →
regions, so "what backs this amount" ends at one box rather than at every box of the item.
