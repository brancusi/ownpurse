---
{
 "slug": "gocardless-bank-account-data",
 "name": "GoCardless Bank Account Data",
 "url": "https://gocardless.com/bank-account-data/",
 "segment": "bank-aggregator",
 "status": "pivoted",
 "founded": "Nordigen founded in Riga, Latvia, 2016 [unverified]; GoCardless founded 24 February 2011",
 "hq": "London, United Kingdom (GoCardless); Nordigen from Riga, Latvia",
 "backing": "GoCardless raised $312M at a $2.1B valuation in 2022 and was acquired by Mollie, announced 11 December 2025 (reported at €1.05B, mostly in stock) and completed 1 September 2026. It now operates as 'GoCardless, a Mollie company'. GoCardless announced the Nordigen acquisition on 1 July 2022.",
 "tagline": "A PSD2 account-information API for EEA and UK banks, formerly the free Nordigen feed, closed to new sign-ups since July 2025.",
 "scope": "It reads account details, balances and up to 24 months of transactions from banks in the EEA and the UK, on the account holder's consent, through a small self-serve REST API. Its free tier made it the bank feed of choice for self-hosted budgeting tools. It does not initiate payments (that is GoCardless's Pay by Bank) and it keeps no ledger. It has not accepted new accounts since July 2025; existing accounts still work.",
 "domains": [
  "bank",
  "personal",
  "integration"
 ],
 "customers_served": [
  "developers",
  "consumer",
  "smb"
 ],
 "spectrum": {
  "custody": {
   "score": 1,
   "note": "The bank is the source and GoCardless holds the consent; whoever calls the API gets current data and must keep their own copy."
  },
  "locality": {
   "score": 1,
   "note": "Hosted, but a person could call it from their own machine with their own secrets and no intermediary, which is why self-hosters used it."
  },
  "history": {
   "score": 1,
   "note": "Up to 24 months of transactions per consent (90 days by default); no versioning."
  },
  "openness": {
   "score": 2,
   "note": "An open, documented API with a free tier; the code is closed and new sign-ups are shut."
  },
  "breadth": {
   "score": 1,
   "note": "Bank accounts only: details, balances, transactions."
  },
  "autonomy": {
   "score": 0,
   "note": "Read-only data access; writes nothing."
  },
  "agent_readiness": {
   "score": 2,
   "note": "A small, clean REST API with token auth that an agent could drive; no MCP or agent features."
  },
  "integration_ease": {
   "score": 1,
   "note": "Was the easiest bank feed in Europe; since July 2025 only existing accounts can use it."
  },
  "market": {
   "score": 1,
   "note": "Developers, self-hosters and small fintechs; paying, verified customers by arrangement."
  }
 },
 "api": {
  "public": true,
  "style": "REST (JSON), /api/v2: institutions, agreements, requisitions, accounts",
  "auth": "secret_id and secret_key from the portal, exchanged at /token/new for a refresh token (30 days) and at /token/refresh for an access token (24 hours)",
  "limits": "Banks may limit calls to as few as 4 a day per account, per endpoint (details, balances, transactions); 429 with a reset header",
  "cost": "Free tier of up to 50 bank connections a month (per Actual Budget's docs); verification and higher volumes for paying customers, prices by contact",
  "export": "No bulk export; the caller fetches transactions per account within the consent window and must store them itself.",
  "notes": "Default agreement is 90 days of history and 90 days of access; max_historical_days and access_valid_for_days can be set (up to 24 months of history per the docs). Coverage is all EEA countries under PSD2, plus the UK; in 2022 Nordigen counted 2,300+ banks in 31 countries."
 },
 "pricing": {
  "model": "Freemium, then contract",
  "entry": "Free: up to 50 bank connections a month (existing accounts only since July 2025)",
  "top": "Negotiated for verified, paying customers",
  "notes": "GoCardless's pricing page lists only payment plans; Bank Account Data has no public price list."
 },
 "agents": {
  "mcp": "none",
  "ai_features": "None.",
  "agent_writes": "No; account information only.",
  "approval": "Bank-side consent (strong customer authentication) at each requisition, which lapses after the access period (90 days by default), so a person must reconnect periodically.",
  "notes": "The periodic re-consent is a built-in human checkpoint, but it gates reads, not writes."
 },
 "integration": {
  "scope": "EEA banks under PSD2 plus the UK; 2,300+ banks in 31 countries (Nordigen, July 2022); GoCardless cites >100 UK banks covering 99% of accounts.",
  "notes": "Integrated by Actual Budget, Firefly III's data importer and many hobby projects. Actual now lists it as 'not accepting new accounts' and points to Enable Banking for European banks."
 },
 "preference": {
  "scale": "GoCardless as a whole: over 350,000 businesses across 30+ markets for the combined Mollie group (September 2026). No Bank Account Data user count found.",
  "ratings": "GoCardless Trustpilot 2.3/5 from 2,492 reviews (October 2026), almost all about direct debit payments, not Bank Account Data.",
  "sentiment": "Self-hosters valued the free tier and the simple API; the complaints were rate limits introduced later (Firefly III had to add handling for them), limits that did not reset, the 90-day re-consent, and then the closure to new users."
 },
 "timeline": [
  {
   "date": "2011-02-24",
   "event": "GoCardless founded in London"
  },
  {
   "date": "2016",
   "event": "Nordigen founded in Riga [unverified]"
  },
  {
   "date": "2022-07-01",
   "event": "GoCardless announces it will acquire Nordigen (2,300+ banks, 31 countries) and promises free open-banking connectivity at scale"
  },
  {
   "date": "2022",
   "event": "GoCardless raises $312M at a $2.1B valuation"
  },
  {
   "date": "2025-07",
   "event": "GoCardless stops accepting new Bank Account Data accounts; existing accounts continue"
  },
  {
   "date": "2025-12-11",
   "event": "Mollie agrees to acquire GoCardless"
  },
  {
   "date": "2026-09-01",
   "event": "Mollie completes the acquisition; GoCardless becomes 'a Mollie company'"
  },
  {
   "date": "2026-10",
   "event": "nordigen.com no longer resolves; gocardless.com/bank-account-data shows Pay by Bank marketing"
  }
 ],
 "relation": "cautionary-tale",
 "relation_note": "The free bank feed that self-hosted tools built on, then closed to newcomers: a reminder that an owner's bank connector must not depend on one aggregator's goodwill, and that what was fetched must be kept locally.",
 "borrow": "The API shape: institutions, a consent agreement with explicit history and access windows, a requisition that yields accounts, then per-account reads. Its rate-limit headers and windows map directly onto ownpurse's rule of respecting each source's limits.",
 "avoid": "Building a connector that only works while a vendor's free tier lasts. Treat any aggregator as one replaceable path to a bank, and keep every fetched line in the record, because the consent window (90 days by default) means the source forgets what it once returned.",
 "sources": [
  {
   "title": "GoCardless Bank Account Data overview (docs)",
   "url": "https://docs.gocardless.com/bank-account-data/overview"
  },
  {
   "title": "GoCardless Bank Account Data quick-start guide",
   "url": "https://docs.gocardless.com/bank-account-data/quick-start-guide"
  },
  {
   "title": "GoCardless to acquire Nordigen (1 July 2022)",
   "url": "https://gocardless.com/g/gc-nordigen/"
  },
  {
   "title": "Actual Budget: GoCardless setup",
   "url": "https://actualbudget.org/docs/advanced/bank-sync/gocardless"
  },
  {
   "title": "Actual Budget: bank sync providers",
   "url": "https://actualbudget.org/docs/advanced/bank-sync/"
  },
  {
   "title": "Firefly III data importer FAQ: GoCardless",
   "url": "https://docs.firefly-iii.org/references/faq/data-importer/gocardless/"
  },
  {
   "title": "Mollie to acquire GoCardless (11 December 2025)",
   "url": "https://www.mollie.com/news/mollie-to-acquire-gocardless"
  },
  {
   "title": "Mollie and GoCardless combine (1 September 2026)",
   "url": "https://www.mollie.com/news/mollie-and-gocardless"
  },
  {
   "title": "GoCardless pricing",
   "url": "https://gocardless.com/pricing/"
  },
  {
   "title": "GoCardless (Wikipedia)",
   "url": "https://en.wikipedia.org/wiki/GoCardless"
  },
  {
   "title": "GoCardless on Trustpilot",
   "url": "https://www.trustpilot.com/review/gocardless.com"
  }
 ],
 "unverified": [
  "Exact day in July 2025 that sign-ups closed, and GoCardless's own announcement of it (taken from Actual Budget's docs)",
  "Nordigen's founding year (2016) and its launch date for the free tier",
  "Mollie deal price of €1.05B (from Wikipedia; not stated on Mollie's pages)",
  "Current bank count and country count",
  "What Mollie plans for Bank Account Data after the merger",
  "Prices for verified, paid accounts"
 ],
 "researched": "2026-10-10"
}
---

