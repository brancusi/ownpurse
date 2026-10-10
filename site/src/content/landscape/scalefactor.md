---
{
 "slug": "scalefactor",
 "name": "ScaleFactor",
 "url": "https://en.wikipedia.org/wiki/ScaleFactor",
 "segment": "bookkeeping-service",
 "status": "shut",
 "founded": "2014",
 "hq": "Austin, Texas, US",
 "backing": "Venture-backed, about $100M to $103M raised, about $90M of it in 2019: $2.5M seed via Techstars Austin (2017), $10M led by Canaan (2018), $30M from Bessemer Venture Partners, and a $60M Series C led by Coatue (August 2019). Other investors included Broadhaven, Firebrand, Vulcan, Stripes and NextPlay. Wound down June to August 2020.",
 "tagline": "An Austin startup that sold automated bookkeeping to small businesses as AI-driven software, raised about $100M, and shut in 2020 after reports that much of the work was done by hand.",
 "scope": "ScaleFactor sold bookkeeping that it said classified transactions, updated books daily, kept the general ledger and produced reports, on top of QuickBooks and Xero, plus bill pay, payroll and a commercial card. Former customers and staff told Forbes the software never did most of this and accountants, in Austin and at an outsourcing firm in the Philippines, did the work. It had no public API or agent interface.",
 "domains": [
  "ledger",
  "close",
  "reporting",
  "payments",
  "payroll",
  "spend"
 ],
 "customers_served": [
  "smb"
 ],
 "spectrum": {
  "custody": {
   "score": 1,
   "note": "Books were kept in the customer's QuickBooks or Xero, so a copy existed outside ScaleFactor, but ScaleFactor held the work and customers were moved to other services when it closed."
  },
  "locality": {
   "score": 0,
   "note": "Cloud service only."
  },
  "history": {
   "score": 1,
   "note": "No versioned record of its own is known; only the underlying ledger's audit trail."
  },
  "openness": {
   "score": 0,
   "note": "Closed; no public API was documented."
  },
  "breadth": {
   "score": 3,
   "note": "Bookkeeping, reporting, bill pay, payroll and a commercial card under one subscription."
  },
  "autonomy": {
   "score": 1,
   "note": "Sold as automated, but reporting says accountants did most of the work and the classifier could not sort transactions reliably."
  },
  "agent_readiness": {
   "score": 0,
   "note": "No API, CLI or agent interface; it predates MCP."
  },
  "integration_ease": {
   "score": 1,
   "note": "Connected to QuickBooks and Xero for its own use; nothing was offered for third parties to build on."
  },
  "market": {
   "score": 2,
   "note": "US small and mid-sized businesses, on flat fees of about $6,000 to $30,000 a year."
  }
 },
 "api": {
  "public": false,
  "style": "None documented.",
  "auth": "Not applicable.",
  "limits": "Not applicable.",
  "cost": "Not applicable.",
  "export": "No export terms found; books lived in the customer's QuickBooks or Xero, and ScaleFactor said it would transition customers to other services in July and August 2020.",
  "notes": "The company is gone; nothing is live to test."
 },
 "pricing": {
  "model": "Flat-fee annual packages.",
  "entry": "About $6,000 a year (PYMNTS, June 2020).",
  "top": "About $30,000 a year (PYMNTS, June 2020).",
  "notes": "No official price list survives in the sources found."
 },
 "agents": {
  "mcp": "n/a",
  "ai_features": "Marketed AI transaction classification, daily-updated books, automated reports and mobile alerts, said to be 'powered by AI and accounting experts'.",
  "agent_writes": "No external agents. The 'automation' that wrote to customers' books was, per Forbes's sources, largely people.",
  "approval": "None described for customers; bookkeeping was done for them.",
  "notes": "The case is about the gap between what was sold as software and what was delivered as labour."
 },
 "integration": {
  "scope": "QuickBooks and Xero; its own commercial card (announced October 2019) fed spend data into the platform.",
  "notes": "No developer program."
 },
 "preference": {
  "scale": "About $7M ARR at the end of 2019; about 100 employees before cutting 40 in February 2020 (Silicon Hills News). The CEO said COVID-19 halved ARR by June 2020.",
  "ratings": "No surviving review-site ratings were found.",
  "sentiment": "Former customers told Forbes (July 2020) their books came back full of errors, they had to rehire accountants or fix the books themselves, some cancelled and asked for refunds, and they saw it as a bookkeeping service, not software. The CEO's stated view was that software alone was not enough and customers wanted live people, which was hard to provide at scale."
 },
 "timeline": [
  {
   "date": "2014",
   "event": "Founded in Austin by Kurt Rathmann (CEO) and Corey Bach (CFO)."
  },
  {
   "date": "2017",
   "event": "Graduated from Techstars Austin with a $2.5M seed; first product launched."
  },
  {
   "date": "2018",
   "event": "$10M round from Canaan Partners."
  },
  {
   "date": "2019",
   "event": "$30M from Bessemer Venture Partners; named to the Austin Chamber's A-List."
  },
  {
   "date": "2019-08",
   "event": "$60M Series C led by Coatue, about $100M raised in roughly a year."
  },
  {
   "date": "2019-10",
   "event": "Commercial card for small businesses announced."
  },
  {
   "date": "2020-02",
   "event": "About 40 of about 100 staff laid off."
  },
  {
   "date": "2020-06-23",
   "event": "CEO tells staff half the team is let go that day and most operations stop on 28 August; COVID-19 blamed. Forbes reports first."
  },
  {
   "date": "2020-07-20",
   "event": "Forbes reports problems predated the pandemic and much of the bookkeeping was done by accountants in Austin and at The Outsourced Accountant in the Philippines."
  },
  {
   "date": "2020-08-28",
   "event": "Most operations cease."
  }
 ],
 "relation": "cautionary-tale",
 "relation_note": "ScaleFactor is the earlier wave's warning: it sold automation that was mostly people, the customer could not see which, and the books came back wrong. ownpurse answers both halves: every write is a reviewable package a person approves, and every fact records which source or agent produced it.",
 "borrow": "The CEO's own lesson, that small businesses wanted a person in the loop, is worth keeping; ownpurse's 'agents propose, people post' builds that in rather than hiding it.",
 "avoid": "Claiming automation the software does not do, hiding who (or what) did the work, raising and selling ahead of a product that is accurate to the cent, and leaving customers to find the errors after the fact.",
 "sources": [
  {
   "title": "G2 Research: ScaleFactor raises $100 million then abruptly shuts down",
   "url": "https://research.g2.com/insights/scalefactor-raises-100-million-then-abruptly-shuts-down"
  },
  {
   "title": "Tech Brew: ScaleFactor reportedly struggled to develop the software it promised (July 2020)",
   "url": "https://www.techbrew.com/stories/2020/07/22/scalefactor-reportedly-struggled-develop-software-promised-customers"
  },
  {
   "title": "PYMNTS: SMB accounting automation firm ScaleFactor to close (June 2020)",
   "url": "https://pymnts.com/?p=848970"
  },
  {
   "title": "Silicon Hills News: ScaleFactor shutdown",
   "url": "https://siliconhillsnews.com/?p=22668"
  },
  {
   "title": "Fintech Futures: ScaleFactor winds down after COVID-19 cripples sales (June 2020)",
   "url": "https://www.fintechfutures.com/fintech/scalefactor-winds-down-after-covid-19-cripples-sales"
  },
  {
   "title": "Forbes: ScaleFactor raised $100 million in a year, then blamed COVID-19 for its demise (July 2020)",
   "url": "https://www.forbes.com/sites/kenrickcai/2020/07/20/scalefactor-raised-100-million-in-a-year-then-blamed-covid-19-for-its-demise-employees-say-it-had-much-bigger-problems/"
  }
 ],
 "unverified": [
  "The Forbes articles themselves could not be fetched (403); their content is taken from G2 Research, Tech Brew and other summaries.",
  "No Wikipedia article exists at the given URL (404).",
  "Layoff count: PYMNTS and Silicon Hills say half of about 100 staff on 23 June 2020; G2's summary says about 100 were laid off.",
  "Total raised: about $100M (PYMNTS, G2) or $103M (Silicon Hills); the year of the Bessemer $30M round is not stated in the sources found.",
  "The manual-work allegations rest on anonymous former staff and customers; ScaleFactor attributed the closure to COVID-19.",
  "No surviving review-site ratings or official price list were found.",
  "Search budget ran out before every point could be cross-checked."
 ],
 "researched": "2026-10-10"
}
---

