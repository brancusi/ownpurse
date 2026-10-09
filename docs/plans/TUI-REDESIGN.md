# Plan: the TUI in the ownpurse design system

Status: planned, not started. Target release: 0.17.0. Owner: the next session.

The TUI is where ownpurse lives day to day, and it still looks like the tool it was before the rename: slate and
paper themes, five scope accents, `✓ ! ≠` marks and a plain text product label. This plan brings it into the brand
([brand/README.md](../../brand/README.md), [brand/TERMINAL.md](../../brand/TERMINAL.md),
[brand/tokens.json](../../brand/tokens.json)) without breaking what the TUI already guarantees: the V21 round-trip
law, the V22 contrast law, the V23–V25 theme laws, in-place upgrades and the agent bridge.

Read first: [docs/PHILOSOPHY.md](../PHILOSOPHY.md), [brand/README.md](../../brand/README.md),
[brand/TERMINAL.md](../../brand/TERMINAL.md), [docs/TUI.md](../TUI.md), and `AGENTS.md`.

## What the TUI is today (facts from the code)

- **Themes** (`rust/crates/ownpurse-tui/src/theme/`): a three-layer token system. `def.rs` holds palettes
  (named colours) → semantic tokens (`bg`, `surface.top`, `surface.tabs`, `surface.section`, `surface.raised`,
  `surface.foot`, `selection`, `rule`, `text.1`–`text.4`, `ok`, `warn`, `err`, `agent`, `proposed`, `key`) →
  component tokens. Built-ins: `ledger-dark` and `ledger-light` (`BUILTINS` in `def.rs`). User themes `extends`
  a built-in (`user.rs`). Modes: truecolor, xterm-256, ANSI-16, `NO_COLOR` (`theme/mod.rs`).
- **Scope accents**: five per-organisation accents (`harbor`, `ember`, `iris`, `rose`, `sea`) in `def.rs`, mixed into
  bands by **knobs** (`Knobs` in `def.rs`: `header_band`, `header_gradient`, `section_band`, `pair_split`,
  `selection`, …; `tui.accent_intensity`).
- **Glyphs** (`glyphs.rs`, `theme/tokens.rs`): `UNICODE` (`▌ ✓ ! ≠ ⌁ · ›`) and `ASCII` sets, plus a fold table.
- **Header** (`widgets/header.rs`, `layout.rs` `Part`): row 0 is fitted, never wrapped. The product part is the
  text `ownpurse · <profile> · v<version>` (`App::header_facts` in `app.rs`), followed by badges, scope, as-of,
  freshness, quota, agent, drift, classify and manifest chips.
- **Tabs** (`state.rs` `Tab`): Overview, Ledger, Intercompany, Activity, System, Classify, Manifest.
- **Laws and tests**: V21 canonical view state round-trip (`tests/v21.rs`), V22 contrast across every theme ×
  accent × surface (`theme::def::tests`), V23 no colour literal outside `theme/` (`tests/v0120.rs`), V24 snapshot
  coverage (`tests/snapshots.rs`, about 1,800 `.snap` files), V25 cosmetic overlay. Frozen view states for each
  release live in `rust/tests/fixtures/state/v*/` and name the themes they were drawn in (`ledger-dark` appears in
  the v0.16.0 states).
- **In-place upgrade** (`upgrade.rs`): `RENDER_EPOCH = 2`. A change to how a view is drawn from the same state
  bumps the epoch.

## The target

From `brand/TERMINAL.md` and the brand book:

