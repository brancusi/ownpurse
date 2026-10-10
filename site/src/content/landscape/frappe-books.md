---
{
 "slug": "frappe-books",
 "name": "Frappe Books",
 "url": "https://frappe.io/books",
 "segment": "plain-text-local",
 "status": "pivoted",
 "founded": "2018 (repository created 8 January 2018; first tagged release 0.0.2, 16 July 2018)",
 "hq": "Mumbai, India (Frappe Technologies) [HQ not stated on frappe.io/about; unverified]",
 "backing": "Product of Frappe Technologies, maker of ERPNext and the Frappe Framework. Frappe was bootstrapped until Zerodha's investment arm Rainmatter announced an investment on 23 November 2020; the amount and stake were not disclosed.",
 "tagline": "Free, AGPL-licensed small-business accounting that started as a desktop app keeping one company in one SQLite file, and is now being rebuilt as a self-hosted web app on the Frappe Framework.",
 "scope": "Double-entry books, sales and purchase invoices, payments, journal entries, inventory, point of sale and India GST reports, for freelancers and small businesses. The Electron desktop app (one company, one SQLite file, offline) reached end of life on 2 October 2026; its successor runs on a Frappe site in a browser or as a phone PWA, with only a develop branch so far. It has no bank feeds, no multi-source aggregation and no agent features.",
 "domains": [
  "ledger",
  "invoicing",
  "commerce"
 ],
 "customers_served": [
  "freelancer",
  "smb"
 ],
 "spectrum": {
  "custody": {
   "score": 4,
   "note": "The desktop app kept the books in a SQLite file on the owner's machine; the successor keeps them on the owner's own server. No vendor copy."
  },
  "locality": {
   "score": 3,
   "note": "The desktop app ran entirely on your machine (4), but it is archived; the supported product is now a self-hosted web app (2-3)."
  },
  "history": {
   "score": 2,
   "note": "Submitted documents cannot be edited and cancelling posts reversing ledger entries, but drafts change in place, documents can be deleted, and there is no tamper-evident log."
  },
  "openness": {
   "score": 4,
   "note": "AGPL-3.0 source for both the archived desktop app and the new Frappe app."
  },
  "breadth": {
   "score": 1,
   "note": "One business's books, invoicing, inventory and POS; no bank feeds, documents pipeline or other sources."
  },
  "autonomy": {
   "score": 0,
   "note": "People enter and submit every document; nothing posts on its own."
  },
  "agent_readiness": {
   "score": 2,
   "note": "The desktop app had no API (only the SQLite file); the successor inherits the Frappe Framework REST API. No MCP, CLI or agent docs."
  },
  "integration_ease": {
   "score": 2,
   "note": "An open SQLite schema and, now, Frappe's generic REST API; but no Books-specific API docs, no connectors and a develop-only successor."
  },
  "market": {
   "score": 1,
   "note": "Built for freelancers and small businesses; Frappe points larger needs to ERPNext."
  }
 },
 "api": {
  "public": true,
  "style": "Desktop: none (direct SQLite file). Successor: Frappe Framework REST API over its doctypes.",
  "auth": "Successor: Frappe API key and secret or session login, governed by Frappe roles and permissions. Desktop: none (file access).",
  "limits": "None published for Books; on a self-hosted Frappe site the owner sets them.",
  "cost": "Free. No metering; you pay only for your own server if you self-host the successor.",
  "export": "Desktop: the whole SQLite file, plus report export. Successor: database backups and Frappe's data export on your own server.",
  "notes": "The new repository says only that 'Frappe Framework provides' the database layer and REST API; it gives no Books-specific endpoints. A community migrator moves an Electron .db file into the web version (forum, 7 October 2026)."
 },
 "pricing": {
  "model": "Free and open source (AGPL-3.0); self-hosted",
  "entry": "Free",
  "top": "Free (hosting costs are your own)",
  "notes": "frappe.io/books says 'no subscription'. No Frappe Cloud hosting plan for the new Books was found."
 },
 "agents": {
  "mcp": "none",
  "ai_features": "None.",
  "agent_writes": "Desktop: only by writing to the SQLite file directly, outside the app's rules. Successor: through the Frappe REST API with a user's permissions.",
  "approval": "Frappe's draft, submit and cancel states act as a manual gate for people; there is no agent-specific approval flow.",
  "notes": "Nothing in the product, docs or announcement mentions agents or AI."
 },
 "integration": {
  "scope": "No bank feeds or third-party connectors. Successor sits on a Frappe site beside other Frappe apps (CRM, HRMS were requested on the forum).",
  "notes": "The forum thread shows a community debate over keeping Books small versus adding integrations; the maintainer has confirmed only Custom Fields as an extension point so far."
 },
 "preference": {
  "ratings": "No G2, Capterra or Trustpilot rating found (search budget exhausted). GitHub: 5,000 stars and 1,158 forks on the desktop repo as of 10 October 2026; about 591,000 release-asset downloads across all desktop releases.",
  "scale": "No user counts published. Download count above is a proxy, not users.",
  "sentiment": "The maintainer's end-of-life note says 'we've heard of nothing but good things about Books'. The main complaint, in Frappe's own words, is that the desktop app 'went unmaintained for a while, due to lack of an owner'. On the new version, users ask for a stable release (it needs Frappe v17 or later), a desktop version (unanswered) and a clear upgrade path."
 },
 "timeline": [
  {
   "date": "2018-01-08",
   "event": "Repository frappe/books created"
  },
  {
   "date": "2018-07-16",
   "event": "First tagged release, 0.0.2"
  },
  {
   "date": "2019-12-13",
   "event": "v0.0.3 beta series of the Electron desktop app"
  },
  {
   "date": "2020-11-23",
   "event": "Zerodha's Rainmatter announces an investment in Frappe/ERPNext (amount undisclosed)"
  },
  {
   "date": "2025-11-06",
   "event": "v0.36.0, the last desktop release"
  },
  {
   "date": "2026-08-28",
   "event": "frappe/frappe-books repository created for the Frappe Framework version"
  },
  {
   "date": "2026-10-02",
   "event": "End-of-life announcement; desktop repository archived"
  },
  {
   "date": "2026-10-03",
   "event": "Forum post 'Announcing a new Frappe Books': develop branch only, PWA phone app"
  }
 ],
 "relation": "kin",
 "relation_note": "Kin: a small-business ledger whose books were a local SQLite file the owner held. It shows that SMB users will accept a local file, and also what happens when one vendor-maintained desktop app loses its maintainer. It is a possible source for an ownpurse connector (read the SQLite file), not a competitor.",
 "borrow": "One company in one SQLite file the owner can copy, open and back up. Submit-and-cancel discipline, where a posted document is frozen and a reversal is a new entry. A permissive data path: the file is the export.",
 "avoid": "A desktop app that depends on one maintainer, with no plan for the owner's data when the app is retired. Mutable drafts and hard deletes beside an otherwise immutable ledger. A rewrite that leaves desktop users with a develop branch and a community migration script.",
 "sources": [
  {
   "title": "Frappe Books product page",
   "url": "https://frappe.io/books"
  },
  {
   "title": "frappe/books repository (archived desktop app)",
   "url": "https://github.com/frappe/books"
  },
  {
   "title": "End of Life announcement, frappe/books issue 1566",
   "url": "https://github.com/frappe/books/issues/1566"
  },
  {
   "title": "frappe/books releases",
   "url": "https://github.com/frappe/books/releases"
  },
  {
   "title": "frappe/frappe-books repository (Frappe Framework version)",
   "url": "https://github.com/frappe/frappe-books"
  },
  {
   "title": "Announcing a new Frappe Books (Frappe forum)",
   "url": "https://discuss.frappe.io/t/announcing-a-new-frappe-books/165047"
  },
  {
   "title": "Frappe Books documentation",
   "url": "https://docs.frappe.io/books"
  },
  {
   "title": "About Frappe",
   "url": "https://frappe.io/about"
  },
  {
   "title": "Rainmatter: announcing the investment in ERPNext (Zerodha)",
   "url": "https://zerodha.com/z-connect/rainmatter/announcing-the-investment-in-erpnext"
  }
 ],
 "unverified": [
  "Frappe Technologies HQ in Mumbai: not stated on the about page.",
  "Amount and stake of the Rainmatter investment (not disclosed).",
  "Whether the desktop app had a CSV data import tool and change-version log (not confirmed this pass).",
  "Which database engines the new Frappe Books supports (README does not name them).",
  "Any review-site ratings: not searched because the web search budget was used up."
 ],
 "researched": "2026-10-10"
}
---

