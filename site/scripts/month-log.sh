#!/usr/bin/env bash
# Export the synthetic month's fact log for the interactive pages on the site.
#
# Builds ownpurse-facts from this checkout, loads samples/ingest.json into a fresh log, runs the link rules, and
# writes what the browser needs to rebuild the same log with sql.js:
#   public/data/month-<commit>.json   the log's own schema.sql, every transaction and datom, the sample queries
#   public/data/month.json            the same, the latest; the docs load this one
#   public/data/*.jpg                 the receipt photo the extractor read
# <commit> is the last commit that changed rust/ or samples/, so the same model always exports to the same name. A
# post loads its own pinned file and keeps showing that version; keep old ones. Run this on purpose, after the fact
# model changes, and commit the result with the post that uses it. Needs Rust and the sqlite3 command line. Every
# record in it is synthetic (samples/README.md).
set -euo pipefail

SITE="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
ROOT="$(cd "$SITE/.." && pwd)"
OUT="$SITE/public/data"
WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT

command -v sqlite3 >/dev/null || { echo "month-log: sqlite3 is not installed" >&2; exit 1; }
[[ -z "$(git -C "$ROOT" status --porcelain -- rust samples)" ]] \
  || { echo "month-log: rust/ or samples/ has uncommitted changes; commit them so the export names a real commit" >&2; exit 1; }
COMMIT="$(git -C "$ROOT" log -1 --format=%H -- rust samples)"

(cd "$ROOT/rust" && cargo build --release --quiet -p ownpurse-facts-cli)
BIN="$ROOT/rust/target/release/ownpurse-facts"
LOG="$WORK/month.sqlite"
cd "$ROOT"
"$BIN" --log "$LOG" init >/dev/null
"$BIN" --log "$LOG" load samples/ingest.json >/dev/null
"$BIN" --log "$LOG" link >/dev/null
"$BIN" --log "$LOG" verify

mkdir -p "$OUT"
python3 - "$LOG" "$ROOT" "$COMMIT" "$OUT/month.json" <<'PY'
import json, pathlib, sqlite3, sys
log, root, commit, out = sys.argv[1:]
root = pathlib.Path(root)
db = sqlite3.connect(log)
queries = []
for f in sorted((root / "samples/queries").glob("*.sql")):
    queries.append({"file": f.name, "sql": f.read_text()})
data = {
    "commit": commit,
    "format": db.execute("SELECT value FROM meta WHERE key = 'format'").fetchone()[0],
    "schema": (root / "rust/crates/ownpurse-store/src/schema.sql").read_text(),
    "tx": db.execute("SELECT t, instant, prev, hash FROM tx ORDER BY t").fetchall(),
    "datoms": db.execute("SELECT e, a, v, vt, tx, added FROM datom ORDER BY rowid").fetchall(),
    "queries": queries,
}
pathlib.Path(out).write_text(json.dumps(data, separators=(",", ":"), ensure_ascii=False))
print(f"month-log: {len(data['tx'])} transactions, {len(data['datoms'])} datoms, {len(queries)} queries at {commit[:12]}")
PY
cp "$OUT/month.json" "$OUT/month-${COMMIT:0:7}.json"
cp samples/receipts/copper-pot-2026-03-12.jpg "$OUT/"
echo "month-log: wrote public/data/month-${COMMIT:0:7}.json and public/data/month.json"
