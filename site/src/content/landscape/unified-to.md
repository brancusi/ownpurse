---
{
 "slug": "unified-to",
 "name": "Unified.to",
 "url": "https://unified.to",
 "segment": "unified-api",
 "status": "active",
 "founded": "Not confirmed (Techstars Toronto 2023 cohort)",
 "hq": "Toronto, Canada",
 "backing": "Pre-seed CAD 1.1M led by LOI Venture and DevCap with Techstars, Alumni Ventures and Gurtin Ventures (October 2024); about CAD 1.4M total reported in December 2024. Techstars and Creative Destruction Lab alumni.",
 "tagline": "A real-time, zero-storage unified API and MCP server across 34 categories and 1,250+ integrations, billed by API call.",
 "scope": "Unified.to gives software and AI companies one API over 1,250+ integrations in 34 categories, including 228 accounting integrations with journals, transactions, accounts, invoices and read-only trial balance, balance sheet and P&L. Every call goes live to the source with no cache, and it says it never stores end-customer data. It offers native and virtual webhooks, raw fields, a passthrough API, and an MCP server with 20,000+ tools. It keeps no history and no ledger.",
 "domains": [
  "integration",
  "ledger",
  "bank",
  "payments",
  "commerce"
 ],
 "customers_served": [
  "developers",
  "smb",
  "mid-market",
  "enterprise"
 ],
 "spectrum": {
  "custody": {
   "score": 1,
   "note": "Never stores end-customer data; the record stays with the source and any copy is the integrating app's."
  },
  "locality": {
   "score": 1,
   "note": "Hosted in US, EU and AU regions; on-prem or single-tenant only on the Enterprise plan."
  },
  "history": {
   "score": 0,
   "note": "Live calls only, nothing stored between virtual-webhook checks; no versions or as-of reads."
  },
  "openness": {
   "score": 2,
   "note": "Closed service with a public API, SDKs in seven languages and open sample MCP code."
  },
  "breadth": {
   "score": 3,
   "note": "Accounting with bank feeds, payments, commerce, storage and 30 more categories; no bank aggregation of its own."
  },
  "autonomy": {
   "score": 2,
   "note": "Writes when the integrating app or agent calls; no unattended flows of its own."
  },
  "agent_readiness": {
   "score": 4,
   "note": "Hosted MCP (streamable HTTP, 2026-07-28 revision), tool and permission filters, agent skills and a Claude Code plugin."
  },
  "integration_ease": {
   "score": 3,
   "note": "30-day free trial, sandbox with mock data, broad docs; paid plans start at $750 a month."
  },
  "market": {
   "score": 3,
   "note": "Sells to software and AI companies of all sizes, with HIPAA BAA, SSO and private-cloud options."
  }
 },
 "api": {
  "public": true,
  "style": "REST/JSON unified models per category; fields=raw returns the vendor's original object and a raw object can be written; a passthrough API for native endpoints.",
  "auth": "Workspace API key (bearer) plus a connection id per end customer, created through an embedded authorization component; region-specific hosts.",
  "limits": "Plan rate limits of 5,000 / 7,500 / 10,000 (unit not stated on the pricing page); provider limits apply beneath.",
  "cost": "Usage-based: Grow $750+/month with 750,000 calls ($1.00 per 1,000 over), Pro $1,500+ with 2M ($0.75), Scale $3,000+ with 6M ($0.50); unlimited connections. Each MCP tool call is one API request; empty virtual-webhook checks are free, each delivered page is one call.",
  "export": "Nothing to export from Unified: it stores no data. Log retention is 60 days (365+ on Scale).",
  "notes": "Accounting objects (29) include account, transaction, journal, invoice, bill, trialbalance, balancesheet, profitloss, cashflow, bankfeedaccount and bankfeedtransaction; journals, transactions and accounts support full CRUD."
 },
 "pricing": {
  "model": "Monthly or yearly (17% off) subscription with an included call allowance and per-1,000 overage; connections unlimited",
  "entry": "Grow: $750+/month, 750,000 API calls, 30-day free trial",
  "top": "Scale $3,000+/month with 6M calls; Enterprise custom, usage- or connection-based, with private cloud or on-prem",
  "notes": "From the pricing page as read 2026-10-10. Data-region sync, BAA and 365-day logs need Scale; SAML SSO and Datadog logs need Pro."
 },
 "agents": {
  "mcp": "official",
  "ai_features": "Hosted MCP in three regions with 20,000+ tools across 1,250+ integrations, scoped to one connection per request; optional PII stripping; a Core MCP server for building (connections, webhooks, logs); agent skills via npx skills add; a Claude Code plugin; llms.txt docs.",
  "agent_writes": "Yes: tools both read and act (e.g. update a record, post a note).",
  "approval": "No human approval described. Admins can restrict object-level read/write per connection and limit which tools an agent sees via the permissions and tools parameters.",
  "notes": "Positions itself for AI companies (RAG sync, agents over warehouse data, a Datastore API from June 2026). Scoping is configuration, not a gate on each write."
 },
 "integration": {
  "scope": "1,250+ integrations across 34 unified APIs; 228 accounting integrations including Xero, QuickBooks Online and Desktop, NetSuite, Sage Intacct/50/100/200/X3, Zoho Books, Business Central, Exact Online, Odoo, DATEV and Fortnox.",
  "notes": "Native and virtual webhooks; virtual webhooks poll on an interval you set (minimum 1 minute), retry for about two weeks, and emit deletes only where the source exposes them."
 },
 "preference": {
  "scale": "No totals published. Vendor press release of 8 December 2025: API usage up 550% in six months, customers up 170% and ARR up 375% over a year, 25% month-over-month call growth.",
  "ratings": "Not confirmed: G2 refused access and no Trustpilot page exists.",
  "sentiment": "Not confirmed from independent reviews in this pass. Awards: Best Business Software API at the 2026 API Awards (vendor-reported)."
 },
 "timeline": [
  {
   "date": "2023",
   "event": "Techstars Toronto cohort"
  },
  {
   "date": "2024",
   "event": "Creative Destruction Lab graduate; SXSW 2024 pitch finalist"
  },
  {
   "date": "2024-10",
   "event": "CAD 1.1M pre-seed led by LOI Venture and DevCap"
  },
  {
   "date": "2024-12",
   "event": "About CAD 1.4M raised in total (reported on The SaaS CFO podcast)"
  },
  {
   "date": "2025-12-08",
   "event": "Reports 550% API usage growth in six months; MCP server announced"
  },
  {
   "date": "2026-03-03",
   "event": "Assessment and Advertising APIs added"
  },
  {
   "date": "2026-06-30",
   "event": "Unified Datastore API over spreadsheets, databases and warehouses"
  },
  {
   "date": "2026-07-28",
   "event": "MCP v2 on the stateless 2026-07-28 protocol revision in production"
  }
 ],
 "relation": "neighbour",
 "relation_note": "Same pole as Apideck: zero storage, live reads, priced for vendors building many integrations. It aims at AI companies whose agents need live customer data; ownpurse aims at an owner who needs a durable, checkable record. Not a useful source for ownpurse.",
 "borrow": "Reading raw fields alongside the common model (fields=raw) and writing raw back; per-connection tool and permission filters for agents; the honest rule that deletes are emitted only where the source exposes them; free empty polls.",
 "avoid": "Live-only reads as the basis for accounting: nothing to compare against when a source changes, and every agent read costs a metered call twice (Unified's and the provider's). Breadth of 1,250 shallow connectors over depth in a few.",
 "sources": [
  {
   "title": "Unified.to pricing",
   "url": "https://unified.to/pricing"
  },
  {
   "title": "Unified.to about",
   "url": "https://unified.to/about"
  },
  {
   "title": "Unified.to accounting API",
   "url": "https://unified.to/accounting"
  },
  {
   "title": "Unified MCP",
   "url": "https://unified.to/mcp"
  },
  {
   "title": "MCP overview docs",
   "url": "https://docs.unified.to/mcp/overview"
  },
  {
   "title": "Unified.to docs",
   "url": "https://docs.unified.to"
  },
  {
   "title": "Virtual webhooks concept",
   "url": "https://docs.unified.to/concepts/virtual_webhooks.md"
  },
  {
   "title": "Request and write raw integration data",
   "url": "https://docs.unified.to/guides/how_to_request_and_write_raw_integration_data.md"
  },
  {
   "title": "Skills and Core MCP install",
   "url": "https://docs.unified.to/skills/install.md"
  },
  {
   "title": "Unified.to blog",
   "url": "https://unified.to/blog"
  },
  {
   "title": "Unified powers 550% API usage growth (press release, 8 Dec 2025)",
   "url": "https://ca.finance.yahoo.com/news/unified-powers-550-api-usage-120000495.html"
  },
  {
   "title": "Unified.to secures CAD 1.1M pre-seed (The SaaS News)",
   "url": "https://www.thesaasnews.com/news/unified-to-secures-1-1m-cad-in-pre-seed-round"
  }
 ],
 "unverified": [
  "Founding year (no primary source found; Techstars 2023 is the earliest date confirmed)",
  "The December 2024 CAD 1.4M figure comes from a podcast listing linked on the about page; whether it is cumulative or a second round is unclear",
  "Absolute customer count, ARR and call volume",
  "G2, Capterra and Trustpilot ratings",
  "Unit of the plan rate limits (5,000 / 7,500 / 10,000)",
  "Whether you must bring your own Xero app (and pay Xero's tier fees) as with Apideck",
  "Web search budget was exhausted during this pass; independent press and review coverage was not checked"
 ],
 "researched": "2026-10-10"
}
---

