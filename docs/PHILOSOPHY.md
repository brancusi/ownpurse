# Philosophy

This is what ownpurse is for and the principles every change is checked against. The website's
[philosophy page](https://ownpurse.com/philosophy/) says the same thing for a general reader.

## The problem

Financial data no longer lives in one system. A business keeps its books in Xero, a household budget lives in a
spreadsheet, one card pays for both, and a brokerage account, a pension and a folder of PDF statements each hold
another piece. Each source is good at its part and none shows the whole.

The data is also held hostage by its tool: shaped by one provider's chart of accounts, rationed by its API (a new
Xero app gets 1,000 calls a day per organisation), reached through an export button, and painful to move when you
change providers.

## Why now

Tools that tried to unify this were ahead of their time: every source needed a connector, and someone had to build
and maintain each one. Agents removed that bottleneck. An agent can write a connector for a bank's export in an
afternoon. The hard problem has moved from *getting the data* to *where it goes and how you make sense of it*.
ownpurse is that place.

## Sovereignty

Your financial data belongs to you, not to the tool that recorded it.

- The record is a file you hold, on your laptop or your own server. No account with us; nothing is sent anywhere
  you did not choose. Reads cost no API calls.
- Every version of every document is kept, and a hash chain proves nothing was edited later.
- Moving providers should be one sentence: here is my data, make it work there.

## The model: atomic facts over time

Double-entry bookkeeping is the oldest working data model in business. In 1494 Luca Pacioli wrote down the method
Venetian merchants already used: record every movement twice, as a debit and a credit, in a book you add to and
never erase. A mistake is corrected by a new entry, not scratched out. That is an append-only log of facts with a
built-in integrity check, five centuries before databases had the words.

ownpurse brings that log back to the surface. Its unit is a **fact**: this entity has this attribute with this
value, as of this moment, according to this source. It is the shape of an RDF triple or a Datomic datom, with
time and provenance attached. A posting, a bank line, a budget row and a share price all fit it.

- **Facts, not tables.** Tables are something you ask for, not something you are stuck with.
- **Append, never overwrite.** A correction is a new fact. The record only grows.
- **Stay uncollapsed.** A fact can arrive and wait in an intermediate state, held but not posted, like a waveform
  that has not collapsed. A card payment can be "possibly repairs, possibly household" until a person decides.
  Posting is the moment it collapses, and that moment is recorded too.
- **Every view is a query.** A trial balance, a P&L, a household budget and a reconciliation are questions asked of
  the same facts. The closer you get to the record, the less ownpurse decides for you.
- **Balance is the test.** Debits equal credits, and every book agrees with its source to the cent, or ownpurse
  shows exactly where it does not.

Today the record holds versions of Xero documents (`docs/design.md`); the fact model above is the direction the
data model is moving in. Changes should move toward it, not away.

## The layers

From the bottom up. Each layer reads the one below and never writes past it.

1. **Connectors** turn a source into facts with provenance, respecting the source's limits. Xero today.
2. **The record**: one SQLite file, append-only by trigger, SHA-256 chained, exact decimals.
3. **The daemon** (planned): a small local process that schedules syncs and serves facts and live updates to any
   tool or agent. It does nothing the CLI cannot.
4. **The CLI**: the whole surface, agent-first (AXI): compact output, counts, structured errors, stable exit codes,
   `--json`, no prompts, next-step hints.
5. **The TUI**, built on caretline in the Elm architecture: one state value, messages that change it, every screen
   derived from it. An agent can read that state and drive the same views a person would, to lead them through the
   data.
6. **Your tools**: anything you and your agents build on top.

## You and your agents

The work is a collaboration. Instead of a person digging through the complexity alone, an agent finds the facts,
builds the view and walks the person through it. **Agents propose; people post.** Reports, reconciliations and
manual journals are small, reviewable events layered on the record. Writes to a source go out only as planned,
sealed packages that a person approves, and the record keeps every attempt and result.

## Simple underneath

However complicated the books get, the base stays the same. It always was, and always will be, atomic events over
time.
