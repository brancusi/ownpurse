---
{
 "slug": "codat",
 "name": "Codat",
 "url": "https://www.codat.io",
 "segment": "unified-api",
 "status": "pivoted",
 "founded": "2017",
 "hq": "London, UK (Codat Limited, company number 10480375)",
 "backing": "Venture-backed. $10M Series A led by Index Ventures (June 2020); $40M led by Tiger Global (July 2021), per TechCrunch. Codat names J.P. Morgan, PayPal, Amex, Plaid and Shopify as backers (2026). A 2022 Series C led by J.P. Morgan was not confirmed against a primary source in this pass.",
 "tagline": "A unified API for small-business accounting, banking and commerce data that in 2026 repositioned itself as an advisory intelligence platform for commercial banks.",
 "scope": "Codat connects to a business's accounting software (about 20 platforms), commerce platforms (about 18) and bank data (through Plaid and TrueLayer), syncs that data into its own cloud and serves it through use-case APIs: Lending, Bank Feeds, Bill Pay, Expenses and, since 2026, Spend Insights. It also writes back (bills, payments, bank feed lines) as asynchronous push operations. It does not keep versions of records, does not give the business owner a copy, and no longer presents a general-purpose accounting API in its docs navigation.",
 "domains": [
  "integration",
  "ledger",
  "bank",
  "payments",
  "spend",
  "reporting",
  "commerce"
 ],
 "customers_served": [
  "enterprise",
  "mid-market",
  "developers"
 ],
 "spectrum": {
  "custody": {
   "score": 1,
   "note": "A third-party cache in Codat's Azure cloud beside the source; the source stays authoritative and the business owner holds no copy."
  },
  "locality": {
   "score": 0,
   "note": "Cloud only; enterprise clients can get dedicated databases and customer keys, still hosted by Codat."
  },
  "history": {
   "score": 1,
   "note": "Since April 2023 records deleted between syncs are kept with metadata.isDeleted; push operations have a history; no prior versions of a record."
  },
  "openness": {
   "score": 2,
   "note": "Public docs, public OpenAPI specs and open-source SDKs on GitHub; closed service, no public pricing."
  },
  "breadth": {
   "score": 3,
   "note": "Accounting, commerce, banking, bank feeds, bill pay and expenses across about 40 platforms; no documents or receipts as a domain."
  },
  "autonomy": {
   "score": 3,
   "note": "Its clients' software reads and pushes writes (bills, payments, feed lines) unattended; no approval step in the platform."
  },
  "agent_readiness": {
   "score": 2,
   "note": "Public REST API with SDKs; no MCP server or agent interface found; its llms.txt forbids training on the docs."
  },
  "integration_ease": {
   "score": 2,
   "note": "Well-documented API and Link auth flow, but sales-led, no public pricing page, and the product is now pitched to banks."
  },
  "market": {
   "score": 4,
   "note": "Commercial banks (BMO, J.P. Morgan) and large lenders and fintechs."
  }
 },
 "api": {
  "public": true,
  "style": "REST over HTTPS with JSON; asynchronous push operations for writes; webhooks for sync and connection events; OpenAPI specs published on GitHub.",
  "auth": "API key in the Authorization header for the client; end businesses connect their platforms through Codat's Link flow (OAuth to the source).",
  "limits": "1,000 x (1 + number of active connected companies) requests per day and 10 concurrent requests per connected company, reset 00:00 UTC; 429 with Retry-After. Source platforms' own limits handled by Codat.",
  "cost": "Not public in October 2026 (the pricing page returns 404). In 2021 TechCrunch reported pricing based on the number of companies connected. Source-side costs (for example Xero's egress fees) are carried by Codat and passed on in its contract, not itemised publicly.",
  "export": "No owner export: the client (a bank or fintech) reads data by API; the business whose books they are has no Codat export of its own.",
  "notes": "Records created and deleted before the first sync are never seen. A connection must refresh at least one data type monthly or its token expires. Legacy products are being retired: Visualize removed October 2025, Sync for Commerce v1 January 2026, Sync for Payables v1, Sync for Expenses v1 and Sync for Payroll in maintenance from 1 May 2026 for 12 months."
 },
 "pricing": {
  "model": "Contract-based, historically per connected company.",
  "entry": "No public entry price found (October 2026).",
  "top": "Enterprise and bank contracts, not published.",
  "notes": "Any earlier free or self-serve tier could not be confirmed in this pass."
 },
 "agents": {
  "mcp": "none",
  "ai_features": "Advisory intelligence for banks (May 2026): FX Insights, Spend Insights and Working Capital Insights, described as AI-powered insights over clients' off-bank data; an accounting score update (January 2026).",
  "agent_writes": "No agent interface. Writes are API push operations made by the client's software.",
  "approval": "None in the platform. Writes go as asynchronous push operations whose status is recorded in a push history; any approval lives in the client's own product.",
  "notes": "Codat's AI is aimed at bankers reading insights, not at agents driving the API. Its llms.txt disallows commercial AI training on its documentation."
 },
 "integration": {
  "scope": "About 20 accounting platforms (Xero, QuickBooks Online and Desktop, NetSuite, Sage Intacct, Sage 50/200, Dynamics 365, MYOB, FreeAgent, Zoho Books, Exact Online and others), about 18 commerce platforms (Shopify, Stripe, Square, Amazon Seller Central, PayPal and others) and banking through Plaid and TrueLayer; bank feeds into Xero, QuickBooks and Sage.",
  "notes": "Over 350,000 connections to business customers' systems, per Codat (April and May 2026). Codat is also a supplier of bank feeds into accounting platforms, which makes it a source for those ledgers, not only a reader."
 },
 "preference": {
  "scale": "Over 350,000 connections to business customers' financial systems and work with 'over a dozen' of the largest commercial banks, per Codat (April 2026). About 150 staff in July 2021 per TechCrunch; no current headcount found.",
  "ratings": "Trustpilot 3.7 from 1 review (July 2018), effectively unrated. G2 and Capterra could not be read in this pass.",
  "sentiment": "Not enough independent review data was readable to summarise customer sentiment. What can be said from primary sources: Codat has retired or frozen several products since 2025, which existing developer customers have to migrate off."
 },
 "timeline": [
  {
   "date": "2017",
   "event": "Founded in London."
  },
  {
   "date": "2020-06-10",
   "event": "Raises $10M Series A led by Index Ventures."
  },
  {
   "date": "2021-07-01",
   "event": "Raises $40M led by Tiger Global; about 150 staff, ARR tripled in 2020."
  },
  {
   "date": "2023-04-11",
   "event": "Starts keeping records deleted between syncs, flagged metadata.isDeleted, instead of dropping them."
  },
  {
   "date": "2025-10-03",
   "event": "Announces legacy product deprecations: Visualize and Commerce Visualize removed 31 October 2025, Sync for Commerce v1 and Xero Bank Feeds v1 removed early 2026."
  },
  {
   "date": "2026-03-30",
   "event": "Announces Sync for Payables v1, Sync for Expenses v1 and Sync for Payroll enter maintenance on 1 May 2026, retired after 12 months."
  },
  {
   "date": "2026-04-20",
   "event": "Rebrands around 'advisory intelligence' for commercial banks: 'Banking on better data'."
  },
  {
   "date": "2026-05-05",
   "event": "Launches its advisory intelligence platform with FX, Spend and Working Capital Insights; BMO among the first banks."
  }
 ],
 "relation": "neighbour",
 "relation_note": "Codat pulls the same Xero and QuickBooks data ownpurse pulls, but for a bank or lender to read, in Codat's cloud. Its move from developer infrastructure to bank insight shows where the money in a unified API went: to the party reading the books, not the owner.",
 "borrow": "Keeping deleted records with a flag instead of dropping them (since 2023), the push-operation history for writes, and an explicit lifecycle policy (active, maintenance, deprecated, retired) with dated notices.",
 "avoid": "A current-state cache with only a deleted flag, records that were created and deleted before the first sync never seen, and a product line that can move to a new buyer and leave its developers to migrate.",
 "sources": [
  {
   "title": "Codat launches advisory intelligence platform (5 May 2026)",
   "url": "https://codat.io/resources/codat-launches-advisory-intelligence-platform-bringing-real-time-data-ai-solutions-to-commercial-banks/"
  },
  {
   "title": "The story behind our new brand (20 April 2026)",
   "url": "https://codat.io/resources/codat-rebrand/"
  },
  {
   "title": "Advisory Intelligence: why we built it (16 April 2026)",
   "url": "https://codat.io/resources/why-we-built-advisory-intelligence/"
  },
  {
   "title": "Codat Docs",
   "url": "https://docs.codat.io/"
  },
  {
   "title": "Codat Docs updates",
   "url": "https://docs.codat.io/updates"
  },
  {
   "title": "Codat docs source: rate limits",
   "url": "https://github.com/codatio/codat-docs/blob/main/docs/using-the-api/rate-limits.md"
  },
  {
   "title": "Codat docs source: changes to handling of deleted data (2023)",
   "url": "https://github.com/codatio/codat-docs/blob/main/blog/230411-deletion-of-data.md"
  },
  {
   "title": "Codat docs source: legacy product deprecations (October 2025)",
   "url": "https://github.com/codatio/codat-docs/blob/main/blog/251003-deprecation-of-legacy-products.md"
  },
  {
   "title": "Codat docs source: product lifecycle changes (March 2026)",
   "url": "https://github.com/codatio/codat-docs/blob/main/blog/260330-product-lifecycle-changes.md"
  },
  {
   "title": "Codat OpenAPI specs",
   "url": "https://github.com/codatio/oas"
  },
  {
   "title": "TechCrunch: Codat raises $10 million (June 2020)",
   "url": "https://techcrunch.com/2020/06/10/codat-raises-10-million-to-aggregate-all-financial-data-from-small-businesses/"
  },
  {
   "title": "TechCrunch: Codat raises $40M from Tiger (July 2021)",
   "url": "https://techcrunch.com/2021/07/01/codat-raises-40m-from-tiger-for-its-business-focused-fintech-app-service/"
  },
  {
   "title": "Trustpilot: Codat",
   "url": "https://www.trustpilot.com/review/codat.io"
  }
 ],
 "unverified": [
  "A $100M Series C led by J.P. Morgan in 2022 at an $825M valuation is widely reported but was not confirmed against a primary source in this pass.",
  "Total funding raised to date.",
  "Founders other than CEO Pete Lord (named as CEO in 2021); current CEO in 2026.",
  "Current pricing and whether any free or self-serve tier still exists.",
  "Whether the general Accounting API is still sold to new developer customers or only maintained.",
  "Any layoffs or headcount changes 2023-2026.",
  "G2 and Capterra ratings (pages not readable)."
 ],
 "researched": "2026-10-10"
}
---

