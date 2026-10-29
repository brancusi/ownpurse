# The ownpurse admin

A web admin over the ownpurse API, in ClojureScript with [UIx](https://uix-cljs.dev) (React 19) and Tailwind. ownpurse
serves it at **/admin**, over whatever record it runs on: the repository's own development record, or a workspace's
real books. Its first section is the **sandbox**, where a rule set for coding bank lines is built and tested.

## Running it

```sh
bin/admin dev      # builds on save (ClojureScript and Tailwind); a save shows in the open page, its state kept
bin/admin build    # a release build, once: what a workspace needs before its server serves /admin
bin/admin check    # compiles it, warnings as errors
```

With the REPL's system running (`(go)`), open <http://127.0.0.1:7070/admin/>. The first run installs the npm
packages; nothing else is needed (Node and the Clojure CLI, as for the rest of ownpurse).

While `bin/admin dev` runs:

- **A save is live.** Components keep their state across a save (React Fast Refresh, through `uix.preload`); a CSS
  change is swapped in without a reload.
- **A REPL into the browser.** Connect your editor to shadow-cljs's nREPL on port 9631 (Calva: "Connect to a running
  REPL", shadow-cljs, build `:app`), or run `npx shadow-cljs cljs-repl app` in `admin/`. Evaluate in a `.cljs` file
  and it runs in the open page.
- **The hooks are linted as you build**: UIx's linter warns about a missing dependency of an effect or a list item
  without a `:key`, at compile time. `clj-kondo` knows UIx through its own config, vendored at `.clj-kondo/com.pitch/uix.core`.

## How it fits together

```
browser ── /admin/ ───────────▶ ownpurse server ── resources: admin/resources/ownpurse/admin (built)
        ── /sandbox/… (Transit) ▶ bases/api ── components/sandbox ── components/classify
```

- **One origin.** The page and the API come from the same server, so there is no proxy, no CORS and no second port in
  production. Any workspace that runs ownpurse (`(go)`, or `bin/ownpurse serve`) has the admin on its own port, over
  its own record.
- **Data, not translations.** The API answers in Transit when asked (`Accept: application/transit+json`), so the
  browser gets the components' data as they hold them: keywords, sets, maps. The admin reads `(:expect unit)` exactly
  as the server code does. JSON is still the default for every other client.
- **Shared code.** `.cljc` namespaces in the bricks are compiled into the admin too. `ownpurse.classify.outcome` is the
  eval's own scoring: the sandbox scores every line at any threshold in the browser with the same function, without
  asking the server.
- **Where it is in the URL.** `#/sandbox/atlas/<run>?t=85&s=choice&f=wrong&line=d211`: every view, its threshold and
  its open line are a link someone else can open.

## Code

| Namespace | What |
|---|---|
| `ownpurse.admin.main` | the shell (the bar of sections), the router's switch |
| `ownpurse.admin.theme` | light or dark: the system's, or one picked in the bar and kept in the browser |
| `ownpurse.admin.route` | the hash route: parse, `href`, `go!`, `use-route` |
| `ownpurse.admin.api` | `use-get`: an API answer, read from Transit, kept for the page's life |
| `ownpurse.admin.ui` | the building blocks: tag, pill, dot, chip, bar, amount, eyebrow, panel, disclosure, notice |
| `ownpurse.admin.terms` | the vocabulary as data (each word's meaning and where it comes from), and `term`: a dotted word that shows its meaning on hover and links to its entry |
| `ownpurse.admin.docs.*` | the docs: `guide` (a bank line to a bucket in ten steps, on a real line of the sandbox) and `vocabulary` (every word) |
| `ownpurse.admin.sandbox.*` | the sandbox: `home` (scenarios and runs), `run` (levers, buckets, lines), `line` (a line's seven steps) |

A new section is a namespace under `ownpurse.admin.<section>` with a `page`, an entry in `main/sections`, and a case
in `main/app`. Its data come from a route in `bases/api` over a component's interface, as data: the component never
builds markup, and the admin never recomputes what a component decides.

## Words

The admin is read by people new to bookkeeping and to models alike. A word a newcomer may not know (bucket, block,
threshold, per diem) goes through `term`: `($ term {:id :block} "blocks")`. A new word is an entry in
`ownpurse.admin.terms`: what it means in a sentence, more if needed, where it comes from (an accountant's, a bank's,
machine learning's, TypeSafe's, or ownpurse's own), and the guide's step where it happens. When the pipeline changes,
the guide (`ownpurse.admin.docs.guide`) changes with it: it reads a real line through the API, so a step that no
longer matches the code shows wrong data, not stale prose.

## Styles

All colour, type and radius come from ownpurse's design tokens (`brand/tokens.json` → `brand/tokens.css`), which
`src/admin.css` maps into Tailwind's theme. Change a token there and every screen follows, in both themes.

- **Two themes, following the system's** unless one is picked with Auto · Light · Dark in the bar (kept in that
  browser; `ownpurse.admin.theme`). Light is the brand's; dark is the admin's own softer slate (set in
  `src/admin.css`, the same roles with pastel values). Text is JetBrains Mono; page titles keep the brand's display
  face.
- **Use the token utilities**: `bg-surface-1`, `text-ink-3`, `border-line-1`, `bg-coin-tint`, `text-verdigris`,
  `font-mono`, `font-display`, `rounded-1`, and the admin's type scale `text-2xs` … `text-2xl`. Never a hex value, never
  a Tailwind palette colour (`text-gray-500`).
- **Reuse a block before writing classes.** Anything used twice is a `defui` in `ownpurse.admin.ui`, and its classes
  live there only. A screen lays blocks out (grid, gap, margins); it does not restyle them.
- **Colour marks an outcome only**, as the design system's coins do: verdigris right, coin wrong, gilt sent to a
  person needlessly, ink sent to a person rightly. Copy follows `brand/README.md`: plain, exact, sentence case.
- Class names must be whole strings in the source (Tailwind finds them by reading `src/`): `(cx "px-2" (when on?
  "bg-ink-1"))`, never `(str "bg-" colour)`.

## For a workspace

A workspace that loads ownpurse beside it (its `deps.edn` names `ownpurse/server`) already serves `/admin` on its
own port. Two steps:

1. Build the admin once in the ownpurse checkout: `bin/admin build` (or keep `bin/admin dev` running).
2. Point the sandbox at the workspace's own folders, never at this repository: `OWNPURSE_RUNS_DIR` for its runs and
   `OWNPURSE_SCENARIOS_DIR` for its own scenarios (`<name>.edn`, the same shape as
   `components/classify/resources/classify/eval/atlas.edn`). They are listed beside ownpurse's invented ones.

A workspace's own scenarios in its record (Scenarios section) code its books' lines and show in the Sandbox beside the
made-up ones. How the Sandbox, Scenarios and lines fit together next is [docs/plans/SANDBOX.md](../docs/plans/SANDBOX.md).
