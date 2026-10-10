---
{
 "slug": "mercury",
 "name": "Mercury",
 "url": "https://mercury.com",
 "segment": "spend-management",
 "status": "active",
 "founded": "2017",
 "hq": "San Francisco, California, US",
 "backing": "Venture-backed. Series C of $300 million led by Sequoia at a $3.5 billion valuation (March 2025); Series D of $200 million at a $5.2 billion valuation (May 2026, per Wikipedia). Conditional OCC approval for a national bank charter (April 2026, per Wikipedia). Banking today through Choice Financial Group and Column N.A., Members FDIC.",
 "tagline": "A fintech banking platform for startups and small businesses, with cards, bill pay, invoicing, treasury and, since September 2026, its own double-entry books.",
 "scope": "Mercury offers business checking and savings, credit cards, spend management, bill pay, invoicing, treasury and lending, plus personal banking. It pushes coded transactions one way into QuickBooks Online, Xero and NetSuite, and since September 2026 it offers Mercury Books, a double-entry ledger inside the bank. It is not itself a bank (until a charter is final), and it does not read other banks or ledgers.",
 "domains": [
  "bank",
  "payments",
  "spend",
  "invoicing",
  "ledger",
  "integration"
 ],
 "customers_served": [
  "smb",
  "freelancer",
  "consumer",
  "developers"
 ],
 "spectrum": {
  "custody": {
   "score": 1,
   "note": "Mercury holds the bank record; you can pull it by API and statements, and it pushes copies to your ledger."
  },
  "locality": {
   "score": 0,
   "note": "Cloud only."
  },
  "history": {
   "score": 1,
   "note": "Bank transactions and statements are kept, but notes and categories are edited in place, with no version history exposed."
  },
  "openness": {
   "score": 2,
   "note": "Self-serve API tokens for your own account, webhooks, OAuth for approved partners, CSV and statement export; closed source."
  },
  "breadth": {
   "score": 3,
   "note": "Banking, cards, bill pay, invoicing, treasury and now a ledger in one place, though only for money held at Mercury."
  },
  "autonomy": {
   "score": 2,
   "note": "AI GL coding, scheduled pushes to the ledger and auto-reconciliation in Books; money moves only after human approval."
  },
  "agent_readiness": {
   "score": 4,
   "note": "Hosted MCP with OAuth, scoped write permissions, request-only payments that need approval in Mercury, setup steps for Claude Code and Codex, and llms.txt."
  },
  "integration_ease": {
   "score": 3,
   "note": "An owner can make a read-only token in minutes; write tokens need an IP allowlist; partner OAuth needs prior approval."
  },
  "market": {
   "score": 2,
   "note": "Startups and small businesses (300K+), with personal banking alongside."
  }
 },
 "api": {
  "public": true,
  "style": "REST JSON at https://api.mercury.com/api/v1: accounts, transactions, statements, recipients, payments, transfers, invoices, customers, cards, treasury, webhooks and events; a sandbox for testing.",
  "auth": "Personal API tokens (basic or bearer): Read Only (no IP allowlist), Read and Write (IP allowlist required) or Custom scopes. Tokens unused for 45 days are deleted, and unused permissions are downgraded after 45 days. OAuth2 (authorization code, PKCE) for third parties after prior approval; the MCP server supports dynamic client registration with PKCE.",
  "limits": "Not stated in the documentation read.",
  "cost": "No API fee is stated; the pricing page notes that mass payments on the API can incur fees.",
  "export": "Transactions to CSV, monthly statements, API pulls of full transaction history, and bank feeds to QuickBooks, Xero and NetSuite. Mercury Books export was not described.",
  "notes": "The token policy is a good model of least privilege: write needs an allowlisted IP, and power that goes unused decays automatically."
 },
 "pricing": {
  "model": "Free core banking with paid tiers; revenue from interchange, deposits, treasury and lending; Books as an add-on.",
  "entry": "Mercury, $0 per month: free domestic and USD international wires, free bank feeds and enriched sync for QuickBooks Online and Xero.",
  "top": "Mercury Pro $299 per month (relationship manager, NetSuite categorisation); custom pricing via sales. Mercury Plus is $29.90 per month.",
  "notes": "Pricing page read 2026-10-10. Non-USD wires carry a 1% FX fee. Mercury Books is $35 per month, free to all customers until 31 December 2026. NetSuite enriched automations need a paid plan."
 },
 "agents": {
  "mcp": "official",
  "ai_features": "AI GL coding from similar transactions, Mercury Command (plain-language questions and bulk actions such as GL coding and rule creation), Insights, receipt matching, and auto-categorisation and reconciliation in Mercury Books.",
  "agent_writes": "Through MCP an agent can create payment requests (requestSendMoney) and internal transfer requests, and can create or edit categories and update transaction notes and categories (these apply immediately). There are 31 read tools.",
  "approval": "Payment and transfer requests from an agent move money only after a person approves them in Mercury under the organisation's approval rules. The agent cannot approve, and a confirmation in chat does not count. Write scopes must be granted explicitly; existing read-only connections do not gain them.",
  "notes": "MCP is in beta at https://mcp.mercury.com/mcp, with official integrations for ChatGPT, Claude, Cursor, Perplexity, Gemini Enterprise, Grok, Microsoft 365 Copilot, Notion and Replit. The Mercury Books announcement says a full accounting API and an MCP layer for agents are coming."
 },
 "integration": {
  "scope": "Direct, one-way integrations with QuickBooks Online, Xero and NetSuite (bank feed plus enriched pushes of GL-coded transactions, bills, reimbursements, wires, ACH and checks; treasury excluded); CSV for others.",
  "notes": "Only admins and accountant roles can set up accounting automations. Xero App Store: 4.8 out of 5 from 86 reviews (read 2026-10-10). Mercury Books positions banking and books together as a way to remove the integration."
 },
 "preference": {
  "scale": "300K+ businesses (homepage and Xero App Store listing, read 2026-10-10); 1,000+ employees (About page); about $500 million revenue and $156 billion payment volume in 2024 (Wikipedia).",
  "ratings": "Trustpilot 3.9 out of 5 from 2,854 reviews (67% five-star, 21% one-star, read 2026-10-10); Xero App Store 4.8 out of 5 from 86 reviews. G2 and app store ratings not read.",
  "sentiment": "Praised for a clean product, easy onboarding, free wires and good accounting integrations. The serious complaints are accounts closed or frozen without a clear reason (funds held 60+ days, returned by check), no phone support, scripted replies, denied disputes and forced partner-bank moves. Mercury replies to 98% of negative Trustpilot reviews."
 },
 "timeline": [
  {
   "date": "2017",
   "event": "Founded by Immad Akhund, Max Tagher and Jason Zhang."
  },
  {
   "date": "2022",
   "event": "Corporate charge card launched (branded Mercury IO in 2024)."
  },
  {
   "date": "2023-03",
   "event": "Gains more than $2 billion in deposits after Silicon Valley Bank's failure."
  },
  {
   "date": "2024-04",
   "event": "Personal banking launched."
  },
  {
   "date": "2024-05",
   "event": "Bill pay and expense software launched."
  },
  {
   "date": "2024-06",
   "event": "Partner bank Evolve suffers a data breach exposing some Mercury account data."
  },
  {
   "date": "2025-03",
   "event": "$300 million Series C led by Sequoia at a $3.5 billion valuation; says it will leave Evolve for Column and Choice by end of 2025."
  },
  {
   "date": "2026-04",
   "event": "Conditional OCC approval for a national bank charter (per Wikipedia)."
  },
  {
   "date": "2026-05",
   "event": "$200 million Series D at a $5.2 billion valuation (per Wikipedia)."
  },
  {
   "date": "2026-09-16",
   "event": "Announces Mercury Books, a double-entry ledger in the bank, free until end of 2026 then $35 per month."
  }
 ],
 "relation": "source",
 "relation_note": "Mercury is a bank-side source with a clean, self-serve API: an owner can make a read-only token and pull every transaction, statement and invoice. It is also becoming a neighbour, because Mercury Books makes the bank the ledger, the opposite of ownpurse's owner-held, multi-source record.",
 "borrow": "Payment requests from an agent that move money only after a person approves in the product; scopes granted one by one, with read-only connections never silently upgraded; write tokens tied to an IP allowlist; unused permissions that decay after 45 days; MCP setup lines for Claude Code and Codex; a review step before data reaches the ledger.",
 "avoid": "Folding the books into the bank, so the ledger lives with the institution that holds the money and covers only accounts held there; edits to categories and notes that apply immediately with no history; account closures the customer cannot see the reason for.",
 "sources": [
  {
   "title": "Mercury homepage",
   "url": "https://mercury.com/"
  },
  {
   "title": "Mercury pricing",
   "url": "https://mercury.com/pricing"
  },
  {
   "title": "Mercury about",
   "url": "https://mercury.com/about"
  },
  {
   "title": "Introducing Mercury Books",
   "url": "https://mercury.com/blog/introducing-mercury-books"
  },
  {
   "title": "Mercury accounting automations",
   "url": "https://mercury.com/accounting-automations"
  },
  {
   "title": "Mercury API getting started",
   "url": "https://docs.mercury.com/docs/getting-started.md"
  },
  {
   "title": "Mercury API token security policies",
   "url": "https://docs.mercury.com/docs/api-token-security-policies.md"
  },
  {
   "title": "Mercury integrations with OAuth2",
   "url": "https://docs.mercury.com/docs/integrations-with-oauth2.md"
  },
  {
   "title": "What is Mercury MCP?",
   "url": "https://docs.mercury.com/docs/what-is-mercury-mcp.md"
  },
  {
   "title": "Connecting Mercury MCP",
   "url": "https://docs.mercury.com/docs/connecting-mercury-mcp.md"
  },
  {
   "title": "Supported tools on Mercury's MCP",
   "url": "https://docs.mercury.com/docs/supported-tools-on-mercury-mcp.md"
  },
  {
   "title": "Mercury docs llms.txt",
   "url": "https://docs.mercury.com/llms.txt"
  },
  {
   "title": "Wikipedia: Mercury Technologies",
   "url": "https://en.wikipedia.org/wiki/Mercury_Technologies"
  },
  {
   "title": "Trustpilot: mercury.com",
   "url": "https://www.trustpilot.com/review/mercury.com"
  },
  {
   "title": "Xero App Store: Mercury",
   "url": "https://apps.xero.com/us/app/mercury"
  }
 ],
 "unverified": [
  "Series D ($200 million at $5.2 billion, May 2026) and conditional OCC charter approval (April 2026) come from Wikipedia; Mercury's own announcements and CNBC could not be read (CNBC blocked, web search budget exhausted).",
  "2024 revenue of about $500 million and $156 billion payment volume (Wikipedia).",
  "API rate limits (not stated in the docs read).",
  "Launch date of the Mercury MCP beta.",
  "Whether Mercury Books offers ledger export, and when its accounting API and MCP layer will ship.",
  "G2, Capterra, Reddit and mobile app store ratings were not read."
 ],
 "researched": "2026-10-10"
}
---