- **Night is the TUI's own theme**: `term-ground` #1B1B19, `term-ink` #E9E6DD, `term-dim` #9A978D, `term-coin`
  #EE5A3F (negatives, differences, the coin), `term-ok` #6FC0AA (agreed, balanced), `term-gilt` #E0B04F (held,
  warnings), `term-note` #8DB6EC (selection, links, notes), `term-line` #3A3934. A **Paper** light theme follows the
  Paper tokens (`surface-0` #F5F3EE, `ink-1` #23231F, `coin-text`, `verdigris`, `gilt-text`, `lapis`).
- **The coin is the only round thing and is spent sparingly.** Status is a coin glyph plus a word: posted and
  balanced `●`, open `○`, held `◌`, short `−`, over `+` (ASCII: `*`, `o`, `~`, `-`, `+`). The word always shows.
- **The mark in the header**: `┳ ●○ ownpurse 0.17 · roast, hold · 2026-09-30` (ASCII `T *o`). The filled eye is
  in `term-coin`. The open eye **blinks** (`○` → `−` → `○`, about every six seconds) while a sync or an agent is
  working, and is still otherwise. It reads `−` when the books in view are short, `+` when they are over.
- **Figures**: right-aligned, two decimals, thousands separators, negatives in parentheses in `term-coin`; a total
  sits under `─` with `═` beneath it.
- **Rules, not bands.** Sections open with a rule. Header gradients and tinted bands are decoration the brand
  avoids; scope colour belongs to small marks (a scope chip, a column header), not washes.
- **Voice**: lowercase `ownpurse` everywhere; books, accounts, entries, sources; every list states its count; a
  next-step hint in `term-note`; errors say what failed and the command that fixes it.

## Decisions to make at the start of the session

1. **Default theme.** Recommendation: Night is the default; Paper is chosen with `--theme ownpurse-paper` or by
   following the terminal's background where it can be detected.
2. **Names and compatibility.** Recommendation: the new built-ins are `ownpurse-night` and `ownpurse-paper`;
   `ledger-dark` and `ledger-light` stay as aliases that resolve to them, so frozen states, user configs and user
   themes that `extends` them keep loading. The frozen v0.10.x–v0.16.0 states are never edited.
3. **Scope accents.** Five organisation colours are still needed (pair views, compare columns). The coin is never
   one of them. Recommendation: build the set from the brand's pigments and two quiet additions, all tested by V22:
   lapis, verdigris, gilt, a muted plum and a slate blue. Decide in the spike.
4. **Knobs.** Recommendation: keep the knob machinery (users and the V25 overlay depend on it) but set the Night
   and Paper defaults to rules-not-bands: `header_gradient` off, low `header_band` and `section_band`.
5. **Motion.** Recommendation: the blink is on by default and off with `[tui] motion = "off"`, with `--no-motion`,
   in `NO_COLOR` mode and in snapshot and agent-drive runs (deterministic frames).

## Phases

Each phase lands as its own commits with snapshots reviewed; nothing waits on CI (see `AGENTS.md`).

### 0. Spike (half a day)

Write the Night palette as a **user theme file** extending `ledger-dark` (no code changes), render the main screens
with `rust/scripts/tui-snapshots.sh` (txt and html), and look at them beside the design system's Terminal component.
Settle decisions 3 and 4 here. Output: a screenshot set and the chosen accent values.

### 1. One source for the colours

- Add the Night surface steps the TUI needs (`surface.top`, `surface.tabs`, `surface.section`, `surface.raised`,
  `selection`) to `brand/tokens.json` as `term-*` tokens, so the brand stays the source of truth.
- Add a test in `ownpurse-tui` that reads `brand/tokens.json` and asserts the built-in Night and Paper values match
  it, so the TUI can't drift from the brand silently.

### 2. The themes

- `def.rs`: add `ownpurse-night` and `ownpurse-paper` (palette → semantic mapping below), make them the
  `BUILTINS`, map `ledger-dark` and `ledger-light` to them as aliases, and set the new accents and knob defaults.
- ANSI-16 fallbacks per `brand/TERMINAL.md` (agreed is cyan, not green, so it stays apart from red).
- Pass V22 for every theme × accent × surface; fix values, not the law.

| Semantic token | Night | Paper |
|---|---|---|
| `bg` | `term-ground` #1B1B19 | `surface-0` #F5F3EE |
| `text.1` | `term-ink` #E9E6DD | `ink-1` #23231F |
| `text.2` / `text.3` / `text.4` | between `term-ink` and `term-dim`, chosen in the spike | `ink-2` #4D4C45 / `ink-3` #6B695F / a lighter step |
| `rule` | `term-line` #3A3934 | `line-1` #DEDAD0 (`line-2` #8F8C82 where it must carry meaning) |
| `ok` | `term-ok` #6FC0AA | `verdigris` #2A6F5E |
| `warn` | `term-gilt` #E0B04F | `gilt-text` #7F5C17 |
| `err` | `term-coin` #EE5A3F | `coin-text` #B2341F |
| `agent` | `term-note` #8DB6EC | `lapis` #2C5FA8 |
| `proposed` | `term-gilt` (held, not posted) | `gilt-text` |
| `selection` | a raised step with a `term-note` marker | `lapis-tint` #DCE6F4 with a `lapis` marker |

### 3. Status glyphs and figures

- Replace the status marks with the coin glyphs and their words (`●` posted/balanced, `○` open, `◌` held, `−` short,
  `+` over), with ASCII forms in `glyphs.rs` and the fold table. Keep `≠` only where it means "differs from Xero" and
  pair it with a word.
- Negatives in parentheses in `err`, totals under `─` and `═`, everywhere amounts are drawn (ledger, compare,
  intercompany, viz panels).

### 4. The header mark and its eye

- `layout.rs` `Part::Product`: draw `┳ ●○ ownpurse` (ASCII `T *o`) with the filled eye in the coin colour, then
  the version and scope. The profile name moves into the scope part or the System tab if the row is too tight;
  protected items still never truncate (V5).
- The eye's state comes from the model: blinking while a sync runs or the agent bridge is acting (`agent.rs`,
  `net.rs`), `−` when the view's books are short, `+` when over. The blink is a timer-driven cosmetic frame: it must
  not enter the canonical state (V21) and must be off in snapshots and agent-drive.
