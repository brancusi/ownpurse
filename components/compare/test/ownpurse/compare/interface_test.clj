(ns ownpurse.compare.interface-test
  (:require
   [clojure.test :refer [deftest is testing]]
   [ownpurse.compare.interface :as cmp]
   [ownpurse.entity.interface :as entity]
   [ownpurse.store.interface :as store])
  (:import
   (java.time LocalDate)))

(def ^:private xero-org "xero:t1:organisation:o1")
(def ^:private xero-1010 "xero:t1:account:a1010")
(def ^:private operating "mercury:m1:account:a1")

(defn- d [s] (LocalDate/parse s))

(defn- line [account n date amount & {:as more}]
  (merge {:xt/id (str "line:" account ":" n) :account account :source "test" :source-table "test.lines"
          :source-row (str account ":" n) :org "o" :date (d date) :amount amount :status "posted"}
         more))

(defn- with-node [f]
  (let [node (store/start {:mode :in-memory})]
    (try (f node) (finally (store/stop node)))))

(defn- error [f]
  (try (f) nil (catch clojure.lang.ExceptionInfo e (ex-data e))))

(deftest match-test
  (testing "the same amount a few days apart is the same money; further apart it is not"
    (let [{:keys [matched evidence-only book-only]}
          (cmp/match [(line "e" 1 "2026-03-09" -3215.47M) (line "e" 2 "2026-03-20" -80.00M)]
            [(line "b" 1 "2026-03-07" -3215.47M) (line "b" 2 "2026-03-11" -80.00M)]
            5)]
      (is (= [["line:e:1" "line:b:1"]] (map (juxt (comp :xt/id :evidence) (comp :xt/id :book)) matched)))
      (is (= ["line:e:2"] (map :xt/id evidence-only)))
      (is (= ["line:b:2"] (map :xt/id book-only)))))
  (testing "two identical lines on one day match two, not one twice"
    (let [{:keys [matched book-only]}
          (cmp/match [(line "e" 1 "2026-03-06" -4.75M) (line "e" 2 "2026-03-06" -4.75M)]
            [(line "b" 1 "2026-03-06" -4.75M) (line "b" 2 "2026-03-06" -4.75M)]
            5)]
      (is (= 2 (count matched)))
      (is (= 2 (count (set (map (comp :xt/id :book) matched)))))
      (is (empty? book-only))))
  (testing "the closest date wins, and 1.10 is 1.1"
    (let [{:keys [matched]} (cmp/match [(line "e" 1 "2026-03-10" 1.10M)]
                              [(line "b" 1 "2026-03-06" 1.1M) (line "b" 2 "2026-03-11" 1.100M)]
                              5)]
      (is (= "line:b:2" (:xt/id (:book (first matched))))))))

(deftest checks-test
  (testing "a check pairs with the book line of its number and amount, weeks apart, and not with a same-amount
            line from the month before"
    (let [{:keys [matched evidence-only book-only]}
          (cmp/match [(line "e" 1 "2026-03-24" -850.00M :check-number "1043")]
            [(line "b" 1 "2026-02-27" -850.00M :check-number "1031")
             (line "b" 2 "2026-03-02" -850.00M :check-number "1043")
             (line "b" 3 "2026-03-22" -850.00M)]
            5)]
      (is (= [["line:e:1" "line:b:2"]] (map (juxt (comp :xt/id :evidence) (comp :xt/id :book)) matched)))
      (is (empty? evidence-only))
      (is (= ["line:b:1" "line:b:3"] (map :xt/id book-only)))))
  (testing "the same check number with another amount is no match: both sides show it"
    (let [{:keys [matched]} (cmp/match [(line "e" 1 "2026-03-24" -850.00M :check-number "1043")]
                              [(line "b" 1 "2026-03-02" -805.00M :check-number "1043")]
                              5)]
      (is (empty? matched)))))

