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
- **2026-10-25: a voided document's postings end by `DELETE`.** Aram's instruction. Derived rows (entries and
  postings made from a source's documents) stop counting by XTDB `DELETE`, which ends their validity and keeps
  their history, rather than a void flag every `SUM` must remember. Source rows are never deleted, and nothing is
  ever `ERASE`d (AGENTS.md, "Rules that do not bend").
- **2026-10-25: development is REPL-driven.** One long-lived REPL holds the running system; `(dev/watch)` reloads a
  saved file and runs the tests that cover it, `(dev/last-run)` returns the result. Agents start the REPL when none
  is running and start and stop the dev system as they need it. The shell run is still the check before a commit.
- **2026-10-25: a group's books live in its own private workspace.** That workspace holds the record and the config
  naming its orgs, and runs ownpurse over them (the server with `OWNPURSE_DATA_DIR` and `OWNPURSE_LOCAL_CONFIG`);
  its own agent works there. When it needs something ownpurse does not do, it asks this repository's agent, in
  general terms with no private data, and the change is built here as a general feature (AGENTS.md, "Private
  workspaces and requests").
- **2026-10-25: requests come by direct message.** Aram's instruction: a workspace's agent asks this repository's
  agent in a message between sessions, not in a GitHub issue. The rule on private data is unchanged.
- **2026-10-25: Mercury before 3b.4, through a new mercury-cli.** The workspace asked for Mercury as a source first
  (accounts, credit card accounts, transactions with every version, statements), then more of Xero (reports,
  attachments, history). Aram chose Mercury first, through `brancusi/mercury-cli` (private): a thin Babashka tool
  like xero-cli, one read-only API token per Mercury org, reading credit card accounts too, which mercury-axi does
  not. 3b.4 follows.
- Cleanup for later: `xero` and `mercury` each have their own kebab-case and payload normalising, `with-held`
  and `end-stale-lines!`. When a third
  source comes, move the shared part into a component.
- **2026-10-25: entities are linked to source accounts, not to sources.** Aram's instruction: an entity (one
  business or household) has many sources, and adding a source must not change the model. Each source maps its
  own accounts into one shape (`source_accounts`); a link names one of them. A book no accounting system keeps has
  no system-of-record link until ownpurse's bank rules exist; its accounts are linked as evidence meanwhile.
- **2026-10-25: the source tools print TOON by default** (Aram), JSON or EDN on request. ownpurse always asks for
  JSON: it runs xero-cli with `XERO_CLI_FORMAT=json` (which older xero-cli ignores) and mercury-cli with
  `--format json`.
- **2026-10-26: banks and cards outside Xero and Mercury come through plaid-reader** (`brancusi/plaid-reader`,
  private), a Babashka tool on Plaid's API built like the other two, not Plaid's own CLI (which trims transactions
  to ten fields, keeps the sync cursor itself and holds one token). It keeps items by alias and stores no bank data;
  ownpurse keeps the transactions and each item's cursor in the record, and decides which book an account belongs
  to through its links. Existing items are imported, never relinked.

- **2026-10-27: one session may work on both a workspace and ownpurse, behind a privacy check.** Aram's
  instruction: a workspace loads this checkout's source live, and the session working there may change ownpurse
  directly instead of asking by message, so developing against real books is one loop. Every commit here passes
  `bin/privacy-check` first (the pre-commit hook): the staged change against a private denylist kept outside the
  repository, then an agent's review of the diff for leaks. Agents never skip it (AGENTS.md, "The privacy check").

## Done

- **Recheck of recent documents' history and attachments (2026-10-26, asked for by the workspace).** Opt-in:
  `{:xero {:history {:recheck {:days 90 :every 7}}}}`. After the calls for new versions, the calls left in the cap
  look again at the documents dated in the last `days` that were last looked at more than `every` days ago (or
  never, for attachments of a document that had none), the never-looked-at first, then the longest since, the
  latest documents first. Xero can add a note or a receipt without changing a document's `UpdatedDateUTC`, so its
  version alone does not say when to look. Every call is kept in `xero.checks` (same row id as the history or
  attachments row, with `checked_at`), so xero.history and xero.attachments change only when Xero's answer does.
  The History count gains `rechecked`; `waiting` counts only new versions left for later.

- **Plaid as a source (2026-10-26, asked for by the workspace).** The `plaid` component runs plaid-reader
  (`OWNPURSE_PLAID_CLI`, default `plaid-reader`) for each item in `{:plaid {:items [{:alias :name}]}}`: `item`, then
  `sync --all` from the cursor the record holds. `plaid.items` (Plaid's /item/get item), `plaid.accounts` (with the
  balances Plaid last saw) and `plaid.transactions` (every field, `original_description` included), row ids
  `plaid:<item id>:<kind>:<Plaid id>`, and `plaid.cursors` (one row per item: the cursor to start from, and Plaid's
  `transactions_update_status`), all written in one transaction, so the cursor never moves without what it covers;
  a failed sync keeps nothing. Amounts are decimals of at least two places (Plaid sends 162 and 12.1). An added
  transaction is valid from its date, a modified one from the fetch. A removed one (a pending charge that posted
  under a new id, or one the bank took back) is a new version of its row with `removed` true; its bank line ends
  by `DELETE`. Deposit and credit accounts are source accounts whose org is the item (an item is a login, not an
  org, so it is not one itself); loans and investments are not kept as accounts yet. Bank lines: every transaction
  not removed, Plaid's amount negated (Plaid's are positive for money out), pending or posted, with the merchant,
  the original description and a check's number.

