---
{
 "slug": "botkeeper",
 "name": "Botkeeper",
 "url": "https://www.botkeeper.com",
 "segment": "bookkeeping-service",
 "status": "acquired",
 "founded": "2015",
 "hq": "Founded in Boston, Massachusetts; the Infinite platform is now run as a division of Xendoo, Fort Lauderdale, Florida",
 "backing": "Raised nearly $90M, including a $42M Series C in November 2021 led by Grand Oaks Capital (Tom Golisano); other listed investors Greycroft, Ignition, Sorenson Capital, Gradient Ventures, High Alpha. Wound down February 2026; Infinite platform bought by Xendoo (announced 27 February to 3 March 2026), terms undisclosed.",
 "tagline": "AI bookkeeping automation sold to accounting firms per client entity; the company wound down in February 2026 and Xendoo bought and kept running its Infinite platform.",
 "scope": "Botkeeper began as automated bookkeeping for small businesses and moved to selling software, with optional offshore staff, to accounting firms. Infinite categorizes transactions, reconciles banks, automates journal entries, flags exceptions and tracks the close, posting to the firm's ledger when confidence is high. It holds no ledger of its own and has no public API.",
 "domains": [
  "bank",
  "close",
  "documents",
  "reporting",
  "integration"
 ],
 "customers_served": [
  "accountants",
  "smb"
 ],
 "spectrum": {
  "custody": {
   "score": 2,
   "note": "The general ledger usually sits in a firm-owned QuickBooks Online or Xero subscription, so the wind-down did not take the books; Botkeeper's working data stayed with Botkeeper."
  },
  "locality": {
   "score": 0,
   "note": "Hosted only."
  },
  "history": {
   "score": 1,
   "note": "Advertises audit-friendly transparency and a model-training timestamp; no client-held or as-of history."
  },
  "openness": {
   "score": 1,
   "note": "CSV exports from Transaction Manager and Close Tracker; no public API or developer docs."
  },
  "breadth": {
   "score": 2,
   "note": "Bank linking, categorization, bank reconciliation, journal entries, documents, password manager and practice management; no payments, payroll or tax."
  },
  "autonomy": {
   "score": 3,
   "note": "Posts directly to the general ledger when confidence is high (97% accuracy claimed on those); lower-confidence items go to review."
  },
  "agent_readiness": {
   "score": 0,
   "note": "No API, no MCP server, no agent docs."
  },
  "integration_ease": {
   "score": 1,
   "note": "Self-serve signup and bank linking through Smart Connect, but nothing documented for a third party to connect."
  },
  "market": {
   "score": 2,
   "note": "Accounting firms serving small and mid-sized businesses, priced per client entity."
  }
 },
 "api": {
  "public": false,
  "style": "None found",
  "auth": "n/a",
  "limits": "n/a",
  "cost": "n/a",
  "export": "CSV from Transaction Manager and Close Tracker, Business Insights downloads (knowledge base, as reported by Uplinq); the ledger itself is in the firm's QuickBooks or Xero.",
  "notes": "No developer portal, OpenAPI spec or webhooks found on botkeeper.com in October 2026."
 },
 "pricing": {
  "model": "Per client-entity license, monthly or annual (10% off), volume tiers.",
  "entry": "$149 per license a month billed monthly for 1 to 4 licenses ($134 annually), October 2026.",
  "top": "$59 per license a month for 25+ licenses ($53 annually).",
  "notes": "One license is one client entity. No free trial. An add-on for dedicated offshore bookkeeping staff had a 10-license minimum."
 },
 "agents": {
  "mcp": "none",
  "ai_features": "AI categorization (Transaction Manager), Auto Bank Rec, JE Automation, Bot Review anomaly detection, Transaction Insights on model performance; Financial Insights with Google listed as coming soon.",
  "agent_writes": "Its own models write to the firm's ledger (AutoPush) above a confidence threshold; no outside agent access.",
  "approval": "A confidence threshold, not an approval gate: high-confidence entries post directly, the rest are surfaced for a person to review.",
  "notes": "In July 2026 its founder argued for 'AI sovereignty': firms should keep control of their data and not train the platforms that will compete with them."
 },
 "integration": {
  "scope": "Bank and card linking through Smart Connect (being replaced by an Accounts page, September 2026); the firm's QuickBooks Online or Xero as the ledger.",
  "notes": "botkeeper.com does not name supported ledgers in October 2026; QuickBooks Online and Xero come from third-party reporting."
 },
 "preference": {
  "scale": "'Hundreds of firms' and 'thousands of clients' (Botkeeper, October 2026). Inc. 5000 in 2020, 2021 and 2022.",
  "ratings": "Not confirmed; G2 and Capterra refused direct fetch.",
  "sentiment": "Firms valued the cleanup speed and per-entity price; the end came from revenue concentrated in a few large CPA firms that were consolidated in late 2025. The founder said the company never found product-market fit strong enough to survive that."
 },
 "timeline": [
  {
   "date": "2015",
   "event": "Founded by Enrico Palmerino as automated bookkeeping for small businesses"
  },
  {
   "date": "2020-2022",
   "event": "Inc. 5000 three years running; shift to selling to accounting firms"
  },
  {
   "date": "2021-11",
   "event": "$42M Series C led by Grand Oaks Capital, part of nearly $90M raised"
  },
  {
   "date": "2024-04",
   "event": "Infinite platform rolled out to firms"
  },
  {
   "date": "2025-Q4",
   "event": "Large CPA firm clients consolidated; concentrated revenue lost"
  },
  {
   "date": "2026-02-07",
   "event": "Founder posts closure statement; orderly wind-down begins"
  },
  {
   "date": "2026-02-27",
   "event": "Xendoo announces it has bought Infinite; platform stays operational"
  },
  {
   "date": "2026-06-12",
   "event": "First release notes under the new owner"
  },
  {
   "date": "2026-09-11",
   "event": "Accounts page in public beta, replacing Smart Connect"
  }
 ],
 "relation": "cautionary-tale",
 "relation_note": "The second cautionary tale, and a milder one: Botkeeper failed, but because the books lived in the firms' own QuickBooks or Xero, clients kept their ledgers and a buyer kept the software running. It shows both why the owner should hold the record and why a confidence threshold is not the same as approval.",
 "borrow": "Leaving the ledger in the firm's own QuickBooks or Xero; publishing model performance to the user; per-entity pricing that scales down for firms.",
 "avoid": "Revenue concentrated in a few large firms; auto-posting above a confidence threshold with no person approving the write; no API or export of the vendor's own working data.",
 "sources": [
  {
   "title": "Xendoo buys Botkeeper Infinite (CPA Practice Advisor, 28 Feb 2026)",
   "url": "https://www.cpapracticeadvisor.com/2026/02/28/xendoo-buys-botkeeper-infinite-ai-platform-will-remain-active-and-supported/179027/"
  },
  {
   "title": "What happened to Botkeeper (Uplinq)",
   "url": "https://www.uplinq.com/compare/what-happened-to-botkeeper"
  },
  {
   "title": "Botkeeper home page",
   "url": "https://www.botkeeper.com"
  },
  {
   "title": "Botkeeper pricing",
   "url": "https://www.botkeeper.com/pricing"
  },
  {
   "title": "Botkeeper Infinite overview",
   "url": "https://www.botkeeper.com/botkeeper-infinite-overview"
  },
  {
   "title": "Botkeeper about",
   "url": "https://www.botkeeper.com/about"
  },
  {
   "title": "Botkeeper Infinite new releases (11 Sep 2026)",
   "url": "https://www.botkeeper.com/blog/botkeeper-infinite-new-releases-new-pages-smarter-recs-faster-flows"
  },
  {
   "title": "AI sovereignty (Botkeeper blog, 14 Jul 2026)",
   "url": "https://www.botkeeper.com/blog/ai-sovereignty-accounting-firms-must-stop-training-future-competitors"
  },
  {
   "title": "Xendoo",
   "url": "https://www.xendoo.com"
  },
  {
   "title": "Xendoo buys Botkeeper's Infinite platform (Yahoo Finance)",
   "url": "https://finance.yahoo.com/news/xendoo-buys-botkeeper-infinite-platform-123402767.html"
  }
 ],
 "unverified": [
  "Supported ledgers (QuickBooks Online and Xero) come from Uplinq, a competitor; botkeeper.com does not name them",
  "Acquisition date: 27 February 2026 (trade press) versus 3 March 2026 (Xendoo's site)",
  "Earlier funding rounds before the 2021 Series C, and their amounts",
  "Headquarters history: Boston at founding; any later move (the 2024 Inc. Southeast regional listing suggests a Southeast base)",
  "G2, Capterra and Trustpilot ratings (pages refused fetch; web search budget was exhausted)",
  "The 97% and 98% accuracy figures are the company's own",
  "Whether client data held only in Botkeeper (categorization history, documents) was retained or deleted for users who left during the wind-down"
 ],
 "researched": "2026-10-10"
}
---

