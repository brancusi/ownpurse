---
{
 "slug": "duckdb-motherduck",
 "name": "DuckDB and MotherDuck",
 "url": "https://duckdb.org",
 "segment": "data-platform",
 "status": "active",
 "founded": "DuckDB: 2018 at CWI Amsterdam (first release 2019, 1.0 on 3 June 2024). MotherDuck: 2022, Seattle.",
 "hq": "DuckDB: Amsterdam (DuckDB Foundation; DuckLabs, now an AWS subsidiary). MotherDuck: Seattle, Washington.",
 "backing": "DuckDB's IP is held by the non-profit DuckDB Foundation. DuckLabs (formerly DuckDB Labs), the core developers, announced on 26 August 2026 that it is joining AWS as a wholly owned subsidiary; projects stay MIT. MotherDuck is venture-backed: reported $47.5 million (2022) and $52.5 million (2023), about $100 million in total; it acquired Tower Computing in August 2026.",
 "tagline": "An embedded, open-source analytical database in a single file (DuckDB), plus a hosted serverless service built on it (MotherDuck).",
 "scope": "DuckDB is an in-process SQL engine for analytics: it runs inside your program or CLI, stores a database in one file, and reads Parquet, CSV, JSON, Iceberg, Delta and other databases (SQLite, Postgres) in place. MotherDuck hosts DuckDB in the cloud with sharing, scheduled Python jobs (Flights), dashboards (Dives) and an MCP server. Neither keeps books, ingests from SaaS APIs on its own, or knows anything about accounting.",
 "domains": [
  "reporting",
  "integration"
 ],
 "customers_served": [
  "developers",
  "smb",
  "mid-market",
  "freelancer"
 ],
 "spectrum": {
  "custody": {
   "score": 4,
   "note": "A DuckDB database is a file on your own disk; MotherDuck is optional and data can be copied back to a local file at any time."
  },
  "locality": {
   "score": 4,
   "note": "Runs in-process on your own machine with no server; MotherDuck is the cloud option, not a requirement."
  },
  "history": {
   "score": 2,
   "note": "Plain DuckDB tables hold current state; DuckLake 1.0 adds snapshots and time travel, and MotherDuck keeps snapshots up to 90 days on Business."
  },
  "openness": {
   "score": 4,
   "note": "DuckDB, DuckLake and Quack are MIT, governed by a non-profit foundation; MotherDuck's service is closed but its local MCP server is MIT."
  },
  "breadth": {
   "score": 1,
   "note": "A general engine with no financial domain of its own; it reads many file formats and databases but holds no books, banks or documents by itself."
  },
  "autonomy": {
   "score": 0,
   "note": "It answers queries; it does nothing to anyone's books on its own."
  },
  "agent_readiness": {
   "score": 4,
   "note": "Official MCP servers (hosted read-write and local MIT), a CLI agent mode with compact output and JSON errors (October 2026), and published Claude Code skills."
  },
  "integration_ease": {
   "score": 4,
   "note": "One binary or library with clients for Python, R, Java, Node, Rust, Go, C and Wasm; it can attach a SQLite file directly."
  },
  "market": {
   "score": 2,
   "note": "Developers and data teams from solo analysts to mid-market; MotherDuck has a free tier and a $250 per month business plan."
  }
 },
 "api": {
  "public": true,
  "style": "SQL through embedded client libraries (C, C++, Python, R, Java/JDBC, Node, Rust, Go, Wasm, ODBC, ADBC) and a CLI; Quack, an HTTP client-server protocol (May 2026, extension; production release planned with v2.0); MotherDuck adds the same clients over an md: connection plus a hosted MCP endpoint.",
  "auth": "DuckDB: none locally; Quack uses a generated token by default with pluggable auth. MotherDuck: OAuth for interactive clients, bearer tokens and service-account tokens for backends.",
  "limits": "DuckDB has no rate limits; one process writes a file at a time unless you use Quack or DuckLake for concurrent writers. MotherDuck limits by plan (Lite: 3 users, 10 GB, 10 hours of Pulse compute a month) and a default soft limit of 10 AI units a day.",
  "cost": "DuckDB is free. MotherDuck: Lite free; Business $250 per org per month plus usage (Pulse $0.60 per CU-hour, Standard $2.40 per hour up to Giga $24 per hour, storage $0.04 per GB-month, AI functions $1 per AI unit); data on your laptop or your own bucket is not charged. Egress terms are not stated on the pricing page.",
  "export": "COPY TO and EXPORT DATABASE write Parquet, CSV or JSON; the native file is itself portable and backward compatible since v0.10. MotherDuck data can be copied to a local DuckDB file.",
  "notes": "The storage format reads older files from v0.10 onward; forward compatibility is best effort. Data-at-rest encryption arrived in November 2025."
 },
 "pricing": {
  "model": "DuckDB open source and free; MotherDuck a platform fee plus metered compute, storage and AI units.",
  "entry": "DuckDB free; MotherDuck Lite free with 10 GB storage and 10 hours of Pulse compute a month, no card (pricing page, read 10 October 2026).",
  "top": "MotherDuck Enterprise: custom pricing, unlimited users, AWS Marketplace private offers.",
  "notes": "MotherDuck Business is $250 per org per month plus usage, up to 10 internal users, 90-day snapshots, a 7-day trial. Commercial support for open-source DuckDB was added in September 2026."
 },
 "agents": {
  "mcp": "official",
  "ai_features": "MotherDuck: hosted MCP server with Dives (interactive visualizations), Flights (scheduled Python jobs, from the Tower acquisition), Guides (written context for agents), a SQL assistant and AI SQL functions. DuckDB: CLI agent mode (October 2026) that prints compact Markdown, caps results, adds a result hash and reports errors as JSON; DuckDB skills for Claude Code (September 2026).",
  "agent_writes": "Yes. MotherDuck's hosted MCP exposes read-only and read-write SQL tools, and the local MIT server can open any DuckDB file; an agent with a shell can write to a DuckDB file directly.",
  "approval": "No approval gate beyond what the client provides; MotherDuck documents how to restrict an assistant to read-only, and the local server can open databases read-only. Writes are whatever SQL the agent sends.",
  "notes": "MotherDuck reported that agent queries overtook human queries in February 2026 and ran 29 times more queries than humans in a recent month, with each organization averaging twice as many agent users as human users (September 2026)."
 },
 "integration": {
  "scope": "Reads Parquet, CSV, JSON, Excel, Arrow, Iceberg, Delta, Lance and Vortex; attaches SQLite, Postgres and MySQL; DuckLake for lakehouse tables; built into dbt v2; MotherDuck integrates with BI tools, dbt and MCP clients.",
  "notes": "For a single-file record the key point is that DuckDB can attach a SQLite file and query it in place, so analytics need no copy and no server."
 },
 "preference": {
  "scale": "DuckDB: about 42,000 GitHub stars (October 2026); MotherDuck says users run millions of serverless DuckDB instances each week (August 2026). No customer count published.",
  "ratings": "G2, Capterra and Reddit could not be retrieved for this dossier; no rating is given.",
  "sentiment": "Developers praise speed on a laptop, zero setup, the single file and broad format support. Common complaints are the single-writer limit, memory pressure on large joins, and storage-format churn before 1.0. For MotherDuck, users like the simple hybrid local-and-cloud model; open questions after the AWS news are about its independence."
 },
 "timeline": [
  {
   "date": "2018-06",
   "event": "DuckDB repository created at CWI Amsterdam by Mark Raasveldt and Hannes Muhleisen"
  },
  {
   "date": "2019",
   "event": "First DuckDB release"
  },
  {
   "date": "2022",
   "event": "MotherDuck founded by Jordan Tigani with DuckDB Labs as a co-founder; first round reported at $47.5 million"
  },
  {
   "date": "2023",
   "event": "MotherDuck raises a reported $52.5 million"
  },
  {
   "date": "2024-06-03",
   "event": "DuckDB 1.0 released"
  },
  {
   "date": "2024-12",
   "event": "MotherDuck MCP server repository published"
  },
  {
   "date": "2025-05-27",
   "event": "DuckLake announced"
  },
  {
   "date": "2025-09-16",
   "event": "DuckDB 1.4.0 LTS"
  },
  {
   "date": "2025-11-19",
   "event": "Data-at-rest encryption in DuckDB"
  },
  {
   "date": "2026-03-09",
   "event": "DuckDB 1.5.0"
  },
  {
   "date": "2026-04-13",
   "event": "DuckLake 1.0 reaches production readiness"
  },
  {
   "date": "2026-05-12",
   "event": "Quack client-server protocol introduced"
  },
  {
   "date": "2026-08-25",
   "event": "MotherDuck acquires Tower Computing, which powers Flights"
  },
  {
   "date": "2026-08-26",
   "event": "DuckLabs announces it will join AWS as a subsidiary; projects stay MIT under the foundation"
  },
  {
   "date": "2026-09-02",
   "event": "DuckDB v2.0-dev available for testing"
  },
  {
   "date": "2026-10-09",
   "event": "Agent mode in the DuckDB CLI"
  }
 ],
 "relation": "infrastructure",
 "relation_note": "DuckDB is the analytics engine that sits beside a single-file record: it can attach ownpurse's SQLite file and query it in place, on the owner's machine, with no server. MotherDuck shows what the hosted version of that idea looks like.",
 "borrow": "Agent mode: compact output, capped results, a result hash and JSON errors, close to AXI; a native file that later versions always read; a database that holds only views over files; DuckLake's catalog-in-SQL with snapshots; the hybrid model where local is complete and cloud is optional.",
 "avoid": "Treating analytics tables as the record: DuckDB tables overwrite, and an analytical copy can drift from the source of truth. Do not move the hash-chained log into a format whose forward compatibility is best effort.",
 "sources": [
  {
   "title": "Why DuckDB",
   "url": "https://duckdb.org/why_duckdb"
  },
  {
   "title": "DuckDB news index",
   "url": "https://duckdb.org/news/"
  },
  {
   "title": "DuckLabs to join AWS, projects to remain open source (26 August 2026)",
   "url": "https://duckdb.org/2026/08/26/ducklabs-to-join-aws.html"
  },
  {
   "title": "Quack: the DuckDB client-server protocol (12 May 2026)",
   "url": "https://duckdb.org/2026/05/12/quack-remote-protocol.html"
  },
  {
   "title": "Agent mode in the DuckDB CLI (9 October 2026)",
   "url": "https://duckdb.org/2026/10/09/agent-mode"
  },
  {
   "title": "DuckDB now ships inside dbt v2 (22 September 2026)",
   "url": "https://duckdb.org/2026/09/22/dbt-fusion.html"
  },
  {
   "title": "DuckDB storage format and compatibility",
   "url": "https://duckdb.org/docs/current/internals/storage.html"
  },
  {
   "title": "MotherDuck pricing",
   "url": "https://motherduck.com/product/pricing/"
  },
  {
   "title": "MotherDuck MCP server docs",
   "url": "https://motherduck.com/docs/sql-reference/mcp/"
  },
  {
   "title": "DuckDB outgrows its nest (MotherDuck, August 2026)",
   "url": "https://motherduck.com/blog/duckdb-amazon/"
  },
  {
   "title": "MotherDuck acquires Tower",
   "url": "https://motherduck.com/blog/motherduck-acquires-tower/"
  },
  {
   "title": "Agents don't query like humans do (MotherDuck, September 2026)",
   "url": "https://motherduck.com/blog/agents-dont-query-like-humans-do/"
  },
  {
   "title": "MotherDuck about us",
   "url": "https://motherduck.com/about-us/"
  },
  {
   "title": "DuckDB (Wikipedia)",
   "url": "https://en.wikipedia.org/wiki/DuckDB"
  },
  {
   "title": "duckdb/duckdb on GitHub",
   "url": "https://github.com/duckdb/duckdb"
  }
 ],
 "unverified": [
  "MotherDuck funding amounts ($47.5 million in 2022, $52.5 million in 2023), lead investors and any valuation come from Wikipedia's citations, not a primary source; later rounds, if any, were not found",
  "Terms and price of the AWS deal for DuckLabs",
  "Exact date the Tower acquisition closed (the MotherDuck post is dated 25 August 2026)",
  "Whether MotherDuck holds equity in or has a commercial agreement with DuckLabs after the AWS deal",
  "MotherDuck egress charges (not stated on the pricing page)",
  "G2, Capterra and Reddit ratings and counts (pages blocked)",
  "MotherDuck customer count and revenue"
 ],
 "researched": "2026-10-10"
}
---

