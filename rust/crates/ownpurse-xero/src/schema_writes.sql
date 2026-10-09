-- Phase 8 (v0.13.0), additive: the owner's approvals of sealed write packages, append-only and chained like
-- every history table. Existing records gain it on their next open by a writer; nothing else changes.
CREATE TABLE IF NOT EXISTS approvals(
  id INTEGER PRIMARY KEY,
  hash TEXT NOT NULL,
  verbatim TEXT NOT NULL,
  via TEXT NOT NULL CHECK (via IN ('tty','chat','demo_standing')),
  where_given TEXT NOT NULL,
  at TEXT NOT NULL,
  scope TEXT NOT NULL,
  recorded_at TEXT NOT NULL
) STRICT;
CREATE INDEX IF NOT EXISTS approvals_hash ON approvals(hash);
