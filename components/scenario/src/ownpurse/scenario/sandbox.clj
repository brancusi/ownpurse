(ns ownpurse.scenario.sandbox
  "The record's coding scenarios in the sandbox's shapes (ownpurse.sandbox), so the admin's sandbox and its guide show
  a book's own bank lines as they show the invented ones. A scenario's coding is one run there, \"latest\": the one
  its runs left. A line a person has keyed (a key layer) is scored right, wrong, unsafe or sent to a person
  needlessly, as the invented ones are; one nobody has keyed only coded or sent to a person. What Jev was asked and answered is rebuilt from the line's facts and read from the answers the record keeps."
  (:require
   [clojure.edn :as edn]
   [clojure.string :as str]
   [clojure.walk :as walk]
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

;; ----------------------------------------------------------------------------------------------- the trace

(defn- plain
  "A value as a client can show it: dates and instants as their text."
  [x]
  (walk/postwalk #(if (instance? java.time.temporal.Temporal %) (str %) %) x))

(defn trace
  "One unit of scenario `ref` through the pipeline, step by step as the guide tells it, by calling each step's
  function again on the record's data: {:step :call :about :in :out :db}. :db is every query the step made, with
  its parameters and what came back; a run's writes are shown as the rows it wrote, read back. Nothing is asked of
  Jev: its answers are the ones the record keeps."
  [node ref unit-id]
  (let [log (atom [])
        q! (fn [sql & params]
             (let [rows (if (store/table-exists? node (keyword (second (re-find #"FROM (\w+)" sql))))
                          (store/q node (into [sql] params))
                          [])]
               (swap! log conj {:sql sql :params (vec params) :rows (count rows) :row (first rows)})
               rows))
        step (fn [m] (let [db @log] (reset! log []) (plain (assoc m :db db))))
        stack (concat (reverse (core/bases-of node ref)) [ref])
        _ (doseq [r stack] (q! "SELECT * FROM scenarios WHERE _id = ?" (str "scenario:" r)))
        s (core/classifier-scenario node ref)
        units (mapcat classify/units (:cases s))
        unit (or (some #(when (= unit-id (:id %)) %) units)
                 (throw (ex-info (str "Scenario " ref " does not read " unit-id ".") {:type :ownpurse/not-found})))
        line-id (or (:line unit) unit-id)
        ;; The lines layers' own selections, as ownpurse.scenario.coding/read-lines makes them; the row shown is this line's.
        raw (some (fn [{:keys [accounts from to]}]
                    (let [sql (str "SELECT * FROM bank_lines WHERE account IN (" (str/join ", " (repeat (count accounts) "?")) ")"
                                   (when from " AND date >= ?") (when to " AND date <= ?") " ORDER BY date, _id")
                          params (cond-> (vec accounts) from (conj (java.time.LocalDate/parse (str from)))
                                   to (conj (java.time.LocalDate/parse (str to))))
                          rows (apply q! sql params)
                          row (some #(when (= line-id (:xt/id %)) %) rows)]
                      (swap! log #(conj (pop %) (assoc (peek %) :row row :shown "this line's row")))
                      row))
                  (:selects (core/composed node ref)))
        case (some #(when (= line-id (:id %)) %) (:cases s))
        model (:model (first (run/runs node ref)))
        held (fn [{:keys [state questions]}]
               (let [id (run/judgment-id model state questions)]
                 [id (some-> (first (q! "SELECT * FROM judgments WHERE _id = ?" id)) :response edn/read-string)]))
        world (select-keys s [:home :business :entities :cards :projects :trips :categories])
        ;; With an answer missing (never coded, or the scenario changed since), there is nothing to decide from.
        missing "No answer kept for this line's requests as the scenario stands: run it, and this step is shown."
        s1 (step {:step 1 :call "ownpurse.scenario.coding/read-lines → case-of"
                  :about (str "The scenario's layers are read (its bases' first) and composed; its lines layer "
                              "selects the bank lines, and each row becomes a case: money out as a positive "
                              "amount, in the merchant's currency, on the day the card was used.")
                  :in {:bank-line raw} :out case})
        s2 (step {:step 2 :call "(ownpurse.classify.interface/units case)"
                  :about "A line is one unit, or one unit per item its receipt or order lists."
                  :in {:case case} :out (vec (classify/units case))})
        enrich (classify/enrichment s unit)
        [enrich-id enrich-response] (when enrich (held enrich))
        category (some->> enrich-response :answers :category (classify/enriched-category s))
        s4 (step {:step 4 :call "(ownpurse.classify.interface/enrichment s unit), then enriched-category"
                  :about (if enrich
                           (str "No merchant code: Jev is asked which kind of merchant it is (a Choice over the "
                                "scenario's categories). Its answer is looked up in the record by the request's "
                                "hash; at 50% or more it gives the line its category.")
                           "The bank sent a merchant code (or it is a transfer), so no request is made: nil.")
                  :in {:unit unit}
                  :out (when enrich {:request enrich :judgment-id enrich-id :answer enrich-response :category category})})
        facts (classify/facts s unit category)
        s3 (step {:step 3 :call "(ownpurse.classify.interface/facts s unit category)"
                  :about (str "Code looks things up and calculates, never guesses: the category from the merchant "
                              "code (or step 4's), the card's holder, where they were that day from the trips, the "
                              "projects running, an earlier purchase a refund returns. `s` is the composed "
                              "scenario; its world is shown.")
                  :in {:unit unit :category category :s-world world} :out facts})
        state (classify/state s facts)
        s5 (step {:step 5 :call "(ownpurse.classify.interface/state s facts)"
                  :about "The facts written out as the JSON Jev reads: the line itself, and what code adds about it."
                  :in {:facts facts} :out state})
        {:keys [rules plain]} (classify/requests s facts)
        id-of (fn [{:keys [state questions]}] (run/judgment-id model state questions))
        rules-id (id-of rules)
        plain-id (id-of plain)
        s6 (step {:step 6 :call "(ownpurse.classify.interface/requests s facts)"
                  :about (str "Two requests over the same state (step 5): :rules asks which bucket with the rules "
                              "as text, and whether each applies; :plain asks which bucket from the facts alone, and "
                              "those rules' fuzzy blocks' questions. Only rules no exact block rules out are sent. "
                              "Each is kept by the SHA-256 of [model state "
                              "questions], so the same request is never asked twice.")
                  :in {:facts facts :model model}
                  :out {:rules {:judgment-id rules-id :questions (:questions rules)}
                        :plain {:judgment-id plain-id :questions (:questions plain)}}})
        [_ rules-response] (held rules)
        [_ plain-response] (held plain)
        responses (cond-> {}
                    rules-response (assoc :rules rules-response)
                    plain-response (assoc :plain plain-response)
                    enrich-response (assoc :enrich enrich-response))
        s7 (step {:step 7 :call "ownpurse.scenario.run/caching-ask → ownpurse.classify.interface/jev-ask"
                  :about (str "Each request is looked up in the record's judgments by its id. One the record keeps "
                              "is answered from it; one it does not is sent (POST /v1/systemone with model, state "
                              "and questions) and its answer kept as a new judgments row. Here every answer is "
                              "read from the record; nothing is sent.")
                  :in {:judgment-ids {:rules rules-id :plain plain-id}}
                  :out (cond-> responses (not rules-response) (assoc :rules :none-kept) (not plain-response) (assoc :plain :none-kept))})
        result {:id unit-id :facts facts :responses responses}
        s8 (step {:step 8 :call "(ownpurse.classify.interface/explain s result)"
                  :about (str "Each rule's building blocks checked against the facts (the fuzzy ones by Jev's "
                              "answers in :plain); a rule holds as much as its weakest block. Jev's own judgment "
                              "of the whole rule (from :rules) beside it.")
                  :in {:result result} :out (if (and rules-response plain-response) (classify/explain s result) missing)})
        decisions (when (and rules-response plain-response)
                    (classify/decide s facts {:rules-answers (:answers rules-response) :plain-answers (:answers plain-response)}))
        s9 (step {:step 9 :call "(ownpurse.classify.interface/decide s facts answers)"
                  :about "Every strategy's pick from the same answers: Jev's Choice, stopped by checks, rule by rule, the blocks…"
                  :in {:answers {:rules-answers (:answers rules-response) :plain-answers (:answers plain-response)}}
                  :out (or decisions missing)})
        settings (select-keys (merge {:threshold run/default-threshold :strategy run/default-strategy} s) [:threshold :strategy])
        standing (if decisions (run/standing settings decisions) missing)
        _ (q! "SELECT * FROM codings WHERE _id = ?" (str "coding:" ref ":" unit-id))
        _ (q! "SELECT * FROM coding_runs WHERE scenario = ? ORDER BY at DESC LIMIT 1" ref)
        s10 (step {:step 10 :call "(ownpurse.scenario.run/standing {:threshold :strategy} decisions)"
                   :about (str "The scenario's strategy's coding stands when it is a bucket and sure enough; else a "
                               "person looks. A run keeps it as one codings row per unit (put only when changed), "
                               "and one coding_runs row; both are shown as the run wrote them.")
                   :in {:settings settings :decisions decisions} :out standing})]
    {:unit unit-id :scenario ref :model model :steps [s1 s2 s3 s4 s5 s6 s7 s8 s9 s10]}))
