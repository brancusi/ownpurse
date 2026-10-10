---
{
 "slug": "cointracker",
 "name": "CoinTracker",
 "url": "https://www.cointracker.io",
 "segment": "crypto-accounting",
 "status": "active",
 "founded": "2017",
 "hq": "San Francisco, California, US",
 "backing": "Venture-backed. Y Combinator Winter 2018. Series A led by Accel, announced 27 January 2022, with General Catalyst, Initialized, YC Continuity and the venture arms of Coinbase, Kraken and Intuit; Accel said the company had raised only about $2 million before that round. The round size (widely reported as $100 million) and any valuation could not be confirmed against a primary source in this pass. The App Store lists the publisher as Nino Finance, Inc.",
 "tagline": "A hosted crypto portfolio tracker and tax calculator that syncs wallets and exchanges and produces tax forms.",
 "scope": "CoinTracker pulls transactions from wallets, blockchains and exchanges (it claims 500+ integrations, 400+ exchanges and 70+ blockchains and wallets), works out cost basis and capital gains, removes spam, categorizes DeFi activity and produces tax forms you can file through TurboTax, H&R Block or a CPA. It also tracks portfolio value and offers tax-loss harvesting. It does not keep general books, does not handle fiat business accounting, and runs only in its own cloud.",
 "domains": [
  "crypto",
  "tax",
  "personal",
  "reporting"
 ],
 "customers_served": [
  "consumer",
  "freelancer",
  "accountants",
  "smb"
 ],
 "spectrum": {
  "custody": {
   "score": 1,
   "note": "CoinTracker's cloud holds the reconciled record; you can download reports and forms, but the working copy is theirs."
  },
  "locality": {
   "score": 0,
   "note": "A web and mobile service only; nothing runs on your machine."
  },
  "history": {
   "score": 1,
   "note": "Transaction history is kept, but recalculation and edits replace earlier results; no versioned or as-of record is offered."
  },
  "openness": {
   "score": 1,
   "note": "Closed source with no public developer API found; data leaves as tax forms and reports."
  },
  "breadth": {
   "score": 1,
   "note": "Many sources, but one domain: crypto holdings and their tax."
  },
  "autonomy": {
   "score": 2,
   "note": "Syncs, categorizes and computes gains unattended; it writes to no one else's books."
  },
  "agent_readiness": {
   "score": 0,
   "note": "No public API, CLI or MCP server found; the only AI surface seen is the support chatbot Chatoshi."
  },
  "integration_ease": {
   "score": 1,
   "note": "Connecting your own wallets is easy, but a third party or an owner's own tools have no documented way in or out besides exports."
  },
  "market": {
   "score": 1,
   "note": "Mainly individual investors, with tax professionals and some businesses served."
  }
 },
 "api": {
  "public": false,
  "style": "No public API found. The product connects outward to exchanges and wallets with read-only keys and addresses.",
  "auth": "n/a for third parties; user accounts with token-based two-factor authentication (App Store listing).",
  "limits": "Not published.",
  "cost": "No API offered. Tax forms sit behind paid plans; plan prices could not be read on 2026-10-10 because cointracker.com refused automated requests.",
  "export": "Tax forms and reports for TurboTax, H&R Block or a CPA (App Store listing). The exact export formats and whether a full raw transaction export is available were not confirmed.",
  "notes": "The site moved from cointracker.io to cointracker.com (both 301-redirect to .com as of 2026-10-10). Its site, help centre and pricing page returned HTTP 403 to every fetch in this pass, so most product detail here comes from the App Store listing, Accel, YC and Trustpilot."
 },
 "pricing": {
  "model": "Annual subscription tiered by number of transactions per tax year; free portfolio tracking.",
  "entry": "A free tier for portfolio tracking (limits not confirmed).",
  "top": "A Trustpilot reviewer (7 October 2026) said tax forms for their volume needed a $1,999 plan; the full price list was not confirmed.",
  "notes": "Pricing page unreachable (HTTP 403) on 2026-10-10. Tiers and limits below the top end are unverified."
 },
 "agents": {
  "mcp": "none",
  "ai_features": "Automatic categorization of DeFi transactions and spam removal; personalized insights; a support chatbot named Chatoshi (its outage is logged on the status page, 6 October 2026).",
  "agent_writes": "No agent interface. The product itself writes only inside its own record.",
  "approval": "None relevant: there is no external write path to gate.",
  "notes": "No MCP server, CLI or agent documentation found as of 2026-10-10."
 },
 "integration": {
  "scope": "Claims 500+ crypto integrations, 400+ exchanges, 70+ blockchains and wallets, 600+ dapps and 50K+ smart contracts (App Store listing, version of 7 October 2026). Hands off to TurboTax and H&R Block. Accel named partnerships with Coinbase, OpenSea, Blockchain.com and TurboTax (January 2022).",
  "notes": "Integrations are inbound: CoinTracker collects from sources. Outbound integration is the tax hand-off. Accuracy of syncs on large histories is a recurring review complaint, and the status page logged an ETH and EVM wallet sync failure that ran from 9 September to 2 October 2026."
 },
 "preference": {
  "scale": "\"Trusted by more than 3 million people since 2017\" (App Store description, 2026). YC lists a team of 70.",
  "ratings": "Trustpilot 4.6 from 1,878 reviews (77% five-star, 5% one-star), 289 in the last 12 months, read 2026-10-10. Apple App Store 4.72 from 14,356 ratings (US, 2026-10-10). G2 and Capterra pages could not be read.",
  "sentiment": "Praised for ease of use, a clean interface, seeing many exchanges in one place and the TurboTax hand-off. Complaints: wrong capital gains or missing cost basis on complex histories, generic support replies, slow sync and recalculation on large files, and the cost of the plan needed for forms."
 },
 "timeline": [
  {
   "date": "2017",
   "event": "Founded in San Francisco by Jon Lerner and Chandan Lodha"
  },
  {
   "date": "2018",
   "event": "Y Combinator Winter 2018 batch"
  },
  {
   "date": "2018-07-27",
   "event": "iOS app first released"
  },
  {
   "date": "2022-01-27",
   "event": "Accel announces its Series A in CoinTracker, with General Catalyst, Initialized, YC Continuity, Coinbase, Kraken and Intuit"
  },
  {
   "date": "by 2026-10",
   "event": "Primary domain moved from cointracker.io to cointracker.com"
  },
  {
   "date": "2026-09-09 to 2026-10-02",
   "event": "ETH and EVM wallet sync failures for some users, logged on the status page"
  },
  {
   "date": "2026-10-06",
   "event": "Chatoshi support chatbot outage blocks tickets for about an hour"
  }
 ],
 "relation": "neighbour",
 "relation_note": "CoinTracker solves a person's crypto tax problem in its own cloud. ownpurse does not compete for that job; at most, a CoinTracker export is one more source an owner could drop into the drawer, and CoinTracker shows what people expect a crypto view to answer.",
 "borrow": "The tax-lot breakdown and cost-basis options as views over the facts; read-only connections to wallets and exchanges as the default; a clean hand-off to the tools people already file with (TurboTax, H&R Block, a CPA).",
 "avoid": "Being the only place the reconciled record lives; opaque recalculation that leaves a person arguing with support about a number; putting the output a person needs (their forms) behind the top price.",
 "sources": [
  {
   "title": "CoinTracker on the Apple App Store (description, ratings, publisher)",
   "url": "https://apps.apple.com/us/app/id1401499763"
  },
  {
   "title": "CoinTracker, Y Combinator company page",
   "url": "https://www.ycombinator.com/companies/cointracker"
  },
  {
   "title": "Accel: Our Series A in CoinTracker (27 January 2022)",
   "url": "https://www.accel.com/noteworthies/our-series-a-in-cointracker-the-leader-in-crypto-tax-portfolio-tracking"
  },
  {
   "title": "Accel portfolio page: CoinTracker",
   "url": "https://www.accel.com/companies/cointracker"
  },
  {
   "title": "CoinTracker reviews on Trustpilot",
   "url": "https://www.trustpilot.com/review/cointracker.io"
  },
  {
   "title": "CoinTracker status page",
   "url": "https://status.cointracker.com"
  },
  {
   "title": "CoinTracker on GitHub",
   "url": "https://github.com/cointracker"
  },
  {
   "title": "CoinTracker pricing (unreachable, HTTP 403, 2026-10-10)",
   "url": "https://www.cointracker.com/pricing"
  }
 ],
 "unverified": [
  "Series A amount (widely reported as $100 million) and any valuation",
  "Current plan names, prices and transaction limits; only one reviewer's mention of a $1,999 plan was seen",
  "Whether a full raw transaction export exists and its format",
  "Whether any partner or embedded API exists (for exchanges or tax software)",
  "Date of the move from cointracker.io to cointracker.com",
  "Business or enterprise product details",
  "G2 and Capterra ratings (pages blocked)",
  "Any past security incidents; none were checked against a primary source"
 ],
 "researched": "2026-10-10"
}
---

