---
{
 "slug": "bitwave-cli",
 "name": "Bitwave CLI",
 "url": "https://www.bitwave.io/blog/bitwave-mcp-for-agentic-finance",
 "segment": "agent-native",
 "status": "active",
 "founded": "Repository created 23 June 2026; first tagged release v0.2.0 on 18 July 2026. Bitwave (BitAlpha, Inc.) founded 2018.",
 "hq": "Not stated; Bitwave describes a fully remote team in 7+ countries",
 "backing": "Made by Bitwave (legal name BitAlpha, Inc.), a private crypto accounting company founded in 2018 by Pat White (CEO) and Amy Kalnoki (COO). Funding is not stated on its site.",
 "tagline": "An open-source, agent-first double-entry CLI that keeps books in plain-text journal files on your machine, with an optional cloud workspace and commands for the hosted Bitwave platform.",
 "scope": "The CLI writes balance-checked journal entries to ledger/hledger-format files, imports and exports ledger, hledger and beancount, manages EVM wallets (Ethereum and Base), syncs on-chain history, signs and broadcasts payments, and produces expense reports. It can move a workspace to Bitwave's hosted workspace ledger and, separately, drive a Bitwave platform organisation (transactions, categorisation rules, imports, reports). It does not connect to Xero, QuickBooks or banks from the local side; provider connections stay in the Bitwave web app.",
 "domains": [
  "ledger",
  "crypto",
  "payments",
  "spend",
  "reporting",
  "integration"
 ],
 "customers_served": [
  "developers",
  "smb",
  "enterprise"
 ],
 "spectrum": {
  "custody": {
   "score": 3,
   "note": "In local mode the books are plain-text files you hold and any plain-text accounting tool can read; cloud mode and platform commands put data in Bitwave's hosted services."
  },
  "locality": {
   "score": 4,
   "note": "Runs on your own machine against a directory of journal files; cloud is a TOML switch away, not required."
  },
  "history": {
   "score": 2,
   "note": "Plain text in git gives every version if you commit it; the tool itself edits files in place with no append-only rule or hash chain."
  },
  "openness": {
   "score": 4,
   "note": "CLI under AGPL-3.0, accounting and wallet SDKs under MPL-2.0; the Bitwave platform behind the org commands is closed."
  },
  "breadth": {
   "score": 2,
   "note": "Journal entries, EVM wallets, CSV imports and expense reports locally; the wider reach (exchanges, custodians, NetSuite, QuickBooks, Xero) belongs to the hosted platform."
  },
  "autonomy": {
   "score": 3,
   "note": "Agents write entries directly with only a balance check, and 'wallets send' signs and broadcasts; the README asks agents to confirm irreversible actions, and platform writes need a --yes flag the agent can pass."
  },
  "agent_readiness": {
   "score": 4,
   "note": "Non-interactive commands, stable exit codes, JSON output on many commands, dry runs, an agent token, a Go SDK with an operation catalogue and schemas, an 'Agents, please read' section, and a companion MCP server."
  },
  "integration_ease": {
   "score": 3,
   "note": "Install by brew, npm, Go or script and start in a directory; the hosted platform needs an organisation and pricing is by demo."
  },
  "market": {
   "score": 2,
   "note": "The CLI reaches individuals, developers and agent builders; the platform it feeds sells to crypto companies and institutions."
  }
 },
 "api": {
  "public": true,
  "style": "CLI and Go SDK (sdk.Operations catalogue with JSON schemas); the hosted workspace ledger and platform are REST/JSON under api.bitwave.io; a Bitwave MCP server for MCP-compatible apps",
  "auth": "Local mode needs none. Cloud: OAuth PKCE browser login, BITWAVE_AGENT_TOKEN, or BITWAVE_TOKEN. Platform: org-scoped client id and key (ReadAll or WriteAll) exchanged for a 1-hour JWT, no refresh token",
  "limits": "Rate limits not published; hosted SDK results bounded to 1 MiB per output stream",
  "cost": "The CLI is free (AGPL-3.0). Cloud workspaces, MCP and the Bitwave platform are by early access or sales; no prices published",
  "export": "Local: the journal files themselves, 'je export' in ledger format, CSV of postings, expense reports in CSV, JSON, HTML and QIF. Platform: transaction export and balance and actions reports as CSV",
  "notes": "Anonymous usage telemetry (command shape, flag names, never values) is on by default with a one-time notice and an opt-out. The repo's own docs note a bug where client-credential login could delete credentials.json on expiry (state at time of reading)."
 },
 "pricing": {
  "model": "Open-source CLI free; Bitwave platform priced by sales in three tiers (Startups, Enterprises, Institutions)",
  "entry": "Free (the CLI, local mode)",
  "top": "Institutions tier, price on request",
  "notes": "No dollar amounts are published for any hosted tier, the MCP server or cloud workspaces (pricing page read 2026-10-10)."
 },
 "agents": {
  "mcp": "official",
  "ai_features": "Bitwave Agentic: the CLI, a Bitwave MCP server, Data and Analytics (SQL-style queries over eight years of blockchain data), KubeClaw (an open-source Kubernetes operator for running governed agents) and wallet and agent-accounting SDKs, launched together as 'Bitwave AI Week'",
  "agent_writes": "Yes, directly. Local journal writes, wallet syncs and signed on-chain sends; platform transaction categorisation, rules, imports and wallet additions",
  "approval": "No gate on local writes beyond the balance check. Platform mutations require --yes and offer --dry-run to print the exact request; the README tells agents to confirm with the user before any irreversible action such as broadcasting a payment. Sharing is read-only by default with write as a server-side capability. KubeClaw advertises approval workflows for hosted agents",
  "notes": "The guidance to agents is careful (record before you spend, never invent a balancing posting), but approval is a convention the agent is asked to follow, not a sealed step a person signs."
 },
 "integration": {
  "scope": "Locally: ledger, hledger and beancount files; EVM chains Ethereum and Base; CSV. Through the platform: Bitcoin, Ethereum and L2s, Solana and others; Fireblocks, BitGo, Anchorage, NYDIG; Coinbase, Kraken, Binance and other exchanges; QuickBooks, NetSuite, Sage Intacct and Xero; Rillet",
  "notes": "The local side is a clean plain-text-accounting citizen with a compatibility suite against real hledger, ledger and bean-check binaries. The provider integrations are the hosted product's, not the CLI's."
 },
 "preference": {
  "scale": "No customer count published. Logos on bitwave.io include Coinbase, OpenSea, Blockdaemon and DTCC. The CLI repository had 0 GitHub stars and 11 releases (v0.6.1 on 24 September 2026) when read on 2026-10-10.",
  "ratings": "Bitwave shows a G2 High Performer badge (Winter 2024) and claims to be the only enterprise crypto accounting platform on G2's Winter Accounting Grid; no rating counts confirmed. No reviews of the CLI itself found.",
  "sentiment": "No independent user opinion of the CLI found; it is three months old."
 },
 "timeline": [
  {
   "date": "2018",
   "event": "Bitwave founded by Pat White and Amy Kalnoki"
  },
  {
   "date": "2026-06-23",
   "event": "bitwave-cli (AGPL-3.0) and the wallet and accounting SDKs (MPL-2.0) created on GitHub"
  },
  {
   "date": "2026-07-18",
   "event": "First tagged releases (v0.2.0 to v0.2.2) and a Homebrew tap"
  },
  {
   "date": "2026 (undated)",
   "event": "Bitwave AI Week: KubeClaw, the CLI, the MCP server, Data and Analytics and open-source libraries"
  },
  {
   "date": "2026-08-21",
   "event": "v0.4.0"
  },
  {
   "date": "2026-09-24",
   "event": "v0.6.1, with organisation commands for the hosted platform"
  }
 ],
 "relation": "kin",
 "relation_note": "The nearest framing to ownpurse: open source, local, agent-first, double-entry, with an honest 'Agents, please read'. It differs in three ways: it keeps edited journal files rather than an append-only, hash-chained record; it writes directly rather than through approved packages; and its local books are a funnel to a hosted, crypto-first platform rather than a record beside the owner's existing ledgers.",
 "borrow": "An agent section in the README that states rules; dry runs that print the exact request; --yes on every remote mutation; a reversibility table by command class; interop with hledger, ledger and beancount proved by a test suite against the real binaries; on-chain hash as the dedupe key; telemetry that records command shape and never values.",
 "avoid": "Confirmation as an instruction to the agent instead of a step a person takes; history that depends on the user remembering to commit; editing files in place; letting 'cloud' mean two different hosted services.",
 "sources": [
  {
   "title": "Introducing Bitwave MCP",
   "url": "https://www.bitwave.io/blog/bitwave-mcp-for-agentic-finance"
  },
  {
   "title": "Introducing Bitwave CLI",
   "url": "https://www.bitwave.io/blog/bitwave-ai-command-line-interface"
  },
  {
   "title": "Introducing Bitwave Agentic",
   "url": "https://www.bitwave.io/blog/introducing-bitwave-agentic"
  },
  {
   "title": "Bitwave AI Week",
   "url": "https://www.bitwave.io/blog/bitwave-ai-features"
  },
  {
   "title": "bitwave-cli on GitHub (README, docs, releases)",
   "url": "https://github.com/bitwave-io/bitwave-cli"
  },
  {
   "title": "bitwave-accounting-sdk on GitHub",
   "url": "https://github.com/bitwave-io/bitwave-accounting-sdk"
  },
  {
   "title": "Bitwave home page",
   "url": "https://www.bitwave.io/"
  },
  {
   "title": "Bitwave pricing",
   "url": "https://www.bitwave.io/pricing"
  },
  {
   "title": "Bitwave about",
   "url": "https://www.bitwave.io/about"
  }
 ],
 "unverified": [
  "Bitwave's funding history (not stated on its site; search was unavailable)",
  "Bitwave's headquarters city",
  "Exact publication dates of the AI Week posts (pages show only a 2026-09-21 modification date); the earlier survey's 'July 2026' matches the first CLI release",
  "Bitwave G2 rating and review counts",
  "Whether the MCP server and cloud workspaces are charged separately",
  "Whether the documented client-credential login bug has been fixed in later releases"
 ],
 "researched": "2026-10-10"
}
---

