---
title: CLI reference
description: Every ownpurse command and option, generated from the binary's own help.
sidebar:
  order: 1
---

This page is generated from `ownpurse --help` and the help of every subcommand (ownpurse 0.16.0). Run
`ownpurse <command> --help` for the same text in your terminal.

Commands marked ⇅ call the provider (Xero today). Every other command reads the local record and spends zero API
calls.

## Global options

These options work on every command. They are left out of the per-command help below to keep it short.

```text
      --profile <PROFILE>  profile from the config (default: default_profile)
      --json               machine-readable output
      --offline            never call Xero (network commands refuse; read commands never call Xero anyway)
      --full               show all rows and extra columns
      --limit <LIMIT>      rows to show (default 50)
      --manifest <NAME>    read main + this manifest (tb, gl, parity, ic, compare, classify queue; default: the current manifest)
  -h, --help               Print help
  -V, --version            Print version
```

## Exit codes

| Code | Meaning |
|---:|---|
| `0` | Success |
| `1` | Runtime error (also `--strict` when something differs) |
| `2` | Usage error, or a request the tool refuses by design |
| `3` | `write approve-check`: no approval is recorded for the package |
| `4` | Refused because the state moved: a view changed under an agent, the user is busy, a stale revision, a timeout |
| `5` | `write verify`: the trial balance moved differently from the expected deltas |

## ownpurse

```text
ownpurse read commands over the local record (no network)

Usage: ownpurse [OPTIONS] <COMMAND>

Commands:
  demo         Launch a scripted walkthrough in the real TUI over an isolated mock database (offline; no setup)
  manifest     Manifests: collect changes as document diffs, check them locally, plan them into a sealed write package (nothing reaches Xero before approve + apply)
  walkthrough  Walkthroughs: guided, replayable explanations played in the TUI (`W`); files beside ownpurse.json
  classify     Classify lines locally: rules suggest project, bearer and treatment; labels live in db/classify.sqlite, never in Xero
  ui           The agent bridge: read and change a running TUI's view (view only; local socket)
  write        Xero changes as a sealed, approved package (plan, seal, approve; the run side is separate)
  doctor       Check the setup (binaries on PATH, config, record, token file permissions); read-only, prints fixes
  sync         ⇅ Mirror entities into the local record (calls Xero)
  login        ⇅ Browser consent (PKCE): read access plus anything already granted; one consent covers every org
  orgs         ⇅ List connected organisations (1 call; `--offline` reads the token file's last list)
  accounts     Chart of accounts (one org or all)
  org          Organisation details: plan class, lock dates
  contacts     Contacts
  txns         Bank transactions (spend/receive money)
  journals     Manual journals
  transfers    Bank transfers
  show         One record: current payload and version history
  open         The document's page in the Xero web app, through the org switch (local; no Xero call)
  list         List any mirrored entity (Invoices, Payments, CreditNotes, TaxRates, …)
  gl           Local general ledger: postings derived from mirrored documents
  tb           Local trial balance computed from the ledger (exact decimals)
  pl           Profit and loss from the local ledger (accrual or --cash), checked against Xero's cached report
  bs           Balance sheet from the local ledger (accrual or --cash), checked against Xero's cached report
  parity       Compare the local trial balance with Xero's cached one, account by account
  report       Xero's reports: fetch one into the record (⇅ one call per org), or list what the record holds
  export       A file for the accountant from the record (zero calls): tb / gl from the local ledger, bs / pl from Xero's cached reports
  attachments  ⇅ A document's attachments from Xero, into the record; `--get` saves one file (`--cached`: the record)
  history      ⇅ A document's history and notes from Xero, into the record (`--cached`: what the record holds)
  usage        API usage: calls, quotas, endpoints, records fetched
  config       Effective settings with their source; links; edit
  ic           Intercompany pairs from config: both sides, difference, bridge, residual (zero calls)
  viz          Explainer models (bridge, flow, t_account, timeline, before_after, checklist): the numbers the TUI's panels show (zero calls)
  help         Print this message or the help of the given subcommand(s)

Options:
      --profile <PROFILE>  profile from the config (default: default_profile)
      --json               machine-readable output
      --offline            never call Xero (network commands refuse; read commands never call Xero anyway)
      --full               show all rows and extra columns
      --limit <LIMIT>      rows to show (default 50)
      --manifest <NAME>    read main + this manifest (tb, gl, parity, ic, compare, classify queue; default: the current manifest)
  -h, --help               Print help
  -V, --version            Print version
```

## ownpurse demo

Launch a scripted walkthrough in the real TUI over an isolated mock database (offline; no setup)

```text
Launch a scripted walkthrough in the real TUI over an isolated mock database (offline; no setup)

Usage: ownpurse demo [OPTIONS]

Options:
      --step-seconds <STEP_SECONDS>  seconds each step stays on screen; moving the view yourself stops autoplay [default: 8]
```

## ownpurse manifest

Manifests: collect changes as document diffs, check them locally, plan them into a sealed write package (nothing reaches Xero before approve + apply)

```text
Manifests: collect changes as document diffs, check them locally, plan them into a sealed write package (nothing reaches Xero before approve + apply)

Usage: ownpurse manifest [OPTIONS] <COMMAND>

Commands:
  new       Create a manifest (changes collected locally, reviewed as a diff) and switch to it
  list      List manifests (main first) with their stage and issues
  switch    Make a manifest (or main) the default for the read commands
  show      The entries, the stage and the commands; or one entry's field diff (`show <seq>`, `--patch` for the raw ops)
  log       The manifest's events: entries, undos, notes, acknowledgements, plans
  diff      The effect against main: TB, statements, intercompany bridges, must-be-zero accounts
  check     Every issue, locally (no Xero call): errors block plan, warnings need acknowledging
  ack       Acknowledge a warning (an undoable event): `ack <entry> <code> [--path P]`
  plan      Compile the checked manifest into the sealed write package (local; prints the approve and apply commands)
  approve   The owner approves the planned package at his own terminal (= write approve-record; refused in agent sessions)
  apply     ⇅ Apply the approved package (= write run; needs OWNPURSE_WRITE_RUN=<hash> for this one run)
  add       Add any document change: its "after" (`--after file.json`) or an RFC 6902 patch (`--patch file.json`)
  journal   A new manual journal (NoTax; check reports an unbalanced one)
  edit      Replace a new-journal entry with an edited one (shown as "edited → n")
  recode    Recode a bank line's account and/or description (org:bt:id#LINE; LINE a line id, n<k> or L<k>)
  void      Void a document (org:mj:id)
  attach    Attach a PDF, CSV or TXT to a journal (org:mj:id <file>; permanent once applied)
  note      A note in a journal's or bank transaction's history (org:mj:id or org:bt:id "<text>"; permanent once applied; at most 250 characters)
  resolve   How a reconciled bank line's recode is carried out: `reclass` (a reclass journal, the default) or `browser` (unreconcile, edit, re-reconcile: a checklist step); `resolve default <how>` sets the manifest's default
  undo      Undo an entry (default: the newest), or `--restore` it
  annotate  Attach an accountant decision reference and/or a note to an entry
  discard   Discard a manifest (a marker; its history stays)
  help      Print this message or the help of the given subcommand(s)
```

### ownpurse manifest new

Create a manifest (changes collected locally, reviewed as a diff) and switch to it

```text
Create a manifest (changes collected locally, reviewed as a diff) and switch to it

Usage: ownpurse manifest new [OPTIONS] <NAME> [WHY]

Arguments:
  <NAME>
  [WHY]   why: one line, shown in the list, the review and the package title

Options:
      --client <NAME>      your name: the change is recorded as agent:<NAME> (default: human)
      --expect-rev <N>     the manifest head_rev you last read; refused with view_changed (exit 4) if it moved
      --overwrite          undo or discard in a manifest another actor created (agents only need it)
```

### ownpurse manifest list

List manifests (main first) with their stage and issues

```text
List manifests (main first) with their stage and issues

Usage: ownpurse manifest list [OPTIONS]
```

### ownpurse manifest switch

Make a manifest (or main) the default for the read commands

```text
Make a manifest (or main) the default for the read commands

Usage: ownpurse manifest switch [OPTIONS] <NAME>

Arguments:
  <NAME>
```

### ownpurse manifest show

The entries, the stage and the commands; or one entry's field diff (`show <seq>`, `--patch` for the raw ops)

