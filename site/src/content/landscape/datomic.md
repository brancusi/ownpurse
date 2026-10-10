---
{
 "slug": "datomic",
 "name": "Datomic",
 "url": "https://www.datomic.com",
 "segment": "fact-ledger",
 "status": "active",
 "founded": "2012 (launch; exact month unverified)",
 "hq": "Cognitect, Inc. (US); parent Nu Holdings, São Paulo",
 "backing": "Owned by Nubank (Nu Holdings) since Cognitect joined Nubank on 23 July 2020; Nubank had used Datomic since 2014. Free of licence fees since 27 April 2023.",
 "tagline": "An immutable database of facts (datoms) with the database as a value and every past state queryable.",
 "scope": "Datomic stores facts as five-tuples of entity, attribute, value, transaction and an added/retracted flag, never overwrites them, and lets you query any point in time. It is general-purpose infrastructure: it has no accounts, ledgers, connectors or financial semantics, and it does no bookkeeping itself.",
 "domains": [
  "ledger"
 ],
 "customers_served": [
  "developers",
  "enterprise"
 ],
 "spectrum": {
  "custody": {
   "score": 4,
   "note": "You run it on your own storage or in your own AWS account; no vendor holds the data."
  },
  "locality": {
   "score": 4,
   "note": "Datomic Local runs in-process and stores each database as a directory on your filesystem."
  },
  "history": {
   "score": 4,
   "note": "Append-only datoms with retraction as a new fact; as-of and history views are built in; excision exists only for privacy and retention."
  },
  "openness": {
   "score": 3,
   "note": "Binaries under Apache 2.0 and free, but Datomic says itself it is not open source: the source is closed."
  },
  "breadth": {
   "score": 0,
   "note": "A database, not a finance product; no sources or domains of its own."
  },
  "autonomy": {
   "score": 0,
   "note": "Not applicable: it does no accounting work; your code writes whatever it writes."
  },
  "agent_readiness": {
   "score": 1,
   "note": "Clojure and Java client APIs only; no MCP server, agent docs or SQL interface from the vendor."
  },
  "integration_ease": {
   "score": 2,
   "note": "Free from Maven Central with no signup, but JVM and Clojure centred, with a transactor and storage service to run for Pro."
  },
  "market": {
   "score": 3,
   "note": "Developers and large production users such as Nubank and Netflix; no small-business offering."
  }
 },
 "api": {
  "public": true,
  "style": "Embedded library and Client API (Clojure, Java); Datalog queries and Pull; Pro uses peers and a transactor over pluggable storage",
  "auth": "None for Local; Cloud uses AWS IAM; Pro storage credentials as configured",
  "limits": "Datomic Local: about 32 bytes of JVM heap per datom, strings up to 4096 characters, a per-transaction datom cap; Local is aimed at small, single-process applications",
  "cost": "Free of licence fees for all editions (Pro, Cloud, Local). Cloud costs only your own AWS infrastructure; paid enterprise support on request",
  "export": "You own the storage; Local databases back up by copying a directory; the full log of transactions can be read through the API",
  "notes": "Storage for Pro: in-memory, dev (embedded H2 files), PostgreSQL, MySQL, Oracle, DynamoDB, Cassandra 3.x. Jepsen (May 2024) found every history Serializable but intra-transaction semantics concurrent rather than serial."
 },
 "pricing": {
  "model": "Free binaries; paid support",
  "entry": "Free",
  "top": "Enterprise support, price on request",
  "notes": "Before April 2023 Datomic Pro and Cloud were sold under paid licences; Cloud users on version 995-9204 or later pay no licence fees."
 },
 "agents": {
  "mcp": "none",
  "ai_features": "None from the vendor as of October 2026; the blog and product pages make no mention of AI or MCP.",
  "agent_writes": "Only through code you write against the Client API.",
  "approval": "None built in; transactions commit when submitted. Entity predicates and transaction functions can enforce invariants.",
  "notes": "Its value to agents is the model: a database as an immutable value gives an agent a stable snapshot to reason over, and reified transactions can carry provenance."
 },
 "integration": {
  "scope": "No connectors. Integrates with JVM languages, and via Pro storage with PostgreSQL, MySQL, Oracle, DynamoDB and Cassandra.",
  "notes": "Transactions are entities, so change data capture is a query over the log rather than a separate feed."
 },
 "preference": {
  "scale": "Nubank, its main user, cited nearly 94 million users and 2.3 billion user transactions a day in Jepsen's May 2024 report; no public count of Datomic installations.",
  "ratings": "No G2, Capterra or Trustpilot listings found.",
  "sentiment": "Admired by developers for the data model, time travel and the database-as-a-value idea; common complaints are the closed source, the JVM and Clojure orientation and the operational weight of Pro (transactor plus storage). Not checked against review sites in this pass."
 },
 "timeline": [
  {
   "date": "2012",
   "event": "Datomic launched by the team around Rich Hickey (exact month unverified)"
  },
  {
   "date": "2014",
   "event": "Nubank adopts Datomic as its primary transactional database"
  },
  {
   "date": "2020-07-23",
   "event": "Cognitect, Datomic's maker, joins Nubank"
  },
  {
   "date": "2023-04-27",
   "event": "All Datomic editions become free of licence fees; binaries under Apache 2.0 on Maven Central"
  },
  {
   "date": "2023-06-16",
   "event": "Datomic Cloud also free, binaries under Apache 2.0"
  },
  {
   "date": "2023-08-16",
   "event": "Datomic Local released: embeddable, redistributable, Apache 2.0"
  },
  {
   "date": "2024-05-15",
   "event": "Jepsen publishes its analysis of Datomic Pro 1.0.7075; Datomic revises its safety docs"
  },
  {
   "date": "2026-09-02",
   "event": "DatomicConf 2026 announced for Durham, North Carolina"
  }
 ],
 "relation": "design-cousin",
 "relation_note": "ownpurse's log follows Datomic's datom: entity, attribute, value, transaction, added. ownpurse adds provenance (which source said it), observed time and a hash chain, and keeps it in SQLite instead of a JVM database.",
 "borrow": "The five-tuple datom with retraction as a fact; the database as an immutable value so every view reads a stable snapshot; reified transactions that carry who and why; as-of, since and history views; excision kept separate from correction and recorded in the log itself.",
 "avoid": "A runtime that ties the record to one language and platform; closed source for the thing a person must trust; Datomic's concurrent intra-transaction semantics, where two checks in one transaction can both pass and leave an invariant broken (Jepsen's approve-and-deny case).",
 "sources": [
  {
   "title": "Datomic",
   "url": "https://www.datomic.com"
  },
  {
   "title": "Datomic is free (27 April 2023)",
   "url": "https://blog.datomic.com/2023/04/datomic-is-free.html"
  },
  {
   "title": "Datomic Cloud is free (16 June 2023)",
   "url": "https://blog.datomic.com/2023/06/datomic-cloud-is-free.html"
  },
  {
   "title": "Nubank: Datomic is available free of licensing fees",
   "url": "https://building.nu.com/datomic-is-available-free-of-licensing-fees/"
  },
  {
   "title": "Datomic data model",
   "url": "https://docs.datomic.com/whatis/data-model.html"
  },
  {
   "title": "Datomic excision",
   "url": "https://docs.datomic.com/reference/excision.html"
  },
  {
   "title": "Datomic storage services",
   "url": "https://docs.datomic.com/operation/storage.html"
  },
  {
   "title": "Datomic Local",
   "url": "https://docs.datomic.com/datomic-local.html"
  },
  {
   "title": "Jepsen: Datomic Pro 1.0.7075 (May 2024)",
   "url": "https://jepsen.io/analyses/datomic-pro-1.0.7075"
  },
  {
   "title": "Cognitect joins Nubank (23 July 2020)",
   "url": "https://www.cognitect.com/blog/2020/07/23/Cognitect-Joins-Nubank"
  },
  {
   "title": "Datomic blog",
   "url": "https://blog.datomic.com/"
  }
 ],
 "unverified": [
  "Launch year 2012 and month (not confirmed from a primary source in this pass)",
  "Netflix's production use (from the blog listing title; the Conj 2025 recap page returned 403)",
  "Price of Datomic enterprise support",
  "Absence of any community Datomic MCP server (web search budget was exhausted, so not searched)",
  "Review-site and Reddit sentiment (not checked this pass; summary is general developer opinion)",
  "Cognitect's HQ location"
 ],
 "researched": "2026-10-10"
}
---

