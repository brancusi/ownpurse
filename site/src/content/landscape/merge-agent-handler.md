---
{
 "slug": "merge-agent-handler",
 "name": "Merge Agent Handler",
 "url": "https://docs.merge.dev/merge-agent-handler/how-it-works",
 "segment": "agent-native",
 "status": "active",
 "founded": "Merge founded 2020; Agent Handler launched October 2025 (date per the brief, not confirmed from a primary source)",
 "hq": "San Francisco, CA (offices in San Francisco, New York and Berlin)",
 "backing": "Merge (Merge API, Inc.), founded by Shensi Ding and Gil Feig, backed by Accel, Addition and NEA; a $4.5M seed reported by TechCrunch on 17 May 2021 and a $55M Series B (date not confirmed).",
 "tagline": "A hosted MCP gateway that gives AI agents per-user authenticated tools across about 330 third-party systems, with data-loss rules and call logs.",
 "scope": "Agents connect to one MCP endpoint, a bearer token resolves to a Tool Pack and a Registered User, and each tool call goes out with that user's stored credentials after the Security Gateway scans it. Accounting is one category among many: QuickBooks Online, Xero, NetSuite, FreshBooks, Business Central, Stripe, Ramp, Brex, Mercury and others. It keeps no copy of the data and no history of it; it is plumbing for live calls, not a record.",
 "domains": [
  "integration",
  "ledger",
  "payments",
  "spend",
  "bank"
 ],
 "customers_served": [
  "developers",
  "enterprise",
  "mid-market"
 ],
 "spectrum": {
  "custody": {
   "score": 1,
   "note": "Data stays in each source; Merge stores credentials and logs, never an owner-held record."
  },
  "locality": {
   "score": 1,
   "note": "Cloud service; on-prem and private-cloud deployments only as custom Enterprise contracts."
  },
  "history": {
   "score": 1,
   "note": "Tool call logs and an admin audit trail kept 90 days by default; no history of the data itself."
  },
  "openness": {
   "score": 1,
   "note": "Closed hosted service with thorough public docs and a pip-installed CLI; no source, no self-serve self-hosting."
  },
  "breadth": {
   "score": 3,
   "note": "About 330 connectors spanning ledgers, payments, banks, spend and HR, but each is a set of live tools, not one combined view."
  },
  "autonomy": {
   "score": 3,
   "note": "Agents write directly (the QuickBooks pack has 25 write tools including deletes) with rule-based blocking but no human approval step."
  },
  "agent_readiness": {
   "score": 4,
   "note": "Built for agents: MCP endpoint, per-user scoped keys, Tool Packs, description overrides, a progressive-discovery CLI and llms.txt docs."
  },
  "integration_ease": {
   "score": 4,
   "note": "Free tier with 2,000 calls a month, self-serve keys, embedded auth UI and a quickstart."
  },
  "market": {
   "score": 3,
   "note": "Sold to software companies building agents (logos include OpenAI, Perplexity, Ramp) and to enterprises governing employee AI."
  }
 },
 "api": {
  "public": true,
  "style": "MCP (tools/list, tools/call) plus a REST management API for Registered Users, Tool Packs, logs and audit events; CLI for coding agents",
  "auth": "Access keys (production and sandboxed test) for your backend; production keys mint user-scoped keys limited to given Registered Users and Tool Packs. End users authenticate to each connector through Link (React component, hosted URL or Magic Link); Merge's OAuth apps by default, your own optionally.",
  "limits": "Free tier stops at 2,000 credits a month with 402 billing_limit_reached; one tool call is one credit, failed and test calls included. Upstream rate limits apply per OAuth app.",
  "cost": "Free $0 (2,000 credits/month); Pro $1,000/month for 25,000 credits plus 2,000 bonus, overage billed at an unpublished rate; Enterprise custom; Employees Pro per provisioned user (price not published).",
  "export": "Audit trail exports as CSV and through the audit-log API; tool call logs viewable and streamable to a SIEM on Enterprise. No export of third-party data, since none is kept.",
  "notes": "Distinct from Merge's Unified API (normalized common models, $650/month Launch tier for 10 linked accounts, first 3 free, $65 each beyond), which syncs and caches data; Agent Handler makes live calls."
 },
 "pricing": {
  "model": "Usage-based credits, one per tool call, with unlimited registered users and Tool Packs",
  "entry": "Free: $0/month, 2,000 credits",
  "top": "Enterprise: custom, with custom connectors, on-prem deployment, SIEM streaming, forward-deployed engineers",
  "notes": "Pro starts at $1,000/month for 25,000 credits, with higher allowances up to 100,000 credits at unpublished prices. Employees Pro is priced per IdP-provisioned user."
 },
 "agents": {
  "mcp": "official",
  "ai_features": "The product is agent infrastructure: AI Guardrails (plain-English policies that block or log, including on returned data), PII, PHI and payment-data detectors, custom regex rules, a rule tester, alerts and logs.",
  "agent_writes": "Yes. Connectors expose write tools directly; the QuickBooks Online connector has 42 tools, 17 read and 25 write (create, update, delete, void and send for bills, invoices, payments, customers, vendors, items, estimates, accounts).",
  "approval": "No human approval or confirmation step is documented. Control is by choosing which tools a Tool Pack exposes (list only read tools for a read-only pack) and by Security Gateway rules that allow, redact or block arguments.",
  "notes": "Omitting tool_names in a Tool Pack enables every tool the connector has, writes included, so read-only must be set deliberately."
 },
 "integration": {
  "scope": "About 329 connector entries (October 2026), including QuickBooks Online, Xero, NetSuite, FreshBooks, Dynamics 365 Business Central, Concur, Expensify, Stripe, PayPal, Square, GoCardless, Brex, Ramp, Mercury, Airwallex, Avalara, Carta and Deel. Plaid and Sage are not listed.",
  "notes": "Agent Handler for Employees provisions access through SCIM identity providers (Okta, Azure AD) and installs as a remote MCP server in Claude, Claude Code, Cursor or ChatGPT."
 },
 "preference": {
  "scale": "Merge says 'thousands of companies' use it (October 2026); Agent Handler logos include OpenAI, Culture Amp, JetBrains, Ramp, Perplexity and Factory. No Agent Handler usage figures published.",
  "ratings": "Not verified; G2 and Capterra pages for Merge could not be checked.",
  "sentiment": "Not verified from review sites. Perplexity, quoted by Merge, says it lets them bring 'a broader set of data connectors to our users faster'. The earlier landscape survey notes Merge sells full delete detection as a paid add-on on its Unified API."
 },
 "timeline": [
  {
   "date": "2020",
   "event": "Merge founded by Shensi Ding and Gil Feig"
  },
  {
   "date": "2021-05-17",
   "event": "TechCrunch reports Merge's seed round ($4.5M per the article address)"
  },
  {
   "date": "2025-10",
   "event": "Agent Handler launches (per the brief; no primary announcement found)"
  },
  {
   "date": "2026-07",
   "event": "Merge blog publishes MCP how-to guides for Codex, Cursor and Claude Code"
  },
  {
   "date": "2026-10",
   "event": "About 330 connectors listed; Free, Pro ($1,000/month) and Enterprise tiers"
  }
 ],
 "relation": "infrastructure",
 "relation_note": "A unified-API vendor turning into an agent tool gateway. It could carry an agent's calls to Xero or QuickBooks, but it keeps no record and gates writes only with rules, so it is neither a competitor nor a likely source for ownpurse; it is the hosted, enterprise version of 'let agents call tools safely'.",
 "borrow": "Tool Packs as named, scoped bundles (a read pack, a write pack); per-user scoped keys; tool description overrides; scanning arguments before they leave; a progressive-discovery CLI for coding agents; every tool call logged with latency.",
 "avoid": "Write tools on by default when a pack does not list them; safety as pattern rules rather than a person's approval; 90-day logs as the only memory of what agents did; per-call metering that makes reading cost money.",
 "sources": [
  {
   "title": "Agent Handler: how it works",
   "url": "https://docs.merge.dev/merge-agent-handler/how-it-works"
  },
  {
   "title": "Agent Handler product page",
   "url": "https://www.merge.dev/agent-handler"
  },
  {
   "title": "Agent Handler pricing",
   "url": "https://www.merge.dev/pricing/agent-handler"
  },
  {
   "title": "Billing and usage",
   "url": "https://docs.merge.dev/merge-agent-handler/administer/billing-and-usage.md"
  },
  {
   "title": "Connectors list",
   "url": "https://docs.merge.dev/merge-agent-handler/connectors.md"
  },
  {
   "title": "QuickBooks Online connector tools",
   "url": "https://docs.merge.dev/merge-agent-handler/connectors/quick-books-online"
  },
  {
   "title": "Tool Packs",
   "url": "https://docs.merge.dev/merge-agent-handler/build/tools/tool-packs.md"
  },
  {
   "title": "Audit trail",
   "url": "https://docs.merge.dev/merge-agent-handler/observe/audit-trail.md"
  },
  {
   "title": "Merge CLI",
   "url": "https://docs.merge.dev/merge-agent-handler/build/cli/merge-cli.md"
  },
  {
   "title": "Merge pricing (Unified API)",
   "url": "https://www.merge.dev/pricing"
  },
  {
   "title": "Merge about page",
   "url": "https://www.merge.dev/about"
  },
  {
   "title": "ownpurse landscape survey, October 2026",
   "url": "https://ownpurse.com/blog/a-run-on-your-data/"
  }
 ],
 "unverified": [
  "October 2025 launch date for Agent Handler (from the brief; no primary announcement found)",
  "Date of Merge's $55M Series B and any later rounds or valuation",
  "Seed amount ($4.5M appears only in the TechCrunch article address)",
  "Pro overage rate per credit and prices for higher Pro allowances",
  "Employees Pro per-user price",
  "Whether on-prem deployment is available below Enterprise",
  "CLI licence",
  "G2, Capterra and Reddit sentiment; the web search budget ran out before they could be checked"
 ],
 "researched": "2026-10-10"
}
---

