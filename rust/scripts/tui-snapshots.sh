#!/usr/bin/env bash
# Guide-style renders of every TUI shell state plus the Activity/System tab states (80×24, 100×30, 140×40),
# from the committed demo fixture with the clock pinned, so ages never drift. Zero network.
#   rust/scripts/tui-snapshots.sh <out-dir> [theme] [state-prefix,...] [formats]
# formats: comma list of txt (plain), html (<pre> with inline colours per style run) and ansi (SGR escapes;
# `cat` it in a terminal). txt alone writes into <out-dir>; several formats write <out-dir>/{txt,html,ansi}/.
# GLYPHS=ascii switches the glyph set (default unicode).
set -euo pipefail
cd "$(dirname "$0")/.."
out="${1:?usage: tui-snapshots.sh <out-dir> [theme]}"
theme="${2:-ledger-dark}"
only="${3:-}"
formats="${4:-txt}"
glyphs="${GLYPHS:-unicode}"
cfg=tests/fixtures/demo/ownpurse.json
[[ -f "$cfg" ]] || { echo "missing $cfg (cargo run -p xtask -- demo fixtures)" >&2; exit 1; }
cargo build --quiet -p ownpurse-tui --bin ownpurse-tui
# the net-done-* states run the fake CLI, which appends to a temp copy of the record with this helper
cargo build --quiet -p ownpurse-xero --bin ownpurse-fake-record
mkdir -p "$out"
env -i HOME=/nonexistent PATH="$PATH" LANG=en_US.UTF-8 OWNPURSE_FAKE_WRITER="$PWD/target/debug/ownpurse-fake-record" \
  target/debug/ownpurse-tui --config "$cfg" --profile demo \
  --now 2026-10-05T09:46Z --theme "$theme" --glyphs "$glyphs" \
  --dump-states "$out" --sizes 80x24,100x30,140x40 --only "$only" --format "$formats" </dev/null
# every render has a header with READ-ONLY (V5 smoke) and none is empty
txt="$out"; [[ "$formats" == "txt" ]] || txt="$out/txt"
[[ ",$formats," == *,txt,* ]] || exit 0
for f in "$txt"/*.txt; do
  [[ -s "$f" ]] || { echo "empty render: $f" >&2; exit 1; }
  case "$f" in *too-small*) continue ;; esac
  head -1 "$f" | grep -q "READ-ONLY" || { echo "header missing READ-ONLY: $f" >&2; exit 1; }
done
