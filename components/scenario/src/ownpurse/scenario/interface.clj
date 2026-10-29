(ns ownpurse.scenario.interface
  "Scenarios: the books as layers over facts. A scenario is a named stack of layers, each one kind of interpretation
  as data (journals of its own, the close register's journals), standing on another scenario if it says so (its
  :base). Evaluating a scenario over a book gives what its layers add to each account; the reports take it and show
  the book as the scenario would make it. Nothing here changes a fact or writes to a system of record: a scenario is
  dropped, duplicated or changed freely, and every version is kept. Knows nothing about any one source."
  (:require
   [ownpurse.scenario.core :as core]))

(def kinds
  "The kinds of layer, each with what it says."
  core/kinds)

(def built-in
  "The scenarios every record has and none can change: \"approved\" and \"proposed\", the close register's journals
  as approved and as proposed."
  core/built-in)

(defn put!
  "Adds a scenario or changes it: {:ref :name (required) :note :base (the scenario it stands on) :layers [{:id :kind
  :enabled (default true) :note, and what the kind reads: :journals [{:ref :book :date :narration :postings [{:account
  :amount :commodity :description}]}] for \"journals\", :basis \"approved\" or \"proposed\" for \"close\"}]}. Each
  journal must balance in every commodity; a base must exist and not stand on this scenario. Returns the scenario."
  [node scenario]
  (core/put! node scenario))

(defn put-layer!
  "Adds one layer to scenario `ref`, or replaces its layer of the same :id (to change it, or turn it on or off).
  Returns the scenario."
  [node ref layer]
  (core/put-layer! node ref layer))

(defn scenario
  "The scenario `ref` as it stands, or nil."
  [node ref]
  (core/scenario node ref))

(defn scenarios
  "Every scenario, the built-in ones first, then by ref."
  [node]
  (core/scenarios node))

(defn duplicate!
  "A new scenario `ref` (named `name`, else the original's name and \"(copy)\") with the base and layers of `from`
  as they stand, and :duplicated-from naming it. The two change apart from then on. Returns the new scenario."
  [node from opts]
  (core/duplicate! node from opts))

(defn drop!
  "Ends scenario `ref`: it no longer counts anywhere, and its history stays. Refused while another scenario stands
  on it."
  [node ref]
  (core/drop! node ref))

(defn versions
  "Every version of scenario `ref`, oldest first, each with :version (from 1) and :at (when it was kept)."
  [node ref]
  (core/versions node ref))

(defn evaluate
  "Scenario `ref` over `book` from `from` to `to` (LocalDates, either nil for no bound): {:scenario :name :book :from
  :to :stack (the scenarios evaluated, its bases first) :layers [{:scenario :layer :kind :enabled :balances}] (what
  each layer adds; nothing when it is off) :balances (their sum, {account amount}, zeros left out)}. An account is
  named as the layer names it."
  [node ref book opts]
  (core/evaluate node ref book opts))

(defn balances
  "{account amount}: what scenario `ref` adds to `book` from `from` to `to`, as `evaluate`'s :balances. What the
  trial balance's :adjustments takes."
  [node ref book opts]
  (:balances (core/evaluate node ref book opts)))
