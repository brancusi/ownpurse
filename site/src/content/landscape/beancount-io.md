---
{
 "slug": "beancount-io",
 "name": "Beancount.io",
 "url": "https://beancount.io",
 "segment": "plain-text-local",
 "status": "active",
 "founded": "2019 (site copyright 2019–2026; iOS app released 17 August 2020)",
 "hq": "United States [unverified]; operated by Stargately, Inc. per its Terms of Service",
 "backing": "Operated by Stargately, Inc.; AI requests routed through bex.co / BlockEden. No funding rounds found [unverified].",
 "tagline": "A hosted Beancount service where each ledger is a git repository, with a commercial MCP server that runs BQL queries, previews edits as dry runs and lands every write as a git commit.",
 "scope": "Hosts plain-text Beancount ledgers with Fava, git access over SSH, Plaid bank sync, AI import of statements and receipts, managed prices, budgets, mobile apps and an AI assistant; also ships bea, an open-source local CLI that works without an account. It is a ledger for personal, household, freelance and small-business books, not a layer that reads other accounting systems. The hosted service is closed and cloud-only; the format and tools around it are open.",
 "domains": [
  "ledger",
  "bank",
  "documents",
  "personal",
  "reporting",
  "crypto"
 ],
 "customers_served": [
  "consumer",
  "freelancer",
  "smb",
  "accountants",
  "developers"
 ],
 "spectrum": {
  "custody": {
   "score": 3,
   "note": "The hosted git repository is the working copy, but you can clone it with full history, push it anywhere and keep using open-source Beancount."
  },
  "locality": {
   "score": 2,
   "note": "The service and MCP are cloud only; the bea CLI and Android app work locally, and the format runs anywhere."
  },
  "history": {
   "score": 3,
   "note": "Every edit, import and AI write is a git commit you can diff and revert; not a fact-level, source-checked log, and history-rewrite rules on the server are not documented."
  },
  "openness": {
   "score": 3,
   "note": "Beancount (GPL-2.0), the bea CLI (MIT and GPL-2.0) and the mobile app (MIT) are open; the hosted platform is closed."
  },
  "breadth": {
   "score": 2,
   "note": "Ledger plus Plaid bank sync, receipt and statement parsing, prices and budgets; no connectors to other ledgers."
  },
  "autonomy": {
   "score": 2,
   "note": "Daily bank sync to staging and AI-suggested entries, but writes to the ledger are reviewed or requested by a client."
  },
  "agent_readiness": {
   "score": 4,
   "note": "Remote MCP with about 31 tools, OAuth 2.1 with per-ledger scope, scoped API keys, GraphQL, OpenAPI, llms.txt and a JSON CLI with exit codes."
  },
  "integration_ease": {
   "score": 4,
   "note": "Self-serve, documented for agents, one-command MCP setup; API keys need a paid plan."
  },
  "market": {
   "score": 1,
   "note": "Mainly people, couples and freelancers, with plans up to multi-entity organisations."
  }
 },
 "api": {
  "public": true,
  "style": "GraphQL (/graphql), remote MCP over streamable HTTP (/api-gateway/mcp), REST JSON for queries, git over SSH, and docs as Markdown",
  "auth": "OAuth 2.1 with PKCE (scopes read, write, admin; one ledger or all), scoped X-API-Key keys, static MCP tokens, SSH keys for git",
  "limits": "No published rate limits; error responses include a retryAfter field.",
  "cost": "No per-call metering found. Free plan: 1 ledger, 1,000 entries; minting API keys requires a paid plan.",
  "export": "The whole ledger as plain-text files in a git repository with its history; clone over SSH or push to another git host.",
  "notes": "The MCP manifest at /.well-known/mcp.json lists tools for BQL, ledger files, entry-level edits, receipts, bank import, pull requests, API keys, collaborators, ledgers and account deletion."
 },
 "pricing": {
  "model": "Freemium subscription by ledgers, collaborators, entries and AI tokens",
  "entry": "Free: 1 ledger, 1 collaborator, 1,000 entries, 20K AI tokens",
  "top": "Organization USD 499.99/month (USD 399.99 billed annually): 100 ledgers, 50 collaborators, 10M AI tokens, SSO, multi-entity; Enterprise by quote",
  "notes": "Premium USD 14.99/month (USD 8.33 billed annually); Growth USD 99.99/month (USD 79.99 annually). Prices as shown on 10 October 2026, in USD."
 },
 "agents": {
  "mcp": "official",
  "ai_features": "Ask AI assistant that drafts and writes entries and reads receipts and statement screenshots (June 2026); AI categorisation of bank lines; Smart Import of files; 'bea ask' with SKILL.md conventions.",
  "agent_writes": "Yes: editLedgerFiles (atomic commit), addLedgerEntries, appendLedgerText, editEntrySource, manageBankImport submit, and managePullRequests; each write is a git commit.",
  "approval": "Optional and client-side. dry_run returns a diff and bean-check verdict; committing is a separate call, and the MCP page says 'approval settings depend on your client'. managePullRequests can create and approve (merge) a proposal, and the same credential can approve its own. The in-app Ask AI commits nothing without the user's explicit approval.",
  "notes": "Writes are reversible through git, but the gate is not enforced by the server for MCP clients. An OAuth credential can call deleteAccount."
 },
 "integration": {
  "scope": "Plaid bank sync (Plaid cites over 12,000 institutions), CSV, OFX and QIF converters, receipts and PDFs through AI parsing, managed price feeds, any git host, MCP clients (Claude Code, Claude Desktop, Cursor, Windsurf, Zed).",
  "notes": "No connectors to Xero, QuickBooks or other ledgers; the docs offer a QuickBooks migration guide instead."
 },
 "preference": {
  "ratings": "App Store (US): 3.5 from 8 ratings for 'Beancount: Plain Text Ledger' by Stargately, Inc., 10 October 2026. GitHub: bex-co/beancount-io 296 stars. No G2, Capterra or Trustpilot listing found (search budget exhausted).",
  "scale": "No customer or user counts published.",
  "sentiment": "Not established this pass: Reddit and review sites could not be searched. The vendor pitches itself on not locking you in ('cancel anytime — your plain-text ledger keeps working')."
 },
 "timeline": [
  {
   "date": "2019",
   "event": "Beancount.io service begins (site copyright from 2019)"
  },
  {
   "date": "2020-08-19",
   "event": "Beancount mobile apps for iOS and Android launched"
  },
  {
   "date": "2020-10-08",
   "event": "Mobile app open-sourced"
  },
  {
   "date": "2023-05-01",
   "event": "Free plan reintroduced for new customers"
  },
  {
   "date": "2025-11-26",
   "event": "Beancount.io v3.0 with git integration"
  },
  {
   "date": "2026-01-28",
   "event": "Open Ledger: public-company ledgers published as Beancount"
  },
  {
   "date": "2026-02-15",
   "event": "Plaid bank sync"
  },
  {
   "date": "2026-06-30",
   "event": "Beancount MCP launched; Ask AI can write entries with approval"
  },
  {
   "date": "2026-09-16",
   "event": "bea 0.2.0 local CLI with the full Beancount toolchain pinned to 3.2.3"
  }
 ],
 "relation": "neighbour",
 "relation_note": "The nearest neighbour in the agent-native, plain-text space: it gives agents a ledger with git as its audit log, while ownpurse sits beside existing ledgers, keeps a local fact log of what each source said, and enforces approval on writes to a source. It competes for the same developer and personal-finance users who want their books readable by agents.",
 "borrow": "Agent-first docs (llms.txt, Markdown docs, an MCP manifest with error codes and hints). Dry-run previews with a validator verdict before every write. Ledger-scoped OAuth that sets the blast radius. A JSON CLI with stable exit codes that works without an account. Git commits labelled 'AI edit' so agent changes are reviewable.",
 "avoid": "An approval step that lives in the client and can be skipped, or approved by the same agent that proposed it. A hosted repository as the working copy, with the owner's local copy only as fresh as their last fetch. Granting an agent account-level powers such as deleting the account.",
 "sources": [
  {
   "title": "Beancount.io home",
   "url": "https://beancount.io/"
  },
  {
   "title": "Beancount.io llms.txt",
   "url": "https://beancount.io/llms.txt"
  },
  {
   "title": "Beancount.io pricing",
   "url": "https://beancount.io/pricing"
  },
  {
   "title": "Beancount MCP",
   "url": "https://beancount.io/mcp"
  },
  {
   "title": "Beancount.io MCP manifest",
   "url": "https://beancount.io/.well-known/mcp.json"
  },
  {
   "title": "Beancount.io OpenAPI",
   "url": "https://beancount.io/openapi.json"
  },
  {
   "title": "Git for Beancount",
   "url": "https://beancount.io/git"
  },
  {
   "title": "Bank Sync",
   "url": "https://beancount.io/bank-sync"
  },
  {
   "title": "Security",
   "url": "https://beancount.io/security"
  },
  {
   "title": "Terms of Service (Stargately, Inc.)",
   "url": "https://beancount.io/terms-of-service"
  },
  {
   "title": "Changelog",
   "url": "https://beancount.io/changelog"
  },
  {
   "title": "About Beancount.io",
   "url": "https://beancount.io/about-us"
  },
  {
   "title": "bea CLI quick start",
   "url": "https://beancount.io/docs/Basics/bea-cli.md"
  },
  {
   "title": "bea CLI reference",
   "url": "https://beancount.io/docs/bea-cli-reference.md"
  },
  {
   "title": "bex-co/beancount-io on GitHub",
   "url": "https://github.com/bex-co/beancount-io"
  },
  {
   "title": "beancount-io on PyPI",
   "url": "https://pypi.org/project/beancount-io/"
  }
 ],
 "unverified": [
  "Stargately, Inc.'s location and any funding.",
  "Whether the hosted git server allows force-pushes or other history rewrites.",
  "Rate limits for GraphQL, REST and MCP.",
  "User or customer counts.",
  "Customer sentiment on Reddit, G2, Capterra or Trustpilot (search budget exhausted).",
  "Plaid's 'over 12,000 institutions' figure is quoted from Beancount.io, not checked with Plaid.",
  "The exact MCP tool count (about 31 in the manifest on 10 October 2026)."
 ],
 "researched": "2026-10-10"
}
---

