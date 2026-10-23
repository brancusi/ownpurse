# Edges: bank CSV downloads and OFX statements

What reading Ridgeway's activity downloads and its OFX statement into facts taught us. The samples are
`samples/bank/activity-2026-03-20.csv`, `activity-2026-04-02.csv` (read through `ridgeway-csv-mapping.json`) and
`2026-03.ofx`; the projectors are `bank.csv@1` and `bank.ofx@1`; the tests are `tests/bank_csv.rs` and
`tests/ofx.rs` in `ownpurse-ingest`. Every amount agrees with the month table in `samples/README.md`: opening
24,310.55, 13 lines, closing 13,413.22.

## What the projectors write

- One account, `ridgeway:checking:4821`, with its mask, name, institution, `USD` (scale 2) and source.
- Every bank line is an `obs.kind/money-movement` with account, date, amount (money in positive), commodity,
  description, reference (the check number), status, fingerprint and one region per file it appears in.
- CSV lines: `obs/id` = `<source>|csv|<fingerprint>#<n>`, where `n` counts identical lines from the earliest.
  Pending lines are `<source>|csv-pending|…`. Extras: `bank.csv/type`, `bank.csv/balance-after`,
  `bank.csv/occurrence`.
- OFX lines: `obs/id` = `<source>|ofx|<account>|<FITID>`. Extras: `bank.ofx/fitid`, `bank.ofx/trntype`,
  `bank.ofx/memo`, `bank.ofx/dtposted` (the raw text with its zone bracket).
- Documents: one feed per account and format (`doc.kind/bank-activity`) that every line points at; one
  `doc.kind/bank-download` per CSV file with its window (`doc/period-*`), when it was taken
  (`bank.csv/observed-at`) and the mapping it was read with (`bank.csv/mapping`); one
  `doc.kind/bank-ofx-statement` per OFX period.
- OFX `LEDGERBAL` and `AVAILBAL` are `obs.kind/balance` observations of the statement
  (`bank.ofx/balance-kind`).

## Strains

### Identity without ids

**Source.** A CSV line has no id. Two lines can be identical in every column but the running balance: the
two HARBOR STREET COFFEE lines on 03-06. **Model.** Identity is the fingerprint (account, date, amount, text)
plus an occurrence number counted from the earliest line, so the coffees are `#1` and `#2` in both downloads.
**Lost or awkward.** The occurrence number only works if every download shows the same set of identical lines
for a day. Day-granular windows guarantee that today; a bank that lets you download "since 14:00" would not.
The running balance tells the two coffees apart (26,151.93 and 26,147.18), but it cannot be part of identity,
since a correction earlier in the month changes every later balance. **Options.** (a) fingerprint + occurrence
(chosen); (b) fingerprint + running balance; (c) file hash + row, which makes every overlapping download a new
set of lines. **Recommendation.** Keep (a), and say in the pressure log (`no-identity`) that the identity is
derived. Propose `obs/occurrence` in the core vocabulary so link rules and the statement-PDF projector use the
same counting.

### Overlapping windows

**Source.** The 03-20 download covers 03-01 to 03-20; the 04-02 download covers the whole month. Eight lines
appear in both. **Model.** The same identities, so re-reading the overlap asserts nothing new except one more
region per line. The 04-02 file after the 03-20 one adds exactly five lines; reading the 03-20 file again
after that is an empty transaction. **Lost or awkward.** `obs/doc` takes one value. If each download were the
line's document, the second download would retract the first as the line's document. So the line points at a
per-account feed document, and each download is a second document that holds the window. The walk from a line
to "which downloads showed it" goes line → regions → files → download documents, which works but is two hops
longer than it should be. **Options.** (a) feed document plus per-download documents (chosen); (b) make
`obs/doc` many; (c) first witness wins (only assert `obs/doc` when the line is new). **Recommendation.** Keep
(a). It matches what a bank actually has (one list per account, many windows onto it). Do not make `obs/doc`
many: a receipt or an invoice line really has one document.

