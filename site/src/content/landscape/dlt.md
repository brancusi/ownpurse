---
{
 "slug": "dlt",
 "name": "dlt (dltHub)",
 "url": "https://dlthub.com",
 "segment": "connector-framework",
 "status": "active",
 "founded": "2022 (GitHub repository created 2022-01-26)",
 "hq": "Berlin, with offices in San Francisco, Montreal and Amsterdam",
 "backing": "Venture-backed by Bessemer Venture Partners, Foundation Capital and Dig Ventures; the site cites 'dltHub raises $8M led by Bessemer Venture Partners' (date not stated).",
 "tagline": "An open-source Python library that loads data from APIs and files into a destination such as a local DuckDB file, with a paid platform to run it.",
 "scope": "dlt extracts from REST APIs, databases and files, infers and evolves a schema, normalises nested JSON into tables and loads them with append, replace or merge (including scd2). It keeps incremental state and deduplicates at the cursor boundary. It is a pipeline library, not a ledger: it knows nothing of accounts or balances, and dltHub, the paid platform, adds hosted runtime, observability and agents.",
 "domains": [
  "integration"
 ],
 "customers_served": [
  "developers",
  "smb",
  "mid-market",
  "enterprise"
 ],
 "spectrum": {
  "custody": {
   "score": 3,
   "note": "The library runs in your process and loads into a store you own; the vendor holds nothing, but the result is a copy, not a record with provenance."
  },
  "locality": {
   "score": 4,
   "note": "pip install and run on your laptop into a local DuckDB file, no backend."
  },
  "history": {
   "score": 2,
   "note": "scd2 merge keeps versions with _dlt_valid_from/_dlt_valid_to and load ids are kept, but it is opt-in per table and not as-of over every attribute."
  },
  "openness": {
   "score": 4,
   "note": "Apache-2.0 library, 'forever free'; the platform is closed and paid."
  },
  "breadth": {
   "score": 3,
   "note": "Any REST API, SQL database or file, plus 10,100+ source context pages, but as generic tables."
  },
  "autonomy": {
   "score": 1,
   "note": "Loads run unattended and background agents diagnose failures; nothing writes to books."
  },
  "agent_readiness": {
   "score": 4,
   "note": "Docs built for coding agents, dlthub ai init installs skills, rules and a dlt-workspace-mcp server for Claude Code, Codex and Cursor."
  },
  "integration_ease": {
   "score": 3,
   "note": "Well documented and self-serve, but you write Python for each source."
  },
  "market": {
   "score": 2,
   "note": "Python data engineers from startups to large firms; 16,000+ companies claimed in production."
  }
 },
 "api": {
  "public": true,
  "style": "Python library (decorated sources and resources, a REST API source configured declaratively), a CLI, and an MCP server over the local workspace",
  "auth": "Source credentials in local secrets.toml or environment; no vendor account for the library",
  "limits": "None from dlt; the source's API limits apply",
  "cost": "Library free; dltHub $12,000 a month with 5,000 credits (1 credit = 1 runtime hour), 12-month minimum, per the pricing page on 2026-10-10 (the FAQ on the same page says $11,900)",
  "export": "Everything: data, schema and pipeline state live in your destination and working directory",
  "notes": "Incremental state is saved with the pipeline and in the destination; backfills with end_value are stateless and can run in parallel."
 },
 "pricing": {
  "model": "Free open-source library; platform billed as a monthly subscription with included runtime credits and tiered overage",
  "entry": "dlt: free (Apache-2.0); dltHub trial 14 days with $30 credits, no card",
  "top": "Enterprise: custom, 12-month minimum",
  "notes": "The pricing page is internally inconsistent on 2026-10-10: $12,000 vs $11,900 a month and 5,000 vs 500 included credits. Self-serve checkout and auto-pause are promised for Q3 2026."
 },
 "agents": {
  "mcp": "official",
  "ai_features": "Library and docs built 'from the ground up for LLMs and coding agents'; dlthub ai init installs skills, rules and the dlt-workspace-mcp server (tools include list_pipelines, get_table_schema, execute_sql_query); dltHub Context pages per source (auth, pagination, cursors, rate limits); background agents in public preview (8 October 2026) that diagnose failed jobs.",
  "agent_writes": "Coding agents scaffold and run pipelines locally; the MCP server can run SQL over the loaded data. Whether the MCP server is read-only is not stated.",
  "approval": "No built-in approval gate for data; code changes pass through the coding tool and the team's own review. Background agents start with diagnosis rather than autonomous fixes.",
  "notes": "A security rule tells the assistant never to ask for credentials in chat."
 },
 "integration": {
  "scope": "Verified OSS sources, a generic REST API source, SQL databases, files; destinations include DuckDB, Snowflake, BigQuery, Postgres and others; 10,100+ source context pages",
  "notes": "No verified Xero source was confirmed; a Xero pipeline would be written with the REST API source."
 },
 "preference": {
  "scale": "6M+ PyPI downloads a month and 16,000+ companies in production (vendor, 2026-10-10); dlt-hub/dlt has 5,950 GitHub stars and released 1.31.0 on 7 October 2026.",
  "ratings": "No review-site ratings confirmed.",
  "sentiment": "Not confirmed from review sites or forums (Reddit unreachable). The vendor's own testimonials praise the local DuckDB workspace; the main cost is that every source is code you maintain."
 },
 "timeline": [
  {
   "date": "2022-01-26",
   "event": "dlt-hub/dlt repository created"
  },
  {
   "date": "undated",
   "event": "dltHub raises $8M led by Bessemer Venture Partners"
  },
  {
   "date": "2026-08-06",
   "event": "dltHub adds agentic alerts, team workspaces and managed infrastructure"
  },
  {
   "date": "2026-10-07",
   "event": "dlt 1.31.0 released"
  },
  {
   "date": "2026-10-08",
   "event": "Background agents in public preview"
  }
 ],
 "relation": "kin",
 "relation_note": "dlt is the nearest open-source tool in spirit: a library you run on your machine that lands source data in a local file you hold. It stops at tables; ownpurse goes on to keep every version as facts and rebuild the books to the cent.",
 "borrow": "Deduplication at the incremental boundary (content hash by default, primary_key optional) with a closed start range; stateless backfills over half-open ranges that do not touch the live cursor; schema inferred and evolved, with agent-ready docs and an init command that installs skills, rules and an MCP server.",
 "avoid": "History as an opt-in merge strategy rather than the default; scd2 retiring rows absent from a partial extract unless a merge_key is set (only a completed sweep should retract); a pricing page that contradicts itself.",
 "sources": [
  {
   "title": "dltHub home page",
   "url": "https://dlthub.com"
  },
  {
   "title": "dltHub pricing",
   "url": "https://dlthub.com/pricing"
  },
  {
   "title": "dltHub about",
   "url": "https://dlthub.com/about"
  },
  {
   "title": "Incremental loading: cursor",
   "url": "https://dlthub.com/docs/general-usage/incremental/cursor"
  },
  {
   "title": "Merge loading",
   "url": "https://dlthub.com/docs/general-usage/merge-loading"
  },
  {
   "title": "Incremental loading overview",
   "url": "https://dlthub.com/docs/general-usage/incremental-loading"
  },
  {
   "title": "LLM-native workflow",
   "url": "https://dlthub.com/docs/dlt-ecosystem/llm-tooling/llm-native-workflow"
  },
  {
   "title": "dltHub blog",
   "url": "https://dlthub.com/blog"
  },
  {
   "title": "dlt-hub/dlt on GitHub",
   "url": "https://github.com/dlt-hub/dlt"
  }
 ],
 "unverified": [
  "Date and stage of the $8M Bessemer-led round, and any other funding",
  "Founding year of the company (2022 inferred from the repository)",
  "Whether dlt-workspace-mcp is read-only",
  "Correct dltHub price and included credits (page says both $12,000/5,000 and $11,900/500)",
  "Existence of a verified Xero source",
  "Review-site ratings and forum sentiment"
 ],
 "researched": "2026-10-10"
}
---

