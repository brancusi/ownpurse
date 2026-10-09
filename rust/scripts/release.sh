#!/usr/bin/env bash
# Cut a release in one command, without waiting on CI:
#
#   rust/scripts/release.sh 0.17.0              bump, changelog, freeze view states, commit, tag, push
#   rust/scripts/release.sh 0.17.0 --no-push    everything except the push
#
# The changelog's `## [Unreleased]` section becomes `## [0.17.0]` (a fresh empty Unreleased goes above it) unless a
# section for the version already exists. Pushing the tag makes the release workflow publish a GitHub Release.
set -euo pipefail

v="${1:-}"; push=1
[[ "${2:-}" == --no-push ]] && push=0
[[ "$v" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]] || { echo "usage: release.sh X.Y.Z [--no-push]" >&2; exit 2; }

root="$(git rev-parse --show-toplevel)"
cd "$root"
[[ -z "$(git status --porcelain)" ]] || { echo "release.sh: commit or stash your changes first" >&2; exit 1; }
git rev-parse -q --verify "refs/tags/v$v" >/dev/null && { echo "release.sh: tag v$v already exists" >&2; exit 1; }

# changelog
if ! grep -q "^## \[$v\]" CHANGELOG.md; then
  grep -q '^## \[Unreleased\]' CHANGELOG.md || { echo "release.sh: CHANGELOG.md has no [Unreleased] section" >&2; exit 1; }
  awk -v v="$v" '!done && /^## \[Unreleased\]/ { print; print ""; print "## [" v "]"; done=1; next } { print }' \
    CHANGELOG.md > CHANGELOG.md.tmp && mv CHANGELOG.md.tmp CHANGELOG.md
fi

# version (the workspace version; Cargo.lock follows)
perl -0pi -e "s/^version = \"[^\"]+\"/version = \"$v\"/m" rust/Cargo.toml
(cd rust && cargo check --workspace --quiet)

# freeze this release's own view states (the next release's upgrade tests read them)
(cd rust && cargo test --quiet -p ownpurse-tui --test v21 write_frozen_states -- --ignored >/dev/null)

git add -A
git commit -q -m "release: v$v"
git tag -a "v$v" -m "v$v"
echo "release.sh: committed and tagged v$v"
if [[ "$push" == 1 ]]; then
  git push --follow-tags
  echo "release.sh: pushed; the release workflow publishes the GitHub Release"
fi
