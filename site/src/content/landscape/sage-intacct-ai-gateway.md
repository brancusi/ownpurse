---
{
 "slug": "sage-intacct-ai-gateway",
 "name": "Sage Intacct AI Gateway",
 "url": "https://www.sage.com/en-us/sage-business-cloud/intacct/product-capabilities/platform/ai-gateway/",
 "segment": "agent-native",
 "status": "active",
 "founded": "2025-11 (announced 2025-11-04 at Sage Future for Partners, Barcelona)",
 "hq": "Sage Group: Newcastle upon Tyne, UK; Sage Intacct: San Jose, California",
 "backing": "The Sage Group plc (London Stock Exchange). FY25 (to 2025-09-30): ARR GBP 2,574m, up 11%; underlying revenue GBP 2,513m, up 10%. Sage acquired Intacct in 2017 for $850m.",
 "tagline": "Sage's official MCP server and REST API layer that lets outside AI tools read Sage Intacct data under its roles and permissions, with the MCP held to read-only.",
 "scope": "A governed endpoint that gives external AI clients (Claude, other MCP clients, custom agents, Amazon Bedrock apps) real-time read access to Intacct's GL, AP, AR, cash management, purchasing and order entry through MCP, beside full read and write through the REST API for applications built to use it. It is separate from Sage Copilot, the AI inside Intacct, and does not move data out of Intacct or keep any record of its own.",
 "domains": [
  "ledger",
  "invoicing",
  "spend",
  "close",
  "reporting",
  "integration"
 ],
 "customers_served": [
  "mid-market",
  "enterprise",
  "accountants"
 ],
 "spectrum": {
  "custody": {
   "score": 1,
   "note": "Sage holds the books in its cloud; owners can export through the Data Delivery Service and APIs, but the gateway adds no owner-held copy."
  },
  "locality": {
   "score": 0,
   "note": "Cloud only; the MCP server and REST API are hosted by Sage."
  },
  "history": {
   "score": 1,
   "note": "Intacct keeps an audit trail and posted entries are reversed rather than erased, but most records are current state and the gateway offers no as-of view."
  },
  "openness": {
   "score": 1,
   "note": "Closed product; the REST API is documented but needs a Web Services developer licence arranged through an account manager."
  },
  "breadth": {
   "score": 2,
   "note": "Wide within one system: GL, AP, AR, cash, purchasing and order entry for multi-entity mid-market books; nothing outside Intacct."
  },
  "autonomy": {
   "score": 1,
   "note": "Through MCP, agents cannot post, alter records or change balances; writes need a separately built REST application, and Sage Copilot automates inside the product."
  },
  "agent_readiness": {
   "score": 3,
   "note": "An official first-party MCP server with role-scoped access and an auditable endpoint, but admin setup, scoped credentials and no listing in Anthropic's connector directory."
  },
  "integration_ease": {
   "score": 2,
   "note": "Modern REST with new objects added only there, but licence gating, partner-led setup and thinner REST coverage for project accounting and fixed assets."
  },
  "market": {
   "score": 3,
   "note": "Mid-market finance teams, multi-entity organisations and the partners and firms that implement them."
  }
 },
 "api": {
  "public": true,
  "style": "REST (recommended; new objects ship only on REST), legacy XML Web Services, and MCP via the AI Gateway",
  "auth": "Web Services sender id and password authorised per company for the XML API; access for AI clients runs through a Sage Intacct user and role. OAuth details for the MCP not confirmed.",
  "limits": "Concurrent gateway connections limited per company id, excess requests queued first in, first out; Sage's example: of 100 concurrent requests about 5 run while the rest wait about 5 seconds. Guidance: queries under 1,000 records per page, under 100 records per write; 15-minute timeout on large requests.",
  "cost": "Web Services licence priced through the account manager (not published). CBIZ reports Finance AI available to all customers at no cost through May 2027; whether that covers the gateway is not confirmed.",
  "export": "REST and XML query APIs, saved reports, and the Data Delivery Service for bulk extracts.",
  "notes": "Partners report project accounting and fixed assets still lean on the XML API, so agent coverage there is thinner."
 },
 "pricing": {
  "model": "Annual subscription, negotiated per module and user through Sage or a partner; prices not published.",
  "entry": "Third-party estimates put entry at about $9,000-12,000 a year for one user and core financials (2026; not from Sage).",
  "top": "Reseller and buyer-platform estimates of $25,000-75,000 a year for typical mid-market deployments, plus $10,000-200,000 implementation (2026; not from Sage).",
  "notes": "All Intacct price figures come from resellers and aggregators that disagree widely; treat as rough."
 },
 "agents": {
  "mcp": "official",
  "ai_features": "AI Gateway (MCP plus REST) for outside agents; Sage Copilot inside Intacct for invoice processing, natural-language questions, close task tracking and anomaly flags; a Sage and AWS programme on Amazon Bedrock and AgentCore; plans to bring certified third-party agents into Copilot.",
  "agent_writes": "Not through MCP: partners quoting Sage say it gives read access only, governed by Intacct roles and permissions. Writes are possible through the REST API for applications designed and governed to use it.",
  "approval": "No approval gate; the control is policy and permissions. The MCP is read-only, and anything an application writes through REST is limited only by the role behind its credentials.",
  "notes": "Baker Tilly warns that the connection is only as narrow as the identity behind it, that read-only access does nothing about data leaving for an unvetted AI provider, and that AI output is not audit evidence."
 },
 "integration": {
  "scope": "AI clients: Claude and other MCP clients, custom agents on any LLM via REST, Amazon Bedrock apps. Intacct itself has a large marketplace of partner integrations (count not confirmed). Third parties such as CData also sell read-only Intacct MCP connectors.",
  "notes": "Setup is partner-led: an administrator enables the gateway and scopes a user and role before an agent can connect."
 },
 "preference": {
  "scale": "Sage cites Intacct as a main driver of 12% North America revenue growth in FY25; cloud-native revenue GBP 885m, up 23%. No Intacct customer count published.",
  "ratings": "G2 4.3/5 from about 4,075-4,096 reviews; Capterra 4.3/5 from 606 (March 2026) to 684 (May 2026) reviews.",
  "sentiment": "Reviewers praise automation, multi-entity consolidation and dimensional reporting; they complain about cost, a learning curve, a dated interface and hard-to-reach support. No independent reviews of the AI Gateway yet."
 },
 "timeline": [
  {
   "date": "1999",
   "event": "Intacct founded in San Jose as one of the first cloud accounting applications"
  },
  {
   "date": "2017-07",
   "event": "Sage Group acquires Intacct for $850m"
  },
  {
   "date": "2025",
   "event": "Intacct REST API generally available (2025 Release 1)"
  },
  {
   "date": "2025-11-04",
   "event": "Sage launches AI Developer Solutions at Sage Future for Partners; the Intacct AI Gateway, an MCP server on the REST API, is the first phase, with AWS Bedrock and AgentCore"
  },
  {
   "date": "2025-11-19",
   "event": "Sage FY25 results: ARR GBP 2,574m, up 11%, with Intacct named as a growth driver"
  },
  {
   "date": "2026-07-09",
   "event": "CBIZ: gateway available to all customers and partners; MCP read-only across AP, AR, GL, cash, purchasing and order entry"
  },
  {
   "date": "2026-09-01",
   "event": "Baker Tilly guidance for finance leaders on the read-only MCP and its risks"
  }
 ],
 "relation": "design-cousin",
 "relation_note": "An incumbent that chose read-only for agents by policy and kept writes on a separate, governed REST path. That is half of ownpurse's stance (agents read freely); ownpurse adds the other half, a sealed package a person approves before anything is written, and a record the owner holds. Intacct is also a likely future source.",
 "borrow": "State the agent policy in one plain sentence (MCP reads, REST writes) and enforce it in the product rather than in prompts; scope agent access to an existing role so permissions are the ones finance already audits; publish the coverage boundary by module.",
 "avoid": "Read-only as the whole answer, which leaves agents unable to propose anything and pushes writes to bespoke integrations with no review step; licence and partner gating that make a sanctioned agent connection slower to set up than an unofficial one.",
 "sources": [
  {
   "title": "Sage: Sage Intacct AI Gateway",
   "url": "https://www.sage.com/en-us/sage-business-cloud/intacct/product-capabilities/platform/ai-gateway/"
  },
  {
   "title": "CPA Practice Advisor: Sage launches AI Developer Solutions (2025-11-05)",
   "url": "https://www.cpapracticeadvisor.com/2025/11/05/sage-launches-ai-developer-solutions-next-stage-in-sages-platform-journey/172363/"
  },
  {
   "title": "Sage newsroom: Sage launches AI Developer Solutions (2025-11)",
   "url": "https://www.sage.com/en-gb/company/digital-newsroom/2025/11/sage-launches-ai-developer-solutions-to-accelerate-partner-led-innovation/"
  },
  {
   "title": "Baker Tilly: Sage Intacct MCP server for finance leaders (2026-09-01)",
   "url": "https://www.bakertilly.com/insights/sage-intacct-mcp-server-finance-leaders"
  },
  {
   "title": "CBIZ: How AI is evolving in Sage Intacct (2026-07-09)",
   "url": "https://www.cbiz.com/insights/article/how-ai-is-evolving-in-sage-intacct-and-what-it-means-for-finance-teams"
  },
  {
   "title": "Blytheco: Sage Intacct AI Gateway",
   "url": "https://blytheco.com/blog/posts/sage-intacct-ai-gateway-putting-your-financial-data-to-work-with-the-llms-you-choose"
  },
  {
   "title": "Audit Friendly: Does Sage Intacct have an MCP server?",
   "url": "https://auditfriendly.co/answers/sage-intacct-mcp-server-ai-gateway"
  },
  {
   "title": "Sage Intacct developer site",
   "url": "https://developer.intacct.com/"
  },
  {
   "title": "Sage Intacct Web Services",
   "url": "https://developer.intacct.com/web-services/"
  },
  {
   "title": "Sage FY25 results (2025-11-19)",
   "url": "https://www.sage.com/investors/investor-downloads/press-releases/2025/11/full-year-2025-results-19-november-2025/"
  },
  {
   "title": "Wikipedia: Intacct",
   "url": "https://en.wikipedia.org/wiki/Intacct"
  },
  {
   "title": "G2: QuickBooks vs Sage Intacct",
   "url": "https://www.g2.com/compare/intuit-quickbooks-vs-sage-intacct"
  },
  {
   "title": "Capterra: Sage Intacct reviews",
   "url": "https://www.capterra.com/p/76/Intacct/reviews/"
  },
  {
   "title": "ERP Research: Sage pricing",
   "url": "https://erpresearch.com/en-us/sage-pricing"
  }
 ],
 "unverified": [
  "Sage's own wording of the read-only policy: sage.com and developer.sage.com returned 403 to direct fetches, so the read-only claim rests on partners quoting Sage (Baker Tilly, CBIZ, Blytheco) and search summaries of the product page",
  "Authentication method for the MCP server (OAuth or a Web Services user)",
  "Exact general availability date of the MCP server (announced for November 2025; 'available to all customers' by July 2026)",
  "Whether the gateway or MCP carries any charge, and whether the 'no cost through May 2027' offer covers it",
  "MCP tool list and object coverage",
  "Claim that project accounting and fixed assets rely on the XML API (single secondary source)",
  "Intacct customer count and marketplace integration count",
  "All Intacct price figures (resellers and aggregators only)"
 ],
 "researched": "2026-10-10"
}
---

