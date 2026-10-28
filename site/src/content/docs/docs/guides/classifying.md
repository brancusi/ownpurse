---
title: Classifying bank lines
description: How ownpurse codes a bank line by its context (who paid, where they were, which project was running), the rules and the model it asks, and the sandbox where you try a rule set before trusting it.
sidebar:
  order: 0
  badge:
    text: New
---

:::note[Experimental]
Classification is built in the Clojure rewrite (the `next` branch), as an eval and a sandbox. It codes nothing in
your books yet. Every line you see in the live sandbox is invented.
:::

A bank line says when, who was paid, how much, and by which card. It never says what the money was for. A restaurant
bill could be a traveller's dinner, a lunch with a client, or a personal treat, and each belongs in a different
account. ownpurse codes a line the way a bookkeeper who knows the context would, and sends the ones it is not sure
of to a person.

**Try it:** [the sandbox](/sandbox/#/sandbox/atlas) shows 391 invented lines coded by the model, with the threshold
and the strategy as levers. [The guide](/sandbox/#/docs) follows one line through every step, and
[the vocabulary](/sandbox/#/docs/terms) explains every word, with where it comes from.

## The pipeline

Four parts do the work: the bank sends the line; **code** looks up and calculates what can be known for certain;
**Jev**, a model, judges what cannot and says how sure it is; and **a person** decides the rest and writes the rules.

| Step | Who | What happens |
|---|---|---|
| 1. The line arrives | The bank | Date, merchant, amount, card, a merchant code (MCC) for the kind of shop, sometimes a note |
| 2. Split into items | Code | A receipt or an order splits a line into items, each coded on its own (a *unit*) |
| 3. Facts | Code | Whose card; where they were that day (on a trip, which day of it, or at home); which projects were running; a refund matched to its purchase; a tip matched to the bill before it |
| 4. Kind of shop | Jev | Only when the merchant code is missing: the kind of business from the name, used at 50% or more |
| 5. In words | Code | The facts written out as the *state* Jev reads: labelled sentences, nothing it does not need |
| 6. Asked | Jev | Two requests: one with every rule written out (which bucket? does each rule apply?), one without (which bucket, from the facts alone? and the yes/no questions some rules depend on) |
| 7. Answers | Jev | A probability for every option, and a confidence for the answer |
| 8. Rules | Code | Each rule's *blocks* checked: exact ones by code, fuzzy ones from Jev's yes/no answers. Exceptions win over the rules they name; a check rule sends the line to a person |
| 9. Strategies | Code | Seven ways of turning the answers into one coding, all from the same answers |
| 10. Code or ask | A person | A coding at or above the threshold is applied; the rest go to a person |

Dates are compared by code, never by the model. If something can be computed, it is computed.

## Rules and blocks

A rule is a sentence a person writes once, and the same rule as building blocks code can check:

```clojure
{:id "A1"
 :code :harbour-per-diem
 :rule "While a traveller is on a trip for the Harbour Exhibition, their food, groceries, restaurants, cafés,
        gym and pharmacy purchases are Harbour per diem."
 :when [[:on-trip-for "harbour"]
        [:category :in #{"5912" "5812" "5814" "7997" "5411"}]]}
```

A rule applies when all its blocks hold. Exact blocks (`:category`, `:card`, `:channel`, `:trip`, `:at-home`,
`:on-trip-for`, `:amount`, `:present`, `:country-mismatch`, `:not`, `:any`) are checked by code. A fuzzy block,
`[:jev "Is this a meal for people who are not the travellers?"]`, is a yes/no question Jev answers. A rule can name
the rules it is the exception to (`:over`), and a check rule (`:check true`) codes nothing: when it holds, a person
looks.

Bookkeeping tools call these bank rules, and their blocks *conditions*. The difference is the fuzzy block: a
condition only judgment can settle.

## Strategies and outcomes

The strategies are compared on the same answers, so trying one costs nothing: Jev with the rules (it picks the
bucket, reading the rules as text), the rule blocks alone, blocks else Jev, Jev stopped by checks, and three for
comparison (without the rules, rule by rule, options reversed).

Each line in a test scenario has an answer key: the right buckets, and whether a person must or may look. At a
threshold, each line has one outcome: **coded, right**; **coded, wrong**; **coded, but a person had to look**
(unsafe, however right the bucket); **to a person, rightly**; or **to a person, needlessly**.

On the invented `atlas` scenario (300 lines, 391 units, 17 rules), Jev with the rules at 85% coded 240 units
rightly, 4 wrongly and 10 unsafely, and sent 137 to a person (108 rightly, 29 needlessly). What that taught us is
in [the blog post](/blog/a-sandbox-for-coding-bank-lines/).

## Run it yourself

You need a JDK (21 or later), the Clojure CLI and Node. From a checkout of the `next` branch:

```sh
bin/admin demo      # builds the admin, runs ownpurse on an empty record; open http://127.0.0.1:7070/admin/
```

The sandbox reads the runs committed in `components/classify/runs`. To make a new run you need a TypeSafe API key
(`TYPESAFE_API_KEY`, or the Keychain item `ownpurse-typesafe`):

```sh
clojure -M:dev -m ownpurse.classify.eval atlas    # about 800 requests for atlas; harbour is smaller
```

The scenarios are `components/classify/resources/classify/eval/<name>.edn`: the world (people, cards, trips,
projects), the buckets, the rules, and the lines with their answer key.

## Your own lines

A private workspace that runs ownpurse serves the same admin at `/admin` on its own port, over its own record. Point
the sandbox at the workspace's own folders, never at the ownpurse repository:

| Variable | What |
|---|---|
| `OWNPURSE_SCENARIOS_DIR` | the workspace's own scenarios, listed beside the invented ones |
| `OWNPURSE_RUNS_DIR` | where its runs are written and read |

Reading lines straight from a book in the record, with no scenario file, is the next step.
