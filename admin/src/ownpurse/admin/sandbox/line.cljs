(ns ownpurse.admin.sandbox.line
  "One line's way through, in seven steps: the bank line; what code works out; what Jev reads (the state exactly as
  sent, field by field, with how code made each field and where to change it); what Jev is asked (the questions word
  for word, every option as Jev reads it, with its probability); the rules applied; the decision; and where to
  change it. The line's data come from the API when it opens."
  (:require
   [clojure.string :as str]
   [ownpurse.admin.api :as api]
   [ownpurse.admin.route :as route]
   [ownpurse.admin.sandbox.key :as key]
   [ownpurse.admin.ui :as ui :refer [cx]]
   [ownpurse.classify.outcome :as outcome]
   [uix.core :as uix :refer [$ defui]]))

(defn- yes? [p] (>= (or p 0) 0.5))

(def strategies
  "The ways of coding a line the run compared (ownpurse.classify.core), in words."
  [[:choice "Jev, with the rules" "Jev picks the bucket, given the facts and, as text, every rule that could apply to the line (call 1)."]
   [:checked "Jev, stopped by checks" "Jev's pick, unless a check rule holds by its blocks: then a person looks."]
   [:reversed "Jev, options reversed" "The same Choice with its options in reverse order: Jev can lean to the first option."]
   [:no-rules "Jev, without the rules" "Jev picks from the facts alone (call 2): what the rules add."]
   [:rule-nouls "Jev, rule by rule" "Jev says whether each rule applies; code combines them."]
   [:blocks "The rule blocks" "Each rule's building blocks decide; Jev answers only the fuzzy ones."]
   [:hybrid "Blocks, else Jev" "The blocks where one rule decides; where none or several disagree, Jev's pick."]])

(defn- option-code [option] (keyword (str/replace option "_" "-")))

(defn- where-text [{:keys [projects]} {:keys [place city trip project day days]}]
  (let [at (if city (str city ", " place) place)]
    (case trip
      :project (str "in " at ", on a trip for " (get-in projects [project :name]) ", day " day " of " days)
      :personal (str "in " at ", on a personal trip")
      (str "at home in " at))))

(defn- files
  "The files a lever or a field names, by short name: a record scenario's data are its layers, not a file."
  [scenario record]
  {:facts "facts.clj" :core "core.clj" :scenario (if record (str scenario "'s layers") (str scenario ".edn"))})

(defn- field-sources
  "How code made each field of the state Jev reads, and where to change it: [how, where]."
  [scenario record]
  (let [{:keys [facts scenario]} (files scenario record)]
    {[:transaction :merchant] ["the bank line's merchant, as the bank sends it" "the bank's data"]
     [:transaction :amount] ["the bank line's amount and currency; for an item, the whole line's" "the bank's data"]
     [:transaction :how] ["the bank line's channel, in words" (str facts " · state")]
     [:transaction :category] ["the merchant code turned into its name by the scenario's categories; with no code, Jev's guess from the merchant's name (the enrichment call)" (str scenario " · :categories")]
     [:transaction :merchant_country] ["the bank line's country" "the bank's data"]
     [:transaction :merchant_city] ["the bank line's city" "the bank's data"]
     [:transaction :note] ["the note the cardholder typed on the line" "the bank's data"]
     [:transaction :item] ["one item of the receipt or order, scanned apart from the bank line; the rest of it beside" (str scenario " · the case's :items")]
     [:context :paid_with] ["the card's name, from the list of cards" (str scenario " · :cards")]
     [:context :cardholder] ["who holds the card" (str scenario " · :cards")]
     [:context :cardholder_was] ["where the cardholder was that day: the trips, by date (none on record: at home)" (str scenario " · :trips; " facts " · whereabouts")]
     [:context :projects_running] ["the projects whose dates include the day, with what each makes" (str scenario " · :projects")]
     [:context :travellers_of_running_projects] ["each running project's travellers" (str scenario " · :projects :travellers")]
     [:context :cardholder_travels_for_a_running_project] ["whether the cardholder is one of those travellers" (str facts " · enrich")]
     [:context :cardholder_upcoming_trips] ["the cardholder's trips starting in the next 90 days" (str facts " · upcoming-trips")]
     [:context :shop_is_where_cardholder_was] ["for a purchase in person: the shop's city and country against where the cardholder was" (str facts " · enrich")]
     [:context :refund_of] ["for a refund: the earlier purchase it returns, same card, merchant and currency, by amount" (str facts " · refund-of")]
     [:context :earlier_charge_here] ["the latest other charge on the same card at the same merchant in the week before" (str facts " · earlier-here")]
     [:rules] ["the id and text of every rule that could apply (none an exact block rules out); not its exceptions (:over) nor its conditions (:when). Sent in call 1 only" (str scenario " · :rules")]}))

