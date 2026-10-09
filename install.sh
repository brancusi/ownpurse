#!/usr/bin/env bash
# Install ownpurse from a clone, in one command:
#
#   ./install.sh                 build and install ownpurse + ownpurse-tui into ~/.local/bin
#   ./install.sh --prefix DIR    install into DIR/bin instead
#   ./install.sh --uninstall     remove both binaries (the record, config and token file are never touched)
#
# Safe to rerun (it rebuilds and replaces the binaries). Needs a Rust toolchain: with mise it installs the
# pinned one (rust/mise.toml); with rustup, rust/rust-toolchain.toml selects it automatically.
set -euo pipefail

PREFIX="$HOME/.local"
ACTION=install
while [[ $# -gt 0 ]]; do
  case "$1" in
    --prefix) PREFIX="${2:?--prefix needs a folder}"; shift 2 ;;
    --prefix=*) PREFIX="${1#--prefix=}"; shift ;;
    --uninstall) ACTION=uninstall; shift ;;
    -h|--help) sed -n '2,9p' "$0" | sed 's/^# \{0,1\}//'; exit 0 ;;
    *) echo "install.sh: unknown option $1 (try --help)" >&2; exit 2 ;;
  esac
done

ROOT="$(cd "$(dirname "$0")" && pwd)"
BIN="$PREFIX/bin"
say() { printf '%s\n' "$*"; }

if [[ "$ACTION" == uninstall ]]; then
  had=()
  for b in ownpurse ownpurse-tui; do [[ -e "$BIN/$b" ]] && had+=("$b"); done
  # cargo's own bookkeeping first (so a later install starts clean), then any leftover files
  if command -v cargo >/dev/null 2>&1; then
    cargo uninstall --quiet --root "$PREFIX" ownpurse-cli >/dev/null 2>&1 || true
    cargo uninstall --quiet --root "$PREFIX" ownpurse-tui >/dev/null 2>&1 || true
  fi
  for b in ownpurse ownpurse-tui; do
    rm -f "$BIN/$b"
    if [[ " ${had[*]-} " == *" $b "* ]]; then say "removed: $BIN/$b"; else say "not installed: $BIN/$b"; fi
  done
  say "Your config, record and token file were left as they are."
  exit 0
fi

case "$(uname -s)" in
  Darwin|Linux) ;;
  *) say "install.sh: unsupported platform $(uname -s) (macOS and Linux only)" >&2; exit 1 ;;
esac

# Caretline (the TUI's overlay and walkthrough crates) is pinned by caretline.rev. The first install exports that exact
# commit from a fresh clone of the public repository into vendor/caretline; later installs only verify the export.
if ! bash "$ROOT/rust/scripts/prepare-caretline.sh" --check >/dev/null 2>&1; then
  if [[ -e "$ROOT/vendor/caretline" ]]; then
    # an export exists but does not match the pin: say why and stop (it is never overwritten)
    bash "$ROOT/rust/scripts/prepare-caretline.sh" --check
  fi
  say "== caretline: exporting the pinned commit from https://github.com/brancusi/caretline"
  src="$(mktemp -d)"
  trap 'rm -rf "$src"' EXIT
  git clone --quiet https://github.com/brancusi/caretline "$src/caretline"
  bash "$ROOT/rust/scripts/prepare-caretline.sh" "$src/caretline"
fi

cd "$ROOT/rust"
if command -v mise >/dev/null 2>&1; then
  say "== toolchain: mise (pinned in rust/mise.toml)"
  mise trust --quiet . >/dev/null 2>&1 || true
  mise install --quiet
  CARGO=(mise exec -- cargo)
elif command -v cargo >/dev/null 2>&1; then
  say "== toolchain: cargo on PATH (rust-toolchain.toml selects the pinned version via rustup)"
  CARGO=(cargo)
else
  cat >&2 <<'EOF'
install.sh: no Rust toolchain found. Install one of:
  mise    https://mise.jdx.dev        then rerun ./install.sh (it installs the pinned Rust for you)
  rustup  https://rustup.rs           then rerun ./install.sh
EOF
  exit 1
fi

want="$(sed -n 's/^version *= *"\(.*\)"/\1/p' Cargo.toml | head -1)"
say "== building ownpurse $want (release) into $BIN"
mkdir -p "$BIN"
"${CARGO[@]}" install --locked --quiet --force --root "$PREFIX" --path crates/ownpurse-tui
"${CARGO[@]}" install --locked --quiet --force --root "$PREFIX" --path crates/ownpurse-cli

got="$("$BIN/ownpurse" --version | awk '{print $2}')"
# the semver part decides; a dev build adds `+g<commit>[.dirty]` (shown, never a mismatch)
if [[ "${got%%+*}" != "$want" ]]; then
  say "install.sh: $BIN/ownpurse reports ${got%%+*}, expected $want" >&2; exit 1
fi
build=""
[[ "$got" == *+* ]] && build=" (build ${got#*+})"
say "installed: $BIN/ownpurse ${got%%+*}$build"
say "           $BIN/ownpurse-tui $("$BIN/ownpurse-tui" --version | awk '{print $2}')"

case ":$PATH:" in
  *":$BIN:"*)
    first="$(command -v ownpurse || true)"
    if [[ "$first" != "$BIN/ownpurse" ]]; then
      say "warning: \`ownpurse\` on your PATH is $first, not $BIN/ownpurse; put $BIN earlier in PATH."
    fi ;;
  *) say "warning: $BIN is not on PATH. Add to your shell profile:  export PATH=\"$BIN:\$PATH\"" ;;
esac

say ""
say "Next: run \`ownpurse doctor\` in your project folder (the one holding ownpurse.json)."
