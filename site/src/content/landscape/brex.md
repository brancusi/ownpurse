---
{
 "slug": "brex",
 "name": "Brex",
 "url": "https://www.brex.com",
 "segment": "spend-management",
 "status": "acquired",
 "founded": "January 2017",
 "hq": "San Francisco, US",
 "backing": "Acquired by Capital One Financial (NYSE: COF); agreement 22 January 2026 at $5.15 billion, completed 7 April 2026 for about $2.56 billion in cash plus 10,646,306 Capital One shares. Before that, about $2.3 billion in venture equity, last valued at $12.3 billion (October 2021).",
 "tagline": "Corporate cards, business banking, bill pay, travel and expense software in one platform, now part of Capital One.",
 "scope": "Brex combines corporate cards (issued globally), business accounts, bill pay, reimbursements, travel booking and expense management, with AI-assisted GL coding, accruals and two-way ERP sync. It does not keep a general ledger; it prepares and exports spend to yours. Since April 2026 it is a Capital One business, led by its founder Pedro Franceschi.",
 "domains": [
  "spend",
  "bank",
  "payments",
  "documents",
  "integration"
 ],
 "customers_served": [
  "smb",
  "mid-market",
  "enterprise",
  "developers"
 ],
 "spectrum": {
  "custody": {
   "score": 1,
   "note": "Brex holds the card, banking and expense record; you get ERP exports, statements and API reads, not the authoritative history."
  },
  "locality": {
   "score": 0,
   "note": "Cloud only; the MCP is a remote server."
  },
  "history": {
   "score": 1,
   "note": "Records carry export status and API logs, but nothing found offers versions or as-of queries."
  },
  "openness": {
   "score": 2,
   "note": "Closed product with an OpenAPI-defined public API included on every plan; the Accounting API is alpha and gated."
  },
  "breadth": {
   "score": 3,
   "note": "Cards, business banking, payments (ACH, wires, checks), bill pay, travel and expenses in one place; no ledger, payroll or tax."
  },
  "autonomy": {
   "score": 2,
   "note": "Brex says 71% of expenses are handled entirely by automations and offers one-click accruals, but no unattended posting agent was confirmed."
  },
  "agent_readiness": {
   "score": 3,
   "note": "Remote MCP (beta) with OAuth for Claude, ChatGPT, Cursor and Codex; mostly read tools with a few writes; no CLI found."
  },
  "integration_ease": {
   "score": 3,
   "note": "Clear REST docs, OpenAPI, webhooks and generous limits; tokens need an admin to accept the API agreement, and the Accounting API needs Brex's approval."
  },
  "market": {
   "score": 3,
   "note": "Venture-backed startups to global enterprises (DoorDash, Robinhood, Intel); left non-venture SMBs in 2022."
  }
 },
 "api": {
  "public": true,
  "style": "REST with JSON, defined in OpenAPI; webhooks; remote MCP server (beta)",
  "auth": "User tokens created by account or card admins after accepting the Developer API agreement, with chosen scopes; tokens expire after 90 days unused; OAuth for partner apps and for the MCP (with Dynamic Client Registration)",
  "limits": "1,000 requests per 60 seconds per Client ID and account; 1,000 transfers, 100 international wires and 5,000 cards created per 24 hours; higher limits by request",
  "cost": "API access is listed as included on Essentials, Premium and Enterprise; no usage fee found",
  "export": "Transactions, accounts, statements, expenses with receipts, budgets, vendors and payments through the API; CSV and statement exports in the app",
  "notes": "APIs cover accounting, budgets and limits, expenses, fields, onboarding, payments, team, transactions, travel and webhooks. The Accounting API (1.0.0-alpha) lets an integration read accounting records and report export success or failure; access by email to developer support."
 },
 "pricing": {
  "model": "Per-user SaaS tiers on top of card and banking revenue",
  "entry": "Essentials: $0 per user per month, with NetSuite and other accounting integrations and API access",
  "top": "Enterprise: custom, unlimited entities, local card issuance, named account manager",
  "notes": "Premium is $12 per user per month and adds custom ERP and HRIS integrations, AI compliance audit and AI-suggested accounting mapping. A separate Smart Card plan is custom. Checked on brex.com/pricing, 10 October 2026."
 },
 "agents": {
  "mcp": "official",
  "ai_features": "Brex AI expense assistant for employees and managers, AI custom rules, AI compliance audit, AI-suggested GL mapping, AI invoice entry for bill pay, one-click accruals.",
  "agent_writes": "Limited. The MCP can bulk-update expense memos, upload receipts, replace attendees and assign expenses to spend limits; approvals and card management are not available through MCP. The REST API can create payments, cards and transfers.",
  "approval": "Brex's MCP docs recommend turning on human confirmation in the AI client; access follows each user's Brex permissions, and MCP activity appears in the dashboard API logs. No approval gate is enforced by Brex itself.",
  "notes": "The MCP must be switched on by an admin under beta features ('Brex in AI assistants'); restricted API tokens are recommended for clients without OAuth."
 },
 "integration": {
  "scope": "Two-way ERP and accounting integrations including NetSuite, plus HRIS; Brex says 'thousands' of integrations",
  "notes": "The integration list was not checked one by one; the pricing page puts NetSuite on the free tier."
 },
 "preference": {
  "scale": "More than 25,000 companies in over 50 countries (Capital One, 22 January 2026); the Brex homepage says 35,000+ (October 2026).",
  "ratings": "Trustpilot 1.8 of 5 from 576 reviews, 14 in the last 12 months (October 2026). G2 and Capterra pages could not be read.",
  "sentiment": "Praise for responsive support from some reviewers and a well-built app. Complaints about frozen or closed accounts, unexplained application denials, repeated verification requests, eligibility changes that dropped long-time customers, and time-consuming expense and travel tools."
 },
 "timeline": [
  {
   "date": "2017-01",
   "event": "Founded by Henrique Dubugras and Pedro Franceschi; joins Y Combinator Winter 2017"
  },
  {
   "date": "2018-10",
   "event": "$125 million Series C at $1.1 billion"
  },
  {
   "date": "2021-10",
   "event": "$300 million at a $12.3 billion valuation"
  },
  {
   "date": "2022-04",
   "event": "Brex Empower spend platform launched"
  },
  {
   "date": "2022-06",
   "event": "Exits small and midsize businesses that were not venture-backed, to focus on larger companies"
  },
  {
   "date": "2023-03-09",
   "event": "Takes in billions in deposits from Silicon Valley Bank customers the day before SVB fails"
  },
  {
   "date": "2024-01",
   "event": "Lays off about 300 people, roughly 20% of staff"
  },
  {
   "date": "2025-08",
   "event": "Licensed to operate in the European Union"
  },
  {
   "date": "2026-01-22",
   "event": "Capital One agrees to buy Brex for $5.15 billion in cash and stock"
  },
  {
   "date": "2026-04-07",
   "event": "Acquisition completes; Franceschi continues to lead Brex inside Capital One"
  }
 ],
 "relation": "source",
 "relation_note": "Brex is a source of card lines, bank transactions, bills and receipts, not a competitor. Its sale to Capital One shows spend data moving into banks.",
 "borrow": "MCP docs that tell users to turn on human confirmation, MCP activity in the API logs, restricted tokens, and tokens that expire after 90 days unused. Separate daily limits for actions that move money (transfers, wires, cards) on top of the request limit. An accounting API where the integration reports export success or failure back to the source.",
 "avoid": "Depending on a vendor that can change who it serves: the 2022 exit from non-venture SMBs and Trustpilot reports of frozen accounts show customers can lose access with little notice. Gating the accounting feed behind an alpha programme.",
 "sources": [
  {
   "title": "Capital One to acquire Brex, press release (22 January 2026)",
   "url": "https://www.sec.gov/Archives/edgar/data/927628/000119312526019543/d97758dex991.htm"
  },
  {
   "title": "Capital One 8-K: completion of the Brex acquisition (7 April 2026)",
   "url": "https://www.sec.gov/Archives/edgar/data/927628/000119312526145764/0001193125-26-145764.txt"
  },
  {
   "title": "Brex pricing",
   "url": "https://www.brex.com/pricing"
  },
  {
   "title": "Brex homepage",
   "url": "https://www.brex.com/"
  },
  {
   "title": "Brex developer platform",
   "url": "https://developer.brex.com/"
  },
  {
   "title": "Brex API rate limits",
   "url": "https://developer.brex.com/guides/rate_limits.md"
  },
  {
   "title": "Brex API authentication",
   "url": "https://developer.brex.com/guides/authentication.md"
  },
  {
   "title": "Brex Accounting API (alpha)",
   "url": "https://developer.brex.com/openapi/accounting_api.md"
  },
  {
   "title": "Brex MCP",
   "url": "https://developer.brex.com/docs/mcp"
  },
  {
   "title": "Wikipedia: Brex",
   "url": "https://en.wikipedia.org/wiki/Brex"
  },
  {
   "title": "Trustpilot: Brex",
   "url": "https://www.trustpilot.com/review/brex.com"
  }
 ],
 "unverified": [
  "Whether Brex's automations post to the ERP unattended or wait for a person; the 71% automation figure is Brex's own (March 2025 internal metrics)",
  "Customer count: 25,000+ (Capital One, January 2026) against 35,000+ on the Brex homepage (October 2026)",
  "The full list and count of ERP integrations ('thousands' is Brex's word)",
  "Brex AI agent product names and launch dates beyond the features on the pricing page and homepage",
  "G2 and Capterra ratings could not be retrieved",
  "Brex revenue: not disclosed in the Capital One release",
  "Any change to Brex plans, API terms or branding since the Capital One close"
 ],
 "researched": "2026-10-10"
}
---

