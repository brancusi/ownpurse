---
{
 "slug": "bill",
 "name": "BILL",
 "url": "https://www.bill.com",
 "segment": "spend-management",
 "status": "active",
 "founded": "2006 (as Cashboard, Inc.)",
 "hq": "San Jose, California, US",
 "backing": "Public on the NYSE (ticker BILL) since its December 2019 IPO. Activist investors since 2025: Starboard Value (about 8.5%, nominated four directors in September 2025), Elliott (about 5%, per Reuters), Barington (December 2025). Bloomberg reported a strategic review in November 2025 and Hellman & Friedman buyout talks in February 2026. No sale had been announced as of the FY2026 results (19 August 2026); a $1.0 billion buyback was authorised on 7 May 2026.",
 "tagline": "Accounts payable, receivables, cards and expense software for US small and mid-sized businesses, with its own payments network, which syncs into the customer's ledger.",
 "scope": "BILL takes in vendor bills, routes them for approval, pays them by ACH, check, card or wire, sends and collects invoices, and runs corporate cards and expenses (the former Divvy) and, since April 2026, travel. It syncs bills, payments and card spend two ways with QuickBooks, Xero, NetSuite, Sage Intacct and Dynamics. It is not a general ledger: the books stay in the accounting system, and BILL holds its own record of documents, approvals and money movement.",
 "domains": [
  "payments",
  "invoicing",
  "spend",
  "documents",
  "integration"
 ],
 "customers_served": [
  "smb",
  "mid-market",
  "accountants",
  "developers"
 ],
 "spectrum": {
  "custody": {
   "score": 1,
   "note": "BILL holds the approvals, documents and payment record; only the synced summary reaches your ledger, though the API lets you pull your own data."
  },
  "locality": {
   "score": 0,
   "note": "Cloud only."
  },
  "history": {
   "score": 1,
   "note": "It keeps an approval and payment audit trail, but documents are current-state records, with no as-of queries or full version history exposed."
  },
  "openness": {
   "score": 2,
   "note": "Public v3 API with a self-serve production key and report exports; closed source, and the developer terms restrict how customer data may be shown or reused."
  },
  "breadth": {
   "score": 2,
   "note": "Spans payables, receivables, cards, expenses, travel and payments, but not books or bank accounts."
  },
  "autonomy": {
   "score": 2,
   "note": "Agents code invoices, collect W-9s and complete card transactions; paying money still runs through approval policies."
  },
  "agent_readiness": {
   "score": 3,
   "note": "Hosted MCP server (read-only beta, OAuth, per-role), a docs MCP and llms.txt; no write tools for agents yet."
  },
  "integration_ease": {
   "score": 3,
   "note": "Self-serve developer key and sandbox with good docs, but session logins, 3 concurrent calls per org and partner tiers add friction."
  },
  "market": {
   "score": 2,
   "note": "479,300 businesses at 30 June 2026, mostly SMB, with enterprise plans and NetSuite or Intacct sync for the upper mid-market."
  }
 },
 "api": {
  "public": true,
  "style": "REST-style v3 JSON API covering AP, AR, BILL network and webhooks, with a separate Spend & Expense API; Elements embeddable UI for partners.",
  "auth": "Developer key plus session: POST /v3/login returns a sessionId that expires after 35 minutes idle; MFA step-up for sensitive actions; partner login and sync tokens for app partners; separate API token for Spend & Expense. Hosted MCP uses OAuth.",
  "limits": "20,000 requests per developer key per hour; 3 concurrent requests per developer key per organisation; login 200 per key per hour; SMS endpoints 5 per minute; Spend & Expense 60 calls per token per minute (developer docs, read 2026-10-10).",
  "cost": "The production key is self-serve from Settings; the guide charges only the normal plan fee after a 30-day trial. The developer terms (effective 3 March 2026) allow BILL to charge API licence and development fees per the developer site or an order form, with 90 days' notice of changes. Payments carry per-transaction fees.",
  "export": "Reports and lists export to CSV from the app; API reads of bills, payments, vendors, invoices and card transactions; files go to non-synced ledgers by CSV.",
  "notes": "Developer terms section 5: customer data may be used only during the term and only for approved integrated applications, may not be displayed 'on any other web site or with any other applications', and may go to third-party AI models only under zero-retention, no-training terms. A local copy kept by an owner-side tool sits in a grey area under these terms."
 },
 "pricing": {
  "model": "Per user per month for AP and AR, plus per-transaction payment fees; Spend & Expense has no software fee and earns on card interchange.",
  "entry": "Essentials $49 per user per month (CSV sync only); Team $65 per user per month adds 2-way QuickBooks Online and Xero sync.",
  "top": "Corporate $89 per user per month; Enterprise custom (NetSuite, Intacct, Dynamics sync, SSO, dual control).",
  "notes": "Pricing page read 2026-10-10. ACH $0.59, check $1.99, international USD wire $19.99, instant pay 1% ($9.99 min, $100 max), virtual card free, receiving by card 2.9%. Accountant plans: AP & AR Partner $49 per month, Accountant Console $49 per month."
 },
 "agents": {
  "mcp": "official",
  "ai_features": "Invoice Coding Agent (codes multi-line bills from past behaviour), W-9 Agent (emails vendors and collects W-9s on its own), Transaction Agent (finds or generates receipts and codes card transactions), Smart Response Agent (announced, not yet available), BILL Assistant, duplicate and missing-bill detection, fraud and vendor checks.",
  "agent_writes": "BILL's own agents write coding and vendor data inside BILL. The external MCP server is read-only: agents can search bills, payments, invoices and card spend but cannot create, edit, approve or pay. Through the REST API a program can create bills and payments.",
  "approval": "Payments go through BILL's approval policies and roles (custom policies on Corporate, dual control on Enterprise). W-9 records need user review before they are created. The marketing says BILL AI keeps humans in control, but does not set out a gate for each agent.",
  "notes": "MCP is a beta at https://gateway.prod.bill.com/ai-mcp-server/v1, US organisations only, one organisation per connection, validated with Claude and ChatGPT; other clients need BILL's approval. The May 2026 reorganisation put the founding engineer, now CTO, in charge of the AI platform."
 },
 "integration": {
  "scope": "Two-way automatic sync with QuickBooks Online, QuickBooks Pro/Premier and Enterprise, Xero, NetSuite, Sage Intacct and Microsoft Dynamics (AP and AR); Spend & Expense syncs with QuickBooks Online and Desktop, NetSuite, Xero and Intacct; CSV for Blackbaud, Sage 50/100, FreshBooks, Acumatica and others. Sync runs every 24 hours or on demand.",
  "notes": "For ownpurse BILL is a source with its own record: bills, approvals, payment states and card spend that exist before, or apart from, what lands in Xero. Xero App Store listing shows 4.2 to 4.3 stars from 75 reviews (read 2026-10-10)."
 },
 "preference": {
  "scale": "479,300 businesses using BILL's solutions at 30 June 2026; 9.2 million network members; Q4 FY2026 total payment volume $98 billion; FY2026 revenue $1,653.2 million, up 13% (BILL press release, 19 August 2026).",
  "ratings": "Trustpilot 1.6 out of 5 from 1,669 reviews (read 2026-10-10); Xero App Store about 4.2 out of 5 from 75 reviews. G2 and Capterra pages could not be fetched.",
  "sentiment": "Praised for being easy to use when it works and for covering daily AP needs at an acceptable price. Complaints centre on support (long waits, transfers, scripted replies), payments held and accounts frozen or closed without explanation, repeated identity verification, login lockouts and fees. Trustpilot reviews skew toward people with a problem, and BILL answers about 7% of negative ones."
 },
 "timeline": [
  {
   "date": "2006-04",
   "event": "Founded by René Lacerte as Cashboard, Inc."
  },
  {
   "date": "2019-12",
   "event": "IPO on the NYSE as BILL."
  },
  {
   "date": "2021-06",
   "event": "Acquires Divvy (spend and expense) for about $2.5 billion."
  },
  {
   "date": "2021-09",
   "event": "Closes Invoice2go (mobile invoicing) for about $625 million."
  },
  {
   "date": "2025-09",
   "event": "Starboard Value nominates four directors; Elliott reported holding about 5%."
  },
  {
   "date": "2025-11",
   "event": "Bloomberg reports BILL is exploring strategic options, including a sale."
  },
  {
   "date": "2026-02",
   "event": "Hellman & Friedman reported in buyout talks."
  },
  {
   "date": "2026-03-03",
   "event": "New developer terms take effect, including rules for sending customer data to AI models."
  },
  {
   "date": "2026-04-28",
   "event": "Launches a travel offering for SMBs."
  },
  {
   "date": "2026-05-07",
   "event": "Q3 FY2026 results and a $1.0 billion share repurchase authorisation."
  },
  {
   "date": "2026-05-26",
   "event": "Reorganisation: founding engineer Eric Chan becomes CTO to lead AI platform strategy; a single product organisation is formed."
  },
  {
   "date": "2026-08-19",
   "event": "FY2026 results: $1,653.2 million revenue, 479,300 businesses."
  }
 ],
 "relation": "source",
 "relation_note": "BILL is a payments source with its own record that sits between documents and the ledger. ownpurse would read it as a connector (bills, approvals, payments, card spend) and check what BILL says against what reached Xero or QuickBooks. It does not compete for the ledger.",
 "borrow": "Approval policies with roles and dual control as data, not code; the read-only, per-role OAuth MCP that only sees what the signed-in person can see; the docs MCP and llms.txt for agent developers; the line that customer data may go to AI models only under zero-retention, no-training terms.",
 "avoid": "Developer terms that forbid showing customer data 'with any other applications', which works against owner-held copies; session logins with a 35-minute idle timeout and a 3-call concurrency cap that make bulk sync slow; account holds and verification loops with no explanation to the owner.",
 "sources": [
  {
   "title": "BILL pricing",
   "url": "https://www.bill.com/pricing"
  },
  {
   "title": "BILL Reports Fourth Quarter and Fiscal Year 2026 Financial Results",
   "url": "https://www.bill.com/press-release/bill-reports-fourth-quarter-and-fiscal-year-2026-financial-results"
  },
  {
   "title": "BILL Announces Executive Leadership Appointments and Organizational Updates",
   "url": "https://www.bill.com/press-release/bill-announces-executive-leadership-appointments-and-organizational-updates"
  },
  {
   "title": "BILL newsroom",
   "url": "https://www.bill.com/newsroom"
  },
  {
   "title": "BILL AI",
   "url": "https://www.bill.com/product/ai"
  },
  {
   "title": "BILL integrations",
   "url": "https://www.bill.com/integrations"
  },
  {
   "title": "BILL MCP Server (developer docs)",
   "url": "https://developer.bill.com/docs/bill-mcp-server.md"
  },
  {
   "title": "BILL API rate limits",
   "url": "https://developer.bill.com/docs/api-rate-limits.md"
  },
  {
   "title": "BILL keys and tokens",
   "url": "https://developer.bill.com/docs/bill-keys-tokens.md"
  },
  {
   "title": "Get started in production",
   "url": "https://developer.bill.com/docs/get-started-in-production.md"
  },
  {
   "title": "BILL Developer Terms",
   "url": "https://developer.bill.com/docs/bill-developer-terms.md"
  },
  {
   "title": "Build with LLMs (BILL)",
   "url": "https://developer.bill.com/docs/ai-agent-resources.md"
  },
  {
   "title": "BILL developer llms.txt",
   "url": "https://developer.bill.com/llms.txt"
  },
  {
   "title": "Wikipedia: Bill.com",
   "url": "https://en.wikipedia.org/wiki/Bill.com"
  },
  {
   "title": "Trustpilot: bill.com",
   "url": "https://www.trustpilot.com/review/bill.com"
  },
  {
   "title": "Xero App Store: BILL",
   "url": "https://apps.xero.com/us/app/bill"
  }
 ],
 "unverified": [
  "Activist stakes, the November 2025 strategic review and February 2026 Hellman & Friedman talks come from Wikipedia's summary of Reuters and Bloomberg reports; primary filings were not checked (web search budget was exhausted in this session).",
  "Launch date of the BILL MCP server beta is not stated in the docs.",
  "Whether BILL actually charges any API licence fee today; the terms allow it, but no fee schedule was found.",
  "FY2027 guidance figures (published only as an image in the release).",
  "G2 and Capterra ratings (pages blocked).",
  "Whether keeping a local owner-side copy of BILL data via the API complies with developer terms section 5; the terms do not address owner-internal use directly.",
  "Finmark acquisition (2022) was not confirmed from a source read in this session."
 ],
 "researched": "2026-10-10"
}
---

