---
{
 "slug": "boomi",
 "name": "Boomi",
 "url": "https://boomi.com",
 "segment": "ipaas-middleware",
 "status": "active",
 "founded": "2000",
 "hq": "Conshohocken, Pennsylvania, US",
 "backing": "Private equity. Owned by Dell from 2010; sold to Francisco Partners and TPG Capital in a deal announced 2 May 2021 at a reported $4 billion. CEO Steve Lucas since December 2022.",
 "tagline": "A long-running cloud integration platform with API management, master data (Data Hub), data integration and, since 2025, an agent builder and governance layer.",
 "scope": "Boomi connects applications and data through low-code processes that run on its cloud or on runtimes you install, and adds B2B/EDI, API management, a master data hub that keeps golden records, ELT data integration (from Rivery) and Agentstudio for building and governing agents. It keeps no ledger; Data Hub is the nearest it comes to a unified record, and it lives in Boomi's cloud.",
 "domains": [
  "integration",
  "commerce"
 ],
 "customers_served": [
  "smb",
  "mid-market",
  "enterprise",
  "developers"
 ],
 "spectrum": {
  "custody": {
   "score": 1,
   "note": "Data passes through; Data Hub golden records are held in Boomi's cloud, not by the owner."
  },
  "locality": {
   "score": 2,
   "note": "Runtimes can be installed in your own environment and run behind the firewall; the control plane is Boomi's cloud."
  },
  "history": {
   "score": 1,
   "note": "Execution records and monitoring events; no versioned record of the data that moved."
  },
  "openness": {
   "score": 1,
   "note": "Closed platform with REST and SOAP platform APIs and a connector SDK; Boomi Companion agent skills are open source."
  },
  "breadth": {
   "score": 3,
   "note": "Integration, EDI, APIs, MDM and ELT in one suite, connecting 1,000+ applications, without being the record itself."
  },
  "autonomy": {
   "score": 4,
   "note": "Processes write to target systems unattended."
  },
  "agent_readiness": {
   "score": 3,
   "note": "Governed MCP tools, MCP connectors, Agentstudio and open-source agent skills for Claude Code; no general MCP server over the platform found."
  },
  "integration_ease": {
   "score": 3,
   "note": "Low-code builder reviewers call easy; a self-serve $99-a-month pay-as-you-go plan exists; licensing is still hard to read."
  },
  "market": {
   "score": 3,
   "note": "From mid-market to large enterprise, 30,000+ customers, with a self-serve entry tier."
  }
 },
 "api": {
  "public": true,
  "style": "Platform REST APIs and legacy SOAP APIs (WS-Security); Agent Garden APIs; Data Hub APIs",
  "auth": "Requires credentials per API; methods not confirmed on the pages read",
  "limits": "Not confirmed",
  "cost": "API use is tied to platform licensing ('API licensing' page not read).",
  "export": "Integration components can be pushed and pulled and processes packaged and deployed through the platform APIs and Companion; execution logs downloadable. No bulk business data export because Boomi is not the record, apart from Data Hub golden records via the Data Hub API.",
  "notes": "The Platform APIs cover process execution, listeners, deployments for CI/CD, events and execution records."
 },
 "pricing": {
  "model": "Editions by product (Integration, API Management, Data Hub, Data Integration) quote-based, plus pay-as-you-go",
  "entry": "$99 a month plus usage, billed monthly, self-serve by card (pay-as-you-go); 30-day trial",
  "top": "Enterprise Plus, quote-based",
  "notes": "Per-connection and per-message rates are not published (pricing page, 2026-10-10). One PeerSpot reviewer reports about $6,000 a year; others call it very expensive."
 },
 "agents": {
  "mcp": "official",
  "ai_features": "Agentstudio (no-code agent designer with memory, knowledge bases and MCP tools; Agent Control Plane with registry, observability, anomaly detection and a kill switch); 'Connect Governed MCP Tools' to turn enterprise assets into MCP tools; MCP connectors; AI Gateway; Boomi Companion, open-source Agent Skills and Claude Code plugins that build and deploy Boomi processes.",
  "agent_writes": "Yes. Agents use integrations and APIs as tools, and Companion lets a coding agent push components and deploy processes.",
  "approval": "No human approval gate found on the Agentstudio page; governance is registry, observability and a kill switch.",
  "notes": "Named a Pioneer in the June 2026 Gartner Emerging Market Quadrant for no-code agent builders."
 },
 "integration": {
  "scope": "Boomi Connect claims access to 1,000+ enterprise applications; the company cites 300,000 application connections (company page, 2026-10-10). EDI and B2B trading partners, managed file transfer (Thru).",
  "notes": "Connector SDK for custom connectors; low-code process builder; hybrid runtimes that keep syncing on-prem through internet outages."
 },
 "preference": {
  "scale": "30,000+ customers and 250,000+ community members (company page, 2026-10-10); about 1,907 employees (2022, Wikipedia).",
  "ratings": "PeerSpot: 4.0 of 5 from 31 reviews, 87% would recommend, ranked #1 in iPaaS with a 9.0 average (September 2026). Trustpilot: no reviews. Leader in the 2026 Gartner Magic Quadrant for iPaaS and the September 2026 Magic Quadrant for API Management.",
  "sentiment": "Praised for an easy low-code UI, fast setup, scale (one reviewer built about 1,100 interfaces for one customer) and responsive support; some find it cheap next to SAP or Oracle. Criticised for pricing that some call very high, a licensing model hard to understand, slowdowns on large files, weak version control and rollback, hard transaction tracking and an immature ETL module."
 },
 "timeline": [
  {
   "date": "2000",
   "event": "Founded by Rick Nucci in Berwyn, Pennsylvania"
  },
  {
   "date": "2010-11-02",
   "event": "Dell announces it will acquire Boomi"
  },
  {
   "date": "2017-03",
   "event": "Acquires ManyWho (low-code workflow, now Flow)"
  },
  {
   "date": "2019-12",
   "event": "Acquires Unifi Software"
  },
  {
   "date": "2021-05-02",
   "event": "Dell sells Boomi to Francisco Partners and TPG for a reported $4 billion"
  },
  {
   "date": "2022-12",
   "event": "Steve Lucas becomes chairman and CEO"
  },
  {
   "date": "2024-05",
   "event": "Buys APIIDA's federated API management business and Cloud Software Group's API assets, including Mashery"
  },
  {
   "date": "2024-12",
   "event": "Agreement to acquire Rivery (data integration)"
  },
  {
   "date": "2025-05",
   "event": "Agreement to acquire Thru (managed file transfer)"
  },
  {
   "date": "2026-06",
   "event": "Pioneer in Gartner's Emerging Market Quadrant for no-code agent builders"
  },
  {
   "date": "2026-09",
   "event": "Leader in Gartner's Magic Quadrant for API Management"
  }
 ],
 "relation": "neighbour",
 "relation_note": "Boomi is one of the oldest attempts to put one platform between a company's systems, and its Data Hub tried to keep a single golden record across them. That record lives in Boomi's cloud and holds current state, not history. ownpurse keeps every source's facts, held by the owner, and leaves the golden record as a query.",
 "borrow": "Data Hub's model of source records linked to one golden record, with each source kept as a contributor, is close to ownpurse's link rules; runtimes that keep working through outages; an agent kill switch and registry; shipping agent skills as open source so coding agents can drive the product.",
 "avoid": "A single golden record that overwrites what each source said; licensing customers cannot read; version control and rollback as afterthoughts; a suite grown by acquisition (ManyWho, Unifi, Mashery, Rivery, Thru) that reviewers find overcomplicated.",
 "sources": [
  {
   "title": "Boomi on Wikipedia",
   "url": "https://en.wikipedia.org/wiki/Boomi,_LP"
  },
  {
   "title": "Boomi company page",
   "url": "https://boomi.com/company/"
  },
  {
   "title": "Boomi pricing",
   "url": "https://boomi.com/pricing/"
  },
  {
   "title": "Boomi Integration",
   "url": "https://boomi.com/platform/integration/"
  },
  {
   "title": "Boomi Agentstudio",
   "url": "https://boomi.com/platform/agentstudio/"
  },
  {
   "title": "Boomi developer portal",
   "url": "https://developer.boomi.com/"
  },
  {
   "title": "Boomi Platform APIs overview",
   "url": "https://developer.boomi.com/docs/APIs/PlatformAPI/Platform_APIs_Overview_New"
  },
  {
   "title": "Boomi Companion overview",
   "url": "https://developer.boomi.com/docs/BoomiCompanion/Boomi_companion_overview"
  },
  {
   "title": "Boomi blog",
   "url": "https://boomi.com/blog/"
  },
  {
   "title": "PeerSpot Boomi iPaaS reviews",
   "url": "https://www.peerspot.com/products/boomi-ipaas-reviews"
  },
  {
   "title": "PeerSpot iPaaS category",
   "url": "https://www.peerspot.com/categories/integration-platform-as-a-service-ipaas"
  },
  {
   "title": "Boomi on Trustpilot",
   "url": "https://www.trustpilot.com/review/boomi.com"
  }
 ],
 "unverified": [
  "Price Dell paid in 2010 and the closing date of the 2021 sale",
  "Closing dates of the Rivery and Thru acquisitions",
  "Platform API authentication methods and rate limits",
  "Revenue and current employee count",
  "Whether Data Hub keeps a history of golden record versions",
  "Per-connection and per-message pay-as-you-go rates",
  "G2, Capterra and Gartner Peer Insights ratings (fetches blocked); web search budget exhausted, so press coverage was not checked"
 ],
 "researched": "2026-10-10"
}
---

