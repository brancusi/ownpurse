---
{
 "slug": "zapier",
 "name": "Zapier",
 "url": "https://zapier.com",
 "segment": "ipaas-middleware",
 "status": "active",
 "founded": "2011 (side project in Columbia, Missouri; launched through Y Combinator in 2012)",
 "hq": "San Francisco, California; fully remote, no offices",
 "backing": "Private. $1.2M seed in October 2012 (Bessemer, DFJ); profitable from 2014 with little outside capital; January 2021 secondary sale to Sequoia and Steadfast valued it at $5B (founders did not sell). Per Wikipedia, read 2026-10-10.",
 "tagline": "A hosted automation service that moves data between 9,000+ apps through trigger-and-action workflows, now with MCP, an SDK and AI agents.",
 "scope": "Zapier connects apps with Zaps: a trigger in one app (a new Xero invoice, a Stripe payment) fires actions in others. It reaches ledgers, banks, payments and documents, but keeps no record of its own beyond run logs held about 60 days. It does not reconcile, balance or check anything to the cent; it is plumbing, not a book.",
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
   "note": "Holds no copy of the books; the source keeps the record and Zapier keeps only run logs (up to 60 days, 10,000 runs) and its own config."
  },
  "locality": {
   "score": 0,
   "note": "Cloud only; no self-hosted or local edition."
  },
  "history": {
   "score": 1,
   "note": "Zap history guaranteed only 60 days; versions kept 1 month on Pro, 6 on Team, 1 year on Enterprise. Event-driven, nothing as-of."
  },
  "openness": {
   "score": 1,
   "note": "Closed platform with a public integration platform and SDK, but no bulk export of what flowed through it."
  },
  "breadth": {
   "score": 3,
   "note": "Reaches ledgers, banks, payments, documents and commerce across 9,000+ apps, but never holds them together in one place."
  },
  "autonomy": {
   "score": 4,
   "note": "Zaps create bills, invoices, payments and bank transactions in Xero unattended; approval is an optional step."
  },
  "agent_readiness": {
   "score": 4,
   "note": "Official MCP server, a TypeScript SDK and SDK CLI, app- and action-level controls, MCP actions logged in history."
  },
  "integration_ease": {
   "score": 4,
   "note": "The easiest on-ramp in the market: self-serve, templates, no code, a large catalogue and a documented integration platform."
  },
  "market": {
   "score": 2,
   "note": "Mostly small businesses and teams (3M+ businesses claimed) with a growing Enterprise tier."
  }
 },
 "api": {
  "public": true,
  "style": "Integration platform (CLI/UI) for app builders; Zapier MCP (remote MCP server); Zapier SDK (TypeScript, open beta) with raw .fetch() to about 3,600 app APIs; Powered by Zapier embed; Observability API on Enterprise",
  "auth": "Zapier account; SDK uses browser login, client credentials or direct token; app credentials held in Zapier's connection layer, outside the model",
  "limits": "Task quotas per plan; polling every 15 min (Free), 2 min (Pro), 1 min (Team, Enterprise); usage pauses at 3x the subscription even with pay-per-task",
  "cost": "Tasks are the meter: each successful action is a task; one MCP tool call is 2 tasks; overage 1.25x (annual) or 2.5x (monthly) the base rate. SDK free during open beta.",
  "export": "No bulk export of data that passed through; Zap history can be exported for longer records but is guaranteed only 60 days. Enterprise gets custom retention and log streaming to Datadog or Splunk.",
  "notes": "Zapier's developer surface has shifted from building integrations to letting agents call its catalogue (MCP, SDK). Governance of SDK raw fetch calls is not yet covered by org policies (per the SDK docs)."
 },
 "pricing": {
  "model": "Subscription by monthly task volume, per seat tier",
  "entry": "Free: $0, 100 tasks a month, two-step Zaps, 15-minute polling. Pro: $19.99 a month billed annually ($29.99 monthly) for 750 tasks.",
  "top": "Team: $69 a month annually ($103.50 monthly) for 2,000 tasks, up to 25 seats; tiers to 2,000,000 tasks. Enterprise: custom annual task limit.",
  "notes": "Prices from zapier.com/pricing, read 2026-10-10. Triggers, Formatter, Filter, Paths, Tables and Forms do not count as tasks; AI steps, code steps and MCP draw from the same pool."
 },
 "agents": {
  "mcp": "official",
  "ai_features": "Zapier MCP (9,000+ apps; 30,000+ or 66,000+ actions, the site gives both), Zapier Agents (build agents that act across apps), Copilot (build Zaps from natural language), AI by Zapier steps priced by model tier, Canvas, Tables, Interfaces.",
  "agent_writes": "Yes, directly: an MCP client or SDK script can create invoices, bills, payments and bank transactions in Xero or QuickBooks through Zapier actions, with whatever scope the connected account has.",
  "approval": "No approval step on MCP calls by default; controls are choosing which apps and actions are exposed, reviewing MCP actions in history afterwards, and Enterprise action restrictions. Human in the Loop by Zapier adds Request Approval and Collect Data steps inside a Zap, with timeouts. Approval requests for Enterprise admins are in beta; SDK agent-action permissions are experimental.",
  "notes": "Approval is a workflow step you add, not a property of the write path; nothing seals or records the exact package a person approved."
 },
 "integration": {
  "scope": "9,000+ apps. Xero: 18 triggers and 37 actions (25 writes, 10 searches), including Create Bank Transaction, Create Bill, Create Payment and Get Invoice History. QuickBooks Online, Stripe, banks via third-party apps.",
  "notes": "Triggers are mostly 'new' or 'updated' records, polled; a Zap sees an event once and moves on. Deletes, voids and silent edits are invisible unless the app emits them."
 },
 "preference": {
  "scale": "'Trusted by 3 million+ businesses' and 9,000+ apps (zapier.com, read 2026-10-10); about 730 employees in 2025 (Wikipedia). Revenue not disclosed by Zapier.",
  "ratings": "Trustpilot 1.3/5 from 322 reviews (read 2026-10-10). G2 and Capterra pages blocked automated reading; scores not confirmed.",
  "sentiment": "Praised for ease, breadth of apps and speed of setup. Complaints on Trustpilot centre on billing (auto-renewals, no refunds), task allowances used up within days, scripted support, and Zaps that fail silently or fire twice. Trustpilot reviewers skew to billing disputes, so the score understates day-to-day satisfaction."
 },
 "timeline": [
  {
   "date": "2011",
   "event": "Started as a side project in Columbia, Missouri by Wade Foster, Bryan Helmig and Mike Knoop"
  },
  {
   "date": "2012",
   "event": "Y Combinator; $1.2M seed in October (Bessemer, DFJ)"
  },
  {
   "date": "2014",
   "event": "Profitable"
  },
  {
   "date": "2021-01",
   "event": "Secondary sale to Sequoia and Steadfast at a $5B valuation"
  },
  {
   "date": "2021-03",
   "event": "Acquired Makerpad"
  },
  {
   "date": "2023",
   "event": "Zapier Canvas; Tables and Interfaces in the same period"
  },
  {
   "date": "2024-03",
   "event": "Acquired Vowel (AI video meetings)"
  },
  {
   "date": "2024-07",
   "event": "Acquired NoCodeOps"
  },
  {
   "date": "2025-10",
   "event": "Acquired Utopian Labs, which ceased operations the following month"
  },
  {
   "date": "2026-10",
   "event": "MCP, SDK (open beta) and Agents on every plan; MCP calls metered at 2 tasks each"
  }
 ],
 "relation": "neighbour",
 "relation_note": "Zapier is how most small businesses glue their ledger to everything else today. It moves events between systems and forgets them; ownpurse keeps every fact and moves nothing without approval. They meet where a person wants ownpurse's record to trigger, or be fed by, a Zap.",
 "borrow": "The catalogue shape: a trigger, an action and a search per object, named in plain words ('Find or Create Contact'). Metering only successful actions. Human in the Loop's request-approval step with a timeout and a reminder is a good model for how an approval request reaches a person.",
 "avoid": "Event-driven sync with no history: a Zap that missed a trigger, or a voided invoice that never fired, leaves no trace. Short run-log retention as the only audit. Approval as an optional step rather than the only path to a write. Metering agent reads as tasks.",
 "sources": [
  {
   "title": "Zapier pricing",
   "url": "https://zapier.com/pricing"
  },
  {
   "title": "Zapier MCP",
   "url": "https://zapier.com/mcp"
  },
  {
   "title": "Zapier SDK docs",
   "url": "https://docs.zapier.com/sdk"
  },
  {
   "title": "Zapier developer docs",
   "url": "https://docs.zapier.com/"
  },
  {
   "title": "View and manage your Zap history",
   "url": "https://help.zapier.com/hc/en-us/articles/8496291148685-View-and-manage-your-Zap-history"
  },
  {
   "title": "Xero integrations on Zapier",
   "url": "https://zapier.com/apps/xero/integrations"
  },
  {
   "title": "Human in the Loop by Zapier",
   "url": "https://zapier.com/apps/human-in-the-loop/integrations"
  },
  {
   "title": "Zapier Agents",
   "url": "https://zapier.com/agents"
  },
  {
   "title": "Zapier homepage",
   "url": "https://zapier.com/"
  },
  {
   "title": "Zapier on Wikipedia",
   "url": "https://en.wikipedia.org/wiki/Zapier"
  },
  {
   "title": "Zapier on Trustpilot",
   "url": "https://www.trustpilot.com/review/zapier.com"
  }
 ],
 "unverified": [
  "Zapier revenue or ARR: not disclosed; no figure confirmed",
  "G2 and Capterra ratings and review counts (pages blocked automated reading)",
  "Launch date of Zapier MCP (Wikipedia says 2026; earlier availability in 2025 not confirmed against a primary source)",
  "Whether Xero is a premium app on Zapier and the Xero trigger polling behaviour",
  "Plans on which Human in the Loop is available",
  "Zap export (as JSON) availability by plan",
  "Action count: the MCP page gives 66,000+ triggers and actions, its FAQ 30,000+ actions"
 ],
 "researched": "2026-10-10"
}
---

