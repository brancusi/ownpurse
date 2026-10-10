---
{
 "slug": "truelayer",
 "name": "TrueLayer",
 "url": "https://truelayer.com",
 "segment": "bank-aggregator",
 "status": "active",
 "founded": "2016 (London). The 2016 date comes from secondary knowledge; the first public API launch was 8 August 2017.",
 "hq": "London, UK, with a European hub in Dublin and an engineering hub in Milan.",
 "backing": "Venture-backed. Series D $70m led by Addition (8 April 2021); about $130m round with Tiger Global and Stripe as new investors (20 September 2021); $27m led by CDP Venture Capital (8 October 2026). Early backers include Connect Ventures. FCA-authorised EMI, FRN 901096.",
 "tagline": "A UK and European open banking platform for Pay by Bank payments, payouts and account data, built on PSD2 and UK Open Banking.",
 "scope": "TrueLayer connects to banks through regulated open banking APIs to initiate payments (single, VRP, Bank on File), send payouts, verify accounts and read account data. Today it is mostly a payments network (eBay, bet365, Stripe in some markets). The Data API is still offered, but Data v3 is UK-only and non-regulated clients must use TrueLayer's hosted consent flow. It does not keep books or categorise to a chart of accounts.",
 "domains": [
  "bank",
  "payments"
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
   "note": "The bank holds the record; TrueLayer passes data through under consent that lapses after 90 days unless reconfirmed."
  },
  "locality": {
   "score": 0,
   "note": "Cloud API only."
  },
  "history": {
   "score": 1,
   "note": "Current accounts, balances and transactions on request; no versions or as-of queries."
  },
  "openness": {
   "score": 2,
   "note": "Public docs (Markdown and llms.txt), free self-serve sandbox; production needs onboarding and a contract, and pricing is not public."
  },
  "breadth": {
   "score": 1,
   "note": "Bank data plus payments and payouts; no documents, books or investments."
  },
  "autonomy": {
   "score": 1,
   "note": "Can refresh data while the user is absent under recurring consent; nothing posts to books."
  },
  "agent_readiness": {
   "score": 3,
   "note": "Experimental official MCP server, llms.txt and .md docs; selected for the FCA's agentic-payments sandbox in July 2026."
  },
  "integration_ease": {
   "score": 2,
   "note": "Sign up to Console with Google or GitHub and test in sandbox at once; going live needs sales, licences or TrueLayer's flow, and reviewers report rejections."
  },
  "market": {
   "score": 3,
   "note": "Merchants from mid-size to eBay and bet365; also iGaming, investing and utilities."
  }
 },
 "api": {
  "public": true,
  "style": "REST with JSON; webhooks for payments, payouts, mandates and refunds; SDKs for Node.js, Python, Java and .NET.",
  "auth": "OAuth 2.0: client credentials for the client, auth-code flow with refresh_token (offline_access) for user data; payment requests are signed with your private key.",
  "limits": "Consent typically lasts 90 days; in the UK it must be reconfirmed at least every 90 days (the /connections/extend endpoint). Data v3 is UK-only. Rate limits are not published.",
  "cost": "Not public. No pricing page (truelayer.com/pricing returns 404); priced through sales.",
  "export": "No bulk export of end-user data beyond the API; Console offers payments reporting and SFTP report delivery for merchants.",
  "notes": "Data v1 documents amounts as JSON 'number (a float number)', so a client that needs exact cents must parse carefully."
 },
 "pricing": {
  "model": "Custom, contract-based; per-transaction for payments, per-connection or per-call for data (terms not published).",
  "entry": "Free sandbox for any Console account.",
  "top": "Enterprise contracts.",
  "notes": "No public price list found on 10 October 2026."
 },
 "agents": {
  "mcp": "official",
  "ai_features": "Transaction categorisation and merchant names in Data; no customer-facing AI product. Publishes positions on agentic payments (levels of autonomy, July and August 2026).",
  "agent_writes": "The experimental MCP server can create payment links and payouts from a merchant account; TrueLayer recommends testing it in sandbox.",
  "approval": "Every Pay by Bank payment is approved by the payer in their bank app (strong customer authentication). TrueLayer says fully autonomous agent payments need VRP mandates and regulatory change.",
  "notes": "The MCP server (github.com/TrueLayer/truelayer_mcp, created April 2025) is labelled a demonstration, not supported or contractually provided."
 },
 "integration": {
  "scope": "Live in 22 countries (October 2026); more than 30 million users on its network (October 2026, up from 20 million in December 2025). Plugins for Shopify and Magento.",
  "notes": "Acquired Zimpler (completed 3 March 2026) for the Nordics and in3 (May 2026) for Pay by Bank credit in the Netherlands."
 },
 "preference": {
  "scale": "30m+ network users and 22 countries (8 October 2026). In April 2021 it said it processed over half of open banking volume in the UK, Ireland and Spain.",
  "ratings": "Trustpilot 1.9 out of 5 from 45 reviews (October 2026), mostly from end users paying merchants. G2 not reachable.",
  "sentiment": "Consumers complain about confusing bank-redirect flows, lost-payment disputes and unreachable support (TrueLayer replies that it does not hold funds). Prospective customers complain of automated rejections and slow sales. Paying users praise speed and simplicity."
 },
 "timeline": [
  {
   "date": "2017-08-08",
   "event": "Launches a REST API for UK bank and card data in beta."
  },
  {
   "date": "2021-04-08",
   "event": "Raises a $70m Series D led by Addition."
  },
  {
   "date": "2021-09-20",
   "event": "Raises about $130m with Tiger Global and Stripe as new investors."
  },
  {
   "date": "2025-04",
   "event": "Publishes an experimental MCP server for AI assistants."
  },
  {
   "date": "2025-10-21",
   "event": "Announces the Zimpler acquisition."
  },
  {
   "date": "2025-12-02",
   "event": "Reaches 20 million network users."
  },
  {
   "date": "2026-02-19",
   "event": "eBay adds TrueLayer Pay by Bank at checkout."
  },
  {
   "date": "2026-03-03",
   "event": "Completes the Zimpler acquisition."
  },
  {
   "date": "2026-05-29",
   "event": "Acquires in3, adding credit."
  },
  {
   "date": "2026-07-22",
   "event": "Selected for the FCA Supercharged Sandbox to test agentic payments."
  },
  {
   "date": "2026-10-08",
   "event": "Raises $27m led by CDP Venture Capital; 30m+ users, 22 countries."
  }
 ],
 "relation": "source",
 "relation_note": "A possible UK and EU bank source for ownpurse, via an owner's own app registration or a partner, but it is built for merchants and regulated firms, not for one person's books.",
 "borrow": "Explicit consent lifetimes and reconfirmation, an llms.txt with Markdown docs for agents, and a clear statement that a human approves each payment in their bank.",
 "avoid": "Float amounts in JSON, opaque contract pricing, and a hosted consent flow that a non-regulated client cannot change.",
 "sources": [
  {
   "title": "TrueLayer about us (FCA authorisation)",
   "url": "https://truelayer.com/about/"
  },
  {
   "title": "TrueLayer docs home",
   "url": "https://docs.truelayer.com/"
  },
  {
   "title": "TrueLayer docs llms.txt",
   "url": "https://docs.truelayer.com/llms.txt"
  },
  {
   "title": "Data API v1 overview",
   "url": "https://docs.truelayer.com/docs/data-api-basics.md"
  },
  {
   "title": "General Data API concepts (consent, 90 days)",
   "url": "https://docs.truelayer.com/docs/general-data-api-concepts.md"
  },
  {
   "title": "Data API v3 (UK only)",
   "url": "https://docs.truelayer.com/docs/enable-your-users-to-connect-their-bank-account"
  },
  {
   "title": "Create a Console account",
   "url": "https://docs.truelayer.com/docs/quickstart-create-a-console-account.md"
  },
  {
   "title": "TrueLayer MCP server for AI assistants",
   "url": "https://docs.truelayer.com/docs/truelayer-mcp-integration-for-claude-ai.md"
  },
  {
   "title": "Three levels of agentic autonomy (14 Aug 2026)",
   "url": "https://truelayer.com/blog/payments/the-three-levels-of-agentic-autonomy-and-how-pay-by-bank-fits/"
  },
  {
   "title": "TrueLayer newsroom",
   "url": "https://truelayer.com/newsroom/"
  },
  {
   "title": "Introducing TrueLayer (8 Aug 2017)",
   "url": "https://truelayer.com/newsroom/announcements/introducing-truelayer/"
  },
  {
   "title": "Series D $70m (8 Apr 2021)",
   "url": "https://truelayer.com/newsroom/announcements/seriesd-fundraise-rewiring-finance/"
  },
  {
   "title": "Fundraise with Tiger Global and Stripe (20 Sep 2021)",
   "url": "https://truelayer.com/newsroom/announcements/130m-fundraise-to-accelerate-open-banking-payments/"
  },
  {
   "title": "TrueLayer to acquire Zimpler (21 Oct 2025)",
   "url": "https://truelayer.com/newsroom/announcements/truelayer-to-acquire-zimpler/"
  },
  {
   "title": "CDP Venture Capital invests $27m (8 Oct 2026)",
   "url": "https://truelayer.com/newsroom/announcements/italys-cdp-venture-capital-invests-in-truelayer/"
  },
  {
   "title": "Trustpilot: TrueLayer",
   "url": "https://www.trustpilot.com/review/truelayer.com"
  }
 ],
 "unverified": [
  "Founding year 2016 and co-founder Luca Martinetti (only Francesco Simoneschi is named in the pages read).",
  "The September 2021 round size ($130m appears only in the page URL) and the reported valuation of over $1bn.",
  "Total funding to date and current employee count (the October 2026 release says 70 in Italy is about a quarter of staff).",
  "Any layoffs in 2022 to 2024.",
  "Data API pricing and rate limits.",
  "G2 and Capterra ratings (sites not reachable)."
 ],
 "researched": "2026-10-10"
}
---

