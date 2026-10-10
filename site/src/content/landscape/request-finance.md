---
{
 "slug": "request-finance",
 "name": "Request Finance",
 "url": "https://www.request.finance",
 "segment": "crypto-accounting",
 "status": "active",
 "founded": "2020",
 "hq": "Paris, France",
 "backing": "Private company built on the Request Network protocol; Request Network describes itself as a community-owned foundation, not venture-funded, with code running since 2017. Request Finance's own funding was not confirmed.",
 "tagline": "Crypto and stablecoin invoicing, payables, payroll and expenses for web3 businesses, synced to Xero and QuickBooks.",
 "scope": "Request Finance lets a business issue invoices that can be paid in crypto or fiat, pay bills and salaries in batches from its own wallets, track expenses and, more recently, offers stablecoin accounts and cards. Payment requests are recorded on the Request Network protocol, and the app syncs invoices and payments to Xero and QuickBooks. It is not a ledger or a subledger: cost basis and full crypto accounting are left to tools like Cryptio, which integrates with it.",
 "domains": [
  "invoicing",
  "payments",
  "payroll",
  "spend",
  "crypto",
  "integration"
 ],
 "customers_served": [
  "freelancer",
  "smb",
  "mid-market"
 ],
 "spectrum": {
  "custody": {
   "score": 2,
   "note": "Funds stay in the customer's own wallets and payment requests sit on a public protocol, but invoice records and app state live in Request Finance's cloud."
  },
  "locality": {
   "score": 0,
   "note": "A web app only; the protocol is open, but the product cannot be self-hosted."
  },
  "history": {
   "score": 2,
   "note": "On-chain requests and payments are immutable and public; invoice state in the app (accepted, canceled) is current state, not documented as versioned."
  },
  "openness": {
   "score": 2,
   "note": "The underlying protocol libraries are MIT open source; the app is closed with a documented public REST API."
  },
  "breadth": {
   "score": 2,
   "note": "Covers AR, AP, payroll, expenses and accounting sync for crypto and fiat, but no ledger, banks or documents beyond invoices."
  },
  "autonomy": {
   "score": 1,
   "note": "A person signs payments from a wallet; the API can create invoices and payroll items, but payment happens in the app."
  },
  "agent_readiness": {
   "score": 2,
   "note": "A public REST API with OAuth, rate limits stated, webhooks and an llms.txt docs index; no MCP found."
  },
  "integration_ease": {
   "score": 3,
   "note": "Free sign-up gives API keys at once and docs are public; OAuth apps and webhooks need support to whitelist the account."
  },
  "market": {
   "score": 1,
   "note": "Web3 freelancers, DAOs and small to mid-sized crypto businesses."
  }
 },
 "api": {
  "public": true,
  "style": "REST, JSON, Bearer auth, X-Network header (test or live); webhooks for invoice events",
  "auth": "API keys for development; OAuth 2 (Auth0, authorization code or client credentials) for production; access tokens last 24 hours",
  "limits": "500 calls an hour unauthenticated, 5,000 authenticated, 15,000 with OAuth; some endpoints stricter; Retry-After headers (docs, 2026-10)",
  "cost": "No API fees stated; account sign-up is free. Payment fees on the app were not confirmed. The underlying Request Network API charges 0.9% per transaction, capped at $500.",
  "export": "PDF download of invoices; invoices and payments sync to Xero and QuickBooks; a bulk export format was not confirmed",
  "notes": "Endpoints cover invoices (off-chain, then converted to on-chain requests), payroll (an invoice with format rnf_salary, created by API and paid in the app), organizations, clients, currencies and webhooks. Docs are on GitBook with an llms.txt index and .md versions of each page."
 },
 "pricing": {
  "model": "Freemium SaaS with paid business plans (details not confirmed)",
  "entry": "Free account (sign-up and API keys); Product Hunt listed it as Free in 2021",
  "top": "Not confirmed; request.finance blocked automated access on 2026-10-10",
  "notes": "The pricing page could not be read. The 0.9% fee capped at $500 belongs to the separate Request Network API, not necessarily the Request Finance app."
 },
 "agents": {
  "mcp": "none",
  "ai_features": "No AI or agent features were confirmed. The docs ship an llms.txt index and Markdown pages, which help agents read the API.",
  "agent_writes": "Yes, through the API: create invoices, payroll payments and clients, and convert them to on-chain requests. Paying is done in the app.",
  "approval": "Payments need a person to sign from a wallet in the app (and, for multisig, a Safe threshold). That gate comes from wallet custody, not a designed review step.",
  "notes": "Its good agent surface is the API's split between creating a request and paying it, which lets an agent prepare the work and a person pay it."
 },
 "integration": {
  "scope": "Xero and QuickBooks accounting sync; 300+ cryptocurrencies, 20+ blockchains and 20+ fiat currencies (LinkedIn); Safe multisig; Cryptio lists it as an integration",
  "notes": "Request Network chains: Ethereum, BNB Chain, Base, Polygon, Arbitrum, Optimism and Tron."
 },
 "preference": {
  "scale": "$1.5B processed and payments to 190+ countries (LinkedIn, 2026-10); at the 2021 Product Hunt launch the makers claimed 650+ regular users, $122M+ processed and named MakerDAO, Aave and The Graph; 11-50 employees",
  "ratings": "Trustpilot 4.4/5 from 12 reviews, all from Feb to May 2023, none in the last 12 months (2026-10-10); Product Hunt: 111 upvotes, no reviews; G2 and Capterra not reachable",
  "sentiment": "Reviewers praise how simple it is, getting paid or reimbursed in stablecoins, hours saved each month, the Xero sync and responsive support. No complaints appear in the visible reviews, but the sample is small and old."
 },
 "timeline": [
  {
   "date": "2017",
   "event": "Request Network protocol code begins running (per request.network)"
  },
  {
   "date": "2020",
   "event": "Request Finance founded in Paris (LinkedIn)"
  },
  {
   "date": "2021",
   "event": "Launched on Product Hunt: invoicing, payroll and expenses for web3"
  },
  {
   "date": "2023",
   "event": "Trustpilot reviews cluster, praising the Xero sync"
  },
  {
   "date": "2026",
   "event": "Positions around stablecoin accounts, cards and accrual accounting; the Request Network adds Safe batch payouts and wallet screening"
  }
 ],
 "relation": "source",
 "relation_note": "A source, not a rival: its invoices and payments are documents a connector can read, and they already flow into Xero, which ownpurse reads. Its split between request and payment also matches ownpurse's split between agent and person.",
 "borrow": "Create a request first and pay it as a separate step; publish limits and Retry-After headers; an llms.txt index with Markdown versions of every docs page; ids that tie an invoice to its on-chain payment, so links can be checked.",
 "avoid": "Gating webhooks and OAuth apps behind manual whitelisting; letting the accounting sync be the only record of what happened.",
 "sources": [
  {
   "title": "Request Finance API: welcome",
   "url": "https://docs.request.finance/welcome.md"
  },
  {
   "title": "Request Finance API: getting started",
   "url": "https://docs.request.finance/getting-started.md"
  },
  {
   "title": "Request Finance API: going live (OAuth)",
   "url": "https://docs.request.finance/going-live.md"
  },
  {
   "title": "Request Finance API: rate limits",
   "url": "https://docs.request.finance/rate-limits.md"
  },
  {
   "title": "Request Finance API: webhooks",
   "url": "https://docs.request.finance/webhooks.md"
  },
  {
   "title": "Request Finance API: invoices",
   "url": "https://docs.request.finance/invoices.md"
  },
  {
   "title": "Request Finance API: payroll",
   "url": "https://docs.request.finance/salaries.md"
  },
  {
   "title": "Request Finance docs index (llms.txt)",
   "url": "https://docs.request.finance/llms.txt"
  },
  {
   "title": "Request Network home (fees, structure)",
   "url": "https://www.request.network"
  },
  {
   "title": "Request Network on GitHub",
   "url": "https://github.com/RequestNetwork"
  },
  {
   "title": "Request Finance on LinkedIn",
   "url": "https://www.linkedin.com/company/request-finance/"
  },
  {
   "title": "Request Finance on Product Hunt",
   "url": "https://www.producthunt.com/products/request-finance"
  },
  {
   "title": "Trustpilot: request.finance",
   "url": "https://www.trustpilot.com/review/request.finance"
  },
  {
   "title": "Cryptio integrations (lists Request Finance)",
   "url": "https://cryptio.co/integrations"
  }
 ],
 "unverified": [
  "Request Finance plan prices and payment fees (request.finance returned 403)",
  "Funding: any venture rounds for Request Finance, and the Request Network 2017 token sale amount",
  "Founders' names",
  "The depth of the Xero and QuickBooks sync (which objects, one-way or two-way)",
  "Any AI, agent or MCP features on the current site",
  "Whether the stablecoin accounts and cards run through partners, and which ones",
  "G2 and Capterra ratings"
 ],
 "researched": "2026-10-10"
}
---

