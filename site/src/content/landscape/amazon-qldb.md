---
{
 "slug": "amazon-qldb",
 "name": "Amazon QLDB",
 "url": "https://aws.amazon.com/qldb/",
 "segment": "fact-ledger",
 "status": "shut",
 "founded": "2018 (announced at re:Invent 2018 as a preview; generally available 10 September 2019)",
 "hq": "Seattle, Washington, USA (Amazon Web Services)",
 "backing": "A service of Amazon Web Services, part of Amazon.com, Inc. (NASDAQ: AMZN). No separate funding. Closed to new customers 19 July 2024; support ended 31 July 2025.",
 "tagline": "A hosted, append-only, hash-chained ledger database from AWS, now shut down.",
 "scope": "QLDB was a managed document database whose journal was an append-only chain of cryptographically linked blocks, with SQL-like queries (PartiQL), queryable revision history and a digest you could use to verify that nothing had been changed. It held no accounting model, connected to no financial sources and did no bookkeeping: it was infrastructure that fintech and supply-chain teams built ledgers on. It is gone. AWS closed it to new customers in July 2024 and ended support on 31 July 2025; the product URL now redirects to Amazon Aurora.",
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
   "score": 1,
   "note": "AWS held the authoritative journal; an owner could export it to S3 and save digests, but the ledger itself was deleted when the service ended."
  },
  "locality": {
   "score": 0,
   "note": "Cloud only, inside an AWS account; never self-hostable or local."
  },
  "history": {
   "score": 4,
   "note": "Append-only journal, every document revision kept, history() queries over time ranges, and digest-based verification."
  },
  "openness": {
   "score": 2,
   "note": "Closed service, but a public API, Apache-2.0 drivers, open formats (Ion, PartiQL) and full journal export to S3."
  },
  "breadth": {
   "score": 0,
   "note": "A general database; no books, banks, documents or payments of its own."
  },
  "autonomy": {
   "score": 0,
   "note": "Did no accounting work; anything written to it was written by the customer's own code."
  },
  "agent_readiness": {
   "score": 0,
   "note": "Shut down in July 2025, so there is no API to call; while live it was a plain public API (about a 2), with no MCP or agent features."
  },
  "integration_ease": {
   "score": 0,
   "note": "Cannot be connected to today; while live it needed an AWS account, IAM, a QLDB driver and Amazon Ion."
  },
  "market": {
   "score": 3,
   "note": "Sold to engineering teams at fintechs, retailers and enterprises (Earnin, Specright, Shinsegae International in AWS case studies), not to small businesses directly."
  }
 },
 "api": {
  "public": true,
  "style": "AWS control-plane API (create ledger, GetDigest, GetBlock, GetRevision, ExportJournalToS3, StreamJournalToKinesis) plus a session API used through language drivers to run PartiQL transactions over Amazon Ion documents. No longer callable since 31 July 2025.",
  "auth": "AWS IAM (SigV4-signed requests, IAM policies, optional PartiQL-level permissions).",
  "limits": "Not confirmed from a primary source; the developer guide that listed quotas has been removed.",
  "cost": "Usage-based: write I/Os, read I/Os, journal storage, indexed storage and data transfer were each metered (factors confirmed in the 2019 launch post). Exact dollar rates not confirmed; the pricing page now redirects to Aurora pricing.",
  "export": "The full journal could be exported to S3 (block by block, with hashes) and streamed continuously to Kinesis Data Streams. AWS's migration sample used both to move ledgers into Aurora PostgreSQL.",
  "notes": "Verification worked by fetching a 256-bit digest that covered the ledger's entire history at a point in time, then requesting proofs for a block or revision and recomputing hashes. The guarantee was only as independent as the place the customer kept the digest. Drivers were open source under Apache-2.0; the Node.js driver repository was archived on 3 December 2025."
 },
 "pricing": {
  "model": "Pay per use: I/O requests, storage and transfer, no instances or fixed fee.",
  "entry": "No minimum; billed from the first request (exact rates unverified).",
  "top": "Uncapped usage billing; no enterprise tier of its own.",
  "notes": "The service no longer exists, so there is nothing to buy. AWS points former customers to Aurora PostgreSQL, which is priced by instance or serverless capacity."
 },
 "agents": {
  "mcp": "none",
  "ai_features": "None. QLDB predated the agent era and was retired before AWS shipped MCP servers for its databases.",
  "agent_writes": "Not applicable now. While live, any client with IAM credentials and a driver could write transactions; there was no agent-specific surface.",
  "approval": "None. QLDB recorded every write immutably but had no notion of a human approving a write before it was committed.",
  "notes": "Its value to an agent would have been as a tamper-evident store an agent could read and prove against; that value disappeared with the service."
 },
 "integration": {
  "scope": "Inside AWS only: drivers for Java, .NET, Go, Node.js and Python; journal export to S3; streaming to Kinesis; Glue and Athena for analysis of exported journals. No financial-source connectors.",
  "notes": "Integration meant writing application code against the driver and modelling data as Ion documents. Leaving it meant a two-phase migration (S3 export through Glue and DMS for the full load, Kinesis plus Lambda for ongoing changes) that AWS's own sample does not verify by digest; it checks document counts."
 },
 "preference": {
  "scale": "No usage or revenue figures were published. AWS published at least four customer stories (Specright, January 2022; BungkusIT, April 2022; Shinsegae International, August 2022; Earnin, February 2023).",
  "ratings": "No G2, Capterra or Trustpilot ratings found for QLDB as a product.",
  "sentiment": "Engineers who used it liked it: Corey Quinn called it 'a good database' whose only flaw was being 'a blockchain-adjacent offering' (InfoQ, July 2024). The shutdown was resented: Reddit commenters said customers could lose data-integrity guarantees in migration, and that AWS's habit of not killing services had been a reason to choose it. Noah described its zero-downtime migration (June 2025) as its largest to date, with dual writes and three layers of consistency checks."
 },
 "timeline": [
  {
   "date": "2018-11",
   "event": "Announced at AWS re:Invent 2018 as a preview."
  },
  {
   "date": "2019-09-10",
   "event": "Generally available in five regions (N. Virginia, Ohio, Oregon, Ireland, Tokyo), announced by Jeff Barr."
  },
  {
   "date": "2022-01 to 2023-02",
   "event": "AWS publishes customer stories: Specright, BungkusIT, Shinsegae International, Earnin."
  },
  {
   "date": "2024-07-18",
   "event": "AWS publishes two posts on replacing QLDB with Aurora PostgreSQL for audit use cases and migrating ledgers."
  },
  {
   "date": "2024-07-19",
   "event": "Closed to new customers; existing customers emailed that support ends 31 July 2025. No formal public announcement."
  },
  {
   "date": "2025-06-25",
   "event": "Noah publishes its zero-downtime migration from QLDB to Aurora PostgreSQL Serverless."
  },
  {
   "date": "2025-07-31",
   "event": "End of support; per Classmethod's reading of the notice, the service stops and ledger data is deleted."
  },
  {
   "date": "2025-12-03",
   "event": "The QLDB Node.js driver repository is archived as read-only."
  },
  {
   "date": "2026-10",
   "event": "aws.amazon.com/qldb redirects to Amazon Aurora; the developer guide returns 404."
  }
 ],
 "relation": "cautionary-tale",
 "relation_note": "QLDB is the closest hosted precedent for ownpurse's record: an append-only, hash-chained journal with history and verification. It shows what happens when the vendor holds the chain: the tamper-evidence was real, then the vendor withdrew it, and customers had to rebuild an audit trail in a database that, in AWS's own words, does not keep a permanent, immutable record of changes.",
 "borrow": "The verification model: a digest that commits to the whole history at a moment, proofs for any block or revision, and an exportable journal where each block carries its hash. A first-class history() query over every revision. Streaming the journal out so other systems can follow it.",
 "avoid": "Keeping the only copy of the chain, and the only digest, with the vendor. A proprietary data format (Ion documents in a hosted service) that makes leaving a project. A migration path that checks counts rather than hashes. ownpurse should make the record a file the owner holds and let anyone recompute the chain without the vendor.",
 "sources": [
  {
   "title": "InfoQ: AWS Discontinues Amazon Quantum Ledger Database (QLDB), 26 July 2024",
   "url": "https://www.infoq.com/news/2024/07/aws-kill-qldb"
  },
  {
   "title": "AWS News Blog: Now Available – Amazon Quantum Ledger Database (QLDB), 10 September 2019",
   "url": "https://aws.amazon.com/blogs/aws/now-available-amazon-quantum-ledger-database-qldb/"
  },
  {
   "title": "AWS Database Blog: Amazon QLDB category (posts 2021-2024)",
   "url": "https://aws.amazon.com/blogs/database/category/database/amazon-quantum-ledger-database/"
  },
  {
   "title": "AWS Database Blog: Migrate an Amazon QLDB Ledger to Amazon Aurora PostgreSQL",
   "url": "https://aws.amazon.com/blogs/database/migrate-an-amazon-qldb-ledger-to-amazon-aurora-postgresql/"
  },
  {
   "title": "aws-samples/example-qldb-ledger-migration (GitHub)",
   "url": "https://github.com/aws-samples/example-qldb-ledger-migration"
  },
  {
   "title": "Classmethod: Amazon QLDB end of support on 31 July 2025, 19 July 2024",
   "url": "https://dev.classmethod.jp/articles/amazon-qldb-eos-announcement/"
  },
  {
   "title": "Noah: Zero-downtime migration of Noah's internal ledger, 25 June 2025",
   "url": "https://noah.com/blog/zero-downtime-migration-of-noahs-internal-ledger"
  },
  {
   "title": "awslabs/amazon-qldb-driver-nodejs (archived 3 December 2025)",
   "url": "https://github.com/awslabs/amazon-qldb-driver-nodejs"
  },
  {
   "title": "aws.amazon.com/qldb (now redirects to Amazon Aurora)",
   "url": "https://aws.amazon.com/qldb/"
  }
 ],
 "unverified": [
  "Exact QLDB price rates (recalled as roughly $0.70 per million write I/Os, $0.136 per million read I/Os, $0.03 per GB-month journal storage and $0.25 per GB-month indexed storage in US East); the pricing page now redirects to Aurora and could not be checked.",
  "Service quotas (transaction size, documents per transaction, throughput); the developer guide is removed.",
  "Exact re:Invent 2018 preview date (late November 2018 assumed from InfoQ's 're:Invent 2018').",
  "That all ledger data was deleted after 31 July 2025: taken from Classmethod's reading of the AWS notice, not from an AWS page available today.",
  "Whether journal export supported JSON in addition to Ion text and binary.",
  "Number of QLDB customers or ledgers at shutdown: never published.",
  "The AWS migration posts' dates and author (18 July 2024, Dan Blaner) come from the blog category listing; the post pages themselves no longer render that metadata.",
  "Web search budget ran out during this research, so some facts could not be cross-checked against a second source."
 ],
 "researched": "2026-10-10"
}
---