## What it is

The Sage Intacct AI Gateway is Sage's sanctioned door for outside AI tools into Intacct, its cloud financials product for mid-market companies. Sage [announced it on 4 November 2025](https://www.cpapracticeadvisor.com/2025/11/05/sage-launches-ai-developer-solutions-next-stage-in-sages-platform-journey/172363/) at Sage Future for Partners as the first phase of "AI Developer Solutions": an MCP server built on Intacct's REST API, developed with AWS (Amazon Bedrock and AgentCore). Sage's [product page](https://www.sage.com/en-us/sage-business-cloud/intacct/product-capabilities/platform/ai-gateway/) calls it a governed, auditable endpoint where role-based permissions decide what AI applications can see and do. It is separate from Sage Copilot, the assistant built into Intacct.

## How it works

The gateway has two halves. The MCP server gives real-time, read-only access to the general ledger, payables, receivables, cash management, purchasing and order entry. Beside it, the REST API "can support full access for applications designed and governed to use them" ([Blytheco](https://blytheco.com/blog/posts/sage-intacct-ai-gateway-putting-your-financial-data-to-work-with-the-llms-you-choose)). Partners quoting Sage are plain about the policy: "read access only, governed by Sage Intacct roles and permissions". An AI application cannot post transactions, alter records or change balances ([Baker Tilly](https://www.bakertilly.com/insights/sage-intacct-mcp-server-finance-leaders), September 2026). [CBIZ](https://www.cbiz.com/insights/article/how-ai-is-evolving-in-sage-intacct-and-what-it-means-for-finance-teams) said in July 2026 that it was available to all customers and partners.

Setup is not one click. An administrator enables the gateway and scopes a user and role, and Intacct is not in Anthropic's connector directory ([Audit Friendly](https://auditfriendly.co/answers/sage-intacct-mcp-server-ai-gateway)).

## Data and access

Sage now recommends REST and ships new objects only there ([developer.intacct.com](https://developer.intacct.com/)). The older XML Web Services need a developer licence and sender id arranged through an account manager, and concurrency is capped per company with queued requests ([Web Services](https://developer.intacct.com/web-services/)). Partners report project accounting and fixed assets still lean on XML, so coverage there is thinner. Data stays in Sage's cloud; bulk extracts go through the Data Delivery Service. Pricing is negotiated; third-party estimates of $9,000 to $75,000 a year disagree widely. CBIZ reports Finance AI at no cost to customers through May 2027.

## Agents

This is an incumbent choosing read-only for agents on purpose. The upside is clear: a finance team can let Claude or ChatGPT answer questions over live books without fearing a stray posting. Baker Tilly names what read-only does not fix: data reaching an unvetted AI provider, retention of prompts, misread answers, roles that grant too much, and output mistaken for audit evidence. There is no approval step because there is nothing to approve over MCP; writes go to custom REST applications governed only by their credentials.

## Customers

Intacct reviews average 4.3 out of 5 on G2 (about 4,000 reviews) and Capterra (606 to 684 reviews in 2026). Users praise multi-entity consolidation and dimensional reporting; they complain about cost, the learning curve and support. In its [FY25 results](https://www.sage.com/investors/investor-downloads/press-releases/2025/11/full-year-2025-results-19-november-2025/), Sage reported annual recurring revenue of £2,574m, up 11%, and named Intacct as a driver of 12% North America growth. Sage does not publish an Intacct customer count.

## History

Intacct was founded in 1999 in San Jose and bought by Sage in 2017 for $850m ([Wikipedia](https://en.wikipedia.org/wiki/Intacct)). The REST API reached general availability in 2025, the gateway followed that November, and Sage has said it will bring certified third-party agents into Copilot.

## For ownpurse

Intacct is a design cousin and a likely future source. Sage reached the same first rule as ownpurse: agents read freely. It stopped there, so an agent can describe a problem but cannot propose the journal that fixes it, and every write goes through a bespoke integration with no review step. ownpurse fills that gap. Agents draft a sealed package, a person approves it, and both the reads and the attempt are kept in a record on the owner's machine. The read-only stance also tells you what finance teams in the mid-market will accept: start from the line "it cannot change your books" and earn trust from there.
