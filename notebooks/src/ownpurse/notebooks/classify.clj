;; # Classifying by context: the eval, case by case
;;
;; A saved run of the classification eval: every case of the scenario, what code computed about it, what Jev was
;; asked and answered, and how each strategy coded it. Nothing here asks Jev: the run holds every raw answer, so a
;; different threshold is only a different reading of it. Every name, number and place here is invented.
;;
;; Params: `:run` (a run's file; the latest in components/classify/runs when absent), `:threshold` (the confidence
;; a coding needs to be automatic; 0.7 when absent), `:cases` (case ids to show; all when absent):
;; `(show :classify {:threshold 0.9M :cases ["c12" "c25"]})`.
(ns ownpurse.notebooks.classify
  {:nextjournal.clerk/no-cache true
   :nextjournal.clerk/visibility {:code :hide}}
  (:require
   [clojure.java.io :as io]
   [clojure.pprint :as pprint]
   [clojure.string :as str]
   [nextjournal.clerk :as clerk]
   [ownpurse.classify.interface :as classify]
   [ownpurse.notebook :as notebook]))

^{:nextjournal.clerk/visibility {:result :hide}}
(def root
  "This repository's folder, from where the notebook is on the classpath."
  (-> (io/resource "ownpurse/notebooks/classify.clj") io/file
      .getParentFile .getParentFile .getParentFile .getParentFile .getParentFile))

^{:nextjournal.clerk/visibility {:result :hide}}
(def params (notebook/params))

