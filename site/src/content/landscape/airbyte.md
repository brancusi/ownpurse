---
{
 "slug": "airbyte",
 "name": "Airbyte",
 "url": "https://airbyte.com",
 "segment": "connector-framework",
 "status": "active",
 "founded": "2020",
 "hq": "San Francisco, California (not confirmed on a primary page)",
 "backing": "Venture-backed; $181M raised in total per Airbyte's about page (read 2026-10-10). Rounds named in Airbyte's own blog: a $5M seed led by Accel and a $26M Series A led by Benchmark (both 2021); a $150M Series B at a reported $1.5B valuation (late 2021, figure from secondary sources). Y Combinator W20.",
 "tagline": "An open-source (ELv2) ELT platform that moves data from 600+ sources into warehouses, lakes and, since 2025-26, AI agents.",
 "scope": "Airbyte extracts data from APIs and databases and loads it into a destination you choose, on a schedule, with per-stream cursors and resumable checkpoints. It has a Xero source among its catalog. It does not model accounting, reconcile, or keep its own copy of your data as a product; since late 2025 it also sells Airbyte Agents, a hosted MCP endpoint and Python agent connectors that can read and write third-party APIs.",
 "domains": [
  "integration",
  "reporting"
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
   "note": "Data lands in a destination you own; Airbyte holds connector state, not the record, and Core can run on your own infrastructure."
  },
  "locality": {
   "score": 3,
   "note": "Airbyte Core is self-hostable and PyAirbyte runs connectors from a Python script; the platform itself is a server stack, not a desktop tool."
  },
  "history": {
   "score": 2,
   "note": "Append modes and raw tables keep extracted rows with _airbyte_extracted_at, but the newer Direct-Load drops the raw table and append-dedup keeps only the current row."
  },
  "openness": {
   "score": 3,
   "note": "Source-available: platform and connectors under ELv2, the protocol under MIT; Cloud, Enterprise and Agents are commercial."
  },
  "breadth": {
   "score": 3,
   "note": "600+ replication connectors cover books, payments, banks and SaaS, but it moves them side by side rather than joining them."
  },
  "autonomy": {
   "score": 1,
   "note": "Syncs run unattended, but replication never writes to the books; agent connectors can write, with no gate of Airbyte's own."
  },
  "agent_readiness": {
   "score": 3,
   "note": "Hosted MCP server at mcp.airbyte.ai, a Python agent SDK, CLI and API (docs read 2026-10-10)."
  },
  "integration_ease": {
   "score": 3,
   "note": "Connector Builder and low-code manifests are well documented, but self-hosting is a Kubernetes-class deployment and the Xero source is rated low success."
  },
  "market": {
   "score": 3,
   "note": "Data teams from startups to enterprise; Airbyte claims 25K companies and 20%+ of the Fortune 500."
  }
 },
 "api": {
  "public": true,
  "style": "REST API for the platform (connections, jobs, workspaces); connectors speak the Airbyte Protocol, newline-delimited JSON messages over stdout",
  "auth": "API keys / OAuth application credentials for the platform API; each connector uses the source's own auth (Xero: PKCE bearer token or OAuth client credentials plus a tenant id)",
  "limits": "Governed by each source's limits; the Xero source documents 5 concurrent calls, 60 a minute, 5,000 a day per tenant and retries on 429 after Retry-After. Max sync frequency 1 hour on Standard, 15 minutes on Plus, 5 minutes on Pro.",
  "cost": "Self-hosted Core: no licence fee. Cloud Standard from $20 a month (5 credits, $5 per extra credit); Plus from $189 a month for 40 credits up to $4,999 for 2,000; Pro and Enterprise Flex priced by Data Workers through sales (pricing page read 2026-10-10).",
  "export": "Everything it moves is already in your destination (warehouse, lake, files, database) in open formats; connector configs and state are exportable through the API.",
  "notes": "The Xero source is manifest-only (low-code), Marketplace not certified, beta, 21 streams, all incremental on UpdatedDateUTC; latest version 2.1.6 dated 2026-06-30. Its page lists availability as Enterprise Flex and PyAirbyte only, not Core, Standard, Plus or Pro."
 },
 "pricing": {
  "model": "Open-source Core free to self-host; Cloud billed by volume credits (Standard, Plus) or by capacity in Data Workers (Pro, Enterprise Flex); Airbyte Agents is a separate subscription",
  "entry": "$0 self-hosted; Cloud Standard from $20 a month with 5 credits",
  "top": "Plus at $4,999 a month for 2,000 credits; Pro and Enterprise Flex custom",
  "notes": "14-day trial on Standard and Plus with 400 credits. Airbyte Agents tiers are named Free, Individual, Team and Custom; their prices were not visible on the pages read."
 },
 "agents": {
  "mcp": "official",
  "ai_features": "Airbyte Agents: a hosted MCP server and MCP gateway, open-source agent connectors in the airbyte-agent-sdk Python package, a Context Store that copies a subset of connected data and refreshes hourly for search, plus an AI assist in the Connector Builder.",
  "agent_writes": "Yes, through agent connectors: Airbyte says they support write actions such as creating records and updating fields. Replication connectors only read.",
  "approval": "No human approval gate described by Airbyte; any confirmation step is left to the MCP client or the developer.",
  "notes": "The agent connector catalog read on 2026-10-10 lists finance connectors such as Stripe, Chargebee and Orb, but not Xero, QuickBooks or NetSuite."
 },
 "integration": {
  "scope": "600+ data replication connectors and 50+ agent connectors per Airbyte (about page, read 2026-10-10); destinations include warehouses, lakes, databases and vector stores.",
  "notes": "New connectors are built in the no-code Connector Builder or as YAML manifests on the low-code CDK; Python and Java CDKs remain for harder sources. Schema changes can be propagated or set to pause a connection."
 },
 "preference": {
  "scale": "Airbyte reports 25K companies, 2M pipelines synced daily, 27K community members, 1,300+ contributors and 20%+ of the Fortune 500 (about page, undated, read 2026-10-10); 22.2K GitHub stars and about 2,560 open issues on the main repo (2026-10-10).",
  "ratings": "Not retrieved: G2 and Capterra refused automated access on 2026-10-10.",
  "sentiment": "From Airbyte's own connector pages and the repo: breadth and the self-hosted option are the draw; long-tail connector reliability is the weak point (the Xero source is marked 'Low' sync success rate). Review-site themes could not be checked."
 },
 "timeline": [
  {
   "date": "2020-01",
   "event": "Founders join Y Combinator W20 as a pre-idea company"
  },
  {
   "date": "2020-07",
   "event": "GitHub repository created; MVP with 6 connectors shipped July to September 2020"
  },
  {
   "date": "2021",
   "event": "$5M seed led by Accel, then a $26M Series A led by Benchmark"
  },
  {
   "date": "2021",
   "event": "$150M Series B at a reported $1.5B valuation; total raised now stated as $181M"
  },
  {
   "date": "2025-10-15",
   "event": "Airbyte 2.0 released"
  },
  {
   "date": "2025-11-26",
   "event": "Agent connectors repository created, the start of Airbyte Agents"
  },
  {
   "date": "2026-06-30",
   "event": "Xero source 2.1.6, manifest-only since 2.1.0"
  },
  {
   "date": "2026-08",
   "event": "Airbyte Agents positioned as a data and context layer for AI agents, with a hosted MCP server"
  }
 ],
 "relation": "design-cousin",
 "relation_note": "Airbyte solves the same fetch problem ownpurse's connectors solve (cursors, checkpoints, rate limits, schema drift) for warehouses instead of a ledger. It is a pattern library and, through PyAirbyte, a possible way to run a long-tail source; it is not a competitor for the record.",
 "borrow": "State checkpoints that count only after the destination confirms the records before them; per-stream state rather than one global blob; a declarative manifest per source; _airbyte_meta as a per-row note of what was coerced or nulled, which maps to ownpurse's pressure reports; pausing a connection on a breaking schema change.",
 "avoid": "Dropping the raw record for speed (Direct-Load removes the raw JSON table); treating a Marketplace connector as production-grade (the Xero source is beta and rated low success); a refresh or reset that erases the raw tables; ELv2 terms if ownpurse ever wanted to embed and redistribute the platform.",
 "sources": [
  {
   "title": "Airbyte pricing",
   "url": "https://airbyte.com/pricing"
  },
  {
   "title": "Airbyte about us",
   "url": "https://airbyte.com/company/about-us"
  },
  {
   "title": "Airbyte Protocol",
   "url": "https://docs.airbyte.com/platform/understanding-airbyte/airbyte-protocol"
  },
  {
   "title": "Typing and deduping",
   "url": "https://docs.airbyte.com/platform/using-airbyte/core-concepts/typing-deduping"
  },
  {
   "title": "Xero source connector",
   "url": "https://docs.airbyte.com/integrations/sources/xero"
  },
  {
   "title": "Airbyte licenses",
   "url": "https://docs.airbyte.com/platform/developer-guides/licenses/"
  },
  {
   "title": "Airbyte Agents docs",
   "url": "https://docs.airbyte.com/ai-agents/"
  },
  {
   "title": "What is Airbyte Agents",
   "url": "https://airbyte.com/agentic-data/airbyte-agents"
  },
  {
   "title": "Agent connectors",
   "url": "https://docs.airbyte.com/ai-agents/connectors/"
  },
  {
   "title": "airbytehq/airbyte on GitHub",
   "url": "https://github.com/airbytehq/airbyte"
  },
  {
   "title": "Airbyte seed announcement",
   "url": "https://airbyte.com/blog/we-raised-a-5m-seed-round-with-accel-to-commoditize-data-integration"
  },
  {
   "title": "Airbyte Series A announcement",
   "url": "https://airbyte.com/blog/airbyte-raised-a-26m-series-a-to-change-how-data-is-moved"
  }
 ],
 "unverified": [
  "Series B amount ($150M), valuation ($1.5B) and exact date come from secondary sources; only the $181M total is on Airbyte's own page",
  "Exact months of the seed and Series A rounds",
  "Headquarters city",
  "Prices for the Airbyte Agents tiers (Free, Individual, Team, Custom)",
  "Per-Data-Worker price for Pro and Enterprise Flex",
  "G2, Capterra and Reddit ratings and sentiment (sites refused automated access)",
  "When the connectors moved from MIT to ELv2",
  "Whether the Xero source handles DELETED or VOIDED documents or archived contacts"
 ],
 "researched": "2026-10-10"
}
---

