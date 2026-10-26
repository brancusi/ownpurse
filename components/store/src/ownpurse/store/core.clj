(ns ownpurse.store.core
  (:require
   [clojure.java.io :as io]
   [clojure.string :as str]
   [clojure.walk :as walk]
   [ownpurse.money.interface :as money]
   [ownpurse.store.backup :as backup]
   [ownpurse.store.files :as files]
   [xtdb.api :as xt]
   [xtdb.node :as xtn])
  (:import
   (java.time Duration OffsetDateTime ZonedDateTime)
   (xtdb.api Xtdb Xtdb$Config)))

(defn- sql-server
  "XTDB's Postgres wire server, read-only: -1 turns the read-write port off."
  [{:keys [host read-only-port]}]
  {:host host :port -1 :read-only-port read-only-port})

(defn node-config
  "Builds the XTDB configuration from ours. Flight SQL is always off; the SQL server only when asked for. Paths go to
  XTDB as Paths: a string with a space in it is not a valid URI, and XTDB parses strings as URIs."
  ^Xtdb$Config [{:keys [mode dir sql]}]
  (let [storage (case mode
                  :in-memory {}
                  :disk (let [root (io/file dir "xtdb")]
                          {:log [:local {:path (.toPath (io/file root "log")) :epoch (backup/epoch dir)}]
                           :storage [:local {:path (.toPath (io/file root "objects"))}]}))]
    (doto (xtn/->config (assoc storage :server (some-> sql sql-server)))
      (.setFlightSql nil))))

(defn start
  "Starts an in-process node. A node on disk first catches up with its log, so that what it answers includes
  every transaction written before it stopped."
  ^Xtdb [{:keys [mode] :as config}]
  (let [node (xtn/start-node (node-config config))]
    (when (= :disk mode)
      (backup/sync! node (Duration/ofMinutes 5)))
    node))

(defn stop
  "Closes the node."
  [^Xtdb node]
  (.close node))

(defn health
  "Asks the node a trivial query."
  [node]
  (try
    (xt/q node "SELECT 1 AS ok")
    {:ok? true}
    (catch Exception e
      {:ok? false :error (ex-message e)})))

(defn sql-port
  "The bound read-only SQL port, or nil."
  [^Xtdb node]
  (let [port (.getServerReadOnlyPort node)]
    (when (pos? port) port)))

(defn- check-metadata! [{:keys [source why] :as metadata}]
  (when-not (and (string? source) (seq source) (string? why) (seq why))
    (throw (ex-info "Every write says where it comes from and why: pass {:source \"...\" :why \"...\"}."
                    {:type :ownpurse/no-metadata :metadata metadata}))))

(defn- check-floats! [tables metadata]
  (let [paths (money/float-paths {:tables tables :metadata metadata})]
    (when (seq paths)
      (throw (ex-info (str "Nothing was written: " (count paths) " value(s) are floats. Amounts are exact decimals "
                           "(BigDecimal); convert them where they enter, never with a double. First at "
                           (pr-str (first paths)) ".")
                      {:type :ownpurse/float-money :paths paths})))))

(defn put! [node tables metadata]
  (check-metadata! metadata)
  (check-floats! tables metadata)
  (xt/execute-tx node
                 (for [[table docs] tables :when (seq docs)]
                   (into [:put-docs table] docs))
                 {:metadata metadata}))

(defn q [node query opts]
  (xt/q node query opts))

(defn- comparable
  "A document as values: XTDB hands timestamps back zoned, so every instant compares as an Instant."
  [doc]
  (walk/postwalk (fn [x]
                   (cond
                     (instance? ZonedDateTime x) (.toInstant ^ZonedDateTime x)
                     (instance? OffsetDateTime x) (.toInstant ^OffsetDateTime x)
                     :else x))
                 doc))

(defn- sql-part [s]
  (when-not (re-matches #"[a-z][a-z0-9_-]*" s)
    (throw (ex-info (str "The table name " (pr-str s) " is not one ownpurse writes.") {:type :ownpurse/bad-table})))
  (str/replace s "-" "_"))

(defn- table-name
  "The table as SQL names it: [schema table]. `:invoices` is public.invoices, `:xero/invoices` xero.invoices."
  [table]
  [(sql-part (or (namespace table) "public")) (sql-part (name table))])

(defn- sql-table-exists? [node [schema table]]
  (seq (q node ["SELECT table_name FROM information_schema.tables WHERE table_schema = ? AND table_name = ?"
                schema table]
          {})))

(defn table-exists? [node table]
  (boolean (sql-table-exists? node (table-name table))))

(defn delete! [node table ids {:keys [valid-from]} metadata]
  (check-metadata! metadata)
  (when (namespace table)
    (throw (ex-info (str "Nothing was deleted: " table " is a source table, and source rows are never deleted. "
                         "A source's correction is a new version of its row.")
                    {:type :ownpurse/source-row-delete :table table})))
  (xt/execute-tx node
                 [(into [:delete-docs (cond-> {:from table} valid-from (assoc :valid-from valid-from))] ids)]
                 {:metadata metadata}))

(defn- current
  "The documents of `table` the store holds now, by id."
  [node table ids]
  (let [[schema t :as sql-name] (table-name table)]
    (if-not (sql-table-exists? node sql-name)
      {}
      (into {}
            (for [chunk (partition-all 500 ids)
                  doc (q node (into [(str "SELECT * FROM " schema "." t " WHERE _id IN ("
                                          (str/join ", " (repeat (count chunk) "?")) ")")]
                                    chunk)
                         {})]
              [(:xt/id doc) (comparable doc)])))))

(defn- content
  "A document as `current` sees it: without the valid time it was written with, which is not a column."
  [doc]
  (comparable (dissoc doc :xt/valid-from :xt/valid-to)))

(defn put-changed! [node tables metadata]
  (let [changed (into {}
                      (for [[table docs] tables
                            :let [now (current node table (map :xt/id docs))
                                  fresh (filterv #(not= (content %) (now (:xt/id %))) docs)]
                            :when (seq fresh)]
                        [table fresh]))]
    {:written (update-vals changed count)
     :tx (when (seq changed) (put! node changed metadata))}))

(defn snapshot! [node {:keys [mode dir]} to]
  (when-not (= :disk mode)
    (throw (ex-info "The record is in memory: there are no files to snapshot. Run ownpurse with a data directory."
                    {:type :ownpurse/no-disk})))
  (let [manifest (backup/snapshot! node {:dir dir :to to})]
    (put! node {:backups [(-> (select-keys manifest [:dir :block :epoch :transactions :tables :files :bytes :kept-files :kept-bytes])
                              (assoc :xt/id (str "snapshot:" (:taken-at manifest))
                                     :taken-at (ZonedDateTime/parse (:taken-at manifest))))]}
          {:source "ownpurse" :why "backup"})
    manifest))

(defn keep-file! [{:keys [mode dir]} write!]
  (when-not (= :disk mode)
    (throw (ex-info "The record is in memory: it cannot keep files. Run ownpurse with a data directory."
                    {:type :ownpurse/no-disk})))
  (files/keep-file! dir write!))

(defn last-snapshot [node]
  (when (table-exists? node :backups)
    (first (q node "SELECT * FROM backups ORDER BY taken_at DESC LIMIT 1" {}))))

(defn restore! [opts]
  (backup/restore! opts))
