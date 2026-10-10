---
{
 "slug": "truewind",
 "name": "Truewind",
 "url": "https://www.truewind.ai",
 "segment": "agent-beside-ledger",
 "status": "active",
 "founded": "2023 (YC profile); the founders started building in November 2022 per a secondary source",
 "hq": "San Francisco, California, USA",
 "backing": "Y Combinator (Winter 2023). Seed of about $3M in September 2023 led by Fin Capital and Y Combinator (Axios headline; some sources say $4M). $13M Series A on 11 December 2024 co-led by Rho Capital and Thomson Reuters Ventures with Pathlight Ventures; total over $17M.",
 "tagline": "AI bookkeeping and close software for accounting firms and finance teams that posts entries to QuickBooks Online, Sage Intacct or Intuit Enterprise Suite, with the team reviewing exceptions.",
 "scope": "Truewind turns bank statements, card activity, payout reports, PDFs and spreadsheets into categorized transactions, journal entries, reconciliations, prepaid and fixed-asset schedules, flux analysis and SOPs, and posts them to the customer's existing GL. It has vertical workflows for brokerage, nonprofits, point of sale, payroll and field services. It does not keep its own ledger, does not publish an API, and does not publish pricing.",
 "domains": [
  "ledger",
  "bank",
  "documents",
  "close",
  "reporting"
 ],
 "customers_served": [
  "smb",
  "mid-market",
  "accountants"
 ],
 "spectrum": {
  "custody": {
   "score": 1,
   "note": "'Keep Sage or QBO as your source of truth': the books stay in the provider's cloud, and Truewind keeps its workpapers and activity history."
  },
  "locality": {
   "score": 0,
   "note": "Hosted service only."
  },
  "history": {
   "score": 1,
   "note": "An activity history 'preserves what was proposed, reviewed, and changed'; the GL itself is edited in place and there are no as-of queries."
  },
  "openness": {
   "score": 0,
   "note": "Closed, demo-led, no public API, no pricing page; export details are not published."
  },
  "breadth": {
   "score": 2,
   "note": "Bank, card, payout, payroll, POS and brokerage data into the GL plus close work; no payments, bill pay or tax."
  },
  "autonomy": {
   "score": 2,
   "note": "It posts entries directly to the GL, but its pitch is review by the team: approval gates and control policies are set by the customer, and 'your team only reviews the exceptions' on reconciliation."
  },
  "agent_readiness": {
   "score": 0,
   "note": "No API, CLI or MCP published; it is an agent product, not something agents drive."
  },
  "integration_ease": {
   "score": 1,
   "note": "Connects to Sage Intacct through Sage's APIs and to QBO and Intuit Enterprise Suite, but onboarding is by demo and nothing can connect to Truewind."
  },
  "market": {
   "score": 3,
   "note": "Accounting firms (EisnerAmper, Frank Rimerman) and in-house finance teams on Sage Intacct and QBO, from startups to mid-market."
  }
 },
 "api": {
  "public": false,
  "style": "None published",
  "auth": "n/a (Truewind connects to Sage Intacct through Sage's APIs, and to QBO and Intuit Enterprise Suite)",
  "limits": "n/a",
  "cost": "No API offered",
  "export": "Not specified; the trust page mentions handling of data access and portability requests",
  "notes": "Truewind can anonymize some fields (memo, check number) before they reach AI workflows, but says it does 'not remove all personally identifiable information from every AI input'. It is SOC 2 certified, type not stated."
 },
 "pricing": {
  "model": "Not published; sold by demo",
  "entry": "Not published (checked 2026-10-10; /pricing returns 404)",
  "top": "Not published",
  "notes": "No public figures were found."
 },
 "agents": {
  "mcp": "none",
  "ai_features": "Rule-driven transaction coding using customer SOPs, prepaid and fixed-asset schedules, third-party data reconciliation with mismatch flags, flux analysis against historical patterns, AI-assisted SOPs, and follow-up messages to the client when it is unsure.",
  "agent_writes": "Yes. 'AI Accountants' post entries directly to the Sage Intacct GL, and depreciation and amortization entries are 'posted back'; it also posts to QBO and Intuit Enterprise Suite.",
  "approval": "The customer 'define[s] the approval gates and control policies'. Role-based access controls who reviews, approves and exports, and the team reviews exceptions. Whether entries are drafted or posted at once is not published.",
  "notes": "The homepage does not use the word 'agent'. The YC profile calls it a 'digital staff accountant'."
 },
 "integration": {
  "scope": "QuickBooks Online, Sage Intacct (marketplace partner; partnership posts dated 17 and 18 November 2025) and Intuit Enterprise Suite as ledgers; bank, card, payroll, POS, brokerage and payout data as inputs.",
  "notes": "It calls itself the '#1 Rated Sage Marketplace Partner'."
 },
 "preference": {
  "scale": "'Trusted by 500+ accountants' (homepage) and '500+ finance teams' (Sage page), 2026, self-reported. Revenue grew 4x in 2024 and the platform 'absorbs 47% of month-end close tasks' (company, December 2024). YC lists a team of 30.",
  "ratings": "Not retrieved: G2 blocked direct fetch; Truewind cites a five-star G2 review without a count.",
  "sentiment": "Customer quotes on the site praise time saved: 'cut my categorization time by about 75%' (HHL Advisors). Independent review sentiment could not be gathered for this dossier."
 },
 "timeline": [
  {
   "date": "2022-11",
   "event": "Alex Lee and Tennison Chan start building Truewind in San Francisco (secondary source)."
  },
  {
   "date": "2023-01",
   "event": "Joins Y Combinator, Winter 2023 batch."
  },
  {
   "date": "2023-09",
   "event": "Seed of about $3M led by Fin Capital and Y Combinator (Axios)."
  },
  {
   "date": "2024-12-11",
   "event": "$13M Series A co-led by Rho Capital and Thomson Reuters Ventures with Pathlight; total over $17M; customers include EisnerAmper and Frank Rimerman."
  },
  {
   "date": "2025-11-17",
   "event": "Sage Intacct partnership posts; positions itself as the top-rated Sage Marketplace partner."
  },
  {
   "date": "2026-10",
   "event": "Posts to QBO, Sage Intacct and Intuit Enterprise Suite; claims 500+ accountants; no public pricing."
  }
 ],
 "relation": "neighbour",
 "relation_note": "Truewind is an agent beside the ledger for the firm and mid-market end: it leaves the ledger where it is and keeps a proposed-reviewed-changed history, but that history and the workpapers stay with Truewind, and only Truewind's agents can act. ownpurse would give a firm using Truewind an owner-held record of what was posted.",
 "borrow": "Recording what was proposed, reviewed and changed as distinct states; customer-written SOPs as the rules an agent follows; flux analysis against the record's own history before close.",
 "avoid": "Demo-only access with no published pricing or API; direct GL posting with approval rules that are not published.",
 "sources": [
  {
   "title": "Truewind homepage",
   "url": "https://www.truewind.ai"
  },
  {
   "title": "Truewind AI bookkeeping product",
   "url": "https://www.truewind.ai/product/ai-bookkeeping"
  },
  {
   "title": "Truewind Sage partner page",
   "url": "https://www.truewind.ai/sage-partner"
  },
  {
   "title": "Truewind trust and security",
   "url": "https://www.truewind.ai/trust-and-security"
  },
  {
   "title": "Announcing Truewind's Series A",
   "url": "https://www.truewind.ai/blog/announcing-truewind-series-a"
  },
  {
   "title": "Truewind on Y Combinator",
   "url": "https://www.ycombinator.com/companies/truewind"
  },
  {
   "title": "Truewind raises $13 million Series A (CPA Practice Advisor)",
   "url": "https://www.cpapracticeadvisor.com/2025/01/08/truewind-accounting-ai-platform-raises-13-million-in-series-a-funding/154157/"
  },
  {
   "title": "AI accounting startup Truewind books $3 million seed (Axios)",
   "url": "https://www.axios.com/pro/fintech-deals/2023/09/18/ai-accounting-startup-truewind-3-million-seed"
  },
  {
   "title": "Founders in Focus: Alex Lee, CEO of Truewind",
   "url": "https://mbaventures.substack.com/p/founders-in-focus-alex-lee-ceo-truewind"
  }
 ],
 "unverified": [
  "Seed amount ($3M per the Axios headline; $4M per another source); the Axios article body could not be fetched",
  "Start of building in November 2022 (secondary source)",
  "Pricing (not published)",
  "G2 rating and review count (fetch blocked)",
  "Whether entries are drafted for approval or posted immediately by default",
  "SOC 2 type",
  "'500+ accountants' and '500+ finance teams', 4x revenue growth and 47% of close tasks are self-reported"
 ],
 "researched": "2026-10-10"
}
---

