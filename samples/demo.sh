#!/usr/bin/env bash
# One month of books from every kind of source, loaded into a fact log while you watch.
#
# Pinned to a commit, so it shows that version forever:
#   curl -fsSL https://raw.githubusercontent.com/brancusi/ownpurse/<commit>/samples/demo.sh | bash -s <commit>
# From a checkout, it runs what you have:
#   bash samples/demo.sh
#
# Needs git and a Rust toolchain (https://rustup.rs). Writes only under ~/.cache/ownpurse/demo/ (or the checkout's
# rust/target/), never calls a provider, and every record in it is synthetic.
# OWNPURSE_DEMO_PACE=0 skips the pauses; OWNPURSE_DEMO_DIR picks another folder.
set -euo pipefail

PACE="${OWNPURSE_DEMO_PACE:-1.2}"
REPO="https://github.com/brancusi/ownpurse"

if [[ -t 1 ]]; then B=$'\e[1m'; D=$'\e[2m'; C=$'\e[38;2;212;64;43m'; R=$'\e[0m'; else B= D= C= R=; fi
say() { printf '%s\n' "$*"; }
step() { printf '\n%s== %s%s\n' "$B" "$*" "$R"; }
note() { printf '%s%s%s\n' "$D" "$*" "$R"; }
pause() { [[ "$PACE" == 0 ]] || sleep "$PACE"; }
fail() { printf 'demo: %s\n' "$*" >&2; exit 1; }

command -v git >/dev/null || fail "git is not installed. Install it, then run this again."
command -v cargo >/dev/null || fail "cargo is not installed. Install Rust from https://rustup.rs, then run this again."

# Where the code comes from: this checkout, or the commit named on the command line.
HERE="$(cd "$(dirname "${BASH_SOURCE[0]:-$0}")" 2>/dev/null && pwd || true)"
if [[ $# == 0 && -n "$HERE" && -f "$HERE/../rust/Cargo.toml" && -f "$HERE/ingest.json" ]]; then
  ROOT="$(cd "$HERE/.." && pwd)"
  REV="$(git -C "$ROOT" rev-parse --short=12 HEAD 2>/dev/null || echo checkout)"
  WORK="${OWNPURSE_DEMO_DIR:-$ROOT/rust/target/demo}"
else
  [[ $# == 1 && "$1" =~ ^[0-9a-f]{7,40}$ ]] || fail "name the commit to show: bash -s <commit>"
  REV="$1"
  ROOT="${OWNPURSE_DEMO_DIR:-$HOME/.cache/ownpurse/demo}/$REV"
  WORK="$ROOT/rust/target/demo"
  step "ownpurse at commit $REV"
  if [[ ! -f "$ROOT/rust/Cargo.toml" ]]; then
    note "fetching $REPO at $REV into $ROOT"
    mkdir -p "$ROOT"
    git -C "$ROOT" init --quiet
    git -C "$ROOT" fetch --quiet --depth 1 "$REPO" "$REV" || fail "could not fetch commit $REV from $REPO"
    git -C "$ROOT" -c advice.detachedHead=false checkout --quiet FETCH_HEAD
  else
    note "already fetched: $ROOT"
  fi
  if ! bash "$ROOT/rust/scripts/prepare-caretline.sh" --check >/dev/null 2>&1; then
    # The workspace also holds the TUI, which builds on caretline; the demo only needs it present.
    note "exporting the pinned caretline commit (the workspace needs it to resolve)"
    src="$(mktemp -d)"
    git clone --quiet https://github.com/brancusi/caretline "$src/caretline"
    bash "$ROOT/rust/scripts/prepare-caretline.sh" "$src/caretline" >/dev/null
    rm -rf "$src"
  fi
fi

step "Building the fact log's command line (ownpurse-facts)"
note "the first build takes a minute or two; later runs reuse it"
(cd "$ROOT/rust" && cargo build --release --quiet -p ownpurse-facts-cli)
BIN="$ROOT/rust/target/release/ownpurse-facts"
mkdir -p "$WORK"
LOG="$WORK/march.sqlite"
rm -rf "$LOG" "$WORK/march.drawer"
of() {
  printf '%s$ ownpurse-facts %s%s\n' "$C" "$*" "$R"
  "$BIN" --log "$LOG" "$@"
}
query() {
  local f="$ROOT/samples/queries/$1"
  grep '^--' "$f" | sed 's/^-- \{0,1\}//' | sed "s/^/$D/; s/\$/$R/"
  of sql -f "samples/queries/$1"
}
cd "$ROOT"

say
say "${B}Tallowmere Coffee Roasting Co., March 2026.${R} A synthetic business seen through every kind of source:"
say "Xero (two pulls), two overlapping bank downloads, OFX, a statement PDF, a check image, a receipt photo,"
say "a supplier's QuickBooks and another's FreshBooks, a budget sheet, a loan workbook, an Ethereum wallet."
note "commit $REV · every name, number and id here is invented"
pause

step "1. An empty log"
note "One SQLite file, append-only, hash-chained. Its schema is facts too."
of init
pause

step "2. Fifty files arrive, in the order a person would bring them in"
note "Each file is kept by its SHA-256 in the drawer; a projector turns it into one transaction of facts."
of load --progress samples/ingest.json >/dev/null
pause

step "3. Again: nothing changes"
note "A re-read of the same files states nothing new, so no transaction is written."
"$BIN" --log "$LOG" load samples/ingest.json | grep -c "no change" | sed 's/$/ of 50 files: no change/'
pause

step "4. Link the witnesses"
note "Sources are never merged. Rules link what they say about the same event, with a confidence."
of link
pause

step "5. Everything that backs one dinner"
query 09-what-backs-the-dinner.sql
pause

step "6. One invoice, two companies' books"
query 10-both-sides-of-an-invoice.sql
pause

step "7. Nothing is overwritten"
note "Between the two Xero pulls someone adds a reference to the dinner. Pull 2 adds that fact; pull 1's stay."
query 03-one-entity-over-time.sql
pause

step "8. Checks, not facts: does every stream carry the opening balance to the closing one?"
query 11-the-statement-reconciles.sql
pause

step "9. Where the sources pushed back on the model"
{ of pressure || true; } | head -16
note "…every report: ownpurse-facts --log $LOG pressure"
pause

step "10. Verify: re-hash every file and the whole chain"
of verify

say
say "${B}Your turn.${R} The log is at $LOG"
say "  alias of='$BIN --log $LOG'"
say "  of sql \"SELECT * FROM observations\"      views: facts history transactions files observations links postings"
say "  of history doc/id=xero:tallowmere/BankTransaction/c0ffee05-0000-4000-8000-000000000003"
say "  of --as-of 491 entity doc/id=xero:tallowmere/Invoice/c0ffee03-0000-4000-8000-000000000002   (INV-0415 before pull 2)"
say "  ls $ROOT/samples/queries     more questions, in SQL"
