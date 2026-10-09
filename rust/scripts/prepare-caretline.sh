#!/usr/bin/env bash
# Export the exact local Caretline pin; never fetch, switch, or write to the source checkout.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
PIN="$(tr -d '\n' < "$ROOT/caretline.rev")"
DEST="$ROOT/vendor/caretline"

check() {
  [[ -f "$DEST/.ownpurse-caretline-rev" ]] &&
    [[ "$(< "$DEST/.ownpurse-caretline-rev")" == "$PIN" ]] &&
    [[ -f "$DEST/crates/caretline-layers/Cargo.toml" ]] &&
    [[ -f "$DEST/crates/caretline-tour/Cargo.toml" ]] &&
    [[ -f "$DEST/.ownpurse-caretline-sha256" ]] &&
    (cd "$DEST" && shasum -a 256 --status -c .ownpurse-caretline-sha256)
}

if [[ $# == 1 && "$1" == --check ]]; then
  if check; then exit 0; fi
  echo "Caretline export missing, changed, or not pinned to $PIN." >&2
  echo "Run: bash rust/scripts/prepare-caretline.sh /absolute/path/to/local/caretline" >&2
  exit 1
fi
if [[ $# != 1 ]]; then
  echo "Usage: $0 /path/to/local/caretline | --check" >&2
  exit 2
fi
SOURCE="$1"
[[ "$PIN" =~ ^[0-9a-f]{40}$ ]] || { echo "Invalid caretline.rev" >&2; exit 1; }
git -C "$SOURCE" cat-file -e "$PIN^{commit}"
if [[ -e "$DEST" || -L "$DEST" ]]; then
  if check; then echo "Caretline already prepared at $PIN"; exit 0; fi
  echo "Refusing to overwrite $DEST; preserve or remove that export explicitly first." >&2
  exit 1
fi
mkdir -p "$ROOT/vendor"
TMP="$(mktemp -d "$ROOT/vendor/.caretline-export.XXXXXX")"
trap 'rm -rf "$TMP"' EXIT
# A committed-tree archive excludes untracked/dirty source files and needs no sibling layout.
git -C "$SOURCE" archive "$PIN" | tar -xf - -C "$TMP"
[[ -f "$TMP/crates/caretline-layers/Cargo.toml" && -f "$TMP/crates/caretline-tour/Cargo.toml" ]]
(cd "$TMP" && find . -type f ! -name .ownpurse-caretline-sha256 -exec shasum -a 256 {} + > .ownpurse-caretline-sha256)
printf '%s\n' "$PIN" > "$TMP/.ownpurse-caretline-rev"
mv "$TMP" "$DEST"
echo "Prepared Caretline $PIN in $DEST"
