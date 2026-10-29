# Where ownpurse stands, and what comes next

This file is the working state: the decisions that still hold, where things stand, and the next steps in order. Update
it when a step lands. The plans: [SPIKE.md](SPIKE.md) (the ledger in Clojure on XTDB), [SCENARIOS.md](SCENARIOS.md)
(the books as layers), [SANDBOX.md](SANDBOX.md) (lines, scenarios and runs as building blocks). The history to
2026-10-29, with every step's details and what was learnt, is
[archive/plans/NEXT-2026-10-29.md](../archive/plans/NEXT-2026-10-29.md).

## Now

**SANDBOX.md step 4: evidence.** Then step 5 (in use).

- The world is edited in place (2026-10-29): a scenario's Rules tab lists every project, trip, person, card, home
  and the business its world layers say (`:world` beside `:rules` on `GET /scenarios/{ref}/composed`, from
  `world-ledger`), each on or off (greyed in place) with where it comes from, and changes, adds or turns one off by
  writing into the scenario's own world layer. A trip is known by `:id`, else person and start; one of the same id
  above replaces it, and `:enabled false` turns off a trip, project, card or entity. A trip for a project that is
  off reads as personal.
- Step 3 done (2026-10-29): a run is a scenario over a set of lines. Sets: a scenario's lines layer
  (`<scenario>~<layer>`), a saved set (`line_sets`, `PUT /lines/{name}`: accounts or every book's, days, or chosen
  lines, with its own key, `POST /lines/{name}/key`) and the samples (`sample~<name>`). Over its own lines a run is
  kept as before ("latest"); over another set under `<ref>@<set>` (`:of`, `:over`), its sandbox run the set's id.
  `over` on the coding endpoints and `?run=` on the trace. The Sandbox offers every workspace scenario for any set;
  the Lines page saves a set; a line of the books can be run on its own. Learnt: a set changes a line's context (a
  refund is matched to purchases in the same set), so a smaller set can ask Jev again for a few lines.
- Step 2 done (2026-10-29): `GET /sandbox/lines` lists the sets, `GET /sandbox/lines/{set}` previews one. The
  Sandbox front is Lines → Scenario → Run, picked in the URL (`#/sandbox?lines=<set>&scenario=<name>`): one list a
  step, each row saying what it holds as a query, with a peek, opening in place. A Lines section lists the sets; a
  scenario's coding tab opens its coding in the sandbox.

- Step 1 done (2026-10-29): `GET /workspace` says where ownpurse runs (`{:workspace {:name}}` in the local config,
  the books in the record, the sources the config names; "demo" with none), and the admin's bar shows it on every
  page and in the tab's title ("Demo on ownpurse.com" on the static copy). The screens say "your books" and "sample
  lines (made up)" instead of "the record"; the terms page gains Workspace, Books, Record (storage), Lines, Key,
  Evidence and In use, and Sandbox, Scenario and Run read as SANDBOX.md defines them.

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
