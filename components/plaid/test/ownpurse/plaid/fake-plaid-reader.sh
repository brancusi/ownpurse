#!/usr/bin/env bash
# A stand-in for plaid-reader in tests. It never talks to Plaid: it prints saved answers from a folder.
#
#   fake-plaid-reader.sh <answers dir> <exit code> <log file> <plaid-reader arguments...>
#
# With exit code 0, `item` prints <dir>/item.json, and `sync` prints <dir>/sync.json, or with --cursor <c>
# <dir>/sync-<c>.json; a missing sync file is a sync with nothing new.
# With any other code, it prints plaid-reader's error JSON on stderr and exits with that code. An exit code of the
# form <command>:<code> fails only for that command. Each call's arguments are appended to the log file.
set -euo pipefail
dir=$1 code=$2 log=$3
shift 3
echo "$*" >> "$log"

if [ "${1:-}" = "--help" ]; then
  echo "plaid-reader 0.0.0-fake: a stand-in for tests."
  exit 0
fi

# plaid-reader prints TOON unless asked for JSON; ownpurse must always ask.
case " $* " in
  *" --format json "*) ;;
  *) echo 'error:' >&2; echo '  type: usage' >&2; exit 1 ;;
esac

command=$1
case "$code" in
  *:*) [ "${code%%:*}" = "$command" ] && code=${code#*:} || code=0 ;;
esac
if [ "$code" != "0" ]; then
  echo "item: fake · calls: 1" >&2
  echo '{"error":{"type":"login-needed","message":"Plaid answered ITEM_LOGIN_REQUIRED.","hint":"plaid-reader relink fake."}}' >&2
  exit "$code"
fi

cursor=""
prev=""
for arg in "$@"; do
  [ "$prev" = "--cursor" ] && cursor=$arg
  prev=$arg
done

case "$command" in
  item) cat "$dir/item.json" ;;
  sync)
    file="$dir/sync.json"
    [ -n "$cursor" ] && file="$dir/sync-$cursor.json"
    if [ -f "$file" ]; then
      cat "$file"
    else
      echo "{\"added\":[],\"modified\":[],\"removed\":[],\"accounts\":[],\"next_cursor\":\"$cursor\",\"has_more\":false}"
    fi ;;
  *) echo '{"error":{"type":"usage","message":"Not in the stand-in."}}' >&2; exit 1 ;;
esac