```text
The entries, the stage and the commands; or one entry's field diff (`show <seq>`, `--patch` for the raw ops)

Usage: ownpurse manifest show [OPTIONS] [SEQ]

Arguments:
  [SEQ]

Options:
      --patch              the raw RFC 6902 patch
```

### ownpurse manifest log

The manifest's events: entries, undos, notes, acknowledgements, plans

```text
The manifest's events: entries, undos, notes, acknowledgements, plans

Usage: ownpurse manifest log [OPTIONS]
```

### ownpurse manifest diff

The effect against main: TB, statements, intercompany bridges, must-be-zero accounts

```text
The effect against main: TB, statements, intercompany bridges, must-be-zero accounts

Usage: ownpurse manifest diff [OPTIONS]

Options:
      --date <DATE>        as-of date (default: the latest cached Xero TB date)
      --csv                CSV instead of text
```

### ownpurse manifest check

Every issue, locally (no Xero call): errors block plan, warnings need acknowledging

```text
Every issue, locally (no Xero call): errors block plan, warnings need acknowledging

Usage: ownpurse manifest check [OPTIONS]

Options:
      --date <DATE>
```

### ownpurse manifest ack

Acknowledge a warning (an undoable event): `ack <entry> <code> [--path P]`

```text
Acknowledge a warning (an undoable event): `ack <entry> <code> [--path P]`

Usage: ownpurse manifest ack [OPTIONS] <ENTRY> <CODE>

Arguments:
  <ENTRY>
  <CODE>

Options:
      --path <PATH>
      --withdraw           withdraw the acknowledgement
      --client <NAME>      your name: the change is recorded as agent:<NAME> (default: human)
      --expect-rev <N>     the manifest head_rev you last read; refused with view_changed (exit 4) if it moved
      --overwrite          undo or discard in a manifest another actor created (agents only need it)
```

### ownpurse manifest plan

Compile the checked manifest into the sealed write package (local; prints the approve and apply commands)

```text
Compile the checked manifest into the sealed write package (local; prints the approve and apply commands)

Usage: ownpurse manifest plan [OPTIONS]

Options:
      --date <DATE>
      --client <NAME>      your name: the change is recorded as agent:<NAME> (default: human)
      --expect-rev <N>     the manifest head_rev you last read; refused with view_changed (exit 4) if it moved
      --overwrite          undo or discard in a manifest another actor created (agents only need it)
```

### ownpurse manifest approve

The owner approves the planned package at his own terminal (= write approve-record; refused in agent sessions)

```text
The owner approves the planned package at his own terminal (= write approve-record; refused in agent sessions)

Usage: ownpurse manifest approve [OPTIONS] --package <PACKAGE> --response-file <RESPONSE_FILE> <NAME>

Arguments:
  <NAME>

Options:
      --package <PACKAGE>              the planned package's full hash
      --response-file <RESPONSE_FILE>
      --i-am-the-owner
```

### ownpurse manifest apply

⇅ Apply the approved package (= write run; needs OWNPURSE_WRITE_RUN=<hash> for this one run)

```text
⇅ Apply the approved package (= write run; needs OWNPURSE_WRITE_RUN=<hash> for this one run)

Usage: ownpurse manifest apply [OPTIONS] --package <PACKAGE> <NAME>

Arguments:
  <NAME>

Options:
      --package <PACKAGE>  the planned package's full hash
      --batch <BATCH>
      --dry-run
```

### ownpurse manifest add

Add any document change: its "after" (`--after file.json`) or an RFC 6902 patch (`--patch file.json`)

```text
Add any document change: its "after" (`--after file.json`) or an RFC 6902 patch (`--patch file.json`)

Usage: ownpurse manifest add [OPTIONS] --entity <ENTITY> --org <ORG>

Options:
      --entity <ENTITY>          the entity (ManualJournals, BankTransactions, …)
      --org <ORG>
      --id <ID>                  the document id (omit for a new document)
      --after <AFTER>
      --patch <PATCH>
      --why <WHY>                [default: ""]
      --accountant <ACCOUNTANT>
      --client <NAME>            your name: the change is recorded as agent:<NAME> (default: human)
      --expect-rev <N>           the manifest head_rev you last read; refused with view_changed (exit 4) if it moved
      --overwrite                undo or discard in a manifest another actor created (agents only need it)
```

### ownpurse manifest journal

A new manual journal (NoTax; check reports an unbalanced one)

```text
A new manual journal (NoTax; check reports an unbalanced one)

Usage: ownpurse manifest journal [OPTIONS]

Options:
      --org <ORG>                  the org alias
      --date <DATE>                the journal date (YYYY-MM-DD)
      --narration <NARRATION>
      --cash-basis <yes|no>        yes|no: show on cash-basis reports (required; Xero's ShowOnCashBasisReports)
      --line <CODE=AMOUNT[:DESC]>  CODE=AMOUNT[:description], repeated; debit positive, credit negative (or `Dr`/`Cr` before the amount)
      --file <FILE>                the journal as JSON ({org, date, narration, cash_basis, lines}) or CSV (account,description,amount)
      --why <WHY>                  why (shown in reviews)
      --accountant <ACCOUNTANT>    the accountant's decision reference (an adjusting-entry id, an email)
      --client <NAME>              your name: the change is recorded as agent:<NAME> (default: human)
      --expect-rev <N>             the manifest head_rev you last read; refused with view_changed (exit 4) if it moved
      --overwrite                  undo or discard in a manifest another actor created (agents only need it)
```

### ownpurse manifest edit

Replace a new-journal entry with an edited one (shown as "edited → n")

```text
Replace a new-journal entry with an edited one (shown as "edited → n")

Usage: ownpurse manifest edit [OPTIONS] <SEQ>

Arguments:
  <SEQ>

Options:
      --org <ORG>                  the org alias
      --date <DATE>                the journal date (YYYY-MM-DD)
      --narration <NARRATION>
      --cash-basis <yes|no>        yes|no: show on cash-basis reports (required; Xero's ShowOnCashBasisReports)
      --line <CODE=AMOUNT[:DESC]>  CODE=AMOUNT[:description], repeated; debit positive, credit negative (or `Dr`/`Cr` before the amount)
      --file <FILE>                the journal as JSON ({org, date, narration, cash_basis, lines}) or CSV (account,description,amount)
      --why <WHY>                  why (shown in reviews)
      --accountant <ACCOUNTANT>    the accountant's decision reference (an adjusting-entry id, an email)
      --client <NAME>              your name: the change is recorded as agent:<NAME> (default: human)
      --expect-rev <N>             the manifest head_rev you last read; refused with view_changed (exit 4) if it moved
      --overwrite                  undo or discard in a manifest another actor created (agents only need it)
```

### ownpurse manifest recode

Recode a bank line's account and/or description (org:bt:id#LINE; LINE a line id, n<k> or L<k>)

```text
Recode a bank line's account and/or description (org:bt:id#LINE; LINE a line id, n<k> or L<k>)

Usage: ownpurse manifest recode [OPTIONS] <LINE>

Arguments:
  <LINE>

Options:
      --account <ACCOUNT>
      --description <DESCRIPTION>
      --why <WHY>                  [default: ""]
      --client <NAME>              your name: the change is recorded as agent:<NAME> (default: human)
      --expect-rev <N>             the manifest head_rev you last read; refused with view_changed (exit 4) if it moved
      --overwrite                  undo or discard in a manifest another actor created (agents only need it)
```

### ownpurse manifest void

Void a document (org:mj:id)

```text
Void a document (org:mj:id)

Usage: ownpurse manifest void [OPTIONS] <DOC>

Arguments:
  <DOC>

Options:
      --why <WHY>          [default: ""]
      --client <NAME>      your name: the change is recorded as agent:<NAME> (default: human)
      --expect-rev <N>     the manifest head_rev you last read; refused with view_changed (exit 4) if it moved
      --overwrite          undo or discard in a manifest another actor created (agents only need it)
```

### ownpurse manifest attach

Attach a PDF, CSV or TXT to a journal (org:mj:id <file>; permanent once applied)

```text
Attach a PDF, CSV or TXT to a journal (org:mj:id <file>; permanent once applied)

Usage: ownpurse manifest attach [OPTIONS] <DOC> <FILE>

Arguments:
  <DOC>
  <FILE>

Options:
      --why <WHY>          [default: ""]
      --client <NAME>      your name: the change is recorded as agent:<NAME> (default: human)
      --expect-rev <N>     the manifest head_rev you last read; refused with view_changed (exit 4) if it moved
      --overwrite          undo or discard in a manifest another actor created (agents only need it)
```

