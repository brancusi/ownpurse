
CREATE TABLE IF NOT EXISTS meta(key TEXT PRIMARY KEY, value TEXT NOT NULL) STRICT;

CREATE TABLE IF NOT EXISTS versions(
  id INTEGER PRIMARY KEY,
  tenant_id TEXT NOT NULL,
  entity TEXT NOT NULL,
  xero_id TEXT NOT NULL,
  version INTEGER NOT NULL,
  content_hash TEXT NOT NULL,
  status TEXT,
  xero_updated_utc TEXT,
  observed_at TEXT NOT NULL,
  source TEXT NOT NULL CHECK (source IN ('sync','readback','webhook','backfill')),
  sync_run_id TEXT,
  caused_by_op TEXT,
  payload TEXT NOT NULL,
  UNIQUE (tenant_id, entity, xero_id, version)
) STRICT;

CREATE TABLE IF NOT EXISTS reports(
  id INTEGER PRIMARY KEY,
  tenant_id TEXT NOT NULL,
  report TEXT NOT NULL,
  params TEXT NOT NULL,
  fetched_at TEXT NOT NULL,
  content_hash TEXT NOT NULL,
  payload TEXT NOT NULL
) STRICT;
CREATE INDEX IF NOT EXISTS reports_key ON reports(tenant_id, report, params, fetched_at);

CREATE TABLE IF NOT EXISTS sync_runs(
  id TEXT PRIMARY KEY,
  tenant_id TEXT NOT NULL,
  entity TEXT NOT NULL,
  mode TEXT NOT NULL CHECK (mode IN ('full','incremental')),
  started_at TEXT NOT NULL,
  finished_at TEXT NOT NULL,
  outcome TEXT NOT NULL CHECK (outcome IN ('ok','error')),
  calls INTEGER NOT NULL,
  fetched INTEGER NOT NULL,
  new_versions INTEGER NOT NULL,
  high_water TEXT,
  detail TEXT
) STRICT;
CREATE INDEX IF NOT EXISTS sync_runs_key ON sync_runs(tenant_id, entity, finished_at);

CREATE TABLE IF NOT EXISTS http_log(
  id INTEGER PRIMARY KEY,
  at TEXT NOT NULL,
  tenant_id TEXT,
  method TEXT NOT NULL,
  path TEXT NOT NULL,
  params TEXT,
  status INTEGER,
  duration_ms INTEGER,
  day_remaining INTEGER,
  min_remaining INTEGER,
  problem TEXT,
  op_id TEXT
) STRICT;
CREATE INDEX IF NOT EXISTS http_log_tenant ON http_log(tenant_id, at);

CREATE TABLE IF NOT EXISTS operations(
  op_id TEXT PRIMARY KEY,
  created_at TEXT NOT NULL,
  tenant_id TEXT NOT NULL,
  kind TEXT NOT NULL,
  target_entity TEXT NOT NULL,
  target_id TEXT,
  fingerprint TEXT NOT NULL UNIQUE,
  request TEXT NOT NULL,
  expected_before_version INTEGER,
  decision_refs TEXT,
  approval_ref TEXT,
  supersedes_op TEXT REFERENCES operations(op_id)
) STRICT;

CREATE TABLE IF NOT EXISTS op_events(
  id INTEGER PRIMARY KEY,
  op_id TEXT NOT NULL REFERENCES operations(op_id),
  seq INTEGER NOT NULL,
  at TEXT NOT NULL,
  state TEXT NOT NULL CHECK (state IN
    ('planned','approved','sending','confirmed','rejected','uncertain','reconciled')),
  idempotency_key TEXT,
  http_status INTEGER,
  detail TEXT,
  UNIQUE (op_id, seq)
) STRICT;

CREATE TABLE IF NOT EXISTS chain(
  seq INTEGER PRIMARY KEY,
  at TEXT NOT NULL,
  tbl TEXT NOT NULL,
  row_id TEXT NOT NULL,
  row_hash TEXT NOT NULL,
  prev_hash TEXT NOT NULL,
  hash TEXT NOT NULL
) STRICT;

-- Derived (rebuildable, mutable, outside the integrity chain): the local general ledger.
CREATE TABLE IF NOT EXISTS gl(
  tenant_id TEXT NOT NULL,
  date TEXT NOT NULL,
  source TEXT NOT NULL,
  source_id TEXT NOT NULL,
  line INTEGER NOT NULL,
  account_id TEXT NOT NULL,
  account_code TEXT,
  account_name TEXT,
  amount TEXT NOT NULL,
  amount_cents INTEGER,
  description TEXT,
  contact TEXT,
  reference TEXT
) STRICT;
CREATE INDEX IF NOT EXISTS gl_acct ON gl(tenant_id, account_id, date);
CREATE TABLE IF NOT EXISTS gl_state(
  tenant_id TEXT PRIMARY KEY,
  built_from_version_id INTEGER NOT NULL,
  built_at TEXT NOT NULL,
  postings INTEGER NOT NULL,
  unbalanced TEXT
) STRICT;

