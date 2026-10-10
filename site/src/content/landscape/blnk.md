---
{
 "slug": "blnk",
 "name": "Blnk",
 "url": "https://www.blnkfinance.com",
 "segment": "fact-ledger",
 "status": "active",
 "founded": "Repository created December 2022; Blnk Finance LLC copyright from 2023",
 "hq": "Not stated; Blnk Finance LLC, United States region on the site",
 "backing": "No funding disclosed on the company's site; investors not found",
 "tagline": "An open-source, double-entry ledger for fintech products, with reconciliation, PII tokenization and a managed cloud.",
 "scope": "Blnk Core is a Go ledger on PostgreSQL with immutable transactions, inflight (hold, then commit or void) transactions, balance snapshots and historical balances, reconciliation against uploaded external records, identity tokenization and search through Typesense. Blnk Cloud hosts it and adds a dashboard, a data lake and an MCP server. It does not connect to banks or accounting systems itself; you upload or post their records.",
 "domains": [
  "ledger",
  "payments",
  "close"
 ],
 "customers_served": [
  "developers",
  "smb",
  "mid-market"
 ],
 "spectrum": {
  "custody": {
   "score": 3,
   "note": "Apache 2.0 core self-hosts on your own Postgres; the docs and Cloud push a hosted instance, with migration in and out documented."
  },
  "locality": {
   "score": 2,
   "note": "Self-hostable with Docker Compose (Postgres, Redis, Typesense); not a single-file local tool."
  },
  "history": {
   "score": 3,
   "note": "Protected transaction fields are guarded by a database trigger, an optional global hash chain seals them, and balances can be asked as of any timestamp; metadata stays mutable and outside the chain."
  },
  "openness": {
   "score": 3,
   "note": "Core is Apache 2.0 including reconciliation; MCP, data lake and dashboard are Cloud-only."
  },
  "breadth": {
   "score": 1,
   "note": "Ledger plus reconciliation, identity and fraud rules (Watch); no connectors to banks, PSPs or books."
  },
  "autonomy": {
   "score": 2,
   "note": "Scheduled transactions and automatic matching runs that mark transactions reconciled; posting is still your code or your agent."
  },
  "agent_readiness": {
   "score": 4,
   "note": "A 74-tool Cloud MCP with scoped API keys or OAuth, a CLI with output formats, a public docs MCP and agent-directed docs."
  },
  "integration_ease": {
   "score": 3,
   "note": "REST API, SDKs in TypeScript, Go, Python and Java, Docker setup and a CLI; three services to run when self-hosted."
  },
  "market": {
   "score": 2,
   "note": "Fintech startups and scale-ups, many in emerging markets; Cloud from $100 a month (sandbox) and $1,000 a month (production)."
  }
 },
 "api": {
  "public": true,
  "style": "REST with SDKs (TypeScript, Go, Python, Java) and a CLI; webhooks and transaction hooks",
  "auth": "API key (Core server secret); Cloud API keys and OAuth clients with scopes",
  "limits": "Cloud throughput caps: 5,000 transactions a minute (Sandbox), 100,000 (Production); instant reconciliation up to 10,000 external records per call",
  "cost": "Open source free. Cloud (2026-10-10): Sandbox $100/month incl. 100,000 records then $0.00135 each; Production $1,000/month incl. 200,000 records then $0.00175 each; a record is a ledger, transaction, balance, identity or uploaded external row",
  "export": "Backups to disk or Amazon S3, a documented migration in or out of Blnk Cloud, and a Cloud data lake with SQL queries",
  "notes": "Historical balances since 0.8.4, rebuilt from the nearest daily snapshot plus replay; from_source=true since 0.10.1 replays from genesis. Hash chain since 0.15.0, off by default. Latest release v0.15.6, 28 September 2026."
 },
 "pricing": {
  "model": "Open source free; Cloud per instance per month with included records and metered overage; support plans annual",
  "entry": "$0 self-hosted; Cloud Sandbox $100 a month (5-day free trial)",
  "top": "Production $1,000 a month per instance; Enterprise custom (on-prem, bring your own cloud); Pro Support $5,000 a year",
  "notes": "Records keep counting until an instance is stopped or deleted. Forward-deploy engineering is custom priced."
 },
 "agents": {
  "mcp": "official",
  "ai_features": "No AI features in the ledger itself; agent access through the Cloud MCP and a public docs MCP",
  "agent_writes": "Yes. The Cloud MCP exposes 74 tools including record_transaction, refund_transaction, commit_inflight, bulk_transactions, delete_identity, detokenize_field and matching-rule changes.",
  "approval": "None described; access is limited by mcp:read and mcp:write scopes on the key or OAuth client, proxied through Cloud",
  "notes": "Agents never reach Core directly; Cloud checks credentials and forwards allowed calls. The docs pages embed instructions to agents steering them to a Cloud sandbox rather than self-hosting."
 },
 "integration": {
  "scope": "No pre-built connectors to banks, PSPs or accounting systems; external data arrives as CSV or JSON uploads for reconciliation or as transactions through the API",
  "notes": "Typesense powers search; webhooks and transaction hooks notify your systems; Blnk Watch evaluates fraud rules on transactions."
 },
 "preference": {
  "scale": "Hundreds of developers in over 20 countries (company, 2026-10-10); customer logos include Bakkt, Paystack, Busha and Yousend; 549 GitHub stars and 157 forks (2026-10-10)",
  "ratings": "No G2, Capterra or Trustpilot ratings could be checked (G2 returned 403); none found",
  "sentiment": "Testimonials on the site praise speed of building; not enough independent review data was found to summarize."
 },
 "timeline": [
  {
   "date": "2022-12-16",
   "event": "Blnk repository created"
  },
  {
   "date": "2023",
   "event": "Blnk Finance LLC (earliest copyright year)"
  },
  {
   "date": "unknown",
   "event": "Historical balances (0.8.4) and from_source rebuilds (0.10.1)"
  },
  {
   "date": "unknown",
   "event": "Optional global transaction hash chain (0.15.0)"
  },
  {
   "date": "2026-09-28",
   "event": "Release v0.15.6"
  }
 ],
 "relation": "design-cousin",
 "relation_note": "An open-source ledger whose reconciliation rules are objects you create, run and dry-run against outside records: the shape ownpurse's link rules take. It serves fintech builders, and its agent access writes without a person in the loop.",
 "borrow": "Matching rules as stored objects with field, operator and allowable drift; dry runs; one-to-one and one-to-many strategies with grouping; balances as of any time from a snapshot plus replay, with a from-genesis rebuild to check the snapshot; a hash chain verified by a CLI with a meaningful exit code; inflight as a held, uncollapsed state.",
 "avoid": "Writing reconciliation results into mutable metadata outside the hash chain; an MCP with write and detokenize tools and no approval gate; docs that carry instructions to agents in favour of the vendor's cloud.",
 "sources": [
  {
   "title": "Blnk home page",
   "url": "https://www.blnkfinance.com"
  },
  {
   "title": "Blnk pricing",
   "url": "https://www.blnkfinance.com/pricing"
  },
  {
   "title": "Blnk about page",
   "url": "https://www.blnkfinance.com/about"
  },
  {
   "title": "Blnk Cloud MCP page",
   "url": "https://www.blnkfinance.com/tools/mcp"
  },
  {
   "title": "MCP server reference (74 tools)",
   "url": "https://docs.blnkfinance.com/cloud/reference/mcp.md"
  },
  {
   "title": "Transaction integrity and hashing",
   "url": "https://docs.blnkfinance.com/transactions/hash.md"
  },
  {
   "title": "Reconciliation overview",
   "url": "https://docs.blnkfinance.com/reconciliations/overview.md"
  },
  {
   "title": "Matching rules",
   "url": "https://docs.blnkfinance.com/reconciliations/matching-rules.md"
  },
  {
   "title": "Retrieving historical balances",
   "url": "https://docs.blnkfinance.com/balances/historical-balances.md"
  },
  {
   "title": "Docs index (llms.txt)",
   "url": "https://docs.blnkfinance.com/llms.txt"
  },
  {
   "title": "blnkfinance/blnk on GitHub",
   "url": "https://github.com/blnkfinance/blnk"
  }
 ],
 "unverified": [
  "Funding, investors and headquarters (none disclosed)",
  "Founding year of the company (inferred from the repository date and copyright line)",
  "Release dates for 0.8.4, 0.10.1 and 0.15.0",
  "Which MCP tools fall under mcp:read versus mcp:write (not mapped in the docs)",
  "Whether the customer logos are current production customers",
  "No review-site ratings or press could be checked; web search was unavailable"
 ],
 "researched": "2026-10-10"
}
---

