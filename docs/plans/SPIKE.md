# The spike: a ledger in Clojure on XTDB

Status: plan for review, 2026-10-23. Nothing is built until this is agreed.

ownpurse starts again from zero: a local ledger with a clean API, in Clojure, on XTDB. Clients (the TUI, a web
interface, agents) are separate projects that use the API. The Rust code is archived for reference. The spike
answers one question with real numbers: is XTDB the right store for our books? If it is, the spike becomes the
base we build on.

## What carries over, what doesn't

| Carries over | Archived |
|---|---|
| [PHILOSOPHY.md](../PHILOSOPHY.md), [SECURITY.md](../../SECURITY.md), `brand/`, `site/` (as is, for now) | `rust/`, `vendor/`, `caretline.rev`, `install.sh`, `.githooks/`, the Rust CI |
| The ideas: books, sources, evidence, links, laws, sealed writes, money never a float | The fact-model crates, the datom log, the TUI |
| The synthetic data as test input (xero-axi's demo record, `samples/`) | `docs/plans/*` (moved to `docs/archive/`) |

xero-axi stays the Xero tool, in Rust, in its own repository. The new ledger reads its record. Each source keeps
its own tool; only the ledger is rewritten.

## The archive

1. Commit what is uncommitted on `facts` (the plans and research), then tag `archive/rust-main` (main) and
   `archive/rust-facts` (facts) and push the tags. The two blog demo tags stay as they are.
2. A new branch `next` from `main`. One commit removes everything in the right column above and moves the old
   plans to `docs/archive/`. AGENTS.md, README.md and CHANGELOG.md are rewritten for the new project.
3. `main` moves to `next` only when the spike passes. Until then `main` is the Rust code as released.

Nothing is deleted from history: `git checkout archive/rust-facts` brings everything back.

## The stack

| Layer | Choice |
|---|---|
| Runtime | JDK 21 or later (Temurin), Clojure 1.12, installed with mise and pinned in `mise.toml` |
| Workspace | Polylith (`poly` through a `deps.edn` alias): components, bases, projects |
| State | donut.system: config, the XTDB node, the HTTP server |
| Store | XTDB 2.x in-process, with its log and storage on local disk; its read-only Postgres port for SQL clients |
| HTTP | Reitit, Ring on Jetty, Malli for coercion and contracts, OpenAPI generated from the Malli schemas |
| JSON | jsonista, reading every number as `BigDecimal` |
| Config | Aero (`config.edn`, profiles, environment variables) |
| Reading xero-axi | next.jdbc and the SQLite driver, read-only |
| Tests | Kaocha, `test.check` for the laws, the synthetic records as fixtures |
| Quality | clj-kondo (with a rule against float literals and `double` in money code), cljfmt, `poly check` |
| CLI (after the spike) | Babashka, as a thin client of the API |

Clojure practice we hold to: pure functions in components and side effects at the edges; data in, data out;
Malli schemas at every boundary; namespaced keywords; `ex-info` with a `:type` for every error; no global state
outside donut.system; the REPL as the main way of working, with `user.clj` starting and stopping the system.

## The workspace

```
ownpurse/
  workspace.edn  deps.edn  mise.toml  config.edn
  components/
    money/          BigDecimal amounts and commodities; rounding; no floats anywhere
    schema/         Malli schemas for every entity the ledger holds
    store/          the XTDB node, transactions with provenance, queries
    book/           books, accounts, parties
    ledger/         entries and postings; trial balance, general ledger, as-of
    source-xero/    reads a xero-axi record (read-only) and states what it holds as ledger data
    evidence/       the drawer (raw files by hash) and the hash chain over our transactions
  bases/
    api/            Reitit routes, Malli contracts, OpenAPI, errors as data
  projects/
    server/         the uberjar: donut.system with the store and the API
  development/      user.clj, the REPL system, scratch
```

One rule from the old project stays: Xero knowledge lives only in `source-xero`. Everything else speaks books,
accounts, entries and postings.

## The model in XTDB

XTDB stores documents in tables with no fixed schema, each with two times: valid time (when it was true in the
world) and system time (when we recorded it).

| Table | Holds | Notes |
|---|---|---|
| `sources` | One row per source (a xero-axi record and the organisation in it) | Tool, version, record path, chain head when read |
| `books` | One per organisation | Currency, fiscal year end, the source it comes from |
| `accounts` | Per book | Code, name, type, class. Renames are new versions |
| `parties` | Contacts | Per book; linking the same person across books is a `links` row, never a merge |
| `documents` | Every source document, with its payload as nested data | Source id, source version, content hash |
| `entries` | Journal entries | Book, date, narration, the document it comes from |
| `postings` | One per entry line | Entry, book, account, `DECIMAL` amount, commodity, date; separate rows so SQL can sum them |
| `links` | Intercompany pairs, matches | Author, confidence, reason |
| `provenance` | One row per transaction we submit | Who, why, from what source, our hash chain |

Budgets, projects, closes and reports come after the spike. They are new tables, not migrations.

The postings come from xero-axi's own ledger (its `gl` table), so the rules that turn Xero documents into
postings stay in the Xero tool. The ledger checks them: every entry balances, and the trial balance matches
Xero's own report.

## The steps

| # | Step | Done when |
|---|---|---|
| 0 | Archive and toolchain: tags, branch `next`, the clean tree, JDK and Clojure through mise | `clojure -M:poly info` runs on a clean clone |
| 1 | The workspace: Polylith, donut.system, XTDB in-process, `/health`, Kaocha, clj-kondo, cljfmt | The system starts and stops from the REPL; tests and lint pass |
| 2 | Money and the schemas: the `money` and `schema` components; properties for "an entry balances" and "no float reaches the store" | Property tests pass; a float is refused at the boundary with a clear error |
| 3 | Reading xero-axi: books, accounts, parties, documents and postings from a record, with provenance | The synthetic record loads; a second load changes nothing |
| 4 | The checks: trial balance per book at a date, in SQL and in XTQL | Matches the synthetic goldens to the cent; then, locally, our four organisations at 2025-12-31 and 2026-09-30 |
| 5 | Time: what the books said on a date, against what we knew at a moment in the past | One query for each, and a test where a changed document shows both |
| 6 | The API: books, accounts, trial balance, general ledger, read-only SQL; OpenAPI | `curl` and a generated client both work; errors are structured |
| 7 | Durability: our hash chain over transactions, an export of every transaction to plain files, a restore | Restoring the export into an empty node gives the same trial balances |
| 8 | DataGrip | Connects to the read-only Postgres port and runs the trial balance query |

Done: 0, 1.

What step 1 found:

- XTDB opens listeners by default: the Postgres wire server and Flight SQL, each on a random port. `:flight-sql nil`
  in the config map does not turn Flight SQL off; `(.setFlightSql config nil)` does. The store turns both off unless
  the config asks for the SQL server.
- The SQL server is read-only: `:port -1` turns the read-write port off. An `INSERT` over the read-only port is
  refused (`xtdb/dml-in-read-only-server`), so every write goes through ownpurse. Checked with pgjdbc: `1.10` comes
  back as `1.10`.
- A `DECIMAL(32,4)` keeps its scale through `xt/q`; transaction metadata lands in `xt.txs.user_metadata`.
- `poly test` runs tests in its own classloaders, where XTDB does not start. Kaocha in a plain JVM is the test run
  for now; Polylith's external test runner is the way back to `poly test`.

## The decision at the end

The spike passes if steps 4, 5 and 7 hold, and if querying the real books feels right to you in DataGrip and the
REPL. Then we build on it: the Babashka CLI, budgets, the close, reports, a Docker image, Mercury through
mercury-axi. If it fails, we know why, and the archive is one checkout away.

## Packaging (after the spike)

Two ways to install, both wrapping the same uberjar:

- **A native installer with its own Java runtime** (`jpackage`), and a Homebrew formula: `brew install ownpurse`,
  then `ownpurse start`. This is the default for people on a laptop.
- **One OCI image** on GitHub's container registry, with the data folder as a volume. Docker, OrbStack, Podman and
  Apple's `container` all run it, so the runtime is the user's choice.

The Babashka CLI is a native binary that finds the server, or starts it. GraalVM native-image is ruled out for
the server (XTDB's Arrow storage).

What the spike keeps true so that packaging stays easy: one process (XTDB in-process); one data folder for the
XTDB log and storage, the drawer and the exports, set by config or environment; every setting from Aero config
with environment overrides; 127.0.0.1 by default; `/health` and a clean shutdown through donut.system; the uberjar
built with tools.build.

## What the spike must answer

- Can we load history in the order it happened? Each xero-axi version has the time we observed it. If XTDB lets a
  transaction carry an earlier system time, history keeps its real times; if not, the observed time becomes a
  field.
- Valid time for a document: the document's own date, or always open? The test in step 5 decides.
- Can our provenance ride on XTDB's transaction metadata, or does it need its own table?
- Exact sums: a `DECIMAL` sum in SQL, at our sizes (about 11,000 postings for the largest book), to the cent.
- How long does a cold start take, in-process, on a laptop?

## Private data

The repository stays public. Tests use only the synthetic records. The real record is read from a path in a
local, uncommitted config; its data never enters the repository, a fixture or a commit message.
