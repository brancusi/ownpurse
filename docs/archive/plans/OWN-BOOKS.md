# Our own books first

Status: plan, 2026-10-23. This replaces the order in [NEXT.md](NEXT.md) for now: other users wait. The goal is to run
our own group of Xero organisations from sync to a closed year, with project budgets and reports for the people who
need them.

## What we need

1. **Every organisation in the group, working.** Sync, the ledger rebuilt to the cent, parity with Xero and the
   intercompany pairs, for every organisation in one profile.
2. **One data model over all of them.** The fact model, with each organisation as a book, intercompany and groups
   as facts, and a consolidated view with eliminations.
3. **Closing the books on Xero.** A close is a checklist computed from the record, the adjusting journals go out
   as sealed packages, and the close itself is recorded.
4. **Internal budgets for projects.** Budgets that live in ownpurse (not in Xero), by project, account and period,
   with every revision kept, and budget against actual across organisations.
5. **Reports for stakeholders.** Report definitions as files, rendered from the record at a stated point in time,
   so any report can be produced again exactly.

## One tool per source, ownpurse above them

Decided 2026-10-23. Each source has its own dedicated tool: xero-axi for Xero, mercury-axi for Mercury, and later
one for QuickBooks and the rest. ownpurse is not a Xero tool. It is the data model over every source, and the
work that needs more than one source.

| | The source tool (xero-axi) | ownpurse |
|---|---|---|
| Holds | the token, sync, the source's own record and chain | the fact log over every source |
| Computes | the source's ledger, parity with the source's own reports | books, groups, consolidation, budgets, reports |
| Writes | sealed, approved packages, run against the source | write intents, handed to the source tool to plan and seal |
| Never | knows about other sources | holds a token or calls a provider |

Three rules follow:

- **The boundary is a written contract.** ownpurse reads a versioned export from each tool (or its record,
  read-only, with the chain checked), never its internal tables. Each fact names the tool, its version and the
  chain head it was read at.
- **Writes stay in the source tool.** ownpurse drafts a closing journal as an intent. xero-axi plans, seals and
  runs it after a person approves it, as today.
- **ownpurse drops its copy of the Xero tool.** `ownpurse-xero`, the Xero commands in `ownpurse-cli` and the TUI
  are xero-axi under another name. They are removed once ownpurse reads xero-axi's export, so the two never
  drift. AGENTS.md and the site change to match.

## Where we start

- xero-axi 0.16.0 runs our real books across four organisations, with intercompany pairs, groups and write
  packages. Its configuration and record live outside this repository.
- The fact model (branch `facts`) is not in any binary. Its Xero projector runs only on synthetic pulls.

## Phase 1: xero-axi on every organisation (in xero-axi)

- Offline first: `doctor`, `parity`, `ic`, `compare`, `pl` and `bs` over the existing record, accrual and cash,
  for every organisation. List what disagrees and why.
- Then a live sync of every organisation, and the same checks again.
- The connector fixes that matter for a close: `includeDeleted=true` on bank transfers, the DELETED/VOIDED pass on
  incremental syncs, each page kept as it arrives.
- The export contract: a versioned, documented stream of documents and their versions, with the chain head, that
  ownpurse reads.

**Done when:** every organisation agrees with Xero to the cent, and the export runs on all of them.

## Phase 2: the data model over every organisation (in ownpurse)

Phase A of NEXT.md cut down to what Xero needs, then phase B's M3 and M4, reading xero-axi's export.

- The Xero projector on the export: every organisation's record replays into the log (laws L1 to L4).
- The ledger from facts agrees with xero-axi's `tb`, `gl`, `parity` and `ic`, to the cent (L5, L6).
- Books, groups, parties and intercompany pairs as facts, not only config.
- A consolidated trial balance, P&L and balance sheet for a group, with eliminations from pairs that net to zero.
- Then ownpurse's copy of the Xero tool is removed.
- Out of scope for now: bank files, receipts, other ledgers, the TUI.

**Done when:** our four organisations replay into the log, both tools agree to the cent, and the group view
reconciles to the sum of its books less eliminations.

## Phase 3: closing the books

- `ownpurse close check <period>`: a checklist from the record for each book (parity, intercompany at zero, drafts
  and awaiting-payment items dated in the period, suspense and clearing balances, unreconciled bank lines, lock
  dates).
- Adjusting and closing journals drafted in ownpurse as intents, then planned, sealed and run by xero-axi.
- The close recorded as a fact: the period, the books, the checks that passed, the package hashes and the chain
  heads it was taken at. A later change to a closed period is reported.
- The lock date is set in Xero by a person (we don't assume the API can set it); ownpurse checks that it was set.

**Done when:** we close a real month on every organisation this way.

## Phase 4: project budgets (in ownpurse)

- The project as a dimension: Xero tracking options and the `classify` project label, mapped to one project
  entity that can span organisations.
- Budgets as facts: project, account (or a group of accounts), period, amount, and a named version (original,
  revised). A revision adds facts; nothing is overwritten.
- `ownpurse budget` to load a budget from a file and show budget against actual, committed (open bills and
  purchase orders) and remaining, by project, across organisations.

**Done when:** every live project has a budget, and budget against actual matches the ledger to the cent.

## Phase 5: reports for stakeholders (in ownpurse)

- A report is a file: its queries, its layout and its audience. It is rendered to PDF, HTML or CSV at a stated
  point in the log, and the output records that point and its hash.
- The first reports: a monthly management pack per organisation, the group consolidation, project budget against
  actual, and the year-end pack for the accountant.

**Done when:** each stakeholder's report is produced by one command, and an old report can be produced again
byte for byte.

## Private data

Everything above runs on our real books, which never enter this repository. Code, tests and fixtures stay
synthetic; any bug found on real data gets a synthetic reproduction before it is fixed. Read every diff before
committing, as [SECURITY.md](../../SECURITY.md#what-never-goes-in-this-repository) says.
