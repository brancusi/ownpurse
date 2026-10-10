---
{
 "slug": "akahu",
 "name": "Akahu",
 "url": "https://www.akahu.nz",
 "segment": "bank-aggregator",
 "status": "active",
 "founded": "December 2020 (integrations began as the side project Jude in 2013)",
 "hq": "New Zealand (city not stated on the about page)",
 "backing": "Westpac New Zealand's venture arm invested, announced December 2020, amount not disclosed. No other rounds confirmed.",
 "tagline": "New Zealand's open finance API for bank data, payments and identity, with free personal apps for developers reading their own accounts.",
 "scope": "Akahu connects to New Zealand banks and other financial providers and offers data (accounts, balances, settled and pending transactions, enrichment), one-off and enduring payments, identity verification and a loan-application product. It is an accredited requestor in New Zealand's regulated open banking system and uses 'classic connections' where banks are not yet on it. It does not keep books, reconcile or post to an accounting system.",
 "domains": [
  "bank",
  "payments",
  "integration",
  "personal"
 ],
 "customers_served": [
  "consumer",
  "developers",
  "smb",
  "mid-market",
  "enterprise"
 ],
 "spectrum": {
  "custody": {
   "score": 1,
   "note": "Akahu holds and updates a copy of the transactions; the owner can pull their own through a personal app, but nothing is held by the owner by default."
  },
  "locality": {
   "score": 0,
   "note": "Hosted API; nothing runs locally."
  },
  "history": {
   "score": 1,
   "note": "Transactions are updated in place with stable ids, about 0.1% deleted and recreated; personal apps see 730 days; pending lines have no ids and are rebuilt each fetch."
  },
  "openness": {
   "score": 2,
   "note": "Public docs, an llms.txt index and a TypeScript SDK; free personal access to your own data; closed source."
  },
  "breadth": {
   "score": 2,
   "note": "Bank data, payments, identity and enrichment across New Zealand providers; no books or documents."
  },
  "autonomy": {
   "score": 1,
   "note": "Daily scheduled refresh runs unattended; nothing posts to books."
  },
  "agent_readiness": {
   "score": 2,
   "note": "Personal user and app tokens suit a local agent and the docs have an llms.txt index; no MCP server, CLI or scoped read-only tokens."
  },
  "integration_ease": {
   "score": 3,
   "note": "Sign up, verify identity, enable 2FA and get tokens the same day; full apps need accreditation and a pricing conversation."
  },
  "market": {
   "score": 2,
   "note": "130+ organisations from fintechs to banks and government, plus individual developers; New Zealand only."
  }
 },
 "api": {
  "public": true,
  "style": "REST JSON with cursor pagination (up to 100 transactions per page), webhooks for full apps, OAuth2 for third-party apps",
  "auth": "Personal apps: a user access token as Bearer plus an app id token in X-Akahu-Id. Full apps: OAuth2 user authorisation",
  "limits": "Global limits not published; POST /v1/payments is limited to 200 requests per user per minute; 429 with backoff. Personal apps: one user (yourself), daily scheduled refresh, manual refresh no more than hourly, no webhooks, 730 days of history; payments limited to $5,000 single and $10,000 daily",
  "cost": "Personal apps free ('We don't charge fees for accessing your own data'). Full apps: ongoing connectivity typically $0.50 to $2.50 per user per month; one-off payments $0.15 each; identity or account verification $1.00; loan applications $5.00 (pricing page, 2026, indicative, NZD assumed)",
  "export": "Through the API only; no bulk export file confirmed",
  "notes": "The POST /transfers endpoint is disabled for all apps by 1 June 2026 because the regulated open banking standard does not support it. Since 31 October 2023 personal apps need identity verification and 2FA."
 },
 "pricing": {
  "model": "Usage-based for businesses (per user per month, per payment, per verification); free for personal apps and consumers",
  "entry": "Free personal app for your own accounts",
  "top": "Custom terms by volume",
  "notes": "Figures are indicative and confirmed after Akahu learns the functionality and volumes needed; currency assumed NZD."
 },
 "agents": {
  "mcp": "none",
  "ai_features": "Genie transaction enrichment (trading name, category, logo); categorisation and insights in Akahu Apply. No LLM or agent features found.",
  "agent_writes": "An agent holding a full app's consent can initiate payments; personal apps cannot make payments per the personal apps guide, while the 2023 changelog sets payment limits for personal apps (the two pages conflict). No writes to books.",
  "approval": "User consent through OAuth or the personal app setup; enduring payment consents; payment limits. No per-action human approval gate in the API itself.",
  "notes": "A personal app token is the simplest way for a local agent to read one New Zealander's bank data; no community MCP server confirmed (search budget exhausted)."
 },
 "integration": {
  "scope": "New Zealand banks and other providers; the institution list and count were not retrieved. Named customers include Westpac, AMP, PaySauce, PocketSmith, Simplicity, MBIE, NZHL, Squirrel and Sorted (home page, 2026).",
  "notes": "Moving from classic connections to official open banking APIs bank by bank; changelog notes BNZ enduring consent and an ASB transaction period increase."
 },
 "preference": {
  "scale": "Used by 130+ products and organisations (home page and about page, 2026)",
  "ratings": "None found on G2, Capterra or Trustpilot; app-store and Reddit sentiment not checked (search budget exhausted, Reddit not fetchable).",
  "sentiment": "Not gathered from forums. Its customer list spans a major bank, government and budgeting apps, which suggests it is the default NZ connection layer."
 },
 "timeline": [
  {
   "date": "2013",
   "event": "Ben Lynch, then on Xero's bank feeds team, starts personal bank integrations as the side project Jude"
  },
  {
   "date": "2020-12",
   "event": "Akahu launches as an API company with an investment from Westpac New Zealand"
  },
  {
   "date": "2023-10-31",
   "event": "Deadline for personal app users to complete identity verification and 2FA; personal app payment limits set"
  },
  {
   "date": "2026-06-01",
   "event": "POST /transfers disabled for all apps as Akahu moves to official open banking APIs"
  }
 ],
 "relation": "source",
 "relation_note": "For a New Zealand owner, an Akahu personal app is a free, sanctioned way for ownpurse to pull their own bank lines and balances as an independent witness beside Xero, which cannot return bank statement lines.",
 "borrow": "The personal app model (free, own data only, identity check and 2FA as the gate); its honest identity notes: stable ids updated in place, about 0.1% recreated, pending lines without ids that should be rebuilt each fetch, which maps to ownpurse's identity ladder and held-but-not-posted facts.",
 "avoid": "Depending on one intermediary for a capability it can withdraw (transfers ended June 2026); a 730-day window means the owner must keep their own history.",
 "sources": [
  {
   "title": "Akahu home page",
   "url": "https://www.akahu.nz"
  },
  {
   "title": "Akahu about",
   "url": "https://www.akahu.nz/about"
  },
  {
   "title": "Akahu pricing",
   "url": "https://www.akahu.nz/pricing"
  },
  {
   "title": "Akahu personal apps",
   "url": "https://developers.akahu.nz/docs/personal-apps"
  },
  {
   "title": "Akahu: accessing transactional data",
   "url": "https://developers.akahu.nz/docs/accessing-transactional-data"
  },
  {
   "title": "Akahu rate limits",
   "url": "https://developers.akahu.nz/docs/reference-rate-limits"
  },
  {
   "title": "Akahu changelog: important changes to personal apps",
   "url": "https://developers.akahu.nz/changelog/important-changes-to-personal-apps"
  },
  {
   "title": "Akahu changelog: transfers API deprecation",
   "url": "https://developers.akahu.nz/changelog/transfers-api-deprecation"
  },
  {
   "title": "Akahu developer llms.txt",
   "url": "https://developers.akahu.nz/llms.txt"
  },
  {
   "title": "Akahu on GitHub",
   "url": "https://github.com/akahu-io"
  }
 ],
 "unverified": [
  "Headquarters city",
  "Funding amount and any rounds after December 2020",
  "Number of supported institutions",
  "Currency of pricing (assumed NZD)",
  "Whether personal apps can make payments now (personal apps guide says no; 2023 changelog sets personal app payment limits)",
  "Status and dates of New Zealand's Customer and Product Data Act bank designations",
  "Community MCP servers or agent integrations (search budget exhausted)",
  "Reddit, app-store and review-site sentiment"
 ],
 "researched": "2026-10-10"
}
---

