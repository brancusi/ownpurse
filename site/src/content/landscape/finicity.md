---
{
 "slug": "finicity",
 "name": "Finicity (Mastercard Open Finance)",
 "url": "https://www.finicity.com",
 "segment": "bank-aggregator",
 "status": "acquired",
 "founded": "1999 [unverified]",
 "hq": "Utah, United States (Salt Lake City area) [unverified]; parent Mastercard, Purchase, New York",
 "backing": "Owned by Mastercard (NYSE: MA). Mastercard's acquisition was announced in June 2020 and closed in late 2020; the commonly reported price, $825M plus up to $160M in earn-outs, was not confirmed against a primary source this session. By October 2026 finicity.com redirects to Mastercard's Open Finance business page.",
 "tagline": "Mastercard's US open-finance API for reading consumers' and small businesses' bank, card and investment accounts, mainly for lending and account verification.",
 "scope": "Finicity connects to US financial institutions on a consumer's consent and returns accounts, balances, transactions, account owner details, statements, and verification-of-income and verification-of-assets reports, mostly for lenders and fintechs. It also handles account verification for payments. It keeps no ledger and does no bookkeeping, and it does not let an individual sign up and pull their own data for their own use. It is a pipe, not a record.",
 "domains": [
  "bank",
  "payments",
  "integration"
 ],
 "customers_served": [
  "enterprise",
  "mid-market",
  "developers"
 ],
 "spectrum": {
  "custody": {
   "score": 1,
   "note": "The bank is the source and Finicity holds the connection; a client gets the data it fetches, but an owner never holds an authoritative copy through Finicity."
  },
  "locality": {
   "score": 0,
   "note": "A hosted API at api.finicity.com; there is no local or self-hosted mode."
  },
  "history": {
   "score": 1,
   "note": "Current balances and a window of transactions, refreshed about once a day; nothing versioned or append-only."
  },
  "openness": {
   "score": 1,
   "note": "The OpenAPI spec is public on GitHub under Apache-2.0, but production access is contract-gated and the code is closed."
  },
  "breadth": {
   "score": 1,
   "note": "One domain: financial accounts (bank, card, investment) plus derived lending reports."
  },
  "autonomy": {
   "score": 0,
   "note": "A data pipe; it writes nothing to anyone's books."
  },
  "agent_readiness": {
   "score": 2,
   "note": "Public REST API with a documented spec; Mastercard's MCP server covers developer docs only, not account data."
  },
  "integration_ease": {
   "score": 2,
   "note": "A free developer account and Test Drive sandbox are self-serve, but production needs a contract and compliance review."
  },
  "market": {
   "score": 4,
   "note": "Sold to lenders, banks, payment companies and large fintechs."
  }
 },
 "api": {
  "public": true,
  "style": "REST (JSON/XML), OpenAPI 'Open Finance' spec v1.44.1",
  "auth": "Partner ID, Partner Secret and App Key from a Mastercard Developers account; an app token that the spec says is valid for two hours (renew after 90 minutes)",
  "limits": "The spec says clients may not automate calls to the refresh services; Finicity refreshes active accounts once a day. No published per-call limits were found.",
  "cost": "Test Drive sandbox is free; production pricing is by contract and not published.",
  "export": "No owner export: a client application fetches through the API what each consumer has consented to.",
  "notes": "The spec now calls the product US Open Finance, 'formerly called Open Banking'. finicity.com and finicity.com/pricing both redirect to mastercard.com."
 },
 "pricing": {
  "model": "Enterprise contract, per product and volume",
  "entry": "Free Test Drive sandbox for development",
  "top": "Negotiated",
  "notes": "No public price list; finicity.com/pricing redirects to Mastercard's Open Finance page."
 },
 "agents": {
  "mcp": "official",
  "ai_features": "No agent features for account data were found. Mastercard publishes developers-agent-toolkit (MIT) with an MCP server, @mastercard/developers-mcp, that lets agents browse services, API operations and integration guides.",
  "agent_writes": "No. The MCP server reads documentation only; account data goes through the normal API under a contract.",
  "approval": "Consumer consent at connection time through Mastercard's Connect flow; there is no gate on agent actions because agents take none.",
  "notes": "Mastercard's agentic work (for example Agent Pay) is about payments, not data access [unverified in this pass]."
 },
 "integration": {
  "scope": "US financial institutions through direct connections and screen scraping [coverage counts not confirmed this session]; delivered through Mastercard Connect, a hosted consent flow, plus REST endpoints for accounts, transactions, statements and reports.",
  "notes": "Reachable only from a registered partner application. A person building their own local tool cannot get production keys without a contract."
 },
 "preference": {
  "scale": "Not confirmed this session; Mastercard's open-finance page returned 403 to our fetch.",
  "ratings": "Not found this session (G2 returned 403; web search budget was exhausted).",
  "sentiment": "Not gathered this session. The developer community treats it as the bank pipe behind lending and verification flows, not a consumer product."
 },
 "timeline": [
  {
   "date": "1999",
   "event": "Finicity founded in Utah [unverified]"
  },
  {
   "date": "2020-06",
   "event": "Mastercard announces it will acquire Finicity [price unverified]"
  },
  {
   "date": "2020 (late)",
   "event": "Acquisition closes [unverified exact date]"
  },
  {
   "date": "2026-10",
   "event": "finicity.com redirects to mastercard.com Open Finance; the spec renames the product US Open Finance"
  }
 ],
 "relation": "source",
 "relation_note": "A possible US bank source for an ownpurse connector, but only through a business that holds a Mastercard contract; not something an owner can connect to alone.",
 "borrow": "The token model (short-lived app token from long-lived partner credentials) and the explicit rule that refreshes are not automated: the source sets the cadence, which is how ownpurse should treat every source's limits.",
 "avoid": "Depending on a card network's pipe for an owner's own data: access is contract-gated, the brand has already been folded into Mastercard, and the owner never holds a copy.",
 "sources": [
  {
   "title": "Mastercard open-banking-us-openapi (GitHub)",
   "url": "https://github.com/Mastercard/open-banking-us-openapi"
  },
  {
   "title": "Open Finance OpenAPI spec (openbanking-us.yaml)",
   "url": "https://raw.githubusercontent.com/Mastercard/open-banking-us-openapi/main/openbanking-us.yaml"
  },
  {
   "title": "Mastercard developers agent toolkit (GitHub)",
   "url": "https://github.com/Mastercard/developers-agent-toolkit"
  },
  {
   "title": "Mastercard on GitHub",
   "url": "https://github.com/Mastercard"
  },
  {
   "title": "Mastercard Open Finance (redirect target of finicity.com)",
   "url": "https://www.mastercard.com/us/en/business/open-finance.html"
  }
 ],
 "unverified": [
  "Founding year 1999 and Utah headquarters",
  "Acquisition price ($825M plus up to $160M earn-out) and exact close date in 2020",
  "Coverage counts (institutions, share of US accounts)",
  "Production pricing and Test Drive limits",
  "Mastercard Agent Pay scope",
  "Aiia as the European counterpart",
  "G2/Capterra ratings (search budget exhausted, G2 returned 403)"
 ],
 "researched": "2026-10-10"
}
---

