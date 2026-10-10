---
{
 "slug": "dbt",
 "name": "dbt",
 "url": "https://www.getdbt.com",
 "segment": "data-platform",
 "status": "acquired",
 "founded": "2016 (as Fishtown Analytics; the dbt repository dates from March 2016)",
 "hq": "Philadelphia, Pennsylvania (dbt Labs); since June 2026 part of Fivetran + dbt Labs, led by Fivetran's CEO",
 "backing": "Venture-backed (Andreessen Horowitz, Sequoia, Altimeter, Databricks and Snowflake among reported investors) until it merged with Fivetran in an all-stock deal announced 13 October 2025 and completed 1 June 2026. The combined company reported about $600 million ARR at announcement (October 2025).",
 "tagline": "An open-source SQL transformation framework, plus a hosted platform, that turns raw warehouse tables into tested, documented models.",
 "scope": "dbt compiles SQL and YAML models into tables and views inside a warehouse you already have (Snowflake, BigQuery, Databricks, Postgres, DuckDB and others), with tests, documentation, lineage and a semantic layer of metrics. It does not ingest data, hold data itself, or keep books: it is the T in ELT, and finance teams use it to model revenue, close and reporting tables on top of what Fivetran-style connectors load.",
 "domains": [
  "reporting",
  "close",
  "integration"
 ],
 "customers_served": [
  "mid-market",
  "enterprise",
  "developers"
 ],
 "spectrum": {
  "custody": {
   "score": 3,
   "note": "Data stays in your own warehouse and models are code in your own Git repository; the hosted platform holds only metadata and job history."
  },
  "locality": {
   "score": 3,
   "note": "dbt Core v2 is a local binary and, with the built-in DuckDB adapter, runs fully on a laptop; most real use runs against a cloud warehouse and the hosted platform."
  },
  "history": {
   "score": 2,
   "note": "Models are normally rebuilt and overwritten each run; snapshots give SCD2-style history for tables you choose, but nothing is append-only by default."
  },
  "openness": {
   "score": 3,
   "note": "dbt Core is Apache 2.0 and dbt Core v2 open-sources the Fusion runtime, but there is also a proprietary distribution and a closed hosted platform; Fusion first shipped under ELv2."
  },
  "breadth": {
   "score": 1,
   "note": "One layer only: transformation and metrics over data someone else loaded."
  },
  "autonomy": {
   "score": 1,
   "note": "Runs scheduled builds unattended, but writes only derived tables; it never posts to a ledger."
  },
  "agent_readiness": {
   "score": 4,
   "note": "Official MCP server (local and remote), a CLI agents can drive, service tokens, an agent assistant (dbt Wizard) and an open Agents Schema for agent context."
  },
  "integration_ease": {
   "score": 3,
   "note": "Well documented with adapters for most warehouses, but you need a warehouse, SQL and YAML skills, and a data engineer's habits."
  },
  "market": {
   "score": 3,
   "note": "Data teams at mid-market and enterprise companies; a free single-seat tier exists but small businesses rarely use it."
  }
 },
 "api": {
  "public": true,
  "style": "Hosted platform: Administrative API (REST), Discovery API (GraphQL metadata), Semantic Layer APIs (GraphQL and JDBC), job webhooks; dbt Core itself is a CLI that writes JSON artifacts (manifest.json, run_results.json).",
  "auth": "Personal access tokens or service account tokens; OAuth for the remote MCP server.",
  "limits": "API access only on Starter and above; documented per-API rate limits; remote MCP defaults to 5,000 requests per minute per IP.",
  "cost": "API access is included in paid plans (Starter at $100 per user per month and up); the free Developer plan has no API access. Warehouse compute is billed by your warehouse vendor, not dbt.",
  "export": "Everything you write is plain SQL and YAML in your own Git repository; built models live in your warehouse; run metadata is exported as JSON artifacts and, in v2, as Parquet.",
  "notes": "dbt Core v2 (alpha, June 2026) has two distributions, 'dbt' (proprietary) and 'dbt-oss' (Apache 2.0), both free to install locally; platform features unlock by logging in from the terminal."
 },
 "pricing": {
  "model": "Per seat plus metered models built and metrics queried on the hosted platform; dbt Core is free.",
  "entry": "Developer: free, 1 seat, 3,000 models built per month, no API access (pricing page, read 10 October 2026).",
  "top": "Enterprise and Enterprise+: custom pricing, 100,000 models built and 20,000 queried metrics per month, SSO, RBAC, PrivateLink.",
  "notes": "Starter is $100 per user per month for up to 5 seats and 15,000 models built; overage rates are on a separate consumption page not confirmed here."
 },
 "agents": {
  "mcp": "official",
  "ai_features": "dbt Copilot (generation and text-to-SQL), dbt Wizard (beta, June 2026) for authoring, refactoring and debugging grounded in lineage and tests, the open Agents Schema for agent context in a warehouse schema, and dbt State (preview) to rebuild only changed models.",
  "agent_writes": "The self-hosted MCP server (Apache 2.0, github.com/dbt-labs/dbt-mcp) can run dbt run, build and test against a local project, so an agent can change and rebuild derived tables. The remote MCP server is read-oriented: metadata, lineage, Semantic Layer queries.",
  "approval": "No human approval gate on tool calls is documented; control is by token scope, plan, and enabling or disabling individual tools. Code changes still go through the team's Git review and CI checks, which act as the practical gate.",
  "notes": "text_to_sql is the only MCP tool that consumes Copilot allotment; the rest keep working when it runs out."
 },
 "integration": {
  "scope": "Adapters for the major warehouses and lakes (Snowflake, BigQuery, Databricks, Redshift, Postgres, DuckDB built into v2, and more); a large package ecosystem; Fivetran publishes over 100 open-source dbt packages for its connectors, including accounting sources.",
  "notes": "Integration means pointing dbt at a warehouse where data already sits. DuckDB ships inside dbt v2 (September 2026), so a single local DuckDB file is now a first-class target."
 },
 "preference": {
  "scale": "More than 10,000 customers at the merger announcement (October 2025); the combined Fivetran + dbt Labs says it serves more than 100,000 data teams (June 2026). The dbt repository has about 14,000 GitHub stars (October 2026).",
  "ratings": "G2 and Capterra pages could not be retrieved (blocked) for this dossier; no rating is given.",
  "sentiment": "Practitioners praise version-controlled SQL, tests, documentation and the community. Complaints center on hosted-platform pricing changes, the ELv2 license on the first Fusion release, migration effort to the new engine, and concern about vendor consolidation after the Fivetran merger."
 },
 "timeline": [
  {
   "date": "2016-03",
   "event": "dbt repository created at Fishtown Analytics, Philadelphia"
  },
  {
   "date": "2023",
   "event": "Acquires Transform, the basis of the dbt Semantic Layer (date not confirmed here)"
  },
  {
   "date": "2025-01-14",
   "event": "Acquires SDF Labs for SQL comprehension and faster compilation"
  },
  {
   "date": "2025-03",
   "event": "dbt MCP server repository published"
  },
  {
   "date": "2025-05",
   "event": "dbt Fusion engine released in beta, mostly under ELv2"
  },
  {
   "date": "2025-10-13",
   "event": "Merger with Fivetran announced; combined ARR about $600 million"
  },
  {
   "date": "2026-06-01",
   "event": "Merger completed; dbt Core v2.0 alpha open-sources the Fusion runtime under Apache 2.0; dbt Wizard, dbt State and Agents Schema launched"
  },
  {
   "date": "2026-09-22",
   "event": "DuckDB adapter ships built into dbt v2"
  }
 ],
 "relation": "neighbour",
 "relation_note": "dbt is where data teams model finance after a connector loads it. It does not hold a record or keep books, so it is not a competitor; an owner could point dbt at ownpurse's record through DuckDB and build their own models over it.",
 "borrow": "Views as versioned code with tests attached; lineage from source to report; JSON artifacts describing every run; a local binary agents can drive; the open Agents Schema idea of a plain-SQL context layer for agents.",
 "avoid": "Rebuild-and-overwrite as the default, which loses what a table said yesterday; dependence on a warehouse bill; a split between an open core and a proprietary distribution that leaves users unsure which license they are on.",
 "sources": [
  {
   "title": "Fivetran and dbt Labs complete merger (press release, 1 June 2026)",
   "url": "https://www.fivetran.com/press/fivetran-dbt-labs-complete-merger-to-create-the-data-infrastructure-for-trusted-ai-agents"
  },
  {
   "title": "dbt Labs and Fivetran merge announcement (13 October 2025)",
   "url": "https://www.getdbt.com/blog/dbt-labs-and-fivetran-merge-announcement"
  },
  {
   "title": "dbt pricing",
   "url": "https://www.getdbt.com/pricing"
  },
  {
   "title": "About the dbt MCP server",
   "url": "https://docs.getdbt.com/docs/dbt-ai/about-mcp"
  },
  {
   "title": "dbt APIs overview",
   "url": "https://docs.getdbt.com/docs/dbt-cloud-apis/overview"
  },
  {
   "title": "dbt Labs acquires SDF Labs (14 January 2025)",
   "url": "https://www.getdbt.com/blog/dbt-labs-acquires-sdf-labs"
  },
  {
   "title": "dbt-fusion repository (license note)",
   "url": "https://github.com/dbt-labs/dbt-fusion"
  },
  {
   "title": "DuckDB now ships inside dbt v2 (22 September 2026)",
   "url": "https://duckdb.org/2026/09/22/dbt-fusion.html"
  },
  {
   "title": "dbt-mcp repository",
   "url": "https://github.com/dbt-labs/dbt-mcp"
  }
 ],
 "unverified": [
  "Investor list and the February 2022 Series D ($222 million at a reported $4.2 billion valuation) were not re-checked against a primary source",
  "Date of the Transform acquisition (reported as February 2023)",
  "HQ of the combined company after the merger",
  "Starter overage rates for models built beyond 15,000 per month",
  "G2, Capterra and Reddit ratings and counts (pages blocked)",
  "The '100,000 data teams' figure (June 2026) versus 'more than 10,000 customers' (October 2025) use different measures and were not reconciled"
 ],
 "researched": "2026-10-10"
}
---

