---
{
 "slug": "nango",
 "name": "Nango",
 "url": "https://www.nango.dev",
 "segment": "connector-framework",
 "status": "active",
 "founded": "2022 (the codebase began in 2020 as Bearer's open-source Pizzly OAuth proxy)",
 "hq": "San Francisco, California (per Y Combinator)",
 "backing": "Y Combinator W23. No funding amounts were confirmed from primary sources.",
 "tagline": "A code-first integration platform that handles auth, syncs, actions and MCP tools for 1,000+ APIs, for products and their agents.",
 "scope": "Nango manages OAuth and API keys for your users' accounts, proxies requests, runs your TypeScript sync and action functions on its infrastructure, and keeps a records cache your app reads as a change stream. It ships pre-built Xero syncs and read and write actions. It does not keep history, model accounting, or give an end user a copy of their own data; it is infrastructure for software companies.",
 "domains": [
  "integration"
 ],
 "customers_served": [
  "developers",
  "smb",
  "mid-market"
 ],
 "spectrum": {
  "custody": {
   "score": 2,
   "note": "Your app copies records into its own database; Nango's cache is an intermediate store that prunes payloads after 30 days and all records after 60 idle days."
  },
  "locality": {
   "score": 1,
   "note": "Cloud by default; the free self-host gives only auth and proxy, and syncs, tools and MCP need Enterprise BYOC or Self-Managed."
  },
  "history": {
   "score": 1,
   "note": "The cache keeps only the latest version of each record, with soft deletes and a cursor-ordered change stream, but no payload history."
  },
  "openness": {
   "score": 2,
   "note": "Source on GitHub under ELv2, integration templates public, full HTTP API; key features are gated off the free self-host."
  },
  "breadth": {
   "score": 3,
   "note": "1,000+ APIs across accounting, CRM, HR, payments and more, but each integration is separate and nothing joins them."
  },
  "autonomy": {
   "score": 2,
   "note": "Actions write to Xero and other APIs directly when your code or agent calls them; no gate of its own."
  },
  "agent_readiness": {
   "score": 4,
   "note": "Scoped agent sessions each expose their own MCP server the agent cannot widen, 7,000+ tools, a management MCP and llms.txt docs."
  },
  "integration_ease": {
   "score": 4,
   "note": "Self-serve free tier, a Xero connection in minutes, documented functions SDK, open templates to fork."
  },
  "market": {
   "score": 2,
   "note": "Software companies, mostly startups and growth-stage products such as Replit, Motion and Vapi, building customer-facing integrations."
  }
 },
 "api": {
  "public": true,
  "style": "REST HTTP API and Node SDK for connections, proxy, records and syncs; TypeScript functions SDK for sync and action code; MCP over Streamable HTTP for agent sessions",
  "auth": "Environment secret keys with scopes (for example environment:agent_sessions:write); session tokens as bearer auth for agent MCP; end users authorize providers through Nango Connect",
  "limits": "Free tier hard-capped at 10 connections, 10 compute hours and 10 GB egress a month; provider limits are handled by Nango's retries and rate-limit logic",
  "cost": "Free $0; Pay-as-you-go $50 a month including $50 of credit, then $0.29 per connection, $0.72 per compute hour and $0.50 per GB egress (ingress free); Enterprise custom, down to about $0.01 a connection at scale (pricing page, read 2026-10-10)",
  "export": "Records through GET /records with cursors (ADDED, UPDATED, DELETED); there is no history to export, and the cache expires.",
  "notes": "Xero: OAuth, pre-built syncs (accounts, bank transactions, contacts, credit notes, invoices, items, organisations, payments, purchase orders, general ledger journals) and create, get, list and update actions. The invoices sync uses If-Modified-Since with includeArchived and turns DELETED and VOIDED invoices into deletes."
 },
 "pricing": {
  "model": "Usage-based: connections, compute time and egress, with a free capped tier and an Enterprise contract",
  "entry": "$0 (10 connections, 10 compute hours, 10 GB a month)",
  "top": "Enterprise custom; a Growth support add-on at $450 a month",
  "notes": "Self-hosting and BYOC are Enterprise only for syncs, tools and MCP; there is no EU-hosted cloud."
 },
 "agents": {
  "mcp": "official",
  "ai_features": "Agent sessions with a per-session MCP server, 7,000+ MCP tools and templates, MCP auth for third-party MCP servers, a management MCP to operate Nango itself, and integrations generated from plain-English descriptions.",
  "agent_writes": "Yes. Action functions such as create-invoice, create-payment and update-bank-transaction for Xero are exposed as tools; a session can be scoped to read-only tools.",
  "approval": "None built in. Scoping is the control: your backend chooses the tools and the agent cannot change its session's scope. Nango's own example sets require_approval to never.",
  "notes": "The older Connection MCP is being replaced by agent sessions, which keep raw credentials out of the agent's reach."
 },
 "integration": {
  "scope": "1,000+ APIs (site and GitHub, read 2026-10-10), with pre-built syncs and actions for many, including Xero.",
  "notes": "Integrations are code: TypeScript functions with zod schemas, checkpoints, deletion tracking and sync partitioning, deployed from a CLI. Templates live in the public NangoHQ/integration-templates repo."
 },
 "preference": {
  "scale": "Nango reports 1M+ integrations and running customer code 150M+ times a month (blog, 2026-06-08); 12.6K GitHub stars and about 130 open issues (2026-10-10). Named customers include Replit, Vapi, Plain, Electric and Motion.",
  "ratings": "Not retrieved: review sites refused automated access on 2026-10-10; no G2 or Capterra count confirmed.",
  "sentiment": "Testimonials on its site praise breadth and speed of building integrations (Motion cites 60+). Independent review themes could not be checked."
 },
 "timeline": [
  {
   "date": "2020-04",
   "event": "Code begins as Pizzly, Bearer's open-source OAuth proxy"
  },
  {
   "date": "2022",
   "event": "Nango founded by Robin Guldener and Bastien Beurier"
  },
  {
   "date": "2022-11-30",
   "event": "v0.3, a rewrite of the codebase"
  },
  {
   "date": "2023",
   "event": "Y Combinator W23"
  },
  {
   "date": "2026-06-08",
   "event": "Reports running untrusted customer code 150M+ times a month"
  },
  {
   "date": "2026-08-06",
   "event": "Reports operating auth for 1,000+ APIs"
  },
  {
   "date": "2026-10-02",
   "event": "Release v0.71.12"
  }
 ],
 "relation": "design-cousin",
 "relation_note": "Nango is the plumbing a SaaS product uses to read and write its customers' Xero and other accounts. ownpurse solves the same sync problems for one owner on their own machine. Its deletion and change-detection rules are directly reusable; its hosted, latest-only cache is the opposite of ownpurse's record.",
 "borrow": "Infer deletes only from a full sweep that completed without an exception, and never act irreversibly on an inferred delete; a payload hash per record so a re-save that changed nothing emits nothing; a change stream with ADDED, UPDATED and DELETED plus a cursor (ownpurse's assert and retract); typed checkpoints saved mid-run; agent sessions scoped by the backend that the agent cannot widen.",
 "avoid": "A cache that keeps only the latest version and expires payloads after 30 days; treating a VOIDED invoice as a deleted one, which loses an accounting fact; write tools wired to agents with require_approval set to never; gating syncs behind an enterprise self-host.",
 "sources": [
  {
   "title": "Nango pricing",
   "url": "https://www.nango.dev/pricing"
  },
  {
   "title": "Records cache",
   "url": "https://nango.dev/docs/guides/functions/syncs/records-cache"
  },
  {
   "title": "Deletion detection",
   "url": "https://nango.dev/docs/guides/functions/syncs/deletion-detection"
  },
  {
   "title": "Checkpoints",
   "url": "https://nango.dev/docs/guides/functions/syncs/checkpoints"
  },
  {
   "title": "Self-host Nango",
   "url": "https://nango.dev/docs/guides/platform/self-hosting/self-hosting"
  },
  {
   "title": "Agent sessions",
   "url": "https://nango.dev/docs/guides/agent-sessions"
  },
  {
   "title": "Xero integration docs",
   "url": "https://nango.dev/docs/api-integrations/xero"
  },
  {
   "title": "Xero integration templates",
   "url": "https://github.com/NangoHQ/integration-templates/tree/main/integrations/xero"
  },
  {
   "title": "NangoHQ/nango on GitHub",
   "url": "https://github.com/NangoHQ/nango"
  },
  {
   "title": "Nango on Y Combinator",
   "url": "https://www.ycombinator.com/companies/nango"
  },
  {
   "title": "Nango blog",
   "url": "https://www.nango.dev/blog"
  }
 ],
 "unverified": [
  "Funding amounts, rounds and investors beyond Y Combinator W23",
  "When the licence became ELv2 (a short-licence update was committed 2023-04-24)",
  "G2, Capterra and Reddit ratings and sentiment",
  "Current headcount (YC lists 6, which may be dated)",
  "Compliance certifications (the site links a trust center without naming SOC 2)"
 ],
 "researched": "2026-10-10"
}
---

