---
{
 "slug": "layer",
 "name": "Layer",
 "url": "https://www.layerfi.com",
 "segment": "ai-ledger",
 "status": "active",
 "founded": "Unconfirmed (2020 or 2023, sources disagree)",
 "hq": "San Francisco, California, US",
 "backing": "Venture-backed. USD 2.3M led by Costanoa Ventures (May 2024, labelled a Series A by one outlet); USD 6.6M seed led by Emergence Capital with Better Tomorrow Ventures (July 2025). Total about USD 8.9M.",
 "tagline": "An embedded accounting API and white-label bookkeeping service that vertical software platforms ship as their own ledger.",
 "scope": "Layer gives a software platform (a medspa system, a coffee shop POS, a lending app) a double-entry general ledger per business customer, React components and a REST API to show it, an AI categorization engine, and an optional full-service bookkeeping team under the platform's brand. It sells to platforms, not to business owners, and is not available self-serve; it does not hold money or run payments.",
 "domains": [
  "ledger",
  "bank",
  "invoicing",
  "documents",
  "close",
  "reporting",
  "tax"
 ],
 "customers_served": [
  "smb",
  "developers"
 ],
 "spectrum": {
  "custody": {
   "score": 1,
   "note": "Layer holds each business's ledger; the platform, not the owner, can export ledger entries, reports and transactions as CSV or Excel."
  },
  "locality": {
   "score": 0,
   "note": "Cloud only, hosted by Layer."
  },
  "history": {
   "score": 2,
   "note": "Changes to payments, payouts, refunds and journal entries are made by reversing old ledger entries and posting new ones, but source records are still updated and deleted."
  },
  "openness": {
   "score": 1,
   "note": "Detailed public docs, but API credentials and sandbox access only through sales; closed source."
  },
  "breadth": {
   "score": 3,
   "note": "Ledger, bank feeds via Plaid, AR invoices and payments, AP bills, payroll, loans and assets imports, receipts, tax reports and a bookkeeping service; no banking or payments of its own."
  },
  "autonomy": {
   "score": 3,
   "note": "AI categorization of bank transactions with human bookkeepers reviewing and closing each month on the full-service package."
  },
  "agent_readiness": {
   "score": 3,
   "note": "An MCP server with about 190 tools, read and write, business- and platform-scoped tokens, idempotency and llms.txt docs; no CLI and gated access."
  },
  "integration_ease": {
   "score": 2,
   "note": "Well documented with drop-in components and a 2 to 3 week typical integration, but not self-serve: sandbox and credentials come from the sales team."
  },
  "market": {
   "score": 2,
   "note": "Small businesses, reached through vertical and horizontal SaaS platforms and SMB fintechs."
  }
 },
 "api": {
  "public": false,
  "style": "REST (v1), JSON, idempotency keys, pagination, webhooks with an event catalog, async exports, plus embeddable React components",
  "auth": "OAuth 2.0 client credentials for the platform (client_id and client_secret issued by Layer, tokens expire in 3,600 seconds), and business-scoped tokens for client-side use",
  "limits": "Token bucket by endpoint cost: 500 requests per second in production and 100 in sandbox, initial bucket 100, refilled every second; some endpoints have their own limits",
  "cost": "Not published; contact sales. No per-call metering found in the docs.",
  "export": "Ledger entries, ledger balances, transactions, P&L, balance sheet and cash flow as CSV or Excel through the API; async exports for large reports; tax document export. These are available to the platform, not directly to the business owner.",
  "notes": "Docs are public at docs.layerfi.com, but access is not: 'reach out to our team' for a sandbox. Includes a QuickBooks Online sync-in, trial balance import and a demo-business generator."
 },
 "pricing": {
  "model": "B2B contract with each platform; packages Core Accounting, Advanced Accounting and Full-Service Bookkeeping",
  "entry": "Not published",
  "top": "Not published",
  "notes": "No prices on the site; every path leads to Contact Sales. How the platform charges its own SMBs is up to the platform."
 },
 "agents": {
  "mcp": "official",
  "ai_features": "AI categorization engine for bank transactions, suggested categorization rules, an AI-supported bookkeeper that SMBs can chat with (seed announcement, July 2025), and an MCP server for platforms' agents and for clients such as Claude.",
  "agent_writes": "Yes. In business mode the MCP exposes accounting mutations: categorizing transactions, imports, voids, journal entries and more. Platform mode is mostly read-only apart from creating and updating businesses.",
  "approval": "No approval gate. Tool annotations (readOnlyHint, destructiveHint) are described as hints, not authorization; destructive tools are named separately (void_, delete_, reverse_ and so on). Plaid linking hands a hosted URL to a human.",
  "notes": "About 190 tools; clients are told to use tool search, with a few always-loaded tools such as get_financial_report and categorize_bank_transaction. Amounts are integer cents. Capability groups (core, reporting, accounting, banking_imports, ar_imports, ap_imports, loans_assets, qbo) limit what is registered."
 },
 "integration": {
  "scope": "Plaid for bank accounts, Stripe and the platform's own payments, payroll, bills, loans and assets imports, QuickBooks Online sync-in, receipts upload; named platform customers Moxie, Nav, Duet and Dripos.",
  "notes": "The integration is the platform's job: it pushes its own sales, payouts and bills into Layer through the import APIs, and Layer validates backfills against source totals."
 },
 "preference": {
  "scale": "Over 30,000 SMBs across its platforms, with the platform customer base quadrupled in a year (July 2025 announcement). Moxie reports over 40% bookkeeping adoption among new medspas; Nav and Duet over 25% within three months.",
  "ratings": "No G2, Capterra or Trustpilot listing found.",
  "sentiment": "No independent reviews found. The available evidence is platform case studies on Layer's own site, which report adoption rates and launch times of under four weeks."
 },
 "timeline": [
  {
   "date": "2024-05",
   "event": "Raises USD 2.3M led by Costanoa Ventures to embed accounting in vertical software"
  },
  {
   "date": "2025-07",
   "event": "USD 6.6M seed led by Emergence Capital with Better Tomorrow Ventures; over 30,000 SMBs"
  },
  {
   "date": "2026",
   "event": "Docs list an MCP server with about 190 tools for platform and business agents"
  }
 ],
 "relation": "neighbour",
 "relation_note": "Layer sells ledgers to software platforms, not to owners, so it does not compete for ownpurse's person. It is the opposite answer to the same question: the books live inside whatever software the business already uses, held by Layer, rather than in a file the owner holds. A Layer-powered book could become a source only if its platform lets the owner export.",
 "borrow": "Corrections by reversal rather than edit; validating a backfill against source totals; idempotency keys on every write; integer cents; tool groups and always-loaded tools for a large MCP surface; naming destructive tools so they stand apart.",
 "avoid": "Agent writes with no approval step and tool annotations that are only hints; a ledger the owner cannot reach except through the platform; access only through sales.",
 "sources": [
  {
   "title": "Layer homepage",
   "url": "https://www.layerfi.com/"
  },
  {
   "title": "Layer product",
   "url": "https://www.layerfi.com/product"
  },
  {
   "title": "Layer customers",
   "url": "https://www.layerfi.com/customers"
  },
  {
   "title": "Layer docs overview",
   "url": "https://docs.layerfi.com/overview"
  },
  {
   "title": "Layer key concepts",
   "url": "https://docs.layerfi.com/guides/key-concepts.md"
  },
  {
   "title": "Layer authentication",
   "url": "https://docs.layerfi.com/api-details/authentication.md"
  },
  {
   "title": "Layer rate limiting",
   "url": "https://docs.layerfi.com/api-details/rate-limiting.md"
  },
  {
   "title": "Layer MCP server",
   "url": "https://docs.layerfi.com/insights/mcp-server.md"
  },
  {
   "title": "Layer docs index",
   "url": "https://docs.layerfi.com/llms.txt"
  },
  {
   "title": "Layer async reports",
   "url": "https://docs.layerfi.com/guides/async-reports.md"
  },
  {
   "title": "Pulse 2.0: Layer USD 6.6M seed (July 2025)",
   "url": "https://pulse2.com/layer-6-6-million-seed-raised-for-embedded-accounting-platform"
  },
  {
   "title": "GlobeNewswire: Layer raises USD 6.6M",
   "url": "https://www.globenewswire.com/news-release/2025/07/09/3112479/0/en/Layer-Raises-6-6M-Led-by-Emergence-Capital-to-Bring-Embedded-Accounting-to-the-SMB-Software-Stack.html"
  },
  {
   "title": "Entrepreneur Loop: Layer raises USD 2.3M (May 2024)",
   "url": "https://entrepreneurloop.com/accounting-startup-layer-raises-2-3-million-to-challenge-quickbooks-with-automated-software/"
  }
 ],
 "unverified": [
  "Founding year: one outlet says 2020, a profile says 2023",
  "Co-founder Daniel O'Neel (named in a third-party profile only)",
  "Whether the May 2024 USD 2.3M round was a Series A",
  "Pricing for platforms",
  "Launch date of the MCP server",
  "Current SMB count after July 2025"
 ],
 "researched": "2026-10-10"
}
---

