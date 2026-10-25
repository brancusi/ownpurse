(ns ownpurse.mercury.interface
  "Mercury. The only brick that knows about Mercury. It runs mercury-cli, which talks to Mercury, and keeps what
  Mercury said as source rows in the mercury schema (mercury.accounts, mercury.credit_accounts,
  mercury.transactions, ...). It never calls Mercury itself."
  (:require
   [ownpurse.mercury.cli :as cli]
   [ownpurse.mercury.pull :as pull]
   [ownpurse.mercury.source :as source]))

(def resources
  "The Mercury resources ownpurse keeps, by mercury-cli's name, each with its table (`:mercury/transactions` is
  mercury.transactions)."
  source/resources)

(def exit-types
  "mercury-cli's exit codes, as the :type of the error a fetch throws: 1 :mercury/usage, 2 :mercury/login-needed,
  3 :mercury/rate-limited, 4 :mercury/refused, 5 :mercury/failed."
  cli/exit-types)

(def default-window-days
  "How far back, in days, a later pull asks for transactions by creation date (pending ones are always asked for)."
  pull/default-window-days)

(defn rows
  "Mercury's answer to `resource` for Mercury org `org` as rows for its table, {table [rows]}: each item as Mercury
  sent it, in kebab case, amounts as decimals, times as instants, without signed links, its row id derived from
  Mercury's id, the org as `org` (and the account as `account`, for statements), a transaction valid from the time
  its status says (never later than `fetched-at`). Pure."
  ([org resource answer fetched-at] (source/rows org resource answer fetched-at))
  ([org resource answer fetched-at account] (source/rows org resource answer fetched-at account)))

(defn fetch
  "Runs `mercury-cli get <resource> --org <alias> --all --format json` and returns Mercury's answer, every number
  exact. opts: :account, :start, :status. Throws one of `exit-types`."
  [cli alias resource opts]
  (cli/get-resource cli alias resource opts))

(defn keep!
  "Keeps Mercury's answer to `resource` for Mercury org `org`, writing only the rows that changed, in one
  transaction whose metadata names the tool, the org and the resource. opts: :tool-version, :alias, :account
  (statements), :start and :now (the time of the fetch). Returns {:items n :written n}."
  [node org resource answer opts]
  (pull/keep! node org resource answer opts))

(defn pull!
  "Fetches and keeps one Mercury org: {:cli :alias :window-days :now}. The first pull asks for every transaction;
  later ones for those created in the last window and every pending one. Returns {resource {:items :written}}."
  [node opts]
  (pull/pull! node opts))

(defn read-orgs
  "The Mercury orgs named in a local config file, {:mercury {:orgs [{:alias :name}] :window-days n}}, or [] when
  it names none. Throws :ownpurse/no-local-config, saying how to make the file, when it is missing."
  [path]
  (pull/read-orgs path))

(defn pull-orgs!
  "Pulls each org: {:cli :orgs :now}. Returns {name {resource {:items :written}}}."
  [node opts]
  (pull/pull-orgs! node opts))