## What it is

GoCardless Bank Account Data is a PSD2 account-information API. It reads account details, balances and transactions from banks across the European Economic Area and the UK on the account holder's consent. It started as [Nordigen](https://gocardless.com/g/gc-nordigen/), a Latvian provider whose free tier covered more than 2,300 banks in 31 countries. GoCardless announced it would buy Nordigen on 1 July 2022 and promised "free open banking connectivity at scale". For three years that promise held, and it became the default bank feed for self-hosted budgeting.

What it did well: it was simple and it was free. A person could sign up, get a `secret_id` and `secret_key`, and pull their own bank lines from their own server, with no business, no contract and no app in the middle.

## How it works

The [quick-start guide](https://docs.gocardless.com/bank-account-data/quick-start-guide) is four calls. Exchange the secrets for a refresh token (30 days) and an access token (24 hours). List institutions by country. Create an end-user agreement and a requisition that returns a bank login link. Then read details, balances and transactions per account. By default an agreement grants 90 days of history and 90 days of access. You can set `max_historical_days` and `access_valid_for_days`, and the [overview](https://docs.gocardless.com/bank-account-data/overview) promises "up to 24 months of transaction history". Banks may cap reads at four calls a day per account, per endpoint.

## Data and access

In July 2025 GoCardless stopped accepting new accounts. [Actual Budget's docs](https://actualbudget.org/docs/advanced/bank-sync/gocardless) say "GoCardless has stopped accepting new Bank Account Data accounts", and that existing accounts "should continue to work". Its [bank-sync page](https://actualbudget.org/docs/advanced/bank-sync/) marks it "not accepting new accounts" and points European users to Enable Banking. The free tier allowed up to 50 bank connections a month. Paid, verified accounts are arranged through sales, and GoCardless's [pricing page](https://gocardless.com/pricing/) lists only its payment plans. nordigen.com no longer resolves, and the product URL now shows Pay by Bank marketing.

## Agents

There is no MCP server and there are no agent features. The API is small enough that an agent could drive it easily. The 90-day re-consent is a human checkpoint, but it gates reads, not writes.

## Customers

Self-hosters and small developers: Actual Budget, Firefly III's [data importer](https://docs.firefly-iii.org/references/faq/data-importer/gocardless/) (which had to add handling when "GoCardless introduced rate limits"), and many hobby projects, plus fintechs on paid terms. GoCardless's [Trustpilot](https://www.trustpilot.com/review/gocardless.com) score (2.3 from 2,492 reviews) is almost entirely about direct debits, not this product.

## History

GoCardless was founded in London in 2011 and raised $312M at a $2.1B valuation in 2022 ([Wikipedia](https://en.wikipedia.org/wiki/GoCardless)). Mollie agreed to buy it on [11 December 2025](https://www.mollie.com/news/mollie-to-acquire-gocardless) and [completed the deal on 1 September 2026](https://www.mollie.com/news/mollie-and-gocardless). The announcements talk about Pay by Bank and do not mention account data. The pattern is a familiar one: a free data feed bought to win developers, then closed once the buyer's focus moved to payments.

## For ownpurse

This is the cautionary tale for ownpurse's bank connectors. The API shape is worth copying: explicit consent windows, per-account reads and honest rate-limit headers. The dependency is not. Treat any aggregator as one replaceable path to a bank. Record which path each line came through, and keep every line fetched, because the source forgets after the consent window.
