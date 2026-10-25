#!/usr/bin/env bash
# A stand-in for mercury-cli in tests. It never talks to Mercury: it prints saved answers from a folder.
#
#   fake-mercury-cli.sh <answers dir> <exit code> <log file> <mercury-cli arguments...>
#
# With exit code 0, `get <resource>` prints <dir>/<resource>.json, or for statements and credit-statements
# <dir>/<resource>/<account id>.json; a missing file is an empty list. Filters (--start, --status) are not applied:
# the answer is a superset, as a fetch may be.
# With any other code, it prints mercury-cli's error JSON on stderr and exits with that code. An exit code of the
# form <resource>:<code> fails only for that resource. Each call's arguments are appended to the log file.
set -euo pipefail
dir=$1 code=$2 log=$3
shift 3
echo "$*" >> "$log"

if [ "${1:-}" = "--help" ]; then
  echo "mercury-cli 0.0.0-fake: a stand-in for tests."
  exit 0
fi

# mercury-cli prints TOON unless asked for JSON; ownpurse must always ask.
case " $* " in
  *" --format json "*) ;;
  *) echo 'error:' >&2; echo '  type: usage' >&2; exit 1 ;;
esac

resource=$2
case "$code" in
  *:*) [ "${code%%:*}" = "$resource" ] && code=${code#*:} || code=0 ;;
esac
if [ "$code" != "0" ]; then
  echo "org: fake · calls: 1" >&2
  echo '{"error":{"type":"refused","message":"Mercury refused the request.","hint":"Check the token."}}' >&2
  exit "$code"
fi

account=""
prev=""
for arg in "$@"; do
  [ "$prev" = "--account" ] && account=$arg
  prev=$arg
done

file="$dir/$resource.json"
[ -n "$account" ] && file="$dir/$resource/$account.json"
if [ -f "$file" ]; then
  cat "$file"
else
  echo '{"statements":[],"transactions":[],"accounts":[]}'
fi
