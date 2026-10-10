---
{
 "slug": "fivetran",
 "name": "Fivetran",
 "url": "https://www.fivetran.com",
 "segment": "connector-framework",
 "status": "active",
 "founded": "2012 (Y Combinator, 2013)",
 "hq": "Oakland, California",
 "backing": "Private, venture-backed; its site cites a $5.6B valuation. Merged with dbt Labs in an all-stock deal announced 13 October 2025 and completed 1 June 2026; the combined company said it would approach $600M ARR. Acquired Teleport Data and HVR (2021); steward of Great Expectations and GX Core (May 2026).",
 "tagline": "A managed service that copies data from 700+ sources into your warehouse or lake, now merged with dbt Labs.",
 "scope": "Fivetran runs hosted connectors that replicate SaaS apps, databases and files into a destination you own, with schema management, delete capture and an SCD2 history mode on some connectors; with dbt it now also transforms, and with Activations writes back to apps. It is not an accounting system and does no bookkeeping or reconciliation; it copies Xero's tables as they are.",
 "domains": [
  "integration",
  "reporting"
 ],
 "customers_served": [
  "mid-market",
  "enterprise",
  "smb"
 ],
 "spectrum": {
  "custody": {
   "score": 2,
   "note": "Data lands in your warehouse, but the pipeline, its state and its schema are Fivetran's; you cannot re-run without them."
  },
  "locality": {
   "score": 1,
   "note": "Cloud service; Hybrid Deployment runs the agent in your environment on Enterprise and Business Critical only."
  },
  "history": {
   "score": 2,
   "note": "History mode keeps SCD2 versions on select tables, only from the moment it is switched on, and restarts on a re-sync."
  },
  "openness": {
   "score": 2,
   "note": "Closed service with a full REST API on every plan, an MIT MCP server and data in your own store; dbt Core is open source."
  },
  "breadth": {
   "score": 3,
   "note": "700+ connectors across every domain, as replicated tables without accounting meaning."
  },
  "autonomy": {
   "score": 2,
   "note": "Syncs unattended every 1 to 15 minutes, and Activations write to SaaS apps unattended; no bookkeeping."
  },
  "agent_readiness": {
   "score": 3,
   "note": "Official MCP server with read-only default scope, REST API, agent plugins to build Connector SDK connectors; no approval gate enforced."
  },
  "integration_ease": {
   "score": 4,
   "note": "No-code setup, self-serve, with a 14-day trial and a free plan."
  },
  "market": {
   "score": 4,
   "note": "Enterprise data teams; names OpenAI, LVMH, Pfizer, Verizon and Spotify."
  }
 },
 "api": {
  "public": true,
  "style": "REST management API (connections, destinations, users, Connector SDK packages); the data itself is read in your own warehouse",
  "auth": "API key and secret; system keys for service accounts",
  "limits": "Rate limits documented on a separate page (figures not confirmed); the source's own limits apply to syncs",
  "cost": "API on all plans including the trial; usage billed by monthly active rows per connection, with a $5 base charge per Standard connection between 1 and 1M MAR; inserts, updates and deletes count; every new history-mode row counts",
  "export": "All replicated data is in your own destination; Managed Data Lake keeps Iceberg tables in your cloud storage",
  "notes": "The Xero connector requires you to create your own Xero app and authorise it; one organisation per connection; an uncertified Xero app is limited to 25 active connections."
 },
 "pricing": {
  "model": "Usage-based by monthly active rows (MAR), priced per connection on a declining curve, plus model runs for transformations",
  "entry": "Free plan: 500,000 MAR, 5,000 model runs a month",
  "top": "Business Critical, or Enterprise License Agreements at a fixed annual price",
  "notes": "Pricing changed 1 March 2025 (re-sync rules, per-connection curves); a $5 per-connection base charge appears on the 2026 pricing page. Example on the pricing page: four ad and marketing connectors for $511.15 a month. Annual contracts save 5% to 22.6%."
 },
 "agents": {
  "mcp": "official",
  "ai_features": "fivetran/fivetran-mcp (MIT, created January 2026, v0.5.1 on 6 October 2026) answers questions about your account and can manage connections; AI coding-agent plugins for Claude Code, Codex, Gemini CLI and Copilot to build Connector SDK connectors; an AI Connector Agent that generates connectors from API docs; dbt Wizard and a Context Layer (September 2026).",
  "agent_writes": "The MCP server is read-only by default; FIVETRAN_SCOPE can raise it to read/write or read/write/delete, and DISALLOWED_ACTIONS denies specific actions.",
  "approval": "Write and delete tools carry advisory warnings asking the client model to confirm, but the server does not enforce confirmation.",
  "notes": "Agents manage pipelines, not books; the Context Layer exposes warehouse context 'via preferred MCP or AI tools'."
 },
 "integration": {
  "scope": "700+ connectors on the Standard plan, destinations across the major warehouses and lakes, dbt, Activations (reverse ETL) and the Python Connector SDK",
  "notes": "Xero connector: re-imports reference tables every sync and contacts daily; captures deletes on accounts, bank transactions, contacts, invoices, manual journals and payments; supports history mode; notes that Xero does not update UpdatedDateUTC on invoice_line_item, so changed account codes need a full re-sync."
 },
 "preference": {
  "scale": "Combined with dbt Labs: more than 100,000 data teams (June 2026); approaching $600M ARR at the announcement (October 2025); dbt alone had 80,000+ teams.",
  "ratings": "Not confirmed (G2, Capterra and TrustRadius unreachable on 2026-10-10).",
  "sentiment": "Not confirmed from review sites. Generally known for reliable, set-and-forget connectors; cost under MAR pricing is the recurring complaint, and competitors (Meltano, dltHub) publish migration guides aimed at it."
 },
 "timeline": [
  {
   "date": "2012",
   "event": "Founded by George Fraser and Taylor Brown; Y Combinator in 2013"
  },
  {
   "date": "2021",
   "event": "Acquires Teleport Data and HVR"
  },
  {
   "date": "2025-03-01",
   "event": "Updated pricing launches, with new re-sync rules and per-connection pricing"
  },
  {
   "date": "2025-10-13",
   "event": "Fivetran and dbt Labs announce an all-stock merger"
  },
  {
   "date": "2026-01-13",
   "event": "fivetran-mcp repository created"
  },
  {
   "date": "2026-05-13",
   "event": "Becomes steward of Great Expectations and GX Core"
  },
  {
   "date": "2026-06-01",
   "event": "Merger with dbt Labs completes; Fraser CEO, Handy President; dbt Core v2.0 alpha under Apache 2.0"
  },
  {
   "date": "2026-09-16",
   "event": "dbt v2, dbt State and Managed Data Lake GA; Context Layer and Lake Compute (DuckDB) in private beta"
  }
 ],
 "relation": "neighbour",
 "relation_note": "Fivetran is how mid-market and enterprise finance teams get Xero and their bank and billing data into one warehouse today. It solves breadth, not ownership: it copies tables to a warehouse for analysts; ownpurse keeps every fact on the owner's machine and rebuilds the books.",
 "borrow": "Publishing per-connector quirks plainly (UpdatedDateUTC not bumped on line items, which tables are re-imported); history-mode columns as a familiar SCD2 projection of a fact log; delete capture listed per table; an MCP server that is read-only by default with scopes and a deny list.",
 "avoid": "History that starts only when switched on and resets on a re-sync; pricing per changed row, which makes history and deletes cost money; advisory-only confirmation for agent writes.",
 "sources": [
  {
   "title": "Fivetran + dbt Labs complete merger (1 June 2026)",
   "url": "https://www.fivetran.com/press/fivetran-dbt-labs-complete-merger-to-create-the-data-infrastructure-for-trusted-ai-agents"
  },
  {
   "title": "Fivetran and dbt Labs unite (13 October 2025)",
   "url": "https://www.fivetran.com/press/fivetran-and-dbt-labs-unite-to-set-the-standard-for-open-data-infrastructure-2025"
  },
  {
   "title": "dbt Labs merger announcement",
   "url": "https://www.getdbt.com/blog/dbt-labs-and-fivetran-merge-announcement"
  },
  {
   "title": "dbt Summit 2026 capabilities (16 September 2026)",
   "url": "https://www.fivetran.com/press/fivetran-dbt-labs-announces-new-capabilities-to-make-enterprise-data-agent-ready-at-dbt-summit-2026"
  },
  {
   "title": "Fivetran pricing",
   "url": "https://www.fivetran.com/pricing"
  },
  {
   "title": "Usage-based pricing docs",
   "url": "https://fivetran.com/docs/usage-based-pricing"
  },
  {
   "title": "History mode",
   "url": "https://fivetran.com/docs/core-concepts/sync-modes/history-mode"
  },
  {
   "title": "Xero connector",
   "url": "https://fivetran.com/docs/connectors/applications/xero"
  },
  {
   "title": "Xero connector setup guide",
   "url": "https://fivetran.com/docs/connectors/applications/xero/setup-guide"
  },
  {
   "title": "Connector SDK",
   "url": "https://fivetran.com/docs/connector-sdk"
  },
  {
   "title": "REST API",
   "url": "https://fivetran.com/docs/rest-api"
  },
  {
   "title": "fivetran/fivetran-mcp on GitHub",
   "url": "https://github.com/fivetran/fivetran-mcp"
  },
  {
   "title": "About Fivetran",
   "url": "https://www.fivetran.com/about"
  },
  {
   "title": "Fivetran press",
   "url": "https://www.fivetran.com/press"
  }
 ],
 "unverified": [
  "Total funding raised and the round behind the $5.6B valuation",
  "Acquisitions of Census and Tobiko Data (not on Fivetran's press pages reached)",
  "Fivetran-only ARR and customer count after the merger",
  "Dollar rate limits for the REST API",
  "Date the $5 per-connection base charge began (2026 pricing page; no date given)",
  "Review-site ratings and counts (G2, Capterra, TrustRadius unreachable)",
  "Whether Xero's 2026 egress fees fall on the customer's own app under Fivetran's setup (inferred from the setup guide)"
 ],
 "researched": "2026-10-10"
}
---

