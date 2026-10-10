---
{
 "slug": "fava",
 "name": "Fava",
 "url": "https://beancount.github.io/fava/",
 "segment": "plain-text-local",
 "status": "active",
 "founded": "2015 (first commit 2015-12-04; v1.0 on 2016-10-19)",
 "hq": "No company; a volunteer open-source project under the beancount GitHub organisation. Started by Dominik Aumayr; maintained mainly by Jakob Schnitzer (yagebu, 2,518 of the commits).",
 "backing": "None found. No FUNDING.yml in the repository (checked 2026-10-10); translations hosted on Weblate.",
 "tagline": "An MIT-licensed web interface for Beancount that you run locally over your own ledger files.",
 "scope": "Fava reads Beancount files and shows balance sheets, income statements, journals, trial balances, holdings, budgets and charts, with filters by time, account and tag. It has a source editor, a form for adding transactions, a Beancount Query Language console with CSV, XLSX and ODS export, document attachments, an import page that runs beangulp importers, up-to-date indicators from balance assertions, and an extension system. It has no bank connectors, no built-in authentication and no hosted service; the ledger is whatever Beancount files you point it at.",
 "domains": [
  "ledger",
  "reporting",
  "documents",
  "personal"
 ],
 "customers_served": [
  "consumer",
  "freelancer",
  "smb",
  "developers"
 ],
 "spectrum": {
  "custody": {
   "score": 4,
   "note": "Your Beancount files are the record; Fava reads them and writes edits back to them."
  },
  "locality": {
   "score": 4,
   "note": "pip install fava; serves localhost:5000 on your machine."
  },
  "history": {
   "score": 1,
   "note": "Edits rewrite the file in place; a bundled auto_commit extension can commit each change to git."
  },
  "openness": {
   "score": 4,
   "note": "Open source under the MIT license."
  },
  "breadth": {
   "score": 2,
   "note": "Whatever Beancount holds (accounts, investments, prices, documents, budgets), but only what you import into it."
  },
  "autonomy": {
   "score": 1,
   "note": "Importers run from the Import page, but a person chooses and saves each entry."
  },
  "agent_readiness": {
   "score": 2,
   "note": "A working JSON API (including BQL queries and source writes guarded by a sha256 check) and a documented extension API, but the JSON API is internal to the UI; community MCPs only."
  },
  "integration_ease": {
   "score": 3,
   "note": "One pip install pointed at a file; documented options, extensions and import hooks."
  },
  "market": {
   "score": 1,
   "note": "Individuals and freelancers, with some small businesses keeping books in Beancount."
  }
 },
 "api": {
  "public": false,
  "style": "JSON API at /api/... used by the Svelte front end (query, journal, source get/put, add entries, documents, import extract); Python extension API (FavaExtensionBase)",
  "auth": "None built in; options include --host, --prefix, --read-only and --incognito; put a reverse proxy in front for access control",
  "limits": "None; local",
  "cost": "Free",
  "export": "BQL query results as CSV by default, XLSX and ODS with fava[excel]; the ledger is plain-text Beancount",
  "notes": "The module docstring says the JSON API is for the web interface's asynchronous functions, so it carries no stability promise. Writes to source require the sha256 of the version you read, which refuses edits to a file that changed underneath."
 },
 "pricing": {
  "model": "Free and open source",
  "entry": "Free",
  "top": "Free",
  "notes": "A public demo runs at fava.pythonanywhere.com. No paid tier or hosted service."
 },
 "agents": {
  "mcp": "community",
  "ai_features": "None.",
  "agent_writes": "Possible through the internal JSON API (add entries, put source) or by editing the Beancount files directly; --read-only disables all changes through Fava.",
  "approval": "None beyond the person reviewing in the UI; the import page lets a person pick which extracted entries to save, which works as a review step.",
  "notes": "Community MCP servers include beanquery-mcp (53 stars, BQL queries over a ledger), beancount-mcp by StdioA (11 stars, queries and transaction submission) and small Fava-backed ones (cookie223/beancount-fava-mcp, FavaMCPServer). beanquery-mcp warns that it may send ledger contents to a hosted model."
 },
 "integration": {
  "scope": "Beancount 3 files, with includes and multiple ledgers; beangulp importers via import-config; documents folders; extensions in Python with optional JavaScript modules.",
  "notes": "Fava 1.30.13 (2026-05-19) dropped Beancount 2. Since 1.30, duplicate detection on import is a hook you add. A GTK desktop wrapper (favagtk) exists separately."
 },
 "preference": {
  "scale": "2,583 GitHub stars, 413 forks, 100+ contributors (2026-10-10); about 39,400 PyPI downloads in the last month (pypistats, 2026-10-10; includes mirrors and CI). plaintextaccounting.org lists a Fava chat room of about 260 members.",
  "ratings": "No G2, Capterra or Trustpilot listing found; web search was unavailable for this pass, so absence is not confirmed.",
  "sentiment": "Widely treated as the default interface for Beancount; plaintextaccounting.org credits it with budget, multi-period and unrealized-gains reports. Open requests are refinements: external editor commands, filter presets, envelope budgeting, better import help. Steady releases over ten years; the bus factor is one main maintainer."
 },
 "timeline": [
  {
   "date": "2015-12-04",
   "event": "First commit"
  },
  {
   "date": "2016-02-15",
   "event": "v0.2.3, first version installable with pip"
  },
  {
   "date": "2016-10-19",
   "event": "v1.0"
  },
  {
   "date": "2024-10-09",
   "event": "v1.29: query results rendered in the front end, dark mode"
  },
  {
   "date": "2024-12-29",
   "event": "v1.30: Beancount 3 support, beanquery for queries, beangulp for imports, Svelte 5"
  },
  {
   "date": "2026-05-19",
   "event": "v1.30.13 drops Beancount 2; translations move to Weblate"
  },
  {
   "date": "2026-08-18",
   "event": "v1.30.16 published to PyPI"
  },
  {
   "date": "2026-10-07",
   "event": "Latest commit on main; unreleased work adds fiscal-year intervals and a rewritten date filter parser"
  }
 ],
 "relation": "kin",
 "relation_note": "The proof that a plain local file can have an interface people use for years. Fava views and edits one normalized Beancount ledger; ownpurse keeps the sources' raw facts, rebuilds the books and checks them against each source.",
 "borrow": "The sha256 precondition on writes (Fava's version of ownpurse's before_fp drift check); a --read-only switch and an --incognito mode that obscures numbers for screen sharing; up-to-date indicators driven by passing or failing balance assertions; a query console over the record with charts and export; an import review screen that shows the raw source row (__source__) beside each candidate entry.",
 "avoid": "Stripping the source row and all underscore metadata before saving, which loses provenance at the moment it matters; an unauthenticated server that can write the ledger; an internal API that agents end up depending on without a stability promise.",
 "sources": [
  {
   "title": "Fava documentation home",
   "url": "https://beancount.github.io/fava/"
  },
  {
   "title": "Fava getting started",
   "url": "https://beancount.github.io/fava/usage.html"
  },
  {
   "title": "Fava GitHub repository",
   "url": "https://github.com/beancount/fava"
  },
  {
   "title": "Fava CHANGES",
   "url": "https://github.com/beancount/fava/blob/main/CHANGES"
  },
  {
   "title": "Fava features help",
   "url": "https://github.com/beancount/fava/blob/main/src/fava/help/features.md"
  },
  {
   "title": "Fava import help",
   "url": "https://github.com/beancount/fava/blob/main/src/fava/help/import.md"
  },
  {
   "title": "Fava extensions help",
   "url": "https://github.com/beancount/fava/blob/main/src/fava/help/extensions.md"
  },
  {
   "title": "Fava JSON API source",
   "url": "https://github.com/beancount/fava/blob/main/src/fava/json_api.py"
  },
  {
   "title": "Fava CLI source",
   "url": "https://github.com/beancount/fava/blob/main/src/fava/cli.py"
  },
  {
   "title": "Fava on PyPI",
   "url": "https://pypi.org/project/fava/"
  },
  {
   "title": "pypistats: fava",
   "url": "https://pypistats.org/packages/fava"
  },
  {
   "title": "Fava demo",
   "url": "https://fava.pythonanywhere.com"
  },
  {
   "title": "plaintextaccounting.org",
   "url": "https://plaintextaccounting.org/"
  },
  {
   "title": "beanquery-mcp",
   "url": "https://github.com/vanto/beanquery-mcp"
  },
  {
   "title": "beancount-mcp (StdioA)",
   "url": "https://github.com/StdioA/beancount-mcp"
  }
 ],
 "unverified": [
  "Whether any G2, Capterra, Trustpilot or Reddit reviews exist; web search was unavailable in this pass.",
  "Share of Fava users who are small businesses rather than individuals.",
  "Where the maintainers are based.",
  "PyPI download counts as a measure of users (they include mirrors and CI)."
 ],
 "researched": "2026-10-10"
}
---

