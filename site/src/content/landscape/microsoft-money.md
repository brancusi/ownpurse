---
{
 "slug": "microsoft-money",
 "name": "Microsoft Money",
 "url": "https://www.microsoft.com",
 "segment": "personal-finance",
 "status": "shut",
 "founded": "1991 (version 1.0, October 1991)",
 "hq": "Redmond, Washington",
 "backing": "Microsoft Corporation (NASDAQ: MSFT). Development and sales ended 2009-2010; support and online services ended 31 January 2011.",
 "tagline": "Microsoft's desktop personal finance program, 1991 to 2011, which kept your books in a local file and downloaded statements from banks.",
 "scope": "Money kept household accounts, budgets, bills and investments in a local .mny file on Windows, and downloaded bank and brokerage statements online. The Home & Business edition added small-business features such as invoices and business reports. It was never a cloud service, had no developer API, and after 2011 the free Sunset edition can only import files by hand.",
 "domains": [
  "personal",
  "bank",
  "ledger",
  "invoicing"
 ],
 "customers_served": [
  "consumer",
  "freelancer",
  "smb"
 ],
 "spectrum": {
  "custody": {
   "score": 3,
   "note": "You hold the only copy on your own disk, but in a proprietary file only Money reads."
  },
  "locality": {
   "score": 4,
   "note": "Desktop software; the record never left your machine except for online downloads."
  },
  "history": {
   "score": 1,
   "note": "A register you edit in place; no versioned history or as-of view beyond backups."
  },
  "openness": {
   "score": 1,
   "note": "Closed and proprietary; OFX and QIF import, QIF export, no API."
  },
  "breadth": {
   "score": 2,
   "note": "Banking, investments, bills, budgets and, in Home & Business, light business features."
  },
  "autonomy": {
   "score": 1,
   "note": "Downloaded statements, but a person accepted and categorized each transaction."
  },
  "agent_readiness": {
   "score": 0,
   "note": "No API, no scripting surface; dead before agents."
  },
  "integration_ease": {
   "score": 1,
   "note": "Banks connected through OFX servers they had to build; owners can only import files today."
  },
  "market": {
   "score": 1,
   "note": "Consumers, with a Home & Business edition for very small businesses."
  }
 },
 "api": {
  "public": false,
  "style": "None. Data came in through OFX downloads from bank servers and OFX or QIF files.",
  "auth": "n/a",
  "limits": "n/a",
  "cost": "n/a; the Sunset edition is a free download",
  "export": "QIF export and reimport is the documented manual path; no full export of the .mny file to an open format.",
  "notes": "Community tools (for example PocketSense, a Google Sites project for automated downloads into the Sunset edition) keep it alive for some owners."
 },
 "pricing": {
  "model": "Boxed and downloadable desktop software, one-off purchase per yearly version; free Sunset edition from June 2010",
  "entry": "Free (Money Plus Sunset Deluxe, no online features)",
  "top": "Money Plus Home & Business (retail price not confirmed)",
  "notes": "Retail prices per version not confirmed. Microsoft stopped boxed retail versions in August 2008."
 },
 "agents": {
  "mcp": "n/a",
  "ai_features": "None.",
  "agent_writes": "n/a",
  "approval": "n/a",
  "notes": "Discontinued long before agents; an agent could only reach the data through a QIF export or by reading the proprietary file."
 },
 "integration": {
  "scope": "US banks, card issuers and brokerages through OFX direct connect while online services ran (count not confirmed); OFX and QIF file import since.",
  "notes": "Online services, including automatic statement downloads, bill payment and quotes, were removed from existing installs in January 2011. Activation servers for Money 2007 and later were switched off after 31 January 2011, blocking reinstalls of those versions; the Sunset edition needs no activation."
 },
 "preference": {
  "scale": "No user counts or market share confirmed here. Microsoft said in 2009 that 'demand for a comprehensive personal finance toolset has declined.'",
  "ratings": "None current (predates review sites).",
  "sentiment": "Owners valued the local file and offline control; a community still runs the Sunset edition and builds download helpers for it. The pain was being stranded: no online downloads after 2011, no migration path to an open format, and conversion to Quicken as the main way out [unverified detail]."
 },
 "timeline": [
  {
   "date": "1991-10",
   "event": "Money 1.0 released"
  },
  {
   "date": "1995-08-24",
   "event": "Money 95 released"
  },
  {
   "date": "1997-01",
   "event": "Microsoft, Intuit and CheckFree announce the OFX standard [unverified]"
  },
  {
   "date": "2005",
   "event": "Last update to UK, French and international editions (Money 2005)"
  },
  {
   "date": "2007-12-06",
   "event": "Money Plus, the final retail release"
  },
  {
   "date": "2008-08",
   "event": "Microsoft says there will be no 2009 version and stops boxed retail versions"
  },
  {
   "date": "2009-06-10",
   "event": "Microsoft announces it will stop developing and selling Money; support until 31 January 2011"
  },
  {
   "date": "2010-06-17",
   "event": "Money Plus Sunset announced: free, no activation, no online features, OFX and QIF import only"
  },
  {
   "date": "2011-01-31",
   "event": "Support and online services end; activation servers for Money 2007 and later go off"
  },
  {
   "date": "2012-08-01",
   "event": "An unrelated news app called Money ships with Windows 8"
  },
  {
   "date": "2020",
   "event": "Money in Excel launches for Microsoft 365 Personal and Family in the US"
  },
  {
   "date": "2023-06-30",
   "event": "Money in Excel ends; Microsoft points users to Tiller"
  }
 ],
 "relation": "cautionary-tale",
 "relation_note": "Microsoft Money is the proof that local is not enough. Owners held the file, yet were stranded when the vendor stopped the online services and the only program that could read the file stopped changing. ownpurse keeps the record local and also open, so its life does not depend on one program.",
 "borrow": "The record as a file on the owner's disk; standard file imports (OFX, QIF) as a fallback that outlives any online feed; a free final edition with no activation that keeps the data readable.",
 "avoid": "A proprietary file format only one program reads; activation servers that can lock owners out; online downloads as a single vendor service that can be switched off.",
 "sources": [
  {
   "title": "Wikipedia: Microsoft Money",
   "url": "https://en.wikipedia.org/wiki/Microsoft_Money"
  },
  {
   "title": "Microsoft Support: End of support for Money in Excel, Wolfram data types and partner offers",
   "url": "https://support.microsoft.com/en-us/office/end-of-support-for-money-in-excel-wolfram-data-types-and-partner-offers-43ceac07-794e-4015-af04-21bb4795a14a"
  },
  {
   "title": "Microsoft: Money Plus Sunset page (archived link via Wikipedia references)",
   "url": "https://www.microsoft.com/money/sunset.mspx"
  },
  {
   "title": "PocketSense: automated downloads for Microsoft Money Sunset Edition",
   "url": "https://sites.google.com/site/pocketsense/"
  },
  {
   "title": "Tiller Help: Transitioning from Money in Excel to Tiller (2 October 2025)",
   "url": "https://help.tiller.com/en/articles/6226887-transitioning-from-money-in-excel-to-tiller"
  }
 ],
 "unverified": [
  "Market share against Quicken, and user counts at any point.",
  "Retail prices of any version, including Home & Business.",
  "Microsoft's co-founding of OFX with Intuit and CheckFree in January 1997 (from memory; not fetched).",
  "The data provider behind Money in Excel (believed to be Plaid; Microsoft's notice does not name it).",
  "Quicken's Money-to-Quicken conversion as the main migration path (support page could not be fetched).",
  "Wikipedia conflicts on end of sales: 30 June 2009 in one passage, 18 March 2010 in another.",
  "Microsoft's primary pages on Money (sunset.mspx, KB articles) could not be fetched directly; dates come from Wikipedia's account of them.",
  "Web search budget was exhausted during this pass; findings rely on directly fetched pages."
 ],
 "researched": "2026-10-10"
}
---

