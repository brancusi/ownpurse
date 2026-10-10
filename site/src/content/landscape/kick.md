---
{
 "slug": "kick",
 "name": "Kick",
 "url": "https://kick.co",
 "segment": "ai-ledger",
 "status": "active",
 "founded": "2022 (per Dealroom; first public post September 2023)",
 "hq": "Palo Alto, California, US (per Dealroom)",
 "backing": "Venture-backed. USD 9M seed (October 2024) with General Catalyst and the OpenAI Startup Fund plus 60+ angels; a further round in November 2025 brought the total to USD 20M, with Felicis and GV named by Dealroom among investors. Founder and CEO Conrad Wadowski (co-founder of Teachable).",
 "tagline": "Self-driving bookkeeping software for US small business owners and the accounting firms that serve them.",
 "scope": "Kick connects banks, cards and tools such as Stripe, Gusto, Ramp and Mercury, categorizes transactions with an AI model and rules, and posts them to a cash or accrual ledger in real time, with reconciliation, AR/AP, multi-entity, consolidation and revenue recognition on higher plans. It ships an MCP server and a CLI for agents. It does not prepare or file tax returns, hold money or run payroll.",
 "domains": [
  "ledger",
  "bank",
  "documents",
  "close",
  "reporting",
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
   "note": "Kick holds the books; you can pull reports, a year-end tax package and JSON through the CLI, but no full raw export is documented."
  },
  "locality": {
   "score": 0,
   "note": "Cloud only; the CLI is a client of the hosted service."
  },
  "history": {
   "score": 1,
   "note": "An Activity log records who changed what, but journal entries can be updated and deleted along with their postings."
  },
  "openness": {
   "score": 1,
   "note": "MCP and CLI on paid plans; the Platform REST API is not yet public; closed source."
  },
  "breadth": {
   "score": 3,
   "note": "Ledger, bank and card feeds, receipts, AR/AP, payroll imports, revenue recognition and consolidation; no banking, payments or tax filing of its own."
  },
  "autonomy": {
   "score": 4,
   "note": "About 97% of transactions are categorized on first pass and post to the ledger in real time without review; only low-confidence ones wait."
  },
  "agent_readiness": {
   "score": 4,
   "note": "Official MCP with mcp:read and mcp:write scopes, a CLI with json-compact output and stable exit codes, scoped PATs, preview-and-confirm writes, agent docs."
  },
  "integration_ease": {
   "score": 3,
   "note": "An owner can connect an MCP client or the CLI self-serve on a paid plan; partners need Kick to enable the Platform API."
  },
  "market": {
   "score": 1,
   "note": "Sole proprietors, creators and small US businesses, plus the firms that keep their books."
  }
 },
 "api": {
  "public": false,
  "style": "REST Platform API (v1, OpenAPI), hosted MCP server and a CLI",
  "auth": "Organization access tokens (one per organization, reaching every client workspace) for the Platform API; personal access tokens (read-only or write, workspace-bound, expiring) for the CLI and MCP; OAuth with mcp:read and mcp:write scopes for MCP clients",
  "limits": "Platform API pages at 100 items, bulk journal entries up to 100 per atomic request, idempotency keys; numeric rate limits not found (CLI exit code 6 means rate limited)",
  "cost": "MCP and CLI access included from the Basic plan (USD 40/month billed annually as of 2026-10-10); Platform API access by request, price not published",
  "export": "Year-end tax package download; P&L, balance sheet, trial balance, general ledger and AR/AP aging reports; JSON from the CLI; live reports into Google Sheets. No full raw data export documented.",
  "notes": "The Platform API covers workspaces, entities, chart of accounts, Plaid connections, transactions, rules, classes, ledgers, manual journal entries, an accounting migration flow and reports. Writes go through the same path as the app, so a locked period rejects them with 409."
 },
 "pricing": {
  "model": "Freemium subscription per workspace, by plan tier",
  "entry": "Free (1 entity, 250 categorized transactions a year) as of 2026-10-10",
  "top": "USD 300/month (Advanced, billed monthly, sales call)",
  "notes": "Basic USD 40/month billed annually (USD 480); Plus USD 100/month billed quarterly (USD 300 a quarter); Advanced USD 300/month. On Plus and Advanced extra entities are free for their first 250 transactions a year, then USD 50/month each. Accountants are invited at no cost."
 },
 "agents": {
  "mcp": "official",
  "ai_features": "AI categorization (about 97% on first pass), learning from corrections with a prompt to apply them to similar transactions, suggested rules, document processing, receipt matching, transfer matching, AI actions logged under named Kick agents, Google Sheets with Gemini, Claude or ChatGPT, and an Agent Builder page.",
  "agent_writes": "Yes. Write tools can create transactions, categorize, create rules and post journal entries, with an mcp:write scope or a write PAT.",
  "approval": "Partial gate. Kick MCP is read-only by default, and supported write actions are preview-first: the tool returns a preview and a confirmation token, and the change happens only when the token is used. The docs tell users to have the assistant stop at the preview, but the agent itself can confirm, and Kick's own categorization posts without review.",
  "notes": "Hosted endpoint https://use.kick.co/mcp; not yet in the Claude or ChatGPT directories, added as a custom server. The CLI resolves credentials from flag, KICK_PAT, keychain or a config file, sends data to stdout and errors to stderr, and supports table, json, json-compact and yaml output with exit codes 0 to 6."
 },
 "integration": {
  "scope": "Plaid (10,000+ banks), Stripe, PayPal, Mercury, Ramp, Bill, Gusto and 50+ others, a universal importer for statements and CSVs, Google Sheets, MCP clients (Claude, ChatGPT, Gemini, Cursor, Copilot, Perplexity), and a QuickBooks Online migration path.",
  "notes": "The migration API pauses categorization, takes history as journal entries, then builds rules from it, which is a clean way to bring in old books."
 },
 "preference": {
  "scale": "More than 3,000 businesses in the 12 months after launch (Kick, 3 November 2025); about 70 employees (Dealroom, 2026). One bookkeeper is said to close books for 311 businesses.",
  "ratings": "Trustpilot 4.5/5 from 17 reviews in the last 12 months, 94% five-star (fetched 2026-10-10). G2 page could not be fetched.",
  "sentiment": "Reviewers praise time saved by automatic categorization and reconciliation, an interface simpler than QuickBooks, responsive support and accountant year-end reviews, and value against a bookkeeper. Complaints are few: price, and setup choices that cost one reviewer deductions until support fixed them."
 },
 "timeline": [
  {
   "date": "2022",
   "event": "Founded (per Dealroom)"
  },
  {
   "date": "2023-09-13",
   "event": "Introducing Kick post: freemium accounting built with accountants"
  },
  {
   "date": "2024-10-17",
   "event": "USD 9M seed with General Catalyst and the OpenAI Startup Fund; over 1,000 businesses"
  },
  {
   "date": "2025-11-03",
   "event": "Announces USD 20M raised in total; more than 3,000 businesses; pivots toward accounting firms"
  },
  {
   "date": "2026",
   "event": "MCP server, CLI and Platform API (by request) documented; accrual, consolidation and revenue recognition on higher plans"
  }
 ],
 "relation": "competitor",
 "relation_note": "Kick is the closest product to ownpurse in how agents are meant to use it: a CLI with JSON output and stable exit codes, read-only by default, and writes that stop at a preview for confirmation. The difference is custody and history: Kick holds the only ledger in its cloud and lets entries be edited, while ownpurse keeps an append-only record on the owner's machine over the books you already have.",
 "borrow": "Preview-first writes with a confirmation token; read and write scopes and workspace-bound expiring tokens; CLI output modes and exit codes; attributing every change to a person, rule, integration or named agent; the migration flow that builds rules from history.",
 "avoid": "Confirmation that the same agent can give itself; posting AI categorizations straight to the ledger; journal entries that can be deleted with their postings; no full export.",
 "sources": [
  {
   "title": "Kick pricing",
   "url": "https://kick.co/pricing"
  },
  {
   "title": "Kick about",
   "url": "https://www.kick.co/about"
  },
  {
   "title": "Kick resources",
   "url": "https://www.kick.co/resources"
  },
  {
   "title": "Kick raises USD 9M seed (October 2024)",
   "url": "https://www.kick.co/resources/kick-raises-9m-seed-round"
  },
  {
   "title": "Kick has raised USD 20M (November 2025)",
   "url": "https://kick.co/resources/building-accounting-firm-of-the-future"
  },
  {
   "title": "Introducing Kick (September 2023)",
   "url": "https://kick.co/resources/introducing-kick"
  },
  {
   "title": "Kick docs index",
   "url": "https://docs.kick.co/llms.txt"
  },
  {
   "title": "Kick categorization",
   "url": "https://docs.kick.co/accountants/bookkeeping-workflows/categorization.md"
  },
  {
   "title": "Kick activity log",
   "url": "https://docs.kick.co/working-with-your-team/activity.md"
  },
  {
   "title": "Kick permissions and security",
   "url": "https://docs.kick.co/ai/permissions-and-security.md"
  },
  {
   "title": "Kick CLI reference",
   "url": "https://docs.kick.co/ai/developer-tools/cli/cli-reference.md"
  },
  {
   "title": "Kick MCP reference",
   "url": "https://docs.kick.co/ai/developer-tools/mcp/mcp-reference.md"
  },
  {
   "title": "Kick Platform API",
   "url": "https://docs.kick.co/api/readme.md"
  },
  {
   "title": "Kick journal entries API",
   "url": "https://docs.kick.co/api/api-reference/journal-entries.md"
  },
  {
   "title": "Kick migration API",
   "url": "https://docs.kick.co/api/migration.md"
  },
  {
   "title": "Trustpilot: Kick",
   "url": "https://www.trustpilot.com/review/kick.co"
  },
  {
   "title": "Dealroom: Kick",
   "url": "https://dealroom.co/companies/kick/"
  }
 ],
 "unverified": [
  "Size, label and lead investors of the November 2025 round (Felicis and GV named by Dealroom and Neuronfeed; Kick's post names none)",
  "Founding year 2022 and Palo Alto HQ (Dealroom only)",
  "Seed valuation of USD 36M (Dealroom)",
  "Co-founder Adrian's full name",
  "Current customer count after November 2025",
  "Numeric API rate limits",
  "G2 rating (page blocked)",
  "Whether Kick still offers the cash-back and banking product Dealroom describes"
 ],
 "researched": "2026-10-10"
}
---

