---
{
 "slug": "puzzle",
 "name": "Puzzle",
 "url": "https://puzzle.io",
 "segment": "ai-ledger",
 "status": "active",
 "founded": "2019 (launched end of 2019 per TechCrunch)",
 "hq": "Unverified (United States)",
 "backing": "Venture-backed: $5M seed and $15M Series A (closed end of 2022), both led by General Catalyst, $20M total as of February 2023 (TechCrunch). Site lists Felicis, Village, Soma Capital and others, plus a syndicate of 'hundreds' of startup CFOs. No later round confirmed.",
 "tagline": "An AI-native accounting ledger for startups and their bookkeepers that asks you to move your books in, with an MCP server whose writes need your confirmation.",
 "scope": "Puzzle is a cloud general ledger with cash and accrual books, real-time burn and runway, revenue recognition from Stripe, and integrations with banks, cards, payroll and spend tools. Its AI Suite (Chat, Cowork, Close) drafts work that accountants approve. It replaces QuickBooks rather than sitting beside it: the default migration imports a verified trial balance as one opening entry. It does not run locally or hold an owner-side copy.",
 "domains": [
  "ledger",
  "bank",
  "payroll",
  "invoicing",
  "close",
  "reporting"
 ],
 "customers_served": [
  "freelancer",
  "smb",
  "accountants"
 ],
 "spectrum": {
  "custody": {
   "score": 1,
   "note": "Puzzle holds the ledger in its cloud; reads are possible through MCP and an on-request API, no documented export."
  },
  "locality": {
   "score": 0,
   "note": "Cloud only, on US servers (Google Cloud named)."
  },
  "history": {
   "score": 2,
   "note": "API docs say the ledger is append-only and changes are tracked rather than overwritten; no as-of queries documented."
  },
  "openness": {
   "score": 1,
   "note": "Closed source; MCP is self-serve but the REST API is by email request; no export documented."
  },
  "breadth": {
   "score": 3,
   "note": "Ledger, banks, cards, payroll, Stripe revenue, invoices and bills, inbox capture."
  },
  "autonomy": {
   "score": 2,
   "note": "Auto-categorises up to 98%, but close agents draft and stop; nothing posts until a person approves (vendor)."
  },
  "agent_readiness": {
   "score": 3,
   "note": "Official remote MCP, read-only by default, opt-in writes with confirmation per change; still alpha and beta."
  },
  "integration_ease": {
   "score": 2,
   "note": "MCP is self-serve OAuth; the API needs a request to api@puzzle.io and endpoints are thinly documented."
  },
  "market": {
   "score": 1,
   "note": "Founders, early-stage startups and small businesses, plus the firms that serve them."
  }
 },
 "api": {
  "public": false,
  "style": "REST returning JSON (Puzzle Accounting API on readme.io); remote MCP at app.puzzle.io/mcp",
  "auth": "API key for the REST API; OAuth with Puzzle account for MCP, choosing which companies to share",
  "limits": "Not published; MCP high-volume use is rate limited during alpha; MCP sessions re-authenticate every 5 days (marketing page) or stay signed in if used every 30 days (help page)",
  "cost": "Not stated; MCP tools included, with a limited read-only set after a trial ends without a paid plan",
  "export": "Not documented. Reads cover balance sheet, income statement, trial balance, transactions, journal entries, chart of accounts, invoices, bills, vendors and customers.",
  "notes": "The API is requested by email to api@puzzle.io; write endpoints are implied by use cases but not documented."
 },
 "pricing": {
  "model": "Flat monthly software tiers, with optional bookkeeping services priced by monthly expense bands and AI credits per tier",
  "entry": "Starter $30 per month (free for 2 months)",
  "top": "Scale $360 per month; Full Service bookkeeping priced by expense band, not listed",
  "notes": "Core $72 (5 seats, up to $10K monthly expenses), Complete $120. Annual billing saves 20%. Migration $149 per system moved; catch-up from $300 per month of books; tax filing from $1,250 a year. A refund guarantee if close time does not fall by at least 50%."
 },
 "agents": {
  "mcp": "official",
  "ai_features": "Up to 98% auto-categorisation; AI Chat; AI Cowork (matching invoices to payments, finding subscriptions; coming soon at launch); AI Close for firms, where each task described in plain language becomes an agent; accuracy review panel.",
  "agent_writes": "Yes, opt-in. New MCP connections are read-only; write access is a beta feature enabled per connection, after a disclaimer.",
  "approval": "The assistant confirms each change before applying it, and Close agents draft entries with reasoning and sources, then stop until a person approves. Approved workflows rerun the same way next close.",
  "notes": "The closest any AI ledger comes to 'agents propose, people post', though the approval sits inside Puzzle's cloud and there is no sealed, owner-held record of what was approved."
 },
 "integration": {
  "scope": "14 integrations listed: Mercury, Meow, Ramp, Brex, Bill.com, Stripe, Gusto, Rippling, Deel, Runway, Causal, Joiin and others; QuickBooks import; MCP clients Claude, ChatGPT, Codex, Cursor, Windsurf.",
  "notes": "Puzzle says its data integrations are read-only and it cannot change your data at the source. QuickBooks migration is by trial balance as a single opening entry; earlier history stays in QuickBooks."
 },
 "preference": {
  "scale": "'7,000+ companies and operators' (site, October 2026); 500+ startups in beta (February 2023).",
  "ratings": "Trustpilot 3.2 from 1 review (a billing complaint, unclaimed profile); G2 and Capterra could not be read.",
  "sentiment": "Firm case studies report faster closes (Accountalent 15-20 days to 3-5). Public independent reviews are too few to judge; the one Trustpilot review complains about annual billing."
 },
 "timeline": [
  {
   "date": "2019",
   "event": "Puzzle launches at the end of 2019, founded by Sasha Orloff (CEO) and John Cwikla (CTO)"
  },
  {
   "date": "2022",
   "event": "$15M Series A led by General Catalyst closes at year end, after a $5M seed also led by General Catalyst"
  },
  {
   "date": "2023-02-16",
   "event": "General availability after a beta with 500+ startups (TechCrunch)"
  },
  {
   "date": "2026-07-09",
   "event": "AI Suite with Close generally available to accounting firms; nothing posts without human approval"
  },
  {
   "date": "2026",
   "event": "MCP server in alpha: read-only by default, opt-in writes in beta with confirmation per change"
  }
 ],
 "relation": "neighbour",
 "relation_note": "Puzzle wants to be your ledger and imports only an opening balance; ownpurse keeps every source's own record beside the ledger you have. Its approval-gated MCP writes are the nearest product statement of ownpurse's 'agents propose, people post'.",
 "borrow": "Read-only by default with per-connection opt-in writes; confirmation of each change; agents that draft with reasoning and sources then stop; approved workflows saved and rerun; the 50% close-time refund guarantee as a way to state a measurable promise.",
 "avoid": "Migration by a single opening entry, which leaves the history behind in the old system; an API you have to email for; approvals that live only in the vendor's cloud with no export of what was approved.",
 "sources": [
  {
   "title": "Puzzle home page",
   "url": "https://puzzle.io"
  },
  {
   "title": "Puzzle pricing",
   "url": "https://puzzle.io/pricing"
  },
  {
   "title": "Puzzle API and MCP",
   "url": "https://puzzle.io/api-mcp"
  },
  {
   "title": "How to set up your Puzzle MCP, Puzzle help",
   "url": "https://help.puzzle.io/en/articles/15331316-how-to-setup-your-puzzle-mcp"
  },
  {
   "title": "Puzzle Accounting API docs",
   "url": "https://puzzle-api.readme.io/docs/welcome"
  },
  {
   "title": "Puzzle launches AI Suite",
   "url": "https://puzzle.io/blog/puzzle-launches-ai-suite"
  },
  {
   "title": "QuickBooks migration guide for messy books, Puzzle",
   "url": "https://puzzle.io/blog/quickbooks-trial-balance-migration"
  },
  {
   "title": "Puzzle security",
   "url": "https://puzzle.io/security"
  },
  {
   "title": "Puzzle integrations",
   "url": "https://puzzle.io/integrations"
  },
  {
   "title": "Puzzle about",
   "url": "https://puzzle.io/about"
  },
  {
   "title": "Puzzle is building a modern accounting package, TechCrunch, 16 February 2023",
   "url": "https://techcrunch.com/2023/02/16/puzzle-is-building-a-modern-accounting-package-for-todays-api-enabled-startups/"
  },
  {
   "title": "Puzzle on Trustpilot",
   "url": "https://www.trustpilot.com/review/puzzle.io"
  }
 ],
 "unverified": [
  "Headquarters location",
  "Any funding after the $15M Series A (end of 2022)",
  "Exact date the MCP server entered alpha and write access entered beta",
  "Whether Puzzle offers a full-history import as well as the trial balance method",
  "Data export options and terms on cancellation",
  "API rate limits and pricing",
  "G2 and Capterra ratings (pages blocked)",
  "The 7,000+ customer figure and 98% categorisation rate are vendor claims",
  "Web search budget ran out mid-research; findings come from direct fetches of primary pages"
 ],
 "researched": "2026-10-10"
}
---

