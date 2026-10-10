---
{
 "slug": "apideck",
 "name": "Apideck",
 "url": "https://www.apideck.com",
 "segment": "unified-api",
 "status": "active",
 "founded": "2019",
 "hq": "Antwerp, Belgium and San Francisco, US (both listed as offices; neither named as headquarters)",
 "backing": "Venture-backed. $7.5M Series A led by Airbridge Equity Partners with PMV and angels, announced 20 November 2024. Earlier rounds not confirmed.",
 "tagline": "A pass-through unified API that reads and writes accounting, CRM, HR and other SaaS data on demand and says it stores none of it.",
 "scope": "Apideck gives software companies one API over 200+ connectors in eight categories (accounting, CRM, HRIS, ATS, file storage, e-commerce, issue tracking, ERP), with Vault for end-user OAuth, a proxy for native endpoints, virtual webhooks, a hosted MCP server and a CLI. Payloads are processed in memory and not stored, so it keeps no cache and no history. It does not hold a ledger, reconcile, or keep anything for the business whose books it reads.",
 "domains": [
  "integration",
  "ledger",
  "bank",
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
   "note": "Stores nothing; the record stays with the source provider, and any copy belongs to the integrating app, not the owner."
  },
  "locality": {
   "score": 0,
   "note": "Cloud only, EU-hosted by default; the CLI runs locally but calls the hosted API."
  },
  "history": {
   "score": 0,
   "note": "Live current state from the source on every call; no versions, no as-of queries."
  },
  "openness": {
   "score": 2,
   "note": "Closed service with a public OpenAPI-described API, open SDKs and an MIT-licensed CLI."
  },
  "breadth": {
   "score": 3,
   "note": "Accounting (incl. bank feeds, reports, attachments) plus CRM, HR, files and commerce; no payments or bank aggregation of its own."
  },
  "autonomy": {
   "score": 2,
   "note": "Writes only when the integrating app calls it; no automation of its own."
  },
  "agent_readiness": {
   "score": 4,
   "note": "Hosted MCP with 229 tools, an agent CLI with read/write/dangerous gating, llms.txt and a Claude Code skill."
  },
  "integration_ease": {
   "score": 3,
   "note": "Self-serve docs and SDKs, but plans start at EUR 599 a month and you bring your own provider apps."
  },
  "market": {
   "score": 3,
   "note": "Sells to B2B software vendors from startups to enterprise (SSO, SLA, NetSuite and Workday sandboxes)."
  }
 },
 "api": {
  "public": true,
  "style": "REST/JSON over https://unify.apideck.com, one unified model per category, plus a Proxy API to call connector-native endpoints and pass_through (extend_object, extend_paths) to add provider fields.",
  "auth": "Apideck API key plus app id and consumer id; end-user provider credentials held in Vault (managed OAuth).",
  "limits": "No Apideck-wide call limit on the pricing page; downstream limits are normalised into x-downstream-ratelimit-* headers, and the TypeScript SDK retries on Retry-After.",
  "cost": "Priced per consumer (connected end customer), not per call: Launch EUR 599/month for 25 consumers and 1 API category; Scale from EUR 1,299 for 100 consumers and 3 categories; Enterprise custom. The provider's own fees (e.g. Xero's tiers and egress since 2 March 2026) fall on the app owner, since you connect your own Xero app.",
  "export": "Nothing to export from Apideck: it stores no data. You export from the source through it.",
  "notes": "Accounting resources include journal entries, ledger accounts, bank feed accounts and statements, invoices, bills, payments, attachments and read-only balance sheet, P&L and aged reports; most support full CRUD."
 },
 "pricing": {
  "model": "Subscription by number of consumers and API categories, monthly or yearly (10% off)",
  "entry": "Launch: EUR 599/month, 25 consumers, 1 unified API category",
  "top": "Scale up to EUR 4,250/month for 500 consumers; Enterprise custom annual contract",
  "notes": "Prices from the pricing page as read 2026-10-10 (no effective date shown). Custom field mapping and data scopes need Scale; SSO, white-label Vault and SLA need Enterprise. New connectors are a paid add-on."
 },
 "agents": {
  "mcp": "official",
  "ai_features": "Hosted MCP server at mcp.apideck.com generated from the OpenAPI spec (229 tools across CRM, accounting, HRIS, ATS and file storage, discovered progressively); a docs MCP; an agent CLI with an 80-token agent prompt and a Claude Code skill; llms.txt docs.",
  "agent_writes": "Yes. MCP tools and CLI subcommands map to the full API, including create, update and delete.",
  "approval": "The CLI classifies GET as read (auto-approved), POST/PUT/PATCH as write (confirmation prompt) and DELETE as dangerous (blocked unless --force), configurable in ~/.apideck-cli/permissions.yaml; non-interactive runs refuse writes with a JSON error. The MCP docs describe no approval or read-only mode.",
  "notes": "The CLI's permission model is the closest thing in this segment to ownpurse's read-freely, write-with-approval rule, but it is a per-call prompt, not a sealed package with a record."
 },
 "integration": {
  "scope": "200+ connectors in 8 unified APIs; 45-50+ accounting and ERP connectors (Xero, QuickBooks Online and Desktop, NetSuite, Sage Intacct, Intuit Enterprise Suite, SAP S/4HANA among them) across 45+ countries.",
  "notes": "Native and virtual webhooks; virtual webhooks poll for you and track change 'without persisting your data'. Delete detection is not documented. Joined FDX in August 2026, working on the corporate side of the US bank data standard."
 },
 "preference": {
  "scale": "No customer count published. Named customers include BILL, Drata, 11x and Principal Group (Series A post, November 2024).",
  "ratings": "Not confirmed: G2 and Capterra pages refused access and no Trustpilot page exists.",
  "sentiment": "Not confirmed from reviews in this pass. The vendor's own pitch is developer experience and the no-storage stance."
 },
 "timeline": [
  {
   "date": "2019",
   "event": "Founded in Antwerp"
  },
  {
   "date": "2024-11-20",
   "event": "$7.5M Series A led by Airbridge Equity Partners, with PMV"
  },
  {
   "date": "2026-01-19",
   "event": "Intuit Enterprise Suite connector"
  },
  {
   "date": "2026-04",
   "event": "Posts on its MCP server with 229 tools generated from the OpenAPI spec"
  },
  {
   "date": "2026-06-30",
   "event": "QuickBooks Desktop, SAP S/4HANA and 25 more connectors announced"
  },
  {
   "date": "2026-08-17",
   "event": "Joins the Financial Data Exchange (FDX)"
  },
  {
   "date": "2026-09-24",
   "event": "Partnership with TreasurUp on ERP-embedded business banking"
  }
 ],
 "relation": "neighbour",
 "relation_note": "The far pole from ownpurse: it keeps nothing, where ownpurse keeps everything. It serves software vendors who build integrations for many customers; ownpurse serves the owner of one set of books. Not a source worth using: a paid third party between you and Xero that adds no history.",
 "borrow": "The CLI's permission classes (read auto, write prompts, delete blocked, non-TTY refuses writes with structured JSON) and its token-thin agent prompt; normalised downstream rate-limit headers; the Proxy and pass_through escape hatches to the source's native shape.",
 "avoid": "Treating 'we store nothing' as a virtue for the owner: it means no history, no as-of view and no evidence when a source edits or deletes a record. Per-consumer pricing in the hundreds of euros a month is built for vendors, not one person.",
 "sources": [
  {
   "title": "Apideck pricing",
   "url": "https://www.apideck.com/pricing"
  },
  {
   "title": "Apideck about",
   "url": "https://www.apideck.com/about"
  },
  {
   "title": "Apideck home",
   "url": "https://www.apideck.com"
  },
  {
   "title": "Announcing our $7.5m Series A",
   "url": "https://www.apideck.com/blog/series-a-funding"
  },
  {
   "title": "Apideck joins FDX",
   "url": "https://www.apideck.com/blog/apideck-joins-fdx-financial-data-exchange"
  },
  {
   "title": "Xero API pricing and the App Partner Program (Apideck)",
   "url": "https://www.apideck.com/blog/xero-api-pricing-and-the-app-partner-program"
  },
  {
   "title": "What is a unified MCP server?",
   "url": "https://www.apideck.com/blog/what-is-a-unified-mcp-server"
  },
  {
   "title": "Apideck MCP server docs",
   "url": "https://developers.apideck.com/mcp.md"
  },
  {
   "title": "Apideck CLI docs",
   "url": "https://developers.apideck.com/cli.md"
  },
  {
   "title": "apideck-libraries/cli on GitHub",
   "url": "https://github.com/apideck-libraries/cli"
  },
  {
   "title": "Accounting API reference",
   "url": "https://developers.apideck.com/apis/accounting/reference.md"
  },
  {
   "title": "Unified rate limits",
   "url": "https://developers.apideck.com/guides/unified-rate-limits.md"
  },
  {
   "title": "Webhooks guide",
   "url": "https://developers.apideck.com/guides/webhooks.md"
  },
  {
   "title": "Pass-through guide",
   "url": "https://developers.apideck.com/guides/pass-through.md"
  },
  {
   "title": "Apideck blog index",
   "url": "https://www.apideck.com/blog"
  }
 ],
 "unverified": [
  "Founders' names (the Series A post is signed 'GJ'; commonly reported as Gertjan De Wilde, not confirmed here)",
  "Any seed or pre-Series A rounds",
  "G2, Capterra and Trustpilot ratings (G2 and Capterra refused access; no Trustpilot page found)",
  "Customer count and revenue",
  "Whether the MCP server has a read-only mode or any write approval",
  "How virtual webhooks detect deletes and their polling interval",
  "Web search budget was exhausted during this pass; press coverage beyond Apideck's own pages was not checked"
 ],
 "researched": "2026-10-10"
}
---

