---
{
 "slug": "ofx",
 "name": "OFX (Open Financial Exchange)",
 "url": "https://www.ofx.net",
 "segment": "bank-aggregator",
 "status": "archived",
 "founded": "1997 (announced 16 January 1997 by Microsoft, Intuit and CheckFree; version 1.0 on 14 February 1997)",
 "hq": "No headquarters; a standard, stewarded by the Financial Data Exchange (FDX) since 2019",
 "backing": "Created by Microsoft, Intuit and CheckFree; the OFX consortium joined FDX in 2019, which now manages the specification.",
 "tagline": "The 1997 file and protocol standard for bank, card and brokerage downloads, behind Quicken's QFX, Direct Connect and Web Connect, now frozen at version 2.3 under FDX.",
 "scope": "OFX defines statements (bank, card, investment), balances, transactions with a per-transaction id (FITID), transfers and bill pay, as SGML (1.x) or XML (2.x), either as a file the user downloads or a request-response protocol a client sends to the bank's OFX server (Direct Connect). It is not a service, an aggregator or a ledger. QFX is Intuit's proprietary variant with extra tags.",
 "domains": [
  "bank",
  "integration",
  "personal"
 ],
 "customers_served": [
  "consumer",
  "freelancer",
  "smb",
  "developers"
 ],
 "spectrum": {
  "custody": {
   "score": 3,
   "note": "A downloaded file sits on the owner's disk; the bank remains authoritative but the owner holds a copy outright."
  },
  "locality": {
   "score": 4,
   "note": "Files are read locally and Direct Connect goes from the owner's machine straight to the bank, with no intermediary."
  },
  "history": {
   "score": 1,
   "note": "Each download is a snapshot of a date range; nothing versions earlier statements, though FITID lets a reader dedupe."
  },
  "openness": {
   "score": 3,
   "note": "A published, widely implemented spec with open-source parsers such as ofxtools; QFX extensions and FDX membership terms are not fully open."
  },
  "breadth": {
   "score": 1,
   "note": "Bank, card and investment statements plus bill pay; one domain."
  },
  "autonomy": {
   "score": 1,
   "note": "Direct Connect can fetch unattended; Web Connect needs a person to download each file; nothing posts to books by itself."
  },
  "agent_readiness": {
   "score": 1,
   "note": "No API or agent interface beyond Direct Connect, but an agent can parse a downloaded file with standard libraries."
  },
  "integration_ease": {
   "score": 3,
   "note": "Files are easy to read and well understood; Direct Connect needs per-bank server details, enrolment and sometimes a fee."
  },
  "market": {
   "score": 1,
   "note": "Personal finance and small-business desktop software (Quicken, QuickBooks Desktop, GnuCash, Moneydance)."
  }
 },
 "api": {
  "public": true,
  "style": "File format (SGML 1.x, XML 2.x) and HTTPS request-response protocol (Direct Connect)",
  "auth": "Direct Connect: bank-issued user id and password or PIN sent to the bank's OFX server; files need no auth once downloaded",
  "limits": "Download date ranges set by each bank; Direct Connect availability varies by bank and many have dropped it (extent unverified)",
  "cost": "The format is free to read; some banks charge for Direct Connect (amounts unverified)",
  "export": "An OFX, QFX or QBO file is itself an export of a statement range",
  "notes": "Latest reference version 2.3, published 16 October 2020. The www.ofx.net domain served a parked lander when checked on 2026-10-10. Most Canadian, UK and Australian banks do not offer Direct Connect but many offer OFX or QIF file downloads."
 },
 "pricing": {
  "model": "Free open standard; bank-specific fees for Direct Connect in some cases",
  "entry": "Free",
  "top": "n/a",
  "notes": "Specification downloads now sit with FDX; exact licence terms not confirmed."
 },
 "agents": {
  "mcp": "n/a",
  "ai_features": "None; it is a format.",
  "agent_writes": "Direct Connect supports transfers and bill pay messages, so a client could initiate money movement where a bank allows it; files are read-only.",
  "approval": "None in the standard beyond bank authentication.",
  "notes": "An OFX file is one of the most agent-friendly bank sources: structured, local, with stable transaction ids."
 },
 "integration": {
  "scope": "Supported by Quicken, QuickBooks, GnuCash, Moneydance, hledger and beancount importers, ofxtools and many banks' download menus (US most widely).",
  "notes": "ofxtools (Python) handles OFX 1.6 and 2.03, SGML and XML, and QFX while ignoring Intuit's proprietary tags; its author has run more than 10 years of data from various institutions through it."
 },
 "preference": {
  "scale": "Many US banks support OFX import into personal finance software (Wikipedia); no current count found.",
  "ratings": "n/a (a standard)",
  "sentiment": "Not gathered from forums for this dossier. Known in practice as the most reliable structured download where offered, with bank-by-bank quirks."
 },
 "timeline": [
  {
   "date": "1997-01-16",
   "event": "Microsoft, Intuit and CheckFree announce OFX, merging Microsoft's OFC and Intuit's Open Exchange"
  },
  {
   "date": "1997-02-14",
   "event": "OFX 1.0 released (SGML)"
  },
  {
   "date": "2019",
   "event": "The OFX consortium joins FDX, which takes over the specification"
  },
  {
   "date": "2020-10-16",
   "event": "OFX 2.3 published, the latest reference version"
  },
  {
   "date": "2026-10-10",
   "event": "www.ofx.net serves a parked domain lander"
  }
 ],
 "relation": "source",
 "relation_note": "OFX files are a bank export ownpurse can read today on the owner's machine, giving the statement side Xero's API cannot supply, with FITIDs for identity and a ledger balance as a checkpoint.",
 "borrow": "FITID as the source id in the identity ladder; the statement's ledger balance with its as-of date as a balance fact to reconcile against; accepting both SGML and XML; storing the raw file in the drawer and reprojecting.",
 "avoid": "Relying on Direct Connect, which banks have been withdrawing; QFX's proprietary tags; assuming FITIDs are unique across accounts or never reused.",
 "sources": [
  {
   "title": "Wikipedia: Open Financial Exchange",
   "url": "https://en.wikipedia.org/wiki/Open_Financial_Exchange"
  },
  {
   "title": "ofxtools on GitHub",
   "url": "https://github.com/csingley/ofxtools"
  },
  {
   "title": "Financial Data Exchange (FDX)",
   "url": "https://financialdataexchange.org/"
  },
  {
   "title": "www.ofx.net (parked lander as of 2026-10-10)",
   "url": "https://www.ofx.net"
  }
 ],
 "unverified": [
  "Exact licence terms for the OFX specification under FDX",
  "Where FDX hosts the OFX specification downloads now",
  "How many banks still offer Direct Connect, and current fees",
  "Dates of versions 1.6, 2.0 and 2.03",
  "Month of the OFX consortium joining FDX in 2019 (Wikipedia only)",
  "Forum sentiment (search budget exhausted)"
 ],
 "researched": "2026-10-10"
}
---

