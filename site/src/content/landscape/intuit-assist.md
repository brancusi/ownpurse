---
{
 "slug": "intuit-assist",
 "name": "Intuit Assist",
 "url": "https://www.intuit.com/intuitassist/",
 "segment": "agent-native",
 "status": "pivoted",
 "founded": "Announced 2023 (Intuit founded 1983); folded into 'Intuit Intelligence' and a virtual team of AI agents from 2025",
 "hq": "Mountain View, California (Intuit Inc.)",
 "backing": "Part of Intuit Inc., listed on Nasdaq (INTU). FY2026 revenue $21.4B, Global Business Solutions $12.9B (reported 2026-08-25). $100M+ multi-year contract with OpenAI (announced 2025-11-18).",
 "tagline": "Intuit's generative AI assistant, now a team of in-product AI agents under 'Intuit Intelligence', that works on QuickBooks books, payments, payroll and customers.",
 "scope": "Intuit Assist began in 2023 as a GenAI assistant across QuickBooks, TurboTax, Credit Karma and Mailchimp. In July 2025 Intuit added named agents in QuickBooks (Accounting, Payments, Finance, Customer, then Payroll, Project Management and Sales Tax), and in October 2025 grouped them under Intuit Intelligence, which can also ingest third-party systems and spreadsheets. The agents act inside QuickBooks and Intuit Enterprise Suite. Outside agents use Intuit's open-source QuickBooks MCP server or the metered API; Intuit apps inside ChatGPT were announced as coming. It does not give the owner a copy of their books or reach other ledgers' records.",
 "domains": [
  "ledger",
  "bank",
  "payments",
  "invoicing",
  "payroll",
  "tax",
  "reporting",
  "close",
  "commerce"
 ],
 "customers_served": [
  "freelancer",
  "smb",
  "mid-market",
  "accountants"
 ],
 "spectrum": {
  "custody": {
   "score": 1,
   "note": "Intuit holds the books; reports export to Excel and the API can read everything, but reads are metered since 2025."
  },
  "locality": {
   "score": 0,
   "note": "Cloud only; the open-source MCP server runs locally but only relays to QuickBooks Online."
  },
  "history": {
   "score": 1,
   "note": "QuickBooks keeps an audit log and labels automated actions; no append-only owner record or as-of queries."
  },
  "openness": {
   "score": 1,
   "note": "The agents are closed with no API; the QuickBooks API is public but data-out calls are metered; the MCP server is open source."
  },
  "breadth": {
   "score": 4,
   "note": "Books, bank feeds, payments, bill pay, payroll, sales tax, projects, CRM and marketing (Mailchimp), with third-party and spreadsheet data ingestion in Intuit Intelligence."
  },
  "autonomy": {
   "score": 3,
   "note": "Resolves high-confidence transactions automatically under customer rules; payroll arrives as a draft to approve; chat 'acts only after the user confirms an action'."
  },
  "agent_readiness": {
   "score": 2,
   "note": "An official open-source MCP server with 145 tools and flags to disable writes, but it is local, needs your own developer app and a public HTTPS redirect for production; hosted third-party agent access was announced, not confirmed live."
  },
  "integration_ease": {
   "score": 2,
   "note": "Self-serve developer portal and a large app ecosystem, but the App Partner Program meters reads (free Builder tier capped at 500,000 CorePlus calls a month; Silver $300 a month)."
  },
  "market": {
   "score": 3,
   "note": "From solo users on Simple Start to mid-market on QuickBooks Online Advanced and Intuit Enterprise Suite."
  }
 },
 "api": {
  "public": true,
  "style": "Agents: none. QuickBooks Online: REST accounting API, webhooks; MCP server (open source, local) with get, search, create, update and delete tools",
  "auth": "OAuth 2.0 via an Intuit Developer app; the MCP server does a one-time browser handshake and stores a rotating refresh token",
  "limits": "Builder tier blocked past 500,000 CorePlus (data-out) calls a month; per-minute throttles not re-checked here",
  "cost": "App Partner Program from 2025-07-28: Builder $0 (500K CorePlus calls/month), Silver $300/month (1M, then $3.50 per 1,000), Gold $1,700/month, Platinum $4,500/month (up to 75M, then $0.25 per 1,000). Core (data-in, mostly writes) calls free and unlimited. Full variable fees from 2025-11-01",
  "export": "Reports and lists to Excel/PDF in the app; bulk data through the metered API; no full-ledger backup file found",
  "notes": "Writes are free and reads are metered, which favours data flowing into QuickBooks over data flowing out."
 },
 "pricing": {
  "model": "Included in QuickBooks Online plans, with agent features scaling by tier",
  "entry": "Simple Start $38/month list ($19 promo): limited AI chat (beta), categorisation, 'handles the busywork' (beta)",
  "top": "Advanced $340/month list ($170 promo): Finance AI, Project Management AI, continuous books checks; Intuit Enterprise Suite above that, unpriced here",
  "notes": "Free plan has no AI features listed. Essentials $85 adds Accounting AI, Payments AI, Business Tax AI; Plus $140 adds Sales Tax AI, Customer AI, anomaly detection (prices from quickbooks.intuit.com/pricing, 2026-10-10). 'Additional terms, conditions and fees may apply.'"
 },
 "agents": {
  "mcp": "official",
  "ai_features": "Intuit Assist chat (2023), then Accounting, Payments, Finance, Customer, Marketing, Payroll, Project Management and Sales Tax agents; Intuit Intelligence Chat (beta on QBO Advanced and Intuit Enterprise Suite); Intuit Accountant Suite; agentic readiness check for Desktop migrations",
  "agent_writes": "Intuit's own agents write inside QuickBooks (categorisation, reconciliation, invoices and reminders, payroll drafts). The open-source MCP server lets outside agents create (25 entities), update (26) and delete or void (20) records directly, unless you set QUICKBOOKS_DISABLE_WRITE, _UPDATE or _DELETE.",
  "approval": "Intuit Intelligence Chat 'acts only after the user confirms an action'; payroll is sent as 'a ready-to-approve payroll draft'; multi-entity entries are drafted 'for review and approval'; high-confidence transactions resolve automatically under customer rules and automated actions are labelled. The MCP server has no per-write confirmation, only environment flags.",
  "notes": "The Intuit Assist name appears to have given way to Intuit Intelligence in 2025-26 product copy; the Assist page still reads as a 2023 forward-looking announcement."
 },
 "integration": {
  "scope": "QuickBooks App Store ecosystem; Intuit Intelligence ingests 'data from Intuit sources, third-party systems, and spreadsheets'; Mailchimp; OpenAI partnership for Intuit apps in ChatGPT; Circle partnership for stablecoin payments (Dec 2025)",
  "notes": "Integration is mostly inbound to QuickBooks. The read meter makes it costlier for third parties, including an owner's own tools, to pull data out."
 },
 "preference": {
  "scale": "Intuit: about 100 million customers worldwide across products; QuickBooks Online Accounting revenue up 23% in FY2026 (2026-08-25). No agent usage figures disclosed.",
  "ratings": "Trustpilot (QuickBooks): 3.7/5 from 16,890 reviews; Apple App Store (US QuickBooks app): 4.7/5 from 271K ratings (both read 2026-10-10). G2 and Capterra blocked.",
  "sentiment": "Praise for phone support and onboarding staff; complaints about interface changes that make simple tasks harder, bugs, forced plan migrations, price, and billing after cancellation. One review read accuses Intuit of 'trying to bolt AI onto everything'. Intuit's own survey says 78% find its AI makes running the business easier (vendor figure)."
 },
 "timeline": [
  {
   "date": "2023",
   "event": "Intuit Assist announced as a GenAI assistant across Intuit products (exact date not confirmed)"
  },
  {
   "date": "2025-07-01",
   "event": "Virtual team of AI agents in QuickBooks Online (Accounting, Payments, Finance, Customer; Payroll and Project Management to follow), rolling out in the US"
  },
  {
   "date": "2025-07-22",
   "event": "Agentic AI experiences for Intuit Enterprise Suite"
  },
  {
   "date": "2025-07-28",
   "event": "App Partner Program with metered CorePlus (read) calls; full fees from 2025-11-01"
  },
  {
   "date": "2025-10-28",
   "event": "Intuit Intelligence 'system of intelligence' and Intuit Accountant Suite; Sales Tax and Payroll agents"
  },
  {
   "date": "2025-11-14",
   "event": "AI agents launched for UK and Canadian businesses"
  },
  {
   "date": "2025-11-18",
   "event": "Intuit and OpenAI partnership, $100M+ multi-year contract; Intuit apps in ChatGPT announced"
  },
  {
   "date": "2026-08-12",
   "event": "Intuit Intelligence Chat in beta for QBO Advanced and Intuit Enterprise Suite; acts only after user confirmation"
  },
  {
   "date": "2026-08-25",
   "event": "FY2026 results: revenue $21.4B, QuickBooks Online Accounting revenue up 23%"
  }
 ],
 "relation": "neighbour",
 "relation_note": "QuickBooks is a planned ownpurse source; Intuit's agents keep the work inside QuickBooks and Intuit meters reads, so an owner's own agent pays to look. ownpurse offers the local, cross-provider record and one approval gate that Intuit's per-feature model does not.",
 "borrow": "'Acts only after the user confirms an action' as the default for chat; drafts to approve (payroll) as a pattern; clearly labelled automated actions; environment flags in the MCP server to turn off create, update and delete separately.",
 "avoid": "Approval rules that vary agent by agent; metering data-out while writes stay free; rebranding the agent layer every year so customers cannot tell what changed; an MCP server that writes directly once write flags are left on.",
 "sources": [
  {
   "title": "Intuit Assist",
   "url": "https://www.intuit.com/intuitassist/"
  },
  {
   "title": "Virtual team of AI agents (2025-07-01)",
   "url": "https://investors.intuit.com/news-events/press-releases/detail/1258/intuit-introduces-ground-breaking-virtual-team-of-ai-agents-to-fuel-growth-for-businesses"
  },
  {
   "title": "App Partner Program (2025-07-28)",
   "url": "https://investors.intuit.com/news-events/press-releases/detail/1261/intuit-launches-new-app-partner-program-to-drive-growth-for-third-party-developers-and-enhance-customer-experiences"
  },
  {
   "title": "Apideck: QuickBooks API pricing and the App Partner Program",
   "url": "https://www.apideck.com/blog/quickbooks-api-pricing-and-the-intuit-app-partner-program"
  },
  {
   "title": "Intuit Intelligence (2025-10-28)",
   "url": "https://investors.intuit.com/news-events/press-releases/detail/1277/intuit-unveils-revolutionary-system-of-intelligence-to-help-businesses-grow-in-the-ai-era"
  },
  {
   "title": "Intuit and OpenAI (2025-11-18)",
   "url": "https://investors.intuit.com/news-events/press-releases/detail/1284/intuit-and-openai-join-forces-to-revolutionize-financial-intelligence-powering-every-person-business-and-dream-with-personalized-experiences"
  },
  {
   "title": "Mid-market conversational AI (2026-08-12)",
   "url": "https://investors.intuit.com/news-events/press-releases/detail/1319/intuit-advances-its-mid-market-platform-with-conversational-ai-enterprise-scale-and-deep-industry-workflows-for-cfos-and-accounting-firms"
  },
  {
   "title": "FY2026 results (2026-08-25)",
   "url": "https://investors.intuit.com/news-events/press-releases/detail/1320/intuit-reports-fourth-quarter-and-full-year-fiscal-2026-results-sets-fiscal-2027-guidance"
  },
  {
   "title": "Intuit press releases",
   "url": "https://investors.intuit.com/news-events/press-releases"
  },
  {
   "title": "QuickBooks pricing",
   "url": "https://quickbooks.intuit.com/pricing/"
  },
  {
   "title": "intuit/quickbooks-online-mcp-server",
   "url": "https://github.com/intuit/quickbooks-online-mcp-server"
  },
  {
   "title": "Trustpilot: QuickBooks",
   "url": "https://www.trustpilot.com/review/quickbooks.intuit.com"
  },
  {
   "title": "Apple App Store: QuickBooks",
   "url": "https://apps.apple.com/us/app/quickbooks-accounting/id584606479"
  }
 ],
 "unverified": [
  "Exact announcement date of Intuit Assist (page assets dated 2023; commonly reported as September 2023)",
  "Whether Intuit has formally retired the Intuit Assist name in favour of Intuit Intelligence (App Store notes use Intuit Intelligence; no announcement found)",
  "Whether Intuit apps in ChatGPT are live (announced as 'coming soon' in November 2025)",
  "Gold tier CorePlus allowance and overage rate",
  "QuickBooks API per-minute throttles",
  "MCP server license (README says MIT, GitHub sidebar says Apache-2.0)",
  "Intuit Enterprise Suite pricing",
  "G2 and Capterra ratings (pages blocked); Reddit sentiment (unreachable)"
 ],
 "researched": "2026-10-10"
}
---