### ownpurse manifest note

A note in a journal's or bank transaction's history (org:mj:id or org:bt:id "<text>"; permanent once applied; at most 250 characters)

```text
A note in a journal's or bank transaction's history (org:mj:id or org:bt:id "<text>"; permanent once applied; at most 250 characters)

Usage: ownpurse manifest note [OPTIONS] <DOC> <TEXT>

Arguments:
  <DOC>
  <TEXT>

Options:
      --why <WHY>          [default: ""]
      --client <NAME>      your name: the change is recorded as agent:<NAME> (default: human)
      --expect-rev <N>     the manifest head_rev you last read; refused with view_changed (exit 4) if it moved
      --overwrite          undo or discard in a manifest another actor created (agents only need it)
```

### ownpurse manifest resolve

How a reconciled bank line's recode is carried out: `reclass` (a reclass journal, the default) or `browser` (unreconcile, edit, re-reconcile: a checklist step); `resolve default <how>` sets the manifest's default

```text
How a reconciled bank line's recode is carried out: `reclass` (a reclass journal, the default) or `browser` (unreconcile, edit, re-reconcile: a checklist step); `resolve default <how>` sets the manifest's default

Usage: ownpurse manifest resolve [OPTIONS] <ENTRY> <HOW>

Arguments:
  <ENTRY>  the entry number, or `default`
  <HOW>    reclass | browser

Options:
      --client <NAME>      your name: the change is recorded as agent:<NAME> (default: human)
      --expect-rev <N>     the manifest head_rev you last read; refused with view_changed (exit 4) if it moved
      --overwrite          undo or discard in a manifest another actor created (agents only need it)
```

### ownpurse manifest undo

Undo an entry (default: the newest), or `--restore` it

```text
Undo an entry (default: the newest), or `--restore` it

Usage: ownpurse manifest undo [OPTIONS] [SEQ]

Arguments:
  [SEQ]

Options:
      --restore
      --client <NAME>      your name: the change is recorded as agent:<NAME> (default: human)
      --expect-rev <N>     the manifest head_rev you last read; refused with view_changed (exit 4) if it moved
      --overwrite          undo or discard in a manifest another actor created (agents only need it)
```

### ownpurse manifest annotate

Attach an accountant decision reference and/or a note to an entry

```text
Attach an accountant decision reference and/or a note to an entry

Usage: ownpurse manifest annotate [OPTIONS] <SEQ>

Arguments:
  <SEQ>

Options:
      --accountant <ACCOUNTANT>
      --note <NOTE>
      --client <NAME>            your name: the change is recorded as agent:<NAME> (default: human)
      --expect-rev <N>           the manifest head_rev you last read; refused with view_changed (exit 4) if it moved
      --overwrite                undo or discard in a manifest another actor created (agents only need it)
```

### ownpurse manifest discard

Discard a manifest (a marker; its history stays)

```text
Discard a manifest (a marker; its history stays)

Usage: ownpurse manifest discard [OPTIONS] [NAME]

Arguments:
  [NAME]

Options:
      --client <NAME>      your name: the change is recorded as agent:<NAME> (default: human)
      --expect-rev <N>     the manifest head_rev you last read; refused with view_changed (exit 4) if it moved
      --overwrite          undo or discard in a manifest another actor created (agents only need it)
```

## ownpurse walkthrough

Walkthroughs: guided, replayable explanations played in the TUI (`W`); files beside ownpurse.json

```text
Walkthroughs: guided, replayable explanations played in the TUI (`W`); files beside ownpurse.json

Usage: ownpurse walkthrough [OPTIONS] <COMMAND>

Commands:
  new       Create an empty walkthrough file (walkthroughs.dir, default walkthroughs/ beside ownpurse.json)
  add-step  Append a step: narration, optional anchored hint/rings, view verbs, checks, a visualisation spec
  show      A walkthrough's steps
  list      The walkthroughs in the folder
  validate  Lay every step out at 140x40, 100x30 and 80x24: anchors resolve, checks hold, glossary links exist, no box over a protected region
  render    Write every step's frame (text, svg, png) for review or as a slideshow
  help      Print this message or the help of the given subcommand(s)
```

### ownpurse walkthrough new

Create an empty walkthrough file (walkthroughs.dir, default walkthroughs/ beside ownpurse.json)

```text
Create an empty walkthrough file (walkthroughs.dir, default walkthroughs/ beside ownpurse.json)

Usage: ownpurse walkthrough new [OPTIONS] --title <TITLE> <ID>

Arguments:
  <ID>

Options:
      --title <TITLE>
      --audience <AUDIENCE>  lay | informed
```

### ownpurse walkthrough add-step

Append a step: narration, optional anchored hint/rings, view verbs, checks, a visualisation spec

```text
Append a step: narration, optional anchored hint/rings, view verbs, checks, a visualisation spec

Usage: ownpurse walkthrough add-step [OPTIONS] --title <TITLE> --text <TEXT> <ID>

Arguments:
  <ID>

Options:
      --title <TITLE>
      --text <TEXT>              the body: plain English, **bold**, [term] glossary links
      --why <WHY>                a one-line "why this matters" (lay mode)
      --anchor <ANCHORS>         an anchor (row:code=6140, chip:stage, diff:/Date, …); repeat for rings on more
      --hint <HINT>              the hint text on the first anchor
      --hint-title <HINT_TITLE>
      --arrow                    an arrow from the hint to its anchor
      --ring
      --spotlight
      --view <VERBS>             a view verb the step applies (`tab 2`, `scope roast`, `select 6140`, `drill`); repeat, in order
      --check <CHECKS>           an assertion: `amount(org, account, YYYY-MM-DD) = value` or `count(org, account[, from, to]) = n`
      --viz <VIZ>                a visualisation spec (JSON `{kind, …}`)
```

### ownpurse walkthrough show

A walkthrough's steps

```text
A walkthrough's steps

Usage: ownpurse walkthrough show [OPTIONS] <ID>

Arguments:
  <ID>
```

### ownpurse walkthrough list

The walkthroughs in the folder

```text
The walkthroughs in the folder

Usage: ownpurse walkthrough list [OPTIONS]
```

### ownpurse walkthrough validate

Lay every step out at 140x40, 100x30 and 80x24: anchors resolve, checks hold, glossary links exist, no box over a protected region

```text
Lay every step out at 140x40, 100x30 and 80x24: anchors resolve, checks hold, glossary links exist, no box over a protected region

Usage: ownpurse walkthrough validate [OPTIONS] <ID>

Arguments:
  <ID>
```

### ownpurse walkthrough render

Write every step's frame (text, svg, png) for review or as a slideshow

```text
Write every step's frame (text, svg, png) for review or as a slideshow

Usage: ownpurse walkthrough render [OPTIONS] --out <OUT> <ID>

Arguments:
  <ID>

Options:
      --out <OUT>
      --format <FORMAT>    txt, svg, png, html (comma-separated) [default: txt]
      --sizes <SIZES>      [default: 100x30]
```

## ownpurse classify

Classify lines locally: rules suggest project, bearer and treatment; labels live in db/classify.sqlite, never in Xero

```text
Classify lines locally: rules suggest project, bearer and treatment; labels live in db/classify.sqlite, never in Xero

Usage: ownpurse classify [OPTIONS] <COMMAND>

Commands:
  set      Set labels on lines (agent write): anchors or a filter (status:open unless it says), then <type>=<id>…
  note     Add a note to a line or a group (agent write)
  group    Groups (agent writes)
  undo     Undo an agent write: --batch (the latest agent batch), an anchor (the last agent write on it) or --events
  config   Edit classify.toml (agent writes)
  suggest  Run the rules over the lines in the filter and store their suggestions (local store only; nothing is confirmed, nothing goes to Xero)
  rules    Check the classification config
  export   One CSV row per line: Xero facts, each label and its source, stale, notes (default: every line)
  queue    The classification queue as data: lines without confirmed required labels, or with stale ones, largest first
  help     Print this message or the help of the given subcommand(s)
```

### ownpurse classify set

Set labels on lines (agent write): anchors or a filter (status:open unless it says), then <type>=<id>…

