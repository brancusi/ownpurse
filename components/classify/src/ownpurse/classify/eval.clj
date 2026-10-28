(ns ownpurse.classify.eval
  "The classification eval: a scenario's cases, classified by every strategy, scored against the coding a bookkeeper
  would give. A run keeps every raw answer and the requests exactly as sent, so a new threshold or a new strategy over
  the same questions is scored again without asking anything. Run it with `clojure -M:dev -m ownpurse.classify.eval`
  (it needs a TypeSafe key).

  Each unit's answer key says two things apart: which codings are right (:expect, and :ok for other acceptable
  buckets), and whether a person must look (:review :required: coding it automatically is unsafe, however right the
  bucket) or may (:review :allowed: sending it to a person is not needless). A unit whose :expect is :needs-person has
  no bucket to give, and a person must look."
  (:require
   [clojure.edn :as edn]
   [clojure.java.io :as io]
   [clojure.java.shell :as shell]
   [clojure.pprint :as pprint]
   [clojure.string :as str]
   [ownpurse.classify.blocks :as blocks]
   [ownpurse.classify.core :as core]
   [ownpurse.classify.facts :as facts]
   [ownpurse.classify.jev :as jev]
   [ownpurse.config.interface :as config])
  (:import
   (java.security MessageDigest)
   (java.time Instant)))

(def thresholds [0M 0.5M 0.7M 0.85M 0.9M])

(def outcomes
  "How a coding fares, in the order the reports list them."
  [:right :wrong :unsafe :stopped :review])

(defn units
  "Every unit the eval codes in `scenario`: each case, or each item of a case whose receipt or order lists them."
  [scenario]
  (vec (mapcat facts/units (:cases scenario))))

