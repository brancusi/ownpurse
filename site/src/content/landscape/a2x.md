---
{
 "slug": "a2x",
 "name": "A2X",
 "url": "https://www.a2xaccounting.com",
 "segment": "ipaas-middleware",
 "status": "active",
 "founded": "2014",
 "hq": "Toronto, Canada (A2X Software Limited, 2 Bloor Street E, per its Shopify listing); distributed team in 15 countries",
 "backing": "Funding and ownership are not disclosed on its site; incorporated in Canada as A2X Software Limited. Led by CEO Jamie McDonald, a co-founder of Hubdoc (acquired by Xero in 2018). Outside investors unverified.",
 "tagline": "Software that turns marketplace and payment payouts (Amazon, Shopify, eBay, Etsy, Walmart, TikTok Shop, PayPal) into summarized entries in Xero, QuickBooks Online, NetSuite or Sage that match each bank deposit.",
 "scope": "A2X Core reads settlement data from sales channels, splits each payout into sales, fees, refunds, taxes and COGS, and posts one summary per payout to the ledger. In Xero, that summary is an invoice, or a bill when the payout is negative. A2X Subledger keeps an order- and SKU-level double-entry subledger beneath the general ledger, and Clarity adds profitability analytics. It does not keep the books, reach banks or capture documents, and it has no public API.",
 "domains": [
  "commerce",
  "payments",
  "ledger",
  "integration"
 ],
 "customers_served": [
  "smb",
  "mid-market",
  "accountants"
 ],
 "spectrum": {
  "custody": {
   "score": 1,
   "note": "The ledger keeps only summaries; the order-level detail stays in A2X's cloud, and the only documented way out is manual exports."
  },
  "locality": {
   "score": 0,
   "note": "Cloud only."
  },
  "history": {
   "score": 1,
   "note": "Settlements are kept, and posted entries can be voided or 'removed and undone', but import history depends on the plan (3 months on Mini up to 'max') and on the channel archiving old reports."
  },
  "openness": {
   "score": 1,
   "note": "Closed, with no public API; manual exports exist for ledgers such as QuickBooks Desktop, and Clarity has scheduled report exports."
  },
  "breadth": {
   "score": 2,
   "note": "Seven sales and payment channels and four ledgers, all in the e-commerce settlement domain; COGS through Cin7, Finale, Settle and Unleashed."
  },
  "autonomy": {
   "score": 3,
   "note": "Entries can be reviewed before posting, but once auto-posting is turned on new payouts post unattended."
  },
  "agent_readiness": {
   "score": 0,
   "note": "No API, no MCP, no agent features; only an llms.txt file 'for our silicon accountants'."
  },
  "integration_ease": {
   "score": 2,
   "note": "An owner connects channels and ledgers in a few clicks, but a third party or an agent cannot connect to A2X at all."
  },
  "market": {
   "score": 2,
   "note": "E-commerce SMBs and their accountants (1,500+ practices), moving up-market with Subledger and Clarity for high-volume brands."
  }
 },
 "api": {
  "public": false,
  "style": "None published. A2X reads channel APIs (Amazon SP-API settlements, Shopify payouts) and writes to ledger APIs (Xero's invoice API).",
  "auth": "n/a for third parties; owners authorize channels and ledgers by OAuth inside A2X",
  "limits": "Bound by the sources' limits; for example, A2X cannot import Amazon settlement reports that Amazon has archived, so the owner must request them again in Seller Central",
  "cost": "No API to pay for. The subscription sets how much history you can import (Shopify Mini 3 months, Basic 6, Professional 12, Premium max).",
  "export": "Manual exports for ledgers without an integration (such as QuickBooks Desktop); settlement files are downloadable; Clarity can schedule report exports. No documented bulk export of the order-level subledger.",
  "notes": "In Xero, A2X posts through the invoice API: each payout becomes a draft or approved invoice (or a bill if negative) that matches the bank deposit for reconciliation."
 },
 "pricing": {
  "model": "Per sales channel, tiered by monthly order volume; multi-channel bundles; Subledger and Clarity quoted",
  "entry": "US$29/month (Mini, up to 200 orders, 3 months of history) for Shopify, Amazon, eBay, Etsy, TikTok Shop or PayPal; Walmart from US$79",
  "top": "Shopify or Amazon Premium 250k at US$1,499/month; Multi Premium 250K at US$1,869/month; custom above 250,000 orders",
  "notes": "Prices from a2xaccounting.com, read 2026-10-10, in USD and excluding Canadian taxes. The free trial needs no card and posts your first 3 payouts (the Shopify listing says 30 days). COGS needs the Basic plan or above."
 },
 "agents": {
  "mcp": "none",
  "ai_features": "None described on the site as of October 2026.",
  "agent_writes": "No agent access. A2X itself writes summaries to the ledger.",
  "approval": "A person can review each summary in A2X before it is sent and can send drafts rather than approved invoices; after that, 'auto-posting' sends new entries automatically. A posted entry can be voided or removed and undone.",
  "notes": "The review-then-auto-post pattern is a human gate the owner can switch off once they trust it."
 },
 "integration": {
  "scope": "Sales channels: Amazon, Shopify (including B2B), eBay, Etsy, Walmart, TikTok Shop, PayPal (and Amazon Pay). Ledgers: QuickBooks Online, Xero, NetSuite, Sage; manual exports for QuickBooks Desktop and Enterprise. Inventory and COGS: Cin7, Finale, Settle, Unleashed.",
  "notes": "Summaries cut the volume posted to the ledger to one entry per payout, which keeps Xero within its limits and makes bank matching easy. Order-level detail stays in A2X."
 },
 "preference": {
  "scale": "13,000+ e-commerce businesses and 1,500+ accounting and bookkeeping practices (A2X llms.txt, read 2026-10-10)",
  "ratings": "Xero App Store 4.96/5 from 942 reviews; Shopify App Store 5.0/5 from 360 reviews (345 five-star, 4 one-star); Trustpilot 4.8/5 from 112 reviews (66 in the last 12 months), all read 2026-10-10. Xero Small Business App of the Year (US) 2025, Practice App of the Year (UK and Canada) 2024.",
  "sentiment": "Reviewers consistently praise support, guided onboarding and payouts that reconcile to the deposit for multi-store, multi-currency sellers. The few complaints are a learning curve in mapping and clearing accounts and gaps caused by outages at the ledger or channel. Pricing that rises with order volume is a likely complaint but was not confirmed in the reviews read."
 },
 "timeline": [
  {
   "date": "2014",
   "event": "Founded by two e-commerce entrepreneurs ('Paul and Ashley'); their bookkeeper was the first user"
  },
  {
   "date": "2018",
   "event": "Xero acquires Hubdoc, whose co-founder Jamie McDonald later becomes A2X's CEO [timing unverified]"
  },
  {
   "date": "2019-07-12",
   "event": "A2X launches on the Shopify App Store"
  },
  {
   "date": "2022-03",
   "event": "Amazon stops supporting V1 settlement reports; A2X recommends V2"
  },
  {
   "date": "2024",
   "event": "Xero Practice App of the Year (UK and Canada)"
  },
  {
   "date": "2025",
   "event": "Xero Small Business App of the Year (US)"
  },
  {
   "date": "2026",
   "event": "A2X for TikTok Shop; finalist for two Xero Global App Awards 2026; Subledger and Clarity offered to high-volume brands [launch dates unverified]"
  }
 ],
 "relation": "neighbour",
 "relation_note": "A2X sits between the sales channels and the ledger, and its summary invoices are what ownpurse will find in a seller's Xero books. It is not a rival: it compresses the source into one entry per payout, while ownpurse keeps every fact. Without an API, A2X cannot be a source today; the channels themselves (Shopify, Amazon settlement files) are the sources ownpurse would read.",
 "borrow": "Summarize to the deposit: one entry per payout that matches the bank line, with the detail kept beneath it; review before posting, drafts first; an explicit 'remove and undo' for a posted entry; a subledger linking each settlement to its order and SKU.",
 "avoid": "Keeping the order-level detail in a closed cloud with no API or bulk export; limiting import history by plan; depending on a channel that archives its own reports (keep the raw settlement file the first time it is seen).",
 "sources": [
  {
   "title": "A2X homepage",
   "url": "https://www.a2xaccounting.com"
  },
  {
   "title": "A2X llms.txt",
   "url": "https://www.a2xaccounting.com/llms.txt"
  },
  {
   "title": "A2X about and leadership",
   "url": "https://www.a2xaccounting.com/about"
  },
  {
   "title": "A2X pricing",
   "url": "https://www.a2xaccounting.com/pricing"
  },
  {
   "title": "A2X for Shopify pricing",
   "url": "https://www.a2xaccounting.com/shopify/pricing"
  },
  {
   "title": "A2X for Amazon pricing",
   "url": "https://www.a2xaccounting.com/amazon/pricing"
  },
  {
   "title": "A2X multi-channel pricing",
   "url": "https://www.a2xaccounting.com/multi-channel/pricing"
  },
  {
   "title": "A2X for Xero",
   "url": "https://www.a2xaccounting.com/xero"
  },
  {
   "title": "A2X Subledger",
   "url": "https://www.a2xaccounting.com/subledger"
  },
  {
   "title": "A2X Clarity",
   "url": "https://www.a2xaccounting.com/clarity"
  },
  {
   "title": "Why are some of my settlement files missing in A2X?",
   "url": "https://intercom.help/a2x/en/articles/959796-why-are-some-of-my-settlement-files-missing-in-a2x"
  },
  {
   "title": "A2X help centre",
   "url": "https://support.a2xaccounting.com/en/"
  },
  {
   "title": "A2X on the Xero App Store",
   "url": "https://apps.xero.com/app/a2x"
  },
  {
   "title": "A2X on the Shopify App Store",
   "url": "https://apps.shopify.com/a2x"
  },
  {
   "title": "A2X on Trustpilot",
   "url": "https://www.trustpilot.com/review/a2xaccounting.com"
  }
 ],
 "unverified": [
  "Funding, investors and ownership (not disclosed on the site)",
  "Country of origin (often described as New Zealand); the founders' full names",
  "When Jamie McDonald became CEO",
  "Launch dates of Subledger and Clarity",
  "Whether A2X posts manual journals rather than invoices to QuickBooks Online, NetSuite or Sage",
  "Free trial length (the Shopify listing says 30 days; the pricing page gives a three-payout limit)",
  "Annual billing discounts",
  "G2, Capterra and Reddit sentiment (G2 and Capterra blocked; web search budget exhausted)",
  "Whether the order-level data can be exported in bulk on request"
 ],
 "researched": "2026-10-10"
}
---

