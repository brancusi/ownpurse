(ns ownpurse.sandbox.interface
  "A sandbox for building a rule set: bank lines, the world they happened in, the rules, and the runs that classified
  them, as data for the admin to show. Its lines are a classify scenario's for now."
  (:require
   [ownpurse.sandbox.core :as core]))

(defn open
  "A sandbox over the runs kept in folder `:dir`, and the scenarios in folder `:scenarios` (a workspace's own)
  beside ownpurse's."
  [config]
  (core/open config))

(defn scenarios
  "Every scenario, its size, and its runs, newest first."
  [sandbox]
  (core/scenarios sandbox))

(defn world
  "Everything about scenario `name` but its lines: people, cards, trips, projects, buckets, categories and rules."
  [sandbox name]
  (core/world sandbox name))

(defn run-lines
  "Run `id` of scenario `name`: one light row a unit, each strategy's coding, for a list scored at any threshold."
  [sandbox name id]
  (core/run-lines sandbox name id))

(defn line
  "One unit of run `id` all the way through: facts, requests as sent, answers, rules, decisions and commentary."
  [sandbox name id line-id]
  (core/line sandbox name id line-id))
