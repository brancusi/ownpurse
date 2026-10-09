---
title: Roadmap
description: What ships today, what comes next, and what is planned for ownpurse.
sidebar:
  order: 2
---

These are design notes. Nothing here is built until it is scheduled. Reads stay computed from the local record with
zero API calls, and everything stays generic: organisation pairs, parties and account mappings come from
configuration, never code.

## Shipped

- **The record**: append-only, hash-chained, exact decimals. Every version of every entity, report snapshots, sync
  runs and the call log.
- **The Xero connector**: login with PKCE, multi-organisation sync, report fetch, history, attachments.
- **The local ledger**: `gl`, `tb`, `pl`, `bs` on accrual or cash basis, checked against Xero's reports to the cent
  (`parity`, `--strict`), and CSV exports for your accountant.
- **Multi-organisation work**: fast scope switching (picker, groups, per-scope state, comma lists in the CLI),
  intercompany pairs from config (`ownpurse ic`, the Intercompany tab) and side-by-side compare (`v` on the Ledger,
  `tb a,b --compare`).
- **The terminal UI**: read-only, themes, per-org accents, your own key bindings, explainer panels, walkthroughs, and
  in-place upgrades.
- **The agent bridge**: an agent reads the exact view state and moves the view with view-only verbs.
- **Classification**: local labels, rules and groups, never sent to Xero.
- **Writes**: manifests, local checks, sealed packages approved by hash, guarded runs with read-back and recovery.

## Next: consolidated view

A **Consolidated** tab for a group scope: the group's trial balance, balance sheet and profit and loss, with one
column per org, an **eliminations** column and the consolidated total. Every cell drills to postings.

- **Eliminations** come only from pairs that net to zero, or from pair items explicitly marked `"treated": true` in
  config; never inferred. Every elimination shows its source pair or item. Pairs with a difference are listed, not
  eliminated.
- **Multi-currency groups and differing fiscal years are unsupported**: cross-org views refuse with a clear message
  instead of guessing.
- Config: `"groups": {"all-co": {"orgs": […], "eliminate": "intercompany"}}`.

Open question: performance on large records (caching ledgers per org, keyed by the record's latest version).

### Example configuration

The demo world is invented. Brackenridge Lane Holdings (`hold`) lends Tallowmere Coffee Roasting (`roast`) a term
loan; each side books it in its own account (14xx receivable, 24xx payable), and the pair matches postings on both
sides within 5 days.

```json
{
  "groups": {"all-co": {"orgs": ["roast", "hold", "design", "lab"]}, "ops": {"orgs": ["roast", "design", "lab"]}},
  "parties": {
    "hold": {"contact": ["\\bbrackenridge\\b"], "narration": ["\\bbrackenridge\\b"]}
  },
  "intercompany": {
    "pairs": [
      {"id": "hold-roast", "label": "Brackenridge ↔ Tallowmere",
       "a": {"org": "hold", "accounts": ["1460"]},
       "b": {"org": "roast", "accounts": ["2410"], "party": "hold", "component": "residual"},
       "movement": "match_both", "match_days": 5}
    ]
  },
  "account_map": {
    "categories": {
      "cash": {"default": "type:BANK"},
      "loan_balances": {"hold": ["1460"], "roast": ["2410"]},
      "revenue": {"default": "class:REVENUE"}
    }
  }
}
```

The keys are documented in the [configuration reference](/docs/guides/configuration/).

## Planned

- **More connectors.** QuickBooks, bank exports, spreadsheets, investment portfolios and plain files. The connector
  interface is being designed; see [Writing a connector](/docs/developers/connectors/).
- **Facts.** A data model of entity, attribute, value, time and source, with entries held but not posted until they
  are classified. See [Data model](/docs/developers/data-model/#facts).
- **A daemon** that keeps the record current and serves it, so the CLI, the TUI and agents share one live process.
- **Commands not available yet**: `init` (write the config for you), `q` (queries), `verify` (recompute the record's
  chain from the CLI), `skills` (install the agent skill) and `status`. Each answers `not_available` today.
- **Spreadsheet exports**: `export --xlsx`.

## Smaller items

- TUI: shorten the middle of long Ledger breadcrumbs at 80 columns.
- TUI: keep Activity's problem text together, or drop it when empty.
- Verify the per-document "open in Xero" URL patterns in a browser and mark them verified.
