---
{
 "slug": "dynamics-365-business-central",
 "name": "Microsoft Dynamics 365 Business Central",
 "url": "https://www.microsoft.com/en-us/dynamics-365/products/business-central",
 "segment": "incumbent-ledger",
 "status": "active",
 "founded": "1984 (as PC&C, maker of Navision); Business Central launched 2018-04-02",
 "hq": "Redmond, Washington, USA (Microsoft); product roots in Copenhagen, Denmark",
 "backing": "A product of Microsoft (NASDAQ: MSFT), which acquired Navision A/S on 2002-07-11.",
 "tagline": "Microsoft's ERP for small and mid-sized businesses, the cloud successor to Navision and Dynamics NAV, sold through partners.",
 "scope": "Full ERP: general ledger, multi-company and multi-currency finance, bank reconciliation, sales and purchasing, inventory, projects, service and, on Premium, manufacturing. It is extended in AL and sold and implemented through Microsoft partners. It is not a freelancer or household tool, and in most countries payroll comes from partner apps rather than the base product.",
 "domains": [
  "ledger",
  "bank",
  "invoicing",
  "spend",
  "close",
  "reporting",
  "commerce",
  "integration"
 ],
 "customers_served": [
  "smb",
  "mid-market",
  "accountants",
  "developers"
 ],
 "spectrum": {
  "custody": {
   "score": 2,
   "note": "Online, Microsoft holds the data, but owners can export the whole database as BACPAC files (10 times a month per environment), and an on-premises edition keeps the SQL database on your servers."
  },
  "locality": {
   "score": 2,
   "note": "Self-hostable: an on-premises edition runs on your own SQL Server through partners; the online edition is Microsoft cloud only."
  },
  "history": {
   "score": 2,
   "note": "Posted ledger entries are not edited, and corrections are reversals (NAV's long-standing design); an optional change log tracks field edits, but there are no as-of queries."
  },
  "openness": {
   "score": 2,
   "note": "Proprietary runtime, but with open REST/OData APIs, a full database export, and application source (System App, Business Foundation, Base App) published under MIT on GitHub in BCApps."
  },
  "breadth": {
   "score": 3,
   "note": "Finance, banking, sales, purchasing, inventory, projects and manufacturing in one system; no personal finance, receipts vault or wallets."
  },
  "autonomy": {
   "score": 3,
   "note": "Payables, Sales Order and Expense agents read mail and documents and prepare or post transactions for people to review."
  },
  "agent_readiness": {
   "score": 4,
   "note": "An official MCP server at mcp.businesscentral.dynamics.com, read-only by default, with per-API create/modify/delete/action switches, Entra OAuth with PKCE, and public docs for non-Microsoft clients."
  },
  "integration_ease": {
   "score": 2,
   "note": "Well documented, but you need Entra app registration and per-user throttling, adding fields means writing AL, and most deployments go through a partner."
  },
  "market": {
   "score": 3,
   "note": "Small and mid-sized businesses, up to the lower mid-market and subsidiaries of larger groups."
  }
 },
 "api": {
  "public": true,
  "style": "REST on OData v4 (API v2.0 pages, queries and codeunit actions), plus custom API pages written in AL; webhooks (200 subscriptions max); SOAP is being deprecated",
  "auth": "OAuth 2.0 through Microsoft Entra ID (delegated or service-to-service); the MCP server uses authorization code with PKCE",
  "limits": "Per user: 6,000 OData requests per 5-minute sliding window, 5 concurrent requests, 100 connections, 95 queued; 20,000 entities per page, 100 operations per $batch, 8-minute timeout. Microsoft has said daily entitlement quotas may come later (operational limits page, updated 2025-12-29).",
  "cost": "No separate API fee; calls run under a licensed user or an application user in a paid tenant. Agent use consumes Copilot Credits (a third-party source cites 50 credits per receipt for the Expense Agent).",
  "export": "Admins with the D365 BACKUP/RESTORE permission can export a production database as a BACPAC file to their own Azure standard storage, up to 10 times a month per environment, on a paid subscription. Lists open or edit in Excel, and data can also be pulled in bulk through the API.",
  "notes": "You can't extend standard APIs with extra fields; you copy the AL code into a custom API page. An OpenAPI specification is published."
 },
 "pricing": {
  "model": "Per named user per month, paid yearly, sold through partners; agents are metered separately in Copilot Credits",
  "entry": "Team Members $8 per user per month (light use); Essentials $80 per user per month",
  "top": "Premium $110 per user per month (adds service management and manufacturing)",
  "notes": "Prices read from Microsoft's US pricing page on 2026-10-10. Copilot is included with Essentials. The Sales Order, Payables and custom agents need Copilot Credits (prepaid or pay-as-you-go) and an Azure subscription. Partner implementation fees, typically the larger cost, are not included."
 },
 "agents": {
  "mcp": "official",
  "ai_features": "Copilot is included with licences: bank reconciliation assistance, ad-hoc analysis, cash-flow and inventory forecasts, marketing text. Built-in agents: Sales Order Agent, Payables Agent and Expense Agent (in public preview from May 2026, US English first). Custom agents are built with Copilot Studio and the Agent Designer.",
  "agent_writes": "Through the official MCP server. The default configuration is read-only discovery; admins build configurations that list API pages and turn on Allow Create, Modify, Delete and Actions for each one (posting documents included). In preview since October 2026, Data Query Tools let agents compile and run read-only AL queries.",
  "approval": "Built-in agents put drafts and suggestions in a task pane for a person to review, confirm or dismiss. The admin-center MCP server asks for explicit confirmation of follow-up actions. Writes through the data MCP server are gated by configuration and the user's permissions, not by a per-write approval.",
  "notes": "Every MCP action runs as the signed-in user, so the audit trail shows who did it. Microsoft MCP hosts use a preregistered app; Claude, ChatGPT and other clients need their own Entra app registration."
 },
 "integration": {
  "scope": "Thousands of AppSource add-ons (Microsoft's FAQ, no exact count); native links to Microsoft 365, Teams, Power Platform, Dataverse and Shopify; webhooks, OData and Power Automate connectors.",
  "notes": "The partner channel is both the strength and the cost: most fit and finish comes from partner AL extensions. The admin-center MCP server (preview since 2026-04-01) covers environment management."
 },
 "preference": {
  "scale": "About 55,000 companies (Microsoft's product page, read 2026-10-10). Dynamics NAV had 102,000 customers in Q1 2014 (Wikipedia).",
  "ratings": "Not confirmed: G2, Capterra, TrustRadius and Software Advice all returned 403 to the fetch.",
  "sentiment": "Not drawn from review sites this pass. Widely reported themes are tight Microsoft 365 integration and breadth, against implementation cost, partner dependence and complexity for small teams; treat these as unconfirmed."
 },
 "timeline": [
  {
   "date": "1984",
   "event": "PC&C founded in Denmark; first Navision version in 1987"
  },
  {
   "date": "1995",
   "event": "Company renamed Navision Software A/S"
  },
  {
   "date": "2002-07-11",
   "event": "Microsoft acquires Navision A/S"
  },
  {
   "date": "2005-09",
   "event": "Rebranded Microsoft Dynamics NAV"
  },
  {
   "date": "2014 Q1",
   "event": "Dynamics NAV reports 102,000 customers"
  },
  {
   "date": "2018-04-02",
   "event": "Dynamics 365 Business Central released as NAV's cloud successor"
  },
  {
   "date": "2023-12",
   "event": "API throttling moves from per-environment to per-user limits"
  },
  {
   "date": "2025 wave 2",
   "event": "Business Central MCP server appears with BC27.1 (per a partner blog)"
  },
  {
   "date": "2026-04-01",
   "event": "Admin-center MCP server enters public preview; 2026 wave 1 brings the Payables and Sales Order agents"
  },
  {
   "date": "2026-05",
   "event": "Expense Agent public preview (US English)"
  },
  {
   "date": "2026-10",
   "event": "BC29 (2026 wave 2) begins; MCP Data Query Tools in preview"
  }
 ],
 "relation": "neighbour",
 "relation_note": "An enterprise-leaning SMB ledger that has gone furthest toward an official agent interface; a possible source connector, and the reference point for how a big vendor gates agent writes.",
 "borrow": "Read-only by default, with per-object create/modify/delete/action switches as a clean model for scoping what an agent may touch; every agent action runs as a named user; a task pane where agent drafts wait for review; and a database export the owner can run themselves.",
 "avoid": "Throttling that makes you rotate service principals to get throughput, extending an API meaning a code deployment, metered agent credits on top of seat licences, and a partner channel between the owner and their own data.",
 "sources": [
  {
   "title": "Business Central product page",
   "url": "https://www.microsoft.com/en-us/dynamics-365/products/business-central"
  },
  {
   "title": "Business Central pricing",
   "url": "https://www.microsoft.com/en-us/dynamics-365/products/business-central/pricing"
  },
  {
   "title": "Operational limits in Business Central online",
   "url": "https://learn.microsoft.com/en-us/dynamics365/business-central/dev-itpro/administration/operational-limits-online"
  },
  {
   "title": "Business Central MCP server overview",
   "url": "https://learn.microsoft.com/en-us/dynamics365/business-central/dev-itpro/ai/mcp-overview"
  },
  {
   "title": "Configure Business Central MCP server",
   "url": "https://learn.microsoft.com/en-us/dynamics365/business-central/dev-itpro/ai/configure-mcp-server"
  },
  {
   "title": "Admin center MCP server (2026 wave 1 release plan)",
   "url": "https://learn.microsoft.com/en-us/dynamics365/release-plan/2026wave1/smb/dynamics365-business-central/connect-ai-agents-admin-center-through-mcp-server"
  },
  {
   "title": "Exporting databases in the admin center",
   "url": "https://learn.microsoft.com/en-us/dynamics365/business-central/dev-itpro/administration/tenant-admin-center-database-export"
  },
  {
   "title": "API (v2.0) for Business Central",
   "url": "https://learn.microsoft.com/en-us/dynamics365/business-central/dev-itpro/api-reference/v2.0/"
  },
  {
   "title": "microsoft/BCApps on GitHub",
   "url": "https://github.com/microsoft/BCApps"
  },
  {
   "title": "Microsoft Dynamics 365 Business Central (Wikipedia)",
   "url": "https://en.wikipedia.org/wiki/Microsoft_Dynamics_365_Business_Central"
  },
  {
   "title": "Business Central AI agents: 2026 MCP guide (ERP Software Blog, 2026-08)",
   "url": "https://erpsoftwareblog.com/2026/08/dynamics-365-business-central-ai-agents-mcp-guide/"
  },
  {
   "title": "BC29: run data queries with MCP server (yzhums, 2026-10-02)",
   "url": "https://yzhums.com/76419/"
  }
 ],
 "unverified": [
  "G2, Capterra and TrustRadius ratings and review counts: all returned 403.",
  "The Navision purchase price (often reported at about $1.3 billion) is not confirmed from a primary source.",
  "The Expense Agent's rate of 50 Copilot Credits per receipt, and the Payables Agent's 50 included invoices a month, come from third-party blogs.",
  "The date the MCP server first appeared (BC27.1, 2025 wave 2) and its GA status come from partner blogs; Microsoft's docs do not label the core server preview or GA.",
  "The earlier price increase (Essentials from $70 and Premium from $100) and its date were not confirmed this pass.",
  "Customer sentiment themes were not drawn from review sites in this pass."
 ],
 "researched": "2026-10-10"
}
---

