---
{
 "slug": "rillet",
 "name": "Rillet",
 "url": "https://www.rillet.com",
 "segment": "ai-ledger",
 "status": "active",
 "founded": "2021",
 "hq": "San Francisco (Series C release dateline)",
 "backing": "Venture-backed, more than $200M raised. Series B led by Andreessen Horowitz (announced 6 August 2025). $100M Series C at a $1B valuation led by ICONIQ, 17 August 2026, with Sequoia, a16z, Bain Capital Ventures, Oak HC/FT, Battery, FirstMark, Scale and Creandum: its third round in about 14 months.",
 "tagline": "An AI-native ERP for venture-backed and mid-market finance teams, sold as the replacement for NetSuite.",
 "scope": "Rillet is a cloud general ledger and ERP with AR, AP, bank reconciliation, ASC 606 revenue recognition, multi-entity consolidation, close management and reporting, fed by native integrations with CRM, billing, payroll and banks. Its Aura AI layer answers questions, runs workflow agents and exposes an MCP server for reads and writes. It replaces the ledger rather than sitting beside one, and it is cloud only.",
 "domains": [
  "ledger",
  "bank",
  "invoicing",
  "payments",
  "close",
  "reporting",
  "tax"
 ],
 "customers_served": [
  "smb",
  "mid-market",
  "enterprise"
 ],
 "spectrum": {
  "custody": {
   "score": 1,
   "note": "Rillet holds the ledger in its cloud; the API and MCP read it out, but no documented bulk export."
  },
  "locality": {
   "score": 0,
   "note": "Cloud only; MCP is remote at api.rillet.com/mcp."
  },
  "history": {
   "score": 1,
   "note": "Every entry traceable to its source and an audit trail by default, but no append-only guarantee or as-of queries documented."
  },
  "openness": {
   "score": 2,
   "note": "Closed source; public OpenAPI spec, but API access must be enabled by the Rillet team; export unconfirmed."
  },
  "breadth": {
   "score": 3,
   "note": "GL, AR, AP, bank rec, revenue recognition, multi-entity, sales tax and many native integrations."
  },
  "autonomy": {
   "score": 3,
   "note": "Perpetual ledger posts automatically; vendor says 93% of journal entries are booked automatically."
  },
  "agent_readiness": {
   "score": 3,
   "note": "Official MCP mirroring the public API with reads and writes, OAuth or API key, approval routing by source."
  },
  "integration_ease": {
   "score": 2,
   "note": "Well-designed API (idempotency keys, sandbox, decimal money) but gated behind enablement and sales-led onboarding; 60 requests a minute."
  },
  "market": {
   "score": 3,
   "note": "Venture-backed scale-ups to public companies replacing NetSuite, Oracle and SAP."
  }
 },
 "api": {
  "public": true,
  "style": "REST with an OpenAPI spec; keyset pagination (25 default, 100 max, cursors expire in 2 hours); Idempotency-Key on POST; RFC 9457 errors; money as decimal string with currency; version header X-Rillet-API-Version",
  "auth": "API key as Bearer token, created in Organization Settings once the Rillet team enables API access; OAuth2 for MCP",
  "limits": "60 requests per rolling minute; 429 when exceeded",
  "cost": "Not published; included in a subscription priced by features and complexity",
  "export": "Not documented. The API exposes the same features as the product, so reads are broad, but no bulk export or exit terms were found.",
  "notes": "Sandbox at sandbox.api.rillet.com. Default API version moved to latest on 1 August 2026. September 2026: custom invoice schedules with a preview endpoint, journal entry attachments."
 },
 "pricing": {
  "model": "Custom annual subscription priced by features used and complexity, not seats or revenue; sales-led",
  "entry": "Not published; implementation $10K over 4-8 weeks (Rillet's llms.txt)",
  "top": "Not published",
  "notes": "Rillet contrasts its implementation with NetSuite quotes of $150K-$400K+ (its own comparison). Request a demo to get a price."
 },
 "agents": {
  "mcp": "official",
  "ai_features": "Aura Assistant (questions over the GL), Aura Agents (workflows described in plain English, with scheduling and handoffs), Continuous AI that flags anomalies and proposes accruals, ASC 606 analysis from contracts, fixed-asset grouping from a mapping file.",
  "agent_writes": "Yes. The MCP exposes the same features as the public API, 'one secure surface for reads and writes', limited by the connected user's permissions. Works with Claude, ChatGPT and Codex (September 2026).",
  "approval": "Journal entry approval workflows can route by source (Aura, UI, public API or MCP client), combined with amount, creator, subsidiary and account conditions. 'Approve and edit every step.'",
  "notes": "Source-based approval routing is a direct answer to agents writing to the books: the same entry can need review when an agent wrote it and not when a person did."
 },
 "integration": {
  "scope": "Native integrations named include Stripe, Salesforce, HubSpot, Rippling, Gusto, ADP, Deel, Ramp, Brex, Bill, Tipalti, Avalara, Anrok, Snowflake, Anaplan, Maxio, J.P. Morgan, Bank of America, RBC; acts as an agent of Plaid for account information.",
  "notes": "White-glove, CPA-led implementations of 4-8 weeks; partnerships with EY and more than half of the Accounting Today top 20 CPA firms (August 2026)."
 },
 "preference": {
  "scale": "More than 600 customers and new ARR doubled over three months (17 August 2026); 500+ finance teams in its llms.txt. Logos include Mercor, Temporal, Kickstarter, Sotheby's, LangChain.",
  "ratings": "G2 5.0 as stated on Rillet's site ('only ERP with 5 stars on G2'); review count not confirmed because G2 blocked the fetch.",
  "sentiment": "Controllers praise close speed (VSee 30 days to 4, Postscript to 3, vendor case studies) and the integrations. We could not reach independent complaints in this pass."
 },
 "timeline": [
  {
   "date": "2021",
   "event": "Founded by Nicolas Kopp and Stelios Modes"
  },
  {
   "date": "2025",
   "event": "Sequoia invests (Series A per Sequoia's portfolio page year)"
  },
  {
   "date": "2025-08-06",
   "event": "Andreessen Horowitz announces it is leading Rillet's Series B"
  },
  {
   "date": "2026-08-01",
   "event": "Default public API version moves to latest"
  },
  {
   "date": "2026-08-17",
   "event": "$100M Series C at $1B led by ICONIQ; 600+ customers; more than $200M raised in total"
  },
  {
   "date": "2026-09",
   "event": "MCP extended to ChatGPT and Codex; journal entry approvals can route by source, including MCP clients"
  }
 ],
 "relation": "neighbour",
 "relation_note": "Rillet replaces NetSuite for growing companies and holds the books in its cloud; ownpurse sits beside whatever ledger you keep and holds a copy you own. It shows where the 2025-26 money went: into new ledgers, not into owners holding their record.",
 "borrow": "Approval routing by the source of an entry, so agent-written entries can be reviewed differently; idempotency keys with a 24-hour saved response; money as a decimal string with a currency; a preview endpoint that shows what a write would create before it does.",
 "avoid": "API access that the vendor has to switch on; sales-only pricing; a migration path that moves the books in with no stated way out.",
 "sources": [
  {
   "title": "Rillet home page",
   "url": "https://www.rillet.com"
  },
  {
   "title": "Rillet pricing",
   "url": "https://www.rillet.com/pricing"
  },
  {
   "title": "Rillet API getting started",
   "url": "https://docs.api.rillet.com/docs/getting-started"
  },
  {
   "title": "Rillet MCP docs",
   "url": "https://docs.api.rillet.com/docs/mcp"
  },
  {
   "title": "Aura AI",
   "url": "https://www.rillet.com/product/aura-ai"
  },
  {
   "title": "What's new in Rillet: September 2026",
   "url": "https://www.rillet.com/blog/whats-new-in-rillet-september-2026-product-updates"
  },
  {
   "title": "Rillet raises $100M Series C at $1B valuation",
   "url": "https://www.rillet.com/blog/rillet-raises-100m-series-c-at-1b-valuation-to-build-accounting-superintelligence"
  },
  {
   "title": "Investing in Rillet, a16z, 6 August 2025",
   "url": "https://a16z.com/announcement/investing-in-rillet/"
  },
  {
   "title": "Rillet, Sequoia portfolio",
   "url": "https://www.sequoiacap.com/companies/rillet/"
  },
  {
   "title": "Rillet customers",
   "url": "https://www.rillet.com/customers"
  },
  {
   "title": "Rillet llms.txt",
   "url": "https://www.rillet.com/llms.txt"
  },
  {
   "title": "Rillet content for AI",
   "url": "https://info.rillet.com/"
  }
 ],
 "unverified": [
  "Amounts and dates of the Series A and Series B (commonly reported as $25M and $70M; not confirmed from a primary source here)",
  "Headquarters beyond the San Francisco dateline",
  "G2 review count behind the 5.0 rating",
  "Subscription prices",
  "Data export options and exit terms",
  "Whether Aura agents post journal entries directly or only propose them by default",
  "The 93% automatic booking and close-time figures are vendor claims",
  "Web search budget ran out mid-research; findings come from direct fetches of primary pages"
 ],
 "researched": "2026-10-10"
}
---

