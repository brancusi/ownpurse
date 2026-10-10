---
{
 "slug": "paisa",
 "name": "Paisa",
 "url": "https://paisa.fyi",
 "segment": "plain-text-local",
 "status": "active",
 "founded": "2022 (repository created 2022-03-26; v0.1.0 on 2022-04-20)",
 "hq": "No company; a personal open-source project by Anantha Kumaran. Its feature set is India-centred (INR, NPS funds, Schedule AL); a location was not confirmed.",
 "backing": "None. The installation page says it is a free app that generates no revenue; the author mentions an affiliate link to PikaPods hosting. No funding, no sponsors file in the repository (checked 2026-10-10).",
 "tagline": "A free, AGPL-licensed personal finance app that puts a web and desktop interface over a ledger, hledger or beancount journal on your own machine.",
 "scope": "Paisa reads a plain-text journal, syncs it into a local SQLite cache and shows net worth, cash flow, budgets, goals, investments with fetched prices, credit cards, capital gains and Indian tax schedules. It has an editor for the journal, a notepad calculator (Sheets) and a template-driven importer for CSV, Excel and experimental PDF. It has no bank or provider connectors, no public API, no multi-user books and no business accounting features such as invoicing or payroll.",
 "domains": [
  "ledger",
  "personal",
  "reporting",
  "tax"
 ],
 "customers_served": [
  "consumer"
 ],
 "spectrum": {
  "custody": {
   "score": 4,
   "note": "The journal file on your disk is the authoritative record; the SQLite database is a cache rebuilt from it."
  },
  "locality": {
   "score": 4,
   "note": "Desktop app for macOS, Windows and Linux, or a local binary serving localhost:7500; the site says data never leaves your system."
  },
  "history": {
   "score": 1,
   "note": "The journal is edited in place; the editor writes a timestamped .backup copy before each save, and git is up to you."
  },
  "openness": {
   "score": 4,
   "note": "Open source under AGPL-3.0 or later."
  },
  "breadth": {
   "score": 2,
   "note": "Covers personal money broadly (banks, cards, investments, prices, goals, tax), but only what you import into one journal."
  },
  "autonomy": {
   "score": 1,
   "note": "A person imports and saves; the only automation is an account suggestion matched against earlier entries."
  },
  "agent_readiness": {
   "score": 1,
   "note": "An undocumented internal JSON API serves the UI; no CLI for queries, no MCP. Agents do better reading the journal with hledger or ledger directly."
  },
  "integration_ease": {
   "score": 2,
   "note": "Drops onto an existing journal with a YAML config; import templates are Handlebars; no plugin or extension API."
  },
  "market": {
   "score": 0,
   "note": "Individuals tracking personal and household finances."
  }
 },
 "api": {
  "public": false,
  "style": "Internal REST/JSON endpoints under /api (Go, Gin) used by the Svelte front end; not documented as a public interface",
  "auth": "Optional username and password list in paisa.yaml (password hashed twice with SHA-256); no authentication if the list is empty",
  "limits": "None; local",
  "cost": "Free",
  "export": "The journal itself is plain text you already hold; v0.7.6 (2026-09-06) added the transaction id to Paisa's export",
  "notes": "An authentication bypass and path traversal let an unauthenticated request overwrite the ledger binary and run code (issue #294, reported and fixed 2024-10-20, released in v0.7.1). Anyone exposing Paisa beyond localhost should run v0.7.1 or later and set user_accounts."
 },
 "pricing": {
  "model": "Free and open source",
  "entry": "Free",
  "top": "Free (third-party hosting on PikaPods is paid; price not checked)",
  "notes": "Binaries are unsigned because code signing would cost about $99 a year for macOS and about $300 a year for Windows, per the installation page."
 },
 "agents": {
  "mcp": "community",
  "ai_features": "None marketed. predictAccount suggests an account during import by matching the description against earlier categorized entries (TF-IDF, per the /api/account/tf_idf route).",
  "agent_writes": "Not through a supported interface. The internal API can save journal files, and an agent can edit the plain-text journal directly.",
  "approval": "None built in; whatever review the person does before saving or committing the file.",
  "notes": "A community repository named paisa-mcp exists on GitHub (0 stars, 2025-12); its contents were not reviewed. Several Beancount MCP servers can read a beancount journal that Paisa also displays."
 },
 "integration": {
  "scope": "Three ledger engines (ledger by default and bundled; hledger and beancount installed separately). Price providers for mutual funds, NPS and stocks. Import from CSV, TXT, XLS, XLSX and PDF via templates.",
  "notes": "No bank, card or accounting-system connectors. On beancount, budgets are not supported and recurring transactions only partly."
 },
 "preference": {
  "scale": "3,223 GitHub stars and 214 forks (2026-10-10); 107,753 Docker Hub pulls of ananthakumaran/paisa (2026-10-10). Commits: 638 of 648 by the author.",
  "ratings": "No G2, Capterra or Trustpilot listing found; web search was unavailable for this pass, so absence is not confirmed.",
  "sentiment": "Listed on plaintextaccounting.org as a web UI or desktop app for all three engines. Open issues centre on UI bugs, slow pages on large ledgers and Docker setup. The README says the author uses it for his own finances and most of his use cases are covered; release notes since late 2024 are mainly build and price-fetcher fixes."
 },
 "timeline": [
  {
   "date": "2022-03-26",
   "event": "Repository created"
  },
  {
   "date": "2022-04-20",
   "event": "v0.1.0 released"
  },
  {
   "date": "2023-05-27",
   "event": "v0.3.6 adds hledger support"
  },
  {
   "date": "2023-09-22",
   "event": "v0.5.2 adds the desktop app"
  },
  {
   "date": "2023-10-07",
   "event": "v0.5.5 adds beancount as a ledger client"
  },
  {
   "date": "2024-01-13",
   "event": "v0.6.3 introduces Sheets, a notepad calculator over the ledger"
  },
  {
   "date": "2024-08-26",
   "event": "v0.7.0 adds Docker variants for hledger and beancount"
  },
  {
   "date": "2024-10-20",
   "event": "Remote code execution via auth bypass reported and fixed (v0.7.1)"
  },
  {
   "date": "2026-09-06",
   "event": "v0.7.6: transaction id in export, toolchain updates"
  }
 ],
 "relation": "kin",
 "relation_note": "Same conviction that the owner holds the file and the app is a view over it. Paisa stops at a single normalized journal you edit by hand; ownpurse keeps every source's raw facts and checks the books against the source.",
 "borrow": "A derived SQLite cache that is always rebuildable from the owner's record (ownpurse's projections work the same way); a timestamped backup before any write; account suggestions learned from the owner's own earlier entries; a desktop app as the zero-setup path to a local server.",
 "avoid": "A local web server with write endpoints and optional auth (issue #294 shows the attack surface); importers that discard the original file; a single-maintainer project as the only home for someone's finances.",
 "sources": [
  {
   "title": "Paisa home page",
   "url": "https://paisa.fyi"
  },
  {
   "title": "Paisa installation",
   "url": "https://paisa.fyi/getting-started/installation/"
  },
  {
   "title": "Paisa ledger CLI reference",
   "url": "https://paisa.fyi/reference/ledger-cli/"
  },
  {
   "title": "Paisa import reference",
   "url": "https://paisa.fyi/reference/import/"
  },
  {
   "title": "Paisa configuration reference",
   "url": "https://paisa.fyi/reference/config/"
  },
  {
   "title": "Paisa GitHub repository",
   "url": "https://github.com/ananthakumaran/paisa"
  },
  {
   "title": "Paisa CHANGELOG",
   "url": "https://github.com/ananthakumaran/paisa/blob/master/CHANGELOG.md"
  },
  {
   "title": "Issue #294: unauthorized remote command execution",
   "url": "https://github.com/ananthakumaran/paisa/issues/294"
  },
  {
   "title": "Docker Hub: ananthakumaran/paisa",
   "url": "https://hub.docker.com/r/ananthakumaran/paisa"
  },
  {
   "title": "plaintextaccounting.org",
   "url": "https://plaintextaccounting.org/"
  },
  {
   "title": "beanquery-mcp (community MCP for Beancount)",
   "url": "https://github.com/vanto/beanquery-mcp"
  }
 ],
 "unverified": [
  "The author's location and Paisa's user geography (inferred from India-specific features only).",
  "Whether any G2, Capterra, Trustpilot or Reddit reviews exist; web search was unavailable in this pass.",
  "PikaPods hosting price for Paisa.",
  "What FinBodhi is and how it relates to Paisa (its site returned no content).",
  "Contents of the community paisa-mcp repository.",
  "Release date of v0.7.5: the CHANGELOG says 2025-12-02, the GitHub release is dated 2026-08-16."
 ],
 "researched": "2026-10-10"
}
---

