---
{
 "slug": "digits",
 "name": "Digits",
 "url": "https://digits.com",
 "segment": "ai-ledger",
 "status": "active",
 "founded": "2018",
 "hq": "San Francisco, California",
 "backing": "Venture-backed, about $97.5M raised (CB Insights). Last disclosed round: $65M Series C led by SoftBank Vision Fund, 24 March 2022, at a reported $565M valuation. Other investors include Benchmark and GV. A March 2025 release says 'nearly $100M'. No round found in 2025 or 2026.",
 "tagline": "An AI-native general ledger for startups, small businesses and their accounting firms, with a free API and a read-only MCP server.",
 "scope": "Digits replaces QuickBooks or Xero with its own cloud ledger (the 'Autonomous General Ledger') that books and reconciles transactions from bank, card and payroll feeds, and adds reporting, bill pay, invoicing and a document vault. Firms run many clients from one dashboard. It does not sit beside your existing ledger: the QuickBooks import is a one-time pull, after which Digits is the book of record. It does not run locally or keep an owner-held copy.",
 "domains": [
  "ledger",
  "bank",
  "invoicing",
  "payments",
  "documents",
  "close",
  "reporting"
 ],
 "customers_served": [
  "smb",
  "accountants",
  "developers"
 ],
 "spectrum": {
  "custody": {
   "score": 1,
   "note": "Digits holds the ledger in its cloud; the free API lets you read entries and statements out, but no documented bulk export was found."
  },
  "locality": {
   "score": 0,
   "note": "Cloud only; the MCP server is remote at api.digits.com/mcp."
  },
  "history": {
   "score": 1,
   "note": "Entries are kept and traceable, but no documented append-only guarantee or as-of query; audit trail details not found."
  },
  "openness": {
   "score": 2,
   "note": "Closed source, but a free public REST API with OpenAPI-generated SDKs on every plan; full export unconfirmed."
  },
  "breadth": {
   "score": 3,
   "note": "Ledger, bank and payroll feeds, bill pay, invoicing, vault and reporting in one place; no crypto or personal finance."
  },
  "autonomy": {
   "score": 3,
   "note": "Claims to auto-book 95%+ of transactions without a person; complex accruals still need manual work."
  },
  "agent_readiness": {
   "score": 3,
   "note": "Remote MCP with OAuth (read-only), scoped OAuth API, llms.txt; agents cannot write through MCP."
  },
  "integration_ease": {
   "score": 3,
   "note": "Self-serve, free, documented API with OAuth scopes, webhooks (beta) and published rate limits."
  },
  "market": {
   "score": 2,
   "note": "Startups, small businesses and the firms that serve them."
  }
 },
 "api": {
  "public": true,
  "style": "REST (Digits Connect, connect.digits.com/v1), route-versioned, cursor pagination, SDKs generated from OpenAPI, webhooks in beta",
  "auth": "OAuth 2.0 authorization code with refresh tokens; scopes include ledger:read, source:sync, organization:sync, user:sync, bills:manage",
  "limits": "180 requests per minute per client id, 60 per minute per access token (install); 429 with retry_after; higher limits on request",
  "cost": "Free on all plans ('The Digits API is free', April 2026 release)",
  "export": "Ledger reads cover entries, transactions, chart of accounts, trial balance, P&L, balance sheet, cash flow, aging and parties. No bulk export endpoint or export guarantee found.",
  "notes": "The API is not read-only: source:sync writes and deletes transactions keyed by external_id, and bills:manage (beta) can approve and pay bills. Only the MCP server is read-only."
 },
 "pricing": {
  "model": "Flat monthly subscription per business or location; firm pricing custom",
  "entry": "Essentials $65 per month",
  "top": "Pro $250 per month (marked coming soon); Core $100 per month is the most popular",
  "notes": "30-day free trial, no setup fees, each business or location needs its own subscription. API and MCP included on every plan. Partner services (full-service accounting, CFO, tax) priced separately and not listed."
 },
 "agents": {
  "mcp": "official",
  "ai_features": "Autonomous General Ledger auto-categorises and reconciles; Ask Digits conversational assistant; Agentic Close for firms; plain-English quality checks compiled into deterministic checks; AI quality review.",
  "agent_writes": "Not through MCP: 'All access is read-only by design, preserving ledger integrity' (21 April 2026). Programs with OAuth scopes can write through the API (sync transactions, manage bills).",
  "approval": "No approval gate for outside agents because the MCP cannot write. Inside the product, the ledger books automatically and firms review through quality checks.",
  "notes": "Read-only MCP is the clearest current statement that a vendor does not trust outside agents to write to the books. The reason given is ledger integrity, not a review workflow."
 },
 "integration": {
  "scope": "Bank and card feeds (the firm page claims 12,000+ financial institutions), payroll connections, one-time QuickBooks Online migration, Xero conversion through services; MCP clients Claude, ChatGPT, Cursor.",
  "notes": "QuickBooks migration is a one-time pull of transactions, journal entries, chart of accounts, classes, locations, contacts and products; reconciliation history, documents and attachments do not move. Digits never writes back to QuickBooks."
 },
 "preference": {
  "scale": "'Thousands' of startups, small businesses and firms (vendor, 2026); named customers include Particle News, Wispr, Partiful, Replika.",
  "ratings": "G2 4.8 from 5 reviews (as found October 2026); Trustpilot profile claimed May 2025, 0 reviews.",
  "sentiment": "Praise for the interface, reports and drill-down from trend to transaction. Accountants worry about black-box categorisation and want to see why a decision was made; some reviewers could not find report export; bank connection breakage is a general complaint. Review volume is too small to read much into."
 },
 "timeline": [
  {
   "date": "2018",
   "event": "Founded by Jeff Seibert and Wayne Chang (previously Crashlytics, acquired by Twitter in 2013)"
  },
  {
   "date": "2022-03-24",
   "event": "$65M Series C led by SoftBank Vision Fund at a reported $565M valuation"
  },
  {
   "date": "2025-03-10",
   "event": "Launches the Autonomous General Ledger at HumanX, positioned against QuickBooks; Xero co-founder Craig Walker joins"
  },
  {
   "date": "2026-02-09",
   "event": "Named a 2026 Top New Product for Accountants by Accounting Today"
  },
  {
   "date": "2026-04-21",
   "event": "Launches a free, read-only remote MCP server for Claude, ChatGPT and Cursor"
  }
 ],
 "relation": "neighbour",
 "relation_note": "Digits asks you to move your books into its ledger; ownpurse leaves them where they are and keeps a copy you hold. Digits is also a plausible future source: its free OAuth API exposes entries and statements.",
 "borrow": "The read-only-by-default MCP stance and the stated reason (ledger integrity); a free API on every plan; published per-install rate limits; plain-English checks compiled into deterministic rules.",
 "avoid": "A one-way migration that leaves reconciliation history and attachments behind, and a ledger with no documented bulk export; autonomy claims (95%+, 97.8%) measured only by the vendor.",
 "sources": [
  {
   "title": "Digits pricing",
   "url": "https://digits.com/pricing"
  },
  {
   "title": "Digits Connect API docs",
   "url": "https://developer.digits.com"
  },
  {
   "title": "Digits API rate limits",
   "url": "https://developer.digits.com/docs/rate-limits"
  },
  {
   "title": "Digits Launches MCP Server (21 April 2026, via Yahoo Finance)",
   "url": "https://finance.yahoo.com/sectors/technology/articles/digits-launches-mcp-server-connecting-130000822.html"
  },
  {
   "title": "Digits MCP press release, GlobeNewswire",
   "url": "https://www.globenewswire.com/news-release/2026/04/21/3278035/0/en/digits-launches-mcp-server-connecting-ai-tools-directly-to-its-real-time-ai-native-ledger.html"
  },
  {
   "title": "Digits announces Autonomous General Ledger, Accounting Today",
   "url": "https://accountingtoday.com/news/digits-announces-autonomous-general-ledger"
  },
  {
   "title": "AGL launch press release, 10 March 2025",
   "url": "https://www.globenewswire.com/news-release/2025/03/10/3039814/0/en/AI-Startup-Digits-Takes-on-QuickBooks-with-the-World-s-First-Autonomous-General-Ledger-for-Accounting-Xero-Co-founder-Craig-Walker-Joins-Digits.html"
  },
  {
   "title": "Migrate data from QuickBooks Online, Digits help",
   "url": "https://help.digits.com/firms-getting-started/migrate-from-quickbooks"
  },
  {
   "title": "Digits for firms comparison",
   "url": "https://digits.com/accountants/compare/"
  },
  {
   "title": "SoftBank leads $65M round of Digits, Finance Magnates",
   "url": "https://www.financemagnates.com/fintech/news/softbank-leads-65-million-funding-round-of-digits/"
  },
  {
   "title": "Digits vs Docyt, CB Insights",
   "url": "https://www.cbinsights.com/compare/digits-2-vs-docyt"
  },
  {
   "title": "Digits on G2",
   "url": "https://www.g2.com/sellers/digits"
  },
  {
   "title": "Digits on Trustpilot",
   "url": "https://www.trustpilot.com/review/digits.com"
  },
  {
   "title": "Accounting Today 2026 Top New Product, GlobeNewswire",
   "url": "https://www.globenewswire.com/news-release/2026/02/09/3234583/0/en/Digits-Named-a-2026-Top-New-Product-for-Accountants-by-Accounting-Today.html"
  },
  {
   "title": "Review summary, beancount.io blog",
   "url": "https://beancount.io/blog/2025/08/05/digits-ai-accountant-balancing-brilliant-dashboards-with-the-need-for-human-trust"
  }
 ],
 "unverified": [
  "Dates of the $10.5M Series A (Benchmark) and $22M round (GV); amounts appear in secondary sources only",
  "The 97.8% accuracy and 95%+ auto-booking figures are vendor-measured",
  "The 12,000+ financial institutions figure is from Digits' own comparison page",
  "A reported CLA partnership to co-build a firm-specific model (Runtimewire); not confirmed against CLA or Digits releases",
  "Whether Digits offers a bulk export of the full ledger, and its terms on data on cancellation",
  "Whether the ledger is append-only or keeps prior versions of edited entries",
  "The earlier survey's grouping of Digits in the 2025-26 funding wave: no Digits round after March 2022 was found",
  "Web search budget ran out mid-research; later findings come from direct fetches of primary pages only"
 ],
 "researched": "2026-10-10"
}
---