## What it is

Airbyte is a data integration platform: it reads from a source, such as a SaaS API or a database, and writes to a destination, such as Snowflake, BigQuery, Postgres or files. It started in Y Combinator's W20 batch, shipped an MVP with six connectors in mid-2020 and now claims 600+ replication connectors ([about](https://airbyte.com/company/about-us)). Since late 2025 it also sells Airbyte Agents, which gives agents live, authenticated access to third-party APIs ([docs](https://docs.airbyte.com/ai-agents/)).

## How it works

Every connector speaks the [Airbyte Protocol](https://docs.airbyte.com/platform/understanding-airbyte/airbyte-protocol): newline-delimited JSON messages such as RECORD, STATE, CATALOG and TRACE. The protocol credits Singer's spec as its starting point. State is opaque to the platform, kept per stream where possible, and a state is safe to resume from only after the destination has confirmed every record before it. That is what lets a failed sync resume from the last confirmed checkpoint rather than start over.

On the destination side, the classic design keeps a raw table of JSON blobs (with `_airbyte_raw_id` and `_airbyte_extracted_at`) beside a typed final table, and records per-row coercions in `_airbyte_meta` ([typing and deduping](https://docs.airbyte.com/platform/using-airbyte/core-concepts/typing-deduping)). Airbyte is phasing that out for Direct-Load, which types at insert and keeps one table per stream with no raw copy. A refresh or reset erases raw tables either way.

The [Xero source](https://docs.airbyte.com/integrations/sources/xero) is a good look at the long tail: 21 streams, all incremental on `UpdatedDateUTC`, manifest-only since 2.1.0, Marketplace (not certified), beta, version 2.1.6 dated 2026-06-30, with a sync success rate the page marks as low. It documents Xero's limits as 5,000 calls a day per tenant, which is higher than the 1,000 a day a new Starter app gets today, so treat that figure as dated.

## Data and access

What Airbyte moves is already yours: it sits in your destination in open formats. Airbyte Core is free to self-host; Cloud starts at $20 a month for 5 credits and Plus runs from $189 for 40 credits to $4,999 for 2,000 ([pricing](https://airbyte.com/pricing), read 2026-10-10). Pro and Enterprise Flex are priced by Data Workers through sales. The platform and connectors are under ELv2 and the protocol under MIT ([licenses](https://docs.airbyte.com/platform/developer-guides/licenses/)); ELv2 mainly stops you reselling Airbyte as a hosted ELT service.

## Agents

Airbyte Agents is a hosted MCP server at `mcp.airbyte.ai`, an MCP gateway, a Python SDK with open-source agent connectors, and a Context Store that copies a subset of data for search and refreshes hourly ([overview](https://airbyte.com/agentic-data/airbyte-agents)). Agent connectors can create and update records. Airbyte describes no approval step of its own. The catalog read on 2026-10-10 has Stripe, Chargebee and Orb, but not Xero, QuickBooks or NetSuite.

## Customers

Data teams, from startups using Cloud to large companies self-hosting. Airbyte reports 25K companies, 2M pipelines a day and 20%+ of the Fortune 500. The main repo has 22.2K stars and about 2,560 open issues. Review sites could not be read for this dossier.

## History

Airbyte's bet in 2020 was that connectors should be a commodity built in the open, after Fivetran had made them a paid moat. It raised a seed from Accel, a Series A led by Benchmark and a $150M Series B in 2021, $181M in all. Version 2.0 shipped in October 2025, and the agent pivot followed within weeks. It is the clearest case of a connector company deciding that agents, not warehouses, are the next destination.

## For ownpurse

Airbyte has already worked through most of the fetch problems ownpurse faces: cursors per stream, checkpoints that count only once the data is safely written, schema drift, rate limits. Borrow the confirm-then-advance rule and the per-row note of what was coerced. Do not copy its drift away from the raw record: ownpurse keeps the bytes, and that is the difference. Airbyte moves data into a warehouse that someone else then has to model; ownpurse keeps the source's own facts and checks the ledger against the source to the cent. PyAirbyte could still be a cheap way to pull a long-tail source into the drawer, as long as its output is treated as one more source with its own provenance.
