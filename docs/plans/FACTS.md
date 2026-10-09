# The fact model

Status: plan, agreed in conversation on 2026-10-09. Nothing here is built yet. It replaces the "Facts in the record"
item in [ROADMAP.md](../ROADMAP.md) with a concrete design, and it is checked against
[PHILOSOPHY.md](../PHILOSOPHY.md). The public write-ups are the blog posts "Where your money lives" and "The fact
model" (`site/src/content/blog/`).

## Summary

ownpurse decomposes every source into one kind of thing, a **fact**, stamped by a **transaction** that says who
wrote it, when, why and from what evidence. Raw input is kept byte for byte in a content-addressed **drawer**.
**Projectors** turn drawer files into facts in one shared vocabulary. The books (entries and postings) are facts too,
minted by ownpurse, held until posted, and evidenced by what the sources said. Every view is a query; indexes and
running totals are caches rebuilt from the log.

The log is ours, in SQLite. Any query engine (our own indexes, XTDB, Datomic, Datalevin) is a projection of the log,
so choosing one is never a migration.

The long-term goal is that ownpurse becomes the **book of record**. It starts as a faithful mirror of Xero,
QuickBooks and the rest; when a person is comfortable, the book's authority flips to ownpurse and the providers
become targets kept in sync, or are dropped.

## Decisions

| # | Decision | Why |
|---|---|---|
| D1 | The unit is a fact `[e a v tx op]`; provenance lives on the transaction | One shape for every source, correction, label, link and posting decision; provenance written once per batch |
| D2 | Three strata: evidence (drawer), claims (facts), derivations (indexes, postings computed by rule, reports) | Mappings can be fixed and replayed; nothing derived is authoritative |
| D3 | Postings are the shared vocabulary every source can reach, not the root of truth | Bank lines, budgets and wallet transfers are not postings until someone decides what they mean |
| D4 | Never merge entities across sources; relate them with `Link` facts that carry an author and a confidence | Matching can be wrong, so it must be reviewable and reversible |
| D5 | The log is SQLite: `tx` and `fact` tables, append-only by trigger, SHA-256 chained per transaction | Single binary, instant open, the file people hold; the chain and triggers already exist |
| D6 | Indexes (EAVT, AEVT, AVET, VAET) and running totals live in a separate, deletable cache | Speed without trusting a second copy of the truth |
| D7 | Any heavier engine is a projection of the log, adopted only if the benchmark shows SQLite plus indexes is not enough | Keeps one runtime unless numbers say otherwise |
| D8 | Every connector writes raw bytes to the drawer; a projector writes facts. Xero is one connector among many | Reprocessing never costs an API call; every source has evidence in the same form |
| D9 | The drawer defaults to `~/.local/share/ownpurse/<profile>/drawer/`, unencrypted; `drawer` config key and `$OWNPURSE_DRAWER` move it | Beside the record's existing default; encryption can come later as an option |
| D10 | Entries are held by default; posting rules in config may post when their conditions hold | Agents propose, people post, unless the person writes a rule that says otherwise |
| D11 | Regions are as exact as the format allows for free: row, element or JSON pointer for structured files; the page always, and a box when the extractor has one, for PDFs and images | Every observation can be shown in place; a finer reading later is a retract and an assert |
| D12 | One observation per item (a line, a receipt, a check), with shared attributes plus typed, namespaced extras | One query finds every witness of an event across sources; a check stays one thing |
| D13 | Amounts are `(quantity, commodity)` from day one | Crypto, FX and holdings need it; retrofitting is painful |
| D14 | Transaction time is native (log order); business time is attributes (`:entry/date`, periods, lock dates) | Accounting already has its own valid time; per-fact bitemporality is more than we need |
| D15 | `:book/authority` names who owns a book's entries: a source (mirror) or `ownpurse` (book of record) | The path from mirror to book of record is one fact |
| D16 | A manifest is a speculative transaction (`db.with(proposed)`); CHECK is queries over it | Today's `before_fp`/`after_fp` semantics carry over unchanged |
| D17 | `redact` removes a drawer file's bytes and records why; the hash stays in the chain | The one exception to "never delete": bytes, never facts |

## The five layers

```
1 Drawer        File                     bytes as they arrived, named by SHA-256
2 Documents     Document, Region         what a file is; a place inside it
3 Observations  Observation, Balance     one item a document reports, pinned to its region
4 Links         Link, Label, Note        how things relate; dimensions; notes
5 Books         Book, Account, Entry,    your double-entry books, in any commodity
                Posting, Party, Commodity, Price
  Transaction   stamps every fact in every layer
```

Each layer refers only to the layers before it, except links, labels and notes, which may refer to anything.

