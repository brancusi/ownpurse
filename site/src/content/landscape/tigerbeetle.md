---
{
 "slug": "tigerbeetle",
 "name": "TigerBeetle",
 "url": "https://tigerbeetle.com",
 "segment": "fact-ledger",
 "status": "active",
 "founded": "2020 (project, July 2020); TigerBeetle Inc. founded 29 August 2022 as a spin-out of Coil",
 "hq": "Not stated on the company page; distributed team",
 "backing": "Seed $6.4M led by Amplify with Coil (25 January 2023, announced 30 January 2023); Series A $24M led by Spark Capital with Amplify and Coil (30 May 2024); over $30M in total per the company page (checked 2026-10-10). Angels include Alexander Gallego (Redpanda), Derek Collison (Synadia) and Andrew Kelley (Zig Software Foundation).",
 "tagline": "An open-source database built only for double-entry transfers between accounts, designed for high throughput and strict safety.",
 "scope": "TigerBeetle stores accounts and transfers as immutable debit/credit records with balances kept by the database, two-phase (pending, post, void) transfers, balance limits and per-ledger partitioning. It is a data-plane OLTP engine meant to sit beside a general-purpose database that holds names, metadata and everything else. It has no connectors, no chart of accounts, no reporting and no UI.",
 "domains": [
  "ledger"
 ],
 "customers_served": [
  "developers",
  "enterprise",
  "mid-market"
 ],
 "spectrum": {
  "custody": {
   "score": 4,
   "note": "Apache 2.0 and self-hosted by default: you run the cluster and hold the data files; the managed cloud (August 2026) promises full export through the open protocol."
  },
  "locality": {
   "score": 3,
   "note": "A single statically linked binary runs on Linux, macOS and Windows, including a one-replica development mode on a laptop; production means a six-replica cluster."
  },
  "history": {
   "score": 3,
   "note": "Accounts and transfers are immutable, corrections are new transfers, and the history flag keeps balances at every transfer; there are no general as-of queries over arbitrary state."
  },
  "openness": {
   "score": 4,
   "note": "Apache 2.0, no CLA, the whole database is in the open repository."
  },
  "breadth": {
   "score": 0,
   "note": "One domain only: transfers between accounts, as integers on a ledger."
  },
  "autonomy": {
   "score": 1,
   "note": "It enforces invariants but decides nothing; your application code writes every transfer."
  },
  "agent_readiness": {
   "score": 2,
   "note": "Documented client libraries and a REPL; no MCP server, agent docs or scoped tokens found."
  },
  "integration_ease": {
   "score": 3,
   "note": "Thorough docs, seven client libraries and a single binary, but a narrow data model you must pair with another database."
  },
  "market": {
   "score": 3,
   "note": "Sold to payment companies, brokerages, energy and core banking; cloud plans start at $10,240 a year per cluster."
  }
 },
 "api": {
  "public": true,
  "style": "Custom binary protocol over TCP through official clients (.NET, Go, Java, Node.js, Python, Ruby, Rust); batched requests of up to 8,189 events",
  "auth": "None built in at the protocol level in the open-source build [unverified]; cloud runs on single-tenant private networking",
  "limits": "Batch size up to 8,189 transfers per request; queries limited to indexed fields (user_data, ledger, code, timestamps) through QueryFilter; no SQL",
  "cost": "Self-hosted: free. Cloud (2026-10-10): Pro $10,240/yr incl. 64M transfers; Startup $40,960/yr incl. 1B; Growth $163,840/yr incl. 8B; Enterprise $655,360/yr incl. 64B; overage per million transfers and per lookup or query",
  "export": "All data through the open protocol API at any time; change data capture streams every transfer and resulting balances as JSON over AMQP 0.9.1 (at-least-once)",
  "notes": "The CDC job requires cluster 0.16.43 or later. Amounts are unsigned 128-bit integers with an application-chosen asset scale; changing scale means a new ledger."
 },
 "pricing": {
  "model": "Open source free to self-host; managed cloud priced per cluster per year with included transfers and metered overage",
  "entry": "$0 self-hosted; cloud Pro $10,240 a year",
  "top": "Enterprise $655,360 a year (64 billion transfers included); custom beyond that",
  "notes": "The cloud page says no custom agreements and no discounts; multi-region and multi-cloud add 28 to 40 percent; 99.99% SLA with service credits up to 100 percent. No on-premises or customer-cloud deployment of the managed offer."
 },
 "agents": {
  "mcp": "none",
  "ai_features": "None. The product has no AI features.",
  "agent_writes": "Only through client libraries, as any program would",
  "approval": "None; no human gate exists at the database level (two-phase transfers can serve as one if your application builds it)",
  "notes": "Pending transfers that are later posted or voided are the closest primitive to a proposal waiting for approval."
 },
 "integration": {
  "scope": "No connectors to banks, PSPs or accounting systems. Seven official client libraries; CDC to RabbitMQ and other AMQP brokers.",
  "notes": "Designed to sit in the hot path beside an OLGP database (Postgres and the like) that holds metadata and control-plane state."
 },
 "preference": {
  "scale": "17,159 GitHub stars and 917 forks (2026-10-10); production customers passed 100M transactions a month on 1 January 2025; a trillion-transaction scale test on 19 March 2026 (company page)",
  "ratings": "No G2, Capterra or Trustpilot listings found (G2 blocked the fetch); not rated",
  "sentiment": "Engineers praise the safety engineering, deterministic simulation testing and the Jepsen result (June 2025: no strong-serializability violations from 0.16.26). Jepsen's criticisms: latency could rise by orders of magnitude after one node failure (mitigated in 0.16.43) and clients retry indefinitely without surfacing definite errors (unresolved). The narrow data model is the usual objection."
 },
 "timeline": [
  {
   "date": "2020-07",
   "event": "Created while consulting on a central bank switch for the Gates Foundation (Mojaloop)"
  },
  {
   "date": "2022-08-29",
   "event": "TigerBeetle Inc. founded, spun out of Coil"
  },
  {
   "date": "2023-01-30",
   "event": "$6.4M seed led by Amplify announced"
  },
  {
   "date": "2024-03",
   "event": "0.15.3 released for production"
  },
  {
   "date": "2024-05-30",
   "event": "$24M Series A led by Spark Capital"
  },
  {
   "date": "2025-01-01",
   "event": "Production customers reach 100M transactions a month"
  },
  {
   "date": "2025-06-06",
   "event": "Jepsen publishes its analysis of 0.16.11 to 0.16.30"
  },
  {
   "date": "2025-10-25",
   "event": "With Synadia, pledges $512,000 to the Zig Software Foundation"
  },
  {
   "date": "2026-03-19",
   "event": "Reports a trillion transactions in a scale test"
  },
  {
   "date": "2026-07-06",
   "event": "Release 0.17.9"
  },
  {
   "date": "2026-08-26",
   "event": "TigerBeetle Cloud launches"
  }
 ],
 "relation": "design-cousin",
 "relation_note": "A purpose-built, append-only double-entry store with the same instinct as ownpurse's record (never edit, correct with a new entry), but for a payment system's hot path, not for an owner's books gathered from several sources.",
 "borrow": "The id as idempotency key, so a retried write never applies twice; reversals as new transfers; pending, posted and voided transfers as a recorded uncollapsed state; the imported flag (original timestamps, unique, strictly increasing, in a linked batch); a separate field for real-world time beside the database's own timestamp; deterministic simulation testing; CDC consumers that must be idempotent.",
 "avoid": "Its narrowness: integers and opaque user_data only, no strings or provenance, no general queries. Also correct the earlier survey: TigerBeetle ids are client-generated and time-ordered (48-bit millisecond timestamp, 80 random bits), not derived from content.",
 "sources": [
  {
   "title": "TigerBeetle home page",
   "url": "https://tigerbeetle.com"
  },
  {
   "title": "TigerBeetle company page (history, funding)",
   "url": "https://tigerbeetle.com/company"
  },
  {
   "title": "TigerBeetle Cloud plans and pricing",
   "url": "https://tigerbeetle.com/cloud"
  },
  {
   "title": "Data modeling (ids, immutability, user_data)",
   "url": "https://docs.tigerbeetle.com/coding/data-modeling/"
  },
  {
   "title": "Account reference (history and imported flags)",
   "url": "https://docs.tigerbeetle.com/reference/account/"
  },
  {
   "title": "Change data capture",
   "url": "https://docs.tigerbeetle.com/operating/cdc/"
  },
  {
   "title": "Single-page docs (clients, reversals, local run)",
   "url": "https://docs.tigerbeetle.com/single-page/"
  },
  {
   "title": "Seed announcement, 30 January 2023",
   "url": "https://tigerbeetle.com/blog/2023-01-30-series-seed-announcement"
  },
  {
   "title": "Jepsen: TigerBeetle 0.16.11",
   "url": "https://jepsen.io/analyses/tigerbeetle-0.16.11"
  },
  {
   "title": "GitHub repository",
   "url": "https://github.com/tigerbeetle/tigerbeetle"
  }
 ],
 "unverified": [
  "Headquarters location (not stated by the company)",
  "Whether the open-source build has any protocol-level authentication",
  "Throughput figures (100K-500K TPS, 1000x) are vendor claims",
  "Series A date of 30 May 2024 is from the company page; no separate announcement post was read",
  "No review-site ratings could be checked (G2 returned 403); web search was unavailable for press coverage"
 ],
 "researched": "2026-10-10"
}
---

