---
{
 "slug": "freshbooks",
 "name": "FreshBooks",
 "url": "https://www.freshbooks.com",
 "segment": "incumbent-ledger",
 "status": "active",
 "founded": "2003",
 "hq": "Toronto, Canada",
 "backing": "Venture-backed, private. $30M (July 2014, led by Oak Investment Partners), $43M (July 2017), a strategic investment from JPMorgan Chase (August 2019), $130M in August 2021 ($80.75M Series E plus $50M debt) at a valuation over $1B, and a debt facility of up to $100M (August 2022).",
 "tagline": "Invoicing-first accounting software for freelancers, solopreneurs and small service businesses.",
 "scope": "FreshBooks covers invoicing, estimates, payments, expenses and receipts, time tracking, projects, bills, payroll (US) and double-entry accounting with a chart of accounts, journal entries, bank reconciliation and standard reports. It is built for service businesses that bill clients. It is not a multi-entity ledger, has no inventory depth, and holds your books only in its cloud.",
 "domains": [
  "invoicing",
  "payments",
  "ledger",
  "bank",
  "payroll",
  "documents",
  "reporting"
 ],
 "customers_served": [
  "freelancer",
  "smb",
  "accountants"
 ],
 "spectrum": {
  "custody": {
   "score": 1,
   "note": "FreshBooks' cloud holds the record; a full copy can be pulled through the free API, including journal entries and a general ledger report."
  },
  "locality": {
   "score": 0,
   "note": "Cloud only."
  },
  "history": {
   "score": 1,
   "note": "Webhooks report create, update and delete events, but no documented audit trail or versioned history is exposed."
  },
  "openness": {
   "score": 2,
   "note": "Free public JSON API across clients, invoices, expenses, bills, payments, journal entries and accounting reports; closed source."
  },
  "breadth": {
   "score": 2,
   "note": "Invoicing, payments, expenses, time, bank feeds, payroll and the ledger in one place for a small business; no multi-entity or inventory."
  },
  "autonomy": {
   "score": 1,
   "note": "Rules, recurring invoices and automatic reminders; the official AI connector only reads."
  },
  "agent_readiness": {
   "score": 3,
   "note": "Official Anthropic-verified MCP connector (20 read-only tools, September 2026) plus a scoped OAuth API with read and write scopes per object."
  },
  "integration_ease": {
   "score": 3,
   "note": "Self-serve app registration, OAuth 2.0, webhooks with HMAC signatures, no daily request cap; rate thresholds unpublished and public app listing reviewed."
  },
  "market": {
   "score": 1,
   "note": "Freelancers and small service businesses; a Select tier for larger small businesses."
  }
 },
 "api": {
  "public": true,
  "style": "REST-style JSON over HTTPS (api.freshbooks.com), with webhooks.",
  "auth": "OAuth 2.0 authorization code with refresh tokens; scopes per object in the form user:<object>:read or user:<object>:write across 22 objects, including journal_entries and reports.",
  "limits": "No daily cap on requests; bursts are rate-limited (HTTP 429) with thresholds not published. List methods return at most 100 results per page.",
  "cost": "No API fee found. A connected business needs a paid FreshBooks plan.",
  "export": "Through the API: clients, invoices, estimates, payments, expenses, bills, time, journal entries, chart of accounts and reports (balance sheet, general ledger, trial balance, cash flow, aging). In the app, reports download for your accountant.",
  "notes": "Webhooks cover create, update and delete for bills, clients, credit notes, estimates, expenses, invoices, items, payments, projects, recurring profiles, services, taxes and time entries, plus send-by-email events; callbacks must answer within 10 seconds and are verified with an HMAC-SHA256 header. Public App Store apps are reviewed; payment-gateway apps are prohibited."
 },
 "pricing": {
  "model": "Monthly or yearly subscription per business, tiered by number of billable clients, with paid add-ons.",
  "entry": "Lite: $23 a month list for 5 billable clients (promotion of $1 a month for the first year, October 2026).",
  "top": "Premium: $70 a month list for unlimited clients; Select is custom-priced and includes 2 team members.",
  "notes": "Team members $11 a month each; Advanced Payments $20 a month; Payroll $40 a month plus $6 per person. Plus is $43 a month for 50 clients. 30-day free trial; yearly billing about 10% off (pricing page, October 2026)."
 },
 "agents": {
  "mcp": "official",
  "ai_features": "An official FreshBooks connector in Claude's directory (mcp.freshbooks.com/v1), announced 25 September 2026: 20 tools for reports (balance sheet, general ledger, P&L, revenue by client, trial balance), lookups of clients, estimates, expenses, invoices, payments and recurring templates, and listing businesses. FreshBooks also mentions an AI help assistant in support.",
  "agent_writes": "Not through the official connector, which is read-only. An agent can write through the OAuth API with write scopes, or through community MCP servers (one exposes 87 tools).",
  "approval": "No approval gate on API writes beyond the OAuth consent and scopes the owner grants. The official connector avoids the question by not writing.",
  "notes": "FreshBooks reports that 72% of its customers who use AI use Claude (survey of over 600 customers, September 2026). Several third-party MCP servers and hosted options (Truto and others) exist."
 },
 "integration": {
  "scope": "FreshBooks App Store (app count not confirmed), bank feeds, payment gateways (FreshBooks Payments, Stripe, PayPal), payroll, lending partners (Grasshopper, July 2026), Facturama e-invoicing in Mexico, and the unified APIs.",
  "notes": "The public API covers the ledger as well as invoicing, so a third party can read the whole books without scraping."
 },
 "preference": {
  "scale": "FreshBooks says over 500,000 small businesses in over 160 countries have used it (press page, October 2026). Valued over $1B in August 2021.",
  "ratings": "Trustpilot: 3.7 from 1,046 reviews (83 in the last 12 months), October 2026. G2 and Capterra figures could not be retrieved.",
  "sentiment": "Praised for ease of use for invoicing and basic bookkeeping and for friendly phone support. Complaints: held or slow card payouts, hard cancellation and billing after closing, login verification problems, reconciliation and reports judged unreliable by some users needing full accounting, and price rises."
 },
 "timeline": [
  {
   "date": "2003",
   "event": "Founded in Toronto by Mike McDerment, Levi Cooperman and Joe Sawada; self-funded for years."
  },
  {
   "date": "2014-07-23",
   "event": "Raises $30M led by Oak Investment Partners."
  },
  {
   "date": "2016",
   "event": "Rebuilt platform (developed as BillSpring from 2015) folded back into FreshBooks; double-entry accounting follows on the new platform."
  },
  {
   "date": "2017-07-18",
   "event": "Raises $43M."
  },
  {
   "date": "2019-08-09",
   "event": "Strategic investment from JPMorgan Chase."
  },
  {
   "date": "2020-09-24",
   "event": "Acquires Facturama, a Mexican e-invoicing company."
  },
  {
   "date": "2021-01-14",
   "event": "Don Epperson replaces Mike McDerment as CEO."
  },
  {
   "date": "2021-08-10",
   "event": "$130M in funding ($80.75M Series E and $50M debt), valuation over $1B."
  },
  {
   "date": "2022",
   "event": "FreshBooks Classic discontinued."
  },
  {
   "date": "2022-08-03",
   "event": "Debt facility of up to $100M."
  },
  {
   "date": "2026-09-25",
   "event": "Official FreshBooks connector for Claude via MCP, read-only."
  }
 ],
 "relation": "source",
 "relation_note": "FreshBooks is on ownpurse's connector roadmap. Its free, scoped API reads journal entries and the general ledger, so ownpurse can rebuild the books from it and check them against its trial balance, the same way as Xero.",
 "borrow": "Per-object read and write scopes that an owner can see at consent; webhooks that include delete events and are signed; a read-only official agent connector as the default; no daily cap on reads.",
 "avoid": "Unpublished rate thresholds, and a history model where an edit or delete leaves no versioned trace a third party can read.",
 "sources": [
  {
   "title": "FreshBooks pricing",
   "url": "https://www.freshbooks.com/pricing"
  },
  {
   "title": "FreshBooks API: getting started",
   "url": "https://www.freshbooks.com/api/start"
  },
  {
   "title": "FreshBooks API: scopes",
   "url": "https://www.freshbooks.com/api/scopes"
  },
  {
   "title": "FreshBooks API: request limits",
   "url": "https://www.freshbooks.com/api/limits"
  },
  {
   "title": "FreshBooks API: webhooks",
   "url": "https://www.freshbooks.com/api/webhooks"
  },
  {
   "title": "FreshBooks API: requirements for public apps",
   "url": "https://www.freshbooks.com/api/requirement-for-public-apps-on-the-freshbooks-app-store"
  },
  {
   "title": "FreshBooks press releases",
   "url": "https://www.freshbooks.com/press"
  },
  {
   "title": "FreshBooks press: Anthropic MCP connector (25 September 2026)",
   "url": "https://www.freshbooks.com/press/releases/freshbooks-anthropicmcpconnector"
  },
  {
   "title": "FreshBooks press: solopreneur AI survey (September 2026)",
   "url": "https://www.freshbooks.com/press/releases/eraofthesoloprener"
  },
  {
   "title": "Claude connectors: FreshBooks",
   "url": "https://claude.com/connectors/freshbooks"
  },
  {
   "title": "FreshBooks accounting software features",
   "url": "https://www.freshbooks.com/accounting-software"
  },
  {
   "title": "Wikipedia: FreshBooks",
   "url": "https://en.wikipedia.org/wiki/FreshBooks"
  },
  {
   "title": "Trustpilot: FreshBooks",
   "url": "https://www.trustpilot.com/review/freshbooks.com"
  },
  {
   "title": "Glama: community FreshBooks MCP server",
   "url": "https://glama.ai/mcp/servers/bf3wd3ka4u"
  }
 ],
 "unverified": [
  "Founding year: Wikipedia gives both 2003 and 2004.",
  "G2 and Capterra ratings could not be retrieved (403).",
  "Whether FreshBooks keeps an internal audit trail of edits that is not exposed through the API.",
  "Exact burst rate-limit thresholds are not published.",
  "Number of apps in the FreshBooks App Store not confirmed.",
  "Layoffs at FreshBooks in 2022-2023 are recalled from press but were not confirmed in this pass.",
  "Current revenue and paying customer count not found; 500,000+ is 'have used'."
 ],
 "researched": "2026-10-10"
}
---

