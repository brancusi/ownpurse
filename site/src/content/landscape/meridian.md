---
{
 "slug": "meridian",
 "name": "Meridian (Pilot)",
 "url": "https://meridian.pilot.com",
 "segment": "agent-beside-ledger",
 "status": "active",
 "founded": "Pilot in 2017 (production since 2017); Meridian offered to other firms in 2026",
 "hq": "San Francisco, CA (Pilot also lists a Nashville office)",
 "backing": "A product of Pilot.com, Inc., venture-backed: over $150M across Series A ($15M, March 2018), Series B ($40M) and Series C (March 2021, reported $1.2B valuation). Investors include Sequoia, Index Ventures, Stripe, Bezos Expeditions, Whale Rock, Patrick and John Collison, Drew Houston and Diane Greene.",
 "tagline": "Pilot's AI platform for accounting firms that runs the month-end close inside the client's own QuickBooks Online or Xero and hands back books for the firm to review.",
 "scope": "Meridian runs the close across a firm's client portfolio: categorization, reconciliation, schedules, flux checks and exceptions. It writes changes into the client's ledger and holds delivery until the firm signs off. It adds Meridian AI (chat for ad hoc work), a client portal and a close dashboard. Pilot also gives away hosted MCP servers for QuickBooks and Xero with direct read-write access. It is not a ledger, has no public API for the platform, and supports only QBO and Xero today (NetSuite, Sage Intacct and others in development).",
 "domains": [
  "ledger",
  "close",
  "bank",
  "reporting",
  "integration"
 ],
 "customers_served": [
  "accountants",
  "smb"
 ],
 "spectrum": {
  "custody": {
   "score": 1,
   "note": "'Your books stay in your own general ledger the whole time'; Meridian-held logs, files and derived data fall under separate contract terms."
  },
  "locality": {
   "score": 0,
   "note": "Hosted only, including the MCP servers."
  },
  "history": {
   "score": 1,
   "note": "'Every decision is logged for review', in Pilot's systems; no owner-held or as-of history."
  },
  "openness": {
   "score": 1,
   "note": "Closed platform with no public API, but free hosted MCP servers for QBO and Xero anyone can use."
  },
  "breadth": {
   "score": 3,
   "note": "Ledgers, bank and card feeds, uploaded statements, payroll, Stripe, client portal and firm management."
  },
  "autonomy": {
   "score": 3,
   "note": "Checklist work runs autonomously and changes the ledger before review; delivery to the client is gated, and sensitive entries can be reserved for manual approval."
  },
  "agent_readiness": {
   "score": 3,
   "note": "Free hosted MCP servers for QuickBooks (Aug 2026) and Xero (Sep 2026) work with Claude, ChatGPT and Codex; the Meridian platform itself has no MCP or API."
  },
  "integration_ease": {
   "score": 2,
   "note": "MCP setup is self-serve in under three minutes; the platform is sales-led with no API."
  },
  "market": {
   "score": 2,
   "note": "Accounting firms serving small businesses and startups."
  }
 },
 "api": {
  "public": false,
  "style": "No platform API. Hosted MCP servers over QuickBooks Online and Xero.",
  "auth": "MCP: sign in with Intuit or Xero OAuth and pick companies; no Meridian account or email needed. Platform: delegated OAuth to ledgers, SSO with MFA internally.",
  "limits": "MCP: one connector user per client company or organisation; no shared access, workflow or review. Rate limits not stated.",
  "cost": "MCP servers free (2026-10-10); platform pricing not published.",
  "export": "No export process needed for the books, which stay in the ledger. Export, retention and deletion of Meridian-held files, logs and derived data 'should be confirmed in the contract' (FAQ).",
  "notes": "The QuickBooks MCP post says Intuit's own hosted connector is effectively read-only and cannot read underlying transactions or create arbitrary journal entries (Pilot's claim)."
 },
 "pricing": {
  "model": "Sales-led for firms; the 'one accountant, $1M book' post suggests firms bill a flat monthly close fee",
  "entry": "Not published (30-minute call with sales)",
  "top": "Not published",
  "notes": "Pilot's own bookkeeping service starts at $99 a month (Essentials, up to $100,000 monthly expenses); CFO plans $1,750-$5,250 a month (pilot.com/pricing, 2026-10-10). Those are Pilot's direct-to-business prices, not Meridian's."
 },
 "agents": {
  "mcp": "official",
  "ai_features": "AI Accountant running the month-end close; Categorization Agent (8 Sep 2026) with materiality thresholds; Meridian AI chat for cleanup, restatements and migrations (26 Aug 2026); self-review with firm and client checks and flux analysis; Close Dashboard (6 Oct 2026); client portal.",
  "agent_writes": "Yes. Meridian makes ledger and schedule changes itself, including when a reviewer describes a correction. The free MCP servers give external agents full read-write access to Xero and per-company read-only or read-write access to QBO.",
  "approval": "Platform: nothing is delivered to the client without the firm's approval; sensitive entries can be reserved for manual approval; cleanup scope, period-lock treatment and approval are set before execution. MCP servers: no approval step, only the read-only/read-write choice per company.",
  "notes": "Pilot's direct service is marketed as 'zero human intervention' (4 Feb 2026); Meridian sold to firms keeps a firm sign-off at the end."
 },
 "integration": {
  "scope": "QuickBooks Online and Xero today; NetSuite, Sage Intacct, QuickBooks Desktop, Rillet and custom ledgers in development (FAQ, Oct 2026). Bank and card connections, uploaded statements, payroll and Stripe.",
  "notes": "The books never leave the client's ledger, so there is no migration in or out."
 },
 "preference": {
  "scale": "Pilot says 8,000+ businesses since 2017, 199K months of books closed, 600K transactions a month, $46.2B in assets a year (meridian.pilot.com, 2026-10-10). No count of Meridian firm customers.",
  "ratings": "No Meridian-specific ratings found. Pilot's G2 and Trustpilot pages could not be fetched.",
  "sentiment": "Only vendor posts by Pilot staff: overnight onboarding cleanups, faster reallocations. No independent firm reviews found yet."
 },
 "timeline": [
  {
   "date": "2017",
   "event": "Pilot starts bookkeeping for startups"
  },
  {
   "date": "2018-03",
   "event": "$15M Series A"
  },
  {
   "date": "2021-03-26",
   "event": "Series C at a reported $1.2B valuation (CNBC)"
  },
  {
   "date": "2026-02-04",
   "event": "Pilot unveils its 'AI Accountant', 'zero human intervention'"
  },
  {
   "date": "2026-06-15",
   "event": "Earliest post on the Meridian blog ('One accountant, a $1M book')"
  },
  {
   "date": "2026-08-04",
   "event": "Post on Meridian's AI agent running month-end"
  },
  {
   "date": "2026-08-11",
   "event": "Free hosted QuickBooks MCP server"
  },
  {
   "date": "2026-08-19",
   "event": "'Why we built Meridian': Pilot's automation offered to other firms"
  },
  {
   "date": "2026-08-26",
   "event": "Meridian AI chat interface"
  },
  {
   "date": "2026-09-08",
   "event": "Categorization Agent"
  },
  {
   "date": "2026-09-21",
   "event": "Free hosted Xero MCP server"
  },
  {
   "date": "2026-10-06",
   "event": "Month-end Close Dashboard"
  }
 ],
 "relation": "competitor",
 "relation_note": "Two overlaps. The platform runs agents beside the client's ledger with firm sign-off, the same pattern as Basis. The free hosted Xero and QuickBooks MCP servers compete for ownpurse's 'my agent reaches my books' use case, but they are hosted, keep no record for the owner, and write straight to the ledger without a gate.",
 "borrow": "A per-company read-only or read-write choice when connecting; materiality thresholds that force manual review (e.g. over $25k); reserving sensitive entries for manual approval; setting scope, period-lock treatment and approval before a cleanup runs; a visible Prepare, In progress, Review, Done pipeline; self-review checks before work reaches a person.",
 "avoid": "Full write access for any agent with no approval step (the MCP servers); a hosted passthrough of Xero data whose retention and terms are unstated; logs held by the vendor under separate contract terms.",
 "sources": [
  {
   "title": "Meridian home",
   "url": "https://meridian.pilot.com"
  },
  {
   "title": "Meridian overview on pilot.com",
   "url": "https://pilot.com/meridian"
  },
  {
   "title": "Meridian FAQ",
   "url": "https://meridian.pilot.com/faq"
  },
  {
   "title": "Meridian blog index",
   "url": "https://meridian.pilot.com/blog"
  },
  {
   "title": "We built a hosted MCP server for QuickBooks (11 Aug 2026)",
   "url": "https://meridian.pilot.com/blog/quickbooks-mcp-server"
  },
  {
   "title": "Hosted MCP server for Xero (21 Sep 2026)",
   "url": "https://meridian.pilot.com/blog/xero-mcp-server"
  },
  {
   "title": "Introducing Meridian AI (26 Aug 2026)",
   "url": "https://meridian.pilot.com/blog/introducing-meridian-ai"
  },
  {
   "title": "Why we built Meridian (19 Aug 2026)",
   "url": "https://meridian.pilot.com/blog/why-we-built-meridian"
  },
  {
   "title": "Categorization Agent (8 Sep 2026)",
   "url": "https://meridian.pilot.com/blog/categorization-agent"
  },
  {
   "title": "Month-end Close Dashboard (6 Oct 2026)",
   "url": "https://meridian.pilot.com/blog/month-end-close-dashboard"
  },
  {
   "title": "One accountant, a $1M book",
   "url": "https://meridian.pilot.com/blog/one-accountant-1m-book"
  },
  {
   "title": "Pilot unveils AI Accountant (4 Feb 2026)",
   "url": "https://pilot.com/blog/pilot-unveils-ai-accountant-a-major-leap-toward-artificial-general-intelligence-in-accounting"
  },
  {
   "title": "Pilot about",
   "url": "https://pilot.com/about"
  },
  {
   "title": "Pilot pricing",
   "url": "https://pilot.com/pricing"
  },
  {
   "title": "CNBC: Pilot hits $1.2B valuation (26 Mar 2021)",
   "url": "https://www.cnbc.com/2021/03/26/jeff-bezos-backed-start-up-pilot-hits-1point2-billion-valuation-.html"
  }
 ],
 "unverified": [
  "Exact launch date of Meridian as a product for firms (one post says introduced 'at the beginning of this year'; the earliest blog post is 15 June 2026; the 'June 2026 platform' date in the brief is not confirmed)",
  "Series B date and Series C amount",
  "Founders beyond Jessica McKellar (Waseem Daher and Jeff Arnold are widely reported as co-founders but not confirmed in sources read)",
  "Whether Pilot's hosted Xero MCP server operates under a Xero certified partner arrangement, and who pays Xero egress fees",
  "Data retention and security of the hosted MCP servers",
  "Number of firms using Meridian; Meridian pricing",
  "Pilot ratings on G2/Trustpilot (pages could not be fetched)"
 ],
 "researched": "2026-10-10"
}
---

