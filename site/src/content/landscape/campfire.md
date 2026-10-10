---
{
 "slug": "campfire",
 "name": "Campfire",
 "url": "https://campfire.ai",
 "segment": "ai-ledger",
 "status": "active",
 "founded": "2023 (Y Combinator; per ERP Scorecard, not confirmed on Campfire's own site)",
 "hq": "San Francisco, California, US; London office opened 2026",
 "backing": "More than $100M raised: $35M Series A led by Accel (reported July 2025) and $65M Series B co-led by Accel and Ribbit (15 October 2025), with Foundation Capital and Y Combinator; private, venture-backed (Campfire Software, Inc.)",
 "tagline": "Campfire is a cloud ERP for venture-backed and mid-market finance teams, with its own accounting model and an agent called Ember that works inside the ledger.",
 "scope": "Campfire replaces the general ledger and the tools around it: multi-entity core accounting, revenue recognition (ASC 606), close management, reporting and forecasting, with about 100 native integrations and bank feeds through Plaid. Its agent, Ember, categorises, reconciles, books accruals and drafts invoices inside Campfire. It does not cover inventory, manufacturing, order management or project accounting, and it does not sit beside another ledger: you move your books in.",
 "domains": [
  "ledger",
  "bank",
  "invoicing",
  "close",
  "reporting",
  "integration"
 ],
 "customers_served": [
  "smb",
  "mid-market",
  "accountants"
 ],
 "spectrum": {
  "custody": {
   "score": 1,
   "note": "Campfire's cloud holds the authoritative books; a full CRUD REST API lets you pull a copy, but no bulk export is documented."
  },
  "locality": {
   "score": 0,
   "note": "Cloud only; no self-hosted or local option."
  },
  "history": {
   "score": 2,
   "note": "Every Ember action is timestamped, attributed and kept in an audit trail; no documented append-only guarantee or as-of queries."
  },
  "openness": {
   "score": 2,
   "note": "Closed source, but a public REST API (about 396 doc pages, OpenAPI spec) with self-serve keys; bulk export not documented."
  },
  "breadth": {
   "score": 3,
   "note": "GL, AP, AR, revenue, close, reporting and bank feeds in one system, plus CRM, billing and spend data via connectors; no documents, payroll or personal finance."
  },
  "autonomy": {
   "score": 3,
   "note": "Ember Agents run continuously and post entries, gated by confidence thresholds that teams widen over time."
  },
  "agent_readiness": {
   "score": 3,
   "note": "Official MCP server (12 mostly read tools), an MCP store for Ember, OAuth apps with Read/Draft/Write/Approve scopes; no CLI."
  },
  "integration_ease": {
   "score": 3,
   "note": "Self-serve API keys and good docs, but OAuth apps must be enabled by support and published apps need Campfire approval; rate limits undocumented."
  },
  "market": {
   "score": 3,
   "note": "Software and fintech companies of roughly $5M to $250M revenue, post-seed to early enterprise; some SAP migrations."
  }
 },
 "api": {
  "public": true,
  "style": "REST, JSON; base https://api.meetcampfire.com; OpenAPI spec and llms.txt published; webhooks",
  "auth": "API key in 'Authorization: Token' header, created per API user under Settings > API Keys, with roles (admin, clerk can post draft entries, view only = GET only); OAuth 2.0 apps with scopes of the form resource:tier (Read, Draft, Write, Approve)",
  "limits": "Rate limits not documented as of 2026-10-10",
  "cost": "No separate API fee found; needs an active Campfire account",
  "export": "No dedicated bulk export documented; list endpoints across cash, core accounting, AP, AR, revenue and financial statements, plus file download URLs and PDFs",
  "notes": "Docs list about 396 pages: Revenue Recognition (89), Core Accounting (82), Company Objects (67), AP (41), AR (36). Apideck lists Campfire as a beta connector in its unified accounting API."
 },
 "pricing": {
  "model": "Quote-based annual SaaS subscription",
  "entry": "Not published; campfire.ai/pricing returned 404 on 2026-10-10. ERP Scorecard estimates $20K to $40K a year for software (low confidence, peer-anchored)",
  "top": "Not published",
  "notes": "Implementation priced separately per ERP Scorecard (estimated mid-four to low-five figures). Site offers a savings calculator and demos only."
 },
 "agents": {
  "mcp": "official",
  "ai_features": "Proprietary 'Large Accounting Model' (LAM, announced October 2025, vendor-claimed 95% accuracy on reconciliations and variance detection); Ember assistant for questions; Ember Agents (beta, March 2026) for transaction matching, AP/AR, anomaly detection, accruals, flux analysis and close prep; MCP store (May 2026) letting Ember reach Ramp, HubSpot, Salesforce, Stripe and any hosted MCP server",
  "agent_writes": "Ember writes inside Campfire: posts accrual journal entries, reconciles Stripe payments, drafts invoices. The outward MCP server at api.meetcampfire.com/mcp exposes 12 tools (chart of accounts, transactions, entities, vendors, reports, budgets, contracts, aging) described as retrieve, search and generate; docs do not say whether any write.",
  "approval": "Confidence thresholds decide what Ember does on its own versus flags; users can review actions before they post; invoices are drafted by Ember and approved by a person. API roles and OAuth scope tiers separate Draft, Write and Approve.",
  "notes": "MCP docs warn that the API token gives full access to your Campfire data. Campfire says data is not sent to an external AI for processing."
 },
 "integration": {
  "scope": "About 100 native integrations (Salesforce, HubSpot, Stripe, Ramp, Brex, Rho, Rippling, DOSS, Cin7 and others); bank feeds via Plaid; any hosted MCP server inside Ember",
  "notes": "Migration tooling advertises a '15-minute journal migration' (vendor claim, partial scope). Integrations are inbound to Campfire's ledger."
 },
 "preference": {
  "scale": "Vendor-claimed 10x year-to-date revenue growth (October 2025); about 40 employees as of late 2025 (ERP Scorecard); named customers include Replit, PostHog, Decagon, Heidi Health, Klarity, CloudZero, Advisor360, Framer, Warp, TwelveLabs, and LimaOne (an MFA Financial company)",
  "ratings": "Roughly two dozen G2 reviews as of mid-2026 per ERP Scorecard; average score not confirmed",
  "sentiment": "Praise (via ERP Scorecard's review digest): modern interface, value of the modules, automatic FX gain and loss booking, responsive support. Complaints: fits tech companies best, features still being built, a learning curve, opaque pricing."
 },
 "timeline": [
  {
   "date": "2023",
   "event": "Founded by John Glasgow (ex-Invoice2go) and Fernando San Martin (ex-Trovata), through Y Combinator (per ERP Scorecard)"
  },
  {
   "date": "About October 2024",
   "event": "Public launch, roughly nine months before the Series A (per Cyber News Centre)"
  },
  {
   "date": "July 2025",
   "event": "$35M Series A led by Accel"
  },
  {
   "date": "2025-10-15",
   "event": "$65M Series B co-led by Accel and Ribbit; total over $100M; announces its Large Accounting Model"
  },
  {
   "date": "2026-03-12",
   "event": "Ember Agents launched in beta for existing customers"
  },
  {
   "date": "2026-05-14",
   "event": "Connectors and MCP store for Ember (Ramp, HubSpot, Salesforce, Stripe, any hosted MCP server)"
  },
  {
   "date": "2026",
   "event": "London office, first outside the US (per ERP Scorecard)"
  }
 ],
 "relation": "neighbour",
 "relation_note": "Campfire asks a company to move its books into Campfire's cloud and lets its own agent run them; ownpurse leaves the books where they are and keeps the owner's copy. Its customers are venture-backed mid-market finance teams, not ownpurse's people and small businesses. Its API is complete enough that Campfire could become an ownpurse source.",
 "borrow": "OAuth scope tiers of Read, Draft, Write and Approve, where Approve is granted only on its own: the same separation ownpurse makes between drafting a package and a person approving it. An API role ('clerk') that can post only drafts. Every agent action attributed to the agent in the audit trail.",
 "avoid": "One token that 'provides full access to your Campfire data' for an MCP client; confidence thresholds as the main gate on unattended writes; accuracy figures from a vendor's own benchmark with no independent check.",
 "sources": [
  {
   "title": "Campfire home page",
   "url": "https://campfire.ai/"
  },
  {
   "title": "Campfire raises $65 million Series B (PR Newswire, 15 Oct 2025)",
   "url": "https://www.prnewswire.com/news-releases/campfire-raises-65-million-series-b-to-redefine-how-finance-works-in-the-ai-era-302585077.html"
  },
  {
   "title": "Campfire launches Ember Agents (press release, 12 Mar 2026)",
   "url": "https://fortune.com/press-releases/campfire-ember-agents-ai-workers-finance-accounting-2026-03-12"
  },
  {
   "title": "Campfire is the first ERP with an MCP store (14 May 2026)",
   "url": "https://campfire.ai/blog/campfire-is-the-first-erp-with-an-mcp-store"
  },
  {
   "title": "Campfire docs: AI integration with MCP",
   "url": "https://docs.campfire.ai/guides/ai-integration"
  },
  {
   "title": "Campfire docs: quickstart",
   "url": "https://docs.campfire.ai/quickstart"
  },
  {
   "title": "Campfire docs: OAuth apps",
   "url": "https://docs.campfire.ai/guides/oauth-apps"
  },
  {
   "title": "Campfire docs index (llms.txt)",
   "url": "https://docs.campfire.ai/llms.txt"
  },
  {
   "title": "ERP Scorecard: Campfire profile",
   "url": "https://erpscorecard.com/erp-systems/campfire"
  },
  {
   "title": "Cyber News Centre: Campfire raises $35M",
   "url": "https://www.cybernewscentre.com/campfire-raises-35m-to-replace-legacy-erp-with-ai-platform/"
  },
  {
   "title": "FinTech Global: Campfire secures $65m Series B",
   "url": "https://fintech.global/2025/10/16/campfire-secures-65m-series-b-to-transform-finance-ai/"
  }
 ],
 "unverified": [
  "Founding year 2023 and co-founder Fernando San Martin come from ERP Scorecard, not Campfire's own pages",
  "Exact Series A date: reports say June or July 2025 (12 weeks before the 15 Oct 2025 Series B suggests late July)",
  "Total funding: sources say 'more than $100M' or $103.5M",
  "Pricing: ERP Scorecard's $20K-$40K/yr software and $30K-$55K year-one estimates are peer-anchored and low confidence",
  "G2 average rating and exact review count not confirmed (G2 blocked fetch)",
  "Whether any Campfire MCP server tool writes; one third-party site calls it read-only",
  "About 40 employees as of late 2025 (ERP Scorecard only)",
  "London office in 2026 and packaged Ember close skills in July 2026 come from third-party reviews",
  "SOC 2 status not confirmed on the home page",
  "Reddit and Trustpilot sentiment not checked: web search budget ran out during this pass"
 ],
 "researched": "2026-10-10"
}
---

