# Architecture

## Principles

1. **Solid test coverage**: every behaviour has a test; goldens to the cent; invariants as tests.
2. **Performance at the core**: budgets checked by benches; one frame per interaction; idle writes nothing.
3. **Simplicity**: few crates, few dependencies, each justified; plain data structs; one Elm-style update loop;
   no async runtime.

## Crates

Everything lives in the Cargo workspace under `rust/`.

| Crate | Contents | Depends on |
|---|---|---|
| `ownpurse-core` | the record (read-only SQLite access), exact-decimal JSON decoding, chart, ledger builder, bases (Xero opening / full history), parity, intercompany engine, compare, config (layered, with origins), the agent-bridge registry, PNG rasterising, classification (`classify`: the label store, `classify.toml`, rules, the queue model), manifests (`manifest`: the entry store, normalisation, the rules table, CHECK, PLAN into the write package), the write package side (`write`), the explainer models (`viz`: bridge, flow, T-account, timeline, before/after, checklist) | rusqlite, serde, serde_json, rust_decimal, sha2, toml_edit, json-patch, caretline-tour |
| `ownpurse-xero` | OAuth PKCE, the token store (cross-process lock), HTTP client, rate limits, the call log, sync, the record's write side | core, ureq |
| `ownpurse-cli` (binary `ownpurse`) | the CLI: read commands over the record; `sync`, `login`, `orgs`; the agent-bridge client (`ui …`) | core, api, clap |
| `ownpurse-tui` (binary `ownpurse-tui`) | the ratatui terminal UI; runs the installed CLI as a child for anything that calls Xero | core (never api: V1, compile-time), notify, toml_edit, sha2, similar, caretline-layers, caretline-tour |
| `xtask` | repository checks and generators for the gate (hygiene, docs grep, docs inputs) | aho-corasick, regex |

A future write path (Xero writes, through an operations log) will be a separate crate; the TUI cannot depend on it
(V1, compile-time).

## Dependencies (each one justified)

| Crate | Why |
|---|---|
| `rusqlite` (bundled) | The record is SQLite; bundled = identical SQLite everywhere, no system lib drift |
| `serde`, `serde_json` (`arbitrary_precision`) | Config and Xero payloads; arbitrary precision keeps every JSON number's digits so money never passes through `f64` |
| `rust_decimal` | Exact money (V11); `Amount(Decimal)` newtype with no `From<f64>` |
| `ratatui` + `crossterm` | The TUI; crossterm gives synchronized output, mouse, kitty keyboard |
| `unicode-width` + `unicode-segmentation` | Grapheme-correct measuring and truncation |
| `clap` (derive) | CLI parsing |
| `ureq` (rustls) | Blocking HTTP: the API layer is sequential by design (rate limits 60/min, 5 concurrent, 1,000/day), so no async runtime. One thread per network action, results as `Msg`s |
| `sha2` | Content hashes and the record's integrity chain; the classification store's chain and line fingerprints |
| `toml_edit` (core `classify`; TUI user config) | One spanned TOML parser for every hand-edited file: `classify.toml`, and (v0.12.0) `~/.config/ownpurse/keys.toml` and `themes/*.toml`, whose errors all name file:line. `classify.toml` is edited by hand AND by the TUI and agents: a format-preserving parser/writer keeps hand-written comments and layout through every write, and its spans give `file:line` in every validation error. `serde`+a plain TOML crate would drop comments on write; JSON (like `ownpurse.json`) has no comments |
| `json-patch` (core, CLI) | Manifests: every proposed change is ONE document before → after, stored as an RFC 6902 JSON Patch against the normalised document. The crate is the standard diff (`json_patch::diff`) and apply (`json_patch::patch`) of that RFC: serialisable, applicable, and readable as keyed paths (`/JournalLines/n2/AccountCode`) once line arrays are keyed. Small (jsonptr, serde, thiserror), no `unsafe` of its own; writing a patch format and its diff by hand would be the same thing with more bugs |
| `similar` (TUI) | The generic diff view's word highlights inside changed text fields (narration, description, reference): word-level `TextDiff`. Already in the tree through `insta`, so no new code is downloaded |
| `caretline-layers` (TUI) | Overlays: where hint boxes, arrows, rings, spotlights and edge chips go over the screen, given the anchors we record while drawing; pure (serde only, no I/O, no clock), the host draws. Ours (the caretline project); a path dependency on its checkout until it is published |
| `caretline-tour` (core, TUI) | Walkthroughs: the file format with its strict parser and lint, the step reducer (state as data in the canonical view state, so replay, jump and scrub are deterministic) and `plan_steps` for validation at several sizes; pure (serde, toml). Same project and terms as caretline-layers |
| `regex` | Intercompany party rules from config (case-insensitive searches); the repository checks |
| `fontdue` + `png` (core `raster`) | `ownpurse ui screen --format png` and `ownpurse-tui --state … --format png`: the TUI's cell grid is rasterised and encoded as a PNG. fontdue (pure Rust, small) rasterises single glyphs, which is all a cell grid needs; `png` (pure Rust) encodes. An SVG renderer would pull an SVG parser, text shaping and a raster stack to draw what is already a grid. Font: Hack Regular (MIT + Bitstream Vera licence) embedded from `crates/ownpurse-core/assets/`; characters Hack lacks fall back to system fonts and render blank when none is installed |
| `notify` (TUI, `macos_fsevent` only) | The Classify tab reloads `classify.toml`, and (v0.12.0) one watcher on `~/.config/ownpurse/` reloads `keys.toml` and `themes/*.toml`, the moment they are saved. The OS file-event API (FSEvents, inotify) wakes the loop through its channel, so there is no polling timer and an idle TUI still writes nothing (V13). It watches the config's folder (an editor's atomic save replaces the file) |
| `libc` | `flock` (token file, session registry), `kill(pid, 0)` (pruning dead TUI sessions), terminal and signal handling |
| dev: `insta`, `proptest`, `criterion` | Snapshots, property tests, benches |
| dev: `cargo-llvm-cov` (tool) | Coverage |

