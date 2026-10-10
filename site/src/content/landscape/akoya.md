---
{
 "slug": "akoya",
 "name": "Akoya",
 "url": "https://akoya.com",
 "segment": "bank-aggregator",
 "status": "active",
 "founded": "2020 (Akoya's about page: 'Established 2020'; Dealroom says it was developed inside Fidelity from 2018, unverified)",
 "hq": "Boston, Massachusetts (CB Insights); hubs in Boston, New York City and Raleigh (Akoya about page, 2026)",
 "backing": "Spun out of FMR (Fidelity) and announced on 20 February 2020 as jointly owned by Fidelity Investments, The Clearing House Payments Company and 11 large US banks (Bank of America, Capital One, Citi, Huntington, JPMorgan Chase, KeyBank, PNC, TD, Truist, US Bank, Wells Fargo). Funding amounts are not published; a Series C dated 31 October 2025 appears only in an AI-compiled review page and is unverified.",
 "tagline": "A bank-owned network that lets fintechs pull consumer-permissioned account data from US financial institutions over one FDX-aligned API, without screen scraping.",
 "scope": "Akoya routes data requests from apps (data recipients) to banks, brokerages and other institutions (data providers) through one API: accounts, balances, transactions, investments and tax lots, customer details, statements as PDFs, tax forms and payment account details. Consumers sign in at their own institution through OAuth; no credentials are shared. It is a pipe, not a ledger: it does not categorise, reconcile or keep books, and it is not open to an individual who wants their own data.",
 "domains": [
  "bank",
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
   "note": "The bank holds the record and the app receiving it keeps a copy; the account owner holds nothing directly."
  },
  "locality": {
   "score": 0,
   "note": "A hosted network between institutions and recipients; nothing runs on the owner's machine."
  },
  "history": {
   "score": 1,
   "note": "Returns the institution's transactions and statement PDFs as they stand; no versioning or as-of view."
  },
  "openness": {
   "score": 1,
   "note": "Docs and a sandbox are public and the API follows FDX, but production needs a signed agreement and security screening."
  },
  "breadth": {
   "score": 2,
   "note": "Bank, card, brokerage, statements, tax forms and payment details across thousands of US institutions; no books."
  },
  "autonomy": {
   "score": 1,
   "note": "Data flows to recipients unattended once consented, but nothing is posted to any books."
  },
  "agent_readiness": {
   "score": 1,
   "note": "A documented REST API with webhooks, but gated to contracted businesses; no MCP or agent interface, only blog posts about agents (Aug and Sep 2026)."
  },
  "integration_ease": {
   "score": 1,
   "note": "Self-serve sandbox, but production means contracting, security review, unpublished pricing and possible setup fees."
  },
  "market": {
   "score": 4,
   "note": "Owned by and serving the largest US banks and enterprise fintechs and aggregators."
  }
 },
 "api": {
  "public": true,
  "style": "REST, FDX-aligned, with OpenAPI specs; Token API, data APIs (accounts, balances, transactions, investments, customers, payments, statements, tax) and service APIs (apps, notifications/webhooks, consent)",
  "auth": "OAuth with the consumer authenticating at their institution; Akoya Token API issues ID and refresh tokens, with a revoke endpoint; separate service tokens for service APIs",
  "limits": "Rate limits are not stated on the public docs index; production access only after a request, security screening and a signed agreement",
  "cost": "Free sandbox. Production Standard tier for under 10,000 monthly connections with no published price; Enterprise for 10,000+ at custom pricing; a setup or implementation fee may apply (akoya.com/pricing, 2026)",
  "export": "None for the account owner; recipients get data per consent through the API",
  "notes": "Docs list Token API v2.2.0, Akoya APIs v3.0.0, Apps Management v2.0.0, Notifications v1.1.0; the Consent API shows v1.0.2 in one list and v1.1.0 in another. Tokenized account numbers (TAN) are supported for payments."
 },
 "pricing": {
  "model": "Per-connection volume tiers for data recipients, plus possible setup fees",
  "entry": "Free sandbox; Standard (under 10,000 monthly connections) price not published",
  "top": "Enterprise (10,000+ monthly connections), custom pricing with a dedicated customer success manager",
  "notes": "No dollar figures are public as of 2026-10-10. The CFPB's 1033 reconsideration (ANPR, 22 August 2025) asks whether institutions may charge fees for data access, which could change economics across the network."
 },
 "agents": {
  "mcp": "none",
  "ai_features": "None in the product; Akoya's blog discusses AI agents' access to bank data (1 September 2026, 12 August 2026).",
  "agent_writes": "No writes to books; the Payments API returns account and routing details (or tokenized numbers) for a recipient to initiate payments elsewhere.",
  "approval": "Consumer consent at the institution, revocable; the Consent API reports consent changes. No per-write human gate since nothing is written.",
  "notes": "An agent can only reach Akoya through a contracted data recipient app."
 },
 "integration": {
  "scope": "Akoya's home page claims 4.5K+ institutions and 7.5K+ apps powered; its about page says more than 4,300 institutions (both 2026, undated). Logos include Chase, Wells Fargo, Bank of America, Citi, Capital One, PNC, TD, US Bank, Fidelity, Morningstar, Yodlee, Fiserv, Jack Henry and eMoney.",
  "notes": "Aggregators and core providers sit on the network, so many apps reach Akoya indirectly through an aggregator. Implementation details and test users vary by data provider."
 },
 "preference": {
  "scale": "4.5K+ institutions and 7.5K+ apps (home page, 2026); 130+ employees (about page, 2026)",
  "ratings": "No G2, Capterra or Trustpilot ratings found; an RFP.wiki page (AI-compiled) also reports no verifiable review-site ratings.",
  "sentiment": "Secondary sources credit Akoya with secure API-only connections, better conversion and data consistency than screen scraping, and a credible consortium model; they note US-only coverage, a long tail of smaller banks to check, and production onboarding that depends on contracting. Direct user reviews were not found."
 },
 "timeline": [
  {
   "date": "2020-02-20",
   "event": "Announced as a spin-out from FMR, jointly owned by Fidelity, The Clearing House and 11 US banks"
  },
  {
   "date": "2024-10-22",
   "event": "CFPB issues the 1033 personal financial data rights final rule, which favours API access through recognised standards such as FDX"
  },
  {
   "date": "2025-08-22",
   "event": "CFPB publishes an ANPR reconsidering the 1033 rule, including fees for data access"
  },
  {
   "date": "2026",
   "event": "Network claims 4.5K+ institutions and 7.5K+ apps (home page)"
  },
  {
   "date": "2026-08-12",
   "event": "Blog: 'Your customers' AI agents are coming. Will you be able to see them?'"
  },
  {
   "date": "2026-09-01",
   "event": "Blog: 'AI agents have access. Can they trust the data?'"
  }
 ],
 "relation": "infrastructure",
 "relation_note": "Akoya is plumbing that carries US bank data to businesses; ownpurse cannot use it directly for one person's own books, but the aggregators and apps an owner already uses may get their data through it, and its FDX data model shapes what bank facts look like.",
 "borrow": "Treat consent as a first-class, revocable fact with its own change events (Akoya's Consent API and token revocation); use the FDX account and transaction shapes as a reference for ownpurse's bank facts; tokenized account numbers as a way to name an account without holding the real number.",
 "avoid": "A network open only to contracted businesses, where the account owner cannot hold a token to their own data; unpublished pricing.",
 "sources": [
  {
   "title": "Akoya home page",
   "url": "https://akoya.com"
  },
  {
   "title": "Akoya about",
   "url": "https://akoya.com/about"
  },
  {
   "title": "Akoya for fintechs",
   "url": "https://akoya.com/fintechs"
  },
  {
   "title": "Akoya pricing",
   "url": "https://akoya.com/pricing"
  },
  {
   "title": "Akoya developer docs",
   "url": "https://docs.akoya.com"
  },
  {
   "title": "Akoya blog",
   "url": "https://akoya.com/blog"
  },
  {
   "title": "Business Wire: Financial industry to give consumers more control over their data (2020-02-20)",
   "url": "https://www.businesswire.com/news/home/20200220005175/en/Financial-Industry-to-Give-Consumers-More-Control-Over-Their-Data"
  },
  {
   "title": "CB Insights: Akoya",
   "url": "https://www.cbinsights.com/company/akoya-1"
  },
  {
   "title": "RFP.wiki: Akoya (AI-compiled review)",
   "url": "https://www.rfp.wiki/financial-services-banking-fintech/open-banking-platforms/akoya"
  },
  {
   "title": "CFPB 1033 final rule",
   "url": "https://www.consumerfinance.gov/rules-policy/final-rules/required-rulemaking-on-personal-financial-data-rights/"
  },
  {
   "title": "CFPB 1033 reconsideration",
   "url": "https://www.consumerfinance.gov/rules-policy/rules-under-development/personal-financial-data-rights-reconsideration/"
  }
 ],
 "unverified": [
  "Series C dated 31 October 2025 (only in an AI-compiled RFP.wiki page; no amount or valuation)",
  "API v2 deprecated February 2026 (RFP.wiki only)",
  "Whether Akoya stores or copies data in transit (claimed by third-party profiles; not stated on Akoya's pages fetched)",
  "Dollar prices for the Standard tier",
  "Rate limits and token lifetimes",
  "Development inside Fidelity from 2018 (Dealroom)",
  "Exact current owner list (sources say 11 or 12 institutions plus Fidelity and The Clearing House)",
  "Reddit and review-site sentiment: the web search budget ran out, so forums were not searched"
 ],
 "researched": "2026-10-10"
}
---

