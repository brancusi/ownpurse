---
{
 "slug": "mx",
 "name": "MX",
 "url": "https://www.mx.com",
 "segment": "bank-aggregator",
 "status": "active",
 "founded": "2010 (not confirmed against a primary source in this pass)",
 "hq": "Lehi, Utah, US (3401 North Thanksgiving Way, Suite 500)",
 "backing": "Private, venture-backed (MX Technologies, Inc.). Funding history not confirmed in this pass; see unverified.",
 "tagline": "A US financial data platform that aggregates, cleanses and categorises account data, sold mainly to banks and credit unions.",
 "scope": "MX connects to financial institutions, aggregates accounts and transactions, cleans descriptions, assigns more than 100 categories and adds merchant data, then sells that data and the widgets, insights and analytics built on it to banks, credit unions and fintechs. It also runs FDX-standard data access for institutions sharing their own data. It does not keep books, and consumers reach it only through their bank's or an app's interface.",
 "domains": [
  "bank",
  "personal"
 ],
 "customers_served": [
  "developers",
  "mid-market",
  "enterprise"
 ],
 "spectrum": {
  "custody": {
   "score": 0,
   "note": "MX and its client institution hold the data; the person has no copy of their own."
  },
  "locality": {
   "score": 0,
   "note": "Cloud only, with IP allowlisting and normally US-only client addresses."
  },
  "history": {
   "score": 1,
   "note": "Webhooks for created, updated and deleted transactions and from/to_updated_at filters, up to 24 months of history, but current state only."
  },
  "openness": {
   "score": 2,
   "note": "Free self-serve developer keys and published OpenAPI specs; production is by contract and the service is closed."
  },
  "breadth": {
   "score": 2,
   "note": "Accounts, transactions, investments, verification, budgets, goals and insights; no books, documents or tax."
  },
  "autonomy": {
   "score": 1,
   "note": "Aggregates in the background every 24 hours and cleanses and categorises automatically; never writes to books."
  },
  "agent_readiness": {
   "score": 2,
   "note": "A broad REST API with an llms.txt docs index; a consumer AI assistant in early access; no MCP."
  },
  "integration_ease": {
   "score": 2,
   "note": "Developer keys and test banks are free, but production keys, IP allowlisting and mutual TLS go through MX."
  },
  "market": {
   "score": 3,
   "note": "Banks, credit unions and fintechs, up to large institutions."
  }
 },
 "api": {
  "public": true,
  "style": "REST (Platform API) with dated versions (v20111101, v20250224, v20260929); Connect Widget; webhooks; legacy Nexus API; FDX Data Access",
  "auth": "HTTP Basic with client_id and api_key; bearer tokens on some processor endpoints; optional mutual TLS; IP allowlist required for all requests",
  "limits": "Production per client per second: GET 2,000, POST and PUT 750, DELETE 150 (integration environment 300/100/100/50). 429 without Retry-After. Up to 25 members per user. Standard aggregation throttled to once every 3 hours per member by default; balance checks 5 per member per 2 hours.",
  "cost": "Developer API keys are free; production pricing is not published and goes through sales.",
  "export": "A client can read all accounts, transactions and holdings it aggregates through the API; there is no consumer export.",
  "notes": "Extended Transaction History returns up to 24 months. List Transactions accepts from/to_created_at and from/to_updated_at (added 31 July 2025). v20260929 (29 September 2026) added deprecation and sunset response headers. Cleansed descriptions, categories and merchant data are added on top of the institution's raw data."
 },
 "pricing": {
  "model": "Enterprise contracts; per-product pricing not published",
  "entry": "Free developer keys and integration environment (100 users)",
  "top": "Custom production contracts for institutions and fintechs",
  "notes": "No self-serve production plan was found; the open-banking page offers only demos."
 },
 "agents": {
  "mcp": "none",
  "ai_features": "A financial AI assistant for institutions' customers (announced 30 June 2026, early access); AI growth agents for financial institutions with Primitive (15 April 2026); an insights feed generated from transaction data.",
  "agent_writes": "The API can create and update users, members, budgets, goals, tags and transaction categories for the client's users; no agent-specific write interface.",
  "approval": "No approval gate documented beyond the end user's consent when connecting an account.",
  "notes": "MX's AI is aimed at banks keeping the customer relationship inside the bank's app (\"The financial relationship is moving to AI. Keep it inside your app.\"), not at giving a person or their agent the data."
 },
 "integration": {
  "scope": "Thousands of US institutions through aggregation and OAuth; the exact count was not shown in page text. Partners include digital banking platforms such as Lumin Digital (September 2026). It is the provider behind SimpleFIN Bridge.",
  "notes": "Integration can use the Connect Widget or the API alone. Non-US client IP addresses need MX review. Test institutions MX Bank and MXCU simulate OAuth, MFA and connection states."
 },
 "preference": {
  "scale": "No current customer or connection counts appear in text on mx.com (read 2026-10-10). Named to CNBC's list of top fintech companies for a fourth year (July 2026).",
  "ratings": "Trustpilot profile unclaimed with 0 reviews (read 2026-10-10); G2 could not be read (403).",
  "sentiment": "Few public end-user reviews, because consumers meet MX inside their bank's app. Developers and apps cite its cleansing and categorisation; downstream users of SimpleFIN Bridge report MX-side issues such as re-issued ids and truncated descriptions."
 },
 "timeline": [
  {
   "date": "2010",
   "event": "Founded in Utah by Ryan Caldwell (year not confirmed in this pass)."
  },
  {
   "date": "2025-07-31",
   "event": "Platform API adds from/to_updated_at filters on List Transactions."
  },
  {
   "date": "2026-03-30",
   "event": "MX leaders speak at the FDX Global Summit; MX co-chairs the FDX board."
  },
  {
   "date": "2026-04-15",
   "event": "Partners with Primitive to launch AI growth agents for financial institutions."
  },
  {
   "date": "2026-06-30",
   "event": "Announces a conversational financial AI assistant for institutions' customers."
  },
  {
   "date": "2026-09-29",
   "event": "Platform API v20260929 with breaking changes and deprecation headers."
  }
 ],
 "relation": "infrastructure",
 "relation_note": "MX is the layer under SimpleFIN Bridge, so if ownpurse uses SimpleFIN, MX's coverage and errors are what it sees. It is not a direct source for a person: production access is by contract with businesses.",
 "borrow": "Dated API versions with machine-readable deprecation headers; filtering by updated_at as well as date; keeping cleansed description, category and merchant as added data beside the raw line rather than in place of it.",
 "avoid": "A sales gate, IP allowlisting and US-only client addresses as the price of entry; and letting cleansed fields overwrite the institution's raw text, which is what the 32-character truncation downstream looks like.",
 "sources": [
  {
   "title": "MX homepage",
   "url": "https://www.mx.com/"
  },
  {
   "title": "MX company page",
   "url": "https://www.mx.com/company/"
  },
  {
   "title": "MX contact",
   "url": "https://www.mx.com/contact/"
  },
  {
   "title": "MX terms",
   "url": "https://www.mx.com/terms/"
  },
  {
   "title": "MX news",
   "url": "https://www.mx.com/news/"
  },
  {
   "title": "MX open banking",
   "url": "https://www.mx.com/products/open-banking/"
  },
  {
   "title": "MX docs index (llms.txt)",
   "url": "https://docs.mx.com/llms.txt"
  },
  {
   "title": "MX docs: manage API keys",
   "url": "https://docs.mx.com/resources/client-dashboard/manage-api-keys.md"
  },
  {
   "title": "MX docs: changelog",
   "url": "https://docs.mx.com/resources/changelog/index.md"
  },
  {
   "title": "MX docs: rate limits",
   "url": "https://docs.mx.com/api-reference/platform-api/v20260929/overview/rate-limits.md"
  },
  {
   "title": "MX docs: aggregation",
   "url": "https://docs.mx.com/api-reference/platform-api/v20260929/overview/aggregation.md"
  },
  {
   "title": "MX docs: authentication and security",
   "url": "https://docs.mx.com/api-reference/platform-api/v20260929/overview/authentication-and-security.md"
  },
  {
   "title": "MX docs: extended transaction history",
   "url": "https://docs.mx.com/products/connectivity/extended-transaction-history/index.md"
  },
  {
   "title": "SimpleFIN Bridge security (MX as provider)",
   "url": "https://beta-bridge.simplefin.org/info/security"
  },
  {
   "title": "Trustpilot: MX",
   "url": "https://www.trustpilot.com/review/mx.com"
  }
 ],
 "unverified": [
  "Founding year (2010) and co-founders other than Ryan Caldwell.",
  "Funding history, including a reported $300M Series C at about $1.9B in January 2021; not confirmed against a primary source because web search was exhausted.",
  "Leadership changes between 2022 and 2026 (a different CEO for a period is reported); the company page lists Ryan Caldwell as founder and CEO as of 2026-10-10.",
  "Counts of connected institutions, client institutions, fintechs and end users; figures on mx.com are in images, not text.",
  "G2 and Capterra ratings (G2 returned 403).",
  "Production pricing."
 ],
 "researched": "2026-10-10"
}
---

