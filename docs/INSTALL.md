# Install and first run

## Supported platforms

| Platform | Status |
|---|---|
| macOS (Apple silicon, Intel) | supported, tested |
| Linux (x86_64, aarch64) | supported, not yet tested (CI is paused) |
| Windows | not supported (token locking uses POSIX `flock`) |

## Install (one command)

```sh
git clone https://github.com/brancusi/ownpurse && cd ownpurse
./install.sh
```

The installer checks for a Rust toolchain, builds both binaries in release mode, and installs them into `~/.local/bin`:

| Binary | What it is |
|---|---|
| `ownpurse` | the CLI: read commands over the local record (`ownpurse tui` starts the TUI) |
| `ownpurse-tui` | the terminal UI (read-only) |

At the end it checks the installed version with `ownpurse --version` and warns if `~/.local/bin` is missing from
your `PATH`. You can safely rerun it.

**Toolchain.** If [mise](https://mise.jdx.dev) is installed, the script installs the pinned Rust from `rust/mise.toml`. With
[rustup](https://rustup.rs), `rust/rust-toolchain.toml` picks the same pinned version automatically. If neither is
installed, the script says so and stops.

| Option | Effect |
|---|---|
| `--prefix DIR` | install into `DIR/bin` instead of `~/.local/bin` |
| `--uninstall` | remove both binaries |
| `--help` | usage |

## The caretline dependency

The TUI's overlay and walkthrough crates come from [caretline](https://github.com/brancusi/caretline), pinned to an
exact commit by `caretline.rev` (not a branch or "this revision or later"). On the first install, `install.sh` clones
caretline into a temporary folder and exports that commit with `git archive` into the ignored `vendor/caretline/`,
with a checksum of every file. Later installs only verify the export; a mismatched or modified export is refused,
never overwritten.

To prepare it yourself (for example to build without installing), from any caretline checkout that contains the pin:

```sh
bash rust/scripts/prepare-caretline.sh /absolute/path/to/caretline
bash rust/scripts/prepare-caretline.sh --check
```

To change the pin, update `caretline.rev`, remove the old `vendor/caretline/`, prepare again, and run the gate.
Do not commit the export.

Building without installing:

```sh
cd rust
cargo check --workspace --all-targets --locked
cargo test --workspace --locked
cargo build --release --locked -p ownpurse-cli -p ownpurse-tui
```

## Check the setup

```sh
cd path/to/your-project        # the folder holding ownpurse.json
ownpurse doctor
```

`doctor` is read-only. It checks:
- both binaries are on `PATH`;
- the config is found and lists orgs;
- the local record opens (read-only);
- the token file exists and only its owner can read it (mode 600).

Every problem comes with a fix. It never opens the token file and never prints your home folder; paths show as
`~/…`. It exits 0 when nothing fails (warnings are fine) and 1 when a check fails.

## First run

Until `ownpurse init` arrives, you create the config by hand.

1. **Create a Xero app**: a *mobile or desktop app* (PKCE, no client secret) at
   [developer.xero.com/app/manage](https://developer.xero.com/app/manage), with redirect URI
   `http://localhost:8765/callback`.
2. **Write the config**: copy `ownpurse.example.json` to `ownpurse.json` in your project folder and put your
   app's client id in the profile:
   ```json
   {"default_profile": "live",
    "profiles": {"live": {"client_id": "YOUR_XERO_APP_CLIENT_ID", "store": "db/xero.sqlite"}}}
   ```
   See [docs/CONFIG.md](CONFIG.md) for every key (org aliases, groups, limits).
3. **Log in**: tick every organisation you need. The browser opens once.
   ```sh
   ownpurse login
   ```
   Login asks for read access. If you log in again, it also re-requests the access you have already granted, so it
   never drops permissions.
4. **Name your orgs**: add aliases under `profiles.<name>.orgs`, for example `"roast": "<tenant id>"`.
   `ownpurse orgs` shows the tenant ids.
5. **First sync**:
   ```sh
   ownpurse sync all
   ```
   Every call is logged with the remaining daily budget (1,000 calls per org per day). If an org reaches its daily limit,
   the sync stops at once and keeps what it already fetched.
6. **Read**:
   ```sh
   ownpurse doctor
   ownpurse tb roast --date 2025-12-31
   ownpurse parity roast --date 2025-12-31
   ownpurse tui
   ```

## Upgrade

```sh
git pull          # or check out a release tag: git checkout <tag>
./install.sh
```

## Uninstall

```sh
./install.sh --uninstall
```

This removes the two binaries only. Your config (`ownpurse.json`), the local record (for example `db/xero.sqlite`) and
the token file are left as they are. Delete those yourself if you want them gone.