```text
Set labels on lines (agent write): anchors or a filter (status:open unless it says), then <type>=<id>…

Usage: ownpurse classify set [OPTIONS] <WORDS>...

Arguments:
  <WORDS>...  anchors or filter words, and labels as <type>=<id>, e.g. `org:roast month:2025-09 project=harvest`

Options:
      --expect-store-rev <N>  the store_rev you last read (`classify queue --json`); a write is refused with view_changed (exit 4) if the store moved
      --client <NAME>         your name: the write is recorded as agent:<NAME>
      --caption <CAPTION>     one line (at most 120 characters) shown with the write and in the detail pane
      --allow-bulk            allow more than 50 lines in one write
      --overwrite             also replace labels a human confirmed (they are skipped otherwise)
      --session <SESSION>     send through this running TUI (default: the TUI running this profile, if any)
      --direct                write the store directly even when a TUI is running
```

### ownpurse classify note

Add a note to a line or a group (agent write)

```text
Add a note to a line or a group (agent write)

Usage: ownpurse classify note [OPTIONS] <TARGET> <TEXT>

Arguments:
  <TARGET>  an anchor or a group id (g12)
  <TEXT>    the note

Options:
      --expect-store-rev <N>  the store_rev you last read (`classify queue --json`); a write is refused with view_changed (exit 4) if the store moved
      --client <NAME>         your name: the write is recorded as agent:<NAME>
      --caption <CAPTION>     one line (at most 120 characters) shown with the write and in the detail pane
      --allow-bulk            allow more than 50 lines in one write
      --overwrite             also replace labels a human confirmed (they are skipped otherwise)
      --session <SESSION>     send through this running TUI (default: the TUI running this profile, if any)
      --direct                write the store directly even when a TUI is running
```

### ownpurse classify group

Groups (agent writes)

```text
Groups (agent writes)

Usage: ownpurse classify group [OPTIONS] <COMMAND>

Commands:
  create  Create a group (agent write): its kind (from [groups].kinds) and a title
  remove  Remove lines from a group (agent write)
  status  Set a group's status (agent write): open, complete, gap or accountant
  add     Add lines to a group (agent write)
  help    Print this message or the help of the given subcommand(s)
```

#### ownpurse classify group create

Create a group (agent write): its kind (from [groups].kinds) and a title

```text
Create a group (agent write): its kind (from [groups].kinds) and a title

Usage: ownpurse classify group create [OPTIONS] <GROUP_KIND> <TITLE>

Arguments:
  <GROUP_KIND>  the group kind, e.g. split, intercompany_chain
  <TITLE>       the title

Options:
      --expect-store-rev <N>  the store_rev you last read (`classify queue --json`); a write is refused with view_changed (exit 4) if the store moved
      --client <NAME>         your name: the write is recorded as agent:<NAME>
      --caption <CAPTION>     one line (at most 120 characters) shown with the write and in the detail pane
      --allow-bulk            allow more than 50 lines in one write
      --overwrite             also replace labels a human confirmed (they are skipped otherwise)
      --session <SESSION>     send through this running TUI (default: the TUI running this profile, if any)
      --direct                write the store directly even when a TUI is running
```

#### ownpurse classify group remove

Remove lines from a group (agent write)

```text
Remove lines from a group (agent write)

Usage: ownpurse classify group remove [OPTIONS] <GROUP> <ANCHORS>...

Arguments:
  <GROUP>       the group id, e.g. g12
  <ANCHORS>...  the lines (anchors)

Options:
      --expect-store-rev <N>  the store_rev you last read (`classify queue --json`); a write is refused with view_changed (exit 4) if the store moved
      --client <NAME>         your name: the write is recorded as agent:<NAME>
      --caption <CAPTION>     one line (at most 120 characters) shown with the write and in the detail pane
      --allow-bulk            allow more than 50 lines in one write
      --overwrite             also replace labels a human confirmed (they are skipped otherwise)
      --session <SESSION>     send through this running TUI (default: the TUI running this profile, if any)
      --direct                write the store directly even when a TUI is running
```

#### ownpurse classify group status

Set a group's status (agent write): open, complete, gap or accountant

```text
Set a group's status (agent write): open, complete, gap or accountant

Usage: ownpurse classify group status [OPTIONS] <GROUP> <STATUS>

Arguments:
  <GROUP>   the group id, e.g. g12
  <STATUS>  open, complete, gap or accountant

Options:
      --expect-store-rev <N>  the store_rev you last read (`classify queue --json`); a write is refused with view_changed (exit 4) if the store moved
      --client <NAME>         your name: the write is recorded as agent:<NAME>
      --caption <CAPTION>     one line (at most 120 characters) shown with the write and in the detail pane
      --allow-bulk            allow more than 50 lines in one write
      --overwrite             also replace labels a human confirmed (they are skipped otherwise)
      --session <SESSION>     send through this running TUI (default: the TUI running this profile, if any)
      --direct                write the store directly even when a TUI is running
```

#### ownpurse classify group add

Add lines to a group (agent write)

```text
Add lines to a group (agent write)

Usage: ownpurse classify group add [OPTIONS] <GROUP> <ANCHORS>...

Arguments:
  <GROUP>       the group id, e.g. g12
  <ANCHORS>...  the lines (anchors)

Options:
      --role <ROLE>           origin, mirror or settlement
      --expect-store-rev <N>  the store_rev you last read (`classify queue --json`); a write is refused with view_changed (exit 4) if the store moved
      --client <NAME>         your name: the write is recorded as agent:<NAME>
      --caption <CAPTION>     one line (at most 120 characters) shown with the write and in the detail pane
      --allow-bulk            allow more than 50 lines in one write
      --overwrite             also replace labels a human confirmed (they are skipped otherwise)
      --session <SESSION>     send through this running TUI (default: the TUI running this profile, if any)
      --direct                write the store directly even when a TUI is running
```

### ownpurse classify undo

Undo an agent write: --batch (the latest agent batch), an anchor (the last agent write on it) or --events

```text
Undo an agent write: --batch (the latest agent batch), an anchor (the last agent write on it) or --events

Usage: ownpurse classify undo [OPTIONS] [ANCHOR]

Arguments:
  [ANCHOR]  the line whose last agent write to undo

Options:
      --batch                 the latest agent batch
      --events <EVENTS>       these event ids (agent events only), comma-separated
      --expect-store-rev <N>  the store_rev you last read (`classify queue --json`); a write is refused with view_changed (exit 4) if the store moved
      --client <NAME>         your name: the write is recorded as agent:<NAME>
      --caption <CAPTION>     one line (at most 120 characters) shown with the write and in the detail pane
      --allow-bulk            allow more than 50 lines in one write
      --overwrite             also replace labels a human confirmed (they are skipped otherwise)
      --session <SESSION>     send through this running TUI (default: the TUI running this profile, if any)
      --direct                write the store directly even when a TUI is running
```

### ownpurse classify config

Edit classify.toml (agent writes)

```text
Edit classify.toml (agent writes)

Usage: ownpurse classify config [OPTIONS] <COMMAND>

Commands:
  add-project  Add a project to classify.toml (agent write; --dry-run shows the diff, --confirm writes it)
  help         Print this message or the help of the given subcommand(s)
```

#### ownpurse classify config add-project

Add a project to classify.toml (agent write; --dry-run shows the diff, --confirm writes it)

```text
Add a project to classify.toml (agent write; --dry-run shows the diff, --confirm writes it)

Usage: ownpurse classify config add-project [OPTIONS] <ID> <NAME>

Arguments:
  <ID>    the new project id (letters, digits, - _ .)
  <NAME>  its display name

Options:
      --alias <ALIASES>       an alias rules match in descriptions (repeatable)
      --dry-run               show the diff only
      --confirm               write it (required for a write)
      --expect-store-rev <N>  the store_rev you last read (`classify queue --json`); a write is refused with view_changed (exit 4) if the store moved
      --client <NAME>         your name: the write is recorded as agent:<NAME>
      --caption <CAPTION>     one line (at most 120 characters) shown with the write and in the detail pane
      --allow-bulk            allow more than 50 lines in one write
      --overwrite             also replace labels a human confirmed (they are skipped otherwise)
      --session <SESSION>     send through this running TUI (default: the TUI running this profile, if any)
      --direct                write the store directly even when a TUI is running
```

### ownpurse classify suggest

Run the rules over the lines in the filter and store their suggestions (local store only; nothing is confirmed, nothing goes to Xero)

