(ns ownpurse.scenario.coding-test
  (:require
   [clojure.test :refer [deftest is testing]]
   [ownpurse.scenario.interface :as scenario]
   [ownpurse.store.interface :as store])
  (:import
   (java.time LocalDate)))

(def ^:private checking "acct:robin:checking")

(defn- line [n date merchant amount & {:as more}]
  (into {} (remove (comp nil? val))
        (merge {:xt/id (str "line:test:" n) :account checking :source "test" :source-table "test.transactions"
                :source-row (str "test:" n) :org "test" :date (LocalDate/parse date) :amount amount :status "posted"
                :counterparty merchant :channel "card" :card "card-r"}
               more)))

(def ^:private lines
  [(line 1 "2025-10-03" "Mercado Azul" -47.97M :merchant-code "5411" :original-amount -41.18M :original-currency "EUR")
   ;; Used on the trip's last day, posted the day after: it is coded where the card was used.
   (line 2 "2025-11-01" "RideNow" -12.10M :merchant-code "4121" :original-amount -10.40M :original-currency "EUR"
         :purchased-on (LocalDate/parse "2025-10-31"))
   (line 3 "2025-11-20" "Prairie Market" -63.15M :merchant-code "5411")
   (line 4 "2025-12-01" "Elsewhere" -9.00M :account "acct:other")
   ;; A line the bank says nothing about how it was paid (a fee refunded): no channel, no card.
   (line 5 "2025-12-02" "ATM fee refund" 6.61M :channel nil :card nil)])

;; As a JSON client sends it: strings where the classifier reads keywords and sets.
(def ^:private world
  {:home {:country "United States" :city "Los Angeles"}
   :business "tallowmere"
   :entities {"tallowmere" {:name "Tallowmere Studio" :kind "business"} "robin" {:name "Robin" :kind "household"}}
   :cards {"card-r" {:holder "robin" :entity "robin" :name "Robin's card"}}
   :projects {"harbour" {:name "Harbour Exhibition" :from "2025-09-01" :to "2025-12-31" :place "Lisbon, Portugal"
                         :bears "tallowmere" :travellers ["robin"]}}
   :trips [{:person "robin" :place "Portugal" :city "Lisbon" :from "2025-09-25" :to "2025-10-31" :project "harbour"}]
   :categories {"5411" "Grocery stores and supermarkets" "4121" "Taxis and rideshare"}})

(def ^:private coding
  {:codes {"per-diem" "Per diem: a traveller's own food while on a project trip"
           "travel" "Travel for the project: transport and lodging"
           "personal" "Personal"
           "needs-person" "A person must look"}
   :rules [{:id "R1" :code "per-diem" :rule "Groceries on a project trip are per diem."
            :when [["on-trip-for" "harbour"] ["category" "in" ["5411"]]]}
           {:id "R2" :code "travel" :rule "Taxis on a project trip are travel."
            :when [["on-trip-for" "harbour"] ["category" "in" ["4121"]]]}
           {:id "R3" :code "personal" :rule "Groceries at home are personal." :when [["at-home"] ["category" "in" ["5411"]]]}]})

(def ^:private trial
  {:ref "harbour-coding" :name "Coding the Harbour trip"
   :layers [{:id "world" :kind "world" :world world}
            {:id "coding" :kind "coding" :coding coding}
            {:id "lines" :kind "lines" :lines {:select {:accounts [checking] :from "2025-01-01" :to "2025-12-31"}}}]})

(def ^:private answers
  "What the stand-in answers, by merchant: a bucket and how sure."
  {"Mercado Azul" ["per_diem" 0.97M] "RideNow" ["travel" 0.62M] "Prairie Market" ["personal" 0.93M]
   "ATM fee refund" ["needs_person" 0.9M]})

(defn- stub [calls]
  (fn [{:keys [transaction]} questions]
    (swap! calls inc)
    (let [[choice confidence] (answers (:merchant transaction))]
      {:model "stub" :usage {:input_tokens 10 :output_tokens 1}
       :answers (into {} (for [[id {:keys [type]}] questions]
                           [id (if (= "choice" type)
                                 {:type "choice" :choice choice :confidence confidence}
                                 {:type "noul" :noul 0M})]))})))

(defn- with-node [f]
  (let [node (store/start {:mode :in-memory})]
    (try
      (store/put! node {:bank-lines lines} {:source "test" :why "fixture"})
      (f node)
      (finally (store/stop node)))))

