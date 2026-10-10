---
{
 "slug": "workato",
 "name": "Workato",
 "url": "https://www.workato.com",
 "segment": "ipaas-middleware",
 "status": "active",
 "founded": "2013",
 "hq": "Mountain View, California, US",
 "backing": "Venture-backed, private. Series E of over $200 million at a $5.7 billion valuation (November 2021, reported by TechCrunch); an earlier 2021 round valued it at $1.7 billion (Bloomberg, 12 January 2021). Investors include Altimeter, Insight Partners, Redpoint and ServiceNow.",
 "tagline": "A cloud integration and automation platform that runs workflows (recipes) and AI agents across a company's SaaS apps, now sold as a control plane for enterprise AI.",
 "scope": "Workato moves and transforms data between systems through low-code recipes triggered by events or polling, and lets AI agents (genies) and MCP clients call those recipes as tools. It connects to ERPs and ledgers such as NetSuite and Xero but keeps no ledger or durable record of its own; job data is retained for 30 days (Standard, Business) or 90 days (Enterprise). It is cloud only, with an on-prem agent to reach systems behind a firewall.",
 "domains": [
  "integration",
  "close",
  "reporting"
 ],
 "customers_served": [
  "mid-market",
  "enterprise",
  "developers"
 ],
 "spectrum": {
  "custody": {
   "score": 1,
   "note": "Data passes through Workato's cloud; the books stay in each source and the owner holds no copy of their own."
  },
  "locality": {
   "score": 1,
   "note": "Cloud only (regional data centres); an on-prem agent reaches local systems but the platform does not run on your machine."
  },
  "history": {
   "score": 1,
   "note": "Job history kept 30 or 90 days by edition, job details truncated past 1 MB; a run log, not a versioned record."
  },
  "openness": {
   "score": 1,
   "note": "Closed platform; a Developer API exists only on some plans, and custom connectors can be shared as open or closed source."
  },
  "breadth": {
   "score": 3,
   "note": "Claims 17,000+ apps, APIs, databases, files and MCP servers, but it connects them rather than holding them in one place."
  },
  "autonomy": {
   "score": 4,
   "note": "Recipes write to target systems unattended; approval steps are something you build in, not a default."
  },
  "agent_readiness": {
   "score": 4,
   "note": "Enterprise MCP servers built from recipes and API collections, a Developer API MCP server, scoped API clients, verified user access and logged MCP actions."
  },
  "integration_ease": {
   "score": 3,
   "note": "Low-code builder and pre-built connectors praised by reviewers; access is sales-led and the API is plan-gated."
  },
  "market": {
   "score": 4,
   "note": "Sold to large enterprises; logos include Cisco, Vodafone, NASDAQ and Intuit."
  }
 },
 "api": {
  "public": true,
  "style": "REST, JSON; Developer API, Embedded API, Headless API, plus MCP servers",
  "auth": "API clients with roles and project scopes issue Bearer tokens; legacy x-user-token keys removed from 14 October 2025; MCP uses OAuth and optional verified end-user credentials",
  "limits": "Developer API mostly 60 requests a minute (some 1,000); Headless API 50 a second; webhooks 20 events a second with a 9,000 burst; job timeout 90 minutes; MCP tool calls 30 s synchronous, 300 s total",
  "cost": "Not published; quote-based. Workato meters tasks (it claims 100M+ tasks a month across customers). A PeerSpot summary cites roughly $120,000-130,000 a year for an enterprise deal [reviewer-reported].",
  "export": "Recipes, connections and manifests can be deployed between environments through the API (packages); there is no data export because Workato is not meant to keep the data.",
  "notes": "Limits and retention from docs.workato.com/en/limits.html, read 2026-10-10."
 },
 "pricing": {
  "model": "Annual contract, quote-based, metered by tasks and editions",
  "entry": "Free trial; no published price",
  "top": "Enterprise edition, quote-based",
  "notes": "The pricing page shows no figures and directs you to sales (2026-10-10). Reviewers name price as the main complaint."
 },
 "agents": {
  "mcp": "official",
  "ai_features": "Agent Studio genies (LLM, job description, knowledge base, skills built from recipes) in Slack, Teams and Workato GO; Enterprise MCP servers; AI Gateway; AIRO multi-agent system; Enterprise Data MCP (verified queries over a semantic model) used internally.",
  "agent_writes": "Yes. Genie skills and MCP tools run recipes that create and update records in connected apps, optionally under the end user's own credentials.",
  "approval": "Optional. Agent Studio docs say you can set approval workflows for critical actions, and Workflow apps add request and approval steps (for example invoice and PO approvals) to recipes. Not a default gate on every write.",
  "notes": "MCP is available in US, EU, AU, JP, SG, IL, KR and UK data centres, not CN. All MCP actions are logged with identity."
 },
 "integration": {
  "scope": "Claims 17,000+ apps, APIs, databases, files and MCP servers (home page, 2026-10-10); a Xero connector with triggers on contacts, invoices, payments and employees and actions including manual journals and payments; NetSuite, Salesforce, Workday, ServiceNow.",
  "notes": "Connector SDK for custom connectors, private by default, shareable through a community library; a universal HTTP connector for anything else."
 },
 "preference": {
  "scale": "Home page (2026-10-10): 100M+ tasks automated a month; control plane for AI agents used by over 100,000 employees. No customer count published.",
  "ratings": "PeerSpot: 4.1 of 5 from 28 reviews, 100% would recommend, ranked #2 in iPaaS (September 2026). Workato's own site states 4.9 on Gartner Peer Insights and 4.7 on G2 (counts not shown). Named a Gartner Peer Insights Customers' Choice for iPaaS in 2026.",
  "sentiment": "Praised for a low-code builder, pre-built connectors and fast delivery ('done within a week, what we thought might take us two months'). Criticised for price and add-ons, struggles with very large data volumes and the 90-minute job timeout, an on-prem agent that can be unstable, and weak debugging and unit testing."
 },
 "timeline": [
  {
   "date": "2013",
   "event": "Founded by Vijay Tella, Gautham Viswanathan, Harish Shetty and Dimitris Kogias"
  },
  {
   "date": "2021-01-12",
   "event": "Round valuing the company at $1.7 billion, led by Altimeter and Insight (Bloomberg)"
  },
  {
   "date": "2021-11-10",
   "event": "Series E of over $200 million at a $5.7 billion valuation"
  },
  {
   "date": "2023",
   "event": "Lays off about 10% of staff; opens an Australian data centre"
  },
  {
   "date": "2025-03",
   "event": "Acquires DeepConverse (support automation)"
  },
  {
   "date": "2025-08",
   "event": "Acquires XMAD.ai"
  },
  {
   "date": "2025-10-14",
   "event": "Legacy user-token API keys removed in favour of scoped API clients"
  },
  {
   "date": "2026-09-18",
   "event": "Publishes how it built Enterprise Data MCP, a verified-query analytics agent"
  },
  {
   "date": "2026-09-21",
   "event": "Named a Customers' Choice in the 2026 Gartner Peer Insights for iPaaS"
  }
 ],
 "relation": "neighbour",
 "relation_note": "Workato is how a large company wires its finance stack together: data moves between NetSuite, Salesforce, banks and Slack in flight, and nothing durable is kept in the middle. ownpurse does the opposite job: it keeps the record and moves nothing without approval. They do not compete for the same buyer.",
 "borrow": "Verified user access (an agent acts with the person's own credentials, not a shared token); per-server rate limits and an audit log of every MCP action with identity; recipe packages promoted between environments; verified queries with the answer path logged (exact match, adapted, or generated and flagged), a good pattern for ownpurse's named views.",
 "avoid": "Calling a 30- or 90-day job log history; truncating payloads; approval as an optional add-on rather than the only path to a write; opaque task-metered pricing that puts the tool out of reach of small businesses.",
 "sources": [
  {
   "title": "Workato home page",
   "url": "https://www.workato.com/"
  },
  {
   "title": "Workato pricing",
   "url": "https://www.workato.com/pricing"
  },
  {
   "title": "Workato Enterprise MCP docs",
   "url": "https://docs.workato.com/en/mcp.html"
  },
  {
   "title": "Workato Developer API docs",
   "url": "https://docs.workato.com/en/workato-api.html"
  },
  {
   "title": "Workato limits",
   "url": "https://docs.workato.com/en/limits.html"
  },
  {
   "title": "Workato Agent Studio docs",
   "url": "https://docs.workato.com/en/agentic/agent-studio.html"
  },
  {
   "title": "Workato Workflow apps docs",
   "url": "https://docs.workato.com/en/workflow-apps.html"
  },
  {
   "title": "Workato Connector SDK docs",
   "url": "https://docs.workato.com/en/developing-connectors/sdk.html"
  },
  {
   "title": "Workato Xero connector docs",
   "url": "https://docs.workato.com/en/connectors/xero.html"
  },
  {
   "title": "How Workato built Enterprise Data MCP (2026-09-18)",
   "url": "https://www.workato.com/the-connector/how-workato-built-enterprise-data-mcp/"
  },
  {
   "title": "Workato for finance",
   "url": "https://www.workato.com/solutions/finance"
  },
  {
   "title": "Workato on Wikipedia",
   "url": "https://en.wikipedia.org/wiki/Workato"
  },
  {
   "title": "PeerSpot Workato reviews",
   "url": "https://www.peerspot.com/products/workato-reviews"
  },
  {
   "title": "PeerSpot iPaaS category",
   "url": "https://www.peerspot.com/categories/integration-platform-as-a-service-ipaas"
  }
 ],
 "unverified": [
  "Amount of the January 2021 round (only the $1.7 billion valuation was seen)",
  "Enterprise contract cost of $120,000-130,000 a year (PeerSpot reviewer summary, not Workato)",
  "G2 4.7 and Gartner 4.9 ratings are Workato's own claims; review counts not seen (G2, Capterra and Gartner blocked fetches)",
  "Customer count, revenue and employee count not found",
  "Connector SDK language (believed Ruby) not confirmed on the page read",
  "Web search budget was exhausted; Reddit and press coverage not checked"
 ],
 "researched": "2026-10-10"
}
---

