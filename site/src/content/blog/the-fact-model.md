---
title: The fact model
description: Every source broken down to one atom, a fact stamped by a transaction, and built back up into evidence, links and books. The prior art it borrows from, the design, and the database question.
date: 2026-10-09T11:00:00Z
author: Aram Zadikian (@brancusi) and Tally
---

<div class="fm-convo-note"><span class="fm-k">A conversation</span><p>Written up from a working session between Aram Zadikian (<a href="https://github.com/brancusi">@brancusi</a>), who builds ownpurse, and Tally, the Claude agent he builds it with. The <a href="/blog/where-your-money-lives/">previous post</a> covers why. Aram's words are lightly edited; Tally wrote the rest.</p></div>

**Tally:** The [previous post](/blog/where-your-money-lives/) ended with a list: keep what every source said, turn every source into one small unit, keep every correction, connect every figure to its evidence, let things wait until posted, answer as of any moment, and do it in one program on your machine. This post is the model we drew to meet it.

The short version: everything that arrives is a file in a drawer. Everything ownpurse knows is a fact about something, stamped with who said it and from what. Everything you see is a query over those facts, down to the page of the statement that backs a figure.

## Down to the atom

> **Aram:** I want to think about reworking the abstraction and the data model to be more RDF style, and have this event log that we can build up from. We're pulling in the facts, the individual datoms, to use Datomic lingo.

**Tally:** So we took every kind of financial data apart until there was nothing left to take apart. A Xero invoice is a document with lines. A line has an account and an amount. A bank statement is a list of lines and two balances. A receipt is a merchant, a total, a tip. A check is a number, a payee, an amount and a signature.

Every one of those, at the bottom, is the same sentence: *this thing has this property with this value, according to this source, as of this moment.* That sentence is the atom.

<div class="fm-wide">
<div class="fm-atom">
<div><span class="fm-atom-k">e</span><span class="fm-atom-d">the thing: a file, a statement line, an entry</span><span class="fm-atom-x">obs:88</span></div>
<div><span class="fm-atom-k">a</span><span class="fm-atom-d">the attribute, from a typed schema that is itself made of facts</span><span class="fm-atom-x">:obs/amount</span></div>
<div><span class="fm-atom-k">v</span><span class="fm-atom-d">the value: text, exact decimal, date, or a reference to another thing</span><span class="fm-atom-x">−186.40 USD</span></div>
<div><span class="fm-atom-k">tx</span><span class="fm-atom-d">the transaction that wrote it, in log order</span><span class="fm-atom-x">tx 3101</span></div>
<div><span class="fm-atom-k">op</span><span class="fm-atom-d">assert or retract; nothing is ever overwritten</span><span class="fm-atom-x">+</span></div>
</div>
<div class="fm-txline"><b>Transaction</b><span>:tx/actor agent:bank</span><span>:tx/at 2026-03-15T06:02</span><span>:tx/reason "Daily card download"</span><span>:tx/evidence file:41ab</span><span>:tx/projector bank-csv@1</span><span>:tx/prev-hash · :tx/hash</span></div>
</div>

The transaction is itself a thing with facts of its own. It records who wrote the batch (you, an agent, a rule, a connector), when, why, from which file, and with which version of the code that read it. Provenance is written once per batch, and every fact in the batch inherits it. Each transaction carries the hash of the one before, so the whole log is a chain.

From that one shape we build everything back up: the files, what they say, how the things they say relate, and finally your books.

## Prior art

None of this is new, and that's the point. We're borrowing from people who got it right.

**Luca Pacioli, 1494.** Double entry is the oldest working data model in business. Record every movement twice, in a book you add to and never erase; fix a mistake with a new entry, not a scratched-out line. That's an append-only log with a built-in integrity check, five centuries before databases had the words. The [coin and T-account](/blog/the-coin-and-the-t-account/) in our mark come from there.

**RDF and Datalog.** The W3C's Resource Description Framework made the triple (subject, predicate, object) the way to describe anything, so data from different sources can sit in one graph without first agreeing on tables. Datalog, from the deductive-database research of the 1980s, is the query language that fits it: you write patterns with variables, and joins happen wherever a variable repeats.