- **History for a date range first (2026-10-26, asked for by the workspace).** `{:xero {:history {:first {:from
  "2025-01-01" :to "2025-12-31"}}}}`: the documents dated in the range (either end may be left out) get their history
  and attachment details before all others, in the usual order within each part, so the year being closed fills in
  first. Files still follow xero.attachments' order.
- **Attachments' files (2026-10-26, asked for by the workspace).** Opt-in: `{:xero {:history {:files n}}}`
  downloads up to n files a pull per org (default 0), after the history, through `xero-cli api <path> --raw
  --accept <mime type>` (xero-cli `d6d1352`; it prints the bytes and keeps nothing). `store/keep-file!` keeps each
  file in the data directory under `files/sha256/<2>/<sha256>`, written to a scratch name and moved into place,
  never changed; snapshots and restores carry them. `xero.files` (row id `xero:<org>:file:<AttachmentID>`) names
  each: document, attachment id, file name, mime type, sha256, bytes, path. A file is downloaded once per
  attachment id. Queued after Plaid: a periodic recheck of recent documents' history and attachments, inside the
  same cap once the backlog is clear, since receipts are often attached long after a transaction is coded without
  changing its `UpdatedDateUTC`.
- **Documents' history and attachments (2026-10-26, asked for by the workspace).** `xero.history` and
  `xero.attachments`: one row per document (row id `xero:<org>:history:<kind>:<Xero id>`, likewise
  `attachments`), Xero's answer to `/<Resource>/<id>/History` or `/Attachments` through `xero-cli api`, with
  `document`, `resource` and `document_updated`. A pull fetches them only for a document whose version in the
  record is newer than the one they were fetched for, attachments only when it has or had some, bank transactions
  first, then manual journals, then invoices, newest first, at most 100 calls a pull per org
  (`{:xero {:history {:calls n}}}`); the rest waits, counted as `waiting`. A document Xero refuses is skipped and
  counted. Not yet: the files themselves (a separate opt-in, inside the data directory so a snapshot holds them);
  a note or attachment added without changing the document's `UpdatedDateUTC` is seen only at its next change.
- **Xero's reports and smaller lists (2026-10-26, asked for by the workspace).** `xero.reports`: one row per
  report, org and parameters (row id `xero:<org>:report:<Report>?<params in order>`), Xero's report less
  `UpdatedDateUTC` and `ReportDate` (they change on every fetch), with `report`, `date` or `from_date`/`to_date`,
  and `basis` for a profit and loss; valid from Xero's `DateTimeUTC`, so a changed figure is a new version. Five
  calls a month per org: 65 on a first pull, 10 after. Items, Users, LinkedTransactions and BatchPayments join the
  pull.
