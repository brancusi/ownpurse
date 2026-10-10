---
{
 "slug": "tiller",
 "name": "Tiller",
 "url": "https://tiller.com",
 "segment": "personal-finance",
 "status": "active",
 "founded": "Launched 2016 (Tiller's own FAQ); founding year of the company not confirmed",
 "hq": "Seattle, Washington (remote team in several states)",
 "backing": "Bootstrapped: 'funded exclusively by customer subscriptions', no venture capital or outside shareholders (tiller.com/about, checked 10 October 2026). Founder Peter Polson.",
 "tagline": "A subscription service that fills your own Google Sheets or Excel spreadsheet with daily bank and card transactions and balances.",
 "scope": "Tiller connects to banks, cards, loans and investment accounts through Yodlee and Plaid and writes transactions and balances into spreadsheets you own, with templates, rule-based categorization (AutoCat), daily email summaries and an MCP server for Claude. It does not do double-entry bookkeeping, tax filing, bill payment or investment positions, and has no public API of its own. It cannot connect EU or UK banks.",
 "domains": [
  "personal",
  "bank",
  "reporting"
 ],
 "customers_served": [
  "consumer",
  "freelancer"
 ],
 "spectrum": {
  "custody": {
   "score": 3,
   "note": "The spreadsheet is yours and stays yours if you leave, but it sits in Google's or Microsoft's cloud and Tiller holds the feed."
  },
  "locality": {
   "score": 1,
   "note": "Cloud spreadsheets (Google Sheets, Microsoft 365 Excel); nothing runs on your own machine by design."
  },
  "history": {
   "score": 1,
   "note": "New rows are appended and your edits are kept, but rows are mutable and there is no as-of view beyond the spreadsheet's version history."
  },
  "openness": {
   "score": 2,
   "note": "Closed service, but full data in an open spreadsheet readable through the Sheets or Excel APIs; templates are open to edit."
  },
  "breadth": {
   "score": 1,
   "note": "Bank, card, loan and investment balances and transactions; one domain, personal cash flow."
  },
  "autonomy": {
   "score": 2,
   "note": "Fills transactions daily unattended and AutoCat categorizes by rules; AI Suggest proposes, MCP writes need approval."
  },
  "agent_readiness": {
   "score": 3,
   "note": "Official MCP server for Claude (read plus category and note writes), stable ids and a fixed schema documented for AI use."
  },
  "integration_ease": {
   "score": 3,
   "note": "Easy for an owner: the data is a spreadsheet any tool can read; no partner API for third parties."
  },
  "market": {
   "score": 0,
   "note": "Consumers, households and solo businesses."
  }
 },
 "api": {
  "public": false,
  "style": "No Tiller API. Data lands in Google Sheets or Excel, reachable through Google's Sheets API or Microsoft's Graph and Excel APIs with the owner's own account; an MCP server for Claude.",
  "auth": "Sign in with a Google or Microsoft account; bank logins go to Yodlee or Plaid; MCP connects through Claude's connector directory.",
  "limits": "Up to five Google or Microsoft spreadsheets per subscription (tiller.com/reviews); some institutions (Fidelity, Vanguard) restrict refresh timing; missing transactions after 90+ days of inactivity may not be recoverable.",
  "cost": "Included in the $99 a year subscription, including Tiller MCP for Claude; the spreadsheet APIs are Google's and Microsoft's.",
  "export": "The data already is the export: a spreadsheet in your account that stays yours if you cancel; download as CSV or XLSX from Sheets or Excel.",
  "notes": "Tiller calls out stable Transaction, Account and Balance IDs and standard headers so AI tools can join and filter rows reliably."
 },
 "pricing": {
  "model": "Single annual subscription with a 30-day free trial",
  "entry": "$99 a year after a 30-day free trial (checked 10 October 2026)",
  "top": "$99 a year (one plan)",
  "notes": "Sales tax may apply. No ads, no data sales; full refund on request. Earlier price of $79 a year is from memory, date of change unverified."
 },
 "agents": {
  "mcp": "official",
  "ai_features": "Tiller MCP for Claude (listed in Claude's connector directory) reads transactions and categories in Google Sheets; AI Suggest uses Gemini to propose categories for transactions AutoCat leaves uncategorized; guides for ChatGPT, Gemini, Copilot and Claude inside Sheets and Excel.",
  "agent_writes": "Tiller MCP for Claude can update transaction categories and notes; other AI tools can edit whatever spreadsheet cells their connection is granted.",
  "approval": "Tiller MCP writes categories and notes 'only after you allow the action' (a per-action permission); there is no sealed package or change log beyond the spreadsheet's own version history.",
  "notes": "Tiller says it sends transactions to an AI tool only if you turn on AI Suggest or connect the MCP, and never trains AI on your data. Its pitch: 'Unlike apps or MCPs, with Tiller you can verify answers directly in your spreadsheet.'"
 },
 "integration": {
  "scope": "Over 20,000 institutions through Yodlee (main provider) and Plaid (payment apps such as Venmo and PayPal); the marketing site says 21,000+. Mostly US; no EU or UK banks; Canadian institutions out of troubleshooting support since March 2024. No crypto exchanges, no Apple Card, no investment positions.",
  "notes": "Escalated feed problems take about 4 to 6 weeks on average to fix at the data provider. Institutions can move between a primary and a backup aggregator, which creates a second account instance and duplicates that a cleanup workflow removes (help article, 27 May 2026). Import guides exist for Mint, Quicken, YNAB, Personal Capital and Money in Excel."
 },
 "preference": {
  "scale": "More than 131,000 installations of its add-on (tiller.com/reviews, 2026); 'tens of thousands' of customers (vendor claim). Team of 19 plus 3 advisors listed.",
  "ratings": "4.8 stars on the Google Workspace Marketplace (vendor badge, September 2024; count not confirmed). Vendor Q1 2026 survey: NPS 43, 95% would be disappointed to lose it. Trustpilot: no reviews (checked 10 October 2026).",
  "sentiment": "Praised for flexibility, owning the data, no ads, rule-based categorization and responsive human support; a common landing spot for people leaving Mint and Quicken, and Microsoft's recommended replacement for Money in Excel. Complaints: a spreadsheet learning curve ('not a tool for the general masses'), and bank connection breakages that can take weeks to fix through Yodlee."
 },
 "timeline": [
  {
   "date": "2016",
   "event": "Tiller launches as a spreadsheet-based personal finance service"
  },
  {
   "date": "2023-06-30",
   "event": "Microsoft ends Money in Excel and recommends Tiller as the replacement"
  },
  {
   "date": "2024-03",
   "event": "Canadian institutions no longer eligible for troubleshooting support"
  },
  {
   "date": "2024",
   "event": "Mint shuts; Tiller publishes a Mint transition guide [timing of guide unverified]"
  },
  {
   "date": "2024-09",
   "event": "4.8-star Google Workspace Marketplace badge"
  },
  {
   "date": "2026-05-27",
   "event": "Help article on switching between primary and backup aggregators"
  },
  {
   "date": "2026",
   "event": "Tiller MCP for Claude live in Claude's connector directory [launch date unverified]"
  },
  {
   "date": "2026-10-10",
   "event": "$99 a year, 30-day trial; tillerhq.com redirects to tiller.com"
  }
 ],
 "relation": "kin",
 "relation_note": "Tiller is the closest consumer kin: it already sells the idea that the owner holds the copy and that AI should read your own record rather than your bank. It differs on everything under the spreadsheet: cloud-hosted, mutable rows, one aggregator pipe, no double entry, no provenance, and no business books.",
 "borrow": "A fixed, documented schema with stable ids that AI tools can rely on; 'point AI at your data, not your bank' as the pitch; answers you can check against the source rows; AI as optional; writes from an agent only after the owner allows them; import guides for every app people are leaving; subscriptions as the only business model.",
 "avoid": "A record that anyone, or any agent with edit access, can silently overwrite; deduplication by date cutoff when an aggregator changes; a single aggregator as the only path for bank data; storing the authoritative copy in a third party's cloud.",
 "sources": [
  {
   "title": "Tiller: pricing",
   "url": "https://tiller.com/pricing/"
  },
  {
   "title": "Tiller: home page",
   "url": "https://tiller.com/"
  },
  {
   "title": "Tiller: AI and Tiller",
   "url": "https://tiller.com/ai/"
  },
  {
   "title": "Tiller: about",
   "url": "https://tiller.com/about/"
  },
  {
   "title": "Tiller: reviews",
   "url": "https://tiller.com/reviews/"
  },
  {
   "title": "Tiller Help: supported financial institutions and accounts",
   "url": "https://help.tiller.com/en/articles/432693-tiller-supported-financial-institutions-and-accounts"
  },
  {
   "title": "Tiller Help: expectations around data feed disconnections and outages",
   "url": "https://help.tiller.com/en/articles/4135767-expectations-around-data-feed-disconnections-and-outages"
  },
  {
   "title": "Tiller Help: changing aggregators and cleaning up data (27 May 2026)",
   "url": "https://help.tiller.com/en/articles/14847059-changing-aggregators-and-cleaning-up-data"
  },
  {
   "title": "Tiller Help: transitioning from Money in Excel to Tiller",
   "url": "https://help.tiller.com/en/articles/6226887-transitioning-from-money-in-excel-to-tiller"
  },
  {
   "title": "Microsoft Support: end of support for Money in Excel",
   "url": "https://support.microsoft.com/en-us/office/end-of-support-for-money-in-excel-wolfram-data-types-and-partner-offers-43ceac07-794e-4015-af04-21bb4795a14a"
  },
  {
   "title": "Trustpilot: tillerhq.com",
   "url": "https://www.trustpilot.com/review/tillerhq.com"
  }
 ],
 "unverified": [
  "Company founding year and co-founders other than Peter Polson.",
  "Launch date of Tiller MCP for Claude.",
  "Date and size of any price change (an earlier $79 a year price is from memory).",
  "Date Excel support launched, and whether Excel use requires a Microsoft 365 workbook in OneDrive.",
  "Count of Google Workspace Marketplace ratings behind the 4.8 badge; G2 and Capterra ratings not checked.",
  "Reddit sentiment could not be fetched (blocked); complaints come from Tiller's own help articles and review page.",
  "Customer count beyond the vendor's 'tens of thousands' and 131,000 installations.",
  "Web search budget was exhausted during this pass; findings rely on directly fetched pages."
 ],
 "researched": "2026-10-10"
}
---