;; ----------------------------------------------------------------------------------------------- pieces

(defui step [{:keys [n title where children]}]
  ($ :div {:class "grid gap-3.5 border-t border-line-1 py-2.5 md:grid-cols-[170px_minmax(0,1fr)]"}
     ($ :div {:class "flex flex-col gap-1"}
        ($ :span {:class "inline-flex size-5 items-center justify-center rounded-full bg-ink-1 font-mono text-2xs font-bold text-ink-inverse"} n)
        ($ :span {:class "text-sm font-bold"} title)
        ($ ui/mono {:class "text-2xs text-ink-3"} where))
     ($ :div {:class "min-w-0"} children)))

(defui where-code [{:keys [children]}]
  ($ :code {:class "font-mono text-2xs text-lapis"} children))

(defui value [{:keys [v]}]
  (cond
    (map? v) ($ :ul {:class "list-disc pl-4"}
                (for [[k x] v] ($ :li {:key (str k)} ($ ui/mono {:class "text-ink-3"} (name k) ": ") ($ value {:v x}))))
    (sequential? v) ($ :ul {:class "list-disc pl-4"}
                       (for [[i x] (map-indexed vector v)]
                         ($ :li {:key i}
                            (if (and (map? x) (:id x))
                              ($ :<> ($ :b {:class "font-mono"} (:id x)) " " (:rule x))
                              ($ value {:v x})))))
    (boolean? v) ($ ui/mono {} (str v))
    :else (str v)))

