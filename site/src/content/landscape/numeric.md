---
{
 "slug": "numeric",
 "name": "Numeric",
 "url": "https://www.numeric.io",
 "segment": "close-analytics",
 "status": "active",
 "founded": "2020",
 "hq": "San Francisco, California, US",
 "backing": "Venture-backed, private. $10M seed (about May 2024, per Numeric's Series A post: 'five months after' the seed); $28M Series A led by Menlo Ventures (10 October 2024, with IVP, Socii, Founders Fund, 8VC); $51M Series B led by IVP (19 November 2025). Press puts the total at about $89M.",
 "tagline": "A close management and finance data platform that keeps a synced copy of your ledger, reconciles it and drafts flux analysis, and now offers itself as a NetSuite replacement.",
 "scope": "Numeric connects to NetSuite, QuickBooks Online, Xero and Sage Intacct, pulls the trial balance and transaction lines into its own cloud, and runs the close on that copy: checklists, reconciliations, transaction monitors, flux and custom reports. Since late 2025 it also sells cash matching, billing and revenue recognition, and an ERP replacement it calls an agent-first system of record. It does not keep an owner-held copy, and it has no documented public REST API; agents reach it through a remote MCP server.",
 "domains": [
  "close",
  "reporting",
  "ledger",
  "bank",
  "invoicing"
 ],
 "customers_served": [
  "mid-market",
  "enterprise"
 ],
 "spectrum": {
  "custody": {
   "score": 1,
   "note": "The ERP keeps the books, but the synced copy, sign-offs and review history live only in Numeric's cloud; for ERP-replacement customers Numeric is the ledger."
  },
  "locality": {
   "score": 0,
   "note": "Cloud-only SaaS; no self-hosted or local option."
  },
  "history": {
   "score": 2,
   "note": "Timestamped task events, comments and submit/unsubmit history, plus period-over-period reports; no documented as-of query over the synced ledger."
  },
  "openness": {
   "score": 1,
   "note": "Closed source, no public REST API found; access is the app, report exports and the MCP server."
  },
  "breadth": {
   "score": 3,
   "note": "GL from four ERPs, 17+ named bank feeds, Stripe, Brex, Ramp, Mercury, HubSpot, Salesforce, Snowflake and Databricks, plus billing and revenue."
  },
  "autonomy": {
   "score": 3,
   "note": "Auto-submits reconciliations that tie out, batch-posts cash journals to NetSuite, and the ERP replacement derives journals from source records; reviewers handle exceptions."
  },
  "agent_readiness": {
   "score": 3,
   "note": "Official remote MCP (api.numeric.io/mcp) with 22 tools, including writes to tasks and flux explanations; no CLI or public REST API."
  },
  "integration_ease": {
   "score": 2,
   "note": "Self-serve ERP connections (Sage Intacct in under fifteen minutes, by Numeric's account); a third party cannot build on it except through MCP as a user."
  },
  "market": {
   "score": 3,
   "note": "Venture-backed mid-market and late-stage companies (OpenAI, Brex, Plaid, Wealthfront); 75% of G2 reviewers mid-market."
  }
 },
 "api": {
  "public": false,
  "style": "No public REST API documented; a remote MCP server over HTTP (https://api.numeric.io/mcp) is the programmatic surface.",
  "auth": "MCP clients log in with the user's Numeric account; the agent acts with that user's permissions.",
  "limits": "Not published.",
  "cost": "Not separately priced; MCP is part of the workspace. Overall pricing is quote-based by transaction volume and seats.",
  "export": "Reports can be built and exported (by Numeric's MCP use cases); no documented bulk export of the synced ledger or audit history.",
  "notes": "MCP tools include query_transaction_lines, get_report_data, build_report, list_tasks, get_task_events, plus writes: create_task, edit_task, assign_task, submit_task (submit, approve or return), add_task_comment, update_flux_explanation."
 },
 "pricing": {
  "model": "Custom quote, by product module, transaction volume and seat count; onboarding and migration scoped upfront.",
  "entry": "Not published on numeric.io. A third-party directory reports an Essentials tier from $30 per user per month (unverified).",
  "top": "ERP replacement (full Financial Data Platform), custom quote.",
  "notes": "Four packages on the pricing page (viewed 10 October 2026): ERP Replacement, Close Management and Analytics, Cash Management, Billing and Revenue."
 },
 "agents": {
  "mcp": "official",
  "ai_features": "Prism (in-app AI surfacing daily agent items for review), AI-drafted flux explanations (Numeric says 80% of first drafts), AI-suggested cash matches and an AI rule builder, transaction monitors, Agent Builder for background tasks, App Studio (apps from a prompt), agentic subledgers.",
  "agent_writes": "Through MCP an agent can create, edit, assign and submit or approve close tasks, add comments and update flux explanations. In-app agents post cash journals to NetSuite and, in the ERP replacement, derive journal entries from source records.",
  "approval": "Review is task-based: preparers submit, reviewers approve or return, control reviews can be added, and auto-submit applies when a reconciliation ties out. MCP writes run as the user; the Claude workspace admin decides whether write tools are enabled. Bulk operations can be previewed before they run.",
  "notes": "MCP launched 20 April 2026. Same-as-user permissions mean an agent can approve what the user can approve; there is no separate sealed-package gate."
 },
 "integration": {
  "scope": "ERPs: NetSuite (deepest, transaction-line detail), QuickBooks Online, Xero, Sage Intacct (certified API integration). Banks: 17+ named (Chase, J.P. Morgan, Bank of America, Wells Fargo, SVB, PNC, HSBC US and others), plus Brex, Ramp, Mercury, Stripe, HubSpot, Salesforce, Snowflake, Databricks, Anrok.",
  "notes": "Trial balances refresh in near real time by Numeric's account. Journal-entry write-back is reported to be NetSuite-only (third-party review, unverified)."
 },
 "preference": {
  "scale": "No customer count published. ARR grew 4x in the year before October 2024 (Numeric). Named customers include OpenAI, Brex, Plaid, Wealthfront, Asana, ClickUp, Mercury, GOAT.",
  "ratings": "G2: 4.8/5 from 48 reviews on a G2 comparison page (2026); an aggregator cites 4.8 from 65 reviews. Numeric reports G2 Winter 2024 awards for best estimated ROI in close management.",
  "sentiment": "Reviewers praise ease of use (G2 9.6) and support (10.0 on a comparison page) and fast setup. Third-party write-ups say it is strongest on close and reporting, thinner on complex global consolidation and tax, and that integration depth beyond NetSuite should be checked with a reference customer."
 },
 "timeline": [
  {
   "date": "2020",
   "event": "Founded in San Francisco by Parker Gilbert (formerly VP finance at Hearth) with co-founders Anthony and Andrew, to build AI close management."
  },
  {
   "date": "2024-05",
   "event": "$10M seed round (per Numeric's Series A announcement)."
  },
  {
   "date": "2024-10-10",
   "event": "$28M Series A led by Menlo Ventures; ARR up 4x year on year."
  },
  {
   "date": "2025-11-19",
   "event": "$51M Series B led by IVP; launches cash management with a claimed 90%+ auto-match rate; plans to become a connected platform for finance operations by 2027."
  },
  {
   "date": "2026-04-20",
   "event": "Numeric MCP server live, with read and write tools for Claude, ChatGPT and other clients."
  },
  {
   "date": "2026",
   "event": "Pricing page lists an ERP Replacement package positioned against NetSuite."
  }
 ],
 "relation": "neighbour",
 "relation_note": "Numeric is the nearest architecture: it syncs a copy of the ledger, checks it to the cent and explains the variances. The difference is who holds the copy. Numeric's copy, its review history and its sign-offs live in its cloud, and it is moving to become the ledger itself; ownpurse keeps the copy, every version and the approvals in a file the owner holds, and leaves the books where they are.",
 "borrow": "Tie-out as a state: a reconciliation that ties auto-submits, one that drifts re-opens. Transaction monitors that flag anomalies before close. Flux drafts written for a reviewer to edit, not to post. An MCP tool set shaped around the close (list_tasks, query_transaction_lines, get_report_row_trend, update_flux_explanation) is a good template for ownpurse's agent surface.",
 "avoid": "Letting an agent act with the full permissions of the person who connected it, so it can approve what it prepared. Keeping the synced ledger and audit history only on the vendor's side with no bulk export. Drifting from checking the ledger to replacing it.",
 "sources": [
  {
   "title": "Numeric home page",
   "url": "https://www.numeric.io"
  },
  {
   "title": "Numeric pricing",
   "url": "https://www.numeric.io/pricing"
  },
  {
   "title": "Numeric close management software",
   "url": "https://www.numeric.io/solutions/close-management-software"
  },
  {
   "title": "Numeric integrations",
   "url": "https://www.numeric.io/integrations"
  },
  {
   "title": "Numeric's MCP server: how to connect and 11 key use cases (20 April 2026)",
   "url": "https://numeric.io/blog/numeric-mcp-server"
  },
  {
   "title": "Numeric MCP server help article",
   "url": "https://help.numeric.io/articles/7292808089-numeric-mcp-server"
  },
  {
   "title": "Numeric raises $51M Series B, led by IVP (19 November 2025)",
   "url": "https://numeric.io/blog/numeric-raises-51m-series-b"
  },
  {
   "title": "Numeric raises $28M Series A (10 October 2024)",
   "url": "https://www.numeric.io/blog/numerics-raises-28m"
  },
  {
   "title": "CPA Practice Advisor: Numeric raises $51M Series B",
   "url": "https://www.cpapracticeadvisor.com/2025/11/20/ai-accounting-platform-numeric-raises-51m-series-b/173638/"
  },
  {
   "title": "Numeric announces Sage Intacct integration",
   "url": "https://numeric.io/blog/numeric-announces-sage-intacct-integration"
  },
  {
   "title": "ERP Research: Numeric profile (updated August 2026)",
   "url": "https://erpresearch.com/erp-add-ons/financial-close/numeric"
  },
  {
   "title": "G2: Keeper vs Numeric comparison",
   "url": "https://www.g2.com/compare/keeper-vs-numeric"
  }
 ],
 "unverified": [
  "Essentials tier from $30 per user per month (third-party directory; not on Numeric's pricing page).",
  "Seed round date and amount ($10M, about May 2024) from aggregators and Numeric's 'five months after' wording; some sources label rounds differently.",
  "Total funding of about $89M (press sum; one source says $92.5M).",
  "Journal-entry write-back limited to NetSuite (third-party review).",
  "Refresh every 1-2 minutes (third-party).",
  "G2 review count varies between 48 and 65 by source; ratings could not be read directly from G2 (blocked).",
  "Co-founders' surnames not confirmed from a primary source."
 ],
 "researched": "2026-10-10"
}
---