### Pending and posted

**Source.** The 03-20 download shows `PENDING - KESTREL FUEL #0417`, a 1.00 card verification hold, at the top
with no running balance. It is not in the 04-02 download. **Model.** Pending lines are their own observations
(`csv-pending`, `obs.status/pending`) and stay out of the balance check. When a newer download whose window
covers the line's date no longer shows it, the projector asserts `obs.status/lapsed` (pressure
`pending-lapsed`); nothing is retracted, so the history still says it was pending. If the older download is
read after the newer one, the line is written as lapsed at once (`out-of-order`), so order of reading does not
change the result. A posted line that disappears from a newer covering download is only pressure (`vanished`),
never a change. **Lost or awkward.** A pending line that later posts under other words, another date or
another amount (a restaurant tip) cannot be told apart from one that dropped: both lapse, and the posted line
is a new observation. Whether it posted is a link question, not an identity one. Deciding status from absence
needs the projector to read earlier facts (windows, when each download was taken), which makes `bank.csv@1`
depend on the order of the log, not just the file. **Options.** (a) lapse on absence, link later (chosen);
(b) retract pending lines when they disappear; (c) never record pending lines. **Recommendation.** Keep (a), and
add a link rule `link.kind/same-event` from a lapsed pending line to a posted line on the same account within a
few days with the same merchant text, amount allowed to differ, confidence below 1.

### Debit and credit columns, and signs

**Source.** Ridgeway splits money out and money in into two positive columns. **Model.** Amount = credit −
debit, so money in is positive as `obs/amount` requires. A negative number in the debit column, or both columns
filled, is read anyway and reported (`sign`). OFX's `TRNAMT` is already signed; a sign that disagrees with
`TRNTYPE` (a positive `CHECK`) is reported and kept as written. **Lost or awkward.** Nothing in this sample.
Card downloads often flip the convention (a purchase is positive), and only the mapping can say so.
**Recommendation.** The mapping says what each column means in words (`"debit_means": "money out"`). Add an
explicit `"sign": "money-in-positive" | "money-out-positive"` for single-amount columns before the first card
download.

### Local dates without zones

**Source.** CSV dates are `MM/DD/YYYY` in the bank's local day, with no time. **Model.** `obs/date`, which is
a business date, not an instant. **Lost or awkward.** Nothing for matching within one bank. Matching a card
line to a receipt photo or a block time near midnight needs the zone, which the file does not give.
**Recommendation.** Keep business dates as dates. Put the institution's zone on the account
(`account/zone`, proposed below) when a rule needs it, rather than guess per line.

### OFX zone brackets and placeholder times

**Source.** `DTPOSTED` is `20260302120000[-5:EST]` before 03-08 and `20260309120000[-4:EDT]` after, and every
line is posted at 12:00:00. **Model.** `obs/date` is the local date before the bracket; the raw text is kept in
`bank.ofx/dtposted`; no `obs/instant` is written, because noon is a placeholder (`placeholder-time`) and the
brackets change mid-statement (`zone-mixed`). **Lost or awkward.** A real time, where a bank gives one, is
dropped too. Converting to UTC would move a line posted late in the evening to the next day and break the date
match with the CSV and the statement. **Recommendation.** Keep the local date as the date. Write `obs/instant`
only when times vary within a file, which is a cheap test for "these are real".

### Truncated names

**Source.** OFX 1.02 cuts `NAME` at 32 characters: `SALTMARSH CUPPING LAB LLC PAYMEN`, with the full text
in `MEMO`. **Model.** `obs/description` is `NAME` as written; `MEMO` is kept; a 32-character name is reported
(`truncated`). **Lost or awkward.** The fingerprint includes the text, so the OFX line and the CSV line for
the same payment (`SALTMARSH CUPPING LAB`) have different fingerprints. Across all 13 lines, 12 fingerprints
match exactly and this one does not (`csv_and_ofx_lines_share_fingerprints_but_stay_apart`). Matching CSV to
Xero is worse: Xero's text is a contact name. **Options.** (a) rules match on `account|date|amount` and score
the text separately; (b) drop text from the fingerprint; (c) normalise text harder. **Recommendation.** (a).
The fingerprint string is fine as a key for exact agreement, but rules need its parts. Add a helper that gives
the `account|date|amount` prefix (below), and keep the text score out of identity.