(defui option-rows
  "A Choice's options as Jev reads them, each with its description and the probability Jev gave it; the buckets the
  answer key accepts are marked."
  [{:keys [criteria probabilities unit]}]
  (let [ok (conj (set (:ok unit)) (:expect unit))
        top (some-> (seq probabilities) (->> (apply max-key val)) key)]
    ($ :div {:class "grid grid-cols-[minmax(150px,200px)_minmax(0,1fr)_96px_38px] items-center gap-x-2.5 gap-y-1 text-sm"}
       (for [h ["option" "what Jev reads it as" "probability" ""]] ($ ui/eyebrow {:key h} h))
       (for [[option desc] (sort-by #(- (get probabilities (key %) 0)) criteria)
             :let [p (get probabilities option 0)
                   c (option-code option)]]
         ($ :<> {:key option}
            ($ :span {:class (when (= option top) "font-bold")} ($ ui/tag {:code c})
               (when (ok c) ($ :span {:class "ml-1 font-mono text-[9.5px] uppercase text-verdigris"}
                               (if (= c (:expect unit)) "should be" "also fine"))))
            ($ :span {:class "text-xs text-ink-2"} desc)
            ($ ui/bar {:p p :wide true})
            ($ ui/mono {:class "text-right text-2xs"} (ui/pct p)))))))

(defn- block-text
  "A rule block in words; a category block names the line's own category when it is one of the block's."
  [{:keys [categories projects]} [op & args] facts]
  (case op
    :category (let [codes (second args)
                    names (map #(get categories % %) (sort codes))]
                (cond
                  (contains? codes (:mcc facts)) (str "category is " (:category facts) " (one of " (count codes) ")")
                  (> (count names) 2) (str "category is one of " (count codes) ": " (first names) ", …")
                  :else (str "category is " (str/join " or " names))))
    :on-trip-for (str "on a trip for " (get-in projects [(first args) :name]))
    :trip (case (first args) :personal "on a personal trip" :project "on a project trip" "on a trip")
    :at-home "at home"
    :card (str "card is " (str/join " or " (sort (second args))))
    :channel (str "paid by " (str/join " or " (map name (second args))))
    :country-mismatch "the shop is not where the cardholder was"
    :present (str "the line has a " (name (first args)))
    :traveller "the cardholder travels for a running project"
    :during-project "a project is running"
    :jev (first args)
    :not (str "not: " (block-text {:categories categories :projects projects} (first args) facts))
    :any (str "any of: " (str/join "; " (map #(block-text {:categories categories :projects projects} % facts) args)))
    (pr-str (into [op] args))))

(defui block-chip [{:keys [world facts block holds]}]
  (let [fuzzy? (some #{:jev} (flatten (seq block)))
        yes (yes? holds)]
    ($ :span {:title (cond (and fuzzy? (nil? holds)) "A yes/no question for Jev with no answer: its rule was not sent, or the line has not been coded since the rules changed"
                           fuzzy? "A yes/no question Jev answered, asked without the rules (call 2)"
                           :else "Exact: code checks it")
              :class (cx "inline-flex items-baseline gap-1.5 rounded-1 border bg-surface-0 px-1.5 py-px text-xs"
                         (if yes "border-verdigris" "border-line-1 text-ink-3") (when fuzzy? "border-dashed"))}
       (block-text world block facts)
       ($ :span {:class (cx "font-mono text-2xs" (when yes "text-verdigris"))} (cond (nil? holds) "not asked" fuzzy? (ui/pct holds) yes "yes" :else "no")))))

(defui rule-rows [{:keys [world facts rows overruled-by]}]
  (let [rules (into {} (map (juxt :id identity)) (:rules world))]
    ($ :<>
       (for [{:keys [id code check rule blocks holds jev]} rows
             :let [applies (yes? holds)
                   by (overruled-by id)
                   status (cond (and applies check) :check (and applies by) :overruled applies :applies :else :no)
                   off (when-not applies "opacity-55")]]
         ($ :<> {:key id}
            ($ :span {:title ({:check "a check that applies: a person must look" :overruled (str "applies, but set aside by " by)
                               :applies "applies" :no "does not apply"} status)
                      :class (cx "inline-block size-2.5 rounded-full"
                                 ({:check "bg-coin" :overruled "ring-2 ring-inset ring-ink-2" :applies "bg-ink-1" :no "bg-surface-2"} status))})
            ($ ui/mono {:class (cx "font-medium" off)} id)
            ($ :span {:class (cx "hidden md:block" off)} ($ ui/tag {:code code}))
            ($ :span {:class off}
               ($ :span {:class "flex flex-wrap gap-1"}
                  (for [{:keys [block holds]} blocks]
                    ($ block-chip {:key (pr-str block) :world world :facts facts :block block :holds holds})))
               ($ :div {:class "mt-0.5 text-xs text-ink-2"} rule
                  (when by ($ :b " Set aside here by " by ", its exception."))
                  (when-let [over (seq (:over (rules id)))] ($ ui/muted {} " Exception to " (str/join ", " over) "."))))
            ($ ui/mono {:class (cx "text-right text-2xs" off)} (if holds (ui/pct holds) "—"))
            ($ ui/mono {:class (cx "hidden text-right text-2xs md:block" off) :title "Jev's own reading of the whole rule, given the rules sent (call 1)"}
               (when jev (ui/pct jev))))))))

(defui rules-grid [{:keys [children]}]
  ($ :div {:class "grid grid-cols-[14px_34px_minmax(0,1fr)_44px] items-baseline gap-x-2.5 gap-y-1 text-sm md:grid-cols-[14px_34px_minmax(130px,170px)_minmax(0,1fr)_54px_54px]"}
     children))

(defui rules-panel
  "Every rule of a line the same way wherever rules are shown: how many apply, the blocks' legend, the rules that
  apply (most first), those that came close (Jev read them as applying, or a block nearly held), and the rest on
  asking, each with why it does not apply: sent and did not apply, or never sent because an exact block rules it
  out. `all` opens on every rule."
  [{:keys [world facts explained all note]}]
  (let [[all? set-all!] (uix/use-state (boolean all))
        rules-by-id (into {} (map (juxt :id identity)) (:rules world))
        {sent true unsent false} (group-by #(not (false? (:sent %))) explained)
        applying (sort-by #(- (:holds %)) (filter #(yes? (:holds %)) sent))
        near (filter #(and (not (yes? (:holds %))) (or (yes? (:jev %)) (>= (or (:holds %) 0) 0.25))) sent)
        rest-rules (remove (set (concat applying near)) sent)
        overruled-by (into {} (for [r applying o (:over (rules-by-id (:id r)))] [o (:id r)]))
        head (fn [text] ($ :<> ($ :span) ($ ui/eyebrow {:class "col-span-3 mt-1.5 md:col-span-5"} text)))]
    ($ :div
       ($ :div {:class "mb-2 flex flex-wrap items-baseline justify-between gap-x-4 gap-y-1"}
          ($ ui/eyebrow {} (count applying) " of " (count explained) (if (= 1 (count explained)) " rule applies" " rules apply")
             " to this line"
             (when (seq unsent) (str " · " (count sent) " sent to Jev")))
          ($ :span {:class "flex flex-wrap gap-x-4 gap-y-1 text-xs text-ink-3"}
             ($ :span {} ($ :span {:class "mr-1 inline-block rounded-1 border border-verdigris px-1"} "solid") "exact block, checked by code")
             ($ :span {} ($ :span {:class "mr-1 inline-block rounded-1 border border-dashed border-line-2 px-1"} "dashed") "fuzzy block, Jev's yes/no")))
       (when note ($ ui/say {:class "mt-0 mb-2"} note))
       ($ rules-grid {}
          ($ :span) ($ ui/eyebrow {} "rule") ($ ui/eyebrow {:class "hidden md:block"} "codes to")
          ($ ui/eyebrow {} "its blocks, and whether each holds") ($ ui/eyebrow {:class "text-right"} "holds")
          ($ ui/eyebrow {:class "hidden text-right md:block"} "Jev")
          (if (seq applying)
            ($ rule-rows {:world world :facts facts :rows applying :overruled-by overruled-by})
            (head "No rule applies: Jev's pick decides, or a person looks"))
          (when (seq near)
            ($ :<>
               (head "Did not apply, but Jev read them as applying (the Jev column), or a block came close")
               ($ rule-rows {:world world :facts facts :rows near :overruled-by overruled-by})))
          (when (and all? (seq rest-rules))
            ($ :<>
               (head "Did not apply")
               ($ rule-rows {:world world :facts facts :rows rest-rules :overruled-by overruled-by})))
          (when (and all? (seq unsent))
            ($ :<>
               (head "Not sent to Jev: an exact block rules it out (the one that says no)")
               ($ rule-rows {:world world :facts facts :rows unsent :overruled-by overruled-by}))))
       (when (or (seq rest-rules) (seq unsent))
         ($ :button {:class "mt-3 text-xs text-lapis hover:underline" :on-click #(set-all! not)}
            (if all? "Show only the rules that apply, or came close"
                (str "Show all " (count explained) " rules, and why the other " (+ (count rest-rules) (count unsent)) " don't apply")))))))

;; ----------------------------------------------------------------------------------------------- levers

(defn- levers-rows [scenario record unit decisions o t]
  (let [{:keys [facts core] sc :scenario} (files scenario record)
        ok (conj (set (:ok unit)) (:expect unit))
        good? #(ok (get-in decisions [% :code]))
        conf (get-in decisions [:choice :confidence])
        signs {:no-rules (and (not (good? :choice)) (not (good? :no-rules)))
               :rules (and (not (good? :choice)) (good? :no-rules))
               :blocks (and (not (good? :choice)) (good? :blocks) (not= :needs-person (get-in decisions [:blocks :code])))
               :check (= :unsafe o)
               :close (< (abs (- conf t)) 0.1)}]
    (for [[k see change where]
          [[:fact "A field in step 3 is missing, wrong, or points the wrong way"
            "How code makes that field, or the scenario's data it comes from (trips, cards, projects)" (str facts ", " sc)]
           [:no-rules "Jev picks a wrong bucket even without the rules (call 2, step 4)"
            "What Jev reads the buckets as (the option descriptions in step 4), or a fact it needs and is not given" (str sc " · :codes")]
           [:rules "Jev is wrong with the rules, right without them"
            "The text of the rule Jev leaned on (step 3, rules; step 5, the Jev column)" (str sc " · :rules")]
           [:blocks "The rule blocks (step 5) are right and the Choice is wrong"
            "The Choice does not see a rule's exceptions or conditions: say them in the rule's text, or send such disagreements to a person"
            (str sc " · :rules; " core " · decide")]
           [:check "A person had to look, but Jev was sure of a bucket"
            "A check rule (:check true): exact blocks where code can tell, or one narrow yes/no question; the :checked strategy lets it stop the line"
            (str sc " · :rules")]
           [:close "Confidence within 10 points of the threshold"
            "It moves a few points between runs of the same questions (up to 25 seen): the threshold, or a rule that settles it"
            "the threshold, above"]
           [:key "The answer key itself looks wrong" "The case's :expect, :ok and :review" (str sc " · :cases")]]]
      {:k k :see see :change change :where where :hit (signs k)})))

;; ----------------------------------------------------------------------------------------------- the walk

(defn- fact-rows
  "What code worked out about the line, as [what, value, where it came from]."
  [world facts responses line-link]
  (let [person #(get-in world [:entities % :name])]
    (cond-> [["category" ($ :<> (or (:category facts) ($ :span {:class "font-semibold text-coin-text"} "none"))
                                                     (some->> (:mcc facts) (str " · ")))
                                       (if (= :enrichment (:category-from facts))
                                         (str "Jev guessed it from the name (" (ui/pct (get-in responses [:enrich :answers :category :confidence])) ")")
                                         "the bank's merchant code")]
                                      ["card" (:card-name facts) "the list of cards"]
                                      ["cardholder was" (str (person (:holder facts)) " was " (where-text world (:whereabouts facts))) "the trips, by the date"]]
                               (= :in-store (:channel facts))
                               (conj ["shop" (if (:country-mismatch facts)
                                               ($ :span {:class "font-semibold text-coin-text"} "not where the cardholder was")
                                               "where the cardholder was")
                                      "the shop's city against the trip"])
                               true
                               (conj ["projects running" (or (not-empty (str/join ", " (map #(get-in world [:projects % :name]) (:projects facts)))) "none")
                                      "the projects' dates"])
                               (seq (:upcoming facts))
                               (conj ["trips coming" (str/join "; " (for [{:keys [city place from project]} (:upcoming facts)]
                                                                      (str (or city place) " from " from
                                                                           (if project (str ", " (get-in world [:projects project :name])) ", personal"))))
                                      "the trips, next 90 days"])
                               (:item facts)
                               (conj ["this item" ($ :<> (:item facts) (when (seq (:other-items facts))
                                                                         ($ ui/muted {} " · beside it: " (str/join ", " (:other-items facts)))))
                                      "the receipt"])
                               (:refund-of facts)
                               (conj (let [r (:refund-of facts)]
                                       ["refunds" ($ :<> (line-link (:id r)) ", " (:date r) ", when the cardholder was " (where-text world (:whereabouts r)))
                                        "an earlier purchase, same card and merchant"]))
                               (:earlier-here facts)
                               (conj (let [e (:earlier-here facts)]
                                       ["just before" ($ :<> (line-link (:id e)) ", " (:date e) ", " (:currency e) " " (ui/money (:amount e))
                                                         (some->> (:note e) (str ", note: ")))
                                        "a charge in the week before, same card and merchant"])))))

(defn- and-list [xs] (if (< (count xs) 2) (str/join xs) (str (str/join ", " (butlast xs)) " and " (last xs))))

(defui came-back
  "What a request's answers were, raw; or, when the record keeps none for it as it is sent now, why."
  [{:keys [response]}]
  (if response
    ($ ui/code-block {:value (:answers response)})
    ($ ui/notice {}
       "Not asked yet. The record has no answer to this request as it is sent now: the rules (or which of them "
       "could apply) changed since the line was coded. Run the scenario to ask it.")))

(defui decision-story
  "How the line's coding was decided, in three sentences: what the rules say, what Jev says, and what the chosen
  strategy and the threshold make of them."
  [{:keys [decisions strategy code confidence t o stale]}]
  (let [{:keys [blocks checked choice no-rules]} decisions
        confidence (or confidence 0)
        one? (= 1 (count (:by blocks)))
        rules-say (case (:reason blocks)
                    :no-rule "No rule applies, so the rules alone don't decide this line."
                    :conflict (str "Rules " (and-list (:by blocks)) " apply but disagree, so the rules alone don't decide.")
                    :check (str (if one? "Check " "Checks ") (and-list (:by blocks)) (if one? " applies" " apply")
                                ": the rules say a person must look.")
                    ($ :<> (if one? "Rule " "Rules ") (and-list (:by blocks)) (if one? " applies" " apply")
                       (when (seq (:overruled blocks)) (str " (setting aside " (and-list (:overruled blocks)) ", its exception)"))
                       ", so the rules say " ($ ui/tag {:code (:code blocks)}) ", " ($ ui/mono {} (ui/pct (:confidence blocks)))
                       " sure: as sure as the least certain block they rest on."))
        jev-say ($ :<> "Given the facts and those rules, Jev picks " ($ ui/tag {:code (:code choice)}) " ("
                   ($ ui/mono {} (ui/pct (:confidence choice))) ")."
                   (if (= (:code choice) (:code no-rules))
                     " From the facts alone it picks the same."
                     ($ :<> " From the facts alone it would pick " ($ ui/tag {:code (:code no-rules)}) " ("
                        ($ ui/mono {} (ui/pct (:confidence no-rules))) "): the rules changed its mind.")))
        how (case strategy
              :checked (if (= :check (:reason checked))
                         (str "check " (and-list (:by checked)) " applies, so a person looks whatever Jev picks")
                         "no check applies, so Jev's pick stands")
              :choice "it takes Jev's pick"
              :reversed "it takes Jev's pick with the options asked in reverse order"
              :no-rules "it takes Jev's pick from the facts alone"
              :rule-nouls "it combines Jev's yes or no on each rule"
              :blocks "it takes what the rules say"
              :hybrid (if (#{:no-rule :conflict} (:reason blocks))
                        "the rules don't decide, so it takes Jev's pick"
                        "the rules decide, so it takes what they say")
              "it decides")
        so (cond
             (= :needs-person code) " That means a person looks."
             (>= confidence t) ($ :<> " At " ($ ui/mono {} (ui/pct confidence)) " it clears the " ($ ui/mono {} (ui/pct t))
                                  " bar, so the line is coded " ($ ui/tag {:code code}) " with no one looking.")
             :else ($ :<> " At " ($ ui/mono {} (ui/pct confidence)) " it is under the " ($ ui/mono {} (ui/pct t))
                      " bar to code a line alone, so a person looks, with " ($ ui/tag {:code code}) " as the suggestion."))]
    ($ :div
       (when stale
         ($ ui/notice {} "This is the last run's decision, made before the rules changed. Run the scenario to decide it as "
            "they stand now."))
       ($ ui/panel {:class "grid grid-cols-[64px_minmax(0,1fr)] gap-x-3 gap-y-2.5 px-3 py-2.5 text-sm leading-relaxed"}
          ($ ui/eyebrow {:class "pt-0.5"} "The rules") ($ :p {} rules-say)
          ($ ui/eyebrow {:class "pt-0.5"} "Jev") ($ :p {} jev-say)
          ($ ui/eyebrow {:class "pt-0.5"} "So")
          ($ :p {} "This view codes by " ($ :b {} (some (fn [[k n]] (when (= k strategy) n)) strategies)) ": " how "." so
             " " ($ ui/pill {:outcome o}))))))

(defui walk-body [{:keys [scenario row strategy t data]}]
  (let [world (ui/use-world)
        {:keys [unit line facts requests enrich rebuilt responses decisions explained note]} data
        {:keys [code confidence]} (get decisions strategy)
        o (outcome/outcome unit (get decisions strategy) t)
        rules-by-id (into {} (map (juxt :id identity)) (:rules world))
        applying (filter #(yes? (:holds %)) explained)
        near (filter #(and (not (yes? (:holds %))) (or (yes? (:jev %)) (>= (or (:holds %) 0) 0.25))) explained)
        overruled-by (into {} (for [r applying o (:over (rules-by-id (:id r)))] [o (:id r)]))
        sources (field-sources scenario (:record data))
        line-link (fn [id] ($ :a {:class "font-mono text-lapis hover:underline"
                                  :href (route/href ["sandbox" scenario (get-in data [:run :id])]
                                                    (assoc (:query (route/parse js/location.hash)) :line id))} id))]
    ($ :div {:class "border-t border-dashed border-line-1 bg-surface-1 px-3 pt-3 pb-4 md:pl-8"}
       ($ :div {:class "flex flex-wrap items-center gap-x-3.5 gap-y-1.5 pb-2.5 text-sm"}
          (if (:record data)
            ($ :div {:class "w-full border-b border-line-1 pb-2.5"} ($ key/confirm {:unit unit :chosen code :world world}))
            ($ :<> ($ ui/eyebrow {} "Should be") ($ ui/tag {:code (:expect unit)})))
          (when (seq (:ok unit))
            ($ ui/muted {} "also fine " (for [c (:ok unit)] ($ :span {:key (name c) :class "mr-1"} ($ ui/tag {:code c})))))
          (when-not (:record data)
            (case (:review unit) :required ($ ui/eyebrow {} "· a person must look") :allowed ($ ui/muted {} "· a person may look") nil))
          (when-not (:record data) ($ ui/muted {} "·")) ($ ui/eyebrow {} "Chosen") ($ ui/tag {:code code}) ($ ui/mono {} (ui/pct confidence))
          ($ ui/muted {} "· automatic at " (ui/pct t) " →") ($ ui/pill {:outcome o}))
       (when-let [w (or (:hard unit) (:why unit))]
         ($ :div {:class "mb-2.5 border-l-2 border-line-2 pl-2 text-sm text-ink-2"}
            ($ ui/eyebrow {:class "mr-1.5 inline"} (if (:hard unit) "Why it is hard" "Why")) w))
       (when-let [{:keys [why fix]} note]
         ($ :div {:class (cx "mb-3 grid grid-cols-[110px_1fr] gap-x-3 gap-y-1 border border-l-[3px] border-line-1 bg-surface-0 px-3 py-2.5 text-sm leading-normal"
                             (if (ui/bad? (:outcome row)) "border-l-coin" "border-l-gilt"))}
            ($ ui/eyebrow {:class "pt-0.5"} "Why it failed") ($ :span why)
            (when fix ($ :<> ($ ui/eyebrow {:class "pt-0.5"} "What would fix it") ($ :span fix)))))

       ($ step {:n 1 :title "The bank line" :where "as the bank sends it"}
          ($ :div {:class "grid gap-4 md:grid-cols-2"}
             ($ :dl {:class "grid grid-cols-[58px_minmax(0,1fr)_58px_minmax(0,1fr)] items-baseline gap-x-2.5 gap-y-1 text-sm"}
                (for [k [:date :card :merchant :mcc :amount :currency :country :city :channel :note]
                      :let [v (get line k) wide (#{:merchant :note} k)] :when (some? v)]
                  ($ :<> {:key (name k)}
                     ($ :dt {:class (cx "font-mono text-2xs text-ink-3" (when wide "col-start-1"))} (name k))
                     ($ :dd {:class (cx "min-w-0 break-words" (when wide "col-span-3"))}
                        (cond (number? v) (ui/money v) (keyword? v) (name v) :else (str v))))))
             (when-let [items (:items line)]
               ($ :div
                  ($ ui/eyebrow {} "The " (name (:items-from line)) ", scanned apart from the bank line: one item is coded at a time")
                  ($ :ul {:class "mt-2 text-sm"}
                     (for [{:keys [item amount]} items]
                       ($ :li {:key item :class (cx "flex justify-between gap-2.5 border-b border-dotted border-line-1 py-0.5"
                                                    (when (= item (:item unit)) "font-bold"))}
                          ($ :span item (when (= item (:item unit)) "  ← this one"))
                          ($ ui/mono {} (ui/money amount)))))))))

       ($ step {:n 2 :title "What code works out" :where "facts/enrich: exact, no model"}
          ($ :div {:class "grid grid-cols-[100px_minmax(0,1fr)] items-baseline gap-x-3 gap-y-1 text-sm md:grid-cols-[120px_minmax(0,1fr)_170px]"}
             (for [[k v src] (fact-rows world facts responses line-link)]
               ($ :<> {:key k}
                  ($ ui/mono {:class "text-2xs text-ink-3"} k)
                  ($ :span {} v)
                  ($ ui/mono {:class "hidden text-2xs text-ink-3 md:block"} src))))
          (when enrich
            ($ ui/disclosure {:summary "The enrichment call, which came first: Jev guessed the category from the name"}
               ($ :div {:class "grid gap-2.5 md:grid-cols-2"}
                  ($ :div ($ ui/eyebrow {} "Sent") ($ ui/code-block {:value enrich}))
                  ($ :div ($ ui/eyebrow {} "Came back") ($ ui/code-block {:value (get-in responses [:enrich :answers])}))))))

       ($ step {:n 3 :title "What Jev reads" :where "facts/state: the state, exactly as sent"}
          ($ ui/say {:class "mt-0 mb-2"}
             "This is everything Jev knows about the line: no other line, no answer key, nothing else. Both calls send "
             "the same state; call 1 adds the rules. Read it as Jev would: is anything missing, wrong, or worded so it "
             "points the wrong way?")
          (cond
            (seq (:unanswered data))
            ($ ui/notice {:tone :bad}
               "The record keeps no answer to " (str/join " and " (map #(str "the " (name %) " request") (:unanswered data)))
               " as the scenario stands now: it changed since this line was coded (a rule, or the order of the rules). "
               "The coding shown is the last run's. Run the scenario again to ask; its page says what that costs.")
            (:record data) ($ ui/notice {} "Rebuilt from the line's facts as the scenario stands; Jev's answers are the ones the record kept for exactly these requests.")
            rebuilt ($ ui/notice {} "This run did not keep its requests: these are rebuilt from today's code and scenario."))
          (let [state (get-in requests [:rules :state])]
            ($ :div {:class "grid border border-line-1 bg-surface-0 text-sm md:grid-cols-[minmax(150px,240px)_minmax(0,1.3fr)_minmax(180px,1fr)]"}
               (for [sec [:transaction :context]]
                 ($ :<> {:key (name sec)}
                    ($ ui/eyebrow {:class "border-t border-line-1 bg-surface-1 px-2 py-1 md:col-span-3"}
                       (name sec) " — " (if (= :transaction sec) "the line itself" "what code adds about it"))
                    (for [[k v] (get state sec)
                          :let [[how where] (sources [sec k] ["" ""])]]
                      ($ :<> {:key (name k)}
                         ($ :div {:class "min-w-0 break-words border-t border-line-1 px-2 py-1 font-mono text-2xs text-ink-2"} (name sec) "." (name k))
                         ($ :div {:class "min-w-0 break-words px-2 py-1 md:border-t md:border-line-1"} ($ value {:v v}))
                         ($ :div {:class "min-w-0 px-2 py-1 text-2xs text-ink-3 md:border-t md:border-line-1"}
                            how (when (seq where) ($ :<> " · " ($ where-code {} where))))))))
               (when-let [rules (:rules state)]
                 (let [[how where] (sources [:rules])]
                   ($ :<>
                      ($ ui/eyebrow {:class "border-t border-line-1 bg-surface-1 px-2 py-1 md:col-span-3"} "rules — sent with call 1 only")
                      ($ :div {:class "border-t border-line-1 px-2 py-1 font-mono text-2xs text-ink-2"} "rules")
                      ($ :div {:class "px-2 py-1 text-2xs text-ink-3 md:col-span-2 md:border-t md:border-line-1"} how " · " ($ where-code {} where))
                      ($ :div {:class "border-t border-line-1 px-2 py-2 md:col-span-3"}
                         ($ rules-panel {:world world :facts facts :explained explained :all true
                                         :note (str (ui/count-of rules "rule is" "rules are") " sent, each its id and text: those no exact "
                                                    "block rules out for this line. Which apply is worked out by code from their "
                                                    "blocks (step 5), not by Jev.")})))))))
          ($ :div {:class "mt-2 border-l-2 border-line-2 pl-2 text-sm text-ink-2"}
             ($ :b "Not sent to Jev")
             ($ :ul {:class "list-disc pl-4"}
                ($ :li "the answer key, and why the line is hard")
                ($ :li "each rule's exceptions (" ($ :code ":over") ") and its conditions (" ($ :code ":when") "): those are applied by code, in step 5")
                ($ :li "the other lines on the card, except a refund's purchase or a charge in the week before, when found")
                ($ :li "trips more than 90 days ahead, and anything about the cardholder beyond their trips")))
          ($ ui/disclosure {:summary "The raw state, as JSON"} ($ ui/code-block {:value (get-in requests [:rules :state])})))

       ($ step {:n 4 :title "What Jev is asked, and what it said" :where "core/requests: the questions, exactly as sent"}
          (let [code-q (get-in requests [:rules :questions :code])
                plain-q (get-in requests [:plain :questions :code])]
            ($ :<>
               ($ ui/panel {:class "mb-2.5 px-2.5 py-2"}
                  ($ ui/eyebrow {} "Call 1, the Choice that decides: given the state and the rules")
                  ($ :div {:class "my-2 border-l-2 border-ink-1 pl-2 text-sm leading-normal"} (:instructions code-q))
                  ($ option-rows {:criteria (:criteria code-q) :probabilities (get-in responses [:rules :answers :code :probabilities]) :unit unit})
                  ($ ui/say {} "Asked again with the options in reverse order, it chose " ($ ui/tag {:code (get-in decisions [:reversed :code])})
                     " " ($ ui/mono {} (ui/pct (get-in decisions [:reversed :confidence]))) ". In the same call, one yes/no question per "
                     "rule (\"does this rule apply?\"): its answers are the Jev column in step 5."))
               ($ ui/panel {:class "px-2.5 py-2"}
                  ($ :details
                     ($ :summary {:class "cursor-pointer"} ($ ui/eyebrow {:class "inline"} "Call 2, the same Choice without the rules: what the rules add"))
                     ($ :div {:class "my-2 border-l-2 border-ink-1 pl-2 text-sm leading-normal"} (:instructions plain-q))
                     ($ option-rows {:criteria (:criteria plain-q) :probabilities (get-in responses [:plain :answers :code :probabilities]) :unit unit})
                     ($ ui/say {} "Call 2 also asks the yes/no question in each dashed block of step 5, with the state alone."))))))

       ($ step {:n 5 :title "The rules, applied" :where "blocks/holds: each block checked"}
          ($ rules-panel {:world world :facts facts :explained explained})
          ($ ui/disclosure {:summary "Call 2, raw: the yes/no questions in the dashed blocks, sent without the rules"}
             ($ :div {:class "grid gap-2.5 md:grid-cols-2"}
                ($ :div ($ ui/eyebrow {} "Sent") ($ ui/code-block {:value (assoc (get requests :plain) :state "… the state in step 3")}))
                ($ :div ($ ui/eyebrow {} "Came back") ($ came-back {:response (:plain responses)})))))

       ($ step {:n 6 :title "The decision" :where "core/combine · the Choice · the threshold"}
          ($ decision-story {:decisions decisions :strategy strategy :code code :confidence confidence :t t :o o
                             :stale (seq (:unanswered data))})
          ($ ui/disclosure {:summary "Call 1, raw: the payload with the rules (the state is in step 3), and what came back"}
             ($ :div {:class "grid gap-2.5 md:grid-cols-2"}
                ($ :div ($ ui/eyebrow {} "Sent") ($ ui/code-block {:value (update (get requests :rules) :state dissoc :transaction :context)}))
                ($ :div ($ ui/eyebrow {} "Came back") ($ came-back {:response (:rules responses)})))))

       ($ step {:n 7 :title "Where to change it" :where "the levers, by what you see"}
          ($ :div {:class "grid border border-line-1 bg-surface-0 text-sm md:grid-cols-[minmax(200px,1fr)_minmax(200px,1.2fr)_minmax(160px,.8fr)]"}
             (for [h ["When you see" "What to change" "Where"]]
               ($ ui/eyebrow {:key h :class "hidden bg-surface-1 px-2 py-1.5 md:block"} h))
             (for [{:keys [k see change where hit]} (levers-rows scenario (:record data) unit decisions o t)
                   :let [c (cx "border-t border-line-1 px-2 py-1.5" (when hit "bg-gilt-tint"))]]
               ($ :<> {:key (name k)}
                  ($ :div {:class c} see (when hit ($ :b " — this line")))
                  ($ :div {:class c} change)
                  ($ :div {:class c} ($ where-code {} where)))))))))

(defui walk
  "A line's walk-through, fetched when its row opens."
  [{:keys [scenario run row strategy t]}]
  (let [id (:id row)
        state (api/use-get (str "/sandbox/scenarios/" scenario "/runs/" run "/lines/" id))]
    (uix/use-effect
     (fn [] (some-> (js/document.getElementById id) (.scrollIntoView #js {:block "nearest"})))
     [id])
    ($ :div
       ($ ui/loading {:state state}
          ($ walk-body {:scenario scenario :row row :strategy strategy :t t :data (:data state)})))))
