---
{
 "slug": "sure",
 "name": "Sure",
 "url": "https://github.com/we-promise/sure",
 "segment": "personal-finance",
 "status": "active",
 "founded": "2025-07-23 (fork and GitHub organization created the day after Maybe's pivot announcement)",
 "hq": "No headquarters; a distributed community. The website footer names Sure Finances, LLC (location not confirmed).",
 "backing": "No outside funding found. Voluntary contributions through Stripe on the demo environment; the site says no goods or services are provided in exchange.",
 "tagline": "A community AGPL fork of Maybe: a self-hosted personal finance app with many bank, broker and crypto connectors, an API and an MCP endpoint.",
 "scope": "Sure carries Maybe's net-worth, budget, transaction and investment app forward and widens it: about 26 provider connectors (Plaid, SimpleFIN, Enable Banking, Lunch Flow, Mercury, Brex, Wise, IBKR, SnapTrade, Coinbase, Kraken, on-chain wallets and more), imports from YNAB, Mint, Actual, QIF and PDF, a statement vault, bills, and AI with external agents over MCP. It is self-hosted only; app.sure.am is a demo. It is not double-entry accounting and keeps no version history of transactions.",
 "domains": [
  "personal",
  "bank",
  "crypto",
  "documents",
  "reporting"
 ],
 "customers_served": [
  "consumer",
  "developers"
 ],
 "spectrum": {
  "custody": {
   "score": 3,
   "note": "You run the server and your Postgres holds the data, with a full CSV and NDJSON export; it is an app database, not a portable owner-held record."
  },
  "locality": {
   "score": 2,
   "note": "Self-hostable with Docker, Helm, Render, PikaPods or Hetzner; needs Rails, Postgres, Redis and Sidekiq, and Plaid needs a public SSL domain."
  },
  "history": {
   "score": 1,
   "note": "Transactions are updated in place by the UI, rules, AI jobs and MCP tools; only statement files are kept by SHA-256."
  },
  "openness": {
   "score": 4,
   "note": "AGPLv3, developed in the open with 100+ contributors."
  },
  "breadth": {
   "score": 3,
   "note": "Banks, cards, brokers, crypto exchanges and wallets, statements, bills and budgets for a household, across about 26 connectors; no business books."
  },
  "autonomy": {
   "score": 2,
   "note": "Background AI jobs categorize transactions and detect merchants; agents can update transactions and budgets through MCP."
  },
  "agent_readiness": {
   "score": 4,
   "note": "MCP endpoint with OAuth dynamic client registration, an OpenAPI-specified REST API, API keys and LLM guides in the docs; scopes are coarse (read_write)."
  },
  "integration_ease": {
   "score": 3,
   "note": "Self-serve and documented, but every instance is separately hosted and some connectors (Plaid) need your own developer account."
  },
  "market": {
   "score": 0,
   "note": "Individuals and households, mostly technical self-hosters."
  }
 },
 "api": {
  "public": true,
  "style": "REST (OpenAPI spec at docs/api/openapi.yaml) and MCP (JSON-RPC at POST /mcp), per self-hosted instance",
  "auth": "X-Api-Key header with user-generated keys, or OAuth2 bearer tokens from its Doorkeeper server; MCP via OAuth with dynamic client registration (read_write scope) or a static token pinned to one user email",
  "limits": "Per instance; provider request counts tracked per connector. No hosted API.",
  "cost": "Free software. You pay for hosting, your own Plaid or other aggregator accounts, and any LLM provider.",
  "export": "Family export: a zip of accounts, transactions, trades, categories, merchants and rules as CSV, an attachments manifest and all.ndjson; Sure-to-Sure import.",
  "notes": "Native reporting endpoints (for example GET /api/v1/cash_flow) let clients read server-calculated totals instead of rebuilding them."
 },
 "pricing": {
  "model": "Free and open source (AGPLv3); self-hosted only",
  "entry": "Free",
  "top": "Free; third-party hosts (Railway, PikaPods, Render) set their own prices",
  "notes": "The pricing page reads 'Pay (us) nothing' and 'one day we'll get our act together and offer a proper hosted version' (2026-10-10). Voluntary recurring contributions on the demo environment."
 },
 "agents": {
  "mcp": "official",
  "ai_features": "Built-in assistant (OpenAI-compatible provider, default gpt-4.1, or local inference via Ollama, LM Studio or LocalAI), an option to route chat to an external agent, background AI categorization and merchant detection, an insights feed, bill audits.",
  "agent_writes": "Yes: MCP tools include update_transaction, update_budget, create and update tags and categories, create_goal, set_opening_balance, import_bank_statement, upload_account_statement, record_valuation and bill tools.",
  "approval": "None for most writes: MCP and API calls apply directly. set_opening_balance supports dry_run; record_valuation requires a source citation; write tools refuse accounts shared read-only.",
  "notes": "The docs warn that the token gives the assistant access to the whole family's data and write-capable tools, and to treat it as a production secret. The statement vault deduplicates by SHA-256 and reports month-by-month coverage against the ledger."
 },
 "integration": {
  "scope": "About 26 connectors in the code (2026-10-10): Plaid, SimpleFIN, Enable Banking, Lunch Flow, Akahu, Up, Monobank, Fio, Redbark, Sophtron, FinanceKit, Mercury, Brex, Wise, IBKR, Questrade, SnapTrade, Trading 212, Trade Republic, Indexa Capital, Coinbase, Kraken, Binance, CoinSpot, CoinStats, on-chain wallets. Imports: CSV, PDF, QIF, YNAB, Mint, Actual, Sure. Clients: web, macOS (Tauri), Flutter mobile, API, MCP.",
  "notes": "The Plaid guide warns that production Plaid works only for Western (non-European) users and needs an internet-reachable instance with SSL."
 },
 "preference": {
  "ratings": "No G2, Capterra, Trustpilot or app store ratings found.",
  "scale": "10,457 GitHub stars, 614 forks, 725 open issues and about 3,682 commits on main (2026-10-10); 100+ contributors; releases from v0.6.0 (2025-07-25) to v0.7.5 (2026-09-30) and v0.7.6 alphas.",
  "sentiment": "Seen in the self-hosting community as the living continuation of Maybe; fast-moving with frequent releases. The project itself lists reliability, performance and technical debt as its first roadmap item, and publishes a performance dashboard of slow requests. Reddit threads could not be read in this pass."
 },
 "timeline": [
  {
   "date": "2025-07-22",
   "event": "Maybe announces its pivot to B2B forecasting"
  },
  {
   "date": "2025-07-23",
   "event": "we-promise organization and the sure repository created"
  },
  {
   "date": "2025-07-25",
   "event": "Sure v0.6.0, the first stable release of the fork"
  },
  {
   "date": "2025-2026",
   "event": "Connectors added well beyond Plaid: SimpleFIN, Enable Banking, Lunch Flow, brokers, crypto exchanges, on-chain wallets"
  },
  {
   "date": "2026",
   "event": "MCP endpoint with OAuth dynamic client registration; statement vault; bills and paycheck plans"
  },
  {
   "date": "2026-09-30",
   "event": "v0.7.5 released; v0.7.6 alphas follow in October"
  }
 ],
 "relation": "kin",
 "relation_note": "Sure is the closest open-source relative on the personal side: self-hosted, AGPL, many connectors, and agents first-class over MCP. Where it differs is what ownpurse is about: its database is mutable current state, there is no tamper-evident history, and agent writes apply without a person approving them. It could also be a source or a destination for ownpurse through its API and export.",
 "borrow": "MCP with OAuth dynamic client registration so a hosted assistant can connect without copying tokens; a statement vault keyed by SHA-256 with month-by-month coverage checks; dry_run on writes; valuations that require a source citation; the wide connector list as a map of what people actually hold (SimpleFIN, Enable Banking, Lunch Flow, brokers, wallets); a full export as CSV plus NDJSON.",
 "avoid": "Agent and AI writes applied directly to the data with no sealed, approved package; a single read_write scope; transactions edited in place by background jobs; a server stack (Rails, Postgres, Redis, Sidekiq, a public SSL domain for Plaid) that is heavy for one person's laptop.",
 "sources": [
  {
   "title": "we-promise/sure on GitHub",
   "url": "https://github.com/we-promise/sure"
  },
  {
   "title": "Sure website",
   "url": "https://sure.am"
  },
  {
   "title": "Sure pricing page",
   "url": "https://sure.am/pricing_free"
  },
  {
   "title": "Sure clients guide",
   "url": "https://github.com/we-promise/sure/blob/main/docs/clients.md"
  },
  {
   "title": "Sure MCP server documentation",
   "url": "https://github.com/we-promise/sure/blob/main/docs/hosting/mcp.md"
  },
  {
   "title": "Sure AI configuration",
   "url": "https://github.com/we-promise/sure/blob/main/docs/hosting/ai.md"
  },
  {
   "title": "Sure Plaid setup guide",
   "url": "https://github.com/we-promise/sure/blob/main/docs/hosting/plaid.md"
  },
  {
   "title": "Sure data exporter (family export format)",
   "url": "https://github.com/we-promise/sure/blob/main/app/models/family/data_exporter.rb"
  },
  {
   "title": "Sure roadmap",
   "url": "https://github.com/we-promise/sure/blob/main/docs/roadmap.md"
  },
  {
   "title": "Sure releases",
   "url": "https://github.com/we-promise/sure/releases"
  },
  {
   "title": "Maybe v0.6.0 final release notes",
   "url": "https://github.com/maybe-finance/maybe/releases/tag/v0.6.0"
  }
 ],
 "unverified": [
  "Who owns or runs Sure Finances, LLC, and where it is registered.",
  "Whether any maintainer is paid, and how much voluntary contribution revenue exists.",
  "Number of active self-hosted instances or users.",
  "Community sentiment on Reddit (r/selfhosted) and Discord; Reddit could not be fetched and the web search budget was exhausted in this pass.",
  "The exact date the MCP endpoint and OAuth dynamic client registration shipped.",
  "The connector count is from model files in the repository on 2026-10-10; some may be preview or partial.",
  "Whether Sure was initially started by former Maybe staff or only by community members (top contributor jjmata is not confirmed as ex-Maybe)."
 ],
 "researched": "2026-10-10"
}
---

