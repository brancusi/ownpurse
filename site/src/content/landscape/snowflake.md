---
{
 "slug": "snowflake",
 "name": "Snowflake",
 "url": "https://www.snowflake.com",
 "segment": "data-platform",
 "status": "active",
 "founded": "2012",
 "hq": "Menlo Park, California (per Wikipedia; Snowflake had no formal headquarters for a period after 2020)",
 "backing": "Public on the NYSE (SNOW) since September 2020, when its IPO raised $3.4 billion. Before that, private rounds up to $479 million (February 2020). In September 2026 it priced $3.75 billion of 0.00% convertible senior notes.",
 "tagline": "A cloud data warehouse, now called an AI data cloud, where large companies land and query data from every system they run.",
 "scope": "Snowflake stores and queries structured and semi-structured data at any scale, shares data between accounts, and runs managed AI agents (Cortex, Snowflake CoWork) over it. It holds copies of data loaded from ERPs, CRMs, banks and billing systems through connectors, Openflow, or third-party ETL. It is not a ledger: it has no chart of accounts, no double-entry rules and no reconciliation to a source unless a customer builds them.",
 "domains": [
  "reporting",
  "integration"
 ],
 "customers_served": [
  "enterprise",
  "mid-market",
  "developers"
 ],
 "spectrum": {
  "custody": {
   "score": 1,
   "note": "The customer owns the data under contract and can unload it, but by default it sits in Snowflake-managed storage; Iceberg tables on your own bucket raise this."
  },
  "locality": {
   "score": 0,
   "note": "Cloud only, on AWS, Azure or GCP; there is no self-hosted or local edition."
  },
  "history": {
   "score": 2,
   "note": "Time Travel keeps prior table states for 1 day (Standard) up to 90 days (Enterprise and above), then 7 days of Fail-safe only Snowflake can use; not a permanent history."
  },
  "openness": {
   "score": 2,
   "note": "Closed engine, but a documented SQL REST API, drivers for every language, full unload and Apache Iceberg read and write."
  },
  "breadth": {
   "score": 3,
   "note": "Can hold data from every domain at once via native, zero-copy and marketplace connectors, but the customer must bring and model each source."
  },
  "autonomy": {
   "score": 1,
   "note": "Agents query and summarise; custom tools can run stored procedures, but nothing writes to anyone's books on its own."
  },
  "agent_readiness": {
   "score": 4,
   "note": "Snowflake-managed MCP server with OAuth, per-tool RBAC and read-only SQL by default, plus Cortex Agents, PATs and a CLI."
  },
  "integration_ease": {
   "score": 3,
   "note": "Well documented and supported by almost every ETL and BI tool, but needs an account, roles, warehouses and cost controls set up."
  },
  "market": {
   "score": 4,
   "note": "Enterprise-led: 829 Forbes Global 2000 customers and 828 customers above $1 million a year as of July 31, 2026."
  }
 },
 "api": {
  "public": true,
  "style": "SQL over drivers (JDBC, ODBC, Python, Go, Node and others) and a REST SQL API; managed MCP server for agents",
  "auth": "OAuth (Snowflake or external IdP such as Okta or Entra ID), key-pair, and programmatic access tokens",
  "limits": "MCP tool responses truncated at 250 KB for SQL and generic tools; up to 50 tools per MCP server; recursion depth 10 for chained calls. Throughput is governed by warehouse size and spend rather than call quotas.",
  "cost": "Consumption: compute in credits (per-second billing, 60-second minimum per warehouse start; a Small warehouse uses 2 credits an hour), storage per TB per month, and egress charged per TB across regions or clouds. Loading data in is free. Snowflake's own worked example uses $2 per credit and $23 per TB; real list prices vary by edition and region and are in the credit consumption table.",
  "export": "Everything: COPY INTO unloads tables to CSV, JSON or Parquet in cloud storage, and Iceberg tables keep data in an open format readable by other engines.",
  "notes": "The API surface is large and stable. Cost is the constraint, not access: every query an agent runs spends credits."
 },
 "pricing": {
  "model": "Usage-based credits for compute plus storage and transfer, on demand or prepaid capacity",
  "entry": "Standard edition on demand; a free trial exists (trial credit amount not confirmed this session)",
  "top": "Virtual Private Snowflake, priced through sales",
  "notes": "Editions are Standard, Enterprise, Business Critical and Virtual Private Snowflake. Cortex AI features are billed in credits or tokens on a separate table. Exact per-credit list prices could not be read from the PDF this session."
 },
 "agents": {
  "mcp": "official",
  "ai_features": "Cortex Agents, Cortex Analyst (text to SQL over semantic views), Cortex Search, Snowflake CoWork (the business-user agent app, formerly Snowflake Intelligence per its app link, not confirmed), dynamic model routing (August 2026).",
  "agent_writes": "Yes, if granted: the MCP SQL tool can be configured beyond its read-only default, and custom tools run any UDF or stored procedure the role allows.",
  "approval": "No built-in human approval gate for writes; control is by roles, least-privilege grants, budgets and per-user credit quotas.",
  "notes": "Snowflake advises not putting raw SQL execution on the same MCP server as an agent because it bypasses the agent's governance. The read-only-by-default SQL tool is the pattern ownpurse already follows."
 },
 "integration": {
  "scope": "Native and zero-copy connectors for SAP, Workday, Salesforce Data Cloud, ServiceNow, Google Analytics, Kafka, MySQL, PostgreSQL and SharePoint; Openflow for ingestion; 3,400 marketplace listings (July 31, 2025); LSEG financial data (expanded September 2026). Xero, QuickBooks and NetSuite are not on the native connector index; they arrive through third-party ETL.",
  "notes": "Small-business accounting systems reach Snowflake only through middleware such as Fivetran or Airbyte, and the warehouse then holds a current-state copy unless the pipeline keeps history."
 },
 "preference": {
  "scale": "More than 14,500 customers; 828 with trailing 12-month product revenue above $1 million; 829 Forbes Global 2000 customers; Q2 FY2027 revenue $1.55 billion, up 35% (quarter ended July 31, 2026). FY2026 revenue $4.684 billion. 4.1% of all Stack Overflow 2025 survey respondents used it.",
  "ratings": "G2, Capterra and TrustRadius pages returned 403 this session, so no ratings are given.",
  "sentiment": "Not checked against review sites this session. Net revenue retention of 126% (July 2026) shows existing customers keep spending more. Cost predictability under credit billing is the complaint most often raised in public discussion, but this was not verified from a review source here."
 },
 "timeline": [
  {
   "date": "2012-07-23",
   "event": "Founded in San Mateo by Benoît Dageville, Thierry Cruanes and Marcin Żukowski"
  },
  {
   "date": "2020-09",
   "event": "IPO on the NYSE as SNOW, raising $3.4 billion"
  },
  {
   "date": "2024-02-28",
   "event": "Sridhar Ramaswamy replaces Frank Slootman as CEO"
  },
  {
   "date": "2025-06",
   "event": "Acquires Crunchy Data (Postgres) for about $250 million"
  },
  {
   "date": "2026-01",
   "event": "Agrees to acquire Observe"
  },
  {
   "date": "2026-02-25",
   "event": "FY2026 results: revenue $4.684 billion, 733 customers above $1 million"
  },
  {
   "date": "2026-08-18",
   "event": "Announces dynamic model routing for AI workloads"
  },
  {
   "date": "2026-09-02",
   "event": "Q2 FY2027: revenue $1.55 billion (+35%), 828 customers above $1 million, FY guidance raised to $6.07 billion product revenue"
  },
  {
   "date": "2026-09-29",
   "event": "Prices $3.75 billion of 0.00% convertible senior notes"
  },
  {
   "date": "2026-09-30",
   "event": "Expands collaboration with LSEG on financial data"
  }
 ],
 "relation": "neighbour",
 "relation_note": "Snowflake is where a large company's finance data ends up for analysis. It answers 'one record' for enterprises with a central data team and a budget; ownpurse answers it for an owner with a laptop. They meet only if an ownpurse log is exported to a warehouse.",
 "borrow": "The managed MCP pattern: an explicit server object listing its tools, read-only SQL by default, per-tool grants, and a warning not to mix raw SQL with governed agents. Time Travel's AT/BEFORE syntax is a familiar shape for as-of queries.",
 "avoid": "Bounded history (90 days at most, then gone) presented as time travel; metered reads that make every agent question cost money; and making the owner build their own chart of accounts and reconciliation on top of raw copies.",
 "sources": [
  {
   "title": "Snowflake Q2 FY2027 results",
   "url": "https://investors.snowflake.com/news/news-details/2026/Snowflake-Reports-Financial-Results-for-the-Second-Quarter-of-Fiscal-2027/default.aspx"
  },
  {
   "title": "Snowflake Q4 and FY2026 results",
   "url": "https://investors.snowflake.com/news/news-details/2026/Snowflake-Reports-Financial-Results-for-the-Fourth-Quarter-and-Full-Year-of-Fiscal-2026/default.aspx"
  },
  {
   "title": "Snowflake press releases",
   "url": "https://www.snowflake.com/en/news/press-releases/"
  },
  {
   "title": "Snowflake pricing options",
   "url": "https://www.snowflake.com/en/pricing-options/"
  },
  {
   "title": "Understanding overall cost",
   "url": "https://docs.snowflake.com/en/user-guide/cost-understanding-overall"
  },
  {
   "title": "Time Travel",
   "url": "https://docs.snowflake.com/en/user-guide/data-time-travel"
  },
  {
   "title": "Snowflake-managed MCP server",
   "url": "https://docs.snowflake.com/en/user-guide/snowflake-cortex/cortex-agents-mcp"
  },
  {
   "title": "Snowflake CoWork docs",
   "url": "https://docs.snowflake.com/en/user-guide/snowflake-cortex/snowflake-intelligence"
  },
  {
   "title": "SQL API",
   "url": "https://docs.snowflake.com/en/developer-guide/sql-api/index"
  },
  {
   "title": "Iceberg tables",
   "url": "https://docs.snowflake.com/en/user-guide/tables-iceberg"
  },
  {
   "title": "Snowflake connectors index",
   "url": "https://other-docs.snowflake.com/en/connectors"
  },
  {
   "title": "About Snowflake",
   "url": "https://www.snowflake.com/en/company/overview/about-snowflake/"
  },
  {
   "title": "Snowflake Inc. (Wikipedia)",
   "url": "https://en.wikipedia.org/wiki/Snowflake_Inc."
  },
  {
   "title": "Stack Overflow Developer Survey 2025, technology",
   "url": "https://survey.stackoverflow.co/2025/technology"
  }
 ],
 "unverified": [
  "Exact on-demand price per credit by edition and storage price per TB (the credit consumption table PDF could not be decoded; $2 and $23 are Snowflake's illustrative example)",
  "Free trial length and credit amount",
  "HQ location and date of the move back to California (Wikipedia only)",
  "Crunchy Data price (~$250 million) and Observe deal, from Wikipedia, not primary releases",
  "That Snowflake CoWork is the renamed Snowflake Intelligence",
  "GA status and date of the managed MCP server",
  "G2, Capterra, TrustRadius ratings and review sentiment (pages blocked)",
  "Whether Fivetran's Xero connector lands in Snowflake with history mode"
 ],
 "researched": "2026-10-10"
}
---

