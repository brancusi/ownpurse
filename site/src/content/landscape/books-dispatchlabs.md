---
{
 "slug": "books-dispatchlabs",
 "name": "Books (dispatchlabs-ai)",
 "url": "https://pkg.go.dev/github.com/dispatchlabs-ai/books",
 "segment": "agent-native",
 "status": "active",
 "founded": "2026 (first public release v0.1.0-alpha.2 on 2026-08-19)",
 "hq": "Not stated",
 "backing": "Published by Dispatch Labs, a GitHub organisation created 2026-05-09 with four other public repos (agent-wiki and related tools). No funding, company site or people found.",
 "tagline": "A local-first, noninteractive command-line general ledger in SQLite, built for humans and software agents, with immutable postings and a hash-chained audit log.",
 "scope": "Exact double-entry books for one or many legal entities in a SQLite file per company: manual chart, transaction entry, statement imports, bank, card, loan and investment reconciliation, GL, trial balance, P&L, balance sheet, period and fiscal-year close, consolidation, a one-time QuickBooks import, audit verification and verified backup. It does not do invoicing, AR/AP workflow, bill pay, payroll, inventory, tax, automatic bank feeds, non-USD currencies or a web interface, and runs on macOS and Linux only.",
 "domains": [
  "ledger",
  "bank",
  "close",
  "reporting"
 ],
 "customers_served": [
  "developers",
  "smb",
  "freelancer"
 ],
 "spectrum": {
  "custody": {
   "score": 4,
   "note": "The authoritative ledger is a SQLite file under ~/.books on your machine; provider exports are treated as evidence only."
  },
  "locality": {
   "score": 4,
   "note": "A Go binary that runs entirely on your own machine; no service."
  },
  "history": {
   "score": 3,
   "note": "Posted journals, evidence, reconciliations and audit events are immutable; provider observations are kept as superseding revisions with full history; no general as-of query interface is documented."
  },
  "openness": {
   "score": 4,
   "note": "MIT licensed source on the Go module proxy, though the GitHub repository returned 404 on 2026-10-10."
  },
  "breadth": {
   "score": 1,
   "note": "One ledger fed by manual entry, JSON statement imports and a one-time QuickBooks import; no live connectors, documents or payments."
  },
  "autonomy": {
   "score": 2,
   "note": "An agent can post directly through the CLI with idempotency keys; drafts and dry runs exist but there is no approval gate and nothing runs unattended."
  },
  "agent_readiness": {
   "score": 3,
   "note": "Noninteractive by design, a stable JSON envelope, exact decimal strings, stable error codes, idempotent --key, dry runs and plan/apply; no MCP."
  },
  "integration_ease": {
   "score": 2,
   "note": "Documented JSON import schema and CLI, but an alpha that needs Go 1.26.6 and CGO to build, USD only, and few users."
  },
  "market": {
   "score": 1,
   "note": "Aimed at technical owners and small multi-entity businesses who keep their own books."
  }
 },
 "api": {
  "public": true,
  "style": "CLI with a books.cli/v1 JSON envelope (also JSONL, CSV, table); JSON import schema banking-import-v1",
  "auth": "None; local file permissions. Direct SQLite writes are unsupported; the CLI is the only write interface.",
  "limits": "None beyond the local machine.",
  "cost": "Free.",
  "export": "The whole SQLite file, verified backup and restore, and every report in JSON, JSONL or CSV.",
  "notes": "Each open checks the SQLite application id, a checksummed migration ledger and the live schema against a pristine embedded copy."
 },
 "pricing": {
  "model": "Free, open-source (MIT).",
  "entry": "Free",
  "top": "Free",
  "notes": "No paid tier, support or service level; the project says it is experimental and not accounting, tax or legal advice."
 },
 "agents": {
  "mcp": "none",
  "ai_features": "None built in. It is a tool for agents to drive, not an agent.",
  "agent_writes": "Yes, through the CLI. Agents must pass --company and a stable --key on mutating commands; every agent-operated transaction must carry company, date and actor.",
  "approval": "No approval gate. Safety comes from --draft (save without posting), --dry-run, plan/apply workflows that bind state and reject stale plans, and corrections only by linked reversal.",
  "notes": "The project's AI policy requires a named human accountable for every AI-assisted change to the code, and AGENTS.md tells agents to work only in a disposable BOOKS_HOME."
 },
 "integration": {
  "scope": "QuickBooks initial import (GeneralLedger JSON report, QBO object exports or a reviewed XLSX); JSON statement imports for bank, card, loan and investment accounts. No live connectors.",
  "notes": "The changelog says deployment-specific accounting mappings and private integration bridges were removed before release, so live feeds probably exist privately."
 },
 "preference": {
  "scale": "No public usage figures. Two tagged releases on 2026-08-19; the Go proxy shows a later commit on 2026-09-20.",
  "ratings": "None found.",
  "sentiment": "No public reviews or discussion found."
 },
 "timeline": [
  {
   "date": "2026-05-09",
   "event": "Dispatch Labs GitHub organisation created"
  },
  {
   "date": "2026-08-19",
   "event": "v0.1.0-alpha.1 and v0.1.0-alpha.2 published as the first public source release from a fresh root commit, MIT"
  },
  {
   "date": "2026-09-20",
   "event": "Latest commit visible on the Go module proxy"
  },
  {
   "date": "2026-10-10",
   "event": "github.com/dispatchlabs-ai/books returns 404; source still served by the Go proxy and pkg.go.dev"
  }
 ],
 "relation": "kin",
 "relation_note": "The closest project to ownpurse found: local, SQLite, exact money, immutable postings, a hash-chained audit log, a noninteractive CLI with a JSON envelope for agents. It differs in being the ledger of record you post into, while ownpurse rebuilds the books from what each source reports and checks them against it.",
 "borrow": "Source observations as immutable revisions with an explicit disposition (posted, pending, needs review, source only), close to ownpurse's uncollapsed facts; plan then apply, with apply binding the plan's state and rejecting stale plans; one failure envelope with stable codes; idempotency keys required of agents; a doctor that checks schema, migrations and invariants on every open; closed-period balance digests; an honest note that a hash chain detects uncoordinated edits but cannot stop someone who rewrites the whole file.",
 "avoid": "A narrow alpha surface (USD only, no live sources, CGO build) that limits who can try it; agent writes with no approval step; a repository that disappears from GitHub without notice, which undercuts trust in a tool meant to hold the books.",
 "sources": [
  {
   "title": "pkg.go.dev: github.com/dispatchlabs-ai/books",
   "url": "https://pkg.go.dev/github.com/dispatchlabs-ai/books"
  },
  {
   "title": "Go module proxy: version list",
   "url": "https://proxy.golang.org/github.com/dispatchlabs-ai/books/@v/list"
  },
  {
   "title": "Go module proxy: latest version info",
   "url": "https://proxy.golang.org/github.com/dispatchlabs-ai/books/@latest"
  },
  {
   "title": "Dispatch Labs on GitHub",
   "url": "https://github.com/dispatchlabs-ai"
  }
 ],
 "unverified": [
  "Who is behind Dispatch Labs, where it is based and whether it is funded",
  "Why the GitHub repository returns 404 as of 2026-10-10 (made private, moved or deleted)",
  "Whether the private integration bridges named in the changelog include live bank feeds",
  "Any user count or adoption"
 ],
 "researched": "2026-10-10"
}
---

