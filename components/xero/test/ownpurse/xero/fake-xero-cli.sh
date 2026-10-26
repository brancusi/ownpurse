#!/usr/bin/env bash
# A stand-in for xero-cli in tests. It never talks to Xero: it prints saved answers from a folder.
#
#   fake-xero-cli.sh <answers dir> <exit code> <log file> <xero-cli arguments...>
#
# With exit code 0, `get <Resource>` prints <dir>/<Resource>.json (Organisation prints Organisations.json), or an
# empty list when there is no such file; `report <Name>` prints <dir>/reports/<Name>.json, or an empty report;
# `api /api.xro/2.0/<Resource>/<id>/History` prints <dir>/history/<id>.json, or no records (Attachments likewise,
# from <dir>/attachments), and Xero refuses it when <dir>/history/<id>.refused exists.
# With any other code, it prints xero-cli's error JSON on stderr and exits with that code. Each call's arguments
# are appended to the log file, one line per call.
set -euo pipefail
dir=$1 code=$2 log=$3
shift 3
echo "$*" >> "$log"

if [ "${1:-}" = "--help" ]; then
  echo "xero-cli 0.0.0-fake: a stand-in for tests."
  exit 0
fi

# xero-cli prints TOON unless asked for JSON; ownpurse must always ask.
if [ "${XERO_CLI_FORMAT:-}" != "json" ]; then
  echo '{"error":{"type":"usage","message":"The stand-in was not asked for JSON (XERO_CLI_FORMAT=json)."}}' >&2
  exit 1
fi

if [ "$code" != "0" ]; then
  echo "fetched 0 items in 1 call" >&2
  echo '{"error":{"type":"refused","message":"Xero refused the request.","hint":"Check the org id."}}' >&2
  exit "$code"
fi

if [ "$1" = "report" ]; then
  # `report <Name> --org <org> [--param k=v]...`: <dir>/reports/<Name>.json, or an empty report whose title names
  # the parameters, so that each set of parameters is a report of its own.
  report=$2
  shift 4
  params=""
  while [ $# -gt 0 ]; do params="$params $2"; shift 2; done
  if [ -f "$dir/reports/$report.json" ]; then
    cat "$dir/reports/$report.json"
  else
    echo "{\"Reports\":[{\"ReportID\":\"$report\",\"ReportName\":\"$report\",\"ReportTitles\":[\"$report\",\"${params# }\"],\"ReportDate\":\"$(date +%d%H%M%S%N)\",\"Rows\":[]}]}"
  fi
  echo "fetched in 1 call" >&2
  exit 0
fi

if [ "$1" = "api" ]; then
  IFS=/ read -r _ _ _ _ id what <<< "$2"
  sub=$(echo "$what" | tr '[:upper:]' '[:lower:]')
  if [ -f "$dir/$sub/$id.refused" ]; then
    echo '{"error":{"type":"refused","message":"Xero refused the request.","hint":"Check the id."}}' >&2
    exit 4
  fi
  if [ -f "$dir/$sub/$id.json" ]; then
    cat "$dir/$sub/$id.json"
  elif [ "$what" = "History" ]; then
    echo '{"HistoryRecords":[]}'
  else
    echo '{"Attachments":[]}'
  fi
  echo "fetched in 1 call" >&2
  exit 0
fi

resource=$2
[ "$resource" = "Organisation" ] && resource=Organisations
if [ -f "$dir/$resource.json" ]; then
  cat "$dir/$resource.json"
else
  echo "{\"$resource\":[]}"
fi
echo "fetched in 1 call" >&2