## What it is

BILL is the accounts payable and receivable tool many US small businesses use beside QuickBooks or Xero. It takes in vendor bills, routes them for approval, pays them, sends invoices and collects payment. Since buying Divvy in 2021 it also runs corporate cards and expenses, and in April 2026 it added travel. It is a public company (NYSE: BILL) with [FY2026 revenue of $1,653.2 million and 479,300 businesses](https://www.bill.com/press-release/bill-reports-fourth-quarter-and-fiscal-year-2026-financial-results) at 30 June 2026.

## How it works

Bills arrive by email, upload or the BILL network of 9.2 million members. BILL reads them, suggests coding, routes them through approval policies and pays by ACH, check, card or wire. Spend & Expense issues cards with budgets and matches receipts. The [integrations page](https://www.bill.com/integrations) lists two-way sync with QuickBooks, Xero, NetSuite, Sage Intacct and Dynamics, run every 24 hours or on demand. The books stay in the ledger; BILL keeps its own record of every document, approval and payment.

## Data and access

The v3 API covers AP, AR, the network and webhooks, and a separate API covers Spend & Expense. A customer can [generate a production developer key from Settings](https://developer.bill.com/docs/get-started-in-production.md) with no approval step. You sign in with the key to get a session that expires after 35 minutes idle. [Rate limits](https://developer.bill.com/docs/api-rate-limits.md) are 20,000 calls per key per hour and only 3 concurrent calls per organisation, so a full pull of a busy account is slow.

The [developer terms](https://developer.bill.com/docs/bill-developer-terms.md) (effective 3 March 2026) matter more than the limits. Customer data may be used only during the term and for approved integrated applications, may not be displayed "with any other applications", and may reach third-party AI models only under zero-retention, no-training terms. BILL may also charge API licence fees with 90 days' notice. For a tool that keeps an owner-held copy, this is a grey area to read carefully.

[Pricing](https://www.bill.com/pricing) runs from $49 to $89 per user per month for AP and AR, plus fees for each payment: ACH $0.59, check $1.99, international USD wire $19.99. Spend & Expense has no software fee.

## Agents

BILL has gone further than most of its peers here. Its [AI page](https://www.bill.com/product/ai) lists an Invoice Coding Agent, a W-9 Agent that emails vendors on its own, and a Transaction Agent that finds receipts and codes card spend. A Smart Response Agent is announced but not yet available. For outside agents there is a [hosted MCP server](https://developer.bill.com/docs/bill-mcp-server.md), in beta. It is read-only, works for US organisations only, connects to one organisation at a time and is validated with Claude and ChatGPT. Every call runs under the signed-in person's role. There is also a [docs MCP and llms.txt](https://developer.bill.com/docs/ai-agent-resources.md) for developers. Writes and payments by an outside agent are not offered through MCP; approval policies and dual control gate payments inside BILL.

## Customers

Customers are mostly US SMBs and their accounting firms; enterprise plans reach the mid-market. The scale is real, and so is the friction: [Trustpilot](https://www.trustpilot.com/review/bill.com) shows 1.6 out of 5 from 1,669 reviews, mostly about support, payments held and frozen accounts. The Xero App Store listing sits at about 4.2 from 75 reviews. Review sites over-represent problems, but the complaints about holds with no explanation are consistent.

## History

René Lacerte founded the company in 2006 as Cashboard. It listed in December 2019 and spent about $3.1 billion in 2021 on Divvy and Invoice2go to cover spend and receivables. From 2025, activists (Starboard, Elliott, Barington) pushed for change. Reports of a strategic review (November 2025) and Hellman & Friedman talks (February 2026) did not lead to an announced sale. Instead, BILL authorised a $1.0 billion buyback in May 2026 and reorganised around a single product group, with the founding engineer leading AI.

## For ownpurse

BILL is a source with its own truth. A bill is approved in BILL before it posts, and a payment has states (scheduled, sent, cleared, voided) that the ledger never sees. A BILL connector would append those facts and check them against what reached Xero. The patterns to borrow are policies held as data, a read-only MCP scoped to each person's role, and the zero-retention rule for AI. The terms are the main thing to avoid: ownpurse's promise is that the owner keeps a copy, and BILL's terms are written for distributed apps, not owners.