## What it is

Open Financial Exchange is the standard most people meet as a .ofx, .qfx or .qbo file in their bank's download menu. Microsoft, Intuit and CheckFree [announced it on 16 January 1997](https://en.wikipedia.org/wiki/Open_Financial_Exchange), merging Microsoft's Open Financial Connectivity with Intuit's Open Exchange, and released version 1.0 on 14 February 1997. Versions 1.x use SGML; later versions use XML. The OFX consortium joined FDX in 2019, and the latest reference version, 2.3, was published on 16 October 2020. When checked on 2026-10-10, www.ofx.net served a parked domain page, a small sign of where the standard sits now.

## How it works

OFX defines statement responses for bank, card and investment accounts: each transaction has a type, date, amount, name, memo and a FITID, the institution's id for that transaction, and each statement carries a ledger balance as of a date. It works two ways. Direct Connect is a protocol: the client (Quicken, GnuCash, Moneydance) sends a signed-on request to the bank's OFX server and gets the response back, with no third party in between. Web Connect is the file route: the person signs in to the bank's site, downloads a QFX or OFX file and imports it. QFX is Intuit's variant with proprietary extensions.

What it does well: it is structured, local, and carries a stable per-transaction id and a balance checkpoint, which is more than most CSV exports give.

## Data and access

The format is free to read and has mature open-source parsers. [ofxtools](https://github.com/csingley/ofxtools) handles OFX 1.6 and 2.03, both SGML and XML, and QFX (ignoring Intuit's tags), and its author reports more than 10 years of data from various institutions run through it. Many US banks support OFX import; most Canadian, UK and Australian banks do not offer Direct Connect but many offer OFX or QIF downloads. Direct Connect availability and fees vary bank by bank, and many banks have moved downloads behind aggregators such as Intuit's own Express Web Connect (extent unverified).

## Agents

There is nothing agent-specific, and nothing needs to be. A downloaded OFX file is one of the easiest bank sources for an agent: parse it, key each line by account and FITID, read the ledger balance, compare.

## Customers

OFX's users are people and small businesses on desktop money software, and the developers of importers for hledger, beancount and GnuCash. No review ratings apply.

## History

OFX was the first serious attempt to standardise how banks hand data to software, a generation before open banking. Its successor in the US, the FDX API, is what Akoya and the CFPB's 1033 rule build on. OFX solved the format and left the access problem to each bank, and many banks let Direct Connect lapse, which pushed users toward scraping aggregators.

## For ownpurse

OFX is a source ownpurse can use now. Xero does not return bank statement lines through its public API, so the bank's own file is the independent witness. ownpurse should store each file by SHA-256 in the drawer, project lines with FITID as the source id (scoped to the account, since uniqueness is only per account and some banks reuse or change them), record the ledger balance and its date as a balance fact to check against Xero, and replay on reimport without changing anything. Direct Connect is worth supporting only where a bank still offers it; the file path is the dependable one.
