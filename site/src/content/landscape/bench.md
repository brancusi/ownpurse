---
{
 "slug": "bench",
 "name": "Bench",
 "url": "https://bench.co",
 "segment": "bookkeeping-service",
 "status": "acquired",
 "founded": "2012",
 "hq": "Vancouver, British Columbia (founded as 10sheet Inc.; owned since January 2025 by Employer.com, San Francisco)",
 "backing": "Raised over $100M to $113M (reports differ), including a $60M Series C in mid-2021 ($37M equity, $23M debt per PitchBook, $232M post-money); investors included Bain Capital Ventures, Inovia, Altos, Shopify, BMO, Sage, Silicon Valley Bank. Acquired by Employer.com, closed 1 January 2025, terms undisclosed.",
 "tagline": "An online bookkeeping and tax service for US small businesses that shut without notice on 27 December 2024 with its clients' books in a proprietary ledger, then was bought by Employer.com and reopened.",
 "scope": "Bench paired a human bookkeeping team with its own software and ledger to produce monthly statements and tax filings for small businesses, fed by bank, card and commerce integrations (Shopify, Stripe, Square, PayPal). It did not run on QuickBooks or Xero, and it had no public API. Under Employer.com it sells bookkeeping, tax and a QuickBooks-native fractional bookkeeper service.",
 "domains": [
  "ledger",
  "bank",
  "tax",
  "reporting",
  "commerce"
 ],
 "customers_served": [
  "freelancer",
  "smb"
 ],
 "spectrum": {
  "custody": {
   "score": 0,
   "note": "Books lived in Bench's proprietary ledger; when the platform went dark clients had no copy and only a download window to March 2025."
  },
  "locality": {
   "score": 0,
   "note": "Hosted only."
  },
  "history": {
   "score": 0,
   "note": "No evidence of a client-accessible history or audit log; exports were statements and documents."
  },
  "openness": {
   "score": 1,
   "note": "No API; a data download existed, and after the shutdown competitors had to build tools to move the exports into QuickBooks."
  },
  "breadth": {
   "score": 2,
   "note": "Bookkeeping and tax with bank, card and e-commerce feeds; no payroll, payments or spend."
  },
  "autonomy": {
   "score": 1,
   "note": "Bookkeepers did the work with Bench software; automation assisted rather than wrote unattended."
  },
  "agent_readiness": {
   "score": 0,
   "note": "No API, no agent interface."
  },
  "integration_ease": {
   "score": 1,
   "note": "Read-only inbound connections to banks and commerce platforms; nothing outbound for a third party."
  },
  "market": {
   "score": 1,
   "note": "Sole proprietors and small US businesses; the Light plan is for under $250K annual revenue."
  }
 },
 "api": {
  "public": false,
  "style": "None",
  "auth": "n/a",
  "limits": "n/a",
  "cost": "n/a",
  "export": "Before the acquisition, Bench said clients could download their data from 30 December 2024 to March 2025 (1-800Accountant gives 7 March 2025). Exports were not in a ledger format; Pilot told Bench clients they would need to move to a new accounting tool.",
  "notes": "The current site describes read-only connections to your accounts and a fractional service that works inside your own QuickBooks."
 },
 "pricing": {
  "model": "Monthly subscription, 20% off annually; hourly fractional bookkeeping.",
  "entry": "Bookkeeping Light $199 a month for businesses with $250K or less annual revenue (bench.co, October 2026).",
  "top": "Bookkeeping + Tax $649 a month; fractional bookkeeper $55 an hour plus $1,200 onboarding.",
  "notes": "Full-service tax $300 a month or $2,880 a year; catch-up bookkeeping free with an annual plan. After the shutdown, clients who left were told prepaid fees would not be refunded."
 },
 "agents": {
  "mcp": "none",
  "ai_features": "None advertised on the current site.",
  "agent_writes": "No.",
  "approval": "n/a; humans do the books.",
  "notes": "No developer or agent surface found."
 },
 "integration": {
  "scope": "Banks and cards through read-only connections; Shopify, Stripe, Square and PayPal integrations (2024). Today, also bookkeeping inside the client's QuickBooks.",
  "notes": "No outbound integrations; e-commerce sellers were among the hardest hit by the shutdown."
 },
 "preference": {
  "scale": "Bench claimed 35,000+ US small business customers; Employer.com told TechCrunch about 12,000 at acquisition (January 2025). About 450 to 650 staff in 2024, more than half rehired by early 2025.",
  "ratings": "Trustpilot 4.2/5 from 1,354 reviews, 123 in the last 12 months (October 2026).",
  "sentiment": "Current reviews praise named bookkeepers, ease of use and tax help, and some credit the Employer.com era with better processes; complaints include billing after cancellation and early inconsistency. The shutdown itself left a lasting distrust of vendor-held books in small-business forums."
 },
 "timeline": [
  {
   "date": "2012",
   "event": "Founded as 10sheet Inc.; renamed Bench in 2013"
  },
  {
   "date": "2021",
   "event": "$60M Series C ($37M equity, $23M debt), $232M post-money"
  },
  {
   "date": "2022",
   "event": "Co-founder and CEO Ian Crosby leaves; Jean-Philippe Durrios becomes CEO in August"
  },
  {
   "date": "2024-11",
   "event": "Durrios released; Inovia's Adam Schlesinger assesses the business while a sale is sought"
  },
  {
   "date": "2024-12-27",
   "event": "Platform shut without notice; about 450 staff laid off; clients pointed to Kick"
  },
  {
   "date": "2024-12-30",
   "event": "Employer.com agrees to buy Bench"
  },
  {
   "date": "2025-01-01",
   "event": "Acquisition closes; service resumes for clients who consent to transfer"
  },
  {
   "date": "2025-03",
   "event": "End of the data download window for clients who opted out"
  }
 ],
 "relation": "cautionary-tale",
 "relation_note": "The founding cautionary tale for ownpurse: a vendor held the only copy of thousands of businesses' books in a format no one else could read, and it went dark overnight. ownpurse exists so the owner's record survives any vendor, including ownpurse.",
 "borrow": "A clear per-tier price and a catch-up offer; named, reachable people, which is what reviewers still praise.",
 "avoid": "A proprietary ledger with no API and no standard export; venture debt that can switch a service off overnight; making the owner's continued access depend on consenting to a new owner.",
 "sources": [
  {
   "title": "Bench (current site and pricing)",
   "url": "https://bench.co"
  },
  {
   "title": "Bench had a crazier holiday break than your startup (BetaKit)",
   "url": "https://betakit.com/bench-had-a-crazier-holiday-break-than-your-startup/"
  },
  {
   "title": "Bench acquired by Employer.com after sudden shutdown (Accounting Today)",
   "url": "https://www.accountingtoday.com/news/bench-acquired-by-employer-com-after-sudden-shutdown"
  },
  {
   "title": "Acquisition saves Bench Accounting from abrupt closure (Accountants Daily)",
   "url": "https://www.accountantsdaily.com.au/business/20798-acquisition-saves-bench-accounting-from-abrupt-closure"
  },
  {
   "title": "Bench Accounting to be acquired by Employer.com (GeekWire)",
   "url": "https://www.geekwire.com/2024/bench-accounting-to-be-acquired-by-employer-com-following-abrupt-shutdown/"
  },
  {
   "title": "Bench shut down: what customers need to know (1-800Accountant)",
   "url": "https://qa2-a.1800accountant.com/blog/bench-accounting-shut-down-alternative"
  },
  {
   "title": "Switch from Bench to Pilot (Pilot)",
   "url": "https://pilot.com/bench-export"
  },
  {
   "title": "Bench on Trustpilot",
   "url": "https://www.trustpilot.com/review/bench.co"
  }
 ],
 "unverified": [
  "Total funding: 'over $100 million' and 'over $113 million' both appear in reporting",
  "Customer count at shutdown: 35,000+ (Bench's site) versus about 12,000 (Employer.com via TechCrunch) versus about 11,000 (a customer's estimate)",
  "Exact end of the download window: 7 March 2025 per 1-800Accountant; Accountants Daily says only March 2025",
  "Cause: The Information's report that a bank called the venture-debt loan was not confirmed by Bench or Employer.com",
  "Staff count: about 450 laid off (BetaKit) versus 600-650+ employees cited elsewhere",
  "Whether Bench's current core bookkeeping plans still run on the proprietary ledger or on QuickBooks"
 ],
 "researched": "2026-10-10"
}
---