## What it is

dlt (data load tool) is an [Apache-2.0 Python library](https://github.com/dlt-hub/dlt) for loading data from APIs, databases and files into a destination. It runs anywhere Python runs, with no backend, and a local DuckDB file is a first-class destination. dltHub, the Berlin company behind it, sells a managed platform on top. The [home page](https://dlthub.com) claims 6M+ PyPI downloads a month and 16,000+ companies loading data with dlt in production (vendor figures, 10 October 2026).

What it is not: a ledger. It does not know an invoice from a contact, and it does not check that anything balances.

## How it works

You write a source in Python, or configure the generic REST API source, and dlt infers a schema, unpacks nested JSON into child tables and loads with a write disposition: append, replace or merge. Merge strategies include delete-insert, upsert and [scd2](https://dlthub.com/docs/general-usage/merge-loading), which keeps versions in `_dlt_valid_from` and `_dlt_valid_to`. By default scd2 retires a row that is missing from a load, unless you set a natural `merge_key`.

The [incremental cursor](https://dlthub.com/docs/general-usage/incremental/cursor) is what the earlier survey singled out. dlt stores the last cursor value in pipeline state and, because the start of the range is closed by default, deduplicates rows at the boundary by a content hash, or by a `primary_key` you name. Backfills with an `end_value` are stateless, half-open ranges that never move the live cursor, so they can run beside regular loads.

## Data and access

The library is free. Data, schema and state live where you put them. The [dltHub platform](https://dlthub.com/pricing) costs $12,000 a month with 5,000 runtime credits (one credit is one hour) on a 12-month minimum, with tiered overage from $1.00 a credit, and a 14-day trial with $30 in credits. The same page also says $11,900 and 500 credits, so check before quoting.

## Agents

This is where dlt is furthest along of the three connector frameworks. Its README says the library and docs are built for LLMs and coding agents. [`dlthub ai init`](https://dlthub.com/docs/dlt-ecosystem/llm-tooling/llm-native-workflow) detects Claude Code, Cursor or Codex and installs skills, rules and the `dlt-workspace-mcp` server (tools include `list_pipelines`, `get_table_schema` and `execute_sql_query`). dltHub Context publishes a page per source on auth, pagination, cursors and rate limits for an agent to read. Background agents that diagnose failed jobs entered public preview on [8 October 2026](https://dlthub.com/blog). There is no approval gate on data, because dlt only loads into your own store.

## Customers

Python data engineers, from single developers with a DuckDB file to teams loading into Snowflake (1,000+ companies, by the vendor's count). The repository has 5,950 stars and released 1.31.0 on 7 October 2026. Review-site ratings could not be confirmed.

## History

The repository dates from January 2022. The company is backed by Bessemer Venture Partners, Foundation Capital and Dig Ventures, and its site cites an $8M round led by Bessemer (date not stated). Through 2026 it moved from library to platform: team workspaces and managed infrastructure in August, background agents in October.

## For ownpurse

dlt is kin. It proves people want a library they run themselves that lands source data in a local file they hold, and that agents can write connectors against good per-source context. Borrow boundary deduplication and stateless backfills that do not move the cursor. Do not borrow scd2 retirement on a partial extract: in ownpurse only a completed, verified sweep may retract. ownpurse's difference is that history is the default for every attribute, and the books are rebuilt and checked rather than left as tables.
