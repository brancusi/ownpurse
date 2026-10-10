---
{
 "slug": "quicken",
 "name": "Quicken",
 "url": "https://www.quicken.com",
 "segment": "personal-finance",
 "status": "active",
 "founded": "1983 (written at Intuit; first released 1984). Quicken Inc. independent since the 2016 sale by Intuit",
 "hq": "Menlo Park, California (3760 Haven Avenue); offices also in Tucson and Bangalore per the 2021 release",
 "backing": "Majority owned by Aquiline Capital Partners since the deal announced 9 September 2021 (terms undisclosed), bought from an affiliate of H.I.G. Capital, which bought Quicken from Intuit in 2016. The Quicken trademark is owned by Rocket Mortgage, LLC and used under license.",
 "tagline": "The longest-running consumer finance software: a desktop data file on your machine (Quicken Classic) and a cloud budgeting app (Simplifi).",
 "scope": "Quicken Classic (Windows and Mac) keeps a household's banking, investments, bills, rentals and small-business cash flow in a data file stored locally, fed by downloads from 14,000+ institutions. Simplifi and the cloud Business & Personal app do budgeting, projections and light invoicing in Quicken's cloud. LifeHub stores household records. It is not double-entry accounting, has no public API and no agent interface today.",
 "domains": [
  "personal",
  "bank",
  "reporting",
  "tax",
  "invoicing",
  "documents"
 ],
 "customers_served": [
  "consumer",
  "freelancer",
  "smb"
 ],
 "spectrum": {
  "custody": {
   "score": 3,
   "note": "Classic's data file lives on your disk and the Data Access Guarantee keeps view, edit and export after a lapse; but the file is proprietary and Simplifi data is cloud only."
  },
  "locality": {
   "score": 3,
   "note": "Classic runs on your own machine; downloads, quotes, mobile sync and support go through Quicken's servers and stop without a membership."
  },
  "history": {
   "score": 1,
   "note": "A register edited in place; history exists only as backup snapshots and year-end copies."
  },
  "openness": {
   "score": 1,
   "note": "Closed, no public API; export is report-level (Excel, CSV, clipboard) plus the proprietary QXF transfer file."
  },
  "breadth": {
   "score": 3,
   "note": "Banking, investments, bills, loans, rentals, business P&L, Schedule A-E tax reports and LifeHub documents in one product family."
  },
  "autonomy": {
   "score": 1,
   "note": "Rules and auto-categorized downloads; a person still does the reconciling and categorizing."
  },
  "agent_readiness": {
   "score": 0,
   "note": "No public API, CLI or MCP; a 2026 job post describes internal MCP services, not a public interface."
  },
  "integration_ease": {
   "score": 1,
   "note": "Data flows in from 14,000+ institutions, but no third party or owner can connect programmatically."
  },
  "market": {
   "score": 1,
   "note": "Consumers and households first, with sole proprietors and landlords through Business & Personal."
  }
 },
 "api": {
  "public": false,
  "style": "none public; bank downloads via Direct Connect and Web Connect (QFX), file transfer via QXF",
  "auth": "n/a (Quicken ID for the product)",
  "limits": "n/a",
  "cost": "No API offered; online services (downloads, quotes, mobile sync) are included only while a membership is paid.",
  "export": "Reports to Excel workbook, CSV or clipboard (Windows and Mac support articles); QXF (Quicken Transfer Format) between Quicken editions; full data file backups. No documented bulk export of every transaction in an open, complete format was found.",
  "notes": "Quicken for Mac moved to an SQLite-based file from Quicken 2015 for Mac (Wikipedia); Windows uses the proprietary QDF file. Neither is documented for third-party use."
 },
 "pricing": {
  "model": "Annual subscription per product; first-year promotional prices, auto-renewal at then-current rates",
  "entry": "Simplifi $3.99/month billed annually first year, $6.99/month regular (pricing page, 2026-10-10)",
  "top": "Classic Business & Personal $10.99/month billed annually first year, $12.49/month listed (pricing page, 2026-10-10); Quicken Family Enterprise unpriced on the page",
  "notes": "Classic Deluxe $6.99/month and Premier $7.99/month first-year promo; cloud Business & Personal $4.99 promo, $8.99 regular. Promos valid for new memberships ordered direct by 6 November 2026. Simplifi launched at $3.99/month or $39.99/year (January 2020). Desktop became a subscription from Quicken 2018."
 },
 "agents": {
  "mcp": "none",
  "ai_features": "Quicken AI in Simplifi answers questions about your finances and gives guidance (pricing comparison page, 2026); Quicken Assist in LifeHub answers estate and emergency-planning questions (November 2024).",
  "agent_writes": "No external agent can read or write; the in-app AI is advisory.",
  "approval": "n/a: no agent write path exists.",
  "notes": "An October 2026 job post for a staff AI engineer describes building conversational features, tool orchestration and 'MCP services with permissioned access to user data' for web, mobile and desktop teams, which suggests an agent surface is coming but says nothing about public access."
 },
 "integration": {
  "scope": "Inbound: 14,000+ financial institutions (Direct Connect, Web Connect/QFX, Quicken's aggregation); imports from older Quicken, Microsoft Money, Banktivity, Moneydance, CSV (Mac) and QuickBooks conversion. Outbound: reports, QXF, tax reports; Stripe payments for invoices in the cloud Business & Personal app.",
  "notes": "Integration runs one way, into Quicken. Bank connectivity is the most common complaint in reviews."
 },
 "preference": {
  "ratings": "Trustpilot 3.6 TrustScore from 84,606 reviews, 25,656 in the last 12 months (captured 2026-10-10). Apple App Store, the Simplifi app now listed as 'Quicken: Budget & Track Money': 4.5 from about 12K ratings (captured 2026-10-10). G2 and Capterra not retrieved.",
  "scale": "Over 2 million active users (Quicken, September 2021 release); 'over 20 million' customers across 40 years (October 2023 and November 2024 releases; the same releases also say 17 million or 20 million active, which conflict). 200K+ Simplifi users cited in homepage footnotes (2026). Revenue not disclosed.",
  "sentiment": "Praised for breadth across accounts and investments, tax-time reports and decades of continuity (many reviewers report 10 to 40 years of use). Complaints: rising renewal prices, broken bank downloads, updates that lose or scramble data, upsell contacts, the Mac edition lagging Windows, and in Simplifi balances that do not match the bank."
 },
 "timeline": [
  {
   "date": "1983",
   "event": "Quicken written at Intuit for MS-DOS and Apple II; first released 1984"
  },
  {
   "date": "1988",
   "event": "Quicken for Mac 1.0, rewritten with an incompatible file format"
  },
  {
   "date": "2009-11-02",
   "event": "Intuit completes its acquisition of Mint.com"
  },
  {
   "date": "2010-08-29",
   "event": "Quicken Online discontinued; users pointed to Mint, data could not transfer"
  },
  {
   "date": "2014-08",
   "event": "Quicken 2015 for Mac; later Mac versions use an SQLite database"
  },
  {
   "date": "2016",
   "event": "Intuit sells Quicken to H.I.G. Capital"
  },
  {
   "date": "2017-2018",
   "event": "Quicken 2018 makes the desktop product a subscription"
  },
  {
   "date": "2020-01-14",
   "event": "Simplifi launches at $3.99/month or $39.99/year"
  },
  {
   "date": "2021-09-09",
   "event": "Aquiline Capital Partners agrees to buy a majority stake from H.I.G.; over 2 million active users"
  },
  {
   "date": "2023-10-10",
   "event": "Forty-year rebrand: Quicken Classic, Business & Personal, Simplifi under the Quicken name"
  },
  {
   "date": "2024-11-25",
   "event": "Quicken LifeHub launches with the Quicken Assist AI"
  },
  {
   "date": "2026",
   "event": "Quicken AI offered in Simplifi; job post describes internal MCP services"
  }
 ],
 "relation": "neighbour",
 "relation_note": "Quicken is the forty-year proof that people want their money in a file on their own machine, and the cautionary half of that story: the file is proprietary, the bank feed is rented, and nothing outside Quicken can read it. ownpurse could later read Quicken's exports as a source; it does not compete for Quicken's budgeting user.",
 "borrow": "The Quicken Data Access Guarantee (your desktop data stays viewable, editable and exportable when you stop paying) as a public promise; one file holding household, rental and business cash flow together; a named transfer format (QXF) for moving between editions.",
 "avoid": "A proprietary file only the vendor's app can open; export limited to reports; online services that vanish when a subscription lapses; a register edited in place with no history; product lines (Quicken Online in 2010) shut down with data that could not move.",
 "sources": [
  {
   "title": "Quicken home page",
   "url": "https://www.quicken.com/"
  },
  {
   "title": "Quicken plans and pricing",
   "url": "https://www.quicken.com/products/pricing-comparison-products/"
  },
  {
   "title": "Quicken Classic pricing",
   "url": "https://www.quicken.com/products/pricing-comparison-classic/"
  },
  {
   "title": "Aquiline Capital Partners investment (press release, 2021-09-09)",
   "url": "https://www.quicken.com/about-us/press/aquiline-capital-partners-llc-investment/"
  },
  {
   "title": "Quicken celebrates forty years (press release, 2023-10-10)",
   "url": "https://www.quicken.com/about-us/press/quicken-celebrates-forty-years-in-personal-finance-management/"
  },
  {
   "title": "Introducing Simplifi (press release, 2020-01-14)",
   "url": "https://www.quicken.com/about-us/press/introducing-simplifi/"
  },
  {
   "title": "Introducing Quicken LifeHub (press release, 2024-11-25)",
   "url": "https://www.quicken.com/about-us/press/introducing-quicken-lifehub-organize-protect-share-essential-information/"
  },
  {
   "title": "Quicken welcomes new users (Mint switchers)",
   "url": "https://www.quicken.com/about-us/press/quicken-welcomes-new-users/"
  },
  {
   "title": "Quicken subscription membership FAQs (Data Access Guarantee)",
   "url": "https://www.quicken.com/support/quicken-subscription-membership-faqs/"
  },
  {
   "title": "Eric Dunn letter on the subscription plan",
   "url": "https://www.quicken.com/support/eric-dunn-ceo-quicken-letter-quicken-community-about-subscription-plan/"
  },
  {
   "title": "How to export Quicken reports to Excel and other formats",
   "url": "https://www.quicken.com/support/how-export-quicken-reports-other-program-formats-such-excel/"
  },
  {
   "title": "Managing a QXF file in Quicken for Mac",
   "url": "https://www.quicken.com/support/how-to-manage-a-quicken-transfer-format-qxf-file-in-quicken-for-mac/"
  },
  {
   "title": "Quicken job post: staff software engineer, AI",
   "url": "https://www.quicken.com/careers/staff-software-engineer-ai-1024.26/"
  },
  {
   "title": "Quicken on Trustpilot",
   "url": "https://www.trustpilot.com/review/www.quicken.com"
  },
  {
   "title": "Quicken: Budget & Track Money (Simplifi) on the App Store",
   "url": "https://apps.apple.com/us/app/quicken-simplifi-budget-smart/id1449777194"
  },
  {
   "title": "Quicken (Wikipedia)",
   "url": "https://en.wikipedia.org/wiki/Quicken"
  }
 ],
 "unverified": [
  "The 2016 sale price from Intuit to H.I.G. Capital and the exact closing date.",
  "The exact launch date of the Quicken 2018 subscription line (late 2017 is likely but not confirmed here).",
  "Mint's shutdown date (widely reported as March 2024) and the date of Quicken's Mint-switcher release.",
  "Which aggregator(s) Quicken uses behind its 14,000+ institution connections.",
  "Whether Quicken for Windows reads and writes QXF, and whether QIF export remains available in current Windows and Mac editions.",
  "Current active user counts and revenue; Quicken's own releases give conflicting customer figures (17 million vs over 20 million).",
  "G2 and Capterra ratings (Capterra returned 403; web search budget was exhausted during this pass).",
  "Whether Quicken AI is included in the cloud Business & Personal plan and in Classic, and its launch date.",
  "Quicken Family Enterprise and LifeHub prices."
 ],
 "researched": "2026-10-10"
}
---