## What it is

Business Central is Microsoft's ERP for small and mid-sized businesses. It descends from Navision, written in Denmark from 1984 and bought by Microsoft in 2002, renamed Dynamics NAV in 2005 and relaunched as a cloud service on [2 April 2018](https://en.wikipedia.org/wiki/Microsoft_Dynamics_365_Business_Central). Microsoft's [product page](https://www.microsoft.com/en-us/dynamics-365/products/business-central) says about 55,000 companies use it (read 2026-10-10). It is sold and implemented only through partners.

## How it works

The system is a classic ERP: general ledger, multi-company and multi-currency finance, bank reconciliation, sales, purchasing, inventory, projects, and on Premium, service and manufacturing. Partners extend it in AL, Microsoft's language for the product, and publish apps on AppSource. Its ledger design is older and stricter than most SMB tools: posted ledger entries are not edited, and you fix a mistake with a reversing entry, with an optional change log for field edits. It runs online in Microsoft's cloud or on-premises on your own SQL Server.

[Pricing](https://www.microsoft.com/en-us/dynamics-365/products/business-central/pricing) is per user per month, paid yearly: Essentials $80, Premium $110, Team Members $8 (read 2026-10-10). Copilot comes with the licence; the agents consume Copilot Credits, sold separately, and need an Azure subscription. Partner implementation is usually the larger bill.

## Data and access

The API is REST on OData v4 with Entra ID OAuth and webhooks. The [operational limits](https://learn.microsoft.com/en-us/dynamics365/business-central/dev-itpro/administration/operational-limits-online) are per user: 6,000 requests per five-minute window, five at a time, 20,000 entities per page. Microsoft's own advice for heavy integrations is to spread the work across several service principals, and it says longer-term quotas may follow. You can't add a field to a standard API without writing a custom API page in AL.

Custody is better than the cloud label suggests. A paying customer's admin can [export the database as a BACPAC](https://learn.microsoft.com/en-us/dynamics365/business-central/dev-itpro/administration/tenant-admin-center-database-export) to their own Azure storage, ten times a month per environment, and load it into SQL Server. Microsoft also publishes the application source, including the Base Application, under MIT in [BCApps](https://github.com/microsoft/BCApps) and accepts contributions.

## Agents

This is where Business Central leads the incumbents. The [official MCP server](https://learn.microsoft.com/en-us/dynamics365/business-central/dev-itpro/ai/mcp-overview) at `mcp.businesscentral.dynamics.com` works with Copilot Studio, VS Code, GitHub Copilot CLI, Claude and ChatGPT. By default it gives read-only discovery of the API pages the signed-in user may see. To let an agent write, an admin [builds a configuration](https://learn.microsoft.com/en-us/dynamics365/business-central/dev-itpro/ai/configure-mcp-server) listing API pages and turning on Allow Create, Modify, Delete and Actions for each, posting included. In October 2026 (BC29) Data Query Tools entered preview: agents find tables, read schemas and relations, and compile read-only AL queries on the fly. Every action runs as the user, so the audit trail names them.

The built-in Sales Order, Payables and Expense agents read mailboxes and documents and leave drafts in a task pane for review. A [separate MCP server](https://learn.microsoft.com/en-us/dynamics365/release-plan/2026wave1/smb/dynamics365-business-central/connect-ai-agents-admin-center-through-mcp-server) for the admin centre (preview since 1 April 2026) asks for explicit confirmation of each follow-up action.

## Customers

Typical customers are companies of tens to hundreds of staff that have outgrown QuickBooks or Xero, and subsidiaries of larger groups already on Microsoft 365. Review-site ratings could not be fetched for this dossier.

## History

Business Central is the survivor of Microsoft's 2000s ERP roll-up (Navision, Great Plains, Axapta, Solomon). Microsoft moved NAV's large partner base to the cloud and then, from 2025, made it one of the first ledgers with a first-party MCP server, following Microsoft's push to expose Dynamics 365 actions to agents.

## For ownpurse

Business Central is a neighbour and a possible source rather than a competitor: its buyers are mid-market finance teams, not one person with their agent. It matters as a reference design. Read-only by default, write tools switched on per object, actions bound to a named user, drafts that wait in a review pane: that is close to ownpurse's "agents propose; people post", built from inside the vendor's cloud. What it lacks is what ownpurse is for: an append-only record the owner holds, as-of queries, reads that cost nothing and no throttle, and approvals that are sealed packages recorded in a log rather than permission flags.
