;; # Bank lines into buckets
;;
;; Every line of an eval scenario as a card: the raw bank line, the bucket it belongs in, and the bucket Jev chose with
;; its confidence. Open a card's "What happened" to see the calls: each payload sent to Jev and what came back. The
;; buckets come first, by project, with what landed in each. Every name, number and place here is invented.
;;
;; Params: `:scenario` ("atlas" when absent), `:run` (a run's file; the scenario's latest when absent), `:threshold`
;; (0.7 when absent): `(show :cards {:scenario "harbour"})`.
(ns ownpurse.notebooks.cards
  {:nextjournal.clerk/no-cache true
   :nextjournal.clerk/visibility {:code :hide :result :hide}}
  (:require
   [clojure.java.io :as io]
   [clojure.string :as str]
   [jsonista.core :as j]
   [nextjournal.clerk :as clerk]
   [ownpurse.classify.core :as core]
   [ownpurse.classify.interface :as classify]
   [ownpurse.notebook :as notebook]))

(def root
  "This repository's folder, from where the notebook is on the classpath."
  (-> (io/resource "ownpurse/notebooks/cards.clj") io/file
      .getParentFile .getParentFile .getParentFile .getParentFile .getParentFile))

(def params (notebook/params))

(def scenario-name (or (:scenario params) "atlas"))

(def run-file
  (or (some-> (:run params) io/file)
      (->> (.listFiles (io/file root "components/classify/runs"))
           (filter #(let [n (.getName ^java.io.File %)]
                      (and (str/starts-with? n (str scenario-name "-")) (str/ends-with? n ".edn"))))
           (sort-by #(.getName ^java.io.File %))
           last)))

(def run (classify/load-run run-file))

(def scenario (classify/scenario scenario-name))

(def threshold (bigdec (or (:threshold params) 0.7M)))

(def units (into {} (map (juxt :id identity)) (mapcat classify/units (:cases scenario))))

(def lines (into {} (map (juxt :id identity)) (:cases scenario)))

(def results (:results run))

(def groups
  "How the buckets are laid out: the scenario's :buckets, or, without them, its codes as one group."
  (or (:buckets scenario)
      [{:title "Codes" :codes (for [[c _] (:codes scenario)] [c (str/capitalize (str/replace (name c) "-" " "))])}]))

(def palette ["#2563eb" "#d97706" "#059669" "#7c3aed" "#db2777" "#0891b2"])

(def group-of
  (into {} (for [[i {:keys [project title codes]}] (map-indexed vector groups)
                 [code label] codes]
             [code {:group (or (get-in scenario [:projects project :name]) title)
                    :label label
                    :colour (if (= :needs-person code) "#6b7280" (nth palette (mod i (count palette))))}])))

(defn- pct [p] (str (.setScale (.multiply (bigdec p) 100M) 0 java.math.RoundingMode/HALF_UP) "%"))

(defn- money [{:keys [amount currency]}] (str currency " " (.toPlainString ^BigDecimal amount)))

(defn- bucket-chip [code]
  (let [{:keys [group label colour]} (group-of code {:group "?" :label (name code) :colour "#6b7280"})]
    [:span {:style {:display "inline-block" :border (str "1px solid " colour) :border-left (str "6px solid " colour)
                    :border-radius 6 :padding "2px 8px" :white-space "nowrap" :background "white"}}
     [:span {:style {:color colour :font-size "0.75em" :font-weight 600}} group] " "
     [:b label]]))

(def outcome-style
  {:right {:background "#dcfce7" :color "#14532d"}
   :wrong {:background "#fee2e2" :color "#7f1d1d" :font-weight 700}
   :stopped {:background "#e0e7ff" :color "#312e81"}
   :review {:background "#fef3c7" :color "#78350f"}})

(def outcome-words
  {:right "coded, right" :wrong "coded, WRONG" :stopped "to a person, rightly" :review "to a person, needlessly"})

(defn- pill [o]
  [:span {:style (merge {:padding "1px 8px" :border-radius 4 :font-size "0.8em" :white-space "nowrap"}
                        (outcome-style o))}
   (outcome-words o)])

