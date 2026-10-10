---
{
 "slug": "railz",
 "name": "Railz (FIS Accounting Data as a Service)",
 "url": "https://www.railz.ai",
 "segment": "unified-api",
 "status": "acquired",
 "founded": "June 2020 (BetaKit, May 2021)",
 "hq": "Toronto, Canada (as Railz); now part of FIS, Jacksonville, Florida",
 "backing": "Series A of $12M USD ($14.5M CAD), led by Nyca Partners with Susa Ventures, Vestigo Ventures, Entrée Capital, Global Founders Capital, Plug and Play Ventures, N49P and Hack VC, reported 7 May 2021. Acquired by FIS (NYSE: FIS); renamed FIS Accounting Data as a Service. Its public SDK repos were rebranded in October 2024, and railz.ai has 301-redirected to fisglobal.com since at least 10 October 2026. The deal date and terms are not confirmed here.",
 "tagline": "An accounting, banking and commerce data API for lenders and fintechs, built by Railz and now sold by FIS as Accounting Data as a Service.",
 "scope": "It reads normalized data from 11 accounting systems, from Plaid for banking and from Shopify and Square for commerce, through one REST API with a hosted connect widget. It layers analytics on top: rebuilt statements, credit scores, probability of default, fraud metrics and benchmarks. It can push many record types back into the ledger. It does not keep the books, give the business owner a copy, or provide a versioned history.",
 "domains": [
  "integration",
  "ledger",
  "bank",
  "commerce",
  "reporting"
 ],
 "customers_served": [
  "enterprise",
  "mid-market",
  "developers"
 ],
 "spectrum": {
  "custody": {
   "score": 1,
   "note": "Syncs keep a copy in FIS storage on behalf of the lender or fintech. The business owner's books stay with their provider and the owner gets no copy."
  },
  "locality": {
   "score": 0,
   "note": "Cloud only, hosted by FIS."
  },
  "history": {
   "score": 1,
   "note": "Pulls up to 5 years (10 on enterprise) of records and keeps them in storage. Nothing documents versioning, as-of queries or how deletes are handled."
  },
  "openness": {
   "score": 2,
   "note": "Closed platform, but the REST API, sandbox and free tier are self-serve, and statements export as PDF, JSON, XLSX and CSV. Only the connect widget SDK is MIT-licensed."
  },
  "breadth": {
   "score": 3,
   "note": "Covers accounting (11 systems), banking via Plaid and commerce (Shopify, Square) behind one model, plus analytics. No documents, payroll or payments."
  },
  "autonomy": {
   "score": 1,
   "note": "Middleware. It syncs on a schedule, but any write to the books is a call made by the client's own code."
  },
  "agent_readiness": {
   "score": 2,
   "note": "A public REST API with an OpenAPI reference and webhooks. No MCP server, CLI or agent docs found."
  },
  "integration_ease": {
   "score": 3,
   "note": "Free self-serve sandbox and production access for 5 businesses, Postman setup, a hosted connect widget and a no-code Sites page, and migration from other providers without reauthorization."
  },
  "market": {
   "score": 3,
   "note": "Sold to banks, lenders and fintechs that serve small and midsize businesses. Enterprise contracts sit at the top."
  }
 },
 "api": {
  "public": true,
  "style": "REST/JSON (v2, base https://api.railz.ai/v2), OpenAPI/Swagger reference, webhooks",
  "auth": "client_id and secret_key sent as HTTP Basic to auth.railz.ai return a Bearer access_token valid for 60 minutes. Sandbox keys start with SB_, production keys with ID_. Up to 5 keys per team, and keys do not expire until deleted.",
  "limits": "No rate limits are published in the pages read. Webhooks allow up to 5 event types per URL, may arrive out of order, and are retried twice if no 2xx comes back within 60 s. Sync can be daily (the default), weekly, bi-weekly, monthly, quarterly, annual or off, with on-demand sync through the Queue Data Sync API. History depth is 5 years on the free and pay-as-you-go plans and up to 10 years on enterprise.",
  "cost": "Freemium: lifetime sandbox and production access for up to 5 connected businesses. The docs also mention a pay-as-you-go plan. Enterprise is custom-priced with volume discounts and unlimited businesses. No per-connection prices are published.",
  "export": "The client, not the business owner, can export financial statements as PDF, JSON, XLSX and CSV, and can pull every normalized object through the API.",
  "notes": "Supports reconstruct=true, which rebuilds balance sheets, income statements and, for QuickBooks Online, Xero and FreshBooks, cash flow statements when the ERP's API does not provide them. Push support for Xero covers invoices, bills, payments, journal entries, bank transactions, contacts, chart of accounts and more (create, and for some types update), per the FIS knowledge base."
 },
 "pricing": {
  "model": "Freemium developer tier plus custom enterprise contracts",
  "entry": "Free: up to 5 connected businesses, sandbox and production",
  "top": "Enterprise, custom: unlimited businesses, white-label, SSO, tax benchmarking, private betas, dedicated account manager",
  "notes": "The docs name a pay-as-you-go plan, but no price for it was found. Analytics such as credit scores and benchmarks are part of the product; how they are priced is not published."
 },
 "agents": {
  "mcp": "none",
  "ai_features": "A proprietary machine-learning model normalizes charts of accounts into section, subsection, group and subgroup. Proprietary models produce credit scores, probability of default, fraud risk and forecasts. Bring Your Own Data (beta) uses LLM models to read financial data that small businesses upload by hand. FIS says the product is not a decisioning engine.",
  "agent_writes": "Only as ordinary API calls. Push endpoints create and update invoices, bills, journal entries, bank transactions, contacts and so on in the connected ledger.",
  "approval": "None in the product. A push goes straight to the ledger once the business has granted the connection. Any review step is left to the integrating app.",
  "notes": "No agent-specific interface, MCP server or llms.txt was found on 10 October 2026. The web search budget ran out, so community MCP servers could not be ruled out."
 },
 "integration": {
  "scope": "Accounting: Dynamics 365 Business Central, Dynamics 365 Finance, FreshBooks, MYOB, Oracle NetSuite, QuickBooks Desktop, QuickBooks Online, Sage Business Cloud, Sage Intacct, Wave, Xero and Zoho Books (12 in the docs nav; the FIS product page lists 10 and calls MYOB upcoming). Banking: Plaid. Commerce: Shopify, Square. Plus BYOD manual upload (beta).",
  "notes": "Two ways to connect: Connect, an embeddable widget with React and Angular examples, or Sites, a hosted white-label page. Connections can migrate from another provider without reauthorization for QuickBooks Online, Xero, FreshBooks, Wave, Sage Business Cloud, Business Central, Plaid and Zoho Books. Known Xero gaps: AP/AR aging is not supported, and discount fields can be missing on invoice and PO pulls."
 },
 "preference": {
  "scale": "Not found. FIS publishes no customer count for the product.",
  "ratings": "Not found. G2 and Capterra blocked access, and no Trustpilot or app store listing was found.",
  "sentiment": "No public reviews were found. In its favour: a generous free tier, broad push coverage, and analytics that lenders can use straight away. Against: since the FIS acquisition the brand is split between fisglobal.com and railz.ai, and parts of the docs are thin or 404 (rate limits, sync semantics)."
 },
 "timeline": [
  {
   "date": "2020-06",
   "event": "Railz founded in Toronto (per BetaKit)"
  },
  {
   "date": "2021-05-07",
   "event": "$12M USD Series A led by Nyca Partners reported"
  },
  {
   "date": "2021-07-30",
   "event": "railz-ai GitHub organization created"
  },
  {
   "date": "2024-10-08",
   "event": "Public visualization SDK renamed from Railz to FIS Accounting Data as a Service"
  },
  {
   "date": "2025-09-18 to 2026-10-06",
   "event": "Quarterly releases continue under FIS (latest Q3.2, 6 October 2026)"
  },
  {
   "date": "2026-10-10",
   "event": "railz.ai 301-redirects to the FIS product page; docs, dashboard and API stay on railz.ai domains"
  }
 ],
 "relation": "neighbour",
 "relation_note": "Railz is middleware for lenders and fintechs that read a business's books, and FIS now owns it. It works for whoever holds the API key, not for the business owner. It is not a competitor to ownpurse, and it is a poor source: a third party sitting between ownpurse and Xero, with no history and a cost.",
 "borrow": "Reconstructed statements (rebuilding a balance sheet or cash flow statement when the source API lacks one) and the four-level normalized account hierarchy. Webhook rules worth copying: events can arrive out of order, so never assume a sequence; retries are bounded. Per-data-type sync settings that pull parent types automatically. Connection migration without making people reauthorize.",
 "avoid": "A single mutable copy, synced on a schedule, with no versioning or delete semantics. A push that reaches the ledger with no approval step. A brand and docs split across two owners after an acquisition. Custody that serves the lender rather than the owner.",
 "sources": [
  {
   "title": "FIS Accounting Data as a Service product page",
   "url": "https://www.fisglobal.com/products/accounting-data-as-a-service"
  },
  {
   "title": "FIS Accounting Data as a Service docs",
   "url": "https://docs.railz.ai/"
  },
  {
   "title": "Authentication",
   "url": "https://docs.railz.ai/docs/authentication"
  },
  {
   "title": "Webhooks",
   "url": "https://docs.railz.ai/docs/webhooks"
  },
  {
   "title": "Manage data sync settings",
   "url": "https://docs.railz.ai/docs/dashboard-data-sync-settings.md"
  },
  {
   "title": "Reconstructed data",
   "url": "https://docs.railz.ai/docs/reconstructed-data.md"
  },
  {
   "title": "Xero supported data types",
   "url": "https://knowledge.railz.ai/services/xero"
  },
  {
   "title": "Analytics overview",
   "url": "https://docs.railz.ai/docs/analytics-overview"
  },
  {
   "title": "Bring Your Own Data",
   "url": "https://docs.railz.ai/docs/byod.md"
  },
  {
   "title": "Security",
   "url": "https://docs.railz.ai/docs/security.md"
  },
  {
   "title": "Migrate to Accounting Data as a Service",
   "url": "https://docs.railz.ai/docs/migrate-to-railz.md"
  },
  {
   "title": "Xero FAQ",
   "url": "https://docs.railz.ai/docs/integrations-accounting-xero-faq.md"
  },
  {
   "title": "Release notes",
   "url": "https://docs.railz.ai/changelog"
  },
  {
   "title": "BetaKit: Today in funding (Railz Series A)",
   "url": "https://betakit.com/today-in-funding-railz-relay-platform-ferme-dhiver-optable-interval/"
  },
  {
   "title": "railz-ai/railz-visualizations on GitHub",
   "url": "https://github.com/railz-ai/railz-visualizations"
  }
 ],
 "unverified": [
  "Date and terms of the FIS acquisition: the press release was not found (FIS media room, PR Newswire and SEC full-text search turned up nothing, Business Wire blocked access, and the web search budget for the session was exhausted). The rebrand in the public repos dates from October 2024, so the deal was at or before then.",
  "Railz founders' names and any pre-Series A funding.",
  "Pay-as-you-go plan pricing (the plan is named in the docs, no price found).",
  "Customer count, revenue and any G2, Capterra or Trustpilot ratings (sites blocked access or had no listing).",
  "API rate limits and whether pushes are synchronous or asynchronous (not in the docs pages read; several doc URLs returned 404).",
  "Whether stored synced data is overwritten, versioned or deleted on resync.",
  "Whether any community MCP server exists for the API (could not search)."
 ],
 "researched": "2026-10-10"
}
---