## What it is

Sure is the community fork of Maybe, created on 23 July 2025, the day after Maybe's company announced its pivot ([repository](https://github.com/we-promise/sure)). It is a personal finance app for net worth, transactions, budgets, investments and bills, licensed AGPLv3 and developed in the open: 10,457 stars, 614 forks and 100+ contributors on 2026-10-10. The [website](https://sure.am) is run under the name Sure Finances, LLC. There is no hosted product. The [pricing page](https://sure.am/pricing_free) says "Pay (us) nothing" and asks only for voluntary contributions on the demo environment.

## How it works

Sure keeps Maybe's Rails, PostgreSQL, Redis and Sidekiq stack and adds a great deal to it. You self-host it with Docker, Helm, one-click Render templates, PikaPods or Hostim. Its code has about 26 connectors, including Plaid, SimpleFIN, Enable Banking, Lunch Flow, Mercury, Brex, Wise, IBKR, SnapTrade, Trading 212, Coinbase, Kraken, Binance and on-chain wallets, plus imports from CSV, PDF, QIF, YNAB, Mint and Actual. The [Plaid guide](https://github.com/we-promise/sure/blob/main/docs/hosting/plaid.md) shows the cost of self-hosting a bank feed: you request your own Plaid production access, it works only for Western users, and your instance must be reachable on the internet over SSL. Clients include the web app, a macOS shell, a Flutter mobile app, the API and agents ([clients guide](https://github.com/we-promise/sure/blob/main/docs/clients.md)).

