---
{
 "slug": "digits-mcp",
 "name": "Digits MCP",
 "url": "https://digits.com",
 "segment": "agent-native",
 "status": "active",
 "founded": "2018 (company; product launched 2019 per TechCrunch); MCP server launched 2026-04-21",
 "hq": "San Francisco, California (inferred from site; not stated on About page)",
 "backing": "Venture-backed, private. $10.5M Series A (Nov 2019, Benchmark), $22M Series B (Apr 2020), $65M Series C at a $565M valuation (Mar 2022, led by SoftBank Vision Fund 2 with 20VC Growth, GV, Benchmark); $97.5M total as of March 2022. No later round confirmed.",
 "tagline": "A hosted, read-only MCP server over Digits' own AI-native general ledger, so outside agents can ask questions of the books but cannot change them.",
 "scope": "Digits is a US-only cloud ledger (the AGL) that categorises, reconciles and drafts schedules with its own models, plus bill pay and invoicing. Its MCP server, launched April 2026, gives Claude, ChatGPT, Cursor and Perplexity nine read tools over transactions, dimensions and financial statements. The MCP cannot write: writes go through the separate Connect API (OAuth scopes such as source:sync and bills:manage), and bills and invoices are not yet exposed as MCP tools. It does not sit beside another ledger; you move your books into Digits.",
 "domains": [
  "ledger",
  "bank",
  "invoicing",
  "payments",
  "close",
  "reporting",
  "documents"
 ],
 "customers_served": [
  "smb",
  "freelancer",
  "accountants",
  "developers"
 ],
 "spectrum": {
  "custody": {
   "score": 1,
   "note": "Digits holds the authoritative ledger in its cloud; you can download the ledger as CSV and a financial package (PDF/XLS/XLSX), so a copy is possible but not the record."
  },
  "locality": {
   "score": 0,
   "note": "Cloud only; the MCP is a hosted endpoint at api.digits.com/mcp."
  },
  "history": {
   "score": 1,
   "note": "A conventional ledger with entries and voids via API; no documented append-only log or as-of queries."
  },
  "openness": {
   "score": 2,
   "note": "Closed source, but a free public API with an OpenAPI spec, sandbox, llms.txt, and ledger export."
  },
  "breadth": {
   "score": 3,
   "note": "Ledger, bank and card feeds (12,000+ institutions claimed), payroll connections, bills, invoices and a document vault in one product; US entities only."
  },
  "autonomy": {
   "score": 3,
   "note": "The AGL categorises and resolves high-confidence items and posts entries itself, with configurable thresholds and exceptions surfaced; the MCP itself writes nothing."
  },
  "agent_readiness": {
   "score": 3,
   "note": "Hosted MCP with OAuth on your existing login, setup for five clients, llms.txt, OpenAPI and service accounts (Oct 2026); held back from 4 because agents cannot write through MCP and bills and invoices are missing."
  },
  "integration_ease": {
   "score": 3,
   "note": "Self-serve developer accounts, sandbox, no partner or usage fees, documented rate limits (180/min per client, 60/min per token); US-only books limit reach."
  },
  "market": {
   "score": 2,
   "note": "Small businesses, startups and the accounting firms that serve them."
  }
 },
 "api": {
  "public": true,
  "style": "REST (route-versioned), cursor pagination, OpenAPI spec, beta webhooks; separate MCP (streamable HTTP) for read-only agent access",
  "auth": "OAuth 2.0 authorization code with scopes (ledger:read, source:sync, organization:sync, user:sync, bills:manage); service accounts with client ID and secret since 2026-10-05; MCP uses OAuth on the user's Digits login, no API key",
  "limits": "180 requests per minute per client ID; 60 requests per minute per access token",
  "cost": "API and MCP included on every plan; Digits states no partner fees, no usage paywalls, no marketplace restrictions",
  "export": "Ledger CSV for a date range; Financial Package (ledger, trial balance, statements, aging); P&L, balance sheet, cash flow and aging as PDF or XLS; chart of accounts CSV; documents from Vault",
  "notes": "The API writes (sync transactions, void and delete, bill workflows); the MCP deliberately does not. Read-only status is stated on the MCP help page: 'cannot make any changes to your Digits account'."
 },
 "pricing": {
  "model": "Per-business monthly subscription; firm partner pricing separate",
  "entry": "Essentials $65/month",
  "top": "Pro $250/month (listed 'coming soon'); firm plans unpriced",
  "notes": "30-day free trial, no setup fees, one subscription per business or location, US entities only; prices from digits.com/pricing on 2026-10-10."
 },
 "agents": {
  "mcp": "official",
  "ai_features": "Agentic General Ledger (AGL) for categorisation and posting, AI bank reconciliation, automated schedules (prepaids, fixed assets), configurable plain-language quality checks, Ask Digits chat, Agentic Close for firms",
  "agent_writes": "Not through MCP. External code can write through the Connect API with write scopes; Digits' own agents write to the ledger internally.",
  "approval": "No approval gate for external agents because they cannot write via MCP. Inside the product, lower-confidence items and exceptions are surfaced for review with configurable thresholds by party, category and materiality; workspace admins in Claude or ChatGPT may need to approve the connector.",
  "notes": "Nine MCP tools: list_businesses, select_business, list_business_users, query_transactions, search_term, list_departments, list_locations, list_categories, dimensional_summarize_transactions, financial_statement (P&L, balance sheet, cash flow, AR/AP aging). Digits has not published a reason for read-only; its October 2026 post argues agents outside the ledger are less accurate than ones trained on it, which fits keeping outside agents to reads."
 },
 "integration": {
  "scope": "Bank and card feeds (12,000+ institutions claimed), payroll connectors, Zapier, Connect API for third-party sources; MCP clients Claude Desktop, Claude Code, ChatGPT (OpenAI marketplace), Cursor, Perplexity",
  "notes": "Integration is inbound: other systems feed Digits' ledger. There is no connector that leaves books in QuickBooks or Xero and reads them."
 },
 "preference": {
  "scale": "'Trusted by 7,000+ accountants and business owners' (digits.com/agl, 2026); 'thousands of businesses run Digits without an accountant' (digits.com/ai-info, 2026)",
  "ratings": "Trustpilot: no reviews (profile claimed May 2025). G2 and Capterra pages could not be read (blocked).",
  "sentiment": "Not enough independent reviews found to characterise. Vendor benchmark claims AGL is 97.8% accurate at categorisation (Digits, 2026-10-07), unverified independently."
 },
 "timeline": [
  {
   "date": "2019-11-06",
   "event": "$10.5M Series A from Benchmark and others (TechCrunch)"
  },
  {
   "date": "2020-04-23",
   "event": "Free expense-monitoring dashboard launched; $22M Series B"
  },
  {
   "date": "2022-03-24",
   "event": "$65M Series C at a $565M valuation, led by SoftBank Vision Fund 2"
  },
  {
   "date": "2025-10-28",
   "event": "Connect API launched ('Digits is now programmable'), free, with MCP and llms.txt support"
  },
  {
   "date": "2026-02-24",
   "event": "Digits Essentials plan for smaller businesses"
  },
  {
   "date": "2026-04-21",
   "event": "Digits MCP server launched for Claude, ChatGPT, Cursor and other MCP clients"
  },
  {
   "date": "2026-06-02",
   "event": "Digits MCP available to all users in the OpenAI marketplace"
  },
  {
   "date": "2026-10-05",
   "event": "Service accounts for the API without building an OAuth app"
  },
  {
   "date": "2026-10-07",
   "event": "Blog post 'AI can't close your books from the outside' arguing for ledger-native agents"
  }
 ],
 "relation": "competitor",
 "relation_note": "Digits asks you to move your books into its ledger and keeps outside agents to reads; ownpurse leaves the books where they are and lets agents read a local copy freely, with writes only through approved packages. Same instinct about agent writes, opposite answer about custody.",
 "borrow": "Read-only by default for agents is a defensible, explainable stance; the small, well-named MCP tool set (select_business, query_transactions, dimensional_summarize_transactions, financial_statement) is a good model for ownpurse's own MCP; OAuth on an existing login with no developer setup; llms.txt for agent docs; free API as a positioning statement.",
 "avoid": "Making read-only the only answer, which leaves agents unable to propose any change; tying agent access to a hosted ledger in one country; vendor-run benchmarks as the main evidence of accuracy.",
 "sources": [
  {
   "title": "Digits MCP help page",
   "url": "https://help.digits.com/firms-connections/digits-mcp"
  },
  {
   "title": "Digits Developer API & MCP",
   "url": "https://digits.com/api/"
  },
  {
   "title": "Digits developer docs",
   "url": "https://developer.digits.com"
  },
  {
   "title": "Digits rate limits",
   "url": "https://developer.digits.com/docs/rate-limits.md"
  },
  {
   "title": "Digits pricing",
   "url": "https://digits.com/pricing/"
  },
  {
   "title": "Digits changelog",
   "url": "https://help.digits.com/changelog"
  },
  {
   "title": "Digits is now programmable (2025-10-28)",
   "url": "https://digits.com/blog/digits-is-now-programmable/"
  },
  {
   "title": "AI can't close your books from the outside (2026-10-07)",
   "url": "https://digits.com/blog/ai-cant-close-books-from-outside/"
  },
  {
   "title": "Digits AGL",
   "url": "https://digits.com/agl/"
  },
  {
   "title": "Digits Agentic Close",
   "url": "https://digits.com/product/agentic-close/"
  },
  {
   "title": "Digits for AI assistants",
   "url": "https://digits.com/ai-info/"
  },
  {
   "title": "TechCrunch: Digits books $65M on a $565M valuation (2022-03-24)",
   "url": "https://techcrunch.com/2022/03/24/digits-books-65m-on-a-565m-valuation-to-bring-a-more-dynamic-automated-approach-to-legacy-accounting-tools/"
  },
  {
   "title": "TechCrunch Digits tag (Series A and B)",
   "url": "https://techcrunch.com/tag/digits/"
  },
  {
   "title": "Trustpilot: Digits",
   "url": "https://www.trustpilot.com/review/digits.com"
  }
 ],
 "unverified": [
  "Founding year 2018 and San Francisco HQ (TechCrunch says launched 2019; site shows only a San Francisco clock)",
  "Which founder is CEO (TechCrunch 2022 names Wayne Chang as CEO)",
  "Any funding after March 2022; the earlier survey grouped Digits among AI ledgers raising in 2025-26, not confirmed here",
  "Why Digits chose read-only for MCP: no stated reason found",
  "Whether Digits keeps an append-only audit history or supports as-of queries",
  "G2 and Capterra ratings (pages blocked)",
  "AGL accuracy figures (97.8%, '23% more accurate') are vendor benchmarks"
 ],
 "researched": "2026-10-10"
}
---

