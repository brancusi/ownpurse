---
{
 "slug": "dualentry",
 "name": "DualEntry",
 "url": "https://www.dualentry.com",
 "segment": "ai-ledger",
 "status": "active",
 "founded": "Not confirmed; came out of stealth in October 2025 (likely founded 2024)",
 "hq": "New York, NY, US (7 World Trade Center)",
 "backing": "$90M Series A on 2 October 2025, co-led by Lightspeed Venture Partners and Khosla Ventures with GV, Contrary and Vesey Ventures, at a $415M post-money valuation; company says over $100M raised in total; private (Entry Inc.)",
 "tagline": "DualEntry is a cloud ERP for mid-market and multi-entity finance teams that promises to move your books in from NetSuite, Sage Intacct, QuickBooks or Xero in a day.",
 "scope": "DualEntry is a full financials ERP: GL, AP, AR, cash, tax, purchase orders, reconciliation, close, revenue recognition, subscription billing, fixed assets, multi-entity consolidation, planning and analytics, with AI categorisation, bank matching and flux analysis. Its pitch is migration: 'NextDay Migration' ingests your history from the old system in about 24 hours. It replaces your ledger rather than sitting beside one, and it has no inventory or manufacturing depth beyond order management.",
 "domains": [
  "ledger",
  "bank",
  "invoicing",
  "tax",
  "close",
  "reporting",
  "integration"
 ],
 "customers_served": [
  "smb",
  "mid-market",
  "accountants"
 ],
 "spectrum": {
  "custody": {
   "score": 1,
   "note": "DualEntry's cloud holds the authoritative books; the API and CLI can export records as JSON, but no full-ledger export is documented."
  },
  "locality": {
   "score": 0,
   "note": "Cloud only."
  },
  "history": {
   "score": 2,
   "note": "Record history is exposed (an MCP get_record_history tool) and posting is distinct from drafts; no documented append-only guarantee or as-of queries."
  },
  "openness": {
   "score": 2,
   "note": "Closed source; versioned public REST API (v1, v2, v3-alpha; about 769 reference pages) with OpenAPI specs and self-serve keys."
  },
  "breadth": {
   "score": 3,
   "note": "Broad financials suite plus 13,000+ bank connections, Stripe, HubSpot, Salesforce and Expensify; no documents, payroll or personal finance."
  },
  "autonomy": {
   "score": 2,
   "note": "AI categorises, scores bank matches and flags anomalies for accountants; no documented always-on agent that posts unattended."
  },
  "agent_readiness": {
   "score": 4,
   "note": "Official MCP server with OAuth 2.1 and 15 tools including writes, a Homebrew CLI aimed at agents and CI, and free Claude finance skills."
  },
  "integration_ease": {
   "score": 3,
   "note": "Self-serve, role-scoped keys, documented token-bucket rate limits with headers, OpenAPI; the API landing page still routes partners through a form."
  },
  "market": {
   "score": 3,
   "note": "Mid-market multi-entity finance teams, from startups to a public company."
  }
 },
 "api": {
  "public": true,
  "style": "REST, JSON, versioned in path (https://api.dualentry.com/public/v2/...); OpenAPI specs for v1, v2 and v3-alpha; CLI",
  "auth": "API key in X-API-KEY header, issued by an org Admin under Settings > Organization > Developer access; keys carry their own roles, do not expire, are bound to one organization and reach every company in it; an Admin-role key bypasses permission checks; user-scoped keys added 25 Sep 2026; MCP uses OAuth 2.1 browser sign-in",
  "limits": "Per key: aggregate bucket burst 50, refill 5/s (300/min); per endpoint burst 10, refill 1/s (60/min); org overrides; 429 with Retry-After and X-RateLimit headers. Pricing page says 200 requests/min on base and Plus, 'on request' on Ultra",
  "cost": "No separate API fee found; included in the subscription",
  "export": "No dedicated bulk export; paginated list endpoints and CLI how-tos that export invoices and journal entries as JSON",
  "notes": "Webhooks are not documented. About 108 release-note pages; many September 2026 releases touched the API (invoice sending, account balances endpoint)."
 },
 "pricing": {
  "model": "Annual subscription by tier, priced by entity count; quote via demo",
  "entry": "'DualEntry' tier: up to 3 entities, unlimited users and transactions, core accounting, SSO; price not published",
  "top": "'Ultra': unlimited entities, advanced revenue recognition, planning and allocations, SAML; price not published",
  "notes": "Implementation included on every plan. Pricing FAQ says no free trial, while the CLI page says 'free to get started'. Through the SVB Offers marketplace (Oct 2026), new customers save up to $20,000. ERP Rescue Fund ($6M) credits the remaining term of a legacy ERP contract."
 },
 "agents": {
  "mcp": "official",
  "ai_features": "AI transaction categorisation, AI bill upload, AI bank match scoring (Sep 2026), anomaly detection, flux analysis, continuous close, 'always-on' auditing; seven free Claude .skill files for close, audit, board reporting, revenue and cash",
  "agent_writes": "Yes. The MCP server (https://api.dualentry.com/mcp/, streamable HTTP, OAuth 2.1) has 15 tools; save_record, save_entity, create_contract and create_fixed_asset write. The CLI creates and sends invoices from JSON.",
  "approval": "No approval step before MCP writes. Every tool checks the user's DualEntry role (View for reads, Create for writes), and MCP never exceeds dashboard access. Preview tools for contract and fixed-asset schedules compute without creating, but are optional. The product has separate approval workflows.",
  "notes": "Agents act as the signed-in user. The CLI page pitches it as a structured interface for agents instead of scraping the UI."
 },
 "integration": {
  "scope": "Claims 13,000+ integrations, mostly bank connections; Stripe (multiple accounts), Salesforce, HubSpot, payroll, data warehouses; two-way Expensify (October 2026); migrations from NetSuite, Sage Intacct, SAP, Dynamics, Great Plains, QuickBooks, PeopleSoft and Xero",
  "notes": "The integration count is dominated by bank connections. 'Go live in 24 hours' sits next to 'live in 4-6 weeks' on the same site; the 24 hours refers to data ingestion."
 },
 "preference": {
  "scale": "Says it has processed over $100 billion in transactions (about page, 2026); about 40 employees (July 2026 press); customers named include Slash, Graphite, Signal Ventures, On Track Storage, Trillion Digital, Streamside Parks",
  "ratings": "Vendor home page cites 4.9 stars on G2 from 120+ reviews (not independently confirmed)",
  "sentiment": "Case studies praise close times (Graphite: 20 days to 1) and $0 implementation. Independent complaints not gathered in this pass; job-aggregator 'critic' notes on go-to-market churn are unverified."
 },
 "timeline": [
  {
   "date": "Before 2025-10",
   "event": "Founded by Santiago Nestares (co-founder of Benitago) and Benedict Dohmen, who had run a 12-person finance team"
  },
  {
   "date": "2025-10-02",
   "event": "Out of stealth with a $90M Series A co-led by Lightspeed and Khosla, $415M post-money; NextDay Migration as the headline feature"
  },
  {
   "date": "2026-02",
   "event": "Product release covering running balances and resubmission tracking"
  },
  {
   "date": "2026",
   "event": "MCP server, Homebrew CLI and Claude finance skills published (exact dates not given)"
  },
  {
   "date": "2026-08-27",
   "event": "$6M ERP Rescue Fund to pay out legacy ERP contracts (date of last update; the launch may be earlier)"
  },
  {
   "date": "2026-09-25",
   "event": "User-scoped API keys, sales quotes, GST/HST return report"
  },
  {
   "date": "2026-10-06",
   "event": "Native two-way integration with Expensify"
  },
  {
   "date": "2026-10-07",
   "event": "Listed on the Silicon Valley Bank Offers marketplace, up to $20,000 off"
  }
 ],
 "relation": "neighbour",
 "relation_note": "DualEntry competes for the company that is leaving NetSuite, Sage or QuickBooks, and wins by making the move cheap. ownpurse argues that you should not have to move at all, and that moving should be one sentence when you do. DualEntry's migration engine is the best evidence that moving is where the pain is. Its open API and MCP make it a plausible future ownpurse source.",
 "borrow": "Rate limits documented as token buckets, with headers and Retry-After that mean exactly one token. Preview tools that compute a schedule without creating it, an honest dry run. A CLI pitched at agents and CI jobs. Version numbers in the API path, with an alpha version published openly.",
 "avoid": "MCP write tools with no approval step, where the user's role is the only gate. API keys that never expire, reach every company in the organization, and with the Admin role skip permission checks. Headline claims ('live in 24 hours', '13,000+ integrations') that the site's own fine print qualifies.",
 "sources": [
  {
   "title": "DualEntry home page",
   "url": "https://www.dualentry.com/"
  },
  {
   "title": "DualEntry pricing",
   "url": "https://www.dualentry.com/pricing"
  },
  {
   "title": "DualEntry about",
   "url": "https://www.dualentry.com/about"
  },
  {
   "title": "DualEntry CLI",
   "url": "https://www.dualentry.com/cli"
  },
  {
   "title": "DualEntry Claude finance skills",
   "url": "https://www.dualentry.com/claude-finance-skills"
  },
  {
   "title": "DualEntry API overview",
   "url": "https://www.dualentry.com/api"
  },
  {
   "title": "DualEntry developer guides",
   "url": "https://docs.dualentry.com/developers/guides"
  },
  {
   "title": "DualEntry docs: authentication",
   "url": "https://docs.dualentry.com/developers/guides/api/authentication.md"
  },
  {
   "title": "DualEntry docs: rate limiting",
   "url": "https://docs.dualentry.com/developers/guides/api/core-concepts/rate-limiting.md"
  },
  {
   "title": "DualEntry docs: MCP tools and permissions",
   "url": "https://docs.dualentry.com/accountants/ai-automation/mcp/mcp-integration.md"
  },
  {
   "title": "DualEntry release notes index",
   "url": "https://docs.dualentry.com/_llms/release-notes.md"
  },
  {
   "title": "DualEntry launches $6M ERP Rescue Fund",
   "url": "https://www.dualentry.com/blog/6m-rescue-fund-launch"
  },
  {
   "title": "Data Mania: DualEntry Series A (citing Reuters)",
   "url": "https://www.data-mania.com/blog/ai-startup-dualentry-series-a-funding-erp-strategy/"
  },
  {
   "title": "DualEntry launches two-way Expensify integration (Business Wire via Yahoo Finance, Oct 2026)",
   "url": "https://finance.yahoo.com/technology/articles/dualentry-launches-native-two-way-111900878.html"
  },
  {
   "title": "DualEntry on the SVB Offers marketplace (Oct 2026)",
   "url": "https://finance.yahoo.com/technology/ai/articles/dualentry-now-available-silicon-valley-112200039.html"
  }
 ],
 "unverified": [
  "Founding year (not stated on DualEntry's site; 2024 is likely but unconfirmed)",
  "Total funding: DualEntry says over $100M; other sources say $96M; only the $90M Series A is confirmed",
  "G2 4.9 stars from 120+ reviews is the vendor's own claim; G2 and Capterra blocked fetch",
  "About 40 employees (July 2026 secondary press)",
  "ERP Rescue Fund launch date: page shows last updated 27 Aug 2026; other metadata suggests February 2026",
  "Rate limit conflict: docs say 300/min aggregate per key; pricing page says 200/min on base and Plus",
  "Free start: CLI page says 'free to get started' while the pricing FAQ says no free trial",
  "Dates the MCP server, CLI and Claude skills first shipped",
  "Reddit, Trustpilot and independent review sentiment not checked: web search budget ran out during this pass",
  "Reuters original article could not be fetched; Series A details rely on secondary copies"
 ],
 "researched": "2026-10-10"
}
---

