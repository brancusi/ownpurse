---
{
 "slug": "merge",
 "name": "Merge",
 "url": "https://www.merge.dev",
 "segment": "unified-api",
 "status": "active",
 "founded": "2020",
 "hq": "San Francisco, USA (offices in San Francisco, New York and Berlin)",
 "backing": "Venture-backed: $4.5M seed led by NEA; $55M Series B led by Accel with NEA and Addition (24 October 2022), bringing total funding to $75M. No later round confirmed.",
 "tagline": "A unified API across HR, recruiting, CRM, ticketing, file storage, accounting and more, now with an MCP tool-calling layer for agents (Agent Handler), an LLM gateway and an employee AI product.",
 "scope": "Merge normalises data from 260+ integrations in 8 categories into Common Models, syncs it on a schedule into its cloud and serves it by REST, with writes, passthrough requests and field mapping. Accounting is one category with 14 platforms. Agent Handler gives AI agents per-user authenticated tools over 190+ connectors through MCP, including read and write tools for Xero, QuickBooks Online, NetSuite and FreshBooks. It keeps current state with a soft-delete flag, not versions, and gives the end business no copy.",
 "domains": [
  "integration",
  "ledger",
  "documents"
 ],
 "customers_served": [
  "developers",
  "smb",
  "mid-market",
  "enterprise"
 ],
 "spectrum": {
  "custody": {
   "score": 1,
   "note": "A synced cache of the source in Merge's cloud, held for Merge's customer; the business owner holds no copy."
  },
  "locality": {
   "score": 1,
   "note": "Multi-tenant cloud (US, EU, APAC); a single-tenant or self-hosted option is 'available for purchase' at enterprise level; Agent Handler Enterprise offers on-prem."
  },
  "history": {
   "score": 1,
   "note": "Current state with remote_was_deleted soft deletes and modified_after; enhanced delete detection is a paid add-on; no prior versions."
  },
  "openness": {
   "score": 2,
   "note": "Public docs with clean Markdown and llms.txt, open-source SDKs and MCP server; closed service."
  },
  "breadth": {
   "score": 2,
   "note": "Wide across business software, narrow for finance: accounting and file storage, no banks or payments."
  },
  "autonomy": {
   "score": 3,
   "note": "Its customers' software and agents write through it unattended; Tool Packs and guardrails bound what an agent can do, no per-write approval."
  },
  "agent_readiness": {
   "score": 4,
   "note": "Hosted MCP (Agent Handler), an open-source MCP server for the Unified API, a CLI, agent self-signup, scoped tool packs and agent-facing docs."
  },
  "integration_ease": {
   "score": 4,
   "note": "Self-serve sign-up, 3 free production linked accounts, published prices, docs written for agents as well as people."
  },
  "market": {
   "score": 3,
   "note": "B2B software companies from startups to enterprise (Ramp, Brex, Navan, Revolut, OpenAI and Perplexity on Agent Handler)."
  }
 },
 "api": {
  "public": true,
  "style": "REST with JSON per category (for accounting, https://api.merge.dev/api/accounting/v1, plus EU and APAC hosts); webhooks plus polling with modified_after; async writes with Idempotency-Key; /meta endpoint for per-integration write fields; authenticated passthrough to the source API.",
  "auth": "Bearer API key for the customer plus an X-Account-Token per linked account; end users connect through Merge Link (OAuth to the source).",
  "limits": "Per linked account: 100 requests/minute on Launch, 400 on Professional, 600 on Enterprise. Sync frequency daily on Launch, customisable above; Monthly, Quarterly and Manual sync plans use sync credits.",
  "cost": "Launch: 3 production linked accounts free, $650/month for up to 10, then $65 per linked account; Professional and Enterprise by contract. Data usage listed as unlimited. Remote Data needs Professional or Enterprise; enhanced deleted data detection and single-tenant hosting are paid add-ons with no published price (October 2026).",
  "export": "No owner export. Merge's customer reads its end users' data by API; the end business has nothing to download from Merge.",
  "notes": "remote_data returns the source's raw response alongside the Common Model, off by default, enabled per model, and refreshed only when a Common Model field changes. Log retention is 3 days on Launch, 30 on Professional, 90+ on Enterprise."
 },
 "pricing": {
  "model": "Per linked account (one customer's connection to one source) for the Unified API; credit-based subscription for Agent Handler.",
  "entry": "Free for 3 production linked accounts; Agent Handler Free at $0 with 2,000 credits a month.",
  "top": "Enterprise by contract; Agent Handler Pro $1,000/month for 25,000 credits (up to 100,000 selectable), Enterprise custom.",
  "notes": "Launch is $650/month for up to 10 linked accounts and $65 for each one after (October 2026). Credits per Agent Handler tool call are not stated on the pricing page."
 },
 "agents": {
  "mcp": "official",
  "ai_features": "Agent Handler (MCP tool calling with per-user auth, Tool Packs, a Security Gateway that redacts or blocks sensitive arguments, AI guardrails in plain English, audit logs); an open-source merge-mcp server over the Unified API; AI-generated JSONata for field mapping; Merge Gateway for LLM routing; Merge for Workforce (August 2026); a 'Universal Context Layer' (September 2026).",
  "agent_writes": "Yes. The Agent Handler Xero connector exposes create and update tools for accounts, bank transactions, contacts, credit notes and invoices, and archive for contacts; QuickBooks Online, NetSuite and FreshBooks connectors exist too. An agent can sign itself up with one unauthenticated POST and get an organisation-wide key.",
  "approval": "No per-call human approval gate found. Control is by Tool Pack (which tools an agent surface can reach, for example a read-only pack), Security Gateway rules and guardrails that allow, redact or block, and an audit trail. A human claims an agent-created organisation later through a claim URL.",
  "notes": "Merge's own pitch is that most MCP servers are unmaintained and unsafe for production, and that the fix is a governed, hosted layer. It is the furthest along of the unified APIs in making agents first-class users."
 },
 "integration": {
  "scope": "260+ documented integrations in 8 Unified API categories and 190+ Agent Handler connectors (as of August 2026). Accounting: Clear Books, FreeAgent, FreshBooks, Dynamics 365 Business Central, Moneybird, NetSuite, Oracle Fusion Cloud ERP, QuickBooks Online, Sage Business Cloud, Sage Intacct, Wave, Workday, Xero and Zoho Books (14).",
  "notes": "No QuickBooks Desktop in the accounting list. Two ids per record (id and remote_id). Field Mapping and Remote Field Classes reach custom fields the Common Model lacks."
 },
 "preference": {
  "scale": "Over 3,000 companies at the October 2022 Series B, with ARR up about 26-fold in the prior year; 'thousands of companies' on its site in 2026.",
  "ratings": "G2 and Capterra pages could not be read in this pass; no figures confirmed.",
  "sentiment": "Not enough independent review data was readable to summarise. From primary sources: the self-serve start and documentation are strong; raw data, faster sync and full delete detection sit behind higher plans and add-ons, which competitors use against it in comparison posts."
 },
 "timeline": [
  {
   "date": "2020",
   "event": "Founded by Shensi Ding and Gil Feig."
  },
  {
   "date": "2021",
   "event": "Announces a $4.5M seed led by NEA and launches its Accounting API (exact date not confirmed)."
  },
  {
   "date": "2022-10-24",
   "event": "Raises $55M Series B led by Accel, total $75M; over 3,000 customers; launches a free tier for startups."
  },
  {
   "date": "2026-06-02",
   "event": "Announces Agent Handler coming to the Microsoft Agent Store (reported; secondary source)."
  },
  {
   "date": "2026-08-18",
   "event": "Launches Merge for Workforce, governed AI access for employees."
  },
  {
   "date": "2026-09-10",
   "event": "Announces a 'Universal Context Layer' for AI tools and agents."
  }
 ],
 "relation": "neighbour",
 "relation_note": "Merge serves software companies that want their customers' accounting data, and agents that act on it, from Merge's cloud. ownpurse serves the owner of the books on their own machine. The two meet in agents writing to Xero: Merge lets them write directly behind guardrails; ownpurse makes every write a sealed package a person approves.",
 "borrow": "Docs written for agents (Markdown for every page, llms.txt, an agent quickstart), two ids per record, the /meta endpoint that tells a writer which fields a given source needs, Idempotency-Key on writes, and Tool Packs as a named, versioned set of what an agent may do.",
 "avoid": "The common model as the product with the source's own data as an optional, plan-gated side channel refreshed only when a modelled field changes; delete detection sold as an add-on; agent writes to a ledger with no human approval step.",
 "sources": [
  {
   "title": "Merge pricing",
   "url": "https://www.merge.dev/pricing"
  },
  {
   "title": "Merge Agent Handler pricing",
   "url": "https://www.merge.dev/pricing/agent-handler"
  },
  {
   "title": "Merge docs index (llms.txt)",
   "url": "https://docs.merge.dev/llms.txt"
  },
  {
   "title": "Merge Accounting API overview",
   "url": "https://docs.merge.dev/merge-unified/accounting/overview.md"
  },
  {
   "title": "Merge accounting integrations",
   "url": "https://docs.merge.dev/merge-unified/accounting/integrations/overview.md"
  },
  {
   "title": "Merge Remote Data",
   "url": "https://docs.merge.dev/merge-unified/supplemental-data/remote-data.md"
  },
  {
   "title": "Merge Invoice object",
   "url": "https://docs.merge.dev/merge-unified/accounting/common-models/invoices/the-invoice-object.md"
  },
  {
   "title": "Merge rate limits",
   "url": "https://docs.merge.dev/merge-unified/hris/merge-api-basics/rate-limits.md"
  },
  {
   "title": "Merge MCP (Unified API)",
   "url": "https://docs.merge.dev/merge-unified/specifications/model-context-protocol-mcp.md"
  },
  {
   "title": "Merge Agent Handler overview",
   "url": "https://docs.merge.dev/merge-agent-handler/overview.md"
  },
  {
   "title": "Merge Agent Handler: how it works",
   "url": "https://docs.merge.dev/merge-agent-handler/how-it-works.md"
  },
  {
   "title": "Merge Agent Handler: Xero connector",
   "url": "https://docs.merge.dev/merge-agent-handler/connectors/xero.md"
  },
  {
   "title": "Merge Agent Handler: agent quickstart",
   "url": "https://docs.merge.dev/merge-agent-handler/setup/agent-quickstart.md"
  },
  {
   "title": "Merge Agent Handler product page",
   "url": "https://www.merge.dev/products/merge-agent-handler"
  },
  {
   "title": "Merge raises $55M Series B (October 2022)",
   "url": "https://www.prnewswire.com/news-releases/merge-raises-55m-series-b-from-accel-launches-new-free-tier-for-startups-301657402.html"
  },
  {
   "title": "Announcing Merge's $4.5M seed round and Accounting API",
   "url": "https://www.merge.dev/blog/announcing-merges-4-5m-seed-round-and-accounting-api"
  },
  {
   "title": "Merge about",
   "url": "https://www.merge.dev/about"
  },
  {
   "title": "Merge blog",
   "url": "https://www.merge.dev/blog"
  }
 ],
 "unverified": [
  "Any funding after the October 2022 Series B (none found; web search budget was exhausted during this pass).",
  "The launch date of Agent Handler (believed 2025).",
  "The Microsoft Agent Store announcement date (2 June 2026) comes from a secondary summary, not Merge's own page.",
  "The earlier survey's claim that enhanced delete detection runs a full sync every 3 days; the pricing page confirms the add-on but gives no cadence or price.",
  "Exact date of the seed round and Accounting API launch.",
  "Current customer count and ARR.",
  "G2 and Capterra ratings (pages not readable).",
  "How many credits an Agent Handler tool call uses."
 ],
 "researched": "2026-10-10"
}
---

