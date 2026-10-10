---
{
 "slug": "beancount",
 "name": "Beancount",
 "url": "https://beancount.github.io",
 "segment": "plain-text-local",
 "status": "active",
 "founded": "2008 (Martin Blais dates his own start to around 2007)",
 "hq": "No headquarters; a volunteer open-source project written and led by Martin Blais",
 "backing": "No funding or company. Donations go to the author through Wise or PayPal (README, checked 2026-10-10). A separate commercial service, beancount.io, sells a hosted ledger and MCP endpoint; its page does not say it is affiliated with the project.",
 "tagline": "A double-entry bookkeeping language for text files, with a strict Python core, a SQL-like query language and a separate importer framework.",
 "scope": "Beancount parses a plain-text journal, checks it strictly (transactions must balance, lots are matched, balance assertions are dated) and produces reports and a Python data structure. Fava gives it a web interface; beangulp handles importing bank files; beanquery runs BQL queries. It does not fetch from banks or accounting systems, does not run a server of its own, and does not keep versions: history lives in whatever version control you use.",
 "domains": [
  "ledger",
  "personal",
  "documents",
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
   "note": "The journal is a text file you hold; nothing is sent anywhere."
  },
  "locality": {
   "score": 4,
   "note": "Runs entirely on your machine as a Python package; Fava serves locally."
  },
  "history": {
   "score": 2,
   "note": "The journal is edited in place, but the recommended setup keeps it in Git and files every original download into a documents archive; no as-of queries."
  },
  "openness": {
   "score": 4,
   "note": "Open source under GPL-2.0-only; format and code fully public."
  },
  "breadth": {
   "score": 2,
   "note": "Any source can be imported through a beangulp importer, and prices, investments and documents fit, but you write or find each importer yourself."
  },
  "autonomy": {
   "score": 1,
   "note": "Importers extract entries for a person to review and merge; nothing posts to the books unattended by default."
  },
  "agent_readiness": {
   "score": 2,
   "note": "A Python loader API, BQL and a strict checker an agent can run; MCP servers exist only from the community and a commercial third party."
  },
  "integration_ease": {
   "score": 3,
   "note": "A documented importer interface and a Python API make connecting a new source an afternoon's work for a programmer."
  },
  "market": {
   "score": 1,
   "note": "Technical individuals first; the author notes some companies and foundations use it, but it is not sold to businesses."
  }
 },
 "api": {
  "public": true,
  "style": "Python library (beancount.loader, beancount.core), BQL via beanquery, plugin hooks; no network API in the core",
  "auth": "None; local files",
  "limits": "None; bounded only by your machine",
  "cost": "Free",
  "export": "Everything: the journal is the data. BQL and Fava export query results to CSV and other formats.",
  "notes": "Beancount 3 (stable since June 2024) moved importing and querying into separate packages: beangulp 0.2.0 (2025-01-20) and beanquery 0.2.0 (2025-03-24). The latest beancount release on PyPI is 3.2.3 (2026-05-05)."
 },
 "pricing": {
  "model": "Free open-source software",
  "entry": "Free",
  "top": "Free",
  "notes": "Donations are optional. beancount.io, an unrelated-by-statement commercial host, requires a paid plan for personal API keys; its prices were not checked."
 },
 "agents": {
  "mcp": "community",
  "ai_features": "None in the core. The ecosystem has StdioA/beancount-mcp (query and submit transactions to a local ledger), vanto/beanquery-mcp (experimental, BQL), Countbean (a Claude Code plugin and MCP server), Beanhand (receipt photos to entries) and Finzytrack, all third party.",
  "agent_writes": "An agent can append text to the journal or call a community MCP tool that writes transactions; the core itself has no write API. beancount.io's hosted MCP has appendLedgerText and editLedgerFiles tools that commit to Git.",
  "approval": "No built-in approval gate. The strict checker (bean-check) catches unbalanced or inconsistent entries after the fact; beancount.io offers a dry_run diff but says its server shows no human approval dialog and leaves confirmation to the client.",
  "notes": "The strictness is what makes Beancount a good target for agents: an agent's mistake usually fails bean-check rather than silently skewing a balance."
 },
 "integration": {
  "scope": "Inputs through beangulp importers (CSV, OFX and PDF importers are written per bank, many shared by the community); outputs through BQL, Fava and the Python API. Fava 1.30.16 (2026-08-18) also offers an import screen over beangulp.",
  "notes": "No bank feeds or accounting-system connectors ship with it. The importer interface is small: identify, account, date, filename, extract, deduplicate and sort."
 },
 "preference": {
  "scale": "6,065 GitHub stars for beancount and 2,583 for Fava (GitHub API, 2026-10-10). plaintextaccounting.org (2026) lists about 103 committers and a mailing list of about 880. No user counts are published.",
  "ratings": "No G2, Capterra or Trustpilot profiles were found for the open-source project.",
  "sentiment": "Its author's comparison calls it 'highly pessimistic', and users who choose it tend to value that strictness, the Python plugin system, BQL and Fava. The usual costs are the learning curve, writing your own importers, and the v2 to v3 split that moved tools into separate packages."
 },
 "timeline": [
  {
   "date": "2008",
   "event": "Beancount begun by Martin Blais, inspired by Ledger (project history doc)."
  },
  {
   "date": "2013",
   "event": "Development on version 1, partly Ledger-compatible, halted; version 2 is a full rewrite with a new grammar."
  },
  {
   "date": "2016-02-15",
   "event": "First PyPI upload (2.0b5)."
  },
  {
   "date": "2020",
   "event": "Version 2 enters maintenance mode."
  },
  {
   "date": "2024-05-29",
   "event": "beangulp 0.1.0 published, the successor to beancount.ingest."
  },
  {
   "date": "2024-06",
   "event": "Beancount 3 becomes the stable version, trimmed down with tools moved to their own projects."
  },
  {
   "date": "2025-01-20",
   "event": "beangulp 0.2.0 released."
  },
  {
   "date": "2025-03-29",
   "event": "Community beancount-mcp 0.1.1 published on PyPI."
  },
  {
   "date": "2026-05-05",
   "event": "beancount 3.2.3 released."
  }
 ],
 "relation": "kin",
 "relation_note": "The closest relative in spirit: an owner-held, local, exact record checked by strict rules. It differs from ownpurse in keeping one normalized journal you edit, rather than an append-only log of what each source reported.",
 "borrow": "beangulp's importer shape: identify the file, extract entries, archive the original under a dated, account-named path, and mark (not drop) likely duplicates within a date window. Also bean-check's pessimism: refuse a record that does not balance.",
 "avoid": "Making the edited journal the only record: once an entry is changed in place, what the source said is gone unless Git or the documents archive happen to hold it. Avoid deduplication by fuzzy date window as the identity rule; keep it as a review hint.",
 "sources": [
  {
   "title": "beancount on GitHub (README, versions, donations)",
   "url": "https://github.com/beancount/beancount"
  },
  {
   "title": "Beancount documentation index",
   "url": "https://beancount.github.io/docs/"
  },
  {
   "title": "Beancount history and credits",
   "url": "https://beancount.github.io/docs/beancount_history_and_credits/"
  },
  {
   "title": "A comparison of Beancount and Ledger/hledger",
   "url": "https://beancount.github.io/docs/a_comparison_of_beancount_and_ledger_hledger/"
  },
  {
   "title": "Importing external data in Beancount",
   "url": "https://beancount.github.io/docs/importing_external_data/"
  },
  {
   "title": "beangulp on GitHub",
   "url": "https://github.com/beancount/beangulp"
  },
  {
   "title": "beangulp Importer class source",
   "url": "https://raw.githubusercontent.com/beancount/beangulp/master/beangulp/importer.py"
  },
  {
   "title": "beancount on PyPI",
   "url": "https://pypi.org/project/beancount/"
  },
  {
   "title": "Fava",
   "url": "https://beancount.github.io/fava/"
  },
  {
   "title": "StdioA/beancount-mcp listing",
   "url": "https://glama.ai/mcp/servers/StdioA/beancount-mcp"
  },
  {
   "title": "vanto/beanquery-mcp",
   "url": "https://github.com/vanto/beanquery-mcp"
  },
  {
   "title": "beancount.io MCP announcement",
   "url": "https://beancount.io/blog/2026/06/30/beancount-mcp"
  },
  {
   "title": "plaintextaccounting.org",
   "url": "https://plaintextaccounting.org/"
  }
 ],
 "unverified": [
  "Whether beancount.io is run by anyone connected to the Beancount project; its page does not say.",
  "beancount.io plan prices.",
  "Exact release date of Beancount 3.0.0 beyond 'June 2024' from the README.",
  "Customer sentiment is drawn from the author's docs and the project's own positioning; Reddit could not be fetched and no G2, Capterra or Trustpilot profiles were found.",
  "Activity and maintenance state of the community MCP servers beyond their last PyPI or GitHub dates (beancount-mcp last pushed 2025-05-08)."
 ],
 "researched": "2026-10-10"
}
---