## What it is

Amazon Quantum Ledger Database (QLDB) was a managed database from AWS whose journal was an append-only sequence of blocks, each cryptographically chained to the one before. It was [announced at re:Invent 2018 and made generally available on 10 September 2019](https://aws.amazon.com/blogs/aws/now-available-amazon-quantum-ledger-database-qldb/), pitched for banking, ecommerce, HR and payroll, and supply chains: anywhere you needed a complete, verifiable history under one trusted authority rather than a blockchain. It is no longer available. AWS [closed it to new customers in July 2024 and ended support on 31 July 2025](https://www.infoq.com/news/2024/07/aws-kill-qldb); the product page now redirects to Amazon Aurora.

## How it works

You stored Amazon Ion documents (a typed superset of JSON) in tables and queried them with PartiQL, an SQL-compatible language. Every change became a new revision in the journal; nothing was overwritten. A history() function returned every revision of a document over a time range. To prove integrity you requested a digest, a 256-bit hash covering the ledger's entire history at that moment, then asked for a proof for any block or revision and recomputed the hashes. The check was only as independent as the place you kept the digest: if you never saved it outside AWS, you were trusting AWS to tell you AWS had not changed anything.

## Data and access

Access was through the AWS API with IAM credentials, using open-source drivers (Apache-2.0) for Java, .NET, Go, Node.js and Python. Billing was per use across write I/Os, read I/Os, journal storage, indexed storage and transfer; the exact rates are no longer on AWS's site and are listed below as unverified. You could export the full journal to S3 and stream it to Kinesis, and both were the basis of AWS's exit path: [a sample migration](https://github.com/aws-samples/example-qldb-ledger-migration) that loads exports through Glue and DMS into Aurora PostgreSQL, then replays the Kinesis stream until cutover. That sample checks revision counts per table. It does not compare digests.

## Agents

QLDB had no MCP server, no agent features and no approval step. Any client with IAM permission could write. Its value to an agent would have been as a store an agent could read and prove against, and that value ended with the service.

## Customers

AWS published stories from [Specright, BungkusIT, Shinsegae International and Earnin](https://aws.amazon.com/blogs/database/category/database/amazon-quantum-ledger-database/), mostly traceability and fintech ledgers. No usage figures were ever published and no review-site ratings exist. Engineers rated it well; Corey Quinn called it "a good database" whose only flaw was being blockchain-adjacent. The shutdown was not formally announced: AWS updated documentation and emailed customers, [according to Classmethod](https://dev.classmethod.jp/articles/amazon-qldb-eos-announcement/), with the warning that after 31 July 2025 ledger data would be deleted. Leaving was real work: [Noah](https://noah.com/blog/zero-downtime-migration-of-noahs-internal-ledger) called its migration its largest to date, with four phases of dual writes and three kinds of consistency check, and rebuilt the append-only trail with trigger-fed history tables in Aurora.

## History

QLDB arrived in the 2018 wave of ledger and blockchain products alongside Amazon Managed Blockchain, and was the sober option: centralised, fast, SQL-like. It lasted under six years from preview to end of support. AWS's recommended replacement, in [Dan Blaner's July 2024 post](https://aws.amazon.com/blogs/database/category/database/amazon-quantum-ledger-database/), is Aurora PostgreSQL with pgAudit, S3 and Database Activity Streams, and he states plainly that Aurora PostgreSQL does not keep a permanent, immutable record of changes. The cryptographic verification has no direct replacement.

## For ownpurse

QLDB is the strongest evidence for ownpurse's custody principle. Its design was close to ownpurse's record: append-only, hash-chained, every revision kept, history as a query, a digest that commits to everything. What it got wrong was where the chain lived. When the vendor holds the only copy and the only digest, tamper-evidence can be taken away by a product decision, and customers fall back to trigger-based audit tables they then have to trust. Borrow the verification model (digests, proofs, an exportable journal with each block's hash). Make the record a SQLite file the owner holds, in open formats, so anyone can recompute the chain without the vendor and nothing depends on ownpurse staying in business.
