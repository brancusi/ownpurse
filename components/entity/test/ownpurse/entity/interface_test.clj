(ns ownpurse.entity.interface-test
  (:require
   [clojure.test :refer [deftest is testing]]
   [ownpurse.entity.interface :as entity]
   [ownpurse.store.interface :as store])
  (:import
   (java.time LocalDate)))

(def ^:private xero-org "xero:t1:organisation:o1")
(def ^:private xero-1010 "xero:t1:account:a1010")
(def ^:private other-org "xero:t2:organisation:o2")
(def ^:private other-bank "xero:t2:account:b1")
(def ^:private mercury-org "mercury:m1:organization:m1")
(def ^:private operating "mercury:m1:account:a1")
(def ^:private card "mercury:m1:credit-account:c1")

(def ^:private accounts
  [{:xt/id xero-org :source "xero" :source-table "xero.organisations" :org "t1" :kind "org" :name "Tallowmere"
    :status "active"}
   {:xt/id xero-1010 :source "xero" :source-table "xero.accounts" :org "t1" :kind "bank"
    :name "Operating account" :code "1010" :currency "USD" :status "active"}
   {:xt/id other-org :source "xero" :source-table "xero.organisations" :org "t2" :kind "org" :name "Fernhill"
    :status "active"}
   {:xt/id other-bank :source "xero" :source-table "xero.accounts" :org "t2" :kind "bank" :name "Checking"
    :status "active"}
   {:xt/id mercury-org :source "mercury" :source-table "mercury.organizations" :org "m1" :kind "org"
    :name "Tallowmere Coffee Roasting Co." :status "active"}
   {:xt/id operating :source "mercury" :source-table "mercury.accounts" :org "m1" :kind "bank"
    :name "Operating (Mercury Checking ••2210)" :status "active"}
   {:xt/id card :source "mercury" :source-table "mercury.credit_accounts" :org "m1" :kind "credit-card"
    :name "Credit card" :status "active"}])

(defn- with-node [f]
  (let [node (store/start {:mode :in-memory})]
    (try (f node) (finally (store/stop node)))))

(defn- error [f]
  (try (f) nil (catch clojure.lang.ExceptionInfo e (ex-data e))))

(deftest empty-record-test
  (with-node
    (fn [node]
      (testing "before anything is written there are no entities and no source accounts"
        (is (= [] (entity/entities node)))
        (is (= [] (entity/source-accounts node {})))
        (is (nil? (entity/source-account node operating)))))))

