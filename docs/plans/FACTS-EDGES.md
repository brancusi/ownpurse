# The fact model at its edges

Status: findings from phase 1, 2026-10-09, branch `facts`. We built the model ([FACTS.md](FACTS.md)), then made one
synthetic month (March 2026, [samples/README.md](../../samples/README.md)) arrive through every kind of source on
the list and recorded every place a source did not fit. This is the summary; each source family has a full
write-up in [edges/](edges/).

## What was built

| Piece | What it is |
|---|---|
| `ownpurse-facts` | The datom `[e a v tx added]`, schema as facts (accretion only), transactions with temp ids, upserts and lookups, the database as a value (`with`, `as_of`, `history`), pull, the canonical encoding and hash chain |
| `ownpurse-store` | The log in SQLite (append-only by trigger, chain checked on open), the drawer of evidence files by SHA-256, `verify`, readable views, exact `dsum()` |
| `ownpurse-ingest` | The ingest API (a file and its companions → a projector → a plan of transactions), the snapshot diff, the JSON mirror, the round-trip law, ten projectors, three link rules |
| `ownpurse-facts` (CLI) | `init`, `ingest`, `load`, `transact`, `entity`, `pull`, `history`, `log`, `sql`, `schema`, `pressure`, `link`, `verify` |

The month: 50 files from 8 sources (Xero ×2 pulls, two overlapping bank CSV downloads, OFX, a statement PDF, a check
image, a receipt photo, QuickBooks, FreshBooks, a Google Sheet read twice, an Excel workbook, an Ethereum wallet,
prices). They load into 84 transactions and 12,933 datoms, and the log verifies (every cited file re-hashed). Loading
everything again changes nothing. The link rules then join 93 pairs: every witness of each bank movement, the receipt
and the check to the lines they back, and two supplier invoices to the bills they became. 142 tests, plus two ignored ones that state the open gaps; 343 pressure
reports in 49 kinds.

Watch it: `bash samples/demo.sh` builds the CLI and plays the month with progress. By hand:

```sh
cd rust
cargo run -p ownpurse-facts-cli -- --log /tmp/march.sqlite init
cargo run -p ownpurse-facts-cli -- --log /tmp/march.sqlite load ../samples/ingest.json
cargo run -p ownpurse-facts-cli -- --log /tmp/march.sqlite link
cargo run -p ownpurse-facts-cli -- --log /tmp/march.sqlite sql -f ../samples/queries/09-what-backs-the-dinner.sql
cargo run -p ownpurse-facts-cli -- --log /tmp/march.sqlite pressure
```

## The verdict

**The datom held.** Nothing in any source forced a change to `[e a v tx added]`, to transactions as the unit of
provenance, to the drawer, or to the chain. Corrections, time travel and audit fell out as promised: an edited Xero
invoice retracts and asserts exactly the fields that changed, a person's note is one transaction with their name on it, and
`--as-of` reads any earlier database.

**The strain is all one level up.** Every finding is about one of five things the datom leaves to us: *what an item
is* (its shape), *when* it is, *where it came from at what grain*, *what it is called*, and *how a read relates to
the reads before it*. Those are the building block we were really looking for.

## The building block

A fact is the atom. The molecule every source decomposed into is the same one:

> **An observation: one item a source states, at a place in a file, about an account, in a commodity, at a time,
> with legs when it moves more than one thing.**

Five patterns make it fit every source we tried:

1. **An event has legs.** A bank transfer touches two accounts; a wallet transaction moves USDC and pays gas in
   ETH; a loan payment splits into interest and principal; a receipt's paid total is subtotal, tax and tip. In each
   case the parent observation is the event and each leg is a child observation (`obs/parent`) with its own
   account, amount and commodity, and a role (`obs/role`: `principal`, `interest`, `fee`, `tip`, `from`, `to`). A
   parent with legs in more than one commodity has no single amount. *Found in:* Xero transfers, Ethereum, the loan
   schedule, the receipt.