**Datomic, Rich Hickey and Clojure.** This is the biggest debt. Rich Hickey designed Clojure (2007) around immutable values, and then, with the team that became Cognitect, built Datomic (2012) on the same idea. His talks *The Value of Values* and *Deconstructing the Database* make the argument better than we can: a fact, once true at a time, stays true about that time. Nothing is updated, only accreted. The database itself becomes a value you can hold, pass around and ask questions of as of any moment. Datomic's datom, `[entity attribute value transaction added?]`, is our atom almost exactly. So are its transactions as entities that carry their own metadata, its schema stored as data, its pull API that fetches a shape rather than rows, and its speculative `with` that applies a transaction to a database without committing it. Nubank runs a bank on it, and since 2023 it has been free to use.

**XTDB.** JUXT's XTDB took the same ideas further into time: every row has both the time the system learned it and the time it was true in the world. Its query language, XTQL, keeps Datalog's logic variables and adds composable pipelines that make aggregation natural.

**Plain-text accounting.** Ledger, hledger and Beancount showed that a person can keep real books as a log of entries, with amounts in any commodity (dollars, shares, coins) and balance assertions that fail loudly when the books and the bank disagree. Our `Balance` facts and multi-commodity amounts come from there.

### What immutability buys

> **Aram:** We really need this event log so that we can quickly time travel, roll back, and have a single source of truth that we can audit at a finely granular level.

**Tally:** That's the case for immutability in one sentence. Concretely, a log of facts that only grows gives ownpurse:

- **An audit trail at the level of a single value.** Every amount, account and status has an author, a time, a reason and the file it came from.
- **Time travel for free.** The books as of last April are the log up to last April's transaction.
- **Rollback without loss.** Undoing a change is a new transaction that retracts what the old one asserted. The mistake and its correction both stay visible, the way Pacioli wanted.
- **Safe agents.** Everything an agent does is a small transaction you can read, keep or retract. Proposals are speculative transactions that never touch the log until you approve them.
- **Rebuildable everything.** Indexes, totals and reports are caches. Delete them and they come back from the log.
- **Reprocessing.** If the code that reads a bank's CSV had a bug, fix it and replay the drawer. No source is asked twice.

## Five layers, read left to right

Here is the whole model on one sheet. Arrows point from the thing that refers to the thing it refers to. Each layer only refers to the layers to its left, except links, which may join any two things.

