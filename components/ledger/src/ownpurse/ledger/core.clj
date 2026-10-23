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
