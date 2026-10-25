# Where the spike stands, and what comes next

The plan is [SPIKE.md](SPIKE.md). This file is the working state: decisions taken since the plan was written, what
is done, and the next steps in order with what "done" means. Update it when a step lands.

## Decisions

- **2026-10-24: the Xero tool is now xero-cli, and xero-axi is archived.** xero-axi kept a local record and its own
  ledger rules, so the same business logic lived in two places. xero-cli (a separate, private repository,
  `brancusi/xero-cli`) is a thin Babashka command line over the Xero API: it logs in, keeps tokens fresh, picks
  the org, keeps to Xero's limits, pages and batches, and prints Xero's JSON. It stores no Xero data.
  ownpurse now owns everything after the fetch: keeping Xero's JSON as facts in XTDB, and turning it into entries
  and postings.
- What does not change: ownpurse never calls Xero and never holds a Xero token. It runs xero-cli, which does.
  Xero knowledge stays in the `xero` component.
- **2026-10-24: postings come from Xero's documents, not its Journals.** Xero's `Journals` endpoint (its general
  ledger) is open only to apps on its Advanced developer tier: about A$1,445 a month, a security assessment and
  use-case approval ([LANDSCAPE-2026-10.md](../research/LANDSCAPE-2026-10.md)). Our app, like any app a person
  registers for their own books, is on Starter: 5 orgs, 1,000 calls a day per org, no Journals. So the `xero`
  component turns invoices, bills, credit notes, payments, bank transactions, transfers and manual journals into
  entries and postings itself, porting xero-axi's rules, and the ledger checks the result against Xero's
  `TrialBalance` report, which Starter does allow.
- **2026-10-25: two systems of record.** Each entity's books (one Xero organisation) have Xero as their system of
  record: it is what the accountant and the tax filings use, and ownpurse's copy must equal its trial balance to the
  cent. ownpurse is the **group system of record**: for the group as a whole (intercompany matches, eliminations,
  group reports), for matches across sources, for the history of what each source said and when, and for books no
  accounting system keeps (a household). A fix to an entity's books goes back to Xero as a sealed package a person
  approves; ownpurse never edits its copy of them. See [PHILOSOPHY.md](../PHILOSOPHY.md#the-group-system-of-record).
- **2026-10-25: the words.** A *book* is the record of one entity (one Xero organisation is one entity's book; the
  group book belongs to no single entity). Each book has exactly one *system of record*; every other source for it is
  *evidence*, used to reconcile, never posted. Mercury is not a book: its statements are evidence for a bank account
  in an entity's book, or the system of record of a book nothing else keeps.
- **2026-10-25: xero-axi is archived and nothing here uses it.** Its repository (`brancusi/xero-axi`, private) is
  archived on GitHub, with every branch pushed and tagged `archive/*`. Its posting rules, which 3b.4 ports, are in
  `rust/crates/xero-axi-core/src/ledger.rs` at the tag `archive/gl-full-contract`, with the goldens and property
  tests that pinned them.
- **2026-10-25: a group's books live in its own private workspace.** That workspace holds the record and the config
  naming its orgs, and runs ownpurse over them (the server with `OWNPURSE_DATA_DIR` and `OWNPURSE_LOCAL_CONFIG`);
  its own agent works there. When it needs something ownpurse does not do, it asks in a GitHub issue on this
  repository, in general terms with no private data, and the change is built here as a general feature (AGENTS.md,
  "Private workspaces and requests").

## Done

| Step | What | Commit |
|---|---|---|
| 0–2 | Archive and toolchain; the workspace; money and the schemas | see SPIKE.md |
| 3 | Reading xero-axi's record and gl into the ledger; a second load writes nothing | `15e4eb8` |
| xero-cli 1–5 | xero-cli built, tested against a fake Xero, and answering for our four orgs | xero-cli `78f64e1`…`9ee93fa` |
| 3b.1–3b.3 | The model and its names; fetching through xero-cli; Xero's answers kept as source rows; `POST /pull`; step 3's xero-axi reader and SQLite removed | `ac4d935` |

Step 3's xero-axi reader, its Brackenridge sample, next.jdbc, the SQLite driver and the archived Rust loader's SQL
(`samples/queries`) are gone (2026-10-25). The ledger's tests use a small inline fixture instead. The
`ledger` and `schema` components still have step 3's shapes (`documents`, `sources` with a record and chain head);
3b.4 replaces them with the model's documents and ledger tables.

## Next: step 3b, reading Xero through xero-cli

Replaces step 3's source. Steps 4 onward are unchanged and build on this.

### 3b.1 The model in XTDB (done)

Three layers, each derived from the one below and never written past it. The diagram and a worked example are in
[MODEL.md](../MODEL.md).

| Layer | Tables | Holds | Owned by |
|---|---|---|---|
| Sources | one schema per provider: `xero.invoices`, `xero.contacts`, `xero.bank_transactions`, … (later `mercury.transactions`, `qbo.invoices`) | each item exactly as the provider sent it | that provider's component |
| Documents | `public`: `parties`, `invoices`, `payments`, `bank_lines`, `accounts`, `links` | the same kind of thing from every source, with `book`, `source` and `source_row` | the provider's component maps into them |
| Ledger | `public`: `books`, `sources`, `entries`, `postings` | double entry, one system of record per book | the `ledger` component |

Why: per-provider tables alone make every client know every provider; one universal table alone loses what its
shape has no column for, and the original with it. Both, in separate layers, keep the evidence exact and give
clients one model. XTDB schemas work (checked 2026-10-25): `:xero/invoices` is the SQL table `xero.invoices`,
apart from `public.invoices`, and DataGrip shows the schema.

- **Source rows.** One row per item, `_id` `xero:<tenant>:<kind>:<Xero id>`, the org as `org`, Xero's fields at the
  top level in kebab case (`InvoiceNumber` is `invoice_number`), no nils, Xero's `/Date(...)/` as a date (an instant
  for fields ending in `UTC`), money exact.