## What it is

ScaleFactor was an Austin startup, founded in 2014 by Kurt Rathmann and Corey Bach, that sold bookkeeping and back-office software to small and mid-sized US businesses. It said its product classified transactions, updated books daily, kept the general ledger and produced reports on top of QuickBooks and Xero, with bill pay, payroll and later a commercial card, all "powered by AI and accounting experts" ([G2 Research, September 2020](https://research.g2.com/insights/scalefactor-raises-100-million-then-abruptly-shuts-down)). It shut in August 2020. No Wikipedia article for it was found at the given address.

## How it works

Customers paid flat annual packages of roughly $6,000 to $30,000 ([PYMNTS, June 2020](https://pymnts.com/?p=848970)). The books stayed in the customer's QuickBooks or Xero, and ScaleFactor's platform and staff worked on them.

After the shutdown, Forbes reported (20 July 2020, summarised by [G2 Research](https://research.g2.com/insights/scalefactor-raises-100-million-then-abruptly-shuts-down) and [Tech Brew](https://www.techbrew.com/stories/2020/07/22/scalefactor-reportedly-struggled-develop-software-promised-customers)) that the AI could not reliably sort transactions or produce accurate reports. Much of the work was done by dozens of accountants at the Austin office and by The Outsourced Accountant, a firm in the Philippines. Customers said their books came back full of errors, and some rehired accountants or asked for refunds. These accounts come from anonymous former staff and customers; the company's public explanation was different.

## Data and access

There was no public API, export program or developer access. Because the ledger was the customer's QuickBooks or Xero file, customers kept a copy when the company closed, and ScaleFactor said it would move them to other services in July and August 2020 ([Silicon Hills News](https://siliconhillsnews.com/?p=22668)). That is a better exit than Bench gave its customers in 2024, though what the books contained was the problem.

## Agents

None. It predates MCP and offered no programmatic surface. Its relevance to agents is the claim itself: "AI" on the label, people doing the work, and no way for the customer to tell which.

## Customers

It reached about $7M in annual recurring revenue by the end of 2019, with about 100 staff, and cut about 40 in February 2020. In June 2020 Rathmann said COVID-19 had wiped out nearly half of revenue. No review-site ratings survive in the sources found.

## History

ScaleFactor graduated from Techstars Austin in 2017 with a $2.5M seed, raised $10M from Canaan in 2018 and $30M from Bessemer, then a $60M Series C led by Coatue in August 2019: about $100M in about a year, from Coatue, Bessemer, Canaan, Vulcan, Stripes and others. It announced a small-business card in October 2019. On 23 June 2020 Rathmann told staff that half the team would go that day and most operations would stop on 28 August, saying "it's not the outcome we wanted, but it's the fiscally responsible CEO thing to do". He also said software alone was not enough: customers wanted live people, and that was hard to provide at scale. Forbes reported in July that the problems predated the pandemic. Its reports pointed to the same thing.

## For ownpurse

ScaleFactor is the earlier wave's cautionary tale, and it bears on ownpurse directly. Its failure was not that people did bookkeeping; it was that the customer was told software did it, could not see who did, and found the errors later. Two ownpurse rules answer this. Every fact carries its provenance, so you can see whether a source, a rule, an agent or a person produced it. And the ledger is rebuilt and checked against the source to the cent, so errors surface before anyone relies on them. The CEO's own conclusion, that small businesses want a person in the loop, is the case for "agents propose, people post": put the person there openly, at the approval step, instead of behind a label.
