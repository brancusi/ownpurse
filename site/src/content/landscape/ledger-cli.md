---
{
 "slug": "ledger-cli",
 "name": "Ledger",
 "url": "https://ledger-cli.org",
 "segment": "plain-text-local",
 "status": "active",
 "founded": "August 2003",
 "hq": "No headquarters; an open-source project begun by John Wiegley, maintained by volunteers",
 "backing": "No company, investors or sponsorship programme found; volunteer-maintained under the ledger GitHub organization.",
 "tagline": "The original plain-text double-entry accounting tool: a C++ command-line reporter over a journal you write by hand.",
 "scope": "Ledger reads a text journal and reports balances, registers, budgets, forecasts and commodity values from the command line, with a value-expression language and automated transactions. It converts bank CSV to journal entries with its convert command. Its manual says it 'will never alter your input file': it only reads, so it has no write path, no bank connections and no versioning.",
 "domains": [
  "ledger",
  "personal",
  "reporting",
  "crypto"
 ],
 "customers_served": [
  "consumer",
  "freelancer",
  "developers"
 ],
 "spectrum": {
  "custody": {
   "score": 4,
   "note": "The journal is a text file you hold; Ledger never writes to it."
  },
  "locality": {
   "score": 4,
   "note": "A local C++ binary."
  },
  "history": {
   "score": 2,
   "note": "Edited in place, but 3.4.0 added --hashes, a SHA-512 chain over transactions that detects later edits; raw CSV lines are kept only with --rich-data."
  },
  "openness": {
   "score": 4,
   "note": "Open source under BSD-3-Clause."
  },
  "breadth": {
   "score": 1,
   "note": "One journal and CSV conversion; other sources rely on third-party tools."
  },
  "autonomy": {
   "score": 1,
   "note": "convert prints entries for you to append; Ledger itself never writes."
  },
  "agent_readiness": {
   "score": 1,
   "note": "A scriptable CLI with CSV, XML and Lisp output and Python bindings, but no JSON output, no API and no verified MCP server."
  },
  "integration_ease": {
   "score": 2,
   "note": "A documented, widely parsed format; the Python bindings are known to be hard to build and had many crash fixes pending in 3.5.0."
  },
  "market": {
   "score": 0,
   "note": "Technical individuals; no business offering."
  }
 },
 "api": {
  "public": true,
  "style": "CLI reports with custom --format strings; csv, xml, lisp/emacs output; Python bindings to the C++ core",
  "auth": "None; local files",
  "limits": "None; local",
  "cost": "Free",
  "export": "Everything: the journal is the data; reports as text, CSV, XML or Lisp.",
  "notes": "Latest release 3.4.1 (2025-10-26) after 3.4.0 (2025-10-22). The development NEWS lists an unreleased 3.5.0 with more than 575 merged pull requests, a C++17 migration and many Python-binding crash fixes."
 },
 "pricing": {
  "model": "Free open-source software",
  "entry": "Free",
  "top": "Free",
  "notes": "No paid tier or hosted service."
 },
 "agents": {
  "mcp": "none",
  "ai_features": "None. plaintextaccounting.org lists ledger-guesser, a neural network that suggests entries resembling past ones, used with ledger-autosync.",
  "agent_writes": "Ledger has no write path; an agent writes by appending text to the journal, often from convert's output.",
  "approval": "None built in. Because Ledger only reads, any gate is whatever the person or their editor imposes; --hashes can detect that history was changed afterwards.",
  "notes": "An agent can drive Ledger well as a read-only query engine over the journal: the CLI and value expressions are expressive, but outputs are text, CSV or XML rather than JSON."
 },
 "integration": {
  "scope": "Inputs: the Ledger journal and CSV via convert (with --auto-match, --invert, --account, --rich-data). Outputs: text reports, CSV, XML, Lisp, pricedb. Emacs ledger-mode is a separate project.",
  "notes": "With --rich-data, convert stores the original CSV line, the import date and a UUID that is a checksum of that line; an entry whose UUID is already in the journal is not printed again. Without it, the raw line is not kept."
 },
 "preference": {
  "scale": "6,053 GitHub stars and 548 forks (GitHub API and page, 2026-10-10); plaintextaccounting.org (2026) lists about 245 committers, a mailing list of about 900 and about 70 on IRC. No user counts are published.",
  "ratings": "No G2, Capterra or Trustpilot profiles were found.",
  "sentiment": "Respected as the origin of plain-text accounting and for its flexible value expressions and speed; its manual itself says it 'provides no bells or whistles' and points people who want an entry interface to GnuCash. Releases were sparse between 2023 and 2025."
 },
 "timeline": [
  {
   "date": "2003-08",
   "event": "John Wiegley begins Ledger."
  },
  {
   "date": "2011",
   "event": "Featured on FLOSS Weekly 150."
  },
  {
   "date": "2014-10-05",
   "event": "Ledger 3.1 (3.0 preceded it); payee uuid directive added."
  },
  {
   "date": "2020-05-01",
   "event": "Ledger 3.2.0."
  },
  {
   "date": "2023-02-08",
   "event": "Ledger 3.3, adding a debit column to convert."
  },
  {
   "date": "2025-10-22",
   "event": "Ledger 3.4.0: hash chaining with --hashes, commodity swaps, C++17 and Boost 1.86."
  },
  {
   "date": "2025-10-26",
   "event": "Ledger 3.4.1."
  },
  {
   "date": "2026-09-22",
   "event": "Latest push to the repository; 3.5.0 notes in progress on master."
  }
 ],
 "relation": "kin",
 "relation_note": "The origin of the idea ownpurse shares: your books as a file you own, read by a tool that never edits them. It keeps a hand-written journal, discards the raw source by default, and leaves history to you.",
 "borrow": "Read-only by design ('Ledger will never alter your input file'); a content checksum of the raw line as import identity (--rich-data UUID); a chained hash over transactions so the last hash vouches for the whole history (--hashes).",
 "avoid": "Discarding the raw record by default: without --rich-data, nothing of what the bank said survives conversion. Also avoid an integrity check that is opt-in and per-transaction metadata rather than built into the record.",
 "sources": [
  {
   "title": "Ledger home page",
   "url": "https://ledger-cli.org/"
  },
  {
   "title": "Ledger 3 manual (convert, --rich-data, --hashes)",
   "url": "https://ledger-cli.org/doc/ledger3.html"
  },
  {
   "title": "Ledger NEWS",
   "url": "https://github.com/ledger/ledger/blob/master/NEWS.md"
  },
  {
   "title": "Ledger releases on GitHub",
   "url": "https://github.com/ledger/ledger/releases"
  },
  {
   "title": "Ledger on GitHub",
   "url": "https://github.com/ledger/ledger"
  },
  {
   "title": "A comparison of Beancount and Ledger/hledger",
   "url": "https://beancount.github.io/docs/a_comparison_of_beancount_and_ledger_hledger/"
  },
  {
   "title": "plaintextaccounting.org",
   "url": "https://plaintextaccounting.org/"
  }
 ],
 "unverified": [
  "Whether any community MCP server for Ledger exists; web search was unavailable and none was found in the sources read.",
  "The release date of Ledger 3.0 (NEWS gives none).",
  "Who currently leads maintenance of Ledger; no maintainer statement was read.",
  "Sentiment comes from the manual, Beancount's comparison and project activity; Reddit could not be fetched and no review-site profiles were found."
 ],
 "researched": "2026-10-10"
}
---

