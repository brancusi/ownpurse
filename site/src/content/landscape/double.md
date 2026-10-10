---
{
 "slug": "double",
 "name": "Double (formerly Keeper)",
 "url": "https://doublehq.com",
 "segment": "agent-beside-ledger",
 "status": "active",
 "founded": "2020 (YC Summer 2021)",
 "hq": "New York, NY",
 "backing": "Y Combinator (Summer 2021). $6.5M Series A announced 10-11 December 2025 from Album Ventures, Jack Altman and Y Combinator.",
 "tagline": "Close and practice-management software for bookkeeping firms that syncs two ways with clients' QuickBooks Online and Xero and adds AI that drafts and proposes entries for review.",
 "scope": "Double runs the month-end close and the practice around it: close checklists, file reviews, bank-feed categorization, reconciliation, accruals and prepaids, AI journal entries from source files, a client portal, 1099s, receipts and a tax suite. It writes to the client's ledger when a user posts. It is not a ledger, and its AI ledger features are mostly US QuickBooks Online first.",
 "domains": [
  "close",
  "ledger",
  "bank",
  "documents",
  "tax",
  "reporting"
 ],
 "customers_served": [
  "accountants",
  "smb",
  "mid-market"
 ],
 "spectrum": {
  "custody": {
   "score": 1,
   "note": "Books stay in the client's QBO, Xero, NetSuite or Sage; Double holds workflow, review and activity data in its cloud."
  },
  "locality": {
   "score": 0,
   "note": "Hosted only."
  },
  "history": {
   "score": 1,
   "note": "An activity log attributes agent changes; no owner-held or as-of history."
  },
  "openness": {
   "score": 1,
   "note": "Closed product with no documented public REST API, but an official MCP server and Zapier."
  },
  "breadth": {
   "score": 3,
   "note": "Ledgers, its own bank feeds, receipts and source documents, client portal, tax suite and practice management."
  },
  "autonomy": {
   "score": 2,
   "note": "AI classifies, matches and drafts; a person approves and clicks Post. Scheduled AI agents (private beta) act on Double data."
  },
  "agent_readiness": {
   "score": 3,
   "note": "Official OAuth MCP server for Claude, ChatGPT, Gemini CLI and Copilot; per-tool allow, ask or block; ledger writes through MCP limited to comments."
  },
  "integration_ease": {
   "score": 3,
   "note": "Xero and QuickBooks app store listings, Zapier, MCP; a free trial after a demo."
  },
  "market": {
   "score": 2,
   "note": "Bookkeeping and accounting firms serving small businesses, plus internal finance teams on NetSuite and Sage Intacct."
  }
 },
 "api": {
  "public": false,
  "style": "No documented public REST API. MCP server at https://api.doublehq.com/mcp (and a dynamic tool-search variant for Copilot); Zapier.",
  "auth": "OAuth 2.0 sign-in with a Double account for MCP clients.",
  "limits": "Not published beyond Copilot's 70-tool cap; AI usage metered by credits.",
  "cost": "AI credits included per client per month ($1 Core, $5 Plus, $10 Scale), pooled at the firm; overage billed on the next invoice at an unpublished rate.",
  "export": "Not documented.",
  "notes": "About 113 Ask Double tools: 13 ledger tools (12 read-only, plus add_transaction_comment) and about 100 Double app tools to read, create, update or delete."
 },
 "pricing": {
  "model": "Per connected client per month for firms, unlimited users; flat annual plan for internal finance teams",
  "entry": "$10 per client per month (Core)",
  "top": "$50 per client per month (Scale); internal finance plan by quote",
  "notes": "Tiers from the September 2025 Scale launch: Core $10, Plus $25, Scale $50. The current pricing page shows the model but not the figures. Tax Suite included with a $200/month annual commitment; extra team emails $10 each a month. 14-day Scale trial per QBO client (May 2026). Billed in USD, CAD, AUD, GBP."
 },
 "agents": {
  "mcp": "official",
  "ai_features": "AI Bank Feeds (matches, rules, auto-classifications, needs-review, with confidence levels); AI Transactions (source documents to journal entries, bills, invoices and more); AI Fix This Rec; AI Flux Analysis; AI Financial Summaries; automated accruals and prepaids; Ask Double chat with connectors (Slack, Notion, HubSpot, Google, custom MCP); AI Agents on schedules or triggers (private beta, Sep 2026).",
  "agent_writes": "AI drafts ledger transactions, which a person posts. Through MCP and Ask Double, agents can write Double data; the only ledger write tool is add_transaction_comment.",
  "approval": "AI Transactions: 'nothing posts automatically'. Review the lines, approve with a thumbs-up, then Post. Ask Double tools set to Always allow, Always ask or Blocked (reads allow, writes ask by default). Agents log every change under their own name; no approval step during agent runs, read-and-report advised first.",
  "notes": "Agent guardrails: agents cannot create agents, recursive triggers are blocked, loops are monitored, agents pause when credits run out."
 },
 "integration": {
  "scope": "Two-way sync with QuickBooks Online and Xero for firms; NetSuite and Sage Intacct for internal finance teams; Zapier; Ping and Abacor for meeting notes.",
  "notes": "AI Bank Feeds support only US QuickBooks Online files ('Xero and non-US coming soon'). The Xero listing shows two-way sharing of invoices, contacts, bank transactions, journals and manual journals."
 },
 "preference": {
  "scale": "More than 4,000 firms run their close on Double, serving hundreds of thousands of businesses (company, December 2025). YC lists a team of 80.",
  "ratings": "Xero App Store: 5.0 from 11 reviews (2026-10-10). The homepage shows a G2 5/5 badge; count not confirmed (G2 and Capterra could not be fetched).",
  "sentiment": "Case studies cite a close 10 days faster (Springstead), 1,000 hours a month saved (Attivo) and 95% client-portal adoption (MCO Bookkeeping). App store reviews are uniformly positive. No complaints were found in the sources read."
 },
 "timeline": [
  {
   "date": "2020",
   "event": "Keeper founded by Ben Stein and Kenny Song"
  },
  {
   "date": "2021-06",
   "event": "Y Combinator Summer 2021 batch"
  },
  {
   "date": "2025-09-17",
   "event": "Scale tier launched ($50 per client): accruals and AI journal entries"
  },
  {
   "date": "2025-10-23",
   "event": "Keeper renamed Double after a lawsuit over the name"
  },
  {
   "date": "2025-11-10",
   "event": "'AI to empower, not replace' sets the review-first stance"
  },
  {
   "date": "2025-12-10",
   "event": "$6.5M Series A from Album Ventures, Jack Altman and Y Combinator; 4,000+ firms"
  },
  {
   "date": "2026-05-18",
   "event": "14-day free Scale AI trial per QBO client"
  },
  {
   "date": "2026-08-31",
   "event": "MCP server help article last updated"
  },
  {
   "date": "2026-09-04",
   "event": "AI Agents in private beta"
  }
 ],
 "relation": "neighbour",
 "relation_note": "A firm's workbench beside the client's ledger, with AI that drafts and a person who posts. It is closer to ownpurse's approval gate than Basis or Meridian, but hosted, firm-facing and without an owner-held record.",
 "borrow": "Per-tool permissions (Always allow, Always ask, Blocked) with reads allowed and writes asked by default; every agent change attributed to that agent in the log; approve, then post as two separate steps; agent guardrails (no agents creating agents, no recursive triggers, loop detection, pause when the budget runs out); confidence buckets (match, rule, auto, needs review) that explain why.",
 "avoid": "AI features that only work for US QuickBooks; a history that lives only in the vendor's activity log; metered AI with an unpublished overage rate.",
 "sources": [
  {
   "title": "Double home page",
   "url": "https://doublehq.com"
  },
  {
   "title": "Double pricing",
   "url": "https://doublehq.com/pricing"
  },
  {
   "title": "Double integrations",
   "url": "https://doublehq.com/integrations"
  },
  {
   "title": "Keeper is now Double (23 Oct 2025)",
   "url": "https://doublehq.com/blog/keeper-is-now-double/"
  },
  {
   "title": "Introducing Double Scale (17 Sep 2025)",
   "url": "https://doublehq.com/blog/introducing-keeper-scale/"
  },
  {
   "title": "AI to empower, not replace (10 Nov 2025)",
   "url": "https://doublehq.com/blog/ai-to-empower-not-replace/"
  },
  {
   "title": "A note on our Series A (11 Dec 2025)",
   "url": "https://doublehq.com/blog/a-note-on-our-series-a-and-the-future-were-building/"
  },
  {
   "title": "Business Wire: Double raises $6.5M Series A",
   "url": "https://www.businesswire.com/news/home/20251210027851/en/Double-Raises-%246.5-Million-Series-A-from-Album-Ventures-Jack-Altman-and-Y-Combinator-to-Double-Accountants-Productivity"
  },
  {
   "title": "Taking the friction out of trying Double's AI (18 May 2026)",
   "url": "https://doublehq.com/blog/were-taking-the-friction-out-of-trying-doubles-ai/"
  },
  {
   "title": "Double's MCP server (help)",
   "url": "https://help.doublehq.com/en/articles/14301574-double-s-mcp-server"
  },
  {
   "title": "Ask Double tool reference",
   "url": "https://help.doublehq.com/en/articles/15656647-ask-double-tool-reference"
  },
  {
   "title": "Connectors in Ask Double",
   "url": "https://help.doublehq.com/en/articles/16143262-connectors-in-ask-double"
  },
  {
   "title": "AI Agents (help, 4 Sep 2026)",
   "url": "https://help.doublehq.com/en/articles/16631377-ai-agents"
  },
  {
   "title": "AI Transactions (help)",
   "url": "https://help.doublehq.com/en/articles/12169300-ai-transactions"
  },
  {
   "title": "AI Bank Feeds (help)",
   "url": "https://help.doublehq.com/en/articles/11814308-ai-bank-feeds"
  },
  {
   "title": "AI credits (help)",
   "url": "https://help.doublehq.com/en/articles/15959965-ai-credits-usage-budgets-and-adding-credits"
  },
  {
   "title": "Double on the Xero App Store",
   "url": "https://apps.xero.com/us/app/double"
  },
  {
   "title": "Y Combinator: Keeper",
   "url": "https://www.ycombinator.com/companies/keeper"
  }
 ],
 "unverified": [
  "Current per-client prices: $10/$25/$50 come from the September 2025 launch post; the current pricing page did not show figures",
  "Who sued Keeper over the name",
  "G2 and Capterra ratings and review counts (pages blocked; the homepage shows a G2 5/5 badge)",
  "QuickBooks App Store rating (page did not render)",
  "AI credit overage rate",
  "Whether AI Bank Feeds can post without a per-item approval (the May 2026 post says 'auto-classify and post'; the AI Transactions help page says nothing posts automatically)",
  "Whether a public REST API exists beyond MCP and Zapier"
 ],
 "researched": "2026-10-10"
}
---