^{:nextjournal.clerk/visibility {:result :hide}}
(def run-file
  (or (some-> (:run params) io/file)
      (->> (.listFiles (io/file root "components/classify/runs"))
           (filter #(let [n (.getName ^java.io.File %)] (and (str/ends-with? n ".edn") (not (str/ends-with? n ".notes.edn")))))
           (sort-by #(.getName ^java.io.File %))
           last)))

^{:nextjournal.clerk/visibility {:result :hide}}
(def run (classify/load-run run-file))

^{:nextjournal.clerk/visibility {:result :hide}}
(def scenario (classify/scenario (:scenario run)))

^{:nextjournal.clerk/visibility {:result :hide}}
(def threshold (bigdec (or (:threshold params) 0.7M)))

^{:nextjournal.clerk/visibility {:result :hide}}
(def cases (into {} (map (juxt :id identity)) (mapcat classify/units (:cases scenario))))

^{:nextjournal.clerk/visibility {:result :hide}}
(def strategy-names
  {:choice "Choice, with rules" :reversed "Same, options reversed" :no-rules "Choice, no rules"
   :rule-nouls "Jev judges each rule" :blocks "Building blocks" :hybrid "Blocks, Choice as fallback"})

^{:nextjournal.clerk/visibility {:result :hide}}
(def outcome-style
  {:right {:background "#dcfce7" :color "#14532d"}
   :wrong {:background "#fee2e2" :color "#7f1d1d" :font-weight 700}
   :stopped {:background "#e0e7ff" :color "#312e81"}
   :review {:background "#fef3c7" :color "#78350f"}})

^{:nextjournal.clerk/visibility {:result :hide}}
(def outcome-words
  {:right "coded, right" :wrong "coded, WRONG" :stopped "to a person, rightly" :review "to a person, needlessly"})

^{:nextjournal.clerk/visibility {:result :hide}}
(defn- code-name [code] (some-> code name (str/replace "-" " ")))

^{:nextjournal.clerk/visibility {:result :hide}}
(defn- pct [p] (when p (str (.setScale (* 100M (bigdec p)) 0 java.math.RoundingMode/HALF_UP) "%")))

^{:nextjournal.clerk/visibility {:result :hide}}
(defn- pill [outcome text]
  [:span {:style (merge {:padding "1px 6px" :border-radius 4 :white-space "nowrap" :font-size "0.85em"}
                        (outcome-style outcome))}
   text])

^{:nextjournal.clerk/visibility {:result :hide}}
(defn- coding-pill [case coding]
  (let [o (classify/outcome case coding threshold)]
    (pill o (str (code-name (:code coding)) " " (pct (:confidence coding))))))

^{:nextjournal.clerk/visibility {:result :hide}}
(defn- bar [p]
  [:div {:style {:display "inline-block" :width 120 :height 10 :background "#e5e7eb" :border-radius 2
                 :vertical-align "middle" :margin-right 6}}
   [:div {:style {:width (pct p) :height "100%" :background "#4f46e5" :border-radius 2}}]])

^{:nextjournal.clerk/visibility {:result :hide}}
(defn- money [{:keys [amount currency]}] (str currency " " (.toPlainString ^BigDecimal amount)))

;; ## The run

(clerk/html
 [:div
  [:p "Run " [:code (.getName ^java.io.File run-file)] " of the " [:b (:scenario run)] " scenario, by "
   [:code (:model run)] ": " (get-in run [:usage :requests]) " requests, "
   (get-in run [:usage :input-tokens]) " tokens in, " (get-in run [:usage :output-tokens]) " out."]
  [:p "A coding is automatic when it is not " [:i "needs person"] " and its confidence is at least "
   [:b (pct threshold)] ". Each is then one of: "
   (interpose " " (for [o [:right :wrong :stopped :review]] (pill o (outcome-words o))))
   ". A wrong automatic coding is the silent error that matters most."]])

;; ## How each strategy did

(let [{:keys [strategies choice-stable]} (classify/score scenario run)]
  (clerk/html
   [:div
    [:table
     [:thead [:tr [:th "Strategy"] [:th "Its answer acceptable"]
              (for [t classify/thresholds] [:th {:style {:text-align "right"}} (str "At " (pct t))])]]
     [:tbody
      (for [s classify/strategies
            :let [{:keys [accurate by-threshold]} (strategies s)]]
        [:tr [:td (strategy-names s)] [:td (str accurate " of " (count (:results run)))]
         (for [t classify/thresholds
               :let [{:keys [right wrong stopped review]} (by-threshold t)]]
           [:td {:style {:text-align "right" :white-space "nowrap"}}
            (pill :right right) " " (pill :wrong wrong) " " (pill :stopped stopped) " " (pill :review review)])])]]
    [:p "The Choice kept its answer with its options reversed on " choice-stable " of " (count (:results run))
     " cases."]]))

;; ## The scenario
;;
;; What code knows before anything is judged: the project, who was where on which days, and the codes a line can
;; take.

(clerk/html
 [:div
  (for [[_ {:keys [name from to place travellers]}] (:projects scenario)]
    [:p [:b name] ", in " place ", from " from " to " to "; its travellers are "
     (str/join " and " (map #(get-in scenario [:entities % :name]) travellers)) "."])
  [:table
   [:thead [:tr [:th "Who"] [:th "Where"] [:th "From"] [:th "To"] [:th "Why"]]]
   [:tbody (for [{:keys [person place from to project]} (:trips scenario)]
             [:tr [:td (get-in scenario [:entities person :name])] [:td place] [:td from] [:td to]
              [:td (if project (get-in scenario [:projects project :name]) "a personal trip")]])]]
  [:p "On any other day each person is at home, in " (:home scenario) "."]
  [:table
   [:thead [:tr [:th "Card"] [:th "Held by"]]]
   [:tbody (for [[_ {:keys [name holder]}] (:cards scenario)]
             [:tr [:td name] [:td (get-in scenario [:entities holder :name])]])]]
  [:table
   [:thead [:tr [:th "Code"] [:th "Means"]]]
   [:tbody (for [[c d] (:codes scenario)] [:tr [:td [:b (code-name c)]] [:td d]])]]])

;; ## The rules
;;
;; Each rule is one sentence a person said, and the same rule as building blocks, all of which must hold. A check
;; rule only ever stops a line for a person, and wins over every other rule. A `:jev` block is a yes/no question Jev
;; answers about the line.

(clerk/html
 [:table
  [:thead [:tr [:th "Rule"] [:th "Says"] [:th "Codes"] [:th "Blocks"]]]
  [:tbody (for [{:keys [id rule code check] :as r} (:rules scenario)]
            [:tr [:td [:b id]] [:td rule] [:td (if check "stops for a person" (code-name code))]
             [:td [:code {:style {:font-size "0.8em"}} (str/join " " (map pr-str (:when r)))]]])]])

;; ## Every case
;;
;; One row a case: what the bookkeeper would code it, and each strategy's coding with its confidence. The id leads
;; to the case's walk-through below.

^{:nextjournal.clerk/visibility {:result :hide}}
(def shown
  (let [ids (some-> (:cases params) set)]
    (filterv #(or (nil? ids) (ids (:id %))) (:results run))))

(clerk/html
 [:table {:style {:font-size "0.9em"}}
  [:thead [:tr [:th "Case"] [:th "Line"] [:th "Expected"]
           (for [s classify/strategies] [:th (strategy-names s)])]]
  [:tbody
   (for [{:keys [id decisions]} shown
         :let [{:keys [date merchant expect ok] :as c} (cases id)]]
     [:tr [:td [:a {:href (str "#" id)} id]]
      [:td date " · " merchant (when (:item c) [:span " · " [:i (:item c)]]) " · " (money c)]
      [:td [:b (code-name expect)] (when (seq ok) (str " (or " (str/join ", " (map code-name ok)) ")"))]
      (for [s classify/strategies] [:td (coding-pill c (decisions s))])])]])

;; ## Walking through each case
;;
;; For each case: the line; what code computed about it; what Jev was shown (exactly as sent); Jev's Choice and how
;; its probability spread over the codes; and rule by rule, which blocks held and what Jev made of the whole rule.

^{:nextjournal.clerk/visibility {:result :hide}}
(defn- facts-view [{:keys [card-name whereabouts projects country-mismatch category category-from mcc]}]
  (let [{:keys [place trip project day days]} whereabouts]
    [:ul
     [:li "Paid with " card-name "."]
     [:li "The cardholder was "
      (case trip
        :project [:span "in " [:b place] ", on a project trip for " (get-in scenario [:projects project :name])
                  " (day " day " of " days ")"]
        :personal [:span "in " [:b place] ", on a personal trip"]
        [:span [:b "at home"] ", in " place])
      "."]
     [:li (if (seq projects)
            (str "Running that day: " (str/join ", " (map #(get-in scenario [:projects % :name]) projects)) ".")
            "No project was running that day.")]
     [:li (if category
            [:span "Category " [:b category] " (" mcc ")"
             (when category-from [:i ", found by enrichment: the bank gave none"]) "."]
            "No category: the bank gave none and enrichment found none.")]
     (when country-mismatch [:li [:b "The shop is not where the cardholder was recorded to be."]])]))

^{:nextjournal.clerk/visibility {:result :hide}}
(defn- choice-view [title {:keys [probabilities choice confidence]}]
  [:div {:style {:margin-right 24}}
   [:div [:b title] ": " (str/replace (str choice) "_" " ") ", confidence " (pct confidence)]
   [:table {:style {:font-size "0.85em"}}
    [:tbody (for [[option p] (sort-by val > probabilities)]
              [:tr [:td (str/replace option "_" " ")] [:td (bar p) (pct p)]])]]])

^{:nextjournal.clerk/visibility {:result :hide}}
(defn- held [p]
  (let [p (bigdec p)]
    [:span {:style (merge {:padding "0 4px" :border-radius 3}
                          (cond (= 1M p) {:background "#dcfce7"}
                                (zero? p) {:color "#9ca3af"}
                                :else {:background "#fef3c7"}))}
     (pct p)]))

^{:nextjournal.clerk/visibility {:result :hide}}
(defn- rules-view [result]
  [:table {:style {:font-size "0.85em"}}
   [:thead [:tr [:th "Rule"] [:th "Codes"] [:th "Blocks: how far each held"] [:th "By blocks"] [:th "Jev's view"]]]
   [:tbody
    (for [{:keys [id code check blocks holds jev]} (classify/explain scenario result)
          :let [applies? (>= holds 0.5M)]]
      [:tr {:style (when applies? {:background "#f5f3ff"})}
       [:td [:b id]] [:td (if check "stop" (code-name code))]
       [:td (interpose [:br] (for [{:keys [block holds]} blocks]
                               [:span (held holds) " " [:code {:style {:font-size "0.9em"}} (pr-str block)]]))]
       [:td (held holds) (when applies? [:b " applies"])]
       [:td (held jev)]])]])

^{:nextjournal.clerk/visibility {:result :hide}}
(defn- case-view [{:keys [id facts responses decisions] :as result}]
  (let [{:keys [date merchant note channel expect ok why] :as c} (cases id)
        plain-answers (get-in responses [:plain :answers])
        fuzzy (filter #(str/starts-with? (name (key %)) "fuzzy_") plain-answers)]
    [:section {:id id :style {:border-top "1px solid #d1d5db" :padding-top 12 :margin-top 24}}
     [:h3 id ": " merchant (when (:item c) (str " · " (:item c))) ", " (money c)]
     [:p date " · " (name channel) (when note [:span " · note: “" note "”"])]
     [:p "Expected: " [:b (code-name expect)]
      (when (seq ok) (str " (also acceptable: " (str/join ", " (map code-name ok)) ")"))
      (when why [:span ". " [:i why]])]
     [:div {:style {:display "flex" :flex-wrap "wrap" :gap 6 :margin "8px 0"}}
      (for [s classify/strategies
            :let [{:keys [reason fallback by] :as coding} (decisions s)]]
        [:div {:style {:border "1px solid #e5e7eb" :border-radius 6 :padding "4px 8px"}}
         [:div {:style {:font-size "0.8em" :color "#6b7280"}} (strategy-names s)]
         (coding-pill c coding)
         (when (or reason fallback)
           [:div {:style {:font-size "0.8em"}}
            (str (some-> (or fallback reason) name) (when (seq by) (str ": " (str/join ", " by))))])])]
     [:h4 "What code knew"]
     (facts-view facts)
     (when-let [enrich (get-in responses [:enrich :answers :category])]
       [:div [:h4 "Enrichment"] (choice-view "Which kind of merchant" enrich)])
     [:h4 "What Jev answered"]
     [:div {:style {:display "flex" :flex-wrap "wrap"}}
      (choice-view "Coding, with the rules" (get-in responses [:rules :answers :code]))
      (choice-view "Coding, no rules" (:code plain-answers))]
     (when (seq fuzzy)
       [:div [:b "The fuzzy blocks' questions"]
        [:table {:style {:font-size "0.85em"}}
         [:tbody (for [[k {:keys [noul]}] (sort-by key fuzzy)]
                   [:tr [:td (nth (classify/fuzzy-questions (:rules scenario)) (parse-long (subs (name k) 6)))]
                    [:td (bar noul) (pct noul)]])]]])
     [:h4 "Rule by rule"]
     (rules-view result)
     [:details [:summary "What Jev was shown"]
      [:pre {:style {:font-size "0.8em" :white-space "pre-wrap"}}
       (with-out-str (pprint/pprint (classify/state scenario facts)))]]]))

(clerk/html [:div (map case-view shown)])
