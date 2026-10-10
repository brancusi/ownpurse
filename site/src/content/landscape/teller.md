---
{
 "slug": "teller",
 "name": "Teller",
 "url": "https://teller.io",
 "segment": "bank-aggregator",
 "status": "shut",
 "founded": "Founded in the UK; first public launch in June 2017 (UK beta), US product under Teller, Inc. by 2020. Exact founding year unverified.",
 "hq": "San Francisco County, California governs its developer terms (Teller, Inc.); originally UK-domiciled. No street address published.",
 "backing": "Not confirmed. No funding round, investor or valuation could be verified from primary sources.",
 "tagline": "A US bank-data API that called banks' own mobile-app APIs directly, with a free developer tier of 100 live connections, withdrawn in July 2026.",
 "scope": "Teller gave apps accounts, balances, transactions, identity, account details and Zelle payments for US banks through one REST API, using Teller Connect for the user's login. It did not do categorised ledgers, documents or accounting; it was a pipe. Customers reported by email on 9 July 2026 that the API product was being withdrawn that week; signups had closed by late June 2026 and the site still shows the old pricing.",
 "domains": [
  "bank",
  "payments"
 ],
 "customers_served": [
  "developers",
  "consumer",
  "smb"
 ],
 "spectrum": {
  "custody": {
   "score": 1,
   "note": "The bank holds the authoritative record; Teller passed data through and you kept only what you pulled."
  },
  "locality": {
   "score": 0,
   "note": "Cloud API only."
  },
  "history": {
   "score": 1,
   "note": "Current transactions with stable ids, but pending items could be replaced by new records with new ids; no versions."
  },
  "openness": {
   "score": 2,
   "note": "Public docs and a self-serve API with real data free for 100 enrollments; closed source apart from example apps."
  },
  "breadth": {
   "score": 1,
   "note": "US bank accounts plus Zelle payments; nothing else."
  },
  "autonomy": {
   "score": 1,
   "note": "Polled banks at least daily and categorised automatically; never wrote to any books."
  },
  "agent_readiness": {
   "score": 2,
   "note": "Plain REST API with HTTP Basic tokens; no official MCP, though community MCP servers wrapped it."
  },
  "integration_ease": {
   "score": 0,
   "note": "Was among the easiest (free real-data tier, one-line Plaid migration), but signups closed by June 2026 and the API was withdrawn in July 2026."
  },
  "market": {
   "score": 1,
   "note": "Loved by individual developers; the company said it could not attract enough large customers."
  }
 },
 "api": {
  "public": true,
  "style": "REST over HTTPS with JSON; webhooks (enrollment.disconnected, transactions.processed, account.number_verification.processed).",
  "auth": "Mutual TLS with a Teller-issued client certificate, plus a per-enrollment access token sent as HTTP Basic; the token is useless without the app's certificate.",
  "limits": "Development environment: 100 enrollments, free, real data, never restored by deleting one. Rate limits not published. Polling at least once every 24 hours.",
  "cost": "Homepage pricing (undated, still shown October 2026): Developer free with 100 live connections; Verify $1.50 per account; Balance $0.10 per call; Transactions $0.30 per enrollment per month; Identity $1.75 per call; Enterprise custom.",
  "export": "No bulk export; whatever the developer fetched through the API was theirs to keep.",
  "notes": "Transaction amounts are signed strings, not floats. Pending items can post under a new id, so Teller advised widening sync windows by 7 to 10 days."
 },
 "pricing": {
  "model": "Usage-based per account, per call and per enrollment per month, with a free developer tier.",
  "entry": "Free: 100 live connections in the development environment.",
  "top": "Enterprise: custom, volume-based.",
  "notes": "Production required KYB. Paid customers were told on 9 July 2026 they would not be billed for future periods."
 },
 "agents": {
  "mcp": "community",
  "ai_features": "Automatic transaction categorisation and counterparty enrichment; an internal support bot (Heath, February 2023). No agent features for customers.",
  "agent_writes": "Payments over Zelle were available through the API (beta); no write path into accounting systems.",
  "approval": "None beyond the bank's own authentication and the developer's code.",
  "notes": "A community personal-finance MCP server used Teller to keep a ledger-cli/hledger file in sync with banks until the shutdown."
 },
 "integration": {
  "scope": "Homepage claimed more than 7,000 US institutions with instant connection (5,000 announced October 2023), plus same-day ACH microdeposit verification for the rest.",
  "notes": "Sidecar (October 2023) offered Plaid-compatible migration. Connections came from reverse-engineered bank mobile-app APIs routed through Teller's own LTE edge network (Telnet), not from bank partnerships."
 },
 "preference": {
  "scale": "No customer counts published. Homepage logos: Pipe, Brex, Capchase, BrightMoney, Airbase.",
  "ratings": "No G2, Capterra or Trustpilot profile found with reviews.",
  "sentiment": "Developers praised the experience and free tier ('frankly a better dev experience', HN, June 2026) and missed coverage for some banks (Bilt). After July 2026 the sentiment is frustration at one week's notice and a domain left up with no notice."
 },
 "timeline": [
  {
   "date": "2016-05",
   "event": "Publishes TAuth, an argument against OAuth 2.0 for banking APIs."
  },
  {
   "date": "2017-06",
   "event": "UK launch: 'Teller – API for your bank account' reaches the front of Hacker News (609 points)."
  },
  {
   "date": "2020-06",
   "event": "Introduces Telnet, its edge network for bank traffic, and a migration path for apps leaving other providers."
  },
  {
   "date": "2021-07",
   "event": "Announces SOC 2 Type 2 (achieved December 2020)."
  },
  {
   "date": "2021-09",
   "event": "Launches real-time payments on Zelle."
  },
  {
   "date": "2023-10",
   "event": "Announces 5,000+ instant bank connections, Sidecar (Plaid migration) and Bridge (Zelle for neobanks)."
  },
  {
   "date": "2026-06",
   "event": "Developers report the sign-up and sales links are gone."
  },
  {
   "date": "2026-07-09",
   "event": "Customers receive an email: the API product will be withdrawn that week; a lower-cost new version is in the works."
  }
 ],
 "relation": "cautionary-tale",
 "relation_note": "Teller was the cleanest way for one person to pull their own US bank transactions into a local ledger, and then it disappeared with a week's notice. ownpurse should treat any aggregator as a replaceable source, never as the record.",
 "borrow": "Signed-string amounts, stable transaction ids with an explicit note on when they break, a free real-data tier sized for one person, and honest guidance on pending-to-posted drift.",
 "avoid": "Building a person's bank history on a single aggregator's goodwill. Everything ownpurse pulled from Teller should already sit in the owner's log; the connector should fail loudly and leave the record intact.",
 "sources": [
  {
   "title": "Teller homepage (pricing and coverage)",
   "url": "https://teller.io/"
  },
  {
   "title": "Teller docs: environments",
   "url": "https://teller.io/docs/guides/environments"
  },
  {
   "title": "Teller docs: authentication",
   "url": "https://teller.io/docs/api/authentication"
  },
  {
   "title": "Teller docs: transactions",
   "url": "https://teller.io/docs/api/account/transactions"
  },
  {
   "title": "Teller docs: webhooks",
   "url": "https://teller.io/docs/api/webhooks"
  },
  {
   "title": "Teller blog",
   "url": "https://blog.teller.io/"
  },
  {
   "title": "Teller developer terms",
   "url": "https://teller.io/legal/developer/terms"
  },
  {
   "title": "HN: Teller.io shuts down at the end of the week (2026-07-09)",
   "url": "https://news.ycombinator.com/item?id=48841633"
  },
  {
   "title": "GitHub: Teller API being discontinued (email text, 2026-07-09)",
   "url": "https://github.com/zrabin/personal-finance-mcp/issues/1"
  },
  {
   "title": "GitHub: tellerhq/examples#6, sign-ups gone",
   "url": "https://github.com/tellerhq/examples/issues/6"
  },
  {
   "title": "HN: Teller – API for your bank account (2017)",
   "url": "https://news.ycombinator.com/item?id=14605342"
  }
 ],
 "unverified": [
  "No primary public announcement of the shutdown on teller.io; status rests on the customer email reproduced on HN and GitHub.",
  "Whether any existing enrollments still return data after July 2026.",
  "Founding year, founders' full names and any funding or investors.",
  "Date of the homepage pricing figures.",
  "The 7,000+ institution count (homepage claim, undated)."
 ],
 "researched": "2026-10-10"
}
---

