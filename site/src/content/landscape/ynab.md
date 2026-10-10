---
{
 "slug": "ynab",
 "name": "YNAB",
 "url": "https://www.ynab.com",
 "segment": "personal-finance",
 "status": "active",
 "founded": "2004 (the spreadsheet dates from early 2003)",
 "hq": "Fully remote team; United States company (founder based in Utah, unverified)",
 "backing": "Privately owned and founder-led (Jesse Mecham); no outside venture funding found. 115 employees, all remote, in 2020 (Wikipedia).",
 "tagline": "A subscription zero-based budgeting app (You Need A Budget) with a public read/write API and an unusually loyal user base.",
 "scope": "YNAB helps a person or household give every dollar a job: zero-based budgeting across categories, with accounts linked through Plaid or imported from files, goals and targets, and loan planning. It is a budget, not a ledger: no double-entry books, no invoicing, no business accounting, no investment performance tracking. It has a documented public API with personal access tokens and OAuth.",
 "domains": [
  "personal",
  "bank"
 ],
 "customers_served": [
  "consumer",
  "freelancer",
  "developers"
 ],
 "spectrum": {
  "custody": {
   "score": 1,
   "note": "YNAB holds the record, but the API returns the whole plan as JSON and file export exists, so you can keep a full copy."
  },
  "locality": {
   "score": 0,
   "note": "Hosted only since the 2015 move to subscription; the old local desktop YNAB 4 is retired."
  },
  "history": {
   "score": 1,
   "note": "Current-state records with deleted flags and delta sync through server_knowledge; no versioned or as-of history."
  },
  "openness": {
   "score": 2,
   "note": "Public, documented read/write API with full data access; closed source."
  },
  "breadth": {
   "score": 1,
   "note": "One domain: budgeting over bank and card accounts."
  },
  "autonomy": {
   "score": 1,
   "note": "Imported transactions wait for a person to approve them; auto-categorization only suggests."
  },
  "agent_readiness": {
   "score": 2,
   "note": "Public API with personal access tokens, a read-only OAuth scope and delta requests; MCP servers are community-built, none official."
  },
  "integration_ease": {
   "score": 3,
   "note": "Self-serve tokens, clear docs, official SDKs and a 'Works with YNAB' directory of 68 apps; limited by 200 requests per hour and OAuth review."
  },
  "market": {
   "score": 0,
   "note": "Individuals and households."
  }
 },
 "api": {
  "public": true,
  "style": "REST over JSON (v1, 'mostly read-only' with POST, PATCH, PUT and DELETE on transactions, scheduled transactions, categories and others); delta requests via server_knowledge / last_knowledge_of_server.",
  "auth": "Personal access tokens (do not expire, revocable) for the account owner; OAuth 2 implicit or authorization code grant with refresh tokens, 2-hour access tokens, optional read-only scope and PKCE. New OAuth apps are limited to 25 other users until reviewed.",
  "limits": "200 requests per hour per access token, rolling window; HTTP 429 on excess.",
  "cost": "No API fee; access requires an active YNAB subscription or trial (lapsed subscriptions get 403.1).",
  "export": "The API returns plans, accounts, categories, payees, months and transactions in full as JSON; the app exports plan and register data as CSV.",
  "notes": "API terms forbid handling users' bank credentials, require a 'not affiliated' line and 'for YNAB' naming, and let YNAB revoke access at any time."
 },
 "pricing": {
  "model": "Subscription, single tier",
  "entry": "$14.99 per month (pricing page, 2026-10); 34-day free trial with no card when signing up directly",
  "top": "$109 per year (about $9.08 per month, pricing page, 2026-10)",
  "notes": "Free year for college students; one subscription shares with up to several people. Repeated price increases are a common complaint in reviews; the dated price history was not confirmed."
 },
 "agents": {
  "mcp": "community",
  "ai_features": "None official in product (no AI on the homepage or in what's-new entries June to October 2026). Third-party tools in the directory: GPT for YNAB, LedgerGPT reconciliation, Receipts for YNAB with on-device Apple Intelligence.",
  "agent_writes": "Yes, through the public API with a full-access token: create, update and delete transactions, assign category budgets.",
  "approval": "No platform approval gate on API writes. Agents can be limited with the read-only OAuth scope. In the product, imported transactions sit unapproved until a person approves them, which works as a natural review step.",
  "notes": "Three community MCP servers are listed in the directory (Avenir-mcp, Local MCP Server for YNAB, MCP for YNAB)."
 },
 "integration": {
  "scope": "Bank linking through Plaid (US, UK under Plaid Financial Ltd., and others); file-based import of CSV, OFX and QFX with a built-in CSV converter (2026-09-14); 68 apps in the Works with YNAB directory (66 third-party, 2 official, 2026-10).",
  "notes": "Integration runs inward through the API, which is the easiest of the three personal finance tools here to build on."
 },
 "preference": {
  "scale": "No user count published; 'millions of people' helped (about page). 104,117 app reviews averaging 4.7 (homepage structured data, 2026-10).",
  "ratings": "Trustpilot 4.6 from 3,115 reviews (2026-10, 77% five-star, replies to 89% of negative reviews). App stores 4.7 average from 104K reviews (YNAB's own figure). Real Simple Best Budgeting App 2025; CNBC World's Top 250 Fintechs 2024.",
  "sentiment": "Users credit the method and the teaching (workshops, videos, support) with changing their finances, and they stay for years. Complaints are bank connections that need repair, occasional duplicates, limited search, and price increases."
 },
 "timeline": [
  {
   "date": "2003",
   "event": "Jesse Mecham builds the budgeting spreadsheet that becomes YNAB."
  },
  {
   "date": "2004",
   "event": "Released as standalone software."
  },
  {
   "date": "2015",
   "event": "Moves to a web subscription (the new YNAB); YNAB 4 desktop later retired."
  },
  {
   "date": "2018",
   "event": "Public API launched (date from memory, unverified)."
  },
  {
   "date": "2020",
   "event": "115 employees, all remote."
  },
  {
   "date": "2026-09-14",
   "event": "File-based import gains a CSV converter and the same rules as direct import."
  },
  {
   "date": "2026-10",
   "event": "$14.99 per month or $109 per year; API at 200 requests per hour per token."
  }
 ],
 "relation": "source",
 "relation_note": "YNAB is a natural ownpurse source for a household budget: its public API with delta sync and a read-only scope lets a connector append a person's plan and transactions as facts next to their business books, without scraping.",
 "borrow": "Delta sync with server_knowledge, a read-only OAuth scope, personal access tokens for the owner, the 'for YNAB' naming and non-affiliation rule, the unapproved state for imported transactions until a person confirms, and a directory of third-party tools.",
 "avoid": "Ending local ownership: the 2015 move from a desktop file to hosted-only subscription is still the most-cited reason long-time users left.",
 "sources": [
  {
   "title": "YNAB API documentation",
   "url": "https://api.ynab.com/"
  },
  {
   "title": "YNAB pricing",
   "url": "https://www.ynab.com/pricing"
  },
  {
   "title": "YNAB homepage",
   "url": "https://www.ynab.com/"
  },
  {
   "title": "YNAB about",
   "url": "https://www.ynab.com/about"
  },
  {
   "title": "YNAB what's new",
   "url": "https://www.ynab.com/whats-new"
  },
  {
   "title": "YNAB (Wikipedia)",
   "url": "https://en.wikipedia.org/wiki/YNAB"
  },
  {
   "title": "YNAB on Trustpilot",
   "url": "https://www.trustpilot.com/review/ynab.com"
  }
 ],
 "unverified": [
  "Exact headquarters city.",
  "Public API launch year (2018 recalled, not confirmed).",
  "Dated history of price increases (for example the move from about $99 to $109 per year).",
  "Which SDKs are official (JavaScript and Ruby recalled; Python not confirmed).",
  "Number of active users or subscribers.",
  "Countries where Plaid direct import is offered beyond the US and UK."
 ],
 "researched": "2026-10-10"
}
---

