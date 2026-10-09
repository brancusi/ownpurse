# Rolling out the fact model

Status: plan, 2026-10-09. How [FACTS.md](FACTS.md) gets built: the milestones, the crate boundaries, the
practices we hold the code to, and the tests that say each milestone is done. The first target is Xero: by the end
of milestone 5, every Xero book ownpurse mirrors lives in the log as facts, and we can prove that representation is
sound.

## Summary

- **Xero first, engine later.** The log format is ours whatever engine wins (D5, D7), so the benchmark that picks a
  query engine moves after the Xero cutover. Xero needs the log, the drawer, a pure projector and the ledger built
  from facts. None of that waits on a query engine.
- **Strangler, not big bang.** The new path runs beside today's `versions` path until both give byte-identical
  goldens. Only then does the CLI read from facts, and only after that does sync stop writing `versions`.
- **Functional core, imperative shell, enforced by crates.** Pure crates hold the model, the projector and the
  derivations; they cannot open a file, a socket or a clock. I/O lives in two thin shells.
- **Soundness is a set of laws, each a test.** Round-trip, idempotence, minimal diff, replay determinism, ledger
  parity, as-of and the chain. A milestone is done when its laws pass on the fixtures and on the Demo Company.

## Milestones

| # | Milestone | Ships | Done when |
|---|---|---|---|
| M0 | Decisions and scaffolding | D18–D23 below agreed; empty crates wired into the gate | `check.sh` runs the new crates' (empty) tests, purity and size checks |
| M1 | The fact core (pure) | `Fact`, `Value`, schema as data, tx validation, canonical encoding and hash, the `Db` value with `as_of`, `with`, `datoms`, `entity`, `pull` | Model-based tests against the naive oracle; encoding test vectors; no surviving mutants in encoding, hashing or index order |
| M2 | The log and the drawer (shell) | SQLite `tx` and `fact` tables, triggers, chain, content-addressed drawer, `verify`, `redact` | UPDATE and DELETE refused; an independent verifier agrees; frozen head hash; a killed append leaves no half transaction |
| M3 | Xero schema and projector | Mapping tables for the 14 synced entities; a pure projector that diffs against the `Db`; replay of today's `versions` into drawer and log | Laws L1–L4 on every fixture version; the unmapped-field report is empty on the fixtures and the Demo Company |
| M4 | The ledger from facts | Mirror entries and postings as facts; `:book/authority` = `xero`; core's ledger reads postings, not JSON | L5 and L6: `tb`, `gl`, `parity`, `ic`, `compare` byte-identical from both paths, accrual and cash, every golden date |
| M5 | Cutover for Xero | Sync writes drawer files and facts; `versions` frozen; `ownpurse verify`, `ownpurse facts` (pull, history, as-of); the TUI reads the `Db` | Live Demo Company run: sync, project, parity with Xero's trial balance; release notes; the old path removed |
| M6 | Indexes and the engine benchmark | The index cache, running totals, the ten benchmark queries at 10M facts (FACTS phase 1 and 4) | Budgets met, or the benchmark names the engine to project into (D7) |
| M7 | Classify and manifests on the log | Labels, notes, links as transactions; manifests as `db.with` | Today's classify and manifest goldens unchanged |
| M8 | Bank evidence | CSV, OFX, statement PDFs, balances, four-witness links, the backing walk | Every statement checks itself; the dinner example walks to the page and box |
| M9 | Receipts and checks | Agent extraction with regions and confidence | Every extracted field has a region and a confidence |
| M10 | Authority | Native books, posting rules, the flip | A demo book flips from mirror to book of record with one fact, goldens unchanged |

M6 can start in parallel with M3 once M2 lands, because it only needs a synthetic log. M7 to M10 follow FACTS phases
5 to 8 and get their own detailed plans when M5 ships.

## Decisions this plan adds

Change any of these before M1 starts; after that they are expensive.

| # | Decision | Why |
|---|---|---|
| D18 | Entity ids are 64-bit integers allocated by the log in transaction order; external ids are unique-identity attributes (`[:xero/id "…"]` lookup refs). This replaces UUIDv7 for minted entities in FACTS.md | Replay is deterministic, indexes stay small and integer-ordered, and there is one writer per record, so global uniqueness buys nothing |
| D19 | Three new crates: `ownpurse-facts` (pure), `ownpurse-log` (SQLite and drawer), `ownpurse-xero-facts` (pure: the Xero schema, mapping tables and projector) | The purity boundary is checked by the compiler, not by review |
| D20 | The Xero projector emits accrual mirror entries and `:settles` links with amounts. Cash basis becomes a generic derivation in core over settlements | Moves every Xero rule out of `ownpurse-core` (today `ledger.rs` knows `SPEND-TRANSFER`), which is what AGENTS.md already asks for |
| D21 | Each Xero entity's mapping is a table of field → attribute → type → cardinality, read by one generic JSON walker. Code is written only for what a table cannot say | A new entity or field is one row; review reads data, not control flow |
| D22 | Fields a mapping does not cover are not dropped: the drawer keeps the bytes, and the projector records them as `:projector/unmapped` JSON pointers that a check reports | The mirror is lossless by construction, and schema drift at Xero shows up as a report, not a silent gap |
| D23 | Schemas only grow. An attribute is never renamed, retyped or removed; it is deprecated and a new one added. A gate compares the schema with the last release's frozen copy | Hickey's accretion rule: old logs, old projector versions and agent queries never break |