(deftest entities-test
  (with-node
    (fn [node]
      (store/put! node {:source-accounts accounts} {:source "test" :why "fixtures"})
      (testing "an entity is made, and made again with a new name"
        (is (= {:id "tallowmere" :name "Tallowmere" :kind "business"}
               (entity/put-entity! node {:id "tallowmere" :name "Tallowmere" :kind "business"})))
        (entity/put-entity! node {:id "tallowmere" :name "Tallowmere Coffee Roasting Co." :kind "business"})
        (entity/put-entity! node {:id "home" :name "Home" :kind "household"})
        (is (= ["Home" "Tallowmere Coffee Roasting Co."] (map :name (entity/entities node)))))
      (testing "an entity's id is a slug"
        (is (= :ownpurse/invalid (:type (error #(entity/put-entity! node {:id "Tallow Mere" :name "x"
                                                                          :kind "business"}))))))

      (testing "a link names an entity and a source account that exist"
        (is (= :ownpurse/not-found
               (:type (error #(entity/link! node {:entity "nobody" :account operating :role "evidence"})))))
        (is (= :ownpurse/not-found
               (:type (error #(entity/link! node {:entity "tallowmere" :account "mercury:m1:account:zz"
                                                  :role "evidence"}))))))

      (testing "a system of record is an organisation, one per entity"
        (is (= :ownpurse/invalid-link
               (:type (error #(entity/link! node {:entity "tallowmere" :account xero-1010
                                                  :role "system-of-record"})))))
        (is (= {:entity "tallowmere" :account xero-org :role "system-of-record" :from "2025-01-01"}
               (entity/link! node {:entity "tallowmere" :account xero-org :role "system-of-record"
                                   :from (LocalDate/parse "2025-01-01")})))
        (is (= :ownpurse/link-conflict
               (:type (error #(entity/link! node {:entity "tallowmere" :account other-org
                                                  :role "system-of-record"}))))))

      (testing "evidence is an account, for an account of the entity's system of record"
        (is (= :ownpurse/invalid-link
               (:type (error #(entity/link! node {:entity "tallowmere" :account mercury-org :role "evidence"})))))
        (is (= :ownpurse/invalid-link
               (:type (error #(entity/link! node {:entity "tallowmere" :account operating :role "evidence"
                                                  :evidence-for other-bank})))))
        (is (= :ownpurse/invalid-link
               (:type (error #(entity/link! node {:entity "tallowmere" :account operating :role "evidence"
                                                  :evidence-for xero-org})))))
        (is (= {:entity "tallowmere" :account operating :role "evidence" :evidence-for xero-1010
                :from "2025-01-01"}
               (entity/link! node {:entity "tallowmere" :account operating :role "evidence"
                                   :evidence-for xero-1010 :from (LocalDate/parse "2025-01-01")}))))

      (testing "an entity without a system of record has evidence, but not evidence for an account"
        (is (= :ownpurse/invalid-link
               (:type (error #(entity/link! node {:entity "home" :account card :role "evidence"
                                                  :evidence-for xero-1010})))))
        (entity/link! node {:entity "home" :account card :role "evidence" :from (LocalDate/parse "2025-01-01")}))

      (testing "an account belongs to one entity at a time"
        (is (= :ownpurse/link-conflict
               (:type (error #(entity/link! node {:entity "home" :account operating :role "evidence"}))))))

      (testing "each entity lists its links, and the accounts not linked are easy to find"
        (is (= [{:id "home" :name "Home" :kind "household"
                 :links [{:entity "home" :account card :role "evidence" :from "2025-01-01"}]}
                {:id "tallowmere" :name "Tallowmere Coffee Roasting Co." :kind "business"
                 :links [{:entity "tallowmere" :account operating :role "evidence" :evidence-for xero-1010
                          :from "2025-01-01"}
                         {:entity "tallowmere" :account xero-org :role "system-of-record" :from "2025-01-01"}]}]
               (entity/entities node)))
        (is (= #{xero-1010 other-org other-bank mercury-org}
               (set (map :id (entity/source-accounts node {:unlinked true})))))
        (is (= [mercury-org operating card] (map :id (entity/source-accounts node {:source "mercury"}))))
        (is (= {:entity "home" :role "evidence" :from "2025-01-01"}
               (:link (first (filter #(= card (:id %)) (entity/source-accounts node {}))))))))))

(deftest unlink-test
  (with-node
    (fn [node]
      (store/put! node {:source-accounts accounts} {:source "test" :why "fixtures"})
      (entity/put-entity! node {:id "tallowmere" :name "Tallowmere" :kind "business"})
      (entity/put-entity! node {:id "home" :name "Home" :kind "household"})
      (entity/link! node {:entity "tallowmere" :account card :role "evidence" :from (LocalDate/parse "2025-01-01")})
      (testing "a link cannot end before it starts"
        (is (= :ownpurse/invalid-link
               (:type (error #(entity/unlink! node {:account card :on (LocalDate/parse "2024-12-31")}))))))
      (testing "ending a link keeps its history"
        (is (= {:entity "tallowmere" :account card :role "evidence" :from "2025-01-01" :until "2025-07-01"}
               (entity/unlink! node {:account card :on (LocalDate/parse "2025-07-01")})))
        (is (= [] (:links (first (filter #(= "tallowmere" (:id %)) (entity/entities node))))))
        (is (= [{:entity "entity:tallowmere"}]
               (store/q node ["SELECT entity FROM entity_links FOR VALID_TIME AS OF ? WHERE _id = ?"
                              #inst "2025-03-01T00:00:00Z" (str "link:" card)]))))
      (testing "an account that is not linked has nothing to end"
        (is (= :ownpurse/not-found (:type (error #(entity/unlink! node {:account card}))))))
      (testing "once ended, the account can belong to another entity"
        (is (= "home" (:entity (entity/link! node {:entity "home" :account card :role "evidence"}))))))))
