# Contributing

## Development

Everything lives under `rust/` (a Cargo workspace; the toolchain is pinned in `rust/rust-toolchain.toml` and
`rust/mise.toml`).

```sh
# once per clone: export the pinned caretline commit into vendor/caretline (./install.sh does this for you)
git clone https://github.com/brancusi/caretline /tmp/caretline
bash rust/scripts/prepare-caretline.sh /tmp/caretline
cd rust
cargo build --workspace
cargo test --workspace          # offline: fake Xero endpoints, the demo fixture, frozen goldens
scripts/check.sh                # the full gate (below)
```

Tests never call Xero. They run against loopback fakes of Xero's identity and Accounting endpoints, the demo
fixture (`rust/tests/fixtures/demo`, synthetic orgs and ids) and frozen goldens. When you rely on a new Xero
quirk, add it to a fake and note the source (doc URL or observed behaviour) in a comment.

## Design rules

Read [docs/PHILOSOPHY.md](docs/PHILOSOPHY.md) first: every change is checked against it. Visual and copy work follows
[brand/README.md](brand/README.md).


- The surface grows one command at a time, driven by a real need, each with tests.
- Output follows axi.md: compact text by default, `--json`, counts, `help[]`, structured errors.
- History tables are append-only (enforced by triggers); never add an UPDATE/DELETE path.
- Money is `rust_decimal`; float arithmetic is denied by clippy.
- The TUI never links the API crate: it reads the record and runs the CLI for anything that calls Xero.
- No write to Xero without the operations log (see docs/design.md).

## The gate and the pre-push hook

`rust/scripts/check.sh` is the release gate: fixture tracking, repository hygiene, the docs grep, fmt, clippy, every test (fixture goldens, invariants, snapshots, the V21 round-trip law) and the benches
at 2× budget. `CHECK_COVERAGE=1` adds the coverage threshold. Install the tracked hook once per clone:

    git config core.hooksPath .githooks

`.githooks/pre-push` runs the gate and refuses the push when it fails.

**Private data.** The repository is public and works only with synthetic data and the Xero Demo Company. Read
[SECURITY.md](SECURITY.md#what-never-goes-in-this-repository) before your first commit.

The repository tools are one crate, `rust/xtask`:

| Command | What |
|---|---|
| `cargo run -q -p xtask -- hygiene [--self-test]` | no stray toolchain files, no real paths, UUIDs only from `rust/scripts/uuid-allow.txt`, synthetic configs |
| `cargo run -q -p xtask -- docs-grep [--self-test]` | no references to retired tools in any doc or the CLI help |
| `cargo run -q -p xtask -- docs-inputs <out-dir>` | the docs-site inputs, from the demo fixture only |

## Releases

[Semantic Versioning](https://semver.org/). While `0.x`, minor bumps may break the CLI surface; every break is
called out in the changelog.

During rapid development nothing waits on CI. Write changes under `## [Unreleased]` in `CHANGELOG.md` as you go,
then cut a release in one command:

```sh
rust/scripts/release.sh 0.17.0             # changelog section, version bump, frozen view states, commit, tag, push
rust/scripts/release.sh 0.17.0 --no-push   # the same, without the push
```

Pushing the tag makes the `release` workflow publish a GitHub Release with the changelog section; it does not rerun
the gate. Install from a tag with `./install.sh`.

The pre-push hook runs `cargo fmt --check` and `cargo check` (about a minute when the build is warm).
`OWNPURSE_FULL_GATE=1 git push` runs the whole gate instead, and CI runs it on every push without blocking anything.
Run the full gate yourself before a release that matters: `rust/scripts/check.sh`.
