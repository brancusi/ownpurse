---
{
 "slug": "formance",
 "name": "Formance",
 "url": "https://www.formance.com",
 "segment": "fact-ledger",
 "status": "active",
 "founded": "2021",
 "hq": "Paris, France and New York (Y Combinator lists Paris)",
 "backing": "Y Combinator (Summer 2021). $21M Series A co-led by PayPal Ventures and Portage Ventures, with Y Combinator, Hoxton Ventures and Axeleo Capital, announced 29 January 2025.",
 "tagline": "An open-source programmable ledger for fiat and digital assets, with a paid platform of payment connectors, reconciliation, wallets and flows around it.",
 "scope": "Formance Ledger is a double-entry, multi-asset ledger on PostgreSQL with an immutable, optionally hash-chained log, bi-temporal transactions and a DSL (Numscript) for money movement. The Enterprise platform adds pre-built PSP, bank and exchange connectors, reconciliation, wallets, workflows and a console. It is infrastructure for fintechs and platforms that move money, not an accounting system for a business's books.",
 "domains": [
  "ledger",
  "payments",
  "crypto",
  "integration",
  "close"
 ],
 "customers_served": [
  "developers",
  "mid-market",
  "enterprise"
 ],
 "spectrum": {
  "custody": {
   "score": 3,
   "note": "The MIT ledger self-hosts on your own Postgres and the enterprise edition can run in your Kubernetes; the Private Cloud option is operated by Formance."
  },
  "locality": {
   "score": 2,
   "note": "Self-hostable (Go binary plus PostgreSQL); the enterprise stack expects Kubernetes on AWS, GCP or Azure; not aimed at a laptop."
  },
  "history": {
   "score": 4,
   "note": "Immutable log, compensating transactions for corrections, two timestamps per transaction and point-in-time state; HASH_LOGS chains each log or block to the previous one."
  },
  "openness": {
   "score": 3,
   "note": "The ledger and Numscript are MIT; pre-built connectors, reconciliation, wallets and flows are in the paid Enterprise edition."
  },
  "breadth": {
   "score": 2,
   "note": "Ledger plus payments connectivity (PSPs, banks, exchanges), reconciliation and workflows; no accounting-system connectors, documents or tax."
  },
  "autonomy": {
   "score": 2,
   "note": "Flows run multi-step money movements unattended and reconciliation evaluates rules on a schedule; your code still decides what to post."
  },
  "agent_readiness": {
   "score": 3,
   "note": "An MCP server served by the fctl CLI over stdio, read-only by design, plus a CLI and SDKs; no agent write path."
  },
  "integration_ease": {
   "score": 3,
   "note": "REST API, SDKs, fctl and detailed docs; the enterprise stack's Kubernetes deployment and stack versioning add weight."
  },
  "market": {
   "score": 3,
   "note": "Fintechs and scale-ups such as Doctolib, Liberis and Booksy; annual enterprise subscriptions; DORA compliance."
  }
 },
 "api": {
  "public": true,
  "style": "REST (v2 API), with SDKs and the fctl CLI; Numscript for transactions",
  "auth": "OAuth2 client credentials through the stack's auth module [unverified for open-source deployments]",
  "limits": "No published rate limits found; backdated transactions are validated against the recomputed final state only",
  "cost": "Open source free; Enterprise subscription price not published",
  "export": "Full log stream export (fctl ledger export, POST /v2/{ledger}/logs/export) and import into a fresh ledger, which recomputes and verifies the per-log hash chain",
  "notes": "Ledger 2.2 removed the auto-calculated point-in-time that made volume queries flaky with future timestamps. Ledger v2.3.23 released 5 October 2026."
 },
 "pricing": {
  "model": "Open core: MIT ledger free; Enterprise as an annual subscription, Private Cloud or Assisted Self-Hosted",
  "entry": "$0 (open source, self-hosted)",
  "top": "Enterprise annual subscription, price not published",
  "notes": "Enterprise includes pre-built PSP, bank and digital-asset connectors, Wallets, Flows, Reconciliation, enterprise auth, auditing and streaming, a console, a solutions engineer and 24/7 support with 30-minute SEV-1 response; Private Cloud carries a 99.9% SLA."
 },
 "agents": {
  "mcp": "official",
  "ai_features": "No AI features in the product; a run of 2026 blog posts on agentic payments, Know Your Agent and idempotency for agents.",
  "agent_writes": "Not through MCP: the server \"lets an agent inspect financial data and validate Numscript without giving it write access to the stack\". Writes go through the API like any client.",
  "approval": "None built in; the read-only MCP is the safeguard",
  "notes": "Started with fctl stack mcp serve --transport=stdio after enabling the module; reads ledgers, accounts, balances, volumes, transactions, payments and reconciliation state."
 },
 "integration": {
  "scope": "PSP connectors (Stripe, Adyen, Wise and others), exchange connectors (Kraken Pro, September 2026), open banking and a Banking Bridge in Cloud; a Generic Connector with a documented wire contract in the open-source edition",
  "notes": "Connectors to payment rails, not to accounting systems such as Xero or QuickBooks."
 },
 "preference": {
  "scale": "Revenue up 10x in the 12 months before January 2025 (company); 33 people (Y Combinator profile, 2026-10-10); formancehq/ledger has 1,425 stars and 215 forks (2026-10-10)",
  "ratings": "No G2, Capterra or Trustpilot ratings could be checked (G2 returned 403); none found",
  "sentiment": "Not enough public review data was found to summarize. Developers can see the open repository and docs; the paid split of connectors and reconciliation is the evident trade-off."
 },
 "timeline": [
  {
   "date": "2021-05",
   "event": "Ledger repository created"
  },
  {
   "date": "2021",
   "event": "Founded by Clément Salaün and Anne-Sybille Pradelles; Y Combinator Summer 2021"
  },
  {
   "date": "2024-07",
   "event": "Numscript split into its own MIT repository"
  },
  {
   "date": "2025-01-29",
   "event": "$21M Series A co-led by PayPal Ventures and Portage Ventures"
  },
  {
   "date": "2025-12-10",
   "event": "Announces DORA compliance"
  },
  {
   "date": "2026-09",
   "event": "Kraken Pro connector; posts on agentic payments and ledgers"
  },
  {
   "date": "2026-10-05",
   "event": "Ledger v2.3.23; platform released as Stack v3.2"
  }
 ],
 "relation": "design-cousin",
 "relation_note": "The closest open-source cousin to ownpurse's record: an immutable, hash-chained, bi-temporal ledger with reconciliation against outside sources. It serves platforms moving money, not owners gathering their books.",
 "borrow": "Two timestamps per transaction with state as of either; HASH_LOGS plus external anchoring plus reconciliation as three checks that each cover what the others cannot; export and import that recompute the chain; reconciliation results that store a compact proof on pass and fuller evidence on fail; a safety margin before the newest edge of data; alert timelines; an MCP that is read-only by design.",
 "avoid": "Effective volumes that, by its own docs, escape historization; metadata that changes over time; open core that puts connectors and reconciliation behind the paid edition; Kubernetes as the serious deployment.",
 "sources": [
  {
   "title": "Formance home page",
   "url": "https://www.formance.com"
  },
  {
   "title": "Formance pricing",
   "url": "https://www.formance.com/pricing"
  },
  {
   "title": "Series A announcement, 29 January 2025",
   "url": "https://www.formance.com/blog/company/formance-secures-21-million-dollar-in-series-a-funding-co-led-by-paypal-ventures-and-portage-ventures-to-expand-its-open-source-financial-infrastructure"
  },
  {
   "title": "Verifying integrity (HASH_LOGS)",
   "url": "https://docs.formance.com/modules/ledger/advanced/verifying-integrity"
  },
  {
   "title": "Bi-temporality",
   "url": "https://docs.formance.com/modules/ledger/working-with/bi-temporality"
  },
  {
   "title": "MCP server",
   "url": "https://docs.formance.com/mcp"
  },
  {
   "title": "Reconciliation concepts",
   "url": "https://docs.formance.com/modules/reconciliation/concepts"
  },
  {
   "title": "Release policy",
   "url": "https://docs.formance.com/release-policy"
  },
  {
   "title": "Y Combinator company profile",
   "url": "https://www.ycombinator.com/companies/formance"
  },
  {
   "title": "formancehq/ledger on GitHub",
   "url": "https://github.com/formancehq/ledger"
  },
  {
   "title": "Formance blog",
   "url": "https://www.formance.com/blog"
  }
 ],
 "unverified": [
  "Any seed round before the Series A (not mentioned by the company)",
  "Earlier name of the project (believed to be Numary) was not confirmed",
  "Authentication model of a bare open-source ledger deployment",
  "Enterprise subscription prices (not published)",
  "No review-site ratings or press beyond the company's own announcement could be checked; web search was unavailable"
 ],
 "researched": "2026-10-10"
}
---