```text
Run the rules over the lines in the filter and store their suggestions (local store only; nothing is confirmed, nothing goes to Xero)

Usage: ownpurse classify suggest [OPTIONS] [FILTER]...

Arguments:
  [FILTER]...  queue filter, e.g. `org:roast month:2025-09 status:all` (default: open lines)

Options:
      --dry-run            report what would be written (per rule: lines matched, suggestions per label type); append nothing
      --engine <ENGINE>    `rules` (Jev is not enabled in this release: `jev` refuses with jev_not_enabled, nothing is sent) [default: rules] [possible values: rules, jev]
```

### ownpurse classify rules

Check the classification config

```text
Check the classification config

Usage: ownpurse classify rules [OPTIONS] <COMMAND>

Commands:
  check  Validate classify.toml: exit 0 when valid, 1 with every problem as `file:line: path: reason`
  help   Print this message or the help of the given subcommand(s)
```

#### ownpurse classify rules check

Validate classify.toml: exit 0 when valid, 1 with every problem as `file:line: path: reason`

```text
Validate classify.toml: exit 0 when valid, 1 with every problem as `file:line: path: reason`

Usage: ownpurse classify rules check [OPTIONS]
```

### ownpurse classify export

One CSV row per line: Xero facts, each label and its source, stale, notes (default: every line)

```text
One CSV row per line: Xero facts, each label and its source, stale, notes (default: every line)

Usage: ownpurse classify export [OPTIONS] [FILTER]...

Arguments:
  [FILTER]...  queue filter, e.g. `org:design project:refit`

Options:
      --csv                CSV on stdout (the only format)
```

### ownpurse classify queue

The classification queue as data: lines without confirmed required labels, or with stale ones, largest first

```text
The classification queue as data: lines without confirmed required labels, or with stale ones, largest first

Usage: ownpurse classify queue [OPTIONS] [FILTER]...

Arguments:
  [FILTER]...  queue filter: `org:<alias> acct:<prefix> month:YYYY-MM year:YYYY project:<id> conf<0.7 by:agent since:today status:open|all|stale` and words

Options:
      --sort <SORT>        `stale` (stale lines first, then largest; the default), `amount` (largest first) or `date` [possible values: stale, amount, date]
```

## ownpurse ui

The agent bridge: read and change a running TUI's view (view only; local socket)

```text
The agent bridge: read and change a running TUI's view (view only; local socket)

Usage: ownpurse ui [OPTIONS] <COMMAND>

Commands:
  ls       List the running TUIs an agent can talk to (stale entries are pruned)
  state    The view state of a running TUI as data: tab, scope, as-of, basis, filter, the selected and visible rows (anchors), strip facts, size, `rev`
  screen   The frame the TUI shows now (text, ansi, html, svg or png), with the `rev` it belongs to
  send     Change the view (view verbs only): tab <n|name>, scope <alias|group:x|all>, asof <date>, filter <text>, clear-filter, select <#n|anchor|text>, drill, up (one level, as Esc), basis year|history, differs on|off, compare <org,org…>, reload, open (the document on screen in Xero; a link, no call); cosmetic (never refused while the user types): theme <name>, accent <family|#hex>, overlay set <token=value…> | overlay clear [token…] (last)
  wait     Wait until the TUI's state meets a condition (answered by the TUI when it holds; no polling): idle, running=none, rev>N, store_rev>N, record>N (each also >=N); `timeout` (exit 4) with the current values otherwise
  upgrade  Upgrade a running TUI in place to the newer installed ownpurse-tui, view restored exactly (the one non-view action an agent may send; no prompt): upgraded, deferred (a sync runs), up_to_date, preflight_failed or exec_failed
  flash    Pulse a running TUI's header to get the user's attention: the CLI pushes overlay states and restores the prior overlay after (always, Ctrl-C included); at most three flashes a second (too_fast)
  help     Print this message or the help of the given subcommand(s)
```

### ownpurse ui ls

List the running TUIs an agent can talk to (stale entries are pruned)

```text
List the running TUIs an agent can talk to (stale entries are pruned)

Usage: ownpurse ui ls [OPTIONS]
```

### ownpurse ui state

The view state of a running TUI as data: tab, scope, as-of, basis, filter, the selected and visible rows (anchors), strip facts, size, `rev`

```text
The view state of a running TUI as data: tab, scope, as-of, basis, filter, the selected and visible rows (anchors), strip facts, size, `rev`

Usage: ownpurse ui state [OPTIONS]

Options:
      --session <SESSION>  which TUI: an id, pid, label or `scope:<alias>` (required when several run; `ownpurse ui ls`)
      --all-rows           every row of the screen's list, not just those on screen (read-only: the view doesn't move)
      --rows <A..B>        rows A..B of the list (1-based, the `#n` of `select`; read-only)
      --fields <PATHS>     only these fields, comma-separated dotted paths (e.g. `tab,breadcrumb,view,selected.account_code`)
```

### ownpurse ui screen

The frame the TUI shows now (text, ansi, html, svg or png), with the `rev` it belongs to

```text
The frame the TUI shows now (text, ansi, html, svg or png), with the `rev` it belongs to

Usage: ownpurse ui screen [OPTIONS]

Options:
      --format <FORMAT>       output format [default: text] [possible values: text, ansi, html, svg, png]
      --out <OUT>             write to this file (required for png)
      --session <SESSION>     which TUI: an id, pid, label or `scope:<alias>` (required when several run)
      --from-state <FILE>     render a saved `ui state` instead of a live TUI (no session needed; runs ownpurse-tui --state)
      --allow-record-drift    with --from-state: render even if the record changed since the state was saved (marked drifted)
      --allow-config-drift    with --from-state: render even if the config changed beyond a reorder (marked drifted)
      --allow-classify-drift  with --from-state: render even if labels changed since the state was saved (marked drifted)
```

### ownpurse ui send

Change the view (view verbs only): tab <n|name>, scope <alias|group:x|all>, asof <date>, filter <text>, clear-filter, select <#n|anchor|text>, drill, up (one level, as Esc), basis year|history, differs on|off, compare <org,org…>, reload, open (the document on screen in Xero; a link, no call); cosmetic (never refused while the user types): theme <name>, accent <family|#hex>, overlay set <token=value…> | overlay clear [token…] (last)

```text
Change the view (view verbs only): tab <n|name>, scope <alias|group:x|all>, asof <date>, filter <text>, clear-filter, select <#n|anchor|text>, drill, up (one level, as Esc), basis year|history, differs on|off, compare <org,org…>, reload, open (the document on screen in Xero; a link, no call); cosmetic (never refused while the user types): theme <name>, accent <family|#hex>, overlay set <token=value…> | overlay clear [token…] (last)

Usage: ownpurse ui send [OPTIONS] <VERBS>...

Arguments:
  <VERBS>...  verbs and their arguments, e.g. `tab 2 scope design`; or with --json one message: '{"verb": "tab", "args": ["2"]}'

Options:
      --expect-rev <EXPECT_REV>  refuse (exit 4, view_changed) unless the view is still at this rev (from `ui state`) [alias: --expect-seq]
      --expect <PRED>            refuse (exit 4, view_mismatch) unless the view matches: PATH=VALUE, PATH!=VALUE or PATH~TEXT over `ui state` fields (repeatable, AND-ed), e.g. `--expect tab=ledger --expect breadcrumb~"Trading Clearing"`
      --screen <SCREEN>          also return the new frame in this format [possible values: text, ansi, html, svg, png]
      --out <OUT>                write the frame (--screen) to this file (required for png)
      --json-out                 machine-readable output (as --json; use it when --json carries a message)
      --caption <CAPTION>        also set the footer caption, in the same update (atomic: a refused verb applies nothing)
      --client <CLIENT>          the client name captions show as their author (default `agent`)
      --state                    print the whole view state (text) instead of the compact summary
      --session <SESSION>        which TUI: an id, pid, label or `scope:<alias>` (required when several run)
```

### ownpurse ui wait

Wait until the TUI's state meets a condition (answered by the TUI when it holds; no polling): idle, running=none, rev>N, store_rev>N, record>N (each also >=N); `timeout` (exit 4) with the current values otherwise

```text
Wait until the TUI's state meets a condition (answered by the TUI when it holds; no polling): idle, running=none, rev>N, store_rev>N, record>N (each also >=N); `timeout` (exit 4) with the current values otherwise

Usage: ownpurse ui wait [OPTIONS] --until <UNTIL>

