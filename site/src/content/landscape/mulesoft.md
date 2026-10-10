---
{
 "slug": "mulesoft",
 "name": "MuleSoft",
 "url": "https://www.mulesoft.com",
 "segment": "ipaas-middleware",
 "status": "acquired",
 "founded": "2006 (as MuleSource; open-source Mule project from 2003)",
 "hq": "San Francisco, California, US",
 "backing": "Salesforce subsidiary. IPO on the NYSE on 17 March 2017; Salesforce agreed to acquire it on 20 March 2018 for an enterprise value of about $6.5 billion. Before the IPO it raised $259 million across seven rounds (last a $37 million Series E, April 2013).",
 "tagline": "Salesforce's enterprise integration and API platform (Anypoint), now also sold as a registry and governance layer for AI agents (Agent Fabric).",
 "scope": "MuleSoft builds and runs Mule applications that connect systems through APIs, with DataWeave for transformation, API management, an exchange of connectors and assets, and runtimes in its cloud or on your own servers. It is developer-heavy and suited to large estates; it keeps no ledger or business record of its own. Since 2025 it also registers, routes and governs AI agents and MCP servers.",
 "domains": [
  "integration"
 ],
 "customers_served": [
  "enterprise",
  "developers"
 ],
 "spectrum": {
  "custody": {
   "score": 1,
   "note": "A conduit: the books stay in each source system; self-hosted runtimes keep data in your network but nothing is held as a record."
  },
  "locality": {
   "score": 3,
   "note": "Standalone Mule runtimes run on your own servers, even without a control plane connection, and a customer-hosted control plane (PCE) exists."
  },
  "history": {
   "score": 1,
   "note": "Monitoring and logs of transactions; no versioned record of what passed through."
  },
  "openness": {
   "score": 2,
   "note": "Mule Community Edition was open source, but its GitHub repository was archived in 2026; Anypoint is closed with documented platform APIs."
  },
  "breadth": {
   "score": 3,
   "note": "Connects SaaS, SAP, databases and legacy systems at large scale, without holding them in one place."
  },
  "autonomy": {
   "score": 4,
   "note": "Mule flows write to target systems unattended."
  },
  "agent_readiness": {
   "score": 4,
   "note": "MCP connector makes any Mule app an MCP server or client; DX and Platform MCP servers for IDE agents; Agent Fabric registry and governance."
  },
  "integration_ease": {
   "score": 2,
   "note": "Powerful but reviewers report a steep learning curve, heavy tooling and complex vCore and usage licensing."
  },
  "market": {
   "score": 4,
   "note": "Large enterprises; reviewers say small and mid-sized businesses struggle to justify the cost."
  }
 },
 "api": {
  "public": true,
  "style": "REST platform APIs for Anypoint (Exchange, Runtime Manager, API Manager), plus the APIs customers build on Mule; MCP over Streamable HTTP",
  "auth": "Anypoint connected apps and OAuth [details not confirmed on a page read]; DX MCP server described as permission-scoped",
  "limits": "Not confirmed",
  "cost": "Quote-based. Usage-based packaging (Anypoint Integration Advanced) meters Mule flows, Mule messages and data throughput instead of vCores; CloudHub 2.0 replicas range from 0.05 to 4 vCores.",
  "export": "Mule applications are code (XML flows and DataWeave) you can keep in your own repository and deploy elsewhere; MuleSoft holds no business data to export.",
  "notes": "Developers own their integration code, a real strength against low-code platforms."
 },
 "pricing": {
  "model": "Annual contract, quote-based; usage-based (flows, messages, throughput) or legacy vCore capacity",
  "entry": "30-day free trial of Anypoint Platform, no card",
  "top": "Anypoint Platform and MuleSoft Automation (RPA, Composer, Anypoint), contact for pricing",
  "notes": "Salesforce's pricing page shows 'Contact for pricing' for every edition (2026-10-10). Reviewers name price and licensing complexity as the main complaints."
 },
 "agents": {
  "mcp": "official",
  "ai_features": "MCP connector (v1.7) to expose Mule apps, connectors and APIs to MCP clients or call MCP servers from flows; MuleSoft DX MCP server and Platform MCP server for IDE agents; Agent Fabric (registry, broker, governance through Flex Gateway for MCP and A2A, visualizer); Agentforce integration.",
  "agent_writes": "Yes. Anything a Mule flow can do, an agent can invoke through an MCP tool, including writes to ERPs.",
  "approval": "No built-in human approval gate found; governance is policy at the gateway (PII detection, token controls, routing). Approval would be something you build into a flow.",
  "notes": "Agent Fabric announced 25 September 2025, with registry and visualizer GA from October 2025."
 },
 "integration": {
  "scope": "Anypoint Exchange connectors for SaaS, SAP, databases and legacy systems; counts not confirmed. Salesforce integration is the most praised strength.",
  "notes": "API-led connectivity: system, process and experience APIs. DataWeave is a declarative transformation language."
 },
 "preference": {
  "scale": "More than 1,200 customers at the 2018 acquisition (Salesforce); more than 1,600 customers and 1,400 employees by August 2019; revenue $296.46 million in 2017. Current figures are reported inside Salesforce's results and were not confirmed.",
  "ratings": "PeerSpot: 4.0 of 5 from 62 reviews, 92% would recommend, ranked #4 in iPaaS (September 2026). G2, Capterra and Gartner could not be read.",
  "sentiment": "Praised for Salesforce and SAP integration, API management and security policies, hybrid deployment and stability once configured. Criticised for price, licensing complexity (vCores, volumes), a steep learning curve, a dated Anypoint Studio IDE, weak monitoring and mixed support."
 },
 "timeline": [
  {
   "date": "2003",
   "event": "Ross Mason starts the open-source Mule project"
  },
  {
   "date": "2006",
   "event": "MuleSource founded by Ross Mason and Dave Rosenberg"
  },
  {
   "date": "2009",
   "event": "Renamed MuleSoft"
  },
  {
   "date": "2013-04",
   "event": "$37 million Series E led by NEA; $259 million raised in total"
  },
  {
   "date": "2017-03-17",
   "event": "Begins trading on the NYSE"
  },
  {
   "date": "2018-03-20",
   "event": "Salesforce agrees to acquire MuleSoft for about $6.5 billion"
  },
  {
   "date": "2025-09-25",
   "event": "Agent Fabric announced: agent registry, broker, governance, visualizer"
  },
  {
   "date": "2026-09-18",
   "event": "GitHub repository of Mule Community Edition archived (read-only)"
  }
 ],
 "relation": "neighbour",
 "relation_note": "MuleSoft is the heavy end of the integration market: developer teams building an API layer across a large estate. It moves and exposes data; it keeps no record and has no fit for a small business or a person. Relevant to ownpurse as the pattern large companies use, not as a rival.",
 "borrow": "API-led layering (system APIs over each source, process APIs, experience APIs) maps onto connectors, the record and views; integration as code you own and version; DataWeave's declarative mappings as a model for pure projectors; standalone runtimes that run with no control plane, the same promise ownpurse makes locally.",
 "avoid": "Licensing by vCores and messages that customers cannot predict; tooling so heavy that only specialists can use it; letting an open-source core wither once the commercial platform takes over.",
 "sources": [
  {
   "title": "MuleSoft on Wikipedia",
   "url": "https://en.wikipedia.org/wiki/MuleSoft"
  },
  {
   "title": "Salesforce signs definitive agreement to acquire MuleSoft (2018-03-20)",
   "url": "https://www.salesforce.com/news/press-releases/2018/03/20/salesforce-signs-definitive-agreement-to-acquire-mulesoft/"
  },
  {
   "title": "MuleSoft pricing (Salesforce)",
   "url": "https://www.salesforce.com/mulesoft/pricing/"
  },
  {
   "title": "Usage-based pricing docs",
   "url": "https://docs.mulesoft.com/general/pricing"
  },
  {
   "title": "Mule runtime deployment docs",
   "url": "https://docs.mulesoft.com/mule-runtime/latest/"
  },
  {
   "title": "MuleSoft MCP connector docs",
   "url": "https://docs.mulesoft.com/mcp-connector/latest/"
  },
  {
   "title": "MuleSoft MCP servers docs",
   "url": "https://docs.mulesoft.com/mulesoft-mcp-server/"
  },
  {
   "title": "MuleSoft Agent Fabric announcement (2025-09-25)",
   "url": "https://www.salesforce.com/news/press-releases/2025/09/24/mulesoft-agent-fabric-announcement/"
  },
  {
   "title": "Mule Community Edition on GitHub",
   "url": "https://github.com/mulesoft/mule"
  },
  {
   "title": "PeerSpot MuleSoft Anypoint reviews",
   "url": "https://www.peerspot.com/products/mulesoft-anypoint-platform-reviews"
  },
  {
   "title": "PeerSpot iPaaS category",
   "url": "https://www.peerspot.com/categories/integration-platform-as-a-service-ipaas"
  }
 ],
 "unverified": [
  "Current MuleSoft revenue and customer counts (reported inside Salesforce's Integration and Analytics results; not read)",
  "Anypoint platform API auth details and rate limits",
  "Release date of the MCP connector 1.0 and the MCP servers",
  "Connector count in Anypoint Exchange and whether a Xero connector exists",
  "Mule CE license (GitHub reports 'Other'; historically CPAL)",
  "G2, Capterra and Gartner ratings (fetches blocked); web search budget exhausted"
 ],
 "researched": "2026-10-10"
}
---