## What it is

Campfire is a cloud ERP for finance teams at venture-backed and mid-market companies. It sells itself as a modern replacement for NetSuite, SAP and Oracle: a multi-entity general ledger, revenue recognition, close management, reporting and forecasting, with an agent called Ember working inside it ([Campfire](https://campfire.ai/)). It claims a 3 to 10 day faster close, 180+ currencies, unlimited entities and 100+ native integrations. It does not handle inventory, manufacturing or project accounting ([ERP Scorecard](https://erpscorecard.com/erp-systems/campfire)).

## How it works

You move your books into Campfire. Bank feeds arrive through Plaid, and CRM, billing and spend data through native integrations. In October 2025 Campfire announced its own "Large Accounting Model", trained on accounting data, with a vendor-claimed 95% accuracy on reconciliations and variance detection ([PR Newswire](https://www.prnewswire.com/news-releases/campfire-raises-65-million-series-b-to-redefine-how-finance-works-in-the-ai-era-302585077.html)). Ember Agents, in beta from 12 March 2026, run all the time on transaction matching, AP and AR, anomaly detection and accruals, and on a schedule for flux analysis and close prep. Confidence thresholds set what an agent does on its own and what it flags, and you can review actions before they post ([press release](https://fortune.com/press-releases/campfire-ember-agents-ai-workers-finance-accounting-2026-03-12)).

## Data and access

The API is public and large: about 396 documentation pages across cash management, core accounting, AP, AR, bank reconciliation, revenue recognition and financial statements, with an OpenAPI spec and webhooks ([docs index](https://docs.campfire.ai/llms.txt)). Keys are self-serve, sent as `Authorization: Token`, and carry a role: admin, clerk (can post draft entries) or view only ([quickstart](https://docs.campfire.ai/quickstart)). OAuth apps use scopes like `invoices:read` with tiers Read, Draft, Write and Approve, where Approve is granted only on its own. Support has to switch OAuth apps on, and Campfire reviews any app published beyond your workspace ([OAuth apps](https://docs.campfire.ai/guides/oauth-apps)). Rate limits and bulk export are not documented. Pricing is by quote; there was no pricing page on 10 October 2026.

## Agents

Campfire works with agents in both directions. Outward, an official MCP server at `api.meetcampfire.com/mcp` exposes 12 tools covering accounts, transactions, entities, vendors, the three statements, budgets, contracts and aging. It is documented for Cursor, Claude Desktop and VS Code, and the docs warn that the token "provides full access to your Campfire data" ([AI integration](https://docs.campfire.ai/guides/ai-integration)). Inward, the May 2026 MCP store lets Ember call Ramp, HubSpot, Salesforce, Stripe or any hosted MCP server, then post accruals, reconcile payments and draft invoices in Campfire. Invoices wait for a person to approve them, and every action is logged and attributed to Ember ([Campfire blog](https://campfire.ai/blog/campfire-is-the-first-erp-with-an-mcp-store)).

## Customers

Named customers include Replit, PostHog, Decagon, Klarity, Heidi Health, CloudZero and Advisor360, and Campfire claimed 10x year-to-date revenue growth in October 2025. ERP Scorecard counted roughly two dozen G2 reviews in mid-2026. It says reviewers praise the interface, automatic FX booking and support, and complain about the narrow tech-company fit, features still being built and opaque pricing. It names the best fit as companies with about $5M to $250M in revenue.

## History

John Glasgow and Fernando San Martin founded Campfire in 2023 through Y Combinator. It launched publicly in late 2024, raised a $35M Series A led by Accel in July 2025 ([Cyber News Centre](https://www.cybernewscentre.com/campfire-raises-35m-to-replace-legacy-erp-with-ai-platform/)), and raised a $65M Series B co-led by Accel and Ribbit on 15 October 2025, for more than $100M in total. It opened a London office in 2026.

## For ownpurse

Campfire is the replacement-ledger thesis done well: one cloud, one model, one agent, with your books inside. It leaves out the pole ownpurse takes, where the owner holds the record and the books stay in Xero or wherever they already are. Two things are worth taking. One is the scope ladder, where Approve is a separate grant and not the top of Write. The other is the clerk role that can only post drafts. Both match ownpurse's split between drafting a package and a person approving it. What to avoid is one all-access token handed to any MCP client, and confidence thresholds as the only gate on writes nobody watches. Campfire is a neighbour, and a possible future source, but not a competitor for the people and small businesses ownpurse serves.
