---
{
 "slug": "pilot",
 "name": "Pilot",
 "url": "https://pilot.com",
 "segment": "bookkeeping-service",
 "status": "active",
 "founded": "2017",
 "hq": "San Francisco, California",
 "backing": "Venture-backed, private. Series C extended to $100M on 26 March 2021 (Bezos Expeditions and Whale Rock leading the extension; Sequoia and Index participating), at a $1.2B valuation. Total raised is reported between about $158M (TechCrunch) and $218M (Latka); not confirmed by Pilot.",
 "tagline": "A US bookkeeping, tax and CFO service for startups and small businesses that now runs its books with an autonomous AI accountant and sells that engine, Meridian, to accounting firms.",
 "scope": "Pilot does the books for you, on QuickBooks Online as the core ledger, plus tax filing, fractional CFO and outsourced operations work. Since February 2026 an AI Accountant runs bookkeeping from onboarding to close, and since June 2026 Meridian sells the same close engine to accounting firms on QuickBooks Online, Xero and NetSuite. It does not offer a public API to its own data, and it does not keep an owner-held record of what it changed.",
 "domains": [
  "ledger",
  "bank",
  "tax",
  "close",
  "reporting",
  "payroll",
  "invoicing",
  "integration"
 ],
 "customers_served": [
  "smb",
  "mid-market",
  "accountants"
 ],
 "spectrum": {
  "custody": {
   "score": 2,
   "note": "The books live in QuickBooks Online, which a client can log into and keep on leaving; Pilot's working data and decision log stay with Pilot."
  },
  "locality": {
   "score": 0,
   "note": "A hosted service; nothing runs on the owner's machine."
  },
  "history": {
   "score": 1,
   "note": "Pilot says every decision is logged with a full audit history, but that log is Pilot's, and the ledger itself is current-state QuickBooks."
  },
  "openness": {
   "score": 1,
   "note": "No public API to Pilot's own data; export goes through the client's QuickBooks file. The free Meridian QuickBooks connector is the one open door."
  },
  "breadth": {
   "score": 3,
   "note": "Bookkeeping, AP and AR, payroll administration, tax, CFO and COO services, R&D credits, built on bank, card, payroll and payment-platform feeds."
  },
  "autonomy": {
   "score": 4,
   "note": "Markets the AI Accountant as 'zero human intervention'; Meridian writes completed close output straight into the ledger."
  },
  "agent_readiness": {
   "score": 2,
   "note": "A free hosted MCP server for QuickBooks Online (August 2026) with per-company read-only or read-write; no API or agent interface to Pilot itself, and writes apply immediately."
  },
  "integration_ease": {
   "score": 2,
   "note": "Many inbound integrations (Stripe, Gusto, Mercury, Ramp, Brex, Bill.com); outbound only through QuickBooks or negotiated partner flows."
  },
  "market": {
   "score": 2,
   "note": "Venture-backed startups and small businesses up to custom multi-entity plans; Meridian adds accounting firms."
  }
 },
 "api": {
  "public": false,
  "style": "None for Pilot data. The Meridian QuickBooks connector is a hosted MCP server (https://qbo-connector.meridian.pilot.com/mcp) over the QuickBooks Online API.",
  "auth": "Connector: Intuit OAuth, with Pilot holding encrypted QuickBooks credentials and never passing them to the agent.",
  "limits": "Connector: one user per QuickBooks company on the free tier; access level fixed per company at connection time; QuickBooks Online only. Intuit's own API metering applies underneath.",
  "cost": "Connector free; team sharing is part of paid Meridian Starter (price not published).",
  "export": "Through QuickBooks Online, which Pilot uses as the core ledger; the client can log in and keep the file. No Pilot-native bulk export found.",
  "notes": "Supergood's API report card rates Pilot as having no outbound API; partner data flows exist with fintechs such as Mercury and Brex."
 },
 "pricing": {
  "model": "Monthly subscription by tier and monthly expense volume, plus annual tax fees and monthly CFO and COO retainers.",
  "entry": "Bookkeeping Essentials $99 a month for up to $100,000 in monthly expenses (pricing page, October 2026); the partner FAQ says bookkeeping starts at $169 a month.",
  "top": "Custom bookkeeping by quote; CFO Custom from $5,250 a month billed annually.",
  "notes": "Tax from $1,000 a year (single-member LLC) to $2,450+ (C-corp), each requiring Pilot Bookkeeping. Month-to-month, cancel any time. Free trial since 11 February 2026. Meridian pricing not published."
 },
 "agents": {
  "mcp": "official",
  "ai_features": "AI Accountant (February 2026): onboarding, historical cleanup, accrual bookkeeping, reconciliation, revenue recognition, depreciation, statements; 24/7 AI chat on past transactions and cash flow. Meridian (June 2026): the close engine sold to firms, with 52 automated checks per close. Meridian AI for ad hoc cleanups and journal entries (August 2026).",
  "agent_writes": "Yes. Pilot's own agent writes to client books. The Meridian connector lets an outside agent (Claude, ChatGPT, Codex) create and delete QuickBooks objects on read-write companies.",
  "approval": "No approval queue on connector writes: the terms say writes apply immediately and the user is responsible. The AI Accountant asks a human about judgment calls with potentially material impact; Meridian hands firms finished books to review after the fact.",
  "notes": "Pilot's MCP is official but serves QuickBooks data, not Pilot's. Pilot says the connector keeps no copy of the books."
 },
 "integration": {
  "scope": "QuickBooks Online as core ledger; Meridian also Xero and NetSuite. Inbound feeds from banks, cards, payroll (Gusto), payments (Stripe), spend (Ramp, Brex), AP (Bill.com), banking (Mercury), cap tables (Carta, Pulley). Partner FAQ claims 10,000+ tools.",
  "notes": "Integrations flow into Pilot; nothing documented flows out except through QuickBooks."
 },
 "preference": {
  "scale": "8,000+ businesses served since 2017 and 187,000+ months of books closed (Pilot, June 2026); 600,000 transactions a month (Meridian site). Revenue estimates disagree: about $43M (Latka) and $88.6M for 2025 (RocketReach), neither audited.",
  "ratings": "G2 4.7/5 from 142 reviews (October 2026 search result); Trustpilot 2.9/5 from 20 reviews (October 2026).",
  "sentiment": "Praise for responsive bookkeepers, easy onboarding and clear statements for startups. Complaints about price, slow tax-season replies, bank-sync gaps, unfinished work and the requirement to move onto QuickBooks Online."
 },
 "timeline": [
  {
   "date": "2017",
   "event": "Founded in San Francisco by Waseem Daher, Jeff Arnold and Jessica McKellar"
  },
  {
   "date": "2021-03-26",
   "event": "Series C extended to $100M at a $1.2B valuation"
  },
  {
   "date": "2024-12",
   "event": "Courts Bench clients after Bench's shutdown with a free Bench export and migration offer"
  },
  {
   "date": "2025-12-09",
   "event": "Local Partner Program for independent bookkeepers and firms"
  },
  {
   "date": "2026-02-04",
   "event": "AI Accountant launched, marketed as running bookkeeping with zero human intervention"
  },
  {
   "date": "2026-02-11",
   "event": "Free trial opened to any business"
  },
  {
   "date": "2026-06-16",
   "event": "Meridian launched for accounting firms: QuickBooks Online, Xero, NetSuite"
  },
  {
   "date": "2026-08-11",
   "event": "Free hosted QuickBooks Online MCP connector, per-company read-only or read-write"
  }
 ],
 "relation": "competitor",
 "relation_note": "The opposite pole in the same market: Pilot takes the work off the owner and its agent writes to the books unattended, while ownpurse keeps the owner's own record and lets writes out only after a person approves them. Its free MCP connector is also the nearest neighbour to ownpurse's agent interface.",
 "borrow": "A per-company read-only versus read-write choice made at connection time; staying on the incumbent ledger (QuickBooks) so the books stay portable; publishing the count of automated checks per close; a free connector as a funnel.",
 "avoid": "Immediate writes and delete tools with no preview or approval step; a decision log only the vendor holds; marketing 'zero human intervention' while terms leave accuracy with the owner.",
 "sources": [
  {
   "title": "Pilot pricing",
   "url": "https://pilot.com/pricing"
  },
  {
   "title": "Pilot unveils AI Accountant (Pilot blog, 4 Feb 2026)",
   "url": "https://pilot.com/blog/pilot-unveils-ai-accountant-a-major-leap-toward-artificial-general-intelligence-in-accounting"
  },
  {
   "title": "Pilot launches fully autonomous AI bookkeeper (Accounting Today, 6 Feb 2026)",
   "url": "https://www.accountingtoday.com/news/pilot-launches-fully-autonomous-ai-bookkeeper"
  },
  {
   "title": "Pilot says Meridian runs monthly close (CPA Practice Advisor, 23 Jun 2026)",
   "url": "https://www.cpapracticeadvisor.com/2026/06/23/pilot-says-new-ai-accounting-platform-meridian-runs-monthly-close-from-start-to-finish/185541/"
  },
  {
   "title": "Pilot launches Meridian (Insightful Accountant)",
   "url": "https://insightfulaccountant.com/accounting-tech/general-ledger/pilot-launches-meridian-the-ai-platform-that-fully-closes-the-books-for-accounting-firms/"
  },
  {
   "title": "Meridian",
   "url": "https://meridian.pilot.com"
  },
  {
   "title": "QuickBooks MCP server (Meridian blog, Aug 2026)",
   "url": "https://meridian.pilot.com/blog/quickbooks-mcp-server"
  },
  {
   "title": "QuickBooks Connector by Meridian FAQ",
   "url": "https://qbo-connector.meridian.pilot.com/faq"
  },
  {
   "title": "QuickBooks Connector terms",
   "url": "https://qbo-connector.meridian.pilot.com/terms"
  },
  {
   "title": "Pilot Partner Hub FAQ",
   "url": "https://partners.pilot.com/Partner%20Hub/pages/faq.html"
  },
  {
   "title": "Pilot Partner Hub announcements",
   "url": "https://partners.pilot.com/Partner%20Hub/pages/whats-new.html"
  },
  {
   "title": "Pilot Bench export",
   "url": "https://pilot.com/bench-export"
  },
  {
   "title": "Pilot API report card (Supergood)",
   "url": "https://supergood.ai/api-report-card/pilot"
  },
  {
   "title": "Newest fintech unicorn (BusinessWire, 26 Mar 2021)",
   "url": "https://www.businesswire.com/news/home/20210326005284/en/Meet-the-Newest-Fintech-Unicorn-Bezos-Expeditions-Whale-Rock-join-Pilot%E2%80%99s-Push-to-Modernize-the-Back-Office"
  },
  {
   "title": "Pilot on Trustpilot",
   "url": "https://www.trustpilot.com/review/pilot.com"
  },
  {
   "title": "Pilot on G2",
   "url": "https://g2.com/products/pilot/reviews"
  },
  {
   "title": "Agentman listing of the Meridian connector",
   "url": "https://agentman.ai/agentskills/connections/mcp-server/meridian-connector-for-quickbooks"
  }
 ],
 "unverified": [
  "Total funding raised: reported as $158M+ (TechCrunch), $161.3M (CB Insights), $215M-$218M (Tryfundable, Latka)",
  "Revenue: $43M (Latka) versus $88.6M for 2025 (RocketReach); both third-party estimates",
  "Entry price: $99/month on the pricing page versus $169/month in the partner FAQ; the Core tier's price did not parse",
  "G2 count of 142 reviews and 4.7 rating come from a search snippet; the G2 page refused direct fetch",
  "Whether Essentials-tier clients own the QuickBooks file or Pilot does; the Custom tier advertises keeping your existing QuickBooks account",
  "When Jessica McKellar became CEO (Waseem Daher was the earlier CEO)",
  "Agentman's report that three connector tools delete and that no scope separates create from delete",
  "Meridian Starter and Meridian platform pricing"
 ],
 "researched": "2026-10-10"
}
---

