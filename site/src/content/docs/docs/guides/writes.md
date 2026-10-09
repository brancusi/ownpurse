---
title: Writes and classification
description: Propose changes as manifests, check them locally, seal them into a write package you approve by its hash, run them, and label lines locally with classify.
sidebar:
  order: 6
---

ownpurse reads your books freely and changes them only on your say-so. A change goes through four separate steps,
and the first three never touch the provider:

1. **Propose.** Collect changes in a **manifest**: each one is a single document, before and after.
2. **Check.** Run every check locally. Errors block; warnings must be acknowledged.
3. **Plan and approve.** Compile the manifest into a sealed **write package**, identified by its SHA-256 hash. You,
   the owner, approve that exact hash.
4. **Run.** A separate command sends the approved package, in small batches, and reads every target back.

Agents can propose, check and plan. They cannot approve, and the TUI and the agent bridge can never start a run.

## What can be written

| Operation | Notes |
|---|---|
| Create a manual journal | No tax; an unbalanced journal is a check error |
| Recode a bank-transaction line | Account and/or description, on an unreconciled transaction |
| Void a manual journal | A posted journal becomes voided |
| Attach a file | PDF, CSV or TXT, to a journal or a bank transaction. Permanent once applied |
| Add a note | To a journal's or bank transaction's history in Xero, at most 250 characters. Permanent once applied |
| Create a bank transaction | Spend or receive money, for cash with no bank feed. Recovered by deleting it |
| Create a bank transfer | Recovered by a reverse transfer (the history stays) |
| Create a contact | Guarded against an existing exact name. Recovered by archiving it |

Anything else is "not writable", with the supported path as the hint.

**Reconciled lines.** Xero refuses API edits of a reconciled bank transaction. A recode of a reconciled line is
carried out by its entry's resolution instead:

- `reclass` (the default) compiles into the org's monthly reclass journal: debit the new account and credit the old
  one for the line's amount, one journal per org per month, its narration naming each transaction and line.
- `browser` becomes a checklist step in the package: unreconcile, edit and re-reconcile in Xero's web app, by hand.

Set it per entry with `ownpurse manifest resolve <n> reclass|browser`, or for the whole manifest with
`ownpurse manifest resolve default <how>`.

**Every landed write leaves a note.** Each operation that lands adds `ownpurse package <hash12> op <id>: <why>` to the
document's history in Xero, so anyone looking at the document there can trace it back. Notes and other free text
ownpurse writes go out in ASCII.

## Manifests