No tokio, no chrono-tz (dates are ISO strings and day arithmetic in a small module), no ORM.

## Tests and gates

| Kind | What | Target |
|---|---|---|
| Unit | Every module; `update(App, Msg)` is pure and unit-tested | — |
| Goldens | JSON and text **to the cent** for `tb`, `gl`, `parity`, `ic`, `compare` and every CLI command, against frozen goldens on committed fixtures; optionally on a private real record (local only: `OWNPURSE_GOLDEN_RECORD`, `OWNPURSE_GOLDEN_REAL`) | exact |
| Frozen reference artifacts | `rust/tests/fixtures/frozen/`: records, a call log and a two-round sync the writer, the HTTP client and the syncer must keep reproducing byte for byte | exact |
| Property (proptest) | Decimal formatting round-trips; every document's postings sum to 0 or are flagged; TB debits = credits; opening + activity = closing; fiscal-year arithmetic | — |
| Snapshot | `TestBackend` + `insta` for every screen and state at 60/80/100/140 × dark/light/ansi/NO_COLOR | every screen |
| Invariant | V1–V13 (no write path in the TUI, zero calls on navigation, palette-only network, header never truncates, residual never hidden, amounts never truncate, compare rules, Decimal only, never colour alone, idle 0 bytes); V21, the round-trip law: export → import → export of the canonical view state is byte-identical, and a fresh app built from it plus the same record draws the live frame cell for cell, after every step of a seeded random walk over keys, clicks, resizes, ticks and agent verbs (`tests/v21.rs`); V22 contrast: every built-in theme × accent × surface keeps text ≥ 4.5:1 (`text.4` ≥ 3:1) at the knob defaults and at the computed ceilings (`theme::def` tests); V23 derived theme and accent state: the accent is one derivation from the scope, read by renderers only through the state, no colour literal outside `theme/` (`tests/v0120.rs`); V24 snapshot coverage: the accent, knob, view and overlay states render in every theme (`tests/snapshots.rs`); V25 cosmetic overlay: `theme` / `accent` / `overlay set|clear` set, clear and round-trip, `bad_token` exits 2, never refused while the user types (`tests/v0120.rs`) | all |
| State versions | Every release reads the previous releases' frozen view states (`rust/tests/fixtures/state/v*/`) and freezes its own | all |
| Concurrency | Six token refresher processes race on one file for 50 rounds: exactly one refresh per round | 50/50 |
| Coverage | `cargo llvm-cov` | ≥ 85 % lines |
| Benches | criterion: cold start → first frame ≤ 200 ms; tab/drill ≤ 8 ms; org switch ≤ 16 ms; compare ≤ 30 ms | fail at 2× |

