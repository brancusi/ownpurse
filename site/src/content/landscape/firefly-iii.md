---
{
 "slug": "firefly-iii",
 "name": "Firefly III",
 "url": "https://www.firefly-iii.org",
 "segment": "plain-text-local",
 "status": "active",
 "founded": "2012 (started by James Cole for his own finances)",
 "hq": "The Netherlands (a single maintainer)",
 "backing": "No company or outside funding. One developer, James Cole, builds it as a side project, supported by Patreon, GitHub Sponsors, Ko-fi and Liberapay. TestMu AI sponsors browser testing, and Hostinger and PikaPods pay small referral fees per hosting customer.",
 "tagline": "A self-hosted, open-source personal finance manager with double-entry books, a rules engine, a full REST API and a separate data importer.",
 "scope": "Personal double-entry finance: asset, expense, revenue and liability accounts, budgets, categories, tags, piggy banks, recurring transactions, bill prediction, rules, webhooks and reports, in any currency. A separate Data Importer brings in CSV and CAMT files and pulls from bank data providers. The project says it is not for businesses: no small-business accounting or payroll, no stock portfolios, no forecasting, no subcategories or sub-accounts, and no shared administrations between users.",
 "domains": [
  "personal",
  "ledger",
  "bank",
  "reporting"
 ],
 "customers_served": [
  "consumer"
 ],
 "spectrum": {
  "custody": {
   "score": 4,
   "note": "Self-hosted: the database is on your server. No vendor account exists unless you choose a managed host."
  },
  "locality": {
   "score": 2,
   "note": "Self-hostable PHP/Laravel server (Docker, Kubernetes, bare metal), used through a browser. It is not an offline desktop app."
  },
  "history": {
   "score": 1,
   "note": "Current state in a relational database. Edits overwrite. The import hash is not updated after an edit, and there are no as-of queries."
  },
  "openness": {
   "score": 4,
   "note": "AGPL-3.0, for both the app and the data importer."
  },
  "breadth": {
   "score": 2,
   "note": "Personal books, with bank feeds from about ten providers and file formats through the importer. No documents, invoicing or business books."
  },
  "autonomy": {
   "score": 2,
   "note": "Scheduled importer runs plus rules and webhooks can create and categorize transactions with no one watching."
  },
  "agent_readiness": {
   "score": 3,
   "note": "A full REST JSON API with OpenAPI docs, OAuth2 clients, personal access tokens and webhooks. About 45 community MCP repos. No official agent interface, and the project brands itself 'No AI'."
  },
  "integration_ease": {
   "score": 3,
   "note": "Self-serve and documented (api-docs.firefly-iii.org), with OAuth and tokens. Each install is its own server, so there is no central endpoint to integrate once."
  },
  "market": {
   "score": 0,
   "note": "Individuals and households. Business use is explicitly out of scope."
  }
 },
 "api": {
  "public": true,
  "style": "REST JSON API (JSON:API-style media types), documented in Swagger/OpenAPI at api-docs.firefly-iii.org. It covers almost the whole application and adds webhooks with JSON delivery.",
  "auth": "OAuth2 clients created on the profile page (confidential or public, through /oauth/authorize and /oauth/token), or personal access tokens sent as Bearer tokens. The docs describe no fine-grained scopes.",
  "limits": "No documented rate limits on your own install. The public demo may block heavy use.",
  "cost": "Free on your own install. Bank providers set their own prices (for example, SimpleFIN Bridge charges users $1.50 a month or $15 a year as of 2026-10-10). GoCardless, Enable Banking and others have their own terms.",
  "export": "The whole database is yours (MySQL, PostgreSQL or SQLite). Everything is also readable through the API. The app offers a CSV export of transactions.",
  "notes": "The API changelog was not kept consistently before v6.4.0. Diffs are published from there on."
 },
 "pricing": {
  "model": "Free, open source, self-hosted. Third parties sell managed hosting.",
  "entry": "$0 self-hosted. PikaPods from about $2.70 a month (2026-10-10).",
  "top": "$0 for the software. Hosting and bank-data provider fees are separate.",
  "notes": "There is no official hosted service. LumaDock offers hosting that includes the data importer. Hostinger and PikaPods pay the maintainer per referral."
 },
 "agents": {
  "mcp": "community",
  "ai_features": "None. The README tagline is 'No AI, no cloud and privacy-friendly', and it says the README was written without LLMs.",
  "agent_writes": "Yes, through the REST API with a token: create and edit transactions, accounts, budgets, rules and more. Community MCP servers wrap it, such as etnperlong/firefly-iii-mcp (86 stars) and daften/fireflyiii-mcp (140 tools).",
  "approval": "None in Firefly III. A token can write directly. Rules and webhooks fire on new transactions with no review step.",
  "notes": "The maintainer's stance on AI is skeptical, so agent support will probably stay community-built on the general API."
 },
 "integration": {
  "scope": "Data Importer: CSV, CAMT.052 and CAMT.053 files. Bank data providers include GoCardless, Enable Banking, SimpleFIN, Lunch Flow, basiq.io, Sophtron, Teller, and FinTS / Open Banking (HBCI). It also takes JSON import configs, a CLI, POST-based automated imports and notifications. Firefly itself exposes the REST API and webhooks.",
  "notes": "There are two duplicate checks. The importer can skip a transaction whose unique identifier (external id, notes, description or internal reference) is already present. Firefly III also hashes each submitted transaction after the importer's mappings and roles are applied, but before rules and webhooks run. So changing a mapping, a column role or capitalization, or a bank reissuing an id, produces a different hash and a duplicate. Editing a transaction after import does not update its hash."
 },
 "preference": {
  "scale": "24.9k GitHub stars and 2.3k forks for the main repo, and 845 stars for the data importer (2026-10-10). Stable v6.7.8 shipped on 10 October 2026, with daily develop builds.",
  "ratings": "AlternativeTo: 3.2/5 from 8 ratings, 36 likes (2026-10-10). Firefly III has no G2, Capterra or Trustpilot listing with meaningful volume that we could confirm.",
  "sentiment": "Users praise privacy, self-hosting, a deep feature set, double-entry rigour, the rules engine and the API, and they like the PWA on phones. Complaints: setup is hard without Docker or database experience, the CSV importer is strict, there are no subcategories, budgeting is weaker than envelope apps, and the maintainer often declines feature requests."
 },
 "timeline": [
  {
   "date": "2012",
   "event": "James Cole starts the project to manage his own finances."
  },
  {
   "date": "2026-01 (approx.)",
   "event": "v6.4.0. From here the API changelog is published as diffs."
  },
  {
   "date": "2026-10-10",
   "event": "v6.7.8 stable: category colours, a release notes endpoint, and a fix for a static access token written to the logs in full."
  }
 ],
 "relation": "kin",
 "relation_note": "Like ownpurse, it is self-hosted and private, and its API and importer reach many bank providers. But it is a personal ledger that keeps only the normalized, mapped transaction. Its duplicate check hashes the mapped result, which ownpurse avoids.",
 "borrow": "Identifier-first duplicate detection with a content check behind it. A data importer kept separate from the ledger, with saved JSON configs and a CLI for unattended runs. One small adapter per bank-data provider. Webhooks on new transactions. A full, documented API with personal access tokens.",
 "avoid": "Hashing after mapping: identity that depends on your current column roles and capitalization breaks the moment a mapping changes. Hashes that go stale after an edit. Writes from a token or a rule with no review. Keeping only the mapped row and throwing away the raw line it came from.",
 "sources": [
  {
   "title": "Firefly III home page",
   "url": "https://www.firefly-iii.org"
  },
  {
   "title": "Firefly III on GitHub",
   "url": "https://github.com/firefly-iii/firefly-iii"
  },
  {
   "title": "Firefly III releases",
   "url": "https://github.com/firefly-iii/firefly-iii/releases"
  },
  {
   "title": "Firefly III Data Importer on GitHub",
   "url": "https://github.com/firefly-iii/data-importer"
  },
  {
   "title": "Firefly III docs: duplicate detection",
   "url": "https://docs.firefly-iii.org/references/data-importer/duplicate-detection/"
  },
  {
   "title": "Firefly III docs: API reference",
   "url": "https://docs.firefly-iii.org/references/firefly-iii/api/"
  },
  {
   "title": "Firefly III docs: API authentication",
   "url": "https://docs.firefly-iii.org/how-to/firefly-iii/features/api/"
  },
  {
   "title": "Firefly III docs: what it is not",
   "url": "https://docs.firefly-iii.org/explanation/more-information/what-its-not/"
  },
  {
   "title": "Firefly III docs: support and maintainer",
   "url": "https://docs.firefly-iii.org/explanation/support/"
  },
  {
   "title": "Firefly III docs index (data importer providers)",
   "url": "https://docs.firefly-iii.org/"
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
   "title": "GitHub search: firefly-iii mcp",
   "url": "https://github.com/search?q=firefly-iii+mcp&type=repositories"
  },
  {
   "title": "Firefly III on AlternativeTo",
   "url": "https://alternativeto.net/software/firefly-iii/about/"
  }
 ],
 "unverified": [
  "The exact date of v6.4.0. Only the API changelog note was seen, so the timeline date is approximate.",
  "The in-app CSV export and any CLI export command were not confirmed in current docs. The export docs URL returned 404.",
  "Whether Firefly III keeps an audit log of changes to transactions.",
  "G2, Capterra, Trustpilot and Reddit sentiment. Those sites blocked automated access, so sentiment comes from AlternativeTo and the docs.",
  "The link between the importer adding providers and GoCardless closing new sign-ups is an inference, not a project statement."
 ],
 "researched": "2026-10-10"
}
---

