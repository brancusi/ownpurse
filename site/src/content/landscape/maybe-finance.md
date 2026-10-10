---
{
 "slug": "maybe-finance",
 "name": "Maybe Finance",
 "url": "https://github.com/maybe-finance/maybe",
 "segment": "personal-finance",
 "status": "archived",
 "founded": "2021 (first app built 2021-2022; open-source Rails rebuild from January 2024)",
 "hq": "Not confirmed",
 "backing": "Venture-backed: the final release says the company's obligation is to its investors. Amount and investors not confirmed. Founded by Josh Pigford (@Shpigford, earlier founder of Baremetrics). Maybe Finance Inc. holds the 'Maybe' trademark.",
 "tagline": "An open-source personal finance and net-worth app, AGPL, whose company archived it in July 2025 to pivot to B2B forecasting.",
 "scope": "A self-hostable Rails app for net worth, transactions, budgets, investments and multi-currency accounts, with Plaid sync, CSV and Mint imports, rules, an AI chat (bring your own OpenAI key) and an API. It is not accounting software and keeps no version history. The company no longer maintains it; the code lives on in forks, chiefly Sure.",
 "domains": [
  "personal",
  "bank",
  "reporting"
 ],
 "customers_served": [
  "consumer"
 ],
 "spectrum": {
  "custody": {
   "score": 3,
   "note": "Self-hosted, your own Postgres holds the data and the AGPL code; the short-lived hosted Maybe kept it in the company's cloud."
  },
  "locality": {
   "score": 2,
   "note": "Self-hostable in Docker (one container plus Postgres; Sidekiq and Redis from v0.5.0); a server app rather than a desktop one."
  },
  "history": {
   "score": 1,
   "note": "Entries are stored as facts but are mutable and deletable via UI and API; balances and holdings are rebuilt into cache tables, with no versions kept."
  },
  "openness": {
   "score": 4,
   "note": "AGPLv3 source; forks allowed if they drop the Maybe name and logo."
  },
  "breadth": {
   "score": 2,
   "note": "Bank, card, loan, property and investment accounts for a household; no business books, documents or payments."
  },
  "autonomy": {
   "score": 1,
   "note": "Rules auto-categorize and detect merchants; the AI chat answers questions; a person does the rest."
  },
  "agent_readiness": {
   "score": 2,
   "note": "A key-managed REST API (accounts, transactions CRUD, chats) and built-in AI chat; no MCP for the app itself."
  },
  "integration_ease": {
   "score": 2,
   "note": "API and CSV imports are self-serve, but every instance is separately hosted and the project is unmaintained."
  },
  "market": {
   "score": 0,
   "note": "Consumers and households."
  }
 },
 "api": {
  "public": true,
  "style": "REST (Rails, /api/v1) per self-hosted instance",
  "auth": "API keys managed in the app; signup, login and refresh endpoints for the mobile client",
  "limits": "Per instance; a usage endpoint exists. No hosted API.",
  "cost": "Free for self-hosters; Plaid and market data (Synth) were bring-your-own or optional dependencies.",
  "export": "A family export feature (family_export model) in the final code; CSV imports for accounts, transactions and trades; Mint import.",
  "notes": "Read-only archive since 27 July 2025; the final release v0.6.0 (24 July 2025) is offered as-is."
 },
 "pricing": {
  "model": "Free self-hosted (AGPL); a hosted 'Maybe Plus' subscription briefly offered",
  "entry": "Free to self-host",
  "top": "Hosted: $9 per month or $90 per year (Maybe marketing site source, undated, 2024-2025)",
  "notes": "The hosted version did not prove a sustainable business and ended with the July 2025 pivot."
 },
 "agents": {
  "mcp": "none",
  "ai_features": "From v0.5.0 (11 April 2025): optional AI chat that knows your finances, using your own OpenAI key, disabled by default; AI-assisted categorization and merchant detection.",
  "agent_writes": "The REST API allows creating, updating and deleting transactions with an API key; the AI chat used read tools.",
  "approval": "None: API writes apply immediately.",
  "notes": "The company published a separate synth-mcp repository (June 2025) for its Synth market-data product, not for the finance app."
 },
 "integration": {
  "scope": "Plaid for bank sync (US and EU regions), Synth for market data and exchange rates (optional), CSV and Mint imports, an API for automations.",
  "notes": "The final release names bank data providers as the single biggest challenge: unsupported banks, wrong data, documentation that did not match production."
 },
 "preference": {
  "ratings": "No G2, Capterra or app store ratings found for the self-hosted app.",
  "scale": "54,236 GitHub stars and 5,677 forks at the archived repository (2026-10-10); over 100 contributors and 1,419 commits; 'hundreds' of contributors per the final release.",
  "sentiment": "Widely praised for a clean, simple design and for treating transfers and one-time expenses as first-class. Disappointment at the abandonment; data-provider gaps and investment data were the main weaknesses, by the team's own account."
 },
 "timeline": [
  {
   "date": "2021-2022",
   "event": "First Maybe app built: personal finance and wealth management with an 'Ask an Advisor' CFP feature; Teller for account connections"
  },
  {
   "date": "2023 (mid)",
   "event": "Business shut down after spending close to $1,000,000"
  },
  {
   "date": "2024-01",
   "event": "Revived as open source (maybe-archive), then rebuilt in Rails in maybe-finance/maybe"
  },
  {
   "date": "2024-2025",
   "event": "Hosted Maybe offered at $9 per month or $90 per year"
  },
  {
   "date": "2025-04-11",
   "event": "v0.5.0: AI chat with your own OpenAI key, dark mode, Sidekiq and Redis"
  },
  {
   "date": "2025-07-22",
   "event": "Josh Pigford announces the pivot to B2B financial forecasting and scenario planning"
  },
  {
   "date": "2025-07-24",
   "event": "Final release v0.6.0, 'Farewell... Maybe'"
  },
  {
   "date": "2025-07-27",
   "event": "Repository archived, read-only"
  },
  {
   "date": "2026-10-10",
   "event": "maybe.co no longer resolves; maybefinance.com redirects to it"
  }
 ],
 "relation": "cautionary-tale",
 "relation_note": "Maybe tried twice to sell a personal finance app, failed both times as a consumer business, and left a well-built AGPL engine behind. Its farewell names the problems ownpurse is built around: bank data that is wrong, and the trade between event-sourced accuracy and fast reads.",
 "borrow": "Transfers as first-class, never categorized; reconciling to a balance without an orphan 'adjustment' transaction; one-time expenses kept out of averages; the honest post-mortem itself. Above all the stated lesson: write facts and derive views on the read side for guaranteed accuracy.",
 "avoid": "The 'hybrid event-sourced' compromise: mutable facts plus cache tables that give up consistency. Depending on one paid bank aggregator for the core experience. A consumer SaaS business model that obliged the company to abandon the product. Bug reports that need users' real financial data (ownpurse's synthetic fixtures answer this).",
 "sources": [
  {
   "title": "maybe-finance/maybe on GitHub (archived)",
   "url": "https://github.com/maybe-finance/maybe"
  },
  {
   "title": "Maybe v0.6.0 final release notes, 'Farewell... Maybe' (2025-07-24)",
   "url": "https://github.com/maybe-finance/maybe/releases/tag/v0.6.0"
  },
  {
   "title": "Maybe v0.5.0 release notes (2025-04-11)",
   "url": "https://github.com/maybe-finance/maybe/releases/tag/v0.5.0"
  },
  {
   "title": "maybe-finance/maybe-archive README (backstory)",
   "url": "https://github.com/maybe-finance/maybe-archive"
  },
  {
   "title": "maybe-finance/marketing (hosted pricing in site source)",
   "url": "https://github.com/maybe-finance/marketing"
  },
  {
   "title": "Maybe API routes (config/routes.rb)",
   "url": "https://github.com/maybe-finance/maybe/blob/main/config/routes.rb"
  },
  {
   "title": "Josh Pigford's pivot announcement (linked from the final release)",
   "url": "https://x.com/Shpigford/status/1947725345244709240"
  },
  {
   "title": "we-promise/sure, the community fork",
   "url": "https://github.com/we-promise/sure"
  }
 ],
 "unverified": [
  "Total funding raised by Maybe Finance Inc., the investors and the round dates.",
  "Headquarters location.",
  "The exact launch and end dates of the hosted Maybe Plus subscription, and its subscriber count.",
  "Whether the self-hosted edition supported Plaid out of the box or only with your own Plaid credentials.",
  "What the B2B forecasting product is called and whether it launched; maybe.co did not resolve on 2026-10-10.",
  "Founding year of the company (2021 is inferred from 'the better part of 2021/2022').",
  "The date of the pivot post is inferred from the linked status id; the post itself was not fetched."
 ],
 "researched": "2026-10-10"
}
---

