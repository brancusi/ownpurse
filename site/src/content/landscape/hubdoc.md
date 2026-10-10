---
{
 "slug": "hubdoc",
 "name": "Hubdoc",
 "url": "https://www.hubdoc.com",
 "segment": "capture",
 "status": "acquired",
 "founded": "Toronto; founding year not confirmed from a primary source (Xero ecosystem partner since 2014)",
 "hq": "Toronto, Canada",
 "backing": "Acquired by Xero Limited (ASX: XRO): announced 31 July 2018 for US$70M in cash and equity (US$60M up front, 35% cash and 65% Xero shares; US$10M in equity deferred 18 months against targets). Xero's largest acquisition at the time.",
 "tagline": "Hubdoc is Xero's document capture app: it reads bills, receipts and statements and creates transactions in Xero or QuickBooks Online with the document attached.",
 "scope": "Hubdoc takes documents by mobile photo, email forwarding, upload or scan, extracts supplier, amount, invoice number and due date, and publishes a transaction with the source file attached to Xero or QuickBooks Online (plus Bill.com and Box). It once fetched statements automatically from 700+ institutions; standard fetch was retired in April 2022. It has no public API, its terms forbid scraping, and Xero is now building capture directly into Xero as Smart Document Capture.",
 "domains": [
  "documents"
 ],
 "customers_served": [
  "freelancer",
  "smb",
  "accountants"
 ],
 "spectrum": {
  "custody": {
   "score": 1,
   "note": "Hubdoc holds the documents in Xero's cloud and its terms make the owner responsible for their own backups; the copy that matters ends up as a Xero attachment."
  },
  "locality": {
   "score": 0,
   "note": "Cloud only."
  },
  "history": {
   "score": 1,
   "note": "Keeps the original documents; no versioned history of extracted data, and its terms say data may be deleted without notice."
  },
  "openness": {
   "score": 1,
   "note": "Closed, no API, scraping banned; only an in-app document download and backup."
  },
  "breadth": {
   "score": 1,
   "note": "Documents only, into two ledgers plus Bill.com and Box."
  },
  "autonomy": {
   "score": 2,
   "note": "Supplier rules can auto-publish to the ledger; reviewers report items publishing without review."
  },
  "agent_readiness": {
   "score": 0,
   "note": "No API, no MCP, and the acceptable use policy forbids automated agents that scrape or mine data."
  },
  "integration_ease": {
   "score": 1,
   "note": "Connecting to Xero or QBO is one click for an owner; a third party can reach Hubdoc's output only through the ledger's attachments."
  },
  "market": {
   "score": 1,
   "note": "Freelancers and the smallest Xero businesses, plus their bookkeepers."
  }
 },
 "api": {
  "public": false,
  "style": "None",
  "auth": "n/a",
  "limits": "n/a",
  "cost": "n/a; reading Hubdoc's output through Xero's Attachments API counts against Xero's daily call limit and, since 2 March 2026, Xero's egress usage fees",
  "export": "In-app 'backup and document download' (Hubdoc pricing page); format and scope of bulk export not documented on the pages reached",
  "notes": "Hubdoc's terms of use bar 'automated agents or scripts' that 'strip, scrape, or mine data' and building a competing product. Its documents reach ownpurse only as Xero (or QBO) transactions with attachments."
 },
 "pricing": {
  "model": "Included free with Xero business plans since 18 March 2020; standalone subscription otherwise",
  "entry": "Free with a Xero business plan; standalone US$12 a month after a 30-day trial, unlimited usage, multiple collaborators (hubdoc.com/pricing, 2026-10-10)",
  "top": "Same single plan; no tiers",
  "notes": "Xero's own 2026 pricing lists Smart Document Capture as included in Early (US$27), Growing (US$59) and Established (US$97) plans, and still links Hubdoc terms of use in its footnotes."
 },
 "agents": {
  "mcp": "none",
  "ai_features": "OCR extraction of supplier, amount, invoice number and due date; supplier rules. No agent features announced.",
  "agent_writes": "No; there is no programmatic interface, and automated access is prohibited by the terms.",
  "approval": "A person reviews and publishes by default; supplier rules can publish automatically.",
  "notes": "Xero's agent and AI work (including Smart Document Capture) is happening in Xero itself, not in Hubdoc; a Xero staff comment on its product ideas forum says new inbox workflows will not be built in Hubdoc."
 },
 "integration": {
  "scope": "Xero, QuickBooks Online, Bill.com, Box; direct fetch connections with Bank of America, Wells Fargo and Stripe remain; e-invoicing in Australia, New Zealand and Singapore.",
  "notes": "Hubdoc organisations included with a Xero plan are owned by the Xero subscriber, who keeps control even if disconnected from Xero (ownership terms, for organisations created from 18 March 2020)."
 },
 "preference": {
  "scale": "No current user count published. At acquisition (2018) it collected documents from 700+ financial institutions, utilities, telecoms and suppliers.",
  "ratings": "Trustpilot 2.1 from 18 reviews, 4 in the last 12 months, unclaimed profile (2026-10-10). Capterra and G2 not reached.",
  "sentiment": "Praised for its tight Xero link and for capture by email or photo, free with Xero. Complaints are about clunky usability, VAT errors that are hard to fix, duplicate Xero transactions, items auto-publishing without review, login problems, slow support, and a sense that development stalled after the acquisition and the end of fetch."
 },
 "timeline": [
  {
   "date": "2014",
   "event": "Becomes a Xero ecosystem partner"
  },
  {
   "date": "2018-07-31",
   "event": "Xero announces it will acquire Hubdoc for US$70M in cash and equity, its largest acquisition; Hubdoc to run standalone under founders Jamie McDonald and Jamie Shulman"
  },
  {
   "date": "2018-08",
   "event": "Acquisition completes"
  },
  {
   "date": "2020-03-18",
   "event": "Hubdoc included in Xero business plans; new ownership terms tie Hubdoc organisations to the Xero subscriber"
  },
  {
   "date": "2022-01-31",
   "event": "New standard fetch connections switched off"
  },
  {
   "date": "2022-04-27",
   "event": "Standard fetch connections retired; direct fetch with Bank of America, Wells Fargo and Stripe kept"
  },
  {
   "date": "2025-10-18",
   "event": "Xero updates its fetch-retirement post; email forwarding and uploads remain the main inputs"
  },
  {
   "date": "2026",
   "event": "Xero pricing lists Smart Document Capture in every plan; Hubdoc continues as a separate app"
  }
 ],
 "relation": "source",
 "relation_note": "Hubdoc is an indirect document source. ownpurse cannot talk to Hubdoc; its documents arrive as Xero bills, receipts and spend money with attachments, which the Xero connector reads.",
 "borrow": "The ownership terms: the Xero subscriber owns the Hubdoc organisation and keeps control even after disconnecting. Attaching the source document to the transaction it created is the right link between a document and an entry.",
 "avoid": "No API and a ban on automated access, so the owner's documents are reachable only through another vendor's metered API; terms that put backups on the customer and allow deletion without notice; and fetch built on screen-scraping supplier portals, which banks blocked until it was retired.",
 "sources": [
  {
   "title": "Hubdoc home page",
   "url": "https://www.hubdoc.com"
  },
  {
   "title": "Hubdoc pricing",
   "url": "https://www.hubdoc.com/pricing"
  },
  {
   "title": "Hubdoc add-ons",
   "url": "https://www.hubdoc.com/addons"
  },
  {
   "title": "Hubdoc terms of use",
   "url": "https://www.hubdoc.com/terms"
  },
  {
   "title": "Hubdoc ownership terms",
   "url": "https://www.hubdoc.com/ownershipterms"
  },
  {
   "title": "Hubdoc security FAQs",
   "url": "https://www.hubdoc.com/security-faqs"
  },
  {
   "title": "Xero blog: changes to how Hubdoc imports documents (updated 18 October 2025)",
   "url": "https://blog.xero.com/product-updates/hubdoc-importing-docs-changing/"
  },
  {
   "title": "Osler: Hubdoc in its acquisition by Xero",
   "url": "https://www.osler.com/en/expertise/deals-cases/hubdoc"
  },
  {
   "title": "NBR: Xero buys Hubdoc for up to US$70 million",
   "url": "https://www.nbr.co.nz/tech/xero-buys-hubdoc-for-up-to-us70-million"
  },
  {
   "title": "CPA Practice Advisor: Xero includes Hubdoc in Xero plans, March 2020",
   "url": "https://cpapracticeadvisor.com/2020/03/17/xero-includes-hubdoc-in-xero-plans-and-launches-xero-tax-uk-to-bring-more-value-to-global-customers/37265/"
  },
  {
   "title": "Xero US pricing plans (Smart Document Capture)",
   "url": "https://www.xero.com/us/pricing-plans/"
  },
  {
   "title": "Wikipedia: Xero (software), acquisitions",
   "url": "https://en.wikipedia.org/wiki/Xero_(software)"
  },
  {
   "title": "Trustpilot: Hubdoc",
   "url": "https://www.trustpilot.com/review/hubdoc.com"
  },
  {
   "title": "Eightx, Dext vs Hubdoc 2026 (competitor comparison)",
   "url": "https://eightx.co/blog/dext-vs-hubdoc-receipt-capture"
  }
 ],
 "unverified": [
  "Hubdoc's founding year (commonly given as 2011) and founders beyond the names in acquisition coverage (Jamie McDonald, Jamie Shulman).",
  "Whether Xero intends to retire Hubdoc in favour of Smart Document Capture; only a Xero forum comment and the redirected product URL point that way.",
  "The date and region availability of Xero's Smart Document Capture launch.",
  "Format and completeness of Hubdoc's bulk document download.",
  "Capterra, G2 and app store ratings and counts (fetches blocked).",
  "Current number of Hubdoc users.",
  "Whether QuickBooks Online publishing is still actively maintained (a competitor calls it legacy)."
 ],
 "researched": "2026-10-10"
}
---

