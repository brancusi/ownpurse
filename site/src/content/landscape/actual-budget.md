---
{
 "slug": "actual-budget",
 "name": "Actual Budget",
 "url": "https://actualbudget.org",
 "segment": "plain-text-local",
 "status": "active",
 "founded": "Created by James Long as a paid app in the late 2010s. Open-sourced in 2022. The hosted subscription was shut down in 2024.",
 "hq": "No headquarters; a distributed open-source community",
 "backing": "No company or venture funding now. It is funded by donations through Open Collective and GitHub Sponsors, and by a share of PikaPods hosting fees: PikaPods gives 20% of fees to the project, and over 8,000 PikaPods users were counted in December 2025. The monthly contributor pool rose from $1,000 to $2,000 in January 2026.",
 "tagline": "A local-first, open-source envelope budgeting app. Every device holds the full budget, and an optional self-hosted server syncs changes between devices.",
 "scope": "Personal envelope budgeting, accounts and net worth, transactions with splits and linked transfers, rules, schedules, custom reports, and import from QIF, OFX, QFX, CAMT.053, CSV and YNAB. Bank sync runs through SimpleFIN (US/Canada), GoCardless and Enable Banking (Europe), Pluggy.ai (Brazil) and Akahu (New Zealand). It is not double-entry accounting and does not handle business books, invoicing or documents.",
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
   "score": 4,
   "note": "Each device holds a full copy, and the sync server is yours. Optional end-to-end encryption hides budget contents from the server."
  },
  "locality": {
   "score": 4,
   "note": "Local-first: the engine runs on your device in SQLite and works offline. A server is needed only to sync."
  },
  "history": {
   "score": 2,
   "note": "The budget file stores every mutation as a sync message and has undo and redo. But you cannot ask what it looked like on a date, and resetting sync compacts the history."
  },
  "openness": {
   "score": 4,
   "note": "MIT-licensed."
  },
  "breadth": {
   "score": 1,
   "note": "Personal budgets and accounts, with bank sync through five aggregators. No business books, documents or payments."
  },
  "autonomy": {
   "score": 1,
   "note": "Bank sync and rules apply categories and fuzzy-merge duplicates, but people budget and review."
  },
  "agent_readiness": {
   "score": 3,
   "note": "A Node API that runs the engine on a local copy, and a stable CLI (July 2026) that outputs JSON by default with stderr errors and --dry-run. Popular community MCP servers. No HTTP API or scoped tokens."
  },
  "integration_ease": {
   "score": 2,
   "note": "Documented and self-serve, but Node only. No HTTP endpoints, and you integrate through the sync server's password and sync id."
  },
  "market": {
   "score": 0,
   "note": "Individuals and households."
  }
 },
 "api": {
  "public": true,
  "style": "@actual-app/api, a Node.js package. It downloads the budget, runs the same engine locally, and syncs changes back. There is an experimental browser build (WASM SQLite in a Web Worker) and ActualQL queries. Actual has no HTTP or REST endpoints. Since July 2026 the @actual-app/cli also reads and writes from the terminal.",
  "auth": "The sync server's password or a session token, plus the budget's sync id and, for end-to-end encrypted budgets, an encryption password. There are no per-app tokens or scopes.",
  "limits": "No documented rate limits. The CLI docs warn that each call opens a new server connection, so rapid calls can be rate limited or fail authentication.",
  "cost": "Free. Bank sync providers charge on their own terms (for example, SimpleFIN Bridge charges users $1.50 a month or $15 a year as of 2026-10-10). GoCardless no longer accepts new accounts.",
  "export": "Settings > Export Data downloads the budget file. The API and CLI expose accounts, transactions, payees, rules and schedules as JSON or CSV. YNAB imports keep history.",
  "notes": "importTransactions dedupes by imported id first, then fuzzy-matches on a similar date, the same amount and a similar payee. addTransactions skips that reconciliation."
 },
 "pricing": {
  "model": "Free, open source. Third parties sell managed hosting of the sync server.",
  "entry": "$0 self-hosted. PikaPods from about $2.00 a month (2026-10-10).",
  "top": "$0 for the software.",
  "notes": "The earlier paid hosted subscription was shut down in 2024."
 },
 "agents": {
  "mcp": "community",
  "ai_features": "None in the product. The project uses AI to write code, but since November 2025 it closes pull requests that look '100% AI-generated with no sign of human oversight'.",
  "agent_writes": "Yes. The Node API and the CLI can create, update and delete accounts, categories, payees, rules, schedules and transactions, set budget amounts, and trigger bank sync. Community MCP servers include s-stefanov/actual-mcp (254 stars) and others. One adds confirmation before deletes.",
  "approval": "None in Actual. Whoever holds the server password can write. The CLI's --dry-run for imports is the only rehearsal.",
  "notes": "With JSON output by default, non-zero exit codes and stderr errors, the CLI is close to an agent-first interface, though the project does not market it that way."
 },
 "integration": {
  "scope": "Bank sync: SimpleFIN Bridge (US/Canada), GoCardless BankAccountData and Enable Banking (Europe), Pluggy.ai (Brazil) and Akahu (New Zealand; experimental in July 2026). Files: CSV, QIF, OFX, QFX, CAMT.053, YNAB4 and nYNAB. API and CLI for custom importers. Plugins are on the 2026 roadmap, starting with moving the bank sync providers into plugins.",
  "notes": "You sign up with each bank provider and bring your own API keys. Bank sync tokens are stored on the server separately and are not covered by end-to-end encryption."
 },
 "preference": {
  "scale": "29.4k GitHub stars and 3.1k forks (2026-10-10). Over 8,000 PikaPods users (December 2025). Monthly releases, the latest 26.10.0 on 2026-10-02.",
  "ratings": "AlternativeTo: 4.0/5 from 5 ratings, 36 likes (2026-10-10). No confirmed G2, Capterra or Trustpilot listing with meaningful volume.",
  "sentiment": "Users praise speed, a clean design, offline use, sync across devices, privacy, being free instead of paying for YNAB, Monarch or Simplifi, and a responsive community. Complaints: no native mobile app (the PWA is the mobile client), weak auto-categorization for CSV imports, manual work for irregular bills, and the effort of self-hosting the server."
 },
 "timeline": [
  {
   "date": "2019-12",
   "event": "James Long presents 'CRDTs for Mortals' at dotJS, describing the sync design behind Actual."
  },
  {
   "date": "2022",
   "event": "Actual open-sourced under the MIT licence. The community takes over development, and the first community releases ship (22.10.x)."
  },
  {
   "date": "2023-06-13",
   "event": "The open-source project moves its site, docs and demo to actualbudget.org."
  },
  {
   "date": "2024",
   "event": "The original paid hosted subscription shuts down."
  },
  {
   "date": "2025",
   "event": "Small stipends for core maintainers begin, based on points for maintenance work."
  },
  {
   "date": "2025-11-15",
   "event": "'Fighting AI Slop' policy on contributions with no human oversight."
  },
  {
   "date": "2025-12-20",
   "event": "2026 roadmap: plugins (bank sync first), a goals UI, mobile parity, a transaction table rewrite, debt accounts."
  },
  {
   "date": "2026-01-17",
   "event": "Contributor funding pool raised from $1,000 to $2,000 a month."
  },
  {
   "date": "2026-07-02",
   "event": "26.7.0: the Actual CLI becomes stable, and Akahu bank sync arrives (experimental)."
  },
  {
   "date": "2026-10-02",
   "event": "26.10.0 released."
  }
 ],
 "relation": "kin",
 "relation_note": "The closest local-first kin. Each device holds the whole record in SQLite, changes travel as messages through a server you run, and the server cannot read them. It is a budget, though, not books beside a ledger. It keeps the merged transaction, not the source's raw report, and its history is for sync, not something you can query.",
 "borrow": "A full copy on every device, with sync as messages that a server only relays, and optional end-to-end encryption. Import that tries the source id first, then fuzzy-matches date, amount and payee, keeps the bank's date, and lets you choose whether to reimport deleted lines. A CLI with JSON by default, exit codes and --dry-run. Bring-your-own-key bank providers, moving into plugins. Funding maintainers from hosting revenue share and donations.",
 "avoid": "Letting the imported line silently overwrite a person's manual entry (it rewrites the existing transaction's date). A mutation history that 'reset sync' compacts away. Integer cents with no decimal type at the boundary. Writes with a shared server password and no scoped tokens or approval.",
 "sources": [
  {
   "title": "Actual Budget home page",
   "url": "https://actualbudget.org"
  },
  {
   "title": "Actual on GitHub",
   "url": "https://github.com/actualbudget/actual"
  },
  {
   "title": "Actual releases",
   "url": "https://github.com/actualbudget/actual/releases"
  },
  {
   "title": "Actual docs: API",
   "url": "https://actualbudget.org/docs/api/"
  },
  {
   "title": "Actual docs: CLI",
   "url": "https://actualbudget.org/docs/api/cli"
  },
  {
   "title": "Actual docs: bank sync",
   "url": "https://actualbudget.org/docs/advanced/bank-sync"
  },
  {
   "title": "Actual docs: importing transactions",
   "url": "https://actualbudget.org/docs/transactions/importing"
  },
  {
   "title": "Actual docs: sync",
   "url": "https://actualbudget.org/docs/getting-started/sync"
  },
  {
   "title": "Actual docs: architecture",
   "url": "https://actualbudget.org/docs/contributing/project-details/architecture"
  },
  {
   "title": "Actual docs: backup and export",
   "url": "https://actualbudget.org/docs/backup-restore/backup"
  },
  {
   "title": "Actual docs: FAQ (hosted service shut down in 2024)",
   "url": "https://actualbudget.org/docs/faq"
  },
  {
   "title": "Release 26.7.0",
   "url": "https://actualbudget.org/blog/release-26.7.0"
  },
  {
   "title": "Roadmap for 2026",
   "url": "https://actualbudget.org/blog/roadmap-for-2026"
  },
  {
   "title": "Next steps for funding contributors",
   "url": "https://actualbudget.org/blog/next-steps-for-funding-contributors"
  },
  {
   "title": "Fighting AI slop",
   "url": "https://actualbudget.org/blog/fighting-ai-slop"
  },
  {
   "title": "Welcome to Actual Budget open source",
   "url": "https://actualbudget.org/blog/welcome-actualbudget-os"
  },
  {
   "title": "James Long's archived blog (Actual sync engine posts)",
   "url": "https://archive.jlongster.com/"
  },
  {
   "title": "PikaPods app prices",
   "url": "https://www.pikapods.com/apps"
  },
  {
   "title": "SimpleFIN Bridge pricing",
   "url": "https://beta-bridge.simplefin.org/"
  },
  {
   "title": "GitHub search: actual budget mcp",
   "url": "https://github.com/search?q=actual+budget+mcp&type=repositories"
  },
  {
   "title": "Actual Budget on AlternativeTo",
   "url": "https://alternativeto.net/software/actual-budget/about/"
  }
 ],
 "unverified": [
  "The exact open-sourcing date in 2022 (believed April 2022), and the original subscription price (believed about $4 a month). James Long's announcement could not be fetched.",
  "The year Actual first launched as a paid product, and the legal entity behind it (believed Shift Reset LLC).",
  "The exact CRDT details (hybrid logical clocks, merkle trie). They come from James Long's 2019 talk and code, not from the current docs, which do not mention CRDTs.",
  "Whether the exported budget is a zip of db.sqlite and metadata.json. The docs do not say.",
  "G2, Capterra, Trustpilot and Reddit sentiment. Those sites blocked automated access."
 ],
 "researched": "2026-10-10"
}
---