## Vocabulary

Attributes are namespaced. The schema is itself facts (`:db/type`, `:db/card`, `:db/unique`, `:db/doc`), so an
agent can propose a new attribute and a person can review it like any other transaction.

Value types: `text`, `decimal`, `amount` (decimal + commodity ref), `date`, `instant`, `ref`, `keyword`, `bool`,
`hash`, `locator`, `tuple`.

### Transaction

| Attribute | Type | Meaning |
|---|---|---|
| `:tx/at` | instant | When it was written |
| `:tx/actor` | text | `human`, `agent:<name>`, `rule:<id>`, `connector:<id>` |
| `:tx/reason` | text | One line: "Daily card download", "Posted it" |
| `:tx/evidence` | ref → File | The file this batch was projected from, if any |
| `:tx/projector` | text | `<connector>@<version>` that produced the facts |
| `:tx/prev-hash`, `:tx/hash` | hash | The chain |

### Drawer and documents

| Attribute | Type | Card | Meaning |
|---|---|---|---|
| `:file/sha256` | hash | one, unique identity | The bytes' hash; the file's identity |
| `:file/media-type`, `:file/name`, `:file/size` | text, text, decimal | one | As received |
| `:file/captured-at` | instant | one | When the bytes were captured |
| `:file/redacted` | text | one | Why the bytes were removed (D17) |
| `:doc/kind` | keyword | one | `:statement`, `:card-activity`, `:receipt`, `:check`, `:invoice`, `:xero.bank-transaction`, `:sheet`… |
| `:doc/files` | ref → File | many | A check's front and back; a statement's PDF and OFX |
| `:doc/issuer` | ref → Party | one | The bank, the merchant, the provider |
| `:doc/period` | tuple(date, date) | one | A statement's period |
| `:doc/external-id` | text | one | Statement number, check number, provider id |
| `:doc/source` | text | one | The connector and account it came from |
| `:region/doc` | ref → Document | one | |
| `:region/page` | decimal | one | Required for PDFs and images |
| `:region/box` | tuple(x, y, w, h) | one | When the extractor returns one |
| `:region/row`, `:region/pointer` | decimal, text | one | CSV/sheet row; OFX element path or JSON pointer |

### Observations

| Attribute | Type | Meaning |
|---|---|---|
| `:obs/region` | ref → Region | Where the item is |
| `:obs/kind` | keyword | `:bank-line`, `:card-line`, `:receipt`, `:check`, `:invoice`, `:sheet-row`, `:chain-transfer`… |
| `:obs/date`, `:obs/amount`, `:obs/party`, `:obs/number`, `:obs/text` | shared | Everything that witnesses money uses these |
| `:obs/confidence` | decimal | For extracted readings (never `f64`) |
| `:obs/located` | tuple(attr, locator), many | Where a field sits: `[:check/signer box(…)]` |
| `:obs/fingerprint` | hash | Account, date, amount, normalised text, same-day ordinal; for link rules, not identity |
| `:<kind>/<field>` | typed | Fields only one kind has: `:check/signer`, `:check/memo`, `:receipt/tip`, `:xero.invoice/branding-theme` |
| `:balance/account`, `:balance/date`, `:balance/amount`, `:balance/region` | | A balance a source states: statement closing balance, Xero TB line, on-chain balance |

Source-specific attributes are kept in full, because writing back to a provider needs its own fields (tracking
category ids, line ids, branding themes). Reports read only the shared vocabulary.

### Links, labels, notes

| Attribute | Meaning |
|---|---|
| `:link/kind` | `:same-event`, `:evidences`, `:settles`, `:mirrors`, `:splits`, `:transfer-pair` |
| `:link/a`, `:link/b` | Any two entities |
| `:link/confidence` | Exact decimal |
| `:link/status` | `:proposed`, `:accepted`, `:rejected` |
| `:label/on`, `:label/type`, `:label/value` | The classify store's labels, as facts |
| `:note/on`, `:note/text` | Notes on anything |

### Books

| Attribute | Meaning |
|---|---|
| `:book/name`, `:book/base` (→ Commodity), `:book/fiscal-start`, `:book/lock-date` | A set of accounts |
| `:book/kind` | `:actual` or `:plan` (a budget is a plan book with the same accounts) |
| `:book/authority` | A connector (mirror) or `ownpurse` (book of record) (D15) |
| `:account/book`, `:account/type`, `:account/code`, `:account/name`, `:account/parent` | Asset, liability, equity, income, expense |
| `:entry/book`, `:entry/date`, `:entry/narration`, `:entry/status` | `:held`, `:posted`, `:void` |
| `:posting/entry`, `:posting/account`, `:posting/quantity`, `:posting/commodity`, `:posting/cost`, `:posting/party`, `:posting/memo` | One line of an entry; an entry's postings sum to zero per commodity |
| `:party/name`, `:party/identifiers` | Who |
| `:commodity/code`, `:commodity/precision` | USD, NZD, BTC, a share class |
| `:price/of`, `:price/in`, `:price/value`, `:price/date` | A price, from a source |

