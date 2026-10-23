# Changelog

## [Unreleased]

- Started again from zero: a ledger in Clojure on XTDB (the spike, `docs/plans/SPIKE.md`). The Rust version is
  archived at `archive/rust-main`; its changelog is `docs/archive/CHANGELOG-rust.md`.
- The workspace runs: XTDB 2.2 in-process under donut.system, `GET /health` and `GET /openapi.json`, a read-only
  SQL port for DataGrip and psql, Kaocha, clj-kondo and cljfmt.