### Naming the account from an OFX file

**Source.** OFX names the account only by number and type (`ACCTID 9990004821`, `CHECKING`). **Model.** The id
`ridgeway:checking:4821` is built from the source id's institution part, the type and the last four digits,
and reported (`account-identity`) when the account is new to the log. **Lost or awkward.** The convention lives
in code. A second bank with a different source id shape would get a wrong id silently. The OFX file's own
institution name (`RIDGEWAY FEDERAL`) is not asserted, because `account/institution` takes one value and the
mapping already says "Ridgeway Federal Bank"; two witnesses of an account fight over cardinality-one fields.
**Recommendation.** Let a bank source carry its accounts in config (or a small companion file like the CSV
mapping), keyed by mask and type, and let the OFX projector look them up. Treat account names and institutions
as asserted by a person or the mapping, not by each file.

### The mapping as evidence

**Source.** The mapping is a JSON file an agent wrote after looking at one download: preamble lines, columns,
date format, order, the pending rule and the account. **Model.** It is a drawer file like any other: cited by
`tx/evidence` on every download's transaction and named by `bank.csv/mapping` on the download document. A
mapping that names another account than the download's preamble is refused
(`a_mapping_for_another_account_is_refused`). **Lost or awkward.** Who wrote the mapping and when is inside the
JSON, not in facts. A fixed mapping is a new file, so replaying with it is a new transaction per download, which
is right, but nothing says "mapping B replaces mapping A". **Recommendation.** Propose `doc/read-with` (many, ref
to file) in core for every projector that takes a mapping, and a `file/supersedes` ref for a corrected mapping.

### Balances: facts or checks

