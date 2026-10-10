---
{
 "slug": "blackline",
 "name": "BlackLine",
 "url": "https://www.blackline.com",
 "segment": "close-analytics",
 "status": "active",
 "founded": "2001",
 "hq": "Los Angeles (Woodland Hills), California, US",
 "backing": "Public on Nasdaq (BL) since 2016. FY2025 revenue $700.4M; Q2 2026 revenue $187.8M (4 August 2026). Activist Engaged Capital signed a cooperation agreement on 9 March 2026 adding two directors; founder Therese Tucker retired from full-time work on 2 June 2026.",
 "tagline": "Enterprise software for account reconciliation, the financial close, intercompany and invoice-to-cash, now sold as an 'agentic financial operations platform'.",
 "scope": "BlackLine sits between a large company's ERPs and its auditors: account reconciliations, transaction matching, journal entries, consolidation, intercompany, task management and invoice-to-cash, on its Studio360 data and workflow platform with Verity AI agents. It integrates with SAP (which resells it), Oracle, Workday, Microsoft Dynamics, Sage Intacct and NetSuite plus banks and other sources. It is not a ledger and not built for small businesses; it is cloud-only.",
 "domains": [
  "close",
  "reporting",
  "payments",
  "invoicing",
  "integration",
  "documents"
 ],
 "customers_served": [
  "enterprise",
  "mid-market"
 ],
 "spectrum": {
  "custody": {
   "score": 1,
   "note": "ERPs keep the books; reconciliations, certifications, supporting documents and audit trails live in BlackLine's cloud, exportable through the API and reports."
  },
  "locality": {
   "score": 0,
   "note": "Cloud-only on GCP, Azure and AWS, with regional data residency (including Saudi Arabia from December 2025); no self-hosting."
  },
  "history": {
   "score": 2,
   "note": "Audit trails on every action, certified reconciliations kept by period, explainable decision records for AI; no documented as-of query over source history."
  },
  "openness": {
   "score": 2,
   "note": "Closed source, but a documented public REST API with OpenAPI specs viewable without login: reconciliations, matching, journals, tasks, reporting exports and more."
  },
  "breadth": {
   "score": 4,
   "note": "Record-to-report and invoice-to-cash in one platform, fed by ERPs, banks, point-of-sale, treasury and other sources."
  },
  "autonomy": {
   "score": 2,
   "note": "Verity agents prepare reconciliations and match transactions and journals can auto-post, but every package ends in human sign-off and finance-defined controls."
  },
  "agent_readiness": {
   "score": 2,
   "note": "Public API with sandboxes and a Finance Control Console (preview, June 2026) to govern third-party agents; no MCP server found."
  },
  "integration_ease": {
   "score": 2,
   "note": "Documented OpenAPI, sandboxes and a Postman guide, but client credentials and scopes come from BlackLine support and quotas are unpublished."
  },
  "market": {
   "score": 4,
   "note": "Enterprise and large mid-market; 4,260 customers (30 June 2026); SAP resells it."
  }
 },
 "api": {
  "public": true,
  "style": "REST, OpenAPI 3 specs on an Apigee developer portal (developer.blackline.com): Account Reconciliation, Matching, Matching Jobs, Journals, Journal Templates, Tasks, Variance, Consolidation Integrity Manager, Document Management, Supporting Documents, Master Data, Period, Reporting, Roles, Teams, Users, Integrations (Studio Integrate).",
  "auth": "OAuth token from BlackLine's STS (/connect/token); client id and secret and scopes issued by BlackLine support; resource-owner flow uses a BlackLine user name and that user's API key.",
  "limits": "Quota-based; 429 'Rate limit quota violation' responses are documented, values not published.",
  "cost": "Not separately priced; part of the subscription. Production servers in US, EU and AP, with sandboxes for each.",
  "export": "Reporting API runs queries and initiates report exports (CSV); data ingestion endpoints for accounts, subledgers and bank balances.",
  "notes": "Specs are public, credentials are not: an owner's IT team can build against it, an outside developer cannot without a customer."
 },
 "pricing": {
  "model": "Enterprise subscription, quote-based. Moving from per-user to a 'platform pricing' model 'driven by the size and complexity of the customer' (FY2025 10-K); platform pricing was 17% of eligible ARR at 30 June 2026.",
  "entry": "Not published.",
  "top": "Not published.",
  "notes": "Also bought through SAP as a solution extension and through the GCP, Azure and AWS marketplaces."
 },
 "agents": {
  "mcp": "none",
  "ai_features": "Verity AI (launched 9 September 2025): Verity Prepare (multi-agent reconciliation preparation, generally available 27 July 2026), Verity Match, Verity Collect and Remit, Verity Forecast, Verity Flag; WiseLayer agents for accruals and payroll accounting (acquired 15 December 2025); Finance Control Console for AI governance (preview, 25 June 2026). ISO 42001 certified (September 2025).",
  "agent_writes": "Agents ingest support, match transactions, identify reconciling items, write variance explanations and assemble reconciliation packages; journal entries can be automated and posted to ERPs; collections agents contact customers.",
  "approval": "Agents assemble 'an audit-ready reconciliation package for human sign-off'; the Finance Control Console adds policy enforcement, end-to-end audit trails of automated actions, explainable decision records and human-in-the-loop exception management over BlackLine, partner, customer-built and third-party agents.",
  "notes": "BlackLine's framing is 'a glass box': AI may do the work, finance must be able to validate it. Its CEO: 'AI, but governed by finance.'"
 },
 "integration": {
  "scope": "ERPs: Workday, SAP (deepest; Smart Close for SAP, resold as SAP SolEx since 2018), Oracle, Microsoft Dynamics 365, Sage Intacct, NetSuite; Snowflake and Workday connectors added April 2026; banks, point-of-sale, treasury and other sources through Studio360 Integrate.",
  "notes": "Implementations usually run through partners (Deloitte, EY, Capgemini, Genpact, RSM); the 10-K notes customers may input incorrect or incomplete data and need training."
 },
 "preference": {
  "scale": "4,260 customers at 30 June 2026 (4,394 at end of 2025, 4,443 at end of 2024; the 10-K says an enterprise focus has raised churn among smaller customers). FY2025 revenue $700.4M, up 7%; Q2 2026 revenue $187.8M, up 9.2%; net revenue retention 102%; RPO $1.1B.",
  "ratings": "Earned four TrustRadius Top Rated awards (BlackLine, June 2026). G2, Capterra and Gartner Peer Insights figures not verified (blocked).",
  "sentiment": "Not verified from review sites in this pass. Investor signals: an activist (Engaged Capital) won two board seats in March 2026 and the company has spent $327.8M on buybacks (to 3 August 2026), while revenue growth sits in single digits."
 },
 "timeline": [
  {
   "date": "2001",
   "event": "Founded by Therese Tucker in Los Angeles."
  },
  {
   "date": "2016",
   "event": "IPO on Nasdaq (BL)."
  },
  {
   "date": "2018",
   "event": "SAP begins reselling BlackLine as an SAP solution extension."
  },
  {
   "date": "2025-09-09",
   "event": "Launches Verity AI for the Office of the CFO; ISO 42001 certification follows on 10 September."
  },
  {
   "date": "2025-12-15",
   "event": "Acquires WiseLayer, AI agents for accruals and payroll accounting."
  },
  {
   "date": "2026-02-26",
   "event": "FY2025 10-K: revenue $700.4M; 4,394 customers; move to platform pricing."
  },
  {
   "date": "2026-03-09",
   "event": "Cooperation agreement with activist Engaged Capital; board grows to 14."
  },
  {
   "date": "2026-04-14",
   "event": "Announces Agentic Financial Operations at BeyondTheBlack London, with Snowflake and Workday connectors."
  },
  {
   "date": "2026-06-02",
   "event": "Founder Therese Tucker retires from full-time employment (announced 24 March)."
  },
  {
   "date": "2026-06-25",
   "event": "Finance Control Console preview for governing AI agents."
  },
  {
   "date": "2026-07-27",
   "event": "Verity Prepare generally available."
  },
  {
   "date": "2026-08-04",
   "event": "Q2 2026: revenue $187.8M, 4,260 customers."
  },
  {
   "date": "2026-09-21",
   "event": "Acquires NetNow (credit risk and customer onboarding)."
  }
 ],
 "relation": "cautionary-tale",
 "relation_note": "BlackLine is how large companies solve the same record problem: a separate system that pulls from every ERP, proves each balance with support, and keeps a signed audit trail, sold for twenty-five years at enterprise prices with partner-led setup. It shows the problem is real and paid for; it also shows what happens when that layer is a vendor's cloud: small customers churn out, and the record stays with the vendor.",
 "borrow": "The reconciliation as an evidence package: balance, support, reconciling items, explanation, preparer, certifier. Separate products for matching and for certification. A governance console that treats native, partner, customer and third-party agents the same way and records an explainable decision for each action. Public OpenAPI specs with regional sandboxes.",
 "avoid": "Credentials issued only through support; unpublished quotas; pricing that is opaque and drifting; an enterprise focus that drops small customers; a 'system of record for AI' the customer cannot hold.",
 "sources": [
  {
   "title": "BlackLine home page",
   "url": "https://www.blackline.com"
  },
  {
   "title": "BlackLine press releases",
   "url": "https://www.blackline.com/about/press-releases/"
  },
  {
   "title": "BlackLine expands agentic financial operations platform; Finance Control Console preview (25 June 2026)",
   "url": "https://www.blackline.com/about/press-releases/2026/blackline-expands-agentic-financial-operations-platform-establishing-the-trust-infrastructure-for-ai-powered-finance/"
  },
  {
   "title": "BlackLine unveils Agentic Financial Operations (14 April 2026)",
   "url": "https://www.blackline.com/about/press-releases/2026/blackline-unveils-agentic-financial-operations-to-close-ais-governance-and-trust-gap/"
  },
  {
   "title": "Verity Prepare general availability (27 July 2026)",
   "url": "https://www.blackline.com/about/press-releases/2026/blackline-advances-governed-ai-for-finance-with-general-availability-of-verity-prepare/"
  },
  {
   "title": "BlackLine acquires WiseLayer (15 December 2025)",
   "url": "https://www.blackline.com/about/press-releases/2025/blackline-acquires-wiselayer-to-enhance-its-ai-capabilities/"
  },
  {
   "title": "BlackLine acquires NetNow (21 September 2026)",
   "url": "https://www.blackline.com/about/press-releases/2026/blackline-acquires-netnow-expanding-invoice-to-cash-with-intelligent-customer-onboarding-and-credit-risk-management/"
  },
  {
   "title": "BlackLine Studio360",
   "url": "https://www.blackline.com/products/studio360/"
  },
  {
   "title": "BlackLine developer portal",
   "url": "https://developer.blackline.com/"
  },
  {
   "title": "BlackLine FY2025 Form 10-K (filed 26 February 2026)",
   "url": "https://www.sec.gov/Archives/edgar/data/1666134/000162828026011915/bl-20251231.htm"
  },
  {
   "title": "BlackLine Q2 2026 results, Exhibit 99.1 (4 August 2026)",
   "url": "https://www.sec.gov/Archives/edgar/data/1666134/000117184326005201/exh_991.htm"
  },
  {
   "title": "Form 8-K: cooperation agreement with Engaged Capital (9 March 2026)",
   "url": "https://www.sec.gov/Archives/edgar/data/1666134/000119312526100451/d121231d8k.htm"
  },
  {
   "title": "Form 8-K: Therese Tucker consulting agreement (31 July 2026)",
   "url": "https://www.sec.gov/Archives/edgar/data/1666134/000162828026051856/bl-20260731.htm"
  },
  {
   "title": "Form 8-K: buyback increase (31 July 2026)",
   "url": "https://www.sec.gov/Archives/edgar/data/1666134/000162828026052541/bl-20260731.htm"
  }
 ],
 "unverified": [
  "G2, Capterra and Gartner Peer Insights ratings and review counts (sites blocked).",
  "Customer sentiment from reviews and forums (not gathered; web search budget exhausted).",
  "IPO month and price (2016 known; details not re-checked from a primary source in this pass).",
  "Exact HQ address (press datelines say Los Angeles; Woodland Hills not re-checked).",
  "The April 2026 announcement is listed as 7 April on the press page and dated 14 April in the release body; the body date is used.",
  "Absence of an MCP server: none found on BlackLine's site or press, but not confirmed by BlackLine."
 ],
 "researched": "2026-10-10"
}
---

