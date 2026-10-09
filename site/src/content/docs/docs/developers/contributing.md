---
title: Contributing
description: Build ownpurse from source, run the tests and the release gate, follow the design rules, and cut a release.
sidebar:
  order: 4
---

The repository is [github.com/brancusi/ownpurse](https://github.com/brancusi/ownpurse).

## Development

Everything lives under `rust/`, a Cargo workspace. The toolchain is pinned in `rust/rust-toolchain.toml` and
`rust/mise.toml`.

```sh
./install.sh   # once per clone: exports the pinned caretline commit into vendor/caretline
cd rust
cargo build --workspace
cargo test --workspace          # offline: fake Xero endpoints, the demo fixture, frozen goldens
scripts/check.sh                # the full gate (below)
```

Tests never call Xero. They run against loopback fakes of Xero's identity and Accounting endpoints, the demo fixture
(`rust/tests/fixtures/demo`, with invented organisations and ids) and frozen goldens. When you rely on a new Xero
quirk, add it to a fake and note the source (a documentation URL or the observed behaviour) in a comment.

See [Install](/docs/start/install/#the-caretline-dependency) for the caretline export the build needs.

## Design rules

- The surface grows one command at a time, driven by a real need, each with tests.
- Output follows the [AXI](https://axi.md/) conventions: compact text by default, `--json`, counts, `help[]`,
  structured errors.
- History tables are append-only, enforced by triggers. Never add an `UPDATE` or `DELETE` path.
- Money is `rust_decimal`. Float arithmetic is denied by clippy.
- The TUI never links the API crate. It reads the record and runs the CLI for anything that calls Xero.
- No write to Xero without the operations log. See [Data model](/docs/developers/data-model/#writes).

## Private data

The repository is public and works only with synthetic data and the Xero Demo Company, never real books. Before every
commit, read your diff for credentials, real records or configs, real ids, personal names, emails, local paths and
real amounts. The full list is in [Security](/docs/developers/security/#what-never-goes-in-the-repository).

## The gate

`rust/scripts/check.sh` is the release gate. It runs fixture tracking, repository hygiene, the docs grep, `cargo fmt`,
clippy, every test (fixture goldens, invariants, snapshots, the view-state round-trip law) and the benchmarks at 2×
their budget. `CHECK_COVERAGE=1` adds the coverage threshold (85% of lines).

Install the tracked pre-push hook once per clone:

```sh
git config core.hooksPath .githooks
```

`.githooks/pre-push` runs the gate and refuses the push when it fails.

The repository tools are one crate, `rust/xtask`:

| Command | What |
|---|---|
| `cargo run -q -p xtask -- hygiene [--self-test]` | No stray toolchain files, no real paths, UUIDs only from `rust/scripts/uuid-allow.txt`, synthetic configs only |
| `cargo run -q -p xtask -- docs-grep [--self-test]` | No references to retired tools in any doc or the CLI help |
| `cargo run -q -p xtask -- docs-inputs <out-dir>` | The docs-site inputs, from the demo fixture only |
| `cargo run -p xtask -- demo big <dir>` | A large synthetic record (the demo plus 10,000 documents) for benchmarks |

Never commit real accounting data, real tenant ids or real names. Fixtures use the invented demo world.

## Releases

ownpurse uses [Semantic Versioning](https://semver.org/). While it is `0.x`, minor releases may break the CLI surface;
every break is called out in the [changelog](/docs/reference/changelog/).

1. Add a section for the new version to `CHANGELOG.md`.
2. Bump `version` in `rust/Cargo.toml` (the workspace version).
3. Freeze the release's own view states:
   `cargo test -p ownpurse-tui --test v21 write_frozen_states -- --ignored` (the next release must read them).
4. Commit `release: vX.Y.Z`, tag `vX.Y.Z`, and push with the tag.
5. The release workflow runs the gate, checks that the tag matches the workspace version, and publishes a GitHub
   release with the changelog section. Install from a tag with `./install.sh`.
