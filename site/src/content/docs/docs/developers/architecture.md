---
title: Architecture
description: The crates, the dependencies and why each one is there, the tests and gates, the safety rules and the deliberate behaviours of ownpurse.
sidebar:
  order: 1
---

## Principles

1. **Solid test coverage.** Every behaviour has a test; goldens to the cent; invariants as tests.
2. **Performance at the core.** Budgets checked by benchmarks; one frame per interaction; an idle TUI writes
   nothing.
3. **Simplicity.** Few crates, few dependencies, each justified; plain data structs; one Elm-style update loop; no
   async runtime.

## Crates

Everything lives in the Cargo workspace under `rust/`.

| Crate | Contents | Depends on |
|---|---|---|
| `ownpurse-core` | The record (read-only SQLite access), exact-decimal JSON decoding, the chart, the ledger builder, bases (Xero opening / full history), parity, the intercompany engine, compare, config (layered, with origins), the agent-bridge registry, PNG rasterising, classification (`classify`: the label store, `classify.toml`, rules, the queue model), manifests (`manifest`: the entry store, normalisation, the rules table, CHECK, PLAN into the write package), the write package side (`write`), the explainer models (`viz`: bridge, flow, T-account, timeline, before/after, checklist) | rusqlite, serde, serde_json, rust_decimal, sha2, toml_edit, json-patch, caretline-tour |
| `ownpurse-api` | OAuth PKCE, the token store (cross-process lock), the HTTP client, rate limits, the call log, sync, the record's write side, the write transport | core, ureq |
| `ownpurse-cli` (binary `ownpurse`) | The CLI: read commands over the record; `sync`, `login`, `orgs`, reports, history, attachments; manifests and writes; the agent-bridge client (`ui …`) | core, api, clap |
| `ownpurse-tui` (binary `ownpurse-tui`) | The ratatui terminal UI. It runs the installed CLI as a child for anything that calls Xero | core (never api, enforced at compile time), notify, toml_edit, sha2, similar, caretline-layers, caretline-tour |
| `xtask` | Repository checks and generators for the gate (hygiene, the docs grep, docs inputs) | aho-corasick, regex |

The TUI cannot depend on the API crate. It has no code path that sends anything to the provider: it reads the record
and, for sync and login, runs the CLI as a child process after you confirm.

## Dependencies

Each one is justified.