<div class="fm-wide">
<div class="fm-sheet" role="img" aria-label="Schema: File in the drawer; Document and Region; Observation and Balance; Link, Label and Note; Entry, Posting and Account in the books; Transaction stamps every fact.">
<svg viewBox="0 0 995 530" xmlns="http://www.w3.org/2000/svg">
<defs>
<marker id="fm-ah" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="8" markerHeight="8" orient="auto-start-reverse">
<path class="arrow" d="M0,0 L10,5 L0,10 z"/>
</marker>
</defs>
<text class="t-col" x="20" y="40">1 DRAWER</text>
<text class="t-col" x="215" y="40">2 DOCUMENTS</text>
<text class="t-col" x="410" y="40">3 OBSERVATIONS</text>
<text class="t-col" x="605" y="40">4 LINKS</text>
<text class="t-col" x="800" y="40">5 BOOKS</text>
<!-- File -->
<g transform="translate(20,60)">
<rect class="box" width="175" height="125"/><rect class="head" x=".75" y=".75" width="173.5" height="25"/><line class="sep" x1="0" y1="26" x2="175" y2="26"/>
<text class="t-name" x="10" y="18">File</text>
<text class="t-attr" x="10" y="44">:file/sha256  id</text>
<text class="t-attr" x="10" y="61">:file/media-type</text>
<text class="t-attr" x="10" y="78">:file/name</text>
<text class="t-attr" x="10" y="95">:file/size</text>
<text class="t-attr" x="10" y="112">:file/captured-at</text>
</g>
<!-- Document -->
<g transform="translate(215,60)">
<rect class="box" width="175" height="125"/><rect class="head" x=".75" y=".75" width="173.5" height="25"/><line class="sep" x1="0" y1="26" x2="175" y2="26"/>
<text class="t-name" x="10" y="18">Document</text>
<text class="t-attr" x="10" y="44">:doc/kind</text>
<text class="t-attr t-ref" x="10" y="61">:doc/files → File</text>
<text class="t-attr t-ref" x="10" y="78">:doc/issuer → Party</text>
<text class="t-attr" x="10" y="95">:doc/period</text>
<text class="t-attr" x="10" y="112">:doc/external-id</text>
</g>
<!-- Region -->
<g transform="translate(215,215)">
<rect class="box" width="175" height="108"/><rect class="head" x=".75" y=".75" width="173.5" height="25"/><line class="sep" x1="0" y1="26" x2="175" y2="26"/>
<text class="t-name" x="10" y="18">Region</text>
<text class="t-attr t-ref" x="10" y="44">:region/doc → Doc</text>
<text class="t-attr" x="10" y="61">:region/page</text>
<text class="t-attr" x="10" y="78">:region/box</text>
<text class="t-attr" x="10" y="95">:region/row | pointer</text>
</g>
<!-- Observation -->
<g transform="translate(410,60)">
<rect class="box" width="175" height="142"/><rect class="head" x=".75" y=".75" width="173.5" height="25"/><line class="sep" x1="0" y1="26" x2="175" y2="26"/>
<text class="t-name" x="10" y="18">Observation</text>
<text class="t-attr t-ref" x="10" y="44">:obs/region → Region</text>
<text class="t-attr" x="10" y="61">:obs/kind</text>
<text class="t-attr" x="10" y="78">:obs/date · /amount</text>
<text class="t-attr" x="10" y="95">:obs/party · /number</text>
<text class="t-attr" x="10" y="112">:obs/text · /confidence</text>
<text class="t-attr" x="10" y="129">:&lt;kind&gt;/* typed extras</text>
</g>
<!-- Balance -->
<g transform="translate(410,232)">
<rect class="box" width="175" height="108"/><rect class="head" x=".75" y=".75" width="173.5" height="25"/><line class="sep" x1="0" y1="26" x2="175" y2="26"/>
<text class="t-name" x="10" y="18">Balance</text>
<text class="t-attr t-ref" x="10" y="44">:balance/account</text>
<text class="t-attr" x="10" y="61">:balance/date</text>
<text class="t-attr" x="10" y="78">:balance/amount</text>
<text class="t-attr t-ref" x="10" y="95">:balance/region</text>
</g>
<g transform="translate(410,370)">
<rect class="dash" width="175" height="70"/>
<text class="t-note" x="10" y="22">also in the books</text>
<text class="t-attr" x="10" y="42">Book · Party</text>
<text class="t-attr" x="10" y="59">Commodity · Price</text>
</g>
<!-- Link -->
<g transform="translate(605,60)">
<rect class="box" width="175" height="125"/><rect class="head" x=".75" y=".75" width="173.5" height="25"/><line class="sep" x1="0" y1="26" x2="175" y2="26"/>
<text class="t-name" x="10" y="18">Link</text>
<text class="t-attr" x="10" y="44">:link/kind</text>
<text class="t-attr t-ref" x="10" y="61">:link/a → any</text>
<text class="t-attr t-ref" x="10" y="78">:link/b → any</text>
<text class="t-attr" x="10" y="95">:link/confidence</text>
<text class="t-attr" x="10" y="112">:link/status</text>
</g>
<!-- Label, note -->
<g transform="translate(605,215)">
<rect class="box" width="175" height="91"/><rect class="head" x=".75" y=".75" width="173.5" height="25"/><line class="sep" x1="0" y1="26" x2="175" y2="26"/>
<text class="t-name" x="10" y="18">Label, Note</text>
<text class="t-attr t-ref" x="10" y="44">:label/on → any</text>
<text class="t-attr" x="10" y="61">:label/type, /value</text>
<text class="t-attr t-ref" x="10" y="78">:note/on, /text</text>
</g>
<!-- Entry -->
<g transform="translate(800,60)">
<rect class="box" width="175" height="108"/><rect class="head" x=".75" y=".75" width="173.5" height="25"/><line class="sep" x1="0" y1="26" x2="175" y2="26"/>
<text class="t-name" x="10" y="18">Entry</text>
<text class="t-attr t-ref" x="10" y="44">:entry/book → Book</text>
<text class="t-attr" x="10" y="61">:entry/date</text>
<text class="t-attr" x="10" y="78">:entry/narration</text>
<text class="t-attr" x="10" y="95">:entry/status</text>
</g>
<!-- Posting -->
<g transform="translate(800,195)">
<rect class="box" width="175" height="125"/><rect class="head" x=".75" y=".75" width="173.5" height="25"/><line class="sep" x1="0" y1="26" x2="175" y2="26"/>
<text class="t-name" x="10" y="18">Posting</text>
<text class="t-attr t-ref" x="10" y="44">:posting/entry</text>
<text class="t-attr t-ref" x="10" y="61">:posting/account</text>
<text class="t-attr" x="10" y="78">:posting/quantity</text>
<text class="t-attr t-ref" x="10" y="95">:posting/commodity</text>
<text class="t-attr t-ref" x="10" y="112">:posting/party</text>
</g>
<!-- Account -->
<g transform="translate(800,348)">
<rect class="box" width="175" height="91"/><rect class="head" x=".75" y=".75" width="173.5" height="25"/><line class="sep" x1="0" y1="26" x2="175" y2="26"/>
<text class="t-name" x="10" y="18">Account</text>
<text class="t-attr t-ref" x="10" y="44">:account/book</text>
<text class="t-attr" x="10" y="61">:account/type</text>
<text class="t-attr" x="10" y="78">:account/code</text>
</g>
<!-- edges -->
<path class="edge" d="M215 120 L197 120" marker-end="url(#fm-ah)"/>
<path class="edge" d="M302 215 L302 187" marker-end="url(#fm-ah)"/>
<path class="edge" d="M410 150 L399 150 L399 255 L392 255" marker-end="url(#fm-ah)"/>
<path class="edge" d="M410 300 L392 300" marker-end="url(#fm-ah)"/>
<path class="edge" d="M605 112 L587 112" marker-end="url(#fm-ah)"/>
<path class="edge" d="M780 112 L798 112" marker-end="url(#fm-ah)"/>
<text class="t-note" x="590" y="104">a</text>
<text class="t-note" x="786" y="104">b</text>
<path class="edge" d="M887 195 L887 170" marker-end="url(#fm-ah)"/>
<path class="edge" d="M887 320 L887 346" marker-end="url(#fm-ah)"/>
<path class="edge" d="M585 320 L596 320 L596 395 L798 395" marker-end="url(#fm-ah)"/>
<!-- transaction band -->
<rect class="fm-band" x="20" y="462" width="955" height="52"/>
<text class="t-name" x="34" y="484">Transaction</text>
<text class="t-note" x="140" y="484">stamps every fact in every column above</text>
<text class="t-attr" x="34" y="503">:tx/actor   :tx/at   :tx/reason   :tx/evidence → File   :tx/projector   :tx/prev-hash   :tx/hash</text>
</svg>
</div>
<div class="fm-titleblock"><span>Sheet 1 of 1 · schema</span><span>refs in lapis · arrows point to the referent</span><span>ownpurse · fact model</span></div>
</div>

<div class="fm-wide">
<div class="fm-legend">
<div><span class="fm-n">1 Drawer</span><p>The bytes, exactly as they arrived: PDFs, photos, CSVs, and every API response body. Named by their SHA-256, so the same file captured twice is one file.</p></div>
<div><span class="fm-n">2 Documents</span><p>What a file is: a March statement, a receipt, a check. A document can have several files (front and back of a check). A region is a place inside it.</p></div>
<div><span class="fm-n">3 Observations</span><p>One item a document reports, pinned to its region: a statement line, a receipt, a check. Shared attributes for date, amount and party; typed extras for the rest.</p></div>
<div><span class="fm-n">4 Links</span><p>How things relate, with an author and a confidence: same-event, evidences, settles, mirrors. Matching is a fact, so it can be reviewed and undone.</p></div>
<div><span class="fm-n">5 Books</span><p>Your double-entry books: entries held until posted, postings in any commodity. Entries are minted by ownpurse; no source owns them.</p></div>
</div>
</div>

The drawer is the part that answers the evidence problem from the first post. Every connector works the same way: it fetches bytes and drops them in the drawer, and a separate, versioned projector turns them into facts. Xero becomes one connector among many. Its API responses are files we already hold, so reprocessing them never costs a call.

> **Aram:** I like the idea of moving the Xero payloads as an artifact in the drawer that we simply process again. Every connector basically should have that. Xero should just be another connector.

### What makes each thing the same thing

Identity decides whether a second sync adds facts or repeats them. Each kind gets one rule:

| Thing | Identity |
|---|---|
| File | The SHA-256 of its bytes |
| Document | The source's own id when it has one (statement number, check number, provider id); otherwise its first file's hash |
| Region | Document and locator: a row, a JSON pointer, a page, or a page and a box |
| Observation | Region and kind: one per item, so a check is one observation, not five |
| A bank line seen by several sources | Not an identity. A fingerprint that link rules match on, so a wrong match can be undone |
| Entry, posting, account | Minted by ownpurse. External ids are ordinary facts, so leaving a provider changes nothing about an entry |

## A dinner, one transaction at a time

A client dinner, paid with card 4417. Three files end up backing it: the card's daily activity download, a phone photo of the receipt, and the month-end statement. Each block is one transaction from the log; `+` asserts and `−` retracts. Every name, amount and id here is invented.

<div class="fm-wide">
<div class="fm-timeline">
<div class="fm-tx"><div class="fm-meta"><span class="fm-id">tx 3101</span><span class="fm-who">agent:bank</span><span class="fm-when">2026-03-15 06:02</span></div><div class="fm-why">Daily card download</div>
<div class="fm-facts"><table>
<tr><td class="op">+</td><td class="e">file:41ab</td><td class="a">:file/name</td><td>activity-2026-03-15.csv</td></tr>
<tr><td class="op">+</td><td class="e">doc:card-0315</td><td class="a">:doc/kind</td><td>:card-activity</td></tr>
<tr><td class="op">+</td><td class="e">obs:88</td><td class="a">:obs/region</td><td class="ref">doc:card-0315 · row 37</td></tr>
<tr><td class="op">+</td><td class="e">obs:88</td><td class="a">:obs/date</td><td>2026-03-14</td></tr>
<tr><td class="op">+</td><td class="e">obs:88</td><td class="a">:obs/amount</td><td>−186.40 USD</td></tr>
<tr><td class="op">+</td><td class="e">obs:88</td><td class="a">:obs/text</td><td>HARBOURLINE BISTRO SEATTLE WA</td></tr>
</table></div></div>
<div class="fm-tx"><div class="fm-meta"><span class="fm-id">tx 3102</span><span class="fm-who">you · phone</span><span class="fm-when">2026-03-14 21:12</span></div><div class="fm-why">Snapped the receipt</div>
<div class="fm-facts"><table>
<tr><td class="op">+</td><td class="e">file:9c1e</td><td class="a">:file/name</td><td>IMG_2041.jpg</td></tr>
<tr><td class="op">+</td><td class="e">doc:rcpt-9c1e</td><td class="a">:doc/kind</td><td>:receipt</td></tr>
</table></div></div>
<div class="fm-tx"><div class="fm-meta"><span class="fm-id">tx 3103</span><span class="fm-who">agent:extract · receipt@2</span><span class="fm-when">2026-03-15 06:03</span></div><div class="fm-why">Read the receipt</div>
<div class="fm-facts"><table>
<tr><td class="op">+</td><td class="e">obs:90</td><td class="a">:obs/region</td><td class="ref">doc:rcpt-9c1e · p1</td></tr>
<tr><td class="op">+</td><td class="e">obs:90</td><td class="a">:obs/amount</td><td>186.40 USD</td></tr>
<tr><td class="op">+</td><td class="e">obs:90</td><td class="a">:receipt/subtotal</td><td>152.00 USD</td></tr>
<tr><td class="op">+</td><td class="e">obs:90</td><td class="a">:receipt/tax</td><td>15.20 USD</td></tr>
<tr><td class="op">+</td><td class="e">obs:90</td><td class="a">:receipt/tip</td><td>19.20 USD</td></tr>
<tr><td class="op">+</td><td class="e">obs:90</td><td class="a">:obs/located</td><td>[:obs/amount  p1 box(40,610,520,48)]</td></tr>
<tr><td class="op">+</td><td class="e">obs:90</td><td class="a">:obs/confidence</td><td>0.97</td></tr>
</table></div></div>
<div class="fm-tx"><div class="fm-meta"><span class="fm-id">tx 3104</span><span class="fm-who">rule:card-receipt</span><span class="fm-when">2026-03-15 06:03</span></div><div class="fm-why">Same amount, same day, same merchant</div>
<div class="fm-facts"><table>
<tr><td class="op">+</td><td class="e">link:40</td><td class="a">:link/kind</td><td>:same-event  obs:88 ↔ obs:90</td></tr>
<tr><td class="op">+</td><td class="e">link:40</td><td class="a">:link/confidence</td><td>0.98</td></tr>
</table></div></div>
<div class="fm-tx fm-held"><div class="fm-meta"><span class="fm-id">tx 3105</span><span class="fm-who">agent:books</span><span class="fm-when">2026-03-15 06:04</span><span class="fm-status fm-s-held"><i></i>Held</span></div><div class="fm-why">Proposed an entry</div>
<div class="fm-facts"><table>
<tr><td class="op">+</td><td class="e">entry:512</td><td class="a">:entry/status</td><td>:held</td></tr>
<tr><td class="op">+</td><td class="e">post:1024</td><td class="a">:posting/account</td><td class="ref">acct:6420 Meals and entertainment</td></tr>
<tr><td class="op">+</td><td class="e">post:1024</td><td class="a">:posting/quantity</td><td>186.40 USD</td></tr>
<tr><td class="op">+</td><td class="e">post:1025</td><td class="a">:posting/account</td><td class="ref">acct:2100 Card 4417</td></tr>
<tr><td class="op">+</td><td class="e">post:1025</td><td class="a">:posting/quantity</td><td>−186.40 USD</td></tr>
<tr><td class="op">+</td><td class="e">link:41</td><td class="a">:link/kind</td><td>:evidences  obs:88 → entry:512</td></tr>
</table></div></div>
<div class="fm-tx fm-post"><div class="fm-meta"><span class="fm-id">tx 3106</span><span class="fm-who">you</span><span class="fm-when">2026-03-15 08:30</span><span class="fm-status fm-s-posted"><i></i>Posted</span></div><div class="fm-why">Posted it</div>
<div class="fm-facts"><table>
<tr class="retract"><td class="op r">−</td><td class="e">entry:512</td><td class="a">:entry/status</td><td>:held</td></tr>
<tr><td class="op">+</td><td class="e">entry:512</td><td class="a">:entry/status</td><td>:posted</td></tr>
</table></div></div>
<div class="fm-tx"><div class="fm-meta"><span class="fm-id">tx 3190</span><span class="fm-who">agent:bank</span><span class="fm-when">2026-04-02 06:00</span></div><div class="fm-why">The March statement arrives, and its line joins the same event</div>
<div class="fm-facts"><table>
<tr><td class="op">+</td><td class="e">doc:stmt-0331</td><td class="a">:doc/files</td><td class="ref">file:77d0  statement-2026-03.pdf</td></tr>
<tr><td class="op">+</td><td class="e">obs:301</td><td class="a">:obs/region</td><td class="ref">doc:stmt-0331 · p2 box(36,412,540,14)</td></tr>
<tr><td class="op">+</td><td class="e">obs:301</td><td class="a">:obs/amount</td><td>−186.40 USD</td></tr>
<tr><td class="op">+</td><td class="e">bal:12</td><td class="a">:balance/amount</td><td>−2,904.18 USD  acct:2100 on 2026-03-31</td></tr>
<tr><td class="op">+</td><td class="e">link:77</td><td class="a">:link/kind</td><td>:same-event  obs:301 ↔ obs:88</td></tr>
</table></div></div>
</div>
</div>

The entry was held until a person posted it. That's the uncollapsed state from our [philosophy](/philosophy/), and posting is just two facts in a transaction with your name on it. Whether a rule may ever post without you is a setting: by default, nothing posts on its own.

### Show me what backs this

From the posting, follow the entry to its evidence, then the same-event links outward, then regions down to files.

<div class="fm-wide">
<div class="fm-tree">
<div class="fm-root"><div><div class="fm-k">6420 Meals and entertainment · 2026-03-14</div><div class="fm-what">Entry 512 · client dinner</div></div><div class="fm-amt">186.40</div></div>
<div class="fm-node"><span class="fm-rel">evidences</span><span class="fm-what">Card line, row 37</span><span class="fm-where">activity-2026-03-15.csv · agent:bank</span>
<div class="fm-node"><span class="fm-rel">same-event · 0.98</span><span class="fm-what">Receipt 186.40, tip 19.20</span><span class="fm-where">IMG_2041.jpg · p1 · you, phone</span></div>
<div class="fm-node"><span class="fm-rel">same-event · fingerprint</span><span class="fm-what">Statement line, page 2</span><span class="fm-where">statement-2026-03.pdf · p2 box(36,412,540,14)</span></div>
</div>
</div>
</div>

The same question as a Datalog query:

```clojure
;; every file and place that backs one posting
[:find ?kind ?file ?page ?box
 :in $ ?posting
 :where
 [?posting :posting/entry ?entry]
 [?l :link/kind :evidences]
 [?l :link/b ?entry] [?l :link/a ?seed]
 (same-event* ?seed ?obs)          ; a rule: follow same-event links
 [?obs :obs/region ?r]
 [?r :region/doc ?d] [?r :region/page ?page]
 [(get-else $ ?r :region/box nil) ?box]
 [?d :doc/kind ?kind] [?d :doc/files ?file]]
```

Because every observation is pinned to a region, the terminal UI can open the statement at page 2 and draw a box around the line. The same works for a check:

```clojure
;; who signed check 1043?
[:find ?signer ?conf ?file ?box
 :where
 [?d :doc/kind :check] [?d :doc/external-id "1043"]
 [?r :region/doc ?d] [?o :obs/region ?r]
 [?o :check/signer ?signer]
 [?o :obs/confidence ?conf]
 [?o :obs/located [:check/signer ?box]]
 [?d :doc/files ?file]]
```

<div class="fm-answer"><span class="fm-k">1 result</span><span><strong>R. Calder</strong>, read with confidence 0.91 by agent:extract (check@1)</span><span class="fm-mono">check-1043-front.png · box(380,210,220,40)</span></div>

If you correct the name, that's a new fact from you. The extractor's reading stays in the history.

## One bank line, four witnesses

The same movement of money is reported several times, at different speeds and with different detail. Each report stays its own observation. None of them is the entry; they're evidence for it.

<div class="fm-wide">
<div class="fm-sources">
<div><div class="fm-h">Activity download</div><dl><dt>Arrives</dt><dd>Daily, CSV</dd><dt>Identity</dt><dd>None given; fingerprint</dd><dt>Region</dt><dd>Row number</dd></dl></div>
<div><div class="fm-h">OFX / QFX</div><dl><dt>Arrives</dt><dd>On demand</dd><dt>Identity</dt><dd>FITID, sometimes reissued</dd><dt>Region</dt><dd>Element path</dd></dl></div>
<div><div class="fm-h">Statement PDF</div><dl><dt>Arrives</dt><dd>Monthly</dd><dt>Identity</dt><dd>Account and period</dd><dt>Region</dt><dd>Page and box</dd></dl></div>
<div><div class="fm-h">Provider bank feed</div><dl><dt>Arrives</dt><dd>With each sync</dd><dt>Identity</dt><dd>The provider's line id</dd><dt>Region</dt><dd>JSON pointer</dd></dl></div>
</div>
<div class="fm-merge"><strong>One entry.</strong> Link rules join the witnesses with <code>:same-event</code>. A witness with no partner is a question for you, never a silent merge.</div>
</div>

A statement's opening and closing balances are facts too, so every statement checks itself with the same rule ownpurse already uses for Xero's trial balance:

<div class="fm-checks">
<div><span class="fm-ok">Balanced</span><span>opening −2,317.62 + 41 lines −586.56 = closing −2,904.18</span></div>
<div><span class="fm-ok">Balanced</span><span>ledger 2100 Card 4417 on 2026-03-31 = −2,904.18</span></div>
<div><span class="fm-short">Short 12.00</span><span>the daily downloads hold 40 lines and the statement holds 41: p3 box(36,302,540,14) has no partner</span></div>
</div>

## Choosing the engine

> **Aram:** I would like to see if we can find a database that exists that fits this, rather than build our own.

**Tally:** So we went looking. The constraints were one runtime, one executable, a fast start, exact decimals, history you can query as of any moment, and a project that's still maintained.

| Engine | What fits | What doesn't |
|---|---|---|
| Datomic | Exactly the model | JVM, closed source, no public client protocol; clauses run in the order written |
| XTDB v2 | Bitemporal, columnar, a cost-based planner, XTQL, reachable from Rust over the Postgres protocol | JVM; in-process use not yet stable; rows are document versions, not datoms |
| Datahike | Datomic-style, active | Clojure on the JVM |
| Datalevin | The strongest Datalog optimizer we found | Keeps no history by design |
| Cozo | Rust, Datalog, SQLite storage, as-of queries | Quiet since 2024; no decimal type |
| Minigraf | Rust, bitemporal Datalog, one file | Months old; slow at a million facts in its own benchmarks |
| Oxigraph | Rust RDF and SPARQL, exact decimals | No time model |
| SQLite | Mature, instant to open, already holds the record | No Datalog; no decimal type |

> **Aram:** What if we open up the requirements and go with Datomic or XTDB, even if we have to run a server? My fear is that without a robust enough query engine, I don't know how we deal with the graph nature of this if we start to slow down on queries with SQL.

**Tally:** I listed the queries ownpurse actually runs: a trial balance by period, account rollups, a drill from a figure to its evidence, reconciliation, settlement chains, provenance, intercompany pairs, as-of versions of all of them, and whatever an agent asks. Accounting turns out to be a shallow graph with heavy sums. Hops are short and bounded. The expensive part is adding up millions of postings, and that's where Datomic is weakest: it builds the whole result before it aggregates.

I also got one thing wrong, and Aram caught it. I said XTDB v2 no longer speaks Datalog. It speaks SQL and XTQL, its own language, which keeps Datalog's central idea (joins by repeating a logic variable inside `unify`) with a real planner behind it. That makes XTDB the stronger server candidate of the two.

The decision we landed on doesn't need to pick one yet:

> **Aram:** I like what you're saying about using SQLite as the source and then building on top of that.

The log is ours, in SQLite: append-only by trigger, hash-chained, the file you hold. Indexes and running totals are a cache beside it. Any heavier engine, XTDB or Datomic included, would be a projection of that log, filled by replaying it. So choosing one later isn't a migration, and we can measure before we decide. The benchmark runs the queries above on a synthetic record of ten million facts, against SQLite with indexes and against XTDB, and the numbers settle it.

## What we decided

- **The atom is a fact** stamped by a transaction, and provenance lives on the transaction.
- **The drawer keeps every file** byte for byte, named by its hash. By default it lives in your home folder beside the record (`~/.local/share/ownpurse/<profile>/drawer/`), unencrypted, and a config key moves it.
- **Every connector writes to the drawer**; projectors write facts. Xero is one connector among many.
- **Regions are as exact as the format allows for free**: a row or a pointer for structured files, at least the page for PDFs and photos, and a box when the reader finds one.
- **One observation per item**, with shared attributes for date, amount and party, and typed extras such as `:check/signer`.
- **Entries are held until posted.** Posting rules are a setting.
- **Amounts carry their commodity** from day one: dollars, shares and coins alike.
- **Every book names its authority.** While Xero or QuickBooks owns a book, ownpurse mirrors it. When you're ready, the authority becomes ownpurse, in one fact.
- **SQLite holds the log**, and any other engine is a projection of it.

## What it allows

Taking financial data down to the atom is what lets it be built back up in any shape. A trial balance, a household budget, a card reconciliation and "who signed that check" are all questions asked of the same facts. A new source is one more projector: an agent reads your spreadsheet, proposes which columns mean what, and the mapping itself becomes a reviewable fact. A correction is never lost, a proposal never touches your books until you say so, and every figure leads back to the paper it came from.

And when you're ready to stop mirroring and keep your books here, nothing has to move. The facts are already yours.

The full plan, with every attribute, the migration from today's record and the phases, is in [`docs/plans/FACTS.md`](https://github.com/brancusi/ownpurse/blob/main/docs/plans/FACTS.md).

<p class="fm-fine">ownpurse is not affiliated with Xero Limited, Nubank, Cognitect or JUXT. Datomic is a trademark of its owner. Every name, amount and id on this page is invented.</p>
