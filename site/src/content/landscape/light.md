---
{
 "slug": "light",
 "name": "Light",
 "url": "https://light.inc",
 "segment": "ai-ledger",
 "status": "active",
 "founded": "2022 (per Balderton and Silicon Canals; Sacra says 2023)",
 "hq": "Copenhagen, Denmark; London office; New York office planned in Sept 2025",
 "backing": "$43M raised: $13M seed led by Atomico (June 2024, per Sacra) and $30M Series A led by Balderton Capital (25 September 2025) with Atomico, Cherry Ventures, Seedcamp, Entrée Capital and angels including Thomas Wolf (Hugging Face) and Charles Songhurst (Meta board); private",
 "tagline": "Light is a Copenhagen-built cloud ledger and financials platform for multi-entity companies, with agents that code bills, clear bank lines and book accruals inside policies you write.",
 "scope": "Light replaces the ERP and the stack around it for multi-entity companies: one ledger for every entity, payables and receivables, global bill pay, cards and spend, subscriptions, consolidation, multibook accounting, e-invoicing and reporting, with 600+ apps and agents on top. It is financials only, with no manufacturing or warehouse modules, and like the others it asks you to move your books in.",
 "domains": [
  "ledger",
  "bank",
  "payments",
  "invoicing",
  "spend",
  "documents",
  "tax",
  "close",
  "reporting",
  "integration"
 ],
 "customers_served": [
  "smb",
  "mid-market"
 ],
 "spectrum": {
  "custody": {
   "score": 1,
   "note": "Light's cloud holds the authoritative books; the REST API allows a paged full read, but there is no bulk export endpoint."
  },
  "locality": {
   "score": 0,
   "note": "Cloud only."
  },
  "history": {
   "score": 3,
   "note": "Documented: 'Ledger lines are never edited or deleted', and corrections are reversals that stay beside the original; documents can return to draft, and there are no as-of queries."
  },
  "openness": {
   "score": 2,
   "note": "Closed source; public REST API (about 221 endpoints) with self-serve keys; no bulk export endpoint."
  },
  "breadth": {
   "score": 3,
   "note": "Ledger, AP, AR, cards, bill pay, receipts, e-invoicing, consolidation and 600+ apps in one place; still one company's books, no personal finance."
  },
  "autonomy": {
   "score": 3,
   "note": "Agents code bills, clear bank transactions and book accruals under written policies, and people approve exceptions; Custom Agents run on schedules."
  },
  "agent_readiness": {
   "score": 4,
   "note": "Official MCP server with about 100 tools filtered to the caller's roles, a Claude connector, personal tokens with expiry, Custom Agents and a CFO agent pack for Claude Code."
  },
  "integration_ease": {
   "score": 3,
   "note": "Self-serve keys, clear docs with documented rate limits and idempotency; OAuth needs Light support to set up."
  },
  "market": {
   "score": 3,
   "note": "Multi-entity, multi-currency mid-market companies and fast-growing startups (Lovable, Legora)."
  }
 },
 "api": {
  "public": true,
  "style": "REST, JSON (camelCase), base https://api.light.inc; amounts as integers in minor units paired with currency; separate MCP endpoint (JSON-RPC 2.0)",
  "auth": "Self-serve API keys (Settings > API Keys) with roles, sent with the Basic scheme as-is; OAuth 2.0 authorization code flow with rotating refresh tokens, set up through Light support; MCP uses personal lmcp_ tokens (up to 20 per user, optional expiry) or the Claude connector sign-in",
  "limits": "300 requests per minute per credential and 100,000 per day per organisation; 429 with limit headers; MCP returns rate-limit errors inside a 200 JSON-RPC response",
  "cost": "No separate API fee found",
  "export": "No bulk export endpoint; list endpoints page (default 50, cap 200; ledger lines up to 1,000 with a single postingDate sort) with filters and guidance on checking a full extract",
  "notes": "About 221 endpoints across 33 resources. Idempotency keys on bill creation and four bill actions, not every write. Docs warn offset paging can skip or repeat rows after inserts. Webhooks not documented."
 },
 "pricing": {
  "model": "Quote-based subscription; Sacra suggests per-entity pricing scaled to multinational complexity",
  "entry": "Not published; light.inc/pricing returned 404 on 2026-10-10",
  "top": "Not published",
  "notes": "Calls to action are 'Book a demo' and 'Build your Light'. Migration takes 4 to 16 weeks from signature to first close, per Light's FAQ."
 },
 "agents": {
  "mcp": "official",
  "ai_features": "Light agents (Bill Agent, Internal Audit Agent and others) code bills, clear bank transactions, book month-end accruals and chase missing receipts; Custom Agents written in plain English and run on schedules (up to 50 per company); Slack, Teams and Google Sheets plugins; CFO agent pack for Claude Code",
  "agent_writes": "Yes. The MCP server (https://api.light.inc/rest/ext/mcp) exposes about 100 tools: search and read, create and submit, approve, and flagged writes such as creating journal entries, updating bills and triggering workflows. It acts as the connected person with their roles.",
  "approval": "Light says every MCP action runs through its permissions, approval workflows and audit trail, and that the CFO agent pack puts a person in front of every write. Agents work inside policies the customer writes, with people approving exceptions. For Custom Agents, approval before acting is something you ask the agent for in its instructions, not a separate setting.",
  "notes": "Custom Agents each have their own identity and permissions, and every run is logged with the prompt, the tools called and the result. Tools/list is filtered to what the caller may do."
 },
 "integration": {
  "scope": "600+ apps in the Light App Store: banks, payroll, CRM, BI and planning, e-invoicing networks; partners include JP Morgan, Adyen and BDO",
  "notes": "Replaces per-entity Xero or QuickBooks, NetSuite, Dynamics, Sage Intacct and SAP. Holds SOC 1 Type 2 and SOC 2 Type 2, per its site."
 },
 "preference": {
  "scale": "Vendor-claimed 30x growth over the 12 months to September 2025 and 84% less time on finance operations after switching; named customers include Lovable, Sana, Legora, Famly, Tillo, Ocean.io, All Gravy, Officeguru, SEON, Perlego",
  "ratings": "No G2, Capterra or Trustpilot rating confirmed in this pass",
  "sentiment": "Case studies: Tillo cut its close from 12 days to 5 across four entities; Famly cut revenue processing from 20 hours to 15 minutes; Officeguru went live in 7 days. Independent customer reviews not found in this pass."
 },
 "timeline": [
  {
   "date": "2022",
   "event": "Founded in Copenhagen by CEO Jonathan Sanders (Sacra says 2023)"
  },
  {
   "date": "2024-06",
   "event": "$13M seed led by Atomico with Cherry Ventures and Seedcamp (per Sacra)"
  },
  {
   "date": "2025-09-25",
   "event": "$30M Series A led by Balderton; total $43M; plans to open in New York and triple engineering by Q2 2026"
  },
  {
   "date": "2026",
   "event": "MCP server, Claude connector, Custom Agents and CFO agent pack published (exact dates not given)"
  }
 ],
 "relation": "neighbour",
 "relation_note": "Light is the European version of the replacement thesis. It asks multi-entity companies to retire per-entity Xero or QuickBooks and NetSuite and run on its cloud ledger. ownpurse keeps whatever ledger you have and gives you the copy. Light's ledger is the closest of the three to ownpurse's own model (append-only lines, reversals, minor-unit integers), and its complete API makes it a plausible future source.",
 "borrow": "'Ledger lines are never edited or deleted', with corrections as reversals that stay beside the original. Amounts as integers in minor units paired with a currency. MCP tool lists filtered to what the caller may do. Personal agent tokens with expiry, kept separate from API keys. A jobs table that records each agent run's prompt, tools and result. Docs that warn offset paging can skip rows.",
 "avoid": "Approval for Custom Agents that is an instruction to the agent ('confirm with you first'), not a structural gate. No bulk export, so a complete copy means paging through everything. Idempotency on only some writes.",
 "sources": [
  {
   "title": "Light home page",
   "url": "https://light.inc/"
  },
  {
   "title": "Light API docs",
   "url": "https://light.inc/docs"
  },
  {
   "title": "Light docs: authentication",
   "url": "https://light.inc/docs/getting-started/authentication"
  },
  {
   "title": "Light docs: AI clients (MCP)",
   "url": "https://light.inc/docs/getting-started/ai-clients"
  },
  {
   "title": "Light docs: how Light records money",
   "url": "https://light.inc/docs/concepts/how-light-records-money"
  },
  {
   "title": "Light docs: pagination, filtering and errors",
   "url": "https://light.inc/docs/getting-started/pagination-filtering-errors"
  },
  {
   "title": "Light: Claude MCP",
   "url": "https://light.inc/products/claude-mcp"
  },
  {
   "title": "Light: Custom Agents",
   "url": "https://light.inc/products/custom-agents"
  },
  {
   "title": "Balderton: Light raises $30M Series A (25 Sep 2025)",
   "url": "https://www.balderton.com/news/light-raises-30m-series-a-to-replace-legacy-finance-systems-with-ai-native-platform/"
  },
  {
   "title": "Silicon Canals: Copenhagen's Light gets €25.5M",
   "url": "https://siliconcanals.com/copenhagen-light-gets-25-5m/"
  },
  {
   "title": "Sacra: Light",
   "url": "https://sacra.com/c/light"
  }
 ],
 "unverified": [
  "Founding year: 2022 (Balderton, Silicon Canals) versus 2023 (Sacra)",
  "Seed round: $13M led by Atomico in June 2024 per Sacra; Dealroom headline says €12.1M",
  "Whether the New York office opened and engineering tripled by Q2 2026",
  "Dates the MCP server, Claude connector, Custom Agents and CFO agent pack shipped",
  "Pricing model (per-entity) is Sacra's inference; no prices published",
  "Growth (30x) and 84% time savings are vendor claims",
  "No G2, Capterra or Trustpilot ratings found; Reddit not checked because the web search budget ran out during this pass",
  "Founder backgrounds beyond 'serial entrepreneur' not confirmed"
 ],
 "researched": "2026-10-10"
}
---