## What it is

Formance describes itself as "the open-source programmable ledger for fiat and digital assets" ([site](https://www.formance.com)). The core is Formance Ledger, a double-entry, multi-asset ledger in Go on PostgreSQL, licensed MIT ([GitHub](https://github.com/formancehq/ledger)). Around it the company sells a platform: connectivity to PSPs, banks and exchanges, reconciliation, wallets, workflows (Flows) and a console. Its customers move money: marketplaces, lenders, platforms. It is not a general ledger for a company's books and has no connectors to accounting systems.

## How it works

Transactions are atomic and immutable, written in Numscript, a small language for splits, fees and multi-party movements. A mistake is fixed by reverting it with a compensating transaction. Each transaction carries two times, when it happened and when it was inserted, and "the ledger state is a snapshot of the ledger at the requested point in time" ([bi-temporality](https://docs.formance.com/modules/ledger/working-with/bi-temporality)). The docs are candid about the edges: a backdated transaction is checked only against the recomputed final state, and effective volumes "are the only data that escapes the control of historization". Metadata is historized but can change over time.

The log can be tamper-evident. HASH_LOGS chains each log to the previous one (by trigger at insert) or each block to the previous block (by a worker on a schedule). The docs say what the chain cannot catch, a rewrite that recomputes every hash after it, and prescribe three checks together: recompute the chain, anchor hashes outside the database, and reconcile against systems someone else holds ([verifying integrity](https://docs.formance.com/modules/ledger/advanced/verifying-integrity)). Exporting the full log and importing it into a fresh ledger re-verifies the chain through the API.

Reconciliation (Enterprise) evaluates rules per asset on daily, weekly, monthly or continuous periods, keeps a compact proof when a check passes and fuller evidence when it fails, waits a safety margin before the newest data, and keeps an event timeline on each alert ([concepts](https://docs.formance.com/modules/reconciliation/concepts)).

## Data and access

The REST v2 API, SDKs and the `fctl` CLI cover everything; log export is a single call. The open-source edition includes the ledger, Numscript, a Generic Connector, CLI and SDKs. Enterprise, sold annually as Private Cloud or Assisted Self-Hosted on your Kubernetes, adds pre-built connectors, Wallets, Flows, Reconciliation, auditing and 24/7 support ([pricing](https://www.formance.com/pricing)). No prices are published.

## Agents

Formance ships an MCP server through `fctl stack mcp serve`. It is read-only on purpose: it "lets an agent inspect financial data and validate Numscript without giving it write access to the stack" ([MCP](https://docs.formance.com/mcp)). An agent can read ledgers, balances, payments and reconciliation state and check a Numscript before a person promotes it. That is close to ownpurse's "agents propose; people post", minus a recorded approval step.

## Customers

The Series A announcement names Doctolib, Liberis, Booksy and Btech and says revenue grew tenfold in the prior year ([announcement](https://www.formance.com/blog/company/formance-secures-21-million-dollar-in-series-a-funding-co-led-by-paypal-ventures-and-portage-ventures-to-expand-its-open-source-financial-infrastructure)). Y Combinator lists 33 people ([YC](https://www.ycombinator.com/companies/formance)). No review-site ratings were found.

## History

Founded in 2021 by Clément Salaün and Anne-Sybille Pradelles, in Y Combinator's Summer 2021 batch. The ledger repository dates from May 2021. A $21 million Series A co-led by PayPal Ventures and Portage Ventures followed in January 2025, and DORA compliance in December 2025. In September 2026 it added a Kraken Pro connector and published a series on ledgers for agentic payments.

## For ownpurse

Formance is the nearest open-source cousin of ownpurse's record: an immutable log, a hash chain, two times per event, and reconciliation against an outside witness, with docs that say plainly what each check proves. The difference is who it serves and where it runs. Formance is a platform's ledger in Postgres and Kubernetes; ownpurse is an owner's record in one SQLite file, built from sources the owner does not control. Borrow its three-check integrity model, its export-and-verify path, its pass and fail evidence, and its read-only MCP. Avoid data that escapes history, and putting the connectors behind the paid door.