## What it is

Digits is a US cloud accounting product built around its own ledger, which it calls the [Agentic General Ledger (AGL)](https://digits.com/agl/). In April 2026 it added a hosted [MCP server](https://help.digits.com/firms-connections/digits-mcp) at `api.digits.com/mcp` so that Claude, ChatGPT, Cursor and Perplexity can query a Digits ledger. The server is read-only: Digits' own help page says it "cannot make any changes to your Digits account" and answers "Can I write back to Digits from the MCP?" with "No."

## How it works

You add the Digits connector in your AI tool and sign in with your Digits login over OAuth. No API key or developer account is needed. Nine tools cover choosing a business, querying and fuzzy-searching transactions, listing categories, departments and locations, multi-dimensional summaries, and financial statements (P&L, balance sheet, cash flow, AR and AP aging). Bills and invoices are not yet MCP tools.

Writes exist, but elsewhere. The [Connect API](https://developer.digits.com), launched on [28 October 2025](https://digits.com/blog/digits-is-now-programmable/), takes raw transactions and posts categorised entries, voids and deletes transactions, and runs bill workflows under scopes such as `source:sync` and `bills:manage`. Inside the product, Digits' agents categorise, reconcile and draft schedules on their own, and surface lower-confidence items and exceptions for review with thresholds you can set by party, category and materiality ([Agentic Close](https://digits.com/product/agentic-close/)).

Digits has not published a reason for keeping MCP read-only. Its [October 2026 post](https://digits.com/blog/ai-cant-close-books-from-outside/) argues that agents working from outside the ledger look context up instead of learning it, and are less accurate and slower. The benchmark is Digits' own.

## Data and access

The API is free on every plan, with "no partner fees, no usage paywalls, no marketplace lock-ins." Limits are 180 requests a minute per client and 60 per access token ([rate limits](https://developer.digits.com/docs/rate-limits.md)). There is an OpenAPI spec, a sandbox, an `llms.txt`, and since 5 October 2026 service accounts for your own systems ([changelog](https://help.digits.com/changelog)). You can export the ledger as CSV, a financial package with trial balance and statements, reports as PDF or XLS, and the chart of accounts as CSV. Plans run from $65 to $250 a month per business ([pricing](https://digits.com/pricing/)), US entities only.

## Agents

Digits is one of the few vendors that has said, in product, that outside agents may read and may not write. That is close to ownpurse's instinct. The difference is what happens next: with Digits an agent that finds a problem can only tell you; there is no way for it to propose a change for you to approve.

## Customers

Digits says it is trusted by 7,000+ accountants and business owners and serves small businesses, startups and firms. Independent reviews are thin: no Trustpilot reviews, and G2 and Capterra could not be read for this dossier.

## History

Founded by Wayne Chang and Jeff Seibert (the founders of Crashlytics), Digits raised $97.5M through a $65M Series C at a $565M valuation in [March 2022](https://techcrunch.com/2022/03/24/digits-books-65m-on-a-565m-valuation-to-bring-a-more-dynamic-automated-approach-to-legacy-accounting-tools/). It began as an analytics dashboard over other ledgers and became a ledger itself. The API arrived in October 2025, MCP in April 2026, the ChatGPT listing in June 2026.

## For ownpurse

Digits is the clearest example of an AI ledger choosing read-only for agents, and it does it well: a small tool set, OAuth on your normal login, clear limits. ownpurse should borrow the shape of the tools and the plain read-only default. Where it differs is custody and the path to writes. Digits wants your books in its cloud and lets external agents do nothing but look. ownpurse leaves the books in Xero or QuickBooks, gives agents a local record to read at no API cost, and gives them a way to propose a write as a sealed package that you approve.
