---
{
 "slug": "yodlee",
 "name": "Yodlee",
 "url": "https://www.yodlee.com",
 "segment": "bank-aggregator",
 "status": "acquired",
 "founded": "1999",
 "hq": "Redwood City, California",
 "backing": "Private equity: STG agreed to buy Yodlee from Envestnet on 25 June 2025 and closed on 2 September 2025 (terms not disclosed). Before that: Nasdaq IPO 3 October 2014 (YDLE, $75M at $12 a share), then Envestnet bought it on 10 August 2015 for about $660M, as reported.",
 "tagline": "The oldest financial-data aggregator, selling account aggregation, verification and data enrichment to banks, wealth firms and fintechs.",
 "scope": "Yodlee aggregates bank, card, loan and investment accounts from about 17,000 to 19,000 data sources. It sells aggregation, account verification, transaction enrichment, and products for personal finance, credit, wealth and business finance. Its customers are institutions and fintechs, not individuals. It keeps no ledger and posts nothing, and an individual cannot sign up and pull their own data for personal use.",
 "domains": [
  "bank",
  "personal",
  "integration"
 ],
 "customers_served": [
  "enterprise",
  "mid-market",
  "developers"
 ],
 "spectrum": {
  "custody": {
   "score": 1,
   "note": "Yodlee holds the connections and the aggregated copy for its client; the owner holds nothing through it."
  },
  "locality": {
   "score": 0,
   "note": "A hosted API and the FastLink widget; no self-hosted option."
  },
  "history": {
   "score": 1,
   "note": "Current accounts and a transaction window, refreshed; no versioned record."
  },
  "openness": {
   "score": 1,
   "note": "The OpenAPI spec for the v1.1 APIs is on GitHub, but access and code are closed and contract-gated."
  },
  "breadth": {
   "score": 2,
   "note": "Wider than bank feeds: cards, loans, investment holdings and rewards from a very large source list."
  },
  "autonomy": {
   "score": 0,
   "note": "A data pipe; writes nothing to books."
  },
  "agent_readiness": {
   "score": 2,
   "note": "Public REST API and spec; no MCP or agent tooling found."
  },
  "integration_ease": {
   "score": 1,
   "note": "Sandbox registration exists, but production is sales-led and the developer site's pricing and docs URLs we tried returned 404."
  },
  "market": {
   "score": 4,
   "note": "14 of the 20 largest US financial institutions and 7 of the top 10 US wealth firms, by Yodlee's own count."
  }
 },
 "api": {
  "public": true,
  "style": "REST (JSON), OpenAPI 2.0 spec for the v1.1 APIs; FastLink hosted linking widget",
  "auth": "Client credentials issue access tokens per user; JWT samples exist on GitHub [exact token lifetimes unverified]",
  "limits": "Not published on pages we could reach",
  "cost": "Contract pricing; not published. In November 2025 Yodlee and JPMorganChase amended their data access agreement to include 'a pricing structure', which suggests banks now charge aggregators.",
  "export": "No owner export; client applications pull consented data through the API.",
  "notes": "The GitHub OpenAPI repo was updated on 9 July 2026. The developer portal still shows Envestnet-era figures ($5.4T+ platform assets, ~108K advisors)."
 },
 "pricing": {
  "model": "Enterprise contract",
  "entry": "Developer sandbox, free to register",
  "top": "Negotiated",
  "notes": "yodlee.com/pricing and developer.yodlee.com/pricing returned 404 in October 2026. An older self-serve developer plan may have existed [unverified]."
 },
 "agents": {
  "mcp": "none",
  "ai_features": "Marketing copy mentions 'AI-powered personalization' and AI-driven decision-making as a focus under STG; no specific AI or agent product was found.",
  "agent_writes": "No.",
  "approval": "End-user consent at linking (FastLink); no agent gate because agents take no actions.",
  "notes": "None of the five public GitHub repos is an MCP server or agent kit."
 },
 "integration": {
  "scope": "601M+ connected consumer accounts, 19K+ data sources, >91% direct connections (yodlee.com, October 2026); the developer portal says 17K+ sources.",
  "notes": "FastLink handles consent and credentials, and the REST API returns accounts, holdings and transactions. Integration is sales-led for production."
 },
 "preference": {
  "scale": "601M+ connected consumer accounts; 14 of the top 20 US financial institutions; 7 of the top 10 US wealth firms (yodlee.com, October 2026). Named clients include Alkami, Fifth Third, M&T Bank, Navy Federal and PayPal.",
  "ratings": "Trustpilot 2.1/5 from 9 reviews, none in the last 12 months (October 2026). G2 not reachable this session.",
  "sentiment": "End-user reviews complain about dropped connections, missing and duplicate transactions, and slow support that blames the bank. Older reviews say small customers get little attention. The sample is small and comes from people using apps built on Yodlee."
 },
 "timeline": [
  {
   "date": "1999",
   "event": "Founded by Venkat Rangan and others in Sunnyvale"
  },
  {
   "date": "2000",
   "event": "Merges with VerticalOne of Atlanta"
  },
  {
   "date": "2014-10-03",
   "event": "Nasdaq IPO as YDLE, $75M at $12 a share"
  },
  {
   "date": "2015-08-10",
   "event": "Acquired by Envestnet for about $660M (reported)"
  },
  {
   "date": "2020",
   "event": "Lawmakers call for an FTC look at its sale of consumer data; class action filed against Envestnet and Yodlee"
  },
  {
   "date": "2025-06-25",
   "event": "Envestnet agrees to sell Yodlee to STG"
  },
  {
   "date": "2025-09-02",
   "event": "STG closes the acquisition; Yodlee relaunches as an independent brand under CEO Farouk Ferchichi"
  },
  {
   "date": "2025-11-07",
   "event": "Amended data access agreement with JPMorganChase, adding a pricing structure"
  }
 ],
 "relation": "source",
 "relation_note": "A possible bank and investment source, but only through a client business with a contract; it shows the aggregator model ownpurse is the opposite of: the aggregator and the app hold the data, and the person does not.",
 "borrow": "Its breadth of account types (holdings, loans, rewards) is a useful list of fact kinds a personal-finance connector should model, and its enrichment fields show what people expect a bank line to carry.",
 "avoid": "Its data-resale history (the 2020 scrutiny) and the trust cost that came with it. ownpurse should never broker the data it holds or make an intermediary the custodian of an owner's feed.",
 "sources": [
  {
   "title": "Yodlee home page",
   "url": "https://www.yodlee.com"
  },
  {
   "title": "Yodlee developer portal",
   "url": "https://developer.yodlee.com"
  },
  {
   "title": "Yodlee OpenAPI (GitHub)",
   "url": "https://github.com/Yodlee/OpenAPI"
  },
  {
   "title": "Yodlee on GitHub",
   "url": "https://github.com/Yodlee"
  },
  {
   "title": "Envestnet announces definitive agreement to sell Yodlee to STG (25 June 2025)",
   "url": "https://stg.com/news/envestnet-inc-announces-definitive-agreement-to-sell-yodlee-inc-to-stg/"
  },
  {
   "title": "Yodlee launches new era with STG partnership (2 September 2025)",
   "url": "https://stg.com/news/yodlee-launches-new-era-with-stg-partnership-poised-to-boost-open-finance-innovation/"
  },
  {
   "title": "Yodlee and JPMorganChase updated data access agreement (7 November 2025)",
   "url": "https://stg.com/news/yodlee-and-jpmorganchase-strengthen-decades-long-partnership-with-updated-data-access-agreement/"
  },
  {
   "title": "Yodlee leadership",
   "url": "https://www.yodlee.com/company/leadership"
  },
  {
   "title": "Yodlee (Wikipedia)",
   "url": "https://en.wikipedia.org/wiki/Yodlee"
  },
  {
   "title": "Yodlee on Trustpilot",
   "url": "https://www.trustpilot.com/review/yodlee.com"
  }
 ],
 "unverified": [
  "The direction of payment in the JPMorganChase pricing structure",
  "Token lifetimes and API rate limits",
  "Any current self-serve developer plan or price",
  "G2/Capterra ratings",
  "Outcome of the 2020 class action"
 ],
 "researched": "2026-10-10"
}
---