## What it is

Pilot is a US bookkeeping, tax and CFO service for startups and small businesses, founded in 2017 in San Francisco. It keeps your books on QuickBooks Online, which its [partner FAQ](https://partners.pilot.com/Partner%20Hub/pages/faq.html) calls the core ledger, and adds tax filing, fractional CFO work and outsourced operations. In 2026 it turned its internal tooling into two products: an [AI Accountant](https://pilot.com/blog/pilot-unveils-ai-accountant-a-major-leap-toward-artificial-general-intelligence-in-accounting) for its own clients (4 February), and [Meridian](https://www.cpapracticeadvisor.com/2026/06/23/pilot-says-new-ai-accounting-platform-meridian-runs-monthly-close-from-start-to-finish/185541/), the same close engine sold to accounting firms (16 June).

## How it works

You connect banks, cards, payroll and payment platforms. Pilot's software, and for higher tiers a US bookkeeper, categorizes, reconciles and closes the month in QuickBooks. Pilot says the AI Accountant does full accrual bookkeeping from onboarding to close "with zero human intervention", flags judgment calls with a potentially material impact for a human, and logs every decision ([Accounting Today, 6 February 2026](https://www.accountingtoday.com/news/pilot-launches-fully-autonomous-ai-bookkeeper)). Meridian, which Pilot says it has used internally since 2017 to close more than 187,000 months of books for over 8,000 businesses, writes completed output directly into the general ledger and returns review-ready statements; it supports QuickBooks Online best, plus Xero and NetSuite ([Insightful Accountant](https://insightfulaccountant.com/accounting-tech/general-ledger/pilot-launches-meridian-the-ai-platform-that-fully-closes-the-books-for-accounting-firms/)). The figures are Pilot's own.

Pricing in October 2026 starts at $99 a month for Essentials (up to $100,000 in monthly expenses), with Custom by quote, tax from $1,000 a year and CFO from $1,750 a month ([pricing](https://pilot.com/pricing)). The partner FAQ quotes $169 a month, so the entry price depends on which page you read. Plans are month-to-month.

## Data and access

Pilot has no public API to its own data; [Supergood's report card](https://supergood.ai/api-report-card/pilot) finds no developer portal, and data flows exist only with negotiated partners such as Mercury and Brex. Because the books sit in QuickBooks, you can log in, pull the ledger through QuickBooks, and keep the file when you leave. That is a real strength compared with Bench's proprietary ledger, and Pilot used it to win Bench clients with a [free export](https://pilot.com/bench-export). What you do not get is Pilot's decision log or its working data.

## Agents

On 11 August 2026 Pilot released a [free hosted MCP server for QuickBooks Online](https://meridian.pilot.com/blog/quickbooks-mcp-server) that works with Claude, ChatGPT and Codex. You sign in with Intuit and choose read-only or read-write separately for each company; changing it means reconnecting. Pilot says it keeps no copy of your books ([FAQ](https://qbo-connector.meridian.pilot.com/faq)). Writes on a read-write company apply immediately; there is no preview, queue or approval step, and the terms make you responsible. The free tier allows one user per company; sharing is part of paid Meridian Starter.

## Customers

Pilot serves venture-backed startups and small businesses, and through Meridian, firms. G2 shows 4.7 out of 5 from 142 reviews; [Trustpilot](https://www.trustpilot.com/review/pilot.com) shows 2.9 from 20. Reviewers praise responsive bookkeepers and clean startup books, and complain about price, slow tax replies, sync gaps and being moved onto QuickBooks. Revenue estimates range from about $43M to $88.6M and are unaudited.

## History

The founders previously built Ksplice and Zulip. Pilot became a unicorn in March 2021 when Bezos Expeditions and Whale Rock extended its Series C to $100M at a $1.2B valuation ([BusinessWire](https://www.businesswire.com/news/home/20210326005284/en/Meet-the-Newest-Fintech-Unicorn-Bezos-Expeditions-Whale-Rock-join-Pilot%E2%80%99s-Push-to-Modernize-the-Back-Office)). It is now the largest of the startup bookkeeping services and the one moving fastest from people to software.

## For ownpurse

Pilot is the opposite pole. Its answer to messy books is to take the work away, write to the ledger unattended and keep the log itself. ownpurse keeps the owner's record and lets writes out only as sealed packages a person approves. The two can coexist: a Pilot client still owns a QuickBooks file that ownpurse can read and check to the cent. Worth borrowing: the per-company read or write choice, and staying on the incumbent ledger so the books stay portable. Worth avoiding: immediate writes and deletes with no approval step, and a history only the vendor can see.