## What it is

Merge, founded in 2020 by Shensi Ding and Gil Feig, sells a unified API: one integration that reaches hundreds of business systems in 8 categories, from HR and recruiting to CRM, file storage and accounting. Since 2025 it has added three products for AI: [Agent Handler](https://docs.merge.dev/merge-agent-handler/overview.md), which gives agents authenticated tools over MCP; Gateway, which routes LLM calls; and Merge for Workforce, for employees' AI use. Its [docs index](https://docs.merge.dev/llms.txt) counts 260+ Unified API integrations and 190+ Agent Handler connectors as of August 2026.

## How it works

An end user connects a source through Merge Link. Merge syncs the data on a schedule into its cloud (US, EU or APAC), maps it to a Common Model and serves it by REST. You poll with `modified_after` and listen to webhooks. Every record has Merge's `id` and the source's `remote_id`. Deletes show as `remote_was_deleted: true`; [enhanced deleted data](https://www.merge.dev/pricing) detection is a paid add-on. A change overwrites the record: there are no versions.

The source's own response is available as [`remote_data`](https://docs.merge.dev/merge-unified/supplemental-data/remote-data.md), but only on Professional and Enterprise, off by default, enabled per model, and "updates during a sync only when a Common Model field changes". For anything outside the model there is an authenticated passthrough to the source API. Writes are POST and PATCH, sync or async, with an Idempotency-Key and a `/meta` endpoint that tells you which fields each source requires.

Accounting covers [14 platforms](https://docs.merge.dev/merge-unified/accounting/integrations/overview.md), including Xero, QuickBooks Online, NetSuite, Sage Intacct, FreeAgent and Zoho Books.

## Data and access

Pricing is public. Launch gives 3 production linked accounts free, then $650 a month for up to 10 and $65 for each one after, with daily sync, 100 requests a minute per linked account and 3 days of logs. Professional and Enterprise are by contract with 400 and 600 requests a minute and customisable sync. A single-tenant or self-hosted deployment is for sale. The business whose books are synced gets no export.

## Agents

This is where Merge leads its segment. Agent Handler is a hosted MCP server with per-user OAuth, Tool Packs that name what an agent surface may reach, a Security Gateway that redacts or blocks sensitive arguments, plain-English guardrails and an audit log. Its [Xero connector](https://docs.merge.dev/merge-agent-handler/connectors/xero.md) can create and update invoices, bank transactions, credit notes, contacts and accounts. An agent can [sign itself up](https://docs.merge.dev/merge-agent-handler/setup/agent-quickstart.md) with one unauthenticated request and hand a person a claim link later. There is also an open-source [merge-mcp](https://github.com/merge-api/merge-mcp) server over the Unified API. [Pricing](https://www.merge.dev/pricing/agent-handler) runs from free (2,000 credits a month) to $1,000 a month for 25,000 credits. No human approval step before a tool call was found: control is by scope and filter.

## Customers

Merge reported over 3,000 customers at its [Series B](https://www.prnewswire.com/news-releases/merge-raises-55m-series-b-from-accel-launches-new-free-tier-for-startups-301657402.html) in October 2022. Its site shows Ramp, Brex, Navan, Revolut, Airwallex and Drata, and OpenAI and Perplexity for Agent Handler. Review-site ratings could not be confirmed in this pass.

## History

An NEA-led $4.5M seed launched the Accounting API; Accel led a $55M Series B in October 2022 for a $75M total. Through 2026 the company has shifted weight from integrations to AI plumbing: Agent Handler, Gateway, Workforce, and a "Universal Context Layer" in September 2026.

## For ownpurse

Merge is the purest example of the common-model-first pattern ownpurse inverts. It treats the source's own data as a debugging side channel; ownpurse treats it as the facts and the common model as a projection. It is also the clearest picture of how agent writes to a ledger are being sold in 2026: direct, governed by scopes and filters, logged after the fact. ownpurse's answer is the sealed package a person approves before anything leaves. There is a lot to take from Merge's craft, especially docs written for agents and the `/meta` endpoint for writes. Merge is not a source ownpurse should read through: it would add a hop, a cost and a lossy model between the owner and Xero.