## What it is

Maybe was an open-source personal finance app: net worth, transactions, budgets, investments and multi-currency accounts for a household. Its company, Maybe Finance Inc., founded by Josh Pigford, built it twice and stopped twice. The repository [maybe-finance/maybe](https://github.com/maybe-finance/maybe) has been archived since 27 July 2025, with 54,236 stars and 5,677 forks (2026-10-10). The code is AGPLv3; forks must drop the Maybe name and logo.

## How it works

The final version is a Ruby on Rails app with Hotwire, backed by PostgreSQL and, from v0.5.0, Sidekiq and Redis. You self-host it in Docker. Accounts sync through Plaid or are kept by hand. CSV imports cover accounts, transactions and trades, and there is a Mint importer. Rules auto-categorize transactions and detect merchants. The [final release notes](https://github.com/maybe-finance/maybe/releases/tag/v0.6.0) describe the design: transfers are detected and cannot be categorized, one-time expenses are kept out of averages, and you reconcile an account to a new balance without an unexplained adjustment entry.

The same notes describe the data model honestly. The team wrote "facts" (transactions, trades, valuations), synced them in background jobs and materialized them into cache tables (balances, holdings). Their verdict: full event sourcing guarantees accuracy but was too slow for their pages; the cache tables made reads fast "at the cost of data consistency".

## Data and access

Each instance has a REST API under `/api/v1` with keys managed in the app. It covers accounts, create-read-update-delete on transactions, AI chats and usage. A family export exists in the final code. History is not kept: an edited or deleted transaction is gone, and the cache tables are rebuilt from whatever is current.

## Agents

[v0.5.0](https://github.com/maybe-finance/maybe/releases/tag/v0.5.0) (11 April 2025) added an AI chat that knows your finances. It uses your own OpenAI key and is off by default. There was no MCP server for the app. An API key can write transactions directly, with no approval step.

## Customers

Maybe drew a large open-source following (over 100 contributors, "hundreds" per the team) and praise for its clean design. No review-site ratings exist. The hosted "Maybe Plus" cost $9 a month or $90 a year according to its [marketing site source](https://github.com/maybe-finance/marketing); it did not become a sustainable business.

## History

The first Maybe (2021-2022) was a hosted wealth app with an "Ask an Advisor" feature that connected users with a CFP. It used Teller for bank links. The [archived first repository](https://github.com/maybe-finance/maybe-archive) says the team spent close to $1,000,000 on it before shutting down in mid-2023. In January 2024 they open-sourced it, then rebuilt it in Rails with a self-hosted edition and a paid hosted one. On 22 July 2025 Pigford announced a pivot to B2B forecasting. v0.6.0 shipped on 24 July and the repository was archived three days later. The company's obligation, the notes say, was to its investors. A community fork, [Sure](https://github.com/we-promise/sure), was created on 23 July 2025.

## For ownpurse

Maybe is the clearest cautionary tale in the personal segment, and it was candid about why. The biggest problem it named was bank data: unsupported institutions, data that is wrong, documentation that does not match production. ownpurse expects that and keeps every source's raw report to check against. Its other lesson is the one ownpurse is built on: write facts, derive the views. Maybe compromised there for speed. ownpurse makes the log authoritative and treats tables as caches it can rebuild. And Maybe could not see users' data to fix bugs. ownpurse's rule of synthetic fixtures only, with a record the owner holds, is the answer to that.
