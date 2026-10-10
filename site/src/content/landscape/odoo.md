---
{
 "slug": "odoo",
 "name": "Odoo",
 "url": "https://www.odoo.com",
 "segment": "incumbent-ledger",
 "status": "active",
 "founded": "2005 (as TinyERP; OpenERP from 2008, Odoo from 2014)",
 "hq": "Grand-Rosière, Belgium",
 "backing": "Mostly self-funded and profitable; €3 million in 2010, a $10 million Series B in 2014, $90 million in December 2019 (Summit Partners), and a €500 million (about $527 million) secondary round in November 2024 led by CapitalG and Sequoia at a €5 billion ($5.26 billion) valuation.",
 "tagline": "An open-core suite of business apps, accounting included, that you can run in Odoo's cloud or host yourself.",
 "scope": "About 80 official apps sharing one database: accounting and invoicing, bank reconciliation, expenses, inventory, manufacturing, CRM, sales, POS, ecommerce, payroll, documents, sign, projects, HR and more, plus 50,000+ community modules. Community edition is LGPLv3; Enterprise (including the full accounting app and Studio) is proprietary. Accounting is localised for many countries; it is a business system, not personal finance.",
 "domains": [
  "ledger",
  "bank",
  "invoicing",
  "payroll",
  "documents",
  "tax",
  "spend",
  "close",
  "reporting",
  "commerce",
  "payments",
  "integration"
 ],
 "customers_served": [
  "smb",
  "mid-market",
  "enterprise",
  "accountants",
  "developers"
 ],
 "spectrum": {
  "custody": {
   "score": 3,
   "note": "Self-hosted, the owner holds the PostgreSQL database and filestore outright; on Odoo Online, Odoo holds it, but you can download a full database backup."
  },
  "locality": {
   "score": 3,
   "note": "Community and Enterprise run on your own server or laptop (Python and PostgreSQL); Odoo Online and Odoo.sh are options, not requirements."
  },
  "history": {
   "score": 2,
   "note": "Optional SHA-256 hash chaining locks posted journal entries per journal, with an inalterability report, and chatter tracks field changes; but drafts are edited in place and there are no as-of queries."
  },
  "openness": {
   "score": 3,
   "note": "Open core: the Community edition is LGPLv3 and anyone can run it; Enterprise apps are proprietary and the hosted API is gated to the Custom plan."
  },
  "breadth": {
   "score": 4,
   "note": "Ledger, bank, invoicing, payroll, documents, POS, ecommerce, payments and inventory in one database."
  },
  "autonomy": {
   "score": 2,
   "note": "Reconciliation models match bank lines automatically, OCR drafts vendor bills, and AI server actions can fill fields; posting usually stays with people."
  },
  "agent_readiness": {
   "score": 2,
   "note": "A complete JSON-2 API with bearer API keys and built-in AI agents, but no official external MCP server; MCP comes from community projects."
  },
  "integration_ease": {
   "score": 3,
   "note": "Self-hosted, every model is reachable through a uniform, documented API at no cost; on Odoo Online, external API access needs the Custom plan."
  },
  "market": {
   "score": 2,
   "note": "Mostly small and mid-sized businesses, with reach into larger companies through its 21,000+ partners."
  }
 },
 "api": {
  "public": true,
  "style": "JSON-2 API since Odoo 19: POST /json/2/<model>/<method> over the ORM, with per-database /doc pages. XML-RPC and JSON-RPC at /xmlrpc and /jsonrpc are deprecated and scheduled for removal in Odoo 22 (fall 2028).",
  "auth": "Bearer API keys (160-bit, expire after at most three months, so they must be rotated); older RPC uses login and password or a key",
  "limits": "No published rate limits for the external API found in the docs; self-hosted capacity is your own server's.",
  "cost": "Free on a self-hosted install; on Odoo Online the external API is only available on the Custom plan (€22.90 per user per month, yearly) and not on One App Free or Standard.",
  "export": "Any list view exports to CSV or XLSX with chosen fields and an import-compatible option; database managers download a full backup (PostgreSQL dump plus filestore) that restores into another Odoo, self-hosted or hosted.",
  "notes": "The API exposes every model and method a user may call, including posting journal entries; access rights and record rules apply as for that user."
 },
 "pricing": {
  "model": "Per user per month for all apps, with one app free; the hosting option depends on the plan",
  "entry": "One App Free: €0 for a single app with unlimited users; Standard €11.90 per user per month (yearly, Odoo Online only)",
  "top": "Custom €22.90 per user per month (yearly), adding Studio, agentic AI, multi-company, external API, and Odoo.sh or on-premise hosting",
  "notes": "Prices in EUR read from odoo.com/pricing on 2026-10-10; monthly billing shows €14.90 and €27.90, and an introductory first-year discount applies. Odoo.sh hosting is charged separately. The Community edition is free to self-host with no licence fee. Implementation is usually done by partners."
 },
 "agents": {
  "mcp": "community",
  "ai_features": "Odoo 19 (2025-09-18) added an AI app: Ask AI from anywhere, natural-language search turned into domains, AI fields, AI in server actions, document-grounded agents for live chat and lead creation, drafting and summaries, voice transcription; you pick ChatGPT or Gemini models per agent. Accounting gets improved reconciliation models, bank statement OCR and PO matching on imported bills.",
  "agent_writes": "Built-in agents only write through the Topics and tools you give them (for example Create Leads); the default Ask AI agent can't change the database. External agents write through the JSON-2 API or community MCP servers such as mcp-server-odoo (401 GitHub stars), which can create, update and delete records.",
  "approval": "No general approval gate for agent writes was found in the docs; control comes from the user's access rights, Topics, and journal locks and hashing.",
  "notes": "Agentic AI is listed as a Custom plan feature. The community MCP server's standard mode needs an Odoo module and recommends it for production over its 'YOLO' mode on raw XML-RPC."
 },
 "integration": {
  "scope": "50+ official apps and 50,000+ community apps on the Odoo Apps store, 21,000+ partners, built-in payment providers (Adyen, Stripe, Mollie and others), bank synchronisation, EDI and e-invoicing for many countries.",
  "notes": "Most integrations are Odoo modules installed into the database rather than outside services, which makes deep integration easy and upgrades harder."
 },
 "preference": {
  "scale": "Over 28 million users, 8,000+ employees and 21,000+ partners (odoo.com about page, read 2026-10-10). TechCrunch (2024-11-20) reported billings projected above €650 million over the next 12 months, about 40% growth a year and a target of €1 billion by 2027. De Tijd (June 2025) reported revenue of €282 million, up 33%.",
  "ratings": "Trustpilot 3.1 from 1,291 reviews, 46% five-star and 43% one-star (read 2026-10-10). G2 and Capterra could not be fetched.",
  "sentiment": "People praise the breadth, the price against SAP and NetSuite, the modules once configured, and responsive live-chat support. Complaints: a steep learning curve and setup that needs paid partners, customisation costing more than expected, upgrades that break things, annual-contract and refund disputes, and pushy sales."
 },
 "timeline": [
  {
   "date": "2005-02",
   "event": "Fabien Pinckaers releases TinyERP"
  },
  {
   "date": "2008",
   "event": "Renamed OpenERP"
  },
  {
   "date": "2010-02",
   "event": "Raises €3 million for US expansion"
  },
  {
   "date": "2014",
   "event": "Renamed Odoo; $10 million Series B, its last primary round for years"
  },
  {
   "date": "2015",
   "event": "Version 9 moves to open core: LGPL Community, proprietary Enterprise"
  },
  {
   "date": "2019-12-17",
   "event": "Raises $90 million, mainly from Summit Partners"
  },
  {
   "date": "2022-10-18",
   "event": "New single per-user pricing for all apps"
  },
  {
   "date": "2024-11-20",
   "event": "€500 million secondary round led by CapitalG and Sequoia at a €5 billion valuation"
  },
  {
   "date": "2025-09-18",
   "event": "Odoo 19 released with the AI app and the JSON-2 API; XML-RPC and JSON-RPC deprecated"
  },
  {
   "date": "2028 (planned)",
   "event": "XML-RPC and JSON-RPC endpoints to be removed in Odoo 22"
  }
 ],
 "relation": "kin",
 "relation_note": "The open-source incumbent: a ledger an owner can run and hold themselves, with optional hash-chained entries. It is the nearest relative to ownpurse's sovereignty idea, and a likely source connector.",
 "borrow": "A database the owner can back up, restore and self-host; SHA-256 chaining of posted entries with an inalterability report; one uniform API over every model; LTS releases on a predictable yearly cycle.",
 "avoid": "Gating API access by plan, API keys that expire every three months for long-running syncs, a module sprawl that makes upgrades painful, and AI writing fields from server actions with no record of who proposed what and who approved it.",
 "sources": [
  {
   "title": "Odoo pricing",
   "url": "https://www.odoo.com/pricing"
  },
  {
   "title": "Odoo about us",
   "url": "https://www.odoo.com/page/about-us"
  },
  {
   "title": "Odoo 19 external JSON-2 API",
   "url": "https://www.odoo.com/documentation/19.0/developer/reference/external_api.html"
  },
  {
   "title": "Odoo 19 data inalterability check report",
   "url": "https://www.odoo.com/documentation/19.0/applications/finance/accounting/reporting/data_inalterability.html"
  },
  {
   "title": "Odoo 19 AI (Ask AI)",
   "url": "https://www.odoo.com/documentation/19.0/applications/productivity/ai.html"
  },
  {
   "title": "Odoo 19 AI agents",
   "url": "https://www.odoo.com/documentation/19.0/applications/productivity/ai/agents.html"
  },
  {
   "title": "Odoo 19 release notes",
   "url": "https://www.odoo.com/odoo-19-release-notes"
  },
  {
   "title": "TechCrunch: Odoo raises $527M via secondaries at $5.26B (2024-11-20)",
   "url": "https://techcrunch.com/2024/11/20/riding-high-on-open-source-erp-odoo-raises-527m-via-secondaries-lifting-its-valuation-to-5-26b/"
  },
  {
   "title": "TechCrunch: Odoo grabs $90M (2019-12-17)",
   "url": "https://techcrunch.com/2019/12/17/odoo-grabs-90m-to-sell-more-smes-on-its-business-app-suite/"
  },
  {
   "title": "Odoo (Wikipedia)",
   "url": "https://en.wikipedia.org/wiki/Odoo"
  },
  {
   "title": "Odoo on Trustpilot",
   "url": "https://www.trustpilot.com/review/odoo.com"
  },
  {
   "title": "mcp-server-odoo (community)",
   "url": "https://github.com/ivnvxd/mcp-server-odoo"
  }
 ],
 "unverified": [
  "Which year the €282 million revenue (De Tijd, June 2025) refers to: Wikipedia says 2023, but a June 2025 report may cover 2024.",
  "Whether the hash-chaining inalterability feature is in Community or Enterprise only.",
  "The pairing of yearly and monthly prices on the pricing page (€11.90/€22.90 yearly versus €14.90/€27.90 monthly) was inferred from the scraped page.",
  "Whether Odoo 20 was released or announced by 2026-10-10.",
  "G2 and Capterra ratings: not fetched.",
  "Whether Odoo Online backup download is available on every plan.",
  "Whether any official Odoo MCP server was announced after the docs read."
 ],
 "researched": "2026-10-10"
}
---