## What it is

MuleSoft makes the Anypoint Platform, an enterprise integration and API management suite, and has been part of Salesforce since 2018. It grew from the open-source Mule project Ross Mason started in 2003; the company was founded as MuleSource in 2006 and renamed in 2009 ([Wikipedia](https://en.wikipedia.org/wiki/MuleSoft)). Its pitch is the "application network": every system exposed through reusable APIs.

## How it works

Developers build Mule applications (flows in XML, transformations in DataWeave) in Anypoint Studio or the newer tooling, publish assets to Exchange and deploy them to CloudHub 2.0, to Runtime Fabric on their own Kubernetes, or to standalone runtimes on their own servers, including isolated ones with no control plane connection ([Mule runtime docs](https://docs.mulesoft.com/mule-runtime/latest/)). Because applications are code, you own and version them. MuleSoft keeps monitoring and logs, not a record of the business data it carries.

## Data and access

Anypoint has documented platform APIs for its own management, and its customers build REST APIs on top of their systems. Pricing is not published: every edition shows "Contact for pricing", with a 30-day trial ([Salesforce pricing](https://www.salesforce.com/mulesoft/pricing/)). Newer contracts are usage-based, metering Mule flows, messages and throughput instead of vCores ([usage-based pricing](https://docs.mulesoft.com/general/pricing)).

## Agents

The [MCP connector](https://docs.mulesoft.com/mcp-connector/latest/) (version 1.7) lets any Mule app act as an MCP server, exposing connectors and APIs to clients such as Claude, or as an MCP client inside a flow. The [DX and Platform MCP servers](https://docs.mulesoft.com/mulesoft-mcp-server/) let IDE agents design APIs, deploy apps and inspect traffic. [Agent Fabric](https://www.salesforce.com/news/press-releases/2025/09/24/mulesoft-agent-fabric-announcement/), announced on 25 September 2025, adds a registry of agents and MCP and A2A servers, a broker that routes tasks, governance through Flex Gateway and a visualizer. Governance here means gateway policy (PII detection, token limits); no human approval step on writes is built in.

## Customers

At acquisition MuleSoft had more than 1,200 customers, including Coca-Cola, Barclays and Unilever ([Salesforce, 2018](https://www.salesforce.com/news/press-releases/2018/03/20/salesforce-signs-definitive-agreement-to-acquire-mulesoft/)). On [PeerSpot](https://www.peerspot.com/products/mulesoft-anypoint-platform-reviews) it rates 4.0 of 5 from 62 reviews. Reviewers value Salesforce and SAP integration, security policies and hybrid deployment, and complain about price, licensing complexity, the learning curve and a dated IDE; several say smaller companies struggle to justify it.

## History

MuleSoft listed on the NYSE on 17 March 2017 and was bought a year later for about $6.5 billion, one of the largest bets that a company's data problem is an integration problem. Its open-source Community Edition, once the way developers met Mule, now sits in an [archived GitHub repository](https://github.com/mulesoft/mule) (archived 18 September 2026; last commit on master June 2025).

## For ownpurse

MuleSoft shows what solving cross-domain wrangling costs at enterprise scale: a dedicated integration team, an API layer over every system, a contract priced by capacity. It never tried to hold the combined record; each system remains the truth and MuleSoft is the plumbing. That leaves the same gap for small businesses and people that ownpurse is built for, with nothing in their price range. The ideas worth taking are architectural: one system API per source, integration as code you own, declarative mappings, and runtimes that work with no control plane at all.
