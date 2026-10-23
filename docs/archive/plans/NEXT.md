# What comes next

Status: proposal for review, 2026-10-12; updated 2026-10-13 with the Beancount, Beancount.io and TigerBeetle
assessment ([BEANCOUNT-TIGERBEETLE-2026-10.md](../research/BEANCOUNT-TIGERBEETLE-2026-10.md)). Nothing here is scheduled until it is agreed. This puts the existing plans
([FACTS-ROLLOUT.md](FACTS-ROLLOUT.md), [FACTS-EDGES.md](FACTS-EDGES.md), [TUI-REDESIGN.md](TUI-REDESIGN.md),
[ROADMAP.md](../ROADMAP.md)) in one order, and adds what the landscape research changed
([LANDSCAPE-2026-10.md](../research/LANDSCAPE-2026-10.md), [the post](https://ownpurse.com/blog/the-landscape/)).

## Where we are

- **Shipped (0.16.0):** the `ownpurse` CLI and TUI over a Xero-only `versions` record: sync, the ledger rebuilt
  to the cent, parity with Xero, multi-organisation scopes, intercompany, compare, classify, sealed write packages.
- **On `facts`, not in the binary:** the fact model, phase 1. The datom log, the drawer, ten projectors, three link
  rules, one synthetic month from every kind of source, `ownpurse-facts`, the demo and three blog posts. Two open
  gaps are written as ignored tests.
- **Research:** 135 dossiers, the history, and a synthesis of where ownpurse fits; a closer look at Beancount,
  Beancount.io and TigerBeetle.

## What the research changed

The landscape says ownpurse's claim is four things together (the owner holds the record, every version is kept, it
sits beside several ledgers, and writes are approved with the approval recorded), and that each one alone is
available elsewhere. It also says the need is unproven and that ease of use has beaten principle before (Wesabe and
Mint). Four consequences shape this plan:

1. **Finish the Xero path on facts before adding breadth.** One connector that works end to end on the real log is
   worth more than five on synthetic data.
2. **The bank is the second source, not QuickBooks.** Xero can't check itself (no statement lines in its API), and
   the bank witness is the check no one else gives a small business.
3. **The first run has to take minutes.** Today it means registering a Xero app and learning a CLI. This is the
   risk most likely to sink us, so it gets its own phase, early.
4. **Test the need while building.** Put the tool in front of real Xero users and bookkeepers by the end of phase
   C, and let what they do decide the order of phases E onwards.

The closer look at three neighbours adds:

- **Beancount** is something to export to, not build on: Fava becomes a free interface and `bean-check` a second,
  independent checker of our ledger.
- **Beancount.io** has the best agent interface among our neighbours, and approval that lives in the client. Phase
  D has to match the first and keep the contrast on the second. Watch for it adding a Xero or QuickBooks connector.
- **TigerBeetle** is not a component. Borrow its simulation testing (phase B) and its pending, posted and voided
  lifecycle (phase D).

## Decisions for you

These block or shape the phases below. My recommendation is first in each.

| # | Decision | Options | Recommendation and why |
|---|---|---|---|
| N1 | The Xero app model | Each person registers their own app (free Starter tier: 5 organisations, 1,000 calls a day, no Journals) / ownpurse runs a shared, certified app | **Own app by default**, with the setup scripted (phase C). A shared app puts us in the middle, brings egress fees and Xero's approval, and contradicts "nothing passes through us". Revisit only if setup is still too hard after phase C |
| N2 | Consent before data goes to a hosted model | Ask once per source / ask once per session / never send without an explicit flag | **Ask once per source, recorded as a fact**, revocable, and shown in `ownpurse status`. Xero's terms require consent before data passes to a third party; whether a hosted model counts is unconfirmed, so we assume it does |
| N3 | Mirror entries as derivations (open since FACTS-EDGES) | Projector claims / derivations rebuilt by a rule over the log | **Derivations.** It makes replay order-free (core change 8) and keeps Xero rules out of core (D20) |
| N4 | When the TUI redesign happens | Now, in parallel / after the Xero cutover (phase B) | **After phase B.** The TUI switches to reading the `Db` at the cutover; restyling it first means doing the snapshot review twice |
| N5 | An MCP server | Read-only MCP beside the CLI / CLI only | **Read-only MCP in phase D.** Every close neighbour has one; it's the cheapest way for any agent to reach the record. Writes stay CLI-only, through packages |
| N6 | Who we build for first | Developer-leaning Xero users with agents / bookkeepers and small firms / both | **Xero users with agents first, firms second.** The first group can run a CLI today; firms need a multi-client model (phase G) |
| N7 | Early users | A small invited group from phase C / public release only | **Invite 10 to 20 people at the end of phase C**, Xero users who already use an agent, plus three to five bookkeepers. Their behaviour orders phases E to H |
| N8 | A hands-on test of Beancount.io | Free plan with a synthetic ledger / desk research only | **A short test on the free plan**, synthetic data only, to confirm whether an MCP client can merge its own proposal and whether force-pushes are allowed. It means creating an account there, so it's your call |

## The phases

Each phase has its own detailed plan when it starts (as FACTS-ROLLOUT did), a release, and a blog post with a
pinned demo ([BLOG.md](../BLOG.md)). Sizes are relative: S is about a session, M a few, L many.

### A. Fact model, phase 2: close the gaps (M)

The core changes phase 1 found, in the order [FACTS-EDGES.md](FACTS-EDGES.md) lists them.

- Snapshot ownership by projector, so that a re-read never undoes a person's correction, plus `Snapshot::sweep`.
- Rules, not lookups: account codes, the tax account and account masks become link rules, so replay order doesn't
  matter (N3).
- Time (`obs/period-*`, `book/time-zone`, `doc/version` for stale reads), legs (`obs/role`), readings in core with
  geometry, balances (`obs/balance-after`, `obs/balance-kind`), commodities (CAIP-19, `price/date`).
- Pressure levels (note, problem).
- Phase 2's month adds a person's note and a correction, as the up-close post asked for.

**Done when:** the two ignored tests in `ownpurse-facts-cli/tests/month.rs` pass; loading the month in any order
gives the same facts; pressure problems are zero on the month.

### B. Xero on facts: the cutover (L)

FACTS-ROLLOUT milestones M3 to M5 on real data, together with the connector fixes the research found.

- **M3:** the projector over today's `versions` and the Demo Company, the mapping tables (D21), unmapped fields
  reported (D22). Laws L1 to L4 on every fixture version.
- **M4:** the ledger from facts, accrual and cash. L5 and L6: `tb`, `gl`, `parity`, `ic` and `compare`
  byte-identical on both paths.
- **Connector fixes:**
  - `includeDeleted=true` on bank transfers
  - the DELETED/VOIDED pass on incremental syncs, not only on full sweeps
  - each page written to the drawer as it arrives, so a failure loses nothing
  - the cursor stored as facts
  - the sweep cadence made a per-connector setting
  - attachments for Accounts and Quotes, history for ExpenseClaims and Receipts
  - check `unitdp=4` on the Demo Company before relying on it
- **Simulation testing for sync** (from TigerBeetle): a seeded simulator drives sync against the loopback Xero fake
  with timeouts, rate limits, partial pages and deletes in the middle of a sweep, and checks laws L1 to L9 after
  each run. A failure replays from its seed.
- **M5:** sync writes to the drawer and the log; `versions` is frozen; `ownpurse verify` and `ownpurse facts` ship;
  the TUI reads the `Db`; the old path is removed.

**Done when:** a live Demo Company run syncs, projects and matches Xero's trial balance; every golden is unchanged;
the sync simulator runs in the gate with no law broken; release 0.17.0.

### C. The first run in minutes (M)

The Wesabe lesson. Measured, not guessed: time from install to the first checked trial balance.

- `ownpurse init`: a guided setup that walks through registering the Xero app, step by step, checking each value
  as it's pasted, and ends with the first sync and the parity check.
- `ownpurse demo`: the synthetic month in one command, with no Xero account, for people deciding whether to try it.
- Install: one line on macOS and Linux, and a prebuilt binary, so nobody needs Rust.
- The agent skill that installs with the CLI (from `docs/agent-bridge-skill-snippet.md`), so an agent knows how to
  drive ownpurse from the first session.
- Consent (N2) as part of `init`.
- `ownpurse export beancount`: a journal derived from the record, each entry carrying metadata back to its fact, so
  Fava works as an interface from day one. Tax rates and tracking categories become tags or metadata, exchange
  gains and losses are mapped with care, and the tolerance is zero. The gate runs `bean-check` on the demo export
  and compares its balances with `ownpurse tb`.

**Done when:** someone new to ownpurse, with a Xero login, reaches a checked trial balance in under 15 minutes,
timed on three people who haven't seen it before. Then invite the early users (N7).

### D. Agents on the record (M)

- A read-only MCP server over the record (N5): entities, history, as-of, the views, `sql`, and the checks. Reads
  never call Xero.
- `ownpurse q`: the query command the roadmap lists, the same surface as the MCP server.
- Proposals as facts: an agent's suggested link, label or write is a held transaction, and a person approves it
  from the CLI or the TUI. This uses the package path that already exists. Each proposal is pending, then posted
  or voided, every step recorded (TigerBeetle's lifecycle); a package goes out whole or not at all; its id is the
  idempotency key, so a retry never applies twice.
- The agent interface meets Beancount.io's: a dry run returns the diff and the checks' verdict, tokens are scoped
  to one set of books, errors are typed with hints, and there is an `llms.txt` and an MCP manifest. No agent gets
  account-level powers, and the agent that proposes can never approve.
- A trace of what each agent read and proposed, recorded in the log.

**Done when:** an agent with only the MCP server answers the four questions in the up-close post's agent traces on
a real Demo Company record, and every proposal it makes lands as a held transaction.

### E. The bank as witness (L)

FACTS-ROLLOUT M8 on the real path, starting with what one person can connect alone.

- Files first: OFX, QFX and CSV downloads (the phase 1 projectors, hardened), then statement PDFs with readings.
- Then a feed: SimpleFIN (a read-only token the person holds), and Akahu personal apps for New Zealand.
- Link rules to Xero bank transactions; a reconciliation report showing what the bank has that Xero doesn't, and the
  reverse; statement balance checks, recorded as balance-assertion facts ("the bank said X on date D"), the check
  Beancount has shown works.
- `ownpurse import <file>` for anything dropped into the drawer by hand.

**Done when:** on a synthetic and a real (local, never committed) account, every bank line is linked to Xero or
listed as missing, and every statement checks itself.

### F. The record you can read without us (S to M)

Custody is only real if the file outlives the program (Microsoft Money's lesson).

- `docs/RECORD-FORMAT.md`: the tables, the value encodings, the canonical encoding and the chain, with the frozen
  test vectors.
- An independent reader: a short Python script in `docs/` that opens a record, verifies the chain and prints a
  trial balance, tested in the gate.
- Export: the whole record to Parquet or CSV per view, and a portable bundle (log plus drawer).

**Done when:** the script, written only from the document, verifies the demo record and matches `ownpurse tb`.
This can run in parallel with D or E.

### G. The accountant (M to L)

The history says the accountant is the integration layer for small businesses, and firms already pay per client.

- One record per client, and a firm view across them, read-only.
- Sign-offs as facts: a preparer and a reviewer per period, recorded in the client's record.
- A shareable, verifiable bundle for a period (the record up to a transaction, plus the drawer files it cites).
- A changes report: what each tool or agent changed in the client's ledger, before and after.

**Done when:** a bookkeeper among the early users closes a month for two clients with it. Decide the scope after
talking to the bookkeepers in N7.

### H. Breadth (L, ongoing)

Ordered by what early users ask for. The default order is the one the research suggests:

1. **Stripe:** payouts and fees against bank deposits and ledger entries.
2. **QuickBooks Online:** the second ledger. Its reads are metered, so a local copy saves money.
3. **Cards and spend:** Ramp, Brex and Expensify exports against bills.
4. **Receipts and checks** (FACTS-ROLLOUT M9), with an extractor and readings.
5. **FreshBooks, sheets and workbooks, wallets**, from the phase 1 projectors.
6. **A Beancount journal as a source**, so people who keep one bring their history in.

Each lands through **the connector framework**, built with the first of these: a manifest per source (streams, the
identity and cursor of each, the raw shape), a pure projector, the fetch writing pages to the drawer, and the rule
that only a completed, verified sweep may retract. A guide for writing one goes in `docs/`, with a test kit that
runs every law against a new connector.

### Alongside

- **The TUI redesign** ([TUI-REDESIGN.md](TUI-REDESIGN.md)), after phase B (N4), as 0.18.0.
- **The engine benchmark** (FACTS-ROLLOUT M6), when a real record passes a million facts, or before phase H if load
  times grow.
- **Classify and manifests on the log** (M7), with or right after phase D, since proposals use the same mechanism.
- **Authority** (M10, native books): not before the early users ask for it.
- **The landscape:** refresh quarterly. Re-check the claims marked unverified, add new entrants, and re-score.
  Confirm with Xero whether sending data to a hosted model counts as passing it to a third party (N2). Watch
  Beancount.io for a Xero or QuickBooks connector.
- **Later:** TigerBeetle's change data capture stream as a source, for companies that run it as their payment
  ledger. Not before phase H.
- **Every phase:** CHANGELOG notes, docs pages updated in the same change, and a blog post with a pinned demo.

## Order and dependencies

```
A ─► B ─► C ─► (early users) ─► E ─► H
          │                     ▲
          └─► D ────────────────┘
F runs in parallel from B on.  G after C, scoped by the bookkeepers.  TUI redesign after B.
```

- A before B: the cutover should land on the corrected core, not be migrated twice.
- C right after B: there's no point inviting people before the real path works, and no point building breadth
  before people can get in.
- D can start once B lands; it doesn't need C.
- E before H: the bank witness is the differentiator; more ledgers are not.

## What we measure

The research left one question open: is the empty corner an opening, or a market that doesn't exist? These are
the signals we'd watch from the end of phase C:

- time from install to the first checked trial balance (target under 15 minutes)
- how many early users still sync weekly after a month
- how many connect a bank witness when it ships, and whether it finds anything
- how many proposals agents make, and how many people approve
- what people ask for next, in their words

If few still sync after a month, phases E to H wait until we understand why.

## Risks to this plan

| Risk | What we do |
|---|---|
| Phase B is bigger than it looks (cash basis parity, D20) | FACTS-ROLLOUT's fallback: keep today's cash logic behind the same interface until L5 passes |
| Xero changes its terms or the Starter tier | N1 keeps us off a shared app; the record is already local, so a change limits new reads, never old data |
| Setup is still too hard after phase C | Measure it; if it misses 15 minutes, revisit N1 before inviting more people |
| Bank feeds close (Teller, GoCardless did) | Files first in phase E; feeds are additions, never the only path |
| A neighbour adds a Xero connector (Beancount.io is one short step away) | The difference is then custody, source history and the enforced gate; phase C must make our first run as easy as theirs |
| One maintainer | Phase F makes the record readable without us; every phase leaves a plan and a post |
