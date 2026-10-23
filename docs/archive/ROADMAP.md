# Roadmap

Status: design notes. Nothing here is built until it is scheduled. Everything stays **read-only**, computed from
the local record with **zero API calls**, and **generic**: organisation pairs, parties and account mappings come
from configuration, never code.

## Next

In order of intent, not scheduled:

- **The TUI in the ownpurse design system**: the new name, the Night terminal theme (`brand/TERMINAL.md`), the coin
  status glyphs, and the mark in the frame header with its blinking eye. Plan:
  [docs/plans/TUI-REDESIGN.md](plans/TUI-REDESIGN.md).
- **A connector interface** so sources beyond Xero (QuickBooks, bank exports, spreadsheets, portfolios, files) can
  append facts with provenance, and a guide for writing one.
- **Facts in the record**: move the record toward entity, attribute, value, time and source, with entries that can
  be held before they are posted (`docs/PHILOSOPHY.md`). Plan: [docs/plans/FACTS.md](plans/FACTS.md).
- **The daemon**: a small local process that schedules syncs and serves facts and live updates to tools and agents.
- **An agent skill that installs with the CLI**, built from `docs/agent-bridge-skill-snippet.md`.

## Multi-organisation work

People who run several related Xero organisations need to move between them instantly and see them together:
who owes whom, whether both sides agree, and what the group looks like as one. Xero itself has no
cross-organisation API or consolidation; ownpurse holds every org in one local record, so it can do this locally.

Shipped: fast scope switching (picker, groups, per-scope state, comma lists in the CLI), intercompany pairs from
config (`ownpurse ic`, the Intercompany tab) and side-by-side Compare (`v` on the Ledger, `tb a,b --compare`).

### Example configuration (the demo world, invented)

Brackenridge Lane Holdings (`hold`) lends Tallowmere Coffee Roasting (`roast`) a term loan; each side books it in
its own account (14xx receivable, 24xx payable), and the pair matches postings on both sides within 5 days.

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

The keys are documented in [CONFIG.md](CONFIG.md).

## Next: consolidated view

A **Consolidated** tab for a group scope: group trial balance / balance sheet / P&L with one column per org, an
**eliminations** column and the consolidated total. Every cell drills to postings.

- **Eliminations** come only from pairs that net to zero, or from pair items explicitly marked `"treated": true`
  in config; never inferred. Every elimination shows its source pair or item. Pairs with a difference are listed,
  not eliminated.
- **Multi-currency groups and differing fiscal years are unsupported**: cross-org views refuse with a clear
  message instead of guessing.
- Config: `"groups": {"all-co": {"orgs": […], "eliminate": "intercompany"}}`.

Open question: performance on large records (cache ledgers per org keyed by the record's latest version id).

## Smaller items

- TUI: ellipsize the middle of long Ledger breadcrumbs at 80 columns.
- TUI: Activity detail keeps "problem —" together (or drops it when empty).
- Verify the per-document "open in Xero" URL patterns in a browser and mark them `verified: true`.
- The CLI commands that are not available yet: report fetch, `history`, `attachments`, `export`, `q`, `init`,
  `verify`, `skills`, `status`.
- Writes, one command per approved need, through the operations log (`docs/design.md`).