(defn- error [f]
  (try (f) nil (catch clojure.lang.ExceptionInfo e (ex-data e))))

(deftest layers-test
  (with-node
    (fn [node]
      (scenario/put! node trial)
      (testing "the layers come back as they were given, id keys and all"
        (is (= ["card-r"] (keys (get-in (scenario/scenario node "harbour-coding") [:layers 0 :world :cards])))))
      (let [s (scenario/classifier-scenario node "harbour-coding")]
        (testing "the world, the coding and the lines compose into the classifier's scenario"
          (is (= #{:per-diem :travel :personal :needs-person} (set (keys (:codes s)))))
          (is (= [[:on-trip-for "harbour"] [:category :in #{"5411"}]] (:when (first (:rules s))))))
        (testing "the lines are read from the record: money out as a purchase, in the merchant's currency"
          (is (= [["line:test:1" "2025-10-03" 41.18M "EUR" "5411" :card "card-r"]
                  ["line:test:2" "2025-10-31" 10.40M "EUR" "4121" :card "card-r"]
                  ["line:test:3" "2025-11-20" 63.15M nil "5411" :card "card-r"]]
                 (map (juxt :id :date :amount :currency :mcc :channel :card) (take 3 (:cases s)))))))
      (testing "a rule that codes to no bucket, or a trip with no dates, is refused"
        (is (= :ownpurse/invalid
               (:type (error #(scenario/put! node (assoc-in trial [:layers 1 :coding :rules 0 :code] "nowhere"))))))
        (is (= :ownpurse/invalid
               (:type (error #(scenario/put! node (assoc-in trial [:layers 0 :world :trips 0 :from] "soon"))))))))))

(deftest code-test
  (with-node
    (fn [node]
      (scenario/put! node trial)
      (let [calls (atom 0)]
        (testing "the plan says what coding would take before anything is asked"
          (is (= {:lines 4 :units 4 :at-most-requests 9} (scenario/plan node "harbour-coding")))
          (is (zero? @calls)))
        (testing "a run codes every line: sure codings stand, the rest go to a person"
          (is (= {:units 4 :coded 2 :asked 9 :kept 0}
                 (select-keys (scenario/code! node "harbour-coding" (stub calls) {:model "stub"})
                              [:units :coded :asked :kept])))
          (is (= [["line:test:1" "per-diem" true] ["line:test:2" "needs-person" false] ["line:test:3" "personal" true]
                  ["line:test:5" "needs-person" false]]
                 (map (juxt :unit :code :auto) (scenario/codings node "harbour-coding"))))
          (is (= "travel" (:proposed (second (scenario/codings node "harbour-coding"))))))
        (testing "coding again asks nothing: every answer is in the record"
          (is (= {:asked 0 :kept 9} (select-keys (scenario/code! node "harbour-coding" (stub calls) {:model "stub"})
                                                 [:asked :kept])))
          (is (= 9 @calls)))
        (testing "a scenario standing on it with a lower threshold codes the taxi too, and asks nothing new"
          (scenario/put! node {:ref "looser" :name "Trust the taxi" :base "harbour-coding"
                               :layers [{:id "threshold" :kind "coding" :coding {:threshold "0.6"}}]})
          (is (= {:coded 3 :asked 0} (select-keys (scenario/code! node "looser" (stub calls) {:model "stub"})
                                                  [:coded :asked])))
          (is (= [{:unit "line:test:2" :a {:code "needs-person" :proposed "travel" :confidence 0.62M}
                   :b {:code "travel" :proposed "travel" :confidence 0.62M}}]
                 (map #(dissoc % :date :merchant :amount :currency)
                      (scenario/compare-codings node "harbour-coding" "looser")))))
        (testing "a rule added asks again only the request that carries the rules"
          (scenario/put! node {:ref "rule-more" :name "One rule more" :base "harbour-coding"
                               :layers [{:id "more" :kind "coding"
                                         :coding {:rules [{:id "R4" :code "travel" :rule "Rideshare is travel."
                                                           :when [["contains" "merchant" "ride"]]}]}}]})
          (is (= {:asked 4 :kept 5} (select-keys (scenario/code! node "rule-more" (stub calls) {:model "stub"})
                                                 [:asked :kept]))))
        (testing "one unit all the way through"
          (is (= :harbour (keyword (get-in (scenario/coding node "harbour-coding" "line:test:1")
                                           [:facts :whereabouts :project]))))
          (is (= 2 (count (scenario/coding-runs node "harbour-coding")))))))))
