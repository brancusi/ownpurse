# Security

## Credentials
- ownpurse never holds a provider's token and never calls a provider. It runs each source's tool (xero-cli for
  Xero), which logs in, keeps its own tokens outside any project (xero-cli: `~/.config/xero-cli`) and talks to the
  provider.
- Tests never run a source's tool against a provider; they use a stand-in that prints synthetic answers.

## Local record
The record (an XTDB store, `data/` by default or `OWNPURSE_DATA_DIR`) holds your accounting data. Keep it out of
version control (the default `.gitignore` does), on an encrypted disk, and in your backups. The SQL port it serves is
read-only and listens on 127.0.0.1 only.

Real books are best kept in a private workspace outside this repository, which runs ownpurse over its own record and
config. This repository then never sees them.

## What never goes in this repository

This repository is public. Development and testing use only synthetic data and the **Xero Demo Company**, never real
books. Before every commit, people and agents check their diff against this list:

- **Credentials.** Tokens, refresh tokens, API keys, client ids and secrets, passwords, private keys, session
  cookies, `.env` files. A source's tool keeps its tokens outside any project, and `tokens*.json` is ignored.
- **Records and configs from real books.** The record's `data/` folder, `config.local.edn` or any config naming
  real orgs, exports, reports, attachments, statements or screenshots of real
  organisations.
- **Identifiers from real accounts.** Real tenant ids, client ids, contact or invoice ids, bank account numbers.
  Any id in a fixture, test or doc is synthetic.
- **Personal details.** Real names of people or businesses (other than Xero itself and public open-source
  projects), personal email addresses, phone numbers, home addresses, and absolute paths from a developer's machine
  (`/Users/<name>/…`). Use `~/…` and the demo names (`roast`, `hold`, Brackenridge, Tallowmere).
- **Real amounts.** Figures in fixtures, docs, posts and screenshots come from the demo data or are invented.

Check the diff for real paths, unknown UUIDs and non-synthetic configs before every commit. If something private lands in a commit, do not just delete it in the next commit: tell the maintainer, rotate any
credential involved, and rewrite the history before it is pushed.

## Reporting
Report vulnerabilities privately to the maintainer (repository owner) rather than in a public issue.
