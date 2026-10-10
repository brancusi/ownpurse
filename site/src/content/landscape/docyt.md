---
{
 "slug": "docyt",
 "name": "Docyt",
 "url": "https://docyt.com",
 "segment": "agent-beside-ledger",
 "status": "pivoted",
 "founded": "Not confirmed by a primary source (commonly cited as 2016)",
 "hq": "Mountain View, California, USA (2022 press release dateline: Santa Clara)",
 "backing": "Venture-backed: $1.5M seed extension led by First Rays Venture Partners (February 2021); $11.5M Series A led by Lobby Capital with First Rays and Morado Ventures (April 2022); $12M pre-Series B led by Pivot Investment Partners (August 2025). Total raised is not stated by the company.",
 "tagline": "An AI accounting platform for multi-location businesses, hotels first, that began as an automation layer over QuickBooks and now sells itself as an AI-native ERP with its own ledger.",
 "scope": "Docyt automates bookkeeping, revenue posting from hotel PMS and POS systems, bank and card reconciliation, the month-end close, bill pay and corporate cards, and BI (daily flash, budgets, forecasts) for multi-entity operators, with in-house accountants resolving exceptions. It now keeps its own multi-entity general ledger and syncs to QuickBooks Online or another GL only if you want that for statutory reporting. It does not do tax filing beyond 1099s, and its assistant connector is read-only.",
 "domains": [
  "ledger",
  "bank",
  "payments",
  "documents",
  "close",
  "reporting",
  "spend"
 ],
 "customers_served": [
  "smb",
  "mid-market",
  "enterprise",
  "accountants"
 ],
 "spectrum": {
  "custody": {
   "score": 1,
   "note": "Docyt's own GL is now the working record; scheduled exports in open formats and an optional QBO sync let you keep a copy, but you do not hold the authoritative one."
  },
  "locality": {
   "score": 0,
   "note": "Hosted service only."
  },
  "history": {
   "score": 2,
   "note": "States an immutable audit log in which 'nothing is deleted; corrections are new entries', with the evidence, decision and confidence for each entry; no as-of query interface is published."
  },
  "openness": {
   "score": 1,
   "note": "Closed; API access for enterprise customers only, plus scheduled open-format exports and a documented data-return process."
  },
  "breadth": {
   "score": 3,
   "note": "Ledger, bank and card reconciliation, PMS and POS revenue, payroll accounting, bill pay, cards, receivables and BI in one product."
  },
  "autonomy": {
   "score": 3,
   "note": "Agents auto-post what clears per-workflow confidence thresholds (its example: 1,839 auto-posted, 2 sent to review); in-house accountants resolve the rest."
  },
  "agent_readiness": {
   "score": 2,
   "note": "A read-only connector for ChatGPT, Claude and Muse (managed under 'MCP Access' in settings) plus an enterprise API; no public developer docs or write tools."
  },
  "integration_ease": {
   "score": 1,
   "note": "Docyt connects to 30+ PMS and POS systems for you, but a third party can reach Docyt only through an enterprise API or the read-only assistant connector."
  },
  "market": {
   "score": 3,
   "note": "Multi-location hotel groups, management companies, restaurants, retail and franchises, with portfolio and enterprise plans."
  }
 },
 "api": {
  "public": false,
  "style": "Not documented publicly; 'API access for enterprise customers'",
  "auth": "The assistant connector uses sign-in with a Docyt email and password; the credential mechanism and scopes are not published",
  "limits": "Not published",
  "cost": "Included in the Enterprise Suite (custom pricing); otherwise not offered",
  "export": "Scheduled exports in open formats, a documented data-return process, and 'no exit penalties, data can be exported at any time' on hotel plans",
  "notes": "An optional sync to QuickBooks Online or another GL for statutory reporting."
 },
 "pricing": {
  "model": "Per property (or location) per month, annual contract billed monthly, unlimited users",
  "entry": "Accounting Suite from $299 per property per month (hotels); Daily Flash standalone from $100 per location per month; catch-up accounting at $200 per month caught up (checked 2026-10-10)",
  "top": "Enterprise Suite: custom per-property pricing that falls with portfolio size, outcome-based service, named account team",
  "notes": "The Payments and Business Intelligence suites are add-ons with no listed price. Check and ACH payments carry transaction fees. All integrations and a dedicated hospitality accountant are included."
 },
 "agents": {
  "mcp": "official",
  "ai_features": "The HpAI engine with '16 AI agents' covering money-out, money-in, reconciliation, close and reporting (named ones include a Closing Agent, a Quality of Books Agent and a Migration Agent), BooksGPT and 'Ask Docyt AI', AI dashboards, invoice extraction and auto-categorization. It claims training on '128 billion accounting data points across more than 20 industries' (August 2025).",
  "agent_writes": "Docyt's own agents write to its ledger within policy. Its generative and predictive models are 'never with write access to the ledger on their own'. External assistants connected to Docyt are read-only: they cannot categorize, verify, edit, upload or run close.",
  "approval": "Configurable: approval workflows by amount, vendor, department and entity; confidence thresholds per workflow; materiality rules for close escalation. Below-threshold items go to a human reviewer, often Docyt's own accountants. Docyt does not state that every write needs approval.",
  "notes": "The assistant connector page does not name the protocol, but settings call the section 'MCP Access', so the mcp value is inferred. The connector works with ChatGPT (Plus and above), Claude (Pro and above) and Muse."
 },
 "integration": {
  "scope": "30+ PMS and POS integrations (Oracle OPERA Cloud, Hilton OnQ, Marriott FSPMS, Sabre SynXis, Mews, Cloudbeds and others), payroll, banks and ledgers. The site's totals are inconsistent: '300+ systems' and 'more than 500 tools'. Migration sources include M3, Sage Intacct, NetSuite and QuickBooks.",
  "notes": "All integrations are included in every plan. The site's integration counts conflict, so treat them as marketing figures."
 },
 "preference": {
  "scale": "No customer total. The site cites 250,000+ vendors processed, $1.3 billion+ in invoice volume and Shamin Hotels with 80+ hotels (2026). Customer base grew 4x in 2021 (April 2022 release).",
  "ratings": "Not retrieved: G2 and Capterra blocked direct fetches and the search budget ran out.",
  "sentiment": "Customer quotes in press coverage praise how little intervention it needs. Independent review sentiment could not be gathered for this dossier."
 },
 "timeline": [
  {
   "date": "2021-02-25",
   "event": "$1.5M seed extension led by First Rays Venture Partners, with Morado Ventures and angels."
  },
  {
   "date": "2022-04-06",
   "event": "$11.5M Series A led by Lobby Capital; described as software that 'turbocharges accounting ledgers, like QuickBooks'."
  },
  {
   "date": "2025-08-06",
   "event": "$12M pre-Series B led by Pivot Investment Partners; launches the HpAI engine and precision-trained agents inside Accountant Copilot."
  },
  {
   "date": "2026",
   "event": "Repositioned as an 'AI-native ERP' with its own multi-entity GL, hotel-first pricing from $299 per property per month, 16 agents and a read-only connector for ChatGPT and Claude."
  }
 ],
 "relation": "neighbour",
 "relation_note": "Docyt started beside QuickBooks and has moved toward being the ledger itself, for multi-location operators. It shows where the agent-beside-the-ledger pattern drifts once a vendor owns the workflow: toward holding the books. ownpurse goes the other way and leaves the books where they are.",
 "borrow": "Its audit model ('nothing is deleted; corrections are new entries', each entry tied to source document, decision, confidence and review); per-workflow confidence thresholds tuned from verification outcomes; a read-only assistant connector with clearly listed limits.",
 "avoid": "Moving the authoritative ledger into the vendor's cloud; integration counts that contradict each other; API access only for enterprise customers.",
 "sources": [
  {
   "title": "Docyt homepage",
   "url": "https://docyt.com"
  },
  {
   "title": "Docyt pricing (hotels)",
   "url": "https://docyt.com/pricing"
  },
  {
   "title": "Docyt AI-native ERP",
   "url": "https://docyt.com/products/ai-native-erp/"
  },
  {
   "title": "Docyt HpAI",
   "url": "https://docyt.com/technology/hpai/"
  },
  {
   "title": "Connect your AI assistant to Docyt",
   "url": "https://docyt.com/help/connect-ai-assistant/"
  },
  {
   "title": "Docyt raises $11.5 million (April 2022)",
   "url": "https://www.docyt.com/article/docyt-ai-powered-accounting-automation-raises-11-million/"
  },
  {
   "title": "Docyt raises $1.5 million seed extension (Business Wire, February 2021)",
   "url": "https://www.businesswire.com/news/home/20210225005364/en/Accounting-Workflow-Automation-Platform-Docyt-Raises-1.5-Million"
  },
  {
   "title": "Docyt debuts precision-trained AI agents (SiliconANGLE, August 2025)",
   "url": "https://siliconangle.com/2025/08/06/ai-bookkeeper-docyt-debuts-precision-trained-ai-agents-accelerate-accounting-automation/"
  },
  {
   "title": "Docyt secures $12m (FinTech Global, August 2025)",
   "url": "https://fintech.global/2025/08/07/docyt-secures-12m-to-expand-ai-driven-accounting-tools/"
  }
 ],
 "unverified": [
  "Founding year (commonly cited as 2016; no primary source found)",
  "Total funding raised (not stated by the company; roughly $25M implied by the three rounds found, earlier rounds unknown)",
  "Whether the assistant connector is technically an MCP server (the settings label says 'MCP Access'; the help page does not name the protocol)",
  "When the repositioning to an AI-native ERP with its own GL happened",
  "G2, Capterra and other review ratings (fetches blocked; search budget exhausted)",
  "Integration counts conflict on the site (30+, 300+, 500+)",
  "'GARY' as a product name for the month-end close agent appears in a search snippet only"
 ],
 "researched": "2026-10-10"
}
---