## What it is

Brex sells corporate cards, business accounts, bill pay, reimbursements, travel and expense software as one platform. Like Ramp, it does not keep your books. It prepares spend with AI-assisted GL coding, one-click accruals and per-entity rules, then syncs it to your ERP. Brex says 71% of expenses prepared on Brex are handled entirely by automations ([homepage](https://www.brex.com/)).

Since 7 April 2026 Brex is part of Capital One. The deal was announced on 22 January 2026 at $5.15 billion ([Capital One release](https://www.sec.gov/Archives/edgar/data/927628/000119312526019543/d97758dex991.htm)) and closed for about $2.56 billion in cash plus 10,646,306 Capital One shares ([8-K](https://www.sec.gov/Archives/edgar/data/927628/000119312526145764/0001193125-26-145764.txt)). That is well under the $12.3 billion valuation of October 2021. Pedro Franceschi still runs Brex.

## How it works

Brex earns on cards and banking, so the software tiers are cheap: Essentials is $0 per user, Premium is $12 per user per month, Enterprise is on contract ([pricing](https://www.brex.com/pricing)). Every plan includes accounting integrations (NetSuite even on the free tier) and API access. Premium adds custom ERP mappings and AI compliance audits.

## Data and access

The [developer platform](https://developer.brex.com/) is REST, defined in OpenAPI, with webhooks. An admin has to accept the API agreement before anyone can create tokens. Tokens have scopes and expire after 90 days unused. The [limits](https://developer.brex.com/guides/rate_limits.md) are 1,000 requests per minute, with separate daily caps on transfers, international wires and new cards. APIs cover transactions, statements, expenses with receipts, budgets, payments, team and travel. The [Accounting API](https://developer.brex.com/openapi/accounting_api.md) is still in alpha (1.0.0-alpha), and Brex picks who gets access. An integration uses it to read accounting records and report whether each export to the ledger succeeded or failed.

## Agents

The [Brex MCP](https://developer.brex.com/docs/mcp) is a beta remote server, switched on by an admin. It works with Claude Code, Claude Desktop, ChatGPT, Cursor and Codex, and signs in by OAuth. Most tools only read. A few write: bulk memo updates, receipt uploads, attendees, assigning expenses to a spend limit. Approvals and card management are not available through it. Brex tells users to turn on human confirmation in their AI client and logs MCP activity in the dashboard. Brex does not enforce the approval step itself; it leaves that to the AI client.

## Customers

Capital One cited more than 25,000 companies in over 50 countries, including DoorDash, Robinhood, CrowdStrike, Zoom and Intel; Brex's homepage now says 35,000+. Brex scores 1.8 on Trustpilot from 576 reviews. The complaints are mostly about banking: frozen or closed accounts, unexplained application denials, and rules that changed and dropped long-time customers. Some reviewers praise the support.

## History

Henrique Dubugras and Pedro Franceschi, who had sold Pagar.me in Brazil, founded Brex in January 2017 and went through Y Combinator. The card for startups grew fast: $1.1 billion valuation by October 2018, $12.3 billion by 2021. In June 2022 Brex dropped small businesses that were not venture-backed to focus on larger companies. In March 2023 it took in billions in deposits from Silicon Valley Bank customers the day before SVB failed. In January 2024 it cut about 20% of staff. In August 2025 it got an EU licence. Then it sold to a bank.

## For ownpurse

Brex shows spend management consolidating into banks: the card, the account and the expense software end up under one regulated owner. For an owner, that makes the data more concentrated and the access more conditional. It also shows that a vendor can stop serving a customer segment, as Brex did in 2022. Brex is a good source: the API is documented, the limits are generous, and statements and transactions can be read in bulk. A Brex connector in ownpurse would record card lines, bank lines and receipts with provenance, and keep them if the relationship ends.