## What it is

Frappe Books is free, open-source accounting for freelancers and small businesses from [Frappe Technologies](https://frappe.io/about), the company behind ERPNext. For most of its life it was an Electron desktop app: the [archived README](https://github.com/frappe/books) says all financial data, transactions and configuration lived 'in an SQLite file on the user's machine'. It covered double-entry books, invoices, bills, payments, journal entries, the four standard reports, and later inventory and point of sale.

On 2 October 2026 the maintainer posted an [end-of-life notice](https://github.com/frappe/books/issues/1566) and archived the desktop repository. Its successor, [frappe/frappe-books](https://github.com/frappe/frappe-books), puts the same interface on a Frappe site: you sign in from a browser or install a phone PWA, and Frappe handles users, roles and permissions on your own server. As of the [forum announcement](https://discuss.frappe.io/t/announcing-a-new-frappe-books/165047) (3 October 2026), only a develop branch exists, it needs Frappe v17 or later, and the stable timeline is 'unclear'.

## How it works

The desktop app was one company in one file on one computer. Documents move from draft to submitted; per the [docs](https://docs.frappe.io/books), a submitted document 'made the ledger entries' and cannot be edited, and cancelling reverses those entries. Drafts can be changed and documents deleted. There are no bank feeds; data arrives by hand or import.

## Data and access

The desktop app had no API. The way in was the SQLite file itself, which is also the full export. The successor inherits the Frappe Framework's REST API and database layer, but its README names no Books-specific endpoints and no database engine. A community member shared a migrator from the Electron `.db` file to the web version on 7 October 2026. Everything is free under AGPL-3.0; there is no metering.

## Agents

Nothing. No MCP, no AI features, no agent docs. An agent could read the desktop SQLite file directly, or call Frappe's REST API on the new version with a user's permissions. Frappe's draft and submit states are the only gate, and they were designed for people.

## Customers

No user counts are published. The desktop repository had 5,000 stars and 1,158 forks, and its releases were downloaded about 591,000 times (GitHub, 10 October 2026). The maintainer wrote that they had 'heard of nothing but good things about Books'. The weak point was ownership: the forum post says the app 'went unmaintained for a while, due to lack of an owner'. No review-site ratings were found.

## History

The repository dates from January 2018; Frappe describes Books as its 2018 'desktop version of ERPNext'. Tagged releases ran from 0.0.2 (July 2018) to v0.36.0 (6 November 2025). Rainmatter, Zerodha's investment arm, [invested in Frappe](https://zerodha.com/z-connect/rainmatter/announcing-the-investment-in-erpnext) in November 2020, after years of bootstrapping. The rebuild on the Frappe Framework began in August 2026.

## For ownpurse

Frappe Books is the clearest small-business precedent for books as a local SQLite file. It shows that a free, local, open ledger can find an audience, and it shows the risk ownpurse is built against: when the maintainer moved on, owners were left with an archived app and a community migration script. Your data survived only because it was a file you held.

For ownpurse it is kin and a possible source. A Frappe Books connector would read the SQLite file (or the new REST API) and keep every version as facts, which neither Books version does: drafts overwrite and deletes remove. The submit-and-cancel rule is worth noting as a familiar pattern for posting as a recorded moment.
