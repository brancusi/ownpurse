(ns ownpurse.intercompany.interface-test
  (:require
   [clojure.test :refer [deftest is testing]]
   [ownpurse.intercompany.interface :as ic]
   [ownpurse.ledger.interface :as ledger]
   [ownpurse.store.interface :as store])
  (:import
   (java.time LocalDate)))

;; A household (book a) pays costs of a business (book b) and lends it money. The household spreads the receivable
;; over an account per cost category and a clearing account; the business holds it in one owner-loan account,
;; with a second person's loan in the same account, told apart by contact.
(def ^:private a "book:test:household")
(def ^:private b "book:test:business")
(def ^:private owner "party:test:owner")
(def ^:private partner "party:test:partner")

(defn- day [s] (LocalDate/parse s))

(defn- entry [book id date party & postings]
  {:entry/id id :entry/book book :entry/date (day date) :entry/narration id
   :entry/postings (vec (map-indexed (fn [i [account amount]]
                                       (cond-> {:posting/line i :posting/account account :posting/amount amount
                                                :posting/commodity "USD"}
                                         party (assoc :posting/party party)))
                                     postings))})

(def ^:private household-entries
  [(entry a "a-prior" "2024-12-31" nil ["a:1810" 25.00M] ["a:1010" -25.00M])
   ;; One payment split over two categories on the household's side.
   (entry a "a-feb" "2025-02-01" nil ["a:1810" 300.00M] ["a:1820" 200.00M] ["a:1010" -500.00M])
   (entry a "a-mar" "2025-03-10" nil ["a:1890" 100.00M] ["a:1010" -100.00M])
   ;; Nothing on the business's side mirrors this one.
   (entry a "a-jun" "2025-06-01" nil ["a:1810" 75.00M] ["a:1010" -75.00M])])

(def ^:private business-entries
  [(entry b "b-feb" "2025-02-02" owner ["b:6000" 500.00M] ["b:2600" -500.00M])
   (entry b "b-mar" "2025-03-11" owner ["b:1010" 100.00M] ["b:2600" -100.00M])
   (entry b "b-partner" "2025-04-01" partner ["b:1010" 1000.00M] ["b:2600" -1000.00M])
   ;; A journal names no contact, so it cannot be told to be the owner's.
   (entry b "b-journal" "2025-05-01" nil ["b:6100" 40.00M] ["b:2600" -40.00M])])

(def ^:private owner-loan
  {:ref "owner-loan" :name "Owner's loan to the business" :from (day "2025-01-01")
   :a {:book a :accounts ["a:1810" "a:1820" "a:1890"]}
   :b {:book b :accounts ["b:2600"] :parties [owner]}})

(defn- with-books [f]
  (let [node (store/start {:mode :in-memory})]
    (try
      (ledger/put-entries! node a household-entries {:source "test" :why "fixture"})
      (ledger/put-entries! node b business-entries {:source "test" :why "fixture"})
      (f node)
      (finally (store/stop node)))))

(defn- error [f]
  (try (f) nil (catch clojure.lang.ExceptionInfo e (ex-data e))))

(deftest pairs-test
  (with-books
    (fn [node]
      (testing "a pair needs a book and an account each side, and dates in order"
        (is (= :ownpurse/invalid (:type (error #(ic/put-pair! node (assoc-in owner-loan [:b :accounts] []))))))
        (is (= :ownpurse/invalid (:type (error #(ic/put-pair! node (assoc owner-loan :to (day "2024-12-31"))))))))
      (testing "a pair is held as given"
        (is (= ["a:1810" "a:1820" "a:1890"] (get-in (ic/put-pair! node owner-loan) [:a :accounts])))
        (is (= ["owner-loan"] (map :ref (ic/pairs node))))))))

(deftest report-test
  (with-books
    (fn [node]
      (ic/put-pair! node owner-loan)
      (let [[r] (ic/report node {:from (day "2025-01-01") :to (day "2025-12-31")})]
        (testing "each side's balance, over several accounts on one side and one party's share on the other"
          (is (= 700.00M (get-in r [:a :balance])))
          (is (= -600.00M (get-in r [:b :balance])))
          (is (= 675.00M (get-in r [:a :movement]))))
        (testing "the difference is the balances summed"
          (is (= 100.00M (:difference r)))
          (is (false? (:reconciled? r))))
        (testing "a line split over two accounts matches its mirror by its entry; what has no mirror is listed"
          (is (= 2 (:matched r)))
          (is (= [["a-jun" 75.00M]] (map (juxt :entry :amount) (get-in r [:unmatched :a]))))
          (is (= [] (get-in r [:unmatched :b]))))
        (testing "lines that name no party are listed apart, not counted"
          (is (= [["b-journal" -40.00M]] (map (juxt :entry :amount) (get-in r [:b :unattributed]))))))
      (testing "a pinned explanation of part of the difference leaves the rest unexplained"
        (ic/put-pair! node (assoc owner-loan :explained [{:ref "prior" :amount 25.00M :note "2024 receivable"}]))
        (let [[r] (ic/report node {:from (day "2025-01-01") :to (day "2025-12-31")})]
          (is (= 75.00M (:unexplained r)))))
      (testing "a pair that ends before the period is left out"
        (ic/put-pair! node (assoc owner-loan :to (day "2025-01-31")))
        (is (= [] (ic/report node {:from (day "2025-02-01") :to (day "2025-12-31")})))))))
