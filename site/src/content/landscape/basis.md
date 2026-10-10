---
{
 "slug": "basis",
 "name": "Basis",
 "url": "https://www.getbasis.ai",
 "segment": "agent-beside-ledger",
 "status": "active",
 "founded": "2023 (emerged from stealth in 2023; one aggregator says 2022)",
 "hq": "New York, NY",
 "backing": "Venture-backed. Seed of about $3.6M to $3.8M (2023; reports differ). $34M Series A on 18 December 2024, led by Keith Rabois (Khosla Ventures) with Nat Friedman and Daniel Gross. $100M Series B on 24 February 2026 at a $1.15B valuation, led by Accel with GV, Lloyd Blankfein, Khosla and existing backers. About $138M raised in total (CPA Practice Advisor, February 2026).",
 "tagline": "A hosted platform of AI agents that does accounting, tax and audit work inside accounting firms' existing ledgers and hands it back for review.",
 "scope": "Basis sells long-running agents to accounting firms (and some corporate accounting teams) for client accounting and advisory, tax returns and audit testing. The agents read and write the client's ledger (QuickBooks and Xero named), draft and push journal entries, reconcile, and return work 'for review'. It is not a ledger, has no self-serve tier and no public API, and does not sell to business owners directly.",
 "domains": [
  "ledger",
  "close",
  "tax",
  "documents",
  "reporting"
 ],
 "customers_served": [
  "accountants",
  "enterprise",
  "mid-market"
 ],
 "spectrum": {
  "custody": {
   "score": 1,
   "note": "The books stay in the client's QuickBooks or Xero; Basis holds workpapers and agent logs in its US cloud. The owner holds neither as a file."
  },
  "locality": {
   "score": 0,
   "note": "Hosted only, data stored in the US."
  },
  "history": {
   "score": 1,
   "note": "'Every agent action is logged and auditable' with sources and rationale, but in a vendor-held log; the ledger itself is current state."
  },
  "openness": {
   "score": 0,
   "note": "Closed product, waitlist and demo only, no public API or export documentation found."
  },
  "breadth": {
   "score": 3,
   "note": "Ledgers (QuickBooks, Xero), AP (Bill.com), file stores (SharePoint, Box), tax and audit work in one platform."
  },
  "autonomy": {
   "score": 3,
   "note": "Long-horizon agents work for hours unattended and write to the ledger; some workflows stage entries for review, others are not described."
  },
  "agent_readiness": {
   "score": 1,
   "note": "Basis is the agent; there is no MCP server, CLI or API for your own agents."
  },
  "integration_ease": {
   "score": 1,
   "note": "Sales-led and waitlisted; a third party or owner cannot connect to it."
  },
  "market": {
   "score": 3,
   "note": "Sold to large firms (about 30% of the top 25 US firms, Baker Tilly, CohnReznick) with a separate small-firm offering."
  }
 },
 "api": {
  "public": false,
  "style": "None public. Connects to ledgers through their OAuth APIs (Xero OAuth 2.0 with revocable tokens).",
  "auth": "n/a for customers; Basis uses OAuth tokens to client ledgers and says it stores no credentials.",
  "limits": "Not published.",
  "cost": "Not published; no self-serve pricing.",
  "export": "Not documented. Customers 'can delete their data at any time' (security page); no export format found.",
  "notes": "Xero App Store listing describes the agents as working 'bidirectionally with your Xero ledger', reconciling bank transactions, creating and updating invoices and bills, and generating journal entries."
 },
 "pricing": {
  "model": "Enterprise contract, sales-led",
  "entry": "Not published (waitlist / get access)",
  "top": "Not published",
  "notes": "No public pricing found on getbasis.ai as of 2026-10-10."
 },
 "agents": {
  "mcp": "none",
  "ai_features": "Long-horizon agents for close, reconciliations, journal entries, financial statements, tax trial-balance mapping, K-1 extraction, return preparation and QC (End-to-End Tax Platform, 5 August 2026), audit testing; 350+ prebuilt 'Procedures' firms can customise.",
  "agent_writes": "Yes: pushes journal entries and creates or updates invoices and bills in QuickBooks and Xero.",
  "approval": "Firm review of deliverables. The small-firm page says JE drafting and write-up 'stage every entry for your review'; other workflows (reconciliation fixes, Xero invoice and bill updates) do not say whether they are staged.",
  "notes": "Customer data not used for training; SOC 2 Type II, ISO 27001 and ISO 42001."
 },
 "integration": {
  "scope": "QuickBooks and Xero (ledgers), Bill.com (AP), SharePoint and Box (files), per Accounting Today, December 2024. The 2026 site names no ERPs, only an optional 'Current ERP' field.",
  "notes": "Xero App Store listing has 0 reviews as of 2026-10-10."
 },
 "preference": {
  "scale": "About 30% of the top 25 US accounting firms (company claim, February 2026). Named firms include Baker Tilly (partnership 28 April 2026), CohnReznick, Wiss, Citrin Cooperman, Sikich, UHY, Clark Nuber.",
  "ratings": "No G2, Capterra or Trustpilot profile found. Xero App Store: not yet rated, 0 reviews (2026-10-10).",
  "sentiment": "Only vendor-published testimonials: Boulay says a 100+ hour partnership return dropped to 15-20 hours; Clark Nuber credits it with 50% CAS growth; Khosla cites 20-50% efficiency gains. No independent user reviews found. Pilot's Meridian page claims tools like Basis 'require babysitting micro-tasks' (a competitor's claim)."
 },
 "timeline": [
  {
   "date": "2023",
   "event": "Founded by Matthew Harpe and Mitchell Troyanovsky; emerges from stealth with about $3.6M-$3.8M"
  },
  {
   "date": "2024",
   "event": "Wiss, a Top 100 firm, partners with Basis"
  },
  {
   "date": "2024-12-18",
   "event": "$34M Series A led by Keith Rabois (Khosla) with Nat Friedman and Daniel Gross"
  },
  {
   "date": "2026-02-24",
   "event": "$100M Series B led by Accel at a $1.15B valuation"
  },
  {
   "date": "2026-04-28",
   "event": "Baker Tilly collaboration for managed accounting services"
  },
  {
   "date": "2026-08-05",
   "event": "End-to-End Tax Platform announced"
  }
 ],
 "relation": "neighbour",
 "relation_note": "The nearest pattern to ownpurse's write model: agents beside an existing ledger, with a person reviewing. It differs on every other axis: closed, hosted, sold only to firms, and the record of what the agents did lives with Basis, not the owner.",
 "borrow": "Logging every agent step with its sources, changes and rationale; staging entries for review before they post; a library of named, shareable procedures per firm; sourcing every number back to its document.",
 "avoid": "A log the owner cannot hold or export; firm-only, waitlisted access; workflows where it is unclear whether a write was staged or posted.",
 "sources": [
  {
   "title": "Basis home page",
   "url": "https://www.getbasis.ai"
  },
  {
   "title": "Basis security",
   "url": "https://www.getbasis.ai/security"
  },
  {
   "title": "Basis for Small Firms",
   "url": "https://www.getbasis.ai/solutions/small-firms"
  },
  {
   "title": "The future of tax is end-to-end intelligence (5 Aug 2026)",
   "url": "https://www.getbasis.ai/blogs/the-future-of-tax-is-end-to-end-intelligence"
  },
  {
   "title": "Basis on the Xero App Store",
   "url": "https://apps.xero.com/us/app/basis-app"
  },
  {
   "title": "CPA Practice Advisor: Basis raises $100M (24 Feb 2026)",
   "url": "https://www.cpapracticeadvisor.com/2026/02/24/basis-raises-100-million-to-deploy-ai-agents-for-accounting-firms/178759/"
  },
  {
   "title": "SiliconANGLE: Basis secures $100M at $1.15B",
   "url": "https://siliconangle.com/2026/02/24/ai-accounting-startup-basis-secures-100m-1-15b-valuation-firms-adopt-agent-based-workflows/"
  },
  {
   "title": "Accounting Today: Major AI players back Basis with $34M (18 Dec 2024)",
   "url": "https://accountingtoday.com/news/major-ai-players-back-basis-with-34-million-series-a"
  },
  {
   "title": "Baker Tilly collaborates with Basis (28 Apr 2026)",
   "url": "https://www.bakertilly.com/news/baker-tilly-collaborates-with-basis"
  },
  {
   "title": "Pulse 2.0: Basis raises $3.6M",
   "url": "https://pulse2.com/basis-accounting-firm-ai-platform-company-raises-3-6-million/"
  }
 ],
 "unverified": [
  "Seed amount: $3.6M (Pulse 2.0) vs $3.8M (Accounting Today)",
  "Founding year 2023 vs 2022 (aggregators differ)",
  "Whether the 2026 platform still integrates QuickBooks, Xero, Bill.com, SharePoint and Box exactly as listed in 2024; the current site names no ERPs",
  "Whether reconciliation fixes and Xero invoice/bill updates are staged for review or posted directly",
  "Pricing of any kind",
  "G2/Capterra presence (searches found none; review sites could not be fetched)",
  "The 30%-of-top-25 and efficiency figures are company and investor claims"
 ],
 "researched": "2026-10-10"
}
---

