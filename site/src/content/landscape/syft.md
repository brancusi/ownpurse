---
{
 "slug": "syft",
 "name": "Syft Analytics",
 "url": "https://www.syftanalytics.com",
 "segment": "close-analytics",
 "status": "acquired",
 "founded": "2016",
 "hq": "Johannesburg, South Africa (contracting entity Syft Analytics Inc., terms governed by California law)",
 "backing": "Xero (ASX: XRO) agreed on 17 September 2024 to buy it for up to US$70m: US$40m upfront (including about US$10m in Xero shares), the rest in earnouts and employee RSUs over three years. It now trades as 'Syft from Xero'. Earlier funding was not found.",
 "tagline": "A cloud reporting, consolidation and forecasting tool over synced small-business ledgers and commerce data, now owned by Xero and partly built into Xero itself.",
 "scope": "Syft pulls accounts, contacts, invoices, journals and bank transactions from Xero, QuickBooks, Sage, MYOB, FreshBooks, FreeAgent and ERPs, plus Stripe, Shopify, Square, Gusto and US and Canadian bank feeds, and refreshes once a day. On that copy it builds dashboards, financial statements, consolidations, forecasts, KPIs, benchmarks, valuations and data-quality reviews, with AI summaries. It is read-only toward the ledger, does not post entries, and publishes no public API.",
 "domains": [
  "reporting",
  "ledger",
  "close",
  "commerce",
  "payments",
  "bank"
 ],
 "customers_served": [
  "accountants",
  "smb",
  "mid-market"
 ],
 "spectrum": {
  "custody": {
   "score": 1,
   "note": "The ledger stays authoritative, but Syft's transaction-level copy and all report, KPI and forecast work live in Syft's cloud; after you cancel, data is archived and no longer available to you."
  },
  "locality": {
   "score": 0,
   "note": "Cloud only; nothing self-hosted or local."
  },
  "history": {
   "score": 1,
   "note": "Holds transaction-level history for drilldown, refreshed in place daily; no versioned record or as-of view of what the source said."
  },
  "openness": {
   "score": 1,
   "note": "Closed, with no public API (its footer 'API' link returns a 404). It exports to PDF, Excel and Word, and Spreadsheet Link feeds Excel and Google Sheets."
  },
  "breadth": {
   "score": 2,
   "note": "Ledgers, ERPs, e-commerce, payments, payroll, bank feeds and spreadsheets in one reporting layer; no documents and no writes."
  },
  "autonomy": {
   "score": 1,
   "note": "Daily refresh, scheduled reports and AI insights run unattended; nothing is written to the books."
  },
  "agent_readiness": {
   "score": 0,
   "note": "No public API, CLI or MCP found; its AI is an in-app assistant. Xero's separate MCP server does not expose Syft."
  },
  "integration_ease": {
   "score": 1,
   "note": "Owners connect about 20 sources in a few clicks; third parties and agents have no documented way in."
  },
  "market": {
   "score": 2,
   "note": "Accountants, bookkeepers and SMBs, up to mid-market groups (for example Atari's consolidation, BDO)."
  }
 },
 "api": {
  "public": false,
  "style": "none",
  "auth": "n/a. Users sign in with Google, Microsoft, Xero or Intuit; Syft connects to sources by OAuth.",
  "limits": "n/a. Upstream refresh is daily for Xero and most sources, every 48 hours for Sage.",
  "cost": "n/a; no API product",
  "export": "Reports export to PDF, Excel and Word (with optional AI text). Spreadsheet Link pushes report data to Excel and Google Sheets. No bulk raw-data export documented. On cancellation the subscription is archived and the data is no longer available to you, though it can be reactivated during the retention period.",
  "notes": "The site footer links to /api, which returned 'Page not found' on 2026-10-10; api.syftanalytics.com is the app's sign-in page, not a developer API."
 },
 "pricing": {
  "model": "Tiered monthly subscription per connected entity; plus fixed 'Packages' of 10, 25, 50 or 100+ connections for firms; free tier",
  "entry": "Basic: free for ever (unlimited users, dashboards and graphs). Standard: US$23 a month",
  "top": "Advanced: US$95 a month; above 10 entities, a quote. Plus is US$47 a month",
  "notes": "From syftanalytics.com/pricing-page, read 2026-10-10. 14-day trial, no card. Xero partners get Syft at no extra cost on qualifying Xero business plans, matched to the Xero plan tier. Per-entity billing is inferred from 'More than 10 entities? Contact us' and the package terms, not stated outright."
 },
 "agents": {
  "mcp": "none",
  "ai_features": "Syft Assist AI: insights, email summaries, improvement suggestions, voice questions and narrated summaries in 56 languages, PDF or Word with AI text, on any report or visualisation. In Xero Analytics (2026): an AI commentary widget on dashboards and 'Explain this to me' on industry benchmarks.",
  "agent_writes": "No. There is no external interface, and Syft never writes to the ledger.",
  "approval": "n/a. A person copies or exports the AI text.",
  "notes": "Its AI reads Syft's own copy, inside Syft's UI. Agents reach Xero itself through Xero's own API and MCP server, not through Syft."
 },
 "integration": {
  "scope": "About 20 sources: Xero, QuickBooks Online and Desktop, Sage, Sage Intacct (premium), MYOB, FreshBooks, FreeAgent, Acumatica, Excel, Google Sheets, Square, Stripe, Stripe Connect (premium), Shopify, Gusto, Xero Payroll, US and Canadian bank feeds, trial balance and transaction-list uploads",
  "notes": "From Xero it reads accounts, contacts, invoices, journals and bank transactions, so you can drill down to the transaction. Refresh is daily, and a stale connection needs re-authentication."
 },
 "preference": {
  "scale": "50,000+ SMBs at acquisition (September 2024, Accounting Times). '200,000+ businesses' on its Xero App Store listing (undated, read 2026-10-10). About 70 staff in 2024. Xero calls it the most used reporting and insights app in its store.",
  "ratings": "Xero App Store 4.81/5 from 476 reviews (read 2026-10-10). G2 4.8/5 from 76 reviews (read 2026-10-10). Pricing page claims '2,000+ five star reviews' and 'App of the Year 2024'.",
  "sentiment": "Praised for support (human live chat, Syft Campus), ease of use, attractive reports and smooth ledger sync. Complaints: less custom than Power BI or Tableau, a learning curve, some Excel-integration and connectivity issues, budget-sync trouble, and cost for some users."
 },
 "timeline": [
  {
   "date": "2016",
   "event": "Founded in Johannesburg by Vangelis Kyriazis, Eleftherios Kyriazis, Duran Hamer and Matt Stephanou"
  },
  {
   "date": "2024",
   "event": "Claims Xero 'App of the Year 2024'"
  },
  {
   "date": "2024-09-17",
   "event": "Xero announces it will buy Syft for up to US$70m; 50,000+ SMB users, about 70 staff; Syft to stay standalone and be built into Xero over time"
  },
  {
   "date": "2025-06-18",
   "event": "'Analytics powered by Syft' appears inside Xero in early release, for some plans and regions"
  },
  {
   "date": "2025-07-09",
   "event": "Syft's terms of use are rewritten to match Xero's; legal notices go to Xero"
  },
  {
   "date": "2026-09-10",
   "event": "Xero Analytics adds new Budgets (beta), an AI commentary widget and AI benchmark explanations, built by the Syft team"
  }
 ],
 "relation": "neighbour",
 "relation_note": "Syft is the strongest reporting layer over a synced copy of the books, and Xero bought it. The ledger vendor now owns the copy and builds its views into the ledger, which narrows the room for a hosted third-party reporting tool. It leaves open the case ownpurse makes: a record the owner holds that spans every vendor and that agents can read.",
 "borrow": "Breadth beyond the ledger (commerce, payments, payroll, bank feeds) in one reporting surface; a data-quality 'review' (bank, accounts, contacts, VAT/GST) before reporting; transaction-level drilldown from every figure; a free tier; consolidations without extra charges.",
 "avoid": "A daily overwrite of a vendor-held copy; data that becomes unavailable when you cancel; an advertised API that does not exist; analysis that only lives in the vendor's UI, which an agent cannot reach.",
 "sources": [
  {
   "title": "Syft pricing",
   "url": "https://www.syftanalytics.com/pricing-page"
  },
  {
   "title": "Syft integrations",
   "url": "https://www.syftanalytics.com/integrations"
  },
  {
   "title": "Syft terms of use (updated 9 July 2025)",
   "url": "https://www.syftanalytics.com/terms-of-use"
  },
  {
   "title": "Syft help: Xero connection",
   "url": "https://help.syftanalytics.com/en/articles/9025863-xero"
  },
  {
   "title": "Syft help: refreshes",
   "url": "https://help.syftanalytics.com/en/articles/10516866-refreshes-fyi"
  },
  {
   "title": "Syft help: Syft Assist AI",
   "url": "https://help.syftanalytics.com/en/articles/9360648-syft-assist-ai"
  },
  {
   "title": "Syft help: Analytics powered by Syft vs Syft Analytics (18 June 2025)",
   "url": "https://help.syftanalytics.com/en/articles/11602802-analytics-powered-by-syft-vs-syft-analytics"
  },
  {
   "title": "Syft help: What's new on Xero Analytics (10 September 2026)",
   "url": "https://help.syftanalytics.com/en/articles/11205090-what-s-new-on-xero-analytics"
  },
  {
   "title": "Syft from Xero (Xero partner offer)",
   "url": "https://www.xero.com/us/syft-from-xero"
  },
  {
   "title": "Xero to acquire Syft (Xero media release)",
   "url": "https://www.xero.com/media-releases/xero-to-acquire-syft-enhance-reporting-insights-capability"
  },
  {
   "title": "Accounting Times: Xero to buy Syft for US$70m (18 September 2024)",
   "url": "https://www.accountingtimes.com.au/technology/xero-to-buy-reporting-app-syft-analytics-for-us-70m"
  },
  {
   "title": "TechCabal: Xero to acquire Syft (17 September 2024)",
   "url": "https://techcabal.com/2024/09/17/xero-syft-acquisition/"
  },
  {
   "title": "Syft on the Xero App Store",
   "url": "https://apps.xero.com/au/app/syft"
  },
  {
   "title": "Syft on G2",
   "url": "https://www.g2.com/products/syft-analytics/reviews"
  }
 ],
 "unverified": [
  "The date the acquisition closed: completion was expected in Q3 of Xero's FY25 (roughly early 2025), and the 'Syft from Xero' branding and Xero legal notices imply it closed, but no completion notice was read.",
  "Pre-acquisition funding: none found; the search budget ran out before checking.",
  "Whether the US$23/47/95 prices are per entity: implied by 'More than 10 entities? Contact us' and the package terms, not stated outright. Aggregators list other figures (US$19, US$29/99/199) that conflict.",
  "'200,000+ businesses' (Xero App Store listing) is undated and conflicts with 50,000+ at acquisition; it may count entities connected, not paying businesses.",
  "Whether a partner or private API exists behind the dead /api footer link.",
  "No MCP server or agent interface was found, but the search budget ran out before an exhaustive check.",
  "Capterra and Trustpilot ratings were not checked."
 ],
 "researched": "2026-10-10"
}
---

