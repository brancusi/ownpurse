---
title: Changelog
description: Every notable change to ownpurse, release by release.
sidebar:
  order: 3
---

All notable changes are listed here. The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and ownpurse uses [Semantic Versioning](https://semver.org/). While ownpurse is `0.x`, a minor release may change the
CLI surface; each change is called out below.

Releases up to 0.16.0 were built under the project's development name. The commands, files and environment
variables below use the ownpurse names.

## [0.16.0] · Statements from the local ledger, cash basis

Added:

- `pl <org> --from --to [--cash]` and `bs <org> --date [--cash]`: the profit and loss and the balance sheet from
  the local ledger, compared account by account with Xero's report for the same dates and basis when the record
  holds one (`report fetch`); `--strict` exits 1 on any difference. `bs` opens from Xero's balance sheet at the
  prior fiscal year-end (same basis) when cached, else (or with `--full-history`) from local documents only.
- Cash basis in the ledger (`LedgerBasis::Cash`, Xero's `paymentsOnly`): invoices and credit notes post at each
  payment, their lines and tax in proportion to the amount paid; manual journals with
  `ShowOnCashBasisReports: false` are left out.

## [0.15.0] · Reports, history, attachments and export

Changed:

- The TUI render epoch is 2 (overlay placement at the row's text): an in-place upgrade from 0.12–0.14 checks
  the view state, not the frame. Fixed: such an upgrade was refused as "render_epoch is lost or changed".
- Dev and test builds carry line tables only, and dependencies no debug info (`[profile.dev]`): a much
  smaller `rust/target`.

Added (API coverage for the year-end cleanup):

- `report fetch <org> trialbalance|balancesheet|pl|banksummary` into the record (parity reads the latest TB);
  `report list`.
- `history <org> <Entity> <id>` (an id prefix resolves through the record); notes as write op `add_note`; every
  landed write adds `ownpurse package <hash12> op <id>: <why>` to the document's history in Xero (on by default
  for real profiles). Notes and other free text this tool writes go out in ASCII; history shows HTML entities
  decoded.
- `attachments <org> <Entity> <id> [--get <file>]`; attach files to bank transactions as well as journals.
- Write ops for cash with no feed: `create_bank_transaction` (SPEND / RECEIVE; recovery `void_bank_transaction`,
  Status DELETED, proven on the Demo Company), `create_bank_transfer` (recovery: a reverse transfer; permanent
  history), `create_contact` (exact-name guard; recovery `archive_contact`). Manifest rules-table rows and Demo
  Company probes for each.
- `export <org> tb|gl|bs|pl` writes a CSV for the accountant.

Fixed:

- `ui upgrade` and System › Writes fixes from 0.13.0's live use (see 0.13.1 notes): drill on a package lists its
  operations, `select key=value`, the upgrade pre-flight is a full start-up dry run, install.sh compares the
  version only.
- A create is read back by the id Xero answered with (Xero's search index can lag a fresh create); reconcile
  retries the marker search and names the query it sent.

Fixed (from the live review at 214×59 and 318×60):

- **Row anchors aim at the row's text**, not its blank end: a row anchor is its content extent (first to last
  glyph drawn). A hint on a table row goes beside the row's text with the arrow head on the row (`◀`); a row
  layer whose arrow would end on a neighbouring row is placed beside it instead.
- **Range anchors**: `row:code=121..131` names every visible row between two rows as one rect, so `spotlight
  row:code=121..131` lights the whole range (verbs and walkthrough files).
- **Spotlight dimming is clearly visible**: dimmed text goes most of the way to the background colour; in
  ANSI and no-colour themes the lit rows get a gutter mark.
- **The walkthrough progress strip sits right under the narration text** (a tall right-docked panel keeps it by
  the words).
- **Wide terminals**: explainer panels keep a readable width (120 columns, the T-account 100) centred in the
  content area; bars and columns stay close to their labels; the money-flow lifelines stop after the last
  edge; the narration text runs at most 100 columns. Snapshots at 200×50 and 320×80, with a test that no
  panel row has a run of more than 40 blank cells between two pieces of content.
- Hint boxes are never narrower than 18 columns (a side with less room is skipped).

Added (demo and caretline):

- `ownpurse demo`: a single command opens the real TUI on an isolated mock database and automatically
  drives a ten-step walkthrough, with overlays and narration, through the guarded view-only bridge.
  Moving the view takes control; `q` exits and cleans up. No setup, login or live accounting writes.
- Build against an exact local caretline 0.4.0 export pinned in `caretline.rev`; preserve walkthrough arrow-head
  rules when recording layers.

## [0.14.0] · Walkthroughs, overlays and explainer panels

Guided, replayable explanations of the books in the TUI: hints, rings and spotlights pointing at what is on
screen, a narration panel in plain English, walkthroughs to play, scrub and jump through, and explainer panels
computed from the ledger. All read-only: nothing here can approve, apply or write.

**Building needs a local caretline checkout**: `caretline-layers` and `caretline-tour` are path dependencies until
they are published. `install.sh` checks for it and says how to get it.

Added:

- **Overlays** (TUI; placement by `caretline-layers`, drawn in our theme, pure ASCII in the plain one). Each frame
  records anchors for what it drew: `row:<key>` (`row:code=6140`, `row:entry=2`, `row:pair=…`), `cell:`,
  `diff:<json path>` (the generic diff view), `chip:<name>` (header chips, tabs, the manifest stage strip),
  `panel:<name>` and `viz:<id>`; `ui state` lists them. Boxes never cover the protected regions (the writes
  badge, the stage strip, CHECK issues, the apply command and hashes, warnings, bands) and keep off the text a
  step explains when anything else fits: caretline-layers' soft avoid keeps boxes and arrows off a layer's
  `avoid` anchors and the referenced content (the diff, its via band, the selected row) and off text; the
  lint warns (`covers_avoid`) when nothing clear was in reach. Walkthrough layers carry `avoid`; record mode
  adds the selected row and the diff shown.
- **Agent verbs** (cosmetic, never refused while the person types): `hint <anchor> <text> [title=T]
  [avoid=A,B] [no-arrow] [no-ring] [spotlight]`, `highlight`, `spotlight`, `clear-hints`, `narrate <title> <body>` / `narrate clear`.
  Agents' hints are attributed (`◆ name`); the reply says where each layer landed (on screen, off above/below,
  or not found and why).
- **Walkthroughs** (`walkthroughs.dir`, default `walkthroughs/` beside ownpurse.json), in caretline-tour's
  format and run by its reducer: per step a narration, layers, and `host` (view verbs or state, `checks` such as
  `amount(org, account, date) = value` — a failing one shows a "the books changed" band — a lay-mode `why`, a
  `viz`). Play from `W`, the palette, `ownpurse-tui --walkthrough <id>` or `walkthrough play|step|next|prev|stop`;
  keys → ← n p Space [ ] g<n> t Home End r ? PgDn/PgUp Esc (restores the view from before). Playing, jumping and
  reloading give the same frame (the walkthrough state is in the canonical view state, V21). Record with
  `walkthrough record start|step|stop`.
- **Glossary and lay mode.** `[term]` links open plain-English definitions (shipped default plus the project's
  `glossary.toml`); `L` toggles lay mode (names before codes, "why this matters" lines). Narration is never
  cut: the panel grows, then scrolls.
- **CLI** `ownpurse walkthrough new|add-step|show|list|validate|render`: `validate` lays every step out at
  140×40, 100×30 and 80×24 (anchors, strips, protected regions, checks, glossary links); `render` writes each
  step's frame as text, svg, png or html.
- **Explainer panels** (`core::viz`, TUI, CLI `ownpurse viz`): `bridge` (an intercompany gap as a waterfall:

  difference → reconciling items → what is left unexplained), `flow` (who owes whom, or what moved, between the
  orgs), `t_account`, `timeline` (month-end balances, journals marked), `before_after` (a manifest's effect on one
  org) and `checklist` (the close worklist file). Computed from the books through the same view, ledgers, basis
  and intercompany engine as the tabs; exact decimals; each part anchored as `viz:<id>` for walkthrough layers.
  Opened by the palette's "Explain: …" entries, `T` / `M` on a Ledger account, or the cosmetic agent verb
  `viz <spec-json>` / `viz close`; the open panel is in the canonical state (V21). Config `tui.checklist`.

## [0.13.0] · The write path, manifests, and Open in Xero

The first release that can change Xero: four operations only (create a manual journal, recode a bank-transaction
line, void a journal, attach a file to a journal), always as a sealed package the owner approves by its hash.
Nothing is sent without that approval; agents can never approve (except the owner's standing approval on a Demo
Company).

Added:

- **The write path (Phase 8).** `write plan | seal | export | approve-record | approve-import | approve-check |
  approve-demo | run | verify | reconcile | recover | status | probe | probe-replay`. A package is planned from
  the record (requests, live preconditions, expected trial-balance deltas, a pre-compiled recovery), sealed
  (sha256), reviewed as a PDF and a CSV for the accountant, approved by the owner at his terminal or by a chat
  message he typed (imported by hash), then run behind three locks (`allow_writes`, `OWNPURSE_WRITE_RUN=<hash>`,
  the recorded approval) through a guarded transport (endpoint allow-list, tenant guard, Idempotency-Key, a
  `sending` entry logged before every request). The freeze gate holds operations on a frozen entity-year or a
  prior period unless the accountant's decision is named. Run and verify read every target back into the record
  (source `readback`), so the record and the TUI follow a write at once. `recover` builds a recovery package that
  needs its own approval. `login --write` asks for the write scopes (writing profiles only); `doctor` lists the
  granted scopes.
- **Manifests** (`ownpurse manifest …`, TUI tab 7): proposed changes as atomic document edits (RFC 6902 JSON
  Patch over a normalised view), a rules table that maps each edit to a write operation (a reconciled line becomes
  a monthly reclass journal or a browser checklist step; editing a posted journal is an explicit void-recreate),
  CHECK (errors block, warnings are acknowledged), PLAN into the sealed write package, apply status per entry and
  posted detection on read-back. The TUI shows the entries, issues, effects and log, the stage strip, the command
  panel (no apply in the TUI), one generic field/line diff view, proposed values in the Ledger, and the header
  chip. Agent verbs `manifest` and `mode` (approve and apply are refused).
- **Verify outcomes.** `write verify` says `verified`, `rejected`, `uncertain`, `incomplete` or `discrepancy`
  (the trial balance moved differently from the expected deltas), with `ok: false` and the reasons otherwise; the
  record keeps each outcome (new lazy `verifications` table), and System › Writes shows it.
- **Writes in the TUI (read-only):** Activity lists every attempt, System › Writes lists the packages with their
  stage and approval source (Enter lists a package's operations), a journal's document view names the package
  that wrote it, and the external-change notice says what landed.
- **Open in Xero.** Links go through Xero's org switch (`organisationlogin/default.aspx?shortcode=…`), so a
  document opens in the right organisation: `url` in `show`, `txns` and `journals` `--json`; `ownpurse open <org>
  <Entity|BankRec> <id> [--launch]`; the package review (PDF line, CSV `xero_url` column) and `write status` link
  each target (the created journal once landed); the refusal of a reconciled recode links the transaction; the
  TUI's documents and System › Writes operations open with `o`/Enter through `tui.open_command`; the agent verb
  `open`. Local only: building a link never calls Xero. The page paths are data and still marked unverified.
- **TUI:** a build identity (`0.13.0+g<sha>[.dirty]`) in `--version`, `ui ls`, the state and the footer, so a
  different build of the same version is an update; live reload of `ownpurse.json`; `too_small` (exit 4) while
  the terminal shows only the size warning; a refused multi-verb send names the verb that failed.

Changed:

- Precondition fingerprints are taken over a normalised business view, the same for Xero's list and by-id
  shapes; a mismatch shows the JSON Patch between what was reviewed and what Xero holds now.
- Xero's top-level error (`Type`, `ErrorNumber`, `Message`) is surfaced with its validation errors.

Fixed (from the Demo Company live probes):

- Xero refuses any edit of a reconciled bank transaction: such a recode is refused at plan (`reconciled_line`)
  with the alternatives (a reclass journal, or unreconcile in the browser), never sent.
- A created journal is read back by its id (a list GET carries no journal lines in real Xero).
- `{id}` placeholders are resolved across the whole request (path and body); a request still holding one is
  refused before sending.
- A run's trial-balance snapshot is never taken at a sentinel date (9999-12-31); prior reports with implausible
  dates are ignored and `doctor` warns about them.
- No signed zero anywhere in requests, reviews or plans; text output carries no debug dumps.
- The footer no longer panics when a caption and a long update notice share a narrow screen; the stopped stage
  strip keeps its counts on a 100-column screen.
- `ui upgrade` never freezes or loses the TUI: the installed build's `--version` is read off the loop (a fresh
  binary's first launch can stall while macOS checks it), and an identical build is refused ("already running this
  build") with the running TUI untouched.

Live probe findings recorded: a 1000-line manual journal is accepted; the same Idempotency-Key with the same body
returns the same journal, with a changed body a 400; attachments to a journal work.

## [0.12.1] · Agents drive the TUI reliably; classification polish

Added:

- **Agent bridge:** `ui send --expect PATH=VALUE|PATH!=VALUE|PATH~TEXT` intent guards (`view_mismatch`, exit 4,
  expected vs actual); `select` reaches any row of the current list (scrolls to it; `drill` acts on it); structured
  send replies (post-state summary + per-verb `effects`, with skips); `ui state --fields`; versioned JSON schemas
  (`docs/schema/ui-state.v1`, `ui-error.v1`) and one error-code table; `ui wait --until idle|rev>N|record>N|
  store_rev>N|running=none` (event-driven, `timeout` exit 4).
- **The TUI picks up other processes' writes** to the record and the classification store (file events plus a
  minute re-check; no new timer) and says what arrived; the view is kept.
- **Classification:** `status:` filters rows and counts alike; stale lines first (Queue and CLI); rule conditions
  `payee_is_org` and `bank_transfer`; faster open at size.
- **Live acceptance suite** in the gate: the real TUI in a PTY driven only through `ownpurse ui`, checking state
  against the actual screen after every step, plus robustness, concurrency and latency budgets.

Fixed:

- A live `ui upgrade` no longer fails because data changed since the TUI started (it refreshes first; drift guards
  apply to file-based imports only).
- The upgrade pre-flight can no longer hang on a chatty build; tests no longer depend on wall-clock timing.

## [0.12.0] · Redesign, themes, per-org accents, key bindings

Added:

- **User config in `~/.config/ownpurse/`** with one watcher (no timer): `keys.toml` and `themes/*.toml` reload on
  save; a bad save keeps the last valid file and shows `file:line` in the error band.
- **Key bindings** (`keys.toml`): bindings and unbinds layered on the built-in keymap; footer, help (`*` marks your
  keys), palette and `--keys` show the effective keymap. ⌘/super keys via keyboard enhancement; `--echo-keys` shows
  what the terminal sends. Alt+← / Alt+→ stay the history defaults.
- **Themes**: truecolor token engine (primitives → semantic → component), built-ins `ledger-dark` / `ledger-light`,
  user themes that extend a built-in (`@palette`, `$token`, `mix()`, `#hex`), `tui.theme`, `--check-theme`. NO_COLOR
  keeps the plain rendering.
- **Per-scope accents** from `tui.accents` (one per org; harbor for pairs, groups and all, with member dots in the
  scope chip) and 8 intensity knobs with computed contrast ceilings.
- **Cosmetic agent verbs** `theme`, `accent`, `overlay set|clear` and `ownpurse ui flash` (≤ 3/s, always restores).
- **Redesign** as the default: a quiet header (exceptions only), tab band, section bands with hairlines, a focus line
  under the selected row, details on `i`, health at the breadcrumb's right, a ≤ 5-key footer with `? more`, and
  `tui.view.density = "compact"`.
- **Render epoch**: `ui upgrade` across a visual redesign checks state equivalence plus the new build's own round
  trip instead of comparing old and new frames; the reply says `render_epoch: changed (visual redesign)`.

Fixed:

- NO_COLOR: a selected row is one continuous reverse span.
- Shell overlays (palette, picker, help) clear the area behind them.

## [0.11.1] · In-place upgrade across UI changes

Fixed:

- `ui upgrade` refused an upgrade to a build that adds a tab ("the restored frame differs from the live one (row
  2)"). The new build's pre-flight now checks STATE equivalence first (the handed-over view state must survive
  import, migration and export unchanged; new fields may be added, none lost), then the frame with the same
  allowance as the frozen-state test: the tab bar may only gain appended tabs, every other row must be identical.
  Anything else still aborts the upgrade and the old TUI keeps running. A running 0.10.3+ TUI upgrades to this build
  in place.

