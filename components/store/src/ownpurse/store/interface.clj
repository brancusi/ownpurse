(ns ownpurse.store.interface
  "The record: an XTDB node, in-process."
  (:require
   [ownpurse.store.core :as core]))

(defn start
  "Starts a node. `config` is {:mode :in-memory|:disk, :dir path, :sql nil|{:host :read-only-port}}. The SQL port
  is read-only; nothing writes to the record except through ownpurse."
  [config]
  (core/start config))

(defn stop
  "Stops a node started with `start`."
  [node]
  (core/stop node))

(defn health
  "Returns {:ok? true} when the node answers a query, or {:ok? false :error message}."
  [node]
  (core/health node))

(defn sql-port
  "Returns the port the read-only SQL server listens on, or nil when it is off."
  [node]
  (core/sql-port node))

(defn put!
  "Writes documents in one transaction and returns its key. `tables` maps a table (a keyword) to documents, each
  with an :xt/id. `metadata` says where the documents come from and why: {:source string, :why string}, and lands
  in xt.txs. Refuses, writing nothing, when any document holds a float (:ownpurse/float-money, with the paths) or
  the metadata is missing (:ownpurse/no-metadata). There is no delete: a correction is a new version."
  [node tables metadata]
  (core/put! node tables metadata))

(defn put-changed!
  "Like `put!`, but writes only the documents that differ from what the store holds now (by :xt/id). Returns
  {:written {table count}, :tx key-or-nil}: when nothing differs, nothing is written and there is no transaction."
  [node tables metadata]
  (core/put-changed! node tables metadata))

(defn q
  "Runs a query (SQL or XTQL) and returns the rows."
  ([node query] (core/q node query {}))
  ([node query opts] (core/q node query opts)))