2. **Time is one of three things.** An instant (a block time, a card authorisation), a business date (an invoice
   date, a posting date), or a period (a budget month, a statement, a service period). Exactly one per
   observation. When a document has two dates (a check written 03-03 and cleared 03-09; a dinner on 03-12 posted
   03-13), those are two witnesses, linked, not two dates on one fact. *Found in:* the budget, the statement, the
   check, the receipt, Ethereum (a UTC block time is a different local date in the books' zone).
3. **Provenance has a grain.** For structured sources the region of the item (a JSON pointer, a CSV row) is the
   provenance of every field in it. For anything read by an extractor, each field has its own place and confidence,
   so each field gets a *reading* (`reading/of`, `reading/attr`, `reading/text` as seen, `reading/region`,
   `reading/confidence`). The value stays plain on the observation, so rules never see readings. *Found in:* the
   statement, the check, the receipt. Cost: about five facts of provenance per fact read; worth it, and optional per
   source.
4. **Identity has a ladder.** The source's own id (Xero GUIDs, FITIDs, `chain:tx:log`), else a natural key (a sheet
   row's category, a statement and line number), else a fingerprint and occurrence (`account|date|amount|text#n`),
   else a position (`@n`, never equal to a real id). Different sources never share an identity; rules link them
   with a confidence (D4). *Found in:* every source.
5. **A read is a snapshot or an event.** Snapshot sources (Xero, QuickBooks, FreshBooks, sheets, workbooks) resend
   the whole document: diff against the last read, and never let an older read overwrite a newer one. Event
   sources (bank lines, chain transactions) append within a window: identity carries across overlapping windows,
   and absence inside a window is evidence only for pending items. *Found in:* Xero pulls, the overlapping bank
   downloads, the second sheet read, the pending card hold.

And one rule that cut across all of them: **what a source states is a fact; what we compute is a check.** A running
balance printed on a statement is a fact; "opening + lines = closing" is a query that can fail, reported, never
fixed in place.

## The edges, in one table

| Edge | Where | What we did | Decide |
|---|---|---|---|
| A reference is named by its field, not by the id inside it | Xero journal lines, QuickBooks `*Ref`, FreshBooks plain ids | The mirror declares references by field name or by a marker key | Settled |
| Order is lost unless it is a fact | Xero tracking options, lines everywhere | `<ns>/index` on every list item | Settled |
| An id and a position can collide | QuickBooks' subtotal line | Positions are `@n` | Settled |
| An older read overwrites a newer one | Xero, FreshBooks, the sheet | Skip a document read after this read (`stale-read`); every snapshot projector | Settled when the read time is true; an old file uploaded today still looks new: use the source's own version (`SyncToken`, `updated`, `UpdatedDateUTC`) |
| A re-read retracts a person's correction | Extraction | The snapshot diff cannot tell its own past assertions from a person's | **Change the diff to retract only what its own projector asserted** |
| Embedded copies of other entities | Xero `Contact` on an invoice, FreshBooks client address | Keep the reference; the copy stays in the drawer and in `obs/counterparty` | Settled |
| Null, empty and absent | Everywhere | All three are "no fact" | Accept |
| Computed values look like edits | QuickBooks `Balance`, FreshBooks `outstanding`, sheet totals | Mirrored (they are what the source said); totals with formulas are checked, not stored | Mark derived fields in the mirror config |
| Projection depends on load order | Xero account codes, the check before the bank CSV | Lookups at projection time, reported as `unresolved-ref` and `derived-identity` | **Move cross-document lookups into rules over the log** |
| The account behind a document | Statement, check, receipt show "ending 4821" | Found by mask among accounts the log already knows | Accounts across sources are linked, not merged; the mask is the weak key every source shares |
| One event, two accounts / two commodities | Transfers, swaps, fees, loan splits | Parent with legs | Add `obs/role`; say "an event has legs" in the vocabulary |
| Periods | Budget, statements, loan schedules | Projector-local `budget/period-*` | **Add `obs/period-start` / `obs/period-end`** |
| Two totals, two dates on one document | Receipt (printed 166.40, paid 186.40), check (written, cleared) | Both kept; which one is "the amount" is projector policy, reported | Accept; the policy is per projector version |
| Pending versus posted | The bank CSV | A pending hold that drops out becomes `lapsed`, never retracted | Accept |
| No ids in CSV | Bank downloads | Fingerprint and occurrence, stable across overlapping downloads | Accept; documented limits |
| Truncated and abbreviated names | OFX, every bank | Link rules match account, amount and date; text only scores | Settled |
| Local dates without zones, zones that change | OFX (EST to EDT on 03-08), QuickBooks offsets, the receipt's "8:47 PM", block times | Keep the local business date and the raw text; instants in UTC | Add `book/time-zone` |
| Floats where money should be | Excel cached values (`1004.1199999999999`) | Shortest round-trip text, rounded to the mapping's places, reported | Accept |
| Numbers beyond 28 digits | ERC-20 max approval (2^256 − 1) | Raw hex mirrored, `unlimited` keyword; amounts stay `Decimal` | Accept |
| Commodity identity | Tokens | `USDC@ethereum:0x…` | Adopt CAIP-19 codes plus `commodity/symbol` |
| Prices as facts | Daily closes | `price/at` at the next midnight UTC | Add `price/date` |
| Schema growth | All mirrors | 799 attributes after one month; each source's fields are its own namespace | Accept; it is what lossless costs. Watch it in the benchmark |
| Pressure is noisy | All | 343 reports; notes and problems weigh the same | **Give pressure a level (note, problem)** |
| SQL sums float | The log by hand | `dsum()` | Settled |
| Two companies' books in one log | Quillfeather's QuickBooks beside the roastery's Xero | `postings` carries `book`; a trial balance is per book | Settled |
| A source's own number scale | QuickBooks writes `2400.0` where Xero writes `2400.00` | Mirrored as stated; `dsum` keeps the widest scale | Decide: scale is the commodity's, applied when shown |

## Proposed changes to the core

Collected from the five write-ups, deduplicated, in the order we would make them.

1. **Snapshot ownership by projector.** `Snapshot::diff` retracts only values its own projector asserted (looked up
   through `tx/projector`), so a person's or a rule's correction survives a re-read. Add `Snapshot::sweep` to retract
   what an earlier read had and this one does not.
2. **Time.** `obs/period-start` and `obs/period-end` (exactly one of these or `obs/date` or `obs/instant`);
   `book/time-zone`; `tx/source-time` and a per-source version field (`doc/version`) for stale reads.
3. **Legs.** `obs/role` (keyword); the docs of `obs/amount` and `obs/kind` say what a parent with legs means.
4. **Readings.** Promote `reading/*` to core, optional per source; give regions geometry (`region/width`,
   `region/height`, `region/unit`, `region/rotation`) instead of numbers inside a string.
5. **Balances.** `obs/balance-after` (a running balance a line states) and `obs/balance-kind` (opening, closing,
   ledger, available); balance checks move into queries over all witnesses.
6. **Commodities and prices.** CAIP-19 `commodity/code` for on-chain assets, `commodity/symbol`, `price/date`.
7. **Pressure levels** (note, problem) and a `Pressure::note` constructor.
8. **Rules, not lookups.** Account codes, the sales-tax account and "which bank account is this mask" become link
   rules over the log, so a projector depends only on its own file and replay is order-free.
9. **Housekeeping.** Shared interpretation helpers (`kit.rs`, `bank_csv/common.rs`) move to one module; a
   `price.csv@1` projector with its own source.

## Open decisions

- **Mirror entries.** Xero's and QuickBooks' entries are projector claims that depend on other documents (codes, the
  tax account). With change 8 they become derivations (D2), rebuilt by a rule. Agree before phase 2.
- **One entity for document and observation.** A Xero bank transaction is both; it carries `doc/id` and `obs/id`.
  Keep, and make `obs/id` optional (an observation is anything with `obs/kind`)?
- **Reading volume.** Five provenance facts per extracted fact. Fine at this size; the 10M-fact benchmark decides
  whether readings need their own table.
- **Which amount a receipt states.** Paid total (chosen) or printed total, as projector policy.

## Write-ups

- [edges/xero-core.md](edges/xero-core.md): the core and Xero, including what the round-trip law found
- [edges/bank-csv-ofx.md](edges/bank-csv-ofx.md): bank downloads and OFX
- [edges/extraction.md](edges/extraction.md): the statement PDF, the check, the receipt
- [edges/qbo-freshbooks.md](edges/qbo-freshbooks.md): QuickBooks Online and FreshBooks
- [edges/sheets-xlsx-eth.md](edges/sheets-xlsx-eth.md): the budget sheet, the loan workbook, the wallet