## What it is

Light is a cloud financials platform from Copenhagen for companies that run several entities across currencies and countries. It offers one ledger for every entity, with payables, receivables, global bill pay, cards, spend and subscription management, consolidation, multibook accounting, e-invoicing and reporting. On top sit 600+ apps and a set of agents ([Light](https://light.inc/)). It sells itself as the replacement for NetSuite, Dynamics, Sage Intacct, SAP, and the per-entity Xero or QuickBooks setups that companies outgrow. It is financials only, with no manufacturing or warehouse modules.

## How it works

You move your entities into Light. Light says go-live takes as little as four weeks and its FAQ says 4 to 16 weeks to the first close. Its concepts page is unusually precise about the ledger: "Ledger lines are never edited or deleted." Every report is a sum over lines, and a correction is a reversal, a new transaction with the directions flipped, kept next to the original. Documents go from draft to posted to settled, and only drafts can be edited ([how Light records money](https://light.inc/docs/concepts/how-light-records-money)). Agents code bills, clear bank transactions, book accruals and chase missing receipts inside policies the customer writes, and people approve the exceptions.

## Data and access

The REST API at `api.light.inc` has about 221 endpoints across 33 resources. Amounts are integers in minor units, each paired with its currency ([API docs](https://light.inc/docs)). Keys are self-serve and carry roles. OAuth 2.0 with rotating refresh tokens is available but has to be set up through support ([authentication](https://light.inc/docs/getting-started/authentication)). Limits are 300 requests a minute per credential and 100,000 a day per organisation. There is no bulk export. List endpoints page at up to 200 rows (1,000 for ledger lines with a single sort), and the docs warn that offset paging can skip or repeat rows after inserts ([pagination](https://light.inc/docs/getting-started/pagination-filtering-errors)). Pricing is not published.

## Agents

Light is built for agents to drive. Its official MCP server exposes about a hundred tools: search and read, create and submit, approve, and flagged writes. The tool list is filtered to what the caller may do, and the server acts as the person who connected. Personal `lmcp_` tokens can expire and are kept apart from API keys, and Claude users can sign in through a connector instead ([AI clients](https://light.inc/docs/getting-started/ai-clients)). Light says every MCP action goes through its permissions, approval workflows and audit trail, and that its CFO agent pack for Claude Code puts a person in front of every write ([Claude MCP](https://light.inc/products/claude-mcp)). Custom Agents are plain-English jobs on a schedule, up to fifty per company. Each run is logged with its prompt, tools and result. To have an agent wait for approval, you tell it to confirm with you first ([Custom Agents](https://light.inc/products/custom-agents)).

## Customers

Named customers include Lovable, Sana, Legora, Famly, Tillo, SEON and Perlego. Light claimed 30x growth over the 12 months to September 2025. Its case studies report Tillo's close falling from 12 days to 5 and Officeguru going live in 7 days. Light holds SOC 1 Type 2 and SOC 2 Type 2. This pass found no independent review ratings.

## History

Jonathan Sanders founded Light in Copenhagen in 2022. A $13M seed led by Atomico followed in June 2024 ([Sacra](https://sacra.com/c/light)). On 25 September 2025 Balderton led a $30M Series A, for $43M in total, with plans to open in New York and triple engineering by Q2 2026 ([Balderton](https://www.balderton.com/news/light-raises-30m-series-a-to-replace-legacy-finance-systems-with-ai-native-platform/)).

## For ownpurse

Light's ledger model is the closest of the AI ERPs to ownpurse's: lines that are never edited, corrections as reversals, money as integers with a currency. The difference is who holds it. Light keeps the one copy in its cloud and offers no bulk export, while ownpurse gives the owner the authoritative record and leaves the books where they are. Worth taking: tool lists filtered to the caller's permissions, expiring agent tokens separate from API keys, and a run log that records the prompt and the tools. Worth avoiding: approval as a sentence in an agent's prompt, not a sealed package a person signs.
