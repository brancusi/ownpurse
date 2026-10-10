---
{
 "slug": "bitwave",
 "name": "Bitwave",
 "url": "https://www.bitwave.io",
 "segment": "crypto-accounting",
 "status": "active",
 "founded": "2018",
 "hq": "Fully remote (team in 7+ countries); July 2026 press release datelined San Francisco",
 "backing": "Venture-backed private company founded 2018 by Pat White (CEO) and Amy Kalnoki (COO). No funding figures appear on its own site or the July 2026 release; round sizes and investors are unverified here.",
 "tagline": "An enterprise back office for digital assets (subledger, payments, reporting) that added an open-source agent CLI and an MCP server in July 2026.",
 "scope": "Bitwave pulls on-chain, exchange, custodian and DeFi activity into a hosted digital-asset subledger, works out cost basis, gains and impairment, and posts journal entries to QuickBooks, Xero, NetSuite or Sage Intacct. It also does stablecoin invoicing and bill pay. Since July 2026 it ships Bitwave CLI (AGPL-3.0, local plain-text double-entry books compatible with hledger and ledger), an MCP server over a set of platform workflows, a data and SQL analytics product, and KubeClaw, a self-hosted Kubernetes sandbox for agents. It is not a general ledger for the whole business, and the enterprise platform is cloud-only.",
 "domains": [
  "ledger",
  "crypto",
  "payments",
  "invoicing",
  "reporting",
  "close",
  "spend"
 ],
 "customers_served": [
  "smb",
  "mid-market",
  "enterprise",
  "accountants",
  "developers"
 ],
 "spectrum": {
  "custody": {
   "score": 2,
   "note": "The enterprise subledger lives in Bitwave's cloud, but CLI books are plain-text journal files in a directory the owner holds and can commit to git."
  },
  "locality": {
   "score": 2,
   "note": "The platform is cloud-only; the CLI runs on your machine and KubeClaw runs on your own Kubernetes."
  },
  "history": {
   "score": 1,
   "note": "CLI journals can be diffed and versioned in git, but nothing enforces append-only; the platform's internal history model is not documented publicly."
  },
  "openness": {
   "score": 3,
   "note": "The CLI is AGPL-3.0 and its accounting SDK MPL-2.0, and KubeClaw is described as open source; the core platform is closed, with a REST API."
  },
  "breadth": {
   "score": 3,
   "note": "It covers more than 70 chains, exchanges, custodians, DeFi, four ERPs, payments and expense reports, but only for digital assets."
  },
  "autonomy": {
   "score": 3,
   "note": "Rules categorize transactions, detect internal transfers and post directly to the ledger and ERP; by Bitwave's own account the MCP drafts entries and needs a person's approval."
  },
  "agent_readiness": {
   "score": 4,
   "note": "An agent-first CLI with a 'Agents, please read' section, an authenticated MCP server, SQL data access and an agent sandbox."
  },
  "integration_ease": {
   "score": 3,
   "note": "The CLI installs from brew or npm, and the REST API with keys and OAuth-style tokens is documented; platform access is sold through sales."
  },
  "market": {
   "score": 4,
   "note": "Sold to crypto startups, enterprises and institutions (exchanges, custodians); pricing is by quote only."
  }
 },
 "api": {
  "public": true,
  "style": "REST (Swagger JSON published); asynchronous report jobs you poll; plus a local CLI and an MCP server",
  "auth": "API key, or a v2 OAuth-style access token; MCP uses a user sign-in and authorization",
  "limits": "Rate limits not documented on the docs page reviewed",
  "cost": "Platform access is by sales quote; the CLI is free and open source. No metering or egress terms found.",
  "export": "The Balance Report API returns structured rows or CSV; the CLI exports journal entries and CSV, and its books are plain-text files that are already an export",
  "notes": "The docs cover rate tables, invoices, customers, payment accounts and balance reports; sample code is on GitLab. Platform API coverage beyond these was not confirmed."
 },
 "pricing": {
  "model": "Quote-based tiers (Startups, Enterprises, Institutions); open-source CLI free",
  "entry": "Startups tier, price not published (Talk to Sales)",
  "top": "Institutions tier, price not published; adds BigQuery or Snowflake integrations and advanced 1099-DA reporting",
  "notes": "Every tier lists high-volume tracking, ERP integrations, lot tracking and multi-book treatments. Subsidiary support starts at Enterprises. No prices or transaction limits are published (checked 2026-10-10)."
 },
 "agents": {
  "mcp": "official",
  "ai_features": "Bitwave Agentic (launched 2026-07-23): CLI, MCP, Data and Analytics (normalized chain data for Ethereum, Base and Berachain with SQL), KubeClaw (a sandbox with tool controls and approval workflows). Agent expense reports let agents report their own spending.",
  "agent_writes": "Yes. The CLI writes balance-checked entries to local books or to a cloud workspace and can send from wallets. The MCP can add wallets and categorize transactions.",
  "approval": "Mixed. Bitwave says the MCP drafts entries and a person approves before anything posts. The CLI has --dry-run for wallet sends and asks agents to confirm irreversible actions, which is guidance, not an enforced gate. KubeClaw claims approval workflows but does not detail them.",
  "notes": "Of the vendors in this survey, this is the closest to ownpurse's framing: local books, agent-first, open source. The difference is that the local books are hledger-style journals, not an append-only record of what sources reported, and the local tool leads into a hosted workspace."
 },
 "integration": {
  "scope": "More than 70 blockchains and DeFi ecosystems; exchanges (Coinbase, Kraken, Binance, Gemini and others); custodians (Fireblocks, BitGo, Anchorage, NYDIG); multisig (Safe); ERPs QuickBooks, Xero, NetSuite, Sage Intacct; Rillet integration; stablecoin rails (Hedera, Canton, Ripple RLUSD, BVNK)",
  "notes": "Most connectors run inside the hosted platform. The CLI's wallet sync is a separate, smaller surface."
 },
 "preference": {
  "scale": "Customer logos include Coinbase, OpenSea, Figment, Blockdaemon and Art Blocks; no customer count is published (site checked 2026-10-10)",
  "ratings": "G2 badges (Winter Accounting Grid) cited on its site, but the G2 page was not reachable to confirm a score or count. Trustpilot: unclaimed, 0 reviews (2026-10-10). The bitwave-cli GitHub repo had 0 stars, 3 forks and 81 commits (2026-10-10).",
  "sentiment": "Testimonials on its site (Greenidge, Art Blocks, Dexalot, Temple Digital) praise faster closes and easier access to data, but give no numbers. Independent review volume is thin."
 },
 "timeline": [
  {
   "date": "2018",
   "event": "Founded by Pat White and Amy Kalnoki"
  },
  {
   "date": "2025-07",
   "event": "Partnership with BVNK for stablecoin invoice payments"
  },
  {
   "date": "2026-03",
   "event": "Ripple RLUSD added to the enterprise payment module"
  },
  {
   "date": "2026-04",
   "event": "Hedera (USDC and HBAR) and Canton payment integrations"
  },
  {
   "date": "2026-07-23",
   "event": "Bitwave Agentic announced: CLI, MCP, Data and Analytics, KubeClaw"
  },
  {
   "date": "2026-09",
   "event": "Blog posts on MCP accounting and plain-text accounting (Beancount vs hledger) updated"
  }
 ],
 "relation": "neighbour",
 "relation_note": "The nearest neighbour in framing: a local, open-source, agent-first books tool. It is crypto first, and its CLI is a way into a hosted enterprise subledger rather than an owner-held record of every source. It could also be a source for ownpurse, via its exports or its plain-text journals.",
 "borrow": "Plain-text books that are diffable and compatible with hledger and ledger; one switch (a TOML field) between local and shared mode; an 'Agents, please read' section in the README; dry-run before irreversible actions; agent expense reports as a first-class entry type; positioning MCP for interactive work and the CLI for automation.",
 "avoid": "Approval that is only advice to the agent (confirm before broadcasting) rather than an enforced gate; journals the owner can edit in place, with no hash chain; a local tool whose upgrade path is a hosted workspace; keeping wallet keystores beside the books.",
 "sources": [
  {
   "title": "Bitwave home page",
   "url": "https://www.bitwave.io"
  },
  {
   "title": "Bitwave about",
   "url": "https://www.bitwave.io/about"
  },
  {
   "title": "Bitwave pricing",
   "url": "https://www.bitwave.io/pricing"
  },
  {
   "title": "Bitwave CLI site",
   "url": "https://cli.bitwave.io"
  },
  {
   "title": "bitwave-cli on GitHub",
   "url": "https://github.com/bitwave-io/bitwave-cli"
  },
  {
   "title": "Bitwave developer docs",
   "url": "https://docs.bitwave.io"
  },
  {
   "title": "Introducing Bitwave MCP",
   "url": "https://www.bitwave.io/blog/bitwave-mcp-for-agentic-finance"
  },
  {
   "title": "Introducing Bitwave CLI",
   "url": "https://www.bitwave.io/blog/bitwave-ai-command-line-interface"
  },
  {
   "title": "Introducing Bitwave Agentic (KubeClaw)",
   "url": "https://www.bitwave.io/blog/introducing-bitwave-agentic"
  },
  {
   "title": "MCP accounting has arrived: the market map (Bitwave blog)",
   "url": "https://www.bitwave.io/blog/mcp-accounting-market-map"
  },
  {
   "title": "Bitwave launches Agentic Finance initiative (GlobeNewswire via FinancialContent, 2026-07-23)",
   "url": "https://www.financialcontent.com/article/gnwcq-2026-7-23-bitwave-launches-agentic-finance-initiative-unveils-open-source-infrastructure-for-ai-native-financial-operations"
  },
  {
   "title": "PYMNTS on the Bitwave Agentic rollout",
   "url": "https://www.pymnts.com/?p=3996942"
  },
  {
   "title": "Trustpilot: bitwave.io",
   "url": "https://www.trustpilot.com/review/bitwave.io"
  }
 ],
 "unverified": [
  "Funding: round sizes, dates and investors (not on its own pages; G2 and Crunchbase were blocked)",
  "G2 rating and review count",
  "API rate limits",
  "The exact launch date of the MCP, beyond 'shortly after' the CLI around 2026-07-23",
  "Whether the MCP approval gate is enforced server-side for every write (stated only in Bitwave's own blog)",
  "KubeClaw's license and how its approval workflows work",
  "The platform's internal history and audit model"
 ],
 "researched": "2026-10-10"
}
---

