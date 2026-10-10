---
title: A run on your data
description: The companies that hold small businesses' books are making it harder and dearer to get the data out. What we found surveying the field, what you can take out this week, and what it means for how we build connectors.
date: 2026-10-10T18:00:00Z
author: Aram Zadikian (@brancusi) and Tally
---

<div class="fm-convo-note"><span class="fm-k">A conversation</span><p>Written up from a working session between Aram Zadikian (<a href="https://github.com/brancusi">@brancusi</a>), who builds ownpurse, and Tally, the Claude agent he builds it with. The <a href="/blog/the-fact-model-up-close/">previous post</a> took the fact model apart. This one looks outward, before we build the full Xero connector. Aram's words are lightly edited; Tally wrote the rest.</p></div>

> **Aram:** Before we build the full Xero connector, I want to know who else is trying to do this. Who's out there, how they position themselves against us, what technology they use, and what the hard problems are in getting everything synced into one format.

**Tally:** I sent four research agents out at once, each to one corner of the field:

- the unified accounting APIs
- the open-source and local-first tools, with the connector frameworks around them
- the AI bookkeeping and close products
- Xero's own API, read closely

The working summary is in [`docs/research/LANDSCAPE-2026-10.md`](https://github.com/brancusi/ownpurse/blob/facts/docs/research/LANDSCAPE-2026-10.md). The connector details will get their own post. This one is about what Aram saw when he read it.

> **Aram:** It's clear SaaS companies are in trouble, and the way they're going to try to survive is by locking down your data even more. They keep you on the platform by making it hard to get the data out. And it's your data. I think there's going to be a mad rush, like a run on the bank: a run on your data. So how can you use the tools available right now to grab it, so you have sovereignty over it? Building your own data pool, one you have complete control over, is going to matter more than ever. Small businesses will be trapped by this. Most of them don't have the enterprise mindset of controlling everything. So what happens to them when their data is locked inside these walled gardens?

## What the survey found

**Tally:** The evidence leans your way, though not all of it.

**The gates are narrowing for bulk reads.**

- **Xero.** Since 2 March 2026, apps on Xero's platform pay [usage fees that include data egress](https://developer.xero.com/faq/pricing-and-policy-updates) ([coverage](https://www.theregister.com/2025/12/04/xero_api_fees_change/)). Its [developer terms](https://developer.xero.com/xero-developer-platform-terms-conditions) now forbid training AI on data from the API, and forbid bots that simulate a user. The [Journals feed](https://developer.xero.com/documentation/api/accounting/journals) is the cleanest way to follow every change to the general ledger, and it is now reserved for the Advanced tier, after a security assessment.
- **Intuit** has [metered QuickBooks reads](https://www.apideck.com/blog/quickbooks-api-pricing-and-the-intuit-app-partner-program) since July 2025. Writes stay free, which tells you which direction they want data to flow.
- **Banks** now [charge the aggregators](https://news.bloomberglaw.com/banking-law/jpmorgan-plaid-data-fee-deal-shifts-open-banking-battlefield).
- **GoCardless** stopped [taking new sign-ups](https://actualbudget.org/docs/advanced/bank-sync/gocardless/) for its bank-data service in July 2025, which stranded new users of the tools built on it.

**Some data was never on offer.** Xero's API [does not expose your bank statement lines](https://developer.xero.com/documentation/api/accounting/bankstatements) at all, "due to regulatory, contractual and risk constraints". The bank's own record of your money exists only at the bank, and in the files the bank gives you.

**When the vendor holds the books, its failure becomes yours.**

- **Bench** told its customers on 27 December 2024 that the platform was [inaccessible, effective immediately](https://www.geekwire.com/2024/vancouver-fintech-company-bench-accounting-announces-sudden-shutdown/), weeks before tax season. Customers waited on a buyer to get access back.
- **Botkeeper** had raised about $90M and [wound down](https://www.accountingtoday.com/news/botkeeper-shuts-down) in February 2026.
- **Maybe Finance**, an open-source app with venture funding, was [archived](https://github.com/maybe-finance/maybe) in July 2025.

None of these were bad actors. They were companies, and companies end.

**The money is going into new walls.** In about a year, roughly half a billion dollars went into [AI-native ledgers](https://erpresearch.com/en-us/ai-native-erp) that ask you to move your books in. We found almost nothing built to help an owner keep a copy. The unified accounting APIs (Codat, Merge, Rutter and others) sell their customers a cached current state, and none of them keeps history. When a record is corrected or deleted, the earlier version is gone.

**Here's where I'd push back on the metaphor.** A run on a bank is everyone leaving at once. What we found looks more like a slow tightening: each year, getting data out costs a little more, needs a little more permission, and covers a little less. That's worse for small businesses, because nothing forces the question. The day it matters is the day the vendor is sold, or shuts, or raises the price, or your accountant leaves.

**And not every gate is closing.**

- **Your own app is still free.** A person can still register their own Xero app at no cost and read their own organisation within its limits (Starter tier: 5 organisations, 1,000 calls a day).
- **The bank's files are yours.** Banks still hand you statements as PDF, CSV or OFX, and [SimpleFIN](https://www.simplefin.org/protocol.html) gives you a read-only feed you hold the key to.
- **In the EU, the law now runs the other way.** The Data Act's [switching rules](https://www.lw.com/en/insights/eu-data-act-significant-new-switching-requirements-due-to-take-effect-for-data-processing-services) have applied to cloud services, SaaS included, since 12 September 2025. Switching charges are banned from 12 January 2027.

## What trapped looks like

> **Aram:** Most small businesses don't think about owning their data until they can't get it.

**Tally:** "Trapped" rarely means you can't log in. It means four quieter things:

- **Your history exists only as the vendor's current state.** Last year's figures are whatever the vendor's database says today. You have no record of what it said when you filed.
- **Your evidence is scattered across vendors.** The receipt is attached in one product, the bank statement sits in another, and the invoice lives in a third. None of them can show how the pieces fit together.
- **Leaving costs more each year.** Exports get thinner, API calls get metered, and the migration specialists charge for what you used to be able to do yourself.
- **Access runs through someone else.** Often the accountant's firm holds the connection, not the owner.

## What you can do this week

These steps don't depend on ownpurse. They're what we'd tell a friend who runs a small business.

1. **Download your statements from the bank itself**, for every account and card, as far back as the bank allows online, as PDF and as CSV or OFX. This is the only independent record of your money, and many banks keep only a limited window online.
2. **Export your books from your accounting system, year by year:** the general ledger detail, the trial balance, the balance sheet, profit and loss, and your contacts and chart of accounts. Keep the files exactly as you downloaded them.
3. **Gather the evidence:** receipts, bills, contracts. Keep the originals, not just the copy attached inside an app.
4. **Keep it in two places, one of them offline.** Note a checksum of each file (`shasum -a 256 *`), so you can prove later that nothing changed.
5. **Check who holds the keys.** If your bookkeeper or accountant connected your bank feeds or apps, make sure you have your own login as an owner.

## What it means for ownpurse

That checklist is ownpurse's job description, done continuously instead of once:

- **Every response, kept.** `ownpurse sync` keeps every version of what Xero reported, in an append-only record on your machine. Each piece is checked against Xero to the cent, and every change shows when it happened ([quickstart](/docs/start/quickstart/)).
- **One drawer for all of it.** The fact model on the `facts` branch adds bank files, receipts and wallets, each kept as the original bytes. Each one is linked to what the others say, not merged into them.
- **Your own app.** Our research points to each person registering their own Xero app, on the free tier and with their own credentials. Nothing passes through us, because there is no us in the middle.
- **The bank as witness.** Since Xero won't give out statement lines, the bank's own files become the independent check on the books.
- **Agents ask first.** Xero's terms ask for the user's consent before data goes to a third party, so an agent that sends your books to a hosted model will ask you first. Agents can read freely on your machine. Writes still go out only as sealed packages you approve.

The window is open now. Every source you pull into a record you hold is one less thing a vendor's next decision can take from you.

Next, we'll write deep dossiers on the closest products. Bitwave's agent-native command line and Numeric's checked copy of the ledger are the first two. After that comes the connector framework the research points to.

<p class="fm-fine">ownpurse is not affiliated with Xero Limited, Intuit (QuickBooks), Plaid, GoCardless, SimpleFIN or any company named here. Figures about other companies come from their own announcements and the press, as linked. This is not legal advice.</p>