## What it is

Layer is accounting infrastructure for software companies. A platform that serves small businesses, such as a medspa system, a nurse practitioner network or a coffee shop POS, embeds Layer and offers bookkeeping inside its own product, under its own brand. Layer's site says it "fully replaces external accounting tools like QuickBooks." Its named customers are [Moxie, Nav, Duet and Dripos](https://www.layerfi.com/customers). In July 2025 it said it served over 30,000 SMBs across its platforms.

## How it works

When a platform onboards a business, Layer provisions a [double-entry general ledger](https://docs.layerfi.com/guides/key-concepts.md) for it. The platform pushes its own data in (sales, payouts, refunds, bills, payroll, loans, assets) and Layer connects bank accounts through Plaid. An AI categorization engine sorts transactions; rules and suggested rules sit on top. Platforms pick a package: Core Accounting (cash basis), Advanced Accounting (accrual, balance sheet, cash flow) or Full-Service Bookkeeping, where Layer's own bookkeepers close the books each month and the owner only answers questions. The UI comes as React components or the platform builds its own on the API. Layer says a typical integration takes two to three weeks.

## Data and access

The [docs](https://docs.layerfi.com/overview) are public and thorough, but access is not: credentials and a sandbox come from the sales team. Auth is [OAuth client credentials](https://docs.layerfi.com/api-details/authentication.md), with business-scoped tokens for browsers. The [rate limit](https://docs.layerfi.com/api-details/rate-limiting.md) is generous, 500 requests a second in production. Ledger entries, balances, transactions and statements can be exported as CSV or Excel, and large reports run as async exports. Corrections are made by reversal: updating a payout or refund reverses its old ledger entries and posts new ones, and journal entries have a reverse endpoint. Prices are not published.

## Agents

Layer's [MCP server](https://docs.layerfi.com/insights/mcp-server.md) registers about 190 tools. In business mode an agent can categorize transactions, import data, void documents and post journal entries; in platform mode it mostly reads and switches between businesses. Destructive tools are named separately, but the docs say plainly that annotations are hints, not authorization. There is no approval step; the platform decides what its agents may do.

## Customers

The customers are platforms, and the evidence is their adoption: over 40% of new medspas on Moxie take bookkeeping, and Nav and Duet saw over 25% in three months. There are no independent reviews on G2, Capterra or Trustpilot. Business owners may never know Layer's name.

## History

Layer raised [USD 2.3M led by Costanoa](https://entrepreneurloop.com/accounting-startup-layer-raises-2-3-million-to-challenge-quickbooks-with-automated-software/) in May 2024 and a [USD 6.6M seed led by Emergence Capital](https://pulse2.com/layer-6-6-million-seed-raised-for-embedded-accounting-platform) in July 2025, with CEO Justin Meretab. It follows the embedded-finance playbook of the 2020s (payments, then cards, then lending inside vertical software) and applies it to the ledger.

## For ownpurse

Layer is a neighbour. It answers the scattered-data problem by putting the books inside the software a business already runs, held by Layer and reached through the platform. ownpurse answers it by putting the record in the owner's hands. Several of Layer's choices are worth copying: corrections by reversal, validating backfills against source totals, idempotency keys, integer cents, and grouping a large MCP surface so agents load only what they need. Its agent writes are the part to avoid: 190 tools, write access in business mode, and no approval step between an agent and the ledger.
