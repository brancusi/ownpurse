---
{
 "slug": "meridian-quickbooks-connector",
 "name": "QuickBooks Connector by Meridian",
 "url": "https://meridian.pilot.com/blog/quickbooks-mcp-server",
 "segment": "agent-native",
 "status": "active",
 "founded": "Connector announced 11 August 2026; Pilot founded 2017",
 "hq": "San Francisco, CA (Pilot also has a Nashville office)",
 "backing": "Built by Pilot, the bookkeeping company, which has raised over $150M across Series A to C from Sequoia, Index Ventures, Stripe, Bezos Expeditions and Whale Rock (Pilot about page); the Series C is linked with a $1.2B valuation.",
 "tagline": "A free hosted MCP server that lets Claude, Cowork, ChatGPT and Codex read and write QuickBooks Online, read-only or read-write per company.",
 "scope": "Connects an AI client to one or more QuickBooks Online companies through an Intuit sign-in, with transaction-level reads and writes including journal entries, bulk invoices, intercompany entries and importing PDF card statements. Pilot says it keeps no copy of your books. It does not do shared access, review workflow or a close process; those are paid Meridian features. QuickBooks Online only.",
 "domains": [
  "ledger",
  "invoicing",
  "documents",
  "integration"
 ],
 "customers_served": [
  "accountants",
  "smb",
  "freelancer"
 ],
 "spectrum": {
  "custody": {
   "score": 1,
   "note": "The books stay in Intuit's cloud and the connector keeps no copy, so the owner holds nothing new; nothing is lost either."
  },
  "locality": {
   "score": 0,
   "note": "Hosted by Pilot; nothing to install or run, and nothing runs on your machine."
  },
  "history": {
   "score": 1,
   "note": "The connector logs no QuickBooks records; history is whatever QuickBooks' own audit log keeps."
  },
  "openness": {
   "score": 1,
   "note": "Free but closed: no public tool list, docs or source; you depend on Pilot's hosted service."
  },
  "breadth": {
   "score": 1,
   "note": "QuickBooks Online only, plus PDF card statements imported into it."
  },
  "autonomy": {
   "score": 3,
   "note": "In read-write mode the agent posts directly to QuickBooks, journal entries included, with no gate beyond the AI client's own confirmations."
  },
  "agent_readiness": {
   "score": 3,
   "note": "A remote MCP listed in Claude's connector directory, OAuth through Intuit and a per-company read-only switch; no published tool catalogue or scoped tokens."
  },
  "integration_ease": {
   "score": 4,
   "note": "Free, no Meridian account or email needed, set up in under three minutes by the author's timing."
  },
  "market": {
   "score": 1,
   "note": "Accountants and small business owners on QuickBooks Online."
  }
 },
 "api": {
  "public": false,
  "style": "Hosted MCP server only; no separate developer API",
  "auth": "Sign in with Intuit, choose companies, connect your AI client; Pilot stores the QuickBooks credentials encrypted and never shares them with the agent",
  "limits": "One connector user per client company on the free tier; no usage or rate limits published (Intuit's own QuickBooks API limits apply underneath)",
  "cost": "Free. Intuit's metering of QuickBooks API reads (since July 2025) falls on Pilot as the app owner, not on the user.",
  "export": "Not applicable: the connector holds nothing. Whatever the agent reads it can save; the books remain in QuickBooks.",
  "notes": "Pilot says the connector processes QuickBooks data only to answer a request, does not keep a copy and does not log QuickBooks records. AI-training use is not addressed in its FAQ."
 },
 "pricing": {
  "model": "Free connector; paid Meridian platform for firms",
  "entry": "Free, with one user per client company",
  "top": "Meridian Starter and above (team sharing, review, close); prices not published",
  "notes": "Team sharing is described as a paid feature of Meridian Starter. The free connector is a funnel into Meridian."
 },
 "agents": {
  "mcp": "official",
  "ai_features": "The connector is the agent surface. Meridian, the paid platform, runs the month-end close (categorization, reconciliations, schedules, statements) with 52 automated checks per close, and surfaces unusual items to a reviewer.",
  "agent_writes": "Yes in read-write mode: it 'can read and write arbitrary transactions, including journal entries'; bulk invoices, intercompany entries across companies and cloning one company into another were shown.",
  "approval": "None in the connector. Access is chosen per company (read-only or read-write) and Pilot advises starting read-only; any confirmation before a write comes from the AI client. Meridian, the paid product, has reviewer approval before books go to the client.",
  "notes": "Compared by Pilot with Intuit's own Claude connector (reports only, effectively read-only) and Intuit's open-source server (full API, but needs a developer token, git, npm and hosting)."
 },
 "integration": {
  "scope": "QuickBooks Online only for the connector. Meridian itself supports QuickBooks Online and Xero, with NetSuite, Sage Intacct, QuickBooks Desktop and Rillet in development; Mercury, Brex and Ramp have direct bank sync.",
  "notes": "Works with several QuickBooks companies at once without configuration, which suits firms."
 },
 "preference": {
  "scale": "Pilot says it uses Meridian to close the books for 4,000 clients a month (August 2026) and has served 8,000+ businesses since 2017. No connector usage figures published.",
  "ratings": "No reviews of the connector found. Pilot on Trustpilot: 2.9 of 5 from 20 reviews (October 2026). G2 not checked.",
  "sentiment": "Pilot's Trustpilot reviews are split: praise for responsive support and onboarding, complaints about Plaid sync errors, unfinished work, tax delays and fees; one reviewer calls it a 'thin wrapper on top of quickbooks + plaid'."
 },
 "timeline": [
  {
   "date": "2017",
   "event": "Pilot starts bookkeeping for startups"
  },
  {
   "date": "2025-07-28",
   "event": "Intuit starts metering QuickBooks API reads (per the October 2026 landscape survey)"
  },
  {
   "date": "2026-08-11",
   "event": "Waseem Daher announces the free QuickBooks Connector by Meridian on Pilot's blog"
  }
 ],
 "relation": "neighbour",
 "relation_note": "A bookkeeping service giving away the agent connector to QuickBooks. It solves 'let my agent touch QuickBooks' with direct writes and no record of its own; ownpurse solves 'let my agent read everything and propose writes I approve', with the record on my machine.",
 "borrow": "The setup bar (three minutes, no account, no email); read-only or read-write chosen per company; PDF card statements as a first-class input; plain statements of what is and is not kept.",
 "avoid": "Read-write as a single switch with no approval or log of what the agent posted; a free hosted bridge whose costs and future depend on one vendor's funnel.",
 "sources": [
  {
   "title": "Pilot: QuickBooks Connector by Meridian (11 Aug 2026)",
   "url": "https://meridian.pilot.com/blog/quickbooks-mcp-server"
  },
  {
   "title": "Connector sign-up page",
   "url": "https://qbo-connector.meridian.pilot.com/"
  },
  {
   "title": "Connector FAQ",
   "url": "https://qbo-connector.meridian.pilot.com/faq"
  },
  {
   "title": "Meridian home page",
   "url": "https://meridian.pilot.com/"
  },
  {
   "title": "Meridian FAQ",
   "url": "https://meridian.pilot.com/faq"
  },
  {
   "title": "Meridian AI info",
   "url": "https://meridian.pilot.com/ai-info"
  },
  {
   "title": "Pilot about page",
   "url": "https://pilot.com/about"
  },
  {
   "title": "Pilot on Trustpilot",
   "url": "https://www.trustpilot.com/review/pilot.com"
  },
  {
   "title": "ownpurse landscape survey, October 2026",
   "url": "https://ownpurse.com/blog/a-run-on-your-data/"
  }
 ],
 "unverified": [
  "The 10 August 2026 launch date in the brief; Pilot's post is dated 11 August 2026",
  "Pilot's Series C date and $1.2B valuation (only in a link address on the about page)",
  "Pilot's co-founders other than Waseem Daher",
  "Meridian Starter and other Meridian prices",
  "Whether connector traffic may be used for AI training",
  "Number of connector users",
  "Exact tool names exposed by the server",
  "G2 and Reddit sentiment; the web search budget ran out before they could be checked"
 ],
 "researched": "2026-10-10"
}
---

