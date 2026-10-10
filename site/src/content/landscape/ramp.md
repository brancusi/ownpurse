---
{
 "slug": "ramp",
 "name": "Ramp",
 "url": "https://ramp.com",
 "segment": "spend-management",
 "status": "active",
 "founded": "March 2019",
 "hq": "New York City, US",
 "backing": "Private, venture-backed. Over $3 billion in equity raised; last round $750 million at a $44 billion valuation on 4 June 2026, led by ICONIQ, GIC and Ontario Teachers' Pension Plan (earlier: $32 billion in November 2025, $22.5 billion in July 2025, $16 billion in June 2025).",
 "tagline": "Corporate cards, bill pay, procurement and travel with an accounting agent that codes spend and syncs it into your ERP.",
 "scope": "Ramp issues corporate cards and runs expenses, reimbursements, bill pay, procurement, travel and treasury, then codes each transaction and syncs it to QuickBooks Online, Xero, NetSuite, Sage Intacct and larger ERPs. Its Accounting Agent (February 2026) codes, accrues, reconciles and can post low-risk items to the ledger unattended; Ramp Stack (June 2026) extends that to accounting firms. It is not a general ledger: the books stay in the ERP, and the spend record stays in Ramp.",
 "domains": [
  "spend",
  "payments",
  "bank",
  "documents",
  "close",
  "integration"
 ],
 "customers_served": [
  "smb",
  "mid-market",
  "enterprise",
  "accountants",
  "developers"
 ],
 "spectrum": {
  "custody": {
   "score": 1,
   "note": "Ramp holds the spend record; coded transactions are copied into your ERP and the API exports the rest, but the authoritative spend history is Ramp's."
  },
  "locality": {
   "score": 0,
   "note": "Cloud only; the one local piece, the archived ramp_mcp, just pulled data into an ephemeral in-memory SQLite."
  },
  "history": {
   "score": 1,
   "note": "Transactions carry status, comments and sync results, but nothing found offers append-only versions or as-of queries."
  },
  "openness": {
   "score": 2,
   "note": "Closed product with a documented public API, an OpenAPI spec and an MIT-licensed CLI."
  },
  "breadth": {
   "score": 3,
   "note": "Cards, bills, reimbursements, procurement, travel, treasury and ERP sync in one place; no ledger, payroll or tax of its own."
  },
  "autonomy": {
   "score": 3,
   "note": "The Accounting Agent can post low-risk transactions to the ERP automatically once checks pass, within limits the customer sets."
  },
  "agent_readiness": {
   "score": 4,
   "note": "Hosted MCP at mcp.ramp.com with write tools and a demo endpoint, an agent-mode CLI with JSON output and dry runs, scoped OAuth, llms.txt docs."
  },
  "integration_ease": {
   "score": 3,
   "note": "Self-serve OAuth API with scopes and clear limits; sandbox, some endpoints and data partnerships need a request or Ramp Plus."
  },
  "market": {
   "score": 3,
   "note": "Startups through enterprise (3,200+ customers paying $100,000+ a year as of June 2026), plus accounting firms through Stack."
  }
 },
 "api": {
  "public": true,
  "style": "REST with JSON, OpenAPI spec at docs.ramp.com/openapi/developer-api.json; hosted MCP server; open-source CLI",
  "auth": "OAuth 2.0: authorization code for acting as a user, client credentials for standalone agents; tokens bound to scopes, and a token requested without scopes has none",
  "limits": "200 requests per 10-second rolling window per source IP; requests over 60 seconds return 504; increases by support ticket",
  "cost": "No API fee found in the docs or on the pricing page; purchase-order endpoints and Custom Records need Ramp Plus, and Ramp Data access goes through a partner program",
  "export": "API reads of transactions, bills, reimbursements, vendors, users, entities and more; CSV exports and custom reports in the app",
  "notes": "Accounting endpoints include beta codings, ready-to-sync marking, Ramp-only custom accounting fields and sync result reporting (for example REPAYMENT_SYNC, INVOICE, TRANSFER_LEDGER_ENTRY_MAPPING). Sandbox at demo-api.ramp.com is by request."
 },
 "pricing": {
  "model": "Per-user SaaS tiers; card interchange is the main revenue source",
  "entry": "Free: $0 per user per month, with QuickBooks Online and Xero sync and basic rules",
  "top": "Enterprise: custom, annual billing, adds Workday, Oracle Fusion Cloud and Dynamics F&O",
  "notes": "Plus is $15 per user per month plus a platform fee by team size (20% off annual) and adds NetSuite, Sage Intacct, Acumatica, Business Central, AI coding for every field and multi-entity. Checked on ramp.com/pricing, 10 October 2026."
 },
 "agents": {
  "mcp": "official",
  "ai_features": "Accounting Agent (coding, line-item splits, accruals and reversals, real-time reconciliation against the ERP, learning from corrections in plain language); policy, AP and procurement agents; AI token spend management; Ramp Stack for accounting firms; agentic payments with Visa.",
  "agent_writes": "Yes. Hosted MCP tools edit transactions, submit reimbursements, post comments and manage cards; the CLI covers accounting, bills, cards, receipts, reimbursements, transactions and travel, acting as the authenticated user or an agent client.",
  "approval": "Customers choose which actions the Accounting Agent may take (code, review, sync); low-risk items can post without review. Admins can restrict MCP by role, department or user. The CLI has --dry_run. Secondary coverage says Stack requires accountant approval before anything posts; Ramp's own release says only that every decision is reviewable.",
  "notes": "The first, local ramp_mcp (March 2025, read scopes only, SQL over loaded data) was archived on 17 July 2026 in favour of the hosted server."
 },
 "integration": {
  "scope": "QuickBooks Online, Xero, NetSuite, Sage Intacct, Acumatica, Microsoft Dynamics 365 Business Central, Workday, Oracle Fusion Cloud, Dynamics F&O, plus HRIS, travel and other apps",
  "notes": "Two-way ERP sync is the core of the accounting product; an ERP migration tool sets up a new API connection when a customer changes ledgers."
 },
 "preference": {
  "scale": "70,000+ customers, over $1 billion annualized revenue, positive free cash flow and $200 billion annualized purchase volume (GIC release, 4 June 2026). Ramp's Stack release a day earlier said 50,000+ customers and over $100 billion annual purchase volume.",
  "ratings": "Trustpilot 2.3 of 5 from 214 reviews (October 2026). G2 and Capterra pages could not be read.",
  "sentiment": "Reviewers praise the clean design, ease of managing cards and budgets, and single-use cards. Complaints centre on support that is hard to reach, slow vendor payments, a 1% fee for faster payouts, a QuickBooks Online sync that failed for one reviewer, and persistent sales outreach."
 },
 "timeline": [
  {
   "date": "2019-03",
   "event": "Founded in New York by Eric Glyman, Karim Atiyeh and Gene Lee (Glyman and Atiyeh had sold Paribus to Capital One in 2016)"
  },
  {
   "date": "2020-02",
   "event": "Corporate card launches with a Series A led by Founders Fund"
  },
  {
   "date": "2021-08",
   "event": "$300 million Series C at about $3.9 billion"
  },
  {
   "date": "2023-08",
   "event": "$300 million Series D at $5.8 billion, down from an $8.1 billion mark in 2022; procurement announced"
  },
  {
   "date": "2025-03",
   "event": "Local, read-only ramp_mcp published"
  },
  {
   "date": "2025-08",
   "event": "$1 billion annualized revenue reported"
  },
  {
   "date": "2025-11",
   "event": "$300 million round at $32 billion led by Lightspeed"
  },
  {
   "date": "2026-02-16",
   "event": "Accounting Agent launched: codes, accrues, reconciles and can auto-post low-risk items"
  },
  {
   "date": "2026-06-03",
   "event": "Ramp Stack launched for accounting firms"
  },
  {
   "date": "2026-06-04",
   "event": "$750 million at $44 billion; Billhop (UK and EU payments) and Juno acquired; UK and Europe launch planned"
  },
  {
   "date": "2026-07-17",
   "event": "Local ramp_mcp archived; hosted MCP at mcp.ramp.com and the Ramp CLI replace it"
  }
 ],
 "relation": "neighbour",
 "relation_note": "Ramp is both a source and a neighbour. Its card lines, bills and receipts are facts ownpurse would ingest, and its Accounting Agent is the closest commercial version of an agent working beside the ledger, except that it writes to the ledger itself and keeps the record in Ramp's cloud.",
 "borrow": "The automation dial: the owner picks which actions the agent may take (code, review, sync) rather than a single on/off. Accrue-and-reverse for items not ready to post, which is the uncollapsed state in practice. A CLI with an --agent JSON mode, --dry_run and named human and agent profiles. A demo MCP endpoint over sample data.",
 "avoid": "Auto-posting to the ledger on the vendor's confidence score. Keeping the only full history of coding decisions inside the vendor. Headline figures that differ between two releases from the same week.",
 "sources": [
  {
   "title": "GIC: Ramp raises Series F at $44 billion valuation (4 June 2026)",
   "url": "https://www.gic.com.sg/newsroom/all/ramp-raises-series-f-at-44-billion-valuation/"
  },
  {
   "title": "Ramp pricing",
   "url": "https://ramp.com/pricing"
  },
  {
   "title": "Ramp developer API docs",
   "url": "https://docs.ramp.com/developer-api/v1/overview/introduction"
  },
  {
   "title": "Ramp docs guides (llms-guides.txt): MCP, rate limits, CLI",
   "url": "https://docs.ramp.com/llms-guides.txt"
  },
  {
   "title": "Ramp CLI on GitHub",
   "url": "https://github.com/ramp-public/ramp-cli"
  },
  {
   "title": "ramp_mcp on GitHub (archived 17 July 2026)",
   "url": "https://github.com/ramp-public/ramp_mcp"
  },
  {
   "title": "CFOtech: Ramp unveils AI accounting agent (16 February 2026)",
   "url": "https://cfotech.asia/story/ramp-unveils-ai-accounting-agent-to-speed-month-end-close"
  },
  {
   "title": "PR Newswire: Ramp launches Stack (3 June 2026)",
   "url": "https://www.prnewswire.com/news-releases/ramp-launches-stack-an-ai-operating-system-for-accounting-firms-302789630.html"
  },
  {
   "title": "Ramp Stack explained (secondary, on human approval)",
   "url": "https://ustechautomations.com/resources/blog/ramp-stack-explained-what-it-changes"
  },
  {
   "title": "Wikipedia: Ramp (company)",
   "url": "https://en.wikipedia.org/wiki/Ramp_(company)"
  },
  {
   "title": "Trustpilot: Ramp",
   "url": "https://www.trustpilot.com/review/ramp.com"
  }
 ],
 "unverified": [
  "Accounting Agent performance figures (98% accuracy on ready-to-sync items, 3.5x more auto-coded, 3x faster close) are Ramp's own claims, reported by press; Ramp's primary announcement was not read",
  "That Stack requires human approval before any GL posting comes from secondary coverage; Ramp's release does not describe the approval step",
  "Customer count differs between sources a day apart: 70,000+ (GIC, 4 June 2026) and 50,000+ (Ramp Stack release, 3 June 2026)",
  "G2 and Capterra ratings could not be retrieved",
  "Exact Accounting Agent launch date: reported as 12 February 2026 by one source and 16 February 2026 by CFOtech",
  "Whether API access carries any fee or plan restriction beyond the endpoints noted"
 ],
 "researched": "2026-10-10"
}
---

