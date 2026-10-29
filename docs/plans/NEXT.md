# Where ownpurse stands, and what comes next

This file is the working state: the decisions that still hold, where things stand, and the next steps in order. Update
it when a step lands. The plans: [SPIKE.md](SPIKE.md) (the ledger in Clojure on XTDB), [SCENARIOS.md](SCENARIOS.md)
(the books as layers), [SANDBOX.md](SANDBOX.md) (lines, scenarios and runs as building blocks). The history to
2026-10-29, with every step's details and what was learnt, is
[archive/plans/NEXT-2026-10-29.md](../archive/plans/NEXT-2026-10-29.md).

## Now

**SANDBOX.md step 1: say where you are, in plain words.** The admin's bar names the workspace; the screens stop saying
"the record" for the data; the terms page gains the new words. Then steps 2 to 5 of SANDBOX.md.

## Decisions that hold

- **Sources.** Xero through xero-cli, Mercury through mercury-cli, other banks and cards through plaid-reader: thin
  private command lines that log in and fetch, and store nothing. ownpurse never calls a source's API and never holds
  its token; it keeps what the tool prints as facts in XTDB. The tools print TOON by default; ownpurse asks for JSON.
- **Postings come from Xero's documents, not its Journals** (Journals needs Xero's Advanced tier). The ledger checks
  itself against Xero's TrialBalance report.
- **Two systems of record.** Each entity's book keeps its accounting system (Xero) as its system of record; ownpurse
  is the group's: intercompany, matches across sources, history, and books no accounting system keeps. A fix to an
  entity's books goes back as a sealed package a person approves; ownpurse never edits its copy (PHILOSOPHY.md).
- **Words for the books.** A *book* is one entity's record; each has one system of record, and every other source for
  it is *evidence*. Entities link to source accounts, not to sources.
- **Nothing is erased.** Source rows are never deleted; derived rows end by XTDB `DELETE`, keeping their history.
- **REPL-driven.** One long-lived REPL holds the running system; the shell run is the check before a commit. After
  many reloads, `(dev/halt)` and `(dev/go)` rebuild a handler that serves stale functions.
- **Workspaces.** A group's books live in its own private workspace, which runs this checkout's source live. Its
  session may change ownpurse directly; every commit passes `bin/privacy-check` (the denylist, then an agent's review).
  Never skip it.
- **Classifying.** Jev answers are kept by their exact request (sha-256 of model, state and questions), so nothing is
  asked twice. Only rules no exact block rules out are sent. A coding under the threshold, or stopped by a check, goes
  to a person. Cost is shown before a run, at TypeSafe's published price.
- **Blog posts argue a position**, distilled from the sessions (docs/BLOG.md); each opens with a demo pinned to a
  tagged commit.

## Where things stand (2026-10-29)

- **Ledger and sources (SPIKE.md steps 1 to 3b):** done. All four orgs of the first group tie to Xero's trial balance
  at 2025-12-31; Mercury and Plaid are sources; history, attachments and their files are kept; backups snapshot the
  record. The close register, intercompany pairs, payees, statement files and the accountant's export are built.
- **Scenarios (SCENARIOS.md):** step 1 done (journals and close layers, reports by scenario). Step 2 mostly done:
  world, coding, lines and key layers, coding runs through Jev with an estimate first, the admin's Scenarios section,
  projects that pay a per diem. Not yet: each bucket naming its account in each book; a child shortening or turning
  off what a world layer under it says.
- **The admin:** Sandbox (runs over made-up and real lines; a line's way through, its rules applied, and its decision
  as a flow with the numbers worked out), Scenarios, Docs (the guide on a real line, with each step's call, input,
  output and database reads; the terms). The static copy is live at ownpurse.com/sandbox.
- **The classifier's eval:** 300 made-up lines (atlas), scored at 0.85; runs vary a few points run to run.

## After the sandbox

1. SCENARIOS.md step 2's rest, then step 3 (a person codes a line and says why; the reasons become proposed rules),
   step 4 (intercompany as a layer), step 5 (compare and publish), step 6 (the manifest), step 7 (the Xero syncer).
2. SPIKE.md steps 4 to 8.

## Housekeeping

- **Repositories.** xero-cli, mercury-cli and plaid-reader are private, each with its own `docs/NEXT.md`. xero-axi is
  archived. ownpurse's `next` branch is the Clojure work; `main` is still the Rust code until the spike passes.
- `site/` still describes the archived Rust tool, apart from the sandbox, its two docs pages and the posts. Rewritten
  when the spike passes.
- **One response shape (for later).** Lists come back under a different key per endpoint; one shape would shorten
  clients. It changes the contract: do it once, with the OpenAPI document and the clients.
- **The eval's `-main` is not for a workspace**: it writes runs into this repository. A workspace runs real lines
  through its own REPL. Small fix: `-main` takes a scenario file and `--runs DIR`.
- Shared source code: `xero` and `mercury` each normalise payloads their own way; move the shared part into a
  component when a third source needs it.
