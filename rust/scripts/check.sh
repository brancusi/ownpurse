#!/usr/bin/env bash
# The release gate: every merge and tag must pass this (CI runs it too).
#   rust/scripts/check.sh            fmt, clippy, all tests (fixture goldens, invariants, snapshots), benches
#   OWNPURSE_GOLDEN_RECORD=… OWNPURSE_GOLDEN_REAL=… rust/scripts/check.sh   + real-record goldens
#   CHECK_COVERAGE=1 rust/scripts/check.sh                                   + coverage threshold (cargo-llvm-cov)
set -euo pipefail
cd "$(dirname "$0")/.."
step() { printf '\n== %s\n' "$*"; }

step "fixtures are tracked (nothing under rust/tests/fixtures may be ignored by git)"
ignored=$(git ls-files --others --ignored --exclude-standard -- tests/fixtures | grep -Ev '\.sqlite-(wal|shm)$' || true)
if [[ -n "$ignored" ]]; then
  echo "ignored fixture files (commit them or fix .gitignore):"; echo "$ignored"; exit 1
fi
untracked=$(git ls-files --others --exclude-standard -- tests/fixtures || true)
if [[ -n "$untracked" ]]; then
  echo "untracked fixture files (commit them):"; echo "$untracked"; exit 1
fi

xtask() { cargo run --quiet --locked -p xtask -- "$@"; }

step "hygiene (no retired toolchain, no real paths, UUIDs only from uuid-allow.txt, synthetic configs)"
xtask hygiene --self-test >/dev/null
xtask hygiene

step "docs (no retired-tool references in any doc or the CLI help)"
xtask docs-grep --self-test >/dev/null
xtask docs-grep

step "fmt"
cargo fmt --all -- --check

step "clippy (warnings are errors; float arithmetic denied)"
cargo clippy --workspace --all-targets -- -D warnings

step "tests"
cargo test --workspace --quiet

step "V21 round-trip law (canonical view state: export/import/render; fixed seeds, release build)"
# OWNPURSE_V21_SOAK=<seeds> runs a longer soak
cargo test --release --quiet -p ownpurse-tui --test v21 -- --nocapture 2>&1 | grep -E "V21|test result|FAILED|panicked"

step "V22 contrast (every built-in theme x accent x surface, at the knob defaults and the computed ceilings)"
cargo test --quiet -p ownpurse-tui --lib theme::def::tests -- --nocapture 2>&1 | grep -E "test result|FAILED|panicked"

step "V23 derived theme and accent state (the accent only through state; no colour literal outside theme/)"
cargo test --quiet -p ownpurse-tui --test v0120 v23 2>&1 | grep -E "test result|FAILED|panicked"

step "V24 snapshot coverage (accent, knob, view and overlay states in every theme)"
cargo test --quiet -p ownpurse-tui --test snapshots 2>&1 | grep -E "test result|FAILED|panicked"

step "V25 cosmetic overlay (set, clear, round-trip, bad_token, never refused while typing; ui flash limits)"
cargo test --quiet -p ownpurse-tui --test v0120 v25 2>&1 | grep -E "test result|FAILED|panicked"
cargo test --quiet -p ownpurse-cli flash_tests 2>&1 | grep -E "test result|FAILED|panicked"

step "agent-drive (the real TUI in a PTY driven through the real ui CLI; fixed seeds; latency budgets at 2x)"
# OWNPURSE_LIVE_SOAK=<seeds> runs a longer soak; OWNPURSE_LIVE_KEEP=1 keeps the temp worlds
cargo build --quiet --locked --release -p ownpurse-cli -p ownpurse-tui
# the fake CLI's record writer (a live sync appends its rows to the world's record copy)
cargo build --quiet --locked --release -p ownpurse-xero --bin ownpurse-fake-record
# a second, newer TUI build for the live `ui upgrade` case (own target dir: the release build stays as is)
OWNPURSE_BUILD_VERSION=0.99.1 cargo build --quiet --locked -p ownpurse-tui --target-dir target/agent-drive
# and a build of the SAME version with another build id (a dev build of another commit)
OWNPURSE_BUILD_ID=gfeed001 cargo build --quiet --locked -p ownpurse-tui --target-dir target/agent-drive-b
OWNPURSE_LIVE_LATENCY_GATE=1 xtask agent-drive --bin-dir target/release --bumped target/agent-drive/debug/ownpurse-tui \
  --same-version target/agent-drive-b/debug/ownpurse-tui

if [[ -n "${OWNPURSE_GOLDEN_RECORD:-}" && -n "${OWNPURSE_GOLDEN_REAL:-}" ]]; then
  step "real-record goldens (to the cent)"
  cargo test --release -p ownpurse-core --test golden --test golden_ic --test golden_compare real_record -- --nocapture
else
  echo "(real-record goldens skipped: set OWNPURSE_GOLDEN_RECORD and OWNPURSE_GOLDEN_REAL)"
fi

if ls crates/*/benches/*.rs >/dev/null 2>&1; then
  step "benches (fail at 2x budget)"
  OWNPURSE_BENCH_GATE=1 cargo bench --workspace --quiet
fi

if [[ "${CHECK_COVERAGE:-0}" == "1" ]]; then
  step "coverage (core >= 85% lines)"
  cargo llvm-cov --workspace --exclude xtask --summary-only --fail-under-lines 85
fi
echo; echo "gate: OK"