(defn- exact-checks
  "The scenario's check rules that code alone decides (no block asks Jev)."
  [rules]
  (filter #(and (:check %) (empty? (blocks/fuzzy-questions [%]))) rules))

(defn- key-problems
  "What is wrong with the answer keys of `scenario`'s units: an unknown bucket, :needs-person among the acceptable
  buckets (a person looking is said with :review), a person not required where the key or an exact check requires
  one."
  [{:keys [codes rules] :as s}]
  (let [checks (exact-checks rules)]
    (concat
     (for [{:keys [id expect ok]} (units s) c (cons expect ok) :when (not (contains? codes c))]
       [:case id :unknown-code c])
     (for [{:keys [id ok]} (units s) :when (contains? (set ok) :needs-person)]
       [:case id :needs-person-in-ok])
     (for [{:keys [id review]} (units s) :when (and review (not (#{:required :allowed} review)))]
       [:case id :unknown-review review])
     (for [{:keys [id expect review]} (units s) :when (and (= :needs-person expect) (not= :required review))]
       [:case id :needs-person-not-required])
     (for [{:keys [id review] :as u} (units s)
           :let [line-facts (facts/enrich s u)]
           {rule :id :as r} checks
           :when (and (>= (blocks/rule-holds r line-facts {}) 0.5M) (not= :required review))]
       [:case id :check-holds-but-not-required rule]))))

(defn scenario
  "Reads the scenario `name` from the classpath (classify/eval/<name>.edn) and checks it."
  [name]
  (let [{:keys [codes rules cases categories cards] :as s}
        (edn/read-string (slurp (io/resource (str "classify/eval/" name ".edn"))))
        rule-ids (set (map :id rules))
        problems (concat
                  (for [{:keys [id code]} rules :when (not (contains? codes code))] [:rule id :unknown-code code])
                  (for [{:keys [id over]} rules o over :when (not (rule-ids o))] [:rule id :overrules-unknown o])
                  (for [{:keys [id mcc card]} cases
                        :when (or (and mcc (not (contains? categories mcc))) (not (contains? cards card)))]
                    [:case id :unknown-category-or-card])
                  (for [[id n] (frequencies (map :id cases)) :when (> n 1)] [:case id :duplicate])
                  (key-problems s))]
    (when (seq problems)
      (throw (ex-info (str "The scenario " name " does not hold together: " (vec problems))
                      {:type :ownpurse/bad-scenario :problems (vec problems)})))
    s))

(defn- sha256 [^String s]
  (let [d (.digest (MessageDigest/getInstance "SHA-256") (.getBytes s "UTF-8"))]
    (apply str (map #(format "%02x" %) d))))

(defn scenario-hash
  "The SHA-256 of the scenario `name`'s file as it is on the classpath now."
  [name]
  (sha256 (slurp (io/resource (str "classify/eval/" name ".edn")))))

(defn- source
  "The code the run ran: the commit checked out, and whether the working tree differed from it."
  []
  (let [git #(let [{:keys [exit out]} (apply shell/sh "git" %&)] (when (zero? exit) (str/trim out)))]
    {:commit (git "rev-parse" "HEAD")
     :dirty (boolean (seq (git "status" "--porcelain")))}))

(defn- lift
  "A run whose requests are the same on every result keeps them once, under :sent: each request kind's questions,
  and the rules sent in the :rules request's state. Each result keeps its own states."
  [{:keys [results] :as run}]
  (let [same (fn [f] (let [vs (distinct (keep f results))] (when (= 1 (count vs)) (first vs))))
        questions (into {} (keep (fn [k] (some->> (same #(get-in % [:requests k :questions])) (vector k))))
                        [:rules :plain :enrich])
        rules (same #(get-in % [:requests :rules :state :rules]))]
    (assoc run
           :sent (cond-> {:questions questions} rules (assoc :rules rules))
           :results (mapv (fn [r]
                            (cond-> (reduce (fn [r k] (cond-> r (get-in r [:requests k]) (update-in [:requests k] dissoc :questions)))
                                            r (keys questions))
                              rules (update-in [:requests :rules :state] dissoc :rules)))
                          results))))

(defn sent
  "The requests a result of `run` sent, exactly: {kind {:state :questions}}, put back together from what the run
  keeps once. Nil for a run saved before runs kept their requests."
  [{{:keys [questions rules]} :sent} {:keys [requests]}]
  (when requests
    (into {} (for [[k {:keys [state] :as req}] requests]
               [k (cond-> (assoc req :state state)
                    (questions k) (assoc :questions (questions k))
                    (and (= :rules k) rules) (assoc-in [:state :rules] rules))]))))

(defn- usage [results]
  (let [us (for [r results resp (vals (:responses r))] (:usage resp))]
    {:requests (count us)
     :input-tokens (reduce + (keep :input_tokens us))
     :output-tokens (reduce + (keep :output_tokens us))}))

(defn run
  "Classifies every unit of `scenario` (each case, or each item of one) with `ask`, four at a time. Returns the run:
  each case's facts, requests, raw responses and codings, with the model, the tokens used, the code that ran and the
  scenario's hash. opts: :done, results already classified (a resumed run: their units are not asked again);
  :on-result, called with each new result as it returns (to save progress); :at, the run's time (a resumed run keeps
  its own)."
  ([scenario ask] (run scenario ask {}))
  ([scenario ask {:keys [done on-result at]}]
   (let [done-ids (set (map :id done))
         fresh (->> (units scenario)
                    (remove #(done-ids (:id %)))
                    (partition-all 4)
                    (mapcat (fn [batch]
                              (mapv deref (mapv #(future (doto (core/classify scenario ask %)
                                                           (as-> r (when on-result (on-result r)))))
                                                batch))))
                    vec)
         order (into {} (map-indexed (fn [i u] [(:id u) i])) (units scenario))
         results (vec (sort-by (comp order :id) (concat done fresh)))]
     (lift {:scenario (:scenario scenario)
            :at (or at (str (Instant/now)))
            :model (some #(get-in % [:responses :rules :model]) results)
            :source (source)
            :scenario-hash (scenario-hash (:scenario scenario))
            :usage (usage results)
            :results results}))))

(defn outcome
  "How one coding fares at `threshold`. Coded automatically: :wrong when its bucket is not an acceptable one, else
  :unsafe when a person had to look, else :right. Sent to a person: :stopped when a person had to or may look,
  :review when it was needless."
  [{:keys [expect ok review]} {:keys [code confidence]} threshold]
  (let [acceptable (conj (set ok) expect)
        auto? (and (not= :needs-person code) (>= confidence threshold))]
    (cond
      (and auto? (not (acceptable code))) :wrong
      (and auto? (= :required review)) :unsafe
      auto? :right
      review :stopped
      :else :review)))

(defn- check-complete
  "Throws unless `results` hold every unit of `scenario` once and nothing else."
  [scenario results]
  (let [want (set (map :id (units scenario)))
        ids (map :id results)
        missing (sort (remove (set ids) want))
        extra (sort (remove want (distinct ids)))
        duplicate (sort (for [[id n] (frequencies ids) :when (> n 1)] id))]
    (when (or (seq missing) (seq extra) (seq duplicate))
      (throw (ex-info (str "The run does not cover the scenario's units once each: " (count missing) " missing, "
                           (count extra) " not in the scenario, " (count duplicate) " more than once")
                      {:type :ownpurse/incomplete-run :missing (vec missing) :extra (vec extra)
                       :duplicate (vec duplicate)})))))

(defn score
  "Each strategy's results on `run` at every threshold: how many lines were coded automatically, how many of those
  wrongly or unsafely (the silent errors that matter most), and how many went to a person. Also each strategy's plain
  accuracy (its answer, ignoring confidence; sending to a person is acceptable where a person may look) and how often the Choice kept its answer with its options reversed.
  Throws unless the run holds every unit of the scenario once. :stale-rules is true when the rules or questions the
  run sent differ from the scenario's now (its answers are to other questions); :changed-key when only the file
  differs."
  [scenario {:keys [results] :as run}]
  (check-complete scenario results)
  (let [cases (into {} (map (juxt :id identity)) (units scenario))
        silent (fn [s o] (into (sorted-map)
                             (for [t thresholds]
                               [t (vec (for [{:keys [id decisions]} results
                                             :when (= o (outcome (cases id) (get decisions s) t))]
                                         [id (get-in decisions [s :code])]))])))
        kept (some->> (first results) (sent run))
        asked-now (when kept (core/requests scenario (facts/enrich scenario (cases (:id (first results))))))
        stale? (boolean (and kept (or (not= (get-in kept [:rules :state :rules]) (get-in asked-now [:rules :state :rules]))
                                      (not= (get-in kept [:rules :questions]) (get-in asked-now [:rules :questions]))
                                      (not= (get-in kept [:plain :questions]) (get-in asked-now [:plain :questions])))))]
    {:cases (count results)
     :stale-rules stale?
     :changed-key (boolean (and (:scenario-hash run) (not stale?)
                                (not= (:scenario-hash run) (scenario-hash (:scenario scenario)))))
     :sent-kept (some? kept)
     :choice-stable (count (filter #(= (get-in % [:decisions :choice :code]) (get-in % [:decisions :reversed :code]))
                                   results))
     :strategies
     (into {}
           (for [s core/strategies]
             [s {:accurate (count (filter (fn [{:keys [id decisions]}]
                                            (let [{:keys [expect ok review]} (cases id)]
                                              ((cond-> (conj (set ok) expect) review (conj :needs-person))
                                               (get-in decisions [s :code]))))
                                          results))
                 :by-threshold (into (sorted-map)
                                     (for [t thresholds]
                                       [t (merge (zipmap outcomes (repeat 0))
                                                 (frequencies (map #(outcome (cases (:id %)) (get-in % [:decisions s]) t)
                                                                   results)))]))
                 :wrong (silent s :wrong)
                 :unsafe (silent s :unsafe)}]))}))

(defn report
  "The score as a table to read."
  [{:keys [cases choice-stable strategies stale-rules changed-key]}]
  (println (format "%d units. The Choice kept its answer with its options reversed on %d." cases choice-stable))
  (when stale-rules
    (println "The run's rules differ from the scenario's now: its answers are to other questions."))
  (when changed-key
    (println "Scored against the scenario's answer key as it is now, which changed since the run."))
  (pprint/print-table
   (for [s core/strategies
         :let [{:keys [accurate by-threshold]} (strategies s)]
         t thresholds
         :let [{:keys [right wrong unsafe stopped review]} (by-threshold t)]]
     {:strategy (name s) :accurate accurate :threshold (str t)
      :auto-right right :auto-wrong wrong :auto-unsafe unsafe :stopped-rightly stopped :needless-review review})))

(defn load-run
  "Reads a saved run."
  [file]
  (edn/read-string (slurp file)))

(defn- run-file [dir {:keys [scenario at]} suffix]
  (io/file dir (str scenario "-" (.replace ^String at ":" "") suffix)))

(defn save!
  "Writes `run` to `dir` as <scenario>-<time>.edn and returns the file, once it reads back as the same run."
  [dir run]
  (let [f (run-file dir run ".edn")]
    (io/make-parents f)
    (spit f (with-out-str (pprint/pprint run)))
    (when-not (= run (load-run f))
      (throw (ex-info (str "The run written to " f " does not read back as the same run")
                      {:type :ownpurse/unreadable-run :file (str f)})))
    f))

(defn progress
  "Where a run in `dir` keeps its results while it runs: <scenario>-<time>.partial.edn, one result per line after a
  first line saying which run it is. Returns {:file :add!}; add! appends a result, safely from several threads."
  [dir head]
  (let [f (run-file dir head ".partial.edn")
        lock (Object.)]
    (io/make-parents f)
    (when-not (.exists f) (spit f (str (pr-str head) "\n")))
    {:file f
     :add! (fn [result] (locking lock (spit f (str (pr-str result) "\n") :append true)))}))

(defn read-progress
  "The head and results of a partial run file."
  [file]
  (let [[head & results] (map edn/read-string (remove str/blank? (str/split-lines (slurp file))))]
    {:head head :results (vec results)}))

(defn -main
  "Runs the eval over the scenario named first (harbour by default) with Jev, saves the run beside the component's
  earlier runs and prints the score. Each result is kept as it returns in a .partial.edn file, removed once the run is
  saved; `--resume FILE` carries on from one, asking only the units it does not hold."
  [& args]
  (let [[name & more] args
        resume (second (drop-while #(not= "--resume" %) more))
        s (scenario (or name "harbour"))
        jev-config (:jev (config/read-config :dev))
        dir "components/classify/runs"
        {:keys [head results]} (if resume (read-progress resume) {:head {:scenario (:scenario s) :at (str (Instant/now))}})
        {:keys [file add!]} (progress dir head)
        r (run s #(jev/ask jev-config %1 %2) {:done results :on-result add! :at (:at head)})
        f (save! dir r)]
    (io/delete-file file)
    (println "Saved" (str f) "-" (:usage r))
    (report (score s r))
    (shutdown-agents)))
