---
{
 "slug": "booke",
 "name": "Booke AI",
 "url": "https://booke.ai",
 "segment": "agent-beside-ledger",
 "status": "active",
 "founded": "2020 (per third-party trackers; not confirmed by the company)",
 "hq": "Wilmington, Delaware, US (per trackers; registered address, team location not confirmed)",
 "backing": "Small and venture-light. Trackers report a pre-seed of about $300,000 in July 2021 (Alchemist Accelerator, CABRA VC, angels) and an undisclosed seed led by Insta Ventures on 7 June 2022; totals of $300,000 (Latka) to $330,000 (Caplight). No company announcement found.",
 "tagline": "A hosted AI bookkeeper that categorizes and reconciles bank transactions inside QuickBooks Online and Xero for small businesses and the firms that keep their books.",
 "scope": "Booke works on the bank feed already in QuickBooks Online, Xero or Zoho Books: it categorizes transactions, matches them to bills, invoices and receipts, asks clients for missing documents, flags anomalies before month-end and sends low-confidence items to a person. It reads OCR'd documents and runs a client query portal. It is not a ledger, keeps no owner-held copy of the books, and offers no public API or MCP server.",
 "domains": [
  "ledger",
  "bank",
  "documents",
  "close"
 ],
 "customers_served": [
  "smb",
  "accountants"
 ],
 "spectrum": {
  "custody": {
   "score": 1,
   "note": "The books stay in QuickBooks or Xero (another vendor); Booke keeps its own activity log and per-client model, exportable for 30 days after cancellation, then deleted."
  },
  "locality": {
   "score": 0,
   "note": "Hosted SaaS plus a browser extension; nothing runs on the owner's machine."
  },
  "history": {
   "score": 1,
   "note": "An Activities Journal logs each AI decision and its reasoning, but the ledger itself remains the provider's current state."
  },
  "openness": {
   "score": 1,
   "note": "Closed source, no public API; export only of your own data, with a 30-day window after cancellation."
  },
  "breadth": {
   "score": 2,
   "note": "Ledger, bank feed, documents, client queries and close checks, across three ledgers, but no payments, payroll or wider sources."
  },
  "autonomy": {
   "score": 3,
   "note": "Categorizes, matches and auto-approves high-confidence items in the books; only exceptions go to a person."
  },
  "agent_readiness": {
   "score": 0,
   "note": "No public API, CLI or MCP; it is an agent, but other agents cannot drive it."
  },
  "integration_ease": {
   "score": 2,
   "note": "Self-serve connection from the QuickBooks and Xero app stores; no way for a third party to build on it."
  },
  "market": {
   "score": 2,
   "note": "Small businesses and the bookkeeping firms that serve them, with a white-label firm plan."
  }
 },
 "api": {
  "public": false,
  "style": "None published. Consumes the QuickBooks Online, Xero and Zoho Books APIs; a browser extension shows statuses beside Xero bank lines.",
  "auth": "n/a (connects to ledgers by OAuth through their app stores)",
  "limits": "n/a; bound by the underlying ledger APIs' limits",
  "cost": "No API to buy. Booke itself absorbs the ledger API costs (Xero egress fees from March 2026, Intuit metered reads since 2025) inside its subscription.",
  "export": "Your data on request; the privacy policy gives a 30-day export window after cancellation, after which the per-client model and data are deleted.",
  "notes": "Security material (whitepaper, DPA, subprocessors) is on request from security@booke.ai; the trust page cites annual audits it describes as verified by Intuit and Xero (app-store security reviews, not a SOC 2 report)."
 },
 "pricing": {
  "model": "Per business per month, with a firm plan by quote",
  "entry": "$129 per business per month (AI Bookkeeper, Business), checked 2026-10-10",
  "top": "Accounting Firms plan: custom pricing, white-label, multi-client management",
  "notes": "Older directory listings show $20 to $29 per business per month, which suggests a move from a cheap tool to a priced 'AI bookkeeper'; the history of that change is not confirmed. Annual billing is offered but its price is not shown."
 },
 "agents": {
  "mcp": "none",
  "ai_features": "Per-client 'AI Brain' trained only on that client's history; bank transaction categorization; document matching; anomaly and duplicate detection ('Clean Up'); OCR; an email-to-task assistant; reasoning shown per transaction.",
  "agent_writes": "Yes. It codes and matches transactions in the provider's ledger, and an auto-approve setting lets it finish easy items without a person.",
  "approval": "A confidence threshold, not a package. Low-confidence items go to a review queue; one Booke page cites a 90% confidence cutoff, another just 'low-confidence'. Approved items are written directly; there is no sealed batch a person signs off before anything is sent.",
  "notes": "Booke says client data trains only that client's model, never a shared one; historical ledger data may be used for training for up to two years (privacy policy dated 2024-08-22)."
 },
 "integration": {
  "scope": "QuickBooks Online and Xero (core), Zoho Books (two-way). Uses the ledger's existing bank feeds; no Plaid or bank link of its own.",
  "notes": "Listed in the Xero App Store (about 4.5 to 4.6 stars from roughly 120 to 130 reviews, by region) and the QuickBooks App Store (Booke cites 4.6 from 117 ratings)."
 },
 "preference": {
  "scale": "Booke claims 10,000+ businesses and 'hundreds of accounting firms' in the US, UK, Canada and Australia (site, checked 2026-10-10). Unaudited.",
  "ratings": "Xero App Store US about 4.5 to 4.6 of 5 from 120 to 126 reviews; UK and Ireland about 4.5 from about 129 (2026-10-10). QuickBooks App Store 4.6 from 117 (as cited by Booke). G2: two negative reviews, profile inactive. No Capterra or Trustpilot profile found; no Reddit discussion found.",
  "sentiment": "Praised for time saved on bank feeds, auto-approve of easy items and the Clean Up view of duplicates and odd amounts. Complaints: demos not attended, and a warning to be careful with billing and plan upgrades."
 },
 "timeline": [
  {
   "date": "2020",
   "event": "Founded (per trackers), co-founders Vadim Chumak (CEO) and Denis Voloshin (CTO)"
  },
  {
   "date": "2021-07",
   "event": "Pre-seed of about $300,000 with Alchemist Accelerator and CABRA VC (per trackers)"
  },
  {
   "date": "2022-06-07",
   "event": "Seed led by Insta Ventures, amount undisclosed (per trackers)"
  },
  {
   "date": "2024-08-22",
   "event": "Current privacy policy: per-client models, 30-day export after cancellation"
  },
  {
   "date": "2026-10-10",
   "event": "Sold as 'AI Bookkeeper' at $129 per business per month; claims 10,000+ businesses"
  }
 ],
 "relation": "competitor",
 "relation_note": "The clearest small-business example of the pattern ownpurse rejects for writes: a hosted agent that writes into the provider's ledger behind a confidence threshold, keeps its own log, and leaves the owner no copy. It solves the same daily bank-feed chore ownpurse's agents would.",
 "borrow": "Per-client models trained on that client's own approvals; reasoning and the influential past transactions shown beside each suggestion; a 'clean up' view of duplicates and odd amounts; the missing-document request loop to the client; a 30-day export promise in writing.",
 "avoid": "Auto-approve that writes straight into the books; an activity log the owner cannot hold or verify; no API so other agents cannot see what it did; a data deletion clock that starts on cancellation.",
 "sources": [
  {
   "title": "Booke AI home page",
   "url": "https://booke.ai"
  },
  {
   "title": "Booke AI pricing",
   "url": "https://booke.ai/pricing"
  },
  {
   "title": "Booke AI for Xero",
   "url": "https://booke.ai/en-us/robotic-ai-bookkeeper/xero"
  },
  {
   "title": "Booke AI data handling",
   "url": "https://booke.ai/data-handling"
  },
  {
   "title": "Booke AI privacy policy",
   "url": "https://booke.ai/privacy-policy"
  },
  {
   "title": "Booke AI trust page",
   "url": "https://booke.ai/trust"
  },
  {
   "title": "Xero App Store reviews (UK)",
   "url": "https://apps.xero.com/uk/app/booke-ai/reviews"
  },
  {
   "title": "Xero App Store reviews (US)",
   "url": "https://apps.xero.com/us/app/booke-ai/reviews"
  },
  {
   "title": "G2 Booke AI reviews",
   "url": "https://www.g2.com/products/booke-ai/reviews"
  },
  {
   "title": "Latka: Booke AI",
   "url": "https://getlatka.com/companies/booke-ai"
  },
  {
   "title": "Caplight: Booke",
   "url": "https://www.caplight.com/company/booke"
  },
  {
   "title": "Crunchbase: Booke AI",
   "url": "https://crunchbase.com/organization/booke-ai"
  }
 ],
 "unverified": [
  "Founding year (2020), HQ (Wilmington, Delaware) and funding figures come only from trackers (Latka, Caplight, Crunchbase), not the company",
  "Seed round amount (undisclosed) and date (7 June 2022)",
  "The 10,000+ businesses and 'hundreds of firms' customer claims",
  "The 90% confidence cutoff (cited on one Booke page, not on the current Xero page) and whether it is configurable",
  "QuickBooks App Store rating of 4.6 from 117 (taken from Booke's own site; the app store page did not load)",
  "The earlier $20 to $29 per month pricing and when it changed",
  "What the 'annual security audits verified by Intuit and Xero' consist of",
  "No Capterra, Trustpilot or Reddit data found; web search budget ran out during this pass"
 ],
 "researched": "2026-10-10"
}
---

