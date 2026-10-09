---
title: Install
description: Build and install the ownpurse CLI and terminal UI, check the setup, upgrade and uninstall.
sidebar:
  order: 2
---

:::note[Early access]
ownpurse is in early development. macOS is the tested platform. Commands and file formats can change between
`0.x` releases; every change is listed in the [changelog](/docs/reference/changelog/).
:::

## Supported platforms

| Platform | Status |
|---|---|
| macOS (Apple silicon, Intel) | Supported, tested |
| Linux (x86_64, aarch64) | Supported, not yet tested |
| Windows | Not supported (token locking uses POSIX `flock`) |

## Install from source

ownpurse is built from source with one script. Clone the repository and run the installer:

```sh
git clone https://github.com/brancusi/ownpurse.git
cd ownpurse
./install.sh
```

The installer checks for a Rust toolchain, builds both binaries in release mode, and installs them into
`~/.local/bin`:

| Binary | What it is |
|---|---|
| `ownpurse` | The CLI: every command over the local record. `ownpurse tui` starts the terminal UI |
| `ownpurse-tui` | The terminal UI (read-only) |

At the end it checks the installed version with `ownpurse --version` and warns if `~/.local/bin` is missing from
your `PATH`. You can safely run it again.

### Options

| Option | Effect |
|---|---|
| `--prefix DIR` | Install into `DIR/bin` instead of `~/.local/bin` |
| `--uninstall` | Remove both binaries |
| `--help` | Usage |

### Toolchain

The Rust version is pinned. If [mise](https://mise.jdx.dev) is installed, the script installs the pinned Rust
from `rust/mise.toml`. With [rustup](https://rustup.rs), `rust/rust-toolchain.toml` selects the same version
automatically. If neither is installed, the script says so and stops.

### The caretline dependency

The terminal UI's overlay and walkthrough crates come from [caretline](https://github.com/brancusi/caretline),
which is not published to crates.io yet. `caretline.rev` pins the exact commit. On the first install, `install.sh`
clones caretline into a temporary folder and exports that commit into the ignored `vendor/caretline/` folder, with a
checksum of every file. Later installs only verify the export, and a modified or mismatched export is refused
instead of overwritten.

To prepare it yourself, from any caretline clone that contains the pinned commit:

```sh
bash rust/scripts/prepare-caretline.sh /path/to/caretline
bash rust/scripts/prepare-caretline.sh --check
```

## Check the setup

From the folder that holds your `ownpurse.json`:

```sh
cd path/to/your-project
ownpurse doctor
```

`doctor` is read-only. It checks that:

- both binaries are on `PATH`;
- the config is found and lists organisations;
- the local record opens (read-only);
- the token file exists and only its owner can read it (mode 600).

Every problem comes with a fix. `doctor` never opens the token file and never prints your home folder; paths show
as `~/…`. It exits `0` when nothing fails (warnings are fine) and `1` when a check fails.

## Upgrade

```sh
git pull          # or check out a release tag: git checkout <tag>
./install.sh
```

A running terminal UI notices the newer build and offers to upgrade in place, with your view kept. See
[In-place upgrade](/docs/guides/terminal-ui/#in-place-upgrade).

## Uninstall

```sh
./install.sh --uninstall
```

This removes the two binaries only. Your config (`ownpurse.json`), the local record (for example
`db/xero.sqlite`) and the token file stay where they are. Delete them yourself if you want them gone.

## Next

[Quickstart](/docs/start/quickstart/): run the demo, then connect your books.