## What it is

Quicken is the oldest consumer finance software still sold. It was written at Intuit in 1983, released in 1984, and sold by Intuit to H.I.G. Capital in 2016; [Aquiline Capital Partners](https://www.quicken.com/about-us/press/aquiline-capital-partners-llc-investment/) took a majority stake in September 2021. Today it is a family of products: Quicken Classic for Windows and Mac, which keeps your data in a file on your own computer; Simplifi, a cloud budgeting app launched in [January 2020](https://www.quicken.com/about-us/press/introducing-simplifi/); a cloud Business & Personal app for sole proprietors and landlords; and [LifeHub](https://www.quicken.com/about-us/press/introducing-quicken-lifehub-organize-protect-share-essential-information/) for household records.

## How it works

Classic pulls transactions, balances and quotes from more than 14,000 institutions into a local data file. You categorize, reconcile and run reports, including tax schedules and, in Business & Personal, business P&L and rental tools ([Classic pricing](https://www.quicken.com/products/pricing-comparison-classic/)). It is a single-entry register, not double-entry books. Since Quicken 2018 the desktop product is a subscription. When a membership lapses, Deluxe and higher keep full view, edit and export of the local file under the [Quicken Data Access Guarantee](https://www.quicken.com/support/quicken-subscription-membership-faqs/), but downloads, quotes, mobile sync and support stop; Starter becomes read-only. Simplifi and the cloud apps hold your data on Quicken's servers (Amazon Web Services, per the [CEO's subscription letter](https://www.quicken.com/support/eric-dunn-ceo-quicken-letter-quicken-community-about-subscription-plan/)).

Prices on 2026-10-10 ([products page](https://www.quicken.com/products/pricing-comparison-products/)): Simplifi $6.99 a month billed annually ($3.99 first-year promotion), cloud Business & Personal $8.99 ($4.99 promotion), and Classic Deluxe, Premier and Business & Personal at $6.99, $7.99 and $10.99 a month in the first year. Renewals are at then-current rates.

## Data and access

There is no public API. The Windows file (QDF) is proprietary; the Mac file has been SQLite-based since Quicken 2015 for Mac ([Wikipedia](https://en.wikipedia.org/wiki/Quicken)), but neither is documented for anyone else. You can [export reports](https://www.quicken.com/support/how-export-quicken-reports-other-program-formats-such-excel/) to Excel, CSV or the clipboard, and move data between editions with [QXF](https://www.quicken.com/support/how-to-manage-a-quicken-transfer-format-qxf-file-in-quicken-for-mac/). History is whatever your backups hold: transactions are edited in place.

## Agents

Simplifi now includes Quicken AI, which answers questions and gives guidance, and LifeHub has Quicken Assist. Neither lets an outside agent in. An October 2026 [job post](https://www.quicken.com/careers/staff-software-engineer-ai-1024.26/) describes building tool orchestration and "MCP services with permissioned access to user data", so an agent surface may be on the way; nothing public exists yet.

## Customers

Quicken reported over 2 million active users in 2021 and says over 20 million people have used it in 40 years (its releases give conflicting figures). On [Trustpilot](https://www.trustpilot.com/review/www.quicken.com) it scores 3.6 from 84,606 reviews; the Simplifi app scores 4.5 from about 12K ratings on the [App Store](https://apps.apple.com/us/app/quicken-simplifi-budget-smart/id1449777194) (both captured 2026-10-10). Long-time users value the breadth and the tax reports. The complaints repeat: renewal prices, bank downloads that fail, updates that lose data, upsells, and a Mac edition behind Windows.

## History

The history is a record of formats and services that did not carry data forward. Quicken for Mac 1.0 (1988) could not read earlier files. Intuit closed Quicken Online in August 2010 and its data could not move to Mint. The 2018 move to subscriptions tied bank feeds to a recurring fee. After the 2023 rebrand to Quicken Classic, Quicken courted former Mint users with Simplifi when Intuit wound Mint down.

## For ownpurse

Quicken shows two things. People will keep a finance file on their own machine for decades. And a file only one vendor's app can open is not really theirs. The Data Access Guarantee is worth matching, but in the structure rather than the terms: an open SQLite record any tool can read, with every version kept. Quicken is not a rival for its budgeting users. If it is anything to ownpurse, it is a possible source through its exports.
