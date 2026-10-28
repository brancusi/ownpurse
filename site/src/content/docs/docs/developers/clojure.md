---
title: The Clojure workspace
description: The rewrite's stack, its Polylith layout, the REPL-driven way of working, the admin in ClojureScript, and the checks before a commit.
sidebar:
  order: 0
  badge:
    text: New
---

:::note[The rewrite]
This page describes the `next` branch: ownpurse rebuilt in Clojure on XTDB since 23 October 2026. The other
developer pages describe the Rust release. Why we started again:
[Starting again, in Clojure](/blog/starting-again-in-clojure/).
:::

ownpurse is a local ledger with a clean API. It keeps every answer a source gave as rows in XTDB, derives entries and
postings from them, and checks the result against the source to the cent. Clients (the admin, notebooks, agents)
use the API.

## The stack

| Layer | Choice |
|---|---|
| Runtime | JDK 21 (Temurin), Clojure 1.12, pinned in `mise.toml` (`mise install`) |
| Workspace | Polylith: components, one base (`api`), projects |
| State | donut.system: config, the XTDB node, the HTTP server |
| Store | XTDB 2 in-process, on local disk, with a read-only Postgres port for SQL clients (DataGrip, psql) |
| HTTP | Reitit, Ring on Jetty, Malli for coercion, OpenAPI from the Malli schemas; JSON, and Transit for the admin |
| Money | `BigDecimal` everywhere, JSON numbers read as `BigDecimal`; clj-kondo refuses `double` and `float` |
| Sources | Small command-line tools run as subprocesses (xero-cli, mercury-cli, plaid-reader); ownpurse keeps what they print |
| Tests | Kaocha, `test.check` for the laws, invented records as fixtures |
| Admin | ClojureScript, UIx (React 19), shadow-cljs, Tailwind themed by the design tokens |

The components: `money`, `json`, `schema`, `config`, `store`, `entity`, `ledger`, `xero`, `mercury`, `plaid`,
`statement`, `compare`, `close`, `intercompany`, `payees`, `export`, `classify`, `sandbox`, and `test-support` for
the tests. Each has an `interface` namespace; nothing outside a component reaches past it.

## Working at the REPL

Development is REPL-driven. One long-lived REPL holds the running system, and code changes go into it while it runs:

```clojure
(require 'dev)
(dev/go)          ; the record and the API: {:http 7070 :sql 5433}
(dev/watch)       ; on every save: reload the file, run the tests that cover it
(dev/last-run)    ; what the last run found
(dev/halt)        ; stop the system (before stopping a REPL that holds a record)
```

- Evaluate a function in your editor and the running server uses it on its next request. No restart, no rebuild.
- `(dev/watch)` turns a save into a test run of exactly the namespaces it touches.
- When something looks stale (a handler built from an older version of a function), `(dev/halt)` then `(dev/go)`
  rebuilds the system from the code as it is. The shell run below is the check that trusts nothing in memory.

Agents work the same way: they evaluate in the same REPL (`clj-nrepl-eval`), start and stop the system as they need
it, and run the shell checks before a commit.

## The admin

`admin/` is the web admin, served by ownpurse itself at `/admin` over whatever record it runs on. Its first sections
are the classification sandbox and its docs ([Classifying bank lines](/docs/guides/classifying/)).

```sh
bin/admin dev      # builds on save; a save shows in the open page with its state kept (React Fast Refresh)
bin/admin build    # a release build, once, before a server serves /admin
bin/admin demo     # builds it and runs ownpurse on an empty record
```

- **Data, not translations.** The API answers the admin in Transit, so the browser reads the components' data as
  they hold them: keywords, sets, maps. A `.cljc` namespace compiles for both sides: the sandbox scores 391 lines at
  any threshold in the browser with the eval's own function.
- **A REPL into the browser** on shadow-cljs's nREPL (port 9631).
- **Styles from the tokens.** Tailwind's theme reads `brand/tokens.css`; a screen composes the blocks in
  `ownpurse.admin.ui` and never writes a colour.

## Before a commit

```sh
clojure -M:dev:test:kaocha    # every test
clojure -M:poly check         # the workspace's structure
bin/admin check               # the admin, warnings as errors
```

The pre-commit hook checks every change for private data: a private denylist, then an agent's review of the diff
(`bin/privacy-check`). It is never skipped by an agent.