## Identity

| Thing | Identity |
|---|---|
| File | `sha256(bytes)` |
| Document | Source and external id when given; otherwise its first file's hash |
| Region | Document and locator |
| Observation | Region and kind |
| Bank line across sources | Not an identity: a fingerprint that link rules match on |
| Entry, posting, account, book | Minted by ownpurse (UUIDv7); external ids are ordinary facts |

## Verbs

| Verb | Who | Writes |
|---|---|---|
| observe | connectors, projectors | Files, documents, regions, observations, balances |
| assert / retract | people, agents, rules | Anything in the vocabulary; a retraction is the only "undo" |
| link | rules, agents, people | Links with a confidence |
| post | people, or posting rules (D10) | `:entry/status` from `:held` to `:posted` |
| propose | agents, people | A speculative transaction (a manifest); nothing enters the log until approved |
| redact | people | Drawer bytes removed, reason recorded (D17) |

## Connectors and projectors

A connector fetches and stores bytes; it never writes facts directly.

1. **Fetch** within the source's limits (Xero's call budget, a bank's download page, a wallet's RPC). Each response
   body or file goes into the drawer. A file already there is not stored twice.
2. **Project**: a versioned projector reads a drawer file and emits one transaction of facts with `:tx/evidence` and
   `:tx/projector` set. Snapshot sources (Xero documents, sheet reads) are diffed against the facts already projected
   from earlier files, so only changes are asserted or retracted.
3. **Replay**: a projector fix re-runs over the drawer. No API calls.

| Source | Identity of an item | Notes |
|---|---|---|
| Xero, QuickBooks, FreshBooks | Provider ids | Whole-document snapshots; in a mirror book the projector also emits entries linked with `:mirrors` |
| OFX / QFX | FITID | Banks sometimes reissue FITIDs; the fingerprint link rule catches it |
| CSV statements, activity downloads | Fingerprint | Running balance is a free integrity check |
| Statement PDFs | Account and period | Opening and closing balances become `Balance` facts; lines carry page and box |
| Spreadsheets | Key columns from the mapping; content fingerprint otherwise | An agent proposes the mapping (key, date, amount, what the rows are); the mapping is versioned and stored. A formula's computed value is the fact |
| Receipts, check images | File hash | Extracted by an agent; confidence and located fields recorded |
| Crypto wallets | `chain:txhash:log_index` | Fees, many commodities, prices from a price source; cost basis is a posting rule's policy |

Xero is "just payloads we've pulled". ownpurse holds them and reprocesses them locally, which keeps calls to the
minimum. Before release, read the current Xero developer terms for retention and agent use; holding data locally
does not by itself change what the terms allow.

## Bank statements and evidence

One movement of money is witnessed up to four times: the daily download, OFX, the monthly statement and the
provider's bank feed. Each stays its own observation; link rules join them with `:same-event`; an entry is
evidenced by any of them. A witness without a partner is a question for the person, never a silent merge.

Balances make every statement check itself, with the same rule as `parity`:

- opening balance + lines = closing balance (per statement);
- the ledger's balance on the closing date = the statement's closing balance;
- each source's running balance, line by line.

"Show me what backs this" is a walk: posting → entry → `:evidences` link → observation → `:same-event` links →
regions → documents → files, ending at the page and box. The TUI opens the file cropped to the region.

## Storage

```
~/.local/share/ownpurse/<profile>.sqlite        the log (D5)
  tx    (tx_id, at, actor, reason, evidence, projector, prev_hash, hash)
  fact  (e, a, v_type, v, tx, op)                append-only by trigger
~/.local/share/ownpurse/<profile>/drawer/        the drawer (D9)
  sha256/ab/cdef…                                bytes, never edited
~/.local/share/ownpurse/<profile>.index          the cache (D6): indexes and running totals, deletable
```

- Decimals are stored as canonical decimal text and compared in Rust; floats are refused at the type level.
- The chain links transactions, and each transaction's hash covers its facts in canonical order and the hashes of
  the files it cites, so `ownpurse verify` proves the drawer too.
- Running totals: `(book, account, commodity, period) → sum`, updated per transaction; as-of a past transaction is a
  replay from the nearest checkpoint.