## What it is

Blnk sells "an open-source immutable ledger for building financial products" ([site](https://www.blnkfinance.com)). Blnk Core is a double-entry ledger in Go on PostgreSQL, Apache 2.0 ([GitHub](https://github.com/blnkfinance/blnk)), for teams building wallets, lending, remittances and marketplaces. It does reconciliation, PII tokenization and fraud rules (Blnk Watch) beside the ledger. It does not connect to banks, PSPs or accounting systems: external records come to it as uploads or API calls.

## How it works

Transactions move value between balances. Inflight transactions hold funds and are later committed or voided; scheduled, split, bulk and overdraft transactions are built in. Protected fields (id, source, destination, amount, currency, status, reference, created time) are guarded by a database immutability trigger, and an optional global hash chain, off by default since 0.15.0, seals them in order; `blnk verify-chain` exits 0 when the chain is intact and names where it broke otherwise. Description and metadata are outside the chain ([hashing](https://docs.blnkfinance.com/transactions/hash.md)).

Historical balances answer "what was this balance at time T" from the nearest daily snapshot plus a replay, or from genesis with `from_source=true` ([historical balances](https://docs.blnkfinance.com/balances/historical-balances.md)).

Reconciliation is the part worth studying. You upload a bank statement or processor report as CSV or JSON, then run matching rules with a strategy (one-to-one, one-to-many with grouping). A rule is a stored object: a list of criteria on amount, currency, date, reference or description, each with an operator and an allowable drift (1 percent on amount, 1,800 seconds on date) ([matching rules](https://docs.blnkfinance.com/reconciliations/matching-rules.md)). A `dry_run` tests without saving. Saved matches are written into each transaction's metadata as `reconciled: true` with the external id ([overview](https://docs.blnkfinance.com/reconciliations/overview.md)).

## Data and access

A REST API, SDKs in TypeScript, Go, Python and Java, webhooks and a CLI. Self-hosting runs Postgres, Redis and Typesense under Docker Compose. Backups go to disk or S3, and moving in or out of Blnk Cloud is documented. Cloud costs $100 a month for a sandbox and $1,000 a month for production with 200,000 records, then $0.00175 a record; a record counts ledgers, transactions, balances, identities and uploaded external rows ([pricing](https://www.blnkfinance.com/pricing)).

## Agents

Blnk has gone furthest of the three toward agents. The Cloud MCP exposes 74 tools across ledgers, balances, transactions, identities, reconciliation, hooks and the data lake, including `record_transaction`, `refund_transaction`, `bulk_transactions` and `detokenize_identity` ([MCP reference](https://docs.blnkfinance.com/cloud/reference/mcp.md)). Access is scoped by API key or OAuth and proxied through Cloud; no human approval step is described. A public docs MCP lets coding agents search the docs. The docs pages also carry instructions addressed to agents that steer them toward a Cloud sandbox instead of self-hosting.

## Customers

Logos include Bakkt, Paystack, Busha, Yousend and Public Grid; the company says hundreds of developers in more than 20 countries use it ([about](https://www.blnkfinance.com/about)). The repository has 549 stars (10 October 2026). No review-site ratings or funding were found.

## History

The repository dates from December 2022; Blnk Finance LLC was founded by Jerry Enebeli (CEO) with co-founder Praise Philemon. Releases added historical balances (0.8.4), rebuilds from source (0.10.1) and the hash chain (0.15.0); the latest is 0.15.6 from 28 September 2026.

## For ownpurse

Blnk is a design cousin with one idea to take directly: reconciliation rules as first-class objects with tolerances, strategies and dry runs, which is what ownpurse's link rules between sources should become, with a confidence and a person's review. Its snapshot-plus-replay balances and from-genesis check mirror rebuilding the ledger from the record. The cautions are equally clear: results written into mutable metadata outside the chain, and an MCP that lets an agent post, refund and detokenize with only a scope between it and the books. ownpurse's sealed, approved packages are the answer to that.
