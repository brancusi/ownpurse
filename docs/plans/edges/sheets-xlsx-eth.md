# Edges: a Google Sheet, an Excel workbook, an Ethereum wallet

Status: findings, 2026-10-09. Part of stress-testing [FACTS.md](../FACTS.md) with real-shaped samples. These three
sources are the furthest from double-entry books: a plan, a lender's forecast, and a ledger that is not ours.

| Source | Projector | Samples | Tests |
|---|---|---|---|
| `sheet:tallowmere-budget-2026` | `sheet.values@1` (`sources/sheets.rs`) | `samples/sheets/`: two reads, each formatted plus `FORMULA`, and the mapping | `tests/sheets.rs` |
| `xlsx:ridgeway-loan-2207` | `xlsx@1` (`sources/xlsx.rs`) | `samples/xlsx/`: the schedule workbook and the mapping | `tests/xlsx.rs` |
| `eth:0x51ab7c3e` | `eth.rpc@1` (`sources/eth.rs`) | `samples/eth/`: three RPC bundles, a token list, the wallet mapping, March prices | `tests/eth.rs` |

Each folder has a `manifest.json` fragment shaped like the entries in `samples/ingest.json`. Every address, hash,
id and amount is synthetic; the USDC contract is not the real one. The workbook was written byte by byte (shared
strings, styles with a date and a currency format, formulas with cached values, a merged title), because writers
that compute formulas do not store cached results.

What loading all of it does, from the CLI (`load` over the three fragments):

| File | + | − | Pressure |
|---|---|---|---|
| `budget-2026.values.json` | 1,442 | 0 | period, derived-value, formula-entry ×2 |
| `budget-2026-read-2.values.json` | 511 | 29 | as above, plus identity, unmapped-key |
| `loan-2207-schedule.xlsx` | 866 | 0 | float ×19, derived-value ×4, merged, skipped-row |
| three RPC bundles | 222 | 0 | date, precision, unknown-event, multi-commodity |
| `prices-2026-03.csv` | 386 | 0 | price-date, price-source |

## Periods versus dates

**What.** A budget figure is "Rent, March 2026: 2,750.00". It covers a month. The core vocabulary gives an
observation one `obs/date`; periods exist only on documents (`doc/period-start`, `doc/period-end`).

**What the model did.** Each figure is `obs.kind/budget` with `budget/period-start` and `budget/period-end` (dates,
installed by the projector) and no `obs/date`. The sheet's document carries the year as `doc/period-*`.

**What was lost or awkward.** A query that wants "every observation in March" has to know two vocabularies: `obs/date`
for things that happened and `budget/period-*` for things that cover a span. Putting the first of the month in
`obs/date` would have been easier to query and wrong: it says money moved on the 1st, and a variance report would
compare a plan for the month with one day's actuals.

**Options.** (a) Keep a projector-local pair, as now. (b) Add `obs/period-start` and `obs/period-end` to core, used
instead of `obs/date` when the item covers a span. (c) A period entity (`period/id` `2026-03`, with start, end and a
kind), referenced by `obs/period`. (d) FACTS.md's own idea: a budget is a plan book (`book/kind :plan`) with entries
dated at the period end.

**Recommendation.** (b) now: two dates on the observation, the same shape documents already use, and the rule
"exactly one of `obs/date` or `obs/period-*`". Accruals, statement lines with a service period, subscriptions and
depreciation runs all need the same thing. (d) is a derivation, not a projection: a person decides that the budget
is a plan book over the chart, and a rule turns budget observations into plan entries (D3).

## Wide to long, and the identity of a sheet cell

**What.** The sheet is wide: one row per category, one column per month. The model is long: one item per fact. A
sheet has no ids; the mapping names a key column (`Category`) and the value columns (`Jan` … `Dec` → `2026-01` …).

**What the model did.** Each row is a `budget.line` entity (`<source>/<sheet>/<key>`, with its name, account code,
side, note and position); each non-blank month cell is one observation, `<line id>/<YYYY-MM>`, pointing at its line
and at its cell (`region/locator` `cell:Budget!D8`). The second read moves "Bank fees" two rows down, changes one
figure and renames "Design and branding" to "Brand and design". The test counts exactly what that does:

- the moved row: nothing about its figures; three lines' `budget.line/position` change (3 retracted, 3 asserted);
- the changed figure: one `obs/amount` retracted, one asserted;
- the rename: the old line (7 facts) and its two figures (9 facts each) are retracted, a new line and two new
  figures are asserted, and the new line has no account code, because the mapping names categories by their old
  text (`unmapped-key`);
