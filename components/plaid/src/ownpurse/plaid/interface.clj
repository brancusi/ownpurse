(ns ownpurse.plaid.interface
  "Plaid. The only brick that knows about Plaid. It runs plaid-reader, which talks to Plaid, and keeps what Plaid
  said as source rows in the plaid schema (plaid.items, plaid.accounts, plaid.transactions), with each item's sync
  cursor in plaid.cursors. It never calls Plaid itself."
  (:require
   [ownpurse.plaid.cli :as cli]
   [ownpurse.plaid.pull :as pull]
   [ownpurse.plaid.source :as source]))

(def tables
  "The tables ownpurse keeps Plaid's answers in."
  source/tables)

(def exit-types
  "plaid-reader's exit codes, as the :type of the error a fetch throws: 1 :plaid/usage, 2 :plaid/login-needed,
  3 :plaid/rate-limited, 4 :plaid/refused, 5 :plaid/failed."
  cli/exit-types)

(defn keep!
  "Keeps one item's answers, /item/get's and a sync's of every page, in one transaction with the cursor to start
  from next time. opts: :tool-version, :alias, :now. Returns {resource {:items :written}}."
  [node item-answer sync-answer opts]
  (pull/keep! node item-answer sync-answer opts))

(defn cursor
  "The cursor the Plaid item `item` (Plaid's item id) last synced to, or nil."
  [node item]
  (pull/cursor node item))

(defn pull!
  "Fetches and keeps one Plaid item: {:cli :alias :now}. Syncs from the cursor the record holds. Returns {resource
  {:items :written}}."
  [node opts]
  (pull/pull! node opts))

(defn read-items
  "The Plaid items named in a local config file, {:plaid {:items [{:alias :name}]}}, or [] when it names none.
  Throws :ownpurse/no-local-config, saying how to make the file, when it is missing."
  [path]
  (pull/read-items path))

(defn pull-items!
  "Pulls each item: {:cli :items :now}. Returns {name {resource {:items :written}}}."
  [node opts]
  (pull/pull-items! node opts))
