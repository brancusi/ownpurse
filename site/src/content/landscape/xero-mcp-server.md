---
{
 "slug": "xero-mcp-server",
 "name": "Xero MCP Server",
 "url": "https://github.com/XeroAPI/xero-mcp-server",
 "segment": "agent-native",
 "status": "active",
 "founded": "2025 (repository created 17 March 2025; first npm release 21 March 2025)",
 "hq": "Wellington, New Zealand (Xero Limited)",
 "backing": "Built and published by Xero Limited, listed on the ASX (XRO). Xero says it serves 5 million customers worldwide (media release, 9 July 2026). Not separately funded.",
 "tagline": "Xero's official open-source MCP server, which runs on your machine and lets an AI client read and write one Xero organisation directly.",
 "scope": "A Node.js stdio MCP server (MIT) that wraps the Xero accounting and payroll APIs as tools: lists and reports (accounts, contacts, invoices, bank transactions, manual journals, trial balance, P&L, balance sheet, aged receivables and payables, NZ and UK payroll) plus create and update tools for invoices, contacts, payments, bank transactions, manual journals, items and tracking. It keeps no data of its own, has no approval step and no log, and cannot reconcile bank statement lines or reach the Journals feed.",
 "domains": [
  "ledger",
  "invoicing",
  "payroll",
  "bank",
  "reporting"
 ],
 "customers_served": [
  "smb",
  "accountants",
  "developers"
 ],
 "spectrum": {
  "custody": {
   "score": 0,
   "note": "Keeps nothing; Xero holds the only copy and every answer is fetched live."
  },
  "locality": {
   "score": 2,
   "note": "The server process runs on your machine, but every read and write is a call to Xero's cloud."
  },
  "history": {
   "score": 0,
   "note": "Passes through Xero's current state; no versions, no log of what the agent changed."
  },
  "openness": {
   "score": 3,
   "note": "MIT-licensed client over a closed, metered ledger API."
  },
  "breadth": {
   "score": 2,
   "note": "Accounting, invoicing, bank transactions and NZ/UK payroll, all in one provider."
  },
  "autonomy": {
   "score": 3,
   "note": "An agent can create invoices, payments and journals directly; only the MCP client's tool-call prompt stands in the way."
  },
  "agent_readiness": {
   "score": 4,
   "note": "Official MCP server, plus Xero's CLI, agent toolkit and prompt library."
  },
  "integration_ease": {
   "score": 3,
   "note": "One npx line and a Custom Connection; Custom Connections are paid and not offered in every region (Canada, per issue, April 2026)."
  },
  "market": {
   "score": 2,
   "note": "Xero's small-business and accountant base."
  }
 },
 "api": {
  "public": true,
  "style": "MCP over stdio, wrapping Xero's REST APIs (Accounting, Payroll NZ/UK) through the xero-node SDK",
  "auth": "Custom Connection (client credentials, one organisation) or a bearer token supplied by the client (e.g. PKCE); V1 scopes for Custom Connections made before 29 April 2026, granular V2 scopes after",
  "limits": "Xero API limits apply: 1,000 calls a day per organisation on the free Starter tier, 5,000 on paid tiers (developer pricing, from 2 March 2026); an open issue (June 2026) reports a 10-record page cap and no date filter on bulk reads",
  "cost": "The server is free. Xero developer tiers from 2 March 2026 (AUD ex tax): Starter free (5 connections), Core $35/month (50 connections, 10 GB egress), Plus $245, Advanced $1,445, Enterprise on application; egress overage $2.40/GB. Custom Connections carry their own subscription",
  "export": "Whatever the Xero API returns; no bulk export of its own. The Journals endpoint needs Advanced tier plus a security assessment",
  "notes": "Xero's terms (from 4 Dec 2025 for new developers, 2 March 2026 for existing) forbid training AI on API data and bots that simulate users."
 },
 "pricing": {
  "model": "Free open-source software; costs come from the Xero subscription, the API tier and the Custom Connection",
  "entry": "Free (Xero Starter developer tier, up to 5 connections)",
  "top": "Xero developer Enterprise tier, priced on application",
  "notes": "Xero's hosted Claude integration (live 12-13 May 2026) is a separate product bundled with a Xero subscription."
 },
 "agents": {
  "mcp": "official",
  "ai_features": "Exposes about 50 tools; Xero's wider AI line includes JAX (its in-product agent), the Claude integration (May 2026), XeroForce agent builder (alpha, May 2026), a CLI and an agent toolkit for OpenAI Agents SDK, LangChain and Google ADK",
  "agent_writes": "Yes, direct: create invoices, quotes, credit notes, payments, bank transactions, manual journals, contacts, items, tracking categories and payroll timesheets; update most of these (invoices, quotes and credit notes only in draft); delete payroll timesheets",
  "approval": "None in the server. The only gate is whatever the MCP client shows before a tool call; nothing is staged, sealed or recorded",
  "notes": "The README recommends the Demo Company for testing. One server process holds one connection; multi-tenant use is an open request."
 },
 "integration": {
  "scope": "One Xero organisation per Custom Connection; any MCP client (Claude Desktop, Cursor and others); Xero's agent toolkit shows OpenAI Agents SDK, LangChain and Google ADK",
  "notes": "Payroll tools work only for NZ and UK organisations. Community issues report auth trouble with Claude Desktop, missing repeating invoices and attachments, and 4-decimal unit amounts."
 },
 "preference": {
  "scale": "374 stars and 203 forks on GitHub (10 Oct 2026); npm package @xeroapi/xero-mcp-server at 0.0.17 (26 May 2026). Xero itself: 5 million customers (9 July 2026)",
  "ratings": "No review-site listing for the server. Xero on Trustpilot: 4.1 from 11,430 reviews (Oct 2026)",
  "sentiment": "Developers like that it is official, quick to start and covers the common objects. Complaints centre on auth setup, missing tools for full bookkeeping, page caps on bulk reads, and maintenance: a 23 Sept 2026 issue notes no merge to main since 5 June 2026 and 77 open pull requests."
 },
 "timeline": [
  {
   "date": "2025-03-17",
   "event": "Repository created under XeroAPI"
  },
  {
   "date": "2025-03-21",
   "event": "First npm release, 0.0.1"
  },
  {
   "date": "2025-08-08",
   "event": "Xero agent toolkit published, built on the MCP server"
  },
  {
   "date": "2025-09-03",
   "event": "Xero announces new JAX features"
  },
  {
   "date": "2025-12-04",
   "event": "New developer terms for new apps: no AI training on API data, no simulated-user bots"
  },
  {
   "date": "2026-03-02",
   "event": "Tiered API pricing with egress fees applies to existing apps"
  },
  {
   "date": "2026-03-23",
   "event": "Xero command-line tool repository created (alpha)"
  },
  {
   "date": "2026-03-27",
   "event": "Xero and Anthropic announce a multi-year partnership"
  },
  {
   "date": "2026-04-29",
   "event": "Server supports granular OAuth scopes for new Custom Connections"
  },
  {
   "date": "2026-05-12",
   "event": "Xero's Claude integration goes live for subscribers"
  },
  {
   "date": "2026-05-14",
   "event": "XeroForce agent builder announced, alpha"
  },
  {
   "date": "2026-05-26",
   "event": "0.0.17 published, fixing a token leak in error output"
  },
  {
   "date": "2026-06-05",
   "event": "Last commit on main as of October 2026"
  },
  {
   "date": "2026-09-23",
   "event": "Community issue asks about maintenance; 77 open pull requests"
  }
 ],
 "relation": "neighbour",
 "relation_note": "The incumbent's own answer to agents: a thin, live pipe from an AI client into Xero with direct writes. ownpurse sits under the same agent but keeps an owner-held record, reads without spending API calls, and sends writes only as approved packages.",
 "borrow": "The tool catalogue as a map of what agents actually ask for; draft-only updates for invoices, quotes and credit notes; one-line install; the Demo Company as the default test target; Xero's toolkit and prompt library as distribution.",
 "avoid": "Writes with no staging, approval or record; one connection per process; reading everything live against a metered, page-capped API; leaving the integrity of the books to the MCP client's confirm dialog.",
 "sources": [
  {
   "title": "XeroAPI/xero-mcp-server on GitHub",
   "url": "https://github.com/XeroAPI/xero-mcp-server"
  },
  {
   "title": "@xeroapi/xero-mcp-server on npm",
   "url": "https://www.npmjs.com/package/@xeroapi/xero-mcp-server"
  },
  {
   "title": "Xero developer pricing",
   "url": "https://developer.xero.com/pricing"
  },
  {
   "title": "Xero developer AI toolkit",
   "url": "https://developer.xero.com/ai"
  },
  {
   "title": "XeroAPI/xero-agent-toolkit",
   "url": "https://github.com/XeroAPI/xero-agent-toolkit"
  },
  {
   "title": "XeroAPI/xero-command-line",
   "url": "https://github.com/XeroAPI/xero-command-line"
  },
  {
   "title": "Xero and Anthropic collaborate (27 March 2026)",
   "url": "https://www.xero.com/us/media-releases/xero-and-anthropic-collaborate/"
  },
  {
   "title": "Xero delivers Claude integration (May 2026)",
   "url": "https://www.xero.com/us/media-releases/xero-delivers-claude-integration-to-advance-ai-powered-financial-intelligence/"
  },
  {
   "title": "Xero introduces XeroForce (14 May 2026)",
   "url": "https://www.xero.com/us/media-releases/xero-introduces-xeroforce/"
  },
  {
   "title": "Xero at Xerocon London, 5 million customers (9 July 2026)",
   "url": "https://www.xero.com/us/media-releases/xero-announces-new-ai-innovations-xerocon-london/"
  },
  {
   "title": "Xero on Trustpilot",
   "url": "https://www.trustpilot.com/review/www.xero.com"
  },
  {
   "title": "xero-mcp-server issues",
   "url": "https://github.com/XeroAPI/xero-mcp-server/issues"
  }
 ],
 "unverified": [
  "The price of a Xero Custom Connection subscription in 2026 and the regions it is offered in (Canada absence is from a community issue only)",
  "Whether Xero's hosted Claude integration uses this server or a separate remote MCP endpoint",
  "Which developer tiers include the AI toolkit",
  "G2 and Capterra ratings for Xero (review sites refused the fetch)",
  "Whether Xero intends to resume maintenance; no maintainer reply was found"
 ],
 "researched": "2026-10-10"
}
---

