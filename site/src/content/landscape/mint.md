---
{
 "slug": "mint",
 "name": "Mint",
 "url": "https://mint.intuit.com",
 "segment": "personal-finance",
 "status": "shut",
 "founded": "2006",
 "hq": "Mountain View, California (as Mint.com; later part of Intuit)",
 "backing": "Raised over $31 million in venture capital (DAG Ventures, Shasta Ventures, First Round Capital, angels including Ram Shriram), last round $14 million on 2009-08-04. Intuit agreed to buy it for a reported $170 million (reported 2009-09-13, closed 2009-11-02). Shut by Intuit on 2024-03-23; users were directed to Credit Karma, which Intuit acquired in December 2020 for about $7.1 billion.",
 "tagline": "A free, ad- and offer-funded personal finance app from Intuit that aggregated a household's accounts, and was shut down in March 2024.",
 "scope": "Mint pulled bank, card, loan and investment accounts into one view, categorized transactions automatically, and offered budgets, bill tracking, net worth and a credit score. It was personal only: no double-entry books, no business ledger, no documents, and no public API. It no longer exists; Credit Karma took over a subset of it.",
 "domains": [
  "personal",
  "bank",
  "reporting"
 ],
 "customers_served": [
  "consumer"
 ],
 "spectrum": {
  "custody": {
   "score": 0,
   "note": "Intuit held the only complete copy; at shutdown only three years of transactions moved to Credit Karma and anything older survived only if the user exported it."
  },
  "locality": {
   "score": 0,
   "note": "Hosted web and mobile only."
  },
  "history": {
   "score": 1,
   "note": "Kept transaction history for years, but edits overwrote categories and there was no versioned record; the history ended with the service."
  },
  "openness": {
   "score": 0,
   "note": "Closed product, no public API; a transaction CSV download and a data-privacy request were the only ways out."
  },
  "breadth": {
   "score": 2,
   "note": "Banks, cards, loans, investments, bills and credit score for a household; no books, documents or business data."
  },
  "autonomy": {
   "score": 2,
   "note": "Synced and auto-categorized transactions unattended, but only inside its own read-only view of the user's money."
  },
  "agent_readiness": {
   "score": 0,
   "note": "No API; it ended before agents mattered, and unofficial access relied on scraping a user session."
  },
  "integration_ease": {
   "score": 0,
   "note": "No developer program; third parties could not connect except by scraping."
  },
  "market": {
   "score": 0,
   "note": "Consumers and households."
  }
 },
 "api": {
  "public": false,
  "style": "None public. Internal web endpoints only, used unofficially by scrapers.",
  "auth": "n/a (user login only)",
  "limits": "n/a",
  "cost": "The app was free to users, funded by advertising and financial product offers.",
  "export": "Transaction CSV download in the web app; data copy or deletion by request at accounts.intuit.com under data privacy. After 2024-03-23 the app is gone.",
  "notes": "Account aggregation was first powered by Yodlee and later by Intuit's own connection system."
 },
 "pricing": {
  "model": "Free, monetized through ads and financial product referrals",
  "entry": "Free",
  "top": "Free",
  "notes": "The free model is part of why it ended: the product did not earn its keep inside Intuit, and long-time users described it as neglected for years before the shutdown (per NBC/CNBC coverage, 2023-11)."
 },
 "agents": {
  "mcp": "n/a",
  "ai_features": "None of note; rule-based auto-categorization.",
  "agent_writes": "No; there was no API.",
  "approval": "n/a",
  "notes": "Shut before agent tooling arrived."
 },
 "integration": {
  "scope": "In 2010 it connected to over 16,000 US and Canadian institutions and more than 17 million accounts (Wikipedia, citing Mint). No outbound integrations.",
  "notes": "Inbound only, through an aggregator; nothing could read from Mint."
 },
 "preference": {
  "scale": "Over 20 million users claimed in 2016; 13 million registered users in 2020 (Wikipedia); an estimated 3.6 million active users in 2021 (Bloomberg, via CNBC 2023-11). Under Val Agostino's tenure as an early product leader it reached 2 million users before the 2009 sale (Monarch's own account).",
  "ratings": "Not applicable now; the app listings are gone.",
  "sentiment": "Loved as the free default for over a decade; at the end users said it had felt abandoned for years, and the shutdown drew wide disappointment because Credit Karma 'does not currently provide budgeting features the same way that Mint has' (Intuit, 2023-11) and carried over only three years of transactions."
 },
 "timeline": [
  {
   "date": "2006",
   "event": "Founded by Aaron Patzer."
  },
  {
   "date": "2009-08-04",
   "event": "Closes a final $14 million round; over $31 million raised in total."
  },
  {
   "date": "2009-09-13",
   "event": "Intuit reported to buy Mint for $170 million; deal closes 2009-11-02."
  },
  {
   "date": "2010",
   "event": "Connects to over 16,000 institutions and 17 million accounts."
  },
  {
   "date": "2016",
   "event": "Over 20 million users claimed."
  },
  {
   "date": "2020-12",
   "event": "Intuit acquires Credit Karma for about $7.1 billion."
  },
  {
   "date": "2023-11",
   "event": "Intuit announces Mint will close and moves users to Credit Karma, first naming January 1, 2024 (December 31, 2023)."
  },
  {
   "date": "2024-03-23",
   "event": "Mint shuts down for good."
  }
 ],
 "relation": "cautionary-tale",
 "relation_note": "Mint is the clearest case of what ownpurse exists to prevent: millions of households kept their financial history in a free hosted tool, and when the owner decided the product no longer fit, the history beyond three years was gone unless each person had exported it.",
 "borrow": "The single household view across every account, automatic categorization, and an onboarding that made connecting accounts feel effortless.",
 "avoid": "A vendor-held only copy, a business model that does not pay for the product, and no API: the user could neither leave cleanly nor let another tool read their data.",
 "sources": [
  {
   "title": "Mint.com (Wikipedia)",
   "url": "https://en.wikipedia.org/wiki/Mint.com"
  },
  {
   "title": "Budgeting app Mint is shutting down (NBC/CNBC, 2023-11)",
   "url": "https://www.nbcmiami.com/news/business/money-report/budgeting-app-mint-is-shutting-down-to-the-disappointment-of-loyal-users-this-is-the-peril-of-relying-on-free-services/3153432/"
  },
  {
   "title": "Mint shutting down: how to migrate to Monarch (Monarch, 2023-11-09)",
   "url": "https://www.monarch.com/mint-shutting-down-how-to-migrate-to-monarch-with-discount"
  },
  {
   "title": "Credit Karma (Wikipedia)",
   "url": "https://en.wikipedia.org/wiki/Credit_Karma"
  },
  {
   "title": "Mint vs Credit Karma: what changed after the shutdown (Plisio)",
   "url": "https://plisio.net/tradfi/mint-vs-credit-karma"
  }
 ],
 "unverified": [
  "The final shutdown date of 2024-03-23 comes from Wikipedia and one secondary explainer; Intuit's own notice was not retrieved.",
  "Whether the accounts.intuit.com data copy request still works after shutdown.",
  "The exact date Mint moved from Yodlee to Intuit's own aggregation.",
  "Whether Mint's transaction CSV export stayed available up to the final day."
 ],
 "researched": "2026-10-10"
}
---

