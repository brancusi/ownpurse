---
{
 "slug": "dext",
 "name": "Dext",
 "url": "https://dext.com",
 "segment": "capture",
 "status": "acquired",
 "founded": "2010 (as Receipt Farm, then Receipt Bank; renamed Dext in February 2021)",
 "hq": "London, United Kingdom",
 "backing": "Venture-backed as Receipt Bank: Series A 2016; US$50M Series B led by Insight Venture Partners, July 2017 (US$65M raised to that point). Acquired by IRIS Software Group: intent announced 3 December 2024, completed 23 December 2024, price not disclosed. Now 'part of IRIS Software Group'.",
 "tagline": "Dext captures receipts, bills and statements, extracts the data and publishes it into an accounting system such as Xero or QuickBooks Online.",
 "scope": "Dext reads documents (receipts, purchase and sales invoices, bank and supplier statements) sent by mobile app, email, WhatsApp, upload or supplier fetch, extracts and categorises them, and publishes them to more than 36 accounting platforms after review or by rule. It adds expense claims, mileage, e-commerce sales summaries (Commerce Lite), practice dashboards and an AI Assist agent; payments are announced but not yet live. It is not a ledger and keeps no general ledger of its own, and it has no public read or write API for documents.",
 "domains": [
  "documents",
  "spend",
  "commerce",
  "integration"
 ],
 "customers_served": [
  "freelancer",
  "smb",
  "accountants"
 ],
 "spectrum": {
  "custody": {
   "score": 1,
   "note": "Dext holds the documents in its cloud (Vault, kept at least ten years); a two-way sync to Dropbox, Google Drive or OneDrive gives the owner a copy, and the ledger holds the published result."
  },
  "locality": {
   "score": 0,
   "note": "Cloud only; no self-hosted or local option."
  },
  "history": {
   "score": 1,
   "note": "Originals are kept and approvals are recorded on items, but there is no versioned, as-of history of extracted data exposed to the owner."
  },
  "openness": {
   "score": 1,
   "note": "Closed source; the only documented API is read-only practice metrics, the document ingest API is partner-gated, and export is through the app and folder sync."
  },
  "breadth": {
   "score": 2,
   "note": "Costs, sales, bank and supplier statements, expenses, mileage and e-commerce summaries, across 36+ ledgers; no ledger, payroll or bank feed of its own."
  },
  "autonomy": {
   "score": 2,
   "note": "Supplier rules can auto-publish and AI Assist guidance can be set to auto-apply, but the default is review and 'nothing happens without your say-so'."
  },
  "agent_readiness": {
   "score": 1,
   "note": "No MCP, no CLI, no public document API; a read-only metrics API with practice-scoped tokens at 60 requests a minute is the only documented surface."
  },
  "integration_ease": {
   "score": 1,
   "note": "Connecting a ledger is self-serve for an owner, but a third party has to go through Dext's partner programme for anything beyond metrics."
  },
  "market": {
   "score": 2,
   "note": "Small businesses and the accounting and bookkeeping firms that serve them; bespoke pricing past five entities."
  }
 },
 "api": {
  "public": false,
  "style": "REST (Data Health and Insights API, read-only); partner-only OCR ingest API",
  "auth": "Practice-scoped bearer tokens for the metrics API [third-party report]; ingest API credentials through the partner programme",
  "limits": "60 requests per minute on the metrics API, with 429 responses and rate-limit headers [third-party report, not confirmed on a Dext page]",
  "cost": "Data health and insights add-on US$7.50 per client per month on firm plans; ingest API pricing only through partner contact",
  "export": "Documents sync two ways to Dropbox, Google Drive or OneDrive from Vault; published items carry the image into the ledger as an attachment. Bulk CSV export of extracted data not confirmed from a primary source.",
  "notes": "No developer link on dext.com; the developer-api URLs that third parties cite return 404 as of 2026-10-10. The practical route to Dext data for ownpurse is the ledger it publishes into (bills with attachments) or the synced cloud folder of originals."
 },
 "pricing": {
  "model": "Subscription by monthly document volume for businesses; per client per month for firms (10-client minimum); metered add-ons and credits",
  "entry": "Business plan from US$25.21 a month (US$302.50 billed annually), 250 documents a month, 5 users, up to 5 entities (dext.com, 2026-10-10)",
  "top": "Higher volumes on a slider with prices not published; more than 5 entities is bespoke; firm Essentials and Advanced plans priced per client, amounts not in the page text",
  "notes": "Add-ons: AI Assist US$10.50 a month for businesses (until 31 January 2027) or US$7 per client for custom guidance on firm plans (core guidance free); line-item and supplier-statement extraction from US$20.50 a month or US$0.50 a document; bank statement extraction from US$13 a month or US$0.32 a document; Commerce Lite from US$7.50; Vault free to 100 MB. Over the limit, documents upload but are not extracted until the next bill date. 14-day trial, no card."
 },
 "agents": {
  "mcp": "none",
  "ai_features": "Extraction claimed at over 99% (99.5% in IRIS's release), trained on more than 1 billion documents; AI Assist, described as a bookkeeping agent, learns from user decisions and proposes categorisation, anomaly and clean-up guidance in the Costs and Sales spaces (Bank space planned).",
  "agent_writes": "No outside agent can write through a public API. Dext's own AI Assist applies guidance inside Dext; publishing to the ledger is by button or by rule.",
  "approval": "Review before publish is the default ('Nothing publishes without your review'); suggestions can be reviewed, refined, rejected, or switched to auto-apply per client or practice-wide; approval history kept on invoices for the coming Payments feature.",
  "notes": "Dext says AI Assist data is not used to train external models and stays in EU/UK data regions. A third-party API report card (Supergood) found no official or community MCP server."
 },
 "integration": {
  "scope": "36+ accounting platforms (Xero, QuickBooks Online, Sage, Zoho named), e-commerce platforms, and 11,500+ banks, suppliers and marketplaces for capture; Dropbox, Google Drive, OneDrive for Vault sync; planned single sign-on and data sharing with IRIS Elements.",
  "notes": "Ledger-agnostic by stated strategy, which IRIS committed to keep. Connecting pulls the chart of accounts, suppliers, customers, tracking categories and payment methods from the ledger."
 },
 "preference": {
  "scale": "700,000+ businesses and 12,000+ accounting and bookkeeping firms (dext.com, 2026-10-10; IRIS release, December 2024); 1.44 billion receipts and invoices scanned and 430 staff (Accounting Today, May 2023); more than 35 million hours of data entry saved (IRIS, December 2024).",
  "ratings": "Trustpilot 4.8 from 1,983 reviews, 188 in the last 12 months (2026-10-10; Dext has not recently invited reviews). Dext's own site cites G2 4.6, Xero App Store 4.8, QuickBooks 4.4, Apple App Store 4.8, Google Play 4.8 without counts.",
  "sentiment": "Praised for ease of capture, time saved on invoice and statement processing, and fast, friendly support. Complaints are occasional login blocks and app glitches; comparisons note that line-item and statement extraction are metered credits on top of the plan."
 },
 "timeline": [
  {
   "date": "2010",
   "event": "Founded in London as Receipt Farm, soon renamed Receipt Bank"
  },
  {
   "date": "2016",
   "event": "Series A round; headcount doubles"
  },
  {
   "date": "2017-07-20",
   "event": "US$50M Series B led by Insight Venture Partners, US$65M raised to date"
  },
  {
   "date": "2021-02",
   "event": "Receipt Bank renames itself Dext after acquiring Xavier; Receipt Bank becomes Dext Prepare"
  },
  {
   "date": "2022-10",
   "event": "Sabby Gill becomes CEO"
  },
  {
   "date": "2023-05",
   "event": "430 staff, 1.44 billion documents scanned, Prepare, Precision and Commerce products"
  },
  {
   "date": "2024-12-03",
   "event": "IRIS Software Group announces intent to acquire Dext"
  },
  {
   "date": "2024-12-23",
   "event": "IRIS completes the acquisition; price not disclosed"
  },
  {
   "date": "2026",
   "event": "AI Assist bookkeeping agent in beta and paid add-on; Payments marked coming soon"
  }
 ],
 "relation": "source",
 "relation_note": "Dext is a document source for ownpurse, not a rival. Its output reaches ownpurse as bills and receipts with attachments in the ledger, and its originals can reach a folder the owner holds through Vault sync.",
 "borrow": "The review-then-publish default, supplier rules that a person sets and can see, and suggestions an accountant can accept, refine or reject before they apply. Metering the expensive extractions (line items, statements) separately is an honest cost model.",
 "avoid": "Gating the data API behind a partner programme, so the owner cannot read their own extracted data programmatically; and letting auto-publish quietly become the norm.",
 "sources": [
  {
   "title": "Dext home page (figures, ratings, pricing example)",
   "url": "https://dext.com/en"
  },
  {
   "title": "Dext business pricing",
   "url": "https://dext.com/en/business/pricing"
  },
  {
   "title": "Dext partner pricing",
   "url": "https://dext.com/en/partner/pricing"
  },
  {
   "title": "Dext integrations",
   "url": "https://dext.com/en/business/product/integrate-with-accounting-software"
  },
  {
   "title": "Dext AI Assist for firms",
   "url": "https://dext.com/en/partner/product/dext-ai-assist-bookkeeping-agent"
  },
  {
   "title": "Dext Vault",
   "url": "https://dext.com/en/business/product/dext-vault"
  },
  {
   "title": "Dext Payments (coming soon)",
   "url": "https://dext.com/en/business/product/dext-payments"
  },
  {
   "title": "Dext our company (leadership)",
   "url": "https://dext.com/en/our-company"
  },
  {
   "title": "Dext Global Product Update 2026 (part of IRIS)",
   "url": "https://info.dext.com/global-partner-updates-na-2026"
  },
  {
   "title": "IRIS announces intent to acquire Dext, 3 December 2024",
   "url": "https://www.iris.co.uk/news/iris-announces-intent-to-acquire-dext/"
  },
  {
   "title": "IRIS completes Dext acquisition, 23 December 2024",
   "url": "https://www.iris.co.uk/news/iris-completes-dext-acquisition/"
  },
  {
   "title": "Accounting Today, Vendor spotlight: Dext, May 2023",
   "url": "https://accountingtoday.com/news/vendor-spotlight-dext"
  },
  {
   "title": "CB Insights, Receipt Bank",
   "url": "https://www.cbinsights.com/company/receipt-bank"
  },
  {
   "title": "Supergood API report card: Dext (third party)",
   "url": "https://supergood.ai/api-report-card/dext"
  },
  {
   "title": "Trustpilot: Dext",
   "url": "https://www.trustpilot.com/review/dext.com"
  },
  {
   "title": "Eightx, Dext vs Hubdoc 2026 (competitor comparison)",
   "url": "https://eightx.co/blog/dext-vs-hubdoc-receipt-capture"
  }
 ],
 "unverified": [
  "Data Health and Insights API details (bearer tokens, 60 requests a minute) and the partner-only OCR ingest API come from a third-party report card; no Dext developer page could be reached on 2026-10-10.",
  "Total funding: third parties give US$133M or US$138M; a 2019 round (reported around US$73M) was not confirmed from a primary source.",
  "Dates of the Xavier and Greenback acquisitions.",
  "Whether bulk CSV or Excel export of extracted data is available to business users.",
  "G2, Capterra and app store rating counts (only Dext's own uncounted figures were found; G2 and Capterra blocked fetches).",
  "That the data health and insights add-on is the former Dext Precision product.",
  "Co-founders named by secondary sources as Michael Wood and Alexis Prenn."
 ],
 "researched": "2026-10-10"
}
---

