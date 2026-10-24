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
- xero-axi is read for ideas, never built on. Its posting rules are in
  `rust/crates/xero-axi-core/src/ledger.rs` (xero-axi commit `430c2897`, branch `gl-full-contract`), with the
  goldens and property tests that pinned them. Both of its working branches are local only; push or tag them
  before relying on that commit from another machine.

## Done

| Step | What | Commit |
|---|---|---|
| 0–2 | Archive and toolchain; the workspace; money and the schemas | see SPIKE.md |
| 3 | Reading xero-axi's record and gl into the ledger; a second load writes nothing | `15e4eb8` |
| xero-cli 1–5 | xero-cli built, tested against a fake Xero, and answering for our four orgs | xero-cli `78f64e1`…`9ee93fa` |

Step 3's reader (`components/xero/src/ownpurse/xero/record.clj`, `samples/xero-axi/brackenridge`, `(load-xero)` and
`(load-sample)` in `dev.clj`) reads a xero-axi record. It keeps working until step 3b replaces it, so its tests
and the load path stay green meanwhile; it is removed in the commit that finishes 3b.

## Next: step 3b, reading Xero through xero-cli

Replaces step 3's source. Steps 4 onward are unchanged and build on this.

### 3b.1 Decide where postings come from

Xero's `Journals` endpoint is Xero's own general ledger: every posting Xero made, by account, for every document.
Using it, ownpurse derives nothing and the trial balance matches by construction (ownpurse still checks it). It
needs the `accounting.journals.read` scope, which neither xero-cli's default scopes nor the current login have:
`xero-cli get Journals --org <org>` is refused today.

1. Add `accounting.journals.read` to xero-cli's scopes, and have the owner of the books log in again in the
   browser (`xero-cli login live`). Only a person can do this step.
2. If Xero grants it: postings come from Journals. Documents (Invoices, BankTransactions, ManualJournals, ...) are
   still kept, as the evidence a journal points to (`SourceID`, `SourceType`).
3. If Xero refuses it for this app: port xero-axi's rules (`ledger.rs`, above) into the `xero` component as pure
   functions over Xero's JSON, with its goldens as tests. Its known gaps: Overpayments and Prepayments post
   nothing, and cash basis is a separate mode.

Done when the choice is written here with the reason.

### 3b.2 Fetch: running xero-cli

- A function at the edge of the `xero` component runs `xero-cli get <Resource> --org <tenant id> --all
  [--modified-since <last fetch>]` and reads stdout with ownpurse's JSON mapper (every number a BigDecimal).
  Pass tenant ids, never aliases or names.
- xero-cli's exit codes become `ex-info` types: 1 `:xero/usage`, 2 `:xero/login-needed`, 3 `:xero/daily-limit`,
  4 `:xero/refused`, 5 `:xero/failed`, with xero-cli's JSON error (on stderr) as the data. The hint is shown to
  the person as it is.
- Where xero-cli is and which orgs to read come from a local, uncommitted config (Aero, with
  `OWNPURSE_XERO_CLI` for the command). Tenant ids never enter the repository.
- Tests never run xero-cli. They use a stand-in command that prints fixture files and exits with a chosen code.
- Budget: this app gets 1000 calls a day per org. A first full fetch is one call per 1000 items per resource
  (Journals: per 100); after that, `--modified-since` keeps it small.

### 3b.3 Keep: Xero's JSON as facts

- Each item is a document keyed by its Xero id, with its payload (kebab-case keys, no nils, as step 3 found), its
  `UpdatedDateUTC`, and the org. Each fetch is one transaction whose metadata names xero-cli's version, the org,
  the resource and the time of the fetch.
- `store/put-changed!` as in step 3: a second fetch of the same data writes nothing.
- Xero reports deletions as status changes (`DELETED`, `VOIDED`) on a later fetch. That is where the open question
  from step 3 lands: when a document is voided or loses a line, its postings must stop counting. Decide here
  between XTDB `DELETE` (the row's validity ends, history keeps it) and a void flag, and write the reason down.

### 3b.4 Derive and check

- Entries and postings from the source chosen in 3b.1, validated by the `schema` component (every entry balances
  in every commodity).
- Fixtures: `samples/xero/pull-1` and `samples/xero/pull-2` are invented Xero API answers for one org (Tallowmere
  Coffee Roasting Co.) on 2026-03-20 and 2026-04-02, in exactly the shape xero-cli prints. They need Journals
  files added if 3b.1 chooses Journals, and a TrialBalance report for the check.

Done when:

1. pull-1 loads; a second load of pull-1 writes nothing; pull-2 after pull-1 writes only what changed.
2. Every entry balances, and the trial balance at a date equals the fixture's TrialBalance to the cent.
3. Locally, against real Xero: each of our four orgs loads, and its trial balance at 2025-12-31 equals `xero-cli
   report TrialBalance --org <tenant id> --param date=2025-12-31` to the cent.
4. Step 3's xero-axi reader, its sample, next.jdbc and the SQLite driver are removed, and `.gitignore` loses the
   `samples/xero-axi` exception.

## After 3b

Steps 4 to 8 as in SPIKE.md. Step 5 (time) can use pull-1 and pull-2: what the books said on 2026-03-31, as we knew
it on 2026-03-20 and on 2026-04-02.

## Housekeeping

- ownpurse's `next` branch is not pushed.
