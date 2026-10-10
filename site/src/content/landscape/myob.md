---
{
 "slug": "myob",
 "name": "MYOB",
 "url": "https://www.myob.com",
 "segment": "incumbent-ledger",
 "status": "active",
 "founded": "1991 in its current form (roots in Teleware in the 1980s; Data-Tech Software predecessor)",
 "hq": "Cremorne, Melbourne, Australia",
 "backing": "Private, owned by KKR since 2019 (takeover approved April 2019, delisted from the ASX May 2019). Earlier: ASX-listed 1999, taken private by an Archer Capital consortium (January 2009), sold to Bain Capital (August 2011), relisted on the ASX (May 2015).",
 "tagline": "Australia and New Zealand's long-standing small business accounting and payroll vendor, now running cloud MYOB Business beside AccountRight and Acumatica-based ERP.",
 "scope": "Bookkeeping, invoicing, bank feeds, BAS and GST, and payroll with Single Touch Payroll, which is central for Australian employers. It sells sole-trader to mid-market products (Solo, MYOB Business Lite and Pro, AccountRight Plus and Premier, MYOB Acumatica, EXO, Greentree) and practice software for accountants. Outside Australia and New Zealand it has no market; it left other countries around 2008.",
 "domains": [
  "ledger",
  "payroll",
  "invoicing",
  "bank",
  "tax",
  "payments"
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
   "note": "MYOB holds the online company file; AccountRight historically also allowed a file on your own PC, but the cloud file is the norm and leaving is through exports."
  },
  "locality": {
   "score": 1,
   "note": "Cloud first; AccountRight still needs a Windows desktop for setup and some features and its API reaches desktop files on localhost, but local files are a legacy path."
  },
  "history": {
   "score": 1,
   "note": "Audit trail inside AccountRight; the API returns current state and changes are found by polling modified dates."
  },
  "openness": {
   "score": 1,
   "note": "Closed source; documented REST API, but MYOB's own developer page sells access in paid tiers (A$110-630 a month) and there are no webhooks."
  },
  "breadth": {
   "score": 3,
   "note": "Ledger, payroll, invoicing, bank feeds, BAS and payments in one vendor, plus inventory and ERP higher up."
  },
  "autonomy": {
   "score": 1,
   "note": "Smart Reconciliation, AI BAS (beta) and invoice reminders suggest and automate pieces; a person still posts and lodges."
  },
  "agent_readiness": {
   "score": 2,
   "note": "Public REST API with granular OAuth scopes since March 2025; no MCP server or agent docs found, AI work tied to Microsoft from April 2026."
  },
  "integration_ease": {
   "score": 1,
   "note": "Paid developer programme, two active keys per account, 20-minute tokens, a 2025 OAuth overhaul, 403 on throttling and no webhooks."
  },
  "market": {
   "score": 2,
   "note": "Small businesses and their accountants, with a mid-market tail through MYOB Acumatica, EXO and Greentree."
  }
 },
 "api": {
  "public": true,
  "style": "REST/JSON with OData query parameters ($filter, $orderby); MYOB Business API and AccountRight API (cloud files at arl2.api.myob.com, desktop files at localhost:8080); separate EXO and Acumatica APIs",
  "auth": "OAuth 2.0; access tokens expire in about 20 minutes; since March 2025 new keys use granular scopes (e.g. sme-sale, sme-contact) and only company file administrators can approve access; older AccountRight paths also use a company-file token header",
  "limits": "8 requests per second and 1,000,000 a day per API key (Apideck, 2026); throttling returns HTTP 403; 29-second request timeout; two active keys per developer account, a third deactivates one",
  "cost": "MYOB's Developer Program page lists Developer Access A$110, Developer Partner A$220 and Premium Developer Partner A$630 a month incl. GST, bundling API access, software licences, sandbox files and marketplace listing. Apideck says API access itself is free; the two are not reconciled.",
  "export": "In-app exports of reports and lists; AccountRight file backups; full reads through the API. A Trustpilot reviewer reports being unable to restore a backup after cancelling.",
  "notes": "No webhooks: changes must be found by polling with timestamp filters. Developer accounts must be approved."
 },
 "pricing": {
  "model": "Subscription per business (AUD), with heavy introductory discounts; payroll and inventory as add-ons on lower plans",
  "entry": "Solo A$99 a year (A$12 for the first year on offer); Business Lite A$315 a year",
  "top": "AccountRight Plus A$165/month listed; AccountRight Premier and MYOB Acumatica by quote",
  "notes": "From myob.com/au/pricing, October 2026: Business Pro A$70/month (A$21 intro), payroll A$3 per employee per month on Lite and Pro, unlimited inventory A$22/month; payroll pricing changes from 1 November 2026."
 },
 "agents": {
  "mcp": "none",
  "ai_features": "Rolled out 1 March 2026: AI BAS (beta), AI Business Insights, Smart Reconciliation, Smart Invoice Reminders. A five-year Microsoft partnership (8 April 2026) promises embedded agents for cash-flow forecasting and compliance, with the first joint features due later in 2026.",
  "agent_writes": "Only through the REST API with an OAuth key and the right scopes; no MYOB-provided agent interface found.",
  "approval": "None for agents. The company file administrator approves the OAuth connection once; individual writes are not gated.",
  "notes": "No official or notable community MCP server was found in this pass. Agent tooling reported for the Microsoft deal (Foundry, Copilot Studio, Agent 365) comes from one secondary source."
 },
 "integration": {
  "scope": "MYOB App Marketplace, bank feeds across Australian and New Zealand banks, practice tools for accountants, and coverage from unified APIs (Apideck, Codat, Railz and others).",
  "notes": "The hardest of the three to start with: pay to join, wait for approval, juggle two keys and short tokens, and poll for changes. For a single owner reading their own file, the developer-programme cost is the main barrier."
 },
 "preference": {
  "scale": "MYOB says it serves \"hundreds of thousands of organisations\" in Australia and New Zealand (myob.com/au/about, October 2026); no exact current count published. The 3.28 million figure in Microsoft deal coverage is the size of the ANZ SME market, not MYOB's customer base.",
  "ratings": "Trustpilot: 1.3/5 from 549 reviews, 226 in the last 12 months (October 2026). G2 and Capterra not retrieved in this pass.",
  "sentiment": "Reviewers call the software adequate for simple needs and praise payroll and BAS fit for Australia; complaints are about support (long phone waits, looping virtual assistants), hard cancellation, unexpected charges and losing access to data after cancelling."
 },
 "timeline": [
  {
   "date": "1980s",
   "event": "Roots in Teleware, the US developer of the original MYOB software"
  },
  {
   "date": "1991",
   "event": "MYOB begins operating in Australia (date MYOB uses for itself)"
  },
  {
   "date": "1999",
   "event": "Data-Tech renamed MYOB Limited and listed on the ASX"
  },
  {
   "date": "2008",
   "event": "MYOB withdraws from markets outside Australia and New Zealand; MYOB US sold to Acclivity"
  },
  {
   "date": "January 2009",
   "event": "Archer Capital consortium takes MYOB private"
  },
  {
   "date": "August 2011",
   "event": "Bain Capital buys MYOB from Archer"
  },
  {
   "date": "October 2012",
   "event": "AccountRight Live released"
  },
  {
   "date": "2013",
   "event": "Becomes the exclusive Acumatica partner for Australia and New Zealand; buys BankLink"
  },
  {
   "date": "May 2015",
   "event": "Relisted on the ASX as MYOB Group"
  },
  {
   "date": "May 2018",
   "event": "Proposed A$180 million purchase of Reckon's accounting group collapses"
  },
  {
   "date": "2019",
   "event": "KKR takeover; delisted from the ASX in May"
  },
  {
   "date": "March 2025",
   "event": "OAuth overhaul: granular scopes for new API keys, admin-only approval"
  },
  {
   "date": "1 March 2026",
   "event": "AI BAS, AI Business Insights, Smart Reconciliation and Smart Invoice Reminders roll out"
  },
  {
   "date": "8 April 2026",
   "event": "Five-year AI partnership with Microsoft announced"
  }
 ],
 "relation": "source",
 "relation_note": "A regional connector beside Xero. Many Australian and New Zealand owners and their accountants keep books or payroll in MYOB; ownpurse would read it into the same record as Xero so a person can see both or move between them.",
 "borrow": "The 2025 move to granular scopes with only the file administrator able to approve a connection is a sound pattern for owner consent. MYOB's API can reach a desktop file on localhost, which is close to ownpurse's local-first stance.",
 "avoid": "Charging developers a monthly fee for API access, which keeps a single owner from reading their own books; no webhooks; and losing access to your own data when a subscription ends.",
 "sources": [
  {
   "title": "MYOB Developer Program details",
   "url": "https://developer.myob.com/developer-program-details/"
  },
  {
   "title": "MYOB API overview",
   "url": "https://developer.myob.com/api/myob-business-api/api-overview/"
  },
  {
   "title": "MYOB pricing (Australia)",
   "url": "https://www.myob.com/au/pricing"
  },
  {
   "title": "About MYOB",
   "url": "https://www.myob.com/au/about"
  },
  {
   "title": "MYOB AccountRight",
   "url": "https://www.myob.com/au/accounting-software/accountright"
  },
  {
   "title": "How to integrate with the MYOB API (Apideck)",
   "url": "https://apideck.com/blog/how-to-integrate-with-the-myob-api"
  },
  {
   "title": "MYOB API connector overview (Apideck)",
   "url": "https://developers.apideck.com/apis/accounting/myob/overview"
  },
  {
   "title": "MYOB and Microsoft strike five-year ANZ AI pact (TechInformed, 10 Apr 2026)",
   "url": "https://techinformed.com/myob-microsoft-strike-five-year-anz-ai-pact/"
  },
  {
   "title": "MYOB (Wikipedia)",
   "url": "https://en.wikipedia.org/wiki/MYOB_(company)"
  },
  {
   "title": "KKR completes MYOB acquisition (Accountants Daily)",
   "url": "https://www.accountantsdaily.com.au/mergers-and-acquisitions/13004-kkr-completes-myob-acquisition"
  },
  {
   "title": "MYOB evolves ERP platform to MYOB Acumatica (iTWire)",
   "url": "https://itwire.com/business-it-news/accounting-software/myob-continues-erp-platform-evolution-with-change-to-myob-acumatica.html"
  },
  {
   "title": "MYOB on Trustpilot",
   "url": "https://www.trustpilot.com/review/myob.com"
  }
 ],
 "unverified": [
  "Whether MYOB API access is free (Apideck) or requires the paid Developer Program (MYOB's page lists A$110-630 a month)",
  "Rate limits of 8 requests per second and 1,000,000 a day (Apideck, not MYOB's docs)",
  "Whether AccountRight still supports locally stored company files in 2026",
  "KKR purchase price (widely reported near A$2 billion; not confirmed in this pass)",
  "Exact MYOB customer count (only 'hundreds of thousands' published)",
  "Microsoft deal agent tooling (Foundry, Copilot Studio, Agent 365) from a single secondary source",
  "Status of Acumatica ownership (one aggregator says Vista Equity Partners acquired it)",
  "G2 and Capterra ratings not retrieved; web search budget ran out mid-research"
 ],
 "researched": "2026-10-10"
}
---

