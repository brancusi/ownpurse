---
{
 "slug": "xtdb",
 "name": "XTDB",
 "url": "https://xtdb.com",
 "segment": "fact-ledger",
 "status": "active",
 "founded": "2019 (as Crux; rename to XTDB later, date unverified)",
 "hq": "London, UK (JUXT Ltd)",
 "backing": "Built by JUXT Ltd, a London engineering consultancy acquired by Grid Dynamics Holdings (NASDAQ: GDYN) on 26 September 2024. No separate venture funding found.",
 "tagline": "An open-source bitemporal SQL database that keeps every version of every row, by when it was true and when it was recorded.",
 "scope": "XTDB records two times for every row, valid time (when it was true, set by you) and system time (when the database learned it, set automatically), and lets you query any combination with SQL:2011 syntax over the Postgres wire protocol. It is general-purpose infrastructure aimed at audit and compliance; it has no accounting model, connectors or financial features.",
 "domains": [
  "ledger",
  "reporting"
 ],
 "customers_served": [
  "developers",
  "enterprise"
 ],
 "spectrum": {
  "custody": {
   "score": 4,
   "note": "You run it on your own machine or cloud; no vendor-hosted copy is required."
  },
  "locality": {
   "score": 4,
   "note": "Runs on your own machine from a Docker image and you connect with psql."
  },
  "history": {
   "score": 4,
   "note": "Bitemporal: valid time and system time on every row, as-of queries on both; ERASE exists for GDPR."
  },
  "openness": {
   "score": 4,
   "note": "MPL-2.0, source on GitHub (about 3.1k stars, October 2026)."
  },
  "breadth": {
   "score": 0,
   "note": "A database, not a finance product; no sources of its own."
  },
  "autonomy": {
   "score": 0,
   "note": "Not applicable: it does no accounting work."
  },
  "agent_readiness": {
   "score": 2,
   "note": "SQL over the Postgres wire lets any Postgres tool or MCP server reach it; no MCP or agent docs of its own."
  },
  "integration_ease": {
   "score": 3,
   "note": "Docker plus psql to start, drivers listed for 11 languages, Debezium ingestion, cloud marketplace listings."
  },
  "market": {
   "score": 3,
   "note": "Positioned for regulated enterprises (audit, compliance, financial services) and developers."
  }
 },
 "api": {
  "public": true,
  "style": "SQL over Postgres wire protocol and FlightSQL; XTQL; Arrow (ADBC); HTTP server moved to a labs module in 2.0",
  "auth": "As configured on your own deployment",
  "limits": "None imposed by a vendor; your hardware",
  "cost": "Free and open source; production support, consulting and training priced on request",
  "export": "Full: you own the log and object storage; SQL over every version",
  "notes": "Storage and compute are separated: hot data on compute nodes, history in object storage such as S3; a log (Kafka is one option) sequences transactions. Drivers listed for C, C#, Clojure, Elixir, Go, Java, Kotlin, Node.js, PHP, Python and Ruby."
 },
 "pricing": {
  "model": "Open source; paid support and consulting",
  "entry": "Free",
  "top": "Production support, price on request",
  "notes": "Also listed on AWS, Google Cloud and Azure marketplaces; no public price list."
 },
 "agents": {
  "mcp": "none",
  "ai_features": "None in the product as of October 2026. The site pitches a \"decision auditability layer for durable orchestration workflows and agentic workflows\"; a September 2026 post covers fraud detection with temporal SQL.",
  "agent_writes": "Through SQL like any client.",
  "approval": "None built in.",
  "notes": "Because it speaks the Postgres wire protocol, generic Postgres MCP servers should work against it (not tested). The repository carries AGENTS.md, CLAUDE.md and .mcp.json for its own contributors."
 },
 "integration": {
  "scope": "Postgres-compatible drivers and BI tools; native Debezium ingestion for change data from other databases; ATTACH (2.1) to integrate other sources.",
  "notes": "Postgres compatibility is its main integration story: existing tools work without a bespoke client."
 },
 "preference": {
  "scale": "No public customer or user counts found; GitHub shows about 3.1k stars and 193 forks (October 2026).",
  "ratings": "No G2, Capterra or Trustpilot listings found.",
  "sentiment": "Not checked against review sites or forums this pass. Developers generally credit its clear bitemporal model; the move from 1.x Datalog documents to 2.x SQL asked existing users to migrate."
 },
 "timeline": [
  {
   "date": "2019",
   "event": "Released as Crux by JUXT (month unverified)"
  },
  {
   "date": "2021",
   "event": "Renamed XTDB; 1.x line with Datalog over documents and two times (date unverified)"
  },
  {
   "date": "2024-09-26",
   "event": "Grid Dynamics announces its acquisition of JUXT"
  },
  {
   "date": "2025-06",
   "event": "XTDB 2.0 generally available: SQL, Postgres wire, Arrow columnar engine, object-store history"
  },
  {
   "date": "2025-12-01",
   "event": "XTDB 2.1.0 released (ATTACH to integrate other sources)"
  },
  {
   "date": "2026-07-13",
   "event": "XTDB 2.2 previewed with leader election; 2.2.0 betas through September 2026"
  }
 ],
 "relation": "design-cousin",
 "relation_note": "XTDB keeps the two times ownpurse needs, when a thing happened and when the record learned it, and shows how to query them with plain SQL. ownpurse needs the same two clocks for back-dated corrections from Xero and banks.",
 "borrow": "Two clocks on every fact (valid time set by the source, system time set by the log); SQL:2011-style FOR VALID_TIME and FOR SYSTEM_TIME AS OF queries; a query-wide basis setting so every view reads one snapshot; DELETE as a valid-time operation that keeps history, with a separate ERASE for legal removal; Postgres wire compatibility so ordinary tools can read the record.",
 "avoid": "A server and log to run for what should be one file a person holds; upserts by id that overwrite a valid-time range (fine for state, wrong for an evidence log, where two sources' claims must both stay); a rewrite of the query language between major versions.",
 "sources": [
  {
   "title": "XTDB",
   "url": "https://xtdb.com"
  },
  {
   "title": "XTDB docs",
   "url": "https://docs.xtdb.com/"
  },
  {
   "title": "XTDB key concepts",
   "url": "https://docs.xtdb.com/concepts/key-concepts.html"
  },
  {
   "title": "XTDB SQL transactions (ERASE, DELETE, backfill)",
   "url": "https://docs.xtdb.com/reference/main/sql/txs.html"
  },
  {
   "title": "XTDB SQL overview",
   "url": "https://docs.xtdb.com/quickstart/sql-overview.html"
  },
  {
   "title": "XTDB 2.0.0 release notes",
   "url": "https://github.com/xtdb/xtdb/releases/tag/v2.0.0"
  },
  {
   "title": "XTDB releases",
   "url": "https://github.com/xtdb/xtdb/releases"
  },
  {
   "title": "XTDB on GitHub",
   "url": "https://github.com/xtdb/xtdb"
  },
  {
   "title": "XTDB blog",
   "url": "https://xtdb.com/blog"
  },
  {
   "title": "Grid Dynamics news: joins forces with JUXT (26 September 2024)",
   "url": "https://www.griddynamics.com/news"
  },
  {
   "title": "JUXT",
   "url": "https://www.juxt.pro"
  }
 ],
 "unverified": [
  "Crux release date (2019) and the date of the rename to XTDB (2021): not confirmed from a primary source this pass",
  "Exact 2.0 GA date: the blog lists 5 June 2025, the GitHub release is dated 12 June (year confirmed by the blog)",
  "Whether generic Postgres MCP servers work against XTDB without changes",
  "Named customers and adoption figures",
  "Price of production support",
  "Deal terms of the Grid Dynamics acquisition of JUXT",
  "Review-site and forum sentiment (web search budget exhausted)"
 ],
 "researched": "2026-10-10"
}
---