- in all, 29 retractions and 26 assertions of change, plus 462 assertions of regions (110 cells × 4, and 22
  formulas), 9 for the new files and 14 for the transaction itself: 511.

**What was lost or awkward.** Three things. A rename is indistinguishable from a delete and an add: the history of
"Design and branding" stops and "Brand and design" starts, and a person's note or link on the old line now points at
a retracted entity. `Snapshot` has no notion of "the whole set": an entity absent from this read is never retracted
unless the projector looks up what the document said before and calls `gone()` for each (both `sheets.rs` and
`xlsx.rs` carry the same loop). And regions dominate: every read mints a region per cell because a region's identity
is `<file sha>#<locator>`, so a read that changes one number writes 462 facts of where things are.

**Options.** For identity: key by a stable column when the sheet has one (an account code); let the mapping carry
renames (`"renamed": {"Brand and design": "Design and branding"}`) so an agent that notices keeps the identity; or
match a vanished line to a new one by its figures and propose a `link.kind/same-line` for a person to accept. For
absence: a `Snapshot::sweep` that retracts what a scope (a document) held and this read does not. For regions:
attach a region only to observations this read asserts or changes, since `tx/evidence` already names the file for
everything in the transaction; or make regions per row; or keep them all.

**Recommendation.** Key by what the mapping says, and let the mapping carry renames: the agent that wrote the mapping
is the one placed to see a rename, and its version of the mapping is evidence like any file. Add `Snapshot::sweep`
(below). Attach regions only to what a read asserts or changes: provenance of an unchanged fact is the read that
first asserted it, and "where is it in the latest file" is cheap to recompute from the mapping.

## Computed versus entered values

**What.** The budget has a Total column and two computed rows (Total expenses, Net). Two of its figure rows are
formulas too: Rent's February to December are `=B8`, `=C8` …, "same as last month". The loan schedule is almost
all formulas: dates (`=EDATE(B6,1)`), interest (`=ROUND(F6*$B$2/12,2)`), principal, balance. Only the payment
column is typed in.

**What the model did.** The Sheets API's formatted read cannot tell typed from computed, so the projector also reads
the range with `valueRenderOption=FORMULA` (a `formulas` companion). Cells in the mapping's computed rows and columns
are not stored: they are checked, by evaluating their formula (sums and differences of cells and `SUM` ranges) over
the figures, and the result is reported (`derived-value`: 36 cells agree). A formula inside a figure row is a plan
figure all the same; its region carries `sheet.cell/formula`. In the workbook, a formula's cached result is the fact
(FACTS.md: "a formula's computed value is the fact"), each value's region keeps the formula and says which
attribute the cell supplied (`sheet.cell/field`), and the totals row is checked against its rows and not stored.

**What was lost or awkward.** The distinction lives on regions, not on the facts: to ask "which amounts did someone
type?" you walk observation → region → formula. Two kinds of computed cell need different treatment and only the
mapping can tell them apart: a derivation of other facts (a total, D2 says it is not authoritative) and an input
written as a formula (Rent's `=B8`). The sheet's own totals are gone from the log; only the pressure line says they
agreed.

**Options.** Keep formulas on regions (now); add `obs/computed` (a bool) on the observation; or store totals as
observations marked derived.

**Recommendation.** Keep it on regions, move the attributes to core as `region/formula` and `region/field`, and keep
not storing derivations. A failed check must stay loud (`check-failed` pressure, and in time a question for the
person) because a sheet whose total disagrees with its rows has been edited by hand.

## Excel serial dates and floats

**What.** Excel stores a date as a day count (46109 is 2026-03-28) in either the 1900 system (with Lotus's fictitious
1900-02-29, day 60) or the 1904 system, and every number as an IEEE double. calamine returns `f64`. The cached
results carry float noise: the March principal is `1004.1199999999999`, February's balance `61359.99999999999`.

**What the model did.** No float arithmetic: an `f64` becomes a decimal through its shortest round-trip text
(`format!("{f}")` then `parse_decimal`), which keeps exactly what the cell holds, and then rounds to the mapping's
`decimals` (2), half away from zero, as Excel's `ROUND` does. Every cell that needed rounding is reported (`float`,
19 places). Serial dates are integers through the same text path and converted with `excel_date(serial, 1904)`,
which refuses day 60 and corrects days 1 to 59; the workbook's `date1904` flag comes from calamine.