- Bump `RENDER_EPOCH` to 3 if the same state now draws differently, and add the epoch-3 case to the upgrade tests.

### 5. Copy pass

- Lowercase `ownpurse` in every string; "Xero" only where the text is about the Xero connector; books, accounts,
  entries, sources; counts first; hints in `agent`/note colour.
- Update the demo walkthrough narration (`walk.rs`, `rust/tests/fixtures/walkthroughs/`) to the new voice.

### 6. Tests, docs and release

- Regenerate snapshots (`INSTA_UPDATE=always cargo test -p ownpurse-tui --test snapshots`) and review every
  changed `.snap`. The V24 coverage list gains the new themes.
- Update `docs/TUI.md`, the site's Terminal UI guide (`site/src/content/docs/docs/guides/terminal-ui.md`),
  `brand/TERMINAL.md` if values moved, and the CHANGELOG `[Unreleased]` section.
- Run the full gate once (`OWNPURSE_FULL_GATE=1 git push` or `rust/scripts/check.sh`), then
  `rust/scripts/release.sh 0.17.0`. Redeploy the site (`cd site && npm run deploy`).

## Done when

- `ownpurse tui` opens in Night with the mark in the header; the open eye blinks during a sync and stops after.
- Every status shows a coin glyph and a word; every negative is in parentheses in the coin colour.
- `--theme ownpurse-paper`, `--theme ansi`, `--theme no-color` and `--glyphs ascii` all render without loss of
  meaning.
- Old configs and user themes naming `ledger-dark` / `ledger-light` still load; v0.16.0 states still upgrade in place.
- V21–V25, the snapshot suite and agent-drive pass; `ownpurse demo` plays through in the new look.

## Out of scope for this release

A splash screen with the wordmark, new tabs or layout changes beyond the header, the daemon, new connectors, and the
move to a fact-based record. These follow in the roadmap ([docs/ROADMAP.md](../ROADMAP.md)).