## What it is

dbt is a framework for transforming data that is already in a warehouse. You write models as SQL select statements with YAML for tests and documentation, and dbt compiles them into tables and views, in dependency order, inside [Snowflake, BigQuery, Databricks, Postgres or DuckDB](https://docs.getdbt.com/docs/dbt-cloud-apis/overview). It began in 2016 at Fishtown Analytics in Philadelphia and became the standard way analytics engineers build reporting tables, including finance teams modelling revenue, close and board reporting on top of loaded accounting data.

On 1 June 2026 dbt Labs [completed its all-stock merger with Fivetran](https://www.fivetran.com/press/fivetran-dbt-labs-complete-merger-to-create-the-data-infrastructure-for-trusted-ai-agents), announced on 13 October 2025. George Fraser stays CEO and Tristan Handy is President. The [announcement](https://www.getdbt.com/blog/dbt-labs-and-fivetran-merge-announcement) gave combined ARR of about $600 million and more than 10,000 customers.

## How it works

There are two parts. dbt Core is a command-line tool, Apache 2.0, that runs anywhere. The hosted dbt platform adds a browser IDE, scheduling, a catalog with lineage, the Semantic Layer for metrics, and APIs. In June 2026 dbt Core v2.0 (alpha) open-sourced the Rust Fusion engine runtime, which came from the [SDF Labs acquisition](https://www.getdbt.com/blog/dbt-labs-acquires-sdf-labs) of 14 January 2025. Fusion first shipped in May 2025 mostly under [ELv2](https://github.com/dbt-labs/dbt-fusion), which upset parts of the community; v2 now comes as a proprietary 'dbt' build and an Apache 2.0 'dbt-oss' build, both free locally.

Since 22 September 2025... correction, 22 September 2026, [DuckDB ships inside dbt v2](https://duckdb.org/2026/09/22/dbt-fusion.html): point a profile at a local file and you build models on a laptop with no server, and dbt can write its own metadata as Parquet for DuckDB to query.

Models are rebuilt each run. History exists only where you ask for it, with snapshots that keep changed rows as SCD2.

## Data and access

Your code lives in your Git repository and your data in your warehouse, so custody is good. The [pricing page](https://www.getdbt.com/pricing) (read 10 October 2026) lists Developer free for one seat and 3,000 models built a month with no API access, Starter at $100 per user per month for up to five seats and 15,000 models, and custom Enterprise plans. APIs (Administrative REST, Discovery GraphQL, Semantic Layer GraphQL and JDBC) need Starter or above and authenticate with personal or service tokens.

## Agents

dbt is one of the more agent-ready tools in data. Its [MCP server](https://docs.getdbt.com/docs/dbt-ai/about-mcp) runs two ways. Self-hosted (`uvx dbt-mcp`, Apache 2.0) it can run `dbt build` and `dbt test`, generate code and query metrics. Remote, it is read-oriented: metadata, lineage, Semantic Layer. June 2026 added dbt Wizard for authoring and debugging, and an open Agents Schema that keeps metrics, lineage and docs in plain SQL tables as context for agents. No human approval gate on tool calls is documented; Git review and CI do that work in practice.

## Customers

dbt serves data teams at mid-market and enterprise companies; the combined company says more than 100,000 data teams. Practitioners praise tested, versioned SQL and the community. They complain about hosted-platform pricing, the Fusion license episode and migration work. Small businesses without a data engineer seldom use it: it presumes a warehouse and someone to run it. Review-site ratings could not be retrieved for this dossier.

## History

dbt is the clearest example of the modern data stack's answer to cross-domain data wrangling: load everything raw into one warehouse (Fivetran), then model it in code (dbt). The merger puts both halves in one company. It works for enterprises with data teams and costs them a warehouse bill, connector bills and seats.

## For ownpurse

dbt is a neighbour, not a rival. It shows that views written as tested code, with lineage, win over hand-built reports, which matches ownpurse's rule that every view is a query. Its weakness for ownpurse's purposes is overwrite-by-default: yesterday's model is gone unless you snapshot it. Because dbt v2 now builds against a local DuckDB file, an owner could point dbt at ownpurse's record and keep their own models in Git. Exporting facts in a shape dbt and DuckDB read easily is cheap and fits sovereignty.
