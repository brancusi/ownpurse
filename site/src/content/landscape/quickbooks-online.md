---
{
 "slug": "quickbooks-online",
 "name": "QuickBooks Online",
 "url": "https://quickbooks.intuit.com",
 "segment": "incumbent-ledger",
 "status": "active",
 "founded": "1992 (QuickBooks); Intuit 1983",
 "hq": "Mountain View, California, US (Intuit)",
 "backing": "Product of Intuit Inc., a public company listed on NASDAQ (INTU). FY2026 revenue US$21.4 billion (year to 31 July 2026).",
 "tagline": "Intuit's cloud accounting software, the largest small-business ledger in the US.",
 "scope": "QuickBooks Online keeps a small business's books in Intuit's cloud: invoicing, bills and bill pay, bank feeds and categorisation, reporting, payroll, payments, inventory and projects, with an Advanced plan up to 25 users and Intuit Enterprise Suite above it. It does not give the owner an authoritative local record, its API returns current state, and since July 2025 reads (CorePlus calls) are metered while writes are free.",
 "domains": [
  "ledger",
  "bank",
  "invoicing",
  "payments",
  "payroll",
  "tax",
  "reporting",
  "spend"
 ],
 "customers_served": [
  "freelancer",
  "smb",
  "mid-market",
  "accountants",
  "developers"
 ],
 "spectrum": {
  "custody": {
   "score": 1,
   "note": "Intuit holds the authoritative books; you can export reports and lists or read through the API, but reads are metered and there is no full-ledger export."
  },
  "locality": {
   "score": 0,
   "note": "Cloud only (QuickBooks Desktop is a separate, legacy product)."
  },
  "history": {
   "score": 1,
   "note": "An audit log exists in the product, but the API returns current state and its change feed covers a short window."
  },
  "openness": {
   "score": 2,
   "note": "Public REST and GraphQL APIs and an open-source official MCP server; the product is closed."
  },
  "breadth": {
   "score": 3,
   "note": "Books, banking, payments, payroll, bill pay, a business credit card (July 2026), tax estimates and Mailchimp in one ecosystem."
  },
  "autonomy": {
   "score": 3,
   "note": "Books Upkeep resolves high-confidence transactions automatically under rules the customer sets and flags the exceptions; agents categorise, reconcile and chase invoices."
  },
  "agent_readiness": {
   "score": 3,
   "note": "Official MCP server with 145 tools and flags that switch off create, update and delete; OAuth only, no per-user API keys, no hosted endpoint confirmed."
  },
  "integration_ease": {
   "score": 3,
   "note": "Free Builder tier with 500,000 metered reads a month and a sandbox; production needs a public HTTPS callback and Intuit's app review, and reads beyond the tier cost money."
  },
  "market": {
   "score": 2,
   "note": "Small businesses mainly, reaching into the lower mid-market with Advanced and Intuit Enterprise Suite."
  }
 },
 "api": {
  "public": true,
  "style": "REST (Accounting API v3) plus GraphQL for some domains; webhooks; Change Data Capture; batch operations",
  "auth": "OAuth 2.0 through an app registered on the Intuit Developer Portal; refresh tokens rotate; no per-user API keys",
  "limits": "500 requests a minute per company (realm), 10 concurrent requests per app per company, 40 batch requests a minute, 200 a minute on resource-intensive endpoints; report responses capped at 400,000 cells (third-party summary, March 2026).",
  "cost": "Intuit App Partner Program, live 28 July 2025, full fees from 1 November 2025 (US): Builder free with 500,000 CorePlus credits a month, hard-capped; Silver US$300 a month with 1,000,000 and US$3.50 per 1,000 over; Gold US$1,700 with 10,000,000 (500 active connections minimum); Platinum US$4,500 with 75,000,000 and US$0.25 per 1,000 over (3,000 connections minimum). Core (data-in) calls are free and unlimited; CorePlus (data-out reads, queries, reports) is metered. Only 2xx responses count; sandbox is free.",
  "export": "Reports and lists export to Excel, CSV or PDF; the API is the bulk path, and every read is now metered.",
  "notes": "The program covers QuickBooks Online only, not Desktop. Metering is per app and added up per workspace. Charging for reads and not writes rewards putting data in and taxes taking it out."
 },
 "pricing": {
  "model": "Monthly subscription by plan, with payroll, payments and other add-ons billed separately.",
  "entry": "Free plan (1 user, 2 invoices a month, 1 bank, no integrations); Simple Start US$38 a month (US list price on 2026-10-10, 50% off for 3 months).",
  "top": "Advanced US$340 a month for 25 users; Intuit Enterprise Suite above it, priced separately.",
  "notes": "Essentials US$85 and Plus US$140. AI features are packaged by plan: Accounting AI on Essentials, auto-categorisation and anomaly detection on Plus, Books Upkeep and Finance AI on Advanced."
 },
 "agents": {
  "mcp": "official",
  "ai_features": "Intuit Assist (generative assistant since September 2023); AI agents announced 1 July 2025 that manage leads, track payments, send invoices and reconcile; Intuit Intelligence and Intelligence Chat (beta) for mid-market customers, announced 12 August 2026; Books Upkeep in QBO Advanced; conversational forecasting; AI-drafted intercompany entries in Intuit Enterprise Suite.",
  "agent_writes": "Yes. The official MCP server exposes create, update and delete tools for most of its 29 entity types; Intuit's own agents post high-confidence transactions automatically.",
  "approval": "Intuit Intelligence Chat acts only after the user confirms. Books Upkeep posts high-confidence items under customer rules, labels every automated action and sends exceptions to a person. Intercompany drafts go to the user for approval. The MCP server has no approval step, but QUICKBOOKS_DISABLE_WRITE, QUICKBOOKS_DISABLE_UPDATE and QUICKBOOKS_DISABLE_DELETE can remove those tools.",
  "notes": "The MCP server runs locally and holds one company's OAuth refresh token. The repository has 417 stars on GitHub as of 2026-10-10; its README says MIT while the sidebar shows Apache-2.0."
 },
 "integration": {
  "scope": "The QuickBooks App Store, bank feeds, QuickBooks Payments, Payroll, Bill Pay, the QuickBooks business credit card (July 2026) and Mailchimp; supported by every unified accounting API.",
  "notes": "The most widely supported ledger in third-party tools. Production keys need a public HTTPS redirect, which a purely local app must work around."
 },
 "preference": {
  "scale": "QuickBooks Online Accounting revenue up 23% in FY2026; Global Business Solutions revenue US$12.9 billion; Intuit reports about 100 million customers across all products (25 August 2026).",
  "ratings": "Trustpilot 3.7 from 16,890 reviews (2026-10-10).",
  "sentiment": "Praised for onboarding help, breadth, and accountants knowing it well. Complaints are about interface changes that add clicks, bugs and slow loading, support that points to help pages, billing and cancellation problems, forced migrations and price rises (Trustpilot)."
 },
 "timeline": [
  {
   "date": "1983",
   "event": "Intuit founded by Scott Cook and Tom Proulx"
  },
  {
   "date": "1992",
   "event": "First QuickBooks release (desktop)"
  },
  {
   "date": "2013",
   "event": "QuickBooks Online rebuilt as a cloud platform"
  },
  {
   "date": "2014",
   "event": "More new customers chose QBO than Desktop for the first time"
  },
  {
   "date": "2023-06-30",
   "event": "QuickBooks Desktop support ended in the UK and Ireland"
  },
  {
   "date": "2023-09",
   "event": "Intuit Assist introduced"
  },
  {
   "date": "2025-05-15",
   "event": "Intuit App Partner Program announced"
  },
  {
   "date": "2025-07-01",
   "event": "Intuit announced a virtual team of AI agents for QuickBooks"
  },
  {
   "date": "2025-07-28",
   "event": "App Partner Program live; CorePlus reads metered (fees discounted to 31 October)"
  },
  {
   "date": "2025-11-01",
   "event": "Full variable API fees began"
  },
  {
   "date": "2026-07-22",
   "event": "Business credit card launched inside QuickBooks"
  },
  {
   "date": "2026-08-12",
   "event": "Intuit Intelligence for mid-market and accounting firms announced"
  },
  {
   "date": "2026-08-25",
   "event": "FY2026 results: revenue US$21.4 billion, QBO Accounting revenue up 23%"
  }
 ],
 "relation": "source",
 "relation_note": "The planned second ledger connector. It is the most common set of US small-business books, and its metered reads make a local, incrementally synced copy cheaper than reading through the API every time.",
 "borrow": "Write-disable flags on the MCP server as a pattern for agent capability tiers; Books Upkeep's labelled automated actions with exceptions sent to a person; counting only successful calls.",
 "avoid": "Charging for data-out and not data-in; agent tools that write and delete by default; hard caps that block reads mid-month.",
 "sources": [
  {
   "title": "QuickBooks Online pricing (US)",
   "url": "https://quickbooks.intuit.com/pricing/"
  },
  {
   "title": "Intuit FY2026 results (25 August 2026)",
   "url": "https://investors.intuit.com/news-events/press-releases/detail/1320/intuit-reports-fourth-quarter-and-full-year-fiscal-2026-results-sets-fiscal-2027-guidance"
  },
  {
   "title": "Intuit advances its mid-market platform with conversational AI (12 August 2026)",
   "url": "https://investors.intuit.com/news-events/press-releases/detail/1319/intuit-advances-its-mid-market-platform-with-conversational-ai-enterprise-scale-and-deep-industry-workflows-for-cfos-and-accounting-firms"
  },
  {
   "title": "Intuit launches business credit card in QuickBooks (22 July 2026)",
   "url": "https://investors.intuit.com/news-events/press-releases/detail/1316/intuit-launches-business-credit-card-that-brings-spend-management-rewards-and-insights-together-in-quickbooks"
  },
  {
   "title": "Intuit App Partner Program FAQ",
   "url": "https://developer.intuit.com/app/developer/qbo/docs/get-started/partner-faq"
  },
  {
   "title": "QuickBooks API pricing and the Intuit App Partner Program (Apideck)",
   "url": "https://www.apideck.com/blog/quickbooks-api-pricing-and-the-intuit-app-partner-program"
  },
  {
   "title": "How much does the QuickBooks API cost in 2026 (Truto)",
   "url": "https://truto.one/blog/how-much-does-the-quickbooks-api-cost-2026-pricing-rate-limits/"
  },
  {
   "title": "QuickBooks Online MCP server (GitHub)",
   "url": "https://github.com/intuit/quickbooks-online-mcp-server"
  },
  {
   "title": "Intuit introduces a virtual team of AI agents (1 July 2025)",
   "url": "https://www.businesswire.com/news/home/20250701338560/en"
  },
  {
   "title": "QuickBooks reviews (Trustpilot)",
   "url": "https://www.trustpilot.com/review/quickbooks.intuit.com"
  },
  {
   "title": "QuickBooks, Wikipedia",
   "url": "https://en.wikipedia.org/wiki/QuickBooks"
  }
 ],
 "unverified": [
  "Gold tier CorePlus credits (10,000,000) and overage rate come from third-party summaries; Intuit's own FAQ page did not render for this pass.",
  "Rate limits (500 a minute per realm, 10 concurrent, 40 batch) come from a third-party summary dated March 2026.",
  "QuickBooks Online subscriber count for FY2026 (Intuit did not disclose one in the results release).",
  "Change Data Capture lookback window (commonly cited as 30 days) and whether the audit log is reachable through the API.",
  "Read-only access after cancelling (commonly cited as one year) not confirmed on an Intuit help page.",
  "Whether Intuit's developer terms restrict training AI on API data, as Xero's do.",
  "Whether a hosted QuickBooks MCP endpoint exists; sources disagree.",
  "Licence of the MCP server (README says MIT, repository sidebar shows Apache-2.0).",
  "Capterra and G2 ratings (pages blocked)."
 ],
 "researched": "2026-10-10"
}
---

