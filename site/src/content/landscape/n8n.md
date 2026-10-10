---
{
 "slug": "n8n",
 "name": "n8n",
 "url": "https://n8n.io",
 "segment": "ipaas-middleware",
 "status": "active",
 "founded": "2019 (public launch October 2019; n8n GmbH registered 27 November 2019)",
 "hq": "Berlin, Germany",
 "backing": "Venture-backed. $1.5M seed March 2020 (Sequoia, firstminute); $12M Series A April 2021 (Felicis); €55M Series B March 2025 (Highland Europe); $180M Series C 9 October 2025 led by Accel at a $2.5B valuation, with NVentures (NVIDIA), Meritech, Redpoint and others, about $240M raised in total. A strategic SAP investment in May 2026 is reported [unverified].",
 "tagline": "A fair-code workflow automation platform you can run on your own machine or n8n's cloud, with AI agent nodes and an MCP server.",
 "scope": "n8n builds node-based workflows with code steps, AI agents and 1,500+ integrations, self-hosted (Docker, SQLite by default) or on n8n Cloud. It is the nearest middleware to local-first. It keeps workflow versions and pruned execution data, not a record of the books, and it does no reconciliation.",
 "domains": [
  "integration"
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
   "note": "Self-hosted, everything it holds sits in your own SQLite or Postgres; but it holds only workflows and pruned execution data, not the books."
  },
  "locality": {
   "score": 4,
   "note": "Runs on your own machine (Docker or npm, SQLite at ~/.n8n/database.sqlite) as well as on its cloud."
  },
  "history": {
   "score": 1,
   "note": "Workflow version history with rollback; execution data pruned by default after 14 days or 10,000 runs."
  },
  "openness": {
   "score": 3,
   "note": "Source-available under the Sustainable Use License (internal or personal use; not OSI open source); .ee files need an enterprise licence."
  },
  "breadth": {
   "score": 3,
   "note": "Reaches ledgers, payments, banks and documents through nodes and an HTTP node, without a unified record."
  },
  "autonomy": {
   "score": 4,
   "note": "Workflows and AI agents write to systems unattended."
  },
  "agent_readiness": {
   "score": 4,
   "note": "Instance-level MCP with OAuth per client, MCP server trigger and client nodes, a CLI over the REST API, approval for tool calls."
  },
  "integration_ease": {
   "score": 3,
   "note": "Self-serve and documented, with templates, but more technical than Zapier; self-hosting adds operations work."
  },
  "market": {
   "score": 3,
   "note": "From developers and small teams to large companies (Vodafone, Deutsche Telekom, United Nations); Business and Enterprise are sold to larger firms."
  }
 },
 "api": {
  "public": true,
  "style": "REST API generated from OpenAPI (workflows, executions and more), n8n CLI wrapping it, instance-level MCP, MCP server trigger and MCP client nodes, custom nodes in TypeScript",
  "auth": "API key; instance MCP via OAuth per client (recommended) or a personal access token",
  "limits": "Cloud: executions per plan, concurrency 5 (Starter) to 200+ (Enterprise), runs up to 5 or 40 min. Self-hosted Community: no execution limit. REST API not on the free trial.",
  "cost": "Cloud billed per workflow execution, regardless of steps; self-hosted Community edition free; AI Assistant uses credits; Business overage €4,000 per 300,000 executions.",
  "export": "Workflows as JSON; self-hosted owners hold the whole database. Execution data pruned by default (14 days, 10,000 runs) unless reconfigured.",
  "notes": "Per-execution pricing (not per step) and free self-hosting make it the cheapest of the three at volume. The security advisory list runs to 21 pages, with several High advisories dated 30 September 2026; self-hosters must patch."
 },
 "pricing": {
  "model": "Cloud subscription by executions; self-hosted free (Community) or licensed (Business, Enterprise)",
  "entry": "Community edition: free, self-hosted. Cloud Starter: €20 a month billed annually for 2,500 executions.",
  "top": "Pro: €50 a month annually for 10,000. Business (self-hosted only): €667 a month annually for 40,000, licence key that checks in daily. Enterprise: custom, cloud or self-hosted, unlimited log retention.",
  "notes": "Prices from n8n.io/pricing, read 2026-10-10; monthly prices not shown. Unlimited users and active workflows on all plans."
 },
 "agents": {
  "mcp": "official",
  "ai_features": "AI Agent nodes (LangChain-based), AI workflow builder and Assistant (credits), Evaluations, Data Tables, instance-level MCP (search, run, build and edit workflows from 2.13.0, manage agents from 2.34.0), MCP server trigger and MCP client nodes, n8n Skills.",
  "agent_writes": "Yes: an MCP client can run exposed workflows that write to the books and, from 2.13.0, create and edit workflows themselves.",
  "approval": "Human approval for an agent's tool calls (read and write tools can be set to ask first); send-and-wait approval steps in workflows; nothing exposed to MCP by default, each workflow enabled individually; OAuth approval per client; workflow version history to review and roll back AI edits. Access is not scoped per client.",
  "notes": "The closest of the three to ownpurse's stance: off by default, explicit exposure, approval on tool calls, versions kept. A High advisory on 30 September 2026 described an HMAC bypass allowing unauthenticated approval of waiting executions, a reminder that the approval gate is only as strong as its signature."
 },
 "integration": {
  "scope": "1,500+ integrations per the README (the site says 500+ and lists 2,329 including community nodes); 10,000+ templates. Xero node: contacts and invoices (create, get, get all, update); everything else through the HTTP Request node with the Xero credential.",
  "notes": "Native accounting nodes are shallow; the HTTP node and code steps do the rest, which suits agents writing their own connectors."
 },
 "preference": {
  "scale": "207k GitHub stars and 61k forks (read 2026-10-10); 200k+ community members; 6x user growth and 10x revenue growth in the year to October 2025 (n8n); revenue reported above $40M in August 2025 (press, not confirmed by n8n).",
  "ratings": "Trustpilot 2.9/5 from 49 reviews (read 2026-10-10). n8n's site claims 4.7/5 on G2; G2 blocked automated reading, so the count is not confirmed.",
  "sentiment": "Praised for power, flexibility, code steps and the cost of self-hosting. Complaints: lost account access and slow email-only support on cloud, AI-builder credits spent on broken results, and a feeling that the self-hosted edition pushes paid features. Developers argue about whether fair-code counts as open source."
 },
 "timeline": [
  {
   "date": "2019-10",
   "event": "Public launch by Jan Oberhauser in Berlin under Apache 2.0 with Commons Clause"
  },
  {
   "date": "2020-03",
   "event": "$1.5M seed (Sequoia, firstminute)"
  },
  {
   "date": "2021-04",
   "event": "$12M Series A (Felicis)"
  },
  {
   "date": "2022-03",
   "event": "Licence changed to the Sustainable Use License"
  },
  {
   "date": "2024",
   "event": "AI agent and LangChain nodes become central"
  },
  {
   "date": "2025-03",
   "event": "€55M Series B (Highland Europe)"
  },
  {
   "date": "2025-10-09",
   "event": "$180M Series C led by Accel at $2.5B"
  },
  {
   "date": "2026-05",
   "event": "Reported SAP strategic investment at about $5.2B [unverified]"
  },
  {
   "date": "2026-09-30",
   "event": "Batch of High security advisories published, including MCP and approval-step issues"
  }
 ],
 "relation": "neighbour",
 "relation_note": "n8n is the nearest middleware to ownpurse's stance: it runs on your machine and lets agents build and run workflows with approval on tool calls. But it moves data rather than keeping it, prunes its execution history after 14 days, and has no ledger or reconciliation. ownpurse could be a source or a sink for n8n workflows, and n8n users are a natural audience.",
 "borrow": "Nothing exposed to MCP by default, each workflow enabled explicitly. OAuth per MCP client with review and revoke. Approval settings on an agent's read and write tools. Version history with one-click rollback for anything an agent edits. A CLI that wraps the REST API for scripts and agents. SQLite by default for a single-user install.",
 "avoid": "A source-available licence that restricts use, which ownpurse's open-source promise should not repeat. Pruning history by default. Access that is not scoped per client. Approval tokens that can be forged: sign approvals properly (the 2026 HMAC bypass advisory).",
 "sources": [
  {
   "title": "n8n pricing",
   "url": "https://n8n.io/pricing/"
  },
  {
   "title": "n8n Series C announcement",
   "url": "https://blog.n8n.io/series-c/"
  },
  {
   "title": "Highland Europe on the Series C",
   "url": "https://www.highlandeurope.com/n8n-raises-180m-to-get-ai-closer-to-value-with-orchestration/"
  },
  {
   "title": "Sifted: n8n hits $2.5bn valuation",
   "url": "https://sifted.eu/articles/n8n-hits-2-5bn-valuation"
  },
  {
   "title": "n8n on GitHub",
   "url": "https://github.com/n8n-io/n8n"
  },
  {
   "title": "n8n licence",
   "url": "https://raw.githubusercontent.com/n8n-io/n8n/master/LICENSE.md"
  },
  {
   "title": "Accessing the n8n MCP server",
   "url": "https://docs.n8n.io/advanced-ai/accessing-n8n-mcp-server/"
  },
  {
   "title": "n8n MCP",
   "url": "https://n8n.io/mcp/"
  },
  {
   "title": "n8n public API",
   "url": "https://docs.n8n.io/api/"
  },
  {
   "title": "n8n hosting",
   "url": "https://docs.n8n.io/hosting/"
  },
  {
   "title": "Supported databases",
   "url": "https://docs.n8n.io/hosting/configuration/supported-databases-settings/"
  },
  {
   "title": "Execution data pruning",
   "url": "https://docs.n8n.io/hosting/scaling/execution-data/"
  },
  {
   "title": "Xero node",
   "url": "https://docs.n8n.io/integrations/builtin/app-nodes/n8n-nodes-base.xero/"
  },
  {
   "title": "n8n security advisories",
   "url": "https://github.com/n8n-io/n8n/security/advisories"
  },
  {
   "title": "n8n on Wikipedia",
   "url": "https://en.wikipedia.org/wiki/N8n"
  },
  {
   "title": "n8n on Trustpilot",
   "url": "https://www.trustpilot.com/review/n8n.io"
  }
 ],
 "unverified": [
  "SAP strategic investment in May 2026 at about $5.2B valuation (Wikipedia only; no primary source read)",
  "Revenue above $40M in August 2025 (press report; n8n declined to confirm)",
  "Series B size and valuation: Wikipedia says €55M at about €250M; Sifted reports $55M and a $355M valuation",
  "G2 rating (4.7/5 claimed on n8n's site; count not confirmed)",
  "Integration count: README 1,500+, About 400+, site 500+, integrations page 2,329",
  "Critical 2025-26 CVEs such as the reported unauthenticated file-access flaw (CVE-2026-21858); the NVD page could not be read",
  "Which plans include each MCP feature and tool-call approval"
 ],
 "researched": "2026-10-10"
}
---