## What it is

TigerBeetle calls itself "the financial transactions database designed for mission critical safety and performance" ([site](https://tigerbeetle.com)). It does one thing: debit/credit transfers between accounts, with balances maintained by the database. It is written in Zig and licensed Apache 2.0 with no CLA ([GitHub](https://github.com/tigerbeetle/tigerbeetle)).

What it does well is narrow and deep: strict serializability by default, a replicated write-ahead log with end-to-end checksums, recovery from storage faults, and throughput the company puts at 100,000 to 500,000 transfers a second (vendor figures). What it does not do: names, descriptions, connectors, reports, a chart of accounts or a UI. The docs say it belongs in the data plane, with a general-purpose database beside it for everything else.

## How it works

Accounts and transfers are fixed-size records and "immutable once created" ([data modeling](https://docs.tigerbeetle.com/coding/data-modeling/)). Amounts are unsigned 128-bit integers at a scale you choose per ledger. A correction is a new transfer: "Reversals are implemented with separate transfers to provide a full and auditable log of business events." Two-phase transfers reserve funds as pending and later post or void them.

Ids are generated by the client with a time-based scheme (48 bits of milliseconds, 80 random bits), and the id is the idempotency key: a retried request never moves money twice. Note that this is not a content-derived id, as the earlier ownpurse survey said. Three `user_data` fields are indexed; the docs suggest one for a real-world "when", which gives you a second time beside TigerBeetle's own timestamp. The `history` account flag keeps the balance at every transfer for `get_account_balances`, and the `imported` flag lets you load past events with their original timestamps, unique and strictly increasing ([account reference](https://docs.tigerbeetle.com/reference/account/)).

## Data and access

You reach it through official clients for .NET, Go, Java, Node.js, Python, Ruby and Rust. Queries are limited to indexed fields; there is no SQL. Change data capture publishes each transfer and the resulting balances as JSON over AMQP, at least once, so consumers must deduplicate ([CDC](https://docs.tigerbeetle.com/operating/cdc/)).

Self-hosting is free. TigerBeetle Cloud, launched 26 August 2026, is single-tenant and priced per cluster per year: Pro $10,240 (64 million transfers) up to Enterprise $655,360 (64 billion), with no discounts and full export through the open protocol ([cloud pricing](https://tigerbeetle.com/cloud)).

## Agents

There is no MCP server, no agent documentation and no AI feature. An agent can drive it like any program through a client library. Pending transfers are the nearest thing to a proposal waiting for a person, but any approval step is yours to build.

## Customers

Testimonials name Clear Street, Triple-A, Fynbos, Super Payments and Pesawise; the company cites energy, core banking and high-volume trading ([company](https://tigerbeetle.com/company)). The repository has about 17,200 stars (10 October 2026). There are no review-site listings. Jepsen's June 2025 analysis found no strong-serializability violations from 0.16.26 onward, and called out availability after a single node failure (since mitigated) and clients that retry forever without a definite error (unresolved) ([Jepsen](https://jepsen.io/analyses/tigerbeetle-0.16.11)).

## History

Joran Dirk Greef started TigerBeetle in July 2020 while working on a central bank switch for the Gates Foundation. It spun out of Coil as TigerBeetle Inc. on 29 August 2022, raised a $6.4 million seed led by Amplify in January 2023 and a $24 million Series A led by Spark Capital in May 2024. Version 0.15.3 went to production in March 2024. In October 2025 it and Synadia pledged $512,000 to the Zig Software Foundation ([blog](https://tigerbeetle.com/blog/)).

## For ownpurse

TigerBeetle is a design cousin, not a competitor. It shares ownpurse's first rule (never edit; correct with a new entry) and proves that rule can run at scale, but it holds a payment system's transfers, not an owner's books gathered from Xero, banks and receipts. Worth borrowing: the id as idempotency key, the imported flag's rules for loading the past, two times per event, pending transfers as a recorded uncollapsed state, and its simulation testing. Worth avoiding: a model so narrow that provenance, raw source attributes and text have to live in another database. ownpurse's record keeps those in the same log, which is the point.
