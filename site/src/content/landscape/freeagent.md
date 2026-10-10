---
{
 "slug": "freeagent",
 "name": "FreeAgent",
 "url": "https://www.freeagent.com",
 "segment": "incumbent-ledger",
 "status": "active",
 "founded": "2007",
 "hq": "Edinburgh, Scotland, UK",
 "backing": "Owned by NatWest Group since 2018 (about £53 million, per Wikipedia). Before that: £1.2 million on Seedrs (2015) and an AIM listing in 2016 that raised £10.7 million at a £34.1 million valuation.",
 "tagline": "A UK cloud ledger for freelancers, sole traders, small limited companies and landlords, free to anyone banking with NatWest, RBS, Ulster Bank or Mettle.",
 "scope": "Double-entry books behind a simple interface: bank feeds and transaction explanations, invoicing, bills, expenses, time and projects, payroll, and direct filing of VAT, Self Assessment, Corporation Tax and Making Tax Digital for Income Tax returns to HMRC. It is single-business, UK-only in practice, and has no inventory, multi-entity consolidation, or built-in chat assistant.",
 "domains": [
  "ledger",
  "bank",
  "invoicing",
  "payroll",
  "tax",
  "spend"
 ],
 "customers_served": [
  "freelancer",
  "consumer",
  "smb",
  "accountants"
 ],
 "spectrum": {
  "custody": {
   "score": 1,
   "note": "FreeAgent holds the live books; you can download a full Excel export and an attachments zip, but FreeAgent says the export can't be restored."
  },
  "locality": {
   "score": 0,
   "note": "Cloud only; there is no self-hosted or desktop edition."
  },
  "history": {
   "score": 1,
   "note": "Records are edited in place under lock dates; no as-of queries or append-only history is exposed."
  },
  "openness": {
   "score": 2,
   "note": "Closed source, but has a public REST API with a free sandbox and a full-account export (Excel plus attachments)."
  },
  "breadth": {
   "score": 2,
   "note": "Books, bank feeds, invoicing, expenses, payroll and HMRC tax filing for one business; no documents vault, wallets or multi-entity."
  },
  "autonomy": {
   "score": 1,
   "note": "'Guess' suggests explanations for bank transactions and rules apply categories; a person still confirms the books."
  },
  "agent_readiness": {
   "score": 2,
   "note": "Well-documented public API with OAuth 2.0; no official MCP server, CLI or agent docs, only community MCP servers."
  },
  "integration_ease": {
   "score": 3,
   "note": "Self-serve developer apps, OAuth 2.0, a sandbox with a rate-limit test header, 120 requests a minute per user; no webhooks documented."
  },
  "market": {
   "score": 1,
   "note": "Freelancers, sole traders, micro limited companies and landlords, plus the accountants who serve them."
  }
 },
 "api": {
  "public": true,
  "style": "REST over HTTPS with JSON or XML responses (v2), paginated with Link headers (25 per page by default, 100 max)",
  "auth": "OAuth 2.0 (authorization code) with refresh tokens; a separate Accountancy Practice API for practices",
  "limits": "120 requests per minute and 3,600 per hour per user of an integration, 15 token refreshes per minute; HTTP 429 with Retry-After when exceeded (dev.freeagent.com, read 2026-10-10)",
  "cost": "No API fee is documented; you need a FreeAgent subscription (or a bank-funded account) for the company being accessed.",
  "export": "Settings > Export All Data produces an Excel workbook (bank accounts and transactions, invoices, bills, contacts, projects, expenses, timeslips and more) plus a separate zip of attachments, available for 7 days; reports such as the trial balance export to CSV. FreeAgent says the export can't be restored into FreeAgent, and its staff say a full export isn't possible through the API.",
  "notes": "The API covers almost the whole product: contacts, invoices, bills, bank accounts, bank transactions and explanations, journal sets, categories, trial balance, P&L, balance sheet, payroll, tax returns, projects and attachments. Sandbox at api.sandbox.freeagent.com."
 },
 "pricing": {
  "model": "Flat monthly subscription per business type, free for customers of NatWest Group banks",
  "entry": "Unincorporated landlord £10 a month ex VAT; sole trader £19 a month ex VAT (50% off the first six months on monthly billing)",
  "top": "Limited company £33 a month ex VAT (£396 a year inc VAT on annual billing)",
  "notes": "Free for as long as you keep a NatWest, RBS or Ulster Bank business current account; Mettle qualifies for limited companies, sole traders and landlords with at least one transaction a month. Add-ons: Smart Capture Unlimited £6 a month and Amazon connection £7.20 a month (inc VAT). Accountants get a free Practice Dashboard and pay per client licence. Prices read from freeagent.com/pricing on 2026-10-10."
 },
 "agents": {
  "mcp": "community",
  "ai_features": "'Guess' auto-suggests explanations for bank transactions (a July 2026 third-party review cites about 93% accuracy); Smart Capture extracts receipts. No chat assistant.",
  "agent_writes": "Only through the public API with a user's OAuth token, using community MCP servers (for example one exposing 76 tools) or hosted connectors such as Composio; none is supported by FreeAgent.",
  "approval": "None in the product for API writes; an agent with a token writes directly. Lock dates and accountant review are the controls.",
  "notes": "A July 2026 review found no MCP server in FreeAgent's developer docs and no app in ChatGPT's directory."
 },
 "integration": {
  "scope": "More than 90 partner integrations as of September 2025 (AccountingWeb), in a curated directory: bank feeds via Open Banking, Stripe, PayPal, GoCardless, Shopify, Amazon, payroll and practice tools; Zapier for the rest.",
  "notes": "FreeAgent describes its approach as curated rather than a large marketplace; Open Banking feeds update about once every 24 hours, per its own Trustpilot reply."
 },
 "preference": {
  "scale": "Over 250,000 small businesses (freeagent.com about page, read 2026-10-10); Wikipedia gives about 225,000 in 2025, up from 200,000 a year earlier. Over 200 employees.",
  "ratings": "Trustpilot UK 4.6 from 3,034 reviews (read 2026-10-10); Capterra 4.5 from 179 reviews (listing updated May 2026).",
  "sentiment": "Praised for ease of use, plain invoicing and bookkeeping, and friendly human support; complaints are about reaching a person through automated chat, bank feeds running a day behind, and occasional sync problems."
 },
 "timeline": [
  {
   "date": "2007",
   "event": "Launched in Edinburgh by freelancers Ed Molyneux, Olly Headey and Roan Lavery"
  },
  {
   "date": "2012",
   "event": "Acquired US startup 60mo"
  },
  {
   "date": "2015",
   "event": "Raised £1.2 million through Seedrs crowdfunding"
  },
  {
   "date": "2016",
   "event": "Listed on AIM, raising £10.7 million at a £34.1 million valuation"
  },
  {
   "date": "2017",
   "event": "RBS began offering FreeAgent free to business banking customers"
  },
  {
   "date": "2018",
   "event": "Acquired by RBS, now NatWest Group, for about £53 million"
  },
  {
   "date": "2021-07",
   "event": "Roan Lavery succeeded Ed Molyneux as CEO"
  },
  {
   "date": "2026-04-06",
   "event": "Making Tax Digital for Income Tax began for sole traders and landlords over £50,000; FreeAgent files quarterly updates"
  },
  {
   "date": "2026-10",
   "event": "About page states over 250,000 small business customers"
  }
 ],
 "relation": "source",
 "relation_note": "A UK ledger ownpurse could read from with a connector, much like Xero: a good API, generous limits and a full export, held by the vendor and given away by a bank.",
 "borrow": "The bank-bundled distribution (the ledger is free because the bank pays for it), the tax-filing finish line that makes books feel worth keeping, and a clean, self-serve API with a sandbox and a rate-limit test header.",
 "avoid": "An export you can read but never restore, and a full export that exists only behind a button and not in the API.",
 "sources": [
  {
   "title": "FreeAgent pricing",
   "url": "https://www.freeagent.com/pricing/"
  },
  {
   "title": "FreeAgent about us",
   "url": "https://www.freeagent.com/company/about-us"
  },
  {
   "title": "FreeAgent API introduction (rate limits, sandbox)",
   "url": "https://dev.freeagent.com/docs/introduction"
  },
  {
   "title": "FreeAgent API docs",
   "url": "https://dev.freeagent.com/docs"
  },
  {
   "title": "Export all data, attachments and files",
   "url": "https://support.freeagent.com/hc/en-gb/articles/115001218490"
  },
  {
   "title": "FreeAgent API forum: export all via API",
   "url": "https://api-discuss.freeagent.com/t/export-all-via-api/2935"
  },
  {
   "title": "NatWest Group: FreeAgent brand page",
   "url": "https://www.natwestgroup.com/who-we-are/our-brands/freeagent.html"
  },
  {
   "title": "FreeAgent (Wikipedia)",
   "url": "https://en.wikipedia.org/wiki/FreeAgent"
  },
  {
   "title": "FreeAgent MTD for Income Tax",
   "url": "https://www.freeagent.com/features/mtd-for-income-tax/"
  },
  {
   "title": "ChatGPT + FreeAgent: the real integration options in 2026",
   "url": "https://www.usecarly.com/blog/chatgpt-freeagent-integration/"
  },
  {
   "title": "Community FreeAgent MCP server",
   "url": "https://github.com/samaxbytez/freeagent-mcp"
  },
  {
   "title": "Composio FreeAgent toolkit",
   "url": "https://composio.dev/toolkits/freeagent"
  },
  {
   "title": "FreeAgent on Trustpilot UK",
   "url": "https://uk.trustpilot.com/review/freeagent.com"
  },
  {
   "title": "FreeAgent on Capterra",
   "url": "https://www.capterra.com/p/122008/FreeAgent/reviews/"
  },
  {
   "title": "Which integrations does FreeAgent support?",
   "url": "https://support.freeagent.com/hc/en-gb/articles/360020765959"
  }
 ],
 "unverified": [
  "The £53 million acquisition price and 2016 AIM figures come from Wikipedia, not a primary filing.",
  "The 225,000 (2025) customer figure is from Wikipedia without a primary source; the about page's 250,000 has no date.",
  "The 90+ integrations figure (September 2025) comes from a search snippet of an AccountingWeb article that returned 403 when fetched.",
  "The 93% Guess accuracy figure comes from a third-party blog, not FreeAgent.",
  "Capterra 4.5 from 179 comes from a search snippet; the page returned 403 when fetched.",
  "Webhook support: none found in the docs read, but not confirmed absent.",
  "Whether FreeAgent for Landlords shipped MTD for Income Tax property submissions before April 2026."
 ],
 "researched": "2026-10-10"
}
---