## What it is

Tiller is a personal finance service that keeps a spreadsheet you own up to date with your bank and card transactions and balances. It works in Google Sheets and Microsoft Excel, launched in 2016, and is based in Seattle with a remote team. Its founder is Peter Polson. It is [bootstrapped](https://tiller.com/about/): "funded exclusively by customer subscriptions," no venture capital, no ads, no data sales. It costs [$99 a year after a 30-day free trial](https://tiller.com/pricing/) (checked 10 October 2026), with one subscription covering up to five spreadsheets.

## How it works

You sign in with a Google or Microsoft account, connect your institutions, and Tiller fills a Transactions sheet and a Balances sheet daily. The Foundation Template adds budgets, categories and reports; AutoCat applies your categorization rules; Hello, Money emails a daily summary. Because it is a spreadsheet, you can add columns, split transactions, and build anything on top, and the community shares templates.

Bank data comes from two aggregators. [Tiller's help centre](https://help.tiller.com/en/articles/432693-tiller-supported-financial-institutions-and-accounts) says it supports over 20,000 institutions through Yodlee, with Plaid for payment apps such as Venmo and PayPal. It is mostly US: it cannot connect EU or UK banks because of open banking rules, and Canadian institutions lost troubleshooting support in March 2024. It does not cover crypto exchanges, Apple Card or investment positions.

## Data and access

The data is the export. The spreadsheet lives in your Google Drive or OneDrive and, as Tiller says, "remains yours, even if you stop using Tiller." Tiller has no public API of its own, but your spreadsheet is reachable through Google's and Microsoft's APIs with your own account.

The limits are those of a spreadsheet fed by an aggregator. Rows are mutable. When a feed breaks, escalated issues take [about 4 to 6 weeks](https://help.tiller.com/en/articles/4135767-expectations-around-data-feed-disconnections-and-outages) to fix, and transactions missed after 90 days of inactivity may be lost. Moving an institution between the primary and backup aggregator creates a second account and duplicate rows, which a [cleanup workflow](https://help.tiller.com/en/articles/14847059-changing-aggregators-and-cleaning-up-data) removes by date cutoff (27 May 2026).

## Agents

Tiller has leaned into AI without becoming an AI product. Its [AI page](https://tiller.com/ai/) says: "Tiller has an MCP server. Tiller MCP for Claude is live in Claude's connector directory." Claude can read transactions and categories in Google Sheets, and "updates transaction categories and notes, and only after you allow the action." AI Suggest uses Gemini to propose categories for transactions your rules miss. It also documents ChatGPT, Gemini, Copilot and Claude inside the spreadsheet apps. It says it never trains AI on your data and shares transactions with AI only if you turn these on. Its pitch is that AI reads a structured source you own, with stable Transaction, Account and Balance IDs, so you can check every answer against the rows.

## Customers

Households, individuals and solo businesses comfortable with spreadsheets. Tiller cites more than 131,000 installations, a 4.8-star Google Workspace Marketplace badge (September 2024), and a Q1 2026 customer survey with an NPS of 43 ([reviews page](https://tiller.com/reviews/), vendor figures). Reviewers praise owning the data, flexibility and human support; many came from Mint or Quicken. When Microsoft ended Money in Excel on 30 June 2023, [it recommended Tiller](https://support.microsoft.com/en-us/office/end-of-support-for-money-in-excel-wolfram-data-types-and-partner-offers-43ceac07-794e-4015-af04-21bb4795a14a). The complaints are the spreadsheet learning curve and broken bank connections.

## History

Tiller has stayed small and profitable-by-design for a decade while Mint, Money in Excel and others closed around it, and it has absorbed their users with import guides for each. It is a quiet counterexample to the venture-funded aggregator apps.

## For ownpurse

Tiller is kin. It already sells two of ownpurse's beliefs to consumers: the owner holds the copy, and AI should read your own record, not your bank. Its MCP even asks before writing. The differences are underneath. The copy is in a third party's cloud, rows can be overwritten without trace, all bank data comes through one aggregator pipe, there is no double entry, no provenance per row and no business books. ownpurse can be what sits under a Tiller-style view: an append-only local record where a categorization by an agent is a new fact, not an overwritten cell.

Borrow: a fixed schema with stable ids that agents can rely on, verifiable answers, AI as optional, and import guides for every tool people are leaving. Avoid: silent overwrites, dedupe by date cutoff, and a single aggregator as the only bank path.
