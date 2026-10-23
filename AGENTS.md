# Working on ownpurse

Guidance for coding agents (and people) working in this repository. Read [docs/PHILOSOPHY.md](docs/PHILOSOPHY.md)
before changing behaviour, and [brand/README.md](brand/README.md) before changing anything visual or any copy.

## Current work

The spike, on branch `next`: a local ledger in Clojure on XTDB, from zero. The plan and its steps are in
[docs/plans/SPIKE.md](docs/plans/SPIKE.md). The Rust code is archived at the tags `archive/rust-main` and
`archive/rust-facts`, and its docs are in `docs/archive/`; read them for ideas, never build on them.

## What this is

ownpurse is a local ledger with a clean API. Each source has its own tool (xero-axi for Xero, mercury-axi for
Mercury); ownpurse reads what those tools hold and keeps one model of books, accounts, entries and postings over
all of them, in XTDB. Clients (a CLI, a TUI, a web interface, agents) are separate and use the API. Never add
knowledge of a source to code outside that source's component.

## Layout

A Polylith workspace (top namespace `ownpurse`):

- `components/`: the bricks with the logic, each with an `interface` namespace. Other bricks call the interface,
  never the implementation.
- `bases/`: the entry points (the HTTP API).
- `projects/`: what gets built (the server uberjar).
- `development/`: the REPL project. `user.clj` stays tiny; `dev.clj` holds the REPL system.
- `samples/`: synthetic source files, used as test input. `docs/`, `brand/`, `site/` as before.

## The REPL

Work through one long-lived REPL, shared by people and agents.

1. Start it once from the workspace root: `clojure -M:dev:test:nrepl`. It writes `.nrepl-port`.
2. In the REPL: `(dev)`, then `(go)` starts the system, `(halt)` stops it, `(reset)` stops it, reloads changed
   namespaces and starts it again.
3. After a rename or a deleted var, `(reset)`. If anything seems off, restart the JVM.
4. Never leave a definition only in the REPL: it must be in a file.
5. The test run from the shell is the truth, not the REPL's state. Run it before every commit.

A person can attach Calva, CIDER or Cursive to the same port (cider-nrepl is in the middleware).

## Clojure practice

- Pure functions in components; side effects (the XTDB node, HTTP, files) at the edges, owned by donut.system.
  No global state outside the system.
- Data in, data out. Namespaced keywords. Malli schemas at every boundary (API, sources, the store).
- Errors are `ex-info` with a `:type` and the data needed to act on them.
- Follow the [Clojure style guide](https://github.com/bbatsov/clojure-style-guide). Format with cljfmt, lint
  with clj-kondo.
- Small namespaces, docstrings on public vars, no `:refer :all`.

## Names and words

- The product is always **ownpurse**: lowercase, one word, even at the start of a sentence. Environment variables
  are `OWNPURSE_*`.
- Say books, accounts, entries, sources, facts. Not tenants, entities or rows in user-facing text.
- Copy is plain and exact: second person, sentence case, numbers to the cent, no emoji, no hype ("seamless",
  "magic", "AI-powered"). Errors say what went wrong and how to fix it.
- "Xero" is Xero Limited's trademark: keep the "not affiliated" line wherever the name is used prominently.

## Private data

The repository is public. Work only with synthetic fixtures, never real books. A real record is read from a path
in a local, uncommitted config, and its data never enters the repository. Before every commit, read your diff for
credentials, real records or configs, real ids, personal names, emails, paths and real amounts. The full list is
in [SECURITY.md](SECURITY.md#what-never-goes-in-this-repository).

## Rules that do not bend

- Money is `BigDecimal` (`1.10M`), stored as XTDB `DECIMAL`. Never a float or a double: not in code, not in JSON
  (read numbers as `BigDecimal`), not in a test.
- History is never rewritten. Corrections are new facts; nothing in the ledger is updated in place or erased.
- ownpurse never calls a provider or holds a provider's token. Writes to a source go through that source's tool,
  as a sealed package a person approves.
- Tests never call a provider. They use the synthetic fixtures.
- The API is the contract: every endpoint has Malli schemas and appears in the OpenAPI document; errors are
  structured.

## Tooling for agents

- `.claude/skills/clojure-house-style`: how to write Clojure here and work through the REPL. Run
  `.claude/skills/clojure-house-style/references/fetch.sh` once per clone to fetch XTDB's docs for language models.
- `.claude/skills/clojure-eval` and the hooks in `.claude/settings.json` come from
  [clojure-mcp-light](https://github.com/bhauman/clojure-mcp-light) v0.2.2. Install its binaries once per machine
  with bbin, pinned to the tag's commit (bbin 0.2.5 ignores `--tag`):

  ```sh
  U=https://github.com/bhauman/clojure-mcp-light.git S=cf48cc6ab1d79809a97a74355492d44ee3bbc4ba
  bbin install $U --git/sha $S
  bbin install $U --git/sha $S --as clj-nrepl-eval --main-opts '["-m" "clojure-mcp-light.nrepl-eval"]'
  bbin install $U --git/sha $S --as clj-paren-repair --main-opts '["-m" "clojure-mcp-light.paren-repair"]'
  ```

- clojure-lsp, clj-kondo and cljfmt come from `mise install`. In Claude Code, `/plugin marketplace add
  juxt/claude-plugins` then `/plugin install clojure-lsp` connects clojure-lsp to the agent.

## Checks

```sh
mise install                    # JDK 21, the Clojure CLI, Babashka (pinned in mise.toml)
clojure -M:poly info            # the workspace
clojure -M:poly check           # the workspace's rules
```

Tests, lint and the release gate arrive with step 1 of the spike.
