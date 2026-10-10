---
{
 "slug": "chift",
 "name": "Chift",
 "url": "https://www.chift.eu",
 "segment": "unified-api",
 "status": "active",
 "founded": "2022",
 "hq": "Brussels, Belgium",
 "backing": "Venture-backed. EUR 10.5M Series A led by BlackFin Capital Partners with Entourage, Shapers, Seeder Fund and Wallonie Entreprendre (announced 2026, exact date not shown). An earlier round is referenced by a June 2024 TechCrunch headline linked from its about page.",
 "tagline": "A European unified API for accounting, invoicing, POS, e-commerce, payments, banking and PMS, covering the local ledgers that US players skip.",
 "scope": "Chift connects European software vendors to 150+ financial tools (80+ accounting systems, many national: DATEV, Exact, Fortnox, Cegid, a3ERP, e-Boekhouden, Fiken and more) through unified APIs, ready-made Syncs it runs for you, and a Datalayer that keeps a synced copy of accounting, POS and banking data. It has remote and local MCP servers and a CLI. It does not hold a ledger for the business, and its copy has no version history.",
 "domains": [
  "integration",
  "ledger",
  "invoicing",
  "bank",
  "payments",
  "commerce"
 ],
 "customers_served": [
  "developers",
  "smb",
  "mid-market"
 ],
 "spectrum": {
  "custody": {
   "score": 1,
   "note": "Live reads by default; the optional Datalayer copy sits on Chift's infrastructure, not with the owner."
  },
  "locality": {
   "score": 0,
   "note": "Cloud only; the local MCP server and CLI run on your machine but call Chift's hosted API."
  },
  "history": {
   "score": 1,
   "note": "Datalayer keeps a refreshed copy with lookback and data_deleted events, but no versions or as-of reads."
  },
  "openness": {
   "score": 2,
   "note": "Closed service with a public OpenAPI spec, an open-source local MCP server and a CLI."
  },
  "breadth": {
   "score": 3,
   "note": "Accounting, invoicing, banking, payments, POS, e-commerce and PMS: the finance side of a European SMB."
  },
  "autonomy": {
   "score": 3,
   "note": "Syncs that Chift builds and runs write invoices, orders and payouts into ledgers on a schedule or event."
  },
  "agent_readiness": {
   "score": 4,
   "note": "Remote MCP with OAuth scopes, local MCP with per-domain function config, CLI with read/write/dangerous classes, agent skill."
  },
  "integration_ease": {
   "score": 2,
   "note": "Good docs and llms.txt, but no published pricing and onboarding goes through a demo."
  },
  "market": {
   "score": 2,
   "note": "Sells to software vendors whose end customers are European SMBs and their accountants."
  }
 },
 "api": {
  "public": true,
  "style": "REST/JSON unified API per domain (OpenAPI published), with raw data access; Syncs for managed flows; Datalayer reads via the x-chift-datalayer header.",
  "auth": "Account id, client id and client secret exchanged for an access token; a consumer id per end customer; remote MCP authorizes via OAuth scopes.",
  "limits": "No Chift-imposed rate limits (fair use); connector limits apply, short-term limits retried up to 5 times with backoff, then 429 with Retry-After.",
  "cost": "Not published; demo-led sales. Datalayer and Sync cadence are agreed with Chift's team.",
  "export": "Nothing owner-facing; the Datalayer copy is readable through the same API but has no stated retention.",
  "notes": "Datalayer covers accounting folders, chart of accounts, journals, journal entries and lines, invoices, partners, VAT codes, book years and analytic plans; incremental refresh with a lookback window for backdated edits; initial load configurable from July 2026 (default 12 months)."
 },
 "pricing": {
  "model": "Not published; sold through demos",
  "entry": "Not published",
  "top": "Not published",
  "notes": "No pricing page found on 2026-10-10 (chift.eu/pricing returns 404 and the navigation shows only 'Book a demo')."
 },
 "agents": {
  "mcp": "official",
  "ai_features": "Remote MCP at mcp.chift.eu for one consumer; open-source local MCP (Python, uvx chift-mcp-server) across all consumers with optional docs search; Chift CLI with agent skill; Chifty, a read-only in-dashboard assistant for diagnosing integrations (all Unified API accounts from July 2026); AI field mapping.",
  "agent_writes": "Yes. MCP tools mirror the unified API one to one, including creating invoices and entries.",
  "approval": "No human approval step. Remote MCP limits tools to granted OAuth scopes; local MCP restricts operations per domain via CHIFT_FUNCTION_CONFIG; the CLI requires --force for mutating calls and CHIFT_ALLOWED_OPERATIONS=read blocks writes.",
  "notes": "The CLI's --force on every mutation and its read-only switch are guard rails, not a reviewed package. Its own Series A post describes building an 'agentic layer' for automated connections."
 },
 "integration": {
  "scope": "150+ connectors (more than 120 financial systems in the Series A post); 80+ accounting tools and 40+ bank sources per its site, across 13 countries; strongest in France, Belgium and the Netherlands, growing in Spain, the UK, the Nordics and Germany.",
  "notes": "Named accounting connectors include DATEV, Exact, AFAS, Cegid Loop, ACD, Fulll, a3ERP, Contasol, e-Boekhouden, e-conomic, Dinero, Fiken, Procountor, Fortnox, FreeAgent, Business Central, Sage 50 UK, Xero and QuickBooks. Webhooks for data_changed and data_deleted per Datalayer refresh."
 },
 "preference": {
  "scale": "More than 150 software companies build on Chift, connecting over 50,000 businesses (Series A post, 2026). Named customers include Pennylane, Qonto, Pleo, Sage, Revolut, Mollie, Spendesk and Agicap.",
  "ratings": "Not confirmed: no Trustpilot page; G2 and Capterra not reachable in this pass.",
  "sentiment": "Not confirmed from independent reviews. A finalist for Deloitte's Rising Star Award 2026 (vendor blog); a case study claims Smovin cut integration time by 80% (vendor-reported)."
 },
 "timeline": [
  {
   "date": "2022",
   "event": "Founded in Brussels by Gauthier Henroz, Matthieu Hertoghe and Henry Hertoghe"
  },
  {
   "date": "2024",
   "event": "Raises funding to expand across Europe; 100+ financial tools connected"
  },
  {
   "date": "2025 (approx.)",
   "event": "Launches in Spain (described as 'a year ago' in the Series A post)"
  },
  {
   "date": "2026",
   "event": "EUR 10.5M Series A led by BlackFin Capital Partners; 150+ software customers, 50,000+ businesses"
  },
  {
   "date": "2026",
   "event": "MCP layer for AI agents (remote and local servers)"
  },
  {
   "date": "2026-07",
   "event": "Chifty assistant on all Unified API accounts; Datalayer initial load window configurable"
  }
 ],
 "relation": "neighbour",
 "relation_note": "The European counterpart to Merge and Apideck, deep where they are thin: national ledgers and e-invoicing. It serves the software vendors around a European SMB; ownpurse serves the owner. Its connector list is a map of the European ledgers ownpurse would need to reach later, one at a time.",
 "borrow": "The Datalayer's split: reference data re-synced in full each run, time-series data incremental with a lookback window for backdated edits, plus explicit data_deleted events; returning 409 'sync pending' instead of empty data; per-domain read-only switches for agents; local and remote MCP for different trust settings.",
 "avoid": "A synced copy without versions, which still loses what a source overwrote; wiping the store when a connection is re-pointed; unpublished pricing, which keeps an owner from knowing what access costs.",
 "sources": [
  {
   "title": "Chift home",
   "url": "https://www.chift.eu"
  },
  {
   "title": "Chift about",
   "url": "https://www.chift.eu/about"
  },
  {
   "title": "Chift integrations",
   "url": "https://www.chift.eu/integrations"
  },
  {
   "title": "Chift raises EUR 10.5M Series A led by BlackFin",
   "url": "https://www.chift.eu/blog/chift-raises-eu10-5m-in-a-series-a-round-led-by-blackfin-capital-partners"
  },
  {
   "title": "Chift blog",
   "url": "https://www.chift.eu/blog"
  },
  {
   "title": "Chift docs",
   "url": "https://docs.chift.eu"
  },
  {
   "title": "Datalayer overview",
   "url": "https://docs.chift.eu/developer-guides/datalayer/overview.md"
  },
  {
   "title": "Syncs overview",
   "url": "https://docs.chift.eu/developer-guides/syncs/overview.md"
  },
  {
   "title": "Rate limits",
   "url": "https://docs.chift.eu/developer-guides/core-concepts/rate-limits.md"
  },
  {
   "title": "MCP server docs",
   "url": "https://docs.chift.eu/ai/mcp.md"
  },
  {
   "title": "Local MCP docs",
   "url": "https://docs.chift.eu/ai/mcp/local.md"
  },
  {
   "title": "Chift CLI docs",
   "url": "https://docs.chift.eu/ai/cli.md"
  },
  {
   "title": "Changelog, July 2026",
   "url": "https://docs.chift.eu/changelogs/2026-07.md"
  },
  {
   "title": "chift-oneapi/chift-mcp on GitHub",
   "url": "https://github.com/chift-oneapi/chift-mcp"
  }
 ],
 "unverified": [
  "Exact date of the EUR 10.5M Series A (announced in 2026; day not shown on the post)",
  "The earlier round: the about page links a TechCrunch article dated 25 June 2024 and a EUR 2.3M headline, but the article could not be fetched",
  "Pricing of any kind (not published)",
  "Datalayer retention period and cost",
  "License of the local MCP server",
  "G2, Capterra and Trustpilot ratings",
  "Exact accounting connector count (site says 80+; the integrations grid shows 19 before 'load more')",
  "Web search budget was exhausted during this pass; independent press and review coverage was not checked"
 ],
 "researched": "2026-10-10"
}
---

