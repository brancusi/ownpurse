(ns ownpurse.classify.eval
  "The classification eval: a scenario's cases, classified by every strategy, scored against the coding a bookkeeper
  would give. A run keeps every raw answer, so a new threshold or a new strategy over the same questions is scored
  again without asking anything. Run it with `clojure -M:dev -m ownpurse.classify.eval` (it needs a TypeSafe key)."
  (:require
   [clojure.edn :as edn]
   [clojure.java.io :as io]
   [clojure.pprint :as pprint]
   [ownpurse.classify.core :as core]
   [ownpurse.classify.facts :as facts]
   [ownpurse.classify.jev :as jev]
   [ownpurse.config.interface :as config])
  (:import
   (java.time Instant)))

(def thresholds [0M 0.5M 0.7M 0.9M])

(defn units
  "Every unit the eval codes in `scenario`: each case, or each item of a case whose receipt or order lists them."
  [scenario]
  (vec (mapcat facts/units (:cases scenario))))

(defn scenario
  "Reads the scenario `name` from the classpath (classify/eval/<name>.edn) and checks it."
  [name]
  (let [{:keys [codes rules cases categories cards] :as s}
        (edn/read-string (slurp (io/resource (str "classify/eval/" name ".edn"))))
        rule-ids (set (map :id rules))
        problems (concat
                  (for [{:keys [id code]} rules :when (not (contains? codes code))] [:rule id :unknown-code code])
                  (for [{:keys [id over]} rules o over :when (not (rule-ids o))] [:rule id :overrules-unknown o])
                  (for [{:keys [id expect ok]} (units s)
                        c (cons expect ok) :when (not (contains? codes c))] [:case id :unknown-code c])
                  (for [{:keys [id mcc card]} cases
                        :when (or (and mcc (not (contains? categories mcc))) (not (contains? cards card)))]
                    [:case id :unknown-category-or-card])
                  (for [[id n] (frequencies (map :id cases)) :when (> n 1)] [:case id :duplicate]))]
    (when (seq problems)
      (throw (ex-info (str "The scenario " name " does not hold together: " (vec problems))
                      {:type :ownpurse/bad-scenario :problems (vec problems)})))
    s))

(defn run
  "Classifies every unit of `scenario` (each case, or each item of one) with `ask`, four at a time. Returns the run: each case's facts, raw
  responses and codings, with the model and the tokens used."
  [scenario ask]
  (let [results (->> (units scenario)
                     (partition-all 4)
                     (mapcat (fn [batch] (mapv deref (mapv #(future (core/classify scenario ask %)) batch))))
                     vec)
        usage (for [r results resp (vals (:responses r))] (:usage resp))]
    {:scenario (:scenario scenario)
     :at (str (Instant/now))
     :model (some #(get-in % [:responses :rules :model]) results)
     :usage {:requests (count usage)
             :input-tokens (reduce + (map :input_tokens usage))
             :output-tokens (reduce + (map :output_tokens usage))}
     :results results}))

(defn outcome
  "How one coding fares at `threshold`: :right or :wrong when it was coded automatically, :stopped when it went to a
  person and should have, :review when it went to a person needlessly."
  [{:keys [expect ok]} {:keys [code confidence]} threshold]
  (let [acceptable (conj (set ok) expect)
        auto? (and (not= :needs-person code) (>= confidence threshold))]
    (cond
      (and auto? (acceptable code)) :right
      auto? :wrong
      (acceptable :needs-person) :stopped
      :else :review)))

(defn score
  "Each strategy's results on `run` at every threshold: how many lines were coded automatically, how many of those
  wrongly (the silent errors that matter most), and how many went to a person. Also each strategy's plain accuracy
  (its answer, ignoring confidence) and how often the Choice kept its answer with its options reversed."
  [scenario {:keys [results]}]
  (let [cases (into {} (map (juxt :id identity)) (units scenario))]
    {:cases (count results)
     :choice-stable (count (filter #(= (get-in % [:decisions :choice :code]) (get-in % [:decisions :reversed :code]))
                                   results))
     :strategies
     (into {}
           (for [s core/strategies]
             [s {:accurate (count (filter (fn [{:keys [id decisions]}]
                                            (let [{:keys [expect ok]} (cases id)]
                                              ((conj (set ok) expect) (get-in decisions [s :code]))))
                                          results))
                 :by-threshold (into (sorted-map)
                                     (for [t thresholds]
                                       [t (merge {:right 0 :wrong 0 :stopped 0 :review 0}
                                                 (frequencies (map #(outcome (cases (:id %)) (get-in % [:decisions s]) t)
                                                                   results)))]))
                 :wrong (into (sorted-map)
                              (for [t thresholds]
                                [t (vec (for [{:keys [id decisions]} results
                                              :when (= :wrong (outcome (cases id) (get decisions s) t))]
                                          [id (get-in decisions [s :code])]))]))}]))}))

(defn report
  "The score as a table to read."
  [{:keys [cases choice-stable strategies]}]
  (println (format "%d units. The Choice kept its answer with its options reversed on %d." cases choice-stable))
  (pprint/print-table
   (for [s core/strategies
         :let [{:keys [accurate by-threshold]} (strategies s)]
         t thresholds
         :let [{:keys [right wrong stopped review]} (by-threshold t)]]
     {:strategy (name s) :accurate accurate :threshold (str t)
      :auto-right right :auto-wrong wrong :stopped-rightly stopped :needless-review review})))

(defn load-run
  "Reads a saved run."
  [file]
  (edn/read-string (slurp file)))

(defn save!
  "Writes `run` to `dir` as <scenario>-<time>.edn and returns the file, once it reads back as the same run."
  [dir run]
  (let [f (io/file dir (str (:scenario run) "-" (.replace ^String (:at run) ":" "") ".edn"))]
    (io/make-parents f)
    (spit f (with-out-str (pprint/pprint run)))
    (when-not (= run (load-run f))
      (throw (ex-info (str "The run written to " f " does not read back as the same run")
                      {:type :ownpurse/unreadable-run :file (str f)})))
    f))

(defn -main
  "Runs the eval over the scenario named first (harbour by default) with Jev, saves the run beside the component's
  earlier runs and prints the score."
  [& [name]]
  (let [s (scenario (or name "harbour"))
        jev-config (:jev (config/read-config :dev))
        r (run s #(jev/ask jev-config %1 %2))
        f (save! "components/classify/runs" r)]
    (println "Saved" (str f) "-" (:usage r))
    (report (score s r))
    (shutdown-agents)))