## What it is

MYOB ("Mind Your Own Business") is Australia and New Zealand's long-standing small business accounting vendor. It sells everything from sole-trader apps up to ERP: Solo, MYOB Business Lite and Pro, AccountRight Plus and Premier, and, higher up, MYOB Acumatica, EXO and Greentree. It also sells practice software to accountants. Its strength at home is payroll and compliance: Single Touch Payroll, BAS and GST. KKR has owned it privately since 2019. It is Xero's main local rival.

## How it works

Prices on [MYOB's pricing page](https://www.myob.com/au/pricing) in October 2026, in Australian dollars: Solo A$99 a year, Business Lite A$315 a year, Business Pro A$70 a month and AccountRight Plus A$165 a month. All of them are discounted heavily for the first months. Payroll is A$3 per employee per month on Lite and Pro. MYOB Business is the cloud product. AccountRight is the older desktop-and-web product, and it still needs a Windows computer for setup and some features.

## Data and access

The API is REST over JSON with OData filters. Cloud files live at arl2.api.myob.com; desktop files are reached on localhost. In March 2025 MYOB [reworked OAuth](https://apideck.com/blog/how-to-integrate-with-the-myob-api): new keys get granular scopes, and only a company file administrator can approve a connection. Access tokens last about 20 minutes. The limits are generous, at 8 requests a second and a million a day per key, but throttling returns 403 rather than 429. Each developer account can hold only two active keys, and there are no webhooks, so you poll by modified date.

The cost is the barrier. MYOB's [Developer Program page](https://developer.myob.com/developer-program-details/) lists paid tiers at A$110, A$220 and A$630 a month including GST, bundling API access, software licences and a marketplace listing. Apideck describes API access as free. The two have not been reconciled. Either way, for one owner reading their own file, the paid programme is a poor fit.

## Agents

No MYOB MCP server or agent documentation turned up. On 1 March 2026 MYOB rolled out AI BAS (in beta), AI Business Insights, Smart Reconciliation and Smart Invoice Reminders. On 8 April 2026 it signed a [five-year AI partnership with Microsoft](https://techinformed.com/myob-microsoft-strike-five-year-anz-ai-pact/) to build embedded agents for cash-flow forecasting and compliance, with the first joint features due later in 2026. For now an agent reaches MYOB only through the REST API, and nothing gates individual writes.

## Customers

MYOB says it serves ["hundreds of thousands of organisations"](https://www.myob.com/au/about). The 3.28 million figure in press coverage is the size of the Australian and New Zealand SME market, not MYOB's customer count. [Trustpilot](https://www.trustpilot.com/review/myob.com) is 1.3 from 549 reviews. The complaints are about support, cancellation, surprise charges, and one reviewer who could not restore a backup after cancelling. The software itself is often called adequate.

## History

MYOB's roots are in Teleware's 1980s software. It listed on the ASX in 1999 and left markets outside Australia and New Zealand around 2008. It was taken private by Archer Capital in 2009 and sold to Bain Capital in 2011. It relisted in 2015 and was bought by KKR in 2019 ([Wikipedia](https://en.wikipedia.org/wiki/MYOB_(company))). Since 2013 it has been Acumatica's exclusive partner in Australia and New Zealand. Through all of this it went from desktop to cloud while Xero, born in the cloud, took much of the market.

## For ownpurse

MYOB is the natural second connector for Australian and New Zealand owners. Many of them run payroll or older books in MYOB and everything else in Xero, and an accountant often sees both. A connector would poll by modified date, refresh tokens every 20 minutes and treat 403 as backoff. It would also have to settle the developer-programme cost before promising it to individual owners. Data you lose access to when you cancel is exactly the problem ownpurse exists to solve.