## What it is

DuckDB is an analytical SQL database that runs inside your process, the way SQLite does for transactions. Mark Raasveldt and Hannes Muhleisen built it at CWI in Amsterdam from 2018; 1.0 shipped on 3 June 2024. It is [MIT licensed](https://duckdb.org/why_duckdb), and its IP sits with the non-profit DuckDB Foundation. A database is a single file. It also reads Parquet, CSV, JSON, Iceberg and Delta in place, and attaches SQLite and Postgres databases.

MotherDuck, founded in 2022 in Seattle by Jordan Tigani with DuckDB Labs as a co-founder ([about](https://motherduck.com/about-us/)), runs DuckDB as a serverless cloud service. You can query a local file and a cloud database in one statement.

## How it works

DuckDB is a columnar, vectorized engine with no server and no dependencies. Recent work stretches it beyond one process: [Quack](https://duckdb.org/2026/05/12/quack-remote-protocol.html) (May 2026) is an HTTP client-server protocol that lets several processes write the same tables; [DuckLake 1.0](https://duckdb.org/news/) (April 2026) keeps a lakehouse catalog in SQL with snapshots and time travel. DuckDB has read files from older versions since v0.10, but [forward compatibility is best effort](https://duckdb.org/docs/current/internals/storage.html). Version 2.0 is in development (preview August 2026).

On 26 August 2026, [DuckLabs announced it will join AWS](https://duckdb.org/2026/08/26/ducklabs-to-join-aws.html) as a subsidiary. DuckDB, DuckLake and Quack stay MIT under the foundation, which is adding a stakeholder advisory board. MotherDuck's [response](https://motherduck.com/blog/duckdb-amazon/) argues Amazon has reason to keep DuckDB open, and it began selling support for open-source DuckDB.

## Data and access

With DuckDB you hold the file; there is nothing to pay and no rate limit. MotherDuck's [pricing](https://motherduck.com/product/pricing/) (read 10 October 2026): Lite free with 10 GB and 10 hours of compute a month; Business $250 per org per month plus usage, Pulse compute at $0.60 per CU-hour, storage $0.04 per GB-month, and no charge for data on your laptop or in your own bucket. Export is plain SQL: `COPY TO` Parquet or CSV, or `EXPORT DATABASE`.

## Agents

This is one of the most agent-ready stacks in data. MotherDuck's [hosted MCP server](https://motherduck.com/docs/sql-reference/mcp/) offers read-only and read-write SQL, Dives (charts), Flights (scheduled Python jobs, from its [Tower acquisition](https://motherduck.com/blog/motherduck-acquires-tower/)) and Guides (written context for agents); the local server is MIT. The DuckDB CLI gained an [agent mode](https://duckdb.org/2026/10/09/agent-mode) on 9 October 2026: compact Markdown tables, capped results, a result hash and JSON errors, which cut tokens read by about 59% in its test. MotherDuck [reports](https://motherduck.com/blog/agents-dont-query-like-humans-do/) that agents now run 29 times more queries than humans on its service. There is no approval gate on writes beyond read-only modes.

## Customers

DuckDB has about 42,000 GitHub stars (October 2026) and is used by solo analysts through to large data teams. Users praise laptop speed, zero setup and format coverage, and complain about the single-writer limit and memory pressure on big joins. MotherDuck publishes no customer count. Review-site ratings could not be retrieved for this dossier.

## History

DuckDB is the strongest argument that analytics does not need a warehouse: most small and mid-sized datasets fit on a laptop. It changed what small teams can do compared with an enterprise warehouse stack, and since September 2026 it is the [built-in local target in dbt v2](https://duckdb.org/2026/09/22/dbt-fusion.html).

## For ownpurse

DuckDB is infrastructure, not a rival. It knows nothing about accounting, provenance or approval, and plain tables overwrite. But it can attach ownpurse's SQLite record and run analytical queries over the facts in place, on the owner's machine, which suits sovereignty exactly. Its agent mode is close to AXI and worth matching. The line to hold: DuckDB reads the record, it does not become the record.
