-- The log. Two tables, both append-only (the triggers below refuse UPDATE and DELETE).
CREATE TABLE meta(key TEXT PRIMARY KEY, value TEXT NOT NULL) STRICT;

CREATE TABLE tx(
  t        INTEGER PRIMARY KEY,   -- the transaction's entity id
  instant  TEXT NOT NULL,         -- when the log took it (also the fact [t tx/instant instant])
  prev     TEXT NOT NULL,         -- the previous transaction's hash (64 zeros before genesis)
  hash     TEXT NOT NULL          -- sha256 over prev, t and the transaction's datoms (ownpurse-facts encode.rs)
) STRICT;

CREATE TABLE datom(
  e      INTEGER NOT NULL,
  a      INTEGER NOT NULL,        -- the attribute's entity id; its name is in the view `attr`
  v      ANY,                     -- INTEGER for refs, ints and bools; TEXT for everything else
  vt     INTEGER NOT NULL,        -- value type tag: 1 ref 2 string 3 int 4 decimal 5 bool 6 date 7 instant 8 keyword 9 hash
  tx     INTEGER NOT NULL REFERENCES tx(t),
  added  INTEGER NOT NULL CHECK (added IN (0, 1))
) STRICT;
CREATE INDEX datom_eavt ON datom(e, a, tx);
CREATE INDEX datom_avet ON datom(a, v, e);
CREATE INDEX datom_tx ON datom(tx);

CREATE TRIGGER tx_append_only_u BEFORE UPDATE ON tx BEGIN SELECT RAISE(ABORT, 'the log is append-only'); END;
CREATE TRIGGER tx_append_only_d BEFORE DELETE ON tx BEGIN SELECT RAISE(ABORT, 'the log is append-only'); END;
CREATE TRIGGER datom_append_only_u BEFORE UPDATE ON datom BEGIN SELECT RAISE(ABORT, 'the log is append-only'); END;
CREATE TRIGGER datom_append_only_d BEFORE DELETE ON datom BEGIN SELECT RAISE(ABORT, 'the log is append-only'); END;
CREATE TRIGGER meta_append_only_u BEFORE UPDATE ON meta BEGIN SELECT RAISE(ABORT, 'meta is written once'); END;

-- Views for reading the log by hand. Nothing below is stored; every view is a query over `datom`.

-- attribute id -> name
CREATE VIEW attr AS
  SELECT e AS id, v AS ident FROM datom WHERE a = 1 AND added = 1;

-- every datom, readable: the history
CREATE VIEW history AS
  SELECT d.tx, d.e, at.ident AS attr, d.v AS value, CASE d.added WHEN 1 THEN 'assert' ELSE 'retract' END AS op
  FROM datom d JOIN attr at ON at.id = d.a;

-- the facts that are true now: asserted, and not retracted by a later transaction
CREATE VIEW facts AS
  SELECT d.e, at.ident AS attr, d.v AS value, d.tx
  FROM datom d JOIN attr at ON at.id = d.a
  WHERE d.added = 1
    AND NOT EXISTS (SELECT 1 FROM datom r
                    WHERE r.e = d.e AND r.a = d.a AND r.v IS d.v AND r.added = 0 AND r.tx > d.tx);

-- one row per transaction, with who, why and from what
CREATE VIEW transactions AS
  SELECT t.t AS tx, t.instant,
    (SELECT value FROM facts WHERE e = t.t AND attr = 'tx/actor') AS actor,
    (SELECT value FROM facts WHERE e = t.t AND attr = 'tx/projector') AS projector,
    (SELECT value FROM facts WHERE e = t.t AND attr = 'tx/reason') AS reason,
    (SELECT count(*) FROM facts WHERE e = t.t AND attr = 'tx/evidence') AS files,
    (SELECT count(*) FROM datom WHERE tx = t.t AND added = 1) AS asserted,
    (SELECT count(*) FROM datom WHERE tx = t.t AND added = 0) AS retracted
  FROM tx t;

