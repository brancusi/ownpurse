(ns ownpurse.money.interface-test
  (:require
   [clojure.test :refer [deftest is testing]]
   [clojure.test.check.clojure-test :refer [defspec]]
   [clojure.test.check.generators :as gen]
   [clojure.test.check.properties :as prop]
   [ownpurse.money.interface :as money]
   [ownpurse.money.interface.gen :as mgen]))

(defn- error-type [f]
  (try (f) nil (catch clojure.lang.ExceptionInfo e (:type (ex-data e)))))

(deftest amount-test
  (testing "exact inputs become BigDecimals and keep their scale"
    (is (= 12.50M (money/amount "12.50")))
    (is (= 2 (.scale ^BigDecimal (money/amount "12.50"))))
    (is (= -3M (money/amount -3)))
    (is (= 7M (money/amount 7N)))
    (let [a 1.10M] (is (identical? a (money/amount a)))))
  (testing "a float is refused, with what to send instead"
    (is (= :ownpurse/float-money (error-type #(money/amount 1.1))))
    (is (= :ownpurse/float-money (error-type #(money/amount #_{:clj-kondo/ignore [:discouraged-var]} (float 1.1)))))
    (is (re-find #"\"12.50\"" (try (money/amount 0.1) (catch Exception e (ex-message e))))))
  (testing "anything else is not an amount"
    (doseq [x ["12,50" "1e3" "" nil :usd "1.2.3"]]
      (is (= :ownpurse/not-an-amount (error-type #(money/amount x))) (pr-str x)))))

(defspec a-string-round-trips 200
  (prop/for-all [a mgen/amount]
    (let [back (money/amount (.toPlainString ^BigDecimal a))]
      (and (= a back) (= (.scale ^BigDecimal a) (.scale ^BigDecimal back))))))

(defspec balanced-amounts-balance 200
  (prop/for-all [ms mgen/balanced]
    (and (money/balanced? ms)
         (every? zero? (vals (money/totals ms)))
         (empty? (money/imbalance ms)))))

(defspec order-does-not-matter 200
  (prop/for-all [ms (gen/vector mgen/money 0 10)]
    (= (money/totals ms) (money/totals (shuffle ms)) (money/totals (reverse ms)))))

(defspec any-change-unbalances 200
  (prop/for-all [ms mgen/balanced
                 i gen/nat
                 delta (gen/such-that (complement zero?) mgen/amount)]
    (let [i (mod i (count ms))
          changed (update-in ms [i :amount] + delta)]
      (and (not (money/balanced? changed))
           (= {(get-in ms [i :commodity]) delta} (money/imbalance changed))))))

(deftest float-paths-test
  (is (= [] (money/float-paths {:a 1.10M :b [1 "x" #{2M}] :c nil})))
  (is (= [[:a] [:b 1] [:c :d 0]]
         (money/float-paths {:a 1.5 :b [1M 2.0] :c {:d '(3.0)}}))))

(defspec floats-are-found-wherever-they-are 200
  (prop/for-all [path (gen/vector (gen/one-of [gen/keyword (gen/return 0)]) 1 4)
                 x gen/double]
    (let [data (reduce (fn [inner k] (if (keyword? k) {k inner} [inner])) x (reverse path))]
      (= [path] (money/float-paths data)))))
