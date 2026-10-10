---
{
 "slug": "make",
 "name": "Make",
 "url": "https://www.make.com",
 "segment": "ipaas-middleware",
 "status": "active",
 "founded": "2015 by its own account (as Integromat, Prague); renamed Make in 2022",
 "hq": "Prague, Czech Republic (offices across Europe, Asia and the US)",
 "backing": "Part of Celonis (process mining, which Make describes as valued at $13B), which acquired Integromat in 2020.",
 "tagline": "A hosted visual automation platform, formerly Integromat, where you build scenarios across 3,000+ apps, now with AI agents and an MCP server.",
 "scope": "Make builds multi-step scenarios on a visual canvas with routers, iterators, error handlers and data stores, priced per credit. It has deep accounting modules (94 for Xero, 93 for QuickBooks) and is popular with bookkeepers and agencies. It keeps execution logs for 7 to 60 days and no record of the books.",
 "domains": [
  "integration"
 ],
 "customers_served": [
  "freelancer",
  "smb",
  "mid-market",
  "enterprise",
  "accountants"
 ],
 "spectrum": {
  "custody": {
   "score": 1,
   "note": "Holds no copy of the books; keeps execution logs 7-60 days by plan, plus any data stores you build."
  },
  "locality": {
   "score": 0,
   "note": "Cloud only; an Enterprise on-prem agent reaches local networks but the platform runs in Make's cloud."
  },
  "history": {
   "score": 1,
   "note": "Execution logs kept 7 days (Free), 30 days (paid) or 60 days (Enterprise); nothing as-of."
  },
  "openness": {
   "score": 2,
   "note": "Closed product with a full REST API over scenarios, connections and data stores, and scenarios exportable as JSON blueprints."
  },
  "breadth": {
   "score": 3,
   "note": "Reaches ledgers, banks, payments, documents and commerce through 3,000+ apps, without a unified record."
  },
  "autonomy": {
   "score": 4,
   "note": "Scenarios write to Xero and QuickBooks unattended on a schedule or webhook."
  },
  "agent_readiness": {
   "score": 3,
   "note": "Hosted MCP server with OAuth and scoped tokens, scenarios as tools, management tools and Make Skills; no CLI."
  },
  "integration_ease": {
   "score": 3,
   "note": "Self-serve and well documented, but steeper to learn than Zapier; reviewers cite the learning curve."
  },
  "market": {
   "score": 2,
   "note": "Small businesses, agencies and bookkeepers, with an Enterprise tier for larger companies."
  }
 },
 "api": {
  "public": true,
  "style": "REST API (scenarios, connections, webhooks, data stores, teams, organizations); custom apps SDK; hosted MCP server over Streamable HTTP",
  "auth": "API tokens with selectable scopes; MCP via OAuth at mcp.make.com or an MCP token in URL or header",
  "limits": "API: 60 requests a minute (Core), 120 (Pro), 240 (Teams), 1,000 (Enterprise); 429 on excess. MCP timeouts 25-60 s, runs continue up to 40 min. Scenario runs up to 40 min on paid plans.",
  "cost": "Credits: one per module action; routers and error handlers free; Make Code uses 2 credits per second; some AI-provider features use more. Data transfer 5 GB per 10,000 credits.",
  "export": "Scenarios as JSON blueprints; execution logs for the retention window; data stores via the API. No bulk export of the data that passed through.",
  "notes": "MCP management tools need a paid plan; scenario run tools are on all plans."
 },
 "pricing": {
  "model": "Subscription by monthly credits, with a slider per plan",
  "entry": "Free: $0, 1,000 credits a month, 2 active scenarios, 15-minute minimum interval. Core: $9 a month for 10,000 credits.",
  "top": "Pro $16 and Teams $29 a month at 10,000 credits, sliders to 8M+ credits; Enterprise custom with overage protection, SSO, audit logs and an on-prem agent.",
  "notes": "Prices from make.com/en/pricing, read 2026-10-10; annual billing saves 15% or more. Make moved from 'operations' to 'credits' in 2025 [date unverified]."
 },
 "agents": {
  "mcp": "official",
  "ai_features": "Make AI Agents (built on the canvas, with a Reasoning panel showing each decision; next generation announced 11 February 2026), AI Toolkit, Make AI Provider or your own LLM key, Make MCP server, Make Skills for MCP clients, Make Grid (a map of your automations).",
  "agent_writes": "Yes: an MCP client can run scenarios that write to the books, and with management scopes can create and modify scenarios, connections and data stores.",
  "approval": "Make says agents can have manual approvals and stop points on the canvas; MCP access is limited by the scopes chosen and the scenarios exposed. No sealed or recorded approval package.",
  "notes": "Visible reasoning per step is a good pattern; approval is a step you design in, not a gate on every write."
 },
 "integration": {
  "scope": "3,000+ apps; 400+ AI app integrations. Xero: 94 modules (12 triggers, 63 actions, 19 searches). QuickBooks: 93 modules (2 triggers, 74 actions, 17 searches).",
  "notes": "Deeper accounting coverage than most iPaaS: batch payments, contact groups, bank transfers. Polling triggers, so the same event-driven blind spots as Zapier."
 },
 "preference": {
  "scale": "400,000+ organizations in 200+ countries and 350+ employees (make.com, read 2026-10-10).",
  "ratings": "Trustpilot 2.7/5 from 173 reviews (read 2026-10-10). Make's homepage shows Capterra 4.8, G2 4.7, GetApp 4.8, Gartner 4.6 without counts; review sites blocked automated reading, so counts not confirmed.",
  "sentiment": "Praised for power, a generous free tier and lower cost than Zapier for complex flows. Complaints: steep learning curve, scenarios that break after updates, a slow interface for data mapping, support hard to reach, and AI assistants that fail to build or fix scenarios."
 },
 "timeline": [
  {
   "date": "2015",
   "event": "Founded as Integromat in Prague, by Make's own account"
  },
  {
   "date": "2020",
   "event": "Acquired by Celonis"
  },
  {
   "date": "2022",
   "event": "Integromat renamed Make"
  },
  {
   "date": "2025",
   "event": "Pricing moves from operations to credits; Make AI Agents and MCP server [dates unverified]"
  },
  {
   "date": "2026-02-11",
   "event": "Next generation of Make AI Agents announced"
  },
  {
   "date": "2026-10-19",
   "event": "Make Waves '26 in Prague"
  }
 ],
 "relation": "neighbour",
 "relation_note": "Make is the automation layer many bookkeepers and agencies use to push documents, payments and contacts into Xero and QuickBooks. Like Zapier it moves events and forgets them; ownpurse would sit underneath as the record those scenarios never keep.",
 "borrow": "The Reasoning panel that shows each agent decision step by step. Scoped MCP tokens, and choosing exactly which scenarios become tools. Make Skills: plain Markdown files that teach an MCP client how to use the server. Blueprint export as a portable description of a flow.",
 "avoid": "Logs as the only history, with a 30-day window. Per-module credit metering that turns every read into a cost. Approvals as an optional canvas step.",
 "sources": [
  {
   "title": "Make pricing",
   "url": "https://www.make.com/en/pricing"
  },
  {
   "title": "Make homepage",
   "url": "https://www.make.com/en"
  },
  {
   "title": "About Make",
   "url": "https://www.make.com/en/about"
  },
  {
   "title": "Make AI Agents",
   "url": "https://www.make.com/en/ai-agents"
  },
  {
   "title": "Make MCP server docs",
   "url": "https://developers.make.com/mcp-server"
  },
  {
   "title": "Make API rate limiting",
   "url": "https://developers.make.com/api-documentation/getting-started/rate-limiting"
  },
  {
   "title": "Make API authentication tokens",
   "url": "https://developers.make.com/api-documentation/authentication/create-authentication-token"
  },
  {
   "title": "Xero integrations on Make",
   "url": "https://www.make.com/en/integrations/xero"
  },
  {
   "title": "QuickBooks integrations on Make",
   "url": "https://www.make.com/en/integrations/quickbooks"
  },
  {
   "title": "Make on Trustpilot",
   "url": "https://www.trustpilot.com/review/make.com"
  }
 ],
 "unverified": [
  "Exact date and price of the Celonis acquisition of Integromat (2020; price not confirmed)",
  "Exact date of the rename to Make (2022; month not confirmed)",
  "Integromat's original founding year and founders (Make says 2015; other accounts say earlier)",
  "Date of the move from operations to credits",
  "Launch dates of Make AI Agents (first generation) and the Make MCP server",
  "G2, Capterra and GetApp review counts behind the scores Make displays",
  "Annual plan prices (page says 15% or more off)",
  "Make revenue: not disclosed"
 ],
 "researched": "2026-10-10"
}
---

