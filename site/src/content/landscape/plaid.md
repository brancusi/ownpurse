---
{
 "slug": "plaid",
 "name": "Plaid",
 "url": "https://plaid.com",
 "segment": "bank-aggregator",
 "status": "active",
 "founded": "2012 (Wikipedia; its infobox says May 2013)",
 "hq": "San Francisco, California, US",
 "backing": "Private, venture-backed. Series D of $425M at $13.4B (April 2021, led by Altimeter); $575M at $6.1B (April 2025, Franklin Templeton, Fidelity, BlackRock, NEA, Ribbit); employee share sale at $8B (confirmed to TechCrunch 26 February 2026). Visa's $5.3B acquisition (announced January 2020) was abandoned in January 2021 after a DOJ antitrust suit.",
 "tagline": "The dominant US bank-data API: apps link a person's bank, card, loan and investment accounts through Plaid and read them as JSON.",
 "scope": "Plaid connects apps to more than 12,000 financial institutions and sells account data (transactions, balances, identity, investments, liabilities, income) plus verification, risk and payment products to businesses. It is a developer platform, not a ledger: it keeps current transaction state and a change feed, not books. A person cannot use it directly except through an app, and Plaid Portal only lets them see and cut connections.",
 "domains": [
  "bank",
  "payments",
  "personal"
 ],
 "customers_served": [
  "developers",
  "smb",
  "mid-market",
  "enterprise"
 ],
 "spectrum": {
  "custody": {
   "score": 1,
   "note": "Plaid and the bank hold the data; the owner gets a copy only if an app they trust pulls it and keeps it, and Portal offers delete, not export."
  },
  "locality": {
   "score": 0,
   "note": "Cloud API only; nothing runs on your machine."
  },
  "history": {
   "score": 1,
   "note": "/transactions/sync gives an added/modified/removed delta stream, but Plaid keeps only current state, and a pending item is removed and replaced by a new id when it posts."
  },
  "openness": {
   "score": 2,
   "note": "Closed service with thorough public docs, published client libraries and an MIT-licensed AI toolkit; no bulk export for the person whose data it is."
  },
  "breadth": {
   "score": 2,
   "note": "Bank, card, loans, investments, payroll income and payments, but no books, documents or tax."
  },
  "autonomy": {
   "score": 1,
   "note": "Refreshes data on its own one to four times a day; never writes to anyone's books."
  },
  "agent_readiness": {
   "score": 3,
   "note": "Two official MCP servers (Dashboard diagnostics and a local coding toolkit) and agent partnerships, but no MCP over an end user's own financial data."
  },
  "integration_ease": {
   "score": 3,
   "note": "Free Sandbox and a self-serve Limited Production trial with good docs; full Production needs a business application and contract."
  },
  "market": {
   "score": 3,
   "note": "Sells to fintechs from startups to large enterprises; consumers reach it only through those apps."
  }
 },
 "api": {
  "public": true,
  "style": "REST, JSON over HTTPS, POST-based endpoints; webhooks (SYNC_UPDATES_AVAILABLE); client SDKs",
  "auth": "client_id and secret per environment; a per-Item access_token obtained by exchanging a public_token from the Link widget; OAuth with banks behind Link",
  "limits": "Production per Item: /transactions/sync 50 per minute, /transactions/refresh 2 per minute and 2,880 per day, /accounts/balance/get 5 per minute; per client: /transactions/sync 2,500 per minute. Fixed windows, subject to change. History: days_requested 30 to 730 (default 90), fixed at first initialisation.",
  "cost": "No public price list. Three billing models: one-time per Item (Auth, Identity), monthly subscription per Item (Transactions, Investments, Liabilities, Recurring) and per successful request (Balance, most refresh endpoints). Pay as you go, Growth (12-month commitment) and Custom plans. Sandbox free; Limited Production gives 200 live calls per product; Trial plans cap Production at 10 Items.",
  "export": "Nothing in bulk for an end user; Plaid Portal lets a US user see connections and delete data. A developer can page full history through /transactions/sync and keep it.",
  "notes": "The /transactions/sync cursor is valid for at least a year; on TRANSACTIONS_SYNC_MUTATION_DURING_PAGINATION the whole pagination loop must restart from the first cursor of that update. Pending transactions are removed and the posted version arrives under a new transaction_id with pending_transaction_id pointing back (null when Plaid cannot match). Some institutions (Capital One, USAA) send no pending data."
 },
 "pricing": {
  "model": "Usage-based per connected account (one-time or monthly) or per request, negotiated by contract",
  "entry": "Free Sandbox; Pay as you go with no commitment (rates not published)",
  "top": "Custom volume contracts with premium support and beta products",
  "notes": "Consumers pay nothing. Dollar rates are disclosed only after applying for Production or talking to sales. Banks have begun charging aggregators for access (JPMorgan, reported 2025), which flows into Plaid's costs."
 },
 "agents": {
  "mcp": "official",
  "ai_features": "Dashboard MCP server (remote, hosted, Production only, 15-minute OAuth tokens) with tools such as plaid_debug_item, plaid_get_link_analytics and plaid_get_usages; the AI coding toolkit (local MCP server, MIT, created 19 May 2025) for mock data, docs search, Sandbox tokens and webhook simulation. Plaid powers account linking in Meta's agent Muse (8 September 2026) and partners with Sierra (August 2026) and Decagon (September 2026). Enrich and foundation models categorise transactions.",
  "agent_writes": "The MCP servers expose diagnostics and developer tooling, not end-user account data. Money movement (Transfer) is an API product for the integrating business, not an agent surface.",
  "approval": "The person approves account access once in Link (and with the bank via OAuth); there is no per-action approval gate for agents.",
  "notes": "Plaid's agent story is about being the plumbing inside other companies' agents. It does not give a person an agent-readable copy of their own data."
 },
 "integration": {
  "scope": "More than 12,000 financial institutions and apps in the US (company page, read 2026-10-10), plus Canada and parts of Europe; more than 9,000 fintechs build on it.",
  "notes": "Link is a drop-in widget; most large US banks are reached by token APIs (Chase agreement October 2018) rather than credentials. Individuals building for themselves can use Sandbox and Limited Production but full Production is aimed at businesses."
 },
 "preference": {
  "scale": "9,000+ fintechs, 12,000+ institutions, 900+ staff (plaid.com/company, undated, read 2026-10-10). Revenue about $170M in 2020 (Forbes via Wikipedia); no recent figure confirmed.",
  "ratings": "Trustpilot 1.2 out of 5 from 225 reviews, 96% one-star (read 2026-10-10). G2 could not be read (403).",
  "sentiment": "Developers value the coverage, the docs and the sync design. End users who reach Trustpilot complain of banks that will not link, connections that drop, failed payments and support that is automated with no person to reach. The 2021 class action ($58M) alleged collection of data without proper consent."
 },
 "timeline": [
  {
   "date": "2012",
   "event": "Founded by Zach Perret and William Hockey; first built budgeting software, then pivoted to a bank API."
  },
  {
   "date": "2018-10-22",
   "event": "Data agreement with JPMorgan Chase: tokenised API access instead of stored credentials."
  },
  {
   "date": "2018-12-11",
   "event": "Series C, $250M at $2.65B."
  },
  {
   "date": "2019-01",
   "event": "Acquires Quovo (investment data) for $200M."
  },
  {
   "date": "2020-01-13",
   "event": "Visa announces a $5.3B acquisition."
  },
  {
   "date": "2021-01-12",
   "event": "Visa and Plaid abandon the merger after the DOJ sues to block it."
  },
  {
   "date": "2021-04-07",
   "event": "Series D, $425M at $13.4B, led by Altimeter."
  },
  {
   "date": "2021-08",
   "event": "Agrees a $58M class action settlement over consent to collect bank data."
  },
  {
   "date": "2025-04-03",
   "event": "Raises $575M at $6.1B; says it will not go public in 2025."
  },
  {
   "date": "2025-05-19",
   "event": "Publishes the AI coding toolkit with a local MCP server."
  },
  {
   "date": "2026-02-26",
   "event": "Employee share sale values Plaid at $8B."
  },
  {
   "date": "2026-09-08",
   "event": "Powers account connections in Meta's agent Muse."
  }
 ],
 "relation": "source",
 "relation_note": "A possible bank source for ownpurse, and the clearest public design for a bank change feed. It sells to businesses, so a person running ownpurse alone would need their own developer account, and Plaid keeps only current state.",
 "borrow": "The added/modified/removed cursor as assert and retract; restarting a whole update from its first cursor when pages mutate; pending_transaction_id as an explicit link between a pending fact and its posted successor; the cursor's long validity.",
 "avoid": "Treating pending-to-posted as delete-and-create with a new id: keep the pending fact, record its retraction and link the posted one. Do not depend on a business-only Production approval for the default path, and do not key identity on transaction_id alone.",
 "sources": [
  {
   "title": "Plaid company page",
   "url": "https://plaid.com/company/"
  },
  {
   "title": "Plaid pricing",
   "url": "https://plaid.com/pricing/"
  },
  {
   "title": "Plaid docs: billing",
   "url": "https://plaid.com/docs/account/billing/"
  },
  {
   "title": "Plaid docs: Transactions API",
   "url": "https://plaid.com/docs/api/products/transactions/"
  },
  {
   "title": "Plaid docs: transaction states",
   "url": "https://plaid.com/docs/transactions/transactions-data/"
  },
  {
   "title": "Plaid docs: rate limits",
   "url": "https://plaid.com/docs/errors/rate-limit-exceeded/"
  },
  {
   "title": "Plaid docs: MCP servers",
   "url": "https://plaid.com/docs/resources/mcp/"
  },
  {
   "title": "plaid/ai-coding-toolkit on GitHub",
   "url": "https://github.com/plaid/ai-coding-toolkit"
  },
  {
   "title": "Plaid legal and privacy policy",
   "url": "https://plaid.com/legal/"
  },
  {
   "title": "Plaid powers Meta's new AI agent, Muse (8 September 2026)",
   "url": "https://plaid.com/blog/meta-muse/"
  },
  {
   "title": "Plaid blog index",
   "url": "https://plaid.com/blog/"
  },
  {
   "title": "Chase: Plaid signs data agreement with JPMorgan Chase (22 October 2018)",
   "url": "https://media.chase.com/news/plaid-signs-data-agreement-with-jpmc"
  },
  {
   "title": "TechCrunch: Plaid raises $575M at $6.1B (3 April 2025)",
   "url": "https://techcrunch.com/2025/04/03/fintech-plaid-raises-575m-at-6-1b-valuation-says-it-will-not-go-public-in-2025/"
  },
  {
   "title": "TechCrunch: Plaid valued at $8B in employee share sale (26 February 2026)",
   "url": "https://techcrunch.com/2026/02/26/plaid-valued-at-8b-in-employee-share-sale/"
  },
  {
   "title": "Wikipedia: Plaid Inc.",
   "url": "https://en.wikipedia.org/wiki/Plaid_Inc."
  },
  {
   "title": "Trustpilot: Plaid",
   "url": "https://www.trustpilot.com/review/plaid.com"
  }
 ],
 "unverified": [
  "Terms and timing of the JPMorgan data-access fee agreement with Plaid (reported in 2025; the earlier survey cites it, but no primary source was read in this pass because web search was exhausted).",
  "Plaid's current revenue; the only confirmed figure is about $170M for 2020.",
  "G2 and Capterra ratings (G2 returned 403).",
  "Whether an individual can obtain full Production access for personal use; the docs read do not say.",
  "Current status of the CFPB section 1033 rule and its effect on aggregator fees; the CFPB page read was last updated August 2025 (ANPR on fees, 22 August 2025)."
 ],
 "researched": "2026-10-10"
}
---