## What it is

Teller was a US bank-data API: one REST interface to accounts, balances, transactions, identity, account and routing numbers and, later, Zelle payments. Its [homepage](https://teller.io/) still advertises more than 7,000 institutions and a free developer tier of 100 live connections. On 9 July 2026 customers received an email saying Teller would withdraw its API product that week because, 'while very popular with individual developers', it had not attracted enough large customers. The text was posted by several recipients on [Hacker News](https://news.ycombinator.com/item?id=48841633) and [GitHub](https://github.com/zrabin/personal-finance-mcp/issues/1). Sign-ups had already vanished by late June ([tellerhq/examples#6](https://github.com/tellerhq/examples/issues/6)). There is no notice on teller.io itself, so this dossier relies on those customer reports.

## How it works

Teller did not partner with banks. Its [blog](https://blog.teller.io/) says it reverse-engineered each bank's mobile app and called the same private APIs, routing traffic through its own network of LTE edge nodes (Telnet) so it looked like ordinary app traffic. The founder argued on [Hacker News in 2017](https://news.ycombinator.com/item?id=14605342) that this gave structured data, fewer 2FA prompts and fewer breakages than screen scraping. Critics there raised credential storage and liability. A user connected through Teller Connect; the app then held an access token that only worked together with the app's Teller-issued client certificate, over [mutual TLS](https://teller.io/docs/api/authentication).

## Data and access

Three [environments](https://teller.io/docs/guides/environments): sandbox (free, simulated), development (free, real data, 100 enrollments) and production (paid, KYB). The [transactions endpoint](https://teller.io/docs/api/account/transactions) returns signed string amounts, `posted` or `pending` status, a running balance on posted items, and enrichment (category, counterparty). Ids are stable except when a pending item changes enough on posting that Teller cannot match it, in which case it appears as a new record. Teller told you to widen sync windows by 7 to 10 days for this. Webhooks announced processed transactions and broken enrollments. Pricing on the homepage was per use: $1.50 per verified account, $0.10 per balance call, $0.30 per enrollment per month for transactions, $1.75 per identity call.

## Agents

There was no official agent interface, but the API's simplicity made it a favourite for personal tools. One community MCP server kept a ledger-cli/hledger file in sync with the owner's banks through Teller, until the shutdown made its author go back to manual downloads. Open-source budgeting projects (Actual, Sure) were still getting requests for a Teller integration in October 2026, from people who had not seen the shutdown.

## Customers

Homepage logos include Pipe, Brex, Capchase, BrightMoney and Airbase. No review-site profile has meaningful volume. On forums, developers preferred Teller's experience to Plaid's but hit coverage gaps.

## History

Teller began in the UK. The founders could not get a bank to sponsor a Faster Payments business, so they started calling their own banks' APIs instead. The UK product launched in June 2017. By 2020 Teller was a US company (Teller, Inc., California law), with SOC 2 Type 2 in 2021, Zelle payments in 2021 and Sidecar, a Plaid migration path, in October 2023. Funding was never confirmed in a primary source.

## For ownpurse

Teller is the best evidence that individuals want their own bank data through a clean API, and that a business selling that to individuals at $0 struggles. It is also why ownpurse keeps the record on the owner's machine. Anything a person pulled through Teller and wrote into their own log survived the shutdown. Anything left in a vendor's cache did not. Borrow its conventions (string amounts, explicit id-stability rules, a free tier sized for one person) and design every bank connector so that losing the provider loses only future syncs, never the past.
