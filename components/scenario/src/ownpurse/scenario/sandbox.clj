(ns ownpurse.scenario.sandbox
  "The record's coding scenarios in the sandbox's shapes (ownpurse.sandbox), so the admin's sandbox and its guide show
  a book's own bank lines as they show the invented ones. A scenario's coding is one run there, \"latest\": the one
  its runs left. A line a person has keyed (a key layer) is scored right, wrong, unsafe or sent to a person
  needlessly, as the invented ones are; one nobody has keyed only coded or sent to a person. What Jev was asked and answered is rebuilt from the line's facts and read from the answers the record keeps."
  (:require
   [clojure.edn :as edn]
   [clojure.string :as str]
   [ownpurse.classify.interface :as classify]
   [ownpurse.scenario.core :as core]
   [ownpurse.scenario.run :as run]
   [ownpurse.store.interface :as store]))

(def run-id "The one run a record scenario has in the sandbox: its coding as its runs left it." "latest")

(defn coding-scenario?
  "Whether scenario `ref` codes lines: a lines layer is on in its stack."
  [node ref]
  (boolean (some #(= "lines" (:kind %)) (core/stack-layers node ref))))

(defn- coded-rows [node ref]
  (if (store/table-exists? node :codings)
    (store/q node ["SELECT * FROM codings WHERE scenario = ? ORDER BY date, unit" ref])
    []))

(defn- buckets [s]
  (or (:buckets s)
      [{:title "Buckets" :codes (vec (for [[c _] (:codes s)] [c (str/capitalize (str/replace (name c) "-" " "))]))}]))

(defn- instant [t] (str (if (instance? java.time.ZonedDateTime t) (.toInstant ^java.time.ZonedDateTime t) t)))

(defn- run-summary [node ref n]
  (when-let [{:keys [at model asked input-tokens]} (first (run/runs node ref))]
    (cond-> {:id run-id :at (instant at) :model model :results n}
      input-tokens (assoc :usage {:requests asked :input-tokens input-tokens}))))

(defn scenarios
  "Every record scenario that codes lines, as the sandbox lists its own: {:name :title :record true :lines :units
  :rules :coded :runs}, its one run \"latest\" when it has been coded."
  [node]
  (vec (for [{:keys [ref] :as sc} (core/scenarios node)
             :when (coding-scenario? node ref)
             :let [s (core/classifier-scenario node ref)
                   coded (seq (coded-rows node ref))
                   {:keys [at]} (first (run/runs node ref))]]
         {:name ref :title (:name sc) :record true
          :lines (count (:cases s))
          :units (count (mapcat classify/units (:cases s)))
          :rules (count (:rules s))
          :coded (count coded)
          :runs (if (and coded at) [{:id run-id :at (instant at)}] [])})))

(defn world
  "What scenario `ref`'s lines happened in, as the sandbox's world: everything of its classifier scenario but the
  lines; and its :stack (itself, then the scenarios it stands on), where a line can be keyed."
  [node ref]
  (let [s (core/classifier-scenario node ref)]
    (-> (dissoc s :cases)
        (assoc :name ref :record true :buckets (buckets s)
               :stack (vec (for [r (cons ref (core/bases-of node ref))
                                 :let [sc (core/scenario node r)]
                                 :when (not (:built-in sc))]
                             {:ref r :name (:name sc)}))))))

(defn- keyed
  "A unit with what the answer key says of it: :expect :ok :review :why, and :keyed-in (the scenario that says it)."
  [k unit]
  (if-let [{:keys [in] :as e} (get k (:id unit))]
    (merge unit (select-keys e [:expect :ok :review :why]) {:keyed-in in})
    unit))

(defn- short-ids
  "Each unit's short name, by date: L1, L2 … (an item of a receipt keeps its number: L7/2)."
  [units]
  (into {} (map-indexed (fn [i {:keys [id]}]
                          [id (str "L" (inc i) (some->> (re-find #"/\d+$" (str id))))]))
        (sort-by (juxt :date :id) units)))

(defn run-lines
  "Scenario `ref`'s coding in the sandbox's run shape: one row a unit it coded, each strategy's coding."
  [node ref]
  (let [s (core/classifier-scenario node ref)
        units (mapcat classify/units (:cases s))
        by-id (into {} (map (juxt :id identity)) units)
        short (short-ids units)
        k (core/answer-key node ref)
        rows (coded-rows node ref)]
    {:run (run-summary node ref (count rows))
     :lines (vec (for [{:keys [unit result]} rows
                       :let [{:keys [decisions]} (edn/read-string result)]]
                   {:id unit
                    :short (short unit)
                    :unit (keyed k (dissoc (or (by-id unit) {:id unit}) :items))
                    :decisions (update-vals decisions #(select-keys % [:code :confidence :reason]))}))}))

(defn line
  "One unit of scenario `ref`'s coding all the way through, as the sandbox's line: the bank line, its facts, the
  requests rebuilt from them, the answers the record kept for those requests, how each rule held, and each strategy's
  decision."
  [node ref unit-id]
  (let [s (core/classifier-scenario node ref)
        row (first (store/q node ["SELECT * FROM codings WHERE scenario = ? AND unit = ?" ref unit-id]))
        _ (when-not row
            (throw (ex-info (str "Scenario " ref " has not coded " unit-id ".") {:type :ownpurse/not-found})))
        {:keys [facts decisions]} (edn/read-string (:result row))
        units (mapcat classify/units (:cases s))
        unit (some #(when (= unit-id (:id %)) %) units)
        line (some #(when (= (or (:line unit) unit-id) (:id %)) %) (:cases s))
        model (:model (first (run/runs node ref)))
        enrich (classify/enrichment s unit)
        requests (cond-> (classify/requests s facts) enrich (assoc :enrich enrich))
        responses (into {} (keep (fn [[k {:keys [state questions]}]]
                                   (when-let [held (run/held-judgment node (run/judgment-id model state questions))]
                                     [k held])))
                        requests)
        result {:id unit-id :facts facts :responses responses :decisions decisions}]
    {:id unit-id
     :short (get (short-ids units) unit-id)
     :run (run-summary node ref (count (coded-rows node ref)))
     :unit (keyed (core/answer-key node ref) unit)
     :line line
     :facts facts
     :requests (dissoc requests :enrich)
     :enrich enrich
     :record true
     :rebuilt true
     ;; Asked for when the line was coded, but under rules that have changed since: nothing kept for them as they are.
     :unanswered (vec (remove responses (keys requests)))
     :responses responses
     :decisions decisions
     :explained (classify/explain s result)}))
