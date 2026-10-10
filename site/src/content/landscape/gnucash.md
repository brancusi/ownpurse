---
{
 "slug": "gnucash",
 "name": "GnuCash",
 "url": "https://www.gnucash.org",
 "segment": "plain-text-local",
 "status": "active",
 "founded": "1997 (programming began; first stable release 1998)",
 "hq": "No headquarters; a volunteer project with no company behind it",
 "backing": "No company, funding or parent. Volunteers maintain it under the GNU GPL. It grew out of Robin Clark's X-Accountant and Linas Vepstas's work (Gnumatic).",
 "tagline": "A free desktop double-entry accounting app for people and small businesses. It keeps your books in a file on your own machine.",
 "scope": "Double-entry books with stock, bond and fund accounts, scheduled transactions, small-business invoicing and bills, reports, and bank import (QIF, OFX, CSV, and AqBanking for HBCI/FinTS, MT940 and CAMT). It stores the books as gzipped XML or in SQLite, MySQL or PostgreSQL. It has no web or mobile client, no multi-user support and no network API, and it does not keep source documents or the raw import beyond the entries it creates.",
 "domains": [
  "ledger",
  "bank",
  "invoicing",
  "reporting",
  "personal"
 ],
 "customers_served": [
  "consumer",
  "freelancer",
  "smb"
 ],
 "spectrum": {
  "custody": {
   "score": 4,
   "note": "The books are a file (or a database) that you hold. No vendor account is involved."
  },
  "locality": {
   "score": 4,
   "note": "A desktop app for Linux, Windows and macOS that runs fully offline."
  },
  "history": {
   "score": 1,
   "note": "Current state. An edit overwrites the transaction. Voiding keeps a record, but there are no versions and no as-of queries."
  },
  "openness": {
   "score": 4,
   "note": "GPL-2.0/3.0 open source, with documented XML and SQL storage formats."
  },
  "breadth": {
   "score": 2,
   "note": "Books, bank import, invoicing and investments in one file. It has no documents, payments or live connections beyond AqBanking."
  },
  "autonomy": {
   "score": 1,
   "note": "A person enters and imports everything. The Bayesian matcher suggests transfer accounts and matches, and a person confirms them."
  },
  "agent_readiness": {
   "score": 1,
   "note": "No API. Agents reach the books through Python bindings, the SQL backend or a small community MCP server."
  },
  "integration_ease": {
   "score": 1,
   "note": "To integrate, you read the file or SQL schema directly or script with bindings. Nothing is hosted and there are no tokens or docs for third parties."
  },
  "market": {
   "score": 1,
   "note": "Individuals, households and small businesses."
  }
 },
 "api": {
  "public": false,
  "style": "No network API. Access is through the file format (XML or SQL schema), Scheme report scripting, C/C++ library bindings for Python, and third-party readers such as piecash for the SQL backend.",
  "auth": "None. Whoever can read the file can read the books.",
  "limits": "No rate limits. The documented storage table says no format supports multi-user use, so concurrent writers are unsafe.",
  "cost": "Free. There is no metering or egress.",
  "export": "The whole book is the file. Gzipped XML is plain text, and the SQL backends are ordinary databases. The app also exports CSV (accounts, transactions) and TXF for US tax software.",
  "notes": "The Python bindings are part of the source tree. Not every packaged build ships them, which matters for agent use."
 },
 "pricing": {
  "model": "Free, open source",
  "entry": "$0",
  "top": "$0",
  "notes": "There is no paid tier or hosted service. The GnuCash project lists no paid support."
 },
 "agents": {
  "mcp": "community",
  "ai_features": "None in the product.",
  "agent_writes": "Only through third-party tools. The community ninetails-io/gnucash-mcp (MIT, 77 GitHub stars, v1.5.0, October 2026) reads and writes a local book over SQLite, PostgreSQL or MySQL. It covers transactions, invoices, budgets, scheduled transactions and reconciliation.",
  "approval": "GnuCash has no approval gate. The MCP server snapshots the book before a session's first write, refuses changes to reconciled splits without an override, and dry-runs statement imports. It logs every write to a daily plain-text audit file. Confirmation happens in the conversation, not through a sealed package.",
  "notes": "The community MCP server's safeguards (a snapshot, an audit file, voids preferred over deletes) are a lightweight version of what ownpurse makes structural."
 },
 "integration": {
  "scope": "Imports QIF, OFX/QFX and CSV. Online banking runs through AqBanking (HBCI/FinTS, MT940/942, CAMT 052, DTAUS). Price quotes come through Finance::Quote, and Finnhub.io was added as a source in 5.16 (June 2026). There are no connectors to cloud accounting systems.",
  "notes": "Imports run through a matcher that searches about ±42 days around each imported line. It scores candidates on date, amount, transaction id, check number, description and memo, then marks each line Clear (already present), Update and Clear, or Add. A Bayesian token matcher learns transfer accounts for each source account. Lines it cannot assign go to an Imbalance account."
 },
 "preference": {
  "scale": "SourceForge counted 6,273 downloads in the week before 2026-10-10. The GitHub mirror has 4.4k stars and 993 forks. Development is on the project's own servers.",
  "ratings": "SourceForge: 4.4/5 from 122 reviews (category scores: ease 3, features 4, design 3, support 3), checked 2026-10-10. G2 and Capterra blocked automated access, so those figures were not checked.",
  "sentiment": "Reviewers praise that it is free, reliable and lock-in free, and they like the double-entry rigour, multi-currency support and OFX import. Complaints: a dated, low-contrast interface, a learning curve, QIF imports that turn Quicken categories into accounts, occasional installer problems on Windows, no reconciliation report, no direct connection to some countries' banks, and a weak mobile story."
 },
 "timeline": [
  {
   "date": "1997",
   "event": "Programming begins, from Robin Clark's X-Accountant (xacc) code."
  },
  {
   "date": "1998",
   "event": "First stable release. The xacc project is renamed GnuCash."
  },
  {
   "date": "2001",
   "event": "Small-business accounting features added."
  },
  {
   "date": "2004",
   "event": "Mac installer."
  },
  {
   "date": "2007",
   "event": "Windows port."
  },
  {
   "date": "2023-03-26",
   "event": "GnuCash 5.0 released."
  },
  {
   "date": "2025-12-21",
   "event": "5.14 adds US savings bonds via Finance::Quote and a Postponed column for scheduled transactions."
  },
  {
   "date": "2026-06-28",
   "event": "5.16 adds Finnhub.io as a quote source."
  },
  {
   "date": "2026-09-27",
   "event": "5.17 released. Releases come at about the end of each calendar quarter."
  },
  {
   "date": "2026-09",
   "event": "A community GnuCash MCP server is added to the awesome-mcp-servers list."
  }
 ],
 "relation": "kin",
 "relation_note": "It has the same sovereignty premise: the books are a file you hold. But GnuCash is the ledger itself and is entered by hand. It keeps normalized entries and discards the raw import. ownpurse sits beside the ledgers, keeps every source's raw facts, and could read a GnuCash file as one more source.",
 "borrow": "The import matcher's three outcomes (already present, update and clear, add), shown with a confidence bar and a picker a person can override. Bayesian account guesses scoped to each source account. Voiding instead of deleting. A documented, open storage format, so the books outlive the app.",
 "avoid": "Overwriting edits in place, with no versions. Sending unassigned lines to an Imbalance account, so the guess becomes a posting before anyone decides. Having no API, so every integration has to reverse-engineer a file or schema. Allowing only a single writer and no remote access.",
 "sources": [
  {
   "title": "GnuCash home page",
   "url": "https://www.gnucash.org"
  },
  {
   "title": "GnuCash news (release dates)",
   "url": "https://www.gnucash.org/news.phtml"
  },
  {
   "title": "GnuCash older announcements (5.0)",
   "url": "https://www.gnucash.org/oldnews.phtml"
  },
  {
   "title": "GnuCash guide: storage formats",
   "url": "https://www.gnucash.org/docs/v5/C/gnucash-guide/basics-files1.html"
  },
  {
   "title": "GnuCash manual: importing transactions and the matcher",
   "url": "https://www.gnucash.org/docs/v5/C/gnucash-manual/trans-import.html"
  },
  {
   "title": "GnuCash on Wikipedia",
   "url": "https://en.wikipedia.org/wiki/GnuCash"
  },
  {
   "title": "GnuCash GitHub mirror",
   "url": "https://github.com/Gnucash/gnucash"
  },
  {
   "title": "ninetails-io/gnucash-mcp",
   "url": "https://github.com/ninetails-io/gnucash-mcp"
  },
  {
   "title": "GnuCash reviews on SourceForge",
   "url": "https://sourceforge.net/projects/gnucash/reviews/"
  },
  {
   "title": "GnuCash 5.16 announcement (mailing list)",
   "url": "https://lists.gnucash.org/pipermail/gnucash-user/2026-June/120999.html"
  }
 ],
 "unverified": [
  "Release dates of 2.4 (the SQL backend, believed to be around December 2010), 3.0 (believed 2018) and 4.0 (believed 2020) were not confirmed. The older-news page was truncated.",
  "Whether the Python bindings ship in the official Windows and macOS builds.",
  "G2 and Capterra ratings for GnuCash. Both sites blocked automated access.",
  "Whether GnuCash's per-session .log transaction journal could serve as an edit history. It was not checked in current docs.",
  "Whether the GnuCash project accepts donations or has any fiscal sponsor."
 ],
 "researched": "2026-10-10"
}
---

