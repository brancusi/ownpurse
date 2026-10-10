---
{
 "slug": "intuit-quickbooks-ai-agents",
 "name": "Intuit AI agents for QuickBooks",
 "url": "https://quickbooks.intuit.com/ai-agents/",
 "segment": "agent-beside-ledger",
 "status": "active",
 "founded": "Agents announced 1 July 2025 (Intuit founded 1983)",
 "hq": "Mountain View, California, US",
 "backing": "Intuit Inc., public on Nasdaq (INTU). FY2026 (year to 31 July 2026) revenue $21.4 billion, up 14%; Global Business Solutions $12.9 billion, up 16% (results of 25 August 2026).",
 "tagline": "The agents Intuit builds into QuickBooks Online itself (accounting, payments, finance, customer and others), now sold as 'Intuit Intelligence' features of each plan.",
 "scope": "Inside QuickBooks Online and Intuit Enterprise Suite, Intuit's agents categorize and verify bank transactions, assist with and run reconciliation, chase invoices, forecast, find leads and draft emails, with human experts on QuickBooks Live. They act on Intuit's ledger in Intuit's cloud; there is no owner-held copy, and outside agents reach the books only through the public API or Intuit's MCP server, not through these agents.",
 "domains": [
  "ledger",
  "bank",
  "payments",
  "invoicing",
  "payroll",
  "reporting",
  "close",
  "spend",
  "tax"
 ],
 "customers_served": [
  "smb",
  "mid-market",
  "accountants",
  "developers"
 ],
 "spectrum": {
  "custody": {
   "score": 0,
   "note": "Intuit holds the only authoritative copy; the owner gets reports, exports and metered API reads."
  },
  "locality": {
   "score": 0,
   "note": "The agents run only in Intuit's cloud; the official MCP server runs locally but only as a client of the cloud API."
  },
  "history": {
   "score": 1,
   "note": "QuickBooks keeps an audit log and a 30-day change-data-capture feed, but the books are current state and edits overwrite."
  },
  "openness": {
   "score": 2,
   "note": "Public REST API and an Apache-licensed MCP server, but the agents are closed and reads are metered."
  },
  "breadth": {
   "score": 3,
   "note": "Books, bank feeds, payments, payroll, CRM, Mailchimp marketing, sales tax and a business card in one platform, all Intuit's."
  },
  "autonomy": {
   "score": 3,
   "note": "Books Upkeep resolves high-confidence transactions on its own under customer rules; chat acts only after the user confirms."
  },
  "agent_readiness": {
   "score": 3,
   "note": "An official 145-tool MCP server with writes on by default and a hosted Claude connector exist, but the in-product agents cannot be driven from outside."
  },
  "integration_ease": {
   "score": 2,
   "note": "Large documented app ecosystem and sandbox, but OAuth app registration, a public HTTPS callback in production, metered reads and premium APIs behind paid tiers."
  },
  "market": {
   "score": 3,
   "note": "Small business at the core, pushed into the mid-market through QuickBooks Advanced and Intuit Enterprise Suite."
  }
 },
 "api": {
  "public": true,
  "style": "REST (QuickBooks Online Accounting API) with batch operations, change data capture and webhooks; an official local MCP server (stdio, TypeScript) over the same API.",
  "auth": "OAuth 2.0 per company; the MCP server needs a registered Intuit developer app, and production requires a public HTTPS redirect for the first handshake; refresh tokens lapse after 100 days unused.",
  "limits": "Per-tier monthly CorePlus (read) credit allowances; Builder calls are blocked once 500,000 credits are used. Per-second rate limits not rechecked in this pass.",
  "cost": "Intuit App Partner Program from 28 July 2025: writes (Core) free and unlimited; reads (CorePlus) metered. Builder $0 with 500,000 credits a month; Silver $300 a month with 1 million and $3.50 per 1,000 over; Gold $1,700; Platinum $4,500 with up to 75 million and $0.25 per 1,000 over. Variable fees began 1 November 2025 (per Apideck's summary).",
  "export": "Reports and lists to Excel/CSV/PDF, and the full API (metered reads). No single bulk export of every transaction and its history.",
  "notes": "Premium APIs (projects, custom fields, sales tax, time with payroll) need a paid tier. Pricing favours writes into QuickBooks over reads out of it."
 },
 "pricing": {
  "model": "Agents are bundled into QuickBooks Online plans as 'Intuit Intelligence' features, more of them on higher tiers; human experts are extra through QuickBooks Live.",
  "entry": "Simple Start, list $38 a month (promo $19 for three months): AI chat (limited, beta), expense categorization, task automation (beta). A Free plan has no AI features. Checked 2026-10-10.",
  "top": "Advanced, list $340 a month (promo $170): Finance AI, Project Management AI, continuous books checks. Intuit Enterprise Suite above that, by quote.",
  "notes": "Bank reconciliation and auto-categorized, verified transactions start at Plus (list $140); Accounting AI and Payments AI at Essentials (list $85). Intuit's release says 'additional terms, conditions and fees may apply'."
 },
 "agents": {
  "mcp": "official",
  "ai_features": "Accounting agent (categorization, reconciliation help, anomaly detection), Payments agent (invoices, reminders, late-payment prediction; Intuit claims paid 5 days faster), Finance agent (KPIs, forecasting, scenarios), Customer agent (leads, email drafts, scheduling), Marketing via Mailchimp, Payroll and Project Management agents, Intuit Intelligence Chat (beta), Books Upkeep on Advanced.",
  "agent_writes": "Yes, directly into QuickBooks. Books Upkeep resolves high-confidence transactions automatically; the official MCP server exposes create, update and delete on 29 entities with writes enabled unless switched off by environment flags.",
  "approval": "Mixed. Chat 'acts only after the user confirms an action'; Books Upkeep applies customer-set rules and surfaces only exceptions; AI-drafted intercompany entries in Intuit Enterprise Suite are reviewed and approved by people. Automated actions are labelled. There is no sealed package and no owner-held record of what was approved.",
  "notes": "Intuit's hosted Claude connector is described by a competitor (Pilot's Meridian, August 2026) as effectively read-only and limited to reports. The open-source MCP server lacks the API's batch, CDC and webhooks."
 },
 "integration": {
  "scope": "QuickBooks Online, Intuit Enterprise Suite, QuickBooks Payments, Payroll, Workforce, Mailchimp, a QuickBooks business card (July 2026), bank feeds, and the QuickBooks App Store of third-party apps.",
  "notes": "Agents work only on Intuit's own products and data. Outside systems come in through the App Store and API; outside agents through the MCP server or third-party connectors."
 },
 "preference": {
  "scale": "QuickBooks Online Accounting revenue up 23% in FY2026; Intuit reports about 100 million customers worldwide across all products (25 August 2026). No agent adoption figures published.",
  "ratings": "Trustpilot (quickbooks.intuit.com): 3.7 TrustScore from 16,890 reviews, 770 in the last 12 months (2026-10-10). G2 and Capterra pages blocked access in this pass.",
  "sentiment": "Praised for onboarding staff and phone support. Complaints: recent interface changes hide tasks and add clicks, slow loads and sync bugs, support waits, billing after cancellation. One October 2026 reviewer says Intuit tries to 'bolt AI onto everything'."
 },
 "timeline": [
  {
   "date": "2025-06-27",
   "event": "Trade press previews the agent rollout"
  },
  {
   "date": "2025-07-01",
   "event": "Intuit announces a 'virtual team of AI agents' for QuickBooks Online, rolling out in the US"
  },
  {
   "date": "2025-07-28",
   "event": "Intuit App Partner Program: QuickBooks API reads metered, writes free"
  },
  {
   "date": "2025-10-06",
   "event": "Intuit publishes the open-source QuickBooks Online MCP server (Apache-2.0)"
  },
  {
   "date": "2025-11-01",
   "event": "Variable API read fees begin (per Apideck)"
  },
  {
   "date": "2026-05-13",
   "event": "Mid-market release combining data and AI"
  },
  {
   "date": "2026-07-22",
   "event": "QuickBooks business credit card with spend management"
  },
  {
   "date": "2026-08-12",
   "event": "Intuit Intelligence Chat (beta), Books Upkeep, AI-drafted intercompany entries for mid-market"
  },
  {
   "date": "2026-08-25",
   "event": "FY2026 results: revenue $21.4 billion; QuickBooks Online Accounting revenue up 23%"
  }
 ],
 "relation": "neighbour",
 "relation_note": "The incumbent putting agents inside the ledger it owns. QuickBooks is a planned ownpurse source, and Intuit's agents do the same daily work ownpurse's agents would, but in Intuit's cloud, on Intuit's copy, with reads out metered and writes in free.",
 "borrow": "Labelling every automated action; 'acts only after the user confirms' as the default for chat; customer-set rules for what may resolve automatically; an agentic readiness check that validates data before a migration; the MCP server's switches that turn off create, update and delete tools.",
 "avoid": "Writes enabled by default in an agent interface; pricing that meters reads out of the books; agents that only work on one vendor's data; a ledger where an edit overwrites what was there; AI features gated by plan tier so the owner's own data costs more to understand.",
 "sources": [
  {
   "title": "Intuit press release, 1 July 2025: virtual team of AI agents",
   "url": "https://investors.intuit.com/news-events/press-releases/detail/1258/intuit-introduces-ground-breaking-virtual-team-of-ai-agents-to-fuel-growth-for-businesses"
  },
  {
   "title": "Intuit press release, 12 August 2026: mid-market platform with conversational AI",
   "url": "https://investors.intuit.com/news-events/press-releases/detail/1319/intuit-advances-its-mid-market-platform-with-conversational-ai-enterprise-scale-and-deep-industry-workflows-for-cfos-and-accounting-firms"
  },
  {
   "title": "Intuit FY2026 results, 25 August 2026",
   "url": "https://investors.intuit.com/news-events/press-releases/detail/1320/intuit-reports-fourth-quarter-and-full-year-fiscal-2026-results-sets-fiscal-2027-guidance"
  },
  {
   "title": "Intuit press releases index",
   "url": "https://investors.intuit.com/news-events/press-releases"
  },
  {
   "title": "QuickBooks Online pricing",
   "url": "https://quickbooks.intuit.com/pricing/"
  },
  {
   "title": "Intuit QuickBooks Online MCP server (GitHub)",
   "url": "https://github.com/intuit/quickbooks-online-mcp-server"
  },
  {
   "title": "Apideck: QuickBooks API pricing and the Intuit App Partner Program",
   "url": "https://www.apideck.com/blog/quickbooks-api-pricing-and-the-intuit-app-partner-program"
  },
  {
   "title": "Meridian (Pilot): QuickBooks MCP server",
   "url": "https://meridian.pilot.com/blog/quickbooks-mcp-server"
  },
  {
   "title": "Trustpilot: QuickBooks",
   "url": "https://www.trustpilot.com/review/quickbooks.intuit.com"
  },
  {
   "title": "CPA Practice Advisor: Intuit rolls out AI agents for QuickBooks",
   "url": "https://www.cpapracticeadvisor.com/2025/06/27/intuit-rolls-out-ai-agents-for-quickbooks/163868/"
  }
 ],
 "unverified": [
  "The URL quickbooks.intuit.com/ai-agents/ returned 404 on 2026-10-10; no current dedicated agents page was found",
  "App Partner Program tier figures come from Apideck's summary; Intuit's developer pages did not render in this pass. Gold tier allowances and overage not confirmed",
  "The 1 November 2025 start of variable fees (per Apideck)",
  "Intuit's hosted Claude connector being read-only and report-only is a competitor's description (Meridian), not Intuit's",
  "Intuit Assist (2023) and the generative AI platform history were not rechecked in this pass",
  "QuickBooks Online subscriber counts; Intuit gives only about 100 million customers across all products",
  "G2 and Capterra ratings (pages blocked); no Reddit sentiment gathered because the web search budget ran out",
  "Per-second and per-minute API rate limits were not rechecked",
  "Any Intuit partnership with OpenAI or Anthropic for ChatGPT or Claude apps was not checked"
 ],
 "researched": "2026-10-10"
}
---

