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

## Reporting
Report vulnerabilities privately to the maintainer (repository owner) rather than in a public issue.