## Data and access

Each instance has a REST API with an OpenAPI spec, authenticated by user-generated keys or OAuth2. A family export writes a zip of CSVs (accounts, transactions, trades, categories, merchants, rules), an attachments manifest and an NDJSON file of everything. That is a fair, complete exit. History is not kept. The UI, rules, AI jobs and agents update transactions in place. Only statement files are immutable, stored in a vault deduplicated by SHA-256.

## Agents

This is where Sure leads its segment. The [MCP endpoint](https://github.com/we-promise/sure/blob/main/docs/hosting/mcp.md) supports OAuth with dynamic client registration, so Claude.ai and similar clients can connect directly, or a static token pinned to one user. It exposes the built-in assistant's tools: reads for transactions, accounts, holdings, the balance sheet, the income statement, budgets and insights; and writes for updating transactions and budgets, managing tags and categories, importing statements, recording valuations (a source citation is required) and managing bills. `set_opening_balance` has a dry run. Otherwise writes apply at once, under one read_write scope. The docs warn that the assistant gets the whole family's data. [AI setup](https://github.com/we-promise/sure/blob/main/docs/hosting/ai.md) takes any OpenAI-compatible provider, including local Ollama or LM Studio. Background jobs that categorize transactions always call that provider.

## Customers

The users are technical self-hosters and households. No review-site ratings exist. The project moves fast, with releases from v0.6.0 (July 2025) to v0.7.5 (30 September 2026) and alphas since. Its own [roadmap](https://github.com/we-promise/sure/blob/main/docs/roadmap.md) puts reliability, performance and technical debt first, and it publishes a dashboard of its slowest requests.

## History

Sure exists because Maybe's company stopped, and it inherited Maybe's [lessons](https://github.com/maybe-finance/maybe/releases/tag/v0.6.0): bank data is the hardest part, and cache tables trade consistency for speed. Several of Maybe's core contributors, including its founder, appear in Sure's commit history through the inherited code.

## For ownpurse

Sure is kin, and the strongest evidence that people want agents inside their own finance data. It has already solved hard parts ownpurse will meet: OAuth MCP for hosted assistants, a statement vault keyed by hash with coverage checks, and a long list of connectors that maps what people actually hold. The gap is the ownpurse thesis. Sure's record is mutable current state on a server, with no tamper-evident history, no as-of queries, and agent writes with no person in the loop. ownpurse keeps the raw report of every source, chains it, and sends writes out only as sealed packages a person approves. Sure's API and NDJSON export also make it a reasonable future source for ownpurse.