## What it is

The [Xero MCP Server](https://github.com/XeroAPI/xero-mcp-server) is Xero's own Model Context Protocol server, published under the MIT licence in March 2025. You run it on your machine (`npx -y @xeroapi/xero-mcp-server@latest`), point an MCP client such as Claude Desktop at it, and the agent gets about fifty tools over one Xero organisation. It is part of a wider set Xero now calls its [AI toolkit](https://developer.xero.com/ai): the server, an [agent toolkit](https://github.com/XeroAPI/xero-agent-toolkit) for OpenAI Agents SDK, LangChain and Google ADK, a prompt library and, since March 2026, an alpha [command-line tool](https://github.com/XeroAPI/xero-command-line).

## How it works

The server is a stdio process built on the xero-node SDK. It authenticates either with a Custom Connection (client id and secret for one organisation) or with a bearer token your client obtains, and turns each tool call into a live Xero API call. Read tools cover accounts, contacts, invoices, quotes, credit notes, payments, bank transactions, manual journals, items, tax rates, tracking categories, the trial balance, P&L, balance sheet, aged receivables and payables, and NZ and UK payroll. Write tools create invoices, quotes, credit notes, payments, bank transactions, manual journals, contacts, items and tracking options, and update most of them; invoices, quotes and credit notes can be changed only while in draft. The one delete is a payroll timesheet.

It keeps nothing. There is no cache, no history and no record of what the agent changed, and there is no approval step: the only check is whatever your MCP client shows before it runs a tool.

## Data and access

The server is free, but every call counts against Xero's API. Since 2 March 2026 Xero's [developer pricing](https://developer.xero.com/pricing) has five tiers: Starter is free with 5 connections and 1,000 calls a day per organisation; Core is AUD 35 a month with 10 GB of egress; Plus AUD 245; Advanced AUD 1,445; Enterprise on application, with overage at AUD 2.40 per GB. The Journals feed needs Advanced and a security assessment. The same terms forbid training AI on API data and bots that simulate users. Custom Connections are a paid add-on and, per a [community issue](https://github.com/XeroAPI/xero-mcp-server/issues), not offered in Canada. Another issue reports a 10-record page cap and no date filter on bulk reads, which matters if an agent tries to look at a year of activity.

## Agents

This is the most agent-ready thing Xero ships: official, documented, and backed by examples for the main agent frameworks. Xero's hosted [Claude integration](https://www.xero.com/us/media-releases/xero-delivers-claude-integration-to-advance-ai-powered-financial-intelligence/) (live 12-13 May 2026) is a separate, read-oriented product, and [XeroForce](https://www.xero.com/us/media-releases/xero-introduces-xeroforce/), an in-product agent builder, is in invite-only alpha. None of them describes an approval gate a person controls outside Xero.

## Customers

The audience is Xero's: small businesses, their accountants and the developers who build for them. The repository had 374 stars and 203 forks on 10 October 2026. Developers value that it is official and quick to start. The complaints are auth setup, missing tools (repeating invoices, attachments, reconciliation) and maintenance: a [23 September 2026 issue](https://github.com/XeroAPI/xero-mcp-server/issues) notes no merge to main since 5 June 2026 and 77 open pull requests.

## History

The server appeared on npm on 21 March 2025, months after MCP itself, and has stayed at 0.0.x. In the same period Xero tightened API access (new terms from December 2025, egress fees from March 2026) and partnered with Anthropic (27 March 2026). The pattern is clear: agents are welcome inside Xero's walls, on Xero's meter.

## For ownpurse

This is the default an ownpurse user will compare against: free, official, and able to post a journal in one sentence. What it does not do is exactly ownpurse's ground. It holds no copy, so every question costs API calls and egress; it keeps no history, so a changed or deleted invoice is gone; and it writes straight into the books with no staged, sealed or recorded package. ownpurse can coexist with it, and should borrow its tool catalogue as a list of what agents ask for, but its argument is that reads should be local and free and writes should be reviewed and logged.