## What it is

A2X is e-commerce accounting software founded in 2014. It reads payouts from [Amazon, Shopify, eBay, Etsy, Walmart, TikTok Shop and PayPal](https://www.a2xaccounting.com) and posts summaries to QuickBooks Online, Xero, NetSuite or Sage that match each bank deposit. Its [llms.txt](https://www.a2xaccounting.com/llms.txt) cites 13,000+ e-commerce businesses and 1,500+ accounting practices. CEO Jamie McDonald co-founded Hubdoc, which Xero bought in 2018, and several of the [leadership team](https://www.a2xaccounting.com/about) came from Hubdoc or Xero.

It is very good at one hard job. A marketplace deposit mixes sales, fees, refunds, taxes and reserves across many orders, and A2X turns it into one entry a bookkeeper can reconcile.

## How it works

A2X Core pulls settlement data from each channel, splits each payout into its parts and maps them to accounts. In Xero it [posts through the invoice API](https://www.a2xaccounting.com/xero): each payout becomes one invoice, or a bill if it is negative, so the ledger is not flooded with orders. You can review each entry in A2X first, send it as a draft, then switch on auto-posting. An approved entry can be voided or "removed and undone".

[A2X Subledger](https://www.a2xaccounting.com/subledger) is a double-entry subledger beneath the general ledger. It links every settlement to its order and SKU with timestamps and references. [Clarity](https://www.a2xaccounting.com/clarity) adds margin reporting by SKU, channel and customer. Both are quoted after a demo.

## Data and access

A2X has no public API and no documented bulk export of its order-level data. Manual exports serve ledgers without an integration, such as QuickBooks Desktop. History is limited by plan: Shopify [Mini](https://www.a2xaccounting.com/shopify/pricing) imports 3 months, Professional 12, Premium the maximum. It is also limited by the channels: Amazon archives old settlement reports, and A2X [cannot import archived ones](https://intercom.help/a2x/en/articles/959796-why-are-some-of-my-settlement-files-missing-in-a2x) until the seller requests them again. Pricing starts at US$29 a month for 200 orders and reaches US$1,499 for 250,000 on one channel, or US$1,869 on [multi-channel](https://www.a2xaccounting.com/multi-channel/pricing) (all read 10 October 2026).

## Agents

None. A2X has no API, no MCP server and no AI features on its site. Its review-before-posting step, with drafts first, is a real human gate, but the owner can switch it off with auto-posting.

## Customers

Customers rate it very highly: 4.96 from 942 reviews on the [Xero App Store](https://apps.xero.com/app/a2x), 5.0 from 360 on [Shopify](https://apps.shopify.com/a2x) and 4.8 from 112 on [Trustpilot](https://www.trustpilot.com/review/a2xaccounting.com). Reviewers praise support, onboarding and payouts that reconcile. The few complaints are about the learning curve of mapping and clearing accounts, and gaps after outages at a ledger. It won Xero's US Small Business App of the Year in 2025. Accountants are a main channel: A2X runs a directory of certified e-commerce bookkeepers.

## History

Two sellers built it in 2014 for their own multi-channel businesses, and their bookkeeper was its first user. It launched on Shopify's app store in July 2019 and added Walmart, PayPal and TikTok Shop. It is now moving up-market with Subledger and Clarity. Its funding has not been disclosed.

## For ownpurse

A2X shows how small and mid-sized sellers handle cross-source data today. A specialist bridge cuts the detail down to what the ledger can hold and keeps the rest in its own cloud. When ownpurse reads a seller's Xero books, it will find A2X's summary invoices and no orders. To rebuild that detail, ownpurse would read the channel's own settlement file, keep the raw file the first time it appears (before Amazon archives it), and link it to A2X's invoice and the bank deposit. Worth borrowing: summarize to the deposit, review before posting, and an explicit undo. Worth avoiding: detail you cannot get out.
