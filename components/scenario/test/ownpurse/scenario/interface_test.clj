(ns ownpurse.scenario.interface-test
  (:require
   [clojure.test :refer [deftest is testing]]
   [ownpurse.close.interface :as close]
   [ownpurse.scenario.interface :as scenario]
   [ownpurse.store.interface :as store])
  (:import
   (java.time LocalDate)))

(def ^:private book "book:test:tallowmere")
(def ^:private other-book "book:test:ostley")
(def ^:private rent "acct:6200")
(def ^:private prepaid "acct:1400")
(def ^:private studio "acct:6210")

(defn- d [s] (LocalDate/parse s))

(def ^:private split-rent
  {:ref "J-split" :book book :date (d "2025-12-31") :narration "A third of the rent is the studio's"
   :postings [{:account studio :amount 400.00M :commodity "USD"}
              {:account rent :amount -400.00M :commodity "USD"}]})

(def ^:private split
  {:ref "rent-split" :name "Rent split three ways"
   :layers [{:id "rent" :kind "journals" :journals [split-rent]}]})

(defn- with-node [f]
  (let [node (store/start {:mode :in-memory})]
    (try (f node) (finally (store/stop node)))))

(defn- error [f]
  (try (f) nil (catch clojure.lang.ExceptionInfo e (ex-data e))))

(deftest layers-test
  (with-node
    (fn [node]
      (testing "a scenario's journals add to its book, and to no other"
        (scenario/put! node split)
        (is (= {studio 400.00M rent -400.00M} (scenario/balances node "rent-split" book {})))
        (is (= {} (scenario/balances node "rent-split" other-book {}))))
      (testing "only the journals dated within the period count"
        (is (= {} (scenario/balances node "rent-split" book {:to (d "2025-12-30")})))
        (is (= {studio 400.00M rent -400.00M}
               (scenario/balances node "rent-split" book {:from (d "2025-01-01") :to (d "2025-12-31")}))))
      (testing "a layer turned off adds nothing, and says so"
        (scenario/put-layer! node "rent-split" {:id "rent" :kind "journals" :enabled false :journals [split-rent]})
        (let [{:keys [balances layers]} (scenario/evaluate node "rent-split" book {})]
          (is (= {} balances))
          (is (= [["rent-split" "rent" false {}]] (map (juxt :scenario :layer :enabled :balances) layers)))))
      (testing "a journal that does not balance, an unknown kind, or two layers of one id are refused"
        (is (= :ownpurse/invalid
               (:type (error #(scenario/put! node (assoc-in split [:layers 0 :journals 0 :postings 0 :amount] 400.01M))))))
        (is (= :ownpurse/invalid (:type (error #(scenario/put! node (assoc-in split [:layers 0 :kind] "rules"))))))
        (is (= :ownpurse/invalid
               (:type (error #(scenario/put! node (update split :layers conj (first (:layers split))))))))))))

(deftest stack-test
  (with-node
    (fn [node]
      (scenario/put! node split)
      (testing "a scenario standing on another adds its own layers to the other's"
        (scenario/put! node {:ref "prepay" :name "And January's rent prepaid" :base "rent-split"
                             :layers [{:id "prepay" :kind "journals"
                                       :journals [{:ref "J-prepay" :book book :date (d "2025-12-31")
                                                   :narration "January rent is prepaid"
                                                   :postings [{:account prepaid :amount 1200.00M :commodity "USD"}
                                                              {:account rent :amount -1200.00M :commodity "USD"}]}]}]})
        (let [{:keys [stack balances layers]} (scenario/evaluate node "prepay" book {})]
          (is (= ["rent-split" "prepay"] stack))
          (is (= {studio 400.00M rent -1600.00M prepaid 1200.00M} balances))
          (is (= [["rent-split" "rent"] ["prepay" "prepay"]] (map (juxt :scenario :layer) layers)))))
      (testing "a change to the base shows through; the scenario on it is not touched"
        (scenario/put-layer! node "rent-split" {:id "rent" :kind "journals" :enabled false :journals [split-rent]})
        (is (= {rent -1200.00M prepaid 1200.00M} (scenario/balances node "prepay" book {}))))
      (testing "a base that does not exist, or one that stands on the scenario, is refused"
        (is (= :ownpurse/not-found (:type (error #(scenario/put! node (assoc split :ref "x" :base "nowhere"))))))
        (is (= :ownpurse/invalid (:type (error #(scenario/put! node (assoc split :base "prepay")))))))
      (testing "a scenario something stands on cannot be dropped"
        (is (= :ownpurse/scenario-conflict (:type (error #(scenario/drop! node "rent-split")))))))))

(deftest duplicate-drop-and-versions-test
  (with-node
    (fn [node]
      (scenario/put! node split)
      (testing "a duplicate starts as the original and changes apart from it"
        (is (= "rent-split" (:duplicated-from (scenario/duplicate! node "rent-split" {:ref "try-half"}))))
        (is (= "Rent split three ways (copy)" (:name (scenario/scenario node "try-half"))))
        (scenario/put-layer! node "try-half" {:id "rent" :kind "journals"
                                              :journals [(assoc-in (assoc-in split-rent [:postings 0 :amount] 600.00M)
                                                                   [:postings 1 :amount] -600.00M)]})
        (is (= {studio 600.00M rent -600.00M} (scenario/balances node "try-half" book {})))
        (is (= {studio 400.00M rent -400.00M} (scenario/balances node "rent-split" book {}))))
      (testing "duplicating onto a scenario that exists is refused"
        (is (= :ownpurse/scenario-conflict (:type (error #(scenario/duplicate! node "rent-split" {:ref "try-half"}))))))
      (testing "every version is kept, oldest first"
        (is (= [[1 400.00M] [2 600.00M]]
               (map (juxt :version #(get-in % [:layers 0 :journals 0 :postings 0 :amount]))
                    (scenario/versions node "try-half")))))
      (testing "a dropped scenario counts nowhere; its versions stay"
        (is (= {:ref "try-half" :dropped true} (scenario/drop! node "try-half")))
        (is (nil? (scenario/scenario node "try-half")))
        (is (not-any? #{"try-half"} (map :ref (scenario/scenarios node))))
        (is (= 2 (count (scenario/versions node "try-half")))))
      (testing "the built-in scenarios are always there and cannot change"
        (is (= ["approved" "proposed" "rent-split"] (map :ref (scenario/scenarios node))))
        (is (= :ownpurse/scenario-conflict (:type (error #(scenario/put! node (assoc split :ref "approved"))))))
        (is (= :ownpurse/scenario-conflict (:type (error #(scenario/drop! node "proposed")))))))))

(deftest close-layer-test
  (with-node
    (fn [node]
      (close/put-item! node {:ref "C-01" :book book :title "January rent paid in December" :status "accountant decision"
                             :journals [{:ref "J1" :date (d "2025-12-31") :narration "January rent is prepaid"
                                         :postings [{:account prepaid :amount 1200.00M :commodity "USD"}
                                                    {:account rent :amount -1200.00M :commodity "USD"}]}]})
      (testing "the built-in scenarios read the close register as approved and as proposed"
        (is (= {} (scenario/balances node "approved" book {})))
        (is (= (close/balances node book {:basis :proposed}) (scenario/balances node "proposed" book {}))))
      (testing "a scenario stands on the proposed close and tries one more journal"
        (scenario/put! node (assoc split :base "proposed"))
        (is (= {prepaid 1200.00M rent -1600.00M studio 400.00M} (scenario/balances node "rent-split" book {})))))))