## What it is

Books describes itself as "a local-first, command-line general ledger for humans and software agents" ([pkg.go.dev](https://pkg.go.dev/github.com/dispatchlabs-ai/books)). It keeps exact double-entry books in SQLite and makes posted accounting immutable. The first public release, v0.1.0-alpha.2, came out on 19 August 2026 under the MIT licence from Dispatch Labs, a GitHub organisation created in May 2026 whose other public work is an agent wiki and related agent tools. It is experimental, USD only, and runs on macOS and Linux.

It covers company setup, a manual chart, transaction entry, statement imports, reconciliation of bank, card, loan and investment accounts, the four core reports, period and fiscal-year close, multi-entity consolidation, a one-time QuickBooks import, audit verification and verified backup. It leaves out invoicing, AR and AP, payroll, tax, automatic bank feeds and any web interface.

## How it works

Each company is a SQLite file under `~/.books/companies/<name>/` with folders for attachments, imports, plans and backups. Money is stored as integer cents and never touches floating point. Journals start as drafts; posting checks balance, open period and account activation, and after that a journal cannot change. A mistake is fixed with a linked reversal (`reverse`, `undo`, `correct`). Closed periods keep a balance digest.

The architecture notes are the interesting part. Every provider observation becomes a new immutable `source_record` revision that points to the one it supersedes, and each has a disposition: `POSTED`, `PENDING`, `NEEDS_REVIEW` or `SOURCE_ONLY`. Only posted evidence enters reconciliation; moving an item out of review needs a reason and JSON evidence. Reconciliation, close and import all follow plan then apply, and apply rejects a plan whose state has changed. Every open checks the SQLite application id, a checksummed migration ledger and the live schema against an embedded copy. The audit log is hash-chained, and the docs say plainly that the chain "detects uncoordinated changes but does not resist a privileged party that can rewrite and replace the entire SQLite file".

## Data and access

You hold the file. There is no service, no account and no limits. The interface is the CLI: table, JSON (`books.cli/v1` envelope), JSONL or CSV output, decimal strings for money, stable error codes, one failure envelope per failed command. Statement data comes in as JSON against a published schema; ownership is never guessed from labels or amounts.

## Agents

Books is written for agents to drive. No command prompts, stdin is read only with `--input -`, ambiguity exits nonzero, and agents must pass `--company` and a stable `--key` so retries are idempotent. `--draft` and `--dry-run` let an agent prepare without posting. There is no MCP and no approval gate: an agent with shell access can post. The project's [AI policy](https://pkg.go.dev/github.com/dispatchlabs-ai/books) holds a named human accountable for every AI-assisted change to the code.

## Customers

No usage figures, reviews or discussion were found. The Go proxy lists two tagged versions, both from 19 August 2026, and a later commit from 20 September 2026.

## History

The changelog says the release came "from a fresh root commit" with "all deployment-specific accounting mappings and private integration bridges" removed. That suggests it was extracted from a private deployment that had live feeds. On 10 October 2026 the GitHub repository returned 404, while pkg.go.dev and the Go proxy still served the source.

## For ownpurse

This is kin, and the nearest we have found. Both are local, SQLite, exact to the cent, append-only, hash-chained and built for agents through a noninteractive CLI. The difference is the job. Books is the ledger of record: you post into it, and sources are evidence for reconciliation. ownpurse leaves the books in Xero or QuickBooks, rebuilds them from what each source reports, and writes back only through approved packages. Worth borrowing: the disposition states on source revisions, plan and apply with stale-plan rejection, closed-period digests, a doctor on every open, and the honest limit of a hash chain. The gap Books leaves open is the one ownpurse aims at: live connectors to the systems people already use, and a person's approval before anything is written.
