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