## What it is

Mint was a free personal finance app. You linked your bank, card, loan and investment accounts, and it showed balances, categorized spending, budgets, bills, net worth and a credit score in one place. Aaron Patzer founded it in 2006, and [Intuit bought it in 2009 for a reported $170 million](https://en.wikipedia.org/wiki/Mint.com). It shut down on March 23, 2024.

## How it works

It worked as a hosted aggregator. Mint stored your bank credentials, at first through Yodlee and later through Intuit's own connection system, pulled transactions on a schedule and categorized them with rules. You edited categories in place. Advertising and financial product offers paid for it. It never kept a versioned record: what you saw was the current state, held only by Intuit.

## Data and access

Mint never had a public API. A transaction CSV download and a data request at accounts.intuit.com were the only ways out. When Intuit [announced the closure in November 2023](https://www.nbcmiami.com/news/business/money-report/budgeting-app-mint-is-shutting-down-to-the-disappointment-of-loyal-users-this-is-the-peril-of-relying-on-free-services/3153432/), the migration to Credit Karma carried "the majority of their Mint financial account balances, historical net worth and three years of transactions." Anything older survived only if you had exported it. Budgets did not carry over at all, because Credit Karma "does not currently provide budgeting features the same way that Mint has in the past." Intuit first named January 1, 2024 as the end date and later moved it to March 23, 2024.

## Agents

Mint offered nothing for agents. It closed before agent tooling mattered, and the only unofficial access came from scraping a logged-in session.

## Customers

Mint served consumers and households. Wikipedia records over 20 million users claimed in 2016 and 13 million registered in 2020, and Bloomberg estimated 3.6 million active users in 2021. In 2010 it connected to over 16,000 institutions. People loved it for a decade as the free default. By the end, long-time users said it felt abandoned.

## History

Mint raised over $31 million before Intuit bought it. Inside Intuit it went years with few updates. Intuit bought Credit Karma in December 2020 for about $7.1 billion and in 2023 folded Mint's users into it. [Monarch](https://www.monarch.com/mint-shutting-down-how-to-migrate-to-monarch-with-discount), founded by Mint's early product leader Val Agostino, built a Mint importer and became the main successor. YNAB, Copilot and others also courted the displaced users.

## For ownpurse

Mint is the cautionary tale behind the sovereignty principle. A household's whole financial history sat in a free hosted tool with no API. When the owner changed its strategy, the history was cut to three years and the budgets disappeared. You could not leave cleanly, and no other tool could read your data on your behalf.

There are three lessons. The authoritative copy should be a file the person holds. Every connector should write facts the person keeps even after the source goes away. Leaving should be one command, not a privacy request. Mint did one thing ownpurse should match: it put every account in one view with almost no setup. A local record has to feel just as easy to connect, or people will choose the hosted convenience again.
