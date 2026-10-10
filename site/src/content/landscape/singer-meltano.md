---
{
 "slug": "singer-meltano",
 "name": "Singer and Meltano",
 "url": "https://meltano.com",
 "segment": "connector-framework",
 "status": "active",
 "founded": "Singer: 2016-2017 at Stitch (singer-python repository created 2016-10). Meltano: started inside GitLab, spun out 2021-06",
 "hq": "Singer: a community spec, sponsored by Stitch (now a Qlik product). Meltano: stewarded by Matatika since 2026 (headquarters not confirmed)",
 "backing": "Singer is sponsored by Stitch, which Talend owned and Qlik now owns. Meltano spun out of GitLab in June 2021 with venture funding (amount not confirmed), became part of Arch, and was acquired by Matatika when Arch shut down (announced 2026-03-01).",
 "tagline": "Singer is an open spec for connectors that emit JSON lines and checkpoint their progress. Meltano is the open-source CLI that runs Singer taps and targets.",
 "scope": "A Singer tap reads a source and writes SCHEMA, RECORD and STATE messages to stdout. A target loads them somewhere. Meltano installs, configures and runs taps and targets declaratively, stores state bookmarks, and offers an SDK and a hub of 600+ connectors, including tap-xero (Stitch) and tap-quickbooks (Hotglue). Neither keeps books, reconciles or keeps history; what is retained depends on the target.",
 "domains": [
  "integration"
 ],
 "customers_served": [
  "developers",
  "smb",
  "mid-market"
 ],
 "spectrum": {
  "custody": {
   "score": 3,
   "note": "You run it and choose the target, so the copy lands in a file or database you own. The source remains the authoritative record."
  },
  "locality": {
   "score": 4,
   "note": "pip install meltano, with state in the project's system database or a local file. Meltano Cloud is optional."
  },
  "history": {
   "score": 1,
   "note": "A STATE bookmark is overwritten each run, and most warehouse targets upsert. An append-only JSONL target can keep every record, but that is the user's choice."
  },
  "openness": {
   "score": 4,
   "note": "Meltano is MIT, the SDK is Apache-2.0, the spec is public, and tap-xero and tap-quickbooks are AGPL-3.0."
  },
  "breadth": {
   "score": 3,
   "note": "600+ connectors across every domain, but each is a separate pipe into a warehouse, with no shared model."
  },
  "autonomy": {
   "score": 1,
   "note": "Pipelines run unattended into destinations, but nothing writes to or posts in the books."
  },
  "agent_readiness": {
   "score": 2,
   "note": "A declarative CLI and YAML that agents can drive. Meltano Cloud shows tools such as run_pipeline to Claude and an AI knowledge base (Aug 2026), but I found no official MCP server."
  },
  "integration_ease": {
   "score": 3,
   "note": "The spec is a few JSON message types over stdout, and the SDK cuts boilerplate. Tap quality and maintenance vary widely."
  },
  "market": {
   "score": 2,
   "note": "Data engineers at startups and mid-market companies, with Cloud tiers quoted on request."
  }
 },
 "api": {
  "public": true,
  "style": "Process interface: JSON lines (SCHEMA, RECORD, STATE) on stdout, with config, state and catalog files as input. Meltano adds a CLI and meltano.yml.",
  "auth": "Each tap handles its source's auth (tap-xero uses Xero OAuth). Secrets live in Meltano config or environment variables.",
  "limits": "Source API limits apply. Taps resume from the last STATE bookmark.",
  "cost": "Free and open source. Meltano Cloud is priced by compute hours (Starter 200, Growth 2,000, Scale 5,000 a month) with no public prices. Stitch, the hosted Singer runner, starts at $100/mo (2026-10).",
  "export": "Whatever the target writes: JSONL, CSV, SQLite, Postgres, warehouses. meltano state get exports bookmarks as JSON.",
  "notes": "The spec (v0.3.0) leaves the meaning of a STATE value to each tap. Meltano persists it per state_id in the system database, a file, S3, GCS, Azure or Snowflake."
 },
 "pricing": {
  "model": "Open source, free. A managed cloud service is billed by compute hours.",
  "entry": "Free (self-hosted Meltano, Singer taps)",
  "top": "Meltano Cloud Enterprise, unlimited compute hours, custom SLA, by quote (2026-10)",
  "notes": "Meltano Cloud tiers list no dollar prices. Stitch: Standard from $100/mo, Advanced $1,500/mo, Premium $3,000/mo billed annually."
 },
 "agents": {
  "mcp": "none",
  "ai_features": "Meltano Cloud shows Claude calling tools such as run_pipeline and diagnose_pipeline. 'Melty AI' writes up pipeline failure diagnoses. The files-melty-ai repository (Aug 2026) packages condensed docs as agent context.",
  "agent_writes": "Agents can run pipelines through the CLI or Cloud. Writing to a source is possible only through reverse-ETL targets; Meltano Cloud lists reverse ETL.",
  "approval": "None. Pipelines run as configured.",
  "notes": "I could not confirm an official MCP server; the Cloud tool demo may be one, and is listed here as unverified."
 },
 "integration": {
  "scope": "600+ connectors on Meltano Hub (2026-10). singer.io lists about 106 taps and 10 targets. Xero via tap-xero (Stitch default variant; 22 streams including journals, bank transactions, invoices and payments). QuickBooks via tap-quickbooks (Hotglue variant).",
  "notes": "tap-xero's Hub page shows maintenance status 'Unknown'. Its README dates from 2017 and still mentions Xero Partner Applications, though the repository was pushed in July 2026 (v2.6.0)."
 },
 "preference": {
  "scale": "meltano/meltano about 2,650 GitHub stars, sdk 119 stars, singer-python 586 stars (2026-10-10). Meltano cites 5,500+ data professionals on Slack and GitHub.",
  "ratings": "Not confirmed. Review sites were unreachable and web search was unavailable.",
  "sentiment": "Not confirmed from review sites. The fragmentation into several variants of the same tap is visible on Meltano Hub."
 },
 "timeline": [
  {
   "date": "2016-10",
   "event": "singer-python repository created at Stitch"
  },
  {
   "date": "2017-08",
   "event": "tap-xero created (Copyright 2017 Stitch)"
  },
  {
   "date": "2020-08",
   "event": "singer-io tap-quickbooks created"
  },
  {
   "date": "2021-06-30",
   "event": "Meltano spins out of GitLab ('Copyright is held by Meltano, not GitLab' commit); Meltano SDK repository created"
  },
  {
   "date": "2026-03-01",
   "event": "Arch shuts down; Matatika announces it has acquired the Meltano open-source project"
  },
  {
   "date": "2026-10-07",
   "event": "Meltano v4.4.1, the latest release as of 2026-10-10"
  }
 ],
 "relation": "kin",
 "relation_note": "The open, local, CLI-first ancestor of ownpurse's connector layer. A Singer tap could feed ownpurse as one more source, and its STATE checkpoints are the pattern ownpurse's cursors follow.",
 "borrow": "STATE emitted after the records it covers, so a crash resumes from the last checkpoint (store each cursor as a fact). SCHEMA with key_properties declared before records. JSON lines on stdout as a connector protocol, so ownpurse could accept any Singer tap's output as a source. Discovery catalogs that let the user choose streams and fields.",
 "avoid": "Overwriting the bookmark and letting the target upsert, which discards history. A spec that leaves state meaning to each tap. Connector rot: unowned taps with stale READMEs and variant sprawl. A sponsor's commercial fortunes (Stitch, then Talend and Qlik; Meltano, then Arch and Matatika) deciding the fate of the standard.",
 "sources": [
  {
   "title": "Meltano home",
   "url": "https://meltano.com"
  },
  {
   "title": "Meltano pricing",
   "url": "https://meltano.com/pricing/"
  },
  {
   "title": "Looking for Arch? (Matatika acquires Meltano)",
   "url": "https://meltano.com/blog/looking-for-arch"
  },
  {
   "title": "Meltano state backends",
   "url": "https://docs.meltano.com/concepts/state_backends/"
  },
  {
   "title": "meltano/meltano on GitHub",
   "url": "https://github.com/meltano/meltano"
  },
  {
   "title": "Meltano SDK",
   "url": "https://github.com/meltano/sdk"
  },
  {
   "title": "Singer spec",
   "url": "https://github.com/singer-io/getting-started/blob/master/docs/SPEC.md"
  },
  {
   "title": "singer.io",
   "url": "https://www.singer.io"
  },
  {
   "title": "singer-io/tap-xero",
   "url": "https://github.com/singer-io/tap-xero"
  },
  {
   "title": "tap-xero on Meltano Hub",
   "url": "https://hub.meltano.com/extractors/tap-xero"
  },
  {
   "title": "tap-quickbooks on Meltano Hub",
   "url": "https://hub.meltano.com/extractors/tap-quickbooks"
  },
  {
   "title": "Stitch pricing",
   "url": "https://www.stitchdata.com/pricing/"
  }
 ],
 "unverified": [
  "Meltano's 2021 seed amount and investors (often reported as $4.2M led by GV)",
  "When Meltano rebranded or operated as Arch, and the dates of that pivot",
  "Matatika's headquarters and the terms of the acquisition",
  "Dates of Talend's acquisition of Stitch (reported 2018) and Qlik's acquisition of Talend (reported 2023)",
  "Whether Meltano Cloud's Claude tools are delivered through an official MCP server",
  "Review-site ratings and sentiment"
 ],
 "researched": "2026-10-10"
}
---

