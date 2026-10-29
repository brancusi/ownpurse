(ns ownpurse.scenario.lines
  "Saved sets of lines: a selection over the books, with a title and its own key (lines a person coded), kept as a
  line_sets row by name. Any scenario runs over one; a scenario's lines layer is a set too, the one it reads by
  default. A new version replaces the row; its history stays."
  (:require
   [clojure.edn :as edn]
   [clojure.string :as str]
   [ownpurse.scenario.coding :as coding]
   [ownpurse.store.interface :as store])
  (:import
   (java.time LocalDate)))

(def ^:private metadata {:source "ownpurse" :why "lines: a saved set"})

(defn- fail! [type message data] (throw (ex-info message (assoc data :type type))))

(defn- row-id [name] (str "line-set:" name))

(defn- check-name! [name]
  (when-not (and (string? name) (re-matches #"[a-z0-9][a-z0-9-]*" name))
    (fail! :ownpurse/invalid (str "A set of lines is named in small letters, digits and dashes: " (pr-str name) ".")
           {:name name})))

(defn- out [{:keys [select key] :as row}]
  (cond-> (dissoc row :xt/id :select :key)
    select (assoc :select (edn/read-string select))
    key (assoc :key (edn/read-string key))))

(defn line-set
  "Saved set `name`, {:name :title :note :select :key}, or nil."
  [node name]
  (when (store/table-exists? node :line-sets)
    (some-> (first (store/q node ["SELECT * FROM line_sets WHERE _id = ?" (row-id name)])) out)))

(defn line-sets
  "Every saved set, by name."
  [node]
  (if (store/table-exists? node :line-sets)
    (mapv out (store/q node ["SELECT * FROM line_sets ORDER BY name"]))
    []))

(defn put!
  "Saves set `name`: {:title :note :select}, the selection as ownpurse.scenario.coding/select-in checks it. Its key
  is kept."
  [node {:keys [name title note select]}]
  (check-name! name)
  (when (str/blank? title) (fail! :ownpurse/invalid "Give the set of lines a title." {:name name}))
  (coding/select-in select)
  (let [key (:key (line-set node name))
        ;; Kept as EDN text: the store would turn a selection's strings and keys into keywords.
        row (cond-> {:xt/id (row-id name) :name name :title title :select (pr-str select)}
              note (assoc :note note)
              key (assoc :key (pr-str key)))]
    (store/put! node {:line-sets [row]} metadata)
    (line-set node name)))

(defn confirm!
  "Keys one line in set `name`: {:unit :expect :ok :review :why}, replacing its entry; {:unit :remove true} takes it
  out."
  [node name {:keys [unit] :as entry}]
  (let [s (or (line-set node name) (fail! :ownpurse/not-found (str "There is no set of lines " name ".") {:name name}))
        others (vec (remove #(= unit (:unit %)) (get-in s [:key :lines])))
        lines (if (:remove entry) others (conj others (-> (select-keys entry [:unit :expect :ok :review :why :by])
                                                 (assoc :at (str (LocalDate/now))))))
        key (coding/key-in {:lines lines})]
    (store/put! node {:line-sets [(-> (dissoc s :select :key)
                                      (assoc :xt/id (row-id name) :select (pr-str (:select s)) :key (pr-str key)))]}
                metadata)
    (line-set node name)))

(defn answer-key
  "Set `name`'s key by unit id, as a scenario's is: buckets as keywords, :in the set's id."
  [s]
  (into {} (for [{:keys [unit] :as e} (get-in s [:key :lines])]
             [unit (cond-> (assoc e :in (:name s) :expect (keyword (:expect e)))
                     (:ok e) (update :ok (partial mapv keyword))
                     (:review e) (update :review keyword))])))
