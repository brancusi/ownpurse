---
{
 "slug": "sage-intacct",
 "name": "Sage Intacct",
 "url": "https://www.sage.com/en-us/sage-business-cloud/intacct/",
 "segment": "incumbent-ledger",
 "status": "acquired",
 "founded": "1999",
 "hq": "San Jose, California (parent The Sage Group plc, Newcastle upon Tyne, UK)",
 "backing": "Raised about $130M in venture funding 2000-2014 (HWVP, Battery Ventures, Bessemer, Sigma, Emergence); last round $30M in 2014 at a $211M valuation. Acquired by The Sage Group plc (LSE: SGE) for $850M, announced 25 July 2017.",
 "tagline": "A cloud general ledger and financial suite for mid-sized businesses, owned by Sage.",
 "scope": "Sage Intacct is a multi-entity, dimensional general ledger with AP, AR, cash management, order management, purchasing and reporting, plus paid modules for projects, fixed assets, inventory, subscription billing, consolidation and more. It is sold through quotes and partners to finance teams that have outgrown QuickBooks or Xero. It does not run on your machine, and it does not keep a copy of your books outside Sage's cloud unless you buy or build an extract.",
 "domains": [
  "ledger",
  "bank",
  "invoicing",
  "close",
  "reporting",
  "spend"
 ],
 "customers_served": [
  "mid-market",
  "enterprise",
  "accountants"
 ],
 "spectrum": {
  "custody": {
   "score": 1,
   "note": "Sage's cloud holds the record; you can pull a copy through the API or the paid Data Delivery Service CSV extract, object by object."
  },
  "locality": {
   "score": 0,
   "note": "Cloud only, no self-hosted option."
  },
  "history": {
   "score": 2,
   "note": "Field-level audit history (who, when, field before and after) kept since 2016 and reportable, but records themselves are current state with no as-of queries."
  },
  "openness": {
   "score": 1,
   "note": "Public REST and XML APIs, but API use needs a paid Web Services subscription, bulk export is a paid add-on, and there is no single full-tenant export."
  },
  "breadth": {
   "score": 3,
   "note": "GL, AP, AR, cash, purchasing, order entry, projects, fixed assets and consolidation in one system; banks, payroll and documents come through partners."
  },
  "autonomy": {
   "score": 2,
   "note": "Sage Copilot, a Close Assistant and AI-assisted AP and reconciliation, with administrators enabling features; not unattended posting across the books."
  },
  "agent_readiness": {
   "score": 3,
   "note": "First-party MCP server (AI Gateway) governed by Intacct roles, plus a REST API; the MCP server is read-only and in an early adopter programme."
  },
  "integration_ease": {
   "score": 1,
   "note": "Needs a Web Services subscription and user type, monthly transaction entitlements with overage fees, low concurrency, and partner-led setup."
  },
  "market": {
   "score": 3,
   "note": "Mid-market finance teams, often multi-entity, with larger organisations on bigger contracts."
  }
 },
 "api": {
  "public": true,
  "style": "REST (OAuth 2.0, generally available from 2025 Release 1) and a legacy XML Web Services API that still covers some objects and functions REST lacks (readReport, attachments).",
  "auth": "OAuth 2.0 for REST; XML uses sender credentials plus company login. Both require the Web Services subscription and a Web Services user type.",
  "limits": "Performance Tier 1 entitlement of 100,000 API transactions a month by third-party accounts; Sage says it does not block calls over the entitlement but charges overage. REST concurrency limits exist but are not published as numbers; Tier 1 reportedly allows about two synchronous connections. XML queries return up to 2,000 records a call.",
  "cost": "Web Services subscription is a paid part of the quote (price not published). Overage beyond the monthly entitlement is charged; one third party quotes $0.15 per 10-transaction pack. Data Delivery Service is a paid add-on. Apideck reports partner connector terms of $2,500 a year plus $0.015 per call.",
  "export": "Report exports (CSV, Excel, PDF, Word, text); Data Delivery Service bulk CSV extracts per object to S3, Azure, Box, Dropbox, Google Drive or HTTP (full once a day, changes hourly); API reads. Attachments and some objects (supporting documents, audit history) are not in DDS. No single full-tenant export.",
  "notes": "Sage says all new objects and features ship only in REST; XML is supported but legacy, with no published sunset date. Smart Events and scripts also consume the transaction entitlement."
 },
 "pricing": {
  "model": "Annual subscription, quote-based, priced by modules, entities and users, sold mostly through partners.",
  "entry": "Not published. Third-party estimates put a single-user Core Financials subscription at about $9,000-$15,000 a year (2025-26 estimates).",
  "top": "Not published. Third-party estimates put typical mid-market spend at $25,000-$75,000 a year and large multi-entity deployments over $100,000, plus implementation.",
  "notes": "All figures are third-party estimates, not Sage's. A UK partner reported rate plans rising about 9% from 1 January 2024. Reviewers mention renewal price creep and modules adding cost."
 },
 "agents": {
  "mcp": "official",
  "ai_features": "Sage Copilot embedded in Intacct; a Finance Intelligence Agent for plain-language questions over Intacct data (US and UK, phased early adopter in 2026); Close Assistant, AI variance explanations and AP/reconciliation automation; the Sage Intacct AI Gateway (REST plus an MCP server) for outside AI tools.",
  "agent_writes": "The MCP server is read-only by Sage's own description. Agents can write through the REST API with a Web Services user whose role permits it.",
  "approval": "No approval gate on API writes beyond Intacct's own role permissions and workflow approvals (for bills, payments, journal entries where configured). MCP access is limited by the assigned role; administrators enable the gateway and AI features.",
  "notes": "Not listed as a Claude connector as of 10 October 2026. Third parties (CData and others) sell their own Intacct MCP servers. Some areas (project accounting, fixed assets) reportedly still rely on the XML API."
 },
 "integration": {
  "scope": "Sage Intacct Marketplace of partner apps (count not confirmed), bank feeds, Sage's own add-ons (AP automation, expense management via the acquired Fyle), and every major unified API (Apideck, Merge, Rutter, Codat and others).",
  "notes": "Integrations are often built and maintained by implementation partners. API activity from every integration counts against one monthly entitlement, a common source of overages."
 },
 "preference": {
  "scale": "11,000 customers at acquisition (July 2017). In Sage's FY25 (year to 30 September 2025), Sage Intacct revenue in the US grew 23% to £461m, over 45% of Sage's US revenue; ARR grew over 20% in the US and about 50% outside it.",
  "ratings": "G2: about 4.3 of 5 from roughly 4,000-4,100 reviews (G2 comparison page snapshots, 2026). Capterra rating not confirmed.",
  "sentiment": "Praised for dimensions, multi-entity consolidation, reporting depth and fit for mid-market controllership. Complaints: a steep learning curve, rigid or complex custom reporting that pushes teams to BI tools, modular pricing that rises at renewal, and dependence on partners for support."
 },
 "timeline": [
  {
   "date": "1999",
   "event": "Founded in San Jose by Odysseas Tsatalos and David Chandler Thomas as one of the first cloud accounting applications."
  },
  {
   "date": "2000",
   "event": "$10M Series A led by HWVP."
  },
  {
   "date": "2014",
   "event": "Last venture round, $30M at a $211M valuation; about $130M raised in total."
  },
  {
   "date": "2017-07-25",
   "event": "The Sage Group announces it will buy Intacct for $850M; Intacct reports 11,000 customers and about $88M revenue for the year to June 2017."
  },
  {
   "date": "2025-02",
   "event": "REST API generally available (2025 Release 1); XML becomes legacy, new features REST only."
  },
  {
   "date": "2025-11-04",
   "event": "Sage Intacct AI Gateway with an MCP server presented at Sage Future for Partners (per partner reports)."
  },
  {
   "date": "2025-11-19",
   "event": "Sage FY25 results: Intacct US revenue £461m, up 23%; Finance Intelligence Agent for US and UK customers."
  },
  {
   "date": "2026-05",
   "event": "Sage announces expanded automation across core finance workflows; AI Gateway in an early adopter programme."
  }
 ],
 "relation": "neighbour",
 "relation_note": "Intacct is the mid-market ledger that the AI ERPs (Rillet, Campfire, DualEntry) pitch to replace. ownpurse starts with smaller books, but Intacct shows where a growing business ends up and how a mid-market ledger rations and prices API access.",
 "borrow": "The audit history model: every change records who, when, field, value before and value after, and is reportable. Dimensions on line items as first-class facts. A read-only MCP surface that follows the same role permissions as people.",
 "avoid": "Pricing API access by monthly transaction counts with overage fees, making bulk export a paid add-on, and offering no single export of the whole company.",
 "sources": [
  {
   "title": "TechCrunch: Sage Group buys Intacct for $850M (2017)",
   "url": "https://techcrunch.com/2017/07/25/sage-group-buys-intacct-accounting-software-for-850m/amp/"
  },
  {
   "title": "Wikipedia: Intacct",
   "url": "https://en.wikipedia.org/wiki/Intacct"
  },
  {
   "title": "Sage FY25 results, 19 November 2025 (FCA NSM)",
   "url": "https://data.fca.org.uk/artefacts/NSM/RNS/3331551a-5060-4fad-88d1-730e3d45176b.html"
  },
  {
   "title": "Sage Intacct developer portal",
   "url": "https://developer.intacct.com/"
  },
  {
   "title": "Sage Intacct 2025 R1 release note: REST API GA",
   "url": "https://www-p02.intacct.com/ia/docs/en_US/releasenotes/2025/2025_Release_1/Platform_Services/2025-R1-platform-restapi-ga.htm"
  },
  {
   "title": "Sage developer: API transaction volume, concurrency and scaling",
   "url": "https://developer.sage.com/intacct/docs/developer-portal/api-essentials/txn-volume-concurrency-scaling"
  },
  {
   "title": "Sage Intacct help: Audit History report",
   "url": "https://www-p02.intacct.com/ia/docs/en_US/help_action/Administration/Audit_trail/Reports/audit-history-report.htm"
  },
  {
   "title": "Sage: Intacct AI Gateway",
   "url": "https://www.sage.com/en-us/sage-business-cloud/intacct/product-capabilities/platform/ai-gateway/"
  },
  {
   "title": "Sage newsroom, May 2026: trusted automation across core finance workflows",
   "url": "https://www.sage.com/en-gb/company/digital-newsroom/2026/05/sage-intacct-expands-trusted-automation-across-core-finance-workflows/"
  },
  {
   "title": "Baker Tilly: Sage Intacct MCP server for finance leaders",
   "url": "https://www.bakertilly.com/insights/sage-intacct-mcp-server-finance-leaders"
  },
  {
   "title": "ClonePartner: exporting data from Sage Intacct",
   "url": "https://clonepartner.com/blog/how-to-export-data-from-sage-intacct-api-limits-dds-and-portability.md"
  },
  {
   "title": "ERP Research: Sage pricing estimates",
   "url": "https://erpresearch.com/en-us/sage-pricing"
  },
  {
   "title": "ERP Research: AI-native ERP systems",
   "url": "https://erpresearch.com/en-us/ai-native-erp"
  },
  {
   "title": "Apideck: Sage Intacct connector",
   "url": "https://developers.apideck.com/apis/accounting/sage-intacct/overview"
  },
  {
   "title": "Capterra: Sage Intacct reviews",
   "url": "https://www.capterra.com/p/76/Intacct/reviews/"
  }
 ],
 "unverified": [
  "The 100,000 transactions a month Tier 1 entitlement, the $0.15 per 10-transaction overage price and the concurrency figures come from third parties; Sage's own page returned 403 and does not publish numbers in what was retrieved.",
  "Price of the Web Services subscription is not published.",
  "All subscription price figures are third-party estimates.",
  "The AI Gateway launch date (Sage Future for Partners, 4-5 November 2025) comes from partner and consultancy sources, not a Sage page read directly.",
  "Early adopter status of the AI Gateway in May 2026 comes from a search summary of a Sage newsroom page that returned 403 when fetched.",
  "G2 rating (about 4.3, roughly 4,000-4,100 reviews) comes from G2 comparison page snapshots; the product page could not be fetched. Capterra figures not obtained.",
  "Apideck's partner terms ($2,500 a year plus $0.015 per call) are Apideck's claim.",
  "Current Sage Intacct customer count not found; 11,000 is the 2017 figure.",
  "Claim that project accounting and fixed assets still need the XML API is from a third-party source."
 ],
 "researched": "2026-10-10"
}
---

