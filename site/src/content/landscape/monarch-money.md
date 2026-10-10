---
{
 "slug": "monarch-money",
 "name": "Monarch Money",
 "url": "https://www.monarchmoney.com",
 "segment": "personal-finance",
 "status": "active",
 "founded": "2018 (unverified; launched around 2019)",
 "hq": "United States, team distributed (exact HQ unverified)",
 "backing": "Venture-backed, privately held (Monarch Money, Inc.). Funding rounds and valuation were not confirmed in this pass (see unverified).",
 "tagline": "A subscription household finance app that aggregates every account through several data providers, with budgets, investments, forecasting and partner sharing.",
 "scope": "Monarch connects bank, card, loan, investment and asset accounts (it claims 13,000+ institutions through multiple data providers) and offers budgets, cash flow, recurring bills, goals, investment tracking, reports and a household view with a separate login for each partner and optional advisor access. It is personal finance only: no double-entry books, no business ledger, no tax filing, and no public developer API.",
 "domains": [
  "personal",
  "bank",
  "reporting"
 ],
 "customers_served": [
  "consumer",
  "freelancer"
 ],
 "spectrum": {
  "custody": {
   "score": 1,
   "note": "Monarch holds the record; you can download transactions as CSV, but there is no full-fidelity export or API."
  },
  "locality": {
   "score": 0,
   "note": "Hosted only: web, iOS and Android."
  },
  "history": {
   "score": 1,
   "note": "Keeps transaction and balance history, but edits overwrite in place; no versioned or as-of record for the user."
  },
  "openness": {
   "score": 1,
   "note": "Closed product with no public API; community libraries reverse-engineer the private web API."
  },
  "breadth": {
   "score": 2,
   "note": "Banks, cards, loans, investments and manual assets for a household; no books, documents or payments."
  },
  "autonomy": {
   "score": 2,
   "note": "Syncs, applies category rules and AI categorization unattended inside its own view; people review."
  },
  "agent_readiness": {
   "score": 1,
   "note": "An in-app AI assistant exists, but outside agents reach it only through unofficial libraries and community MCP servers that log in as the user."
  },
  "integration_ease": {
   "score": 1,
   "note": "No developer program; integrating means session-cookie or password-based unofficial access that can break."
  },
  "market": {
   "score": 0,
   "note": "Consumers, couples and households."
  }
 },
 "api": {
  "public": false,
  "style": "No public API. The web app uses a private API that community libraries (for example hammem/monarchmoney, MIT, 536 stars) call, with methods to read accounts, transactions and budgets and to create or edit transactions.",
  "auth": "Unofficial only: email and password with MFA, or saved browser session tokens.",
  "limits": "Not published.",
  "cost": "No API offering; access comes with the user's subscription.",
  "export": "Transaction CSV download in the app (no full JSON or ledger export confirmed).",
  "notes": "Because access is unofficial, it can break without notice and may sit outside the terms of service."
 },
 "pricing": {
  "model": "Subscription, two tiers (Core and Plus)",
  "entry": "$14.99 per month or $99.99 per year (pricing page, 2026-10); 7-day free trial",
  "top": "Plus tier (forecasting, multiple income streams, deeper investment tools); its price was not shown on the pricing page",
  "notes": "Frequent promotions such as 30% or 50% off the first annual Core year for new members billed directly. Ad-free and subscription-funded; partner and advisor seats included."
 },
 "agents": {
  "mcp": "community",
  "ai_features": "An in-app AI Assistant and AI transaction explanations (named in iOS release notes, versions 2.0.99 and 2.0.104); AI-assisted categorization.",
  "agent_writes": "Not officially. Community MCP servers (for example robcerda/monarch-mcp-server, 411 stars, 60 tools) can create, update, delete and split transactions and set budgets through the private API.",
  "approval": "No platform gate. The community MCP offers a read-only switch (MONARCH_MCP_READ_ONLY=1, which leaves 30 tools), off by default outside Docker, and recommends manual approval in the agent client.",
  "notes": "Demand for agent access is clearly there, but it runs on the user's own credentials and an unsupported API."
 },
 "integration": {
  "scope": "Claims 13,000+ institutions through multiple financial data providers (homepage, 2026-10); a public connection-status page reports success, longevity and update rates by institution and provider. Mint importer for transactions, categories and tags.",
  "notes": "Using more than one aggregator per institution is its main coverage advantage. It does not connect outward to accounting tools."
 },
 "preference": {
  "scale": "No user count published. iOS App Store rating 4.9 from 113K ratings (2026-10); 50,000+ Reddit community members (homepage, 2026-10).",
  "ratings": "iOS App Store 4.9 (113K ratings, 2026-10). Trustpilot 2.0 from 33 reviews (2026-10, uninvited, small sample). Named best overall budgeting app of 2024 by the Wall Street Journal; Forbes and USA Today lists, 2024.",
  "sentiment": "Reviewers praise the design, partner collaboration, customizable dashboard, rules and investment tracking, and it is the most common recommendation for former Mint users. Complaints cluster on sync reliability (disconnects, duplicates, sign errors, missing transactions, Canadian banks), slow email-only support, and price relative to that reliability."
 },
 "timeline": [
  {
   "date": "2018-2019",
   "event": "Founded by Val Agostino (CEO, early Mint product leader), Jon Sutherland and Ozzie Osman (dates unverified)."
  },
  {
   "date": "2023-11-09",
   "event": "Publishes a rebuilt Mint importer and a migration offer the week Intuit announces Mint's closure."
  },
  {
   "date": "2024",
   "event": "Wall Street Journal names it the best overall budgeting app of 2024."
  },
  {
   "date": "2024-2025",
   "event": "AI Assistant appears in the apps (per iOS release notes; launch date unverified)."
  },
  {
   "date": "2026-10",
   "event": "Pricing at $14.99 per month or $99.99 per year; claims 13,000+ institutions."
  }
 ],
 "relation": "neighbour",
 "relation_note": "Monarch is the household view ownpurse could feed but not replace: it serves the same people who want all their money in one place, but it keeps the record itself and offers no API, so a person or agent cannot treat it as a source or a destination they control.",
 "borrow": "Multi-aggregator coverage with a public per-institution connection-health page, the household model with separate logins and advisor access, and a Mint importer that maps categories and tags.",
 "avoid": "Making the vendor's copy the only full record, and leaving agents to reverse-engineer a private API with the user's password.",
 "sources": [
  {
   "title": "Monarch pricing",
   "url": "https://www.monarch.com/pricing"
  },
  {
   "title": "Monarch homepage",
   "url": "https://www.monarch.com/"
  },
  {
   "title": "Monarch about (founders)",
   "url": "https://www.monarch.com/about"
  },
  {
   "title": "Monarch press",
   "url": "https://www.monarch.com/press"
  },
  {
   "title": "Monarch vs Mint comparison",
   "url": "https://www.monarch.com/compare/mint-alternative"
  },
  {
   "title": "Mint shutting down: migrate to Monarch (2023-11-09)",
   "url": "https://www.monarch.com/mint-shutting-down-how-to-migrate-to-monarch-with-discount"
  },
  {
   "title": "Monarch connection status",
   "url": "https://www.monarch.com/connection-status"
  },
  {
   "title": "Monarch on the App Store",
   "url": "https://apps.apple.com/us/app/monarch-budget-track-money/id1459319842"
  },
  {
   "title": "Monarch on Trustpilot",
   "url": "https://www.trustpilot.com/review/monarchmoney.com"
  },
  {
   "title": "hammem/monarchmoney (unofficial Python API)",
   "url": "https://github.com/hammem/monarchmoney"
  },
  {
   "title": "robcerda/monarch-mcp-server (community MCP)",
   "url": "https://github.com/robcerda/monarch-mcp-server"
  }
 ],
 "unverified": [
  "Funding history and valuation (reported seed, Series A and a 2025 Series B were not confirmed from a primary source in this pass; web search budget was exhausted).",
  "Founding year and headquarters.",
  "Which data providers Monarch uses (Plaid, MX and Finicity are commonly reported; Monarch's pages do not name them).",
  "Plus tier price.",
  "AI Assistant launch date and which tier includes it.",
  "Exact scope of data export (balances history, CSV of all transactions)."
 ],
 "researched": "2026-10-10"
}
---

