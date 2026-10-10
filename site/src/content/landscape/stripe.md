---
{
 "slug": "stripe",
 "name": "Stripe",
 "url": "https://stripe.com",
 "segment": "spend-management",
 "status": "active",
 "founded": "2010 (Wikipedia's infobox says 2009, its history section 2010); public launch 2011",
 "hq": "South San Francisco, California, and Dublin, Ireland",
 "backing": "Private. Seed of $2M in May 2011 (Thiel, Musk, Sequoia, a16z, SV Angel); $600M at a $95B valuation in March 2021; Series I of more than $6.5B at a $50B valuation in March 2023; employee tender at $91.5B in February 2025; employee tender at $159B announced with the 2025 annual letter on 24 February 2026. Founders Patrick and John Collison.",
 "tagline": "Payments and financial infrastructure for internet businesses, with an API that most online companies use to take money.",
 "scope": "Stripe processes card, bank and wallet payments and adds billing, invoicing, sales tax, revenue recognition, payouts, card issuing, treasury accounts, stablecoin rails (Bridge) and usage-based billing (Metronome). It holds a very detailed record of money that moves through Stripe, and exports it by API, CSV, Sigma SQL and Data Pipeline. It is not a general ledger: it does not keep your books for money that never touches Stripe, and it relies on connectors (its own NetSuite connector, third-party QuickBooks and Xero apps) to post into an accounting system.",
 "domains": [
  "payments",
  "invoicing",
  "tax",
  "bank",
  "spend",
  "crypto",
  "commerce",
  "reporting",
  "close"
 ],
 "customers_served": [
  "freelancer",
  "smb",
  "mid-market",
  "enterprise",
  "developers"
 ],
 "spectrum": {
  "custody": {
   "score": 1,
   "note": "Stripe holds the authoritative payment record; Data Pipeline can deliver a full copy to a warehouse or bucket you own every 3 hours, but that is a paid mirror, not the source of truth."
  },
  "locality": {
   "score": 0,
   "note": "Cloud only; nothing runs on your machine except the CLI that calls the hosted API."
  },
  "history": {
   "score": 2,
   "note": "Balance transactions are effectively append-only (refunds and adjustments are new entries), but most objects are current-state, and the Events API only reaches back 30 days."
  },
  "openness": {
   "score": 2,
   "note": "Fully public, well-documented API and complete export (CSV reports, Sigma, Data Pipeline to Parquet); SDKs and the stripe/ai toolkit are MIT; the platform itself is closed."
  },
  "breadth": {
   "score": 3,
   "note": "Payments, billing, invoicing, tax, revenue recognition, issuing, treasury and stablecoins in one account, but only for money that flows through Stripe; no general ledger across other sources."
  },
  "autonomy": {
   "score": 2,
   "note": "Automates within its own domain (tax calculation, revenue recognition schedules, retries, payout reconciliation reports) but does not post to your books unattended without a connector."
  },
  "agent_readiness": {
   "score": 4,
   "note": "Official remote MCP at mcp.stripe.com with OAuth, agent-tagged restricted keys, approval rules on sensitive writes, CLI `stripe agent setup`, skills and plugins for major agent harnesses."
  },
  "integration_ease": {
   "score": 4,
   "note": "Self-serve keys, sandboxes, typed SDKs, webhooks and event destinations, and the most widely copied API documentation in fintech."
  },
  "market": {
   "score": 3,
   "note": "Spans solo founders to large companies (over 5 million businesses, 90% of the Dow per the 2025 letter), with self-serve weighted toward online SMBs and startups."
  }
 },
 "api": {
  "public": true,
  "style": "REST (v1, form-encoded, versioned by date header) plus a newer v2 JSON API; webhooks and event destinations; Sigma SQL; Data Pipeline warehouse shares",
  "auth": "Secret keys (sk_), restricted keys (rk_) with per-resource permissions, organization keys (sk_org_), agent-tagged restricted keys, OAuth for Connect platforms and for the MCP server; access policies by IP, ASN and country",
  "limits": "Live mode 100 requests per second globally, 25 per second in sandbox, 25 per second per endpoint unless stated; Search 20 reads per second; default 50 concurrent requests in live mode; read allocation averages 500 GET requests per transaction over 30 days with a 10,000 per month floor; writes have no allocation cap; Events retrievable for 30 days (docs, read 2026-10-10)",
  "cost": "The API itself is free; you pay per transaction. Bulk data access is metered: Sigma from EUR 15 per month plus from EUR 0.03 per charge, Data Pipeline EUR 0.025 per transaction (Italian price page, read 2026-10-10). Data, reporting and tax endpoints are excluded from the read allocation.",
  "export": "CSV reports (balance, payout reconciliation, itemized), scheduled reports, Report Run API, a legacy QuickBooks Desktop IIF export, Sigma SQL results, and Data Pipeline full loads to Snowflake, Redshift, Databricks, BigQuery or Parquet in S3, GCS or Azure every 3 hours. Data Pipeline is not offered in India.",
  "notes": "Payout reconciliation reports break each bank deposit into the charges, refunds and fees inside it, which is exactly the join an accounting connector needs. The 30-day events window means a connector that wants full history must page the object lists or buy Data Pipeline, not replay events."
 },
 "pricing": {
  "model": "Pay per transaction, plus percentage or per-unit fees for add-on products; custom and IC+ pricing for high volume",
  "entry": "No monthly fee for payments. EEA standard cards 1.5% + EUR 0.25 (Italian page, read 2026-10-10). The US list rate has long been 2.9% + 30 cents per domestic card charge, but the US page could not be loaded in this session.",
  "top": "Custom enterprise packages (IC+ pricing, volume discounts, platform rates) through sales; no published ceiling",
  "notes": "Add-ons on the Italian page (2026-10-10): Billing 0.7% of billing volume, Tax 0.5% per transaction where registered, Invoicing Starter 0.4% per paid invoice, Revenue Recognition 0.25% of volume, international cards 3.15% + EUR 0.25, currency conversion +2%, disputes EUR 20 each, Financial Connections EUR 1.25 per instant verification. Fees stack: a subscription business on Billing, Tax and Revenue Recognition pays several percentages on the same volume."
 },
 "agents": {
  "mcp": "official",
  "ai_features": "Remote MCP server with generic `stripe_api_read` and `stripe_api_write` tools over most of the API, docs search, an implementation planner, natural-language analytics (Sigma SQL generation, preview) and Treasury money-movement tools in preview; Radar fraud models; Smart Disputes; agent skills and plugins; the Agentic Commerce Protocol with OpenAI and Instant Checkout in ChatGPT (September 2025); AI billing SDKs (@stripe/ai-sdk, @stripe/token-meter).",
  "agent_writes": "Yes. Through MCP with OAuth (consent per live account or sandbox, with per-environment permissions) or an agent-tagged restricted key; the write tool covers POST, PATCH, PUT and DELETE across most resources. From 31 October 2026 the MCP server accepts only OAuth or agent-tagged keys, not full secret keys or untagged restricted keys.",
  "approval": "Two gates. Stripe requires human confirmation for certain MCP writes made on a user's behalf, such as refunds and outbound payments: the agent returns a URL, the person approves, and the agent retries with a token; unapproved requests expire after 24 hours. Agent-tagged keys are automatically subject to approval rules, with a default rule set covering payouts, refunds and account configuration changes reviewed by a designated approver. Admins can turn MCP access off per team and environment and revoke OAuth sessions.",
  "notes": "This is the closest any large financial provider has come to ownpurse's 'agents propose; people post' rule, applied at the source rather than at the owner's side. Stripe also tells users to enable client-side tool approval and warns about prompt injection when combining MCP servers."
 },
 "integration": {
  "scope": "Official NetSuite connector; an Xero bank-feed integration for platforms; QuickBooks via third-party apps in the Stripe App Marketplace and stripe.partners; webhooks to any endpoint; Data Pipeline into four warehouses and three object stores; Connect for platforms routing money to sellers; the Stripe Apps platform for embedded third-party apps.",
  "notes": "Stripe's own docs say QuickBooks Online cannot import its IIF export and point to third-party apps for one-click imports, so the payments-to-ledger step is still largely left to partners. The number of marketplace apps could not be read in this session."
 },
 "preference": {
  "scale": "$1.9 trillion of total volume in 2025, up 34% from 2024, about 1.6% of global GDP; over 5 million businesses; 90% of the Dow Jones Industrial Average and 80% of the Nasdaq 100 (2025 annual letter, 24 February 2026). Revenue US$5.1B in 2024 and about 8,500 employees in 2025 (Wikipedia). Stablecoin payment volume about $400B in 2025 (annual letter).",
  "ratings": "Trustpilot 1.6 out of 5 from 17,537 reviews, 52% one-star and 38% five-star (read 2026-10-10). G2 and Capterra pages returned HTTP 403 in this session and could not be checked.",
  "sentiment": "Developers praise the API, docs, sandboxes and speed to first charge (20% of 2025 Atlas startups charged within 30 days). Merchants on Trustpilot complain about automated account suspensions, held or reserved funds, slow or automated support, opaque fees and repeated identity checks. Finance teams note that fees on add-ons stack and that reconciling payouts into an accounting system still needs a connector."
 },
 "timeline": [
  {
   "date": "2010",
   "event": "Founded by Patrick and John Collison (Wikipedia gives 2009 in one place)"
  },
  {
   "date": "2011-05",
   "event": "$2M seed from Thiel, Musk, Sequoia, a16z and SV Angel; public launch later in 2011"
  },
  {
   "date": "2020-10",
   "event": "Acquired Paystack (Nigeria), reportedly over $200M"
  },
  {
   "date": "2021-03",
   "event": "Raised $600M at a $95B valuation"
  },
  {
   "date": "2021-04",
   "event": "Acquired TaxJar (sales tax), reportedly about $200M"
  },
  {
   "date": "2021-10",
   "event": "Acquired Recko (reconciliation)"
  },
  {
   "date": "2023-03",
   "event": "Series I of more than $6.5B at a $50B valuation"
  },
  {
   "date": "2024-07",
   "event": "Acquired Lemon Squeezy (merchant of record)"
  },
  {
   "date": "2025-02-04",
   "event": "Completed acquisition of Bridge (stablecoins), reported at $1.1B"
  },
  {
   "date": "2025-02",
   "event": "Employee tender offer at a $91.5B valuation"
  },
  {
   "date": "2025-06",
   "event": "Acquired Privy (crypto wallets)"
  },
  {
   "date": "2025-09-29",
   "event": "Launched the Agentic Commerce Protocol with OpenAI and Instant Checkout in ChatGPT"
  },
  {
   "date": "2025-12-03",
   "event": "Acquired Metronome (usage-based billing)"
  },
  {
   "date": "2026-02-24",
   "event": "2025 annual letter: $1.9T volume, up 34%; tender offer at $159B"
  },
  {
   "date": "2026-04-28",
   "event": "Sessions 2026 (28-30 April): agent-facing commerce, auth patterns that keep humans in control, a Revenue suite roadmap"
  },
  {
   "date": "2026-09-30",
   "event": "Agreed to acquire Parafin (embedded small-business financing)"
  },
  {
   "date": "2026-10-31",
   "event": "MCP server stops accepting full secret keys and untagged restricted keys; OAuth or agent keys only"
  }
 ],
 "relation": "source",
 "relation_note": "Stripe is a source ownpurse should connect to, not a competitor: for most online small businesses it is where revenue, fees, refunds and payouts originate, and the Stripe-to-books join (payouts against bank lines, fees against expense accounts) is one of the most common reconciliation jobs ownpurse could do locally.",
 "borrow": "Agent-tagged restricted keys with default approval rules on payouts, refunds and config changes; the approve-by-URL then retry-with-token flow for agent writes; OAuth consent scoped per environment; payout reconciliation reports that decompose each deposit; immutable balance transactions as the unit of money movement; generic read and write tools over a large API instead of hundreds of tools; `stripe agent setup` as one command that configures every agent harness.",
 "avoid": "Relying on the Events API for history (30 days only); stacking percentage fees on the same volume; leaving the books step to a marketplace of third-party apps; letting automated risk decisions freeze a customer with no clear human path, which is the main source of Stripe's poor Trustpilot score.",
 "sources": [
  {
   "title": "Stripe MCP documentation",
   "url": "https://docs.stripe.com/mcp"
  },
  {
   "title": "Stripe API keys and agent keys",
   "url": "https://docs.stripe.com/keys"
  },
  {
   "title": "Stripe rate limits",
   "url": "https://docs.stripe.com/rate-limits"
  },
  {
   "title": "Stripe Events API",
   "url": "https://docs.stripe.com/api/events"
  },
  {
   "title": "How Data Pipeline works",
   "url": "https://docs.stripe.com/stripe-data/access-data-in-warehouse"
  },
  {
   "title": "Stripe reports",
   "url": "https://docs.stripe.com/reports"
  },
  {
   "title": "Integrate an accounting tool",
   "url": "https://docs.stripe.com/accounting-integrations"
  },
  {
   "title": "Stripe pricing (Italian locale, read 2026-10-10)",
   "url": "https://stripe.com/pricing"
  },
  {
   "title": "Stripe 2025 annual letter and tender offer",
   "url": "https://stripe.com/newsroom/news/stripe-2025-update"
  },
  {
   "title": "Stripe newsroom",
   "url": "https://stripe.com/newsroom"
  },
  {
   "title": "Stripe completes Bridge acquisition",
   "url": "https://stripe.com/newsroom/news/stripe-completes-bridge-acquisition"
  },
  {
   "title": "Stripe and OpenAI: Instant Checkout and the Agentic Commerce Protocol",
   "url": "https://stripe.com/newsroom/news/stripe-openai-instant-checkout"
  },
  {
   "title": "Stripe Sessions 2026",
   "url": "https://stripe.com/sessions/2026"
  },
  {
   "title": "stripe/ai on GitHub",
   "url": "https://github.com/stripe/ai"
  },
  {
   "title": "Stripe on Trustpilot",
   "url": "https://www.trustpilot.com/review/stripe.com"
  },
  {
   "title": "Stripe, Inc. on Wikipedia",
   "url": "https://en.wikipedia.org/wiki/Stripe,_Inc."
  }
 ],
 "unverified": [
  "US list pricing (2.9% + 30 cents domestic cards, ACH, $15 disputes): the US pricing page redirected to the Italian locale in this session; only EUR figures were confirmed.",
  "Founding year: Wikipedia gives both 2009 and 2010.",
  "Revenue of US$5.1B (2024) and about 8,500 employees (2025) come from Wikipedia only.",
  "The Bridge price of $1.1B and the Paystack and TaxJar prices are press reports cited by Wikipedia; Stripe did not disclose terms.",
  "Wikipedia reports a $7.5B acquisition of OpenRouter announced 19 August 2026 and a $53B joint bid with Advent for PayPal on 15 July 2026; neither appeared in the Stripe newsroom list read on 2026-10-10, so both are unconfirmed and left out of the timeline.",
  "Metronome (3 December 2025) and Privy (June 2025) acquisition dates come from Wikipedia; Stripe's own announcements were not read.",
  "G2 and Capterra ratings: both sites returned HTTP 403.",
  "Reddit sentiment was not checked: the session's web search budget ran out.",
  "The number of apps in the Stripe App Marketplace and whether Stripe itself builds a QuickBooks Online connector could not be confirmed."
 ],
 "researched": "2026-10-10"
}
---

