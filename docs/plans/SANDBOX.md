# The sandbox: lines, scenarios and runs as building blocks

Agreed 2026-10-29 with the owner. The admin grew a Sandbox, a Scenarios section and sample data in three steps, and
they overlap: a run started in the Sandbox is the same run the Scenarios section shows, the samples are files that
neither section can edit, and a scenario carries its own lines, so "this scenario on that one line" or "on every book
of the group" cannot be asked. This plan pulls the pieces apart so each is one thing, named plainly.

## The words

| Word | What it is | Was |
|---|---|---|
| **Workspace** | Where ownpurse is running: a group's books (its record, its config), the demo, or the public site. Shown in the admin's bar on every page. | "the record", unnamed on screen |
| **Books** | A workspace's data: entities, their sources' lines and documents, everything pulled. | "the record" |
| **Lines** | A set of bank lines to code: a query over the books (accounts, entities, dates, one line) or made-up lines (the samples). | a scenario's `lines` layer; the samples apart |
| **Key** | Lines a person has coded, the answers a run is scored against. A scenario may key a line otherwise when its policy differs. | a scenario's `key` layer |
| **Evidence** | What is hung on a line beyond the bank's fields: a receipt, an order, a note. It changes what Jev reads, so it is input, never an answer. | receipt and order items only, inside the samples |
| **Scenario** | How to code: the world (people, cards, projects, trips), the buckets and rules, the strategy and threshold. No lines. | a scenario, with its lines inside |
| **Run** | A scenario over lines (with a key and evidence, if any): each line's coding, the cost, the time, and a score where there is a key. | a run, tied to its scenario |
| **In use** | The one scenario whose codings are proposed for the books. Everything else is an experiment. | not yet |

"Record" stays the storage word (the XTDB node and its files, docs/BACKUP.md); it leaves the screens.

## The sections

- **Sandbox**: pick lines and one or more scenarios, run, compare side by side, scored against the key where there
  is one. One line on its own: several scenarios, evidence added or taken away, and run again. The whole group's
  books are a set of lines like any other (for intercompany).
- **Scenarios**: the library of ways to code: layers, versions, duplicate. Each lists its runs, which open in the
  Sandbox.
- **Lines**: saved sets of lines (the samples are sets of made-up lines), the key, and evidence.
- **Docs**: the guide follows any line of any run; the terms.

## What does not change

- Answers are kept by their exact request, so a line under a scenario is never paid for twice, whatever set of lines
  it comes in.
- Nothing in the sandbox writes to a system of record. Codings only propose; a scenario's codings reach the books
  through publishing and a manifest a person approves (SCENARIOS.md).
- The public site (ownpurse.com/sandbox) keeps working through every step: it is the demo workspace, made-up lines
  only.

## Steps

1. **Say where you are, in plain words.** *Done 2026-10-29.* The admin's bar names the workspace on every page (a workspace's name and
   kind from its config: a group's books, the demo, the public site), with how many books it holds. The screens stop
   saying "the record": the Sandbox front shows "Your books" and "Sample lines (made up)"; the terms page gains
   Workspace, Books, Lines, Key, Evidence, Run and In use, and "record" is defined as storage. Done when every page
   says which workspace it is, no screen text says "record" for the data, and the static site says it is the demo.
2. **The Sandbox front as lines × scenario.** *Done 2026-10-29: only the scenarios that read a set of lines are offered for it, and samples are not run from the admin; both lift in step 3.* Pick a set of lines (the samples among them) and a scenario, see what a
   run would cost, run it. The samples move under Lines. Done when a sample and a book's lines are picked the same
   way, and a scenario's runs open from either section.
3. **Lines out of scenarios.** A run is a scenario over a set of lines; a scenario's `lines` layer becomes a saved set
   of lines it defaults to. One line, a filter, or every book's lines can be run under any scenario. The key moves
   with the lines (a scenario's own key layer stays, for a policy that differs). Done when the same scenario runs
   over two sets of lines and both runs are kept and scored, and the existing scenarios still run as before.
4. **Evidence.** Hang a receipt (a PDF or image), an order or a note on a line; the line's facts gain its items; run
   again and compare with and without. Done when one line, with and without a receipt, is compared in the Sandbox.
5. **In use.** Mark one scenario per workspace as in use; the Sandbox and Scenarios show it apart from experiments,
   and only it feeds publishing (SCENARIOS.md step 5).

## Open

- Evidence a workspace already has (Xero's attachments, kept in the record since 2026-10-26) could be offered on the
  lines it belongs to, rather than uploaded again.
- A run over the whole group's books is large (thousands of lines): it needs the estimate first and a limit, as runs
  have now.