## What it is

Botkeeper sold AI bookkeeping automation to accounting firms, priced per client entity. Founded in 2015 by Enrico Palmerino, it started by doing small businesses' books with software and staff, then moved to selling the software to firms. Its Infinite platform, rolled out in April 2024, categorizes transactions, reconciles banks, automates journal entries and tracks the close. On 7 February 2026 the founder announced an orderly wind-down; about three weeks later Xendoo, a Fort Lauderdale bookkeeping firm, said it had bought Infinite and would keep it running ([CPA Practice Advisor](https://www.cpapracticeadvisor.com/2026/02/28/xendoo-buys-botkeeper-infinite-ai-platform-will-remain-active-and-supported/179027/)).

## How it works

A firm links each client's banks and cards, and Infinite's models categorize transactions against the client's chart of accounts. The [site](https://www.botkeeper.com) says the AI "posts directly to the GL only when confidence is high—delivering 97% accuracy on those entries. Anything lower gets surfaced for your review." Bot Review scans the ledger for exceptions, Auto Bank Rec reconciles when nothing looks wrong, and Close Tracker shows where each client stands. Licenses cost $149 a month each for one to four clients, falling to $59 for 25 or more, or 10% less billed annually ([pricing](https://www.botkeeper.com/pricing)). Release notes on [11 September 2026](https://www.botkeeper.com/blog/botkeeper-infinite-new-releases-new-pages-smarter-recs-faster-flows) show the product still shipping.