## What it is

[GnuCash](https://www.gnucash.org) is a free double-entry accounting app for Linux, Windows and macOS. Programming began in 1997 and the first stable release came in 1998, so it is one of the longest-lived open-source finance tools. It covers personal books, investments (stock, bond and fund accounts, with price quotes) and small-business work such as invoices, bills, customers and vendors. It is released under the GNU GPL by volunteers, with no company behind it.

## How it works

You keep your books in one file. The [default format is gzipped XML](https://www.gnucash.org/docs/v5/C/gnucash-guide/basics-files1.html). SQLite, MySQL and PostgreSQL also work, and they save on every commit instead of on command. The same page says no format supports multi-user use.

You bring data in by hand or by import. The [import matcher](https://www.gnucash.org/docs/v5/C/gnucash-manual/trans-import.html) handles QIF, OFX/QFX, CSV and AqBanking formats (HBCI/FinTS, MT940, CAMT 052). It searches about 42 days either side of each imported line. It scores candidates on date, amount, transaction id, check number, description and memo, then marks each line as already present (Clear), present but different (Update and Clear) or new (Add). A Bayesian matcher learns which transfer account each token tends to belong to, separately for each source account. A good match turns green, a weak one waits for you, and anything you do not assign goes to an Imbalance account. You can override every decision.

Releases come at about the end of each calendar quarter. 5.17 shipped on [27 September 2026](https://www.gnucash.org/news.phtml).

## Data and access

There is no network API. You reach the data through the file itself, the SQL schema, Scheme scripting for reports, or the C library's Python bindings. Third-party readers such as piecash work against the SQL backend. Nothing is metered and nothing costs money: whoever holds the file holds the books. Export is the file plus CSV and TXF. The open format is the real export, because the books can outlive the app.

Editing a transaction overwrites it. Voiding keeps a record of the void, but there are no prior versions to ask for and no as-of view.

## Agents

GnuCash has no AI features and no official agent interface. The community [gnucash-mcp](https://github.com/ninetails-io/gnucash-mcp) server (MIT, 77 stars, v1.5.0 as of October 2026) lets an assistant read and write a local book. It snapshots the book before a session's first write and refuses changes to reconciled splits without an override. It prefers voids to deletes, dry-runs statement imports and writes a daily plain-text audit file. Approval happens in the conversation: there is no sealed package and no separate log that a person signs off.

## Customers

It is used by individuals, households and small businesses who want double-entry rigour without a subscription. On [SourceForge](https://sourceforge.net/projects/gnucash/reviews/) it rates 4.4/5 from 122 reviews, with 6,273 downloads in the week before 10 October 2026. Reviewers praise its reliability, its lack of lock-in, multi-currency support and OFX import. They complain about the dated interface, a learning curve, QIF category mapping, occasional Windows installer trouble and weak mobile support.

## History

GnuCash began as Robin Clark's X-Accountant and was renamed when it joined the GNU project. Small-business features arrived in 2001, a Mac installer in 2004 and a Windows port in 2007 (per [Wikipedia](https://en.wikipedia.org/wiki/GnuCash)). 5.0 was announced on [26 March 2023](https://www.gnucash.org/oldnews.phtml). An Android companion app was discontinued. A separate Flutter app, GnuCash Mobile (2022), is independent of the project. It has survived nearly three decades without funding because it is a file format and a desktop app, not a service.

## For ownpurse

GnuCash shows that people will hold their own books for decades if the format is open and the app is free. It also shows the limits of a local ledger kept by hand. Every bank line is entered or imported by a person. The raw import is gone once matched, edits overwrite, and nothing outside the file can talk to it safely. ownpurse is not a replacement ledger. A GnuCash file is a source it could read: the XML or SQLite books become facts beside a person's Xero organisation and bank feeds. The matcher's Clear / Update / Add outcomes, with a visible confidence bar, are a good model for how ownpurse should present proposed links. The Imbalance account is the opposite of staying uncollapsed. ownpurse should hold an unassigned line as undecided rather than post it to a holding account.
