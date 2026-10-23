(ns ownpurse.json.interface-test
  (:require
   [clojure.test :refer [deftest is testing]]
   [ownpurse.json.interface :as json]))

(deftest json-test
  (testing "fractions are BigDecimals both ways, never doubles"
    (is (= 12.50M (:amount (json/parse "{\"amount\": 12.50}"))))
    (is (= 2 (.scale ^BigDecimal (:amount (json/parse "{\"amount\": 12.50}")))))
    (is (instance? BigDecimal (:amount (json/parse "{\"amount\": 0.1}"))))
    (is (= "{\"amount\":1000}" (json/write-str {:amount 1E+3M})))))
