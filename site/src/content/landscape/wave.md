---
{
 "slug": "wave",
 "name": "Wave",
 "url": "https://www.waveapps.com",
 "segment": "incumbent-ledger",
 "status": "acquired",
 "founded": "2009",
 "hq": "Toronto, Canada",
 "backing": "Venture-backed (OMERS Ventures, Charles River Ventures, Social Capital, NAB Ventures; $24M round in 2017) until H&R Block (NYSE: HRB) agreed to buy it, announced 11 June 2019, for a reported US$405M.",
 "tagline": "Free invoicing and accounting for very small businesses in the US and Canada, owned by H&R Block.",
 "scope": "Wave offers free invoicing, estimates, bills and bookkeeping records on its Starter plan, with paid card and bank payments, payroll, receipts and bookkeeping services. Bank auto-import, auto-categorisation and receipts are on the $19-a-month Pro plan. It is aimed at solopreneurs and microbusinesses in the US and Canada; it is not built for growing teams, multiple entities or deep reporting, and its API exposes little of the ledger.",
 "domains": [
  "ledger",
  "invoicing",
  "payments",
  "payroll",
  "bank",
  "documents"
 ],
 "customers_served": [
  "freelancer",
  "smb",
  "consumer"
 ],
 "spectrum": {
  "custody": {
   "score": 1,
   "note": "Wave's cloud holds the record; you can export reports and transactions by hand as CSV, but not read transactions through the API."
  },
  "locality": {
   "score": 0,
   "note": "Cloud only."
  },
  "history": {
   "score": 1,
   "note": "Accounts carry a revision sequence number, but no audit history or versions are exposed."
  },
  "openness": {
   "score": 1,
   "note": "Free GraphQL API, but third-party OAuth requires the business to be on Pro, and transactions and reports cannot be read through it."
  },
  "breadth": {
   "score": 2,
   "note": "Invoicing, payments, payroll, receipts, bank import and a ledger in one product for a microbusiness."
  },
  "autonomy": {
   "score": 1,
   "note": "Rules-based auto-merge and categorisation of imported bank transactions on Pro; no agent features."
  },
  "agent_readiness": {
   "score": 1,
   "note": "A public API exists, but no official MCP, no sandbox, and an agent cannot read the ledger's transactions or reports."
  },
  "integration_ease": {
   "score": 1,
   "note": "Self-serve registration, but no sandbox, a Pro requirement for OAuth, a thin schema and terms that let Wave discontinue the API at any time."
  },
  "market": {
   "score": 1,
   "note": "Solopreneurs and microbusinesses at the free and low-price end."
  }
 },
 "api": {
  "public": true,
  "style": "GraphQL (gql.waveapps.com/graphql/public). The older REST API is no longer the documented path.",
  "auth": "OAuth 2.0 authorization code for third-party apps; a full access token from the developer portal for your own business.",
  "limits": "Not published by Wave in anything retrieved. One third-party guide claims 60 requests a minute per token and 5,000 a day per app, without a source.",
  "cost": "No API fee found. Since 26 May 2025 (per a third-party guide), a connected business needs Pro ($19 a month) or Wave Advisors for third-party OAuth access.",
  "export": "API reads cover businesses, customers, invoices, estimates, products, vendors, sales taxes and accounts with current balances. No query for transactions, journal entries or financial reports. In the app, reports and transactions export as CSV.",
  "notes": "The public schema (introspected May 2026) has write-only beta mutations moneyTransactionCreate and moneyTransactionsCreate, so you can post transactions but not read them back. Imported bank transactions cannot be listed or matched through the API. Payroll has no API surface. Wave's terms let it suspend or discontinue API access at any time."
 },
 "pricing": {
  "model": "Freemium: a free Starter plan, a paid Pro plan per business, and per-transaction payment fees and add-ons.",
  "entry": "Starter: $0, with unlimited invoices, estimates, bills and bookkeeping records; card payments 2.9% + $0.60 (Amex 3.4% + $0.60).",
  "top": "Wave Advisors bookkeeping from $149 or $199 a month; Pro is $19 USD or $25 CAD a month, $190 USD a year.",
  "notes": "Pro adds bank auto-import through Plaid, auto-categorisation, unlimited receipts, late payment reminders and the first 10 card transactions a month without the $0.60 fee. Payroll from $25 or $40 a month depending on plan. Prices from the pricing page, October 2026."
 },
 "agents": {
  "mcp": "community",
  "ai_features": "None advertised on Wave's site as of October 2026. Pro offers rules-based auto-merge and categorisation of bank transactions.",
  "agent_writes": "Through the GraphQL API: create invoices, customers and products, record invoice payments, and post money transactions (beta). Community MCP servers wrap these.",
  "approval": "No approval gate. A token with access can write directly.",
  "notes": "No official MCP server and no Claude connector as of 10 October 2026. Community servers state that transaction reads and financial reports are not exposed because the public API does not provide them; one drives Wave by saving the user's password, which shows how thin the API is."
 },
 "integration": {
  "scope": "About 15 direct integrations (Amazon, Etsy, eBay, Google Sheets, Capsule CRM, GoHighLevel, Driversnote, Fundbox and others) plus Make.com for further apps; Plaid for bank import; Check for US payroll since May 2025.",
  "notes": "Wave's own integrations page counts 13 to 15 direct apps in different places. Zapier is not listed."
 },
 "preference": {
  "scale": "Wave says it has helped more than 5.9 million small business owners in the US and Canada, with over 1.5B accounting transactions since 2011 (about page, October 2026). Its home page cites 'over 350,000' and 'over 300,000' small businesses in different headings.",
  "ratings": "Trustpilot: 3.2 from 555 reviews (395 in the last 12 months), October 2026, with at least one review about an unrelated company. G2 and Capterra figures could not be retrieved.",
  "sentiment": "Praised for being free or cheap, simple invoicing, receipt scanning and easy sharing with an accountant. Complaints: support (slow email, chatbots, no phone), lockouts, held or returned payments, billing after archiving a business, recurring invoices failing, and few integrations."
 },
 "timeline": [
  {
   "date": "2009-07",
   "event": "Wave Accounting founded in Toronto by Kirk Simpson and James Lochrie."
  },
  {
   "date": "2010-11-16",
   "event": "Public launch of free accounting."
  },
  {
   "date": "2011",
   "event": "Series A led by OMERS Ventures; $5M led by Charles River Ventures."
  },
  {
   "date": "2012-05",
   "event": "Series B led by Social+Capital Partnership; rebrand to Wave."
  },
  {
   "date": "2017-01",
   "event": "Advertising removed from the product."
  },
  {
   "date": "2017",
   "event": "US$24M round led by NAB Ventures; Lending by Wave launched (later discontinued)."
  },
  {
   "date": "2019-06-11",
   "event": "H&R Block announces it will acquire Wave, reported at US$405M."
  },
  {
   "date": "2022-06-15",
   "event": "Zahir Khoja replaces Kirk Simpson as CEO."
  },
  {
   "date": "2024",
   "event": "Free Starter plan and paid Pro plan in place; bank auto-import moves to Pro."
  },
  {
   "date": "2025-05",
   "event": "US payroll moved to a system run on Check; third-party API access limited to Pro and Advisors businesses (26 May 2025, per a third-party guide)."
  }
 ],
 "relation": "source",
 "relation_note": "Wave is the low-price end of the market and a possible source for very small businesses, but its API cannot read transactions or reports. A Wave connector for ownpurse would have to start from Wave's CSV exports rather than the API.",
 "borrow": "A free tier that covers a real ledger for a one-person business, and a revision sequence number on each account that shows when it changed.",
 "avoid": "An API that lets you write transactions you cannot read back, access that depends on a paid plan of the business being read, and terms that let the API be withdrawn at any time.",
 "sources": [
  {
   "title": "Wave pricing",
   "url": "https://www.waveapps.com/pricing"
  },
  {
   "title": "Wave about us",
   "url": "https://www.waveapps.com/about-us"
  },
  {
   "title": "Wave home page",
   "url": "https://www.waveapps.com/"
  },
  {
   "title": "Wave integrations",
   "url": "https://www.waveapps.com/integrations"
  },
  {
   "title": "Wikipedia: Wave Financial",
   "url": "https://en.wikipedia.org/wiki/Wave_Financial"
  },
  {
   "title": "mcp-wave: community MCP server and checked-in Wave GraphQL schema (May 2026)",
   "url": "https://github.com/rhanka/mcp-wave"
  },
  {
   "title": "waveapps-mcp: community MCP server",
   "url": "https://github.com/LunaParker/waveapps-mcp"
  },
  {
   "title": "Supergood: Wave API report card (September 2026)",
   "url": "https://supergood.ai/api-report-card/wave"
  },
  {
   "title": "Knit: Wave Financial API integration guide",
   "url": "https://getknit.dev/blog/wave-financial-api-integration-guide-in-depth.md"
  },
  {
   "title": "Trustpilot: Wave",
   "url": "https://www.trustpilot.com/review/waveapps.com"
  },
  {
   "title": "Wave developer portal",
   "url": "https://developer.waveapps.com/hc/en-us"
  }
 ],
 "unverified": [
  "Acquisition price: US$405M is the widely reported figure; Wikipedia's history section gives $537M (likely the Canadian-dollar equivalent). H&R Block's release could not be fetched.",
  "The 26 May 2025 Pro requirement for third-party OAuth comes from a third-party guide (Knit), consistent with Supergood's report; Wave's developer portal returned 403.",
  "Rate limits (60 a minute, 5,000 a day) appear only in a third-party guide without a source.",
  "The absence of transaction and report queries is based on a community repository's schema introspected in May 2026, not Wave's own docs.",
  "Date when bank auto-import moved from free to Pro not confirmed; Wikipedia dates the Starter/Pro split as of 10 June 2024.",
  "G2 and Capterra ratings could not be retrieved (403).",
  "Wave's current paying customer count and revenue inside H&R Block not found."
 ],
 "researched": "2026-10-10"
}
---

