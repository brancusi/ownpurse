---
title: Reports
description: Trial balance, general ledger, profit and loss, balance sheet, parity with Xero, fetching Xero's reports, and CSV exports.
sidebar:
  order: 2
---

ownpurse computes your statements from the local ledger, the postings it derives from the documents in the record.
Every statement below is a read: it spends zero API calls. The provider's own reports come into the record only
through `report fetch`, and the statements check themselves against those snapshots.

All amounts are exact decimals. Add `--json` to any command for machine-readable output, `--full` for every row and
extra columns, and `--limit N` to change the row count (default 50).

## Trial balance: `tb`

```sh
ownpurse tb roast --date 2025-12-31
```

The trial balance at a date, computed from the local ledger. Balance-sheet accounts are cumulative; profit-and-loss
accounts run from the fiscal-year start, with earlier years closed into retained earnings, the way Xero does it.

| Option | Effect |
|---|---|
| `--date <DATE>` | As-of date (required) |
| `--since <DATE>` | Open from Xero's cached trial balance at this date and add later activity |
| `--full-history` | Balances from local documents only. The default opens from Xero's cached trial balance at the prior year-end and adds the year's activity |
| `--compare` | 2 to 4 orgs side by side |
| `--align code\|map` | With `--compare`: merge rows by account code (default) or by `account_map` category |

`<ORG>` takes an alias, a tenant id, `group:<name>`, a comma list or `all`.

### Comparing organisations

```sh
ownpurse tb roast,design --date 2025-12-31 --compare
ownpurse tb roast,hold,design --date 2025-12-31 --compare --align map
```

Each org gets its own column and keeps its own totals. Rows merge only on an identical account code, or on an
`account_map` category with `--align map`. Identical names under different codes never merge. There is no
difference column and no cross-org total. Orgs with different base currencies or fiscal years are refused.

## General ledger: `gl`

```sh
ownpurse gl roast --account 1010 --from 2025-01-01 --to 2025-12-31
```

The postings derived from the mirrored documents, with a running balance.

| Option | Effect |
|---|---|
| `--from`, `--to` | Date range (inclusive) |
| `--account <ACCOUNT>` | Code, name or id (substring match if no exact match) |
| `--source <SOURCE>` | Only postings from one document type: `BankTransactions`, `BankTransfers`, `ManualJournals`, `Invoices`, … |
| `--search <SEARCH>` | Substring of name, contact, reference or description |
| `--full-history` | Running balance from local documents only (the default opens from the cached Xero trial balance) |

How each document posts:

| Source | Postings (debit +, credit −) |
|---|---|
| Bank transactions, spend and receive (authorised) | Lines (net of tax), tax to the sales-tax account, the bank account for the total. Transfer legs are skipped |
| Bank transfers | Debit the receiving bank, credit the paying bank |
| Manual journals (posted) | Each line; a positive line amount is a debit |
| Invoices (authorised, paid) | Sales: debit receivables, credit lines and tax. Bills: the reverse |
| Credit notes (authorised, paid) | The reverse of the matching invoice type |
| Payments (authorised) | Bank against receivables or payables, by payment type |

## Profit and loss: `pl`

```sh
ownpurse pl roast --from 2025-01-01 --to 2025-12-31
ownpurse pl roast --to 2025-12-31 --cash --strict
```

The profit and loss for a period, from the local ledger, compared account by account with Xero's profit and loss
for the same dates and basis when the record holds one.