## What it is

n8n is a workflow automation platform you can run yourself. Jan Oberhauser launched it in Berlin in 2019; it raised a [$180M Series C at a $2.5B valuation on 9 October 2025](https://blog.n8n.io/series-c/), led by Accel with NVIDIA's NVentures, about $240M in all. Its repository has [207k stars](https://github.com/n8n-io/n8n). It calls itself fair-code: the [Sustainable Use License](https://raw.githubusercontent.com/n8n-io/n8n/master/LICENSE.md) allows internal business, personal and non-commercial use, and files marked `.ee` need an enterprise licence. That is source-available, not OSI open source.

## How it works

Workflows are graphs of nodes: triggers, app nodes, code steps in JavaScript or Python, and AI agent nodes. Self-hosted, it runs in Docker and keeps everything in [SQLite at `~/.n8n/database.sqlite` by default](https://docs.n8n.io/hosting/configuration/supported-databases-settings/), or Postgres. It keeps workflow versions, but [prunes execution data after 14 days or 10,000 runs by default](https://docs.n8n.io/hosting/scaling/execution-data/). Accounting nodes are thin: the [Xero node](https://docs.n8n.io/integrations/builtin/app-nodes/n8n-nodes-base.xero/) covers contacts and invoices, and the rest goes through the HTTP Request node with the same credential.

## Data and access

The Community edition is free and unlimited on your own hardware. On [n8n Cloud](https://n8n.io/pricing/) (read 2026-10-10) Starter is €20 a month billed annually for 2,500 executions and Pro €50 for 10,000; Business is self-hosted only at €667 a month for 40,000, with a licence key that checks in daily; Enterprise is custom. An execution is one workflow run however many steps it has, which makes n8n far cheaper than per-step metering at volume. A [REST API](https://docs.n8n.io/api/) and a CLI cover workflows and executions on all self-hosted editions. A self-hoster holds the whole database.

## Agents

n8n has the most careful agent surface of the three. [Instance-level MCP](https://docs.n8n.io/advanced-ai/accessing-n8n-mcp-server/) is off until an admin turns it on, and nothing is exposed until each workflow, project or folder is enabled. Clients connect by OAuth and are approved, reviewed and revoked one by one. They can search and run workflows and, from 2.13.0, build and edit them; every change lands in version history with rollback. [Tool calls can require approval](https://n8n.io/mcp/) before an agent's read or write tools run. Gaps: access is not scoped per client, and a High advisory on 30 September 2026 described an HMAC bypass that let an unauthenticated caller approve a waiting execution ([advisories](https://github.com/n8n-io/n8n/security/advisories)).

## Customers

n8n says users grew sixfold and revenue tenfold in the year to October 2025; press put revenue above $40M in August 2025. Its homepage shows Vodafone, Deutsche Telekom and others, and it claims 200k+ community members. [Trustpilot](https://www.trustpilot.com/review/n8n.io) is polarized at 2.9/5 from 49 reviews (read 2026-10-10): praise for power and self-hosted value, complaints about cloud account access, support and AI credits spent on broken builds. Small teams self-host it to avoid Zapier bills; enterprises buy Business or Enterprise for SSO, scaling and governance.

## History

n8n began under Apache 2.0 with the Commons Clause and moved to its own licence in March 2022. From 2024 it reoriented around AI agents, and its 2025 growth came from people building agents on it. It is what the Zapier generation looks like when the connector is something you can host and an agent can write.

## For ownpurse

n8n is the closest middleware to local-first and the most likely neighbour on an ownpurse user's machine. It shows that people will run automation locally for cost and control. It also shows the gap: n8n moves data and forgets it after 14 days, it has no ledger, no balance check, and no record an owner can prove. ownpurse should be the record an n8n workflow reads from and proposes to, not another workflow engine. Borrow off-by-default exposure, per-client OAuth, approval on tool calls and rollback. Do not borrow the licence: ownpurse's openness has to be plain open source. And sign approvals well.
