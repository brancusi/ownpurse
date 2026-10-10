---
{
 "slug": "hledger",
 "name": "hledger",
 "url": "https://hledger.org",
 "segment": "plain-text-local",
 "status": "active",
 "founded": "2007 (first public release 0.1 on 2008-10-15)",
 "hq": "No headquarters; a volunteer open-source project led by Simon Michael with 140+ contributors",
 "backing": "No company or investors. Sponsorship through its sponsor page and Open Collective: $12,526.08 raised in total, an estimated $937 annual budget and a $4,855.30 balance (Open Collective, checked 2026-10-10). The release notes say the project used a six-month open-source credit from Anthropic in 2026 Q2 and Q3.",
 "tagline": "A Haskell plain-text accounting tool, Ledger-compatible, with CSV rules, a terminal UI, a web UI and a JSON API.",
 "scope": "hledger reads journal, CSV/TSV, timeclock and timedot files, checks them and produces balance sheets, income statements, registers and many other reports in text, HTML, CSV, JSON, SQL and Beancount formats. Its import command turns bank CSV into journal entries with rules files and can run your own fetch scripts. It does not connect to banks or accounting systems itself and keeps no version history of the journal.",
 "domains": [
  "ledger",
  "personal",
  "reporting",
  "crypto"
 ],
 "customers_served": [
  "consumer",
  "freelancer",
  "developers",
  "smb"
 ],
 "spectrum": {
  "custody": {
   "score": 4,
   "note": "The journal is a text file you hold."
  },
  "locality": {
   "score": 4,
   "note": "A local binary; hledger-web serves on your machine."
  },
  "history": {
   "score": 2,
   "note": "The journal is edited in place; since 1.50 the archive rule saves a dated copy of each imported data file, and Git is left to you; no as-of queries."
  },
  "openness": {
   "score": 4,
   "note": "Open source under GPL-3.0-or-later."
  },
  "breadth": {
   "score": 2,
   "note": "Any CSV-shaped source through rules files, plus time and commodities; no built-in connectors."
  },
  "autonomy": {
   "score": 2,
   "note": "hledger import writes new entries straight into the journal and can fetch first with --get, so it can run unattended from a scheduler."
  },
  "agent_readiness": {
   "score": 2,
   "note": "JSON output on reports and a JSON API in hledger-web, with view/add/edit levels but no access control; no official MCP."
  },
  "integration_ease": {
   "score": 3,
   "note": "Declarative CSV rules, many output formats and a documented JSON API make it easy for an owner to connect sources and tools."
  },
  "market": {
   "score": 1,
   "note": "Individuals, freelancers and some small businesses, mostly technical; not sold to firms."
  }
 },
 "api": {
  "public": true,
  "style": "CLI with JSON output (print, register, aregister and more); hledger-web JSON API over HTTP (GET /transactions, /accounts, /accountnames, /prices, /commodities, /accounttransactions; PUT /add); Haskell library",
  "auth": "None. The hledger-web manual says 'There is no access control'; bind to a host, use an authenticating proxy or a firewall. --allow sets view, add or edit (default add).",
  "limits": "None; local",
  "cost": "Free",
  "export": "Everything: the journal itself, and reports as txt, html, csv, tsv, json, sql, ledger or beancount.",
  "notes": "Stable line 1.52.4 (2026-09-10); hledger 2.0 previews 1.99.x, latest 1.99.5 (2026-10-01), add lot tracking, a holdings report and commodity aliases."
 },
 "pricing": {
  "model": "Free open-source software, donation-funded",
  "entry": "Free",
  "top": "Free",
  "notes": "No paid tier or hosted service."
 },
 "agents": {
  "mcp": "none",
  "ai_features": "None in the product. plaintextaccounting.org lists accountant24 (a local-first AI agent that works with hledger) and a write-up on using Claude Code with hledger in Obsidian. The project's homepage mentions combining hledger with 'version control, scripts, and LLMs'.",
  "agent_writes": "An agent can write to the journal by appending text, by running hledger import, or through hledger-web's PUT /add with full transaction JSON.",
  "approval": "No approval gate. import --dry-run previews what would be added; hledger-web's --allow=view makes the API read-only; hledger check --strict validates afterwards.",
  "notes": "hledger 2 is developed with careful AI assistance (from 1.99.1, 2026-03-28) under a published AI policy: changes steered, reviewed and tested by a person, and first-time contributors may not use AI tools. That shapes the developers, not the product's agent interface."
 },
 "integration": {
  "scope": "Inputs: journal, timeclock, timedot, CSV, TSV, SSV via rules files; source rules can run a command (the docs show fetching SimpleFIN JSON and converting it to CSV). Outputs: txt, html, csv, tsv, ledger, beancount, sql, json.",
  "notes": "Interfaces: CLI, hledger-ui (terminal) and hledger-web (browser plus JSON API). The journal format is largely Ledger-compatible."
 },
 "preference": {
  "scale": "About 4.8k GitHub stars and 415 forks (GitHub, 2026-10-10); plaintextaccounting.org (2026) lists about 199 committers, a mailing list of about 250 and a Matrix room of about 240. No user counts are published.",
  "ratings": "No G2, Capterra or Trustpilot profiles were found.",
  "sentiment": "Users praise its speed, friendly error messages, documentation, CSV rules and the choice of CLI, TUI and web UI. The costs are the learning curve, writing rules per bank, and an import that trusts dates, which its own manual says can miss or repeat records when a bank's file changes."
 },
 "timeline": [
  {
   "date": "2007",
   "event": "Simon Michael begins hledger as a Haskell take on Ledger."
  },
  {
   "date": "2008-10-15",
   "event": "hledger 0.1 announced."
  },
  {
   "date": "2016-10-26",
   "event": "hledger 1.0: more hledger-ui features, a better hledger-web, the hledger-api server, the timedot format."
  },
  {
   "date": "2016",
   "event": "Featured on FLOSS Weekly 375."
  },
  {
   "date": "2025-09-03",
   "event": "hledger 1.50: import archiving, CSV data commands, better transaction balancing."
  },
  {
   "date": "2026-03-20",
   "event": "hledger 1.52."
  },
  {
   "date": "2026-03-28",
   "event": "hledger 1.99.1, the first 2.0 preview, with lot tracking and a policy for AI-assisted development."
  },
  {
   "date": "2026-09-10",
   "event": "hledger 1.52.4 (stable) and 1.99.4 (preview)."
  },
  {
   "date": "2026-10-01",
   "event": "hledger 1.99.5, about four times faster than 1.99.4 on a 100,000-transaction journal."
  }
 ],
 "relation": "kin",
 "relation_note": "A local, exact, owner-held ledger like ownpurse wants, with a good CSV path; it keeps a normalized journal you edit, not a log of what each source said.",
 "borrow": "Declarative rules files per source, the archive rule (a dated copy of every imported file), the --dry-run preview, many output formats including JSON, and the import --get split between fetching and reading.",
 "avoid": "The date high-water mark as the dedupe rule: hledger import skips records dated before the last imported date, which the manual says assumes stable file names, stable dates, stable order and newest-last, and it does not detect identical records. Also avoid a write API with no access control.",
 "sources": [
  {
   "title": "hledger home page",
   "url": "https://hledger.org/"
  },
  {
   "title": "hledger release notes",
   "url": "https://hledger.org/release-notes.html"
  },
  {
   "title": "hledger manual (dev): import, deduplication, archiving",
   "url": "https://hledger.org/dev/hledger.html"
  },
  {
   "title": "hledger-web manual 1.52: JSON API and access",
   "url": "https://hledger.org/1.52/hledger-web.html"
  },
  {
   "title": "hledger on GitHub",
   "url": "https://github.com/simonmichael/hledger"
  },
  {
   "title": "hledger on Open Collective",
   "url": "https://opencollective.com/hledger"
  },
  {
   "title": "plaintextaccounting.org",
   "url": "https://plaintextaccounting.org/"
  }
 ],
 "unverified": [
  "Whether any community MCP server for hledger exists; web search was unavailable and none was found in the sources read.",
  "Exact meaning of the GitHub '4.8k' star count date (read from the GitHub page on 2026-10-10, not the API).",
  "Customer sentiment is drawn from project docs and plaintextaccounting.org; Reddit could not be fetched and no review-site profiles were found.",
  "Total sponsorship beyond Open Collective (GitHub Sponsors or direct) is not stated."
 ],
 "researched": "2026-10-10"
}
---