## What it is

Zapier is the hosted automation service most small businesses already use to glue apps together. A Zap is a trigger in one app (a new Stripe charge, a new Xero invoice) followed by actions in others. It claims [3 million+ businesses and 9,000+ apps](https://zapier.com/). It was started in 2011 by Wade Foster, Bryan Helmig and Mike Knoop, went through Y Combinator in 2012, and has been profitable since 2014 on very little outside money; a 2021 secondary sale valued it at $5B ([Wikipedia](https://en.wikipedia.org/wiki/Zapier)).

## How it works

Zaps poll or receive webhooks, then run steps. On accounting tools the catalogue is broad: [Xero has 18 triggers and 37 actions](https://zapier.com/apps/xero/integrations) on Zapier, including creating bank transactions, bills and payments. Zapier holds the connection credentials and runs the steps in its cloud. It keeps no copy of your books. What it keeps is run history, which it [guarantees for only 60 days and up to 10,000 runs](https://help.zapier.com/hc/en-us/articles/8496291148685-View-and-manage-your-Zap-history). Each event is seen once and passed on. A missed trigger, a silent edit or a voided invoice that never fires leaves no trace.

## Data and access

Zapier is metered by tasks. On [the pricing page](https://zapier.com/pricing) (read 2026-10-10), Free gives 100 tasks a month; Pro starts at $19.99 a month billed annually for 750; Team at $69 for 2,000 and up to 25 seats; Enterprise is a custom annual limit. Overage costs 1.25x the base rate on annual plans and 2.5x on monthly ones, and usage stops at three times the subscription. Triggers and built-in tools are free; actions, AI steps, code steps and MCP calls share one pool. There is no bulk export of what flowed through Zapier, because it was never meant to hold it.

## Agents

Zapier has moved fast here. [Zapier MCP](https://zapier.com/mcp) exposes its whole catalogue to Claude, ChatGPT, Cursor and other clients; each tool call costs 2 tasks. The [Zapier SDK](https://docs.zapier.com/sdk) (open beta, free for now) lets code call any action and make raw authenticated calls to about 3,600 app APIs, with a CLI for login and one-off runs. Zapier Agents and Copilot build and run workflows from plain language. That means an agent can create a Xero invoice through Zapier with no human in between. Controls are scoping (which apps and actions are exposed), after-the-fact review in history, and Enterprise action restrictions. [Human in the Loop by Zapier](https://zapier.com/apps/human-in-the-loop/integrations) adds a Request Approval step with a timeout, but you have to put it there; SDK permissions for approving agent actions are still experimental.

## Customers

Zapier is easy, and that is why it is everywhere: self-serve, templates, no code. Reviews on [Trustpilot](https://www.trustpilot.com/review/zapier.com) (1.3/5 from 322, read 2026-10-10) are dominated by billing disputes, task allowances used up within days, scripted support and Zaps that break or fire twice. G2 and Capterra scores could not be read for this dossier. Small businesses use it to stitch a ledger to a CRM, a payment processor and a spreadsheet. Larger companies use the Enterprise tier for governance, SSO and log streaming, often alongside a heavier iPaaS.

## History

Zapier is the survivor of the first wave of consumer-friendly integration (IFTTT, Integromat, Microsoft Flow). It solved the connector problem by building and maintaining thousands of connectors centrally, and it charges per event moved. It did not try to solve the data problem: there is no unified model, no record and no reconciliation. Its 2024–26 acquisitions (Vowel, NoCodeOps, Utopian Labs) and launches (Canvas, Tables, Agents, MCP) push it from plumbing toward an AI work platform.

## For ownpurse

Zapier is the incumbent answer to "how do my systems talk" for a small business, and the opposite of ownpurse's answer. It moves an event and forgets it; ownpurse keeps every fact, as of every moment, on your machine. It writes on a trigger; ownpurse writes only as a sealed package a person approved. They are complementary: a Zap could be a source (a form or a payment landing in the drawer) or a downstream consumer of an approved package. The gap Zapier leaves is exactly the record: when a Zap went wrong last March, nothing can tell you what the books looked like before it ran. Do not compete on catalogue size; Zapier's 9,000 connectors are not the contest. Borrow its plain naming of triggers, actions and searches, and its approval step with timeout and reminder.