## What it is

[Fava](https://beancount.github.io/fava/) is a web interface for Beancount, described by its authors as having a focus on features and usability. You run `pip install fava`, then `fava ledger.beancount`, and open localhost:5000. It is MIT-licensed, started by Dominik Aumayr in December 2015 and maintained mostly by Jakob Schnitzer, who wrote about 2,500 of its commits. A [public demo](https://fava.pythonanywhere.com) shows the example ledger.

## How it works

Fava loads your Beancount files with Beancount itself, watches them for changes, and renders reports: balance sheet, income statement, trial balance, journal, holdings, budgets and charts, all filterable by time, account and tag. The [features help](https://github.com/beancount/fava/blob/main/src/fava/help/features.md) lists a source editor with auto-completion, a form to add transactions, a query page that runs the Beancount Query Language with CSV export (XLSX and ODS with `fava[excel]`), coloured up-to-date indicators based on whether the last entry for an account is a passing balance check, and switching between several ledgers.

The [import page](https://github.com/beancount/fava/blob/main/src/fava/help/import.md) runs beangulp importers over configured folders and lets you pick which extracted entries to save. Setting `__source__` metadata shows the raw CSV row or XML fragment beside each entry, but that key and every other underscore key are stripped before saving. [Extensions](https://github.com/beancount/fava/blob/main/src/fava/help/extensions.md) are Python classes with optional reports and JavaScript; the bundled `auto_commit` extension commits changes to git.

## Data and access

Your data is your Beancount files; Fava keeps no store of its own. Its [JSON API](https://github.com/beancount/fava/blob/main/src/fava/json_api.py) covers queries, the journal, source reads and writes, adding entries, documents and import extraction. Its own docstring says it serves the web interface, so treat it as internal. One detail stands out: `put_source` takes the sha256 of the version you read, so an edit to a file that changed meanwhile is refused. The [CLI](https://github.com/beancount/fava/blob/main/src/fava/cli.py) offers `--read-only` to disable all changes and `--incognito` to obscure numbers. There is no built-in authentication.

## Agents

Fava has no AI or agent features. Agents use the Beancount layer instead: community MCP servers such as [beanquery-mcp](https://github.com/vanto/beanquery-mcp) (BQL queries, with an explicit warning that ledger contents may go to a hosted model) and [beancount-mcp](https://github.com/StdioA/beancount-mcp) (queries and transaction submission), plus small servers that call Fava's API. Running Fava with `--read-only` next to an agent that edits files gives a person a live view, but nothing sits between an agent's write and the ledger.

## Customers

Beancount users, mostly individuals and freelancers, some small businesses. On 2026-10-10 the repository had 2,583 stars, 413 forks and more than 100 contributors, with about 39,400 PyPI downloads in the last month (pypistats; includes mirrors and CI). [plaintextaccounting.org](https://plaintextaccounting.org/) lists it as the Beancount web UI. Open issues are refinements, not complaints about reliability.

## History

First commit December 2015, pip-installable in February 2016, v1.0 in October 2016, then roughly two to four releases a year for a decade ([CHANGES](https://github.com/beancount/fava/blob/main/CHANGES)). v1.30 (December 2024) added Beancount 3, moved queries to beanquery and imports to beangulp, and made duplicate detection a manual hook. v1.30.13 (May 2026) dropped Beancount 2. The latest PyPI release is 1.30.16 (2026-08-18).

## For ownpurse

Fava is the strongest evidence that people will keep their books in a local file if the view over it is good, and it has held that position for ten years with almost no money behind it. Several of its choices match ownpurse's: the sha256 check before a write, a read-only switch, balance assertions surfaced as status, and an import screen that is in effect an approval step. The gap is the record. Fava shows a raw source row during import, then throws it away; it keeps one normalized ledger, and nothing checks it against Xero or the bank. ownpurse would keep that source row as a fact, and Fava-style reports become queries over projections of it.
