---
{
 "slug": "xero-jax",
 "name": "JAX (Just Ask Xero)",
 "url": "https://www.xero.com/us/jax/",
 "segment": "agent-native",
 "status": "active",
 "founded": "Announced 2024-02-29 (Xero founded 2006)",
 "hq": "Wellington, New Zealand (Xero Limited)",
 "backing": "Part of Xero Limited, listed on the ASX (XRO). Xero agreed to acquire Melio for US$2.5B upfront plus up to US$0.5B contingent (announced 2025-06-25). Multi-year partnership with Anthropic (2026-03-27); JAX web research via a collaboration with OpenAI (2025-09-03).",
 "tagline": "Xero's in-product AI agent (its 'financial superagent') that answers questions, reconciles, chases payments and orchestrates other agents inside Xero.",
 "scope": "JAX is a chat and agent layer inside Xero: answers about cash, profit and invoices, help questions, small admin tasks such as invoices from quotes, auto bank reconciliation of high-confidence matches, payment follow-ups, bill inspection, cash-flow plans and document requests. It runs only on Xero's data and inside Xero OS; it has no API of its own. Outside agents reach Xero through separate channels: a read-only Claude connector (May 2026), Xero's open-source MCP server with create and update tools, and XeroForce, an invite-only agent builder. JAX does not reach across other ledgers or keep a copy you hold.",
 "domains": [
  "ledger",
  "bank",
  "invoicing",
  "payments",
  "documents",
  "close",
  "reporting",
  "payroll"
 ],
 "customers_served": [
  "smb",
  "freelancer",
  "accountants"
 ],
 "spectrum": {
  "custody": {
   "score": 1,
   "note": "Xero holds the books; you can export reports and pull data by API, but bulk reads are metered and the Journals feed is gated to an advanced tier."
  },
  "locality": {
   "score": 0,
   "note": "JAX runs only in Xero's cloud; the open-source MCP server can run locally but only relays to Xero's API."
  },
  "history": {
   "score": 1,
   "note": "Xero keeps a per-document history and notes trail; JAX logs actions for review, but there is no append-only record or as-of query."
  },
  "openness": {
   "score": 1,
   "note": "JAX is closed with no API; Xero's API is public but egress is metered, AI training on API data is banned, and there is no full export. The MCP server is MIT-licensed."
  },
  "breadth": {
   "score": 3,
   "note": "Books, bank feeds, bill pay (Melio in the US), document capture, payroll in some regions and practice tools in one platform."
  },
  "autonomy": {
   "score": 3,
   "note": "Auto bank reconciliation matches high-confidence lines in real time and payment follow-ups 'automatically execute'; exceptions and the XeroForce month-end agent's actions go to a person to review and accept."
  },
  "agent_readiness": {
   "score": 3,
   "note": "Not JAX itself, but Xero offers an official open-source MCP server (writes, 1M+ calls by June 2026), a read-only hosted Claude connector and an agent builder; held back by egress fees, a ban on bots that simulate users and no approval gate in the MCP."
  },
  "integration_ease": {
   "score": 2,
   "note": "Well documented and self-serve, with 1,000+ certified apps, but tiered developer plans (free Starter at 5 connections, then A$35 to A$1,445 a month plus A$2.40/GB egress) and certification for scale."
  },
  "market": {
   "score": 2,
   "note": "Small businesses and their accountants and bookkeepers."
  }
 },
 "api": {
  "public": true,
  "style": "JAX: none. Xero: REST accounting API, webhooks for a few objects; MCP server (stdio, Node) over that API",
  "auth": "OAuth 2.0 (custom connections for one organisation, or bearer tokens from a PKCE flow); the Claude connector uses Xero login and organisation choice",
  "limits": "Starter tier 5 connections and 1,000 calls a day per organisation for a new app (per ownpurse's own docs); other per-minute and daily limits not re-checked here",
  "cost": "JAX: 'no additional charge' to subscribers for the chat experience, with the right to change pricing reserved. API from 2026-03-02: Starter free (5 connections, no egress allowance stated), Core A$35/month (50 connections, 10 GB), Plus A$245 (1,000, 50 GB), Advanced A$1,445 (10,000, 250 GB), Enterprise quoted; overage A$2.40/GB; ingress unlimited",
  "export": "Reports and lists to CSV, Excel or PDF from the app; bulk data only through the API, metered by egress; no single full-ledger export file",
  "notes": "Developer terms prohibit using API data to train AI/ML models (new developers from 4 Dec 2025, existing from 2 Mar 2026)."
 },
 "pricing": {
  "model": "Included in Xero subscriptions; Xero plans are per organisation per month",
  "entry": "JAX chat: no additional charge on any Xero plan",
  "top": "Not separately priced; XeroForce pricing not announced",
  "notes": "Xero reserves the right to change JAX terms and pricing. XeroForce is alpha, invite-only (May 2026), with general release 'later this year'."
 },
 "agents": {
  "mcp": "official",
  "ai_features": "JAX chat with charts and tables, Xero help answers, invoices from quotes, auto bank reconciliation, Smart Document Capture, document requests and reminders, payment follow-ups, Bill Protection, cash-flow actions, JAX Assure accuracy controls, XeroForce custom agent builder and month-end agent, Claude and Microsoft 365 integrations",
  "agent_writes": "JAX writes inside Xero (reconciliations, collection plans, documents). The open-source Xero MCP server lets outside agents create and update invoices, bank transactions, manual journals, payments, contacts and more directly. The hosted Claude connector is read-only at launch.",
  "approval": "Mixed and per feature: document requests let customers 'approve every step'; the month-end agent gives 'a complete account of its actions to review and accept'; high-confidence reconciliations and payment follow-ups run automatically. The MCP server has no confirmation step of its own. JAX respects Xero user permissions.",
  "notes": "Xero frames this as 'Accountable Intelligence' and 'humans at the helm'. JAX is powered by Claude since the Anthropic partnership (March 2026). Xero says MCP server usage grew ten-fold from December 2025 to May 2026."
 },
 "integration": {
  "scope": "Xero App Store with 1,000+ certified apps; about 20% of ecosystem connections are custom apps; new app registrations up fourfold since 2025 (Xero, Aug 2026); Claude and Microsoft 365 connectors; XeroForce connects to third-party apps",
  "notes": "JAX itself does not integrate outward; it is the reason to stay inside Xero. The ecosystem is broad, but bulk reads now cost money."
 },
 "preference": {
  "scale": "Xero: 5 million customers globally (Xero, 9 July 2026); 'thousands of customers' use the Claude integration daily; 100M+ transactions auto-reconciled since launch (Aug 2026)",
  "ratings": "Trustpilot (Xero): 4.1/5 from 11,430 reviews; Apple App Store (US): 4.6/5 from 1.7K ratings (both read 2026-10-10). G2 and Capterra blocked.",
  "sentiment": "Xero is praised for ease of use, onboarding and patient support; complaints are slow support, setup complexity (payroll), price, and billing after cancellation. No review found that discusses JAX specifically."
 },
 "timeline": [
  {
   "date": "2023-10-12",
   "event": "GenAI search for answers in Xero Central"
  },
  {
   "date": "2024-02-29",
   "event": "JAX ('Just Ask Xero') announced at Xero's first Investor Day; beta promised later in 2024"
  },
  {
   "date": "2024-06-13",
   "event": "Xero describes the tech behind JAX for accuracy"
  },
  {
   "date": "2024-08-14",
   "event": "GenAI companion rolled out to small businesses"
  },
  {
   "date": "2025-06-25",
   "event": "Xero agrees to acquire Melio for US$2.5B upfront"
  },
  {
   "date": "2025-09-03",
   "event": "JAX relaunched as a 'financial superagent' with JAX Assure and OpenAI web research"
  },
  {
   "date": "2026-03-02",
   "event": "New developer tiers with egress fees and a ban on training AI with API data take effect"
  },
  {
   "date": "2026-03-27",
   "event": "Multi-year partnership with Anthropic; JAX to be powered by Claude"
  },
  {
   "date": "2026-05-13",
   "event": "Read-only Claude connector for Xero data"
  },
  {
   "date": "2026-05-14",
   "event": "XeroForce agent builder in invite-only alpha"
  },
  {
   "date": "2026-07-09",
   "event": "Xerocon London: payment follow-ups, Bill Protection, cash-flow actions; 5 million customers"
  },
  {
   "date": "2026-08-20",
   "event": "Xerocon US: auto bank reconciliation, Smart Document Capture, document requests, XeroForce month-end agent in early access"
  }
 ],
 "relation": "neighbour",
 "relation_note": "JAX works on the same Xero books ownpurse reads as its first source. It keeps agent work inside Xero; ownpurse gives the owner and their own agents a local, cross-provider record and a reviewed path for writes. JAX does not compete for custody, it competes for where agent work happens.",
 "borrow": "Per-feature approval language ('review and accept', 'approve every step'); a complete account of an agent's actions shown before acceptance; respecting the source's user permissions; a read-only connector as the safe first step; free inclusion of the agent in the base plan.",
 "avoid": "Agents that only work inside one provider's walls; an MCP server that writes directly with no confirmation step; approval rules that differ feature by feature with no single gate; metering reads so the owner pays to see their own data.",
 "sources": [
  {
   "title": "JAX product page (US)",
   "url": "https://www.xero.com/us/jax/"
  },
  {
   "title": "JAX vision page",
   "url": "https://www.xero.com/us/ai-in-accounting/jax-vision/"
  },
  {
   "title": "Xero unveils its AI vision (2024-02-29)",
   "url": "https://www.xero.com/media-releases/xero-unveils-its-ai-vision-to-reimagine-small-business-accounting/"
  },
  {
   "title": "JAX superagent launch (2025-09-03)",
   "url": "https://www.xero.com/media-releases/xeros-ai-financial-superagent-jax-launches-powerful-new-features/"
  },
  {
   "title": "Xero and Anthropic collaborate (2026-03-27)",
   "url": "https://www.xero.com/us/media-releases/xero-and-anthropic-collaborate/"
  },
  {
   "title": "Xero delivers Claude integration (2026-05-13)",
   "url": "https://www.xero.com/us/media-releases/xero-delivers-claude-integration-to-advance-ai-powered-financial-intelligence/"
  },
  {
   "title": "Xero Claude connector",
   "url": "https://www.xero.com/us/campaign/claude-connector/"
  },
  {
   "title": "XeroForce (2026-05-14)",
   "url": "https://www.xero.com/us/media-releases/xero-introduces-xeroforce/"
  },
  {
   "title": "Xerocon London (2026-07-09)",
   "url": "https://www.xero.com/us/media-releases/xero-announces-new-ai-innovations-xerocon-london/"
  },
  {
   "title": "Xerocon US (2026-08-20)",
   "url": "https://www.xero.com/us/media-releases/new-ai-innovations-xerocon-denver/"
  },
  {
   "title": "Xero media releases",
   "url": "https://www.xero.com/us/media-releases/"
  },
  {
   "title": "Xero developer pricing",
   "url": "https://developer.xero.com/pricing"
  },
  {
   "title": "XeroAPI/xero-mcp-server",
   "url": "https://github.com/XeroAPI/xero-mcp-server"
  },
  {
   "title": "Melio: Xero acquiring Melio FAQ",
   "url": "https://melio.com/xero-melio-faq/"
  },
  {
   "title": "Trustpilot: Xero",
   "url": "https://www.trustpilot.com/review/xero.com"
  },
  {
   "title": "Apple App Store: Xero",
   "url": "https://apps.apple.com/us/app/xero-accounting-for-business/id441880705"
  }
 ],
 "unverified": [
  "Completion date of the Melio acquisition (FAQ only says targeted within six months of 2025-06-25)",
  "Xero API per-minute and daily rate limits beyond ownpurse's own note of 1,000 calls a day per organisation for a new app",
  "Whether JAX is available on every Xero plan and region (US page says no additional charge; regional availability not stated)",
  "XeroForce pricing and general-availability date",
  "The 'humans at the helm' phrase appears in the XeroForce release, not on the JAX pages",
  "G2 and Capterra ratings (pages blocked); Reddit sentiment (unreachable)",
  "Whether Xero's MCP server is distinct from the hosted Claude connector's backend (Xero does not say)"
 ],
 "researched": "2026-10-10"
}
---

