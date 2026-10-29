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

(defn facts
  "What code works out about a line (or one unit of it): its category (the merchant code's, or `category`, the one an
  enrichment judgment chose), the card's holder, where they were, the projects running, an earlier purchase a
  refund returns."
  [scenario line category]
  (facts/enrich scenario line category))

(defn enriched-category
  "The merchant code an enrichment answer chose, or nil when it chose none or was unsure."
  [scenario answer]
  (core/enriched-category scenario answer))

(defn decide
  "Each strategy's coding of a line, from its facts and the answers to its two requests
  ({:rules-answers :plain-answers})."
  [scenario line-facts answers]
  (core/decide scenario line-facts answers))

(defn requests
  "The two requests a line's facts make to Jev: :rules and :plain, each {:state :questions}."
  [scenario facts]
  (core/requests scenario facts))

(defn enrichment
  "The enrichment request a line needs before its rules can run (its category judged from the merchant's name), or nil."
  [scenario line]
  (core/enrichment scenario line))

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
  "Reads and checks a scenario: `source` is its name on the classpath (classify/eval/<name>.edn), or a file."
  [source]
  (eval/scenario source))

(defn run-eval
  "Classifies every case of `scenario` with `ask` and returns the run, raw answers and the requests as sent included.
  opts: :done (results of a run being resumed), :on-result (called with each result as it returns), :at."
  ([scenario ask] (eval/run scenario ask))
  ([scenario ask opts] (eval/run scenario ask opts)))

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

(def outcomes
  "How a coding can fare, in the order the reports list them."
  eval/outcomes)

(defn outcome
  "How a strategy's coding of a case fares at a threshold: :right or :wrong (coded automatically, by its bucket),
  :unsafe (coded automatically when a person had to look), :stopped (sent to a person, rightly) or :review (sent to
  a person needlessly)."
  [case coding threshold]
  (eval/outcome case coding threshold))

(defn sent
  "The requests a result of `run` sent to Jev, exactly as sent: {kind {:state :questions}}. Nil for a run saved
  before runs kept them."
  [run result]
  (eval/sent run result))

(defn scenario-hash
  "The hash of a scenario's file as it is now (`source` as for `scenario`); a run keeps the one it ran against."
  [source]
  (eval/scenario-hash source))

(defn load-run
  "Reads a saved run."
  [file]
  (eval/load-run file))