## What it is

Booke AI sells an 'AI bookkeeper' that works inside the books you already keep in [QuickBooks Online, Xero or Zoho Books](https://booke.ai). It takes the bank feed the ledger already has, categorizes each line, matches it to a bill, invoice or receipt, asks the client for anything missing, and sends what it is unsure of to a person. Its own copy says it 'actually does the work (not just suggestions)' and claims up to 80% of manual bank-feed work automated. Those are the vendor's figures.

It does that one job carefully. It is not a ledger, it does not hold a copy of your books for you, and it has no API.

## How it works

Each client gets what Booke calls an isolated 'AI Brain', trained only on that client's history, approvals and corrections. Booke says it does not train shared models on client data ([data handling](https://booke.ai/data-handling)). Suggestions show their reasoning and the past transactions that influenced them, and an Activities Journal logs what was done ([Xero page](https://booke.ai/en-us/robotic-ai-bookkeeper/xero)). A browser extension shows Booke's status beside each bank line in Xero.

The review rule is a confidence threshold. One Booke page cites human input below 90% confidence; the current Xero page just says low-confidence items go to your team. Xero App Store reviewers praise the auto-approve setting: 'the system handles the easy transactions on its own'. Approved items are written into the ledger directly.

## Data and access

There is no public API, CLI or export format to build on. Your books stay in the provider; Booke's log and model are Booke's. The [privacy policy](https://booke.ai/privacy-policy) (dated 22 August 2024) keeps transaction data for the life of the account, may use up to two years of history for training that client's model, and gives a 30-day window to export after cancellation before the model and data are deleted. Security documents are on request; the [trust page](https://booke.ai/trust) cites annual audits it describes as verified by Intuit and Xero, which reads as app-store security review rather than an independent attestation.

Pricing on 10 October 2026 is [$129 per business per month](https://booke.ai/pricing) for the Business plan, with a firm plan by quote that adds multi-client management and white-label. Directory listings from earlier show $20 to $29, so the price has moved up as the product moved from tool to 'bookkeeper'.

## Agents

Booke is an agent, but a closed one. No MCP server, no public API, no scoped tokens: another agent cannot ask it what it did or drive it. Writes go straight to the provider behind the confidence rule. Nothing is batched into a package a person approves before it is sent.

## Customers

Booke claims 10,000+ businesses and hundreds of accounting firms in the US, UK, Canada and Australia. On the [Xero App Store](https://apps.xero.com/uk/app/booke-ai/reviews) it rates about 4.5 of 5 from roughly 120 to 130 reviews depending on region; Booke cites 4.6 from 117 on the QuickBooks App Store. Praise is for time saved and the clean-up view; complaints are about missed demos and billing. Its [G2 profile](https://www.g2.com/products/booke-ai/reviews) has two negative reviews and is inactive. No Reddit discussion turned up.

## History

Third-party trackers put the founding in 2020, a pre-seed of about $300,000 in July 2021 with Alchemist Accelerator, and an undisclosed seed led by Insta Ventures in June 2022 ([Latka](https://getlatka.com/companies/booke-ai), [Caplight](https://www.caplight.com/company/booke)). There is no company announcement. It is a small, lightly funded company in a segment where larger rivals (Basis, Docyt, Truewind) raised far more.

## For ownpurse

Booke is the small-business version of the pattern ownpurse defines itself against on writes: an agent beside the ledger that writes directly behind a confidence threshold, with a log the owner cannot hold or verify. It also shows the demand is real. Firms pay $129 a month per client to have bank feeds coded. ownpurse can do the same job with a different trust model: the agent proposes, the categorizations wait uncollapsed in the record, and a person approves one sealed package. Worth borrowing: per-client learning from the owner's own approvals, reasoning shown with the past transactions that drove it, and the missing-receipt request loop.
