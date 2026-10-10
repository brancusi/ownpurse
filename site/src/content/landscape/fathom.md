---
{
 "slug": "fathom",
 "name": "Fathom",
 "url": "https://www.fathomhq.com",
 "segment": "close-analytics",
 "status": "acquired",
 "founded": "2010 per PitchBook and CB Insights; The Access Group says the product launched in 2012",
 "hq": "Brisbane, Australia (offices in Seattle and Loughborough, UK)",
 "backing": "Acquired by The Access Group (UK, private-equity-backed) on 24 August 2022, terms not disclosed. No earlier venture rounds found in public sources.",
 "tagline": "A cloud reporting, forecasting and consolidation tool that imports summary financials from small-business ledgers for advisers and owners.",
 "scope": "Fathom imports monthly summary financials (Xero's standard P&L and balance sheet, and the equivalents from QuickBooks, MYOB, Sage, FreeAgent, Excel and Google Sheets) once a day, then builds management reports, KPIs, three-way cash flow forecasts, consolidations and benchmarks. It is read-only toward the ledger and never writes back to the books. It does not hold transactions, documents or bank data, and it offers no public API.",
 "domains": [
  "reporting",
  "ledger",
  "close"
 ],
 "customers_served": [
  "accountants",
  "smb",
  "mid-market"
 ],
 "spectrum": {
  "custody": {
   "score": 1,
   "note": "The source ledger stays authoritative, but Fathom's derived copy and every customisation (KPIs, mappings, forecasts) live only in Fathom's Azure tenancy and cannot be moved between accounts."
  },
  "locality": {
   "score": 0,
   "note": "Cloud only on Microsoft Azure. A desktop connector exists only to upload QuickBooks Desktop or MYOB Desktop data."
  },
  "history": {
   "score": 1,
   "note": "It keeps monthly period balances over many years (36 months on Portfolio), refreshed in place each day. It keeps no versioned record of what the source said when."
  },
  "openness": {
   "score": 1,
   "note": "Closed product with no public API. Reports, tables, budgets and forecasts export to Excel, PDF and Word."
  },
  "breadth": {
   "score": 1,
   "note": "Ledger summaries plus non-financial KPIs from spreadsheets. No banks, payments, documents or payroll as sources."
  },
  "autonomy": {
   "score": 1,
   "note": "Sync, scheduled reports and AI commentary run unattended, but nothing is written to the books."
  },
  "agent_readiness": {
   "score": 0,
   "note": "No public API, CLI or MCP found. AI is a built-in commentary writer inside reports."
  },
  "integration_ease": {
   "score": 1,
   "note": "Connecting a ledger takes a few clicks for an owner. A third party or agent has no programmatic way in."
  },
  "market": {
   "score": 2,
   "note": "Accounting firms and small to medium businesses, with consolidation and benchmarking for franchises and groups."
  }
 },
 "api": {
  "public": false,
  "style": "none",
  "auth": "n/a (Fathom itself authenticates to ledgers by OAuth)",
  "limits": "n/a. Upstream, Xero companies must be opened in Fathom or have a scheduled report within 90 days or automatic updates stop when the token lapses.",
  "cost": "n/a; no API product",
  "export": "Per-report and per-table export to Excel, PDF and Word; budgets, forecasts and non-financial KPI results download to Excel. No bulk account export: moving a company to another Fathom account means rebuilding it, and Fathom's own help says to screenshot custom KPIs and the chart of accounts first.",
  "notes": "Not to be confused with Fathom (fathom.ai), the meeting recorder that does have a public API."
 },
 "pricing": {
  "model": "Per-company monthly subscription, billed monthly with no minimum term; prices exclude sales tax; seven currencies",
  "entry": "Fathom Portfolio: US$67 a month for 100 companies (about US$0.67 a company), unlimited users, 36 months of history, summary reports only",
  "top": "Enterprise: custom quote for 51+ companies. Fathom Pro: US$450 a month for 25 companies, extra companies US$17 each",
  "notes": "From fathomhq.com/pricing, read 2026-10-10. Consolidated groups are free and unlimited. A 14-day free trial needs no card. G2 still shows an older 'Starter $59' plan."
 },
 "agents": {
  "mcp": "none",
  "ai_features": "Commentary Writer (2026): AI commentary on up to six selected report components, shaped by business and report context, with prompts and freeform chat, up to 50 saved prompts and 'symbolic attribution' that traces every figure to its source.",
  "agent_writes": "No. There is no external interface an agent can drive, and Fathom never writes to the ledger.",
  "approval": "n/a. A person inserts AI commentary into a report and edits it before sharing.",
  "notes": "Traceable numbers in AI commentary is the part worth noting: Fathom chose component-scoped inputs over whole-report generation to cut hallucination."
 },
 "integration": {
  "scope": "Xero, QuickBooks Online and Desktop, MYOB (cloud and desktop AccountRight), Sage Business Cloud Accounting (UK), FreeAgent, Access Financials (UK), Excel and Google Sheets: about nine sources",
  "notes": "Imports summary financials (standard P&L and balance sheet), not transactions; cloud sources sync once a day, usually before 8 am AEST. Divisions come from tracking categories."
 },
 "preference": {
  "scale": "7,500 customers in 81 countries (Access Group, August 2022). 99,000 companies (Xero App Store listing, undated, read 2026-10-10). 110,000+ companies or organisations (fathomhq.com, read 2026-10-10). 'Companies' counts ledgers, not paying accounts.",
  "ratings": "Xero App Store 4.95/5 from 619 reviews, 96% five-star (read 2026-10-10). G2 4.6/5 from 146 reviews (read 2026-10-10).",
  "sentiment": "Praised for ease of use, flexible report building, forecasting and responsive support. Complaints on G2 are about missing features, import and upload issues, customisation limits, and the once-a-day sync, which pushes some users to export from the ledger by hand for point-in-time figures."
 },
 "timeline": [
  {
   "date": "2010",
   "event": "Founded in Brisbane by David Watson, Daniel Walls and Geoff Cook (PitchBook, CB Insights)"
  },
  {
   "date": "2012",
   "event": "Product launched, per The Access Group's announcement"
  },
  {
   "date": "2022-08-24",
   "event": "Acquired by The Access Group; 7,500 customers in 81 countries at the time"
  },
  {
   "date": "2026-03-19",
   "event": "Fathom publishes the build story of Commentary Writer, its context-aware AI commentary"
  },
  {
   "date": "2026-10",
   "event": "Site claims 110,000+ companies; pricing at US$450 a month for Pro (25 companies) and US$67 a month for Portfolio (100 companies)"
  }
 ],
 "relation": "neighbour",
 "relation_note": "Fathom does the job a person wants from ownpurse's views (management reports, KPIs, consolidations) on a once-a-day summary copy of the ledger that Fathom holds. It is read-only like ownpurse's default, but the copy, the mappings and the history belong to the vendor, and agents cannot reach it.",
 "borrow": "Traceable AI commentary (every figure in the prose links back to its source number); component-scoped context rather than whole-report prompts; free unlimited consolidations; reports that schedule and send themselves.",
 "avoid": "Customisations that cannot leave the account; summary-level imports that lose the transactions behind a number; a daily overwrite with no record of what changed; no API, so neither the owner nor an agent can build on the data.",
 "sources": [
  {
   "title": "Fathom pricing",
   "url": "https://www.fathomhq.com/pricing"
  },
  {
   "title": "Fathom integrations",
   "url": "https://www.fathomhq.com/integrations"
  },
  {
   "title": "Fathom for Xero",
   "url": "https://www.fathomhq.com/integrations/xero"
  },
  {
   "title": "Fathom security",
   "url": "https://www.fathomhq.com/security"
  },
  {
   "title": "Fathom about",
   "url": "https://www.fathomhq.com/about"
  },
  {
   "title": "Commentary Writer",
   "url": "https://www.fathomhq.com/features/evo/commentary-writer"
  },
  {
   "title": "Behind the Build: Fathom Commentary Writer (19 March 2026)",
   "url": "https://www.fathomhq.com/blog/behind-the-build-fathom-commentary-writer"
  },
  {
   "title": "Fathom help: automatic updates",
   "url": "https://support.fathomhq.com/en/articles/2387323-automatic-updates"
  },
  {
   "title": "Fathom help: why numbers do not match the source",
   "url": "https://support.fathomhq.com/en/articles/2680204-why-do-the-numbers-in-fathom-not-match-my-source-accounting-system"
  },
  {
   "title": "Fathom help: moving a company to another account",
   "url": "https://support.fathomhq.com/en/articles/1858976-can-i-transfer-data-to-a-new-fathom-account"
  },
  {
   "title": "The Access Group acquires Fathom (24 August 2022)",
   "url": "https://www.theaccessgroup.com/en-gb/about/news/the-access-group-acquires-fathom"
  },
  {
   "title": "Fathom on the Xero App Store",
   "url": "https://apps.xero.com/au/app/fathom"
  },
  {
   "title": "Fathom on G2",
   "url": "https://www.g2.com/products/fathom/reviews"
  },
  {
   "title": "PitchBook profile",
   "url": "https://pitchbook.com/profiles/company/87878-89"
  }
 ],
 "unverified": [
  "Founding year: 2010 (PitchBook, CB Insights) against a 2012 launch (Access Group); the exact incorporation date was not confirmed.",
  "Acquisition price and any pre-2022 funding: not disclosed, and not found.",
  "Current paying-customer count: only company/organisation counts (99,000 on the Xero App Store listing, 110,000+ on the site) were found, both undated.",
  "Whether Commentary Writer is included in Portfolio or only in Pro is not clear from the pricing page.",
  "No MCP server, Zapier app or partner API was found, but the web search budget ran out before an exhaustive check.",
  "Capterra and Trustpilot ratings were not checked."
 ],
 "researched": "2026-10-10"
}
---

