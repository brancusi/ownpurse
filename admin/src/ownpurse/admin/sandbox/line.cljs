(ns ownpurse.admin.sandbox.line
  "One line's way through, in seven steps: the bank line; what code works out; what Jev reads (the state exactly as
  sent, field by field, with how code made each field and where to change it); what Jev is asked (the questions word
  for word, every option as Jev reads it, with its probability); the rules applied; the decision; and where to
  change it. The line's data come from the API when it opens."
  (:require
   [clojure.string :as str]
   [ownpurse.admin.api :as api]
   [ownpurse.admin.route :as route]
   [ownpurse.admin.ui :as ui :refer [cx]]
   [ownpurse.classify.outcome :as outcome]
   [uix.core :as uix :refer [$ defui]]))

(defn- yes? [p] (>= (or p 0) 0.5))

(defn- option-code [option] (keyword (str/replace option "_" "-")))

(defn- where-text [{:keys [projects]} {:keys [place city trip project day days]}]
  (let [at (if city (str city ", " place) place)]
    (case trip
      :project (str "in " at ", on a trip for " (get-in projects [project :name]) ", day " day " of " days)
      :personal (str "in " at ", on a personal trip")
      (str "at home in " at))))

(defn- files
  "The files a lever or a field names, by short name."
  [scenario]
  {:facts "facts.clj" :core "core.clj" :scenario (str scenario ".edn")})

(defn- field-sources
  "How code made each field of the state Jev reads, and where to change it: [how, where]."
  [scenario]
  (let [{:keys [facts scenario]} (files scenario)]
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
     [:rules] ["every rule's id and text; not its exceptions (:over) nor its conditions (:when). Sent in call 1 only" (str scenario " · :rules")]}))

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
        top (key (apply max-key val probabilities))]
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
    ($ :span {:title (if fuzzy? "A yes/no question Jev answered, asked without the rules (call 2)" "Exact: code checks it")
              :class (cx "inline-flex items-baseline gap-1.5 rounded-1 border bg-surface-0 px-1.5 py-px text-xs"
                         (if yes "border-verdigris" "border-line-1 text-ink-3") (when fuzzy? "border-dashed"))}
       (block-text world block facts)
       ($ :span {:class (cx "font-mono text-2xs" (when yes "text-verdigris"))} (if fuzzy? (ui/pct holds) (if yes "yes" "no"))))))

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
            ($ ui/mono {:class (cx "text-right text-2xs" off)} (ui/pct holds))
            ($ ui/mono {:class (cx "hidden text-right text-2xs md:block" off) :title "Jev's own reading of the whole rule, given every rule (call 1)"}
               (when jev (ui/pct jev))))))))

(defui rules-grid [{:keys [children]}]
  ($ :div {:class "grid grid-cols-[14px_34px_minmax(0,1fr)_44px] items-baseline gap-x-2.5 gap-y-1 text-sm md:grid-cols-[14px_34px_minmax(130px,170px)_minmax(0,1fr)_54px_54px]"}
     children))

;; ----------------------------------------------------------------------------------------------- levers