## What it is

Boomi is a cloud integration platform founded by Rick Nucci in 2000 and now headquartered in Conshohocken, Pennsylvania ([Wikipedia](https://en.wikipedia.org/wiki/Boomi,_LP)). It was one of the first integration platforms delivered as a service. Today it sells a suite: Integration, API Management, Data Hub (master data), Data Integration, B2B/EDI and Agentstudio, with 30,000+ customers by its own count ([company page](https://boomi.com/company/)).

## How it works

You build processes in a low-code designer and run them on Boomi's cloud or on runtimes installed in your own environment; the platform describes a single-instance, multi-tenant architecture that keeps on-prem integrations running through internet outages ([integration](https://boomi.com/platform/integration/)). Data Hub matches records from several systems into golden records, the closest any of the big integration platforms came to one unified business record, held in Boomi's cloud as current state.

## Data and access

The [developer portal](https://developer.boomi.com/) lists Platform APIs (REST, plus legacy SOAP with WS-Security), Agent Garden APIs and Data Hub APIs, along with a Connector SDK. The [Platform APIs](https://developer.boomi.com/docs/APIs/PlatformAPI/Platform_APIs_Overview_New) run and track processes, manage listeners, deploy packages for CI/CD and read execution records. Boomi is the only one of the three with a published entry price: pay-as-you-go at $99 a month plus usage, self-serve by card, alongside quote-based editions ([pricing](https://boomi.com/pricing/)).

## Agents

[Agentstudio](https://boomi.com/platform/agentstudio/) builds no-code agents that use integrations, APIs, knowledge bases and MCP servers as tools, and governs them with a registry, observability, anomaly detection and a kill switch. Boomi offers "governed MCP tools" and an AI gateway. [Boomi Companion](https://developer.boomi.com/docs/BoomiCompanion/Boomi_companion_overview) is an open-source set of Agent Skills, shipped as Claude Code plugins, that lets a coding agent build processes, push components and deploy them. No human approval step on agent writes was found.

## Customers

Boomi spans mid-market to large enterprise. On [PeerSpot](https://www.peerspot.com/products/boomi-ipaas-reviews) it rates 4.0 of 5 from 31 reviews and ranks first in the iPaaS category (September 2026). Reviewers like the easy UI and quick setup; complaints are split pricing, a hard-to-read licence, slowdowns on large files, and weak version control and transaction tracking.

## History

Dell bought Boomi in 2010 and sold it to Francisco Partners and TPG in 2021 for a reported $4 billion. Under Steve Lucas (CEO since December 2022) it has bought its way into adjacent problems: API management from APIIDA and Mashery (May 2024), data integration from Rivery (December 2024), file transfer from Thru (May 2025). It is a long-running attempt to make one vendor the place all of a business's data meets.

## For ownpurse

Boomi's Data Hub is the historic version of the idea ownpurse holds: many sources, one picture. The difference is what is kept. A golden record keeps the merged answer and loses what each source said and when; ownpurse keeps every source's facts and treats the merged picture as a query that can be asked again. Boomi's $99 tier shows that integration can reach smaller buyers, but it still needs an integrator's skills and keeps nothing the owner holds. The gap remains for a small business or a person who wants one record of their own.
