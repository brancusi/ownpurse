---
{
 "slug": "rotki",
 "name": "rotki",
 "url": "https://rotki.com",
 "segment": "crypto-accounting",
 "status": "active",
 "founded": "End of 2017 (repository created 5 March 2018)",
 "hq": "Berlin, Germany (Rotki Solutions GmbH)",
 "backing": "Bootstrapped. Revenue has come from integration grants, donation rounds (Gitcoin, Giveth) and premium subscriptions, per the team's own 13 October 2025 post; no venture funding found. Company: Rotki Solutions GmbH. Lead author Lefteris Karapetsas.",
 "tagline": "An open-source, local-first crypto portfolio and accounting app that keeps an encrypted database on your machine.",
 "scope": "rotki tracks balances and decodes history across EVM chains, Bitcoin, Substrate and Solana and 20-odd exchanges, matches movements between accounts, and produces profit and loss reports under FIFO, LIFO, HIFO or ACB with configurable tax rules. It runs as a desktop app or Docker container with an encrypted SQLite (SQLCipher) database you hold. It is not a general ledger: fiat books, invoices and payroll are out of scope, and other assets enter only as manual balances.",
 "domains": [
  "crypto",
  "personal",
  "tax",
  "reporting"
 ],
 "customers_served": [
  "consumer",
  "freelancer",
  "developers"
 ],
 "spectrum": {
  "custody": {
   "score": 4,
   "note": "The authoritative database is an encrypted file on your device; premium sync uploads only a copy encrypted with your password."
  },
  "locality": {
   "score": 4,
   "note": "Desktop app for Windows, macOS and Linux, or a self-hosted Docker image; no account needed to use it."
  },
  "history": {
   "score": 2,
   "note": "Keeps every decoded event and balance snapshots, and marks edited events as customized, but edits and redecodes change rows in place and there is no hash chain or as-of query."
  },
  "openness": {
   "score": 4,
   "note": "AGPL-3.0 on GitHub, with a commercial license offered; documented REST API and direct SQLCipher access."
  },
  "breadth": {
   "score": 1,
   "note": "One domain, crypto, across many chains, exchanges and protocols; other assets only as manual balances."
  },
  "autonomy": {
   "score": 2,
   "note": "Pulls, decodes and categorizes unattended and computes P&L; it writes nothing to external books."
  },
  "agent_readiness": {
   "score": 3,
   "note": "A local MCP server (since v1.44.0, August 2026) with read-only SQL and privacy masking, plus a local REST API; gated to paid tiers and no CLI."
  },
  "integration_ease": {
   "score": 3,
   "note": "Documented local REST API, generic CSV import formats and documented database access; the API is built for its own front end and not declared stable."
  },
  "market": {
   "score": 0,
   "note": "Individual crypto holders and power users; a Custom tier exists for businesses and high-net-worth users."
  }
 },
 "api": {
  "public": true,
  "style": "Local HTTP REST API (Python backend on 127.0.0.1:5042 under /api/1/, plus a Rust service, Colibri); local MCP over HTTP on 127.0.0.1:4445/mcp.",
  "auth": "A local user session (log in with username and password through the API); Docker MCP connections use a bearer token that expires at most seven days after login. The desktop MCP endpoint needs no token.",
  "limits": "No rate limits of its own; outward queries are limited by the chain indexers and exchanges it calls (it recommends a free Etherscan key).",
  "cost": "The software and its API are free. Plan limits apply to data: Free shows 1,000 history events; Supporter €9/month 3,000; Basic €25/month 30,000; Advanced €45/month 100,000 (yearly saves two months). MCP needs Basic or higher.",
  "export": "CSV export of history events and of P&L reports; CSV export and import of blockchain accounts; balance snapshot CSVs; the full encrypted SQLite database, readable with SQLCipher and your password.",
  "notes": "The REST API documentation says the API exists for the Electron front end or a browser; it does not promise stability to third parties. Docs publish llms.txt and llms-full.txt."
 },
 "pricing": {
  "model": "Open-source app with subscription tiers that raise event, device and analytics limits.",
  "entry": "Free: 1,000 most recent history events, P&L over the first 1,000 events, no backup or sync, two-week dashboard graph.",
  "top": "Advanced at €45/month (100,000 events, four devices, staking tracking up to 384 ETH); Custom by quote for businesses and high-net-worth users.",
  "notes": "Tiers introduced in a 13 October 2025 post, the first price change since launch; existing Premium subscribers moved to Basic at €25/month from 10 November 2025. Prices read from rotki.com on 2026-10-10; VAT included per the post."
 },
 "agents": {
  "mcp": "official",
  "ai_features": "A local MCP server lets AI assistants analyze history events and balances with read-only SQL, look up assets and cached historical prices, and use rotki's event taxonomy. Three privacy modes: Strict (anonymous session-stable identifiers, no free text), Balanced (default; masks addresses, hashes and your own labels) and Raw.",
  "agent_writes": "No. The MCP tools are read-only; the REST API can write (settings, credentials, accounts) but is meant for rotki's own front end.",
  "approval": "Not applicable for MCP, which cannot write. You start and stop the server and choose the privacy level yourself.",
  "notes": "Shipped in v1.44.0 on 21 August 2026. Documented clients: Claude Code, OpenCode, and ChatGPT through an OpenAI Secure MCP Tunnel. On Free and Supporter tiers every tool except info returns premium_required."
 },
 "integration": {
  "scope": "Claims 190+ integrations (exchanges, blockchains, DeFi protocols). Chains: Ethereum, Optimism, Polygon PoS, Arbitrum One, Base, Gnosis, Scroll, BNB Smart Chain, Avalanche, zkSync Lite, Bitcoin, Bitcoin Cash, Polkadot, Kusama, Solana. 23 exchanges by API key (Kraken, Binance, Coinbase, Coinbase Prime, Gemini, Bitstamp, OKX, Bybit and more). CSV imports from Binance, Bitstamp, Kucoin, Nexo, Uphold, Blockpit, CoinLedger, cointracking.info, Bitcoin.tax and others, plus generic trade and event formats. Monerium and Gnosis Pay enrichment.",
  "notes": "Exchange keys are read-only. A small team maintains a long tail of chains and protocols, which the October 2025 pricing post names as its main cost."
 },
 "preference": {
  "scale": "4,040 GitHub stars and 771 forks (GitHub API, 2026-10-10); the site claims 200+ contributors. No user or paying-customer counts published.",
  "ratings": "Trustpilot 3.3 from a single review (July 2023). No G2 or Capterra listing found.",
  "sentiment": "Users quoted on its site praise privacy, open code, breadth of DeFi decoding and the tax report. The one Trustpilot reviewer found it too complicated, with confusing errors, sync trouble, basic charts and missing exchanges, and moved to CoinTracking. The docs themselves note limits, such as no custom cost-basis override for assets already held."
 },
 "timeline": [
  {
   "date": "2017-12",
   "event": "Founded at the end of 2017, per the team; name short for Rotkehlchen, German for the European robin"
  },
  {
   "date": "2018-03-05",
   "event": "GitHub repository created"
  },
  {
   "date": "2024-10-09",
   "event": "Post on why open source matters for trust in portfolio tools"
  },
  {
   "date": "2025-10-13",
   "event": "Announces tiered pricing, the first change since launch; Basic at €25/month"
  },
  {
   "date": "2025-11-10",
   "event": "Existing Premium subscribers move to Basic"
  },
  {
   "date": "2026-05-22",
   "event": "Post on supply-chain defences: locked dependencies, pinned actions, build attestations"
  },
  {
   "date": "2026-08-21",
   "event": "v1.44.0 ships a local, read-only MCP server with privacy modes, bridge matching and a Docker overhaul"
  },
  {
   "date": "2026-10-02",
   "event": "v1.44.1 released"
  }
 ],
 "relation": "kin",
 "relation_note": "rotki is the closest thing to ownpurse in crypto: open source, on your machine, an encrypted SQLite file you hold, and a local read-only MCP server. It covers crypto deeply where ownpurse covers books broadly, so it is also a natural source: an ownpurse connector could read rotki's decoded events instead of decoding chains itself.",
 "borrow": "The privacy modes on the MCP server (Strict, Balanced, Raw) as a model for what ownpurse shows an agent; a published event taxonomy that agents can use to interpret activity; generic CSV formats for trades and events; read-only exchange keys; checksummed, attested releases; docs published as llms.txt.",
 "avoid": "Editing and redecoding events in place, which loses what the record said before; gating agent access behind the paid tier when local reads cost the vendor nothing; a REST API built only for its own front end with no stability promise; a global database (assets, prices) that sync does not carry, which makes moving machines a manual job.",
 "sources": [
  {
   "title": "rotki on GitHub (README, license)",
   "url": "https://github.com/rotki/rotki"
  },
  {
   "title": "rotki plans and pricing",
   "url": "https://rotki.com/products"
  },
  {
   "title": "rotki home page",
   "url": "https://rotki.com/"
  },
  {
   "title": "rotki documentation",
   "url": "https://docs.rotki.com/"
  },
  {
   "title": "rotki docs: MCP",
   "url": "https://docs.rotki.com/usage-guides/mcp"
  },
  {
   "title": "rotki docs: MCP settings and privacy levels",
   "url": "https://docs.rotki.com/usage-guides/settings/mcp"
  },
  {
   "title": "rotki docs: database access",
   "url": "https://docs.rotki.com/usage-guides/advanced/database-access"
  },
  {
   "title": "rotki docs, full text for LLMs",
   "url": "https://docs.rotki.com/llms-full.txt"
  },
  {
   "title": "rotki REST API documentation",
   "url": "https://rotki.readthedocs.io/en/latest/api.html"
  },
  {
   "title": "rotki v1.44.0 release notes (21 August 2026)",
   "url": "https://github.com/rotki/rotki/releases/tag/v1.44.0"
  },
  {
   "title": "Why we're changing rotki's pricing after more than 7 years (13 October 2025)",
   "url": "https://blog.rotki.com/2025/10/13/rotki-tiers/"
  },
  {
   "title": "rotki blog",
   "url": "https://blog.rotki.com/"
  },
  {
   "title": "rotki FAQ",
   "url": "https://docs.rotki.com/faq"
  },
  {
   "title": "rotki on Trustpilot",
   "url": "https://www.trustpilot.com/review/rotki.com"
  },
  {
   "title": "Lefteris Karapetsas on GitHub",
   "url": "https://github.com/LefterisJP"
  }
 ],
 "unverified": [
  "Number of users or paying subscribers (not published)",
  "Team size (described only as small)",
  "Total grant and donation income",
  "The old Premium price before October 2025",
  "Yearly prices (only 'save 2 months' is stated)",
  "Whether 'Rotki Solutions GmbH' is registered in Berlin specifically; the founder's GitHub profile gives Berlin",
  "Date of the rename from Rotkehlchen to rotki"
 ],
 "researched": "2026-10-10"
}
---