Options:
      --until <UNTIL>      the condition, e.g. `idle` or `rev>12`
      --timeout <TIMEOUT>  seconds to wait at most (default 30, max 600) [default: 30]
      --session <SESSION>  which TUI: an id, pid, label or `scope:<alias>` (required when several run)
```

### ownpurse ui upgrade

Upgrade a running TUI in place to the newer installed ownpurse-tui, view restored exactly (the one non-view action an agent may send; no prompt): upgraded, deferred (a sync runs), up_to_date, preflight_failed or exec_failed

```text
Upgrade a running TUI in place to the newer installed ownpurse-tui, view restored exactly (the one non-view action an agent may send; no prompt): upgraded, deferred (a sync runs), up_to_date, preflight_failed or exec_failed

Usage: ownpurse ui upgrade [OPTIONS]

Options:
      --session <SESSION>  which TUI: an id, pid, label or `scope:<alias>` (required when several run)
```

### ownpurse ui flash

Pulse a running TUI's header to get the user's attention: the CLI pushes overlay states and restores the prior overlay after (always, Ctrl-C included); at most three flashes a second (too_fast)

```text
Pulse a running TUI's header to get the user's attention: the CLI pushes overlay states and restores the prior overlay after (always, Ctrl-C included); at most three flashes a second (too_fast)

Usage: ownpurse ui flash [OPTIONS]

Options:
      --session <SESSION>    which TUI: an id, pid, label or `scope:<alias>` (required when several run)
      --times <TIMES>        how many flashes (1–5) [default: 3]
      --on <ON>              how long each flash shows (200ms, 0.3s); --on + --off must be at least 334 ms [default: 200ms]
      --off <OFF>            the pause after each flash [default: 200ms]
      --token <TOKEN=VALUE>  the overlay to flash, token=value (repeatable; default: the header band at 3× its tint)
      --accent <ACCENT>      flash this accent family instead
```

## ownpurse write

Xero changes as a sealed, approved package (plan, seal, approve; the run side is separate)

```text
Xero changes as a sealed, approved package (plan, seal, approve; the run side is separate)

Usage: ownpurse write [OPTIONS] <COMMAND>

Commands:
  plan            Compile changes (ops.json) against the local record into an unsealed package (no Xero call)
  seal            Seal a package (its SHA-256), write the review PDF and CSV for the accountant, print the review
  export          Re-write the review PDF and CSV of a sealed package
  approve-record  The owner records his approval at his own terminal (refused in agent sessions)
  approve-import  Import the owner's typed chat approval (quoting the full hash) from his own session transcript
  approve-check   Whether an approval is recorded for the package's hash (exit 0, or 3)
  approve-demo    Record the owner's standing Demo Company approval for this package (demo profiles only; agents may run it)
  run             ⇅ Execute the next batch (at most 10) of an approved package: needs allow_writes, OWNPURSE_WRITE_RUN=<hash> and a recorded approval
  verify          ⇅ Re-read each landed target and the trial balance movement against the expected deltas (exit 5 on a discrepancy)
  reconcile       ⇅ Settle an uncertain attempt by read-back (never by re-sending)
  recover         ⇅ Build the recovery package for what landed (unsealed; it needs its own approval)
  status          The attempt log of a package and the record's chain (no Xero call)
  probe           Milestone 1: write a probe's plan input from the synced Demo Company record (Demo Company profiles only)
  probe-replay    ⇅ Milestone 1, Demo Company only: re-send a landed journal under its own Idempotency-Key (the op must be flagged probe_replay)
  help            Print this message or the help of the given subcommand(s)
```

### ownpurse write plan

Compile changes (ops.json) against the local record into an unsealed package (no Xero call)

```text
Compile changes (ops.json) against the local record into an unsealed package (no Xero call)

Usage: ownpurse write plan [OPTIONS] <INPUT>

Arguments:
  <INPUT>  the requested operations (JSON)

Options:
      --out <OUT>          where to write the package (default: `<input>.package.json` beside it)
```

### ownpurse write seal

Seal a package (its SHA-256), write the review PDF and CSV for the accountant, print the review

```text
Seal a package (its SHA-256), write the review PDF and CSV for the accountant, print the review

Usage: ownpurse write seal [OPTIONS] <PACKAGE>

Arguments:
  <PACKAGE>

Options:
      --dir <DIR>          folder for the PDF and CSV (default: beside the package)
```

### ownpurse write export

Re-write the review PDF and CSV of a sealed package

```text
Re-write the review PDF and CSV of a sealed package

Usage: ownpurse write export [OPTIONS] <PACKAGE>

Arguments:
  <PACKAGE>

Options:
      --dir <DIR>
```

### ownpurse write approve-record

The owner records his approval at his own terminal (refused in agent sessions)

```text
The owner records his approval at his own terminal (refused in agent sessions)

Usage: ownpurse write approve-record [OPTIONS] --response-file <RESPONSE_FILE> <PACKAGE>

Arguments:
  <PACKAGE>

Options:
      --response-file <RESPONSE_FILE>  a file with the owner's response, verbatim (it must quote the hash)
      --i-am-the-owner                 required: you are the owner
      --scope <SCOPE>                  op ids it covers, comma-separated (default: all)
```

### ownpurse write approve-import

Import the owner's typed chat approval (quoting the full hash) from his own session transcript

```text
Import the owner's typed chat approval (quoting the full hash) from his own session transcript

Usage: ownpurse write approve-import [OPTIONS] --transcript <TRANSCRIPT> --session <SESSION> <PACKAGE>

Arguments:
  <PACKAGE>

Options:
      --transcript <TRANSCRIPT>  the session's transcript (`<session id>.jsonl`)
      --session <SESSION>        the session id
      --scope <SCOPE>
```

### ownpurse write approve-check

Whether an approval is recorded for the package's hash (exit 0, or 3)

```text
Whether an approval is recorded for the package's hash (exit 0, or 3)

Usage: ownpurse write approve-check [OPTIONS] <PACKAGE>

Arguments:
  <PACKAGE>
```

### ownpurse write approve-demo

Record the owner's standing Demo Company approval for this package (demo profiles only; agents may run it)

```text
Record the owner's standing Demo Company approval for this package (demo profiles only; agents may run it)

Usage: ownpurse write approve-demo [OPTIONS] <PACKAGE>

Arguments:
  <PACKAGE>
```

### ownpurse write run

⇅ Execute the next batch (at most 10) of an approved package: needs allow_writes, OWNPURSE_WRITE_RUN=<hash> and a recorded approval

```text
⇅ Execute the next batch (at most 10) of an approved package: needs allow_writes, OWNPURSE_WRITE_RUN=<hash> and a recorded approval

Usage: ownpurse write run [OPTIONS] <PACKAGE>

Arguments:
  <PACKAGE>

Options:
      --batch <BATCH>      operations in this batch (1–10, default 10)
      --dry-run            check preconditions live and print each request; send nothing
```

### ownpurse write verify

⇅ Re-read each landed target and the trial balance movement against the expected deltas (exit 5 on a discrepancy)

```text
⇅ Re-read each landed target and the trial balance movement against the expected deltas (exit 5 on a discrepancy)

Usage: ownpurse write verify [OPTIONS] <PACKAGE>

Arguments:
  <PACKAGE>
```

### ownpurse write reconcile

⇅ Settle an uncertain attempt by read-back (never by re-sending)

```text
⇅ Settle an uncertain attempt by read-back (never by re-sending)

Usage: ownpurse write reconcile [OPTIONS] <PACKAGE> <OP>

Arguments:
  <PACKAGE>
  <OP>
```

### ownpurse write recover

⇅ Build the recovery package for what landed (unsealed; it needs its own approval)

```text
⇅ Build the recovery package for what landed (unsealed; it needs its own approval)

Usage: ownpurse write recover [OPTIONS] <PACKAGE> [OP]

Arguments:
  <PACKAGE>
  [OP]       one operation (default: every landed one)

Options:
      --out <OUT>
```

### ownpurse write status

The attempt log of a package and the record's chain (no Xero call)

```text
The attempt log of a package and the record's chain (no Xero call)

Usage: ownpurse write status [OPTIONS] <PACKAGE>

Arguments:
  <PACKAGE>
```

### ownpurse write probe

Milestone 1: write a probe's plan input from the synced Demo Company record (Demo Company profiles only)

```text
Milestone 1: write a probe's plan input from the synced Demo Company record (Demo Company profiles only)

Usage: ownpurse write probe [OPTIONS] <NAME>