## What it is

Akoya is a data access network owned by the banks it connects. It was [announced on 20 February 2020](https://www.businesswire.com/news/home/20200220005175/en/Financial-Industry-to-Give-Consumers-More-Control-Over-Their-Data) as a spin-out from Fidelity's parent, FMR, jointly owned by Fidelity, The Clearing House and 11 large US banks. Its pitch to institutions is that apps should reach their customers' data through an API the bank controls instead of by screen scraping with the customer's password. [Akoya's home page](https://akoya.com) claims 4.5K+ institutions and 7.5K+ apps; its [about page](https://akoya.com/about) says more than 4,300 institutions. Both are undated 2026 pages.

## How it works

A fintech (a data recipient) signs one agreement and integrates one API. When a customer connects an account, they sign in at their own bank through OAuth; Akoya issues ID and refresh tokens and routes requests to the institution. The [developer docs](https://docs.akoya.com) cover accounts, balances, transactions, investments with tax lots, customers, payments (account and routing details, or tokenized numbers), statements as PDFs and tax forms, plus webhooks and a Consent API that reports when a customer changes or revokes access. The API follows the FDX standard. Aggregators and core processors such as Yodlee, Fiserv and Jack Henry appear among its logos, so much traffic reaches apps through an intermediary.

What it does well: it removes shared credentials, gives the bank a view of who holds access, and gives the customer a revocation path. That is a real improvement over scraping.

## Data and access

The sandbox is [free and self-service](https://akoya.com/fintechs). Production needs a request, a security screening and a signed agreement. [Pricing](https://akoya.com/pricing) has two tiers by monthly connections: Standard under 10,000 and Enterprise above, with no dollar figures published and a possible setup fee. There is no route for an individual to get a token to their own accounts. The account owner has no export; data goes to the app they consented to.

## Agents

There is no MCP server or agent interface. Akoya's [blog](https://akoya.com/blog) has taken up the subject: "Your customers' AI agents are coming" (12 August 2026) and "AI agents have access. Can they trust the data?" (1 September 2026), both written for banks deciding how to see and govern agent access. An agent can reach Akoya only through a contracted app.

## Customers

The network's customers are institutions and the fintechs, wealth platforms, lenders and aggregators that receive data. No G2, Capterra or Trustpilot ratings were found. Secondary write-ups praise its API-only reliability and consortium backing and note its US-only coverage and contract-gated production.

## History

Akoya came out of Fidelity's own work on replacing scraping, then became the banks' collective answer to aggregators. The CFPB's [1033 final rule](https://www.consumerfinance.gov/rules-policy/final-rules/required-rulemaking-on-personal-financial-data-rights/) (22 October 2024, effective 17 January 2025) pushed the industry toward standard APIs; the CFPB then opened a [reconsideration](https://www.consumerfinance.gov/rules-policy/rules-under-development/personal-financial-data-rights-reconsideration/) on 22 August 2025 that asks, among other things, whether institutions may charge fees for access. How that ends will shape Akoya's economics and who pays.

## For ownpurse

Akoya is infrastructure ownpurse will rarely touch directly. Its value to ownpurse is as a reference: FDX shapes for accounts and transactions, consent as a revocable object with its own events, and tokenized account numbers. It is also the clearest example of the gap ownpurse fills. The network gives banks and apps a governed pipe, but the person whose money it is cannot hold a token to their own data. ownpurse should read bank data from what the owner can reach (exports, OFX files, personal APIs like Akahu, SimpleFIN) and treat consent as a fact in the record.
