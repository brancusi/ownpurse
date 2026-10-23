# Running the TUI

Read-only. It reads the local record and `ownpurse.json`, and never writes either. Sync and Log in run the
installed `ownpurse` CLI as a child process, from the command palette, after a confirm.

## Install (one command)

    ./install.sh

From a clone of this repository: it builds with the pinned toolchain and installs `ownpurse-tui` and `ownpurse`
into `~/.local/bin` (see [INSTALL.md](INSTALL.md)).

## Run

From the project folder (the config is found by walking up from the current folder):

    cd path/to/your-project     # the folder holding ownpurse.json
    ownpurse tui                # or: ownpurse-tui

Options: `--profile live`, `--theme ledger-dark|ledger-light|ansi|no-color`, `--glyphs ascii`, `--offline`,
`--no-mouse`, `--no-agent`, `--label <text>`, `--keys markdown` (prints every key binding, your keys.toml
included), `--echo-keys` (prints the keys your terminal sends, as keys.toml spells them). Your own bindings live in
`~/.config/ownpurse/keys.toml` and reload the moment you save it (docs/TUI.md, *Your own key bindings*).

Keys: `1`–`5` tabs (Overview · Ledger · Intercompany · Activity · System), `g` scope picker, `[` `]` cycle scope,
`Enter`/`Esc` drill and back, `/` filter, `d` as-of date, `h` basis (Ledger), `!` differs from Xero (Ledger),
`v` compare orgs side by side, `?` help, `⌃P` palette, `q` quit.

WezTerm is the reference terminal (synchronized output, so redraws never tear). Terminal.app works but can tear on
large redraws.

## Without installing

    cd path/to/ownpurse/rust
    cargo run --release -p ownpurse-tui -- --config path/to/your-project/ownpurse.json
