---
{
 "slug": "singer",
 "name": "Singer",
 "url": "https://www.singer.io",
 "segment": "connector-framework",
 "status": "active",
 "founded": "2016 (getting-started repository created 2016-10-31 by Stitch)",
 "hq": "No company of its own; sponsored by Stitch, now a Qlik product (Qlik headquarters not checked)",
 "backing": "Sponsored by Stitch. Talend acquired Stitch in 2018 and Qlik acquired Talend in 2023; Stitch's site now reads 'A Qlik product'. The largest independent ecosystem around Singer is Meltano, which its site describes in 2026 as an open-source project by Matatika.",
 "tagline": "An open specification for extraction scripts (taps) and loading scripts (targets) that pass JSON messages over standard output.",
 "scope": "Singer defines how a tap reads a config and optional state and writes SCHEMA, RECORD and STATE messages, one JSON object per line, for a target to load. It is a spec plus a family of open-source taps (such as tap-xero) and targets, not a service. It does not schedule, store, deduplicate, detect deletes or model anything; those are left to the runner (Stitch, Meltano) and the target.",
 "domains": [
  "integration"
 ],
 "customers_served": [
  "developers",
  "smb"
 ],
 "spectrum": {
  "custody": {
   "score": 4,
   "note": "A tap runs on your machine and writes to stdout; nothing passes through or stays with a vendor unless you choose Stitch."
  },
  "locality": {
   "score": 4,
   "note": "Taps and targets are small CLI programs (mostly Python) that run locally with a config and state file."
  },
  "history": {
   "score": 1,
   "note": "The spec carries time_extracted and a STATE bookmark but no versions or deletes; history depends entirely on the target."
  },
  "openness": {
   "score": 4,
   "note": "Open spec; tap-xero is AGPL-3.0, Meltano is MIT and the Meltano SDK Apache-2.0."
  },
  "breadth": {
   "score": 2,
   "note": "singer.io lists 106 taps and 10 targets (Meltano's hub claims 600+ connectors), but each is a separate pipe with no joining."
  },
  "autonomy": {
   "score": 0,
   "note": "Read-only extraction run by a person or a scheduler; nothing writes to the books."
  },
  "agent_readiness": {
   "score": 2,
   "note": "No MCP or API, but a CLI with JSON lines on stdout is easy for an agent to run and parse; Meltano Cloud shows Claude-callable tools."
  },
  "integration_ease": {
   "score": 2,
   "note": "The spec is simple and a tap is easy to write, but taps vary widely in quality and maintenance, and variants fork."
  },
  "market": {
   "score": 2,
   "note": "Data engineers at small and mid-sized companies, directly or through Stitch and Meltano."
  }
 },
 "api": {
  "public": true,
  "style": "No network API: a process protocol of newline-delimited JSON (SCHEMA, RECORD, STATE) on stdout, with config, state and catalog files on disk",
  "auth": "Whatever each tap needs, kept in its config file (tap-xero: OAuth client credentials and a refresh token)",
  "limits": "Set by each source; tap-xero raises a RateLimitException and backs off on Xero's limits",
  "cost": "Free to run yourself. Stitch, the hosted runner, starts at $100 a month (Standard, 5M to 300M rows); Advanced $1,500 and Premium $3,000 a month billed annually (pricing page, read 2026-10-10).",
  "export": "Everything a tap emits is plain JSON on stdout; the target decides the format (CSV, JSONL, Postgres, a warehouse).",
  "notes": "tap-xero (2.6.0, merged 2026-04-28) has paginated streams with an UpdatedDateUTC bookmark and a page offset written to state per page, Journals by JournalNumber offset, bank_transfers bookmarked by CreatedDateUTC, and 'pull everything' streams for endpoints without Modified-After. 2.6.0 removed the Employees, Expense Claims and Receipts streams."
 },
 "pricing": {
  "model": "Free and open source; paid hosting through Stitch (row-volume tiers) or support through Meltano",
  "entry": "$0 to run taps yourself; Stitch Standard from $100 a month",
  "top": "Stitch Premium at $3,000 a month billed annually (1B rows)",
  "notes": "Stitch offers a 14-day trial; its Xero integration is a Standard-tier, Stitch-certified integration built on tap-xero, version 1 released 2018-08-22."
 },
 "agents": {
  "mcp": "none",
  "ai_features": "None in Singer itself. Meltano's 2026 site shows Claude calling Meltano Cloud tools (list_workspaces, add_connector, run_pipeline) and says connectors can be managed by AI.",
  "agent_writes": "No. Taps only read; targets write to warehouses or files, not to accounting systems.",
  "approval": "n/a",
  "notes": "The stdout JSON protocol is easy for an agent to drive from a shell without any agent-specific layer."
 },
 "integration": {
  "scope": "singer.io lists 106 taps and 10 targets (read 2026-10-10); MeltanoHub claims 600+ sources and destinations.",
  "notes": "Writing a tap takes a config, a catalog and three message types; the Meltano SDK adds a framework for pagination, state and testing. Quality and upkeep vary tap by tap, and many exist in several forks ('variants')."
 },
 "preference": {
  "scale": "The getting-started repo has about 1,350 stars; tap-xero 22 stars; Meltano about 2,650 stars (GitHub, 2026-10-10). No adoption figures are published for Singer itself.",
  "ratings": "Not retrieved; Singer has no review-site listing of its own, and G2 and Capterra refused automated access.",
  "sentiment": "Not checked against review sites or forums for this dossier. Repository activity shows the Stitch-maintained taps still receive dependency and stream updates (tap-xero 2.6.0 in April 2026) while the spec itself has barely changed (v0.3.0)."
 },
 "timeline": [
  {
   "date": "2016-10-31",
   "event": "Stitch creates the Singer getting-started repository"
  },
  {
   "date": "2017-08-03",
   "event": "tap-xero repository created"
  },
  {
   "date": "2018-08-22",
   "event": "Stitch releases version 1 of its Xero integration on tap-xero"
  },
  {
   "date": "2018",
   "event": "Talend acquires Stitch"
  },
  {
   "date": "2020",
   "event": "Airbyte launches, citing Singer's spec as its starting point"
  },
  {
   "date": "2023",
   "event": "Qlik acquires Talend; Stitch becomes a Qlik product"
  },
  {
   "date": "2026-04-28",
   "event": "tap-xero 2.6.0 removes deprecated Employees, Expense Claims and Receipts streams"
  },
  {
   "date": "2026",
   "event": "Meltano, the main Singer runner and SDK, is described on its site as an open-source project by Matatika"
  }
 ],
 "relation": "kin",
 "relation_note": "Singer shares ownpurse's instincts: small local programs, plain JSON over stdout, state in a file you hold. tap-xero is a ready map of Xero's streams and their cursor quirks, and any Singer tap could be run as an ownpurse source, its output landing in the drawer with provenance.",
 "borrow": "The three-message protocol and key_properties and bookmark_properties per stream; tap-xero's split of Xero endpoints into paginated, non-paginated, Journals-by-number and pull-everything; writing a page offset to state on each page and advancing the bookmark only when the stream ends; ordering by UpdatedDateUTC ASC to make paging stable.",
 "avoid": "A spec with no way to say a record was deleted or voided; bookmarks on CreatedDateUTC, which never see a later delete; the bookmark as the only history; tap variants that fork and drift; mixing AGPL-3.0 tap code into ownpurse's own code without checking licence compatibility.",
 "sources": [
  {
   "title": "Singer",
   "url": "https://www.singer.io/"
  },
  {
   "title": "Singer specification",
   "url": "https://github.com/singer-io/getting-started/blob/master/docs/SPEC.md"
  },
  {
   "title": "singer-io/tap-xero",
   "url": "https://github.com/singer-io/tap-xero"
  },
  {
   "title": "tap-xero streams.py",
   "url": "https://github.com/singer-io/tap-xero/blob/master/tap_xero/streams.py"
  },
  {
   "title": "tap-xero PR 129: remove deprecated streams",
   "url": "https://github.com/singer-io/tap-xero/pull/129"
  },
  {
   "title": "Stitch pricing",
   "url": "https://www.stitchdata.com/pricing/"
  },
  {
   "title": "Stitch Xero documentation",
   "url": "https://www.stitchdata.com/docs/integrations/saas/xero"
  },
  {
   "title": "Meltano",
   "url": "https://meltano.com/"
  },
  {
   "title": "Airbyte Protocol acknowledgements",
   "url": "https://docs.airbyte.com/platform/understanding-airbyte/airbyte-protocol"
  }
 ],
 "unverified": [
  "Exact date and price of Talend's acquisition of Stitch (commonly reported as November 2018, $60M)",
  "Exact date Qlik completed its acquisition of Talend (2023 per secondary sources)",
  "When and how Matatika took over Meltano, and the date Meltano spun out of GitLab",
  "Whether Stitch's Xero integration runs the current tap-xero 2.6.0",
  "Qlik's headquarters location",
  "Community sentiment from Reddit, G2 or Capterra"
 ],
 "researched": "2026-10-10"
}
---

