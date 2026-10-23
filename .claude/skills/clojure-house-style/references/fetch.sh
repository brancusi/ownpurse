#!/usr/bin/env bash
# Fetch XTDB's documentation for language models into this folder. The files are not committed (XTDB's docs,
# not ours); run this once per clone, and again when XTDB is upgraded.
set -euo pipefail
cd "$(dirname "$0")"
base=https://docs.xtdb.com
curl -fsSL "$base/llms-small.txt" -o xtdb-small.txt
curl -fsSL "$base/_llms-txt/xtql-reference.txt" -o xtdb-xtql.txt
curl -fsSL "$base/_llms-txt/sql-reference.txt" -o xtdb-sql.txt
wc -c xtdb-*.txt
