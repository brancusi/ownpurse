---
{
 "slug": "erpnext",
 "name": "ERPNext",
 "url": "https://erpnext.com",
 "segment": "incumbent-ledger",
 "status": "active",
 "founded": "2008 (Frappe Technologies, then Web Notes, incorporated; ERPNext 1.0 released June 2010)",
 "hq": "Mumbai, India",
 "backing": "Frappe Technologies Pvt. Ltd., privately held and founder-led (Rushabh Mehta). Ran for about 12 years without outside funding; Rainmatter (Zerodha) invested $1.35M (Rs 10 crore) on 24 November 2020, described as the first round. No later round confirmed.",
 "tagline": "A free, GPL-3.0 open-source ERP with a full double-entry ledger that you can run on your own server or on Frappe Cloud.",
 "scope": "ERPNext covers accounting (general ledger, invoicing, payments, bank reconciliation, budgeting, multi-company consolidation), plus stock, manufacturing, CRM, projects and, through the separate Frappe HRMS app, payroll. It is one system of record you operate, not an aggregator: it does not pull in and reconcile other accounting systems, and its bank feed options are narrow (statement import, a Plaid integration). It has no first-party AI or agent features.",
 "domains": [
  "ledger",
  "bank",
  "payments",
  "invoicing",
  "payroll",
  "reporting",
  "commerce",
  "documents"
 ],
 "customers_served": [
  "smb",
  "mid-market",
  "developers"
 ],
 "spectrum": {
  "custody": {
   "score": 3,
   "note": "Self-hosted, the owner holds the authoritative MariaDB database; on Frappe Cloud the vendor runs it but the owner can download full backups and move off."
  },
  "locality": {
   "score": 2,
   "note": "Self-hostable server stack (Python, MariaDB, Redis, bench or Docker); not a local file you carry on your laptop."
  },
  "history": {
   "score": 2,
   "note": "Submitted GL and stock entries are not edited; cancellation posts reversal rows and amendments create linked documents, and a Version log keeps document diffs, but masters are overwritten and there are no as-of queries."
  },
  "openness": {
   "score": 4,
   "note": "GPL-3.0-only source for ERPNext and the Frappe framework, about 40k GitHub stars."
  },
  "breadth": {
   "score": 3,
   "note": "Books, invoicing, payments, payroll (HRMS), stock and attachments in one database; bank and external-ledger coverage is thin."
  },
  "autonomy": {
   "score": 1,
   "note": "People enter and submit documents; automation is rules, scheduled jobs, auto-repeat and reconciliation matching, not unattended bookkeeping."
  },
  "agent_readiness": {
   "score": 3,
   "note": "Uniform REST and RPC API over every doctype, role-scoped tokens and OAuth, and several community MCP servers (one on the official marketplace); no first-party MCP or agent docs."
  },
  "integration_ease": {
   "score": 3,
   "note": "Self-serve keys, documented metadata-driven API and webhooks; but every instance has its own version and customisations, so a third-party connector must cope with variation."
  },
  "market": {
   "score": 2,
   "note": "Mostly small and mid-sized firms (half of G2 reviewers are small business), with real use in manufacturing and distribution."
  }
 },
 "api": {
  "public": true,
  "style": "REST (/api/resource/:doctype CRUD) plus RPC (/api/method/<dotted.path> for whitelisted Python methods), JSON; webhooks; metadata-driven so every doctype, including custom ones, is exposed the same way.",
  "auth": "API key and secret per user (Authorization: token key:secret), session login, or OAuth 2 bearer tokens. Calls run with the roles of the user who owns the key.",
  "limits": "Self-hosted: no vendor quota, only what your server and any rate_limit settings allow. Frappe Cloud: site plans carry a daily CPU-time allowance for web requests; past it the site answers HTTP 429 until the cycle resets. Default page length is 20 records, paged with limit_start and limit_page_length.",
  "cost": "No API fee. Self-hosting costs your server; on Frappe Cloud the API draws on the plan's CPU-time allowance.",
  "export": "Full database dump plus public and private files via backups (bench backup self-hosted; downloadable backups on Frappe Cloud); per-doctype CSV/Excel through Data Export and reports; the whole system can be restored on another server.",
  "notes": "The API is the same one the web UI uses, so anything a user can do an API client can do, within role permissions. This is the opposite of Xero's rationed per-organisation daily call cap."
 },
 "pricing": {
  "model": "Software free (GPL-3.0); pay for hosting (Frappe Cloud, compute-based per site or per server, not per user) and optionally for partner implementation, product warranty or priority SLA.",
  "entry": "$0 self-hosted; Frappe Cloud sites from $5/month (Rs 410/month), 14-day free trial.",
  "top": "Frappe Cloud dedicated servers from $20 or $40/month depending on page (the two Frappe pages disagree), $125/month for the higher server tier; enterprise by quote.",
  "notes": "Partner implementation, Starter Packs, warranty and SLA prices are not published. Per-plan CPU allowances are not stated in USD on the current pricing page; an AWS Rs 410 plan listed 0.5 h CPU and 250 MB database. Discounts for education and non-profits on request."
 },
 "agents": {
  "mcp": "community",
  "ai_features": "None first-party: the v16 release page and the ERPNext product page mention no AI or agent features. Community apps fill the gap: Frappe Assistant Core (AGPL-3.0, listed on the Frappe Cloud marketplace, maintained by an individual), the Casys.ai MCP server (MIT, 97 to 120 tools, Feb 2026), erpnext-agent on PyPI, and others.",
  "agent_writes": "Yes, through the same REST and RPC API: an agent can create, update, submit and cancel documents within the roles of its API user. The community MCP servers expose create, submit and update tools.",
  "approval": "No agent-specific approval gate. ERPNext's own draft/submit model (docstatus 0 draft, 1 submitted, 2 cancelled) and its configurable Workflow states let a team give an agent a role that can create drafts but not submit them, so a person submits. The community MCP servers describe no confirmation step of their own.",
  "notes": "The draft-then-submit lifecycle is the nearest thing in an incumbent ledger to ownpurse's sealed, approved packages, and it is enforced by the ledger rather than by the agent tool."
 },
 "integration": {
  "scope": "Product page names 20 integrations: Plaid (banking); Stripe, PayPal, Razorpay, GoCardless, Braintree, M-Pesa, Paytm (payments); Shopify, WooCommerce, Amazon, Unicommerce, Zenoti (commerce); LDAP; Dropbox, Google Drive, S3 backups; Twilio, Exotel. Plus a marketplace of Frappe apps and 200+ implementation partners.",
  "notes": "Reviewers ask for more bank and logistics integrations. Connectors are Frappe apps installed per site, so they run inside the instance rather than in a hosted hub."
 },
 "preference": {
  "scale": "Frappe claims 30,000+ companies have adopted ERPNext (product page, read 2026-10-10) and, in its customer guide, 20,000+ customers on Frappe Cloud with an estimated 3 to 5 times as many self-hosting (figure seen in a search snippet; page could not be loaded). Caplight reported 151 to 197 employees as of February 2025. GitHub: about 40k stars, 13.1k forks (2026-10-10).",
  "ratings": "Capterra 4.5/5 from 137 reviews (updated 13 March 2026; UAE page 4.6 from 139, value for money 4.7, support 4.1). G2 4.2 to 4.3/5 from roughly 47 to 50 reviews, support the lowest-rated area. No Trustpilot or Reddit figures gathered.",
  "sentiment": "Praised for cost (no licence fee), breadth of modules in one system, flexibility and customisation, detailed documentation and an active forum. Complaints: setup takes skill and often a dedicated developer, the UI feels less polished than premium ERPs, reporting quirks, weak bank and logistics integrations, local-compliance work, slow performance on large databases, and weak vendor support."
 },
 "timeline": [
  {
   "date": "2008",
   "event": "Rushabh Mehta incorporates the company (then Web Notes) and starts ERPNext."
  },
  {
   "date": "2010-06",
   "event": "ERPNext 1.0 released; source published on Google Code."
  },
  {
   "date": "2014-02",
   "event": "Version 4 introduces the app architecture on the Frappe framework."
  },
  {
   "date": "2020",
   "event": "Frappe Cloud managed hosting priced publicly (2020 pricing post)."
  },
  {
   "date": "2020-11-24",
   "event": "Rainmatter (Zerodha) invests $1.35M, the first outside funding after 12 years."
  },
  {
   "date": "2022-08-01",
   "event": "Version 14 released."
  },
  {
   "date": "2023-09-10",
   "event": "Version 15 released."
  },
  {
   "date": "2025-12 to 2026-01",
   "event": "Version 16 released (forum dates moved from 6 December 2025 to 12 January 2026): financial report templates, consolidated trial balance, automatic closing stock posting."
  },
  {
   "date": "2026-02-28",
   "event": "Casys.ai posts a community MCP server for ERPNext on the Frappe forum."
  },
  {
   "date": "2026-08-18",
   "event": "v16.32.3 is the latest stable build listed."
  }
 ],
 "relation": "source",
 "relation_note": "ERPNext is a ledger ownpurse could read from, with a clean, uniform API and, when self-hosted, no daily call cap. It is also the clearest case that owner-controlled books already exist: it gives you the server, not a portable, provable record across several systems.",
 "borrow": "The draft/submit/cancel/amend lifecycle as an approval gate enforced by the ledger; cancellation as reversal rows rather than deletion; metadata-driven doctypes that expose every type, custom ones included, through one API; compute-based hosting pricing instead of per-seat fees.",
 "avoid": "Becoming an ERP you must operate: the bench, MariaDB, Redis and upgrade stack is the friction reviewers complain about. Also avoid history that is only partly kept (masters overwritten, version log prunable) and agent access with no agent-specific approval step.",
 "sources": [
  {
   "title": "ERPNext product page",
   "url": "https://frappe.io/erpnext"
  },
  {
   "title": "ERPNext pricing",
   "url": "https://frappe.io/erpnext/pricing"
  },
  {
   "title": "Frappe Cloud pricing",
   "url": "https://frappe.io/cloud/pricing"
  },
  {
   "title": "ERPNext version 16 release page",
   "url": "https://frappe.io/erpnext/version-16"
  },
  {
   "title": "ERPNext, HRMS and Frappe v16 release dates (forum)",
   "url": "https://discuss.frappe.io/t/erpnext-hrms-frappe-framework-v16-release-dates/156349"
  },
  {
   "title": "Frappe framework REST API docs",
   "url": "https://docs.frappe.io/framework/user/en/api/rest"
  },
  {
   "title": "Immutable ledger in ERPNext",
   "url": "https://docs.frappe.io/erpnext/immutable-ledger-in-erpnext"
  },
  {
   "title": "Frappe Cloud pricing customer guide",
   "url": "https://docs.frappe.io/customer-guide/pricing/frappe-cloud-pricing"
  },
  {
   "title": "Frappe and Zerodha announcement",
   "url": "https://frappe.io/blog/announcements/frappe-zerodha"
  },
  {
   "title": "Zerodha: announcing the investment in ERPNext",
   "url": "https://zerodha.com/z-connect/rainmatter/announcing-the-investment-in-erpnext"
  },
  {
   "title": "ERPNext on Wikipedia",
   "url": "https://en.wikipedia.org/wiki/ERPNext"
  },
  {
   "title": "frappe/erpnext on GitHub",
   "url": "https://github.com/frappe/erpnext"
  },
  {
   "title": "MCP server for ERPNext (Casys.ai, Frappe forum)",
   "url": "https://discuss.frappe.io/t/mcp-server-for-erpnext-120-tools-to-connect-ai-agents-to-your-erp/161198"
  },
  {
   "title": "Frappe Assistant Core on the Frappe Cloud marketplace",
   "url": "https://cloud.frappe.io/marketplace/apps/frappe_assistant_core"
  },
  {
   "title": "ERPNext reviews on Capterra",
   "url": "https://www.capterra.com/p/164441/ERPNext/reviews/"
  },
  {
   "title": "ERPNext vs TallyPrime on G2",
   "url": "https://www.g2.com/compare/erpnext-vs-tallyprime"
  },
  {
   "title": "Caplight: Frappe",
   "url": "https://www.caplight.com/company/frappe"
  }
 ],
 "unverified": [
  "Frappe Cloud server entry price: the ERPNext pricing page says $20/month and the Frappe Cloud pricing page says $40/month (both read 2026-10-10).",
  "Exact v16 release date: the forum moved it from 6 December 2025 to 12 January 2026; Wikipedia lists 10 December 2025 for both v16 and v17, which looks wrong.",
  "20,000+ Frappe Cloud customers and the 3 to 5 times self-hosted estimate come from a search snippet of Frappe's customer guide; the page returned 404 when fetched.",
  "Headcount of 151 to 197 (Caplight, February 2025) is third-party; revenue is not disclosed (third-party estimates of about $2.3M for FY2023 were not relied on).",
  "The immutable ledger arriving in version 13 comes from a search summary; the official page names no version.",
  "The Plaid integration's current maintenance status was not checked.",
  "Rate limits on self-hosted instances (Frappe's rate_limit settings) and any REST v2 API details were not confirmed because the search budget ran out.",
  "No Trustpilot or Reddit sentiment was gathered; sentiment comes from Capterra, G2 and SoftwareConnect.",
  "The geographic concentration of users (India, Africa, Middle East) is from general knowledge, not a cited source."
 ],
 "researched": "2026-10-10"
}
---