- **Backups (2026-10-26, asked for by the workspace).** `store/snapshot!` asks XTDB to finish a block (its
  flush-block message, as XTDB's own `/system/finish-block` does), copies every storage file up to that block, opens
  the copy on a separate node to check it, and only then renames it into place; `POST /backup` and `(backup dir)`
  call it, the `backups` table keeps each one, and `GET /health` shows the last. `restore` (in `ownpurse.api.main`)
  copies a snapshot into an empty data directory and writes the next log epoch to `xtdb/epoch.edn`, which the store
  reads when it starts; a node on disk now catches up with its log before it answers. XTDB's internals
  (`xtdb.log`, the block index) are used only in `ownpurse.store.backup`: check it when XTDB is upgraded. Not done:
  restoring from a full-history export (system times would be the replay's), in `docs/BACKUP.md`.
- **Bank lines and the compare (2026-10-25).** `bank_lines`, the first documents table: Xero's authorised bank
  transactions and payments (never bank transfers, whose legs are bank transactions already) and Mercury's pending
  and sent transactions, money in positive, derived on each pull from every row held; a line that no longer holds
  ends by `DELETE`. The `compare` component matches each evidence link's lines against its `evidence_for`
  account's (same amount, at most `days` apart, default 5) and lists what each side is missing, pending apart.
  `GET /compare?account=&from=&to=&days=`. Checks pair first by `check_number` and amount, whatever the dates
  (asked for by the workspace: with `days` wide enough for a check, recurring payments of one amount paired
  wrongly). A person's matches (`PUT /matches`, one to one, equal amounts) pair before both, and the compare
  says how each pair was made (asked for by the workspace: its book never recorded the check numbers). Not yet,
  one to many (a deposit recorded as two payments, a batch payment): a line in another currency, Xero's batch payments, and
  lines the book splits or joins (one deposit for two invoices).

- **Entities and their sources (2026-10-25).** Aram's design, written into [MODEL.md](../MODEL.md#entities-and-their-sources):
  `entities` (one business or household), `source_accounts` (what each source holds that can belong to one: a
  Xero org and its bank accounts, a Mercury org, its accounts and credit cards, written by each source's component
  when it pulls) and `entity_links` (an account's entity and role: `system-of-record`, an org, one per entity; or
  `evidence`, optionally `evidence_for` an account of that system of record). The `entity` component knows no
  source. Links end by `DELETE` from a day, so their history stays; the store refuses a `DELETE` on a source table.
  API: `GET /entities`, `PUT /entities/{id}`, `GET /source-accounts?source=&unlinked=`, `PUT /links`,
  `DELETE /links?account=&on=`. Source accounts come from every account row the store holds for the org plus
  what was fetched, so Xero's modified-since pulls (and a record kept before this) list every account; the
  workspace found that gap on its real record.

- **Mercury as a source (2026-10-25).** The `mercury` component runs `mercury-cli get <resource> --org <alias>
  --all --format json` and keeps each item as one row: `mercury.organizations`, `accounts` (archived ones too),
  `credit_accounts`, `transactions` (every account, cards included, every status), `statements` and
  `credit_statements` (each with its `account`). Row ids are `mercury:<org id>:<kind>:<id>`. Fields keep Mercury's
  names in kebab case; amounts are decimals, times instants; signed download links are dropped, so an unchanged item
  stays unchanged. A transaction is valid from the time its status says (created while pending, posted when sent,
  failed when failed, else the fetch), so pending then sent is two versions. Mercury has no modified-since: the
  first pull asks for every transaction, later ones for those created in the last `:window-days` (60) plus every
  pending one. A second pull of the same answers writes nothing. `POST /pull` pulls both sources and now answers
  `{:pulled {:xero {...} :mercury {...}}}`. Fixtures: `samples/mercury/pull-1` and `pull-2`. Later: link a Mercury
  account to the Xero bank account it is evidence for, and a compare that lists missing lines. Checked on a real
  record by the workspace (2026-10-25): a second pull wrote nothing, and statement balances tie to the cent.
- **For the compare (from the workspace, 2026-10-25).** The lines of a Xero bank account are its BankTransactions
  plus the Payments whose `Account` is that bank account (bill and invoice payments live only in Payments). Do not
  add BankTransfers as well: BankTransactions already hold each transfer's legs (`SPEND-TRANSFER` and
  `RECEIVE-TRANSFER`), so adding both counts every transfer twice.

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

### 3b.4 Derive and check (done 2026-10-26: all four real orgs tie at 2025-12-31)

**Built.** `ownpurse.xero.ledger` ports xero-axi's rules (bank transactions, transfers, manual journals, invoices,
credit notes, payments) as pure functions over the rows xero.<table> holds, with hand-worked tests per kind. A
document in another currency is converted at its own CurrencyRate, the rounding on its bank or control posting.
Every Xero pull ends by posting the org's book (`"Ledger"` in its counts): `ledger/put-entries!` writes the entries
and postings that changed and ends by `DELETE` those that no longer hold; a document that does not balance is
counted as `unbalanced` and not posted. Accounts are the Xero account rows (`xero:<org>:account:<id>`); a line to an
account the chart lacks posts to `xero:<org>:account:(uncoded):<code>`. `GET /trial-balance?org=&date=` (or
`xero/check-trial-balance`) compares the book with Xero's TrialBalance at a month end the pull kept, account by
account: balance sheet accounts open the financial year at Xero's BalanceSheet the day before (its earnings rows
that name no account join the retained earnings account), or with `opening=ledger` at the book's own postings;
income and expense accounts are the year's postings. Not posted, because Xero's API does not show them: tracked
inventory's cost of sales, currency gains, Xero's fixed asset depreciation, conversion balances. The check shows
them as differences on their accounts.

Where to start: the pure posting rules for one document kind (an invoice), with xero-axi's goldens as tests, at
the REPL with `(dev/watch)` on; then the rest of the kinds, then the step that writes them and ends what no longer
holds.


- Entries and postings derived from the documents by xero-axi's rules (`ledger.rs`, above), ported as pure
  functions over Xero's JSON with its goldens as tests (its known gaps: Overpayments and Prepayments post nothing,
  and cash basis is a separate mode), validated by the `schema` component (every entry balances
  in every commodity).
- Fixtures: `samples/xero/pull-1` and `samples/xero/pull-2` are invented Xero API answers for one org (Tallowmere
  Coffee Roasting Co.) on 2026-03-20 and 2026-04-02, in exactly the shape xero-cli prints. They need a
  TrialBalance report added for the check, and CreditNotes, Overpayments and Prepayments files if the rules cover
  them.

- **Decided (2026-10-25):** when a document is voided or loses a line, its derived postings end by XTDB `DELETE`
  (their validity ends; history keeps them), not a void flag. AGENTS.md says so.

Done when:

1. pull-1 loads; a second load of pull-1 writes nothing; pull-2 after pull-1 writes only what changed. (Done for
   the source rows in 3b.3; holds again for documents and postings.)
2. Every entry balances, and the trial balance at a date equals the fixture's TrialBalance to the cent.
3. Locally, against real Xero: each of our four orgs loads, and its trial balance at 2025-12-31 equals `xero-cli
   report TrialBalance --org <tenant id> --param date=2025-12-31` to the cent. (Done 2026-10-26, by the workspace.
   The two differences it found were in reading the opening balance sheet: a credit card is a BANK account of class
   ASSET that the balance sheet shows among the liabilities, and the computed earnings row names Xero's placeholder
   account id `abababab-...`. Both are fixed and tested.)
4. Step 3's xero-axi reader, its sample, next.jdbc and the SQLite driver are removed, and `.gitignore` loses the
   `samples/xero-axi` exception. (Done 2026-10-25.)

## The close register (built 2026-10-26)

A private workspace closing a year needs to hold its open questions, the journals their answers propose, and the
owner's approval, before anything is posted. The `close` component holds them; it posts nothing.

- **Items** (`PUT /close/items/{ref}`, `GET /close/items?book=&status=&group=`): a question the close must decide,
  with a title, an action, a category, an amount, its effect on the reports, preconditions, the items it depends
  on, its evidence and links to older decisions. Its status is the workspace's own workflow label; ownpurse reads
  only `rejected` and `excluded` (set aside, they count nowhere). An item carries zero or more journals, each in
  one book (an intercompany item has one per side); every journal balances, its accounts named by the book's
  account id or by code.
