---
{
 "slug": "bitwave-agentic",
 "name": "Bitwave Agentic",
 "url": "https://www.bitwave.io/blog/bitwave-mcp-for-agentic-finance",
 "segment": "agent-native",
 "status": "active",
 "founded": "2018 (Bitwave, legal name BitAlpha, Inc.); the agentic CLI repository on 23 June 2026",
 "hq": "Fully remote, 7+ countries (Bitwave's about page); legal entity BitAlpha, Inc.",
 "backing": "Venture-backed private company; amounts and investors not confirmed from a primary source for this dossier.",
 "tagline": "Bitwave's agent line: an open-source, agent-first double-entry CLI over local plain-text journals, plus an MCP server and SDK into Bitwave's hosted crypto accounting platform.",
 "scope": "The CLI (AGPL-3.0, Go) keeps hledger/ledger-compatible journal files in a local directory, balance-checks every entry, syncs EVM wallets (Ethereum and Base) into pending entries and can sign and broadcast payments; a flag moves the same workspace to Bitwave's hosted ledger. Organization commands and the MCP server drive the Bitwave platform itself: wallets, transaction search and categorization, rules, imports, reports. It is not a multi-provider record of fiat books, and the platform behind it is closed and sold through sales.",
 "domains": [
  "crypto",
  "ledger",
  "reporting",
  "spend",
  "integration"
 ],
 "customers_served": [
  "developers",
  "mid-market",
  "enterprise",
  "accountants"
 ],
 "spectrum": {
  "custody": {
   "score": 3,
   "note": "Local mode keeps the books as plain-text files you hold; the platform and cloud mode hold them on Bitwave's servers."
  },
  "locality": {
   "score": 3,
   "note": "The CLI runs fully on your machine in local mode; the MCP and platform features are hosted."
  },
  "history": {
   "score": 2,
   "note": "Plain text you can keep in git, so history is as good as your commits; entries are editable files with no chain."
  },
  "openness": {
   "score": 3,
   "note": "CLI AGPL-3.0, accounting and wallet SDKs MPL-2.0, KubeClaw AGPL-3.0; the platform is closed."
  },
  "breadth": {
   "score": 2,
   "note": "Crypto wallets, journals and expense reports locally; the platform adds ERP sync, but fiat sources are not ingested."
  },
  "autonomy": {
   "score": 3,
   "note": "Local entries are written directly; platform writes need --yes, which an agent can pass itself; wallet sends broadcast."
  },
  "agent_readiness": {
   "score": 4,
   "note": "Built for agents: non-interactive, parseable output, --json envelopes, dry-run, operation catalog with JSON schemas, agent tokens, MCP, agent docs."
  },
  "integration_ease": {
   "score": 3,
   "note": "brew, npm, curl or go install for the CLI; the platform and MCP need an organization and possibly early access."
  },
  "market": {
   "score": 3,
   "note": "Platform sold to crypto-native startups, enterprises and institutions; the CLI reaches developers and agents."
  }
 },
 "api": {
  "public": true,
  "style": "CLI and Go SDK (operation catalog with JSON schemas), MCP server, and the Bitwave platform REST API",
  "auth": "OAuth PKCE browser login, BITWAVE_AGENT_TOKEN for remote agents, bearer tokens for CI; platform API uses org-scoped client id and key exchanged for 1-hour JWTs",
  "limits": "Not published; hosted SDK results are bounded to 1 MiB per output stream",
  "cost": "Local CLI is free; platform pricing is 'talk to sales' on every tier",
  "export": "Local journals are plain text, readable by hledger, ledger and (with a shim) beancount; je export; platform transaction export and balance reports as CSV",
  "notes": "The CLI collects anonymous usage telemetry (command and flag names only), opt-out with BITWAVE_TELEMETRY=0 or DO_NOT_TRACK=1."
 },
 "pricing": {
  "model": "Open-source CLI free; hosted platform quoted by sales in three tiers (Startups, Enterprises, Institutions)",
  "entry": "Free (local CLI)",
  "top": "Institutions tier, quoted by sales",
  "notes": "No prices are published; the MCP is offered through early access or a demo."
 },
 "agents": {
  "mcp": "official",
  "ai_features": "Bitwave CLI, Bitwave MCP, Data and Analytics for agents, KubeClaw (open-source Kubernetes operator for running agents with approval workflows and restricted tools), open-source wallet and accounting SDKs",
  "agent_writes": "Yes. Locally, journal entries, imports and wallet syncs write files directly, and wallet send broadcasts an on-chain transaction. On the platform, agents can add wallets, categorize transactions, create rules and run imports",
  "approval": "Partial. Platform mutations require --yes and offer --dry-run, but the agent can pass the flag; the README asks agents to confirm with the user before irreversible actions. Bitwave's own market map says its MCP lets an AI propose changes that a person approves; the mechanism is not documented. KubeClaw adds approval workflows for agents run in your cluster",
  "notes": "Agent guidance is written into the README ('Agents, please read'), including when to escalate and never silently fixing imbalances."
 },
 "integration": {
  "scope": "Local: plain-text journals, EVM wallets (Ethereum, Base). Platform: 50+ blockchains named on the pricing page, ERPs named on the home page (QuickBooks, NetSuite, Sage Intacct, Xero), Rillet integration, BigQuery and Snowflake for institutions",
  "notes": "Provider authorization stays in the Bitwave web app. The CLI's own docs separate cloud mode (a hosted journal service) from the Bitwave platform, two different APIs."
 },
 "preference": {
  "scale": "bitwave-cli: 0 stars and 3 forks on GitHub, 11 releases from v0.2.0 (18 July 2026) to v0.6.1 (24 Sept 2026). Platform customer logos include OpenSea, Coinbase, Blockdaemon, Aptos and DTCC, relationship unstated",
  "ratings": "G2 High Performer badge (Winter 2024) shown on Bitwave's site; no numeric rating confirmed",
  "sentiment": "No independent reviews of the agent line were found. Bitwave's platform is known in crypto accounting; the CLI is new, pre-1.0 and has almost no public adoption yet."
 },
 "timeline": [
  {
   "date": "2018",
   "event": "Bitwave founded (BitAlpha, Inc.)"
  },
  {
   "date": "2026-06-23",
   "event": "bitwave-cli and bitwave-accounting-sdk repositories created"
  },
  {
   "date": "2026-06-24",
   "event": "KubeClaw repository created"
  },
  {
   "date": "2026-07-18",
   "event": "First published CLI release, v0.2.0"
  },
  {
   "date": "2026-08-21",
   "event": "CLI v0.4.0"
  },
  {
   "date": "2026-09-08",
   "event": "Bitwave publishes its MCP accounting market map"
  },
  {
   "date": "2026-09-18",
   "event": "CLI v0.5.0; accounting SDK updated"
  },
  {
   "date": "2026-09-21",
   "event": "Agentic, CLI and MCP announcement posts last updated"
  },
  {
   "date": "2026-09-24",
   "event": "CLI v0.6.1 and Homebrew tap"
  },
  {
   "date": "2026-10-03",
   "event": "Latest push to bitwave-cli"
  }
 ],
 "relation": "competitor",
 "relation_note": "The nearest framing to ownpurse: local, open, agent-first books with a CLI built for agents. It differs in three ways: it is crypto first, it keeps normalized journal entries rather than raw source facts with provenance, and its path leads to Bitwave's hosted platform.",
 "borrow": "The agent contract in the README (non-interactive, parseable, balance-checked, escalate rather than fix); the side-effect table ranking commands by reversibility; dry-run plus explicit confirmation; one SDK operation catalog with JSON schemas behind the CLI and MCP; idempotent wallet sync keyed on the transaction hash; pending entries that clear later.",
 "avoid": "A --yes flag the agent can set as the only gate; editable journal files as the record, with history left to git; two cloud services under one 'cloud' word; usage telemetry on by default.",
 "sources": [
  {
   "title": "Introducing Bitwave MCP",
   "url": "https://www.bitwave.io/blog/bitwave-mcp-for-agentic-finance"
  },
  {
   "title": "Introducing Bitwave Agentic",
   "url": "https://www.bitwave.io/blog/introducing-bitwave-agentic"
  },
  {
   "title": "Introducing Bitwave CLI",
   "url": "https://www.bitwave.io/blog/bitwave-ai-command-line-interface"
  },
  {
   "title": "Bitwave AI Week",
   "url": "https://www.bitwave.io/blog/bitwave-ai-features"
  },
  {
   "title": "MCP accounting has arrived: the market map",
   "url": "https://www.bitwave.io/blog/mcp-accounting-market-map"
  },
  {
   "title": "bitwave-io/bitwave-cli on GitHub",
   "url": "https://github.com/bitwave-io/bitwave-cli"
  },
  {
   "title": "bitwave-io/bitwave-accounting-sdk",
   "url": "https://github.com/bitwave-io/bitwave-accounting-sdk"
  },
  {
   "title": "bitwave-io/kube-claw",
   "url": "https://github.com/bitwave-io/kube-claw"
  },
  {
   "title": "Bitwave pricing",
   "url": "https://www.bitwave.io/pricing"
  },
  {
   "title": "About Bitwave",
   "url": "https://www.bitwave.io/about"
  },
  {
   "title": "Bitwave developer docs",
   "url": "https://docs.bitwave.io/"
  }
 ],
 "unverified": [
  "Bitwave's funding rounds, amounts and investors (not stated on its site; press not reachable for this review)",
  "The exact dates of AI Week and of the MCP launch; posts show only a September 2026 modified date",
  "How the MCP's propose-and-approve flow works, if it exists beyond connection authorization",
  "Whether the MCP requires early access or is generally available, and what it costs",
  "Numeric G2 or Capterra ratings",
  "HQ beyond 'fully remote'"
 ],
 "researched": "2026-10-10"
}
---