(defn- levers-rows [scenario unit decisions o t]
  (let [{:keys [facts core] sc :scenario} (files scenario)
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

(defui walk-body [{:keys [scenario row strategy t data]}]
  (let [world (ui/use-world)
        {:keys [unit line facts requests enrich rebuilt responses decisions explained note]} data
        {:keys [code confidence]} (get decisions strategy)
        o (outcome/outcome unit (get decisions strategy) t)
        rules-by-id (into {} (map (juxt :id identity)) (:rules world))
        applying (filter #(yes? (:holds %)) explained)
        near (filter #(and (not (yes? (:holds %))) (or (yes? (:jev %)) (>= (:holds %) 0.25))) explained)
        overruled-by (into {} (for [r applying o (:over (rules-by-id (:id r)))] [o (:id r)]))
        sources (field-sources scenario)
        line-link (fn [id] ($ :a {:class "font-mono text-lapis hover:underline"
                                  :href (route/href ["sandbox" scenario (get-in data [:run :id])]
                                                    (assoc (:query (route/parse js/location.hash)) :line id))} id))]
    ($ :div {:class "border-t border-dashed border-line-1 bg-surface-1 px-3 pt-3 pb-4 md:pl-8"}
       ($ :div {:class "flex flex-wrap items-center gap-x-3.5 gap-y-1.5 pb-2.5 text-sm"}
          ($ ui/eyebrow {} "Should be") ($ ui/tag {:code (:expect unit)})
          (when (seq (:ok unit))
            ($ ui/muted {} "also fine " (for [c (:ok unit)] ($ :span {:key (name c) :class "mr-1"} ($ ui/tag {:code c})))))
          (case (:review unit) :required ($ ui/eyebrow {} "· a person must look") :allowed ($ ui/muted {} "· a person may look") nil)
          ($ ui/muted {} "·") ($ ui/eyebrow {} "Chosen") ($ ui/tag {:code code}) ($ ui/mono {} (ui/pct confidence))
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
          (when rebuilt ($ ui/notice {} "This run did not keep its requests: these are rebuilt from today's code and scenario."))
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
                      ($ :div {:class "px-2 py-1 md:border-t md:border-line-1"}
                         ($ :details ($ :summary {:class "cursor-pointer"} (count rules) " rules, each its id and text") ($ value {:v rules})))
                      ($ :div {:class "px-2 py-1 text-2xs text-ink-3 md:border-t md:border-line-1"} how " · " ($ where-code {} where)))))))
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
          ($ rules-grid {}
             ($ :span) ($ ui/eyebrow {} "rule") ($ ui/eyebrow {:class "hidden md:block"} "codes to")
             ($ ui/eyebrow {} "its blocks, and whether each holds") ($ ui/eyebrow {:class "text-right"} "holds")
             ($ ui/eyebrow {:class "hidden text-right md:block"} "Jev")
             ($ rule-rows {:world world :facts facts :rows (sort-by #(- (:holds %)) applying) :overruled-by overruled-by})
             (when (seq near)
               ($ :<>
                  ($ :span)
                  ($ ui/eyebrow {:class "col-span-3 mt-1.5 md:col-span-5"} "Did not apply, but Jev read them as applying (the Jev column), or a block came close")
                  ($ rule-rows {:world world :facts facts :rows near :overruled-by overruled-by}))))
          (let [rest-rules (remove (set (concat applying near)) explained)]
            ($ ui/disclosure {:summary (str (count rest-rules) " rules did not apply")}
               ($ rules-grid {} ($ rule-rows {:world world :facts facts :rows rest-rules :overruled-by overruled-by}))))
          ($ ui/disclosure {:summary "Call 2, raw: the yes/no questions in the dashed blocks, sent without the rules"}
             ($ :div {:class "grid gap-2.5 md:grid-cols-2"}
                ($ :div ($ ui/eyebrow {} "Sent") ($ ui/code-block {:value (assoc (get requests :plain) :state "… the state in step 3")}))
                ($ :div ($ ui/eyebrow {} "Came back") ($ ui/code-block {:value (get-in responses [:plain :answers])})))))

       ($ step {:n 6 :title "The decision" :where "core/combine · the Choice · the threshold"}
          (let [blocks (:blocks decisions) checked (:checked decisions)]
            ($ :div {:class "grid gap-3.5 md:grid-cols-3"}
               ($ ui/panel {:class "px-2.5 py-2"}
                  ($ ui/eyebrow {} "The rule blocks conclude")
                  ($ ui/say {} ($ ui/tag {:code (:code blocks)}) " "
                     ($ ui/muted {}
                        (case (:reason blocks)
                          :conflict (str "rules that apply disagree: " (str/join ", " (:by blocks)))
                          :no-rule "no rule applies"
                          :check (str "a check applies: " (str/join ", " (:by blocks)))
                          (str "by " (str/join ", " (:by blocks))
                               (when (seq (:overruled blocks)) (str "; set aside " (str/join ", " (:overruled blocks))))))))
                  ($ ui/say {} ($ ui/muted {} "Their confidence is the least certain rule's: ") ($ ui/mono {} (ui/pct (:confidence blocks)))))
               ($ ui/panel {:class "px-2.5 py-2"}
                  ($ ui/eyebrow {} "Jev's Choice, given the facts and every rule (call 1)")
                  ($ :div {:class "mt-1 grid grid-cols-[minmax(0,1fr)_96px_34px] items-center gap-x-2 gap-y-1 text-sm"}
                     (for [[option p] (take 5 (sort-by val > (get-in responses [:rules :answers :code :probabilities])))]
                       ($ :<> {:key option} ($ :span ($ ui/tag {:code (option-code option)})) ($ ui/bar {:p p :wide true})
                          ($ ui/mono {:class "text-right text-2xs"} (ui/pct p)))))
                  ($ ui/say {} ($ ui/muted {} "Without the rules (call 2) it chose ") ($ ui/tag {:code (get-in decisions [:no-rules :code])})
                     " " ($ ui/mono {} (ui/pct (get-in decisions [:no-rules :confidence])))))
               ($ ui/panel {:class "px-2.5 py-2"}
                  ($ ui/eyebrow {} "Checks, then the threshold")
                  ($ ui/say {} (if (= :check (:reason checked))
                                 (str "A check holds (" (str/join ", " (:by checked)) "): a person looks, whatever Jev chose.")
                                 "No check holds, so Jev's Choice stands."))
                  ($ ui/say {} "Confidence " ($ ui/mono {} (ui/pct confidence)) (if (>= confidence t) " ≥ " " < ") ($ ui/mono {} (ui/pct t))
                     (if (and (not= :needs-person code) (>= confidence t)) ": coded automatically." ": sent to a person."))
                  ($ ui/say {} ($ ui/pill {:outcome o})))))
          ($ ui/disclosure {:summary "Call 1, raw: the payload with the rules (the state is in step 3), and what came back"}
             ($ :div {:class "grid gap-2.5 md:grid-cols-2"}
                ($ :div ($ ui/eyebrow {} "Sent") ($ ui/code-block {:value (update (get requests :rules) :state dissoc :transaction :context)}))
                ($ :div ($ ui/eyebrow {} "Came back") ($ ui/code-block {:value (get-in responses [:rules :answers])})))))

       ($ step {:n 7 :title "Where to change it" :where "the levers, by what you see"}
          ($ :div {:class "grid border border-line-1 bg-surface-0 text-sm md:grid-cols-[minmax(200px,1fr)_minmax(200px,1.2fr)_minmax(160px,.8fr)]"}
             (for [h ["When you see" "What to change" "Where"]]
               ($ ui/eyebrow {:key h :class "hidden bg-surface-1 px-2 py-1.5 md:block"} h))
             (for [{:keys [k see change where hit]} (levers-rows scenario unit decisions o t)
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
