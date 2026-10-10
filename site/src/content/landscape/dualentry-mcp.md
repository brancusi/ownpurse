---
{
 "slug": "dualentry-mcp",
 "name": "DualEntry MCP Server",
 "url": "https://www.dualentry.com/blog/introducing-the-dualentry-mcp-server",
 "segment": "agent-native",
 "status": "active",
 "founded": "DualEntry launched about April 2024 (18 months before its October 2025 Series A, per the announcement); MCP server live by May 2026",
 "hq": "New York, NY",
 "backing": "DualEntry raised a $90M Series A on 2 October 2025, led by Lightspeed Venture Partners and Khosla Ventures with GV, Contrary and Vesey Ventures; over $100M raised in total by then. A $415M valuation is reported in secondary coverage only.",
 "tagline": "A hosted MCP server for the DualEntry AI-native ERP, with eight read tools and five write tools built around draft, validate, approve, save.",
 "scope": "Lets an AI client (Claude, Cursor, Codex, VS Code, any MCP client) search, read and draft records inside a DualEntry ERP under the signed-in user's role. Writes go through create_draft_record, validate_record and save_record, so an assistant proposes and a person confirms. It only reaches books kept in DualEntry: it is a door into the vendor's ledger, not a layer over QuickBooks, Xero or NetSuite.",
 "domains": [
  "ledger",
  "invoicing",
  "bank",
  "close",
  "reporting",
  "integration"
 ],
 "customers_served": [
  "mid-market",
  "enterprise",
  "smb",
  "accountants"
 ],
 "spectrum": {
  "custody": {
   "score": 1,
   "note": "The books live in DualEntry's cloud; a full public API (v1, v2, v3-alpha with OpenAPI specs) lets you pull them, but no bulk export is documented."
  },
  "locality": {
   "score": 0,
   "note": "Cloud only; the CLI runs locally but is a client of the hosted API."
  },
  "history": {
   "score": 2,
   "note": "A get_record_history tool and approval workflows expose record changes, but the record is mutable current state, not an append-only log with as-of queries."
  },
  "openness": {
   "score": 2,
   "note": "Closed product with a documented public API, OpenAPI specs and llms.txt; the CLI installs from GitHub, licence not stated."
  },
  "breadth": {
   "score": 3,
   "note": "A full ERP (GL, AR, AP, bank feeds, rev rec, consolidation) behind one MCP, but only for books kept in DualEntry."
  },
  "autonomy": {
   "score": 3,
   "note": "MCP writes are drafts first, but the product also runs scheduled AI agents (hourly since August 2026) and the API posts by default."
  },
  "agent_readiness": {
   "score": 4,
   "note": "Remote MCP over OAuth 2.1, a JSON-output CLI, user-scoped API keys (September 2026), idempotency keys and agent-oriented docs."
  },
  "integration_ease": {
   "score": 3,
   "note": "MCP setup is one URL and a browser sign-in, docs are thorough, but the product itself is demo-gated with no free trial."
  },
  "market": {
   "score": 3,
   "note": "Sold to companies from about $5M revenue up to NYSE-listed businesses, with multi-entity and consolidation."
  }
 },
 "api": {
  "public": true,
  "style": "REST (Public API v1, v2, v3-alpha with OpenAPI specs) plus a remote MCP server over Streamable HTTP",
  "auth": "API: X-API-KEY header, keys issued by an Admin with their own roles, Active or Revoked; user-scoped keys since 25 Sep 2026. MCP and CLI: OAuth 2.1 browser sign-in, no API keys.",
  "limits": "Per key: 300 requests/min aggregate (burst 50) and 60 requests/min per endpoint (burst 10), overridable per organization; pricing page lists 200 requests/min on base and Plus and 'on request' on Ultra. MCP read tools return up to 1,000 results per page.",
  "cost": "Included with the DualEntry subscription; no separate API or MCP fee is listed. Plan prices are not published.",
  "export": "Everything is readable through the API and the CLI (--format json); no dedicated bulk export or backup feature is documented.",
  "notes": "API creates default to record_status 'posted': a valid create posts immediately unless the caller sends 'draft'. The drafts-first behaviour is a property of the MCP tool design, not of the API."
 },
 "pricing": {
  "model": "Annual subscription by plan and entity count, unlimited users and transactions; sold through demos",
  "entry": "DualEntry base plan, up to 3 entities; price not published",
  "top": "DualEntry Ultra, unlimited entities, SAML, advanced rev rec, planning; price not published",
  "notes": "Three plans (base, Plus up to 20 entities, Ultra unlimited). Migration and implementation included at no extra charge. No free trial. Extra users carry an unstated per-user monthly fee."
 },
 "agents": {
  "mcp": "official",
  "ai_features": "Copilot chat, AI-drafted bank match transactions and rules, AI bill capture, anomaly detection, an AI planner agent (June 2026), scheduled agents (daily to hourly), 'autopilot' close claims, plus the MCP server and CLI for outside agents.",
  "agent_writes": "Yes. Five write tools: create_draft_record, validate_record, save_record, create_draft_entity, save_entity. Validation checks required fields, entities and that entries balance.",
  "approval": "The MCP flow is draft, validate, approve, save: 'Nothing is posted until you approve.' The approval is the person confirming in the AI client before save_record is called; the docs do not say the server forces a human step, and DualEntry's own approval workflows (invoices, bills, journals, fixed assets) are separate company settings.",
  "notes": "Every tool checks the user's DualEntry role, so the agent sees only what the user can see. An August 2026 fix closed an OAuth dynamic client registration flaw that could leak tokens."
 },
 "integration": {
  "scope": "Claims 13,000+ integrations (the same figure is also used for banks); Plaid and Helios bank connections; Avalara; QuickBooks Online and Xero connections for statement reconciliation; NetSuite migration; a Bank Connections inbound feed API (GA 18 Aug 2026); Cryptoworth MCP integration (June 2026).",
  "notes": "Integrations mostly feed DualEntry or migrate books into it. The 200+ integrations 'available upon request' in the FAQ and the 13,000+ headline are inconsistent."
 },
 "preference": {
  "scale": "'Thousands of global users' and $100B of journal entries processed (company claims, October 2025); no customer count published.",
  "ratings": "DualEntry's site cites 4.9 stars on G2 from '120+ reviews' (seen October 2026; G2 page could not be fetched to confirm).",
  "sentiment": "Not verified from review sites. Vendor case studies praise fast migration and $0 implementation; the site's own figures conflict ('live in 24 hours' against 'live in 4-6 weeks')."
 },
 "timeline": [
  {
   "date": "2024 (about April)",
   "event": "DualEntry launches, per the '18 months after launch' line in the Series A announcement"
  },
  {
   "date": "2025-10-02",
   "event": "Comes out of stealth with a $90M Series A led by Lightspeed and Khosla"
  },
  {
   "date": "2026-05-21",
   "event": "MCP improvements: per-user OAuth credentials for MCP servers"
  },
  {
   "date": "2026-05-23",
   "event": "AI agent framework for close and master data"
  },
  {
   "date": "2026-06-19",
   "event": "Cryptoworth MCP integration"
  },
  {
   "date": "2026-08-12",
   "event": "Idempotency-Key header on the public API"
  },
  {
   "date": "2026-08-24",
   "event": "Hourly scheduled AI agents"
  },
  {
   "date": "2026-08-27",
   "event": "MCP OAuth hardening; MCP blog post last updated"
  },
  {
   "date": "2026-09-25",
   "event": "User-scoped API keys"
  }
 ],
 "relation": "neighbour",
 "relation_note": "The closest vendor statement of 'agents propose, people post', but inside a ledger DualEntry asks you to migrate to. ownpurse leaves the books where they are and keeps the owner's own record; DualEntry is also a possible future source.",
 "borrow": "The tool split (search, detail, history, schema, related) as a model for ownpurse's MCP surface; validate as its own step that checks balance before anything is sealed; idempotency keys with replay headers; user-scoped keys; llms.txt-first docs.",
 "avoid": "An approval that lives only in the chat client's confirmation, with the API defaulting to posted; it is easy for another caller to skip. ownpurse's gate should be a sealed package recorded in the log, not a convention.",
 "sources": [
  {
   "title": "Introducing the DualEntry MCP Server",
   "url": "https://www.dualentry.com/blog/introducing-the-dualentry-mcp-server"
  },
  {
   "title": "DualEntry pricing",
   "url": "https://www.dualentry.com/pricing"
  },
  {
   "title": "DualEntry docs index (llms.txt)",
   "url": "https://docs.dualentry.com/llms.txt"
  },
  {
   "title": "DualEntry MCP guide",
   "url": "https://docs.dualentry.com/developers/guides/integrations-and-tools/mcp.md"
  },
  {
   "title": "API authentication",
   "url": "https://docs.dualentry.com/developers/guides/api/authentication.md"
  },
  {
   "title": "Rate limiting",
   "url": "https://docs.dualentry.com/developers/guides/api/core-concepts/rate-limiting.md"
  },
  {
   "title": "Idempotency and write validation",
   "url": "https://docs.dualentry.com/developers/guides/api/core-concepts/idempotency-and-write-validation.md"
  },
  {
   "title": "CLI quickstart",
   "url": "https://docs.dualentry.com/developers/guides/cli/quickstart.md"
  },
  {
   "title": "Release notes index",
   "url": "https://docs.dualentry.com/_llms/release-notes.md"
  },
  {
   "title": "Release notes 2026-08-27",
   "url": "https://docs.dualentry.com/developers/release-notes/2026-08-27.md"
  },
  {
   "title": "CPA Practice Advisor: DualEntry $90M Series A",
   "url": "https://www.cpapracticeadvisor.com/2025/10/02/dualentry-touts-90-million-series-a-to-modernize-legacy-erp-software-with-ai/170156/"
  },
  {
   "title": "DualEntry home page",
   "url": "https://www.dualentry.com/"
  }
 ],
 "unverified": [
  "$415M post-money valuation (secondary coverage only)",
  "G2 rating of 4.9 from 120+ reviews (cited on DualEntry's site; G2 returned 403)",
  "Exact launch or founding date of DualEntry (inferred from '18 months after launch')",
  "First release date of the MCP server (it existed by 21 May 2026)",
  "Whether the server itself enforces a human approval before save_record",
  "Plan prices and the per-user fee",
  "CLI licence and whether the CLI can write",
  "Review-site sentiment; the web search budget ran out before Reddit and Capterra could be checked"
 ],
 "researched": "2026-10-10"
}
---

