# Design

## Layers

| Layer | Owner | Contents |
|---|---|---|
| A. Xero record | ownpurse | every observed version of every entity, report snapshots, sync runs, HTTP log |
| B. Operations log | ownpurse | every write planned/attempted and its outcome (schema in place; commands to come) |
| C. Workflow | your application | decisions, approvals, reconciliations — reads A/B, never writes them |

A and B share one SQLite file so "we sent X" and "Xero now shows version N" commit atomically.

## Tables (all history tables append-only, enforced by triggers)

- `versions(tenant_id, entity, xero_id, version, content_hash, status, xero_updated_utc,
  observed_at, source, sync_run_id, caused_by_op, payload)` — a row only when content changes.
  `source` ∈ sync | readback | webhook | backfill. View `current` = latest version per record.
- `reports(tenant_id, report, params, fetched_at, content_hash, payload)` — immutable snapshots.
- `sync_runs(...)` — one row per sync (mode, calls, fetched, new versions, high water, vanished ids).
- `http_log(...)` — every call with status, duration, `X-DayLimit-Remaining`, rate-limit problem.
- `operations(op_id, fingerprint UNIQUE, request, expected_before_version, decision_refs,
  approval_ref, supersedes_op, …)` and `op_events(op_id, seq, state, idempotency_key, …)`.
- `chain(seq, tbl, row_id, row_hash, prev_hash, hash)` — every append above is linked;
  `ownpurse verify` recomputes it. Commit the head hash somewhere external at checkpoints.

A new version not caused by one of our operations is **external drift** (someone edited the
record in Xero's UI). Records are never removed locally; deletions arrive as status changes and
ids missing from a full sweep are listed as `vanished` in the sync run.

## Sync

| Entities | Mode | TTL |
|---|---|---|
| Organisation, Accounts, TaxRates, TrackingCategories, Currencies, BankTransfers | full pull | 24 h (transfers 15 min) |
| Contacts | If-Modified-Since, weekly full sweep | 24 h |
| BankTransactions, ManualJournals, Invoices, CreditNotes, Payments, Overpayments, Prepayments | If-Modified-Since, pageSize 1000, weekly full sweep | 15 min |
| Reports | snapshot per request | — |

- High water = max `UpdatedDateUTC` seen; queries subtract 5 s (IMS is second-precision).
- Full sweeps exist because some edits don't bump `UpdatedDateUTC` (account-code changes on
  lines, contact balances).
- Default BankTransactions lists include DELETED records (verified live); full sweeps add a `Statuses=DELETED,VOIDED`
  for invoices.
- ManualJournals never send `order` (Xero then ignores paging).

## Ledger and parity

The GL feed (Journals) needs Xero's Advanced tier, so the ledger builder (`ownpurse-core` `ledger`) derives postings from documents:

| Source | Postings (debit +, credit −) |
|---|---|
| BankTransactions SPEND* / RECEIVE* (AUTHORISED) | lines (net of tax) + tax → Sales Tax system account, bank account for `Total`; `*-TRANSFER` legs skipped |
| BankTransfers | Dr to-bank, Cr from-bank |
| ManualJournals (POSTED) | each line; positive `LineAmount` = debit |
| Invoices (AUTHORISED, PAID) | ACCREC: Dr receivables / Cr lines + tax; ACCPAY: the reverse |
| CreditNotes (AUTHORISED, PAID) | reverse of the matching invoice type |
| Payments (AUTHORISED) | bank vs receivables/payables by `PaymentType` |

Trial balance follows Xero: balance-sheet accounts cumulative, P&L accounts from the fiscal-year
start, earlier P&L closed into the `RETAINEDEARNINGS` system account. `parity` compares net debit
balances per AccountID with Xero's TrialBalance report (YTD columns). `--since D0` opens from Xero's
TB at D0 and adds only later postings — the way to set aside things the API cannot show:

- conversion balances entered in Xero's opening-balance screen (Historical Adjustment);
- reconcile-screen "minor adjustments", which post to Rounding;
- fixed-asset depreciation, inventory COGS automation, FX revaluation.

All amounts are `Decimal`. JSON numbers are parsed with `parse_float=Decimal` and re-serialised with
their original digits; `money()` rejects floats so they cannot leak into arithmetic.

## Writes (planned)

State machine per operation: `planned → approved → sending → confirmed | rejected | uncertain → reconciled`.

- Fingerprint (org + target + business content + external refs) is UNIQUE: no accidental duplicates.
- The `op_id` is embedded in the record (Reference/Narration) so an uncertain send can be found.
- Idempotency keys are sent but only last 6 minutes at Xero; the operations log is the real dedupe.
- An uncertain outcome is never retried automatically; it is reconciled by searching for the marker.
- Before a write the target is fetched live and must equal `expected_before_version`.
- After a write the response is recorded as a `readback` version with `caused_by_op`, and report
  snapshots dated on/after the change are considered stale.
- Batches use `summarizeErrors=false`; each item is its own operation.

## Webhooks

Xero webhooks cover only Contacts, Invoices, CreditNotes, Overpayments, Prepayments and carry no
data. If added, a webhook event is just a trigger to fetch one record (`source = webhook`).