## What it is

Syft Analytics is a cloud reporting, consolidation and forecasting tool for accountants, bookkeepers and small and medium businesses. It was founded in Johannesburg in 2016. On 17 September 2024 [Xero agreed to buy it for up to US$70 million](https://www.accountingtimes.com.au/technology/xero-to-buy-reporting-app-syft-analytics-for-us-70m): US$40 million upfront, about US$10 million of that in Xero shares, and the rest in earnouts and employee stock over three years. At the time it had more than 50,000 small-business users and about 70 staff. Its logo now reads "Syft from Xero", and its [terms](https://www.syftanalytics.com/terms-of-use), updated in July 2025, were rewritten to match Xero's.

## How it works

You connect a source and Syft keeps a copy. From Xero it takes [accounts, contacts, invoices, journals and bank transactions](https://help.syftanalytics.com/en/articles/9025863-xero), so you can drill from a figure to the transactions behind it. Beyond ledgers it connects Stripe, Shopify, Square, Gusto, Xero Payroll, Sage Intacct, Acumatica and US and Canadian bank feeds, about twenty sources on its [integrations page](https://www.syftanalytics.com/integrations). Data [refreshes once a day](https://help.syftanalytics.com/en/articles/10516866-refreshes-fyi), every 48 hours for Sage. On that copy you build dashboards, IFRS and GAAP statements, consolidations with eliminations and currency handling, driver-based forecasts, KPIs, scorecards, valuations and industry benchmarks. A "Clarify" review checks bank, accounts, contacts and VAT or GST data before you report.

## Data and access

Syft has no public API. Its site footer links to `/api`, which returned "Page not found" on 10 October 2026. Reports export to PDF, Excel and Word, and Spreadsheet Link pushes report data into Excel and Google Sheets. Under the terms, once a subscription ends the data "is no longer available to you" and is kept only for the retention period.

[Pricing](https://www.syftanalytics.com/pricing-page) (read 10 October 2026): a free Basic plan, then Standard at US$23, Plus at US$47 and Advanced at US$95 a month, with a quote above ten entities. Firms buy fixed packages of 10, 25, 50 or 100+ connections, with limits on how often a slot can be swapped. Xero partners get Syft [at no extra cost on qualifying Xero plans](https://www.xero.com/us/syft-from-xero).

## Agents

[Syft Assist AI](https://help.syftanalytics.com/en/articles/9360648-syft-assist-ai) writes insights, email summaries and suggestions for any report, and can answer spoken questions and narrate in 56 languages. Inside Xero, the Syft team now ships an [AI commentary widget](https://help.syftanalytics.com/en/articles/11205090-what-s-new-on-xero-analytics) and AI explanations of benchmarks. All of it runs inside the product. An outside agent cannot read Syft's copy, and Syft never writes to the books.

## Customers

Syft is well liked: 4.81 out of 5 from 476 reviews on the [Xero App Store](https://apps.xero.com/au/app/syft), and 4.8 from 76 on [G2](https://www.g2.com/products/syft-analytics/reviews). Reviewers praise the support, how easy it is to use and how well it syncs. They complain that it is less flexible than Power BI or Tableau, has a learning curve, has occasional Excel and connectivity problems, and costs too much for some. Customers range from small firms to BDO and Atari's group consolidation.

## History

Syft grew as a Xero app partner and became the most used reporting app in Xero's store. After the acquisition Xero kept it as a standalone product and began building it in. In June 2025, ["Analytics powered by Syft"](https://help.syftanalytics.com/en/articles/11602802-analytics-powered-by-syft-vs-syft-analytics) arrived inside Xero in early release, and by September 2026 Xero Analytics had new Budgets and AI commentary built by the Syft team. This is the familiar arc: an independent tool that kept a copy of the ledger becomes a feature of the ledger.

## For ownpurse

Syft shows the most a hosted copy can do: wide coverage of sources, review before reporting, drilldown to the transaction, a free tier and well-liked support. Its sale also shows where that copy ends up, owned by the ledger vendor and serving that vendor's customers. What it does not offer, and what Xero has no reason to build, is a record the owner holds that spans QuickBooks, Stripe and the bank as well as Xero, keeps every version, and can be read by an agent. ownpurse should borrow the review step and the drilldown, and stay on the side of the owner.
