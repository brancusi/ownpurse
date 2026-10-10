---
{
 "slug": "hledger-mcp",
 "name": "hledger MCP",
 "url": "https://github.com/iiAtlas/hledger-mcp",
 "segment": "agent-native",
 "status": "active",
 "founded": "2025-09-22 (repository created; v1.0.0 published to npm the same day)",
 "hq": "Not stated; a single-author open-source project by the GitHub user iiAtlas",
 "backing": "None. A personal open-source project with no company, funding or sponsor found. hledger itself is community free software led by Simon Michael since 2007, who contributed one README fix to this server (merged 2026-02-28).",
 "tagline": "A local MCP server that lets an AI assistant run hledger reports on, and optionally edit, your plain-text journal.",
 "scope": "It wraps the hledger CLI as about two dozen MCP tools over stdio: accounts, balance, register, print, the financial statements, stats and tag queries, plus add, find, remove, replace, import, close and rewrite for the journal, and starting hledger-web. It exposes the journal and every included file as MCP resources. It does not fetch from banks or accounting systems, keep any history beyond .bak backups, or hold anything except the journal you point it at.",
 "domains": [
  "ledger",
  "personal",
  "reporting"
 ],
 "customers_served": [
  "consumer",
  "freelancer",
  "developers"
 ],
 "spectrum": {
  "custody": {
   "score": 4,
   "note": "The journal is a text file on your disk and is the only record; the server keeps nothing of its own."
  },
  "locality": {
   "score": 4,
   "note": "Runs on your machine over stdio via npx or a .mcpb Claude Desktop bundle; Glama lists it as local only and not deployable."
  },
  "history": {
   "score": 1,
   "note": "The journal is edited in place (remove and replace rewrite text); the only history is .bak and timestamped backups, or git if you add it yourself."
  },
  "openness": {
   "score": 4,
   "note": "MIT-licensed TypeScript source (relicensed from GPL to MIT in v1.0.4, 2025-09-24) over GPL hledger."
  },
  "breadth": {
   "score": 1,
   "note": "One hledger journal tree; CSV and other files come in only through hledger import with a rules file you write."
  },
  "autonomy": {
   "score": 3,
   "note": "With read-only off, an agent can append, delete, replace and rewrite entries directly; the only gates are dryRun if the agent chooses it, hledger check, and the client's tool-call confirmation."
  },
  "agent_readiness": {
   "score": 4,
   "note": "An MCP server is the whole product: typed tools, resources, a read-only switch and dry runs, though no finer scopes than on/off."
  },
  "integration_ease": {
   "score": 3,
   "note": "One npx line in an MCP client config, but you need hledger (and hledger-web for the web tools) and Node 18+ installed first."
  },
  "market": {
   "score": 0,
   "note": "Individuals and developers who already keep hledger journals."
  }
 },
 "api": {
  "public": true,
  "style": "MCP over stdio (tools and resources), shelling out to the hledger CLI with spawn",
  "auth": "None; local process with the permissions of the user who runs it. Read-only via --read-only or HLEDGER_READ_ONLY=true.",
  "limits": "No rate limits. hledger commands time out after 30 seconds by default (executor.ts).",
  "cost": "Free. No metering, no egress; the cost is whatever the model provider charges for the agent.",
  "export": "Nothing to export: the journal is already a plain-text file you own, readable by hledger, and convertible to CSV, JSON and other formats with hledger's own output options.",
  "notes": "The tool surface mirrors hledger commands, so its fidelity is hledger's: exact decimal amounts, balance assertions, include files. Published as @iiatlas/hledger-mcp on npm (latest 1.0.5, 2025-09-25)."
 },
 "pricing": {
  "model": "Free and open source (MIT)",
  "entry": "$0",
  "top": "$0",
  "notes": "No paid tier, hosted service or support offering."
 },
 "agents": {
  "mcp": "community",
  "ai_features": "None of its own; it gives any MCP client (Claude Desktop and others) hledger's reports and journal edits as tools. The intelligence is the client's model.",
  "agent_writes": "Yes, unless read-only: hledger_add_transaction, hledger_remove_entry, hledger_replace_entry, hledger_import, hledger_rewrite and hledger_close write to the primary journal. Each validates with hledger check; import and rewrite run on a temporary copy first.",
  "approval": "No approval gate of its own. Every write tool takes dryRun to preview a diff, but the agent decides whether to use it. In read-only mode the write tools stay registered but refuse anything other than a dry run, and hledger-web is forced to view mode. Backups (.bak before appends, timestamped for imports) unless --skip-backup.",
  "notes": "The read-only mode is real and enforced in code (each write tool checks readOnly && !dryRun), and the author recommends starting with it. It is a binary switch, not a sealed package a person reviews and approves."
 },
 "integration": {
  "scope": "Any MCP client that runs stdio servers; any data hledger can read (journal, timeclock, timedot, CSV with rules). No bank, payment or accounting-system connectors.",
  "notes": "Setup is a config entry pointing at a master journal; include'd files are followed and exposed as resources. Dockerfiles exist in a separate community repo (adept/hledger-mcp-docker, 2026-02-22)."
 },
 "preference": {
  "scale": "65 GitHub stars, 5 forks, 2 open issues, 82 commits (GitHub, checked 2026-10-10); 107 npm downloads from 2026-09-09 to 2026-10-08. It is the most-starred of about ten hledger MCP repositories on GitHub; the others have 0 or 1 stars.",
  "ratings": "No G2, Capterra, Trustpilot or app store listings. Glama lists it as MIT, local only, maintenance \"Inactive\".",
  "sentiment": "Not found: forum and Reddit searches could not be run in this pass. The only outside signal found is hledger's creator contributing a README fix, and other projects building on it (an agent framework repo, 2026-09-03, names hledger-mcp as a component)."
 },
 "timeline": [
  {
   "date": "2007",
   "event": "hledger first released by Simon Michael"
  },
  {
   "date": "2025-07",
   "event": "Earlier experimental hledger MCP servers appear (Rigellute, kristianfreeman), each with about 1 star"
  },
  {
   "date": "2025-09-22",
   "event": "Repository created and v1.0.0 published to npm"
  },
  {
   "date": "2025-09-23",
   "event": "v1.0.2: env-var config, read-only and skip-backup variables, .mcpb Claude Desktop bundle, Glama listing"
  },
  {
   "date": "2025-09-23",
   "event": "v1.0.3: hledger-web start, list and stop tools"
  },
  {
   "date": "2025-09-24",
   "event": "v1.0.4: license changed from GPL to MIT"
  },
  {
   "date": "2025-09-25",
   "event": "v1.0.5: find, remove and replace entry tools; latest release"
  },
  {
   "date": "2026-02-28",
   "event": "Last commit: README fix from hledger's creator merged"
  }
 ],
 "relation": "kin",
 "relation_note": "The nearest thing to ownpurse among plain-text tools: local, owner-held, open, agent-facing over MCP. It differs where ownpurse is built: it has no connectors, no raw record from sources, no history beyond backups, and agent writes land in the journal directly rather than as an approved package.",
 "borrow": "A read-only mode as the recommended default, enforced in each write tool rather than by hiding them; dryRun on every write returning a diff; validating with the ledger's own checker before committing; working on a temporary copy for bulk changes; exposing source files as MCP resources; shipping a one-file Claude Desktop bundle.",
 "avoid": "Letting the agent choose whether to preview; editing and deleting entries in place so the previous state survives only in a .bak; one on/off switch as the only scope; a single maintainer with releases stopped after three days of work, which leaves users on an unmaintained wrapper.",
 "sources": [
  {
   "title": "iiAtlas/hledger-mcp on GitHub",
   "url": "https://github.com/iiAtlas/hledger-mcp"
  },
  {
   "title": "hledger-mcp README",
   "url": "https://github.com/iiAtlas/hledger-mcp/blob/main/README.md"
  },
  {
   "title": "hledger-mcp CHANGELOG",
   "url": "https://github.com/iiAtlas/hledger-mcp/blob/main/CHANGELOG.md"
  },
  {
   "title": "GitHub API: repository metadata",
   "url": "https://api.github.com/repos/iiAtlas/hledger-mcp"
  },
  {
   "title": "GitHub API: releases",
   "url": "https://api.github.com/repos/iiAtlas/hledger-mcp/releases"
  },
  {
   "title": "@iiatlas/hledger-mcp on npm",
   "url": "https://www.npmjs.com/package/@iiatlas/hledger-mcp"
  },
  {
   "title": "npm downloads, last month",
   "url": "https://api.npmjs.org/downloads/point/last-month/@iiatlas/hledger-mcp"
  },
  {
   "title": "Glama MCP listing",
   "url": "https://glama.ai/mcp/servers/@iiAtlas/hledger-mcp"
  },
  {
   "title": "hledger",
   "url": "https://hledger.org/"
  },
  {
   "title": "GitHub search: hledger mcp repositories",
   "url": "https://api.github.com/search/repositories?q=hledger+mcp"
  }
 ],
 "unverified": [
  "User sentiment on Reddit, Hacker News or the hledger forums: web search budget was exhausted, so none was gathered.",
  "The author's identity and location (GitHub user iiAtlas) were not looked up.",
  "Whether the .mcpb bundle is listed in any official Claude Desktop extension directory.",
  "How many people use it beyond the 107 npm downloads in 30 days; npx runs and .mcpb installs are not counted separately.",
  "Whether it works with the latest hledger release; the README states no minimum hledger version."
 ],
 "researched": "2026-10-10"
}
---