## What it is

Hubdoc is the document capture app Xero has owned since 2018. It reads bills, receipts and statements and creates transactions in Xero or QuickBooks Online with the source file attached. Toronto-based and a Xero partner since 2014, it was bought for [US$70 million in cash and equity](https://www.osler.com/en/expertise/deals-cases/hubdoc), announced on 31 July 2018: US$60 million up front, 35% cash and 65% Xero shares, and US$10 million deferred against targets. That was Xero's largest acquisition at the time.

## How it works

You photograph a receipt, forward an email, upload a file or scan. [Hubdoc](https://www.hubdoc.com) extracts the supplier, amount, invoice number and due date. Supplier rules set the account and tax, and Hubdoc publishes a transaction to Xero or QuickBooks Online with the document attached. It also connects to Bill.com and Box.

Its best-known feature used to be fetch: logging in to banks, utilities and supplier portals to collect statements from more than 700 sources. Banks and suppliers increasingly blocked it. Xero [switched off new connections on 31 January 2022 and retired standard fetch on 27 April 2022](https://blog.xero.com/product-updates/hubdoc-importing-docs-changing/). Only direct connections with Bank of America, Wells Fargo and Stripe remain. Email forwarding and upload do most of the work now.

Since 18 March 2020 Hubdoc has been [included in Xero business plans](https://cpapracticeadvisor.com/2020/03/17/xero-includes-hubdoc-in-xero-plans-and-launches-xero-tax-uk-to-bring-more-value-to-global-customers/37265/). Standalone it costs [US$12 a month](https://www.hubdoc.com/pricing) with unlimited use. Meanwhile [Xero's 2026 pricing](https://www.xero.com/us/pricing-plans/) lists its own Smart Document Capture in every plan, and Xero's old Hubdoc product URL now shows that feature. A Xero staff comment on its ideas forum says new inbox workflows will be built in Xero, not Hubdoc. Hubdoc is not retired, but Xero is building capture into Xero itself.

## Data and access

Hubdoc has no API. Its [terms of use](https://www.hubdoc.com/terms) forbid "automated agents or scripts" that "strip, scrape, or mine data". They make you responsible for your own backups and allow deletion without notice. The pricing page promises "easy backup and document download" but documents no bulk format. The [ownership terms](https://www.hubdoc.com/ownershipterms) do one thing well. The Xero subscriber owns the Hubdoc organisation and keeps control even after disconnecting from Xero.

## Agents

None. No MCP, no CLI, nothing an agent may drive. Supplier rules can publish without review, and reviewers say this has caused duplicate transactions. Xero's AI work is happening in Xero.

## Customers

Hubdoc's natural users are freelancers and small Xero businesses who get it free, and the bookkeepers who serve them. Xero publishes no user count. [Trustpilot](https://www.trustpilot.com/review/hubdoc.com) shows 2.1 from only 18 reviews (10 October 2026), too few to be representative. People praise the Xero link and capture by email or photo. They complain about clunky screens, VAT errors that are hard to fix, duplicates, unwanted auto-publishing, login trouble and a product that stalled after fetch ended. A competitor's [June 2026 comparison](https://eightx.co/blog/dext-vs-hubdoc-receipt-capture) recommends it for Xero users with fewer than about 200 bills a month.

## History

Hubdoc is the clearest case in this survey of a scraper hitting the wall. Fetch, built on logging in as the user, was the product's promise and is now gone. The banks blocked it four years before Xero banned bots that simulate users in its own API terms. Hubdoc was bought by the ledger and is slowly being absorbed into it.

## For ownpurse

ownpurse cannot talk to Hubdoc, and should not try: the terms forbid it. Its documents reach the record as Xero transactions with attachments. The Xero connector reads them by `HasAttachments` and the Attachments API, and each fetch counts against the daily limit and Xero's egress fees. Store each attachment's hash with the entry it came from, so the document and the posting stay linked in the log. Borrow the ownership terms (the subscriber owns the organisation and keeps control). Avoid the rest: no API, backups left to the customer, and a feature built on pretending to be the user.
