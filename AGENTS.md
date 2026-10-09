# Working on ownpurse

Guidance for coding agents (and people) working in this repository. Read [docs/PHILOSOPHY.md](docs/PHILOSOPHY.md)
before changing behaviour, and [brand/README.md](brand/README.md) before changing anything visual or any copy.

## What this is

ownpurse is a local accounting tool for people and their agents: connectors append what each source reports to one
append-only, hash-chained SQLite record; the ledger is rebuilt locally to the cent and checked against the source;
the CLI (agent-first) and the TUI (built on caretline, Elm architecture) read the record; writes go out only as
sealed packages a person approves. Xero is the first connector. The project is accounting-system agnostic: never
add Xero assumptions to code that is not the Xero connector.

## Layout

- `rust/`: the Cargo workspace. `ownpurse-core` (ledger, config, views; no I/O to providers), `ownpurse-xero`
  (the Xero connector, auth, sync, the record), `ownpurse-cli` (the `ownpurse` binary), `ownpurse-tui` (the
  `ownpurse-tui` binary; never links `ownpurse-xero`, it runs the CLI for anything that calls a provider), `xtask`
  (hygiene, docs grep, demo fixtures, agent-drive tests).
- `docs/`: engineering docs. `brand/`: the identity and tokens. `site/`: ownpurse.com (Astro + Starlight).
- `vendor/caretline/`: an ignored export of the caretline commit pinned in `caretline.rev`. Never commit it.

## Names and words

- The product is always **ownpurse**: lowercase, one word, even at the start of a sentence. Binaries `ownpurse` and
  `ownpurse-tui`; config `ownpurse.json`; config folder `~/.config/ownpurse/`; environment variables `OWNPURSE_*`.
- Say books, accounts, entries, sources, facts. Not tenants, entities or rows in user-facing text.
- Copy is plain and exact: second person, sentence case, numbers to the cent, no emoji, no hype ("seamless",
  "magic", "AI-powered"). Errors say what went wrong and how to fix it.
- "Xero" is Xero Limited's trademark: keep the "not affiliated" line wherever the name is used prominently.

## Private data

The repository is public. Work only with synthetic fixtures and the Xero Demo Company, never real books. Before
every commit, read your diff for credentials, real records or configs, real ids, personal names, emails, paths and
real amounts. The full list is in [SECURITY.md](SECURITY.md#what-never-goes-in-this-repository). Never commit a
`*.sqlite` record, a real `ownpurse.json`, a token file or an `.env` file.

## Rules that do not bend

- History tables are append-only (triggers enforce it). Never add an UPDATE or DELETE path to history.
- Money is `rust_decimal`. Float arithmetic is denied by clippy.
- Nothing reaches a provider except `sync`, `login` and the other commands marked ⇅, and writes only through a
  sealed, approved package recorded in the operations log.
- Tests never call a provider: use the loopback fakes and the synthetic demo fixture. Fixtures are generic; real
  names, amounts and ids never enter the repository.
- Output follows AXI: compact text, `--json`, counts on lists, `help[]` hints, structured errors, stable exit codes.
- Everything built from the mark is symmetric. The coin (`#D4402B`) is spent sparingly.

## Checks

```sh
cd rust
cargo test --workspace --locked
scripts/check.sh                 # the release gate: hygiene, docs grep, fmt, clippy, tests, laws, benches
```

Install the pre-push hook once per clone: `git config core.hooksPath .githooks`.

TUI snapshots: `INSTA_UPDATE=always cargo test -p ownpurse-tui --test snapshots`, then review every changed
`.snap` before committing.

## The website

```sh
cd site
npm ci
npm run dev       # http://localhost:4321
npm run deploy    # astro build, then wrangler deploy to ownpurse.com (Workers static assets)
```

User docs live in `site/src/content/docs/docs/`, blog posts in `site/src/content/blog/`. When a command, flag or
behaviour changes, update the matching docs page in the same change.
