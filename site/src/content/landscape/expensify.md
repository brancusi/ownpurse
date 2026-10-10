---
{
 "slug": "expensify",
 "name": "Expensify",
 "url": "https://www.expensify.com",
 "segment": "spend-management",
 "status": "active",
 "founded": "2008",
 "hq": "Portland, Oregon, US",
 "backing": "Public on Nasdaq (EXFY) since 11 November 2021. Received a Nasdaq minimum-bid-price deficiency notice on 17 April 2026, with until 14 October 2026 to regain compliance; stockholders voted in May 2026 on a possible reverse split. Venture rounds before the IPO totalled about $27 million (2009-2015).",
 "tagline": "Receipt scanning, expense reports, a corporate card and travel for individuals and small businesses, exported to the accounting system.",
 "scope": "Expensify captures receipts, builds and approves expense reports, reimburses employees, issues the Expensify Card, books travel and adds bill pay and invoicing, then exports to QuickBooks, Xero, NetSuite, Sage Intacct and others (45+ integrations). It is a source of documents and card lines, not a ledger. It runs two products: Expensify Classic and the chat-based New Expensify, whose client is open source.",
 "domains": [
  "spend",
  "documents",
  "payments",
  "invoicing",
  "integration"
 ],
 "customers_served": [
  "consumer",
  "freelancer",
  "smb",
  "mid-market",
  "accountants"
 ],
 "spectrum": {
  "custody": {
   "score": 1,
   "note": "Expensify holds the receipts and reports; exports and the Report Exporter give you copies in CSV, PDF, JSON or XML, but the original record stays with Expensify."
  },
  "locality": {
   "score": 0,
   "note": "Cloud only; the MCP is hosted at expensify.com/mcp."
  },
  "history": {
   "score": 1,
   "note": "Reports carry an approval and action trail, but nothing found offers versions or as-of queries."
  },
  "openness": {
   "score": 3,
   "note": "The New Expensify client (Expensify/App) is MIT-licensed on GitHub; the backend is closed; a public API and broad export formats."
  },
  "breadth": {
   "score": 2,
   "note": "Centred on expenses, receipts and cards, with travel, bill pay and invoicing added; no banking or ledger."
  },
  "autonomy": {
   "score": 2,
   "note": "Concierge auto-categorizes, applies rules and flags outliers, and approved reports can export to the ledger automatically; workflow agents are only in beta."
  },
  "agent_readiness": {
   "score": 3,
   "note": "Official hosted MCP with OAuth 2.1 and PKCE for Claude, ChatGPT and Cursor, but read-only through a single Search tool; no CLI."
  },
  "integration_ease": {
   "score": 2,
   "note": "An older single-endpoint job API with Freemarker export templates and tight limits (5 requests per 10 seconds); OAuth only in private beta."
  },
  "market": {
   "score": 2,
   "note": "Free for individuals, priced for small businesses (640,000 paid members, June 2026), with some mid-market use."
  }
 },
 "api": {
  "public": true,
  "style": "Single POST endpoint taking a JSON job description (Integration Server); hosted MCP server",
  "auth": "partnerUserID and partnerUserSecret per account for server-to-server use; OAuth2 authorization-code flow in private beta (2-hour tokens, rotating refresh tokens); OAuth 2.1 with PKCE and the mcp:tools scope for the MCP",
  "limits": "5 requests per 10 seconds and 20 per 60 seconds; HTTP 429 beyond that",
  "cost": "No API fee found in the documentation; access comes with an Expensify account",
  "export": "Report Exporter by report id or date range (up to one year) in CSV, XLS/XLSX, TXT, PDF, JSON or XML, shaped by Freemarker templates; card reconciliation exports by card, report and transaction; SFTP and email delivery",
  "notes": "Writes include creating reports, expenses, policies and expense rules, updating categories, tags and report fields, and moving a report from Approved to Reimbursed. In reconciliation exports, card transactions not yet on a report appear under report id 0."
 },
 "pricing": {
  "model": "Per-member SaaS, discounted by Expensify Card use; card interchange on top",
  "entry": "Collect: $5 per unique member per month, pay-per-use; Submit is free for an individual",
  "top": "Control: $18 per included member per month on an annual subscription (down to $9 with the Expensify Card), or $36 per active member pay-per-use (down to $18)",
  "notes": "Control Annual can be up to 50% off depending on the share of USD expenses paid with the Expensify Card. Workspaces created before 1 April 2025 keep legacy Collect pricing. Regional prices in GBP, EUR, AUD and NZD. From Expensify's billing help page, checked 10 October 2026."
 },
 "agents": {
  "mcp": "official",
  "ai_features": "Concierge AI: auto-categorization, policy flags, rule enforcement, flag-only-outliers approvals, spend analysis and setup in natural language; AI workflow agents in beta (Q2 2026).",
  "agent_writes": "Not through MCP: it is read-only and cannot create or edit expenses, submit, approve or reimburse. The Integration Server API can create reports and expenses and change policy settings.",
  "approval": "The MCP cannot approve, only list what is waiting. Access mirrors the user's account and workspace permissions after an OAuth consent screen.",
  "notes": "A single Search tool covers expenses, reports, reimbursements, invoices, receipts, approvals and trips."
 },
 "integration": {
  "scope": "45+ integrations: QuickBooks Online and Desktop, Xero, NetSuite, Sage Intacct, Certinia (FinancialForce), Workday and HR tools; bring-your-own-card feeds from 10,000+ banks",
  "notes": "Per-card export settings map each card to a ledger bank account (for example a Xero bank account) and default vendor."
 },
 "preference": {
  "scale": "640,000 paid members, revenue $33.9 million (down 5% year on year), Expensify Card interchange $5.9 million (up 12%) for the quarter ended 30 June 2026; the homepage claims 15 million+ members including free users.",
  "ratings": "Trustpilot 4.7 of 5 from 1,265 reviews (October 2026); G2 4.5 from 4,889 reviews as quoted on Expensify's homepage. Capterra could not be read.",
  "sentiment": "Reviewers praise ease of use, fast receipt capture, quick reimbursement and simple approvals. Complaints are slow support, a lost bulk-upload feature after an update, report building, a mobile app that sends users to the web, and occasional glitches."
 },
 "timeline": [
  {
   "date": "2008",
   "event": "Founded by David Barrett"
  },
  {
   "date": "2015",
   "event": "$17 million round, the last before the IPO"
  },
  {
   "date": "2017-11",
   "event": "Admits fewer than 200 receipts were seen by Mechanical Turk workers during a test"
  },
  {
   "date": "2020-10",
   "event": "CEO email to customers urging a vote for Joe Biden draws criticism"
  },
  {
   "date": "2021-11-11",
   "event": "IPO on Nasdaq as EXFY"
  },
  {
   "date": "2022",
   "event": "Revenue $169 million, net loss $27 million"
  },
  {
   "date": "2025-04-01",
   "event": "New pricing for workspaces created from this date"
  },
  {
   "date": "2025-12-30",
   "event": "8-K filed with a separation agreement for COO Anu Muralidharan"
  },
  {
   "date": "2026-04-17",
   "event": "Nasdaq notice for trading below $1.00; deadline 14 October 2026"
  },
  {
   "date": "2026-08-06",
   "event": "Q2 2026: revenue $33.9 million, 640,000 paid members, free cash flow $6.4 million; Expensify MCP and workflow agents announced"
  }
 ],
 "relation": "source",
 "relation_note": "Expensify is a source of receipts, expense reports and card lines for small businesses. Its exports are already designed to feed ledgers, so it suits an ownpurse connector that keeps the documents and checks what reached the books.",
 "borrow": "A read-only MCP built on one Search tool over every object type, with access mirroring the user's own permissions. Owner-defined export templates (Freemarker) for any output shape. The convention that card lines not yet on a report sit under report id 0, a plain way to show items still uncollapsed. An open-source client.",
 "avoid": "A single job endpoint with tight limits and secrets shown only once. Splitting customers across two products (Classic and New) for years. Building an integration on an OAuth flow that is still in private beta.",
 "sources": [
  {
   "title": "Expensify homepage",
   "url": "https://www.expensify.com/"
  },
  {
   "title": "Expensify billing overview (plan prices)",
   "url": "https://help.expensify.com/articles/new-expensify/billing-and-subscriptions/Billing-Overview"
  },
  {
   "title": "Expensify Integration Server API docs",
   "url": "https://integrations.expensify.com/Integration-Server/doc/"
  },
  {
   "title": "Use the Expensify MCP server with AI assistants",
   "url": "https://help.expensify.com/articles/new-expensify/connections/connect-ai-assistants/Use-the-Expensify-MCP-Server-With-AI-Assistants"
  },
  {
   "title": "Expensify Q2 2026 results, Exhibit 99.1 (6 August 2026)",
   "url": "https://www.sec.gov/Archives/edgar/data/1476840/000147684026000073/exhibit991-8xkq226earnings.htm"
  },
  {
   "title": "Expensify 8-K: Nasdaq deficiency notice (21 April 2026)",
   "url": "https://www.sec.gov/Archives/edgar/data/1476840/000147684026000024/0001476840-26-000024.txt"
  },
  {
   "title": "Expensify 8-K: officer separation (30 December 2025)",
   "url": "https://www.sec.gov/Archives/edgar/data/1476840/000147684025000143/0001476840-25-000143-index.htm"
  },
  {
   "title": "New Expensify client on GitHub (MIT)",
   "url": "https://github.com/Expensify/App"
  },
  {
   "title": "Wikipedia: Expensify",
   "url": "https://en.wikipedia.org/wiki/Expensify"
  },
  {
   "title": "Trustpilot: Expensify",
   "url": "https://www.trustpilot.com/review/expensify.com"
  }
 ],
 "unverified": [
  "Whether Expensify regained Nasdaq compliance or carried out a reverse split before the 14 October 2026 deadline",
  "The departing officer in the 30 December 2025 8-K is inferred from the exhibit filename (Anu Muralidharan); the filing text was not read",
  "G2 rating (4.5 from 4,889) is Expensify's own figure on its homepage; Capterra could not be read",
  "The 15 million+ members figure is a homepage claim that includes free users",
  "Details of the AI workflow agents in beta (scope, write access, approval) were not found",
  "Accountants as a served segment rests on Expensify's accountant partner programme, which was not checked in this pass"
 ],
 "researched": "2026-10-10"
}
---

