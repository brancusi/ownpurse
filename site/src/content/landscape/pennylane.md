---
{
 "slug": "pennylane",
 "name": "Pennylane",
 "url": "https://www.pennylane.com",
 "segment": "ai-ledger",
 "status": "active",
 "founded": "2020",
 "hq": "Paris, France",
 "backing": "Venture-backed. Seed of about EUR 4M (May 2020; Global Founders Capital, Partech, Kima Ventures); EUR 40M (February 2024, valuation above EUR 1B); EUR 75M led by Sequoia with CapitalG, Meritech and DST (April 2025, EUR 2B); EUR 175M led by TCV with Blackstone Growth (January 2026, EUR 3.5B per Maddyness; Tech.eu reported USD 200M).",
 "tagline": "A French cloud ledger, invoicing and business account that a company and its accountant work in together.",
 "scope": "Pennylane is one shared accounting system for a small or mid-sized company and its accounting firm: bank sync, invoicing, supplier bills, expense reports, a business account and cards, VAT, closing and annual accounts, plus production software for the firm. It is sold in France, Germany and Spain. It does not run outside its cloud, and its AI connector for outside assistants is read-only.",
 "domains": [
  "ledger",
  "bank",
  "payments",
  "invoicing",
  "documents",
  "tax",
  "spend",
  "close",
  "reporting"
 ],
 "customers_served": [
  "freelancer",
  "smb",
  "mid-market",
  "accountants"
 ],
 "spectrum": {
  "custody": {
   "score": 1,
   "note": "Pennylane holds the books; an owner can pull a FEC, a general ledger export and a Data Sharing feed, so a full copy is possible but not authoritative."
  },
  "locality": {
   "score": 0,
   "note": "Cloud only."
  },
  "history": {
   "score": 1,
   "note": "Ledger entries can be updated through the API; changelog endpoints keep inserts, updates and deletes for only 4 weeks."
  },
  "openness": {
   "score": 2,
   "note": "Documented public REST API v2 with scopes, plus FEC and general ledger exports and a read-only warehouse feed; closed source."
  },
  "breadth": {
   "score": 4,
   "note": "Books, bank account and cards, invoicing and e-invoicing, supplier payments, expenses, VAT and close in one product; payment terminals in beta from February 2026."
  },
  "autonomy": {
   "score": 3,
   "note": "Autopilot runs activated workflows such as bank reconciliation and bookkeeping end to end, with the team supervising exceptions; Copilot only suggests."
  },
  "agent_readiness": {
   "score": 3,
   "note": "Official OAuth MCP server (read-only, seven tool groups), public API with scopes, llms.txt docs; no CLI and no agent write path."
  },
  "integration_ease": {
   "score": 3,
   "note": "Self-serve company tokens and OAuth apps, webhooks, changelog endpoints and good docs; 25 requests per 5 seconds per token and API only from the Essentiel plan up."
  },
  "market": {
   "score": 2,
   "note": "Freelancers and small businesses through their accountants, moving into PME/ETI with custom offers."
  }
 },
 "api": {
  "public": true,
  "style": "REST (Company API v2 and Firm API), JSON, cursor pagination, webhooks, changelog endpoints",
  "auth": "Company or firm developer tokens, or OAuth 2.0 apps, with per-resource scopes (for example ledger_entries:readonly / ledger_entries:all, exports:fec)",
  "limits": "25 requests every 5 seconds per token, in production and sandbox (rate limit page updated 2026-02-06); changelog history retained 4 weeks; list pages up to 1,000 items",
  "cost": "No separate API fee found; API and integrations are included from the Essentiel plan (EUR 24 HT/month for sole traders as of 2026-10-10) and not available on Starter or Basique. Data Sharing pricing not found.",
  "export": "FEC export and general ledger and analytical general ledger exports (xlsx) through the API; Data Sharing gives a read-only, few-hours-delayed feed to BI tools and warehouses such as Snowflake, Power BI and Tableau.",
  "notes": "Pennylane publishes a 2026 API migration guide and a changelog RSS feed with entries through 30 September 2026. Ledger entries, journals, invoices, suppliers, customers, transactions and categories are all reachable."
 },
 "pricing": {
  "model": "Monthly subscription per company, by plan tier; accounting firms on separate firm offers",
  "entry": "EUR 7 HT/month (Starter, sole traders) as of 2026-10-10",
  "top": "EUR 79 HT/month (Premium, sole traders); TPE and PME/ETI plans are priced separately and larger companies get a custom offer",
  "notes": "Sole-trader plans: Starter EUR 7, Basique EUR 14, Essentiel EUR 24, Premium EUR 79, all HT per month with a 15-day trial and up to 1,200 invoices a year. API access and the MCP connector start at Essentiel. Firm (cabinet) prices were not published on the page fetched."
 },
 "agents": {
  "mcp": "official",
  "ai_features": "Copilot (invoice reading with OCR and Factur-X, suggested entries, categories and reconciliations, summaries), ComptAssistant (questions over your financial data), Autopilot (runs activated workflows such as reconciliation, document requests and VAT checks), customizable agents, and an announced Studio IA for building your own agents. Homepage cites 84% of transactions reconciled automatically.",
  "agent_writes": "Outside agents cannot write through the MCP: it is read-only by design. Agents can write through the public REST API with a token that has write scopes (for example creating ledger entries or invoices). Pennylane's own Autopilot writes inside the product.",
  "approval": "No approval gate for API writes; scopes and plan level are the controls. The MCP avoids the question by being read-only. Copilot suggestions need a person to validate; Autopilot runs within a scope you configure and your team reviews exceptions.",
  "notes": "MCP server at https://mcp.pennylane.com/mcp, OAuth with your Pennylane login, seven tool categories that can be switched on individually, including a history-of-changes category. Free for accounting firms; companies need Essentiel or above. Third-party servers exist too (PennyPilot, read-only and local over stdio; Truto, hosted)."
 },
 "integration": {
  "scope": "Bank sync, payment and invoicing tools, POS integrations through the API, BI warehouses through Data Sharing, and MCP clients including Claude, ChatGPT, Mistral, Notion, Lovable, Copilot, Perplexity and Gemini. A full integration count was not found.",
  "notes": "The changelog endpoints are the best sync primitive here: chronological insert, update and delete events per resource, but only for 4 weeks, so a connector must poll at least that often or lose the trail."
 },
 "preference": {
  "scale": "About 800,000 business customers and more than 6,000 accounting firms, EUR 115M ARR in 2025 (Maddyness, 20 January 2026). Homepage as of 2026-10-10 claims 1.5 million businesses, 6,500+ firms and 20 million invoices a month; the two counts may measure different things.",
  "ratings": "Trustpilot 4.3/5 from 1,034 reviews, 80% five-star and 12% one-star (fetched 2026-10-10). Pennylane claims 4.6/5 on Google. G2 and Capterra pages could not be fetched.",
  "sentiment": "Reviewers praise responsive support, the shared view between finance lead and accountant, and integrations for SaaS businesses. Complaints cover slow or hard-to-reach support, unreliable 2FA, GoCardless connection problems, API changes and mobile app errors."
 },
 "timeline": [
  {
   "date": "2020-03",
   "event": "Starts accepting customers in France as a financial data service with accountants checking the data"
  },
  {
   "date": "2020-05",
   "event": "Seed of about EUR 4M from Global Founders Capital, Partech and Kima Ventures"
  },
  {
   "date": "2021-01",
   "event": "Raises USD 18.4M (TechCrunch)"
  },
  {
   "date": "2024-02",
   "event": "Raises EUR 40M at a valuation above EUR 1B"
  },
  {
   "date": "2024",
   "event": "Acquires Billy (per Maddyness)"
  },
  {
   "date": "2025-04",
   "event": "EUR 75M led by Sequoia at EUR 2B; about 4,500 firms and 350,000 SMEs (CNBC)"
  },
  {
   "date": "2025-11",
   "event": "Bloomberg reports talks at about USD 4.3B"
  },
  {
   "date": "2026-01",
   "event": "EUR 175M led by TCV with Blackstone Growth at EUR 3.5B; 800,000 companies and 6,000 firms"
  },
  {
   "date": "2026-02",
   "event": "Payment terminal beta planned, about 1,000 companies pre-registered"
  },
  {
   "date": "2026",
   "event": "Official read-only MCP server available; e-invoicing certified for France's September 2026 reform"
  }
 ],
 "relation": "source",
 "relation_note": "Pennylane is a ledger ownpurse would read from, not replace: a strong connector candidate for European books, with a public API, a FEC export and change feeds. It asks owners to move their books into its cloud; ownpurse leaves books where they are and keeps the history Pennylane drops after 4 weeks.",
 "borrow": "Changelog endpoints with insert, update and delete operations in chronological order; the FEC as a standard full-ledger export; tool groups an owner can switch on one by one; a read-only MCP as the safe default; one shared view for owner and accountant.",
 "avoid": "A 4-week retention window on change history; write access through broad API tokens with no approval step; API and agent access locked behind a higher plan.",
 "sources": [
  {
   "title": "Pennylane homepage",
   "url": "https://www.pennylane.com/en"
  },
  {
   "title": "Pennylane pricing (sole traders)",
   "url": "https://www.pennylane.com/fr/tarifs"
  },
  {
   "title": "Pennylane MCP",
   "url": "https://www.pennylane.com/fr/mcp"
  },
  {
   "title": "Pennylane AI features",
   "url": "https://www.pennylane.com/fr/intelligence-artificielle"
  },
  {
   "title": "Pennylane API overview",
   "url": "https://pennylane.readme.io/docs/api-overview"
  },
  {
   "title": "Pennylane API docs index",
   "url": "https://pennylane.readme.io/llms.txt"
  },
  {
   "title": "Rate limiting in API v2",
   "url": "https://pennylane.readme.io/docs/rate-limiting-1"
  },
  {
   "title": "Track data changes with the API",
   "url": "https://pennylane.readme.io/docs/tracking-data-changes-with-pennylane-api"
  },
  {
   "title": "Data Sharing or API",
   "url": "https://pennylane.readme.io/docs/should-i-use-data-sharing-or-the-api-1"
  },
  {
   "title": "Trustpilot: Pennylane",
   "url": "https://www.trustpilot.com/review/pennylane.com"
  },
  {
   "title": "Maddyness: Pennylane raises EUR 175M (January 2026)",
   "url": "https://www.maddyness.com/uk/?p=764590"
  },
  {
   "title": "CNBC via NBC San Diego: Pennylane doubles valuation (April 2025)",
   "url": "https://www.nbcsandiego.com/news/business/money-report/french-fintech-pennylane-doubles-valuation-to-2-2-billion-as-alphabets-venture-capital-arm-takes-stake/3796569"
  },
  {
   "title": "Bloomberg: Pennylane eyes funding at USD 4.3B",
   "url": "https://news.bloombergtax.com/daily-tax-report-state/sequoia-backed-pennylane-eyes-funding-at-4-3-billion-valuation"
  },
  {
   "title": "TechCrunch: Pennylane launch (May 2020)",
   "url": "https://techcrunch.com/2020/05/19/pennylane-is-an-accounting-service-that-improves-your-financial-visibility"
  },
  {
   "title": "TechCrunch: Pennylane raises USD 18.4M (January 2021)",
   "url": "https://techcrunch.com/2021/01/07/pennylane-raises-18-4-million-for-its-accounting-service"
  }
 ],
 "unverified": [
  "January 2026 round size: EUR 175M (Maddyness) versus USD 200M (Tech.eu); valuation EUR 3.5B from Maddyness only",
  "Customer count: homepage 1.5 million businesses (October 2026) versus 800,000 companies (January 2026); the measures may differ",
  "Launch date of the official MCP server",
  "Full co-founder list beyond Arthur Waller",
  "Details of the 2024 Billy acquisition beyond the Maddyness mention",
  "Prices of TPE, PME/ETI and accounting-firm plans",
  "Whether validated entries are locked against API updates",
  "G2 and Capterra ratings (pages blocked)",
  "Data Sharing pricing"
 ],
 "researched": "2026-10-10"
}
---

