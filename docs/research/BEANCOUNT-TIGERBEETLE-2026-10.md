# Beancount, Beancount.io and TigerBeetle

Status: assessment, 2026-10-13. A closer look at three players from the landscape, drawn from their dossiers
(researched 2026-10-10): [beancount](../../site/src/content/landscape/beancount.md),
[beancount-io](../../site/src/content/landscape/beancount-io.md) and
[tigerbeetle](../../site/src/content/landscape/tigerbeetle.md). No hands-on trial of any of them yet. The steps
this leads to are folded into [NEXT.md](../plans/NEXT.md).

## In one line each

- **Beancount** is kin and something to integrate with: a strict, local, plain-text ledger. Build an export to it;
  don't build on it.
- **Beancount.io** is the nearest neighbour for developers and individuals who want agents on their books. Its agent
  interface sets the bar for ours; its client-side approval is the clearest contrast with ownpurse.
- **TigerBeetle** is a design cousin. Borrow its testing and its transfer lifecycle; don't run it.

## Beancount (the open-source project)

**What it gets right**

- Strictness. Every transaction must balance, lots must match, balance assertions apply by date. A wrong entry fails
  `bean-check` rather than skewing a balance: the right base for agents, and the same instinct as our checks.
- A durable format: plain text, readable without the tool, diffable in git. Agents write it well.
- Investments: cost basis, lots and prices are among the best in plain-text accounting.
- Fava, BQL and the plugin pipeline are well designed.
- beangulp's importer shape: identify, extract, archive the original, mark (not drop) likely duplicates.

**Where it falls short**

- The journal is edited in place, so what the source said is gone unless git kept it. No as-of queries.
- A ledger, not an accounting system: no invoices, contacts, tax, payroll or approvals. Accountants don't use it.
- Fetches nothing; you write or find an importer per bank.
- One author wrote nearly all of it; the version 3 split moved tools into separate packages.
- Small, technical audience (about 6,000 GitHub stars, a mailing list of about 880).
- Python, GPL-2.0-only, rereads the whole file on every run: not something to embed in a Rust binary.

**For ownpurse**

1. `ownpurse export beancount`: a journal derived from the record, each entry carrying metadata back to its fact.
   Fava becomes a free interface, and `bean-check` an independent second checker of our ledger. Map tax rates and
   tracking categories to tags or metadata, take care with exchange gains and losses, and set the tolerance to zero.
2. A Beancount journal as a source, so people bring their history in.
3. Balance assertions as facts ("the bank said X on date D"), the check the bank witness needs.

## Beancount.io (the hosted service)

Operated by Stargately, Inc.; its pages don't say it is connected to the Beancount project. Each ledger is a
private git repository with hosted Fava, Plaid bank sync, AI import, mobile apps and an assistant. Since June 2026,
an MCP server with about 31 tools, ledger-scoped OAuth, scoped API keys, GraphQL and `llms.txt`, plus `bea`, an
open-source local CLI. Free (one ledger, 1,000 entries), then USD 14.99 a month up to USD 499.99 for organisations.

**What it gets right**

- The best agent interface in its segment, and better than ours today: a dry run returns a diff and `bean-check`'s
  verdict, committing is a separate call, every write is a git commit labelled "AI edit", errors are typed with
  hints, and the docs are Markdown written for agents.
- Easy to leave: clone the repository with its history and keep using open-source Beancount.
- Plaid sync removes Beancount's biggest pain, writing importers.
- Setup in minutes: one command adds the MCP server, then a sign-in.

**Where it's weak**

- Approval is optional and lives in the client. Its MCP page says "approval settings depend on your client"; the
  same credential can open a pull request and merge it; an OAuth credential can delete the account.
- History is git over normalized text, not what Plaid or the bank reported. Force-push policy is undocumented.
- You move your books in. For a business on Xero with an accountant, that's a non-starter.
- The hosted repository is the working copy; AI requests pass through a third-party proxy (bex.co / BlockEden).
- Little sign of traction: no user counts, 8 iOS ratings averaging 3.5, 296 stars on its open-source repository.

**Against ownpurse**

- For a freelancer not on Xero it is the better product today: bank sync, mobile, MCP, a first run in minutes.
- ownpurse's books stay where they are, the record is local, history is what each source said, and the approval
  gate can't be skipped.
- Threat: low for Xero businesses, moderate for developers and individuals. Watch for a Xero or QuickBooks
  connector that reads into Beancount; they already have Plaid and a QuickBooks migration guide. If it comes, the
  difference is custody, history and the gate, and our first run has to be as easy as theirs.

**For ownpurse**

- Phase D meets its agent interface: dry runs with a diff and the checks' verdict, tokens scoped to one book,
  typed errors with hints, `llms.txt` and an MCP manifest. No agent ever gets account-level powers.
- Say plainly what sets us apart: approval the system enforces, and the record of what the source said.
- Make it a destination: the Beancount export lets someone view their record there, or in Fava.
- Don't compete on a hosted interface or mobile apps.

**Still unknown:** whether an MCP client can merge its own proposal in practice, whether force-pushes are allowed,
rate limits, user counts, and what users say. A hands-on test on the free plan with a synthetic ledger would
settle the first two; it means creating an account there, so it waits for a decision.

## TigerBeetle

A database for transfers between accounts: millions a second, strict serializability, a Jepsen analysis in June
2025, immutable transfers, pending, posted and voided two-phase transfers, balance limits, linked batches that apply
all or nothing, and deterministic simulation testing. Apache 2.0.

**Why not as a component**

- Wrong kind of system. It is the record for money you move. ownpurse records what other systems say moved; our
  hard problems are provenance, revisions and as-of, not throughput. A small business has thousands to tens of
  thousands of entries a year.
- It holds integers and a few numeric fields: no text, no provenance, no general queries, so it always needs a
  second database.
- Production is a six-replica cluster run as a server, against "a file on your machine".
- Our ledger is derived and rebuilt; loading it into TigerBeetle adds a server to keep in step and gains nothing.

**What to borrow**

1. **Simulation testing for sync.** A seeded simulator drives sync against the loopback Xero fake with timeouts,
   rate limits, partial pages and deletes mid-sweep, and checks laws L1 to L9 after each run. Any failure replays
   from its seed. This is the test behind phase B's "nothing lost when a sync fails".
2. **Pending, posted and voided as the lifecycle of a write package:** held, then posted or voided, each step
   recorded, nothing collapsed.
3. **Linked batches:** a package goes out to the source whole or not at all.
4. **Ids as idempotency keys,** so a retried write never applies twice; and imported history keeps its original
   timestamps beside the time we recorded it.

**Correction.** The landscape research listed "TigerBeetle's content-derived ids" as a borrow. Its ids are
client-generated and time-ordered (a 48-bit millisecond timestamp and 80 random bits). The idea worth taking is the
id as an idempotency key; content addressing is already ours, in the drawer. Fixed in
[LANDSCAPE-2026-10.md](LANDSCAPE-2026-10.md).

**Later:** a company running TigerBeetle as its payment ledger could have ownpurse read its change data capture
stream as a source. An enterprise case for phase H or after.