## What it is

[hledger-mcp](https://github.com/iiAtlas/hledger-mcp) is a small open-source MCP server that puts [hledger](https://hledger.org/), the plain-text double-entry accounting tool Simon Michael first released in 2007, in front of an AI assistant. You point it at your master journal and your MCP client (Claude Desktop, or anything that runs stdio servers) can run balance, register and the financial statements, query payees and tags, and, if you allow it, add, replace, remove, import and rewrite transactions. It is MIT licensed, written in TypeScript and published on [npm](https://www.npmjs.com/package/@iiatlas/hledger-mcp) as `@iiatlas/hledger-mcp`.

## How it works

Each tool spawns the hledger CLI with sanitised arguments and returns its output, with a 30-second timeout. The journal and every file `hledger files` reports become MCP resources, so includes are followed. The write tools are careful in the ways a single file allows: each validates with `hledger check` before committing, import and rewrite run against a temporary copy, `.bak` files are written before appends and timestamped backups before imports, and every write takes a `dryRun` flag that returns a diff. A `--read-only` flag (or `HLEDGER_READ_ONLY=true`) makes every write tool refuse anything but a dry run and forces hledger-web into view mode; the [README](https://github.com/iiAtlas/hledger-mcp/blob/main/README.md) recommends starting there. The check is in the code of each write tool, not just in the docs.

## Data and access

There is no API to pay for and no data to export: your journal is already a text file you hold, and hledger can print it as CSV or JSON. What it cannot do is fetch anything. Bank and card data come in only through `hledger import` with a CSV rules file you write, and hledger's import remembers what it has seen by date per file, which the earlier ownpurse survey flagged as fragile identity. Nothing records what a source said; the journal holds normalized entries only, and `remove` and `replace` edit it in place.

## Agents

This is what it is for, and it does it well for its size: typed tools, resources, a real read-only mode, dry runs with diffs, validation by the ledger's own checker. The gap is approval. With read-only off, an agent writes straight to your books; whether it previews first is the agent's choice, and the only human gate is the MCP client's tool-call prompt. Read-only is all or nothing. There is no package of proposed changes a person reviews and seals, and no log of what the agent did beyond backups.

## Customers

People who already keep hledger journals and use an MCP client: individuals, freelancers, developers. Scale is modest. On 2026-10-10 the repository had 65 stars, 5 forks and 2 open issues, and npm counted 107 downloads in the 30 days to 2026-10-08. It is the most-starred of roughly ten hledger MCP repositories on GitHub. There are no review-site listings; [Glama](https://glama.ai/mcp/servers/@iiAtlas/hledger-mcp) marks its maintenance as inactive. I could not run forum searches in this pass, so user sentiment is unverified.

## History

The whole release history fits in four days: [v1.0.0](https://github.com/iiAtlas/hledger-mcp/blob/main/CHANGELOG.md) on 2025-09-22, the Claude Desktop bundle and environment variables in v1.0.2, hledger-web tools in v1.0.3, a switch from GPL to MIT in v1.0.4 on 2025-09-24, and find, remove and replace in v1.0.5 on 2025-09-25. Since then there have been README commits only, the last a fix from hledger's creator merged on 2026-02-28. Earlier attempts (Rigellute and kristianfreeman, July 2025) and later ones (2026) exist but have little use.

## For ownpurse

This is plain-text kin with an agent surface, and the clearest proof that people want an agent near their own books without a vendor in between. It shares ownpurse's custody, locality and openness. It shows what a single mutable file cannot give you: no record of what each source reported, no history beyond backups, no as-of question, and agent writes that land directly. Take its defaults: read-only first, enforced in every write tool; a dry run that returns a diff; the ledger's own check before anything is written. ownpurse's answer to the rest is the append-only log and the sealed package a person approves. hledger users are also a natural audience, and an export from ownpurse's ledger to an hledger journal would let this server and the hledger tools read ownpurse's books.