## What it is

The [DualEntry MCP Server](https://www.dualentry.com/blog/introducing-the-dualentry-mcp-server) connects AI clients to [DualEntry](https://www.dualentry.com/), a New York AI-native ERP that came out of stealth on 2 October 2025 with a [$90M Series A](https://www.cpapracticeadvisor.com/2025/10/02/dualentry-touts-90-million-series-a-to-modernize-legacy-erp-software-with-ai/170156/) led by Lightspeed and Khosla. The server exposes thirteen tools: eight to ask (search_records, get_record_detail, get_record_by_number, get_record_history, get_related_records, get_record_schema, list_supported_transactions, search_entities) and five to act (create_draft_record, validate_record, save_record, create_draft_entity, save_entity).

## How it works

You add `https://api.dualentry.com/mcp/` to Claude, Cursor, Codex or VS Code and sign in through OAuth 2.1; there are no API keys to mint. Every tool runs under your DualEntry role, so the agent sees what you see in the dashboard and nothing more. A write goes draft, validate, approve, save: the assistant drafts a record, validate_record checks required fields, entities and that the entry balances, and "nothing is posted until you approve." That approval is you confirming in the client before save_record runs. The [docs](https://docs.dualentry.com/developers/guides/integrations-and-tools/mcp.md) do not say the server enforces it.

## Data and access

Beside the MCP there is a documented [public API](https://docs.dualentry.com/llms.txt) (v1, v2, v3-alpha, with OpenAPI specs) and a Python [CLI](https://docs.dualentry.com/developers/guides/cli/quickstart.md) installed with Homebrew or uv that prints `--format json`. API keys are issued by an Admin with their own roles and reach every company in the organization; user-scoped keys arrived on 25 September 2026. The [rate limits](https://docs.dualentry.com/developers/guides/api/core-concepts/rate-limiting.md) are 300 requests a minute per key overall and 60 per endpoint, with overrides. Note one contrast: through the API a create [defaults to posted](https://docs.dualentry.com/developers/guides/api/core-concepts/idempotency-and-write-validation.md) if it validates, so drafts-first is a property of the MCP tools, not the platform. Idempotency keys (August 2026) make retries safe. No bulk export or backup feature is documented; the API is the way out.

## Agents

DualEntry is agent-heavy on its own side too: an AI planner agent (June 2026), scheduled agents that run daily down to hourly (August 2026), AI-drafted bank matches and rules, and close "on autopilot." The MCP is how outside agents get the same reach. What it does well: a small, well-named tool set, a separate validate step that checks balance, role-scoped access and OAuth for people with keys for machines. The August 2026 release notes record a fix for an OAuth dynamic client registration flaw that could have exposed tokens, which shows the risk of a hosted write path.

## Customers

DualEntry sells to companies from about $5M in revenue to NYSE-listed businesses, with multi-entity, multi-book and consolidation. It claims "thousands of global users" and $100B of journal entries processed (October 2025), and its site cites 4.9 stars on G2 from 120+ reviews; we could not confirm that on G2. Pricing is three plans by entity count (up to 3, up to 20, unlimited), unlimited users, no published prices, no free trial, migration included.

## History

About half a billion dollars went into AI ledgers in roughly a year (see the [landscape survey](https://ownpurse.com/blog/a-run-on-your-data/)); DualEntry is one of them. Its pitch is migration speed ("live in 24 hours", elsewhere "4-6 weeks"). The MCP server existed by May 2026, when per-user OAuth for MCP shipped, and the blog post was last updated 27 August 2026.

## For ownpurse

DualEntry is the clearest vendor version of the rule ownpurse is built on: agents propose, people post. The differences are where the books live and where the gate sits. DualEntry asks you to move your ledger into its cloud, and the gate is a confirmation in the chat client while the API posts by default. ownpurse leaves the books in Xero or QuickBooks, keeps the owner's own append-only record, and makes the gate a sealed package that is recorded in the log. Worth borrowing: the tool vocabulary, validate as its own step, idempotent writes and user-scoped keys. DualEntry is also a plausible future source for an ownpurse connector, since its API is public and documented.
