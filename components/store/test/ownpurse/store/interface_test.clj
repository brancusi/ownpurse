(ns ownpurse.store.interface-test
  (:require
   [clojure.test :refer [deftest is testing]]
   [ownpurse.store.core :as core]
   [ownpurse.store.interface :as store]
   [xtdb.api :as xt]))

(defn- with-node [config f]
  (let [node (store/start config)]
    (try (f node) (finally (store/stop node)))))

(deftest node-config-test
  (testing "no listeners unless asked: Flight SQL and the SQL server are off"
    (let [cfg (core/node-config {:mode :in-memory})]
      (is (nil? (.getFlightSql cfg)))
      (is (nil? (.getServer cfg)))))
  (testing "the SQL server is read-only"
    (let [server (.getServer (core/node-config {:mode :in-memory :sql {:host "127.0.0.1" :read-only-port 0}}))]
      (is (= -1 (.getPort server)))
      (is (= 0 (.getReadOnlyPort server))))))

(deftest in-memory-node-test
  (with-node {:mode :in-memory}
    (fn [node]
      (is (= {:ok? true} (store/health node)))
      (is (nil? (store/sql-port node)))
      (testing "a DECIMAL keeps its scale, and the transaction keeps its metadata"
        (xt/execute-tx node [[:sql "INSERT INTO probe (_id, amount) VALUES (1, 12.3450::DECIMAL(32,4))"]]
                       {:metadata {:source "test"}})
        (let [[{:keys [amount]}] (xt/q node "SELECT amount FROM probe")]
          (is (= 12.3450M amount))
          (is (= 4 (.scale ^BigDecimal amount))))
        (is (= [{:user-metadata {:source "test"}}] (xt/q node "SELECT user_metadata FROM xt.txs"))))
      (testing "the read-only SQL port binds when asked for"
        (with-node {:mode :in-memory :sql {:host "127.0.0.1" :read-only-port 0}}
          (fn [node] (is (pos-int? (store/sql-port node)))))))))