## What it is

[Actual Budget](https://actualbudget.org) is a free, open-source, local-first envelope budgeting app. You budget only money you already have, keep every account in one place, and see net worth and cash flow. It is MIT-licensed, has 29.4k GitHub stars (10 October 2026), and ships monthly. [26.10.0](https://github.com/actualbudget/actual/releases) came out on 2 October 2026. It is a budget, not double-entry books, and it does not handle business accounting, invoicing or documents.

## How it works

Each device runs the whole engine against its own SQLite copy (see the [architecture notes](https://actualbudget.org/docs/contributing/project-details/architecture)), so it works offline. An optional sync server, which you host yourself or rent on [PikaPods from about $2.00 a month](https://www.pikapods.com/apps), relays changes between devices. Per the [sync docs](https://actualbudget.org/docs/getting-started/sync), the budget file stores every change and grows over time. With end-to-end encryption on, the server cannot read budget contents. James Long described the design, CRDT-style messages with ordered timestamps, in his 2019 talk 'CRDTs for Mortals'.

[Import](https://actualbudget.org/docs/transactions/importing) takes CSV, QIF, OFX, QFX and CAMT, plus YNAB files with their history. It dedupes by the file's transaction id first, then fuzzy-matches on a similar date, the same amount and a similar payee. When an imported line matches a manual entry, the import wins and the existing date is changed to the bank's. [Bank sync](https://actualbudget.org/docs/advanced/bank-sync) runs through SimpleFIN (North America), GoCardless (no longer taking new accounts) and Enable Banking (Europe), Pluggy.ai (Brazil) and Akahu (New Zealand). You bring your own keys for each.

## Data and access

The [API](https://actualbudget.org/docs/api/) is a Node package, not an HTTP service. It downloads the budget, runs the engine locally and syncs changes back. Amounts are integer cents. Since [26.7.0](https://actualbudget.org/blog/release-26.7.0) (2 July 2026) a stable [CLI](https://actualbudget.org/docs/api/cli) reads and writes accounts, budgets, transactions, rules and schedules, runs ActualQL queries and triggers bank sync. It outputs JSON by default, writes errors to stderr, exits non-zero on failure and supports --dry-run for imports. Authentication is the server password or a session token. There are no per-app scopes. Export is the budget file from Settings. The [backup docs](https://actualbudget.org/docs/backup-restore/backup) note that 'Reset Sync' compresses the stored change history into one file.

## Agents

The product has no AI features. The project uses AI to write code but [closes contributions with no human oversight](https://actualbudget.org/blog/fighting-ai-slop) (November 2025). Agents reach it through the API and CLI, and [community MCP servers](https://github.com/search?q=actual+budget+mcp&type=repositories) are popular: s-stefanov/actual-mcp has 254 stars, and one adds confirmation before deletes. Actual itself has no approval gate. Whoever holds the password can write.

## Customers

Its users are individuals and households, often coming from YNAB, Monarch or Simplifi to avoid a subscription. On [AlternativeTo](https://alternativeto.net/software/actual-budget/about/) it rates 4.0/5 from 5 ratings. People praise its speed, clean design, offline use and sync. They complain about the lack of a native mobile app, weak categorization of CSV imports and the work of handling irregular bills.

## History

James Long built Actual as a paid product with a hosted subscription. It was open-sourced in 2022, community releases began that October, and the project moved to [actualbudget.org in June 2023](https://actualbudget.org/blog/welcome-actualbudget-os). The [hosted subscription shut down in 2024](https://actualbudget.org/docs/faq). The community now funds itself with donations and a PikaPods revenue share: over 8,000 PikaPods users per the [2026 roadmap](https://actualbudget.org/blog/roadmap-for-2026), and a contributor pool raised to [$2,000 a month](https://actualbudget.org/blog/next-steps-for-funding-contributors) in January 2026. The roadmap's main item is plugins, starting with bank sync providers.

## For ownpurse

Actual is the nearest local-first kin, and it shows the model can win users without venture money: a full copy everywhere, a server that only relays, encryption the host cannot read. It also survived its creator stepping back because the data was already on users' machines. That contrasts with Bench or Maybe, where the books lived in the vendor's cloud. ownpurse differs in three ways. It keeps what each source reported, not just the merged transaction. Its history is the queryable record, not a sync log that can be compacted. A fuzzy match becomes a proposed link a person accepts, not an overwrite of their entry. Actual's CLI is worth comparing directly: JSON by default, exit codes and --dry-run are close to AXI. ownpurse adds counts, help hints and sealed, approved writes.