## What it is

Workato is a cloud integration and automation platform founded in 2013 and based in Mountain View. You build **recipes**: a trigger in one app (a new invoice, an updated contact) followed by actions in others. Since 2025 it has repositioned itself as "the platform to govern and scale enterprise AI" ([home page](https://www.workato.com/)), with AI agents called genies, Enterprise MCP servers and an AI gateway layered on the same recipe engine.

## How it works

Recipes run in Workato's cloud, in regional data centres. They poll or receive events from connected apps, transform the data and write it elsewhere. Systems behind a firewall are reached through an on-prem agent. Workato keeps job history, not a record: default retention is 30 days on Standard and Business and 90 days on Enterprise, and job details are truncated past 1 MB ([limits](https://docs.workato.com/en/limits.html)). For finance teams it sells faster closes and real-time reporting ([finance page](https://www.workato.com/solutions/finance)); the Xero connector covers contacts, invoices, payments and manual journals ([Xero connector](https://docs.workato.com/en/connectors/xero.html)).

## Data and access

The [Developer API](https://docs.workato.com/en/workato-api.html) is REST and JSON, authenticated with scoped API clients that issue Bearer tokens; legacy user-token keys were removed on 14 October 2025. It manages recipes, connections and jobs and promotes recipe manifests between environments, and it is included only on some plans. Most endpoints allow 60 requests a minute. Pricing is not published ([pricing](https://www.workato.com/pricing)); reviewers on [PeerSpot](https://www.peerspot.com/products/workato-reviews) cite roughly $120,000 to $130,000 a year for an enterprise deal, a figure not confirmed by Workato.

## Agents

This is where Workato is strongest. [Enterprise MCP](https://docs.workato.com/en/mcp.html) turns API collections and recipes into MCP servers whose skills carry input and output schemas; servers enforce per-server rate limits, can run with the end user's own credentials (verified user access) and log every action with identity. [Agent Studio](https://docs.workato.com/en/agentic/agent-studio.html) genies combine a model, a knowledge base and recipe-backed skills, run in Slack, Teams or Workato GO, and can act as MCP clients. Approval is available but optional: the docs say you can set approval workflows for critical actions, and [Workflow apps](https://docs.workato.com/en/workflow-apps.html) add request and approval steps such as invoice approvals. Its [Enterprise Data MCP](https://www.workato.com/the-connector/how-workato-built-enterprise-data-mcp/) answers analytics questions from verified queries first and flags any generated SQL for review.

## Customers

Workato sells to large companies; its home page shows Cisco, Lenovo, Vodafone, NASDAQ, Red Hat, AWS, Aon, Intuit and Coinbase, and claims 100M+ tasks automated a month. On [PeerSpot](https://www.peerspot.com/categories/integration-platform-as-a-service-ipaas) it rates 4.1 of 5 from 28 reviews (September 2026). Reviewers like the low-code builder and the connectors; they dislike the price, the 90-minute job timeout under heavy loads and the on-prem agent.

## History

Workato raised over $200 million at a $5.7 billion valuation in November 2021 ([Wikipedia](https://en.wikipedia.org/wiki/Workato), citing TechCrunch), cut about 10% of staff in 2023, and bought DeepConverse (March 2025) and XMAD.ai (August 2025) as it moved toward agents. It is the most recent of the three big attempts to make one platform the place where a company's systems meet, and it has bet that the meeting place is now where agents are governed.

## For ownpurse

Workato shows how larger companies solve cross-system wrangling: buy a central integration team and a platform that moves data in flight, and leave each system as its own source of truth. No one holds the combined history; when you want to know what an invoice said last March, you ask the source, if it kept it. A small business or a household cannot afford this, and would not want to run it. ownpurse's gap is the piece Workato deliberately does not keep: a durable, owner-held record of what every source said. What to borrow is the agent governance (scoped tools, the person's own credentials, an audit line per call). What to avoid is approval as an option rather than the only way a write goes out.