## What it is

ERPNext is a free, open-source ERP made by [Frappe Technologies](https://frappe.io), a founder-led company in Mumbai. It is licensed GPL-3.0-only and built on the Frappe framework (Python, JavaScript, MariaDB). The accounting module is a full double-entry ledger with invoicing, payments, bank reconciliation, budgeting and, since version 16, [formula-driven financial report templates and a consolidated trial balance](https://frappe.io/erpnext/version-16). Around it sit stock, manufacturing, CRM, projects and, through Frappe HRMS, payroll. Frappe says [30,000+ companies have adopted ERPNext](https://frappe.io/erpnext).

It does one thing ownpurse does not try to do: be the books. It does not gather what other ledgers, banks and wallets report and reconcile them; you run your business inside it.

## How it works

You either self-host (free; the [pricing page](https://frappe.io/erpnext/pricing) recommends it for companies with a tech team) or rent Frappe Cloud, priced by compute rather than by user: sites from $5 a month, servers from $20 or $40 a month (two Frappe pages disagree), enterprise by quote. A network of 200+ partners does implementations at unpublished prices.

Documents move through draft, submitted and cancelled states. Submitted ledger rows are not edited: the [immutable ledger](https://docs.frappe.io/erpnext/immutable-ledger-in-erpnext) keeps the original rows on cancellation and posts opposite rows, and an amendment is a new document linked by an amended-from reference. A Version log records document changes. That is closer to an append-only log than most incumbents, though master records are still overwritten and there is no as-of query.

## Data and access

The [REST API](https://docs.frappe.io/framework/user/en/api/rest) is the one the web UI uses. `/api/resource/:doctype` gives CRUD over every document type, custom ones included, and `/api/method/...` calls whitelisted server methods. Auth is an API key and secret, a session, or OAuth; every call runs with the roles of its user. Self-hosted, there is no vendor quota. On Frappe Cloud, a plan's daily CPU-time allowance for web requests is the limit, and past it the site answers HTTP 429. You can export everything: a full database dump and files from backups, or per-doctype CSV and Excel.

## Agents

Frappe ships no first-party AI or agent features; neither the v16 release page nor the product page mentions any. The community has filled the gap. [Frappe Assistant Core](https://cloud.frappe.io/marketplace/apps/frappe_assistant_core) is an MCP server on the Frappe Cloud marketplace, maintained by an individual, AGPL-3.0. The [Casys.ai server](https://discuss.frappe.io/t/mcp-server-for-erpnext-120-tools-to-connect-ai-agents-to-your-erp/161198) (MIT, February 2026) is read-write: an agent can create, submit and move documents. None of these describes a confirmation step. The real gate is ERPNext itself: give an agent's user a role that can create drafts but not submit them, and a person does the submitting.

## Customers

Users are mostly small and mid-sized firms, many in manufacturing and distribution, with a strong base in India, Africa and the Middle East. [Capterra](https://www.capterra.com/p/164441/ERPNext/reviews/) shows 4.5/5 from 137 reviews (March 2026); G2 shows about 4.2 from around 48. Reviewers praise the cost, the breadth and the customisation. They complain about setup that needs a developer, a less polished UI, weak bank and logistics integrations, slowness on large databases, and support, which is the lowest-rated area on both sites.

## History

Rushabh Mehta started the company in 2008. ERPNext 1.0 shipped in June 2010, and version 4 (2014) split it into apps on the Frappe framework. Frappe ran for twelve years without outside money before [Rainmatter, Zerodha's fund, invested $1.35M](https://frappe.io/blog/announcements/frappe-zerodha) in November 2020. Zerodha had already built its back office on Frappe without ever contacting the company. Versions 14 (2022), 15 (2023) and 16 (late 2025 or January 2026) followed.

## For ownpurse

ERPNext shows that owner-controlled, open-source books work at scale. It also shows the cost: you have to run an ERP. Its custody comes from operating a server stack, not from holding a file, and it keeps one system's books, not a provable record across several. That gap is ownpurse's: you keep your ERPNext, Xero or QuickBooks, and ownpurse holds the portable, hash-chained record across all of them. As a source, ERPNext is a good connector target: a uniform API, documented auth and no daily call cap when self-hosted. Two ideas are worth borrowing. Draft and submit, enforced by the ledger, is the incumbent version of a sealed, approved package. A correction posted as reversal rows keeps the history readable.