## What it is

MX is a US financial data platform based in Lehi, Utah ([contact](https://www.mx.com/contact/)). It aggregates accounts and transactions from financial institutions, cleans them up, and sells the result, with widgets, insights and analytics, mainly to banks and credit unions, and to fintechs. Its homepage leads with "Leading Financial Intelligence Platform" and outcomes for its bank clients, such as deposit switches and loan engagement ([mx.com](https://www.mx.com/)). Founder Ryan Caldwell is listed as CEO, and its head of public policy co-chairs the FDX board ([company page](https://www.mx.com/company/)).

## How it works

A client creates users, and each user has members, which are connections to institutions, up to 25 per user. MX [aggregates each member in the background](https://docs.mx.com/api-reference/platform-api/v20260929/overview/aggregation.md) about every 24 hours; a client can also start an aggregation while the user is present for MFA, throttled to once every three hours by default. [Extended Transaction History](https://docs.mx.com/products/connectivity/extended-transaction-history/index.md) returns up to 24 months, cleansed and put into more than 100 categories with merchant, direct deposit and subscription flags. Webhooks report transactions created, updated or deleted, and List Transactions can filter by `updated_at` (from July 2025). The API is versioned by date; [v20260929](https://docs.mx.com/resources/changelog/index.md) adds headers that announce deprecations.

## Data and access

[Developer keys are free](https://docs.mx.com/resources/client-dashboard/manage-api-keys.md) after sign-up, with an integration environment of up to 100 users and test banks. Production keys are requested from MX, and pricing is not published. Requests use Basic auth with a client id and key, [must come from allowlisted IP addresses](https://docs.mx.com/api-reference/platform-api/v20260929/overview/authentication-and-security.md), and non-US addresses need review. [Rate limits](https://docs.mx.com/api-reference/platform-api/v20260929/overview/rate-limits.md) are generous, 2,000 GET requests a second in production. MX also runs FDX-standard Data Access, letting institutions share the data they hold with other platforms.

## Agents

MX has no MCP server. Its AI work serves its bank clients: a conversational assistant for their customers (June 2026, early access) and AI growth agents built with Primitive (April 2026) ([news](https://www.mx.com/news/)). The pitch is explicit: "The financial relationship is moving to AI. Keep it inside your app." The docs publish an `llms.txt` index and OpenAPI specs, which make the API easy for an agent to learn.

## Customers

Consumers meet MX inside their bank's app, so there are few public reviews; the [Trustpilot profile](https://www.trustpilot.com/review/mx.com) has none. Its most visible footprint for individuals is indirect: [SimpleFIN Bridge outsources bank connections to MX](https://beta-bridge.simplefin.org/info/security), so the Bridge's public issues (re-issued Capital One ids, truncated descriptions) show MX-side data as a person receives it.

## History

MX was founded in Utah by Ryan Caldwell, reportedly in 2010, and grew by selling data cleansing and money-management tools to banks and credit unions before most of them had their own. Its funding history could not be confirmed against a primary source in this pass. In 2026 it repositioned around AI for financial institutions and open finance through FDX.

## For ownpurse

MX matters to ownpurse mostly as the layer under SimpleFIN. If ownpurse's bank witness comes through SimpleFIN, MX's coverage, cleansing and mistakes are what the record receives, so the record should keep the raw lines as they arrive and treat any cleaned name or category as a separate, later fact. MX's own design has two good habits worth copying: dated API versions that announce their own deprecations, and filtering by when a record changed, not only when it happened. As a direct source it does not fit: production access is a contract with a business, behind an IP allowlist.