## What it is

Double is close and practice-management software for bookkeeping and accounting firms. Until 23 October 2025 it was called Keeper. Ben Stein says the company [renamed after being sued](https://doublehq.com/blog/keeper-is-now-double/) rather than spend resources on the dispute. It covers the month-end close, file reviews, a client portal, 1099s, receipts, accruals and a tax suite, all syncing both ways with the client's QuickBooks Online or Xero. Internal finance teams can also connect NetSuite and Sage Intacct.

## How it works

A firm imports each client's ledger. Double runs its own bank feeds rather than reading QuickBooks's. [AI Bank Feeds](https://help.doublehq.com/en/articles/11814308-ai-bank-feeds) sorts each line into matches, rules, auto-classifications or needs-review, with a high or medium confidence level and, for new vendors, a web lookup. [AI Transactions](https://help.doublehq.com/en/articles/12169300-ai-transactions) reads a source document and drafts journal entries, bills, invoices or deposits, asking questions first. The help page is explicit: 'nothing posts automatically.' You review the lines, approve with a thumbs-up, then press Post. Accruals and prepaids post with one click from the close page. The [Scale tier](https://doublehq.com/blog/introducing-keeper-scale/) adds AI flux analysis, reconciliation fixes and financial summaries.

## Data and access

The books stay in the client's ledger. The [Xero listing](https://apps.xero.com/us/app/double) shows two-way sharing of invoices, contacts, bank transactions and journals, with 11 reviews averaging 5.0. Pricing is per connected client per month with unlimited users: Core $10, Plus $25 and Scale $50 at the September 2025 launch. AI credits worth $1, $5 or $10 per client are pooled across the firm, with overage billed on the next invoice at a rate the help article does not give ([AI credits](https://help.doublehq.com/en/articles/15959965-ai-credits-usage-budgets-and-adding-credits)). There is no documented public API and no documented export.

## Agents

Double has an official [MCP server](https://help.doublehq.com/en/articles/14301574-double-s-mcp-server) at api.doublehq.com/mcp, with OAuth, for Claude, ChatGPT, Gemini CLI and Copilot. Its [tool reference](https://help.doublehq.com/en/articles/15656647-ask-double-tool-reference) lists about 113 tools. Twelve ledger tools are read-only, and the one ledger write is adding a comment. The rest act on Double's own data. [Ask Double connectors](https://help.doublehq.com/en/articles/16143262-connectors-in-ask-double) set each tool to Always allow, Always ask or Blocked, and recommend allowing reads and asking for writes. [AI Agents](https://help.doublehq.com/en/articles/16631377-ai-agents), in private beta since September 2026, run on schedules or events. Each change is logged under the agent's name, agents cannot create agents, recursive triggers are blocked, and agents pause when credits run out.

## Customers

Double says more than 4,000 firms run their close on it, serving hundreds of thousands of businesses ([Series A note, December 2025](https://doublehq.com/blog/a-note-on-our-series-a-and-the-future-were-building/)). Case studies cite a close 10 days faster and 1,000 hours a month saved. Its stance is 'sharper knives for professional chefs': AI gets accountants to the one-yard line and a person reviews, and Double says it is not building an AI accounting firm ([November 2025](https://doublehq.com/blog/ai-to-empower-not-replace/)).

## History

Ben Stein (CEO) and Kenny Song (CTO) founded Keeper in New York in 2020 and went through [YC in Summer 2021](https://www.ycombinator.com/companies/keeper). The company raised a modest $6.5M Series A in December 2025. It added the Scale AI tier in September 2025, a free AI trial in May 2026 and agents in September 2026.

## For ownpurse

Double is the most careful about writes in this segment. Approving and posting are separate steps, write tools ask by default, and every agent change carries the agent's name. That is close to ownpurse's sealed-package idea, run as a hosted firm tool. The owner still holds no record, and the history is Double's activity log. Worth borrowing directly: the allow, ask or block permission model and the agent guardrails. The gap is the same as with the others: nothing here is held, verifiable or portable by the business that owns the books.
