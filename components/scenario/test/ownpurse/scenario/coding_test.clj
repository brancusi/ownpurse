(ns ownpurse.scenario.coding-test
  (:require
   [clojure.string :as str]
   [clojure.test :refer [deftest is testing]]
   [ownpurse.classify.interface :as classify]
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
      (testing "the composed stack says which rules stand and which lines it reads, without reading them"
        (let [c (scenario/composed node "harbour-coding")]
          (is (= ["R1" "R2" "R3"] (map :id (:rules c))))
          (is (= [{:accounts [checking] :from "2025-01-01" :to "2025-12-31"}] (:selects c)))
          (is (= [0.85M :checked] [(:threshold c) (:strategy c)]))
          (is (nil? (:cases c)))))
      (testing "a rule changed by a scenario standing on it keeps its place; a new one comes last"
        (scenario/put! node {:ref "in-place" :name "R2 changed" :base "harbour-coding"
                             :layers [{:id "changes" :kind "coding"
                                       :coding {:rules [{:id "R4" :code "travel" :rule "Rideshare is travel."
                                                         :when [["contains" "merchant" "ride"]]}
                                                        (assoc (get-in coding [:rules 1]) :rule "Taxis are travel.")]}}]})
        (is (= [["R1" nil] ["R2" "Taxis are travel."] ["R3" nil] ["R4" nil]]
               (map (juxt :id #(when (= "R2" (:id %)) (:rule %))) (:rules (scenario/composed node "in-place"))))))
      (testing "a scenario standing on it amends a project field by field: the trip now pays a per diem"
        (scenario/put! node {:ref "per-diem" :name "Harbour pays a per diem" :base "harbour-coding"
                             :layers [{:id "pays" :kind "world" :world {:projects {"harbour" {:per-diem true}}}}]})
        (let [s (scenario/classifier-scenario node "per-diem")
              f (classify/enrich s (first (:cases s)))]
          (is (= ["Harbour Exhibition" true] ((juxt :name :per-diem) (get-in s [:projects "harbour"]))))
          (is (get-in f [:whereabouts :per-diem]))
          (is (str/includes? (get-in (classify/state s f) [:context :cardholder_was]) "pays a per diem")))
        (is (nil? (let [s (scenario/classifier-scenario node "harbour-coding")]
                    (get-in (classify/enrich s (first (:cases s))) [:whereabouts :per-diem])))
            "the scenario under it is unchanged")
        (is (not (str/includes? (let [s (scenario/classifier-scenario node "harbour-coding")]
                                  (get-in (classify/state s (classify/enrich s (first (:cases s)))) [:context :cardholder_was]))
                                "per diem"))
            "a world that sets no per diem on its projects says nothing of it to Jev")
        (is (= :ownpurse/invalid
               (:type (error #(scenario/put! node {:ref "nowhere" :name "Amends nothing" :base "harbour-coding"
                                                   :layers [{:id "w" :kind "world"
                                                             :world {:projects {"atlantis" {:per-diem true}}}}]})))))
        (is (= :ownpurse/invalid
               (:type (error #(scenario/put! node {:ref "nowhere" :name "Not a yes or no" :base "harbour-coding"
                                                   :layers [{:id "w" :kind "world"
                                                             :world {:projects {"harbour" {:per-diem "yes"}}}}]}))))))
      (testing "a rule that codes to no bucket, or a trip with no dates, is refused"
        (is (= :ownpurse/invalid
               (:type (error #(scenario/put! node (assoc-in trial [:layers 1 :coding :threshold] 1.5M))))))
        (is (= :ownpurse/scenario-conflict (:type (error #(scenario/create! node trial)))))
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
        (testing "the estimate asks nothing, and counts every request as one to ask before any answer is kept"
          (is (= {:units 4 :requests 9 :asked 9 :kept 0 :timed false}
                 (select-keys (scenario/estimate node "harbour-coding" "stub" {}) [:units :requests :asked :kept :timed])))
          (is (= 2 (:units (scenario/estimate node "harbour-coding" "stub" {:limit 2}))))
          (is (zero? @calls)))
        (testing "a run codes every line: sure codings stand, the rest go to a person"
          (is (= {:units 4 :coded 2 :asked 9 :kept 0}
                 (select-keys (scenario/code! node "harbour-coding" (stub calls) {:model "stub"})
                              [:units :coded :asked :kept])))
          (is (= [["line:test:1" "per-diem" true] ["line:test:2" "needs-person" false] ["line:test:3" "personal" true]
                  ["line:test:5" "needs-person" false]]
                 (map (juxt :unit :code :auto) (scenario/codings node "harbour-coding"))))
          (is (= "travel" (:proposed (second (scenario/codings node "harbour-coding"))))))
        (testing "after the run, the estimate finds every answer kept: nothing to ask, nothing to pay, and timed"
          (is (= {:asked 0 :kept 9 :input-tokens 0 :cost-usd 0M :timed true}
                 (-> (scenario/estimate node "harbour-coding" "stub" {})
                     (select-keys [:asked :kept :input-tokens :cost-usd :timed])
                     (update :cost-usd #(.stripTrailingZeros ^BigDecimal %))))))
        (testing "the run says how long it took and how many tokens it sent"
          (is (= {:input-tokens 90} (select-keys (first (scenario/coding-runs node "harbour-coding")) [:input-tokens])))
          (is (nat-int? (:took-ms (first (scenario/coding-runs node "harbour-coding"))))))
        (testing "the sandbox shows the record scenario as it shows an invented one: one run, \"latest\""
          (is (= [["harbour-coding" true 4 ["latest"]]]
                 (map (juxt :name :record :units #(map :id (:runs %))) (scenario/sandbox-scenarios node))))
          (is (true? (:record (scenario/sandbox-world node "harbour-coding"))))
          (let [{:keys [run lines]} (scenario/sandbox-run node "harbour-coding")]
            (is (= ["latest" 4] [(:id run) (:results run)]))
            (is (= ["L1" "L2" "L3" "L4"] (map :short lines)))
            (is (= :per-diem (get-in (first lines) [:decisions :checked :code])))))
        (testing "one line all the way through: the requests rebuilt from its facts find the answers the record kept"
          (let [{:keys [responses explained record unanswered]} (scenario/sandbox-line node "harbour-coding" "line:test:1")]
            (is record)
            (is (= [] unanswered))
            (is (= #{:rules :plain} (set (keys responses))))
            (is (= ["R1" "R2" "R3"] (map :id explained)))))
        (testing "a line a person keys is scored against it; a scenario standing on it can key it otherwise"
          (scenario/confirm! node "harbour-coding" {:unit "line:test:1" :expect "per-diem"})
          (scenario/confirm! node "harbour-coding" {:unit "line:test:2" :expect "travel" :review "allowed"})
          (is (= {"line:test:1" [:per-diem "harbour-coding"] "line:test:2" [:travel "harbour-coding"]}
                 (update-vals (scenario/answer-key node "harbour-coding") (juxt :expect :in))))
          (is (= [:per-diem "harbour-coding"]
                 ((juxt :expect :keyed-in) (:unit (first (:lines (scenario/sandbox-run node "harbour-coding")))))))
          (scenario/put! node {:ref "child-key" :name "Keyed otherwise" :base "harbour-coding" :layers []})
          (testing "the lines a scenario selects are one set, read by it and every scenario standing on it"
            (is (= [{:id "harbour-coding~lines" :kind "books" :lines 4 :scenarios ["harbour-coding" "child-key"]}]
                   (map #(select-keys % [:id :kind :lines :scenarios]) (scenario/line-sets node))))
            (let [{:keys [first peek from]} (scenario/line-set node "harbour-coding~lines")]
              (is (= 4 (count first)))
              (is (= 3 (count peek)))
              (is (= from (:date (clojure.core/first first)))))
            (is (= :ownpurse/not-found (:type (error #(scenario/line-set node "nowhere/lines"))))))
          (scenario/confirm! node "child-key" {:unit "line:test:1" :expect "personal"})
          (is (= [:personal "child-key"] ((juxt :expect :in) (get (scenario/answer-key node "child-key") "line:test:1"))))
          (is (= :per-diem (:expect (get (scenario/answer-key node "harbour-coding") "line:test:1"))) "the base keeps its own")
          (scenario/confirm! node "child-key" {:unit "line:test:1" :remove true})
          (is (= "harbour-coding" (:in (get (scenario/answer-key node "child-key") "line:test:1"))) "removed: the base's stands")
          (is (= :ownpurse/invalid (:type (error #(scenario/confirm! node "harbour-coding" {:unit "line:test:3" :expect "nowhere"})))))
          (scenario/drop! node "child-key"))
        (testing "a line traced step by step: each step's call, what came out, and the queries it made"
          (let [{:keys [steps]} (scenario/sandbox-trace node "harbour-coding" "line:test:1")]
            (is (= (range 1 11) (map :step steps)))
            (is (= ["SELECT * FROM scenarios WHERE _id = ?" 4]
                   [(:sql (first (:db (first steps)))) (:rows (second (:db (first steps))))]))
            (is (= "line:test:1" (get-in (first steps) [:db 1 :row :xt/id])) "the row shown is the line's")
            (is (= [1 1] (map :rows (:db (nth steps 6)))) "both answers read from the record")
            (is (= "per-diem" (name (:code (:out (nth steps 9))))))))
        (testing "a rule changed since the run: the line still opens, and says which request has no answer kept"
          (scenario/put! node {:ref "reworded" :name "R1 reworded" :base "harbour-coding"
                               :layers [{:id "changes" :kind "coding"
                                         :coding {:rules [(assoc (get-in coding [:rules 0]) :rule "Food on the trip is per diem.")]}}]})
          (scenario/code! node "reworded" (stub (atom 0)) {:model "stub" :limit 1})
          (scenario/put! node {:ref "reworded" :name "R1 reworded again" :base "harbour-coding"
                               :layers [{:id "changes" :kind "coding"
                                         :coding {:rules [(assoc (get-in coding [:rules 0]) :rule "Groceries away are per diem.")]}}]})
          (is (= [:rules] (:unanswered (scenario/sandbox-line node "reworded" "line:test:1"))))
          (scenario/drop! node "reworded"))
        (testing "a line whose yes/no answers are not kept still shows its rules: the rule asking one holds nothing known"
          (let [asking #(scenario/put! node {:ref "asked" :name "A question" :base "harbour-coding"
                                             :layers [{:id "q" :kind "coding"
                                                       :coding {:rules [{:id "R5" :code "travel" :rule "Rides are travel."
                                                                         :when [["jev" %]]}]}}]})]
            (asking "Is this a ride?")
            (scenario/code! node "asked" (stub (atom 0)) {:model "stub" :limit 1})
            (asking "Is this a ride somewhere?")
            (let [{:keys [unanswered explained]} (scenario/sandbox-line node "asked" "line:test:1")]
              (is (some #{:plain} unanswered))
              (is (= [nil] (map :holds (filter (comp #{"R5"} :id) explained)))))
            (scenario/drop! node "asked")))
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
        (testing "a rule added asks again only the rules requests of the lines it could apply to: one rideshare"
          (scenario/put! node {:ref "rule-more" :name "One rule more" :base "harbour-coding"
                               :layers [{:id "more" :kind "coding"
                                         :coding {:rules [{:id "R4" :code "travel" :rule "Rideshare is travel."
                                                           :when [["contains" "merchant" "ride"]]}]}}]})
          (is (= {:asked 1 :kept 8} (select-keys (scenario/code! node "rule-more" (stub calls) {:model "stub"})
                                                 [:asked :kept]))))
        (testing "one unit all the way through"
          (is (= :harbour (keyword (get-in (scenario/coding node "harbour-coding" "line:test:1")
                                           [:facts :whereabouts :project]))))
          (is (= 2 (count (scenario/coding-runs node "harbour-coding")))))))))

(deftest over-a-set-test
  (with-node
    (fn [node]
      (scenario/put! node trial)
      (let [calls (atom 0)
            code! #(scenario/code! node "harbour-coding" (stub calls) (cond-> {:model "stub"} % (assoc :over %)))]
        (testing "a scenario's lines layer is the set it reads by default; a run over it is the scenario's own"
          (is (= "harbour-coding~lines" (:default-set (first (scenario/sandbox-scenarios node)))))
          (code! (scenario/set-over node "harbour-coding~lines"))
          (is (= 1 (count (scenario/coding-runs node "harbour-coding"))))
          (is (= 4 (count (scenario/codings node "harbour-coding")))))
        (testing "a saved set chooses lines by account and days, one line, or every book's"
          (scenario/line-set-put! node {:name "november" :title "November"
                                        :select {:accounts [checking] :from "2025-11-01" :to "2025-11-30"}})
          (scenario/line-set-put! node {:name "one-line" :title "The groceries in Lisbon"
                                        :select {:lines ["line:test:1"]}})
          (scenario/line-set-put! node {:name "everything" :title "Every book's lines" :select {:all true}})
          (is (= {:lines 2} (select-keys (scenario/plan node "harbour-coding" (scenario/set-over node "november")) [:lines])))
          (is (= 1 (:lines (scenario/plan node "harbour-coding" (scenario/set-over node "one-line")))))
          (is (= 5 (:lines (scenario/plan node "harbour-coding" (scenario/set-over node "everything")))))
          (is (= ["everything" "november" "one-line"] (keep #(when (:saved %) (:id %)) (scenario/line-sets node)))
              "saved sets, by name")
          (is (= :ownpurse/invalid (:type (error #(scenario/line-set-put! node {:name "No Caps" :title "x" :select {:all true}})))))
          (is (= :ownpurse/invalid (:type (error #(scenario/line-set-put! node {:name "empty" :title "x" :select {}}))))))
        (testing "the same scenario over another set is a run of its own, kept and scored apart"
          (scenario/line-set-confirm! node "november" {:unit "line:test:3" :expect "travel"})
          (let [over (scenario/set-over node "november")]
            (code! over)
            (is (= 1 (count (scenario/coding-runs node "harbour-coding" over))))
            (is (= 1 (count (scenario/coding-runs node "harbour-coding"))) "its own lines' runs are as they were")
            (is (= 4 (count (scenario/codings node "harbour-coding"))))
            (let [{:keys [run lines]} (scenario/sandbox-run node "harbour-coding" over)]
              (is (= ["november" "november" 2] [(:id run) (:over run) (:results run)]))
              (is (= [:travel "november"] ((juxt :expect :keyed-in) (:unit (second lines))))
                  "the set's own key scores it"))
            (is (= ["november" "latest"] (map :id (:runs (first (scenario/sandbox-scenarios node))))))
            (is (= "line:test:3" (:id (scenario/sandbox-line node "harbour-coding" over "line:test:3"))))
            (is (= 10 (count (:steps (scenario/sandbox-trace node "harbour-coding" over "line:test:3")))))))
        (testing "the scenario's own key stands over the set's: a policy that differs"
          (scenario/confirm! node "harbour-coding" {:unit "line:test:3" :expect "personal"})
          (is (= [:personal "harbour-coding"]
                 ((juxt :expect :keyed-in)
                  (:unit (second (:lines (scenario/sandbox-run node "harbour-coding" (scenario/set-over node "november")))))))))
        (testing "lines given as they are (a sample's) carry their key with them"
          (let [over {:id "sample~tiny"
                      :cases [{:id "c1" :date "2025-10-05" :amount 10M :currency "EUR" :merchant "Mercado Azul"
                               :card "card-r" :mcc "5411" :expect :per-diem}]}]
            (code! over)
            (is (= :per-diem (get-in (first (:lines (scenario/sandbox-run node "harbour-coding" over))) [:unit :expect])))))))))

(deftest rule-ledger-test
  (with-node
    (fn [node]
      (scenario/put! node trial)
      (scenario/put! node {:ref "child" :name "Changes" :base "harbour-coding"
                           :layers [{:id "changes" :kind "coding"
                                     :coding {:rules [{:id "R2" :code "travel" :enabled false}
                                                      {:id "R3" :code "personal" :rule "Groceries at home are mine."
                                                       :when [["at-home"] ["category" "in" ["5411"]]]}]}}]})
      (let [ledger (scenario/rule-ledger node "child")]
        (testing "every rule in its place, those turned off kept there with where they were turned off"
          (is (= [["R1" true] ["R2" false] ["R3" true]] (map (juxt :id :on) ledger)))
          (is (= {:scenario "child" :layer "changes"} (:off-in (second ledger))))
          (is (= "Taxis on a project trip are travel." (get-in (second ledger) [:rule :rule])) "its last definition"))
        (testing "where each standing definition comes from, and what changed it"
          (is (= {:scenario "harbour-coding" :layer "coding"} (:from (first ledger))))
          (is (= {:scenario "child" :layer "changes"} (:from (nth ledger 2))))
          (is (= [{:scenario "child" :layer "changes"}] (:changed-in (nth ledger 2)))))
        (testing "the ledger's rules that are on are the composed scenario's"
          (is (= (map :id (:rules (scenario/composed node "child"))) (map :id (filter :on ledger)))))))))