-- the drawer: every file the log knows, and whether its bytes were redacted
CREATE VIEW files AS
  SELECT f.e,
    f.value AS sha256,
    (SELECT group_concat(value, ' | ') FROM facts WHERE e = f.e AND attr = 'file/name') AS names,
    (SELECT value FROM facts WHERE e = f.e AND attr = 'file/media-type') AS media_type,
    (SELECT value FROM facts WHERE e = f.e AND attr = 'file/size') AS size,
    (SELECT value FROM facts WHERE e = f.e AND attr = 'file/redacted') AS redacted
  FROM facts f WHERE f.attr = 'file/sha256';

-- observations with their shared attributes, the account and commodity resolved to names
CREATE VIEW observations AS
  SELECT o.e,
    o.value AS id,
    max(CASE WHEN f.attr = 'obs/kind' THEN f.value END) AS kind,
    max(CASE WHEN f.attr = 'obs/date' THEN f.value END) AS date,
    max(CASE WHEN f.attr = 'obs/amount' THEN f.value END) AS amount,
    (SELECT c.value FROM facts r JOIN facts c ON c.e = r.value AND c.attr = 'commodity/code'
      WHERE r.e = o.e AND r.attr = 'obs/commodity') AS commodity,
    (SELECT n.value FROM facts r JOIN facts n ON n.e = r.value AND n.attr = 'account/name'
      WHERE r.e = o.e AND r.attr = 'obs/account') AS account,
    max(CASE WHEN f.attr = 'obs/counterparty' THEN f.value END) AS counterparty,
    max(CASE WHEN f.attr = 'obs/description' THEN f.value END) AS description,
    max(CASE WHEN f.attr = 'obs/confidence' THEN f.value END) AS confidence,
    (SELECT p.value FROM facts r JOIN facts p ON p.e = r.value AND p.attr = 'obs/id'
      WHERE r.e = o.e AND r.attr = 'obs/parent') AS parent,
    (SELECT d.value FROM facts r JOIN facts d ON d.e = r.value AND d.attr = 'doc/id'
      WHERE r.e = o.e AND r.attr = 'obs/doc') AS doc
  FROM facts o JOIN facts f ON f.e = o.e
  WHERE o.attr = 'obs/id'
  GROUP BY o.e;

-- links between any two entities, each end shown by its identity
CREATE VIEW links AS
  SELECT l.e,
    max(CASE WHEN f.attr = 'link/kind' THEN f.value END) AS kind,
    (SELECT group_concat(i.value) FROM facts r JOIN facts i ON i.e = r.value
       AND i.attr IN ('obs/id', 'entry/id', 'doc/id', 'file/sha256', 'account/id')
      WHERE r.e = l.e AND r.attr = 'link/from') AS from_id,
    (SELECT group_concat(i.value) FROM facts r JOIN facts i ON i.e = r.value
       AND i.attr IN ('obs/id', 'entry/id', 'doc/id', 'file/sha256', 'account/id')
      WHERE r.e = l.e AND r.attr = 'link/to') AS to_id,
    max(CASE WHEN f.attr = 'link/confidence' THEN f.value END) AS confidence,
    max(CASE WHEN f.attr = 'link/amount' THEN f.value END) AS amount,
    max(CASE WHEN f.attr = 'link/rule' THEN f.value END) AS rule
  FROM facts l JOIN facts f ON f.e = l.e
  WHERE l.attr = 'link/id'
  GROUP BY l.e;

-- postings with their entry, account and commodity
CREATE VIEW postings AS
  SELECT en.value AS entry,
    (SELECT value FROM facts WHERE e = en.e AND attr = 'entry/date') AS date,
    (SELECT value FROM facts WHERE e = en.e AND attr = 'entry/status') AS status,
    (SELECT n.value FROM facts r JOIN facts n ON n.e = r.value AND n.attr = 'account/name'
      WHERE r.e = p.value AND r.attr = 'posting/account') AS account,
    (SELECT value FROM facts WHERE e = p.value AND attr = 'posting/quantity') AS quantity,
    (SELECT c.value FROM facts r JOIN facts c ON c.e = r.value AND c.attr = 'commodity/code'
      WHERE r.e = p.value AND r.attr = 'posting/commodity') AS commodity,
    (SELECT value FROM facts WHERE e = p.value AND attr = 'posting/memo') AS memo
  FROM facts en
  JOIN facts p ON p.e = en.e AND p.attr = 'entry/posting'
  WHERE en.attr = 'entry/id';
