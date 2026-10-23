(ns ownpurse.ledger.interface
  "The ledger over every source: books, accounts, parties, documents, entries and postings, kept in the store.
  Knows nothing about any one source."
  (:require
   [ownpurse.ledger.core :as core]))

(def tables
  "The tables the ledger keeps, in the order it writes them."
  core/tables)

(defn rows
  "Turns facts ({:sources :books :accounts :parties :documents :entries}) into the store's rows, by table. Keys
  lose their namespace (`:book/name` is the column `name`), keywords become strings, and each entry's postings
  become rows of their own (`postings`), so SQL can sum them. Pure."
  [facts]
  (core/rows facts))

(defn check!
  "Checks every fact against its schema and returns the facts. Throws :ownpurse/invalid for the first that does
  not match, saying which one."
  [facts]
  (core/check! facts))

(defn load!
  "Checks `facts`, then writes the rows that differ from what the store holds, in one transaction carrying
  `metadata`. Returns {:written {table count} :tx key-or-nil}; a second load of the same facts writes nothing."
  [node facts metadata]
  (core/load! node facts metadata))