## What it is

Microsoft Money was Microsoft's personal finance program for Windows, built to compete with Quicken. Version 1.0 shipped in October 1991, and the yearly versions ran through Money Plus, the final retail release, on 6 December 2007 ([Wikipedia](https://en.wikipedia.org/wiki/Microsoft_Money)). It kept household accounts, budgets, bills and investments; the Home & Business edition added features for very small businesses. It is discontinued.

## How it works

Money stored your books in a local file on your Windows machine. While Microsoft ran its online services, Money downloaded bank and brokerage statements through OFX, paid bills and fetched quotes. You accepted and categorized each downloaded transaction in a register. Corrections were edits in place; history meant your backups.

## Data and access

This is the part that matters. In August 2008 Microsoft said there would be no 2009 version. On 10 June 2009 it announced it would stop developing and selling Money, with support until 31 January 2011, saying "demand for a comprehensive personal finance toolset has declined." In January 2011 online services were removed from existing installations, and activation servers for Money 2007 and later were switched off, so those versions could no longer be reinstalled.

Microsoft did leave a way to keep reading the data. Money Plus Sunset, announced on 17 June 2010, is a free download of Money Plus Deluxe and Home & Business that needs no activation and no earlier install. It has no online features: no statement downloads, no bill payment, no quotes. You can still import OFX and QIF files from your bank by hand. The documented way out is QIF export and reimport. There is no API and no export of the whole file to an open format. Volunteers keep it going: [PocketSense](https://sites.google.com/site/pocketsense/) automates downloads into the Sunset edition.

Microsoft tried again with Money in Excel (2020 to 2023, US Microsoft 365 Personal and Family only), which connected accounts to a workbook. When it ended on 30 June 2023, [Microsoft's notice](https://support.microsoft.com/en-us/office/end-of-support-for-money-in-excel-wolfram-data-types-and-partner-offers-43ceac07-794e-4015-af04-21bb4795a14a) told users "your data is yours to keep and is all contained within your Excel workbook," and recommended Tiller as a replacement.

## Agents

None. Money predates agents and has no scripting surface or API. An agent today could reach Money data only through a QIF export or by decoding the proprietary file.

## Customers

Consumers and the smallest businesses. No user counts or market share are confirmed here. Owners liked the local file and offline control, and a community still runs the Sunset edition. The complaint is being stranded: feeds switched off, a format only one program reads, and a lot of manual work to move.

## History

Money ran for about twenty years. Microsoft stopped updating the UK, French and international editions after Money 2005, and the last Canadian edition was Money 2006. The end came in steps across 2008 to 2011, with the Sunset edition as a mercy release. Money is also a reminder that the big players co-wrote the plumbing: OFX, the format Money downloaded, is still how many banks export statements.

## For ownpurse

Money is the cautionary tale for local-first. It had the custody right: the owner held the only copy, on their own disk. It still stranded people, because three things belonged to Microsoft: the file format, the activation servers and the online downloads. ownpurse answers each one. The record is SQLite, readable by any SQLite tool without ownpurse. Nothing activates or phones home. Connectors are many, and file imports (OFX, QIF, CSV) are first-class, so losing a feed means falling back to a file, not losing the books.

What to borrow: the free, activation-free final edition that kept data readable, and standard file imports as the fallback that outlived every online service. What to avoid: a format only one program reads, and any dependency on a vendor's server to open your own books.