## What it is

DualEntry is a cloud ERP for mid-market and multi-entity finance teams, pitched as the way off NetSuite, Sage Intacct and their peers. It covers GL, AP, AR, cash, tax, purchase and order management, reconciliation, close, revenue recognition, subscription billing, fixed assets, consolidation, planning and analytics ([DualEntry](https://www.dualentry.com/)). Its signature claim is "NextDay Migration": your history ingested from NetSuite, Sage, SAP, Dynamics, QuickBooks or Xero in about 24 hours. The same site also says "live in 4-6 weeks", so the 24 hours means ingestion, not go-live.

## How it works

You move your books into DualEntry's cloud. AI categorises transactions, scores bank matches, reads uploaded bills, flags anomalies and drafts flux analysis, and an accountant reviews the results. The company sells three tiers by entity count (up to 3, up to 20, unlimited) with implementation included and prices only by demo ([pricing](https://www.dualentry.com/pricing)). It removes switching cost with money as well as software: a $6M ERP Rescue Fund credits the remaining term of a legacy ERP contract ([DualEntry blog](https://www.dualentry.com/blog/6m-rescue-fund-launch)), and the SVB Offers marketplace takes up to $20,000 off.

## Data and access

The REST API is public, versioned (v1, v2 and a published v3-alpha) and large, with about 769 reference pages and OpenAPI specs ([docs index](https://docs.dualentry.com/llms.txt)). An org Admin issues keys from settings. Each key carries its own roles and reaches every company in the organization. Keys never expire, and an Admin-role key bypasses permission checks ([authentication](https://docs.dualentry.com/developers/guides/api/authentication.md)). Rate limits are unusually well documented: per-key token buckets of 300 requests a minute in aggregate and 60 per endpoint, with `X-RateLimit-*` headers and a `Retry-After` that buys exactly one request ([rate limiting](https://docs.dualentry.com/developers/guides/api/core-concepts/rate-limiting.md)). The pricing page says 200 a minute instead. There is no bulk export or webhook page, though CLI guides export invoices and journals as JSON.

## Agents

DualEntry is the most agent-ready of the AI ledgers checked here. Its official MCP server at `api.dualentry.com/mcp/` uses OAuth 2.1 browser sign-in. It exposes 15 tools: search, record detail and history, related records, bank-match suggestions, schedule previews, and four writes (`save_record`, `save_entity`, `create_contract`, `create_fixed_asset`). Each tool checks the user's role, View for reads and Create for writes, and MCP never exceeds what the dashboard allows. Nothing requires a person to approve a write first ([MCP tools](https://docs.dualentry.com/accountants/ai-automation/mcp/mcp-integration.md)). A Homebrew CLI is pitched at agents and CI jobs ([CLI](https://www.dualentry.com/cli)), and seven free Claude skills run on exports from any ERP or on live data over MCP ([skills](https://www.dualentry.com/claude-finance-skills)).

## Customers

DualEntry says it has processed over $100 billion in transactions and serves customers "ranging from startups to a publicly listed firm". Named ones include Slash, Graphite, Trillion and On Track Storage. Its home page cites 4.9 stars on G2 from 120+ reviews, which this pass could not confirm. Press in July 2026 put the team at about 40.

## History

Santiago Nestares, who co-founded Benitago, and Benedict Dohmen came out of stealth on 2 October 2025 with a $90M Series A. Lightspeed and Khosla co-led it, with GV, Contrary and Vesey, at a $415M post-money valuation ([Data Mania, citing Reuters](https://www.data-mania.com/blog/ai-startup-dualentry-series-a-funding-erp-strategy/)). Since then it has shipped almost daily: September 2026 alone brought user-scoped API keys, an invoice-sending API and an account balances endpoint. A two-way Expensify integration followed on 6 October 2026.

## For ownpurse

DualEntry is the clearest case that the real fight is switching cost: it spends money and engineering to make leaving NetSuite take a day. ownpurse makes the opposite bet. Keep the books where they are, hold your own copy, and when you do move, the move is one sentence. Worth taking: rate limits documented as token buckets, preview tools that compute without creating, and a CLI built for agents. Worth avoiding: MCP write tools gated only by role, with no sealed package and no person in the loop, and keys that never expire and reach every company in the organization.
