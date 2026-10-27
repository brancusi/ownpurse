(ns ownpurse.classify.interface-test
  (:require
   [clojure.string :as str]
   [clojure.test :refer [deftest is testing]]
   [ownpurse.classify.blocks :as blocks]
   [ownpurse.classify.core :as core]
   [ownpurse.classify.interface :as classify]))

(def ^:private s (classify/scenario "harbour"))

(defn- line [id] (first (filter #(= id (:id %)) (concat (:cases s) (mapcat classify/units (:cases s))))))

(def ^:private fabrication-items
  "The items a careful person would call materials, tools or consumables for the sculptures."
  #{"Plaster of Paris, 25 kg" "Sandpaper assortment, 120 to 400 grit" "Masonry drill bit set" "Casting plaster, 10 kg"
    "Paper towels, 12 rolls" "Plaster bandage rolls" "Mixing bucket, 10 litres" "Rubber gloves"
    "Nitrile gloves, box of 100" "Gypsum cement, 50 lb" "Mixing paddle" "Plaster of Paris, 5 lb" "Sandpaper, 240 grit"})

(def ^:private fuzzy-yes
  "The fuzzy blocks a careful person would answer yes, by merchant (and note, where it decides)."
  {"Was this flight or lodging booked for a trip to the project's place?" #{"Atlantic Wing Airways"}
   "Is this purchase materials, equipment, venue hire, shipping or event catering for the project, rather than something for a person's own use?"
   (into #{"plywood and fixings for the installation" "shipping crates" "venue deposit" "catering for the opening reception"}
         fabrication-items)
   "Is this a software subscription or online tool used for a creative studio's work?" #{"Sketchloom" "Layerworks"}
   "Is this item a material, tool or consumable used in fabricating plaster sculptures?" fabrication-items})

(defn- stub-ask
  "A stand-in for Jev: every Choice answers `choice` (or, for a merchant's category, the grocery one), and every fuzzy
  block is answered as a careful person would."
  [choice]
  (fn [{:keys [transaction]} questions]
    {:model "stub"
     :usage {:input_tokens 10 :output_tokens 1}
     :answers (into {}
                    (for [[id {:keys [type instructions]}] questions]
                      [id (if (= "choice" type)
                            {:type "choice" :confidence 1M
                             :choice (if (= :category id) "Grocery stores and supermarkets" choice)}
                            {:type "noul"
                             :noul (if (some (fn [[q yes]]
                                               (and (str/starts-with? instructions q)
                                                    (if-let [item (get-in transaction [:item :name])]
                                                      (yes item)
                                                      (or (yes (:merchant transaction)) (yes (:note transaction))))))
                                             fuzzy-yes)
                                     1M 0M)})]))}))

(deftest scenario-test
  (testing "the scenario holds together: every code, category, card and id is known"
    (is (= 51 (count (:cases s))))
    (is (every? :when (:rules s)))))

(deftest enrich-test
  (testing "where the cardholder was comes from the trips, by day, both ends included"
    (is (= {:place "Portugal" :trip :project :project "harbour" :day 41 :days 41}
           (:whereabouts (classify/enrich s (line "c18")))))
    (is (= {:place "United States" :trip nil} (:whereabouts (classify/enrich s (line "c03")))))
    (is (= :personal (get-in (classify/enrich s (line "c04")) [:whereabouts :trip]))))
  (testing "a purchase in a shop away from where the cardholder was is flagged"
    (is (:country-mismatch (classify/enrich s (line "c19"))))
    (is (not (:country-mismatch (classify/enrich s (line "c26"))))
        "an online order is placed from anywhere"))
  (testing "a line with no merchant code waits for enrichment"
    (is (nil? (:category (classify/enrich s (line "c31")))))
    (is (some? (core/enrichment s (line "c31"))))
    (is (nil? (core/enrichment s (line "c27"))) "a transfer has no merchant to judge")))

(deftest units-test
  (testing "a receipt's items are each a unit: the line narrowed to the item, with the rest of the receipt beside it"
    (let [[plaster sandpaper :as units] (classify/units (line "c42"))]
      (is (= ["c42/1" "c42/2" "c42/3"] (map :id units)))
      (is (= [38.90M 72.70M "c42"] ((juxt :amount :line-amount :line) plaster)))
      (is (= :project-cost (:expect sandpaper)))
      (is (= ["Plaster of Paris, 25 kg" "Masonry drill bit set"] (:other-items sandpaper)))))
  (testing "a line without items is its own unit"
    (is (= [(line "c01")] (classify/units (line "c01")))))
  (testing "Jev reads the item apart from the line, with what the project makes"
    (let [{:keys [transaction context]} (classify/state s (classify/enrich s (line "c44/4")))]
      (is (= "EUR 22.30" (:amount transaction)))
      (is (= {:name "Paper towels, 4 rolls" :amount "EUR 3.90" :listed_on "the scanned receipt"
              :other_items_on_it ["Bread" "Cheese" "Red wine"]}
             (:item transaction)))
      (is (str/includes? (first (:projects_running context)) "plaster sculptures"))
      (is (:cardholder_travels_for_a_running_project context)))))

(deftest attribute-test
  (testing "a project cost on a personal card was paid on behalf of the entity that bears the project's costs"
    (is (= {:bears "tallowmere" :paid-by "robin" :on-behalf true}
           (classify/attribute s (classify/enrich s (line "c42/1")) :project-cost))))
  (testing "on the studio's own card it was not"
    (is (= {:bears "tallowmere" :paid-by "tallowmere" :on-behalf false}
           (classify/attribute s (classify/enrich s (line "c51/1")) :project-cost))))
  (testing "per diem and personal spend are the cardholder's; a line sent to a person is no one's yet"
    (is (= "sol" (:bears (classify/attribute s (classify/enrich s (line "c02")) :per-diem))))
    (is (nil? (classify/attribute s (classify/enrich s (line "c15")) :needs-person)))))

(deftest two-projects-test
  (let [atlas (classify/scenario "atlas")
        unit (fn [id] (first (filter #(= id (:id %)) (mapcat classify/units (:cases atlas)))))
        facts #(classify/enrich atlas (unit %))]
    (testing "the same card, a month apart, on trips for different projects"
      (is (= ["harbour" "skyline"] (map #(get-in (facts %) [:whereabouts :project]) ["d01" "d02"])))
      (is (= 1M (blocks/holds [:on-trip-for "skyline"] (facts "d02") {})))
      (is (= 0M (blocks/holds [:on-trip-for "harbour"] (facts "d02") {}))))
    (testing "a shop in another city of the same country is where the cardholder was not"
      (is (:country-mismatch (facts "d15")))
      (is (not (:country-mismatch (facts "d02"))))
      (is (= "at home in Los Angeles, United States"
             (get-in (classify/state atlas (facts "d15")) [:context :cardholder_was]))))
    (testing "a refund is matched in code to the latest earlier purchase on the same card at the same merchant"
      (is (= "d39" (get-in (facts "d60") [:refund-of :id])))
      (is (= "skyline" (get-in (facts "d60") [:refund-of :whereabouts :project])))
      (is (nil? (:refund-of (facts "d73"))) "a credit with no purchase on record has nothing to follow")
      (is (nil? (:refund-of (facts "d39")))))
    (testing "a check that needs a note does not apply to a line without one"
      (is (= 0M (blocks/holds [:present :note] (facts "d02") {})))
      (is (= 1M (blocks/holds [:present :note] (facts "d74") {}))))
    (testing ":checked is the Choice unless a check rule holds by its blocks"
      (let [chosen {:choice "harbour_per_diem" :confidence 0.9M}
            answers (fn [fuzzy-yes?]
                      {:rules-answers (into {:code chosen :reversed chosen}
                                            (map (fn [{:keys [id]}] [(keyword (str "rule_" id)) {:noul 0M}]))
                                            (:rules atlas))
                       :plain-answers (into {:code chosen}
                                            (map-indexed (fn [i _] [(keyword (str "fuzzy_" i))
                                                                    {:noul (if fuzzy-yes? 1M 0M)}]))
                                            (blocks/fuzzy-questions (:rules atlas)))})]
        (is (= {:code :harbour-per-diem :confidence 0.9M}
               (:checked (core/decide atlas (facts "d02") (answers false)))))
        (is (= :harbour-per-diem (:code (:checked (core/decide atlas (facts "d02") (answers true)))))
            "with no note, the note check cannot hold however the model answers")
        (is (= {:code :needs-person :reason :check :by ["A17"] :confidence 1M}
               (:checked (core/decide atlas (facts "d74") (answers true)))))))))

(deftest blocks-test
  (let [f (classify/enrich s (line "c11"))]
    (testing "exact blocks hold or don't"
      (is (= 1M (blocks/holds [:contains :merchant "casa"] f {})))
      (is (= 0M (blocks/holds [:is :merchant "casa"] f {})))
      (is (= 1M (blocks/holds [:amount :between 300.00M 400.00M] f {})))
      (is (= 0M (blocks/holds [:amount :over 500.00M] f {})))
      (is (= 1M (blocks/holds [:trip :project] f {})))
      (is (= 0M (blocks/holds [:at-home] f {}))))
    (testing "a fuzzy block is its answer; :not and :any combine; a rule is its weakest block"
      (is (= 0.25M (blocks/holds [:not [:jev "q"]] f {"q" 0.75M})))
      (is (= 0.75M (blocks/holds [:any [:jev "q"] [:at-home]] f {"q" 0.75M})))
      (is (= 0.75M (blocks/rule-holds {:when [[:trip :project] [:jev "q"]]} f {"q" 0.75M}))))
    (testing "a fuzzy block nobody answered is an error, not a no"
      (is (= :ownpurse/missing-judgment
             (try (blocks/holds [:jev "unasked"] f {}) (catch clojure.lang.ExceptionInfo e (:type (ex-data e)))))))))

(deftest combine-test
  (let [rules [{:id "a" :code :per-diem} {:id "b" :code :project-cost} {:id "c" :code :needs-person :check true}]]
    (testing "the rules that apply must agree; a check stops the line; the least certain rule sets the confidence"
      (is (= {:code :per-diem :by ["a"] :confidence 0.8M} (core/combine rules {"a" 0.9M "b" 0M "c" 0M})))
      (is (= :conflict (:reason (core/combine rules {"a" 1M "b" 1M "c" 0M}))))
      (is (= :check (:reason (core/combine rules {"a" 1M "b" 0M "c" 1M}))))
      (is (= :no-rule (:reason (core/combine rules {"a" 0M "b" 0M "c" 0M}))))))
  (testing "a rule that is the exception to another decides when both apply, and says which it set aside"
    (let [rules [{:id "food" :code :per-diem} {:id "plaster" :code :project-cost :over ["food"]}
                 {:id "holiday" :code :personal}]]
      (is (= {:code :project-cost :by ["plaster"] :overruled ["food"] :confidence 1M}
             (core/combine rules {"food" 1M "plaster" 1M "holiday" 0M})))
      (is (= :conflict (:reason (core/combine rules {"food" 0M "plaster" 1M "holiday" 1M})))
          "an exception to one rule is not an exception to every rule"))))

(deftest requests-test
  (let [{:keys [rules plain]} (core/requests s (classify/enrich s (line "c01")))]
    (testing "one Noul per rule, beside the rules; the fuzzy blocks apart, with the facts alone"
      (is (= 11 (count (filter #(str/starts-with? (name %) "rule_") (keys (:questions rules))))))
      (is (= 4 (count (filter #(str/starts-with? (name %) "fuzzy_") (keys (:questions plain))))))
      (is (contains? (:state rules) :rules))
      (is (not (contains? (:state plain) :rules))))
    (testing "the reversed Choice has the same options, last first"
      (is (= (reverse (keys (get-in rules [:questions :code :criteria])))
             (keys (get-in rules [:questions :reversed :criteria])))))))

(deftest eval-test
  (let [run (classify/run-eval s (stub-ask "needs_person"))
        score (classify/score s run)]
    (testing "with a careful person's fuzzy answers, the building blocks code every unit acceptably but three: the two
      traps written for them (an airline on a project trip booked for a holiday, a restaurant on a project trip for a
      meeting), and paper towels on a grocery receipt, which pass as per diem unseen"
      (is (= 62 (get-in score [:strategies :blocks :accurate])))
      (is (= [["c34" :project-cost] ["c40" :per-diem] ["c44/4" :per-diem]] (get-in score [:strategies :blocks :wrong 0M]))))
    (testing "a Choice that always says needs_person codes nothing automatically, so it is never silently wrong"
      (is (= 0 (get-in score [:strategies :choice :by-threshold 0M :right])
             (get-in score [:strategies :choice :by-threshold 0M :wrong]))))
    (testing "the run keeps every raw response and counts its requests: two per line, one more to enrich"
      (is (= 132 (get-in run [:usage :requests])))
      (is (= :enrichment (get-in (first (filter #(= "c31" (:id %)) (:results run))) [:facts :category-from]))))))