## What it is

[hledger](https://hledger.org) is plain-text accounting software in Haskell, created by Simon Michael and maintained since 2007, with 140+ contributors. It reads a journal you write (largely compatible with Ledger), checks it and produces reports. It ships three interfaces: the CLI, hledger-ui in the terminal and hledger-web in the browser, which also serves a JSON API. It is free software under GPL-3.0-or-later.

It is not a bank-sync service, has no hosted version and keeps no history of the journal beyond what you put in Git.

## How it works

You record transactions in a journal, or import them. `hledger import` reads CSV, TSV or SSV through a `.rules` file that maps columns to dates, amounts, payees and accounts, with conditional rules for categorizing. Since [1.50](https://hledger.org/release-notes.html) (2025-09-03) a `source` rule can run a command to produce the data (the docs show fetching JSON from SimpleFIN and converting it to CSV), and an `archive` rule saves a dated copy of every imported file in `data/` next to the journal, "useful for troubleshooting, detecting variations in your banks' CSV data, regenerating entries with improved rules". `import --get` runs your fetch scripts first; `--dry-run` previews.

The weak point is how import decides what is new. The [manual](https://hledger.org/dev/hledger.html) says it "remembers the latest date processed previously in each CSV file" in a hidden `.latest.FILE` and skips records before that date (and the first N on that date). It works when the file name, item dates and same-date order are stable and the newest items have the newest dates. It deliberately does not deduplicate identical records, since "it's generally not possible to reliably detect duplicates in bank CSV". A bank that backdates a pending item, reorders a day or renames its export breaks the assumption without an error.

## Data and access

You hold the journal. Reports export as txt, html, csv, tsv, json, sql, ledger or beancount. The [hledger-web API](https://hledger.org/1.52/hledger-web.html) offers GET routes for transactions, accounts, prices and commodities and a PUT `/add` that takes a complete transaction. Access is set with `--allow` (view, add or edit; default add), and the manual is direct: "There is no access control." You bind it to localhost or put an authenticating proxy in front.

## Agents

There is no official MCP server or AI feature. The JSON output and API are a reasonable base for an agent, and [plaintextaccounting.org](https://plaintextaccounting.org/) lists accountant24, a local-first AI agent for hledger, and a guide to using Claude Code with hledger in Obsidian. Writes have no approval gate: an agent can append to the journal or call `/add`, and `hledger check --strict` validates afterwards.

The development side is notable. hledger 1 (up to 1.52.1) was written without AI assistance; hledger 2 (from 1.99.1, 2026-03-28) is "exploring careful AI-assisted development", with every change "steered, reviewed, and tested by Simon or a contributor", a ban on AI tools for first-time contributors' PRs and published token estimates. The project used an Anthropic open-source credit in 2026 Q2 and Q3.

## Customers

Individuals, freelancers and some small businesses, most of them comfortable with a terminal. About 4.8k GitHub stars; plaintextaccounting.org (2026) counts about 199 committers. [Open Collective](https://opencollective.com/hledger) shows $12,526.08 raised in total and an estimated $937 annual budget (2026-10-10). Users praise its speed, documentation, error messages and CSV rules; the costs are the learning curve and rules per bank.

## History

0.1 was announced 2008-10-15 as "a command-line accounting tool similar to John Wiegley's c++ ledger". 1.0 came 2016-10-26. The 1.5x line added import archiving and data commands. The 2.0 previews (1.99.x, latest 1.99.5 on 2026-10-01) add lot tracking, capital gains, a holdings report and commodity aliases, with stable 1.52.4 (2026-09-10) alongside.

## For ownpurse

hledger is kin and a likely neighbour for the same people: someone who already keeps an hledger journal could point it at ownpurse output. Worth borrowing: declarative rules per source, keeping the original file on every import, a dry run before any write, and many honest output formats.

The lesson to avoid is the identity rule. A date high-water mark is cheap and usually right, and wrong in exactly the cases that matter: backdated items, reordered days, identical purchases. ownpurse files each download by hash and identifies records by source id, natural key or fingerprint, so reloading changes nothing. And where hledger-web lets anyone who can reach it add transactions, ownpurse keeps reads open and writes behind a sealed package a person approves.