## What it is

Meridian is [Pilot's](https://pilot.com/meridian) 'AI operating system for accounting firms'. Pilot has kept books for startups since 2017. Meridian packages the automation it built for itself and sells it to other firms. It runs the month-end close across a firm's clients and hands back books for an accountant to review. Pilot's pitch is that 'one accountant can carry a $1M book of business'.

## How it works

A firm connects each client's ledger, then syncs bank and card feeds or uploads statements. Meridian works through a close checklist: categorization (with a [Categorization Agent](https://meridian.pilot.com/blog/categorization-agent) and per-firm materiality thresholds), reconciliations, supporting schedules, then a self-review with firm and client checks and flux analysis. The [Close Dashboard](https://meridian.pilot.com/blog/month-end-close-dashboard) tracks each client through Prepare, In progress, Review and Done. [Meridian AI](https://meridian.pilot.com/blog/introducing-meridian-ai) is a chat interface for ad hoc work such as cleanups, restatements and migrations, and it asks clarifying questions before it makes changes.

The [FAQ](https://meridian.pilot.com/faq) is precise about the gate. Meridian makes ledger and schedule changes itself, and when a reviewer describes a correction it makes that change and reruns the checks. 'Nothing is delivered to the client without the firm's approval.' Sensitive entries can be reserved for manual approval, and for cleanups the firm sets scope, period-lock treatment and approval before anything runs. So the gate sits on delivery, not on every write.

## Data and access

QuickBooks Online and Xero are supported. NetSuite, Sage Intacct, QuickBooks Desktop and Rillet are in development. 'Your books stay in your own general ledger the whole time', so there is nothing to migrate in or out. Meridian-held files, logs and derived data follow separate terms, and the FAQ tells firms to confirm export and retention in the contract. There is no public API and no published price.

## Agents

Pilot also gives away two hosted MCP servers, for [QuickBooks](https://meridian.pilot.com/blog/quickbooks-mcp-server) (11 August 2026) and [Xero](https://meridian.pilot.com/blog/xero-mcp-server) (21 September 2026). They work with Claude, ChatGPT and Codex, and you need only a ledger login. The Xero server gives 'full read and write access', manual journals included, across several organisations at once. On QuickBooks you choose read-only or read-write per company. Neither has an approval step. Pilot advises starting with one journal and reviewing it, and points firms that need review to Meridian. The QuickBooks post argues that Intuit's own hosted connector is effectively read-only.

## Customers

Pilot cites 8,000+ businesses since 2017, 199K months of books closed and 600K transactions a month. How many firms use Meridian is not public. So far the only accounts come from Pilot's own staff. Pilot's direct service is marketed differently: its February 2026 [AI Accountant](https://pilot.com/blog/pilot-unveils-ai-accountant-a-major-leap-toward-artificial-general-intelligence-in-accounting) runs 'with zero human intervention'.

## History

Pilot raised over $150M from Sequoia, Index, Stripe and Bezos Expeditions, among others, with a [Series C in March 2021](https://www.cnbc.com/2021/03/26/jeff-bezos-backed-start-up-pilot-hits-1point2-billion-valuation-.html) at a reported $1.2B. It ran its own bookkeeping on QuickBooks Online. In August 2026 it [explained](https://meridian.pilot.com/blog/why-we-built-meridian) that LLMs let it move from an 'Iron Man suit' for staff to an agent that runs the close, now sold to other firms.

## For ownpurse

Meridian is the strongest proof that leaving the books in place and working beside them is commercially serious. Its free Xero MCP server is the closest thing to ownpurse's agent use case, and it shows the risk ownpurse is designed against: any agent gets direct write access to the books, there is no gate, and the owner keeps no record. ownpurse's answer is to read into a log you hold and to send writes only as sealed packages you approve. Worth borrowing: per-company read and write modes, materiality thresholds and approval set before a cleanup runs.
