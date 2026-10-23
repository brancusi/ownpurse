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