**What was lost or awkward.** The decimals come from the mapping, not the cell: the number format says `"$"#,##0.00`
but calamine does not expose it, so a cell formatted with three places would be silently read at two. A
`1004.1199999999999` is not what the lender meant, and also not exactly what the file says; the file's own text is
shortest-round-trip, which is what the conversion recovers, but that is a property of how Excel writes, not a
guarantee.

**Options.** Read the cell XML text ourselves (needs the zip, which calamine does not re-export); read number formats
for the places; or trust the mapping.

**Recommendation.** Keep the text path and the mapping's decimals, and make the rounding visible (as now). Ask
calamine for raw cell text if this projector meets real lenders' files; it is the one place float noise could
change a cent.

## One row, two amounts

**What.** A schedule row is one scheduled payment of 1,126.84 that is two things in the books: 122.72 interest (an
expense) and 1,004.12 principal (a reduction of a liability). It also states the balance after it, 60,355.88.

**What the model did.** The row is one `obs.kind/scheduled-payment` (`obs/status` `obs.status/scheduled`, amount
−1,126.84, the balance as `loan.schedule/balance-after`), with two child observations,
`obs.kind/scheduled-payment-part`, under `obs/parent`, each with its amount, `loan.schedule/component`
(`loan.component/interest` or `principal`) and the account code the mapping gives (6500, 2500). The projector checks
interest + principal = payment and the balance chain row to row, starting from "Balance forward".

**What was lost or awkward.** The role of a child (interest or principal) is a projector-local keyword, though every
split needs one: tax and net on an invoice line, tip and subtotal on a receipt, principal and interest here, fee and
transfer on chain. The "Balance forward" row is used for the check and then dropped. A scheduled payment is a
forecast; the core has no status vocabulary to say so, and nothing yet links the schedule's March row to the bank's
03-28 line and Xero's split.

**Options.** A core `obs/role` keyword for children; balances as `obs.kind/balance` observations; or a scheduled
payment as an entry in a plan book.

**Recommendation.** Add `obs/role` to core, and document `obs.status/scheduled`. Keep the balance as an extra for
now: a forecast balance is not a stated balance and should not meet `balance` checks. The link from the schedule
row to the bank line is a link rule's job (amount, date, "LOAN PMT 2207"), with `link.kind/same-event`.

## Multi-commodity events

**What.** Sending 1,000.00 USDC costs gas in ether: one on-chain transaction, two commodities, two balances move.

**What the model did.** One `obs.kind/chain-transaction` per transaction touching the wallet (block time, books'
date, status, block, gas), with a child leg per thing that moved: `chain-transfer` for the USDC and
`chain-fee` for the ether, each with its own `obs/amount` and `obs/commodity`. Amounts are only on the legs; the
parent has none, because there is no single amount to give it.

**What was lost or awkward.** Nothing, in the model; this is D13 working. What is awkward is that `obs/amount` on a
parent means "the total" for an invoice and nothing for a chain transaction, so a query that sums parents and
children double-counts or misses. The `multi-commodity` pressure marks it.

**Options.** Document that a parent with legs has no amount when its legs are in different commodities.

**Recommendation.** Make that the rule for every source: when children carry amounts in more than one commodity, the
parent has no `obs/amount`. Queries sum leaves.

## Fees

**What.** The fee is `gasUsed × effectiveGasPrice` wei, paid by whoever sent the transaction, whether it succeeded or
failed. On 03-02 the exchange sent the ether and paid its gas; on 03-21 Fernhill paid the gas; on 03-26 the wallet
paid 46,109 × 23,418,770,219 = 1,079,816,076,027,871 wei, 0.001079816076027871 ETH.

**What the model did.** The fee is a leg only when the wallet sent the transaction (`eth.tx/fee-payer` records who
did, always). Integer multiplication in `u128`, then scaled by 18 decimals. A failed transaction still gets its fee
leg and no value legs.

**What was lost or awkward.** On rollups the receipt adds an `l1Fee`; the projector reports it and does not add it.
A fee's account (an expense, or part of the cost of what was sent) is a posting decision, not an observation.

**Recommendation.** Keep fees as their own legs and leave the accounting to posting rules. Add the rollup fee when a
rollup wallet arrives.

## Precision beyond 28 digits

