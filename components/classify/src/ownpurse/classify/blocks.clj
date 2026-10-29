(ns ownpurse.classify.blocks
  "Rule building blocks, as in a bookkeeping tool's bank rules: exact matches on the line's facts, and one fuzzy
  block, a yes/no question a model answers. Every block gives the probability that it holds: 1 or 0 for an exact one,
  the model's answer for a fuzzy one. A rule holds as much as its weakest block."
  (:require
   [clojure.string :as str]))

(defn- yes [b] (if b 1M 0M))

(defn- text [facts field] (str/lower-case (str (get facts field))))

(defn holds
  "The probability that `block` holds for `facts`, with `fuzzy` mapping each fuzzy question to its answer."
  [[op & args :as block] facts fuzzy]
  (case op
    :is (let [[field v] args] (yes (= (str/lower-case v) (text facts field))))
    :present (yes (not (str/blank? (str (get facts (first args))))))
    :contains (let [[field v] args] (yes (str/includes? (text facts field) (str/lower-case v))))
    :amount (let [[cmp a b] args
                  amount (abs (:amount facts))]
              (yes (case cmp
                     :over (> amount a)
                     :under (< amount a)
                     :between (<= a amount b))))
    :category (yes (contains? (second args) (:mcc facts)))
    :card (yes (contains? (second args) (:card facts)))
    :channel (yes (contains? (second args) (:channel facts)))
    :trip (let [trip (get-in facts [:whereabouts :trip])
                kind (first args)]
            (yes (if (= :any kind) (some? trip) (= kind trip))))
    :at-home (yes (nil? (get-in facts [:whereabouts :trip])))
    :on-trip-for (yes (= (first args) (get-in facts [:whereabouts :project])))
    :per-diem-trip (yes (get-in facts [:whereabouts :per-diem]))
    :during-project (yes (seq (:projects facts)))
    :traveller (yes (:traveller facts))
    :country-mismatch (yes (:country-mismatch facts))
    :not (- 1M (holds (first args) facts fuzzy))
    :any (reduce max 0M (map #(holds % facts fuzzy) args))
    :jev (or (get fuzzy (first args))
             (throw (ex-info (str "No answer for the fuzzy block: " (first args))
                             {:type :ownpurse/missing-judgment :block block})))
    (throw (ex-info (str "Unknown rule block " op) {:type :ownpurse/unknown-block :block block}))))

(defn rule-holds
  "The probability that every block of `rule` holds: the least of them."
  [{:keys [when]} facts fuzzy]
  (reduce min 1M (map #(holds % facts fuzzy) when)))

(defn- fuzzy? [[op & args]]
  (case op :jev true (:not :any) (boolean (some fuzzy? args)) false))

(defn- could-hold
  "At most how much `block` can hold before any judgment: an exact block's own value, but 1 for a fuzzy one (not yet
  asked) and for a category block on a line whose category is unknown (no merchant code, and Jev could not tell)."
  [[op & args :as block] facts]
  (cond
    (fuzzy? block) 1M
    (and (= :category op) (nil? (:mcc facts))) 1M
    (= :not op) (- 1M (holds (first args) facts {}))
    (= :any op) (reduce max 0M (map #(could-hold % facts) args))
    :else (holds block facts {})))

(defn possible?
  "Whether `rule` could apply to `facts`: no exact block of it rules it out. A rule that cannot is not sent to Jev."
  [{:keys [when]} facts]
  (pos? (reduce min 1M (map #(could-hold % facts) when))))

(defn fuzzy-questions
  "Every fuzzy block's question in `rules`, once each, in order."
  [rules]
  (letfn [(walk [[op & args]]
            (case op
              :jev [(first args)]
              (:not :any) (mapcat walk args)
              nil))]
    (vec (distinct (mapcat #(mapcat walk (:when %)) rules)))))
