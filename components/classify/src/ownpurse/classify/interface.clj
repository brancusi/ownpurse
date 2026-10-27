(ns ownpurse.classify.interface
  "Classifying bank lines by context: enrichment computes the facts code can know (category, where the cardholder
  was, which project was running), atomic rules say what a person decided, and Jev judges what is fuzzy. The eval
  compares ways of combining them over a scenario of invented lines."
  (:require
   [ownpurse.classify.core :as core]
   [ownpurse.classify.eval :as eval]
   [ownpurse.classify.facts :as facts]
   [ownpurse.classify.jev :as jev]))

(def strategies
  "The ways of combining rules and judgments the eval compares (see ownpurse.classify.core)."
  core/strategies)

(defn enrich
  "The facts about a bank line in a scenario, computed without any judgment."
  [scenario line]
  (facts/enrich scenario line))

(defn classify
  "Classifies one line with `ask` (state, questions → a System One response): its facts, the raw responses and each
  strategy's coding."
  [scenario ask line]
  (core/classify scenario ask line))

(defn jev-ask
  "An `ask` over the System One API with `jev-config` (the config's :jev: url, model, api-key)."
  [jev-config]
  (fn [state questions] (jev/ask jev-config state questions)))

(defn scenario
  "Reads and checks the eval scenario `name` (classify/eval/<name>.edn on the classpath)."
  [name]
  (eval/scenario name))

(defn run-eval
  "Classifies every case of `scenario` with `ask` and returns the run, raw answers included."
  [scenario ask]
  (eval/run scenario ask))

(defn score
  "Scores a run against its scenario: each strategy's accuracy and, at each confidence threshold, its automatic
  codings right and wrong and its lines sent to a person."
  [scenario run]
  (eval/score scenario run))

(defn report
  "Prints a score as a table."
  [score]
  (eval/report score))

(defn save-run!
  "Writes a run to `dir` and returns the file."
  [dir run]
  (eval/save! dir run))