## What it is

[XTDB](https://xtdb.com) is an open-source database, MPL-2.0, built by JUXT in London, for "development, reporting and compliance for immutable systems." Every row carries two times. Valid time is "a user-managed dimension" for backfilling and out-of-order updates; system time is set automatically and "represents the audit history of all changes to records" ([key concepts](https://docs.xtdb.com/concepts/key-concepts.html)). It builds on the bitemporal parts of SQL:2011.

## How it works

You write ordinary SQL. An UPDATE or DELETE does not destroy the old row; each change "sits alongside the previous versions of that row." DELETE is a valid-time operation; for data that must go, `ERASE` removes it across all valid and system time, for cases like GDPR ([transactions](https://docs.xtdb.com/reference/main/sql/txs.html)). Queries read the past with `FOR VALID_TIME AS OF` and `FOR SYSTEM_TIME AS OF`, or set a basis for a whole query with `SETTING DEFAULT SYSTEM_TIME` ([SQL overview](https://docs.xtdb.com/quickstart/sql-overview.html)). `BEGIN ... WITH (SYSTEM_TIME = ...)` lets you backfill history, as long as you do not go earlier than any committed transaction.

The [2.0 release](https://github.com/xtdb/xtdb/releases/tag/v2.0.0) separated storage from compute: hot data on compute nodes, history on object storage such as S3, an Apache Arrow columnar engine, nested data with schema inference, and "zero-cost snapshots."

## Data and access

You run it, so you hold everything. It speaks the Postgres wire protocol and FlightSQL, so `psql`, BI tools and drivers in eleven languages connect without a bespoke client ([docs](https://docs.xtdb.com/)). Debezium ingestion brings change data in from other databases. Support, consulting and training are sold on request; there is no price list, and it is on the AWS, Google Cloud and Azure marketplaces.

## Agents

There is no XTDB MCP server or AI feature. The site positions it as an audit layer "for durable orchestration workflows and agentic workflows," and the [blog](https://xtdb.com/blog) has a September 2026 post on fraud detection. Postgres compatibility is the practical agent story: anything that can talk to Postgres can query its history.

## Customers

JUXT pitches XTDB to regulated industries and financial services, but names no customer counts. The [repository](https://github.com/xtdb/xtdb) has about 3,100 stars and nearly 9,700 commits. There are no review-site listings.

## History

XTDB began as Crux in 2019, a Datalog database over documents with valid and transaction time, and was renamed XTDB in its 1.x line. [Grid Dynamics acquired JUXT](https://www.griddynamics.com/news) on 26 September 2024. Version 2, a rewrite around SQL, went generally available in June 2025; 2.1 followed on 1 December 2025 and 2.2 was in beta by September 2026.

## For ownpurse

XTDB is the clearest reference for the second clock. A Xero invoice dated in March but entered in May, or a bank line that posts days after the swipe, needs both times: when it happened and when the record learned it. XTDB shows those can be first-class and queried in SQL that people already know. Where ownpurse differs is the unit and the place: ownpurse keeps every source's claim side by side as facts with provenance, in one file on the owner's machine, where XTDB upserts a row by id into a server you run. Its Postgres-wire approach is worth a thought for ownpurse too: a read-only SQL surface over the record would let existing tools and agents query it with nothing new to learn.