**What.** ERC-20 values are `uint256`. `rust_decimal` holds 96 bits of mantissa: 79,228,162,514,264,337,593,543,950,335,
29 digits. The maximum allowance `2^256 − 1` has 78. Real balances fit (a trillion USDC is 19 digits in units;
ether at 18 decimals fits up to about 79 billion ETH), but raw log values need not.

**What the model did.** Every log in a receipt is mirrored raw (`eth.log/*`: contract, topics in order, data as hex).
A decoded value is stored as `eth.log/value` when it fits; the maximum is `eth.log/unlimited true`; anything else past
128 bits keeps only its hex. The Fernhill payment came through a checkout contract, so its receipt carries an
`Approval` with the maximum allowance: it is mirrored, marked unlimited, reported as `precision`, and is never an
observation, because an allowance moves nothing. The checkout contract's own event cannot be decoded; it is mirrored
and its data, read as text, is `INV-0418`, the invoice it pays (`unknown-event`).

**What was lost or awkward.** A number the log holds is not a number the model can compare or sum. Text keeps it
exactly but not as a quantity.

**Options.** A big-integer value type; store raw quantities as text with a typed wrapper; or keep amounts in
`Decimal` and raw values as hex, as now.

**Recommendation.** Keep `Decimal` for amounts and hex for raw values, and add no value type yet. If a source ever
needs a quantity past 28 digits in a sum, the answer is a scale change (store in larger units) before a new type.

## Commodity identity

**What.** "USDC" is a symbol, not an identity: the same symbol exists on many chains and as bridged copies, and
anyone can deploy a token called USDC. Ether on mainnet and ether on a rollup are different assets with one symbol.

**What the model did.** Ether is `ETH` (scale 18, `commodity.kind/crypto`); USDC is
`USDC@ethereum:0x0c5d…5c79` (scale 6, `commodity.kind/token`), with `eth.token/address` and `eth.token/chain-id`, as
core.json's own example suggests. Prices name the base by symbol, so the projector resolves a symbol through the
token list and refuses one that names two tokens.

**What was lost or awkward.** The code has the symbol in it, so a token that changes its symbol changes its identity.
`ETH` will mean mainnet ether forever, and a rollup's ether needs a different code by convention only.

**Options.** Keep `<symbol>@<chain>:<contract>`; or use CAIP-19 (`eip155:1/erc20:0x…`, `eip155:1/slip44:60`) as the
code with the symbol as a separate label.

**Recommendation.** CAIP-19 for on-chain assets, with a new `commodity/symbol` for display, and ISO 4217 codes for
currencies. It is a standard, it carries the chain id, and it never contains a label.

## Timestamps versus dates

**What.** The Fernhill payment's block time is 2026-03-22T03:41:23Z. In Chicago, where the books are kept, that is
22:41 on 03-21, the date the README gives and the date the invoice is paid. Chicago moved from −06:00 to −05:00 on
03-08, between the first transaction and the second.

**What the model did.** `obs/instant` is the block time; `obs/date` is the local date, from UTC offsets the wallet
mapping lists by when they start (an agent wrote them; there is no time zone database in the pure core). When the
two dates differ, `date` pressure says so.

**What was lost or awkward.** A fixed offset would have been wrong for one of three transactions. A list of offsets
in a mapping goes stale.

**Recommendation.** The accountant's date is the books' local date, and it belongs on the observation; the instant
stays beside it. Put the zone on the book (`book/time-zone`) and have the shell, which may read a time zone database,
pass resolved offsets in, so projectors stay pure.

## Prices as facts

**What.** `prices-2026-03.csv` gives a daily close for ETH-USD and USDC-USD. Core has `price/commodity`,
`price/quote`, `price/value`, `price/at` (an instant), `price/source`.

**What the model did.** One `price` per base per day, `price/id` `<source>/<code>/<quote>/<date>`, at 00:00 UTC the
next day (the moment a UTC daily close is taken). 62 prices, exact decimals.

**What was lost or awkward.** A close is "the price for 2026-03-21", but stored as an instant on 03-22, which reads
wrong and makes "the price on the date of this observation" an off-by-one waiting to happen. The prices arrived with
the wallet's files, so `price/source` is the wallet's source; they really come from a price feed.

**Options.** Add `price/date` for daily prices; give prices their own source (`prices:<feed>`) and projector.

**Recommendation.** Both: `price/date` beside `price/at`, and a `price.csv@1` projector with its own source, so the
wallet projector reads only RPC responses.

## Smaller things

