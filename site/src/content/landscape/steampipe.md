---
{
 "slug": "steampipe",
 "name": "Steampipe",
 "url": "https://steampipe.io",
 "segment": "connector-framework",
 "status": "active",
 "founded": "2021 (Steampipe, first public release Feb 2021); Turbot, its maker, 2014",
 "hq": "No office: Turbot says its team is remote across 6 time zones and 4 continents (as of 2026-10)",
 "backing": "Built by Turbot, which says it is 100% bootstrapped, with no outside investors (turbot.com/about, 2026-10)",
 "tagline": "An open-source CLI that queries live APIs with SQL through plugins, without copying the data anywhere.",
 "scope": "Steampipe turns each API into Postgres or SQLite tables and fetches rows from the source when you query them. It ships as a CLI with a bundled Postgres, as Postgres foreign data wrappers, as SQLite extensions and as standalone export binaries. It does not keep a record: results sit in a cache for 300 seconds by default, then they are gone. It is read-only and built mostly for cloud, security and DevOps sources. Turbot publishes no Xero or QuickBooks plugin.",
 "domains": [
  "integration",
  "payments",
  "commerce"
 ],
 "customers_served": [
  "developers",
  "smb",
  "mid-market",
  "enterprise"
 ],
 "spectrum": {
  "custody": {
   "score": 1,
   "note": "It stores nothing durable. The source keeps the only authoritative copy, and Steampipe holds a 300-second cache."
  },
  "locality": {
   "score": 4,
   "note": "A CLI, a SQLite extension or a Postgres FDW, all on your own machine. Turbot Pipes is an optional hosted version."
  },
  "history": {
   "score": 0,
   "note": "Every query returns the source's current state, and nothing is versioned."
  },
  "openness": {
   "score": 4,
   "note": "The CLI is AGPL-3.0, the plugins and the MCP server are Apache-2.0, and the plugin SDK is public."
  },
  "breadth": {
   "score": 2,
   "note": "158 plugins as of 2026-10, mostly cloud and SaaS. Finance covers Stripe, Shopify and market quotes, with no ledger plugin from Turbot."
  },
  "autonomy": {
   "score": 0,
   "note": "It is read-only and never writes to any source."
  },
  "agent_readiness": {
   "score": 3,
   "note": "There is an official read-only MCP server (Apr 2025) and SQL any agent can write, but no write path and no scoped agent tokens."
  },
  "integration_ease": {
   "score": 3,
   "note": "It speaks the Postgres wire protocol, has SQLite virtual tables, a documented Go plugin SDK and public table docs on the Hub."
  },
  "market": {
   "score": 2,
   "note": "Developers and cloud or security teams at companies of any size, with a hosted tier from free to $249 a month."
  }
 },
 "api": {
  "public": true,
  "style": "SQL (Postgres wire protocol, SQLite virtual tables). Plugins are Go gRPC services built on a public SDK.",
  "auth": "Each plugin uses its source's own credentials (API keys, OAuth tokens) set in local .spc config files. Pipes workspaces use a connection string.",
  "limits": "You inherit each source's API limits, because every uncached query calls the source. Cache TTL defaults to 300 s.",
  "cost": "The CLI is free. Turbot Pipes: Developer free (400 compute minutes, 3 GB), Team $49/mo, Enterprise $249/mo. Compute add-on $0.0001/s, storage $0.25/GB/mo (pricing page, 2026-10).",
  "export": "Standalone steampipe export binaries write query results to CSV or JSON. There is no stored record to export beyond what you query.",
  "notes": "Plugins declare key columns and qualifiers, which the planner pushes down to the API as filters. Version 1.0 (Oct 2024) moved dashboards and checks into Powerpipe."
 },
 "pricing": {
  "model": "Open-source CLI. The hosted Turbot Pipes is billed per plan plus compute and storage.",
  "entry": "Free (CLI; Pipes Developer tier)",
  "top": "Pipes Enterprise $249/mo with 3 users, 10,000 compute minutes and 100 GB, plus add-ons (2026-10)",
  "notes": "Turbot sells its governance products (Turbot Guardrails) separately. Steampipe itself costs nothing."
 },
 "agents": {
  "mcp": "official",
  "ai_features": "The Steampipe MCP server (turbot/steampipe-mcp, Apache-2.0, announced 2025-04-17) gives an agent tools to run SQL, list and describe tables and inspect plugins, plus a best_practices prompt.",
  "agent_writes": "No. The MCP README says all queries are read-only, and plugins expose no write operations.",
  "approval": "Not applicable, because nothing is written.",
  "notes": "It needs a running local Steampipe service (postgresql://steampipe@localhost:9193) or a Pipes workspace."
 },
 "integration": {
  "scope": "158 plugins on the Steampipe Hub (2026-10): AWS, Azure, GCP, GitHub, Salesforce, HubSpot, Google Sheets, CSV, Stripe (9 tables, v1.2.0, Oct 2025), Shopify, and a finance plugin for market quotes.",
  "notes": "For a third party, writing a plugin is the documented route. I found no first-party Xero, QuickBooks or Plaid plugin in the turbot GitHub org."
 },
 "preference": {
  "scale": "About 8,000 GitHub stars on turbot/steampipe (2026-10). Turbot cites 12,000+ stars across 340+ repos and a 3,200+ member Slack community.",
  "ratings": "Not confirmed. G2 returned 403 and web search was unavailable.",
  "sentiment": "Not confirmed from review sites. The steady releases (v2.4.7 on 2026-09-16) and the Hub's per-table docs suggest the project is well maintained."
 },
 "timeline": [
  {
   "date": "2014",
   "event": "Turbot founded (bootstrapped)"
  },
  {
   "date": "2021-01",
   "event": "turbot/steampipe repository created; v0.2.0 released 2021-02-18"
  },
  {
   "date": "2024-10-22",
   "event": "v1.0.0 removes check, dashboard and mod commands, now in Powerpipe"
  },
  {
   "date": "2025-04-17",
   "event": "Steampipe MCP server announced"
  },
  {
   "date": "2025-06-11",
   "event": "v2.0.0"
  },
  {
   "date": "2026-09-16",
   "event": "v2.4.7, the latest release as of 2026-10-10"
  }
 ],
 "relation": "design-cousin",
 "relation_note": "The opposite answer to the same question. Steampipe queries the source live and keeps nothing, while ownpurse keeps every fact so reads cost no API calls and every past state can be queried.",
 "borrow": "SQLite virtual tables as an interface (ownpurse could expose its facts as SQLite views or virtual tables to any SQL client). Per-table docs with example queries. A read-only MCP server that teaches the agent the schema through a best_practices prompt. Key-column pushdown so a query names exactly what it needs.",
 "avoid": "Live querying against rationed APIs: against Xero's 1,000 calls a day per organisation, each uncached join spends quota. Also a query layer with no history, so last month's state is gone once the source changes.",
 "sources": [
  {
   "title": "Steampipe home",
   "url": "https://steampipe.io"
  },
  {
   "title": "turbot/steampipe on GitHub",
   "url": "https://github.com/turbot/steampipe"
  },
  {
   "title": "Steampipe Hub plugins",
   "url": "https://hub.steampipe.io/plugins"
  },
  {
   "title": "Steampipe caching guide",
   "url": "https://steampipe.io/docs/guides/caching"
  },
  {
   "title": "Steampipe SQLite extensions",
   "url": "https://steampipe.io/docs/steampipe_sqlite/overview"
  },
  {
   "title": "Steampipe MCP server announcement",
   "url": "https://steampipe.io/blog/steampipe-mcp"
  },
  {
   "title": "turbot/steampipe-mcp",
   "url": "https://github.com/turbot/steampipe-mcp"
  },
  {
   "title": "Turbot Pipes pricing",
   "url": "https://turbot.com/pipes/pricing"
  },
  {
   "title": "About Turbot",
   "url": "https://turbot.com/about"
  },
  {
   "title": "Stripe plugin",
   "url": "https://hub.steampipe.io/plugins/turbot/stripe"
  }
 ],
 "unverified": [
  "G2, Capterra or Reddit ratings and sentiment (G2 returned 403; web search was unavailable)",
  "Whether a community (non-Turbot) Xero or QuickBooks plugin exists on the Hub",
  "Turbot's legal headquarters and founders"
 ],
 "researched": "2026-10-10"
}
---