## Data and access

Botkeeper has no public API and publishes no developer docs. Its knowledge base describes CSV exports from Transaction Manager and Close Tracker. The important fact is where the books live: according to [Uplinq](https://www.uplinq.com/compare/what-happened-to-botkeeper), a competitor, the general ledger usually sits in a firm-owned QuickBooks Online or Xero subscription. That is why Botkeeper's failure was gentler than Bench's: the firms' ledgers were never inside it. botkeeper.com itself does not name the ledgers it supports.

## Agents

There is no MCP server and no outside agent access. Botkeeper's own models are the agent, and they write to the ledger above a confidence threshold. That is automation with a review queue for the doubtful cases, not an approval gate: a confident mistake posts. In July 2026 the founder published an argument for ["AI sovereignty"](https://www.botkeeper.com/blog/ai-sovereignty-accounting-firms-must-stop-training-future-competitors), urging firms to keep control of their data and not train the large platforms that may compete with them, an argument close to ownpurse's own.

## Customers

Botkeeper says it serves hundreds of firms and thousands of clients, and it made the Inc. 5000 in 2020, 2021 and 2022 ([about](https://www.botkeeper.com/about)). Review-site ratings could not be confirmed. Its weakness was concentration: the closure statement blamed revenue lost when large CPA firm clients, signed in late 2025, were caught up in industry consolidation.

## History

Botkeeper raised nearly $90M, including a $42M Series C in November 2021 led by Grand Oaks Capital, with Greycroft, Ignition, Sorenson Capital, Gradient Ventures and High Alpha also listed as investors. The founder said the market "moved faster than our capital could keep up" and that the company did not reach product-market fit strong enough to survive; it tried acquisitions, lender talks and bridge capital first. Xendoo announced the purchase of Infinite around 27 February 2026 (its own site says 3 March), kept access live for existing users, and runs it as a separate division.

## For ownpurse

Botkeeper is the cautionary tale with a better ending, and the difference is custody. Because the ledgers stayed in the firms' own QuickBooks and Xero, a failed vendor did not take anyone's books. But the vendor's working data, its categorization history and its record of what it posted were Botkeeper's, and they went wherever Botkeeper went. ownpurse takes the next step: the owner holds every fact each source reported, and a confident model still cannot write without a person approving the package.
