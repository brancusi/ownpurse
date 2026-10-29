# Scenarios: the books as layers over facts

Started 2026-10-29: step 1 is done, step 2 is mostly done (docs/plans/NEXT.md, "Scenarios"). Asked for by a workspace's owner: run scenarios of the books (classifications,
intercompany, adjustments), see every report as that scenario would make it, throw it away or adjust it, and publish
one when it is right. Nothing about it is destructive.

## The idea

The books are an evaluation, not a store. Facts come in from the sources and are never changed. Everything else
(what a line is coded to, which lines pair across books, which journals exist, where the year opens) is a **layer**:
data that says how to read the facts. A **scenario** is a stack of layers. Evaluating a scenario gives entries and
postings, and every report (trial balance, balance sheet, profit and loss, intercompany, a project's costs) is read
from that evaluation. Change a layer and the reports change; drop the layer and they are as before.

```
facts (sources, as pulled; never edited)
  └ baseline        the system of record's own coding, as pulled
    └ matches       which evidence line is which book line
      └ world       entities, people, cards and holders, projects, whereabouts
        └ coding    rules and buckets: lines coded by context (the classifier)
          └ overrides   a person's coding of one line, with a reason
            └ intercompany  pairs; a coding in one book implies its mirror in the other
              └ adjustments   journals: accruals, reclassifications, prior-period items, close items
                = entries and postings → reports
```

## The words

- **Layer**: one kind of interpretation, as data, versioned. Each kind has one function that applies it: it takes
  the evaluation so far and returns it changed, and says what it changed and why. Layers are pure; only the facts
  come from the store.
- **Scenario**: a named stack, each layer pinned to a version. Duplicating one is cheap (the new stack points at the
  same layer versions until one is edited, then that layer forks). A scenario records its parent, so any two can be
  compared.
- **Evaluation**: a scenario over a book (or the group book) and a period. Cached by the scenario's layer versions,
  so a report over an unchanged scenario costs nothing, and an edit re-evaluates only from the edited layer down.
- **Live**: the one scenario published for a book. Its evaluation is what ownpurse says the books are. Publishing
  another scenario changes that, and nothing outside ownpurse.
- **Manifest**: the difference between the live evaluation and the baseline (what the system of record says now),
  as changes the system of record can take: recode this line to that account, add this journal, void that one. It is
  approved as one package.
- **Syncer**: one per system of record. Applies an approved manifest, keeps what the system returned (ids), and the
  next pull's compare proves the system of record equals the live evaluation. The baseline then catches up, and the
  manifest is empty.

## What it rests on

- XTDB keeps every version of every document, so layer versions, scenarios and their parents are documents, not new
  machinery. A scenario is not a point in valid time: valid time stays what the facts say about when things
  happened, and system time what was known when. Scenarios are a third axis, by id.
- Today's pieces become layers: the ledger built from Xero's documents is the baseline; `PUT /matches` the matches;
  the pairs config the intercompany layer; the close register's journals (already applied by `pro-forma=proposed`,
  through the trial balance's `:adjustments`) the adjustments layer; the classify scenario files the world and coding
  layers.
- Classification answers (Jev) are cached by the exact request, so re-evaluating after a rule edit asks only what
  changed. A coding below the threshold, or stopped by a check, is not a coding: the line stays as the baseline has
  it and is listed for a person.
- A person's labels (their coding of a line, and why) are both an override layer and the answer key every scenario's
  classifier is scored against. Labels are kept apart from scenarios, so no scenario grades itself.

## The API, roughly

- `GET/POST /scenarios`, `GET/PUT /scenarios/{id}`, `POST /scenarios/{id}/duplicate`, `GET /scenarios/{id}/versions`.
- `GET/PUT /scenarios/{id}/layers/{kind}`: one layer's data.
- Every report takes `scenario` (default: the live one), replacing `pro-forma`.
- `GET /scenarios/{a}/compare/{b}?book&from&to`: lines coded differently, entries added or gone, report deltas.
- `POST /scenarios/{id}/publish {book}`; `GET /manifest?book`; `POST /manifest/{id}/approve`; the syncer applies an
  approved manifest only.

## Steps

1. **The evaluator and two layers.** Baseline and adjustments only: a scenario is the baseline plus a set of
   journals; the trial balance and the other reports take `scenario`. The close register's proposed journals become
   one scenario, so `pro-forma` retires. Done when a scenario made, duplicated, changed and dropped gives the reports
   it should over the samples, and the record's own reports are unchanged with no scenario.
2. **World and coding as layers.** The classify scenario's world, buckets and rules move into a scenario's layers;
   the lines come from a book in the record, not a file. Each bucket names its account in each book it touches
   (paid on behalf names two). The admin edits the world and the rules, runs them, and shows what moved.
3. **Overrides and labels.** The review screen: a person codes a line and says why; reasons become proposed rules;
   scores are against labels.
4. **Intercompany as a layer.** Pairs and mirrors per scenario; the intercompany report per scenario.
5. **Compare and publish.** Two scenarios side by side at every level; one published per book.
6. **The manifest**, against the baseline, as a package (the close register's package, generalised).
7. **The Xero syncer.** xero-cli gains write commands (it is read-only now); dry-run, approval, apply, compare.
   What Xero's API cannot do (move a line between bank accounts, reconcile) is listed in the manifest for a person.

## Open questions

- How a layer change is shown before it is kept: an unsaved draft layer is a scenario too (a throwaway child), so
  "try this" is a duplicate, an edit and a compare, and "keep it" is publishing or merging into the parent.
- Merging: when a child scenario's layer is right, it replaces the parent's version. Two children that both edited
  one layer need a person; no automatic merge of rules.
- Evaluation cost for a whole group's year: the ledger already rebuilds a book offline in seconds; caching per layer
  version should keep reports interactive. Measure in step 1.
- A child cannot take away what a world layer under it says: trips concatenate and projects merge, so a child can
  add a trip but not shorten or drop one, and cannot turn a base's layer off. Either a layer that turns off a layer
  below by id, or trips keyed by id like rules.
- Comparing two scenarios' codings mixes two things: the bucket proposed changes, or only how sure the classifier
  is (so a line crosses the threshold). The admin shows them apart; a rule's wording alone can move the second a
  lot (found 2026-10-29: a rule with a `not` made Jev less sure of the lines the rule did not touch).