## What it is

Singer is an open standard for scripts that move data ([singer.io](https://www.singer.io/)). A tap extracts from a source and a target loads into a destination; you pipe one into the other. Stitch created it in 2016 and still sponsors it. The site lists 106 taps and 10 targets. Singer is not a company or a service: it is a [spec](https://github.com/singer-io/getting-started/blob/master/docs/SPEC.md) at v0.3.0 and a set of repositories.

## How it works

A tap reads a config file and an optional state file and writes JSON messages, one per line, to stdout. SCHEMA gives a stream's JSON Schema, its `key_properties` (the primary key) and optional `bookmark_properties`. RECORD carries one row and an optional `time_extracted`. STATE carries whatever the tap needs to resume; the runner saves it and passes it back next time. A catalog chooses streams and full-table or incremental replication. The spec says nothing about deletes or record versions, and that gap matters for books.

[tap-xero](https://github.com/singer-io/tap-xero) shows the details. Streams that support Modified-After, `order` and `page` (bank transactions, contacts, invoices, credit notes, manual journals, payments, overpayments, prepayments, purchase orders, quotes) are fetched in `UpdatedDateUTC ASC` order. The tap writes the page number to state on every page and moves the bookmark only once the stream ends, so a crash resumes mid-stream. Journals page by `JournalNumber` offset. Accounts, items and users use Modified-After without paging; bank transfers bookmark on `CreatedDateUTC`. Branding themes, currencies, tax rates, tracking categories and repeating invoices are pulled in full every run. Contacts add `includeArchived`. Version 2.6.0 (April 2026) dropped the deprecated Employees, Expense Claims and Receipts streams. The tap is AGPL-3.0.

## Data and access

Run yourself, Singer costs nothing and holds nothing: the output is plain JSON you route where you like. Stitch hosts the same taps from $100 a month for Standard, with Advanced at $1,500 and Premium at $3,000 a month billed annually ([pricing](https://www.stitchdata.com/pricing/), read 2026-10-10). Its Xero integration is Standard tier and Stitch-certified, version 1 released in August 2018 ([docs](https://www.stitchdata.com/docs/integrations/saas/xero)).

## Agents

Singer has no agent features and needs few: a CLI that emits JSON lines is already easy for an agent to run and read. [Meltano](https://meltano.com/), the main open-source runner, shows Claude calling Meltano Cloud tools to add connectors and run pipelines. Taps never write to a source.

## Customers

Data engineers at small and mid-sized companies, directly or through Stitch and Meltano. No adoption figures are published; the getting-started repo has about 1,350 GitHub stars.

## History

Singer was Stitch's attempt to open up the connector layer in 2016–17, and it set the shape the field still uses: Airbyte's protocol credits Singer as its starting point. Talend bought Stitch in 2018 and Qlik bought Talend in 2023. Meltano, spun out of GitLab, became the community's runner and SDK and is now run by Matatika. The spec has barely moved, while the energy went to its successors. Singer is a case study in a good protocol left without a steward.

## For ownpurse

Singer is the nearest in spirit to how ownpurse wants connectors to feel: a small local program, JSON on stdout, state in a file the owner holds. Take tap-xero's map of Xero's endpoints and their cursor quirks; it saves weeks of trial and error. Take its rule of checkpointing per page and advancing the bookmark only at the end. Then add what Singer leaves out: raw bytes kept, versions kept, voids and deletes as facts, and a completed sweep before anything is retracted. A Singer tap could run as an ownpurse source as it is, its output landing in the drawer with provenance.
