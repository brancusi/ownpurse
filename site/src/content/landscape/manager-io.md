---
{
 "slug": "manager-io",
 "name": "Manager.io",
 "url": "https://www.manager.io",
 "segment": "plain-text-local",
 "status": "active",
 "founded": "Not confirmed; forum activity dates back to at least 2016 [unverified]",
 "hq": "Sydney, Australia (footer of the earlier site, www2.manager.io)",
 "backing": "No outside funding found. It appears to be a small, self-funded company led by its developer, who posts as 'lubos' on the forum [unverified].",
 "tagline": "Free desktop accounting for Windows, Mac and Linux that keeps each business in a local SQLite file, with paid cloud and server editions of the same program.",
 "scope": "Full small-business accounting: invoicing, bank and cash accounts with statement import and bank rules, inventory, payroll payslips, fixed assets, multi-currency, tax reports and custom fields. The desktop edition is single-user and offline; multi-user and remote access need Cloud (hosted) or Server (self-hosted). It does not connect to other ledgers and has no official bank-feed aggregator or AI features.",
 "domains": [
  "ledger",
  "invoicing",
  "bank",
  "payroll",
  "reporting",
  "tax"
 ],
 "customers_served": [
  "smb",
  "freelancer",
  "accountants"
 ],
 "spectrum": {
  "custody": {
   "score": 4,
   "note": "The free desktop edition keeps each business in a .manager SQLite file on your machine; Cloud lets you download all your data at any time."
  },
  "locality": {
   "score": 4,
   "note": "Desktop runs offline on your own machine; Server Edition is self-hosted; Cloud is optional."
  },
  "history": {
   "score": 2,
   "note": "A per-business History audit trail (since 20.7.14, June 2020) records every create, update and delete, with undo, but it sits in the same mutable file, is not tamper-evident, and undo was reported to remove the entry it undid."
  },
  "openness": {
   "score": 2,
   "note": "Closed source but free, with an API described by 1,515 OpenAPI specs and a full backup file of each business."
  },
  "breadth": {
   "score": 2,
   "note": "One business's books with invoicing, payroll, inventory and bank statement import (CSV, OFX, QIF, FDX added 2024); no other sources."
  },
  "autonomy": {
   "score": 1,
   "note": "Bank rules categorise imported lines automatically; everything else is entered by people."
  },
  "agent_readiness": {
   "score": 2,
   "note": "REST API (api2/api4) with per-business access tokens; no official MCP or CLI, but community ones exist (2026)."
  },
  "integration_ease": {
   "score": 2,
   "note": "Self-serve tokens and OpenAPI specs, but users report undocumented parameters, little input validation and API crashes (forum, March 2026)."
  },
  "market": {
   "score": 2,
   "note": "Small businesses and the accountants who serve them, often in emerging markets."
  }
 },
 "api": {
  "public": true,
  "style": "REST over JSON (api2 and api4 paths), described as 1,515 per-endpoint OpenAPI 3.1 specs; also an extension API where an iframe sends postMessage requests that Manager forwards to its API.",
  "auth": "Access tokens created per business under Settings, Access Tokens (Cloud and Server); extensions inherit the signed-in user's permissions; on Desktop extensions can call every endpoint.",
  "limits": "No published rate limits. API2 is scoped per business; there is no account-level API (forum, September 2026).",
  "cost": "No API fees. API comes with every edition; Cloud is USD 59/month flat.",
  "export": "Full backup file of each business (the SQLite database), restorable on any OS; reports export; Cloud data downloadable at any time.",
  "notes": "The spec index on 10 October 2026 listed 546 GET, 396 PUT, 350 POST, 222 DELETE and 1 QUERY operation. Forum users say api3/api4 are undocumented and api2 docs lack parameter details; a community member published an enhanced OpenAPI schema."
 },
 "pricing": {
  "model": "Free desktop edition; flat-price hosted Cloud Edition; perpetual-licence Server Edition",
  "entry": "Desktop Edition: free, no time or usage limits, no ads",
  "top": "Cloud Edition: USD 59/month or USD 590/year (also CAD 79, AUD 89, EUR 59, GBP 49, NZD 99, SGD 79 per month), unlimited users and businesses, 14-day trial",
  "notes": "No discounts by policy. Server Edition was sold as a one-time perpetual licence with 12 months of updates included and renewal at 50 percent of full price (forum, 2024); in October 2026 the server page says 'no monthly fees', no purchase page could be found, and users report the product-key prompt has gone, so its current price is unclear."
 },
 "agents": {
  "mcp": "community",
  "ai_features": "None in the product. The developer has written about using LLMs to build extensions and country tax reports (forum, June 2025).",
  "agent_writes": "Through the REST API with an access token: create, update and delete across most document types. Community manager-mcp is read-only by default, with writes enabled per domain by environment variables (July 2026).",
  "approval": "None in Manager. The community MCP's read-only default is the only guard; the History audit trail and undo are after the fact.",
  "notes": "Community projects in 2026: manager-ai-skills (a Bash CLI over api2 with skill files for coding agents, March 2026), an MCP server in progress with an enhanced schema (March 2026), and manager-mcp (July 2026)."
 },
 "integration": {
  "scope": "No official third-party connectors. Bank statement import (CSV, OFX, QIF, FDX), batch create and update from spreadsheets, iframe extensions, and the REST API.",
  "notes": "A forum volunteer told a Shopify user in September 2026 that there is no official connector and that the api2 and api4 endpoints 'are made with AI ease of access in mind'."
 },
 "preference": {
  "ratings": "Capterra 4.8 out of 5 from 62 reviewers, as shown on the earlier Manager site (copyright 2023); the current homepage still says 4.8. Capterra itself returned 403, so the count is not rechecked.",
  "scale": "No customer or user counts published. Calendar-versioned releases run continuously (version 26.9.1.0 in September 2026).",
  "sentiment": "Praised as 'free forever', full-featured, offline and portable across operating systems. Recurring forum complaints: frequent changes, thin API documentation and validation, audit trail gaps (custom fields, restricted users), very large SQLite files (over 10 GB) failing to upload to Cloud, and confusion over Server Edition licensing."
 },
 "timeline": [
  {
   "date": "2016-12",
   "event": "Forum shows Server Edition running on Mono with SQLite (earliest record found this pass)"
  },
  {
   "date": "2020-06-16",
   "event": "Version 20.7.14 adds the new History audit trail with undo across all editions"
  },
  {
   "date": "2022-11",
   "event": "Cloud Edition priced at USD 49/month, up from USD 39 in mid-2020 (forum)"
  },
  {
   "date": "2024-08-30",
   "event": "Added support for bank Financial Data Exchange (FDX)"
  },
  {
   "date": "2025-09-28",
   "event": "Added dark mode (release log)"
  },
  {
   "date": "2026-03-17",
   "event": "Community manager-ai-skills CLI for the API with skill files for coding agents"
  },
  {
   "date": "2026-07-28",
   "event": "Community manager-mcp, read-only by default"
  },
  {
   "date": "2026-10",
   "event": "Cloud Edition at USD 59/month flat; Server Edition purchase path unclear"
  }
 ],
 "relation": "kin",
 "relation_note": "Kin and precedent: the longest-running proof that small businesses will keep their books in a free local file. It is a single-ledger product, not a layer beside several ledgers, and a possible ownpurse source through its API or SQLite backup.",
 "borrow": "A free local edition with no limits, paid for by the hosted multi-user edition. One portable file per business that works on every OS. Flat pricing with unlimited users. An audit trail that records every change, including batch operations and imports.",
 "avoid": "A history that lives in the same mutable file and can be undone away. An API with 1,515 operations and thin documentation, which outside developers had to re-document. Unclear licensing for the self-hosted edition.",
 "sources": [
  {
   "title": "Manager.io home",
   "url": "https://www.manager.io/"
  },
  {
   "title": "Manager.io download page",
   "url": "https://www.manager.io/download"
  },
  {
   "title": "Manager Server Edition",
   "url": "https://www.manager.io/server-edition"
  },
  {
   "title": "Manager Cloud pricing",
   "url": "https://www.manager.cloud/"
  },
  {
   "title": "Build an extension for Manager",
   "url": "https://www.manager.io/extend"
  },
  {
   "title": "Manager OpenAPI specifications",
   "url": "https://www.manager.io/extend/openapi"
  },
  {
   "title": "Manager releases",
   "url": "https://www.manager.io/releases"
  },
  {
   "title": "Earlier Manager site (Sydney footer, Capterra 62 reviewers)",
   "url": "https://www2.manager.io/"
  },
  {
   "title": "New Audit Trail (aka History), forum, June 2020",
   "url": "https://forum.manager.io/t/new-audit-trail-aka-history/28801"
  },
  {
   "title": "manager-mcp, forum, July 2026",
   "url": "https://forum.manager.io/t/manager-mcp-ask-an-ai-assistant-about-your-books-read-only-by-default-open-source/69116"
  },
  {
   "title": "Manager.io CLI and AI agent skills, forum, March 2026",
   "url": "https://forum.manager.io/t/manager-io-cli-ai-agent-skills-open-source-command-line-tool-for-the-api/67336"
  },
  {
   "title": "Manager Server Edition question, forum, October 2026",
   "url": "https://forum.manager.io/t/manager-server-edition/69870"
  },
  {
   "title": "New Cloud Edition pricing, forum, November 2022",
   "url": "https://forum.manager.io/t/new-cloud-edition-pricing/44132"
  }
 ],
 "unverified": [
  "Founding year and the company's legal name.",
  "The developer's full name and whether the company has outside funding.",
  "Current Server Edition price and whether it is now free (forum users unsure, October 2026).",
  "Capterra rating and review count today (Capterra returned 403; 4.8 from 62 is the vendor's 2023 citation).",
  "Whether FDX support means live bank feeds in any market or only file import.",
  "The release log endpoint listed entries only through 15 May 2026, though forum posts show version 26.9.1.0."
 ],
 "researched": "2026-10-10"
}
---

