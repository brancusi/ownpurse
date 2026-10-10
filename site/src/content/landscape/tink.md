---
{
 "slug": "tink",
 "name": "Tink",
 "url": "https://tink.com",
 "segment": "bank-aggregator",
 "status": "acquired",
 "founded": "2012, Stockholm",
 "hq": "Stockholm, Sweden (Regeringsgatan 38); 13 offices serving 19 European markets.",
 "backing": "Wholly owned by Visa (NYSE: V) since 10 March 2022; agreed 24 June 2021 for EUR 1.8bn total consideration including cash and retention incentives. Before that, venture-backed: EUR 90m round (January 2020, co-led by Dawn Capital, HMI Capital and Insight Partners), EUR 85m extension (December 2020, co-led by Eurazeo Growth and Dawn Capital), strategic investment from PayPal (2019 and June 2020).",
 "tagline": "A European open banking platform, owned by Visa, that sells bank data, enrichment and account-to-account payments to banks, lenders and payment companies.",
 "scope": "Tink aggregates PSD2 and non-PSD2 bank data across Europe and sells products on top: Transactions, Account Check, Balance, Income, Expense and Risk checks, Business Transactions, Investments, Loans, data enrichment, a Money Manager toolkit, and Pay by Bank and VRP payments. Customers can work under Tink's PSD2 licence. It does not sell to individuals and does not keep books.",
 "domains": [
  "bank",
  "payments"
 ],
 "customers_served": [
  "enterprise",
  "mid-market",
  "developers"
 ],
 "spectrum": {
  "custody": {
   "score": 1,
   "note": "Banks hold the record; Tink holds a user object and fetched data on its platform for your app."
  },
  "locality": {
   "score": 0,
   "note": "Cloud API only."
  },
  "history": {
   "score": 1,
   "note": "Booked and pending transactions with a providerMutability flag; no versions or as-of queries."
  },
  "openness": {
   "score": 2,
   "note": "Public docs with llms.txt and Markdown pages, self-serve Console with Demo Bank; prices via sales only."
  },
  "breadth": {
   "score": 2,
   "note": "Bank accounts, cards, investments and loans data plus payments and enrichment, still all bank-side."
  },
  "autonomy": {
   "score": 1,
   "note": "Background refresh and automatic enrichment; never posts to books."
  },
  "agent_readiness": {
   "score": 2,
   "note": "Clean OAuth REST API and llms.txt; no MCP server or agent tooling found."
  },
  "integration_ease": {
   "score": 2,
   "note": "Console sign-up and Demo Bank are self-serve; production pricing and enterprise products require sales."
  },
  "market": {
   "score": 4,
   "note": "Banks, lenders, PSPs and large merchants (Adyen, Revolut, Deutsche Bahn), under Visa."
  }
 },
 "api": {
  "public": true,
  "style": "REST with JSON (data/v2 endpoints), cursor pagination (nextPageToken, up to 100 per page); Tink Link hosted flow; webhooks; iOS, Android and web SDKs.",
  "auth": "OAuth 2.0: client credentials token (expires in 30 minutes, no refresh) for the backend; authorization-grant code exchanged for a user access token (7,200 seconds) with scopes such as accounts:read and transactions:read.",
  "limits": "100 transactions per page. Consent lifetimes follow PSD2 (typically 90 or 180 days by market). Rate limits not published.",
  "cost": "Not public. Standard plan 'contact us', Enterprise 'custom pricing'; Risk Insights, Business Transactions, Money Manager, Income Check, Payment Initiation and Expense Check are Enterprise-only.",
  "export": "No bulk export for end users; data reaches you only through the API.",
  "notes": "Amounts come as currency code plus unscaledValue and scale (exact decimals, not floats), and each transaction carries a providerTransactionId and a mutability flag, which suits a ledger that must agree to the cent."
 },
 "pricing": {
  "model": "Contract pricing by product and volume; Standard and Enterprise plans.",
  "entry": "Free Console account with Demo Bank test data; Standard plan via contact.",
  "top": "Enterprise: custom pricing, guaranteed SLAs, option to use your own licence.",
  "notes": "No figures published as of 10 October 2026."
 },
 "agents": {
  "mcp": "none",
  "ai_features": "Data enrichment (categories, merchant information launched October 2024), Risk Signals rules engine (January 2024), Risk Insights and income verification. No agent features announced.",
  "agent_writes": "Payment initiation and VRP are available to Enterprise customers through the API; no agent-specific write path.",
  "approval": "The payer authenticates every payment with their bank (strong customer authentication).",
  "notes": "Its docs publish an llms.txt and Markdown pages, which make them readable by agents even without an MCP server."
 },
 "integration": {
  "scope": "About page: 3,000+ bank connections, 19 markets, 10,000 developers, 10bn+ transactions per year; homepage title claims 6,000 connections (both undated, October 2026).",
  "notes": "Grew by acquisition: Instantor, Eurobits, OpenWrks' aggregation platform (September 2020, whose customers included Xero) and FinTecSystems (May 2021)."
 },
 "preference": {
  "scale": "600+ employees, 13 offices, 19 markets (about page, October 2026). At the close of the Visa deal (March 2022): 3,400+ banks connected, 300+ bank and fintech customers, about 500 staff.",
  "ratings": "Trustpilot 1.5 out of 5 from 28 reviews (October 2026), from end users; no business-buyer reviews reachable.",
  "sentiment": "End users distrust the bank-login step ('asking for my bank account password'). Business customers named on its site (Adyen, Revolut, Lydia, Snoop) praise coverage and conversion; independent developer sentiment is thin."
 },
 "timeline": [
  {
   "date": "2012",
   "event": "Founded in Stockholm."
  },
  {
   "date": "2019-06",
   "event": "PayPal makes its first strategic investment."
  },
  {
   "date": "2020-01-20",
   "event": "Closes a EUR 90m round co-led by Dawn Capital, HMI Capital and Insight Partners."
  },
  {
   "date": "2020-09-21",
   "event": "Acquires OpenWrks' aggregation platform, adding UK business account data (customers included Xero)."
  },
  {
   "date": "2020-12-11",
   "event": "Closes a EUR 85m extension co-led by Eurazeo Growth and Dawn Capital."
  },
  {
   "date": "2021-05-18",
   "event": "Announces the acquisition of FinTecSystems (Germany)."
  },
  {
   "date": "2021-06-24",
   "event": "Visa agrees to acquire Tink for EUR 1.8bn."
  },
  {
   "date": "2022-03-10",
   "event": "Visa completes the acquisition; Tink runs as a standalone subsidiary."
  },
  {
   "date": "2024-01-25",
   "event": "Launches Risk Signals."
  },
  {
   "date": "2024-10-01",
   "event": "Launches Merchant Information."
  },
  {
   "date": "2025-11",
   "event": "UK's first commercial VRP over Visa A2A, with Kroo Bank and Utilita."
  }
 ],
 "relation": "infrastructure",
 "relation_note": "Tink sits under other products' bank feeds in Europe, the layer accounting tools buy bank data from, not something a person running ownpurse would connect to directly.",
 "borrow": "Exact amounts as unscaled value plus scale, a provider transaction id kept apart from the platform id, and an explicit mutability flag on each transaction.",
 "avoid": "Enterprise-only gating of the useful products and pricing nobody can see; a person cannot get their own data out this way.",
 "sources": [
  {
   "title": "Tink homepage",
   "url": "https://tink.com/"
  },
  {
   "title": "Tink about us",
   "url": "https://tink.com/about-us/"
  },
  {
   "title": "Tink pricing",
   "url": "https://tink.com/pricing/"
  },
  {
   "title": "Tink docs llms.txt",
   "url": "https://docs.tink.com/llms.txt"
  },
  {
   "title": "Tink docs: how to get an access token",
   "url": "https://docs.tink.com/resources/api-setup/get-access-token.md"
  },
  {
   "title": "Tink docs: list transactions",
   "url": "https://docs.tink.com/resources/transactions/list-transactions.md"
  },
  {
   "title": "Tink press",
   "url": "https://tink.com/press/"
  },
  {
   "title": "Visa completes acquisition of Tink (10 Mar 2022)",
   "url": "https://tink.com/press/visa-completes-acquisition-tink/"
  },
  {
   "title": "Visa acquires Tink (24 Jun 2021)",
   "url": "https://tink.com/press/visa-acquires-tink/"
  },
  {
   "title": "Tink acquires OpenWrks platform (21 Sep 2020)",
   "url": "https://tink.com/press/tink-acquires-openwrks/"
  },
  {
   "title": "Tink acquires FinTecSystems (18 May 2021)",
   "url": "https://tink.com/press/tink-acquires-fintecsystems/"
  },
  {
   "title": "PayPal investment in Tink (3 Jun 2020)",
   "url": "https://tink.com/press/tink-paypal-investment/"
  },
  {
   "title": "Tink blog",
   "url": "https://tink.com/blog/"
  },
  {
   "title": "Trustpilot: Tink",
   "url": "https://www.trustpilot.com/review/tink.com"
  }
 ],
 "unverified": [
  "Co-founder Fredrik Hedberg (only Daniel Kjellén is named in the pages read).",
  "Whether Kjellén is still CEO in 2026, and any Visa-era layoffs.",
  "That Tink began as a consumer budgeting app before turning B2B.",
  "Dates of the Instantor and Eurobits acquisitions.",
  "Consent lifetimes per market and API rate limits.",
  "Which connection count is current (3,000+ on the about page or 6,000 in the homepage title)."
 ],
 "researched": "2026-10-10"
}
---

