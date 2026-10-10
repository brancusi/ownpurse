---
{
 "slug": "netsuite",
 "name": "NetSuite",
 "url": "https://www.netsuite.com",
 "segment": "incumbent-ledger",
 "status": "acquired",
 "founded": "1998 (as NetLedger)",
 "hq": "Austin, Texas, United States",
 "backing": "Oracle (NYSE: ORCL), which bought NetSuite for US$9.3 billion; deal announced 28 July 2016, closed 7 November 2016. Before that NYSE-listed from December 2007; founded with about US$125 million from Larry Ellison's Tako Ventures.",
 "tagline": "Oracle's cloud ERP for mid-sized companies: general ledger, order-to-cash, procurement, inventory and more in one hosted system.",
 "scope": "A full ERP: multi-entity general ledger and consolidation, AP and AR, revenue recognition, inventory and warehouse, procurement, CRM, e-commerce and (in the US) payroll. It is the system growing SMBs move to after QuickBooks or Xero, and the one most AI ledgers say they replace. It does not run on your own machine, and it is sold by quote with implementation partners rather than self-serve.",
 "domains": [
  "ledger",
  "bank",
  "payments",
  "invoicing",
  "payroll",
  "spend",
  "close",
  "reporting",
  "commerce",
  "tax"
 ],
 "customers_served": [
  "mid-market",
  "enterprise",
  "smb"
 ],
 "spectrum": {
  "custody": {
   "score": 1,
   "note": "Oracle hosts the only live copy; Oracle's own docs say Full CSV Export is not a complete export, and a full copy needs saved searches, SuiteQL or the paid SuiteAnalytics Connect."
  },
  "locality": {
   "score": 0,
   "note": "Oracle-hosted SaaS only."
  },
  "history": {
   "score": 2,
   "note": "System Notes record field-level changes with user and time on most records, but there are no as-of queries and integrations read current state."
  },
  "openness": {
   "score": 1,
   "note": "Closed source, no public price list, API open only to paying customers and partners, bulk export partial or paid."
  },
  "breadth": {
   "score": 4,
   "note": "Ledger, banking, AP/AR, inventory, commerce, payroll, close and reporting in one system."
  },
  "autonomy": {
   "score": 2,
   "note": "Rules-based bank matching and workflows today; NetSuite Next promises agentic workflows, and MCP agents can write records directly."
  },
  "agent_readiness": {
   "score": 3,
   "note": "Official, free MCP server (AI Connector Service and MCP Standard Tools SuiteApp) with OAuth 2.0 PKCE and role scoping; concurrency is shared and setup needs an administrator."
  },
  "integration_ease": {
   "score": 1,
   "note": "SOAP, REST, RESTlets and SuiteQL, each with governance limits, per-account customisation and roles; most integrations need a consultant or a partner."
  },
  "market": {
   "score": 3,
   "note": "Mid-market first (57.7% of G2 reviewers), reaching into enterprise and fast-growing small companies."
  }
 },
 "api": {
  "public": true,
  "style": "SuiteTalk SOAP and REST web services, REST record API, SuiteQL (SQL over the record model, read-only via MCP), RESTlets and SuiteScript (server-side JavaScript); ODBC/JDBC via SuiteAnalytics Connect",
  "auth": "Token-based authentication or OAuth 2.0 (authorization code with PKCE for MCP clients); access bound to NetSuite roles and permissions",
  "limits": "Account-wide concurrency limit shared across SOAP, REST, RESTlets and the AI connector, readable through the governanceLimits endpoint and allocatable per integration; extra concurrency comes with higher service tiers and SuiteCloud Plus licences. Third parties cite 15 as a common default, 1,000 records per request and 100,000 rows per SuiteQL query.",
  "cost": "No per-call fee found. API and MCP access come with the licence; more concurrency costs more (service tier, SuiteCloud Plus); SuiteAnalytics Connect (ODBC) is a separate paid licence.",
  "export": "CSV export of lists, reports and saved searches; Full CSV Export covers a fixed list of record types and Oracle says it does not export all data; complete extraction needs saved searches, SuiteQL, Connect or a warehouse integration.",
  "notes": "Default concurrency by tier was not confirmed in Oracle's docs in this pass. Every account is heavily customised (custom records and fields), so a connector must read metadata per account."
 },
 "pricing": {
  "model": "Annual subscription by quote: a base platform licence plus per-user licences and modules, usually with a partner-led implementation",
  "entry": "No public price; third-party estimates for 2026 put the base licence near US$999/month and full users at US$99-199/month (often US$129)",
  "top": "Unbounded by quote; third-party estimates of US$120,000-300,000+ a year for mid-market rollouts before implementation, which is quoted separately (US$25,000-750,000)",
  "notes": "All price figures are estimates from resellers and pricing sites (Vendr, ERP Research, others), not Oracle. Trustpilot reviewers complain of sharp renewal increases and hard-to-exit multi-year contracts."
 },
 "agents": {
  "mcp": "official",
  "ai_features": "NetSuite Next (announced SuiteWorld, October 2025): embedded conversational assistant, agentic workflows and natural-language search; SuiteCloud AI tools for custom agents; September 2026 release added AI portlets to the homepage. SuiteWorld 2026 runs 25-28 October 2026, after this survey.",
  "agent_writes": "Yes. The MCP Standard Tools SuiteApp lets an AI client create, read and update records and run reports, saved searches and read-only SuiteQL, within the connecting role's permissions. Client-side scripts do not run on agent writes.",
  "approval": "No per-write approval gate in NetSuite's agent path. Controls are a dedicated role, the MCP Server Connection permission, an explicit acknowledgment before connecting a third-party model, and the audit trail; Oracle advises clients to require user consent.",
  "notes": "The AI Connector Service is free and the SuiteApp is free; it supports remote MCP clients on protocol 2025-06-18 with streamable HTTP, and Oracle documents setup for Claude, ChatGPT, Codex and Claude Code. Calls draw on the account's shared concurrency."
 },
 "integration": {
  "scope": "SuiteApp Marketplace, a large partner and consultancy ecosystem, connectors from every unified API and iPaaS vendor (Celigo, Boomi, Workato and others), Oracle's own bank and payment connectivity.",
  "notes": "Powerful but heavy. Each account's custom fields and records mean a connector cannot assume a fixed schema; shared concurrency means a new integration can starve existing ones."
 },
 "preference": {
  "scale": "Oracle and analyst coverage put NetSuite above 43,000 customers (Zacks via Nasdaq, 2026, undated figure). Oracle does not break out NetSuite revenue; one analyst preview estimated NetSuite bookings at about US$1.0-1.1 billion a quarter in FY2026.",
  "ratings": "G2: 4.1/5 from about 4,500 reviews (2026; counts on comparison pages ranged 4,407-4,562). Trustpilot: 1.5/5 from 51 reviews (October 2026). Capterra not retrieved.",
  "sentiment": "Praised for depth, multi-entity consolidation and one system for the whole business. Complaints: cost and renewal increases, long implementations, dated interface, consultant dependence, concurrency limits on integrations."
 },
 "timeline": [
  {
   "date": "1998",
   "event": "Evan Goldberg founds NetLedger with about US$125 million from Larry Ellison's Tako Ventures"
  },
  {
   "date": "July 2002",
   "event": "Zach Nelson becomes CEO"
  },
  {
   "date": "September 2003",
   "event": "NetLedger renamed NetSuite"
  },
  {
   "date": "December 2007",
   "event": "IPO on the NYSE under ticker N"
  },
  {
   "date": "28 July 2016",
   "event": "Oracle agrees to buy NetSuite for US$9.3 billion"
  },
  {
   "date": "7 November 2016",
   "event": "Oracle completes the acquisition"
  },
  {
   "date": "August 2025",
   "event": "Brian Chess introduces the AI Connector Service, built on MCP"
  },
  {
   "date": "7 October 2025",
   "event": "SuiteWorld 2025: NetSuite Next and SuiteCloud AI tools announced"
  },
  {
   "date": "September 2026",
   "event": "Monthly release adds AI portlets to the homepage"
  },
  {
   "date": "25-28 October 2026",
   "event": "SuiteWorld 2026 scheduled in Las Vegas"
  }
 ],
 "relation": "source",
 "relation_note": "A possible later connector and the far end of the market ownpurse serves. It is the system growing businesses graduate to and the one Rillet, Campfire, DualEntry and others pitch to replace; ownpurse does not compete with it, it would hold a copy beside it.",
 "borrow": "Oracle's MCP design is worth copying: agents act under an existing role, a named permission gates the connection, the owner acknowledges before any data reaches a third-party model, and every agent action lands in the audit trail. System Notes show the value of field-level change history.",
 "avoid": "Shared, opaque concurrency that a new integration can exhaust; partial bulk export that leaves you building saved searches to leave; pricing by quote with renewal surprises. Also the gap in its agent path: direct writes with no per-write approval.",
 "sources": [
  {
   "title": "NetSuite AI Connector Service FAQ (Oracle docs)",
   "url": "https://docs.oracle.com/en/cloud/saas/netsuite/ns-online-help/article_4160616848.html"
  },
  {
   "title": "Get Started with the NetSuite AI Connector Service (Oracle docs)",
   "url": "https://docs.oracle.com/en/cloud/saas/netsuite/ns-online-help/article_3200541651.html"
  },
  {
   "title": "Integration governance limits (Oracle docs)",
   "url": "https://docs.oracle.com/en/cloud/saas/netsuite/ns-online-help/article_0815123622.html"
  },
  {
   "title": "CSV export of lists and reports (Oracle docs)",
   "url": "https://docs.oracle.com/en/cloud/saas/netsuite/ns-online-help/section_N464443.html"
  },
  {
   "title": "NetSuite expands SuiteCloud platform with new AI innovations (Oracle, 7 Oct 2025)",
   "url": "https://www.oracle.com/in/news/announcement/sw25-netsuite-expands-suitecloud-platform-new-ai-innovation-2025-10-07/"
  },
  {
   "title": "NetSuite Next launch (Oracle, 7 Oct 2025)",
   "url": "https://www.oracle.com/ph/news/announcement/sw25-netsuite-launches-netsuite-next-2025-10-07/"
  },
  {
   "title": "NetSuite (Wikipedia)",
   "url": "https://en.wikipedia.org/wiki/NetSuite"
  },
  {
   "title": "Will Oracle's NetSuite division accelerate revenue growth in 2026 (Zacks via Nasdaq)",
   "url": "https://www.nasdaq.com/articles/will-oracles-netsuite-division-accelerate-revenue-growth-2026"
  },
  {
   "title": "NetSuite buyer guide (Vendr)",
   "url": "https://www.vendr.com/buyer-guides/netsuite"
  },
  {
   "title": "NetSuite MCP (Numeric)",
   "url": "https://www.numeric.io/blog/netsuite-mcp"
  },
  {
   "title": "NetSuite on Trustpilot",
   "url": "https://www.trustpilot.com/review/www.netsuite.com"
  },
  {
   "title": "NetSuite vs QuickBooks Online (G2)",
   "url": "https://www.g2.com/compare/netsuite-vs-quickbooks-online"
  }
 ],
 "unverified": [
  "NetSuite customer count above 43,000 (analyst figure, not confirmed in an Oracle filing in this pass)",
  "All NetSuite pricing figures (third-party estimates; Oracle publishes no price list)",
  "Default account concurrency by service tier (third parties cite 15)",
  "1,000 records per request and 100,000 rows per SuiteQL query (secondary sources)",
  "NetSuite bookings of US$1.0-1.1 billion a quarter in FY2026 (analyst estimate)",
  "September 2026 AI portlets release (from search summary, not read on Oracle's page)",
  "Oracle press releases returned 403 to direct fetch; their content is taken from search summaries and press coverage",
  "Capterra rating not retrieved; web search budget ran out mid-research"
 ],
 "researched": "2026-10-10"
}
---

