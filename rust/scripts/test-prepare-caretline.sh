#!/usr/bin/env bash
# Offline bootstrap regression tests; all repositories and exports live in a temp directory.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT
SOURCE="$TMP/source with spaces"
COPY="$TMP/xero with spaces"
mkdir -p "$SOURCE/crates/caretline-layers" "$SOURCE/crates/caretline-tour" "$COPY/rust/scripts"
cp "$ROOT/rust/scripts/prepare-caretline.sh" "$COPY/rust/scripts/"
for crate in caretline-layers caretline-tour; do
  printf '[package]\nname = "%s"\nversion = "0.1.0"\n' "$crate" > "$SOURCE/crates/$crate/Cargo.toml"
done
git -C "$SOURCE" init -q
git -C "$SOURCE" add crates
git -C "$SOURCE" -c user.name=Fixture -c user.email=fixture@example.invalid -c commit.gpgsign=false commit -qm fixture
git -C "$SOURCE" rev-parse HEAD > "$COPY/caretline.rev"
# Dirty and untracked source files must not enter the export, nor be changed by preparation.
echo '# dirty source' >> "$SOURCE/crates/caretline-tour/Cargo.toml"
echo 'untracked source' > "$SOURCE/untracked"
SCRIPT="$COPY/rust/scripts/prepare-caretline.sh"
reject() {
  if bash "$SCRIPT" "$@" > "$TMP/rejected.log" 2>&1; then
    echo "Expected rejection: $*" >&2; exit 1
  fi
}
reject --check
reject "$TMP/missing"
bash "$SCRIPT" "$SOURCE"
bash "$SCRIPT" --check
bash "$SCRIPT" "$SOURCE"
[[ ! -e "$COPY/vendor/caretline/untracked" ]]
! grep -q 'dirty source' "$COPY/vendor/caretline/crates/caretline-tour/Cargo.toml"
grep -q 'dirty source' "$SOURCE/crates/caretline-tour/Cargo.toml"
[[ -f "$SOURCE/untracked" ]]
# Changed exported files are refused, including on repeated preparation.
echo '# changed export' >> "$COPY/vendor/caretline/crates/caretline-tour/Cargo.toml"
reject --check
reject "$SOURCE"
grep -q 'changed export' "$COPY/vendor/caretline/crates/caretline-tour/Cargo.toml"
# The requested revision must exist locally; no fetch fallback.
printf '%040d\n' 0 > "$COPY/caretline.rev"
reject "$SOURCE"
echo 'prepare-caretline: all tests passed'
