(ns ownpurse.store.put-test
  (:require
   [clojure.test :refer [deftest is testing use-fixtures]]
   [clojure.test.check.clojure-test :refer [defspec]]
   [clojure.test.check.generators :as gen]
   [clojure.test.check.properties :as prop]
   [ownpurse.money.interface.gen :as mgen]
   [ownpurse.store.interface :as store]))

(def ^:dynamic *node* nil)

(use-fixtures :once (fn [f]
                      (let [node (store/start {:mode :in-memory})]
                        (try (binding [*node* node] (f)) (finally (store/stop node))))))

(def ^:private metadata {:source "test" :why "a property"})

(defn- tx-count []
  (:n (first (store/q *node* "SELECT COUNT(*) AS n FROM xt.txs"))))

(defn- error [f]
  (try (f) nil (catch clojure.lang.ExceptionInfo e (ex-data e))))

(def ^:private doc
  (gen/let [id gen/uuid
            amount mgen/amount
            other mgen/amount
            note gen/string-alphanumeric]
    {:xt/id id :amount amount :note note :lines [{:amount other}]}))

(def ^:private float-places
  "Where a float can hide in a document."
  [[:amount] [:lines 0 :amount] [:extra] [:nested :deep]])

(defspec no-float-reaches-the-store 100
  (prop/for-all [docs (gen/vector doc 1 5)
                 i gen/nat
                 place (gen/elements float-places)
                 x (gen/double* {:infinite? false :NaN? false})]
    (let [i (mod i (count docs))
          docs (assoc-in docs (into [i] place) x)
          before (tx-count)
          {:keys [type paths]} (error #(store/put! *node* {:probe docs} metadata))]
      (and (= :ownpurse/float-money type)
           (= [(into [:tables :probe i] place)] paths)
           (= before (tx-count))))))

(defspec exact-amounts-come-back-exactly 50
  (prop/for-all [docs (gen/vector doc 1 5)]
    (store/put! *node* {:exact docs} metadata)
    (let [ids (set (map :xt/id docs))
          back (->> (store/q *node* "SELECT _id, amount, lines FROM exact")
                    (filter (comp ids :xt/id))
                    (map (juxt :xt/id identity))
                    (into {}))]
      (every? (fn [{:keys [xt/id amount lines]}]
                (let [got (back id)]
                  (and (= amount (:amount got))
                       (= (.scale ^BigDecimal amount) (.scale ^BigDecimal (:amount got)))
                       (= (map :amount lines) (map :amount (:lines got))))))
              docs))))

(deftest schema-and-valid-time-test
  (let [doc {:xt/id "a" :total 1.10M :xt/valid-from #inst "2026-03-10T09:30:00Z"}]
    (testing "a namespaced table is a table in that schema, apart from the public one"
      (store/put! *node* {:demo/bills [doc] :bills [{:xt/id "b" :total 2.00M}]} metadata)
      (is (= [{:xt/id "a" :total 1.10M}] (store/q *node* "SELECT _id, total FROM demo.bills")))
      (is (= [{:xt/id "b" :total 2.00M}] (store/q *node* "SELECT _id, total FROM bills"))))
    (testing "a document is valid from the time it carries"
      (is (= #inst "2026-03-10T09:30:00Z"
             (-> (store/q *node* "SELECT _valid_from FROM demo.bills") first :xt/valid-from .toInstant java.util.Date/from))))
    (testing "the same document again writes nothing; changed content writes a version"
      (is (= {:written {} :tx nil} (store/put-changed! *node* {:demo/bills [doc]} metadata)))
      (is (= {:demo/bills 1}
             (:written (store/put-changed! *node* {:demo/bills [(assoc doc :total 1.20M
                                                                       :xt/valid-from #inst "2026-03-11T00:00:00Z")]}
                                           metadata))))
      (is (= [1.10M 1.20M]
             (map :total (store/q *node* "SELECT total FROM demo.bills FOR ALL VALID_TIME ORDER BY _valid_from")))))))

(deftest metadata-test
  (testing "a write without where and why is refused, and nothing is written"
    (let [before (tx-count)]
      (doseq [m [nil {} {:source "test"} {:source "" :why "x"} {:why "x"}]]
        (is (= :ownpurse/no-metadata (:type (error #(store/put! *node* {:t [{:xt/id 1}]} m)))) (pr-str m)))
      (is (= before (tx-count)))))
  (testing "the metadata is kept with the transaction"
    (store/put! *node* {:t [{:xt/id 1 :n 1M}]} {:source "test" :why "kept"})
    (is (= {:source "test" :why "kept"}
           (:user-metadata (last (store/q *node* "SELECT user_metadata FROM xt.txs ORDER BY _id")))))))