## What it is

Intuit Assist was Intuit's [generative AI assistant](https://www.intuit.com/intuitassist/), announced in 2023 across QuickBooks, TurboTax, Credit Karma and Mailchimp. Its page still reads as a forward-looking announcement. The working product today is a set of named agents inside QuickBooks, introduced on [1 July 2025](https://investors.intuit.com/news-events/press-releases/detail/1258/intuit-introduces-ground-breaking-virtual-team-of-ai-agents-to-fuel-growth-for-businesses), and grouped since [28 October 2025](https://investors.intuit.com/news-events/press-releases/detail/1277/intuit-unveils-revolutionary-system-of-intelligence-to-help-businesses-grow-in-the-ai-era) under Intuit Intelligence. This dossier covers that whole line.

## How it works

The agents divide the work: Accounting (categorisation, reconciliation), Payments (late-payment prediction, invoices and reminders), Finance (KPIs, forecasting), Customer (leads, emails), and later Payroll, Project Management and Sales Tax. Which ones you get depends on the plan: Simple Start ($38 a month list) has limited AI chat; Plus ($140) adds Sales Tax AI and Customer AI; Advanced ($340) adds Finance AI and continuous books checks ([pricing](https://quickbooks.intuit.com/pricing/)).

Intuit's approval language is among the clearest of the incumbents. [Intuit Intelligence Chat](https://investors.intuit.com/news-events/press-releases/detail/1319/intuit-advances-its-mid-market-platform-with-conversational-ai-enterprise-scale-and-deep-industry-workflows-for-cfos-and-accounting-firms), in beta on QuickBooks Online Advanced and Intuit Enterprise Suite since August 2026, "acts only after the user confirms an action." Payroll arrives as "a ready-to-approve payroll draft". Elsewhere it "resolves high-confidence transactions automatically under rules the customer sets" and labels what it did.

## Data and access

The agents have no API. Outside agents go through the QuickBooks API or Intuit's open-source [MCP server](https://github.com/intuit/quickbooks-online-mcp-server), which offers 145 tools including create, update and delete for most entities, with environment flags to switch writes off. It runs locally, needs your own Intuit developer app, and needs a public HTTPS redirect for production. Since [28 July 2025](https://investors.intuit.com/news-events/press-releases/detail/1261/intuit-launches-new-app-partner-program-to-drive-growth-for-third-party-developers-and-enhance-customer-experiences) reads are metered: the free Builder tier stops at 500,000 data-out calls a month, Silver is $300 a month plus $3.50 per 1,000 calls, and writes are free ([Apideck summary](https://www.apideck.com/blog/quickbooks-api-pricing-and-the-intuit-app-partner-program)).

## Agents

Intuit's strategy is to run the agents itself, on its own data, sold through plan tiers, with OpenAI models under a [$100M+ contract](https://investors.intuit.com/news-events/press-releases/detail/1284/intuit-and-openai-join-forces-to-revolutionize-financial-intelligence-powering-every-person-business-and-dream-with-personalized-experiences) and Intuit apps in ChatGPT announced as coming. An owner's own agent is a second-class citizen: it reads through a meter and writes through an MCP server with no per-write confirmation.

## Customers

Intuit reports about 100 million customers across products and 23% growth in QuickBooks Online Accounting revenue in FY2026. QuickBooks rates 3.7 on Trustpilot from 16,890 reviews and 4.7 on the US App Store from 271,000 ratings. Reviewers praise phone support and onboarding; they complain about interface changes, bugs, forced migrations and billing, and one calls the AI push "bolt AI onto everything".

## History

Intuit Assist in 2023, agents in July 2025, metered reads the same month, Intuit Intelligence in October 2025, OpenAI in November 2025, conversational AI for mid-market in August 2026. The name changes faster than the product.

## For ownpurse

QuickBooks is a planned source, and Intuit's choices shape that connector: reads cost money, so ownpurse's local record saves the owner real fees each time an agent asks a question. Intuit's "acts only after the user confirms" is the right default and worth citing; ownpurse goes further by making that confirmation one gate for every write, sealed and logged, whoever wrote the proposal.