## What it is

On [1 July 2025](https://investors.intuit.com/news-events/press-releases/detail/1258/intuit-introduces-ground-breaking-virtual-team-of-ai-agents-to-fuel-growth-for-businesses) Intuit announced a 'virtual team of AI agents' inside QuickBooks Online, rolling out to US customers: an Accounting agent (categorization, reconciliation help), a Payments agent (invoices, reminders, late-payment prediction), a Finance agent (KPIs and forecasts for mid-market businesses) and a Customer agent (leads, email drafts, meetings), with Marketing, Payroll and Project Management agents to follow. Human experts sit alongside them through QuickBooks Live.

By October 2026 the [pricing page](https://quickbooks.intuit.com/pricing/) no longer says 'agents'. The same jobs are 'Intuit Intelligence' features sold by plan: Accounting AI and Payments AI from Essentials, auto-categorized and verified transactions and reconciliation from Plus, Finance AI from Advanced. The URL given for this dossier, quickbooks.intuit.com/ai-agents/, returned 404 on 10 October 2026.

Intuit does a lot here well. The agents are where the data already is, cover payments and CRM as well as books, and reach millions of small businesses without an integration.

## How it works

The agents run in Intuit's cloud on Intuit's ledger. The [August 2026 release](https://investors.intuit.com/news-events/press-releases/detail/1319/intuit-advances-its-mid-market-platform-with-conversational-ai-enterprise-scale-and-deep-industry-workflows-for-cfos-and-accounting-firms) states the trust model plainly: Intuit Intelligence Chat 'acts only after the user confirms an action'; Books Upkeep on Advanced resolves high-confidence transactions automatically under customer-set rules and surfaces only exceptions; automated actions are labelled; AI-drafted intercompany entries in Intuit Enterprise Suite are reviewed and approved by people. A footnote warns that Intuit Intelligence can make mistakes. Intuit's headline saving, 'up to 12 hours a month', comes from a commissioned survey where 45% of users of the AI bank feed reported it.

## Data and access

The owner does not hold a copy. You get reports, exports and the API. Since [28 July 2025](https://www.apideck.com/blog/quickbooks-api-pricing-and-the-intuit-app-partner-program) the App Partner Program meters reads and leaves writes free: Builder is $0 with 500,000 read credits a month, then blocked; Silver $300 a month plus $3.50 per 1,000 over 1 million; Platinum $4,500 a month for up to 75 million at $0.25 per 1,000. Reading your own books out of QuickBooks at scale costs a developer money; writing into them does not.

## Agents

For outside agents, Intuit published an [open-source MCP server](https://github.com/intuit/quickbooks-online-mcp-server) on 6 October 2025 (Apache-2.0, 417 stars by October 2026). It runs locally over stdio, exposes 145 tools across 29 entities and 11 reports, and has create, update and delete enabled unless you set `QUICKBOOKS_DISABLE_WRITE` and its siblings. It needs a registered developer app and a public HTTPS callback for the first production login. It lacks the API's batch, change data capture and webhooks. A [competitor's review](https://meridian.pilot.com/blog/quickbooks-mcp-server) (Pilot's Meridian, August 2026) describes Intuit's hosted Claude connector as easy to connect but effectively read-only and limited to reports. The in-product agents themselves cannot be driven from outside.

## Customers

Intuit reported FY2026 revenue of [$21.4 billion](https://investors.intuit.com/news-events/press-releases/detail/1320/intuit-reports-fourth-quarter-and-full-year-fiscal-2026-results-sets-fiscal-2027-guidance), Global Business Solutions $12.9 billion and QuickBooks Online Accounting revenue up 23%. It publishes no agent adoption figures. On [Trustpilot](https://www.trustpilot.com/review/quickbooks.intuit.com) QuickBooks has a 3.7 TrustScore from 16,890 reviews: onboarding staff are praised; interface changes, bugs, support waits and billing are the complaints, and one October 2026 reviewer says Intuit tries to 'bolt AI onto everything'.

## History

The agents followed Intuit Assist and a generative AI platform Intuit has built since 2023. The arc since July 2025: agents announced, API reads metered, an MCP server published, then the agents folded into plan features and pushed up into the mid-market with Intuit Enterprise Suite, Books Upkeep and a business card in 2026.

## For ownpurse

Intuit is the strongest version of 'the agent lives inside the ledger'. That is convenient and also total custody: one vendor's books, one vendor's agents, reads metered out and writes free in. ownpurse's position is the inverse. The record is held by the owner and covers QuickBooks alongside Xero and the rest, reads are local and free, and writes go back only as a sealed package a person approves. Intuit's own rules, confirm before acting and label every automated action, are worth matching. Its MCP default of writes on is not.
