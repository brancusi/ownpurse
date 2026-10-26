(ns ownpurse.plaid.pull
  "Fetching through plaid-reader and keeping what Plaid said, as source rows in the plaid schema, with each item's
  cursor beside them."
  (:require
   [clojure.edn :as edn]
   [clojure.java.io :as io]
   [ownpurse.plaid.cli :as cli]
   [ownpurse.plaid.source :as source]
   [ownpurse.schema.interface :as schema]
   [ownpurse.store.interface :as store])
  (:import
   (java.time Instant)))

(defn- held
  "Every row of `table` the store holds for Plaid item `item`, by id."
  [node table item]
  (if (store/table-exists? node table)
    (into {} (map (juxt :xt/id identity))
          (store/q node [(str "SELECT * FROM " (source/sql-table table) " WHERE item = ?") item]))
    {}))

(defn cursor
  "The cursor item `item`'s last sync ended at, or nil before its first."
  [node item]
  (when (store/table-exists? node :plaid/cursors)
    (:cursor (first (store/q node ["SELECT cursor FROM plaid.cursors WHERE _id = ?" (source/cursor-id item)])))))

(defn- end-stale-lines!
  "Ends, by DELETE, the item's bank lines that are no longer among `ids`: a removed transaction (a pending one that
  posted under a new id, or one the bank took back) stops counting, and its history stays."
  [node item ids metadata]
  (when (store/table-exists? node :bank-lines)
    (let [stale (into [] (comp (map :xt/id) (remove (set ids)))
                      (store/q node ["SELECT _id FROM bank_lines WHERE source_table = ? AND org = ?"
                                     (source/sql-table (:transactions source/tables)) item]))]
      (when (seq stale)
        (store/delete! node :bank-lines stale {} (assoc metadata :why "fetch: lines that no longer hold"))))))

(defn keep!
  "Keeps one item's answers: `item-answer` (/item/get) and `sync-answer` (a sync of every page from the cursor the
  store held). In one transaction, so the cursor never moves without the transactions it covers: the item, its
  accounts and their source accounts, the added and modified transactions, the removed ones as new versions marked
  removed, the bank lines of every transaction held, and the cursor to start from next time. A line whose
  transaction was removed ends. opts: :tool-version, :alias, :now (the time of the fetch). Returns {resource
  {:items :written}}."
  [node item-answer sync-answer {:keys [tool-version alias now]}]
  (let [fetched-at (or now (Instant/now))
        item-row (source/item-row item-answer)
        item (:item item-row)
        {:keys [items accounts transactions cursors]} source/tables
        held-txns (held node transactions item)
        fetched (source/transaction-rows item sync-answer fetched-at)
        removed (source/removed-rows item sync-answer (merge held-txns (into {} (map (juxt :xt/id identity)) fetched))
                                     fetched-at)
        account-rows (source/account-rows item sync-answer)
        every-txn (vals (merge held-txns (into {} (map (juxt :xt/id identity)) (concat fetched removed))))
        lines (mapv #(schema/check! :ownpurse/bank-line %) (source/bank-lines every-txn))
        tables {items [item-row]
                accounts account-rows
                transactions (into fetched removed)
                cursors [(source/cursor-row item sync-answer)]
                :source-accounts (mapv #(schema/check! :ownpurse/source-account %)
                                       (source/source-accounts (vals (merge (held node accounts item)
                                                                            (into {} (map (juxt :xt/id identity))
                                                                                  account-rows)))))
                :bank-lines lines}
        metadata (cond-> {:source "plaid-reader" :why "fetch" :item item :fetched-at (str fetched-at)}
                   alias (assoc :alias alias)
                   tool-version (assoc :tool-version tool-version))
        {:keys [written]} (store/put-changed! node tables metadata)]
    (end-stale-lines! node item (map :xt/id lines) metadata)
    (array-map "item" {:items 1 :written (get written items 0)}
               "accounts" {:items (count account-rows) :written (get written accounts 0)}
               "transactions" {:items (+ (count fetched) (count removed))
                               :written (get written transactions 0)})))

(defn pull!
  "Fetches and keeps one Plaid item, named by its plaid-reader alias: the item's state, then a sync of every page
  from the cursor the store holds (none: from the start). Returns {resource {:items :written}}. Stops at the first
  error plaid-reader reports (see ownpurse.plaid.cli/exit-types), keeping nothing of that item, so the next pull
  starts from the same cursor."
  [node {:keys [cli alias now]}]
  (let [tool-version (cli/version cli)
        item-answer (cli/item cli alias)
        item (get-in item-answer [:item :item_id])
        _ (when-not item
            (throw (ex-info (str "Plaid's answer for " alias " names no item id.") {:type :plaid/failed :alias alias})))
        sync-answer (cli/sync-all cli alias (cursor node item))]
    (keep! node item-answer sync-answer {:tool-version tool-version :alias alias :now now})))

(defn read-items
  "The Plaid items a local config file names, by their plaid-reader alias: {:plaid {:items [{:alias \"...\" :name
  \"...\"}]}}. Returns [] when it names none. Says how to make the file when it is missing."
  [path]
  (let [f (io/file path)]
    (when-not (.isFile f)
      (throw (ex-info (str "No config at " path ". Make it with the items to read: "
                           "{:plaid {:items [{:alias \"<plaid-reader alias>\" :name \"<a name>\"}]}} "
                           "(plaid-reader items lists them).")
                      {:type :ownpurse/no-local-config :path (str path)})))
    (let [{:keys [items]} (:plaid (edn/read-string (slurp f)))]
      (when-not (every? (comp string? :alias) items)
        (throw (ex-info (str path " names a Plaid item without an alias. Write "
                             "{:plaid {:items [{:alias \"<plaid-reader alias>\" :name \"<a name>\"}]}}.")
                        {:type :ownpurse/no-local-config :path (str path)})))
      (vec items))))

(defn pull-items!
  "Pulls each of `items` ({:alias :name}), one after another. Returns {name-or-alias {resource {:items :written}}}."
  [node {:keys [cli items now]}]
  (apply array-map (mapcat (fn [{:keys [alias name]}]
                             [(or name alias) (pull! node {:cli cli :alias alias :now now})])
                           items)))
