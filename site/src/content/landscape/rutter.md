---
{
 "slug": "rutter",
 "name": "Rutter",
 "url": "https://www.rutter.com",
 "segment": "unified-api",
 "status": "active",
 "founded": "2021 (publicly launched August 2021)",
 "hq": "New York, USA",
 "backing": "Venture-backed: $1.5M at launch (August 2021); $27M Series A led by Andreessen Horowitz (March 2022), with Basis Set, Comma Capital, Mischief and others, per TechCrunch. Y Combinator appears among its backers on its site. No later round confirmed.",
 "tagline": "A unified API for accounting, commerce, payments and ads data, sold to fintechs and banks, now also embedding banking and payments inside ERPs.",
 "scope": "Rutter connects to 60 to 75+ accounting, ERP, commerce and payments platforms, syncs them into a cache and serves normalised reads and asynchronous writes, plus bank feeds into accounting platforms. In 2026 it added Embedded ERP (banking, payments and reconciliation inside NetSuite, Dynamics, Sage Intacct and SAP for banks and fintechs) and a read-only-by-default MCP server. It keeps current state only: when a source deletes a record, Rutter removes it from its system.",
 "domains": [
  "integration",
  "ledger",
  "commerce",
  "payments",
  "bank"
 ],
 "customers_served": [
  "developers",
  "mid-market",
  "enterprise"
 ],
 "spectrum": {
  "custody": {
   "score": 1,
   "note": "A cache of the source in Rutter's cloud for Rutter's customer; the business owner holds no copy."
  },
  "locality": {
   "score": 0,
   "note": "Cloud only; no self-hosted option found."
  },
  "history": {
   "score": 0,
   "note": "Current state; 'Rutter will query for deleted data and remove it from its system', leaving only a delete event or webhook."
  },
  "openness": {
   "score": 2,
   "note": "Public, versioned API docs and a Postman collection; closed service, no public pricing."
  },
  "breadth": {
   "score": 3,
   "note": "Accounting, commerce, payments, ads and bank feeds; no documents or receipts."
  },
  "autonomy": {
   "score": 3,
   "note": "Customers' software writes bills, invoices and payments unattended through async jobs; no approval step."
  },
  "agent_readiness": {
   "score": 3,
   "note": "An official MCP server (October 2026), read-only by default with per-key scopes; no agent write tools or CLI found."
  },
  "integration_ease": {
   "score": 3,
   "note": "Free self-serve sandbox for 30 days and clear docs; production needs a custom quote."
  },
  "market": {
   "score": 3,
   "note": "Fintechs and banks, from growth-stage (Mercury, Ramp) to Fortune 500 and financial institutions."
  }
 },
 "api": {
  "public": true,
  "style": "REST with JSON, dated versions in an X-Rutter-Version header (2024-08-31 latest), cursor pagination, asynchronous writes as jobs, an Events API and signed webhooks.",
  "auth": "HTTP Basic with client_id and client_secret, plus an access_token per connection obtained by exchanging a public token from Rutter Link.",
  "limits": "500 requests per 10 seconds per organisation on paid plans, 100 on free, test and sandbox. Reads come from cache and avoid platform limits; writes pass through to each platform's limits, with platform 429s surfaced as status 452.",
  "cost": "Not public: free sandbox for 30 days, then a custom quote for production (October 2026).",
  "export": "No owner export; Rutter's customer reads the data by API.",
  "notes": "The initial sync can take hours to days, and requests before the INITIAL_UPDATE webhook return PRODUCT_NOT_READY. Real-time webhooks for creates and updates exist for QuickBooks, Xero, Stripe (payouts and transactions) and Sage Intacct; other platforms use incremental sync. Rutter tells customers to poll as well, because webhooks can be dropped. Marking a record inactive is not a delete and fires no delete event."
 },
 "pricing": {
  "model": "Custom contract.",
  "entry": "Free Starter: 30 days of sandbox access with sample data and trials of QuickBooks, Xero, FreshBooks and Zoho Books; no card required.",
  "top": "Full Access by custom quote: unlimited production access, all platforms including NetSuite, QuickBooks Desktop and Sage Intacct.",
  "notes": "No prices published (October 2026)."
 },
 "agents": {
  "mcp": "official",
  "ai_features": "Rutter MCP (1 October 2026) for checking connection health, searching docs and reading accounting and commerce data from connections; AI card-acceptance analysis for supplier enablement; an 'agentic commerce' offering.",
  "agent_writes": "Not through MCP as far as found: the MCP is read-only by default and the post does not say writes are available. Writes go through the REST API as async jobs.",
  "approval": "None in the platform for API writes. The MCP is read-only by default, with permissions scoped per key; Rutter says it makes a best effort to strip credentials from responses before they reach the AI client.",
  "notes": "The MCP is aimed at Rutter's developer customers debugging their integrations, not at the end business."
 },
 "integration": {
  "scope": "75+ systems across accounting, ERP, commerce and payments per its homepage (the same page also says 60+): QuickBooks Online and Desktop, Xero, NetSuite, Sage Intacct, FreshBooks, Zoho Books, Dynamics, Shopify, Amazon, WooCommerce, Stripe, PayPal and others; ads platforms; bank feeds into accounting platforms. Embedded ERP covers nine ERPs; Embedded Digital eight accounting platforms.",
  "notes": "Customers deduplicate connections by store_unique_id. Delete webhooks follow <ENTITY>_DELETED with the final state for some types (BILL_DELETED carries id and deleted_at), and support varies by platform (PRODUCT_DELETED only on Shopify, WooCommerce and Wix)."
 },
 "preference": {
  "scale": "'Trusted by 100+ fintechs and banks' per its site (October 2026); named users include Mercury, PayPal, Payoneer, Airwallex, Aspire, Parafin and Numeral.",
  "ratings": "G2, Capterra and Trustpilot figures could not be confirmed in this pass.",
  "sentiment": "Not enough independent review data was readable. Its case studies (Orb shipping a first integration in 2.5 weeks, Airwallex cutting build time by 70%, per Rutter) are the vendor's own."
 },
 "timeline": [
  {
   "date": "2021-08-20",
   "event": "Comes out of stealth with $1.5M for its e-commerce API."
  },
  {
   "date": "2022-03-15",
   "event": "Raises $27M Series A led by Andreessen Horowitz; pitched as 'Plaid for commerce', with Ramp and Airwallex as customers."
  },
  {
   "date": "2024-08-31",
   "event": "Current API version, 2024-08-31."
  },
  {
   "date": "2026",
   "event": "Launches Embedded ERP for banks and fintechs: banking, payments and reconciliation inside NetSuite, Dynamics, Sage Intacct and SAP (exact date not confirmed)."
  },
  {
   "date": "2026-10-01",
   "event": "Launches Rutter MCP, read-only by default, for Claude, ChatGPT and Cursor."
  }
 ],
 "relation": "cautionary-tale",
 "relation_note": "Rutter reads the same accounting systems ownpurse reads, for fintechs and banks. Its handling of deletes, where a deleted record is removed from its cache and survives only as an event, is the history problem ownpurse exists to solve.",
 "borrow": "Dated API versions with a promise of no breaking changes inside a version; a delete webhook that carries the entity's final state; the rule that inactive is not deleted; a distinct status code (452) for limits hit at the source rather than at Rutter; read-only-by-default MCP with per-key scopes.",
 "avoid": "Removing a deleted record from the store so the only trace is an event a consumer may have missed; an initial sync measured in days with no partial results; delete detection that depends on the platform.",
 "sources": [
  {
   "title": "Rutter homepage",
   "url": "https://www.rutter.com/"
  },
  {
   "title": "Rutter pricing",
   "url": "https://www.rutter.com/pricing"
  },
  {
   "title": "Rutter about",
   "url": "https://www.rutter.com/about"
  },
  {
   "title": "Rutter Embedded ERP",
   "url": "https://www.rutter.com/embedded-erp"
  },
  {
   "title": "Introducing Rutter MCP (1 October 2026)",
   "url": "https://www.rutter.com/blog/introducing-rutter-mcp"
  },
  {
   "title": "Rutter blog",
   "url": "https://www.rutter.com/blog"
  },
  {
   "title": "Rutter API basics (2024-08-31)",
   "url": "https://docs.rutter.com/rest/2024-08-31/basics"
  },
  {
   "title": "Rutter webhooks",
   "url": "https://docs.rutter.com/implementation-guides/webhooks"
  },
  {
   "title": "Rutter: monitoring deleted objects",
   "url": "https://docs.rutter.com/implementation-guides/monitoring-deleted-objects"
  },
  {
   "title": "TechCrunch: a16z backs Rutter's goal of being 'Plaid for commerce' (March 2022)",
   "url": "https://techcrunch.com/2022/03/15/a16z-backs-rutters-goal-of-being-plaid-for-commerce/"
  },
  {
   "title": "TechCrunch: Rutter tag page",
   "url": "https://techcrunch.com/tag/rutter/"
  }
 ],
 "unverified": [
  "Funding after the March 2022 Series A (none found; web search budget was exhausted during this pass).",
  "Y Combinator batch and exact founding date.",
  "The exact launch date of Embedded ERP.",
  "Whether deleted records disappear from list endpoints immediately or only after Rutter's deletion query runs, and how often that runs.",
  "Whether Rutter MCP can be configured to write.",
  "The true count of supported platforms (the homepage says both 75+ and 60+).",
  "G2, Capterra and Trustpilot ratings.",
  "Current headcount and revenue."
 ],
 "researched": "2026-10-10"
}
---

