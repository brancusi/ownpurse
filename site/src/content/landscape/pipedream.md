---
{
 "slug": "pipedream",
 "name": "Pipedream",
 "url": "https://pipedream.com",
 "segment": "ipaas-middleware",
 "status": "acquired",
 "founded": "2019 [unverified]",
 "hq": "San Francisco, CA (site footer); legal notices to Workday's address, 6110 Stoneridge Mall Road, Pleasanton, CA",
 "backing": "Workday signed a definitive agreement to acquire Pipedream on 19 November 2025, price undisclosed, expected to close in Workday's fiscal Q4 2026 (ending 31 January 2026). Workday has published no completion release as of 10 October 2026, but Pipedream's terms now give Workday's Pleasanton address for legal notice and refer to Workday and Sana Enterprise, which points to a closed deal. Earlier venture funding (a reported US$20M Series A in 2022) is unverified.",
 "tagline": "A hosted integration layer that gives developers and AI agents managed auth, more than 10,000 prebuilt tools and an MCP server over 3,000+ APIs.",
 "scope": "Pipedream Connect handles OAuth and API keys for your end users, runs prebuilt actions and triggers, proxies raw API calls and serves per-app MCP servers. It now includes Conduit, a gateway that lets a company's staff connect assistants to work apps. It keeps no record of the data that passes through it: Connect does not store request or response bodies. Its original low-code Workflows product is in maintenance mode and shuts down on 31 March 2027.",
 "domains": [
  "integration"
 ],
 "customers_served": [
  "developers",
  "mid-market",
  "enterprise"
 ],
 "spectrum": {
  "custody": {
   "score": 1,
   "note": "Holds no copy of your data (Connect stores no payloads), but holds your OAuth grants and credentials; the record stays with each source vendor."
  },
  "locality": {
   "score": 1,
   "note": "Connect is cloud only on AWS us-east-1; only Conduit, the employee gateway, can be run with Docker on your own infrastructure."
  },
  "history": {
   "score": 0,
   "note": "Nothing is kept: no payloads in Connect, and Workflows event history follows short retention rules."
  },
  "openness": {
   "score": 2,
   "note": "Public API and SDKs; the component registry is on GitHub under the Pipedream Source Available License, which bars commercial use and competing services, so not open source."
  },
  "breadth": {
   "score": 3,
   "note": "Reaches 3,000+ APIs across every domain, including Xero (39 actions, 4 triggers), but unifies nothing into one record."
  },
  "autonomy": {
   "score": 3,
   "note": "Agents call write tools directly (create invoice, create bank transaction); any approval step is up to the developer."
  },
  "agent_readiness": {
   "score": 4,
   "note": "Built for agents: remote MCP servers per app, TypeScript SDK, managed auth, llms.txt docs."
  },
  "integration_ease": {
   "score": 4,
   "note": "Self-serve, free in development, documented quickstarts, hosted Connect Link for user auth."
  },
  "market": {
   "score": 3,
   "note": "Sold to developers and AI companies (Lindy, Perplexity, Relevance AI); now part of Workday's enterprise agent platform."
  }
 },
 "api": {
  "public": true,
  "style": "REST (Connect API) plus remote MCP servers and a raw API proxy; TypeScript SDK",
  "auth": "OAuth client credentials for the developer; short-lived Connect tokens per external user; end-user auth through Pipedream's or your own OAuth clients",
  "limits": "Create Connect Token 100/min per external user; account management 2,000 per 5 min per project; list tools 3,000 per 5 min per project; custom per-user rate limits via POST /rate_limits; higher limits on request",
  "cost": "Free for development. Startup US$99/month billed annually or US$150 monthly, 10,000 credits and 100 external users included, then US$0.012 per credit and US$2 per external user per month. A credit is 30 seconds of compute for an action, MCP tool call, proxy request or trigger event; management calls are free. Business is custom.",
  "export": "Workflows: export each project's workflow structure as JSON, one project at a time, before deletion by 30 April 2027. Connect holds no business data to export.",
  "notes": "Hosted on AWS us-east-1; SOC 2 Type 2 report on request; HIPAA BAAs available. The terms say users keep rights in their data and do not mention AI training."
 },
 "pricing": {
  "model": "Platform fee plus metered usage credits and per-external-user fees",
  "entry": "Free in development; US$99/month (annual) or US$150/month for production",
  "top": "Business, custom annual pricing with SSO, SCIM, HIPAA, static egress IPs and SLAs",
  "notes": "Prices from pipedream.com/pricing, read 2026-10-10. MCP is included on every plan and meters like any other call. Conduit Cloud is free for up to 100 users."
 },
 "agents": {
  "mcp": "official",
  "ai_features": "Per-app remote MCP servers exposing 10,000+ tools generated from the action registry; Conduit gateway connecting Claude, ChatGPT and Cursor to work apps with admin policies and audit logs.",
  "agent_writes": "Yes, directly: write tools (for Xero: create sales invoice, bill, payment, bank transaction, credit note, contact) run against the user's connected account as soon as the agent calls them.",
  "approval": "None built into Connect for individual writes. Conduit lets administrators pick which tools are available and requires explicit consent for administrative scopes, but this is an access policy, not a per-write review.",
  "notes": "Credentials are never exposed to the model or client code; every call goes through Pipedream's servers."
 },
 "integration": {
  "scope": "3,000+ APIs, 10,000+ prebuilt tools and triggers (Workday release, November 2025); Xero has 39 actions and 4 triggers (new or updated contact, invoice, quote, and webhook events).",
  "notes": "Each app is a thin wrapper over the provider's API, so you still meet each provider's rate limits, egress fees and terms. There is no normalized accounting model and no sync state."
 },
 "preference": {
  "scale": "More than 5,000 customers and tens of thousands of users; 1M+ accounts connected (Workday release 19 November 2025; Pipedream homepage, October 2026). GitHub registry: 11.7k stars, 5.8k forks.",
  "ratings": "Trustpilot 2.7/5 from 10 reviews (6 one-star), read 2026-10-10. G2 and Capterra pages could not be fetched.",
  "sentiment": "Developers praise the flexibility (real code, custom integrations, a better fit for them than Zapier) and the breadth of prebuilt components. Complaints on Trustpilot are about billing after cancellation, slow support, credits running out fast and pushed upgrades. Workflows users now have to migrate by 31 March 2027."
 },
 "timeline": [
  {
   "date": "2019",
   "event": "Pipedream founded by Tod Sacerdoti [unverified year]"
  },
  {
   "date": "2022",
   "event": "Reported Series A [unverified]"
  },
  {
   "date": "2025",
   "event": "Repositions Connect and MCP as 'the integration layer for AI agents'"
  },
  {
   "date": "2025-11-19",
   "event": "Workday signs a definitive agreement to acquire Pipedream, announced at Workday Rising EMEA"
  },
  {
   "date": "2026-01-31",
   "event": "Expected close (end of Workday fiscal Q4 2026); no completion release found"
  },
  {
   "date": "2026-08",
   "event": "Conduit releases ship (v0.9.6, 26 August 2026), self-hosted with Docker, then Conduit Cloud"
  },
  {
   "date": "2026",
   "event": "Workflows and String put into maintenance mode, no new signups"
  },
  {
   "date": "2027-03-31",
   "event": "Workflows and String shut down; Workflows data deleted by 30 April 2027"
  }
 ],
 "relation": "neighbour",
 "relation_note": "Pipedream is the plumbing an agent might use to reach Xero or QuickBooks, not a place where books are kept. It solves auth and tool calls, and keeps nothing: no record, no history and no review step before a write. ownpurse is the opposite layer, the record and the gate. Its shutdown of Workflows after the acquisition is also a small lesson about building on someone else's runtime.",
 "borrow": "Managed-auth ergonomics (a hosted connect link, per-user external ids, tokens refreshed for you); generating MCP tools from a declarative action registry; per-user rate limits that the caller sets; metering only the calls that do work, not management calls.",
 "avoid": "Letting agents write directly to a ledger with no review; holding the user's credentials on a hosted service; a source-available licence that reads as open source; a product line that can be shut down with six months' notice and its data deleted.",
 "sources": [
  {
   "title": "Workday Signs Definitive Agreement to Acquire Pipedream (19 November 2025)",
   "url": "https://newsroom.workday.com/2025-11-19-Workday-Signs-Definitive-Agreement-to-Acquire-Pipedream"
  },
  {
   "title": "Pipedream homepage",
   "url": "https://pipedream.com"
  },
  {
   "title": "Pipedream pricing",
   "url": "https://pipedream.com/pricing"
  },
  {
   "title": "Workflows and String are shutting down on March 31, 2027",
   "url": "https://pipedream.com/docs/workflows.md"
  },
  {
   "title": "Pipedream Connect overview",
   "url": "https://pipedream.com/docs/connect.md"
  },
  {
   "title": "Pipedream MCP servers",
   "url": "https://pipedream.com/docs/connect/mcp.md"
  },
  {
   "title": "Connect API rate limits",
   "url": "https://pipedream.com/docs/connect/api-reference/rate-limits.md"
  },
  {
   "title": "Conduit overview",
   "url": "https://pipedream.com/docs/conduit/index.md"
  },
  {
   "title": "Conduit Cloud",
   "url": "https://pipedream.com/docs/conduit/cloud.md"
  },
  {
   "title": "Conduit changelog",
   "url": "https://pipedream.com/docs/conduit/changelog.md"
  },
  {
   "title": "Privacy and security",
   "url": "https://pipedream.com/docs/privacy-and-security.md"
  },
  {
   "title": "Pricing FAQ",
   "url": "https://pipedream.com/docs/pricing/faq.md"
  },
  {
   "title": "Pipedream terms of service",
   "url": "https://pipedream.com/terms"
  },
  {
   "title": "Pipedream Xero integration",
   "url": "https://pipedream.com/apps/xero-accounting-api"
  },
  {
   "title": "PipedreamHQ/pipedream on GitHub",
   "url": "https://github.com/PipedreamHQ/pipedream"
  },
  {
   "title": "Pipedream Source Available License",
   "url": "https://raw.githubusercontent.com/PipedreamHQ/pipedream/master/LICENSE"
  },
  {
   "title": "Pipedream on Trustpilot",
   "url": "https://www.trustpilot.com/review/pipedream.com"
  }
 ],
 "unverified": [
  "Founding year (2019) and founding team beyond Tod Sacerdoti",
  "Venture funding history (reported US$20M Series A, 2022) and investors",
  "Whether and when the Workday acquisition formally closed (no completion release found; the terms point to Workday)",
  "Acquisition price (undisclosed)",
  "G2 and Capterra ratings (pages blocked)",
  "Reddit and forum sentiment (web search budget exhausted)",
  "Conduit's licence (the docs do not say)",
  "Whether the Startup prices of US$99 annual and US$150 monthly are read correctly (the pricing page text was partly garbled)"
 ],
 "researched": "2026-10-10"
}
---

