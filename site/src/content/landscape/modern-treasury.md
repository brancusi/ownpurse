---
{
 "slug": "modern-treasury",
 "name": "Modern Treasury",
 "url": "https://www.moderntreasury.com",
 "segment": "fact-ledger",
 "status": "active",
 "founded": "2018",
 "hq": "San Francisco, California",
 "backing": "Venture-backed; $85M Series C announced 6 October 2021 led by Altimeter Capital with Benchmark and Quiet Capital, valuing the company at over $2B. Earlier rounds not confirmed this pass. Applied for a US national trust bank charter (announced on its site, 2026).",
 "tagline": "A payments operations platform with an API for moving money and an immutable double-entry Ledgers product.",
 "scope": "Modern Treasury moves money across ACH, wires, RTP, FedNow, checks, push-to-card and stablecoins, handles KYC and KYB onboarding, reconciliation and reporting, and offers Ledgers, a managed double-entry database for balances and transactions. It is built for companies that move money in their own products (fintechs, marketplaces, payroll, lending), not for keeping a small business's or a household's books, and it does not do tax, documents or general accounting.",
 "domains": [
  "ledger",
  "payments",
  "bank",
  "crypto"
 ],
 "customers_served": [
  "mid-market",
  "enterprise",
  "developers"
 ],
 "spectrum": {
  "custody": {
   "score": 1,
   "note": "Your ledger lives in Modern Treasury's cloud; you can read everything through the API to keep your own copy."
  },
  "locality": {
   "score": 0,
   "note": "Cloud only."
  },
  "history": {
   "score": 3,
   "note": "Posted entries cannot change or be deleted, reversals are new transactions, effective_at is separate from posting time, and the entries behind each account lock_version can be queried; pending transactions and metadata can still change."
  },
  "openness": {
   "score": 2,
   "note": "Closed service, but a public API with an MIT-licensed OpenAPI spec and SDKs."
  },
  "breadth": {
   "score": 2,
   "note": "Payments, accounts, onboarding, reconciliation and ledger in one API; no documents, tax or general books."
  },
  "autonomy": {
   "score": 1,
   "note": "Programmatic by design; it auto-matches payments to expectations, but the bookkeeping logic is the customer's code."
  },
  "agent_readiness": {
   "score": 4,
   "note": "Official MCP server (npm modern-treasury-mcp, v4.21.0 in September 2026), SDKs in six languages, OpenAPI spec and a sandbox."
  },
  "integration_ease": {
   "score": 3,
   "note": "Well-documented API and sandbox, but sales-led annual contracts with a minimum commitment."
  },
  "market": {
   "score": 3,
   "note": "Fintechs and mid-market to enterprise companies that move money (Navan, Procore, Masterworks, Parafin)."
  }
 },
 "api": {
  "public": true,
  "style": "REST JSON with webhooks; OpenAPI spec; SDKs for Python, Node.js, Go, Kotlin, Java and Ruby",
  "auth": "API keys per organization (HTTP Basic with organization id and key, as used by the SDKs and MCP server configuration)",
  "limits": "Production 100 requests a second for non-list endpoints and 20 for list endpoints; sandbox 25 and 5; 20 synchronous ledger entries per ledger account per second; bulk requests 1 a second",
  "cost": "Ledgers billed in usage tiers; platform fee plus per-payment fees by rail; all within an annual minimum commitment; no public figures",
  "export": "Everything through the API and reporting; no bulk owner-held export format documented on the pages read",
  "notes": "Idempotency keys on all POST requests; optimistic locking by referencing an account's lock_version; Read Committed isolation; ledger transactions can be linked to payment orders, incoming payments, expected payments, returns and reversals."
 },
 "pricing": {
  "model": "Sales-led annual contract: platform fee plus usage-based fees, applied to one minimum commitment",
  "entry": "Not published; contact sales",
  "top": "Not published",
  "notes": "Per-unit costs fall with volume; on and off-ramp fees combine a rail fee and a conversion fee; idle balances can earn interest through banking partners (rate unstated)."
 },
 "agents": {
  "mcp": "official",
  "ai_features": "An official MCP server generated from the API; \"Open in ChatGPT\" and \"Open in Claude\" buttons on articles; an internal Bank Operations Agent (July 2026); research on how well models know double-entry accounting (January 2026).",
  "agent_writes": "The MCP server exposes the API to assistants using an API key, so an agent can create payment orders and ledger transactions with whatever that key allows.",
  "approval": "No approval gate in the MCP server found; the platform has its own approval workflows for payments, not checked in detail. Internally, a human reviews anything their bank operations agent sends out.",
  "notes": "Their own January 2026 tests concluded \"All model output needs to be double-checked by a human to ensure accuracy.\""
 },
 "integration": {
  "scope": "Direct connections to US banks for payments and statements, plus bring-your-own-bank; FedNow certified; stablecoin rails; webhooks to customer systems.",
  "notes": "Reconciliation of bank activity to expected payments is a core feature; general-ledger sync to accounting tools not checked this pass."
 },
 "preference": {
  "scale": "$600 billion in payments processed (site, October 2026); monthly volume over $2B at the time of the Series C (October 2021); no customer count published.",
  "ratings": "G2 and Capterra pages blocked automated reading; ratings not confirmed.",
  "sentiment": "Customer quotes on its site praise integration speed (\"up and running in days, not months\"). Independent review-site and forum sentiment was not checked this pass."
 },
 "timeline": [
  {
   "date": "2018",
   "event": "Founded in San Francisco by Matt Marcus, Dimitri Dadiomov and Sam Aarons"
  },
  {
   "date": "2021-10-06",
   "event": "$85M Series C led by Altimeter at over $2B; monthly volume above $2B"
  },
  {
   "date": "2026-01-08",
   "event": "Publishes tests of how well LLMs know double-entry accounting"
  },
  {
   "date": "2026-07-24",
   "event": "Writes up its internal Bank Operations Agent, human review on anything it sends"
  },
  {
   "date": "2026-09",
   "event": "Official MCP server at version 4.21.0 on npm"
  },
  {
   "date": "2026",
   "event": "Announces an application for a national trust bank charter; FedNow certification completed (date unconfirmed)"
  }
 ],
 "relation": "design-cousin",
 "relation_note": "Modern Treasury Ledgers is the commercial version of a ledger that never edits a posted entry, built for companies moving money in their products. ownpurse borrows its ledger guarantees; a business that uses Modern Treasury could also be a source for ownpurse, but the two serve different people.",
 "borrow": "Posted entries never change and corrections are reversals that point both ways (reverses and reversed_by); effective time kept apart from posting time; pending, posted and available balances; idempotency keys on every write; optimistic locking on an account version, which suits ownpurse's before-fingerprint check on sealed packages; links from a ledger transaction to the payment or document that caused it.",
 "avoid": "A ledger only reachable in the vendor's cloud; editable metadata and mutable pending transactions inside the system of record; an MCP server that hands an agent a full-power API key with no approval step, where ownpurse requires a sealed package a person approves.",
 "sources": [
  {
   "title": "Modern Treasury",
   "url": "https://www.moderntreasury.com/"
  },
  {
   "title": "Modern Treasury pricing",
   "url": "https://www.moderntreasury.com/pricing"
  },
  {
   "title": "Ledgers overview",
   "url": "https://docs.moderntreasury.com/ledgers/docs/ledgers-overview"
  },
  {
   "title": "Ledgers guarantees",
   "url": "https://docs.moderntreasury.com/ledgers/docs/ledgers-guarantees"
  },
  {
   "title": "Ledger transaction object",
   "url": "https://docs.moderntreasury.com/ledgers/reference/ledger-transaction-object"
  },
  {
   "title": "Ledger account object",
   "url": "https://docs.moderntreasury.com/ledgers/reference/ledger-account-object"
  },
  {
   "title": "Rate limits",
   "url": "https://docs.moderntreasury.com/platform/reference/rate-limits"
  },
  {
   "title": "Modern Treasury on GitHub",
   "url": "https://github.com/Modern-Treasury"
  },
  {
   "title": "modern-treasury-node (MCP server)",
   "url": "https://github.com/Modern-Treasury/modern-treasury-node"
  },
  {
   "title": "modern-treasury-mcp on npm",
   "url": "https://registry.npmjs.org/modern-treasury-mcp/latest"
  },
  {
   "title": "Announcing our Series C (6 October 2021)",
   "url": "https://www.moderntreasury.com/journal/announcing-our-series-c"
  },
  {
   "title": "What we learned building a bank operations agent (24 July 2026)",
   "url": "https://www.moderntreasury.com/journal/what-we-learned-building-a-bank-operations-agent"
  },
  {
   "title": "How well do LLMs know double-entry accounting? (8 January 2026)",
   "url": "https://www.moderntreasury.com/journal/how-well-do-llms-know-double-entry-accounting"
  },
  {
   "title": "About Modern Treasury",
   "url": "https://www.moderntreasury.com/about"
  }
 ],
 "unverified": [
  "Founding year 2018 (from general knowledge; not on the pages read)",
  "Series A and B amounts and dates, and total funding",
  "Whether Modern Treasury acquired Beam (a stablecoin company) in 2025: the guessed announcement URL returned 404",
  "Dates of the FedNow certification and the national trust bank application",
  "Ledgers launch date",
  "Any Ledgers pricing figures or free tier",
  "G2, Capterra and Trustpilot ratings (pages returned 403; web search budget exhausted)",
  "Whether API keys can be scoped read-only, and whether the MCP server can be restricted to read tools",
  "Whether balances can be read as of a past effective_at through the API (fields effective_at_lower_bound and upper_bound appear in examples)"
 ],
 "researched": "2026-10-10"
}
---

