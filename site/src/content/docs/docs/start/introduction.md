---
title: Introduction
description: What ownpurse is, what it keeps, and how its parts fit together.
sidebar:
  order: 1
---

ownpurse keeps a local record of the facts from your financial sources, in a file you hold. You, and the agents
you work with, query that record, rebuild the ledger from it, check it against the source, and plan changes to
your books.

Xero is the first source ownpurse connects to. QuickBooks, bank exports, spreadsheets, investment portfolios and
plain files are planned.

## What you get

- **A record you own.** ownpurse mirrors your books into one SQLite file on your disk. Every version of every
  entity it has seen is kept. Rows are only ever appended, and a SHA-256 chain links each one to the last, so you
  can prove nothing was edited afterwards.
- **A ledger rebuilt locally.** ownpurse derives postings from the documents in the record and computes the trial
  balance, general ledger, profit and loss and balance sheet itself, in exact decimals. It then checks them
  against the provider's own reports, account by account, to the cent.
- **Reads that cost nothing.** Once the record is synced, every read is local. Only a handful of commands talk
  to the provider, and they say so.
- **Many organisations at once.** One login covers every organisation you tick. You address them by short aliases
  (`roast`, `hold`), named groups (`group:ops`) or `all`.
- **Changes you approve.** Proposed changes are collected as document diffs, checked locally, and sealed into a
  package. Nothing reaches the provider until you approve that package by its hash and run it yourself.

## The layers

ownpurse is built in layers. Each one reads the layer below it and never writes past it.

| Layer | What it does | Status |
|---|---|---|
| Connectors | Read a source (Xero today) and append what they see to the record, with where and when it came from | Xero shipped; others planned |
| Record | The append-only, hash-chained SQLite file: every version, every report snapshot, every API call | Shipped |
| Daemon | A long-running process that keeps the record current and serves it | Planned |
| CLI | `ownpurse`: the full surface. Every read, sync, report, check and write plan is a command | Shipped |
| TUI | `ownpurse tui`: a terminal interface over the record, built on caretline | Shipped |

The **CLI** is the complete interface. Anything the TUI can show, a command can print, as compact text or as
`--json`.

The **TUI** is a read-only view over the same record. It is built on
[caretline](https://github.com/brancusi/caretline), with one Elm-style state and a single update loop. That state
is complete: an agent can read it, move the view with a small set of view-only verbs, and point things out to you
on screen, while your own keys always win.

## Who it is for

ownpurse is for people who keep their own books, or run several related organisations, and want to see them
exactly, offline, and alongside an agent. It follows the [AXI](https://axi.md/) conventions for agent-friendly
tools: compact output, counts on every list, structured errors, stable exit codes and no interactive prompts.

## Next steps

- [Install ownpurse](/docs/start/install/)
- [Quickstart](/docs/start/quickstart/): try the demo, then connect your own books
- [Concepts](/docs/start/concepts/): the record, versions, sources and the local ledger

> ownpurse is not affiliated with or endorsed by Xero Limited. "Xero" is a trademark of Xero Limited.
