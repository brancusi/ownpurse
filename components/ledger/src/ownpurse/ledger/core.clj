(ns ownpurse.ledger.core
  (:require
   [ownpurse.schema.interface :as schema]
   [ownpurse.store.interface :as store]))

(def tables [:sources :books :accounts :parties :documents :entries :postings])

(def ^:private schemas
  {:sources :ownpurse/source
   :books :ownpurse/book
   :accounts :ownpurse/account
   :parties :ownpurse/party
   :documents :ownpurse/document
   :entries :ownpurse/entry})

(defn- id-key [m]
  (some #(when (= "id" (name %)) %) (keys m)))

(defn- row
  "A fact as a row: its id as :xt/id, plain keys, keyword values as strings, no nils."
  [fact]
  (let [id (id-key fact)]
    (into {:xt/id (get fact id)}
          (keep (fn [[k v]]
                  (when (and (not= k id) (some? v))
                    [(keyword (name k)) (if (keyword? v) (name v) v)])))
          fact)))

(defn- posting-rows [{:entry/keys [id book date postings]}]
  (for [{:posting/keys [line] :as p} postings]
    (row (assoc p
                :posting/id (str id ":" line)
                :posting/entry id
                :posting/book book
                :posting/date date))))

(defn rows [{:keys [entries] :as facts}]
  (-> (into {} (for [[table facts] (dissoc facts :entries)] [table (mapv row facts)]))
      (assoc :entries (mapv (comp row #(dissoc % :entry/postings)) entries)
             :postings (vec (mapcat posting-rows entries)))))

(defn check! [facts]
  (doseq [[table schema-key] schemas
          fact (get facts table)]
    (schema/check! schema-key fact))
  facts)

(defn load! [node facts metadata]
  (let [rows (rows (check! facts))]
    (store/put-changed! node (into (array-map) (for [t tables :when (seq (rows t))] [t (rows t)])) metadata)))

(defn- held-ids
  "The ids of `table`'s rows for `book` the store holds now."
  [node table book]
  (if (store/table-exists? node table)
    (set (map :xt/id (store/q node [(str "SELECT _id FROM " (name table) " WHERE book = ?") book])))
    #{}))

(defn put-entries! [node book entries metadata]
  (doseq [e entries]
    (when (not= book (:entry/book e))
      (throw (ex-info (str "The entry " (:entry/id e) " belongs to " (:entry/book e) ", not " book ".")
                      {:type :ownpurse/invalid :entry (:entry/id e) :book book})))
    (schema/check! :ownpurse/entry e))
  (let [{entry-rows :entries posting-rows :postings} (rows {:entries entries})
        stale (fn [table rows] (vec (remove (set (map :xt/id rows)) (held-ids node table book))))
        stale-entries (stale :entries entry-rows)
        stale-postings (stale :postings posting-rows)
        {:keys [written]} (store/put-changed! node (cond-> {}
                                                     (seq entry-rows) (assoc :entries entry-rows)
                                                     (seq posting-rows) (assoc :postings posting-rows))
                                              metadata)
        ending (assoc metadata :why (str (:why metadata) ": entries that no longer hold"))]
    (when (seq stale-entries) (store/delete! node :entries stale-entries {} ending))
    (when (seq stale-postings) (store/delete! node :postings stale-postings {} ending))
    {:entries (count entry-rows)
     :written (get written :entries 0)
     :postings (count posting-rows)
     :ended (count stale-entries)}))

(defn balances [node book {:keys [from to]}]
  (if-not (store/table-exists? node :postings)
    {}
    (into {}
          (map (juxt :account :amount))
          (store/q node (cond-> [(str "SELECT account, SUM(amount) AS amount FROM postings WHERE book = ?"
                                      (when from " AND date >= ?")
                                      (when to " AND date <= ?")
                                      " GROUP BY account")
                                 book]
                          from (conj from)
                          to (conj to))))))

(defn postings [node book account {:keys [from to]}]
  (if-not (store/table-exists? node :postings)
    []
    ;; Whole rows: XTDB refuses to name a column no row has yet (a description, a narration).
    (let [rows (store/q node (cond-> [(str "SELECT * FROM postings WHERE book = ? AND account = ?"
                                           (when from " AND date >= ?")
                                           (when to " AND date <= ?")
                                           " ORDER BY date, entry, line")
                                      book account]
                               from (conj from)
                               to (conj to)))
          entries (if (seq rows)
                    (into {} (map (juxt :xt/id identity)) (store/q node ["SELECT * FROM entries WHERE book = ?" book]))
                    {})]
      (mapv (fn [r] (let [e (entries (:entry r))]
                      (cond-> (select-keys r [:entry :line :date :amount :commodity :party :description])
                        (:narration e) (assoc :narration (:narration e))
                        (:document e) (assoc :document (:document e)))))
            rows))))