## What it is

Wave is free accounting and invoicing for very small businesses in the US and Canada. It started in Toronto in 2009, launched free accounting in 2010, and was bought by H&R Block, announced on 11 June 2019 ([Wikipedia](https://en.wikipedia.org/wiki/Wave_Financial)). It says it has helped more than 5.9 million small business owners and recorded over 1.5 billion accounting transactions since 2011 ([about](https://www.waveapps.com/about-us)).

What Wave does well is give a one-person business a real double-entry ledger, invoicing and payments for nothing, and keep it simple enough to use without training.

## How it works

The Starter plan is free, with unlimited invoices, estimates, bills and bookkeeping records. Wave earns from payments (2.9% + $0.60 per card transaction), payroll (from $25 or $40 a month), receipts, bookkeeping services (Wave Advisors, from $149 a month) and the Pro plan at $19 USD or $25 CAD a month. Pro adds bank auto-import through Plaid, auto-categorisation, unlimited receipts and late-payment reminders ([pricing](https://www.waveapps.com/pricing)). The free tier keeps the ledger; connecting your bank costs money.

## Data and access

Wave has a free public GraphQL API, and a community schema check shows how little it exposes. The schema, introspected from Wave in May 2026, lets you read businesses, customers, invoices, estimates, products, vendors, sales taxes and accounts with their current balances. It has no query for transactions, journal entries or reports. It does have beta mutations to create money transactions, so you can post entries you cannot read back ([mcp-wave](https://github.com/rhanka/mcp-wave)). Imported bank transactions cannot be listed or matched, and payroll has no API at all ([Supergood](https://supergood.ai/api-report-card/wave)).

Access has also narrowed. Since 26 May 2025, according to a third-party guide, a business must be on Pro or Wave Advisors before a third-party app can connect through OAuth; a developer can still use a full access token for their own business ([Knit](https://getknit.dev/blog/wave-financial-api-integration-guide-in-depth.md)). There is no sandbox, Wave does not publish rate limits in anything retrieved, and its terms let it suspend or discontinue the API at any time. For a full copy of your books, the route is the app's CSV exports.

## Agents

Wave advertises no AI features and has no official MCP server or Claude connector as of 10 October 2026. Community MCP servers wrap the API for invoices, customers and payments. One logs in with the user's saved password and two-factor secret, which shows how far people go when the API does not cover the job ([waveapps-mcp](https://github.com/LunaParker/waveapps-mcp)). There is no approval gate on writes.

## Customers

Trustpilot shows 3.2 from 555 reviews as of October 2026, though at least one review is about a different company ([Trustpilot](https://www.trustpilot.com/review/waveapps.com)). People like that it is free, that invoicing is easy and that receipts scan well. They complain about support with no phone line and slow email, accounts locked for days, payments held or returned, billing after archiving a business, and few integrations; Wave's own page lists about 15 direct apps plus Make.com ([integrations](https://www.waveapps.com/integrations)).

## History

Wave raised venture money from OMERS, Charles River, Social Capital and NAB Ventures, ran on advertising until 2017, tried lending and dropped it, and sold to H&R Block in 2019, reported at US$405M. Under H&R Block it has moved from free-everything to freemium: bank import on Pro, US payroll rebuilt on Check in May 2025, and API access tied to Pro.

## For ownpurse

Wave shows the low-price end of the market and how little a free ledger gives back through its API. A small business on Wave can get its invoices and account balances out by API, but not the transactions that make up the books. For ownpurse that means a Wave connector starts from the files Wave lets you export: drop the CSV in the drawer, keep it by hash, project it into facts, and check the totals against Wave's reports. That fits ownpurse's model, where files are sources like any other, and it is the strongest case that an owner-held copy matters when the vendor decides what you can read.