| Crate | Why |
|---|---|
| `rusqlite` (bundled) | The record is SQLite. Bundled means identical SQLite everywhere, with no system library drift |
| `serde`, `serde_json` (`arbitrary_precision`) | Config and Xero payloads. Arbitrary precision keeps every JSON number's digits, so money never passes through `f64` |
| `rust_decimal` | Exact money: an `Amount(Decimal)` newtype with no `From<f64>` |
| `ratatui` + `crossterm` | The TUI. crossterm gives synchronized output, mouse and kitty keyboard support |
| `unicode-width` + `unicode-segmentation` | Grapheme-correct measuring and truncation |
| `clap` (derive) | CLI parsing |
| `ureq` (rustls) | Blocking HTTP. The API layer is sequential by design (60 calls a minute, 5 concurrent, 1,000 a day), so there is no async runtime. One thread per network action, results as messages |
| `sha2` | Content hashes and the record's integrity chain; the classification store's chain and line fingerprints |
| `toml_edit` | One spanned TOML parser for every hand-edited file: `classify.toml`, `~/.config/ownpurse/keys.toml` and `themes/*.toml`. It preserves comments and layout through every write, and its spans give `file:line` in every validation error |
| `json-patch` | Manifests: every proposed change is one document before and after, stored as an RFC 6902 JSON Patch against the normalised document. The crate is the standard diff and apply for that RFC |
| `similar` (TUI) | Word-level highlights inside changed text fields in the diff view |
| `caretline-layers` (TUI) | Overlays: where hint boxes, arrows, rings, spotlights and edge chips go, given the anchors recorded while drawing. Pure: no I/O, no clock; the host draws. From the [caretline](https://github.com/brancusi/caretline) project |
| `caretline-tour` (core, TUI) | Walkthroughs: the file format with its strict parser and lint, the step reducer (state as data in the canonical view state, so replay, jump and scrub are deterministic), and step planning for validation at several sizes |
| `regex` | Intercompany party rules from config (case-insensitive); the repository checks |
| `fontdue` + `png` (core) | `ownpurse ui screen --format png`: the TUI's cell grid is rasterised and encoded as PNG. The font is Hack Regular (MIT and Bitstream Vera licence), embedded |
| `notify` (TUI) | Reloads `classify.toml`, `keys.toml`, themes, `ownpurse.json` and the record the moment they change. The OS file-event API wakes the loop, so there is no polling timer and an idle TUI writes nothing |
| `libc` | `flock` (token file, session registry), `kill(pid, 0)` (pruning dead TUI sessions), terminal and signal handling |
| `aho-corasick` (xtask) | Multi-term scans in the repository checks |
| dev: `insta`, `proptest`, `criterion` | Snapshots, property tests, benchmarks |
| dev: `cargo-llvm-cov` (tool) | Coverage |

No tokio, no chrono-tz (dates are ISO strings and day arithmetic in a small module), no ORM.

## Tests and gates

| Kind | What | Target |
|---|---|---|
| Unit | Every module; `update(App, Msg)` is pure and unit-tested | — |
| Goldens | JSON and text **to the cent** for `tb`, `gl`, `parity`, `ic`, `compare` and every CLI command, against frozen goldens on committed fixtures | Exact |
| Frozen reference artifacts | `rust/tests/fixtures/frozen/`: records, a call log and a two-round sync that the writer, the HTTP client and the syncer must keep reproducing byte for byte | Exact |
| Property (proptest) | Decimal formatting round-trips; every document's postings sum to 0 or are flagged; TB debits equal credits; opening + activity = closing; fiscal-year arithmetic | — |
| Snapshot | `TestBackend` + `insta` for every screen and state at 60, 80, 100 and 140 columns, in dark, light, ANSI and `NO_COLOR` | Every screen |
| Invariants | No write path in the TUI, zero calls on navigation, network only from the palette, the header never truncates, the residual never hides, amounts never truncate, compare rules, decimals only, never colour alone, idle writes nothing. The round-trip law: export, import and export of the canonical view state is byte-identical, and a fresh app built from it draws the live frame cell for cell, after every step of a seeded random walk over keys, clicks, resizes, ticks and agent verbs. Contrast: every built-in theme, accent and surface keeps text at 4.5:1 or more | All |
| State versions | Every release reads the previous releases' frozen view states (`rust/tests/fixtures/state/v*/`) and freezes its own | All |
| Concurrency | Six token-refresher processes race on one file for 50 rounds: exactly one refresh per round | 50/50 |
| Coverage | `cargo llvm-cov` | ≥ 85% of lines |
| Benchmarks | criterion: cold start to first frame ≤ 200 ms; tab or drill ≤ 8 ms; org switch ≤ 16 ms; compare ≤ 30 ms | Fail at 2× |

`rust/scripts/check.sh` runs all of it, plus repository hygiene and the docs grep. Every release tag is gated on it.
See [Contributing](/docs/developers/contributing/).

## Safety rules

- **OAuth.** PKCE, no client secret. One refresher at a time: re-read under an exclusive `flock`, rotate, write
  atomically with mode 0600. Plain login never requests a write scope.
- **Read-only TUI.** No write path in the TUI, zero calls on navigation, network actions only from the palette after
  a confirm with a call estimate.
- **The record is append-only.** History tables refuse `UPDATE` and `DELETE` (triggers), and a SHA-256 chain links
  every appended row.
- **Writes need three locks.** `allow_writes`, `OWNPURSE_WRITE_RUN=<hash>` and a recorded approval of that hash.
  See [Writes](/docs/guides/writes/).
- **SQLite initialisation.** `ownpurse_core::record::sqlite_ready()` runs SQLite's first-time initialisation once, on
  one thread, before any connection opens (racing it is unsafe on arm64).

## Behaviour notes

Each is deliberate and tested.

| Area | Behaviour |
|---|---|
| Unaliased tenants | Only configured aliases resolve; the token file is never read for names. Unaliased tenants are labelled by their first 8 characters |
| Missing record | Error: "The local record does not exist yet." Read commands never create a file |
| Argument errors | clap wording; the structured `usage` code and exit 2 |
| Party regexes | The `regex` crate, case-insensitive. Unsupported syntax (lookarounds, backreferences) is a config error, never silently skipped |
| Network commands | `sync`, `login`, `orgs`, `report fetch`, `history`, `attachments` and the sending write commands share one record, token file and lock. `--offline` makes them refuse. `init`, `q`, `verify`, `skills` and `status` answer `not_available` (exit 2) |
| Estimate for a never-synced org | 1 call per table plus extra passes (15 for a full sync), shown as `≈ 15+ calls (first full sync)`; refused only if that floor exceeds the calls left. The CLI and the TUI show the same figure |
| `tb --compare` row order | Deterministic (the numbers don't depend on it) |
| Daily-limit reset wording | Xero resets each org's daily limit at its own time. The hint says so; a test scans the sources for wording that implies a fixed reset |
| Calls left without a reported figure | Marked as an estimate wherever it shows: `≈ 1,000 left (estimated)`; `--json` keeps the number and adds `"day_remaining_estimated": true` |
| Count wording | One plural helper: `1 account differs` / `2 accounts differ`. A snapshot scan fails on `1 accounts` and the like |

## Classification internals

Two files, both local, neither ever sent to Xero:

| File | What | Written how |
|---|---|---|
| `classify.toml` | The catalog, rules, `[queue].required`, group kinds and roles | Staged edit → unified diff → temp file, fsync, copy the old file to `classify.toml.~N~` (20 kept), rename. Refused if the file changed on disk since it was loaded. A `config_written` event records the before and after hashes, a summary, the backup and the diff |
| `db/classify.sqlite` | One `events` table: `id, at, actor, batch, kind, payload_json, prev_hash, hash`; append-only triggers; `hash = sha256(prev_hash + canonical({id, at, actor, batch, kind, payload}))` | The only write path. One writer at a time (`flock` on the store's folder), `BEGIN IMMEDIATE`, then a re-read of other writers' events |

- **Reads are materialised in memory** from the events and rebuilt when an `undo` arrives. Undo appends an event
  naming event ids; materialising skips reversed events, so the previous value returns. An undo can be undone.
- **Anchors** `org:doc_type:doc_id:line_id` (`bt`, `mj`, `inv`, `bill`; `L<n>` when Xero gives no line id) name
  lines. Each label stores the line's fingerprint; a different current fingerprint makes the label stale, never
  deletes it.
- **Confidence** is an exact decimal.
- **Who wrote it.** `actor` is `human`, `agent:<name>`, `rule:<id>` or `jev:<x>`; `batch` is one send.
- **Agent writes** go through one path, `Workspace::apply`, for both the CLI and the TUI: caption rules, the store
  revision (`view_changed`, exit 4), target resolution, the 50-line limit, then one batch. Labels a person confirmed
  are skipped unless `overwrite`. An agent undoes only agent events.
- **Performance.** On a synthetic record of 10,000 documents (19,048 lines), opening takes about 30 ms and the first
  queue and counts about 8 ms. A write followed by a queue read stays under 2 ms at 5,000 lines (benchmark-gated).

## Manifest internals

A manifest collects proposed changes, each one Xero document before and after, stored as an RFC 6902 JSON Patch
against the **normalised** "before": noise fields dropped, one ISO `Date`, line arrays keyed by `LineItemID` or
`n<k>`, attachments keyed by `FileName`, money as exact decimal strings. The store, `db/manifests.sqlite`, is
append-only and SHA-256-chained like the classification store; the current manifest for the CLI is
`db/manifests.current` beside it.

- **The view.** `View::open(record, ctx, manifest)` is the one way the books are read. An entry applies while main
  still equals its "before"; when main equals its "after" (the read-back of a landed write) it is posted; anything
  else is a conflict, listed, never applied. Effects (TB, statements, intercompany bridges, suspense) come from the
  same ledger engine.
- **The rules table** maps each edit to a write operation: a new ManualJournal → create; only `/Status` POSTED →
  VOIDED → void; only `/Attachments/<name>` added → attach; one line's `AccountCode` or `Description` on an
  unreconciled BankTransaction → recode. A reconciled line's recode is carried out by its resolution (`reclass` or
  `browser`). Anything else is not writable, with the supported path as the hint.
- **CHECK** runs locally. Errors block PLAN; warnings are acknowledged as undoable events.
- **PLAN** compiles a checked manifest into the sealed write package using the write path's own operation builders.
  Approve and apply stay with the CLI. The TUI and the agent bridge never start an apply: `apply` and `approve` are
  forbidden words, and the TUI shows the exact commands, built by the same function the CLI prints from.