## What it is

CoinTracker is a hosted crypto portfolio tracker and tax calculator. You connect wallets and exchanges, it works out your cost basis and capital gains, and it produces the forms you file. Its App Store listing says it is "trusted by more than 3 million people since 2017" and claims 500+ integrations, including 400+ exchanges and 70+ blockchains and wallets ([App Store](https://apps.apple.com/us/app/id1401499763)). It is a [Y Combinator Winter 2018](https://www.ycombinator.com/companies/cointracker) company in San Francisco, founded by Jon Lerner and Chandan Lodha.

## How it works

You add exchange API keys (read-only) and wallet addresses. CoinTracker pulls the transactions, removes spam, categorizes DeFi activity and calculates gains. It offers tax-loss harvesting, a tax-lot breakdown and a choice of cost-basis method, then files through TurboTax, H&R Block or your own CPA. Everything runs in its cloud: web and mobile apps, no local install. The App Store listing says it is SOC 1 and SOC 2 certified.

The reconciled record lives with CoinTracker. Recalculating replaces the earlier result; there is no versioned history you can query as of a date.

## Data and access

No public developer API was found, and the company's one public GitHub repository is not an SDK. What leaves the product is reports and tax forms. Whether a full raw transaction export is offered, and in what format, was not confirmed in this pass: cointracker.com, its pricing page and its help centre returned HTTP 403 to every fetch on 2026-10-10. Both cointracker.io addresses now redirect to cointracker.com.

Pricing is an annual subscription tiered by transaction count. A [Trustpilot reviewer](https://www.trustpilot.com/review/cointracker.io) on 7 October 2026 said the forms for their volume needed a $1,999 plan. The rest of the price list is unverified.

## Agents

There is no MCP server, CLI or agent documentation. The AI you meet is a support chatbot, Chatoshi; its outage on 6 October 2026 blocked tickets for about an hour ([status page](https://status.cointracker.com)). An agent working for you could only drive the web app, which no tool supports.

## Customers

Mostly individual investors at tax time, plus the accountants who prepare their returns and some businesses. On [Trustpilot](https://www.trustpilot.com/review/cointracker.io) it scores 4.6 from 1,878 reviews (77% five-star), and on the US App Store 4.72 from 14,356 ratings (both read 2026-10-10). People praise how easy it is, the clean interface, seeing every exchange in one place, and the TurboTax hand-off. The complaints are the hard parts of crypto tax: wrong gains or missing cost basis on complex histories, generic support replies, slow syncs on big files. The status page logged EVM wallet sync failures from 9 September to 2 October 2026.

## History

Founded in 2017, CoinTracker went through YC in early 2018 and shipped its iOS app in July 2018. [Accel announced its Series A](https://www.accel.com/noteworthies/our-series-a-in-cointracker-the-leader-in-crypto-tax-portfolio-tracking) on 27 January 2022, with General Catalyst, Initialized, YC Continuity and the venture arms of Coinbase, Kraken and Intuit. Accel noted the company had raised only about $2 million before that. The same post names partnerships with Coinbase, OpenSea, Blockchain.com and TurboTax. The round size and valuation were not confirmed against a primary source here.

## For ownpurse

CoinTracker is a neighbour, not a rival. It answers one question well for a large consumer audience: what do I owe on my crypto? It does that by holding the only reconciled copy in its cloud and giving you forms, not facts. ownpurse would treat a CoinTracker export, if a full one exists, as one more source in the drawer, and would rebuild the gains locally so a person can see why a number is what it is. Worth borrowing: the tax-lot breakdown and cost-basis choice as views, read-only connections by default, and a hand-off to the tools people already file with. Worth avoiding: recalculation you cannot inspect, and putting the output a person needs behind the top price.
