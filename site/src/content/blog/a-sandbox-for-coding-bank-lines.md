---
title: A sandbox for coding bank lines
description: A bank line never says what the money was for. The answer is in its context. How ownpurse splits the work between code, a model that says how sure it is, and a person, why the honest measure is "how often is it confidently wrong", and a live sandbox to try it.
date: 2026-10-28T18:00:00Z
author: Aram Zadikian (@brancusi) and Tally
---

<div class="fm-convo-note"><span class="fm-k">From our working sessions</span><p>Distilled from sessions between Aram Zadikian (<a href="https://github.com/brancusi">@brancusi</a>), who builds ownpurse, and Tally, the Claude agent he builds it with, on 27 and 28 October 2026. The <a href="/blog/starting-again-in-clojure/">previous post</a> argued for the rewrite. This one is the first new thing it made.</p></div>

**Try it now:** [the sandbox](/sandbox/#/sandbox/atlas) on this site, over an invented studio. [The guide](/sandbox/#/docs) follows one line through every step, and [the vocabulary](/sandbox/#/docs/terms) explains every word.

## The question a bank line can't answer

A bank line says when, who was paid, how much, and on which card. It never says what for. A €41.60 lunch in Lisbon could be a traveller's per diem, hospitality for a collaborator, or a personal meal. Each belongs in a different account, and sometimes in a different company's books.

Today's tools answer this with bank rules: if the merchant name contains X, code it to Y. That works for the phone bill. It fails for exactly the lines that take a bookkeeper's time, because the merchant's name is the one thing that doesn't decide them. The same restaurant is per diem on Tuesday and hospitality on Wednesday. What decides it is context: whose card it was, where that person was that day, which project was running, what else was on the receipt, what the note says.

A good bookkeeper codes by context without thinking about it. The idea behind this work is that software can too, if each part of the job goes to whoever is actually good at it.

## Three kinds of work, three workers

ownpurse splits the coding of a line between three workers, and keeps each to what it does well.

- **Code does everything that can be known for certain.** Whose card this is. Whether its holder was on a trip that day, and which day of it. Which projects were running. That this refund returns that purchase, and that this tip belongs to that dinner. Dates are compared by code, never by a model. If something can be computed, it is computed.
- **A model judges what can't be computed, and says how sure it is.** The facts are written out in plain sentences and handed to [Jev](https://typesafe.ai), a model that doesn't write prose. It answers with a probability for every option. "Is this a meal for people who aren't the travellers?" comes back as a number, not an essay.
- **A person decides the rest, and writes the rules.** The rules are sentences a bookkeeper would recognise ("while a traveller is on a project trip, their own meals are that project's per diem"), and each one is also written as blocks code can check. Anything below the confidence threshold, and anything the policy says a person must see, goes to a person.

This split is the insight. Large language models are usually asked to do all three jobs at once, and asked to explain themselves in prose that sounds equally sure whether it's right or not. Here, a model does only the part that needs judgment. Its answer is a probability that code can act on, and a person stays in charge of what that probability is allowed to decide.

## Measuring it honestly

The question that matters isn't how often the model is right. It's how often it is **confidently wrong**, because a wrong coding that nobody looks at is a silent error in the books. Every line in a test scenario has an answer key, and each coding gets one of five outcomes: coded and right; coded and wrong; coded when the policy said a person had to look (*unsafe*, however right the bucket); sent to a person rightly; or sent to a person needlessly.

The test studio, `atlas`, runs two commissions at once, in Lisbon and New York, on the same cards. Other agents wrote most of its 300 bank lines from the bookkeeping policy alone, without seeing the rules, each told to break something. Receipts split some lines into items, so that is 391 things to code. At a threshold of 85%:

| Outcome | Units |
|---|---|
| Coded, right | 240 |
| Coded, wrong | 4 |
| Coded, but a person had to look | 10 |
| To a person, rightly | 108 |
| To a person, needlessly | 29 |

Four rounds of this taught us more than the headline numbers:

- **A model follows a broad rule confidently.** "Restaurants on a project trip are per diem" coded a dinner with a venue's curators as per diem at 92%. The fix wasn't a smarter model. It was rules that state their exceptions, and a bucket for every real outcome.
- **Confidence moves.** The same questions asked three times gave steady totals, but of the 126 units that came near the threshold, 23 changed outcome from one run to the next. A threshold needs a margin, and a threshold tuned on one rule set doesn't carry over to the next.
- **What's left is mostly unsafe, not wrong.** These are sensible codings, confidently chosen, for lines where the policy says a person must look: a note telling the bookkeeper what to do, or a guest no one names. No threshold catches those. A narrow yes/no check for each of those conditions can. That is the next step.

## Seeing it

Numbers like these are easy to misread in a report, so we built a sandbox. Pick a run and move the threshold and the strategy, and every line is rescored in your browser by the same function the evaluation uses. No new questions are asked. A chart shows the trade-off directly: lines coded wrongly against lines sent to a person, at every threshold. Open any line to see its whole path: the raw line, the facts, the words the model read, its answers, each rule's blocks, and seven strategies side by side.

Next to it is a guide for someone who has never thought about any of this. It follows one real line through ten steps, with what each step does in plain words on one side and what it actually did to that line on the other. Because the right-hand side is the run's own data, read through the API, the guide can't quietly go out of date: if the code changes, the guide shows it. There is also a vocabulary of 46 words, each with where it comes from: an accountant's word, a bank's, machine learning's, or one we coined. Writing it forced us to pin our own language down. "Block", for instance, turned out to be ours; bookkeeping tools call these *conditions*.

The sandbox on this site is a static copy of that same app. Every studio, person, card and amount in it is invented, and it was checked against a private list of real names before it was published. No model is called from your browser; the lines were coded by Jev for real, in one of our test runs.

## Run this version

To run the same sandbox on your own machine, at exactly the version described here:

```sh
C=24e3dfffdade19ffda327a56e389ba15d3e9ea97
curl -fsSL https://raw.githubusercontent.com/brancusi/ownpurse/$C/samples/demo.sh | bash -s $C
```

You need git, a JDK (21 or later), the [Clojure CLI](https://clojure.org/guides/install_clojure) and Node. It writes only under `~/.cache/ownpurse/demo/` and serves the admin at `http://127.0.0.1:7070/admin/`. The commit is pinned and tagged [`demo/2026-10-28-sandbox`](https://github.com/brancusi/ownpurse/tree/demo/2026-10-28-sandbox). Read [the script](https://github.com/brancusi/ownpurse/blob/24e3dfffdade19ffda327a56e389ba15d3e9ea97/samples/demo.sh) first if you like.

## Where this goes

The model coding the line is not the interesting part. The interesting part is a bookkeeping process where every automatic decision has a measured error rate, a visible cost and a person's threshold behind it. Next come the checks for lines a person must see, then trips worked out from the spending itself, and then the same pipeline reading real lines straight from your own books, on your own machine.

How it works in more depth: [Classifying bank lines](/docs/guides/classifying/).

<p class="fm-fine">ownpurse is not affiliated with TypeSafe (Jev), Xero Limited or Mercury Technologies. Every studio, person, project, card, merchant and amount in the sandbox and in this post is invented.</p>
