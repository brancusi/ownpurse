---
title: Concepts
description: The record, versions, organisations and profiles, sources, the local ledger, parity, bases, and why reads cost nothing.
sidebar:
  order: 4
---

## The record

The record is one SQLite file on your disk, for example `db/xero.sqlite`. It holds:

- **every version of every entity** ownpurse has seen: accounts, contacts, bank transactions, manual journals,
  invoices, credit notes, payments, transfers and the rest;
- **report snapshots**: the provider's own trial balance, balance sheet, profit and loss and bank summary, each
  stored as fetched;
- **sync runs**: one row per sync, with what it fetched and what vanished;
- **the call log**: every API call, with its status, duration and the remaining daily budget.

The record is **append-only**. Database triggers refuse every `UPDATE` and `DELETE` on the history tables, so the
only way the file changes is by adding rows. Nothing is removed locally: a deletion in the source arrives as a new
version with a deleted or voided status.

Every appended row is linked into a **SHA-256 chain**: each link hashes the row together with the previous link.
If anyone edits or removes a row after the fact, the chain no longer verifies.

**Amounts are exact.** ownpurse reads every number with its original digits and keeps money as decimals, never
as floating point. A total in ownpurse matches the source to the cent because no rounding ever happens on the way.

## Facts and versions

A **version** is a snapshot of one entity as the source reported it at one moment. ownpurse stores a new version
only when the content changes (it compares a content hash), together with:

- which organisation and entity it belongs to, and the entity's id;
- when ownpurse observed it, and when the source says it was last updated;
- **where it came from**: a `sync`, a `readback` after one of your own writes, or a `backfill`;
- which sync run brought it, and, for read-backs, which operation caused it.

The current state of the books is the latest version of each entity. The history is every version before it. A
new version that none of your own operations caused is **external drift**: someone changed the record in the
provider's web app.

## Organisations, books and profiles

An **organisation** (Xero calls it a tenant) is one set of books. You give each one a short **alias** in the
config, such as `roast` or `hold`, and type that alias everywhere. Most commands also accept a tenant id, a
comma list (`roast,hold`), a named **group** (`group:ops`) or `all`.

A **profile** is one provider app, one token file and one record. Most people need one. A second profile is
useful for a sandbox, such as Xero's Demo Company, kept in its own record. `--profile NAME` picks one; otherwise
`default_profile` applies.

## Sources

A **source** is anywhere facts come from. Today the only connector is Xero. Planned connectors include QuickBooks,
bank exports, spreadsheets, investment portfolios and plain files. Whatever the source, the rule is the same: the
connector appends what it saw, says where it came from, and never overwrites what is already there. See
[Writing a connector](/docs/developers/connectors/).

## The local ledger

Xero's general-ledger feed needs its Advanced plan, so ownpurse does not depend on it. It **derives postings from
the documents** in the record: bank transactions, transfers, manual journals, invoices, credit notes and payments.
From those postings it computes the general ledger (`gl`), the trial balance (`tb`), the profit and loss (`pl`)
and the balance sheet (`bs`).

### Parity with the provider's reports

A ledger rebuilt from documents is only useful if it agrees with the books. `parity` compares the local trial
balance with Xero's own, account by account, to the cent. `pl` and `bs` do the same against Xero's profit and
loss and balance sheet. Each check reads a report snapshot already in the record, so it spends no calls; you
fetch the snapshot once with `ownpurse report fetch`.

### Opening balances: year basis and full history

Some balances never appear in any document the API returns: conversion balances entered in Xero's opening-balance
screen, rounding from the reconcile screen, and automated entries such as depreciation or currency revaluation.
A ledger built only from documents can therefore disagree with Xero.

So ownpurse has two bases:

| Basis | How it opens | When to use it |
|---|---|---|
| Year basis (default) | From Xero's own trial balance at the prior fiscal year-end, plus local postings after it | Balances you can rely on |
| Full history (`--full-history`) | From local documents only, from the beginning | Seeing what the documents alone explain |

`--since DATE` opens from Xero's cached trial balance at any date you choose.

### Accrual and cash

By default every statement is on the **accrual** basis: an invoice counts when it is issued. With `--cash`, `pl`
and `bs` use the **cash** basis (Xero's `paymentsOnly`): invoices and credit notes post when they are paid, their
lines and tax in proportion to the amount paid, and manual journals marked not to show on cash-basis reports are
left out.

## Held, not posted

Some entries arrive before anyone knows what they are: a card charge with no project, a transfer whose other side
has not been found. The direction of the data model is to **hold** such entries in the record as facts, visible and
counted, without posting them to an account until they are classified. Classification today is a local layer of
labels on lines (project, bearer, treatment, the account a line should be on), kept in its own append-only store
and never sent to Xero. See [Writes and classification](/docs/guides/writes/).

## Reads cost zero API calls

Every read command answers from the record. Opening the TUI, moving around, drilling into a document and switching
organisations spend nothing.

Only these commands call the provider:

- `sync`, `login`, `orgs`
- `report fetch`
- `attachments` and `history` (each has `--cached` to read the record instead)
- the write commands that send or read back: `write run`, `write verify`, `write reconcile`, `write recover`,
  `write probe-replay` and `manifest apply`

`ownpurse --help` marks each of them with ⇅. `--offline` makes them refuse, so you can guarantee zero calls.

## Next

- [Configuration](/docs/guides/configuration/)
- [Reports](/docs/guides/reports/)
- [Data model](/docs/developers/data-model/)