## What it is

[Merge Agent Handler](https://www.merge.dev/agent-handler) is a hosted MCP server that "fans out to hundreds of third-party Connectors," attaches the right user's credentials to each call, scans it against your security rules and records the result. Merge, founded in 2020 by Shensi Ding and Gil Feig ([about](https://www.merge.dev/about)), built its business on a Unified API that normalizes HR, accounting and other categories into common models. Agent Handler, launched in October 2025, is the same company selling live tools to agents instead of synced data to apps.

## How it works

The [request path](https://docs.merge.dev/merge-agent-handler/how-it-works) is short. An agent connects with a bearer token that resolves to an organization, a Tool Pack and a Registered User. The first time that user touches a connector, they authenticate through Link, and Merge stores the credential against them. The Security Gateway checks the tool's arguments, redacting or blocking matches against PII, PHI and payment-data detectors, custom regex rules or plain-English AI guardrails. Then the connector calls the third party with the stored credentials, refreshing OAuth tokens as needed. A [Tool Pack](https://docs.merge.dev/merge-agent-handler/build/tools/tool-packs.md) is a named bundle of connectors and tools for one agent surface, such as a read-only support pack and a write-enabled revenue-ops pack.

## Data and access

About 330 [connectors](https://docs.merge.dev/merge-agent-handler/connectors.md) are listed, including QuickBooks Online, Xero, NetSuite, FreshBooks, Business Central, Stripe, Ramp, Brex and Mercury; Plaid and Sage are not. The QuickBooks Online connector has 42 tools, 17 that read and 25 that write, delete, void or send. Merge keeps no copy of the data. What it keeps is logs: tool calls, API requests and an admin [audit trail](https://docs.merge.dev/merge-agent-handler/observe/audit-trail.md) with before-and-after diffs, retained 90 days by default and exportable as CSV. [Pricing](https://www.merge.dev/pricing/agent-handler) is per tool call: Free gives 2,000 credits a month and then returns 402; Pro is $1,000 a month for 25,000; Enterprise is custom and can include on-prem deployment.

## Agents

This is the most agent-ready product of the three here. It has user-scoped keys, per-pack tool description and input overrides, a [CLI](https://docs.merge.dev/merge-agent-handler/build/cli/merge-cli.md) for coding agents that discovers tools progressively, SCIM provisioning for employee AI and docs written for machines. What it does not have is a human approval step: the docs describe blocking, redacting and logging, not asking a person. A pack that does not list its tools gets all of them, writes included.

## Customers

Merge shows logos from OpenAI, Culture Amp, JetBrains, Ramp, Perplexity and Factory for Agent Handler and says "thousands of companies" use Merge. Buyers are software teams building agents for their own users, and IT teams governing what employees' AI clients can reach. It is not sold to small businesses or their owners directly.

## History

Unified APIs (Codat, Merge, Rutter, Apideck) were the last decade's answer to "connect to every ledger." They hold current state or nothing, and Merge sells full delete detection on its Unified API as a paid extra (see the [landscape survey](https://ownpurse.com/blog/a-run-on-your-data/)). Agent Handler drops even the cache: an agent asks, Merge calls the source, the answer goes back. The middleware layer is moving from syncing data to brokering actions.

## For ownpurse

Merge solves a real problem well: per-user credentials, scoping and logs for agents across hundreds of tools. ownpurse should borrow Tool Packs as an idea (named read and write surfaces), scoped keys and logging every call. But Merge's model is the opposite of ownpurse's on the axes that matter here. Every read is a metered live call to a source; nothing is kept, so nothing can be checked or queried as of last month; writes go straight to the ledger, filtered by rules but never approved by a person. For an owner, a gateway like this adds a third party between them and their books. ownpurse keeps the record locally, makes reads free and makes writes a sealed package a person approves.
