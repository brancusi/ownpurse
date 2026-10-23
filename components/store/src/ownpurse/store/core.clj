(ns ownpurse.store.core
  (:require
   [clojure.java.io :as io]
   [clojure.string :as str]
   [clojure.walk :as walk]
   [ownpurse.money.interface :as money]
   [xtdb.api :as xt]
   [xtdb.node :as xtn])
  (:import
   (java.time OffsetDateTime ZonedDateTime)
   (xtdb.api Xtdb Xtdb$Config)))

(defn- sql-server
  "XTDB's Postgres wire server, read-only: -1 turns the read-write port off."
  [{:keys [host read-only-port]}]
  {:host host :port -1 :read-only-port read-only-port})

(defn node-config
  "Builds the XTDB configuration from ours. Flight SQL is always off; the SQL server only when asked for."
  ^Xtdb$Config [{:keys [mode dir sql]}]
  (let [storage (case mode
                  :in-memory {}
                  :disk (let [root (io/file dir "xtdb")]
                          {:log [:local {:path (str (io/file root "log"))}]
                           :storage [:local {:path (str (io/file root "objects"))}]}))]
    (doto (xtn/->config (assoc storage :server (some-> sql sql-server)))
      (.setFlightSql nil))))

(defn start
  "Starts an in-process node."
  ^Xtdb [config]
  (xtn/start-node (node-config config)))

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

(defn- table-name [table]
  (let [n (name table)]
    (when-not (re-matches #"[a-z][a-z0-9_-]*" n)
      (throw (ex-info (str "The table name " (pr-str n) " is not one ownpurse writes.") {:type :ownpurse/bad-table})))
    (str/replace n "-" "_")))

(defn- table-exists? [node table-name]
  (seq (q node ["SELECT table_name FROM information_schema.tables WHERE table_name = ?" table-name] {})))

(defn- current
  "The documents of `table` the store holds now, by id."
  [node table ids]
  (let [table (table-name table)]
    (if-not (table-exists? node table)
      {}
      (into {}
            (for [chunk (partition-all 500 ids)
                  doc (q node (into [(str "SELECT * FROM " table " WHERE _id IN ("
                                          (str/join ", " (repeat (count chunk) "?")) ")")]
                                    chunk)
                         {})]
              [(:xt/id doc) (comparable doc)])))))

(defn put-changed! [node tables metadata]
  (let [changed (into {}
                      (for [[table docs] tables
                            :let [now (current node table (map :xt/id docs))
                                  fresh (filterv #(not= (comparable %) (now (:xt/id %))) docs)]
                            :when (seq fresh)]
                        [table fresh]))]
    {:written (update-vals changed count)
     :tx (when (seq changed) (put! node changed metadata))}))
