---
{
 "slug": "cdata",
 "name": "CData",
 "url": "https://www.cdata.com",
 "segment": "connector-framework",
 "status": "active",
 "founded": "Not confirmed (often reported as 2014)",
 "hq": "Chapel Hill, North Carolina, US",
 "backing": "About $350M in growth funding led by Warburg Pincus with Accel, announced 2024-06-25; Updata Partners is an existing investor. CData says it has been profitable since inception.",
 "tagline": "Commercial drivers, sync tools and MCP servers that expose hundreds of business systems, including QuickBooks, Xero, MYOB and Sage, as SQL.",
 "scope": "CData sells connectivity: ODBC, JDBC, ADO.NET and Python drivers, Excel and BI connectors, CData Sync for replication, an embedded OEM business, and Connect AI, a managed MCP platform and gateway. Its drivers model each source as relational tables you can read and write. It does not keep books, reconcile or hold history; Sync replicates current state into a database you choose.",
 "domains": [
  "integration",
  "reporting",
  "ledger"
 ],
 "customers_served": [
  "developers",
  "smb",
  "mid-market",
  "enterprise"
 ],
 "spectrum": {
  "custody": {
   "score": 2,
   "note": "Drivers query live. Sync can replicate into a database you own, but as a mutable copy of the source's current state."
  },
  "locality": {
   "score": 3,
   "note": "Drivers, Sync and the local MCP servers run on your machine or servers. Connect AI is a hosted cloud service."
  },
  "history": {
   "score": 1,
   "note": "Live current state. Sync offers incremental replication and CDC, but I found no versioned or as-of history."
  },
  "openness": {
   "score": 1,
   "note": "Proprietary, licensed drivers. The open MCP wrappers (MIT) still need a licensed or trial JDBC driver."
  },
  "breadth": {
   "score": 3,
   "note": "Hundreds of sources across ledgers (QBO, Xero, MYOB, Sage, NetSuite), payments (Stripe), CRM and ERP, each as its own model rather than one record."
  },
  "autonomy": {
   "score": 1,
   "note": "Software can write unattended through SQL, but CData itself does no bookkeeping or posting."
  },
  "agent_readiness": {
   "score": 4,
   "note": "A managed MCP platform and gateway, per-source local MCP servers, a Python SDK and CLI (Jun 2026), and read/write controls."
  },
  "integration_ease": {
   "score": 3,
   "note": "Standard ODBC/JDBC/SQL, free trials and per-driver docs, but every source is licensed and paid."
  },
  "market": {
   "score": 3,
   "note": "Mostly mid-market and enterprise (Gartner Magic Quadrant 2024 and 2025, OEM to Google, Salesforce, Palantir), with entry plans at $79 a month."
  }
 },
 "api": {
  "public": true,
  "style": "SQL through ODBC, JDBC, ADO.NET and Python DB-API drivers. A REST API on Sync (add-on). MCP through Connect AI and local servers.",
  "auth": "The driver handles each source's OAuth or keys. Connect AI manages credentials centrally and offers SSO and SCIM on Business.",
  "limits": "Source API limits apply because drivers query live. Connect AI has a fair-use limit of 100M records a month.",
  "cost": "Xero JDBC Developer License $999/yr (1 developer, 1 source; server licence by quote). Connect AI Standard $79/mo annual ($99 monthly), Growth $159/mo annual, Business by quote. Sync Standard $7,999/yr (5 connections, 100M rows/mo); marketplace from $2.74/hr (all 2026-10).",
  "export": "CData Sync replicates sources into databases and warehouses, with incremental and CDC options. The drivers let you SELECT everything into any tool.",
  "notes": "The Xero driver page calls the drivers bi-directional, so inserts and updates to Xero are supported through SQL and SSIS tasks."
 },
 "pricing": {
  "model": "Per source, per developer or server, and per connection subscriptions. Sync is priced by connections and monthly rows, or hourly on cloud marketplaces.",
  "entry": "Connect AI Standard $79/mo billed annually (1 user, 1 source); a 14-day trial with no card",
  "top": "Business and enterprise contracts by quote; Sync multi-server, site and OEM licences by quote",
  "notes": "No permanent free plan for Connect AI. CData launched free Connect Spreadsheets for Excel and Google Sheets in Nov 2024."
 },
 "agents": {
  "mcp": "official",
  "ai_features": "Connect AI (launched 2025-09-24): managed MCP to hundreds of sources, an MCP catalog, derived views and toolkits, agent tooling (Mar 2026), Developer Edition with Python SDK and CLI (Jun 2026), and an AI gateway with model routing and governance (Sep 2026). There are also free per-source local MCP servers on GitHub, including xero-mcp-server-by-cdata (May 2025), plus QuickBooks, MYOB AccountRight, Sage 50 UK, Sage 200/300 and NetSuite.",
  "agent_writes": "Yes, through Connect AI and the paid local MCP server, which CData says offers full read, write, update, delete and actions. The open-source GitHub MCP servers are read-only.",
  "approval": "No per-write human approval gate found. Connect AI describes read/write controls per source, which act as permissions rather than a review of each change.",
  "notes": "CData claims 98.5% accuracy across 378 prompts and a 25+ point lead over competing MCP providers (Mar 2026 press release, vendor benchmark)."
 },
 "integration": {
  "scope": "Hundreds of sources. The site says 'hundreds more' in the MCP catalog. Accounting ones include QuickBooks Online and Desktop, QuickBooks Time, Xero, MYOB AccountRight, Sage 50 UK, Sage 200, Sage 300, Sage cloud accounting, NetSuite and SAP Business One, plus Stripe.",
  "notes": "It is embedded by data platforms (Google, Salesforce, Informatica, Palantir Foundry, Databricks Marketplace MCP partner). The same driver model runs in Excel, Power BI, Tableau, SSIS and Python."
 },
 "preference": {
  "scale": "Over 7,000 organisations (press release, 2024-06-25). 10,000+ customers (site, 2026-10). Inc. 5000 2024, 2025 and 2026; Deloitte Fast 500 four times through 2025.",
  "ratings": "Not confirmed. G2 and Capterra returned 403 and web search was unavailable.",
  "sentiment": "Not confirmed from review sites."
 },
 "timeline": [
  {
   "date": "2024-03-05",
   "event": "Acquires Data Virtuality (data virtualisation)"
  },
  {
   "date": "2024-06-25",
   "event": "About $350M growth funding led by Warburg Pincus"
  },
  {
   "date": "2024-11-20",
   "event": "Free Connect Spreadsheets for Excel and Google Sheets"
  },
  {
   "date": "2024-12-06",
   "event": "First appearance in the Gartner Magic Quadrant for Data Integration Tools"
  },
  {
   "date": "2025-05-29",
   "event": "Read-only local Xero MCP server published on GitHub"
  },
  {
   "date": "2025-09-24",
   "event": "Connect AI, a managed MCP platform, launches"
  },
  {
   "date": "2026-06-23",
   "event": "Connect AI Developer Edition, Python SDK and CLI"
  },
  {
   "date": "2026-09-29",
   "event": "Connect AI Gateway with enterprise MCP and model routing"
  }
 ],
 "relation": "neighbour",
 "relation_note": "CData already gives agents SQL and MCP access to Xero, QuickBooks and MYOB, live and with writes. It is the shortest path an agent builder might take instead of ownpurse, but it keeps no record or history and has no approval package.",
 "borrow": "Per-source read/write controls that an operator sets once. One relational model per source that keeps the source's own names (not a lossy common model). A published agent-accuracy benchmark over real prompts. The same connector delivered through every surface (driver, spreadsheet, BI, MCP).",
 "avoid": "Direct agent writes to the books with only a permission toggle between the agent and the ledger. Per-source, per-seat licensing that puts a price on each new source. Live queries that spend the source's rationed API calls.",
 "sources": [
  {
   "title": "CData company page",
   "url": "https://www.cdata.com/company/"
  },
  {
   "title": "CData Xero drivers",
   "url": "https://www.cdata.com/drivers/xero/"
  },
  {
   "title": "Xero JDBC order page",
   "url": "https://www.cdata.com/drivers/xero/order/jdbc/"
  },
  {
   "title": "Connect AI pricing",
   "url": "https://www.cdata.com/ai/pricing/"
  },
  {
   "title": "CData Sync pricing",
   "url": "https://www.cdata.com/sync/pricing/"
  },
  {
   "title": "CData MCP solutions",
   "url": "https://www.cdata.com/solutions/mcp/"
  },
  {
   "title": "CData press releases",
   "url": "https://www.cdata.com/company/press/"
  },
  {
   "title": "CData ~$350M growth funding release",
   "url": "https://www.cdata.com/company/press/cdata-software-secures-350-million-in-strategic-growth-funding/"
  },
  {
   "title": "xero-mcp-server-by-cdata",
   "url": "https://github.com/CDataSoftware/xero-mcp-server-by-cdata"
  },
  {
   "title": "CDataSoftware on GitHub",
   "url": "https://github.com/CDataSoftware"
  }
 ],
 "unverified": [
  "Founding year (commonly given as 2014, not confirmed on CData's pages)",
  "Whether the paid local CData MCP Server for Xero is free, in beta or licensed, and its terms",
  "G2, Capterra and TrustRadius ratings and review sentiment",
  "Whether Sync offers a history or SCD mode that keeps prior versions",
  "Employee count and revenue"
 ],
 "researched": "2026-10-10"
}
---