## What it is

[Steampipe](https://steampipe.io) is an open-source CLI from Turbot that lets you query APIs with SQL. Each source is a plugin that exposes tables. When you run a query, the plugin calls the source's API and returns rows. Turbot calls this zero-ETL: nothing is copied into a warehouse first. The [Hub](https://hub.steampipe.io/plugins) listed 158 plugins as of October 2026, most of them for cloud and SaaS (AWS, Azure, GCP, GitHub, Salesforce), plus Stripe, Shopify, CSV and Google Sheets.

## How it works

The [repository](https://github.com/turbot/steampipe) describes four distributions: the CLI with a bundled Postgres, native Postgres foreign data wrappers, [SQLite extensions](https://steampipe.io/docs/steampipe_sqlite/overview) that create virtual tables, and standalone export binaries. Plugins declare key columns, so a `where` clause is pushed down to the API as a filter. Results are [cached](https://steampipe.io/docs/guides/caching) per table and per connection for 300 seconds by default, and every client on the same service shares that cache. The CLI is AGPL-3.0, and the plugins and SDK are Apache-2.0. Version 1.0 (22 October 2024) removed dashboards and compliance checks, which now live in Powerpipe. Version 2.0 shipped on 11 June 2025, and the latest release is v2.4.7 (16 September 2026).

## Data and access

Steampipe keeps no record of its own. The source holds the data, Steampipe fetches it when you ask, and the cache expires. That is what Steampipe does well: no pipeline to run and always current answers. It also means there is no history, and every uncached query spends the source's API quota. Credentials stay in local `.spc` config files. Turbot Pipes, the hosted version, is [priced](https://turbot.com/pipes/pricing) at free (Developer), $49 a month (Team) and $249 a month (Enterprise), plus compute at $0.0001 per second and storage at $0.25 per GB per month (as of October 2026).

## Agents

Turbot released the [Steampipe MCP server](https://steampipe.io/blog/steampipe-mcp) on 17 April 2025. It is Apache-2.0 and gives an agent tools to run SQL, list and describe tables and inspect plugins. Its [README](https://github.com/turbot/steampipe-mcp) says all queries are read-only. Because the whole interface is SQL with documented tables, an agent can explore it without special tooling. Nothing can be written, so there is no approval question to answer.

## Customers

Steampipe serves developers and cloud, security and DevOps teams. Turbot says it has 12,000+ GitHub stars across its repositories and a Slack community of 3,200+ ([about](https://turbot.com/about)), and the main repository has about 8,000 stars. I could not reach G2 or other review sites, so ratings are unconfirmed. Turbot reports no outside funding.

## History

Turbot was founded in 2014 and built cloud governance software before Steampipe. Steampipe went public in early 2021 (v0.2.0 on 18 February 2021). Over time Turbot split the stack into Steampipe (query), Powerpipe (dashboards and benchmarks), Flowpipe (workflows) and Tailpipe (logs on DuckDB). For accounting it is a road not taken: there is no first-party Xero, QuickBooks or Plaid plugin.

## For ownpurse

Steampipe is the clearest case of the other answer: query the source when you ask, and keep nothing. It works when the source is cheap to call and only the present matters. Accounting is the opposite on both counts. Xero rations calls, and the questions that matter are about the past: what did this invoice say at the close, and when did it change? So the record is necessary. Two parts are still worth borrowing. One is the SQLite virtual-table interface, which would let any SQL client read the record without a server. The other is a read-only MCP server that teaches the agent the schema before it queries.
