---
{
 "slug": "sage-accounting",
 "name": "Sage Accounting",
 "url": "https://www.sage.com/en-gb/sage-business-cloud/accounting/",
 "segment": "incumbent-ledger",
 "status": "active",
 "founded": "Sage Group 1981; Sage Accounting (formerly Sage One, then Sage Business Cloud Accounting) launched in the 2010s",
 "hq": "Newcastle upon Tyne, England (Sage Group)",
 "backing": "Product of The Sage Group plc, listed on the London Stock Exchange since 1989 and in the FTSE 100 since 1999. 2025 revenue £2,513 million.",
 "tagline": "Sage's cloud accounting software for UK and European small businesses and sole traders.",
 "scope": "Sage Accounting keeps a small business's books in Sage's cloud: invoicing, bank feeds and reconciliation, VAT and Making Tax Digital returns, reporting, and payroll as a separate product, with AutoEntry for document capture. It sits below Sage 50, Sage 200 and Sage Intacct in Sage's range. It does not give the owner an authoritative local record, and its API is available only in some regions.",
 "domains": [
  "ledger",
  "bank",
  "invoicing",
  "payroll",
  "documents",
  "tax",
  "reporting"
 ],
 "customers_served": [
  "freelancer",
  "smb",
  "accountants",
  "developers"
 ],
 "spectrum": {
  "custody": {
   "score": 1,
   "note": "Sage holds the authoritative books; exports and the API give copies, not the record."
  },
  "locality": {
   "score": 0,
   "note": "Sage Accounting is cloud only (Sage 50 is the desktop product)."
  },
  "history": {
   "score": 1,
   "note": "An audit trail exists in the product; the API returns current state."
  },
  "openness": {
   "score": 2,
   "note": "Public REST API (v3.1) with self-serve app registration; the product is closed and no official MCP server was found."
  },
  "breadth": {
   "score": 2,
   "note": "Books, banking, invoicing and UK tax, with payroll and AutoEntry capture as separate products."
  },
  "autonomy": {
   "score": 2,
   "note": "Sage Copilot and new agents (Accounts Payable, Close) automate parts of the work; most posting is still confirmed by a person, and the agents are deepest in Sage Intacct."
  },
  "agent_readiness": {
   "score": 2,
   "note": "Public API with OAuth; no official MCP server or agent-facing docs found as of 2026-10-10."
  },
  "integration_ease": {
   "score": 2,
   "note": "Self-serve developer portal, but the API covers only some regions (UK, Ireland, US, Canada, Germany, Spain, France), and the docs were hard to reach for this pass."
  },
  "market": {
   "score": 2,
   "note": "Small businesses and sole traders, mostly in the UK and Europe."
  }
 },
 "api": {
  "public": true,
  "style": "REST (Sage Accounting API v3.1, also called the Sage Business Cloud Accounting API)",
  "auth": "OAuth 2.0 with a client ID and secret from an app registered on the Sage developer portal",
  "limits": "Not confirmed for this pass.",
  "cost": "No developer usage fees found; registration on the developer portal is self-serve.",
  "export": "Reports and transaction lists export to CSV or Excel in the product; the API is the bulk path.",
  "notes": "Codat lists v3.1 and v3.0 support for the UK, Ireland, Germany, Spain, Portugal, France, the US, Canada and the UAE; Australia, New Zealand, South Africa and Asian editions are not supported. v3.1 credentials replaced v3.0 for setups from 19 October 2020."
 },
 "pricing": {
  "model": "Monthly subscription per business by plan, with payroll and AutoEntry billed separately.",
  "entry": "Not confirmed for this pass (Sage's pricing page blocked automated access).",
  "top": "Not confirmed for this pass.",
  "notes": "Sage says its 102% renewal rate by value in H1 FY26 reflects targeted price rises and upselling to an integrated small-business suite with AI. Trustpilot reviewers report steep increases (one from £74.40 to £108 a month in a year). AutoEntry is sold separately by credits, from US$13 a month for 50 credits."
 },
 "agents": {
  "mcp": "none",
  "ai_features": "Sage Copilot across its products, and agents announced in H1 FY26: Accounts Payable Agent, Close Agent, Finance Intelligence Agent and Sales Intelligence Agent; partner-built agents can run on the Sage Platform under new pricing models.",
  "agent_writes": "Through the public API, an app with write scope can create and update records; no agent-specific interface was found.",
  "approval": "No documented approval gate for API writes; Sage's own agents' approval design was not confirmed.",
  "notes": "No official or prominent community MCP server was found in Sage's GitHub organisation (2026-10-10). Most of Sage's agent work is aimed at Sage Intacct and the mid-market."
 },
 "integration": {
  "scope": "Sage Marketplace apps, bank feeds, AutoEntry capture, Sage Payroll, embedded partnerships for Sage Sole Trader (HSBC, Monzo, Tide, SumUp); supported by the main unified accounting APIs (Codat and others).",
  "notes": "Region-by-region editions and API coverage are the main friction; Australia, New Zealand, South Africa and Asia are out of the API's scope."
 },
 "preference": {
  "scale": "Sage Group H1 FY26 (six months to 31 March 2026): revenue £1,363 million, up 11%; Sage Business Cloud revenue £1,162 million; ARR £2,727 million. No customer count for Sage Accounting was disclosed.",
  "ratings": "Trustpilot (www.sage.com, all Sage products) 4.0 from 20,168 reviews (2026-10-10).",
  "sentiment": "Praised for patient onboarding and support staff and quick fixes. Complaints are about long phone waits, complex report layouts, price rises, and charges after cancelling; some reviewers name Xero as the cheaper alternative (Trustpilot)."
 },
 "timeline": [
  {
   "date": "1981",
   "event": "Sage founded in Newcastle upon Tyne by David Goldman, Paul Muller and Graham Wylie"
  },
  {
   "date": "1989",
   "event": "Listed on the London Stock Exchange"
  },
  {
   "date": "1999",
   "event": "Joined the FTSE 100"
  },
  {
   "date": "2017-07",
   "event": "Acquired Intacct for US$850 million"
  },
  {
   "date": "2018-11",
   "event": "Steve Hare became CEO"
  },
  {
   "date": "2020-10-19",
   "event": "API v3.1 credentials became the standard for new integrations"
  },
  {
   "date": "2025-07",
   "event": "Acquired Fyle (AI expense management)"
  },
  {
   "date": "2026-05-21",
   "event": "H1 FY26 results: Sage Copilot expanded, AP, Close and intelligence agents launched"
  },
  {
   "date": "2026-07-29",
   "event": "Nine-month trading update: revenue £2,062 million, up 11%"
  }
 ],
 "relation": "source",
 "relation_note": "A likely third ledger connector, for UK and EU small businesses. AutoEntry, which Sage owns, is also a possible source of captured documents.",
 "borrow": "Region-specific tax filing (Making Tax Digital) as a first-class part of the book; embedding accounting in banks for sole traders.",
 "avoid": "Price rises dressed up as renewal; API coverage that varies by region without a clear map.",
 "sources": [
  {
   "title": "Sage Group H1 FY26 results (21 May 2026)",
   "url": "https://www.investegate.co.uk/announcement/rns/sage-group--sge/results-for-the-six-months-to-31-march-2026/9578788"
  },
  {
   "title": "Sage trading update, nine months to 30 June 2026",
   "url": "https://www.investegate.co.uk/announcement/rns/sage-group--sge/trading-update-for-nine-months-ended-30-june-2026/9692278"
  },
  {
   "title": "Codat: Sage Accounting integration setup",
   "url": "https://docs.codat.io/integrations/accounting/sagebusinesscloud/accounting-sagebusinesscloud-setup"
  },
  {
   "title": "AutoEntry by Sage",
   "url": "https://www.autoentry.com/"
  },
  {
   "title": "Sage reviews (Trustpilot UK)",
   "url": "https://uk.trustpilot.com/review/www.sage.com"
  },
  {
   "title": "Sage on GitHub",
   "url": "https://github.com/Sage"
  },
  {
   "title": "Sage Group, Wikipedia",
   "url": "https://en.wikipedia.org/wiki/Sage_Group"
  },
  {
   "title": "Sage Accounting product page",
   "url": "https://www.sage.com/en-gb/sage-business-cloud/accounting/"
  }
 ],
 "unverified": [
  "Current Sage Accounting plan names and UK prices (sage.com blocked automated access).",
  "Sage Accounting API rate limits, developer terms on data use and AI training, and whether any usage fees apply (developer.sage.com blocked automated access).",
  "Launch date of Sage One and the dates of its renames to Sage Business Cloud Accounting and Sage Accounting.",
  "Date and price of Sage's AutoEntry acquisition (commonly given as 2019).",
  "Whether Sage Copilot is generally available in Sage Accounting for UK small businesses, and how its agents gate writes.",
  "Number of Sage Accounting customers (not disclosed separately).",
  "The Trustpilot score covers all Sage products, not Sage Accounting alone; G2 and Capterra ratings were not reachable.",
  "Whether Sage publishes an official or partner MCP server outside its public GitHub organisation."
 ],
 "researched": "2026-10-10"
}
---