## What it is

[CData](https://www.cdata.com) is a Chapel Hill, North Carolina company that sells connectivity to business systems. It makes standard drivers (ODBC, JDBC, ADO.NET, Python) that present each source as SQL tables, CData Sync for replication, an embedded OEM business, and, since September 2025, Connect AI, a managed MCP platform. Its accounting sources include QuickBooks Online and Desktop, Xero, MYOB AccountRight, Sage 50 UK, Sage 200 and 300, NetSuite and SAP Business One. CData says it has 10,000+ customers ([company page](https://www.cdata.com/company/), 2026-10).

## How it works

A driver maps a source's API onto relational tables. The [Xero page](https://www.cdata.com/drivers/xero/) calls the connectivity bi-directional: you can SELECT invoices and also INSERT or UPDATE them, from Excel, SSIS, Python or any SQL client. CData Sync replicates sources into databases and warehouses with incremental loads, and added real-time CDC and open table formats in March 2026. Connect AI hosts the same connectors behind one MCP endpoint, with derived views, toolkits and read/write controls.

## Data and access

The drivers query live, so you see the source's current state and pay its API limits on each uncached query. Sync gives you your own copy, but it is a current-state replica, not a history. Prices as of October 2026: a Xero JDBC Developer License is [$999 a year](https://www.cdata.com/drivers/xero/order/jdbc/) for one developer and one source, with server licences by quote. Connect AI is [$79 a month](https://www.cdata.com/ai/pricing/) billed annually for one user and one source (Growth $159), with a 14-day trial and a 100M-record monthly fair-use cap. [Sync](https://www.cdata.com/sync/pricing/) starts at $7,999 a year, or $2.74 an hour on cloud marketplaces.

## Agents

CData moved early on MCP. Between April and September 2025 it published 229 MCP repositories on [GitHub](https://github.com/CDataSoftware), among them [xero-mcp-server-by-cdata](https://github.com/CDataSoftware/xero-mcp-server-by-cdata) (29 May 2025), QuickBooks, MYOB and several Sage editions. They are MIT-licensed, read-only wrappers that need a licensed or trial JDBC driver. For writes, the README points to the paid local MCP server, which offers full read, write, update, delete and actions, and to Connect AI. In 2026 CData added agent tooling (March), a Developer Edition with a Python SDK and CLI (June), and an AI gateway (September) ([press](https://www.cdata.com/company/press/)). It claims 98.5% accuracy over 378 prompts, which is its own benchmark. Writes are governed by permissions, not by a person reviewing each change.

## Customers

CData's customers are mainly mid-market and enterprise IT and data teams, plus the platforms that embed it (Google, Salesforce, Informatica, Palantir, Databricks). It appeared in Gartner's Magic Quadrant for Data Integration Tools in 2024 and 2025. Its June 2024 [funding release](https://www.cdata.com/company/press/cdata-software-secures-350-million-in-strategic-growth-funding/) cited over 7,000 organisations, Warburg Pincus leading about $350M with Accel, and profitability since inception. I could not reach G2 or Capterra, so review ratings are unconfirmed.

## History

CData grew as a driver vendor and OEM supplier, bought Data Virtuality in March 2024, took growth funding in June 2024, launched free spreadsheet connectors in November 2024, and has since repositioned around AI connectivity. Connect AI launched on 24 September 2025.

## For ownpurse

CData is what an agent builder reaches for today when they want Claude to read and edit Xero: one SQL model per source, already built for dozens of ledgers. Its strengths are real. It has breadth, it keeps each source's own field names instead of a lossy common model, and it delivers the same connector through every surface. The gap is the one ownpurse is built around. There is no record you hold, no history, no check to the cent, and writes go straight to the books behind a permission toggle. ownpurse should treat CData as proof of demand for agent access to ledgers, and set itself apart on custody, history and the approved package.
