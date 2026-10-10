---
{
 "slug": "beangulp",
 "name": "beangulp",
 "url": "https://github.com/beancount/beangulp",
 "segment": "plain-text-local",
 "status": "active",
 "founded": "2021 as a separate repository (created 2021-02-02); the code descends from beancount.ingest, copyright 2016 (Martin Blais)",
 "hq": "No company; part of the Beancount open-source project. Main authors Martin Blais and Daniele Nicolodi.",
 "backing": "None. Volunteer project under the beancount GitHub organisation; Beancount's own FUNDING.yml points to Martin Blais's GitHub Sponsors and PayPal (checked 2026-10-10).",
 "tagline": "The Python framework Beancount uses to turn downloaded statements into ledger entries: identify the file, extract entries, mark likely duplicates and file the original away.",
 "scope": "beangulp defines an Importer interface and a command-line driver with three commands: identify (which importer handles a file), extract (produce Beancount entries, marking likely duplicates against an existing ledger) and archive (move the original into a dated, account-named filing tree). It ships a CSV importer base and a file-only importer; every bank-specific importer is written by users. It does not fetch data from banks, does not write to the ledger itself, and has no UI.",
 "domains": [
  "integration",
  "documents",
  "bank",
  "ledger"
 ],
 "customers_served": [
  "developers",
  "consumer",
  "freelancer"
 ],
 "spectrum": {
  "custody": {
   "score": 4,
   "note": "Reads files you downloaded, writes entries to your output and moves originals into your own folder."
  },
  "locality": {
   "score": 4,
   "note": "A Python library and CLI that runs on your machine; no network."
  },
  "history": {
   "score": 2,
   "note": "Original documents are archived unchanged under a dated name; the extracted entries themselves are not versioned."
  },
  "openness": {
   "score": 4,
   "note": "Open source under GPL-2.0."
  },
  "breadth": {
   "score": 1,
   "note": "A framework for one job, file import; coverage is whatever importers you write or find."
  },
  "autonomy": {
   "score": 1,
   "note": "Importers run unattended, but output goes to a file or stdout for a person to review and merge."
  },
  "agent_readiness": {
   "score": 2,
   "note": "A small documented Python interface and a scriptable CLI; text output only, no JSON, no MCP. An agent can write an importer quickly."
  },
  "integration_ease": {
   "score": 3,
   "note": "Subclass Importer and implement identify() and account(); docs are docstrings and examples, thin but adequate."
  },
  "market": {
   "score": 0,
   "note": "Technical individuals keeping personal or freelance books in Beancount."
  }
 },
 "api": {
  "public": true,
  "style": "Python library (abstract base class Importer) plus a click-based CLI: identify, extract, archive",
  "auth": "None; local files",
  "limits": "Files over a size threshold are skipped by identify",
  "cost": "Free",
  "export": "Output is Beancount text; originals stay as files in your archive",
  "notes": "Last tagged release v0.2.0 (2025-01-20) while master kept changing until 2026-05-30; an open issue asks for an installation strategy without new releases."
 },
 "pricing": {
  "model": "Free and open source",
  "entry": "Free",
  "top": "Free",
  "notes": "No paid tier or hosted service."
 },
 "agents": {
  "mcp": "none",
  "ai_features": "None in beangulp. The separate smart_importer library adds machine-learned account prediction on top of beangulp importers.",
  "agent_writes": "An agent can write importers and run extract; the output is text a person merges into the ledger.",
  "approval": "Implicit: extract writes candidate entries, with duplicates commented out, for a person to review before adding them to the ledger.",
  "notes": "Several community MCP servers query Beancount ledgers (beanquery-mcp, beancount-mcp); none wraps beangulp's import pipeline as far as this pass found."
 },
 "integration": {
  "scope": "Anything with a downloadable file: CSV, OFX, PDF, XML, via user-written importers. Bundled: a CSV importer base and a file-only importer. Fava runs beangulp importers from its Import page.",
  "notes": "Third-party collections build on it, for example beancount_reds_importers and beancount-import (listed on plaintextaccounting.org). With Beancount 3 and Fava 1.30, deduplication is a hook you add, not automatic in Fava."
 },
 "preference": {
  "scale": "145 GitHub stars, 62 forks, 43 contributors (2026-10-10). PyPI: about 47,100 downloads in the last month (pypistats, 2026-10-10; includes mirrors and CI).",
  "ratings": "No review-site listings; it is a developer library.",
  "sentiment": "Open issues ask for configurable CSV reading, better deduplication (prices, inferred amounts, not merging deduped entries into existing ones) and same-day ordering. The gap between releases and master is a stated pain point. Documentation is in docstrings and examples rather than a manual."
 },
 "timeline": [
  {
   "date": "2016",
   "event": "beancount.ingest written by Martin Blais (copyright date in the source); identify, extract and file commands in Beancount 2"
  },
  {
   "date": "2021-02-02",
   "event": "beangulp repository created as the successor to beancount.ingest"
  },
  {
   "date": "2024-05-29",
   "event": "v0.1.0 and v0.1.1 published to PyPI"
  },
  {
   "date": "2024-06-22",
   "event": "Beancount 3.0.0 released; beangulp replaces beancount.ingest"
  },
  {
   "date": "2024-12-29",
   "event": "Fava 1.30 uses beangulp for Beancount 3; duplicate detection becomes a manual hook"
  },
  {
   "date": "2025-01-20",
   "event": "v0.2.0 released"
  },
  {
   "date": "2026-05-19",
   "event": "Fava 1.30.13 drops Beancount 2, so Fava users must move importers to beangulp"
  },
  {
   "date": "2026-05-30",
   "event": "Latest commit on master"
  }
 ],
 "relation": "design-cousin",
 "relation_note": "The clearest existing shape for a file-based connector: recognize the file, extract entries, keep the original. ownpurse's drawer and projectors do the same steps, but keep the extracted facts and the original in one record instead of handing text to a person to paste.",
 "borrow": "The importer interface (identify, account, date, filename, extract, deduplicate) as the template for ownpurse file connectors; mark duplicates, do not drop them, and point at the entry they match (beangulp comments them out with '; duplicate of file:line'); archive the original under a stable, dated name; an explicit, replaceable comparator (default: same accounts, amounts within 5%, dates within two days).",
 "avoid": "Fuzzy matching as the only identity mechanism; losing the link between an entry and the file it came from unless the importer adds metadata; text output that a person pastes in, which leaves no record of what was imported when; drift between releases and master.",
 "sources": [
  {
   "title": "beangulp repository and README",
   "url": "https://github.com/beancount/beangulp"
  },
  {
   "title": "beangulp Importer interface (importer.py)",
   "url": "https://github.com/beancount/beangulp/blob/master/beangulp/importer.py"
  },
  {
   "title": "beangulp duplicate marking (extract.py)",
   "url": "https://github.com/beancount/beangulp/blob/master/beangulp/extract.py"
  },
  {
   "title": "beangulp similarity heuristic (similar.py)",
   "url": "https://github.com/beancount/beangulp/blob/master/beangulp/similar.py"
  },
  {
   "title": "beangulp on PyPI",
   "url": "https://pypi.org/project/beangulp/"
  },
  {
   "title": "Beancount on PyPI (release dates)",
   "url": "https://pypi.org/project/beancount/"
  },
  {
   "title": "pypistats: beangulp",
   "url": "https://pypistats.org/packages/beangulp"
  },
  {
   "title": "Fava CHANGES",
   "url": "https://github.com/beancount/fava/blob/main/CHANGES"
  },
  {
   "title": "Fava import help",
   "url": "https://github.com/beancount/fava/blob/main/src/fava/help/import.md"
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
  "The year beancount.ingest first shipped in a Beancount release (2016 is the copyright date in beangulp's source).",
  "Whether any community MCP server wraps beangulp's identify, extract and archive commands; GitHub repository search found none, and web search was unavailable.",
  "PyPI download counts as a measure of users (they include mirrors and CI)."
 ],
 "researched": "2026-10-10"
}
---

## What it is

[beangulp](https://github.com/beancount/beangulp) is the import framework of Beancount, the plain-text double-entry tool. Its README describes it as a framework for importing transactions into a Beancount ledger from account statements and other documents, and for managing those documents. It is the evolution of `beancount.ingest` from Beancount 2.3 and replaced it in [Beancount 3.0](https://pypi.org/project/beancount/) (released 2024-06-22). It is GPL-2.0 and written mainly by Martin Blais and Daniele Nicolodi.

## How it works

You write one Python class per source. The [Importer base class](https://github.com/beancount/beangulp/blob/master/beangulp/importer.py) asks for two methods, `identify(filepath)` (is this file mine?) and `account(filepath)` (which account does it belong to?), and offers optional `date`, `filename`, `extract`, `deduplicate` and `sort`. A small script registers your importers with `beangulp.Ingest`, which gives you three commands:

- `identify` walks a downloads folder and says which importer claims each file.
- `extract` runs the importers and prints Beancount entries, in sections per source file. With `--existing ledger.beancount` it compares each new entry with existing ones dated within two days either side. The default comparator treats two transactions as the same if they share accounts and the amounts are within 5%. Likely duplicates get a `__duplicate__` mark and are printed commented out, with a `; duplicate of file:line` pointer.
- `archive` moves each original into a tree that mirrors the account name, renamed `YYYY-MM-DD.name`.

The package ships a CSV importer base and a file-only importer. Bank-specific importers live in users' own repositories and community collections.

## Data and access

Everything is local files. Input is whatever you downloaded; output is Beancount text and an archive folder. There is no server, no network access, no JSON output. The interface is documented in docstrings and an examples directory; the README points to the older `beancount.ingest` docs as partly relevant.

## Agents

There are no agent features, but the shape suits agents. An importer is a small class with two required methods, so an agent can write one for a new bank export in minutes and test it with `identify` and `extract`. The review step is built in: extract produces candidate entries and a person merges them. [Fava](https://beancount.github.io/fava/) runs the same importers from its Import page, which gives that review a UI. Community MCP servers such as [beanquery-mcp](https://github.com/vanto/beanquery-mcp) and [beancount-mcp](https://github.com/StdioA/beancount-mcp) query the ledger afterwards; this pass found none that wraps beangulp itself.

## Customers

Beancount users who write some Python. Signals on 2026-10-10: 145 stars and 43 contributors on GitHub, about 47,100 PyPI downloads in the last month according to pypistats (a count that includes mirrors and CI). Open issues ask for configurable CSV reading and better deduplication, and one asks how to install without new releases: the last tag is v0.2.0 (2025-01-20) while master changed until 2026-05-30.

## History

The identify, extract and file workflow existed as `beancount.ingest` from 2016. beangulp was split out in February 2021, reached PyPI as v0.1.0 in May 2024, became the importer system for Beancount 3 in June 2024, and released v0.2.0 in January 2025. Fava 1.30 (December 2024) switched to beangulp for Beancount 3 and made deduplication an explicit hook; Fava 1.30.13 (May 2026) dropped Beancount 2, leaving beangulp as the only path.

## For ownpurse

This is the closest prior art for ownpurse's file connectors, and much of it should be copied: recognizing a file by content rather than by name, a stable archive name for the original, and duplicates marked with a pointer instead of silently dropped. Where ownpurse departs is in what happens next. beangulp's output is text a person pastes in, and only the importer's own choices link an entry back to its file. A fuzzy match is the only defence against importing twice. ownpurse stores the original by SHA-256 in the drawer, records facts with provenance, and treats a match between sources as a link rule with a confidence, so two witnesses to the same payment stay two facts rather than one being deleted.