## What it is

Puzzle is a cloud accounting ledger for startups, small businesses and the bookkeepers who serve them. It launched at the end of 2019 and became generally available in 2023 ([TechCrunch](https://techcrunch.com/2023/02/16/puzzle-is-building-a-modern-accounting-package-for-todays-api-enabled-startups/)). It keeps cash and accrual books from one ledger, shows burn and runway as they change, recognises revenue from Stripe and connects to Mercury, Ramp, Brex, Gusto, Rippling and Deel. [Pricing](https://puzzle.io/pricing) runs from Starter at $30 a month to Scale at $360, with bookkeeping services on top.

## How it works

Puzzle asks you to move your books in, the opposite of sitting beside them. The default [migration from QuickBooks](https://puzzle.io/blog/quickbooks-trial-balance-migration) imports a verified trial balance as a single opening entry on a go-live date, for $149. Everything earlier stays in QuickBooks as a read-only archive. Puzzle says its own [data integrations are read-only](https://puzzle.io/security), so it never changes your data at the bank or the card provider. It claims up to 98% automated categorisation and a close up to 50% faster, and it refunds the subscription if your close time does not drop by at least half.

## Data and access

The [Puzzle API](https://puzzle-api.readme.io/docs/welcome) is REST with an API key. You get access by emailing api@puzzle.io, and the docs list resource areas (transactions, accounts, journal entries, statements) rather than endpoints. The docs say the ledger is append-only, so changes are tracked rather than overwritten. We found no documented export or rate limits.

## Agents

This is where Puzzle is most interesting for ownpurse. Its [MCP server](https://help.puzzle.io/en/articles/15331316-how-to-setup-your-puzzle-mcp) at `app.puzzle.io/mcp` works with Claude, ChatGPT, Codex and Cursor. "New connections are read-only." Write access is a beta feature you turn on for each connection. When you turn it on, the assistant shows a disclaimer and then confirms each change before it applies it. The [AI Suite](https://puzzle.io/blog/puzzle-launches-ai-suite), generally available to firms from 9 July 2026, works the same way. Each close task you describe in plain language becomes an agent that drafts entries, shows its reasoning and sources, and stops. "Nothing posts to the ledger until a human approves it." Approved workflows run the same way at the next close. All of it is still alpha or beta, and usage is rate-limited.

## Customers

Puzzle says "7,000+ companies & operators" use it. Firm case studies report faster closes; Accountalent says its close went from 15-20 days to 3-5. We could not read Puzzle's G2 or Capterra pages. [Trustpilot](https://www.trustpilot.com/review/puzzle.io) has one review, a complaint about annual billing.

## History

Sasha Orloff and John Cwikla founded Puzzle. As of February 2023 it had raised $20M: a $5M seed and a $15M Series A, both led by General Catalyst, with more than 100 CFO angels. We confirmed no later round. In 2026 it moved toward accounting firms, with a partner console, AI Close and a Puzzle 2.0 offer that pairs the software with a vetted firm.

## For ownpurse

Puzzle is the nearest product statement of "agents propose, people post": read-only by default, writes opted into per connection, each change confirmed, and close agents that stop before posting. The difference is where the approval lives and what survives a move. Puzzle's approval sits inside Puzzle's cloud, and migrating in keeps one opening entry. In ownpurse, each source's record stays whole in a file you hold, and the approved package is sealed into that record. Worth borrowing: opt-in writes per connection, drafts that cite their sources, and a guarantee you can measure.
