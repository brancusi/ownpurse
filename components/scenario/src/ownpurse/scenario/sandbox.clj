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
   [ownpurse.scenario.coding :as coding]
   [ownpurse.scenario.core :as core]
   [ownpurse.scenario.lines :as lines]
   [ownpurse.scenario.run :as run]
   [ownpurse.store.interface :as store]))

(def run-id "The one run a record scenario has in the sandbox: its coding as its runs left it." "latest")

(defn coding-scenario?
  "Whether scenario `ref` codes lines: a coding or lines layer is on in its stack."
  [node ref]
  (boolean (some #(#{"lines" "coding"} (:kind %)) (core/stack-layers node ref))))

(defn- coded-rows
  "The codings kept under `scope`: a scenario's ref over its own lines, <ref>@<set> over another set."
  [node scope]
  (if (store/table-exists? node :codings)
    (store/q node ["SELECT * FROM codings WHERE scenario = ? ORDER BY date, unit" scope])
    []))

(defn run-of
  "The sandbox's id of a run over set `over-id`: \"latest\" over the scenario's own lines, else the set's id."
  [over-id]
  (or over-id run-id))

(defn- buckets [s]
  (or (:buckets s)
      [{:title "Buckets" :codes (vec (for [[c _] (:codes s)] [c (str/capitalize (str/replace (name c) "-" " "))]))}]))

(defn- instant [t] (str (if (instance? java.time.ZonedDateTime t) (.toInstant ^java.time.ZonedDateTime t) t)))

(defn- run-summary [node scope over-id n]
  (when-let [{:keys [at model asked input-tokens]} (first (run/runs node scope))]
    (cond-> {:id (run-of over-id) :at (instant at) :model model :results n}
      over-id (assoc :over over-id)
      input-tokens (assoc :usage {:requests asked :input-tokens input-tokens}))))

(defn- runs-listed
  "Scenario `ref`'s runs as the sandbox lists them, newest first: one a set it ran over, its last run's time; over
  its own lines \"latest\", once it has coded them."
  [node ref]
  (let [coded (seq (coded-rows node ref))]
    (->> (run/runs-of node ref)
         (filter #(or (:over %) coded))
         (reduce (fn [acc {:keys [over at]}] (if (some #(= (run-of over) (:id %)) acc) acc
                                                (conj acc (cond-> {:id (run-of over) :at (instant at)} over (assoc :over over)))))
                 []))))

(defn scenarios
  "Every record scenario that codes lines, as the sandbox lists its own: {:name :title :record true :default-set
  :lines :units :rules :coded :runs}: its runs, one a set of lines it ran over (\"latest\" over its own lines)."
  [node]
  (vec (for [{:keys [ref] :as sc} (core/scenarios node)
             :when (coding-scenario? node ref)
             :let [s (core/classifier-scenario node ref)
                   coded (seq (coded-rows node ref))]]
         {:name ref :title (:name sc) :record true :base (:base sc)
          :layers (vec (for [{:keys [id kind note enabled]} (:layers sc)]
                         (cond-> {:id id :kind kind :enabled enabled} note (assoc :note note))))
          :buckets (vec (for [{:keys [codes]} (buckets s) [_ label] codes] label))
          :lines (count (:cases s))
          :units (count (mapcat classify/units (:cases s)))
          :rules (count (:rules s))
          :coded (count coded)
          :default-set (core/default-set node ref)
          :runs (runs-listed node ref)})))

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
  "Scenario `ref`'s coding in the sandbox's run shape, over its own lines or over set `over` (as
  ownpurse.scenario.core/set-over gives it): one row a unit it coded, each strategy's coding."
  [node ref over]
  (let [{:keys [scope s key] over-id :over} (core/over node ref over)
        units (mapcat classify/units (:cases s))
        by-id (into {} (map (juxt :id identity)) units)
        short (short-ids units)
        k key
        rows (coded-rows node scope)]
    {:run (run-summary node scope over-id (count rows))
     :lines (vec (for [{:keys [unit result]} rows
                       :let [{:keys [decisions]} (edn/read-string result)]]
                   {:id unit
                    :short (short unit)
                    :unit (keyed k (dissoc (or (by-id unit) {:id unit}) :items))
                    :decisions (update-vals decisions #(select-keys % [:code :confidence :reason]))}))}))

(defn line
  "One unit of scenario `ref`'s coding all the way through, as the sandbox's line: the bank line, its facts, the
  requests rebuilt from them, the answers the record kept for those requests, how each rule held, and each strategy's
  decision. Over its own lines, or over set `over`."
  [node ref over unit-id]
  (let [{:keys [scope s key] over-id :over} (core/over node ref over)
        row (first (store/q node ["SELECT * FROM codings WHERE scenario = ? AND unit = ?" scope unit-id]))
        _ (when-not row
            (throw (ex-info (str "Scenario " ref " has not coded " unit-id ".") {:type :ownpurse/not-found})))
        {:keys [facts decisions]} (edn/read-string (:result row))
        units (mapcat classify/units (:cases s))
        unit (some #(when (= unit-id (:id %)) %) units)
        line (some #(when (= (or (:line unit) unit-id) (:id %)) %) (:cases s))
        model (:model (first (run/runs node scope)))
        enrich (classify/enrichment s unit)
        requests (cond-> (classify/requests s facts) enrich (assoc :enrich enrich))
        responses (into {} (keep (fn [[k {:keys [state questions]}]]
                                   (when-let [held (run/held-judgment node (run/judgment-id model state questions))]
                                     [k held])))
                        requests)
        result {:id unit-id :facts facts :responses responses :decisions decisions}]
    {:id unit-id
     :short (get (short-ids units) unit-id)
     :run (run-summary node scope over-id (count (coded-rows node scope)))
     :unit (keyed key unit)
     :line line
     :facts facts
     :requests (dissoc requests :enrich)
     :enrich enrich
     :record true
     ;; A line read from the books (not given as it is, as a sample's): it can be chosen on its own, as a set.
     :books (boolean (if over-id (:select over) (seq (:selects (core/composed node ref)))))
     :rebuilt true
     ;; Asked for when the line was coded, but under rules that have changed since: nothing kept for them as they are.
     :unanswered (vec (remove responses (keys requests)))
     :responses responses
     :decisions decisions
     :explained (classify/explain s result)}))

;; ----------------------------------------------------------------------------------------------- lines

(defn- account-names [node]
  (if (store/table-exists? node :source-accounts)
    (into {} (map (juxt :xt/id :name)) (store/q node ["SELECT _id, name FROM source_accounts"]))
    {}))

(defn- lines-title [names {:keys [select cases]}]
  (if select
    (let [{:keys [accounts all from to]} select]
      (str (if all "Every book's lines" (str/join ", " (map #(get names % %) accounts)))
           (cond (and from to) (str ", " from " to " to) from (str ", from " from) to (str ", to " to))))
    (str (count cases) " lines given as they are")))

(defn- lines-where
  "What a set of lines is, as the query that chooses it: [[field value] ...]."
  [names {:keys [select cases]}]
  (if select
    (let [{:keys [accounts all lines from to]} select]
      (cond-> []
        all (conj ["account" "every book's"])
        (and (seq accounts) (not all)) (conj ["account" (str/join " or " (map #(get names % %) accounts))])
        (seq lines) (conj ["lines" (if (= 1 (count lines)) "this one" (str (count lines) " chosen"))])
        from (conj ["from" (str from)])
        to (conj ["to" (str to)])))
    [["given" (str (count cases) " lines, as they are")]]))

(defn glance
  "A line as a preview shows it: when, where, how much, what kind of merchant (its code, and its name where
  `categories` has it), how it was paid, and how many items its receipt lists."
  [categories {:keys [id date merchant amount currency items mcc channel]}]
  (cond-> {:id id :date date :merchant merchant :amount amount :currency currency}
    mcc (assoc :mcc mcc)
    (get categories mcc) (assoc :category (get categories mcc))
    channel (assoc :channel (name channel))
    (seq items) (assoc :items (count items))))

(defn- category-names
  "Every merchant category's name the workspace's coding scenarios know, by code."
  [node]
  (into {} (for [{:keys [ref]} (core/scenarios node)
                 :when (coding-scenario? node ref)
                 c (:categories (core/composed node ref))]
             c)))

(defn- sets-with-cases [node]
  (let [names (account-names node)
        glance (partial glance (category-names node))
        coding (filter #(coding-scenario? node %) (map :ref (core/scenarios node)))
        reading (for [ref coding [owner id l] (core/lines-layers node ref)] [[owner id l] ref])
        read (fn [lines] (sort-by :date (if-let [select (:select lines)] (coding/read-lines node {} select) (:cases lines))))]
    (concat
     (for [[[owner id {:keys [lines note]}] refs] (group-by first reading)
          :let [cases (read lines)]]
      [(cond-> {:id (core/layer-set-id owner id) :kind "books" :title (lines-title names lines) :lines (count cases)
                :where (lines-where names lines)
                :units (count (mapcat classify/units cases))
                :scenarios (vec (distinct (cons owner (sort (map second refs)))))
                :peek (mapv glance (take 3 cases))}
         note (assoc :note note))
       cases])
     ;; Saved sets: read by no scenario by default, run under any.
     (for [{:keys [name title note select]} (lines/line-sets node)
           :let [cases (read {:select select})]]
       [(cond-> {:id name :kind "books" :saved true :title title :lines (count cases)
                 :where (lines-where names {:select select})
                 :units (count (mapcat classify/units cases)) :scenarios [] :peek (mapv glance (take 3 cases))}
          note (assoc :note note))
        cases]))))

(defn line-sets
  "The sets of this workspace's own lines the sandbox picks from: one a lines layer of a scenario that codes lines,
  {:id \"<scenario>~<layer>\" :kind \"books\" :title :note :where :lines :units :scenarios :peek}, its :scenarios every
  scenario that reads it by default (the one that holds it first), :peek its first three lines; then the saved sets
  (:saved true, by name), which no scenario reads by default. A scenario reads the lines layers of its stack, its
  bases' included."
  [node]
  (mapv first (sets-with-cases node)))

(defn preview
  "A set's preview, from its lines oldest first: its first two hundred, the days they span, their currencies."
  [s cases categories]
  (assoc s
         :first (mapv (partial glance categories) (take 200 cases))
         :from (:date (first cases)) :to (:date (last cases))
         :currencies (into (sorted-map) (frequencies (keep :currency cases)))))

(defn line-set
  "Set `id` of the books' lines with a preview: its first two hundred lines, the days they span, and their currencies."
  [node id]
  (or (some (fn [[s cases]] (when (= id (:id s)) (preview s cases (category-names node)))) (sets-with-cases node))
      (throw (ex-info (str "No set of lines " id ".") {:type :ownpurse/not-found}))))

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
  [node ref over unit-id]
  (let [{:keys [scope s]} (core/over node ref over)
        log (atom [])
        q! (fn [sql & params]
             (let [rows (if (store/table-exists? node (keyword (second (re-find #"FROM (\w+)" sql))))
                          (store/q node (into [sql] params))
                          [])]
               (swap! log conj {:sql sql :params (vec params) :rows (count rows) :row (first rows)})
               rows))
        step (fn [m] (let [db @log] (reset! log []) (plain (assoc m :db db))))
        stack (concat (reverse (core/bases-of node ref)) [ref])
        _ (doseq [r stack] (q! "SELECT * FROM scenarios WHERE _id = ?" (str "scenario:" r)))
        units (mapcat classify/units (:cases s))
        unit (or (some #(when (= unit-id (:id %)) %) units)
                 (throw (ex-info (str "Scenario " ref " does not read " unit-id ".") {:type :ownpurse/not-found})))
        line-id (or (:line unit) unit-id)
        ;; The lines layers' own selections, as ownpurse.scenario.coding/read-lines makes them; the row shown is this line's.
        raw (some (fn [select]
                    (let [[sql & params] (coding/select-sql select)
                          rows (apply q! sql params)
                          row (some #(when (= line-id (:xt/id %)) %) rows)]
                      (swap! log #(conj (pop %) (assoc (peek %) :row row :shown "this line's row")))
                      row))
                  (if (= scope ref)
                    (:selects (core/composed node ref))
                    (some-> (:select over) vector)))
        case (some #(when (= line-id (:id %)) %) (:cases s))
        model (:model (first (run/runs node scope)))
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
        _ (q! "SELECT * FROM codings WHERE _id = ?" (str "coding:" scope ":" unit-id))
        _ (q! "SELECT * FROM coding_runs WHERE scenario = ? ORDER BY at DESC LIMIT 1" scope)
        s10 (step {:step 10 :call "(ownpurse.scenario.run/standing {:threshold :strategy} decisions)"
                   :about (str "The scenario's strategy's coding stands when it is a bucket and sure enough; else a "
                               "person looks. A run keeps it as one codings row per unit (put only when changed), "
                               "and one coding_runs row; both are shown as the run wrote them.")
                   :in {:settings settings :decisions decisions} :out standing})]
    {:unit unit-id :scenario ref :model model :steps [s1 s2 s3 s4 s5 s6 s7 s8 s9 s10]}))