## What it is

Numeric is a finance data platform for accounting teams, founded in 2020 in San Francisco by Parker Gilbert, a former VP of finance at Hearth, with two co-founders. It started as AI close management sitting beside the ERP: connect [NetSuite, QuickBooks Online, Xero or Sage Intacct](https://www.numeric.io/solutions/close-management-software), pull in the trial balance and transaction lines, and run the month-end close on that copy. Since its November 2025 Series B it has widened into [cash management, billing and revenue, and an ERP replacement](https://www.numeric.io/pricing) it calls "the agent-first system of record."

## How it works

Numeric keeps a near-real-time synced copy of your general ledger in its cloud. On that copy it runs a close checklist with business-day due dates, balance sheet reconciliations, transaction monitors that flag anomalies before close, and custom and flux reports. When a reconciliation ties to its workpaper, you can let it auto-submit; reviewers add control reviews and review notes, and every comment, submission and attachment is logged with a timestamp. Auditors get read-only licences. Flux explanations are drafted by AI for a person to edit, and Numeric says 80% of first drafts are now written that way. Cash management matches bank lines to the ledger (Numeric claims 90%+ auto-match) and posts routine cash entries back to NetSuite.

## Data and access

There is no documented public REST API. The programmatic surface is a remote [MCP server](https://numeric.io/blog/numeric-mcp-server) at `api.numeric.io/mcp`, live since 20 April 2026, with 22 tools: reading reports, tasks, task events, accounts and transaction lines, and writing tasks, assignments, submissions and flux explanations. The agent runs with the same access as the user who logged in. Pricing is quote-based by module, transaction volume and seats; no prices are published on numeric.io as of 10 October 2026. A directory reports an entry tier at $30 per user per month, which Numeric's own page does not confirm. Report export exists; a bulk export of the synced ledger or the audit history is not documented.

## Agents

Numeric is one of the few close vendors with an official MCP server, and it allows writes. An agent can create and assign close tasks, submit or approve them, and update flux commentary. Whether write tools are on is decided in the AI client's admin settings, and bulk changes can be previewed. In-app, Prism surfaces "daily agent items requiring review," and Agent Builder runs background tasks such as chasing overdue invoices. There is no separate approval object: an agent can approve what its user can approve.

## Customers

Numeric names OpenAI, Brex, Plaid, Wealthfront, Asana, ClickUp, Mercury and GOAT. It publishes no customer count; in October 2024 it said ARR had grown 4x in a year. On G2 it scores 4.8 of 5 from 48 reviews on a comparison page, with 75% of reviewers mid-market, and high marks for ease of use and support. Independent write-ups say it is strongest on close and reporting and lighter on complex global consolidation, and that integration depth beyond NetSuite (journal write-back in particular) should be checked.

## History

A $10M seed in 2024 was followed five months later by a [$28M Series A led by Menlo Ventures](https://www.numeric.io/blog/numerics-raises-28m) on 10 October 2024, and a [$51M Series B led by IVP](https://numeric.io/blog/numeric-raises-51m-series-b) on 19 November 2025, which also brought in a former BlackLine CEO as an angel. The stated plan is to become a connected platform for finance operations by 2027, adding one automated workflow at a time.

## For ownpurse

Numeric is the closest thing to ownpurse's architecture in the market: a copy of the ledger, synced and checked to the cent, with variances explained and a human signing off. It does that well, and its MCP tool set is a useful template. The difference is custody and direction. Numeric's copy, its event history and its sign-offs live in Numeric's cloud, the agent acts with its user's full rights, and the company is moving from checking the ledger to replacing it. ownpurse holds the same copy in a file the owner keeps, keeps every version, and turns an agent's proposal into a sealed package that someone else approves. Numeric sells to venture-backed mid-market finance teams; ownpurse's open ground is the owner, the small business and the accountant who want the same checks without handing over the record.
