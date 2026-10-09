<!-- Recipe block for an agent skill that drives ownpurse; not part of the CLI reference. -->

## Read and steer the user's TUI (agent bridge, v0.10, view only)

When the user has `ownpurse tui` open, you can see exactly what they see and move their view. Every change is
announced on their screen; their own keys always win.

**Work only through `ui state` and `ui send`.** The state is complete (a fresh TUI built from it draws the
same frame); never read a screenshot or the rendered text to understand the view. `ui screen` is for
showing the user, or a cross-check.

```bash
ownpurse ui ls                                   # running TUIs: id, label, pid, scope, tab (--session id|label|scope:roast)
ownpurse --json ui state                         # the view as data: rev, tab, scope, as_of, basis, filter,
                                                 # selected + visible rows (anchors), strip facts
ownpurse --json ui state --fields rev,tab,scope,selected   # only what you need
ownpurse ui screen --format png --out view.png   # a real screenshot (also text, ansi, html, svg)
ownpurse ui send --expect-rev 12 tab ledger scope roast select 1010 drill
ownpurse ui send --expect-rev 13 up
ownpurse ui send --expect-rev 14 --caption "Green-bean card spend, 2025 · 3 lines · 1,990.90" filter "Green"
ownpurse ui send --expect tab=ledger --expect level=1 clear-filter   # refused (exit 4) unless the view is that
ownpurse ui send back                            # navigation history (forward too)
ownpurse ui wait --until idle --timeout 10       # also rev>N, store_rev>N, record>N, running=none
```

Rules:
- Read `ui state` first and pass its `rev` as `--expect-rev`. Exit 4 `view_changed` means the user moved since
  your read: read again and decide whether the change still makes sense. Exit 4 `user_busy` means they are
  typing or in an overlay: do not retry in a loop; wait and read again.
- Verbs (the whole allow-list): `tab`, `scope`, `asof`, `filter`, `clear-filter`, `select <#n|anchor|text>`,
  `drill`, `up` (one level, as Esc), `back`, `forward`, `basis year|history`, `differs on|off`, `compare a,b`,
  `reload`, `open` (the document on screen in Xero's web app, in the user's browser; a link, never an API
  call), `caption`, `label`. Sync, login, confirm, edits and quit are refused (exit 2); ask the user to do
  those themselves. The one non-view action is `ownpurse ui upgrade` (a newer installed TUI takes over with
  the view restored).
- `select` takes any row of the current list, on screen or not: `#n` (the `row` number, as in
  `ui state --all-rows`), an anchor as JSON (`'{"account_code":"1010"}'`), or text (an exact field value such as
  an account code first, then a substring). The view scrolls to it, so `select '{"account_code":"4010"}' drill`
  works in one send. A row that is not in the list at all is `not_found` (exit 2): clear the filter or change
  the scope.
- `filter` is a case-insensitive substring over what the screen lists (TB: code + account name), with
  spaces and punctuation literal: `filter "Kiln Room -"` does not match `Kiln - Operating`.
  `code:2410` / `code:1450..1480` select account codes. Check `filter_matches` / `filter_total` in the
  answer. On screens where a filter means nothing it is refused (exit 2, `not_applicable`); go back first.
- Row `fields` carry every column on screen in snake_case (`parity_year`, `tb_fetched`, `calls_left`, …);
  every shown amount has an exact-number twin (`debit_value`, `credit_value`, `<org>_net_value`, …;
  see Display vs value in docs/TUI.md); read the twin, never parse the display string.
- `rows` are the rows on screen. If `truncated` is true, read the rest with `ownpurse ui state --all-rows`
  (or `--rows A..B`): read-only, the user's view doesn't move.
- `--expect PATH=VALUE` (also `PATH!=VALUE`, `PATH~TEXT`; repeatable; paths into `ui state`, e.g.
  `scope`, `selected.fields.account_code`) states what you believe the view is. If it is not, nothing is done:
  exit 4 `view_mismatch` with `expected` and `actual`. Use it with or instead of `--expect-rev`.
- A send answers with `summary` (the view after it: rev, tab, level, scope, breadcrumb, selected, filter) and
  `effects`, one per verb (`{verb, changed, what}`); a verb that changed nothing says so (`changed: false`,
  e.g. `back` with no history notes `no_history`). Read the summary; no second `ui state` is needed.
- `ui wait --until idle|rev>N|store_rev>N|record>N|running=none [--timeout S]` blocks until it holds (default 30 s;
  exit 4 `timeout`). Use `running=none` after asking the user to sync, `rev>N` to wait for them to move.
- Exit 4 `too_small`: the user's terminal is below 60×20 and shows only a size warning, so a change
  couldn't be announced. Ask them to enlarge it; reads still work.
- Errors are one JSON object `{error, message, help, expected?, actual?}` (schema: docs/schema/ui-error.v1.json;
  every code with its exit status and meaning: "Error codes" in docs/TUI.md). Follow `help`.
- Captions: `--caption "<text>"` with a send (atomic, one update) or `caption "<text>"` alone describe the view
  in the user's footer; ≤ 120 characters, one line, no control or escape characters (`bad_caption`).
- Screenshots and sends show `◆ agent` in their header; say what you did and why.

### The user's key bindings (v0.12.0)

The user's keys are the built-in table plus `~/.config/ownpurse/keys.toml`. It's theirs to edit; don't write it
unless they ask. `ui state` shows `keymap.revision` and `keymap.overrides`, and `ownpurse-tui --keys markdown`
prints the effective keymap (their keys marked with the keys.toml line). The format:

```toml
v = 1
[[bindings]]
key = "cmd+["
action = "history.back"
context = "global"
```

A save reloads at once; an invalid file keeps the previous keymap and the TUI names file:line.