## What it is

Request Finance is a finance app for web3 businesses: invoicing, bills, payroll and expenses in crypto, stablecoins or fiat, with sync to Xero and QuickBooks. It was founded in Paris in 2020 ([LinkedIn](https://www.linkedin.com/company/request-finance/)) on the Request Network protocol. Request Network describes itself as community-owned and says its code has run since 2017 ([request.network](https://www.request.network)). Its LinkedIn page now says it has processed $1.5B, with payments to 190+ countries across 20+ chains and 300+ tokens. It also describes stablecoin accounts, corporate cards and accrual accounting.

## How it works

An invoice is created off-chain, then converted into an on-chain request with a predefined schema ([invoices](https://docs.request.finance/invoices.md)). The buyer pays from a wallet, and the request records that it was paid without anyone polling the chain. A payroll item is the same object with format `rnf_salary`. You can create it through the API, but you pay it in the app ([payroll](https://docs.request.finance/salaries.md)). Funds never pass through Request: they move from wallet to wallet, and multisig payouts go through Safe.

## Data and access

The [API](https://docs.request.finance/getting-started.md) is REST and JSON. API keys come with a free account; production apps move to OAuth 2 (authorization code or client credentials), with 24-hour access tokens and an `X-Network` header for test or live ([going live](https://docs.request.finance/going-live.md)). [Rate limits](https://docs.request.finance/rate-limits.md) are 500, 5,000 and 15,000 calls an hour (unauthenticated, API key, OAuth), with Retry-After on blocks. [Webhooks](https://docs.request.finance/webhooks.md) report invoices created, accepted, canceled, rejected and paid, but support must whitelist your account first. The docs publish an [llms.txt](https://docs.request.finance/llms.txt) index. The pricing page blocked automated access on 2026-10-10, so plan prices are unconfirmed. The Request Network API, a separate product, charges 0.9% per transaction, capped at $500.

## Agents

No MCP server or AI feature was confirmed. Still, the API suits an agent well. The agent can create invoices and payroll, but money moves only when a person signs from a wallet in the app. That gate comes from the wallet holding the funds, not from a designed review step, yet it does what ownpurse's approval does.

## Customers

Its customers are web3 freelancers, DAOs and crypto companies. At the 2021 [Product Hunt](https://www.producthunt.com/products/request-finance) launch the makers claimed 650+ regular users and $122M+ processed, naming MakerDAO, Aave and The Graph. [Trustpilot](https://www.trustpilot.com/review/request.finance) shows 4.4 from 12 reviews, all from 2023. They praise how simple it is, stablecoin payouts, the Xero sync and responsive support. It is a small and old sample.

## History

Request began as a protocol for payment requests and built Request Finance as its main business app around 2020 and 2021. By 2026 the protocol had added Safe batch payouts and wallet screening (blog posts from June 2026), and the app was pitched around stablecoin accounts and cards. Cryptio lists Request Finance as an integration, which shows where each sits: Request handles payments and invoices, Cryptio does the accounting.

## For ownpurse

Request Finance is a source. Its invoices, payroll and on-chain payment ids are documents a connector can read, and the same payments already land in Xero, so ownpurse can link the two and check them to the cent. Worth borrowing: request and payment as separate steps, published limits, and docs written for machines (llms.txt and .md pages).
