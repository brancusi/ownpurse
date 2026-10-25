(ns ownpurse.ledger.interface-test
  (:require
   [clojure.test :refer [deftest is testing]]
   [ownpurse.ledger.interface :as ledger]
   [ownpurse.store.interface :as store])
  (:import
   (java.time Instant LocalDate)))

;; A small invented book: Tallowmere sells coffee on account, then is paid.

(def ^:private facts
  {:sources [{:source/id "test:tallowmere" :source/tool "test" :source/org "tallowmere" :source/record "inline"}]
   :books [{:book/id "book:tallowmere" :book/name "Tallowmere Coffee Roasting Co." :book/currency "USD"
            :book/source "test:tallowmere" :book/fiscal-year-end {:month 12 :day 31}}]
   :accounts [{:account/id "acct:1010" :account/book "book:tallowmere" :account/code "1010"
               :account/name "Operating account" :account/class :asset}
              {:account/id "acct:1200" :account/book "book:tallowmere" :account/code "1200"
               :account/name "Accounts receivable" :account/class :asset}
              {:account/id "acct:4000" :account/book "book:tallowmere" :account/code "4000"
               :account/name "Wholesale coffee sales" :account/class :income}]
   :parties [{:party/id "party:fernhill" :party/book "book:tallowmere" :party/name "Fernhill Grocers"}]
   :documents [{:document/id "doc:inv-0418" :document/book "book:tallowmere" :document/kind "Invoices"
                :document/source-id "inv-0418" :document/version 1 :document/content-hash "test"
                :document/status "AUTHORISED" :document/date (LocalDate/parse "2026-03-19")
                :document/observed-at (Instant/parse "2026-03-20T17:05:00Z")
                :document/payload {:contact {:name "Fernhill Grocers"} :total 1250.00M}}]
   :entries [{:entry/id "entry:inv-0418" :entry/book "book:tallowmere" :entry/date (LocalDate/parse "2026-03-19")
              :entry/narration "INV-0418" :entry/document "doc:inv-0418"
              :entry/postings [{:posting/line 1 :posting/account "acct:1200" :posting/amount 1250.00M
                                :posting/commodity "USD" :posting/party "party:fernhill"}
                               {:posting/line 2 :posting/account "acct:4000" :posting/amount -1250.00M
                                :posting/commodity "USD"}]}
             {:entry/id "entry:pay-0418" :entry/book "book:tallowmere" :entry/date (LocalDate/parse "2026-03-26")
              :entry/narration "Fernhill Grocers"
              :entry/postings [{:posting/line 1 :posting/account "acct:1010" :posting/amount 1250.00M
                                :posting/commodity "USD"}
                               {:posting/line 2 :posting/account "acct:1200" :posting/amount -1250.00M
                                :posting/commodity "USD" :posting/party "party:fernhill"}]}]})

(def ^:private metadata {:source "test" :why "load"})

(def ^:private counts {:sources 1 :books 1 :accounts 3 :parties 1 :documents 1 :entries 2 :postings 4})

(defn- with-node [f]
  (let [node (store/start {:mode :in-memory})]
    (try (f node) (finally (store/stop node)))))

(defn- tx-count [node]
  (:n (first (store/q node "SELECT COUNT(*) AS n FROM xt.txs"))))

(deftest rows-test
  (let [rows (ledger/rows facts)
        posting (first (:postings rows))]
    (is (= counts (update-vals rows count)))
    (testing "plain keys, ids as :xt/id, keywords as strings"
      (is (every? (comp nil? namespace) (keys (dissoc posting :xt/id))))
      (is (= (str (:entry posting) ":" (:line posting)) (:xt/id posting)))
      (is (every? string? (map :class (:accounts rows)))))))

(deftest load-test
  (with-node
    (fn [node]
      (is (= counts (:written (ledger/load! node facts metadata))))
      (testing "a second load changes nothing: no rows, no transaction"
        (let [before (tx-count node)]
          (is (= {:written {} :tx nil} (ledger/load! node facts metadata)))
          (is (= before (tx-count node)))))
      (testing "the transaction says where it came from"
        (is (= metadata
               (select-keys (:user-metadata (last (store/q node "SELECT user_metadata FROM xt.txs ORDER BY _id")))
                            [:source :why]))))
      (testing "the book nets to zero, in SQL"
        (is (= [0.00M] (map :net (store/q node "SELECT book, SUM(amount) AS net FROM postings GROUP BY book")))))
      (testing "a document's payload can be queried in SQL"
        (is (= [{:party "Fernhill Grocers" :total 1250.00M}]
               (store/q node "SELECT payload #> ARRAY['contact', 'name'] AS party, payload->'total' AS total
                              FROM documents WHERE source_id = 'inv-0418'"))))
      (testing "a changed entry rewrites only its postings"
        (let [changed (update facts :entries
                              (fn [es] (update es 0 #(-> %
                                                         (update-in [:entry/postings 0 :posting/amount] + 1M)
                                                         (update-in [:entry/postings 1 :posting/amount] - 1M)))))
              {:keys [written tx]} (ledger/load! node changed metadata)]
          (is (= {:postings 2} written))
          (is (some? tx))))
      (testing "an unbalanced entry is refused before anything is written"
        (let [before (tx-count node)
              broken (update-in facts [:entries 0 :entry/postings 0 :posting/amount] + 0.01M)]
          (is (= :ownpurse/invalid
                 (try (ledger/load! node broken metadata)
                      (catch clojure.lang.ExceptionInfo e (:type (ex-data e))))))
          (is (= before (tx-count node))))))))
