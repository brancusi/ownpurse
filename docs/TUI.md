# ownpurse TUI — reference

`ownpurse tui` is a **read-only** terminal interface over the local record (the SQLite file ownpurse
keeps of everything it has read from Xero). It never writes to Xero. Opening it, moving around,
drilling down and switching organisations spend **zero API calls**. The only actions that reach Xero —
Sync, Log in, Refetch trial balance, Fetch history, Fetch attachments — live in the command palette
and always ask first.

Installed with the CLI by `./install.sh` (the `ownpurse-tui` binary; `ownpurse tui` starts it).

Examples below use generic organisation aliases (`roast`, `hold`, `design`, `lab`).

---

## Launching

```
ownpurse tui [--as-of YYYY-MM-DD] [--profile NAME]
```

| Flag | Effect |
|---|---|
| `--as-of YYYY-MM-DD` | Date used by the Ledger and Parity tabs. Overrides config `tui.as_of`. |
| `--profile NAME` | Which profile in `ownpurse.json` (org aliases, token file, record file). Default: `default_profile`. |
| `--offline` | Offline mode: the header shows **OFFLINE** next to READ-ONLY, the network palette commands (Sync, Log in, Refetch, Fetch) are not offered, and any network request is refused. Guaranteed zero calls. |
| `--fresh`, `--json`, `--full`, `--limit` | Global CLI flags; no effect in the TUI (it never syncs on its own). |

**As-of date resolution:** `--as-of` → config `tui.as_of` → the last fiscal year-end of the scoped
organisation (from Xero's financial-year settings; the first configured org when the scope is `all`).
When neither override is set, cycling the org scope re-derives it for the new org. `d` changes it for
the current session.

---

## Screen layout

```
header   │ ownpurse · <profile> · v0.5.2  READ-ONLY  org: roast ▾  as of 2025-12-31  record 07:46 UTC · 2h ago  938/1,000 left
tabs     │ Overview  Ledger  Parity  Activity  Usage  Organisations  Settings  Links
content  │ …
footer   │ keys that work here                                                            ^p palette
```

### Header fields (v0.12.0: calm unless something's wrong)

```
  ● roast ▾    as of 2025-12-31                                             ◦ read-only
  ● roast ▾    as of 2025-12-31    stale · 07:46Z · 3d    84/1,000 left     ◦ read-only
```

| Field | Meaning |
|---|---|
| scope chip | Always. One org: `● roast ▾`, the dot and name in the org's accent. A pair (Compare with two orgs, an Intercompany pair, another org's books from a bridge): `● hold ↔ ● roast`. Compare with 3–4 orgs: `●●● compare: a · b · c ▾` (compact `●●● compare (3) ▾`). A group: `group:ops (3) ●●● ▾`. All: `All organisations ●●●● ▾`. Plain text (NO_COLOR): `[roast] ▾`, `[hold ↔ roast]`, with no dots. `g` opens the picker; `[` / `]` cycle. |
| `as of <date>` | Always. Below 100 columns: just the date. |
| exceptions | Only when they apply, after the date: the drift chip (`≠ record changed`, warning), `offline`, the record age when stale or never synced (`tui.view.record_age_in_header`), `classify: N open`, the agent mark (`◇` / `◆ agent`), calls left when under `limits.warn_day_remaining`, offline or while a sync runs (`tui.view.budget_in_header`). |
| `◦ read-only` | Always, quietly, at the right end: a statement of what the TUI can do, not a mode. |

### Scope accents (v0.12.0)

Each org has an accent family (`harbor`, `ember`, `iris`, `rose`, `sea`, `graphite`), set in
`tui.accents` (`{"roast": "ember", "cross": "harbor"}`); unlisted orgs take the unused ones in the order
ember, iris, rose, sea. Views across several orgs use the cross accent (harbor). The accent appears in the
scope chip, the active-tab underline, the selection gutter `▌`, and the knob-driven tints below; never in
amounts, status words or table text. The scope alone decides it (`accent` in `ownpurse ui state`).

**Intensities** (`tui.accent_intensity`, a number for both or `{"dark": …, "light": …}`):
`header_band` (default 0.10 / 0.06), `section_band`, `breadcrumb`, `selection` (0.04 / 0.04),
`compare_columns`, `status_warn`, `status_err` (each 0 = off), `header_gradient` (true/false) and
`pair_band` (`neutral` / `split`). Each has a computed ceiling (the largest value keeping text ≥ 4.5:1 on that
surface for every accent); a configured value above it is applied, with a warning in System › Settings.

### Themes (v0.12.0)

`tui.theme`: `auto`, a built-in (`ledger-dark`, `ledger-light`) or a user theme in
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

Expressions: `#hex`, `@palette.name`, `@accent.<family>`, `$token`, `mix(a, b, t)` (t a number or
`knob.<name>`). The file is watched: a valid save re-applies it (`theme.revision` increments); an invalid
one keeps the previous theme and shows `themes/warm.toml:9: semantic.bg = "@slate.95": unknown palette entry
(did you mean "slate.950"?)` in the error band. A state carries the theme file's text, so a render from
state never reads the file.

### The focused view (`tui.view`, v0.12.0)

| Key | Default | Meaning |
|---|---|---|
| `details` | `false` | `i` toggles the details layer: IDs, record versions and links, provenance sentences, Overview's record times and calls left, Activity's method, HTTP code and per-call budget |
| `budget_in_header` | `low` | `always` · `low` (under the warning line, or offline) · `never` |
| `record_age_in_header` | `stale` | `always` · `stale` |
| `focus_line` | `true` | the quiet second line under the selected row (where Enter goes; Compare: the row's per-org names) |
| `quiet_ok` | `true` | a healthy ✓ in the quiet tone |
| `group_tb_by_class` | `true` | trial balance rows under class bands, each with its net |
| `density` | `comfortable` | `compact` drops the blank rows between sections |

A healthy state sits at the breadcrumb's right (`✓ matches Xero`, `✓ fully explained`, or
`263.94 unexplained` in the error colour); the sentence behind it is under `i`.

---

## Keys

### Always available

| Key | Action |
|---|---|
| `1`…`9` | Jump to tab: 1 Overview · 2 Ledger · 3 Parity · 4 Intercompany · 5 Activity · 6 Usage · 7 Organisations · 8 Settings · 9 Links |
| `g` | Scope picker (see *Switching scope*); footer label `Scope` |
| `[` / `]` | Cycle scope backward / forward: `all → group:<a> → … → <alias 1> → <alias 2> → … → all` (wraps both ways; groups in config order, then configured aliases, then any other connected orgs) |
| `/` | Filter the current tab's main table (Activity's filter is always visible). `Esc` clears and hides it. |
| `Enter` | Drill down / open (Overview row → Ledger for that org; Ledger levels; Parity rows; Links) |
| `Esc` | Back up one level, close a filter, cancel the confirm bar or an input |
| `i` | Details (v0.12.0): show or hide the details layer: IDs, record versions and links, provenance sentences, budgets (`tui.view.details`) |
| `r` | Reload from the local record (no calls) |
| `?` | Help overlay (keys + the configured instructions) |
| `ctrl+p` | Command palette (network actions live here) |
| `q` | Quit |

The footer (v0.12.0) shows this screen's keys first, then the global ones, at most five, then `? more` and the
palette hint, with the version at the right. Below 100 columns the global keys go key-only (`q / ]`);
contextual keys keep their labels.

### Your own key bindings: `keys.toml` (v0.12.0)

The built-in table is the default. `~/.config/ownpurse/keys.toml` (or `$XDG_CONFIG_HOME/ownpurse/keys.toml`;
`tui.keys_file` in ownpurse.json points elsewhere) layers your bindings on top. It's personal, not per-project,
and the same folder holds your themes.

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

- **Keys:** `ctrl`, `alt`, `shift` and `cmd` (or `super`) joined with `+` to a key: a character, `space`,
  `enter`, `esc`, `tab`, `backspace`, the arrows, `home`, `end`, `pageup`, `pagedown`.
- **Actions and contexts** are the names `ownpurse-tui --keys markdown` prints. A binding adds a key to the
  action in that context (the built-in key stays unless you unbind it); an action you rebind after unbinding
  its key keeps its place in the footer.
- **Saving reloads it at once:** the footer, help (your keys carry `*`), palette hints and `--keys` follow on
  the next frame, with the note `keys reloaded (N overrides)`. Nothing polls: the OS reports the save.
- **A file that isn't valid** keeps the previous keymap and shows a band naming the file and line, e.g.
  `keys.toml:4: bindings[0].action: unknown action \`histroy.back\` (did you mean \`history.back\`?)`, until the
  next valid save. Refused: an unknown action or context, a bad key, a key with two meanings in a layer or
  across the stack, a network action (sync, login, refetch: they stay in the palette), the confirm band's keys,
  an unbind of a key that isn't bound, an unknown `v`.
- **⌘ keys** need a terminal that reports them (keyboard enhancement, e.g. WezTerm with
  `enable_kitty_keyboard = true`, Ghostty, kitty). Without it, a keymap with a `cmd+` binding shows once: *Cmd
  bindings won't work in this terminal (no keyboard enhancement)*. `ownpurse-tui --echo-keys` prints each key
  as keys.toml spells it, to check what your terminal sends.