## What it is

NetSuite is Oracle's cloud ERP. It started in 1998 as NetLedger, backed with about US$125 million from Larry Ellison, listed on the NYSE in 2007 and was bought by Oracle for US$9.3 billion in November 2016 ([Wikipedia](https://en.wikipedia.org/wiki/NetSuite)). It is the default answer when a company outgrows QuickBooks or Xero: one hosted system for the general ledger, multi-entity consolidation, AP and AR, revenue recognition, inventory, procurement, CRM and e-commerce.

## How it works

NetSuite is sold by quote. Oracle publishes no price list. Third-party estimates for 2026 ([Vendr](https://www.vendr.com/buyer-guides/netsuite) and others) put the base licence near US$999 a month, full users at US$99 to US$199 a month, and a mid-market rollout at US$120,000 to US$300,000 a year before an implementation that partners quote separately. Treat these as estimates; resellers wrote most of them.

Each account is customised with its own fields, records, workflows and SuiteScript. That is why it fits so many businesses, and why every integration has to discover the schema it is talking to.

## Data and access

There are several ways in: SOAP and REST web services, SuiteQL (SQL over the record model), RESTlets and server-side SuiteScript, plus ODBC through the separately licensed SuiteAnalytics Connect. All of them draw on one account-wide concurrency pool, which you can read from the [governanceLimits endpoint](https://docs.oracle.com/en/cloud/saas/netsuite/ns-online-help/article_0815123622.html) and split between integrations. More concurrency comes with a higher service tier.

Getting everything out is harder. Oracle's [CSV export docs](https://docs.oracle.com/en/cloud/saas/netsuite/ns-online-help/section_N464443.html) say Full CSV Export does not export all data. People leaving NetSuite build saved searches per record type or pull through SuiteQL. On history, NetSuite does better than most: System Notes log field-level changes with who and when. You still cannot ask what the books looked like on a given date.

## Agents

NetSuite is the most agent-ready of the incumbents. The [AI Connector Service](https://docs.oracle.com/en/cloud/saas/netsuite/ns-online-help/article_4160616848.html) is a remote MCP server. With the free MCP Standard Tools SuiteApp, an AI client can create, read and update records, run reports and saved searches, and run read-only SuiteQL. Oracle names Claude, ChatGPT, Codex and Claude Code as clients. It uses OAuth 2.0 with PKCE and acts under a NetSuite role. You have to acknowledge before data reaches a third-party model, and every agent action is logged. Agent calls share the account's concurrency. There is no per-write approval: a role that can write, writes. At SuiteWorld in October 2025 Oracle announced NetSuite Next, with agentic workflows. SuiteWorld 2026 runs 25 to 28 October.

## Customers

Analyst coverage ([Zacks via Nasdaq](https://www.nasdaq.com/articles/will-oracles-netsuite-division-accelerate-revenue-growth-2026)) cites more than 43,000 customers. G2 shows 4.1 from about 4,500 reviews, 57.7% of them from mid-market companies. Reviewers praise its depth and consolidation. They complain about cost, renewals, long implementations and needing consultants. [Trustpilot](https://www.trustpilot.com/review/www.netsuite.com) is small and harsh: 1.5 from 51 reviews.

## History

In 1998 Ellison backed the idea that a whole business could run on one hosted system, years before "cloud" was a word for it. A 2002-2004 deal sold NetLedger as the Oracle Small Business Suite. Oracle bought the company in 2016. NetSuite now ships product updates monthly, and its AI direction is in the open.

## For ownpurse

NetSuite is not ownpurse's first audience, but it sets the reference point. Oracle's MCP design (an existing role, a named permission, consent before a model sees data, an audit log) is close to the controls ownpurse needs. It still writes directly, with no package a person seals. A NetSuite connector would be heavy work, with per-account metadata, shared concurrency and SuiteQL paging. Its value is a held, versioned copy for owners whose only full record otherwise lives at Oracle.
