---
{
 "slug": "zoho-books",
 "name": "Zoho Books",
 "url": "https://www.zoho.com/books/",
 "segment": "incumbent-ledger",
 "status": "active",
 "founded": "2011 (Zoho Books launch); Zoho Corporation founded 1996 as AdventNet, renamed 2009",
 "hq": "Chennai, India (Zoho Corporation); US headquarters Del Valle, Texas",
 "backing": "Zoho Corporation, privately held and bootstrapped with no external investment; FY2025 revenue Rs 12,313 crore and net income Rs 3,191 crore (per Wikipedia, citing filings)",
 "tagline": "A low-cost cloud ledger for small businesses, part of the Zoho suite, with a self-serve REST API and an official MCP server.",
 "scope": "Double-entry bookkeeping, invoicing, bills, bank feeds and reconciliation, inventory, projects, multi-currency and country tax editions (GST in India, VAT and e-invoicing in the Gulf and Europe). It does not do payroll inside Books (Zoho Payroll is a separate product), and it runs only in Zoho's cloud. Its pull is the rest of the Zoho suite: CRM, Expense, Inventory and Billing share one login and one data centre.",
 "domains": [
  "ledger",
  "invoicing",
  "bank",
  "payments",
  "tax",
  "documents"
 ],
 "customers_served": [
  "freelancer",
  "smb",
  "accountants"
 ],
 "spectrum": {
  "custody": {
   "score": 1,
   "note": "Zoho holds the authoritative books; you can export CSV and read everything through the API, but nothing gives you a held, verifiable copy."
  },
  "locality": {
   "score": 0,
   "note": "Cloud only, in one of nine regional Zoho data centres; no self-hosted or desktop edition."
  },
  "history": {
   "score": 1,
   "note": "Transactions carry a history and comments trail in the app, but the API returns current state and edits overwrite."
  },
  "openness": {
   "score": 2,
   "note": "Closed source, but a public REST API reachable on every plan including Free, with self-serve OAuth and CSV export."
  },
  "breadth": {
   "score": 3,
   "note": "Ledger, invoicing, bank feeds, payments, receipt scanning and tax filing in one product; payroll and spend sit in sibling Zoho apps."
  },
  "autonomy": {
   "score": 1,
   "note": "Zia answers questions and flags anomalies, and bank rules auto-categorise; people still post and approve."
  },
  "agent_readiness": {
   "score": 3,
   "note": "Official Zoho MCP covers Books, with fine-grained OAuth scopes; held back by per-organisation daily call caps and no human approval gate."
  },
  "integration_ease": {
   "score": 3,
   "note": "Self-serve developer console and a 'self client' for scripts, documented REST; the nine data-centre domains and daily caps are the main friction."
  },
  "market": {
   "score": 2,
   "note": "Freelancers to small businesses, with a free plan under US$50K revenue; not built for multi-entity mid-market."
  }
 },
 "api": {
  "public": true,
  "style": "REST/JSON under https://www.zohoapis.com/books/v3, organization_id on every call, region-specific domains (.com, .eu, .in, .com.au, .jp, .ca, .com.cn, .sa, .ae)",
  "auth": "Zoho OAuth 2.0; scopes per module and operation (e.g. ZohoBooks.invoices.READ); access tokens last one hour, refresh tokens until revoked; 'self client' grant for backend scripts; no app review mentioned",
  "limits": "100 requests per minute per organisation; daily caps by plan: Free 1,000, Standard 2,000, Professional 5,000, Premium/Elite/Ultimate 10,000; 5 concurrent calls on Free, 10 (soft) on paid; HTTP 429 when exceeded",
  "cost": "No separate API fee; API access comes with every plan, including Free. Higher daily caps require a higher plan.",
  "export": "CSV/XLS export of modules from the app, bulk export endpoints for some documents (estimates, sales orders, invoices) and full read access through the API.",
  "notes": "No sandbox and no webhook section in the API introduction; workflow rules in the app can call webhooks (not confirmed in this pass). The daily caps are low for a full history pull on lower plans."
 },
 "pricing": {
  "model": "Per organisation per month, tiered by users and document volume; extra users US$3/month (US$2.50 annual)",
  "entry": "Free (1 user plus 1 accountant, 1,000 invoices a year, revenue up to US$50K); Standard US$20/month or US$15 billed annually",
  "top": "Ultimate US$275/month or US$240 billed annually (15 users, 100,000 invoices a year)",
  "notes": "US prices from zoho.com/us/books/pricing, October 2026, excluding local taxes. Regional prices differ: the UAE page lists Standard at AED69/month and Ultimate at AED799/month."
 },
 "agents": {
  "mcp": "official",
  "ai_features": "Zia: plain-English questions over the books, anomaly detection, automated collection reminders; receipt autoscan (50 a month on Free).",
  "agent_writes": "Yes. Through Zoho MCP (and the REST API) an agent can create and update invoices, bills, expenses and contacts within its OAuth scopes. Third-party MCP servers (StackOne, Composio, Knit, open-source projects) do the same.",
  "approval": "None built into the agent path. Zoho MCP relies on user-level permissions, OAuth and audit trails; Books has approval workflows for some documents, but no gate specific to agent writes was found.",
  "notes": "Zoho MCP lets you assemble an agent endpoint across Zoho apps and 500+ third-party apps; pricing and launch date are not stated on its page."
 },
 "integration": {
  "scope": "Native integration with the Zoho suite (CRM, Inventory, Expense, Billing, Payroll, Analytics); bank feeds; payment gateways; a marketplace of third-party apps; covered by unified APIs and iPaaS tools.",
  "notes": "Easy to start with: register a client, pick scopes, call. Every request needs the right regional domain and organisation id. Unified APIs often list Zoho Books, so coverage exists elsewhere."
 },
 "preference": {
  "scale": "Zoho Corporation reported more than one million paying organisations and 150 million users across its products by February 2026 (Wikipedia). No Zoho Books-specific customer count is published on its site.",
  "ratings": "Zoho Corporation on Trustpilot: 3.8/5 from about 6,155 reviews (October 2026), mostly about other Zoho products. G2 and Capterra figures for Zoho Books could not be retrieved in this pass.",
  "sentiment": "Praised for price, breadth for the money and an interface that is easy to learn; complaints centre on slow or scripted support, account restrictions and occasional bank-feed breakage (Trustpilot, company-wide)."
 },
 "timeline": [
  {
   "date": "1996",
   "event": "AdventNet founded in New Jersey by Sridhar Vembu, Tony Thomas and Shailesh Kumar Davey"
  },
  {
   "date": "2009",
   "event": "AdventNet renamed Zoho Corporation"
  },
  {
   "date": "2011",
   "event": "Zoho Books launched"
  },
  {
   "date": "January 2025",
   "event": "Shailesh Kumar Davey succeeds Sridhar Vembu as Zoho CEO; Vembu becomes chief scientist"
  },
  {
   "date": "February 2026",
   "event": "Zoho reports over one million paying organisations and 150 million users"
  },
  {
   "date": "October 2026",
   "event": "Zoho Books sells editions in 18 countries; UAE edition is an accredited e-invoicing service provider with EmaraTax filing"
  }
 ],
 "relation": "source",
 "relation_note": "A future connector, not a rival. Zoho Books is where many small businesses in India, the Gulf and elsewhere keep their books; ownpurse would read it into the owner's record and send approved writes back.",
 "borrow": "Fine-grained, per-module OAuth scopes (READ, CREATE, UPDATE, DELETE) are the right model for what an agent may do; publishing rate limits per plan in one table is a good habit for ownpurse's own docs.",
 "avoid": "Suite lock-in: Books is cheapest and most useful when you also use Zoho CRM, Inventory and Payroll, which makes leaving expensive. ownpurse should make the record useful on its own and never depend on a suite.",
 "sources": [
  {
   "title": "Zoho Books pricing (US)",
   "url": "https://www.zoho.com/us/books/pricing/"
  },
  {
   "title": "Zoho Books API v3 introduction",
   "url": "https://www.zoho.com/books/api/v3/introduction/"
  },
  {
   "title": "Zoho Books API OAuth",
   "url": "https://www.zoho.com/books/api/v3/oauth/"
  },
  {
   "title": "Zoho Books home",
   "url": "https://www.zoho.com/books/"
  },
  {
   "title": "Zoho Books UAE",
   "url": "https://www.zoho.com/ae/books/"
  },
  {
   "title": "Zoho MCP",
   "url": "https://www.zoho.com/mcp/"
  },
  {
   "title": "Zoho Corporation (Wikipedia)",
   "url": "https://en.wikipedia.org/wiki/Zoho_Corporation"
  },
  {
   "title": "Zoho on Trustpilot",
   "url": "https://www.trustpilot.com/review/zoho.com"
  },
  {
   "title": "StackOne Zoho Books MCP",
   "url": "https://stackone.com/connectors/zohobooks/mcp/"
  }
 ],
 "unverified": [
  "Zoho Books-specific customer count (none published)",
  "G2 and Capterra ratings for Zoho Books (pages blocked in this pass)",
  "Zoho MCP launch date and pricing",
  "Whether Zoho Books workflow rules can send webhooks on change",
  "Exact behaviour and limits of Zoho Books' full data backup export",
  "Web search budget ran out mid-research; Zoho-specific press on agent features after 2025 was not searched"
 ],
 "researched": "2026-10-10"
}
---