## What it is

Dext is the largest independent receipt and bill capture service for small businesses and the firms that keep their books. It began in London in 2010 as Receipt Farm, became Receipt Bank, and [renamed itself Dext in 2021](https://accountingtoday.com/news/vendor-spotlight-dext) after buying the data specialist Xavier. Since [23 December 2024](https://www.iris.co.uk/news/iris-completes-dext-acquisition/) it has belonged to IRIS Software Group, which did not disclose the price.

It does one job well: turn a photo, an email or a PDF into a categorised cost or sale and publish it to your accounting system. It is not a ledger.

## How it works

You send documents by the mobile app, email, WhatsApp, upload or supplier fetch. Dext extracts supplier, date, amounts and tax (it claims over 99% accuracy, 99.5% in IRIS's release), applies supplier rules, flags duplicates and matches against bank lines. Connecting a ledger pulls the chart of accounts, suppliers, customers and tracking categories. Then you press Publish, or let a rule do it. [Dext's integrations page](https://dext.com/en/business/product/integrate-with-accounting-software) says "nothing publishes without your review", and it works with 36+ ledgers including Xero, QuickBooks Online, Sage and Zoho.

Around that core sit expense claims and mileage, Commerce Lite for e-commerce sales, Vault for document storage (kept at least ten years, with two-way sync to Dropbox, Google Drive or OneDrive), practice dashboards, and AI Assist. Payments is [announced but marked coming soon](https://dext.com/en/business/product/dext-payments).

Pricing is published for businesses: [US$25.21 a month](https://dext.com/en/business/pricing) (US$302.50 a year) for 250 documents and 5 users. Line-item, supplier-statement and bank-statement extraction are metered add-ons, from US$0.32 to US$0.50 a document pay-as-you-go. Firms pay per client with a [10-client minimum](https://dext.com/en/partner/pricing).

## Data and access

This is where Dext is weakest for you. There is no developer link on its site. The one documented API is a read-only Data Health and Insights API with practice-scoped tokens and 60 requests a minute; the document ingest API is sales-qualified and partner-only, according to a [third-party report card](https://supergood.ai/api-report-card/dext). We could not confirm either on a Dext page as of 10 October 2026. Your own extracted data comes out through the app, the ledger it publishes into, or the folder sync of originals.

## Agents

There is no MCP server and no CLI. Dext's own [AI Assist](https://dext.com/en/partner/product/dext-ai-assist-bookkeeping-agent) is the agent: it learns from your decisions and proposes guidance on categorisation and anomalies. The page says "nothing happens without your say-so": you review, refine or reject each suggestion, or switch it to auto-apply for a client or the whole practice. Core guidance is free; custom guidance is US$7 per client a month for firms. Dext says the data stays in EU and UK regions and does not train outside models.

## Customers

Dext reports 700,000+ businesses and 12,000+ accounting and bookkeeping firms. On [Trustpilot](https://www.trustpilot.com/review/dext.com) it scores 4.8 from 1,983 reviews (10 October 2026). Reviewers praise how easy capture is, the time saved and the support. Complaints are occasional login and app problems. Comparisons note that the useful extras cost more on top of the plan.

## History

Receipt Bank raised a Series A in 2016 and a [US$50M Series B led by Insight](https://www.cbinsights.com/company/receipt-bank) in July 2017. It bought Xavier (later Dext Precision, now sold as the data health add-on) and Greenback (now Dext Commerce). Sabby Gill became CEO in October 2022. IRIS [announced the deal](https://www.iris.co.uk/news/iris-announces-intent-to-acquire-dext/) on 3 December 2024. It promised to keep Dext ledger-agnostic and to tie it to IRIS Elements. Dext is the capture industry consolidating into practice-software groups.

## For ownpurse

Dext is a source, not a competitor. Its work arrives in Xero as bills and receipts with the image attached, and ownpurse already reads those through the ledger and the attachments API. Fetching attachments costs Xero calls and egress. That makes the Vault folder sync worth a connector: a folder of originals the owner holds, which ownpurse can hash, link to the ledger entry, and keep as an independent witness. Borrow the review-then-publish default and the per-client approve, refine or reject flow. Do not copy the gated API: an owner should never need a partner agreement to read their own documents.