## What it is

Bench was an online bookkeeping and tax service for US small businesses, founded in 2012 as 10sheet and based in Vancouver ([Accounting Today](https://www.accountingtoday.com/news/bench-acquired-by-employer-com-after-sudden-shutdown)). A team of bookkeepers used Bench's own software to produce monthly statements and tax returns from bank, card and commerce feeds. On 27 December 2024 the platform stopped without notice. Three days later Employer.com agreed to buy it, and it reopened under new ownership.

## How it works

Before the shutdown, you connected your accounts, Bench's bookkeepers categorized and reconciled each month, and you saw the results in Bench's app. The ledger was Bench's own: Pilot's migration page told former clients that "Bench has a proprietary ledger so you're going to need to move to a new accounting tool" ([Pilot](https://pilot.com/bench-export)). Today the [site](https://bench.co) sells Bookkeeping Light at $199 a month (under $250K revenue), Bookkeeping at $399, Bookkeeping + Tax at $649, and a fractional bookkeeper at $55 an hour who works inside your own QuickBooks. That last offer is a quiet change of direction toward the client holding the ledger.

## Data and access

Bench had no public API and no agent interface. When it closed, the notice read: "We regret to inform you that as of December 27 2024, the Bench platform will no longer be accessible" ([Accountants Daily](https://www.accountantsdaily.com.au/business/20798-acquisition-saves-bench-accounting-from-abrupt-closure)). Clients were pointed to Kick and given a download window from 30 December 2024 into March 2025; [1-800Accountant](https://qa2-a.1800accountant.com/blog/bench-accounting-shut-down-alternative) gives 7 March. After the acquisition, logging in meant choosing to transfer your data to Employer.com or opt out, and opting out suspended service without a refund of prepaid fees. Competitors, Pilot among them, built export and migration tools because the download was not something another ledger could load. In the middle of tax season, owners had statements but not books.

## Agents

None. Bench advertises no AI features, no MCP server and no API.

## Customers

Bench claimed more than 35,000 US small business owners; Employer.com told TechCrunch the figure was about 12,000 ([BetaKit](https://betakit.com/bench-had-a-crazier-holiday-break-than-your-startup/)). Shopify, Stripe, Square and PayPal sellers were among the hardest hit. Despite the shutdown, [Trustpilot](https://www.trustpilot.com/review/bench.co) shows 4.2 out of 5 from 1,354 reviews in October 2026, with recent reviews praising named bookkeepers and some crediting the new owner with better processes; complaints include charges after cancellation.

## History

Bench raised over $100M to $113M (reports differ), including a $60M Series C in 2021 of which $23M was debt, at a $232M post-money valuation, from investors including Bain Capital Ventures, Inovia, Shopify and Sage. Co-founder and CEO Ian Crosby left in early 2022; his successor left in November 2024 while investors tried to sell the company. The Information reported that a bank calling in Bench's venture debt forced the closure; an anonymous source cited scalability and cash burn. About 450 people were laid off. Employer.com, formed in November 2024 by Recruiter.com's Jesse Tinsley to consolidate brands, closed the purchase on 1 January 2025 and rehired more than half of them ([BetaKit](https://betakit.com/bench-had-a-crazier-holiday-break-than-your-startup/)).

## For ownpurse

Bench is why ownpurse exists. A vendor held the only readable copy of thousands of businesses' books, the business failed for reasons that had nothing to do with the books, and the owners found out on a Friday in December. The lesson is not that services are bad; Bench's bookkeepers are still well reviewed. It is that the record should never live only with the people doing the work. ownpurse keeps every fact a source reports in a file the owner holds, so a bookkeeper, a ledger or ownpurse itself can disappear and the books stay readable to the cent.
