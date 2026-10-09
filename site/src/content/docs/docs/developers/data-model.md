---
title: Data model
description: The layers of the ownpurse record, its tables, how sync fills them, how the ledger is derived, how writes are logged, and the direction toward facts.
sidebar:
  order: 2
---

## Layers

| Layer | Owner | Contents |
|---|---|---|
| A. Source record | ownpurse | Every observed version of every entity, report snapshots, sync runs, the HTTP log |
| B. Operations log | ownpurse | Every write planned, sealed, approved and attempted, and its outcome |
| C. Workflow | Your application | Decisions, approvals, reconciliations. It reads A and B and never writes them |

A and B share one SQLite file, so "we sent X" and "Xero now shows version N" commit atomically.

## Tables

All history tables are append-only, enforced by triggers.

| Table | One row per | Key columns |
|---|---|---|
| `versions` | Observed version of an entity, only when its content changes | `tenant_id, entity, xero_id, version, content_hash, status, xero_updated_utc, observed_at, source, sync_run_id, caused_by_op, payload` |
| `reports` | Report snapshot (immutable) | `tenant_id, report, params, fetched_at, content_hash, payload` |
| `sync_runs` | Sync | Mode, calls, records fetched, new versions, high-water mark, vanished ids |
| `http_log` | API call | Status, duration, `X-DayLimit-Remaining`, rate-limit problem |
| `operations`, `op_events` | Write operation, and each state it passed through | Fingerprint (unique), request, expected "before" version, decision and approval references, idempotency key |
| `packages` | Sealed write package | Hash, title, profile, operation count and the operations, source manifest, sealed time |
| `approvals` | Recorded approval | Package hash, the verbatim response, how it was given (terminal, chat or a Demo Company standing approval), scope |
| `verifications` | Verify outcome | Package hash, `verified`, `rejected`, `uncertain`, `incomplete` or `discrepancy`, the problems |
| `chain` | Appended row anywhere above | `seq, tbl, row_id, row_hash, prev_hash, hash` |

`source` on a version is one of `sync`, `readback` (after one of your own writes), `webhook` or `backfill`. The
`current` view holds the latest version of each record.

**The chain.** Every append above is linked into `chain`: each link hashes the row and the previous link. Recomputing
it proves that no row was edited or removed. Copy the head hash somewhere outside the record at checkpoints, so you
can later prove the whole file up to that point.

**Drift and deletions.** A new version not caused by one of your own operations is **external drift**: someone
edited the record in Xero's web app. Records are never removed locally. Deletions arrive as status changes, and ids
missing from a full sweep are listed as `vanished` in the sync run.

## Sync

| Entities | Mode | Stale after |
|---|---|---|
| Organisation, Accounts, TaxRates, TrackingCategories, Currencies, BankTransfers | Full pull | 24 h (transfers 15 min) |
| Contacts | If-Modified-Since, weekly full sweep | 24 h |
| BankTransactions, ManualJournals, Invoices, CreditNotes, Payments, Overpayments, Prepayments | If-Modified-Since, 1,000 per page, weekly full sweep | 15 min |
| Reports | A snapshot per request (`report fetch`) | — |

- The high-water mark is the latest `UpdatedDateUTC` seen. Queries subtract 5 seconds, because If-Modified-Since has
  one-second precision.
- Full sweeps exist because some edits don't change `UpdatedDateUTC` (account-code changes on lines, contact
  balances).
- Bank transaction lists include deleted records. Full sweeps of invoices add deleted and voided statuses.
- Manual journal queries never send an `order` parameter, because Xero then ignores paging.

## Ledger and parity

Xero's journals feed needs its Advanced plan, so the ledger builder in `ownpurse-core` derives postings from
documents:

| Source | Postings (debit +, credit −) |
|---|---|
| BankTransactions SPEND / RECEIVE (AUTHORISED) | Lines (net of tax) + tax → the sales-tax system account, the bank account for `Total`. `*-TRANSFER` legs are skipped |
| BankTransfers | Debit the receiving bank, credit the paying bank |
| ManualJournals (POSTED) | Each line; a positive `LineAmount` is a debit |
| Invoices (AUTHORISED, PAID) | ACCREC: debit receivables, credit lines + tax. ACCPAY: the reverse |
| CreditNotes (AUTHORISED, PAID) | The reverse of the matching invoice type |
| Payments (AUTHORISED) | Bank against receivables or payables, by `PaymentType` |

On the cash basis, invoices and credit notes post at each payment, in proportion to the amount paid, and manual
journals with `ShowOnCashBasisReports: false` are left out.

The trial balance follows Xero: balance-sheet accounts are cumulative, P&L accounts run from the fiscal-year start,
and earlier P&L is closed into the retained-earnings system account. `parity` compares net debit balances per account
with Xero's trial balance. `--since D0` opens from Xero's trial balance at D0 and adds only later postings. That is
how to set aside what the API cannot show:

- conversion balances entered in Xero's opening-balance screen;
- reconcile-screen "minor adjustments", which post to rounding;
- fixed-asset depreciation, inventory cost-of-goods automation, currency revaluation.

All amounts are decimals. JSON numbers are parsed with their original digits and written back with the same digits.
Floating point cannot reach money arithmetic.

## Writes

Each operation moves through `planned → approved → sending → confirmed | rejected | uncertain → reconciled`.

- The fingerprint (org, target, business content, external references) is unique, so there are no accidental
  duplicates.
- A marker naming the operation goes into the document (reference or narration), so an uncertain send can be found.
- Idempotency keys are sent, but Xero keeps them for only 6 minutes. The operations log is the real dedupe.
- An uncertain outcome is never retried automatically. It is reconciled by searching for the marker.
- Before a write, the target is fetched live and must equal the expected "before" version.
- After a write, the response is recorded as a `readback` version with `caused_by_op`, and report snapshots dated on
  or after the change count as stale.
- Batches report each item's errors separately; each item is its own operation.

See [Writes](/docs/guides/writes/) for the commands.

## Webhooks

Xero webhooks cover only Contacts, Invoices, CreditNotes, Overpayments and Prepayments, and carry no data. If they
are added, a webhook event will only trigger a fetch of one record (`source = webhook`).

## Facts

Today a row in `versions` is a whole document as one source reported it. The direction of the data model is to
record **facts**: small, single statements, each with

- an **entity**: the thing the fact is about (a bank transaction, an account, a holding);
- an **attribute**: what is being said (its amount, its account, its counterparty);
- a **value**;
- a **time**: when it was true, and when ownpurse learned it;
- a **source**: where it came from (a connector, a read-back, a person, an agent).

This follows the tradition of [Datomic](https://www.datomic.com/) and [RDF](https://www.w3.org/RDF/): an
append-only log of assertions, from which any view of the books at any moment can be derived. Facts from different
sources can sit side by side about the same entity, each with its provenance, and disagreements stay visible instead
of being overwritten.

It also gives a natural home to entries that are not yet classified. They can be **held** as facts, counted and
visible, without being posted to an account, until someone decides what they are.

Versions already behave this way at the document level: append-only, timed, sourced and chained. The fact model is
being designed; it is not a shipped schema.