## What it is

Chift is a unified API for European financial software, [founded in Brussels in 2022](https://www.chift.eu/about) by Gauthier Henroz, Matthieu Hertoghe and Henry Hertoghe. Where US unified APIs cover QuickBooks, Xero and NetSuite well and stop, Chift covers the [national ledgers](https://www.chift.eu/integrations) a European SMB actually uses: DATEV, Exact, AFAS, Cegid, a3ERP, e-Boekhouden, Fortnox, Fiken, e-conomic and more, alongside Xero and QuickBooks. It offers six unified APIs (accounting, invoicing and CRM, POS, e-commerce, payment, PMS) plus banking, over 150+ connectors.

## How it works

Chift has three ways in. The unified API calls the source live, with one shape per domain and access to raw data. [Syncs](https://docs.chift.eu/developer-guides/syncs/overview.md) are flows Chift builds and runs for you, such as pushing invoices, POS orders or payouts into a customer's ledger nightly or on an event. The [Datalayer](https://docs.chift.eu/developer-guides/datalayer/overview.md) keeps a copy of accounting, POS and banking data on Chift's infrastructure: reference data (chart of accounts, partners) is fully re-synced each run, time-series data (journal entries, invoices, bank transactions) is pulled incrementally with a lookback window for backdated edits, and each refresh emits `data_changed` and `data_deleted` webhooks. You opt in per request with a header. The copy keeps no versions, and re-pointing a connection to another account wipes it.

## Data and access

Chift publishes an OpenAPI spec and [imposes no rate limits of its own](https://docs.chift.eu/developer-guides/core-concepts/rate-limits.md); it retries short-term connector limits up to five times and returns a 429 with Retry-After after that. Auth is an account id, client id and secret for a token, plus a consumer id per end customer. Pricing is not published: there is no pricing page and the site offers a demo. Since July 2026 you can set how far back the Datalayer's first load reaches ([changelog](https://docs.chift.eu/changelogs/2026-07.md)); the default is 12 months.

## Agents

Chift ships two [MCP servers](https://docs.chift.eu/ai/mcp.md). The remote one, at `mcp.chift.eu`, serves one consumer and limits tools to the OAuth scopes granted. The [local one](https://github.com/chift-oneapi/chift-mcp) is an open-source Python package that spans all your consumers and can be limited per domain to get, create, update or add. The [CLI](https://docs.chift.eu/ai/cli.md) classes calls as read, write or dangerous, demands `--force` for any mutation, and can be held to reads with `CHIFT_ALLOWED_OPERATIONS=read`. Chifty, a read-only assistant in the dashboard, diagnoses failed integrations. None of this is a human approval of a specific write.

## Customers

Per its [Series A announcement](https://www.chift.eu/blog/chift-raises-eu10-5m-in-a-series-a-round-led-by-blackfin-capital-partners), more than 150 software companies build on Chift and connect over 50,000 businesses in 13 countries; named customers include Pennylane, Qonto, Pleo, Sage, Revolut and Mollie. These are the fintechs and vertical tools that sell to European small businesses and their accountants. Independent review ratings could not be found in this pass.

## History

After its 2022 founding, Chift raised money in 2024 to expand across Europe, launched in Spain about a year before its Series A, and in 2026 raised EUR 10.5M led by BlackFin Capital Partners, with existing investors Entourage, Shapers, Seeder Fund and Wallonie Entreprendre. It plans to grow in the UK, the Nordics, Germany and Italy and to build an "agentic layer" for automated connections.

## For ownpurse

Chift is the closest of the unified APIs to keeping a record: its Datalayer refreshes a copy, re-pulls a lookback window and reports deletes. It still overwrites, holds the copy itself, and wipes it on a re-point, so it cannot say what a ledger said before a correction. For ownpurse, its connector list is a map of where European owners keep their books, and its Datalayer rules (full re-sync for reference data, incremental with lookback for movements, explicit delete events, 409 while the first load runs) are good defaults for ownpurse's own connectors, written as facts instead of overwrites.
