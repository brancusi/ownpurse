;; # Walking one bank line through the classifier
;;
;; One bank line, in three parts: **A**, the raw line as the bank gives it; **B**, how code turns it into facts;
;; **C**, how it is classified: the context and rules, the exact payload sent to Jev, what Jev answers, and the coding.
;; Every step is the code that ran and what it returned.
;;
;; Pick a case of the eval to replay its saved answers (no calls to Jev), or change any field and press
;; **Classify** to ask Jev live about your own line. Every name, number and place here is invented.
(ns ownpurse.notebooks.walk
  {:nextjournal.clerk/no-cache true
   :nextjournal.clerk/auto-expand-results? true}
  (:require
   [clojure.java.io :as io]
   [clojure.string :as str]
   [jsonista.core :as j]
   [nextjournal.clerk :as clerk]
   [ownpurse.classify.core :as core]
   [ownpurse.classify.facts :as facts]
   [ownpurse.classify.interface :as classify]
   [ownpurse.classify.jev :as jev]
   [ownpurse.config.interface :as config]))

^{:nextjournal.clerk/visibility {:code :hide :result :hide}}
(def root
  "This repository's folder, from where the notebook is on the classpath."
  (-> (io/resource "ownpurse/notebooks/walk.clj") io/file
      .getParentFile .getParentFile .getParentFile .getParentFile .getParentFile))

