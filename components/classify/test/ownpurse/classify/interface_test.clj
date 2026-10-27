(ns ownpurse.classify.interface-test
  (:require
   [clojure.string :as str]
   [clojure.test :refer [deftest is testing]]
   [ownpurse.classify.blocks :as blocks]
   [ownpurse.classify.core :as core]
   [ownpurse.classify.interface :as classify]))

(def ^:private s (classify/scenario "harbour"))

(defn- line [id] (first (filter #(= id (:id %)) (:cases s))))

(def ^:private fuzzy-yes
  "The fuzzy blocks a careful person would answer yes, by merchant (and note, where it decides)."
  {"Was this flight or lodging booked for a trip to the project's place?" #{"Atlantic Wing Airways"}
   "Is this purchase materials, equipment, venue hire, shipping or event catering for the project, rather than something for a person's own use?"
   #{"plywood and fixings for the installation" "shipping crates" "venue deposit" "catering for the opening reception"}
   "Is this a software subscription or online tool used for a creative studio's work?" #{"Sketchloom" "Layerworks"}})

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
                                                    (or (yes (:merchant transaction)) (yes (:note transaction)))))
                                             fuzzy-yes)
                                     1M 0M)})]))}))

(deftest scenario-test
  (testing "the scenario holds together: every code, category, card and id is known"
    (is (= 41 (count (:cases s))))
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
      (is (= :no-rule (:reason (core/combine rules {"a" 0M "b" 0M "c" 0M})))))))

(deftest requests-test
  (let [{:keys [rules plain]} (core/requests s (classify/enrich s (line "c01")))]
    (testing "one Noul per rule, beside the rules; the fuzzy blocks apart, with the facts alone"
      (is (= 10 (count (filter #(str/starts-with? (name %) "rule_") (keys (:questions rules))))))
      (is (= 3 (count (filter #(str/starts-with? (name %) "fuzzy_") (keys (:questions plain))))))
      (is (contains? (:state rules) :rules))
      (is (not (contains? (:state plain) :rules))))
    (testing "the reversed Choice has the same options, last first"
      (is (= (reverse (keys (get-in rules [:questions :code :criteria])))
             (keys (get-in rules [:questions :reversed :criteria])))))))

(deftest eval-test
  (let [run (classify/run-eval s (stub-ask "needs_person"))
        score (classify/score s run)]
    (testing "with a careful person's fuzzy answers, the building blocks code every case acceptably but the two traps
      written for them: an airline on a project trip booked for a holiday, a restaurant on a project trip for a meeting"
      (is (= 39 (get-in score [:strategies :blocks :accurate])))
      (is (= [["c34" :project-cost] ["c40" :per-diem]] (get-in score [:strategies :blocks :wrong 0M]))))
    (testing "a Choice that always says needs_person codes nothing automatically, so it is never silently wrong"
      (is (= 0 (get-in score [:strategies :choice :by-threshold 0M :right])
             (get-in score [:strategies :choice :by-threshold 0M :wrong]))))
    (testing "the run keeps every raw response and counts its requests: two per line, one more to enrich"
      (is (= 84 (get-in run [:usage :requests])))
      (is (= :enrichment (get-in (first (filter #(= "c31" (:id %)) (:results run))) [:facts :category-from]))))))
