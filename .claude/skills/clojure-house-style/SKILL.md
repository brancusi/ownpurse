---
name: clojure-house-style
description: How to write Clojure in the ownpurse workspace (Polylith, donut.system, XTDB 2, Malli, money as BigDecimal) and how to work through the shared REPL. Use before writing or changing any .clj, .cljc, .edn or deps.edn file here, before writing an XTDB query or transaction, and whenever you need to evaluate, reload or test code.
---

# Clojure in ownpurse

The project's rules are in `AGENTS.md`; this skill is how to apply them in code. When this skill and AGENTS.md
disagree, AGENTS.md wins.

## Work through the REPL

There is one long-lived nREPL for the workspace, shared with the person you work with.

- The port is in `.nrepl-port` at the workspace root. Use it; ask which port only if the file is missing. If no REPL
  is running, ask the person before starting one (`clojure -M:dev:test:nrepl`, in the background).
- Evaluate with `clj-nrepl-eval -p "$(cat .nrepl-port)"` and a heredoc (`<<'EOF'`). The `clojure-eval` skill has
  the details.
- After editing a file, load it: `(require 'ownpurse.x.core :reload)`. After a rename, a deleted var or a change to
  the system, run `(dev/reset)`, which stops the system, reloads changed namespaces and starts it again.
- The REPL is shared: never `(dev/halt)` or `(dev/reset)` while the person is in the middle of something without
  saying so, and never redefine a var only in the REPL. Every definition lives in a file.
- Before saying something works, show it: evaluate it, or run its test.
- The test run from the shell is the truth: `clojure -M:dev:test:kaocha` (or `clojure -M:poly test`) before every
  commit.
- Edit files with Edit and Write, not sed: a hook repairs delimiters and runs cljfmt on those. If you must change a
  file another way, run `clj-paren-repair <file>` after.

## Polylith

- Bricks live in `components/<name>` and `bases/<name>`, with namespaces `ownpurse.<name>.*`.
- Every component has `ownpurse.<name>.interface`. Other bricks require only the interface, never
  `ownpurse.<name>.core` or anything else inside it. The interface holds thin functions that delegate to the
  implementation, with the docstrings.
- A component owns one idea (money, the store, the ledger, one source). If two bricks need the same helper, it
  belongs in a component, not copied.
- Bases are thin: they translate HTTP to interface calls and back. No ledger logic in a base.
- `clojure -M:poly check` must pass. `clojure -M:poly info` shows what changed.
- New bricks: `clojure -M:poly create component name:<name>`, then add it to the `:dev` alias in the root
  `deps.edn`.

## donut.system

- Every stateful thing (the XTDB node, the HTTP server, a scheduler) is a component in the system, with `:start`
  and `:stop`. Nothing stateful is created at namespace load, in a `def`, or in a top-level `defonce`.
- Components get what they need through `ds/ref` and `::ds/config`; functions take the node or the handler as an
  argument rather than reaching for a global.
- Config comes from Aero (`config.edn`); a component's config is data, so a test can build a system with an
  in-memory node by overriding one key.

## Data and functions

- Pure functions in `core` namespaces; I/O at the edges (the store component, the API base, a source reader).
- Data in, data out: plain maps with namespaced keys (`:ledger/amount`, `:book/id`). No records or protocols unless
  there is real polymorphism.
- Malli schemas at every boundary: what the API accepts and returns, what a source reader produces, what goes into
  XTDB. Validate on the way in; trust the data inside.
- Errors: `(throw (ex-info "What went wrong, and how to fix it" {:type :ownpurse/<kind> ...}))`. The `:type` is
  what callers and the API dispatch on; the data carries what they need to act.
- Docstrings on every public var. Private helpers with `defn-`. No `:refer :all`; alias namespaces (`[xtdb.api :as
  xt]`).
- Follow the [Clojure style guide](https://github.com/bbatsov/clojure-style-guide): threading macros over nesting,
  `when` over single-branch `if`, destructuring in argument lists, small functions.

## Money

- Amounts are `BigDecimal`: literals with `M` (`1.10M`), arithmetic with `+`, `-`, `*` on BigDecimals, and
  rounding only with an explicit `RoundingMode` where a rule says so. Never `double`, `float`, `/` on money without
  a scale and rounding mode, or a literal like `1.1`.
- An amount always travels with its commodity: `{:amount 12.50M :commodity "USD"}`.
- JSON in and out with jsonista, configured to read every number as `BigDecimal`. Never let a JSON library decide.
- In XTDB, money columns are `DECIMAL` with an explicit precision and scale. Sum in SQL with `SUM` over `DECIMAL`
  columns only.
- Tests for money use `test.check` with generators of BigDecimal amounts, and assert exact equality.

## XTDB 2

The fetched reference is in `references/` (run `references/fetch.sh` if the files are missing):
`xtdb-small.txt` (the whole manual, abridged), `xtdb-sql.txt`, `xtdb-xtql.txt`. Search them before guessing at
syntax.

- Run the node in-process, started and stopped by donut.system. Tests use an in-memory node per test.
- Write with `xt/execute-tx` (waits, and throws on failure) unless you have a reason to use `xt/submit-tx`.
- Every transaction we submit carries `METADATA` (who, why, from what source). It lands in `xt.txs.user_metadata`.
- Tables have no fixed schema. Every document has `_id`; keep ids stable and derived from the source's own id.
- Valid time is when something was true in the world; system time is when we recorded it. Query the past with
  `FOR VALID_TIME AS OF …` and `FOR SYSTEM_TIME AS OF …`, or `SETTING DEFAULT SYSTEM_TIME TO AS OF …`.
- Never `ERASE`, and never `DELETE` ledger history: corrections are new versions. The only exception is a person's
  explicit instruction, written down in the commit.
- Prefer SQL for anything a person or a report might read; use XTQL from Clojure when composing queries as data
  helps.

## Tests

- Tests sit beside the brick: `components/<name>/test/ownpurse/<name>/*_test.clj`.
- Laws as properties (`test.check`): an entry balances, a second load changes nothing, a restore gives the same
  trial balance.
- Fixtures are synthetic only (see SECURITY.md). Never put real names, ids or amounts in a test.
