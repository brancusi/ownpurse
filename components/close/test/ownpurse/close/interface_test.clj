(ns ownpurse.close.interface-test
  (:require
   [clojure.test :refer [deftest is testing]]
   [ownpurse.close.interface :as close]
   [ownpurse.store.interface :as store])
  (:import
   (java.time LocalDate)))

(def ^:private book "book:test:tallowmere")
(def ^:private other-book "book:test:ostley")
(def ^:private rent "acct:6200")
(def ^:private prepaid "acct:1400")

(def ^:private prepaid-rent
  {:ref "C-01" :book book :title "January rent paid in December" :category "prior-period" :status "accountant decision"
   :action "Move January's rent to prepaid" :amount 1200.00M :effect "Rent down, prepaid up"
   :depends-on [] :evidence ["line:bank:feed:txn-0042"] :group "G-1"
   :journals [{:ref "J1" :date (LocalDate/parse "2025-12-31") :narration "January rent is prepaid"
               :postings [{:account prepaid :amount 1200.00M :commodity "USD"}
                          {:account rent :amount -1200.00M :commodity "USD" :description "January rent"}]}]})

(def ^:private question
  {:ref "C-02" :book book :title "Owner-paid software" :status "evidence needed"
   :action "Find the receipts, then decide whether it is a loan from the owner"})

(defn- with-node [f]
  (let [node (store/start {:mode :in-memory})]
    (try (f node) (finally (store/stop node)))))

(defn- error [f]
  (try (f) nil (catch clojure.lang.ExceptionInfo e (ex-data e))))

