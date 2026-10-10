---
{
 "slug": "simplefin",
 "name": "SimpleFIN",
 "url": "https://www.simplefin.org",
 "segment": "bank-aggregator",
 "status": "active",
 "founded": "2013 (GitHub organisation created 26 January 2013); the Bridge's public issue tracker dates from November 2021",
 "hq": "Not stated; the protocol is maintained by Matt Haggard (GitHub iffy), who also makes the Buckets budgeting app",
 "backing": "No outside funding found. The Bridge is funded by user subscriptions ($1.50 a month or $15 a year).",
 "tagline": "An open, read-only protocol, and a small paid bridge, that let a person hand an app a token to read their own bank accounts.",
 "scope": "SimpleFIN is a protocol for read-only bank data, described as RSS for financial information, plus SimpleFIN Bridge, a paid service that implements it for banks that do not. The person creates a token and gives it to whichever app they choose. It returns accounts, balances and transactions by date window; it does no categorisation, no payments and no books.",
 "domains": [
  "bank",
  "personal"
 ],
 "customers_served": [
  "consumer",
  "developers"
 ],
 "spectrum": {
  "custody": {
   "score": 3,
   "note": "The person holds the access token and pulls into an app of their choice, which can keep the copy; the Bridge stores bank connections through MX, and the bank remains the authority."
  },
  "locality": {
   "score": 3,
   "note": "The client runs anywhere, including your laptop, with no server of your own; the Bridge itself is a hosted service you cannot self-host."
  },
  "history": {
   "score": 1,
   "note": "Date-window queries over current transactions; no change feed, no delete signal, at most 90 days per request."
  },
  "openness": {
   "score": 3,
   "note": "The protocol is a public spec on GitHub that anyone can implement; the Bridge is a closed hosted service."
  },
  "breadth": {
   "score": 1,
   "note": "Accounts, balances and transactions only."
  },
  "autonomy": {
   "score": 0,
   "note": "Returns data when asked; categorises and writes nothing."
  },
  "agent_readiness": {
   "score": 2,
   "note": "One GET endpoint with basic auth that any agent can call; several community MCP servers, none official."
  },
  "integration_ease": {
   "score": 4,
   "note": "The spec reads in about a day, setup is pasting one token, and the Bridge offers a demo token for testing."
  },
  "market": {
   "score": 0,
   "note": "Individuals and the small budgeting apps they use."
  }
 },
 "api": {
  "public": true,
  "style": "REST: one GET /accounts endpoint returning JSON; v2.0.0 released 19 March 2026",
  "auth": "A one-time setup token (Base64 URL) is claimed by POST for an Access URL carrying HTTP Basic credentials; claiming twice returns 403, a sign of compromise",
  "limits": "Bridge: plan for 24 requests a day or fewer, with separate quotas for all-accounts and single-account requests; exceeding the warning level disables the token. At most 90 days per request; history depth varies by bank. Up to 25 institutions and 25 apps per subscription.",
  "cost": "$1.50 plus tax a month or $15.00 plus tax a year for the Bridge; the protocol is free and banks pay nothing to implement it.",
  "export": "Every account and transaction the token can see, as JSON, by date window; the person can mint tokens for as many apps as they like (up to 25).",
  "notes": "Parameters: start-date, end-date, pending=1, account (repeatable), balances-only=1, version=2. Transactions carry id, posted, amount, description, optional transacted_at and pending. Structured errors in errlist with gen., con. and act. prefixes; 402 means payment required."
 },
 "pricing": {
  "model": "Flat subscription paid by the person",
  "entry": "$1.50 a month",
  "top": "$15.00 a year (same service)",
  "notes": "No tiers. No developer fee: an app integrates for free and the user pays the Bridge."
 },
 "agents": {
  "mcp": "community",
  "ai_features": "None from SimpleFIN. Community projects on GitHub (2026) include read-only SimpleFIN MCP servers and self-hosted finance apps with an MCP endpoint.",
  "agent_writes": "No. All access is read-only by design.",
  "approval": "The person creates each token and can revoke it at the Bridge; staff debugging access to transaction data needs the user's explicit, logged, expiring grant.",
  "notes": "The simplest bank source an agent can hold on a person's behalf: one URL, read-only, revocable."
 },
 "integration": {
  "scope": "Through the Bridge, banks reachable by MX (exact count not stated); apps listed in the ecosystem include Actual Budget, Buckets, Strata Money, Skwad, Sparky Budget, ZapYeti, EnvelopeBudget and Synci; Sure also supports it.",
  "notes": "The Bridge outsources bank connections to MX, cards to Stripe and hosting to DigitalOcean. Coverage and data quality therefore follow MX's, with a further layer of translation."
 },
 "preference": {
  "scale": "No user or revenue figures published.",
  "ratings": "None found on G2, Capterra or Trustpilot; Reddit could not be read in this pass.",
  "sentiment": "Self-hosters and local-first budgeters adopt it because it is cheap and puts the token in their hands. The public issue tracker shows the costs of a thin layer over MX: Capital One transaction ids re-issued for whole date blocks so clients import duplicates (September 2026), descriptions cut to 32 characters from the OFX NAME field (April 2026), mirror transactions across linked Mercury accounts (February 2026), and connections stuck in maintenance for weeks."
 },
 "timeline": [
  {
   "date": "2013-01-26",
   "event": "SimpleFIN GitHub organisation created; the protocol site follows."
  },
  {
   "date": "2013-10",
   "event": "bank-access repository of per-bank retrieval scripts (Apache-2.0)."
  },
  {
   "date": "2016-06",
   "event": "sfin2ledger published, converting SimpleFIN data to Ledger journals (MIT)."
  },
  {
   "date": "2021-11-29",
   "event": "Public issue tracker for SimpleFIN Bridge opens."
  },
  {
   "date": "2026-03-19",
   "event": "Protocol v2.0.0: errlist, connections, conn_id, account filter, balances-only."
  },
  {
   "date": "2026-09-07",
   "event": "Issue reported: Capital One ids re-issued in blocks via the Bridge, producing duplicates in clients that key on id."
  }
 ],
 "relation": "source",
 "relation_note": "The local-first fit for ownpurse's bank witness: the person holds the token, the data lands on their machine, and it costs them $15 a year. Its data comes from MX, so it is an independent witness to Xero but not to the bank.",
 "borrow": "The claim-once setup token exchanged for a revocable read-only URL; one endpoint with date windows; structured error prefixes; a service whose stated mission is to stop being needed once banks share data themselves.",
 "avoid": "Keying identity on SimpleFIN's transaction id (Capital One reissues them); trusting descriptions as complete (32-character truncation); assuming a missing transaction was deleted, since there is no delete signal. Use the identity ladder and fingerprints, and never infer a delete from a window.",
 "sources": [
  {
   "title": "SimpleFIN home",
   "url": "https://www.simplefin.org/"
  },
  {
   "title": "SimpleFIN protocol v2",
   "url": "https://www.simplefin.org/protocol.html"
  },
  {
   "title": "SimpleFIN changelog",
   "url": "https://www.simplefin.org/CHANGELOG.html"
  },
  {
   "title": "SimpleFIN ecosystem",
   "url": "https://www.simplefin.org/ecosystem.html"
  },
  {
   "title": "SimpleFIN Bridge",
   "url": "https://beta-bridge.simplefin.org/"
  },
  {
   "title": "SimpleFIN Bridge developer guide",
   "url": "https://beta-bridge.simplefin.org/info/developers"
  },
  {
   "title": "SimpleFIN Bridge security",
   "url": "https://beta-bridge.simplefin.org/info/security"
  },
  {
   "title": "SimpleFIN Bridge mission",
   "url": "https://beta-bridge.simplefin.org/info/mission"
  },
  {
   "title": "SimpleFIN on GitHub",
   "url": "https://github.com/simplefin"
  },
  {
   "title": "Bridge issue: Capital One ids re-issued",
   "url": "https://github.com/simplefin/bridge-issues/issues/28"
  },
  {
   "title": "Bridge issue: descriptions truncated",
   "url": "https://github.com/simplefin/bridge-issues/issues/22"
  },
  {
   "title": "Bridge issue: Mercury mirror transactions",
   "url": "https://github.com/simplefin/bridge-issues/issues/21"
  }
 ],
 "unverified": [
  "The legal entity that operates SimpleFIN Bridge; the site names only \"SimpleFIN\".",
  "The number of institutions reachable through the Bridge.",
  "Number of users or subscribers.",
  "Reddit and review-site sentiment (Reddit could not be fetched; no review-site profiles found).",
  "Whether MX is the only provider behind the Bridge; the security page names only MX."
 ],
 "researched": "2026-10-10"
}
---