## What it is

Akahu is New Zealand's open finance infrastructure: one API for bank data, pay-by-bank payments, identity checks and transaction enrichment. Its roots run through Xero. Ben Lynch built personal bank integrations as a side project called Jude in 2013 while on Xero's bank feeds team, and the integrations [moved into Akahu in December 2020](https://www.akahu.nz/about), when Westpac New Zealand's venture arm invested. It describes itself as an accredited requestor in the country's regulated open banking system and serves 130+ organisations, including Westpac, PocketSmith, Sorted and MBIE ([home page](https://www.akahu.nz)).

## How it works

A business app gets a user's consent through OAuth2 and then reads accounts, balances and transactions, receives webhooks, makes payments or runs identity checks. Where a bank is on the regulated system Akahu uses its official API; elsewhere it uses classic connections. Transactions come in pages of up to 100 with a cursor. Akahu gives most transactions a stable id, since banks do not supply one, and updates modified ones in place; about 0.1% are deleted and recreated with a new id, and pending transactions have no id at all and should be rebuilt each time ([transactions guide](https://developers.akahu.nz/docs/accessing-transactional-data)). That is unusually candid documentation of how bank data really behaves.

## Data and access

The distinctive part is the [personal app](https://developers.akahu.nz/docs/personal-apps). Anyone with a MyAkahu account can agree to the developer terms, verify their identity and enable 2FA (required since [31 October 2023](https://developers.akahu.nz/changelog/important-changes-to-personal-apps)), and get a user token and an app token for their own accounts. It is free: "We don't charge fees for accessing your own data" ([pricing](https://www.akahu.nz/pricing)). Limits: one user, a daily scheduled refresh, manual refresh no more than hourly, no webhooks, 730 days of history. Business pricing is usage-based: typically $0.50 to $2.50 per user per month for ongoing data, $0.15 per payment, $1.00 per verification. Global rate limits are not published; payments are capped at 200 requests per user per minute ([rate limits](https://developers.akahu.nz/docs/reference-rate-limits)). Capabilities can go away: the transfers endpoint was [disabled by 1 June 2026](https://developers.akahu.nz/changelog/transfers-api-deprecation) because the regulated standard does not support it.

## Agents

There is no MCP server or CLI. The docs publish an [llms.txt index](https://developers.akahu.nz/llms.txt) and Akahu maintains a [TypeScript SDK](https://github.com/akahu-io). A personal token is all a local agent needs to read one person's bank data, which makes Akahu friendlier to agents than most aggregators, though the token is not scoped to read-only.

## Customers

Customers are New Zealand fintechs, lenders, budgeting apps, a major bank and government. No review-site ratings were found and forum sentiment was not gathered for this dossier.

## History

Akahu shows the path a market takes from scraping to regulated APIs: a developer side project, a bank investment, accreditation, then retiring features the official standard lacks.

## For ownpurse

Akahu is a source. Xero does not expose bank statement lines through its public API, so ownpurse needs the bank's own account of each line as an independent witness. For a New Zealand owner, an Akahu personal app supplies that for free with the owner's own credentials, which fits local-first. ownpurse should store every read as facts, keep its own history past the 730-day window, treat pending lines as held facts rebuilt on each read, and handle the recreated ids through its identity ladder. Xero is popular in New Zealand, so this is likely to be one of the first bank connectors worth building.
