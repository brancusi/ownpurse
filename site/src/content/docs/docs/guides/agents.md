---
title: Working with agents
description: How ownpurse behaves for agents, how an agent reads and steers your terminal UI through the ui bridge, and how to give your agent the ownpurse skill.
sidebar:
  order: 5
---

ownpurse is designed for AI coding agents and the people supervising them. The CLI follows the
[AXI](https://axi.md/) conventions, and a running TUI exposes a view-only bridge, so an agent can see exactly what you
see and point things out on your screen.

## AXI conventions

Every command behaves the same way, so an agent can rely on it without reading prose:

- **Compact output by default.** Text tables sized for a terminal, and `--limit N` (default 50) on lists.
- **`--json` everywhere.** The same data, machine-readable. Amounts are exact JSON numbers with the original digits,
  never floats.
- **Counts on every list**, so an agent knows when it saw only part of the answer (`--full` shows everything).
- **Structured errors.** A refusal is an `error` code, a `message`, and a `help[]` list of next steps:

  ```text
  error: view_changed
  message: The user changed the view since you read it (rev 41 → 43). Nothing was changed.
  help[1]:
    Run `ownpurse ui state` again and resend if it still makes sense.
  ```

- **Stable exit codes.** `0` ok, `1` runtime error, `2` usage error or refused by design, `4` refused because the state
  moved (another actor changed something since you read it, the user is busy, a timeout). `write approve-check` uses
  `3` and `write verify` uses `5`. See [CLI reference](/docs/reference/cli/#exit-codes).
- **No prompts.** The one interactive step is the browser consent in `login`.
- **Next-step hints.** Output and errors carry `help` lines naming the command to run next.
- **Calls are visible.** Commands that call the provider are marked ⇅ in `--help`; `--offline` makes them refuse.

## The agent bridge (`ownpurse ui`)

When you have `ownpurse tui` open, an agent can read your view as data and move it. Every change is announced on
your screen, and your own keys always win. The running TUI is the server: a Unix socket with mode 0600 in a 0700
folder, local only, never TCP. There is no daemon.

Turn it off with `ownpurse tui --no-agent` or `"tui": {"agent": false}`. Then no socket is opened.

### Read through the state, never the screen

The state is complete: a fresh TUI built from it draws the same frame, cell for cell. An agent should work only
through `ui state` and `ui send`. `ui screen` is for showing you, or for a cross-check.

```sh
ownpurse ui ls                                   # running TUIs: id, label, pid, scope, tab
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

### Rules for agents

- **Read first, then send with `--expect-rev`.** Pass the `rev` from `ui state`. Exit 4 `view_changed` means the user
  moved since your read: read again and decide whether the change still makes sense. Exit 4 `user_busy` means they
  are typing or in an overlay: do not retry in a loop; wait and read again.
- **Say what you believe the view is.** `--expect PATH=VALUE` (also `PATH!=VALUE` and `PATH~TEXT`; repeatable) checks
  paths into `ui state` before any verb runs. If one fails, nothing is done: exit 4 `view_mismatch`, with `expected`
  and `actual`.
- **Read the answer.** A send replies with a `summary` of the view after it and one `effects` entry per verb
  (`{verb, changed, what}`). No second `ui state` is needed.
- **Read the exact twin of an amount.** Row `fields` carry every column as shown, in snake_case. Every shown amount
  has a `_value` twin (`debit_value`, `<org>_net_value`, …) that is an exact number. Never parse the display string.
- **Rows on screen only.** If `truncated` is true, read the rest with `ui state --all-rows` or `--rows A..B`. Both are
  read-only: the view does not move.
- **Errors carry the fix.** Every error is `{error, message, help, expected?, actual?}`. Follow `help`.
- **Say what you did.** Screenshots and sends show `◆ agent` in the header. Tell the user what you did and why.

### Verbs

The verbs are the whole allow-list. Sync, login, confirm, edits, quit, approve and apply are refused by the protocol
(exit 2), not just hidden. Ask the user to do those themselves.

| Verb | Effect |
|---|---|
| `tab <n\|name>` | Switch tab (`tab ledger`, `tab classify`) |
| `scope <alias\|group:x\|all>` | Switch scope |
| `asof <YYYY-MM-DD>` | Change the as-of date |
| `filter "<text>"`, `clear-filter` | Set or clear the filter (same meaning as `/`) |
| `select <#n\|anchor\|text>` | Select any row of the current list, on screen or not: `#n`, an anchor as JSON (`'{"account_code":"1010"}'`), or text |
| `drill`, `up` | Drill into the selected row; up one level (as `Esc`) |
| `back`, `forward` | Navigation history |
| `basis year\|history`, `differs on\|off` | Ledger basis; only accounts that differ |
| `compare <org,org…>` | Open Compare |
| `reload` | Re-read the record (zero calls) |
| `open` | Open the document on screen in Xero's web app, in the user's browser (a link, never an API call) |
| `caption "<text>"`, `caption clear` | A line in the user's footer (at most 120 characters, plain text) |
| `label <text>` | Label the session |
| `manifest`, `mode` | Show a manifest; change a tab's mode |
| `viz <spec-json>`, `viz close` | Open or close an explainer panel |

**Cosmetic verbs** are never refused while the user types, because they move no focus: `theme <name>`,
`accent <family|#hex>`, `overlay set <token>=<value>…`, `overlay clear`, `hint <anchor> <text>`,
`highlight <anchor>`, `spotlight <anchor>…`, `clear-hints`, `narrate <title> <body>` and `narrate clear`. An agent's
hints are attributed (`◆ name · Title`), and the reply says where each landed on screen. Walkthroughs are driven
with `walkthrough play <id>`, `step <n>`, `next`, `prev`, `stop`, and recorded with `walkthrough record start|step|stop`.

The one non-view action is `ownpurse ui upgrade`: a newer installed TUI takes over with the view restored.
`ownpurse ui flash` pulses the header to get the user's attention (at most three flashes a second; the previous look
is always restored).

### What the user sees

- `◇ agent` in the header while an agent is connected, `◆ agent` after it changed the view or took a screenshot.
- A status line saying what changed, for example `◆ agent: opened Ledger · scope roast · only accounts that differ`.
- `` ` `` (backtick) returns to the view before the agent's last change.
- A refused stale send shows `◇ agent: change skipped, you moved first`.
- Activity lists one row per agent read, screenshot and send.

### Classification writes through the bridge

The one kind of write an agent can make is a local classification label, never a change to Xero.
`ownpurse classify set|note|group …|undo|config add-project` sends the write through the running TUI for the
profile, so the user sees it and `user_busy` applies, or writes the store directly with `--direct` or when no TUI
runs. An agent's write never overwrites a label a person confirmed (unless `--overwrite`), touches at most 50 lines
(unless `--allow-bulk`), and can be undone by the user with `U`. See
[Classification](/docs/guides/writes/#classification).

### Error codes

| Code | Exit | Meaning |
|---|---:|---|
| `usage` | 2 | The command, a verb or its argument is malformed |
| `unknown_verb` | 2 | Not a view verb |
| `forbidden` | 2 | The bridge never does this (sync, login, quit, confirm, edits, keys). Ask the user |
| `not_applicable` | 2 | The verb means nothing on this screen. Move somewhere it applies |
| `not_found` | 2 | The row, anchor, line or group is not in the current list. Read `ui state --all-rows` |
| `bad_caption` | 2 | Empty, over 120 characters, or with control, escape or invisible characters |
| `bad_token` | 2 | A theme, accent or overlay token or value is unknown |
| `too_fast` | 2 | `ui flash` beyond the photosensitivity limits |
| `too_many` | 2 | A classify write would touch more than 50 lines |
| `filter` | 2 | A classify filter token is malformed |
| `bad_state` | 2 | The file given to `--from-state` is not a saved `ui state` |
| `not_available` | 2 | The request kind is reserved and not implemented |
| `protocol` | 2 | The CLI and the TUI speak different protocol versions. Reinstall both |
| `view_changed` | 4 | The view or store moved since the revision you sent. Read again |
| `view_mismatch` | 4 | An `--expect` predicate does not hold. Read again and decide |
| `user_busy` | 4 | The user is typing or in an overlay. Wait; never retry in a loop |
| `too_small` | 4 | The terminal is below 60×20. Ask the user to enlarge it |
| `timeout` | 4 | `ui wait` reached its timeout |
| `record_changed`, `config_changed`, `classify_changed` | 4 | A saved state was taken against another record, config or label store |
| `no_session` | 1 | No running TUI. Ask the user to start `ownpurse tui` |
| `bridge`, `closed` | 1 | The TUI did not answer, or is closing. Run `ownpurse ui ls` |
| `preflight_failed`, `exec_failed` | 1 | An upgrade could not take over. The old TUI carries on |
| `conflict`, `config`, `store`, `integrity` | 1 | Classification: the file changed on disk, is invalid, cannot be written, or its hash chain does not verify. On `integrity`, stop and tell the user |
| `record` | 1 | The local record could not be read. Run `ownpurse doctor` |
| `internal` | 1 | A bug. Report it with the command and `ownpurse ui state` |

`back` or `forward` with nothing to go to is not an error: exit 0 with `note: no_history`.

### Rendering a saved state

`ownpurse --json ui state > state.json` saves the view. `ownpurse ui screen --from-state state.json` (or
`ownpurse-tui --state state.json --format png --out view.png`) draws it again later from the state and the record
alone, with no live session. If the record, the config or the labels changed since, it refuses (`record_changed`,
`config_changed`, `classify_changed`, exit 4) unless you pass the matching `--allow-…-drift` flag, which marks the
render as drifted.

## The agent skill

An agent works best with a short skill that tells it when to use ownpurse and how. The CLI reserves a `skills`
command to install one; it is not available yet.

Until it is, the repository ships the bridge recipe as `docs/agent-bridge-skill-snippet.md`. Copy it into your
agent's skill for ownpurse (for Claude Code, a `SKILL.md` in a skill folder such as `~/.claude/skills/ownpurse/`),
together with a few lines on when to use the CLI: for any lookup in your books before reaching for raw API calls or
the provider's web app. Keep the rule from the snippet: work only through `ui state` and `ui send`.

## Your key bindings and agents

Your keys are the built-in table plus your `keys.toml`. It is yours to edit; an agent should not write it unless you
ask. `ui state` shows `keymap.revision` and `keymap.overrides`.
