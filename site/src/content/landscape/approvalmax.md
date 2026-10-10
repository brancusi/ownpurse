---
{
 "slug": "approvalmax",
 "name": "ApprovalMax",
 "url": "https://approvalmax.com",
 "segment": "spend-management",
 "status": "active",
 "founded": "2014",
 "hq": "Remote-first; legal entity ApprovalMax Limited, registered in England and Wales (company number 11326265)",
 "backing": "Private. No funding rounds or investors are listed on its own site; outside funding could not be confirmed in this session.",
 "tagline": "Multi-step approval workflows for bills, purchase orders, expenses and other documents before they post to Xero, QuickBooks Online or NetSuite.",
 "scope": "ApprovalMax pulls bills, purchase orders, credit notes, quotes, expenses and journals from the ledger (or takes them in through its OCR, Capture), routes them through rules by amount, supplier, account or tracking category, and writes the approved document back. It adds PO matching, budget checks, supplier onboarding, an audit trail and, in the UK, payments. It is not a ledger, a bank or a payments network outside the UK.",
 "domains": [
  "documents",
  "payments",
  "spend",
  "integration"
 ],
 "customers_served": [
  "smb",
  "mid-market",
  "accountants"
 ],
 "spectrum": {
  "custody": {
   "score": 1,
   "note": "The documents live in the ledger; the approval trail lives only in ApprovalMax's cloud."
  },
  "locality": {
   "score": 0,
   "note": "Cloud only."
  },
  "history": {
   "score": 2,
   "note": "Each approval decision is kept with who and when, and request and workflow version history comes on Advanced and above; no as-of queries over the books."
  },
  "openness": {
   "score": 1,
   "note": "Closed; a public API exists only on the top Premium plan, and export is by audit and scheduled reports."
  },
  "breadth": {
   "score": 1,
   "note": "One domain: approvals (with OCR capture and UK payments attached) over documents in one ledger."
  },
  "autonomy": {
   "score": 1,
   "note": "Humans approve; software adds auto-approval of repeat transactions under set rules."
  },
  "agent_readiness": {
   "score": 1,
   "note": "No MCP or agent features; a Premium-only API; it publishes thinking on agent authority levels but has shipped nothing for agents."
  },
  "integration_ease": {
   "score": 2,
   "note": "One-click marketplace apps for Xero, QuickBooks Online and NetSuite for owners; little for third-party developers."
  },
  "market": {
   "score": 2,
   "note": "SMB and mid-market, often through accounting firms; NetSuite support reaches larger firms."
  }
 },
 "api": {
  "public": true,
  "style": "A 'Public API' feature on the Premium plan; style and endpoints are not documented publicly.",
  "auth": "Not documented publicly.",
  "limits": "Not documented publicly.",
  "cost": "Included only in the Premium plan; plan prices are not published.",
  "export": "Audit reports, scheduled reports and accrual reports from the app; the approved documents themselves sit in the connected ledger and export from there.",
  "notes": "No public developer portal was found (developers.approvalmax.com does not resolve). For most customers the only way to reach the data is the ledger it posts into."
 },
 "pricing": {
  "model": "Per organisation per month, by plan type (Standard, Advanced, Premium) and, from 2026, plan size (Small, Medium, Large) set by documents processed, with automatic overage billing. Unlimited users and approvers.",
  "entry": "Standard (bill, PO and credit note approvals); price not published.",
  "top": "Premium (stand-alone workflows, Xero Batch Payments checks, public API, phone support); price not published.",
  "notes": "Prices are shown only through a calculator or sales (read 2026-10-10). Annual and monthly billing, seven currencies, 20% off for not-for-profits. Under the new model (blog, 3 June 2026), Capture OCR is included at no charge, multi-org discounts are removed, and migration of existing customers starts 3 August 2026 at renewal."
 },
 "agents": {
  "mcp": "none",
  "ai_features": "OCR capture of invoices and expenses, duplicate-invoice and bank-detail-change detection, auto-approval of repeat transactions. No named AI agents.",
  "agent_writes": "No agent interface. A Premium API exists but is not documented publicly.",
  "approval": "This is the product: multi-step, conditional approval routes with segregation of duties, audit trail, review-and-edit before final approval and skipped-approval alerts, then a write back to the ledger.",
  "notes": "Its blog post 'AI agents in finance: The rise of approval intelligence' (12 August 2026) proposes four levels of agent authority (read, recommend, act after approval, act within limits). It says the approval policy stays 'the single source of authority' and that AI and human actions should be in the same record. This is close to ownpurse's stance, but has not shipped as a product."
 },
 "integration": {
  "scope": "Xero, QuickBooks Online and Oracle NetSuite; Slack, Dext Prepare and a mobile app on every plan; Airwallex and Open Banking for UK payments.",
  "notes": "Xero access: invoices, contacts, purchase orders and credit notes both ways; accounts, tracking categories, tax rates, items, currencies and branding themes read-only (Xero App Store listing). Live budget sync and Xero purchase matching."
 },
 "preference": {
  "scale": "More than 20,000 businesses and 150+ staff (About page, read 2026-10-10); 18,000+ Xero-connected and 1,700+ QuickBooks Online organisations in the 12 months to August 2026, across nearly 60 countries (blog, 23 September 2026). In the last year it processed 8.5m+ bills and 1.9m+ purchase orders (About page).",
  "ratings": "Xero App Store 4.8 out of 5 from 659 reviews (82% five-star); QuickBooks App Store 4.6 (company-reported, September 2026). G2 and Capterra pages could not be fetched.",
  "sentiment": "Xero App Store reviews are strongly positive about approving from phone or email without a Xero login, the audit trail, catching duplicates and fair value for money. Complaints could not be sampled from G2, Capterra or Reddit in this session; the June 2026 pricing change to usage tiers and the removal of multi-org discounts may be felt by firms with many organisations."
 },
 "timeline": [
  {
   "date": "2014",
   "event": "Founded to bring AP and AR approval automation to small and mid-sized businesses."
  },
  {
   "date": "2024-10",
   "event": "NetSuite approvals launched (inferred from an October 2025 'one year on' post)."
  },
  {
   "date": "2025-08-14",
   "event": "Global payments ('now boarding') announced for ApprovalMax Pay."
  },
  {
   "date": "2026-02-09",
   "event": "Multi-currency Wallets introduced."
  },
  {
   "date": "2026-06-03",
   "event": "Moves to tiered, usage-based pricing; Capture OCR included in all plans."
  },
  {
   "date": "2026-06-26",
   "event": "Improved bill-to-PO matching."
  },
  {
   "date": "2026-08-03",
   "event": "Existing customers begin migrating to the new pricing at renewal."
  },
  {
   "date": "2026-08-12",
   "event": "Publishes its position on AI agents and approval authority levels."
  },
  {
   "date": "2026-09-23",
   "event": "Publishes data on 3 million approval decisions across 1,700+ QuickBooks Online organisations."
  }
 ],
 "relation": "design-cousin",
 "relation_note": "ApprovalMax is the closest working analogue to ownpurse's approval gate: nothing posts to the ledger until named people approve it, and every decision is recorded. It gates human-created documents in the cloud; ownpurse gates agent-proposed packages on the owner's machine. It is also a possible source, since its approval trail is data the ledger never sees.",
 "borrow": "Approval rules as conditions on amount, supplier, account and tracking category; multi-step routes with segregation of duties; review-and-edit before final approval; skipped-approval alerts; approving from a phone without a ledger login; the four-level model of agent authority, which maps onto ownpurse's read, propose, approve.",
 "avoid": "A closed approval trail that only the vendor holds, with an API reserved for the top plan; unpublished prices; billing that changes with document volume, which is hard for a small owner to predict.",
 "sources": [
  {
   "title": "ApprovalMax about",
   "url": "https://approvalmax.com/about"
  },
  {
   "title": "ApprovalMax pricing",
   "url": "https://approvalmax.com/pricing"
  },
  {
   "title": "ApprovalMax for Xero plans",
   "url": "https://approvalmax.com/pricing/approvalmax-for-xero"
  },
  {
   "title": "ApprovalMax for QuickBooks Online plans",
   "url": "https://approvalmax.com/pricing/approvalmax-for-quickbooks-online"
  },
  {
   "title": "ApprovalMax for Xero",
   "url": "https://approvalmax.com/xero"
  },
  {
   "title": "We've changed the way we charge",
   "url": "https://blog.approvalmax.com/weve-changed-the-way-we-charge"
  },
  {
   "title": "AI agents in finance: The rise of approval intelligence",
   "url": "https://blog.approvalmax.com/ai-agents-in-finance-the-rise-of-approval-intelligence"
  },
  {
   "title": "3 million approval decisions in QuickBooks Online businesses",
   "url": "https://blog.approvalmax.com/3-million-approval-decisions-what-approvalmax-data-says-about-spend-control-in-quickbooks-online-businesses"
  },
  {
   "title": "ApprovalMax blog",
   "url": "https://blog.approvalmax.com/"
  },
  {
   "title": "ApprovalMax privacy policy",
   "url": "https://approvalmax.com/privacy-policy"
  },
  {
   "title": "Xero App Store: ApprovalMax",
   "url": "https://apps.xero.com/us/app/approvalmax"
  }
 ],
 "unverified": [
  "Plan prices in any currency (not published; calculator output not readable).",
  "Public API style, authentication, endpoints and limits (no public documentation found).",
  "Funding history and investors (none stated on its site; outside sources could not be searched because the session's web search budget was exhausted).",
  "Founders' names and an operating headquarters.",
  "NetSuite launch month (inferred from an October 2025 'one year on' post).",
  "G2, Capterra and Reddit sentiment (pages blocked or not reached).",
  "QuickBooks App Store 4.6 rating is company-reported, not read from the store."
 ],
 "researched": "2026-10-10"
}
---