- The loan in `samples/README.md` is "60,000.00 at 2.40 %", but a balance of 61,360.00 before the March payment
  means more was borrowed. The workbook brings 63,362.24 forward into 2026 (payment 14, 2025-12-28), which reaches
  exactly 61,360.00 after February with interest rounded to the cent.
- The budget's categories are matched to the chart by name in the mapping. A rename breaks that too; an account code
  column in the sheet would fix both identity and the mapping.
- `.gitignore` drops `tokens*.json` (OAuth token files), which also catches a token list; the sample is
  `samples/eth/token-list.json`. A real connector will keep token lists in the drawer, where the rule does not apply.
- Ownership is per attribute namespace, not per entity and source. The wallet projector describes commodities it
  shares with every other source, so it leaves `commodity/*` out of `owns`; otherwise a re-read would retract a name
  another source gave the same commodity.

## Proposed shared-file changes

`rust/crates/ownpurse-facts/schema/core.json`, add:

```json
{"ident": "obs/period-start", "type": "date", "doc": "First day of the span the item covers (a plan figure, an accrual); used instead of obs/date"},
{"ident": "obs/period-end", "type": "date", "doc": "Last day of the span the item covers"},
{"ident": "obs/role", "type": "keyword", "doc": "What a child is to its parent: obs.role/interest, obs.role/principal, obs.role/fee, obs.role/tax, obs.role/tip"},
{"ident": "region/formula", "type": "string", "doc": "The formula behind a cell's value, when a formula computed it"},
{"ident": "region/field", "type": "keyword", "doc": "The attribute this place supplied, e.g. obs/date"},
{"ident": "commodity/symbol", "type": "string", "doc": "The label people use (USDC, ETH); not an identity"},
{"ident": "price/date", "type": "date", "doc": "The day a daily price is for (a close), beside price/at"},
{"ident": "book/time-zone", "type": "string", "doc": "IANA zone the book's dates are in, e.g. America/Chicago"}
```

and amend these docs: `obs/kind` (add `obs.kind/scheduled-payment`, `obs.kind/chain-transaction`,
`obs.kind/chain-transfer`, `obs.kind/chain-fee`), `obs/status` ("obs.status/scheduled for a forecast"),
`obs/amount` ("absent on a parent whose children are in more than one commodity"), `commodity/code` ("ISO 4217 for
currencies; CAIP-19 for on-chain assets, eip155:1/erc20:<contract>").

`rust/crates/ownpurse-ingest/src/snapshot.rs`, add:

```rust
/// Retract every entity that `scope` = `scope_value` held before and this snapshot does not name by
/// `identity`: the rows a re-read of a document no longer has.
pub fn sweep(&mut self, db: &Db, scope: &Kw, scope_value: &Value, identity: &Kw) {
    let (Some(a), true) = (db.attr(scope).map(|d| d.id), db.attr(identity).is_some()) else { return };
    let named: BTreeSet<Value> = self.ents.values()
        .filter(|d| &d.identity.0 == identity).map(|d| d.identity.1.clone()).collect();
    for e in db.with_value(a, scope_value).collect::<Vec<_>>() {
        if let Some(v) = db.get(e, identity).filter(|v| !named.contains(*v)) {
            self.gone.push((identity.clone(), v.clone()));
        }
    }
}
```

`rust/crates/ownpurse-ingest/src/facts.rs`: a `region` that writes into a `Snapshot` (both sheet projectors build
their own today), and, once the region recommendation is taken, a way to attach regions only to what a read
asserts or changes.

`rust/crates/ownpurse-ingest/src/sources/mod.rs`: register a `price.csv@1` projector (moving `prices()` out of
`eth.rs`), with a `prices:<feed>` source.

`samples/ingest.json`: append the entries of `samples/sheets/manifest.json`, `samples/xlsx/manifest.json` and
`samples/eth/manifest.json`, in that order; the source ids already match.

`samples/README.md`: the loan line becomes "Loan 2207 at 2.40 %, monthly payment 1,126.84; 63,362.24 brought into
2026, 61,360.00 owed before the March payment, which splits 1,004.12 principal and 122.72 interest"; the wallet's full
address is `0x51ab9e04c2d7f3a1b6e85d20c4f9a7136b0e7c3e`.

Once core has them, the projectors drop `budget/period-*`, `sheet.cell/*` and `loan.schedule/component` for the core
attributes; `sheet.doc/title`, `budget.line/*`, `loan.schedule/*`, `eth.*` stay as source extras (D12).
