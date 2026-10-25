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

Today the record holds every version of each Xero document, in XTDB, with the time Xero changed it and the time
ownpurse learned of it ([MODEL.md](MODEL.md)); the fact model above is the direction the data model is moving in.
Changes should move toward it, not away.

## The group system of record

Each of your businesses keeps its books somewhere: one Xero organisation per entity, perhaps QuickBooks for another,
nothing at all for the household. Each of those is the **system of record** for one entity, and it stays that way:
it is what your accountant and the tax filings use.

None of them sees the whole. A loan from one company to another is in both companies' books, on opposite sides,
and only someone looking at both can check that the sides agree, or take them out of a group total. ownpurse is the
**group system of record**: the one place that holds every entity's books together, the intercompany links and
eliminations between them, the matches between a bank's record and the books, and every version of what each
source said.

So there are two kinds of system of record, and they do not compete. An entity's books are official in its own
system, and ownpurse keeps an exact copy, checked to the cent. The group is official in ownpurse. When the group
view shows that an entity's books should change, the change goes to that entity's system as a package a person
approves; ownpurse never edits its copy behind the source's back. The words and the layers are in
[MODEL.md](MODEL.md).

## The layers

From the bottom up. Each layer reads the one below and never writes past it.

1. **Sources**: each source's own tool (xero-cli for Xero) talks to the provider and stores nothing; ownpurse runs
   it and keeps what it returns as facts with provenance, respecting the source's limits.
2. **The record**: an XTDB store on your disk, bitemporal (when it was true, and when ownpurse learned it), never
   rewritten, exact decimals.
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