## What it is

Snowflake is a cloud data warehouse, now marketed as an AI data cloud. Companies load data from their ERP, CRM, billing, HR and bank systems into it and query it with SQL. It was founded in [2012](https://en.wikipedia.org/wiki/Snowflake_Inc.) and has been public on the NYSE since September 2020. For an enterprise finance team it is the usual answer to having one place where all the numbers meet.

It is not a ledger. There is no chart of accounts, no double-entry check and no reconciliation to a source unless a data team builds them.

## How it works

Storage and compute are separate. Data sits in Snowflake-managed storage, or as [Apache Iceberg tables](https://docs.snowflake.com/en/user-guide/tables-iceberg) on your own S3, GCS or Azure bucket. You run queries on virtual warehouses billed in credits. [Time Travel](https://docs.snowflake.com/en/user-guide/data-time-travel) lets you query a table `AT` a timestamp or `BEFORE` a statement. Retention is 1 day on Standard and up to 90 days on Enterprise and above, followed by 7 days of Fail-safe that only Snowflake can recover from. That is recovery from mistakes, not a permanent record.

## Data and access

Access is open. Snowflake offers JDBC, ODBC and language drivers, a [REST SQL API](https://docs.snowflake.com/en/developer-guide/sql-api/index) authenticated by OAuth or key pair, and full unload to CSV, JSON or Parquet. Cost is the constraint. [Compute is billed per second](https://docs.snowflake.com/en/user-guide/cost-understanding-overall) with a 60-second minimum each time a warehouse starts; a Small warehouse uses 2 credits an hour. Storage is billed per TB per month, and egress across regions or clouds is billed per TB. Snowflake's worked example uses $2 per credit and $23 per TB; actual list prices depend on edition and region. Loading data is free.

The [connector index](https://other-docs.snowflake.com/en/connectors) covers SAP, Workday, Salesforce, ServiceNow and databases. Xero, QuickBooks and NetSuite are not on it, so small-business books reach Snowflake through middleware.

## Agents

This is where Snowflake is strong. A [Snowflake-managed MCP server](https://docs.snowflake.com/en/user-guide/snowflake-cortex/cortex-agents-mcp) is an object you create and list tools on: Cortex Agents, Cortex Analyst (text to SQL over semantic views), Cortex Search, SQL execution and custom UDFs. OAuth is the default, each tool needs its own grant, and the SQL tool is read-only unless configured otherwise. Snowflake advises against putting raw SQL beside a governed agent because it bypasses the agent's rules. Snowflake CoWork, its business-user agent app, can [take actions through custom tools](https://docs.snowflake.com/en/user-guide/snowflake-cortex/snowflake-intelligence). Writes are governed by roles and budgets; there is no approval step for a person.

## Customers

As of July 31, 2026, Snowflake reported [more than 14,500 customers](https://investors.snowflake.com/news/news-details/2026/Snowflake-Reports-Financial-Results-for-the-Second-Quarter-of-Fiscal-2027/default.aspx), 828 spending more than $1 million a year, and 829 of the Forbes Global 2000. Quarterly revenue was $1.55 billion, up 35%, with net revenue retention of 126%. FY2026 revenue was [$4.684 billion](https://investors.snowflake.com/news/news-details/2026/Snowflake-Reports-Financial-Results-for-the-Fourth-Quarter-and-Full-Year-of-Fiscal-2026/default.aspx). In the [Stack Overflow 2025 survey](https://survey.stackoverflow.co/2025/technology), 4.1% of respondents used it. Review-site ratings could not be retrieved this session.

## History

From 2012 Snowflake sold the separation of storage and compute in the cloud, then grew through data sharing, a marketplace and, since 2023, AI. It bought Neeva (2023), [Crunchy Data](https://en.wikipedia.org/wiki/Snowflake_Inc.) for Postgres (June 2025), and agreed to buy Observe (January 2026). In [September 2026](https://www.snowflake.com/en/news/press-releases/) it raised $3.75 billion in convertible notes and expanded its work with LSEG on financial data. Its story is the enterprise version of the unification problem: pay a central team and a platform to copy everything into one place.

## For ownpurse

Snowflake shows what one record looks like when money is no object: every source copied in, governed by roles, queried by agents through a managed MCP server. It also shows the costs. History lasts weeks, not forever. Every read an agent makes is metered. Small-business accounting systems arrive only through someone else's pipeline, and the warehouse knows nothing about balancing. ownpurse can borrow the MCP pattern directly: explicit tool lists, read-only SQL by default, per-tool grants. Its gap is the owner Snowflake never serves: one person or a small firm, history that never expires, reads that cost nothing, and a ledger that proves itself against the source. An ownpurse log exported as Parquet or Iceberg could still feed a warehouse for a business that grows into one.
