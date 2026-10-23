# Security

## Credentials
- Uses OAuth 2.0 PKCE ("mobile or desktop" Xero app); there is no client secret to leak.
- Tokens live outside any project directory (default `~/.config/ownpurse/tokens/<profile>.json`),
  mode 0600, written atomically. Refresh tokens rotate on every use; refreshes are serialised
  with a file lock so concurrent processes cannot invalidate each other.
- Authorization codes and tokens are never logged. The loopback receiver binds 127.0.0.1 only
  and validates `state`.
- Error output never includes request headers; Xero response bodies are truncated.

## Local record
The SQLite record contains your accounting data. Keep it out of version control (the default
`.gitignore` does), on an encrypted disk, and in your backups.

## What never goes in this repository

This repository is public. Development and testing use only synthetic data and the **Xero Demo Company**, never real
books. Before every commit, people and agents check their diff against this list:

- **Credentials.** Tokens, refresh tokens, API keys, client secrets, passwords, private keys, session cookies,
  `.env` files. The token store lives in `~/.config/ownpurse/tokens/`, outside any project, and `tokens*.json`
  is ignored.
- **Records and configs from real books.** Any `*.sqlite` record (only synthetic fixtures are tracked), the
  ledger's `data/` folder, a real local config, exports, reports, attachments, statements or screenshots of real
  organisations.
- **Identifiers from real accounts.** Real tenant ids, client ids, contact or invoice ids, bank account numbers.
  Any id in a fixture, test or doc is synthetic.
- **Personal details.** Real names of people or businesses (other than Xero itself and public open-source
  projects), personal email addresses, phone numbers, home addresses, and absolute paths from a developer's machine
  (`/Users/<name>/…`). Use `~/…` and the demo names (`roast`, `hold`, Brackenridge, Tallowmere).
- **Real amounts.** Figures in fixtures, docs, posts and screenshots come from the demo data or are invented.

The gate's hygiene step checks for real paths, unknown UUIDs and non-synthetic configs; the rest is the reviewer's
job. If something private lands in a commit, do not just delete it in the next commit: tell the maintainer, rotate any
credential involved, and rewrite the history before it is pushed.

## Reporting
Report vulnerabilities privately to the maintainer (repository owner) rather than in a public issue.
