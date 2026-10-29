(ns ownpurse.scenario.run
  "Coding a scenario's lines: each unit classified (ownpurse.classify), every judgment kept in the record by the
  exact request that asked it, so coding again after an edit asks only what changed. Each line's coding is kept per
  scenario, its earlier codings in the record's history, and each run says what it asked and what it cost."
  (:require
   [clojure.edn :as edn]
   [ownpurse.classify.interface :as classify]
   [ownpurse.store.interface :as store])
  (:import
   (java.nio.charset StandardCharsets)
   (java.security MessageDigest)
   (java.time Instant)))

(def ^:private metadata {:source "ownpurse" :why "scenario: coding"})

(def default-threshold
  "The confidence a coding needs to stand without a person, unless the scenario's coding layer says another."
  0.85M)

(def default-strategy
  "The strategy whose coding stands, unless the scenario's coding layer names another."
  :checked)

(defn- sha-256 [^String s]
  (let [d (.digest (MessageDigest/getInstance "SHA-256") (.getBytes s StandardCharsets/UTF_8))]
    (apply str (map #(format "%02x" %) d))))

(defn- judgment-id [model state questions]
  (str "judgment:" (sha-256 (pr-str [model state questions]))))

(defn- held-judgment [node id]
  (when (store/table-exists? node :judgments)
    (some-> (first (store/q node ["SELECT response FROM judgments WHERE _id = ?" id])) :response edn/read-string)))

(defn caching-ask
  "`ask` (state, questions → a response) behind the record: a request asked before, of the same model, is answered
  from what the record kept; a new one is asked and kept. `counts` (an atom) counts :asked and :kept answers."
  [node model ask counts]
  (fn [state questions]
    (let [id (judgment-id model state questions)]
      (if-let [held (held-judgment node id)]
        (do (swap! counts update :kept (fnil inc 0)) held)
        (let [response (ask state questions)]
          (swap! counts update :asked (fnil inc 0))
          ;; Kept as EDN text: an answer's option names are data, and the store would make them keywords.
          (store/put! node {:judgments [{:xt/id id :model (str model) :response (pr-str response) :at (Instant/now)}]}
                      metadata)
          response)))))

(defn standing
  "The coding that stands for a unit: the strategy's, when it is a bucket and sure enough; else a person looks."
  [{:keys [threshold strategy]} decisions]
  (let [{:keys [code confidence reason by] :as d} (get decisions (or strategy default-strategy))
        sure (>= (bigdec (or confidence 0)) (or threshold default-threshold))]
    (cond-> {:code (if (and sure (not= :needs-person code)) code :needs-person)
             :proposed code
             :confidence (bigdec (or confidence 0))
             :auto (and sure (not= :needs-person code))}
      reason (assoc :reason reason)
      by (assoc :by by)
      (not d) (assoc :reason :no-decision))))

(defn- coding-id [ref unit] (str "coding:" ref ":" unit))

(defn plan
  "What coding classifier scenario `s` would take, before any request: its lines, its units, and at most how many
  requests (two a unit, and one more for a line with no merchant code); answers the record holds already are not
  asked again, so a second run costs less."
  [s]
  (let [units (mapcat classify/units (:cases s))]
    {:lines (count (:cases s))
     :units (count units)
     :at-most-requests (+ (* 2 (count units))
                          (count (remove #(or (:mcc %) (= :transfer (:channel %))) units)))}))

(defn code!
  "Codes every unit of classifier scenario `s` (scenario `ref`'s, as ownpurse.scenario.core/classifier-scenario
  composes it) with `ask`, four at a time, at most `limit` units when given. Keeps each unit's coding and the run.
  Returns the run: {:scenario :at :units :coded :asked :kept :model}."
  [node ref s ask {:keys [model limit on-unit]}]
  (let [counts (atom {})
        ask (caching-ask node model ask counts)
        units (cond->> (mapcat classify/units (:cases s)) limit (take limit))
        at (Instant/now)
        results (->> units
                     (partition-all 4)
                     (mapcat (fn [batch] (mapv deref (mapv #(future (doto (classify/classify s ask %) (as-> r (when on-unit (on-unit r)))))
                                                            batch))))
                     vec)
        codings (vec (for [{:keys [id facts decisions] :as r} results
                           :let [{:keys [code proposed confidence auto reason by]} (standing s decisions)]]
                       (cond-> {:xt/id (coding-id ref id) :scenario ref :unit id :line (or (:line facts) id)
                                :date (:date facts) :merchant (:merchant facts) :amount (:amount facts)
                                :currency (:currency facts)
                                :code (name code) :proposed (name proposed) :confidence confidence :auto auto
                                :result (pr-str (select-keys r [:id :facts :decisions]))}
                         reason (assoc :reason (name reason))
                         by (assoc :by (vec by)))))
        run {:xt/id (str "coding-run:" ref ":" at) :scenario ref :at at :units (count units)
             :coded (count (filter :auto codings)) :asked (get @counts :asked 0) :kept (get @counts :kept 0)
             :model (str model)}]
    (store/put-changed! node {:codings codings} metadata)
    ;; A whole run is the scenario's coding: a unit it no longer reads stops counting (its history stays).
    (when-not limit
      (let [now (set (map :xt/id codings))
            gone (remove now (map :xt/id (store/q node ["SELECT _id FROM codings WHERE scenario = ?" ref])))]
        (when (seq gone) (store/delete! node :codings (vec gone) {} (assoc metadata :why "scenario: lines no longer read")))))
    (store/put! node {:coding-runs [run]} metadata)
    (dissoc run :xt/id)))

(defn codings
  "Scenario `ref`'s coding of each unit, as its last run left it, by date: {:unit :line :date :merchant :amount
  :currency :code :proposed :confidence :auto :reason :by}."
  [node ref]
  (if-not (store/table-exists? node :codings)
    []
    (mapv #(dissoc % :xt/id :result :scenario)
          (store/q node ["SELECT * FROM codings WHERE scenario = ? ORDER BY date, unit" ref]))))

(defn coding
  "One unit's coding by scenario `ref`, all the way through: its facts and each strategy's decision."
  [node ref unit]
  (when (store/table-exists? node :codings)
    (when-let [row (first (store/q node ["SELECT * FROM codings WHERE _id = ?" (coding-id ref unit)]))]
      (merge (dissoc row :xt/id :result) (edn/read-string (:result row))))))

(defn runs
  "Scenario `ref`'s coding runs, newest first."
  [node ref]
  (if-not (store/table-exists? node :coding-runs)
    []
    (mapv #(dissoc % :xt/id) (store/q node ["SELECT * FROM coding_runs WHERE scenario = ? ORDER BY at DESC" ref]))))

(defn compare-codings
  "Each unit two scenarios code differently: {:unit :date :merchant :amount :a {:code :confidence} :b {...}}, a side
  nil where that scenario has not coded the unit."
  [node a b]
  (let [by-unit #(into {} (map (juxt :unit identity)) (codings node %))
        ca (by-unit a)
        cb (by-unit b)]
    (vec (for [u (sort (into (set (keys ca)) (keys cb)))
               :let [x (ca u) y (cb u)]
               :when (not= (:code x) (:code y))]
           (merge (select-keys (or x y) [:unit :date :merchant :amount :currency])
                  {:a (some-> x (select-keys [:code :proposed :confidence]))
                   :b (some-> y (select-keys [:code :proposed :confidence]))})))))
