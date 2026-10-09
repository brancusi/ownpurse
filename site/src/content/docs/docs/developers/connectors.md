---
title: Writing a connector
description: What a connector must do to bring a new financial source into the ownpurse record, and how the Xero connector does it today.
sidebar:
  order: 3
---

A **connector** brings facts from one source into the record. Xero is the first connector. QuickBooks, bank exports,
spreadsheets, investment portfolios and plain files are planned.

The connector interface for new sources is being designed. This page describes what any connector has to do, and how
the Xero connector meets each responsibility today. It does not describe a stable API yet.

## Responsibilities

A connector must:

1. **Read a source.** Pull what the source holds: documents, balances, reports, files. It never changes the source
   while reading.
2. **Append, never overwrite.** Each observation becomes a new row. If the content has not changed, nothing is
   written. If it has, a new version is added beside the old one. Existing rows are never updated or deleted; the
   record's triggers refuse it anyway.
3. **Record provenance.** Every row says which source and which organisation it came from, when it was observed,
   when the source says it last changed, and which run brought it.
4. **Keep exact values.** Amounts keep the source's own digits, as decimals. No floating point on the way in.
5. **Respect the source's limits.** Rate limits, daily budgets and paging rules are the source's, and the connector
   must stay within them. When a budget runs out, it stops cleanly and keeps what it already fetched.
6. **Log every call.** Each request, with its status, duration and remaining budget, goes into the call log.
7. **Notice what disappeared.** A source may delete things without saying so. A connector that can list everything
   should do a full sweep from time to time and report ids that vanished, without removing anything locally.
8. **Join the chain.** Every row it appends is linked into the record's SHA-256 chain.

## How the Xero connector does it

**Reading.** `ownpurse sync <org> [Entity…]` pulls each entity from Xero's Accounting API. Settings tables
(accounts, tax rates, tracking categories, currencies) are pulled in full. Transaction tables use If-Modified-Since
from the last high-water mark, with a 5-second overlap, 1,000 records per page, and a full sweep every 7 days to
catch deletions and edits that don't move the modified date. `--reference` syncs only settings tables and
`--transactions` only transaction tables.

**Appending.** Each record is hashed. A row goes into `versions` only when its content hash differs from the latest
one for that record, with `source = sync` and the sync run's id. Writes that ownpurse itself makes are read back and
stored with `source = readback` and the operation that caused them.

**Reports.** `ownpurse report fetch` stores a report as an immutable snapshot in `reports`, one call per org.

**Budget.** Xero allows 60 calls a minute and 1,000 a day per organisation. The connector pauses briefly after 55
calls in a minute (`limits.minute_soft_limit`), reads `X-DayLimit-Remaining` from every response, and before a sync
estimates the calls it will need from the record. If an org reaches its daily limit, the sync stops at once and keeps
what it fetched. Every limit is a setting; see [configuration](/docs/guides/configuration/#limits).

**Call log.** Every request goes into `http_log` with its status, duration and remaining budget. `ownpurse usage`
and the TUI's Activity tab read it.

**Vanished records.** Ids that a full sweep no longer returns are listed as `vanished` on the sync run. Nothing is
removed from the record.

**Auth.** OAuth 2.0 with PKCE, read scopes by default, tokens in a file only the owner can read, refreshed under a
cross-process lock. See [Security](/docs/developers/security/).

## What a new connector will need to decide

- How the source identifies an entity, so successive observations become versions of the same thing.
- What counts as a change, so unchanged content writes nothing.
- How to page and how to sweep, within the source's limits.
- What the source's own reports are, if any, so the local ledger can be checked against them.

If you want to work on a connector, open an issue on [GitHub](https://github.com/brancusi/ownpurse) first, so the
design can take your source into account. See [Contributing](/docs/developers/contributing/).