## What it is

[Ledger](https://ledger-cli.org/) is "a powerful, double-entry accounting system" run from the command line over a plain-text journal. John Wiegley began it in August 2003; it is BSD-licensed C++ and the tool that hledger, Beancount and the other plain-text accounting projects descend from or react to. Its [manual](https://ledger-cli.org/doc/ledger3.html) is clear about scope: "Ledger will never alter your input file" and "Ledger is only for analyzing the data, not for altering it." It "provides no bells or whistles" and suggests GnuCash to anyone who wants an entry interface.

## How it works

You write transactions in a journal with any editor. Ledger parses it and answers questions: balances, registers, budgets, forecasts, values at market prices, with a value-expression language and automated transactions for anything else. It is optimistic where Beancount is strict: virtual postings are allowed, lots are not matched against earlier additions, and balance assertions follow file order, as Beancount's author [notes](https://beancount.github.io/docs/a_comparison_of_beancount_and_ledger_hledger/).

For bank data, `ledger convert` turns a CSV file into entries. With `--rich-data`, each entry carries the original CSV line, the import date and a `UUID` that is "a checksum of original csv line"; if an entry with that UUID is already in the journal, it is not printed again. That is content-based identity on the raw line, done right. Without the flag, the raw line is not kept.

Version 3.4.0 (2025-10-22, per the [NEWS](https://github.com/ledger/ledger/blob/master/NEWS.md)) added `--hashes`: a SHA-512 chain over transactions, built from each posting's account, amount and other details plus the previous transaction's hash, so that "the single hash value of the final transaction is sufficient to guarantee the shape of the entire history leading up to it". You record Hash metadata on some transactions and Ledger reports a mismatch.

## Data and access

The journal is yours and Ledger never writes to it. Reports come out as text, CSV, XML or Lisp, with custom `--format` strings; there is no JSON output and no network API. Python bindings expose the core, though Beancount's author found them hard to build, and the unreleased 3.5.0 notes list many fixes for crashes in them.

## Agents

Ledger has no AI features and no official MCP server; none from the community was verified for this dossier. [plaintextaccounting.org](https://plaintextaccounting.org/) lists ledger-guesser, a neural network that proposes entries resembling past ones. An agent can use Ledger as a capable read-only query engine and write by appending text, with no gate except the person and, now, a hash chain that shows if history was changed afterwards.

## Customers

Technical individuals. The repository has 6,053 GitHub stars (2026-10-10); plaintextaccounting.org (2026) counts about 245 committers and a mailing list of about 900, the largest of the three. There are no review-site profiles. Releases slowed: 3.3.2 in March 2023, then 3.4.0 and 3.4.1 in October 2025, with a large 3.5.0 (more than 575 merged pull requests, C++17) in progress.

## History

Ledger 2.x added XML output that Ledger could also read back. 3.0 changed defaults; 3.1 came 2014-10-05; 3.2.0 on 2020-05-01; 3.3 on 2023-02-08. hledger (2007) began as a Haskell take on it and Beancount (2008) as a stricter one.

## For ownpurse

Ledger is kin and the original statement of the idea: your books are a file you own and the tool only reads it. Three of its choices line up with ownpurse: never write the owner's data, identify an import by a checksum of the raw line rather than by date, and chain hashes so the last one vouches for the whole history.

Where it stops is where ownpurse starts. The raw record is discarded unless you ask; the chain is opt-in metadata on a file you can still edit; and every source must first be flattened into one journal by hand or by script. ownpurse keeps what each source reported as facts in an append-only, chained log by default and derives the journal, which a Ledger user could still export and read with Ledger.
