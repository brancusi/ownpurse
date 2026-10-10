---
{
 "slug": "stripe-agent-toolkit",
 "name": "Stripe Agent Toolkit",
 "url": "https://github.com/stripe/agent-toolkit",
 "segment": "agent-native",
 "status": "active",
 "founded": "2024-11 (npm package first published 2024-10-15; repo created 2024-11-11; announced 2024-11-14)",
 "hq": "Stripe: South San Francisco, California and Dublin, Ireland",
 "backing": "Built by Stripe, Inc. (private). Feb 2026 employee tender offer at a $159B valuation, funded by Thrive Capital, Coatue, a16z and others plus Stripe's own capital; 2025 total volume $1.9T, up 34% (Stripe 2025 annual letter, Feb 2026).",
 "tagline": "Stripe's official open-source tools and hosted MCP server that let AI agents read and write Stripe data through scoped keys or OAuth.",
 "scope": "Function-calling tools for OpenAI's Agent SDK, LangChain, CrewAI and Vercel's AI SDK (Python and TypeScript), a local MCP server (@stripe/mcp) and a hosted one at mcp.stripe.com that exposes generic read and write over most of the Stripe API, plus docs search, analytics and Treasury money movement in preview. It covers only what lives in Stripe: payments, billing, invoices, tax, payouts, issuing. It keeps no record of its own and does no bookkeeping.",
 "domains": [
  "payments",
  "invoicing",
  "tax",
  "bank",
  "commerce",
  "integration"
 ],
 "customers_served": [
  "developers",
  "smb",
  "mid-market",
  "enterprise"
 ],
 "spectrum": {
  "custody": {
   "score": 1,
   "note": "Stripe holds the record; the owner can copy it out in bulk only through paid Sigma or Data Pipeline, or by paging the API."
  },
  "locality": {
   "score": 1,
   "note": "The recommended server is hosted at mcp.stripe.com; the local npx @stripe/mcp runs on your machine but only proxies Stripe's cloud API."
  },
  "history": {
   "score": 1,
   "note": "Balance transactions and events are append-style, but customers, invoices and subscriptions are mutable objects overwritten in place; the toolkit adds no history of its own."
  },
  "openness": {
   "score": 3,
   "note": "The toolkit and MCP package are MIT open source; the platform they front is closed, with an open, fully documented API."
  },
  "breadth": {
   "score": 1,
   "note": "One source: Stripe's payments, billing, tax, issuing and Treasury objects; no books, other banks or documents."
  },
  "autonomy": {
   "score": 3,
   "note": "Agents write directly (stripe_api_write covers POST, PATCH, PUT, DELETE); only refunds, outbound payments and a few configured actions pause for a human."
  },
  "agent_readiness": {
   "score": 4,
   "note": "Official remote MCP with OAuth, agent-tagged API keys, plugins for Claude Code, Codex, Cursor and Grok, agent skills, a CLI setup command and tool-call logs in Workbench."
  },
  "integration_ease": {
   "score": 4,
   "note": "Self-serve: one command (stripe agent setup) or one URL; sandboxes, exhaustive public docs, no partner gate."
  },
  "market": {
   "score": 2,
   "note": "Used by solo developers through enterprises; the toolkit itself is aimed at developers building on any size of Stripe account."
  }
 },
 "api": {
  "public": true,
  "style": "REST (v1 form-encoded, v2 JSON) plus MCP over streamable HTTP; function-calling wrappers for Python and TypeScript",
  "auth": "OAuth for interactive MCP clients; agent-tagged restricted API keys (rk_) for autonomous agents. From 2026-10-31 the hosted MCP rejects full secret keys and restricted keys without the Agent tag.",
  "limits": "Live mode 100 requests/s per account, sandbox 25/s, 25/s per endpoint by default, 50 concurrent live requests; read allocation averages 500 GET requests per transaction over 30 days with a 10,000/month floor; writes unallocated.",
  "cost": "API and MCP access carry no separate fee; Stripe earns on processing. Bulk data: Sigma from EUR 15/month plus per-charge fee from EUR 0.03; Data Pipeline EUR 0.025 per transaction (EU pricing page, Oct 2026).",
  "export": "Data Pipeline to a warehouse or cloud storage, Sigma SQL, CSV reports from the Dashboard, and paging every list endpoint.",
  "notes": "Stripe points data-heavy work to Sigma or Data Pipeline because GET calls are rationed per transaction. The MCP exposes the whole API through four generic tools (search, details, read, write) to keep context small."
 },
 "pricing": {
  "model": "Free, open-source toolkit; Stripe charges per transaction processed and per data product.",
  "entry": "Free (toolkit and MCP). Processing priced per country, e.g. EEA standard cards 1.5% + EUR 0.25.",
  "top": "Custom enterprise pricing for processing; Sigma and Data Pipeline metered per transaction.",
  "notes": "The commonly cited US card rate of 2.9% + 30 cents could not be confirmed from the fetched pricing page, which served the Italian locale."
 },
 "agents": {
  "mcp": "official",
  "ai_features": "Hosted MCP with generic API read/write, docs search, an implementation planner, analytics over metrics and Sigma (preview), Treasury balance and money movement (preview); token-metering SDKs to bill for LLM usage.",
  "agent_writes": "Yes. stripe_api_write calls any POST, PATCH, PUT or DELETE method; permissions follow the OAuth grant or the agent key's restrictions.",
  "approval": "Two mechanisms. With user OAuth, Stripe requires a human to click a confirmation URL before certain writes such as refunds and outbound payments; the approval returns a token and the agent retries; unapproved requests expire after 24 hours. With agent-tagged keys, Dashboard approval rules return approval_required, create an approval request for a designated reviewer (not the requester), complete the action automatically once approved and expire after 14 days. Defaults cover refund creation and subscription cancellation; rules can add conditions on amount, actor or rate. Every rule change and decision is logged in Security history and emitted as v2.core.approval_request events.",
  "notes": "Stripe also tells users to turn on their MCP client's tool-approval setting and warns about prompt injection when combining servers. The gate covers a short list of actions, not every write."
 },
 "integration": {
  "scope": "Agent frameworks: OpenAI Agent SDK, LangChain, CrewAI, Vercel AI SDK; MCP clients: Claude, Claude Code, ChatGPT, Codex, Cursor, VS Code, Gemini CLI, Grok Build. Connect platforms can act for connected accounts with a Stripe-Account header.",
  "notes": "Stripe already integrates with most ledgers through its own and third-party sync apps; the toolkit is about agents, not accounting sync."
 },
 "preference": {
  "scale": "Stripe-wide: $1.9T total volume in 2025, up 34%; stripe/ai repo about 1,865 stars and 353 forks (2026-10-10).",
  "ratings": "Trustpilot 1.6/5 from 17,537 reviews (Oct 2026), 52% one-star. G2 and Capterra figures for Stripe not confirmed.",
  "sentiment": "Developers praise the API and docs; merchant reviews complain about account holds, payout delays and hard-to-reach support. No independent reviews of the agent toolkit itself were found."
 },
 "timeline": [
  {
   "date": "2024-10-15",
   "event": "@stripe/agent-toolkit first published to npm"
  },
  {
   "date": "2024-11-14",
   "event": "Stripe announces the agent toolkit (Vercel AI SDK, LangChain, CrewAI), single-use Issuing cards for agents and token-usage billing middleware, calling it an early exploration"
  },
  {
   "date": "2025-02-19",
   "event": "@stripe/mcp local MCP server first published"
  },
  {
   "date": "2026-02",
   "event": "Stripe annual letter: $1.9T 2025 volume; tender offer at $159B"
  },
  {
   "date": "2026-02-12",
   "event": "@stripe/agent-toolkit 0.9.0 and Python stripe-agent-toolkit 0.7.0 released"
  },
  {
   "date": "2026",
   "event": "Repository renamed from stripe/agent-toolkit to stripe/ai (old URL redirects); README now leads with the hosted MCP, plugins and skills"
  },
  {
   "date": "2026-10-31",
   "event": "Hosted MCP stops accepting full secret keys and untagged restricted keys; agent keys or OAuth required"
  }
 ],
 "relation": "source",
 "relation_note": "Stripe is a source ownpurse will connect to, and its MCP is how an owner's agent already reaches Stripe directly. It is also the clearest incumbent example of a human gate on agent writes, though it covers only a few actions inside Stripe and leaves no record on the owner's machine.",
 "borrow": "Mark agent credentials as agent credentials and govern them differently from a person's; require approval by someone other than the requester; return a structured approval_required error and an approval id the agent can attach a reason to; expire unapproved requests; log every rule and decision; keep the agent tool surface to a few generic read and write tools plus search instead of hundreds of endpoints.",
 "avoid": "A gate that covers only a list of risky actions while every other write goes straight through; approval that lives only in the vendor's dashboard and audit log the owner cannot hold; read rationing that pushes bulk history into a paid product.",
 "sources": [
  {
   "title": "stripe/ai repository (formerly stripe/agent-toolkit)",
   "url": "https://github.com/stripe/ai"
  },
  {
   "title": "Stripe docs: Model Context Protocol",
   "url": "https://docs.stripe.com/mcp"
  },
  {
   "title": "Stripe docs: API keys and agent keys",
   "url": "https://docs.stripe.com/keys"
  },
  {
   "title": "Stripe docs: two-party approvals",
   "url": "https://docs.stripe.com/account/approvals"
  },
  {
   "title": "Stripe docs: rate limits and read allocations",
   "url": "https://docs.stripe.com/rate-limits"
  },
  {
   "title": "Stripe blog: Adding payments to your agentic workflows (2024-11-14)",
   "url": "https://stripe.dev/blog/adding-payments-to-your-agentic-workflows"
  },
  {
   "title": "Stripe pricing",
   "url": "https://stripe.com/pricing"
  },
  {
   "title": "Stripe 2025 annual letter and tender offer",
   "url": "https://stripe.com/newsroom/news/stripe-2025-update"
  },
  {
   "title": "The Block: Stripe valuation jumps to $159 billion",
   "url": "https://www.theblock.co/post/391075/stripe-valuation-jumps-to-159-billion-on-back-of-tender-offer"
  },
  {
   "title": "Trustpilot: Stripe reviews",
   "url": "https://www.trustpilot.com/review/stripe.com"
  },
  {
   "title": "npm: @stripe/agent-toolkit",
   "url": "https://www.npmjs.com/package/@stripe/agent-toolkit"
  },
  {
   "title": "npm: @stripe/mcp",
   "url": "https://www.npmjs.com/package/@stripe/mcp"
  }
 ],
 "unverified": [
  "Exact date stripe/agent-toolkit was renamed to stripe/ai (only the redirect is confirmed)",
  "Date agent-tagged API keys and approval rules launched",
  "Launch date of the hosted mcp.stripe.com server",
  "US standard card rate (commonly cited as 2.9% + 30 cents; the fetched pricing page showed EU rates)",
  "Stripe G2 and Capterra ratings",
  "Retention window of the Events API (commonly 30 days)",
  "Whether approval rules are generally available or in preview (the API example uses a 2026-09-30.preview version header)"
 ],
 "researched": "2026-10-10"
}
---