## What it is

BlackLine is the enterprise standard for account reconciliation and the financial close. Therese Tucker founded it in Los Angeles in 2001, it listed on Nasdaq in 2016, and SAP has resold it since 2018. Its software sits between a large company's ERPs and its auditors: reconciliations, transaction matching, journal entries, consolidation, intercompany, task management and, since acquisitions, invoice-to-cash. It now calls itself ["the agentic financial operations platform"](https://www.blackline.com), built on a data and workflow layer called Studio360 and an AI suite called Verity.

## How it works

BlackLine pulls balances and transactions from ERPs (Workday, SAP, Oracle, Microsoft Dynamics, Sage Intacct, NetSuite) and from banks, point-of-sale and treasury systems, then proves each balance sheet account with support, matches high-volume transactions, routes journals for approval and certifies reconciliations period by period, with an audit trail on every action. The [FY2025 10-K](https://www.sec.gov/Archives/edgar/data/1666134/000162828026011915/bl-20251231.htm) describes hosting on Google Cloud, Azure and AWS, with regional data residency. Implementations usually run through partners such as Deloitte, EY, Capgemini, Genpact and RSM.

## Data and access

BlackLine publishes [OpenAPI specs](https://developer.blackline.com/) that anyone can read: reconciliations, matching, journals, tasks, variance, documents, master data, reporting exports, users and Studio Integrate jobs, on US, EU and AP servers with sandboxes. Authentication is an OAuth token from BlackLine's own token service, with a client id, secret and scopes issued by BlackLine support; quotas exist but are not published. Pricing is by quote and changing: the 10-K describes a move from per-user pricing to platform pricing "driven by the size and complexity of the customer," which was 17% of eligible ARR at 30 June 2026.

## Agents

[Verity](https://www.blackline.com/about/press-releases/) launched on 9 September 2025. Verity Prepare, [generally available on 27 July 2026](https://www.blackline.com/about/press-releases/2026/blackline-advances-governed-ai-for-finance-with-general-availability-of-verity-prepare/), is a multi-agent system that ingests support, matches transactions, finds reconciling items, writes explanations and assembles "an audit-ready reconciliation package for human sign-off." BlackLine bought WiseLayer (accrual and payroll agents) in December 2025 and previewed a [Finance Control Console](https://www.blackline.com/about/press-releases/2026/blackline-expands-agentic-financial-operations-platform-establishing-the-trust-infrastructure-for-ai-powered-finance/) on 25 June 2026 to govern BlackLine, partner, customer-built and third-party agents with policies, audit trails and explainable decision records. No MCP server was found.

## Customers

BlackLine had 4,260 customers at 30 June 2026, down from 4,443 at the end of 2024; the 10-K says its enterprise focus has raised churn among smaller customers. Revenue was $700.4M in 2025, up 7%, and $187.8M in Q2 2026, up 9.2%, with net revenue retention of 102%. Review-site ratings could not be read in this pass; BlackLine reports four TrustRadius Top Rated awards in 2026.

## History

After twenty-five years of steady growth, 2026 has been eventful. The activist Engaged Capital won two board seats under a [cooperation agreement](https://www.sec.gov/Archives/edgar/data/1666134/000119312526100451/d121231d8k.htm) on 9 March, the founder retired from full-time work on 2 June, buybacks reached $327.8M by 3 August, and the company acquired NetNow on 21 September. The CEO calls the first half of 2026 "the most consequential period in BlackLine's history."

## For ownpurse

BlackLine shows the record problem is real and that large companies pay a lot to solve it: a separate system that reads every ledger, proves every balance with evidence, and keeps a signed trail an auditor accepts. Its reconciliation-as-package and its governance console (one policy layer for every agent, an explainable record per action) line up with ownpurse's sealed, approved packages. The difference is who it serves and who holds the record. BlackLine needs an ERP, a partner and an enterprise budget, is shedding small customers, and keeps the evidence in its cloud. ownpurse can give the same checks to the people BlackLine has never served, with the evidence in a file they hold.