(defn- outcome [{:keys [id decisions]}]
  (classify/outcome (units id) (:choice decisions) threshold))

(defn- json [x]
  [:pre {:style {:font-size "0.75em" :background "#f8fafc" :border "1px solid #e5e7eb" :border-radius 6 :padding 10
                 :white-space "pre-wrap" :color "#1f2937" :margin "4px 0 10px"}}
   (j/write-value-as-string x (j/object-mapper {:pretty true}))])

;; ## The projects

^{:nextjournal.clerk/visibility {:result :show}}
(clerk/html
 {:nextjournal.clerk/width :wide}
 [:div
  [:div {:style {:display "flex" :gap 12 :flex-wrap "wrap"}}
   (for [[id {:keys [name place from to work travellers]}] (:projects scenario)]
     [:div {:style {:border "1px solid #e5e7eb" :border-radius 8 :padding "10px 14px" :flex "1 1 260px"}}
      [:div [:b name]] [:div {:style {:color "#6b7280" :font-size "0.9em"}} place ", " from " to " to]
      (when work [:div {:style {:font-size "0.9em"}} work])
      [:div {:style {:font-size "0.85em" :margin-top 4}}
       "Travellers: " (str/join ", " (map #(get-in scenario [:entities % :name]) travellers))]
      [:div {:style {:font-size "0.85em"}}
       "Trips: " (str/join "; " (for [{:keys [person city place from to project]} (:trips scenario) :when (= id project)]
                                  (str (get-in scenario [:entities person :name]) " in " (or city place) ", "
                                       from " to " to)))]])]
  [:p {:style {:font-size "0.9em" :color "#6b7280"}}
   "Any other day, everyone is at home"
   (let [home (:home scenario)] (if (map? home) (str ", in " (:city home) ", " (:country home)) (str ", in " home)))
   ". Cards: " (str/join "; " (for [[_ {:keys [name holder]}] (:cards scenario)]
                               (str name " (held by " (get-in scenario [:entities holder :name]) ")")))]])

;; ## The buckets
;;
;; Each bucket with what it means, how many lines belong in it, and where Jev's Choice (given the facts and the rules)
;; put them. A chip is a line Jev put there: green when right, red when wrong, grey when it went to a person below the
;; confidence threshold.

^{:nextjournal.clerk/visibility {:result :show}}
(clerk/html
 {:nextjournal.clerk/width :wide}
 (let [expected (frequencies (map #(:expect (units (:id %))) results))
       placed (group-by #(get-in % [:decisions :choice :code]) results)]
   [:div {:style {:display "flex" :flex-direction "column" :gap 14}}
    [:p "Run " [:code (.getName ^java.io.File run-file)] " of " [:b scenario-name] " by " [:code (:model run)] ", "
     (count results) " lines and items. Automatic at " (pct threshold) " confidence."]
    (for [{:keys [project title codes]} groups
          :let [group (or (get-in scenario [:projects project :name]) title)
                colour (:colour (group-of (ffirst codes)))]]
      [:div
       [:div {:style {:font-weight 700 :color colour :margin-bottom 4}} group]
       [:div {:style {:display "flex" :gap 10 :flex-wrap "wrap"}}
        (for [[code label] codes]
          [:div {:style {:border (str "1px solid " colour) :border-top (str "5px solid " colour) :border-radius 8
                         :padding "8px 10px" :flex "1 1 220px" :background "#fff"}}
           [:div [:b label]]
           [:div {:style {:font-size "0.78em" :color "#6b7280" :min-height 34}} (get-in scenario [:codes code])]
           [:div {:style {:font-size "0.85em" :margin "4px 0"}}
            (get expected code 0) " belong here · Jev put " (count (placed code))]
           [:div {:style {:display "flex" :gap 4 :flex-wrap "wrap"}}
            (for [{:keys [id] :as r} (sort-by :id (placed code))
                  :let [o (outcome r)]]
              [:a {:href (str "#" id)
                   :style (merge {:font-size "0.75em" :padding "0 5px" :border-radius 3 :text-decoration "none"}
                                 (case o
                                   :right {:background "#dcfce7" :color "#14532d"}
                                   :wrong {:background "#fee2e2" :color "#7f1d1d" :font-weight 700}
                                   {:background "#f3f4f6" :color "#4b5563"}))}
               id])]])]])]))

;; ## The hard parts
;;
;; Lines coded automatically and wrongly (the silent errors), then lines sent to a person that did not need one, then
;; every line the scenario marks as hard.

^{:nextjournal.clerk/visibility {:result :show}}
(clerk/html
 {:nextjournal.clerk/width :wide}
 (let [by (group-by outcome results)
       row (fn [{:keys [id decisions]}]
             (let [u (units id) {:keys [code confidence]} (:choice decisions)]
               [:tr [:td [:a {:href (str "#" id)} id]] [:td (:merchant u) (when (:item u) [:i " · " (:item u)])]
                [:td (bucket-chip (:expect u))] [:td (bucket-chip code)] [:td (pct confidence)]
                [:td {:style {:font-size "0.85em"}} (or (:hard u) (:why u))]]))
       table (fn [title rows]
               [:div [:h4 title " (" (count rows) ")"]
                (if (seq rows)
                  [:table {:style {:font-size "0.85em"}}
                   [:thead [:tr [:th "Line"] [:th "Merchant"] [:th "Belongs in"] [:th "Jev chose"] [:th "Confidence"]
                            [:th "Why it is hard"]]]
                   [:tbody (map row rows)]]
                  [:p {:style {:color "#6b7280"}} "None."])])]
   [:div
    (table "Coded automatically, and wrongly" (by :wrong))
    (table "Sent to a person needlessly" (by :review))
    (table "Marked hard" (filter #(:hard (units (:id %))) results))]))

;; ## Every line

(defn- raw-view [u]
  (let [line (lines (or (:line u) (:id u)))
        raw (for [k [:date :card :merchant :mcc :amount :currency :country :city :channel :note]
                  :let [v (get line k)] :when (some? v)]
              [k (if (keyword? v) (name v) v)])]
    [:div
     [:table {:style {:font-size "0.82em" :margin 0}}
      [:tbody (for [[k v] raw]
                [:tr [:td {:style {:color "#6b7280" :padding "1px 10px 1px 0"}} (name k)]
                 [:td {:style {:padding "1px 0"}} (if (decimal? v) (.toPlainString ^BigDecimal v) (str v))]])]]
     (when-let [items (:items line)]
       [:div {:style {:font-size "0.82em" :margin-top 6}}
        [:div {:style {:color "#6b7280"}} (str "The " (name (:items-from line)) ", scanned apart from the bank line:")]
        [:ul {:style {:margin "2px 0"}}
         (for [{:keys [item amount]} items]
           [:li {:style (when (= item (:item u)) {:font-weight 700})}
            item ", " (.toPlainString ^BigDecimal amount) (when (= item (:item u)) "  ← this card")])]])]))

(defn- choice-bars [{:keys [probabilities]}]
  [:table {:style {:font-size "0.78em" :margin "4px 0"}}
   [:tbody (for [[option p] (take 4 (sort-by val > probabilities))]
             [:tr [:td {:style {:padding "0 8px 0 0"}} (str/replace option "_" " ")]
              [:td [:div {:style {:width 120 :height 8 :background "#e5e7eb" :border-radius 2}}
                    [:div {:style {:width (pct p) :height "100%" :background "#4f46e5" :border-radius 2}}]]]
              [:td {:style {:padding-left 6}} (pct p)]])]])

(defn- calls-view [{:keys [facts responses]}]
  (let [requests (core/requests scenario facts)
        enrich (core/enrichment scenario (units (:id facts)))]
    [:details {:style {:margin-top 8}}
     [:summary {:style {:cursor "pointer" :color "#4f46e5"}} "What happened: the calls to Jev"]
     [:div {:style {:font-size "0.85em" :margin-top 6}}
      [:p "Code first works out the facts (" [:code "facts/enrich"] "), then builds the payloads ("
       [:code "core/requests"] ") and posts each to " [:code "/v1/systemone"] ". These are the payloads and the answers "
       "as they were in the run."]
      [:div [:b "The facts code worked out"]]
      (json (dissoc facts :id :line))
      (when enrich
        [:div [:b "First: which kind of merchant (the bank gave no category)"]
         (json (assoc enrich :model (:model run)))
         (json (get-in responses [:enrich :answers]))])
      [:div [:b "Call 1: the payload with the rules"]]
      (json (assoc (:rules requests) :model (:model run)))
      [:div [:b "Call 1: what came back"]]
      (json (get-in responses [:rules :answers]))
      [:div [:b "Call 2: the payload without the rules"]]
      (json (assoc (:plain requests) :model (:model run)))
      [:div [:b "Call 2: what came back"]]
      (json (get-in responses [:plain :answers]))]]))

(defn- where-text [{:keys [whereabouts]}]
  (let [{:keys [place city trip project day days]} whereabouts
        at (if city (str city ", " place) place)]
    (case trip
      :project (str "in " at ", on a trip for " (get-in scenario [:projects project :name]) ", day " day " of " days)
      :personal (str "in " at ", on a personal trip")
      (str "at home in " at))))

(defn- card [{:keys [id facts decisions responses] :as result}]
  (let [u (units id)
        {:keys [code confidence]} (:choice decisions)
        o (outcome result)
        blocks (:blocks decisions)]
    [:div {:id id :style {:border "1px solid #e5e7eb" :border-left (str "6px solid " (:color (outcome-style o)))
                          :border-radius 8 :padding "10px 14px" :margin "12px 0" :background "#fff"}}
     [:div {:style {:display "flex" :justify-content "space-between" :gap 8 :flex-wrap "wrap"}}
      [:div [:b id] " · " (:date u) " · " (:merchant u) (when (:item u) [:span " · " [:i (:item u)]]) " · " (money u)]
      (pill o)]
     [:div {:style {:display "flex" :gap 18 :flex-wrap "wrap" :margin-top 8}}
      [:div {:style {:flex "1 1 280px"}}
       [:div {:style {:font-size "0.75em" :color "#6b7280" :text-transform "uppercase"}} "The raw bank line"]
       (raw-view u)]
      [:div {:style {:flex "1 1 320px"}}
       [:div {:style {:font-size "0.75em" :color "#6b7280" :text-transform "uppercase"}} "Belongs in"]
       [:div (bucket-chip (:expect u))
        (when (seq (:ok u)) [:span {:style {:font-size "0.8em" :color "#6b7280"}}
                             " also fine: " (str/join ", " (map #(:label (group-of % {:label (name %)})) (:ok u)))])]
       [:div {:style {:font-size "0.75em" :color "#6b7280" :text-transform "uppercase" :margin-top 8}}
        "Jev chose, with the rules"]
       [:div (bucket-chip code) " " [:b (pct confidence)] " confident"]
       (choice-bars (get-in responses [:rules :answers :code]))
       [:div {:style {:font-size "0.82em" :color "#4b5563"}}
        "Code placed " (get-in scenario [:entities (:holder facts) :name]) " " (where-text facts)
        (when (:country-mismatch facts) [:b "; the shop is not where they were recorded to be"]) "."]
       [:div {:style {:font-size "0.82em" :color "#4b5563"}}
        "The building blocks said " [:b (:label (group-of (:code blocks) {:label (name (:code blocks))}))]
        (when-let [by (seq (:by blocks))] (str " (" (str/join ", " by) ")"))
        (when (:reason blocks) (str ", " (name (:reason blocks))))
        (when (not= code (:code blocks)) [:span {:style {:color "#b45309"}} ": not what Jev chose"]) "."]]]
     (when-let [why (or (:hard u) (:why u))]
       [:div {:style {:font-size "0.85em" :margin-top 6}} [:i (if (:hard u) "Hard: " "Why: ") why]])
     (calls-view result)]))

^{:nextjournal.clerk/visibility {:result :show}}
(clerk/html {:nextjournal.clerk/width :wide} [:div (map card results)])