## What it is

FreshBooks is accounting software built around invoicing, for freelancers, solopreneurs and small service businesses. It started in Toronto in 2003 when its founder wanted a better way to bill clients, and it still leads with invoices, estimates, payments, expenses and time tracking, with double-entry accounting, a chart of accounts, journal entries, bank reconciliation and standard reports on top ([Wikipedia](https://en.wikipedia.org/wiki/FreshBooks), [FreshBooks](https://www.freshbooks.com/accounting-software)). It says over 500,000 small businesses in over 160 countries have used it ([press](https://www.freshbooks.com/press)).

What it does well is make billing easy for someone who is not an accountant, and give an accountant a real ledger to work in.

## How it works

You pay per business, tiered by how many clients you bill. As of October 2026 Lite is $23 a month for 5 clients (on promotion at $1 a month for the first year), Plus $43 for 50, Premium $70 for unlimited, and Select is quoted. Team members cost $11 a month each, Advanced Payments $20 and payroll $40 plus $6 per person ([pricing](https://www.freshbooks.com/pricing)).

The ledger lives in FreshBooks' cloud. Your accountant can edit journal entries and the chart of accounts and download reports, but the product does not describe an audit trail or keep versions you can query.

## Data and access

The API is the best thing about FreshBooks for ownpurse. It is free, self-serve and broad: you register an app, use OAuth 2.0, and request scopes per object, read or write, across 22 objects including journal entries and reports ([scopes](https://www.freshbooks.com/api/scopes)). It covers clients, invoices, payments, expenses, bills, time, projects, the chart of accounts, journal entries and reports including the general ledger, balance sheet and trial balance ([API](https://www.freshbooks.com/api/start)). There is no daily cap; bursts get HTTP 429, with thresholds unpublished, and lists return 100 results a page ([limits](https://www.freshbooks.com/api/limits)).

Webhooks cover create, update and delete for most objects, signed with HMAC-SHA256, and must be answered within 10 seconds ([webhooks](https://www.freshbooks.com/api/webhooks)). Delete events matter: they are the only trace a third party gets that something was removed. A local app cannot receive them without a public endpoint, so polling stays the backbone.

## Agents

On 25 September 2026 FreshBooks announced an official connector for Claude ([press release](https://www.freshbooks.com/press/releases/freshbooks-anthropicmcpconnector)). Claude's directory lists it as built by FreshBooks, with 20 read-only tools: five reports (balance sheet, general ledger, P&L, revenue by client, trial balance) and lookups of clients, invoices, expenses, payments and estimates ([Claude connectors](https://claude.com/connectors/freshbooks)). FreshBooks says 72% of its customers who use AI use Claude. Community MCP servers add write tools on top of the API, and one exposes 87 ([Glama](https://glama.ai/mcp/servers/bf3wd3ka4u)). There is no approval gate on API writes beyond the scopes the owner grants.

## Customers

Trustpilot shows 3.7 from 1,046 reviews as of October 2026 ([Trustpilot](https://www.trustpilot.com/review/freshbooks.com)). People like how easy invoicing is and the friendly phone support. They complain about card payouts held or slow, cancellations that keep billing, login verification, price rises, and, from users who need full accounting, reconciliation and reports they do not trust.

## History

FreshBooks was self-funded for a decade, raised $30M in 2014 and $43M in 2017, rebuilt its platform between 2015 and 2016, took a strategic investment from JPMorgan Chase in 2019, bought Mexican e-invoicing company Facturama in 2020, and became a unicorn with $130M in August 2021. Its founder handed the CEO role to Don Epperson in January 2021. The old platform, FreshBooks Classic, closed in 2022. In 2026 it expanded lending and shipped the Claude connector.

## For ownpurse

FreshBooks is a good second connector. It exposes the ledger, not only invoices, so ownpurse can pull journal entries and the general ledger, rebuild the books to the cent and check them against FreshBooks' own trial balance. Its read-only official connector is a fair model for what a source should offer agents by default. What it lacks is history: an edit overwrites, and a delete is a webhook you may miss. ownpurse's append-only record, with every observed version kept, fills exactly that gap for a freelancer whose books are otherwise one cloud account away from gone.
