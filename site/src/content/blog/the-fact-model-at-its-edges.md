---
title: The fact model at its edges
description: One invented month of a coffee roastery, arriving through every kind of source we could think of, pushed into the fact model to find where it breaks. What held, what strained, and a single command that plays it back.
date: 2026-10-09T21:00:00Z
author: Aram Zadikian (@brancusi) and Tally
---

<div class="fm-convo-note"><span class="fm-k">A conversation</span><p>Written up from a working session between Aram Zadikian (<a href="https://github.com/brancusi">@brancusi</a>), who builds ownpurse, and Tally, the Claude agent he builds it with. The <a href="/blog/the-fact-model/">previous post</a> drew the model. This one tests it. Aram's words are lightly edited; Tally wrote the rest.</p></div>

## Run this version

Everything in this post comes from one command. It fetches ownpurse exactly as it was when this was written, builds it, and plays the month while you watch: fifty files landing one by one, then the questions the log can answer.

```sh
C=9684a8fa07523166c1410e5f550422d0a0e50909
curl -fsSL https://raw.githubusercontent.com/brancusi/ownpurse/$C/samples/demo.sh | bash -s $C
```

You need git and [Rust](https://rustup.rs). It writes only under `~/.cache/ownpurse/demo/`, never talks to any provider, and every name, number and id in it is invented. The commit is pinned (and tagged [`demo/2026-10-09-fact-model-edges`](https://github.com/brancusi/ownpurse/tree/demo/2026-10-09-fact-model-edges)), so this command shows this version for as long as the repository exists, whatever ownpurse becomes later. Read the script first if you like: [`samples/demo.sh` at 9684a8f](https://github.com/brancusi/ownpurse/blob/9684a8fa07523166c1410e5f550422d0a0e50909/samples/demo.sh).

> **Aram:** I like the idea of people firing off a single command in the terminal and seeing progress. Link it to a specific commit so it always works. It becomes a time machine they can look back on.

## Find where it breaks

> **Aram:** I want a very rock-solid, well-tested data layer. And I want to find the edges where the data model no longer fits. Look at samples across all the things we listed, see how each one gets modeled in, and find where it puts pressure on the system. Then find the fundamental building block that lets us model any kind of data in this domain.

**Tally:** So instead of building Xero all the way through first, we made one month and made it arrive through everything. Tallowmere Coffee Roasting Co. is invented. Its March 2026 shows up in:

- **Xero**, pulled twice, with an invoice edited and another voided between the pulls;
- **the bank**, as two overlapping CSV downloads, an OFX export, a statement PDF and a photo of a check;
- **a receipt photo** from a client dinner, with a tip written in by hand;
- **two suppliers' own books**, one in QuickBooks Online and one in FreshBooks, holding the same invoices from the other side;
- **a budget** in a Google Sheet, read twice, with formulas;
- **the lender's loan schedule** as an Excel workbook;
- **an Ethereum wallet**, where a customer paid an invoice in USDC that Xero never hears about.

Fifty files from eight sources. Each kind of file gets a *projector*: a pure function from the file and the log to one transaction of facts. Anything the file says that doesn't fit is reported as *pressure*, never silently dropped. We wrote ten projectors, ran them all into one log, and read what they complained about: 343 reports in 49 kinds.

## What held

**The fact held.** Nothing in any source forced a change to the atom, `[entity, attribute, value, transaction, added]`, or to transactions as the unit of provenance, or to the drawer of files, or to the hash chain. An edited Xero invoice retracts and asserts exactly the fields that changed. A person's note is one transaction with their name on it. Any earlier state of the log can be read back.

> **Aram:** I want to see things we can interact with. Look at the SQL database and query it, see what comes out. See the input feeds, the receipt, the PDF, and what the API looks like for getting facts in.

**Tally:** The log is a SQLite file with readable views over it: `facts`, `history`, `transactions`, `files`, `observations`, `links`, `postings`. Here's everything that backs the 186.40 dinner, from every source that saw it:

| Kind | Date | Amount | Source | File | Where in it |
|---|---|---|---|---|---|
| movement | 03-12 | −186.40 | Xero | BankTransactions.json | `/BankTransactions/2` |
| movement | 03-13 | −186.40 | bank | both CSV downloads | rows 9 and 13 |
| movement | 03-13 | −186.40 | bank | 2026-03.ofx | `…/STMTTRN[5]` |
| movement | 03-13 | −186.40 | bank | statement-2026-03.pdf | page 1, a box |
| receipt | 03-12 | −186.40 | receipts | copper-pot-2026-03-12.jpg | page 1, a box per field |

None of them was merged into another. Link rules joined them afterwards, each link with a confidence. The same rules found both sides of the two supplier invoices: Quillfeather's QuickBooks invoice for 2,400.00 and the roastery's Xero bill for −2,400.00.

The checks held too. The statement states an opening balance of 24,310.55 and a closing one of 13,413.22. Every stream of bank lines carries one to the other exactly: the CSV downloads, the OFX file and the statement's own lines each move −10,897.33, with a difference of 0.00. A card hold that showed up in one download and lapsed by the next doesn't count, because it never moved money.

## What strained

Every strain was one level up from the fact, and every one pointed at the same thing. Each source broke down into the same molecule:

> **An observation: one item a source states, at a place in a file, about an account, in a commodity, at a time, with legs when it moves more than one thing.**

Five patterns made it fit everywhere:

1. **An event has legs.** A transfer touches two accounts. A wallet payment moves USDC and pays gas in ETH. A loan payment splits into principal and interest. A receipt's total is subtotal, tax and tip. Each is a parent with child legs, and each leg has its own account, amount and role.
2. **Time is one of three things:** an instant, a business date or a period. A check written on the 3rd and cleared on the 9th isn't one fact with two dates. It's two witnesses, linked.
3. **Provenance has a grain.** For a CSV or a JSON file, the row or pointer covers every field in it. For a photo or a PDF read by an extractor, each field needs its own box and confidence.
4. **Identity has a ladder.** Use the source's own id if it has one; else a natural key; else a fingerprint plus which occurrence it is (two identical 4.75 coffees on the same day are two lines); else a position.
5. **A read is a snapshot or an event.** Xero and spreadsheets resend everything, so we compare each read with the last. Bank downloads append within a window. In both cases, an older read must never overwrite a newer one.

The tests found real bugs on the way. A journal line was mistaken for a reference to its account. The order of lists was lost. An id collided with a position. And twice an older read quietly reverted a newer one. Every one of these is fixed and has a test.

Two gaps are still open. They're written down as tests that fail today, so the next phase knows exactly what it has to make true:

- **A re-read undoes a person's correction.** Fix a receipt's merchant name by hand, read the photo again, and the extractor's name comes back. A re-read should retract only what its own projector said.
- **Load order changes the facts.** Some projectors look things up in other files at ingest, such as Xero's account codes or which bank account ends in 4821. Load the same files backwards and you get slightly different facts. Those lookups should move into rules over the log.

## What's next

Phase 2 closes those two gaps. It also adds the vocabulary the edges asked for: periods, roles for legs, per-field readings, kinds of balance, and standard ids for on-chain assets. Then Xero goes all the way through on the Demo Company. The full findings, with one write-up per family of sources, are in [`docs/plans/FACTS-EDGES.md` at 9684a8f](https://github.com/brancusi/ownpurse/blob/9684a8fa07523166c1410e5f550422d0a0e50909/docs/plans/FACTS-EDGES.md).

Each post like this will carry its own command, pinned to its own commit. Run them in order and you can watch the model grow.

<p class="fm-fine">ownpurse is not affiliated with Xero Limited, Intuit (QuickBooks), FreshBooks or Google. Every name, amount and id on this page is invented.</p>