## Crate boundaries

```
ownpurse-facts       pure    Fact, Value, Attr, Schema, TxData, validate, encode, hash, Db, pull, oracle (test)
ownpurse-xero-facts  pure    Xero schema (data), mapping tables, project(db, file) -> TxData, mirror entries
ownpurse-core        pure*   ledger, balances, compare, intercompany over the Db; no Xero names in new code
ownpurse-log         shell   SQLite log, drawer, verify, redact, load a Db
ownpurse-xero        shell   auth, HTTP, fetch to the drawer, then project and append
ownpurse-cli / -tui  shell   unchanged roles; the TUI still never links ownpurse-xero
```

`*` core keeps its read-only record access during the migration. The pure crates depend on `rust_decimal`, `sha2`
and `serde` only; `ownpurse-xero-facts` adds `serde_json` with `arbitrary_precision`. A new `xtask purity` check
fails if a pure crate names `std::fs`, `std::net`, `std::process`, `SystemTime`, `Instant::now`, `rusqlite` or
`ureq`, or adds a dependency outside its allow list.

The core types, sketched:

```rust
pub struct Eid(u64);
pub struct TxId(u64);                 // also an Eid: a transaction is an entity with its own facts
pub struct Attr(u32);                 // interned from the schema; its name is a namespaced keyword
pub enum Value { Ref(Eid), Str(Arc<str>), Int(i64), Dec(Decimal), Bool(bool),
                 Date(Date), Instant(Instant), Kw(Kw), Hash([u8; 32]) }
pub struct Datom { e: Eid, a: Attr, v: Value, tx: TxId, added: bool }

impl Db {
    fn as_of(&self, t: TxId) -> Db;
    fn with(&self, tx: TxData) -> Result<(Db, TxReport), TxError>;  // speculative; nothing is written
    fn datoms(&self, index: Index, prefix: &[Component]) -> impl Iterator<Item = &Datom>;
    fn entity(&self, e: Eid) -> Entity<'_>;
    fn pull(&self, pattern: &Pattern, e: Eid) -> Pulled;
}
```

`Db` is a value: a shared, sorted base plus a small novelty overlay, the way Datomic pairs its persisted index with
its memory index. `with` copies only the overlay, so a manifest's speculative database costs what the manifest
touches. No persistent-collection dependency is needed at this size; M6 revisits it with numbers.

## How we write the code

The practices, and what each looks like in this codebase.

| Practice | In ownpurse |
|---|---|
| Values, not places | The `Db` never changes. Writing returns a new `Db`; reading an old one is `as_of`. Nothing in the pure crates takes `&mut` to shared state |
| Data over functions over macros | Schemas, Xero mappings, posting rules and checks are data that one interpreter runs. A `macro_rules!` needs a reason written beside it |
| Functional core, imperative shell | Fetch, append and render are the only effects. Projection, derivation, validation and queries are pure functions of their inputs, so every one is testable without a temp directory |
| Explicit time | No pure function reads a clock. The shell passes `observed_at` and the transaction instant in, as `store::pin_clock` already does for tests |
| Identity, state and value kept apart | An entity is an identity, a `Db` is the state at one transaction, a datom is a value. Code that mixes them (an "updated invoice" object) is the smell to look for in review |
| Small, closed vocabularies | `Value` is a closed enum; attributes are interned keywords from the schema; no `serde_json::Value` crosses out of a projector |
| Accretion | D23, enforced by the gate. Projector versions are added beside old ones, never edited in place once released |
| Errors as data | Validation returns every problem at once with the datom, the attribute and the rule it broke; the CLI renders them with the AXI error shape |
| Generative testing | Every schema type has a proptest generator, so tests can produce any valid transaction or Xero document, the way `test.check` and `spec` work together in Clojure |
| Explore against the real thing | `ownpurse facts pull`, `history` and `as-of` are the REPL: agents and people can inspect any entity at any transaction from the first milestone that has a log |

Code smell budgets for the new crates, checked by `xtask hygiene`:

- a file is at most 600 lines and a function at most 60, outside tests and data tables;
- no `unwrap` or `expect` outside tests and documented invariants;
- no stringly typed ids: `Eid`, `TxId`, `Attr`, `Kw` are newtypes, and a Xero id is a `Value::Str` behind its
  attribute, never a bare `String` passed between functions;
- `clippy::pedantic` on, with each allowed lint listed and explained in the crate root.

## Soundness: the laws

Each law is a property test over generated documents and a fixed test over every fixture version.

