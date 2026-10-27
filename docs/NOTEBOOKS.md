# Notebooks

ownpurse's notebooks are [Clerk](https://clerk.vision) notebooks: Clojure files that Clerk evaluates and shows in a
browser as prose, code and results. They live in `notebooks/`, a small project that is neither a brick nor part of
the server. A REPL puts them on its classpath beside ownpurse and says which record they read.

| Notebook | Shows |
|---|---|
| `ownpurse.notebooks.tour` | ownpurse from nothing over the synthetic samples: a record, a pull through a stand-in xero-cli, the book, the trial balance check, a proposed close journal and the pro-forma. It opens its own record in memory. |
| `ownpurse.notebooks.book` | any report of one Xero org's book for a financial year, from the record the REPL said to read: the tables the accountant's export writes. |

`ownpurse.notebook` holds what they share: `use-record!` (which record), `serve!` and `halt!` (Clerk on
127.0.0.1), `show!` (a notebook with its params), `reshow!` (again, after a pull or a change) and `table` (a report
as a Clerk table, amounts exact).

## In this repository: synthetic data

```clojure
(dev)
(go)
(notebooks)                                                    ; Clerk on http://127.0.0.1:7777, the tour open
(show :profit-and-loss {:org "Tallowmere" :date "2026-03-31"}) ; over the dev system's record
```

`(notebooks)` points the notebooks at the dev system's record, watches `notebooks/src` (a saved notebook is shown)
and `components` and `bases` (a saved source file is loaded, and the last notebook shown again). The tour needs
nothing else; the book notebook needs a record holding a Xero org (`(keep-xero-answers ...)` or a pull of the
samples). In Calva, `ctrl+alt+space` then `n`.

## In a private workspace: its own records

A workspace that runs ownpurse over a group's real books already has a REPL whose classpath is this checkout's
source (`{:local/root "../ownpurse/projects/server"}`). Add the notebooks the same way, and that REPL shows the same
notebooks over its own record:

```clojure
;; the workspace's deps.edn, in its :dev alias
:extra-deps {ownpurse/notebooks {:local/root "../ownpurse/notebooks"}}
```

```clojure
;; in that REPL, once it holds the record
(require '[ownpurse.notebook :as notebook])
(notebook/use-record! (fn [] <the workspace's node>))
(notebook/serve! {:port 7778
                  :watch-paths ["notebooks"]                   ; the workspace's own notebooks
                  :code-paths ["../ownpurse/components" "../ownpurse/bases"]})
(notebook/show! :profit-and-loss {:org "<the start of an org's name>" :date "<year end>"})
(notebook/show! "notebooks/close.clj")                        ; a notebook of the workspace's own
```

The loop: open both folders in one editor window, connected to the workspace's REPL. Change ownpurse's code here;
saving a file loads it into that REPL and shows the notebook again over the real record. ownpurse's tests run here,
over the samples, as before; nothing changes about what is committed.

What keeps the record out of this repository:

- The record, its config and every notebook about a group's own books stay in the workspace. The notebooks here
  name no group, and read a record only through `use-record!`.
- Clerk's cache is off (`ownpurse.notebook` turns it off when it loads): Clerk would otherwise write each result
  under `.clerk/` beside the code, and show stale figures after a pull. `.clerk/` and `public/build/` are ignored
  here all the same.
- Clerk listens on 127.0.0.1 only. A static build (`clerk/build!`) of a notebook over a real record is a file of
  that record: build it into the workspace's own private folders, never here.
- A REPL that shows a real record shows it to whoever drives it. Before every commit here, read the diff as
  AGENTS.md says: no names, ids, amounts or paths from a workspace's books.
