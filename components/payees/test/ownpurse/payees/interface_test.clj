(ns ownpurse.payees.interface-test
  (:require
   [clojure.test :refer [deftest is testing]]
   [ownpurse.ledger.interface :as ledger]
   [ownpurse.payees.interface :as payees]
   [ownpurse.store.interface :as store])
  (:import
   (java.time LocalDate)))

(def ^:private book "book:test:tallowmere")
(def ^:private joiner "party:test:joinery")
(def ^:private designer "party:test:design")
(def ^:private cafe "party:test:cafe")

(defn- entry [id date party & postings]
  {:entry/id id :entry/book book :entry/date (LocalDate/parse date)
   :entry/postings (vec (map-indexed (fn [i [account amount]]
                                       (cond-> {:posting/line i :posting/account account :posting/amount amount
                                                :posting/commodity "USD"}
                                         party (assoc :posting/party party)))
                                     postings))})

(def ^:private entries
  [(entry "e1" "2025-02-10" joiner ["acct:6400" 450.00M] ["acct:bank" -450.00M])
   (entry "e2" "2025-05-10" joiner ["acct:6400" 300.00M] ["acct:bank" -300.00M])
   ;; A refund from the joiner comes back into the bank.
   (entry "e3" "2025-06-01" joiner ["acct:bank" 50.00M] ["acct:6400" -50.00M])
   ;; The designer is paid more, but by card.
   (entry "e4" "2025-03-01" designer ["acct:6400" 900.00M] ["acct:card" -900.00M])
   (entry "e5" "2025-04-01" designer ["acct:6400" 100.00M] ["acct:bank" -100.00M])
   ;; A customer's payment in is not a payee.
   (entry "e6" "2025-04-02" cafe ["acct:bank" 700.00M] ["acct:4000" -700.00M])
   ;; Last year's payment is outside the year.
   (entry "e7" "2024-12-30" joiner ["acct:6400" 5000.00M] ["acct:bank" -5000.00M])
   ;; A transfer between the book's own accounts names no party.
   (entry "e8" "2025-07-01" nil ["acct:card" 900.00M] ["acct:bank" -900.00M])])

(deftest payees-test
  (let [node (store/start {:mode :in-memory})]
    (try
      (ledger/put-entries! node book entries {:source "test" :why "fixture"})
      (let [{:keys [threshold] rows :payees}
            (payees/payees node {:book book :from (LocalDate/parse "2025-01-01") :to (LocalDate/parse "2025-12-31")
                                 :accounts {"acct:bank" "bank" "acct:card" "credit-card"}
                                 :parties {joiner {:name "Joinery" :supplier? true :tax-number "00-0000000"}}})
            by-party (into {} (map (juxt :party identity)) rows)]
        (testing "the year's default threshold"
          (is (= 600.00M threshold)))
        (testing "paid by bank, net of refunds, within the year, makes a candidate above the threshold"
          (is (= {"bank" 700.00M} (:paid (by-party joiner))))
          (is (= [true 2 "Joinery" "00-0000000"] ((juxt :candidate? :payments :name :tax-number) (by-party joiner)))))
        (testing "card payments are shown apart and do not count toward it"
          (is (= {"credit-card" 900.00M "bank" 100.00M} (:paid (by-party designer))))
          (is (false? (:candidate? (by-party designer)))))
        (testing "money in, and transfers with no party, make no payee; the largest paid by bank comes first"
          (is (= [joiner designer] (map :party rows)))))
      (testing "the threshold rises from 2026"
        (is (= 2000.00M (payees/default-threshold 2026))))
      (finally (store/stop node)))))
