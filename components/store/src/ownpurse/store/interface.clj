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

(defn snapshot!
  "Snapshots the running record into a new folder under `to` (ownpurse-<time>), while it runs: asks the node to
  write every transaction to storage files, copies them, and checks that a fresh node opens the copy and holds every
  transaction. Keeps the snapshot in the table backups, and returns its manifest {:dir :taken-at :block :epoch
  :latest-tx-id :transactions :tables :files :bytes}. `config` is the store's config, which says where the record
  is. Throws :ownpurse/no-disk for a record in memory, :ownpurse/backup-timeout, :ownpurse/backup-exists or
  :ownpurse/backup-invalid; a failed snapshot leaves nothing under `to`."
  [node config to]
  (core/snapshot! node config to))

(defn last-snapshot
  "The latest snapshot the record keeps, or nil: {:xt/id :dir :taken-at :block :epoch :transactions :tables :files
  :bytes}."
  [node]
  (core/last-snapshot node))

(defn restore!
  "Restores the snapshot in `from` as the record in the data directory `dir`, which must not hold one yet (throws
  :ownpurse/record-exists, or :ownpurse/not-a-snapshot). Run it with ownpurse stopped, then start ownpurse on `dir`:
  every version keeps its valid and system times, and xt.txs keeps every transaction and its metadata. The record
  continues on a new log, in the next epoch (written to xtdb/epoch.edn). Returns {:dir :epoch :block
  :transactions}."
  [opts]
  (core/restore! opts))