## What it is

Railz began in Toronto in June 2020 as an accounting data API for lenders. It is now [FIS Accounting Data as a Service](https://www.fisglobal.com/products/accounting-data-as-a-service): railz.ai redirects there, while the [docs](https://docs.railz.ai/), dashboard and API stay on railz.ai domains. FIS describes it as "a financial data API that transforms the way financial institutions, fintechs and developers push and pull data." The buyer is a bank or a fintech that wants to see a small business's books. The business itself is not the customer.

## How it works

A business connects through Connect, an embeddable widget, or Sites, a hosted white-label page. Each link to a provider is a connection, and a business can have several. Syncs run daily by default and can be set to anything from weekly to annual, or off. On-demand syncs go through a queue API. The free and pay-as-you-go plans pull up to five years of records and enterprise up to ten ([data sync settings](https://docs.railz.ai/docs/dashboard-data-sync-settings.md)). Synced data is kept in FIS storage. The pages read do not say whether a later sync overwrites earlier values or how deletes are handled.

On top of that sits the part lenders pay for. A machine-learning model maps every chart of accounts into four normalized levels. Analytics produce credit scores, probability of default, fraud metrics, forecasts and benchmarks ([analytics](https://docs.railz.ai/docs/analytics-overview)). `reconstruct=true` rebuilds statements the ERP's API does not provide, including cash flow statements for QuickBooks Online, Xero and FreshBooks ([reconstructed data](https://docs.railz.ai/docs/reconstructed-data.md)).

Writes are broad. For Xero, the [support matrix](https://knowledge.railz.ai/services/xero) shows creates for invoices, bills, payments, journal entries, bank transactions, chart of accounts, tax rates and tracking categories, and updates for several of those. A push is a plain API call, and nothing in the product asks a person to approve it.

## Data and access

The API is REST at `api.railz.ai/v2`. Authentication sends a client id and secret to get a Bearer token that lasts 60 minutes ([authentication](https://docs.railz.ai/docs/authentication)). Webhooks report connections, completed syncs and successful pushes. They may arrive out of order and are retried twice ([webhooks](https://docs.railz.ai/docs/webhooks)). No rate limits are published in the pages read. The free tier gives sandbox and production access for five connected businesses, for life. Enterprise is custom-priced, and no per-connection prices are public. Statements export as PDF, JSON, XLSX and CSV, but only to the client. The security page claims SOC 2 and encryption at rest and in transit, and says nothing about retention or deletion ([security](https://docs.railz.ai/docs/security.md)).

## Agents

There is no MCP server, CLI or agent documentation. The AI here is in-house modelling: account normalization, scoring, and Bring Your Own Data, a beta that uses LLM models to read data small businesses upload by hand ([BYOD](https://docs.railz.ai/docs/byod.md)). An agent could drive the REST API like any other client. If it did, its writes would land in the ledger directly.

## Customers

FIS sells to financial institutions, fintechs and developers building for small and midsize businesses. The use cases are lending, payment reconciliation and financial planning. No customer count or public reviews were found, and G2 and Capterra blocked access.

## History

[BetaKit reported](https://betakit.com/today-in-funding-railz-relay-platform-ferme-dhiver-optable-interval/) a $12M USD Series A led by Nyca Partners on 7 May 2021. FIS later bought the company. The public [visualization SDK](https://github.com/railz-ai/railz-visualizations) was renamed from Railz in October 2024, and quarterly releases continue to 6 October 2026 ([changelog](https://docs.railz.ai/changelog)). Railz follows a familiar path for unified accounting APIs: a startup normalizes small-business ledgers, and a bank or processor buys it to feed underwriting. The data still flows, toward the lender.

## For ownpurse

Railz shows where accounting middleware goes when lenders are the buyers. Railz holds the copy, the lender reads it, and the business owner has neither the copy nor any history. ownpurse puts the record in the opposite place: on the owner's machine, append-only, with writes held for approval. Two ideas are worth taking. One is reconstructed statements, rebuilt and checked against the source's own reports. The other is the webhook discipline of never assuming order. Do not read Xero through Railz: it adds a third party between ownpurse and the source and loses fidelity, and its stored copy has no documented history.