## What it is

Pipedream is a hosted integration platform. Its current homepage calls it "the integration layer for AI agents": it handles authentication to [3,000+ APIs](https://pipedream.com), offers more than 10,000 prebuilt tools and triggers, and serves them to agents through MCP. Workday [signed a definitive agreement to acquire it](https://newsroom.workday.com/2025-11-19-Workday-Signs-Definitive-Agreement-to-Acquire-Pipedream) on 19 November 2025, to connect Workday's HR and finance data to outside apps so its agents can act. Workday did not disclose the price.

It does well what it sets out to do. A developer can let users connect thousands of apps without building OAuth flows, and an agent can call a prebuilt tool instead of reading API docs.

## How it works

[Connect](https://pipedream.com/docs/connect.md) is the core product today. Your users authorize their accounts through a hosted link or SDK, and Pipedream stores and refreshes the credentials. Your code or agent then runs a prebuilt action, a trigger, or a raw request through the proxy. Pipedream says Connect does not store request or response bodies. [MCP servers](https://pipedream.com/docs/connect/mcp.md) at mcp.pipedream.com expose each app's tools. Credentials are never shown to the model.

The original product, low-code Workflows, is ending. A [notice in the docs](https://pipedream.com/docs/workflows.md) says Workflows and String shut down on 31 March 2027 and all Workflows data, including OAuth tokens, will be deleted by 30 April 2027. The reason given is that Workflows "falls outside" the focus on connecting AI to outside services. The export tool downloads each project's workflow structure as JSON.

[Conduit](https://pipedream.com/docs/conduit/index.md) is newer: a gateway that lets a company's staff connect Claude, ChatGPT or Cursor to work apps. Administrators set policies, and it keeps audit logs. It runs from a Docker image on your own servers, or as [Conduit Cloud](https://pipedream.com/docs/conduit/cloud.md), which is free for up to 100 users.

## Data and access

The API is public and documented, with [published rate limits](https://pipedream.com/docs/connect/api-reference/rate-limits.md). [Pricing](https://pipedream.com/pricing) is free in development. Production costs US$99 a month billed annually (US$150 monthly) for 10,000 credits and 100 external users, then US$0.012 a credit and US$2 per extra user. A credit is 30 seconds of compute. The service runs on AWS us-east-1 and offers a [SOC 2 Type 2 report](https://pipedream.com/docs/privacy-and-security.md). The component registry is [on GitHub](https://github.com/PipedreamHQ/pipedream) (11.7k stars) under the [Pipedream Source Available License](https://raw.githubusercontent.com/PipedreamHQ/pipedream/master/LICENSE), which excludes commercial use and competing hosted services.

Pipedream keeps no record you could export. That is by design, but it means there is no history, no as-of view and no reconciliation.

## Agents

This is where Pipedream is strongest: an official MCP, an SDK, managed auth and llms.txt docs. For Xero it offers [39 actions and 4 triggers](https://pipedream.com/apps/xero-accounting-api), including Create Bank Transaction, Create Payment and Create Credit Note. An agent can call them directly. Connect has no per-write approval step. Conduit's controls decide which tools a member can use and require consent for admin scopes, not review of each write.

## Customers

Workday's release cites more than 5,000 customers and tens of thousands of users. The homepage claims 1M+ connected accounts and names Lindy, Perplexity and Relevance AI. Developers praise its flexibility. Its [Trustpilot](https://www.trustpilot.com/review/pipedream.com) score is 2.7 from only 10 reviews, mostly about billing and support. G2 and Capterra could not be checked.

## History

Pipedream was founded around 2019 by Tod Sacerdoti (year unverified). It started as a developer-friendly alternative to Zapier, moved to Connect and MCP in 2025, and sold to Workday, which also bought Flowise (August 2025) and Sana (September 2025) for its agent platform. Workday published no completion release, but Pipedream's [terms](https://pipedream.com/terms) now give Workday's Pleasanton address for legal notice.

## For ownpurse

Pipedream answers "how does an agent reach Xero", not "where do the books live and who checks them". Both its strengths and its gaps point to ownpurse's place. An agent on Pipedream can create a Xero bank transaction with no record of what was there before and no person approving it. ownpurse keeps the record and the gate. Worth borrowing: hosted-link auth ergonomics, MCP tools generated from a declarative registry, and metering only the calls that do work. Worth remembering: a product built on someone else's runtime can be given six months' notice.