- **State:** `ui state` shows `keymap.revision` (the file's hash) and `keymap.overrides`; the canonical state
  carries the file, so a render from state needs no keys.toml.

### Contextual (shown in the footer only where they work)

| Key | Where | Action |
|---|---|---|
| `d` | Overview, Parity, Intercompany, Ledger L1/L2 | Change the as-of date (inline input; `Enter` applies, `Esc` cancels) |
| `h` | Ledger L1/L2 | Toggle balance basis. Footer label is the mode you would switch to: **`h Full history`** (from the default year basis) or **`h Year basis`** (back) |
| `f` | Ledger L2 | Postings range: `from[,to]` (`Esc` resets to the fiscal-year start) |
| `m` | Parity | Toggle **year activity** ⇄ **full history** |
| `s` | Intercompany pairs | Sort: status (unexplained first, then largest difference) → difference → pair name; never drops rows |
| `v` | Ledger L1/L2 | Compare orgs side by side |
| `a` | Compare | Align rows by code ⇄ account_map category |
| `<` / `>`, `x` | Compare | Reorder columns, drop the column under the cursor |
| `a` / `b` (footer `A` / `B`) | Intercompany bridge | That side's paired accounts in the Ledger of its own org (one account → L2, several → L1 filtered to them) |
| `o` | Ledger L3, Intercompany bridge item, Links | Open the document / link in the browser |
| `c` | Ledger L3, Intercompany bridge item, Links | Copy the document ID / link URL |
| `e` | Settings | Open the config file in `tui.editor` / `$VISUAL` / `$EDITOR` (restart to apply) |
| `y` | Confirm bar | Confirm the pending network action |

---

## Tabs

### 1 Overview (default)

One row per organisation; read only from the local record and cached reports.

| Column | Meaning |
|---|---|
| org | Alias |
| connected | Whether the current Xero login covers this org (hidden below 100 columns; a disconnected org becomes an attention item) |
| record fetched | Last successful sync (`fetched` below 100 columns) |
| parity (year) | Local year activity vs Xero's trial balance at the as-of date: `✓ exact`, `N accounts differ`, or `— not checked` (no cached Xero TBs) |
| parity (history) | Same, built from full local history |
| calls left | Calls left today / daily limit |
| token | Days until the refresh token expires (60-day rolling window) |

**Needs attention** lists only real conditions: no local record, stale record, low daily budget,
intercompany residuals or pairs not compared/supported (see 4 Intercompany),
year-activity differences from Xero, unbalanced or uncoded postings, disconnected orgs, not logged
in, refresh token past `limits.refresh_token_warn_days`. Each line names the palette command that
fixes it, e.g. `^p → Sync beta (≈ 14 calls)`. Otherwise: `Nothing needs attention.`

`Enter` on a row scopes to that org and opens the Ledger.

### 2 Ledger (drill-down)

Three levels with a breadcrumb (`roast › Trial balance at 2025-12-31 (accrual) › <account> › <document>`)
and `Esc` to go back up; the cursor position of each level is kept. The Ledger needs one org: under
scope `all` or a group it shows `Ledger needs one org: press g or ] to choose (…)`.

**Basis strip** — pinned under the breadcrumb, recomputed on every render against the cached Xero
trial balance at the current as-of date:

| State | Text | Colour |
|---|---|---|
| Matches | `Opening from Xero TB <prior FY end> + <year> activity · matches Xero ✓` | normal |
| Differs | `Opening from Xero TB <prior FY end> + <year> activity · differs from Xero at <as-of>: N accounts` | warning |
| Not compared | `Opening from Xero TB <prior FY end> + <year> activity · not compared: no cached Xero TB at <as-of> (^p → Refetch)` | warning |
| Full history (`h`) | `Local full-history balances: N accounts differ from Xero (marked !; usually pre-<year> setup balances). ^p → Refetch TB to open from Xero.` (or `… · matches Xero ✓`) | warning unless it matches |
| No opening TB cached | `Local full-history balances — no cached Xero TB to compare; may differ from Xero. ^p → Refetch TB to open from Xero.` | warning |

Why two bases: Xero's API does not expose balances entered in its opening-balance (conversion)
screen or reconcile-screen rounding, so balances rebuilt only from documents can disagree with Xero.
The default **year basis** starts from Xero's own trial balance at the prior fiscal year-end and adds
local postings after it — the basis `ownpurse parity --since` proves exact.

**L1 — Trial balance:** `! · code · account · class · debit · credit`, totals row
`Total debit … credit … balanced ✓` (or `out of balance by …`). `!` marks an account that differs from
Xero; accounts only Xero holds appear at zero, flagged. Below 100 columns `class` is hidden and long
account names are shortened with `…`. `Enter` → L2.

**L2 — Account postings:** strip `opening <amount> from Xero TB <date>` (or `from local history`)
`· debits · credits · closing`. Columns `date · source · reference · contact · description · debit ·
credit · balance` (running). Source labels: Spend, Receive, Transfer, Journal, Invoice, Bill, Credit
note, Payment, Overpayment, Prepayment. Below 100 columns `reference` and `description` are hidden.
`f` sets the range. `Enter` → L3.

**L3 — Document:** left pane — Date, Contact, Type (e.g. *Spend money*), Status, Bank account,
Reference, Narration, Amounts (*tax inclusive / tax exclusive / no tax*), Total, Reconciled; the ID
(middle-shortened below 100 columns) and `o opens in Xero (link unverified)` when the link template is
not verified; then the lines table (`description · account (code · name) · amount · tax`; tax hidden
below 100 columns). Right pane — Attachments (names and sizes) and History, both from the record:
`Local record: vN, synced …` then Xero's history entries. If never fetched:
`Attachments not fetched yet. ^p → Fetch attachments (1 call)` / `History not fetched yet. ^p → Fetch
history (1 call)`. Opening a document never spends a call. Below 80 columns the panes stack.

#### Side-by-side compare (`v`)

Compare 2–4 organisations in columns, read-only and with zero calls. From Ledger L1 under `all` or a group of
2–4 orgs, `v` opens straight away; from one org (or a larger group) it opens the scope picker in multi-select
mode — `space` toggles an org (a group toggles its members), `Enter` opens with 2–4 selected, and the hint
says why Enter is disabled otherwise. Palette: `Compare orgs side by side…`. Different base currencies or
fiscal years are refused in the confirm-bar area, naming the orgs; nothing renders.

**Trial balances (L1).** Header scope `compare: <org> · <org> ▾` with the oldest record age and the lowest
calls left. Each org has one column, headed by the org name centred over its debit and credit sub-fields
(or one signed `net` field) and separated from the next org by a faint `│`; the column header shows a
compact basis (`TB+25 ✓`, `differs: N` or `not compared` in warning colour, whose cells
are then muted); the strip spells out each org's basis and record age. Rows merge **only** on an identical
account code (`align: code`, default) or an `account_map` category (`align: map`, rows marked `▸`); `a`
toggles. Identical names under different codes never merge, code-less accounts never merge in code mode,
and a shared code with different names shows both names and a `≠` after the code. The line above the table
shows the selected row's own name in every org (`1060 · roast Tallowmere Green-Bean Card · design Quillfeather Site-Visit Card
· lab SALTMARSH SENSORY PANEL CARD™ ≠ names differ`). In map mode with no `account_map` configured, the strip says
so ("nothing merges; every account sits in its org's section"); a category may carry a `label`. Unmerged accounts sit in `── only in <org> ──`
sections. `—` = the account does not exist in that org; blank = zero. There is **no difference column and
no cross-org total**: each org keeps its own totals row (balanced ✓). `<` / `>` reorder columns, `x` drops
the column under the cursor (min 2), `h` switches every column's basis, `/` filters.

`Enter` on an amount opens that org's Ledger (L2 for one account, L1 filtered to the members of a map row);
the breadcrumb starts `compare › <org>` and `Esc` returns to the compare view with the cursor kept. `Enter`
on an account label opens **postings side by side (L2)**: one date spine, each org's postings in its column,
same-day postings stacked, per-org opening and closing on the Ledger L2 basis; no pairing is inferred.
`m` marks (`≡`, accent) the postings the Intercompany engine itself already matched for pairs whose orgs are
both shown — read from the engine's `matched` output, never computed here. `f`
sets the range for every column; `Enter` on a posting opens that org's document. `Esc` steps back to L1, then
to where `v` was pressed.

**Widths.** 140+: up to 4 orgs with debit and credit; 100–139: 4 orgs switch to `net`; below 100: 2 orgs,
net, and the strip says "widen the terminal to compare more orgs". The layout also checks the widest amount:
if it would not fit, it switches to net, then shows fewer orgs and says so — amounts never truncate.

CLI: `ownpurse tb a,b --compare [--align code|map] [--json]` returns the same rows and cells.

### 3 Parity

Summary per org from **cached** Xero trial balances (no calls on open):
`org · match · differences · net difference · postings · issues · Xero TB fetched`
(`issues` = `0` or e.g. `2 unbalanced · 5 uncoded`; below 100 columns `postings` and the fetched time
are hidden). `m` toggles **year activity** (opening from Xero TB at the prior FY end) and **full
history**; the mode is shown above the table. `Enter` on an org lists differing accounts
(`account · local · xero · difference`); `Enter` on one opens its postings in the Ledger (L2). In full
history mode with differences, a muted hint explains they usually come from opening/conversion
balances.

### 4 Intercompany

Pairs of organisations whose accounts should mirror each other, from config `intercompany.pairs` and
`parties` (docs/ROADMAP.md §2). The same engine as `ownpurse ic`; read-only and zero calls (open, drill,
back, scope switch and as-of change are tested). Nothing is inferred to force a match: the tab shows what
the engine computed and never offers to balance, eliminate or post.

**Scope:** `all` shows every pair; an org shows pairs where it is either side; `group:<name>` shows pairs
with both sides in the group. Empty: `No intercompany pairs involve <scope>.`; nothing configured:
`No intercompany pairs configured. Add "intercompany.pairs" to ownpurse.json (docs/ROADMAP.md §2).`

**Level 1 · pairs.** Pinned strip: `Intercompany at <as-of> · balances: cached Xero TBs (oldest fetched
<time>) · items: documents`, plus `· not compared for <orgs>: no cached Xero TB at <date> (^p → Refetch)`
in warning colour when a TB is missing. Summary: `N pairs · N matches · N bridged · N unexplained`
(unexplained in error colour when > 0). Columns: `pair · side A · side B (party) · difference · residual ·
status`; below 100 columns only `pair · difference · residual · status` — the residual never hides.

| Status | Meaning |
|---|---|
| `✓ matches` | Difference 0.00 |
| `bridged` | Difference ≠ 0, fully explained: residual 0.00 |
| `unexplained` (error colour) | Residual ≠ 0 |
| `— not compared` (warning) | No cached Xero TB at the as-of date or the prior year-end; cannot be opened |
| `not supported: <reason>` (warning) | Different base currencies or fiscal years, or an unknown org; cannot be opened |

`Enter` opens the bridge; `/` filters by pair name or org; `s` sorts.

**Level 2 · bridge.** Strip: `residual <amount> ✓ · balances: Xero TB <org> <time> · … · items from
documents` (residual first; strips wrap rather than cut). Side panels: org, net (Dr/Cr), paired
accounts (ellipsized; full list via `a` / `b`), and for side B the party's share and its method
(`residual` = account total less other parties' attributed postings, `attributed` = only postings
attributed to the party), the account total and other parties in the account. Panels stack below 100
columns; below 80 the account list collapses to a count. The bridge table always shows every row, even at
zero: difference at year end = prior-year opening difference + this year's movement difference; explained
by N items; then, under a rule, the unexplained residual in bold (error colour when not zero). Items: `side · date · org · account · contact · amount · why`, sorted by
date; the selected item's full `why` and document id show under the table. Known items show `✓ found` or
`! not found in the record`; context accounts are muted. `Esc` returns to Level 1 with the cursor kept.

**Level 3 · the posting in its own org.** `Enter` on an item shows the document view (the same one as
Ledger L3) inside this tab, for that item's org, without changing the scope. The header shows
`org: all ▾ · via <pair-id>` and the breadcrumb `<pair> › bridge › <org> › <account> › Spend 2025-10-21`
(middle segments become `…` when it doesn't fit, so the document always shows). `2` still opens the real
Ledger at the current scope. `Esc` returns to the bridge with the item cursor kept; a second `Esc` returns
to Level 1. `a` / `b` on the bridge open the Ledger tab for that side's accounts (header `· via <pair-id>`;
`Esc` returns to the bridge). Palette `Fetch history…` /
`Fetch attachments…` work as in Ledger L3 (1 call each, confirm first).

**Overview:** with pairs configured, a line under the org table, `Intercompany: N pairs · …`, opens this
tab on `Enter`. Needs-attention items are raised only for a non-zero unexplained residual, a pair not
compared (`^p → Refetch (≈ N calls)`; pairs sharing a cause collapse into one line, `! 4 pairs not
compared: …`, counting each missing TB once) or a pair not supported. The line looks like plain text until
focused; counts not compared/not supported are in warning colour. Fully bridged pairs never raise one, even if
their explaining items still await a decision.

### 5 Activity

Every HTTP call the CLI or TUI made: `time · org · method · endpoint · status · ms · day left`
(time is `HH:MM:SS` for today, `MM-DD HH:MM:SS` otherwise; ids in endpoints shown as `{id}`). Status
colours: 2xx dimmed, 4xx warning, 429 / 5xx / network problems error. The detail panel shows the
selected call's path, params, duration and remaining budgets. Follows the org scope.

### 6 Usage

Today's totals (calls, records fetched, new versions, report snapshots, errors), a quota bar per org
(warning/error thresholds as in the header), the per-minute rate against `limits.minute_soft_limit`,
calls and average latency per endpoint, and syncs per org.

### 7 Organisations

Token state (logged in, days until the refresh token expires, number of scopes and how many are
write-capable — never used by the TUI) and one row per org: alias, Xero name, connected, plan,
records in the local record, last sync, tenant id (hidden below 100 columns).

### 8 Settings

Every effective setting with its source (`defaults`, `config`, `profile:<name>`), the client id and
org aliases. An empty `tui.as_of` is shown as `(default: last FY end → <date>)`. `e` edits the file.

### 9 Links

All configured links expanded for your orgs and bank accounts (developer portal, OAuth callback,
"Open <org> in Xero", reconcile and statement pages per bank account, …). `o`/`Enter` opens, `c`
copies. Per-document links (`per: "document"`) are used by Ledger L3, not listed here.

### 6 Classify (v0.11.0)

Local classification of Xero lines: project, bearer, treatment and should-be account, with notes. Labels are
local intent; they never change Xero (V18). The tab writes only the classification store (`db/classify.sqlite`,
append-only; V14), every write attributed (`actor = human` from keys) and undoable (V15). A sub-bar names the
modes: `Queue N · Groups · Audition · Tree · Catalog` (`,` / `.` move; the palette has `Classify: Queue` …).
Only the Queue is built in this release; the other modes say so. While the tab is open the header shows
`classify: N open · M stale` (dropped at degrade step 4).

**Queue.** Lines with an unconfirmed required label (project, bearer, treatment) or a stale label: stale
lines first (they need a look before anything else), then the largest amount first (`Sort::StaleFirst`, as
the CLI's `classify queue`). At 100 columns the list takes 67 cells (`org · date · payee · amount · suggestion · conf`) and the
detail pane 32; from 140 the list adds the account, the full date and a confidence bar (`▰▰▱ .84`); below 100
the detail stacks under the list (labels first, then one facts line). The date shows `MM-DD` only when every
listed line is in one year. The suggestion is the project's name plus `+n` other suggested labels (`Harvest
Blen… +2`, `no project +2`, `· none`); the detail pane shows each label with its source (R rule, J Jev, H human,
A agent) and confidence, the rule that fired, groups, notes, and the project keys `p1`…`p9`. A label whose
line changed since it was set shows `stale` with the old and new values.

| Key | Action |
|---|---|
| `Enter` / `a` | Accept every suggested label on the line, then move to the next |
| `p` then `1`–`9` | Set the project (the detail pane's order); the tab keys stay global |
| `b` / `t` / `s` | Bearer / treatment / should-be account picker (type to filter; `s` lists the line's org chart) |
| `n` | Note (one line; `⌃E` multi-line, where `Enter` breaks the line and `⌃S` saves) |
| `x` | Skip (no write) |
| `u` | Undo the last write on the line (right after an accept: that accept, and the cursor goes back) |
| `A` | Bulk-accept preview: every line in the filter whose suggestions are all ≥ the threshold (`+` / `-`, default 0.90), counts by project and bearer, total; Cancel is focused |
| `/` | Filter: `org:roast acct:60 month:2025-09 project:harvest conf<0.7 status:open|all|stale by:agent since:today`, bare words search payee and description |

`R` (make a rule) is listed in help as coming. Jev is palette-only (`Classify: Ask Jev…`) and refused while
`classify.toml` has `[jev] enabled = false` (V16). Agents may `tab classify`, `filter` and `select` here; agent
writes come later.

### Explainer panels (walkthroughs)

A panel that explains one thing with the books' own numbers, drawn over the current tab's content area. The
numbers come from the same view, ledgers, balance basis and intercompany engine as the tabs (a shown manifest
applies); nothing is typed in. Every panel starts with a one-line plain-English lead under its title.

| Kind | Shows | Opened by |
|---|---|---|
| `bridge` | An intercompany gap as a waterfall: the difference between the two sides' books, each reconciling item taking its part away (the opening difference reads "Carried in from before <year>"), then what is left "Unexplained" (`✓ 0.00` when fully explained). Labels left, amounts right | palette "Explain: bridge <pair>" |
| `flow` | One box per org (in its accent) with a lifeline under it, and one arrow per configured pair: `owed` (who owes whom at the as-of date, the sentence on the arrow, the other side's view under it) or `paid` (what moved over the period, each direction) | palette "Explain: who owes whom", "Explain: what moved…" |
| `t_account` | The account as a T: debits left, credits right, the opening on the side it sits, the largest postings per side (the rest as "N more"), the totals and the closing balance | `T` on a Ledger account (trial balance or postings), palette |
| `timeline` | Month-end balances as bars (debit balances right of the line, credit left), each month's movement, `◆ journal` where a manual journal landed | `M` on a Ledger account, palette |
| `before_after` | What a manifest changes in one org: the trial-balance lines that move and the balance-sheet and P&L totals, before → after with the change | palette "Explain: what <manifest> changes in <org>" (with a manifest shown) |
| `checklist` | The close worklist from a JSON file (`deadlines`, `questions` by owner, `agent_items`, `postings` by state): Deadlines with days left (`filed: true` with `filed_on` reads `filed <date>`; past and not filed reads `due N days ago`), "Waiting on <owner>" per owner, Agent work, Postings per state. Status words are shown verbatim, never dropped | palette "Explain: the close worklist" (config `tui.checklist`) |

`Esc` closes the panel; `↑`/`↓`, PgUp/PgDn, Home/End scroll it; changing tab or scope closes it. Box drawing in
the colour themes, pure ASCII in the plain theme (letters with accents fold to plain ones (é → e, ß → ss), anything else shows as `?`); below the width a
drawing needs, the panel is a table. Each part registers an anchor `viz:<id>` (`viz:start`, `viz:item/2`,
`viz:residual`, `viz:node/<org>`, `viz:edge/<pair>`, `viz:opening`, `viz:debit/1`, `viz:credit/more`,
`viz:closing`, `viz:month/2025-04`, `viz:line/<code>`, `viz:total/assets`, `viz:group/deadlines`,
`viz:item/<id>`), a part scrolled out of sight is recorded as off screen, and the panel itself is
`panel:viz`. The open panel is part of the canonical state (V21). `ownpurse viz <kind> …` prints the same
model as text or `--json`.

### Help overlay (`?`)

Keys first, then the Markdown from config `instructions`. `Esc`, `?` or `q` closes it.

---

## Network actions (command palette)

Never bound to a single key. `ctrl+p`, choose, then confirm.

| Palette entry (exact) | What it does | Calls |
|---|---|---|
| `Sync <alias>…` (one per org) | Incremental sync of that org | Estimated from the record (`Syncer.estimate`): 1 per settings table, 1 per transaction table when an incremental run is due, otherwise pages × records |
| `Sync all orgs…` | Same for every org | Sum of the above |
| `Log in to Xero…` | Browser consent; **read access only** (requests the read scopes plus any already granted, never adds write scopes) | Token exchange + connections |
| `Refetch Xero trial balance for parity…` | Fetch Xero's TB at the as-of date (and at the prior FY end in year mode) for the scoped orgs that have a record | 1–2 per org |
| `Fetch history…` | Only while a document is open (L3) | 1 |
| `Fetch attachments…` | Only while a document is open (L3) | 1 |

**Confirm bar** (both buttons neutral; the highlight follows focus, which starts on **Cancel**):
`Sync beta from Xero? ≈ 15 calls · 978 left today for beta.   (y to confirm)` — `y` or the action
button confirms; `Enter` on Cancel or `Esc` cancels.

**Refusals** stay visible in the same bar (no action button; the other button reads **Close**) until
`Esc` or Close:
- budget: `Not enough calls left today for beta (needs ≈ 40, 22 left). Resets 00:00 UTC.`
- offline: `Offline mode: network actions are disabled. Restart without --offline to use them.`

After a confirmed action completes, a notification reports the calls used and every view reloads from
the record.

---

## Agent bridge (v0.10, view only)

> **Agents work only through `ui state` and `ui send`.** The state is complete and canonical (the round-trip
> law V21: a fresh TUI built from it draws the same frame, cell for cell), so an agent never needs a
> screenshot or the rendered text to understand the view. `ui screen` exists to show a human, or to
> cross-check in tests, never to read the view.

An agent (Claude Code, a script) can read exactly what you see and move your view: tab, scope, as-of,
filter, selection, drill, up, history back/forward, plus a caption in the footer. You see every change live.
The running TUI is the server; there is no daemon.

- **Sessions:** `ownpurse ui ls` lists `id, label, pid, scope, tab, theme, agent` per running TUI (the
  registry is updated when the scope, tab, label or theme changes). `--session` takes an id, a pid, a label
  or `scope:<alias|group:x|all>`; an ambiguous match exits 2 and lists the matches. Set a label with
  `ownpurse tui --label <text>`, `"tui": {"label": "…"}` or the verb `label <text>`.
- **History:** Alt+← / Alt+→ (and the verbs `back` / `forward`) move through the views you visited, like a
  browser: every change of tab, scope, as-of, basis, drill or filter is an entry (cursor moves are not), up
  to 100 each way; a new navigation after going back drops the forward entries. Esc (`up`) stays "up one
  level". One agent send is one entry. Returning to a filtered trial balance (by Esc, `up` or `back`)
  restores its filter and the row you drilled from.
- **Captions:** `caption "<text>"` (or `ui send --caption "<text>" <verbs…>`, in the same update and
  atomically) puts `◆ <text>` in the footer's left segment, replacing the automatic notice; ` · ` back`
  follows when the send moved the view. It lasts as long as the view: selection moves, scrolling and help
  keep it; a change of tab, scope, drill, filter or as-of, Esc, `caption clear` or a newer caption clears it
  (state: `caption: null`). At most 120 characters, one line; control characters (C0, C1, DEL), every escape
  sequence (CSI, OSC, DCS), zero-width and direction-override characters are **rejected** (`bad_caption`,
  exit 2), never stripped, so the agent knows what it sent wasn't shown. Narrow footers drop middle ` · `
  segments first (`first · … · last`), then shorten the first.
- **Other processes' writes:** a CLI `sync` from another terminal, a `classify suggest` or a
  `classify set --direct` change the record or the classification store under a running TUI. The OS file
  events on their folder wake it (no timer; the minute tick re-checks in case an event was dropped); when
  a revision on disk moved it refreshes through the normal reload, keeps the view and says what arrived in
  the strip: `record updated · 3 new versions`, `12 new suggestions`, `labels updated · 5 events`. The
  state's `record.chain_seq` and `classify.store_rev` follow (`ui wait --until record>N` / `store_rev>N`).
  The TUI's own writes, and events that change nothing, draw nothing (V13). While this TUI's own sync
  runs, the record waits for the reload at its end.
- **The project config, live:** saving `ownpurse.json` while the TUI runs re-reads it (the same file
  events on its folder; no timer). A valid change applies with the view kept: orgs, groups, accents and
  intensities, the account map, intercompany settings and `tui.*`; the strip says what changed
  (`config reloaded · orgs −lab · groups`). Scopes resolve by alias and group name, so a scope that no longer
  exists falls back to `all`, and the note says so (`· lab is gone, showing all`). An invalid file keeps the
  last valid config live behind an error band (`ownpurse.json: … · the previous config stays live; fix the
  file and save`), cleared by the next valid save. A live upgrade re-reads it first, so it goes ahead on the
  new config.
- **Update notice:** when a newer `ownpurse-tui` is installed while this one runs, the footer keeps the
  running version (faint) and appends the update (accent): `v0.10.3  ⬆ v0.10.4 ready · ⌃P Upgrade` at 100+
  columns, `v0.10.3  ⬆ v0.10.4 · ⌃P Upgrade` at 80, `v0.10.3 ⬆ 0.10.4` at 60 (ASCII `^ v0.10.4 ready - ^P
  Upgrade`; state `update_available {version, action: "upgrade_in_place"}`). The check is one stat of this
  binary's file at start-up, on the existing minute tick and on keys, resizes and agent requests, and
  `--version` only when its modification time changed: no timer of its own, no network, and nothing is drawn
  unless the footer changes. (v0.10.2 showed `installed · quit and restart to use it`, action `restart`.)
  An "available in the repo, run ./install.sh" state isn't offered: the TUI has no reliable, local, cheap way
  to find a checkout.
- **In-place upgrade (v0.10.3):** the palette's `Upgrade to v0.10.4` (only while an upgrade is ready), or
  `ownpurse ui upgrade [--session …]` from an agent or a shell, with no prompt, replaces the running TUI with
  the installed build in the same terminal and puts you back exactly where you were, a half-typed filter,
  an open palette or picker, a drill at any level and the history included. In order:
  1. While a sync or login runs, it waits: `deferred` (the footer says `Upgrade will run when sync roast
     finishes`, or `… when login finishes`) and applies by itself when the run ends, after its reload.
  2. The view state and the live frame go to a temp file (0600, in the bridge's 0700 socket directory).
  3. Pre-flight: the new build runs headless with `--check-state FILE`. Two strict checks, both run by the
     NEW build (so an older running TUI gets the newer rule):
     - **State equivalence** (primary): it imports the state (migrating an older schema), exports it again,
       and every field of the handed-over state must still be there with the same value (tab, scope, drill
       path, filter, selection, half-typed input…). A newer build may only add fields, with their defaults.
     - **The frame**: rendered at your size, it must equal the live frame row for row, with one cross-version
       allowance, the same rule the frozen-state test applies: the tab bar may append new tabs after the old
       ones (`1 Overview … 5 System` → `… 5 System  6 Classify`). Anything else that differs fails it, with
       the row.
     - **Across a render epoch** (v0.12.0): each build declares a render epoch, bumped when the visual design
       changes (v0.12.0 is epoch 1, v0.15.0 epoch 2; older builds are epoch 0, derived from their version). Across epochs the
       old frame can't match, so the frame check is replaced by the new build's own round trip: export →
       import → render equals its render at 60, 80, 100 and 140 columns, with no panic. State equivalence
       still applies. The answer carries `render_epoch: changed (visual redesign)` with
       `render_epoch_from` and `render_epoch_to`, and the footer says
       `updated v0.11.1 → v0.12.0 · new look · state kept`.

     Data that moved is never a reason to fail: the old build first picks up what other processes wrote
     (the normal reload: record, classification store), so the state carries the current revisions, and if
     the record or the store moves again between the export and the check, the new build reloads and
     re-verifies the view-state equivalence on the current data (the frame, which then shows newer data, is
     not compared). A config that no longer fits the view (`config_changed`, e.g. a removed org) still fails
     it, with the reason, and so does a schema it can't migrate. (The drift refusals `record_changed` and
     `classify_changed` are for file-based imports: `--state`, `--from-state`.) Then nothing happens: the old build carries on unchanged, the file is
     deleted, the whole reason goes to Activity, and the footer says what to do:

     | Reason | Footer |
     |---|---|
     | `config_changed` | `Upgrade not started: config changed · fix config, then retry` |
     | `bad_state` (a schema it can't read) | `Upgrade not started: state not readable · quit and restart to update` |
     | the view state would be lost | `Upgrade not started: view state would be lost · quit and restart to update` |
     | the frame would differ (beyond appended tabs) | `Upgrade not started: view would change · quit and restart to update` |
     | the new build didn't run, timed out or doesn't know `--check-state` | `Upgrade not started: new build failed its check · reinstall, then retry` |

     When it doesn't fit, the optional key hints drop first, then the `⌃P palette` hint (the update notice
     already says `⌃P Upgrade`), then the reason (`Upgrade not started · r reload, then retry`), then `q`;
     only then is the remedy cut. Below 80 columns the remedy drops "then" (`r reload, retry`, `fix config,
     retry`, `reinstall, retry`). The version and the update notice never drop.
  4. The terminal is released only as far as exec needs: synchronized output, mouse reporting, bracketed
     paste and the keyboard-enhancement flags are popped and raw mode is left, but the **alternate screen is
     kept** (no flash of your shell; the new build draws over it, without a clear). The agent socket closes
     and its registry entry goes; the binary is exec'd in place (same pid) with `--resume FILE`.
  5. The new build reads and deletes the file, registers the socket again (same pid, id and start time:
     `ui ls` shows one session, now on the new version) and shows `updated v0.10.3 → v0.10.4 · state kept`
     (the old version comes from the state file). An agent that asked
     gets its answer from the new build on the same connection: `upgraded {from, to}`.
  6. If exec fails, the old build takes the terminal back, reopens its socket and says `Upgrade failed ·
     still v0.10.3 · <why>` (`exec_failed {reason}`).

  `ui upgrade` answers `upgraded`, `deferred`, `up_to_date` (exit 0), or `preflight_failed {reason}` /
  `exec_failed {reason}` (exit 1). Keys you type during the handover stay in the terminal for whichever
  build reads next.
- **Rendering a saved state:** `ownpurse-tui --state FILE [--snapshot WxH] [--format text|ansi|html|svg|png]
  [--out FILE]` and `ownpurse ui screen --from-state FILE` render a saved `ui state` from the state and the
  record alone (no terminal, no socket, no live session). If the record has changed since the state was
  saved (its `record.chain_seq`), they refuse: `error: record_changed`, exit 4, naming both versions.
  `--allow-record-drift` renders anyway, marked: a `≠ record changed` chip in the header (ASCII
  `!= record changed`; `≠ record`, then `≠`, in narrow headers) and `drift {from, to}` in the state.
  Scopes are saved by org alias and group name, not position, and resolved against the current config. The
  state also carries a fingerprint of the config parts that affect the frame (`canonical.config`: org aliases
  with tenant ids, groups, the account map; never secrets, never paths). A pure reorder of orgs or groups
  resolves (same orgs, same frame wherever the view doesn't list orgs in config order). Anything else (an org
  removed, renamed, added or pointing at another tenant; groups or the account map changed) is refused:
  `error: config_changed`, exit 4, naming what differs (`orgs: roast,hold,design,lab → roast,hold,lab`).
  `--allow-record-drift` does not cover it; `--allow-config-drift` renders anyway, marked: a
  `≠ config changed` chip and `drift.config` in the state. Aliases still configured resolve; a scope whose
  org or group is gone falls back to `all`, and a Compare left with fewer than two orgs closes. A state that
  was rendered under drift stays drifted and needs the same flag again.
  The classification store is checked the same way: a state saved before a label, note or group was written
  is refused `classify_changed` (exit 4, `The classification store changed since this state was saved: rev A →
  B.`), never drawn with other rows. `--allow-classify-drift` (also on `ui screen --from-state`) renders it
  anyway, marked: a `≠ labels changed` chip and `drift.classify` in the state. The record and config flags
  don't cover it.

- **Transport:** a Unix socket, mode 0600 in a 0700 directory: `$XDG_RUNTIME_DIR/ownpurse/tui-<pid>.sock`,
  else `~/.local/state/ownpurse/tui-<pid>.sock` (`/tmp/ownpurse-<uid>/` when that path is too long for a
  socket). Local only, never TCP. Running TUIs are listed in `~/.local/state/ownpurse/sessions.json`
  (`$XDG_STATE_HOME` if set); an entry goes away when the TUI exits, and readers prune dead ones.
- **Off switch:** `ownpurse tui --no-agent`, or `"tui": {"agent": false}` in the config. Either way no socket
  is opened.
- **View only:** the verbs below are the whole allow-list. Sync, login, refetch, confirm, edits, quit and
  config writes are refused by the protocol (exit 2), not just hidden. The ONE non-view action an agent may
  send is the in-place upgrade, as its own request (`kind: "upgrade"`, `op: "upgrade"`; `ownpurse ui
  upgrade`); `upgrade` as a view verb is refused (`forbidden`, exit 2). State never contains token values or
  the token file, and lists only the rows on screen.

| Command | What it does |
|---|---|
| `ownpurse ui ls` | Running TUIs (pid, profile, version, started, cwd, socket) |
| `ownpurse ui state [--session PID] [--json] [--fields a,b.c] [--all-rows\|--rows A..B]` | `--fields` returns only those paths (dotted, comma-separated). The view as data: `rev`, tab and level, scope, as-of, basis, filter, breadcrumb, the selected row and each visible row as an anchor (`org`, `doc_type`, `doc_id`, `line_id` or `account_code`) with its key fields, the strip (matches Xero, record age, calls left with `calls_left_estimated`), size, overlay or band |
| `ownpurse ui screen [--format text\|ansi\|html\|svg\|png] [--out FILE]` | The frame you see now, with its `rev`. PNG needs `--out` (Hack, embedded; system fonts for characters it lacks) |
| `ownpurse ui send <verb …> [--expect-rev N] [--expect PRED]… [--screen FORMAT] [--state] [--json-out]` | Change the view. The text answer is a summary: `changed`, `rev`, `breadcrumb`, `filter "<q>" n of N` (when set), `selected` (the row's label) and its `anchor`, `caption` and `notice` (when set). `--state` prints the whole new state instead; `--json` / `--json-out` give the whole answer as JSON (`rev`, `changed`, `summary`, `effects`, `state`) |
| `ownpurse ui wait --until idle\|rev>N\|store_rev>N\|record>N\|running=none [--timeout S]` | Block until the condition holds (event-driven, no polling; default 30 s, at most 600). Exit 0 with `held: true` and the values; exit 4 `timeout` with the values at the deadline |
| `ownpurse ui upgrade [--session PID]` | Upgrade the TUI in place to the newer installed build (see In-place upgrade) |
| `ownpurse ui flash [--session S] [--times 3] [--on 200ms] [--off 200ms] [--token T=V… \| --accent F]` | Pulse a session's header to get the user's attention: the CLI pushes overlay states (default: the header band at 3× its tint, capped at 0.30), then restores the overlay it read, always, Ctrl-C included. At most three flashes a second: `--on` + `--off` < 334 ms or `--times` > 5 is `too_fast` (exit 2). The TUI never animates |

Verbs: `tab <1-6|name>` (`tab classify`), `scope <alias|group:x|all>`, `asof <YYYY-MM-DD>`, `filter "<text>"`, `clear-filter`,
`select <#n|anchor JSON|text>`, `drill`, `up` (one level, as Esc), `back` / `forward` (navigation history), `caption "<text>"` / `caption clear`, `label <text>`, `basis year|history`, `differs on|off`,
`compare <org,org…>`, `reload` (re-reads the record; zero calls), `open` (the document on screen, Ledger or
Intercompany level 3, in Xero's web app through the org switch, via `tui.open_command`; a link, zero calls;
`not_applicable` anywhere else). `ui send --json '{"verb": "tab", "args": ["2"]}'`
sends the same verbs as a message. Each verb runs through the same `update` as your keys.

**Cosmetic verbs (v0.12.0)**, never refused for `user_busy` (they move no focus and touch no input):
`theme <name>` (a built-in or user theme, this session only), `accent <family|#hex>` (overrides the scope
accent: `theme.overlay.tokens.accent`), `overlay set <token>=<value>…` (semantic, component or `knob.<name>`
tokens; values `#hex`, `@palette`, `$token`, `mix(…)` or a number for knobs) and `overlay clear [<token>…]`
(last in a send: it takes the rest of the words). An unknown token or a malformed value is `bad_token`
(exit 2) with the valid names; a value that breaks 4.5:1 (3:1 for `text.4`) on any surface is applied and
warned on stderr (`warning: contrast: …`, exit 0). Each change increments `theme.revision`, shows in
Activity, round-trips through the state, and the palette's "Clear theme overlay" removes it.
`viz <spec-json>` opens an explainer panel (`{"kind": "bridge", "pair": "hold-design"}`; the kinds and their
parameters are `ownpurse viz --help`'s, the same shape as a walkthrough step's `host.viz`) and `viz close` closes
it; also cosmetic. A malformed spec is `usage` and one naming a pair, org or account that doesn't exist is
`not_found` (both exit 2, nothing changes).

`back` / `forward` with nothing to go to are never silent: exit 0, `changed: []`, `note: "no_history"` and
`message: "nothing to go back to"` (or forward), the footer says `Nothing to go back to`, and `rev` doesn't
move (a one-line note alone is not a view change; Alt+← / Alt+→ behave the same).

In text output the rows are a plain table: the anchor's keys as `anchor.<key>` columns, then one column per
field; no JSON inside a cell.

**Filter (`/` and `filter`, one meaning):** case-insensitive substring, spaces and punctuation literal, no
fuzzy matching: `filter "Kiln Room -"` lists `Kiln Room - Afterburner` but not `Kiln - Operating` or
`Kiln Room Capital`. What it searches: Ledger trial balance, account code and name; postings, date, source,
type, reference, contact, description; Overview, alias and Xero name; Intercompany, pair id, label and both
orgs; Activity, time, org, method, path, status, problem; System, the row's cells; Compare, code and label.
`"quoted text"` matches the phrase with its quotes removed (only differs for leading or trailing spaces).
`code:2410` and `code:1450..1480` match account codes (exact, or an inclusive range) on the trial balance and
in Compare. The strip shows it with its count, `filter: "Kiln Room -" · 9 of 43` (state: `filter`,
`filter_matches`, `filter_total`); the trial balance's totals still read **Total (all accounts)**. Where a
filter means nothing (a Ledger or Intercompany document, the Intercompany bridge, Compare postings, the
Ledger without one org) `/` shows a note and `filter` / `clear-filter` are refused with exit 2
`not_applicable`; so are `basis` (Ledger trial balance, postings and Compare only), `differs` (trial balance
only) and `scope` while Compare is open. Drilling from a filtered trial balance into postings clears the
filter (it chose the account).

**State rows:** `rows[]` lists only the rows on screen. Each has `row` (its 1-based place in the whole,
filtered list: the `#n` of `select`), `selected`, `anchor`, and `fields`: one key per column the screen's
table has, the header in snake_case (`parity (year)` → `parity_year`, `TB fetched` → `tb_fetched`,
`calls left` → `calls_left`, `side B (party)` → `side_b_party`, `day left` → `day_left`; the trial
balance's `!` → `flag` and its wide `local − xero` → `difference`), valued as the screen shows it, plus a
few raw extras (`balance`, `differs`, `amount`, `name`, `calls_left_estimated`). Compare rows have `code`,
`account` and `<org>_debit`, `<org>_credit`, `<org>_net`. Postings rows carry `debit` and `credit` as
shown, plus `amount` (signed: debit positive, credit negative) and `balance_value` (the running balance,
signed the same way; `balance` is the shown text). Every amount has a machine twin (`_value`; see
**Display vs value** under the field reference). `view: {first_visible, last_visible, total_rows}` says
which part of the list is on screen, and `truncated: true` means `rows` holds fewer than `total_rows`.
`ownpurse ui state --all-rows` returns the whole list and `--rows A..B` a slice (1-based, the same
numbers as `#n`); both are read-only: the view doesn't move, `rev` doesn't change, nothing is drawn or
logged.

A verb reports `changed` only when the frame and the state show it (tested on every screen); otherwise it
answers `changed: []` and draws nothing.

**Your input always wins.** Every change you make moves `rev`. A send with `--expect-rev` (alias
`--expect-seq`) from before your change is refused, exit 4:

```
error: view_changed
message: The user changed the view since you read it (rev 41 → 43). Nothing was changed.
help[1]:
  Run `ownpurse ui state` again and resend if it still makes sense.
```

While you are typing in a prompt, in the palette, picker or help, at a confirm band or in `$EDITOR`, a send
is refused with `error: user_busy` (exit 4). Nothing is queued. While the terminal is below 60×20 (the screen
shows only the size warning) view-changing sends and classify writes are refused with `error: too_small`
(exit 4; `actual`/`expected` give the sizes): the user couldn't see the change announced. Reads, screenshots,
`ui wait`, cosmetic verbs and `ui upgrade` still answer.

**Intent guards (`--expect`).** `--expect PATH=VALUE`, `PATH!=VALUE` or `PATH~TEXT` (case-insensitive
substring), repeatable, state what the agent believes the view is; paths are dotted into `ui state`
(`tab`, `level`, `scope`, `selected.account_code`, `filter`, `view.total_rows`). Every predicate is checked
before any verb runs; if one fails nothing is done: `error: view_mismatch`, exit 4, with `expected` (the
predicates) and `actual` (`{path, value, text}` for each). In text mode they read `expected: level = 2` /
`actual: level is 1`.

**Structured replies.** A successful send answers `summary` (the view after it: `rev, tab, level, scope,
breadcrumb, filter, selected, view, caption`, plus `store_rev` on Classify) and `effects`, one per verb in
order: `{verb, changed, what}` (`what` in words, as the status strip says it); a verb that changed nothing
says `changed: false` (`back`/`forward` with no history also answer `note: no_history`). A classify write routed through the TUI answers
`effects` per op with `skipped` (human-confirmed lines it left alone) and `view` (the summary).

**What you see:**
- the header shows `◇ agent` (dim) while an agent is connected (until 5 minutes without a request) and
  `◆ agent` (accent) after it changed your view or took a screenshot, until your next key; at 80 columns
  only the glyph; `[agent]` / `[agent*]` without colour;
- the status strip says what changed, e.g. `◆ agent: opened Ledger · scope roast · only accounts that differ
  · ` back`, until your next key (Esc only dismisses it; any other key dismisses it and does its job);
- `` ` `` (backtick) returns to the view before the agent's change (one step);
- a refused stale send shows `◇ agent: change skipped, you moved first` (after you close an overlay);
- Activity lists one `agent` row per read, screenshot and send (in memory; not in the record).

## Walkthroughs and overlays (v0.14.0)

**One-command demo.** `ownpurse demo` launches the real TUI on a fresh private mock database and
runs a ten-step walkthrough automatically through the view-only CLI bridge. `--step-seconds N`
changes the pace (1–60, default 8). The script stops when you change the view, and stays on the final
step when complete. `q` exits and removes the temporary database and private session files.
No live config, login, approval, apply or network is used; both binaries must be installed together.

Guided, replayable explanations over the books: hints, rings and spotlights pointing at what is on screen,
a narration panel in plain English, and walkthroughs you can play, scrub and jump through. Read-only: a
walkthrough step uses view verbs only and can never approve, apply or write.

**Overlays.** Every frame records where it drew the things an overlay can point at, as anchors:
`row:<key>` (a table row by its stable key: `row:code=6140`, `row:entry=2`, `row:pair=…`, `row:log_id=12`;
`#n` names the n-th visible row); a row anchor is the row's text (first to last glyph); `row:<k>=<a>..<b>` names the rows from one to another as one rect (`spotlight row:code=121..131`), `cell:<row>/<column>`, `diff:<json path>` (a field or line of the
generic diff view: `diff:/LineItems/n1`), `chip:<name>` (header chips `scope`, `asof`, `writes`,
`manifest`, …; tabs `tab-ledger`, …; the manifest `stage` strip), `panel:<name>` (`apply`, `issues`,
`narration`, `via`, `diff-title`, …) and `viz:<id>` (visualisation parts). `ui state` lists them under
`anchors`. Placement is caretline-layers'; drawing is ours, in the theme (box-drawing in colour themes,
pure ASCII in the plain one). Boxes never cover the protected regions (the writes badge, the stage strip,
CHECK issues, the apply command and its hashes, warnings, bands) and keep off the text a step explains when
anything else fits.

**Agent verbs** (cosmetic: never refused while the person types; send them last in a list, they take the
rest of it): `hint <anchor> <text> [title=T] [avoid=A,B] [no-arrow] [no-ring] [spotlight]`, `highlight <anchor>` (a
ring), `spotlight <anchor>…`, `clear-hints [id|all]` (agents' layers only), `narrate <title> <body>` or
`narrate clear`. An agent's hint is attributed (`◆ name · Title`). The reply's `layers` says where each
landed: `on_screen` (with `rect` and `box`), `off_above` / `off_below` (an edge chip points the way), or
`not_found` with a `reason`.

**Walkthroughs** are files in `walkthroughs.dir` (default `walkthroughs/` beside ownpurse.json), in
caretline-tour's format: `{id, version, title, kind, meta, step[]}`, each step `{id, narration {title, text},
layers[] (or the one-layer shorthand anchor/kind/data/place), host}`. `host` is ours: `view` (`{"verbs":
[["scope", "roast"], ["tab", "2"], ["select", "6140"]]}` and/or `{"state": {view keys}}`), `checks`
(`amount(org, account, date) = value`, `count(org, account[, from, to]) = n`: a failing check shows a "the
books changed" band, never silently wrong numbers), `why` (lay mode's "why this matters") and `viz`. `meta`
holds `author`, `created`, `record`, `audience` (`lay` turns lay mode on). Narration text uses `**bold**`
and `[term]` glossary links.

**Playing.** `W` (or the palette's "Walkthroughs…") lists them; `ownpurse-tui --walkthrough <id>` starts
one; agents send `walkthrough play <id>`, `step <n>`, `next`, `prev`, `stop`. Keys while playing: `→` `n`
Space next, `←` `p` back, `[` `]` scrub, `g` then a number jumps, `t` the contents (Enter jumps), `Home`
`End`, `r` re-applies the step's view, `?` the first term's definition, `PgDn` / `PgUp` scroll a long
narration (it is never cut), `Esc` leaves and restores the view from before. The progress strip is
clickable. Each step's view is applied over a neutral view, so playing, jumping and reloading give the same
frame. `L` toggles lay mode (names before codes, why-lines).

**Recording.** `walkthrough record start <id> [title=T]`, then for each page the view, `narrate` and the
hints, then `walkthrough record step`; `walkthrough record stop` writes `<id>.json`.

**Glossary.** `[term]` links open plain-English definitions (2–4 sentences with "in your books" examples)
from the shipped glossary plus the project's `glossary.toml` beside ownpurse.json (its entries add to or
replace the shipped ones).

**CLI.** `ownpurse walkthrough new <id> --title T [--audience lay|informed]`, `add-step <id> --title
--text [--why] [--anchor A --hint H --hint-title T --arrow --ring --spotlight] [--view "tab 2"]… [--check
…]… [--viz JSON]`, `show <id>`, `list`, `validate <id>` (caretline-tour's lint, every step laid out at
140×40, 100×30 and 80×24: anchors resolve, no strips, no box over a protected region, checks hold, glossary
links exist) and `render <id> --out DIR [--format txt,svg,png,html] [--sizes 100x30,…]`.

## Switching scope

`g` opens the picker over the current tab with focus in the filter. Sections, in order (empty ones are
omitted): **Pinned** (`tui.pinned_orgs`), **Recent** (the last `tui.recent_scopes` scopes, excluding pinned),
**Groups** (`groups` in config, with member aliases), **All orgs**, then `all`. Each org row shows alias,
Xero name, short code, record age (warning colour when stale, error colour when never synced) and calls
left; group rows show the oldest record age and lowest calls left among members. The current scope is
marked `● current`. Typing ranks the scopes in one list (headings drop): exact alias, alias prefix,
short-code prefix, a word prefix in the Xero name, then fuzzy; the cursor starts on the top hit and matched
letters are bold in the accent colour.
`Enter` switches and closes; `Esc` closes without switching. The palette has the same entries as
`Switch to <alias>` / `Switch to group:<name>`, with no confirm.

Switching never calls Xero (tested). Only the header and the visible tab redraw; other scope-dependent
tabs redraw when you open them. Trial balances and account ledgers are cached per org, as-of date and
basis until `r` reloads the record.

A group scope works like `all` restricted to its members: Overview, Parity, Activity and Usage follow it;
the Ledger asks you to pick one org.

## State while navigating

Each scope keeps its own Ledger level, account, cursors and basis (`h`), Parity mode, Intercompany level,
pair, cursors and sort, and table filters, so
switching back lands where you left it. The first visit to a scope starts at Ledger L1.

| Kept across tab switches | Reset |
|---|---|
| Scope, as-of date, and the per-scope state above | Switching scope re-derives the as-of date unless an override (`--as-of`, `tui.as_of`, `d`) is set |

---

## Configuration keys used by the TUI

All live in `ownpurse.json` (top level or under `profiles.<name>`); see `ownpurse config`.

| Key | Default | Effect |
|---|---|---|
| `tui.as_of` | `""` | As-of override (below `--as-of`) |
| `tui.refresh_seconds` | `5` | How often the header, Activity and Usage re-read the record |
| `tui.log_rows` | `500` | Rows shown in Activity |
| `tui.editor` | `""` | Editor for `e` (else `$VISUAL` / `$EDITOR` / `vi`) |
| `tui.open_command` | `""` | Runs "Open in Xero" links (`o`, the agent verb `open`); else `open` (macOS) / `xdg-open` |
| `tui.narrow_columns` | `100` | Below this width: compact header, narrow column sets, key-only global footer |
| `tui.stack_columns` | `80` | Below this width the document panes stack |
| `tui.pinned_orgs` | `[]` | Aliases or `group:<name>` scopes listed first in the picker |
| `tui.recent_scopes` | `3` | How many recent scopes the picker lists |
| `tui.compare_default_align` | `code` | Compare alignment on open: `code` or `map` |
| `tui.keys_file` | `""` | Your key bindings elsewhere than `~/.config/ownpurse/keys.toml` (relative to this config's folder) |
| `tui.theme` | `auto` | `auto`, `ledger-dark`, `ledger-light` or a user theme in `themes/<name>.toml` |
| `tui.accents` | `{}` | Org alias (or `cross`) → accent family |
| `tui.accent_intensity` | see above | The accent tints, each up to its computed ceiling |
| `tui.view` | see above | The focused view: details, header exceptions, focus line, class bands, density |
| `account_map.categories` | `{}` | Categories for `align: map` (per org: codes or `{"name": …}`; or `default: type:BANK` / `class:REVENUE`) |
| `parties`, `intercompany.pairs` | `{}`, `[]` | Intercompany pairs (tab 4 and `ownpurse ic`); see docs/ROADMAP.md §2 |
| `groups` | `{}` | Named scopes, e.g. `{"ops": {"orgs": ["roast", "design", "lab"]}}`; members are aliases, tenant ids or names |
| `limits.calls_per_day` | `1000` | Daily limit used for quota display and budget checks |
| `limits.warn_day_remaining` | `100` | Warning threshold for calls left |
| `limits.minute_soft_limit` | `55` | Per-minute line in Usage |
| `limits.refresh_token_days` / `refresh_token_warn_days` | `60` / `45` | Token expiry display and attention item |
| `sync.ttl_seconds.default` | `86400` | Record age after which the header shows `stale` |
| `links`, `extra_links` | see defaults | Links tab and per-document links (`per: app|org|bank_account|document`, `source`, `verified`) |
| `instructions` | see defaults | Help overlay text |

---

## Troubleshooting

| Symptom | Cause / fix |
|---|---|
| `No ownpurse config found (looked for ownpurse.json in <folder> and its parent folders, then ~/.config/ownpurse/config.json)` | You ran it outside your project (e.g. from your home folder). `cd` into the folder that contains `ownpurse.json`, or `export OWNPURSE_CONFIG=/path/to/ownpurse.json`. New setup: create `ownpurse.json` from `ownpurse.example.json`. |
| Header says `no local record` | Never synced: `ctrl+p → Sync <org>…` (or `ownpurse sync <org>`). |
| Header record time in warning colour, `stale` | Older than `sync.ttl_seconds.default`: sync if you need newer data. |
| Ledger strip says `not compared` | No cached Xero TB at the as-of date: `ctrl+p → Refetch Xero trial balance for parity…` (1–2 calls) or `ownpurse parity <org> --date <as-of> --since <prior FY end>`. |
| Ledger shows `Local full-history balances` without pressing `h` | No cached Xero TB at the prior FY end, so the year basis is unavailable: refetch as above. |
| Accounts marked `!` in full history | Expected where Xero holds setup/conversion balances the API does not expose; use the default year basis for balances. |
| `Not enough calls left today …` | Daily budget nearly used; the window resets at 00:00 UTC. Everything else keeps working offline. |
| `Not logged in` | `ctrl+p → Log in to Xero…` (opens the browser; read access only). |
| "Open in Xero" lands on the wrong page | Document link templates are shipped `verified: false`; open the org in Xero first, or fix the template under `links` in config. |
| Version in the header looks old | `ownpurse --version`; reinstall/upgrade the package (editable installs: reinstall to refresh metadata). |
| Colours unreadable | Switch Textual theme with `ctrl+p → Change theme`; the TUI uses theme tokens only. |

## State field reference (`ownpurse ui state`)

<!-- state-fields -->
Generated from and tested against `state::ViewState` (a field missing here fails the build). **Derived**
fields are computed from the model on each export and never imported; **reserved** fields are fixed
shapes for later releases. Agents read these fields; `canonical` is the importable block that restores the
exact view (`ownpurse-tui --state`, `ownpurse ui screen --from-state`).

| Field | Meaning |
|---|---|
| `v` | State schema version (1; v0.10.2 and v0.10.3 both write 1). Each release reads the previous releases' states (`tests/fixtures/state/v<release>/`, a gate rule), migrating older schemas step by step; a schema it can't migrate is refused (exit 2), never reset. |
| `rev` | View revision: +1 on every change of the view (a key, a click, an agent send, a reload). `--expect-rev` guards on it. |
| `version` | The running build. |
| `build` | Its build identity: the git commit it was built from (`g6dc723f`, `.dirty` with uncommitted changes), or null. `--version` and `ui ls` show `0.12.1+g6dc723f`; the footer shows it with `i` (details). A different build of the same version counts as an update (`ui upgrade`); releases compare by version. |
| `update_available` | Null, or a newer installed build: `update_available.version`, `update_available.action` (`upgrade_in_place` from v0.10.3; `restart` in v0.10.2). |
| `label` | This session's label (`--label`, `tui.label`, agent `label`). |
| `clock` | Derived. `clock.now`: the clock the frame shows (UTC ISO); `clock.model_now`: the clock the record was read at. |
| `record` | The record version the rows came from: `record.chain_seq` (the integrity chain's last sequence number), `record.derivation` (the row-derivation revision of the build) and `record.classify_rev` (the classification store's revision; null without one). |
| `drift` | Null, or `drift.from` / `drift.to` / `drift.config`: rendered from a state exported against record version `from` while the record is at `to` (`--allow-record-drift`; the header shows `≠ record changed`), and/or under another config, `drift.config` naming what differs, e.g. `orgs: a,b,c → a,c` (`--allow-config-drift`; the header shows `≠ config changed`). `drift.config` is null when only the record drifted; `from` equals `to` when only the config drifted. `drift.classify` is `[from, to]` when the classification store moved since the state was saved (`--allow-classify-drift`; the header shows `≠ labels changed`), else null. `drift.manifest` is `[from, to]` when the shown manifest moved since the state was saved (`--allow-manifest-drift`; the header shows `≠ manifest changed`), else null. |
| `profile` | Config profile. |
| `tab` | overview, ledger, intercompany, activity, system, classify. |
| `level` | Drill level on the tab (1–3). |
| `mode` | What the level shows: orgs, trial balance, postings, document, compare, compare postings, pairs, bridge, log, by endpoint, settings; on Classify the mode (queue, groups, audition, tree, catalog). |
| `scope` | all, an alias, or `group:<name>`. |
| `as_of` | As-of date. |
| `basis` | year or history. |
| `filter` | The `/` filter text (raw). |
| `filter_applies` | Derived. Whether a filter means something on this screen (else `filter` is refused). |
| `filter_matches` | Derived. Rows the filter lets through (null without a filter). |
| `filter_total` | Derived. Rows before the filter (null where a filter doesn't apply). |
| `differs_only` | The trial balance's differs filter. |
| `sort` | Intercompany's sort (null elsewhere). |
| `breadcrumb` | Derived. The breadcrumb, as shown. |
| `compare` | Null, or the open Compare: `orgs`, `align`, `level`. |
| `selected` | Derived. The selected row's anchor (or the open document's). |
| `selected_label` | Derived. The selected row's key label, as `select` names it (`1060 Tallowmere Green-Bean Card`); null with no row selected. |
| `rows` | Derived. The rows on screen (or all / a slice with `--all-rows` / `--rows A..B`): `rows[].row` (1-based place in the whole filtered list, the `#n` of `select`), `rows[].selected`, `rows[].anchor`, `rows[].fields` (one key per column, the header in snake_case, valued as shown, plus raw extras). |
| `view` | Derived. `view.first_visible`, `view.last_visible`, `view.total_rows`. |
| `truncated` | Derived. `rows` holds fewer than `view.total_rows`. |
| `strip` | Derived. `strip.matches_xero`, `strip.record_age`, `strip.record_level`, `strip.calls_left`, `strip.calls_left_estimated`, `strip.calls_limit`, `strip.calls_level`. |
| `size` | `size.cols` × `size.rows` of the terminal. |
| `theme` | `theme.name` (the built-in or user theme in use), `theme.glyphs` (unicode / ascii), `theme.color` (color / nocolor), `theme.revision` (increments on each applied theme change: an overlay set or clear, a theme switch, a reload of the theme file in use; null before any), `theme.overlay` with `theme.overlay.tokens` (the agent's cosmetic overlay, token → value; `ui send overlay …`), `theme.knobs` (the effective accent intensities, `tui.accent_intensity` with any overlay `knob.*`: `theme.knobs.header_band`, `theme.knobs.section_band`, `theme.knobs.breadcrumb`, `theme.knobs.selection`, `theme.knobs.compare_columns`, `theme.knobs.status_warn`, `theme.knobs.status_err`, `theme.knobs.proposed_rows` (the proposed-row tint) as decimals, `theme.knobs.header_gradient` true/false, `theme.knobs.pair_band` neutral/split). |
| `accent` | Derived from the scope, never stored: `accent.scope_kind` (org, pair, compare, group, all), `accent.token` (the accent family the scope resolves to: the org's own, or the cross accent, default harbor; an overlay `accent` wins), `accent.members` (the orgs in scope, each `accent.members[].alias` with its own `accent.members[].accent` family). |
| `keymap` | The effective keymap (v0.12.0): `keymap.revision`, the content hash of the keys.toml it was built from (null for the built-in table); `keymap.overrides`, the keys keys.toml adds or removes. |
| `overlay` | help, palette, picker, or null. |
| `prompt` | Null, or the open editor: `prompt.kind` (filter, as_of, range), `prompt.text`. |
| `band` | Null, or the confirm band: `band.kind` (confirm / refusal), `band.text`, `band.buttons`, `band.focus`. |
| `input` | Null, or the text being typed: `input.field` (filter, as_of, range, palette, picker, classify_picker, note), `input.text`, `input.caret` (characters from the start). |
| `palette` | Null, or the open palette: `palette.query`, `palette.highlighted`. |
| `note` | The footer's transient note. |
| `notice` | The agent's notice, as shown. |
| `caption` | Null, or the agent's caption: `caption.text`, `caption.by`, `caption.rev` (the revision it was set at). |
| `result` | Null, or the last run's result band: level and lines. |
| `history` | Navigation history: `history.back`, `history.forward` (entry counts; bounded at 100 each way), `history.back_to`, `history.forward_to` (where the next step lands). |
| `running` | Null, or the running sync/login: `running.label`, `running.progress`. |
| `busy` | What the user is in when a send would be refused `user_busy`. |
| `data_refreshes` | How many times this session refreshed for other processes' writes (record, classification store, `ownpurse.json`). A burst of writes (a sync in another terminal) is coalesced: at most two refreshes per burst. |
| `manifest` | Derived; null on main. The shown manifest (spec manifests §5): `manifest.name`, `manifest.stage` (draft, review, check, planned, approved, applying, applied), `manifest.chip` (the header's short form: `7 changes · 1 issue`, `check ✗ 2`, `planned`, `applying 3/7`, `stopped ✗`, `applied ✓`), `manifest.head_rev` (the store revision: an agent's CLI write sends it as `--expect-rev`), `manifest.entries` (active entries), `manifest.issues.errors`, `manifest.issues.warnings`, `manifest.issues.acknowledged`, `manifest.apply.sent`, `manifest.apply.landed`, `manifest.apply.verified`, `manifest.apply.rejected`, `manifest.apply.uncertain`, `manifest.package_hash` (planned onward), `manifest.approve_command` and `manifest.apply_command` (exactly as `ownpurse manifest plan` prints them; there is no apply or approve verb: the owner runs them) |
| `viz` | Derived; absent with no panel open. The open explainer panel: `viz.kind`, `viz.spec` (as sent), `viz.title`, `viz.parts` (every part's id in drawing order: the `viz:<id>` anchors), `viz.scroll`, `viz.error` (why it could not be computed) |
| `classify` | Derived; null when classification isn't set up (present on every tab, so agents can read `classify.store_rev`). `classify.mode` (queue, groups, audition, tree, catalog), `classify.filter` (the Queue filter, raw), `classify.filter_error` (why it doesn't parse; the Queue then lists the unfiltered scope), `classify.selected_anchor` (`org:doc_type:doc_id:line_id`), `classify.audition_set` (reserved), `classify.open` and `classify.stale` (lines across the store), `classify.editor` (bearer, treatment, should_be_account, note, bulk, project, or null), `classify.bulk` (the open preview: `classify.bulk.threshold`, `classify.bulk.lines`, `classify.bulk.total`, `classify.bulk.total_value`), `classify.jev_enabled` (V16; false in v0.11.0), `classify.store_rev` (the classification store's revision: an agent write sends it as `expect_store_rev`), `classify.agent_writes` (agent writes this session not yet undone). Queue rows: `org`, `date`, `payee`, `amount` + `amount_value`, `account`, `suggestion`, `conf` + `conf_value` (a number, null without a suggestion), and the raw `project`, `bearer`, `treatment`, `source` (R, J, H, A, ·), `description`, `stale`, `open`; the anchor carries `anchor`, `org`, `doc_type`, `doc_id`, `line_id`. |
| `layers` | Overlay layers on this frame (hints, rings, spotlights; walkthroughs design): each with `layer` (its id), `owner` (`guide` for a walkthrough, `agent:<name>`), `anchor` (`row:code=6140`, `chip:stage`, `diff:/LineItems/n1`, …) and `landed`: `on_screen` (with `rect` and `box`), `off_above`, `off_below`, or `not_found` with a `reason`; `strip` when only a one-row strip fitted, `no_arrow` when an arrow could not be drawn |
| `anchors` | Every overlay anchor drawn on this frame, in draw order: what `hint`, `highlight` and `spotlight` can point at here |
| `narration` | The narration panel when shown (an agent's `narrate`, or the walkthrough step): `title`, `body`, `by` (the agent; null for a walkthrough step), `why` (lay mode's "why this matters") |
| `walkthrough` | The walkthrough playing, null otherwise: `id`, `title`, `step` (1-based), `of`, `step_id`, `failed_checks` (the step's `checks` that don't hold now, with today's value) |
| `recording` | A recording in progress (`walkthrough record start`): `id`, `steps` |
| `lay_mode` | Lay mode (`L`): account names lead, codes hide, "why this matters" lines show |
| `canonical` | The importable block (in the export, scopes and orgs are by alias): `canonical.v`; `canonical.record` (`canonical.record.chain_seq`, `canonical.record.derivation`, `canonical.record.classify_rev`: the classification store's revision, so a state taken before a label was written is refused `classify_changed` rather than drawn with other rows); `canonical.config`, the config fingerprint the state was made under, never secrets or paths (`canonical.config.orgs`, ordered `[alias, tenant id]` pairs; `canonical.config.groups`, ordered `[name, members]`; `canonical.config.account_map`, a hash of the account map; `canonical.config.hash`, a hash of all three); `canonical.rev`; `canonical.manifest_rev` (the shown manifest's head revision when the state was taken: a render refuses `manifest_changed` when it moved, unless `--allow-manifest-drift`); `canonical.env` (`canonical.env.now`, `canonical.env.model_now`, `canonical.env.offline`, `canonical.env.version_label`, `canonical.env.profile`, `canonical.env.cli`, `canonical.env.cli_version`, `canonical.env.cli_shown`, `canonical.env.paths`, `canonical.env.label`, `canonical.env.update_available`, `canonical.env.drift`); `canonical.ui` (`canonical.ui.tab`, `canonical.ui.scope`, `canonical.ui.overlay`, `canonical.ui.band`, `canonical.ui.prompt`, `canonical.ui.filter`, `canonical.ui.as_of`, `canonical.ui.overview_sel`, `canonical.ui.overview_top`, `canonical.ui.overview_col`, `canonical.ui.chain`, `canonical.ui.note`, `canonical.ui.theme`, `canonical.ui.theme_overlay` (the agent's cosmetic overlay: token → value), `canonical.ui.theme_revision` (increments on each applied theme change), `canonical.ui.theme_user` (the user theme in use: its name and the file text when applied, so a render from state never reads the file), `canonical.ui.details` (the details layer, `i`), `canonical.ui.size`, `canonical.ui.full_history`, `canonical.ui.differs_only`, `canonical.ui.run`, `canonical.ui.result`, `canonical.ui.ledger`, `canonical.ui.ic`, `canonical.ui.ic_sort`, `canonical.ui.via`, `canonical.ui.cmp`, `canonical.ui.activity`, `canonical.ui.system`, `canonical.ui.caret`, `canonical.ui.history`, `canonical.ui.caption`, `canonical.ui.classify` (mode, filter, selected_anchor, audition_set, top, and the open picker, note, bulk preview, `p` pending and last write), `canonical.ui.manifest` (the manifest shown, null on main), `canonical.ui.manifest_tab` (tab 7: mode, list cursor, the open prompt and the open entry diff with its expanded, patch-view and focus state), `canonical.ui.layers` (caretline-layers' layers: the walkthrough's and agents'), `canonical.ui.narration`, `canonical.ui.walk` (the walkthrough playing: caretline-tour's state with the tour itself, the view from before it, the contents cursor, a typed step number, the failed checks), `canonical.ui.walk_list` (the `W` list), `canonical.ui.recording` (the walkthrough being recorded), `canonical.ui.lay_mode`, `canonical.ui.glossary_focus` (the glossary term whose definition is open)); `canonical.agent` (`canonical.agent.connected`, `canonical.agent.last_seen`, `canonical.agent.acted`, `canonical.agent.notice`, `canonical.agent.undo`, `canonical.agent.skipped_pending`, `canonical.agent.log`, `canonical.agent.writes`: the agent's classify writes this session, each with its Activity row, events, summary and whether it was undone); `canonical.keymap`, the keys.toml the keymap was built from (`canonical.keymap.v`, `canonical.keymap.bindings`, `canonical.keymap.unbind`, `canonical.keymap.hash`, `canonical.keymap.file` (how errors name it: empty for keys.toml, else the `tui.keys_file` setting); empty for the built-in table): a render from state uses it, never the file on disk; `canonical.render_epoch`, the render epoch of the build that wrote it (v0.12.0 is epoch 1; absent in older states, which are epoch 0): an in-place upgrade across epochs checks the state, not the frame. |

### Display vs value

Row fields are valued **as the screen shows them** (`"7,036.41"`, `"(7,036.41)"`, `"roast Cr 1,213.47"`, `""`).
Every one that shows an amount has a machine twin, the same key plus `_value`: an exact JSON number (the
digits as shown, never a float), or `null` where the cell shows no amount. Read the twin; never parse the
display string.

| Screen | Display fields → twins | Sign of the twin |
|---|---|---|
| Ledger trial balance | `debit` → `debit_value`, `credit` → `credit_value` | none: both columns are positive amounts (the column says which side) |
| Trial balance, differs | also `xero` → `xero_value`, `difference` → `difference_value` | signed as shown (parentheses or `−` = negative) |
| Ledger postings | `debit_value`, `credit_value` (unsigned, as above); `balance` → `balance_value` | `balance_value` signed: debit positive, credit negative |
| Compare | `<org>_debit` → `<org>_debit_value`, `<org>_credit` → `<org>_credit_value`, `<org>_net` → `<org>_net_value` | debit and credit unsigned; net signed (`(1,213.47)` → −1213.47) |
| Intercompany pairs | `side_a` → `side_a_value`, `side_b_party` → `side_b_party_value`, `difference` → `difference_value`, `residual` → `residual_value` | sides: `Dr` positive, `Cr` negative; difference and residual as shown |
| Intercompany items | `amount` → `amount_value` | as shown |

Already numeric, as raw extras: the trial balance's `balance` and the postings' `amount` are signed decimal
strings (debit positive, credit negative), each with its own `_value` number. Counts are shown as counts and
have no twin: Overview's `calls_left`, Activity's `day_left`, `calls`, `errors`, `avg_ms`. (v0.10.2's
postings `balance_value` was a decimal string; from v0.10.3 it is the number.)

## Error codes

<!-- generated from core::bridge::UI_ERRORS; a test keeps them equal -->

Every refusal of a `ownpurse ui …` command or an agent's classify write, with `--json`, is
`{error, message, help, expected?, actual?, rev?}` (schema: `docs/schema/ui-error.v1.json`; the state's schema is
`docs/schema/ui-state.v1.json`). `help` always holds at least one line saying what to do next.

| Code | Exit | Meaning | What to do |
|---|---:|---|---|
| `usage` | 2 | The command, a verb or its argument is malformed. | Check the syntax with the command's --help; values (verbs, rows, choices, groups) must exist in the current view or classify.toml. |
| `unknown_verb` | 2 | Not a view verb. | View verbs: tab, scope, asof, filter, clear-filter, select, drill, up, back, forward, basis, differs, compare, reload, open, caption, label, theme, accent, overlay, manifest, mode, viz, hint, highlight, spotlight, clear-hints, narrate, walkthrough. |
| `forbidden` | 2 | The verb names something the bridge never does (sync, login, quit, confirm, edits, keys). | Ask the user to do it in the TUI; an agent sends view verbs only. |
| `not_applicable` | 2 | The verb means nothing on this screen (e.g. a filter on a document). | Move to a screen where it applies (`up`, or another tab) and resend. |
| `not_found` | 2 | The row, anchor, line or group is not in the current list. | Read `ownpurse ui state --all-rows`; clear the filter or change the scope if the row should be there. |
| `bad_caption` | 2 | The caption is empty, longer than 120 characters, or has control, escape or invisible characters. | Send one line of plain text, at most 120 characters. |
| `bad_token` | 2 | A theme, accent or overlay token (or its value) is unknown or invalid. | Use a name the message lists; read the theme's tokens in `ownpurse ui state` (theme). |
| `too_fast` | 2 | `ui flash` beyond the photosensitivity limits (a cycle under 334 ms, or not 1 to 5 flashes). | Use --on + --off of at least 334 ms and --times from 1 to 5. |
| `interrupted` | 130 | Ctrl-C during `ui flash`; the overlay that was there before was restored. | Nothing to undo; run it again if it still makes sense. |
| `too_many` | 2 | A classify write would touch more than 50 lines. | Narrow the filter, or pass --allow-bulk when the user asked for a bulk change. |
| `filter` | 2 | A classify filter token is malformed. | Filter tokens: org:, acct:, month:, year:, <type>:<id>, conf<x, conf>=x, by:, since:, status:open|all|stale. |
| `bad_state` | 2 | The file given to --from-state is not a saved ui state. | Save one with `ownpurse --json ui state > state.json`. |
| `not_available` | 2 | The request kind is reserved and not implemented. | Use `ownpurse ui state|screen|send|wait|upgrade` or the classify write commands. |
| `view_changed` | 4 | The view (or the classify store) moved since the revision you sent; nothing was changed. | Run `ownpurse ui state` again and resend if it still makes sense. |
| `view_mismatch` | 4 | An --expect predicate does not hold on the view; nothing was changed. | Read `ownpurse ui state`, decide again, and resend with --expect for the view you now intend. |
| `user_busy` | 4 | The user is typing, in an overlay, or has that line open in a picker; nothing was changed. | Wait, then read `ownpurse ui state` again; never retry in a loop. |
| `too_small` | 4 | The user's terminal is below the minimum size (60×20) and shows only the size warning; a change could not be announced, so nothing was changed. | Ask the user to enlarge their terminal, then read `ownpurse ui state` and resend. |
| `timeout` | 4 | `ui wait` reached its timeout before the condition held. | Read the values in the reply; wait again with a longer --timeout or a different condition. |
| `record_changed` | 4 | The saved state was taken against another record version. | Pass --allow-record-drift to draw it anyway (marked as drifted), or save a new state. |
| `config_changed` | 4 | The saved state was taken under another config. | Pass --allow-config-drift to draw it anyway (marked as drifted), or save a new state. |
| `classify_changed` | 4 | Labels were written since the state was saved. | Pass --allow-classify-drift to draw it anyway (marked as drifted), or save a new state. |
| `no_session` | 1 | No running TUI with the agent bridge. | Ask the user to start `ownpurse tui` (the bridge is on unless --no-agent). |
| `bridge` | 1 | The TUI's socket did not answer (it may have exited). | Run `ownpurse ui ls`; retry once the TUI runs again. |
| `closed` | 1 | The TUI is closing. | Run `ownpurse ui ls` and talk to a running session. |
| `protocol` | 2 | The CLI and the TUI speak different protocol versions. | Install matching builds (./install.sh) and restart the TUI (or `ownpurse ui upgrade`). |
| `preflight_failed` | 1 | The newer build could not take over the view; the old one carries on unchanged. | Read the message; the user can restart the TUI instead. |
| `exec_failed` | 1 | The newer build could not start; the old one carries on unchanged. | Reinstall (./install.sh) and try `ownpurse ui upgrade` again. |
| `tui_missing` | 1 | The ownpurse-tui binary was not found (needed for --from-state). | Install both binaries with ./install.sh. |
| `render` | 1 | Rendering a saved state failed. | Save a fresh state with `ownpurse --json ui state` and retry. |
| `write` | 1 | The output file could not be written. | Check the --out path and its folder. |
| `png` | 1 | The PNG could not be encoded. | Retry with --format text or svg. |
| `conflict` | 1 | classify.toml changed on disk since it was loaded. | Reload (the TUI does it live; for the CLI just retry) and resend the edit. |
| `config` | 1 | classify.toml is invalid, or a config edit would make it invalid. | Run `ownpurse classify rules check` and fix the file, or adjust the edit. |
| `store` | 1 | The classification store could not be read or written. | Check db/classify.sqlite (permissions, disk) and retry. |
| `integrity` | 1 | The classification store's hash chain does not verify. | Stop and tell the user; do not write until the store is checked. |
| `record` | 1 | The local record could not be read. | Run `ownpurse doctor`. |
| `not_set_up` | 1 | Classification is not set up in this TUI (no classify.toml). | Create classify.toml next to ownpurse.json (see docs/CONFIG.md, classify). |
| `internal` | 1 | A bug in ownpurse. | Report it with the command and `ownpurse ui state`; rerun with OWNPURSE_DEBUG=1. |

Not an error: `back`/`forward` with nothing to go to answers exit 0 with `note: no_history` (and `effects[].changed: false`).
