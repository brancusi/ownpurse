---
{
 "slug": "databricks",
 "name": "Databricks",
 "url": "https://www.databricks.com",
 "segment": "data-platform",
 "status": "active",
 "founded": "2013",
 "hq": "San Francisco, California",
 "backing": "Private. Per Wikipedia: $10 billion at a $62 billion valuation (December 2024), $1 billion Series K at over $100 billion (August 2025), $4 billion-plus Series L at $134 billion (December 2025), and a round closed at $5 billion with a $190 billion post-money valuation (August 2026). Investors include Andreessen Horowitz, Franklin Templeton, Nvidia and Coatue.",
 "tagline": "A lakehouse platform that stores data as open-format tables in cloud storage and runs SQL, Spark, machine learning and agents over them.",
 "scope": "Databricks runs data engineering, warehousing (Databricks SQL), machine learning and AI agents over Delta Lake or Iceberg tables governed by Unity Catalog, and adds Postgres (Lakebase, from Neon). It ingests from SaaS and databases through Lakeflow Connect and shares data through an open protocol. Like Snowflake it is not a ledger: no accounting rules, chart of accounts or reconciliation unless a team builds them.",
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
   "score": 2,
   "note": "Table files sit in open formats in the customer's own cloud storage in the classic setup, so the data outlives the vendor; the control plane is still Databricks."
  },
  "locality": {
   "score": 1,
   "note": "Not self-hostable, but the classic data plane runs inside your own cloud account; the open-source parts (Spark, Delta) run anywhere."
  },
  "history": {
   "score": 2,
   "note": "Every Delta write is a numbered version with DESCRIBE HISTORY and VERSION/TIMESTAMP AS OF, but defaults keep 30 days of log and 7 days of files, and VACUUM removes old versions."
  },
  "openness": {
   "score": 3,
   "note": "Built on open source it created (Spark, Delta Lake, MLflow, Unity Catalog) and an open sharing protocol; the managed platform itself is closed."
  },
  "breadth": {
   "score": 3,
   "note": "Can hold every domain at once, with managed connectors for Salesforce, Workday and databases, but the customer brings and models each source."
  },
  "autonomy": {
   "score": 1,
   "note": "Agents answer questions and run jobs; nothing posts to anyone's books unattended."
  },
  "agent_readiness": {
   "score": 4,
   "note": "Managed MCP servers (Genie One, Databricks SQL), external and custom MCP registration under Unity Gateway, Unity Catalog permissions, a CLI and SDKs."
  },
  "integration_ease": {
   "score": 3,
   "note": "Well documented with SDKs and connectors; workspace, cloud account and catalog setup make it a team project, not a self-serve afternoon."
  },
  "market": {
   "score": 4,
   "note": "Over 20,000 organisations including 70% of the Fortune 500, per its site (2026)."
  }
 },
 "api": {
  "public": true,
  "style": "REST APIs, SQL (JDBC/ODBC, SQL Statement Execution), Python/Go/Java SDKs, CLI, managed MCP servers, open Delta Sharing (OpenSharing) protocol",
  "auth": "OAuth (user and service principal), personal access tokens, OIDC federation for sharing recipients",
  "limits": "Not confirmed this session; throughput depends on compute size and DBU spend.",
  "cost": "Consumption in DBUs, billed per second, on top of the customer's own cloud compute and storage in classic mode; committed-use discounts by contract. Sharing within a region has no egress cost; cross-region or cross-cloud may. Genie One and Genie Agents usage by users is free through January 31, 2027; Genie Code moved to pay-as-you-go on July 8, 2026.",
  "export": "Full: Delta and Iceberg tables are Parquet files plus a transaction log in your own storage, readable by Spark, DuckDB, pandas and others without Databricks.",
  "notes": "Of the large platforms, its data is the easiest to take with you, because the storage format is open and lives in your bucket."
 },
 "pricing": {
  "model": "Usage-based DBUs per workload type, plus cloud infrastructure; pay as you go or committed use",
  "entry": "Free Edition (serverless, quota-limited, for learning and personal use); 14-day free trial on paid products",
  "top": "Committed-use enterprise contracts, priced through sales",
  "notes": "Per-DBU list prices did not load from the pricing pages this session. Azure Databricks prices are set by Microsoft. Community Edition was retired in 2025 in favour of Free Edition."
 },
 "agents": {
  "mcp": "official",
  "ai_features": "Genie One (business-user questions over governed data), Genie Agents (configured by data teams), Genie Code (coding assistant), Genie Ontology semantic layer, Agent Bricks and Mosaic AI for building agents, Lakebase Postgres for agent state.",
  "agent_writes": "Yes, within Unity Catalog permissions: the Databricks SQL MCP and custom tools can run data engineering and DML.",
  "approval": "No human approval gate built in; governance through Unity Catalog grants, Unity Gateway credential management and monitoring, and budgets.",
  "notes": "Coding agents such as Claude Code, Codex and Cursor can add the managed MCPs directly. Agent Bricks was not confirmed on the pages read."
 },
 "integration": {
  "scope": "Lakeflow Connect managed connectors for Salesforce, Workday, HubSpot, Jira, Zendesk, Google Ads, Meta Ads, Slack and others; CDC from MySQL, PostgreSQL and SQL Server; Google Drive and SharePoint; 1,200+ partners. NetSuite, Xero and QuickBooks are not named on the Lakeflow Connect overview.",
  "notes": "Small-business accounting systems arrive through third-party ETL. Some SaaS connectors support only browser OAuth and cannot be created programmatically."
 },
 "preference": {
  "scale": "Over 20,000 organisations and 70% of the Fortune 500 (company site, 2026). Revenue run rate $5.4 billion (February 2026) and about $7 billion (August 2026), per Wikipedia citing company statements. About 10,000 employees (2026). 3.4% of Stack Overflow 2025 respondents used Databricks SQL.",
  "ratings": "G2 returned 403 this session; no ratings given.",
  "sentiment": "Not checked against review sites this session."
 },
 "timeline": [
  {
   "date": "2013",
   "event": "Founded in San Francisco by the creators of Apache Spark from UC Berkeley's AMPLab"
  },
  {
   "date": "2023-06",
   "event": "Acquires MosaicML (about $1.3 billion)"
  },
  {
   "date": "2024",
   "event": "Acquires Tabular, the company of Iceberg's creators (over $1 billion)"
  },
  {
   "date": "2024-12",
   "event": "Raises $10 billion at a $62 billion valuation"
  },
  {
   "date": "2025",
   "event": "Acquires Neon (Postgres), later Lakebase; retires Community Edition for Free Edition"
  },
  {
   "date": "2025-12",
   "event": "Series L of $4 billion-plus at $134 billion"
  },
  {
   "date": "2026-02-09",
   "event": "Reports a $5.4 billion revenue run rate, growing over 65%"
  },
  {
   "date": "2026-07-08",
   "event": "Genie Code moves to pay-as-you-go pricing"
  },
  {
   "date": "2026-08",
   "event": "Round closes at $5 billion, $190 billion post-money; about $7 billion run rate; acquires Electric"
  }
 ],
 "relation": "neighbour",
 "relation_note": "Databricks is the other place enterprise finance data lands. Its open table formats prove that a company can keep its analytical copy in files it controls, which is close to ownpurse's stance, but it serves data teams at large companies, not owners.",
 "borrow": "Numbered versions with DESCRIBE HISTORY and VERSION AS OF; a transaction log beside data files any engine can read; an open sharing protocol where recipients need no account; MCP tools registered and governed in one gateway.",
 "avoid": "Versions that silently expire (7-day file retention by default, VACUUM) under the name of time travel; consumption pricing on reads; and a platform whose setup needs a data team before the first question.",
 "sources": [
  {
   "title": "About Databricks",
   "url": "https://www.databricks.com/company/about-us"
  },
  {
   "title": "Databricks pricing",
   "url": "https://www.databricks.com/product/pricing"
  },
  {
   "title": "Databricks SQL pricing",
   "url": "https://www.databricks.com/product/pricing/databricks-sql"
  },
  {
   "title": "Delta table history and time travel",
   "url": "https://docs.databricks.com/aws/en/delta/history"
  },
  {
   "title": "MCP on Databricks",
   "url": "https://docs.databricks.com/aws/en/generative-ai/mcp/"
  },
  {
   "title": "Genie",
   "url": "https://docs.databricks.com/aws/en/genie/"
  },
  {
   "title": "Lakeflow Connect",
   "url": "https://docs.databricks.com/aws/en/ingestion/lakeflow-connect/"
  },
  {
   "title": "OpenSharing (Delta Sharing)",
   "url": "https://docs.databricks.com/aws/en/delta-sharing/"
  },
  {
   "title": "Databricks Free Edition",
   "url": "https://docs.databricks.com/aws/en/getting-started/free-edition"
  },
  {
   "title": "Databricks (Wikipedia)",
   "url": "https://en.wikipedia.org/wiki/Databricks"
  },
  {
   "title": "Stack Overflow Developer Survey 2025, technology",
   "url": "https://survey.stackoverflow.co/2025/technology"
  }
 ],
 "unverified": [
  "All funding rounds, valuations and run-rate figures (from Wikipedia; Databricks press release pages did not render this session)",
  "Per-DBU list prices for SQL, Jobs and serverless",
  "API rate limits",
  "Acquisition prices for MosaicML, Tabular and Neon",
  "Agent Bricks status and pricing",
  "G2 and other review ratings and sentiment",
  "Whether Free Edition allows commercial use"
 ],
 "researched": "2026-10-10"
}
---