## What it is

Bitwave is a crypto accounting platform founded in 2018. In mid-2026 it launched an agent line it calls Bitwave Agentic, during a five-day ['AI Week'](https://www.bitwave.io/blog/bitwave-ai-features): KubeClaw (an open-source Kubernetes operator for running agents), the [Bitwave CLI](https://github.com/bitwave-io/bitwave-cli), the [Bitwave MCP](https://www.bitwave.io/blog/bitwave-mcp-for-agentic-finance), Data and Analytics for agents, and open-source wallet and accounting libraries. Of everything in this landscape, it is the closest in spirit to ownpurse.

## How it works

The CLI is a Go binary (AGPL-3.0) that calls itself 'an agent-first accounting platform'. `bitwave init` makes a directory with a `.bitwave.toml` and a `default.journal`; entries are plain text in ledger format, and every one must balance before it is written. The output is readable by hledger and ledger, and the CLI imports their files and beancount's. A wallet extension generates or watches EVM wallets (Ethereum and Base), syncs on-chain history as pending entries that clear after confirmations, deduplicates on the transaction hash, and can sign and broadcast payments with a dry run first.

The same commands work against a hosted 'workspace ledger' when you flip one field, and `bitwave migrate` pushes a local workspace up. A separate family of `org` commands drives the Bitwave platform proper: wallets across many chains, transaction search and categorization, rules, CSV imports, balance and inventory reports, users and roles. The [SDK](https://github.com/bitwave-io/bitwave-cli) exposes one operation catalog with JSON schemas that the CLI and the MCP both call.

## Data and access

In local mode you hold your books as text files and need no account. Everything platform-side is hosted and priced by sales; the [pricing page](https://www.bitwave.io/pricing) lists Startups, Enterprises and Institutions tiers with no figures. The platform API uses org-scoped client keys exchanged for one-hour tokens. The CLI sends anonymous usage telemetry (command and flag names) unless you opt out.

## Agents

This is where Bitwave is strongest. The README has a section headed 'Agents, please read': a table of commands by side effect and reversibility, instructions to confirm before irreversible actions, and a list of when to escalate, ending with 'never silently fix inconsistencies'. Platform writes need `--yes` and offer `--dry-run`, with versioned JSON envelopes. Bitwave's own [market map](https://www.bitwave.io/blog/mcp-accounting-market-map) says its MCP lets an AI propose changes that a person approves, though the mechanism is not documented, and the `--yes` flag is one the agent can pass itself.

## Customers

The platform's site shows logos such as OpenSea, Coinbase, Blockdaemon, Aptos and DTCC, without saying what each relationship is. The CLI is new: 11 releases between 18 July and 24 September 2026, no stars and three forks on 10 October 2026, and the README calls it pre-1.0.

## History

Bitwave spent eight years on crypto subledgers that sync into ERPs (QuickBooks, NetSuite, Sage Intacct, Xero). Its agent line reads as a bet that agents will spend money and need books, starting with their own: the README's headline workflow is an agent tracking its API spend.

## For ownpurse

Bitwave reached several of ownpurse's conclusions independently: local files, open source, a CLI designed for agents, balance checks, escalation over silent fixes. The differences are where ownpurse's case rests. Bitwave keeps normalized journal entries, not what each source reported with its provenance, so you cannot rebuild and check against the source to the cent. Its history is a git habit, not an append-only, hash-chained log. Its write gate is a flag rather than a sealed package a person approves. It is crypto first, and its path leads to a hosted platform. Borrow the agent contract and the reversibility table; keep the record raw, chained and owner-held.
