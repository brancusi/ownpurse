---
{
 "slug": "zeni",
 "name": "Zeni",
 "url": "https://www.zeni.ai",
 "segment": "bookkeeping-service",
 "status": "active",
 "founded": "2019",
 "hq": "Palo Alto, California, US (team also in India)",
 "backing": "Venture-backed. $13.5M Series A led by Saama Capital (announced March 2021), $34M Series B led by Elevation Capital with Think Investments and Neeraj Arora (August 2021). Aggregators put the total at about $49M to $50.9M; CB Insights lists a further small round of about $1.5M on 20 November 2025 (unconfirmed by Zeni).",
 "tagline": "A bookkeeping, tax and fractional-CFO service for US startups, run by an in-house finance team with AI tooling, on top of the customer's QuickBooks Online.",
 "scope": "Zeni does monthly bookkeeping, reconciliation, month-end close, reporting, bill pay, reimbursements, tax filing, payroll management and fractional CFO work for startups, and sells business checking and cards through partner banks. The books themselves sit in QuickBooks Online Plus, which Zeni requires, and Zeni's dashboard and agents read from them. It does not offer a public API, does not run on your machine, and its MCP server is read-only.",
 "domains": [
  "ledger",
  "bank",
  "payments",
  "close",
  "reporting",
  "tax",
  "payroll",
  "spend"
 ],
 "customers_served": [
  "smb"
 ],
 "spectrum": {
  "custody": {
   "score": 1,
   "note": "The ledger lives in a QuickBooks Online Plus company Zeni requires, so you can export it from QBO, but Zeni's team keeps it and Zeni's terms on ownership and offboarding are not published."
  },
  "locality": {
   "score": 0,
   "note": "Cloud service only."
  },
  "history": {
   "score": 1,
   "note": "Whatever QBO's audit log keeps; Zeni offers no as-of queries or versioned record of its own."
  },
  "openness": {
   "score": 1,
   "note": "Closed, no public API; a read-only MCP server is the only programmatic way in."
  },
  "breadth": {
   "score": 3,
   "note": "Books, close, reporting, bill pay, reimbursements, tax, payroll service, checking and cards in one vendor relationship, though many are services and not data feeds."
  },
  "autonomy": {
   "score": 2,
   "note": "The AI Accountant agent and Zeni's human team post to the books for you; Zeni describes the work as AI and people together and does not publish how much runs unattended."
  },
  "agent_readiness": {
   "score": 3,
   "note": "Official remote MCP server with OAuth, scoped to your company, 20 read-only tools; no CLI, no write tools yet, no public API docs."
  },
  "integration_ease": {
   "score": 2,
   "note": "Connecting an AI client takes a URL and a sign-in, but there is no API for third parties, and all books must first be moved to QBO Plus."
  },
  "market": {
   "score": 2,
   "note": "Startups and small businesses, pre-revenue up to companies with over $100M in revenue (2021); 92% of G2 reviewers are small businesses."
  }
 },
 "api": {
  "public": false,
  "style": "No public REST API documented. Remote MCP server at https://api.zeni.ai/zeni-mcp-server/mcp.",
  "auth": "OAuth sign-in with a Zeni account, scoped to one company (MCP).",
  "limits": "Not published.",
  "cost": "MCP included at no extra cost with a Zeni Bookkeeping subscription; not available to non-subscribers as far as the page says.",
  "export": "Not documented by Zeni. Because the books are kept in the customer's QuickBooks Online Plus, QBO's own exports apply; the FAQ does not say who owns the QBO company or what happens on cancellation.",
  "notes": "During onboarding Zeni asks for read-only access to QBO, banks, cards, payroll and payment processors. The MCP page says the Zeni dashboard is always the source of truth."
 },
 "pricing": {
  "model": "Monthly subscription priced on bookkeeping complexity, monthly expenses and revenue, plus separate service lines with setup fees and per-form add-ons.",
  "entry": "Bookkeeping Starter from $549/month ($494/month billed annually), as of October 2026.",
  "top": "Bookkeeping Enterprise custom; Fractional CFO Enterprise from $4,990/month plus a $6,000 setup fee.",
  "notes": "Bookkeeping Growth from $799/month. CFO Lite from $1,599/month plus $2,000 setup; CFO Premium from $2,990/month plus $6,000 setup. Tax $2,499/year (pre-revenue) and $3,899/year (revenue up to $500K). Payroll manager from $199/month on an annual contract. R&D credit work at 20% of the credit. Bill pay and reimbursements listed as free. Each business or location needs its own subscription. QBO Plus is a separate cost."
 },
 "agents": {
  "mcp": "official",
  "ai_features": "AI Accountant agent (categorisation, reconciliation, receipt matching, journal entries, close support), AI CFO agent (runway, scenarios, plain-English Q&A, statements for boards), AI financial reporting, Zeni MCP for Claude, OpenAI Codex and Google Antigravity.",
  "agent_writes": "Your own agents cannot write: the MCP tools only report and analyse, and Zeni says nothing moves money or edits your books. Zeni's own AI Accountant agent does write to the books.",
  "approval": "No owner approval gate is described for Zeni's own agent writes; the customer relies on Zeni's finance team. Agentic write actions over MCP are on Zeni's roadmap with no gate described yet.",
  "notes": "Launched around mid-2026 (inferred from a LinkedIn post, no exact date). Answers are said to refresh when the books update."
 },
 "integration": {
  "scope": "QuickBooks Online Plus (required), Plaid, Stripe, PayPal, Square, Avalara; banks, cards, payroll providers and payment processors read during onboarding; banking via Thread Bank, Community Federal Savings Bank, Column N.A. and Nium.",
  "notes": "Zeni migrates customers on other ledgers to QBO Plus. The pricing table says 'QBO Pro' while the FAQ says 'QBO Plus'."
 },
 "preference": {
  "scale": "Over 100 startup customers and over $500M a month in funds managed (August 2021, company figures). Homepage claims $20B in transactions managed a year (October 2026). Third-party estimates of customers (about 400), staff (about 200 to 450) and revenue ($5.7M to $128.9M) conflict and are not reliable.",
  "ratings": "G2: 4.6 of 5 from 81 reviews (October 2026). Trustpilot: no reviews, profile unclaimed (October 2026). Capterra could not be checked.",
  "sentiment": "Reviewers praise the people more than the software: personable, quick, knowledgeable bookkeepers and responsiveness (G2 score about 9.6). Complaints found were few: many onboarding meetings, and bugs in bill pay. No Reddit discussion turned up."
 },
 "timeline": [
  {
   "date": "2019",
   "event": "Founded by twin brothers Swapnil Shinde (CEO) and Snehal Shinde (CPO), earlier founders of Mezi, which American Express bought in 2018; beta in late 2019."
  },
  {
   "date": "2021-03",
   "event": "Public launch as an 'AI-powered finance concierge' for startups with $13.5M raised in a Series A led by Saama Capital."
  },
  {
   "date": "2021-08",
   "event": "$34M Series B led by Elevation Capital; over 100 customers, 550% revenue growth year over year."
  },
  {
   "date": "2025-11-20",
   "event": "Small funding round of about $1.5M to $2M reported by CB Insights and Caplight (unconfirmed)."
  },
  {
   "date": "2026-01-15",
   "event": "AI treasury product launch listed by Caplight (unconfirmed)."
  },
  {
   "date": "2026 (about June)",
   "event": "Zeni MCP launched: 20 read-only tools for Claude, OpenAI Codex and Google Antigravity."
  }
 ],
 "relation": "competitor",
 "relation_note": "Zeni sells the opposite answer to the same problem: hand your books to a vendor's team and its agents, and ask questions of them through its dashboard or MCP. ownpurse keeps the record with the owner and makes the owner's agents propose while the person posts.",
 "borrow": "A remote MCP server that takes one URL and an OAuth sign-in, scoped to one company, with a short fixed set of named read tools (balance sheet, P&L, cash flow, burn, runway, AR/AP ageing) is a good model for ownpurse's first agent surface. So is saying plainly on the page that the tools cannot move money or edit books.",
 "avoid": "Requiring migration onto one ledger (QBO Plus) before anything works, leaving data ownership and offboarding unstated, and letting the vendor's own agent post to the books with no owner-side approval or record of each change.",
 "sources": [
  {
   "title": "Zeni pricing",
   "url": "https://www.zeni.ai/pricing"
  },
  {
   "title": "Zeni MCP product page",
   "url": "https://www.zeni.ai/product/zeni-mcp"
  },
  {
   "title": "Zeni FAQ",
   "url": "https://www.zeni.ai/faq"
  },
  {
   "title": "Zeni AI CFO agent page",
   "url": "https://www.zeni.ai/ai-cfo-agent"
  },
  {
   "title": "Zeni homepage",
   "url": "https://www.zeni.ai"
  },
  {
   "title": "Accounting Today: Zeni receives $34M (August 2021)",
   "url": "https://accountingtoday.com/news/zeni-receives-34m-in-funding-for-ai-bookkeeping-and-accounting"
  },
  {
   "title": "Accounting Today: Zeni launches with $13.5M (March 2021)",
   "url": "https://www.accountingtoday.com/news/ai-bookkeeping-startup-zeni-launches-with-13-5m-in-funding"
  },
  {
   "title": "Business Wire: Zeni Series B release",
   "url": "https://www.businesswire.com/news/home/20210804005691/en/Fintech-Zeni-Announces-34-Million-Series-B-In-Wake-of-550-Revenue-Growth"
  },
  {
   "title": "CB Insights: Zeni financials",
   "url": "https://www.cbinsights.com/company/zeni/financials"
  },
  {
   "title": "Caplight: Zeni",
   "url": "https://www.caplight.com/company/zeni"
  },
  {
   "title": "G2: Zeni",
   "url": "https://www.g2.com/sellers/zeni"
  },
  {
   "title": "Trustpilot: zeni.ai",
   "url": "https://www.trustpilot.com/review/zeni.ai"
  },
  {
   "title": "Zeni on LinkedIn",
   "url": "https://www.linkedin.com/company/zeni-ai"
  }
 ],
 "unverified": [
  "The November 2025 round: CB Insights says $1.5M unattributed VC, Caplight says $2M seed; Zeni has not announced it.",
  "The January 2026 AI treasury launch is from Caplight only.",
  "Zeni MCP launch date (about June 2026) is inferred from the age of a LinkedIn post.",
  "Total funding: aggregators say $49M to $50.9M; Series A is reported as $13.5M (most sources) or $11.5M (Fintech Futures).",
  "Current customer count, headcount and revenue: third-party figures conflict (about 400 customers; 200 to 450 staff; $5.7M to $128.9M revenue).",
  "Whether the QBO Plus company belongs to the customer or to Zeni, and the offboarding process; neither is documented.",
  "Whether the required edition is QBO Plus (FAQ) or QBO Pro (pricing table).",
  "How much of the AI Accountant agent's posting runs without human review.",
  "Capterra rating could not be retrieved; no Reddit threads were found.",
  "Search budget ran out before every point could be cross-checked."
 ],
 "url_note": null,
 "researched": "2026-10-10"
}
---

