#!/usr/bin/env bash
# Kept for old instructions: the installer lives at the repo root (./install.sh --help).
exec "$(cd "$(dirname "$0")/../.." && pwd)/install.sh" "$@"