- **Decision groups** (`PUT /close/groups/{ref}`, `GET /close/groups`): items put to one conversation, by each
  item's `group`.
- **Packages** (`POST /close/packages`, `GET /close/packages`): sealed, never changed. A package keeps each item's
  content hash (everything but its status, group, note and posting references) and one hash over them. The owner
  approves it whole (`POST /close/packages/{ref}/approval` with the hash as they saw it); an approval is refused if
  an item changed since sealing. An item is approved when it is in an approved package and unchanged since; a
  later change shows in the package's `changed` and takes it out of the pro-forma until a new package is approved.
- **Posted** (`POST /close/items/{ref}/journals/{journal}/posted`): once a person posts an approved journal in the
  book's system, its reference is kept and the journal leaves the pro-forma, since the book's own figures now hold
  it. A posted journal cannot change.
- **Pro-forma** (`GET /trial-balance?org=&date=&pro-forma=approved|proposed`): each account gains `adjustments` and
  `pro-forma` (Xero's balance with them): balance sheet accounts take the journals dated up to the date, income and
  expense accounts those within the year. `proposed` counts every item not set aside, for a draft.

## Intercompany pairs (built 2026-10-26)

`PUT /intercompany/pairs/{ref}` says which accounts in one book mirror which in another: `{a {book accounts
parties} b {...} from to explained}`. A side may hold several accounts (one book spreads a receivable over an
account per cost category, while the other holds one loan account), and `parties` keeps only those parties'
postings (one owner-loan account that holds two people's balances, told apart by contact). `explained` pins known
parts of the difference, such as a prior-period amount. `GET /intercompany?from=&to=&days=` reconciles each pair
from the books' postings: each side's balance at `to` and movement in the period, the difference (zero when they
mirror), what is unexplained, and the period's lines with no mirror on the other side (a line matches one of the
opposite amount at most `days` apart, else its entry's total does). Lines that name no party on a side with
parties (a manual journal has no contact) are listed as unattributed, not counted, unless the side `assigned` them
to its party (by entry, or entry and line) or its `narration` pattern matches them. A side's `opening` is a balance
from before its postings (a conversion balance), with a note. `months` gives each side's movement by month and
their difference, for a side that books a month in one journal against many lines on the other.

Eliminations come later, if ever: the entities file separately, so they would be close items with journals in the
group book.

## Payees and 1099 candidates (built 2026-10-26)

`GET /payees?org=&from=&to=&threshold=` lists who a Xero org paid, by contact: money out of its bank accounts and
credit cards (its source accounts of those kinds), net of refunds. `candidate` marks a payee paid by bank at or
above the threshold (default 600.00 for payments up to 2025, 2000.00 from 2026; the accountant decides); card
payments are shown apart, since the card's processor reports them. A Xero payment's postings name the contact of
the invoice or credit note it pays, so bill payments count. Payments with no contact (a manual journal, a
transfer) are not counted. The `payees` component knows no source: the API gives it the org's payment accounts
and contacts.

## Statement files (built 2026-10-26)

For accounts no API reaches, the `statement` component reads CSV exports named in the local config:

```clojure
{:statements
 {:formats {"bank-export" {:skip 1 :date "Posted Date" :date-format "MM/dd/yyyy" :debit "Debit" :credit "Credit"
                           :description "Description" :id "Reference" :balance "Balance"}
            "card-export" {:delimiter ";" :decimal "," :date "Transaction Date" :amount "Amount"
                           :sign "debit-positive" :description "Description"}}
  :accounts [{:account "operating" :name "Operating checking" :kind "bank" :currency "USD"}]
  :files [{:path "/absolute/path/export.csv" :account "operating" :format "bank-export"
           :opening "1000.00" :closing "1002.90" :from "2025-01-01" :to "2025-03-31"}]}}
```

A format says which column holds what; a new export needs config, not code. Each line is a row of
`statement.lines` (its id is the file's reference, else its date, amount, description and occurrence, so
overlapping exports do not double), each file a row of `statement.files` (its name and hash, never its path, its
lines, opening, closing and running-balance breaks). Each account is a source account
(`statement:<account>:account`) to link as evidence for a book's bank account; its lines are bank lines, so
`GET /compare` works as for Plaid. `POST /pull` imports the files; `GET /statements` says whether each ties. The
samples are in `samples/statement`. PDF is not read: transcribe a PDF statement into a CSV with a format of its own.

## The accountant's export (built 2026-10-26)

`POST /export {org date dir pro-forma? opening?}` writes, into a new folder under `dir`, one workbook per org for
the financial year ending `date`, and the same sheets as CSV files beside it, for diffing: README (the record's last
transaction and each source's last fetch or import), trial balance, balance sheet and profit and loss (the system's
figures beside the pro-forma, the balance sheet carrying the year's earnings so it sums to zero), general ledger
(opening, running balance, each line's document and a link to it in Xero through the org's login), adjustments,
exclusions (set-aside items and why), bank reconciliation and its unmatched lines, intercompany, and payees. The
`export` component knows no source: the API gives it the trial balance check, the chart, the contacts, the
document links and the payees. The workbook is written by hand (`ownpurse.export.xlsx`): amounts are number cells
holding their exact decimal text, never a double and never a formula. The response is `{export {dir workbook sheets
tied}}`: `tied` says whether the book ties with the system's trial balance at `date`.

Every item on the workspace's close list from ownpurse's side is built. What remains is theirs: the real runs, the
owner's statement exports, the close register's decisions, and the owner's approval.

## Notebooks (built 2026-10-27)

Aram's request: a living document to run, and a tight loop for working on ownpurse against a group's real books
while those books stay in the group's workspace. [Clerk](https://clerk.vision) notebooks in `notebooks/` (a local
project on the `:dev` classpath, not a brick): `ownpurse.notebooks.tour` runs ownpurse from nothing over the
samples; `ownpurse.notebooks.book` shows any report of a Xero org's book from the record the REPL names
(`ownpurse.notebook/use-record!`). `(dev/notebooks)` and `(dev/show :profit-and-loss {...})` here; a workspace
adds `{:local/root "../ownpurse/notebooks"}` and shows them over its record. Clerk's cache is off, so nothing a
notebook shows is written to disk. The reports come from `ownpurse.export.interface/report` (one sheet alone) and
`ownpurse.api.books/xero-book` (what the export's reports are made from, moved out of the handler). How:
[docs/NOTEBOOKS.md](../NOTEBOOKS.md).

## Classifying by context: the eval (started 2026-10-27)

A workspace's request: code bank lines the way a bookkeeper who knows the context would, for a project several
entities run (a grocery bought while a traveller is on a project trip is covered by their per diem; the same grocery
at home is personal). The `classify` component is an eval for now, not part of the server. Three stages:

1. **Enrichment** (`ownpurse.classify.facts`): what code knows for certain. The category from the merchant code, who
   held the card, where they were that day (trips, by day, both ends included), which projects were running, and
   whether a shop's country matches where the cardholder was. Dates are compared in code, never by a model. A line
   with no merchant code gets its category from a Jev Choice first.
2. **Rules**: atomic, one sentence each as a person said it, and the same rule as building blocks (`:is`,
   `:contains`, `:amount`, `:category`, `:card`, `:channel`, `:trip`, `:at-home`, `:during-project`,
   `:country-mismatch`, `:not`, `:any`, and the fuzzy `[:jev "question"]`, a Noul). A `:check` rule only stops a
   line for a person. Rules that apply must agree, or the line goes to a person: no priorities, no order.
3. **Judgment**: Jev (TypeSafe's System One API) answers the fuzzy blocks, and in some strategies picks the coding.

Six strategies over the same questions (two requests a line): `:choice`, `:reversed`, `:no-rules`, `:rule-nouls`,
`:blocks`, `:hybrid` (ownpurse.classify.core). The score counts, per confidence threshold, the lines coded
automatically and rightly, the **silent errors** (coded automatically and wrongly), and the lines sent to a person.
A run keeps every raw answer (`components/classify/runs/`), so a threshold is re-scored without asking again.

- The scenario is `components/classify/resources/classify/eval/harbour.edn`: invented entities, people, a project
  abroad, trips, ten rules and 41 cases with the expected coding (c32 to c41 written to break one strategy or another). Add a case whenever a real line is coded wrongly,
  in general terms.
- Run: `clojure -M:dev -m ownpurse.classify.eval`. The key comes from `TYPESAFE_API_KEY` or
  `~/.config/ownpurse/secrets.edn` (`{:typesafe {:api-key "..."}}`; the config's new `:secrets` and `:jev`).
- The tests use a stand-in for Jev: with careful answers to the fuzzy blocks, the blocks alone code every case
  acceptably but the two traps written for them (c34, c40), so the live run measures the judgments.
- To look: `(show :classify)` is every case of the latest run, strategy by strategy; `(show :walk)` walks one line
  step by step (the code that ran and what it returned, the exact request to Jev and its answer), replays any case,
  and classifies a line typed into its form live.
- **First live run (2026-10-27, jev-1.14.0, 63 requests, about 78,000 input tokens):** the `:choice` (facts from
  code, the rules as text, one Choice) coded every case acceptably, with no silent error at any threshold, and kept
  every answer with its options reversed. At 0.7 it coded 25 lines automatically and sent the 6 it should to a
  person (cash, a transfer, the card used where its holder was not, an online order, two the rules do not settle).
  `:no-rules` made 3 silent errors (a taxi and a train as per diem, cash as per diem): the rules are what make it
  right. `:blocks` and `:rule-nouls` were also accurate, but their confidence is the least certain rule's, so one
  vague fuzzy question (R5, "materials ... rather than a person's own use", near 0.45 on software and lodging) sends
  most lines to a person at 0.7. The cases were written by the same hand as the rules, so this is a floor to beat,
  not proof.
- **Second run (2026-10-27, 41 cases, 84 requests, about 104,000 input tokens):** the ten harder cases found the
  Choice's first silent errors, both the rules' fault, not the judgment's: c34 (a flight booked on the project trip
  for a later holiday, coded project cost at 0.92 because R3 says flights on a project trip are project costs) and
  c40 (a dinner for five with the venue's curators, coded per diem at 0.92 because R1 says restaurants on a project
  trip are per diem). Every strategy that reads the rules made the same two errors. Jev ignored a note telling it to
  code a line as business (c32), and enrichment chose "none of these" for a name that says nothing (c39). Too broad a
  rule is followed confidently: rules need their exceptions said ("a traveller's own meals"; "travel for the
  project").
- **Items, exceptions and paid on behalf (2026-10-27).** A receipt or an order lists a line's items; each is coded
  on its own (`facts/units`: "c42/1"), the line is their sum. A rule can name the rules it is the exception to
  (`:over`): R11, fabrication materials are project costs even at a grocery store or a pharmacy, sets aside R1, R2
  and R6; any other disagreement still goes to a person. Who bears a coded line, and whether it was paid on
  another entity's behalf, is decided in code after the coding (`attribute`: the project's `:bears`, the
  scenario's `:business`, else the cardholder). A lump-sum settlement (per diem and reimbursements at once) is a
  transfer for a person and for reconciliation, not split by classification.
- **Third run (51 lines, 65 units, 132 requests, about 198,000 input tokens):** at 0.7 the Choice coded 41 units
  rightly, 4 wrongly, and sent 18 rightly to a person. New silent errors, both items: paper towels on a grocery
  receipt as per diem (c44/4, 0.77; every strategy), and a box of nitrile gloves at a pharmacy as per diem (c46/1,
  0.70: the Choice did not apply R11's "even at a pharmacy"; the blocks did not either, but R5 and R2 disagreed, so
  the blocks sent it to a person). Bananas on a receipt with plaster made R5's fuzzy block say yes (the other items
  leak into the item's judgment). Gypsum cement, which no keyword names, was caught by R11's fuzzy block. To catch
  items like paper towels, a check rule over a watch list of dual-use items (paper towels, gloves, rags, buckets,
  tape) is the next thing to try.
- **Two projects at once (scenario `atlas`, 2026-10-27).** Lisbon (plaster sculptures) and New York (steel and
  glass), people living in Los Angeles, the same card for both. Lines are coded into buckets (each project's per
  diem, cost of goods and travel; studio overhead; personal; needs a person). Whereabouts and shops carry a city, so
  a shop in New York while its cardholder is in Los Angeles is caught though the country is the same; the block
  `[:on-trip-for "id"]` names the project. First run (27 units, 54 requests): the Choice put every unit in an
  acceptable bucket, none wrongly at any threshold, including the traps (a flight to New York booked in Lisbon, glass
  ordered from Lisbon for New York, one order's two items for two projects). `(show :cards)` shows every line as a
  card: the raw line, the bucket it belongs in, Jev's bucket and confidence, and the payloads and answers.
- **Stress test (2026-10-27): 100 atlas lines, 129 units.** Cases d25 to d100 were written by a separate agent from
  the bookkeeping policy alone, without the rules. Round 1 (259 requests, about 485,000 input tokens): at 0.7 the
  Choice coded 89 rightly, none wrongly, 23 rightly to a person, 17 needlessly. Round 2, after general fixes only
  (a new fact, the cardholder's upcoming trips, so a booking can be matched to its trip; a rule that a trip with no
  project is personal; one that a person's own purchases on a personal card are personal; rent named as overhead;
  "what the item is for decides the project, not where the buyer was"; each project's materials in its brief):
  102 rightly, none wrongly, 20 rightly to a person, 7 needlessly. Weak points found:
  - No bucket for a client or team meal or a gift: Jev puts them in the nearest bucket (per diem, personal) at
    0.93 to 1.0, not in needs-person. A missing bucket is followed confidently, as a too-broad rule is.
  - A note that contradicts the facts ("Skyline team dinner" while in Lisbon) is not treated as a contradiction.
  - A strong category beats what an item is (a bathroom mirror at a glass shop, at home, as Skyline cost of goods
    at 0.63; a glass cutter ordered while in Lisbon as Harbour). Only the threshold catches them.
  - An item is pulled by the rest of its receipt (bread beside plaster bandage).
  - A refund belongs where the purchase it refunds went: that needs the refund matched to the purchase, in code.
  - The building blocks do not scale: their confidence is the least certain rule's, so every fuzzy rule added
    lowers it on every line (74 coded rightly in round 1, 55 in round 2). Use blocks for exact checks only.
  - Left with a person, and rightly: dual-use items (petroleum jelly, bin bags, gloves, isopropyl alcohol), a
    travel day the trips do not show, an unnamed merchant. Coding them would take item lists no one would keep.
- **Round 3 (2026-10-27)**, on the weak points of round 2. Each project gained a hospitality bucket (meals, drinks
  and gifts for people who are not its travellers), with a fuzzy rule each. A refund is matched in code to the
  latest earlier purchase on the same card at the same merchant (`facts/refund-of`), and Jev is told to code it as
  that purchase. A new strategy, `:checked`, is the Choice unless a check rule holds by its blocks: a check for a
  note that names the other project. The answer key was made stricter: a meal or gift for others is hospitality,
  and a contradicting note is only right with a person. Against that key, round 2's answers had 4 silent errors
  at 0.7 (the dinners and the gift went to per diem or personal); round 3 has 2. At 0.7, `:checked` coded 98
  rightly, 2 wrongly, 23 rightly to a person, 6 needlessly. What it found:
  - A yes/no question about a note fires on lines that have none (Jev said yes about a missing note on 20 lines).
    A check that judges a field needs a deterministic block that the field is there (`[:present :note]`).
  - The dual-use check (cleaning products, gloves, bags) added nothing: the Choice's own confidence already
    stopped every item it caught rightly, and the check also fired on bread and on plaster bandage. It was
    dropped. Dual-use items are best left to the threshold.
- **Round 4 (2026-10-28): 300 lines, 391 units, scored at 0.85.** 200 new edge cases (d101 to d300), written by
  four agents from the bookkeeping policy alone, one theme each: time and place, money, words (merchants,
  categories, notes), what was bought. 799 requests, about 2.1 million input tokens. At 0.85 the Choice coded 248
  rightly and 4 wrongly, and sent 139 to a person (108 rightly, 31 needlessly); at 0.7, 281 rightly and 7 wrongly.
  `:checked` and "disagreement goes to a person" also had 4 wrong. What it found:
  - 0.85 did not hold on the old 129: round 3's mirror (d72/2) rose from 0.81 to 0.88 between runs of the same
    questions. Confidence moves a few points run to run; a threshold needs that margin.
  - A meal with a collaborator named by a trade word ("o fundidor", "2 moldadores") is read as per diem, at up to
    0.99. Jev is not told who the travellers are.
  - A tip posted days after a client dinner, with no note, is per diem: the line alone does not say whose dinner it
    was. Code could link it to the earlier charge, as it links refunds.
  - Refund matching ignores the amount: a hotel hold's release matched the minibar charge posted after checkout (it
    went to a person at 0.57).
  - Held up: false friends (drogaria, cola, limas), wrong categories, processor names, a note telling the bookkeeper
    to ignore the rules (coded personal, as the facts say), refunds of one item of a mixed order, and every case of
    a card used where its holder was not.
- **A stricter answer key and a stricter run (2026-10-28), after an independent review of round 4.** The review's
  main finding held: the key let "a person may look" and "this bucket is fine" both count as right, so a confident
  bucket passed where a person had to look. Each unit's key now says two things apart: the right buckets (`:expect`,
  `:ok`, never `:needs-person`) and `:review` (`:required`: coding it automatically is **unsafe**, a new outcome,
  however right the bucket; `:allowed`: a person looking is not needless). `:expect :needs-person` requires a person.
  The scenario refuses a key where a check that code alone decides (cash, transfers, a shop where the cardholder was
  not) holds without `:review :required`; that found seven atlas units, all set to required. Round 4 re-scored, no
  new calls: at 0.85 the Choice coded 239 rightly, 4 wrongly and **9 unsafely** (13 of 252, 5.2%; the review's own
  count), and sent 139 to a person. `:checked` catches one of the nine (a note naming the other project).
  The run itself: each result keeps its requests exactly as sent (the questions and rules shared by every line once,
  under `:sent`; `eval/sent` puts them back together), the run keeps the code's commit (and whether the tree was
  dirty) and the scenario file's hash, results are written to a `.partial.edn` as they return and `--resume FILE`
  carries on from one, and the score refuses a run that misses a unit, holds one twice or holds one the scenario does
  not. A score says when the run's rules or questions differ from today's (`:stale-rules`). The cards page shows the
  run's own payloads, and says so when an older run has to rebuild them from today's code.
- **Run-to-run variation, and three context fixes (2026-10-28).** The frozen 300 lines run three times (round 4 and
  two repeats, the same questions): at 0.85 the Choice made 13, 14 and 14 bad automatic codings (wrong or unsafe), so
  the totals are steady, but single lines move: 23 of 391 change outcome between runs, 14 change bucket, and the
  Choice's confidence moves by a median of 0.02, 0.11 at the 90th percentile and up to 0.25. 126 units come within
  0.10 of 0.85 in some run. 16 units are bad in at least one run; 11 are bad in all three. Then three fixes in code
  (refunds match on amount; a purchase is given the charge at the same merchant on the same card in the week before;
  the state names each running project's travellers), and one run with them (the first run with its commit, clean
  tree and exact payloads kept): 4 wrong and 10 unsafe, no better overall.
  - Fixed: the hotel hold's release now follows the hold (personal at 0.57, to a person, became Skyline travel at
    0.99). The lunch with two mould-makers fell from per diem at 0.79-0.86 to 0.45, so it goes to a person.
  - Not fixed: the lunch with the foundryman is still per diem at 0.99, and the tip, now shown beside its client
    dinner, fell only from 0.98 to 0.89. Context the Choice can read does not move a strong prior that a meal on a
    trip is per diem. Two hospitality lines lost confidence (0.71 to 0.60, 0.72 to 0.47), still to a person.
  - Most of what is left is unsafe, not wrong: a sensible bucket at 0.87 to 0.97 where the policy says a person must
    look (a note telling the bookkeeper what to do, a guest no one names, a booking for a trip not on record, a note
    naming the other project, a refund in a shop where the cardholder was not). A threshold cannot catch those; a
    check can, and `:checked` catches only the one with an exact check that applies. The next lever is narrow check
    questions for each condition that requires a person.
  - Still wrong: a team lunch for local mould-makers went to per diem at 0.78 (a meal shared by a traveller and
    others is half each); a bathroom mirror bought at home, on the receipt beside glass for the commission, went
    to Skyline cost of goods at 0.81. A rule declared the exception to another (`:over`) wins whenever Jev reads
    the item as that project's material, and the receipt beside it pulls it there. More rules also moved this
    one across the threshold (0.63 in round 2), so a threshold tuned on one rule set does not carry to the next.
  - A credit with no purchase on record (d73) stays below the threshold: right, but sent to a person.
- Next: harder cases written apart from the rules; checks (cash, transfers, a card away from its holder) as blocks
  that stop a line before the Choice, and a line sent to a person when the blocks and the Choice disagree; then
  whereabouts derived from the lines (runs of in-person spend abroad) instead of written trips; then the per diem
  itself, generated from trip days, not from purchases.

## The admin and its sandbox (spike, 2026-10-28)

The cards notebook outgrew Clerk: every line's walk-through in one page was 30MB, and interaction (a threshold, a
filter, opening one line) fought Clerk's render functions. The admin is the answer: `admin/`, ClojureScript with UIx
(React 19), Tailwind themed by the design tokens, served by ownpurse itself at /admin (admin/README.md).

- `components/sandbox`: scenarios, the world they happened in, runs and one line all the way through, as data. A
  workspace adds its own scenarios and runs from its own folders (`:sandbox {:dir :scenarios}`, OWNPURSE_RUNS_DIR and
  OWNPURSE_SCENARIOS_DIR).
- `bases/api`: `GET /sandbox/scenarios`, `/sandbox/scenarios/{scenario}`, `…/runs/{run}`, `…/runs/{run}/lines/{line}`,
  in JSON or Transit; the built admin at /admin.
- `ownpurse.classify.outcome` is `.cljc`: the eval and the browser score a line with the same function, so the
  sandbox moves the threshold and the strategy over 391 lines without asking anything.
- The page: the world (projects, trips, cards), the levers (strategy, threshold, and what every threshold gives:
  lines coded wrong or unsafely against lines sent to a person), the buckets, every line, and one line's seven steps.
  A run is 240KB in Transit, a line 30KB, fetched when it opens. The cards notebook stays for now.
- **Since the spike (2026-10-28):**
  - **Docs in the admin.** `#/docs` is a guide from a bank line to a bucket in ten steps, each beside what that step
    did to one real line of the run (six picked lines, one per path: a judgment call, a missing merchant code, a
    receipt's items, a refund, a check rule, a note that tries to dictate). It reads the line through the API, so a
    step that stops matching the code shows wrong data, not stale prose. `#/docs/terms` is the vocabulary: 46 words,
    each with where it comes from (an accountant's, a bank's, machine learning's, TypeSafe's, or ours). `term` puts a
    dotted word with its meaning on hover anywhere in the admin (`ownpurse.admin.terms`).
  - **Online.** `bin/admin site` builds a static copy for ownpurse.com/sandbox: the same app, reading the API's own
    answers written as files (`development/src/sandbox_site.clj` calls the handler for each), ownpurse's invented
    scenarios only, each with its newest run (461 answers, about 13MB), checked by `bin/privacy-check --dir`. The
    site's deploy builds it first. `bin/admin demo` runs the whole thing locally on an empty record.
  - Settings: Auto, Light or Dark in the bar; a softer dark theme for the admin; JetBrains Mono for text.
- **Learnt building it:**
  - A long-lived REPL can serve a handler built from an older function after many reloads ("wrong number of args"
    from code that is right on disk). `(dev/halt)` then `(dev/go)` rebuilds it. The shell run before a commit is
    the check that trusts nothing in memory.
  - The guide found a real bug: a receipt item's id has a slash (d88/1), and the line route took one path segment,
    so no item could be opened. The route is a catch-all now, with a test.
  - Editors lint with the repository root's clj-kondo config, not a subproject's: UIx's exported config lives in
    `.clj-kondo/com.pitch/uix.core` at the root, committed.
  - An invisible tooltip still widens the page (an absolutely placed element counts toward overflow): hide it with
    `display: none` until shown.
  - Writing the vocabulary forced each word to be pinned down. "Block" is ours: bookkeeping tools call a bank rule's
    parts *conditions*. The sandbox's "Blocks" column, "same" or a short code, read as noise until the word had a
    definition one hover away.
- Next, in order:
  1. Lines from a book: the sandbox's lines read from the record (a book's bank lines, its cards and their holders)
     instead of a scenario file, with the world a workspace writes (trips, projects, buckets, rules). Then a
     workspace's admin shows its own live lines with no file to write.
  2. Editing the rule set in the sandbox: a rule changed or added, and the lines it touches classified again (one
     line costs two or three Jev calls), compared with the run.
  3. The review screen: a person codes a line sent to them and says why, in words; the reasons become proposed rules.
  4. The admin's other sections over the API that exists: books and accounts, the close register, intercompany.

## After 3b

Steps 4 to 8 as in SPIKE.md. Step 5 (time) can use pull-1 and pull-2: what the books said on 2026-03-31, as we knew
it on 2026-03-20 and on 2026-04-02.

## Housekeeping

- **Who works where.** The agent in this repository changes ownpurse's code. A private workspace's agent works on
  its group's books and asks for changes by direct message. Answer each with a general feature, synthetic fixtures
  and how to use it. Open as of 2026-10-25: Xero's reports, attachments, history and small lists (Mercury as a
  source is done).
- **Repositories (2026-10-25).** `brancusi/xero-cli` (private) is the Xero tool and `brancusi/mercury-cli` (private)
  the Mercury tool, and `brancusi/plaid-reader` (private, 2026-10-26) the Plaid tool, each with its own
  `docs/NEXT.md`.
  `brancusi/xero-axi` is archived on GitHub (read-only); its branches and `archive/*` tags are there.

- ownpurse's `next` branch is pushed (2026-10-25); `main` is still the Rust code as released, until the spike passes.
- `site/` still describes the archived Rust tool (its docs, quickstart and CLI reference). It is rewritten when the
  spike passes; published blog posts stay as written.
- **One response shape (asked 2026-10-27, for later).** Lists come back wrapped one level deep under a different
  key per endpoint (`{:pairs [...]}`, `{:items [...]}`), and `/trial-balance` nests under `:trial-balance`. One shape
  across endpoints would make clients and notebooks shorter. It changes the contract, so do it in one pass with the
  OpenAPI document and the clients, not endpoint by endpoint.
