---
title: Intercompany
description: Check that related organisations agree about what they owe each other, with groups, parties and pairs from your config.
sidebar:
  order: 3
---

If you run several related organisations, their books should mirror each other: a loan one org records as a
receivable, the other records as a payable. Xero has no cross-organisation API and no consolidation. ownpurse holds
every org in one local record, so it can compare them locally, with zero API calls.

What ships today:

- **Fast scope switching**: groups, a scope picker in the TUI, per-scope state, and comma lists in the CLI.
- **Intercompany pairs** from config: `ownpurse ic` and the TUI's Intercompany tab.
- **Side-by-side compare**: `ownpurse tb a,b --compare` and `v` on the TUI's Ledger.
- **Explainer panels**: the bridge as a waterfall, and who owes whom between orgs.

Everything here is read-only and generic. Organisation pairs, parties and account mappings come from your config,
never from code.

## Groups

A group is a named set of orgs:

```json
{"groups": {"all-co": {"orgs": ["roast", "hold", "design", "lab"]},
            "ops":    {"orgs": ["roast", "design", "lab"]}}}
```

Use `group:ops` anywhere an org is accepted: `ownpurse sync group:ops`, `ownpurse tb group:ops --date …`, or the
TUI's scope picker (`g`).

## Parties and pairs

A **party** says how to recognise one organisation in another's books: case-insensitive regular expressions
matched against the contact and the narration, and optionally a matching posting in another org.

A **pair** names the accounts on each side that should mirror each other.

This example uses the invented demo world. Brackenridge Lane Holdings (`hold`) lends Tallowmere Coffee Roasting
(`roast`) a term loan; each side books it in its own account (1460 receivable, 2410 payable), and the pair matches
postings on both sides within 5 days.

```json
{
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
  }
}
```

Every key is in the [configuration reference](/docs/guides/configuration/#intercompany-parties-intercompanypairs).
In short:

- `a` and `b` are the two sides: an org alias and its account codes (or `{"name": …}` for code-less bank accounts).
- `b.party` is the party whose share of `b`'s accounts is compared. `b.component` is `residual` (the account total
  less other parties' attributed postings) or `attributed` (only this party's postings).
- `movement` explains the year's change in the difference: `b_unexplained`, or `match_both` (opposite amounts on
  both sides within `match_days`).
- `known_items` lists expected historical items; the engine reports whether the record contains each one.

## `ownpurse ic`

```sh
ownpurse ic                       # every pair
ownpurse ic hold-roast --as-of 2025-12-31
ownpurse --json ic hold-roast
```

For each pair the output shows:

- both sides' balances and the **difference** between them;
- the **bridge**: the prior-year opening difference plus this year's movement;
- the **explaining items**, each with its side, date, org, account, contact, amount and why it was matched;
- an explicit **unexplained residual**.

Nothing is ever filled in to force a match. A pair is in one of these states:

| Status | Meaning |
|---|---|
| `✓ matches` | Difference 0.00 |
| `bridged` | Difference is not zero, but fully explained: residual 0.00 |
| `unexplained` | The residual is not zero |
| `— not compared` | No cached Xero trial balance at the as-of date or the prior year-end |
| `not supported: <reason>` | Different base currencies or fiscal years, or an unknown org |

`--as-of` defaults to the scoped org's last fiscal year-end. Balances come from Xero trial balances in the record,
so fetch them first if a pair says `not compared`:

```sh
ownpurse report fetch hold,roast trialbalance --date 2025-12-31
ownpurse report fetch hold,roast trialbalance --date 2024-12-31
```

## In the terminal UI

Tab `3` is **Intercompany**. It runs the same engine as `ownpurse ic`.

- **Level 1, pairs**: one row per pair with both sides, the difference, the residual and the status. The residual
  column never hides, even on narrow screens. `s` sorts (unexplained first, then by difference, then by name); `/`
  filters by pair name or org.
- **Level 2, the bridge**: both sides, the difference at year end, the opening difference plus this year's movement,
  every explaining item, and the unexplained residual in bold. `a` and `b` open that side's accounts in its own
  org's Ledger.
- **Level 3, the posting**: `Enter` on an item shows the document in its own org without changing your scope.

The scope decides which pairs show: `all` shows every pair, an org shows pairs where it is either side, and a group
shows pairs with both sides in the group. The Overview tab raises an attention item only for a non-zero residual, a
pair that cannot be compared, or a pair that is not supported.

## Explainer panels

Two panels draw intercompany numbers as pictures, from the same engine:

- **Bridge**: the gap between two sides' books as a waterfall. Each reconciling item takes its part away, and what
  is left reads "Unexplained" (or `✓ 0.00`). Open it from the palette ("Explain: bridge <pair>") or with
  `ownpurse viz bridge <pair>`.
- **Flow**: one box per org and one arrow per pair, showing who owes whom at a date (`--basis owed`) or what moved
  over a period (`--basis paid`). Open it from the palette ("Explain: who owes whom") or with `ownpurse viz flow`.

## Side-by-side compare

```sh
ownpurse tb hold,roast --date 2025-12-31 --compare
ownpurse tb hold,roast --date 2025-12-31 --compare --align map
```

Compare puts 2 to 4 orgs' trial balances in columns. Rows merge only on an identical account code, or on a category
from `account_map` with `--align map`. In the TUI, press `v` on the Ledger. See
[Reports](/docs/guides/reports/#comparing-organisations).

## Planned: a consolidated view

A consolidated tab for a group scope is planned: the group's trial balance, balance sheet and profit and loss with
one column per org, an eliminations column, and the consolidated total. Eliminations will come only from pairs that
net to zero, or from items you mark as treated in config, never inferred. Multi-currency groups and differing fiscal
years will be refused with a clear message. See the [roadmap](/docs/reference/roadmap/).
