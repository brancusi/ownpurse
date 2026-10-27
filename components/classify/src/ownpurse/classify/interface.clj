(ns ownpurse.classify.interface
  "Classifying bank lines by context: enrichment computes the facts code can know (category, where the cardholder
  was, which project was running), atomic rules say what a person decided, and Jev judges what is fuzzy. The eval
  compares ways of combining them over a scenario of invented lines."
  (:require
   [ownpurse.classify.core :as core]
   [ownpurse.classify.blocks :as blocks]
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

(defn units
  "What is coded about a line: the line, or each item its receipt or order lists."
  [line]
  (facts/units line))

(defn attribute
  "Who bears a line coded `code`, who paid it, and whether it was paid on behalf of another entity."
  [scenario facts code]
  (core/attribute scenario facts code))

(defn state
  "What a judgment is shown about a line, from its facts."
  [scenario facts]
  (facts/state scenario facts))

(defn fuzzy-questions
  "Every fuzzy block's question in `rules`, once each, in the order they are asked."
  [rules]
  (blocks/fuzzy-questions rules))

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

(def thresholds
  "The confidence thresholds the eval scores at."
  eval/thresholds)

(defn explain
  "Why each rule did or did not apply to a classified line: its blocks, what each held, and Jev's judgment of it."
  [scenario result]
  (core/explain scenario result))

(defn outcome
  "How a strategy's coding of a case fares at a threshold: :right, :wrong, :stopped (sent to a person, rightly) or
  :review (sent to a person needlessly)."
  [case coding threshold]
  (eval/outcome case coding threshold))

(defn load-run
  "Reads a saved run."
  [file]
  (eval/load-run file))