## What it is

Sage Accounting is the cloud accounting product of The Sage Group, the FTSE 100 software company founded in Newcastle in 1981. It began as Sage One and was later sold as Sage Business Cloud Accounting. It is Sage's entry-level cloud ledger for small businesses and sole traders, mostly in the UK and Europe, and it sits below Sage 50 (desktop), Sage 200 and Sage Intacct. Sage Group reported [revenue of £1,363 million in the six months to 31 March 2026](https://www.investegate.co.uk/announcement/rns/sage-group--sge/results-for-the-six-months-to-31-march-2026/9578788), up 11%, with cloud native revenue up 25%.

What it does well: UK tax. Sage leans on Making Tax Digital for VAT, and now for Income Tax, and its Sage Sole Trader product is embedded in HSBC, Monzo, Tide and SumUp. Accountants and bookkeepers across the UK know it.

## How it works

The books live in Sage's cloud. You raise invoices, bring in bank feeds, reconcile and file VAT. Payroll is a separate product, and [AutoEntry](https://www.autoentry.com/), which Sage owns, captures receipts and statements on credits from US$13 a month for 50. AutoEntry also feeds Xero, QuickBooks and FreeAgent, not only Sage. Sage's pricing page blocked automated access for this pass, so plan prices are not given here.

## Data and access