## [0.11.0] · Classification

Added:

- **Classification layer** (`core::classify`): local labels on Xero lines (project, bearer, treatment, should-be
  account), notes and groups in `db/classify.sqlite` (append-only, SHA-256 chained, every write attributed and
  undoable), with the catalog and rules in `classify.toml` (hand-editable; edits are diffed and written atomically
  with numbered backups; `[queue] scope` limits everything to, e.g., one year). A label remembers the line it was set
  on and shows `stale` when a later sync changes it. Nothing in it is ever sent to Xero.
- **`ownpurse classify`**: `suggest` (rules; `--dry-run`; idempotent), `rules check`, `queue`, `export --csv`, and the
  write commands `set`, `note`, `group create|add|remove|status`, `undo`, `config add-project` (with
  `--expect-store-rev`, `--client`, `--caption`, `--allow-bulk`, `--overwrite`; routed through a running TUI).
- **TUI tab 6 Classify**: Queue (accept, `p`+digit project, bearer/treatment/should-be pickers, notes, skip, undo,
  filter, bulk-accept preview, `R` make-rule, `G` add to group), Catalog (projects and rules with live "would match",
  staged diff and confirm, archive instead of delete), Groups (chain check one-to-one / many-to-one with share /
  running balance with paired settlement legs and a configurable pair rule). `classify.toml` hand edits reload live.