A manifest collects proposed changes. Each entry is one document, before and after, stored as an
[RFC 6902](https://www.rfc-editor.org/rfc/rfc6902) JSON Patch against a normalised version of the document. The
store (`db/manifests.sqlite`) is append-only and hash-chained, so every entry, undo and acknowledgement is kept.

While a manifest is current, read commands (`tb`, `gl`, `parity`, `ic`, compare, `classify queue`) show **main plus
the manifest**: your books as they would be. `--manifest <NAME>` picks one for a single command;
`ownpurse manifest switch main` goes back.

```sh
ownpurse manifest new year-end "Year-end cleanup for roast"
ownpurse manifest journal --org roast --date 2025-12-31 --narration "Accrue December rent" \
    --cash-basis no --line 6140=1200.00 --line 2100=-1200.00 --why "Rent invoice arrived in January"
ownpurse manifest recode "roast:bt:<id>#n1" --account 6150 --why "Green beans, not packaging"
ownpurse manifest show            # entries, stage, the next commands
ownpurse manifest diff            # the effect against main: TB, statements, intercompany bridges
ownpurse manifest check           # every issue, locally
ownpurse manifest plan            # compile into the sealed package; prints the approve and apply commands
```

### Adding entries

| Command | Adds |
|---|---|
| `manifest journal` | A new manual journal: `--org`, `--date`, `--narration`, `--cash-basis yes\|no` (required), and `--line CODE=AMOUNT[:description]` repeated (debit positive, credit negative, or `Dr`/`Cr` before the amount), or `--file` as JSON or CSV |
| `manifest edit <seq>` | Replaces a new-journal entry with an edited one |
| `manifest recode <org:bt:id#LINE>` | A bank line's new `--account` and/or `--description`. `LINE` is a line id, `n<k>` or `L<k>` |
| `manifest void <org:mj:id>` | Voids a journal |
| `manifest attach <org:mj:id> <file>` | Attaches a file |
| `manifest note <org:mj:id\|org:bt:id> "<text>"` | A note in the document's history |
| `manifest add --entity E --org O [--id ID] --after F \| --patch F` | Any document change, as its full "after" or a patch |

Every entry takes `--why`, shown in reviews and the package. `manifest annotate <seq> --accountant <ref>` attaches
your accountant's decision reference. `manifest undo [seq]` undoes an entry (`--restore` brings it back).
`manifest discard` marks a manifest discarded; its history stays.

### How an entry applies

An entry applies while the document in the record still equals its "before". When the record equals its "after" (the
read-back of a landed write), the entry is **posted** and the overlay does nothing more for it. Anything else is a
**conflict**: listed, never applied.

### Checks

`manifest check` runs locally and makes no call.

**Errors block planning**: a stale "before", a change that is not writable, an unresolved reconciled line, an
unbalanced journal, an unknown, inactive or bank account, the lock date, the accountant gate, a duplicate journal,
two entries on one document, or a recode line without a Xero line id.

**Warnings must be acknowledged** (`manifest ack <entry> <code>`, itself an undoable event): attachments are
permanent, an amount is an outlier, a suspense account is left non-zero, a balanced intercompany bridge would break,
a journal is over the line limit.

### Agents and manifests

Every manifest write takes `--client <NAME>` (recorded as `agent:<NAME>`; default `human`) and `--expect-rev <N>`,
the `head_rev` the agent last read. If the manifest moved since, the write is refused with `view_changed` (exit 4).
`manifest approve` is refused in agent sessions.

In the TUI, tab 7 shows the manifest: entries, issues, effects, the log, a stage strip, one field-by-field diff
view, and the exact approve and apply commands. There is no apply key, palette entry or agent verb.

## Write packages

A package is what actually reaches Xero. `manifest plan` builds one from a manifest; `write plan` builds one from an
operations file directly.

| Step | Command | Calls Xero |
|---|---|---|
| Plan | `write plan <ops.json>`: requests, live preconditions, expected trial-balance deltas and a pre-compiled recovery, into an unsealed package | No |
| Seal | `write seal <package>`: fixes its SHA-256 and writes a review PDF and CSV for your accountant | No |
| Approve | `write approve-record <package> --response-file F --i-am-the-owner`: you record your approval at your own terminal, quoting the hash | No |
| Check approval | `write approve-check <package>`: exit 0 if an approval is recorded, 3 if not | No |
| Run | `write run <package>`: the next batch (at most 10 operations) | Yes |
| Verify | `write verify <package>`: re-read each landed target and the trial balance movement | Yes |

`manifest approve` and `manifest apply` wrap `write approve-record` and `write run` with the same locks.

### The three locks

`write run` sends nothing unless all three hold:

1. the profile has `allow_writes: true`, and you granted the write scopes with `ownpurse login --write`;
2. `OWNPURSE_WRITE_RUN=<hash>` is set to this package's hash, for this one run;
3. an approval for that hash is recorded.

A run goes through a guarded transport: an endpoint allow-list, a tenant guard, an idempotency key on every request,
and a `sending` entry logged before each request leaves. `--dry-run` checks the preconditions live and prints each
request without sending anything. Operations on a frozen period are held unless your accountant's decision is named.

### After a run

- **Read-back.** Run and verify read every target back into the record (source `readback`), so the record and the TUI
  follow a write at once.
- **Verify** answers `verified`, `rejected`, `uncertain`, `incomplete` or `discrepancy` (the trial balance moved
  differently from the expected deltas; exit 5). The record keeps each outcome.
- **Uncertain** sends are never retried automatically. `write reconcile <package> <op>` settles one by reading back,
  never by re-sending.
- **Recovery.** `write recover <package> [op]` builds a recovery package for what landed. It needs its own approval.
- `write status <package>` shows the attempt log and the record's chain, with no call.

System › Writes in the TUI lists every package with its stage and approval source.

## Classification

Classification is a local layer of labels on Xero lines: **project**, **bearer**, **treatment** and **should-be
account**, plus notes and groups. Labels are your intent. They never change Xero.

Two files hold it, both beside your `ownpurse.json` by default:

| File | What | How it changes |
|---|---|---|
| `classify.toml` | The catalog (label types and choices, with aliases), rules, queue settings, group kinds | Hand-editable. Edits from the TUI or an agent are shown as a diff, written atomically, and keep numbered backups (`classify.toml.~N~`, 20 kept) |
| `db/classify.sqlite` | Every label, suggestion, note and group as an event | Append-only and SHA-256-chained. Every event records who wrote it: `human`, `agent:<name>` or `rule:<id>`. Every write can be undone, and an undo can be undone |

A label remembers the line it was set on (org, document, line, account, signed amount, date and a hash of the
description). If a later sync changes the line, the label shows as **stale**. It is never deleted.

`[queue] scope = "year:2025"` in `classify.toml` limits every read and write to that filter.

### Commands

| Command | What |
|---|---|
| `classify suggest [filter]` | Run the rules over the lines and store their suggestions. Nothing is confirmed. Idempotent; `--dry-run` reports what would be written |
| `classify queue [filter]` | Lines without confirmed required labels, or with stale ones: stale first, then largest (`--sort stale\|amount\|date`) |
| `classify set <anchors or filter> <type>=<id>…` | Set labels, e.g. `classify set org:roast month:2025-09 project=harvest` |
| `classify note <target> <text>` | A note on a line or a group |
| `classify group create\|add\|remove\|status` | Groups of lines: kinds from `classify.toml`, roles `origin`, `mirror`, `settlement`, status `open`, `complete`, `gap` or `accountant` |
| `classify undo [anchor] [--batch] [--events ids]` | Undo an agent write |
| `classify config add-project <id> <name>` | Add a project to `classify.toml` (`--dry-run` shows the diff, `--confirm` writes it) |
| `classify rules check` | Validate `classify.toml`: every problem as `file:line: path: reason` |
| `classify export [filter] --csv` | One CSV row per line: Xero facts, each label and its source, stale, notes |

Filters: `org:<alias> acct:<prefix> month:YYYY-MM year:YYYY project:<id> conf<0.7 by:agent since:today
status:open|all|stale`, plus bare words that search payee and description. Anchors name a line as
`org:doc_type:doc_id:line_id`, with `bt`, `mj`, `inv` or `bill` as the type.

### Writes by agents

The write commands (`set`, `note`, `group …`, `undo`, `config add-project`) are built for agents:

- `--client <NAME>` records the writer as `agent:<NAME>`.
- `--expect-store-rev <N>` is the store revision the agent last read (`classify queue --json`). If the store moved,
  the write is refused with `view_changed` (exit 4).
- A label a person confirmed is skipped unless `--overwrite`.
- At most 50 lines per write unless `--allow-bulk` (`too_many`).
- `--caption` shows one line with the write.
- The write goes through the running TUI for the profile when there is one, so the person sees it and can press `U`
  to undo it. `--direct` writes the store directly.

The rule conditions you can use in `classify.toml` are listed in the
[configuration reference](/docs/guides/configuration/#classify).
