---
{
 "slug": "quickbooks-online-mcp-server",
 "name": "QuickBooks Online MCP Server",
 "url": "https://github.com/intuit/quickbooks-online-mcp-server",
 "segment": "agent-native",
 "status": "active",
 "founded": "2025 (first commit 6 October 2025)",
 "hq": "Mountain View, California, US (Intuit)",
 "backing": "Published by Intuit Inc., listed on NASDAQ (INTU). Intuit says about 100 million customers use its products (press release, 12 August 2026). Not separately funded.",
 "tagline": "Intuit's official open-source MCP server, run locally, that gives an AI client create, read, update and delete over a QuickBooks Online company.",
 "scope": "A TypeScript stdio MCP server with 145 tools over 29 QuickBooks Online entities and 11 reports (balance sheet, P&L, cash flow, trial balance, general ledger, aged receivables and payables and others). Writes, updates and deletes are on by default and can be switched off by category with environment flags. It keeps no data, no history and no approval queue, and cannot reach QuickBooks' bank-feed review queue.",
 "domains": [
  "ledger",
  "invoicing",
  "payments",
  "reporting",
  "bank"
 ],
 "customers_served": [
  "smb",
  "accountants",
  "developers"
 ],
 "spectrum": {
  "custody": {
   "score": 0,
   "note": "Keeps nothing; Intuit holds the only copy and every answer is fetched live."
  },
  "locality": {
   "score": 2,
   "note": "The process runs on your machine; the books and every call are in Intuit's cloud."
  },
  "history": {
   "score": 0,
   "note": "Current state only; deletes are real deletes and nothing records what the agent did."
  },
  "openness": {
   "score": 3,
   "note": "Apache-2.0 client (the README says MIT) over a closed, metered API."
  },
  "breadth": {
   "score": 2,
   "note": "Wide QuickBooks coverage (sales, purchases, journals, time, attachments), one provider."
  },
  "autonomy": {
   "score": 3,
   "note": "Create, update and delete are enabled by default with no gate in the server."
  },
  "agent_readiness": {
   "score": 3,
   "note": "Official MCP with broad tools and tests, but no CLI, no package on npm and painful production auth."
  },
  "integration_ease": {
   "score": 2,
   "note": "Production OAuth rejects localhost, so the first handshake needs a public HTTPS callback (ngrok); a 100-day refresh window."
  },
  "market": {
   "score": 2,
   "note": "QuickBooks' small-business and accountant base."
  }
 },
 "api": {
  "public": true,
  "style": "MCP over stdio, wrapping the QuickBooks Online Accounting REST API through node-quickbooks",
  "auth": "OAuth 2.0 app on the Intuit Developer Portal; client id, secret, refresh token and realm id in the environment; refresh tokens rotate and are persisted by the server",
  "limits": "QuickBooks API limits and App Partner Program metering apply; the server adds none of its own",
  "cost": "The server is free. Intuit App Partner Program (US, from 28 July 2025; full variable fees from 1 Nov 2025): Core calls (writes) free and unlimited; CorePlus calls (reads) metered: Builder free with 500,000 a month then blocked, Silver $300/month with 1 million and $3.50 per 1,000 over, Gold $1,700, Platinum $4,500 with up to 75 million and $0.25 per 1,000 over",
  "export": "Whatever the API returns, including the general ledger report and invoice PDFs; no bulk export of its own",
  "notes": "Intuit meters reads and leaves writes free, so an agent that reads a lot costs more than one that posts a lot."
 },
 "pricing": {
  "model": "Free open-source software; costs come from the QuickBooks subscription and the App Partner Program tier",
  "entry": "Free (Builder tier, 500,000 metered reads a month)",
  "top": "Platinum, $4,500 a month",
  "notes": "Tier figures are from Apideck's summary of the program (Sept 2025); Intuit's own pages did not render for this review."
 },
 "agents": {
  "mcp": "official",
  "ai_features": "145 tools across 29 entities and 11 reports, an invoice-PDF tool and preferences; separately, Intuit sells in-product agents (Intuit Intelligence Chat, beta, which acts only after the user confirms; Books Upkeep in QuickBooks Online Advanced, which auto-resolves high-confidence transactions under rules)",
  "agent_writes": "Yes, direct: full create, read, update and delete for 20 entities including invoices, bills, journal entries, payments, deposits, transfers and attachments; enabled by default",
  "approval": "None in the server. Environment flags (QUICKBOOKS_DISABLE_WRITE, _UPDATE, _DELETE) remove whole categories; there is no per-action review, staging or log",
  "notes": "Community work in 2026 added input validation and asked for idempotency guards on create retries; a July 2026 issue reported 15 vulnerable production dependencies via node-quickbooks, after which CI began blocking high and critical ones (Sept 2026)."
 },
 "integration": {
  "scope": "One QuickBooks Online company (realm) per configuration; any MCP client",
  "notes": "Sandbox accepts a localhost redirect; production needs a public HTTPS callback for the first handshake. Staged bank-feed and receipt-review entities are not exposed (open feature request)."
 },
 "preference": {
  "scale": "417 stars and 259 forks on GitHub (10 Oct 2026); not published to npm. Intuit: about 100 million customers across its products (12 Aug 2026)",
  "ratings": "No review-site listing for the server. QuickBooks on Trustpilot: 3.7 from 16,890 reviews (Oct 2026)",
  "sentiment": "Users credit the breadth of tools and the test suite. The loudest complaints are about setup: production OAuth, running it locally at all, and early docs; one 2026 issue is titled 'DO NO USE THIS REPO'. Others asked in January 2026 whether it is maintained; merges resumed in April and September 2026."
 },
 "timeline": [
  {
   "date": "2025-07-28",
   "event": "Intuit App Partner Program starts for US partners: reads metered, writes free"
  },
  {
   "date": "2025-10-06",
   "event": "First commit of the MCP server"
  },
  {
   "date": "2025-10",
   "event": "First issue: the lockfile pointed at Intuit's private npm registry; fixed 14 Oct"
  },
  {
   "date": "2025-11-01",
   "event": "Full variable API fees begin"
  },
  {
   "date": "2026-01-13",
   "event": "Comprehensive coverage: 143 tools and full test coverage"
  },
  {
   "date": "2026-04",
   "event": "Community fixes and a read-only budgets tool merged"
  },
  {
   "date": "2026-07-28",
   "event": "Issue reports 15 vulnerable production dependencies (3 critical)"
  },
  {
   "date": "2026-08-12",
   "event": "Intuit announces Intelligence Chat (beta), which acts only after the user confirms"
  },
  {
   "date": "2026-09-15",
   "event": "Latest merges; CI blocks high and critical dependency regressions"
  }
 ],
 "relation": "neighbour",
 "relation_note": "The second incumbent's agent pipe, wider than Xero's and more permissive: deletes on by default. ownpurse would sit beneath the same agent as the owner-held record and the approval step this server lacks, and a QuickBooks connector would face the same metered reads.",
 "borrow": "Per-category kill switches (disable write, update, delete) as a coarse first gate; the entity-by-operation coverage table as a spec for a QuickBooks connector; the community's idempotency guard keyed on DocNumber for create retries.",
 "avoid": "Writes and deletes on by default; reading everything live against metered CorePlus calls; an install that needs a public tunnel; deletes with no record of what was there.",
 "sources": [
  {
   "title": "intuit/quickbooks-online-mcp-server on GitHub",
   "url": "https://github.com/intuit/quickbooks-online-mcp-server"
  },
  {
   "title": "quickbooks-online-mcp-server issues",
   "url": "https://github.com/intuit/quickbooks-online-mcp-server/issues"
  },
  {
   "title": "Apideck: QuickBooks API pricing and the Intuit App Partner Program",
   "url": "https://www.apideck.com/blog/quickbooks-api-pricing-and-the-intuit-app-partner-program"
  },
  {
   "title": "Intuit advances its mid-market platform with conversational AI (12 Aug 2026)",
   "url": "https://investors.intuit.com/news-events/press-releases/detail/1319/intuit-advances-its-mid-market-platform-with-conversational-ai-enterprise-scale-and-deep-industry-workflows-for-cfos-and-accounting-firms"
  },
  {
   "title": "QuickBooks on Trustpilot",
   "url": "https://www.trustpilot.com/review/quickbooks.intuit.com"
  },
  {
   "title": "Intuit Developer Portal",
   "url": "https://developer.intuit.com/"
  }
 ],
 "unverified": [
  "That Intuit labelled the October 2025 release a 'preview'; the current README carries no such label",
  "Gold tier included reads and overage rate, and non-US App Partner Program dates",
  "QuickBooks API rate limits per company (Intuit's docs did not render for this review)",
  "Whether Intuit offers a hosted or remote QuickBooks MCP endpoint (for example in ChatGPT or Claude)",
  "QuickBooks Online subscriber count specifically, as opposed to Intuit's 100 million customers",
  "G2 and Capterra ratings for QuickBooks Online"
 ],
 "researched": "2026-10-10"
}
---

