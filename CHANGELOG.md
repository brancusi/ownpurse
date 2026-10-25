# Changelog

## [Unreleased]

- Started again from zero: a ledger in Clojure on XTDB (the spike, `docs/plans/SPIKE.md`). The Rust version is
  archived at `archive/rust-main`; its changelog is `docs/archive/CHANGELOG-rust.md`.
- The workspace runs: XTDB 2.2 in-process under donut.system, `GET /health` and `GET /openapi.json`, a read-only
  SQL port for DataGrip and psql, Kaocha, clj-kondo and cljfmt.
- Money and the schemas: amounts are exact decimals with their commodity; an entry must balance in every commodity;
  a float is refused where data enters and again before anything is written. Property tests for each.
- Reading xero-axi: books, accounts, parties, documents, entries and postings from a xero-axi record and its gl,
  with where each load came from. A second load of the same record writes nothing. The sample is the invented
  Brackenridge group (`samples/xero-axi/brackenridge`); `(load-sample)` at the REPL loads it.
- Xero is now read through xero-cli, a thin command line over the Xero API that stores nothing; xero-axi is
  archived. ownpurse will keep Xero's JSON and derive the postings itself (`docs/plans/NEXT.md`, step 3b).
- The model, in `docs/MODEL.md`: source tables in a schema per provider (`xero.invoices`), shared document tables,
  and the ledger; each book has one system of record, and ownpurse is the group system of record.
- Reading Xero through xero-cli: each item is kept as one row in its `xero.*` table, valid from Xero's
  `UpdatedDateUTC`; a second pull asks only for what changed and writes nothing new. xero-cli's exit codes become
  structured errors.
- `POST /pull` fetches every org in the local config (`OWNPURSE_LOCAL_CONFIG`) and keeps what changed, so a private
  workspace can run ownpurse over its own record (`OWNPURSE_DATA_DIR`).
- Removed: the xero-axi reader, its sample, the SQLite driver and the archived loader's SQL samples.
- Tests on save at the REPL: `(watch)` reloads a saved file and runs the tests that cover it; `(last-run)` has the
  result. Calva turns it on when the folder opens.