## What it is

Zoho Books is the accounting app inside Zoho, a privately held, bootstrapped software company based in Chennai. It launched in 2011 and is sold in 18 country editions, with local tax built in: GST in India, VAT, corporate tax and e-invoicing in the UAE and Saudi Arabia, VERI*FACTU in Spain. It is one of the cheapest full ledgers you can buy, and the [pricing page](https://www.zoho.com/us/books/pricing/) starts at free.

## How it works

You get double-entry books, invoicing, bills, bank feeds and reconciliation, inventory, projects and multi-currency in one cloud app. Payroll, expenses and CRM are sibling products that share the login. That is the strength and the catch: Books gets more useful the more of Zoho you use, and harder to leave.

Plans in October 2026, per organisation per month in US dollars: Free (one user plus an accountant, revenue up to US$50K, 1,000 invoices a year), Standard US$20 (US$15 annual), Professional US$50, Premium US$70, Elite US$150 and Ultimate US$275 (US$240 annual). Regional prices differ; the [UAE page](https://www.zoho.com/ae/books/) lists Standard at AED69.

## Data and access

The [API](https://www.zoho.com/books/api/v3/introduction/) is plain REST over JSON. Every call carries an organisation id and must go to the right one of nine regional domains. Auth is [Zoho OAuth](https://www.zoho.com/books/api/v3/oauth/) with scopes per module and operation, one-hour access tokens and refresh tokens that last until revoked. A "self client" lets a script on your own machine get a token without a web app, and no app review is mentioned.

The limits are the constraint: 100 requests a minute per organisation, and a daily cap set by plan (1,000 on Free, 2,000 on Standard, 5,000 on Professional, 10,000 above). There is no separate API fee, which is better than Xero or QuickBooks, but a full history pull on a small plan will take days. The API returns current state; the in-app history trail is not exposed as versions.

## Agents

Zoho runs its own [Zoho MCP](https://www.zoho.com/mcp/), which lets you stand up an agent endpoint over Books and other Zoho apps. Agents work under the user's permissions and OAuth, with audit trails; there is no approval gate specific to agent writes. Third parties such as StackOne, Composio and Knit host Zoho Books MCP servers too. Zia, Zoho's assistant, answers questions in plain English and flags anomalies.

## Customers

Zoho reported more than one million paying organisations and 150 million users across all products by February 2026, according to [Wikipedia](https://en.wikipedia.org/wiki/Zoho_Corporation). There is no published count for Books alone. Zoho's [Trustpilot](https://www.trustpilot.com/review/zoho.com) score is 3.8 from about 6,155 reviews, but most of them are about other Zoho products. Common praise: price and breadth. Common complaints: slow support and account restrictions.

## History

Zoho started as AdventNet in 1996 and was renamed in 2009. Books followed in 2011. In January 2025 Shailesh Kumar Davey took over as CEO from Sridhar Vembu.

## For ownpurse

Zoho Books is a good second- or third-wave connector. The API is open on every plan, the OAuth scopes are granular, and a self client fits a local tool. Plan the sync around the daily cap: incremental by modified time, patient backfill. Zoho's market is the one Xero is thin in (India, the Gulf), and these owners are as exposed to suite lock-in as anyone. Moving between Zoho and another ledger is the kind of sentence ownpurse wants to make easy.