## What it is

The Stripe Agent Toolkit is Stripe's official way to let an AI agent act on a Stripe account. Stripe [announced it on 14 November 2024](https://stripe.dev/blog/adding-payments-to-your-agentic-workflows) as function-calling tools for Vercel's AI SDK, LangChain and CrewAI, calling it "an early exploration of integrating Stripe into agentic workflows". Today the repository lives at [github.com/stripe/ai](https://github.com/stripe/ai) (the old agent-toolkit URL redirects), MIT licensed, and leads with a hosted MCP server at `mcp.stripe.com`, plugins for Claude Code, Codex, Cursor and Grok, and agent skills. The Python and TypeScript toolkit and a local MCP package (`npx @stripe/mcp`) still ship in its `tools/` folder.

## How it works

The [hosted MCP server](https://docs.stripe.com/mcp) keeps the tool list short: `stripe_api_search` and `stripe_api_details` find a method, `stripe_api_read` calls any GET, and `stripe_api_write` calls any POST, PATCH, PUT or DELETE. Around those sit docs search, an implementation planner, analytics over metrics and Sigma (preview), and Treasury tools in preview that can send and convert money. The supported method list covers customers, charges, refunds, invoices, subscriptions, prices, disputes, payouts, balance transactions, tax and Issuing.

Permissions come from the credential. You connect with OAuth for an interactive client, or with an [agent API key](https://docs.stripe.com/keys): a restricted key tagged as belonging to an autonomous agent. From 31 October 2026 the hosted server refuses full secret keys and untagged restricted keys.

## Data and access

The API is public and documented. Live mode allows [100 requests a second per account, 25 per endpoint and 50 in flight](https://docs.stripe.com/rate-limits). Reads are also rationed: an average of 500 GET requests per transaction over 30 days, with a floor of 10,000 a month. For bulk history Stripe points you to Sigma (from EUR 15 a month plus a per-charge fee) or Data Pipeline (EUR 0.025 per transaction) on its [pricing page](https://stripe.com/pricing). The record stays in Stripe's cloud. Customers, invoices and subscriptions are overwritten in place; balance transactions and events are closer to a log.

## Agents

This is the most developed approval model among incumbents. With user OAuth, Stripe asks a person to open a confirmation link before refunds and outbound payments; the approval hands the agent a token and the agent retries. Unapproved requests expire after 24 hours. Agent-tagged keys fall under [approval rules](https://docs.stripe.com/account/approvals): the call returns `approval_required`, an approval request goes to a designated reviewer who cannot be the requester, and the action completes once approved or expires after 14 days. Defaults cover refunds and subscription cancellations; you can add amount, actor and rate conditions. Every rule change and decision goes to Security history and webhooks. On a one-person account, the agent key counts as the second party, so the owner approves the agent.

The limit is coverage. Only a short list of actions can be gated (bank account changes, admin invites, payment intents, refunds, subscriptions, credit notes, customer balance transactions). Every other write goes straight through, and Stripe tells you to turn on your client's tool approval and to watch for prompt injection.

## Customers

Stripe processed $1.9 trillion in 2025, up 34%, and priced its February 2026 tender at $159 billion ([Stripe newsroom](https://stripe.com/newsroom/news/stripe-2025-update), [The Block](https://www.theblock.co/post/391075/stripe-valuation-jumps-to-159-billion-on-back-of-tender-offer)). The repository has about 1,865 stars. Developers like the API; merchants on [Trustpilot](https://www.trustpilot.com/review/stripe.com) give Stripe 1.6 out of 5 across 17,537 reviews, mostly over account holds, payout delays and support. We found no independent reviews of the toolkit.

## History

The npm package first appeared in October 2024, the announcement followed in November with single-use Issuing cards for agents and token-billing middleware, and the local MCP package arrived in February 2025. Through 2026 the emphasis moved from framework wrappers to the hosted MCP, plugins and agent keys with approvals.

## For ownpurse

Stripe is a source ownpurse will connect to, not a rival. Its agent surface answers "let my agent act on Stripe". It does not answer "let me hold Stripe's history beside my books and check it to the cent". The design is worth copying: agent credentials marked as agents, a structured `approval_required` error, a reviewer who is not the requester, expiry, and a log of every decision. The gap is that the gate covers a list of actions rather than every write, and the approval and its record live in Stripe's dashboard, not in a file you hold. ownpurse's sealed packages go further: every write is gated, and the record keeps every attempt.