## What it is

[Singer](https://www.singer.io) is an open specification for data connectors. Stitch created it around 2016-2017 and still sponsors it. A tap reads a source; a target writes somewhere. [Meltano](https://meltano.com) is the open-source CLI that grew up around Singer: it installs taps and targets, configures them in `meltano.yml`, runs them, and stores their progress. Meltano says it supports 600+ connectors. singer.io itself lists about 106 taps and 10 targets.

## How it works

The [Singer spec](https://github.com/singer-io/getting-started/blob/master/docs/SPEC.md) (v0.3.0) is short. A tap takes a config file and, optionally, state and catalog files. It writes JSON lines to stdout: SCHEMA (a JSON Schema and `key_properties`), RECORD (one row of one stream, with an optional `time_extracted`) and STATE (a value the runner should save). Next run, the tap reads that state and resumes. The catalog selects streams and fields and marks each one FULL_TABLE or INCREMENTAL. Meltano saves state per `state_id` in its system database by default, or in a [file, S3, GCS or Azure](https://docs.meltano.com/concepts/state_backends/), with locking. The [Meltano SDK](https://github.com/meltano/sdk) (Apache-2.0) handles pagination, state and schema for anyone writing a tap.

## Data and access

Everything runs on your machine and the output goes where you point it, which is what Singer gets right: no vendor in the middle. [tap-xero](https://github.com/singer-io/tap-xero) (AGPL-3.0, v2.6.0) pulls 22 Xero resources, including journals, bank transactions, invoices, payments, manual journals and linked transactions, incrementally from saved state. Meltano Hub's default QuickBooks tap is the [Hotglue variant](https://hub.meltano.com/extractors/tap-quickbooks). History depends on the target. A STATE bookmark is overwritten each run, and most warehouse loaders upsert. Meltano is free and MIT-licensed. Meltano Cloud is [priced](https://meltano.com/pricing/) by compute hours with no public figures. [Stitch](https://www.stitchdata.com/pricing/), the hosted Singer runner, now 'a Qlik product', starts at $100 a month.

## Agents

Meltano is a declarative CLI, which agents handle well. Meltano Cloud's homepage shows Claude calling tools such as `run_pipeline` and `diagnose_pipeline`, and in August 2026 the project published an AI knowledge base for agents. I found no official MCP server repository. Nothing in Singer writes to the books, and there is no approval step.

## Customers

Users are data engineers at startups and mid-market companies who want ELT without per-row SaaS pricing. Meltano's blog positions it directly against Fivetran. On GitHub, meltano/meltano has about 2,650 stars, the SDK 119 and singer-python 586 (October 2026). I could not reach review sites.

## History

Singer is a case study in an open standard tied to a sponsor. Stitch launched it, Talend bought Stitch and Qlik bought Talend, and many singer-io taps now look lightly maintained: tap-xero's README still dates from 2017 and its Hub maintenance status is 'Unknown', though it was pushed in July 2026. Meltano began inside GitLab and [spun out in June 2021](https://github.com/meltano/meltano/commits/main/LICENSE). It later operated as Arch. On 1 March 2026 Arch shut down, and Matatika [announced](https://meltano.com/blog/looking-for-arch) it had acquired the Meltano open-source project, which it says stays open and community-driven. Releases continue (v4.4.1, 7 October 2026).

## For ownpurse

Singer is ownpurse's nearest ancestor in spirit: open, local, a small process protocol, checkpoints the runner keeps. Two lessons carry over directly. Emit STATE only after the records it covers, so a failure resumes cleanly; ownpurse should store that cursor as a fact. And keep the connector protocol simple enough that agents can write taps, so any Singer tap could become an ownpurse source. Where ownpurse differs is what happens after the pipe. Singer hands records to a target that usually overwrites, while ownpurse keeps every observation and rebuilds the ledger from it.
