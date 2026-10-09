---
title: Terminal UI
description: The read-only ownpurse terminal UI. Launching it, tabs, keys, scopes, compare, explainer panels, walkthroughs, themes, key bindings and in-place upgrades.
sidebar:
  order: 4
---

`ownpurse tui` is a **read-only** terminal interface over the local record. Opening it, moving around, drilling
down and switching organisations spend **zero API calls**. The only actions that reach Xero (Sync, Log in, Refetch
trial balance, Fetch history, Fetch attachments) live in the command palette and always ask first, with an estimate
of the calls they will use.

The TUI is built on [caretline](https://github.com/brancusi/caretline). It has one Elm-style state and one update
loop: every key, click, resize and agent request goes through the same `update`. That state is complete, so a
fresh TUI built from it draws the same frame cell for cell. This is what lets an agent read your view exactly and
lets an upgrade swap the running binary without losing your place.

Examples use invented organisation aliases (`roast`, `hold`, `design`, `lab`).

## Launching

```sh
cd path/to/your-project     # the folder holding ownpurse.json
ownpurse tui                # or: ownpurse-tui
```

The config is found by walking up from the current folder.

| Option | Effect |
|---|---|
| `--profile NAME` | Which profile in `ownpurse.json` |
| `--as-of YYYY-MM-DD` | The date the Ledger and parity use. Overrides `tui.as_of` |
| `--offline` | Network actions are not offered and any network request is refused. Guaranteed zero calls |
| `--theme NAME` | `ledger-dark`, `ledger-light`, `ledger-dark-256`, `ledger-light-256`, `ansi` or `no-color` |
| `--glyphs ascii` | ASCII instead of Unicode glyphs |
| `--no-mouse` | No mouse reporting |
| `--no-agent` | No agent bridge: no socket is opened |
| `--label <text>` | A label agents see in `ownpurse ui ls` |
| `--walkthrough <id>` | Start a walkthrough |
| `--keys markdown` | Print every key binding, your `keys.toml` included (also `json`, `conflicts`) |
| `--echo-keys` | Print the keys your terminal sends, spelled as `keys.toml` spells them |
| `--check-theme` | Validate your theme files |

**As-of date.** `--as-of`, then `tui.as_of`, then the last fiscal year-end of the scoped organisation. When neither
override is set, changing scope re-derives it for the new org. `d` changes it for the session.

**Terminal.** WezTerm is the reference terminal: it supports synchronized output, so redraws never tear.
Terminal.app works but can tear on large redraws. The minimum size is 60×20.

## Screen layout

```text
  ● roast ▾    as of 2025-12-31                                             ◦ read-only
  Overview  Ledger  Intercompany  Activity  System  Classify  Manifest
  …
  keys that work here                                            ? more  ⌃P palette
```

### Header

The header is calm unless something needs you.

| Field | Meaning |
|---|---|
| Scope chip | Always. One org: `● roast ▾`, in the org's accent. A pair: `● hold ↔ ● roast`. Compare: `●●● compare: a · b · c ▾`. A group: `group:ops (3) ●●● ▾`. All: `All organisations ●●●● ▾`. Without colour: `[roast] ▾` |
| `as of <date>` | Always |
| Exceptions | Only when they apply: `≠ record changed`, `offline`, the record age when stale or never synced, `classify: N open`, the agent mark (`◇` / `◆ agent`), calls left when low, offline or while a sync runs |
| `◦ read-only` | Always, at the right end |

### Footer

The footer shows this screen's keys first, then the global ones, at most five, then `? more` and the palette hint,
with the version at the right. Below 100 columns the global keys show without labels.

## Keys

### Always available

| Key | Action |
|---|---|
| `1` … `7` | Tabs: 1 Overview · 2 Ledger · 3 Intercompany · 4 Activity · 5 System · 6 Classify · 7 Manifest |
| `Tab` / `⇧Tab` | Next / previous tab |
| `g` | Scope picker |
| `[` / `]` | Previous / next scope: `all → group:<a> → … → <alias 1> → <alias 2> → … → all` |
| `d` | As-of date (where a tab uses one) |
| `/` | Filter the current table. `Esc` clears it |
| `↑` `↓` (`k` `j`), `PgUp` `PgDn`, `Home` `End` | Move |
| `Enter` | Drill down or open |
| `Esc` | Back up one level, close a filter, cancel a prompt or the confirm bar |
| `⌥←` / `⌥→` | Back / forward through the views you visited |
| `i` | Details: IDs, record versions, provenance sentences, budgets |
| `W` | Walkthroughs |
| `L` | Lay mode: account names before codes, "why this matters" lines |
| `r` | Reload from the record (no calls) |
| `U` | Undo an agent's classification writes (when there are any) |
| `?` | Help: every key, then your configured instructions |
| `⌃P` or `:` | Command palette |
| `q` | Quit |

### Contextual

| Key | Where | Action |
|---|---|---|
| `h` | Ledger, Compare | Toggle the basis: year basis ⇄ full history |
| `!` | Ledger trial balance | Show only accounts that differ from Xero |
| `v` | Ledger | Compare organisations side by side |
| `T` | Ledger account | T-account panel |
| `M` | Ledger account | Month-end timeline panel |
| `f` | Ledger postings, Compare postings | Date range `from[,to]` (`Esc` resets to the fiscal-year start) |
| `o` | A document, an intercompany item, a link | Open in the browser (Xero's web app, through the org switch) |
| `y` | A document, an intercompany item, a link | Copy the id or URL |
| `←` `→` | Overview, Compare | Move between cells or columns |
| `s` | Intercompany pairs | Sort |
| `a` / `b` | Intercompany bridge | That side's accounts in its own org's Ledger |
| `a` | Compare | Align rows by code ⇄ `account_map` category |
| `<` / `>`, `x` | Compare | Move a column, drop the column under the cursor |
| `m` | Compare postings | Mark the postings the intercompany engine already matched |
| `G` | Activity | Group calls by endpoint |
| `e` | System › Settings | Open the config in your editor |

Run `ownpurse-tui --keys markdown` for the full, current table, including every context and the keys you bound
yourself.

## Tabs

### 1 Overview

One row per organisation, read from the record and cached reports: whether the current login covers it, when it
was last fetched, parity with Xero's trial balance (year activity and full history), calls left today, and days
until the refresh token expires.

**Needs attention** lists only real conditions: no local record, a stale record, a low daily budget, intercompany
residuals, differences from Xero, unbalanced or uncoded postings, disconnected orgs, not logged in, or a refresh
token near expiry. Each line names the palette command that fixes it, for example `^p → Sync roast (≈ 14 calls)`.
Otherwise it says `Nothing needs attention.`

`Enter` on a row scopes to that org and opens the Ledger.

### 2 Ledger

Three levels with a breadcrumb (`roast › Trial balance at 2025-12-31 (accrual) › <account> › <document>`). `Esc`
goes back up, and each level keeps its cursor. The Ledger needs one org; under `all` or a group it asks you to pick
one.

**The basis strip**, under the breadcrumb, says how balances open and whether they match Xero:

| State | Text |
|---|---|
| Matches | `Opening from Xero TB <prior FY end> + <year> activity · matches Xero ✓` |
| Differs | `… · differs from Xero at <as-of>: N accounts` |
| Not compared | `… · not compared: no cached Xero TB at <as-of> (^p → Refetch)` |
| Full history (`h`) | `Local full-history balances: N accounts differ from Xero (marked !; usually pre-<year> setup balances)` |

Why two bases: Xero's API does not expose balances entered in its opening-balance screen or reconcile-screen
rounding, so balances rebuilt only from documents can disagree with Xero. The default year basis starts from Xero's
own trial balance at the prior fiscal year-end and adds local postings after it. See
[Concepts](/docs/start/concepts/#opening-balances-year-basis-and-full-history).

- **Level 1, trial balance**: flag, code, account, class, debit, credit, and a totals row (`balanced ✓` or
  `out of balance by …`). `!` marks an account that differs from Xero. Rows sit under class bands.
- **Level 2, account postings**: opening, debits, credits and closing, then each posting with its date, source
  (Spend, Receive, Transfer, Journal, Invoice, Bill, Credit note, Payment, …), reference, contact, description,
  amount and running balance.
- **Level 3, document**: the document's fields and lines, then its attachments and history as the record holds
  them. If they were never fetched, the palette offers `Fetch attachments…` or `Fetch history…` (1 call each).
  Opening a document never spends a call.

#### Compare (`v`)

Compare puts 2 to 4 organisations in columns, read-only and with zero calls. From the trial balance under `all` or a
small group, `v` opens it at once; otherwise the scope picker opens in multi-select mode (`space` toggles an org,
`Enter` opens with 2 to 4 selected). Orgs with different base currencies or fiscal years are refused.

- Each org has one column, with its own basis, record age and totals.
- Rows merge only on an identical account code (`align: code`) or an `account_map` category (`align: map`, rows
  marked `▸`). `a` toggles. A shared code with different names shows both names and a `≠`.
- `—` means the account does not exist in that org; blank means zero. There is no difference column and no
  cross-org total.
- `Enter` on an amount opens that org's Ledger. `Enter` on an account opens the postings side by side on one date
  spine.
- At 140 columns and wider, 4 orgs show debit and credit; at 100 to 139, they switch to one net column; below 100,
  2 orgs. Amounts never truncate: the layout drops to net, then to fewer orgs, and says so.

### 3 Intercompany

Pairs of organisations whose accounts should mirror each other, from your config. See
[Intercompany](/docs/guides/intercompany/#in-the-terminal-ui).

### 4 Activity

Every HTTP call the CLI or TUI made: time, org, method, endpoint, status, duration and calls left that day. Status
colours: 2xx dimmed, 4xx warning, 429, 5xx and network problems as errors. `G` groups by endpoint. Activity also
lists one row per agent read, screenshot and send (in memory only).

### 5 System

- **Connection**: token state (logged in, days until the refresh token expires, granted scopes) and one row per org
  with its Xero name, plan, records held, last sync and tenant id.
- **Settings**: every effective setting with its source (`defaults`, `config`, `profile:<name>`). `e` opens the
  config in `tui.editor`, `$VISUAL` or `$EDITOR`.
- **Links**: your configured links, expanded for your orgs and bank accounts. `o` or `Enter` opens, `y` copies.
- **Writes**: write packages with their stage and approval source, listed only when the record has any. `Enter`
  lists a package's operations.

### 6 Classify

Local classification of Xero lines: project, bearer, treatment and should-be account, with notes and groups.
Labels never change Xero. See [Classification](/docs/guides/writes/#classification).

The **Queue** lists lines with an unconfirmed required label or a stale one: stale lines first, then the largest
amounts.

| Key | Action |
|---|---|
| `Enter` / `a` | Accept every suggested label on the line, then move on |
| `p` then `1`–`9` | Set the project |
| `b` / `t` / `s` | Bearer, treatment or should-be account picker (type to filter) |
| `G` | Add to a group |
| `n` | Note (`⌃E` for multi-line, `⌃S` saves) |
| `x` | Skip (no write) |
| `u` | Undo the last write on the line |
| `A` | Bulk-accept preview: every line in the filter whose suggestions are all at or above the threshold (`+` / `-`, default 0.90). Cancel has focus |
| `/` | Filter: `org:roast acct:60 month:2025-09 project:harvest conf<0.7 status:open\|all\|stale by:agent since:today`; bare words search payee and description |
| `,` / `.` | Previous / next mode: Queue, Groups, Audition, Tree, Catalog |

**Groups** checks chains of lines across orgs: one-to-one, many-to-one with the share explained, or a running
balance of settlements. **Catalog** edits projects and rules with a live "would match" count, a staged diff and a
confirm (`w` writes, `u` discards); archiving replaces deleting.

### 7 Manifest

A manifest is a set of proposed changes to the books. This tab shows its entries, issues, effects and log, a stage
strip (draft, check, planned, approved, applying, applied), and the exact approve and apply commands to run
yourself. The TUI never applies anything. See [Writes](/docs/guides/writes/).

| Key | Action |
|---|---|
| `b` | Switch manifest |
| `N` | New manifest |
| `c` | Check |
| `P` | Plan |
| `y` | Copy the shown command |
| `,` / `.` | Previous / next mode |
| `Enter` | Entries: open the field diff. Issues: go to the field |
| `u`, `e`, `n` | Entries: undo, edit, annotate |
| `a` | Acknowledge a warning |
| `z`, `J` | Diff: all fields, raw patch |

While a manifest is shown, the Ledger and statements show proposed values, and the header carries a chip such as
`7 changes · 1 issue`.

## Switching scope

`g` opens the picker with focus in the filter. Sections, in order: **Pinned** (`tui.pinned_orgs`), **Recent**,
**Groups**, **All orgs**, then `all`. Each org row shows its alias, Xero name, record age and calls left. Typing
ranks every scope in one list: exact alias, alias prefix, short-code prefix, a word prefix in the Xero name, then
fuzzy. `Enter` switches; `Esc` closes. The palette has the same entries as `Switch to <alias>`.

Switching never calls Xero. Each scope keeps its own Ledger level, account, cursors, basis, Intercompany level, sort
and filters, so switching back lands where you left it.

## Filtering

`/` and the agent verb `filter` mean the same thing: a case-insensitive substring, with spaces and punctuation
literal and no fuzzy matching. `filter "Kiln Room -"` lists `Kiln Room - Afterburner` but not `Kiln - Operating`.
`code:2410` and `code:1450..1480` match account codes, exactly or as an inclusive range. The strip shows the count:
`filter: "Kiln Room -" · 9 of 43`. The trial balance's totals still cover all accounts.

## Network actions (command palette)

Never bound to a key. `⌃P`, choose, then confirm.

| Palette entry | What it does | Calls |
|---|---|---|
| `Sync <alias>…` | Incremental sync of that org | Estimated from the record |
| `Sync all orgs…` | The same for every org | The sum |
| `Log in to Xero…` | Browser consent, read access only | Token exchange + connections |
| `Refetch Xero trial balance for parity…` | Xero's trial balance at the as-of date (and the prior year-end) for the scoped orgs | 1–2 per org |
| `Fetch history…` | Only while a document is open | 1 |
| `Fetch attachments…` | Only while a document is open | 1 |

The confirm bar starts with focus on **Cancel**:
`Sync roast from Xero? ≈ 15 calls · 978 left today for roast.   (y to confirm)`. A refusal (not enough calls left,
or offline mode) stays in the same bar until you close it. After an action completes, a note reports the calls used
and every view reloads from the record.

## Explainer panels

A panel explains one thing with your books' own numbers, drawn over the current tab. Nothing is typed in: the numbers
come from the same ledgers, basis and intercompany engine as the tabs.

| Panel | Shows | Opened by |
|---|---|---|
| Bridge | An intercompany gap as a waterfall, down to what is left unexplained | Palette "Explain: bridge <pair>" |
| Flow | One box per org and one arrow per pair: who owes whom, or what moved | Palette "Explain: who owes whom", "Explain: what moved…" |
| T-account | Debits left, credits right, the opening, the largest postings, the totals and the closing | `T` on a Ledger account |
| Timeline | Month-end balances as bars, each month's movement, where manual journals landed | `M` on a Ledger account |
| Before / after | What a manifest changes in one org: the trial-balance lines and statement totals that move | Palette, with a manifest shown |
| Checklist | The close worklist from a JSON file: deadlines, questions by owner, agent work, postings by state | Palette "Explain: the close worklist" (`tui.checklist`) |

`Esc` closes a panel; arrows, PgUp/PgDn and Home/End scroll it. `ownpurse viz <kind> …` prints the same numbers as
text or `--json`.

## Walkthroughs

Walkthroughs are guided, replayable explanations played over the real screen: a narration panel in plain English,
with hints, rings and spotlights pointing at what is on screen. They are read-only: a step uses view verbs only and
can never approve, apply or write. `ownpurse demo` plays one over a mock database.

**Playing.** `W` (or the palette's "Walkthroughs…") lists them; `ownpurse-tui --walkthrough <id>` starts one.

| Key | Action |
|---|---|
| `→`, `n`, `Space` | Next step |
| `←`, `p` | Previous step |
| `[` / `]` | Scrub |
| `g` then a number | Jump to a step |
| `t` | Contents |
| `Home` / `End` | First / last step |
| `r` | Re-apply the step's view |
| `?` | The first glossary term's definition |
| `PgDn` / `PgUp` | Scroll a long narration (it is never cut) |
| `Esc` | Leave, and restore the view from before |

Each step's view is applied over a neutral view, so playing, jumping and reloading give the same frame.

**Files.** Walkthroughs are JSON files in `walkthroughs.dir` (default `walkthroughs/` beside `ownpurse.json`), in
caretline-tour's format. Each step has a narration, layers anchored to what is on screen, and a `host` block: the
view to apply, `checks` such as `amount(org, account, date) = value` (a failing check shows a "the books changed"
band, never silently wrong numbers), a lay-mode `why`, and an optional explainer panel.

**Writing them.** `ownpurse walkthrough new <id> --title T`, then `add-step` for each step, `validate` (every step
laid out at 140×40, 100×30 and 80×24: anchors resolve, checks hold, glossary links exist, nothing covers a
protected region) and `render` (each step's frame as text, SVG, PNG or HTML). An agent can also record one live
with `walkthrough record start <id>`, `walkthrough record step` and `walkthrough record stop`.

**Glossary.** `[term]` links in narration open plain-English definitions from the shipped glossary plus your
project's `glossary.toml` beside `ownpurse.json`, which adds to or replaces the shipped entries.

## Themes

`tui.theme` is `auto`, a built-in (`ledger-dark`, `ledger-light`) or a theme of your own in
`~/.config/ownpurse/themes/<name>.toml`:

```toml
[meta]
name    = "warm"           # must match the file name
extends = "ledger-dark"    # a built-in, one level
[palette]
slate.950 = { hex = "#161412" }
[semantic]
bg = "@slate.950"
[component]
"section.band.bg" = "mix($surface.section, $accent, 0.04)"
[accents.ember]
dark  = { hex = "#F08A55" }
light = { hex = "#A8461A" }
```

Values are `#hex`, `@palette.name`, `@accent.<family>`, `$token` or `mix(a, b, t)` (where `t` is a number or
`knob.<name>`). The file is watched: a valid save applies at once; an invalid one keeps the previous theme and names
the file, line and problem, for example
`themes/warm.toml:9: semantic.bg = "@slate.95": unknown palette entry (did you mean "slate.950"?)`.
`NO_COLOR` keeps a plain rendering. Every built-in theme keeps body text at a contrast of at least 4.5:1.

### Scope accents

Each org has an accent family (`harbor`, `ember`, `iris`, `rose`, `sea`, `graphite`), set in `tui.accents`
(`{"roast": "ember", "cross": "harbor"}`). Unlisted orgs take the unused ones in the order ember, iris, rose, sea.
Views across several orgs use the cross accent (harbor). The accent shows in the scope chip, the active tab, the
selection gutter and a few light tints; never in amounts, status words or table text.

`tui.accent_intensity` sets the tints, as one number or `{"dark": …, "light": …}`: `header_band` (default 0.10 dark,
0.06 light), `section_band`, `breadcrumb`, `selection` (0.04), `compare_columns`, `status_warn`, `status_err` (0 is
off), plus `header_gradient` (true/false) and `pair_band` (`neutral`/`split`). Each has a computed ceiling that keeps
text at 4.5:1; a value above it is applied, with a warning in System › Settings.

### The focused view

`tui.view` controls how much the screen says:

| Key | Default | Meaning |
|---|---|---|
| `details` | `false` | Start with the details layer on (`i` toggles it) |
| `budget_in_header` | `low` | `always`, `low` (under the warning line, or offline) or `never` |
| `record_age_in_header` | `stale` | `always` or `stale` |
| `focus_line` | `true` | A quiet second line under the selected row |
| `quiet_ok` | `true` | A healthy ✓ in the quiet tone |
| `group_tb_by_class` | `true` | Trial balance rows under class bands |
| `density` | `comfortable` | `compact` drops the blank rows between sections |

## Your own key bindings

The built-in table is the default. `~/.config/ownpurse/keys.toml` (or `tui.keys_file`) layers your bindings on top.
It is personal, not per project.

```toml
v = 1

[[bindings]]
key = "cmd+["               # ⌘[ : back through the views you visited
action = "history.back"
context = "global"           # optional; global by default

[[bindings]]
key = "cmd+]"
action = "history.forward"

[[unbind]]                   # remove a built-in key (then bind the action elsewhere)
key = "q"
```

- **Keys**: `ctrl`, `alt`, `shift` and `cmd` (or `super`) joined with `+` to a character, `space`, `enter`, `esc`,
  `tab`, `backspace`, an arrow, `home`, `end`, `pageup` or `pagedown`.
- **Actions and contexts** are the names `ownpurse-tui --keys markdown` prints. A binding adds a key; the built-in
  key stays unless you unbind it.
- **Saving reloads it at once.** The footer, help (your keys carry `*`), palette hints and `--keys` follow on the
  next frame.
- **An invalid file** keeps the previous keymap and shows the file and line, for example
  ``keys.toml:4: bindings[0].action: unknown action `histroy.back` (did you mean `history.back`?)``. Refused: an
  unknown action or context, a bad key, a key with two meanings, a network action (they stay in the palette), the
  confirm bar's keys, and unbinding a key that is not bound.
- **⌘ keys** need a terminal that reports them (WezTerm with `enable_kitty_keyboard = true`, Ghostty, kitty).
  `ownpurse-tui --echo-keys` shows what your terminal sends.

## Live changes from elsewhere

The TUI follows changes made outside it, without a polling timer: the operating system reports file changes.

- A `sync` in another terminal, or a classification write, refreshes the view in place and says what arrived:
  `record updated · 3 new versions`.
- Saving `ownpurse.json` re-reads it with your view kept. An invalid file keeps the last valid config behind an
  error band until the next valid save.

An idle TUI writes nothing to the screen.

## In-place upgrade

When a newer `ownpurse-tui` is installed while one runs, the footer says so: `v0.16.0  ⬆ v0.16.1 ready · ⌃P Upgrade`.
Choosing **Upgrade** in the palette (or `ownpurse ui upgrade` from a shell or an agent) replaces the running TUI with
the new build in the same terminal and puts you back exactly where you were: tab, scope, drill level, filter, a
half-typed input and your history.

Before it switches, the new build checks that it can take over: it imports your view state and exports it again,
and every field must survive. Within the same visual design, the new build's first frame must also match the live
one row for row. If a check fails, nothing happens: the old build carries on and the footer says what to do. While
a sync or login runs, the upgrade waits and applies when it finishes.

## Troubleshooting

| Symptom | Fix |
|---|---|
| `No ownpurse config found …` | `cd` into the folder that holds `ownpurse.json`, or `export OWNPURSE_CONFIG=/path/to/ownpurse.json` |
| Header says `no local record` | Never synced: `⌃P → Sync <org>…` or `ownpurse sync <org>` |
| Record age shows `stale` | Older than `sync.ttl_seconds.default`. Sync if you need newer data |
| Ledger strip says `not compared` | No cached Xero trial balance at the as-of date: `⌃P → Refetch Xero trial balance for parity…` (1–2 calls) or `ownpurse report fetch <org> trialbalance --date <as-of>` |
| Accounts marked `!` in full history | Expected where Xero holds setup or conversion balances the API does not expose. Use the default year basis |
| `Not enough calls left today …` | The daily budget is nearly used. Xero resets each org's limit at its own time. Everything else keeps working offline |
| `Not logged in` | `⌃P → Log in to Xero…` |
| "Open in Xero" lands on the wrong page | Document link templates ship unverified. Open the org in Xero first, or fix the template under `links` |
| Terminal too small | The TUI needs at least 60×20 |

## For agents

An agent can read the TUI's exact state and move the view with view-only verbs. See
[Working with agents](/docs/guides/agents/).