## What it is

Kick calls itself self-driving bookkeeping. You connect your banks, cards and tools, and Kick categorizes transactions and posts them to your books with little effort from you. It started with sole proprietors and creators in the US, separating personal from business spending to find deductions, and since late 2025 has [built for accounting firms](https://kick.co/resources/building-accounting-firm-of-the-future) too: firms run client workspaces in Kick, and AI does the first draft so staff review instead of prepare. Kick said in November 2025 that more than 3,000 businesses had switched to it in twelve months.

## How it works

Transactions arrive through Plaid, Stripe, Gusto, Ramp, Mercury and 50+ other connections, or through a universal importer for statements and CSVs. A [categorization model](https://docs.kick.co/accountants/bookkeeping-workflows/categorization.md) handles about 97% on first pass and posts them to the general ledger in real time; low-confidence ones wait for a person, and deterministic rules override the model. When you correct one, Kick offers to apply the change to similar transactions. Higher plans add an accrual ledger, AR/AP, multi-entity, consolidation and revenue recognition. An [Activity log](https://docs.kick.co/working-with-your-team/activity.md) records every change and who made it: a person, a rule, an integration or a named Kick agent. Kick does not file taxes; it hands your preparer a year-end package.

## Data and access

[Pricing](https://kick.co/pricing) as of 10 October 2026: Free for one entity and 250 categorized transactions a year, Basic USD 40 a month billed annually, Plus USD 100 a month billed quarterly, Advanced USD 300 a month. Reports cover P&L, balance sheet, trial balance, general ledger and agings, and flow into Google Sheets. The [Platform API](https://docs.kick.co/api/readme.md) reads and writes workspaces, chart of accounts, transactions, rules and journal entries, but is not yet public; Kick enables it per organization. Journal entries can be updated or deleted along with their postings, though a locked period rejects writes. No full raw export is documented.

## Agents

This is where Kick stands out. MCP and CLI access come with Basic. The [CLI](https://docs.kick.co/ai/developer-tools/cli/cli-reference.md) offers json-compact output for agents, data on stdout and errors on stderr, and fixed exit codes for usage, network, auth, not-found and rate-limit failures. The [MCP server](https://docs.kick.co/ai/developer-tools/mcp/mcp-reference.md) separates read and write scopes, and [supported writes are preview-first](https://docs.kick.co/ai/permissions-and-security.md): the tool returns a preview and a confirmation token, and nothing changes until the token is used. The docs warn about prompt injection from memos and emails. The gate is soft, though: the same agent can confirm, and Kick's own model posts without one.

## Customers

[Trustpilot](https://www.trustpilot.com/review/kick.co) shows 4.5 from 17 reviews in the past year (fetched 10 October 2026). People like the time saved, an interface simpler than QuickBooks and the support; the complaints are price and early setup.

## History

Conrad Wadowski, a co-founder of Teachable, started Kick; Dealroom dates it to 2022. The first public post came in September 2023, a [USD 9M seed](https://www.kick.co/resources/kick-raises-9m-seed-round) with General Catalyst and the OpenAI Startup Fund followed in October 2024, and Kick announced USD 20M raised in November 2025.

## For ownpurse

Kick is the nearest product to ownpurse in agent design, and a competitor for a small business owner who wants an agent to keep the books. Its CLI conventions, read and write scopes and preview-then-confirm writes are close to ownpurse's AXI output and sealed packages, which suggests both are on the right track. The differences are the ones ownpurse exists for. Kick holds the only ledger in its cloud, lets entries be edited and deleted, and asks you to move your books in. ownpurse keeps an append-only, hash-chained record on your machine over the books you already have, and its approval belongs to a person, not to a token the agent can spend.