## What it is

SimpleFIN is a small, open protocol for read-only financial data, which its [site](https://www.simplefin.org/) calls "like RSS for financial information." Its picture is a window on a safe: people can look but cannot touch, and you decide who looks. Next to the protocol sits [SimpleFIN Bridge](https://beta-bridge.simplefin.org/), a paid service that speaks SimpleFIN for banks that do not, for $1.50 a month or $15 a year, covering up to 25 institutions and 25 apps. The project is maintained by Matt Haggard, who also makes the Buckets budgeting app ([GitHub](https://github.com/simplefin/simplefin.github.com)).

## How it works

The person creates a setup token at the Bridge and pastes it into an app. The app decodes it, claims it once by POST and receives an Access URL with Basic credentials ([protocol](https://www.simplefin.org/protocol.html)). A second claim returns 403, which the app should treat as a possible compromise. From then on the app calls `GET /accounts` with `start-date`, `end-date`, optional `pending=1`, an `account` filter or `balances-only=1`. Version 2.0.0 ([19 March 2026](https://www.simplefin.org/CHANGELOG.html)) added structured errors (`errlist`, with `con.auth`, `act.missingdata` and similar codes), a `connections` list, and `conn_id` to tell apart two logins to the same bank.

Behind the Bridge, [MX holds the bank connections](https://beta-bridge.simplefin.org/info/security); Stripe takes payment and DigitalOcean hosts the servers. No bank credentials touch SimpleFIN's servers.

## Data and access

The [developer page](https://beta-bridge.simplefin.org/info/developers) asks apps to make 24 requests a day or fewer and to span at most 90 days per request; tokens that keep exceeding the warning level are disabled. History depth depends on the bank. There is no change feed: you ask for a window and get what is there now. A transaction that disappears is simply absent. All access is read-only.

## Agents

There is no official agent surface, and none is needed: one URL and one endpoint is about the easiest thing an agent can call. Community developers have published several read-only SimpleFIN MCP servers and self-hosted finance apps with MCP endpoints in 2026. The approval model is the token itself: the person mints it, gives it to one app, and can revoke it.

## Customers

The ecosystem page lists [Actual Budget, Buckets, Strata Money and others](https://www.simplefin.org/ecosystem.html); Sure also supports it. No usage figures are published, and no review-site ratings were found. The public [Bridge issue tracker](https://github.com/simplefin/bridge-issues) is candid about the costs of a thin layer over MX: Capital One ids re-issued for whole date blocks so that clients import duplicates ([September 2026](https://github.com/simplefin/bridge-issues/issues/28)), descriptions cut to 32 characters ([April 2026](https://github.com/simplefin/bridge-issues/issues/22)), and mirror transactions across linked Mercury accounts ([February 2026](https://github.com/simplefin/bridge-issues/issues/21)).

## History

The protocol dates from 2013 and was designed so banks could serve it directly without paying anyone. Few did, so the Bridge exists, and its [mission page](https://beta-bridge.simplefin.org/info/mission) says the Bridge succeeds when it is turned off. Early companions included per-bank scraping scripts and `sfin2ledger`, which turned SimpleFIN data into Ledger journals.

## For ownpurse

SimpleFIN is the bank source that matches ownpurse's shape: the person holds the credential, the data lands on their machine, and no company stands between them and the app they chose. It fits the bank-witness role well, with two cautions. First, it is downstream of MX, so it is independent of Xero but not of MX's errors; the Capital One issue shows that ids cannot be trusted alone, which the identity ladder (source id, natural key, fingerprint, position) already anticipates. Second, it has no deletes, so only a completed, verified sweep may infer one, and the 90-day window and 24-requests-a-day budget should set the connector's cadence. Keep the raw JSON in the drawer so a later fix in MX or the Bridge can be replayed.