## What it is

Databricks is a lakehouse platform. It keeps data as open-format tables (Delta Lake, or Apache Iceberg) in cloud object storage and runs SQL, Spark, machine learning and AI agents over them, all governed by Unity Catalog. Its founders created Apache Spark at UC Berkeley, and the company was [founded in 2013](https://www.databricks.com/company/about-us). Large finance teams use it the way others use Snowflake: as the place where ERP, CRM, billing and bank data meet.

It is not a ledger. Accounting rules, a chart of accounts and reconciliation are left to the customer.

## How it works

A Delta table is Parquet files plus a transaction log. Every write is a [numbered version](https://docs.databricks.com/aws/en/delta/history): `DESCRIBE HISTORY` lists who changed what and when, and `VERSION AS OF` or `TIMESTAMP AS OF` reads the table as it was. The defaults limit this. The log is kept for 30 days, data files for 7, and `VACUUM` deletes the files older versions need. Since Runtime 18.0, time travel past the file retention window is blocked. Longer history is possible, but it costs storage and someone has to configure it.

In the classic setup the files sit in your own cloud account. Databricks has also added Postgres, as Lakebase (from its Neon acquisition), for application and agent state.

## Data and access

Databricks has REST APIs, SQL drivers, SDKs, a CLI and an open sharing protocol, now documented as [OpenSharing](https://docs.databricks.com/aws/en/delta-sharing/), whose recipients can read shared tables with Spark, pandas or Power BI and no Databricks account. Because tables are open files in your bucket, bulk export is just reading them. Pricing is [consumption in DBUs](https://www.databricks.com/product/pricing), billed per second, on top of your own cloud bill, with discounts for committed use. Per-DBU list prices did not load this session. [Free Edition](https://docs.databricks.com/aws/en/getting-started/free-edition) is serverless and quota-limited, for learning and personal use.

[Lakeflow Connect](https://docs.databricks.com/aws/en/ingestion/lakeflow-connect/) has managed connectors for Salesforce, Workday, HubSpot, Jira and database change capture. Xero, QuickBooks and NetSuite are not named on it.

## Agents

Databricks offers [managed MCP servers](https://docs.databricks.com/aws/en/generative-ai/mcp/) for Genie One (natural-language questions grounded in its Genie Ontology semantic layer) and Databricks SQL. You can register external or custom servers under Unity Gateway, which manages credentials and monitors usage, and Unity Catalog permissions apply throughout. Claude Code, Codex and Cursor can add these tools directly. [Genie](https://docs.databricks.com/aws/en/genie/) usage by people is free through January 31, 2027; Genie Code moved to pay-as-you-go on July 8, 2026. Agents can write wherever their grants allow, and there is no approval step for a person.

## Customers

Databricks says it serves over 20,000 organisations, including 70% of the Fortune 500. Wikipedia, citing company statements, gives a $5.4 billion revenue run rate in February 2026 and about $7 billion in August 2026, with about 10,000 employees. In the [Stack Overflow 2025 survey](https://survey.stackoverflow.co/2025/technology), 3.4% of respondents used Databricks SQL. Review-site ratings could not be retrieved.

## History

Databricks grew from managed Spark into a full data and AI platform through acquisitions: MosaicML (2023), Tabular, the company of Iceberg's creators (2024), Neon (2025) and Electric (2026), [per Wikipedia](https://en.wikipedia.org/wiki/Databricks). Its private valuation went from $62 billion (December 2024) to $190 billion post-money (August 2026). Its bet on open table formats, in a long contest with Snowflake, has made open, file-based analytical storage the industry norm.

## For ownpurse

Databricks is the strongest evidence that companies want data in formats they control. A Delta table is close in spirit to the ownpurse log: an ordered record of changes beside data any engine can read. It still differs in three ways. History expires by default. The unit is a table version, not a fact with a source. Every read is metered. ownpurse should borrow the shape: numbered transactions, `AS OF` queries, and a share format that needs no account to read. It should treat Parquet or Iceberg export as the bridge for a business that later needs a lakehouse. The gap Databricks leaves is the same as Snowflake's: an owner without a data team, books that must balance to the cent against their source, and a person approving each write.
