-- Phase 8 (v0.13.0), additive: what each `write verify` concluded, append-only and chained. `outcome`:
-- verified | rejected | uncertain | incomplete | discrepancy (the trial balance moved differently from the
-- package's expected deltas); `problems_json`: the reasons. Created on the first verify; frozen records
-- never see it.
CREATE TABLE IF NOT EXISTS verifications(
  id INTEGER PRIMARY KEY,
  hash TEXT NOT NULL,
  outcome TEXT NOT NULL CHECK (outcome IN ('verified','rejected','uncertain','incomplete','discrepancy')),
  problems_json TEXT NOT NULL,
  verified_at TEXT NOT NULL
) STRICT;
CREATE INDEX IF NOT EXISTS verifications_hash ON verifications(hash);