## What it is

Sage Intacct is a cloud financial system for mid-sized businesses: a multi-entity general ledger with dimensions, accounts payable and receivable, cash management, purchasing and order entry, and reporting, with paid modules for projects, fixed assets, inventory, subscription billing and consolidation ([Wikipedia](https://en.wikipedia.org/wiki/Intacct)). It was founded in San Jose in 1999 and bought by The Sage Group for $850M in July 2017 ([TechCrunch](https://techcrunch.com/2017/07/25/sage-group-buys-intacct-accounting-software-for-850m/amp/)). It is what a finance team moves to after QuickBooks or Xero, and it is the system Rillet, Campfire and DualEntry name when they pitch an AI-native replacement ([ERP Research](https://erpresearch.com/en-us/ai-native-erp)).

It does a lot well. Reviewers credit its dimensional reporting and multi-entity consolidation, and its growth inside Sage is real: in the year to 30 September 2025, Intacct revenue in the US grew 23% to £461m, more than 45% of Sage's US revenue ([Sage FY25 results](https://data.fca.org.uk/artefacts/NSM/RNS/3331551a-5060-4fad-88d1-730e3d45176b.html)).

## How it works

You buy it on an annual quote, usually through a partner who implements it. Sage does not publish prices; third-party estimates put a single-user Core Financials subscription at about $9,000 to $15,000 a year and typical mid-market spend at $25,000 to $75,000, before implementation ([ERP Research](https://erpresearch.com/en-us/sage-pricing)). Those figures are estimates, not Sage's.

Intacct keeps an audit trail on records, and an Audit History data source that shows who changed what, when, and each field's value before and after, captured since May 2016 ([Intacct help](https://www-p02.intacct.com/ia/docs/en_US/help_action/Administration/Audit_trail/Reports/audit-history-report.htm)). That is more history than most small-business ledgers expose, but the records themselves are current state; you cannot ask for the books as of a past moment.

## Data and access

Intacct has two APIs. The REST API, with OAuth 2.0, became generally available in 2025 Release 1, and Sage says new objects and features ship only there; the XML API is legacy but supported, with no sunset date ([developer.intacct.com](https://developer.intacct.com/)). Both need a paid Web Services subscription and a Web Services user type ([release note](https://www-p02.intacct.com/ia/docs/en_US/releasenotes/2025/2025_Release_1/Platform_Services/2025-R1-platform-restapi-ga.htm)).

Access is metered. Sage says companies that exceed their monthly Performance Tier entitlement pay overage, and that it does not block the calls; third parties put Tier 1 at 100,000 transactions a month, with low concurrency and overage of $0.15 per 10-transaction pack, and note that Smart Events and scripts count too ([ClonePartner](https://clonepartner.com/blog/how-to-export-data-from-sage-intacct-api-limits-dds-and-portability.md)). Bulk export is the Data Delivery Service, a paid add-on that writes CSV per object to S3, Azure, Box, Dropbox or Google Drive: full extracts once a day, changes hourly, with attachments and audit history left out. There is no single export of a whole company.

## Agents

Sage builds AI in two directions. Sage Copilot sits inside the product, with a Finance Intelligence Agent for plain-language questions, rolling out to US and UK customers in a phased early adopter programme ([Sage FY25](https://data.fca.org.uk/artefacts/NSM/RNS/3331551a-5060-4fad-88d1-730e3d45176b.html)). The Sage Intacct AI Gateway goes the other way: REST plus a first-party MCP server that brings Intacct data to Claude, ChatGPT or your own agent. Sage describes the MCP server as read-only and governed by Intacct roles; writes go through the REST API ([Baker Tilly](https://www.bakertilly.com/insights/sage-intacct-mcp-server-finance-leaders)). Administrators enable it and choose the role. It is not in Claude's connector directory as of 10 October 2026, so connecting it is an admin build.

There is no approval gate on agent writes beyond Intacct's own roles and approval workflows.

## Customers

Intacct had 11,000 customers in 2017. Sage now cites strength in construction, real estate, not-for-profit and financial services, and rapid growth in the UK, Canada and South Africa. G2 shows about 4.3 of 5 from roughly 4,000 reviews. The complaints are consistent: a steep learning curve, custom reports that are rigid or complex, modular pricing that rises at renewal, and support that runs through partners ([Capterra reviews](https://www.capterra.com/p/76/Intacct/reviews/)).

## History

Intacct was one of the first cloud ledgers and raised about $130M before Sage bought it. Sage, a UK company built on desktop accounting, bought Intacct to get a cloud product in North America, and has since made it its flagship for mid-sized businesses. Now the venture money has moved to AI ledgers that want to replace it, and Sage's answer is automation inside the product plus an MCP gateway.

## For ownpurse

Intacct is a neighbour, not a near-term connector. It shows the ceiling of the market ownpurse starts below, and what happens to API access as a ledger moves up-market: it becomes a line on the quote, metered by the transaction, with bulk export sold separately. Worth borrowing is the audit history (field, before, after, who, when), which maps directly onto facts with provenance, and a read-only MCP surface that obeys the same roles as people. A future Intacct connector would have to budget its reads against the monthly entitlement, which is the argument for an owner-held record that answers questions without calling Sage.