## What it is

Plaid is the dominant US bank-data API. An app embeds Plaid Link, a person logs in to their bank, and the app can then read that person's accounts, balances and transactions as JSON. Plaid says more than 9,000 fintechs build on it and it reaches more than 12,000 institutions ([company page](https://plaid.com/company/), read 10 October 2026). It also sells account verification, identity, income, fraud and payment products. It is infrastructure for businesses: a person meets Plaid only inside someone else's app.

## How it works

Each link is an Item with an access token. For transactions, [/transactions/sync](https://plaid.com/docs/api/products/transactions/) returns three arrays, `added`, `modified` and `removed`, and a `next_cursor`. You page while `has_more` is true; the cursor stays valid for at least a year. If data changes mid-update, Plaid returns `TRANSACTIONS_SYNC_MUTATION_DURING_PAGINATION` and you must restart the whole loop from the first cursor of that update. Plaid checks institutions one to four times a day and fires `SYNC_UPDATES_AVAILABLE` when there is something new. History is set once, 30 to 730 days (default 90), when Transactions is first added; more history means relinking.

When a pending transaction posts, Plaid [removes the pending one and adds a new transaction with a new id](https://plaid.com/docs/transactions/transactions-data/), linked back by `pending_transaction_id`, which is null when Plaid cannot match them. Some banks, Capital One and USAA among them, send no pending data at all.

## Data and access

The API is public and well documented. Pricing is not: the [pricing page](https://plaid.com/pricing/) names three plans (Pay as you go, Growth, Custom) and three models, and the [billing docs](https://plaid.com/docs/account/billing/) say Transactions is a monthly fee per Item while Balance is charged per call, but no dollar figures are published. Sandbox is free and Limited Production gives 200 live calls per product. [Rate limits](https://plaid.com/docs/errors/rate-limit-exceeded/) are per Item and per client, for example 50 sync calls a minute per Item. A person whose data it is cannot export it; [Plaid Portal](https://plaid.com/legal/) lets US users see connections and delete data. The privacy policy (effective 8 December 2025) lists training AI systems as one use of data.

## Agents

Plaid ships two [MCP servers](https://plaid.com/docs/resources/mcp/): a hosted Dashboard server with diagnostics and usage tools, and a local [AI coding toolkit](https://github.com/plaid/ai-coding-toolkit) (MIT, May 2025) for Sandbox tokens, mock data and webhook simulation. Neither exposes an end user's money. Plaid's agent work is as plumbing for other companies' agents: Meta's Muse ([8 September 2026](https://plaid.com/blog/meta-muse/)), Sierra and Decagon. Approval is the one-time consent in Link.

## Customers

Developers like the coverage, the docs and the sync design. End users on [Trustpilot](https://www.trustpilot.com/review/plaid.com) rate it 1.2 out of 5 from 225 reviews, citing banks that will not link, dropped connections and support with no person behind it. Those reviewers are a skewed sample: people who had a problem and could not reach the app that sent them.

## History

Zach Perret and William Hockey started in 2012 building budgeting software and pivoted when bank connections proved the hard part ([Wikipedia](https://en.wikipedia.org/wiki/Plaid_Inc.)). Chase moved Plaid to a token API in [October 2018](https://media.chase.com/news/plaid-signs-data-agreement-with-jpmc). Visa's $5.3B bid fell to a DOJ antitrust suit in January 2021; three months later Plaid raised at $13.4B. A $58M class action over consent followed in 2021. It raised $575M at $6.1B in [April 2025](https://techcrunch.com/2025/04/03/fintech-plaid-raises-575m-at-6-1b-valuation-says-it-will-not-go-public-in-2025/) and an employee sale valued it at $8B in [February 2026](https://techcrunch.com/2026/02/26/plaid-valued-at-8b-in-employee-share-sale/). Banks now charge aggregators for access, starting with JPMorgan in 2025, which raises the cost of every read.

## For ownpurse

Plaid is the best public example of a bank change feed, and its three arrays map directly onto assert and retract. The lesson to keep is the pending-to-posted swap: Plaid deletes the pending record and issues a new id, so a record that keys on Plaid's id loses the history of what was pending. ownpurse should keep the pending fact, retract it, and link the posted one through `pending_transaction_id`. As a source, Plaid suits a hosted app more than a person on their own machine: full Production is a business application, and the price is negotiated. It is worth a connector for people who already have Plaid access, but it should not be the default bank witness.