## What it is

JAX, short for Just Ask Xero, is Xero's agent inside Xero. Xero [announced it on 29 February 2024](https://www.xero.com/media-releases/xero-unveils-its-ai-vision-to-reimagine-small-business-accounting/) as a GenAI companion and [relaunched it on 3 September 2025](https://www.xero.com/media-releases/xeros-ai-financial-superagent-jax-launches-powerful-new-features/) as a "financial superagent" that orchestrates other agents. Since the [Anthropic partnership of 27 March 2026](https://www.xero.com/us/media-releases/xero-and-anthropic-collaborate/) it is powered by Claude.

## How it works

In the [product page](https://www.xero.com/us/jax/)'s words, JAX gives "instant, easy-to-understand answers" about your financials, answers Xero help questions and does small admin tasks such as invoices from quotes. Through 2026 Xero has attached more work to it: auto bank reconciliation of high-confidence matches, Smart Document Capture, document requests to clients, payment follow-ups that "automatically execute tailored collection plans", Bill Protection, and cash-flow actions ([London](https://www.xero.com/us/media-releases/xero-announces-new-ai-innovations-xerocon-london/), [Denver](https://www.xero.com/us/media-releases/new-ai-innovations-xerocon-denver/)). [XeroForce](https://www.xero.com/us/media-releases/xero-introduces-xeroforce/), an invite-only agent builder since May 2026, adds a month-end agent that gives you "a complete account of its actions to review and accept."

Approval is decided feature by feature. Document requests let you approve every step; high-confidence reconciliations and payment follow-ups run on their own. JAX follows your Xero user permissions.

## Data and access

JAX has no API. Outside agents reach Xero three other ways. The [Claude connector](https://www.xero.com/us/campaign/claude-connector/) (13 May 2026) is free and "read-only at launch". Xero's open-source [MCP server](https://github.com/XeroAPI/xero-mcp-server) (MIT) has create and update tools for invoices, bank transactions, manual journals, payments and contacts, with no confirmation step; Xero says its usage grew ten-fold from December 2025 to May 2026 and passed a million API calls. And the public API is now priced: from 2 March 2026 the [developer tiers](https://developer.xero.com/pricing) run from a free Starter (5 connections) to A$1,445 a month, with egress at A$2.40 per GB and a ban on training AI with API data.

## Agents

JAX is the incumbent's answer to agents: keep the work in Xero, where Xero's controls (JAX Assure, permissions, review screens) apply. That is a reasonable answer for a small business that lives in Xero. It gives little to an owner whose money is spread across Xero, a bank, a card and a brokerage, and nothing to an agent the owner runs themselves, except a metered API or an MCP server that writes without asking.

## Customers

Xero reports 5 million customers (July 2026) and 100 million transactions auto-reconciled. Xero rates 4.1 on Trustpilot from 11,430 reviews and 4.6 on the US App Store from 1,700 ratings. Reviewers praise ease of use and onboarding and complain about slow support and billing; none of the reviews read mentioned JAX.

## History

Xero's AI work goes back to ML bank-reconciliation suggestions in 2021. JAX was promised in 2024, rebuilt as an agent platform in 2025, and in 2026 became the frame for Xero OS, the Melio payments business and the Anthropic partnership.

## For ownpurse

JAX is not a rival for custody, since it holds nothing Xero did not already hold. It competes for where agent work happens. ownpurse's answer is a record the owner holds, across providers, read at no API cost, with every write to Xero going out as a package a person approves. Two things to borrow: showing a full account of an agent's actions before acceptance, and respecting the source's own permissions. Xero's own terms (no bots that simulate users, no training on API data, metered egress) set the limits ownpurse's Xero connector has to work within.