## What it is

rotki is an open-source crypto portfolio tracker and accounting app that runs on your own machine. It describes itself as "a portfolio tracking, analytics, accounting and management application that protects your privacy" ([GitHub](https://github.com/rotki/rotki)). The code is AGPL-3.0, with a commercial license offered. Rotki Solutions GmbH in Berlin makes it, and its lead author, Lefteris Karapetsas, has the most commits by far. The name is short for Rotkehlchen, German for the European robin.

## How it works

You install the desktop app (Windows, macOS, Linux) or run the Docker image. You add wallet addresses on EVM chains, Bitcoin, Substrate or Solana, plus read-only exchange keys for 23 exchanges ([docs](https://docs.rotki.com/)). rotki pulls the transactions, decodes them into typed events using its own taxonomy, and matches deposits, withdrawals and bridge transfers between your accounts. It produces profit and loss reports under FIFO, LIFO, HIFO or ACB. The default rules follow German tax law, including tax-free sales after a one-year hold. Exchanges it cannot reach come in through CSV, including exports from CoinLedger, Blockpit and cointracking.info, or through generic trade and event formats.

Everything sits in a local SQLite database encrypted with SQLCipher and unlocked with your password. You can open it yourself with the `sqlcipher` CLI or DB Browser ([database access](https://docs.rotki.com/usage-guides/advanced/database-access)). Premium sync uploads a copy encrypted with your password before upload.

## Data and access

The Python backend serves a documented REST API on `127.0.0.1:5042`. A newer Rust service, Colibri, sits beside it ([API docs](https://rotki.readthedocs.io/en/latest/api.html)). The API exists for rotki's own front end and makes no stability promise to anyone else, but it is complete. History events and P&L reports export as CSV. The software is free. The [plans](https://rotki.com/products) cap data instead: Free shows the latest 1,000 events, Supporter is €9 a month for 3,000, Basic €25 for 30,000, and Advanced €45 for 100,000 and four devices. Custom is quoted for businesses. The tiers arrived in [October 2025](https://blog.rotki.com/2025/10/13/rotki-tiers/), the first price change since launch, because grants and donations (Gitcoin, Giveth) had proved too unpredictable for a small team.

## Agents

v1.44.0 (21 August 2026) shipped a [local MCP server](https://docs.rotki.com/usage-guides/mcp) on `127.0.0.1:4445/mcp`. Assistants can query events and balances with read-only SQL, look up assets and cached prices, and use the event taxonomy instead of guessing from raw exports ([release notes](https://github.com/rotki/rotki/releases/tag/v1.44.0)). Three [privacy modes](https://docs.rotki.com/usage-guides/settings/mcp) set what the assistant sees. Strict replaces identifiers with stable anonymous values and sends no free text. Balanced, the default, masks addresses, hashes and your own labels. Raw sends everything. The docs cover Claude Code, OpenCode, and ChatGPT through a secure tunnel. MCP needs the Basic tier: on Free every tool except `info` returns `premium_required`. Agents cannot write.

## Customers

rotki serves privacy-minded individuals and DeFi power users, not firms. It has 4,040 GitHub stars and 771 forks (2026-10-10), and the site claims 200+ contributors and 190+ integrations. There are almost no public reviews. The single Trustpilot review, from 2023, found it too complicated and switched to CoinTracking. Its strengths and its friction come from the same choice: it does everything locally, so you set up keys, indexers and backups yourself.

## History

The project started at the end of 2017 and the repository dates from March 2018. It ran on one subscription price for seven years, funded largely by grants and donation rounds. In 2025 to 2026 it professionalized: tiered pricing, a supply-chain hardening post (May 2026), checksummed and attested releases, and the MCP server. Releases are frequent: v1.44.1 came out on 2 October 2026.

## For ownpurse

rotki is kin. It proves that people will run a local, open-source finance tool that holds an encrypted file they own, and it reached agents through MCP with privacy masking before most hosted vendors did. The differences are where ownpurse is built. rotki is deep in one domain. It edits events in place, marking them "customized", so earlier states are lost. It keeps no hash chain, and its agent surface is read-only and paid. ownpurse could treat rotki as a crypto source, reading its decoded events and taxonomy instead of writing chain decoders, and record each read as facts. The privacy modes are worth copying almost as they are.
