---
title: Starting again, in Clojure
description: Books are a history of answers, not a table of balances. Why we rebuilt ownpurse on a database that remembers everything, in a language you work inside of, and why that pairing fits money better than anything we had before.
date: 2026-10-28T12:00:00Z
author: Aram Zadikian (@brancusi) and Tally
---

<div class="fm-convo-note"><span class="fm-k">From our working sessions</span><p>Distilled from a week of sessions between Aram Zadikian (<a href="https://github.com/brancusi">@brancusi</a>), who builds ownpurse, and Tally, the Claude agent he builds it with, from 23 to 28 October 2026. The <a href="/blog/the-landscape/">previous post</a> mapped the field. This one argues for where we are taking ownpurse, and why.</p></div>

On 23 October we archived about 150,000 lines of working Rust and started again in Clojure, on XTDB. That sounds like the kind of decision a project regrets. Five days later, the new version reads four real organisations' books from Xero, rebuilds every ledger, and matches Xero's trial balances to the cent. It does that in about a twentieth of the code. This post is about why that was possible, and why we think the direction is right, not just faster.

## Books are a history, not a state

Most accounting software treats the books as a state: a set of balances, kept current, edited in place. Anyone who has closed a year knows that isn't what books are. Books are a history of answers. The bank said this on Tuesday. The invoice said that, until someone corrected it in March. The accountant posted a journal in December that we now think was wrong. What happened, when we learnt it, and who said so are the actual substance of the work. Today's balance is just the last line of that story.

We had already seen this. ownpurse's Rust version was built on a fact model: every answer a source gave, kept forever and stamped with when it arrived, with the books derived from those facts. It was the right idea. But two weeks of pushing it to its edges showed what we had actually built: a database. We had written our own storage, our own indexes and our own query language for an immutable log of facts. Every hard problem we hit, like a re-read undoing a correction, or load order changing the result, was a database problem we were solving by hand.

The insight was simple once we saw it: that database already exists. XTDB keeps every version of every row with two clocks, when it was true and when we learnt it. Nothing is overwritten; a correction is a new version beside the old one. "What did Xero say about this invoice on 20 March, as we knew it on 2 April?" is a single query. Auditors want exactly that question answered, and spreadsheets can't answer it at all.

## Every source keeps its own shape

The second thing XTDB gives us is quieter, and matters as much. Xero, Mercury, a bank's CSV and a receipt all describe money in different shapes. A conventional schema forces a choice: squeeze them into common columns and lose what doesn't fit, or keep adding columns until the table is mostly empty. Either way, the original is gone.

In XTDB, a table is a set of documents, and two rows can have different fields while SQL still reads them both. So ownpurse keeps each source's answers exactly as they were sent, one schema per source (`xero.invoices`, `mercury.transactions`), and maps them into a shared layer that clients read (`public.invoices`, `public.bank_lines`). Nothing a source said is ever lost to a column we didn't think of. And because the record speaks the Postgres wire protocol, any SQL tool can open it. The record is a folder on your own disk, with no server to run.

None of these properties is new on its own. Document stores, temporal tables and immutable logs all exist. What is new for personal and small-business books is having them together, locally, under one owner. That is the foundation ownpurse needed and had been building badly by hand.

## Working inside the program

Clojure came with XTDB, but it is not an accident of the choice. It fits the work.

A Clojure program is developed from inside, while it runs. One long-lived process holds the record and the API; a changed function is evaluated into it, and the next request uses it. The tests that cover a file run each time it is saved. There is no build step between a thought and its answer.

For books, that matters more than it would for most software, because the work is an investigation. Why does this account not tie? Which lines moved bucket between two runs? What did the journal look like before it was edited? Each answer is plain data in the REPL: maps, vectors and sets you can filter, join and ask the next question of in the same session. Nothing has to be exported to be looked at. The code and the questions about the books are the same activity.

The same data travels all the way to the screen. The admin is ClojureScript, and the API answers it in Transit, so the browser receives the same keywords, sets and maps the server holds, with no translation layer in between. The function that scores a bank line is one file, compiled for both the JVM and the browser. When the [sandbox](/sandbox/#/sandbox/atlas) on this site shows how many lines a model coded wrongly, it is the evaluation's own function saying so, not a re-implementation of it.

And it suits an agent as well as a person. Tally evaluates code in the same running system Aram uses from his editor, with the same tools. When something breaks, the agent can call the function directly and look at what it returns, rather than guessing from a stack trace. A person and an agent working inside one live program, on the same data, is a better way to build software than either of them working in a file.

## What it costs

It isn't free, and the costs are worth saying plainly. A long-lived process can hold on to code that has since changed. We met a handler built from an older function, which gave errors from code that was correct on disk. So nothing is committed on the REPL's word: before every commit, a fresh process runs every test (100 of them, 729 assertions) and Polylith's structural check, trusting nothing held in memory. The JVM and its libraries take minutes to download the first time. And parentheses are unforgiving for an agent writing without structural editing. The linter catches those in seconds, but it has to be there.

## What carried over

The rewrite kept everything that was true about the Rust version. ownpurse is still a local record that keeps every answer a source gave, with books rebuilt from it and checked to the cent. Money is never a float; the linter now refuses one anywhere in the code. Writes to a provider happen only as sealed packages a person approves. What changed is that the foundation now does the hard part instead of us.

The [next post](/blog/a-sandbox-for-coding-bank-lines/) shows the first thing this foundation made possible: coding bank lines by what was really going on, and a sandbox to watch it. The developer docs for the new version start at [the Clojure workspace](/docs/developers/clojure/).

<p class="fm-fine">ownpurse is not affiliated with JUXT (XTDB), Cognitect or Nubank (Datomic), Xero Limited or Mercury Technologies.</p>