- **Time.** A row is valid from Xero's `UpdatedDateUTC` (never later than the fetch); system time is when ownpurse
  wrote it. Resources without `UpdatedDateUTC` (tax rates, currencies, the org) are valid from the fetch.
- **Bills** are payable invoices; credit notes are invoices of kind `credit_note`.
- **Who posts.** A book whose system of record is an accounting system posts from its source rows by that provider's
  own rules, so its trial balance can match the provider's report. A book with none posts from `bank_lines` by
  shared bank rules. One system of record per book, so nothing is counted twice.

### 3b.2 Fetch: running xero-cli (done)

- `ownpurse.xero.cli` runs `xero-cli get <Resource> --org <tenant id> --all [--modified-since <instant>]` and
  parses stdout with ownpurse's mapper. Exit codes become `:xero/usage`, `:xero/login-needed`,
  `:xero/daily-limit`, `:xero/refused`, `:xero/failed` (and `:xero/not-found` when the program is missing), with
  xero-cli's JSON error, hint included, as data.
- The command is `:xero :cli` in `config.edn` (`OWNPURSE_XERO_CLI`). Which orgs to read is in `config.local.edn`
  (ignored): `{:xero {:orgs [{:tenant "<tenant id>" :name "..."}]}}`. `(dev/pull-xero)` pulls them all.
- After the first fetch, a resource with `UpdatedDateUTC` asks only for what changed since the latest one held.
- Tests run a stand-in (`components/xero/test/ownpurse/xero/fake-xero-cli.sh`), never xero-cli.

### 3b.3 Keep: Xero's answers as source rows (done)

