---
title: Introducing ownpurse
description: Your financial data is spread across a dozen systems, and none of them will show you the whole. ownpurse is a local record of every fact about your money, built for you and your agents.
date: 2026-10-09
---

Start with an ordinary week. Your business keeps its books in Xero. Your household budget lives in a spreadsheet. One card pays for both. A brokerage account, a pension and a folder of PDF statements in a cloud drive each hold another piece. Each system is good at its own part, and none of them will show you the whole.

For years the answer was to export, stash and reconcile by hand: a CSV, a spreadsheet, an afternoon, and next month the same again. Tools that promised to unify it all were ahead of their time. Every bank, every provider and every spreadsheet needed its own connector, and someone had to build and maintain each one.

That bottleneck has gone. An agent can write a connector for your bank's export in an afternoon. The hard problem has moved: once everything arrives, where does it go, and how do you make sense of it?

## A record you hold

ownpurse is our answer. It keeps one record, on your machine or your own server, of every fact about your money. A fact is small: this entity has this attribute with this value, as of this moment, according to this source. The record only grows. A correction is a new fact, database triggers refuse edits and deletes, and a SHA-256 chain proves nothing was changed after the fact.

From that record ownpurse rebuilds your ledger and checks it against the source, account by account, to the cent. Today the source is Xero: one login covers every organisation you tick, sync fetches only what changed, and reading never costs an API call.

## Every view is a query

Because the record holds facts rather than one system's tables, a profit and loss, a household budget and a card reconciliation are all questions asked of the same data. A fact doesn't have to be forced into a chart of accounts on the way in. It can arrive and wait, held but not posted, until you decide what it means.

## You and your agents

The CLI is written for agents first: compact output, a count on every list, structured errors with stable exit codes, and `--json` on everything. The terminal UI, built on caretline, keeps a single state value and derives every screen from it, so an agent can read what you're looking at and drive the same views you would.

That changes the work. Instead of digging through the complexity yourself, your agent finds the facts, builds the view and walks you through it. When something should change in your books, it goes out as a planned, sealed package that you approve, and the record keeps the attempt and the result.

## What's next

ownpurse is in early access. The Xero connector works today; QuickBooks, bank exports, spreadsheets, portfolios and files are next. A small daemon will keep the record company, scheduling syncs and serving live updates to any tool you or your agents build.

Underneath, it will stay simple. Bookkeeping got complicated. The thing at its core never did: atomic events over time.

[Get started](/get-started/), read [the philosophy](/philosophy/), or go straight to [the docs](/docs/).
