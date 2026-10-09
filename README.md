<p align="center"><img src="brand/logos/ownpurse-wordmark.svg" alt="ownpurse" width="320"></p>

# ownpurse

**Local books for you and your agents.** ownpurse pulls your financial data into one append-only, tamper-evident
record on your own machine, so you and your agents can query it, reconcile it and plan changes together.
Xero is the first connector; QuickBooks, bank exports, spreadsheets, portfolios and files are planned. Written in Rust.

Website and docs: [ownpurse.com](https://ownpurse.com) · [Philosophy](docs/PHILOSOPHY.md) · [Brand](brand/README.md)

> Not affiliated with or endorsed by Xero Limited. "Xero" is a trademark of Xero Limited.

## Why

Your financial life is spread across an accounting system, bank feeds, spreadsheets, portfolios and a folder of
statements. Each holds part of the truth, and none will show you the whole. ownpurse keeps one record of facts
from all of them, in a file you hold.

- **Sovereign.** The record is one SQLite file on your laptop or your own server. Reads cost no API calls and send
  nothing anywhere.
- **Append-only and tamper-evident.** Every change a source reports becomes a new immutable version; triggers refuse
  edits and deletes, and a SHA-256 chain proves nothing was changed later.
- **Exact to the cent.** Amounts keep the source's own digits as decimals. A ledger rebuilt from the documents is
  checked against Xero's trial balance, P&L and balance sheet by `parity`, `pl` and `bs`.
- **Many organisations from one login.** One PKCE consent covers every organisation you tick; address them by alias
  (`roast`, `all`, `group:<name>`), compare them side by side and match intercompany balances.
- **Built for agents.** Following the [AXI](https://axi.md/) conventions: compact output, counts on every list,
  structured errors with stable exit codes, `--json` everywhere, no interactive prompts, next-step hints. The TUI's
  agent bridge lets an agent read and drive the view you are looking at.
- **Writes you approve.** Login asks for read access. Changes are collected into manifests, planned into sealed
  packages and applied only after you approve them; every landed write leaves a note in the document's history.

## Install

```sh
git clone https://github.com/brancusi/ownpurse && cd ownpurse
./install.sh        # builds and installs ownpurse + ownpurse-tui into ~/.local/bin
ownpurse doctor     # in your project folder: checks PATH, config, record and token-file permissions
```

macOS is tested; Linux should work; Windows is not supported. The installer uses the pinned Rust toolchain (mise or
rustup) and, on first run, exports the exact [caretline](https://github.com/brancusi/caretline) commit pinned by
[`caretline.rev`](caretline.rev) into `vendor/caretline/`. See [docs/INSTALL.md](docs/INSTALL.md) for upgrade,
uninstall and first-run setup.

## Scripted demo (no account setup)

```sh
ownpurse demo                   # real TUI, mock database, automatic ten-step walkthrough
ownpurse demo --step-seconds 3  # faster playback (default: 8 seconds per step)
```

The command creates a private temporary copy of the synthetic demo database and a local proposed
correction, then opens the actual TUI. It moves through the books, document diff, reclass proposal,
checks, sealed plan and T-account, with caretline callouts, rings, a spotlight and narration.
No login, live project config or network connection is needed. Nothing is approved or applied.

## Set up

1. Create a **mobile or desktop app** (PKCE, no client secret) at
   [developer.xero.com/app/manage](https://developer.xero.com/app/manage) with redirect URI
   `http://localhost:8765/callback`.
2. Create `./ownpurse.json` from [`ownpurse.example.json`](ownpurse.example.json) and put your client id in the
   profile.
3. `ownpurse login` and tick every organisation you need.
4. Add aliases under `profiles.<name>.orgs` (`ownpurse orgs` shows the tenant ids), then `ownpurse sync all`.

## Use

| Command | Purpose |
|---|---|
| `login`, `orgs` | ⇅ consent (read scopes only) and the connected organisations |
| `sync <org\|all\|group:x> [Entity…]` | ⇅ refresh the local record (the only command that spends calls) |
| `accounts <org\|all> [--bank] [--search]` | chart of accounts / bank accounts |
| `org <org\|all>` | plan class and lock dates |
| `contacts`, `txns`, `journals`, `transfers` | list records (filters: `--from/--to`, `--search`, …) |
| `list <org\|all> <Entity> [--fields …]` | any mirrored entity: Invoices, Payments, CreditNotes, TaxRates, … |
| `show <org> <Entity> <id-prefix>` | one record with every observed version (`url`: its page in Xero) |
| `open <org> <Entity\|BankRec> <id> [--launch]` | the document's page in Xero's web app, through the org switch (local; no call) |
| `gl <org\|all> [--account] [--from/--to]` | local general ledger derived from the documents |
| `tb <org\|all> --date [--since]` | local trial balance (exact decimals) |
| `tb a,b --compare [--align code\|map]` | 2–4 orgs side by side; rows merge only by code or account_map category |
| `parity <org\|all> --date [--since]` | local trial balance vs Xero's, account by account, to the cent |
| `pl <org> --to [--from] [--cash] [--strict]` | profit and loss from the local ledger (accrual or cash basis), checked line by line against Xero's cached report for the same dates and basis |
| `bs <org> --date [--cash] [--since \| --full-history] [--strict]` | balance sheet from the local ledger, opening from Xero's cached balance sheet at the prior year-end (same basis) or from full local history, checked against Xero's |
| `report fetch <org\|all> trialbalance\|balancesheet\|pl\|banksummary [--date \| --from --to] [--cash]` | ⇅ one Xero report into the record (parity reads the latest TB); `report list` shows what is held |
| `history <org> <Entity> <id> [--cached]` | ⇅ a document's history and notes from Xero, into the record (every landed write adds `ownpurse package <hash12> op <id>: <why>` there) |
| `attachments <org> <Entity> <id> [--cached] [--get <file> --out <path>]` | ⇅ a document's attachments into the record; `--get` saves one file (never over an existing one) |
| `export <org> tb\|gl\|bs\|pl [--date \| --from --to] [--out <file>]` | a CSV for the accountant: tb / gl from the local ledger, bs / pl from Xero's cached reports (never over an existing file) |
| `ic [pair-id] [--as-of]` | intercompany pairs from config: both sides, difference, bridge, unexplained residual |
| `usage [--days N]` | API calls, remaining daily quota, endpoints, records fetched |
| `config [show\|path\|links\|edit]` | effective settings and where each came from; links |
| `doctor` | check the setup; prints a fix for every problem |
| `tui` | the terminal UI (below) |
| `ui ls\|state\|screen\|send\|upgrade` | the agent bridge to a running TUI (below) |

⇅ marks the commands that call Xero. Global flags: `--json`, `--full`, `--limit N`, `--offline` (never call
Xero), `--profile NAME`. Exit codes: `0` ok, `1` runtime error, `2` usage error, `4` refused (a view changed under
an agent, or the user is busy). Full reference: `ownpurse help` and [docs/TUI.md](docs/TUI.md).

## Terminal UI

`ownpurse tui` is read-only. Tabs: **Overview** · **Ledger** (trial balance → postings → document; Enter / Esc,
`v` compares orgs side by side) · **Intercompany** (configured org pairs: difference, bridge, residual) ·
**Activity** (the call log) · **System** (config, orgs, links). `g` picks the scope, `d` the as-of date, `/`
filters, `?` shows every key, `⌃P` opens the palette. Sync and Log in live only in the palette and confirm first
with an estimate of the calls they will use. Full reference: [docs/TUI.md](docs/TUI.md).

**Agent bridge.** An agent reads exactly what you see (`ownpurse ui state`), moves your view with view-only verbs
(`ownpurse ui send tab ledger scope roast`), and can add a caption to the footer. You see every change live, your
input always wins, and `` ` `` undoes the agent's last move. The one non-view action is `ownpurse ui upgrade`:
it swaps a running TUI for a newer installed build without losing your place.

## Configuration

Every behaviour is a setting: shipped defaults ← `ownpurse.json` (top level) ← `profiles.<name>`.
`ownpurse config` lists each value with its origin. See [docs/CONFIG.md](docs/CONFIG.md) and
[`ownpurse.example.json`](ownpurse.example.json).

## Repository

| Path | What |
|---|---|
| `rust/` | the workspace: `ownpurse-core` (ledger, config, views), `ownpurse-xero` (the Xero connector and the record), `ownpurse-cli`, `ownpurse-tui`, `xtask` |
| `docs/` | engineering docs: [PHILOSOPHY](docs/PHILOSOPHY.md), [ARCHITECTURE](docs/ARCHITECTURE.md), [CONFIG](docs/CONFIG.md), [TUI](docs/TUI.md), [ROADMAP](docs/ROADMAP.md) |
| `brand/` | the identity: logos, colour and type tokens, the brand book |
| `site/` | [ownpurse.com](https://ownpurse.com): Astro + Starlight (marketing pages, user docs, blog) |

## Development

See [CONTRIBUTING.md](CONTRIBUTING.md), [AGENTS.md](AGENTS.md) and [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md).
The gate: `rust/scripts/check.sh`.

## License

MIT. See [LICENSE](LICENSE).
