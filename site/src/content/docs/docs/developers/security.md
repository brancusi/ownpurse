---
title: Security
description: How ownpurse handles credentials, tokens, the local record and writes, and how to report a vulnerability.
sidebar:
  order: 5
---

## Credentials

- ownpurse uses OAuth 2.0 with PKCE (a "mobile or desktop" Xero app). There is no client secret to leak.
- Authorization codes and tokens are never logged. The loopback receiver for the login redirect binds `127.0.0.1`
  only and validates `state`.
- Error output never includes request headers, and Xero response bodies are truncated.
- `ownpurse doctor` lists the granted scope names, never a token value. It never opens the token file and never
  prints your home folder.

## Tokens

- **Outside your project.** Tokens live in `~/.config/ownpurse/tokens/<profile>.json` by default (`token_file` in the
  profile changes it). Keep it outside any project folder.
- **Mode 0600.** The file is readable by its owner only. `doctor` checks this.
- **Atomic writes.** The file is replaced atomically, so a crash never leaves it half-written. It is never deleted on
  failure.
- **One refresher at a time.** Xero's refresh tokens rotate on every use. Refreshes take an exclusive cross-process
  lock (`flock`), re-read the file under it, rotate, and write it back, so concurrent processes cannot invalidate each
  other. A test races six processes for 50 rounds and requires exactly one refresh per round.
- **Unused tokens expire.** Xero's refresh token lapses after 60 days without use. The TUI reminds you after 45.

## Scopes

- Login requests **read scopes only** by default.
- Consent is additive: logging in again re-requests what you already granted, so it never silently drops a
  permission.
- Write scopes are requested only by `ownpurse login --write`, and only for a profile with `allow_writes: true`. The
  TUI never requests them.

See [Scopes](/docs/guides/configuration/#scopes) for the full list.

## Writes

Nothing reaches Xero unless three locks hold: `allow_writes` on the profile, `OWNPURSE_WRITE_RUN` set to the exact
package hash for that run, and a recorded approval of that hash by the owner. Agents cannot record an approval, and
the TUI has no code path that sends anything. See [Writes](/docs/guides/writes/).

## The agent bridge

The bridge is a Unix socket with mode 0600 in a 0700 folder. It is local only, never TCP. It accepts view verbs only:
sync, login, confirm, edits, approve, apply and quit are refused by the protocol. The state it serves never contains
token values or the token file path. Turn it off with `--no-agent` or `"tui": {"agent": false}`.

## The local record

The SQLite record contains your accounting data. Keep it out of version control, on an encrypted disk, and in your
backups. The record is append-only and hash-chained, so later tampering is detectable, but it is not encrypted by
ownpurse.

## Reporting a vulnerability

Report vulnerabilities privately to the maintainer, the owner of the
[repository](https://github.com/brancusi/ownpurse), rather than in a public issue.