(deftest items-test
  (with-node
    (fn [node]
      (testing "an item is a question with a workflow status; it may carry journals or none"
        (is (= [1200.00M -1200.00M] (map :amount (:postings (first (:journals (close/put-item! node prepaid-rent)))))))
        (is (= "evidence needed" (:status (close/put-item! node question))))
        (is (= ["C-01" "C-02"] (map :ref (close/items node {:book book}))))
        (is (= ["C-02"] (map :ref (close/items node {:status "evidence needed"})))))
      (testing "a journal that does not balance, or an item without a title or status, is refused"
        (is (= :ownpurse/invalid
               (:type (error #(close/put-item! node (assoc-in prepaid-rent [:journals 0 :postings 0 :amount] 1200.01M))))))
        (is (= :ownpurse/invalid (:type (error #(close/put-item! node (dissoc question :status))))))
        (is (= :ownpurse/invalid (:type (error #(close/put-item! node (assoc question :ref "C 2")))))))
      (testing "an intercompany item's journals may be in two books; each book finds it"
        (close/put-item! node {:ref "C-03" :book book :title "Loan between the orgs" :status "treatment needed"
                               :journals [{:ref "A" :date (LocalDate/parse "2025-12-31") :narration "Loan to Ostley"
                                           :postings [{:account "acct:1800" :amount 500M :commodity "USD"}
                                                      {:account "acct:1010" :amount -500M :commodity "USD"}]}
                                          {:ref "B" :book other-book :date (LocalDate/parse "2025-12-31")
                                           :narration "Loan from Tallowmere"
                                           :postings [{:account "acct:1010" :amount 500M :commodity "USD"}
                                                      {:account "acct:2800" :amount -500M :commodity "USD"}]}]})
        (is (= ["C-03"] (map :ref (close/items node {:book other-book})))))
      (testing "decision groups gather items by their group"
        (is (= ["C-01"] (:items (close/put-group! node {:ref "G-1" :title "Prepayments" :prompt "Which to defer?"}))))
        (is (= [["G-1" ["C-01"]]] (map (juxt :ref :items) (close/groups node))))))))

(deftest package-test
  (with-node
    (fn [node]
      (close/put-item! node prepaid-rent)
      (close/put-item! node question)
      (testing "nothing counts as approved before a package is"
        (is (= {} (close/balances node book {})))
        (is (= {prepaid 1200.00M rent -1200.00M} (close/balances node book {:basis :proposed}))))
      (let [package (close/seal! node {:ref "P-2025-1" :as-of "2026-03-01" :scope "2025 close" :items ["C-01" "C-02"]
                                       :coverage {:mapped 2}})]
        (testing "a sealed package keeps each item's hash and one hash over them"
          (is (= [(close/content-hash (close/item node "C-01")) (close/content-hash (close/item node "C-02"))]
                 (:hashes package)))
          (is (= [] (:changed package)))
          (is (nil? (:approval package))))
        (testing "a package is never changed: sealing the same name again is refused"
          (is (= :ownpurse/close-conflict (:type (error #(close/seal! node {:ref "P-2025-1" :items ["C-01"]})))))))
      (testing "a status change does not change what was sealed"
        (close/put-item! node (assoc prepaid-rent :status "exact candidate"))
        (is (= [] (:changed (close/package node "P-2025-1")))))
      (testing "the owner approves the package with the hash they saw"
        (is (= :ownpurse/close-conflict
               (:type (error #(close/approve! node {:package "P-2025-1" :hash "not-it" :by "owner"})))))
        (let [{:keys [hash]} (close/package node "P-2025-1")
              approved (close/approve! node {:package "P-2025-1" :hash hash :by "owner"})]
          (is (= "owner" (get-in approved [:approval :by])))
          (is (= :ownpurse/close-conflict
                 (:type (error #(close/approve! node {:package "P-2025-1" :hash hash :by "owner"})))))))
      (testing "an item in an approved package counts in the pro-forma, whatever its status label"
        (is (close/approved? node (close/item node "C-01")))
        (is (= {prepaid 1200.00M rent -1200.00M} (close/balances node book {})))
        (is (= {} (close/balances node book {:to (LocalDate/parse "2025-12-30")}))))
      (testing "an item changed after approval no longer counts, and the package shows it"
        (close/put-item! node (assoc question :action "Receipts found; treat as a loan"))
        (is (= ["C-02"] (:changed (close/package node "P-2025-1"))))
        (is (not (close/approved? node (close/item node "C-02"))))
        (is (= :ownpurse/close-conflict
               (:type (error #(close/approve! node {:package "P-2025-1" :hash "x" :by "owner"}))))))
      (testing "a set-aside item counts nowhere"
        (close/put-item! node (assoc question :status "rejected" :journals (:journals prepaid-rent)))
        (is (= {prepaid 1200.00M rent -1200.00M} (close/balances node book {:basis :proposed})))))))

(deftest posted-test
  (with-node
    (fn [node]
      (close/put-item! node prepaid-rent)
      (testing "only an approved item's journal is posted"
        (is (= :ownpurse/close-conflict
               (:type (error #(close/mark-posted! node {:item "C-01" :journal "J1" :reference "MJ-0091"}))))))
      (let [{:keys [hash]} (close/seal! node {:ref "P-1" :items ["C-01"]})]
        (close/approve! node {:package "P-1" :hash hash :by "owner"}))
      (testing "posting records where, and keeps the item approved; it leaves the pro-forma"
        (is (= :ownpurse/invalid (:type (error #(close/mark-posted! node {:item "C-01" :journal "J1" :reference ""})))))
        (is (= "MJ-0091" (-> (close/mark-posted! node {:item "C-01" :journal "J1" :reference "MJ-0091"})
                             :journals first :posted-reference)))
        (is (close/approved? node (close/item node "C-01")))
        (is (= {} (close/balances node book {}))))
      (testing "a posted journal cannot change or go, but the item's other fields can"
        (is (= :ownpurse/close-conflict
               (:type (error #(close/put-item! node (assoc-in prepaid-rent [:journals 0 :narration] "other"))))))
        (is (= :ownpurse/close-conflict (:type (error #(close/put-item! node (assoc prepaid-rent :journals [])))))))
      (is (= "MJ-0091" (-> (close/put-item! node (assoc prepaid-rent :status "posted")) :journals first :posted-reference))))))