## What it is

[Yodlee](https://www.yodlee.com) is the oldest of the financial-data aggregators, with "a 25-year legacy". It connects to banks, card issuers, lenders and brokers. It sells the data to institutions and fintechs for personal finance, credit decisions, wealth management, business finance and payment verification. Its own figures, from October 2026: 601M+ connected consumer accounts, 19K+ data sources, more than 91% of them direct connections, 14 of the top 20 US financial institutions and 7 of the top 10 US wealth firms. Named clients include Fifth Third, M&T Bank, Navy Federal and PayPal.

What it does well: breadth. Few aggregators cover investment holdings, loans and rewards across as many sources, and that is why wealth platforms still use it.

## How it works

An app embeds FastLink, Yodlee's hosted linking flow. The user consents and links accounts, and the app then reads accounts, holdings and transactions over the v1.1 REST API. Yodlee publishes an [OpenAPI spec on GitHub](https://github.com/Yodlee/OpenAPI), updated in July 2026, plus JWT samples and FastLink code. It has no MCP server and no agent tooling.

## Data and access

Access is sales-led. The [developer portal](https://developer.yodlee.com) offers sandbox sign-up, but the pricing pages we tried (yodlee.com/pricing, developer.yodlee.com/pricing) returned 404, and no price list is public. The data is current state, refreshed: balances, holdings and a transaction window. Nothing is versioned. The money now runs both ways. On [7 November 2025](https://stg.com/news/yodlee-and-jpmorganchase-strengthen-decades-long-partnership-with-updated-data-access-agreement/) Yodlee and JPMorganChase amended their data access agreement to add "a pricing structure", in the same season that the largest US bank began charging aggregators.

## Agents

The site talks about "AI-powered personalization", and STG names AI-driven decision-making as a focus, but no agent product was found. An agent reaches Yodlee data only as part of a client's application.

## Customers

Banks, wealth firms, lenders and fintechs. People reach Yodlee only through apps built on it, and their reviews are about those apps' bank feeds. [Trustpilot](https://www.trustpilot.com/review/yodlee.com) shows 2.1 out of 5 from 9 reviews, none in the past year. The complaints are dropped feeds, duplicate or missing transactions and slow support, and some say small customers are ignored. The sample is too small to weigh heavily.

## History

Yodlee was founded in 1999 and merged with VerticalOne in 2000. It listed on Nasdaq on 3 October 2014 ($75M at $12 a share) and was bought by Envestnet in August 2015 for a reported $660M ([Wikipedia](https://en.wikipedia.org/wiki/Yodlee)). In 2020 lawmakers asked the FTC to look at its sale of consumer data, and a class action followed. Envestnet agreed to sell it to STG on [25 June 2025](https://stg.com/news/envestnet-inc-announces-definitive-agreement-to-sell-yodlee-inc-to-stg/), and the sale closed on [2 September 2025](https://stg.com/news/yodlee-launches-new-era-with-stg-partnership-poised-to-boost-open-finance-innovation/). Yodlee relaunched as an independent brand under CEO Farouk Ferchichi, and Envestnet kept it as a supplier for its advisors.

## For ownpurse

Yodlee is the aggregator model in full. The aggregator holds the connections, the app holds a copy, and the person holds neither. For ownpurse it is a source only through a business that holds a contract. It is not a path an owner can take alone. The useful lessons are its fact kinds, because holdings, loans and rewards all belong in a personal-finance schema, and its history, which shows why an owner-held record must never be resold or brokered.
