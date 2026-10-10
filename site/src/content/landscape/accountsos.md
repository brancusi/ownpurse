---
{
 "slug": "accountsos",
 "name": "AccountsOS",
 "url": "https://accounts-os.com",
 "segment": "agent-native",
 "status": "active",
 "founded": "November 2025",
 "hq": "London, UK (Thrive Venture Labs Ltd, company no. 16871566)",
 "backing": "Owned by Thrive Venture Labs Ltd, a private company registered in England and Wales; founder Paul Gosnell. No funding is disclosed on the site (checked 2026-10-10). A pre-seed raise of £1.25m to £1.5m appears in a search snippet of a pitch page that now returns 404; unverified.",
 "tagline": "A hosted AI accountant, Finn, that keeps the books for sole traders, small companies and accountancy practices, reachable through chat, MCP and a CLI.",
 "scope": "AccountsOS is its own double-entry ledger in the cloud: receipts, invoices, bank feeds, VAT, payroll, expenses, year-end accounts and tax returns, run by an agent called Finn that messages you when it needs a decision. It files VAT to HMRC under MTD after a person signs. It does not sit beside Xero or QuickBooks: you move your books in (CSV import from Xero), and it is not a chartered accountancy firm.",
 "domains": [
  "ledger",
  "bank",
  "documents",
  "invoicing",
  "payroll",
  "tax"
 ],
 "customers_served": [
  "freelancer",
  "smb",
  "accountants"
 ],
 "spectrum": {
  "custody": {
   "score": 1,
   "note": "The ledger lives on AccountsOS servers (EU, Helsinki per its agent page); you can download an account export and reports, and data stays reachable 30 days after you cancel."
  },
  "locality": {
   "score": 0,
   "note": "Hosted only; the CLI and MCP server are clients of the hosted API."
  },
  "history": {
   "score": 1,
   "note": "Practice Mode keeps a logged audit trail per client; nothing describes an append-only or versioned record an owner can query as of a date."
  },
  "openness": {
   "score": 2,
   "note": "Public, documented API (91 MCP tools, public tool discovery) and a data export; the CLI npm package is MIT, the service itself is closed."
  },
  "breadth": {
   "score": 3,
   "note": "Books, bank feeds, receipts, invoices, payroll, VAT and year-end in one place across 27 countries, but only inside its own ledger."
  },
  "autonomy": {
   "score": 2,
   "note": "Finn works in the background, but filings, payments and postings it proposes wait for a person; API keys with write scope write directly."
  },
  "agent_readiness": {
   "score": 4,
   "note": "MCP server with 91 tools, a CLI, a Claude Code plugin, skill.md agent docs, read/write/admin scoped keys and an agent signup endpoint that issues a read-only key."
  },
  "integration_ease": {
   "score": 3,
   "note": "Self-serve keys and public tool catalogue make it easy to drive; getting data in from another ledger is a CSV import, and bank rules and app connections must be recreated."
  },
  "market": {
   "score": 1,
   "note": "Sole traders and small limited companies, plus the practices that serve them; enterprise is 'coming soon'."
  }
 },
 "api": {
  "public": true,
  "style": "JSON tool calls over HTTPS (POST /api/mcp naming a tool and arguments), the same tools as an MCP server; a public GET /api/mcp lists the catalogue without a key",
  "auth": "Bearer API keys (sk_live_...) created per company in Settings, scoped read, write (includes read) or admin (includes write); keys stored hashed; self-signup keys are read-only until the owner grants write",
  "limits": "Rate limits not published",
  "cost": "No separate API or MCP charge; the Claude Desktop (MCP) integration is included in the plan price",
  "export": "Account data export from Settings (includes document metadata, not necessarily the original files) and report exports (P&L, balance sheet, trial balance); formats not specified",
  "notes": "91 published tools and 4 resources at the time of reading. The developers page tells integrators to build their own review and confirmation steps, because Finn's chat confirmation does not exist in API clients."
 },
 "pricing": {
  "model": "Flat monthly subscription per company, everything included; practices pay per live client, billed in arrears",
  "entry": "€11 a month (self-employed) on the pricing page; the about page states £10 a month + VAT for a sole trader",
  "top": "€22 a month per company (limited company or each company in a group); £20 + VAT on the about page; practices £12 per live client a month, banding down with volume",
  "notes": "14-day free trial for founders and 30 days for practices, no card. Price locked at sign-up with no annual increases. The EUR and GBP figures come from different pages of the same site (read 2026-10-10)."
 },
 "agents": {
  "mcp": "official",
  "ai_features": "Finn, an agent that categorises, reconciles, prepares VAT returns and year-end accounts, remembers preferences, and is reachable by WhatsApp, Slack, Telegram, email, phone, voice, the app, a read-only ChatGPT app (25 September 2026) and Claude",
  "agent_writes": "Yes. External agents with a write-scoped key can create and update transactions, upload documents and manage deadlines and projects; Finn proposes writes inside the product",
  "approval": "Partial. 'Every write action Finn proposes waits for a human to confirm it before it posts', and Finn will not file with a regulator or move money without confirmation. Write access for an API key is a one-time grant by the owner; there is no per-action approval for API writes",
  "notes": "The agent surface is broad and well documented for a product this young; the read-only default for agent-issued keys is a good pattern."
 },
 "integration": {
  "scope": "Open Banking bank feeds; CSV import from Xero (columns mapped, contacts created); HMRC MTD VAT submission; ChatGPT and Claude; Claude Code plugin and npm CLI; 27 countries",
  "notes": "It is a destination, not a bridge: it does not read from Xero, QuickBooks or Sage live (the ChatGPT post says so), so a business keeps one ledger, the AccountsOS one."
 },
 "preference": {
  "scale": "No customer count is published; the company names customers only with permission (about page, read 2026-10-10). First live VAT return filed to HMRC on 7 July 2026.",
  "ratings": "Trustpilot 3.8 from 2 reviews (both 5-star, July 2026; profile claimed July 2026). No G2 or Capterra listing found.",
  "sentiment": "The two reviews praise the interactivity (reminders, anticipating the next step) and quick bug fixes. Too few reviews to say anything about typical experience."
 },
 "timeline": [
  {
   "date": "2025-11",
   "event": "Founded by Paul Gosnell after setting up a UK limited company"
  },
  {
   "date": "2026-04-16",
   "event": "CLI published on npm as @thriveventurelabs/accountsos (MIT)"
  },
  {
   "date": "2026-04-24",
   "event": "Claude Code and Cowork plugin repository created"
  },
  {
   "date": "2026 (early)",
   "event": "Live in the UK"
  },
  {
   "date": "2026-06-05",
   "event": "MCP server repository created"
  },
  {
   "date": "2026-07-07",
   "event": "First live VAT return filed to HMRC"
  },
  {
   "date": "2026-09-25",
   "event": "Read-only Finn app live in ChatGPT"
  },
  {
   "date": "2026-10",
   "event": "Site states live in 27 countries, 91 MCP tools"
  }
 ],
 "relation": "competitor",
 "relation_note": "AccountsOS asks a small business to move its books into an agent-run hosted ledger; ownpurse leaves the books where they are and gives the owner a local, verifiable record beside them. They compete for the same person (a director who wants an agent to do the books) with opposite answers on custody.",
 "borrow": "Read-only keys by default for agent signups, with write granted by the owner; a public, keyless tool catalogue; a skill.md that tells agents the request formats and rules; human sign-off on every filing stated plainly.",
 "avoid": "Letting a write-scoped API key write without a per-action gate, and telling integrators to build their own review step; holding the only authoritative copy in the vendor's cloud; prices that differ by page and currency.",
 "sources": [
  {
   "title": "AccountsOS home page",
   "url": "https://accounts-os.com"
  },
  {
   "title": "AccountsOS pricing",
   "url": "https://accounts-os.com/pricing"
  },
  {
   "title": "AccountsOS about",
   "url": "https://accounts-os.com/about"
  },
  {
   "title": "AccountsOS developers and API",
   "url": "https://accounts-os.com/developers"
  },
  {
   "title": "AccountsOS for agents",
   "url": "https://accounts-os.com/for-agents"
  },
  {
   "title": "AccountsOS skill.md",
   "url": "https://accounts-os.com/skill.md"
  },
  {
   "title": "AccountsOS security",
   "url": "https://accounts-os.com/security"
  },
  {
   "title": "AccountsOS compared with Xero",
   "url": "https://accounts-os.com/compare/xero"
  },
  {
   "title": "Chat with your books in ChatGPT: Finn is live (25 September 2026)",
   "url": "https://accounts-os.com/blog/finn-live-in-chatgpt"
  },
  {
   "title": "npm: @thriveventurelabs/accountsos",
   "url": "https://www.npmjs.com/package/@thriveventurelabs/accountsos"
  },
  {
   "title": "GitHub: accountsos-mcp-server",
   "url": "https://github.com/thriveventurelabs/accountsos-mcp-server"
  },
  {
   "title": "Trustpilot: AccountsOS",
   "url": "https://www.trustpilot.com/review/accounts-os.com"
  }
 ],
 "unverified": [
  "Pre-seed raise of £1.25m to £1.5m (pitch page returns 404; seen only in a search snippet)",
  "Data location: the agents page says EU (Helsinki); a search snippet of another page said records stay in London; the security page does not say",
  "HMRC MTD VAT recognition is claimed by AccountsOS on its Xero comparison page; not checked against HMRC's software list",
  "'Top 10 in the world' AI accounting ranking claim has no source or date",
  "Whether the 91-tool count and 27-country claim are current",
  "Licence of the MCP server and plugin repositories (GitHub reports NOASSERTION)",
  "No independent customer or revenue figures"
 ],
 "researched": "2026-10-10"
}
---

