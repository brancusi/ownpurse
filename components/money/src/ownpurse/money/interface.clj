(ns ownpurse.money.interface
  "Exact amounts. An amount is a BigDecimal and travels with its commodity: {:amount 12.50M :commodity \"USD\"}."
  (:require
   [ownpurse.money.core :as core]))

(defn amount
  "Returns `x` as a BigDecimal: a BigDecimal as it is, an integer, or a decimal string such as \"-12.50\". Throws
  :ownpurse/float-money for a float or a double, and :ownpurse/not-an-amount for anything else."
  [x]
  (core/amount x))

(defn amount?
  "True when `x` is a BigDecimal."
  [x]
  (core/amount? x))

(defn totals
  "Sums `amounts` ({:amount :commodity} maps) per commodity: {\"USD\" 0.00M}."
  [amounts]
  (core/totals amounts))

(defn imbalance
  "The commodities whose `amounts` do not sum to zero, with their totals. Empty when they balance."
  [amounts]
  (core/imbalance amounts))

(defn balanced?
  "True when `amounts` sum to zero in every commodity."
  [amounts]
  (core/balanced? amounts))

(defn float-paths
  "The paths to every float or double inside `x` (maps, vectors, sets, lists), as in `get-in`. Empty when there
  are none."
  [x]
  (core/float-paths x))
