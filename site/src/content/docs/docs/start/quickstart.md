---
title: Quickstart
description: Try ownpurse with no setup, then connect your Xero organisations and read your books locally.
sidebar:
  order: 3
---

## Try it with no setup

```sh
ownpurse demo
```

`demo` opens the real terminal UI over a private, temporary mock database and plays a ten-step walkthrough. It
moves through the books, a document diff, a reclass proposal, the checks, a sealed plan and a T-account, with
callouts, rings, a spotlight and narration on screen.

- No login, config or network connection is used. Nothing is approved or applied.
- `--step-seconds N` changes the pace (1 to 60 seconds per step, default 8).
- Moving the view yourself stops autoplay. Use `←` and `→` to explore.
- After the last step the TUI stays open. `q` quits and removes the mock database and its session files.

Both binaries (`ownpurse` and `ownpurse-tui`) must be installed side by side. See [Install](/docs/start/install/).

## Connect your books

Until `ownpurse init` arrives, you write the config by hand. It takes six steps.

### 1. Create a Xero app

At [developer.xero.com/app/manage](https://developer.xero.com/app/manage), create a **mobile or desktop app**.
This type uses PKCE, so there is no client secret. Set the redirect URI to exactly:

```text
http://localhost:8765/callback
```

### 2. Write the config

Copy `ownpurse.example.json` from the repository to `ownpurse.json` in your project folder, and put your app's
client id in the profile:

```json
{
  "default_profile": "live",
  "profiles": {
    "live": {"client_id": "YOUR_XERO_APP_CLIENT_ID", "store": "db/xero.sqlite"}
  }
}
```

`store` is the local record. A relative path resolves against the config file's folder. Every key is in the
[configuration reference](/docs/guides/configuration/).

### 3. Log in

```sh
ownpurse login
```

Your browser opens once. Tick every organisation you need. Login asks for read access only. If you log in again,
it also re-requests the access you already granted, so it never drops a permission.

### 4. Name your organisations

```sh
ownpurse orgs
```

`orgs` lists the connected organisations and their tenant ids (1 call). Add a short alias for each under
`profiles.<name>.orgs`:

```json
"orgs": {"roast": "<tenant id>", "hold": "<tenant id>"}
```

Aliases are what you type from now on. You can also define named groups; see
[configuration](/docs/guides/configuration/#top-level).

### 5. Sync

```sh
ownpurse sync all
```

`sync` mirrors your organisations into the local record. Every call is logged with the remaining daily budget
(Xero allows 1,000 calls per organisation per day). If an organisation reaches its daily limit, the sync stops at
once and keeps what it already fetched. Later syncs are incremental.

### 6. Read

```sh
ownpurse doctor
ownpurse tb roast --date 2025-12-31
ownpurse parity roast --date 2025-12-31
ownpurse tui
```

- `doctor` checks the setup and prints a fix for every problem.
- `tb` prints the trial balance at a date, computed from the local ledger.
- `parity` compares that trial balance with Xero's own, account by account. It needs Xero's trial balance in the
  record: `ownpurse report fetch roast trialbalance --date 2025-12-31` fetches it (1 call).
- `tui` opens the terminal UI. Navigation spends zero calls.

## Where to go next

- [Concepts](/docs/start/concepts/): what the record holds and why reads are free
- [Reports](/docs/guides/reports/): trial balance, ledger, P&L, balance sheet, parity and exports
- [Terminal UI](/docs/guides/terminal-ui/): tabs, keys and themes
- [Working with agents](/docs/guides/agents/)
