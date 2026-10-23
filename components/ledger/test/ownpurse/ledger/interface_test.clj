(ns ownpurse.ledger.interface-test
  (:require
   [clojure.test :refer [deftest is testing]]
   [ownpurse.ledger.interface :as ledger]
   [ownpurse.store.interface :as store]
   [ownpurse.xero.interface :as xero]))

(def ^:private sample
  {:record "samples/xero-axi/brackenridge/record.sqlite"
   :gl "samples/xero-axi/brackenridge/gl.json"})

(def ^:private source (delay (xero/read-source sample)))

(defn- with-node [f]
  (let [node (store/start {:mode :in-memory})]
    (try (f node) (finally (store/stop node)))))

(defn- tx-count [node]
  (:n (first (store/q node "SELECT COUNT(*) AS n FROM xt.txs"))))

(deftest rows-test
  (let [rows (ledger/rows (xero/facts @source))
        posting (first (:postings rows))]
    (is (= {:sources 4 :books 4 :accounts 49 :parties 23 :documents 39 :entries 39 :postings 78}
           (update-vals rows count)))
    (testing "plain keys, ids as :xt/id, keywords as strings"
      (is (every? (comp nil? namespace) (keys (dissoc posting :xt/id))))
      (is (= (str (:entry posting) ":" (:line posting)) (:xt/id posting)))
      (is (every? string? (map :class (:accounts rows)))))))

(deftest load-test
  (with-node
    (fn [node]
      (let [facts (xero/facts @source)
            metadata (xero/metadata @source)]
        (is (= {:sources 4 :books 4 :accounts 49 :parties 23 :documents 39 :entries 39 :postings 78}
               (:written (ledger/load! node facts metadata))))
        (testing "a second load changes nothing: no rows, no transaction"
          (let [before (tx-count node)]
            (is (= {:written {} :tx nil} (ledger/load! node facts metadata)))
            (is (= before (tx-count node)))))
        (testing "the transaction says where it came from"
          (is (= {:source "xero-axi" :why "load"}
                 (select-keys (:user-metadata (last (store/q node "SELECT user_metadata FROM xt.txs ORDER BY _id")))
                              [:source :why]))))
        (testing "every book nets to zero, in SQL"
          (is (= #{0.00M}
                 (set (map :net (store/q node "SELECT book, SUM(amount) AS net FROM postings GROUP BY book"))))))
        (testing "a document's payload can be queried in SQL"
          (is (= [{:bank "r-op" :total 7316.42M}]
                 (store/q node "SELECT payload #> ARRAY['bank_account', 'account_id'] AS bank, payload->'total' AS total
                                FROM documents WHERE source_id = 'bt-101'"))))
        (testing "a changed document rewrites only that entry's postings"
          (let [entry-id (:entry/id (first (:entries facts)))
                changed (update facts :entries
                                (fn [es] (mapv #(if (= entry-id (:entry/id %))
                                                  (-> % (update-in [:entry/postings 0 :posting/amount] + 1M)
                                                      (update-in [:entry/postings 1 :posting/amount] - 1M))
                                                  %)
                                               es)))
                {:keys [written tx]} (ledger/load! node changed metadata)]
            (is (= {:postings 2} written))
            (is (some? tx))))
        (testing "an unbalanced entry is refused before anything is written"
          (let [before (tx-count node)
                broken (update-in facts [:entries 0 :entry/postings 0 :posting/amount] + 0.01M)]
            (is (= :ownpurse/invalid
                   (try (ledger/load! node broken metadata)
                        (catch clojure.lang.ExceptionInfo e (:type (ex-data e))))))
            (is (= before (tx-count node)))))))))