## What it is

YNAB (You Need A Budget) is a zero-based budgeting app. You give every dollar a job, move money between categories when plans change, and plan for irregular costs before they arrive. Jesse Mecham built the spreadsheet in 2003 and released the software in 2004, and the company is still founder-led. It was standalone software until it [moved to a subscription in 2015](https://en.wikipedia.org/wiki/YNAB).

## How it works

YNAB is hosted on web, iOS and Android. Accounts link through Plaid or come in as CSV, OFX or QFX files. A built-in CSV converter arrived on [2026-09-14](https://www.ynab.com/whats-new). Imported transactions wait unapproved until you approve them, so a person reviews each one. Card Mode, a swipe review added in September 2026, makes that review faster. YNAB does no double-entry, investment performance or business accounting.

## Data and access

Of the three apps here, YNAB is the one you can build on. The [public API](https://api.ynab.com/) is REST over JSON. The owner can use a personal access token, and other people's apps use OAuth with a read-only scope and PKCE. The API covers plans, accounts, categories, payees, months, scheduled transactions and transactions, with writes on the main resources. Delta requests return only what changed since your last `server_knowledge`. The limit is 200 requests per hour per token. The API itself is free but needs an active subscription. The terms forbid apps from touching bank credentials, require a non-affiliation line and "for YNAB" naming, and let YNAB revoke access at any time.

## Agents

YNAB has no official AI feature or MCP server. Its [Works with YNAB](https://api.ynab.com/) directory lists 68 apps, among them three community MCP servers, an unofficial GPT assistant, LedgerGPT for reconciliation and an on-device receipt splitter. An agent with a full token can create and delete transactions, and nothing on the platform side asks for approval. The read-only scope and the unapproved state of imports are the only guards.

## Customers

YNAB serves individuals and households. On [Trustpilot](https://www.trustpilot.com/review/ynab.com) it scores 4.6 from 3,115 reviews, and its app reviews average 4.7 across 104K (its own figure, October 2026). Users credit the method and the teaching. They complain about bank connections that need repair and about price increases. Pricing is $14.99 a month or $109 a year, with a 34-day trial.

## History

YNAB has stayed independent and profitable on subscriptions, with 115 remote staff in 2020. The 2015 move from a local desktop file to a hosted subscription was its biggest change, and long-time users still bring it up. Since then the work has gone into the method, education and the API.

## For ownpurse

YNAB is a source to connect early for the household side of the record. A connector can use the owner's personal access token, ask only for read-only data, and use `server_knowledge` as its cursor. Its deleted flags map to retractions, and the approved flag says whether a fact has been confirmed yet. YNAB keeps current state, and ownpurse would add the history: every version kept, checked against the source.

The design ideas worth borrowing are the read-only scope, a token the owner holds, and transactions that stay unapproved until a person confirms them. The thing to avoid is the 2015 move: once the record left the user's machine, it never came back.