## What it is

Manager is accounting software for small businesses that has been free on the desktop for years. The [download page](https://www.manager.io/download) offers Windows, Mac and Linux builds, 'fully-featured and free forever', with 'no time limits, no usage limits, no ads'. The same program comes in two paid forms: [Cloud Edition](https://www.manager.cloud/), hosted by Manager, and [Server Edition](https://www.manager.io/server-edition), which you run yourself as a web server on port 8080. The earlier site placed the company in Sydney, Australia.

The feature set is broad for a free tool: sales and purchase invoices, bank and cash accounts with statement import and bank rules, inventory with FIFO or weighted average costing, payslips, fixed assets, multi-currency, tax reports, custom fields and themes.

## How it works

Each business is a file. The forum's own error text says 'Manager data is stored in SQLite database', and backups restore on any operating system. Desktop is single-user; the FAQ says that is how Manager makes money, because teams that need multi-user or remote access buy Cloud. Releases are calendar-versioned and frequent (26.9.1.0 in September 2026).

Since version 20.7.14 (June 2020) every business has a [History audit trail](https://forum.manager.io/t/new-audit-trail-aka-history/28801) that 'captures all changes to database', including batch operations and bank imports, with undo. Early testers reported that undoing a creation deleted the history entry too, and the trail grows the business file.

## Data and access

Manager publishes [1,515 OpenAPI specs](https://www.manager.io/extend/openapi) for its API (546 GET, 396 PUT, 350 POST, 222 DELETE on 10 October 2026). Access tokens are issued per business. [Extensions](https://www.manager.io/extend) are websites loaded in an iframe that ask Manager to call the API for them; on Desktop they can reach every endpoint, on Cloud and Server only what the user may. There are no API fees. Developers on the forum say parameters are poorly documented, api3 and api4 are undocumented, and probing the API crashed the server.

Cloud costs USD 59 a month or USD 590 a year, flat, for unlimited users and businesses, with no discounts. Server Edition used to be a perpetual licence with 12 months of updates and renewal at half price; in October 2026 its page says 'no monthly fees' and users cannot find where to buy a key.

## Agents

Manager has no AI features and no official MCP. The community filled the gap in 2026: a [Bash CLI with agent skill files](https://forum.manager.io/t/manager-io-cli-ai-agent-skills-open-source-command-line-tool-for-the-api/67336) (March) and [manager-mcp](https://forum.manager.io/t/manager-mcp-ask-an-ai-assistant-about-your-books-read-only-by-default-open-source/69116) (July), which is read-only by default because 'handing an AI agent unrestricted access to live accounting records is a bad idea'. A forum regular warned that cloud models see whatever the tool returns.

## Customers

No user counts are published. The site cites 4.8 out of 5 on Capterra (62 reviewers on the 2023 site). The forum is busy and global; many users are small firms in Africa, the Middle East and Asia paying for Cloud in USD.

## History

The forum shows Server Edition on Mono and SQLite by 2016. History arrived in 2020, Cloud rose from USD 39 (mid-2020) to USD 49 (2022) to USD 59 (2026), and FDX bank data support arrived in 2024.

## For ownpurse

Manager is the strongest precedent that small businesses accept a local file for their books, and that a free local edition can be paid for by a hosted one. Its History module is the nearest small-business analogue to ownpurse's record, but it lives inside the file it audits, can be undone, and proves nothing to anyone else. For ownpurse, Manager is a candidate connector: the API or the SQLite backup are both readable, and the community's read-only-first MCP reaches the same conclusion as ownpurse about agent writes, without a sealed approval step.