## What it is

The [QuickBooks Connector by Meridian](https://meridian.pilot.com/blog/quickbooks-mcp-server) is a free hosted MCP server that connects Claude, Cowork, ChatGPT and Codex to QuickBooks Online. Pilot, the bookkeeping company, announced it on 11 August 2026 in a post by Waseem Daher. Meridian is Pilot's "AI operating system for accounting firms", the platform Pilot says it uses to close the books for 4,000 clients a month. The connector is the free piece of it.

## How it works

You [sign in](https://qbo-connector.meridian.pilot.com/), connect with Intuit, pick your QuickBooks companies and add the server to your AI client. Nothing to install, no developer token, no git or npm, "you don't need to be a Meridian customer or even give up your email." For each company you choose read-only or read-write, and you can reconnect later to change it. In read-write mode it "can read and write arbitrary transactions, including journal entries." The post shows bulk-creating invoices, importing and categorizing card transactions from PDF statements, filling a firm's reporting template, posting intercompany entries across several companies and cloning one company into another.

## Data and access

The [FAQ](https://qbo-connector.meridian.pilot.com/faq) says the connector processes QuickBooks data only to answer a request, does not keep a copy of your books and does not log QuickBooks records. It stores your QuickBooks credentials encrypted and never hands them to the agent. There is no developer API, no published tool list and no source. The free tier allows one connector user per client company; sharing with a team is a paid Meridian feature. Whether data passing through may be used for AI training is not addressed.

## Agents

This is a connector, not a bookkeeper: Pilot says it "does not provide shared access, workflow management, review, or an end-to-end close process." What it does well is close the gap Pilot names in Intuit's own offerings. Intuit's hosted Claude connector runs reports but cannot read the underlying transactions and is effectively read-only; Intuit's open-source server exposes the full API but needs a developer token and hosting. Meridian's sits between: full reach, no setup. There is no approval gate. A write happens when the agent calls it, checked only by whatever confirmation the AI client asks for, and the connector keeps no log of what was posted.

## Customers

The audience is accountants and small business owners on QuickBooks Online, and especially firms with many client companies. No usage figures for the connector are public. Pilot's own [Trustpilot](https://www.trustpilot.com/review/pilot.com) score is 2.9 of 5 from 20 reviews (October 2026), split between praise for support and onboarding and complaints about bank sync, unfinished work and fees.

## History

Pilot started in 2017 and has raised over $150M from Sequoia, Index, Stripe, Bezos Expeditions and Whale Rock ([about](https://pilot.com/about)). Meridian is Pilot turning its internal close tooling into a product for other firms; its [FAQ](https://meridian.pilot.com/faq) lists QuickBooks Online and Xero, with NetSuite, Sage Intacct, QuickBooks Desktop and Rillet in development. Giving away the connector buys distribution among accountants at a moment when Intuit meters API reads (since July 2025) and its own agent tools are limited.

## For ownpurse

The connector shows how low the bar now is for an agent to write to QuickBooks: three minutes and no email. That raises the stakes for the thing ownpurse adds. Meridian holds nothing, logs nothing and gates nothing; the owner ends with books an agent changed and no record of what it changed or why. ownpurse keeps the owner's own append-only record of what each source said, lets agents read it freely, and sends writes only as sealed packages a person approves. Borrow the setup bar, the per-company read-only switch and PDF statements as a normal input. Expect people to run both: Meridian to let an agent act on QuickBooks, ownpurse to keep and check the record. A future ownpurse QuickBooks connector will face the same Intuit metering Pilot absorbs here.