## What it is

[Beancount](https://github.com/beancount/beancount) is a double-entry bookkeeping language: you write transactions in a text file, Beancount reads them into memory, checks them and produces reports. Martin Blais began it in 2008 after learning double entry and finding the tools of the time fragile; John Wiegley's Ledger was the inspiration, as the [history doc](https://beancount.github.io/docs/beancount_history_and_credits/) says. It is free software under GPL-2.0-only, and Blais has written nearly all of the code himself.

It does not fetch data, host anything or keep versions. It is a strict reader of a file you own.

## How it works

The journal holds dated directives: open and close accounts, transactions, prices, balance assertions, documents, notes. Blais's [comparison with Ledger and hledger](https://beancount.github.io/docs/a_comparison_of_beancount_and_ledger_hledger/) states the stance plainly: "Beancount is highly pessimistic. It assumes the user is unreliable." Every transaction must balance, there are no virtual postings, lots held at cost must match a lot that was added earlier, and balance assertions apply by date, not by file position. Python plugins run inside the load pipeline and can transform or check the directive stream.

Version 3, [stable since June 2024](https://github.com/beancount/beancount), is trimmed down: importing moved to [beangulp](https://github.com/beancount/beangulp) and querying to beanquery, and the web interface, [Fava](https://beancount.github.io/fava/), is its own project (1.30.16, 2026-08-18).

Beangulp's [Importer class](https://raw.githubusercontent.com/beancount/beangulp/master/beangulp/importer.py) is small and worth reading. `identify(filepath)` says whether this importer handles a file. `account`, `date` and `filename` decide where the original is archived. `extract(filepath, existing)` returns new entries and may consult the existing ledger. `deduplicate(entries, existing)` marks, rather than drops, entries that look like ones already in the books, by setting `__duplicate__` metadata; the code compares against existing entries within two days either side (its docstring says five). The [import guide](https://beancount.github.io/docs/importing_external_data/) recommends a Git repository with a `documents/` tree mirroring the chart of accounts, so every download is filed next to the account it fed.

## Data and access

You hold everything. The journal is the data and is readable without Beancount. The Python API (`beancount.loader.load_file`) returns the parsed entries, errors and options; BQL answers SQL-like questions; Fava exports query results. There are no rate limits, keys or fees. History is not Beancount's job: if you edit an entry, the old one exists only if Git kept it.

## Agents

The core has no AI features and no MCP server. The ecosystem does. [StdioA/beancount-mcp](https://glama.ai/mcp/servers/StdioA/beancount-mcp) runs BQL and submits transactions to a local ledger (0.1.1 on PyPI, 2025-03-29). [beanquery-mcp](https://github.com/vanto/beanquery-mcp) is an experimental read-focused server. [plaintextaccounting.org](https://plaintextaccounting.org/) lists Countbean (a Claude Code plugin and MCP server), Beanhand (receipts to entries) and Finzytrack. A commercial host, [beancount.io](https://beancount.io/blog/2026/06/30/beancount-mcp), offers an OAuth-scoped MCP endpoint whose write tools commit to Git with an optional `dry_run` diff; it says the server itself shows no human approval dialog.

The strictness matters more than any of these. An agent that writes a wrong entry usually fails `bean-check` instead of quietly skewing a balance, which is the kind of check an agent can run on its own before a person looks.

## Customers

Technical individuals, households and some sole traders and small firms; the README says it has been useful to "many users, companies, and foundations". beancount has 6,065 GitHub stars and Fava 2,583 (GitHub API, 2026-10-10); plaintextaccounting.org counts about 103 committers and a mailing list of about 880. There are no review-site profiles. People who pick it praise the strict checks, the Python plugins, BQL and Fava; the costs are the learning curve and writing your own importers.

## History

Version 1 (to 2013) was partly Ledger-compatible; version 2 was a complete rewrite with a new grammar, in maintenance from 2020; version 3 became stable in June 2024 and split the tools out. The latest release is 3.2.3 (2026-05-05).

## For ownpurse

Beancount is kin: a local, exact, owner-held record that refuses what does not balance. Two things are worth taking. First, beangulp's importer shape, identify, extract, archive the original and mark likely duplicates, maps directly onto an ownpurse connector writing to the drawer. Second, the pessimism: a record that will not load when it is wrong is the right base for agents.

The difference is where truth sits. Beancount's truth is the normalized journal you edit; what the bank actually said survives only as a filed document. ownpurse keeps what each source reported as facts and derives the ledger, so a correction is a new fact and an as-of question has an answer. A Beancount export, and a Beancount importer that reads the ownpurse record, would serve the same people well.