## What it is

[Modern Treasury](https://www.moderntreasury.com) sells one API for moving money: onboarding with KYC and KYB, accounts and sub-accounts, payments across ACH, RTP, FedNow, checks, wires, push-to-card and stablecoins, and reconciliation and reporting. Its Ledgers product is "a fully managed cloud database for recording financial transactions and balances" ([Ledgers overview](https://docs.moderntreasury.com/ledgers/docs/ledgers-overview)), and the company argues that an "immutable, scalable, double-entry ledger is the gold standard" for high-volume payment flows. The site claims $600 billion processed and 99.99% uptime.

## How it works

Ledgers has ledgers, accounts, transactions and entries, with explicit [guarantees](https://docs.moderntreasury.com/ledgers/docs/ledgers-guarantees): "Entries cannot be changed or removed on a Ledger Transaction once it is posted." Transactions "cannot be deleted, only archived," and posted ones cannot be archived. A correction is a reversal, and the [transaction object](https://docs.moderntreasury.com/ledgers/reference/ledger-transaction-object) links both ways with `reverses_ledger_transaction_id` and `reversed_by_ledger_transaction_id`. Each transaction has a status (pending, posted or archived), an `effective_at` for reporting, a `posted_at`, and an optional link to the payment that caused it. Accounts carry pending, posted and available balances and a `lock_version`; you can pass that version on a write so it fails if the balance moved, and "the exact Entries that contribute to each lock_version of an account can always be queried." Every POST takes an idempotency key. What can still change: pending transactions and string metadata.

## Data and access

The API is REST with webhooks, an MIT-licensed OpenAPI spec and SDKs in Python, Node, Go, Kotlin, Java and Ruby ([GitHub](https://github.com/Modern-Treasury)). [Rate limits](https://docs.moderntreasury.com/platform/reference/rate-limits) are 100 requests a second in production (20 for lists) and 20 synchronous entries per ledger account per second. The data lives in their cloud; you can mirror it through the API. [Pricing](https://www.moderntreasury.com/pricing) is unpublished: a platform fee plus usage fees by rail, inside an annual minimum commitment.

## Agents

The Node SDK repository ships an [official MCP server](https://github.com/Modern-Treasury/modern-treasury-node), `modern-treasury-mcp`, at version 4.21.0 in September 2026, configured with an API key and organization id. It exposes the API, writes included; no approval gate is described. The company has also written up an internal [bank operations agent](https://www.moderntreasury.com/journal/what-we-learned-building-a-bank-operations-agent) (July 2026) where "a human reviews anything that leaves the agent's own state," and [tested models on double-entry](https://www.moderntreasury.com/journal/how-well-do-llms-know-double-entry-accounting) (January 2026), concluding that all output needs a human check.

## Customers

Customer stories name Navan, Masterworks, Parafin and Procore ([about](https://www.moderntreasury.com/about)). The buyers are engineering and finance teams at companies whose product moves money, from startups to large firms. Review-site ratings could not be read this pass.

## History

Founded in 2018 by Matt Marcus, Dimitri Dadiomov and Sam Aarons, it raised an [$85M Series C](https://www.moderntreasury.com/journal/announcing-our-series-c) on 6 October 2021 at over $2B, when monthly volume had grown from about $10M two years earlier to over $2B. Since then it has added stablecoin rails and FedNow, and in 2026 announced it had applied to establish a national trust bank.

## For ownpurse

Modern Treasury shows that the ledger rules ownpurse keeps (never edit, reverse instead, keep two times, lock on a version) are what serious money-movement companies pay for. Its customers are building products, not keeping their own books, so it is not a competitor; a business that runs payments through it could be one more source. The contrast is in custody and agents: the ledger sits in their cloud, and the MCP server gives an agent the API key's full power. ownpurse keeps the record on the owner's machine and lets an agent propose a sealed package that a person approves, which is the same human-review rule Modern Treasury applies to its own internal agent.