The Sage Accounting API (v3.1) is REST with OAuth 2.0. You register an app on Sage's developer portal for a client ID and secret. [Codat's setup notes](https://docs.codat.io/integrations/accounting/sagebusinesscloud/accounting-sagebusinesscloud-setup) list support for the UK, Ireland, Germany, Spain, Portugal, France, the US, Canada and the UAE, and not for Australia, New Zealand, South Africa or Asian editions. Unlike Xero and Intuit, Sage has not announced metered or tiered API fees, as far as this pass found. Rate limits could not be confirmed because the developer docs blocked automated access. Bulk export is reports and lists to CSV or Excel, plus the API.

## Agents

Sage's AI story is Sage Copilot, plus agents named in its [H1 FY26 results](https://www.investegate.co.uk/announcement/rns/sage-group--sge/results-for-the-six-months-to-31-march-2026/9578788): an Accounts Payable Agent, a Close Agent, a Finance Intelligence Agent and a Sales Intelligence Agent. Partners can also build agents on the Sage Platform under new pricing models. Most of this targets Sage Intacct and the mid-market. No official MCP server was found in [Sage's GitHub organisation](https://github.com/Sage), and no agent-facing developer docs were found. For an outside agent, Sage Accounting is an OAuth REST API and nothing more, so it scores below Xero and QuickBooks on agent readiness.

## Customers

On [Trustpilot Sage scores 4.0 from 20,168 reviews](https://uk.trustpilot.com/review/www.sage.com), counting all its products. Reviewers praise patient support staff and onboarding. They complain about long phone waits, complex report layouts, and price rises (one went from £74.40 to £108 a month in a year). Some name Xero as the cheaper choice. Sage's own results credit its 102% renewal rate to targeted price rises and upselling.

## History

Sage grew on desktop accounting (Sage 50 and earlier lines), listed in 1989, bought Intacct in 2017 for US$850 million and has been moving customers to subscriptions and the cloud under CEO Steve Hare since 2018. In 2025 it bought Fyle for expense management. Sage Accounting has been its answer to Xero in UK small business. Xero has outgrown it in that segment by most accounts, though Sage still holds a large installed base through Sage 50.

## For ownpurse

Sage Accounting is a likely third ledger source, mostly for UK and EU books, and the API's unmetered access, as far as found, makes a full local copy cheap to keep. Region gaps mean a connector has to check which edition an owner is on. AutoEntry matters on its own as a capture source that sits beside any ledger. Sage has no agent interface for the small-business product, so ownpurse's CLI and approval gate would be the agent surface for a Sage user. Confirm the API's rate limits and terms on data use and AI before building.