| Law | Statement |
|---|---|
| L1 Round-trip | For every Xero document `d`: `render(pull(project(d)))` equals `normalise(d)` minus the unmapped pointers. The facts can rebuild the document |
| L2 Idempotence | Projecting a file that has already been projected gives an empty transaction |
| L3 Minimal diff | Projecting version n+1 over a `Db` holding version n asserts and retracts exactly the fields that differ between them, and nothing else |
| L4 Replay determinism | Replaying the drawer into an empty log gives the same transactions and the same chain head, byte for byte |
| L5 Ledger parity | For every book, basis and golden date, postings from facts equal postings from today's `ledger.rs`, line by line, to the cent |
| L6 As-of | `ledger(db.as_of(t))` equals today's ledger built from only the versions known at `t` |
| L7 Balance | Every entry's postings sum to zero per commodity, or the entry is flagged, never hidden |
| L8 Chain | `verify` accepts every log the writer produced and rejects any single changed byte in a fact, a transaction or a cited drawer file |
| L9 Validation | No transaction that breaks the schema (type, cardinality, uniqueness, a dangling ref) reaches the log |

## Tests

| Kind | Where | Target |
|---|---|---|
| Unit and property | `ownpurse-facts`, `ownpurse-xero-facts` | Every public function; coverage ≥ 90 % lines in the pure crates |
| Model-based | `ownpurse-facts` | Random transaction sequences against a naive oracle (a `Vec<Datom>` and a filter): every index scan, `as_of`, `with` and `pull` agree |
| Laws | as above | L1–L9 on generated data and on every fixture version |
| Test vectors | `ownpurse-facts` | Frozen bytes and hashes for the canonical encoding of each value type and a sample transaction |
| Independent verifier | `ownpurse-log` tests | A second chain verifier written in the test from the definition, closing the gap in [TEST-GAPS.md](../TEST-GAPS.md) |
| Goldens | existing | `tb`, `gl`, `parity`, `ic`, `compare` and CLI goldens unchanged through M4 and M5 |
| Dual run | `check.sh` | From M4 until M5, every golden runs on both paths and the outputs are compared |
| Mutation | `cargo mutants` on the pure crates | No surviving mutants in encoding, hashing, index order, validation and the projector diff |
| Fuzz | `cargo fuzz` target for the projector | Any bytes give facts or a structured error, never a panic |
| Crash | `ownpurse-log` | A writer killed at every step of an append leaves the log verifiable and the drawer with at most an unreferenced file |
| Live | opt-in, `OWNPURSE_LIVE=1` | The Demo Company: sync, project, L1–L5 and parity with Xero's trial balance |
| Benches | criterion | Load a `Db` from the demo log, project one sync, rebuild the ledger; each with a budget that fails at 2× |

## The Xero milestone in detail

**M3 scope.** The 14 entities `sync` fetches today: Organisation, Accounts, TaxRates, TrackingCategories, Currencies,
BankTransfers, Contacts, BankTransactions, ManualJournals, Invoices, CreditNotes, Payments, Overpayments,
Prepayments, plus the report payloads that become `Balance` facts.

- Each document is an entity identified by `[:xero/id "<tenant>:<entity>:<id>"]`. Line items, journal lines,
  allocations and tracking options are component entities with `:xero.line/index` for their order.
- Dates become `Value::Date`, money becomes `Value::Dec`, enumerations such as `Status` become keywords
  (`:xero.invoice.status/authorised`), and references between documents (contact, account, invoice on a payment)
  become `Ref`s resolved through lookup refs. A reference to a document not yet seen is resolved when it arrives;
  until then it is reported, not invented.
- The migration replays `versions` in id order. Each row becomes a drawer file and one transaction, stamped with
  the row's `observed_at`, `sync_run_id` and source. The old chain's head hash goes into the first transaction.

**M4 scope.** The mirror rules in `ledger.rs` move into `ownpurse-xero-facts` as one function per document kind,
each emitting `Entry` and `Posting` facts linked to the document with `:mirrors`. They are written as data where the
rule is a mapping (journal lines) and as small pure functions where it is arithmetic (tax, apportioning). Core's
ledger becomes: pull the postings of a book, sum them. Cash basis (D20) is derived in core from `:settles` links.

**What we do not change during M3 to M5.** No new features on the `versions` path, only fixes. No new Xero
endpoints. The write path (packages, approvals, operations) keeps its tables; a landed write's read-back becomes a
drawer file and a transaction like any sync.

## Risks

| Risk | What we do |
|---|---|
| Cash-basis parity under D20 is harder than expected | Keep today's cash logic as a fallback derivation behind the same interface until L5 passes for cash on every date |
| Load time of a `Db` grows with history | M6 adds the persisted index cache; until then the Demo Company and fixtures set the budget |
| Xero adds fields | D22 reports them; adding a mapping row is an accretion, and a replay picks them up with no API calls |
| Canonical encoding chosen wrong | Test vectors are frozen at M1; changing the encoding afterwards is a new, versioned encoding, never an edit |
| The migration hides a gap | L1 on every historical version, and the dual run, before anything switches |

## Open questions

- Whether a transaction's facts are stored with the attribute's interned number or its keyword text (smaller versus
  self-describing; the hash covers the keyword either way).
- The persistent-collection question for `Db` at 10M facts (M6).
- Whether report payloads (Xero's trial balance) are `Balance` facts per account or one observation per report.
