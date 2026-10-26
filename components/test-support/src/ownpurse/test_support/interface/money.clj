(ns ownpurse.test-support.interface.money
  "test.check generators for exact amounts, for property tests in any brick."
  (:require
   [clojure.test.check.generators :as gen]))

(def commodity
  "A commodity code from a small set, so that generated amounts share commodities."
  (gen/elements ["USD" "EUR" "GBP" "AUD"]))

(def amount
  "A BigDecimal with a scale from 0 to 4, positive, negative or zero."
  (gen/fmap (fn [[unscaled scale]] (BigDecimal/valueOf (long unscaled) (int scale)))
            (gen/tuple gen/large-integer (gen/choose 0 4))))

(def money
  "An {:amount :commodity} map."
  (gen/let [a amount c commodity] {:amount a :commodity c}))

(def balanced
  "Two or more {:amount :commodity} maps that sum to zero in every commodity: random amounts, then one closing
  amount per commodity."
  (gen/let [ms (gen/not-empty (gen/vector money 1 8))]
    (into ms (for [[c total] (reduce (fn [acc {:keys [amount commodity]}] (update acc commodity (fnil + 0M) amount))
                                     {} ms)]
               {:amount (- total) :commodity c}))))