## What it is

Fivetran is the best-known managed ELT service: hosted connectors that copy data from 700+ SaaS apps, databases and files into a warehouse or lake you own. It was founded in 2012 by George Fraser and Taylor Brown and is based in Oakland. On [1 June 2026](https://www.fivetran.com/press/fivetran-dbt-labs-complete-merger-to-create-the-data-infrastructure-for-trusted-ai-agents) it completed an all-stock merger with dbt Labs, first announced on [13 October 2025](https://www.fivetran.com/press/fivetran-and-dbt-labs-unite-to-set-the-standard-for-open-data-infrastructure-2025) with combined ARR approaching $600M. Fraser is CEO and Tristan Handy President, and the combined company says it serves more than 100,000 data teams.

It does what it does well: connectors that keep running without anyone watching them. It does not do bookkeeping. It copies Xero's tables as Xero shapes them.

## How it works

Fivetran syncs incrementally where the source allows it, captures deletes as soft-delete flags, and on supported tables offers [history mode](https://fivetran.com/docs/core-concepts/sync-modes/history-mode): Type 2 versions with `_fivetran_start`, `_fivetran_end` and `_fivetran_active`. History is guaranteed only from the moment you switch it on, and a re-sync marks earlier records inactive and starts again.

The [Xero connector](https://fivetran.com/docs/connectors/applications/xero) is candid about the source. Reference tables are re-imported every sync and contacts once a day. Deletes are captured for accounts, bank transactions, contacts, invoices, manual journals and payments. Xero does not update `UpdatedDateUTC` on `invoice_line_item`, so a changed account code needs a full re-sync. You create your own Xero app and authorise it. Each organisation can be linked to only one connection, and an uncertified app is limited to 25.

## Data and access

The [pricing](https://www.fivetran.com/pricing) is by monthly active rows per connection, on a declining curve. Inserts, updates and deletes count, and every new history-mode row counts too. There is a $5 base charge for each Standard connection between 1 and 1M MAR, and a free plan with 500,000 MAR. The pricing [changed on 1 March 2025](https://fivetran.com/docs/usage-based-pricing). The REST API is on every plan, and your data sits in your own destination, so export is not the issue. Cost and lock-in to the pipeline are.

## Agents

The [official MCP server](https://github.com/fivetran/fivetran-mcp) (MIT, first published January 2026, v0.5.1 on 6 October 2026) is read-only by default. A scope setting raises it to read/write or read/write/delete, and a deny list blocks specific actions. Write tools ask the client model to confirm, but the server does not enforce it. The [Connector SDK](https://fivetran.com/docs/connector-sdk) ships plugins for Claude Code, Codex, Gemini CLI and Copilot, and [dbt Summit 2026](https://www.fivetran.com/press/fivetran-dbt-labs-announces-new-capabilities-to-make-enterprise-data-agent-ready-at-dbt-summit-2026) added dbt Wizard, a Context Layer, and Lake Compute, a DuckDB engine over Iceberg.

## Customers

Mid-market and enterprise data teams. It names OpenAI, LVMH, Pfizer, Verizon and Spotify. A small business can start free, but the product assumes a warehouse and someone to run it. Review-site ratings could not be confirmed. The common complaint is cost under MAR pricing, and both Meltano and dltHub publish guides to leaving it.

## History

Fivetran went through Y Combinator in 2013, bought Teleport Data and HVR in 2021, and became steward of Great Expectations in May 2026. Merging with dbt makes it the default enterprise stack for moving data and modelling it.

## For ownpurse

Fivetran is the answer larger firms already have to getting their data together: a pipe into a warehouse they rent. ownpurse is the answer for a person or small business that has no warehouse and wants to own the record. Borrow the candour of the Xero notes (the `UpdatedDateUTC` gap is why ownpurse runs a weekly full sweep) and the read-only-by-default MCP scopes. Avoid history that starts when someone switches it on and resets on a re-sync, and a price on every change: in ownpurse history is the record itself, and reading it costs nothing.