## What it is

AccountsOS is a hosted accounting product built around an agent called Finn. It is made by [Thrive Venture Labs Ltd](https://accounts-os.com/about), a London company founded in November 2025 by Paul Gosnell, who built it after setting up his own limited company and finding the software assumed he was an accountant. It serves sole traders, small limited companies, groups and accountancy practices, and says it is live in 27 countries. It is not a chartered accountancy firm and says so.

## How it works

AccountsOS keeps its own double-entry ledger. Finn categorises receipts and bank lines, runs VAT, payroll, month end and year-end accounts, and messages you on WhatsApp, Slack, email or voice when it needs a decision. In the UK it submits MTD VAT returns to HMRC; it filed its first live return on 7 July 2026. Filings are [never automatic](https://accounts-os.com): a person reviews and signs. Practices get a Practice Mode that runs work across a client book and marks each client ready or blocked, with role-based access and a logged audit trail.

Pricing is flat and includes everything. The [pricing page](https://accounts-os.com/pricing) lists €11 a month for the self-employed and €22 a month per company; the [about page](https://accounts-os.com/about) lists £10 and £20 + VAT, and £12 per live client a month for practices. There is a 14-day trial (30 for practices) with no card.

## Data and access

The ledger is on AccountsOS servers; the [agents page](https://accounts-os.com/for-agents) says the EU (Helsinki). You can download an account export and the standard reports, and data stays available 30 days after you cancel. Moving in from Xero is a CSV import that maps columns and creates contacts; bank rules and app connections must be rebuilt. AccountsOS does not read Xero, QuickBooks or Sage live.

The [developer page](https://accounts-os.com/developers) is unusually open for a product this size: a public tool catalogue you can fetch without a key, 91 tools and 4 resources, and Bearer keys scoped read, write or admin. No rate limits are published. API access costs nothing extra.

## Agents

This is the product's strength. There is an MCP server, an [npm CLI](https://www.npmjs.com/package/@thriveventurelabs/accountsos) (MIT), a Claude Code plugin with slash commands such as `/vat-check`, a [skill.md](https://accounts-os.com/skill.md) for agents, and an endpoint where an agent can sign up and receive a read-only key. Write access must be granted by the owner in Settings; until then every write tool returns 403. Inside the product, "every write action Finn proposes waits for a human to confirm it before it posts." The ChatGPT app, live since [25 September 2026](https://accounts-os.com/blog/finn-live-in-chatgpt), is read-only.

The gap is at the API edge. Once a key has write scope, an external agent creates and updates transactions directly, and the developer page tells integrators to add their own review step because Finn's confirmation screen does not exist there. The approval is a one-time grant, not a per-write gate.

## Customers

No customer count is published. [Trustpilot](https://www.trustpilot.com/review/accounts-os.com) shows two reviews, both five stars from July 2026, praising the reminders and quick fixes. That is too small a sample to judge.

## History

Founded November 2025, the CLI on npm in April 2026, live in the UK early in 2026, the MCP server repository in June, the first HMRC filing in July, the ChatGPT app in September. Figures here are the company's own.

## For ownpurse

AccountsOS shows how far an agent-first surface can go in a year: scoped keys, a read-only default for agents, public tool discovery, and a plain rule that a person signs every filing. ownpurse should borrow all of that. The difference is custody. AccountsOS asks you to move your books into its cloud and replaces Xero; ownpurse keeps the record on your machine, beside whatever ledger you already use, and every write to a source goes out as a sealed package a person approves, per write, not per key.