## What it is

Finicity is the US open-finance business of Mastercard. It connects to a consumer's or small business's financial institutions on their consent and hands the data to a lender, a payment company or a fintech app. In October 2026 the brand is mostly gone: [finicity.com](https://www.finicity.com) redirects to Mastercard's Open Finance page, and the [OpenAPI spec](https://github.com/Mastercard/open-banking-us-openapi) describes "Finicity APIs" for "US Open Finance solutions, formerly called Open Banking".

What it does well: verification. Account ownership, income and asset reports for mortgage and credit decisions are its core, and that is where a card network's reach and compliance budget are worth the most.

## How it works

A partner registers on Mastercard Developers and gets a Partner ID, Partner Secret and App Key. The partner swaps those for an app token that the spec says "is valid for two hours and is required on all calls". The consumer links accounts through Mastercard's hosted Connect flow. After that the partner reads accounts, balances, transactions, statements and reports from `api.finicity.com`. The spec says Finicity refreshes active accounts once a day, and that clients may not automate calls to the refresh services.

## Data and access

The [spec is public](https://raw.githubusercontent.com/Mastercard/open-banking-us-openapi/main/openbanking-us.yaml) (version 1.44.1, Apache-2.0), and so is a free Test Drive sandbox. Production access is by contract, and no price list is published: finicity.com/pricing redirects too. The data is current state. You get balances and a window of transactions, not a versioned history, and there is no export for owners, because the owner is not the customer.

## Agents

Mastercard publishes a [developers agent toolkit](https://github.com/Mastercard/developers-agent-toolkit) with an MCP server (`@mastercard/developers-mcp`). It lets an agent list services, read docs and inspect API operations. It does not touch anyone's account data. An agent that reads bank data through Finicity is just a partner application's code, under that partner's contract.

## Customers

Lenders, banks, payment processors and fintechs of mid-market size and up. We could not confirm coverage counts or customer numbers this session, because Mastercard's pages blocked our fetches. Review-site ratings were not reachable either.

## History

Finicity was founded in Utah, in 1999 by most accounts (unverified here). Mastercard announced in June 2020 that it would buy it, as the US half of its open-banking strategy. Aiia in Europe was the other half (unverified here). Since then the product has been absorbed into Mastercard's brand and developer platform. It is the plainest example of a card network owning the bank pipe.

## For ownpurse

Finicity is a source in theory and out of reach in practice. A person running ownpurse cannot get production keys for their own accounts. An ownpurse connector to Finicity would need a business with a Mastercard contract in the middle, and that undoes the point of an owner-held record. Two things are worth taking from it. The source decides the refresh cadence. And short-lived tokens come from long-lived secrets. For US banks, SimpleFIN or the bank's own export is the better path for an owner, and ownpurse's provenance on each fact should record which pipe a bank line came through.