## What it is

Truewind is AI bookkeeping and close software for accounting firms and in-house finance teams. Its homepage says: ["Keep Sage or QBO as your source of truth"](https://www.truewind.ai). It does not replace the ledger. It turns bank statements, card activity, payout reports, PDFs and spreadsheets into categorized transactions, journal entries, reconciliations and schedules, then posts them to QuickBooks Online, Sage Intacct or Intuit Enterprise Suite. Y Combinator lists it (Winter 2023) as a ["digital staff accountant"](https://www.ycombinator.com/companies/truewind).

## How it works

Truewind applies your rules and SOPs to every transaction. It builds prepaid and fixed-asset schedules, pulls third-party data to match against your books, and runs flux analysis against past patterns before close. It has workflows for brokerage (family offices), donations (nonprofits), point of sale, payroll and field services. On [Sage Intacct](https://www.truewind.ai/sage-partner) it connects "directly to your GL through Sage's APIs", and its AI Accountants "post them directly". You "define the approval gates and control policies". The [product page](https://www.truewind.ai/product/ai-bookkeeping) promises "audit-friendly activity history that preserves what was proposed, reviewed, and changed", and role-based access over who reviews, approves and exports.

Its strength is fit with how firms already work: the ledger stays put, staff review exceptions, and the work arrives as workpapers a reviewer recognizes.

## Data and access

Truewind publishes no API and no pricing. Its /pricing page returns 404, and the site offers only demos. The [trust page](https://www.truewind.ai/trust-and-security) says it is SOC 2 certified, without giving the type. It can anonymize some fields before they reach AI workflows, but does "not remove all personally identifiable information from every AI input". It does not name its model providers or describe how you export data.

## Agents

Truewind is an agent you hire, not one you drive. It offers no MCP server, CLI or token-scoped API. Its writes go to the provider's GL under approval gates the customer configures, and its record of proposals and reviews lives on Truewind's side.

## Customers

The site claims "500+ accountants" and "500+ finance teams". The [Series A post](https://www.truewind.ai/blog/announcing-truewind-series-a) names EisnerAmper and Frank Rimerman, reports revenue up 4x in 2024 and says the platform absorbs 47% of month-end close tasks (company figures). A partner at HHL Advisors says it cut card categorization time by about 75%. This dossier has no independent G2 or Capterra figures.

## History

Alex Lee (CEO) and Tennison Chan (CTO) founded Truewind in San Francisco and went through YC in Winter 2023. Axios reported a seed of about $3M led by Fin Capital and YC in [September 2023](https://www.axios.com/pro/fintech-deals/2023/09/18/ai-accounting-startup-truewind-3-million-seed). On 11 December 2024 it raised a $13M Series A co-led by Rho Capital and Thomson Reuters Ventures, bringing its total to over $17M. It began with startups and has moved toward firms and Sage Intacct mid-market teams, with partnership posts in November 2025.

## For ownpurse

Truewind already treats proposed, reviewed and changed as separate states, which is close to ownpurse's held-but-not-posted fact. The difference is custody and openness. In ownpurse those states live in the owner's log, and an agent from any vendor can propose into it through a CLI with `--json`. For a firm that uses Truewind, ownpurse is the independent record of what reached the books and when.