## What it is

Odoo is a Belgian suite of business apps, accounting among them, all sharing one PostgreSQL database. Fabien Pinckaers released it as TinyERP in 2005; it became OpenERP in 2008 and Odoo in 2014. Since version 9 it has been open core: the Community edition is LGPLv3 and anyone can run it, while Enterprise adds proprietary apps, including the full accounting app and Studio. Odoo's [about page](https://www.odoo.com/page/about-us) claims over 28 million users, 8,000+ staff and 21,000+ partners (read 2026-10-10).

## How it works

You install the apps you need (invoicing, accounting, inventory, POS, ecommerce, payroll, documents, CRM) and they share records, so a POS sale, a stock move and a journal entry live in the same database. Accounting is localised for many countries, with bank sync, reconciliation models, OCR for bills and statements, and e-invoicing.

[Pricing](https://www.odoo.com/pricing) (read 2026-10-10, EUR, yearly): One App Free costs nothing for a single app; Standard is €11.90 per user per month for all apps on Odoo Online; Custom is €22.90 and adds Studio, agentic AI, multi-company, external API access, and the right to host on Odoo.sh or your own servers. Community is free to self-host. Partners do most implementations.

## Data and access

Odoo 19 introduced the [JSON-2 API](https://www.odoo.com/documentation/19.0/developer/reference/external_api.html): one POST per model and method with a bearer API key, over the same ORM the UI uses, so every model is reachable. XML-RPC and JSON-RPC are deprecated and due to be removed in Odoo 22 in fall 2028. API keys last at most three months. On Odoo Online the external API is only on the Custom plan; self-hosted, it costs nothing.

Custody is Odoo's distinctive strength among incumbents. Self-hosted, you hold the database and filestore; on Odoo Online you can still download a full backup that restores into another Odoo. History is partial: drafts are edited in place, but journals can [secure posted entries with a SHA-256 hash chain](https://www.odoo.com/documentation/19.0/applications/finance/accounting/reporting/data_inalterability.html), where each entry's hash includes the previous one, and a report checks the chain. Secured entries can't be edited, and the setting can't be undone. It began as a compliance feature for French inalterability rules, and it is the closest thing to ownpurse's chain in any mainstream ledger.

## Agents

Odoo 19's [AI app](https://www.odoo.com/documentation/19.0/applications/productivity/ai.html) brings Ask AI everywhere, natural-language search, AI fields and server actions, and [agents](https://www.odoo.com/documentation/19.0/applications/productivity/ai/agents.html) grounded on your documents, running ChatGPT or Gemini models. The default agent can't change the database; others can only act through the Topics and tools you give them. There is no official external MCP server. Community projects fill the gap, notably [mcp-server-odoo](https://github.com/ivnvxd/mcp-server-odoo) (MPL-2.0, 401 stars), which can create, update and delete records. No per-write approval gate is documented.

## Customers

Odoo serves small and mid-sized businesses worldwide, from shops on POS to manufacturers. [TechCrunch](https://techcrunch.com/2024/11/20/riding-high-on-open-source-erp-odoo-raises-527m-via-secondaries-lifting-its-valuation-to-5-26b/) reported about 40% annual growth and billings projected above €650 million over the 12 months after November 2024. Sentiment is split: [Trustpilot](https://www.trustpilot.com/review/odoo.com) shows 3.1 from 1,291 reviews, with 46% five-star and 43% one-star. People praise the breadth and price; they complain about setup complexity, partner costs, upgrades that break things, and contract disputes.

## History

Odoo grew largely on revenue: €3 million in 2010, a $10 million Series B in 2014, $90 million in 2019 led by Summit Partners, then a €500 million secondary in November 2024 led by CapitalG and Sequoia at a €5 billion valuation. It is the one open-source ERP that became a large company, by giving away a capable core and charging for hosting, Enterprise apps and convenience.

## For ownpurse

Odoo is kin. It shows owners will run their own books when the software lets them, and that hash-chained posted entries can ship in a mainstream ledger. It differs from ownpurse in shape: Odoo is the system of record for one business's operations, a mutable relational database with an optional lock, while ownpurse is an append-only log of facts from many sources with every version kept. A connector would read an Odoo database over JSON-2 or straight from a self-hosted PostgreSQL, compare its hash chain, and sit beside the other sources. The gap ownpurse can fill is the whole-picture record (Odoo plus the bank, the household and the wallets), as-of queries, and agent writes that go out as sealed, approved packages instead of API calls with a three-month key.