## What it is

Expensify has been the default expense app for small businesses for more than a decade. You photograph a receipt, the app reads it, builds the report, routes it for approval, reimburses the employee and exports the result to QuickBooks, Xero, NetSuite, Sage Intacct and about 45 other systems ([homepage](https://www.expensify.com/)). It also issues the Expensify Card, books travel and has added bill pay and invoicing. It does not keep a ledger. It feeds documents and card lines into one.

The company runs two products. Expensify Classic is, in CEO David Barrett's words, a profitable workhorse with a shrinking base. New Expensify is built around chat and is where the growth is. Its client is [open source under MIT](https://github.com/Expensify/App).

## How it works

Pricing is per member ([billing help](https://help.expensify.com/articles/new-expensify/billing-and-subscriptions/Billing-Overview)). Collect is $5 per member per month. Control is $18 per member per month on an annual plan, falling to $9 with heavy Expensify Card use, or $36 per active member pay-per-use. Individuals can use it free. Concierge, the built-in assistant, categorizes expenses, enforces policy and lets a manager review only the outliers.

## Data and access

The [Integration Server](https://integrations.expensify.com/Integration-Server/doc/) is old in style: one POST endpoint that takes a JSON job description, a partner id and secret per account, and limits of 5 requests per 10 seconds and 20 per minute. OAuth exists only in private beta. Exports are its strength. The Report Exporter returns reports by id or date range in CSV, XLSX, PDF, JSON or XML, shaped by templates you write. Card reconciliation exports go card by card, and lines not yet on a report sit under report id 0. The API can also create expenses and reports, and change categories, tags and rules.

## Agents

The [Expensify MCP](https://help.expensify.com/articles/new-expensify/connections/connect-ai-assistants/Use-the-Expensify-MCP-Server-With-AI-Assistants) is hosted at `expensify.com/mcp`, signs in with OAuth 2.1 and PKCE, and works with Claude, ChatGPT and Cursor. It is read-only by design: one Search tool covers expenses, reports, receipts, approvals and trips. It cannot submit, approve or reimburse. The Q2 2026 results put AI workflow agents in beta.

## Customers

For the quarter ended 30 June 2026, Expensify reported revenue of $33.9 million, down 5% year on year. It had 640,000 paid members, down 2%, and $5.9 million in card interchange, up 12%. It made $6.4 million in free cash flow and expects $12 to $14 million for 2026 ([Q2 release](https://www.sec.gov/Archives/edgar/data/1476840/000147684026000073/exhibit991-8xkq226earnings.htm)). Revenue from new customers grew over 250%, to more than $10 million ARR. Reviews are strong: 4.7 on Trustpilot from 1,265 reviews, and 4.5 on G2 from 4,889 by Expensify's own count. Users like the receipt capture and dislike slow support.

## History

David Barrett founded Expensify in 2008 and raised little venture money (about $27 million by 2015). The company went public in November 2021, and revenue was $169 million in 2022. The share price has since fallen below $1. Nasdaq sent a deficiency notice on 17 April 2026, with a deadline of 14 October ([8-K](https://www.sec.gov/Archives/edgar/data/1476840/000147684026000024/0001476840-26-000024.txt)). Along the way came a 2017 privacy lapse with Mechanical Turk testers and a disputed 2020 political email to customers.

## For ownpurse

Expensify is the clearest source in this segment for an individual or small business: receipts, reports and card lines, with exports already built for ledgers. Its read-only MCP takes the position ownpurse takes for agents: read freely, write nothing without a person. An Expensify connector would keep every receipt in the drawer by hash and every report as facts. It would then link those to the Xero or QuickBooks entries Expensify exported, so you can see which receipts reached the books and which did not. The tight API limits suit a patient local poller.