## What it is

Digits is an AI-native general ledger for startups, small businesses and accounting firms. It launched its [Autonomous General Ledger](https://accountingtoday.com/news/digits-announces-autonomous-general-ledger) on 10 March 2025 as a replacement for QuickBooks, after years as a reporting product. It books, reconciles and reports, and adds bill pay, invoicing, a document vault and a firm dashboard. [Pricing](https://digits.com/pricing) is flat per business: Essentials $65 a month, Core $100, Pro $250 (coming soon), with a 30-day trial.

## How it works

You connect bank, card and payroll feeds, and Digits books transactions into its own ledger. It claims to auto-book "95%+ of transactions" and, on its [firm comparison page](https://digits.com/accountants/compare/), 97.8% accuracy in its own benchmarking. Both are vendor figures. Firms can write quality checks in plain English that Digits turns into deterministic rules. Moving in from QuickBooks Online is a [one-time pull](https://help.digits.com/firms-getting-started/migrate-from-quickbooks): transactions, journal entries, the chart of accounts and contacts come across, but reconciliation history, invoice and bill documents and attachments do not. After the cutover the QuickBooks connection is archived, and Digits never writes back.

## Data and access

The [Digits Connect API](https://developer.digits.com) is free on every plan. It uses OAuth 2.0 with scopes (`ledger:read`, `source:sync`, `bills:manage` and others), cursor pagination, webhooks in beta and SDKs generated from OpenAPI. Reads cover entries, transactions, the trial balance, statements and aging. Writes exist too: partners can sync and delete transactions keyed by `external_id`, and in beta they can approve and pay bills. [Rate limits](https://developer.digits.com/docs/rate-limits) are 180 requests a minute per app and 60 per install. We found no bulk export endpoint and no stated export guarantee.

## Agents

On 21 April 2026 Digits [launched a remote MCP server](https://finance.yahoo.com/sectors/technology/articles/digits-launches-mcp-server-connecting-130000822.html) at `api.digits.com/mcp` for Claude, ChatGPT and Cursor, free and on all plans. "All access is read-only by design, preserving ledger integrity." That is the clearest statement in this segment of why a vendor will not let outside agents write: the ledger is the product, and its accuracy claim depends on Digits' own models doing the booking. The cost is that an outside agent can only read. It cannot propose a change for a person to approve. Writes go through the API, by programs, with no review step for outside agents.

## Customers

Digits says "thousands" of startups, small businesses and firms use it, and names Particle News, Wispr and Partiful. Public review volume is small: [G2](https://www.g2.com/sellers/digits) shows 4.8 from 5 reviews, and Trustpilot shows none. Reviewers praise the interface and reports. Accountants on forums want to see why the software made each decision ([summary](https://beancount.io/blog/2025/08/05/digits-ai-accountant-balancing-brilliant-dashboards-with-the-need-for-human-trust)).

## History

Jeff Seibert and Wayne Chang founded Digits in 2018. It raised about $97.5M, the last round a $65M Series C led by SoftBank in March 2022 at a reported $565M valuation ([Finance Magnates](https://www.financemagnates.com/fintech/news/softbank-leads-65-million-funding-round-of-digits/)). We found no round since then. That corrects our earlier survey, which grouped Digits with the 2025-26 funding wave: Digits ships into that wave but was not funded by it. In 2025 it moved from reporting to being the ledger itself, and Xero co-founder Craig Walker joined.

## For ownpurse

Digits is a neighbour, not a rival. It wants to be the book of record, and ownpurse sits beside whatever book you already keep. Three things are worth taking from it: read-only by default for agents, a free API with published limits, and checks written in plain English that compile into deterministic rules. ownpurse's answer to read-only is different. Agents may propose writes, and the writes go out only as sealed packages that a person approves. Digits is also a plausible future connector, since its API exposes entries and statements to the owner at no cost.