`rust/scripts/check.sh` runs all of it, plus repository hygiene and the docs grep (see
CONTRIBUTING.md). Every tag is gated on it.

## Safety rules

- **OAuth**: PKCE, no client secret. One refresher at a time: re-read under an exclusive `flock`, rotate, write
  atomically with mode 0600. Login never requests a write scope.
- **Read-only invariants**: no write path in the TUI, zero calls on navigation, network actions only from the
  palette after a confirm with a call estimate.
- **The record is append-only**: history tables refuse UPDATE and DELETE (triggers), and a SHA-256 chain links
  every appended row (`Writer::verify` proves it; a `verify` command is coming).
- **SQLite initialisation**: `ownpurse_core::record::sqlite_ready()` runs SQLite's first-time initialisation once,
  on one thread, before any connection opens (racing it is unsafe on arm64).

## Behaviour notes

Each is deliberate and tested.

| Area | Behaviour |
|---|---|
| Unaliased tenants | Only configured aliases resolve; the token file is never read for names. Unaliased tenants are labelled by their first 8 characters |
| Missing record | Error: "The local record does not exist yet." Read commands never create a file |
| Argument errors | clap wording; the structured `usage` code and exit 2 |
| Party regexes (intercompany) | The `regex` crate, case-insensitive; unsupported syntax (lookarounds, backreferences) is a config error, never silently skipped |
| Network commands | `sync`, `login`, `orgs` (one record, token file and lock; `login --write` adds the write scopes only for an `allow_writes` profile; `--offline` makes them refuse). Report fetch, `history`, `attachments`, `export`, `init`, `q`, `verify`, `skills`, `status`: `not_available`, exit 2, "coming" |
| Estimate for a never-synced org | `core::estimate::Estimate::FirstSyncFloor`: 1 call per table plus extra passes (15 for a full sync), shown as `≈ 15+ calls (first full sync)`; refused only if the floor itself exceeds the calls left. The CLI and the TUI show the same figure |
| `tb --compare` row order | Deterministic (the numbers don't depend on it) |
| Daily-limit reset wording | Xero resets each org's daily limit at its own time ("a fixed window which will reset at different times for each tenant"). The hint says so; a test scans the crates' sources, tests and snapshots for wording that implies a fixed reset |
| Calls left without a reported figure | Marked as an estimate wherever it shows: TUI header `≈ 1,000 left (estimated)`, Overview `≈ 1,000 (estimated)`; `usage` text `≈ 1000 (estimated)`, `--json` keeps the number and adds `"day_remaining_estimated": true` |
| Count wording | One plural helper: `1 account differs` / `2 accounts differ`, `call`/`calls`, `pair`/`pairs`, `line`/`lines`, `item`/`items`; a snapshot scan fails on `1 accounts` and the like |

## Classification (`core::classify`, v0.11.0)

Two files, both local, neither ever sent to Xero (V18):

| File | What | Written how |
|---|---|---|
| `classify.toml` (beside `ownpurse.json`; `"classify": {"config": …}`) | the catalog (label types and choices with aliases), rules, `[queue].required`, group kinds and roles, `[jev]` (must stay `enabled = false` in v0.11.0), `[tree]` (validated, not executed) | staged edit → unified diff → `classify.toml.tmp`, fsync, copy the old file to `classify.toml.~N~` (20 kept), rename; refused if the file changed on disk since it was loaded; a `config_written` store event records `{before_hash, after_hash, summary, backup, diff}` |
| `db/classify.sqlite` (`"classify": {"store": …}`) | one `events` table: `id, at, actor, batch, kind, payload_json, prev_hash, hash`; append-only triggers; `hash = sha256(prev_hash + canonical({id, at, actor, batch, kind, payload}))` | the ONLY write path; one writer at a time (`flock` on the store's folder; not on the database file, whose SQLite `fcntl` locks `flock` would disturb on macOS), `BEGIN IMMEDIATE`, then a re-read of other writers' events |

- **Reads are materialised in memory** from the events (labels, suggestions, notes, groups, writes per line) and
  rebuilt whenever an `undo` event arrives. Undo appends an `undo` event naming event ids; materialising skips
  reversed events, so the previous value returns; an undo can be undone. A config undo restores the backup whose
  hash is the write's `before_hash`.
- **Anchors** `org:doc_type:doc_id:line_id` (`bt`, `mj`, `inv`, `bill`; `L<n>` when Xero gives no line id) name
  bank-transaction, manual-journal, invoice and bill lines from the record (read-only). Each label stores the
  line's **fingerprint** (org, document, line, account, signed amount, date, description hash); a different
  current fingerprint renders the label stale, never deletes it (V19).
- **Confidence** is an exact decimal (no `f64`).
- **Who wrote it**: `actor` is `human`, `agent:<name>`, `rule:<id>` or `jev:<x>`; `batch` is one send. A label's
  `source` is where the value came from (`rule:<id>` when a human accepts a rule suggestion, `agent:<name>` when an
  agent confirms or sets, `human` when typed). Agent sends of more than 50 lines need `allow_bulk` (`too_many`).

### Classification v0.11.0 additions

- **Scope.** `[queue] scope = "year:2025"` (any filter) is ANDed into every read and write over lines: queue,
  counts, suggest, bulk preview, export, would-match. `classify queue` shows `lines/open/stale` for the filter
  and `total_*` for the scope.
- **Lazy store.** Reads never create `db/classify.sqlite` (a missing store reads as empty); the first append
  creates it.
- **Groups.** `Workspace::group_detail` orders members origin → mirror → settlement and runs the chain check:
  one-to-one (signed: origin + mirror = 0), a gap (`mirror missing`, `amount differs by X`, same direction), or
  many-to-one, which uses the origins' SIGNED net, reports only the share of the mirror it explains (a second
  display line) and never claims a line-level match. A settlement group gets a running balance in which the two
  legs of one settlement (same size, opposite signs, different orgs, within `[groups] settle_window_days`,
  default ±7; paired one-to-one, nearest date first, ties by anchor) count once, on the later leg's date, and a
  leg without its partner counts once, marked `(org leg only)`; the pair-rule check reads the gap between the two
  legs as `in transit 07-14 → 07-15`, not a mismatch; `pair_check` tests a config-driven `[groups.pair_rules.<id>]`
  (`a`/`b` sides: org, `plus`/`minus` account codes or ranges, `sign = "opposite"|"same"`) on the ledger
  balances before and after each settlement. `suggest_members` proposes lines of the same size within ±7 days
  in another org.
  Pair-rule orgs must be configured aliases: checked at `Workspace::open`, on hand-edit reloads and by
  `classify rules check` (`file:line` errors).
- **Row cache.** Queue rows and scope membership are materialised once and updated only for the lines new events
  touch (a full rebuild after an undo or a config change), so a write followed by `queue()` + `counts()` stays
  under 2 ms at 5,000 lines (bench-gated). `Workspace` holds the cache in a `RefCell` (it is `Send`, not `Sync`).
- **Opening at size.** `classify::anchor::enumerate` parses only the fields a line needs (borrowed, typed) and
  reads the latest versions through the record's unique index; lines with no label or suggestion get their queue
  row without per-type lookups. On a synthetic record of the demo plus 10,000 documents (19,048 lines;
  `cargo run -p xtask -- demo big <dir>`), `Workspace::open` takes about 30 ms and the first `queue()` + `counts()`
  about 8 ms (bench `classify_open`, gated at 2 × 100 ms).
- **`suggest` is idempotent**: with no rule or record change a second run appends nothing.
- **Agent writes** (`core::classify::request`): the bridge request `kind: "classify"`, schema `cv` 1, ops
  `set clear accept note group_create group_add group_remove group_status undo config`. Every request carries
  `client` (actor `agent:<client>`) and `expect_store_rev` (the STORE revision; distinct from the view `rev`).
  `Workspace::apply` is the single write path for the CLI and the TUI: caption rules (`bad_caption`), the store
  revision (`view_changed`, exit 4), targets (anchors, or a filter resolved in the scope with `status:open` unless
  it names a status), the 50-line limit (`too_many` unless `allow_bulk`), then one batch. Types a human
  confirmed are skipped (`skipped`, reason `human_label`) unless `overwrite`. An agent undoes only agent events.
  A config write needs `confirm: true`; `dry_run` returns the diff. The CLI's `classify set|note|group add|remove|status|create|undo|
  config add-project` build the same request and send it through a running TUI for this profile (so `user_busy`
  and the notice apply), or apply it directly with `--direct` or when no TUI runs.

## Manifests (`core::manifest`)

A manifest collects proposed changes to the books, each ONE Xero document before → after, stored as an
RFC 6902 JSON Patch against the NORMALISED "before" (`manifest::normalise`: noise fields dropped, one ISO
`Date`, line arrays keyed by `LineItemID` or `n<k>`, attachments keyed by `FileName`, money as exact decimal
strings). The store, `db/manifests.sqlite`, is append-only and SHA-256-chained like the classify store; the
current manifest for the CLI is `db/manifests.current` beside it.

- **The view.** `View::open(record, ctx, manifest)` is the one way the books are read. An entry applies
  while main still equals its "before" (`before_fp`); when main equals its "after" (`after_fp`, the read-back
  of a landed write) it is POSTED and the overlay does nothing for it; anything else is a conflict, listed,
  never applied. Effects (TB, statements, intercompany bridges, suspense) come from the same ledger engine.
- **The rules table** (`manifest::rules::RULES`, data + predicates): a new ManualJournal → create; only
  `/Status` POSTED → VOIDED → void; only `/Attachments/<name>` added → attach; one line's
  `AccountCode`/`Description` on an UNRECONCILED BankTransaction → recode. Xero refuses API edits of a
  RECONCILED bank transaction (HTTP 400, "cannot be edited as it has been reconciled with a Bank Statement";
  verified on the Demo Company), so a recode of a reconciled line is carried out by its entry's resolution:
  `reclass` (the default) compiles into the org's monthly reclass journal (Dr new / Cr old for the line's
  amount, one journal per org per month, the narration naming each bank transaction and line, plus the
  `XA:` marker), and the entry is posted once that journal is in the record; `browser` becomes a checklist
  step in the package (unreconcile → edit → re-reconcile; never automated). An entry with no resolution is a
  CHECK error offering both (`ownpurse manifest resolve <n> reclass|browser`; `resolve default …` sets the
  manifest's default). Anything else is "not writable", with the supported path as the hint.
- **CHECK** (local): errors block PLAN (stale before, not writable, unresolved reconciled line, unbalanced,
  unknown/inactive/bank account, lock date, the accountant gate, a duplicate journal, two entries on one
  document, a recode line without a Xero line id); warnings are acknowledged as undoable events
  (attachments are permanent, amount outliers, suspense left non-zero, a balanced bridge broken, a journal
  over the line limit).
- **PLAN** compiles a checked manifest into the sealed write package (`Package.source` names the manifest),
  using the write path's own op builders. Approve and apply stay with the CLI (`manifest approve|apply` wrap
  `write approve-record|run`, same locks). The TUI and the agent bridge never start an apply: no key, no
  palette entry, no verb (`apply`/`approve` are forbidden words); the TUI shows the exact commands, built by
  `compile::commands`, the one builder the CLI prints.