## What it is

Docyt is an AI accounting platform for multi-location businesses, with hotels first, then restaurants, retail and franchises. In 2022 it described itself as software that ["turbocharges accounting ledgers, like QuickBooks"](https://www.docyt.com/article/docyt-ai-powered-accounting-automation-raises-11-million/). In 2026 it sells an ["AI-native ERP"](https://docyt.com/products/ai-native-erp/) with its own multi-entity general ledger. You can still sync to QuickBooks Online or another GL for statutory reporting. That shift is why this dossier marks it as pivoted.

## How it works

The [HpAI engine](https://docyt.com/technology/hpai/) runs "16 AI agents" across money-out, money-in, reconciliation, close and reporting. Every decision carries a confidence score. Below the threshold, the agent escalates instead of guessing, and Docyt's in-house accountants resolve the exception. Thresholds are set per workflow and tuned from verification outcomes. Docyt's own example shows 1,839 transactions auto-posted and two sent to review. Predictive and generative models are "never with write access to the ledger on their own". Every entry links to its source document, the agent's decision and confidence, any human review, and its effect on the ledger. Docyt says "nothing is deleted; corrections are new entries". For hotels it posts PMS and POS revenue nightly with USALI mapping, reconciles merchant deposits and OTA payouts, and aims at a "zero-day close".

Its strengths are depth in one vertical, 30+ PMS and POS integrations, and an audit model closer to append-only than most in this segment.

## Data and access

Docyt offers no public API. API access is [for enterprise customers](https://docyt.com/products/ai-native-erp/). Scheduled exports come in open formats, and hotel plans promise "no exit penalties". [Pricing](https://docyt.com/pricing) is per property per month on annual contracts with unlimited users: the Accounting Suite starts at $299, Daily Flash at $100 per location, catch-up accounting is $200 per month caught up, and the Enterprise Suite is custom (checked 10 October 2026).

## Agents

You can [connect ChatGPT, Claude or Muse](https://docyt.com/help/connect-ai-assistant/) from the assistant's connector directory by signing in to Docyt. Access is managed under "MCP Access" in Docyt settings. The connection is read-only: reports, KPIs, portfolio summaries, transaction searches and statement coverage. An assistant cannot categorize, edit, upload or run close. That makes the read side agent-friendly. All writes stay with Docyt's own agents and staff.

## Customers

Docyt gives no customer total. It cites 250,000+ vendors processed, $1.3 billion+ in invoice volume and Shamin Hotels with 80+ hotels. Its customer base grew 4x in 2021. Its integration counts contradict each other (30+, 300+ and 500+). This dossier has no independent review ratings.

## History

Sidharth Saxena is co-founder and CEO. Docyt raised a [$1.5M seed extension](https://www.businesswire.com/news/home/20210225005364/en/Accounting-Workflow-Automation-Platform-Docyt-Raises-1.5-Million) in February 2021, an $11.5M Series A led by Lobby Capital in April 2022, and a [$12M pre-Series B](https://siliconangle.com/2025/08/06/ai-bookkeeper-docyt-debuts-precision-trained-ai-agents-accelerate-accounting-automation/) led by Pivot Investment Partners in August 2025, alongside HpAI and Accountant Copilot.

## For ownpurse

Docyt shows that the audit model ownpurse wants (corrections as new entries, every write tied to evidence and confidence) sells to operators who care about it. It also shows the drift: a tool beside the ledger becomes the ledger once it owns the workflow. ownpurse's answer is that the owner keeps the log, and any vendor's agent, Docyt's included, reads from it and proposes into it.