## What it is

Unified.to is a unified API from Toronto, founded by Roy Pereira and Alexey Adamsky and backed by [Techstars and Creative Destruction Lab](https://unified.to/about). It connects software and AI companies to 1,250+ integrations across 34 categories, from HR and CRM to accounting, payments and commerce. Its pitch is real time and zero storage: every request goes [live to the source](https://unified.to/accounting), with "no caching, no sync jobs", and it says it never stores end-customer data.

## How it works

An end customer authorizes through an embedded component and you get a connection id; your workspace key and that id address every call. The [accounting API](https://docs.unified.to) has 29 objects over 228 integrations, including journals, transactions and accounts with full CRUD, and read-only trial balance, balance sheet, P&L, cash flow and aged reports. You can ask for the vendor's original object with [`fields=raw`](https://docs.unified.to/guides/how_to_request_and_write_raw_integration_data.md) and write raw fields back. [Virtual webhooks](https://docs.unified.to/concepts/virtual_webhooks.md) poll at an interval you set (one minute at least), store nothing between checks, retry for about two weeks, and report deletes only where the source exposes them.

## Data and access

Pricing is [by call](https://unified.to/pricing): $750 a month for 750,000 calls on Grow, $1,500 for 2M on Pro, $3,000 for 6M on Scale, with overage from $1.00 down to $0.50 per thousand and unlimited connections. A 30-day trial and a sandbox with mock data make it easy to start. Hosting is in US, EU and AU regions; Enterprise adds single-tenant, private cloud or on-prem. Logs are kept 60 days (365+ on Scale). Since nothing is stored, there is nothing to export and no history to ask for.

## Agents

Agents are where Unified.to is pushing hardest. Its [MCP server](https://unified.to/mcp) claims 20,000+ tools, runs in each region over streamable HTTP, and moved to the stateless [2026-07-28 protocol revision](https://unified.to/blog) on launch day. Each request is scoped to one connection, admins can restrict object-level read and write and limit the tools an agent sees ([docs](https://docs.unified.to/mcp/overview)), and PII can be stripped before it reaches the model. Each tool call counts as one API request. Separately, [agent skills and a Claude Code plugin](https://docs.unified.to/skills/install.md) help coding agents build against it. No human approval step for writes is described.

## Customers

Unified.to publishes growth rates rather than totals. A [December 2025 release](https://ca.finance.yahoo.com/news/unified-powers-550-api-usage-120000495.html) reports API usage up 550% in six months, customers up 170% and ARR up 375% over a year. It won Best Business Software API at the 2026 API Awards by its own account. Independent ratings could not be confirmed in this pass. Buyers are software and AI vendors; the $750 floor puts it out of reach for a single small business.

## History

The company went through Techstars Toronto in 2023, raised a [CAD 1.1M pre-seed](https://www.thesaasnews.com/news/unified-to-secures-1-1m-cad-in-pre-seed-round) in October 2024 led by LOI Venture and DevCap, and reported about CAD 1.4M in total by December 2024. Through 2025 and 2026 it widened categories, shipped MCP, and in June 2026 added a Datastore API and guides for keeping RAG indexes and warehouses in sync with live SaaS data. It is small in funding next to Merge or Codat but moving quickly.

## For ownpurse

Unified.to answers a different question: how does an AI product reach a customer's live data in hundreds of apps without holding it? ownpurse asks how one owner keeps a complete, provable record of a few sources. Live-only reads leave no trail when Xero edits or deletes something, and each agent read costs a call twice, once to Unified and again in provider egress. Worth borrowing: the raw-alongside-normalized pattern, the per-connection tool filters, and saying plainly that deletes are visible only where the source shows them.