- **Agent writes** through the bridge (`kind: "classify"`): never overwrite a person's label without `overwrite`,
  at most 50 lines without `allow_bulk`, refused while the user has the line open; a notice with `U undo`, the `A`
  source glyph, and Activity undo.
- Saved view states carry the classification store revision (`classify_changed`, `--allow-classify-drift`).

## [0.10.4] · First public release: Rust CLI + TUI

What exists:

- **CLI (`ownpurse`).** `login` (PKCE, read scopes only; one consent covers every org), `orgs` and `sync` are the
  only commands that call Xero. Every other command reads the local record: `accounts`, `org`, `contacts`, `txns`,
  `journals`, `transfers`, `list`, `show`, `gl`, `tb` (with `--compare` for 2–4 orgs side by side), `parity`
  (local trial balance vs Xero's, to the cent), `ic` (intercompany pairs from config), `usage`, `config`,
  `doctor`. Compact text by default, `--json` for machines, structured errors with stable exit codes.
- **The record.** A local SQLite file, append-only (triggers), every row linked into a SHA-256 integrity chain.
  Amounts keep Xero's exact digits.
- **TUI (`ownpurse tui`).** Read-only: Overview, Ledger (trial balance → postings → document; Compare), Intercompany,
  Activity and System tabs; Sync and Log in from the palette after a confirm with a call estimate.
- **Agent bridge (`ownpurse ui …`).** An agent reads the exact view state, moves the view with view-only verbs,
  adds a footer caption, renders the screen as text, HTML, SVG or PNG, and can upgrade a running TUI in place to a
  newer installed build with the view restored. The user's input always wins.
- **Install.** `./install.sh` (macOS and Linux).