## What it is

Basis is a New York company that builds AI agents for accounting firms. The agents do client accounting, tax and audit work inside the firm's existing systems and hand back finished work for a person to review. It sells only to accountants, from large firms such as Baker Tilly and CohnReznick to a newer [small-firm offering](https://www.getbasis.ai/solutions/small-firms). It is not a ledger and does not ask anyone to move their books.

## How it works

A firm onboards Basis to an engagement and assigns it workflows, as it would a junior staff accountant ([Accounting Today, December 2024](https://accountingtoday.com/news/major-ai-players-back-basis-with-34-million-series-a)). The agents are 'long-horizon': they run for hours in the background and return completed deliverables for review ([CPA Practice Advisor, February 2026](https://www.cpapracticeadvisor.com/2026/02/24/basis-raises-100-million-to-deploy-ai-agents-for-accounting-firms/178759/)). Work covers GL cleanup, journal entries, reconciliations and statements; tax trial-balance mapping, K-1 extraction, workpapers and return QC; and audit testing. Firms get 350+ prebuilt Procedures they can adapt. On 5 August 2026 Basis announced an [End-to-End Tax Platform](https://www.getbasis.ai/blogs/the-future-of-tax-is-end-to-end-intelligence) that runs from client intake to a review-ready return.

The small-firm page says JE drafting and write-up 'stage every entry for your review' before Basis pushes them to the GL. For other workflows, such as fixing reconciliation exceptions, it does not say whether entries are staged or posted.

## Data and access

Basis reaches ledgers through their own OAuth APIs. Its [Xero App Store listing](https://apps.xero.com/us/app/basis-app) says the agents work 'bidirectionally with your Xero ledger', reconciling bank transactions, creating and updating invoices and bills, and generating journal entries. Press in 2024 named QuickBooks, Xero, Bill.com, SharePoint and Box. The [security page](https://www.getbasis.ai/security) lists SOC 2 Type II, ISO 27001 and ISO 42001, US-only storage, no training on customer data, per-tenant isolation and 'every agent action is logged and auditable'. There is no public API, no published export format and no public pricing.

## Agents

Basis is the agent. It offers no MCP server, CLI or API for your own agents, and your agents cannot read what its agents did. Its review model is the one ownpurse's approval gate formalises: the machine prepares, a person reviews, the work moves on. Here the review sits inside Basis's product and the ledger write goes straight to the provider.

## Customers

Basis says about 30% of the top 25 US firms use it (February 2026). Its site names over twenty firms. Customer quotes are strong: Boulay reports a partnership return with 60-70 K-1s dropping from over 100 prep hours to 15-20, and Clark Nuber credits Basis with 50% annual CAS growth. These are vendor-published. No G2, Capterra or Reddit reviews turned up, and the Xero listing has none.

## History

Matthew Harpe (CEO) and Mitchell Troyanovsky, formerly at Puzzle, founded it and came out of stealth in 2023. It raised a $34M Series A in December 2024 and a $100M Series B at $1.15B in February 2026, led by [Accel](https://siliconangle.com/2026/02/24/ai-accounting-startup-basis-secures-100m-1-15b-valuation-firms-adopt-agent-based-workflows/). In April 2026 it announced a [Baker Tilly collaboration](https://www.bakertilly.com/news/baker-tilly-collaborates-with-basis).

## For ownpurse

Basis shows the market will pay well for agents that work beside a ledger and stop for review. That is ownpurse's pattern, sold to firms, hosted and closed. What it lacks is what ownpurse is built on: the owner holds no copy of the record, cannot verify the history, and cannot bring their own agent. Worth borrowing: per-step logs with sources and rationale, staged entries, named procedures. The gap it leaves is the business owner and the small firm that want the same leverage over a record they hold.