## What it is

Paisa is a personal finance manager by Anantha Kumaran that sits on top of a plain-text double-entry journal. It works with [ledger, hledger or beancount](https://paisa.fyi/reference/ledger-cli/) and adds what those command-line tools lack: charts of net worth and cash flow, budgets, goals, investment tracking with fetched prices, credit card bills and capital gains, plus tax pages written for India (tax harvesting, Schedule AL). The [site](https://paisa.fyi) says your data never leaves your system and the app never sends data to any server. It is [AGPL-3.0](https://github.com/ananthakumaran/paisa) and free; the author says it earns nothing.

## How it works

You point `paisa.yaml` at a journal file and a database path. Paisa shells out to the ledger engine, then syncs the result into SQLite (Go, gorm) and serves a Svelte interface on localhost:7500, either from the `paisa serve` binary, a Docker image or the [desktop app](https://paisa.fyi/getting-started/installation/) for macOS, Windows and Linux. The journal stays the record; the database is a cache. The built-in editor checks syntax and formats entries, and before each save it writes a timestamped `.backup` copy of the file.

Import is template-driven: [Handlebars templates](https://paisa.fyi/reference/import/) map spreadsheet columns (`ROW.A`) to journal entries from CSV, XLS, XLSX or experimental PDF. `predictAccount` suggests an account by matching the description against entries you already categorized. The docs do not describe duplicate detection or keeping the original file.

## Data and access

There is no public API. The UI talks to about fifty internal JSON routes under `/api` (dashboard, ledger, budgets, editor save, sync), which are not documented for outside use. Authentication is an optional list of users in the config; with an empty list there is none. In October 2024 a reporter showed that a URL-encoded path skipped that check and a path traversal in Sheets could overwrite the ledger binary, giving remote code execution; it was [fixed the same day](https://github.com/ananthakumaran/paisa/issues/294) (v0.7.1). Export is less of a question than usual: the journal is already a plain file you hold.

## Agents

Paisa has no agent features, no MCP server and no query CLI of its own. That matters less than it sounds: an agent can read the journal with hledger or ledger, and on beancount, with community MCP servers such as [beanquery-mcp](https://github.com/vanto/beanquery-mcp). Writes are whatever the agent does to the text file, with no approval step beyond your own review or git.

## Customers

Individuals managing household money, many in India judging by the feature set. The repository had 3,223 stars and 214 forks and the Docker image 107,753 pulls on 2026-10-10. No review-site listings were found. [plaintextaccounting.org](https://plaintextaccounting.org/) lists it among the web UIs for all three engines. The author wrote 638 of 648 commits, and since late 2024 releases have mostly been maintenance; the site now also points newcomers who do not want to edit text files to a separate product, FinBodhi, which this pass did not examine.

## History

The repository was created in March 2022 and v0.1.0 shipped in April 2022. hledger support came in May 2023, the desktop app in September 2023, beancount in October 2023, Sheets in January 2024, Docker variants per engine in August 2024, and the security fix in October 2024. The latest release is v0.7.6 (2026-09-06).

## For ownpurse

Paisa shows how far a good interface takes a plain file: people who would never run `hledger bal` will use a dashboard over the same journal. It also shows the limits. The journal is a single normalized view, imports keep no trace of the raw source, edits overwrite in place, and nothing checks the books against the bank or the provider. ownpurse's projections can play the role Paisa's SQLite cache plays, always rebuildable from the record, while the record itself keeps what each source said. The security issue is the other lesson: a local server that can write files needs authentication and path checks from day one, even on localhost.