## What it is

Bitwave is an enterprise back office for digital assets, founded in 2018 by Pat White and Amy Kalnoki and run as a fully remote team ([about](https://www.bitwave.io/about)). It sells a hosted subledger that turns on-chain, exchange, custodian and DeFi activity into journal entries, with cost basis, gains, impairment and fair-value reports. It also sells stablecoin invoicing and bill pay. Customers listed on [its site](https://www.bitwave.io) include Coinbase, OpenSea, Figment and Art Blocks. No customer count is published.

On 2026-07-23 it announced [Bitwave Agentic](https://www.financialcontent.com/article/gnwcq-2026-7-23-bitwave-launches-agentic-finance-initiative-unveils-open-source-infrastructure-for-ai-native-financial-operations), four releases for agents: the Bitwave CLI, Bitwave MCP, Bitwave Data and Analytics, and KubeClaw. "We believe agents won't interact with accounting software the same way humans do," the CEO said.

## How it works

The platform connects to more than 70 chains and DeFi ecosystems, to exchanges and custodians (Fireblocks, BitGo, Anchorage), and to four ERPs: QuickBooks, Xero, NetSuite and Sage Intacct. Rules categorize transactions and detect internal transfers. Lots are tracked by configurable methods, and entries post to the ERP.

The [CLI](https://cli.bitwave.io) is something else. It is AGPL-3.0, written in Go, with an MPL-2.0 accounting SDK, and installs with brew or npm. Locally, a workspace is a directory of plain-text journals that are compatible with hledger and ledger (and with Beancount through a shim). Each entry must balance before it is written. Change one TOML field and the same commands run against a cloud workspace; `bitwave migrate` pushes local books up. It can also sync wallets, send from them, and file expense reports for agents ([README](https://github.com/bitwave-io/bitwave-cli)).

## Data and access

The [developer docs](https://docs.bitwave.io) describe a REST API, with a Swagger file, authenticated by API key or a v2 OAuth-style token. It covers rate tables, invoices, customers, payment accounts and an asynchronous balance report that returns rows or CSV. The docs reviewed list no rate limits. Prices for the platform's three tiers (Startups, Enterprises, Institutions) are by quote only ([pricing](https://www.bitwave.io/pricing)). Only the Institutions tier adds BigQuery and Snowflake integrations and advanced 1099-DA reporting.

## Agents

The [MCP server](https://www.bitwave.io/blog/bitwave-mcp-for-agentic-finance) covers a focused first set of tools: view transactions, review and add wallets, categorize, filter and run balance reports, after a sign-in. Bitwave's own [market map](https://www.bitwave.io/blog/mcp-accounting-market-map) puts itself as the one crypto vendor with read and write access, where the AI drafts and "a person must approve before anything posts". The CLI is less strict. It offers `--dry-run` on wallet sends and asks agents to confirm irreversible actions, but that is guidance in the README, not an enforced control. KubeClaw runs agents in your own Kubernetes with tool controls and "approval workflows" whose mechanics are not documented.

## Customers

It sells to crypto startups, enterprises and institutions such as exchanges and custodians. The customer quotes on its site credit it with faster closes but give no figures. Independent review volume is thin: Trustpilot is unclaimed with no reviews, and the G2 page could not be reached to confirm a score. The CLI repo was new and quiet on 2026-10-10: 81 commits, 0 stars, 3 forks.

## History

Since 2025 Bitwave has moved from subledger toward payments (BVNK in July 2025, Ripple RLUSD in March 2026, Hedera and Canton in April 2026, reported by [PYMNTS](https://www.pymnts.com/?p=3996942)). It then moved into agents with the July 2026 launch, plus a Rillet integration and a DTCC working group.

## For ownpurse

Of the vendors in this survey, Bitwave is the closest in framing: local, open source, agent-first, with an MCP for people and a CLI for automation. Two differences matter. The local books are journals the owner can edit, not an append-only, hash-chained record of what each source said, so a correction rewrites history. And the local tool is a way into a hosted workspace. Approval at Bitwave is a property of the MCP flow and of agents following advice. In ownpurse it is the only way a write leaves. Worth borrowing: diffable books compatible with hledger and ledger, one switch between local and shared mode, and the agent notes in the README. Its exports and journals are also a natural source for a crypto connector.