## What it is

Monarch is a subscription personal finance app for households. You link your accounts, and it gives you budgets, cash flow, recurring bills, goals, investment tracking and reports. Partners each get their own login to one shared view, and you can invite a financial advisor. Its co-founder and CEO Val Agostino was an early product leader at Mint, and after [Intuit closed Mint](https://www.monarch.com/mint-shutting-down-how-to-migrate-to-monarch-with-discount) Monarch became the best-known successor.

## How it works

Monarch is hosted on web, iOS and Android. It says it connects to 13,000+ institutions and "syncs with multiple financial data providers, more than other apps" ([homepage](https://www.monarch.com/)). If one aggregator is weak at a bank, it can use another, and a public [connection-status page](https://www.monarch.com/connection-status) reports success and longevity by institution and provider. Rules and AI categorization sort transactions, and you review them. The iOS release notes mention an in-app AI Assistant and AI transaction explanations.

## Data and access

Monarch has no public API. The data lives on Monarch's servers. You can download transactions as CSV, but there is no full export or ledger. Community libraries such as [hammem/monarchmoney](https://github.com/hammem/monarchmoney) call the private web API with your email, password and MFA, or with a saved session. They can read accounts, transactions and budgets and create or edit transactions, but they can break whenever Monarch changes its private API.

## Agents

People clearly want agent access. [robcerda/monarch-mcp-server](https://github.com/robcerda/monarch-mcp-server) exposes 60 tools to Claude, half of them writes: create, update, delete and split transactions, and set budgets. A read-only switch hides the writes, but it is off by default outside Docker, and the project recommends approving each mutating call in the client. Monarch itself provides no scoped tokens, read-only keys or approval step.

## Customers

Monarch does not publish a user count. Its iOS app is rated 4.9 from 113K ratings (October 2026), and the Wall Street Journal named it the best overall budgeting app of 2024. People praise the design, partner sharing and investment views. The complaints are about sync: disconnects, duplicates, wrong signs, missing transactions and some Canadian banks. People also complain of slow email-only support. Its [Trustpilot](https://www.trustpilot.com/review/monarchmoney.com) score of 2.0 rests on only 33 uninvited reviews.

## History

Monarch launched around 2019. Mint's closure in late 2023 gave it its biggest boost, and the Mint importer built for that moment maps categories and tags. Pricing in October 2026 is $14.99 a month or $99.99 a year for Core, with a Plus tier for forecasting and deeper investment tools.

## For ownpurse

Monarch is a neighbour, not a rival. It solves "all my money in one view" for a household, and it solves it well. It does that by being the custodian, with no supported way for you or your agent to take the record out whole. A person who uses Monarch and also runs a small business in Xero has two hosted silos and no shared record.

ownpurse could treat Monarch as one more source: a connector that appends what Monarch reports, with provenance. The community libraries show this is possible, but they are unofficial. The ideas worth borrowing are coverage from several aggregators and connection health in public. The thing to avoid is the agent story Monarch users are left with: an MCP server that holds your password and can delete transactions without an approval step.