## What it is

Rillet is an AI-native ERP for finance teams at venture-backed and mid-market companies, sold as the replacement for NetSuite. It was [founded in 2021](https://www.sequoiacap.com/companies/rillet/) by Nicolas Kopp and Stelios Modes. It covers the general ledger, AR, AP, bank reconciliation, ASC 606 revenue recognition, multi-entity consolidation and close management. Its pitch is a "perpetual general ledger" and a "zero-day close". [Pricing](https://www.rillet.com/pricing) is not published. Its llms.txt gives implementation as $10K over 4-8 weeks, against NetSuite quotes it puts at $150K-$400K+.

## How it works

Rillet pulls from billing, CRM, payroll, spend and banks (Stripe, Salesforce, HubSpot, Rippling, Ramp, Brex, J.P. Morgan and more) and posts entries as events happen, each traceable to its source. Its [case studies](https://www.rillet.com/customers) claim 93% of journal entries booked automatically and an average of 7 days saved each month. Implementations are white-glove and led by CPAs. Like Digits and Puzzle, Rillet becomes the book of record: you move the books in.

## Data and access

The [public API](https://docs.api.rillet.com/docs/getting-started) is well made. It publishes an OpenAPI spec and a sandbox. Money is a decimal string with a currency, POSTs take an `Idempotency-Key`, errors follow RFC 9457, pagination uses keysets, and versions are pinned with a header. Two things count against it: the Rillet team must enable API access before you can create a key, and the limit is 60 requests a minute. We found no bulk export or exit terms.

## Agents

Aura is Rillet's AI layer. The Assistant answers questions over the ledger. Agents run workflows you describe in plain English. Continuous AI flags anomalies and proposes accruals. The [Rillet MCP server](https://docs.api.rillet.com/docs/mcp) at `api.rillet.com/mcp` exposes the same features as the public API, reads and writes, and an agent can do only what the connected user is permitted to do. In [September 2026](https://www.rillet.com/blog/whats-new-in-rillet-september-2026-product-updates) it reached ChatGPT and Codex. Journal entry approvals can now route by source: an entry from Aura, the UI, the API or an MCP client can each follow its own review path. That is a direct answer to agents writing to the books, and it sits inside Rillet's cloud.

## Customers

Rillet reported more than 600 customers on 17 August 2026, with new ARR doubled over the previous three months. Named customers include Mercor, Temporal, Kickstarter and Sotheby's. Its site claims a 5.0 rating on G2; we could not confirm the review count.

## History

Rillet's rise is the clearest example of the 2025-26 money going into replacement ledgers. Andreessen Horowitz [led its Series B](https://a16z.com/announcement/investing-in-rillet/) in August 2025. A year later ICONIQ led a [$100M Series C at $1B](https://www.rillet.com/blog/rillet-raises-100m-series-c-at-1b-valuation-to-build-accounting-superintelligence), its third round in about 14 months, taking the total past $200M. The stated plan is to move beyond tech companies to biotech, healthcare, fintech and logistics, and to target users of Oracle, SAP, Workday and NetSuite. Kopp: "Finance agents need more than access to data; they need to work inside the general ledger."

## For ownpurse

Rillet bets on the opposite of ownpurse: agents work inside one vendor's ledger. ownpurse keeps the record outside every ledger, in a file the owner holds. Rillet's customers are mid-market, not ownpurse's first users. Its API and approval design are still worth copying: approval routing by the source of an entry, idempotency keys, decimal money with a currency, and a preview endpoint that shows what a write would create before anything is created. That preview is close in spirit to a sealed package.