## What it is

Ramp is a spend management company: corporate cards, expenses, reimbursements, bill pay, procurement, travel and treasury, with the accounting work done in the same place. It does not keep your books. It codes each transaction and syncs it into the ledger you already use, from QuickBooks Online and Xero up to NetSuite, Workday and Oracle Fusion ([pricing](https://ramp.com/pricing)).

In February 2026 Ramp launched an Accounting Agent that assigns GL codes, departments and classes as spend happens, splits bills by line, accrues items not yet ready and reverses them the next month, reconciles against the ERP and flags missing or duplicate lines. Low-risk transactions can post automatically once checks pass ([CFOtech, 16 February 2026](https://cfotech.asia/story/ramp-unveils-ai-accounting-agent-to-speed-month-end-close)). Ramp's own figures for it: 98% accuracy on items marked ready to sync, close 3x faster. Those are the company's numbers, not independently checked.

## How it works

Ramp earns mostly on card interchange, so software is cheap: Free at $0 per user, Plus at $15 per user per month plus a platform fee, Enterprise on contract. You choose what the agent may do (code, review, sync), and anything uncertain goes to a person. In June 2026 Ramp launched [Stack](https://www.prnewswire.com/news-releases/ramp-launches-stack-an-ai-operating-system-for-accounting-firms-302789630.html), aimed at accounting firms: close, bank reconciliation, coding and journal posting across their clients, with the firm's procedures captured as SOPs. Secondary coverage says nothing posts in Stack until an accountant approves it; Ramp's release says only that every decision is reviewable.

## Data and access

The [developer API](https://docs.ramp.com/developer-api/v1/overview/introduction) is REST with an OpenAPI spec and OAuth 2.0. Tokens are bound to scopes. The limit is 200 requests per 10 seconds per IP, and a request over 60 seconds is cut off. There are beta accounting endpoints to post codings, mark objects ready to sync and report sync results. No API fee is published, though some endpoints need Plus and the sandbox is by request. You can pull your transactions, bills and vendors in bulk, but the full record of who coded what, and why, lives in Ramp.

## Agents

Ramp is the most agent-ready company in this segment. It runs a hosted MCP server at `mcp.ramp.com` with write tools (edit transactions, submit reimbursements, manage cards), a demo endpoint over sample data, and admin controls by role and department. The [Ramp CLI](https://github.com/ramp-public/ramp-cli) is MIT-licensed, logs in by OAuth as a person or as an agent, gives JSON with `--agent`, and has `--dry_run`. The first [local MCP](https://github.com/ramp-public/ramp_mcp) was read-only and was archived on 17 July 2026. Ramp is also working with Visa on agents that make corporate payments.

## Customers

On 4 June 2026 [GIC's release](https://www.gic.com.sg/newsroom/all/ramp-raises-series-f-at-44-billion-valuation/) on the $750 million round gave 70,000+ customers, over $1 billion annualized revenue, positive free cash flow and $200 billion annualized purchase volume. Ramp's Stack release a day earlier said 50,000+ customers and $100 billion. On Trustpilot Ramp scores 2.3 from 214 reviews: people like the design and the cards, and complain about support, payout timing and sync problems.

## History

Eric Glyman and Karim Atiyeh founded Ramp in 2019, after selling Paribus to Capital One. The card launched in February 2020. The valuation went from $1.6 billion (April 2021) to $8.1 billion (2022), down to $5.8 billion (August 2023), then to $32 billion (November 2025) and $44 billion (June 2026). Ramp bought Billhop for UK and EU payments and plans to serve European companies.

## For ownpurse

Ramp shows where spend tools are heading: from cards into the books. Its agent already does much of what an owner would want an agent to do, and it lets you choose how far the agent goes. But it writes to the ledger directly, decides confidence itself, and keeps the history in its own cloud. For ownpurse, Ramp is a high-value source. A Ramp connector would record card lines, receipts, bills and Ramp's own coding as facts with provenance. ownpurse can then check what reached the ERP against what Ramp says it sent, and keep the history if you leave Ramp. The API's scopes and limits are generous enough for a local poller.