| Option | Effect |
|---|---|
| `--to <TO>` | Period end (required) |
| `--from <FROM>` | Period start (default: the fiscal year's start) |
| `--cash` | Cash basis (Xero's `paymentsOnly`) |
| `--strict` | Exit `1` when anything differs from Xero's cached report |

## Balance sheet: `bs`

```sh
ownpurse bs roast --date 2025-12-31
ownpurse bs roast --date 2025-12-31 --cash
```

The balance sheet at a date, from the local ledger, compared with Xero's balance sheet for the same date and basis
when the record holds one. It opens from Xero's cached balance sheet at the prior fiscal year-end (same basis) when
there is one, and otherwise from local documents only.

| Option | Effect |
|---|---|
| `--date <DATE>` | As-of date (required) |
| `--cash` | Cash basis (Xero's `paymentsOnly`) |
| `--since <SINCE>` | Open from Xero's cached balance sheet at this date (default: the prior fiscal year-end, when cached) |
| `--full-history` | Balances from local documents only |
| `--strict` | Exit `1` when anything differs from Xero's cached report |

## Accrual and cash

Statements are on the accrual basis unless you pass `--cash`. On the cash basis:

- invoices and credit notes post at each payment, their lines and tax in proportion to the amount paid;
- manual journals with `ShowOnCashBasisReports: false` are left out.

A cash-basis `pl` or `bs` compares itself with a cash-basis Xero report, so fetch that one with `--cash`.

## Parity with Xero: `parity`

```sh
ownpurse parity roast --date 2025-12-31
ownpurse parity all --date 2025-12-31 --since 2024-12-31 --strict
```

Compares the local trial balance with Xero's cached trial balance, account by account, to the cent. Nothing is
fetched: `parity` always reads a snapshot already in the record.

| Option | Effect |
|---|---|
| `--date <DATE>` | As-of date (required) |
| `--since <SINCE>` | Open from Xero's cached trial balance at this date |
| `--strict` | Exit `1` when anything differs |
| `--cached` | Reuse a cached snapshot (always the case) |

Differences under full history usually come from balances the API cannot show: conversion balances from Xero's
opening-balance screen, reconcile-screen rounding, and automated depreciation, inventory or currency entries.
`--since` sets those aside by opening from Xero's own figures.

## Fetching Xero's reports: `report`

```sh
ownpurse report fetch roast trialbalance --date 2025-12-31
ownpurse report fetch all pl --from 2025-01-01 --to 2025-12-31 --cash
ownpurse report list roast
```

`report fetch` stores one Xero report in the record (one call per org). `report list` shows what the record holds
(zero calls). `parity` reads the latest trial balance; `pl` and `bs` read the matching profit and loss or balance
sheet.

| Kind | Dates |
|---|---|
| `trialbalance` | `--date` (default: the last fiscal year-end) |
| `balancesheet` | `--date` (default: the last fiscal year-end) |
| `pl` | `--from` (default: that fiscal year's start), `--to` (default: the last fiscal year-end) |
| `banksummary` | `--from` (default: that fiscal year's start), `--to` (default: the last fiscal year-end) |

`--cash` fetches the cash-basis version.

## Exports for your accountant: `export`

```sh
ownpurse export roast tb --date 2025-12-31
ownpurse export roast gl --from 2025-01-01 --to 2025-12-31 --out roast-gl-2025.csv
```

Writes a CSV from the record, with zero calls. `tb` and `gl` come from the local ledger; `bs` and `pl` come from
Xero's cached reports.

| Option | Effect |
|---|---|
| `--date <DATE>` | As-of date for `tb` / `bs` (default: the last fiscal year-end) |
| `--from`, `--to` | Period for `gl` / `pl` (default: that fiscal year) |
| `--out <OUT>` | The file to write (default `<org>-<kind>-<date>.csv` in the current folder). Never written over an existing file |
| `--csv` | CSV (the default) |
| `--xlsx` | A spreadsheet. Not available yet; use `--csv` |

## Reading documents

These commands list and show what the record holds, all with zero calls:

| Command | What |
|---|---|
| `accounts <org> [--bank] [--type] [--search] [--archived]` | Chart of accounts |
| `org <org>` | Plan class and lock dates |
| `contacts <org> [search]` | Contacts |
| `txns <org> [--from] [--to] [--account] [--status] [--search] [--deleted]` | Bank transactions |
| `journals <org>`, `transfers <org>` | Manual journals, bank transfers |
| `list <org> <Entity> [--fields Contact.Name,Total]` | Any mirrored entity |
| `show <org> <Entity> <id>` | One record with every version the record holds |
| `open <org> <Entity> <id> [--launch]` | The document's page in Xero's web app (a link; no call) |
| `history <org> <Entity> <id> --cached` | The document's history and notes, as last fetched |
| `attachments <org> <Entity> <id> --cached` | The document's attachments, as last fetched |

Without `--cached`, `history` and `attachments` fetch from Xero (one call each). `attachments --get <file>` saves
one attachment's bytes, never over an existing file.

Full options for each: [CLI reference](/docs/reference/cli/).
