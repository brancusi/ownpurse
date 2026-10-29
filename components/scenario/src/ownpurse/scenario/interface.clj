(ns ownpurse.scenario.interface
  "Scenarios: the books as layers over facts. A scenario is a named stack of layers, each one kind of interpretation
  as data (journals of its own, the close register's journals), standing on another scenario if it says so (its
  :base). Evaluating a scenario over a book gives what its layers add to each account; the reports take it and show
  the book as the scenario would make it. Nothing here changes a fact or writes to a system of record: a scenario is
  dropped, duplicated or changed freely, and every version is kept. Knows nothing about any one source."
  (:require
   [ownpurse.scenario.core :as core]
   [ownpurse.scenario.lines :as lines]
   [ownpurse.scenario.run :as run]
   [ownpurse.scenario.sandbox :as sandbox]))

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

(defn create!
  "As put!, but refused (:ownpurse/scenario-conflict) when there is a scenario of that ref already: a new scenario
  never replaces one."
  [node scenario]
  (core/create! node scenario))

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

(defn classifier-scenario
  "Scenario `ref`'s world, coding and lines layers (those that are on, its bases' first), composed into the
  classifier's scenario: what ownpurse.classify reads, with the lines from the record as its :cases."
  [node ref]
  (core/classifier-scenario node ref))

(defn composed
  "Scenario `ref`'s world and coding as its layers that are on compose them (its bases' first), without reading its
  lines: the rules that stand, with the lines layers' selections as :selects and their given lines as :cases."
  [node ref]
  (merge {:threshold run/default-threshold :strategy run/default-strategy} (core/composed node ref)))

(defn rule-ledger
  "Every rule scenario `ref`'s coding layers name, in the order they stand, those turned off in their place: where
  each is defined, where it was turned off, and which layers changed it."
  [node ref]
  (core/rule-ledger node ref))

(defn plan
  "What coding scenario `ref` would take, before asking anything: {:lines :units :at-most-requests}. Over its own
  lines, or over set `over` (as set-over gives it)."
  ([node ref] (plan node ref nil))
  ([node ref over] (run/plan (:s (core/over node ref over)))))

(defn estimate
  "What coding scenario `ref` would take now, asking nothing: the requests to ask of `model` and those the record
  answers, the input tokens and cost (at Jev's published price) and the seconds it would take (from its timed runs).
  opts: :limit, as for code!. {:units :requests :asked :kept :input-tokens :cost-usd :seconds :timed
  :price-per-million-input-tokens}. opts :over, a set to run over (as set-over gives it) instead of its own lines."
  [node ref model {:keys [over] :as opts}]
  (let [{:keys [scope s]} (core/over node ref over)]
    (run/estimate node s model (assoc opts :runs (run/runs node scope)))))

(defn code!
  "Codes scenario `ref`'s lines with `ask` (state, questions → a System One response; ownpurse.classify.interface/
  jev-ask makes one), keeping every answer in the record by its exact request, so coding again after an edit asks
  only what changed. opts: :model (part of what an answer is kept by), :limit (code at most that many units, for a
  trial; a run without a limit is the scenario's whole coding), :on-unit (called with each unit's result as it
  returns, for progress). Each unit's coding stands when its strategy's
  (the coding layer's :strategy, default :checked) is a bucket at the layer's :threshold (default 0.85) or above;
  else a person looks. opts :over, a set to run over (as set-over gives it) instead of its own lines: that run is
  kept apart, under <ref>@<set>. Returns the run: {:scenario :at :units :coded :asked :kept :model}."
  [node ref ask {:keys [over] :as opts}]
  (let [{:keys [scope s] set-id :over} (core/over node ref over)]
    (run/code! node scope s ask (cond-> (dissoc opts :over) set-id (assoc :of ref :over set-id)))))

(defn codings
  "Scenario `ref`'s coding of each unit, as its last run left it, by date."
  [node ref]
  (run/codings node ref))

(defn coding
  "One unit's coding by scenario `ref`, with its facts and every strategy's decision, or nil."
  [node ref unit]
  (run/coding node ref unit))

(defn coding-runs
  "Scenario `ref`'s coding runs over its own lines (or over set `over`), newest first."
  ([node ref] (run/runs node ref))
  ([node ref over] (run/runs node (core/scope node ref over))))

(defn set-over
  "Set `id` of the record's lines (a scenario's lines layer, <scenario>~<layer>, or a saved set by name) as a run
  reads it, or nil."
  [node id]
  (core/set-over node id))

(defn line-set-put!
  "Saves a set of lines by name: {:name :title :note :select}; its key is kept."
  [node s]
  (lines/put! node s))

(defn line-set-confirm!
  "Keys one line in saved set `name` ({:unit :expect ...}, or {:unit :remove true})."
  [node name entry]
  (lines/confirm! node name entry))

(defn compare-codings
  "The units scenarios `a` and `b` code differently."
  [node a b]
  (run/compare-codings node a b))

(defn sandbox-scenarios
  "The record's scenarios that code lines, in the sandbox's shape (ownpurse.sandbox/scenarios): each coded one has
  one run, \"latest\"."
  [node]
  (sandbox/scenarios node))

(defn line-sets
  "The sets of the record's own lines the sandbox picks from, each with the scenarios that read it."
  [node]
  (sandbox/line-sets node))

(defn line-set
  "Set `id` of the record's own lines, with a preview of them."
  [node id]
  (sandbox/line-set node id))

(defn coding-scenario?
  "Whether scenario `ref` is in the record and codes lines."
  [node ref]
  (boolean (and (core/scenario node ref) (sandbox/coding-scenario? node ref))))

(defn sandbox-world
  "Scenario `ref`'s world, in the sandbox's shape."
  [node ref]
  (sandbox/world node ref))

(defn sandbox-run
  "Scenario `ref`'s coding as the sandbox's run: one row a unit. Over its own lines, or over set `over` (as set-over
  gives it)."
  ([node ref] (sandbox-run node ref nil))
  ([node ref over] (sandbox/run-lines node ref over)))

(defn sandbox-line
  "One unit of scenario `ref`'s coding all the way through, as the sandbox's line; over set `over` when given."
  ([node ref unit] (sandbox-line node ref nil unit))
  ([node ref over unit] (sandbox/line node ref over unit)))

(defn answer-key
  "Scenario `ref`'s answer key, its bases' first: {unit {:expect :ok :review :why :at :in}}, :in the scenario whose
  key layer says it."
  [node ref]
  (core/answer-key node ref))

(defn confirm!
  "A person's coding of one unit, kept in scenario `ref`'s own key layer: {:unit :expect :ok :review :why}, or
  {:unit :remove true} to take it out of this scenario's key. Returns the scenario."
  [node ref entry]
  (core/confirm! node ref entry))

(defn sandbox-trace
  "One unit of scenario `ref` through the pipeline step by step: each step's function called again on the record's
  data, with what went in, what came out, and every query it made. Nothing is asked of Jev. Over set `over` when
  given."
  ([node ref unit] (sandbox-trace node ref nil unit))
  ([node ref over unit] (sandbox/trace node ref over unit)))
