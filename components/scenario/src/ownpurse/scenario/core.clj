(ns ownpurse.scenario.core
  (:require
   [clojure.string :as str]
   [ownpurse.close.interface :as close]
   [ownpurse.money.interface :as money]
   [ownpurse.scenario.coding :as coding]
   [ownpurse.schema.interface :as schema]
   [ownpurse.store.interface :as store])
  (:import
   (java.time LocalDate)))

(def ^:private metadata {:source "ownpurse" :why "scenario"})

(defn- fail! [type message data]
  (throw (ex-info message (assoc data :type type))))

(def ^:private ref-pattern #"[A-Za-z0-9][A-Za-z0-9._-]{0,63}")

(defn- check-ref! [what ref]
  (when-not (and (string? ref) (re-matches ref-pattern ref))
    (fail! :ownpurse/invalid (str "A " what " needs a reference of letters, digits, dots, dashes and underscores, "
                                  "such as rent-split-b.")
           {:ref ref})))

(defn- scenario-id [ref] (str "scenario:" ref))

(def kinds
  "The kinds of layer a scenario holds, each with what it says."
  {"journals" "Journals of the scenario's own, in any book: each must balance in every commodity."
   "close" "The close register's journals not yet posted: those of approved items (basis approved) or of every item not set aside (basis proposed)."
   "world" "Who and where, for coding bank lines: entities, cards and accounts with their holders, projects with their dates, trips, merchant categories."
   "coding" "Buckets and rules for coding bank lines: each rule a sentence and its building blocks. A rule replaces the rule of its id under it; enabled false takes it out."
   "lines" "Which bank lines the coding reads: a selection from the record (accounts, from, to), or lines given as they are."
   "key" "Lines a person has coded: the answer key the scenario's codings are scored against (right, wrong, or sent to a person needlessly). Never sent to Jev. A line keyed here replaces its key under it."})

(def built-in
  "Scenarios ownpurse gives every record, which cannot be changed or dropped: the close register as approved, and as
  proposed."
  {"approved" {:ref "approved" :name "The close register, approved" :built-in true
               :layers [{:id "close" :kind "close" :basis "approved" :enabled true}]}
   "proposed" {:ref "proposed" :name "The close register, proposed" :built-in true
               :layers [{:id "close" :kind "close" :basis "proposed" :enabled true}]}})

(defn- rows [node sql-and-params]
  (if (store/table-exists? node :scenarios) (store/q node sql-and-params) []))

(defn- out
  "A stored scenario as it is returned: without XTDB's columns, its amounts as BigDecimals."
  [row]
  (-> (dissoc row :xt/id :xt/valid-from :xt/valid-to :xt/system-from :xt/system-to)
      (update :layers (partial mapv (fn [l]
                                      (cond-> (coding/unstored l)
                                        (:journals l)
                                        (update :journals (partial mapv (fn [j] (update j :postings (partial mapv #(update % :amount bigdec))))))))))))

(defn scenario [node ref]
  (or (built-in ref)
      (some-> (first (rows node ["SELECT * FROM scenarios WHERE _id = ?" (scenario-id ref)])) out)))

(defn scenarios [node]
  (into (vec (vals (into (sorted-map) built-in)))
        (map out)
        (rows node ["SELECT * FROM scenarios ORDER BY ref"])))

(defn- check-journal! [ref layer {jref :ref :keys [book date narration postings]}]
  (check-ref! "journal" jref)
  (when (or (str/blank? book) (str/blank? narration) (not (instance? LocalDate date)))
    (fail! :ownpurse/invalid (str "Journal " jref " of layer " layer " of scenario " ref
                                  " needs a book, a date and a narration.")
           {:ref ref :layer layer :journal jref}))
  (schema/check! :ownpurse/entry
                 {:entry/id (str (scenario-id ref) ":" layer ":" jref)
                  :entry/book book
                  :entry/date date
                  :entry/postings (vec (map-indexed (fn [i {:keys [account amount commodity description]}]
                                                      (cond-> {:posting/line i :posting/account account
                                                               :posting/amount amount :posting/commodity commodity}
                                                        description (assoc :posting/description description)))
                                                    postings))}))

(defn- layer-in
  "A layer as it is kept: only the keys its kind reads, enabled unless said otherwise."
  [ref {:keys [id kind basis journals] :as layer}]
  (check-ref! "layer" id)
  (when-not (kinds kind)
    (fail! :ownpurse/invalid (str "Layer " id " of scenario " ref " is of kind " (pr-str kind) "; a layer is one of "
                                  (str/join ", " (keys kinds)) ".")
           {:ref ref :layer id}))
  (cond-> {:id id :kind kind :enabled (if (contains? layer :enabled) (boolean (:enabled layer)) true)}
    (:note layer) (assoc :note (:note layer))
    (= "close" kind)
    (assoc :basis (or (#{"approved" "proposed"} basis)
                      (fail! :ownpurse/invalid (str "Layer " id " of scenario " ref " reads the close register: say "
                                                    "basis approved or proposed.")
                             {:ref ref :layer id})))
    (= "world" kind) (assoc :world (coding/world-in (or (:world layer) {})))
    (= "coding" kind) (assoc :coding (or (:coding layer) {}))
    (= "lines" kind) (assoc :lines (coding/lines-in (or (:lines layer) {})))
    (= "key" kind) (assoc :key (coding/key-in (or (:key layer) {})))
    (= "journals" kind)
    (assoc :journals
           (do (when-let [dupes (seq (keep (fn [[r n]] (when (> n 1) r)) (frequencies (map :ref journals))))]
                 (fail! :ownpurse/invalid (str "Layer " id " of scenario " ref " names journal " (first dupes) " twice.")
                        {:ref ref :layer id}))
               (run! (partial check-journal! ref id) journals)
               (mapv (fn [j]
                       (-> (select-keys j [:ref :book :date :narration :evidence])
                           (assoc :postings (mapv #(-> (select-keys % [:account :amount :commodity :description])
                                                       (update :amount money/amount))
                                                  (:postings j)))))
                     journals)))))

(defn bases-of
  "The refs of the scenarios `ref` stands on, nearest first; throws when one is missing."
  [node ref]
  (loop [r (:base (scenario node ref)) seen [ref]]
    (cond
      (nil? r) (subvec seen 1)
      (some #{r} seen) (fail! :ownpurse/invalid (str "Scenario " ref " stands on itself, through " (str/join " and " (rest seen)) ".")
                              {:ref ref :bases (vec (rest seen))})
      :else (let [s (or (scenario node r)
                        (fail! :ownpurse/not-found (str "Scenario " (peek seen) " stands on " r ", which there is not.")
                               {:ref (peek seen) :base r}))]
              (recur (:base s) (conj seen r))))))

(declare stack-layers)

(defn- check-codings
  "Each coding layer's rules checked against the buckets of the coding layers under it and its own."
  [node base layers]
  (let [below (if base (:codes (coding/compose (stack-layers node base))) {})
        [out codes] (reduce (fn [[out codes] l]
                              (if (= "coding" (:kind l))
                                (let [c (coding/coding-in codes (:coding l))]
                                  [(conj out (assoc l :coding c)) (merge codes (:codes c))])
                                [(conj out l) codes]))
                            [[] below]
                            layers)]
    ;; A key's buckets are the whole stack's, wherever in it the coding layer that names them stands.
    (doseq [l out :when (= "key" (:kind l))] (coding/check-key codes (:key l)))
    out))

(defn put! [node {:keys [ref name note base layers duplicated-from]}]
  (check-ref! "scenario" ref)
  (when (built-in ref)
    (fail! :ownpurse/scenario-conflict (str "Scenario " ref " is ownpurse's own; duplicate it to change it.") {:ref ref}))
  (when (str/blank? name)
    (fail! :ownpurse/invalid "A scenario needs a name." {:ref ref}))
  (let [layers (mapv (partial layer-in ref) layers)]
    (when-let [dupes (seq (keep (fn [[id n]] (when (> n 1) id)) (frequencies (map :id layers))))]
      (fail! :ownpurse/invalid (str "Scenario " ref " has two layers called " (first dupes) ".") {:ref ref}))
    (when base
      (when-not (scenario node base)
        (fail! :ownpurse/not-found (str "Scenario " ref " stands on " base ", which there is not.") {:ref ref :base base}))
      (when (or (= base ref) (some #{ref} (bases-of node base)))
        (fail! :ownpurse/invalid (str "Scenario " ref " cannot stand on " base ", which stands on it.") {:ref ref :base base})))
    (let [world (coding/compose (concat (when base (stack-layers node base)) layers))]
      (doseq [[id p] (:projects world) :when (str/blank? (:name p))]
        (fail! :ownpurse/invalid (str "Scenario " ref " amends project " id ", which no layer of it or under it names.")
               {:ref ref :project id})))
    (let [layers (mapv coding/stored (check-codings node base layers))]
      (store/put-changed! node {:scenarios [(cond-> {:xt/id (scenario-id ref) :ref ref :name name :layers layers}
                                              note (assoc :note note)
                                              base (assoc :base base)
                                              duplicated-from (assoc :duplicated-from duplicated-from))]}
                          metadata))
    (scenario node ref)))

(defn create! [node {:keys [ref] :as s}]
  (when (scenario node ref)
    (fail! :ownpurse/scenario-conflict (str "There is already a scenario " ref ": pick another ref, or change that one.")
           {:ref ref}))
  (put! node s))

(defn put-layer! [node ref layer]
  (let [s (or (scenario node ref) (fail! :ownpurse/not-found (str "There is no scenario " ref ".") {:ref ref}))
        layers (:layers s)
        at (first (keep-indexed #(when (= (:id layer) (:id %2)) %1) layers))]
    (put! node (assoc s :layers (if at (assoc layers at layer) (conj layers layer))))))

(defn duplicate! [node from {:keys [ref name]}]
  (let [s (or (scenario node from) (fail! :ownpurse/not-found (str "There is no scenario " from ".") {:ref from}))]
    (when (scenario node ref)
      (fail! :ownpurse/scenario-conflict (str "There is already a scenario " ref ".") {:ref ref}))
    (put! node (-> (select-keys s [:base :layers :note])
                   (assoc :ref ref :name (or name (str (:name s) " (copy)")) :duplicated-from from)))))

(defn drop! [node ref]
  (when (built-in ref)
    (fail! :ownpurse/scenario-conflict (str "Scenario " ref " is ownpurse's own; it cannot be dropped.") {:ref ref}))
  (when-not (scenario node ref)
    (fail! :ownpurse/not-found (str "There is no scenario " ref ".") {:ref ref}))
  (when-let [on (seq (keep #(when (= ref (:base %)) (:ref %)) (scenarios node)))]
    (fail! :ownpurse/scenario-conflict (str "Scenarios stand on " ref ": " (str/join ", " on) ". Drop or move them first.")
           {:ref ref :standing-on (vec on)}))
  (store/delete! node :scenarios [(scenario-id ref)] {} (assoc metadata :why "scenario dropped"))
  {:ref ref :dropped true})

(defn versions [node ref]
  (if (built-in ref)
    [(assoc (built-in ref) :version 1)]
    (->> (rows node ["SELECT *, _system_from FROM scenarios FOR ALL VALID_TIME WHERE _id = ? ORDER BY _system_from"
                     (scenario-id ref)])
         (map-indexed (fn [i r] (assoc (out r) :version (inc i) :at (str (:xt/system-from r)))))
         vec)))

(defn- within? [^LocalDate d from to]
  (and (or (nil? from) (not (.isBefore d from))) (or (nil? to) (not (.isAfter d to)))))

(defn- layer-balances
  "{account amount}: what one layer adds to `book` over the dates `from` to `to`."
  [node book {:keys [from to]} {:keys [kind basis journals]}]
  (case kind
    "close" (close/balances node book {:from from :to to :basis (keyword basis)})
    "journals" (reduce (fn [acc {:keys [account amount]}] (update acc account (fnil + 0M) amount))
                       {}
                       (for [j journals :when (and (= book (:book j)) (within? (:date j) from to))
                             p (:postings j)]
                         p))
    {}))

(defn evaluate [node ref book opts]
  (let [s (or (scenario node ref) (fail! :ownpurse/not-found (str "There is no scenario " ref ".") {:ref ref}))
        stack (concat (reverse (bases-of node ref)) [ref])
        layers (vec (for [r stack
                          {:keys [id kind enabled] :as l} (:layers (scenario node r))]
                      {:scenario r :layer id :kind kind :enabled enabled
                       :balances (if enabled (layer-balances node book opts l) {})}))]
    {:scenario ref :name (:name s) :book book :from (:from opts) :to (:to opts)
     :stack (vec stack)
     :layers layers
     :balances (into {} (remove (comp zero? val)) (apply merge-with + {} (map :balances layers)))}))

(defn stack-layers
  "Every layer of scenario `ref` that is on, its bases' first."
  [node ref]
  (when-not (scenario node ref)
    (fail! :ownpurse/not-found (str "There is no scenario " ref ".") {:ref ref}))
  (vec (for [r (concat (reverse (bases-of node ref)) [ref])
             l (:layers (scenario node r))
             :when (:enabled l)]
         l)))

(defn composed [node ref]
  (-> (coding/compose (stack-layers node ref))
      (assoc :scenario ref)))

(defn classifier-scenario [node ref]
  (let [{:keys [selects] :as s} (coding/compose (stack-layers node ref))
        read (vec (mapcat #(coding/read-lines node s %) selects))]
    (-> (dissoc s :selects)
        (assoc :scenario ref)
        (update :cases (fnil into []) read))))

(defn answer-key
  "Scenario `ref`'s answer key, its bases' first: each unit a person coded, by unit id, with the scenario it was keyed
  in (:in); a key layer nearer the scenario replaces a unit's key under it. Buckets as keywords, as codings say them."
  [node ref]
  (reduce (fn [acc [r l]]
            (reduce (fn [acc {:keys [unit] :as e}]
                      (assoc acc unit (cond-> (assoc e :in r :expect (keyword (:expect e)))
                                        (:ok e) (update :ok (partial mapv keyword))
                                        (:review e) (update :review keyword))))
                    acc (get-in l [:key :lines])))
          {}
          (for [r (concat (reverse (bases-of node ref)) [ref])
                l (:layers (scenario node r))
                :when (and (:enabled l) (= "key" (:kind l)))]
            [r l])))

(defn confirm!
  "Keys one unit in scenario `ref`'s own key layer (the first, or a new one, \"key\", last): {:unit :expect :ok
  :review :why}, replacing the unit's entry there; {:unit :remove true} takes the unit out of this scenario's key
  (a base's key of it stands). A new version of the scenario, like any change."
  [node ref {:keys [unit] :as entry}]
  (let [s (scenario node ref)]
    (when-not s (fail! :ownpurse/not-found (str "There is no scenario " ref ".") {:ref ref}))
    (when (:built-in s)
      (fail! :ownpurse/scenario-conflict (str "Scenario " ref " is built in: key lines in a scenario standing on it.")
             {:ref ref}))
    (let [layers (:layers s)
          i (first (keep-indexed (fn [i l] (when (= "key" (:kind l)) i)) layers))
          layers (if i layers (conj layers {:id "key" :kind "key" :key {:lines []}
                                            :note "Lines a person has coded, for scoring this scenario's codings."}))
          i (or i (dec (count layers)))
          others (vec (remove #(= unit (:unit %)) (get-in layers [i :key :lines])))
          lines (if (:remove entry) others (conj others (-> (select-keys entry [:unit :expect :ok :review :why :by])
                                                   (assoc :at (str (LocalDate/now))))))]
      (put! node (assoc s :layers (assoc-in layers [i :key :lines] lines))))))