^{:nextjournal.clerk/visibility {:code :hide :result :hide}}
(def run-file
  (->> (.listFiles (io/file root "components/classify/runs"))
       (filter #(str/ends-with? (.getName ^java.io.File %) ".edn"))
       (sort-by #(.getName ^java.io.File %))
       last))

^{:nextjournal.clerk/visibility {:code :hide :result :hide}}
(def run (classify/load-run run-file))

^{:nextjournal.clerk/visibility {:code :hide :result :hide}}
(def scenario (classify/scenario "harbour"))

^{:nextjournal.clerk/visibility {:code :hide :result :hide}}
(def cases (into (sorted-map) (map (juxt :id identity)) (:cases scenario)))

;; What the page shows: a case to replay, or a line of your own (its fields as typed). The form below changes it in
;; the REPL and shows the page again (Clerk's synced atoms need its cache, which the notebooks keep off).
^{:nextjournal.clerk/visibility {:code :hide :result :hide}}
(defonce !pick (atom {:case "c34"}))

^{:nextjournal.clerk/visibility {:code :hide :result :hide}}
(defn pick!
  "Shows `pick` ({:case id} or {:fields {...}}): what the form calls."
  [pick]
  (reset! !pick pick)
  (future ((requiring-resolve 'ownpurse.notebook/reshow!)))
  nil)

^{:nextjournal.clerk/visibility {:code :hide :result :hide}}
(defn- as-fields
  "A line as the form's fields: every value a string."
  [line]
  (into {} (for [k [:date :card :merchant :mcc :amount :currency :country :channel :note :expect]]
             [k (let [v (get line k)]
                  (cond (nil? v) "" (keyword? v) (name v) (decimal? v) (.toPlainString ^BigDecimal v) :else (str v)))])))

^{:nextjournal.clerk/visibility {:code :hide :result :hide}}
(defn- from-fields
  "The form's fields as a line."
  [fields]
  (let [blank->nil #(not-empty (str/trim (str %)))]
    (cond-> {:id "yours" :date (str/trim (:date fields)) :card (:card fields) :merchant (str/trim (:merchant fields))
             :amount (bigdec (str/trim (:amount fields))) :currency (str/trim (:currency fields))
             :channel (keyword (:channel fields))}
      (blank->nil (:mcc fields)) (assoc :mcc (:mcc fields))
      (blank->nil (:country fields)) (assoc :country (str/trim (:country fields)))
      (blank->nil (:note fields)) (assoc :note (str/trim (:note fields)))
      (blank->nil (:expect fields)) (assoc :expect (keyword (:expect fields))))))

^{:nextjournal.clerk/visibility {:code :hide :result :hide}}
(def form-viewer
  {:transform-fn clerk/mark-presented
   :render-fn
   '(fn [{:keys [cases cards categories codes fields picked]} _]
      (let [input-style {:border "1px solid #d1d5db" :border-radius 4 :padding "2px 6px" :background "white"}
            label (fn [text el] [:label {:style {:display "inline-block" :margin "0 12px 10px 0" :font-size "0.85em"
                                                 :vertical-align "top"}}
                                 [:div {:style {:color "#6b7280"}} text] el])
            form (fn [_]
                   (let [draft (nextjournal.clerk.render.hooks/use-state fields)
                         text (fn [k text width]
                                (label text [:input {:value (get @draft k) :style (assoc input-style :width width)
                                                     :on-change #(swap! draft assoc k (.. % -target -value))}]))
                         choose (fn [k text options]
                                  (label text (into [:select {:value (get @draft k) :style input-style
                                                              :on-change #(swap! draft assoc k (.. % -target -value))}]
                                                    (for [[v t] options] [:option {:value v} t]))))]
                     [:div {:style {:border "1px solid #e5e7eb" :border-radius 8 :padding "12px 14px" :margin "8px 0"
                                    :background "#fafafa"}}
                      [:div {:style {:margin-bottom 10}}
                       (label "Replay a case of the eval"
                              (into [:select {:value (or picked "") :style input-style
                                              :on-change #(nextjournal.clerk.render/clerk-eval
                                                (list 'ownpurse.notebooks.walk/pick! {:case (.. % -target -value)}))}
                                     [:option {:value ""} "— your own line —"]]
                                    (for [[id t] cases] [:option {:value id} (str id ": " t)])))]
                      [:div
                       (text :date "Date" 100) (choose :card "Card" cards) (text :merchant "Merchant" 200)
                       (choose :mcc "Category (merchant code)" (cons ["" "none: the bank gave none"] categories))]
                      [:div
                       (text :amount "Amount" 80) (text :currency "Currency" 50) (text :country "Shop's country" 120)
                       (choose :channel "How" [["in-store" "in person, in a shop"] ["online" "online"]
                                               ["transfer" "a transfer"]])]
                      [:div (text :note "Note on the line" 420)
                       (choose :expect "What it should be (to score it)" (cons ["" "—"] codes))]
                      [:button {:style {:background "#4f46e5" :color "white" :border-radius 6 :padding "4px 14px"
                                        :font-weight 600}
                                :on-click #(nextjournal.clerk.render/clerk-eval (list 'ownpurse.notebooks.walk/pick! {:fields @draft}))}
                       "Classify (asks Jev)"]
                      [:span {:style {:font-size "0.8em" :color "#6b7280" :margin-left 10}}
                       "Three calls at most; the same line is never asked twice."]]))]
        ^{:key (pr-str [picked fields])} [form fields]))})

;; ## Choose a line

^{:nextjournal.clerk/visibility {:code :hide :result :hide}}
(def pick @!pick)

^{:nextjournal.clerk/visibility {:code :hide :result :hide}}
(def saved (when-let [id (:case pick)] (first (filter #(= id (:id %)) (:results run)))))

^{:nextjournal.clerk/visibility {:code :hide}}
(clerk/with-viewer form-viewer
  {:cases (vec (for [[id {:keys [merchant]}] cases] [id merchant]))
   :cards (vec (for [[id {:keys [name]}] (:cards scenario)] [id name]))
   :categories (vec (for [[mcc n] (sort-by key (:categories scenario))] [mcc (str mcc " " n)]))
   :codes (vec (for [[c _] (:codes scenario)] [(name c) (name c)]))
   :picked (:case pick)
   :fields (or (:fields pick) (as-fields (cases (:case pick))))})

^{:nextjournal.clerk/visibility {:code :hide :result :hide}}
(defn- pct [p] (str (.setScale (.multiply (bigdec p) 100M) 0 java.math.RoundingMode/HALF_UP) "%"))

^{:nextjournal.clerk/visibility {:code :hide :result :hide}}
(defn- json [x]
  (clerk/html
   [:pre {:style {:font-size "0.8em" :background "#f8fafc" :border "1px solid #e5e7eb" :border-radius 6
                  :padding 12 :white-space "pre-wrap" :color "#1f2937"}}
    (j/write-value-as-string x (j/object-mapper {:pretty true}))]))

^{:nextjournal.clerk/visibility {:code :hide :result :hide}}
(defn- callout [& body]
  (clerk/html (into [:div {:style {:border-left "4px solid #4f46e5" :background "#eef2ff" :padding "8px 12px"
                                   :border-radius 4 :margin "6px 0"}}]
                    body)))

;; # A. The raw transaction
;;
;; ## 1. The bank line, as it arrives
;;
;; This is everything the bank gives: a date, which card, a merchant and its category code, an amount, the shop's
;; country, how it was paid, and sometimes a note. It says nothing about who was where, or about any project.

^{:nextjournal.clerk/visibility {:result :hide}}
(def line
  (if-let [fields (:fields pick)]
    (from-fields fields)
    (cases (:case pick))))

^{:nextjournal.clerk/visibility {:result :hide}}
(def raw (select-keys line [:date :card :merchant :mcc :amount :currency :country :channel :note]))

^{:nextjournal.clerk/visibility {:code :hide}}
(json raw)

;; The answer key. A case of the eval also says how a bookkeeper would code it. That is only for scoring: it is never
;; part of the line and never reaches Jev.

^{:nextjournal.clerk/visibility {:code :hide}}
(let [{:keys [expect ok why]} line]
  (if expect
    (callout [:b "A bookkeeper would code it: " (name expect)]
             (when (seq ok) (str " (also acceptable: " (str/join ", " (map name ok)) ")"))
             (when why [:div [:i why]]))
    (callout "No answer key: choose what it should be in the form to score it.")))

;; # B. Turning it into facts
;;
;; Code, not a model, works out what the line means here, from what the workspace keeps beside the bank: who holds
;; each card, who was where on which days, which projects run when, and what each category code means.

;; ## 2. Who held the card

(def card (get-in scenario [:cards (:card line)]))

;; ## 3. Where they were that day
;;
;; The holder's trips, then the one that covers the line's date. Plain date arithmetic, done in code: a model is never
;; asked whether a date falls in a range.

(def trips (filterv #(= (:holder card) (:person %)) (:trips scenario)))

(def whereabouts (facts/whereabouts scenario (:holder card) (:date line)))

;; ## 4. Which projects were running

(def projects (facts/running-projects scenario (:date line)))

;; ## 5. What kind of merchant
;;
;; The bank's category code, looked up in the table of codes:

(def category-name (get-in scenario [:categories (:mcc line)]))

;; When the bank gave no code (and the line is not a transfer), Jev is asked first: the rules need a category. This is
;; that payload, and its answer; both are nil when the bank gave a code.

(def enrich-request (core/enrichment scenario line))

^{:nextjournal.clerk/visibility {:code :hide :result :hide}}
(defonce ^:private !asked (atom {}))

^{:nextjournal.clerk/visibility {:code :hide :result :hide}}
(def jev-config (:jev (config/read-config :dev)))

;; Every question goes to Jev through `ask`. A case of the eval replays the run's saved answers, so walking it costs
;; nothing; a line of your own asks Jev live, and keeps each answer so the same payload is never sent twice.

^{:nextjournal.clerk/visibility {:result :hide}}
(defn ask [state questions]
  (if saved
    (get-in saved [:responses (cond (contains? questions :category) :enrich
                                    (contains? questions :reversed) :rules
                                    :else :plain)])
    (or (@!asked [state questions])
        (let [response (jev/ask jev-config state questions)]
          (swap! !asked assoc [state questions] response)
          response))))

^{:nextjournal.clerk/visibility {:code :hide}}
(callout (if saved
           [:span "Replaying " [:b (:id saved)] " from the run " [:code (.getName ^java.io.File run-file)]
            ", answered by " [:code (:model run)] "."]
           [:span "A line of your own: Jev (" [:code (:model jev-config)] ") is asked live."]))

(def enrich-response (when enrich-request (ask (:state enrich-request) (:questions enrich-request))))

;; Code takes the chosen category only when Jev is at least 50% confident of it.

(def category (some->> enrich-response :answers :category (core/enriched-category scenario)))

;; ## 6. Before and after
;;
;; Steps 2 to 5 in one call, `facts/enrich`:

(def line-facts (facts/enrich scenario line category))

;; Side by side: what the bank said, what code added, and from what.

^{:nextjournal.clerk/visibility {:code :hide}}
(clerk/html
 (let [{:keys [place trip project day days]} (:whereabouts line-facts)
       row (fn [field value from] [:tr [:td [:code field]] [:td value] [:td {:style {:color "#6b7280"}} from]])]
   [:table {:style {:font-size "0.9em"}}
    [:thead [:tr [:th "Fact"] [:th "Value"] [:th "Where it came from"]]]
    [:tbody
     (for [[k v] raw] (row (name k) (str v) "the bank"))
     (row "holder" (get-in scenario [:entities (:holder line-facts) :name]) "the cards: who holds this card")
     (row "card entity" (get-in scenario [:entities (:card-entity line-facts) :name]) "the cards: whose card it is")
     (row "whereabouts" (case trip
                          :project (str place ", project trip for " (get-in scenario [:projects project :name])
                                        ", day " day " of " days)
                          :personal (str place ", a personal trip")
                          (str "at home, " place))
          "the trips, by the line's date")
     (row "projects running" (str/join ", " (map #(get-in scenario [:projects % :name]) (:projects line-facts)))
          "the projects, by the line's date")
     (row "category" (or (:category line-facts) "none")
          (if (= :enrichment (:category-from line-facts)) "Jev, from the merchant's name (step 5)" "the category codes"))
     (row "country mismatch" (str (:country-mismatch line-facts))
          "in a shop, in a country other than where the holder was")]]))

;; ## 7. The facts, as Jev will read them
;;
;; The facts in words, as named fields: `transaction` (the line) and `context` (what code worked out). Dates reach Jev
;; only as conclusions ("on a project trip, day 24 of 57"), never as ranges to compare.

^{:nextjournal.clerk/visibility {:result :hide}}
(def state (facts/state scenario line-facts))

^{:nextjournal.clerk/visibility {:code :hide}}
(json state)

;; # C. Classifying it
;;
;; ## 8. The context and the rules
;;
;; The codes a line can take, each with what it means (Jev reads these as the options of its Choice):

^{:nextjournal.clerk/visibility {:code :hide}}
(clerk/html
 [:table {:style {:font-size "0.85em"}}
  [:thead [:tr [:th "Code"] [:th "Means"]]]
  [:tbody (for [[c d] (:codes scenario)] [:tr [:td [:b (name c)]] [:td d]])]])

;; The rules: each a sentence a person said, and the same rule as building blocks. Most blocks are exact and code
;; checks them; a `:jev` block is a yes/no question Jev answers about the line. A rule that stops a line for a person
;; wins over every other.

^{:nextjournal.clerk/visibility {:code :hide}}
(clerk/html
 [:table {:style {:font-size "0.85em"}}
  [:thead [:tr [:th "Rule"] [:th "Says"] [:th "Codes"] [:th "Blocks"]]]
  [:tbody (for [{:keys [id rule code check] :as r} (:rules scenario)]
            [:tr [:td [:b id]] [:td rule] [:td (if check "stops for a person" (name code))]
             [:td [:code {:style {:font-size "0.85em"}} (str/join " " (map pr-str (:when r)))]]])]])

;; Two payloads are built from the facts, and sent at once:
;;
;; - **with the rules**: the state above plus every rule's text. It asks one Choice, "how should this line be coded?"
;;   (and the same Choice with its options reversed, to catch a lean to the first option), and one yes/no per rule,
;;   "does this rule apply?".
;; - **plain**: the state alone. It asks the same Choice without the rules (to see what the rules add), and each fuzzy
;;   block's question.

(def requests (core/requests scenario line-facts))

;; ## 9. The payload sent to Jev, with the rules
;;
;; Exactly as posted to `/v1/systemone` (the API key goes in a header, not here). `state` is what Jev reads;
;; `questions` is what it answers, each by its id.

^{:nextjournal.clerk/visibility {:code :hide}}
(json (assoc (:rules requests) :model (:model jev-config)))

(def rules-response (ask (get-in requests [:rules :state]) (get-in requests [:rules :questions])))

;; What Jev returned, by question id:

^{:nextjournal.clerk/visibility {:code :hide}}
(json (:answers rules-response))

;; ## 10. The payload sent to Jev, without the rules

^{:nextjournal.clerk/visibility {:code :hide}}
(json (assoc (:plain requests) :model (:model jev-config)))

(def plain-response (ask (get-in requests [:plain :state]) (get-in requests [:plain :questions])))

^{:nextjournal.clerk/visibility {:code :hide}}
(json (:answers plain-response))

;; ## 11. Reading Jev's Choice
;;
;; A Choice gives a probability for every option. Its confidence says how concentrated they are: 100% when one option
;; has it all, 0% when they are spread evenly.

^{:nextjournal.clerk/visibility {:code :hide :result :hide}}
(defn- bars [{:keys [probabilities choice confidence]}]
  [:div {:style {:display "inline-block" :vertical-align "top" :margin-right 28}}
   [:div "chose " [:b (str/replace (str choice) "_" " ")] ", confidence " [:b (pct confidence)]]
   [:table {:style {:font-size "0.85em"}}
    [:tbody (for [[option p] (sort-by val > probabilities)]
              [:tr [:td (str/replace option "_" " ")]
               [:td [:div {:style {:width 140 :height 10 :background "#e5e7eb" :border-radius 2}}
                     [:div {:style {:width (pct p) :height "100%" :background "#4f46e5" :border-radius 2}}]]]
               [:td (pct p)]])]]])

^{:nextjournal.clerk/visibility {:code :hide}}
(clerk/html [:div
             [:div {:style {:display "inline-block"}} [:h4 "With the rules"] (bars (get-in rules-response [:answers :code]))]
             [:div {:style {:display "inline-block"}} [:h4 "Without them"] (bars (get-in plain-response [:answers :code]))]])

;; ## 12. The building blocks, with Jev's answers
;;
;; Each rule holds as far as its weakest block. A rule applies when it holds at least 50%. Beside it, Jev's own
;; yes/no on the whole rule (step 9).

(def explained (core/explain scenario {:facts line-facts
                                       :responses {:rules rules-response :plain plain-response}}))

^{:nextjournal.clerk/visibility {:code :hide}}
(clerk/html
 (let [held (fn [p]
              (let [p (bigdec p)]
                [:span {:style (merge {:padding "0 4px" :border-radius 3}
                                      (cond (= 1M p) {:background "#dcfce7"}
                                            (zero? p) {:color "#9ca3af"}
                                            :else {:background "#fef3c7"}))}
                 (pct p)]))]
   [:table {:style {:font-size "0.85em"}}
    [:thead [:tr [:th "Rule"] [:th "Codes"] [:th "Each block"] [:th "The rule"] [:th "Jev on the rule"]]]
    [:tbody
     (for [{:keys [id code check blocks holds jev]} explained
           :let [applies? (>= holds 0.5M)]]
       [:tr {:style (when applies? {:background "#f5f3ff"})}
        [:td [:b id]] [:td (if check "stop" (name code))]
        [:td (interpose [:br] (for [{:keys [block holds]} blocks]
                                [:span (held holds) " " [:code {:style {:font-size "0.9em"}} (pr-str block)]]))]
        [:td (held holds) (when applies? [:b " applies"])]
        [:td (held jev)]])]]))

;; ## 13. The coding
;;
;; The same answers, combined six ways. Where rules are combined (judged by Jev, or by their blocks), a check rule
;; stops the line, the rules that apply must agree, and the confidence is the least certain rule's.

(def decisions (core/decide scenario line-facts {:rules-answers (:answers rules-response)
                                                 :plain-answers (:answers plain-response)}))

^{:nextjournal.clerk/visibility {:code :hide}}
(clerk/html
 (let [{:keys [expect ok]} line
       style {:right {:background "#dcfce7" :color "#14532d"} :wrong {:background "#fee2e2" :color "#7f1d1d"}
              :stopped {:background "#e0e7ff" :color "#312e81"} :review {:background "#fef3c7" :color "#78350f"}}
       words {:right "coded automatically, right" :wrong "coded automatically, WRONG"
              :stopped "sent to a person, rightly" :review "sent to a person, needlessly"}
       names {:choice "Choice, with rules" :reversed "Same, options reversed" :no-rules "Choice, no rules"
              :rule-nouls "Jev judges each rule" :blocks "Building blocks" :hybrid "Blocks, Choice as fallback"}]
   [:div
    (if expect
      [:p "A bookkeeper would code it " [:b (name expect)]
       (when (seq ok) (str " (also acceptable: " (str/join ", " (map name ok)) ")")) ". Scored at 70% confidence."]
      [:p "Choose what it should be in the form to score it."])
    [:table
     [:thead [:tr [:th "Strategy"] [:th "Code"] [:th "Confidence"] [:th "Why"] [:th "At 70%"]]]
     [:tbody
      (for [s classify/strategies
            :let [{:keys [code confidence reason fallback by] :as d} (decisions s)
                  o (when expect (classify/outcome line d 0.7M))]]
        [:tr [:td (names s)] [:td [:b (name code)]] [:td (pct confidence)]
         [:td (str (some-> (or fallback reason) name) (when (seq by) (str " " (str/join ", " by))))]
         [:td (when o [:span {:style (merge {:padding "1px 6px" :border-radius 4} (style o))} (words o)])]])]]]))