**Source.** The CSV shows a running balance after each line; the OFX states a ledger balance as of 03-31 and an
available balance as of 04-02. **Model.** What a source states is a fact: the running balance is
`bank.csv/balance-after` on the line, and the OFX balances are `obs.kind/balance` observations. What we compute
is a check, never a fact: previous balance + amount = this balance, line by line, and the OFX ledger balance
against the CSV's running balances for the same day. A mismatch is pressure (`balance-mismatch`,
`balance-disagrees`); a broken cent in the test produces exactly two mismatches (the line itself and the one
after it) and the line's amount is left alone. **Lost or awkward.** The opening balance (24,310.55) is not
stated by either file; it is derived (first balance − first amount) and appears only in a pressure detail when
the OFX has nothing to check against. The checks run at ingest, so a later file that would make an earlier
check fail does not revisit it. End-of-day balance needs the order within a day, which the facts do not keep
(the check works around it by requiring the ledger to be both a shown balance and opening + the day's lines).
**Options.** (a) stated balances as facts, checks as pressure at ingest (chosen); (b) also write derived
openings as facts with a rule actor; (c) checks as queries only. **Recommendation.** (a) now and (c) later: the
same checks belong in the query layer, run over all witnesses, so that the statement PDF's opening and closing
balances check the CSV and the OFX without either projector knowing about the other. Propose
`obs/balance-after` and `obs/balance-kind` in core so those queries do not depend on `bank.csv/*`.

### FITID reuse

**Source.** Banks sometimes reissue FITIDs or reuse one for a changed line. **Model.** If an OFX line's FITID is
already in the log with another date or amount, the newer reading replaces it (cardinality one) and the change
is reported (`fitid-reused`). **Lost or awkward.** The earlier reading survives only in history. **Recommendation.**
Fine as is; the history is the record of the change, and the pressure says where to look.

### Locating a row

**Source.** CRLF line endings and a byte-order mark. **Model.** `row:<n>` is the physical line in the file, so
the check is `row:14` of the 04-02 download. **Lost or awkward.** With CRLF endings the csv reader reports a record
as starting before the previous line's newline, one line early; the projector counts newlines from the
record's byte offset instead. A quoted field with an embedded newline still gets the line it
starts on. **Recommendation.** Keep physical lines: a person opening the file in a text editor finds the row.

### Pressure that is a note

**Source.** Every CSV says `no-identity`, every OFX says `placeholder-time`. **Model.** Pressure lines, with the
same weight as `balance-mismatch`. **Lost or awkward.** The notes drown the problems in `ownpurse-facts
pressure`. **Recommendation.** Give pressure a level (note or problem), below.

## Proposed changes to shared files

`rust/crates/ownpurse-facts/schema/core.json`, add:

```json
{"ident": "obs/balance-after", "type": "decimal", "doc": "The running balance the source shows after this item"},
{"ident": "obs/balance-kind", "type": "keyword", "doc": "For obs.kind/balance: obs.balance/ledger, obs.balance/available, obs.balance/opening, obs.balance/closing"},
{"ident": "obs/occurrence", "type": "int", "doc": "Which of several identical items (same fingerprint) this is, counted from the earliest"},
{"ident": "doc/as-of", "type": "instant", "doc": "When the source produced this document: a download's time, an OFX DTSERVER"},
{"ident": "doc/read-with", "type": "ref", "many": true, "doc": "Files that told the projector how to read this one: a column mapping"},
{"ident": "file/supersedes", "type": "ref", "doc": "An earlier file this one corrects: a fixed mapping"},
{"ident": "account/zone", "type": "string", "doc": "The IANA zone the institution's business dates are in"}
```

and change the doc of `obs/status` to: "obs.status/posted, obs.status/pending, obs.status/lapsed (a pending
item that a later read covering its date no longer shows), or the source's own status".

When these land, `bank.csv/balance-after`, `bank.csv/occurrence`, `bank.csv/observed-at`, `bank.csv/mapping`
and `bank.ofx/balance-kind` become `obs/balance-after`, `obs/occurrence`, `doc/as-of`, `doc/read-with` and
`obs/balance-kind`.

`rust/crates/ownpurse-ingest/src/facts.rs`, add beside `fingerprint`:

```rust
/// `account|date|amount`: the part of a fingerprint every witness of a bank line agrees on. Link rules match
/// on this and score the text separately, since banks cut and abbreviate names.
pub fn day_amount(account: &str, date: &str, amount: Decimal) -> String {
    format!("{account}|{date}|{}", amount.normalize())
}
```

`rust/crates/ownpurse-ingest/src/lib.rs`: add a level to `Pressure` so notes do not drown problems:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum Level { Note, #[default] Problem }
// Pressure gains `pub level: Level`; `Pressure::new` keeps Problem; `Pressure::note(kind, at, detail)` makes a
// Note; `line()` becomes "<kind> | <locator> | <detail>" unchanged, and `plan` writes notes as
// "note: <kind> | …" so the CLI can filter them.
```

`rust/crates/ownpurse-ingest/src/sources/mod.rs`: move `sources/bank_csv/common.rs` to `sources/bank.rs` and
declare `pub(crate) mod bank;`, so the statement-PDF and check projectors name the account and write lines with
the same helper (`bank::account`, `bank::line`) instead of reaching into `bank_csv`.

`samples/ingest.json`: insert the 03-20 download after `xero/pull-1/ManualJournals.json`, and the 04-02 download
and the OFX after `xero/pull-2/ManualJournals.json`, exactly as in `samples/bank/manifest.json`. The source
`bank:ridgeway-4821` is already declared. In `samples/README.md`, the bank row's files become
`bank/activity-2026-03-20.csv`, `bank/activity-2026-04-02.csv` + `bank/ridgeway-csv-mapping.json`,
`bank/2026-03.ofx`.
