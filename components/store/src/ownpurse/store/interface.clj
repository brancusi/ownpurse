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
  with an :xt/id. A namespaced table is in that schema: `:xero/invoices` is the SQL table xero.invoices. A
  document may carry :xt/valid-from (when it became true in the world); without it, it is valid from now.
  `metadata` says where the documents come from and why: {:source string, :why string}, and lands
  in xt.txs. Refuses, writing nothing, when any document holds a float (:ownpurse/float-money, with the paths) or
  the metadata is missing (:ownpurse/no-metadata). A correction is a new version."
  [node tables metadata]
  (core/put! node tables metadata))

(defn put-changed!
  "Like `put!`, but writes only the documents that differ from what the store holds now (by :xt/id; a document's
  :xt/valid-from is not compared, only its content). Returns
  {:written {table count}, :tx key-or-nil}: when nothing differs, nothing is written and there is no transaction."
  [node tables metadata]
  (core/put-changed! node tables metadata))

(defn q
  "Runs a query (SQL or XTQL) and returns the rows."
  ([node query] (core/q node query {}))
  ([node query opts] (core/q node query opts)))

(defn delete!
  "Ends the documents `ids` of `table` from `:valid-from` (default now), in one transaction carrying `metadata`.
  Their history stays: a query `FOR ALL VALID_TIME` still sees them. Only for ownpurse's own tables (derived rows
  and decisions such as links): refuses a source table (a namespaced one, :xero/invoices) with
  :ownpurse/source-row-delete, since source rows are never deleted. Nothing is ever erased."
  [node table ids opts metadata]
  (core/delete! node table ids opts metadata))

(defn table-exists?
  "Whether the store holds `table` (a keyword, as `put!` names it). XTDB refuses a query on a table it has never
  seen, so a read of a table that may not exist yet asks this first."
  [node table]
  (core/table-exists? node table))
