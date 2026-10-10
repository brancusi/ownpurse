---
{
 "slug": "xero",
 "name": "Xero",
 "url": "https://www.xero.com",
 "segment": "incumbent-ledger",
 "status": "active",
 "founded": "2006",
 "hq": "Wellington, New Zealand",
 "backing": "Public company, listed on the ASX (NZX listing June 2007, ASX added November 2012, NZX delisted February 2018). Acquired Melio for US$2.5 billion upfront, completed 15 October 2025.",
 "tagline": "Cloud accounting software for small businesses and their accountants, strongest in Australia, New Zealand and the UK.",
 "scope": "Xero keeps a small business's general ledger in its cloud: invoicing, bills, bank feeds and reconciliation, reporting, multi-currency, and payroll in some regions, with Hubdoc capture, Syft analytics and Melio bill pay around it. It does not let you hold the authoritative record yourself, does not expose unreconciled bank statement lines through its public API, and gates its append-only Journals feed to the top paid developer tier.",
 "domains": [
  "ledger",
  "bank",
  "invoicing",
  "payments",
  "payroll",
  "documents",
  "tax",
  "reporting"
 ],
 "customers_served": [
  "freelancer",
  "smb",
  "accountants",
  "developers"
 ],
 "spectrum": {
  "custody": {
   "score": 1,
   "note": "Xero holds the authoritative books; you can pull copies through the API or report exports, but there is no full-ledger export and read-only access after cancelling is not a standard option."
  },
  "locality": {
   "score": 0,
   "note": "Cloud only."
  },
  "history": {
   "score": 2,
   "note": "Documents carry a History and Notes trail and the Journals endpoint is an append-only feed, but API documents return current state and Journals need the Advanced tier."
  },
  "openness": {
   "score": 2,
   "note": "Public, documented REST API and an MIT-licensed MCP server; the product is closed and bulk export is limited to reports and the API."
  },
  "breadth": {
   "score": 3,
   "note": "Books, bank feeds, invoicing, bill pay (Melio), payroll in some regions, document capture (Hubdoc) and analytics (Syft) in one product family."
  },
  "autonomy": {
   "score": 2,
   "note": "Bank rules, auto-reconciliation on paid plans and the JAX agent automate routine work; most posting is still confirmed by a person."
  },
  "agent_readiness": {
   "score": 3,
   "note": "Official open-source MCP server with create and update tools, granular OAuth scopes and a Claude partnership; no approval gate and no hosted endpoint documented."
  },
  "integration_ease": {
   "score": 3,
   "note": "Self-serve developer signup with a free Starter tier and clear docs; egress fees, daily call limits and the gated Journals feed add cost and friction."
  },
  "market": {
   "score": 2,
   "note": "Small businesses and the accountants who serve them."
  }
 },
 "api": {
  "public": true,
  "style": "REST (JSON or XML) over OAuth 2.0; separate Accounting, Payroll, Files, Assets, Projects and Bank Feeds APIs; webhooks for a few object types",
  "auth": "OAuth 2.0 (authorization code, PKCE) with granular scopes; Custom Connections (client credentials) for a single organisation",
  "limits": "Per organisation: 5 concurrent calls, 60 calls a minute, 1,000 calls a day on Starter and 5,000 a day on paid tiers; 10,000 calls a minute across all tenants per app; HTTP 429 on breach.",
  "cost": "From 2 March 2026, tiers by connections and egress (AUD, ex tax): Starter free (5 connections), Core A$35 a month (50, 10 GB), Plus A$245 (1,000, 50 GB), Advanced A$1,445 (10,000, 250 GB), Enterprise on application; overage A$2.40 per GB; ingress unlimited; the organisation endpoint is not metered. The 15% App Store revenue share was retired.",
  "export": "Reports to PDF, CSV/Excel or Google Sheets (Account Transactions is the closest to a full transaction list); attachments, payroll filings and invoice PDFs need separate handling; the API is the only bulk path.",
  "notes": "Terms in force for new apps from 4 December 2025 and for all from 2 March 2026 ban training AI or ML models on API data and ban bots or browser extensions that simulate users. The Journals endpoint needs the Advanced tier, an initial and annual security assessment and use-case approval; Manual Journals stay available on every tier. Bank statement lines are not exposed except through the lending-only Finance API."
 },
 "pricing": {
  "model": "Monthly subscription per organisation by plan, with paid add-ons (payroll, auto-reconciliation on the lowest plan, inventory).",
  "entry": "US Early plan US$27 a month (20 invoices, 5 bills), with 80% off the first 3 months for new US customers as of 2026-10-10.",
  "top": "US Established plan US$97 a month (multi-currency, projects, expenses, 180-day forecast).",
  "notes": "UK prices rose on 1 September 2025 (Grow £33 to £37, Comprehensive £47 to £50, Ultimate £59 to £65). Australian price rises in 2024 drew public backlash from users and accountants. US payroll is Gusto-powered at US$36 a month plus US$6 per person."
 },
 "agents": {
  "mcp": "official",
  "ai_features": "JAX (Just Ask Xero), described as a financial superagent at Xerocon Brisbane on 3 September 2025, answering questions over cash flow, P&L and balance sheet and orchestrating agents; Claude-powered insights under a multi-year Anthropic partnership announced 27 March 2026; XeroForce, a natural-language agent builder, in invite-only alpha from 14 May 2026; an OpenAI partnership for web research (September 2025).",
  "agent_writes": "Yes. The official MCP server (npm @xeroapi/xero-mcp-server) has create tools for bank transactions, contacts, credit notes, invoices, items, manual journals, payments, quotes and tracking, update tools (invoice, quote and credit note updates only on drafts), and payroll timesheet approve and delete.",
  "approval": "None in the MCP server; a write runs when the agent calls the tool. Xero's own release says one-click actions from Claude.ai will come later. Draft status on invoices is the nearest thing to a gate.",
  "notes": "The MCP server runs locally over stdio, authenticated by a Custom Connection or a bearer token. Xero says data shared with Claude is used only within the session and never used to train Claude."
 },
 "integration": {
  "scope": "Xero App Store of third-party apps (Dext, Syft, Stripe and others top the 2026 Power Lists); bank feeds; Hubdoc; Melio bill pay; Gusto payroll in the US. Xero says up to 78% of customers use apps (January 2026).",
  "notes": "Webhooks cover only contacts, invoices, credit notes, overpayments and prepayments and need a public HTTPS endpoint, so a local app must poll. UpdatedDateUTC does not move for some edits, so a periodic full sweep is needed."
 },
 "preference": {
  "scale": "4.92 million customers at 31 March 2026, up from 4.41 million; FY26 operating revenue NZ$2.8 billion, up 31% (reported 14 May 2026).",
  "ratings": "Trustpilot (UK) 4.1 from 11,430 reviews (2026-10-10); Capterra 4.4 from about 3,295 reviews (April 2026).",
  "sentiment": "Praised for ease of daily bookkeeping, bank feeds, the app ecosystem and onboarding staff. Complaints are about repeated price rises without matching product change, slow or ticket-only support, payroll and bank-feed setup, and billing or cancellation friction (Trustpilot, Accounting Times 2024, AccountingWEB)."
 },
 "timeline": [
  {
   "date": "2006-07",
   "event": "Founded in Wellington by Rod Drury and Hamish Edwards as Accounting 2.0"
  },
  {
   "date": "2007-06",
   "event": "Listed on the NZX"
  },
  {
   "date": "2012-11",
   "event": "Listed on the ASX"
  },
  {
   "date": "2018-02",
   "event": "Delisted from the NZX; ASX only"
  },
  {
   "date": "2018-08",
   "event": "Acquired Hubdoc (document capture)"
  },
  {
   "date": "2023-02",
   "event": "Sukhinder Singh Cassidy became CEO"
  },
  {
   "date": "2024-09",
   "event": "Acquired Syft Analytics for up to US$70 million"
  },
  {
   "date": "2025-09-03",
   "event": "JAX relaunched as a financial superagent at Xerocon Brisbane"
  },
  {
   "date": "2025-10-15",
   "event": "Completed the US$2.5 billion Melio acquisition"
  },
  {
   "date": "2025-12-04",
   "event": "New developer pricing and terms announced: egress fees, AI-training ban, Journals gated"
  },
  {
   "date": "2026-03-02",
   "event": "Developer tiers and egress fees took effect; revenue share retired"
  },
  {
   "date": "2026-03-27",
   "event": "Multi-year partnership with Anthropic announced"
  },
  {
   "date": "2026-04-29",
   "event": "Custom Connections created from this date use granular (V2) scopes"
  },
  {
   "date": "2026-05-14",
   "event": "FY26 results (4.92 million customers); XeroForce alpha announced"
  }
 ],
 "relation": "source",
 "relation_note": "ownpurse's first connector. Xero is where the books live; ownpurse reads them, keeps every version locally and sends writes back only as approved packages.",
 "borrow": "Granular read scopes; draft states as a natural staging area for writes; free and unlimited ingress; the remaining-limit headers on every response; the History and Notes trail as facts.",
 "avoid": "Pricing reads by the gigabyte against the owner's own data; an agent write path with no approval step; gating the one append-only feed behind the top tier.",
 "sources": [
  {
   "title": "Xero developer pricing (tiers, egress, limits)",
   "url": "https://developer.xero.com/pricing"
  },
  {
   "title": "Xero developer FAQ: pricing and policy updates",
   "url": "https://developer.xero.com/faq/pricing-and-policy-updates"
  },
  {
   "title": "Xero API limits FAQ",
   "url": "https://developer.xero.com/faq/limits"
  },
  {
   "title": "Xero MCP server (GitHub)",
   "url": "https://github.com/XeroAPI/xero-mcp-server"
  },
  {
   "title": "Xero and Anthropic collaborate (27 March 2026)",
   "url": "https://www.xero.com/media-releases/xero-and-anthropic-collaborate/"
  },
  {
   "title": "Xero introduces XeroForce",
   "url": "https://www.xero.com/media-releases/xero-introduces-xeroforce/"
  },
  {
   "title": "Xerocon Brisbane 2025 announcements (JAX)",
   "url": "https://www.xero.com/media-releases/xerocon-brisbane-xero-announces-new-features-to-drive-greater-growth-impact/"
  },
  {
   "title": "Xero US pricing plans",
   "url": "https://www.xero.com/us/pricing-plans/"
  },
  {
   "title": "Xero FY26 result (Motley Fool AU, 14 May 2026)",
   "url": "https://www.fool.com.au/2026/05/14/xero-fy26-result-revenue-surges-31-but-profit-dips-due-to-melio-acquisition-costs/"
  },
  {
   "title": "Xero completes Melio acquisition (CPA Practice Advisor)",
   "url": "https://www.cpapracticeadvisor.com/2025/10/15/xero-acquires-payments-platform-melio-for-2-5b/170967/"
  },
  {
   "title": "Xero reviews (Trustpilot UK)",
   "url": "https://uk.trustpilot.com/review/xero.com"
  },
  {
   "title": "Xero price changes 2025 (UHY)",
   "url": "https://www.uhy-uk.com/insights/xero-price-changes-2025"
  },
  {
   "title": "The final straw: Xero users balk at another price hike (Accounting Times)",
   "url": "https://www.accountingtimes.com.au/technology/the-final-straw-xero-users-balk-at-another-price-hike"
  },
  {
   "title": "Xero 2026 App Store Power Lists",
   "url": "https://blog.xero.com/small-business-resources/10-app-power-lists-supercharge-2026/"
  },
  {
   "title": "Xero (software), Wikipedia",
   "url": "https://en.wikipedia.org/wiki/Xero_(software)"
  }
 ],
 "unverified": [
  "Total number of apps in the Xero App Store (no current official count found).",
  "G2 rating for the core Xero product (only a secondary figure of 4.4 found).",
  "Capterra figure (4.4 from about 3,295) taken from a search snippet of Capterra pages dated April 2026; the page itself returned 403.",
  "Whether read-only access after cancellation is available, and for how long Xero retains data.",
  "Whether sending Xero data to a hosted model counts as a passthrough to a third party that needs the user's consent under the December 2025 terms.",
  "General availability date of JAX with Claude and of the Xero integration in Claude.ai.",
  "Whether the Enterprise developer tier includes the Journals endpoint."
 ],
 "researched": "2026-10-10"
}
---

