#!/usr/bin/env bash
# ownpurse exactly as it was at one commit: fetch it, build the admin, and run ownpurse on an empty record, with the
# classification sandbox over invented lines. Blog posts pin it to their commit:
#
#   C=<commit>; curl -fsSL https://raw.githubusercontent.com/brancusi/ownpurse/$C/samples/demo.sh | bash -s $C
#
# Needs git, a JDK 21 or later, the Clojure CLI and Node. Writes under ~/.cache/ownpurse/demo/ and a temporary folder
# for the empty record; never talks to a provider. OWNPURSE_PORT picks another port than 7070.
set -euo pipefail
C=${1:?"usage: demo.sh <commit>"}
for tool in git java clojure node npm; do
  command -v "$tool" >/dev/null || { echo "This demo needs $tool. See https://ownpurse.com/docs/developers/clojure/"; exit 1; }
done
dir="$HOME/.cache/ownpurse/demo/$C"
if [ ! -d "$dir/.git" ]; then
  echo "Fetching ownpurse at $C into $dir"
  git clone --quiet https://github.com/brancusi/ownpurse.git "$dir"
fi
cd "$dir"
git -c advice.detachedHead=false checkout --quiet "$C"
echo "Building the admin and starting ownpurse (the first run downloads its libraries: a few minutes)."
exec bin/admin demo