## What it is

QuickBooks Online is Intuit's cloud accounting product and the default ledger for US small businesses. QuickBooks first shipped in 1992, and Intuit rebuilt the online edition as a cloud platform in 2013. Intuit reported [FY2026 revenue of US$21.4 billion](https://investors.intuit.com/news-events/press-releases/detail/1320/intuit-reports-fourth-quarter-and-full-year-fiscal-2026-results-sets-fiscal-2027-guidance) for the year to 31 July 2026. QuickBooks Online Accounting revenue grew 23% that year.

What it does well: almost every accountant, bank and app in the US knows it. It covers banking, payroll, payments, bill pay and now a business credit card, and it reaches from sole traders to 25-user businesses.

## How it works

The books live in Intuit's cloud. Bank feeds bring in transactions, AI categorises them, and you reconcile, invoice and report. On 2026-10-10 the [US plans](https://quickbooks.intuit.com/pricing/) were Free, Simple Start (US$38 a month), Essentials (US$85), Plus (US$140) and Advanced (US$340). AI features are packaged by plan: Accounting AI from Essentials, anomaly detection on Plus, and Books Upkeep, which keeps the books current through the month, on Advanced. Intuit Enterprise Suite sits above that for multi-entity companies.

## Data and access

The API is REST (Accounting v3) with GraphQL for some domains, OAuth 2.0, webhooks and a change-data-capture endpoint. The [Intuit App Partner Program](https://www.apideck.com/blog/quickbooks-api-pricing-and-the-intuit-app-partner-program), live since 28 July 2025, splits calls in two. Core calls, which put data in, are free and unlimited. CorePlus calls, which read data out, are metered. Builder is free with 500,000 CorePlus credits a month and blocks reads past that. Silver is US$300 a month for 1,000,000 credits. Platinum is US$4,500 for 75,000,000 and needs 3,000 active connections. Full fees began on 1 November 2025.

Rate limits are [500 requests a minute per company, with 10 concurrent](https://truto.one/blog/how-much-does-the-quickbooks-api-cost-2026-pricing-rate-limits/). Reports, lists and transactions can be exported to Excel. The only bulk path is the API, and every read through it now counts.

## Agents

Intuit has two agent stories. Inside the product, Intuit Assist (2023) became a set of agents in July 2025. The [August 2026 Intuit Intelligence release](https://investors.intuit.com/news-events/press-releases/detail/1319/intuit-advances-its-mid-market-platform-with-conversational-ai-enterprise-scale-and-deep-industry-workflows-for-cfos-and-accounting-firms) spells out the gates: chat acts only after you confirm, Books Upkeep posts high-confidence transactions under your rules and sends exceptions to a person, and every automated action is labelled. That is the most explicit human-in-the-loop design among the incumbents.

Outside the product, the [official MCP server](https://github.com/intuit/quickbooks-online-mcp-server) runs locally with 145 tools across 29 entities, including create, update and delete. Environment flags can switch writes, updates and deletes off, but by default they are on, and nothing asks a person first.

## Customers

On [Trustpilot QuickBooks scores 3.7 from 16,890 reviews](https://www.trustpilot.com/review/quickbooks.intuit.com), lower than Xero or Sage. Reviewers praise onboarding staff. They complain about interface changes, bugs, slow support, billing and cancellation, and being moved onto new products. Accountants put up with it because their clients use it.

## History

Intuit was founded in 1983. QuickBooks Desktop dominated US small-business accounting for two decades, and Intuit has since pushed customers to the cloud, ending Desktop in the UK and Ireland in June 2023. The 2025 partner program turned the API from free into a priced product.

## For ownpurse

QuickBooks is the next connector after Xero. Reads cost money and writes do not, so ownpurse's model fits it well: read once, incrementally, keep every version locally, and answer every later question from the local record at no cost. Writes go back as approved packages, which are free Core calls. Because a production app needs a public HTTPS callback, a local tool needs a hosted redirect or a loopback workaround. Intuit's Books Upkeep is the strongest proof that incumbents will automate posting themselves. ownpurse's role is the owner's independent, provable copy beside that automation, not a replacement for it.
