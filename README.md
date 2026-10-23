<p align="center"><img src="brand/logos/ownpurse-wordmark.svg" alt="ownpurse" width="320"></p>

# ownpurse

**Local books for you and your agents.** ownpurse is a ledger that runs on your own machine. It reads what each of
your sources holds (your accounting system, your bank, your spreadsheets) and keeps one model of your books over
all of them, with every version kept. You and your agents use it through a local API, a command line, or SQL.

Website: [ownpurse.com](https://ownpurse.com) · [Philosophy](docs/PHILOSOPHY.md) · [Brand](brand/README.md)

> Not affiliated with or endorsed by Xero Limited. "Xero" is a trademark of Xero Limited.

## Status

ownpurse is being rebuilt from zero in Clojure, on [XTDB](https://xtdb.com). This is a spike, not a release: the
plan is in [docs/plans/SPIKE.md](docs/plans/SPIKE.md). The earlier Rust version is archived at the tag
`archive/rust-main`.

## Development

```sh
mise install                    # JDK 21, the Clojure CLI, Babashka
clojure -M:poly info            # the Polylith workspace
clojure -M:dev:test:nrepl       # the REPL; then (dev) and (go)
```

How we work, and the rules that do not bend, are in [AGENTS.md](AGENTS.md).

## License

[MIT](LICENSE)