## What it is

Mercury is a banking platform for startups and small businesses. It says [300K+ businesses](https://mercury.com/) use it. It is a fintech, not a bank: deposits sit with Choice Financial Group and Column N.A. It offers checking, savings, cards, spend management, bill pay, invoicing, treasury, lending and personal banking. In [September 2026 it launched Mercury Books](https://mercury.com/blog/introducing-mercury-books), described as "a full double-entry accounting engine" inside the bank.

## How it works

For most customers Mercury is the bank account that feeds the ledger. Its [accounting automations](https://mercury.com/accounting-automations) apply GL codes from similar past transactions, match receipts by email, and push coded card spend, bills, reimbursements, wires, ACH and checks to QuickBooks Online, Xero or NetSuite on a schedule. Data flows one way, from Mercury into the books, with a review step first. Mercury Books goes further: the same transactions post to a ledger in Mercury with accrual or cash basis, a P&L, a balance sheet and unlimited bookkeeper seats. It is free until 31 December 2026, then $35 per month.

## Data and access

The API is one of the most owner-friendly in banking. An admin can make a [token](https://docs.mercury.com/docs/getting-started.md) in minutes. [Read Only tokens need no IP allowlist; Read and Write tokens require one](https://docs.mercury.com/docs/api-token-security-policies.md); Custom tokens take chosen scopes. Unused tokens are deleted after 45 days, and permissions a token does not use are downgraded. It covers accounts, transactions, statements, recipients, payments, invoices, cards, treasury and webhooks, and there is a sandbox. Third-party apps use [OAuth2 after prior approval](https://docs.mercury.com/docs/integrations-with-oauth2.md). The docs read did not state rate limits. Core banking is free ([pricing](https://mercury.com/pricing)); Plus is $29.90 and Pro $299 per month.

## Agents

Mercury has the most careful agent design of the three players in this group. Its [hosted MCP server](https://docs.mercury.com/docs/what-is-mercury-mcp.md) (beta) offers [31 read tools and 5 write tools](https://docs.mercury.com/docs/supported-tools-on-mercury-mcp.md). An agent can create a payment request or a transfer request, but money moves only after a person approves it in Mercury. The agent cannot approve, and saying yes in chat does not count. Category and note edits apply immediately. Write scopes are [granted one by one at OAuth](https://docs.mercury.com/docs/connecting-mercury-mcp.md), read-only connections never gain them silently, and the docs give setup lines for Claude Code and Codex. Mercury Books promises an accounting API and an MCP layer next.

## Customers

Mercury serves startups and small businesses, with personal banking alongside. [Trustpilot](https://www.trustpilot.com/review/mercury.com) shows 3.9 out of 5 from 2,854 reviews. People praise the product and the free wires. The one-star reviews (21%) describe accounts closed or frozen without a reason, funds held for 60 days or more, no phone line and forced moves between partner banks. On the [Xero App Store](https://apps.xero.com/us/app/mercury) it has 4.8 from 86 reviews.

## History

Immad Akhund, Max Tagher and Jason Zhang founded Mercury in 2017. It gained more than $2 billion in deposits when Silicon Valley Bank failed in March 2023. It survived the partner-bank turmoil of 2024 (the Evolve breach) by moving to Column and Choice. It raised a $300 million Series C at $3.5 billion in March 2025. Per [Wikipedia](https://en.wikipedia.org/wiki/Mercury_Technologies), it won conditional OCC approval for a national charter in April 2026 and raised a $200 million Series D at $5.2 billion in May 2026. Books in September 2026 marks the turn from feeding the ledger to being it.

## For ownpurse

Mercury is an easy, high-quality source. A read-only token, with no allowlist, gives an owner every transaction, statement and invoice, and webhooks give live updates, with no partner programme in the way. Its MCP is the clearest public example of "agents propose; people post" in banking: the agent drafts the payment, and a person releases it in the product. ownpurse's sealed packages should meet that bar or exceed it. The tension is Books. Mercury is betting that the bank should hold the ledger. ownpurse is betting that the owner should hold one record across every bank and ledger. A Mercury customer with a second bank, a Stripe account or a household is the person ownpurse is for.