- `xero/keep!` writes one transaction per fetch, with only the rows that changed (`store/put-changed!`, which now
  takes schema-qualified tables and ignores a row's valid-from when comparing). Its metadata names the tool and its
  version, the org, the resource, when Xero answered and the `--modified-since` asked.
- Tested on the samples: pull-1 loads (every item of every resource); a second load writes nothing and makes no
  transaction; pull-2 writes exactly the 11 items that are new or changed. An edited invoice keeps both versions; a
  voided one is a new version with status `VOIDED`, not a deleted row.
- **Against real Xero (2026-10-25):** all four orgs of our group pulled in one go, about 9,800 items in 27 seconds;
  a second pull asked only for changes and wrote nothing. Both pulls together cost under 40 calls per org.
- **A private record lives outside this repository.** The server pulls through `POST /pull`: every org in the config
  at `OWNPURSE_LOCAL_CONFIG`, one pull at a time (409 if one is running; 502 with xero-cli's error type when xero-cli
  fails). At the REPL, `(dev/open-record dir sql-port)` and `(dev/pull-xero {:node n :local file})` do the same.
  Our group's config and record are in its own private workspace, which runs the server itself.
- **Open:** an item Xero stops returning (a deleted bank transfer, a deleted draft) is not seen by
  `--modified-since`. A full fetch can find ids that are gone; what to record then is for 3b.4.

### 3b.4 Derive and check (next)

Where to start: the decision below first (ask Aram); the pure posting rules and their goldens can be written
before it, since only the step that ends a voided document's postings depends on it.


- Entries and postings derived from the documents by xero-axi's rules (`ledger.rs`, above), ported as pure
  functions over Xero's JSON with its goldens as tests (its known gaps: Overpayments and Prepayments post nothing,
  and cash basis is a separate mode), validated by the `schema` component (every entry balances
  in every commodity).
- Fixtures: `samples/xero/pull-1` and `samples/xero/pull-2` are invented Xero API answers for one org (Tallowmere
  Coffee Roasting Co.) on 2026-03-20 and 2026-04-02, in exactly the shape xero-cli prints. They need a
  TrialBalance report added for the check, and CreditNotes, Overpayments and Prepayments files if the rules cover
  them.

- **To decide with Aram before writing it:** when a document is voided or loses a line, its postings must stop
  counting. The proposal is XTDB `DELETE` of the derived postings (their validity ends; history keeps them), not a
  void flag every `SUM` must remember. AGENTS.md and the house style forbid `DELETE` without a person's explicit
  instruction, so this needs that instruction and a sentence in AGENTS.md saying derived rows may end this way.

Done when:

1. pull-1 loads; a second load of pull-1 writes nothing; pull-2 after pull-1 writes only what changed. (Done for
   the source rows in 3b.3; holds again for documents and postings.)
2. Every entry balances, and the trial balance at a date equals the fixture's TrialBalance to the cent.
3. Locally, against real Xero: each of our four orgs loads, and its trial balance at 2025-12-31 equals `xero-cli
   report TrialBalance --org <tenant id> --param date=2025-12-31` to the cent.
4. Step 3's xero-axi reader, its sample, next.jdbc and the SQLite driver are removed, and `.gitignore` loses the
   `samples/xero-axi` exception. (Done 2026-10-25.)

## After 3b

Steps 4 to 8 as in SPIKE.md. Step 5 (time) can use pull-1 and pull-2: what the books said on 2026-03-31, as we knew
it on 2026-03-20 and on 2026-04-02.

## Housekeeping

- **Who works where.** The agent in this repository changes ownpurse's code. A private workspace's agent works on
  its group's books and asks for changes as issues on `brancusi/ownpurse` (none open as of 2026-10-25). Answer each
  with a general feature, synthetic fixtures and how to use it, then close it.
- **Repositories (2026-10-25).** `brancusi/xero-cli` (private) is the Xero tool, with its own `docs/NEXT.md`.
  `brancusi/xero-axi` is archived on GitHub (read-only); its branches and `archive/*` tags are there.

- ownpurse's `next` branch is pushed (2026-10-25); `main` is still the Rust code as released, until the spike passes.
- `site/` still describes the archived Rust tool (its docs, quickstart and CLI reference). It is rewritten when the
  spike passes; published blog posts stay as written.