## What it is

Make is a hosted visual automation platform, formerly Integromat, owned by Celonis. You draw a scenario on a canvas: a trigger, then modules that read, transform and write across [3,000+ apps](https://www.make.com/en). It says it serves [400,000+ organizations in 200+ countries with 350+ staff](https://www.make.com/en/about), and describes itself as founded in 2015 and now part of Celonis. It is the usual choice when a Zapier flow gets too complex or too expensive, and it is popular with bookkeepers and agencies.

## How it works

Scenarios run on a schedule (every minute on paid plans) or from webhooks, with routers, iterators, error handlers and data stores. Its accounting modules are deep: [94 for Xero](https://www.make.com/en/integrations/xero) and [93 for QuickBooks](https://www.make.com/en/integrations/quickbooks), including batch payments and bank transfers. Make keeps no copy of your books. It keeps execution logs for 7 days on Free, 30 on paid plans and 60 on Enterprise ([pricing](https://www.make.com/en/pricing)), then they go.

## Data and access

On [the pricing page](https://www.make.com/en/pricing) (read 2026-10-10), Free gives 1,000 credits a month and 2 active scenarios; Core is $9 a month, Pro $16, Teams $29, each from 10,000 credits with a slider beyond 8 million; Enterprise is custom. A credit is one module action, so a read costs the same as a write; routers and error handlers are free, and code runs at 2 credits a second. The [REST API](https://developers.make.com/api-documentation/getting-started/rate-limiting) uses scoped tokens and allows 60 to 1,000 requests a minute by plan. Scenarios export as JSON blueprints, which makes a flow portable; the data that passed through does not.

## Agents

Make runs a hosted [MCP server](https://developers.make.com/mcp-server) over Streamable HTTP, with OAuth or an MCP token and scopes that decide which tools appear. Scenarios become tools on every plan; management tools (create and edit scenarios, connections, webhooks, data stores) need a paid plan. Make Skills are Markdown files that teach a client how to build scenarios. [Make AI Agents](https://www.make.com/en/ai-agents), with a next generation announced on 11 February 2026, are built on the canvas, show every decision in a Reasoning panel, and can have manual approvals and stop points. That is a good pattern. But approval is something you design in; the default write path is direct.

## Customers

On [Trustpilot](https://www.trustpilot.com/review/make.com) Make scores 2.7/5 from 173 reviews (read 2026-10-10), split between praise for the free tier, power and value, and complaints about the learning curve, scenarios that break after updates, a slow interface for mapping data and support that is hard to reach. Make's own site shows Capterra 4.8 and G2 4.7; the counts could not be confirmed. Small firms and bookkeepers use it to route receipts, payments and contacts into a ledger; Enterprise adds SSO, audit logs and an on-prem agent for reaching systems such as SAP behind a firewall.

## History

Integromat came out of Prague as the more technical rival to Zapier. Celonis bought it in 2020, adding automation to process mining, and renamed it Make in 2022. Pricing moved from operations to credits, and since 2025 the product has been repositioned as an AI automation platform with agents, MCP and Make Grid. Like Zapier, it solved connectors by building them centrally and charging per step; it never tried to hold the data.

## For ownpurse

Make is where a lot of small-firm accounting automation already lives, built by bookkeepers who know exactly which Xero fields matter. It writes into the books unattended and keeps 30 days of logs. ownpurse is the missing record beneath it: every fact, every version, on the owner's machine, and writes only by approved package. A Make scenario could feed ownpurse's drawer or consume an approved package. Borrow the Reasoning panel (show the agent's steps in order), scoped MCP tokens with an explicit list of exposed tools, and Skills files as agent docs. Avoid credit-per-read metering and logs as the only history.