## What it is

The [QuickBooks Online MCP Server](https://github.com/intuit/quickbooks-online-mcp-server) is Intuit's open-source MCP server, first committed on 6 October 2025. It runs locally as a stdio subprocess and gives an AI client 145 tools over 29 QuickBooks Online entities and 11 reports. The repository is licensed Apache-2.0 (its README still says MIT).

## How it works

You register an OAuth app on the Intuit Developer Portal, complete a one-time browser handshake, and put the client id, secret, refresh token and company (realm) id in the environment. The server calls the QuickBooks API through node-quickbooks and persists rotated refresh tokens. Twenty entities have full create, read, update and delete, among them invoices, bills, journal entries, payments, deposits, transfers, purchases and attachments; accounts, classes, departments and terms lack delete; tax codes and rates are read-only. Reports include the balance sheet, P&L, cash flow, trial balance and general ledger.

Writes, updates and deletes are on by default. Three flags (`QUICKBOOKS_DISABLE_WRITE`, `_UPDATE`, `_DELETE`) remove those categories; setting all three makes it read-only. There is no per-action approval, no staging and no log.

## Data and access

The server is free; the API is not. Under Intuit's App Partner Program (US partners from 28 July 2025, full fees from 1 November 2025), writes are free and unlimited while reads are metered. Per [Apideck's summary](https://www.apideck.com/blog/quickbooks-api-pricing-and-the-intuit-app-partner-program), Builder is free with 500,000 reads a month and then blocked, Silver is $300 a month with 1 million and $3.50 per 1,000 over, Gold $1,700, and Platinum $4,500 with up to 75 million at $0.25 per 1,000. An agent that asks many questions costs more than one that posts many entries.

Setup is the main barrier. Production rejects localhost redirect URIs, so the first handshake needs a public HTTPS callback, usually an ngrok tunnel, and the README notes a 100-day refresh window. The package is not on npm; you build it from source.

## Agents

This is the most permissive of the incumbents' servers: an agent can delete a bill or a journal entry by default. Intuit's own in-product agents take a more careful line. Its [12 August 2026 release](https://investors.intuit.com/news-events/press-releases/detail/1319/intuit-advances-its-mid-market-platform-with-conversational-ai-enterprise-scale-and-deep-industry-workflows-for-cfos-and-accounting-firms) says Intuit Intelligence Chat acts only after the user confirms, and Books Upkeep labels every automated action. That confirmation lives inside Intuit's product, not in anything the owner holds.

## Customers

The audience is QuickBooks' small businesses, accountants and developers; Intuit reports about 100 million customers across its products. The repository had 417 stars and 259 forks on 10 October 2026. Users credit its breadth and test suite. The [issues](https://github.com/intuit/quickbooks-online-mcp-server/issues) are dominated by setup pain (production OAuth, how to run it locally) and early maintenance doubts, plus a July 2026 report of 15 vulnerable dependencies. Merges resumed in April and September 2026.

## History

Intuit metered reads in July 2025 and published this server three months later. The first issue was that its lockfile pointed at Intuit's private registry. By January 2026 it covered most of the API; since then the work has been community fixes for validation, error output and idempotency.

## For ownpurse

A QuickBooks connector for ownpurse meets the same economics: reads cost money and writes do not. That argues for ownpurse's design of reading once into a local record and answering questions from it for free. The server's per-category flags are a fair first gate, but ownpurse's position is finer: no write reaches QuickBooks unless a person approved that specific sealed package, and a delete is recorded, not lost. The entity coverage table is a useful spec for what a connector must read.