## What it is

Nango is an integration platform for software companies. You put its auth flow in your app, your users connect their Xero, HubSpot or Slack, and Nango stores and refreshes the tokens, proxies requests and runs the sync and action code you write. It says it covers 1,000+ APIs ([GitHub](https://github.com/NangoHQ/nango)). The code began in 2020 as Pizzly, Bearer's open-source OAuth proxy; the company was founded in 2022 and went through Y Combinator's W23 batch ([YC](https://www.ycombinator.com/companies/nango)).

## How it works

Integrations are TypeScript functions. A sync function fetches pages and calls `batchSave`, which writes to Nango's [records cache](https://nango.dev/docs/guides/functions/syncs/records-cache). Each record is keyed by the source id and a hash of its payload: the same id and hash means nothing changed. Your app reads `GET /records` as an ordered change stream of ADDED, UPDATED and DELETED, with a cursor. The cache keeps only the latest version, prunes payloads not updated for 30 days and hard-deletes everything after 60 days without a sync run.

[Deletion detection](https://nango.dev/docs/guides/functions/syncs/deletion-detection) is its most careful rule. For incremental syncs you call `batchDelete` when the API reports a delete. For full syncs, `trackDeletesStart` and `trackDeletesEnd` bracket the sweep and Nango diffs what existed before against what was saved, but only if the run finished with no uncaught exception. The docs warn that swallowing an error causes false deletes and advise against hard-deleting anything on Nango's word alone. Checkpoints let a run save progress mid-way and resume.

The Xero templates show this in practice: hourly syncs on `If-Modified-Since` with `includeArchived`, page offsets, the general ledger by `JournalNumber` offset, and DELETED or VOIDED invoices passed to `batchDelete`.

## Data and access

The free tier allows 10 connections, 10 compute hours and 10 GB of egress a month. Pay-as-you-go is $50 a month including $50 of credit, then $0.29 a connection, $0.72 a compute hour and $0.50 a GB ([pricing](https://www.nango.dev/pricing), read 2026-10-10). The source is under the Elastic License 2.0. The [free self-host](https://nango.dev/docs/guides/platform/self-hosting/self-hosting.md) covers only auth and proxy; syncs, tools, webhooks and MCP need Enterprise BYOC or Self-Managed.

## Agents

Nango is arguably the most agent-ready product in this segment. [Agent sessions](https://nango.dev/docs/guides/agent-sessions) give each agent its own MCP server URL and a short-lived token, scoped by your backend; the agent never sees credentials and cannot widen its own scope. Write tools for Xero, such as `create-invoice`, `create-payment` and `update-bank-transaction`, are available. There is no approval step: the example code sets `require_approval` to never, and confirmation is left to the client.

## Customers

Product teams building customer-facing integrations and agent tools; its site names Replit, Vapi, Plain, Electric and Motion. It reports 1M+ integrations and customer code running 150M+ times a month (June 2026). Review sites could not be read for this dossier, and no funding figures were confirmed.

## History

Nango grew out of the unified-API wave but took the other side of that bet: rather than one normalized model for every CRM or ledger, give developers raw access and the plumbing (auth, retries, scheduling, a cache), and let them write the mapping. Its 2026 blog is almost entirely about agents, which is where it now sells.

## For ownpurse

Nango's sync rules are the ones ownpurse should keep. Only a completed sweep may retract. A hash decides whether anything changed. The output is a stream of changes, not a snapshot. The difference is custody and time. Nango holds a hosted, latest-only cache that expires, built for a vendor serving many customers. ownpurse keeps every version, on the owner's machine, for one owner. Nango also shows the risk to avoid: a VOIDED invoice is a fact the books need, not a deletion, and an agent with write tools needs a person in the loop. That is ownpurse's sealed package.
