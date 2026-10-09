-- Phase 8 (v0.13.0), additive: the sealed write packages this record has seen (registered at `write seal`
-- and again by `write run` as a backstop), append-only and chained. `source_ref`: the manifest or branch a
-- package was compiled from, when there is one. Created on the first seal; frozen records never see it.
CREATE TABLE IF NOT EXISTS packages(
  id INTEGER PRIMARY KEY,
  hash TEXT NOT NULL,
  package_id TEXT NOT NULL,
  title TEXT NOT NULL,
  profile TEXT NOT NULL,
  ops INTEGER NOT NULL,
  ops_json TEXT NOT NULL,
  source_ref TEXT,
  sealed_at TEXT NOT NULL
) STRICT;
CREATE INDEX IF NOT EXISTS packages_hash ON packages(hash);