## What it is

TrueLayer is a London open banking company. It started in 2017 as a [REST API for UK bank data](https://truelayer.com/newsroom/announcements/introducing-truelayer/) and has become mostly a Pay by Bank payments network. On 8 October 2026 it said it was [live in 22 countries with more than 30 million users](https://truelayer.com/newsroom/announcements/italys-cdp-venture-capital-invests-in-truelayer/). It is authorised by the FCA as an electronic money institution ([about page](https://truelayer.com/about/), FRN 901096).

## How it works

Unlike Teller, TrueLayer uses the regulated APIs banks must provide under PSD2 and UK Open Banking. The [product docs](https://docs.truelayer.com/) cover payments, Signup+, Bank on File, payouts, verification, payment links and data. Payments go to the payer's bank app for approval. Data needs the user's consent, which in the UK [must be reconfirmed at least every 90 days](https://docs.truelayer.com/docs/general-data-api-concepts.md). Elsewhere the bank authentication itself usually expires after 90 days.

## Data and access

Anyone can [create a Console account](https://docs.truelayer.com/docs/quickstart-create-a-console-account.md) with Google or GitHub and start in the sandbox. The [Data API v1](https://docs.truelayer.com/docs/data-api-basics.md) returns accounts, cards, balances, transactions, standing orders and direct debits. TrueLayer now points new integrators to [Data v3](https://docs.truelayer.com/docs/enable-your-users-to-connect-their-bank-account), which is UK-only. Clients that are not regulated as account information providers must use TrueLayer's hosted flow. One detail matters for an exact ledger: v1 documents its number type as a float. Production access and pricing go through sales. truelayer.com/pricing returns 404, and no price list is published.

## Agents

TrueLayer is the most agent-aware of the three. Its docs carry an [llms.txt](https://docs.truelayer.com/llms.txt) and Markdown versions of every page. It publishes an [experimental MCP server](https://docs.truelayer.com/docs/truelayer-mcp-integration-for-claude-ai.md) that can read a merchant account, list transactions, create payment links and send payouts, labelled as unsupported and best used in sandbox. In July 2026 the FCA [selected it for its Supercharged Sandbox](https://truelayer.com/newsroom/) to test agent-initiated payments. Its [August 2026 post](https://truelayer.com/blog/payments/the-three-levels-of-agentic-autonomy-and-how-pay-by-bank-fits/) is plain that only human-present agent payments work today. Fully autonomous payments need VRP mandates and regulatory change.

## Customers

Customers include eBay (February 2026), bet365, Stripe in several markets, SumUp and Trading 212. On [Trustpilot](https://www.trustpilot.com/review/truelayer.com) it has 1.9 out of 5 from 45 reviews. Most come from consumers caught in bank-redirect flows or payment disputes, and a few from businesses that could not get past sales.

## History

It raised a [$70m Series D led by Addition](https://truelayer.com/newsroom/announcements/seriesd-fundraise-rewiring-finance/) in April 2021, then a [round with Tiger Global and Stripe](https://truelayer.com/newsroom/announcements/130m-fundraise-to-accelerate-open-banking-payments/) in September 2021 (the page's URL says $130m). It acquired Zimpler (completed March 2026) and in3 (May 2026), and raised $27m led by CDP Venture Capital in October 2026. The arc runs from data API to payments network, the same drift Plaid, Tink and GoCardless made.

## For ownpurse

TrueLayer is a credible regulated source of UK and EU bank lines, and its approval model (the payer approves in their own bank) matches ownpurse's rule that agents propose and people post. But it is built for merchants and regulated firms. A person who wants their own transactions faces contract pricing, a hosted consent screen and 90-day reconfirmation. ownpurse should treat it like any provider: keep what it reports in the owner's log, parse amounts as exact decimals, and record consent expiry as a fact.