## What it is

Beancount.io is a hosted service for [Beancount](https://beancount.github.io/), Martin Blais's open-source plain-text double-entry engine. It is run by Stargately, Inc., according to its [terms](https://beancount.io/terms-of-service). Each ledger is a set of `.beancount` files in a private git repository, with hosted Fava, mobile apps, Plaid bank sync, AI import, managed prices and an AI assistant on top. Since June 2026 it has also sold itself as a ledger for AI agents.

## How it works

You can [clone the ledger over SSH](https://beancount.io/git), edit it in any editor and push, or push it to GitHub or your own server. Bank lines from [Plaid](https://beancount.io/bank-sync) sync daily into staging and are reviewed before they are committed. The open-source `bea` CLI ([quick start](https://beancount.io/docs/Basics/bea-cli.md)) runs local ledgers with no account, with JSON output and documented exit codes; version 0.2.0 (16 September 2026) bundles the toolchain pinned to Beancount 3.2.3.

## Data and access

The [MCP server](https://beancount.io/mcp) is one command (`claude mcp add --transport http beancount https://beancount.io/api-gateway/mcp`) followed by an OAuth 2.1 sign-in where you approve one ledger or all of them. Its [manifest](https://beancount.io/.well-known/mcp.json) lists about 31 tools: BQL queries (plain or typed tables), ledger and entry context, file reads, entry-level edits addressed by hash, receipt and file parsing, bank import, pull requests, prices, API keys, collaborators and account deletion. The [OpenAPI document](https://beancount.io/openapi.json) also lists GraphQL, scoped API keys with read, write and admin roles, and an `llms.txt` index. No rate limits are published. Export is the repository itself.

[Pricing](https://beancount.io/pricing) on 10 October 2026: Free (1 ledger, 1,000 entries, 20K AI tokens), Premium USD 14.99 a month, Growth USD 99.99, Organization USD 499.99 with SSO and multi-entity, and Enterprise by quote. API keys need a paid plan.

## Agents

This is the most complete agent interface in the plain-text segment. `editLedgerFiles` validates and returns a unified diff with bean-check's verdict when `dry_run` is true, and a separate call commits. Every write lands as a git commit ('AI edit: …') you can revert with ordinary git. The gate is the client's, though: the MCP page says 'approval settings depend on your client', and `managePullRequests` lets the same credential create a proposal and approve it by merging. The in-app assistant, by contrast, 'commits nothing without explicit user approval' ([changelog](https://beancount.io/changelog), 30 June 2026). The [security page](https://beancount.io/security) lists Anthropic, OpenAI and a bex.co proxy as the AI data path.

## Customers

No user numbers are published. The iOS app has 3.5 stars from 8 ratings (US store, 10 October 2026) and the open-source mobile repository has 296 stars. Review sites and Reddit could not be checked this pass.

## History

The service dates from 2019; mobile apps followed in August 2020 and were open-sourced that October. A free plan returned in May 2023. Version 3.0 added git integration (November 2025). 2026 brought Open Ledger (public-company books in Beancount), Smart Import, Plaid sync, budgets, the MCP and bea.

## For ownpurse

Beancount.io proves that 'every edit is a commit' is a credible audit story for agents, and its agent ergonomics (dry runs with validator output, ledger-scoped OAuth, typed errors with hints, Markdown docs) are a bar ownpurse should meet. The differences matter. Beancount.io is the ledger: you move your books in. ownpurse leaves your books in Xero or QuickBooks and keeps a local, hash-chained record of what each source said. Beancount.io's history records edits to normalized text, not the raw source. Its approval is optional and client-side; ownpurse's sealed package is the only path a write can take.