## What it is

Pennylane is a cloud accounting system that a company and its accounting firm use together. The company sends invoices, pays suppliers, files expense reports and banks through a [Pennylane business account](https://www.pennylane.com/en); the firm keeps the books, files VAT and prepares annual accounts in the same data. It started in France in 2020 and now sells in France, Germany and Spain. By its own count it serves 1.5 million businesses and more than 6,500 accounting firms; in January 2026 [Maddyness](https://www.maddyness.com/uk/?p=764590) reported 800,000 business customers, 6,000 firms and EUR 115M of annual recurring revenue for 2025.

## How it works

Banks, invoicing and other tools sync into one live ledger. Copilot reads invoices (OCR and Factur-X) and suggests entries, categories and reconciliations for a person to validate. Autopilot runs workflows you switch on, such as bank reconciliation, document requests and VAT checks, while your team handles exceptions; the [AI page](https://www.pennylane.com/fr/intelligence-artificielle) also announces a Studio IA for building your own agents. The homepage cites 84% of transactions reconciled automatically. E-invoicing is certified for France's September 2026 reform, which matters to every French company this year.

## Data and access

The [Company API v2](https://pennylane.readme.io/docs/api-overview) is public and well documented: ledger entries and lines, journals, invoices, suppliers, transactions, categories, with OAuth or developer tokens and per-resource scopes. The limit is [25 requests every 5 seconds](https://pennylane.readme.io/docs/rate-limiting-1) per token. You can export a FEC (the French standard full-ledger file) and the general ledger as xlsx through the API, and a separate [Data Sharing](https://pennylane.readme.io/docs/should-i-use-data-sharing-or-the-api-1) feed sends read-only data to BI tools and warehouses with a few hours of delay. [Changelog endpoints](https://pennylane.readme.io/docs/tracking-data-changes-with-pennylane-api) list inserts, updates and deletes per resource in order, but keep them for only four weeks, and ledger entries can be updated in place. API access is included from the Essentiel plan, EUR 24 HT a month for a sole trader as of 10 October 2026; Starter and Basique have none.

## Agents

Pennylane ships an [official MCP server](https://www.pennylane.com/fr/mcp) at mcp.pennylane.com, with OAuth through your own login and seven tool groups you can enable one by one, including a history of changes. It is read-only: an assistant cannot create, change or delete records or start payments. It works with Claude, ChatGPT, Mistral and others, is free for firms and needs Essentiel or above for companies. Agents that need to write must use the REST API with a write-scoped token, and there is no approval step on that path beyond scopes.

## Customers

On [Trustpilot](https://www.trustpilot.com/review/pennylane.com) Pennylane scores 4.3 from 1,034 reviews (fetched 10 October 2026), with 80% five-star and 12% one-star. People praise the support when they reach it and the shared view with their accountant; they complain about slow support, unreliable 2FA, bank connector failures and API changes. The accountant channel is the growth engine: firms bring their clients.

## History

Pennylane began taking customers in March 2020 with a [EUR 4M seed](https://techcrunch.com/2020/05/19/pennylane-is-an-accounting-service-that-improves-your-financial-visibility), co-founded by Arthur Waller. It raised EUR 40M in February 2024 at over EUR 1B, [EUR 75M led by Sequoia](https://www.nbcsandiego.com/news/business/money-report/french-fintech-pennylane-doubles-valuation-to-2-2-billion-as-alphabets-venture-capital-arm-takes-stake/3796569) in April 2025 at EUR 2B, and EUR 175M led by TCV in January 2026 at EUR 3.5B. It is the largest European challenger to Sage, Cegid and the local incumbents, and its CEO has called a US launch premature.

## For ownpurse

Pennylane is a source, not a rival. It does the European small-business ledger well, and its API is one of the better ones to build a connector on: scoped tokens, a FEC that captures the full ledger, and change events in order. The gap is history. Pennylane forgets its change log after four weeks and lets entries be edited; an ownpurse connector that polls the changelog and appends every version keeps what Pennylane drops. Its read-only MCP is the same instinct as ownpurse's free reads; ownpurse adds the missing half, writes that go out only as sealed packages a person approves.