## What it is

[Firefly III](https://www.firefly-iii.org) is a free, self-hosted personal finance manager. James Cole, a developer in the Netherlands, started it in 2012 to manage his own money and still maintains it as [a side project](https://docs.firefly-iii.org/explanation/support/). It uses double-entry books underneath: asset, expense, revenue and liability accounts, budgets, categories, tags, piggy banks, recurring transactions, a rules engine, webhooks and reports in any currency. Its [README](https://github.com/firefly-iii/firefly-iii) sums it up as 'No AI, no cloud and privacy-friendly'. The project is clear about [what it is not](https://docs.firefly-iii.org/explanation/more-information/what-its-not/): no business accounting or payroll, no stock portfolios, no forecasting, no subcategories, and no shared administrations between users.

## How it works

You run it yourself as a PHP/Laravel web app: in Docker, on Kubernetes or on a plain server, against MySQL, PostgreSQL or SQLite. A separate [Data Importer](https://github.com/firefly-iii/data-importer) (also AGPL) brings data in from CSV, CAMT.052 and CAMT.053 files. It also pulls from bank data providers: [GoCardless, Enable Banking, SimpleFIN, Lunch Flow, basiq, Sophtron, Teller and FinTS](https://docs.firefly-iii.org/). It can run on a schedule from the CLI or by POST, so new transactions arrive and rules categorize them with no one watching.

[Duplicate detection](https://docs.firefly-iii.org/references/data-importer/duplicate-detection/) works two ways. The importer can skip a transaction whose unique identifier is already in Firefly. Firefly III also hashes each transaction the importer submits, after your mappings and column roles are applied but before rules and webhooks run. As a result, a changed mapping, a changed column role, different capitalization or a reissued bank id each produce a new hash, and the same bank line comes in twice. Editing a transaction afterwards does not update its hash. This is the pattern ownpurse avoids: identity depends on your interpretation, not on what the source said.

## Data and access

The REST JSON API covers almost the whole app and is documented in [Swagger](https://docs.firefly-iii.org/references/firefly-iii/api/). You [authenticate](https://docs.firefly-iii.org/how-to/firefly-iii/features/api/) with OAuth2 clients or personal access tokens. No scopes or rate limits are documented. The database is yours and nothing is metered. Bank providers charge on their own terms: SimpleFIN Bridge, for example, costs users [$1.50 a month or $15 a year](https://beta-bridge.simplefin.org/). There is no official hosted plan. [PikaPods](https://www.pikapods.com/apps) runs it from about $2.70 a month, and LumaDock and Hostinger also host it.

## Agents

Firefly III has no AI features, and the maintainer's stance is skeptical. Agents still find it easy to drive because the API is complete. [About 45 community MCP repositories](https://github.com/search?q=firefly-iii+mcp&type=repositories) wrap it, the largest with 86 stars and one with 140 tools. Writes go straight in with a token. There is no review step, and rules and webhooks fire on whatever arrives.

## Customers

Users are technically capable individuals and households who self-host. The main repository has 24.9k GitHub stars and 2.3k forks (10 October 2026). On [AlternativeTo](https://alternativeto.net/software/firefly-iii/about/) it rates 3.2/5 from 8 ratings. People praise its privacy, depth, rules and API. They complain about Docker-heavy setup, a strict CSV importer, the lack of subcategories, and a maintainer who declines many feature requests.

## History

Firefly III is the third iteration of a project started in 2012. It has grown into one of the most-starred self-hosted finance apps, with frequent releases: v6.7.8 shipped on [10 October 2026](https://github.com/firefly-iii/firefly-iii/releases), alongside daily develop builds. Bank access has followed the European market. GoCardless (formerly Nordigen) was the free route, and the importer added Enable Banking, Lunch Flow and others as GoCardless closed new sign-ups for bank data in July 2025.

## For ownpurse

Firefly III shows that a single maintainer can sustain a rich self-hosted finance app with a complete API, and that a separate importer with small per-provider adapters is a workable shape. ownpurse should borrow the identifier-first duplicate check, the saved import configs and the unattended CLI runs. Its central lesson is a warning, though. Because the hash is taken after mapping, identity depends on the mapping. ownpurse identifies an observation by what the source sent (its id, a natural key, or the SHA-256 of the raw bytes in the drawer) and treats mapping as a projection that can be rerun. Firefly also lets any token or rule write straight to the books. ownpurse's rule that agents propose and people post is the difference.