## What it is

Stripe is the payments and financial infrastructure company behind most online small businesses' checkout. It started with a seven-line card API in 2011 and now sells billing, invoicing, sales tax, revenue recognition, card issuing, treasury accounts, stablecoin rails through [Bridge](https://stripe.com/newsroom/news/stripe-completes-bridge-acquisition) and usage-based billing through Metronome. Its [2025 annual letter](https://stripe.com/newsroom/news/stripe-2025-update), published on 24 February 2026, reports $1.9 trillion of volume in 2025, up 34%, across more than 5 million businesses, and a tender offer at a $159 billion valuation.

It is not a ledger. Stripe records money that moves through Stripe in great detail, and stops there.

## How it works

You pay per transaction and per add-on. On the [pricing page](https://stripe.com/pricing) (served in Italian for this session, read 10 October 2026), standard EEA cards are 1.5% + EUR 0.25, Billing is 0.7% of billing volume, Tax 0.5% per transaction, Revenue Recognition 0.25% of volume and Invoicing Starter 0.4% per paid invoice. The US page could not be loaded, so the long-standing US rate of 2.9% + 30 cents is listed as unverified here. The add-on percentages stack on the same volume, which matters for subscription businesses.

The core record is the balance transaction: every charge, refund, fee and payout lands as a new entry rather than an edit. [Payout reconciliation reports](https://docs.stripe.com/reports) break each bank deposit into the transactions inside it.

## Data and access

The API is public, self-serve and well documented, with sandboxes and typed SDKs. [Rate limits](https://docs.stripe.com/rate-limits) are 100 requests per second in live mode, 25 per endpoint, 50 concurrent, and a read allocation averaging 500 GET requests per transaction over 30 days with a 10,000 per month floor. Writes have no allocation. [Events](https://docs.stripe.com/api/events) can be retrieved for only 30 days, so full history comes from paging object lists or from paid bulk access.

Bulk export is good if you pay for it. [Data Pipeline](https://docs.stripe.com/stripe-data/access-data-in-warehouse) delivers a full load to Snowflake, Redshift, Databricks, BigQuery or Parquet files in a bucket you own every 3 hours, at EUR 0.025 per transaction; Sigma adds SQL in the dashboard. Free CSV reports and a QuickBooks Desktop IIF export cover the basics. Stripe's own [accounting docs](https://docs.stripe.com/accounting-integrations) say QuickBooks Online cannot import that IIF file and send you to third-party apps; the official connector is for NetSuite.

## Agents

Stripe is the most agent-ready provider in this survey. Its [remote MCP server](https://docs.stripe.com/mcp) at mcp.stripe.com authenticates by OAuth, with consent per live account or sandbox, or by an agent API key. It exposes generic `stripe_api_read` and `stripe_api_write` tools over most of the API, docs search, analytics in preview and Treasury money movement in private preview. One command, `stripe agent setup`, configures Claude Code, Codex, Cursor and others; the [stripe/ai](https://github.com/stripe/ai) toolkit is MIT licensed.

Writes have two human gates. For MCP writes made on a user's behalf, such as refunds and outbound payments, Stripe returns a URL; you approve, the agent retries with a token, and unapproved requests expire after 24 hours. [Agent-tagged keys](https://docs.stripe.com/keys) carry default approval rules for payouts, refunds and account configuration. From 31 October 2026 the MCP server accepts only OAuth or agent keys. Stripe also co-wrote the [Agentic Commerce Protocol](https://stripe.com/newsroom/news/stripe-openai-instant-checkout) with OpenAI in September 2025, for agents that buy rather than agents that keep books.

## Customers

Stripe serves solo founders through to 90% of the Dow, per the annual letter. Developers praise the API and docs. Merchants are less kind: [Trustpilot](https://www.trustpilot.com/review/stripe.com) shows 1.6 out of 5 from 17,537 reviews, mostly about automated account suspensions, held funds and hard-to-reach support. G2 and Capterra could not be read in this session.

## History

The Collison brothers founded Stripe around 2010. It peaked at a $95 billion valuation in March 2021, fell to $50 billion in its March 2023 Series I, and recovered to $91.5 billion in February 2025 and $159 billion in February 2026. Its acquisitions show it moving toward finance operations: TaxJar (2021), Recko for reconciliation (2021), Lemon Squeezy (2024), Bridge (2025), Privy (2025), Metronome (December 2025) and Parafin (agreed 30 September 2026). Recko is the clearest case of Stripe trying to own the reconciliation step itself. It still leaves the step into the books to connectors.

## For ownpurse

Stripe is a source to build early, after Xero and before most banks. For a typical online small business, the hard monthly reconciliation is this: Stripe payouts against bank lines, gross sales against fees, and refunds and disputes against revenue. Stripe has all of the data for it, but it charges for bulk access and leaves the join to a marketplace. An ownpurse connector can page balance transactions and payouts under a read-only restricted key, keep every version in the local log (Stripe's own events expire after 30 days) and check each payout against the bank feed and the Xero record to the cent.

Stripe has also already built a version of the approval gate ownpurse argues for. Agent keys come with default rules, sensitive writes need an approve-by-URL step, and consent is scoped per environment. ownpurse should borrow that shape for its own sealed packages and point to it as evidence that the pattern works at scale. The gap Stripe leaves is the one ownpurse is built for: one record across Stripe, the bank, the books and receipts, held by the owner.