## What it is

ApprovalMax is an approval layer for accounts payable and other documents. It sits between the people who raise a bill or purchase order and the ledger it posts to. It was [founded in 2014](https://approvalmax.com/about), is remote-first with more than 150 staff, and trades as ApprovalMax Limited, a company registered in England and Wales. It reports more than 20,000 businesses: [18,000+ Xero organisations and 1,700+ QuickBooks Online organisations](https://blog.approvalmax.com/3-million-approval-decisions-what-approvalmax-data-says-about-spend-control-in-quickbooks-online-businesses) in the 12 months to August 2026.

## How it works

Bills, purchase orders and expenses [sync from Xero in real time](https://approvalmax.com/xero), or arrive through Capture, its OCR. A workflow routes each one through steps set by amount, supplier, account, class or tracking category. Approvers act from email or a phone without a Xero login, and the approved document goes back to the ledger. The [Xero App Store listing](https://apps.xero.com/us/app/approvalmax) shows invoices, contacts, purchase orders and credit notes shared both ways, and accounts, tax rates and tracking categories read only. Higher plans add PO matching, budget checks, journal and quote approvals, supplier onboarding, accrual reports and request version history. In the UK, ApprovalMax Pay releases payments after approval.

## Data and access

This is the weak side. The [plan comparison](https://approvalmax.com/pricing/approvalmax-for-xero) lists a public API only on Premium, the top plan, and there is no public developer documentation. Prices are not published; the page offers a calculator and sales contact. In [June 2026](https://blog.approvalmax.com/weve-changed-the-way-we-charge) it moved to plan sizes based on documents processed, with automatic overage, included Capture in every plan, and removed multi-org discounts. Existing customers migrate from 3 August 2026. The documents themselves export from the ledger. The approval trail, which is the record of who approved what and when, is in ApprovalMax's cloud and leaves mainly as reports.

## Agents

ApprovalMax has no MCP server and no named agents. What it has is a clear position. Its [August 2026 post on approval intelligence](https://blog.approvalmax.com/ai-agents-in-finance-the-rise-of-approval-intelligence) sets out four levels of agent authority: read, recommend, act after approval, act within defined limits. It says "the organization's approval policy remains the single source of authority", that "historical behavior alone should not become policy by default", and that "AI and human actions should form part of the same record." It argues its workflows let an agent work at level 1 or 2 today. That is a statement of intent, not a shipped product.

## Customers

Small and mid-sized businesses on Xero, QuickBooks Online and NetSuite, often set up by their accounting firm, with partner pricing for firms. Reviews on the Xero App Store are very strong: 4.8 out of 5 from 659 reviews, with 82% at five stars. They praise mobile approval, the audit trail and duplicate detection. G2, Capterra and Reddit could not be read in this session. The new usage-based pricing is the likeliest source of friction for multi-organisation firms.

## History

ApprovalMax began as an approval tool for Xero, then added QuickBooks Online, NetSuite (about October 2024), OCR, payments in the UK (global payments announced August 2025, Wallets February 2026) and better PO matching (June 2026). It has grown by staying narrow: one job, done inside the ledgers small businesses already use.

## For ownpurse

ApprovalMax shows that people will pay for a gate in front of the ledger and will use it from a phone. It also shows the gate can be fast enough not to get in the way. ownpurse's gate differs in three ways: the proposer is often an agent, the package is sealed and hashed, and the trail lives in the owner's record, not the vendor's. Borrow the rule model (conditions on amount, supplier, account and tracking), the multi-step routes, review-and-edit before final approval and skipped-approval alerts, and the four authority levels. Avoid a trail the owner cannot export and an API reserved for the top plan. ApprovalMax is also a source: its decisions are facts about a document that the ledger never records.