## What it is

[Datomic](https://www.datomic.com) is a database built on immutable facts. Its unit is the datom, "a five-tuple" of entity, attribute, value, transaction and an operation flag that says whether the fact was added or retracted ([data model](https://docs.datomic.com/whatis/data-model.html)). A database is a set of datoms, and "a db is a point-in-time, immutable value and will never change." Nubank owns it and runs its bank on it.

## How it works

Nothing is overwritten. Removing a fact means adding a datom with the flag set to false; the original assertion stays. That gives you as-of views (the database as it was at a transaction), history views (every assertion and retraction) and comparisons between points in time. Transactions are themselves entities, so you can attach who made a change and why, and query the log as change data capture.

There are three editions. Datomic Pro runs a transactor against pluggable storage: PostgreSQL, MySQL, Oracle, DynamoDB, Cassandra, or local dev files ([storage](https://docs.datomic.com/operation/storage.html)). Datomic Cloud runs in your own AWS account. [Datomic Local](https://docs.datomic.com/datomic-local.html) runs in-process and keeps each database as a directory on disk, "best suited for small, single-process applications."

The one escape hatch is [excision](https://docs.datomic.com/reference/excision.html): permanent removal of datoms for privacy or retention. Datomic says plainly that excision "should never be used to correct erroneous data"; use retraction. Excision itself leaves a trace you can query.

[Jepsen's 2024 analysis](https://jepsen.io/analyses/datomic-pro-1.0.7075) found every history Serializable, stronger than claimed, but showed that operations inside one transaction run as if concurrent, so two transaction functions that each check a precondition can together break an invariant.

## Data and access

On 27 April 2023 Nubank made [every edition free](https://blog.datomic.com/2023/04/datomic-is-free.html), with binaries under Apache 2.0 on Maven Central and no signup. Datomic is clear that this is not open source: the binaries are licensed, the source is not published. Cloud followed in [June 2023](https://blog.datomic.com/2023/06/datomic-cloud-is-free.html). Paid enterprise support remains. Because you run it, you hold the data, and a Local database backs up by copying a folder.

Access is through Clojure and Java APIs with Datalog queries. There is no SQL surface and nothing outside the JVM.

## Agents

Datomic has no MCP server and no AI features as of October 2026; its [blog](https://blog.datomic.com/) covers conferences, a newsletter and Jepsen, not agents. What it offers agents is the model: a stable snapshot to read and a log where each change can say who made it.

## Customers

Nubank calls it its "primary transactional database" ([Nubank](https://building.nu.com/datomic-is-available-free-of-licensing-fees/)); Jepsen's report cites nearly 94 million users and 2.3 billion user transactions a day. Netflix spoke about production use at Clojure/Conj 2025. It is a developer's tool: there are no review-site listings to speak of.

## History

Datomic launched in 2012. [Cognitect joined Nubank](https://www.cognitect.com/blog/2020/07/23/Cognitect-Joins-Nubank) on 23 July 2020, after six years in which Nubank had built on it. The licence fees went in 2023 so that small businesses, startups and individual developers "who may have been put off by the cost" could use it. Datomic Local followed in August 2023.

## For ownpurse

Datomic is the model ownpurse's log follows, and the main lesson is that the model works at bank scale: append-only facts, retraction instead of deletion, and the database as a value are enough to run a bank of 94 million users. ownpurse takes the datom and adds what a person's books need: which source said it, when it was observed, and a hash chain that proves nothing changed. It keeps the record in SQLite, a file anyone can open, rather than behind a JVM and closed source. And Jepsen's finding is a warning for ownpurse's own link rules and approval checks: run checks against the state they will commit, not against the state at the start of the batch.