## What it is

Apideck is a unified API: one interface that software companies use to read and write their customers' data in other SaaS systems. It covers [eight categories](https://www.apideck.com) (accounting, CRM, HRIS, ATS, file storage, e-commerce, issue tracking and ERP) over 200+ connectors, and was [founded in 2019 in Antwerp](https://www.apideck.com/about), with an office in San Francisco. Its defining choice is that it stores nothing: integration payloads are, in its words, "processed in memory and never stored".

## How it works

An app sends a request to `unify.apideck.com` with its API key, app id and the consumer id of the end customer. Apideck looks up that customer's provider credentials in Vault, calls the provider live and maps the answer into its common model. The [accounting API](https://developers.apideck.com/apis/accounting/reference.md) covers journal entries, ledger accounts, bank feed accounts and statements, invoices, bills, payments, attachments and read-only balance sheet, P&L and aged reports. When the model is not enough, [pass-through](https://developers.apideck.com/guides/pass-through.md) adds provider fields and the Proxy API calls native endpoints. [Virtual webhooks](https://developers.apideck.com/guides/webhooks.md) poll providers that lack native webhooks, "without persisting your data"; how deletes are detected is not documented.

## Data and access

Pricing is [per consumer](https://www.apideck.com/pricing): EUR 599 a month for 25 consumers on one category, up to EUR 4,250 for 500 on Scale, then Enterprise. No call quota is published; provider limits come back as normalised [rate-limit headers](https://developers.apideck.com/guides/unified-rate-limits.md). You still pay the provider: for Xero, Apideck says you [connect your own Xero app](https://www.apideck.com/blog/xero-api-pricing-and-the-app-partner-program), so Xero's tiers and egress fees (from 2 March 2026) are yours. Because nothing is stored, there is nothing to export from Apideck and no history to query.

## Agents

Apideck has leaned into agents in 2026. A hosted [MCP server](https://developers.apideck.com/mcp.md) exposes 229 tools generated from its OpenAPI spec, discovered progressively to save context, with OAuth through Vault. The [CLI](https://github.com/apideck-libraries/cli) (Go, MIT) is the more interesting piece: reads are auto-approved, writes ask for confirmation, deletes are blocked without `--force`, the classes are configurable per operation, and in a non-interactive run writes fail with a structured JSON error. It ships an 80-token agent prompt and a Claude Code skill. The MCP docs describe no matching approval layer.

## Customers

Apideck sells to B2B software vendors that need many integrations at once; its [Series A post](https://www.apideck.com/blog/series-a-funding) names BILL, Drata, 11x and Principal Group. It publishes no customer count, and review-site ratings could not be confirmed in this pass. The Enterprise tier (SSO, white-label Vault, NetSuite and Workday sandboxes, SLA) points at larger buyers; the entry price rules out a single small business using it directly.

## History

After founding in 2019, Apideck raised a [$7.5M Series A](https://www.apideck.com/blog/series-a-funding) led by Airbridge Equity Partners in November 2024. In 2026 it added Intuit Enterprise Suite, QuickBooks Desktop and SAP S/4HANA connectors, shipped its MCP server and CLI, [joined FDX](https://www.apideck.com/blog/apideck-joins-fdx-financial-data-exchange) in August to work on the corporate side of US bank data sharing, and partnered with TreasurUp on ERP-embedded banking in September.

## For ownpurse

Apideck is the other pole. It keeps nothing so its customers carry no data risk; ownpurse keeps everything so the owner has evidence. A pass-through answers "what does Xero say now" and cannot answer "what did it say last March", which is the question an audit or a dispute asks. It is not worth using as a source: it sits between you and Xero, costs hundreds a month, and still leaves Xero's fees with you. What is worth borrowing is its CLI's permission model, which is the nearest thing in this segment to ownpurse's rule that agents read freely and writes need a person. ownpurse goes further by sealing the write as a package and recording it in the log.