## What it is

Tink is a Stockholm open banking platform, [founded in 2012](https://tink.com/about-us/) and wholly owned by Visa since March 2022. Its homepage describes it as 'a Visa solution' and leads with 'No PSD2 license required': customers can build on Tink's licence instead of getting their own. The [about page](https://tink.com/about-us/) claims 3,000+ bank connections, 19 markets, 10,000 developers and more than 10bn transactions a year. The [homepage](https://tink.com/) title says 6,000 connections.

## How it works

Tink connects to banks through PSD2 APIs and, where needed, other connections, and sells products on that data: Transactions, Account Check, Balance, Income and Expense Check, Risk Insights, Business Transactions, Investments, Loans and data enrichment. A Money Manager toolkit lets banks build budgeting features. On payments it offers Pay by Bank, Auto Payments, sweeping VRP and payouts, and since 2025 Visa A2A in the UK. Users connect through Tink Link, a hosted flow.

## Data and access

The [auth guide](https://docs.tink.com/resources/api-setup/get-access-token.md) is standard OAuth. A client-credentials token (30 minutes, no refresh) creates a Tink user and an authorization grant, which is exchanged for a user token (two hours) with scopes like `transactions:read`. The [transactions endpoint](https://docs.tink.com/resources/transactions/list-transactions.md) pages 100 at a time. Each amount is an `unscaledValue` and a `scale`, which is exact. Each transaction has booked and value dates, original and display descriptions, a `providerTransactionId` and a `providerMutability` flag. That is a better shape for a ledger than most aggregators offer. The [pricing page](https://tink.com/pricing/) lists Standard and Enterprise plans with no figures. Most of the interesting products are Enterprise-only. A free Console account with a Demo Bank lets you test.

## Agents

Tink's docs publish an [llms.txt](https://docs.tink.com/llms.txt) and serve Markdown, so agents can read them. Its press and blog announce no MCP server, agent toolkit or agentic-commerce product. Visa's agent work sits elsewhere in Visa.

## Customers

Tink names Adyen, Revolut, Lydia, Kivra, Snoop, Bank Norwegian, Deutsche Bahn, Vodafone (via Adyen) and Paysend among its users. End-user reviews on [Trustpilot](https://www.trustpilot.com/review/tink.com) are few and negative (1.5 from 28), mostly from people unsure why an app asked for their bank login.

## History

Tink raised [EUR 90m in January 2020](https://tink.com/press/), an [EUR 85m extension in December 2020](https://tink.com/press/) and strategic money from [PayPal](https://tink.com/press/tink-paypal-investment/). It bought aggregators across Europe, including [OpenWrks' platform](https://tink.com/press/tink-acquires-openwrks/) (2020, whose customers included Xero) and [FinTecSystems](https://tink.com/press/tink-acquires-fintecsystems/) (2021). Visa agreed to buy it for EUR 1.8bn in June 2021 and [completed the deal on 10 March 2022](https://tink.com/press/visa-completes-acquisition-tink/), with co-founder and CEO Daniel Kjellén staying on. The pattern of consolidating local connectors into one European pipe, then being absorbed by a card network, is the clearest European example of bank data becoming network infrastructure.

## For ownpurse

Tink is not a source a person can use directly, but it shows where bank lines in European accounting tools come from, and it is a model for the data shape ownpurse wants: exact decimals, provider ids kept apart from platform ids, and a flag for whether a transaction may still change. If a European owner's bank feed reaches Xero through Tink, ownpurse sees it second-hand. That is one more reason to add the owner's own bank exports as an independent source and link the two by rule, not by trust.