Arguments:
  <NAME>  recode-reconciled | void-after-attachment | idempotency | line-limit

Options:
      --org <ORG>
      --lines <LINES>      line-limit: the journal's line count (default 100)
      --out <OUT>          folder for the inputs (default: probes)
```

### ownpurse write probe-replay

⇅ Milestone 1, Demo Company only: re-send a landed journal under its own Idempotency-Key (the op must be flagged probe_replay)

```text
⇅ Milestone 1, Demo Company only: re-send a landed journal under its own Idempotency-Key (the op must be flagged probe_replay)

Usage: ownpurse write probe-replay [OPTIONS] <PACKAGE> <OP>

Arguments:
  <PACKAGE>
  <OP>

Options:
      --changed            change the narration (the marker kept) under the same key
```

## ownpurse doctor

Check the setup (binaries on PATH, config, record, token file permissions); read-only, prints fixes

```text
Check the setup (binaries on PATH, config, record, token file permissions); read-only, prints fixes

Usage: ownpurse doctor [OPTIONS]
```

## ownpurse sync

⇅ Mirror entities into the local record (calls Xero)

```text
⇅ Mirror entities into the local record (calls Xero)

Usage: ownpurse sync [OPTIONS] <ORG> [ENTITIES]...

Arguments:
  <ORG>          org alias, tenant id, `group:<name>`, comma list, or `all`
  [ENTITIES]...  entities to sync (default: all), e.g. Accounts BankTransactions

Options:
      --reference          settings tables only
      --transactions       transaction tables only
```

## ownpurse login

⇅ Browser consent (PKCE): read access plus anything already granted; one consent covers every org

```text
⇅ Browser consent (PKCE): read access plus anything already granted; one consent covers every org

Usage: ownpurse login [OPTIONS]

Options:
      --timeout <TIMEOUT>  seconds to wait for consent (default: config)
      --no-browser         print the URL only
      --write              also request the write scopes (Phase 8; only for a profile with allow_writes)