## What it is

Codat is a London company, founded in 2017, that built one of the first unified APIs for small-business financial data: connect to a business's accounting software, bank and commerce platforms once and read it all in one shape. In 2026 it changed what it says it is. A [rebrand on 20 April 2026](https://codat.io/resources/codat-rebrand/) and a [launch on 5 May 2026](https://codat.io/resources/codat-launches-advisory-intelligence-platform-bringing-real-time-data-ai-solutions-to-commercial-banks/) recast it as an "advisory intelligence" platform for commercial banks, with BMO among the first banks. The connections are still there underneath; they are no longer the product it leads with.

## How it works

A business authorises Codat through its Link flow. Codat syncs that business's data into its own cloud and serves it through use-case APIs: Lending, Bank Feeds, Bill Pay, Expenses and Spend Insights, plus a Platform API for companies and connections. The general accounting API that older integrations used no longer appears in the [docs navigation](https://docs.codat.io/), though its OpenAPI spec is still in the [public spec repository](https://github.com/codatio/oas). Writes are asynchronous push operations with a status you poll or receive by webhook, and a push history.

On deletes Codat is better than some of its peers. Since [11 April 2023](https://github.com/codatio/codat-docs/blob/main/blog/230411-deletion-of-data.md) a record deleted in the source between two syncs is kept and flagged `metadata.isDeleted`. Anything created and deleted before the first sync is never seen, and a changed record overwrites the old one: there is no history of versions.

## Data and access

The client's [rate limit](https://github.com/codatio/codat-docs/blob/main/docs/using-the-api/rate-limits.md) is 1,000 x (1 + active connected companies) requests a day, 10 concurrent per company. Pricing is not public; the pricing page returned 404 in October 2026, and in 2021 [TechCrunch](https://techcrunch.com/2021/07/01/codat-raises-40m-from-tiger-for-its-business-focused-fintech-app-service/) reported it was priced by connected company. The business whose books these are gets no export: the data flows to Codat's client.

Codat is also retiring products. Visualize went in October 2025; Sync for Commerce v1 in early 2026; Sync for Payables v1, Sync for Expenses v1 and Sync for Payroll entered a 12-month [maintenance period](https://github.com/codatio/codat-docs/blob/main/blog/260330-product-lifecycle-changes.md) on 1 May 2026, Payroll with no replacement.

## Agents

No MCP server or agent interface was found. Codat's AI is for bankers: FX, Spend and Working Capital Insights that turn a client's accounting data into conversation starters. Its docs' llms.txt forbids commercial AI training on the documentation.

## Customers

Codat reports over 350,000 connections and work with over a dozen of the largest commercial banks, J.P. Morgan and BMO among them ([April 2026](https://codat.io/resources/why-we-built-advisory-intelligence/)). Named users include Revolut, PayPal, Expensify, Capital on Tap and Uncapped. Independent reviews are thin: Trustpilot shows one review, from 2018.

## History

Index Ventures led a $10M Series A in June 2020; Tiger Global led $40M in July 2021, when ARR had tripled in a year and staff were near 150. The company spent the next years building lending, bill pay and expense products on the same connections, then from 2025 cut legacy products and in 2026 turned to the banks that had become its best buyers.

## For ownpurse

Codat is the clearest case of what a unified API is for: a third party reading a business's books, with the business's consent, for the third party's purpose. That is the opposite direction to ownpurse, where the owner reads their own books and keeps the record. Codat is not a competitor and not a useful source, since it adds a hop, a cost and a lossy model between ownpurse and Xero. What it does well is worth copying: keep the deleted record and say it was deleted, keep a history of every push, and publish a lifecycle with dates. The 2026 repositioning is the lesson: the money in this segment followed the reader of the data, not the owner of it.