(deftest compare-test
  (with-node
    (fn [node]
      (store/put! node {:source-accounts
                        [{:xt/id xero-org :source "xero" :source-table "xero.organisations" :org "t1" :kind "org"
                          :name "Tallowmere"}
                         {:xt/id xero-1010 :source "xero" :source-table "xero.accounts" :org "t1" :kind "bank"
                          :name "Operating account" :code "1010"}
                         {:xt/id operating :source "mercury" :source-table "mercury.accounts" :org "m1"
                          :kind "bank" :name "Operating (Mercury Checking ••2210)"}]
                        :bank-lines
                        [(line operating 1 "2026-03-02" 1846.13M :counterparty "Ostley Joinery")
                         (line operating 2 "2026-03-06" -4.75M)
                         (line operating 3 "2026-03-06" -4.75M)
                         (line operating 4 "2026-03-09" -3215.47M)
                         (line operating 5 "2026-03-20" -999.00M :counterparty "Ridgeway Freight")
                         (line operating 6 "2026-03-21" -50.00M :status "pending")
                         (line operating 7 "2026-04-01" -10.00M)
                         (line xero-1010 1 "2026-03-02" 1846.13M)
                         (line xero-1010 2 "2026-03-06" -4.75M)
                         (line xero-1010 3 "2026-03-06" -4.75M)
                         (line xero-1010 4 "2026-03-07" -3215.47M)
                         (line xero-1010 5 "2026-03-18" -2400.00M :description "BO-7790")
                         (line xero-1010 6 "2026-03-31" -10.00M)]}
                  {:source "test" :why "fixtures"})
      (testing "an account not linked as evidence for one has nothing to compare"
        (is (= [] (cmp/compare-accounts node {})))
        (is (= :ownpurse/not-found (:type (error #(cmp/compare-accounts node {:account operating}))))))
      (entity/put-entity! node {:id "tallowmere" :name "Tallowmere" :kind "business"})
      (entity/link! node {:entity "tallowmere" :account xero-org :role "system-of-record"})
      (entity/link! node {:entity "tallowmere" :account operating :role "evidence" :evidence-for xero-1010})
      (testing "March: what each side has that the other has not, pending apart, and each side's total"
        (let [[c :as all] (cmp/compare-accounts node {:from (d "2026-03-01") :to (d "2026-03-31")})]
          (is (= 1 (count all)))
          (is (= {:entity "tallowmere" :account operating :account-name "Operating (Mercury Checking ••2210)"
                  :evidence-for xero-1010 :evidence-for-name "Operating account" :days 5 :from "2026-03-01"
                  :to "2026-03-31" :matched 4}
                 (select-keys c [:entity :account :account-name :evidence-for :evidence-for-name :days :from :to
                                 :matched])))
          (is (= [{:id (str "line:" operating ":5") :date "2026-03-20" :amount "-999.00" :status "posted"
                   :source-row (str operating ":5") :counterparty "Ridgeway Freight"}]
                 (:missing-from-book c)))
          (is (= [["2026-03-18" "-2400.00" "BO-7790"]]
                 (map (juxt :date :amount :description) (:missing-from-evidence c))))
          (is (= [["2026-03-21" "-50.00" "pending"]] (map (juxt :date :amount :status) (:pending c))))
          (is (= {:evidence "-2377.84" :book "-3788.84"} (:totals c)))))
      (testing "a line at the range's edge finds its match just outside it, so it is not missing"
        (is (not-any? #(= "2026-03-31" (:date %))
                      (:missing-from-evidence (first (cmp/compare-accounts node {:account operating
                                                                                 :to (d "2026-03-31")})))))))))

(deftest manual-matches-test
  (with-node
    (fn [node]
      (store/put! node {:source-accounts
                        [{:xt/id xero-org :source "xero" :source-table "xero.organisations" :org "t1" :kind "org"
                          :name "Tallowmere"}
                         {:xt/id xero-1010 :source "xero" :source-table "xero.accounts" :org "t1" :kind "bank"
                          :name "Operating account"}
                         {:xt/id operating :source "mercury" :source-table "mercury.accounts" :org "m1"
                          :kind "bank" :name "Operating"}]
                        :bank-lines
                        [(line operating 1 "2026-03-05" -460.00M :check-number "1042")
                         (line operating 2 "2026-04-17" -460.00M :check-number "1043")
                         (line operating 3 "2026-04-20" -12.00M)
                         (line xero-1010 1 "2026-02-14" -460.00M :counterparty "Keel Street Storage")
                         (line xero-1010 2 "2026-03-17" -460.00M :counterparty "Keel Street Storage")]}
                  {:source "test" :why "fixtures"})
      (entity/put-entity! node {:id "tallowmere" :name "Tallowmere" :kind "business"})
      (entity/link! node {:entity "tallowmere" :account xero-org :role "system-of-record"})
      (entity/link! node {:entity "tallowmere" :account operating :role "evidence" :evidence-for xero-1010})
      (let [row #(str %1 ":" %2)
            spring {:account operating :from (d "2026-02-01") :to (d "2026-04-30") :pairs true}]
        (testing "checks the book recorded without their numbers, weeks before they cleared, match nothing"
          (let [[c] (cmp/compare-accounts node spring)]
            (is (= 0 (:matched c)))
            (is (= 3 (count (:missing-from-book c))))
            (is (= 2 (count (:missing-from-evidence c))))))
        (testing "a match is refused when the amounts differ, when a line is unknown, or when the lines are not on
                  an evidence account and the account it is evidence for"
          (is (= :ownpurse/invalid-match
                 (:type (error #(cmp/put-match! node {:evidence (row operating 3) :book (row xero-1010 1)})))))
          (is (= :ownpurse/not-found
                 (:type (error #(cmp/put-match! node {:evidence (row operating 9) :book (row xero-1010 1)})))))
          (is (= :ownpurse/invalid-match
                 (:type (error #(cmp/put-match! node {:evidence (row xero-1010 1) :book (row operating 1)}))))))
        (testing "a person says which book payment each check is; the compare pairs them first and says so"
          (is (= {:evidence (row operating 1) :book (row xero-1010 1) :note "check 1042, February storage"
                  :from "2026-03-05"}
                 (cmp/put-match! node {:evidence (row operating 1) :book (row xero-1010 1)
                                       :note "check 1042, February storage" :from (d "2026-03-05")})))
          (cmp/put-match! node {:evidence (row operating 2) :book (row xero-1010 2)})
          (let [[c] (cmp/compare-accounts node spring)]
            (is (= 2 (:matched c)))
            (is (= {:manual 2 :check 0 :amount 0} (:matched-by c)))
            (is (= [] (:missing-from-evidence c)))
            (is (= ["-12.00"] (map :amount (:missing-from-book c))))
            (is (= [["manual" "2026-03-05" "2026-02-14" "check 1042, February storage"]
                    ["manual" "2026-04-17" "2026-03-17" nil]]
                   (map (juxt :by (comp :date :evidence) (comp :date :book) :note) (:pairs c))))))
        (testing "a line is in one match at a time"
          (is (= :ownpurse/match-conflict
                 (:type (error #(cmp/put-match! node {:evidence (row operating 1) :book (row xero-1010 2)}))))))
        (testing "ending a match keeps its history, and the lines are missing again"
          (is (= :ownpurse/invalid-match
                 (:type (error #(cmp/end-match! node {:evidence (row operating 1) :on (d "2026-03-04")}))))
              "not before it starts")
          (is (some? (:until (cmp/end-match! node {:evidence (row operating 2)}))))
          (is (= {:manual 1 :check 0 :amount 0} (:matched-by (first (cmp/compare-accounts node spring)))))
          (is (= 2 (count (store/q node "SELECT _id FROM line_matches FOR ALL VALID_TIME"))))
          (is (= :ownpurse/not-found (:type (error #(cmp/end-match! node {:evidence (row operating 2)}))))))))))
