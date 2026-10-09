# Test gaps

Tests removed or weakened when the repository became Rust-only (v0.10.4). Each row says what the test covered and
what a Rust-native replacement needs. Each becomes an issue.

"The reference implementation" is the earlier implementation the frozen goldens and artifacts under
`rust/tests/fixtures/` were recorded from. It is no longer in this repository, so those files can be read but
never regenerated.

## Cross-implementation checks replaced by frozen data or by the code under test

| Test (crate) | What it covered | Now | A Rust-native replacement needs |
|---|---|---|---|
| `store::schema_and_views_are_verbatim` (api), **deleted** | `schema.sql` and `views.sql` were byte-identical to the reference store's schema and views text | `store::a_new_record_has_the_reference_schema` still compares a new record's `sqlite_master` (tables, indexes, triggers, views) and `meta` with the frozen `ref-empty.sqlite` | nothing for tables and triggers; for the exact view text, a frozen copy of the reference `views.sql` compared byte for byte |
| `store::appends_to_a_reference_record_verify` (api; renamed) | appends by this writer onto a reference-created record were proven by the reference implementation's own chain verifier | proven by `Writer::verify` (new), the code under test | an independent check of the chain: a frozen expected head hash for the fixed append sequence, or a second, minimal verifier written separately from `store.rs` (e.g. in a test, straight from the chain's definition) |
| `fake_record::scenarios_append_real_rows_and_the_chain_verifies` (api) | the fake CLI's record helper appends rows the reference verifier accepts | `Writer::verify` | as above |
| `sync::sync_equals_the_frozen_reference_and_agrees_on_hashes` (api; renamed) | a two-round sync gives the reference versions, sync runs and call log; a sync on top of the reference record records nothing new; the reference verifier accepts the mixed record | the same comparisons against `sync-ref.json` / `sync-ref-record.sqlite`. In v0.10.4 the fake's amounts were changed to irregular cents and the reference was REGENERATED from this syncer (`write_sync_reference`, ignored test), so it now proves stability, not agreement with a second implementation; the final check uses `Writer::verify` | an independent second check of the sync results (hand-reviewed expected rows), and an independent chain check (above) |
| `sync::xero_dates_normalise_to_utc` (api; renamed) | Xero date normalisation matched the reference decoder for 8 inputs | the same 8 inputs with their expected outputs written out | more cases from Xero's documentation (offsets, `/Date()/` forms with and without offsets, fractional seconds) |
| `auth::pkce_matches_rfc7636_and_rfc4648` (api; renamed) | base64url without padding matched the reference encoder for 6 inputs | RFC 7636 appendix B plus RFC 4648 §5 vectors | nothing |
| `auth::consent_url_is_the_frozen_one` (api; renamed) | the consent URL matched the reference builder | compared with the frozen `consent-url.txt` | nothing |
| `http::call_log_equals_the_frozen_reference_for_the_same_script` (api; renamed) | a fixed retry script logged exactly what the reference client logged | compared with the frozen `http-call-log-ref.json` | nothing |
| `token_race::concurrent_refreshers_rotate_exactly_once_per_round` (api) | half of the refresher processes used to be the reference token store on the same file and lock | six refresher processes of this build, 50 rounds; mutation-checked again for v0.10.4 (with the `flock` removed the test fails in round 0) | nothing |

## Removed with the reference implementation

| Tests | What they covered | A Rust-native replacement needs |
|---|---|---|
| The reference implementation's own suite (CLI, config, compare, intercompany, ledger, skills, TUI, retirement checks) | that implementation | nothing where the Rust crates have the behaviour (their goldens and tests cover it); the commands not yet in the Rust CLI need tests when they land: report fetch, `history`, `attachments`, `export`, `q`, `init`, `verify`, `skills`, `status` |
| Live tests (opt-in, read-only, against the Xero Demo Company with a separate login and app) | real Xero behaviour: paging, rate-limit headers, token rotation, `forbid_tenants` / `require_tenant_name` guards | a Rust live-test harness behind an explicit opt-in (`OWNPURSE_LIVE=1`), read-only, refusing to run against any tenant not named in its profile guard |

## Generators that no longer exist

| Generator | What it produced | A Rust-native replacement needs |
|---|---|---|
| Golden exporters (fixture goldens, CLI goldens, estimate goldens) | `rust/tests/fixtures/golden-*.json`, `compare-*.json`, `ic-*.json`, `cli/goldens/*.json`, `demo/estimate-*.json`, `demo/parity-*.json`, `demo/ic-*.json` | a Rust generator (an `xtask` subcommand) that writes goldens from the current code, with each regenerated golden reviewed by hand before it is committed: once the reference implementation is gone, a regenerated golden can only prove the code agrees with itself |
| Frozen reference artifacts (`OWNPURSE_FREEZE_WRITE=1`) | `rust/tests/fixtures/frozen/*` | none: these stay frozen; new behaviour gets new, hand-reviewed artifacts |

## Fixtures regenerated in v0.10.4

- Demo, engine and CLI fixtures are regenerated from Rust specs (`xtask demo`). The ledger, intercompany,
  compare, parity and estimate goldens and the CLI goldens are now regression goldens (Rust vs Rust); they were
  first frozen from the reference implementation.
- Frozen state fixtures v0.10.2 and v0.10.3: the fixture content is regenerated; the schema is unchanged.
- The API's frozen artifacts (`rust/tests/fixtures/frozen/`): the sync and call-log references use the demo
  world's names and irregular amounts; the sync reference is regenerated from this syncer (above).