## What it is

FreeAgent is a cloud accounting product for UK freelancers, sole traders, small limited companies, partnerships and landlords. Since 2018 it has belonged to [NatWest Group](https://www.natwestgroup.com/who-we-are/our-brands/freeagent.html), and that ownership is the most interesting thing about it: if you bank with NatWest, RBS or Ulster Bank, or run a Mettle account with at least one transaction a month, the software is [free for as long as you keep the account](https://www.freeagent.com/pricing/). It is the clearest case of the bank-bundled ledger, where the books are a retention feature for a current account, not a product sold on its own.

## How it works

You connect bank feeds, explain each transaction against a category, raise invoices, log expenses and time, run payroll, and file VAT, Self Assessment, Corporation Tax and, since April 2026, [Making Tax Digital for Income Tax](https://www.freeagent.com/features/mtd-for-income-tax/) updates straight to HMRC. Underneath is a double-entry ledger with a trial balance, P&L and balance sheet, but the interface hides the debits and credits. The built-in intelligence is modest: a feature called Guess suggests explanations for bank lines, and Smart Capture reads receipts. Accountants get a free Practice Dashboard and pay per client licence, and bank-funded clients cost them nothing.

Without a bank, the price is a flat monthly fee by business type: £10 a month ex VAT for an unincorporated landlord, £19 for a sole trader, £27 for a partnership and £33 for a limited company, with half off the first six months (read 2026-10-10).

## Data and access

The [public API](https://dev.freeagent.com/docs) is REST with JSON or XML, OAuth 2.0, and covers nearly the whole product: bank transactions and their explanations, journal sets, invoices, bills, payroll, tax returns and the trial balance. The [documented limits](https://dev.freeagent.com/docs/introduction) are 120 requests a minute and 3,600 an hour per user, generous next to Xero's daily cap, and the sandbox accepts an `X-RateLimit-Test` header that drops you to 5 requests a minute so you can test back-off. No webhooks are documented.

For owners there is [Export All Data](https://support.freeagent.com/hc/en-gb/articles/115001218490): an Excel workbook of every record type plus a zip of attachments. FreeAgent is candid that the file is for record keeping and cannot be restored, and its staff say a full export is [not possible through the API](https://api-discuss.freeagent.com/t/export-all-via-api/2935). The record is vendor-held and edited in place; history beyond lock dates is not exposed.

## Agents

FreeAgent has no official MCP server or chat assistant. A [July 2026 review](https://www.usecarly.com/blog/chatgpt-freeagent-integration/) found none in the developer docs and no ChatGPT app. Community servers fill the gap, such as [freeagent-mcp](https://github.com/samaxbytez/freeagent-mcp) and hosted connectors from [Composio](https://composio.dev/toolkits/freeagent), and they write straight through a user's token with no approval step in the product.

## Customers

FreeAgent's [about page](https://www.freeagent.com/company/about-us) claims over 250,000 small businesses (read 2026-10-10); Wikipedia gives about 225,000 for 2025. Reviews are strong: [Trustpilot UK](https://uk.trustpilot.com/review/freeagent.com) shows 4.6 from 3,034 reviews, and Capterra 4.5 from 179. People praise how easy it is and that they can talk to a human; the complaints are bank feeds a day behind and automated support channels.

## History

Three freelancers started it in Edinburgh in 2007. It crowdfunded £1.2 million on Seedrs in 2015, listed on AIM in 2016, was offered free to RBS customers from 2017, and was bought by RBS (now NatWest) in 2018 for about £53 million. Co-founder Roan Lavery has been CEO since July 2021. The pattern is worth noting: an independent ledger became a bank's customer-retention tool, and its growth since has been driven by the bank channel and by HMRC's digital filing mandates.

## For ownpurse

FreeAgent is a source, not a rival. It holds the books of exactly the people ownpurse has in mind (a freelancer whose business and household share one card), and its API is better suited to a local mirror than Xero's: higher limits, a sandbox, near-complete coverage. A FreeAgent connector would append bank lines, explanations and journal sets as facts and rebuild the trial balance locally, checking it against FreeAgent's own. What ownpurse adds is what FreeAgent lacks: a copy the owner holds that can be restored, every version kept, and an agent interface where writes go back only as approved packages. The lesson from its distribution is that small owners take the ledger that comes free with something they already have.
