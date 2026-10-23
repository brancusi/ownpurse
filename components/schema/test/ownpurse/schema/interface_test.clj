(ns ownpurse.schema.interface-test
  (:require
   [clojure.test :refer [deftest is testing]]
   [clojure.test.check.clojure-test :refer [defspec]]
   [clojure.test.check.generators :as gen]
   [clojure.test.check.properties :as prop]
   [ownpurse.money.interface.gen :as mgen]
   [ownpurse.schema.interface :as schema]))

(defn- entry [moneys]
  {:entry/id "e-1"
   :entry/book "book-1"
   :entry/date (java.time.LocalDate/of 2026 1 31)
   :entry/postings (mapv (fn [i {:keys [amount commodity]}]
                           {:posting/account (str "account-" i) :posting/amount amount :posting/commodity commodity})
                         (range) moneys)})

(defn- error [f]
  (try (f) nil (catch clojure.lang.ExceptionInfo e (ex-data e))))

(defspec a-balanced-entry-is-valid 200
  (prop/for-all [ms mgen/balanced]
    (schema/valid? :ownpurse/entry (entry ms))))

(defspec an-unbalanced-entry-is-refused 200
  (prop/for-all [ms mgen/balanced
                 delta (gen/such-that (complement zero?) mgen/amount)]
    (let [e (update-in (entry ms) [:entry/postings 0 :posting/amount] + delta)
          {:keys [type errors]} (error #(schema/check! :ownpurse/entry e))]
      (and (= :ownpurse/invalid type)
           (some #(re-find #"do not balance" %) (flatten (vals (if (map? errors) errors {:_ errors}))))))))

(defspec a-float-amount-is-refused-wherever-it-is 200
  (prop/for-all [ms mgen/balanced
                 i gen/nat
                 x (gen/double* {:infinite? false :NaN? false})]
    (let [i (mod i (count ms))
          e (assoc-in (entry ms) [:entry/postings i :posting/amount] x)
          {:keys [type errors]} (error #(schema/coerce! :ownpurse/entry e))]
      (and (= :ownpurse/invalid type)
           (= {:entry/postings (assoc (vec (repeat i nil)) i
                                      {:posting/amount ["must be an exact decimal (a string such as \"12.50\" or a BigDecimal), never a float"]})}
              errors)))))

(deftest coerce-test
  (testing "JSON-shaped data becomes the entry: decimal strings, integers, ISO dates"
    (let [e (schema/coerce! :ownpurse/entry
                            {:entry/id "e-1" :entry/book "b" :entry/date "2026-01-31"
                             :entry/postings [{:posting/account "a" :posting/amount "12.50" :posting/commodity "USD"}
                                              {:posting/account "b" :posting/amount -12 :posting/commodity "USD"}
                                              {:posting/account "c" :posting/amount "-0.50" :posting/commodity "USD"}]})]
      (is (= (java.time.LocalDate/of 2026 1 31) (:entry/date e)))
      (is (= [12.50M -12M -0.50M] (map :posting/amount (:entry/postings e))))))
  (testing "an account's class decodes from a string"
    (is (= :income (:account/class (schema/coerce! :ownpurse/account
                                                   {:account/id "a" :account/book "b" :account/name "Sales"
                                                    :account/class "income"})))))
  (testing "unknown keys are refused, so a typo never passes silently"
    (is (= :ownpurse/invalid (:type (error #(schema/check! :ownpurse/book
                                                           {:book/id "b" :book/name "Books" :book/currency "USD"
                                                            :book/source "s" :book/currancy "EUR"}))))))
  (testing "a bad date says what a date looks like"
    (is (= ["must be a date, YYYY-MM-DD"]
           (:entry/date (:errors (error #(schema/coerce! :ownpurse/entry
                                                         (assoc (entry []) :entry/date "31/01/2026")))))))))