## Queries

- **Pull**: fetch the exact shape a screen needs from an entity (an entry with its postings, evidence and files).
- **Rules compiled into Rust** (Ascent or hand-written) for postings derived from mirrored documents, balance
  checks and link rules.
- **Datalog for agents**, over the cached indexes, once the pull and rule layers exist.
- **As-of**: every query takes an optional transaction bound.

## Migration from today's record

| Today | Becomes |
|---|---|
| `versions.payload` (Xero JSON) | Drawer files, replayed through the Xero projector |
| `reports` snapshots | Drawer files projected into `Balance` facts |
| `sync_runs`, `http_log` | Kept as operational tables; each sync's transactions cite its run |
| `db/classify.sqlite` events | Label, note, link and undo transactions in the log |
| `db/manifests.sqlite` | Speculative transactions; approved ones enter the log |
| `operations`, `op_events`, `packages`, `approvals`, `verifications` | Kept; a landed write's read-back arrives as drawer evidence |
| `chain` | The transaction chain; the old chain's head hash is recorded in the first new transaction |

Goldens must hold to the cent through the migration: `tb`, `gl`, `parity`, `ic` and `compare` on the demo fixture
give byte-identical output before and after.

## Phases

1. **Benchmark** (decides D7). Write the ten queries below against (a) SQLite plus a `redb` index cache and running
   totals and (b) XTDB v2 over the Postgres wire, with each query in SQL and XTQL; add Datomic Local if a pure
   Datalog comparison is wanted. Scale the synthetic record to 10M facts (`xtask demo big`). Measure cold open,
   p50/p99 per query, memory, and how hard each query was to write.
2. **The log and the drawer**: `tx`, `fact`, the schema as facts, triggers, the chain, the drawer with `redact`
   and `verify`.
3. **Xero as a projector**: replay today's payloads into facts; goldens unchanged.
4. **Indexes, running totals, pull**; the TUI reads through them.
5. **Classify and manifests on the log**; manifests as speculative transactions.
6. **Bank connector** (CSV, OFX, statement PDF) with balances, the four-witness link rules and the backing trace.
7. **Receipts and checks**: agent extraction with regions and confidence.
8. **Authority**: native books, posting rules, the flip from mirror to book of record.

### The benchmark queries

| Query | Shape |
|---|---|
| TB, P&L, balance sheet by period | Sum postings by account and period |
| Account rollups | Account → parent, up to 5 levels |
| Drill into a figure | Account → postings → entry → evidence |
| Reconciliation | Bank line ↔ link ↔ entry, then balance checks |
| Settlement | Invoice → payments → bank lines |
| Provenance | Fact → transaction → evidence file |
| Intercompany | Posting in book A ↔ link ↔ posting in book B |
| Same-event clusters | Groups of linked observations |
| As-of | Any of the above at transaction N |
| Agent ad-hoc | 2–4 hops plus a sum, unknown in advance |

## Engines considered

| Engine | Verdict |
|---|---|
| Datomic (Local / Pro) | The model we want. JVM, closed source, no public client protocol; queries run clauses in written order and aggregate after realising results |
| XTDB v2 | Bitemporal, columnar, cost-based planner, SQL and XTQL (Datalog-style, `unify` with logic variables), Postgres wire from Rust. JVM; in-process use not yet stable; rows are document versions, not datoms |
| Datahike | Datomic-style, active, Clojure/JVM |
| Datalevin | Strongest Datalog optimizer found; keeps no history, so at most an index |
| Cozo | Right shape (Rust, Datalog, SQLite backend, as-of) but dormant since 2023–24; no decimal type |
| Minigraf | Rust, bitemporal Datalog; months old, one maintainer, slow at 1M facts |
| Oxigraph | Rust RDF/SPARQL, exact `xsd:decimal`; no time model |
| SurrealDB | Rust, decimal, versioned storage still beta; not Datalog |
| Kuzu / RyuGraph | Kuzu archived Oct 2025; no time model |
| DuckDB | Fast exact aggregates, `ASOF` joins; heavy C++ dependency, no triggers. A candidate for report-shaped caches |
| SQLite | Chosen for the log (D5) |

## Open questions

- The exact canonical encoding of a transaction for hashing (sorted facts, value encodings).
- Whether running totals checkpoint per period or per N transactions.
- Encryption at rest for the drawer, as an option.
- How a projector version change is rolled out: replay everything, or only new files with a flagged diff.
- Posting-rule syntax in config.

## Out of scope

- Multi-user write concurrency beyond one writer per record.
- Running any provider write path differently from today's sealed, approved packages.
- A hosted service. The record and the drawer stay on the person's machine or server.