## What it is

Xero is cloud accounting software for small businesses and the accountants who serve them. Founded in Wellington in 2006 and listed on the ASX, it reported [4.92 million customers and NZ$2.8 billion in revenue](https://www.fool.com.au/2026/05/14/xero-fy26-result-revenue-surges-31-but-profit-dips-due-to-melio-acquisition-costs/) for the year to 31 March 2026. It is strongest in Australia, New Zealand and the UK and is pushing into the US, where it bought the bill-pay company Melio for [US$2.5 billion, completed 15 October 2025](https://www.cpapracticeadvisor.com/2025/10/15/xero-acquires-payments-platform-melio-for-2-5b/170967/).

What it does well: daily bookkeeping is simple, bank feeds and reconciliation are good, accountants can work across many clients, and the app ecosystem is large. Dext, Syft and Stripe top its [2026 App Store Power Lists](https://blog.xero.com/small-business-resources/10-app-power-lists-supercharge-2026/).

## How it works

Xero holds the ledger in its cloud. You raise invoices and bills, bank feeds bring in transactions, and you reconcile them against what is recorded. US plans run from [US$27 to US$97 a month](https://www.xero.com/us/pricing-plans/) (2026-10-10), with payroll, inventory and auto-reconciliation as add-ons. Hubdoc handles receipts, Syft analytics is now bundled on eligible plans, and Melio pays bills.

## Data and access

The public API is REST over OAuth 2.0 with granular scopes. Per organisation you get 60 calls a minute and [1,000 a day on Starter, 5,000 on paid tiers](https://developer.xero.com/pricing). Since 2 March 2026 developers pay by connections and egress: Starter is free for 5 organisations, Core is A$35 a month for 50 organisations and 10 GB, Advanced is A$1,445 for 10,000 and 250 GB, and overage is A$2.40 per GB. Uploads are free.

The [terms that came with it](https://developer.xero.com/faq/pricing-and-policy-updates) ban training AI on API data and ban bots that simulate users. The Journals endpoint, the closest thing Xero has to an append-only feed, needs the Advanced tier, a security assessment and use-case approval. A person with their own app on Starter does not get it. Unreconciled bank statement lines are not exposed. Bulk export means reports to CSV or Excel, or the API. Whether you can keep read-only access after you cancel is [unclear and not a standard plan](https://www.accountingweb.co.uk/any-answers/keeping-records-on-xero-business-ceasing).

## Agents

Xero is moving faster on agents than most incumbents. It relaunched JAX as a [financial superagent at Xerocon Brisbane on 3 September 2025](https://www.xero.com/media-releases/xerocon-brisbane-xero-announces-new-features-to-drive-greater-growth-impact/). It announced a [multi-year partnership with Anthropic on 27 March 2026](https://www.xero.com/media-releases/xero-and-anthropic-collaborate/) that puts Claude inside Xero and Xero data inside Claude.ai, and says that data is never used to train Claude. [XeroForce](https://www.xero.com/media-releases/xero-introduces-xeroforce/), an agent builder, went into invite-only alpha in May 2026.

The [official MCP server](https://github.com/XeroAPI/xero-mcp-server) is MIT-licensed and runs locally. It reads accounts, contacts, invoices, bank transactions and reports, and it can create and update invoices, payments, bank transactions and manual journals. There is no approval step: a write runs when the agent calls the tool.

## Customers

Users on [Trustpilot (UK, 4.1 from 11,430 reviews)](https://uk.trustpilot.com/review/xero.com) praise ease of use and onboarding staff. They complain about slow support, payroll and bank-feed setup, and billing. The loudest complaint is about price: rises in [Australia in 2024](https://www.accountingtimes.com.au/technology/the-final-straw-xero-users-balk-at-another-price-hike) and in the UK on 1 September 2025 (Grow went from £33 to £37) were criticised as out of step with how much the product changed.

## History

Rod Drury and Hamish Edwards founded Xero in 2006. It listed on the NZX in 2007 and the ASX in 2012, and bought Hubdoc in 2018, Syft in 2024 and Melio in 2025. Sukhinder Singh Cassidy has been CEO since February 2023. From 2025 to 2026 Xero turned toward agents and, at the same time, started charging for bulk reads.

## For ownpurse

Xero is ownpurse's first source, and its policies shape the connector. Every read costs calls and now egress, so a local copy that reads incrementally and answers queries without calling Xero saves real money. Journals are out of reach on Starter, so ownpurse rebuilds the ledger from documents and checks it against Xero's trial balance. Xero's MCP lets agents write straight into the books. ownpurse's sealed, approved packages are the gate that is missing. The AI-training ban, and the consent Xero requires before data goes to a third party, mean ownpurse needs a stated policy before an agent sends Xero data to a hosted model.