## What it is

[Zeni](https://www.zeni.ai) is a bookkeeping, tax and fractional-CFO service for US startups. You pay a monthly fee, and Zeni's in-house finance team, working with its own AI tooling, keeps your books, closes the month, pays bills and answers questions. It also sells business checking and cards through partner banks. It was founded in 2019 by twin brothers Swapnil and Snehal Shinde, who earlier built Mezi, the travel assistant American Express bought in 2018 ([Accounting Today, August 2021](https://accountingtoday.com/news/zeni-receives-34m-in-funding-for-ai-bookkeeping-and-accounting)).

## How it works

The ledger is not Zeni's own. Its [FAQ](https://www.zeni.ai/faq) says QuickBooks Online Plus is required to sync with the Zeni dashboard, and customers on other software are migrated to it. During onboarding Zeni asks for read-only access to QBO, banks, cards, payroll and payment processors. Entries are made by "a blend of human financial experts in collaboration with our AI-powered technology platform". On the product side Zeni names an [AI Accountant agent](https://www.zeni.ai/ai-cfo-agent) (categorising, reconciling, matching receipts, journal entries, close) and an AI CFO agent (runway, scenarios, plain-English answers). Zeni does not publish how much of the posting runs without a person.

Pricing, as of October 2026, is on the [pricing page](https://www.zeni.ai/pricing): bookkeeping from $549 a month (Starter) or $799 a month (Growth), 10% less billed annually, with Enterprise custom. Fractional CFO runs from $1,599 a month plus a $2,000 setup fee up to $4,990 a month plus $6,000 setup. Tax is $2,499 or $3,899 a year with per-form add-ons, and payroll management starts at $199 a month on an annual contract. Each business or location needs its own subscription.

## Data and access

There is no public API. Zeni does not document data export, ownership of the QBO company or what happens when you leave; the practical exit is QuickBooks' own exports. History is whatever QBO's audit log keeps.

## Agents

[Zeni MCP](https://www.zeni.ai/product/zeni-mcp) is a remote MCP server at one URL. You add it as a custom connector in Claude, OpenAI Codex or Google Antigravity, sign in with OAuth, and get 20 read-only tools across reports, cash, AR/AP, vendors, bills and reimbursements. The page is direct about scope: the tools "report and analyze", "nothing moves money or edits your books", and "your Zeni dashboard is always the source of truth". It is included with a bookkeeping subscription. Write actions are on the roadmap. A LinkedIn post dates the launch to roughly mid-2026; no exact date was found. This is a well-made read surface, and among the clearer MCP offers from a bookkeeping firm.

## Customers

In August 2021 Zeni reported over 100 startup customers, from pre-revenue to over $100M in revenue, and over $500M a month in funds managed. Its homepage now claims $20B in transactions a year. On [G2](https://www.g2.com/sellers/zeni) it holds 4.6 of 5 from 81 reviews, 92% from small businesses; reviewers praise the bookkeepers' responsiveness more than the software, and the few complaints are about onboarding meetings and bill-pay bugs. Trustpilot has no reviews. Third-party estimates of headcount and revenue disagree too widely to use.

## History

Beta in late 2019, public launch in March 2021 with a $13.5M Series A led by Saama Capital, then a $34M Series B led by Elevation Capital in August 2021 on 550% revenue growth. Aggregators list a small round of about $1.5M to $2M in November 2025 and a treasury product in January 2026; neither is confirmed by Zeni. Its category has a hard record: Bench shut without notice in December 2024 with customers' books inside, and ScaleFactor closed in 2020.

## For ownpurse

Zeni is the done-for-you pole: a vendor team and a vendor agent keep the books, and you read them. It is fair to say this suits a founder who wants no part of bookkeeping. Its MCP design is worth copying: one URL, OAuth, company scope, a short list of named read tools, and plain words about what cannot happen. Where ownpurse differs is custody and writes. Zeni's record is the vendor's work in a QBO file; ownpurse keeps every version in a file the owner holds, and its agents propose sealed packages a person approves. A Zeni customer is also a plausible ownpurse user: reading their QBO into a local record gives them a copy that survives the vendor.