```

## ownpurse orgs

⇅ List connected organisations (1 call; `--offline` reads the token file's last list)

```text
⇅ List connected organisations (1 call; `--offline` reads the token file's last list)

Usage: ownpurse orgs [OPTIONS]
```

## ownpurse accounts

Chart of accounts (one org or all)

```text
Chart of accounts (one org or all)

Usage: ownpurse accounts [OPTIONS] <ORG>

Arguments:
  <ORG>  org alias, tenant id, `group:<name>`, comma list, or `all`

Options:
      --bank               bank accounts only
      --type <TYPE>        account type, e.g. EXPENSE, BANK, CURRLIAB
      --search <SEARCH>    substring of name, contact, reference or description
      --archived           include archived
```

## ownpurse org

Organisation details: plan class, lock dates

```text
Organisation details: plan class, lock dates

Usage: ownpurse org [OPTIONS] <ORG>

Arguments:
  <ORG>  org alias or tenant id
```

## ownpurse contacts

Contacts

```text
Contacts

Usage: ownpurse contacts [OPTIONS] <ORG> [SEARCH]

Arguments:
  <ORG>     org alias, tenant id, `group:<name>`, comma list, or `all`
  [SEARCH]  name contains

Options:
      --archived           include archived
```

## ownpurse txns

Bank transactions (spend/receive money)

```text
Bank transactions (spend/receive money)

Usage: ownpurse txns [OPTIONS] <ORG>

Arguments:
  <ORG>  org alias, tenant id, `group:<name>`, comma list, or `all`

Options:
      --from <DATE>        from date (YYYY-MM-DD, inclusive)
      --to <DATE>          to date (YYYY-MM-DD, inclusive)
      --account <ACCOUNT>  bank account code or name substring
      --status <STATUS>    document status, e.g. AUTHORISED, POSTED, DRAFT
      --search <SEARCH>    substring anywhere in the record
      --deleted            include DELETED
```

## ownpurse journals

Manual journals

```text
Manual journals

Usage: ownpurse journals [OPTIONS] <ORG>

Arguments:
  <ORG>  org alias, tenant id, `group:<name>`, comma list, or `all`

Options:
      --from <DATE>        from date (YYYY-MM-DD, inclusive)
      --to <DATE>          to date (YYYY-MM-DD, inclusive)
      --status <STATUS>    document status, e.g. AUTHORISED, POSTED, DRAFT
      --search <SEARCH>    substring of name, contact, reference or description
```

## ownpurse transfers

Bank transfers

```text
Bank transfers

Usage: ownpurse transfers [OPTIONS] <ORG>

Arguments:
  <ORG>  org alias, tenant id, `group:<name>`, comma list, or `all`

Options:
      --from <DATE>        from date (YYYY-MM-DD, inclusive)
      --to <DATE>          to date (YYYY-MM-DD, inclusive)
```

## ownpurse show

One record: current payload and version history

```text
One record: current payload and version history

Usage: ownpurse show [OPTIONS] <ORG> <ENTITY> <ID>

Arguments:
  <ORG>     org alias, tenant id, `group:<name>`, comma list, or `all`
  <ENTITY>  entity name, e.g. BankTransactions, ManualJournals, Invoices
  <ID>      id or unique prefix
```

## ownpurse open

The document's page in the Xero web app, through the org switch (local; no Xero call)

```text
The document's page in the Xero web app, through the org switch (local; no Xero call)

Usage: ownpurse open [OPTIONS] <ORG> <ENTITY> <ID>

Arguments:
  <ORG>     org alias or tenant id
  <ENTITY>  BankTransactions, ManualJournals, Invoices, or BankRec (a bank account's reconcile queue)
  <ID>      id or unique prefix (BankRec: the bank account's AccountID)

Options:
      --launch             also open it in the browser (macOS `open`)
```

## ownpurse list

List any mirrored entity (Invoices, Payments, CreditNotes, TaxRates, …)

```text
List any mirrored entity (Invoices, Payments, CreditNotes, TaxRates, …)

Usage: ownpurse list [OPTIONS] <ORG> <ENTITY>

Arguments:
  <ORG>     org alias, tenant id, `group:<name>`, comma list, or `all`
  <ENTITY>  entity name, e.g. BankTransactions, ManualJournals, Invoices

Options:
      --from <DATE>        from date (YYYY-MM-DD, inclusive)
      --to <DATE>          to date (YYYY-MM-DD, inclusive)
      --status <STATUS>    document status, e.g. AUTHORISED, POSTED, DRAFT
      --search <SEARCH>    substring anywhere in the record
      --fields <FIELDS>    comma list of payload paths, e.g. Contact.Name,Total
```

## ownpurse gl

Local general ledger: postings derived from mirrored documents

```text
Local general ledger: postings derived from mirrored documents

Usage: ownpurse gl [OPTIONS] <ORG>

Arguments:
  <ORG>  org alias, tenant id, `group:<name>`, comma list, or `all`

Options:
      --from <DATE>        from date (YYYY-MM-DD, inclusive)
      --to <DATE>          to date (YYYY-MM-DD, inclusive)
      --account <ACCOUNT>  code, name, or id (substring match if no exact match)
      --full-history       running balance from local documents only (default opens from the cached Xero TB)
      --source <SOURCE>    BankTransactions | BankTransfers | ManualJournals | Invoices | ...
      --search <SEARCH>    substring of name, contact, reference or description
```

## ownpurse tb

Local trial balance computed from the ledger (exact decimals)

```text
Local trial balance computed from the ledger (exact decimals)

Usage: ownpurse tb [OPTIONS] --date <DATE> <ORG>

Arguments:
  <ORG>  org alias, tenant id, `group:<name>`, comma list, or `all`

Options:
      --date <DATE>        as-of date (YYYY-MM-DD)
      --since <SINCE>      open from Xero's cached TB at this date, add later activity
      --full-history       balances from local documents only (default: cached Xero TB at prior year-end + activity)
      --compare            2–4 orgs side by side (rows merge only by code / map)
      --align <ALIGN>      with --compare: code (default) or account_map category [possible values: code, map]
```

## ownpurse pl

Profit and loss from the local ledger (accrual or --cash), checked against Xero's cached report

```text
Profit and loss from the local ledger (accrual or --cash), checked against Xero's cached report

Usage: ownpurse pl [OPTIONS] --to <TO> <ORG>

Arguments:
  <ORG>  org alias or tenant id

Options:
      --from <FROM>        period start (default: the fiscal year's start)
      --to <TO>            period end (YYYY-MM-DD)
      --cash               cash basis (Xero's paymentsOnly)
      --strict             exit 1 when anything differs from Xero's cached report
```

## ownpurse bs

Balance sheet from the local ledger (accrual or --cash), checked against Xero's cached report

```text
Balance sheet from the local ledger (accrual or --cash), checked against Xero's cached report

Usage: ownpurse bs [OPTIONS] --date <DATE> <ORG>

Arguments:
  <ORG>  org alias or tenant id

Options:
      --date <DATE>        as-of date (YYYY-MM-DD)
      --cash               cash basis (Xero's paymentsOnly)
      --strict             exit 1 when anything differs from Xero's cached report
      --since <SINCE>      open from Xero's cached balance sheet at this date (default: the prior fiscal year-end, when cached)
      --full-history       balances from local documents only
```

## ownpurse parity

Compare the local trial balance with Xero's cached one, account by account

```text
Compare the local trial balance with Xero's cached one, account by account

Usage: ownpurse parity [OPTIONS] --date <DATE> <ORG>

Arguments:
  <ORG>  org alias, tenant id, `group:<name>`, comma list, or `all`

Options:
      --date <DATE>        as-of date (YYYY-MM-DD)
      --since <SINCE>      open from Xero's cached TB at this date
      --cached             reuse a cached Xero TB snapshot (always the case here: nothing is fetched)
      --strict             exit 1 when anything differs
```

## ownpurse report

Xero's reports: fetch one into the record (⇅ one call per org), or list what the record holds

```text
Xero's reports: fetch one into the record (⇅ one call per org), or list what the record holds

Usage: ownpurse report [OPTIONS] <COMMAND>

Commands:
  fetch  ⇅ Fetch a report into the record (TrialBalance, BalanceSheet, ProfitAndLoss, BankSummary)
  list   The reports the record holds (zero calls)
  help   Print this message or the help of the given subcommand(s)
```

### ownpurse report fetch

⇅ Fetch a report into the record (TrialBalance, BalanceSheet, ProfitAndLoss, BankSummary)

```text
⇅ Fetch a report into the record (TrialBalance, BalanceSheet, ProfitAndLoss, BankSummary)

Usage: ownpurse report fetch [OPTIONS] <ORG> <KIND>

Arguments:
  <ORG>   org alias, tenant id, `group:<name>`, comma list, or `all`
  <KIND>  trialbalance | balancesheet | pl | banksummary

Options:
      --date <DATE>        as-of date for trialbalance / balancesheet (default: the last fiscal year-end)
      --from <FROM>        period start for pl / banksummary (default: that fiscal year's start)
      --to <TO>            period end for pl / banksummary (default: the last fiscal year-end)
      --cash               cash basis (paymentsOnly)
```

### ownpurse report list

The reports the record holds (zero calls)

```text
The reports the record holds (zero calls)

Usage: ownpurse report list [OPTIONS] <ORG> [KIND]

Arguments:
  <ORG>   org alias, tenant id, `group:<name>`, comma list, or `all`
  [KIND]  only this report (trialbalance, balancesheet, pl, banksummary)
```

## ownpurse export

A file for the accountant from the record (zero calls): tb / gl from the local ledger, bs / pl from Xero's cached reports

```text
A file for the accountant from the record (zero calls): tb / gl from the local ledger, bs / pl from Xero's cached reports

Usage: ownpurse export [OPTIONS] <ORG> <KIND>

Arguments:
  <ORG>   org alias or tenant id
  <KIND>  tb | gl | bs | pl

Options:
      --date <DATE>        as-of date for tb / bs (default: the last fiscal year-end)
      --from <FROM>        period start for gl / pl (default: that fiscal year's start)
      --to <TO>            period end for gl / pl
      --csv                CSV (the default)
      --xlsx               a spreadsheet (not yet: use --csv)
      --out <OUT>          the file to write (default `<org>-<kind>-<date>.csv` here; never over an existing file)
```

## ownpurse attachments

⇅ A document's attachments from Xero, into the record; `--get` saves one file (`--cached`: the record)

```text
⇅ A document's attachments from Xero, into the record; `--get` saves one file (`--cached`: the record)

Usage: ownpurse attachments [OPTIONS] <ORG> <ENTITY> <ID>

Arguments:
  <ORG>     org alias or tenant id
  <ENTITY>  BankTransactions, ManualJournals, Invoices, Contacts, …
  <ID>      the document's id

Options:
      --cached             what the record last saw (zero calls)
      --get <GET>          save this attachment's bytes (by file name)
      --out <OUT>          where to save it (default: the file name here; never over an existing file)
```

## ownpurse history

⇅ A document's history and notes from Xero, into the record (`--cached`: what the record holds)

```text
⇅ A document's history and notes from Xero, into the record (`--cached`: what the record holds)

Usage: ownpurse history [OPTIONS] <ORG> <ENTITY> <ID>

Arguments:
  <ORG>     org alias or tenant id
  <ENTITY>  BankTransactions, ManualJournals, Invoices, Contacts, BankTransfers, Payments, …
  <ID>      the document's id

Options:
      --cached             what the record last saw (zero calls)
```

## ownpurse usage

API usage: calls, quotas, endpoints, records fetched

```text
API usage: calls, quotas, endpoints, records fetched

Usage: ownpurse usage [OPTIONS]

Options:
      --since <SINCE>      window start (default: today, UTC)
      --days <DAYS>        window of the last N days
      --calls <CALLS>      recent calls to list [default: 20]
```

## ownpurse config

Effective settings with their source; links; edit

```text
Effective settings with their source; links; edit

Usage: ownpurse config [OPTIONS] [ACTION]

Arguments:
  [ACTION]  show (effective settings + origin), path, links, edit [default: show] [possible values: show, path, links, edit]
```

## ownpurse ic

Intercompany pairs from config: both sides, difference, bridge, residual (zero calls)

```text
Intercompany pairs from config: both sides, difference, bridge, residual (zero calls)

Usage: ownpurse ic [OPTIONS] [PAIR]

Arguments:
  [PAIR]  pair name from `intercompany.pairs` (default: every pair)

Options:
      --as-of <DATE>       as-of date (default: the scoped org's last fiscal year-end)
```

## ownpurse viz

Explainer models (bridge, flow, t_account, timeline, before_after, checklist): the numbers the TUI's panels show (zero calls)

```text
Explainer models (bridge, flow, t_account, timeline, before_after, checklist): the numbers the TUI's panels show (zero calls)

Usage: ownpurse viz [OPTIONS] <KIND> [TARGET] [ACCOUNT]

Arguments:
  <KIND>     bridge | flow | t_account | timeline | before_after | checklist [possible values: bridge, flow, t_account, timeline, before_after, checklist]
  [TARGET]   bridge: the pair id; t_account, timeline, before_after: the org; checklist: the file (relative to the config's folder)
  [ACCOUNT]  t_account, timeline: the account code

Options:
      --as-of <DATE>       the date (bridge, flow owed; the period's end otherwise). Default: the first org's last fiscal year-end
      --from <DATE>        the period's first day (default: the fiscal-year start)
      --to <DATE>          the period's last day (default: --as-of)
      --orgs <ORGS>        flow: the orgs to show, comma-separated (default: every org)
      --basis <BASIS>      flow: owed (balances at the date) or paid (movements over the period) [possible values: owed, paid]
      --top <TOP>          t_account: the largest postings listed per side (default 5)
      --spec <SPEC>        the whole spec as JSON (the TUI's `viz` verb takes the same), overriding the arguments
```

## ownpurse tui

Start the terminal UI. `ownpurse tui` runs the `ownpurse-tui` binary with the same arguments; its own usage line is below. See [Terminal UI](/docs/guides/terminal-ui/).

```text
usage: ownpurse-tui [--config ownpurse.json] [--profile name] | --record <record.sqlite> --orgs alias=tenant,... [--group name=a+b] [--offline] [--theme ledger-dark|ledger-light|ledger-dark-256|ledger-light-256|ansi|no-color] [--glyphs unicode|ascii] [--profile name] [--now ISO] [--no-mouse] [--no-agent]
       ownpurse-tui --keys [markdown|json|conflicts]   |   ownpurse-tui --echo-keys   |   ownpurse-tui --check-theme
       ... --snapshot WxH [--script KEYS]   |   ... --dump-states DIR [--sizes 80x24,...]
```