## What it is

Bitwave CLI is an open-source command-line tool from [Bitwave](https://www.bitwave.io/), a crypto accounting company founded in 2018. Its [README](https://github.com/bitwave-io/bitwave-cli) calls it "agent-first accounting, local or cloud": AI agents and people keep double-entry books through one CLI, every command is non-interactive, every output is parseable, and every entry is balance-checked before it is written. The repository was created on 23 June 2026 under AGPL-3.0 and first released on 18 July 2026; v0.6.1 shipped on 24 September 2026.

## How it works

In local mode a workspace is a directory: a `.bitwave.toml`, journal files in ledger/hledger format, account and price declarations, and wallet keystores. You can diff it, commit it and pipe `bitwave je export` into hledger. It imports ledger, hledger and beancount, keeping cost-basis annotations, and a compatibility suite in the MPL-2.0 [accounting SDK](https://github.com/bitwave-io/bitwave-accounting-sdk) checks the output against the real tools.

The crypto extension manages EVM wallets on Ethereum and Base, syncs on-chain history as pending entries that clear after confirmations (deduplicated on the transaction hash), and signs and broadcasts payments, writing the entry as it goes. Cloud mode moves the same journals to Bitwave's hosted workspace ledger with `bitwave migrate`. A separate family of `org` commands drives the hosted Bitwave platform: transaction review, categorisation rules, CSV imports, balance and transaction reports.

## Data and access

Locally, the data is the files, and you hold them. There is no hash chain or append-only rule: an entry is corrected by editing the file or reverting in git, so history is as good as your commits. Platform access uses org-scoped client keys with ReadAll or WriteAll, exchanged for one-hour tokens. The CLI is free; the platform, the MCP server and cloud workspaces are sold by demo, with no published prices. The CLI sends anonymous telemetry (command and flag names, never values) unless you opt out.

## Agents

The README has a section headed "Agents, please read" with real rules: record before you spend, never invent a counter-posting to make an entry balance, start a session with `bitwave stats`. It includes a table of side effects by command and says to confirm with the user before anything irreversible. Platform writes require `--yes` and offer `--dry-run`. The [MCP server](https://www.bitwave.io/blog/bitwave-mcp-for-agentic-finance) lets MCP apps view transactions, add wallets and categorise. All of this shipped in a "[Bitwave AI Week](https://www.bitwave.io/blog/bitwave-ai-features)" alongside KubeClaw, an open-source operator for running governed agents. The gate is a rule the agent is asked to follow; nothing makes a person sign.

## Customers

Bitwave sells to crypto companies and institutions; its site shows logos such as Coinbase, OpenSea and DTCC and three tiers priced through sales. The CLI had no GitHub stars when read, and no independent reviews exist yet.

## History

Bitwave has built crypto subledgers since 2018, with ERP integrations to QuickBooks, NetSuite, Sage Intacct and Xero. In mid 2026 it moved into agent tooling with this CLI, an MCP server and open SDKs, and its blog now publishes a plain-text accounting primer and an MCP accounting market map.

## For ownpurse

This is the closest kin. The local, open, agent-first shape is the same, and the README's agent rules, dry runs, reversibility table and interop suite are worth copying. The differences are the ones ownpurse is built on. Bitwave keeps edited text files; ownpurse keeps an append-only, hash-chained log in which nothing is overwritten. Bitwave writes when the agent decides; ownpurse writes to a source only after a person approves a sealed package. And Bitwave's local books lead to its hosted crypto platform, while ownpurse sits beside the owner's existing ledgers (Xero first) and checks them to the cent.