## What it is

Rutter, a New York company that came out of stealth in August 2021, sells a unified API for business financial data: accounting and ERP, commerce, payments and ads, plus bank feeds into accounting platforms. It began as "Plaid for commerce" ([TechCrunch, March 2022](https://techcrunch.com/2022/03/15/a16z-backs-rutters-goal-of-being-plaid-for-commerce/)) and now describes itself as a unified API for B2B financial products, sold to fintechs and banks. In 2026 it added [Embedded ERP](https://www.rutter.com/embedded-erp), which puts a bank's payments, banking and reconciliation inside NetSuite, Dynamics, Sage Intacct and SAP.

## How it works

A business connects a platform through Rutter Link. Rutter runs an initial sync that its [docs](https://docs.rutter.com/rest/2024-08-31/basics) say can take hours to days, then refreshes on a cadence and serves most reads from its cache. Writes are asynchronous jobs that pass through to the platform's own limits. Real-time webhooks for creates and updates exist for QuickBooks, Xero, Stripe and Sage Intacct; elsewhere it uses incremental sync, and it tells customers to poll too because webhooks can be dropped.

Deletes are the weak point. The [deleted-objects guide](https://docs.rutter.com/implementation-guides/monitoring-deleted-objects) says: "When supported, Rutter will query for deleted data and remove it from its system." What remains is an `object_deleted` event or an `<ENTITY>_DELETED` [webhook](https://docs.rutter.com/implementation-guides/webhooks), so "you can remove them from your own database". If you missed the event, the record is simply gone. Support varies by platform, and marking a record inactive is not treated as a delete.

## Data and access

Authentication is HTTP Basic with a client id and secret, plus an access token per connection. Paid organisations get 500 requests per 10 seconds; free and sandbox 100. Versions are dated (`X-Rutter-Version: 2024-08-31`) and Rutter promises no breaking changes inside one. [Pricing](https://www.rutter.com/pricing) is a free 30-day sandbox, then a custom quote. The end business has no export.

## Agents

[Rutter MCP](https://www.rutter.com/blog/introducing-rutter-mcp), launched on 1 October 2026, connects Claude, ChatGPT or Cursor to a Rutter account to check connection health, search the docs and read accounting and commerce data. It is read-only by default with permissions scoped per key, and Rutter says it strips credentials from responses on a best-effort basis. It is a tool for Rutter's developers, not for the business whose data it is.

## Customers

The [homepage](https://www.rutter.com/) claims 100+ fintechs and banks and 75+ supported systems (the same page also says 60+), with Mercury, PayPal, Payoneer, Airwallex, Aspire and Parafin among the logos. Independent ratings could not be confirmed in this pass.

## History

It raised $1.5M at launch in 2021 and a $27M Series A led by Andreessen Horowitz in March 2022, when Ramp and Airwallex were customers. Since then its emphasis has moved from e-commerce toward accounting and ERPs, and in 2026 toward banks: supplier enablement, card acceptance analysis and Embedded ERP. This is the same drift toward banks that Codat made.

## For ownpurse

Rutter is the history problem in plain view. A unified API holds what the source says now. When the source deletes a bill, Rutter deletes it too, and the fact that the bill existed lives only in a webhook a consumer may never have received. ownpurse is built the other way: a delete is a new fact, the old values are retracted but kept, and `as_of` answers what the books said before. Rutter still does several things well that are worth copying: dated versions with no breaking changes, a delete event that carries the final state, a separate status for limits hit at the source, and an MCP server that is read-only unless you say otherwise.