## What it is

Fathom is a cloud reporting, forecasting and consolidation tool for small and medium businesses and the accountants who advise them. It started in Brisbane, its founders came from accounting software, and since [August 2022 it has been owned by The Access Group](https://www.theaccessgroup.com/en-gb/about/news/the-access-group-acquires-fathom). It reads a ledger and does not replace it. Fathom never writes back to the books.

## How it works

You connect Xero, QuickBooks, MYOB, Sage, FreeAgent or a spreadsheet, and Fathom imports *summary* financials: its own help says that for Xero it takes [the standard P&L and balance sheet](https://support.fathomhq.com/en/articles/2680204-why-do-the-numbers-in-fathom-not-match-my-source-accounting-system), and its [integrations page](https://www.fathomhq.com/integrations) describes the import as "summary financial data". With [automatic updates](https://support.fathomhq.com/en/articles/2387323-automatic-updates) on, cloud sources refresh once every 24 hours, usually before 8 am AEST. Each Xero company must be opened, or have a scheduled report, within 90 days, or the token lapses and updates stop.

On that copy you remap the chart of accounts, set financial and non-financial KPIs, build reports from text, charts and statements, schedule them to go out, run three-way cash flow forecasts up to five years ahead, and consolidate groups with eliminations and multi-currency. Benchmarking ranks companies against each other, which suits franchises. The new [Commentary Writer](https://www.fathomhq.com/features/evo/commentary-writer) writes AI commentary on up to six report components at a time, shaped by business and report context, and shows where each figure comes from. Its [build story](https://www.fathomhq.com/blog/behind-the-build-fathom-commentary-writer) (March 2026) says the team dropped whole-report generation because too much input made the output worse.

## Data and access

Fathom offers no public API. Reports, tables, budgets and forecasts export to Excel, PDF and Word. There is no bulk export, and you [cannot move a company from one Fathom account to another](https://support.fathomhq.com/en/articles/1858976-can-i-transfer-data-to-a-new-fathom-account): you rebuild it, and the help article tells you to screenshot your custom KPIs and chart of accounts first, because deleting the company permanently deletes every customisation. The [security page](https://www.fathomhq.com/security) calls Fathom a "caretaker" of your data, hosted on Azure and backed up every six hours.

[Pricing](https://www.fathomhq.com/pricing) (read 10 October 2026) is per company: Fathom Pro costs US$450 a month for 25 companies, with extra companies at US$17 each. Fathom Portfolio costs US$67 a month for 100 companies and gives summary dashboards only. Enterprise is quoted for 51 or more companies. Consolidated groups are free.

## Agents

An outside agent cannot drive Fathom: there is no API, CLI or MCP server, and no integration platform listing was found. Fathom's AI lives inside the product, as a writer of prose about numbers. The idea worth taking is its traceability: every number in the commentary can be checked against its source before you share it.

## Customers

Fathom is popular with advisers. It has 4.95 out of 5 from 619 reviews on the [Xero App Store](https://apps.xero.com/au/app/fathom) and 4.6 from 146 on [G2](https://www.g2.com/products/fathom/reviews). Reviewers praise how easy it is to use, its report building, its forecasting and its support. They complain about missing features, import trouble, limits on customisation and the daily sync, which sends some back to the ledger for figures as of the reporting date. Scale figures grew from 7,500 customers in 2022 to "110,000+ companies" on the site today. The two count different things, accounts against ledgers.

## History

The company was founded around 2010 by David Watson, Daniel Walls and Geoff Cook, and the product launched in 2012. The founders still lead it inside Access, which bought it alongside Reckon's accountant business to build out its Australian and New Zealand practice suite. Fathom is a long-running, mature member of the "reporting over a synced copy" pattern that grew up around Xero's app marketplace.

## For ownpurse

Fathom shows what advisers will pay for: about US$18 a company a month for reports that look good, forecasts and consolidations. It also shows the cost of the pattern. The copy is a summary, refreshed by overwrite, held by the vendor, and every hour of mapping work is locked in the account. ownpurse can give the same views over a record the owner holds, down to the transaction, with every version kept and an API an agent can read. It should also take Fathom's discipline of AI prose where every figure traces to a fact.
